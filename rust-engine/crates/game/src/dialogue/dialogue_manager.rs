//! Dialogue flow management.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use llm_core::{normalize_script_state_key, normalize_script_state_map, Result};

use super::dialogue_node::{Choice, DialogueNode};
use super::dialogue_script::DialogueScript;

/// Callback type for LLM-generated dialogue content.
pub type LLMInferenceCallback = Box<
    dyn Fn(
            String,
            Option<String>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send>>
        + Send
        + Sync,
>;

/// Events fired by the dialogue manager.
#[derive(Debug, Clone)]
pub enum DialogueEvent {
    /// Show dialogue text from a character.
    ShowDialogue {
        speaker_id: Option<String>,
        text: String,
        emotion: Option<String>,
    },
    /// Show choices to the player.
    ShowChoices { choices: Vec<Choice> },
    /// Dialogue has ended.
    DialogueEnd,
}

/// Serializable dialogue cursor and local story state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DialogueRuntimeState {
    #[serde(default)]
    pub active_script_id: Option<String>,
    #[serde(default)]
    pub current_node_id: Option<String>,
    #[serde(default)]
    pub flags: HashMap<String, bool>,
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,
}

/// Stable authoring/runtime metadata for a loaded dialogue script.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DialogueScriptSummary {
    pub id: String,
    pub title: String,
    pub start_node_id: String,
    pub node_count: usize,
}

/// Manages dialogue flow, advancing through nodes and handling choices.
pub struct DialogueManager {
    /// All loaded dialogue scripts.
    scripts: HashMap<String, DialogueScript>,
    /// Currently active dialogue script ID.
    active_script_id: Option<String>,
    /// Current node ID within the active dialogue.
    current_node_id: Option<String>,
    /// Shared game flags for condition evaluation.
    flags: HashMap<String, bool>,
    /// Shared game variables.
    variables: HashMap<String, serde_json::Value>,
    /// Callback for LLM inference.
    llm_callback: Option<Arc<LLMInferenceCallback>>,
    /// Event sender for UI updates.
    event_sender: Option<tokio::sync::mpsc::UnboundedSender<DialogueEvent>>,
}

impl DialogueManager {
    /// Create a new dialogue manager.
    pub fn new() -> Self {
        Self {
            scripts: HashMap::new(),
            active_script_id: None,
            current_node_id: None,
            flags: HashMap::new(),
            variables: HashMap::new(),
            llm_callback: None,
            event_sender: None,
        }
    }

    /// Set the event sender for dialogue events.
    pub fn set_event_sender(&mut self, sender: tokio::sync::mpsc::UnboundedSender<DialogueEvent>) {
        self.event_sender = Some(sender);
    }

    /// Set the LLM inference callback.
    pub fn set_llm_callback(&mut self, callback: Arc<LLMInferenceCallback>) {
        self.llm_callback = Some(callback);
    }

    /// Load a dialogue script from a file.
    pub async fn load_script(&mut self, path: &Path) -> Result<()> {
        let script = DialogueScript::from_file(path).await?;
        if self.scripts.contains_key(&script.id) {
            return Err(llm_core::EngineError::dialogue(
                &script.id,
                "catalog",
                "Duplicate dialogue id",
            ));
        }
        info!("Loaded dialogue script: {} ({})", script.title, script.id);
        self.scripts.insert(script.id.clone(), script);
        Ok(())
    }

    /// Load all dialogue scripts from a directory.
    pub async fn load_from_directory(&mut self, dir: &Path) -> Result<usize> {
        let mut total = 0;
        let mut entries = tokio::fs::read_dir(dir).await?;
        let mut paths = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                paths.push(path);
            }
        }
        paths.sort();
        for path in paths {
            self.load_script(&path).await?;
            total += 1;
        }
        Ok(total)
    }

    /// Start a dialogue by script ID.
    pub async fn start_dialogue(&mut self, script_id: &str) -> Result<()> {
        let script = self.scripts.get(script_id).ok_or_else(|| {
            llm_core::EngineError::dialogue(script_id, "unknown", "Dialogue not found")
        })?;

        self.active_script_id = Some(script_id.to_string());
        self.current_node_id = Some(script.start_node_id.clone());

        debug!("Started dialogue: {}", script.title);
        self.process_current_node().await
    }

    /// Advance to the next node in a linear dialogue.
    pub async fn advance(&mut self) -> Result<()> {
        let current_id = self
            .current_node_id
            .clone()
            .ok_or_else(|| llm_core::EngineError::dialogue("none", "none", "No active dialogue"))?;

        let script_id = self
            .active_script_id
            .clone()
            .ok_or_else(|| llm_core::EngineError::dialogue("none", "none", "No active dialogue"))?;

        let script = self.scripts.get(&script_id).ok_or_else(|| {
            llm_core::EngineError::dialogue(&script_id, "unknown", "Script not found")
        })?;
        let node = script.nodes.get(&current_id).ok_or_else(|| {
            llm_core::EngineError::dialogue("current", &current_id, "Node not found")
        })?;

        if let Some(next_id) = &node.next_node_id {
            self.current_node_id = Some(next_id.clone());
            self.process_current_node().await
        } else {
            // End of dialogue
            self.end_dialogue().await
        }
    }

    /// Select a choice by index.
    pub async fn select_choice(&mut self, choice_index: usize) -> Result<()> {
        let current_id = self
            .current_node_id
            .clone()
            .ok_or_else(|| llm_core::EngineError::dialogue("none", "none", "No active dialogue"))?;

        let script_id = self
            .active_script_id
            .clone()
            .ok_or_else(|| llm_core::EngineError::dialogue("none", "none", "No active dialogue"))?;
        let script = self.scripts.get(&script_id).ok_or_else(|| {
            llm_core::EngineError::dialogue(&script_id, "unknown", "Script not found")
        })?;
        let node = script.nodes.get(&current_id).ok_or_else(|| {
            llm_core::EngineError::dialogue("current", &current_id, "Node not found")
        })?;

        let choice = node.choices.get(choice_index).ok_or_else(|| {
            llm_core::EngineError::dialogue(
                "current",
                "current",
                format!("Invalid choice index: {choice_index}"),
            )
        })?;

        // Apply relationship changes
        // (This would need access to the CharacterManager in a full implementation)

        // Check and set flags from the choice
        let choice_flag =
            normalize_script_state_key(&format!("choice_{script_id}_{choice_index}"))?;
        self.flags.insert(choice_flag, true);

        self.current_node_id = Some(choice.next_node_id.clone());
        self.process_current_node().await
    }

    /// Process the current node: execute scripts, handle LLM, send events.
    async fn process_current_node(&mut self) -> Result<()> {
        let current_id = self
            .current_node_id
            .clone()
            .ok_or_else(|| llm_core::EngineError::dialogue("none", "none", "No active dialogue"))?;

        let script_id = self
            .active_script_id
            .clone()
            .ok_or_else(|| llm_core::EngineError::dialogue("none", "none", "No active dialogue"))?;
        let script = self.scripts.get(&script_id).ok_or_else(|| {
            llm_core::EngineError::dialogue(&script_id, "unknown", "Script not found")
        })?;
        let node = script
            .nodes
            .get(&current_id)
            .ok_or_else(|| {
                llm_core::EngineError::dialogue("current", &current_id, "Node not found")
            })?
            .clone();

        // Execute script if present
        if let Some(script_expr) = &node.script {
            self.execute_script(script_expr)?;
        }

        // Handle LLM-generated content
        let mut text = node.text.clone();
        if node.use_llm {
            if let (Some(callback), Some(prompt)) = (&self.llm_callback, &node.llm_prompt) {
                match callback(prompt.clone(), node.llm_system_prompt.clone()).await {
                    Ok(llm_text) => text = llm_text,
                    Err(e) => {
                        debug!("LLM inference failed, using fallback text: {}", e);
                    }
                }
            }
        }

        // Send event
        if let Some(sender) = &self.event_sender {
            if node.choices.is_empty() {
                let _ = sender.send(DialogueEvent::ShowDialogue {
                    speaker_id: node.speaker_id.clone(),
                    text,
                    emotion: node.emotion.clone(),
                });
            } else {
                let _ = sender.send(DialogueEvent::ShowDialogue {
                    speaker_id: node.speaker_id.clone(),
                    text,
                    emotion: node.emotion.clone(),
                });
                let _ = sender.send(DialogueEvent::ShowChoices {
                    choices: node.choices.clone(),
                });
            }
        }

        Ok(())
    }

    /// End the current dialogue.
    async fn end_dialogue(&mut self) -> Result<()> {
        self.active_script_id = None;
        self.current_node_id = None;

        if let Some(sender) = &self.event_sender {
            let _ = sender.send(DialogueEvent::DialogueEnd);
        }

        debug!("Dialogue ended");
        Ok(())
    }

    /// Get the current dialogue context for character memory.
    /// Returns (speaker_id, text, emotion) of the current node.
    pub fn get_current_context(&self) -> Option<(Option<String>, String, Option<String>)> {
        let script_id = self.active_script_id.as_ref()?;
        let node_id = self.current_node_id.as_ref()?;
        let script = self.scripts.get(script_id)?;
        let node = script.nodes.get(node_id)?;

        Some((
            node.speaker_id.clone(),
            node.text.clone(),
            node.emotion.clone(),
        ))
    }

    /// Get the dialogue history (flags and variables) for persistence.
    pub fn get_state(&self) -> (&HashMap<String, bool>, &HashMap<String, serde_json::Value>) {
        (&self.flags, &self.variables)
    }

    /// Snapshot the active dialogue cursor and its local state for persistence.
    pub fn runtime_state(&self) -> DialogueRuntimeState {
        DialogueRuntimeState {
            active_script_id: self.active_script_id.clone(),
            current_node_id: self.current_node_id.clone(),
            flags: self.flags.clone(),
            variables: self.variables.clone(),
        }
    }

    /// Validate and normalize a persisted dialogue snapshot without mutating runtime state.
    pub fn validate_runtime_state(
        &self,
        mut state: DialogueRuntimeState,
    ) -> Result<DialogueRuntimeState> {
        state.flags = normalize_script_state_map(state.flags)?;
        state.variables = normalize_script_state_map(state.variables)?;

        match (&state.active_script_id, &state.current_node_id) {
            (None, None) => {}
            (Some(script_id), Some(node_id)) => {
                let script = self.scripts.get(script_id).ok_or_else(|| {
                    llm_core::EngineError::dialogue(
                        script_id,
                        node_id,
                        "Saved dialogue script is not loaded",
                    )
                })?;
                if !script.nodes.contains_key(node_id) {
                    return Err(llm_core::EngineError::dialogue(
                        script_id,
                        node_id,
                        "Saved dialogue node does not exist",
                    ));
                }
            }
            _ => {
                return Err(llm_core::EngineError::dialogue(
                    state.active_script_id.as_deref().unwrap_or("none"),
                    state.current_node_id.as_deref().unwrap_or("none"),
                    "Saved dialogue cursor must include both script and node ids",
                ));
            }
        }

        Ok(state)
    }

    /// Restore a previously validated dialogue cursor and local state.
    pub fn restore_runtime_state(&mut self, state: DialogueRuntimeState) -> Result<()> {
        let state = self.validate_runtime_state(state)?;
        self.active_script_id = state.active_script_id;
        self.current_node_id = state.current_node_id;
        self.flags = state.flags;
        self.variables = state.variables;
        Ok(())
    }

    /// Load dialogue state from persistence.
    pub fn load_state(
        &mut self,
        flags: HashMap<String, bool>,
        variables: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        self.flags = normalize_script_state_map(flags)?;
        self.variables = normalize_script_state_map(variables)?;
        Ok(())
    }

    /// Execute a simple script expression (setFlag, setVariable).
    fn execute_script(&mut self, script: &str) -> Result<()> {
        let trimmed = script.trim();

        // Parse setFlag('name', true/false)
        if let Some(inner) = trimmed
            .strip_prefix("setFlag(")
            .and_then(|value| value.strip_suffix(')'))
        {
            let parts: Vec<&str> = inner
                .split(',')
                .map(|s| s.trim().trim_matches('\''))
                .collect();
            if parts.len() >= 2 {
                let flag_name = normalize_script_state_key(parts[0])?;
                let value = parts[1] == "true";
                self.flags.insert(flag_name.clone(), value);
                debug!("Set flag: {} = {}", flag_name, value);
            }
        }

        // Parse setVariable('name', 'value')
        if let Some(inner) = trimmed
            .strip_prefix("setVariable(")
            .and_then(|value| value.strip_suffix(')'))
        {
            let parts: Vec<&str> = inner
                .split(',')
                .map(|s| s.trim().trim_matches('\''))
                .collect();
            if parts.len() >= 2 {
                let var_name = normalize_script_state_key(parts[0])?;
                let value = parts[1];
                self.variables.insert(
                    var_name.clone(),
                    serde_json::Value::String(value.to_string()),
                );
                debug!("Set variable: {} = {}", var_name, value);
            }
        }

        Ok(())
    }

    /// Set a game flag.
    pub fn set_flag(&mut self, name: &str, value: bool) -> Result<()> {
        let name = normalize_script_state_key(name)?;
        self.flags.insert(name, value);
        Ok(())
    }

    /// Check if a game flag is set.
    pub fn has_flag(&self, name: &str) -> bool {
        let Ok(name) = normalize_script_state_key(name) else {
            return false;
        };
        self.flags.get(&name).copied().unwrap_or(false)
    }

    /// Get the current node (for UI rendering).
    pub fn current_node(&self) -> Option<&DialogueNode> {
        let script_id = self.active_script_id.as_ref()?;
        let node_id = self.current_node_id.as_ref()?;
        self.scripts.get(script_id)?.nodes.get(node_id)
    }

    /// Check if a dialogue is currently active.
    pub fn is_active(&self) -> bool {
        self.active_script_id.is_some()
    }

    /// Get available script IDs.
    pub fn script_ids(&self) -> Vec<String> {
        self.scripts.keys().cloned().collect()
    }

    /// Get deterministic dialogue metadata for content browsers and authoring tools.
    pub fn script_summaries(&self) -> Vec<DialogueScriptSummary> {
        let mut summaries = self
            .scripts
            .values()
            .map(|script| DialogueScriptSummary {
                id: script.id.clone(),
                title: script.title.clone(),
                start_node_id: script.start_node_id.clone(),
                node_count: script.nodes.len(),
            })
            .collect::<Vec<_>>();
        summaries.sort_by(|left, right| left.id.cmp(&right.id));
        summaries
    }

    /// Replace the loaded script catalog while retaining runtime callbacks and local state.
    pub fn replace_scripts(&mut self, scripts: Vec<DialogueScript>) -> Result<usize> {
        let mut replacement = HashMap::with_capacity(scripts.len());
        for mut script in scripts {
            script.validate_graph()?;
            for (node_id, node) in &mut script.nodes {
                node.id.clone_from(node_id);
            }
            let script_id = script.id.clone();
            if replacement.insert(script_id.clone(), script).is_some() {
                return Err(llm_core::EngineError::dialogue(
                    &script_id,
                    "catalog",
                    "Duplicate dialogue id",
                ));
            }
        }
        let count = replacement.len();
        self.scripts = replacement;
        self.active_script_id = None;
        self.current_node_id = None;
        Ok(count)
    }

    /// Clone scripts in deterministic id order for authoring and catalog verification.
    pub fn scripts(&self) -> Vec<DialogueScript> {
        let mut scripts = self.scripts.values().cloned().collect::<Vec<_>>();
        scripts.sort_by(|left, right| left.id.cmp(&right.id));
        scripts
    }

    pub fn has_script(&self, script_id: &str) -> bool {
        self.scripts.contains_key(script_id)
    }
}

impl Default for DialogueManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue_state_keys_trim_and_allow_portable_names() {
        let mut manager = DialogueManager::new();

        manager.set_flag(" chapter_1.passed ", true).unwrap();
        manager
            .execute_script("setVariable('chapter_1.score', 'high')")
            .unwrap();

        assert!(manager.has_flag("chapter_1.passed"));
        assert_eq!(
            manager.get_state().1.get("chapter_1.score"),
            Some(&serde_json::Value::String("high".to_string()))
        );
    }

    #[test]
    fn script_summaries_are_sorted_and_do_not_expose_node_content() {
        let mut manager = DialogueManager::new();
        for (id, title) in [("z_last", "Last"), ("a_first", "First")] {
            let script: DialogueScript = serde_json::from_value(serde_json::json!({
                "id": id,
                "title": title,
                "start_node_id": "start",
                "nodes": {
                    "start": {"text": "secret", "choices": []}
                }
            }))
            .unwrap();
            manager.scripts.insert(id.to_string(), script);
        }

        let summaries = manager.script_summaries();
        assert_eq!(summaries[0].id, "a_first");
        assert_eq!(summaries[1].id, "z_last");
        assert_eq!(summaries[0].node_count, 1);
        assert!(manager.has_script("a_first"));
        assert!(!manager.has_script("missing"));
    }

    #[test]
    fn dialogue_state_keys_reject_invalid_names() {
        let mut manager = DialogueManager::new();

        assert!(manager.set_flag("bad/key", true).is_err());
        assert!(manager
            .execute_script("setVariable('bad key', 'value')")
            .is_err());
        assert!(manager.execute_script("setFlag('bad:key', true)").is_err());
    }

    #[test]
    fn dialogue_load_state_rejects_invalid_keys() {
        let mut manager = DialogueManager::new();
        let flags = HashMap::from([("bad/key".to_string(), true)]);
        let variables = HashMap::new();

        assert!(manager.load_state(flags, variables).is_err());
    }

    #[tokio::test]
    async fn dialogue_runtime_snapshot_restores_cursor_and_local_state() {
        let script: DialogueScript = serde_json::from_value(serde_json::json!({
            "id": "intro",
            "title": "Intro",
            "start_node_id": "start",
            "nodes": {
                "start": {
                    "id": "start",
                    "text": "Hello",
                    "next_node_id": "second"
                },
                "second": {
                    "id": "second",
                    "text": "Again"
                }
            }
        }))
        .unwrap();
        let mut manager = DialogueManager::new();
        manager.scripts.insert(script.id.clone(), script);
        manager.start_dialogue("intro").await.unwrap();
        manager.set_flag("started", true).unwrap();
        let snapshot = manager.runtime_state();

        manager.advance().await.unwrap();
        assert_eq!(manager.current_node().unwrap().id, "second");
        manager.restore_runtime_state(snapshot).unwrap();

        assert_eq!(manager.current_node().unwrap().id, "start");
        assert!(manager.has_flag("started"));
    }

    #[test]
    fn dialogue_runtime_snapshot_rejects_broken_cursors() {
        let manager = DialogueManager::new();
        assert!(manager
            .validate_runtime_state(DialogueRuntimeState {
                active_script_id: Some("missing".to_string()),
                current_node_id: Some("start".to_string()),
                ..Default::default()
            })
            .is_err());
        assert!(manager
            .validate_runtime_state(DialogueRuntimeState {
                active_script_id: Some("intro".to_string()),
                current_node_id: None,
                ..Default::default()
            })
            .is_err());
    }
}
