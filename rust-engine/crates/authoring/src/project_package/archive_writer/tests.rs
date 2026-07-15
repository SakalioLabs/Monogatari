use std::io::Read;

use serde_json::{json, Value};
use zip::ZipArchive;

use super::*;
use crate::project_package::{
    build_project_export_manifest, ProjectExportProvenance, ProjectExportRuntimeSnapshot,
};

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "monogatari_headless_package_{label}_{}_{}",
        std::process::id(),
        PACKAGE_STAGE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

fn provenance() -> ProjectExportProvenance {
    ProjectExportProvenance {
        exported_at: "2026-07-15T00:00:00Z".to_string(),
        engine_version: "test-engine".to_string(),
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
            "render": { "title": "Headless Package" },
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

fn manifest(root: &Path) -> Value {
    build_project_export_manifest(root, ProjectExportRuntimeSnapshot::default(), provenance())
        .unwrap()
}

#[test]
fn writes_streamed_credential_free_packages_without_tauri() {
    let root = temp_root("stream");
    let source = root.join("source");
    let destination = root.join("project.monogatari");
    std::fs::create_dir_all(&source).unwrap();
    create_project(&source);

    let result = write_project_package(
        &source,
        &destination,
        manifest(&source),
        ProjectPackageTargetPolicy::CreateNew,
    )
    .unwrap();
    assert_eq!(result.project_title, "Headless Package");
    assert!(result.file_count >= 3);
    assert_eq!(result.content_sha256.len(), 64);
    assert_eq!(
        result.archive_bytes,
        std::fs::metadata(&destination).unwrap().len()
    );

    let mut archive = ZipArchive::new(File::open(&destination).unwrap()).unwrap();
    let mut settings = String::new();
    archive
        .by_name("settings.json")
        .unwrap()
        .read_to_string(&mut settings)
        .unwrap();
    assert!(!settings.contains("must-not-ship"));
    let settings: Value = serde_json::from_str(&settings).unwrap();
    assert_eq!(settings["ai"]["api"]["api_key"], "");
    let mut asset = Vec::new();
    archive
        .by_name("assets/backgrounds/test.bin")
        .unwrap()
        .read_to_end(&mut asset)
        .unwrap();
    assert_eq!(asset.len(), 200 * 1024 + 7);
    drop(archive);

    let error = write_project_package(
        &source,
        &destination,
        manifest(&source),
        ProjectPackageTargetPolicy::CreateNew,
    )
    .unwrap_err();
    assert!(error.contains("already exists"), "{error}");
    write_project_package(
        &source,
        &destination,
        manifest(&source),
        ProjectPackageTargetPolicy::ReplaceExisting,
    )
    .unwrap();

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_package_replacement_preserves_the_previous_file() {
    let root = temp_root("rollback");
    let source = root.join("source");
    let destination = root.join("project.monogatari");
    std::fs::create_dir_all(&source).unwrap();
    create_project(&source);
    std::fs::write(&destination, b"previous package").unwrap();
    let manifest = manifest(&source);
    std::fs::write(
        source.join("characters/guide.json"),
        br#"{"id":"changed","name":"Changed"}"#,
    )
    .unwrap();

    let error = write_project_package(
        &source,
        &destination,
        manifest,
        ProjectPackageTargetPolicy::ReplaceExisting,
    )
    .unwrap_err();
    assert!(error.contains("changed"), "{error}");
    assert_eq!(std::fs::read(&destination).unwrap(), b"previous package");
    assert!(!std::fs::read_dir(&root).unwrap().flatten().any(|entry| {
        entry
            .file_name()
            .to_string_lossy()
            .starts_with(".monogatari-export-")
    }));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_writer_rejects_invalid_extensions_and_non_regular_targets() {
    let root = temp_root("target");
    let source = root.join("source");
    std::fs::create_dir_all(&source).unwrap();
    create_project(&source);

    let error = write_project_package(
        &source,
        &root.join("project.zip"),
        manifest(&source),
        ProjectPackageTargetPolicy::CreateNew,
    )
    .unwrap_err();
    assert!(error.contains("`.monogatari` extension"), "{error}");

    let destination = root.join("directory.monogatari");
    std::fs::create_dir(&destination).unwrap();
    let error = write_project_package(
        &source,
        &destination,
        manifest(&source),
        ProjectPackageTargetPolicy::ReplaceExisting,
    )
    .unwrap_err();
    assert!(error.contains("regular file"), "{error}");

    std::fs::remove_dir_all(root).unwrap();
}
