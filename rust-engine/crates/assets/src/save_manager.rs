//! Game save and load system.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use llm_core::Result;

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
        let path = self.save_path(&save.save_id);
        let json = serde_json::to_string_pretty(save)?;
        tokio::fs::write(&path, json).await?;
        info!("Game saved: {} -> {}", save.save_name, path.display());
        Ok(())
    }

    /// Load a game state from disk by save ID.
    pub async fn load(&self, save_id: &str) -> Result<GameSave> {
        let path = self.save_path(save_id);
        let content = tokio::fs::read_to_string(&path).await?;
        let save: GameSave = serde_json::from_str(&content)?;
        debug!("Loaded save: {}", save.save_name);
        Ok(save)
    }

    /// Delete a save file.
    pub async fn delete(&self, save_id: &str) -> Result<()> {
        let path = self.save_path(save_id);
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
            if path.extension().is_some_and(|e| e == "json") {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => match serde_json::from_str::<GameSave>(&content) {
                        Ok(save) => saves.push(save),
                        Err(e) => debug!("Failed to parse save {}: {}", path.display(), e),
                    },
                    Err(e) => debug!("Failed to read save {}: {}", path.display(), e),
                }
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

    fn save_path(&self, save_id: &str) -> PathBuf {
        self.save_directory.join(format!("{save_id}.json"))
    }

    /// Get the save directory path.
    pub fn save_directory(&self) -> &Path {
        &self.save_directory
    }
}
