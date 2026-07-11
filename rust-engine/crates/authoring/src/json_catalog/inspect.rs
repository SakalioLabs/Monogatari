//! Deterministic traversal and document-level catalog acceptance.

use std::collections::HashMap;
use std::path::Path;

use crate::project::canonical_project_root;

use super::read::read_project_json_from_root;
use super::{
    is_portable_segment, validate_authorable_json_path, AuthorableJsonCatalog, JsonAcceptanceLevel,
    JsonCatalogEntry, JsonCatalogError, JsonCatalogErrorCode, JsonCatalogIssue,
    JsonCatalogIssueSeverity, JsonCatalogReport, JsonCatalogStatus, JSON_CATALOG_REPORT_SCHEMA_V1,
    MAX_JSON_PATH_SEGMENTS,
};

const MAX_CATALOG_DOCUMENTS: usize = 4096;
const MAX_CATALOG_TOTAL_BYTES: u64 = 128 * 1024 * 1024;

/// Inspect every authorable JSON document, optionally restricted to one catalog.
pub fn inspect_project_json_catalog(
    project_root: &Path,
    filter: Option<AuthorableJsonCatalog>,
) -> Result<JsonCatalogReport, JsonCatalogError> {
    let root = canonical_project_root(project_root).map_err(|_| {
        JsonCatalogError::new(
            JsonCatalogErrorCode::ProjectRootInvalid,
            "Project root must be an existing regular directory.",
            None,
        )
    })?;
    let selected = filter.map_or_else(|| AuthorableJsonCatalog::ALL.to_vec(), |value| vec![value]);
    let mut state = CatalogWalkState::default();
    let mut catalogs = Vec::with_capacity(selected.len());

    for catalog in selected {
        let path = root.join(catalog.as_str());
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                state.warning(
                    JsonCatalogErrorCode::CatalogMissing,
                    Some(catalog.as_str().to_string()),
                    "Authorable JSON catalog directory is missing.",
                );
                catalogs.push(JsonCatalogStatus {
                    catalog,
                    exists: false,
                    document_count: 0,
                    total_bytes: 0,
                });
                continue;
            }
            Err(_) => {
                state.error(
                    JsonCatalogErrorCode::IoFailure,
                    Some(catalog.as_str().to_string()),
                    "Unable to inspect the JSON catalog directory.",
                );
                catalogs.push(JsonCatalogStatus {
                    catalog,
                    exists: false,
                    document_count: 0,
                    total_bytes: 0,
                });
                continue;
            }
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            state.error(
                if metadata.file_type().is_symlink() {
                    JsonCatalogErrorCode::SymlinkNotAllowed
                } else {
                    JsonCatalogErrorCode::NotRegularFile
                },
                Some(catalog.as_str().to_string()),
                "JSON catalog roots must be regular directories without symbolic links.",
            );
            catalogs.push(JsonCatalogStatus {
                catalog,
                exists: true,
                document_count: 0,
                total_bytes: 0,
            });
            continue;
        }

        let before_count = state.documents.len();
        let before_bytes = state.total_bytes;
        walk_catalog(&root, catalog, &path, catalog.as_str(), 1, &mut state);
        catalogs.push(JsonCatalogStatus {
            catalog,
            exists: true,
            document_count: state.documents.len() - before_count,
            total_bytes: state.total_bytes - before_bytes,
        });
    }

    state
        .documents
        .sort_by(|left, right| left.path.cmp(&right.path));
    let error_count = state
        .issues
        .iter()
        .filter(|issue| issue.severity == JsonCatalogIssueSeverity::Error)
        .count();
    let warning_count = state.issues.len() - error_count;
    Ok(JsonCatalogReport {
        schema: JSON_CATALOG_REPORT_SCHEMA_V1.to_string(),
        acceptance_level: JsonAcceptanceLevel::Document,
        valid: error_count == 0,
        error_count,
        warning_count,
        document_count: state.documents.len(),
        total_bytes: state.total_bytes,
        catalogs,
        documents: state.documents,
        issues: state.issues,
    })
}

fn walk_catalog(
    root: &Path,
    catalog: AuthorableJsonCatalog,
    directory: &Path,
    portable_directory: &str,
    depth: usize,
    state: &mut CatalogWalkState,
) {
    if state.limit_reached {
        return;
    }
    let directory_entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => {
            state.error(
                JsonCatalogErrorCode::IoFailure,
                Some(portable_directory.to_string()),
                "Unable to enumerate the JSON catalog directory.",
            );
            return;
        }
    };
    let mut entries = Vec::new();
    for entry in directory_entries {
        match entry {
            Ok(entry) => entries.push(entry),
            Err(_) => state.error(
                JsonCatalogErrorCode::IoFailure,
                Some(portable_directory.to_string()),
                "Unable to inspect a JSON catalog directory entry.",
            ),
        }
    }
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            state.error(
                JsonCatalogErrorCode::InvalidPath,
                Some(portable_directory.to_string()),
                "JSON catalog entries require UTF-8 filenames.",
            );
            continue;
        };
        let portable_path = format!("{portable_directory}/{name}");
        let lowercase_path = portable_path.to_ascii_lowercase();
        if let Some(previous) = state
            .seen_paths
            .insert(lowercase_path, portable_path.clone())
        {
            if previous != portable_path {
                state.error(
                    JsonCatalogErrorCode::PathCaseCollision,
                    Some(portable_path.clone()),
                    "JSON catalog entries collide by ASCII case.",
                );
                continue;
            }
        }
        let metadata = match std::fs::symlink_metadata(entry.path()) {
            Ok(metadata) => metadata,
            Err(_) => {
                state.error(
                    JsonCatalogErrorCode::IoFailure,
                    Some(portable_path.clone()),
                    "Unable to inspect a JSON catalog entry.",
                );
                continue;
            }
        };
        if metadata.file_type().is_symlink() {
            state.error(
                JsonCatalogErrorCode::SymlinkNotAllowed,
                Some(portable_path),
                "JSON catalog entries cannot be symbolic links.",
            );
            continue;
        }
        if metadata.is_dir() {
            if !is_portable_segment(&name) || depth + 1 >= MAX_JSON_PATH_SEGMENTS {
                state.error(
                    JsonCatalogErrorCode::InvalidPath,
                    Some(portable_path),
                    "JSON catalog directories must use bounded portable ASCII segments.",
                );
                continue;
            }
            let canonical = match entry.path().canonicalize() {
                Ok(path) if path.starts_with(root) => path,
                _ => {
                    state.error(
                        JsonCatalogErrorCode::InvalidPath,
                        Some(portable_path),
                        "JSON catalog directories must stay inside the project root.",
                    );
                    continue;
                }
            };
            walk_catalog(root, catalog, &canonical, &portable_path, depth + 1, state);
            continue;
        }
        if !metadata.is_file() {
            state.error(
                JsonCatalogErrorCode::NotRegularFile,
                Some(portable_path),
                "JSON catalog entries must be regular files or directories.",
            );
            continue;
        }
        if !name.ends_with(".json") {
            if name.to_ascii_lowercase().ends_with(".json") {
                state.error(
                    JsonCatalogErrorCode::InvalidPath,
                    Some(portable_path),
                    "Authorable documents require a lowercase `.json` extension.",
                );
            }
            continue;
        }
        if let Err(error) = validate_authorable_json_path(&portable_path) {
            state.error(error.code, error.path, error.message);
            continue;
        }
        if state.documents.len() >= MAX_CATALOG_DOCUMENTS
            || state.total_bytes.saturating_add(metadata.len()) > MAX_CATALOG_TOTAL_BYTES
        {
            state.error(
                JsonCatalogErrorCode::CatalogLimitExceeded,
                Some(portable_path),
                "JSON catalog inspection exceeded its bounded document or byte limit.",
            );
            state.limit_reached = true;
            return;
        }
        match read_project_json_from_root(root, &portable_path) {
            Ok(document) => {
                if state
                    .total_bytes
                    .saturating_add(document.metadata.size_bytes)
                    > MAX_CATALOG_TOTAL_BYTES
                {
                    state.error(
                        JsonCatalogErrorCode::CatalogLimitExceeded,
                        Some(portable_path),
                        "JSON catalog inspection exceeded its bounded byte limit.",
                    );
                    state.limit_reached = true;
                    return;
                }
                state.total_bytes += document.metadata.size_bytes;
                debug_assert_eq!(document.metadata.catalog, catalog);
                state.documents.push(document.metadata);
            }
            Err(error) => state.error(error.code, error.path, error.message),
        }
    }
}

#[derive(Default)]
struct CatalogWalkState {
    documents: Vec<JsonCatalogEntry>,
    issues: Vec<JsonCatalogIssue>,
    seen_paths: HashMap<String, String>,
    total_bytes: u64,
    limit_reached: bool,
}

impl CatalogWalkState {
    fn error(
        &mut self,
        code: JsonCatalogErrorCode,
        path: Option<String>,
        message: impl Into<String>,
    ) {
        self.issue(JsonCatalogIssueSeverity::Error, code, path, message);
    }

    fn warning(
        &mut self,
        code: JsonCatalogErrorCode,
        path: Option<String>,
        message: impl Into<String>,
    ) {
        self.issue(JsonCatalogIssueSeverity::Warning, code, path, message);
    }

    fn issue(
        &mut self,
        severity: JsonCatalogIssueSeverity,
        code: JsonCatalogErrorCode,
        path: Option<String>,
        message: impl Into<String>,
    ) {
        self.issues.push(JsonCatalogIssue {
            severity,
            code,
            path,
            message: message.into(),
        });
    }
}
