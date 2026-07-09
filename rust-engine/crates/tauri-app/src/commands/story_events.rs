//! Project story event catalog commands for authoring and runtime diagnostics.

use tauri::State;

use crate::state::AppState;
use crate::story_events::{
    StoryEventAction, StoryEventCatalog, StoryEventCatalogSnapshot, StoryEventDefinition,
};
use crate::story_progress::{StoryEventApplication, StoryProgressSnapshot};

#[tauri::command]
pub async fn get_story_event_catalog(
    state: State<'_, AppState>,
) -> Result<StoryEventCatalogSnapshot, String> {
    Ok(state.story_event_catalog.read().await.snapshot())
}

#[tauri::command]
pub async fn get_story_progress(
    state: State<'_, AppState>,
) -> Result<StoryProgressSnapshot, String> {
    Ok(state.story_progress.read().await.snapshot())
}

#[tauri::command]
pub async fn reload_story_event_catalog(
    state: State<'_, AppState>,
) -> Result<StoryEventCatalogSnapshot, String> {
    reload_story_event_catalog_inner(&state).await
}

async fn reload_story_event_catalog_inner(
    state: &AppState,
) -> Result<StoryEventCatalogSnapshot, String> {
    let project_root = state.current_project_data_root().await;
    let catalog = StoryEventCatalog::load_from_project_root(&project_root)?;
    let character_ids = state.character_manager.read().await.character_ids();
    catalog.validate_character_references(character_ids.iter().map(String::as_str))?;
    let snapshot = catalog.snapshot();
    *state.story_event_catalog.write().await = catalog;
    Ok(snapshot)
}

pub(crate) async fn apply_story_event_definition(
    state: &AppState,
    definition: &StoryEventDefinition,
    character_id: Option<&str>,
    catalog_fingerprint: &str,
) -> Result<StoryEventApplication, String> {
    let mut progress = state.story_progress.write().await;
    let mut next_progress = progress.clone();
    let mut application =
        next_progress.apply_event(definition, character_id, catalog_fingerprint)?;
    if !application.applied {
        return Ok(application);
    }

    let script_engine = state.script_engine.read().await;
    for result in &mut application.action_results {
        let StoryEventAction::SetFlag { flag, value } = &result.action else {
            continue;
        };
        let previous = script_engine.has_flag(flag);
        script_engine
            .set_flag(flag, *value)
            .map_err(|error| format!("Failed to apply story event flag `{flag}`: {error}"))?;
        result.changed = previous != *value;
    }
    drop(script_engine);

    *progress = next_progress;
    Ok(application)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_story_event_command_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[tokio::test]
    async fn rejected_reload_leaves_active_catalog_unchanged() {
        let state = AppState::new();
        let root = temp_root("atomic");
        std::fs::create_dir_all(root.join("events")).unwrap();
        std::fs::write(
            root.join("events").join("custom.json"),
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"unknown_character_event",
                "event_type":"unlock",
                "description":"Unknown character",
                "character_ids":["missing"]
              }]
            }"#,
        )
        .unwrap();
        state.set_project_data_root(root.clone()).await;
        let before = state.story_event_catalog.read().await.snapshot();

        let result = reload_story_event_catalog_inner(&state).await;

        assert!(result.is_err());
        assert_eq!(
            state
                .story_event_catalog
                .read()
                .await
                .snapshot()
                .catalog_fingerprint,
            before.catalog_fingerprint
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn applying_event_updates_progress_and_script_flags_once() {
        let state = AppState::new();
        let mut definition = state
            .story_event_catalog
            .read()
            .await
            .definition("first_friend", None)
            .unwrap()
            .clone();
        definition.actions.push(StoryEventAction::SetFlag {
            flag: "story.first_friend".to_string(),
            value: true,
        });
        let fingerprint = state
            .story_event_catalog
            .read()
            .await
            .catalog_fingerprint()
            .to_string();

        let first = apply_story_event_definition(&state, &definition, Some("sakura"), &fingerprint)
            .await
            .unwrap();
        let second =
            apply_story_event_definition(&state, &definition, Some("sakura"), &fingerprint)
                .await
                .unwrap();

        assert!(first.applied);
        assert!(!second.applied);
        assert!(state
            .script_engine
            .read()
            .await
            .has_flag("story.first_friend"));
        assert!(state
            .story_progress
            .read()
            .await
            .unlocked_scene_ids
            .contains("friend_scene"));
    }
}
