//! Deterministic, side-effect-free Workflow preview execution.

use std::collections::{HashMap, HashSet};

use rhai::Dynamic;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::conversation_quality::ConversationEvaluation;
use crate::prompt_guard;
use crate::story_events::{EventScoreSnapshot, EventTriggerContext, StoryEventCatalog};
use crate::workflow_validation::{
    format_validation_errors, validate_workflow_with_catalog, Workflow, WorkflowNode,
    WorkflowRunContext, WorkflowValidationResult,
};
use llm_scripting::{validate_condition_source, ScriptEngine};

const DEFAULT_WORKFLOW_MAX_STEPS: usize = 64;
const WORKFLOW_MAX_STEPS_LIMIT: usize = 256;
const DEFAULT_RANDOM_SEED: u64 = 0x4d4f_4e4f_4741_5441;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPreviewCharacterState {
    #[serde(default = "default_emotion")]
    pub emotion: String,
    #[serde(default)]
    pub relationships: HashMap<String, f32>,
    #[serde(default)]
    pub has_chat_session: bool,
    #[serde(default)]
    pub last_evaluation: Option<ConversationEvaluation>,
    #[serde(default)]
    pub evaluation_count: u32,
    #[serde(default)]
    pub triggered_event_ids: Vec<String>,
}

impl Default for WorkflowPreviewCharacterState {
    fn default() -> Self {
        Self {
            emotion: default_emotion(),
            relationships: HashMap::new(),
            has_chat_session: false,
            last_evaluation: None,
            evaluation_count: 0,
            triggered_event_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowPreviewAppliedEvent {
    pub event_id: String,
    #[serde(default)]
    pub character_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowPreviewEnvironment {
    #[serde(default)]
    pub script_variables: HashMap<String, Value>,
    #[serde(default)]
    pub script_flags: HashMap<String, bool>,
    #[serde(default)]
    pub characters: HashMap<String, WorkflowPreviewCharacterState>,
    #[serde(default)]
    pub applied_events: Vec<WorkflowPreviewAppliedEvent>,
    #[serde(default)]
    pub scene_access: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPreviewOptions {
    #[serde(default)]
    pub max_steps: Option<usize>,
    #[serde(default)]
    pub choice_selections: HashMap<String, usize>,
    #[serde(default)]
    pub run_context: Option<WorkflowRunContext>,
    #[serde(default = "default_random_seed")]
    pub random_seed: u64,
    #[serde(default)]
    pub random_values: Vec<f64>,
}

impl Default for WorkflowPreviewOptions {
    fn default() -> Self {
        Self {
            max_steps: None,
            choice_selections: HashMap::new(),
            run_context: None,
            random_seed: default_random_seed(),
            random_values: Vec::new(),
        }
    }
}

struct WorkflowPreviewState {
    script_engine: ScriptEngine,
    environment: WorkflowPreviewEnvironment,
    relationships: HashMap<String, HashMap<String, f32>>,
    emotions: HashMap<String, String>,
    random: DeterministicRandom,
}

impl WorkflowPreviewState {
    fn new(
        environment: WorkflowPreviewEnvironment,
        random_seed: u64,
        random_values: Vec<f64>,
    ) -> Result<Self, String> {
        let script_engine = ScriptEngine::new();
        script_engine
            .load_json_state(
                environment.script_variables.clone(),
                environment.script_flags.clone(),
            )
            .map_err(|error| error.to_string())?;
        let relationships = HashMap::new();
        let emotions = environment
            .characters
            .iter()
            .map(|(id, state)| (id.clone(), state.emotion.clone()))
            .collect();

        Ok(Self {
            script_engine,
            environment,
            relationships,
            emotions,
            random: DeterministicRandom::new(random_seed, random_values),
        })
    }

    fn relationship(&self, character_id: &str, target_id: &str) -> Option<f32> {
        self.relationships
            .get(character_id)
            .and_then(|targets| targets.get(target_id))
            .copied()
    }

    fn set_relationship(&mut self, character_id: &str, target_id: &str, value: f32) {
        self.relationships
            .entry(character_id.to_string())
            .or_default()
            .insert(target_id.to_string(), value);
    }
}

struct DeterministicRandom {
    state: u64,
    values: Vec<f64>,
    index: usize,
}

impl DeterministicRandom {
    fn new(seed: u64, values: Vec<f64>) -> Self {
        Self {
            state: seed,
            values,
            index: 0,
        }
    }

    fn next_unit(&mut self) -> f64 {
        if let Some(value) = self.values.get(self.index).copied() {
            self.index += 1;
            return normalize_random_value(value);
        }

        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^= value >> 31;
        ((value >> 11) as f64) / ((1u64 << 53) as f64)
    }
}

pub fn execute_workflow_preview(
    workflow: &Workflow,
    event_catalog: &StoryEventCatalog,
    environment: WorkflowPreviewEnvironment,
    mut options: WorkflowPreviewOptions,
) -> Result<WorkflowExecutionReport, String> {
    let validation = validate_workflow_with_catalog(workflow, event_catalog);
    if !validation.valid {
        return Err(format_validation_errors(&validation));
    }

    options.run_context = options
        .run_context
        .filter(|context| context.enabled)
        .map(normalize_workflow_run_context);
    let mut state =
        WorkflowPreviewState::new(environment, options.random_seed, options.random_values)?;
    let node_lookup = workflow
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<HashMap<_, _>>();
    let step_limit = options
        .max_steps
        .unwrap_or(DEFAULT_WORKFLOW_MAX_STEPS)
        .clamp(1, WORKFLOW_MAX_STEPS_LIMIT);
    let mut current_node_id = workflow.start_node_id.clone();
    let mut steps = Vec::new();
    let mut completed = false;
    let mut stopped_reason = None;

    for step_index in 0..step_limit {
        let node = node_lookup
            .get(&current_node_id)
            .copied()
            .ok_or_else(|| format!("Workflow node `{current_node_id}` was not found."))?;
        let output = execute_preview_node(
            node,
            event_catalog,
            options.run_context.as_ref(),
            &mut state,
        )?;
        let (next_node_id, node_stopped_reason) =
            workflow_next_node(node, &output, &options.choice_selections);

        if node.node_type == "end" {
            completed = true;
        }
        steps.push(WorkflowExecutionStep {
            step_index,
            node_id: node.id.clone(),
            node_type: node.node_type.clone(),
            label: node.label.clone(),
            output,
            next_node_id: next_node_id.clone(),
            stopped_reason: node_stopped_reason.clone(),
        });

        if completed {
            stopped_reason = Some("completed".to_string());
            break;
        }
        if let Some(reason) = node_stopped_reason {
            stopped_reason = Some(reason);
            break;
        }
        let Some(next_node_id) = next_node_id else {
            stopped_reason = Some("no_next_node".to_string());
            break;
        };
        current_node_id = next_node_id;
    }

    if steps.len() >= step_limit && !completed && stopped_reason.is_none() {
        stopped_reason = Some("max_steps_reached".to_string());
    }
    let coverage = workflow_execution_coverage(&workflow.nodes, &steps);

    Ok(WorkflowExecutionReport {
        workflow_id: workflow.id.clone(),
        workflow_name: workflow.name.clone(),
        completed,
        stopped_reason,
        node_count: coverage.node_count,
        executed_node_count: coverage.executed_node_count,
        coverage_percent: coverage.coverage_percent,
        executed_node_ids: coverage.executed_node_ids,
        unvisited_node_ids: coverage.unvisited_node_ids,
        steps,
        validation,
    })
}

fn execute_preview_node(
    node: &WorkflowNode,
    event_catalog: &StoryEventCatalog,
    run_context: Option<&WorkflowRunContext>,
    state: &mut WorkflowPreviewState,
) -> Result<Value, String> {
    match node.node_type.as_str() {
        "start" => Ok(json!({
            "action": "start",
            "node_id": node.id,
            "next_connections": node.connections,
        })),
        "end" => Ok(json!({"action": "end", "node_id": node.id, "complete": true})),
        "dialogue" => Ok(json!({
            "action": "dialogue",
            "speaker": config_string(&node.config, &["speaker_id", "speaker"])
                .unwrap_or_else(|| "Narrator".to_string()),
            "text": config_string(&node.config, &["text"]).unwrap_or_default(),
            "emotion": config_string(&node.config, &["emotion"]),
        })),
        "choice" => Ok(json!({
            "action": "choice",
            "choices": config_string_list(&node.config, "choices"),
            "connection_count": node.connections.len(),
        })),
        "set_variable" => {
            let name = node.config["variable_name"].as_str().unwrap_or("");
            let value = dynamic_from_json(&node.config["value"]);
            state
                .script_engine
                .set_variable(name, value)
                .map_err(|error| error.to_string())?;
            Ok(json!({"status": "ok"}))
        }
        "set_flag" => {
            let name = node.config["flag_name"].as_str().unwrap_or("");
            let value = node.config["value"].as_bool().unwrap_or(true);
            state
                .script_engine
                .set_flag(name, value)
                .map_err(|error| error.to_string())?;
            Ok(json!({"status": "ok"}))
        }
        "condition" => {
            let condition = node
                .config
                .get("condition")
                .and_then(Value::as_str)
                .ok_or_else(|| "Condition field `condition` must be a string.".to_string())?;
            validate_condition_source(condition).map_err(|error| error.to_string())?;
            let scope = workflow_condition_scope_variables(&node.config, run_context, state);
            let result = state
                .script_engine
                .evaluate_condition_with_scope_variables(condition, scope)
                .map_err(|error| error.to_string())?;
            Ok(json!({"result": result}))
        }
        "evaluation" => {
            let metric = workflow_score_metric(&node.config);
            let threshold = optional_config_f32(&node.config, "threshold");
            let character_id = workflow_character_id(&node.config, run_context, state);
            let (evaluation, source) =
                workflow_evaluation(character_id.as_deref(), run_context, state);
            let score = workflow_metric_score(&evaluation, &metric).ok_or_else(|| {
                format!(
                    "Unknown evaluation metric `{metric}`. Use friendliness, engagement, creativity, or overall."
                )
            })?;
            let passed = threshold.map(|threshold| score >= threshold);
            if let Some(variable_name) = config_string(&node.config, &["variable_name"]) {
                state
                    .script_engine
                    .set_variable(&variable_name, Dynamic::from(score as f64))
                    .map_err(|error| error.to_string())?;
                if let Some(passed) = passed {
                    state
                        .script_engine
                        .set_flag(&format!("{variable_name}_passed"), passed)
                        .map_err(|error| error.to_string())?;
                }
            }
            Ok(json!({
                "action": "evaluation",
                "character_id": character_id,
                "metric": metric,
                "score": score,
                "threshold": threshold,
                "passed": passed,
                "source": source,
                "evaluation": evaluation,
            }))
        }
        "scene_change" => {
            let scene_id = config_string(&node.config, &["scene_id"])
                .ok_or_else(|| "scene_change node requires scene_id.".to_string())?;
            let access = state
                .environment
                .scene_access
                .get(&scene_id)
                .cloned()
                .unwrap_or(Value::Null);
            Ok(json!({
                "action": "scene_change",
                "scene_id": scene_id,
                "name": config_string(&node.config, &["name"]),
                "background_path": config_string(&node.config, &["background_path", "background"]),
                "bgm_path": config_string(&node.config, &["bgm_path", "bgm"]),
                "access": access,
            }))
        }
        "llm_generate" => Ok(json!({
            "action": "llm_generate",
            "prompt": node.config["prompt"].as_str().unwrap_or(""),
            "system_prompt": config_string(&node.config, &["system_prompt"]),
            "simulated": true,
        })),
        "narration" => Ok(json!({
            "action": "narration",
            "speaker": node.config["speaker"].as_str().unwrap_or("Narrator"),
            "text": node.config["text"].as_str().unwrap_or(""),
        })),
        "bgm" => Ok(json!({
            "action": "bgm",
            "track": config_string(&node.config, &["track_path", "track"]).unwrap_or_default(),
            "play_action": config_string(&node.config, &["action"]).unwrap_or_else(|| "play".to_string()),
            "volume": node.config["volume"].as_f64().unwrap_or(1.0),
        })),
        "sfx" => Ok(json!({
            "action": "sfx",
            "sound": config_string(&node.config, &["sound_path", "sound"]).unwrap_or_default(),
            "volume": node.config["volume"].as_f64().unwrap_or(1.0),
        })),
        "wait" => Ok(json!({
            "action": "wait",
            "duration_ms": config_duration_ms(&node.config, 1000),
        })),
        "random_branch" => {
            let weights = workflow_branch_weights(&node.config, node.connections.len());
            let selected = select_weighted_branch(&weights, state.random.next_unit());
            let chosen = node.connections.get(selected).cloned().unwrap_or_default();
            Ok(json!({
                "action": "random_branch",
                "chosen_connection": chosen,
                "index": selected,
                "weights": weights,
            }))
        }
        "trigger_event" => {
            let event_id = config_string(&node.config, &["event_id"])
                .ok_or_else(|| "trigger_event node requires event_id.".to_string())?;
            let event_type = config_string(&node.config, &["event_type"]);
            let event = event_catalog
                .definition(&event_id, event_type.as_deref())
                .cloned()
                .ok_or_else(|| match event_type.as_deref() {
                    Some(event_type) => {
                        format!("Unknown workflow event `{event_id}` with type `{event_type}`.")
                    }
                    None => format!("Unknown workflow event `{event_id}`."),
                })?;
            let character_id = workflow_character_id(&node.config, run_context, state);
            let (evaluation, evaluation_source) =
                workflow_evaluation(character_id.as_deref(), run_context, state);
            let relationship =
                workflow_relationship(character_id.as_deref(), "player", run_context, state);
            let (evaluation_count, already_triggered) = workflow_event_session_state(
                character_id.as_deref(),
                &event.event_id,
                run_context,
                state,
            );
            let decision = event_catalog.decision_for(
                &event.event_id,
                Some(&event.event_type),
                EventTriggerContext {
                    character_id: character_id.as_deref(),
                    relationship,
                    scores: EventScoreSnapshot {
                        friendliness: evaluation.friendliness,
                        engagement: evaluation.engagement,
                        creativity: evaluation.creativity,
                        overall: evaluation.overall_score,
                    },
                    evaluation_count,
                    already_triggered,
                },
            )?;
            Ok(json!({
                "action": "trigger_event",
                "character_id": character_id,
                "event_id": event.event_id,
                "event_type": event.event_type,
                "triggered": decision.triggered,
                "applied": false,
                "actions": event.actions,
                "application": null,
                "evaluation_source": evaluation_source,
                "decision": decision,
            }))
        }
        "emotion_change" => {
            let character_id = config_string(&node.config, &["character_id"])
                .ok_or_else(|| "emotion_change node requires character_id.".to_string())?;
            let emotion = config_string(&node.config, &["emotion"])
                .ok_or_else(|| "emotion_change node requires emotion.".to_string())?;
            ensure_character(state, &character_id)?;
            let previous_emotion = state
                .emotions
                .insert(character_id.clone(), emotion.clone())
                .unwrap_or_else(default_emotion);
            Ok(json!({
                "action": "emotion_change",
                "character_id": character_id,
                "previous_emotion": previous_emotion,
                "emotion": emotion,
            }))
        }
        "relationship" => {
            let character_id = config_string(&node.config, &["character_id"])
                .ok_or_else(|| "relationship node requires character_id.".to_string())?;
            ensure_character(state, &character_id)?;
            let target_id = config_string(&node.config, &["target_id", "other_id"])
                .unwrap_or_else(|| "player".to_string());
            let delta = optional_config_f32(&node.config, "delta").unwrap_or(0.0);
            let previous =
                workflow_relationship(Some(&character_id), &target_id, run_context, state);
            let current = clamp_workflow_relationship(previous + delta);
            state.set_relationship(&character_id, &target_id, current);
            Ok(json!({
                "action": "relationship",
                "character_id": character_id,
                "target_id": target_id,
                "delta": delta,
                "previous": previous,
                "current": current,
            }))
        }
        "sub_workflow" => Ok(json!({
            "action": "sub_workflow",
            "workflow_id": config_string(&node.config, &["workflow_id"]).unwrap_or_default(),
            "workflow_path": config_string(&node.config, &["workflow_path"]),
            "status": "delegated",
        })),
        "camera" => Ok(json!({
            "action": "camera",
            "camera_action": config_string(&node.config, &["action"]).unwrap_or_else(|| "move".to_string()),
            "x": node.config["target_x"].as_f64().unwrap_or(0.0),
            "y": node.config["target_y"].as_f64().unwrap_or(0.0),
            "zoom": node.config["zoom"].as_f64().unwrap_or(1.0),
            "duration_ms": config_duration_ms(&node.config, 500),
        })),
        "shake" => Ok(json!({
            "action": "shake",
            "intensity": node.config["intensity"].as_f64().unwrap_or(5.0),
            "duration_ms": config_duration_ms(&node.config, 300),
        })),
        _ => Err(format!("Unknown node type: {}", node.node_type)),
    }
}

fn workflow_character_id(
    config: &Value,
    run_context: Option<&WorkflowRunContext>,
    state: &WorkflowPreviewState,
) -> Option<String> {
    if let Some(character_id) = config_string(config, &["character_id", "speaker_id", "speaker"]) {
        return Some(character_id);
    }
    if let Some(character_id) = run_context.and_then(workflow_run_context_character_id) {
        return Some(character_id.to_string());
    }
    let mut session_ids = state
        .environment
        .characters
        .iter()
        .filter(|(_, character)| character.has_chat_session)
        .map(|(id, _)| id);
    let only = session_ids.next()?.clone();
    session_ids.next().is_none().then_some(only)
}

fn workflow_evaluation(
    character_id: Option<&str>,
    run_context: Option<&WorkflowRunContext>,
    state: &WorkflowPreviewState,
) -> (ConversationEvaluation, &'static str) {
    if workflow_run_context_applies(run_context, character_id) {
        return (
            run_context
                .and_then(|context| context.evaluation.clone())
                .unwrap_or_else(neutral_workflow_evaluation),
            "run_context_evaluation",
        );
    }
    let Some(character_id) = character_id else {
        return (
            neutral_workflow_evaluation(),
            "neutral_no_character_selected",
        );
    };
    let Some(character) = state.environment.characters.get(character_id) else {
        return (neutral_workflow_evaluation(), "neutral_no_chat_session");
    };
    if !character.has_chat_session {
        return (neutral_workflow_evaluation(), "neutral_no_chat_session");
    }
    character
        .last_evaluation
        .clone()
        .map(|evaluation| (evaluation, "last_chat_evaluation"))
        .unwrap_or_else(|| {
            (
                neutral_workflow_evaluation(),
                "neutral_no_recorded_evaluation",
            )
        })
}

fn workflow_relationship(
    character_id: Option<&str>,
    target_id: &str,
    run_context: Option<&WorkflowRunContext>,
    state: &WorkflowPreviewState,
) -> f32 {
    if let Some(value) = character_id.and_then(|id| state.relationship(id, target_id)) {
        return value;
    }
    if workflow_run_context_applies(run_context, character_id) {
        return run_context
            .and_then(|context| context.relationship)
            .unwrap_or(0.0);
    }
    character_id
        .and_then(|id| state.environment.characters.get(id))
        .and_then(|character| character.relationships.get(target_id))
        .copied()
        .unwrap_or(0.0)
}

fn workflow_evaluation_count(
    character_id: Option<&str>,
    run_context: Option<&WorkflowRunContext>,
    state: &WorkflowPreviewState,
) -> u32 {
    if workflow_run_context_applies(run_context, character_id) {
        return run_context
            .and_then(|context| context.evaluation_count)
            .unwrap_or(0);
    }
    character_id
        .and_then(|id| state.environment.characters.get(id))
        .map(|character| character.evaluation_count)
        .unwrap_or(0)
}

fn workflow_event_session_state(
    character_id: Option<&str>,
    event_id: &str,
    run_context: Option<&WorkflowRunContext>,
    state: &WorkflowPreviewState,
) -> (u32, bool) {
    if let Some(context) =
        run_context.filter(|context| workflow_run_context_applies(Some(context), character_id))
    {
        return (
            context.evaluation_count.unwrap_or(0),
            context
                .already_triggered_events
                .iter()
                .any(|id| id == event_id),
        );
    }
    let progress_applied =
        state.environment.applied_events.iter().any(|event| {
            event.event_id == event_id && event.character_id.as_deref() == character_id
        });
    let Some(character_id) = character_id else {
        return (0, progress_applied);
    };
    let Some(character) = state.environment.characters.get(character_id) else {
        return (0, progress_applied);
    };
    (
        character.evaluation_count,
        progress_applied
            || character
                .triggered_event_ids
                .iter()
                .any(|id| id == event_id),
    )
}

fn workflow_condition_scope_variables(
    config: &Value,
    run_context: Option<&WorkflowRunContext>,
    state: &WorkflowPreviewState,
) -> Vec<(String, Dynamic)> {
    let character_id = workflow_character_id(config, run_context, state);
    let (evaluation, evaluation_source) =
        workflow_evaluation(character_id.as_deref(), run_context, state);
    let relationship = workflow_relationship(character_id.as_deref(), "player", run_context, state);
    let evaluation_count = workflow_evaluation_count(character_id.as_deref(), run_context, state);
    vec![
        (
            "character_id".to_string(),
            Dynamic::from(character_id.unwrap_or_default()),
        ),
        (
            "relationship".to_string(),
            Dynamic::from(relationship as f64),
        ),
        (
            "relationship_score".to_string(),
            Dynamic::from(relationship as f64),
        ),
        (
            "evaluation_count".to_string(),
            Dynamic::from(i64::from(evaluation_count)),
        ),
        (
            "friendliness".to_string(),
            Dynamic::from(evaluation.friendliness as f64),
        ),
        (
            "friendliness_score".to_string(),
            Dynamic::from(evaluation.friendliness as f64),
        ),
        (
            "engagement".to_string(),
            Dynamic::from(evaluation.engagement as f64),
        ),
        (
            "engagement_score".to_string(),
            Dynamic::from(evaluation.engagement as f64),
        ),
        (
            "creativity".to_string(),
            Dynamic::from(evaluation.creativity as f64),
        ),
        (
            "creativity_score".to_string(),
            Dynamic::from(evaluation.creativity as f64),
        ),
        (
            "overall".to_string(),
            Dynamic::from(evaluation.overall_score as f64),
        ),
        (
            "overall_score".to_string(),
            Dynamic::from(evaluation.overall_score as f64),
        ),
        (
            "evaluation_source".to_string(),
            Dynamic::from(evaluation_source.to_string()),
        ),
    ]
}

fn workflow_next_node(
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

struct WorkflowExecutionCoverage {
    node_count: usize,
    executed_node_count: usize,
    coverage_percent: f32,
    executed_node_ids: Vec<String>,
    unvisited_node_ids: Vec<String>,
}

fn workflow_execution_coverage(
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

fn workflow_run_context_applies(
    run_context: Option<&WorkflowRunContext>,
    character_id: Option<&str>,
) -> bool {
    let Some(context) = run_context.filter(|context| context.enabled) else {
        return false;
    };
    let Some(context_character_id) = workflow_run_context_character_id(context) else {
        return true;
    };
    character_id
        .map(|character_id| character_id.eq_ignore_ascii_case(context_character_id))
        .unwrap_or(true)
}

fn workflow_run_context_character_id(context: &WorkflowRunContext) -> Option<&str> {
    context
        .character_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn normalize_workflow_run_context(mut context: WorkflowRunContext) -> WorkflowRunContext {
    context.relationship = context.relationship.map(clamp_workflow_relationship);
    context.evaluation = context.evaluation.map(|evaluation| ConversationEvaluation {
        friendliness: clamp_workflow_score(evaluation.friendliness),
        engagement: clamp_workflow_score(evaluation.engagement),
        creativity: clamp_workflow_score(evaluation.creativity),
        overall_score: clamp_workflow_score(evaluation.overall_score),
        summary: prompt_guard::guard_evaluation_summary(&evaluation.summary),
    });
    context
}

fn neutral_workflow_evaluation() -> ConversationEvaluation {
    ConversationEvaluation {
        friendliness: 0.0,
        engagement: 0.0,
        creativity: 0.0,
        overall_score: 0.0,
        summary: "No recorded workflow evaluation is available.".to_string(),
    }
}

fn workflow_score_metric(config: &Value) -> String {
    config
        .get("metric")
        .or_else(|| config.get("criteria"))
        .and_then(Value::as_str)
        .map(normalize_workflow_score_metric)
        .unwrap_or_else(|| "overall".to_string())
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

fn workflow_metric_score(evaluation: &ConversationEvaluation, metric: &str) -> Option<f32> {
    match metric {
        "friendliness" => Some(evaluation.friendliness),
        "engagement" => Some(evaluation.engagement),
        "creativity" => Some(evaluation.creativity),
        "overall" => Some(evaluation.overall_score),
        _ => None,
    }
}

fn workflow_branch_weights(config: &Value, connection_count: usize) -> Vec<f64> {
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

fn select_weighted_branch(weights: &[f64], random: f64) -> usize {
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

fn config_string(config: &Value, fields: &[&str]) -> Option<String> {
    fields.iter().find_map(|field| {
        config
            .get(field)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
}

fn config_string_list(config: &Value, field: &str) -> Vec<String> {
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

fn config_duration_ms(config: &Value, default_ms: u64) -> u64 {
    if let Some(ms) = config.get("duration_ms").and_then(value_as_u64) {
        return ms;
    }
    if let Some(seconds) = config.get("duration").and_then(value_as_f64) {
        return (seconds.max(0.0) * 1000.0).round() as u64;
    }
    default_ms
}

fn config_usize(config: &Value, fields: &[&str]) -> Option<usize> {
    fields.iter().find_map(|field| {
        config.get(field).and_then(|value| match value {
            Value::Number(number) => number.as_u64().map(|value| value as usize),
            Value::String(text) => text.trim().parse::<usize>().ok(),
            _ => None,
        })
    })
}

fn optional_config_f32(config: &Value, field: &str) -> Option<f32> {
    config.get(field).and_then(|value| match value {
        Value::Number(number) => number.as_f64().map(|value| value as f32),
        Value::String(text) => text.trim().parse::<f32>().ok(),
        _ => None,
    })
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

fn dynamic_from_json(value: &Value) -> Dynamic {
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(value) => Dynamic::from(*value),
        Value::Number(value) => value
            .as_i64()
            .map(Dynamic::from)
            .or_else(|| value.as_f64().map(Dynamic::from))
            .unwrap_or(Dynamic::UNIT),
        Value::String(value) => Dynamic::from(value.clone()),
        Value::Array(_) | Value::Object(_) => Dynamic::from(value.to_string()),
    }
}

fn clamp_workflow_score(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

fn clamp_workflow_relationship(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

fn ensure_character(state: &WorkflowPreviewState, character_id: &str) -> Result<(), String> {
    state
        .environment
        .characters
        .contains_key(character_id)
        .then_some(())
        .ok_or_else(|| format!("Character not found: {character_id}"))
}

fn normalize_random_value(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0 - f64::EPSILON)
    } else {
        0.0
    }
}

fn default_emotion() -> String {
    "neutral".to_string()
}

const fn default_random_seed() -> u64 {
    DEFAULT_RANDOM_SEED
}

#[cfg(test)]
#[path = "workflow_preview/tests.rs"]
mod tests;
