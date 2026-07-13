use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use crate::project::default_project_config;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn project(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "monogatari_delivery_{label}_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    for directory in [
        "characters",
        "dialogue",
        "knowledge",
        "scenes",
        "endings",
        "events",
        "workflows",
        "quality_suites",
        "assets/portraits",
    ] {
        std::fs::create_dir_all(root.join(directory)).unwrap();
    }
    std::fs::write(
        root.join("settings.json"),
        serde_json::to_vec_pretty(&default_project_config()).unwrap(),
    )
    .unwrap();
    root
}

#[tokio::test]
async fn reports_existing_and_placeholder_character_assets() {
    let root = project("assets");
    std::fs::write(root.join("assets/portraits/aoi.png"), b"png").unwrap();
    std::fs::write(
        root.join("characters/aoi.json"),
        r#"{"id":"aoi","name":"Aoi","portrait_path":"assets/portraits/aoi.png"}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("characters/emi.json"),
        r#"{"id":"emi","name":"Emi"}"#,
    )
    .unwrap();

    let report = validate_project_delivery(&root).await.unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert_eq!(report.declared_renderer_asset_count, 1);
    assert_eq!(report.existing_renderer_asset_count, 1);
    assert_eq!(report.placeholder_character_count, 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn rejects_missing_declared_assets() {
    let root = project("missing");
    std::fs::write(
        root.join("characters/aoi.json"),
        r#"{"id":"aoi","name":"Aoi","portrait_path":"assets/portraits/missing.png"}"#,
    )
    .unwrap();

    let report = validate_project_delivery(&root).await.unwrap();

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "asset_missing"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn checked_in_project_has_delivery_ready_declared_assets() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");

    let report = validate_project_delivery(&root).await.unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert_eq!(
        report.declared_renderer_asset_count,
        report.existing_renderer_asset_count
    );
}
