//! Tauri adapters for project configuration and export-manifest inspection.

use std::path::{Path, PathBuf};

use chrono::Utc;
pub(crate) use llm_authoring::project::inspect_project_config as build_project_config_state;
use llm_authoring::project::save_project_config as save_project_config_to_disk;
pub use llm_authoring::project::ProjectConfigState;
use llm_authoring::project_package::{
    build_project_export_manifest as build_headless_project_export_manifest,
    ProjectExportProvenance, ProjectExportRuntimeSnapshot,
};
use serde_json::Value;
use tauri::State;

use crate::state::{default_project_data_root, AppState};

/// Load project settings and readiness diagnostics.
#[tauri::command]
pub async fn get_project_config(
    state: State<'_, AppState>,
    project_path: Option<String>,
) -> Result<ProjectConfigState, String> {
    let root = resolve_project_root(&state, project_path).await?;
    build_project_config_state(&root)
}

/// Save settings.json and return refreshed diagnostics.
#[tauri::command]
pub async fn save_project_config(
    state: State<'_, AppState>,
    project_path: String,
    config: Value,
) -> Result<ProjectConfigState, String> {
    let root = normalize_project_path(Some(project_path))?;
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    save_project_config_to_disk(&root, config).await
}

pub(crate) async fn resolve_project_root(
    state: &State<'_, AppState>,
    project_path: Option<String>,
) -> Result<PathBuf, String> {
    if let Some(path) = project_path {
        return normalize_project_path(Some(path));
    }
    if let Some(path) = state.project_path.read().await.clone() {
        return Ok(path);
    }
    normalize_project_path(None)
}

fn normalize_project_path(project_path: Option<String>) -> Result<PathBuf, String> {
    let Some(path) = project_path
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
    else {
        return Ok(default_project_data_root());
    };

    if path.is_absolute() {
        return Ok(path);
    }

    let current_dir = std::env::current_dir().map_err(|error| error.to_string())?;
    let direct = current_dir.join(&path);
    if direct.exists() {
        return Ok(direct);
    }

    Ok(find_upward(&current_dir, &path).unwrap_or(direct))
}

fn find_upward(start: &Path, relative: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|ancestor| ancestor.join(relative))
        .find(|candidate| candidate.exists())
}

/// Return a distributable project manifest without writing an archive.
#[tauri::command]
pub async fn export_project(
    state: State<'_, AppState>,
    project_path: Option<String>,
) -> Result<Value, String> {
    let root = resolve_project_root(&state, project_path).await?;
    let loaded_characters = state.character_manager.read().await.character_ids();
    let loaded_dialogues = state.dialogue_manager.read().await.script_ids();
    let loaded_knowledge_count = state.knowledge_base.read().await.len();
    let current_scene = state
        .scene_manager
        .read()
        .await
        .current_scene_name()
        .map(str::to_string);

    build_project_export_manifest(
        &root,
        loaded_characters,
        loaded_dialogues,
        loaded_knowledge_count,
        current_scene,
    )
}

pub(crate) fn build_project_export_manifest(
    project_root: &Path,
    loaded_characters: Vec<String>,
    loaded_dialogues: Vec<String>,
    loaded_knowledge_count: usize,
    current_scene: Option<String>,
) -> Result<Value, String> {
    build_headless_project_export_manifest(
        project_root,
        ProjectExportRuntimeSnapshot {
            loaded_characters,
            loaded_dialogues,
            loaded_knowledge_count,
            current_scene,
        },
        project_export_provenance(),
    )
}

fn project_export_provenance() -> ProjectExportProvenance {
    ProjectExportProvenance {
        exported_at: Utc::now().to_rfc3339(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: build_git_value(option_env!("MONOGATARI_GIT_COMMIT")),
        git_short_commit: build_git_value(option_env!("MONOGATARI_GIT_SHORT_COMMIT")),
    }
}

fn build_git_value(value: Option<&str>) -> String {
    value
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_provenance_uses_the_tauri_build_identity() {
        let provenance = project_export_provenance();
        assert_eq!(provenance.engine_version, env!("CARGO_PKG_VERSION"));
        assert!(!provenance.exported_at.trim().is_empty());
        assert!(!provenance.git_commit.trim().is_empty());
        assert!(!provenance.git_short_commit.trim().is_empty());
        if provenance.git_commit != "unknown" {
            assert!(provenance
                .git_commit
                .starts_with(&provenance.git_short_commit));
        }
    }
}
