//! Event types for engine-wide communication.

use serde::{Deserialize, Serialize};

/// Fired when a dialogue conversation starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueStartedEvent {
    pub dialogue_id: String,
    pub character_id: Option<String>,
}

/// Fired when a dialogue conversation ends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueEndedEvent {
    pub dialogue_id: String,
}

/// Fired when the player makes a choice in dialogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceMadeEvent {
    pub dialogue_id: String,
    pub choice_index: usize,
    pub choice_text: String,
}

/// Fired when the active scene changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneChangedEvent {
    pub old_scene: Option<String>,
    pub new_scene: String,
}

/// Fired when a character's emotion state changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterEmotionChangedEvent {
    pub character_id: String,
    pub old_emotion: String,
    pub new_emotion: String,
}

/// Fired when the knowledge base is queried.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeQueriedEvent {
    pub query: String,
    pub result_count: usize,
}

/// Fired when an LLM response is received.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponseEvent {
    pub engine_name: String,
    pub response_length: usize,
    pub duration_ms: u64,
}

/// Fired when a game is saved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSaveEvent {
    pub save_id: String,
    pub save_name: String,
}

/// Fired when a game is loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLoadEvent {
    pub save_id: String,
}

/// Fired when LLM inference starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMInferenceStartedEvent {
    pub engine_name: String,
}

/// Fired when LLM inference completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMInferenceCompletedEvent {
    pub engine_name: String,
    pub success: bool,
    pub duration_ms: u64,
}
