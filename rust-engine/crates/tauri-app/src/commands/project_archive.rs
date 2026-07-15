//! Tauri path and transaction adapters for shared `.monogatari` packages.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use serde_json::Value;

use llm_authoring::project_package::{
    is_reserved_windows_segment, write_project_package, ProjectPackageExportResult,
    ProjectPackageInspection, ProjectPackageTargetPolicy,
};

pub(crate) mod commands;

static ARCHIVE_STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub type ProjectArchiveInspection = ProjectPackageInspection;
pub type ProjectArchiveExportResult = ProjectPackageExportResult;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectArchiveImportResult {
    pub archive_path: String,
    pub project_path: String,
    pub project_title: String,
    pub directory_name: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub content_sha256: String,
}

fn write_project_archive(
    project_root: &Path,
    destination: &Path,
    manifest_value: Value,
) -> Result<ProjectArchiveExportResult, String> {
    write_project_package(
        project_root,
        destination,
        manifest_value,
        ProjectPackageTargetPolicy::ReplaceExisting,
    )
}

fn normalize_archive_path(value: &str) -> Result<PathBuf, String> {
    let path = normalize_local_path(value, "Project package")?;
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("monogatari"))
    {
        return Err("Project package paths must use the `.monogatari` extension.".to_string());
    }
    Ok(path)
}

fn normalize_local_path(value: &str, label: &str) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_control) || trimmed.contains("://") {
        return Err(format!("{label} must be a local filesystem path."));
    }
    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        Ok(path)
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .map_err(|error| format!("Unable to resolve {label}: {error}"))
    }
}

fn canonical_regular_directory(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("Unable to inspect {label} `{}`: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "{label} must be a regular directory: {}",
            path.display()
        ));
    }
    path.canonicalize()
        .map_err(|error| format!("Unable to resolve {label} `{}`: {error}", path.display()))
}

fn unique_sibling_path(parent: &Path, prefix: &str, suffix: &str) -> Result<PathBuf, String> {
    for _ in 0..1000 {
        let counter = ARCHIVE_STAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            "{prefix}-{}-{counter}.{suffix}",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("Unable to allocate a unique project package staging path.".to_string())
}

fn available_project_directory_name(
    parent: &Path,
    project_title: &str,
    archive_path: &Path,
) -> Result<String, String> {
    let fallback = archive_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("monogatari-project");
    let base = portable_directory_slug(project_title, fallback);
    for index in 1..=999 {
        let candidate = if index == 1 {
            base.clone()
        } else {
            format!("{base}-{index}")
        };
        if !parent.join(&candidate).exists() {
            return Ok(candidate);
        }
    }
    Err("Unable to allocate an unused imported project directory name.".to_string())
}

fn portable_directory_slug(project_title: &str, fallback: &str) -> String {
    fn slug(value: &str) -> String {
        let mut result = String::new();
        let mut pending_separator = false;
        for ch in value.chars() {
            if ch.is_ascii_alphanumeric() {
                if pending_separator && !result.is_empty() {
                    result.push('-');
                }
                result.push(ch.to_ascii_lowercase());
                pending_separator = false;
            } else {
                pending_separator = true;
            }
            if result.len() >= 72 {
                break;
            }
        }
        result.trim_matches('-').to_string()
    }
    let preferred = slug(project_title);
    if !preferred.is_empty() && !is_reserved_windows_segment(&preferred) {
        return preferred;
    }
    let fallback = slug(fallback);
    if !fallback.is_empty() && !is_reserved_windows_segment(&fallback) {
        fallback
    } else {
        "monogatari-project".to_string()
    }
}

fn remove_import_staging(path: &Path) {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return;
    };
    if name.starts_with(".monogatari-import-") {
        let _ = std::fs::remove_dir_all(path);
    }
}

#[cfg(test)]
mod tests;
