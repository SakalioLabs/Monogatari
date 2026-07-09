//! Game save and load system.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use llm_core::{EngineError, Result};
use llm_game::{characters::CharacterMemory, dialogue::DialogueRuntimeState};

pub const GAME_SAVE_SCHEMA_V1: &str = "monogatari-game-save/v1";
pub const GAME_SAVE_SCHEMA_V2: &str = "monogatari-game-save/v2";
pub const GAME_SAVE_SCHEMA_V3: &str = "monogatari-game-save/v3";
pub const MAX_GAME_SAVE_BYTES: u64 = 32 * 1024 * 1024;

fn default_game_save_schema() -> String {
    GAME_SAVE_SCHEMA_V1.to_string()
}

fn default_character_emotion() -> String {
    "neutral".to_string()
}

/// Serializable game save data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSave {
    /// Versioned persistence contract. Missing values deserialize as the legacy v1 format.
    #[serde(default = "default_game_save_schema")]
    pub schema: String,
    /// Engine version that created this snapshot.
    #[serde(default)]
    pub engine_version: String,
    /// Unique save identifier.
    pub save_id: String,
    /// Display name for this save.
    pub save_name: String,
    /// When the save was created.
    pub timestamp: DateTime<Utc>,
    /// Current scene name.
    #[serde(default)]
    pub current_scene: Option<String>,
    /// Recent scene ids used by runtime navigation and author diagnostics.
    #[serde(default)]
    pub scene_history: Vec<String>,
    /// Active dialogue script ID.
    #[serde(default)]
    pub current_dialogue_id: Option<String>,
    /// Current dialogue node ID.
    #[serde(default)]
    pub current_node_id: Option<String>,
    /// Full dialogue cursor and dialogue-local variables for v2 snapshots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dialogue_state: Option<DialogueRuntimeState>,
    /// Game variables.
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,
    /// Character states keyed by character id.
    #[serde(default)]
    pub characters: HashMap<String, CharacterSaveState>,
    /// Game flags.
    #[serde(default)]
    pub flags: HashMap<String, bool>,
    /// Tauri chat session payloads keyed by character id.
    #[serde(default)]
    pub chat_sessions: HashMap<String, serde_json::Value>,
    /// Versioned Tauri story progress payload for v3 snapshots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story_progress: Option<serde_json::Value>,
}

/// Saved state for a single character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSaveState {
    #[serde(default = "default_character_emotion")]
    pub emotion: String,
    #[serde(default)]
    pub relationships: HashMap<String, f32>,
    #[serde(default)]
    pub memory_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<CharacterMemory>,
}

impl GameSave {
    pub fn validate_schema(&self) -> Result<()> {
        if matches!(
            self.schema.as_str(),
            GAME_SAVE_SCHEMA_V1 | GAME_SAVE_SCHEMA_V2 | GAME_SAVE_SCHEMA_V3
        ) {
            Ok(())
        } else {
            Err(EngineError::config(
                "save_schema",
                format!("Unsupported game save schema: {}", self.schema),
            ))
        }
    }
}

/// Manages game save/load operations.
pub struct SaveManager {
    save_directory: PathBuf,
}

impl SaveManager {
    /// Create a new save manager with the given save directory.
    pub fn new(save_directory: impl Into<PathBuf>) -> Self {
        Self {
            save_directory: save_directory.into(),
        }
    }

    /// Ensure the save directory exists.
    pub async fn ensure_directory(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.save_directory).await?;
        Ok(())
    }

    /// Save a game state to disk.
    pub async fn save(&self, save: &GameSave) -> Result<()> {
        save.validate_schema()?;
        self.ensure_directory().await?;
        let path = self.safe_save_path(&save.save_id)?;
        let json = serde_json::to_string_pretty(save)?;
        if json.len() as u64 > MAX_GAME_SAVE_BYTES {
            return Err(EngineError::config(
                "save_size",
                format!("Game save exceeds the {MAX_GAME_SAVE_BYTES}-byte size limit."),
            ));
        }
        self.write_staged(&save.save_id, &path, json.as_bytes())
            .await?;
        info!("Game saved: {} -> {}", save.save_name, path.display());
        Ok(())
    }

    /// Load a game state from disk by save ID.
    pub async fn load(&self, save_id: &str) -> Result<GameSave> {
        let path = self.safe_save_path(save_id)?;
        self.recover_backup_if_needed(save_id, &path).await?;
        let metadata = tokio::fs::metadata(&path).await?;
        if metadata.len() > MAX_GAME_SAVE_BYTES {
            return Err(EngineError::config(
                "save_size",
                format!("Game save exceeds the {MAX_GAME_SAVE_BYTES}-byte size limit."),
            ));
        }
        let content = tokio::fs::read_to_string(&path).await?;
        let save: GameSave = serde_json::from_str(&content)?;
        save.validate_schema()?;
        debug!("Loaded save: {}", save.save_name);
        Ok(save)
    }

    /// Delete a save file.
    pub async fn delete(&self, save_id: &str) -> Result<()> {
        let path = self.safe_save_path(save_id)?;
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
            info!("Deleted save: {}", save_id);
        }
        let (temp_path, backup_path) = self.staged_paths(save_id)?;
        for staged_path in [temp_path, backup_path] {
            if staged_path.exists() {
                tokio::fs::remove_file(staged_path).await?;
            }
        }
        Ok(())
    }

    /// List all saves in the save directory.
    pub async fn list_saves(&self) -> Result<Vec<GameSave>> {
        let mut saves = Vec::new();
        if !self.save_directory.exists() {
            return Ok(saves);
        }

        let mut entries = tokio::fs::read_dir(&self.save_directory).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !is_save_json_path(&path) {
                continue;
            }

            let Some(file_save_id) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if tokio::fs::metadata(&path)
                .await
                .map(|metadata| metadata.len() > MAX_GAME_SAVE_BYTES)
                .unwrap_or(true)
            {
                debug!("Ignored oversized or unreadable save {}", path.display());
                continue;
            }

            match tokio::fs::read_to_string(&path).await {
                Ok(content) => match serde_json::from_str::<GameSave>(&content) {
                    Ok(save) if save.save_id == file_save_id && save.validate_schema().is_ok() => {
                        saves.push(save)
                    }
                    Ok(save) => debug!(
                        "Ignored save {} with invalid schema or mismatched embedded id {}",
                        path.display(),
                        save.save_id
                    ),
                    Err(e) => debug!("Failed to parse save {}: {}", path.display(), e),
                },
                Err(e) => debug!("Failed to read save {}: {}", path.display(), e),
            }
        }

        // Sort by timestamp, newest first
        saves.sort_by_key(|a| std::cmp::Reverse(a.timestamp));
        Ok(saves)
    }

    /// Create a new save with auto-generated ID and timestamp.
    pub fn create_save(
        name: impl Into<String>,
        scene: Option<String>,
        dialogue_id: Option<String>,
        node_id: Option<String>,
    ) -> GameSave {
        Self::create_save_with_id(
            Uuid::new_v4().to_string(),
            name,
            scene,
            dialogue_id,
            node_id,
        )
        .expect("generated UUID save ids must be valid")
    }

    /// Create a save with a stable caller-provided id, used by quick and auto-save slots.
    pub fn create_save_with_id(
        save_id: impl Into<String>,
        name: impl Into<String>,
        scene: Option<String>,
        dialogue_id: Option<String>,
        node_id: Option<String>,
    ) -> Result<GameSave> {
        let save_id = save_id.into();
        if !is_valid_save_id(&save_id) {
            return Err(EngineError::config(
                "save_id",
                "Save id can contain only ASCII letters, numbers, underscores, or hyphens.",
            ));
        }

        Ok(GameSave {
            schema: GAME_SAVE_SCHEMA_V3.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            save_id,
            save_name: name.into(),
            timestamp: Utc::now(),
            current_scene: scene,
            scene_history: Vec::new(),
            current_dialogue_id: dialogue_id,
            current_node_id: node_id,
            dialogue_state: None,
            variables: HashMap::new(),
            characters: HashMap::new(),
            flags: HashMap::new(),
            chat_sessions: HashMap::new(),
            story_progress: None,
        })
    }

    fn safe_save_path(&self, save_id: &str) -> Result<PathBuf> {
        if !is_valid_save_id(save_id) {
            return Err(EngineError::config(
                "save_id",
                "Save id cannot contain path separators, dots, whitespace, or control characters.",
            ));
        }

        let root = self.save_directory.clone();
        let path = root.join(format!("{save_id}.json"));
        if path.parent() != Some(root.as_path()) {
            return Err(EngineError::config(
                "save_id",
                "Save path must stay inside the save directory.",
            ));
        }

        Ok(path)
    }

    fn staged_paths(&self, save_id: &str) -> Result<(PathBuf, PathBuf)> {
        if !is_valid_save_id(save_id) {
            return Err(EngineError::config(
                "save_id",
                "Save id cannot contain path separators, dots, whitespace, or control characters.",
            ));
        }
        Ok((
            self.save_directory.join(format!(".{save_id}.tmp")),
            self.save_directory.join(format!(".{save_id}.bak")),
        ))
    }

    async fn write_staged(&self, save_id: &str, path: &Path, bytes: &[u8]) -> Result<()> {
        let (temp_path, backup_path) = self.staged_paths(save_id)?;
        if temp_path.exists() {
            tokio::fs::remove_file(&temp_path).await?;
        }
        tokio::fs::write(&temp_path, bytes).await?;

        if !path.exists() {
            tokio::fs::rename(&temp_path, path).await?;
            return Ok(());
        }

        if backup_path.exists() {
            tokio::fs::remove_file(&backup_path).await?;
        }
        tokio::fs::rename(path, &backup_path).await?;
        if let Err(error) = tokio::fs::rename(&temp_path, path).await {
            let _ = tokio::fs::rename(&backup_path, path).await;
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(error.into());
        }
        if backup_path.exists() {
            if let Err(error) = tokio::fs::remove_file(&backup_path).await {
                warn!(
                    "Save replacement succeeded but backup cleanup failed for {}: {}",
                    backup_path.display(),
                    error
                );
            }
        }
        Ok(())
    }

    async fn recover_backup_if_needed(&self, save_id: &str, path: &Path) -> Result<()> {
        if path.exists() {
            return Ok(());
        }
        let (_, backup_path) = self.staged_paths(save_id)?;
        if backup_path.exists() {
            tokio::fs::rename(backup_path, path).await?;
        }
        Ok(())
    }

    /// Get the save directory path.
    pub fn save_directory(&self) -> &Path {
        &self.save_directory
    }
}

fn is_save_json_path(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("json")
        && path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(is_valid_save_id)
}

fn is_valid_save_id(save_id: &str) -> bool {
    !save_id.is_empty()
        && save_id.len() <= 128
        && save_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_save_manager_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn test_save(save_id: &str) -> GameSave {
        GameSave {
            schema: GAME_SAVE_SCHEMA_V3.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            save_id: save_id.to_string(),
            save_name: "Test Save".to_string(),
            timestamp: Utc::now(),
            current_scene: Some("test_scene".to_string()),
            scene_history: vec!["test_scene".to_string()],
            current_dialogue_id: None,
            current_node_id: None,
            dialogue_state: None,
            variables: HashMap::new(),
            characters: HashMap::new(),
            flags: HashMap::new(),
            chat_sessions: HashMap::new(),
            story_progress: None,
        }
    }

    #[test]
    fn new_and_stable_slot_saves_use_v3_schema() {
        let generated = SaveManager::create_save("Manual", None, None, None);
        let quick = SaveManager::create_save_with_id(
            "quick_save_0",
            "Quick Save",
            Some("park".to_string()),
            None,
            None,
        )
        .unwrap();

        assert_eq!(generated.schema, GAME_SAVE_SCHEMA_V3);
        assert_eq!(quick.schema, GAME_SAVE_SCHEMA_V3);
        assert_eq!(quick.save_id, "quick_save_0");
        assert!(
            SaveManager::create_save_with_id("../settings", "Unsafe", None, None, None).is_err()
        );
    }

    #[test]
    fn legacy_save_payloads_deserialize_with_v1_defaults() {
        let legacy = serde_json::json!({
            "save_id": "legacy_slot",
            "save_name": "Legacy",
            "timestamp": Utc::now(),
            "current_scene": "park",
            "current_dialogue_id": null,
            "current_node_id": null,
            "variables": {"score": "7"},
            "characters": {
                "sakura": {
                    "emotion": "happy",
                    "relationships": {"player": 0.5},
                    "memory_count": 2
                }
            },
            "flags": {"met_sakura": true}
        });

        let save: GameSave = serde_json::from_value(legacy).unwrap();

        assert_eq!(save.schema, GAME_SAVE_SCHEMA_V1);
        assert!(save.validate_schema().is_ok());
        assert!(save.scene_history.is_empty());
        assert!(save.dialogue_state.is_none());
        assert!(save.chat_sessions.is_empty());
        assert!(save.story_progress.is_none());
        assert!(save.characters["sakura"].memory.is_none());
    }

    #[tokio::test]
    async fn save_rejects_ids_that_escape_save_directory() {
        let root = temp_root("reject_save_escape");
        std::fs::create_dir_all(&root).unwrap();
        let outside = root.join("settings.json");
        std::fs::write(&outside, b"keep me").unwrap();
        let manager = SaveManager::new(root.join("saves"));
        let save = test_save("../settings");

        let error = manager.save(&save).await.unwrap_err().to_string();

        assert!(error.contains("save_id"));
        assert_eq!(std::fs::read(&outside).unwrap(), b"keep me");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn load_rejects_ids_that_escape_save_directory() {
        let root = temp_root("reject_load_escape");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("settings.json"),
            serde_json::to_string(&test_save("settings")).unwrap(),
        )
        .unwrap();
        let manager = SaveManager::new(root.join("saves"));

        let error = manager.load("../settings").await.unwrap_err().to_string();

        assert!(error.contains("save_id"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn delete_rejects_ids_that_escape_save_directory() {
        let root = temp_root("reject_delete_escape");
        std::fs::create_dir_all(&root).unwrap();
        let outside = root.join("settings.json");
        std::fs::write(&outside, b"keep me").unwrap();
        let manager = SaveManager::new(root.join("saves"));

        let error = manager.delete("../settings").await.unwrap_err().to_string();

        assert!(error.contains("save_id"));
        assert_eq!(std::fs::read(&outside).unwrap(), b"keep me");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn list_saves_ignores_invalid_or_mismatched_save_ids() {
        let root = temp_root("list_filters_ids");
        let saves_dir = root.join("saves");
        std::fs::create_dir_all(&saves_dir).unwrap();
        let manager = SaveManager::new(&saves_dir);
        manager.save(&test_save("slot_1")).await.unwrap();
        std::fs::write(
            saves_dir.join("evil.json"),
            serde_json::to_string(&test_save("../settings")).unwrap(),
        )
        .unwrap();
        std::fs::write(
            saves_dir.join("mismatch.json"),
            serde_json::to_string(&test_save("other_slot")).unwrap(),
        )
        .unwrap();

        let saves = manager.list_saves().await.unwrap();

        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].save_id, "slot_1");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn stable_slot_overwrite_replaces_save_without_staged_files() {
        let root = temp_root("stable_slot_overwrite");
        let saves_dir = root.join("saves");
        let manager = SaveManager::new(&saves_dir);
        let mut first = test_save("quick_save_0");
        first.save_name = "First".to_string();
        manager.save(&first).await.unwrap();
        let mut second = first.clone();
        second.save_name = "Second".to_string();
        manager.save(&second).await.unwrap();

        let restored = manager.load("quick_save_0").await.unwrap();

        assert_eq!(restored.save_name, "Second");
        assert!(!saves_dir.join(".quick_save_0.tmp").exists());
        assert!(!saves_dir.join(".quick_save_0.bak").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
