//! Knowledge entry types.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Categories of knowledge entries.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KnowledgeCategory {
    Location,
    Character,
    Item,
    Lore,
    Event,
    Rule,
    Other(String),
}

impl Serialize for KnowledgeCategory {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            KnowledgeCategory::Location => "location",
            KnowledgeCategory::Character => "character",
            KnowledgeCategory::Item => "item",
            KnowledgeCategory::Lore => "lore",
            KnowledgeCategory::Event => "event",
            KnowledgeCategory::Rule => "rule",
            KnowledgeCategory::Other(value) => value,
        };
        serializer.serialize_str(value)
    }
}

impl<'de> Deserialize<'de> for KnowledgeCategory {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let normalized = value.trim().to_ascii_lowercase();
        Ok(match normalized.as_str() {
            "location" => KnowledgeCategory::Location,
            "character" => KnowledgeCategory::Character,
            "item" => KnowledgeCategory::Item,
            "lore" | "world_lore" => KnowledgeCategory::Lore,
            "event" => KnowledgeCategory::Event,
            "rule" => KnowledgeCategory::Rule,
            _ => KnowledgeCategory::Other(normalized),
        })
    }
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
