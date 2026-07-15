//! Headless verification for an extracted or installed desktop bundle.

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;
use llm_authoring::project::scrub_runtime_secret_config;
use serde::Serialize;
use serde_json::Value;

use crate::commands::{endings, engine, i18n, project, quality_suite, scenes, workflow};
use crate::state::discover_bundled_project_data_root;

const VERIFY_FLAG: &str = "--verify-installation";
const REPORT_SCHEMA: &str = "monogatari-installation-verification/v1";
const MAX_INSTALLED_FILES: usize = 20_000;
const MAX_INSTALLED_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_INSTALLED_JSON_FILES: usize = 4_000;
const MAX_INSTALLED_JSON_BYTES: u64 = 64 * 1024 * 1024;
const MAX_REPORT_BYTES: usize = 1024 * 1024;
const MAX_TREE_DEPTH: usize = 32;
const ALLOWED_PROJECT_WARNING_CODES: &[&str] = &["api_key_missing"];

static REPORT_STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize)]
pub struct InstallationContentCounts {
    pub characters: usize,
    pub dialogues: usize,
    pub knowledge: usize,
    pub story_events: usize,
    pub scenes: usize,
    pub backgrounds: usize,
    pub endings: usize,
    pub workflows: usize,
    pub quality_suites: usize,
    pub locales: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallationVerificationReport {
    pub schema: String,
    pub status: String,
    pub verified_at: String,
    pub engine_version: String,
    pub git_commit: String,
    pub git_short_commit: String,
    pub executable_path: String,
    pub resource_root: String,
    pub project_config_valid: bool,
    pub project_warning_count: usize,
    pub project_warning_codes: Vec<String>,
    pub data_file_count: usize,
    pub json_file_count: usize,
    pub data_total_bytes: u64,
    pub project_content_sha256: String,
    pub counts: InstallationContentCounts,
}

#[derive(Debug, Serialize)]
struct InstallationVerificationEnvelope {
    schema: &'static str,
    status: &'static str,
    generated_at: String,
    report: Option<InstallationVerificationReport>,
    error: Option<String>,
}

#[derive(Debug)]
struct TreeInventory {
    file_count: usize,
    json_file_count: usize,
    total_bytes: u64,
}

/// Handle the headless verification flag before Tauri opens a window.
pub fn run_requested_verification() -> Option<i32> {
    let report_path = match parse_report_path(std::env::args_os().skip(1)) {
        Ok(Some(path)) => path,
        Ok(None) => return None,
        Err(_) => return Some(2),
    };
    let executable_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            let envelope = failed_envelope(format!("Unable to resolve executable path: {error}"));
            return Some(write_envelope(&report_path, &envelope).map_or(3, |_| 2));
        }
    };
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            let envelope =
                failed_envelope(format!("Unable to create verification runtime: {error}"));
            return Some(write_envelope(&report_path, &envelope).map_or(3, |_| 2));
        }
    };
    let verification = runtime.block_on(verify_installed_application(&executable_path));
    drop(runtime);

    let (envelope, exit_code) = match verification {
        Ok(report) => (
            InstallationVerificationEnvelope {
                schema: REPORT_SCHEMA,
                status: "verified",
                generated_at: Utc::now().to_rfc3339(),
                report: Some(report),
                error: None,
            },
            0,
        ),
        Err(error) => (failed_envelope(error), 2),
    };
    Some(write_envelope(&report_path, &envelope).map_or(3, |_| exit_code))
}

pub(crate) async fn verify_installed_application(
    executable_path: &Path,
) -> Result<InstallationVerificationReport, String> {
    let executable_path = canonical_regular_file(executable_path, "Application executable")?;
    let resource_dir = executable_path
        .parent()
        .ok_or_else(|| "Application executable has no resource directory.".to_string())?;
    let project_root = discover_bundled_project_data_root(resource_dir).ok_or_else(|| {
        format!(
            "Bundled project data was not found beside `{}`.",
            executable_path.display()
        )
    })?;
    let project_root = canonical_regular_directory(&project_root, "Bundled project data")?;

    verify_required_layout(&project_root)?;
    let settings = read_bounded_json(&project_root.join("settings.json"))?;
    if scrub_runtime_secret_config(&settings) != settings {
        return Err("Bundled settings.json contains runtime secrets.".to_string());
    }

    let project_state = project::build_project_config_state(&project_root)?;
    if !project_state.valid {
        let errors = project_state
            .issues
            .iter()
            .filter(|issue| issue.severity == "error")
            .map(|issue| issue.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!(
            "Bundled project configuration is invalid: {errors}"
        ));
    }
    let mut project_warning_codes = project_state
        .issues
        .iter()
        .filter(|issue| issue.severity == "warning")
        .map(|issue| issue.code.clone())
        .collect::<Vec<_>>();
    project_warning_codes.sort();
    let unexpected_warning_codes = project_warning_codes
        .iter()
        .filter(|code| !ALLOWED_PROJECT_WARNING_CODES.contains(&code.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unexpected_warning_codes.is_empty() {
        return Err(format!(
            "Bundled project has unexpected configuration warnings: {}.",
            unexpected_warning_codes.join(", ")
        ));
    }

    let (characters, dialogues, knowledge, events) =
        engine::load_project_content(&project_root).await?;
    let scene_catalog = scenes::build_scene_asset_catalog(&project_root)?;
    if !scene_catalog.valid {
        let errors = scene_catalog
            .issues
            .iter()
            .filter(|issue| issue.severity == "error")
            .map(|issue| issue.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("Bundled scene content is invalid: {errors}"));
    }
    let ending_definitions = endings::load_story_endings(&project_root)?;
    endings::validate_story_ending_references(&project_root, &ending_definitions).await?;

    let workflow_paths = collect_json_paths(&project_root, "workflows")?;
    for path in &workflow_paths {
        workflow::load_workflow_from_project(&project_root, path).await?;
    }
    let quality_suite_paths = collect_json_paths(&project_root, "quality_suites")?;
    for path in &quality_suite_paths {
        let document = read_bounded_json(&project_root.join(path))?;
        let content = serde_json::to_string(&document).map_err(|error| error.to_string())?;
        quality_suite::parse_quality_suite(&content)
            .map_err(|error| format!("Bundled quality suite `{path}` is invalid: {error}"))?;
    }
    let locale_paths = collect_json_paths(&project_root, "locales")?;
    for path in &locale_paths {
        let relative = Path::new(path)
            .strip_prefix("locales")
            .map_err(|_| format!("Bundled locale path is invalid: {path}"))?;
        if relative.components().count() != 1 {
            return Err(format!("Bundled locale must be a direct file: {path}"));
        }
        let locale_id = relative
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| format!("Bundled locale id is invalid: {path}"))?;
        i18n::load_locale_from_project(&project_root, locale_id)
            .map_err(|error| format!("Bundled locale `{path}` is invalid: {error}"))?;
    }
    let tree_inventory = inventory_tree(&project_root)?;

    let character_ids = characters.character_ids();
    let dialogue_ids = dialogues.script_ids();
    let event_snapshot = events.snapshot();
    let export_manifest = project::build_project_export_manifest(
        &project_root,
        character_ids.clone(),
        dialogue_ids.clone(),
        knowledge.len(),
        None,
    )?;
    let package = export_manifest
        .get("package")
        .and_then(Value::as_object)
        .ok_or_else(|| "Bundled project export manifest has no package inventory.".to_string())?;
    let package_file_count = package
        .get("file_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| "Bundled project package file count is missing.".to_string())?
        as usize;
    if package_file_count != tree_inventory.file_count {
        return Err(format!(
            "Bundled project inventory contains {package_file_count} files, but the resource tree contains {}.",
            tree_inventory.file_count
        ));
    }
    let project_content_sha256 = package
        .get("content_sha256")
        .and_then(Value::as_str)
        .filter(|value| value.len() == 64)
        .ok_or_else(|| "Bundled project content fingerprint is invalid.".to_string())?
        .to_string();

    Ok(InstallationVerificationReport {
        schema: REPORT_SCHEMA.to_string(),
        status: "verified".to_string(),
        verified_at: Utc::now().to_rfc3339(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: env!("MONOGATARI_GIT_COMMIT").to_string(),
        git_short_commit: env!("MONOGATARI_GIT_SHORT_COMMIT").to_string(),
        executable_path: executable_path.to_string_lossy().to_string(),
        resource_root: project_root.to_string_lossy().to_string(),
        project_config_valid: true,
        project_warning_count: project_state.warning_count,
        project_warning_codes,
        data_file_count: tree_inventory.file_count,
        json_file_count: tree_inventory.json_file_count,
        data_total_bytes: tree_inventory.total_bytes,
        project_content_sha256,
        counts: InstallationContentCounts {
            characters: character_ids.len(),
            dialogues: dialogue_ids.len(),
            knowledge: knowledge.len(),
            story_events: event_snapshot.event_count,
            scenes: scene_catalog.scenes.len(),
            backgrounds: scene_catalog.backgrounds.len(),
            endings: ending_definitions.len(),
            workflows: workflow_paths.len(),
            quality_suites: quality_suite_paths.len(),
            locales: locale_paths.len(),
        },
    })
}

fn parse_report_path(args: impl IntoIterator<Item = OsString>) -> Result<Option<PathBuf>, String> {
    let args = args.into_iter().collect::<Vec<_>>();
    let mut requested = None;
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if arg == OsStr::new(VERIFY_FLAG) {
            let path = args
                .get(index + 1)
                .ok_or_else(|| format!("{VERIFY_FLAG} requires a report path."))?;
            if requested.replace(PathBuf::from(path)).is_some() {
                return Err(format!("{VERIFY_FLAG} may only be provided once."));
            }
            index += 2;
            continue;
        }
        if let Some(value) = arg.to_str().and_then(|value| {
            value
                .strip_prefix(VERIFY_FLAG)
                .and_then(|rest| rest.strip_prefix('='))
        }) {
            if value.is_empty() || requested.replace(PathBuf::from(value)).is_some() {
                return Err(format!("{VERIFY_FLAG} requires one report path."));
            }
        }
        index += 1;
    }
    requested.map(validate_report_path).transpose()
}

fn validate_report_path(path: PathBuf) -> Result<PathBuf, String> {
    if !path.is_absolute()
        || path.to_string_lossy().chars().any(char::is_control)
        || path.extension().and_then(OsStr::to_str) != Some("json")
    {
        return Err(
            "Installation verification report path must be an absolute .json path.".to_string(),
        );
    }
    let parent = path
        .parent()
        .ok_or_else(|| "Installation verification report path has no parent.".to_string())?;
    let parent = canonical_regular_directory(parent, "Verification report directory")?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "Installation verification report path has no file name.".to_string())?;
    let path = parent.join(file_name);
    if path.exists() {
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("Existing verification report must be a regular file.".to_string());
        }
    }
    Ok(path)
}

fn failed_envelope(error: String) -> InstallationVerificationEnvelope {
    InstallationVerificationEnvelope {
        schema: REPORT_SCHEMA,
        status: "failed",
        generated_at: Utc::now().to_rfc3339(),
        report: None,
        error: Some(error),
    }
}

fn write_envelope(
    report_path: &Path,
    envelope: &InstallationVerificationEnvelope,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(envelope).map_err(|error| error.to_string())?;
    if bytes.len() > MAX_REPORT_BYTES {
        return Err("Installation verification report exceeds the 1 MiB limit.".to_string());
    }
    let parent = report_path
        .parent()
        .ok_or_else(|| "Verification report has no parent directory.".to_string())?;
    let stage_path = unique_report_sibling(parent, "tmp")?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&stage_path)
        .map_err(|error| format!("Unable to stage verification report: {error}"))?;
    if let Err(error) = file
        .write_all(&bytes)
        .and_then(|_| file.flush())
        .and_then(|_| file.sync_all())
    {
        drop(file);
        let _ = std::fs::remove_file(&stage_path);
        return Err(format!("Unable to write verification report: {error}"));
    }
    drop(file);

    let backup_path = unique_report_sibling(parent, "backup")?;
    let had_report = report_path.exists();
    if had_report {
        std::fs::rename(report_path, &backup_path)
            .map_err(|error| format!("Unable to back up verification report: {error}"))?;
    }
    if let Err(error) = std::fs::rename(&stage_path, report_path) {
        if had_report {
            let _ = std::fs::rename(&backup_path, report_path);
        }
        let _ = std::fs::remove_file(&stage_path);
        return Err(format!("Unable to commit verification report: {error}"));
    }
    if had_report {
        std::fs::remove_file(&backup_path)
            .map_err(|error| format!("Unable to remove verification report backup: {error}"))?;
    }
    Ok(())
}

fn unique_report_sibling(parent: &Path, suffix: &str) -> Result<PathBuf, String> {
    for _ in 0..1000 {
        let counter = REPORT_STAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".monogatari-installation-{}-{counter}.{suffix}",
            std::process::id()
        ));
        if !path.exists() {
            return Ok(path);
        }
    }
    Err("Unable to allocate a verification report staging path.".to_string())
}

fn verify_required_layout(project_root: &Path) -> Result<(), String> {
    for directory in [
        "assets",
        "characters",
        "dialogue",
        "endings",
        "events",
        "knowledge",
        "locales",
        "quality_suites",
        "scenes",
        "workflows",
    ] {
        canonical_regular_directory(
            &project_root.join(directory),
            &format!("Bundled `{directory}` directory"),
        )?;
    }
    for file in [
        "settings.json",
        "events/story_events.json",
        "quality_suites/character_stability.json",
        "workflows/score_gate_demo.json",
    ] {
        canonical_regular_file(&project_root.join(file), &format!("Bundled `{file}` file"))?;
    }
    Ok(())
}

fn collect_json_paths(project_root: &Path, directory: &str) -> Result<Vec<String>, String> {
    let root = canonical_regular_directory(
        &project_root.join(directory),
        &format!("Bundled `{directory}` directory"),
    )?;
    let mut paths = Vec::new();
    collect_json_paths_inner(project_root, &root, 0, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_json_paths_inner(
    project_root: &Path,
    directory: &Path,
    depth: usize,
    paths: &mut Vec<String>,
) -> Result<(), String> {
    if depth > MAX_TREE_DEPTH {
        return Err("Bundled JSON directory tree is too deep.".to_string());
    }
    for entry in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "Bundled project cannot contain symlinks: {}",
                path.display()
            ));
        }
        if metadata.is_dir() {
            collect_json_paths_inner(project_root, &path, depth + 1, paths)?;
        } else if metadata.is_file() && path.extension().and_then(OsStr::to_str) == Some("json") {
            read_bounded_json(&path)?;
            paths.push(project_relative_path(project_root, &path)?);
            if paths.len() > MAX_INSTALLED_JSON_FILES {
                return Err("Bundled project contains too many JSON files.".to_string());
            }
        }
    }
    Ok(())
}

fn inventory_tree(project_root: &Path) -> Result<TreeInventory, String> {
    let mut inventory = TreeInventory {
        file_count: 0,
        json_file_count: 0,
        total_bytes: 0,
    };
    let mut portable_paths = HashSet::new();
    inventory_tree_inner(
        project_root,
        project_root,
        0,
        &mut portable_paths,
        &mut inventory,
    )?;
    Ok(inventory)
}

fn inventory_tree_inner(
    project_root: &Path,
    directory: &Path,
    depth: usize,
    portable_paths: &mut HashSet<String>,
    inventory: &mut TreeInventory,
) -> Result<(), String> {
    if depth > MAX_TREE_DEPTH {
        return Err("Bundled project directory tree is too deep.".to_string());
    }
    for entry in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "Bundled project cannot contain symlinks: {}",
                path.display()
            ));
        }
        if metadata.is_dir() {
            inventory_tree_inner(project_root, &path, depth + 1, portable_paths, inventory)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(format!(
                "Bundled project entry is not a regular file: {}",
                path.display()
            ));
        }
        let relative = project_relative_path(project_root, &path)?;
        let portable_key = relative.to_lowercase();
        if !portable_paths.insert(portable_key) {
            return Err(format!(
                "Bundled project has a case-colliding path: {relative}"
            ));
        }
        inventory.file_count += 1;
        if inventory.file_count > MAX_INSTALLED_FILES {
            return Err("Bundled project contains too many files.".to_string());
        }
        inventory.total_bytes = inventory
            .total_bytes
            .checked_add(metadata.len())
            .ok_or_else(|| "Bundled project size overflowed.".to_string())?;
        if inventory.total_bytes > MAX_INSTALLED_TOTAL_BYTES {
            return Err("Bundled project exceeds the installed size limit.".to_string());
        }
        if path.extension().and_then(OsStr::to_str) == Some("json") {
            read_bounded_json(&path)?;
            inventory.json_file_count += 1;
            if inventory.json_file_count > MAX_INSTALLED_JSON_FILES {
                return Err("Bundled project contains too many JSON files.".to_string());
            }
        }
    }
    Ok(())
}

fn read_bounded_json(path: &Path) -> Result<Value, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "Bundled JSON must be a regular file: {}",
            path.display()
        ));
    }
    if metadata.len() > MAX_INSTALLED_JSON_BYTES {
        return Err(format!(
            "Bundled JSON `{}` exceeds the {} byte limit.",
            path.display(),
            MAX_INSTALLED_JSON_BYTES
        ));
    }
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_INSTALLED_JSON_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_INSTALLED_JSON_BYTES {
        return Err(format!(
            "Bundled JSON grew while reading: {}",
            path.display()
        ));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("Bundled JSON `{}` is invalid: {error}", path.display()))
}

fn project_relative_path(project_root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(project_root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .map_err(|_| format!("Bundled project path escaped its root: {}", path.display()))
}

fn canonical_regular_file(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("Unable to inspect {label} `{}`: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{label} must be a regular file: {}",
            path.display()
        ));
    }
    path.canonicalize()
        .map_err(|error| format!("Unable to resolve {label} `{}`: {error}", path.display()))
}

fn canonical_regular_directory(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("Unable to inspect {label} `{}`: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "{label} must be a regular directory: {}",
            path.display()
        ));
    }
    path.canonicalize()
        .map_err(|error| format!("Unable to resolve {label} `{}`: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_installation_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn copy_directory(source: &Path, destination: &Path) {
        std::fs::create_dir_all(destination).unwrap();
        for entry in std::fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());
            if source_path.is_dir() {
                copy_directory(&source_path, &destination_path);
            } else {
                std::fs::copy(source_path, destination_path).unwrap();
            }
        }
    }

    #[test]
    fn verification_report_flags_require_absolute_json_paths() {
        let root = temp_root("args");
        std::fs::create_dir_all(&root).unwrap();
        let report = root.join("report.json");

        let parsed =
            parse_report_path([OsString::from(VERIFY_FLAG), report.clone().into_os_string()])
                .unwrap()
                .unwrap();
        assert_eq!(parsed.file_name(), report.file_name());
        assert_eq!(parsed.parent().unwrap(), root.canonicalize().unwrap());
        assert!(parse_report_path([OsString::from(VERIFY_FLAG)]).is_err());
        assert!(
            parse_report_path([OsString::from(VERIFY_FLAG), OsString::from("relative.json")])
                .is_err()
        );
        assert!(parse_report_path([OsString::from(format!(
            "{VERIFY_FLAG}={}",
            root.join("report.txt").display()
        ))])
        .is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn checked_in_data_passes_installed_runtime_verification() {
        let root = temp_root("runtime");
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let expected_data_file_count = inventory_tree(&source).unwrap().file_count;
        copy_directory(&source, &root.join("data"));
        std::fs::write(root.join("llm-galgame-app.exe"), b"test executable").unwrap();

        let report = verify_installed_application(&root.join("llm-galgame-app.exe"))
            .await
            .unwrap();

        assert_eq!(report.schema, REPORT_SCHEMA);
        assert_eq!(report.status, "verified");
        assert_eq!(report.engine_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(report.git_commit, env!("MONOGATARI_GIT_COMMIT"));
        assert_eq!(report.git_short_commit, env!("MONOGATARI_GIT_SHORT_COMMIT"));
        assert_eq!(report.data_file_count, expected_data_file_count);
        assert!(report.project_warning_codes.is_empty());
        assert!(report.counts.characters > 0);
        assert!(report.counts.dialogues > 0);
        assert!(report.counts.story_events > 0);
        assert!(report.counts.workflows > 0);
        assert_eq!(report.project_content_sha256.len(), 64);
        std::fs::remove_dir_all(root).unwrap();
    }
}
