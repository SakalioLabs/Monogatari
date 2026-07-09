//! Save/Load game commands.

use std::collections::{HashMap, HashSet};

use llm_assets::{CharacterSaveState, GameSave, SaveManager};
use llm_core::normalize_script_state_map;
use llm_game::dialogue::DialogueRuntimeState;
use serde::Serialize;
use tauri::State;

use crate::commands::chat::{ChatSession, ConversationEvaluation};
use crate::state::AppState;
use crate::story_events::StoryEventAction;
use crate::story_progress::StoryProgressState;

const MAX_SAVE_NAME_CHARS: usize = 128;
const MAX_SAVED_SCENE_HISTORY: usize = 64;
const MAX_SAVED_CHAT_SESSIONS: usize = 256;
const MAX_SAVED_CHAT_MESSAGES_PER_SESSION: usize = 5_000;
const MAX_SAVED_CHAT_MESSAGE_CHARS: usize = 65_536;
const MAX_SAVED_CHAT_TIMESTAMP_CHARS: usize = 128;
const MAX_SAVED_TRIGGERED_EVENTS: usize = 4_096;
const MAX_SAVED_EVENT_ID_CHARS: usize = 128;

#[derive(Serialize)]
pub struct SaveInfo {
    pub save_id: String,
    pub save_name: String,
    pub timestamp: String,
    pub schema: String,
    pub current_scene: Option<String>,
    pub current_dialogue_id: Option<String>,
    pub character_state_count: usize,
    pub chat_session_count: usize,
}

#[derive(Debug, Default, PartialEq)]
struct SaveRestoreSummary {
    character_count: usize,
    chat_session_count: usize,
    applied_event_count: usize,
}

/// Save a complete runtime snapshot. A stable `save_id` overwrites a named slot.
#[tauri::command]
pub async fn save_game(
    state: State<'_, AppState>,
    save_name: String,
    save_id: Option<String>,
) -> Result<String, String> {
    let save = capture_game_save(&state, save_name, save_id).await?;
    let saved_id = save.save_id.clone();
    let save_mgr = state.save_manager.read().await;
    save_mgr.save(&save).await.map_err(|e| e.to_string())?;
    Ok(saved_id)
}

/// Load a game state by save ID.
#[tauri::command]
pub async fn load_game(state: State<'_, AppState>, save_id: String) -> Result<String, String> {
    let save = {
        let save_mgr = state.save_manager.read().await;
        save_mgr.load(&save_id).await.map_err(|e| e.to_string())?
    };
    let save_name = save.save_name.clone();
    let summary = restore_game_save(&state, save).await?;

    Ok(format!(
        "Loaded save: {save_name} ({} character state(s), {} chat session(s), {} applied story event(s))",
        summary.character_count, summary.chat_session_count, summary.applied_event_count
    ))
}

async fn capture_game_save(
    state: &AppState,
    save_name: String,
    save_id: Option<String>,
) -> Result<GameSave, String> {
    let save_name = normalize_save_name(&save_name)?;
    let dialogue_state = state.dialogue_manager.read().await.runtime_state();
    let active_scene = state.active_scene_id.read().await.clone();
    let managed_scene = state
        .scene_manager
        .read()
        .await
        .current_scene_name()
        .map(str::to_string);
    let current_scene = active_scene.or(managed_scene);

    let mut save = match save_id {
        Some(save_id) => SaveManager::create_save_with_id(
            save_id.trim(),
            &save_name,
            current_scene,
            dialogue_state.active_script_id.clone(),
            dialogue_state.current_node_id.clone(),
        )
        .map_err(|e| e.to_string())?,
        None => SaveManager::create_save(
            &save_name,
            current_scene,
            dialogue_state.active_script_id.clone(),
            dialogue_state.current_node_id.clone(),
        ),
    };

    save.scene_history = normalize_scene_history(state.scene_history.read().await.clone())?;
    if let Some(current_scene) = save.current_scene.as_ref() {
        if save.scene_history.last() != Some(current_scene) {
            save.scene_history.push(current_scene.clone());
        }
        if save.scene_history.len() > MAX_SAVED_SCENE_HISTORY {
            save.scene_history
                .drain(0..save.scene_history.len() - MAX_SAVED_SCENE_HISTORY);
        }
    }
    save.dialogue_state = Some(dialogue_state);

    // Match the event executor lock order so action flags and progress share one snapshot boundary.
    let story_progress = state.story_progress.read().await;
    let script_engine = state.script_engine.read().await;
    save.flags = script_engine.all_flags();
    save.variables = script_variables_to_json(script_engine.all_variables())?;
    save.story_progress = Some(
        serde_json::to_value(story_progress.clone())
            .map_err(|error| format!("Unable to serialize story progress: {error}"))?,
    );
    drop(script_engine);
    drop(story_progress);

    save.characters = snapshot_character_states(state).await;
    save.chat_sessions = snapshot_chat_sessions(state).await?;
    Ok(save)
}

async fn restore_game_save(
    state: &AppState,
    mut save: GameSave,
) -> Result<SaveRestoreSummary, String> {
    save.validate_schema().map_err(|e| e.to_string())?;
    save.flags = normalize_script_state_map(save.flags).map_err(|e| e.to_string())?;
    save.variables = normalize_script_state_map(save.variables).map_err(|e| e.to_string())?;
    let script_variables = json_variables_to_script(&save.variables)?;
    let scene_history = normalize_scene_history(save.scene_history.clone())?;
    let chat_sessions = deserialize_chat_sessions(&save.chat_sessions)?;
    let known_character_ids: HashSet<String> = state
        .character_manager
        .read()
        .await
        .character_ids()
        .into_iter()
        .collect();
    let chat_sessions: HashMap<String, ChatSession> = chat_sessions
        .into_iter()
        .filter(|(character_id, _)| known_character_ids.contains(character_id))
        .collect();
    let had_story_progress = save.story_progress.is_some();
    let mut story_progress = deserialize_story_progress(save.story_progress.take())?;
    if !had_story_progress {
        migrate_legacy_story_progress(state, &chat_sessions, &mut story_progress, &mut save.flags)
            .await?;
    }

    let dialogue_state = save.dialogue_state.clone().unwrap_or(DialogueRuntimeState {
        active_script_id: save.current_dialogue_id.clone(),
        current_node_id: save.current_node_id.clone(),
        flags: save.flags.clone(),
        variables: save.variables.clone(),
    });
    let dialogue_state = state
        .dialogue_manager
        .read()
        .await
        .validate_runtime_state(dialogue_state)
        .map_err(|e| e.to_string())?;

    // Every fallible conversion and cross-reference check is complete before mutation begins.
    state
        .script_engine
        .read()
        .await
        .load_state(script_variables, save.flags.clone())
        .map_err(|e| e.to_string())?;
    state
        .dialogue_manager
        .write()
        .await
        .restore_runtime_state(dialogue_state)
        .map_err(|e| e.to_string())?;

    let character_count = restore_character_states(state, save.characters).await;
    let chat_session_count = chat_sessions.len();
    *state.chat_sessions.write().await = chat_sessions;
    let applied_event_count = story_progress.applied_events.len();
    *state.story_progress.write().await = story_progress;
    *state.active_scene_id.write().await = save.current_scene.clone();
    *state.scene_history.write().await = if scene_history.is_empty() {
        save.current_scene.into_iter().collect()
    } else {
        scene_history
    };

    Ok(SaveRestoreSummary {
        character_count,
        chat_session_count,
        applied_event_count,
    })
}

fn deserialize_story_progress(
    snapshot: Option<serde_json::Value>,
) -> Result<StoryProgressState, String> {
    let progress = snapshot
        .map(serde_json::from_value::<StoryProgressState>)
        .transpose()
        .map_err(|error| format!("Invalid story progress snapshot: {error}"))?
        .unwrap_or_default();
    progress.validate_and_normalize()
}

async fn migrate_legacy_story_progress(
    state: &AppState,
    chat_sessions: &HashMap<String, ChatSession>,
    progress: &mut StoryProgressState,
    flags: &mut HashMap<String, bool>,
) -> Result<(), String> {
    let catalog = state.story_event_catalog.read().await.clone();
    for (character_id, session) in chat_sessions {
        for event_id in &session.triggered_event_ids {
            let Some(definition) = catalog.definition(event_id, None) else {
                continue;
            };
            if !definition.rule.character_ids.is_empty()
                && !definition.applies_to_character(character_id)
            {
                continue;
            }
            let application = progress.apply_event(
                definition,
                Some(character_id),
                catalog.catalog_fingerprint(),
            )?;
            for result in application.action_results {
                if let StoryEventAction::SetFlag { flag, value } = result.action {
                    flags.insert(flag, value);
                }
            }
        }
    }
    Ok(())
}

async fn snapshot_character_states(state: &AppState) -> HashMap<String, CharacterSaveState> {
    let characters: Vec<(String, _)> = state
        .character_manager
        .read()
        .await
        .all_characters()
        .iter()
        .map(|(id, character)| (id.clone(), character.clone()))
        .collect();
    let mut snapshots = HashMap::with_capacity(characters.len());

    for (id, character) in characters {
        let character = character.read().await;
        snapshots.insert(
            id,
            CharacterSaveState {
                emotion: character.emotion.clone(),
                relationships: character.relationships.clone(),
                memory_count: character.memory.len(),
                memory: Some(character.memory.clone()),
            },
        );
    }

    snapshots
}

async fn restore_character_states(
    state: &AppState,
    snapshots: HashMap<String, CharacterSaveState>,
) -> usize {
    let targets: Vec<_> = {
        let manager = state.character_manager.read().await;
        snapshots
            .into_iter()
            .filter_map(|(id, snapshot)| {
                manager
                    .get_character(&id)
                    .map(|character| (character, snapshot))
            })
            .collect()
    };
    let restored_count = targets.len();

    for (character, snapshot) in targets {
        let mut character = character.write().await;
        let emotion = snapshot.emotion.trim();
        character.set_emotion(if emotion.is_empty() {
            "neutral"
        } else {
            emotion
        });
        character.relationships = snapshot
            .relationships
            .into_iter()
            .filter(|(_, score)| score.is_finite())
            .map(|(id, score)| (id, score.clamp(-1.0, 1.0)))
            .collect();
        if let Some(memory) = snapshot.memory {
            character.memory = memory;
        }
    }

    restored_count
}

async fn snapshot_chat_sessions(
    state: &AppState,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let sessions = state.chat_sessions.read().await;
    if sessions.len() > MAX_SAVED_CHAT_SESSIONS {
        return Err(format!(
            "A save can contain at most {MAX_SAVED_CHAT_SESSIONS} chat sessions."
        ));
    }

    sessions
        .iter()
        .map(|(character_id, session)| {
            let session = validate_chat_session(character_id, session.clone())?;
            serde_json::to_value(session)
                .map(|value| (character_id.clone(), value))
                .map_err(|e| format!("Unable to serialize chat session `{character_id}`: {e}"))
        })
        .collect()
}

fn deserialize_chat_sessions(
    snapshots: &HashMap<String, serde_json::Value>,
) -> Result<HashMap<String, ChatSession>, String> {
    if snapshots.len() > MAX_SAVED_CHAT_SESSIONS {
        return Err(format!(
            "A save can contain at most {MAX_SAVED_CHAT_SESSIONS} chat sessions."
        ));
    }

    snapshots
        .iter()
        .map(|(character_id, value)| {
            let session: ChatSession = serde_json::from_value(value.clone())
                .map_err(|e| format!("Invalid chat session snapshot for `{character_id}`: {e}"))?;
            let session = validate_chat_session(character_id, session)?;
            Ok((character_id.clone(), session))
        })
        .collect()
}

fn validate_chat_session(
    character_id: &str,
    mut session: ChatSession,
) -> Result<ChatSession, String> {
    if session.character_id != character_id {
        return Err(format!(
            "Chat session key `{character_id}` does not match embedded character id `{}`.",
            session.character_id
        ));
    }
    if session.messages.len() > MAX_SAVED_CHAT_MESSAGES_PER_SESSION {
        return Err(format!(
            "Chat session `{character_id}` exceeds {MAX_SAVED_CHAT_MESSAGES_PER_SESSION} messages."
        ));
    }
    for message in &session.messages {
        if !matches!(message.role.as_str(), "player" | "character" | "system") {
            return Err(format!(
                "Chat session `{character_id}` contains unsupported role `{}`.",
                message.role
            ));
        }
        if message.content.chars().count() > MAX_SAVED_CHAT_MESSAGE_CHARS {
            return Err(format!(
                "Chat session `{character_id}` contains an oversized message."
            ));
        }
        if message.timestamp.chars().count() > MAX_SAVED_CHAT_TIMESTAMP_CHARS
            || message.timestamp.chars().any(char::is_control)
        {
            return Err(format!(
                "Chat session `{character_id}` contains an invalid message timestamp."
            ));
        }
    }
    if !session.cumulative_score.is_finite() {
        return Err(format!(
            "Chat session `{character_id}` contains an invalid cumulative score."
        ));
    }
    session.cumulative_score = session.cumulative_score.max(0.0);
    if session.triggered_event_ids.len() > MAX_SAVED_TRIGGERED_EVENTS {
        return Err(format!(
            "Chat session `{character_id}` exceeds {MAX_SAVED_TRIGGERED_EVENTS} triggered events."
        ));
    }
    if session.triggered_event_ids.iter().any(|event_id| {
        event_id.trim().is_empty()
            || event_id.chars().count() > MAX_SAVED_EVENT_ID_CHARS
            || event_id.chars().any(char::is_control)
    }) {
        return Err(format!(
            "Chat session `{character_id}` contains an invalid triggered event id."
        ));
    }
    session.triggered_event_ids.sort();
    session.triggered_event_ids.dedup();
    if let Some(evaluation) = &mut session.last_evaluation {
        normalize_evaluation(evaluation);
    }
    Ok(session)
}

fn normalize_evaluation(evaluation: &mut ConversationEvaluation) {
    evaluation.friendliness = normalized_score(evaluation.friendliness);
    evaluation.engagement = normalized_score(evaluation.engagement);
    evaluation.creativity = normalized_score(evaluation.creativity);
    evaluation.overall_score = normalized_score(evaluation.overall_score);
}

fn normalized_score(score: f32) -> f32 {
    if score.is_finite() {
        score.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn script_variables_to_json(
    variables: HashMap<String, rhai::Dynamic>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let variables = normalize_script_state_map(variables).map_err(|e| e.to_string())?;
    variables
        .into_iter()
        .map(|(name, value)| {
            rhai::serde::from_dynamic::<serde_json::Value>(&value)
                .map(|value| (name.clone(), value))
                .map_err(|e| format!("Unable to serialize script variable `{name}`: {e}"))
        })
        .collect()
}

fn json_variables_to_script(
    variables: &HashMap<String, serde_json::Value>,
) -> Result<HashMap<String, rhai::Dynamic>, String> {
    variables
        .iter()
        .map(|(name, value)| {
            rhai::serde::to_dynamic(value.clone())
                .map(|value| (name.clone(), value))
                .map_err(|e| format!("Unable to restore script variable `{name}`: {e}"))
        })
        .collect()
}

fn normalize_save_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Save name is required.".to_string());
    }
    if name.chars().count() > MAX_SAVE_NAME_CHARS {
        return Err(format!(
            "Save name must be {MAX_SAVE_NAME_CHARS} characters or fewer."
        ));
    }
    if name.chars().any(char::is_control) {
        return Err("Save name cannot contain control characters.".to_string());
    }
    Ok(name.to_string())
}

fn normalize_scene_history(history: Vec<String>) -> Result<Vec<String>, String> {
    let mut normalized = Vec::new();
    for scene_id in history {
        let scene_id = scene_id.trim();
        if scene_id.is_empty() || scene_id.chars().any(char::is_control) {
            return Err("Scene history contains an invalid scene id.".to_string());
        }
        if normalized.last().is_none_or(|last| last != scene_id) {
            normalized.push(scene_id.to_string());
        }
    }
    if normalized.len() > MAX_SAVED_SCENE_HISTORY {
        normalized.drain(0..normalized.len() - MAX_SAVED_SCENE_HISTORY);
    }
    Ok(normalized)
}

/// List all saved games.
#[tauri::command]
pub async fn list_saves(state: State<'_, AppState>) -> Result<Vec<SaveInfo>, String> {
    let save_mgr = state.save_manager.read().await;
    let saves = save_mgr.list_saves().await.map_err(|e| e.to_string())?;

    Ok(saves
        .into_iter()
        .map(|save| SaveInfo {
            save_id: save.save_id,
            save_name: save.save_name,
            timestamp: save.timestamp.to_rfc3339(),
            schema: save.schema,
            current_scene: save.current_scene,
            current_dialogue_id: save.current_dialogue_id,
            character_state_count: save.characters.len(),
            chat_session_count: save.chat_sessions.len(),
        })
        .collect())
}

/// Delete a save by ID.
#[tauri::command]
pub async fn delete_save(state: State<'_, AppState>, save_id: String) -> Result<String, String> {
    let save_mgr = state.save_manager.read().await;
    save_mgr.delete(&save_id).await.map_err(|e| e.to_string())?;
    Ok("Save deleted".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::chat::{ChatMessage, ChatSafetyTrace};
    use crate::commands::story_events::apply_story_event_definition;
    use llm_assets::GAME_SAVE_SCHEMA_V2;
    use llm_game::characters::{memory::MemoryType, Character};

    #[test]
    fn script_variable_json_round_trip_preserves_primitive_types() {
        let values = HashMap::from([
            ("score".to_string(), rhai::Dynamic::from(42_i64)),
            ("ready".to_string(), rhai::Dynamic::from(true)),
            (
                "label".to_string(),
                rhai::Dynamic::from("river".to_string()),
            ),
        ]);

        let json = script_variables_to_json(values).unwrap();
        assert_eq!(json["score"], serde_json::json!(42));
        assert_eq!(json["ready"], serde_json::json!(true));
        let restored = json_variables_to_script(&json).unwrap();
        assert_eq!(restored["score"].clone_cast::<i64>(), 42);
        assert!(restored["ready"].clone_cast::<bool>());
        assert_eq!(restored["label"].clone_cast::<String>(), "river");
    }

    #[test]
    fn restored_chat_sessions_validate_identity_and_clamp_scores() {
        let mut session = ChatSession::new("sakura".to_string());
        session.cumulative_score = 1.5;
        session.last_evaluation = Some(ConversationEvaluation {
            friendliness: 2.0,
            engagement: -1.0,
            creativity: 0.5,
            overall_score: 1.2,
            summary: "test".to_string(),
        });

        let restored = validate_chat_session("sakura", session).unwrap();
        assert_eq!(restored.cumulative_score, 1.5);
        let evaluation = restored.last_evaluation.unwrap();
        assert_eq!(evaluation.friendliness, 1.0);
        assert_eq!(evaluation.engagement, 0.0);
        assert_eq!(evaluation.overall_score, 1.0);

        assert!(validate_chat_session("luna", ChatSession::new("sakura".to_string())).is_err());
    }

    #[tokio::test]
    async fn game_save_round_trip_restores_character_chat_scene_and_script_state() {
        let state = AppState::new();
        let mut character = Character::new("sakura", "Sakura");
        character.set_emotion("happy");
        character.relationships.insert("player".to_string(), 0.65);
        character.add_memory(
            "A promise by the river".to_string(),
            MemoryType::Event,
            0.9,
            vec!["promise".to_string()],
        );
        state
            .character_manager
            .write()
            .await
            .add_character(character);
        *state.active_scene_id.write().await = Some("riverbank".to_string());
        *state.scene_history.write().await = vec!["park".to_string(), "riverbank".to_string()];
        state
            .script_engine
            .read()
            .await
            .set_variable("score", rhai::Dynamic::from(7_i64))
            .unwrap();
        state
            .script_engine
            .read()
            .await
            .set_flag("promise_seen", true)
            .unwrap();
        let mut session = ChatSession::new("sakura".to_string());
        session.messages.push(ChatMessage {
            role: "player".to_string(),
            content: "Do you remember?".to_string(),
            emotion: None,
            timestamp: "1".to_string(),
        });
        session.cumulative_score = 0.8;
        session.evaluation_count = 1;
        session.last_safety_trace = Some(ChatSafetyTrace::default());
        state
            .chat_sessions
            .write()
            .await
            .insert("sakura".to_string(), session);
        let (definition, catalog_fingerprint) = {
            let catalog = state.story_event_catalog.read().await;
            (
                catalog.definition("first_friend", None).unwrap().clone(),
                catalog.catalog_fingerprint().to_string(),
            )
        };
        apply_story_event_definition(&state, &definition, Some("sakura"), &catalog_fingerprint)
            .await
            .unwrap();

        let save = capture_game_save(
            &state,
            "Quick Save".to_string(),
            Some("quick_save_0".to_string()),
        )
        .await
        .unwrap();
        assert_eq!(save.save_id, "quick_save_0");
        assert_eq!(save.variables["score"], serde_json::json!(7));
        assert_eq!(save.characters["sakura"].memory_count, 1);
        assert!(save.story_progress.is_some());

        {
            let character = state
                .character_manager
                .read()
                .await
                .get_character("sakura")
                .unwrap();
            let mut character = character.write().await;
            character.set_emotion("sad");
            character.relationships.clear();
            character.memory = Default::default();
        }
        *state.active_scene_id.write().await = Some("elsewhere".to_string());
        state.chat_sessions.write().await.clear();
        *state.story_progress.write().await = StoryProgressState::default();
        state
            .script_engine
            .read()
            .await
            .load_state(HashMap::new(), HashMap::new())
            .unwrap();

        let summary = restore_game_save(&state, save).await.unwrap();
        assert_eq!(
            summary,
            SaveRestoreSummary {
                character_count: 1,
                chat_session_count: 1,
                applied_event_count: 1,
            }
        );
        assert_eq!(
            state.active_scene_id.read().await.as_deref(),
            Some("riverbank")
        );
        assert_eq!(state.scene_history.read().await.len(), 2);
        assert!(state
            .story_progress
            .read()
            .await
            .unlocked_scene_ids
            .contains("friend_scene"));
        assert_eq!(
            state.chat_sessions.read().await["sakura"].evaluation_count,
            1
        );
        assert_eq!(
            state
                .script_engine
                .read()
                .await
                .get_variable("score")
                .unwrap()
                .clone_cast::<i64>(),
            7
        );
        let character = state
            .character_manager
            .read()
            .await
            .get_character("sakura")
            .unwrap();
        let character = character.read().await;
        assert_eq!(character.emotion, "happy");
        assert_eq!(character.relationships["player"], 0.65);
        assert_eq!(character.memory.len(), 1);
    }

    #[tokio::test]
    async fn v2_save_migrates_triggered_events_into_story_progress() {
        let state = AppState::new();
        state
            .character_manager
            .write()
            .await
            .add_character(Character::new("sakura", "Sakura"));
        let mut session = ChatSession::new("sakura".to_string());
        session.triggered_event_ids = vec!["first_friend".to_string()];
        let mut save = SaveManager::create_save("Legacy V2", None, None, None);
        save.schema = GAME_SAVE_SCHEMA_V2.to_string();
        save.story_progress = None;
        save.chat_sessions
            .insert("sakura".to_string(), serde_json::to_value(session).unwrap());

        let summary = restore_game_save(&state, save).await.unwrap();

        assert_eq!(summary.applied_event_count, 1);
        assert!(state
            .story_progress
            .read()
            .await
            .unlocked_scene_ids
            .contains("friend_scene"));
    }

    #[tokio::test]
    async fn invalid_story_progress_is_rejected_before_runtime_mutation() {
        let state = AppState::new();
        state
            .story_progress
            .write()
            .await
            .unlocked_scene_ids
            .insert("keep_scene".to_string());
        let mut save = SaveManager::create_save("Invalid Progress", None, None, None);
        save.story_progress = Some(serde_json::json!({ "schema": "unsupported" }));

        assert!(restore_game_save(&state, save).await.is_err());
        assert!(state
            .story_progress
            .read()
            .await
            .unlocked_scene_ids
            .contains("keep_scene"));
    }
}
