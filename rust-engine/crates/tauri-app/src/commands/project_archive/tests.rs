use super::*;

use llm_authoring::project_package::extract_project_package;
use serde_json::{json, Value};

use super::super::project::build_project_export_manifest;

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

    let verified = extract_project_package(&archive, &extraction).unwrap();
    assert!(verified.verified);
    assert_eq!(verified.content_sha256, exported.content_sha256);
    let settings: Value =
        serde_json::from_slice(&std::fs::read(extraction.join("settings.json")).unwrap()).unwrap();
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
    let verified = extract_project_package(&archive, &extraction).unwrap();
    assert_eq!(verified.file_count, exported.file_count);

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
fn imported_project_directory_names_are_stable_and_non_destructive() {
    let root = temp_root("names");
    std::fs::create_dir_all(root.join("archive-test")).unwrap();
    let first =
        available_project_directory_name(&root, "Archive Test", &root.join("fallback.monogatari"))
            .unwrap();
    assert_eq!(first, "archive-test-2");
    assert_eq!(
        portable_directory_slug("CON", "Fallback Name"),
        "fallback-name"
    );
    std::fs::remove_dir_all(root).unwrap();
}
