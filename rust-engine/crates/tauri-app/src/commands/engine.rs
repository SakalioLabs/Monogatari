//! Engine initialization and status commands.

use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::state::AppState;

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
    let path = normalize_project_path(&project_path)?;

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

    *state.project_path.write().await = Some(path);
    *state.initialized.write().await = true;

    Ok("Engine initialized successfully".to_string())
}

fn normalize_project_path(project_path: &str) -> Result<PathBuf, String> {
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let requested = if project_path.trim().is_empty() {
        PathBuf::from("data")
    } else {
        PathBuf::from(project_path)
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
