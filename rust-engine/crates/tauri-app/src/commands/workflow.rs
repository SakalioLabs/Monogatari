//! Workflow editor commands (Dify-style no-code workflow).

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// A workflow node in the visual editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub config: serde_json::Value,
    pub connections: Vec<String>, // IDs of connected nodes
}

/// A complete workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub nodes: Vec<WorkflowNode>,
    pub start_node_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowNodeTypeInfo {
    pub node_type: String,
    pub label: String,
    pub description: String,
    pub category: String,
    pub configurable_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowValidationIssue {
    pub severity: String,
    pub code: String,
    pub node_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowValidationResult {
    pub valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub issues: Vec<WorkflowValidationIssue>,
}

/// Get available workflow node types.
#[tauri::command]
pub async fn get_workflow_nodes() -> Result<Vec<WorkflowNodeTypeInfo>, String> {
    Ok(vec![
        WorkflowNodeTypeInfo {
            node_type: "start".to_string(),
            label: "Start".to_string(),
            description: "Starting point of the workflow".to_string(),
            category: "flow".to_string(),
            configurable_fields: vec![],
        },
        WorkflowNodeTypeInfo {
            node_type: "dialogue".to_string(),
            label: "Dialogue".to_string(),
            description: "Show dialogue text from a character".to_string(),
            category: "content".to_string(),
            configurable_fields: vec![
                "speaker".to_string(),
                "text".to_string(),
                "emotion".to_string(),
                "use_llm".to_string(),
            ],
        },
        WorkflowNodeTypeInfo {
            node_type: "choice".to_string(),
            label: "Choice".to_string(),
            description: "Present choices to the player".to_string(),
            category: "content".to_string(),
            configurable_fields: vec!["choices".to_string()],
        },
        WorkflowNodeTypeInfo {
            node_type: "condition".to_string(),
            label: "Condition".to_string(),
            description: "Branch based on a condition".to_string(),
            category: "flow".to_string(),
            configurable_fields: vec!["condition".to_string()],
        },
        WorkflowNodeTypeInfo {
            node_type: "set_variable".to_string(),
            label: "Set Variable".to_string(),
            description: "Set a game variable".to_string(),
            category: "logic".to_string(),
            configurable_fields: vec!["variable_name".to_string(), "value".to_string()],
        },
        WorkflowNodeTypeInfo {
            node_type: "set_flag".to_string(),
            label: "Set Flag".to_string(),
            description: "Set a game flag".to_string(),
            category: "logic".to_string(),
            configurable_fields: vec!["flag_name".to_string(), "value".to_string()],
        },
        WorkflowNodeTypeInfo {
            node_type: "llm_generate".to_string(),
            label: "LLM Generate".to_string(),
            description: "Generate text using LLM".to_string(),
            category: "ai".to_string(),
            configurable_fields: vec![
                "prompt".to_string(),
                "system_prompt".to_string(),
                "max_tokens".to_string(),
            ],
        },
        WorkflowNodeTypeInfo {
            node_type: "evaluation".to_string(),
            label: "Evaluation".to_string(),
            description: "Evaluate conversation quality".to_string(),
            category: "ai".to_string(),
            configurable_fields: vec!["criteria".to_string(), "threshold".to_string()],
        },
        WorkflowNodeTypeInfo {
            node_type: "scene_change".to_string(),
            label: "Scene Change".to_string(),
            description: "Change the active background scene".to_string(),
            category: "content".to_string(),
            configurable_fields: vec!["scene_id".to_string()],
        },
        WorkflowNodeTypeInfo {
            node_type: "trigger_event".to_string(),
            label: "Trigger Event".to_string(),
            description: "Trigger a named story event".to_string(),
            category: "flow".to_string(),
            configurable_fields: vec!["event_id".to_string(), "event_type".to_string()],
        },
        WorkflowNodeTypeInfo {
            node_type: "emotion_change".to_string(),
            label: "Change Emotion".to_string(),
            description: "Change a character's emotion".to_string(),
            category: "character".to_string(),
            configurable_fields: vec!["character_id".to_string(), "emotion".to_string()],
        },
        WorkflowNodeTypeInfo {
            node_type: "relationship".to_string(),
            label: "Relationship".to_string(),
            description: "Modify relationship score".to_string(),
            category: "character".to_string(),
            configurable_fields: vec!["character_id".to_string(), "delta".to_string()],
        },
        WorkflowNodeTypeInfo {
            node_type: "end".to_string(),
            label: "End".to_string(),
            description: "End of the workflow".to_string(),
            category: "flow".to_string(),
            configurable_fields: vec![],
        },
    ])
}

/// Validate a workflow before save/import/export.
#[tauri::command]
pub async fn validate_workflow(workflow: Workflow) -> Result<WorkflowValidationResult, String> {
    Ok(validate_workflow_inner(&workflow))
}

/// Execute a single workflow node.
#[tauri::command]
pub async fn execute_workflow_node(
    state: State<'_, AppState>,
    node: WorkflowNode,
) -> Result<serde_json::Value, String> {
    match node.node_type.as_str() {
        "set_variable" => {
            let name = node.config["variable_name"].as_str().unwrap_or("");
            let value = node.config["value"].as_str().unwrap_or("");
            let se = state.script_engine.read().await;
            se.set_variable(name, rhai::Dynamic::from(value.to_string()));
            Ok(serde_json::json!({"status": "ok"}))
        }
        "set_flag" => {
            let name = node.config["flag_name"].as_str().unwrap_or("");
            let value = node.config["value"].as_bool().unwrap_or(true);
            let se = state.script_engine.read().await;
            se.set_flag(name, value);
            Ok(serde_json::json!({"status": "ok"}))
        }
        "condition" => {
            let condition = node.config["condition"].as_str().unwrap_or("true");
            let se = state.script_engine.read().await;
            let result = se
                .evaluate_condition(condition)
                .map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"result": result}))
        }
        "llm_generate" => {
            let prompt = node.config["prompt"].as_str().unwrap_or("");
            let pipeline = state.inference_pipeline.read().await;
            let options = llm_ai::InferenceOptions::default();
            let result = pipeline
                .generate_response(prompt, &options)
                .await
                .map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"text": result.text}))
        }
        _ => Err(format!("Unknown node type: {}", node.node_type)),
    }
}

/// Save a workflow to a file.
#[tauri::command]
pub async fn save_workflow(
    _state: State<'_, AppState>,
    workflow: Workflow,
    path: String,
) -> Result<String, String> {
    let validation = validate_workflow_inner(&workflow);
    if !validation.valid {
        return Err(format_validation_errors(&validation));
    }

    let json = serde_json::to_string_pretty(&workflow).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, json)
        .await
        .map_err(|e| e.to_string())?;
    Ok("Workflow saved".to_string())
}

/// Load a workflow from a file.
#[tauri::command]
pub async fn load_workflow(_state: State<'_, AppState>, path: String) -> Result<Workflow, String> {
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| e.to_string())?;
    let workflow: Workflow = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let validation = validate_workflow_inner(&workflow);
    if !validation.valid {
        return Err(format_validation_errors(&validation));
    }
    Ok(workflow)
}

fn validate_workflow_inner(workflow: &Workflow) -> WorkflowValidationResult {
    let mut issues = Vec::new();
    let mut node_ids = HashSet::new();
    let mut node_lookup: HashMap<&str, &WorkflowNode> = HashMap::new();
    let known_types = known_node_types();

    if workflow.id.trim().is_empty() {
        push_issue(
            &mut issues,
            "error",
            "workflow_id_empty",
            None,
            "Workflow id is required.",
        );
    }

    if workflow.name.trim().is_empty() {
        push_issue(
            &mut issues,
            "error",
            "workflow_name_empty",
            None,
            "Workflow name is required.",
        );
    }

    if workflow.nodes.is_empty() {
        push_issue(
            &mut issues,
            "error",
            "workflow_empty",
            None,
            "Workflow must contain at least one node.",
        );
    }

    for node in &workflow.nodes {
        if node.id.trim().is_empty() {
            push_issue(
                &mut issues,
                "error",
                "node_id_empty",
                None,
                "Every node must have a non-empty id.",
            );
            continue;
        }

        if !node_ids.insert(node.id.clone()) {
            push_issue(
                &mut issues,
                "error",
                "node_id_duplicate",
                Some(node.id.clone()),
                "Node ids must be unique.",
            );
        }

        node_lookup.insert(node.id.as_str(), node);
    }

    let start_nodes = workflow
        .nodes
        .iter()
        .filter(|node| node.node_type == "start")
        .count();
    if start_nodes == 0 && !workflow.nodes.is_empty() {
        push_issue(
            &mut issues,
            "error",
            "start_node_missing",
            None,
            "Workflow must include a start node.",
        );
    } else if start_nodes > 1 {
        push_issue(
            &mut issues,
            "warning",
            "start_node_multiple",
            None,
            "Multiple start nodes found; only the configured start node is used.",
        );
    }

    if workflow.start_node_id.trim().is_empty() && !workflow.nodes.is_empty() {
        push_issue(
            &mut issues,
            "error",
            "start_node_id_empty",
            None,
            "Workflow start_node_id is required.",
        );
    } else if let Some(start_node) = node_lookup.get(workflow.start_node_id.as_str()) {
        if start_node.node_type != "start" {
            push_issue(
                &mut issues,
                "error",
                "start_node_type_invalid",
                Some(start_node.id.clone()),
                "start_node_id must reference a start node.",
            );
        }
    } else if !workflow.nodes.is_empty() {
        push_issue(
            &mut issues,
            "error",
            "start_node_not_found",
            Some(workflow.start_node_id.clone()),
            "start_node_id does not match any node.",
        );
    }

    if workflow
        .nodes
        .iter()
        .all(|node| node.node_type.as_str() != "end")
        && !workflow.nodes.is_empty()
    {
        push_issue(
            &mut issues,
            "warning",
            "end_node_missing",
            None,
            "Workflow has no end node.",
        );
    }

    for node in &workflow.nodes {
        if node.label.trim().is_empty() {
            push_issue(
                &mut issues,
                "warning",
                "node_label_empty",
                Some(node.id.clone()),
                "Node label is empty.",
            );
        }

        if !known_types.contains(node.node_type.as_str()) {
            push_issue(
                &mut issues,
                "error",
                "node_type_unknown",
                Some(node.id.clone()),
                format!("Unknown node type: {}", node.node_type),
            );
            continue;
        }

        for field in required_fields(node.node_type.as_str()) {
            if !config_field_present(&node.config, field) {
                push_issue(
                    &mut issues,
                    "error",
                    "node_config_missing",
                    Some(node.id.clone()),
                    format!("Required field `{field}` is missing."),
                );
            }
        }

        let mut local_targets = HashSet::new();
        for target_id in &node.connections {
            if target_id.trim().is_empty() {
                push_issue(
                    &mut issues,
                    "error",
                    "connection_empty",
                    Some(node.id.clone()),
                    "Connection target id is empty.",
                );
                continue;
            }

            if target_id == &node.id {
                push_issue(
                    &mut issues,
                    "error",
                    "connection_self",
                    Some(node.id.clone()),
                    "Node cannot connect to itself.",
                );
            }

            if !node_ids.contains(target_id) {
                push_issue(
                    &mut issues,
                    "error",
                    "connection_target_missing",
                    Some(node.id.clone()),
                    format!("Connection target `{target_id}` does not exist."),
                );
            }

            if !local_targets.insert(target_id) {
                push_issue(
                    &mut issues,
                    "warning",
                    "connection_duplicate",
                    Some(node.id.clone()),
                    format!("Duplicate connection to `{target_id}`."),
                );
            }
        }
    }

    warn_unreachable_nodes(workflow, &node_lookup, &mut issues);

    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .count();
    let warning_count = issues
        .iter()
        .filter(|issue| issue.severity == "warning")
        .count();

    WorkflowValidationResult {
        valid: error_count == 0,
        error_count,
        warning_count,
        issues,
    }
}

fn warn_unreachable_nodes(
    workflow: &Workflow,
    node_lookup: &HashMap<&str, &WorkflowNode>,
    issues: &mut Vec<WorkflowValidationIssue>,
) {
    if workflow.start_node_id.trim().is_empty()
        || !node_lookup.contains_key(workflow.start_node_id.as_str())
    {
        return;
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(workflow.start_node_id.as_str());

    while let Some(node_id) = queue.pop_front() {
        if !visited.insert(node_id) {
            continue;
        }

        if let Some(node) = node_lookup.get(node_id) {
            for target_id in &node.connections {
                if node_lookup.contains_key(target_id.as_str()) {
                    queue.push_back(target_id.as_str());
                }
            }
        }
    }

    for node in &workflow.nodes {
        if !visited.contains(node.id.as_str()) {
            push_issue(
                issues,
                "warning",
                "node_unreachable",
                Some(node.id.clone()),
                "Node is not reachable from the configured start node.",
            );
        }
    }
}

fn known_node_types() -> HashSet<&'static str> {
    HashSet::from([
        "start",
        "dialogue",
        "choice",
        "condition",
        "set_variable",
        "set_flag",
        "llm_generate",
        "evaluation",
        "scene_change",
        "trigger_event",
        "emotion_change",
        "relationship",
        "end",
    ])
}

fn required_fields(node_type: &str) -> &'static [&'static str] {
    match node_type {
        "dialogue" => &["text"],
        "choice" => &["choices"],
        "condition" => &["condition"],
        "set_variable" => &["variable_name", "value"],
        "set_flag" => &["flag_name", "value"],
        "llm_generate" => &["prompt"],
        "evaluation" => &["criteria"],
        "scene_change" => &["scene_id"],
        "trigger_event" => &["event_id"],
        "emotion_change" => &["character_id", "emotion"],
        "relationship" => &["character_id", "delta"],
        _ => &[],
    }
}

fn config_field_present(config: &serde_json::Value, field: &str) -> bool {
    let Some(value) = config.get(field) else {
        return false;
    };

    match value {
        serde_json::Value::Null => false,
        serde_json::Value::String(value) => !value.trim().is_empty(),
        serde_json::Value::Array(value) => !value.is_empty(),
        serde_json::Value::Object(value) => !value.is_empty(),
        serde_json::Value::Bool(_) | serde_json::Value::Number(_) => true,
    }
}

fn push_issue(
    issues: &mut Vec<WorkflowValidationIssue>,
    severity: impl Into<String>,
    code: impl Into<String>,
    node_id: Option<String>,
    message: impl Into<String>,
) {
    issues.push(WorkflowValidationIssue {
        severity: severity.into(),
        code: code.into(),
        node_id,
        message: message.into(),
    });
}

fn format_validation_errors(validation: &WorkflowValidationResult) -> String {
    let messages: Vec<String> = validation
        .issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .map(|issue| {
            if let Some(node_id) = &issue.node_id {
                format!("{} ({node_id}): {}", issue.code, issue.message)
            } else {
                format!("{}: {}", issue.code, issue.message)
            }
        })
        .collect();

    format!("Workflow validation failed: {}", messages.join("; "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(
        id: &str,
        node_type: &str,
        connections: Vec<&str>,
        config: serde_json::Value,
    ) -> WorkflowNode {
        WorkflowNode {
            id: id.to_string(),
            node_type: node_type.to_string(),
            label: node_type.to_string(),
            x: 0.0,
            y: 0.0,
            config,
            connections: connections.into_iter().map(str::to_string).collect(),
        }
    }

    #[test]
    fn validates_minimal_start_to_end_workflow() {
        let workflow = Workflow {
            id: "wf_valid".to_string(),
            name: "Valid".to_string(),
            nodes: vec![
                node("start", "start", vec!["end"], serde_json::json!({})),
                node("end", "end", vec![], serde_json::json!({})),
            ],
            start_node_id: "start".to_string(),
        };

        let validation = validate_workflow_inner(&workflow);
        assert!(validation.valid, "{:?}", validation.issues);
        assert_eq!(validation.error_count, 0);
    }

    #[test]
    fn catches_missing_required_config_and_broken_links() {
        let workflow = Workflow {
            id: "wf_invalid".to_string(),
            name: "Invalid".to_string(),
            nodes: vec![
                node("start", "start", vec!["dialogue"], serde_json::json!({})),
                node(
                    "dialogue",
                    "dialogue",
                    vec!["missing"],
                    serde_json::json!({}),
                ),
            ],
            start_node_id: "start".to_string(),
        };

        let validation = validate_workflow_inner(&workflow);
        assert!(!validation.valid);
        assert!(validation
            .issues
            .iter()
            .any(|issue| issue.code == "node_config_missing"));
        assert!(validation
            .issues
            .iter()
            .any(|issue| issue.code == "connection_target_missing"));
    }
}
