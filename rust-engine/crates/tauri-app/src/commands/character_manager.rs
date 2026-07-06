//! Character management CRUD commands for the production workbench.
//!
//! Allows creators to create, edit, and delete characters through the UI.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCreateInput {
    pub id: String,
    pub name: String,
    pub description: String,
    pub background: String,
    pub personality_openness: f32,
    pub personality_conscientiousness: f32,
    pub personality_extraversion: f32,
    pub personality_agreeableness: f32,
    pub personality_neuroticism: f32,
    pub speech_style: String,
    pub live2d_model_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharacterSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub emotion: String,
    pub relationship_score: f32,
    pub has_live2d: bool,
}

/// Create a new character and save to the project characters directory.
#[tauri::command]
pub async fn create_character(
    state: State<'_, AppState>,
    input: CharacterCreateInput,
) -> Result<String, String> {
    if input.id.trim().is_empty() {
        return Err("Character id is required.".to_string());
    }
    if input.name.trim().is_empty() {
        return Err("Character name is required.".to_string());
    }

    let project = state.project_path.read().await;
    let base = project.as_ref().map(|p| p.join("characters"));
    let Some(dir) = base else {
        return Err("No project path configured.".to_string());
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join(format!("{}.json", input.id));
    let json = serde_json::json!({
        "id": input.id,
        "name": input.name,
        "description": input.description,
        "background": input.background,
        "personality": {
            "openness": input.personality_openness,
            "conscientiousness": input.personality_conscientiousness,
            "extraversion": input.personality_extraversion,
            "agreeableness": input.personality_agreeableness,
            "neuroticism": input.personality_neuroticism,
            "speech_style": input.speech_style
        },
        "live2d_model_path": input.live2d_model_path
    });
    let content = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())?;

    Ok(format!("Character {} created.", input.id))
}

/// Delete a character by id.
#[tauri::command]
pub async fn delete_character(
    state: State<'_, AppState>,
    character_id: String,
) -> Result<String, String> {
    let project = state.project_path.read().await;
    let base = project.as_ref().map(|p| p.join("characters"));
    let Some(dir) = base else {
        return Err("No project path configured.".to_string());
    };
    let path = dir.join(format!("{character_id}.json"));
    if !path.exists() {
        return Err(format!("Character not found: {character_id}"));
    }
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(format!("Character {character_id} deleted."))
}

/// Get summaries of all loaded characters with relationship scores.
#[tauri::command]
pub async fn get_character_summaries(
    state: State<'_, AppState>,
) -> Result<Vec<CharacterSummary>, String> {
    let cm = state.character_manager.read().await;
    let ids = cm.character_ids();
    let mut summaries = Vec::new();
    for id in ids {
        if let Some(character) = cm.get_character(&id) {
            let c = character.read().await;
            summaries.push(CharacterSummary {
                id: id.clone(),
                name: c.name.clone(),
                description: c.description.clone(),
                emotion: c.emotion.clone(),
                relationship_score: c.relationships.get("player").copied().unwrap_or(0.0),
                has_live2d: c.live2d_model_path.is_some(),
            });
        }
    }
    Ok(summaries)
}