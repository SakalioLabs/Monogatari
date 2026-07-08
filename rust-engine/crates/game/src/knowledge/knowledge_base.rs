//! Knowledge base with keyword-based relevance search.

use std::collections::HashMap;
use std::path::Path;

use tracing::info;

use llm_core::Result;

use super::knowledge_entry::{KnowledgeCategory, KnowledgeEntry};

/// Knowledge base that stores and indexes knowledge entries.
pub struct KnowledgeBase {
    entries: HashMap<String, KnowledgeEntry>,
    tag_index: HashMap<String, Vec<String>>, // tag -> entry IDs
    category_index: HashMap<KnowledgeCategory, Vec<String>>, // category -> entry IDs
}

impl KnowledgeBase {
    /// Create a new empty knowledge base.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            tag_index: HashMap::new(),
            category_index: HashMap::new(),
        }
    }

    /// Add an entry to the knowledge base.
    pub fn add_entry(&mut self, entry: KnowledgeEntry) {
        let id = entry.id.clone();

        // Index by tags
        for tag in &entry.tags {
            self.tag_index
                .entry(tag.to_lowercase())
                .or_default()
                .push(id.clone());
        }

        // Index by category
        self.category_index
            .entry(entry.category.clone())
            .or_default()
            .push(id.clone());

        self.entries.insert(id, entry);
    }

    /// Get an entry by ID.
    pub fn get_entry(&self, id: &str) -> Option<&KnowledgeEntry> {
        self.entries.get(id)
    }

    /// Search for entries by keyword relevance.
    ///
    /// Scoring: title matches = 3x, tag matches = 2x, content matches = 1x,
    /// scaled by the entry's importance score.
    pub fn search(&self, query: &str, limit: usize) -> Vec<&KnowledgeEntry> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        if query_words.is_empty() || query.trim().is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(f32, &KnowledgeEntry)> = self
            .entries
            .values()
            .filter_map(|entry| {
                let title_lower = entry.title.to_lowercase();
                let content_lower = entry.content.to_lowercase();

                let mut score = 0.0f32;

                // Title matches (weight: 3)
                for word in &query_words {
                    if title_lower.contains(word) {
                        score += 3.0;
                    }
                }

                // Tag matches (weight: 2)
                for tag in &entry.tags {
                    for word in &query_words {
                        if tag.to_lowercase().contains(word) {
                            score += 2.0;
                        }
                    }
                }

                // Content matches (weight: 1)
                for word in &query_words {
                    score += content_lower.matches(word).count() as f32;
                }

                // Scale by importance
                score *= entry.importance;

                if score > 0.0 {
                    Some((score, entry))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored.into_iter().take(limit).map(|(_, e)| e).collect()
    }

    /// Get all entries in a category.
    pub fn get_by_category(&self, category: &KnowledgeCategory) -> Vec<&KnowledgeEntry> {
        self.category_index
            .get(category)
            .map(|ids| ids.iter().filter_map(|id| self.entries.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all entries with a specific tag.
    pub fn get_by_tag(&self, tag: &str) -> Vec<&KnowledgeEntry> {
        self.tag_index
            .get(&tag.to_lowercase())
            .map(|ids| ids.iter().filter_map(|id| self.entries.get(id)).collect())
            .unwrap_or_default()
    }

    /// Load entries from a JSON file.
    pub async fn load_from_file(&mut self, path: &Path) -> Result<usize> {
        let content = tokio::fs::read_to_string(path).await?;
        let value: serde_json::Value = serde_json::from_str(&content)?;
        let entries: Vec<KnowledgeEntry> = match value {
            serde_json::Value::Array(items) => items
                .into_iter()
                .map(serde_json::from_value)
                .collect::<std::result::Result<Vec<_>, _>>()?,
            serde_json::Value::Object(_) => vec![serde_json::from_value(value)?],
            _ => Vec::new(),
        };
        let count = entries.len();
        for entry in entries {
            self.add_entry(entry);
        }
        info!("Loaded {} knowledge entries from {}", count, path.display());
        Ok(count)
    }

    /// Load all knowledge entries from a directory.
    pub async fn load_from_directory(&mut self, dir: &Path) -> Result<usize> {
        let mut total = 0;
        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                total += self.load_from_file(&path).await?;
            }
        }
        Ok(total)
    }

    /// Get all entries as a vector.
    pub fn all_entries(&self) -> Vec<&KnowledgeEntry> {
        self.entries.values().collect()
    }

    /// Get all unique tags across all entries.
    pub fn all_tags(&self) -> Vec<String> {
        self.tag_index.keys().cloned().collect()
    }

    /// Get all unique categories across all entries.
    pub fn all_categories(&self) -> Vec<KnowledgeCategory> {
        self.category_index.keys().cloned().collect()
    }

    /// Get the total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the knowledge base is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn loads_single_knowledge_object_from_file() {
        let file_path = std::env::temp_dir().join(format!(
            "monogatari-knowledge-{}-{}.json",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::write(
            &file_path,
            r#"{
              "id": "sakura_nature",
              "category": "world_lore",
              "title": "Sakura's Nature Diary",
              "content": "A pressed flower from the Springtown riverbank is tucked inside the diary.",
              "tags": ["sakura", "diary"]
            }"#,
        )
        .unwrap();

        let mut knowledge = KnowledgeBase::new();
        let loaded = knowledge.load_from_file(&file_path).await.unwrap();
        let _ = std::fs::remove_file(&file_path);

        assert_eq!(loaded, 1);
        let entry = knowledge.get_entry("sakura_nature").unwrap();
        assert_eq!(entry.category, KnowledgeCategory::Lore);
    }
}
