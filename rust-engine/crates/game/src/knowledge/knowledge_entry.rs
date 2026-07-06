//! Knowledge entry types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Categories of knowledge entries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeCategory {
    Location,
    Character,
    Item,
    Lore,
    Event,
    Rule,
    Other(String),
}

/// A single knowledge base entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// Unique identifier.
    pub id: String,
    /// Category of this entry.
    pub category: KnowledgeCategory,
    /// Short title.
    pub title: String,
    /// Detailed content.
    pub content: String,
    /// Searchable tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Importance score (0.0 to 1.0).
    #[serde(default = "default_importance")]
    pub importance: f32,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// IDs of related entries.
    #[serde(default)]
    pub related_entries: Vec<String>,
}

fn default_importance() -> f32 {
    0.5
}

impl KnowledgeEntry {
    /// Create a simple knowledge entry.
    pub fn new(
        id: impl Into<String>,
        category: KnowledgeCategory,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            title: title.into(),
            content: content.into(),
            tags: Vec::new(),
            importance: 0.5,
            metadata: HashMap::new(),
            related_entries: Vec::new(),
        }
    }
}
