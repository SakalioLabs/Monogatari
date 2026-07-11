//! Engine initialization and status commands.

use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

use llm_game::{characters::CharacterManager, dialogue::DialogueManager, knowledge::KnowledgeBase};

use crate::state::{default_project_data_root, AppState};
use crate::story_events::StoryEventCatalog;

#[derive(Serialize)]
pub struct EngineStatus {
    pub initialized: bool,
    pub character_count: usize,
    pub dialogue_count: usize,
    pub knowledge_count: usize,
    pub story_event_count: usize,
    pub story_event_catalog_fingerprint: String,
    pub applied_story_event_count: usize,
    pub unlocked_story_content_count: usize,
    pub story_progress_fingerprint: String,
    pub ai_engines: Vec<String>,
    pub active_ai_engine: Option<String>,
}

/// Initialize the engine with data from the project directory.
#[tauri::command]
pub async fn initialize_engine(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<String, String> {
    let path = if project_path.trim().is_empty() {
        state.current_project_data_root().await
    } else {
        normalize_project_path(&project_path)?
    };
    let path = validate_engine_project_root(path)?;

    let (characters, dialogues, knowledge, story_events) = load_project_content(&path).await?;

    // Initialize AI backends before replacing the active project state.
    let pipeline = state.inference_pipeline.read().await;
    pipeline.initialize_all().await.map_err(|e| e.to_string())?;
    drop(pipeline);

    let root_changed = state.set_project_data_root(path).await;
    if !root_changed {
        state.reset_project_runtime_state().await;
    }
    *state.character_manager.write().await = characters;
    *state.dialogue_manager.write().await = dialogues;
    *state.knowledge_base.write().await = knowledge;
    *state.story_event_catalog.write().await = story_events;
    *state.initialized.write().await = true;

    Ok("Engine initialized successfully".to_string())
}

pub(crate) async fn load_project_content(
    path: &Path,
) -> Result<
    (
        CharacterManager,
        DialogueManager,
        KnowledgeBase,
        StoryEventCatalog,
    ),
    String,
> {
    let mut characters = CharacterManager::new();
    // Load characters
    let char_path = path.join("characters");
    if char_path.exists() {
        characters
            .load_from_directory(&char_path)
            .await
            .map_err(|e| e.to_string())?;
    }

    let mut dialogues = DialogueManager::new();
    // Load dialogues
    let dlg_path = path.join("dialogue");
    if dlg_path.exists() {
        dialogues
            .load_from_directory(&dlg_path)
            .await
            .map_err(|e| e.to_string())?;
    }

    let mut knowledge = KnowledgeBase::new();
    // Load knowledge
    let kb_path = path.join("knowledge");
    if kb_path.exists() {
        knowledge
            .load_from_directory(&kb_path)
            .await
            .map_err(|e| e.to_string())?;
    }

    let story_events = StoryEventCatalog::load_from_project_root(path)?;
    let character_ids = characters.character_ids();
    story_events.validate_character_references(character_ids.iter().map(String::as_str))?;

    Ok((characters, dialogues, knowledge, story_events))
}

fn normalize_project_path(project_path: &str) -> Result<PathBuf, String> {
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    normalize_project_path_from(project_path, &current_dir)
}

fn normalize_project_path_from(project_path: &str, current_dir: &Path) -> Result<PathBuf, String> {
    let trimmed = project_path.trim();
    let requested = if trimmed.is_empty() {
        return Ok(default_project_data_root());
    } else {
        if trimmed.chars().any(char::is_control) {
            return Err("Project path cannot contain control characters.".to_string());
        }
        if trimmed.contains("://") {
            return Err("Project path must be a local filesystem path, not a URI.".to_string());
        }
        PathBuf::from(trimmed)
    };

    if requested.is_absolute() {
        return Ok(requested);
    }

    let direct = current_dir.join(&requested);
    if direct.exists() {
        return Ok(direct);
    }

    Ok(find_upward(current_dir, &requested).unwrap_or(direct))
}

fn validate_engine_project_root(project_root: PathBuf) -> Result<PathBuf, String> {
    if !project_root.exists() {
        return Err(format!(
            "Engine project path does not exist: {}",
            project_root.display()
        ));
    }
    if !project_root.is_dir() {
        return Err(format!(
            "Engine project path is not a directory: {}",
            project_root.display()
        ));
    }
    Ok(project_root)
}

fn find_upward(start: &Path, relative: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|ancestor| ancestor.join(relative))
        .find(|candidate| candidate.exists())
}

/// Get the current engine status.
#[tauri::command]
pub async fn get_engine_status(state: State<'_, AppState>) -> Result<EngineStatus, String> {
    let initialized = *state.initialized.read().await;
    let cm = state.character_manager.read().await;
    let dm = state.dialogue_manager.read().await;
    let kb = state.knowledge_base.read().await;
    let story_events = state.story_event_catalog.read().await;
    let story_progress = state.story_progress.read().await;
    let pipeline = state.inference_pipeline.read().await;
    let story_event_snapshot = story_events.snapshot();
    let story_progress_snapshot = story_progress.snapshot();

    Ok(EngineStatus {
        initialized,
        character_count: cm.character_ids().len(),
        dialogue_count: dm.script_ids().len(),
        knowledge_count: kb.len(),
        story_event_count: story_event_snapshot.event_count,
        story_event_catalog_fingerprint: story_event_snapshot.catalog_fingerprint,
        applied_story_event_count: story_progress_snapshot.applied_event_count,
        unlocked_story_content_count: story_progress_snapshot.unlocked_scene_ids.len()
            + story_progress_snapshot.unlocked_dialogue_ids.len()
            + story_progress_snapshot.unlocked_ending_ids.len(),
        story_progress_fingerprint: story_progress_snapshot.progress_fingerprint,
        ai_engines: pipeline
            .engine_names()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        active_ai_engine: pipeline.active_engine_name().map(|s| s.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_engine_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn engine_project_paths_resolve_existing_relative_dirs() {
        let root = temp_root("resolve");
        let nested = root.join("workspace").join("rust-engine");
        let data = root.join("data");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir_all(&data).unwrap();

        let resolved = normalize_project_path_from("data", &nested).unwrap();
        assert_eq!(resolved, data);
        assert_eq!(validate_engine_project_root(resolved).unwrap(), data);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn engine_project_paths_reject_uri_and_control_input() {
        let root = PathBuf::from("workspace");
        for project_path in ["https://example.test/data", "data\nother", "data\0other"] {
            assert!(
                normalize_project_path_from(project_path, &root).is_err(),
                "{project_path:?} should be rejected"
            );
        }
    }

    #[test]
    fn engine_project_root_validation_requires_existing_directory() {
        let root = temp_root("validate");
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("settings.json");
        std::fs::write(&file, "{}").unwrap();

        assert!(validate_engine_project_root(root.join("missing")).is_err());
        assert!(validate_engine_project_root(file).is_err());
        assert!(validate_engine_project_root(root.clone()).is_ok());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn project_content_loading_replaces_instead_of_merging_managers() {
        let first = temp_root("project_first");
        let second = temp_root("project_second");
        for root in [&first, &second] {
            std::fs::create_dir_all(root.join("characters")).unwrap();
            std::fs::create_dir_all(root.join("knowledge")).unwrap();
            std::fs::create_dir_all(root.join("events")).unwrap();
        }
        std::fs::write(
            first.join("characters").join("first.json"),
            r#"{"id":"first","name":"First"}"#,
        )
        .unwrap();
        std::fs::write(
            second.join("characters").join("second.json"),
            r#"{"id":"second","name":"Second"}"#,
        )
        .unwrap();
        std::fs::write(
            first.join("events").join("events.json"),
            r#"{"schema":"monogatari-story-event-catalog/v1","events":[{"event_id":"first_event","event_type":"unlock","description":"First"}]}"#,
        )
        .unwrap();
        std::fs::write(
            second.join("events").join("events.json"),
            r#"{"schema":"monogatari-story-event-catalog/v1","events":[{"event_id":"second_event","event_type":"unlock","description":"Second"}]}"#,
        )
        .unwrap();

        let (first_characters, _, _, first_events) = load_project_content(&first).await.unwrap();
        let (second_characters, _, _, second_events) = load_project_content(&second).await.unwrap();

        assert!(first_characters.get_character("first").is_some());
        assert!(first_characters.get_character("second").is_none());
        assert!(second_characters.get_character("second").is_some());
        assert!(second_characters.get_character("first").is_none());
        assert!(first_events.definition("first_event", None).is_some());
        assert!(first_events.definition("second_event", None).is_none());
        assert!(second_events.definition("second_event", None).is_some());
        assert!(second_events.definition("first_event", None).is_none());
        std::fs::remove_dir_all(first).unwrap();
        std::fs::remove_dir_all(second).unwrap();
    }

    #[tokio::test]
    async fn checked_in_project_data_loads_as_real_runtime_content() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for root in [
            manifest_dir.join("../../data"),
            manifest_dir.join("../../../data"),
        ] {
            let (characters, dialogues, _, events) = load_project_content(&root)
                .await
                .unwrap_or_else(|error| panic!("{}: {error}", root.display()));
            assert!(!characters.character_ids().is_empty(), "{}", root.display());
            assert!(!dialogues.script_ids().is_empty(), "{}", root.display());
            assert!(!events.snapshot().events.is_empty(), "{}", root.display());
        }
    }
}
