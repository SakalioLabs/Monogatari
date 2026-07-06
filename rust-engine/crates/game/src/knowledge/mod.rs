//! Knowledge base system for world lore and context.

pub mod context_builder;
pub mod knowledge_base;
pub mod knowledge_entry;

pub use context_builder::ContextBuilder;
pub use knowledge_base::KnowledgeBase;
pub use knowledge_entry::{KnowledgeCategory, KnowledgeEntry};
