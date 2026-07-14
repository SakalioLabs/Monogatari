//! Headless validation for Quality Suite authoring documents.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::story_events::EventTriggerRule;
use crate::workflow_validation::WorkflowRunContext;

const MAX_QUALITY_SUITE_FILES: usize = 256;
const MAX_QUALITY_SUITE_FILE_BYTES: u64 = 2 * 1024 * 1024;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySuiteDocument {
    pub version: String,
    pub name: String,
    pub description: String,
    pub scenarios: Vec<QualityScenarioDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScenarioDocument {
    pub id: String,
    pub category: String,
    pub description: String,
    #[serde(default)]
    pub character_id: Option<String>,
    #[serde(default)]
    pub character_name: Option<String>,
    #[serde(default)]
    pub relationship: f32,
    #[serde(default)]
    pub evaluation_count: u32,
    #[serde(default)]
    pub already_triggered_events: Vec<String>,
    #[serde(default)]
    pub mock_evaluation_response: Option<serde_json::Value>,
    #[serde(default)]
    pub mock_character_response: Option<String>,
    #[serde(default)]
    pub guard_character_response: bool,
    #[serde(default)]
    pub mock_workflow_output: Option<String>,
    #[serde(default)]
    pub mock_recent_memories: Vec<String>,
    #[serde(default)]
    pub workflow_path: Option<String>,
    #[serde(default)]
    pub workflow_max_steps: Option<usize>,
    #[serde(default)]
    pub workflow_run_contexts: Vec<WorkflowRunContext>,
    #[serde(default)]
    pub messages: Vec<QualityMessage>,
    pub expect: QualityExpectation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QualityExpectation {
    #[serde(default)]
    pub min_friendliness: Option<f32>,
    #[serde(default)]
    pub max_friendliness: Option<f32>,
    #[serde(default)]
    pub min_engagement: Option<f32>,
    #[serde(default)]
    pub max_engagement: Option<f32>,
    #[serde(default)]
    pub min_creativity: Option<f32>,
    #[serde(default)]
    pub max_creativity: Option<f32>,
    #[serde(default)]
    pub min_overall: Option<f32>,
    #[serde(default)]
    pub max_overall: Option<f32>,
    #[serde(default)]
    pub min_relationship_delta: Option<f32>,
    #[serde(default)]
    pub max_relationship_delta: Option<f32>,
    #[serde(default)]
    pub prompt_injection_detected: Option<bool>,
    #[serde(default)]
    pub private_reasoning_leak_detected: Option<bool>,
    #[serde(default)]
    pub identity_drift_detected: Option<bool>,
    #[serde(default)]
    pub style_drift_detected: Option<bool>,
    #[serde(default)]
    pub evaluation_summary_leak_detected: Option<bool>,
    #[serde(default)]
    pub workflow_output_leak_detected: Option<bool>,
    #[serde(default)]
    pub workflow_output_equals: Option<String>,
    #[serde(default)]
    pub memory_prompt_leak_detected: Option<bool>,
    #[serde(default)]
    pub runtime_safety_trace_required: bool,
    #[serde(default)]
    pub required_runtime_guard_notes: Vec<String>,
    #[serde(default)]
    pub forbidden_runtime_guard_notes: Vec<String>,
    #[serde(default)]
    pub min_workflow_coverage_percent: Option<f32>,
    #[serde(default)]
    pub expected_workflow_unvisited_nodes: Option<Vec<String>>,
    #[serde(default)]
    pub required_workflow_nodes: Vec<String>,
    #[serde(default)]
    pub forbidden_workflow_nodes: Vec<String>,
    #[serde(default)]
    pub knowledge_anchor_missing_detected: Option<bool>,
    #[serde(default)]
    pub knowledge_boundary_violation_detected: Option<bool>,
    #[serde(default)]
    pub required_knowledge_refs: Vec<String>,
    #[serde(default)]
    pub required_knowledge_markers: Vec<String>,
    #[serde(default)]
    pub forbidden_knowledge_markers: Vec<String>,
    #[serde(default)]
    pub required_response_markers: Vec<String>,
    #[serde(default)]
    pub forbidden_response_markers: Vec<String>,
    #[serde(default)]
    pub expected_events: Vec<String>,
    #[serde(default)]
    pub forbidden_events: Vec<String>,
    #[serde(default)]
    pub expected_event_rules: Vec<EventTriggerRule>,
}

pub fn parse_quality_suite_document(content: &str) -> Result<QualitySuiteDocument, String> {
    let suite: QualitySuiteDocument =
        serde_json::from_str(content).map_err(|error| error.to_string())?;
    let issues = validate_quality_suite_shape(&suite);
    if issues.is_empty() {
        Ok(suite)
    } else {
        Err(format!(
            "Quality suite validation failed:\n{}",
            issues.join("\n")
        ))
    }
}

#[derive(Debug, Clone)]
pub struct LoadedQualitySuiteDocument {
    pub suite: QualitySuiteDocument,
    pub source_path: String,
    pub absolute_path: PathBuf,
}

pub fn load_project_quality_suites(
    project_root: &Path,
) -> Result<Vec<LoadedQualitySuiteDocument>, String> {
    let root = project_root.join("quality_suites");
    if !root.exists() {
        return Ok(Vec::new());
    }
    let metadata = std::fs::symlink_metadata(&root).map_err(|error| {
        format!(
            "Failed to inspect Quality Suite directory `{}`: {error}",
            root.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "Quality Suite path must be a regular directory: {}",
            root.display()
        ));
    }
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve Quality Suite directory `{}`: {error}",
            root.display()
        )
    })?;
    let mut files = std::fs::read_dir(&canonical_root)
        .map_err(|error| {
            format!(
                "Failed to read Quality Suite directory `{}`: {error}",
                root.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to read Quality Suite entry: {error}"))?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    files.sort();
    if files.len() > MAX_QUALITY_SUITE_FILES {
        return Err(format!(
            "Quality Suite catalog exceeds {MAX_QUALITY_SUITE_FILES} JSON files."
        ));
    }
    let mut loaded = Vec::with_capacity(files.len());
    for path in files {
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "Failed to inspect Quality Suite `{}`: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Quality Suite must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_QUALITY_SUITE_FILE_BYTES {
            return Err(format!("Quality Suite `{}` is {} bytes; the limit is {MAX_QUALITY_SUITE_FILE_BYTES} bytes.", path.display(), metadata.len()));
        }
        let canonical = path.canonicalize().map_err(|error| {
            format!(
                "Failed to resolve Quality Suite `{}`: {error}",
                path.display()
            )
        })?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "Quality Suite escapes its project directory: {}",
                path.display()
            ));
        }
        let content = std::fs::read_to_string(&canonical).map_err(|error| {
            format!("Failed to read Quality Suite `{}`: {error}", path.display())
        })?;
        loaded.push(LoadedQualitySuiteDocument {
            suite: parse_quality_suite_document(&content)?,
            source_path: source_label(project_root, &path),
            absolute_path: canonical,
        });
    }
    Ok(loaded)
}

pub fn validate_quality_suite_references(
    suites: &[LoadedQualitySuiteDocument],
    character_ids: &HashSet<String>,
    knowledge_ids: &HashSet<String>,
    event_ids: &HashSet<String>,
    workflow_paths: &HashSet<String>,
) -> Vec<(String, String, String)> {
    let mut issues = Vec::new();
    for loaded in suites {
        for scenario in &loaded.suite.scenarios {
            let mut missing = Vec::new();
            if let Some(id) = scenario
                .character_id
                .as_ref()
                .filter(|id| !character_ids.contains(*id))
            {
                missing.push(("quality_character_missing", "character", id.clone()));
            }
            for id in &scenario.expect.required_knowledge_refs {
                if !knowledge_ids.contains(id) {
                    missing.push(("quality_knowledge_missing", "knowledge", id.clone()));
                }
            }
            for id in scenario
                .expect
                .expected_events
                .iter()
                .chain(&scenario.expect.forbidden_events)
            {
                if !event_ids.contains(id) {
                    missing.push(("quality_event_missing", "Story Event", id.clone()));
                }
            }
            if let Some(path) = &scenario.workflow_path {
                let normalized = path.trim().replace('\\', "/");
                let normalized = normalized.strip_prefix("workflows/").unwrap_or(&normalized);
                if !workflow_paths.contains(normalized) {
                    missing.push(("quality_workflow_missing", "workflow", path.clone()));
                }
            }
            for (code, kind, id) in missing {
                issues.push((
                    code.to_string(),
                    loaded.source_path.clone(),
                    format!(
                        "Quality scenario `{}` references unknown {kind} `{id}`.",
                        scenario.id
                    ),
                ));
            }
        }
    }
    issues
}

fn source_label(root: &Path, path: &Path) -> String {
    let resolved_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    path.strip_prefix(&resolved_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn validate_quality_suite_shape(suite: &QualitySuiteDocument) -> Vec<String> {
    let mut issues = Vec::new();

    if suite.version.trim().is_empty() {
        issues.push("Quality suite version is required.".to_string());
    }
    if suite.name.trim().is_empty() {
        issues.push("Quality suite name is required.".to_string());
    }
    if suite.description.trim().is_empty() {
        issues.push("Quality suite description is required.".to_string());
    }
    if suite.scenarios.is_empty() {
        issues.push("Quality suite must contain at least one scenario.".to_string());
        return issues;
    }

    let mut scenario_ids = HashSet::new();
    for scenario in &suite.scenarios {
        let scenario_id = scenario.id.trim();
        let scenario_label = if scenario_id.is_empty() {
            "<missing-id>"
        } else {
            scenario_id
        };

        if scenario_id.is_empty() {
            issues.push("Scenario id is required.".to_string());
        } else if !scenario_ids.insert(scenario_id.to_string()) {
            issues.push(format!("Scenario id `{scenario_id}` must be unique."));
        }
        if scenario.category.trim().is_empty() {
            issues.push(format!("{scenario_label}: category is required."));
        }
        if scenario.description.trim().is_empty() {
            issues.push(format!("{scenario_label}: description is required."));
        }
        if scenario
            .workflow_path
            .as_deref()
            .is_some_and(|path| path.trim().is_empty())
        {
            issues.push(format!(
                "{scenario_label}: workflow_path must not be blank."
            ));
        }
        if scenario
            .expect
            .min_workflow_coverage_percent
            .is_some_and(|value| !(0.0..=100.0).contains(&value))
        {
            issues.push(format!(
                "{scenario_label}: min_workflow_coverage_percent must be between 0 and 100."
            ));
        }
        if scenario.workflow_path.is_none()
            && (scenario.expect.min_workflow_coverage_percent.is_some()
                || scenario.expect.expected_workflow_unvisited_nodes.is_some()
                || !scenario.expect.required_workflow_nodes.is_empty()
                || !scenario.expect.forbidden_workflow_nodes.is_empty())
        {
            issues.push(format!(
                "{scenario_label}: workflow coverage expectations require workflow_path."
            ));
        }
        validate_quality_score_bounds(scenario_label, &scenario.expect, &mut issues);
        validate_no_expectation_conflicts(scenario_label, &scenario.expect, &mut issues);

        for rule in &scenario.expect.expected_event_rules {
            if rule.event_id.trim().is_empty() {
                issues.push(format!("{scenario_label}: event rule id is required."));
            }
            if rule.event_type.trim().is_empty() {
                issues.push(format!("{scenario_label}: event rule type is required."));
            }
            if rule
                .rule_fingerprint
                .as_deref()
                .is_some_and(|fingerprint| !is_sha256_hex(fingerprint))
            {
                issues.push(format!(
                    "{scenario_label}: rule_fingerprint must be a 64-character SHA-256 hex string."
                ));
            }
            if rule
                .score_metric
                .as_deref()
                .is_some_and(|metric| metric.trim().is_empty())
            {
                issues.push(format!(
                    "{scenario_label}: score_metric must not be blank when provided."
                ));
            }
        }
    }

    issues
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn validate_quality_score_bounds(
    scenario_label: &str,
    expect: &QualityExpectation,
    issues: &mut Vec<String>,
) {
    for (label, min, max) in [
        (
            "friendliness",
            expect.min_friendliness,
            expect.max_friendliness,
        ),
        ("engagement", expect.min_engagement, expect.max_engagement),
        ("creativity", expect.min_creativity, expect.max_creativity),
        ("overall", expect.min_overall, expect.max_overall),
    ] {
        if let Some(value) = min {
            if !(0.0..=1.0).contains(&value) {
                issues.push(format!(
                    "{scenario_label}: min_{label} must be between 0 and 1."
                ));
            }
        }
        if let Some(value) = max {
            if !(0.0..=1.0).contains(&value) {
                issues.push(format!(
                    "{scenario_label}: max_{label} must be between 0 and 1."
                ));
            }
        }
        if let (Some(min), Some(max)) = (min, max) {
            if min > max {
                issues.push(format!(
                    "{scenario_label}: min_{label} must be less than or equal to max_{label}."
                ));
            }
        }
    }

    for (label, value) in [
        ("min_relationship_delta", expect.min_relationship_delta),
        ("max_relationship_delta", expect.max_relationship_delta),
    ] {
        if let Some(value) = value {
            if !(-0.5..=0.5).contains(&value) {
                issues.push(format!(
                    "{scenario_label}: {label} must be between -0.5 and 0.5."
                ));
            }
        }
    }
    if let (Some(min), Some(max)) = (expect.min_relationship_delta, expect.max_relationship_delta) {
        if min > max {
            issues.push(format!(
                "{scenario_label}: min_relationship_delta must be less than or equal to max_relationship_delta."
            ));
        }
    }
}

fn validate_no_expectation_conflicts(
    scenario_label: &str,
    expect: &QualityExpectation,
    issues: &mut Vec<String>,
) {
    push_conflicting_values(
        scenario_label,
        "event",
        "expected_events",
        &expect.expected_events,
        "forbidden_events",
        &expect.forbidden_events,
        issues,
    );
    push_conflicting_values(
        scenario_label,
        "response marker",
        "required_response_markers",
        &expect.required_response_markers,
        "forbidden_response_markers",
        &expect.forbidden_response_markers,
        issues,
    );
    push_conflicting_values(
        scenario_label,
        "knowledge marker",
        "required_knowledge_markers",
        &expect.required_knowledge_markers,
        "forbidden_knowledge_markers",
        &expect.forbidden_knowledge_markers,
        issues,
    );
    push_conflicting_values(
        scenario_label,
        "workflow node",
        "required_workflow_nodes",
        &expect.required_workflow_nodes,
        "forbidden_workflow_nodes",
        &expect.forbidden_workflow_nodes,
        issues,
    );
    push_conflicting_values(
        scenario_label,
        "runtime guard note",
        "required_runtime_guard_notes",
        &expect.required_runtime_guard_notes,
        "forbidden_runtime_guard_notes",
        &expect.forbidden_runtime_guard_notes,
        issues,
    );
}

fn push_conflicting_values(
    scenario_label: &str,
    value_label: &str,
    left_name: &str,
    left: &[String],
    right_name: &str,
    right: &[String],
    issues: &mut Vec<String>,
) {
    let right_values: HashSet<String> = right
        .iter()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .collect();
    for value in left
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        if right_values.contains(&value.to_lowercase()) {
            issues.push(format!(
                "{scenario_label}: {value_label} `{value}` cannot appear in both {left_name} and {right_name}."
            ));
        }
    }
}

#[cfg(test)]
mod tests;
