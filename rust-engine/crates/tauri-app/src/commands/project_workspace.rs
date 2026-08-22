//! Desktop project launcher, recent-project registry, and empty-project creation.

use std::path::{Path, PathBuf};

use chrono::Utc;
use llm_authoring::filesystem::stage_json_replacement;
use llm_authoring::project::{inspect_project_config, project_title};
use llm_authoring::project_creation::create_empty_project;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::state::AppState;

use super::engine::{
    activate_project, emit_project_activity, ProjectActivityOperation, ProjectActivityPhase,
};

const REGISTRY_SCHEMA: &str = "monogatari-project-registry/v1";
const MAX_RECENT_PROJECTS: usize = 50;
const MAX_REGISTRY_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectLauncherEntry {
    pub project_path: String,
    pub project_title: String,
    pub last_opened_at: String,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SampleProjectEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    pub package_file_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectWorkspaceState {
    pub active_project: Option<ProjectLauncherEntry>,
    pub recent_projects: Vec<ProjectLauncherEntry>,
    pub sample_projects: Vec<SampleProjectEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectRegistry {
    schema: String,
    projects: Vec<ProjectRegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectRegistryEntry {
    project_path: String,
    project_title: String,
    last_opened_at: String,
}

impl Default for ProjectRegistry {
    fn default() -> Self {
        Self {
            schema: REGISTRY_SCHEMA.to_string(),
            projects: Vec::new(),
        }
    }
}

#[tauri::command]
pub async fn get_project_workspace(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ProjectWorkspaceState, String> {
    build_workspace_state(&app, &state).await
}

#[tauri::command]
pub async fn open_project(
    app: AppHandle,
    state: State<'_, AppState>,
    project_path: String,
) -> Result<ProjectLauncherEntry, String> {
    let path = normalize_existing_project_path(&project_path)?;
    activate_project(&app, &state, path.clone(), ProjectActivityOperation::Open).await?;
    record_project(&app, &path).await
}

#[tauri::command]
pub async fn create_project(
    app: AppHandle,
    state: State<'_, AppState>,
    parent_directory: String,
    directory_name: String,
    project_title: String,
) -> Result<ProjectLauncherEntry, String> {
    let parent = normalize_existing_directory(&parent_directory, "Project parent directory")?;
    emit_project_activity(
        &app,
        ProjectActivityOperation::Create,
        ProjectActivityPhase::CheckingProject,
        Some(&parent),
    );
    let created = create_empty_project(&parent, &directory_name, &project_title)?;
    let project_path = PathBuf::from(&created.project_path);
    if let Err(error) = activate_project(
        &app,
        &state,
        project_path.clone(),
        ProjectActivityOperation::Create,
    )
    .await
    {
        let _ = std::fs::remove_dir_all(&project_path);
        return Err(format!(
            "The project was created but failed initial validation: {error}"
        ));
    }
    record_project(&app, &project_path).await
}

#[tauri::command]
pub async fn forget_project(
    app: AppHandle,
    project_path: String,
) -> Result<ProjectWorkspaceState, String> {
    let registry_path = registry_path(&app)?;
    let mut registry = load_registry(&registry_path)?;
    let target = normalized_registry_key(&project_path);
    registry
        .projects
        .retain(|entry| normalized_registry_key(&entry.project_path) != target);
    save_registry(&registry_path, &registry).await?;
    Ok(ProjectWorkspaceState {
        active_project: None,
        recent_projects: registry_entries(&registry),
        sample_projects: sample_projects(),
    })
}

#[tauri::command]
pub async fn close_project(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ProjectWorkspaceState, String> {
    state.clear_active_project().await;
    build_workspace_state(&app, &state).await
}

async fn build_workspace_state(
    app: &AppHandle,
    state: &AppState,
) -> Result<ProjectWorkspaceState, String> {
    let registry = load_registry(&registry_path(app)?)?;
    let active_path = state.active_project_data_root().await;
    let active_project = active_path
        .as_deref()
        .map(project_entry_from_path)
        .transpose()?;
    Ok(ProjectWorkspaceState {
        active_project,
        recent_projects: registry_entries(&registry),
        sample_projects: sample_projects(),
    })
}

async fn record_project(app: &AppHandle, path: &Path) -> Result<ProjectLauncherEntry, String> {
    let entry = project_entry_from_path(path)?;
    let registry_path = registry_path(app)?;
    let mut registry = load_registry(&registry_path)?;
    let key = normalized_registry_key(&entry.project_path);
    registry
        .projects
        .retain(|candidate| normalized_registry_key(&candidate.project_path) != key);
    registry.projects.insert(
        0,
        ProjectRegistryEntry {
            project_path: entry.project_path.clone(),
            project_title: entry.project_title.clone(),
            last_opened_at: entry.last_opened_at.clone(),
        },
    );
    registry.projects.truncate(MAX_RECENT_PROJECTS);
    save_registry(&registry_path, &registry).await?;
    Ok(entry)
}

fn project_entry_from_path(path: &Path) -> Result<ProjectLauncherEntry, String> {
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("Unable to resolve project path: {error}"))?;
    let config = inspect_project_config(&canonical)?;
    let title = project_title(&config.config);
    Ok(ProjectLauncherEntry {
        project_path: canonical.to_string_lossy().to_string(),
        project_title: title,
        last_opened_at: Utc::now().to_rfc3339(),
        available: true,
    })
}

fn registry_entries(registry: &ProjectRegistry) -> Vec<ProjectLauncherEntry> {
    registry
        .projects
        .iter()
        .map(|entry| ProjectLauncherEntry {
            project_path: entry.project_path.clone(),
            project_title: entry.project_title.clone(),
            last_opened_at: entry.last_opened_at.clone(),
            available: Path::new(&entry.project_path)
                .join("settings.json")
                .is_file(),
        })
        .collect()
}

fn sample_projects() -> Vec<SampleProjectEntry> {
    vec![
        SampleProjectEntry {
            id: "blue-frame".to_string(),
            title: "潮镜：蓝色定格".to_string(),
            description: "自由输入 LLM NPC、确定性评分与安全防护示例。项目内容通过独立项目包分发。"
                .to_string(),
            package_file_name: "monogatari-blue-frame-sample.monogatari".to_string(),
        },
        SampleProjectEntry {
            id: "konosuba".to_string(),
            title: "KonoSuba".to_string(),
            description: "多章节 Campaign 与 Scene Roleplay 综合示例。项目内容通过独立项目包分发。"
                .to_string(),
            package_file_name: "monogatari-konosuba-sample.monogatari".to_string(),
        },
    ]
}

fn registry_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Unable to resolve application data directory: {error}"))?;
    std::fs::create_dir_all(&app_data)
        .map_err(|error| format!("Unable to create application data directory: {error}"))?;
    Ok(app_data.join("projects.json"))
}

fn load_registry(path: &Path) -> Result<ProjectRegistry, String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ProjectRegistry::default());
        }
        Err(error) => return Err(format!("Unable to inspect project registry: {error}")),
    };
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_REGISTRY_BYTES
    {
        return Err("Project registry must be a regular JSON file smaller than 1 MiB.".to_string());
    }
    let bytes =
        std::fs::read(path).map_err(|error| format!("Unable to read project registry: {error}"))?;
    let registry: ProjectRegistry = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Project registry is invalid: {error}"))?;
    if registry.schema != REGISTRY_SCHEMA || registry.projects.len() > MAX_RECENT_PROJECTS {
        return Err("Project registry schema or entry count is invalid.".to_string());
    }
    Ok(registry)
}

async fn save_registry(path: &Path, registry: &ProjectRegistry) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(registry)
        .map_err(|error| format!("Unable to encode project registry: {error}"))?;
    if bytes.len() as u64 > MAX_REGISTRY_BYTES {
        return Err("Project registry exceeds 1 MiB.".to_string());
    }
    stage_json_replacement(path, &bytes, MAX_REGISTRY_BYTES, "project registry")
        .await?
        .commit()
        .await
}

fn normalize_existing_project_path(value: &str) -> Result<PathBuf, String> {
    let path = normalize_existing_directory(value, "Project directory")?;
    if !path.join("settings.json").is_file() {
        return Err("Project directory must contain settings.json.".to_string());
    }
    Ok(path)
}

fn normalize_existing_directory(value: &str, label: &str) -> Result<PathBuf, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().any(char::is_control) || value.contains("://") {
        return Err(format!("{label} must be a local filesystem path."));
    }
    let path = PathBuf::from(value);
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|error| format!("Unable to inspect {label}: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{label} must be a regular directory."));
    }
    path.canonicalize()
        .map_err(|error| format!("Unable to resolve {label}: {error}"))
}

fn normalized_registry_key(value: &str) -> String {
    if cfg!(windows) {
        value.replace('\\', "/").to_lowercase()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_entry_uses_the_project_engine_title() {
        let root =
            std::env::temp_dir().join(format!("monogatari-project-entry-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("settings.json"),
            br#"{"engine":{"title":"Independent Story"}}"#,
        )
        .unwrap();

        let entry = project_entry_from_path(&root).unwrap();

        assert_eq!(entry.project_title, "Independent Story");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recent_entries_report_missing_projects_without_dropping_history() {
        let registry = ProjectRegistry {
            schema: REGISTRY_SCHEMA.to_string(),
            projects: vec![ProjectRegistryEntry {
                project_path: "Z:/missing/monogatari-project".to_string(),
                project_title: "Missing".to_string(),
                last_opened_at: "2026-07-29T00:00:00Z".to_string(),
            }],
        };
        let entries = registry_entries(&registry);
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].available);
    }

    #[test]
    fn sample_catalog_contains_metadata_but_no_project_paths() {
        let samples = sample_projects();
        assert_eq!(samples.len(), 2);
        assert!(samples
            .iter()
            .all(|sample| sample.package_file_name.ends_with(".monogatari")));
    }
}
