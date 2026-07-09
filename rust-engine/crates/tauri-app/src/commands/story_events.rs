//! Project story event catalog commands for authoring and runtime diagnostics.

use std::path::{Path, PathBuf};

use serde_json::Value;
use tauri::State;

use crate::state::AppState;
use crate::story_access::{story_content_access_snapshot, StoryContentAccessSnapshot};
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
pub async fn get_story_content_access(
    state: State<'_, AppState>,
) -> Result<StoryContentAccessSnapshot, String> {
    let catalog = state.story_event_catalog.read().await;
    let progress = state.story_progress.read().await;
    Ok(story_content_access_snapshot(&catalog, &progress))
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

/// Validate and atomically replace the active project's single event catalog document.
#[tauri::command]
pub async fn save_story_event_catalog(
    state: State<'_, AppState>,
    document: Value,
    expected_catalog_fingerprint: String,
) -> Result<StoryEventCatalogSnapshot, String> {
    save_story_event_catalog_inner(&state, document, &expected_catalog_fingerprint).await
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

async fn save_story_event_catalog_inner(
    state: &AppState,
    document: Value,
    expected_catalog_fingerprint: &str,
) -> Result<StoryEventCatalogSnapshot, String> {
    let project_root = state.current_project_data_root().await;
    let event_directory = StoryEventCatalog::project_event_directory(&project_root)?;
    let target_path = editable_event_document_path(&event_directory)?;
    let source_path = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("events/{name}"))
        .ok_or_else(|| "Story event document path is not valid UTF-8.".to_string())?;
    let mut content = serde_json::to_string_pretty(&document)
        .map_err(|error| format!("Unable to serialize story event catalog: {error}"))?;
    content.push('\n');
    let candidate = StoryEventCatalog::from_document_json(&content, &source_path)?;
    let character_ids = state.character_manager.read().await.character_ids();
    candidate.validate_character_references(character_ids.iter().map(String::as_str))?;
    validate_story_event_content_references(&project_root, &candidate)?;

    let mut active_catalog = state.story_event_catalog.write().await;
    if active_catalog.catalog_fingerprint() != expected_catalog_fingerprint {
        return Err(format!(
            "Story event catalog changed since it was opened; expected `{expected_catalog_fingerprint}`, current `{}`. Reload before saving.",
            active_catalog.catalog_fingerprint()
        ));
    }

    tokio::fs::create_dir_all(&event_directory)
        .await
        .map_err(|error| {
            format!(
                "Failed to create story event directory `{}`: {error}",
                event_directory.display()
            )
        })?;
    ensure_event_directory_inside_project(&project_root, &event_directory)?;
    if target_path.exists() {
        let metadata = std::fs::symlink_metadata(&target_path).map_err(|error| {
            format!(
                "Failed to inspect story event document `{}`: {error}",
                target_path.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Story event document must be a regular file: {}",
                target_path.display()
            ));
        }
    }

    let staged = staged_event_paths(&target_path)?;
    write_staged_event_document(&target_path, &staged.0, &staged.1, content.as_bytes()).await?;

    let loaded = match StoryEventCatalog::load_from_project_root(&project_root) {
        Ok(catalog) => catalog,
        Err(error) => {
            rollback_staged_event_document(&target_path, &staged.1).await?;
            return Err(format!(
                "Saved story event catalog failed project reload and was rolled back: {error}"
            ));
        }
    };
    if let Err(error) =
        loaded.validate_character_references(character_ids.iter().map(String::as_str))
    {
        rollback_staged_event_document(&target_path, &staged.1).await?;
        return Err(format!(
            "Saved story event catalog failed character validation and was rolled back: {error}"
        ));
    }
    if loaded.catalog_fingerprint() != candidate.catalog_fingerprint() {
        rollback_staged_event_document(&target_path, &staged.1).await?;
        return Err(
            "Saved story event catalog fingerprint changed during replacement; the original was restored."
                .to_string(),
        );
    }

    cleanup_event_backup(&staged.1).await;
    let snapshot = loaded.snapshot();
    *active_catalog = loaded;
    Ok(snapshot)
}

fn editable_event_document_path(event_directory: &Path) -> Result<PathBuf, String> {
    if !event_directory.exists() {
        return Ok(event_directory.join("story_events.json"));
    }
    let metadata = std::fs::symlink_metadata(event_directory).map_err(|error| {
        format!(
            "Failed to inspect story event directory `{}`: {error}",
            event_directory.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "Story event directory must be a regular directory: {}",
            event_directory.display()
        ));
    }

    let mut documents = std::fs::read_dir(event_directory)
        .map_err(|error| {
            format!(
                "Failed to read story event directory `{}`: {error}",
                event_directory.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    documents.sort();
    match documents.as_slice() {
        [] => Ok(event_directory.join("story_events.json")),
        [document] => Ok(document.clone()),
        _ => Err(
            "The visual Story Event editor only saves single-document catalogs; consolidate the events directory before editing."
                .to_string(),
        ),
    }
}

fn ensure_event_directory_inside_project(
    project_root: &Path,
    event_directory: &Path,
) -> Result<(), String> {
    let canonical_root = project_root.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve project root `{}`: {error}",
            project_root.display()
        )
    })?;
    let canonical_directory = event_directory.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve story event directory `{}`: {error}",
            event_directory.display()
        )
    })?;
    if !canonical_directory.starts_with(canonical_root) {
        return Err(format!(
            "Story event directory escapes the active project: {}",
            event_directory.display()
        ));
    }
    Ok(())
}

fn staged_event_paths(target_path: &Path) -> Result<(PathBuf, PathBuf), String> {
    let file_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Story event document path is not valid UTF-8.".to_string())?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| format!("System clock is invalid: {error}"))?
        .as_nanos();
    let parent = target_path
        .parent()
        .ok_or_else(|| "Story event document has no parent directory.".to_string())?;
    Ok((
        parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce)),
        parent.join(format!(".{file_name}.{}.{}.bak", std::process::id(), nonce)),
    ))
}

async fn write_staged_event_document(
    target_path: &Path,
    temp_path: &Path,
    backup_path: &Path,
    content: &[u8],
) -> Result<(), String> {
    tokio::fs::write(temp_path, content)
        .await
        .map_err(|error| {
            format!(
                "Failed to stage story event document `{}`: {error}",
                temp_path.display()
            )
        })?;
    if !target_path.exists() {
        if let Err(error) = tokio::fs::rename(temp_path, target_path).await {
            let _ = tokio::fs::remove_file(temp_path).await;
            return Err(format!(
                "Failed to install story event document `{}`: {error}",
                target_path.display()
            ));
        }
        return Ok(());
    }

    tokio::fs::rename(target_path, backup_path)
        .await
        .map_err(|error| {
            format!(
                "Failed to stage existing story event document `{}`: {error}",
                target_path.display()
            )
        })?;
    if let Err(error) = tokio::fs::rename(temp_path, target_path).await {
        let _ = tokio::fs::rename(backup_path, target_path).await;
        let _ = tokio::fs::remove_file(temp_path).await;
        return Err(format!(
            "Failed to replace story event document `{}`: {error}",
            target_path.display()
        ));
    }
    Ok(())
}

async fn rollback_staged_event_document(
    target_path: &Path,
    backup_path: &Path,
) -> Result<(), String> {
    if target_path.exists() {
        tokio::fs::remove_file(target_path).await.map_err(|error| {
            format!(
                "Failed to remove rejected story event document `{}`: {error}",
                target_path.display()
            )
        })?;
    }
    if backup_path.exists() {
        tokio::fs::rename(backup_path, target_path)
            .await
            .map_err(|error| {
                format!(
                    "Failed to restore story event document `{}`: {error}",
                    target_path.display()
                )
            })?;
    }
    Ok(())
}

async fn cleanup_event_backup(backup_path: &Path) {
    if backup_path.exists() {
        if let Err(error) = tokio::fs::remove_file(backup_path).await {
            tracing::warn!(
                "Story event catalog saved but backup cleanup failed for {}: {}",
                backup_path.display(),
                error
            );
        }
    }
}

fn validate_story_event_content_references(
    project_root: &Path,
    catalog: &StoryEventCatalog,
) -> Result<(), String> {
    let scene_ids = project_content_ids(project_root, "scenes")?;
    let dialogue_ids = project_content_ids(project_root, "dialogue")?;
    let ending_ids = project_content_ids(project_root, "endings")?;
    for definition in catalog.definitions() {
        for action in &definition.actions {
            let (content_type, content_id, known_ids) = match action {
                StoryEventAction::UnlockScene { scene_id } => ("scene", scene_id, &scene_ids),
                StoryEventAction::UnlockDialogue { dialogue_id } => {
                    ("dialogue", dialogue_id, &dialogue_ids)
                }
                StoryEventAction::UnlockEnding { ending_id } => ("ending", ending_id, &ending_ids),
                StoryEventAction::SetFlag { .. } => continue,
            };
            if !known_ids.contains(content_id) {
                return Err(format!(
                    "Story event `{}` unlocks unknown {content_type} `{content_id}`.",
                    definition.event_id
                ));
            }
        }
    }
    Ok(())
}

fn project_content_ids(
    project_root: &Path,
    directory_name: &str,
) -> Result<std::collections::HashSet<String>, String> {
    let directory = project_root.join(directory_name);
    if !directory.exists() {
        return Ok(std::collections::HashSet::new());
    }
    let metadata = std::fs::symlink_metadata(&directory).map_err(|error| {
        format!(
            "Failed to inspect project {directory_name} directory `{}`: {error}",
            directory.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "Project {directory_name} path must be a regular directory: {}",
            directory.display()
        ));
    }

    let mut ids = std::collections::HashSet::new();
    for entry in std::fs::read_dir(&directory).map_err(|error| {
        format!(
            "Failed to read project {directory_name} directory `{}`: {error}",
            directory.display()
        )
    })? {
        let path = entry
            .map_err(|error| format!("Failed to read {directory_name} entry: {error}"))?
            .path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "Failed to inspect project content `{}`: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Project content must be a regular JSON file: {}",
                path.display()
            ));
        }
        let value: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).map_err(|error| {
                format!(
                    "Failed to read project content `{}`: {error}",
                    path.display()
                )
            })?)
            .map_err(|error| {
                format!(
                    "Invalid project content JSON in `{}`: {error}",
                    path.display()
                )
            })?;
        let records = match value {
            Value::Array(records) => records,
            object @ Value::Object(_) => vec![object],
            _ => Vec::new(),
        };
        for record in records {
            if let Some(id) = record.get("id").and_then(Value::as_str) {
                ids.insert(id.to_string());
            }
        }
    }
    Ok(ids)
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

    static NEXT_TEMP_ROOT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    fn temp_root(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_story_event_command_{label}_{}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_TEMP_ROOT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
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
            .contains("festival_night"));
    }

    fn event_document(event_id: &str, description: &str) -> Value {
        serde_json::json!({
            "schema": "monogatari-story-event-catalog/v1",
            "events": [{
                "event_id": event_id,
                "event_type": "test_unlock",
                "description": description,
                "actions": [{"type": "unlock_scene", "scene_id": "test_scene"}],
                "rule": {"min_relationship": 0.2}
            }]
        })
    }

    async fn state_with_event_document(document: &Value) -> (AppState, std::path::PathBuf) {
        let state = AppState::new();
        let root = temp_root("save");
        std::fs::create_dir_all(root.join("characters")).unwrap();
        std::fs::create_dir_all(root.join("knowledge")).unwrap();
        std::fs::create_dir_all(root.join("events")).unwrap();
        std::fs::create_dir_all(root.join("scenes")).unwrap();
        std::fs::create_dir_all(root.join("dialogue")).unwrap();
        std::fs::create_dir_all(root.join("endings")).unwrap();
        std::fs::write(
            root.join("scenes").join("test_scene.json"),
            r#"{"id":"test_scene","name":"Test Scene"}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("events").join("custom.json"),
            serde_json::to_string_pretty(document).unwrap(),
        )
        .unwrap();
        state.set_project_data_root(root.clone()).await;
        *state.story_event_catalog.write().await =
            StoryEventCatalog::load_from_project_root(&root).unwrap();
        (state, root)
    }

    #[tokio::test]
    async fn save_replaces_single_document_and_active_catalog() {
        let original = event_document("before", "Before");
        let (state, root) = state_with_event_document(&original).await;
        let before = state
            .story_event_catalog
            .read()
            .await
            .catalog_fingerprint()
            .to_string();
        let replacement = event_document("after", "After");

        let snapshot = save_story_event_catalog_inner(&state, replacement, &before)
            .await
            .unwrap();

        assert_eq!(snapshot.event_count, 1);
        assert_eq!(snapshot.events[0].event_id, "after");
        assert!(state
            .story_event_catalog
            .read()
            .await
            .definition("after", None)
            .is_some());
        let loaded = StoryEventCatalog::load_from_project_root(&root).unwrap();
        assert_eq!(loaded.catalog_fingerprint(), snapshot.catalog_fingerprint);
        assert!(std::fs::read_dir(root.join("events"))
            .unwrap()
            .filter_map(Result::ok)
            .all(|entry| !entry.file_name().to_string_lossy().ends_with(".bak")));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn invalid_or_stale_save_does_not_mutate_file_or_catalog() {
        let original = event_document("before", "Before");
        let (state, root) = state_with_event_document(&original).await;
        let path = root.join("events").join("custom.json");
        let file_before = std::fs::read_to_string(&path).unwrap();
        let fingerprint = state
            .story_event_catalog
            .read()
            .await
            .catalog_fingerprint()
            .to_string();

        let invalid = serde_json::json!({"schema": "unsupported", "events": []});
        assert!(
            save_story_event_catalog_inner(&state, invalid, &fingerprint)
                .await
                .is_err()
        );
        assert!(save_story_event_catalog_inner(
            &state,
            event_document("after", "After"),
            "0badfingerprint"
        )
        .await
        .is_err());
        let unknown_target = serde_json::json!({
            "schema": "monogatari-story-event-catalog/v1",
            "events": [{
                "event_id":"unknown_target","event_type":"test","description":"Unknown",
                "actions":[{"type":"unlock_scene","scene_id":"missing_scene"}]
            }]
        });
        assert!(
            save_story_event_catalog_inner(&state, unknown_target, &fingerprint)
                .await
                .unwrap_err()
                .contains("unknown scene")
        );

        assert_eq!(std::fs::read_to_string(path).unwrap(), file_before);
        assert_eq!(
            state.story_event_catalog.read().await.catalog_fingerprint(),
            fingerprint
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn visual_save_rejects_multi_document_catalogs() {
        let original = event_document("before", "Before");
        let (state, root) = state_with_event_document(&original).await;
        std::fs::write(
            root.join("events").join("second.json"),
            serde_json::to_string_pretty(&event_document("second", "Second")).unwrap(),
        )
        .unwrap();
        let fingerprint = state
            .story_event_catalog
            .read()
            .await
            .catalog_fingerprint()
            .to_string();

        let result =
            save_story_event_catalog_inner(&state, event_document("after", "After"), &fingerprint)
                .await;

        assert!(result.unwrap_err().contains("single-document"));
        assert!(state
            .story_event_catalog
            .read()
            .await
            .definition("before", None)
            .is_some());
        std::fs::remove_dir_all(root).unwrap();
    }
}
