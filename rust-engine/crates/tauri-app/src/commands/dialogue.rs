//! Dialogue management commands.

use serde::Serialize;
use tauri::State;

use crate::commands::content_paths::resolve_project_content_dir;
use crate::state::AppState;
use crate::story_access::{
    ensure_story_content_access, story_content_access, StoryContentAccessEntry, StoryContentKind,
};

use llm_game::dialogue::{DialogueManager, DialogueScriptSummary};

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

/// Start a dialogue by script ID.
#[tauri::command]
pub async fn start_dialogue(
    state: State<'_, AppState>,
    dialogue_id: String,
) -> Result<DialogueState, String> {
    start_dialogue_inner(&state, &dialogue_id).await
}

pub(crate) async fn start_dialogue_inner(
    state: &AppState,
    dialogue_id: &str,
) -> Result<DialogueState, String> {
    ensure_dialogue_access(state, dialogue_id).await?;
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
    let mut dm = state.dialogue_manager.write().await;
    dm.select_choice(choice_index)
        .await
        .map_err(|e| e.to_string())?;
    get_dialogue_state_inner(&dm)
}

/// Get the current dialogue state.
#[tauri::command]
pub async fn get_dialogue_state(state: State<'_, AppState>) -> Result<DialogueState, String> {
    let dm = state.dialogue_manager.read().await;
    get_dialogue_state_inner(&dm)
}

/// Load dialogue scripts from a directory.
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

fn get_dialogue_state_inner(
    dm: &llm_game::dialogue::DialogueManager,
) -> Result<DialogueState, String> {
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
        .map(|(i, c)| ChoiceInfo {
            index: i,
            text: c.text.clone(),
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
