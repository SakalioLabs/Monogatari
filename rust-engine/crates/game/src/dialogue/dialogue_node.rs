//! Dialogue node and choice definitions.

use serde::{Deserialize, Serialize};

/// A choice presented to the player during dialogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Display text for this choice.
    pub text: String,
    /// ID of the node to jump to when this choice is selected.
    pub next_node_id: String,
    /// Relationship score changes when this choice is selected (character_id -> delta).
    #[serde(default)]
    pub relationship_changes: std::collections::HashMap<String, f32>,
    /// Condition expression that must be true for this choice to appear.
    #[serde(default)]
    pub condition: Option<String>,
}

/// A single node in a dialogue tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    /// Unique node identifier within the dialogue.
    #[serde(default)]
    pub id: String,
    /// ID of the character speaking (if any).
    #[serde(default)]
    pub speaker_id: Option<String>,
    /// The dialogue text to display.
    pub text: String,
    /// ID of the next node (for linear progression).
    #[serde(default)]
    pub next_node_id: Option<String>,
    /// Choices for branching dialogue.
    #[serde(default)]
    pub choices: Vec<Choice>,
    /// Condition expression that must be true for this node to play.
    #[serde(default)]
    pub condition: Option<String>,
    /// Script to execute when this node is reached.
    #[serde(default)]
    pub script: Option<String>,
    /// Emotion to set on the speaking character.
    #[serde(default)]
    pub emotion: Option<String>,
    /// Whether to use LLM to generate dialogue content.
    #[serde(default)]
    pub use_llm: bool,
    /// Prompt to send to LLM (if use_llm is true).
    #[serde(default)]
    pub llm_prompt: Option<String>,
    /// System prompt override for LLM.
    #[serde(default)]
    pub llm_system_prompt: Option<String>,
}
