//! Workflow editor commands (Dify-style no-code workflow).

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
            configurable_fields: vec![
                "character_id".to_string(),
                "delta".to_string(),
            ],
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

#[derive(Serialize)]
pub struct WorkflowNodeTypeInfo {
    pub node_type: String,
    pub label: String,
    pub description: String,
    pub category: String,
    pub configurable_fields: Vec<String>,
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
    let json = serde_json::to_string_pretty(&workflow).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, json)
        .await
        .map_err(|e| e.to_string())?;
    Ok("Workflow saved".to_string())
}

/// Load a workflow from a file.
#[tauri::command]
pub async fn load_workflow(
    _state: State<'_, AppState>,
    path: String,
) -> Result<Workflow, String> {
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}
