//! Deterministic Workflow execution policy shared by live and headless runtimes.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::conversation_quality::ConversationEvaluation;
use crate::workflow_validation::{WorkflowNode, WorkflowValidationResult};

pub const DEFAULT_WORKFLOW_MAX_STEPS: usize = 64;
pub const WORKFLOW_MAX_STEPS_LIMIT: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionStep {
    pub step_index: usize,
    pub node_id: String,
    pub node_type: String,
    pub label: String,
    pub output: Value,
    pub next_node_id: Option<String>,
    pub stopped_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionReport {
    pub workflow_id: String,
    pub workflow_name: String,
    pub completed: bool,
    pub stopped_reason: Option<String>,
    pub node_count: usize,
    pub executed_node_count: usize,
    pub coverage_percent: f32,
    pub executed_node_ids: Vec<String>,
    pub unvisited_node_ids: Vec<String>,
    pub steps: Vec<WorkflowExecutionStep>,
    pub validation: WorkflowValidationResult,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowExecutionCoverage {
    pub node_count: usize,
    pub executed_node_count: usize,
    pub coverage_percent: f32,
    pub executed_node_ids: Vec<String>,
    pub unvisited_node_ids: Vec<String>,
}

pub fn workflow_step_limit(max_steps: Option<usize>) -> usize {
    max_steps
        .unwrap_or(DEFAULT_WORKFLOW_MAX_STEPS)
        .clamp(1, WORKFLOW_MAX_STEPS_LIMIT)
}

pub fn workflow_next_node(
    node: &WorkflowNode,
    output: &Value,
    choice_selections: &HashMap<String, usize>,
) -> (Option<String>, Option<String>) {
    match node.node_type.as_str() {
        "end" => (None, Some("completed".to_string())),
        "choice" => {
            if node.connections.is_empty() {
                return (None, Some("choice_has_no_connections".to_string()));
            }
            if let Some(index) = choice_selections
                .get(&node.id)
                .copied()
                .or_else(|| config_usize(&node.config, &["selected_index", "default_choice_index"]))
            {
                return node
                    .connections
                    .get(index)
                    .cloned()
                    .map(|next| (Some(next), None))
                    .unwrap_or_else(|| (None, Some("choice_index_out_of_range".to_string())));
            }
            (None, Some("awaiting_choice".to_string()))
        }
        "condition" => branch_by_bool(
            &node.connections,
            output.get("result").and_then(Value::as_bool),
            "condition_result_missing",
        ),
        "evaluation" => branch_by_bool(
            &node.connections,
            output.get("passed").and_then(Value::as_bool),
            "evaluation_threshold_missing",
        ),
        "trigger_event" => branch_by_bool(
            &node.connections,
            output.get("triggered").and_then(Value::as_bool),
            "event_trigger_result_missing",
        ),
        "random_branch" => output
            .get("chosen_connection")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(|value| (Some(value.to_string()), None))
            .unwrap_or_else(|| (None, Some("random_branch_has_no_choice".to_string()))),
        _ => first_connection(&node.connections),
    }
}

pub fn workflow_execution_coverage(
    nodes: &[WorkflowNode],
    steps: &[WorkflowExecutionStep],
) -> WorkflowExecutionCoverage {
    let mut seen = HashSet::new();
    let mut executed_node_ids = Vec::new();
    for step in steps {
        if seen.insert(step.node_id.clone()) {
            executed_node_ids.push(step.node_id.clone());
        }
    }
    let unvisited_node_ids = nodes
        .iter()
        .filter(|node| !seen.contains(&node.id))
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let node_count = nodes.len();
    let executed_node_count = executed_node_ids.len();
    let coverage_percent = if node_count == 0 {
        0.0
    } else {
        (executed_node_count as f32 / node_count as f32) * 100.0
    };

    WorkflowExecutionCoverage {
        node_count,
        executed_node_count,
        coverage_percent,
        executed_node_ids,
        unvisited_node_ids,
    }
}

pub fn config_string(config: &Value, fields: &[&str]) -> Option<String> {
    fields.iter().find_map(|field| {
        config
            .get(field)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
}

pub fn config_string_list(config: &Value, field: &str) -> Vec<String> {
    match config.get(field) {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect(),
        Some(Value::String(text)) => text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToString::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

pub fn config_duration_ms(config: &Value, default_ms: u64) -> u64 {
    if let Some(ms) = config.get("duration_ms").and_then(value_as_u64) {
        return ms;
    }
    if let Some(seconds) = config.get("duration").and_then(value_as_f64) {
        return (seconds.max(0.0) * 1000.0).round() as u64;
    }
    default_ms
}

pub fn config_usize(config: &Value, fields: &[&str]) -> Option<usize> {
    fields.iter().find_map(|field| {
        config.get(field).and_then(|value| match value {
            Value::Number(number) => number.as_u64().map(|value| value as usize),
            Value::String(text) => text.trim().parse::<usize>().ok(),
            _ => None,
        })
    })
}

pub fn optional_config_f32(config: &Value, field: &str) -> Option<f32> {
    config.get(field).and_then(|value| match value {
        Value::Number(number) => number.as_f64().map(|value| value as f32),
        Value::String(text) => text.trim().parse::<f32>().ok(),
        _ => None,
    })
}

pub fn workflow_score_metric(config: &Value) -> String {
    config
        .get("metric")
        .or_else(|| config.get("criteria"))
        .and_then(Value::as_str)
        .map(normalize_workflow_score_metric)
        .unwrap_or_else(|| "overall".to_string())
}

pub fn workflow_metric_score(evaluation: &ConversationEvaluation, metric: &str) -> Option<f32> {
    match metric {
        "friendliness" => Some(evaluation.friendliness),
        "engagement" => Some(evaluation.engagement),
        "creativity" => Some(evaluation.creativity),
        "overall" => Some(evaluation.overall_score),
        _ => None,
    }
}

pub fn workflow_branch_weights(config: &Value, connection_count: usize) -> Vec<f64> {
    if connection_count == 0 {
        return Vec::new();
    }

    let mut weights = match config.get("weights") {
        Some(Value::Array(items)) => items
            .iter()
            .map(|value| value_as_f64(value).unwrap_or(1.0))
            .collect::<Vec<_>>(),
        Some(Value::String(text)) => text
            .lines()
            .map(|line| line.trim().parse::<f64>().unwrap_or(1.0))
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };

    weights.resize(connection_count, 1.0);
    weights.truncate(connection_count);
    for weight in &mut weights {
        if !weight.is_finite() || *weight <= 0.0 {
            *weight = 0.0;
        }
    }
    if weights.iter().sum::<f64>() <= 0.0 {
        weights.fill(1.0);
    }
    weights
}

pub fn select_weighted_branch(weights: &[f64], random: f64) -> usize {
    let total = weights.iter().sum::<f64>();
    let selected_value = random * total;
    let mut accumulated = 0.0;
    for (index, weight) in weights.iter().enumerate() {
        accumulated += weight;
        if selected_value < accumulated {
            return index;
        }
    }
    weights.len().saturating_sub(1)
}

fn first_connection(connections: &[String]) -> (Option<String>, Option<String>) {
    connections
        .first()
        .cloned()
        .map(|next| (Some(next), None))
        .unwrap_or((None, Some("no_next_node".to_string())))
}

fn branch_by_bool(
    connections: &[String],
    value: Option<bool>,
    missing_reason: &str,
) -> (Option<String>, Option<String>) {
    match value {
        Some(true) => connections
            .first()
            .cloned()
            .map(|next| (Some(next), None))
            .unwrap_or((None, Some("true_branch_missing".to_string()))),
        Some(false) => connections
            .get(1)
            .cloned()
            .map(|next| (Some(next), None))
            .unwrap_or((None, Some("false_branch_missing".to_string()))),
        None => connections
            .first()
            .cloned()
            .map(|next| (Some(next), None))
            .unwrap_or((None, Some(missing_reason.to_string()))),
    }
}

fn normalize_workflow_score_metric(metric: &str) -> String {
    match metric.trim().to_lowercase().as_str() {
        "overall_score" | "overall score" | "total" => "overall".to_string(),
        "friendliness_score" | "friendliness score" | "friendly" => "friendliness".to_string(),
        "engagement_score" | "engagement score" | "engaged" => "engagement".to_string(),
        "creativity_score" | "creativity score" | "creative" => "creativity".to_string(),
        "" => "overall".to_string(),
        value => value.to_string(),
    }
}

fn value_as_u64(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.trim().parse::<u64>().ok(),
        _ => None,
    }
}

fn value_as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
#[path = "workflow_execution_policy/tests.rs"]
mod tests;
