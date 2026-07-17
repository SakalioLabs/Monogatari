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
    std::fs::create_dir_all(root.join("scenes")).unwrap();
    std::fs::create_dir_all(root.join("endings")).unwrap();
    std::fs::create_dir_all(root.join("events")).unwrap();
    std::fs::create_dir_all(root.join("workflows")).unwrap();
    std::fs::create_dir_all(root.join("quality_suites")).unwrap();
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
    assert_eq!(report.scene_count, 0);
    assert_eq!(report.ending_count, 0);
    assert_eq!(report.story_event_count, 0);
    assert_eq!(report.workflow_count, 0);
    assert_eq!(report.quality_suite_count, 0);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn rejects_dialogue_authoring_limits_that_runtime_topology_accepts() {
    let root = project_root("dialogue_authoring_limits");
    write_valid_core(&root);
    std::fs::write(
        root.join("dialogue/intro.json"),
        r#"{"id":"intro","title":"Intro","start_node_id":"start","nodes":{"start":{"text":"  ","use_llm":true,"is_ending":true}}}"#,
    )
    .unwrap();

    let report = validate_core_runtime_project(&root).await.unwrap();
    let codes = report
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<HashSet<_>>();

    assert!(!report.valid);
    assert!(codes.contains("dialogue_not_canonical"));
    assert!(codes.contains("dialogue_text_invalid"));
    assert!(codes.contains("dialogue_llm_prompt_missing"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn rejects_knowledge_authoring_rules_that_runtime_deserialization_accepts() {
    let root = project_root("knowledge_authoring_limits");
    write_valid_core(&root);
    std::fs::write(
        root.join("knowledge/aoi_lore.json"),
        r#"{"id":"aoi_lore","title":"  ","content":"Aoi remembers.","category":"character","tags":[" lore "],"importance":2,"related_entries":["missing_lore"]}"#,
    )
    .unwrap();

    let report = validate_core_runtime_project(&root).await.unwrap();
    let codes = report
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<HashSet<_>>();

    assert!(!report.valid);
    for code in [
        "knowledge_importance_invalid",
        "knowledge_not_canonical",
        "knowledge_relation_target_missing",
        "knowledge_title_invalid",
    ] {
        assert!(codes.contains(code), "missing {code}: {:?}", report.issues);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn rejects_story_event_actions_with_unknown_content_targets() {
    let root = project_root("story_events");
    write_valid_core(&root);
    std::fs::write(
        root.join("events/events.json"),
        r#"{"schema":"monogatari-story-event-catalog/v1","events":[{"event_id":"unlock_missing","event_type":"story","description":"Invalid target","actions":[{"type":"unlock_scene","scene_id":"missing"}]}]}"#,
    )
    .unwrap();

    let report = validate_core_runtime_project(&root).await.unwrap();

    assert!(!report.valid);
    assert_eq!(report.story_event_count, 1);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "story_event_content_missing"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn rejects_workflow_nodes_with_unknown_scene_targets() {
    let root = project_root("workflow_references");
    write_valid_core(&root);
    std::fs::write(
        root.join("workflows/rejected.json"),
        r#"{"id":"rejected","name":"Rejected","start_node_id":"start","nodes":[{"id":"start","node_type":"start","label":"Start","x":0,"y":0,"config":{},"connections":["scene"]},{"id":"scene","node_type":"scene_change","label":"Scene","x":1,"y":0,"config":{"scene_id":"missing"},"connections":["end"]},{"id":"end","node_type":"end","label":"End","x":2,"y":0,"config":{},"connections":[]}]}"#,
    )
    .unwrap();

    let report = validate_core_runtime_project(&root).await.unwrap();

    assert_eq!(report.workflow_count, 1);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "workflow_scene_missing"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn rejects_quality_suites_with_unknown_project_references() {
    let root = project_root("quality_references");
    write_valid_core(&root);
    std::fs::write(
        root.join("quality_suites/rejected.json"),
        r#"{"version":"1","name":"Rejected","description":"Rejected suite","scenarios":[{"id":"missing","category":"story","description":"Unknown character","character_id":"missing","expect":{}}]}"#,
    )
    .unwrap();

    let report = validate_core_runtime_project(&root).await.unwrap();

    assert_eq!(report.quality_suite_count, 1);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "quality_character_missing"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn validates_scene_and_ending_references_with_the_loaded_dialogue_runtime() {
    let root = project_root("story_content");
    write_valid_core(&root);
    std::fs::write(
        root.join("scenes/finale.json"),
        r#"{"id":"finale","name":"Finale"}"#,
    )
    .unwrap();
    std::fs::write(root.join("endings/finale.json"), r#"{"schema":"monogatari-story-ending/v1","id":"finale","title":"Finale","description":"Done.","scene_id":"finale","dialogue_id":"missing"}"#).unwrap();

    let report = validate_core_runtime_project(&root).await.unwrap();

    assert_eq!(report.scene_count, 1);
    assert_eq!(report.ending_count, 1);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "ending_dialogue_missing"));
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
