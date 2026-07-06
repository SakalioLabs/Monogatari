//! Scene management commands for background/scene transitions.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneInfo {
    pub id: String,
    pub name: String,
    pub background_path: Option<String>,
    pub bgm_path: Option<String>,
    pub weather: Option<String>,
    pub time_of_day: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveScene {
    pub scene: Option<SceneInfo>,
    pub scene_history: Vec<String>,
}

/// Get the current active scene.
#[tauri::command]
pub async fn get_current_scene(state: State<'_, AppState>) -> Result<ActiveScene, String> {
    let sm = state.scene_manager.read().await;
    let scenes = sm.get_scenes();
    let current = scenes.last().cloned();
    let history: Vec<String> = scenes.iter().map(|s| s.id.clone()).collect();
    Ok(ActiveScene { scene: current, scene_history: history })
}

/// Set the current scene (change background, BGM, etc).
#[tauri::command]
pub async fn set_scene(
    state: State<'_, AppState>,
    scene_id: String,
    name: String,
    background_path: Option<String>,
    bgm_path: Option<String>,
) -> Result<String, String> {
    let mut sm = state.scene_manager.write().await;
    let scene = llm_game::scenes::scene::Scene {
        id: scene_id.clone(),
        name,
        background_path,
        bgm_path,
        weather: None,
        time_of_day: None,
    };
    sm.push_scene(scene);
    Ok(format!("Scene set to: {scene_id}"))
}

/// List all registered scenes.
#[tauri::command]
pub async fn list_scenes(state: State<'_, AppState>) -> Result<Vec<SceneInfo>, String> {
    let sm = state.scene_manager.read().await;
    Ok(sm.get_scenes().iter().map(|s| SceneInfo {
        id: s.id.clone(),
        name: s.name.clone(),
        background_path: s.background_path.clone(),
        bgm_path: s.bgm_path.clone(),
        weather: s.weather.clone(),
        time_of_day: s.time_of_day.clone(),
    }).collect())
}
