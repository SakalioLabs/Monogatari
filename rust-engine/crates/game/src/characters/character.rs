//! Character definition and management.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::info;

use llm_core::Result;

use super::memory::CharacterMemory;
use super::personality::Personality;

/// A game character with personality, memory, and relationship data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    /// Unique character identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Short description for UI display.
    pub description: String,
    /// Detailed background/backstory.
    pub background: String,
    /// Personality traits.
    pub personality: Personality,
    /// Character's memories.
    pub memory: CharacterMemory,
    /// Current emotion state.
    pub emotion: String,
    /// Relationship scores with other characters (id -> score, -1.0 to 1.0).
    pub relationships: HashMap<String, f32>,
    /// Paths to sprite/model assets per emotion state.
    pub sprite_paths: HashMap<String, String>,
    /// Live2D model path (if using Live2D).
    pub live2d_model_path: Option<String>,
}

impl Character {
    /// Create a new character with default values.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            background: String::new(),
            personality: Personality::default(),
            memory: CharacterMemory::default(),
            emotion: "neutral".to_string(),
            relationships: HashMap::new(),
            sprite_paths: HashMap::new(),
            live2d_model_path: None,
        }
    }

    /// Build a system prompt for LLM inference from this character's data.
    pub fn build_system_prompt(&self) -> String {
        let mut parts = vec![
            format!("You are the character \"{}\" in a visual novel game.", self.name),
            format!("Description: {}", self.description),
            format!("Background: {}", self.background),
        ];

        parts.push(self.personality.to_prompt_description());

        // Add relationship context
        if !self.relationships.is_empty() {
            let rel_str: Vec<String> = self
                .relationships
                .iter()
                .map(|(id, score)| format!("{}: {:.1}", id, score))
                .collect();
            parts.push(format!("Relationships: {}", rel_str.join(", ")));
        }

        // Add recent memories as context
        let recent = self.memory.get_recent(5);
        if !recent.is_empty() {
            let mem_str: Vec<String> = recent.iter().map(|m| m.content.clone()).collect();
            parts.push(format!("Recent memories: {}", mem_str.join("; ")));
        }

        parts.join("\n")
    }

    /// Set the character's emotion, updating the current state.
    pub fn set_emotion(&mut self, emotion: impl Into<String>) {
        self.emotion = emotion.into();
        self.personality.current_emotion = self.emotion.clone();
    }

    /// Update relationship score with another character.
    pub fn update_relationship(&mut self, other_id: &str, delta: f32) {
        let score = self.relationships.entry(other_id.to_string()).or_insert(0.0);
        *score = (*score + delta).clamp(-1.0, 1.0);
    }

    /// Add a memory to this character.
    pub fn add_memory(
        &mut self,
        content: String,
        memory_type: super::memory::MemoryType,
        importance: f32,
        tags: Vec<String>,
    ) {
        self.memory.add_memory(content, memory_type, importance, tags);
    }
}

/// Service that manages all characters in the game.
pub struct CharacterManager {
    characters: HashMap<String, Arc<RwLock<Character>>>,
}

impl CharacterManager {
    /// Create a new empty character manager.
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
        }
    }

    /// Add a character to the manager.
    pub fn add_character(&mut self, character: Character) {
        let id = character.id.clone();
        info!("Added character: {} ({})", character.name, id);
        self.characters.insert(id, Arc::new(RwLock::new(character)));
    }

    /// Get a character by ID.
    pub fn get_character(&self, id: &str) -> Option<Arc<RwLock<Character>>> {
        self.characters.get(id).cloned()
    }

    /// Load characters from a JSON file.
    pub async fn load_from_file(&mut self, path: &Path) -> Result<usize> {
        let content = tokio::fs::read_to_string(path).await?;
        let characters: Vec<Character> = serde_json::from_str(&content)?;
        let count = characters.len();
        for character in characters {
            self.add_character(character);
        }
        info!("Loaded {} characters from {}", count, path.display());
        Ok(count)
    }

    /// Load all character JSON files from a directory.
    pub async fn load_from_directory(&mut self, dir: &Path) -> Result<usize> {
        let mut total = 0;
        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                total += self.load_from_file(&path).await?;
            }
        }
        Ok(total)
    }

    /// Get all character IDs.
    pub fn character_ids(&self) -> Vec<String> {
        self.characters.keys().cloned().collect()
    }

    /// Get all characters.
    pub fn all_characters(&self) -> &HashMap<String, Arc<RwLock<Character>>> {
        &self.characters
    }
}

impl Default for CharacterManager {
    fn default() -> Self {
        Self::new()
    }
}
