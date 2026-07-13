//! Tauri adapters for project-package export, inspection, and transactional import.

use tauri::State;

use crate::state::AppState;

use super::super::project::{
    build_project_config_state, build_project_export_manifest, resolve_project_root,
};
use super::{
    available_project_directory_name, canonical_regular_archive, canonical_regular_directory,
    normalize_archive_path, normalize_local_path, remove_import_staging, unique_sibling_path,
    verify_archive, write_project_archive, ProjectArchiveExportResult, ProjectArchiveImportResult,
    ProjectArchiveInspection,
};

/// Create a portable project package containing content and a signed inventory.
#[tauri::command]
pub async fn export_project_archive(
    state: State<'_, AppState>,
    project_path: Option<String>,
    destination_path: String,
) -> Result<ProjectArchiveExportResult, String> {
    let project_root = canonical_regular_directory(
        &resolve_project_root(&state, project_path).await?,
        "Project root",
    )?;
    let destination = normalize_archive_path(&destination_path, false)?;
    let loaded_characters = state.character_manager.read().await.character_ids();
    let loaded_dialogues = state.dialogue_manager.read().await.script_ids();
    let loaded_knowledge_count = state.knowledge_base.read().await.len();
    let current_scene = state
        .scene_manager
        .read()
        .await
        .current_scene_name()
        .map(str::to_string);

    tokio::task::spawn_blocking(move || {
        let project_state = build_project_config_state(&project_root)?;
        if !project_state.valid {
            let details = project_state
                .issues
                .iter()
                .filter(|issue| issue.severity == "error")
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!(
                "Project package export requires a valid project: {details}"
            ));
        }
        let manifest = build_project_export_manifest(
            &project_root,
            loaded_characters,
            loaded_dialogues,
            loaded_knowledge_count,
            current_scene,
        )?;
        write_project_archive(&project_root, &destination, manifest)
    })
    .await
    .map_err(|error| format!("Project package export task failed: {error}"))?
}

/// Verify an archive without writing any project content.
#[tauri::command]
pub async fn inspect_project_archive(
    archive_path: String,
) -> Result<ProjectArchiveInspection, String> {
    let archive_path = canonical_regular_archive(&normalize_archive_path(&archive_path, true)?)?;
    tokio::task::spawn_blocking(move || verify_archive(&archive_path, None).map(|v| v.inspection))
        .await
        .map_err(|error| format!("Project package inspection task failed: {error}"))?
}

/// Import a verified package into a new sibling directory under `destination_parent`.
#[tauri::command]
pub async fn import_project_archive(
    state: State<'_, AppState>,
    archive_path: String,
    destination_parent: String,
) -> Result<ProjectArchiveImportResult, String> {
    let archive_path = canonical_regular_archive(&normalize_archive_path(&archive_path, true)?)?;
    let destination_parent = canonical_regular_directory(
        &normalize_local_path(&destination_parent, "Import destination")?,
        "Import destination",
    )?;
    let active_root = state.current_project_data_root().await;
    if let Ok(active_root) = active_root.canonicalize() {
        if destination_parent.starts_with(&active_root) {
            return Err(
                "Imported projects must be created outside the active project directory."
                    .to_string(),
            );
        }
    }

    let staging_root = unique_sibling_path(&destination_parent, ".monogatari-import", "tmp")?;
    std::fs::create_dir(&staging_root).map_err(|error| {
        format!(
            "Unable to create project import staging directory `{}`: {error}",
            staging_root.display()
        )
    })?;

    let archive_for_task = archive_path.clone();
    let staging_for_task = staging_root.clone();
    let verified = match tokio::task::spawn_blocking(move || {
        verify_archive(&archive_for_task, Some(&staging_for_task))
    })
    .await
    {
        Ok(Ok(verified)) => verified,
        Ok(Err(error)) => {
            remove_import_staging(&staging_root);
            return Err(error);
        }
        Err(error) => {
            remove_import_staging(&staging_root);
            return Err(format!("Project package import task failed: {error}"));
        }
    };

    let project_state = match build_project_config_state(&staging_root) {
        Ok(state) if state.valid => state,
        Ok(state) => {
            let details = state
                .issues
                .iter()
                .filter(|issue| issue.severity == "error")
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            remove_import_staging(&staging_root);
            return Err(format!(
                "Imported project configuration is invalid: {details}"
            ));
        }
        Err(error) => {
            remove_import_staging(&staging_root);
            return Err(error);
        }
    };
    drop(project_state);

    if let Err(error) = super::super::engine::load_project_content(&staging_root).await {
        remove_import_staging(&staging_root);
        return Err(format!("Imported runtime content is invalid: {error}"));
    }
    if let Err(error) = super::super::scenes::build_scene_asset_catalog(&staging_root) {
        remove_import_staging(&staging_root);
        return Err(format!("Imported scene content is invalid: {error}"));
    }
    if let Err(error) = super::super::endings::load_story_endings(&staging_root) {
        remove_import_staging(&staging_root);
        return Err(format!("Imported ending content is invalid: {error}"));
    }

    let directory_name = available_project_directory_name(
        &destination_parent,
        &verified.inspection.project_title,
        &archive_path,
    )?;
    let destination = destination_parent.join(&directory_name);
    if destination.exists() {
        remove_import_staging(&staging_root);
        return Err(format!(
            "Import destination appeared while validating the package: {}",
            destination.display()
        ));
    }
    if let Err(error) = std::fs::rename(&staging_root, &destination) {
        remove_import_staging(&staging_root);
        return Err(format!(
            "Unable to commit imported project to `{}`: {error}",
            destination.display()
        ));
    }

    Ok(ProjectArchiveImportResult {
        archive_path: archive_path.to_string_lossy().to_string(),
        project_path: destination.to_string_lossy().to_string(),
        project_title: verified.inspection.project_title,
        directory_name,
        file_count: verified.inspection.file_count,
        total_bytes: verified.inspection.total_bytes,
        content_sha256: verified.inspection.content_sha256,
    })
}
