//! Headless project-package inventory and export-manifest generation.

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::paths::resolve_project_relative;
use crate::project::{
    inspect_project_config, path_from_config, sanitize_export_config, scrub_runtime_secret_config,
};

use super::manifest::is_json_path;
use super::{
    package_fingerprint, portable_case_key, validate_manifest, validate_portable_path,
    ArchiveFileRecord, ARCHIVE_FORMAT, ARCHIVE_SCHEMA, MAX_ARCHIVE_DIRECTORIES, MAX_ARCHIVE_FILES,
    MAX_ARCHIVE_FILE_BYTES, MAX_ARCHIVE_JSON_BYTES, MAX_ARCHIVE_TOTAL_BYTES,
    PACKAGE_FINGERPRINT_ALGORITHM,
};

pub const PROJECT_EXPORT_DIRECTORIES: &[(&str, &str)] = &[
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProjectExportRuntimeSnapshot {
    #[serde(default)]
    pub loaded_characters: Vec<String>,
    #[serde(default)]
    pub loaded_dialogues: Vec<String>,
    #[serde(default)]
    pub loaded_knowledge_count: usize,
    #[serde(default)]
    pub current_scene: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProjectExportProvenance {
    pub exported_at: String,
    pub engine_version: String,
    pub git_commit: String,
    pub git_short_commit: String,
}

/// Build a complete package manifest without depending on Tauri or live application state.
pub fn build_project_export_manifest(
    project_root: &Path,
    runtime: ProjectExportRuntimeSnapshot,
    provenance: ProjectExportProvenance,
) -> Result<Value, String> {
    validate_export_provenance(&provenance)?;
    let config_state = inspect_project_config(project_root)?;
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
    let total_bytes = files.iter().map(|file| file.size_bytes).sum::<u64>();
    let content_sha256 = package_fingerprint(files.iter());
    let content_summary = project_content_summary(&files);

    let manifest = json!({
        "format": ARCHIVE_FORMAT,
        "schema": ARCHIVE_SCHEMA,
        "version": "1.0",
        "exported_at": provenance.exported_at,
        "export_metadata": {
            "engine_version": provenance.engine_version,
            "git_commit": provenance.git_commit,
            "git_short_commit": provenance.git_short_commit,
        },
        "project_path": ".",
        "settings": sanitize_export_config(&config),
        "content": {
            "characters": if file_characters.is_empty() {
                runtime.loaded_characters
            } else {
                file_characters
            },
            "dialogues": if file_dialogues.is_empty() {
                runtime.loaded_dialogues
            } else {
                file_dialogues
            },
            "knowledge": file_knowledge,
            "knowledge_count": runtime
                .loaded_knowledge_count
                .max(count_json_files(&knowledge_path)),
            "scenes": file_scenes,
            "current_scene": runtime.current_scene,
        },
        "content_summary": content_summary,
        "package": {
            "file_count": files.len(),
            "total_bytes": total_bytes,
            "fingerprint_algorithm": PACKAGE_FINGERPRINT_ALGORITHM,
            "content_sha256": content_sha256,
            "directories": directories,
            "files": files,
            "excluded": ["saves", "analytics.json", ".sync_manifest.json"]
        },
        "archive": {
            "format": "zip",
            "manifest_path": super::ARCHIVE_MANIFEST_PATH,
            "extension": ".monogatari"
        }
    });

    validate_manifest(manifest.clone())?;
    Ok(manifest)
}

/// Encode the credential-free settings bytes inventoried and written into the archive.
pub fn project_export_settings_bytes(config: &Value) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(&scrub_runtime_secret_config(config))
        .map_err(|error| error.to_string())
}

fn validate_export_provenance(provenance: &ProjectExportProvenance) -> Result<(), String> {
    for (label, value, limit) in [
        ("exported_at", provenance.exported_at.as_str(), 80usize),
        (
            "engine_version",
            provenance.engine_version.as_str(),
            80usize,
        ),
        ("git_commit", provenance.git_commit.as_str(), 160usize),
        (
            "git_short_commit",
            provenance.git_short_commit.as_str(),
            80usize,
        ),
    ] {
        if value.trim().is_empty() || value.len() > limit || value.chars().any(char::is_control) {
            return Err(format!(
                "Project export provenance `{label}` must be non-empty, bounded plain text."
            ));
        }
    }
    Ok(())
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
) -> Result<Vec<ArchiveFileRecord>, String> {
    let mut files = Vec::new();
    let mut seen_paths = HashMap::new();
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

    for (key, fallback) in PROJECT_EXPORT_DIRECTORIES {
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

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

#[allow(clippy::too_many_arguments)]
fn collect_export_files(
    project_root: &Path,
    category: &str,
    dir: &Path,
    seen_paths: &mut HashMap<String, String>,
    files: &mut Vec<ArchiveFileRecord>,
    total_bytes: &mut u64,
    directory_count: &mut usize,
) -> Result<(), String> {
    *directory_count += 1;
    if *directory_count > MAX_ARCHIVE_DIRECTORIES {
        return Err(format!(
            "Project export exceeds the {MAX_ARCHIVE_DIRECTORIES} directory limit."
        ));
    }
    let relative_directory = project_relative_path(project_root, dir);
    validate_portable_path(&relative_directory, "Project export directory")?;
    for entry in std::fs::read_dir(dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
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

fn push_export_file(
    project_root: &Path,
    category: &str,
    path: &Path,
    seen_paths: &mut HashMap<String, String>,
    files: &mut Vec<ArchiveFileRecord>,
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
    validate_portable_path(&relative, "Project export file")?;
    if !register_export_path(seen_paths, &relative)? {
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
    *total_bytes += size_bytes;
    files.push(ArchiveFileRecord {
        category: category.to_string(),
        path: relative,
        size_bytes,
        checksum_md5,
        checksum_sha256,
    });
    Ok(())
}

fn push_export_bytes(
    category: &str,
    relative: String,
    bytes: &[u8],
    seen_paths: &mut HashMap<String, String>,
    files: &mut Vec<ArchiveFileRecord>,
    total_bytes: &mut u64,
) -> Result<(), String> {
    validate_portable_path(&relative, "Project export file")?;
    if !register_export_path(seen_paths, &relative)? {
        return Ok(());
    }
    let size_bytes = bytes.len() as u64;
    ensure_export_inventory_capacity(files.len(), *total_bytes, size_bytes, &relative)?;
    *total_bytes += size_bytes;
    files.push(ArchiveFileRecord {
        category: category.to_string(),
        path: relative,
        size_bytes,
        checksum_md5: format!("{:x}", md5::compute(bytes)),
        checksum_sha256: checksum_sha256(bytes),
    });
    Ok(())
}

fn register_export_path(
    seen_paths: &mut HashMap<String, String>,
    relative: &str,
) -> Result<bool, String> {
    let portable_key = portable_case_key(relative);
    if let Some(existing) = seen_paths.get(&portable_key) {
        if existing == relative {
            return Ok(false);
        }
        return Err(format!(
            "Project export paths `{existing}` and `{relative}` collide on portable filesystems."
        ));
    }
    seen_paths.insert(portable_key, relative.to_string());
    Ok(true)
}

fn ensure_export_inventory_capacity(
    current_files: usize,
    current_bytes: u64,
    next_bytes: u64,
    relative_path: &str,
) -> Result<(), String> {
    if current_files >= MAX_ARCHIVE_FILES {
        return Err(format!(
            "Project export exceeds the {MAX_ARCHIVE_FILES} file limit."
        ));
    }
    if next_bytes > MAX_ARCHIVE_FILE_BYTES {
        return Err(format!(
            "Project export source `{relative_path}` exceeds the per-file limit of {MAX_ARCHIVE_FILE_BYTES} bytes."
        ));
    }
    if is_json_path(relative_path) && next_bytes > MAX_ARCHIVE_JSON_BYTES {
        return Err(format!(
            "Project export JSON `{relative_path}` exceeds the validation limit of {MAX_ARCHIVE_JSON_BYTES} bytes."
        ));
    }
    let next_total = current_bytes
        .checked_add(next_bytes)
        .ok_or_else(|| "Project export size overflowed.".to_string())?;
    if next_total > MAX_ARCHIVE_TOTAL_BYTES {
        return Err(format!(
            "Project export exceeds the total size limit of {MAX_ARCHIVE_TOTAL_BYTES} bytes."
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
        if total > MAX_ARCHIVE_FILE_BYTES {
            return Err(format!(
                "Project export source `{}` exceeds the per-file limit of {MAX_ARCHIVE_FILE_BYTES} bytes.",
                path.display()
            ));
        }
        md5.consume(&buffer[..read]);
        sha256.update(&buffer[..read]);
    }
    Ok((total, format!("{:x}", md5.compute()), finish_sha256(sha256)))
}

fn collect_project_export_directories(
    project_root: &Path,
    config: &Value,
) -> Result<Vec<String>, String> {
    let mut paths_by_key = BTreeMap::<String, String>::new();
    for (key, fallback) in PROJECT_EXPORT_DIRECTORIES {
        let path = configured_project_path(project_root, config, key, fallback)?;
        let relative = project_relative_path(project_root, &path);
        validate_portable_path(&relative, "Project export directory")?;
        let portable_key = portable_case_key(&relative);
        if let Some(existing) = paths_by_key.get(&portable_key) {
            if existing != &relative {
                return Err(format!(
                    "Project export directories `{existing}` and `{relative}` collide on portable filesystems."
                ));
            }
        } else {
            paths_by_key.insert(portable_key, relative);
        }
    }
    let mut directories = paths_by_key.into_values().collect::<Vec<_>>();
    directories.sort();
    Ok(directories)
}

fn project_content_summary(files: &[ArchiveFileRecord]) -> Value {
    let mut category_counts = BTreeMap::<String, usize>::new();
    let mut category_bytes = BTreeMap::<String, u64>::new();
    let mut category_files = BTreeMap::<String, Vec<&ArchiveFileRecord>>::new();
    let mut json_file_count = 0usize;
    let mut asset_file_count = 0usize;

    for file in files {
        *category_counts.entry(file.category.clone()).or_insert(0) += 1;
        *category_bytes.entry(file.category.clone()).or_insert(0) += file.size_bytes;
        category_files
            .entry(file.category.clone())
            .or_default()
            .push(file);
        if file.path.ends_with(".json") {
            json_file_count += 1;
        }
        if file.category == "assets" {
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
        "category_fingerprint_algorithm": PACKAGE_FINGERPRINT_ALGORITHM,
        "category_fingerprints": category_files
            .iter()
            .map(|(category, files)| {
                (category.clone(), package_fingerprint(files.iter().copied()))
            })
            .collect::<BTreeMap<_, _>>(),
        "exported_categories": PROJECT_EXPORT_DIRECTORIES
            .iter()
            .map(|(category, _)| *category)
            .collect::<Vec<_>>(),
    })
}

fn checksum_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn finish_sha256(hasher: Sha256) -> String {
    format!("{:x}", hasher.finalize())
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
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
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
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|extension| extension.to_str())
                        == Some("json")
                })
                .count()
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_project_export_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn test_provenance() -> ProjectExportProvenance {
        ProjectExportProvenance {
            exported_at: "2026-07-15T00:00:00Z".to_string(),
            engine_version: "test-engine".to_string(),
            git_commit: "0123456789abcdef".to_string(),
            git_short_commit: "0123456".to_string(),
        }
    }

    #[test]
    fn export_manifest_inventories_files_and_redacts_secrets_headlessly() {
        let root = test_root("manifest");
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
                    "api": { "api_key": "secret-key", "model": "test-model" }
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
        std::fs::write(root.join("characters/sakura.json"), r#"{"id":"sakura"}"#).unwrap();
        std::fs::write(root.join("dialogue/intro.json"), r#"{"id":"intro"}"#).unwrap();
        std::fs::write(root.join("knowledge/world.json"), r#"{"id":"world"}"#).unwrap();
        std::fs::write(root.join("assets/tts/line.wav"), b"voice").unwrap();
        std::fs::write(root.join("saves/slot.json"), b"private save").unwrap();
        std::fs::write(root.join("analytics.json"), b"[]").unwrap();

        let manifest = build_project_export_manifest(
            &root,
            ProjectExportRuntimeSnapshot::default(),
            test_provenance(),
        )
        .unwrap();
        assert_eq!(manifest["schema"], ARCHIVE_SCHEMA);
        assert_eq!(manifest["export_metadata"]["engine_version"], "test-engine");
        assert_eq!(manifest["exported_at"], "2026-07-15T00:00:00Z");
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
            PACKAGE_FINGERPRINT_ALGORITHM
        );
        assert_eq!(
            manifest["content_summary"]["category_fingerprints"]["assets"]
                .as_str()
                .unwrap()
                .len(),
            64
        );
        assert!(manifest["content_summary"]["exported_categories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|category| category.as_str() == Some("workflows")));
        assert_eq!(
            manifest["package"]["fingerprint_algorithm"],
            PACKAGE_FINGERPRINT_ALGORITHM
        );
        assert_eq!(
            manifest["package"]["content_sha256"]
                .as_str()
                .unwrap()
                .len(),
            64
        );
        let files = manifest["package"]["files"].as_array().unwrap();
        let tts_file = files
            .iter()
            .find(|file| file["path"] == "assets/tts/line.wav")
            .unwrap();
        assert_eq!(
            tts_file["checksum_sha256"],
            "c57d7e92019708b614c90fa3685cd644f543a60153fb99ec9b67c381a245fb2a"
        );
        assert!(tts_file["checksum_md5"].as_str().is_some());
        assert!(files.iter().any(|file| file["path"] == "settings.json"));
        assert!(!files.iter().any(|file| file["path"] == "saves/slot.json"));
        assert!(!files.iter().any(|file| file["path"] == "analytics.json"));

        let repeat_manifest = build_project_export_manifest(
            &root,
            ProjectExportRuntimeSnapshot::default(),
            test_provenance(),
        )
        .unwrap();
        assert_eq!(
            repeat_manifest["package"]["content_sha256"],
            manifest["package"]["content_sha256"]
        );
        assert_eq!(
            repeat_manifest["content_summary"]["category_fingerprints"],
            manifest["content_summary"]["category_fingerprints"]
        );

        std::fs::write(root.join("assets/tts/line.wav"), b"new voice").unwrap();
        let changed_manifest = build_project_export_manifest(
            &root,
            ProjectExportRuntimeSnapshot::default(),
            test_provenance(),
        )
        .unwrap();
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

    #[test]
    fn export_manifest_uses_runtime_fallbacks_without_desktop_state() {
        let root = test_root("fallbacks");
        for dir in ["characters", "dialogue", "knowledge", "assets"] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        std::fs::write(root.join("settings.json"), "{}").unwrap();
        let manifest = build_project_export_manifest(
            &root,
            ProjectExportRuntimeSnapshot {
                loaded_characters: vec!["runtime_character".to_string()],
                loaded_dialogues: vec!["runtime_dialogue".to_string()],
                loaded_knowledge_count: 3,
                current_scene: Some("runtime_scene".to_string()),
            },
            test_provenance(),
        )
        .unwrap();
        assert_eq!(
            manifest["content"]["characters"],
            json!(["runtime_character"])
        );
        assert_eq!(
            manifest["content"]["dialogues"],
            json!(["runtime_dialogue"])
        );
        assert_eq!(manifest["content"]["knowledge_count"], 3);
        assert_eq!(manifest["content"]["current_scene"], "runtime_scene");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn export_inventory_rejects_portable_aliases_and_unsafe_paths_without_io() {
        let mut seen = HashMap::new();
        let mut files = Vec::new();
        let mut total = 0;
        push_export_bytes(
            "assets",
            "assets/Guide.json".to_string(),
            b"{}",
            &mut seen,
            &mut files,
            &mut total,
        )
        .unwrap();
        let error = push_export_bytes(
            "assets",
            "assets/guide.json".to_string(),
            b"{}",
            &mut seen,
            &mut files,
            &mut total,
        )
        .unwrap_err();
        assert!(error.contains("collide on portable filesystems"), "{error}");

        let error = push_export_bytes(
            "assets",
            "assets/CON.json".to_string(),
            b"{}",
            &mut seen,
            &mut files,
            &mut total,
        )
        .unwrap_err();
        assert!(error.contains("unsafe path segment"), "{error}");
    }

    #[test]
    fn export_inventory_limits_fail_before_file_reads() {
        let error =
            ensure_export_inventory_capacity(MAX_ARCHIVE_FILES, 0, 0, "assets/a").unwrap_err();
        assert!(error.contains("file limit"), "{error}");
        let error = ensure_export_inventory_capacity(0, 0, MAX_ARCHIVE_FILE_BYTES + 1, "assets/a")
            .unwrap_err();
        assert!(error.contains("per-file limit"), "{error}");
        let error =
            ensure_export_inventory_capacity(0, 0, MAX_ARCHIVE_JSON_BYTES + 1, "assets/a.JSON")
                .unwrap_err();
        assert!(error.contains("JSON"), "{error}");
        let error = ensure_export_inventory_capacity(0, MAX_ARCHIVE_TOTAL_BYTES, 1, "assets/a")
            .unwrap_err();
        assert!(error.contains("total size limit"), "{error}");
    }

    #[test]
    fn export_manifest_rejects_unbounded_or_empty_provenance() {
        let mut provenance = test_provenance();
        provenance.engine_version.clear();
        assert!(validate_export_provenance(&provenance)
            .unwrap_err()
            .contains("engine_version"));
        provenance = test_provenance();
        provenance.git_commit = "x".repeat(161);
        assert!(validate_export_provenance(&provenance)
            .unwrap_err()
            .contains("git_commit"));
    }
}
