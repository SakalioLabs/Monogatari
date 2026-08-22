//! Dialogue node and choice definitions.

use serde::{Deserialize, Serialize};

const DEFAULT_RESPONSE_MAX_CHARACTERS: usize = 240;
const DEFAULT_RESPONSE_MAX_SENTENCES: usize = 3;
const DEFAULT_FREE_TALK_MAX_TURNS: u8 = 4;
const DEFAULT_FREE_TALK_MAX_CHARACTERS: usize = 240;

fn default_response_max_characters() -> usize {
    DEFAULT_RESPONSE_MAX_CHARACTERS
}

fn default_response_max_sentences() -> usize {
    DEFAULT_RESPONSE_MAX_SENTENCES
}

fn default_free_talk_max_turns() -> u8 {
    DEFAULT_FREE_TALK_MAX_TURNS
}

fn default_free_talk_max_characters() -> usize {
    DEFAULT_FREE_TALK_MAX_CHARACTERS
}

/// Author-owned constraints for one generated character reaction.
///
/// The dialogue graph still owns every transition. This contract only controls
/// the wording shown for the current node and always falls back to `text`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueResponseGeneration {
    /// Optional character override. When omitted, the node speaker is used.
    #[serde(default)]
    pub character_id: Option<String>,
    /// The authored story beat that the response must acknowledge.
    pub context: String,
    /// Terms that must appear in a generated response when supplied.
    #[serde(default)]
    pub grounding_markers: Vec<String>,
    /// Terms that invalidate a generated response when supplied.
    #[serde(default)]
    pub forbidden_markers: Vec<String>,
    /// Maximum visible Unicode characters in the generated response.
    #[serde(default = "default_response_max_characters")]
    pub max_characters: usize,
    /// Maximum authored sentence count in the generated response.
    #[serde(default = "default_response_max_sentences")]
    pub max_sentences: usize,
}

/// A contained, optional player-to-character conversation at a chapter beat.
///
/// Free talk is deliberately separate from dialogue transitions, relationship
/// changes, event evaluation, and campaign routing. Closing it resumes the
/// exact authored node that opened it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueFreeTalk {
    /// Character available for this contained conversation.
    pub character_id: String,
    /// The authored scene boundary the conversation must stay within.
    pub context: String,
    /// Optional compact UI label for the conversation entry point.
    #[serde(default)]
    pub title: Option<String>,
    /// Optional initial line shown before the player writes a message.
    #[serde(default)]
    pub opening_text: Option<String>,
    /// Safe authored fallback when inference is unavailable or rejected.
    pub fallback_text: String,
    /// Maximum player turns before returning to the authored story.
    #[serde(default = "default_free_talk_max_turns")]
    pub max_turns: u8,
    /// Maximum visible Unicode characters in each NPC reply.
    #[serde(default = "default_free_talk_max_characters")]
    pub max_characters: usize,
}

/// A choice presented to the player during dialogue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueNode {
    /// Unique node identifier within the dialogue.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// ID of the character speaking (if any).
    #[serde(default)]
    pub speaker_id: Option<String>,
    /// Optional project scene to activate when this node is entered.
    #[serde(default)]
    pub scene_id: Option<String>,
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
    /// Bounded, character-aware generated response for a fixed-route node.
    #[serde(default)]
    pub response_generation: Option<DialogueResponseGeneration>,
    /// Optional contained free-talk entry available while this node is active.
    #[serde(default)]
    pub free_talk: Option<DialogueFreeTalk>,
    /// Explicitly marks a terminal node for route authoring and analytics.
    #[serde(default)]
    pub is_ending: bool,
    /// Optional project-defined ending classification such as `good` or `best`.
    #[serde(default)]
    pub ending_type: Option<String>,
}
