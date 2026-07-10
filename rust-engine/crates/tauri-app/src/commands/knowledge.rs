//! Knowledge base commands.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use llm_game::knowledge::{KnowledgeCategory, KnowledgeEntry};
use llm_game::KnowledgeBase;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::commands::content_paths::resolve_project_content_dir;
use crate::content_authoring::{
    ensure_regular_project_directory, sha256_json, source_label, stage_json_deletion,
    stage_json_replacement,
};
use crate::content_references::knowledge_references;
use crate::state::AppState;

const KNOWLEDGE_AUTHORING_SCHEMA_V1: &str = "monogatari-knowledge-authoring/v1";
const MAX_KNOWLEDGE_FILE_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Serialize)]
pub struct KnowledgeResult {
    pub id: String,
    pub category: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub importance: f32,
}

#[derive(Serialize)]
pub struct KnowledgeAuthoringCatalogSnapshot {
    pub schema: String,
    pub catalog_fingerprint: String,
    pub entries: Vec<KnowledgeEntry>,
}

#[derive(Clone, Copy)]
enum KnowledgeDocumentShape {
    Single,
    Array,
}

struct LoadedKnowledgeDocument {
    absolute_path: PathBuf,
    source_path: String,
    shape: KnowledgeDocumentShape,
    values: Vec<Value>,
    entries: Vec<KnowledgeEntry>,
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
    let mut kb = state.knowledge_base.write().await;
    kb.load_from_directory(&path)
        .await
        .map_err(|error| error.to_string())
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
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    let project_root = state.current_project_data_root().await;
    let documents = load_knowledge_documents(&project_root)?;
    require_catalog_fingerprint(
        &knowledge_catalog_fingerprint(&documents),
        &expected_catalog_fingerprint,
    )?;

    let entry = normalize_knowledge_entry(entry);
    validate_knowledge_entry(&entry)?;
    if let Some(original_id) = original_entry_id.as_deref() {
        if original_id != entry.id {
            return Err("Knowledge entry ids are immutable after creation.".to_string());
        }
    }

    let existing = documents
        .iter()
        .enumerate()
        .find_map(|(document_index, document)| {
            document
                .entries
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
    let known_ids = documents
        .iter()
        .flat_map(|document| document.entries.iter().map(|item| item.id.clone()))
        .chain(std::iter::once(entry.id.clone()))
        .collect::<HashSet<_>>();
    validate_knowledge_relations(&entry, &known_ids)?;

    let (target_path, output_value) = if let Some((document_index, entry_index)) = existing {
        let document = &documents[document_index];
        let mut values = document.values.clone();
        values[entry_index] = serde_json::to_value(&entry)
            .map_err(|error| format!("Unable to serialize knowledge entry: {error}"))?;
        (
            document.absolute_path.clone(),
            document_value(document.shape, values)?,
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

    let reload = reload_knowledge_state(&state, &project_root).await;
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
        .flat_map(|document| document.entries.iter())
        .any(|item| item.id == entry.id)
    {
        staged.rollback().await?;
        return Err("Saved knowledge entry was not present after reload.".to_string());
    }
    *state.knowledge_base.write().await = knowledge_base;
    staged.commit().await?;
    snapshot_from_documents(&documents)
}

/// Atomically delete one knowledge entry and hot-reload runtime knowledge.
#[tauri::command]
pub async fn delete_knowledge_entry_definition(
    state: State<'_, AppState>,
    entry_id: String,
    expected_catalog_fingerprint: String,
) -> Result<KnowledgeAuthoringCatalogSnapshot, String> {
    validate_knowledge_id(&entry_id)?;
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    let project_root = state.current_project_data_root().await;
    let documents = load_knowledge_documents(&project_root)?;
    require_catalog_fingerprint(
        &knowledge_catalog_fingerprint(&documents),
        &expected_catalog_fingerprint,
    )?;

    let (document_index, entry_index) = documents
        .iter()
        .enumerate()
        .find_map(|(document_index, document)| {
            document
                .entries
                .iter()
                .position(|item| item.id == entry_id)
                .map(|entry_index| (document_index, entry_index))
        })
        .ok_or_else(|| format!("Knowledge entry `{entry_id}` does not exist."))?;
    let document = &documents[document_index];
    let mut references = knowledge_references(&project_root, &entry_id)?;
    references.extend(
        documents
            .iter()
            .flat_map(|loaded| loaded.entries.iter())
            .filter(|entry| {
                entry.id != entry_id
                    && entry
                        .related_entries
                        .iter()
                        .any(|related_id| related_id == &entry_id)
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
    let mut remaining = document.values.clone();
    remaining.remove(entry_index);
    let staged = if remaining.is_empty() {
        stage_json_deletion(&document.absolute_path, "knowledge entry").await?
    } else {
        let output = document_value(document.shape, remaining)?;
        let content = pretty_json(&output)?;
        stage_json_replacement(
            &document.absolute_path,
            content.as_bytes(),
            MAX_KNOWLEDGE_FILE_BYTES,
            "knowledge entry",
        )
        .await?
    };

    let reload = reload_knowledge_state(&state, &project_root).await;
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
        .flat_map(|loaded| loaded.entries.iter())
        .any(|item| item.id == entry_id)
    {
        staged.rollback().await?;
        return Err("Deleted knowledge entry remained present after reload.".to_string());
    }
    *state.knowledge_base.write().await = knowledge_base;
    staged.commit().await?;
    snapshot_from_documents(&documents)
}

fn knowledge_result(entry: &KnowledgeEntry) -> KnowledgeResult {
    KnowledgeResult {
        id: entry.id.clone(),
        category: knowledge_category_label(&entry.category),
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
    let documents = load_knowledge_documents(&project_root)?;
    snapshot_from_documents(&documents)
}

async fn reload_knowledge_state(
    _state: &AppState,
    project_root: &Path,
) -> Result<(Vec<LoadedKnowledgeDocument>, KnowledgeBase), String> {
    let documents = load_knowledge_documents(project_root)?;
    let knowledge_root = project_root.join("knowledge");
    let mut knowledge_base = KnowledgeBase::new();
    if knowledge_root.is_dir() {
        knowledge_base
            .load_from_directory(&knowledge_root)
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok((documents, knowledge_base))
}

fn load_knowledge_documents(project_root: &Path) -> Result<Vec<LoadedKnowledgeDocument>, String> {
    let knowledge_root = project_root.join("knowledge");
    if !knowledge_root.exists() {
        return Ok(Vec::new());
    }
    let root_metadata = std::fs::symlink_metadata(&knowledge_root)
        .map_err(|error| format!("Unable to inspect knowledge directory: {error}"))?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err("Knowledge path must be a regular directory.".to_string());
    }

    let mut paths = std::fs::read_dir(&knowledge_root)
        .map_err(|error| format!("Unable to read knowledge directory: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut documents = Vec::new();
    let mut ids = HashSet::new();
    for path in paths {
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "Unable to inspect knowledge file `{}`: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Knowledge file must be regular: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_KNOWLEDGE_FILE_BYTES {
            return Err(format!(
                "Knowledge file `{}` exceeds the {} byte limit.",
                path.display(),
                MAX_KNOWLEDGE_FILE_BYTES
            ));
        }
        let source = std::fs::read_to_string(&path).map_err(|error| {
            format!(
                "Unable to read knowledge file `{}`: {error}",
                path.display()
            )
        })?;
        let value: Value = serde_json::from_str(&source)
            .map_err(|error| format!("Invalid knowledge JSON `{}`: {error}", path.display()))?;
        let (shape, values) = match value {
            Value::Object(_) => (KnowledgeDocumentShape::Single, vec![value]),
            Value::Array(values) if !values.is_empty() => (KnowledgeDocumentShape::Array, values),
            Value::Array(_) => {
                return Err(format!(
                    "Knowledge file `{}` cannot be empty.",
                    path.display()
                ))
            }
            _ => {
                return Err(format!(
                    "Knowledge file `{}` must contain an object or array.",
                    path.display()
                ))
            }
        };
        let entries = values
            .iter()
            .cloned()
            .map(serde_json::from_value::<KnowledgeEntry>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Invalid knowledge entry in `{}`: {error}", path.display()))?
            .into_iter()
            .map(normalize_knowledge_entry)
            .collect::<Vec<_>>();
        for entry in &entries {
            validate_knowledge_entry(entry)?;
            if !ids.insert(entry.id.clone()) {
                return Err(format!("Duplicate knowledge entry id `{}`.", entry.id));
            }
        }
        documents.push(LoadedKnowledgeDocument {
            absolute_path: path.clone(),
            source_path: source_label(project_root, &path),
            shape,
            values,
            entries,
        });
    }
    for entry in documents
        .iter()
        .flat_map(|document| document.entries.iter())
    {
        validate_knowledge_relations(entry, &ids)?;
    }
    Ok(documents)
}

fn snapshot_from_documents(
    documents: &[LoadedKnowledgeDocument],
) -> Result<KnowledgeAuthoringCatalogSnapshot, String> {
    let mut entries = documents
        .iter()
        .flat_map(|document| document.entries.iter().cloned())
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(KnowledgeAuthoringCatalogSnapshot {
        schema: KNOWLEDGE_AUTHORING_SCHEMA_V1.to_string(),
        catalog_fingerprint: knowledge_catalog_fingerprint(documents),
        entries,
    })
}

fn knowledge_catalog_fingerprint(documents: &[LoadedKnowledgeDocument]) -> String {
    let payload = documents
        .iter()
        .map(|document| json!({ "source": document.source_path, "entries": document.entries }))
        .collect::<Vec<_>>();
    sha256_json(&json!({ "schema": KNOWLEDGE_AUTHORING_SCHEMA_V1, "documents": payload }))
}

fn require_catalog_fingerprint(current: &str, expected: &str) -> Result<(), String> {
    if current == expected {
        Ok(())
    } else {
        Err("Knowledge catalog changed; reload before saving.".to_string())
    }
}

fn normalize_knowledge_entry(mut entry: KnowledgeEntry) -> KnowledgeEntry {
    entry.id = entry.id.trim().to_string();
    entry.title = entry.title.trim().to_string();
    entry.content = entry.content.trim().to_string();
    let mut seen = HashSet::new();
    entry.tags = entry
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty() && seen.insert(tag.to_ascii_lowercase()))
        .collect();
    let mut related_seen = HashSet::new();
    entry.related_entries = entry
        .related_entries
        .into_iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty() && related_seen.insert(id.clone()))
        .collect();
    entry
}

fn validate_knowledge_entry(entry: &KnowledgeEntry) -> Result<(), String> {
    validate_knowledge_id(&entry.id)?;
    validate_bounded_text(&entry.title, "title", 1, 256, false)?;
    validate_bounded_text(&entry.content, "content", 1, 16_384, true)?;
    let category = knowledge_category_label(&entry.category);
    if category.is_empty()
        || category.len() > 64
        || !category.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
    {
        return Err(
            "Knowledge category can contain only lowercase ASCII letters, numbers, underscores, or hyphens."
                .to_string(),
        );
    }
    if !entry.importance.is_finite() || !(0.0..=1.0).contains(&entry.importance) {
        return Err("Knowledge importance must be between 0 and 1.".to_string());
    }
    if entry.tags.len() > 64 {
        return Err("Knowledge entries can contain at most 64 tags.".to_string());
    }
    for tag in &entry.tags {
        validate_bounded_text(tag, "tag", 1, 64, false)?;
    }
    for related_id in &entry.related_entries {
        validate_knowledge_id(related_id)?;
    }
    Ok(())
}

fn validate_knowledge_relations(
    entry: &KnowledgeEntry,
    known_ids: &HashSet<String>,
) -> Result<(), String> {
    for related_id in &entry.related_entries {
        if related_id == &entry.id {
            return Err("Knowledge entries cannot reference themselves.".to_string());
        }
        if !known_ids.contains(related_id) {
            return Err(format!(
                "Related knowledge entry `{related_id}` does not exist."
            ));
        }
    }
    Ok(())
}

fn validate_knowledge_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 128 {
        return Err("Knowledge entry id must contain 1 to 128 characters.".to_string());
    }
    if !id.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
    }) {
        return Err(
            "Knowledge entry id can contain only lowercase ASCII letters, numbers, underscores, or hyphens."
                .to_string(),
        );
    }
    Ok(())
}

fn validate_bounded_text(
    value: &str,
    field: &str,
    min: usize,
    max: usize,
    allow_multiline: bool,
) -> Result<(), String> {
    let length = value.chars().count();
    let has_disallowed_control = value.chars().any(|character| {
        character.is_control() && !(allow_multiline && matches!(character, '\n' | '\r' | '\t'))
    });
    if length < min || length > max || has_disallowed_control {
        return Err(format!(
            "Knowledge {field} must contain {min} to {max} characters without control characters."
        ));
    }
    Ok(())
}

fn knowledge_category_label(category: &KnowledgeCategory) -> String {
    serde_json::to_value(category)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "other".to_string())
}

fn document_value(shape: KnowledgeDocumentShape, values: Vec<Value>) -> Result<Value, String> {
    match shape {
        KnowledgeDocumentShape::Single => values
            .into_iter()
            .next()
            .ok_or_else(|| "Single-entry knowledge document cannot be empty.".to_string()),
        KnowledgeDocumentShape::Array => Ok(Value::Array(values)),
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
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari-knowledge-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn authoring_loader_supports_single_and_array_documents() {
        let root = temp_root("documents");
        let knowledge = root.join("knowledge");
        std::fs::create_dir_all(&knowledge).unwrap();
        std::fs::write(
            knowledge.join("single.json"),
            r#"{"id":"single","category":"lore","title":"Single","content":"One entry"}"#,
        )
        .unwrap();
        std::fs::write(
            knowledge.join("array.json"),
            r#"[{"id":"first","category":"item","title":"First","content":"First entry"},{"id":"second","category":"location","title":"Second","content":"Second entry"}]"#,
        )
        .unwrap();

        let documents = load_knowledge_documents(&root).unwrap();
        let snapshot = snapshot_from_documents(&documents).unwrap();
        let _ = std::fs::remove_dir_all(&root);

        assert_eq!(snapshot.entries.len(), 3);
        assert_eq!(snapshot.schema, KNOWLEDGE_AUTHORING_SCHEMA_V1);
        assert!(!snapshot.catalog_fingerprint.is_empty());
    }

    #[test]
    fn validation_rejects_non_portable_ids_and_out_of_range_importance() {
        let mut entry: KnowledgeEntry = serde_json::from_value(json!({
            "id": "Bad Id",
            "category": "lore",
            "title": "Title",
            "content": "Content",
            "importance": 2.0
        }))
        .unwrap();
        assert!(validate_knowledge_entry(&entry).is_err());
        entry.id = "valid_id".to_string();
        assert!(validate_knowledge_entry(&entry).is_err());
        entry.importance = 0.8;
        assert!(validate_knowledge_entry(&entry).is_ok());

        entry.content = "First line\nSecond line".to_string();
        assert!(validate_knowledge_entry(&entry).is_ok());

        let mut known_ids = HashSet::from([entry.id.clone(), "other_entry".to_string()]);
        entry.related_entries = vec!["missing_entry".to_string()];
        assert!(validate_knowledge_relations(&entry, &known_ids).is_err());
        entry.related_entries = vec![entry.id.clone()];
        assert!(validate_knowledge_relations(&entry, &known_ids).is_err());
        entry.related_entries = vec!["other_entry".to_string()];
        assert!(validate_knowledge_relations(&entry, &known_ids).is_ok());
        known_ids.remove("other_entry");
        assert!(validate_knowledge_relations(&entry, &known_ids).is_err());
    }
}
