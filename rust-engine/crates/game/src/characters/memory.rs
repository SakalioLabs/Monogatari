//! Character memory system with importance-based eviction.

use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of memories a character can have.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    Conversation,
    Event,
    Emotion,
    Knowledge,
    Relationship,
}

/// A single memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// The memory content.
    pub content: String,
    /// Type of memory.
    pub memory_type: MemoryType,
    /// Importance score (0.0 to 1.0). Higher = kept longer.
    pub importance: f32,
    /// When this memory was created.
    pub timestamp: DateTime<Utc>,
    /// Associated tags for search.
    pub tags: Vec<String>,
}

/// In-memory store with importance-based eviction.
///
/// Maintains a bounded list of memories, evicting the least important
/// ones when the capacity is reached.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMemory {
    /// The memories stored in chronological order.
    memories: VecDeque<MemoryEntry>,
    /// Maximum number of memories to keep.
    max_capacity: usize,
}

impl CharacterMemory {
    /// Create a new memory store with the given capacity.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            memories: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    /// Add a new memory. If at capacity, evicts the least important memory.
    pub fn add_memory(
        &mut self,
        content: String,
        memory_type: MemoryType,
        importance: f32,
        tags: Vec<String>,
    ) {
        let entry = MemoryEntry {
            content,
            memory_type,
            importance: importance.clamp(0.0, 1.0),
            timestamp: Utc::now(),
            tags,
        };

        if self.memories.len() >= self.max_capacity {
            // Find and remove the least important memory
            if let Some(min_idx) = self
                .memories
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.importance.partial_cmp(&b.1.importance).unwrap())
                .map(|(i, _)| i)
            {
                self.memories.remove(min_idx);
            }
        }

        self.memories.push_back(entry);
    }

    /// Recall memories matching a keyword search.
    ///
    /// Searches content and tags. Returns memories sorted by relevance
    /// (importance * keyword match count).
    pub fn recall(&self, keyword: &str, limit: usize) -> Vec<&MemoryEntry> {
        let keyword_lower = keyword.to_lowercase();
        let mut scored: Vec<(f32, &MemoryEntry)> = self
            .memories
            .iter()
            .filter_map(|entry| {
                let content_lower = entry.content.to_lowercase();
                let matches = content_lower.matches(&keyword_lower).count()
                    + entry
                        .tags
                        .iter()
                        .filter(|t| t.to_lowercase().contains(&keyword_lower))
                        .count();
                if matches > 0 {
                    Some((entry.importance * matches as f32, entry))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored.into_iter().take(limit).map(|(_, e)| e).collect()
    }

    /// Get the N most recent memories.
    pub fn get_recent(&self, count: usize) -> Vec<&MemoryEntry> {
        self.memories.iter().rev().take(count).collect()
    }

    /// Forget memories below the given importance threshold.
    pub fn forget_old(&mut self, importance_threshold: f32) {
        self.memories
            .retain(|m| m.importance >= importance_threshold);
    }

    /// Get the current number of stored memories.
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    /// Check if the memory store is empty.
    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }

    /// Get all memories as a slice.
    pub fn all(&self) -> &VecDeque<MemoryEntry> {
        &self.memories
    }
}

impl Default for CharacterMemory {
    fn default() -> Self {
        Self::new(100)
    }
}
