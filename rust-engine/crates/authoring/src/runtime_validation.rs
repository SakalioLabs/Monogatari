//! Headless validation of the project catalogs consumed by the core game runtime.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use llm_game::campaign::RoleplayCampaignDefinition;
use llm_game::characters::CharacterManager;
use llm_game::dialogue::DialogueManager;
use llm_game::knowledge::KnowledgeBase;
use llm_game::scene_roleplay::SceneRoleplayDefinition;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::campaign_validation::{
    load_project_roleplay_campaigns, validate_roleplay_campaign_references,
};
use crate::dialogue_validation::{normalize_dialogue_script, validate_dialogue_script};
use crate::json_catalog::{
    inspect_project_json_catalog, JsonAcceptanceLevel, JsonCatalogIssueSeverity,
};
use crate::knowledge_documents::{knowledge_base_from_documents, load_knowledge_documents};
use crate::project::inspect_project_config;
use crate::quality_suite_validation::{
    load_project_quality_suites, validate_quality_suite_references,
};
use crate::scene_roleplay_validation::{
    load_project_scene_roleplays, validate_scene_roleplay_references,
};
use crate::story_content_validation::{
    load_scene_documents, load_story_ending_sources, scene_ids, validate_ending_references,
};
use crate::story_events::StoryEventCatalog;
use crate::workflow_validation::{load_project_workflows, validate_workflow_references};

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
    pub workflow_count: usize,
    pub quality_suite_count: usize,
    pub roleplay_count: usize,
    pub campaign_count: usize,
    pub error_count: usize,
    pub issues: Vec<CoreRuntimeValidationIssue>,
}

pub struct CoreRuntimeProject {
    pub report: CoreRuntimeValidationReport,
    pub characters: CharacterManager,
    pub dialogues: DialogueManager,
    pub knowledge: KnowledgeBase,
    pub story_events: StoryEventCatalog,
    pub scene_roleplays: Vec<SceneRoleplayDefinition>,
    pub roleplay_campaigns: Vec<RoleplayCampaignDefinition>,
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
    let knowledge = load_knowledge(project_root, &knowledge_dir, &mut issues);

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
    let story_content = validate_story_content(project_root, &dialogues, &mut issues);
    validate_dialogue_documents(
        &dialogues,
        &character_ids,
        &story_content.scene_ids,
        &mut issues,
    );
    let scene_roleplay_content = validate_scene_roleplays(
        project_root,
        &story_content,
        &character_ids,
        &knowledge_ids,
        &mut issues,
    );
    let roleplay_campaigns =
        validate_roleplay_campaigns(project_root, &scene_roleplay_content.loaded, &mut issues);
    let story_events = validate_story_events(
        project_root,
        &character_ids,
        &dialogues,
        &story_content,
        &mut issues,
    );
    let workflow_content = validate_workflows(
        project_root,
        &story_events,
        &story_content.scene_ids,
        &character_ids,
        &mut issues,
    );
    let quality_suite_count = validate_quality_suites(
        project_root,
        &character_ids,
        &knowledge_ids,
        &story_events,
        &workflow_content.paths,
        &scene_roleplay_content.paths,
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
        workflow_count: workflow_content.count,
        quality_suite_count,
        roleplay_count: scene_roleplay_content.definitions.len(),
        campaign_count: roleplay_campaigns.len(),
        error_count: issues.len(),
        issues,
    };
    Ok(CoreRuntimeProject {
        report,
        characters,
        dialogues,
        knowledge,
        story_events,
        scene_roleplays: scene_roleplay_content.definitions,
        roleplay_campaigns,
    })
}

struct ValidatedWorkflowContent {
    count: usize,
    paths: HashSet<String>,
}

fn validate_workflows(
    project_root: &Path,
    story_events: &StoryEventCatalog,
    scene_ids: &HashSet<String>,
    character_ids: &HashSet<String>,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) -> ValidatedWorkflowContent {
    let workflows = match load_project_workflows(project_root, story_events) {
        Ok(workflows) => workflows,
        Err(error) => {
            issues.push(issue("workflow_catalog_invalid", Some("workflows"), error));
            return ValidatedWorkflowContent {
                count: 0,
                paths: HashSet::new(),
            };
        }
    };
    for (code, path, message) in validate_workflow_references(&workflows, scene_ids, character_ids)
    {
        issues.push(issue(code, Some(path), message));
    }
    let paths = workflows
        .iter()
        .map(|loaded| {
            loaded
                .source_path
                .strip_prefix("workflows/")
                .unwrap_or(&loaded.source_path)
                .to_string()
        })
        .collect();
    ValidatedWorkflowContent {
        count: workflows.len(),
        paths,
    }
}

fn validate_quality_suites(
    project_root: &Path,
    character_ids: &HashSet<String>,
    knowledge_ids: &HashSet<String>,
    story_events: &StoryEventCatalog,
    workflow_paths: &HashSet<String>,
    roleplay_paths: &HashSet<String>,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) -> usize {
    let suites = match load_project_quality_suites(project_root) {
        Ok(suites) => suites,
        Err(error) => {
            issues.push(issue(
                "quality_suite_catalog_invalid",
                Some("quality_suites"),
                error,
            ));
            return 0;
        }
    };
    let event_ids = story_events
        .definitions()
        .iter()
        .map(|definition| definition.event_id.clone())
        .collect();
    for (code, path, message) in validate_quality_suite_references(
        &suites,
        character_ids,
        knowledge_ids,
        &event_ids,
        workflow_paths,
        roleplay_paths,
    ) {
        issues.push(issue(code, Some(path), message));
    }
    suites.len()
}

struct ValidatedStoryContent {
    scene_ids: HashSet<String>,
    ending_ids: HashSet<String>,
}

struct ValidatedSceneRoleplayContent {
    definitions: Vec<SceneRoleplayDefinition>,
    paths: HashSet<String>,
    loaded: Vec<crate::scene_roleplay_validation::LoadedSceneRoleplay>,
}

fn validate_roleplay_campaigns(
    project_root: &Path,
    roleplays: &[crate::scene_roleplay_validation::LoadedSceneRoleplay],
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) -> Vec<RoleplayCampaignDefinition> {
    let loaded = match load_project_roleplay_campaigns(project_root) {
        Ok(loaded) => loaded,
        Err(error) => {
            issues.push(issue("campaign_catalog_invalid", Some("campaigns"), error));
            return Vec::new();
        }
    };
    for reference in validate_roleplay_campaign_references(&loaded, roleplays) {
        issues.push(issue(
            reference.code,
            Some(reference.path),
            reference.message,
        ));
    }
    loaded.into_iter().map(|loaded| loaded.definition).collect()
}

fn validate_scene_roleplays(
    project_root: &Path,
    content: &ValidatedStoryContent,
    character_ids: &HashSet<String>,
    knowledge_ids: &HashSet<String>,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) -> ValidatedSceneRoleplayContent {
    let loaded = match load_project_scene_roleplays(project_root) {
        Ok(loaded) => loaded,
        Err(error) => {
            issues.push(issue("roleplay_catalog_invalid", Some("roleplays"), error));
            return ValidatedSceneRoleplayContent {
                definitions: Vec::new(),
                paths: HashSet::new(),
                loaded: Vec::new(),
            };
        }
    };
    for reference in validate_scene_roleplay_references(
        &loaded,
        &content.scene_ids,
        character_ids,
        knowledge_ids,
        &content.ending_ids,
    ) {
        issues.push(issue(
            reference.code,
            Some(reference.path),
            reference.message,
        ));
    }
    let paths = loaded
        .iter()
        .map(|loaded| {
            loaded
                .source_path
                .strip_prefix("roleplays/")
                .unwrap_or(&loaded.source_path)
                .to_string()
        })
        .collect();
    let definitions = loaded
        .iter()
        .map(|loaded| loaded.definition.clone())
        .collect();
    ValidatedSceneRoleplayContent {
        definitions,
        paths,
        loaded,
    }
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

fn load_knowledge(
    project_root: &Path,
    directory: &Path,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) -> KnowledgeBase {
    match load_knowledge_documents(project_root, directory) {
        Ok(documents) => knowledge_base_from_documents(&documents),
        Err(error) => {
            if error.validation_issues().is_empty() {
                issues.push(CoreRuntimeValidationIssue {
                    code: error.code().to_string(),
                    path: error.path().map(str::to_string),
                    message: error.to_string(),
                });
            } else {
                for validation_issue in error.validation_issues() {
                    issues.push(CoreRuntimeValidationIssue {
                        code: validation_issue.code.clone(),
                        path: validation_issue
                            .entry_id
                            .as_deref()
                            .filter(|entry_id| !entry_id.is_empty())
                            .map(|entry_id| format!("knowledge/{entry_id}"))
                            .or_else(|| error.path().map(str::to_string)),
                        message: validation_issue.message.clone(),
                    });
                }
            }
            KnowledgeBase::new()
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

fn validate_dialogue_documents(
    dialogues: &DialogueManager,
    character_ids: &HashSet<String>,
    scene_ids: &HashSet<String>,
    issues: &mut Vec<CoreRuntimeValidationIssue>,
) {
    for dialogue in dialogues.scripts() {
        let mut source_shape = dialogue.clone();
        for node in source_shape.nodes.values_mut() {
            node.id.clear();
        }
        let dialogue = match normalize_dialogue_script(dialogue) {
            Ok(dialogue) => dialogue,
            Err(message) => {
                issues.push(issue(
                    "dialogue_normalization_invalid",
                    Some("dialogue"),
                    message,
                ));
                continue;
            }
        };
        if dialogue != source_shape {
            issues.push(issue(
                "dialogue_not_canonical",
                Some(format!("dialogue/{}", dialogue.id)),
                format!(
                    "Dialogue `{}` contains non-canonical authoring fields; normalize it before acceptance.",
                    dialogue.id
                ),
            ));
        }
        let validation = validate_dialogue_script(&dialogue, character_ids);
        for validation_issue in validation.issues {
            let path = validation_issue
                .node_id
                .as_deref()
                .map(|node_id| format!("dialogue/{}/{node_id}", dialogue.id))
                .unwrap_or_else(|| format!("dialogue/{}", dialogue.id));
            issues.push(issue(
                validation_issue.code,
                Some(path),
                validation_issue.message,
            ));
        }
        for (node_id, node) in &dialogue.nodes {
            if let Some(scene_id) = node.scene_id.as_deref() {
                if !scene_ids.contains(scene_id) {
                    issues.push(issue(
                        "dialogue_scene_target_missing",
                        Some(format!("dialogue/{}/{node_id}", dialogue.id)),
                        format!(
                            "Dialogue `{}` node `{node_id}` references unknown scene `{scene_id}`.",
                            dialogue.id
                        ),
                    ));
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
