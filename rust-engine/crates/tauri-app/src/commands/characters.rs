//! Character management commands.

use std::collections::HashMap;
use std::path::Path;

use llm_authoring::paths::resolve_project_relative;
use serde::Serialize;
use tauri::State;

use crate::commands::content_paths::resolve_project_content_dir;
use crate::state::AppState;
use llm_game::characters::{Character, CharacterKnowledgeEntry, CharacterManager, Personality};

#[derive(Serialize)]
pub struct CharacterInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub background: String,
    pub personality: Personality,
    pub emotion: String,
    pub relationships: HashMap<String, f32>,
    pub sprite_paths: HashMap<String, String>,
    pub absolute_sprite_paths: HashMap<String, String>,
    pub sprite_path: Option<String>,
    pub absolute_sprite_path: Option<String>,
    pub live2d_model_path: Option<String>,
    pub absolute_live2d_model_path: Option<String>,
    pub model_3d_path: Option<String>,
    pub absolute_model_3d_path: Option<String>,
    pub portrait_path: Option<String>,
    pub absolute_portrait_path: Option<String>,
    pub knowledge_entries: Vec<CharacterKnowledgeEntry>,
    pub knowledge_refs: Vec<String>,
    pub emotion_modifiers: HashMap<String, String>,
}

fn character_info(id: String, character: &Character, project_root: &Path) -> CharacterInfo {
    CharacterInfo {
        id,
        name: character.name.clone(),
        description: character.description.clone(),
        background: character.background.clone(),
        personality: character.personality.clone(),
        emotion: character.emotion.clone(),
        relationships: character.relationships.clone(),
        sprite_paths: character.sprite_paths.clone(),
        absolute_sprite_paths: absolute_asset_paths(project_root, &character.sprite_paths),
        sprite_path: character.sprite_path.clone(),
        absolute_sprite_path: absolute_asset_path(project_root, character.sprite_path.as_deref()),
        live2d_model_path: character.live2d_model_path.clone(),
        absolute_live2d_model_path: absolute_asset_path(
            project_root,
            character.live2d_model_path.as_deref(),
        ),
        model_3d_path: character.model_3d_path.clone(),
        absolute_model_3d_path: absolute_asset_path(
            project_root,
            character.model_3d_path.as_deref(),
        ),
        portrait_path: character.portrait_path.clone(),
        absolute_portrait_path: absolute_asset_path(
            project_root,
            character.portrait_path.as_deref(),
        ),
        knowledge_entries: character.knowledge_entries.clone(),
        knowledge_refs: character.knowledge_refs.clone(),
        emotion_modifiers: character.emotion_modifiers.clone(),
    }
}

fn absolute_asset_path(project_root: &Path, relative: Option<&str>) -> Option<String> {
    let resolved = resolve_project_relative(project_root, relative?).ok()?;
    resolved
        .is_file()
        .then(|| resolved.to_string_lossy().to_string())
}

fn absolute_asset_paths(
    project_root: &Path,
    paths: &HashMap<String, String>,
) -> HashMap<String, String> {
    paths
        .iter()
        .filter_map(|(slot, relative)| {
            absolute_asset_path(project_root, Some(relative))
                .map(|absolute| (slot.clone(), absolute))
        })
        .collect()
}

/// Get all characters.
#[tauri::command]
pub async fn get_characters(state: State<'_, AppState>) -> Result<Vec<CharacterInfo>, String> {
    ensure_project_characters_loaded(&state).await?;
    let project_root = state.current_project_data_root().await?;
    let cm = state.character_manager.read().await;
    let mut characters = Vec::new();

    for (id, character) in cm.all_characters() {
        let character = character.read().await;
        characters.push(character_info(id.clone(), &character, &project_root));
    }

    Ok(characters)
}

/// Get a specific character by ID.
#[tauri::command]
pub async fn get_character(
    state: State<'_, AppState>,
    character_id: String,
) -> Result<CharacterInfo, String> {
    ensure_project_characters_loaded(&state).await?;
    let project_root = state.current_project_data_root().await?;
    let cm = state.character_manager.read().await;
    let character = cm
        .get_character(&character_id)
        .ok_or_else(|| format!("Character not found: {character_id}"))?;

    let character = character.read().await;
    Ok(character_info(
        character.id.clone(),
        &character,
        &project_root,
    ))
}

pub(crate) async fn ensure_project_characters_loaded(state: &AppState) -> Result<(), String> {
    if !state
        .character_manager
        .read()
        .await
        .character_ids()
        .is_empty()
    {
        return Ok(());
    }
    let character_root = state.current_project_data_root().await?.join("characters");
    if !character_root.is_dir() {
        return Ok(());
    }

    let mut loaded = CharacterManager::new();
    loaded
        .load_from_directory(&character_root)
        .await
        .map_err(|error| error.to_string())?;
    let mut active = state.character_manager.write().await;
    if active.character_ids().is_empty() {
        *active = loaded;
    }
    Ok(())
}

/// Load characters from a directory.
#[tauri::command]
pub async fn load_characters(
    state: State<'_, AppState>,
    directory: String,
) -> Result<usize, String> {
    let path = resolve_project_content_dir(&state, &directory, "characters").await?;
    let mut cm = state.character_manager.write().await;
    cm.load_from_directory(&path)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn character_info_keeps_fallback_sprite_separate_from_expression_sprites() {
        let mut character = Character::new("sakura", "Sakura");
        character.emotion = "happy".to_string();
        character.sprite_path = Some("assets/sprites/sakura_base.png".to_string());
        character.sprite_paths.insert(
            "happy".to_string(),
            "assets/sprites/sakura_happy.png".to_string(),
        );

        let info = character_info(character.id.clone(), &character, Path::new("."));

        assert_eq!(
            info.sprite_path.as_deref(),
            Some("assets/sprites/sakura_base.png")
        );
        assert_eq!(
            info.sprite_paths.get("happy").map(String::as_str),
            Some("assets/sprites/sakura_happy.png")
        );
    }

    #[test]
    fn character_info_resolves_existing_assets_inside_the_active_project() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        let project_root =
            std::env::temp_dir().join(format!("monogatari-character-assets-{nonce}"));
        let asset_root = project_root.join("assets").join("characters");
        fs::create_dir_all(&asset_root).expect("asset directory should be created");
        fs::write(asset_root.join("sakura.png"), b"sprite").expect("test sprite should be written");

        let mut character = Character::new("sakura", "Sakura");
        character.portrait_path = Some("assets/characters/sakura.png".to_string());
        character.sprite_path = Some("assets/characters/missing.png".to_string());
        character.sprite_paths.insert(
            "happy".to_string(),
            "assets/characters/sakura.png".to_string(),
        );
        character.sprite_paths.insert(
            "sad".to_string(),
            "assets/characters/missing.png".to_string(),
        );

        let info = character_info(character.id.clone(), &character, &project_root);
        let expected = asset_root.join("sakura.png").to_string_lossy().to_string();

        assert_eq!(
            info.absolute_portrait_path.as_deref(),
            Some(expected.as_str())
        );
        assert_eq!(
            info.absolute_sprite_paths.get("happy").map(String::as_str),
            Some(expected.as_str())
        );
        assert!(!info.absolute_sprite_paths.contains_key("sad"));
        assert!(info.absolute_sprite_path.is_none());
        assert_eq!(
            info.portrait_path.as_deref(),
            Some("assets/characters/sakura.png")
        );

        fs::remove_dir_all(&project_root).expect("test project should be removed");
    }
}
