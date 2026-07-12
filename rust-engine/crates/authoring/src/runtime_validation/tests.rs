use std::sync::atomic::{AtomicU64, Ordering};

use super::*;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn project_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "monogatari_core_runtime_{label}_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(root.join("characters")).unwrap();
    std::fs::create_dir_all(root.join("dialogue")).unwrap();
    std::fs::create_dir_all(root.join("knowledge")).unwrap();
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(
        root.join("settings.json"),
        r#"{"paths":{"characters":"characters","dialogue":"dialogue","knowledge":"knowledge","assets":"assets"}}"#,
    )
    .unwrap();
    root
}

fn write_valid_core(root: &Path) {
    std::fs::write(
        root.join("knowledge/aoi_lore.json"),
        r#"{"id":"aoi_lore","title":"Aoi Lore","content":"Aoi remembers.","category":"character","tags":[],"importance":0.8}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("characters/aoi.json"),
        r#"{"id":"aoi","name":"Aoi","relationships":{"player":0.2},"knowledge_refs":["aoi_lore"]}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("dialogue/intro.json"),
        r#"{"id":"intro","title":"Intro","start_node_id":"start","nodes":{"start":{"speaker_id":"aoi","text":"Hello","is_ending":true}}}"#,
    )
    .unwrap();
}

#[tokio::test]
async fn validates_real_core_runtime_catalogs_and_references() {
    let root = project_root("valid");
    write_valid_core(&root);

    let report = validate_core_runtime_project(&root).await.unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert_eq!(report.acceptance_level, JsonAcceptanceLevel::CoreRuntime);
    assert_eq!(report.character_count, 1);
    assert_eq!(report.dialogue_count, 1);
    assert_eq!(report.dialogue_node_count, 1);
    assert_eq!(report.knowledge_count, 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn reports_cross_catalog_reference_failures_deterministically() {
    let root = project_root("references");
    write_valid_core(&root);
    std::fs::write(
        root.join("characters/aoi.json"),
        r#"{"id":"aoi","name":"Aoi","relationships":{"missing":0.2},"knowledge_refs":["missing_lore"]}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("dialogue/intro.json"),
        r#"{"id":"intro","title":"Intro","start_node_id":"start","nodes":{"start":{"speaker_id":"missing","text":"Hello","choices":[{"text":"Go","next_node_id":"end","relationship_changes":{"missing":0.2}}]},"end":{"text":"End","is_ending":true}}}"#,
    )
    .unwrap();

    let report = validate_core_runtime_project(&root).await.unwrap();
    let codes = report
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>();

    assert!(!report.valid);
    assert_eq!(
        codes,
        vec![
            "character_knowledge_target_missing",
            "character_relationship_target_missing",
            "dialogue_relationship_target_missing",
            "dialogue_speaker_missing",
        ]
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn rejects_duplicate_runtime_ids_instead_of_accepting_loader_overwrites() {
    let root = project_root("duplicates");
    write_valid_core(&root);
    std::fs::write(
        root.join("characters/duplicate.json"),
        r#"{"id":"aoi","name":"Duplicate Aoi"}"#,
    )
    .unwrap();

    let report = validate_core_runtime_project(&root).await.unwrap();

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "duplicate_character_id"));
    std::fs::remove_dir_all(root).unwrap();
}
