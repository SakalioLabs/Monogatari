//! Project configuration commands for commercial authoring readiness.

use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use chrono::Utc;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectPathStatus {
    pub key: String,
    pub label: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub exists: bool,
    pub item_count: usize,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectConfigIssue {
    pub severity: String,
    pub code: String,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectConfigState {
    pub project_path: String,
    pub settings_path: String,
    pub settings_exists: bool,
    pub valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub config: Value,
    pub paths: Vec<ProjectPathStatus>,
    pub issues: Vec<ProjectConfigIssue>,
}

#[derive(Debug, Clone, Copy)]
struct PathDefinition {
    key: &'static str,
    label: &'static str,
    fallback: &'static str,
    required: bool,
}

const PROJECT_PATHS: &[PathDefinition] = &[
    PathDefinition {
        key: "characters",
        label: "Characters",
        fallback: "characters",
        required: true,
    },
    PathDefinition {
        key: "dialogue",
        label: "Dialogue",
        fallback: "dialogue",
        required: true,
    },
    PathDefinition {
        key: "knowledge",
        label: "Knowledge",
        fallback: "knowledge",
        required: true,
    },
    PathDefinition {
        key: "scenes",
        label: "Scenes",
        fallback: "scenes",
        required: false,
    },
    PathDefinition {
        key: "assets",
        label: "Assets",
        fallback: "assets",
        required: true,
    },
    PathDefinition {
        key: "saves",
        label: "Saves",
        fallback: "saves",
        required: false,
    },
    PathDefinition {
        key: "quality_suites",
        label: "Quality Suites",
        fallback: "quality_suites",
        required: false,
    },
];

/// Load project settings and readiness diagnostics.
#[tauri::command]
pub async fn get_project_config(
    state: State<'_, AppState>,
    project_path: Option<String>,
) -> Result<ProjectConfigState, String> {
    let root = resolve_project_root(&state, project_path).await?;
    build_project_config_state(&root)
}

/// Save project settings.json and return refreshed diagnostics.
#[tauri::command]
pub async fn save_project_config(
    state: State<'_, AppState>,
    project_path: String,
    config: Value,
) -> Result<ProjectConfigState, String> {
    let root = normalize_project_path(Some(project_path))?;
    if !root.exists() {
        return Err(format!("Project path does not exist: {}", root.display()));
    }
    if !root.is_dir() {
        return Err(format!(
            "Project path is not a directory: {}",
            root.display()
        ));
    }
    if !config.is_object() {
        return Err("Project config must be a JSON object.".to_string());
    }

    let settings_path = root.join("settings.json");
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    tokio::fs::write(&settings_path, json)
        .await
        .map_err(|e| e.to_string())?;

    *state.project_path.write().await = Some(root.clone());
    build_project_config_state(&root)
}

fn build_project_config_state(project_root: &Path) -> Result<ProjectConfigState, String> {
    let settings_path = project_root.join("settings.json");
    let settings_exists = settings_path.exists();
    let mut issues = Vec::new();
    let config = if settings_exists {
        match std::fs::read_to_string(&settings_path)
            .map_err(|e| e.to_string())
            .and_then(|content| serde_json::from_str::<Value>(&content).map_err(|e| e.to_string()))
        {
            Ok(value) if value.is_object() => merge_with_default_config(value),
            Ok(_) => {
                push_issue(
                    &mut issues,
                    "error",
                    "settings_not_object",
                    Some("settings.json".to_string()),
                    "settings.json must contain a JSON object.",
                );
                default_project_config()
            }
            Err(error) => {
                push_issue(
                    &mut issues,
                    "error",
                    "settings_invalid_json",
                    Some("settings.json".to_string()),
                    format!("Unable to parse settings.json: {error}"),
                );
                default_project_config()
            }
        }
    } else {
        push_issue(
            &mut issues,
            "warning",
            "settings_missing",
            Some("settings.json".to_string()),
            "Project has no settings.json; defaults are being used.",
        );
        default_project_config()
    };

    let paths = collect_path_statuses(project_root, &config, &mut issues);
    validate_ai_config(&config, &mut issues);

    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .count();
    let warning_count = issues
        .iter()
        .filter(|issue| issue.severity == "warning")
        .count();

    Ok(ProjectConfigState {
        project_path: project_root.to_string_lossy().to_string(),
        settings_path: settings_path.to_string_lossy().to_string(),
        settings_exists,
        valid: error_count == 0,
        error_count,
        warning_count,
        config,
        paths,
        issues,
    })
}

fn collect_path_statuses(
    project_root: &Path,
    config: &Value,
    issues: &mut Vec<ProjectConfigIssue>,
) -> Vec<ProjectPathStatus> {
    let mut seen_paths = HashSet::new();
    let mut statuses = Vec::new();

    for definition in PROJECT_PATHS {
        let relative_path = path_from_config(config, definition.key)
            .unwrap_or_else(|| definition.fallback.to_string());

        match resolve_project_relative(project_root, &relative_path) {
            Ok(absolute_path) => {
                let exists = absolute_path.exists();
                if definition.required && !exists {
                    push_issue(
                        issues,
                        "error",
                        "project_path_missing",
                        Some(format!("paths.{}", definition.key)),
                        format!("{} directory is missing.", definition.label),
                    );
                }
                if !seen_paths.insert(absolute_path.clone()) {
                    push_issue(
                        issues,
                        "warning",
                        "project_path_duplicate",
                        Some(format!("paths.{}", definition.key)),
                        "Multiple project path entries resolve to the same directory.",
                    );
                }

                statuses.push(ProjectPathStatus {
                    key: definition.key.to_string(),
                    label: definition.label.to_string(),
                    relative_path,
                    absolute_path: absolute_path.to_string_lossy().to_string(),
                    exists,
                    item_count: count_directory_items(&absolute_path),
                    required: definition.required,
                });
            }
            Err(error) => {
                push_issue(
                    issues,
                    "error",
                    "project_path_invalid",
                    Some(format!("paths.{}", definition.key)),
                    error,
                );
                statuses.push(ProjectPathStatus {
                    key: definition.key.to_string(),
                    label: definition.label.to_string(),
                    relative_path,
                    absolute_path: String::new(),
                    exists: false,
                    item_count: 0,
                    required: definition.required,
                });
            }
        }
    }

    statuses
}

fn validate_ai_config(config: &Value, issues: &mut Vec<ProjectConfigIssue>) {
    let provider = get_string(config, &["ai", "provider"]).unwrap_or_else(|| "api".to_string());
    match provider.as_str() {
        "api" => {
            if get_string(config, &["ai", "api", "base_url"])
                .or_else(|| get_string(config, &["ai", "api", "baseUrl"]))
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                push_issue(
                    issues,
                    "warning",
                    "api_base_url_missing",
                    Some("ai.api.base_url".to_string()),
                    "API base URL is empty.",
                );
            }
            if get_string(config, &["ai", "api", "api_key"])
                .or_else(|| get_string(config, &["ai", "api", "apiKey"]))
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                push_issue(
                    issues,
                    "warning",
                    "api_key_missing",
                    Some("ai.api.api_key".to_string()),
                    "API key is not configured.",
                );
            }
        }
        "onnx" => {
            if get_string(config, &["ai", "onnx", "model_path"])
                .or_else(|| get_string(config, &["ai", "onnx", "modelPath"]))
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                push_issue(
                    issues,
                    "warning",
                    "onnx_model_missing",
                    Some("ai.onnx.model_path".to_string()),
                    "ONNX model path is empty.",
                );
            }
        }
        _ => push_issue(
            issues,
            "error",
            "ai_provider_invalid",
            Some("ai.provider".to_string()),
            "AI provider must be `api` or `onnx`.",
        ),
    }
}

fn merge_with_default_config(mut config: Value) -> Value {
    merge_json(&mut config, default_project_config());
    config
}

fn merge_json(target: &mut Value, defaults: Value) {
    let (Some(target_map), Some(default_map)) = (target.as_object_mut(), defaults.as_object())
    else {
        return;
    };

    for (key, default_value) in default_map {
        match target_map.get_mut(key) {
            Some(target_value) if target_value.is_object() && default_value.is_object() => {
                merge_json(target_value, default_value.clone());
            }
            Some(_) => {}
            None => {
                target_map.insert(key.clone(), default_value.clone());
            }
        }
    }
}

fn default_project_config() -> Value {
    json!({
        "engine": {
            "target_fps": 60,
            "vsync": true,
            "debug_mode": false
        },
        "render": {
            "width": 1280,
            "height": 720,
            "fullscreen": false,
            "title": "LLM Galgame Engine"
        },
        "dialogue": {
            "typewriter_speed_ms": 30,
            "auto_advance_seconds": 0,
            "text_font_size": 18,
            "name_font_size": 16
        },
        "ai": {
            "provider": "api",
            "api": {
                "base_url": "https://api.openai.com/v1",
                "api_key": "",
                "model": "gpt-4o-mini",
                "max_tokens": 512,
                "temperature": 0.7,
                "timeout_seconds": 60
            },
            "onnx": {
                "model_path": "models/model.onnx",
                "tokenizer_path": "models/tokenizer.json",
                "max_sequence_length": 2048,
                "use_directml": true
            }
        },
        "paths": {
            "characters": "characters",
            "dialogue": "dialogue",
            "knowledge": "knowledge",
            "scenes": "scenes",
            "assets": "assets",
            "saves": "saves"
        }
    })
}

fn path_from_config(config: &Value, key: &str) -> Option<String> {
    get_string(config, &["paths", key])
}

fn get_string(config: &Value, path: &[&str]) -> Option<String> {
    let mut current = config;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(str::to_string)
}

fn count_directory_items(path: &Path) -> usize {
    if !path.is_dir() {
        return 0;
    }
    std::fs::read_dir(path)
        .map(|entries| entries.filter_map(Result::ok).count())
        .unwrap_or(0)
}

async fn resolve_project_root(
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
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let Some(path) = project_path
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
    else {
        return Ok(find_upward(&current_dir, Path::new("data")).unwrap_or(current_dir));
    };

    if path.is_absolute() {
        return Ok(path);
    }

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

fn resolve_project_relative(project_root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() {
        return Err("Project paths must be relative to the project root.".to_string());
    }
    if relative_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir
        )
    }) {
        return Err("Project paths cannot escape the project root.".to_string());
    }
    Ok(project_root.join(relative_path))
}

fn push_issue(
    issues: &mut Vec<ProjectConfigIssue>,
    severity: impl Into<String>,
    code: impl Into<String>,
    path: Option<String>,
    message: impl Into<String>,
) {
    issues.push(ProjectConfigIssue {
        severity: severity.into(),
        code: code.into(),
        path,
        message: message.into(),
    });
}

/// Export project as a distributable package manifest.
/// Returns a JSON manifest of all project content for packaging.
#[tauri::command]
pub async fn export_project(
    state: State<'_, AppState>,
    project_path: Option<String>,
) -> Result<Value, String> {
    let root = resolve_project_root(&state, project_path).await?;

    let cm = state.character_manager.read().await;
    let dm = state.dialogue_manager.read().await;
    let kb = state.knowledge_base.read().await;
    let sm = state.scene_manager.read().await;

    let file_characters = collect_json_ids(&root.join("characters"));
    let file_dialogues = collect_json_ids(&root.join("dialogue"));
    let file_knowledge = collect_json_ids(&root.join("knowledge"));
    let file_scenes = collect_json_ids(&root.join("scenes"));

    let loaded_characters = cm.character_ids();
    let loaded_dialogues = dm.script_ids();
    let current_scene = sm.current_scene_name().map(str::to_string);

    let manifest = json!({
        "format": "monogatari-project",
        "version": "1.0",
        "exported_at": Utc::now().to_rfc3339(),
        "project_path": root.to_string_lossy(),
        "content": {
            "characters": if file_characters.is_empty() { loaded_characters } else { file_characters },
            "dialogues": if file_dialogues.is_empty() { loaded_dialogues } else { file_dialogues },
            "knowledge": file_knowledge,
            "knowledge_count": kb.len().max(count_json_files(&root.join("knowledge"))),
            "scenes": file_scenes,
            "current_scene": current_scene,
        }
    });

    Ok(manifest)
}

fn collect_json_ids(dir: &Path) -> Vec<String> {
    let mut ids = Vec::new();
    if !dir.is_dir() {
        return ids;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return ids;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let parsed_id = std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<Value>(&content).ok())
            .and_then(|value| value.get("id").and_then(Value::as_str).map(str::to_string));

        let fallback_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(str::to_string);

        if let Some(id) = parsed_id.or(fallback_id) {
            ids.push(id);
        }
    }

    ids.sort();
    ids
}

fn count_json_files(dir: &Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }

    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("json"))
                .count()
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_project_paths_that_escape_root() {
        let root = PathBuf::from("project");
        assert!(resolve_project_relative(&root, "../characters").is_err());
        assert!(resolve_project_relative(&root, "characters").is_ok());
    }

    #[test]
    fn merges_defaults_without_overwriting_existing_values() {
        let config = merge_with_default_config(json!({
            "ai": {
                "provider": "onnx",
                "api": {
                    "model": "custom-model"
                }
            }
        }));

        assert_eq!(get_string(&config, &["ai", "provider"]).unwrap(), "onnx");
        assert_eq!(
            get_string(&config, &["ai", "api", "model"]).unwrap(),
            "custom-model"
        );
        assert_eq!(
            get_string(&config, &["paths", "characters"]).unwrap(),
            "characters"
        );
    }

    #[test]
    fn builds_state_for_complete_project() {
        let root = std::env::temp_dir().join(format!(
            "monogatari_project_config_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for dir in ["characters", "dialogue", "knowledge", "assets"] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        std::fs::write(
            root.join("settings.json"),
            default_project_config().to_string(),
        )
        .unwrap();

        let state = build_project_config_state(&root).unwrap();
        assert!(state.valid, "{:?}", state.issues);
        assert_eq!(state.paths.len(), PROJECT_PATHS.len());
        assert!(state.paths.iter().any(|path| path.key == "characters"));

        std::fs::remove_dir_all(root).unwrap();
    }
}
