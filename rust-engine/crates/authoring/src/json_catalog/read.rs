//! Exact-case, bounded JSON document reads and fingerprints.

use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::filesystem::sha256_json;
use crate::paths::resolve_project_relative;
use crate::project::canonical_project_root;

use super::{
    validate_authorable_json_path, JsonAcceptanceLevel, JsonCatalogDocument, JsonCatalogEntry,
    JsonCatalogError, JsonCatalogErrorCode, JsonDocumentKind, JSON_CATALOG_DOCUMENT_SCHEMA_V1,
    MAX_AUTHORABLE_JSON_BYTES,
};

/// Read one exact, authorable JSON document and return both optimistic and semantic hashes.
pub fn read_project_json(
    project_root: &Path,
    path: &str,
) -> Result<JsonCatalogDocument, JsonCatalogError> {
    let root = canonical_project_root(project_root).map_err(|_| {
        JsonCatalogError::new(
            JsonCatalogErrorCode::ProjectRootInvalid,
            "Project root must be an existing regular directory.",
            None,
        )
    })?;
    read_project_json_from_root(&root, path)
}

pub(super) fn read_project_json_from_root(
    root: &Path,
    path: &str,
) -> Result<JsonCatalogDocument, JsonCatalogError> {
    let catalog = validate_authorable_json_path(path)?;
    verify_exact_path(root, path)?;
    let target = resolve_project_relative(root, path).map_err(|_| {
        JsonCatalogError::new(
            JsonCatalogErrorCode::InvalidPath,
            "Project JSON path must stay inside the project root.",
            Some(path.to_string()),
        )
    })?;
    let metadata = std::fs::symlink_metadata(&target).map_err(|error| {
        JsonCatalogError::new(
            if error.kind() == std::io::ErrorKind::NotFound {
                JsonCatalogErrorCode::FileNotFound
            } else {
                JsonCatalogErrorCode::IoFailure
            },
            "Unable to inspect the project JSON document.",
            Some(path.to_string()),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(JsonCatalogError::new(
            if metadata.file_type().is_symlink() {
                JsonCatalogErrorCode::SymlinkNotAllowed
            } else {
                JsonCatalogErrorCode::NotRegularFile
            },
            "Project JSON documents must be regular files without symbolic links.",
            Some(path.to_string()),
        ));
    }
    if metadata.len() > MAX_AUTHORABLE_JSON_BYTES {
        return Err(JsonCatalogError::new(
            JsonCatalogErrorCode::FileTooLarge,
            format!("Project JSON documents cannot exceed {MAX_AUTHORABLE_JSON_BYTES} bytes."),
            Some(path.to_string()),
        ));
    }
    let bytes = std::fs::read(&target).map_err(|_| {
        JsonCatalogError::new(
            JsonCatalogErrorCode::IoFailure,
            "Unable to read the project JSON document.",
            Some(path.to_string()),
        )
    })?;
    if bytes.len() as u64 > MAX_AUTHORABLE_JSON_BYTES {
        return Err(JsonCatalogError::new(
            JsonCatalogErrorCode::FileTooLarge,
            format!("Project JSON documents cannot exceed {MAX_AUTHORABLE_JSON_BYTES} bytes."),
            Some(path.to_string()),
        ));
    }
    let document = serde_json::from_slice::<Value>(&bytes).map_err(|_| {
        JsonCatalogError::new(
            JsonCatalogErrorCode::InvalidJson,
            "Project JSON document could not be parsed.",
            Some(path.to_string()),
        )
    })?;
    let document_kind = if document.is_object() {
        JsonDocumentKind::Object
    } else if document.is_array() {
        JsonDocumentKind::Array
    } else {
        return Err(JsonCatalogError::new(
            JsonCatalogErrorCode::InvalidDocument,
            "Project JSON documents must contain an object or array.",
            Some(path.to_string()),
        ));
    };
    let metadata = JsonCatalogEntry {
        catalog,
        path: path.to_string(),
        size_bytes: bytes.len() as u64,
        sha256: sha256_bytes(&bytes),
        content_fingerprint: sha256_json(&document),
        document_kind,
    };
    Ok(JsonCatalogDocument {
        schema: JSON_CATALOG_DOCUMENT_SCHEMA_V1.to_string(),
        acceptance_level: JsonAcceptanceLevel::Document,
        metadata,
        document,
    })
}

fn verify_exact_path(root: &Path, path: &str) -> Result<(), JsonCatalogError> {
    let mut current = root.to_path_buf();
    for segment in path.split('/') {
        let entries = std::fs::read_dir(&current).map_err(|error| {
            JsonCatalogError::new(
                if error.kind() == std::io::ErrorKind::NotFound {
                    JsonCatalogErrorCode::FileNotFound
                } else {
                    JsonCatalogErrorCode::IoFailure
                },
                "Unable to resolve the exact project JSON path.",
                Some(path.to_string()),
            )
        })?;
        let mut case_collision = false;
        let mut exact = None;
        for entry in entries {
            let entry = entry.map_err(|_| {
                JsonCatalogError::new(
                    JsonCatalogErrorCode::IoFailure,
                    "Unable to inspect a project JSON path segment.",
                    Some(path.to_string()),
                )
            })?;
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if name == segment {
                exact = Some(entry.path());
                break;
            }
            if name.eq_ignore_ascii_case(segment) {
                case_collision = true;
            }
        }
        if exact.is_none() && case_collision {
            return Err(JsonCatalogError::new(
                JsonCatalogErrorCode::PathCaseCollision,
                "Project JSON path does not match the existing path's exact ASCII case.",
                Some(path.to_string()),
            ));
        }
        let next = exact.ok_or_else(|| {
            JsonCatalogError::new(
                JsonCatalogErrorCode::FileNotFound,
                "Project JSON document was not found.",
                Some(path.to_string()),
            )
        })?;
        let metadata = std::fs::symlink_metadata(&next).map_err(|_| {
            JsonCatalogError::new(
                JsonCatalogErrorCode::IoFailure,
                "Unable to inspect a project JSON path segment.",
                Some(path.to_string()),
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(JsonCatalogError::new(
                JsonCatalogErrorCode::SymlinkNotAllowed,
                "Project JSON paths cannot contain symbolic links.",
                Some(path.to_string()),
            ));
        }
        current = next;
    }
    Ok(())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
