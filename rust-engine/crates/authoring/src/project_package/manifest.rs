//! Pure project-package manifest parsing and semantic validation.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::portable_path::{add_directory_and_parents, portable_case_key, validate_portable_path};
use super::ARCHIVE_MANIFEST_PATH;

pub const ARCHIVE_FORMAT: &str = "monogatari-project";
pub const ARCHIVE_SCHEMA: &str = "monogatari-project-export@1";
pub const PACKAGE_FINGERPRINT_ALGORITHM: &str = "sha256:path-size-file-sha256-v1";
const MAX_ARCHIVE_FILES: usize = 20_000;
const MAX_ARCHIVE_DIRECTORIES: usize = 4_000;
const MAX_ARCHIVE_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;
pub const MAX_ARCHIVE_FILE_BYTES: u64 = 4 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveManifest {
    format: String,
    schema: String,
    version: String,
    #[serde(default)]
    pub export_metadata: ArchiveExportMetadata,
    pub settings: Value,
    pub package: ArchivePackage,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArchiveExportMetadata {
    #[serde(default)]
    pub engine_version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchivePackage {
    pub file_count: usize,
    pub total_bytes: u64,
    fingerprint_algorithm: String,
    pub content_sha256: String,
    #[serde(default)]
    pub directories: Vec<String>,
    files: Vec<ArchiveFileRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveFileRecord {
    pub category: String,
    pub path: String,
    pub size_bytes: u64,
    #[serde(default)]
    pub checksum_md5: String,
    pub checksum_sha256: String,
}

#[derive(Debug, Clone)]
pub struct ValidatedManifest {
    pub raw: Value,
    pub parsed: ArchiveManifest,
    pub files_by_path: BTreeMap<String, ArchiveFileRecord>,
    pub allowed_directories: BTreeSet<String>,
    pub project_title: String,
}

pub fn validate_manifest(raw: Value) -> Result<ValidatedManifest, String> {
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

pub fn validate_package_path_topology(
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

pub fn package_fingerprint<'a>(records: impl Iterator<Item = &'a ArchiveFileRecord>) -> String {
    let mut hasher = Sha256::new();
    for record in records {
        hasher.update(record.path.as_bytes());
        hasher.update(b"\0");
        hasher.update(record.size_bytes.to_string().as_bytes());
        hasher.update(b"\0");
        hasher.update(record.checksum_sha256.as_bytes());
        hasher.update(b"\n");
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn minimal_manifest() -> Value {
        let record = ArchiveFileRecord {
            category: "settings".to_string(),
            path: "settings.json".to_string(),
            size_bytes: 0,
            checksum_md5: String::new(),
            checksum_sha256: format!("{:x}", Sha256::digest([])),
        };
        let fingerprint = package_fingerprint(std::iter::once(&record));
        json!({
            "format": ARCHIVE_FORMAT,
            "schema": ARCHIVE_SCHEMA,
            "version": "1.0",
            "settings": { "render": { "title": "Manifest Unit" } },
            "package": {
                "file_count": 1,
                "total_bytes": 0,
                "fingerprint_algorithm": PACKAGE_FINGERPRINT_ALGORITHM,
                "content_sha256": fingerprint,
                "directories": [],
                "files": [{
                    "category": record.category,
                    "path": record.path,
                    "size_bytes": record.size_bytes,
                    "checksum_sha256": record.checksum_sha256
                }]
            }
        })
    }

    #[test]
    fn minimal_manifest_validates_without_zip_io() {
        let validated = validate_manifest(minimal_manifest()).unwrap();
        assert_eq!(validated.project_title, "Manifest Unit");
        assert_eq!(validated.files_by_path.len(), 1);
    }

    #[test]
    fn manifest_rejects_invalid_categories_without_zip_io() {
        let mut manifest = minimal_manifest();
        manifest["package"]["files"][0]["category"] = json!("Invalid-Category");
        let error = validate_manifest(manifest).unwrap_err();
        assert!(error.contains("invalid category"), "{error}");
    }

    #[test]
    fn manifest_rejects_traversal_and_portable_collisions_without_zip_io() {
        let settings = br#"{"render":{"title":"Unsafe"}}"#;
        let settings_sha = format!("{:x}", Sha256::digest(settings));
        let traversal = json!({
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
        assert!(validate_manifest(traversal)
            .unwrap_err()
            .contains("unsafe path segment"));

        let empty_sha = format!("{:x}", Sha256::digest([]));
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
                    { "category": "characters", "path": "characters/A.json", "size_bytes": 0, "checksum_sha256": empty_sha },
                    { "category": "characters", "path": "characters/a.json", "size_bytes": 0, "checksum_sha256": empty_sha }
                ]
            }
        });
        assert!(validate_manifest(collision)
            .unwrap_err()
            .contains("duplicate portable file path"));
    }

    #[test]
    fn manifest_rejects_file_directory_topology_conflicts_without_zip_io() {
        let record = |path: &str| ArchiveFileRecord {
            category: "assets".to_string(),
            path: path.to_string(),
            size_bytes: 0,
            checksum_md5: String::new(),
            checksum_sha256: format!("{:x}", Sha256::digest([])),
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
    fn manifest_rejects_declared_size_bombs_without_allocating() {
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
                    "checksum_sha256": format!("{:x}", Sha256::digest([]))
                }]
            }
        });
        assert!(validate_manifest(manifest)
            .unwrap_err()
            .contains("per-file limit"));
    }
}
