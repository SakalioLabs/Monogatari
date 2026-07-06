//! Context builder for combining game state into LLM context.

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::characters::Character;
use crate::knowledge::KnowledgeBase;

/// Builds LLM context strings from game state.
pub struct ContextBuilder {
    max_knowledge_entries: usize,
    max_memory_entries: usize,
}

impl ContextBuilder {
    /// Create a new context builder with default limits.
    pub fn new() -> Self {
        Self {
            max_knowledge_entries: 5,
            max_memory_entries: 5,
        }
    }

    /// Set the maximum number of knowledge entries to include.
    pub fn with_max_knowledge(mut self, max: usize) -> Self {
        self.max_knowledge_entries = max;
        self
    }

    /// Set the maximum number of memory entries to include.
    pub fn with_max_memory(mut self, max: usize) -> Self {
        self.max_memory_entries = max;
        self
    }

    /// Build character context from a Character reference.
    pub fn build_character_context(&self, character: &Character) -> String {
        let mut parts = vec![
            format!("Character: {}", character.name),
            format!("Description: {}", character.description),
            format!("Background: {}", character.background),
            character.personality.to_prompt_description(),
        ];

        // Add recent memories
        let recent = character.memory.get_recent(self.max_memory_entries);
        if !recent.is_empty() {
            let mem_str: Vec<String> = recent
                .iter()
                .map(|m| format!("- {}", m.content))
                .collect();
            parts.push(format!("Recent memories:\n{}", mem_str.join("\n")));
        }

        parts.join("\n")
    }

    /// Build knowledge context by searching the knowledge base.
    pub fn build_knowledge_context(&self, knowledge: &KnowledgeBase, query: &str) -> String {
        let entries = knowledge.search(query, self.max_knowledge_entries);
        if entries.is_empty() {
            return String::new();
        }

        let parts: Vec<String> = entries
            .iter()
            .map(|e| format!("[{}] {}: {}", format!("{:?}", e.category).to_lowercase(), e.title, e.content))
            .collect();

        format!("Relevant knowledge:\n{}", parts.join("\n"))
    }

    /// Build a combined context string from character and knowledge.
    pub async fn build_full_context(
        &self,
        character: Option<&Arc<RwLock<Character>>>,
        knowledge: &KnowledgeBase,
        query: &str,
    ) -> String {
        let mut parts = Vec::new();

        if let Some(character) = character {
            let character = character.read().await;
            parts.push(self.build_character_context(&character));
        }

        let knowledge_ctx = self.build_knowledge_context(knowledge, query);
        if !knowledge_ctx.is_empty() {
            parts.push(knowledge_ctx);
        }

        parts.join("\n\n")
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
