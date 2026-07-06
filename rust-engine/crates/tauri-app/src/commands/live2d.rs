//! Live2D model management commands.

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct Live2DModelInfo {
    pub model_path: String,
    pub expressions: Vec<String>,
    pub motions: Vec<String>,
    pub current_expression: Option<String>,
    pub current_motion: Option<String>,
}

/// Load a Live2D model.
#[tauri::command]
pub async fn load_model(
    _state: State<'_, AppState>,
    model_path: String,
) -> Result<Live2DModelInfo, String> {
    // Validate model path exists
    let path = std::path::PathBuf::from(&model_path);
    if !path.exists() {
        return Err(format!("Model path does not exist: {model_path}"));
    }

    // In a full implementation, this would:
    // 1. Load the Live2D Cubism model (.model3.json)
    // 2. Parse expressions and motions from the model config
    // 3. Initialize the WebGL rendering context

    // For now, return placeholder data
    let expressions = if path.join("expressions").exists() {
        std::fs::read_dir(path.join("expressions"))
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        e.path()
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec!["default".to_string()]
    };

    let motions = if path.join("motions").exists() {
        std::fs::read_dir(path.join("motions"))
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        e.path()
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec!["idle".to_string()]
    };

    Ok(Live2DModelInfo {
        model_path,
        expressions,
        motions,
        current_expression: None,
        current_motion: None,
    })
}

/// Set the expression on the active Live2D model.
#[tauri::command]
pub async fn set_expression(
    state: State<'_, AppState>,
    character_id: String,
    expression: String,
) -> Result<String, String> {
    // Update character emotion in the character manager
    let cm = state.character_manager.read().await;
    if let Some(character) = cm.get_character(&character_id) {
        let mut character = character.write().await;
        character.set_emotion(&expression);
    }

    Ok(format!("Expression set to: {expression}"))
}

/// Set the motion on the active Live2D model.
#[tauri::command]
pub async fn set_motion(
    _state: State<'_, AppState>,
    _character_id: String,
    motion_group: String,
    motion_index: Option<usize>,
) -> Result<String, String> {
    // In a full implementation, this would trigger the Live2D motion
    // via the Cubism SDK JavaScript API in the frontend
    Ok(format!(
        "Motion set: {} (index: {:?})",
        motion_group, motion_index
    ))
}

/// Get info about a loaded Live2D model.
#[tauri::command]
pub async fn get_model_info(
    state: State<'_, AppState>,
    character_id: String,
) -> Result<Option<Live2DModelInfo>, String> {
    let model_path = {
        let cm = state.character_manager.read().await;
        if let Some(character) = cm.get_character(&character_id) {
            let character = character.read().await;
            character.live2d_model_path.clone()
        } else {
            None
        }
    };
    if let Some(path) = model_path {
        load_model(state, path).await.map(Some)
    } else {
        Ok(None)
    }
}
