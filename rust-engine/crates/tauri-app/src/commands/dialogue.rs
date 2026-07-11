//! Dialogue runtime and project authoring commands.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use llm_authoring::filesystem::{
    ensure_regular_project_directory, sha256_json, source_label, stage_json_deletion,
    stage_json_replacement,
};
use llm_core::normalize_script_state_map;
use llm_game::characters::{Character, CharacterManager};
use llm_game::dialogue::{
    DialogueChoiceEffects, DialogueManager, DialogueScript, DialogueScriptSummary,
};
use llm_scripting::{validate_condition_source, validate_script_source};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;
use tokio::sync::RwLock;

use crate::commands::characters::ensure_project_characters_loaded;
use crate::commands::content_paths::resolve_project_content_dir;
use crate::content_references::dialogue_references;
use crate::state::AppState;
use crate::story_access::{
    ensure_story_content_access, story_content_access, StoryContentAccessEntry, StoryContentKind,
};

const DIALOGUE_AUTHORING_CATALOG_SCHEMA_V1: &str = "monogatari-dialogue-authoring-catalog/v1";
const MAX_DIALOGUE_FILES: usize = 512;
const MAX_DIALOGUE_FILE_BYTES: u64 = 1024 * 1024;
const MAX_DIALOGUE_NODES: usize = 2048;
const MAX_DIALOGUE_CHOICES_PER_NODE: usize = 32;
const MAX_RELATIONSHIP_CHANGES_PER_CHOICE: usize = 128;
const MAX_DIALOGUE_VARIABLES: usize = 512;
const MAX_DIALOGUE_TEXT_CHARS: usize = 16_384;
const MAX_DIALOGUE_PROMPT_CHARS: usize = 20_000;

#[derive(Serialize)]
pub struct DialogueCatalogEntry {
    #[serde(flatten)]
    pub dialogue: DialogueScriptSummary,
    pub access: StoryContentAccessEntry,
}

#[derive(Debug, Clone, Serialize)]
pub struct DialogueState {
    pub is_active: bool,
    pub speaker: Option<String>,
    pub text: String,
    pub emotion: Option<String>,
    pub choices: Vec<ChoiceInfo>,
    pub live2d_expression: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceInfo {
    pub index: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DialogueAuthoringEntry {
    #[serde(flatten)]
    pub dialogue: DialogueScript,
    pub source_path: String,
    pub content_fingerprint: String,
    pub access: StoryContentAccessEntry,
}

#[derive(Debug, Clone, Serialize)]
pub struct DialogueAuthoringCatalogSnapshot {
    pub schema: String,
    pub catalog_fingerprint: String,
    pub dialogue_count: usize,
    pub node_count: usize,
    pub choice_count: usize,
    pub llm_node_count: usize,
    pub dialogues: Vec<DialogueAuthoringEntry>,
}

#[derive(Debug, Clone)]
struct LoadedDialogueScript {
    dialogue: DialogueScript,
    source_path: String,
    absolute_path: PathBuf,
}

/// Start a dialogue by script ID.
#[tauri::command]
pub async fn start_dialogue(
    state: State<'_, AppState>,
    dialogue_id: String,
) -> Result<DialogueState, String> {
    start_dialogue_inner(&state, &dialogue_id).await
}

/// Preview a validated dialogue from authoring without applying player unlock gates.
#[tauri::command]
pub async fn preview_dialogue(
    state: State<'_, AppState>,
    dialogue_id: String,
) -> Result<DialogueState, String> {
    start_dialogue_authoring_inner(&state, &dialogue_id).await
}

pub(crate) async fn start_dialogue_inner(
    state: &AppState,
    dialogue_id: &str,
) -> Result<DialogueState, String> {
    ensure_dialogue_access(state, dialogue_id).await?;
    start_dialogue_authoring_inner(state, dialogue_id).await
}

/// Start a validated project dialogue for author preview without applying player unlock gates.
pub(crate) async fn start_dialogue_authoring_inner(
    state: &AppState,
    dialogue_id: &str,
) -> Result<DialogueState, String> {
    ensure_project_dialogues_loaded(state).await?;
    let mut dm = state.dialogue_manager.write().await;
    dm.start_dialogue(dialogue_id)
        .await
        .map_err(|e| e.to_string())?;
    get_dialogue_state_inner(&dm)
}

/// List loaded project dialogues together with runtime access decisions.
#[tauri::command]
pub async fn list_dialogues(
    state: State<'_, AppState>,
) -> Result<Vec<DialogueCatalogEntry>, String> {
    ensure_project_dialogues_loaded(&state).await?;
    let summaries = state.dialogue_manager.read().await.script_summaries();
    let catalog = state.story_event_catalog.read().await;
    let progress = state.story_progress.read().await;
    Ok(summaries
        .into_iter()
        .map(|dialogue| DialogueCatalogEntry {
            access: story_content_access(
                &catalog,
                &progress,
                StoryContentKind::Dialogue,
                &dialogue.id,
            ),
            dialogue,
        })
        .collect())
}

/// Return complete editable dialogue documents with a stable catalog fingerprint.
#[tauri::command]
pub async fn get_dialogue_authoring_catalog(
    state: State<'_, AppState>,
) -> Result<DialogueAuthoringCatalogSnapshot, String> {
    dialogue_authoring_catalog_snapshot(&state).await
}

/// Atomically create or update one dialogue and hot-reload the runtime catalog.
#[tauri::command]
pub async fn save_dialogue_definition(
    state: State<'_, AppState>,
    dialogue: DialogueScript,
    original_dialogue_id: Option<String>,
    expected_catalog_fingerprint: String,
) -> Result<DialogueAuthoringCatalogSnapshot, String> {
    save_dialogue_definition_inner(
        &state,
        dialogue,
        original_dialogue_id.as_deref(),
        &expected_catalog_fingerprint,
    )
    .await
}

/// Delete a dialogue after checking event and ending references, then hot-reload runtime state.
#[tauri::command]
pub async fn delete_dialogue_definition(
    state: State<'_, AppState>,
    dialogue_id: String,
    expected_catalog_fingerprint: String,
) -> Result<DialogueAuthoringCatalogSnapshot, String> {
    delete_dialogue_definition_inner(&state, &dialogue_id, &expected_catalog_fingerprint).await
}

/// Advance to the next dialogue node.
#[tauri::command]
pub async fn advance_dialogue(state: State<'_, AppState>) -> Result<DialogueState, String> {
    let mut dm = state.dialogue_manager.write().await;
    dm.advance().await.map_err(|e| e.to_string())?;
    get_dialogue_state_inner(&dm)
}

/// Select a dialogue choice by index.
#[tauri::command]
pub async fn select_choice(
    state: State<'_, AppState>,
    choice_index: usize,
) -> Result<DialogueState, String> {
    select_choice_inner(&state, choice_index).await
}

async fn select_choice_inner(
    state: &AppState,
    choice_index: usize,
) -> Result<DialogueState, String> {
    let effects = state
        .dialogue_manager
        .read()
        .await
        .choice_effects(choice_index)
        .map_err(|error| error.to_string())?;
    let targets = resolve_dialogue_choice_relationship_targets(state, &effects).await?;
    {
        let mut dialogue_manager = state.dialogue_manager.write().await;
        dialogue_manager
            .select_choice_from(&effects.source_node_id, choice_index)
            .await
            .map_err(|error| error.to_string())?;
    };
    apply_dialogue_choice_relationship_targets(targets).await;
    let dialogue_manager = state.dialogue_manager.read().await;
    get_dialogue_state_inner(&dialogue_manager)
}

async fn resolve_dialogue_choice_relationship_targets(
    state: &AppState,
    effects: &DialogueChoiceEffects,
) -> Result<Vec<(Arc<RwLock<Character>>, f32)>, String> {
    if effects.relationship_changes.is_empty() {
        return Ok(Vec::new());
    }

    ensure_project_characters_loaded(state).await?;
    let mut changes = effects
        .relationship_changes
        .iter()
        .map(|(character_id, delta)| (character_id.clone(), *delta))
        .collect::<Vec<_>>();
    changes.sort_by(|left, right| left.0.cmp(&right.0));

    {
        let manager = state.character_manager.read().await;
        changes
            .into_iter()
            .map(
                |(character_id, delta)| match manager.get_character(&character_id) {
                    Some(character) => Ok((character, delta)),
                    None => Err(format!(
                        "Dialogue choice {} changes unknown character `{character_id}`.",
                        effects.choice_index + 1
                    )),
                },
            )
            .collect::<Result<Vec<_>, String>>()
    }
}

async fn apply_dialogue_choice_relationship_targets(targets: Vec<(Arc<RwLock<Character>>, f32)>) {
    for (character, delta) in targets {
        character.write().await.update_relationship("player", delta);
    }
}

/// Get the current dialogue state.
#[tauri::command]
pub async fn get_dialogue_state(state: State<'_, AppState>) -> Result<DialogueState, String> {
    let dm = state.dialogue_manager.read().await;
    get_dialogue_state_inner(&dm)
}

/// Load dialogue scripts from a project-contained directory.
#[tauri::command]
pub async fn load_dialogues(
    state: State<'_, AppState>,
    directory: String,
) -> Result<usize, String> {
    let path = resolve_project_content_dir(&state, &directory, "dialogue").await?;
    let mut dm = state.dialogue_manager.write().await;
    dm.load_from_directory(&path)
        .await
        .map_err(|e| e.to_string())
}

async fn dialogue_authoring_catalog_snapshot(
    state: &AppState,
) -> Result<DialogueAuthoringCatalogSnapshot, String> {
    let project_root = state.current_project_data_root().await;
    let loaded = load_dialogue_authoring_state(&project_root).await?;
    dialogue_authoring_snapshot_from_loaded(state, loaded).await
}

async fn dialogue_authoring_snapshot_from_loaded(
    state: &AppState,
    loaded: Vec<LoadedDialogueScript>,
) -> Result<DialogueAuthoringCatalogSnapshot, String> {
    let catalog_fingerprint = dialogue_authoring_catalog_fingerprint(&loaded);
    let event_catalog = state.story_event_catalog.read().await;
    let progress = state.story_progress.read().await;
    let mut node_count = 0;
    let mut choice_count = 0;
    let mut llm_node_count = 0;
    let dialogues = loaded
        .into_iter()
        .map(|loaded| {
            node_count += loaded.dialogue.nodes.len();
            choice_count += loaded
                .dialogue
                .nodes
                .values()
                .map(|node| node.choices.len())
                .sum::<usize>();
            llm_node_count += loaded
                .dialogue
                .nodes
                .values()
                .filter(|node| node.use_llm)
                .count();
            DialogueAuthoringEntry {
                content_fingerprint: dialogue_content_fingerprint(&loaded.dialogue),
                access: story_content_access(
                    &event_catalog,
                    &progress,
                    StoryContentKind::Dialogue,
                    &loaded.dialogue.id,
                ),
                source_path: loaded.source_path,
                dialogue: loaded.dialogue,
            }
        })
        .collect::<Vec<_>>();
    Ok(DialogueAuthoringCatalogSnapshot {
        schema: DIALOGUE_AUTHORING_CATALOG_SCHEMA_V1.to_string(),
        catalog_fingerprint,
        dialogue_count: dialogues.len(),
        node_count,
        choice_count,
        llm_node_count,
        dialogues,
    })
}

async fn save_dialogue_definition_inner(
    state: &AppState,
    dialogue: DialogueScript,
    original_dialogue_id: Option<&str>,
    expected_catalog_fingerprint: &str,
) -> Result<DialogueAuthoringCatalogSnapshot, String> {
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    let project_root = state.current_project_data_root().await;
    let character_ids = load_project_character_ids(&project_root).await?;
    let dialogue = normalize_dialogue_script(dialogue)?;
    validate_dialogue_script(&dialogue, &character_ids)?;
    let current = load_dialogue_documents(&project_root, &character_ids)?;
    ensure_dialogue_catalog_fingerprint(&current, expected_catalog_fingerprint)?;
    let dialogue_root =
        ensure_regular_project_directory(&project_root, "dialogue", "dialogue").await?;
    let target_path = match original_dialogue_id {
        Some(original_id) => {
            if original_id != dialogue.id {
                return Err(
                    "Dialogue ids are immutable after creation; duplicate the dialogue to use a new id."
                        .to_string(),
                );
            }
            current
                .iter()
                .find(|loaded| loaded.dialogue.id == original_id)
                .map(|loaded| loaded.absolute_path.clone())
                .ok_or_else(|| {
                    format!("Dialogue `{original_id}` no longer exists; reload before saving.")
                })?
        }
        None => {
            if current
                .iter()
                .any(|loaded| loaded.dialogue.id == dialogue.id)
            {
                return Err(format!(
                    "Dialogue `{}` already exists; reload it before editing.",
                    dialogue.id
                ));
            }
            dialogue_root.join(format!("{}.json", dialogue.id))
        }
    };

    let canonical = serde_json::to_value(&dialogue)
        .map_err(|error| format!("Unable to serialize dialogue: {error}"))?;
    let mut content = serde_json::to_string_pretty(&canonical)
        .map_err(|error| format!("Unable to serialize dialogue: {error}"))?;
    content.push('\n');
    let staged = stage_json_replacement(
        &target_path,
        content.as_bytes(),
        MAX_DIALOGUE_FILE_BYTES,
        "dialogue",
    )
    .await?;

    let loaded = match load_dialogue_documents(&project_root, &character_ids) {
        Ok(loaded) => loaded,
        Err(error) => {
            staged.rollback().await?;
            return Err(format!(
                "Saved dialogue failed project reload and was rolled back: {error}"
            ));
        }
    };
    if !loaded.iter().any(|loaded| loaded.dialogue == dialogue) {
        staged.rollback().await?;
        return Err(
            "Saved dialogue changed during replacement; the original was restored.".to_string(),
        );
    }
    let runtime_scripts = loaded
        .iter()
        .map(|loaded| loaded.dialogue.clone())
        .collect::<Vec<_>>();
    staged.commit().await?;
    state
        .dialogue_manager
        .write()
        .await
        .replace_scripts(runtime_scripts)
        .map_err(|error| format!("Saved dialogue could not hot-reload: {error}"))?;
    dialogue_authoring_snapshot_from_loaded(state, loaded).await
}

async fn delete_dialogue_definition_inner(
    state: &AppState,
    dialogue_id: &str,
    expected_catalog_fingerprint: &str,
) -> Result<DialogueAuthoringCatalogSnapshot, String> {
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    let project_root = state.current_project_data_root().await;
    let character_ids = load_project_character_ids(&project_root).await?;
    let current = load_dialogue_documents(&project_root, &character_ids)?;
    ensure_dialogue_catalog_fingerprint(&current, expected_catalog_fingerprint)?;
    let target = current
        .iter()
        .find(|loaded| loaded.dialogue.id == dialogue_id)
        .ok_or_else(|| format!("Dialogue `{dialogue_id}` does not exist."))?;
    let references = dialogue_references(&project_root, dialogue_id)?;
    if !references.is_empty() {
        return Err(format!(
            "Dialogue `{dialogue_id}` is still referenced by: {}. Remove those references before deleting it.",
            references.join(", ")
        ));
    }

    let staged = stage_json_deletion(&target.absolute_path, "dialogue").await?;
    let loaded = match load_dialogue_documents(&project_root, &character_ids) {
        Ok(loaded)
            if !loaded
                .iter()
                .any(|loaded| loaded.dialogue.id == dialogue_id) =>
        {
            loaded
        }
        Ok(_) => {
            staged.rollback().await?;
            return Err(
                "Deleted dialogue remained in the authored catalog; the file was restored."
                    .to_string(),
            );
        }
        Err(error) => {
            staged.rollback().await?;
            return Err(format!(
                "Deleting dialogue broke the project catalog and was rolled back: {error}"
            ));
        }
    };
    let runtime_scripts = loaded
        .iter()
        .map(|loaded| loaded.dialogue.clone())
        .collect::<Vec<_>>();
    staged.commit().await?;
    state
        .dialogue_manager
        .write()
        .await
        .replace_scripts(runtime_scripts)
        .map_err(|error| format!("Dialogue deletion could not hot-reload: {error}"))?;
    dialogue_authoring_snapshot_from_loaded(state, loaded).await
}

async fn load_dialogue_authoring_state(
    project_root: &Path,
) -> Result<Vec<LoadedDialogueScript>, String> {
    let character_ids = load_project_character_ids(project_root).await?;
    load_dialogue_documents(project_root, &character_ids)
}

async fn load_project_character_ids(project_root: &Path) -> Result<HashSet<String>, String> {
    let character_root = project_root.join("characters");
    if !character_root.is_dir() {
        return Ok(HashSet::new());
    }
    let mut manager = CharacterManager::new();
    manager
        .load_from_directory(&character_root)
        .await
        .map_err(|error| format!("Failed to load project characters: {error}"))?;
    Ok(manager.character_ids().into_iter().collect())
}

fn load_dialogue_documents(
    project_root: &Path,
    character_ids: &HashSet<String>,
) -> Result<Vec<LoadedDialogueScript>, String> {
    let dialogue_root = project_root.join("dialogue");
    if !dialogue_root.exists() {
        return Ok(Vec::new());
    }
    let root_metadata = std::fs::symlink_metadata(&dialogue_root).map_err(|error| {
        format!(
            "Failed to inspect dialogue directory `{}`: {error}",
            dialogue_root.display()
        )
    })?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(format!(
            "Dialogue path must be a regular directory: {}",
            dialogue_root.display()
        ));
    }
    let canonical_root = dialogue_root.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve dialogue directory `{}`: {error}",
            dialogue_root.display()
        )
    })?;
    let mut files = Vec::new();
    for entry in std::fs::read_dir(&dialogue_root).map_err(|error| {
        format!(
            "Failed to read dialogue directory `{}`: {error}",
            dialogue_root.display()
        )
    })? {
        let path = entry
            .map_err(|error| format!("Failed to read dialogue directory entry: {error}"))?
            .path();
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        {
            files.push(path);
        }
    }
    files.sort();
    if files.len() > MAX_DIALOGUE_FILES {
        return Err(format!(
            "Dialogue directory contains {} JSON files; the limit is {MAX_DIALOGUE_FILES}.",
            files.len()
        ));
    }

    let mut seen = HashSet::new();
    let mut dialogues = Vec::with_capacity(files.len());
    for path in files {
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| format!("Failed to inspect dialogue `{}`: {error}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Dialogue must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_DIALOGUE_FILE_BYTES {
            return Err(format!(
                "Dialogue `{}` is {} bytes; the limit is {MAX_DIALOGUE_FILE_BYTES} bytes.",
                path.display(),
                metadata.len()
            ));
        }
        let canonical_path = path
            .canonicalize()
            .map_err(|error| format!("Failed to resolve dialogue `{}`: {error}", path.display()))?;
        if !canonical_path.starts_with(&canonical_root) {
            return Err(format!(
                "Dialogue escapes the project dialogue directory: {}",
                path.display()
            ));
        }
        let content = std::fs::read_to_string(&canonical_path)
            .map_err(|error| format!("Failed to read dialogue `{}`: {error}", path.display()))?;
        let dialogue: DialogueScript = serde_json::from_str(&content)
            .map_err(|error| format!("Invalid dialogue JSON in `{}`: {error}", path.display()))?;
        let dialogue = normalize_dialogue_script(dialogue)
            .map_err(|error| format!("Invalid dialogue `{}`: {error}", path.display()))?;
        validate_dialogue_script(&dialogue, character_ids)
            .map_err(|error| format!("Invalid dialogue `{}`: {error}", path.display()))?;
        if !seen.insert(dialogue.id.clone()) {
            return Err(format!("Duplicate dialogue id `{}`.", dialogue.id));
        }
        dialogues.push(LoadedDialogueScript {
            dialogue,
            source_path: source_label(project_root, &path),
            absolute_path: canonical_path,
        });
    }
    dialogues.sort_by(|left, right| left.dialogue.id.cmp(&right.dialogue.id));
    Ok(dialogues)
}

fn normalize_dialogue_script(mut dialogue: DialogueScript) -> Result<DialogueScript, String> {
    dialogue.id = dialogue.id.trim().to_string();
    dialogue.title = dialogue.title.trim().to_string();
    dialogue.description = normalize_optional(dialogue.description);
    dialogue.start_node_id = dialogue.start_node_id.trim().to_string();
    dialogue.variables = normalize_script_state_map(dialogue.variables)
        .map_err(|error| format!("Dialogue variables are invalid: {error}"))?;
    for (node_id, node) in &mut dialogue.nodes {
        if !node.id.is_empty() && node.id.trim() != node_id {
            return Err(format!(
                "Embedded node id `{}` does not match map key `{node_id}`.",
                node.id
            ));
        }
        node.id.clear();
        node.speaker_id = normalize_optional(node.speaker_id.take());
        node.text = node.text.trim().to_string();
        node.next_node_id = normalize_optional(node.next_node_id.take());
        node.condition = normalize_optional(node.condition.take());
        node.script = normalize_optional(node.script.take());
        node.emotion = normalize_optional(node.emotion.take());
        node.llm_prompt = normalize_optional(node.llm_prompt.take());
        node.llm_system_prompt = normalize_optional(node.llm_system_prompt.take());
        node.ending_type = normalize_optional(node.ending_type.take());
        for choice in &mut node.choices {
            choice.text = choice.text.trim().to_string();
            choice.next_node_id = choice.next_node_id.trim().to_string();
            choice.condition = normalize_optional(choice.condition.take());
            let mut normalized_changes = HashMap::with_capacity(choice.relationship_changes.len());
            for (character_id, delta) in std::mem::take(&mut choice.relationship_changes) {
                let character_id = character_id.trim().to_string();
                if normalized_changes
                    .insert(character_id.clone(), delta)
                    .is_some()
                {
                    return Err(format!(
                        "Choice has duplicate relationship target `{character_id}` after normalization."
                    ));
                }
            }
            choice.relationship_changes = normalized_changes;
        }
    }
    Ok(dialogue)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn validate_dialogue_script(
    dialogue: &DialogueScript,
    character_ids: &HashSet<String>,
) -> Result<(), String> {
    dialogue
        .validate_graph()
        .map_err(|error| error.to_string())?;
    validate_dialogue_text(&dialogue.title, "title", 1, 256, &dialogue.id, None)?;
    if let Some(description) = dialogue.description.as_deref() {
        validate_dialogue_text(description, "description", 1, 2_048, &dialogue.id, None)?;
    }
    if dialogue.nodes.is_empty() || dialogue.nodes.len() > MAX_DIALOGUE_NODES {
        return Err(format!(
            "Dialogue `{}` must contain 1 to {MAX_DIALOGUE_NODES} nodes.",
            dialogue.id
        ));
    }
    if dialogue.variables.len() > MAX_DIALOGUE_VARIABLES {
        return Err(format!(
            "Dialogue `{}` has too many variables; the limit is {MAX_DIALOGUE_VARIABLES}.",
            dialogue.id
        ));
    }
    let variable_bytes = serde_json::to_vec(&dialogue.variables)
        .map_err(|error| format!("Dialogue variables cannot be serialized: {error}"))?
        .len();
    if variable_bytes > MAX_DIALOGUE_FILE_BYTES as usize / 2 {
        return Err(format!(
            "Dialogue `{}` variables exceed the catalog size budget.",
            dialogue.id
        ));
    }

    for (node_id, node) in &dialogue.nodes {
        validate_dialogue_text(
            &node.text,
            "text",
            1,
            MAX_DIALOGUE_TEXT_CHARS,
            &dialogue.id,
            Some(node_id),
        )?;
        if node.choices.len() > MAX_DIALOGUE_CHOICES_PER_NODE {
            return Err(format!(
                "Dialogue `{}` node `{node_id}` has too many choices; the limit is {MAX_DIALOGUE_CHOICES_PER_NODE}.",
                dialogue.id
            ));
        }
        if let Some(speaker_id) = node.speaker_id.as_deref() {
            if !character_ids.contains(speaker_id) {
                return Err(format!(
                    "Dialogue `{}` node `{node_id}` references unknown speaker `{speaker_id}`.",
                    dialogue.id
                ));
            }
        }
        if let Some(emotion) = node.emotion.as_deref() {
            validate_dialogue_text(emotion, "emotion", 1, 64, &dialogue.id, Some(node_id))?;
        }
        if let Some(ending_type) = node.ending_type.as_deref() {
            validate_dialogue_text(
                ending_type,
                "ending type",
                1,
                64,
                &dialogue.id,
                Some(node_id),
            )?;
        }
        if let Some(condition) = node.condition.as_deref() {
            validate_condition_source(condition).map_err(|error| {
                format!(
                    "Dialogue `{}` node `{node_id}` condition is invalid: {error}",
                    dialogue.id
                )
            })?;
        }
        if let Some(script) = node.script.as_deref() {
            validate_script_source(script).map_err(|error| {
                format!(
                    "Dialogue `{}` node `{node_id}` script is invalid: {error}",
                    dialogue.id
                )
            })?;
        }
        if node.use_llm && node.llm_prompt.is_none() {
            return Err(format!(
                "Dialogue `{}` node `{node_id}` enables LLM generation without an LLM prompt.",
                dialogue.id
            ));
        }
        for (label, prompt) in [
            ("LLM prompt", node.llm_prompt.as_deref()),
            ("LLM system prompt", node.llm_system_prompt.as_deref()),
        ] {
            if let Some(prompt) = prompt {
                validate_dialogue_text(
                    prompt,
                    label,
                    1,
                    MAX_DIALOGUE_PROMPT_CHARS,
                    &dialogue.id,
                    Some(node_id),
                )?;
            }
        }
        for (choice_index, choice) in node.choices.iter().enumerate() {
            validate_dialogue_text(
                &choice.text,
                &format!("choice {} text", choice_index + 1),
                1,
                2_048,
                &dialogue.id,
                Some(node_id),
            )?;
            if choice.relationship_changes.len() > MAX_RELATIONSHIP_CHANGES_PER_CHOICE {
                return Err(format!(
                    "Dialogue `{}` node `{node_id}` choice {} has too many relationship changes.",
                    dialogue.id,
                    choice_index + 1
                ));
            }
            if let Some(condition) = choice.condition.as_deref() {
                validate_condition_source(condition).map_err(|error| {
                    format!(
                        "Dialogue `{}` node `{node_id}` choice {} condition is invalid: {error}",
                        dialogue.id,
                        choice_index + 1
                    )
                })?;
            }
            for (character_id, delta) in &choice.relationship_changes {
                if !character_ids.contains(character_id) {
                    return Err(format!(
                        "Dialogue `{}` node `{node_id}` choice {} changes unknown character `{character_id}`.",
                        dialogue.id,
                        choice_index + 1
                    ));
                }
                if !delta.is_finite() || !(-1.0..=1.0).contains(delta) {
                    return Err(format!(
                        "Dialogue `{}` node `{node_id}` choice {} relationship delta for `{character_id}` must be between -1 and 1.",
                        dialogue.id,
                        choice_index + 1
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_dialogue_text(
    value: &str,
    label: &str,
    min: usize,
    max: usize,
    dialogue_id: &str,
    node_id: Option<&str>,
) -> Result<(), String> {
    let count = value.chars().count();
    if count < min
        || count > max
        || value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        let location = node_id
            .map(|node_id| format!(" node `{node_id}`"))
            .unwrap_or_default();
        return Err(format!(
            "Dialogue `{dialogue_id}`{location} {label} must contain {min} to {max} supported characters."
        ));
    }
    Ok(())
}

fn ensure_dialogue_catalog_fingerprint(
    current: &[LoadedDialogueScript],
    expected: &str,
) -> Result<(), String> {
    let actual = dialogue_authoring_catalog_fingerprint(current);
    if actual != expected {
        return Err(format!(
            "Dialogue catalog changed since it was opened; expected `{expected}`, current `{actual}`. Reload before saving."
        ));
    }
    Ok(())
}

fn dialogue_authoring_catalog_fingerprint(dialogues: &[LoadedDialogueScript]) -> String {
    let entries = dialogues
        .iter()
        .map(|loaded| {
            json!({
                "source_path": loaded.source_path,
                "dialogue": loaded.dialogue,
            })
        })
        .collect::<Vec<Value>>();
    sha256_json(&json!({
        "schema": DIALOGUE_AUTHORING_CATALOG_SCHEMA_V1,
        "dialogues": entries,
    }))
}

fn dialogue_content_fingerprint(dialogue: &DialogueScript) -> String {
    sha256_json(&json!({
        "schema": "monogatari-dialogue-content-fingerprint/v1",
        "dialogue": dialogue,
    }))
}

async fn ensure_dialogue_access(state: &AppState, dialogue_id: &str) -> Result<(), String> {
    let catalog = state.story_event_catalog.read().await;
    let progress = state.story_progress.read().await;
    ensure_story_content_access(&catalog, &progress, StoryContentKind::Dialogue, dialogue_id)?;
    Ok(())
}

pub(crate) async fn ensure_project_dialogues_loaded(state: &AppState) -> Result<(), String> {
    if !state.dialogue_manager.read().await.script_ids().is_empty() {
        return Ok(());
    }

    let dialogue_root = state.current_project_data_root().await.join("dialogue");
    if !dialogue_root.is_dir() {
        return Ok(());
    }
    let mut loaded = DialogueManager::new();
    loaded
        .load_from_directory(&dialogue_root)
        .await
        .map_err(|error| error.to_string())?;

    let mut active = state.dialogue_manager.write().await;
    if active.script_ids().is_empty() {
        *active = loaded;
    }
    Ok(())
}

fn get_dialogue_state_inner(dm: &DialogueManager) -> Result<DialogueState, String> {
    if !dm.is_active() {
        return Ok(DialogueState {
            is_active: false,
            speaker: None,
            text: String::new(),
            emotion: None,
            choices: Vec::new(),
            live2d_expression: None,
        });
    }

    let node = dm.current_node().ok_or("No current node")?;
    let choices = node
        .choices
        .iter()
        .enumerate()
        .map(|(index, choice)| ChoiceInfo {
            index,
            text: choice.text.clone(),
        })
        .collect();

    Ok(DialogueState {
        is_active: true,
        speaker: node.speaker_id.clone(),
        text: node.text.clone(),
        emotion: node.emotion.clone(),
        choices,
        live2d_expression: node.emotion.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story_events::StoryEventCatalog;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_dialogue_authoring_{label}_{}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn write_project(root: &Path) {
        for directory in [
            "characters",
            "knowledge",
            "events",
            "endings",
            "scenes",
            "dialogue",
        ] {
            std::fs::create_dir_all(root.join(directory)).unwrap();
        }
        std::fs::write(
            root.join("characters").join("sakura.json"),
            r#"{"id":"sakura","name":"Sakura"}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("events").join("events.json"),
            r#"{"schema":"monogatari-story-event-catalog/v1","events":[]}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("dialogue").join("intro.json"),
            r#"{
              "id":"intro","title":"Intro","description":"Opening scene.","start_node_id":"start",
              "nodes":{
                "start":{"speaker_id":"sakura","text":"Hello.","next_node_id":"end"},
                "end":{"speaker_id":"sakura","text":"Goodbye.","is_ending":true,"ending_type":"good"}
              },
              "variables":{}
            }"#,
        )
        .unwrap();
    }

    async fn authoring_state(root: &Path) -> AppState {
        let state = AppState::new();
        state.set_project_data_root(root.to_path_buf()).await;
        *state.story_event_catalog.write().await =
            StoryEventCatalog::load_from_project_root(root).unwrap();
        state
    }

    #[tokio::test]
    async fn dialogue_save_is_atomic_rejects_stale_graphs_and_hot_reloads_runtime() {
        let root = temp_root("save");
        write_project(&root);
        let state = authoring_state(&root).await;
        let before = dialogue_authoring_catalog_snapshot(&state).await.unwrap();
        assert_eq!(before.dialogue_count, 1);
        let mut replacement = before.dialogues[0].dialogue.clone();
        replacement.title = "A Better Intro".to_string();

        let saved = save_dialogue_definition_inner(
            &state,
            replacement.clone(),
            Some("intro"),
            &before.catalog_fingerprint,
        )
        .await
        .unwrap();
        assert_ne!(saved.catalog_fingerprint, before.catalog_fingerprint);
        assert_eq!(saved.dialogues[0].dialogue.title, "A Better Intro");
        assert_eq!(
            state.dialogue_manager.read().await.scripts()[0].title,
            "A Better Intro"
        );

        let mut stale = replacement.clone();
        stale.title = "Stale Intro".to_string();
        assert!(save_dialogue_definition_inner(
            &state,
            stale,
            Some("intro"),
            &before.catalog_fingerprint,
        )
        .await
        .unwrap_err()
        .contains("changed since it was opened"));

        let path = root.join("dialogue").join("intro.json");
        let file_before_invalid = std::fs::read_to_string(&path).unwrap();
        let mut invalid = replacement;
        invalid.nodes.get_mut("start").unwrap().next_node_id = Some("missing".to_string());
        assert!(save_dialogue_definition_inner(
            &state,
            invalid,
            Some("intro"),
            &saved.catalog_fingerprint,
        )
        .await
        .unwrap_err()
        .contains("does not exist"));
        assert_eq!(std::fs::read_to_string(path).unwrap(), file_before_invalid);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn dialogue_delete_requires_event_and_ending_references_to_be_removed() {
        let root = temp_root("delete");
        write_project(&root);
        std::fs::write(
            root.join("events").join("events.json"),
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"unlock_intro","event_type":"unlock","description":"Unlock intro",
                "actions":[{"type":"unlock_dialogue","dialogue_id":"intro"}]
              }]
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("endings").join("intro_ending.json"),
            r#"{
              "schema":"monogatari-story-ending/v1","id":"intro_ending","title":"Intro Ending",
              "description":"An ending.","scene_id":"intro_scene","dialogue_id":"intro"
            }"#,
        )
        .unwrap();
        let state = authoring_state(&root).await;
        let before = dialogue_authoring_catalog_snapshot(&state).await.unwrap();

        let error = delete_dialogue_definition_inner(&state, "intro", &before.catalog_fingerprint)
            .await
            .unwrap_err();
        assert!(error.contains("event:unlock_intro"), "{error}");
        assert!(error.contains("ending:intro_ending"), "{error}");
        assert!(root.join("dialogue").join("intro.json").is_file());

        std::fs::write(
            root.join("events").join("events.json"),
            r#"{"schema":"monogatari-story-event-catalog/v1","events":[]}"#,
        )
        .unwrap();
        std::fs::remove_file(root.join("endings").join("intro_ending.json")).unwrap();
        let after = delete_dialogue_definition_inner(&state, "intro", &before.catalog_fingerprint)
            .await
            .unwrap();
        assert_eq!(after.dialogue_count, 0);
        assert!(!root.join("dialogue").join("intro.json").exists());
        assert!(state.dialogue_manager.read().await.script_ids().is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn dialogue_loader_rejects_unknown_fields_speakers_and_unreachable_nodes() {
        let root = temp_root("invalid");
        write_project(&root);
        let character_ids = load_project_character_ids(&root).await.unwrap();
        let path = root.join("dialogue").join("intro.json");
        std::fs::write(
            &path,
            r#"{
              "id":"intro","title":"Intro","start_node_id":"start","extra":true,
              "nodes":{"start":{"text":"Hello."}}
            }"#,
        )
        .unwrap();
        assert!(load_dialogue_documents(&root, &character_ids).is_err());

        std::fs::write(
            &path,
            r#"{
              "id":"intro","title":"Intro","start_node_id":"start",
              "nodes":{"start":{"speaker_id":"missing","text":"Hello."},"orphan":{"text":"Lost."}}
            }"#,
        )
        .unwrap();
        let error = load_dialogue_documents(&root, &character_ids).unwrap_err();
        assert!(
            error.contains("Unreachable") || error.contains("unknown speaker"),
            "{error}"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn dialogue_choices_apply_and_clamp_relationship_effects() {
        let root = temp_root("choice_effects");
        write_project(&root);
        std::fs::write(
            root.join("dialogue").join("intro.json"),
            r#"{
              "id":"intro","title":"Intro","start_node_id":"start",
              "nodes":{
                "start":{"speaker_id":"sakura","text":"Choose.","choices":[{
                  "text":"Be kind","next_node_id":"end","relationship_changes":{"sakura":0.6}
                },{
                  "text":"Unknown","next_node_id":"end","relationship_changes":{"missing":0.2}
                }]},
                "end":{"speaker_id":"sakura","text":"Thank you.","is_ending":true}
              }
            }"#,
        )
        .unwrap();
        let state = authoring_state(&root).await;

        start_dialogue_authoring_inner(&state, "intro")
            .await
            .unwrap();
        let error = select_choice_inner(&state, 1).await.unwrap_err();
        assert!(error.contains("unknown character `missing`"));
        assert_eq!(
            state
                .dialogue_manager
                .read()
                .await
                .current_node()
                .unwrap()
                .text,
            "Choose."
        );

        for expected in [0.6_f32, 1.0_f32] {
            start_dialogue_authoring_inner(&state, "intro")
                .await
                .unwrap();
            let dialogue = select_choice_inner(&state, 0).await.unwrap();
            assert_eq!(dialogue.text, "Thank you.");
            let character = state
                .character_manager
                .read()
                .await
                .get_character("sakura")
                .unwrap();
            let character = character.read().await;
            let actual = character
                .relationships
                .get("player")
                .copied()
                .unwrap_or(0.0);
            assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
        }

        std::fs::remove_dir_all(root).unwrap();
    }
}
