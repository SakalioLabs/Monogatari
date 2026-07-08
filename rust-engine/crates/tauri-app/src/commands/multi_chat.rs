//! Multi-character simultaneous chat commands.
//!
//! Allows players to interact with multiple characters in a shared conversation,
//! where characters can react to each other and the player.

use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::debug;

use crate::commands::{chat, prompt_guard};
use crate::state::AppState;

/// A message in a multi-character conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatMessage {
    pub role: String, // "player" or "character"
    pub character_id: Option<String>,
    pub character_name: String,
    pub content: String,
    pub emotion: Option<String>,
    pub timestamp: String,
}

/// A group chat session with multiple characters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatSession {
    pub session_id: String,
    pub character_ids: Vec<String>,
    pub messages: Vec<GroupChatMessage>,
    pub active: bool,
}

impl GroupChatSession {
    pub fn new(session_id: String, character_ids: Vec<String>) -> Self {
        Self {
            session_id,
            character_ids,
            messages: Vec::new(),
            active: true,
        }
    }
}

/// Start a group chat session with multiple characters.
#[tauri::command]
pub async fn start_group_chat(
    state: State<'_, AppState>,
    character_ids: Vec<String>,
) -> Result<GroupChatSession, String> {
    if character_ids.len() < 2 {
        return Err("Group chat requires at least 2 characters.".to_string());
    }

    // Verify all characters exist
    let cm = state.character_manager.read().await;
    for id in &character_ids {
        if cm.get_character(id).is_none() {
            return Err(format!("Character not found: {id}"));
        }
    }

    let session_id = format!(
        "group_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let session = GroupChatSession::new(session_id, character_ids);
    Ok(session)
}

/// Send a message in a group chat and get responses from all characters.
#[tauri::command]
pub async fn send_group_message(
    state: State<'_, AppState>,
    session: GroupChatSession,
    message: String,
) -> Result<GroupChatSession, String> {
    let mut updated = session;
    let now = format!(
        "{:?}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // Add player message
    updated.messages.push(GroupChatMessage {
        role: "player".to_string(),
        character_id: None,
        character_name: "Player".to_string(),
        content: message.clone(),
        emotion: None,
        timestamp: now,
    });

    // Generate responses from each character
    for char_id in &updated.character_ids {
        let (char_name, char_profile, char_emotion, knowledge_refs) = {
            let cm = state.character_manager.read().await;
            if let Some(character) = cm.get_character(char_id) {
                let c = character.read().await;
                (
                    c.name.clone(),
                    c.build_system_prompt(),
                    c.emotion.clone(),
                    c.knowledge_refs.clone(),
                )
            } else {
                continue;
            }
        };

        let knowledge_context = {
            let kb = state.knowledge_base.read().await;
            chat::build_character_knowledge_context(&kb, &message, &knowledge_refs, 2)
        };

        // Build context with other characters' recent messages
        let other_context: Vec<String> = updated
            .messages
            .iter()
            .rev()
            .take(15)
            .rev()
            .map(|m| {
                if m.role == "player" {
                    prompt_guard::transcript_line("Player", &m.content)
                } else {
                    prompt_guard::transcript_line(&m.character_name, &m.content)
                }
            })
            .collect();

        let guard_notice = prompt_guard::latest_input_notice(&message);
        let system_prompt = format!(
            r#"You are "{name}" in a group conversation in a visual novel game.
{profile}

Other characters are also present. React naturally to what they say.
Stay in character. Keep responses concise (1-2 sentences).
Show emotion through *actions*.
{mind_contract}
{guard_contract}
{guard_notice}
{knowledge}"#,
            name = char_name,
            profile = char_profile,
            mind_contract = prompt_guard::character_mind_contract(),
            guard_contract = prompt_guard::character_safety_contract(),
            guard_notice = guard_notice,
            knowledge = if knowledge_context.is_empty() {
                String::new()
            } else {
                format!("\nContext:\n{}", knowledge_context)
            },
        );

        let conversation = other_context.join("\n");
        let full_prompt = format!(
            "[System]\n{system_prompt}\n\n[User]\nTRANSCRIPT_BEGIN\n{conversation}\nTRANSCRIPT_END\n\n[Assistant]\n"
        );

        let pipeline = state.inference_pipeline.read().await;
        let options = llm_ai::InferenceOptions {
            max_tokens: 150,
            temperature: 0.85,
            ..Default::default()
        };

        match pipeline.generate_response(&full_prompt, &options).await {
            Ok(result) if result.success => {
                let response_text =
                    prompt_guard::guard_character_response(&char_name, &result.text);
                debug!("Group chat response from {}: {}", char_name, response_text);
                updated.messages.push(GroupChatMessage {
                    role: "character".to_string(),
                    character_id: Some(char_id.clone()),
                    character_name: char_name,
                    content: response_text,
                    emotion: Some(char_emotion),
                    timestamp: format!(
                        "{:?}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    ),
                });
            }
            _ => {
                debug!("Failed to get response from {}", char_id);
            }
        }
    }

    Ok(updated)
}

/// Get available characters for group chat.
#[tauri::command]
pub async fn get_group_chat_characters(
    state: State<'_, AppState>,
) -> Result<Vec<(String, String)>, String> {
    let cm = state.character_manager.read().await;
    let ids = cm.character_ids();
    let mut characters = Vec::new();
    for id in ids {
        if let Some(character) = cm.get_character(&id) {
            let c = character.read().await;
            characters.push((id, c.name.clone()));
        }
    }
    Ok(characters)
}
