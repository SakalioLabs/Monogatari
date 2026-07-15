//! Tauri adapters for project-package export, inspection, and transactional import.

use tauri::State;

use llm_authoring::delivery_validation::{
    validate_project_delivery, DeliveryIssueSeverity, DeliveryValidationReport,
};
use llm_authoring::project_package::{extract_project_package, inspect_project_package};

use crate::state::AppState;

use super::super::project::{
    build_project_config_state, build_project_export_manifest, resolve_project_root,
};
use super::{
    available_project_directory_name, canonical_regular_directory, normalize_archive_path,
    normalize_local_path, remove_import_staging, unique_sibling_path, write_project_archive,
    ProjectArchiveExportResult, ProjectArchiveImportResult, ProjectArchiveInspection,
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
    let destination = normalize_archive_path(&destination_path)?;
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
    let archive_path = normalize_archive_path(&archive_path)?;
    tokio::task::spawn_blocking(move || inspect_project_package(&archive_path))
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
    let archive_path = normalize_archive_path(&archive_path)?;
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
        extract_project_package(&archive_for_task, &staging_for_task)
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

    let delivery = match validate_project_delivery(&staging_root).await {
        Ok(delivery) => delivery,
        Err(error) => {
            remove_import_staging(&staging_root);
            return Err(format!(
                "Imported project runtime validation could not be completed: {error}"
            ));
        }
    };
    if !delivery.valid {
        let details = delivery_failure_summary(&delivery);
        remove_import_staging(&staging_root);
        return Err(format!(
            "Imported project failed runtime and delivery acceptance: {details}"
        ));
    }

    let directory_name = match available_project_directory_name(
        &destination_parent,
        &verified.project_title,
        &archive_path,
    ) {
        Ok(directory_name) => directory_name,
        Err(error) => {
            remove_import_staging(&staging_root);
            return Err(error);
        }
    };
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
        archive_path: verified.archive_path,
        project_path: destination.to_string_lossy().to_string(),
        project_title: verified.project_title,
        directory_name,
        file_count: verified.file_count,
        total_bytes: verified.total_bytes,
        content_sha256: verified.content_sha256,
    })
}

fn delivery_failure_summary(report: &DeliveryValidationReport) -> String {
    let core = report.core_runtime.issues.iter().map(|issue| {
        issue.path.as_ref().map_or_else(
            || issue.code.clone(),
            |path| format!("{}: {path}", issue.code),
        )
    });
    let assets = report
        .issues
        .iter()
        .filter(|issue| issue.severity == DeliveryIssueSeverity::Error)
        .map(|issue| {
            issue.path.as_ref().map_or_else(
                || issue.code.clone(),
                |path| format!("{}: {path}", issue.code),
            )
        });
    let evidence = core.chain(assets).take(5).collect::<Vec<_>>();
    if evidence.is_empty() {
        "validation failed without issue evidence".to_string()
    } else {
        evidence.join(", ")
    }
}
