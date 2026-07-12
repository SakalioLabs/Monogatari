//! Versioned, project-scoped story event assets and deterministic trigger evaluation.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use llm_core::normalize_script_state_key;

pub const STORY_EVENT_CATALOG_SCHEMA_V1: &str = "monogatari-story-event-catalog/v1";
pub const EVENT_TRIGGER_RULE_SCHEMA_V1: &str = "monogatari-event-trigger-rule/v1";
pub const EVENT_TRIGGER_RULE_SCHEMA_V2: &str = "monogatari-event-trigger-rule/v2";
pub const MAX_STORY_EVENT_FILE_BYTES: u64 = 512 * 1024;
pub const MAX_STORY_EVENT_CATALOG_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_STORY_EVENT_FILES: usize = 128;
pub const MAX_STORY_EVENTS: usize = 512;

const MAX_EVENT_ID_CHARS: usize = 128;
const MAX_EVENT_TYPE_CHARS: usize = 128;
const MAX_CHARACTER_ID_CHARS: usize = 128;
const MAX_EVENT_DESCRIPTION_CHARS: usize = 2_048;
const MAX_EVENT_DATA_BYTES: usize = 64 * 1024;
const MAX_EVENT_CHARACTER_IDS: usize = 128;
const MAX_EVENT_ACTIONS: usize = 64;
const MAX_EVALUATION_COUNT: u32 = 1_000_000;
const DEFAULT_STORY_EVENT_CATALOG_JSON: &str =
    include_str!("../../../../data/events/story_events.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoryEventAction {
    UnlockScene { scene_id: String },
    UnlockDialogue { dialogue_id: String },
    UnlockEnding { ending_id: String },
    SetFlag { flag: String, value: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggeredEvent {
    pub event_id: String,
    pub event_type: String,
    pub description: String,
    pub data: Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<StoryEventAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventTriggerRule {
    pub event_id: String,
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_fingerprint: Option<String>,
    #[serde(default)]
    pub min_relationship: Option<f32>,
    #[serde(default)]
    pub score_metric: Option<String>,
    #[serde(default)]
    pub min_score: Option<f32>,
    #[serde(default)]
    pub min_evaluation_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub character_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub repeatable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoryEventDefinition {
    pub event_id: String,
    pub event_type: String,
    pub description: String,
    pub data: Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<StoryEventAction>,
    pub rule: EventTriggerRule,
    pub source_path: String,
}

impl StoryEventDefinition {
    pub fn triggered_event(&self) -> TriggeredEvent {
        TriggeredEvent {
            event_id: self.event_id.clone(),
            event_type: self.event_type.clone(),
            description: self.description.clone(),
            data: self.data.clone(),
            actions: self.actions.clone(),
        }
    }

    pub fn applies_to_character(&self, character_id: &str) -> bool {
        self.rule.character_ids.is_empty()
            || self
                .rule
                .character_ids
                .iter()
                .any(|candidate| candidate == character_id)
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct EventScoreSnapshot {
    pub friendliness: f32,
    pub engagement: f32,
    pub creativity: f32,
    pub overall: f32,
}

impl EventScoreSnapshot {
    fn metric(&self, metric: &str) -> Option<f32> {
        match metric {
            "friendliness" => Some(self.friendliness),
            "engagement" => Some(self.engagement),
            "creativity" => Some(self.creativity),
            "overall" => Some(self.overall),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EventTriggerContext<'a> {
    pub character_id: Option<&'a str>,
    pub relationship: f32,
    pub scores: EventScoreSnapshot,
    pub evaluation_count: u32,
    pub already_triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventTriggerDecision {
    pub event_id: String,
    pub event_type: String,
    pub description: String,
    pub triggered: bool,
    pub already_triggered: bool,
    pub actual_relationship: f32,
    pub actual_evaluation_count: u32,
    pub actual_score_metric: Option<String>,
    pub actual_score: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_fingerprint: Option<String>,
    pub rule: Option<EventTriggerRule>,
    pub blocked_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoryEventCatalogSnapshot {
    pub schema: String,
    pub source: String,
    pub event_count: usize,
    pub catalog_fingerprint: String,
    pub events: Vec<StoryEventDefinition>,
}

#[derive(Debug, Clone)]
pub struct StoryEventCatalog {
    source: String,
    definitions: Vec<StoryEventDefinition>,
    catalog_fingerprint: String,
}

impl StoryEventCatalog {
    pub fn from_document_json(content: &str, source_path: &str) -> Result<Self, String> {
        if content.len() as u64 > MAX_STORY_EVENT_FILE_BYTES {
            return Err(format!(
                "Story event document is {} bytes; the limit is {MAX_STORY_EVENT_FILE_BYTES} bytes.",
                content.len()
            ));
        }
        let document = parse_story_event_document(content, source_path)?;
        Self::from_documents("project", vec![(source_path.to_string(), document)])
    }

    pub fn load_from_project_root(project_root: &Path) -> Result<Self, String> {
        let (directory, explicitly_configured) = project_event_directory(project_root)?;
        if explicitly_configured && !directory.exists() {
            return Err(format!(
                "Configured story event directory does not exist: {}",
                directory.display()
            ));
        }
        if directory.exists() {
            let metadata = std::fs::symlink_metadata(&directory).map_err(|error| {
                format!(
                    "Failed to inspect story event directory `{}`: {error}",
                    directory.display()
                )
            })?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "Story event directory cannot be a symbolic link: {}",
                    directory.display()
                ));
            }
            let canonical_root = project_root.canonicalize().map_err(|error| {
                format!(
                    "Failed to resolve project root `{}`: {error}",
                    project_root.display()
                )
            })?;
            let canonical_directory = directory.canonicalize().map_err(|error| {
                format!(
                    "Failed to resolve story event directory `{}`: {error}",
                    directory.display()
                )
            })?;
            if !canonical_directory.starts_with(&canonical_root) {
                return Err(format!(
                    "Story event directory escapes the project root: {}",
                    directory.display()
                ));
            }
        }
        Self::load_from_directory(&directory)
    }

    pub fn load_from_directory(directory: &Path) -> Result<Self, String> {
        if !directory.exists() {
            return Self::compatibility_default();
        }
        if !directory.is_dir() {
            return Err(format!(
                "Story event path is not a directory: {}",
                directory.display()
            ));
        }

        let canonical_directory = directory.canonicalize().map_err(|error| {
            format!(
                "Failed to resolve story event directory `{}`: {error}",
                directory.display()
            )
        })?;
        let mut files = std::fs::read_dir(directory)
            .map_err(|error| {
                format!(
                    "Failed to read story event directory `{}`: {error}",
                    directory.display()
                )
            })?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            })
            .collect::<Vec<_>>();
        files.sort();

        if files.len() > MAX_STORY_EVENT_FILES {
            return Err(format!(
                "Story event directory contains {} JSON files; the limit is {MAX_STORY_EVENT_FILES}.",
                files.len()
            ));
        }

        let mut documents = Vec::with_capacity(files.len());
        let mut total_bytes = 0_u64;
        for path in files {
            let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
                format!(
                    "Failed to inspect story event file `{}`: {error}",
                    path.display()
                )
            })?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "Story event files cannot be symbolic links: {}",
                    path.display()
                ));
            }
            if !metadata.is_file() {
                continue;
            }
            if metadata.len() > MAX_STORY_EVENT_FILE_BYTES {
                return Err(format!(
                    "Story event file `{}` is {} bytes; the limit is {MAX_STORY_EVENT_FILE_BYTES} bytes.",
                    path.display(),
                    metadata.len()
                ));
            }
            total_bytes = total_bytes.saturating_add(metadata.len());
            if total_bytes > MAX_STORY_EVENT_CATALOG_BYTES {
                return Err(format!(
                    "Story event catalog is larger than {MAX_STORY_EVENT_CATALOG_BYTES} bytes."
                ));
            }

            let canonical_path = path.canonicalize().map_err(|error| {
                format!(
                    "Failed to resolve story event file `{}`: {error}",
                    path.display()
                )
            })?;
            if !canonical_path.starts_with(&canonical_directory) {
                return Err(format!(
                    "Story event file escapes its project directory: {}",
                    path.display()
                ));
            }

            let content = std::fs::read_to_string(&canonical_path).map_err(|error| {
                format!(
                    "Failed to read story event file `{}`: {error}",
                    path.display()
                )
            })?;
            let document = parse_story_event_document(&content, &source_label(&path))?;
            documents.push((source_label(&path), document));
        }

        Self::from_documents("project", documents)
    }

    pub fn compatibility_default() -> Result<Self, String> {
        let source = "builtin:events/story_events.json".to_string();
        let document = parse_story_event_document(DEFAULT_STORY_EVENT_CATALOG_JSON, &source)?;
        Self::from_documents("compatibility_default", vec![(source, document)])
    }

    fn from_documents(
        source: &str,
        documents: Vec<(String, StoryEventDocument)>,
    ) -> Result<Self, String> {
        let mut definitions = Vec::new();
        let mut seen_ids = HashSet::new();

        for (source_path, document) in documents {
            if document.schema != STORY_EVENT_CATALOG_SCHEMA_V1 {
                return Err(format!(
                    "Story event file `{source_path}` uses unsupported schema `{}`; expected `{STORY_EVENT_CATALOG_SCHEMA_V1}`.",
                    document.schema
                ));
            }
            for asset in document.events {
                if definitions.len() >= MAX_STORY_EVENTS {
                    return Err(format!(
                        "Story event catalog contains more than {MAX_STORY_EVENTS} events."
                    ));
                }
                let definition = normalize_story_event(asset, &source_path)?;
                if !seen_ids.insert(definition.event_id.clone()) {
                    return Err(format!(
                        "Duplicate story event id `{}` in `{source_path}`.",
                        definition.event_id
                    ));
                }
                definitions.push(definition);
            }
        }

        definitions.sort_by(|left, right| {
            left.event_id
                .cmp(&right.event_id)
                .then_with(|| left.event_type.cmp(&right.event_type))
        });
        let catalog_fingerprint = story_event_catalog_fingerprint(&definitions);
        Ok(Self {
            source: source.to_string(),
            definitions,
            catalog_fingerprint,
        })
    }

    pub fn definitions(&self) -> &[StoryEventDefinition] {
        &self.definitions
    }

    pub fn definition(
        &self,
        event_id: &str,
        event_type: Option<&str>,
    ) -> Option<&StoryEventDefinition> {
        self.definitions.iter().find(|definition| {
            definition.event_id == event_id
                && event_type
                    .map(|event_type| definition.event_type == event_type)
                    .unwrap_or(true)
        })
    }

    pub fn trigger_rules(&self) -> Vec<EventTriggerRule> {
        self.definitions
            .iter()
            .map(|definition| definition.rule.clone())
            .collect()
    }

    pub fn event_definitions(&self) -> Vec<TriggeredEvent> {
        self.definitions
            .iter()
            .map(StoryEventDefinition::triggered_event)
            .collect()
    }

    pub fn decisions(
        &self,
        character_id: &str,
        relationship: f32,
        scores: EventScoreSnapshot,
        evaluation_count: u32,
        already_triggered: &[String],
    ) -> Vec<EventTriggerDecision> {
        self.definitions
            .iter()
            .filter(|definition| definition.applies_to_character(character_id))
            .map(|definition| {
                evaluate_story_event(
                    definition,
                    Some(character_id),
                    relationship,
                    scores,
                    evaluation_count,
                    already_triggered
                        .iter()
                        .any(|event_id| event_id == &definition.event_id),
                )
            })
            .collect()
    }

    pub fn decision_for(
        &self,
        event_id: &str,
        event_type: Option<&str>,
        context: EventTriggerContext<'_>,
    ) -> Result<EventTriggerDecision, String> {
        let definition = self
            .definition(event_id, event_type)
            .ok_or_else(|| match event_type {
                Some(event_type) => {
                    format!("Unknown story event `{event_id}` with type `{event_type}`.")
                }
                None => format!("Unknown story event `{event_id}`."),
            })?;
        Ok(evaluate_story_event(
            definition,
            context.character_id,
            context.relationship,
            context.scores,
            context.evaluation_count,
            context.already_triggered,
        ))
    }

    pub fn triggered_events(&self, decisions: &[EventTriggerDecision]) -> Vec<TriggeredEvent> {
        let triggered_ids = decisions
            .iter()
            .filter(|decision| decision.triggered)
            .map(|decision| decision.event_id.as_str())
            .collect::<HashSet<_>>();
        self.definitions
            .iter()
            .filter(|definition| triggered_ids.contains(definition.event_id.as_str()))
            .map(StoryEventDefinition::triggered_event)
            .collect()
    }

    pub fn validate_character_references<'a>(
        &self,
        character_ids: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), String> {
        let known = character_ids.into_iter().collect::<HashSet<_>>();
        for definition in &self.definitions {
            for character_id in &definition.rule.character_ids {
                if !known.contains(character_id.as_str()) {
                    return Err(format!(
                        "Story event `{}` references unknown character `{character_id}`.",
                        definition.event_id
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn validate_content_references(
        &self,
        scene_ids: &HashSet<String>,
        dialogue_ids: &HashSet<String>,
        ending_ids: &HashSet<String>,
    ) -> Result<(), String> {
        for definition in &self.definitions {
            for action in &definition.actions {
                let (kind, id, known) = match action {
                    StoryEventAction::UnlockScene { scene_id } => ("scene", scene_id, scene_ids),
                    StoryEventAction::UnlockDialogue { dialogue_id } => {
                        ("dialogue", dialogue_id, dialogue_ids)
                    }
                    StoryEventAction::UnlockEnding { ending_id } => {
                        ("ending", ending_id, ending_ids)
                    }
                    StoryEventAction::SetFlag { .. } => continue,
                };
                if !known.contains(id) {
                    return Err(format!(
                        "Story event `{}` unlocks unknown {kind} `{id}`.",
                        definition.event_id
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn snapshot(&self) -> StoryEventCatalogSnapshot {
        StoryEventCatalogSnapshot {
            schema: STORY_EVENT_CATALOG_SCHEMA_V1.to_string(),
            source: self.source.clone(),
            event_count: self.definitions.len(),
            catalog_fingerprint: self.catalog_fingerprint.clone(),
            events: self.definitions.clone(),
        }
    }

    pub fn catalog_fingerprint(&self) -> &str {
        &self.catalog_fingerprint
    }

    pub fn project_event_directory(project_root: &Path) -> Result<PathBuf, String> {
        project_event_directory(project_root).map(|(directory, _)| directory)
    }
}

fn project_event_directory(project_root: &Path) -> Result<(PathBuf, bool), String> {
    let settings_path = project_root.join("settings.json");
    if !settings_path.is_file() {
        return Ok((project_root.join("events"), false));
    }

    let content = std::fs::read_to_string(&settings_path).map_err(|error| {
        format!(
            "Failed to read project settings `{}`: {error}",
            settings_path.display()
        )
    })?;
    let Ok(settings) = serde_json::from_str::<Value>(&content) else {
        return Ok((project_root.join("events"), false));
    };
    let Some(configured) = settings.get("paths").and_then(|paths| paths.get("events")) else {
        return Ok((project_root.join("events"), false));
    };
    let configured = configured.as_str().ok_or_else(|| {
        "Project setting `paths.events` must be a relative directory string.".to_string()
    })?;
    let relative = normalize_event_directory_reference(configured)?;
    Ok((project_root.join(relative), true))
}

fn normalize_event_directory_reference(value: &str) -> Result<PathBuf, String> {
    if value.is_empty() || value.trim() != value {
        return Err("Project setting `paths.events` cannot be empty or padded.".to_string());
    }
    if value.chars().any(char::is_control) {
        return Err(
            "Project setting `paths.events` cannot contain control characters.".to_string(),
        );
    }
    if value.contains("://") || value.contains(':') {
        return Err("Project setting `paths.events` cannot be a URI or drive path.".to_string());
    }

    let path = Path::new(value);
    if path.is_absolute() {
        return Err("Project setting `paths.events` must be relative.".to_string());
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(segment) => normalized.push(segment),
            _ => {
                return Err(
                    "Project setting `paths.events` cannot contain current or parent segments."
                        .to_string(),
                );
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err("Project setting `paths.events` cannot be empty.".to_string());
    }
    Ok(normalized)
}

impl Default for StoryEventCatalog {
    fn default() -> Self {
        Self::compatibility_default()
            .expect("the checked-in compatibility story event catalog must be valid")
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoryEventDocument {
    schema: String,
    events: Vec<StoryEventAsset>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoryEventAsset {
    event_id: String,
    event_type: String,
    description: String,
    #[serde(default = "empty_object")]
    data: Value,
    #[serde(default)]
    actions: Vec<StoryEventAction>,
    #[serde(default)]
    character_ids: Vec<String>,
    #[serde(default)]
    repeatable: bool,
    #[serde(default)]
    rule: StoryEventRuleAsset,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoryEventRuleAsset {
    #[serde(default)]
    min_relationship: Option<f32>,
    #[serde(default)]
    score_metric: Option<String>,
    #[serde(default)]
    min_score: Option<f32>,
    #[serde(default)]
    min_evaluation_count: Option<u32>,
}

fn empty_object() -> Value {
    json!({})
}

fn parse_story_event_document(
    content: &str,
    source_path: &str,
) -> Result<StoryEventDocument, String> {
    serde_json::from_str(content)
        .map_err(|error| format!("Invalid story event JSON in `{source_path}`: {error}"))
}

fn normalize_story_event(
    asset: StoryEventAsset,
    source_path: &str,
) -> Result<StoryEventDefinition, String> {
    let event_id =
        validate_portable_id(&asset.event_id, "event_id", MAX_EVENT_ID_CHARS, source_path)?;
    let event_type = validate_portable_id(
        &asset.event_type,
        "event_type",
        MAX_EVENT_TYPE_CHARS,
        source_path,
    )?;
    let description = validate_text(
        &asset.description,
        "description",
        MAX_EVENT_DESCRIPTION_CHARS,
        source_path,
    )?;
    if !asset.data.is_object() {
        return Err(format!(
            "Story event `{event_id}` data in `{source_path}` must be a JSON object."
        ));
    }
    let data_bytes = serde_json::to_vec(&asset.data)
        .map_err(|error| format!("Unable to serialize story event `{event_id}` data: {error}"))?
        .len();
    if data_bytes > MAX_EVENT_DATA_BYTES {
        return Err(format!(
            "Story event `{event_id}` data is {data_bytes} bytes; the limit is {MAX_EVENT_DATA_BYTES} bytes."
        ));
    }

    if asset.character_ids.len() > MAX_EVENT_CHARACTER_IDS {
        return Err(format!(
            "Story event `{event_id}` has {} character ids; the limit is {MAX_EVENT_CHARACTER_IDS}.",
            asset.character_ids.len()
        ));
    }
    let mut character_ids = BTreeSet::new();
    for character_id in asset.character_ids {
        let normalized = validate_portable_id(
            &character_id,
            "character_id",
            MAX_CHARACTER_ID_CHARS,
            source_path,
        )?;
        if !character_ids.insert(normalized.clone()) {
            return Err(format!(
                "Story event `{event_id}` repeats character id `{normalized}`."
            ));
        }
    }
    let character_ids = character_ids.into_iter().collect::<Vec<_>>();

    let actions =
        normalize_story_event_actions(&event_id, asset.actions, &asset.data, source_path)?;
    validate_rule(&event_id, &asset.rule, source_path)?;
    let mut rule = EventTriggerRule {
        event_id: event_id.clone(),
        event_type: event_type.clone(),
        rule_fingerprint: None,
        min_relationship: asset.rule.min_relationship,
        score_metric: asset.rule.score_metric,
        min_score: asset.rule.min_score,
        min_evaluation_count: asset.rule.min_evaluation_count,
        character_ids,
        repeatable: asset.repeatable,
    };
    rule.rule_fingerprint = Some(event_trigger_rule_fingerprint(&rule));

    Ok(StoryEventDefinition {
        event_id,
        event_type,
        description,
        data: asset.data,
        actions,
        rule,
        source_path: source_path.replace('\\', "/"),
    })
}

fn normalize_story_event_actions(
    event_id: &str,
    actions: Vec<StoryEventAction>,
    data: &Value,
    source_path: &str,
) -> Result<Vec<StoryEventAction>, String> {
    if actions.len() > MAX_EVENT_ACTIONS {
        return Err(format!(
            "Story event `{event_id}` has {} actions; the limit is {MAX_EVENT_ACTIONS}.",
            actions.len()
        ));
    }

    let mut normalized = Vec::new();
    let mut seen = HashSet::new();
    for action in actions {
        let action = match action {
            StoryEventAction::UnlockScene { scene_id } => StoryEventAction::UnlockScene {
                scene_id: validate_portable_id(
                    &scene_id,
                    "action scene_id",
                    MAX_EVENT_ID_CHARS,
                    source_path,
                )?,
            },
            StoryEventAction::UnlockDialogue { dialogue_id } => {
                StoryEventAction::UnlockDialogue {
                    dialogue_id: validate_portable_id(
                        &dialogue_id,
                        "action dialogue_id",
                        MAX_EVENT_ID_CHARS,
                        source_path,
                    )?,
                }
            }
            StoryEventAction::UnlockEnding { ending_id } => StoryEventAction::UnlockEnding {
                ending_id: validate_portable_id(
                    &ending_id,
                    "action ending_id",
                    MAX_EVENT_ID_CHARS,
                    source_path,
                )?,
            },
            StoryEventAction::SetFlag { flag, value } => StoryEventAction::SetFlag {
                flag: normalize_script_state_key(&flag).map_err(|error| {
                    format!(
                        "Story event `{event_id}` set_flag action in `{source_path}` is invalid: {error}"
                    )
                })?,
                value,
            },
        };
        if !seen.insert(action.clone()) {
            return Err(format!(
                "Story event `{event_id}` repeats action `{action:?}` in `{source_path}`."
            ));
        }
        normalized.push(action);
    }

    for (field, action_type) in [
        ("unlock_scene", "unlock_scene"),
        ("dialogue_id", "unlock_dialogue"),
        ("unlock_ending", "unlock_ending"),
    ] {
        let Some(value) = data.get(field) else {
            continue;
        };
        let value = value.as_str().ok_or_else(|| {
            format!(
                "Story event `{event_id}` legacy data field `{field}` in `{source_path}` must be a string."
            )
        })?;
        let value = validate_portable_id(
            value,
            &format!("legacy data {field}"),
            MAX_EVENT_ID_CHARS,
            source_path,
        )?;
        let action = match action_type {
            "unlock_scene" => StoryEventAction::UnlockScene { scene_id: value },
            "unlock_dialogue" => StoryEventAction::UnlockDialogue { dialogue_id: value },
            "unlock_ending" => StoryEventAction::UnlockEnding { ending_id: value },
            _ => unreachable!("legacy story event action mapping is exhaustive"),
        };
        if seen.insert(action.clone()) {
            normalized.push(action);
        }
    }

    if normalized.len() > MAX_EVENT_ACTIONS {
        return Err(format!(
            "Story event `{event_id}` has more than {MAX_EVENT_ACTIONS} normalized actions."
        ));
    }
    Ok(normalized)
}

fn validate_rule(
    event_id: &str,
    rule: &StoryEventRuleAsset,
    source_path: &str,
) -> Result<(), String> {
    if let Some(min_relationship) = rule.min_relationship {
        if !min_relationship.is_finite() || !(-1.0..=1.0).contains(&min_relationship) {
            return Err(format!(
                "Story event `{event_id}` min_relationship in `{source_path}` must be between -1 and 1."
            ));
        }
    }
    match (&rule.score_metric, rule.min_score) {
        (Some(metric), Some(min_score)) => {
            if !matches!(
                metric.as_str(),
                "friendliness" | "engagement" | "creativity" | "overall"
            ) {
                return Err(format!(
                    "Story event `{event_id}` uses unknown score_metric `{metric}` in `{source_path}`."
                ));
            }
            if !min_score.is_finite() || !(0.0..=1.0).contains(&min_score) {
                return Err(format!(
                    "Story event `{event_id}` min_score in `{source_path}` must be between 0 and 1."
                ));
            }
        }
        (Some(_), None) => {
            return Err(format!(
                "Story event `{event_id}` score_metric in `{source_path}` requires min_score."
            ));
        }
        (None, Some(_)) => {
            return Err(format!(
                "Story event `{event_id}` min_score in `{source_path}` requires score_metric."
            ));
        }
        (None, None) => {}
    }
    if rule
        .min_evaluation_count
        .is_some_and(|count| count > MAX_EVALUATION_COUNT)
    {
        return Err(format!(
            "Story event `{event_id}` min_evaluation_count in `{source_path}` exceeds {MAX_EVALUATION_COUNT}."
        ));
    }
    Ok(())
}

fn validate_portable_id(
    value: &str,
    field: &str,
    max_chars: usize,
    source_path: &str,
) -> Result<String, String> {
    if value.is_empty() {
        return Err(format!(
            "Story event {field} in `{source_path}` cannot be empty."
        ));
    }
    if value.trim() != value {
        return Err(format!(
            "Story event {field} `{value}` in `{source_path}` cannot have surrounding whitespace."
        ));
    }
    if value.chars().count() > max_chars {
        return Err(format!(
            "Story event {field} `{value}` in `{source_path}` exceeds {max_chars} characters."
        ));
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'))
    {
        return Err(format!(
            "Story event {field} `{value}` in `{source_path}` can contain only ASCII letters, numbers, dots, underscores, or hyphens."
        ));
    }
    Ok(value.to_string())
}

fn validate_text(
    value: &str,
    field: &str,
    max_chars: usize,
    source_path: &str,
) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "Story event {field} in `{source_path}` cannot be empty."
        ));
    }
    if trimmed.chars().count() > max_chars {
        return Err(format!(
            "Story event {field} in `{source_path}` exceeds {max_chars} characters."
        ));
    }
    if trimmed.chars().any(char::is_control) {
        return Err(format!(
            "Story event {field} in `{source_path}` cannot contain control characters."
        ));
    }
    Ok(trimmed.to_string())
}

pub fn evaluate_story_event(
    definition: &StoryEventDefinition,
    character_id: Option<&str>,
    relationship: f32,
    scores: EventScoreSnapshot,
    evaluation_count: u32,
    already_triggered: bool,
) -> EventTriggerDecision {
    let rule = &definition.rule;
    let mut blocked_reasons = Vec::new();
    let mut actual_score_metric = None;
    let mut actual_score = None;

    if !rule.character_ids.is_empty() {
        match character_id {
            Some(character_id) if definition.applies_to_character(character_id) => {}
            Some(character_id) => blocked_reasons.push(format!(
                "Event `{}` is not available for character `{character_id}`.",
                definition.event_id
            )),
            None => blocked_reasons.push(format!(
                "Event `{}` requires a character context.",
                definition.event_id
            )),
        }
    }
    if already_triggered && !rule.repeatable {
        blocked_reasons.push(format!(
            "Event `{}` has already triggered.",
            definition.event_id
        ));
    }
    if let Some(min_relationship) = rule.min_relationship {
        if relationship < min_relationship {
            blocked_reasons.push(format!(
                "relationship {relationship:.3} is below required {min_relationship:.3}"
            ));
        }
    }
    if let Some(min_evaluation_count) = rule.min_evaluation_count {
        if evaluation_count < min_evaluation_count {
            blocked_reasons.push(format!(
                "evaluation_count {evaluation_count} is below required {min_evaluation_count}"
            ));
        }
    }
    if let (Some(metric), Some(min_score)) = (rule.score_metric.as_deref(), rule.min_score) {
        actual_score_metric = Some(metric.to_string());
        match scores.metric(metric) {
            Some(actual) => {
                actual_score = Some(actual);
                if actual < min_score {
                    blocked_reasons.push(format!(
                        "{metric} {actual:.3} is below required {min_score:.3}"
                    ));
                }
            }
            None => blocked_reasons.push(format!("Unknown score metric `{metric}`.")),
        }
    }

    EventTriggerDecision {
        event_id: definition.event_id.clone(),
        event_type: definition.event_type.clone(),
        description: definition.description.clone(),
        triggered: blocked_reasons.is_empty(),
        already_triggered,
        actual_relationship: relationship,
        actual_evaluation_count: evaluation_count,
        actual_score_metric,
        actual_score,
        rule_fingerprint: rule.rule_fingerprint.clone(),
        rule: Some(rule.clone()),
        blocked_reasons,
    }
}

pub fn event_trigger_rule_fingerprint(rule: &EventTriggerRule) -> String {
    let payload = if rule.character_ids.is_empty() && !rule.repeatable {
        json!({
            "schema": EVENT_TRIGGER_RULE_SCHEMA_V1,
            "event_id": rule.event_id.as_str(),
            "event_type": rule.event_type.as_str(),
            "min_relationship": rule.min_relationship.map(format_rule_float),
            "score_metric": rule.score_metric.as_deref(),
            "min_score": rule.min_score.map(format_rule_float),
            "min_evaluation_count": rule.min_evaluation_count,
        })
    } else {
        json!({
            "schema": EVENT_TRIGGER_RULE_SCHEMA_V2,
            "event_id": rule.event_id.as_str(),
            "event_type": rule.event_type.as_str(),
            "min_relationship": rule.min_relationship.map(format_rule_float),
            "score_metric": rule.score_metric.as_deref(),
            "min_score": rule.min_score.map(format_rule_float),
            "min_evaluation_count": rule.min_evaluation_count,
            "character_ids": rule.character_ids,
            "repeatable": rule.repeatable,
        })
    };
    sha256_json(&payload)
}

fn story_event_catalog_fingerprint(definitions: &[StoryEventDefinition]) -> String {
    let events = definitions
        .iter()
        .map(|definition| {
            json!({
                "event_id": definition.event_id,
                "event_type": definition.event_type,
                "description": definition.description,
                "data": definition.data,
                "actions": definition.actions,
                "rule_fingerprint": definition.rule.rule_fingerprint,
            })
        })
        .collect::<Vec<_>>();
    sha256_json(&json!({
        "schema": "monogatari-story-event-catalog-fingerprint/v1",
        "events": events,
    }))
}

fn sha256_json(value: &Value) -> String {
    let encoded = serde_json::to_vec(value).expect("fingerprint payload should serialize");
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    format!("{:x}", hasher.finalize())
}

fn format_rule_float(value: f32) -> String {
    format!("{value:.6}")
}

fn source_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("events/{name}"))
        .unwrap_or_else(|| path.to_string_lossy().replace('\\', "/"))
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_story_events_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn write_catalog(root: &Path, json: &str) -> PathBuf {
        let events = root.join("events");
        std::fs::create_dir_all(&events).unwrap();
        std::fs::write(events.join("catalog.json"), json).unwrap();
        events
    }

    #[test]
    fn checked_in_catalog_preserves_pinned_v1_rule_fingerprints() {
        let catalog = StoryEventCatalog::default();
        let expected = [
            (
                "first_friend",
                "954678be9c9f7d214ce6712145e4ceeb507c270e32e87e8acf13828000a1bb06",
            ),
            (
                "close_friend",
                "f2e4280a3e3bd3fd02a1455bcb829d892cce7ecb31f3f0ad7d592eb183a74705",
            ),
            (
                "best_friend",
                "044ba048b10647016fc08d21c7dee0b7e3d2f52b6cf28df4d52a0d7d052b7027",
            ),
            (
                "high_engagement",
                "321b18638227d52cfd1f3cfc730c5906c670c46ce6402de8cc603380ad339809",
            ),
            (
                "creative_talk",
                "b30ee634a448d8a40355a0a3e90a79ea3220b8e291e80fec6610a31d622a3658",
            ),
            (
                "dedicated_player",
                "be35e779dccc13ad5ce640d93d3524c39e155cc119b8aa0b0f13879c562cbe86",
            ),
            (
                "super_dedicated",
                "65f1d0d4ced603aa517f51c6a1ed514a3b7904a0d51342eff5c3522781c5d14c",
            ),
        ];

        assert_eq!(catalog.definitions().len(), expected.len());
        for (event_id, fingerprint) in expected {
            assert_eq!(
                catalog
                    .definition(event_id, None)
                    .and_then(|definition| definition.rule.rule_fingerprint.as_deref()),
                Some(fingerprint)
            );
        }
    }

    #[test]
    fn checked_in_catalog_preserves_cross_runtime_catalog_fingerprint() {
        assert_eq!(
            StoryEventCatalog::default().catalog_fingerprint(),
            "f79ea33cd8ee91e961889e74cc3db23995219711c50324cad6fd9dae94a25b10"
        );
    }

    #[test]
    fn project_catalog_supports_character_scope_and_repeatable_rules() {
        let root = temp_root("scoped");
        let events = write_catalog(
            &root,
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"luna_secret",
                "event_type":"special_dialogue",
                "description":"Luna shares a secret.",
                "character_ids":["luna"],
                "repeatable":true,
                "rule":{"score_metric":"overall","min_score":0.7,"min_evaluation_count":2}
              }]
            }"#,
        );
        let catalog = StoryEventCatalog::load_from_directory(&events).unwrap();
        let scores = EventScoreSnapshot {
            overall: 0.8,
            ..Default::default()
        };

        let luna = catalog
            .decision_for(
                "luna_secret",
                None,
                EventTriggerContext {
                    character_id: Some("luna"),
                    scores,
                    evaluation_count: 2,
                    already_triggered: true,
                    ..Default::default()
                },
            )
            .unwrap();
        let sakura = catalog
            .decision_for(
                "luna_secret",
                None,
                EventTriggerContext {
                    character_id: Some("sakura"),
                    scores,
                    evaluation_count: 2,
                    ..Default::default()
                },
            )
            .unwrap();

        assert!(luna.triggered, "{:?}", luna.blocked_reasons);
        assert!(!sakura.triggered);
        assert!(sakura.blocked_reasons[0].contains("not available"));
        assert_eq!(
            luna.rule.as_ref().unwrap().rule_fingerprint,
            catalog
                .definition("luna_secret", None)
                .unwrap()
                .rule
                .rule_fingerprint
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_catalog_rejects_duplicate_ids_and_invalid_metrics() {
        let duplicate_root = temp_root("duplicate");
        let duplicate_events = write_catalog(
            &duplicate_root,
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[
                {"event_id":"same","event_type":"unlock","description":"One"},
                {"event_id":"same","event_type":"unlock","description":"Two"}
              ]
            }"#,
        );
        let error = StoryEventCatalog::load_from_directory(&duplicate_events).unwrap_err();
        assert!(error.contains("Duplicate story event id `same`"));
        std::fs::remove_dir_all(duplicate_root).unwrap();

        let metric_root = temp_root("metric");
        let metric_events = write_catalog(
            &metric_root,
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"bad_metric",
                "event_type":"unlock",
                "description":"Bad metric",
                "rule":{"score_metric":"hidden_score","min_score":0.5}
              }]
            }"#,
        );
        let error = StoryEventCatalog::load_from_directory(&metric_events).unwrap_err();
        assert!(error.contains("unknown score_metric `hidden_score`"));
        std::fs::remove_dir_all(metric_root).unwrap();
    }

    #[test]
    fn catalog_normalizes_typed_actions_and_legacy_data_actions() {
        let root = temp_root("actions");
        let events = write_catalog(
            &root,
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"luna_unlock",
                "event_type":"unlock",
                "description":"Unlock Luna content",
                "data":{"dialogue_id":"legacy_dialogue"},
                "actions":[
                  {"type":"unlock_scene","scene_id":"moon_garden"},
                  {"type":"set_flag","flag":"luna.secret","value":true}
                ]
              }]
            }"#,
        );

        let catalog = StoryEventCatalog::load_from_directory(&events).unwrap();
        let actions = &catalog.definition("luna_unlock", None).unwrap().actions;

        assert_eq!(actions.len(), 3);
        assert!(actions.contains(&StoryEventAction::UnlockDialogue {
            dialogue_id: "legacy_dialogue".to_string(),
        }));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn catalog_rejects_duplicate_or_invalid_actions() {
        let duplicate_root = temp_root("duplicate_actions");
        let duplicate_events = write_catalog(
            &duplicate_root,
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"duplicate_action",
                "event_type":"unlock",
                "description":"Duplicate action",
                "actions":[
                  {"type":"unlock_scene","scene_id":"same"},
                  {"type":"unlock_scene","scene_id":"same"}
                ]
              }]
            }"#,
        );
        assert!(StoryEventCatalog::load_from_directory(&duplicate_events)
            .unwrap_err()
            .contains("repeats action"));
        std::fs::remove_dir_all(duplicate_root).unwrap();

        let invalid_root = temp_root("invalid_action");
        let invalid_events = write_catalog(
            &invalid_root,
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"bad_flag",
                "event_type":"unlock",
                "description":"Bad flag",
                "actions":[{"type":"set_flag","flag":"bad flag","value":true}]
              }]
            }"#,
        );
        assert!(StoryEventCatalog::load_from_directory(&invalid_events)
            .unwrap_err()
            .contains("set_flag action"));
        std::fs::remove_dir_all(invalid_root).unwrap();
    }

    #[test]
    fn missing_directory_uses_compatibility_catalog_but_empty_directory_stays_empty() {
        let root = temp_root("compatibility");
        let missing = StoryEventCatalog::load_from_directory(&root.join("missing")).unwrap();
        assert_eq!(missing.snapshot().source, "compatibility_default");
        assert_eq!(missing.definitions().len(), 7);

        let empty = root.join("events");
        std::fs::create_dir_all(&empty).unwrap();
        let catalog = StoryEventCatalog::load_from_directory(&empty).unwrap();
        assert_eq!(catalog.snapshot().source, "project");
        assert!(catalog.definitions().is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn catalog_validates_scoped_character_references() {
        let root = temp_root("character_refs");
        let events = write_catalog(
            &root,
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"luna_only",
                "event_type":"unlock",
                "description":"Luna only",
                "character_ids":["luna"]
              }]
            }"#,
        );
        let catalog = StoryEventCatalog::load_from_directory(&events).unwrap();
        assert!(catalog.validate_character_references(["luna"]).is_ok());
        assert!(catalog
            .validate_character_references(["sakura"])
            .unwrap_err()
            .contains("unknown character `luna`"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn configured_event_directory_is_project_relative_and_enforced() {
        let root = temp_root("configured_path");
        let configured = root.join("story").join("events");
        std::fs::create_dir_all(&configured).unwrap();
        std::fs::write(
            configured.join("catalog.json"),
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{"event_id":"configured","event_type":"unlock","description":"Configured"}]
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("settings.json"),
            r#"{"paths":{"events":"story/events"}}"#,
        )
        .unwrap();

        let catalog = StoryEventCatalog::load_from_project_root(&root).unwrap();
        assert!(catalog.definition("configured", None).is_some());

        std::fs::write(
            root.join("settings.json"),
            r#"{"paths":{"events":"../outside"}}"#,
        )
        .unwrap();
        assert!(StoryEventCatalog::load_from_project_root(&root)
            .unwrap_err()
            .contains("cannot contain current or parent segments"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
