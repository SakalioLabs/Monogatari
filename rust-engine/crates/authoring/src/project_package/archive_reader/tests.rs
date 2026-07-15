use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::*;
use crate::project::sanitize_export_config;
use crate::project_package::{
    build_project_export_manifest, package_fingerprint, write_project_package,
    ProjectExportProvenance, ProjectExportRuntimeSnapshot, ProjectPackageTargetPolicy,
    ARCHIVE_FORMAT, ARCHIVE_SCHEMA, PACKAGE_FINGERPRINT_ALGORITHM,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "monogatari_headless_package_reader_{label}_{}_{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

fn provenance() -> ProjectExportProvenance {
    ProjectExportProvenance {
        exported_at: "2026-07-15T00:00:00Z".to_string(),
        engine_version: "reader-test-engine".to_string(),
        git_commit: "0123456789abcdef".to_string(),
        git_short_commit: "0123456".to_string(),
    }
}

fn create_project(root: &Path) {
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
            "render": { "title": "Headless Reader" },
            "ai": {
                "provider": "api",
                "api": {
                    "base_url": "https://example.test/v1",
                    "api_key": "must-not-ship",
                    "model": "test-model"
                }
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

fn minimal_manifest(settings: &Value, settings_bytes: &[u8]) -> Value {
    let record = ArchiveFileRecord {
        category: "settings".to_string(),
        path: "settings.json".to_string(),
        size_bytes: settings_bytes.len() as u64,
        checksum_md5: format!("{:x}", md5::compute(settings_bytes)),
        checksum_sha256: format!("{:x}", Sha256::digest(settings_bytes)),
    };
    let fingerprint = package_fingerprint(std::iter::once(&record));
    json!({
        "format": ARCHIVE_FORMAT,
        "schema": ARCHIVE_SCHEMA,
        "version": "1.0",
        "export_metadata": { "engine_version": "raw-test-engine" },
        "settings": sanitize_export_config(settings),
        "package": {
            "file_count": 1,
            "total_bytes": settings_bytes.len(),
            "fingerprint_algorithm": PACKAGE_FINGERPRINT_ALGORITHM,
            "content_sha256": fingerprint,
            "directories": [],
            "files": [{
                "category": record.category,
                "path": record.path,
                "size_bytes": record.size_bytes,
                "checksum_md5": record.checksum_md5,
                "checksum_sha256": record.checksum_sha256
            }]
        }
    })
}

fn write_raw_archive(path: &Path, manifest: &Value, entries: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap();
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    writer.start_file(ARCHIVE_MANIFEST_PATH, options).unwrap();
    writer
        .write_all(&serde_json::to_vec_pretty(manifest).unwrap())
        .unwrap();
    for (name, bytes) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().sync_all().unwrap();
}

#[test]
fn generated_packages_inspect_and_extract_without_tauri() {
    let root = temp_root("round_trip");
    let source = root.join("source");
    let extraction = root.join("extracted");
    let archive = root.join("project.monogatari");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir_all(&extraction).unwrap();
    create_project(&source);

    let manifest = build_project_export_manifest(
        &source,
        ProjectExportRuntimeSnapshot::default(),
        provenance(),
    )
    .unwrap();
    let exported = write_project_package(
        &source,
        &archive,
        manifest,
        ProjectPackageTargetPolicy::CreateNew,
    )
    .unwrap();
    let inspected = inspect_project_package(&archive).unwrap();
    assert!(inspected.verified);
    assert_eq!(inspected.project_title, "Headless Reader");
    assert_eq!(inspected.engine_version, "reader-test-engine");
    assert_eq!(inspected.content_sha256, exported.content_sha256);
    assert!(std::fs::read_dir(&extraction).unwrap().next().is_none());

    let extracted = extract_project_package(&archive, &extraction).unwrap();
    assert_eq!(extracted, inspected);
    let settings: Value =
        serde_json::from_slice(&std::fs::read(extraction.join("settings.json")).unwrap()).unwrap();
    assert_eq!(settings["ai"]["api"]["api_key"], "");
    assert!(!std::fs::read_to_string(extraction.join("settings.json"))
        .unwrap()
        .contains("must-not-ship"));
    let restored = std::fs::read(extraction.join("assets/backgrounds/test.bin")).unwrap();
    assert_eq!(restored.len(), 200 * 1024 + 7);
    assert_eq!(restored[restored.len() - 1], 241);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_reader_rejects_tampered_content() {
    let root = temp_root("tampered");
    std::fs::create_dir_all(&root).unwrap();
    let archive = root.join("tampered.monogatari");
    let settings = json!({ "render": { "title": "Tampered" } });
    let settings_bytes = serde_json::to_vec_pretty(&settings).unwrap();
    let manifest = minimal_manifest(&settings, &settings_bytes);
    let mut changed = settings_bytes.clone();
    let index = changed.iter().position(|byte| *byte == b'T').unwrap();
    changed[index] = b'X';
    write_raw_archive(&archive, &manifest, &[("settings.json", &changed)]);

    let error = inspect_project_package(&archive).unwrap_err();
    assert!(error.contains("SHA-256 mismatch"), "{error}");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_reader_rejects_undeclared_and_duplicate_entries() {
    let root = temp_root("entry_inventory");
    std::fs::create_dir_all(&root).unwrap();
    let settings = json!({ "render": { "title": "Inventory" } });
    let settings_bytes = serde_json::to_vec_pretty(&settings).unwrap();
    let manifest = minimal_manifest(&settings, &settings_bytes);

    let undeclared = root.join("undeclared.monogatari");
    write_raw_archive(
        &undeclared,
        &manifest,
        &[
            ("settings.json", &settings_bytes),
            ("assets/extra.bin", b"extra"),
        ],
    );
    let error = inspect_project_package(&undeclared).unwrap_err();
    assert!(error.contains("undeclared file"), "{error}");

    let duplicate = root.join("duplicate.monogatari");
    write_raw_archive(
        &duplicate,
        &manifest,
        &[
            ("settings.json", &settings_bytes),
            ("Settings.JSON", &settings_bytes),
        ],
    );
    let error = inspect_project_package(&duplicate).unwrap_err();
    assert!(error.contains("duplicate entry"), "{error}");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_reader_rejects_secret_bearing_settings_after_checksum_validation() {
    let root = temp_root("secret");
    std::fs::create_dir_all(&root).unwrap();
    let archive = root.join("secret.monogatari");
    let settings = json!({
        "render": { "title": "Secret" },
        "ai": { "api": { "api_key": "runtime-secret" } }
    });
    let settings_bytes = serde_json::to_vec_pretty(&settings).unwrap();
    let manifest = minimal_manifest(&settings, &settings_bytes);
    write_raw_archive(&archive, &manifest, &[("settings.json", &settings_bytes)]);

    let error = inspect_project_package(&archive).unwrap_err();
    assert!(error.contains("runtime secrets"), "{error}");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_extraction_requires_an_existing_empty_regular_directory() {
    let root = temp_root("extraction_root");
    std::fs::create_dir_all(&root).unwrap();
    let archive = root.join("valid.monogatari");
    let settings = json!({ "render": { "title": "Extraction" } });
    let settings_bytes = serde_json::to_vec_pretty(&settings).unwrap();
    let manifest = minimal_manifest(&settings, &settings_bytes);
    write_raw_archive(&archive, &manifest, &[("settings.json", &settings_bytes)]);

    let non_empty = root.join("non-empty");
    std::fs::create_dir(&non_empty).unwrap();
    std::fs::write(non_empty.join("existing.txt"), b"keep").unwrap();
    let error = extract_project_package(&archive, &non_empty).unwrap_err();
    assert!(error.contains("must be empty"), "{error}");
    assert_eq!(
        std::fs::read(non_empty.join("existing.txt")).unwrap(),
        b"keep"
    );

    let regular_file = root.join("not-a-directory");
    std::fs::write(&regular_file, b"keep").unwrap();
    let error = extract_project_package(&archive, &regular_file).unwrap_err();
    assert!(error.contains("regular directory"), "{error}");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_reader_rejects_invalid_extensions_and_special_zip_entries() {
    let root = temp_root("boundaries");
    std::fs::create_dir_all(&root).unwrap();
    let settings = json!({ "render": { "title": "Boundaries" } });
    let settings_bytes = serde_json::to_vec_pretty(&settings).unwrap();
    let manifest = minimal_manifest(&settings, &settings_bytes);

    let wrong_extension = root.join("project.zip");
    write_raw_archive(
        &wrong_extension,
        &manifest,
        &[("settings.json", &settings_bytes)],
    );
    let error = inspect_project_package(&wrong_extension).unwrap_err();
    assert!(error.contains("`.monogatari` extension"), "{error}");

    let symlink_entry = root.join("symlink.monogatari");
    let file = File::create(&symlink_entry).unwrap();
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    writer.start_file(ARCHIVE_MANIFEST_PATH, options).unwrap();
    writer
        .write_all(&serde_json::to_vec_pretty(&manifest).unwrap())
        .unwrap();
    writer
        .add_symlink("settings.json", "outside.json", options)
        .unwrap();
    writer.finish().unwrap();
    let error = inspect_project_package(&symlink_entry).unwrap_err();
    assert!(error.contains("not a regular file or directory"), "{error}");

    let directory_archive = root.join("directory.monogatari");
    std::fs::create_dir(&directory_archive).unwrap();
    let error = inspect_project_package(&directory_archive).unwrap_err();
    assert!(error.contains("regular file"), "{error}");
    std::fs::remove_dir_all(root).unwrap();
}
