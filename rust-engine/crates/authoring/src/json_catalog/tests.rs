//! JSON catalog path, parsing, and fingerprint regressions.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;

use super::*;

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

fn temp_project(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "monogatari-json-catalog-{label}-{}-{}",
        std::process::id(),
        NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    for catalog in AuthorableJsonCatalog::ALL {
        std::fs::create_dir_all(root.join(catalog.as_str())).unwrap();
    }
    root
}

#[test]
fn lists_and_reads_exact_and_semantic_document_hashes() {
    let root = temp_project("hashes");
    std::fs::write(
        root.join("characters/aoi.json"),
        b"{\n  \"id\": \"aoi\"\n}\n",
    )
    .unwrap();
    std::fs::write(root.join("characters/compact.json"), b"{\"id\":\"aoi\"}").unwrap();
    std::fs::write(root.join("assets/portrait.svg"), b"not json").unwrap();

    let report =
        inspect_project_json_catalog(&root, Some(AuthorableJsonCatalog::Characters)).unwrap();
    assert!(report.valid);
    assert_eq!(report.document_count, 2);

    let pretty = read_project_json(&root, "characters/aoi.json").unwrap();
    let compact = read_project_json(&root, "characters/compact.json").unwrap();
    assert_eq!(pretty.acceptance_level, JsonAcceptanceLevel::Document);
    assert_ne!(pretty.metadata.sha256, compact.metadata.sha256);
    assert_eq!(
        pretty.metadata.content_fingerprint,
        compact.metadata.content_fingerprint
    );
    assert_eq!(pretty.document["id"], "aoi");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn inspection_reports_invalid_documents_without_claiming_graph_validity() {
    let root = temp_project("invalid");
    std::fs::write(root.join("characters/broken.json"), b"{").unwrap();
    std::fs::write(root.join("dialogue/scalar.json"), b"true").unwrap();
    std::fs::write(root.join("events/upper.JSON"), b"{}").unwrap();

    let report = inspect_project_json_catalog(&root, None).unwrap();
    assert_eq!(report.acceptance_level, JsonAcceptanceLevel::Document);
    assert!(!report.valid);
    assert_eq!(report.error_count, 3);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == JsonCatalogErrorCode::InvalidJson));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == JsonCatalogErrorCode::InvalidDocument));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reads_reject_escape_restricted_and_case_mismatched_paths() {
    let root = temp_project("paths");
    std::fs::write(root.join("characters/Aoi.json"), b"{}\n").unwrap();
    for path in [
        "../characters/Aoi.json",
        "settings.json",
        "saves/slot.json",
        "characters/Aoi.JSON",
        "characters/.hidden.json",
    ] {
        assert!(read_project_json(&root, path).is_err(), "{path}");
    }
    let error = read_project_json(&root, "characters/aoi.json").unwrap_err();
    assert_eq!(error.code, JsonCatalogErrorCode::PathCaseCollision);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_catalogs_are_warnings_and_do_not_invalidate_documents() {
    let root = temp_project("missing");
    std::fs::remove_dir_all(root.join("quality_suites")).unwrap();
    std::fs::write(
        root.join("scenes/studio.json"),
        serde_json::to_vec(&json!({"id": "studio"})).unwrap(),
    )
    .unwrap();

    let report = inspect_project_json_catalog(&root, None).unwrap();
    assert!(report.valid);
    assert_eq!(report.warning_count, 1);
    assert_eq!(report.document_count, 1);
    std::fs::remove_dir_all(root).unwrap();
}
