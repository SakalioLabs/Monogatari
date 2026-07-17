use std::sync::atomic::{AtomicU64, Ordering};

use llm_game::knowledge::KnowledgeEntry;
use serde_json::json;

use super::*;

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "monogatari-knowledge-documents-{label}-{}-{}",
        std::process::id(),
        NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn create_root(label: &str) -> PathBuf {
    let root = temp_root(label);
    std::fs::create_dir_all(root.join("knowledge")).unwrap();
    root
}

#[test]
fn loader_supports_single_and_array_documents_with_legacy_relations() {
    let root = create_root("documents");
    let knowledge = root.join("knowledge");
    std::fs::write(
        knowledge.join("single.json"),
        r#"{"id":"single","category":"world_lore","title":"Single","content":"One entry","relatedEntries":["first"]}"#,
    )
    .unwrap();
    std::fs::write(
        knowledge.join("array.json"),
        r#"[{"id":"first","category":"item","title":"First","content":"First entry"},{"id":"second","category":"location","title":"Second","content":"Second entry"}]"#,
    )
    .unwrap();

    let documents = load_knowledge_documents(&root, &knowledge).unwrap();
    let snapshot = snapshot_from_documents(&documents);
    let knowledge_base = knowledge_base_from_documents(&documents);

    assert_eq!(snapshot.entries.len(), 3);
    assert_eq!(snapshot.schema, KNOWLEDGE_AUTHORING_SCHEMA_V1);
    assert!(!snapshot.catalog_fingerprint.is_empty());
    assert_eq!(
        knowledge_base.get_entry("single").unwrap().related_entries,
        ["first"]
    );
    assert_eq!(
        knowledge_base
            .get_entry("single")
            .unwrap()
            .category
            .as_str(),
        "world_lore"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn loader_rejects_unknown_fields_and_empty_documents() {
    let root = create_root("shape");
    let knowledge = root.join("knowledge");
    let path = knowledge.join("invalid.json");
    std::fs::write(
        &path,
        r#"{"id":"entry","category":"lore","title":"Entry","content":"Content","extra":true}"#,
    )
    .unwrap();

    let error = load_knowledge_documents(&root, &knowledge).unwrap_err();
    assert_eq!(error.code(), "knowledge_unknown_field");
    assert!(error.to_string().contains("extra"));

    std::fs::write(&path, "[]").unwrap();
    let error = load_knowledge_documents(&root, &knowledge).unwrap_err();
    assert_eq!(error.code(), "knowledge_document_empty");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn loader_returns_structured_authoring_validation_evidence() {
    let root = create_root("validation");
    let knowledge = root.join("knowledge");
    std::fs::write(
        knowledge.join("invalid.json"),
        r#"{"id":"entry","category":"lore","title":"  ","content":"Content","importance":2,"tags":[" lore "],"related_entries":["missing"]}"#,
    )
    .unwrap();

    let error = load_knowledge_documents(&root, &knowledge).unwrap_err();
    let codes = error
        .validation_issues()
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>();

    assert_eq!(error.code(), "knowledge_catalog_invalid");
    for code in [
        "knowledge_importance_invalid",
        "knowledge_not_canonical",
        "knowledge_relation_target_missing",
        "knowledge_title_invalid",
    ] {
        assert!(codes.contains(&code), "missing {code}: {codes:?}");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn replacement_and_removal_preserve_document_shape() {
    let root = create_root("mutation");
    let knowledge = root.join("knowledge");
    std::fs::write(
        knowledge.join("entries.json"),
        r#"[{"id":"first","category":"lore","title":"First","content":"First"},{"id":"second","category":"lore","title":"Second","content":"Second"}]"#,
    )
    .unwrap();
    let documents = load_knowledge_documents(&root, &knowledge).unwrap();
    let document = &documents[0];
    let mut replacement: KnowledgeEntry = serde_json::from_value(json!({
        "id": "first",
        "category": "lore",
        "title": "Replaced",
        "content": "First"
    }))
    .unwrap();
    replacement = normalize_knowledge_entry(replacement);

    let replaced = document.replacing_entry(0, &replacement).unwrap();
    let remaining = document.removing_entry(0).unwrap().unwrap();

    assert_eq!(replaced.as_array().unwrap().len(), 2);
    assert_eq!(replaced[0]["title"], "Replaced");
    assert_eq!(remaining.as_array().unwrap().len(), 1);
    assert_eq!(remaining[0]["id"], "second");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn loader_rejects_oversized_files_before_json_parsing() {
    let root = create_root("oversized");
    let knowledge = root.join("knowledge");
    std::fs::write(
        knowledge.join("oversized.json"),
        vec![b' '; MAX_KNOWLEDGE_FILE_BYTES as usize + 1],
    )
    .unwrap();

    let error = load_knowledge_documents(&root, &knowledge).unwrap_err();

    assert_eq!(error.code(), "knowledge_file_too_large");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_catalog_returns_a_stable_empty_snapshot() {
    let root = temp_root("missing");
    std::fs::create_dir_all(&root).unwrap();

    let documents = load_knowledge_documents(&root, &root.join("knowledge")).unwrap();
    let first = snapshot_from_documents(&documents);
    let second = snapshot_from_documents(&documents);

    assert!(first.entries.is_empty());
    assert_eq!(first.catalog_fingerprint, second.catalog_fingerprint);
    std::fs::remove_dir_all(root).unwrap();
}
