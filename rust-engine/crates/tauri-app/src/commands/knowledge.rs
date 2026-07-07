//! Knowledge base commands.

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct KnowledgeResult {
    pub id: String,
    pub category: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub importance: f32,
}

/// Search the knowledge base.
#[tauri::command]
pub async fn search_knowledge(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<KnowledgeResult>, String> {
    let kb = state.knowledge_base.read().await;
    let results = kb.search(&query, limit.unwrap_or(10));

    Ok(results
        .into_iter()
        .map(|e| KnowledgeResult {
            id: e.id.clone(),
            category: format!("{:?}", e.category),
            title: e.title.clone(),
            content: e.content.clone(),
            tags: e.tags.clone(),
            importance: e.importance,
        })
        .collect())
}

/// Load knowledge entries from a directory.
#[tauri::command]
pub async fn load_knowledge(
    state: State<'_, AppState>,
    directory: String,
) -> Result<usize, String> {
    let path = std::path::PathBuf::from(&directory);
    let mut kb = state.knowledge_base.write().await;
    kb.load_from_directory(&path)
        .await
        .map_err(|e| e.to_string())
}

/// List all knowledge entries (returns full metadata for each entry).
#[tauri::command]
pub async fn list_knowledge_entries(
    state: State<'_, AppState>,
) -> Result<Vec<KnowledgeResult>, String> {
    let kb = state.knowledge_base.read().await;
    let entries = kb.all_entries();

    Ok(entries
        .into_iter()
        .map(|e| KnowledgeResult {
            id: e.id.clone(),
            category: format!("{:?}", e.category),
            title: e.title.clone(),
            content: e.content.clone(),
            tags: e.tags.clone(),
            importance: e.importance,
        })
        .collect())
}

/// Get a single knowledge entry by ID.
#[tauri::command]
pub async fn get_knowledge_entry(
    state: State<'_, AppState>,
    entry_id: String,
) -> Result<Option<KnowledgeResult>, String> {
    let kb = state.knowledge_base.read().await;
    Ok(kb.get_entry(&entry_id).map(|e| KnowledgeResult {
        id: e.id.clone(),
        category: format!("{:?}", e.category),
        title: e.title.clone(),
        content: e.content.clone(),
        tags: e.tags.clone(),
        importance: e.importance,
    }))
}

/// Get all unique tags in the knowledge base.
#[tauri::command]
pub async fn list_knowledge_tags(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let kb = state.knowledge_base.read().await;
    Ok(kb.all_tags())
}
