//! Versioned ending assets and gated Story Mode launch commands.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::dialogue::{
    ensure_project_dialogues_loaded, start_dialogue_inner, DialogueState,
};
use crate::commands::scenes::{resolve_project_scene, set_scene_inner, SceneInfo};
use crate::state::AppState;
use crate::story_access::{
    ensure_story_content_access, story_content_access, StoryContentAccessEntry, StoryContentKind,
};

const STORY_ENDING_SCHEMA_V1: &str = "monogatari-story-ending/v1";
const MAX_ENDING_FILES: usize = 256;
const MAX_ENDING_FILE_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StoryEndingDefinition {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub description: String,
    pub scene_id: String,
    pub dialogue_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoryEndingCatalogEntry {
    #[serde(flatten)]
    pub ending: StoryEndingDefinition,
    pub access: StoryContentAccessEntry,
}

#[derive(Debug, Serialize)]
pub struct StoryEndingLaunch {
    pub ending: StoryEndingDefinition,
    pub scene: SceneInfo,
    pub dialogue: DialogueState,
}

#[tauri::command]
pub async fn list_story_endings(
    state: State<'_, AppState>,
) -> Result<Vec<StoryEndingCatalogEntry>, String> {
    list_story_endings_inner(&state).await
}

async fn list_story_endings_inner(
    state: &AppState,
) -> Result<Vec<StoryEndingCatalogEntry>, String> {
    let definitions = load_story_endings(&state.current_project_data_root().await)?;
    let catalog = state.story_event_catalog.read().await;
    let progress = state.story_progress.read().await;
    Ok(definitions
        .into_iter()
        .map(|ending| StoryEndingCatalogEntry {
            access: story_content_access(&catalog, &progress, StoryContentKind::Ending, &ending.id),
            ending,
        })
        .collect())
}

#[tauri::command]
pub async fn start_story_ending(
    state: State<'_, AppState>,
    ending_id: String,
) -> Result<StoryEndingLaunch, String> {
    start_story_ending_inner(&state, &ending_id).await
}

async fn start_story_ending_inner(
    state: &AppState,
    ending_id: &str,
) -> Result<StoryEndingLaunch, String> {
    let ending = load_story_endings(&state.current_project_data_root().await)?
        .into_iter()
        .find(|ending| ending.id == ending_id)
        .ok_or_else(|| {
            format!("Story ending `{ending_id}` does not exist in the active project.")
        })?;
    {
        let catalog = state.story_event_catalog.read().await;
        let progress = state.story_progress.read().await;
        ensure_story_content_access(&catalog, &progress, StoryContentKind::Ending, &ending.id)?;
        ensure_story_content_access(
            &catalog,
            &progress,
            StoryContentKind::Scene,
            &ending.scene_id,
        )?;
        ensure_story_content_access(
            &catalog,
            &progress,
            StoryContentKind::Dialogue,
            &ending.dialogue_id,
        )?;
    }

    resolve_project_scene(state, &ending.scene_id).await?;
    ensure_project_dialogues_loaded(state).await?;
    if !state
        .dialogue_manager
        .read()
        .await
        .has_script(&ending.dialogue_id)
    {
        return Err(format!(
            "Story ending `{}` references missing dialogue `{}`.",
            ending.id, ending.dialogue_id
        ));
    }

    let scene = set_scene_inner(state, ending.scene_id.clone(), None, None, None).await?;
    let dialogue = start_dialogue_inner(state, &ending.dialogue_id).await?;
    Ok(StoryEndingLaunch {
        ending,
        scene,
        dialogue,
    })
}

fn load_story_endings(project_root: &Path) -> Result<Vec<StoryEndingDefinition>, String> {
    let ending_root = project_root.join("endings");
    if !ending_root.exists() {
        return Ok(Vec::new());
    }
    let root_metadata = std::fs::symlink_metadata(&ending_root).map_err(|error| {
        format!(
            "Failed to inspect story ending directory `{}`: {error}",
            ending_root.display()
        )
    })?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(format!(
            "Story ending path must be a regular directory: {}",
            ending_root.display()
        ));
    }
    let canonical_root = ending_root.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve story ending directory `{}`: {error}",
            ending_root.display()
        )
    })?;
    let mut files = std::fs::read_dir(&ending_root)
        .map_err(|error| {
            format!(
                "Failed to read story ending directory `{}`: {error}",
                ending_root.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<PathBuf>>();
    files.sort();
    if files.len() > MAX_ENDING_FILES {
        return Err(format!(
            "Story ending directory contains {} JSON files; the limit is {MAX_ENDING_FILES}.",
            files.len()
        ));
    }

    let mut seen = HashSet::new();
    let mut endings = Vec::with_capacity(files.len());
    for path in files {
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "Failed to inspect story ending `{}`: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Story ending must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_ENDING_FILE_BYTES {
            return Err(format!(
                "Story ending `{}` is {} bytes; the limit is {MAX_ENDING_FILE_BYTES} bytes.",
                path.display(),
                metadata.len()
            ));
        }
        let canonical_path = path.canonicalize().map_err(|error| {
            format!(
                "Failed to resolve story ending `{}`: {error}",
                path.display()
            )
        })?;
        if !canonical_path.starts_with(&canonical_root) {
            return Err(format!(
                "Story ending escapes the project ending directory: {}",
                path.display()
            ));
        }
        let content = std::fs::read_to_string(&canonical_path).map_err(|error| {
            format!("Failed to read story ending `{}`: {error}", path.display())
        })?;
        let ending: StoryEndingDefinition = serde_json::from_str(&content).map_err(|error| {
            format!("Invalid story ending JSON in `{}`: {error}", path.display())
        })?;
        validate_ending(&ending, &path)?;
        if !seen.insert(ending.id.clone()) {
            return Err(format!("Duplicate story ending id `{}`.", ending.id));
        }
        endings.push(ending);
    }
    endings.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(endings)
}

fn validate_ending(ending: &StoryEndingDefinition, path: &Path) -> Result<(), String> {
    if ending.schema != STORY_ENDING_SCHEMA_V1 {
        return Err(format!(
            "Story ending `{}` uses unsupported schema `{}`.",
            path.display(),
            ending.schema
        ));
    }
    for (label, value) in [
        ("id", ending.id.as_str()),
        ("scene_id", ending.scene_id.as_str()),
        ("dialogue_id", ending.dialogue_id.as_str()),
    ] {
        if !is_portable_id(value) {
            return Err(format!(
                "Story ending `{}` has invalid {label} `{value}`.",
                path.display()
            ));
        }
    }
    if ending.title.trim().is_empty() || ending.title.chars().count() > 256 {
        return Err(format!(
            "Story ending `{}` title must contain 1 to 256 characters.",
            ending.id
        ));
    }
    if ending.description.trim().is_empty() || ending.description.chars().count() > 2048 {
        return Err(format!(
            "Story ending `{}` description must contain 1 to 2048 characters.",
            ending.id
        ));
    }
    Ok(())
}

fn is_portable_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.trim() == value
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story_events::StoryEventCatalog;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_story_ending_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn write_project(root: &Path) {
        for directory in [
            "characters",
            "knowledge",
            "events",
            "endings",
            "scenes",
            "dialogue",
        ] {
            std::fs::create_dir_all(root.join(directory)).unwrap();
        }
        std::fs::write(
            root.join("events").join("events.json"),
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"finale","event_type":"ending","description":"Finale",
                "actions":[
                  {"type":"unlock_ending","ending_id":"finale"},
                  {"type":"unlock_scene","scene_id":"finale_scene"},
                  {"type":"unlock_dialogue","dialogue_id":"finale_dialogue"}
                ]
              }]
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("endings").join("finale.json"),
            r#"{
              "schema":"monogatari-story-ending/v1","id":"finale","title":"Finale",
              "description":"The final scene.","scene_id":"finale_scene","dialogue_id":"finale_dialogue"
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("scenes").join("finale_scene.json"),
            r#"{"id":"finale_scene","name":"Finale","background_path":null}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("dialogue").join("finale_dialogue.json"),
            r#"{
              "id":"finale_dialogue","title":"Finale","start_node_id":"start",
              "nodes":{"start":{"speaker_id":null,"text":"The end.","choices":[]}}
            }"#,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn ending_launch_enforces_unlocks_then_starts_scene_and_dialogue() {
        let root = temp_root("launch");
        write_project(&root);
        let state = AppState::new();
        state.set_project_data_root(root.clone()).await;
        *state.story_event_catalog.write().await =
            StoryEventCatalog::load_from_project_root(&root).unwrap();

        assert!(start_story_ending_inner(&state, "finale").await.is_err());
        {
            let mut progress = state.story_progress.write().await;
            progress.unlocked_ending_ids.insert("finale".to_string());
            progress
                .unlocked_scene_ids
                .insert("finale_scene".to_string());
            progress
                .unlocked_dialogue_ids
                .insert("finale_dialogue".to_string());
        }

        let launch = start_story_ending_inner(&state, "finale").await.unwrap();
        assert_eq!(launch.ending.id, "finale");
        assert_eq!(launch.scene.id, "finale_scene");
        assert!(launch.dialogue.is_active);
        assert_eq!(
            state.active_scene_id.read().await.as_deref(),
            Some("finale_scene")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ending_loader_rejects_duplicates_and_unknown_fields() {
        let root = temp_root("invalid");
        std::fs::create_dir_all(root.join("endings")).unwrap();
        std::fs::write(
            root.join("endings").join("invalid.json"),
            r#"{
              "schema":"monogatari-story-ending/v1","id":"ending","title":"Ending",
              "description":"Description","scene_id":"scene","dialogue_id":"dialogue","extra":true
            }"#,
        )
        .unwrap();

        assert!(load_story_endings(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
