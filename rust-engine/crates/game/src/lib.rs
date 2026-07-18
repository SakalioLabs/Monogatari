//! # LLM Galgame Engine - Game
//!
//! Game logic: characters, dialogue trees, knowledge base, and scene management.

pub mod characters;
pub mod dialogue;
pub mod knowledge;
pub mod scene_roleplay;
pub mod scenes;
pub mod script;

pub use characters::{Character, CharacterManager, CharacterMemory, Personality};
pub use dialogue::{DialogueManager, DialogueNode, DialogueScript};
pub use knowledge::{ContextBuilder, KnowledgeBase, KnowledgeEntry};
pub use scene_roleplay::{SceneRoleplayDefinition, SceneRoleplaySession};
pub use scenes::{GameScene, Scene, SceneManager, TitleScene};
