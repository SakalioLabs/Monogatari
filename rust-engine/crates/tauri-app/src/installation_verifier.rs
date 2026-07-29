//! Headless verification for a project-free extracted or installed desktop shell.

use std::ffi::{OsStr, OsString};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;
use serde::Serialize;

const VERIFY_FLAG: &str = "--verify-installation";
const REPORT_SCHEMA: &str = "monogatari-installation-verification/v2";
const MAX_REPORT_BYTES: usize = 1024 * 1024;
const PROHIBITED_PROJECT_ENTRIES: &[&str] = &[
    "data",
    "campaigns",
    "characters",
    "dialogue",
    "endings",
    "events",
    "knowledge",
    "models",
    "project-assets.json",
    "quality_suites",
    "roleplays",
    "scenes",
    "settings.json",
    "workflows",
];
static REPORT_STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    pub project_content_embedded: bool,
    pub prohibited_project_entries_checked: Vec<String>,
}

#[derive(Debug, Serialize)]
struct InstallationVerificationEnvelope {
    schema: &'static str,
    status: &'static str,
    generated_at: String,
    report: Option<InstallationVerificationReport>,
    error: Option<String>,
}

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

    let (envelope, exit_code) = match verify_installed_application(&executable_path) {
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

pub(crate) fn verify_installed_application(
    executable_path: &Path,
) -> Result<InstallationVerificationReport, String> {
    let executable_path = canonical_regular_file(executable_path, "Application executable")?;
    let resource_root = executable_path
        .parent()
        .ok_or_else(|| "Application executable has no resource directory.".to_string())?;

    let embedded_entries = PROHIBITED_PROJECT_ENTRIES
        .iter()
        .filter(|entry| resource_root.join(entry).exists())
        .copied()
        .collect::<Vec<_>>();
    if !embedded_entries.is_empty() {
        return Err(format!(
            "Installed engine must not contain project content: {}.",
            embedded_entries.join(", ")
        ));
    }

    Ok(InstallationVerificationReport {
        schema: REPORT_SCHEMA.to_string(),
        status: "verified".to_string(),
        verified_at: Utc::now().to_rfc3339(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: env!("MONOGATARI_GIT_COMMIT").to_string(),
        git_short_commit: env!("MONOGATARI_GIT_SHORT_COMMIT").to_string(),
        executable_path: executable_path.to_string_lossy().to_string(),
        resource_root: resource_root.to_string_lossy().to_string(),
        project_content_embedded: false,
        prohibited_project_entries_checked: PROHIBITED_PROJECT_ENTRIES
            .iter()
            .map(|entry| (*entry).to_string())
            .collect(),
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

fn canonical_regular_file(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("{label} is unavailable: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("{label} must be a regular file."));
    }
    path.canonicalize()
        .map_err(|error| format!("Unable to resolve {label}: {error}"))
}

fn canonical_regular_directory(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("{label} is unavailable: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{label} must be a regular directory."));
    }
    path.canonicalize()
        .map_err(|error| format!("Unable to resolve {label}: {error}"))
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

    #[test]
    fn project_free_installation_passes_verification() {
        let root = temp_root("clean");
        std::fs::create_dir_all(&root).unwrap();
        let executable = root.join("llm-galgame-app.exe");
        std::fs::write(&executable, b"test").unwrap();

        let report = verify_installed_application(&executable).unwrap();
        assert!(!report.project_content_embedded);
        assert_eq!(
            PathBuf::from(report.resource_root),
            root.canonicalize().unwrap()
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn installation_rejects_adjacent_project_content() {
        let root = temp_root("content");
        std::fs::create_dir_all(root.join("data")).unwrap();
        let executable = root.join("llm-galgame-app.exe");
        std::fs::write(&executable, b"test").unwrap();

        let error = verify_installed_application(&executable).unwrap_err();
        assert!(error.contains("data"), "{error}");

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn report_path_requires_an_absolute_json_target() {
        assert!(validate_report_path(PathBuf::from("report.json")).is_err());
        assert!(validate_report_path(PathBuf::from("report.txt")).is_err());
    }
}
