//! Knowledge base commands.

use std::path::Path;

use llm_authoring::filesystem::{
    ensure_regular_project_directory, stage_json_deletion, stage_json_replacement,
};
pub use llm_authoring::knowledge_documents::KnowledgeAuthoringCatalogSnapshot;
use llm_authoring::knowledge_documents::{
    knowledge_base_from_documents, knowledge_catalog_fingerprint, load_knowledge_documents,
    snapshot_from_documents, LoadedKnowledgeDocument, MAX_KNOWLEDGE_FILE_BYTES,
};
use llm_authoring::knowledge_validation::{
    ensure_valid_knowledge_catalog, ensure_valid_knowledge_id, normalize_knowledge_entry,
};
use llm_game::knowledge::KnowledgeEntry;
use llm_game::KnowledgeBase;
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::commands::content_paths::resolve_project_content_dir;
use crate::content_references::knowledge_references;
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

    Ok(results.into_iter().map(knowledge_result).collect())
}

/// Load knowledge entries from a directory.
#[tauri::command]
pub async fn load_knowledge(
    state: State<'_, AppState>,
    directory: String,
) -> Result<usize, String> {
    let path = resolve_project_content_dir(&state, &directory, "knowledge").await?;
    let project_root = state.current_project_data_root().await;
    let documents =
        load_knowledge_documents(&project_root, &path).map_err(|error| error.to_string())?;
    let count = documents
        .iter()
        .map(|document| document.entries().len())
        .sum();
    *state.knowledge_base.write().await = knowledge_base_from_documents(&documents);
    Ok(count)
}

/// List all knowledge entries.
#[tauri::command]
pub async fn list_knowledge_entries(
    state: State<'_, AppState>,
) -> Result<Vec<KnowledgeResult>, String> {
    let kb = state.knowledge_base.read().await;
    Ok(kb.all_entries().into_iter().map(knowledge_result).collect())
}

/// Get a single knowledge entry by ID.
#[tauri::command]
pub async fn get_knowledge_entry(
    state: State<'_, AppState>,
    entry_id: String,
) -> Result<Option<KnowledgeResult>, String> {
    let kb = state.knowledge_base.read().await;
    Ok(kb.get_entry(&entry_id).map(knowledge_result))
}

/// Get all unique tags in the knowledge base.
#[tauri::command]
pub async fn list_knowledge_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let kb = state.knowledge_base.read().await;
    Ok(kb.all_tags())
}

/// Return a directly authorable snapshot of the active project knowledge catalog.
#[tauri::command]
pub async fn get_knowledge_authoring_catalog(
    state: State<'_, AppState>,
) -> Result<KnowledgeAuthoringCatalogSnapshot, String> {
    knowledge_authoring_snapshot(&state).await
}

/// Atomically create or replace one knowledge entry and hot-reload runtime knowledge.
#[tauri::command]
pub async fn save_knowledge_entry_definition(
    state: State<'_, AppState>,
    entry: KnowledgeEntry,
    original_entry_id: Option<String>,
    expected_catalog_fingerprint: String,
) -> Result<KnowledgeAuthoringCatalogSnapshot, String> {
    save_knowledge_entry_definition_inner(
        &state,
        entry,
        original_entry_id.as_deref(),
        &expected_catalog_fingerprint,
    )
    .await
}

async fn save_knowledge_entry_definition_inner(
    state: &AppState,
    entry: KnowledgeEntry,
    original_entry_id: Option<&str>,
    expected_catalog_fingerprint: &str,
) -> Result<KnowledgeAuthoringCatalogSnapshot, String> {
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    let project_root = state.current_project_data_root().await;
    let documents = load_project_knowledge_documents(&project_root)?;
    require_catalog_fingerprint(
        &knowledge_catalog_fingerprint(&documents),
        expected_catalog_fingerprint,
    )?;

    let entry = normalize_knowledge_entry(entry);
    if let Some(original_id) = original_entry_id {
        if original_id != entry.id {
            return Err("Knowledge entry ids are immutable after creation.".to_string());
        }
    }

    let existing = documents
        .iter()
        .enumerate()
        .find_map(|(document_index, document)| {
            document
                .entries()
                .iter()
                .position(|item| item.id == entry.id)
                .map(|entry_index| (document_index, entry_index))
        });
    if original_entry_id.is_none() && existing.is_some() {
        return Err(format!("Knowledge entry `{}` already exists.", entry.id));
    }
    if original_entry_id.is_some() && existing.is_none() {
        return Err(format!(
            "Knowledge entry `{}` no longer exists; reload before saving.",
            entry.id
        ));
    }
    let mut candidate_entries = documents
        .iter()
        .flat_map(|document| document.entries().iter().cloned())
        .filter(|item| item.id != entry.id)
        .collect::<Vec<_>>();
    candidate_entries.push(entry.clone());
    ensure_valid_knowledge_catalog(&candidate_entries)?;

    let (target_path, output_value) = if let Some((document_index, entry_index)) = existing {
        let document = &documents[document_index];
        (
            document.absolute_path().to_path_buf(),
            document.replacing_entry(entry_index, &entry)?,
        )
    } else {
        let knowledge_root =
            ensure_regular_project_directory(&project_root, "knowledge", "knowledge").await?;
        let target_path = knowledge_root.join(format!("{}.json", entry.id));
        if target_path.exists() {
            return Err(format!(
                "Knowledge file `{}` already exists; reload before saving.",
                target_path.display()
            ));
        }
        (
            target_path,
            serde_json::to_value(&entry).map_err(|error| error.to_string())?,
        )
    };

    let content = pretty_json(&output_value)?;
    let staged = stage_json_replacement(
        &target_path,
        content.as_bytes(),
        MAX_KNOWLEDGE_FILE_BYTES,
        "knowledge entry",
    )
    .await?;

    let reload = reload_knowledge_state(&project_root);
    let (documents, knowledge_base) = match reload {
        Ok(value) => value,
        Err(error) => {
            staged.rollback().await?;
            return Err(format!(
                "Saved knowledge entry failed project reload and was rolled back: {error}"
            ));
        }
    };
    if !documents
        .iter()
        .flat_map(|document| document.entries().iter())
        .any(|item| item.id == entry.id)
    {
        staged.rollback().await?;
        return Err("Saved knowledge entry was not present after reload.".to_string());
    }
    *state.knowledge_base.write().await = knowledge_base;
    staged.commit().await?;
    Ok(snapshot_from_documents(&documents))
}

/// Atomically delete one knowledge entry and hot-reload runtime knowledge.
#[tauri::command]
pub async fn delete_knowledge_entry_definition(
    state: State<'_, AppState>,
    entry_id: String,
    expected_catalog_fingerprint: String,
) -> Result<KnowledgeAuthoringCatalogSnapshot, String> {
    delete_knowledge_entry_definition_inner(&state, &entry_id, &expected_catalog_fingerprint).await
}

async fn delete_knowledge_entry_definition_inner(
    state: &AppState,
    entry_id: &str,
    expected_catalog_fingerprint: &str,
) -> Result<KnowledgeAuthoringCatalogSnapshot, String> {
    ensure_valid_knowledge_id(entry_id)?;
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    let project_root = state.current_project_data_root().await;
    let documents = load_project_knowledge_documents(&project_root)?;
    require_catalog_fingerprint(
        &knowledge_catalog_fingerprint(&documents),
        expected_catalog_fingerprint,
    )?;

    let (document_index, entry_index) = documents
        .iter()
        .enumerate()
        .find_map(|(document_index, document)| {
            document
                .entries()
                .iter()
                .position(|item| item.id == entry_id)
                .map(|entry_index| (document_index, entry_index))
        })
        .ok_or_else(|| format!("Knowledge entry `{entry_id}` does not exist."))?;
    let document = &documents[document_index];
    let mut references = knowledge_references(&project_root, entry_id)?;
    references.extend(
        documents
            .iter()
            .flat_map(|loaded| loaded.entries().iter())
            .filter(|entry| {
                entry.id != entry_id
                    && entry
                        .related_entries
                        .iter()
                        .any(|related_id| related_id == entry_id)
            })
            .map(|entry| format!("knowledge:{}", entry.id)),
    );
    references.sort();
    references.dedup();
    if !references.is_empty() {
        return Err(format!(
            "Knowledge entry `{entry_id}` is still referenced by {}.",
            references.join(", ")
        ));
    }
    let staged = if let Some(output) = document.removing_entry(entry_index)? {
        let content = pretty_json(&output)?;
        stage_json_replacement(
            document.absolute_path(),
            content.as_bytes(),
            MAX_KNOWLEDGE_FILE_BYTES,
            "knowledge entry",
        )
        .await?
    } else {
        stage_json_deletion(document.absolute_path(), "knowledge entry").await?
    };

    let reload = reload_knowledge_state(&project_root);
    let (documents, knowledge_base) = match reload {
        Ok(value) => value,
        Err(error) => {
            staged.rollback().await?;
            return Err(format!(
                "Deleted knowledge entry failed project reload and was rolled back: {error}"
            ));
        }
    };
    if documents
        .iter()
        .flat_map(|loaded| loaded.entries().iter())
        .any(|item| item.id == entry_id)
    {
        staged.rollback().await?;
        return Err("Deleted knowledge entry remained present after reload.".to_string());
    }
    *state.knowledge_base.write().await = knowledge_base;
    staged.commit().await?;
    Ok(snapshot_from_documents(&documents))
}

fn knowledge_result(entry: &KnowledgeEntry) -> KnowledgeResult {
    KnowledgeResult {
        id: entry.id.clone(),
        category: entry.category.as_str().to_string(),
        title: entry.title.clone(),
        content: entry.content.clone(),
        tags: entry.tags.clone(),
        importance: entry.importance,
    }
}

async fn knowledge_authoring_snapshot(
    state: &AppState,
) -> Result<KnowledgeAuthoringCatalogSnapshot, String> {
    let project_root = state.current_project_data_root().await;
    let documents = load_project_knowledge_documents(&project_root)?;
    Ok(snapshot_from_documents(&documents))
}

fn reload_knowledge_state(
    project_root: &Path,
) -> Result<(Vec<LoadedKnowledgeDocument>, KnowledgeBase), String> {
    let documents = load_project_knowledge_documents(project_root)?;
    let knowledge_base = knowledge_base_from_documents(&documents);
    Ok((documents, knowledge_base))
}

fn load_project_knowledge_documents(
    project_root: &Path,
) -> Result<Vec<LoadedKnowledgeDocument>, String> {
    load_knowledge_documents(project_root, &project_root.join("knowledge"))
        .map_err(|error| error.to_string())
}

fn require_catalog_fingerprint(current: &str, expected: &str) -> Result<(), String> {
    if current == expected {
        Ok(())
    } else {
        Err("Knowledge catalog changed; reload before saving.".to_string())
    }
}

fn pretty_json(value: &Value) -> Result<String, String> {
    let mut content = serde_json::to_string_pretty(value)
        .map_err(|error| format!("Unable to serialize knowledge JSON: {error}"))?;
    content.push('\n');
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari-knowledge-command-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn write_project(root: &Path) {
        std::fs::create_dir_all(root.join("knowledge")).unwrap();
        std::fs::create_dir_all(root.join("characters")).unwrap();
        std::fs::write(
            root.join("knowledge/world.json"),
            r#"{"id":"world","category":"world_lore","title":"World","content":"World knowledge."}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("knowledge/place.json"),
            r#"{"id":"place","category":"location","title":"Place","content":"A known place."}"#,
        )
        .unwrap();
    }

    async fn authoring_state(root: &Path) -> AppState {
        let state = AppState::new();
        state.set_project_data_root(root.to_path_buf()).await;
        state
    }

    #[tokio::test]
    async fn knowledge_save_rejects_stale_or_invalid_updates_and_hot_reloads_runtime() {
        let root = temp_root("save");
        write_project(&root);
        let state = authoring_state(&root).await;
        let before = knowledge_authoring_snapshot(&state).await.unwrap();
        let mut replacement = before
            .entries
            .iter()
            .find(|entry| entry.id == "world")
            .unwrap()
            .clone();
        replacement.title = "A Better World".to_string();

        let saved = save_knowledge_entry_definition_inner(
            &state,
            replacement.clone(),
            Some("world"),
            &before.catalog_fingerprint,
        )
        .await
        .unwrap();

        assert_ne!(saved.catalog_fingerprint, before.catalog_fingerprint);
        assert_eq!(
            state
                .knowledge_base
                .read()
                .await
                .get_entry("world")
                .unwrap()
                .title,
            "A Better World"
        );
        assert_eq!(
            state
                .knowledge_base
                .read()
                .await
                .get_entry("world")
                .unwrap()
                .category
                .as_str(),
            "world_lore"
        );

        let stale = save_knowledge_entry_definition_inner(
            &state,
            replacement.clone(),
            Some("world"),
            &before.catalog_fingerprint,
        )
        .await
        .unwrap_err();
        assert!(stale.contains("changed; reload"), "{stale}");

        let path = root.join("knowledge/world.json");
        let file_before_invalid = std::fs::read(&path).unwrap();
        replacement.related_entries = vec!["missing".to_string()];
        let error = save_knowledge_entry_definition_inner(
            &state,
            replacement,
            Some("world"),
            &saved.catalog_fingerprint,
        )
        .await
        .unwrap_err();
        assert!(error.contains("unknown entry `missing`"), "{error}");
        assert_eq!(std::fs::read(path).unwrap(), file_before_invalid);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn knowledge_delete_requires_character_and_related_entry_references_to_be_removed() {
        let root = temp_root("delete");
        write_project(&root);
        std::fs::write(
            root.join("knowledge/world.json"),
            serde_json::to_vec_pretty(&json!({
                "id": "world",
                "category": "world_lore",
                "title": "World",
                "content": "World knowledge.",
                "related_entries": ["place"]
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("characters/author.json"),
            r#"{"id":"author","name":"Author","knowledge_refs":["place"]}"#,
        )
        .unwrap();
        let state = authoring_state(&root).await;
        let before = knowledge_authoring_snapshot(&state).await.unwrap();

        let error =
            delete_knowledge_entry_definition_inner(&state, "place", &before.catalog_fingerprint)
                .await
                .unwrap_err();
        assert!(error.contains("character:author"), "{error}");
        assert!(error.contains("knowledge:world"), "{error}");
        assert!(root.join("knowledge/place.json").is_file());

        std::fs::write(
            root.join("knowledge/world.json"),
            r#"{"id":"world","category":"world_lore","title":"World","content":"World knowledge."}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("characters/author.json"),
            r#"{"id":"author","name":"Author","knowledge_refs":[]}"#,
        )
        .unwrap();
        let current = knowledge_authoring_snapshot(&state).await.unwrap();
        let after =
            delete_knowledge_entry_definition_inner(&state, "place", &current.catalog_fingerprint)
                .await
                .unwrap();

        assert_eq!(after.entries.len(), 1);
        assert!(!root.join("knowledge/place.json").exists());
        assert!(state
            .knowledge_base
            .read()
            .await
            .get_entry("place")
            .is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
