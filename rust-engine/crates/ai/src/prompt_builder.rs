//! Prompt builder for constructing LLM prompts from game context.

/// A fluent builder for constructing LLM prompts from game context.
///
/// Combines system prompt, character context, knowledge context,
/// world context, and message history into a structured format.
#[derive(Debug, Clone, Default)]
pub struct PromptBuilder {
    system_prompt: Option<String>,
    character_context: Option<String>,
    knowledge_context: Option<String>,
    world_context: Option<String>,
    messages: Vec<(String, String)>, // (role, content)
}

impl PromptBuilder {
    /// Create a new empty prompt builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the system prompt.
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set character context (personality, background, etc.).
    pub fn character_context(mut self, context: impl Into<String>) -> Self {
        self.character_context = Some(context.into());
        self
    }

    /// Set knowledge context (relevant lore, world info).
    pub fn knowledge_context(mut self, context: impl Into<String>) -> Self {
        self.knowledge_context = Some(context.into());
        self
    }

    /// Set world context (setting, atmosphere).
    pub fn world_context(mut self, context: impl Into<String>) -> Self {
        self.world_context = Some(context.into());
        self
    }

    /// Add a user message to the conversation history.
    pub fn user_message(mut self, message: impl Into<String>) -> Self {
        self.messages.push(("User".to_string(), message.into()));
        self
    }

    /// Add an assistant message to the conversation history.
    pub fn assistant_message(mut self, message: impl Into<String>) -> Self {
        self.messages
            .push(("Assistant".to_string(), message.into()));
        self
    }

    /// Build the final prompt string.
    ///
    /// Output format:
    /// ```text
    /// [System]
    /// <system prompt>
    /// <character context>
    /// <knowledge context>
    /// <world context>
    ///
    /// [User]
    /// <first user message>
    ///
    /// [Assistant]
    /// <first assistant response>
    ///
    /// [User]
    /// <latest user message>
    /// ```
    pub fn build(self) -> String {
        let mut parts = Vec::new();

        // System section
        let mut system_parts = Vec::new();
        if let Some(sp) = &self.system_prompt {
            system_parts.push(sp.clone());
        }
        if let Some(cc) = &self.character_context {
            system_parts.push(cc.clone());
        }
        if let Some(kc) = &self.knowledge_context {
            system_parts.push(kc.clone());
        }
        if let Some(wc) = &self.world_context {
            system_parts.push(wc.clone());
        }

        if !system_parts.is_empty() {
            parts.push(format!("[System]\n{}", system_parts.join("\n\n")));
        }

        // Conversation messages (skip empty ones)
        for (role, content) in &self.messages {
            if !content.is_empty() {
                parts.push(format!("[{role}]\n{content}"));
            }
        }

        parts.join("\n\n")
    }

    /// Check if the builder has any content.
    pub fn is_empty(&self) -> bool {
        self.system_prompt.is_none()
            && self.character_context.is_none()
            && self.knowledge_context.is_none()
            && self.world_context.is_none()
            && self.messages.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_builder() {
        let prompt = PromptBuilder::new().build();
        assert!(prompt.is_empty());
    }

    #[test]
    fn test_system_only() {
        let prompt = PromptBuilder::new()
            .system_prompt("You are a helpful assistant.")
            .build();
        assert!(prompt.contains("[System]"));
        assert!(prompt.contains("You are a helpful assistant."));
    }

    #[test]
    fn test_full_conversation() {
        let prompt = PromptBuilder::new()
            .system_prompt("You are a character in a game.")
            .character_context("Name: Sakura, Personality: Cheerful")
            .knowledge_context("Setting: Cherry blossom park")
            .user_message("Hello!")
            .assistant_message("Hi there! Nice to meet you!")
            .user_message("How are you?")
            .build();

        assert!(prompt.contains("[System]"));
        assert!(prompt.contains("Sakura"));
        assert!(prompt.contains("Cherry blossom"));
        assert!(prompt.contains("[User]"));
        assert!(prompt.contains("[Assistant]"));
    }
}
