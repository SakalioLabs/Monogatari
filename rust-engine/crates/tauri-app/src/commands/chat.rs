//! Free-form chat and LLM evaluation commands.
//!
//! This is the core feature: players chat freely with LLM-driven characters,
//! and the LLM evaluates conversations to trigger special plot events.

use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::debug;

use crate::state::AppState;

/// A single chat message in the conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "player" or "character"
    pub content: String,
    pub emotion: Option<String>,
    pub timestamp: String,
}

/// Result of a free-form chat interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub character_response: String,
    pub emotion: String,
    pub relationship_delta: f32,
    pub evaluation: Option<ConversationEvaluation>,
    pub triggered_events: Vec<TriggeredEvent>,
}

/// LLM evaluation of a conversation segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEvaluation {
    pub friendliness: f32,  // 0.0 - 1.0
    pub engagement: f32,    // 0.0 - 1.0
    pub creativity: f32,    // 0.0 - 1.0
    pub overall_score: f32, // 0.0 - 1.0
    pub summary: String,
}

/// A special event triggered by the evaluation system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredEvent {
    pub event_id: String,
    pub event_type: String, // "special_dialogue", "scene_change", "relationship_milestone", "unlock"
    pub description: String,
    pub data: serde_json::Value,
}

/// Chat session state for a character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub character_id: String,
    pub messages: Vec<ChatMessage>,
    pub cumulative_score: f32,
    pub evaluation_count: u32,
    pub triggered_event_ids: Vec<String>,
}

impl ChatSession {
    pub fn new(character_id: String) -> Self {
        Self {
            character_id,
            messages: Vec::new(),
            cumulative_score: 0.0,
            evaluation_count: 0,
            triggered_event_ids: Vec::new(),
        }
    }
}

/// Send a message to an LLM-driven character and get a response.
///
/// This is the core chat function. It:
/// 1. Builds context from character personality + knowledge base
/// 2. Includes conversation history
/// 3. Calls the LLM pipeline for character response
/// 4. Periodically evaluates the conversation quality
/// 5. Triggers special events when score thresholds are met
#[tauri::command]
pub async fn send_chat_message(
    state: State<'_, AppState>,
    character_id: String,
    message: String,
) -> Result<ChatResponse, String> {
    let now = format!(
        "{:?}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // Get character info
    let (char_name, char_description, char_background, char_personality, char_emotion) = {
        let cm = state.character_manager.read().await;
        if let Some(character) = cm.get_character(&character_id) {
            let c = character.read().await;
            (
                c.name.clone(),
                c.description.clone(),
                c.background.clone(),
                c.personality.to_prompt_description(),
                c.emotion.clone(),
            )
        } else {
            return Err(format!("Character not found: {character_id}"));
        }
    };

    // Get knowledge context
    let knowledge_context = {
        let kb = state.knowledge_base.read().await;
        let entries = kb.search(&message, 3);
        if entries.is_empty() {
            String::new()
        } else {
            let parts: Vec<String> = entries
                .iter()
                .map(|e| format!("{}: {}", e.title, e.content))
                .collect();
            format!("Relevant world knowledge:\n{}", parts.join("\n"))
        }
    };

    // Add player message and snapshot recent history without holding the
    // session write lock during the potentially slow LLM request.
    let history: Vec<String> = {
        let mut sessions = state.chat_sessions.write().await;
        let session = sessions
            .entry(character_id.clone())
            .or_insert_with(|| ChatSession::new(character_id.clone()));

        session.messages.push(ChatMessage {
            role: "player".to_string(),
            content: message.clone(),
            emotion: None,
            timestamp: now.clone(),
        });

        session
            .messages
            .iter()
            .rev()
            .take(10)
            .rev()
            .map(|m| {
                let role = if m.role == "player" {
                    "Player"
                } else {
                    &char_name
                };
                format!("{role}: {}", m.content)
            })
            .collect()
    };

    // Build the full prompt
    let system_prompt = format!(
        r#"You are playing the character "{name}" in a visual novel game.
Description: {desc}
Background: {bg}
Personality: {personality}
Current emotion: {emotion}

You MUST:
- Stay in character at all times
- Respond naturally and emotionally based on your personality
- Reference your background and knowledge when relevant
- Show emotional reactions (use *actions* for body language)
- Keep responses concise (1-3 sentences typically)
- Never break character or acknowledge you are an AI

{knowledge}"#,
        name = char_name,
        desc = char_description,
        bg = char_background,
        personality = char_personality,
        emotion = char_emotion,
        knowledge = if knowledge_context.is_empty() {
            String::new()
        } else {
            format!("\nContext:\n{}", knowledge_context)
        }
    );

    let conversation = history.join("\n");
    let full_prompt =
        format!("[System]\n{system_prompt}\n\n[Conversation]\n{conversation}\n\n[{char_name}]");

    // Call LLM for character response
    let pipeline = state.inference_pipeline.read().await;
    let options = llm_ai::InferenceOptions {
        max_tokens: 300,
        temperature: 0.85,
        ..Default::default()
    };

    let response_text = match pipeline.generate_response(&full_prompt, &options).await {
        Ok(result) if result.success => result.text,
        Ok(result) => {
            return Err(result
                .error
                .unwrap_or_else(|| "AI generation failed".to_string()));
        }
        Err(e) => {
            return Err(format!("AI error: {e}"));
        }
    };

    // Analyze emotion from response (simple keyword matching)
    let detected_emotion = detect_emotion(&response_text, &char_emotion);

    // Calculate relationship delta based on message sentiment
    let relationship_delta = estimate_sentiment(&message);

    // Update character emotion
    {
        let cm = state.character_manager.read().await;
        if let Some(character) = cm.get_character(&character_id) {
            let mut c = character.write().await;
            c.set_emotion(&detected_emotion);
            c.update_relationship("player", relationship_delta * 0.1);
            c.add_memory(
                format!("Player said: {}", message),
                llm_game::characters::memory::MemoryType::Conversation,
                0.5,
                vec!["conversation".to_string()],
            );
        }
    }

    // Add character response and snapshot evaluation inputs.
    let (should_evaluate, messages_for_eval, cumulative_score, evaluation_count, already_triggered) = {
        let mut sessions = state.chat_sessions.write().await;
        let session = sessions
            .entry(character_id.clone())
            .or_insert_with(|| ChatSession::new(character_id.clone()));

        session.messages.push(ChatMessage {
            role: "character".to_string(),
            content: response_text.clone(),
            emotion: Some(detected_emotion.clone()),
            timestamp: format!(
                "{:?}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        });

        let player_msg_count = session
            .messages
            .iter()
            .filter(|m| m.role == "player")
            .count();
        (
            player_msg_count > 0 && player_msg_count % 5 == 0,
            session.messages.clone(),
            session.cumulative_score,
            session.evaluation_count,
            session.triggered_event_ids.clone(),
        )
    };

    // Periodic evaluation (every 5 player messages)
    let mut evaluation = None;
    let mut triggered_events = Vec::new();

    if should_evaluate {
        let eval_result = evaluate_conversation_internal(
            &state,
            &char_name,
            &messages_for_eval,
            cumulative_score,
            evaluation_count,
        )
        .await;

        if let Ok(eval) = eval_result {
            let next_cumulative_score = cumulative_score + eval.overall_score;
            let next_evaluation_count = evaluation_count + 1;
            evaluation = Some(eval.clone());

            // Check for triggered events
            let events = check_event_triggers(
                &state,
                &character_id,
                &eval,
                next_cumulative_score,
                next_evaluation_count,
                &already_triggered,
            )
            .await;

            {
                let mut sessions = state.chat_sessions.write().await;
                let session = sessions
                    .entry(character_id.clone())
                    .or_insert_with(|| ChatSession::new(character_id.clone()));
                session.cumulative_score += eval.overall_score;
                session.evaluation_count += 1;
                for event in &events {
                    if !session.triggered_event_ids.contains(&event.event_id) {
                        session.triggered_event_ids.push(event.event_id.clone());
                    }
                }
            }
            triggered_events = events;
        }
    }

    Ok(ChatResponse {
        character_response: response_text,
        emotion: detected_emotion,
        relationship_delta,
        evaluation,
        triggered_events,
    })
}

/// Get the chat history for a character.
#[tauri::command]
pub async fn get_chat_history(
    state: State<'_, AppState>,
    character_id: String,
) -> Result<Vec<ChatMessage>, String> {
    let sessions = state.chat_sessions.read().await;
    Ok(sessions
        .get(&character_id)
        .map(|s| s.messages.clone())
        .unwrap_or_default())
}

/// Clear the chat history for a character.
#[tauri::command]
pub async fn clear_chat_history(
    state: State<'_, AppState>,
    character_id: String,
) -> Result<(), String> {
    let mut sessions = state.chat_sessions.write().await;
    sessions.remove(&character_id);
    Ok(())
}

/// Manually trigger a conversation evaluation.
#[tauri::command]
pub async fn evaluate_conversation(
    state: State<'_, AppState>,
    character_id: String,
) -> Result<ConversationEvaluation, String> {
    let sessions = state.chat_sessions.read().await;
    let session = sessions
        .get(&character_id)
        .ok_or_else(|| format!("No chat session for {character_id}"))?;

    let char_name = {
        let cm = state.character_manager.read().await;
        if let Some(character) = cm.get_character(&character_id) {
            let c = character.read().await;
            c.name.clone()
        } else {
            character_id.clone()
        }
    };

    evaluate_conversation_internal(
        &state,
        &char_name,
        &session.messages,
        session.cumulative_score,
        session.evaluation_count,
    )
    .await
}

/// Get the current relationship score between player and character.
#[tauri::command]
pub async fn get_relationship_score(
    state: State<'_, AppState>,
    character_id: String,
) -> Result<f32, String> {
    let cm = state.character_manager.read().await;
    if let Some(character) = cm.get_character(&character_id) {
        let c = character.read().await;
        Ok(c.relationships.get("player").copied().unwrap_or(0.0))
    } else {
        Err(format!("Character not found: {character_id}"))
    }
}

/// Get special events that are available based on current scores.
#[tauri::command]
pub async fn get_available_events(
    state: State<'_, AppState>,
    character_id: String,
) -> Result<Vec<TriggeredEvent>, String> {
    let sessions = state.chat_sessions.read().await;
    let session = sessions
        .get(&character_id)
        .ok_or_else(|| format!("No chat session for {character_id}"))?;

    let events = get_event_definitions();
    let available: Vec<TriggeredEvent> = events
        .into_iter()
        .filter(|e| {
            !session.triggered_event_ids.contains(&e.event_id)
                && check_event_condition(
                    e.event_type.as_str(),
                    session.cumulative_score,
                    session.evaluation_count,
                )
        })
        .collect();

    Ok(available)
}

// === Internal helper functions ===

/// Use LLM to evaluate the conversation quality.
async fn evaluate_conversation_internal(
    state: &State<'_, AppState>,
    character_name: &str,
    messages: &[ChatMessage],
    _cumulative_score: f32,
    _eval_count: u32,
) -> Result<ConversationEvaluation, String> {
    let conversation: Vec<String> = messages
        .iter()
        .rev()
        .take(20)
        .rev()
        .map(|m| {
            let role = if m.role == "player" {
                "Player"
            } else {
                character_name
            };
            format!("{role}: {}", m.content)
        })
        .collect();

    let eval_prompt = format!(
        r#"[System]
You are a conversation quality evaluator for a visual novel game.
Evaluate the player's conversation with character "{name}".

Conversation:
{conversation}

Rate the following aspects from 0.0 to 1.0:
- friendliness: How kind and friendly is the player?
- engagement: How engaged and interesting are the player's messages?
- creativity: How creative and thoughtful are the player's responses?

Respond ONLY in this exact JSON format:
{{"friendliness": 0.0, "engagement": 0.0, "creativity": 0.0, "summary": "brief summary"}}

[Assistant]
{{"#,
        name = character_name,
        conversation = conversation.join("\n")
    );

    let pipeline = state.inference_pipeline.read().await;
    let options = llm_ai::InferenceOptions {
        max_tokens: 200,
        temperature: 0.3,
        ..Default::default()
    };

    match pipeline.generate_response(&eval_prompt, &options).await {
        Ok(result) if result.success => {
            // Try to parse the JSON response
            let json_str = format!("{{{}}}", result.text);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                let friendliness = parsed["friendliness"].as_f64().unwrap_or(0.5) as f32;
                let engagement = parsed["engagement"].as_f64().unwrap_or(0.5) as f32;
                let creativity = parsed["creativity"].as_f64().unwrap_or(0.5) as f32;
                let summary = parsed["summary"]
                    .as_str()
                    .unwrap_or("No summary")
                    .to_string();
                let overall = (friendliness + engagement + creativity) / 3.0;

                Ok(ConversationEvaluation {
                    friendliness,
                    engagement,
                    creativity,
                    overall_score: overall,
                    summary,
                })
            } else {
                // Fallback if JSON parsing fails
                Ok(ConversationEvaluation {
                    friendliness: 0.5,
                    engagement: 0.5,
                    creativity: 0.5,
                    overall_score: 0.5,
                    summary: "Evaluation parsing failed".to_string(),
                })
            }
        }
        _ => Ok(ConversationEvaluation {
            friendliness: 0.5,
            engagement: 0.5,
            creativity: 0.5,
            overall_score: 0.5,
            summary: "Could not evaluate".to_string(),
        }),
    }
}

/// Check if any special events should be triggered.
async fn check_event_triggers(
    state: &State<'_, AppState>,
    character_id: &str,
    eval: &ConversationEvaluation,
    _cumulative_score: f32,
    eval_count: u32,
    already_triggered: &[String],
) -> Vec<TriggeredEvent> {
    let mut events = Vec::new();
    let definitions = get_event_definitions();

    for def in definitions {
        if already_triggered.contains(&def.event_id) {
            continue;
        }

        let should_trigger = match def.event_type.as_str() {
            "relationship_milestone" => {
                let relationship = {
                    let cm = state.character_manager.read().await;
                    if let Some(character) = cm.get_character(character_id) {
                        let c = character.read().await;
                        c.relationships.get("player").copied().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                        .unwrap_or(0.0)
                };
                match def.event_id.as_str() {
                    "first_friend" => relationship >= 0.3,
                    "close_friend" => relationship >= 0.6,
                    "best_friend" => relationship >= 0.8,
                    _ => false,
                }
            }
            "special_dialogue" => match def.event_id.as_str() {
                "high_engagement" => eval.engagement > 0.8 && eval_count >= 2,
                "creative_talk" => eval.creativity > 0.8 && eval_count >= 2,
                _ => false,
            },
            "cumulative_achievement" => match def.event_id.as_str() {
                "dedicated_player" => eval_count >= 5,
                "super_dedicated" => eval_count >= 10,
                _ => false,
            },
            _ => false,
        };

        if should_trigger {
            debug!("Triggered event: {}", def.event_id);
            events.push(def);
        }
    }

    events
}

/// Define all possible special events in the game.
fn get_event_definitions() -> Vec<TriggeredEvent> {
    vec![
        TriggeredEvent {
            event_id: "first_friend".to_string(),
            event_type: "relationship_milestone".to_string(),
            description: "You've become friends with the character!".to_string(),
            data: serde_json::json!({
                "unlock_scene": "friend_scene",
                "dialogue_id": "friend_celebration"
            }),
        },
        TriggeredEvent {
            event_id: "close_friend".to_string(),
            event_type: "relationship_milestone".to_string(),
            description: "You've become close friends!".to_string(),
            data: serde_json::json!({
                "unlock_scene": "close_friend_scene",
                "dialogue_id": "close_friend_dialogue"
            }),
        },
        TriggeredEvent {
            event_id: "best_friend".to_string(),
            event_type: "relationship_milestone".to_string(),
            description: "You've become best friends! A special ending is now available."
                .to_string(),
            data: serde_json::json!({
                "unlock_ending": "best_friend_ending",
                "dialogue_id": "best_friend_dialogue"
            }),
        },
        TriggeredEvent {
            event_id: "high_engagement".to_string(),
            event_type: "special_dialogue".to_string(),
            description: "Your engaging conversation has unlocked a special dialogue!".to_string(),
            data: serde_json::json!({
                "dialogue_id": "engaged_conversation"
            }),
        },
        TriggeredEvent {
            event_id: "creative_talk".to_string(),
            event_type: "special_dialogue".to_string(),
            description: "Your creative responses have impressed the character!".to_string(),
            data: serde_json::json!({
                "dialogue_id": "impressed_character"
            }),
        },
        TriggeredEvent {
            event_id: "dedicated_player".to_string(),
            event_type: "cumulative_achievement".to_string(),
            description: "You've had many conversations! New content unlocked.".to_string(),
            data: serde_json::json!({
                "unlock_scene": "memories_scene"
            }),
        },
        TriggeredEvent {
            event_id: "super_dedicated".to_string(),
            event_type: "cumulative_achievement".to_string(),
            description: "Your dedication is remarkable! A secret scene is available.".to_string(),
            data: serde_json::json!({
                "unlock_scene": "secret_scene"
            }),
        },
    ]
}

/// Check if an event condition is met.
fn check_event_condition(event_type: &str, cumulative_score: f32, eval_count: u32) -> bool {
    match event_type {
        "relationship_milestone" => cumulative_score > 1.0,
        "special_dialogue" => eval_count >= 2,
        "cumulative_achievement" => eval_count >= 5,
        _ => false,
    }
}

/// Simple emotion detection from response text.
fn detect_emotion(text: &str, current: &str) -> String {
    let lower = text.to_lowercase();

    if lower.contains("happy")
        || lower.contains("smile")
        || lower.contains("laugh")
        || lower.contains("grin")
        || lower.contains("joy")
        || lower.contains("*beam")
    {
        "happy".to_string()
    } else if lower.contains("sad")
        || lower.contains("cry")
        || lower.contains("tear")
        || lower.contains("sigh")
        || lower.contains("melancholy")
    {
        "sad".to_string()
    } else if lower.contains("angry")
        || lower.contains("frown")
        || lower.contains("glare")
        || lower.contains("stern")
        || lower.contains("mad")
    {
        "angry".to_string()
    } else if lower.contains("surprise")
        || lower.contains("gasp")
        || lower.contains("shock")
        || lower.contains("oh!")
        || lower.contains("wow")
    {
        "surprised".to_string()
    } else if lower.contains("love")
        || lower.contains("heart")
        || lower.contains("blush")
        || lower.contains("adore")
    {
        "love".to_string()
    } else if lower.contains("embarrass") || lower.contains("shy") || lower.contains("fluster") {
        "embarrassed".to_string()
    } else {
        current.to_string()
    }
}

/// Simple sentiment estimation from player message.
fn estimate_sentiment(message: &str) -> f32 {
    let lower = message.to_lowercase();
    let mut score: f32 = 0.0;

    // Positive words
    for word in &[
        "love",
        "happy",
        "great",
        "amazing",
        "wonderful",
        "beautiful",
        "kind",
        "nice",
        "good",
        "like",
        "enjoy",
        "fun",
        "thank",
        "cute",
        "sweet",
        "pretty",
        "awesome",
        "cool",
        "friend",
    ] {
        if lower.contains(word) {
            score += 0.15;
        }
    }

    // Negative words
    for word in &[
        "hate",
        "bad",
        "ugly",
        "stupid",
        "boring",
        "annoying",
        "dumb",
        "worst",
        "terrible",
        "horrible",
        "disgusting",
        "idiot",
    ] {
        if lower.contains(word) {
            score -= 0.2;
        }
    }

    // Questions show engagement
    if message.contains('?') {
        score += 0.05;
    }

    // Longer messages show more engagement
    if message.len() > 50 {
        score += 0.05;
    }

    score.clamp(-0.5, 0.5)
}

/// Send a streaming chat message. Chunks are emitted via Tauri events.
#[tauri::command]
pub async fn send_chat_message_stream(
    state: State<'_, AppState>,
    character_id: String,
    message: String,
    window: tauri::WebviewWindow,
) -> Result<(), String> {
    use tauri::Emitter;

    let (char_name, char_description, char_background, char_personality, char_emotion) = {
        let cm = state.character_manager.read().await;
        if let Some(character) = cm.get_character(&character_id) {
            let c = character.read().await;
            (
                c.name.clone(),
                c.description.clone(),
                c.background.clone(),
                c.personality.to_prompt_description(),
                c.emotion.clone(),
            )
        } else {
            return Err(format!("Character not found: {character_id}"));
        }
    };

    let knowledge_context = {
        let kb = state.knowledge_base.read().await;
        let entries = kb.search(&message, 3);
        if entries.is_empty() {
            String::new()
        } else {
            let parts: Vec<String> = entries
                .iter()
                .map(|e| format!("{}: {}", e.title, e.content))
                .collect();
            format!("Relevant world knowledge:\n{}", parts.join("\n"))
        }
    };

    let now = format!(
        "{:?}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let history: Vec<String> = {
        let mut sessions = state.chat_sessions.write().await;
        let session = sessions
            .entry(character_id.clone())
            .or_insert_with(|| ChatSession::new(character_id.clone()));
        session.messages.push(ChatMessage {
            role: "player".to_string(),
            content: message.clone(),
            emotion: None,
            timestamp: now,
        });

        session
            .messages
            .iter()
            .rev()
            .take(10)
            .rev()
            .map(|m| {
                let role = if m.role == "player" {
                    "Player"
                } else {
                    &char_name
                };
                format!("{role}: {}", m.content)
            })
            .collect()
    };

    let system_prompt = format!(
        r#"You are the character "{name}" in a visual novel game.
Description: {desc}
Background: {bg}
Personality: {personality}
Current emotion: {emotion}
Stay in character. Respond naturally with emotion (use *actions* for body language). Keep responses 1-3 sentences.
{knowledge}"#,
        name = char_name,
        desc = char_description,
        bg = char_background,
        personality = char_personality,
        emotion = char_emotion,
        knowledge = if knowledge_context.is_empty() {
            String::new()
        } else {
            format!("\nContext:\n{}", knowledge_context)
        }
    );

    let conversation = history.join("\n");
    let full_prompt =
        format!("[System]\n{system_prompt}\n\n[Conversation]\n{conversation}\n\n[{char_name}]");

    let pipeline = state.inference_pipeline.read().await;
    let options = llm_ai::InferenceOptions {
        max_tokens: 300,
        temperature: 0.85,
        ..Default::default()
    };

    let window_clone = window.clone();
    let on_chunk = Box::new(move |chunk: String| {
        let _ = window_clone.emit("chat-chunk", &chunk);
    });

    let result = pipeline
        .generate_stream(&full_prompt, &options, on_chunk)
        .await
        .map_err(|e| format!("AI error: {e}"))?;

    if result.success {
        let _ = window.emit("chat-complete", &result.text);

        let detected_emotion = detect_emotion(&result.text, &char_emotion);
        let relationship_delta = estimate_sentiment(&message);

        let (
            should_evaluate,
            messages_for_eval,
            cumulative_score,
            evaluation_count,
            already_triggered,
        ) = {
            let mut sessions = state.chat_sessions.write().await;
            let session = sessions
                .entry(character_id.clone())
                .or_insert_with(|| ChatSession::new(character_id.clone()));
            session.messages.push(ChatMessage {
                role: "character".to_string(),
                content: result.text.clone(),
                emotion: Some(detected_emotion.clone()),
                timestamp: format!(
                    "{:?}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                ),
            });

            let player_msg_count = session
                .messages
                .iter()
                .filter(|m| m.role == "player")
                .count();
            (
                player_msg_count > 0 && player_msg_count % 5 == 0,
                session.messages.clone(),
                session.cumulative_score,
                session.evaluation_count,
                session.triggered_event_ids.clone(),
            )
        };

        {
            let cm = state.character_manager.read().await;
            if let Some(character) = cm.get_character(&character_id) {
                let mut c = character.write().await;
                c.set_emotion(&detected_emotion);
                c.update_relationship("player", relationship_delta * 0.1);
                c.add_memory(
                    format!("Player said: {}", message),
                    llm_game::characters::memory::MemoryType::Conversation,
                    0.5,
                    vec!["conversation".to_string()],
                );
            }
        }

        let mut evaluation = None;
        let mut triggered_events = Vec::new();

        if should_evaluate {
            let eval_result = evaluate_conversation_internal(
                &state,
                &char_name,
                &messages_for_eval,
                cumulative_score,
                evaluation_count,
            )
            .await;

            if let Ok(eval) = eval_result {
                let next_cumulative_score = cumulative_score + eval.overall_score;
                let next_evaluation_count = evaluation_count + 1;
                let events = check_event_triggers(
                    &state,
                    &character_id,
                    &eval,
                    next_cumulative_score,
                    next_evaluation_count,
                    &already_triggered,
                )
                .await;

                {
                    let mut sessions = state.chat_sessions.write().await;
                    let session = sessions
                        .entry(character_id.clone())
                        .or_insert_with(|| ChatSession::new(character_id.clone()));
                    session.cumulative_score += eval.overall_score;
                    session.evaluation_count += 1;
                    for event in &events {
                        if !session.triggered_event_ids.contains(&event.event_id) {
                            session.triggered_event_ids.push(event.event_id.clone());
                        }
                    }
                }

                evaluation = Some(eval);
                triggered_events = events;
            }
        }

        let _ = window.emit("chat-emotion", &detected_emotion);
        let _ = window.emit("chat-relationship", &relationship_delta);
        if let Some(eval) = evaluation {
            let _ = window.emit("chat-evaluation", &eval);
        }
        if !triggered_events.is_empty() {
            let _ = window.emit("chat-events", &triggered_events);
        }
    } else {
        let _ = window.emit(
            "chat-error",
            &result
                .error
                .unwrap_or_else(|| "Generation failed".to_string()),
        );
    }

    Ok(())
}
