//! Character management commands.

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct CharacterInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub emotion: String,
    pub live2d_model_path: Option<String>,
}

/// Get all characters.
#[tauri::command]
pub async fn get_characters(state: State<'_, AppState>) -> Result<Vec<CharacterInfo>, String> {
    let cm = state.character_manager.read().await;
    let mut characters = Vec::new();

    for (id, character) in cm.all_characters() {
        let character = character.read().await;
        characters.push(CharacterInfo {
            id: id.clone(),
            name: character.name.clone(),
            description: character.description.clone(),
            emotion: character.emotion.clone(),
            live2d_model_path: character.live2d_model_path.clone(),
        });
    }

    Ok(characters)
}

/// Get a specific character by ID.
#[tauri::command]
pub async fn get_character(
    state: State<'_, AppState>,
    character_id: String,
) -> Result<CharacterInfo, String> {
    let cm = state.character_manager.read().await;
    let character = cm
        .get_character(&character_id)
        .ok_or_else(|| format!("Character not found: {character_id}"))?;

    let character = character.read().await;
    Ok(CharacterInfo {
        id: character.id.clone(),
        name: character.name.clone(),
        description: character.description.clone(),
        emotion: character.emotion.clone(),
        live2d_model_path: character.live2d_model_path.clone(),
    })
}

/// Load characters from a directory.
#[tauri::command]
pub async fn load_characters(
    state: State<'_, AppState>,
    directory: String,
) -> Result<usize, String> {
    let path = std::path::PathBuf::from(&directory);
    let mut cm = state.character_manager.write().await;
    cm.load_from_directory(&path)
        .await
        .map_err(|e| e.to_string())
}
