//! Dialogue flow management.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use tracing::{debug, info};

use llm_core::Result;

use super::dialogue_node::{Choice, DialogueNode};
use super::dialogue_script::DialogueScript;

/// Callback type for LLM-generated dialogue content.
pub type LLMInferenceCallback =
    Box<dyn Fn(String, Option<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send>> + Send + Sync>;

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
    ShowChoices {
        choices: Vec<Choice>,
    },
    /// Dialogue has ended.
    DialogueEnd,
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
        info!("Loaded dialogue script: {} ({})", script.title, script.id);
        self.scripts.insert(script.id.clone(), script);
        Ok(())
    }

    /// Load all dialogue scripts from a directory.
    pub async fn load_from_directory(&mut self, dir: &Path) -> Result<usize> {
        let mut total = 0;
        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                self.load_script(&path).await?;
                total += 1;
            }
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
        let current_id = self.current_node_id.clone().ok_or_else(|| {
            llm_core::EngineError::dialogue("none", "none", "No active dialogue")
        })?;

        let script_id = self.active_script_id.clone().ok_or_else(|| {
            llm_core::EngineError::dialogue("none", "none", "No active dialogue")
        })?;

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
        let current_id = self.current_node_id.clone().ok_or_else(|| {
            llm_core::EngineError::dialogue("none", "none", "No active dialogue")
        })?;

        let script_id = self.active_script_id.clone().ok_or_else(|| {
            llm_core::EngineError::dialogue("none", "none", "No active dialogue")
        })?;
        let script = self.scripts.get(&script_id).ok_or_else(|| {
            llm_core::EngineError::dialogue(&script_id, "unknown", "Script not found")
        })?;
        let node = script.nodes.get(&current_id).ok_or_else(|| {
            llm_core::EngineError::dialogue("current", &current_id, "Node not found")
        })?;

        let choice = node.choices.get(choice_index).ok_or_else(|| {
            llm_core::EngineError::dialogue("current", "current", format!("Invalid choice index: {choice_index}"))
        })?;

        // Apply relationship changes
        // (This would need access to the CharacterManager in a full implementation)

        // Check and set flags from the choice
        self.flags
            .insert(format!("choice_{script_id}_{choice_index}"), true);

        self.current_node_id = Some(choice.next_node_id.clone());
        self.process_current_node().await
    }

    /// Process the current node: execute scripts, handle LLM, send events.
    async fn process_current_node(&mut self) -> Result<()> {
        let current_id = self.current_node_id.clone().ok_or_else(|| {
            llm_core::EngineError::dialogue("none", "none", "No active dialogue")
        })?;

        let script_id = self.active_script_id.clone().ok_or_else(|| {
            llm_core::EngineError::dialogue("none", "none", "No active dialogue")
        })?;
        let script = self.scripts.get(&script_id).ok_or_else(|| {
            llm_core::EngineError::dialogue(&script_id, "unknown", "Script not found")
        })?;
        let node = script.nodes.get(&current_id).ok_or_else(|| {
            llm_core::EngineError::dialogue("current", &current_id, "Node not found")
        })?.clone();

        // Execute script if present
        if let Some(script_expr) = &node.script {
            self.execute_script(script_expr);
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

    /// Load dialogue state from persistence.
    pub fn load_state(
        &mut self,
        flags: HashMap<String, bool>,
        variables: HashMap<String, serde_json::Value>,
    ) {
        self.flags = flags;
        self.variables = variables;
    }

    /// Execute a simple script expression (setFlag, setVariable).
    fn execute_script(&mut self, script: &str) {
        let trimmed = script.trim();

        // Parse setFlag('name', true/false)
        if trimmed.starts_with("setFlag(") {
            let inner = &trimmed[8..trimmed.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim().trim_matches('\'')).collect();
            if parts.len() >= 2 {
                let flag_name = parts[0];
                let value = parts[1] == "true";
                self.flags.insert(flag_name.to_string(), value);
                debug!("Set flag: {} = {}", flag_name, value);
            }
        }

        // Parse setVariable('name', 'value')
        if trimmed.starts_with("setVariable(") {
            let inner = &trimmed[12..trimmed.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim().trim_matches('\'')).collect();
            if parts.len() >= 2 {
                let var_name = parts[0];
                let value = parts[1];
                self.variables.insert(
                    var_name.to_string(),
                    serde_json::Value::String(value.to_string()),
                );
                debug!("Set variable: {} = {}", var_name, value);
            }
        }
    }

    /// Set a game flag.
    pub fn set_flag(&mut self, name: &str, value: bool) {
        self.flags.insert(name.to_string(), value);
    }

    /// Check if a game flag is set.
    pub fn has_flag(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
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
}

impl Default for DialogueManager {
    fn default() -> Self {
        Self::new()
    }
}
