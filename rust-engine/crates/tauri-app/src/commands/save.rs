//! Save/Load game commands.

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct SaveInfo {
    pub save_id: String,
    pub save_name: String,
    pub timestamp: String,
    pub current_scene: Option<String>,
}

/// Save the current game state.
#[tauri::command]
pub async fn save_game(state: State<'_, AppState>, save_name: String) -> Result<String, String> {
    let _dm = state.dialogue_manager.read().await;
    let sm = state.scene_manager.read().await;
    let se = state.script_engine.read().await;
    let active_scene_id = state.active_scene_id.read().await.clone();

    let mut save = llm_assets::SaveManager::create_save(
        &save_name,
        active_scene_id.or_else(|| sm.current_scene_name().map(|s| s.to_string())),
        None,
        None,
    );

    save.flags = se.all_flags();
    // Convert rhai::Dynamic variables to serde_json::Value via string representation
    save.variables = se
        .all_variables()
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(format!("{}", v))))
        .collect();

    let save_mgr = state.save_manager.read().await;
    save_mgr.save(&save).await.map_err(|e| e.to_string())?;

    Ok(save.save_id)
}

/// Load a game state by save ID.
#[tauri::command]
pub async fn load_game(state: State<'_, AppState>, save_id: String) -> Result<String, String> {
    let save_mgr = state.save_manager.read().await;
    let save = save_mgr.load(&save_id).await.map_err(|e| e.to_string())?;

    // Restore flags and variables
    let se = state.script_engine.read().await;
    for (name, value) in &save.flags {
        se.set_flag(name, *value);
    }

    Ok(format!("Loaded save: {}", save.save_name))
}

/// List all saved games.
#[tauri::command]
pub async fn list_saves(state: State<'_, AppState>) -> Result<Vec<SaveInfo>, String> {
    let save_mgr = state.save_manager.read().await;
    let saves = save_mgr.list_saves().await.map_err(|e| e.to_string())?;

    Ok(saves
        .into_iter()
        .map(|s| SaveInfo {
            save_id: s.save_id,
            save_name: s.save_name,
            timestamp: s.timestamp.to_rfc3339(),
            current_scene: s.current_scene,
        })
        .collect())
}

/// Delete a save by ID.
#[tauri::command]
pub async fn delete_save(state: State<'_, AppState>, save_id: String) -> Result<String, String> {
    let save_mgr = state.save_manager.read().await;
    save_mgr.delete(&save_id).await.map_err(|e| e.to_string())?;
    Ok("Save deleted".to_string())
}
