//! Project configuration commands for commercial authoring readiness.

use std::collections::{BTreeMap, HashSet};
use std::path::{Component, Path, PathBuf};

use chrono::Utc;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::state::{default_project_data_root, AppState};

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
        key: "events",
        label: "Story Events",
        fallback: "events",
        required: false,
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

const EXPORT_DIRECTORIES: &[(&str, &str)] = &[
    ("characters", "characters"),
    ("dialogue", "dialogue"),
    ("knowledge", "knowledge"),
    ("scenes", "scenes"),
    ("assets", "assets"),
    ("events", "events"),
    ("locales", "locales"),
    ("quality_suites", "quality_suites"),
    ("workflows", "workflows"),
];

const SECRET_CONFIG_KEYS: &[&str] = &[
    "api_key",
    "apiKey",
    "api-token",
    "api_token",
    "authorization",
    "x-api-key",
    "api-key",
    "token",
    "access_token",
    "accessToken",
    "secret",
    "password",
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
    _state: State<'_, AppState>,
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

    let config = scrub_runtime_secret_config(&config);
    let settings_path = root.join("settings.json");
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    tokio::fs::write(&settings_path, json)
        .await
        .map_err(|e| e.to_string())?;

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
    let config = scrub_runtime_secret_config(&config);

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
            "events": "events",
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
    let Some(path) = project_path
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
    else {
        return Ok(default_project_data_root());
    };

    if path.is_absolute() {
        return Ok(path);
    }

    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
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

fn build_project_export_manifest(
    project_root: &Path,
    loaded_characters: Vec<String>,
    loaded_dialogues: Vec<String>,
    loaded_knowledge_count: usize,
    current_scene: Option<String>,
) -> Result<Value, String> {
    let config_state = build_project_config_state(project_root)?;
    let config = config_state.config;
    let characters_path =
        configured_project_path(project_root, &config, "characters", "characters")?;
    let dialogue_path = configured_project_path(project_root, &config, "dialogue", "dialogue")?;
    let knowledge_path = configured_project_path(project_root, &config, "knowledge", "knowledge")?;
    let scenes_path = configured_project_path(project_root, &config, "scenes", "scenes")?;

    let file_characters = collect_json_ids(&characters_path);
    let file_dialogues = collect_json_ids(&dialogue_path);
    let file_knowledge = collect_json_ids(&knowledge_path);
    let file_scenes = collect_json_ids(&scenes_path);
    let files = collect_project_file_inventory(project_root, &config)?;
    let total_bytes: u64 = files
        .iter()
        .filter_map(|file| file.get("size_bytes").and_then(Value::as_u64))
        .sum();
    let content_sha256 = package_content_sha256(&files);
    let content_summary = project_content_summary(&files);

    Ok(json!({
        "format": "monogatari-project",
        "schema": "monogatari-project-export@1",
        "version": "1.0",
        "exported_at": Utc::now().to_rfc3339(),
        "export_metadata": project_export_metadata(),
        "project_path": project_root.to_string_lossy(),
        "settings": sanitize_export_config(&config),
        "content": {
            "characters": if file_characters.is_empty() { loaded_characters } else { file_characters },
            "dialogues": if file_dialogues.is_empty() { loaded_dialogues } else { file_dialogues },
            "knowledge": file_knowledge,
            "knowledge_count": loaded_knowledge_count.max(count_json_files(&knowledge_path)),
            "scenes": file_scenes,
            "current_scene": current_scene,
        },
        "content_summary": content_summary,
        "package": {
            "file_count": files.len(),
            "total_bytes": total_bytes,
            "fingerprint_algorithm": "sha256:path-size-file-sha256-v1",
            "content_sha256": content_sha256,
            "files": files,
            "excluded": ["saves", "analytics.json", ".sync_manifest.json"]
        }
    }))
}

fn project_export_metadata() -> Value {
    json!({
        "engine_version": env!("CARGO_PKG_VERSION"),
        "git_commit": option_env!("MONOGATARI_GIT_COMMIT")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("unknown"),
        "git_short_commit": option_env!("MONOGATARI_GIT_SHORT_COMMIT")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("unknown"),
    })
}

fn configured_project_path(
    project_root: &Path,
    config: &Value,
    key: &str,
    fallback: &str,
) -> Result<PathBuf, String> {
    let relative_path = path_from_config(config, key).unwrap_or_else(|| fallback.to_string());
    resolve_project_relative(project_root, &relative_path)
}

fn collect_project_file_inventory(
    project_root: &Path,
    config: &Value,
) -> Result<Vec<Value>, String> {
    let mut files = Vec::new();
    let mut seen_paths = HashSet::new();

    let settings_path = project_root.join("settings.json");
    if settings_path.is_file() {
        push_export_file(
            project_root,
            "settings",
            &settings_path,
            &mut seen_paths,
            &mut files,
        )?;
    }

    for (key, fallback) in EXPORT_DIRECTORIES {
        let dir = configured_project_path(project_root, config, key, fallback)?;
        if dir.is_dir() {
            collect_export_files(project_root, key, &dir, &mut seen_paths, &mut files)?;
        }
    }

    files.sort_by(|a, b| {
        a.get("path")
            .and_then(Value::as_str)
            .cmp(&b.get("path").and_then(Value::as_str))
    });
    Ok(files)
}

fn collect_export_files(
    project_root: &Path,
    category: &str,
    dir: &Path,
    seen_paths: &mut HashSet<String>,
    files: &mut Vec<Value>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_export_files(project_root, category, &path, seen_paths, files)?;
        } else if path.is_file() {
            push_export_file(project_root, category, &path, seen_paths, files)?;
        }
    }
    Ok(())
}

fn push_export_file(
    project_root: &Path,
    category: &str,
    path: &Path,
    seen_paths: &mut HashSet<String>,
    files: &mut Vec<Value>,
) -> Result<(), String> {
    let relative = project_relative_path(project_root, path);
    if !seen_paths.insert(relative.clone()) {
        return Ok(());
    }
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let checksum_md5 = format!("{:x}", md5::compute(&bytes));
    let checksum_sha256 = checksum_sha256(&bytes);
    files.push(json!({
        "category": category,
        "path": relative,
        "size_bytes": metadata.len(),
        "checksum_md5": checksum_md5,
        "checksum_sha256": checksum_sha256,
    }));
    Ok(())
}

fn project_content_summary(files: &[Value]) -> Value {
    let mut category_counts = BTreeMap::<String, usize>::new();
    let mut category_bytes = BTreeMap::<String, u64>::new();
    let mut category_files = BTreeMap::<String, Vec<&Value>>::new();
    let mut json_file_count = 0usize;
    let mut asset_file_count = 0usize;

    for file in files {
        let category = file
            .get("category")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let size_bytes = file
            .get("size_bytes")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let path = file.get("path").and_then(Value::as_str).unwrap_or("");

        *category_counts.entry(category.clone()).or_insert(0) += 1;
        *category_bytes.entry(category.clone()).or_insert(0) += size_bytes;
        category_files
            .entry(category.clone())
            .or_default()
            .push(file);
        if path.ends_with(".json") {
            json_file_count += 1;
        }
        if category == "assets" {
            asset_file_count += 1;
        }
    }

    json!({
        "schema": "monogatari-project-content-summary/v1",
        "file_count": files.len(),
        "json_file_count": json_file_count,
        "asset_file_count": asset_file_count,
        "category_counts": category_counts,
        "category_bytes": category_bytes,
        "category_fingerprint_algorithm": "sha256:path-size-file-sha256-v1",
        "category_fingerprints": category_files
            .iter()
            .map(|(category, files)| (category.clone(), category_content_sha256(files)))
            .collect::<BTreeMap<_, _>>(),
        "exported_categories": EXPORT_DIRECTORIES
            .iter()
            .map(|(category, _)| *category)
            .collect::<Vec<_>>(),
    })
}

fn checksum_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn package_content_sha256(files: &[Value]) -> String {
    let mut hasher = Sha256::new();
    for file in files {
        update_file_fingerprint(&mut hasher, file);
    }
    finish_sha256(hasher)
}

fn category_content_sha256(files: &[&Value]) -> String {
    let mut hasher = Sha256::new();
    for file in files {
        update_file_fingerprint(&mut hasher, *file);
    }
    finish_sha256(hasher)
}

fn update_file_fingerprint(hasher: &mut Sha256, file: &Value) {
    let path = file.get("path").and_then(Value::as_str).unwrap_or("");
    let size_bytes = file
        .get("size_bytes")
        .and_then(Value::as_u64)
        .unwrap_or_default()
        .to_string();
    let checksum = file
        .get("checksum_sha256")
        .and_then(Value::as_str)
        .unwrap_or("");
    hasher.update(path.as_bytes());
    hasher.update(b"\0");
    hasher.update(size_bytes.as_bytes());
    hasher.update(b"\0");
    hasher.update(checksum.as_bytes());
    hasher.update(b"\n");
}

fn finish_sha256(hasher: Sha256) -> String {
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn project_relative_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn sanitize_export_config(config: &Value) -> Value {
    match config {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if is_secret_config_key(key) {
                        (key.clone(), Value::String("<redacted>".to_string()))
                    } else {
                        (key.clone(), sanitize_export_config(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(sanitize_export_config).collect()),
        _ => config.clone(),
    }
}

fn scrub_runtime_secret_config(config: &Value) -> Value {
    match config {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if is_secret_config_key(key) {
                        (key.clone(), Value::String(String::new()))
                    } else {
                        (key.clone(), scrub_runtime_secret_config(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.iter().map(scrub_runtime_secret_config).collect())
        }
        Value::String(value) => Value::String(scrub_runtime_secret_string(value)),
        _ => config.clone(),
    }
}

fn is_secret_config_key(key: &str) -> bool {
    SECRET_CONFIG_KEYS
        .iter()
        .any(|candidate| key.eq_ignore_ascii_case(candidate))
}

fn scrub_runtime_secret_string(value: &str) -> String {
    let token_redacted = scrub_token_like_values(value);
    scrub_secret_assignments(&token_redacted)
}

fn scrub_token_like_values(value: &str) -> String {
    let mut redacted = String::with_capacity(value.len());
    let mut cursor = 0;
    while cursor < value.len() {
        if let Some((prefix, min_len)) = token_prefix_at(value, cursor) {
            let token_end = token_end(value, cursor);
            let body_len = value[cursor + prefix.len()..token_end]
                .chars()
                .filter(|ch| is_token_char(*ch))
                .count();
            if body_len >= min_len {
                redacted.push_str("<redacted>");
                cursor = token_end;
                continue;
            }
        }

        let ch = value[cursor..]
            .chars()
            .next()
            .expect("cursor at char boundary");
        redacted.push(ch);
        cursor += ch.len_utf8();
    }
    redacted
}

fn token_prefix_at(value: &str, cursor: usize) -> Option<(&'static str, usize)> {
    let rest = &value[cursor..];
    for (prefix, min_len) in [("github_pat_", 20), ("ghp_", 20), ("sk-", 20)] {
        if rest.starts_with(prefix) {
            return Some((prefix, min_len));
        }
    }
    None
}

fn token_end(value: &str, cursor: usize) -> usize {
    value[cursor..]
        .char_indices()
        .find_map(|(offset, ch)| (!is_token_char(ch)).then_some(cursor + offset))
        .unwrap_or(value.len())
}

fn is_token_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

fn scrub_secret_assignments(value: &str) -> String {
    let mut redacted = value.to_string();
    for key in SECRET_CONFIG_KEYS {
        redacted = scrub_assignment_values(&redacted, key);
    }
    redacted
}

fn scrub_assignment_values(value: &str, key: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut cursor = 0;

    while let Some(relative) = find_key_case_insensitive(&value[cursor..], key) {
        let key_start = cursor + relative;
        let key_end = key_start + key.len();
        if !is_secret_key_boundary(value, key_start, key_end) {
            result.push_str(&value[cursor..key_end]);
            cursor = key_end;
            continue;
        }

        let Some((value_start, value_end)) =
            secret_assignment_value_span(value, key_start, key_end)
        else {
            result.push_str(&value[cursor..key_end]);
            cursor = key_end;
            continue;
        };

        result.push_str(&value[cursor..value_start]);
        result.push_str("<redacted>");
        cursor = value_end;
    }

    result.push_str(&value[cursor..]);
    result
}

fn find_key_case_insensitive(value: &str, key: &str) -> Option<usize> {
    value.to_ascii_lowercase().find(&key.to_ascii_lowercase())
}

fn is_secret_key_boundary(value: &str, start: usize, end: usize) -> bool {
    let before = value[..start].chars().next_back();
    let after = value[end..].chars().next();
    !before.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        && !after.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn secret_assignment_value_span(
    value: &str,
    key_start: usize,
    key_end: usize,
) -> Option<(usize, usize)> {
    let mut idx = skip_secret_ws(value, key_end);

    if let Some(quote) = value[..key_start]
        .chars()
        .next_back()
        .filter(|ch| matches!(ch, '"' | '\''))
    {
        if value[idx..].starts_with(quote) {
            idx += quote.len_utf8();
            idx = skip_secret_ws(value, idx);
        }
    }

    if let Some(quote) = value[idx..]
        .chars()
        .next()
        .filter(|ch| matches!(ch, '"' | '\''))
    {
        idx += quote.len_utf8();
        idx = skip_secret_ws(value, idx);
    }

    let separator = value[idx..].chars().next()?;
    if !matches!(separator, ':' | '=') {
        return None;
    }
    idx += separator.len_utf8();
    idx = skip_secret_ws(value, idx);

    if let Some(quote) = value[idx..]
        .chars()
        .next()
        .filter(|ch| matches!(ch, '"' | '\''))
    {
        let value_start = idx + quote.len_utf8();
        let value_end = value[value_start..]
            .char_indices()
            .find_map(|(offset, ch)| (ch == quote).then_some(value_start + offset))
            .unwrap_or(value.len());
        return Some((value_start, value_end));
    }

    let value_start = idx;
    let value_end = value[value_start..]
        .char_indices()
        .find_map(|(offset, ch)| {
            (matches!(ch, '&' | ',' | '}' | ']' | ';') || ch == '\n' || ch == '\r')
                .then_some(value_start + offset)
        })
        .unwrap_or(value.len());
    (value_start < value_end).then_some((value_start, value_end))
}

fn skip_secret_ws(value: &str, start: usize) -> usize {
    value[start..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some(start + offset))
        .unwrap_or(value.len())
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

    #[test]
    fn scrub_runtime_secret_config_removes_sensitive_settings_before_save() {
        let bearer_key = format!("sk-{}", "A".repeat(24));
        let github_key = format!("{}{}", "github_pat_", "B".repeat(24));
        let header_secret = "header-runtime-secret";
        let config = json!({
            "ai": {
                "provider": "api",
                "api": {
                    "base_url": format!("https://example.test/v1?access_token={github_key}&model=chat"),
                    "api_key": "plain-runtime-secret",
                    "apiKey": "legacy-runtime-secret",
                    "authorization": format!("Bearer {bearer_key}"),
                    "model": "test-model",
                    "headers": {
                        "trace": format!("Authorization: Bearer {header_secret}; request-id=kept")
                    }
                }
            },
            "sync": {
                "token": "sync-runtime-secret",
                "endpoint": "https://sync.example.test"
            },
            "nested": [
                { "password": "nested-runtime-secret", "label": "kept" }
            ]
        });

        let scrubbed = scrub_runtime_secret_config(&config);

        assert_eq!(scrubbed["ai"]["api"]["api_key"], "");
        assert_eq!(scrubbed["ai"]["api"]["apiKey"], "");
        assert_eq!(scrubbed["ai"]["api"]["authorization"], "");
        assert_eq!(scrubbed["sync"]["token"], "");
        assert_eq!(scrubbed["nested"][0]["password"], "");
        assert_eq!(
            scrubbed["ai"]["api"]["base_url"],
            "https://example.test/v1?access_token=<redacted>&model=chat"
        );
        assert_eq!(
            scrubbed["ai"]["api"]["headers"]["trace"],
            "Authorization: <redacted>; request-id=kept"
        );
        assert!(!scrubbed.to_string().contains(&bearer_key));
        assert!(!scrubbed.to_string().contains(&github_key));
        assert!(!scrubbed.to_string().contains(header_secret));
        assert_eq!(scrubbed["ai"]["api"]["model"], "test-model");
        assert_eq!(scrubbed["sync"]["endpoint"], "https://sync.example.test");
        assert_eq!(scrubbed["nested"][0]["label"], "kept");
    }

    #[test]
    fn build_state_scrubs_legacy_settings_secrets() {
        let root = std::env::temp_dir().join(format!(
            "monogatari_project_secret_scrub_{}",
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
            json!({
                "ai": {
                    "provider": "api",
                    "api": {
                        "base_url": "https://example.test/v1",
                        "api_key": "legacy-runtime-secret",
                        "model": "test-model"
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let state = build_project_config_state(&root).unwrap();

        assert_eq!(state.config["ai"]["api"]["api_key"], "");
        assert!(state
            .issues
            .iter()
            .any(|issue| issue.code == "api_key_missing"));
        assert!(!state.config.to_string().contains("legacy-runtime-secret"));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn export_manifest_inventories_files_and_redacts_secrets() {
        let root = std::env::temp_dir().join(format!(
            "monogatari_project_export_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for dir in [
            "characters",
            "dialogue",
            "knowledge",
            "scenes",
            "assets/tts",
            "workflows",
            "quality_suites",
            "locales",
            "saves",
        ] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        std::fs::write(
            root.join("settings.json"),
            json!({
                "ai": {
                    "provider": "api",
                    "api": {
                        "api_key": "secret-key",
                        "model": "test-model"
                    }
                },
                "paths": {
                    "characters": "characters",
                    "dialogue": "dialogue",
                    "knowledge": "knowledge",
                    "scenes": "scenes",
                    "assets": "assets"
                }
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            root.join("characters").join("sakura.json"),
            r#"{"id":"sakura"}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("dialogue").join("intro.json"),
            r#"{"id":"intro"}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("knowledge").join("world.json"),
            r#"{"id":"world"}"#,
        )
        .unwrap();
        std::fs::write(root.join("assets").join("tts").join("line.wav"), b"voice").unwrap();
        std::fs::write(root.join("saves").join("slot.json"), b"private save").unwrap();
        std::fs::write(root.join("analytics.json"), b"[]").unwrap();

        let manifest =
            build_project_export_manifest(&root, Vec::new(), Vec::new(), 0, None).unwrap();
        assert_eq!(manifest["schema"], "monogatari-project-export@1");
        assert_eq!(
            manifest["export_metadata"]["engine_version"],
            env!("CARGO_PKG_VERSION")
        );
        assert!(manifest["export_metadata"]["git_commit"]
            .as_str()
            .is_some_and(|value| !value.trim().is_empty()));
        assert!(manifest["export_metadata"]["git_short_commit"]
            .as_str()
            .is_some_and(|value| !value.trim().is_empty()));
        if manifest["export_metadata"]["git_commit"] != "unknown" {
            assert!(manifest["export_metadata"]["git_commit"]
                .as_str()
                .unwrap()
                .starts_with(
                    manifest["export_metadata"]["git_short_commit"]
                        .as_str()
                        .unwrap()
                ));
        }
        assert_eq!(manifest["settings"]["ai"]["api"]["api_key"], "<redacted>");
        assert_eq!(
            manifest["content_summary"]["schema"],
            "monogatari-project-content-summary/v1"
        );
        assert_eq!(
            manifest["content_summary"]["category_counts"]["characters"],
            1
        );
        assert_eq!(
            manifest["content_summary"]["category_counts"]["dialogue"],
            1
        );
        assert_eq!(
            manifest["content_summary"]["category_counts"]["knowledge"],
            1
        );
        assert_eq!(manifest["content_summary"]["category_counts"]["assets"], 1);
        assert_eq!(
            manifest["content_summary"]["category_counts"]["settings"],
            1
        );
        assert_eq!(manifest["content_summary"]["json_file_count"], 4);
        assert_eq!(manifest["content_summary"]["asset_file_count"], 1);
        assert_eq!(manifest["content_summary"]["category_bytes"]["assets"], 5);
        assert_eq!(
            manifest["content_summary"]["category_fingerprint_algorithm"],
            "sha256:path-size-file-sha256-v1"
        );
        let asset_category_sha256 = manifest["content_summary"]["category_fingerprints"]["assets"]
            .as_str()
            .expect("asset category fingerprint");
        assert_eq!(asset_category_sha256.len(), 64);
        assert!(manifest["content_summary"]["exported_categories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|category| category.as_str() == Some("workflows")));
        assert_eq!(
            manifest["package"]["fingerprint_algorithm"],
            "sha256:path-size-file-sha256-v1"
        );
        let package_sha256 = manifest["package"]["content_sha256"]
            .as_str()
            .expect("package content hash");
        assert_eq!(package_sha256.len(), 64);
        let files = manifest["package"]["files"].as_array().unwrap();
        let tts_file = files
            .iter()
            .find(|file| file["path"] == "assets/tts/line.wav")
            .expect("tts asset should be included");
        assert_eq!(
            tts_file["checksum_sha256"],
            "c57d7e92019708b614c90fa3685cd644f543a60153fb99ec9b67c381a245fb2a"
        );
        assert!(tts_file["checksum_md5"].as_str().is_some());
        assert_eq!(tts_file["checksum_sha256"].as_str().unwrap().len(), 64);
        assert!(files.iter().any(|file| file["path"] == "settings.json"));
        assert!(!files.iter().any(|file| file["path"] == "saves/slot.json"));
        assert!(!files.iter().any(|file| file["path"] == "analytics.json"));

        let repeat_manifest =
            build_project_export_manifest(&root, Vec::new(), Vec::new(), 0, None).unwrap();
        assert_eq!(
            repeat_manifest["package"]["content_sha256"],
            manifest["package"]["content_sha256"]
        );
        assert_eq!(
            repeat_manifest["content_summary"]["category_fingerprints"],
            manifest["content_summary"]["category_fingerprints"]
        );

        std::fs::write(
            root.join("assets").join("tts").join("line.wav"),
            b"new voice",
        )
        .unwrap();
        let changed_manifest =
            build_project_export_manifest(&root, Vec::new(), Vec::new(), 0, None).unwrap();
        assert_ne!(
            changed_manifest["package"]["content_sha256"],
            manifest["package"]["content_sha256"]
        );
        assert_ne!(
            changed_manifest["content_summary"]["category_fingerprints"]["assets"],
            manifest["content_summary"]["category_fingerprints"]["assets"]
        );

        std::fs::remove_dir_all(root).unwrap();
    }
}
