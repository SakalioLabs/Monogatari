//! Character management CRUD commands for the production workbench.
//!
//! Allows creators to create, edit, and delete characters through the UI.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;
use llm_game::characters::{Character, CharacterKnowledgeEntry, Personality};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonalityInput {
    #[serde(default)]
    pub openness: Option<f32>,
    #[serde(default)]
    pub conscientiousness: Option<f32>,
    #[serde(default)]
    pub extraversion: Option<f32>,
    #[serde(default)]
    pub agreeableness: Option<f32>,
    #[serde(default)]
    pub neuroticism: Option<f32>,
    #[serde(default)]
    pub speech_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCreateInput {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub personality: Option<PersonalityInput>,
    #[serde(default)]
    pub personality_openness: Option<f32>,
    #[serde(default)]
    pub personality_conscientiousness: Option<f32>,
    #[serde(default)]
    pub personality_extraversion: Option<f32>,
    #[serde(default)]
    pub personality_agreeableness: Option<f32>,
    #[serde(default)]
    pub personality_neuroticism: Option<f32>,
    #[serde(default)]
    pub speech_style: Option<String>,
    #[serde(default)]
    pub default_emotion: Option<String>,
    #[serde(default)]
    pub live2d_model_path: Option<String>,
    #[serde(default)]
    pub model_3d_path: Option<String>,
    #[serde(default)]
    pub portrait_path: Option<String>,
    #[serde(default)]
    pub sprite_path: Option<String>,
    #[serde(default)]
    pub sprite_paths: HashMap<String, String>,
    #[serde(default)]
    pub relationships: HashMap<String, f32>,
    #[serde(default)]
    pub knowledge_entries: Vec<CharacterKnowledgeEntry>,
    #[serde(default, alias = "knowledge", alias = "knowledgeRefs")]
    pub knowledge_refs: Vec<String>,
    #[serde(default)]
    pub emotion_modifiers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharacterSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub emotion: String,
    pub relationship_score: f32,
    pub has_live2d: bool,
    pub has_3d: bool,
    pub has_sprite: bool,
}

fn clamp_trait(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn merge_personality(input: &CharacterCreateInput) -> Personality {
    let nested = input.personality.as_ref();
    let mut personality = Personality::default();

    personality.openness = clamp_trait(
        nested
            .and_then(|p| p.openness)
            .or(input.personality_openness)
            .unwrap_or(personality.openness),
    );
    personality.conscientiousness = clamp_trait(
        nested
            .and_then(|p| p.conscientiousness)
            .or(input.personality_conscientiousness)
            .unwrap_or(personality.conscientiousness),
    );
    personality.extraversion = clamp_trait(
        nested
            .and_then(|p| p.extraversion)
            .or(input.personality_extraversion)
            .unwrap_or(personality.extraversion),
    );
    personality.agreeableness = clamp_trait(
        nested
            .and_then(|p| p.agreeableness)
            .or(input.personality_agreeableness)
            .unwrap_or(personality.agreeableness),
    );
    personality.neuroticism = clamp_trait(
        nested
            .and_then(|p| p.neuroticism)
            .or(input.personality_neuroticism)
            .unwrap_or(personality.neuroticism),
    );
    personality.speech_style = nested
        .and_then(|p| p.speech_style.clone())
        .or_else(|| input.speech_style.clone())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(personality.speech_style);

    personality
}

fn normalize_optional_path(path: Option<String>) -> Option<String> {
    path.map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_path_map(paths: HashMap<String, String>) -> HashMap<String, String> {
    paths
        .into_iter()
        .map(|(emotion, path)| (emotion.trim().to_string(), path.trim().to_string()))
        .filter(|(emotion, path)| !emotion.is_empty() && !path.is_empty())
        .collect()
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

    let id = input.id.trim().to_string();
    let name = input.name.trim().to_string();
    let personality = merge_personality(&input);
    let emotion = input
        .default_emotion
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "neutral".to_string());
    let live2d_model_path = normalize_optional_path(input.live2d_model_path.clone());
    let model_3d_path = normalize_optional_path(input.model_3d_path.clone());
    let portrait_path = normalize_optional_path(input.portrait_path.clone());
    let sprite_path = normalize_optional_path(input.sprite_path.clone());
    let path = dir.join(format!("{id}.json"));

    let mut character = Character::new(id.clone(), name.clone());
    character.description = input.description;
    character.background = input.background;
    character.personality = personality;
    character.emotion = emotion;
    character.personality.current_emotion = character.emotion.clone();
    character.live2d_model_path = live2d_model_path;
    character.model_3d_path = model_3d_path;
    character.portrait_path = portrait_path;
    character.sprite_path = sprite_path;
    character.sprite_paths = normalize_path_map(input.sprite_paths);
    if let Some(sprite_path) = &character.sprite_path {
        character
            .sprite_paths
            .entry(character.emotion.clone())
            .or_insert_with(|| sprite_path.clone());
        character
            .sprite_paths
            .entry("neutral".to_string())
            .or_insert_with(|| sprite_path.clone());
    }
    character.relationships = input.relationships;
    character.knowledge_entries = input.knowledge_entries;
    character.knowledge_refs = input
        .knowledge_refs
        .into_iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect();
    character.emotion_modifiers = input.emotion_modifiers;

    let content = serde_json::to_string_pretty(&character).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())?;

    let mut cm = state.character_manager.write().await;
    cm.add_character(character);

    Ok(format!("Character {id} saved."))
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
                has_3d: c.model_3d_path.is_some(),
                has_sprite: c.sprite_for_emotion(&c.emotion).is_some() || c.portrait_path.is_some(),
            });
        }
    }
    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_path_map_trims_and_drops_empty_sprite_slots() {
        let paths = HashMap::from([
            (
                " happy ".to_string(),
                " assets/sprites/sakura_happy.png ".to_string(),
            ),
            (" ".to_string(), "assets/sprites/invalid.png".to_string()),
            ("sad".to_string(), " ".to_string()),
        ]);

        let normalized = normalize_path_map(paths);

        assert_eq!(
            normalized.get("happy").map(String::as_str),
            Some("assets/sprites/sakura_happy.png")
        );
        assert!(!normalized.contains_key(""));
        assert!(!normalized.contains_key("sad"));
    }
}
