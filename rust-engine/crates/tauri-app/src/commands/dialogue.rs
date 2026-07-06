//! Dialogue management commands.

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct DialogueState {
    pub is_active: bool,
    pub speaker: Option<String>,
    pub text: String,
    pub emotion: Option<String>,
    pub choices: Vec<ChoiceInfo>,
    pub live2d_expression: Option<String>,
}

#[derive(Serialize)]
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
    let mut dm = state.dialogue_manager.write().await;
    dm.start_dialogue(&dialogue_id)
        .await
        .map_err(|e| e.to_string())?;
    get_dialogue_state_inner(&dm)
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
pub async fn get_dialogue_state(
    state: State<'_, AppState>,
) -> Result<DialogueState, String> {
    let dm = state.dialogue_manager.read().await;
    get_dialogue_state_inner(&dm)
}

/// Load dialogue scripts from a directory.
#[tauri::command]
pub async fn load_dialogues(
    state: State<'_, AppState>,
    directory: String,
) -> Result<usize, String> {
    let path = std::path::PathBuf::from(&directory);
    let mut dm = state.dialogue_manager.write().await;
    dm.load_from_directory(&path)
        .await
        .map_err(|e| e.to_string())
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
