//! Bounded Knowledge catalog loading shared by runtime, desktop, and Agent transports.

use std::fmt;
use std::path::{Path, PathBuf};

use llm_game::knowledge::{KnowledgeBase, KnowledgeEntry};
use serde::Serialize;
use serde_json::{json, Value};

use crate::filesystem::{sha256_json, source_label};
use crate::knowledge_validation::{
    format_knowledge_validation_errors, normalize_knowledge_entry, validate_knowledge_catalog,
    KnowledgeValidationIssue, KnowledgeValidationResult,
};

pub const KNOWLEDGE_AUTHORING_SCHEMA_V1: &str = "monogatari-knowledge-authoring/v1";
pub const MAX_KNOWLEDGE_FILES: usize = 1_024;
pub const MAX_KNOWLEDGE_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_KNOWLEDGE_CATALOG_BYTES: u64 = 32 * 1024 * 1024;

const KNOWLEDGE_FIELDS: &[&str] = &[
    "id",
    "category",
    "title",
    "content",
    "tags",
    "importance",
    "metadata",
    "related_entries",
    "relatedEntries",
];

#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeAuthoringCatalogSnapshot {
    pub schema: String,
    pub catalog_fingerprint: String,
    pub entries: Vec<KnowledgeEntry>,
}

#[derive(Debug, Clone, Copy)]
enum KnowledgeDocumentShape {
    Single,
    Array,
}

#[derive(Debug, Clone)]
pub struct LoadedKnowledgeDocument {
    absolute_path: PathBuf,
    source_path: String,
    shape: KnowledgeDocumentShape,
    values: Vec<Value>,
    entries: Vec<KnowledgeEntry>,
}

impl LoadedKnowledgeDocument {
    pub fn absolute_path(&self) -> &Path {
        &self.absolute_path
    }

    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    pub fn entries(&self) -> &[KnowledgeEntry] {
        &self.entries
    }

    pub fn replacing_entry(
        &self,
        entry_index: usize,
        entry: &KnowledgeEntry,
    ) -> Result<Value, String> {
        if entry_index >= self.values.len() {
            return Err("Knowledge entry index is outside its source document.".to_string());
        }
        let mut values = self.values.clone();
        values[entry_index] = serde_json::to_value(entry)
            .map_err(|error| format!("Unable to serialize knowledge entry: {error}"))?;
        document_value(self.shape, values)
    }

    pub fn removing_entry(&self, entry_index: usize) -> Result<Option<Value>, String> {
        if entry_index >= self.values.len() {
            return Err("Knowledge entry index is outside its source document.".to_string());
        }
        let mut values = self.values.clone();
        values.remove(entry_index);
        if values.is_empty() {
            Ok(None)
        } else {
            document_value(self.shape, values).map(Some)
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeCatalogError {
    code: String,
    path: Option<String>,
    message: String,
    validation: Option<KnowledgeValidationResult>,
}

impl KnowledgeCatalogError {
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn validation_issues(&self) -> &[KnowledgeValidationIssue] {
        self.validation
            .as_ref()
            .map(|result| result.issues.as_slice())
            .unwrap_or_default()
    }

    fn new(code: &str, path: Option<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            path,
            message: message.into(),
            validation: None,
        }
    }

    fn validation(result: KnowledgeValidationResult) -> Self {
        Self {
            code: "knowledge_catalog_invalid".to_string(),
            path: Some("knowledge".to_string()),
            message: format_knowledge_validation_errors(&result),
            validation: Some(result),
        }
    }
}

impl fmt::Display for KnowledgeCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for KnowledgeCatalogError {}

/// Load, normalize, and validate every JSON document in one Knowledge directory.
pub fn load_knowledge_documents(
    project_root: &Path,
    knowledge_root: &Path,
) -> Result<Vec<LoadedKnowledgeDocument>, KnowledgeCatalogError> {
    if !knowledge_root.exists() {
        return Ok(Vec::new());
    }
    let root_metadata = std::fs::symlink_metadata(knowledge_root).map_err(|error| {
        KnowledgeCatalogError::new(
            "knowledge_directory_invalid",
            Some(source_label(project_root, knowledge_root)),
            format!("Unable to inspect knowledge directory: {error}"),
        )
    })?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(KnowledgeCatalogError::new(
            "knowledge_directory_invalid",
            Some(source_label(project_root, knowledge_root)),
            "Knowledge path must be a regular directory.",
        ));
    }

    let entries = std::fs::read_dir(knowledge_root).map_err(|error| {
        KnowledgeCatalogError::new(
            "knowledge_directory_read_failed",
            Some(source_label(project_root, knowledge_root)),
            format!("Unable to read knowledge directory: {error}"),
        )
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            KnowledgeCatalogError::new(
                "knowledge_directory_read_failed",
                Some(source_label(project_root, knowledge_root)),
                format!("Unable to read a knowledge directory entry: {error}"),
            )
        })?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.len() > MAX_KNOWLEDGE_FILES {
        return Err(KnowledgeCatalogError::new(
            "knowledge_file_count_invalid",
            Some(source_label(project_root, knowledge_root)),
            format!("Knowledge catalogs can contain at most {MAX_KNOWLEDGE_FILES} JSON files."),
        ));
    }

    let mut documents = Vec::with_capacity(paths.len());
    let mut source_entries = Vec::new();
    let mut total_bytes = 0_u64;
    for path in paths {
        let source_path = source_label(project_root, &path);
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            KnowledgeCatalogError::new(
                "knowledge_file_invalid",
                Some(source_path.clone()),
                format!("Unable to inspect knowledge file `{source_path}`: {error}"),
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(KnowledgeCatalogError::new(
                "knowledge_file_invalid",
                Some(source_path.clone()),
                format!("Knowledge file must be regular: {source_path}"),
            ));
        }
        if metadata.len() > MAX_KNOWLEDGE_FILE_BYTES {
            return Err(KnowledgeCatalogError::new(
                "knowledge_file_too_large",
                Some(source_path.clone()),
                format!(
                    "Knowledge file `{source_path}` exceeds the {MAX_KNOWLEDGE_FILE_BYTES} byte limit."
                ),
            ));
        }
        if total_bytes.saturating_add(metadata.len()) > MAX_KNOWLEDGE_CATALOG_BYTES {
            return Err(KnowledgeCatalogError::new(
                "knowledge_catalog_too_large",
                Some(source_label(project_root, knowledge_root)),
                format!("Knowledge catalog exceeds the {MAX_KNOWLEDGE_CATALOG_BYTES} byte limit."),
            ));
        }

        let bytes = std::fs::read(&path).map_err(|error| {
            KnowledgeCatalogError::new(
                "knowledge_file_read_failed",
                Some(source_path.clone()),
                format!("Unable to read knowledge file `{source_path}`: {error}"),
            )
        })?;
        if bytes.len() as u64 > MAX_KNOWLEDGE_FILE_BYTES {
            return Err(KnowledgeCatalogError::new(
                "knowledge_file_too_large",
                Some(source_path.clone()),
                format!(
                    "Knowledge file `{source_path}` exceeds the {MAX_KNOWLEDGE_FILE_BYTES} byte limit."
                ),
            ));
        }
        total_bytes = total_bytes.saturating_add(bytes.len() as u64);
        if total_bytes > MAX_KNOWLEDGE_CATALOG_BYTES {
            return Err(KnowledgeCatalogError::new(
                "knowledge_catalog_too_large",
                Some(source_label(project_root, knowledge_root)),
                format!("Knowledge catalog exceeds the {MAX_KNOWLEDGE_CATALOG_BYTES} byte limit."),
            ));
        }
        let source = String::from_utf8(bytes).map_err(|error| {
            KnowledgeCatalogError::new(
                "knowledge_file_utf8_invalid",
                Some(source_path.clone()),
                format!("Knowledge file `{source_path}` is not valid UTF-8: {error}"),
            )
        })?;
        let value: Value = serde_json::from_str(&source).map_err(|error| {
            KnowledgeCatalogError::new(
                "knowledge_json_invalid",
                Some(source_path.clone()),
                format!("Invalid knowledge JSON `{source_path}`: {error}"),
            )
        })?;
        let (shape, values) = match value {
            Value::Object(_) => (KnowledgeDocumentShape::Single, vec![value]),
            Value::Array(values) if !values.is_empty() => (KnowledgeDocumentShape::Array, values),
            Value::Array(_) => {
                return Err(KnowledgeCatalogError::new(
                    "knowledge_document_empty",
                    Some(source_path.clone()),
                    format!("Knowledge file `{source_path}` cannot be empty."),
                ));
            }
            _ => {
                return Err(KnowledgeCatalogError::new(
                    "knowledge_document_shape_invalid",
                    Some(source_path.clone()),
                    format!("Knowledge file `{source_path}` must contain an object or array."),
                ));
            }
        };

        let mut parsed_entries = Vec::with_capacity(values.len());
        for (index, value) in values.iter().enumerate() {
            validate_entry_fields(value, &source_path, index)?;
            let entry =
                serde_json::from_value::<KnowledgeEntry>(value.clone()).map_err(|error| {
                    KnowledgeCatalogError::new(
                        "knowledge_entry_invalid",
                        Some(source_path.clone()),
                        format!("Invalid knowledge entry {index} in `{source_path}`: {error}"),
                    )
                })?;
            source_entries.push(entry.clone());
            parsed_entries.push(normalize_knowledge_entry(entry));
        }
        documents.push(LoadedKnowledgeDocument {
            absolute_path: path,
            source_path,
            shape,
            values,
            entries: parsed_entries,
        });
    }

    let validation = validate_knowledge_catalog(&source_entries);
    if !validation.valid {
        return Err(KnowledgeCatalogError::validation(validation));
    }
    Ok(documents)
}

pub fn knowledge_base_from_documents(documents: &[LoadedKnowledgeDocument]) -> KnowledgeBase {
    let mut knowledge_base = KnowledgeBase::new();
    for entry in documents
        .iter()
        .flat_map(|document| document.entries.iter().cloned())
    {
        knowledge_base.add_entry(entry);
    }
    knowledge_base
}

pub fn snapshot_from_documents(
    documents: &[LoadedKnowledgeDocument],
) -> KnowledgeAuthoringCatalogSnapshot {
    let mut entries = documents
        .iter()
        .flat_map(|document| document.entries.iter().cloned())
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    KnowledgeAuthoringCatalogSnapshot {
        schema: KNOWLEDGE_AUTHORING_SCHEMA_V1.to_string(),
        catalog_fingerprint: knowledge_catalog_fingerprint(documents),
        entries,
    }
}

pub fn knowledge_catalog_fingerprint(documents: &[LoadedKnowledgeDocument]) -> String {
    let payload = documents
        .iter()
        .map(|document| json!({ "source": document.source_path, "entries": document.entries }))
        .collect::<Vec<_>>();
    sha256_json(&json!({ "schema": KNOWLEDGE_AUTHORING_SCHEMA_V1, "documents": payload }))
}

fn validate_entry_fields(
    value: &Value,
    source_path: &str,
    entry_index: usize,
) -> Result<(), KnowledgeCatalogError> {
    let object = value.as_object().ok_or_else(|| {
        KnowledgeCatalogError::new(
            "knowledge_entry_shape_invalid",
            Some(source_path.to_string()),
            format!("Knowledge entry {entry_index} in `{source_path}` must be an object."),
        )
    })?;
    let mut unknown = object
        .keys()
        .filter(|field| !KNOWLEDGE_FIELDS.contains(&field.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    unknown.sort();
    if !unknown.is_empty() {
        return Err(KnowledgeCatalogError::new(
            "knowledge_unknown_field",
            Some(source_path.to_string()),
            format!(
                "Knowledge entry {entry_index} in `{source_path}` contains unknown field(s): {}.",
                unknown.join(", ")
            ),
        ));
    }
    Ok(())
}

fn document_value(shape: KnowledgeDocumentShape, values: Vec<Value>) -> Result<Value, String> {
    match shape {
        KnowledgeDocumentShape::Single => {
            if values.len() != 1 {
                return Err(
                    "Single-entry knowledge document must contain exactly one entry.".to_string(),
                );
            }
            Ok(values.into_iter().next().expect("length checked"))
        }
        KnowledgeDocumentShape::Array => Ok(Value::Array(values)),
    }
}

#[cfg(test)]
mod tests;
