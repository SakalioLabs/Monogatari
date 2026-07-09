//! Engine initialization and status commands.

use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::state::{default_project_data_root, AppState};

#[derive(Serialize)]
pub struct EngineStatus {
    pub initialized: bool,
    pub character_count: usize,
    pub dialogue_count: usize,
    pub knowledge_count: usize,
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

    // Load characters
    let char_path = path.join("characters");
    if char_path.exists() {
        let mut cm = state.character_manager.write().await;
        cm.load_from_directory(&char_path)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Load dialogues
    let dlg_path = path.join("dialogue");
    if dlg_path.exists() {
        let mut dm = state.dialogue_manager.write().await;
        dm.load_from_directory(&dlg_path)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Load knowledge
    let kb_path = path.join("knowledge");
    if kb_path.exists() {
        let mut kb = state.knowledge_base.write().await;
        kb.load_from_directory(&kb_path)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Initialize AI pipeline
    let pipeline = state.inference_pipeline.read().await;
    pipeline.initialize_all().await.map_err(|e| e.to_string())?;

    state.set_project_data_root(path).await;
    *state.initialized.write().await = true;

    Ok("Engine initialized successfully".to_string())
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

    Ok(find_upward(&current_dir, &requested).unwrap_or(direct))
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
    let pipeline = state.inference_pipeline.read().await;

    Ok(EngineStatus {
        initialized,
        character_count: cm.character_ids().len(),
        dialogue_count: dm.script_ids().len(),
        knowledge_count: kb.len(),
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
}
