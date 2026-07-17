//! Headless Quality Suite execution and structured evidence.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::conversation_quality as chat;
use crate::prompt_guard;
use crate::quality_suite_validation::{
    QualityScenarioDocument as QualityScenario, QualitySuiteDocument as QualitySuite,
};
use crate::story_events::{EventTriggerDecision, EventTriggerRule, StoryEventCatalog};
use crate::workflow_preview::{
    execute_workflow_preview, WorkflowPreviewEnvironment, WorkflowPreviewOptions,
};
use crate::workflow_validation::Workflow;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualitySuiteReport {
    pub suite_name: String,
    pub version: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub run_metadata: QualitySuiteRunMetadata,
    pub audit_summary: QualitySuiteAuditSummary,
    pub scenarios: Vec<QualityScenarioReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualitySuiteRunMetadata {
    pub generated_at: String,
    pub engine_version: String,
    pub git_commit: String,
    pub git_short_commit: String,
    pub suite_path: String,
    pub suite_sha256: String,
    pub scenario_count: usize,
    pub pass_rate: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct QualitySuiteAuditSummary {
    pub failed_scenario_ids: Vec<String>,
    pub category_summary: Vec<QualityCategorySummary>,
    pub safety_signal_counts: QualitySafetySignalCounts,
    pub workflow_coverage: Vec<QualityWorkflowCoverageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityCategorySummary {
    pub category: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct QualitySafetySignalCounts {
    pub prompt_injection_detected: usize,
    pub private_reasoning_leak_detected: usize,
    pub identity_drift_detected: usize,
    pub style_drift_detected: usize,
    pub evaluation_summary_leak_detected: usize,
    pub workflow_output_leak_detected: usize,
    pub memory_prompt_leak_detected: usize,
    pub runtime_guard_interventions: usize,
    pub knowledge_anchor_missing_detected: usize,
    pub knowledge_boundary_violation_detected: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityWorkflowCoverageSummary {
    pub scenario_id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub coverage_percent: f32,
    pub executed_node_count: usize,
    pub node_count: usize,
    pub unvisited_node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityScenarioReport {
    pub id: String,
    pub category: String,
    pub passed: bool,
    pub issues: Vec<String>,
    pub evaluation: chat::ConversationEvaluation,
    pub relationship_delta: f32,
    pub triggered_events: Vec<String>,
    pub event_trigger_decisions: Vec<EventTriggerDecision>,
    pub event_rules_verified: Vec<EventTriggerRule>,
    pub prompt_injection_detected: bool,
    pub private_reasoning_leak_detected: bool,
    pub identity_drift_detected: bool,
    pub style_drift_detected: bool,
    pub evaluation_summary_leak_detected: bool,
    pub workflow_output_leak_detected: bool,
    pub workflow_output: Option<String>,
    pub memory_prompt_leak_detected: bool,
    pub runtime_safety_trace: Option<chat::ChatSafetyTrace>,
    pub workflow_coverage: Option<QualityWorkflowCoverageReport>,
    pub knowledge_anchor_missing_detected: bool,
    pub knowledge_boundary_violation_detected: bool,
    pub knowledge_refs_resolved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityWorkflowCoverageReport {
    pub workflow_id: String,
    pub workflow_name: String,
    pub run_count: usize,
    pub node_count: usize,
    pub executed_node_count: usize,
    pub coverage_percent: f32,
    pub executed_node_ids: Vec<String>,
    pub unvisited_node_ids: Vec<String>,
    pub runs: Vec<QualityWorkflowRunReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityWorkflowRunReport {
    pub index: usize,
    pub choice_selections: BTreeMap<String, usize>,
    pub completed: bool,
    pub stopped_reason: Option<String>,
    pub coverage_percent: f32,
    pub executed_node_ids: Vec<String>,
    pub unvisited_node_ids: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct QualitySuiteRunProvenance {
    pub generated_at: String,
    pub engine_version: String,
    pub git_commit: String,
    pub git_short_commit: String,
}

pub fn execute_quality_suite(
    suite: &QualitySuite,
    project_root: Option<&Path>,
    suite_source_path: &str,
    suite_source_sha256: &str,
    event_catalog: &StoryEventCatalog,
    provenance: QualitySuiteRunProvenance,
) -> QualitySuiteReport {
    let mut scenarios = Vec::with_capacity(suite.scenarios.len());
    for scenario in &suite.scenarios {
        scenarios.push(run_quality_scenario(scenario, project_root, event_catalog));
    }
    let passed = scenarios.iter().filter(|scenario| scenario.passed).count();
    let total = scenarios.len();
    let audit_summary = quality_suite_audit_summary(&scenarios);
    let run_metadata = quality_suite_run_metadata(
        total,
        passed,
        suite_source_path,
        suite_source_sha256,
        &provenance,
    );

    QualitySuiteReport {
        suite_name: suite.name.clone(),
        version: suite.version.clone(),
        total,
        passed,
        failed: total - passed,
        run_metadata,
        audit_summary,
        scenarios,
    }
}

fn quality_suite_run_metadata(
    total: usize,
    passed: usize,
    suite_source_path: &str,
    suite_source_sha256: &str,
    provenance: &QualitySuiteRunProvenance,
) -> QualitySuiteRunMetadata {
    QualitySuiteRunMetadata {
        generated_at: provenance.generated_at.clone(),
        engine_version: provenance.engine_version.clone(),
        git_commit: provenance.git_commit.clone(),
        git_short_commit: provenance.git_short_commit.clone(),
        suite_path: suite_source_path.replace('\\', "/"),
        suite_sha256: suite_source_sha256.to_string(),
        scenario_count: total,
        pass_rate: if total > 0 {
            passed as f32 / total as f32
        } else {
            0.0
        },
    }
}

fn quality_suite_audit_summary(scenarios: &[QualityScenarioReport]) -> QualitySuiteAuditSummary {
    let mut categories: BTreeMap<String, QualityCategorySummary> = BTreeMap::new();
    let mut safety_signal_counts = QualitySafetySignalCounts::default();
    let mut failed_scenario_ids = Vec::new();
    let mut workflow_coverage = Vec::new();

    for scenario in scenarios {
        let entry = categories
            .entry(scenario.category.clone())
            .or_insert_with(|| QualityCategorySummary {
                category: scenario.category.clone(),
                total: 0,
                passed: 0,
                failed: 0,
            });
        entry.total += 1;
        if scenario.passed {
            entry.passed += 1;
        } else {
            entry.failed += 1;
            failed_scenario_ids.push(scenario.id.clone());
        }

        if scenario.prompt_injection_detected {
            safety_signal_counts.prompt_injection_detected += 1;
        }
        if scenario.private_reasoning_leak_detected {
            safety_signal_counts.private_reasoning_leak_detected += 1;
        }
        if scenario.identity_drift_detected {
            safety_signal_counts.identity_drift_detected += 1;
        }
        if scenario.style_drift_detected {
            safety_signal_counts.style_drift_detected += 1;
        }
        if scenario.evaluation_summary_leak_detected {
            safety_signal_counts.evaluation_summary_leak_detected += 1;
        }
        if scenario.workflow_output_leak_detected {
            safety_signal_counts.workflow_output_leak_detected += 1;
        }
        if scenario.memory_prompt_leak_detected {
            safety_signal_counts.memory_prompt_leak_detected += 1;
        }
        if scenario.runtime_safety_trace.as_ref().is_some_and(|trace| {
            !trace
                .guard_notes
                .iter()
                .any(|note| note == "no_runtime_safety_interventions")
        }) {
            safety_signal_counts.runtime_guard_interventions += 1;
        }
        if scenario.knowledge_anchor_missing_detected {
            safety_signal_counts.knowledge_anchor_missing_detected += 1;
        }
        if scenario.knowledge_boundary_violation_detected {
            safety_signal_counts.knowledge_boundary_violation_detected += 1;
        }

        if let Some(coverage) = &scenario.workflow_coverage {
            workflow_coverage.push(QualityWorkflowCoverageSummary {
                scenario_id: scenario.id.clone(),
                workflow_id: coverage.workflow_id.clone(),
                workflow_name: coverage.workflow_name.clone(),
                coverage_percent: coverage.coverage_percent,
                executed_node_count: coverage.executed_node_count,
                node_count: coverage.node_count,
                unvisited_node_ids: coverage.unvisited_node_ids.clone(),
            });
        }
    }

    QualitySuiteAuditSummary {
        failed_scenario_ids,
        category_summary: categories.into_values().collect(),
        safety_signal_counts,
        workflow_coverage,
    }
}

fn run_quality_scenario(
    scenario: &QualityScenario,
    project_root: Option<&Path>,
    event_catalog: &StoryEventCatalog,
) -> QualityScenarioReport {
    let messages = scenario_chat_messages(scenario);
    let prompt_injection_detected = scenario.messages.iter().any(|message| {
        message.role == "player" && prompt_guard::has_prompt_injection_markers(&message.content)
    });
    let raw_character_response = scenario_raw_character_response(scenario);
    let character_response = scenario_character_response(scenario, &raw_character_response);
    let runtime_safety_trace =
        scenario_runtime_safety_trace(scenario, &raw_character_response, &character_response);
    let private_reasoning_leak_detected = !character_response.trim().is_empty()
        && prompt_guard::has_private_reasoning_leak(&character_response);
    let identity_drift_detected = !character_response.trim().is_empty()
        && prompt_guard::has_identity_drift(
            scenario.character_name.as_deref().unwrap_or("Sakura"),
            &character_response,
        );
    let style_drift_detected =
        !character_response.trim().is_empty() && prompt_guard::has_style_drift(&character_response);
    let evaluation = scenario_evaluation(scenario, &messages);
    let relationship_delta = scenario_relationship_delta(scenario);
    let event_relationship = (scenario.relationship + relationship_delta * 0.1).clamp(-1.0, 1.0);
    let evaluation_summary_leak_detected =
        prompt_guard::has_private_reasoning_leak(&evaluation.summary)
            || prompt_guard::has_prompt_injection_markers(&evaluation.summary);
    let workflow_output = scenario_workflow_output(scenario);
    let workflow_output_report =
        (!workflow_output.trim().is_empty()).then(|| workflow_output.clone());
    let workflow_output_leak_detected = !workflow_output.trim().is_empty()
        && (prompt_guard::has_private_reasoning_leak(&workflow_output)
            || prompt_guard::has_prompt_injection_markers(&workflow_output));
    let memory_prompt_leak_detected = scenario_memory_prompt_leak_detected(scenario);
    let workflow_evidence = scenario_workflow_coverage(scenario, project_root, event_catalog);
    let knowledge_evidence = scenario_knowledge_evidence(scenario, project_root);
    let knowledge_anchor_missing_detected = !knowledge_evidence.issues.is_empty();
    let knowledge_boundary_violation_detected =
        scenario_knowledge_boundary_violation(scenario, &character_response);
    let event_character_id = scenario.character_id.as_deref().unwrap_or("sakura");
    let event_trigger_decisions = chat::build_event_trigger_decisions(
        event_catalog,
        event_character_id,
        event_relationship,
        &evaluation,
        scenario.evaluation_count,
        &scenario.already_triggered_events,
    );
    let triggered_events: Vec<String> = event_trigger_decisions
        .iter()
        .filter(|decision| decision.triggered)
        .map(|decision| decision.event_id.clone())
        .collect();
    let event_rules_verified = scenario_event_rules(scenario, event_catalog);
    let issues = validate_scenario_expectations(
        scenario,
        &evaluation,
        ScenarioExpectationEvidence {
            relationship_delta,
            prompt_injection_detected,
            private_reasoning_leak_detected,
            identity_drift_detected,
            style_drift_detected,
            evaluation_summary_leak_detected,
            workflow_output_leak_detected,
            workflow_output: &workflow_output,
            memory_prompt_leak_detected,
            runtime_safety_trace: runtime_safety_trace.as_ref(),
            workflow_coverage: workflow_evidence.coverage.as_ref(),
            workflow_issues: &workflow_evidence.issues,
            knowledge_anchor_missing_detected,
            knowledge_boundary_violation_detected,
            character_response: &character_response,
            triggered_events: &triggered_events,
            knowledge_issues: &knowledge_evidence.issues,
            event_rules_verified: &event_rules_verified,
        },
    );

    QualityScenarioReport {
        id: scenario.id.clone(),
        category: scenario.category.clone(),
        passed: issues.is_empty(),
        issues,
        evaluation,
        relationship_delta,
        triggered_events,
        event_trigger_decisions,
        event_rules_verified,
        prompt_injection_detected,
        private_reasoning_leak_detected,
        identity_drift_detected,
        style_drift_detected,
        evaluation_summary_leak_detected,
        workflow_output_leak_detected,
        workflow_output: workflow_output_report,
        memory_prompt_leak_detected,
        runtime_safety_trace,
        workflow_coverage: workflow_evidence.coverage,
        knowledge_anchor_missing_detected,
        knowledge_boundary_violation_detected,
        knowledge_refs_resolved: knowledge_evidence.resolved_refs,
    }
}

fn scenario_raw_character_response(scenario: &QualityScenario) -> String {
    scenario.mock_character_response.clone().unwrap_or_else(|| {
        scenario
            .messages
            .iter()
            .filter(|message| message.role == "character")
            .map(|message| message.content.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn scenario_character_response(scenario: &QualityScenario, response: &str) -> String {
    if scenario.guard_character_response {
        prompt_guard::guard_character_response(
            scenario.character_name.as_deref().unwrap_or("Sakura"),
            response,
        )
    } else {
        response.to_string()
    }
}

fn scenario_runtime_safety_trace(
    scenario: &QualityScenario,
    raw_response: &str,
    guarded_response: &str,
) -> Option<chat::ChatSafetyTrace> {
    if raw_response.trim().is_empty() && guarded_response.trim().is_empty() {
        return None;
    }
    let player_message = scenario
        .messages
        .iter()
        .rev()
        .find(|message| message.role == "player")?;
    let relationship_delta = chat::relationship_delta_for_player_message(&player_message.content);
    let pinned_knowledge_ref_ids = scenario.expect.required_knowledge_refs.clone();
    Some(chat::build_chat_safety_trace(
        &player_message.content,
        scenario.character_name.as_deref().unwrap_or("Sakura"),
        raw_response,
        guarded_response,
        relationship_delta,
        false,
        &pinned_knowledge_ref_ids,
    ))
}

#[derive(Debug, Default)]
struct KnowledgeEvidence {
    resolved_refs: Vec<String>,
    combined_text: String,
    issues: Vec<String>,
}

fn scenario_knowledge_evidence(
    scenario: &QualityScenario,
    project_root: Option<&Path>,
) -> KnowledgeEvidence {
    let expect = &scenario.expect;
    let checks_requested = !expect.required_knowledge_refs.is_empty()
        || !expect.required_knowledge_markers.is_empty()
        || !expect.forbidden_knowledge_markers.is_empty()
        || expect.knowledge_anchor_missing_detected.is_some();

    if !checks_requested {
        return KnowledgeEvidence::default();
    }

    let mut evidence = KnowledgeEvidence::default();
    let Some(root) = project_root else {
        evidence
            .issues
            .push("Knowledge anchor checks require a project data root.".to_string());
        return evidence;
    };

    let Some(character_key) = scenario_character_key(scenario) else {
        evidence.issues.push(
            "Knowledge anchor checks require `character_id` or `character_name`.".to_string(),
        );
        return evidence;
    };

    let Some(character) = load_character_value(root, character_key) else {
        evidence
            .issues
            .push(format!("Character `{character_key}` was not found."));
        return evidence;
    };

    let declared_refs = character_knowledge_refs(&character);
    let mut refs_to_resolve = declared_refs.clone();
    for required in &expect.required_knowledge_refs {
        if !declared_refs.iter().any(|id| id == required) {
            evidence.issues.push(format!(
                "Required knowledge ref `{required}` was not declared by character `{character_key}`."
            ));
        }
        if !refs_to_resolve.iter().any(|id| id == required) {
            refs_to_resolve.push(required.clone());
        }
    }

    evidence
        .combined_text
        .push_str(&inline_character_knowledge(&character));
    for knowledge_ref in refs_to_resolve {
        if knowledge_ref.trim().is_empty() {
            continue;
        }
        match load_knowledge_text(root, &knowledge_ref) {
            Some(text) => {
                if !evidence.resolved_refs.contains(&knowledge_ref) {
                    evidence.resolved_refs.push(knowledge_ref);
                }
                evidence.combined_text.push('\n');
                evidence.combined_text.push_str(&text);
            }
            None => evidence.issues.push(format!(
                "Knowledge ref `{knowledge_ref}` could not be resolved."
            )),
        }
    }

    for marker in &expect.required_knowledge_markers {
        if !contains_marker(&evidence.combined_text, marker) {
            evidence
                .issues
                .push(format!("Required knowledge marker `{marker}` was missing."));
        }
    }

    for marker in &expect.forbidden_knowledge_markers {
        if contains_marker(&evidence.combined_text, marker) {
            evidence.issues.push(format!(
                "Forbidden knowledge marker `{marker}` was present."
            ));
        }
    }

    evidence
}

fn scenario_knowledge_boundary_violation(
    scenario: &QualityScenario,
    character_response: &str,
) -> bool {
    let checks_requested = scenario.category == "knowledge"
        || scenario
            .expect
            .knowledge_boundary_violation_detected
            .is_some();

    checks_requested
        && scenario
            .expect
            .forbidden_response_markers
            .iter()
            .any(|marker| contains_marker(character_response, marker))
}

fn scenario_character_key(scenario: &QualityScenario) -> Option<&str> {
    scenario
        .character_id
        .as_deref()
        .or(scenario.character_name.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn load_character_value(project_root: &Path, character_key: &str) -> Option<serde_json::Value> {
    let characters_dir = project_root.join("characters");
    let direct = characters_dir.join(format!("{}.json", character_key.trim()));
    if let Some(value) = read_json_value(&direct) {
        if let Some(character) = select_character_value(value, character_key) {
            return Some(character);
        }
    }

    for entry in std::fs::read_dir(characters_dir).ok()? {
        let path = entry.ok()?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if let Some(value) = read_json_value(&path) {
            if let Some(character) = select_character_value(value, character_key) {
                return Some(character);
            }
        }
    }

    None
}

fn select_character_value(
    value: serde_json::Value,
    character_key: &str,
) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::Array(items) => items.into_iter().find(|item| {
            value_field_matches(item, "id", character_key)
                || value_field_matches(item, "name", character_key)
        }),
        serde_json::Value::Object(_) => {
            if value_field_matches(&value, "id", character_key)
                || value_field_matches(&value, "name", character_key)
            {
                Some(value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn value_field_matches(value: &serde_json::Value, field: &str, expected: &str) -> bool {
    value
        .get(field)
        .and_then(|field| field.as_str())
        .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
}

fn character_knowledge_refs(character: &serde_json::Value) -> Vec<String> {
    ["knowledge_refs", "knowledgeRefs", "knowledge"]
        .iter()
        .filter_map(|field| character.get(field))
        .find_map(string_array_from_value)
        .unwrap_or_default()
}

fn inline_character_knowledge(character: &serde_json::Value) -> String {
    let Some(entries) = character
        .get("knowledge_entries")
        .and_then(|value| value.as_array())
    else {
        return String::new();
    };

    entries
        .iter()
        .map(|entry| {
            let topic = entry
                .get("topic")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let content = entry
                .get("content")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            format!("{topic}: {content}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn load_knowledge_text(project_root: &Path, knowledge_ref: &str) -> Option<String> {
    let knowledge_dir = project_root.join("knowledge");
    let direct = knowledge_dir.join(format!("{}.json", knowledge_ref.trim()));
    if let Some(value) = read_json_value(&direct) {
        if let Some(text) = knowledge_text_from_value(value, knowledge_ref) {
            return Some(text);
        }
    }

    for entry in std::fs::read_dir(knowledge_dir).ok()? {
        let path = entry.ok()?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if let Some(value) = read_json_value(&path) {
            if let Some(text) = knowledge_text_from_value(value, knowledge_ref) {
                return Some(text);
            }
        }
    }

    None
}

fn knowledge_text_from_value(value: serde_json::Value, knowledge_ref: &str) -> Option<String> {
    match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .find(|item| value_field_matches(item, "id", knowledge_ref))
            .map(knowledge_entry_text),
        serde_json::Value::Object(_) => {
            if value_field_matches(&value, "id", knowledge_ref) {
                Some(knowledge_entry_text(value))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn knowledge_entry_text(entry: serde_json::Value) -> String {
    let title = entry
        .get("title")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let content = entry
        .get("content")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let tags = entry
        .get("tags")
        .and_then(string_array_from_value)
        .unwrap_or_default()
        .join(", ");
    format!("{title}\n{content}\n{tags}")
}

fn string_array_from_value(value: &serde_json::Value) -> Option<Vec<String>> {
    Some(
        value
            .as_array()?
            .iter()
            .filter_map(|item| item.as_str())
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect(),
    )
}

fn read_json_value(path: &Path) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn scenario_workflow_output(scenario: &QualityScenario) -> String {
    scenario
        .mock_workflow_output
        .as_deref()
        .map(prompt_guard::guard_workflow_story_output)
        .unwrap_or_default()
}

fn scenario_memory_prompt_leak_detected(scenario: &QualityScenario) -> bool {
    if scenario.mock_recent_memories.is_empty() {
        return false;
    }

    let mut character = llm_game::characters::Character::new(
        scenario
            .character_id
            .as_deref()
            .unwrap_or("quality_character"),
        scenario
            .character_name
            .as_deref()
            .unwrap_or("Quality Character"),
    );
    for memory in &scenario.mock_recent_memories {
        character.add_memory(
            memory.clone(),
            llm_game::characters::memory::MemoryType::Conversation,
            0.8,
            vec!["conversation".to_string(), "quality".to_string()],
        );
    }

    let prompt = character.build_system_prompt();
    prompt_guard::has_private_reasoning_leak(&prompt)
        || prompt_guard::has_prompt_injection_markers(&prompt)
}

#[derive(Debug, Default)]
struct WorkflowCoverageEvidence {
    coverage: Option<QualityWorkflowCoverageReport>,
    issues: Vec<String>,
}

fn scenario_workflow_coverage(
    scenario: &QualityScenario,
    project_root: Option<&Path>,
    event_catalog: &StoryEventCatalog,
) -> WorkflowCoverageEvidence {
    let Some(workflow_path) = scenario
        .workflow_path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    else {
        return WorkflowCoverageEvidence::default();
    };

    let mut evidence = WorkflowCoverageEvidence::default();
    let Some(root) = project_root else {
        evidence
            .issues
            .push("Workflow coverage checks require a project data root.".to_string());
        return evidence;
    };

    let workflow_path = match resolve_project_file(root, workflow_path) {
        Ok(path) => path,
        Err(error) => {
            evidence.issues.push(error);
            return evidence;
        }
    };

    let content = match std::fs::read_to_string(&workflow_path) {
        Ok(content) => content,
        Err(error) => {
            evidence.issues.push(format!(
                "Workflow fixture `{}` could not be read: {error}",
                workflow_path.display()
            ));
            return evidence;
        }
    };

    let workflow: Workflow = match serde_json::from_str(&content) {
        Ok(workflow) => workflow,
        Err(error) => {
            evidence.issues.push(format!(
                "Workflow fixture `{}` could not be parsed: {error}",
                workflow_path.display()
            ));
            return evidence;
        }
    };

    let run_count = scenario
        .workflow_run_contexts
        .len()
        .max(scenario.workflow_choice_selections.len())
        .max(1);
    let mut run_reports = Vec::with_capacity(run_count);
    let mut executed = HashSet::new();
    for index in 0..run_count {
        let run_context = scenario.workflow_run_contexts.get(index).cloned();
        let choice_selections = scenario
            .workflow_choice_selections
            .get(index)
            .cloned()
            .unwrap_or_default();
        match execute_workflow_preview(
            &workflow,
            event_catalog,
            WorkflowPreviewEnvironment::default(),
            WorkflowPreviewOptions {
                max_steps: scenario.workflow_max_steps,
                choice_selections: choice_selections
                    .iter()
                    .map(|(node_id, selection)| (node_id.clone(), *selection))
                    .collect(),
                run_context,
                ..WorkflowPreviewOptions::default()
            },
        ) {
            Ok(report) => {
                for node_id in &report.executed_node_ids {
                    executed.insert(node_id.clone());
                }
                run_reports.push(QualityWorkflowRunReport {
                    index,
                    choice_selections,
                    completed: report.completed,
                    stopped_reason: report.stopped_reason,
                    coverage_percent: report.coverage_percent,
                    executed_node_ids: report.executed_node_ids,
                    unvisited_node_ids: report.unvisited_node_ids,
                });
            }
            Err(error) => evidence.issues.push(format!(
                "Workflow fixture `{}` run {index} failed: {error}",
                workflow_path.display()
            )),
        }
    }

    let executed_node_ids: Vec<String> = workflow
        .nodes
        .iter()
        .filter(|node| executed.contains(&node.id))
        .map(|node| node.id.clone())
        .collect();
    let unvisited_node_ids: Vec<String> = workflow
        .nodes
        .iter()
        .filter(|node| !executed.contains(&node.id))
        .map(|node| node.id.clone())
        .collect();
    let node_count = workflow.nodes.len();
    let executed_node_count = executed_node_ids.len();
    let coverage_percent = if node_count == 0 {
        0.0
    } else {
        (executed_node_count as f32 / node_count as f32) * 100.0
    };

    evidence.coverage = Some(QualityWorkflowCoverageReport {
        workflow_id: workflow.id,
        workflow_name: workflow.name,
        run_count: run_reports.len(),
        node_count,
        executed_node_count,
        coverage_percent,
        executed_node_ids,
        unvisited_node_ids,
        runs: run_reports,
    });
    evidence
}
fn resolve_project_file(project_root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let candidate = PathBuf::from(relative_path);
    if candidate.is_absolute() {
        return Err(
            "Quality suite fixture paths must be relative to the project data root.".to_string(),
        );
    }

    let root = project_root
        .canonicalize()
        .map_err(|e| format!("Failed to resolve project root: {e}"))?;
    let path = root
        .join(candidate)
        .canonicalize()
        .map_err(|e| format!("Failed to resolve fixture path `{relative_path}`: {e}"))?;

    if !path.starts_with(&root) {
        return Err(
            "Quality suite fixture path must stay inside the project data root.".to_string(),
        );
    }

    Ok(path)
}

fn scenario_chat_messages(scenario: &QualityScenario) -> Vec<chat::ChatMessage> {
    scenario
        .messages
        .iter()
        .map(|message| chat::ChatMessage {
            role: if message.role == "character" {
                "character".to_string()
            } else {
                "player".to_string()
            },
            content: message.content.clone(),
            emotion: None,
            timestamp: "quality-suite".to_string(),
        })
        .collect()
}

fn scenario_event_rules(
    scenario: &QualityScenario,
    event_catalog: &StoryEventCatalog,
) -> Vec<EventTriggerRule> {
    let expected_ids: Vec<&str> = scenario
        .expect
        .expected_event_rules
        .iter()
        .map(|rule| rule.event_id.as_str())
        .collect();

    if expected_ids.is_empty() {
        return Vec::new();
    }

    event_catalog
        .trigger_rules()
        .into_iter()
        .filter(|rule| expected_ids.contains(&rule.event_id.as_str()))
        .collect()
}

fn scenario_evaluation(
    scenario: &QualityScenario,
    messages: &[chat::ChatMessage],
) -> chat::ConversationEvaluation {
    if let Some(response) = &scenario.mock_evaluation_response {
        if let Ok(text) = serde_json::to_string(response) {
            if let Some(parsed) = prompt_guard::parse_evaluation_response(&text) {
                return chat::conversation_evaluation_from_draft(parsed);
            }
        }
    }

    chat::fallback_conversation_evaluation(messages, "Quality suite fallback")
}

fn scenario_relationship_delta(scenario: &QualityScenario) -> f32 {
    scenario
        .messages
        .iter()
        .filter(|message| message.role == "player")
        .map(|message| chat::relationship_delta_for_player_message(&message.content))
        .sum::<f32>()
        .clamp(-0.5, 0.5)
}

#[doc(hidden)]
#[derive(Default)]
pub struct ScenarioExpectationEvidence<'a> {
    pub relationship_delta: f32,
    pub prompt_injection_detected: bool,
    pub private_reasoning_leak_detected: bool,
    pub identity_drift_detected: bool,
    pub style_drift_detected: bool,
    pub evaluation_summary_leak_detected: bool,
    pub workflow_output_leak_detected: bool,
    pub workflow_output: &'a str,
    pub memory_prompt_leak_detected: bool,
    pub runtime_safety_trace: Option<&'a chat::ChatSafetyTrace>,
    pub workflow_coverage: Option<&'a QualityWorkflowCoverageReport>,
    pub workflow_issues: &'a [String],
    pub knowledge_anchor_missing_detected: bool,
    pub knowledge_boundary_violation_detected: bool,
    pub character_response: &'a str,
    pub triggered_events: &'a [String],
    pub knowledge_issues: &'a [String],
    pub event_rules_verified: &'a [EventTriggerRule],
}

#[doc(hidden)]
pub fn validate_scenario_expectations(
    scenario: &QualityScenario,
    evaluation: &chat::ConversationEvaluation,
    evidence: ScenarioExpectationEvidence<'_>,
) -> Vec<String> {
    let ScenarioExpectationEvidence {
        relationship_delta,
        prompt_injection_detected,
        private_reasoning_leak_detected,
        identity_drift_detected,
        style_drift_detected,
        evaluation_summary_leak_detected,
        workflow_output_leak_detected,
        workflow_output,
        memory_prompt_leak_detected,
        runtime_safety_trace,
        workflow_coverage,
        workflow_issues,
        knowledge_anchor_missing_detected,
        knowledge_boundary_violation_detected,
        character_response,
        triggered_events,
        knowledge_issues,
        event_rules_verified,
    } = evidence;
    let mut issues = Vec::new();
    let expect = &scenario.expect;

    if scenario.id.trim().is_empty() {
        issues.push("Scenario id is required.".to_string());
    }

    check_min(
        "friendliness",
        evaluation.friendliness,
        expect.min_friendliness,
        &mut issues,
    );
    check_max(
        "friendliness",
        evaluation.friendliness,
        expect.max_friendliness,
        &mut issues,
    );
    check_min(
        "engagement",
        evaluation.engagement,
        expect.min_engagement,
        &mut issues,
    );
    check_max(
        "engagement",
        evaluation.engagement,
        expect.max_engagement,
        &mut issues,
    );
    check_min(
        "creativity",
        evaluation.creativity,
        expect.min_creativity,
        &mut issues,
    );
    check_max(
        "creativity",
        evaluation.creativity,
        expect.max_creativity,
        &mut issues,
    );
    check_min(
        "overall",
        evaluation.overall_score,
        expect.min_overall,
        &mut issues,
    );
    check_max(
        "overall",
        evaluation.overall_score,
        expect.max_overall,
        &mut issues,
    );
    check_min(
        "relationship_delta",
        relationship_delta,
        expect.min_relationship_delta,
        &mut issues,
    );
    check_max(
        "relationship_delta",
        relationship_delta,
        expect.max_relationship_delta,
        &mut issues,
    );

    if let Some(expected) = expect.prompt_injection_detected {
        if prompt_injection_detected != expected {
            issues.push(format!(
                "prompt_injection_detected expected {expected}, got {prompt_injection_detected}"
            ));
        }
    }

    if let Some(expected) = expect.private_reasoning_leak_detected {
        if private_reasoning_leak_detected != expected {
            issues.push(format!(
                "private_reasoning_leak_detected expected {expected}, got {private_reasoning_leak_detected}"
            ));
        }
    }

    if let Some(expected) = expect.identity_drift_detected {
        if identity_drift_detected != expected {
            issues.push(format!(
                "identity_drift_detected expected {expected}, got {identity_drift_detected}"
            ));
        }
    }

    if let Some(expected) = expect.style_drift_detected {
        if style_drift_detected != expected {
            issues.push(format!(
                "style_drift_detected expected {expected}, got {style_drift_detected}"
            ));
        }
    }

    if let Some(expected) = expect.evaluation_summary_leak_detected {
        if evaluation_summary_leak_detected != expected {
            issues.push(format!(
                "evaluation_summary_leak_detected expected {expected}, got {evaluation_summary_leak_detected}"
            ));
        }
    }

    if let Some(expected) = expect.workflow_output_leak_detected {
        if workflow_output_leak_detected != expected {
            issues.push(format!(
                "workflow_output_leak_detected expected {expected}, got {workflow_output_leak_detected}"
            ));
        }
    }

    if let Some(expected) = &expect.workflow_output_equals {
        if workflow_output != expected {
            issues.push(format!(
                "workflow_output_equals expected `{expected}`, got `{workflow_output}`"
            ));
        }
    }

    if let Some(expected) = expect.memory_prompt_leak_detected {
        if memory_prompt_leak_detected != expected {
            issues.push(format!(
                "memory_prompt_leak_detected expected {expected}, got {memory_prompt_leak_detected}"
            ));
        }
    }

    if expect.runtime_safety_trace_required && runtime_safety_trace.is_none() {
        issues.push("runtime_safety_trace expected, got none".to_string());
    }
    for note in &expect.required_runtime_guard_notes {
        match runtime_safety_trace {
            Some(trace) if trace.guard_notes.contains(note) => {}
            Some(_) => issues.push(format!(
                "runtime_safety_trace missing required guard note `{note}`"
            )),
            None => issues.push(format!(
                "runtime_safety_trace missing required guard note `{note}` because no trace was produced"
            )),
        }
    }
    if let Some(trace) = runtime_safety_trace {
        for note in &expect.forbidden_runtime_guard_notes {
            if trace.guard_notes.contains(note) {
                issues.push(format!(
                    "runtime_safety_trace included forbidden guard note `{note}`"
                ));
            }
        }
        for required_ref in &expect.required_knowledge_refs {
            if !trace.pinned_knowledge_ref_ids.contains(required_ref) {
                issues.push(format!(
                    "runtime_safety_trace missing pinned knowledge ref `{required_ref}`"
                ));
            }
        }
    }

    issues.extend(workflow_issues.iter().cloned());

    let workflow_checks_requested = expect.min_workflow_coverage_percent.is_some()
        || expect.expected_workflow_unvisited_nodes.is_some()
        || !expect.required_workflow_nodes.is_empty()
        || !expect.forbidden_workflow_nodes.is_empty();

    if workflow_checks_requested {
        match workflow_coverage {
            Some(coverage) => {
                if let Some(expected) = expect.min_workflow_coverage_percent {
                    if coverage.coverage_percent + f32::EPSILON < expected {
                        issues.push(format!(
                            "workflow coverage expected >= {expected:.3}%, got {:.3}%",
                            coverage.coverage_percent
                        ));
                    }
                }

                if let Some(expected_unvisited) = &expect.expected_workflow_unvisited_nodes {
                    let expected: HashSet<&str> =
                        expected_unvisited.iter().map(String::as_str).collect();
                    let actual: HashSet<&str> = coverage
                        .unvisited_node_ids
                        .iter()
                        .map(String::as_str)
                        .collect();
                    if actual != expected {
                        issues.push(format!(
                            "workflow unvisited nodes expected {:?}, got {:?}",
                            expected_unvisited, coverage.unvisited_node_ids
                        ));
                    }
                }

                for node_id in &expect.required_workflow_nodes {
                    if !coverage.executed_node_ids.contains(node_id) {
                        issues.push(format!(
                            "Required workflow node `{node_id}` was not executed."
                        ));
                    }
                }

                for node_id in &expect.forbidden_workflow_nodes {
                    if coverage.executed_node_ids.contains(node_id) {
                        issues.push(format!("Forbidden workflow node `{node_id}` was executed."));
                    }
                }
            }
            None => issues.push(
                "Workflow coverage expectations require a workflow_path scenario fixture."
                    .to_string(),
            ),
        }
    }

    issues.extend(knowledge_issues.iter().cloned());

    if let Some(expected) = expect.knowledge_anchor_missing_detected {
        if knowledge_anchor_missing_detected != expected {
            issues.push(format!(
                "knowledge_anchor_missing_detected expected {expected}, got {knowledge_anchor_missing_detected}"
            ));
        }
    }

    if let Some(expected) = expect.knowledge_boundary_violation_detected {
        if knowledge_boundary_violation_detected != expected {
            issues.push(format!(
                "knowledge_boundary_violation_detected expected {expected}, got {knowledge_boundary_violation_detected}"
            ));
        }
    }

    for marker in &expect.required_response_markers {
        if !contains_marker(character_response, marker) {
            issues.push(format!("Required response marker `{marker}` was missing."));
        }
    }

    for marker in &expect.forbidden_response_markers {
        if contains_marker(character_response, marker) {
            issues.push(format!("Forbidden response marker `{marker}` was present."));
        }
    }

    for event_id in &expect.expected_events {
        if !triggered_events.contains(event_id) {
            issues.push(format!("Expected event `{event_id}` was not triggered."));
        }
    }

    for event_id in &expect.forbidden_events {
        if triggered_events.contains(event_id) {
            issues.push(format!("Forbidden event `{event_id}` was triggered."));
        }
    }

    for expected_rule in &expect.expected_event_rules {
        match event_rules_verified
            .iter()
            .find(|rule| rule.event_id == expected_rule.event_id)
        {
            Some(actual_rule) => compare_event_rule(expected_rule, actual_rule, &mut issues),
            None => issues.push(format!(
                "Expected event rule `{}` was not found.",
                expected_rule.event_id
            )),
        }
    }

    issues
}

fn compare_event_rule(
    expected: &EventTriggerRule,
    actual: &EventTriggerRule,
    issues: &mut Vec<String>,
) {
    if actual.event_type != expected.event_type {
        issues.push(format!(
            "Event rule `{}` type expected `{}`, got `{}`.",
            expected.event_id, expected.event_type, actual.event_type
        ));
    }
    if actual.min_relationship != expected.min_relationship {
        issues.push(format!(
            "Event rule `{}` min_relationship expected {:?}, got {:?}.",
            expected.event_id, expected.min_relationship, actual.min_relationship
        ));
    }
    if actual.score_metric != expected.score_metric {
        issues.push(format!(
            "Event rule `{}` score_metric expected {:?}, got {:?}.",
            expected.event_id, expected.score_metric, actual.score_metric
        ));
    }
    if actual.min_score != expected.min_score {
        issues.push(format!(
            "Event rule `{}` min_score expected {:?}, got {:?}.",
            expected.event_id, expected.min_score, actual.min_score
        ));
    }
    if actual.min_evaluation_count != expected.min_evaluation_count {
        issues.push(format!(
            "Event rule `{}` min_evaluation_count expected {:?}, got {:?}.",
            expected.event_id, expected.min_evaluation_count, actual.min_evaluation_count
        ));
    }
    if let Some(expected_fingerprint) = expected.rule_fingerprint.as_deref() {
        if actual.rule_fingerprint.as_deref() != Some(expected_fingerprint) {
            issues.push(format!(
                "Event rule `{}` rule_fingerprint expected {:?}, got {:?}.",
                expected.event_id, expected.rule_fingerprint, actual.rule_fingerprint
            ));
        }
    }
}

fn check_min(label: &str, actual: f32, expected: Option<f32>, issues: &mut Vec<String>) {
    if let Some(expected) = expected {
        if actual < expected {
            issues.push(format!(
                "{label} expected >= {expected:.3}, got {actual:.3}"
            ));
        }
    }
}

fn check_max(label: &str, actual: f32, expected: Option<f32>, issues: &mut Vec<String>) {
    if let Some(expected) = expected {
        if actual > expected {
            issues.push(format!(
                "{label} expected <= {expected:.3}, got {actual:.3}"
            ));
        }
    }
}

fn contains_marker(content: &str, marker: &str) -> bool {
    if marker.trim().is_empty() {
        return true;
    }
    content
        .to_lowercase()
        .contains(&marker.trim().to_lowercase())
}

#[cfg(test)]
#[path = "quality_suite_execution/tests.rs"]
mod tests;
