//! Verified `.monogatari` project package export and import.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[cfg(test)]
use super::project::build_project_export_manifest;
use super::project::{
    project_export_settings_bytes, sanitize_export_config, scrub_runtime_secret_config,
};

pub(crate) mod commands;

const ARCHIVE_MANIFEST_PATH: &str = "monogatari-project.json";
const ARCHIVE_FORMAT: &str = "monogatari-project";
const ARCHIVE_SCHEMA: &str = "monogatari-project-export@1";
const PACKAGE_FINGERPRINT_ALGORITHM: &str = "sha256:path-size-file-sha256-v1";
const MAX_ARCHIVE_FILES: usize = 20_000;
const MAX_ARCHIVE_DIRECTORIES: usize = 4_000;
const MAX_ARCHIVE_COMPRESSED_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_ARCHIVE_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_ARCHIVE_FILE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_ARCHIVE_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const MAX_ARCHIVE_JSON_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PORTABLE_PATH_BYTES: usize = 512;
const MAX_PORTABLE_SEGMENTS: usize = 32;
const MAX_PORTABLE_SEGMENT_BYTES: usize = 160;

static ARCHIVE_STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize)]
pub struct ProjectArchiveInspection {
    pub archive_path: String,
    pub project_title: String,
    pub engine_version: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub archive_bytes: u64,
    pub content_sha256: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectArchiveExportResult {
    pub archive_path: String,
    pub project_path: String,
    pub project_title: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub archive_bytes: u64,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectArchiveImportResult {
    pub archive_path: String,
    pub project_path: String,
    pub project_title: String,
    pub directory_name: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ArchiveManifest {
    format: String,
    schema: String,
    version: String,
    #[serde(default)]
    export_metadata: ArchiveExportMetadata,
    settings: Value,
    package: ArchivePackage,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ArchiveExportMetadata {
    #[serde(default)]
    engine_version: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ArchivePackage {
    file_count: usize,
    total_bytes: u64,
    fingerprint_algorithm: String,
    content_sha256: String,
    #[serde(default)]
    directories: Vec<String>,
    files: Vec<ArchiveFileRecord>,
}

#[derive(Debug, Clone, Deserialize)]
struct ArchiveFileRecord {
    category: String,
    path: String,
    size_bytes: u64,
    #[serde(default)]
    checksum_md5: String,
    checksum_sha256: String,
}

#[derive(Debug, Clone)]
struct ValidatedManifest {
    raw: Value,
    parsed: ArchiveManifest,
    files_by_path: BTreeMap<String, ArchiveFileRecord>,
    allowed_directories: BTreeSet<String>,
    project_title: String,
}

#[derive(Debug, Clone)]
struct VerifiedArchive {
    inspection: ProjectArchiveInspection,
}

fn write_project_archive(
    project_root: &Path,
    destination: &Path,
    manifest_value: Value,
) -> Result<ProjectArchiveExportResult, String> {
    let validated = validate_manifest(manifest_value)?;
    let parent = destination
        .parent()
        .ok_or_else(|| "Project package destination must have a parent directory.".to_string())?;
    let parent = canonical_regular_directory(parent, "Project package destination")?;
    let destination = parent.join(
        destination
            .file_name()
            .ok_or_else(|| "Project package destination has no file name.".to_string())?,
    );
    ensure_replaceable_archive_target(&destination)?;

    let stage_path = unique_sibling_path(&parent, ".monogatari-export", "tmp")?;
    let write_result = (|| {
        let stage_file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&stage_path)
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
        let manifest_bytes = serde_json::to_vec_pretty(&validated.raw)
            .map_err(|error| format!("Unable to encode project package manifest: {error}"))?;
        writer
            .start_file(ARCHIVE_MANIFEST_PATH, options)
            .map_err(|error| format!("Unable to add project package manifest: {error}"))?;
        writer
            .write_all(&manifest_bytes)
            .map_err(|error| format!("Unable to write project package manifest: {error}"))?;

        for record in validated.files_by_path.values() {
            write_export_record(
                &mut writer,
                options,
                project_root,
                &validated.parsed.settings,
                record,
            )?;
        }

        let file = writer
            .finish()
            .map_err(|error| format!("Unable to finalize project package: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("Unable to sync project package: {error}"))?;
        Ok::<(), String>(())
    })();

    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&stage_path);
        return Err(error);
    }
    atomic_replace_archive(&stage_path, &destination)?;
    let archive_bytes = std::fs::metadata(&destination)
        .map_err(|error| format!("Unable to inspect exported project package: {error}"))?
        .len();

    Ok(ProjectArchiveExportResult {
        archive_path: destination.to_string_lossy().to_string(),
        project_path: project_root.to_string_lossy().to_string(),
        project_title: validated.project_title,
        file_count: validated.parsed.package.file_count,
        total_bytes: validated.parsed.package.total_bytes,
        archive_bytes,
        content_sha256: validated.parsed.package.content_sha256,
    })
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
    let canonical_root = project_root
        .canonicalize()
        .map_err(|error| format!("Unable to resolve project root: {error}"))?;
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

fn verify_archive(
    archive_path: &Path,
    extraction_root: Option<&Path>,
) -> Result<VerifiedArchive, String> {
    let archive_metadata = std::fs::metadata(archive_path)
        .map_err(|error| format!("Unable to inspect project package: {error}"))?;
    if archive_metadata.len() > MAX_ARCHIVE_COMPRESSED_BYTES {
        return Err(format!(
            "Project package is {} bytes; the compressed size limit is {} bytes.",
            archive_metadata.len(),
            MAX_ARCHIVE_COMPRESSED_BYTES
        ));
    }
    let file = File::open(archive_path)
        .map_err(|error| format!("Unable to open project package: {error}"))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| format!("Unable to read project package ZIP: {error}"))?;
    if archive.len() > MAX_ARCHIVE_FILES + MAX_ARCHIVE_DIRECTORIES + 1 {
        return Err(format!(
            "Project package has {} entries; the limit is {}.",
            archive.len(),
            MAX_ARCHIVE_FILES + MAX_ARCHIVE_DIRECTORIES + 1
        ));
    }

    let mut manifest_index = None;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| format!("Unable to inspect ZIP entry {index}: {error}"))?;
        if entry.name() == ARCHIVE_MANIFEST_PATH && manifest_index.replace(index).is_some() {
            return Err("Project package contains duplicate manifests.".to_string());
        }
    }
    let manifest_index = manifest_index
        .ok_or_else(|| format!("Project package is missing `{ARCHIVE_MANIFEST_PATH}`."))?;
    let manifest_value = {
        let mut entry = archive
            .by_index(manifest_index)
            .map_err(|error| format!("Unable to read project package manifest: {error}"))?;
        if entry.size() > MAX_ARCHIVE_MANIFEST_BYTES {
            return Err("Project package manifest exceeds the 4 MiB limit.".to_string());
        }
        let bytes = read_entry_bytes(&mut entry, MAX_ARCHIVE_MANIFEST_BYTES)?;
        serde_json::from_slice::<Value>(&bytes)
            .map_err(|error| format!("Project package manifest is invalid JSON: {error}"))?
    };
    let validated = validate_manifest(manifest_value)?;

    if let Some(root) = extraction_root {
        for directory in &validated.parsed.package.directories {
            let path = root.join(directory);
            ensure_extraction_target(root, &path)?;
            std::fs::create_dir_all(&path).map_err(|error| {
                format!("Unable to create imported directory `{directory}`: {error}")
            })?;
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
        let raw_name = entry.name().to_string();
        if raw_name == ARCHIVE_MANIFEST_PATH {
            manifest_entries += 1;
            continue;
        }
        if entry.is_dir() {
            let directory = raw_name.trim_end_matches('/');
            validate_portable_path(directory, "Archive directory")?;
            if !validated.allowed_directories.contains(directory) {
                return Err(format!(
                    "Project package contains an undeclared directory entry: {directory}"
                ));
            }
            let folded = portable_case_key(directory);
            if !seen_entries.insert(format!("dir:{folded}")) {
                return Err(format!(
                    "Project package contains duplicate entry `{directory}`."
                ));
            }
            continue;
        }

        validate_portable_path(&raw_name, "Archive file")?;
        let folded = portable_case_key(&raw_name);
        if !seen_entries.insert(format!("file:{folded}")) {
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

        let output_path = extraction_root.map(|root| root.join(&raw_name));
        if let (Some(root), Some(path)) = (extraction_root, output_path.as_ref()) {
            ensure_extraction_target(root, path)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "Unable to create import directory `{}`: {error}",
                        parent.display()
                    )
                })?;
            }
        }
        let capture_json = raw_name.ends_with(".json");
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

    Ok(VerifiedArchive {
        inspection: ProjectArchiveInspection {
            archive_path: archive_path.to_string_lossy().to_string(),
            project_title: validated.project_title,
            engine_version: validated.parsed.export_metadata.engine_version,
            file_count: validated.parsed.package.file_count,
            total_bytes: validated.parsed.package.total_bytes,
            archive_bytes: archive_metadata.len(),
            content_sha256: validated.parsed.package.content_sha256,
            verified: true,
        },
    })
}

fn validate_manifest(raw: Value) -> Result<ValidatedManifest, String> {
    let parsed = serde_json::from_value::<ArchiveManifest>(raw.clone())
        .map_err(|error| format!("Project package manifest has an invalid shape: {error}"))?;
    if parsed.format != ARCHIVE_FORMAT {
        return Err(format!(
            "Unsupported project package format `{}`.",
            parsed.format
        ));
    }
    if parsed.schema != ARCHIVE_SCHEMA {
        return Err(format!(
            "Unsupported project package schema `{}`.",
            parsed.schema
        ));
    }
    if parsed.version != "1.0" {
        return Err(format!(
            "Unsupported project package version `{}`.",
            parsed.version
        ));
    }
    if !parsed.settings.is_object() {
        return Err("Project package manifest settings must be an object.".to_string());
    }
    if parsed.package.fingerprint_algorithm != PACKAGE_FINGERPRINT_ALGORITHM {
        return Err(format!(
            "Unsupported package fingerprint algorithm `{}`.",
            parsed.package.fingerprint_algorithm
        ));
    }
    validate_sha256(&parsed.package.content_sha256, "Package content SHA-256")?;
    if parsed.package.files.len() != parsed.package.file_count {
        return Err(format!(
            "Package file_count is {}, but {} file records were declared.",
            parsed.package.file_count,
            parsed.package.files.len()
        ));
    }
    if parsed.package.files.is_empty() || parsed.package.files.len() > MAX_ARCHIVE_FILES {
        return Err(format!(
            "Project package must contain between 1 and {MAX_ARCHIVE_FILES} files."
        ));
    }
    if parsed.package.directories.len() > MAX_ARCHIVE_DIRECTORIES {
        return Err(format!(
            "Project package declares too many directories; the limit is {MAX_ARCHIVE_DIRECTORIES}."
        ));
    }

    let mut directory_keys = HashSet::new();
    let mut allowed_directories = BTreeSet::new();
    for directory in &parsed.package.directories {
        validate_portable_path(directory, "Package directory")?;
        if !directory_keys.insert(portable_case_key(directory)) {
            return Err(format!(
                "Project package declares a duplicate directory `{directory}`."
            ));
        }
        add_directory_and_parents(&mut allowed_directories, directory);
    }

    let mut files_by_path = BTreeMap::new();
    let mut file_keys = HashSet::new();
    let mut total_bytes = 0u64;
    let mut ordered_paths = Vec::new();
    for record in &parsed.package.files {
        validate_portable_path(&record.path, "Package file")?;
        if record.path == ARCHIVE_MANIFEST_PATH {
            return Err(format!(
                "`{ARCHIVE_MANIFEST_PATH}` cannot inventory itself."
            ));
        }
        if record.category.trim().is_empty()
            || record.category.len() > 64
            || !record
                .category
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch == '_')
        {
            return Err(format!(
                "Package file `{}` has invalid category `{}`.",
                record.path, record.category
            ));
        }
        if record.size_bytes > MAX_ARCHIVE_FILE_BYTES {
            return Err(format!(
                "Package file `{}` exceeds the per-file limit of {} bytes.",
                record.path, MAX_ARCHIVE_FILE_BYTES
            ));
        }
        validate_sha256(
            &record.checksum_sha256,
            &format!("SHA-256 for `{}`", record.path),
        )?;
        if !record.checksum_md5.is_empty()
            && (record.checksum_md5.len() != 32
                || !record.checksum_md5.chars().all(|ch| ch.is_ascii_hexdigit()))
        {
            return Err(format!("MD5 for `{}` is invalid.", record.path));
        }
        let folded = portable_case_key(&record.path);
        if !file_keys.insert(folded) {
            return Err(format!(
                "Project package declares a duplicate portable file path `{}`.",
                record.path
            ));
        }
        if record.path != "settings.json"
            && !parsed
                .package
                .directories
                .iter()
                .any(|directory| record.path.starts_with(&format!("{directory}/")))
        {
            return Err(format!(
                "Package file `{}` is outside declared project directories.",
                record.path
            ));
        }
        total_bytes = total_bytes
            .checked_add(record.size_bytes)
            .ok_or_else(|| "Project package total size overflowed.".to_string())?;
        if total_bytes > MAX_ARCHIVE_TOTAL_BYTES {
            return Err(format!(
                "Project package exceeds the total size limit of {MAX_ARCHIVE_TOTAL_BYTES} bytes."
            ));
        }
        ordered_paths.push(record.path.clone());
        files_by_path.insert(record.path.clone(), record.clone());
    }
    if !files_by_path.contains_key("settings.json") {
        return Err("Project package must include settings.json.".to_string());
    }
    validate_package_path_topology(&files_by_path, &directory_keys)?;
    let mut sorted_paths = ordered_paths.clone();
    sorted_paths.sort();
    if ordered_paths != sorted_paths {
        return Err("Project package file inventory must be sorted by path.".to_string());
    }
    if total_bytes != parsed.package.total_bytes {
        return Err(format!(
            "Package total_bytes is {}, but file records total {total_bytes}.",
            parsed.package.total_bytes
        ));
    }
    let fingerprint = package_fingerprint(files_by_path.values());
    if fingerprint != parsed.package.content_sha256 {
        return Err(
            "Project package content fingerprint does not match its inventory.".to_string(),
        );
    }

    let project_title = parsed
        .settings
        .get("render")
        .and_then(|render| render.get("title"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .unwrap_or("Monogatari Project")
        .chars()
        .take(120)
        .collect::<String>();

    Ok(ValidatedManifest {
        raw,
        parsed,
        files_by_path,
        allowed_directories,
        project_title,
    })
}

fn validate_package_path_topology(
    files_by_path: &BTreeMap<String, ArchiveFileRecord>,
    directory_keys: &HashSet<String>,
) -> Result<(), String> {
    let file_keys = files_by_path
        .keys()
        .map(|path| portable_case_key(path))
        .collect::<HashSet<_>>();

    for path in files_by_path.keys() {
        let folded = portable_case_key(path);
        if directory_keys.contains(&folded) {
            return Err(format!(
                "Project package path `{path}` is declared as both a file and a directory."
            ));
        }

        let mut ancestor = String::new();
        for segment in path.split('/').take(path.split('/').count() - 1) {
            if !ancestor.is_empty() {
                ancestor.push('/');
            }
            ancestor.push_str(segment);
            if file_keys.contains(&portable_case_key(&ancestor)) {
                return Err(format!(
                    "Project package file `{ancestor}` cannot contain descendant `{path}`."
                ));
            }
        }
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
    if sha256_hex(bytes) != record.checksum_sha256 {
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
    let mut buffer = [0u8; 64 * 1024];
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
    let sha256 = finish_sha256(sha256);
    if sha256 != record.checksum_sha256 {
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

fn reject_non_regular_zip_entry<R: Read>(entry: &zip::read::ZipFile<'_, R>) -> Result<(), String> {
    if let Some(mode) = entry.unix_mode() {
        let file_type = mode & 0o170000;
        if file_type != 0 && file_type != 0o100000 && file_type != 0o040000 {
            return Err(format!(
                "Project package entry `{}` is not a regular file or directory.",
                entry.name()
            ));
        }
    }
    Ok(())
}

fn package_fingerprint<'a>(records: impl Iterator<Item = &'a ArchiveFileRecord>) -> String {
    let mut hasher = Sha256::new();
    for record in records {
        hasher.update(record.path.as_bytes());
        hasher.update(b"\0");
        hasher.update(record.size_bytes.to_string().as_bytes());
        hasher.update(b"\0");
        hasher.update(record.checksum_sha256.as_bytes());
        hasher.update(b"\n");
    }
    finish_sha256(hasher)
}

fn validate_sha256(value: &str, label: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, 'a'..='f'))
    {
        return Err(format!("{label} must be a lowercase hexadecimal SHA-256."));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    finish_sha256(Sha256::new_with_prefix(bytes))
}

fn finish_sha256(hasher: Sha256) -> String {
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn validate_portable_path(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > MAX_PORTABLE_PATH_BYTES
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err(format!(
            "{label} `{value}` is not a portable relative path."
        ));
    }
    let segments = value.split('/').collect::<Vec<_>>();
    if segments.is_empty() || segments.len() > MAX_PORTABLE_SEGMENTS {
        return Err(format!("{label} `{value}` has too many path segments."));
    }
    for segment in segments {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || segment.len() > MAX_PORTABLE_SEGMENT_BYTES
            || segment.ends_with(' ')
            || segment.ends_with('.')
            || segment
                .chars()
                .any(|ch| matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
            || is_reserved_windows_segment(segment)
        {
            return Err(format!(
                "{label} `{value}` contains an unsafe path segment."
            ));
        }
    }
    Ok(())
}

fn is_reserved_windows_segment(segment: &str) -> bool {
    let stem = segment
        .split('.')
        .next()
        .unwrap_or(segment)
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

fn portable_case_key(value: &str) -> String {
    value.to_lowercase()
}

fn add_directory_and_parents(target: &mut BTreeSet<String>, directory: &str) {
    let mut current = String::new();
    for segment in directory.split('/') {
        if !current.is_empty() {
            current.push('/');
        }
        current.push_str(segment);
        target.insert(current.clone());
    }
}

fn ensure_extraction_target(root: &Path, target: &Path) -> Result<(), String> {
    if !target.starts_with(root) {
        return Err("Project package extraction attempted to escape its staging root.".to_string());
    }
    Ok(())
}

fn normalize_archive_path(value: &str, must_exist: bool) -> Result<PathBuf, String> {
    let path = normalize_local_path(value, "Project package")?;
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| !extension.eq_ignore_ascii_case("monogatari"))
        .unwrap_or(true)
    {
        return Err("Project package paths must use the `.monogatari` extension.".to_string());
    }
    if must_exist && !path.exists() {
        return Err(format!(
            "Project package does not exist: {}",
            path.display()
        ));
    }
    Ok(path)
}

fn normalize_local_path(value: &str, label: &str) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_control) || trimmed.contains("://") {
        return Err(format!("{label} must be a local filesystem path."));
    }
    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        Ok(path)
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .map_err(|error| format!("Unable to resolve {label}: {error}"))
    }
}

fn canonical_regular_archive(path: &Path) -> Result<PathBuf, String> {
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

fn ensure_replaceable_archive_target(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        format!(
            "Unable to inspect existing project package `{}`: {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "Existing project package target must be a regular file: {}",
            path.display()
        ));
    }
    Ok(())
}

fn atomic_replace_archive(stage_path: &Path, destination: &Path) -> Result<(), String> {
    let parent = destination
        .parent()
        .ok_or_else(|| "Project package destination has no parent directory.".to_string())?;
    let backup_path = unique_sibling_path(parent, ".monogatari-export", "backup")?;
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

fn unique_sibling_path(parent: &Path, prefix: &str, suffix: &str) -> Result<PathBuf, String> {
    for _ in 0..1000 {
        let counter = ARCHIVE_STAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            "{prefix}-{}-{counter}.{suffix}",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("Unable to allocate a unique project package staging path.".to_string())
}

fn available_project_directory_name(
    parent: &Path,
    project_title: &str,
    archive_path: &Path,
) -> Result<String, String> {
    let fallback = archive_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("monogatari-project");
    let base = portable_directory_slug(project_title, fallback);
    for index in 1..=999 {
        let candidate = if index == 1 {
            base.clone()
        } else {
            format!("{base}-{index}")
        };
        if !parent.join(&candidate).exists() {
            return Ok(candidate);
        }
    }
    Err("Unable to allocate an unused imported project directory name.".to_string())
}

fn portable_directory_slug(project_title: &str, fallback: &str) -> String {
    fn slug(value: &str) -> String {
        let mut result = String::new();
        let mut pending_separator = false;
        for ch in value.chars() {
            if ch.is_ascii_alphanumeric() {
                if pending_separator && !result.is_empty() {
                    result.push('-');
                }
                result.push(ch.to_ascii_lowercase());
                pending_separator = false;
            } else {
                pending_separator = true;
            }
            if result.len() >= 72 {
                break;
            }
        }
        result.trim_matches('-').to_string()
    }
    let preferred = slug(project_title);
    if !preferred.is_empty() && !is_reserved_windows_segment(&preferred) {
        return preferred;
    }
    let fallback = slug(fallback);
    if !fallback.is_empty() && !is_reserved_windows_segment(&fallback) {
        fallback
    } else {
        "monogatari-project".to_string()
    }
}

fn remove_import_staging(path: &Path) {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return;
    };
    if name.starts_with(".monogatari-import-") {
        let _ = std::fs::remove_dir_all(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_project_archive_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn create_test_project(root: &Path) {
        for directory in [
            "characters",
            "dialogue",
            "knowledge",
            "scenes",
            "assets/backgrounds",
            "events",
            "endings",
            "locales",
            "quality_suites",
            "workflows",
        ] {
            std::fs::create_dir_all(root.join(directory)).unwrap();
        }
        std::fs::write(
            root.join("settings.json"),
            serde_json::to_vec_pretty(&json!({
                "render": { "title": "Archive Test" },
                "ai": {
                    "provider": "api",
                    "api": {
                        "base_url": "https://example.test/v1",
                        "api_key": "must-not-ship",
                        "model": "test-model"
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
                    "quality_suites": "quality_suites"
                }
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("characters/guide.json"),
            br#"{"id":"guide","name":"Guide"}"#,
        )
        .unwrap();
        let streamed_asset = (0..(200 * 1024 + 7))
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>();
        std::fs::write(root.join("assets/backgrounds/test.bin"), streamed_asset).unwrap();
    }

    fn write_raw_archive(path: &Path, manifest: &Value, files: &[(&str, &[u8])]) {
        let file = File::create(path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);
        writer.start_file(ARCHIVE_MANIFEST_PATH, options).unwrap();
        writer
            .write_all(&serde_json::to_vec_pretty(manifest).unwrap())
            .unwrap();
        for (name, bytes) in files {
            writer.start_file(*name, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
    }

    #[test]
    fn project_archives_round_trip_with_sanitized_settings() {
        let root = temp_root("round_trip");
        let source = root.join("source");
        let extraction = root.join("extracted");
        let archive = root.join("project.monogatari");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(&extraction).unwrap();
        create_test_project(&source);

        let manifest = build_project_export_manifest(&source, vec![], vec![], 0, None).unwrap();
        let exported = write_project_archive(&source, &archive, manifest).unwrap();
        assert!(archive.is_file());
        assert_eq!(exported.project_title, "Archive Test");
        assert!(exported.file_count >= 3);

        let verified = verify_archive(&archive, Some(&extraction)).unwrap();
        assert!(verified.inspection.verified);
        assert_eq!(verified.inspection.content_sha256, exported.content_sha256);
        let settings: Value =
            serde_json::from_slice(&std::fs::read(extraction.join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(settings["ai"]["api"]["api_key"], "");
        assert!(!std::fs::read_to_string(extraction.join("settings.json"))
            .unwrap()
            .contains("must-not-ship"));
        let restored_asset = std::fs::read(extraction.join("assets/backgrounds/test.bin")).unwrap();
        assert_eq!(restored_asset.len(), 200 * 1024 + 7);
        assert_eq!(restored_asset[0], 0);
        assert_eq!(restored_asset[restored_asset.len() - 1], 241);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn checked_in_project_packages_reload_as_runtime_content() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
        let root = temp_root("checked_in");
        let extraction = root.join("extracted");
        let archive = root.join("checked-in.monogatari");
        std::fs::create_dir_all(&extraction).unwrap();

        let manifest = build_project_export_manifest(&fixture, vec![], vec![], 0, None).unwrap();
        let exported = write_project_archive(&fixture, &archive, manifest).unwrap();
        let verified = verify_archive(&archive, Some(&extraction)).unwrap();
        assert_eq!(verified.inspection.file_count, exported.file_count);

        let (characters, dialogues, knowledge, events) =
            super::super::engine::load_project_content(&extraction)
                .await
                .unwrap();
        assert!(!characters.character_ids().is_empty());
        assert!(!dialogues.script_ids().is_empty());
        assert!(!knowledge.is_empty());
        assert!(!events.snapshot().catalog_fingerprint.is_empty());
        super::super::scenes::build_scene_asset_catalog(&extraction).unwrap();
        super::super::endings::load_story_endings(&extraction).unwrap();

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archive_manifest_rejects_traversal_and_portable_collisions() {
        let settings = br#"{"render":{"title":"Unsafe"}}"#;
        let settings_sha = sha256_hex(settings);
        let base = json!({
            "format": ARCHIVE_FORMAT,
            "schema": ARCHIVE_SCHEMA,
            "version": "1.0",
            "settings": { "render": { "title": "Unsafe" } },
            "package": {
                "file_count": 1,
                "total_bytes": settings.len(),
                "fingerprint_algorithm": PACKAGE_FINGERPRINT_ALGORITHM,
                "content_sha256": "0".repeat(64),
                "directories": ["characters"],
                "files": [{
                    "category": "settings",
                    "path": "../settings.json",
                    "size_bytes": settings.len(),
                    "checksum_sha256": settings_sha
                }]
            }
        });
        assert!(validate_manifest(base)
            .unwrap_err()
            .contains("unsafe path segment"));

        let collision = json!({
            "format": ARCHIVE_FORMAT,
            "schema": ARCHIVE_SCHEMA,
            "version": "1.0",
            "settings": { "render": { "title": "Unsafe" } },
            "package": {
                "file_count": 2,
                "total_bytes": 0,
                "fingerprint_algorithm": PACKAGE_FINGERPRINT_ALGORITHM,
                "content_sha256": "0".repeat(64),
                "directories": ["characters"],
                "files": [
                    { "category": "characters", "path": "characters/A.json", "size_bytes": 0, "checksum_sha256": sha256_hex(b"") },
                    { "category": "characters", "path": "characters/a.json", "size_bytes": 0, "checksum_sha256": sha256_hex(b"") }
                ]
            }
        });
        assert!(validate_manifest(collision)
            .unwrap_err()
            .contains("duplicate portable file path"));
    }

    #[test]
    fn archive_manifest_rejects_file_directory_topology_conflicts() {
        let record = |path: &str| ArchiveFileRecord {
            category: "assets".to_string(),
            path: path.to_string(),
            size_bytes: 0,
            checksum_md5: String::new(),
            checksum_sha256: sha256_hex(b""),
        };

        let exact_conflict = BTreeMap::from([
            ("settings.json".to_string(), record("settings.json")),
            ("assets/portraits".to_string(), record("assets/portraits")),
        ]);
        let directories = HashSet::from([
            portable_case_key("assets"),
            portable_case_key("assets/portraits"),
        ]);
        let error = validate_package_path_topology(&exact_conflict, &directories).unwrap_err();
        assert!(error.contains("both a file and a directory"), "{error}");

        let ancestor_conflict = BTreeMap::from([
            ("settings.json".to_string(), record("settings.json")),
            ("assets/portraits".to_string(), record("assets/portraits")),
            (
                "assets/portraits/guide.png".to_string(),
                record("assets/portraits/guide.png"),
            ),
        ]);
        let directories = HashSet::from([portable_case_key("assets")]);
        let error = validate_package_path_topology(&ancestor_conflict, &directories).unwrap_err();
        assert!(error.contains("cannot contain descendant"), "{error}");
    }

    #[test]
    fn archive_verification_rejects_tampered_content() {
        let root = temp_root("tampered");
        std::fs::create_dir_all(&root).unwrap();
        let archive = root.join("tampered.monogatari");
        let settings = br#"{"render":{"title":"Tampered"}}"#;
        let record = ArchiveFileRecord {
            category: "settings".to_string(),
            path: "settings.json".to_string(),
            size_bytes: settings.len() as u64,
            checksum_md5: format!("{:x}", md5::compute(settings)),
            checksum_sha256: sha256_hex(settings),
        };
        let fingerprint = package_fingerprint(std::iter::once(&record));
        let manifest = json!({
            "format": ARCHIVE_FORMAT,
            "schema": ARCHIVE_SCHEMA,
            "version": "1.0",
            "settings": { "render": { "title": "Tampered" } },
            "package": {
                "file_count": 1,
                "total_bytes": settings.len(),
                "fingerprint_algorithm": PACKAGE_FINGERPRINT_ALGORITHM,
                "content_sha256": fingerprint,
                "directories": [],
                "files": [{
                    "category": "settings",
                    "path": "settings.json",
                    "size_bytes": settings.len(),
                    "checksum_md5": record.checksum_md5,
                    "checksum_sha256": record.checksum_sha256
                }]
            }
        });
        let mut changed = settings.to_vec();
        let changed_index = changed.len() - 2;
        changed[changed_index] = b'X';
        write_raw_archive(&archive, &manifest, &[("settings.json", &changed)]);

        let error = verify_archive(&archive, None).unwrap_err();
        assert!(error.contains("SHA-256 mismatch"), "{error}");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_archive_exports_preserve_existing_packages() {
        let root = temp_root("export_rollback");
        let source = root.join("source");
        let archive = root.join("project.monogatari");
        std::fs::create_dir_all(&source).unwrap();
        create_test_project(&source);
        std::fs::write(&archive, b"previous package").unwrap();

        let manifest = build_project_export_manifest(&source, vec![], vec![], 0, None).unwrap();
        std::fs::write(
            source.join("characters/guide.json"),
            br#"{"id":"changed","name":"Changed"}"#,
        )
        .unwrap();
        let error = write_project_archive(&source, &archive, manifest).unwrap_err();

        assert!(error.contains("changed"), "{error}");
        assert_eq!(std::fs::read(&archive).unwrap(), b"previous package");
        assert!(!std::fs::read_dir(&root)
            .unwrap()
            .flatten()
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with(".monogatari-export-")));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archive_manifest_rejects_declared_size_bombs_without_allocating() {
        let manifest = json!({
            "format": ARCHIVE_FORMAT,
            "schema": ARCHIVE_SCHEMA,
            "version": "1.0",
            "settings": {},
            "package": {
                "file_count": 1,
                "total_bytes": MAX_ARCHIVE_FILE_BYTES + 1,
                "fingerprint_algorithm": PACKAGE_FINGERPRINT_ALGORITHM,
                "content_sha256": "0".repeat(64),
                "directories": [],
                "files": [{
                    "category": "settings",
                    "path": "settings.json",
                    "size_bytes": MAX_ARCHIVE_FILE_BYTES + 1,
                    "checksum_sha256": sha256_hex(b"")
                }]
            }
        });
        assert!(validate_manifest(manifest)
            .unwrap_err()
            .contains("per-file limit"));
    }

    #[test]
    fn imported_project_directory_names_are_stable_and_non_destructive() {
        let root = temp_root("names");
        std::fs::create_dir_all(root.join("archive-test")).unwrap();
        let first = available_project_directory_name(
            &root,
            "Archive Test",
            &root.join("fallback.monogatari"),
        )
        .unwrap();
        assert_eq!(first, "archive-test-2");
        assert_eq!(
            portable_directory_slug("CON", "Fallback Name"),
            "fallback-name"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
