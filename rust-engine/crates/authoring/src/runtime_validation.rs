//! Headless validation of the project catalogs consumed by the core game runtime.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use llm_game::characters::CharacterManager;
use llm_game::dialogue::DialogueManager;
use llm_game::knowledge::KnowledgeBase;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::json_catalog::{
    inspect_project_json_catalog, JsonAcceptanceLevel, JsonCatalogIssueSeverity,
};
use crate::project::inspect_project_config;
use crate::story_content_validation::{
    load_scene_documents, load_story_ending_sources, scene_ids, validate_ending_references,
};
use crate::story_events::StoryEventCatalog;

pub const CORE_RUNTIME_VALIDATION_SCHEMA_V1: &str = "monogatari-core-runtime-validation/v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CoreRuntimeValidationIssue {
    pub code: String,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CoreRuntimeValidationReport {
    pub schema: String,
    pub acceptance_level: JsonAcceptanceLevel,
    pub valid: bool,
    pub document_count: usize,
    pub total_bytes: u64,
    pub warning_count: usize,
    pub character_count: usize,
    pub dialogue_count: usize,
    pub dialogue_node_count: usize,
    pub knowledge_count: usize,
    pub scene_count: usize,
    pub ending_count: usize,
    pub story_event_count: usize,
    pub error_count: usize,
    pub issues: Vec<CoreRuntimeValidationIssue>,
}

pub struct CoreRuntimeProject {
    pub report: CoreRuntimeValidationReport,
    pub characters: CharacterManager,
    pub dialogues: DialogueManager,
    pub knowledge: KnowledgeBase,
    pub story_events: StoryEventCatalog,
}

/// Validate document safety, real core runtime loading, and core cross-catalog references.
pub async fn validate_core_runtime_project(
    project_root: &Path,
) -> Result<CoreRuntimeValidationReport, String> {
    Ok(load_core_runtime_project(project_root).await?.report)
}

/// Load the core managers and retain the validation report for transport adapters.
pub async fn load_core_runtime_project(project_root: &Path) -> Result<CoreRuntimeProject, String> {
    let project = inspect_project_config(project_root)?;
    let documents =
        inspect_project_json_catalog(project_root, None).map_err(|error| error.to_string())?;
    let mut issues = Vec::new();
    if !project.settings_exists || !project.valid {
        issues.push(issue(
            "project_settings_invalid",
            Some("settings.json"),
            "Project settings are not ready for runtime loading.",
        ));
    }
    for document_issue in documents
        .issues
        .iter()
        .filter(|issue| issue.severity == JsonCatalogIssueSeverity::Error)
    {
        issues.push(CoreRuntimeValidationIssue {
            code: format!(
                "document_{}",
                serde_json::to_value(document_issue.code)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_string))
                    .unwrap_or_else(|| "catalog_error".to_string())
            ),
            path: document_issue.path.clone(),
            message: document_issue.message.clone(),
        });
    }

    let characters_dir = runtime_path(&project.paths, "characters", project_root);
    let dialogue_dir = runtime_path(&project.paths, "dialogue", project_root);
    let knowledge_dir = runtime_path(&project.paths, "knowledge", project_root);

    let mut characters = CharacterManager::new();
    load_characters(&mut characters, &characters_dir, &mut issues).await;
    let mut dialogues = DialogueManager::new();
    load_dialogues(&mut dialogues, &dialogue_dir, &mut issues).await;
    let mut knowledge = KnowledgeBase::new();
    load_knowledge(&mut knowledge, &knowledge_dir, &mut issues).await;

    let character_ids = characters
        .character_ids()
        .into_iter()
        .collect::<HashSet<_>>();
    let knowledge_ids = knowledge
        .all_entries()
        .into_iter()
        .map(|entry| entry.id.clone())
        .collect::<HashSet<_>>();
    validate_character_references(&characters, &character_ids, &knowledge_ids, &mut issues).await;
    validate_dialogue_references(&dialogues, &character_ids, &mut issues);

    let story_content = validate_story_content(project_root, &dialogues, &mut issues);
    let story_events = validate_story_events(
        project_root,
        &character_ids,
        &dialogues,
        &story_content,
        &mut issues,
    );

    issues.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.message.cmp(&right.message))
    });
    let dialogue_node_count = dialogues
        .scripts()
        .iter()
        .map(|dialogue| dialogue.nodes.len())
        .sum();
    let report = CoreRuntimeValidationReport {
        schema: CORE_RUNTIME_VALIDATION_SCHEMA_V1.to_string(),
        acceptance_level: JsonAcceptanceLevel::CoreRuntime,
        valid: issues.is_empty(),
        document_count: documents.document_count,
        total_bytes: documents.total_bytes,
        warning_count: documents.warning_count + project.warning_count,
        character_count: character_ids.len(),
        dialogue_count: dialogues.script_ids().len(),
        dialogue_node_count,
        knowledge_count: knowledge_ids.len(),
        scene_count: story_content.scene_ids.len(),
        ending_count: story_content.ending_ids.len(),
        story_event_count: story_events.definitions().len(),
        error_count: issues.len(),
        issues,
    };
    Ok(CoreRuntimeProject {
        report,
        characters,
        dialogues,
        knowledge,
        story_events,
    })
}

struct ValidatedStoryContent {
    scene_ids: HashSet<String>,
    ending_ids: HashSet<String>,
}

fn validate_story_content(
    project_root: &Path,
    dialogues: &DialogueManager,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) -> ValidatedStoryContent {
    let scenes = match load_scene_documents(project_root) {
        Ok(scenes) => scenes,
        Err(error) => {
            issues.push(issue("scene_catalog_invalid", Some("scenes"), error));
            return ValidatedStoryContent {
                scene_ids: HashSet::new(),
                ending_ids: HashSet::new(),
            };
        }
    };
    let scene_ids = match scene_ids(project_root, &scenes) {
        Ok(ids) => ids,
        Err(error) => {
            issues.push(issue("scene_assets_invalid", Some("assets"), error));
            return ValidatedStoryContent {
                scene_ids: scenes.into_iter().map(|loaded| loaded.scene.id).collect(),
                ending_ids: HashSet::new(),
            };
        }
    };
    let endings = match load_story_ending_sources(project_root) {
        Ok(endings) => endings,
        Err(error) => {
            issues.push(issue("ending_catalog_invalid", Some("endings"), error));
            return ValidatedStoryContent {
                scene_ids,
                ending_ids: HashSet::new(),
            };
        }
    };
    for (code, message) in validate_ending_references(&endings, &scene_ids, dialogues) {
        issues.push(issue(code, Some("endings"), message));
    }
    ValidatedStoryContent {
        scene_ids,
        ending_ids: endings.into_iter().map(|loaded| loaded.ending.id).collect(),
    }
}

fn validate_story_events(
    project_root: &Path,
    character_ids: &HashSet<String>,
    dialogues: &DialogueManager,
    content: &ValidatedStoryContent,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) -> StoryEventCatalog {
    let catalog = match StoryEventCatalog::load_from_project_root(project_root) {
        Ok(catalog) => catalog,
        Err(error) => {
            issues.push(issue("story_event_catalog_invalid", Some("events"), error));
            return StoryEventCatalog::from_document_json(
                r#"{"schema":"monogatari-story-event-catalog/v1","events":[]}"#,
                "events/<invalid>",
            )
            .expect("empty Story Event catalog is valid");
        }
    };
    if let Err(error) =
        catalog.validate_character_references(character_ids.iter().map(String::as_str))
    {
        issues.push(issue(
            "story_event_character_missing",
            Some("events"),
            error,
        ));
    }
    let dialogue_ids = dialogues.script_ids().into_iter().collect::<HashSet<_>>();
    if let Err(error) =
        catalog.validate_content_references(&content.scene_ids, &dialogue_ids, &content.ending_ids)
    {
        issues.push(issue("story_event_content_missing", Some("events"), error));
    }
    catalog
}

fn runtime_path(
    paths: &[crate::project::ProjectPathStatus],
    key: &str,
    project_root: &Path,
) -> PathBuf {
    paths
        .iter()
        .find(|path| path.key == key)
        .map(|path| project_root.join(&path.relative_path))
        .unwrap_or_else(|| project_root.join(key))
}

async fn load_characters(
    manager: &mut CharacterManager,
    directory: &Path,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) {
    match manager.load_from_directory(directory).await {
        Ok(_) => {}
        Err(error) => {
            issues.push(issue(
                if error.to_string().contains("Duplicate character id") {
                    "duplicate_character_id"
                } else {
                    "character_runtime_load_failed"
                },
                Some("characters"),
                format!("Character runtime loading failed: {error}"),
            ));
        }
    }
}

async fn load_dialogues(
    manager: &mut DialogueManager,
    directory: &Path,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) {
    match manager.load_from_directory(directory).await {
        Ok(_) => {}
        Err(error) => {
            issues.push(issue(
                if error.to_string().contains("Duplicate dialogue id") {
                    "duplicate_dialogue_id"
                } else {
                    "dialogue_runtime_load_failed"
                },
                Some("dialogue"),
                format!("Dialogue runtime loading failed: {error}"),
            ));
        }
    }
}

async fn load_knowledge(
    knowledge: &mut KnowledgeBase,
    directory: &Path,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) {
    match knowledge.load_from_directory(directory).await {
        Ok(_) => {}
        Err(error) => {
            issues.push(issue(
                if error.to_string().contains("Duplicate knowledge id") {
                    "duplicate_knowledge_id"
                } else {
                    "knowledge_runtime_load_failed"
                },
                Some("knowledge"),
                format!("Knowledge runtime loading failed: {error}"),
            ));
        }
    }
}

async fn validate_character_references(
    characters: &CharacterManager,
    character_ids: &HashSet<String>,
    knowledge_ids: &HashSet<String>,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) {
    let mut ids = character_ids.iter().cloned().collect::<Vec<_>>();
    ids.sort();
    for character_id in ids {
        let Some(character) = characters.get_character(&character_id) else {
            continue;
        };
        let character = character.read().await;
        if !portable_id(&character.id) {
            issues.push(issue(
                "character_id_invalid",
                Some(format!("characters/{}", character.id)),
                format!("Character ID `{}` is not portable.", character.id),
            ));
        }
        for target_id in character.relationships.keys() {
            if target_id != "player" && !character_ids.contains(target_id) {
                issues.push(issue(
                    "character_relationship_target_missing",
                    Some(format!("characters/{}", character.id)),
                    format!(
                        "Character `{}` relates to unknown character `{target_id}`.",
                        character.id
                    ),
                ));
            }
        }
        for knowledge_id in &character.knowledge_refs {
            if !knowledge_ids.contains(knowledge_id) {
                issues.push(issue(
                    "character_knowledge_target_missing",
                    Some(format!("characters/{}", character.id)),
                    format!(
                        "Character `{}` pins unknown knowledge `{knowledge_id}`.",
                        character.id
                    ),
                ));
            }
        }
    }
}

fn validate_dialogue_references(
    dialogues: &DialogueManager,
    character_ids: &HashSet<String>,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) {
    for dialogue in dialogues.scripts() {
        for (node_id, node) in &dialogue.nodes {
            if let Some(speaker_id) = node.speaker_id.as_deref() {
                if !character_ids.contains(speaker_id) {
                    issues.push(issue(
                        "dialogue_speaker_missing",
                        Some(format!("dialogue/{}/{}", dialogue.id, node_id)),
                        format!(
                            "Dialogue `{}` references unknown speaker `{speaker_id}`.",
                            dialogue.id
                        ),
                    ));
                }
            }
            for (choice_index, choice) in node.choices.iter().enumerate() {
                for character_id in choice.relationship_changes.keys() {
                    if !character_ids.contains(character_id) {
                        issues.push(issue(
                            "dialogue_relationship_target_missing",
                            Some(format!("dialogue/{}/{}", dialogue.id, node_id)),
                            format!(
                                "Dialogue `{}` node `{node_id}` choice {} changes unknown character `{character_id}`.",
                                dialogue.id,
                                choice_index + 1
                            ),
                        ));
                    }
                }
            }
        }
    }
}

fn portable_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.trim() == value
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn issue(
    code: impl Into<String>,
    path: Option<impl Into<String>>,
    message: impl Into<String>,
) -> CoreRuntimeValidationIssue {
    CoreRuntimeValidationIssue {
        code: code.into(),
        path: path.map(Into::into),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests;
