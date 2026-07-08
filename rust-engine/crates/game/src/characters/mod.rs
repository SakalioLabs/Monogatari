//! Character system: personality, memory, and management.

pub mod character;
pub mod memory;
pub mod personality;

pub use character::{Character, CharacterKnowledgeEntry, CharacterManager};
pub use memory::CharacterMemory;
pub use personality::Personality;
