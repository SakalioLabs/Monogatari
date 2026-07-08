//! Game save and load system.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use llm_core::{EngineError, Result};

/// Serializable game save data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSave {
    /// Unique save identifier.
    pub save_id: String,
    /// Display name for this save.
    pub save_name: String,
    /// When the save was created.
    pub timestamp: DateTime<Utc>,
    /// Current scene name.
    pub current_scene: Option<String>,
    /// Active dialogue script ID.
    pub current_dialogue_id: Option<String>,
    /// Current dialogue node ID.
    pub current_node_id: Option<String>,
    /// Game variables.
    pub variables: HashMap<String, serde_json::Value>,
    /// Character states (id -> (emotion, relationships, memory_count)).
    pub characters: HashMap<String, CharacterSaveState>,
    /// Game flags.
    pub flags: HashMap<String, bool>,
}

/// Saved state for a single character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSaveState {
    pub emotion: String,
    pub relationships: HashMap<String, f32>,
    pub memory_count: usize,
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
        self.ensure_directory().await?;
        let path = self.safe_save_path(&save.save_id)?;
        let json = serde_json::to_string_pretty(save)?;
        tokio::fs::write(&path, json).await?;
        info!("Game saved: {} -> {}", save.save_name, path.display());
        Ok(())
    }

    /// Load a game state from disk by save ID.
    pub async fn load(&self, save_id: &str) -> Result<GameSave> {
        let path = self.safe_save_path(save_id)?;
        let content = tokio::fs::read_to_string(&path).await?;
        let save: GameSave = serde_json::from_str(&content)?;
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

            match tokio::fs::read_to_string(&path).await {
                Ok(content) => match serde_json::from_str::<GameSave>(&content) {
                    Ok(save) if save.save_id == file_save_id => saves.push(save),
                    Ok(save) => debug!(
                        "Ignored save {} with mismatched embedded id {}",
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
        GameSave {
            save_id: Uuid::new_v4().to_string(),
            save_name: name.into(),
            timestamp: Utc::now(),
            current_scene: scene,
            current_dialogue_id: dialogue_id,
            current_node_id: node_id,
            variables: HashMap::new(),
            characters: HashMap::new(),
            flags: HashMap::new(),
        }
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
            save_id: save_id.to_string(),
            save_name: "Test Save".to_string(),
            timestamp: Utc::now(),
            current_scene: Some("test_scene".to_string()),
            current_dialogue_id: None,
            current_node_id: None,
            variables: HashMap::new(),
            characters: HashMap::new(),
            flags: HashMap::new(),
        }
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
}
