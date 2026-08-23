//! Pure Workflow models and graph validation shared by desktop and Agent transports.

use crate::conversation_quality::ConversationEvaluation;
use crate::story_events::StoryEventCatalog;
use llm_core::normalize_script_state_key;
use llm_game::campaign::RoleplayCampaignDefinition;
use llm_game::dialogue::DialogueScript;
use llm_game::scene_roleplay::SceneRoleplayDefinition;
use llm_scripting::validate_condition_source;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

pub const MAX_WORKFLOW_FILES: usize = 1_000;
pub const MAX_WORKFLOW_DEPTH: usize = 8;
pub const MAX_WORKFLOW_FILE_BYTES: u64 = 2 * 1024 * 1024;

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

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowRunContext {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub character_id: Option<String>,
    #[serde(default)]
    pub evaluation: Option<ConversationEvaluation>,
    #[serde(default)]
    pub relationship: Option<f32>,
    #[serde(default)]
    pub evaluation_count: Option<u32>,
    #[serde(default)]
    pub already_triggered_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowNodeTypeInfo {
    pub node_type: String,
    pub label: String,
    pub description: String,
    pub category: String,
    pub configurable_fields: Vec<String>,
}

/// Project content that a Workflow can reference without duplicating story data.
#[derive(Debug, Clone, Default)]
pub struct WorkflowStoryReferenceCatalog {
    dialogue_node_ids: HashMap<String, HashSet<String>>,
    roleplay_ids: HashSet<String>,
    campaign_ids: HashSet<String>,
}

impl WorkflowStoryReferenceCatalog {
    pub fn from_project_content(
        dialogues: &[DialogueScript],
        roleplays: &[SceneRoleplayDefinition],
        campaigns: &[RoleplayCampaignDefinition],
    ) -> Self {
        let mut catalog = Self::default();
        for dialogue in dialogues {
            catalog.register_dialogue(&dialogue.id, dialogue.nodes.keys().cloned());
        }
        for roleplay in roleplays {
            catalog.register_roleplay(&roleplay.id);
        }
        for campaign in campaigns {
            catalog.register_campaign(&campaign.id);
        }
        catalog
    }

    pub fn register_dialogue<I, S>(&mut self, dialogue_id: impl Into<String>, node_ids: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.dialogue_node_ids.insert(
            dialogue_id.into(),
            node_ids.into_iter().map(Into::into).collect(),
        );
    }

    pub fn register_roleplay(&mut self, roleplay_id: impl Into<String>) {
        self.roleplay_ids.insert(roleplay_id.into());
    }

    pub fn register_campaign(&mut self, campaign_id: impl Into<String>) {
        self.campaign_ids.insert(campaign_id.into());
    }
}

/// Return the authoritative Workflow node catalog shared by every Rust transport.
pub fn workflow_node_types() -> Vec<WorkflowNodeTypeInfo> {
    fn node_type(
        node_type: &str,
        label: &str,
        description: &str,
        category: &str,
        configurable_fields: &[&str],
    ) -> WorkflowNodeTypeInfo {
        WorkflowNodeTypeInfo {
            node_type: node_type.to_string(),
            label: label.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            configurable_fields: configurable_fields
                .iter()
                .map(|field| (*field).to_string())
                .collect(),
        }
    }

    vec![
        node_type(
            "start",
            "Start",
            "Starting point of the workflow",
            "flow",
            &[],
        ),
        node_type(
            "dialogue",
            "Dialogue",
            "Show dialogue text from a character",
            "content",
            &["speaker", "text", "emotion", "use_llm"],
        ),
        node_type(
            "dialogue_entry",
            "Dialogue Entry",
            "Reference a real Dialogue node for authoring and stage preview",
            "story",
            &["dialogue_id", "entry_node_id"],
        ),
        node_type(
            "scene_roleplay_entry",
            "Free Talk Entry",
            "Reference a real free-talk Scene Roleplay for authoring and stage preview",
            "story",
            &["roleplay_id"],
        ),
        node_type(
            "roleplay_campaign_entry",
            "Roleplay Campaign Entry",
            "Reference a real multi-scene Roleplay Campaign for authoring and stage preview",
            "story",
            &["campaign_id"],
        ),
        node_type(
            "choice",
            "Choice",
            "Present choices to the player",
            "content",
            &["choices"],
        ),
        node_type(
            "condition",
            "Condition",
            "Branch based on a condition",
            "flow",
            &["condition"],
        ),
        node_type(
            "set_variable",
            "Set Variable",
            "Set a game variable",
            "logic",
            &["variable_name", "value"],
        ),
        node_type(
            "set_flag",
            "Set Flag",
            "Set a game flag",
            "logic",
            &["flag_name", "value"],
        ),
        node_type(
            "llm_generate",
            "LLM Generate",
            "Generate text using LLM",
            "ai",
            &["prompt", "system_prompt", "max_tokens"],
        ),
        node_type(
            "evaluation",
            "Evaluation",
            "Read the latest LLM conversation score and compare a threshold",
            "ai",
            &["character_id", "criteria", "threshold", "variable_name"],
        ),
        node_type(
            "scene_change",
            "Scene Change",
            "Change the active background scene",
            "content",
            &["scene_id"],
        ),
        node_type(
            "trigger_event",
            "Trigger Event",
            "Preview and trigger a score-aware story event",
            "flow",
            &["character_id", "event_id", "event_type"],
        ),
        node_type(
            "emotion_change",
            "Change Emotion",
            "Change a character's emotion",
            "character",
            &["character_id", "emotion"],
        ),
        node_type(
            "relationship",
            "Relationship",
            "Modify relationship score",
            "character",
            &["character_id", "delta"],
        ),
        node_type("end", "End", "End of the workflow", "flow", &[]),
        node_type(
            "narration",
            "Narration",
            "Display narrator text or inner monologue",
            "content",
            &["text", "speaker"],
        ),
        node_type(
            "bgm",
            "BGM",
            "Control background music playback",
            "media",
            &["track_path", "action", "volume"],
        ),
        node_type(
            "sfx",
            "SFX",
            "Play a sound effect",
            "media",
            &["sound_path", "volume"],
        ),
        node_type(
            "wait",
            "Wait",
            "Pause workflow execution for a duration",
            "flow",
            &["duration_ms"],
        ),
        node_type(
            "random_branch",
            "Random Branch",
            "Randomly select one of multiple branches",
            "flow",
            &["weights"],
        ),
        node_type(
            "sub_workflow",
            "Sub Workflow",
            "Execute another workflow as a subroutine",
            "flow",
            &["workflow_id", "workflow_path"],
        ),
        node_type(
            "camera",
            "Camera",
            "Control camera position, zoom, and effects",
            "media",
            &["action", "target_x", "target_y", "zoom", "duration_ms"],
        ),
        node_type(
            "shake",
            "Shake",
            "Screen shake effect for dramatic moments",
            "media",
            &["intensity", "duration_ms"],
        ),
    ]
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WorkflowFileSummary {
    pub path: String,
    pub workflow_id: String,
    pub name: String,
    pub node_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowValidationIssue {
    pub severity: String,
    pub code: String,
    pub node_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowValidationResult {
    pub valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub issues: Vec<WorkflowValidationIssue>,
}

#[derive(Debug, Clone)]
pub struct LoadedWorkflow {
    pub workflow: Workflow,
    pub source_path: String,
    pub absolute_path: PathBuf,
}

pub fn load_project_workflows(
    project_root: &Path,
    event_catalog: &StoryEventCatalog,
) -> Result<Vec<LoadedWorkflow>, String> {
    let root = project_root.join("workflows");
    if !root.exists() {
        return Ok(Vec::new());
    }
    let metadata = std::fs::symlink_metadata(&root).map_err(|error| {
        format!(
            "Failed to inspect workflow directory `{}`: {error}",
            root.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "Workflow path must be a regular directory: {}",
            root.display()
        ));
    }
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve workflow directory `{}`: {error}",
            root.display()
        )
    })?;
    let mut pending = vec![(canonical_root.clone(), 0usize)];
    let mut files = Vec::new();
    while let Some((directory, depth)) = pending.pop() {
        if depth > MAX_WORKFLOW_DEPTH {
            return Err(format!(
                "Workflow directory depth exceeds {MAX_WORKFLOW_DEPTH}."
            ));
        }
        let mut entries = std::fs::read_dir(&directory)
            .map_err(|error| {
                format!(
                    "Failed to read workflow directory `{}`: {error}",
                    directory.display()
                )
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Failed to read workflow entry: {error}"))?;
        entries.sort_by_key(|entry| entry.path());
        for entry in entries {
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
                format!(
                    "Failed to inspect workflow path `{}`: {error}",
                    path.display()
                )
            })?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "Workflow paths cannot be symbolic links: {}",
                    path.display()
                ));
            }
            if metadata.is_dir() {
                pending.push((path, depth + 1));
            } else if metadata.is_file()
                && path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case("json"))
            {
                if metadata.len() > MAX_WORKFLOW_FILE_BYTES {
                    return Err(format!(
                        "Workflow `{}` is {} bytes; the limit is {MAX_WORKFLOW_FILE_BYTES} bytes.",
                        path.display(),
                        metadata.len()
                    ));
                }
                files.push(path);
                if files.len() > MAX_WORKFLOW_FILES {
                    return Err(format!(
                        "Workflow catalog exceeds {MAX_WORKFLOW_FILES} JSON files."
                    ));
                }
            }
        }
    }
    files.sort();
    let mut ids = HashSet::new();
    let mut loaded = Vec::with_capacity(files.len());
    for path in files {
        let canonical = path
            .canonicalize()
            .map_err(|error| format!("Failed to resolve workflow `{}`: {error}", path.display()))?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "Workflow escapes its project directory: {}",
                path.display()
            ));
        }
        let content = std::fs::read_to_string(&canonical)
            .map_err(|error| format!("Failed to read workflow `{}`: {error}", path.display()))?;
        let workflow: Workflow = serde_json::from_str(&content)
            .map_err(|error| format!("Invalid workflow JSON in `{}`: {error}", path.display()))?;
        let validation = validate_workflow_with_catalog(&workflow, event_catalog);
        if !validation.valid {
            return Err(format!(
                "{}: {}",
                source_label(project_root, &path),
                format_validation_errors(&validation)
            ));
        }
        if !ids.insert(workflow.id.clone()) {
            return Err(format!("Duplicate workflow id `{}`.", workflow.id));
        }
        loaded.push(LoadedWorkflow {
            workflow,
            source_path: source_label(project_root, &path),
            absolute_path: canonical,
        });
    }
    Ok(loaded)
}

pub fn validate_workflow_references(
    workflows: &[LoadedWorkflow],
    scene_ids: &HashSet<String>,
    character_ids: &HashSet<String>,
) -> Vec<(String, String, String)> {
    let workflow_ids = workflows
        .iter()
        .map(|loaded| loaded.workflow.id.as_str())
        .collect::<HashSet<_>>();
    let mut issues = Vec::new();
    for loaded in workflows {
        for node in &loaded.workflow.nodes {
            let missing = match node.node_type.as_str() {
                "scene_change" => config_string(&node.config, &["scene_id"])
                    .filter(|id| !scene_ids.contains(id))
                    .map(|id| ("workflow_scene_missing", "scene", id)),
                "dialogue" => {
                    config_string(&node.config, &["character_id", "speaker_id", "speaker"])
                        .filter(|id| {
                            !character_ids.contains(id)
                                && !matches!(id.as_str(), "player" | "narrator")
                        })
                        .map(|id| ("workflow_character_missing", "character", id))
                }
                "evaluation" | "emotion_change" | "relationship" | "trigger_event" => {
                    config_string(&node.config, &["character_id"])
                        .filter(|id| !character_ids.contains(id))
                        .map(|id| ("workflow_character_missing", "character", id))
                }
                "sub_workflow" => config_string(&node.config, &["workflow_id"])
                    .filter(|id| !workflow_ids.contains(id.as_str()))
                    .map(|id| ("workflow_subworkflow_missing", "workflow", id)),
                _ => None,
            };
            if let Some((code, kind, id)) = missing {
                issues.push((
                    code.to_string(),
                    loaded.source_path.clone(),
                    format!(
                        "Workflow `{}` node `{}` references unknown {kind} `{id}`.",
                        loaded.workflow.id, node.id
                    ),
                ));
            }
        }
    }
    issues
}

pub fn is_story_entry_node_type(node_type: &str) -> bool {
    matches!(
        node_type,
        "dialogue_entry" | "scene_roleplay_entry" | "roleplay_campaign_entry"
    )
}

pub fn workflow_uses_story_entries(workflow: &Workflow) -> bool {
    workflow
        .nodes
        .iter()
        .any(|node| is_story_entry_node_type(&node.node_type))
}

pub fn validate_workflow_story_references(
    workflow: &Workflow,
    story_catalog: &WorkflowStoryReferenceCatalog,
) -> Vec<WorkflowValidationIssue> {
    let mut issues = Vec::new();
    for node in &workflow.nodes {
        match node.node_type.as_str() {
            "dialogue_entry" => {
                let Some(dialogue_id) = config_string(&node.config, &["dialogue_id"]) else {
                    continue;
                };
                let Some(node_ids) = story_catalog.dialogue_node_ids.get(&dialogue_id) else {
                    push_issue(
                        &mut issues,
                        "error",
                        "node_dialogue_unknown",
                        Some(node.id.clone()),
                        format!("Dialogue `{dialogue_id}` is not in the active project catalog."),
                    );
                    continue;
                };
                let Some(entry_node_id) = config_string(&node.config, &["entry_node_id"]) else {
                    continue;
                };
                if !node_ids.contains(&entry_node_id) {
                    push_issue(
                        &mut issues,
                        "error",
                        "node_dialogue_entry_unknown",
                        Some(node.id.clone()),
                        format!(
                            "Dialogue `{dialogue_id}` does not contain entry node `{entry_node_id}`."
                        ),
                    );
                }
            }
            "scene_roleplay_entry" => {
                let Some(roleplay_id) = config_string(&node.config, &["roleplay_id"]) else {
                    continue;
                };
                if !story_catalog.roleplay_ids.contains(&roleplay_id) {
                    push_issue(
                        &mut issues,
                        "error",
                        "node_roleplay_unknown",
                        Some(node.id.clone()),
                        format!(
                            "Scene Roleplay `{roleplay_id}` is not in the active project catalog."
                        ),
                    );
                }
            }
            "roleplay_campaign_entry" => {
                let Some(campaign_id) = config_string(&node.config, &["campaign_id"]) else {
                    continue;
                };
                if !story_catalog.campaign_ids.contains(&campaign_id) {
                    push_issue(
                        &mut issues,
                        "error",
                        "node_campaign_unknown",
                        Some(node.id.clone()),
                        format!(
                            "Roleplay Campaign `{campaign_id}` is not in the active project catalog."
                        ),
                    );
                }
            }
            _ => {}
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

fn config_string(config: &serde_json::Value, fields: &[&str]) -> Option<String> {
    fields.iter().find_map(|field| {
        config.get(*field).and_then(|value| {
            value
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
    })
}

pub fn validate_workflow_graph(workflow: &Workflow) -> WorkflowValidationResult {
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
            if !config_field_present_for_node(node.node_type.as_str(), &node.config, field) {
                push_issue(
                    &mut issues,
                    "error",
                    "node_config_missing",
                    Some(node.id.clone()),
                    format!("Required field `{field}` is missing."),
                );
            }
        }
        validate_workflow_state_keys(node, &mut issues);
        validate_workflow_condition(node, &mut issues);

        let output_count = workflow_output_count(node);
        if node.connections.len() > output_count {
            push_issue(
                &mut issues,
                "error",
                "connection_output_overflow",
                Some(node.id.clone()),
                format!(
                    "Node type `{}` exposes {output_count} output port(s), but has {} connections.",
                    node.node_type,
                    node.connections.len()
                ),
            );
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

pub fn validate_workflow_with_catalog(
    workflow: &Workflow,
    event_catalog: &StoryEventCatalog,
) -> WorkflowValidationResult {
    let mut result = validate_workflow_graph(workflow);

    for node in workflow
        .nodes
        .iter()
        .filter(|node| node.node_type == "trigger_event")
    {
        let Some(event_id) = config_string(&node.config, &["event_id"]) else {
            continue;
        };
        let event_type = config_string(&node.config, &["event_type"]);
        let Some(definition) = event_catalog.definition(&event_id, event_type.as_deref()) else {
            push_issue(
                &mut result.issues,
                "error",
                "node_event_unknown",
                Some(node.id.clone()),
                match event_type {
                    Some(event_type) => format!(
                        "Story event `{event_id}` with type `{event_type}` is not in the active project catalog."
                    ),
                    None => format!(
                        "Story event `{event_id}` is not in the active project catalog."
                    ),
                },
            );
            continue;
        };

        if let Some(character_id) = config_string(&node.config, &["character_id"]) {
            if !definition.applies_to_character(&character_id) {
                push_issue(
                    &mut result.issues,
                    "error",
                    "node_event_character_mismatch",
                    Some(node.id.clone()),
                    format!(
                        "Story event `{event_id}` is not available for character `{character_id}`."
                    ),
                );
            }
        }
    }

    result.error_count = result
        .issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .count();
    result.warning_count = result
        .issues
        .iter()
        .filter(|issue| issue.severity == "warning")
        .count();
    result.valid = result.error_count == 0;
    result
}

pub fn validate_workflow_with_project_catalog(
    workflow: &Workflow,
    event_catalog: &StoryEventCatalog,
    story_catalog: &WorkflowStoryReferenceCatalog,
) -> WorkflowValidationResult {
    let mut result = validate_workflow_with_catalog(workflow, event_catalog);
    result
        .issues
        .extend(validate_workflow_story_references(workflow, story_catalog));
    result.error_count = result
        .issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .count();
    result.warning_count = result
        .issues
        .iter()
        .filter(|issue| issue.severity == "warning")
        .count();
    result.valid = result.error_count == 0;
    result
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

fn validate_workflow_condition(node: &WorkflowNode, issues: &mut Vec<WorkflowValidationIssue>) {
    if node.node_type != "condition" {
        return;
    }

    let Some(value) = node.config.get("condition") else {
        return;
    };
    if value.is_null() {
        return;
    }
    let Some(condition) = value.as_str() else {
        push_issue(
            issues,
            "error",
            "node_condition_invalid",
            Some(node.id.clone()),
            "Condition field `condition` must be a string.",
        );
        return;
    };
    if condition.trim().is_empty() {
        return;
    }
    if let Err(error) = validate_condition_source(condition) {
        push_issue(
            issues,
            "error",
            "node_condition_invalid",
            Some(node.id.clone()),
            format!("Condition field `condition` is invalid: {error}"),
        );
    }
}

fn validate_workflow_state_keys(node: &WorkflowNode, issues: &mut Vec<WorkflowValidationIssue>) {
    let state_key_fields: &[&str] = match node.node_type.as_str() {
        "set_variable" | "evaluation" => &["variable_name"],
        "set_flag" => &["flag_name"],
        _ => &[],
    };

    for field in state_key_fields {
        let Some(value) = node.config.get(field) else {
            continue;
        };
        if value.is_null() {
            continue;
        }
        let Some(value) = value.as_str() else {
            push_issue(
                issues,
                "error",
                "node_state_key_invalid",
                Some(node.id.clone()),
                format!("State key field `{field}` must be a string."),
            );
            continue;
        };
        if value.trim().is_empty() {
            continue;
        }
        if let Err(error) = normalize_script_state_key(value) {
            push_issue(
                issues,
                "error",
                "node_state_key_invalid",
                Some(node.id.clone()),
                format!("State key field `{field}` is invalid: {error}"),
            );
        }
    }
}

pub fn known_node_types() -> HashSet<&'static str> {
    HashSet::from([
        "start",
        "dialogue",
        "dialogue_entry",
        "scene_roleplay_entry",
        "roleplay_campaign_entry",
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
        "narration",
        "bgm",
        "sfx",
        "wait",
        "random_branch",
        "sub_workflow",
        "camera",
        "shake",
        "end",
    ])
}

pub fn required_fields(node_type: &str) -> &'static [&'static str] {
    match node_type {
        "dialogue" => &["text"],
        "dialogue_entry" => &["dialogue_id", "entry_node_id"],
        "scene_roleplay_entry" => &["roleplay_id"],
        "roleplay_campaign_entry" => &["campaign_id"],
        "choice" => &["choices"],
        "condition" => &["condition"],
        "set_variable" => &["variable_name", "value"],
        "set_flag" => &["flag_name", "value"],
        "llm_generate" => &["prompt"],
        "evaluation" => &["criteria"],
        "scene_change" => &["scene_id"],
        "trigger_event" => &["event_id"],
        "narration" => &["text"],
        "bgm" => &["track_path"],
        "sfx" => &["sound_path"],
        "wait" => &["duration_ms"],
        "sub_workflow" => &["workflow_id"],
        "camera" => &[],
        "shake" => &["duration_ms"],
        "emotion_change" => &["character_id", "emotion"],
        "relationship" => &["character_id", "delta"],
        _ => &[],
    }
}

fn config_field_present_for_node(node_type: &str, config: &serde_json::Value, field: &str) -> bool {
    match (node_type, field) {
        ("bgm", "track_path") => ["track_path", "track"]
            .iter()
            .any(|alias| config_field_present(config, alias)),
        ("sfx", "sound_path") => ["sound_path", "sound"]
            .iter()
            .any(|alias| config_field_present(config, alias)),
        ("wait", "duration_ms") => ["duration_ms", "duration"]
            .iter()
            .any(|alias| config_field_present(config, alias)),
        ("shake", "duration_ms") => ["duration_ms", "duration"]
            .iter()
            .any(|alias| config_field_present(config, alias)),
        _ => config_field_present(config, field),
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

fn workflow_output_count(node: &WorkflowNode) -> usize {
    match node.node_type.as_str() {
        "end" => 0,
        "condition" | "evaluation" | "trigger_event" => 2,
        "choice" => indexed_config_len(node.config.get("choices")).max(1),
        "random_branch" => indexed_config_len(node.config.get("weights")).max(2),
        _ => 1,
    }
}

fn indexed_config_len(value: Option<&serde_json::Value>) -> usize {
    match value {
        Some(serde_json::Value::Array(items)) => items.len(),
        Some(serde_json::Value::String(text)) => {
            text.lines().filter(|line| !line.trim().is_empty()).count()
        }
        _ => 0,
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

pub fn format_validation_errors(validation: &WorkflowValidationResult) -> String {
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
mod tests;
