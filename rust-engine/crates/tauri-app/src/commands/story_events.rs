//! Project story event catalog commands for authoring and runtime diagnostics.

use tauri::State;

use crate::state::AppState;
use crate::story_events::{StoryEventCatalog, StoryEventCatalogSnapshot};

#[tauri::command]
pub async fn get_story_event_catalog(
    state: State<'_, AppState>,
) -> Result<StoryEventCatalogSnapshot, String> {
    Ok(state.story_event_catalog.read().await.snapshot())
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
}
