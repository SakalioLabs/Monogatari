//! Bounded inspection and extraction for `.monogatari` project packages.

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::project::{sanitize_export_config, scrub_runtime_secret_config};

use super::manifest::is_json_path;
use super::{
    portable_case_key, validate_manifest, validate_portable_path, ArchiveFileRecord,
    ARCHIVE_MANIFEST_PATH, MAX_ARCHIVE_COMPRESSED_BYTES, MAX_ARCHIVE_DIRECTORIES,
    MAX_ARCHIVE_FILES, MAX_ARCHIVE_FILE_BYTES, MAX_ARCHIVE_JSON_BYTES, MAX_ARCHIVE_MANIFEST_BYTES,
};

const STREAM_BUFFER_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProjectPackageInspection {
    pub archive_path: String,
    pub project_title: String,
    pub engine_version: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub archive_bytes: u64,
    pub content_sha256: String,
    pub verified: bool,
}

/// Verify a project package without writing its contents.
pub fn inspect_project_package(archive_path: &Path) -> Result<ProjectPackageInspection, String> {
    verify_project_package(archive_path, None)
}

/// Verify and extract a project package into an existing empty regular directory.
///
/// The caller owns removal of the extraction root if validation fails after writes begin.
pub fn extract_project_package(
    archive_path: &Path,
    extraction_root: &Path,
) -> Result<ProjectPackageInspection, String> {
    let extraction_root = canonical_empty_extraction_root(extraction_root)?;
    verify_project_package(archive_path, Some(&extraction_root))
}

fn verify_project_package(
    archive_path: &Path,
    extraction_root: Option<&Path>,
) -> Result<ProjectPackageInspection, String> {
    let archive_path = canonical_regular_archive(archive_path)?;
    let archive_metadata = std::fs::metadata(&archive_path)
        .map_err(|error| format!("Unable to inspect project package: {error}"))?;
    if archive_metadata.len() > MAX_ARCHIVE_COMPRESSED_BYTES {
        return Err(format!(
            "Project package is {} bytes; the compressed size limit is {MAX_ARCHIVE_COMPRESSED_BYTES} bytes.",
            archive_metadata.len()
        ));
    }

    let file = File::open(&archive_path)
        .map_err(|error| format!("Unable to open project package: {error}"))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| format!("Unable to read project package ZIP: {error}"))?;
    let entry_limit = MAX_ARCHIVE_FILES + MAX_ARCHIVE_DIRECTORIES + 1;
    if archive.len() > entry_limit {
        return Err(format!(
            "Project package has {} entries; the limit is {entry_limit}.",
            archive.len()
        ));
    }

    let manifest_index = locate_manifest(&mut archive)?;
    let manifest_value = read_manifest(&mut archive, manifest_index)?;
    let validated = validate_manifest(manifest_value)?;

    if let Some(root) = extraction_root {
        for directory in &validated.allowed_directories {
            create_extraction_directory(root, directory)?;
        }
    }

    let mut seen_files = HashSet::new();
    let mut seen_entries = HashSet::new();
    let mut manifest_entries = 0usize;
    let mut settings_value = None;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("Unable to read ZIP entry {index}: {error}"))?;
        reject_non_regular_zip_entry(&entry)?;
        let raw_name = strict_zip_entry_name(&entry)?.to_string();
        if raw_name == ARCHIVE_MANIFEST_PATH {
            manifest_entries += 1;
            continue;
        }

        if entry.is_dir() {
            let directory = raw_name
                .strip_suffix('/')
                .ok_or_else(|| format!("Archive directory `{raw_name}` has an invalid name."))?;
            validate_portable_path(directory, "Archive directory")?;
            if !validated.allowed_directories.contains(directory) {
                return Err(format!(
                    "Project package contains an undeclared directory entry: {directory}"
                ));
            }
            let folded = portable_case_key(directory);
            if !seen_entries.insert(folded) {
                return Err(format!(
                    "Project package contains duplicate entry `{directory}`."
                ));
            }
            continue;
        }

        validate_portable_path(&raw_name, "Archive file")?;
        let folded = portable_case_key(&raw_name);
        if !seen_entries.insert(folded) {
            return Err(format!(
                "Project package contains duplicate entry `{raw_name}`."
            ));
        }
        let record = validated
            .files_by_path
            .get(&raw_name)
            .ok_or_else(|| format!("Project package contains undeclared file `{raw_name}`."))?;
        if entry.size() != record.size_bytes {
            return Err(format!(
                "Project package size mismatch for `{raw_name}`: manifest {}, archive {}.",
                record.size_bytes,
                entry.size()
            ));
        }

        let output_path = extraction_root
            .map(|root| extraction_file_path(root, &raw_name))
            .transpose()?;
        let capture_json = is_json_path(&raw_name);
        let captured =
            verify_and_extract_entry(&mut entry, record, output_path.as_deref(), capture_json)?;
        if capture_json {
            let bytes = captured.ok_or_else(|| format!("Unable to validate `{raw_name}`."))?;
            let value = serde_json::from_slice::<Value>(&bytes)
                .map_err(|error| format!("Imported JSON `{raw_name}` is invalid: {error}"))?;
            if raw_name == "settings.json" {
                settings_value = Some(value);
            }
        }
        seen_files.insert(raw_name);
    }

    if manifest_entries != 1 {
        return Err("Project package must contain exactly one manifest.".to_string());
    }
    if seen_files.len() != validated.files_by_path.len() {
        let missing = validated
            .files_by_path
            .keys()
            .filter(|path| !seen_files.contains(*path))
            .cloned()
            .collect::<Vec<_>>();
        return Err(format!(
            "Project package is missing declared files: {}",
            missing.join(", ")
        ));
    }

    let settings = settings_value
        .ok_or_else(|| "Project package is missing a valid `settings.json`.".to_string())?;
    if !settings.is_object() {
        return Err("Imported settings.json must contain a JSON object.".to_string());
    }
    if scrub_runtime_secret_config(&settings) != settings {
        return Err(
            "Imported settings.json contains runtime secrets and was rejected.".to_string(),
        );
    }
    if sanitize_export_config(&settings) != validated.parsed.settings {
        return Err(
            "Imported settings.json does not match the package manifest settings.".to_string(),
        );
    }

    Ok(ProjectPackageInspection {
        archive_path: archive_path.to_string_lossy().to_string(),
        project_title: validated.project_title,
        engine_version: validated.parsed.export_metadata.engine_version,
        file_count: validated.parsed.package.file_count,
        total_bytes: validated.parsed.package.total_bytes,
        archive_bytes: archive_metadata.len(),
        content_sha256: validated.parsed.package.content_sha256,
        verified: true,
    })
}

fn locate_manifest<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Result<usize, String> {
    let mut manifest_index = None;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| format!("Unable to inspect ZIP entry {index}: {error}"))?;
        reject_non_regular_zip_entry(&entry)?;
        if strict_zip_entry_name(&entry)? == ARCHIVE_MANIFEST_PATH
            && manifest_index.replace(index).is_some()
        {
            return Err("Project package contains duplicate manifests.".to_string());
        }
    }
    manifest_index.ok_or_else(|| format!("Project package is missing `{ARCHIVE_MANIFEST_PATH}`."))
}

fn read_manifest<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    manifest_index: usize,
) -> Result<Value, String> {
    let mut entry = archive
        .by_index(manifest_index)
        .map_err(|error| format!("Unable to read project package manifest: {error}"))?;
    if entry.is_dir() || entry.size() > MAX_ARCHIVE_MANIFEST_BYTES {
        return Err(format!(
            "Project package manifest exceeds the {MAX_ARCHIVE_MANIFEST_BYTES} byte limit or is not a file."
        ));
    }
    let bytes = read_entry_bytes(&mut entry, MAX_ARCHIVE_MANIFEST_BYTES)?;
    serde_json::from_slice::<Value>(&bytes)
        .map_err(|error| format!("Project package manifest is invalid JSON: {error}"))
}

fn verify_and_extract_entry<R: Read>(
    entry: &mut R,
    record: &ArchiveFileRecord,
    output_path: Option<&Path>,
    capture: bool,
) -> Result<Option<Vec<u8>>, String> {
    if capture && record.size_bytes > MAX_ARCHIVE_JSON_BYTES {
        return Err(format!(
            "JSON file `{}` exceeds the validation limit of {MAX_ARCHIVE_JSON_BYTES} bytes.",
            record.path
        ));
    }
    let mut output = if let Some(path) = output_path {
        Some(
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
                .map_err(|error| {
                    format!(
                        "Unable to create imported file `{}`: {error}",
                        path.display()
                    )
                })?,
        )
    } else {
        None
    };
    let mut captured = capture.then(|| Vec::with_capacity(record.size_bytes as usize));
    let mut sha256 = Sha256::new();
    let mut md5 = md5::Context::new();
    let mut total = 0u64;
    let mut buffer = [0u8; STREAM_BUFFER_BYTES];
    loop {
        let read = entry
            .read(&mut buffer)
            .map_err(|error| format!("Unable to read `{}` from package: {error}", record.path))?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read as u64)
            .ok_or_else(|| format!("Package file `{}` size overflowed.", record.path))?;
        if total > record.size_bytes || total > MAX_ARCHIVE_FILE_BYTES {
            return Err(format!(
                "Package file `{}` expanded beyond its declared size.",
                record.path
            ));
        }
        sha256.update(&buffer[..read]);
        md5.consume(&buffer[..read]);
        if let Some(bytes) = captured.as_mut() {
            bytes.extend_from_slice(&buffer[..read]);
        }
        if let Some(file) = output.as_mut() {
            file.write_all(&buffer[..read]).map_err(|error| {
                format!("Unable to write imported file `{}`: {error}", record.path)
            })?;
        }
    }
    if total != record.size_bytes {
        return Err(format!(
            "Package file `{}` contains {total} bytes; {} were declared.",
            record.path, record.size_bytes
        ));
    }
    if finish_sha256(sha256) != record.checksum_sha256 {
        return Err(format!(
            "SHA-256 mismatch for package file `{}`.",
            record.path
        ));
    }
    if !record.checksum_md5.is_empty() && format!("{:x}", md5.compute()) != record.checksum_md5 {
        return Err(format!("MD5 mismatch for package file `{}`.", record.path));
    }
    if let Some(file) = output.as_mut() {
        file.flush()
            .map_err(|error| format!("Unable to flush imported file `{}`: {error}", record.path))?;
        file.sync_all()
            .map_err(|error| format!("Unable to sync imported file `{}`: {error}", record.path))?;
    }
    Ok(captured)
}

fn read_entry_bytes<R: Read>(reader: &mut R, limit: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Unable to read project package entry: {error}"))?;
    if bytes.len() as u64 > limit {
        return Err(format!(
            "Project package entry exceeds the {limit} byte limit."
        ));
    }
    Ok(bytes)
}

fn strict_zip_entry_name<'a, R: Read>(
    entry: &'a zip::read::ZipFile<'_, R>,
) -> Result<&'a str, String> {
    let raw_name = std::str::from_utf8(entry.name_raw()).map_err(|_| {
        "Project package entry names must use unambiguous UTF-8 encoding.".to_string()
    })?;
    if raw_name != entry.name() {
        return Err(format!(
            "Project package entry `{}` has ambiguous path metadata.",
            entry.name()
        ));
    }
    Ok(raw_name)
}

fn reject_non_regular_zip_entry<R: Read>(entry: &zip::read::ZipFile<'_, R>) -> Result<(), String> {
    if entry.encrypted() {
        return Err(format!(
            "Project package entry `{}` must not be encrypted.",
            entry.name()
        ));
    }
    if let Some(mode) = entry.unix_mode() {
        let file_type = mode & 0o170000;
        let expected = if entry.is_dir() { 0o040000 } else { 0o100000 };
        if file_type != 0 && file_type != expected {
            return Err(format!(
                "Project package entry `{}` is not a regular file or directory.",
                entry.name()
            ));
        }
    }
    Ok(())
}

fn canonical_regular_archive(path: &Path) -> Result<PathBuf, String> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("monogatari"))
    {
        return Err("Project package paths must use the `.monogatari` extension.".to_string());
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        format!(
            "Unable to inspect project package `{}`: {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "Project package must be a regular file: {}",
            path.display()
        ));
    }
    path.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve project package `{}`: {error}",
            path.display()
        )
    })
}

fn canonical_empty_extraction_root(path: &Path) -> Result<PathBuf, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        format!(
            "Unable to inspect project package extraction root `{}`: {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "Project package extraction root must be a regular directory: {}",
            path.display()
        ));
    }
    let canonical = path.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve project package extraction root `{}`: {error}",
            path.display()
        )
    })?;
    let mut entries = std::fs::read_dir(&canonical).map_err(|error| {
        format!(
            "Unable to read project package extraction root `{}`: {error}",
            canonical.display()
        )
    })?;
    if entries
        .next()
        .transpose()
        .map_err(|error| {
            format!(
                "Unable to inspect project package extraction root `{}`: {error}",
                canonical.display()
            )
        })?
        .is_some()
    {
        return Err("Project package extraction root must be empty.".to_string());
    }
    Ok(canonical)
}

fn create_extraction_directory(root: &Path, relative: &str) -> Result<(), String> {
    validate_portable_path(relative, "Package directory")?;
    let mut current = root.to_path_buf();
    for segment in relative.split('/') {
        current.push(segment);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(format!(
                        "Imported directory target must remain a regular directory: {}",
                        current.display()
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(&current).map_err(|error| {
                    format!(
                        "Unable to create imported directory `{}`: {error}",
                        current.display()
                    )
                })?;
            }
            Err(error) => {
                return Err(format!(
                    "Unable to inspect imported directory `{}`: {error}",
                    current.display()
                ));
            }
        }
        ensure_canonical_descendant(root, &current, "Imported directory")?;
    }
    Ok(())
}

fn extraction_file_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    validate_portable_path(relative, "Archive file")?;
    if let Some((parent, _)) = relative.rsplit_once('/') {
        create_extraction_directory(root, parent)?;
    }
    let target = root.join(relative);
    let parent = target
        .parent()
        .ok_or_else(|| format!("Imported file `{relative}` has no parent directory."))?;
    ensure_canonical_descendant(root, parent, "Imported file parent")?;
    if !target.starts_with(root) {
        return Err("Project package extraction attempted to escape its staging root.".to_string());
    }
    Ok(target)
}

fn ensure_canonical_descendant(root: &Path, path: &Path, label: &str) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("Unable to inspect {label} `{}`: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "{label} must be a regular directory: {}",
            path.display()
        ));
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("Unable to resolve {label} `{}`: {error}", path.display()))?;
    if !canonical.starts_with(root) {
        return Err("Project package extraction attempted to escape its staging root.".to_string());
    }
    Ok(())
}

fn finish_sha256(hasher: Sha256) -> String {
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests;
