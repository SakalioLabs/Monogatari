//! Dialogue runtime and project authoring commands.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use llm_authoring::dialogue_validation::{
    ensure_valid_dialogue_script, normalize_dialogue_script, MAX_DIALOGUE_FILE_BYTES,
};
use llm_authoring::filesystem::{
    ensure_regular_project_directory, sha256_json, source_label, stage_json_deletion,
    stage_json_replacement,
};
use llm_game::characters::{Character, CharacterManager};
use llm_game::dialogue::{
    DialogueChoiceEffects, DialogueFreeTalk, DialogueGenerationRequest, DialogueManager,
    DialogueScript, DialogueScriptSummary, LLMInferenceCallback,
};
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
const MAX_FREE_TALK_PLAYER_CHARS: usize = 2_000;

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
    pub scene_id: Option<String>,
    pub text: String,
    pub emotion: Option<String>,
    pub choices: Vec<ChoiceInfo>,
    pub live2d_expression: Option<String>,
    pub free_talk: Option<DialogueFreeTalk>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceInfo {
    pub index: usize,
    pub text: String,
}

/// One contained free-talk reply. It never records relationship, event, or route state.
#[derive(Debug, Clone, Serialize)]
pub struct DialogueFreeTalkResponse {
    pub character_response: String,
    pub emotion: String,
    pub used_fallback: bool,
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
    ensure_project_characters_loaded(state).await?;
    configure_dialogue_generation_callback(state).await;
    let mut dm = state.dialogue_manager.write().await;
    dm.start_dialogue(dialogue_id)
        .await
        .map_err(|e| e.to_string())?;
    drop(dm);
    *state.active_scene_roleplay_id.write().await = None;
    *state.active_roleplay_campaign_id.write().await = None;
    let dm = state.dialogue_manager.read().await;
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
    configure_dialogue_generation_callback(&state).await;
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
    configure_dialogue_generation_callback(state).await;
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

/// Generate one optional chapter free-talk reply without mutating story progress.
#[tauri::command]
pub async fn send_dialogue_free_talk_message(
    state: State<'_, AppState>,
    message: String,
) -> Result<DialogueFreeTalkResponse, String> {
    send_dialogue_free_talk_message_inner(&state, &message).await
}

async fn send_dialogue_free_talk_message_inner(
    state: &AppState,
    message: &str,
) -> Result<DialogueFreeTalkResponse, String> {
    let message = message.trim().to_string();
    if message.is_empty() {
        return Err("Free talk message cannot be empty.".to_string());
    }
    if message.chars().count() > MAX_FREE_TALK_PLAYER_CHARS {
        return Err(format!(
            "Free talk messages cannot exceed {MAX_FREE_TALK_PLAYER_CHARS} characters."
        ));
    }
    ensure_project_characters_loaded(state).await?;

    let free_talk = state
        .dialogue_manager
        .read()
        .await
        .current_node()
        .and_then(|node| node.free_talk.clone())
        .ok_or_else(|| {
            "The current dialogue node does not offer contained free talk.".to_string()
        })?;

    let (character_name, character_profile, character_emotion, knowledge_refs) = {
        let manager = state.character_manager.read().await;
        let character = manager
            .get_character(&free_talk.character_id)
            .ok_or_else(|| {
                format!(
                    "Free talk character `{}` is unavailable.",
                    free_talk.character_id
                )
            })?;
        let character = character.read().await;
        (
            character.name.clone(),
            character.build_system_prompt(),
            character.emotion.clone(),
            character.knowledge_refs.clone(),
        )
    };
    let knowledge_context = {
        let knowledge_base = state.knowledge_base.read().await;
        crate::commands::chat::build_character_knowledge_context_details(
            &knowledge_base,
            &message,
            &knowledge_refs,
            2,
        )
    };
    let prompt = build_contained_free_talk_prompt(
        &character_name,
        &character_profile,
        &free_talk,
        &message,
        &knowledge_context.content,
    );
    let result = state
        .inference_pipeline
        .read()
        .await
        .generate_response(
            &prompt,
            &llm_ai::InferenceOptions {
                max_tokens: 160,
                temperature: 0.72,
                ..Default::default()
            },
        )
        .await;

    let response = match result {
        Ok(result) if result.success => {
            let guarded = crate::commands::prompt_guard::guard_character_response(
                &character_name,
                &result.text,
            );
            let bounded = truncate_visible_text(&guarded, free_talk.max_characters);
            if bounded.is_empty() {
                return Ok(DialogueFreeTalkResponse {
                    character_response: free_talk.fallback_text,
                    emotion: character_emotion,
                    used_fallback: true,
                });
            }
            DialogueFreeTalkResponse {
                character_response: bounded,
                emotion: character_emotion,
                used_fallback: false,
            }
        }
        _ => DialogueFreeTalkResponse {
            character_response: free_talk.fallback_text,
            emotion: character_emotion,
            used_fallback: true,
        },
    };
    Ok(response)
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
    let project_root = state.current_project_data_root().await?;
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
                .filter(|node| node.use_llm || node.response_generation.is_some())
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
    let project_root = state.current_project_data_root().await?;
    let character_ids = load_project_character_ids(&project_root).await?;
    let dialogue = normalize_dialogue_script(dialogue)?;
    ensure_valid_dialogue_script(&dialogue, &character_ids)?;
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
    let project_root = state.current_project_data_root().await?;
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
        ensure_valid_dialogue_script(&dialogue, character_ids)
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

    let dialogue_root = state.current_project_data_root().await?.join("dialogue");
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
            scene_id: None,
            text: String::new(),
            emotion: None,
            choices: Vec::new(),
            live2d_expression: None,
            free_talk: None,
        });
    }

    let node = dm.current_node().ok_or("No current node")?;
    let choices = dm
        .available_choices()
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|(index, choice)| ChoiceInfo {
            index,
            text: choice.text,
        })
        .collect();

    Ok(DialogueState {
        is_active: true,
        speaker: node.speaker_id.clone(),
        scene_id: node.scene_id.clone(),
        text: dm.current_text().to_string(),
        emotion: node.emotion.clone(),
        choices,
        live2d_expression: node.emotion.clone(),
        free_talk: node.free_talk.clone(),
    })
}

async fn configure_dialogue_generation_callback(state: &AppState) {
    let character_manager = state.character_manager.clone();
    let knowledge_base = state.knowledge_base.clone();
    let inference_pipeline = state.inference_pipeline.clone();
    let callback: Arc<LLMInferenceCallback> = Arc::new(Box::new(move |request| {
        let character_manager = character_manager.clone();
        let knowledge_base = knowledge_base.clone();
        let inference_pipeline = inference_pipeline.clone();
        Box::pin(async move {
            generate_controlled_dialogue_response(
                character_manager,
                knowledge_base,
                inference_pipeline,
                request,
            )
            .await
        })
    }));
    state
        .dialogue_manager
        .write()
        .await
        .set_llm_callback(callback);
}

async fn generate_controlled_dialogue_response(
    character_manager: Arc<RwLock<CharacterManager>>,
    knowledge_base: Arc<RwLock<llm_game::knowledge::KnowledgeBase>>,
    inference_pipeline: Arc<RwLock<llm_ai::InferencePipeline>>,
    request: DialogueGenerationRequest,
) -> llm_core::Result<String> {
    let character_id = request.speaker_id.as_deref().ok_or_else(|| {
        llm_core::EngineError::inference(
            "dialogue",
            format!(
                "{}:{} generated response has no character speaker",
                request.script_id, request.node_id
            ),
        )
    })?;
    let (character_name, character_profile, knowledge_refs) = {
        let manager = character_manager.read().await;
        let character = manager.get_character(character_id).ok_or_else(|| {
            llm_core::EngineError::inference(
                "dialogue",
                format!("Generated dialogue character `{character_id}` is unavailable."),
            )
        })?;
        let character = character.read().await;
        (
            character.name.clone(),
            character.build_system_prompt(),
            character.knowledge_refs.clone(),
        )
    };
    let knowledge_context = {
        let knowledge_base = knowledge_base.read().await;
        crate::commands::chat::build_character_knowledge_context_details(
            &knowledge_base,
            &request.prompt,
            &knowledge_refs,
            2,
        )
    };
    let prompt = build_controlled_dialogue_prompt(
        &character_name,
        &character_profile,
        &request,
        &knowledge_context.content,
    );
    let result = inference_pipeline
        .read()
        .await
        .generate_response(
            &prompt,
            &llm_ai::InferenceOptions {
                max_tokens: 180,
                temperature: 0.72,
                ..Default::default()
            },
        )
        .await
        .map_err(|error| llm_core::EngineError::inference("dialogue", error))?;
    if !result.success {
        return Err(llm_core::EngineError::inference(
            "dialogue",
            result
                .error
                .unwrap_or_else(|| "Generated dialogue response failed.".to_string()),
        ));
    }
    Ok(crate::commands::prompt_guard::guard_character_response(
        &character_name,
        &result.text,
    ))
}

fn build_controlled_dialogue_prompt(
    character_name: &str,
    character_profile: &str,
    request: &DialogueGenerationRequest,
    knowledge_context: &str,
) -> String {
    let contract = request.response_generation.as_ref().map_or_else(
        || {
            "Keep the reply short and in character. The author controls every story branch; do not mention or alter plot routing."
                .to_string()
        },
        |generation| {
            format!(
                "Reply in at most {} sentences and {} visible characters. The author controls every story branch; do not choose, describe, or alter a route. {} {}",
                generation.max_sentences,
                generation.max_characters,
                if generation.grounding_markers.is_empty() {
                    String::new()
                } else {
                    format!("Naturally acknowledge at least one of: {}.", generation.grounding_markers.join(", "))
                },
                if generation.forbidden_markers.is_empty() {
                    String::new()
                } else {
                    format!("Never use: {}.", generation.forbidden_markers.join(", "))
                },
            )
        },
    );
    let creator_context =
        crate::commands::prompt_guard::wrap_creator_system_instructions(&request.prompt);
    let legacy_system = request
        .system_prompt
        .as_deref()
        .map(crate::commands::prompt_guard::wrap_creator_system_instructions)
        .unwrap_or_default();
    let knowledge = if knowledge_context.trim().is_empty() {
        String::new()
    } else {
        crate::commands::prompt_guard::wrap_creator_system_instructions(knowledge_context)
    };

    format!(
        "[System]\nYou are {character_name}, a character in a fixed-route visual novel. You are not an assistant.\n\nCHARACTER PROFILE\n{character_profile}\n\n{}\n\n{}\n\n{}\n\n{}\n\nWrite only the visible in-character reply.\n\n[Assistant]\n",
        crate::commands::prompt_guard::character_mind_contract(),
        crate::commands::prompt_guard::character_safety_contract(),
        contract,
        [creator_context, legacy_system, knowledge]
            .into_iter()
            .filter(|section| !section.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
    )
}

fn build_contained_free_talk_prompt(
    character_name: &str,
    character_profile: &str,
    free_talk: &DialogueFreeTalk,
    player_message: &str,
    knowledge_context: &str,
) -> String {
    let creator_context =
        crate::commands::prompt_guard::wrap_creator_system_instructions(&free_talk.context);
    let knowledge = if knowledge_context.trim().is_empty() {
        String::new()
    } else {
        crate::commands::prompt_guard::wrap_creator_system_instructions(knowledge_context)
    };
    format!(
        "[System]\nYou are {character_name}, a character in a visual novel. This is an optional contained chapter conversation. Do not change relationship scores, unlocks, events, chapter outcomes, or story route. Keep the reply inside the authored scene boundary and finish in at most {} visible characters.\n\nCHARACTER PROFILE\n{character_profile}\n\n{}\n\n{}\n\n{}\n\n[User]\n{}\n\n[Assistant]\n",
        free_talk.max_characters,
        crate::commands::prompt_guard::character_mind_contract(),
        crate::commands::prompt_guard::character_safety_contract(),
        [creator_context, knowledge]
            .into_iter()
            .filter(|section| !section.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
        crate::commands::prompt_guard::wrap_player_message(player_message),
    )
}

fn truncate_visible_text(value: &str, maximum: usize) -> String {
    value
        .chars()
        .take(maximum.max(1))
        .collect::<String>()
        .trim()
        .to_string()
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
    async fn dialogue_create_rejects_portable_case_aliases_without_replacing_script() {
        let root = temp_root("case_alias");
        write_project(&root);
        let path = root.join("dialogue").join("intro.json");
        let original = std::fs::read(&path).unwrap();
        let state = authoring_state(&root).await;
        let before = dialogue_authoring_catalog_snapshot(&state).await.unwrap();
        let mut alias = before.dialogues[0].dialogue.clone();
        alias.id = "INTRO".to_string();

        let error =
            save_dialogue_definition_inner(&state, alias, None, &before.catalog_fingerprint)
                .await
                .unwrap_err();

        assert!(error.contains("collides with existing path"), "{error}");
        assert!(error.contains("by ASCII case"), "{error}");
        assert_eq!(std::fs::read(&path).unwrap(), original);
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
                "start":{"speaker_id":"sakura","text":"Choose.","script":"setFlag('visible', true)","choices":[{
                  "text":"Be kind","next_node_id":"end","condition":"hasFlag(\"visible\")","relationship_changes":{"sakura":0.6}
                },{
                  "text":"Hidden","next_node_id":"end","condition":"hasFlag(\"missing\")"
                },{
                  "text":"Unknown","next_node_id":"end","relationship_changes":{"missing":0.2}
                }]},
                "end":{"speaker_id":"sakura","text":"Thank you.","is_ending":true}
              }
            }"#,
        )
        .unwrap();
        let state = authoring_state(&root).await;

        let started = start_dialogue_authoring_inner(&state, "intro")
            .await
            .unwrap();
        assert_eq!(
            started
                .choices
                .iter()
                .map(|choice| choice.index)
                .collect::<Vec<_>>(),
            vec![0, 2]
        );
        let error = select_choice_inner(&state, 2).await.unwrap_err();
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

    #[tokio::test]
    async fn contained_free_talk_falls_back_without_mutating_story_or_chat_state() {
        let root = temp_root("contained_free_talk");
        write_project(&root);
        std::fs::write(
            root.join("dialogue").join("intro.json"),
            r#"{
              "id":"intro","title":"Intro","start_node_id":"start",
              "nodes":{
                "start":{
                  "speaker_id":"sakura","text":"Authored line.",
                  "free_talk":{
                    "character_id":"sakura",
                    "context":"Sakura is waiting beside the bridge. The chapter route is fixed.",
                    "fallback_text":"Sakura looks back toward the bridge and waits for the story to continue.",
                    "max_turns":2,
                    "max_characters":160
                  }
                }
              }
            }"#,
        )
        .unwrap();
        let state = authoring_state(&root).await;
        start_dialogue_authoring_inner(&state, "intro")
            .await
            .unwrap();
        let before_runtime = state.dialogue_manager.read().await.runtime_state();
        let before_relationship = state
            .character_manager
            .read()
            .await
            .get_character("sakura")
            .unwrap()
            .read()
            .await
            .relationships
            .get("player")
            .copied();

        let response = send_dialogue_free_talk_message_inner(&state, "What comes next?")
            .await
            .unwrap();

        assert!(response.used_fallback);
        assert_eq!(
            response.character_response,
            "Sakura looks back toward the bridge and waits for the story to continue."
        );
        assert_eq!(
            state.dialogue_manager.read().await.runtime_state(),
            before_runtime
        );
        assert!(state.chat_sessions.read().await.is_empty());
        let after_relationship = state
            .character_manager
            .read()
            .await
            .get_character("sakura")
            .unwrap()
            .read()
            .await
            .relationships
            .get("player")
            .copied();
        assert_eq!(after_relationship, before_relationship);
        std::fs::remove_dir_all(root).unwrap();
    }
}
