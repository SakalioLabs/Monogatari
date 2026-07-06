//! Dialogue script container.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::dialogue_node::DialogueNode;

/// A complete dialogue script containing a tree of nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueScript {
    /// Unique dialogue identifier.
    pub id: String,
    /// Display title for this dialogue.
    pub title: String,
    /// ID of the starting node.
    pub start_node_id: String,
    /// All nodes in this dialogue, keyed by node ID.
    pub nodes: HashMap<String, DialogueNode>,
    /// Script variables that can be referenced in conditions.
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,
}

impl DialogueScript {
    /// Load a dialogue script from a JSON file.
    pub async fn from_file(path: &std::path::Path) -> llm_core::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let script: Self = serde_json::from_str(&content)?;
        Ok(script)
    }

    /// Get the starting node.
    pub fn start_node(&self) -> Option<&DialogueNode> {
        self.nodes.get(&self.start_node_id)
    }

    /// Get a node by ID.
    pub fn get_node(&self, node_id: &str) -> Option<&DialogueNode> {
        self.nodes.get(node_id)
    }
}
