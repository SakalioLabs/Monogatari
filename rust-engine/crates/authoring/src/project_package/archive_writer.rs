//! Atomic streaming writer for validated `.monogatari` project packages.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::project::canonical_project_root;

use super::{
    project_export_settings_bytes, validate_manifest, ArchiveFileRecord, ARCHIVE_MANIFEST_PATH,
    MAX_ARCHIVE_FILE_BYTES,
};

static PACKAGE_STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageTargetPolicy {
    CreateNew,
    ReplaceExisting,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProjectPackageExportResult {
    pub archive_path: String,
    pub project_path: String,
    pub project_title: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub archive_bytes: u64,
    pub content_sha256: String,
}

/// Write a validated manifest and its inventoried project files through a staged archive.
pub fn write_project_package(
    project_root: &Path,
    destination: &Path,
    manifest_value: Value,
    target_policy: ProjectPackageTargetPolicy,
) -> Result<ProjectPackageExportResult, String> {
    let project_root = canonical_project_root(project_root)?;
    let validated = validate_manifest(manifest_value)?;
    let destination = normalize_destination(destination)?;
    ensure_package_target(&destination, target_policy)?;

    let parent = destination
        .parent()
        .expect("normalized package destinations always have a parent");
    let stage_path = unique_stage_path(parent, "tmp")?;
    let write_result = write_staged_package(
        &project_root,
        &stage_path,
        &validated.raw,
        &validated.parsed.settings,
        validated.files_by_path.values(),
    );
    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&stage_path);
        return Err(error);
    }
    if let Err(error) = commit_staged_package(&stage_path, &destination, target_policy) {
        let _ = std::fs::remove_file(&stage_path);
        return Err(error);
    }
    let archive_bytes = std::fs::metadata(&destination)
        .map_err(|error| format!("Unable to inspect exported project package: {error}"))?
        .len();

    Ok(ProjectPackageExportResult {
        archive_path: destination.to_string_lossy().to_string(),
        project_path: project_root.to_string_lossy().to_string(),
        project_title: validated.project_title,
        file_count: validated.parsed.package.file_count,
        total_bytes: validated.parsed.package.total_bytes,
        archive_bytes,
        content_sha256: validated.parsed.package.content_sha256,
    })
}

fn normalize_destination(destination: &Path) -> Result<PathBuf, String> {
    if destination
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("monogatari"))
    {
        return Err("Project package destinations must use the `.monogatari` extension.".into());
    }
    let file_name = destination
        .file_name()
        .ok_or_else(|| "Project package destination has no file name.".to_string())?;
    let parent = destination
        .parent()
        .ok_or_else(|| "Project package destination must have a parent directory.".to_string())?;
    let parent = canonical_regular_directory(parent, "Project package destination")?;
    Ok(parent.join(file_name))
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

fn ensure_package_target(
    destination: &Path,
    target_policy: ProjectPackageTargetPolicy,
) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(destination) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "Unable to inspect existing project package `{}`: {error}",
                destination.display()
            ));
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "Existing project package target must be a regular file: {}",
            destination.display()
        ));
    }
    if target_policy == ProjectPackageTargetPolicy::CreateNew {
        return Err(format!(
            "Project package destination already exists: {}",
            destination.display()
        ));
    }
    Ok(())
}

fn write_staged_package<'a>(
    project_root: &Path,
    stage_path: &Path,
    manifest: &Value,
    manifest_settings: &Value,
    records: impl Iterator<Item = &'a ArchiveFileRecord>,
) -> Result<(), String> {
    let stage_file = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(stage_path)
        .map_err(|error| {
            format!(
                "Unable to create staged project package `{}`: {error}",
                stage_path.display()
            )
        })?;
    let mut writer = ZipWriter::new(stage_file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    let manifest_bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| format!("Unable to encode project package manifest: {error}"))?;
    writer
        .start_file(ARCHIVE_MANIFEST_PATH, options)
        .map_err(|error| format!("Unable to add project package manifest: {error}"))?;
    writer
        .write_all(&manifest_bytes)
        .map_err(|error| format!("Unable to write project package manifest: {error}"))?;

    for record in records {
        write_export_record(
            &mut writer,
            options,
            project_root,
            manifest_settings,
            record,
        )?;
    }

    let file = writer
        .finish()
        .map_err(|error| format!("Unable to finalize project package: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("Unable to sync project package: {error}"))
}

fn write_export_record(
    writer: &mut ZipWriter<File>,
    options: SimpleFileOptions,
    project_root: &Path,
    manifest_settings: &Value,
    record: &ArchiveFileRecord,
) -> Result<(), String> {
    if record.path == "settings.json" {
        let bytes = project_export_settings_bytes(manifest_settings)?;
        verify_export_bytes(record, &bytes)?;
        writer.start_file(&record.path, options).map_err(|error| {
            format!(
                "Unable to add `{}` to project package: {error}",
                record.path
            )
        })?;
        return writer.write_all(&bytes).map_err(|error| {
            format!(
                "Unable to write `{}` to project package: {error}",
                record.path
            )
        });
    }

    let path = project_root.join(&record.path);
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
        format!(
            "Unable to inspect export source `{}`: {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "Project export source must be a regular file: {}",
            path.display()
        ));
    }
    if metadata.len() != record.size_bytes {
        return Err(format!(
            "Project file `{}` changed size while the package was being created.",
            record.path
        ));
    }
    let canonical_path = path.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve export source `{}`: {error}",
            path.display()
        )
    })?;
    if !canonical_path.starts_with(project_root) {
        return Err(format!(
            "Project export source escapes the project root: {}",
            path.display()
        ));
    }
    let mut source = File::open(&canonical_path).map_err(|error| {
        format!(
            "Unable to open export source `{}`: {error}",
            canonical_path.display()
        )
    })?;
    writer.start_file(&record.path, options).map_err(|error| {
        format!(
            "Unable to add `{}` to project package: {error}",
            record.path
        )
    })?;

    let mut sha256 = Sha256::new();
    let mut md5 = md5::Context::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|error| format!("Unable to read export source `{}`: {error}", record.path))?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read as u64)
            .ok_or_else(|| format!("Project file `{}` size overflowed.", record.path))?;
        if total > record.size_bytes || total > MAX_ARCHIVE_FILE_BYTES {
            return Err(format!(
                "Project file `{}` expanded while the package was being created.",
                record.path
            ));
        }
        sha256.update(&buffer[..read]);
        md5.consume(&buffer[..read]);
        writer.write_all(&buffer[..read]).map_err(|error| {
            format!(
                "Unable to write `{}` to project package: {error}",
                record.path
            )
        })?;
    }
    if total != record.size_bytes {
        return Err(format!(
            "Project file `{}` changed size while the package was being created.",
            record.path
        ));
    }
    if finish_sha256(sha256) != record.checksum_sha256 {
        return Err(format!(
            "Project file `{}` changed while the package was being created.",
            record.path
        ));
    }
    if !record.checksum_md5.is_empty() && format!("{:x}", md5.compute()) != record.checksum_md5 {
        return Err(format!(
            "Project file `{}` failed its compatibility checksum.",
            record.path
        ));
    }
    Ok(())
}

fn verify_export_bytes(record: &ArchiveFileRecord, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() as u64 != record.size_bytes {
        return Err(format!(
            "Project file `{}` changed size while the package was being created.",
            record.path
        ));
    }
    if format!("{:x}", Sha256::digest(bytes)) != record.checksum_sha256 {
        return Err(format!(
            "Project file `{}` changed while the package was being created.",
            record.path
        ));
    }
    if !record.checksum_md5.is_empty()
        && format!("{:x}", md5::compute(bytes)) != record.checksum_md5
    {
        return Err(format!(
            "Project file `{}` failed its compatibility checksum.",
            record.path
        ));
    }
    Ok(())
}

fn commit_staged_package(
    stage_path: &Path,
    destination: &Path,
    target_policy: ProjectPackageTargetPolicy,
) -> Result<(), String> {
    ensure_package_target(destination, target_policy)?;
    let parent = destination
        .parent()
        .expect("normalized package destinations always have a parent");
    let backup_path = unique_stage_path(parent, "backup")?;
    let had_destination = destination.exists();
    if had_destination {
        std::fs::rename(destination, &backup_path).map_err(|error| {
            format!("Unable to stage the previous project package for replacement: {error}")
        })?;
    }
    if let Err(error) = std::fs::rename(stage_path, destination) {
        if had_destination {
            let _ = std::fs::rename(&backup_path, destination);
        }
        let _ = std::fs::remove_file(stage_path);
        return Err(format!("Unable to commit project package: {error}"));
    }
    if had_destination {
        std::fs::remove_file(&backup_path).map_err(|error| {
            format!("Project package was replaced, but its backup could not be removed: {error}")
        })?;
    }
    Ok(())
}

fn unique_stage_path(parent: &Path, suffix: &str) -> Result<PathBuf, String> {
    for _ in 0..1000 {
        let counter = PACKAGE_STAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".monogatari-export-{}-{counter}.{suffix}",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("Unable to allocate a unique project package staging path.".to_string())
}

fn finish_sha256(hasher: Sha256) -> String {
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests;
