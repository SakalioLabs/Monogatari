//! Analytics engine for player behavior tracking.
//!
//! Collects anonymized gameplay metrics in memory with file persistence
//! to help creators understand player engagement and conversation patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_type: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_events: usize,
    pub session_count: usize,
    pub avg_session_duration_ms: u64,
    pub top_choices: Vec<(String, u32)>,
    pub top_characters: Vec<(String, u32)>,
    pub conversation_count: u32,
    pub avg_relationship_score: f32,
}

type ProjectAnalyticsStore = Arc<RwLock<HashMap<PathBuf, Vec<AnalyticsEvent>>>>;

/// Project-scoped analytics event stores (in-memory, persisted to disk).
static ANALYTICS_STORE: once_cell::sync::Lazy<ProjectAnalyticsStore> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

fn analytics_file_path(project_root: &Path) -> PathBuf {
    project_root.join("analytics.json")
}

async fn load_events_from_disk(project_root: &Path) -> Vec<AnalyticsEvent> {
    let path = analytics_file_path(project_root);
    if path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            if let Ok(events) = serde_json::from_str::<Vec<AnalyticsEvent>>(&content) {
                return events;
            }
        }
    }
    Vec::new()
}

async fn persist_events(project_root: &Path, events: &[AnalyticsEvent]) {
    let path = analytics_file_path(project_root);
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    if let Ok(json) = serde_json::to_string_pretty(events) {
        let _ = tokio::fs::write(&path, json).await;
    }
}

/// Record an analytics event with persistence.
#[tauri::command]
pub async fn record_analytics_event(
    state: State<'_, AppState>,
    event_type: String,
    data: serde_json::Value,
) -> Result<String, String> {
    let project_root = state.current_project_data_root().await;
    let event = AnalyticsEvent {
        event_type: event_type.clone(),
        timestamp: format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ),
        data,
    };
    let mut store = ANALYTICS_STORE.write().await;
    if !store.contains_key(&project_root) {
        let events = load_events_from_disk(&project_root).await;
        store.insert(project_root.clone(), events);
    }
    let events = store.get_mut(&project_root).expect("analytics store entry");
    events.push(event);
    tracing::debug!(
        "Analytics event recorded: {} (total: {})",
        event_type,
        events.len()
    );
    persist_events(&project_root, events).await;
    Ok("recorded".to_string())
}

/// Get analytics summary from recorded events.
#[tauri::command]
pub async fn get_analytics_summary(state: State<'_, AppState>) -> Result<AnalyticsSummary, String> {
    let project_root = state.current_project_data_root().await;
    let mut store = ANALYTICS_STORE.write().await;
    if !store.contains_key(&project_root) {
        let events = load_events_from_disk(&project_root).await;
        store.insert(project_root.clone(), events);
    }
    let events = store.get(&project_root).cloned().unwrap_or_default();
    drop(store);
    let total = events.len();

    // Count characters
    let mut char_counts: HashMap<String, u32> = HashMap::new();
    let mut choice_counts: HashMap<String, u32> = HashMap::new();
    let mut session_ids: HashMap<String, u64> = HashMap::new();
    let mut conv_count: u32 = 0;

    for event in events.iter() {
        match event.event_type.as_str() {
            "chat_message" => {
                if let Some(cid) = event.data.get("character_id").and_then(|v| v.as_str()) {
                    *char_counts.entry(cid.to_string()).or_insert(0) += 1;
                }
                conv_count += 1;
            }
            "choice_selected" => {
                if let Some(text) = event.data.get("choice_text").and_then(|v| v.as_str()) {
                    *choice_counts.entry(text.to_string()).or_insert(0) += 1;
                }
            }
            "session_start" => {
                if let Some(sid) = event.data.get("session_id").and_then(|v| v.as_str()) {
                    let ts = event.timestamp.parse::<u64>().unwrap_or(0);
                    session_ids.insert(sid.to_string(), ts);
                }
            }
            _ => {}
        }
    }

    let mut top_characters: Vec<(String, u32)> = char_counts.into_iter().collect();
    top_characters.sort_by_key(|entry| std::cmp::Reverse(entry.1));
    top_characters.truncate(10);

    let mut top_choices: Vec<(String, u32)> = choice_counts.into_iter().collect();
    top_choices.sort_by_key(|entry| std::cmp::Reverse(entry.1));
    top_choices.truncate(10);

    // Get relationship score
    let avg_rel = {
        let cm = state.character_manager.read().await;
        let chars = cm.character_ids();
        if chars.is_empty() {
            0.0
        } else {
            let mut total_rel: f32 = 0.0;
            let mut count = 0;
            for cid in &chars {
                if let Some(ch) = cm.get_character(cid) {
                    let c = ch.read().await;
                    if let Some(score) = c.relationships.get("player") {
                        total_rel += score;
                        count += 1;
                    }
                }
            }
            if count > 0 {
                total_rel / count as f32
            } else {
                0.0
            }
        }
    };

    Ok(AnalyticsSummary {
        total_events: total,
        session_count: session_ids.len(),
        avg_session_duration_ms: 0,
        top_choices,
        top_characters,
        conversation_count: conv_count,
        avg_relationship_score: avg_rel,
    })
}

/// Export analytics data as JSON.
#[tauri::command]
pub async fn export_analytics(
    state: State<'_, AppState>,
    _format: Option<String>,
) -> Result<String, String> {
    let project_root = state.current_project_data_root().await;
    let mut store = ANALYTICS_STORE.write().await;
    if !store.contains_key(&project_root) {
        let events = load_events_from_disk(&project_root).await;
        store.insert(project_root.clone(), events);
    }
    let events = store.get(&project_root).cloned().unwrap_or_default();
    serde_json::to_string_pretty(&events).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytics_file_stays_inside_project_root() {
        let root = PathBuf::from("project-data");
        assert_eq!(analytics_file_path(&root), root.join("analytics.json"));
    }
}
