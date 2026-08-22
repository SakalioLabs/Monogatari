//! Dialogue system: scripts, nodes, and dialogue flow management.

pub mod dialogue_manager;
pub mod dialogue_node;
pub mod dialogue_script;

pub use dialogue_manager::{
    DialogueChoiceEffects, DialogueGenerationRequest, DialogueManager, DialogueRuntimeState,
    DialogueScriptSummary, LLMInferenceCallback,
};
pub use dialogue_node::{Choice, DialogueFreeTalk, DialogueNode, DialogueResponseGeneration};
pub use dialogue_script::DialogueScript;
