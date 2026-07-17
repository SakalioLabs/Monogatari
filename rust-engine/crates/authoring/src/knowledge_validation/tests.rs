use llm_game::knowledge::{KnowledgeCategory, KnowledgeEntry};
use serde_json::json;

use super::*;

fn entry(id: &str) -> KnowledgeEntry {
    KnowledgeEntry::new(
        id,
        KnowledgeCategory::Other("world_lore".to_string()),
        format!("Title {id}"),
        format!("Content for {id}."),
    )
}

#[test]
fn normalization_preserves_custom_categories_and_canonicalizes_lists() {
    let source: KnowledgeEntry = serde_json::from_value(json!({
        "id": "  world  ",
        "category": " World_Lore ",
        "title": "  World title  ",
        "content": "  World content  ",
        "tags": [" Lore ", "lore", " place "],
        "relatedEntries": [" place ", "place", ""]
    }))
    .unwrap();

    let normalized = normalize_knowledge_entry(source);

    assert_eq!(normalized.id, "world");
    assert_eq!(normalized.category.as_str(), "world_lore");
    assert_eq!(normalized.title, "World title");
    assert_eq!(normalized.content, "World content");
    assert_eq!(normalized.tags, ["Lore", "place"]);
    assert_eq!(normalized.related_entries, ["place"]);
}

#[test]
fn valid_catalog_accepts_bounded_bidirectional_relations() {
    let mut world = entry("world");
    world.related_entries = vec!["place".to_string()];
    let mut place = entry("place");
    place.related_entries = vec!["world".to_string()];

    let result = validate_knowledge_catalog(&[world, place]);

    assert!(result.valid, "{:?}", result.issues);
    assert_eq!(result.error_count, 0);
}

#[test]
fn validation_reports_authoring_rules_beyond_runtime_deserialization() {
    let mut invalid = entry("invalid");
    invalid.title = "  ".to_string();
    invalid.importance = 2.0;
    invalid.tags = vec!["tag".repeat(65)];
    invalid.related_entries = vec!["missing".to_string()];

    let result = validate_knowledge_catalog(&[invalid]);
    let codes = result
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<HashSet<_>>();

    assert!(!result.valid);
    for code in [
        "knowledge_not_canonical",
        "knowledge_title_invalid",
        "knowledge_importance_invalid",
        "knowledge_tag_invalid",
        "knowledge_relation_target_missing",
    ] {
        assert!(codes.contains(code), "missing {code}: {:?}", result.issues);
    }
}

#[test]
fn validation_reports_duplicate_self_and_missing_relations() {
    let mut first = entry("same");
    first.related_entries = vec!["same".to_string(), "missing".to_string()];
    let second = entry("same");

    let result = validate_knowledge_catalog(&[first, second]);
    let codes = result
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>();

    assert!(codes.contains(&"duplicate_knowledge_id"));
    assert!(codes.contains(&"knowledge_relation_self"));
    assert!(codes.contains(&"knowledge_relation_target_missing"));
}

#[test]
fn entry_validation_uses_the_caller_supplied_catalog_closure() {
    let mut world = entry("world");
    world.related_entries = vec!["place".to_string()];
    let valid = validate_knowledge_entry(
        &world,
        &HashSet::from(["world".to_string(), "place".to_string()]),
    );
    let invalid = validate_knowledge_entry(&world, &HashSet::from(["world".to_string()]));

    assert!(valid.valid);
    assert_eq!(invalid.error_count, 1);
    assert_eq!(invalid.issues[0].code, "knowledge_relation_target_missing");
}

#[test]
fn validation_evidence_is_deterministic_and_bounded() {
    let entries = (0..300)
        .map(|index| {
            let mut invalid = entry(&format!("BAD_{index:03}"));
            invalid.title.clear();
            invalid.content.clear();
            invalid
        })
        .collect::<Vec<_>>();
    let mut reversed = entries.clone();
    reversed.reverse();

    let first = validate_knowledge_catalog(&entries);
    let second = validate_knowledge_catalog(&reversed);

    assert_eq!(first, second);
    assert_eq!(first.issues.len(), MAX_KNOWLEDGE_VALIDATION_ISSUES);
    assert!(first
        .issues
        .iter()
        .any(|issue| issue.code == "knowledge_validation_issue_limit_reached"));
}

#[test]
fn portable_id_validation_rejects_whitespace_and_uppercase_aliases() {
    assert!(ensure_valid_knowledge_id("world_lore-2").is_ok());
    assert!(ensure_valid_knowledge_id(" world_lore-2").is_err());
    assert!(ensure_valid_knowledge_id("World_Lore").is_err());
}
