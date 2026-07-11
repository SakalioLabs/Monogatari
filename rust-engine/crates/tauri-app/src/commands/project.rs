//! Project configuration commands for commercial authoring readiness.

use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::Utc;
use llm_authoring::paths::resolve_project_relative;
pub use llm_authoring::project::ProjectConfigState;
pub(crate) use llm_authoring::project::{
    inspect_project_config as build_project_config_state, sanitize_export_config,
    scrub_runtime_secret_config,
};
use llm_authoring::project::{
    path_from_config, save_project_config as save_project_config_to_disk,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::state::{default_project_data_root, AppState};

pub(crate) const EXPORT_DIRECTORIES: &[(&str, &str)] = &[
    ("characters", "characters"),
    ("dialogue", "dialogue"),
    ("knowledge", "knowledge"),
    ("scenes", "scenes"),
    ("assets", "assets"),
    ("events", "events"),
    ("endings", "endings"),
    ("locales", "locales"),
    ("quality_suites", "quality_suites"),
    ("workflows", "workflows"),
];

const MAX_PROJECT_EXPORT_FILES: usize = 20_000;
const MAX_PROJECT_EXPORT_DIRECTORIES: usize = 4_000;
const MAX_PROJECT_EXPORT_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_PROJECT_EXPORT_FILE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_PROJECT_EXPORT_PATH_BYTES: usize = 512;
const MAX_PROJECT_EXPORT_PATH_SEGMENTS: usize = 32;

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

pub(crate) fn build_project_export_manifest(
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
    let directories = collect_project_export_directories(project_root, &config)?;
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
        "project_path": ".",
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
            "directories": directories,
            "files": files,
            "excluded": ["saves", "analytics.json", ".sync_manifest.json"]
        },
        "archive": {
            "format": "zip",
            "manifest_path": "monogatari-project.json",
            "extension": ".monogatari"
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
    let mut total_bytes = 0u64;
    let mut directory_count = 0usize;

    let settings_bytes = project_export_settings_bytes(config)?;
    push_export_bytes(
        "settings",
        "settings.json".to_string(),
        &settings_bytes,
        &mut seen_paths,
        &mut files,
        &mut total_bytes,
    )?;

    for (key, fallback) in EXPORT_DIRECTORIES {
        let dir = configured_project_path(project_root, config, key, fallback)?;
        let metadata = match std::fs::symlink_metadata(&dir) {
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(format!(
                    "Unable to inspect export directory `{}`: {error}",
                    dir.display()
                ));
            }
        };
        if let Some(metadata) = metadata {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(format!(
                    "Project export directories must be regular directories: {}",
                    dir.display()
                ));
            }
            collect_export_files(
                project_root,
                key,
                &dir,
                &mut seen_paths,
                &mut files,
                &mut total_bytes,
                &mut directory_count,
            )?;
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
    total_bytes: &mut u64,
    directory_count: &mut usize,
) -> Result<(), String> {
    *directory_count += 1;
    if *directory_count > MAX_PROJECT_EXPORT_DIRECTORIES {
        return Err(format!(
            "Project export exceeds the {MAX_PROJECT_EXPORT_DIRECTORIES} directory limit."
        ));
    }
    let relative_directory = project_relative_path(project_root, dir);
    validate_project_export_path_shape(&relative_directory, "directory")?;
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "Unable to inspect export source `{}`: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "Project exports cannot include symbolic links: {}",
                path.display()
            ));
        }
        if metadata.is_dir() {
            collect_export_files(
                project_root,
                category,
                &path,
                seen_paths,
                files,
                total_bytes,
                directory_count,
            )?;
        } else if metadata.is_file() {
            push_export_file(
                project_root,
                category,
                &path,
                seen_paths,
                files,
                total_bytes,
            )?;
        }
    }
    Ok(())
}

fn validate_project_export_path_shape(path: &str, label: &str) -> Result<(), String> {
    if path.is_empty()
        || path.len() > MAX_PROJECT_EXPORT_PATH_BYTES
        || path.split('/').count() > MAX_PROJECT_EXPORT_PATH_SEGMENTS
    {
        return Err(format!(
            "Project export {label} `{path}` exceeds portable path depth or length limits."
        ));
    }
    Ok(())
}

fn push_export_file(
    project_root: &Path,
    category: &str,
    path: &Path,
    seen_paths: &mut HashSet<String>,
    files: &mut Vec<Value>,
    total_bytes: &mut u64,
) -> Result<(), String> {
    let canonical_root = project_root.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve project root `{}`: {error}",
            project_root.display()
        )
    })?;
    let canonical_path = path.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve export source `{}`: {error}",
            path.display()
        )
    })?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err(format!(
            "Project export source escapes the project root: {}",
            path.display()
        ));
    }
    let relative = project_relative_path(project_root, path);
    validate_project_export_path_shape(&relative, "file")?;
    if seen_paths.contains(&relative) {
        return Ok(());
    }
    let size_bytes = std::fs::metadata(&canonical_path)
        .map_err(|error| error.to_string())?
        .len();
    ensure_export_inventory_capacity(files.len(), *total_bytes, size_bytes, &relative)?;
    let (read_bytes, checksum_md5, checksum_sha256) = checksum_export_file(&canonical_path)?;
    if read_bytes != size_bytes {
        return Err(format!(
            "Project export source `{relative}` changed size while it was being inventoried."
        ));
    }
    seen_paths.insert(relative.clone());
    *total_bytes += size_bytes;
    files.push(json!({
        "category": category,
        "path": relative,
        "size_bytes": size_bytes,
        "checksum_md5": checksum_md5,
        "checksum_sha256": checksum_sha256,
    }));
    Ok(())
}

fn push_export_bytes(
    category: &str,
    relative: String,
    bytes: &[u8],
    seen_paths: &mut HashSet<String>,
    files: &mut Vec<Value>,
    total_bytes: &mut u64,
) -> Result<(), String> {
    if !seen_paths.insert(relative.clone()) {
        return Ok(());
    }
    let size_bytes = bytes.len() as u64;
    ensure_export_inventory_capacity(files.len(), *total_bytes, size_bytes, &relative)?;
    *total_bytes += size_bytes;
    let checksum_md5 = format!("{:x}", md5::compute(bytes));
    let checksum_sha256 = checksum_sha256(bytes);
    files.push(json!({
        "category": category,
        "path": relative,
        "size_bytes": size_bytes,
        "checksum_md5": checksum_md5,
        "checksum_sha256": checksum_sha256,
    }));
    Ok(())
}

pub(crate) fn project_export_settings_bytes(config: &Value) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(&scrub_runtime_secret_config(config))
        .map_err(|error| error.to_string())
}

fn ensure_export_inventory_capacity(
    current_files: usize,
    current_bytes: u64,
    next_bytes: u64,
    relative_path: &str,
) -> Result<(), String> {
    if current_files >= MAX_PROJECT_EXPORT_FILES {
        return Err(format!(
            "Project export exceeds the {MAX_PROJECT_EXPORT_FILES} file limit."
        ));
    }
    if next_bytes > MAX_PROJECT_EXPORT_FILE_BYTES {
        return Err(format!(
            "Project export source `{relative_path}` exceeds the per-file limit of {MAX_PROJECT_EXPORT_FILE_BYTES} bytes."
        ));
    }
    let next_total = current_bytes
        .checked_add(next_bytes)
        .ok_or_else(|| "Project export size overflowed.".to_string())?;
    if next_total > MAX_PROJECT_EXPORT_TOTAL_BYTES {
        return Err(format!(
            "Project export exceeds the total size limit of {MAX_PROJECT_EXPORT_TOTAL_BYTES} bytes."
        ));
    }
    Ok(())
}

fn checksum_export_file(path: &Path) -> Result<(u64, String, String), String> {
    let mut file = File::open(path)
        .map_err(|error| format!("Unable to open export source `{}`: {error}", path.display()))?;
    let mut md5 = md5::Context::new();
    let mut sha256 = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            format!("Unable to read export source `{}`: {error}", path.display())
        })?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read as u64)
            .ok_or_else(|| "Project export source size overflowed.".to_string())?;
        if total > MAX_PROJECT_EXPORT_FILE_BYTES {
            return Err(format!(
                "Project export source `{}` exceeds the per-file limit of {MAX_PROJECT_EXPORT_FILE_BYTES} bytes.",
                path.display()
            ));
        }
        md5.consume(&buffer[..read]);
        sha256.update(&buffer[..read]);
    }
    Ok((total, format!("{:x}", md5.compute()), finish_sha256(sha256)))
}

pub(crate) fn collect_project_export_directories(
    project_root: &Path,
    config: &Value,
) -> Result<Vec<String>, String> {
    let mut directories = EXPORT_DIRECTORIES
        .iter()
        .map(|(key, fallback)| {
            configured_project_path(project_root, config, key, fallback)
                .map(|path| project_relative_path(project_root, &path))
        })
        .collect::<Result<Vec<_>, _>>()?;
    directories.sort();
    directories.dedup();
    Ok(directories)
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
        update_file_fingerprint(&mut hasher, file);
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
