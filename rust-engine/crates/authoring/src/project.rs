//! Headless project configuration inspection and persistence.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::filesystem::stage_json_replacement;
use crate::paths::resolve_project_relative;

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

pub const MAX_PROJECT_SETTINGS_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProjectPathStatus {
    pub key: String,
    pub label: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub exists: bool,
    pub item_count: usize,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProjectConfigIssue {
    pub severity: String,
    pub code: String,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
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
        key: "endings",
        label: "Story Endings",
        fallback: "endings",
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
    PathDefinition {
        key: "roleplays",
        label: "Scene Roleplays",
        fallback: "roleplays",
        required: false,
    },
];

/// Inspect project settings and return stable readiness diagnostics.
pub fn inspect_project_config(project_root: &Path) -> Result<ProjectConfigState, String> {
    let project_root = canonical_project_root_if_present(project_root)?;
    let settings_path = project_root.join("settings.json");
    let mut issues = Vec::new();
    let settings_metadata = match std::fs::symlink_metadata(&settings_path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(format!(
                "Unable to inspect project settings `{}`: {error}",
                settings_path.display()
            ));
        }
    };
    let settings_exists = settings_metadata.is_some();
    let config = if let Some(metadata) = settings_metadata {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            push_issue(
                &mut issues,
                "error",
                "settings_not_regular_file",
                Some("settings.json".to_string()),
                "settings.json must be a regular file inside the project root.",
            );
            default_project_config()
        } else if metadata.len() > MAX_PROJECT_SETTINGS_BYTES {
            push_issue(
                &mut issues,
                "error",
                "settings_too_large",
                Some("settings.json".to_string()),
                format!(
                    "settings.json is {} bytes; the limit is {MAX_PROJECT_SETTINGS_BYTES} bytes.",
                    metadata.len()
                ),
            );
            default_project_config()
        } else {
            match std::fs::read_to_string(&settings_path)
                .map_err(|error| error.to_string())
                .and_then(|content| {
                    serde_json::from_str::<Value>(&content).map_err(|error| error.to_string())
                }) {
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

    let paths = collect_path_statuses(&project_root, &config, &mut issues);
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

/// Persist scrubbed settings atomically, then return the refreshed project state.
pub async fn save_project_config(
    project_root: &Path,
    config: Value,
) -> Result<ProjectConfigState, String> {
    let project_root = canonical_project_root(project_root)?;
    if !config.is_object() {
        return Err("Project config must be a JSON object.".to_string());
    }

    let config = scrub_runtime_secret_config(&config);
    let settings_path = project_root.join("settings.json");
    let json = serde_json::to_vec_pretty(&config).map_err(|error| error.to_string())?;
    let staged = stage_json_replacement(
        &settings_path,
        &json,
        MAX_PROJECT_SETTINGS_BYTES,
        "project settings",
    )
    .await?;
    match inspect_project_config(&project_root) {
        Ok(project_state) => {
            staged.commit().await?;
            Ok(project_state)
        }
        Err(error) => {
            staged.rollback().await?;
            Err(error)
        }
    }
}

/// Replace secret fields with an explicit marker for exported manifests.
pub fn sanitize_export_config(config: &Value) -> Value {
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

/// Remove runtime credentials and redact token-shaped strings before persistence or display.
pub fn scrub_runtime_secret_config(config: &Value) -> Value {
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

pub fn default_project_config() -> Value {
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
            "title": "Monogatari Engine"
        },
        "dialogue": {
            "typewriter_speed_ms": 30,
            "auto_advance_seconds": 0,
            "text_font_size": 18,
            "name_font_size": 16
        },
        "ai": {
            "provider": "onnx",
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
            },
            "webgpu": {
                "model_id": "onnx-community/Qwen3.5-0.8B-Text-ONNX",
                "dtype": "q4",
                "max_new_tokens": 96,
                "temperature": 0.7,
                "top_p": 0.9
            }
        },
        "paths": {
            "characters": "characters",
            "dialogue": "dialogue",
            "knowledge": "knowledge",
            "scenes": "scenes",
            "assets": "assets",
            "events": "events",
            "endings": "endings",
            "saves": "saves",
            "quality_suites": "quality_suites",
            "roleplays": "roleplays"
        }
    })
}

fn canonical_project_root_if_present(project_root: &Path) -> Result<PathBuf, String> {
    match std::fs::symlink_metadata(project_root) {
        Ok(_) => canonical_project_root(project_root),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(project_root.to_path_buf())
        }
        Err(error) => Err(format!(
            "Unable to inspect project path `{}`: {error}",
            project_root.display()
        )),
    }
}

/// Resolve an existing project root without following a root-level symbolic link.
pub fn canonical_project_root(project_root: &Path) -> Result<PathBuf, String> {
    let metadata = std::fs::symlink_metadata(project_root).map_err(|error| {
        format!(
            "Unable to inspect project path `{}`: {error}",
            project_root.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "Project path must be a regular directory: {}",
            project_root.display()
        ));
    }
    project_root.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve project path `{}`: {error}",
            project_root.display()
        )
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
    let provider = get_string(config, &["ai", "provider"]).unwrap_or_else(|| "onnx".to_string());
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
        "webgpu" => {
            if get_string(config, &["ai", "webgpu", "model_id"])
                .or_else(|| get_string(config, &["ai", "webgpu", "modelId"]))
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                push_issue(
                    issues,
                    "warning",
                    "webgpu_model_missing",
                    Some("ai.webgpu.model_id".to_string()),
                    "WebGPU model ID is empty.",
                );
            }
        }
        _ => push_issue(
            issues,
            "error",
            "ai_provider_invalid",
            Some("ai.provider".to_string()),
            "AI provider must be `api`, `onnx`, or `webgpu`.",
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

/// Read a configured project content path without resolving it.
pub fn path_from_config(config: &Value, key: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari-authoring-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn make_complete_project(root: &Path) {
        for directory in ["characters", "dialogue", "knowledge", "assets"] {
            std::fs::create_dir_all(root.join(directory)).unwrap();
        }
        std::fs::write(
            root.join("settings.json"),
            default_project_config().to_string(),
        )
        .unwrap();
    }

    #[test]
    fn inspect_merges_defaults_without_overwriting_project_values() {
        let root = temp_root("inspect");
        make_complete_project(&root);
        std::fs::write(
            root.join("settings.json"),
            json!({
                "ai": { "provider": "onnx", "api": { "model": "custom-model" } },
                "paths": {
                    "characters": "characters",
                    "dialogue": "dialogue",
                    "knowledge": "knowledge",
                    "assets": "assets"
                }
            })
            .to_string(),
        )
        .unwrap();

        let state = inspect_project_config(&root).unwrap();
        assert!(state.valid, "{:?}", state.issues);
        assert_eq!(state.paths.len(), PROJECT_PATHS.len());
        assert_eq!(state.config["ai"]["api"]["model"], "custom-model");
        assert_eq!(state.config["render"]["width"], 1280);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inspect_reports_non_regular_and_oversized_settings() {
        let directory_root = temp_root("settings-directory");
        make_complete_project(&directory_root);
        std::fs::remove_file(directory_root.join("settings.json")).unwrap();
        std::fs::create_dir(directory_root.join("settings.json")).unwrap();
        let state = inspect_project_config(&directory_root).unwrap();
        assert!(!state.valid);
        assert!(state
            .issues
            .iter()
            .any(|issue| issue.code == "settings_not_regular_file"));
        std::fs::remove_dir_all(directory_root).unwrap();

        let oversized_root = temp_root("settings-oversized");
        make_complete_project(&oversized_root);
        let file = std::fs::File::create(oversized_root.join("settings.json")).unwrap();
        file.set_len(MAX_PROJECT_SETTINGS_BYTES + 1).unwrap();
        drop(file);
        let state = inspect_project_config(&oversized_root).unwrap();
        assert!(!state.valid);
        assert!(state
            .issues
            .iter()
            .any(|issue| issue.code == "settings_too_large"));
        std::fs::remove_dir_all(oversized_root).unwrap();
    }

    #[test]
    fn inspect_rejects_non_portable_configured_paths() {
        let root = temp_root("invalid-path");
        make_complete_project(&root);
        let mut config = default_project_config();
        config["paths"]["assets"] = Value::String("assets\\private".to_string());
        std::fs::write(root.join("settings.json"), config.to_string()).unwrap();

        let state = inspect_project_config(&root).unwrap();
        assert!(!state.valid);
        assert!(state.issues.iter().any(|issue| {
            issue.code == "project_path_invalid" && issue.path.as_deref() == Some("paths.assets")
        }));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inspect_scrubs_legacy_persisted_secrets_from_returned_state() {
        let root = temp_root("legacy-secret");
        make_complete_project(&root);
        let token = format!("github_pat_{}", "D".repeat(24));
        let mut config = default_project_config();
        config["ai"]["api"]["api_key"] = Value::String("legacy-secret".to_string());
        config["metadata"] = Value::String(format!("Authorization: Bearer {token}"));
        std::fs::write(root.join("settings.json"), config.to_string()).unwrap();

        let state = inspect_project_config(&root).unwrap();
        assert_eq!(state.config["ai"]["api"]["api_key"], "");
        assert_eq!(state.config["metadata"], "Authorization: <redacted>");
        assert!(!state.config.to_string().contains(&token));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn save_is_atomic_and_scrubs_runtime_secrets() {
        let root = temp_root("save");
        make_complete_project(&root);
        let bearer_key = format!("sk-{}", "A".repeat(24));
        let github_key = format!("github_pat_{}", "B".repeat(24));
        let config = json!({
            "ai": {
                "provider": "api",
                "api": {
                    "base_url": format!("https://example.test/v1?access_token={github_key}&model=chat"),
                    "api_key": "runtime-secret",
                    "authorization": format!("Bearer {bearer_key}"),
                    "model": "test-model"
                }
            },
            "paths": {
                "characters": "characters",
                "dialogue": "dialogue",
                "knowledge": "knowledge",
                "assets": "assets"
            }
        });

        let state = save_project_config(&root, config).await.unwrap();
        let persisted: Value =
            serde_json::from_slice(&std::fs::read(root.join("settings.json")).unwrap()).unwrap();
        assert!(state.valid, "{:?}", state.issues);
        assert_eq!(persisted["ai"]["api"]["api_key"], "");
        assert_eq!(persisted["ai"]["api"]["authorization"], "");
        assert_eq!(
            persisted["ai"]["api"]["base_url"],
            "https://example.test/v1?access_token=<redacted>&model=chat"
        );
        let persisted_text = persisted.to_string();
        assert!(!persisted_text.contains(&bearer_key));
        assert!(!persisted_text.contains(&github_key));
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 5);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn config_scrubbers_cover_nested_export_and_free_text_values() {
        let github_key = format!("github_pat_{}", "C".repeat(24));
        let config = json!({
            "sync": {
                "token": "runtime-secret",
                "endpoint": format!("https://example.test?api_key={github_key}"),
                "nested": [{ "password": "nested-secret", "label": "kept" }]
            }
        });

        let scrubbed = scrub_runtime_secret_config(&config);
        assert_eq!(scrubbed["sync"]["token"], "");
        assert_eq!(scrubbed["sync"]["nested"][0]["password"], "");
        assert_eq!(scrubbed["sync"]["nested"][0]["label"], "kept");
        assert_eq!(
            scrubbed["sync"]["endpoint"],
            "https://example.test?api_key=<redacted>"
        );
        assert_eq!(
            sanitize_export_config(&config)["sync"]["token"],
            "<redacted>"
        );
    }
}
