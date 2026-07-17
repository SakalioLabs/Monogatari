//! Bounded Workflow discovery and atomic document persistence shared by transports.

use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::filesystem::{ensure_regular_project_directory, stage_json_replacement};
use crate::story_events::StoryEventCatalog;
use crate::workflow_validation::{
    format_validation_errors, validate_workflow_with_catalog, Workflow, WorkflowFileSummary,
    MAX_WORKFLOW_DEPTH, MAX_WORKFLOW_FILES, MAX_WORKFLOW_FILE_BYTES,
};

const WORKFLOW_DIRECTORY: &str = "workflows";
const MAX_WORKFLOW_PATH_BYTES: usize = 512;
const MAX_WORKFLOW_PATH_SEGMENTS: usize = 16;
const MAX_WORKFLOW_PATH_SEGMENT_BYTES: usize = 128;

/// List parseable Workflow documents under the fixed project catalog.
pub fn list_project_workflow_summaries(
    project_root: &Path,
) -> Result<Vec<WorkflowFileSummary>, String> {
    let workflow_root = project_root.join(WORKFLOW_DIRECTORY);
    if !workflow_root.exists() {
        return Ok(Vec::new());
    }
    let metadata = std::fs::symlink_metadata(&workflow_root)
        .map_err(|error| format!("Failed to inspect Workflow directory: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Project workflows path must be a regular directory.".to_string());
    }
    let canonical_root = workflow_root
        .canonicalize()
        .map_err(|error| format!("Failed to resolve Workflow directory: {error}"))?;

    let mut summaries = Vec::new();
    let mut discovered_files = 0;
    collect_workflow_summaries(
        &canonical_root,
        &canonical_root,
        0,
        &mut discovered_files,
        &mut summaries,
    )?;
    summaries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(summaries)
}

/// Validate and atomically create or replace one Workflow document.
pub async fn save_project_workflow(
    project_root: &Path,
    workflow: &Workflow,
    requested_path: &str,
) -> Result<(), String> {
    let event_catalog = StoryEventCatalog::load_from_project_root(project_root)?;
    let validation = validate_workflow_with_catalog(workflow, &event_catalog);
    if !validation.valid {
        return Err(format_validation_errors(&validation));
    }

    let target_path = workflow_target_path_for_save(project_root, requested_path).await?;
    let content = serde_json::to_vec_pretty(workflow).map_err(|error| error.to_string())?;
    let staged = stage_json_replacement(
        &target_path,
        &content,
        MAX_WORKFLOW_FILE_BYTES,
        "Workflow document",
    )
    .await?;
    staged.commit().await
}

/// Load and validate one Workflow document from the fixed project catalog.
pub async fn load_project_workflow(
    project_root: &Path,
    requested_path: &str,
) -> Result<Workflow, String> {
    let target_path = workflow_target_path_for_load(project_root, requested_path)?;
    let content = tokio::fs::read_to_string(&target_path)
        .await
        .map_err(|error| format!("Failed to read Workflow document: {error}"))?;
    let workflow: Workflow = serde_json::from_str(&content)
        .map_err(|error| format!("Invalid Workflow JSON: {error}"))?;
    let event_catalog = StoryEventCatalog::load_from_project_root(project_root)?;
    let validation = validate_workflow_with_catalog(&workflow, &event_catalog);
    if !validation.valid {
        return Err(format_validation_errors(&validation));
    }
    Ok(workflow)
}

fn collect_workflow_summaries(
    workflow_root: &Path,
    directory: &Path,
    depth: usize,
    discovered_files: &mut usize,
    summaries: &mut Vec<WorkflowFileSummary>,
) -> Result<(), String> {
    if depth > MAX_WORKFLOW_DEPTH || *discovered_files >= MAX_WORKFLOW_FILES {
        return Ok(());
    }

    let mut entries = std::fs::read_dir(directory)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        if *discovered_files >= MAX_WORKFLOW_FILES {
            break;
        }
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            let Ok(canonical) = path.canonicalize() else {
                continue;
            };
            if canonical.starts_with(workflow_root) {
                collect_workflow_summaries(
                    workflow_root,
                    &canonical,
                    depth + 1,
                    discovered_files,
                    summaries,
                )?;
            }
            continue;
        }
        if !metadata.is_file()
            || path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| !extension.eq_ignore_ascii_case("json"))
                .unwrap_or(true)
        {
            continue;
        }
        *discovered_files += 1;
        if metadata.len() > MAX_WORKFLOW_FILE_BYTES {
            continue;
        }

        let Ok(canonical) = path.canonicalize() else {
            continue;
        };
        if !canonical.starts_with(workflow_root) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&canonical) else {
            continue;
        };
        let Ok(workflow) = serde_json::from_str::<Workflow>(&content) else {
            continue;
        };
        let relative = canonical
            .strip_prefix(workflow_root)
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        summaries.push(WorkflowFileSummary {
            path: relative,
            workflow_id: workflow.id,
            name: workflow.name,
            node_count: workflow.nodes.len(),
        });
    }
    Ok(())
}

async fn workflow_target_path_for_save(
    project_root: &Path,
    requested_path: &str,
) -> Result<PathBuf, String> {
    let relative_path = normalize_workflow_relative_path(requested_path)?;
    let workflow_root =
        ensure_regular_project_directory(project_root, WORKFLOW_DIRECTORY, "Workflow").await?;
    let file_name = relative_path
        .file_name()
        .ok_or_else(|| "Workflow paths must name a JSON file.".to_string())?;
    let mut directory = workflow_root.clone();
    if let Some(parent) = relative_path.parent() {
        for component in parent.components() {
            let Component::Normal(segment) = component else {
                return Err("Workflow paths must stay inside the project catalog.".to_string());
            };
            directory = ensure_workflow_directory(&workflow_root, &directory, segment).await?;
        }
    }
    Ok(directory.join(file_name))
}

fn workflow_target_path_for_load(
    project_root: &Path,
    requested_path: &str,
) -> Result<PathBuf, String> {
    let relative_path = normalize_workflow_relative_path(requested_path)?;
    let workflow_root = project_root.join(WORKFLOW_DIRECTORY);
    let root_metadata = std::fs::symlink_metadata(&workflow_root)
        .map_err(|error| format!("Failed to inspect Workflow directory: {error}"))?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err("Project workflows path must be a regular directory.".to_string());
    }
    let canonical_root = workflow_root
        .canonicalize()
        .map_err(|error| format!("Failed to resolve Workflow directory: {error}"))?;
    let target = workflow_root.join(relative_path);
    let metadata = std::fs::symlink_metadata(&target)
        .map_err(|error| format!("Failed to inspect Workflow document: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("Workflow document must be a regular file.".to_string());
    }
    if metadata.len() > MAX_WORKFLOW_FILE_BYTES {
        return Err(format!(
            "Workflow document is {} bytes; the limit is {MAX_WORKFLOW_FILE_BYTES} bytes.",
            metadata.len()
        ));
    }
    let canonical_target = target
        .canonicalize()
        .map_err(|error| format!("Failed to resolve Workflow document: {error}"))?;
    if !canonical_target.starts_with(&canonical_root) {
        return Err("Workflow document escapes the project workflows directory.".to_string());
    }
    Ok(canonical_target)
}

async fn ensure_workflow_directory(
    workflow_root: &Path,
    parent: &Path,
    segment: &std::ffi::OsStr,
) -> Result<PathBuf, String> {
    ensure_no_workflow_directory_case_alias(parent, segment)?;
    let directory = parent.join(segment);
    match tokio::fs::create_dir(&directory).await {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => {
            return Err(format!(
                "Failed to create Workflow directory `{}`: {error}",
                directory.display()
            ));
        }
    }
    let metadata = std::fs::symlink_metadata(&directory).map_err(|error| {
        format!(
            "Failed to inspect Workflow directory `{}`: {error}",
            directory.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "Workflow directory must be a regular directory: {}",
            directory.display()
        ));
    }
    let canonical = directory.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve Workflow directory `{}`: {error}",
            directory.display()
        )
    })?;
    if !canonical.starts_with(workflow_root) {
        return Err(format!(
            "Workflow directory escapes the project catalog: {}",
            directory.display()
        ));
    }
    Ok(canonical)
}

fn ensure_no_workflow_directory_case_alias(
    parent: &Path,
    requested_segment: &std::ffi::OsStr,
) -> Result<(), String> {
    let requested = requested_segment
        .to_str()
        .ok_or_else(|| "Workflow directory name is not valid UTF-8.".to_string())?;
    let entries = std::fs::read_dir(parent).map_err(|error| {
        format!(
            "Failed to inspect Workflow directory `{}`: {error}",
            parent.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to inspect Workflow directory entry `{}`: {error}",
                parent.display()
            )
        })?;
        let Some(existing) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if existing.eq_ignore_ascii_case(requested) && existing != requested {
            return Err(format!(
                "Workflow directory `{}` collides with existing path `{}` by ASCII case.",
                parent.join(requested).display(),
                entry.path().display()
            ));
        }
    }
    Ok(())
}

fn normalize_workflow_relative_path(requested_path: &str) -> Result<PathBuf, String> {
    let normalized = requested_path.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.chars().any(char::is_control) {
        return Err(
            "Workflow paths must be non-empty and cannot contain control characters.".to_string(),
        );
    }
    if normalized.len() > MAX_WORKFLOW_PATH_BYTES {
        return Err(format!(
            "Workflow paths cannot exceed {MAX_WORKFLOW_PATH_BYTES} UTF-8 bytes."
        ));
    }
    if normalized.contains(':') {
        return Err("Workflow paths cannot contain drive prefixes or URI schemes.".to_string());
    }

    let mut segments = normalized.split('/').collect::<Vec<_>>();
    if segments.len() > MAX_WORKFLOW_PATH_SEGMENTS
        || segments.iter().any(|segment| {
            segment.is_empty()
                || *segment == "."
                || *segment == ".."
                || segment.len() > MAX_WORKFLOW_PATH_SEGMENT_BYTES
        })
    {
        return Err(
            "Workflow paths must use bounded non-empty segments without current or parent traversal."
                .to_string(),
        );
    }

    if segments.first() == Some(&WORKFLOW_DIRECTORY) {
        segments.remove(0);
    }
    if segments.is_empty() {
        return Err("Workflow paths must name a JSON workflow file.".to_string());
    }

    let relative = segments.join("/");
    let relative_path = Path::new(&relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err("Workflow paths must be relative to the workflows directory.".to_string());
    }
    if relative_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("json"))
        != Some(true)
    {
        return Err("Workflow paths must end with .json.".to_string());
    }
    Ok(relative_path.to_path_buf())
}

#[cfg(test)]
mod tests;
