//! Dialogue script container.

use std::collections::{HashMap, HashSet, VecDeque};

use llm_scripting::{validate_condition_source, validate_script_source};
use serde::{Deserialize, Serialize};

use super::dialogue_node::DialogueNode;

/// A complete dialogue script containing a tree of nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueScript {
    /// Unique dialogue identifier.
    pub id: String,
    /// Display title for this dialogue.
    pub title: String,
    /// Optional author-facing synopsis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
        let mut script: Self = serde_json::from_str(&content)?;
        script.validate_graph()?;
        for (node_id, node) in &mut script.nodes {
            node.id.clone_from(node_id);
        }
        Ok(script)
    }

    /// Validate node identity, transitions, terminal semantics, and reachability.
    pub fn validate_graph(&self) -> llm_core::Result<()> {
        if !portable_id(&self.id) {
            return Err(llm_core::EngineError::dialogue(
                &self.id,
                "catalog",
                "Dialogue id must be a portable id",
            ));
        }
        if !portable_id(&self.start_node_id) || !self.nodes.contains_key(&self.start_node_id) {
            return Err(llm_core::EngineError::dialogue(
                &self.id,
                &self.start_node_id,
                "Start node does not exist",
            ));
        }

        for (node_id, node) in &self.nodes {
            if !portable_id(node_id) {
                return Err(llm_core::EngineError::dialogue(
                    &self.id,
                    node_id,
                    "Node id must be a portable id",
                ));
            }
            if !node.id.is_empty() && node.id != *node_id {
                return Err(llm_core::EngineError::dialogue(
                    &self.id,
                    node_id,
                    format!("Embedded node id `{}` does not match its map key", node.id),
                ));
            }
            if node.next_node_id.is_some() && !node.choices.is_empty() {
                return Err(llm_core::EngineError::dialogue(
                    &self.id,
                    node_id,
                    "A node cannot combine a linear next node with player choices",
                ));
            }
            if node
                .condition
                .as_deref()
                .is_some_and(|condition| !condition.trim().is_empty())
                && node.next_node_id.is_none()
            {
                return Err(llm_core::EngineError::dialogue(
                    &self.id,
                    node_id,
                    "A conditional node requires a linear fallback transition",
                ));
            }
            if let Some(condition) = node.condition.as_deref() {
                validate_condition_source(condition)?;
            }
            if let Some(script) = node.script.as_deref() {
                validate_script_source(script)?;
            }
            if node.is_ending && (node.next_node_id.is_some() || !node.choices.is_empty()) {
                return Err(llm_core::EngineError::dialogue(
                    &self.id,
                    node_id,
                    "An ending node cannot have outgoing transitions",
                ));
            }
            if node.ending_type.is_some() && !node.is_ending {
                return Err(llm_core::EngineError::dialogue(
                    &self.id,
                    node_id,
                    "ending_type requires is_ending=true",
                ));
            }
            if let Some(next_node_id) = &node.next_node_id {
                ensure_target(self, node_id, next_node_id)?;
            }
            for choice in &node.choices {
                if let Some(condition) = choice.condition.as_deref() {
                    validate_condition_source(condition)?;
                }
                ensure_target(self, node_id, &choice.next_node_id)?;
            }
        }

        let mut reachable = HashSet::new();
        let mut queue = VecDeque::from([self.start_node_id.as_str()]);
        while let Some(node_id) = queue.pop_front() {
            if !reachable.insert(node_id) {
                continue;
            }
            let node = &self.nodes[node_id];
            if let Some(next_node_id) = node.next_node_id.as_deref() {
                queue.push_back(next_node_id);
            }
            for choice in &node.choices {
                queue.push_back(&choice.next_node_id);
            }
        }
        if reachable.len() != self.nodes.len() {
            let mut unreachable = self
                .nodes
                .keys()
                .filter(|node_id| !reachable.contains(node_id.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            unreachable.sort();
            return Err(llm_core::EngineError::dialogue(
                &self.id,
                "catalog",
                format!("Unreachable dialogue nodes: {}", unreachable.join(", ")),
            ));
        }
        Ok(())
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

fn ensure_target(script: &DialogueScript, node_id: &str, target: &str) -> llm_core::Result<()> {
    if !script.nodes.contains_key(target) {
        return Err(llm_core::EngineError::dialogue(
            &script.id,
            node_id,
            format!("Transition target `{target}` does not exist"),
        ));
    }
    Ok(())
}

fn portable_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.trim() == value
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}
