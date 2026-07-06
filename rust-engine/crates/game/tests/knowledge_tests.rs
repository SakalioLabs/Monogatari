//! Tests for the knowledge base system.

use llm_game::knowledge::{KnowledgeBase, KnowledgeEntry, KnowledgeCategory};

#[test]
fn test_knowledge_base_creation() {
    let kb = KnowledgeBase::new();
    assert!(kb.is_empty());
    assert_eq!(kb.len(), 0);
}

#[test]
fn test_knowledge_base_add_entry() {
    let mut kb = KnowledgeBase::new();
    let entry = KnowledgeEntry::new(
        "test1",
        KnowledgeCategory::Location,
        "Test Location",
        "A beautiful test location.",
    );
    kb.add_entry(entry);
    
    assert_eq!(kb.len(), 1);
    assert!(!kb.is_empty());
}

#[test]
fn test_knowledge_base_get_entry() {
    let mut kb = KnowledgeBase::new();
    let entry = KnowledgeEntry::new(
        "test1",
        KnowledgeCategory::Location,
        "Test Location",
        "A beautiful test location.",
    );
    kb.add_entry(entry);
    
    let retrieved = kb.get_entry("test1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().title, "Test Location");
    
    let missing = kb.get_entry("nonexistent");
    assert!(missing.is_none());
}

#[test]
fn test_knowledge_base_search() {
    let mut kb = KnowledgeBase::new();
    
    let entry1 = KnowledgeEntry::new(
        "loc1",
        KnowledgeCategory::Location,
        "Cherry Blossom Park",
        "A beautiful park with cherry blossom trees.",
    );
    let entry2 = KnowledgeEntry::new(
        "char1",
        KnowledgeCategory::Character,
        "Sakura",
        "A cheerful girl who loves cherry blossoms.",
    );
    let entry3 = KnowledgeEntry::new(
        "item1",
        KnowledgeCategory::Item,
        "Magic Sword",
        "A powerful magical sword.",
    );
    
    kb.add_entry(entry1);
    kb.add_entry(entry2);
    kb.add_entry(entry3);
    
    // Search for "cherry" should find 2 entries
    let results = kb.search("cherry", 10);
    assert_eq!(results.len(), 2);
    
    // Search for "sword" should find 1 entry
    let results = kb.search("sword", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Magic Sword");
    
    // Search with limit
    let results = kb.search("cherry", 1);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_knowledge_base_get_by_category() {
    let mut kb = KnowledgeBase::new();
    
    kb.add_entry(KnowledgeEntry::new(
        "loc1",
        KnowledgeCategory::Location,
        "Park",
        "A park.",
    ));
    kb.add_entry(KnowledgeEntry::new(
        "char1",
        KnowledgeCategory::Character,
        "Sakura",
        "A girl.",
    ));
    kb.add_entry(KnowledgeEntry::new(
        "loc2",
        KnowledgeCategory::Location,
        "School",
        "A school.",
    ));
    
    let locations = kb.get_by_category(&KnowledgeCategory::Location);
    assert_eq!(locations.len(), 2);
    
    let characters = kb.get_by_category(&KnowledgeCategory::Character);
    assert_eq!(characters.len(), 1);
}

#[test]
fn test_knowledge_base_get_by_tag() {
    let mut kb = KnowledgeBase::new();
    
    let mut entry1 = KnowledgeEntry::new(
        "loc1",
        KnowledgeCategory::Location,
        "Park",
        "A park.",
    );
    entry1.tags = vec!["nature".to_string(), "peaceful".to_string()];
    
    let mut entry2 = KnowledgeEntry::new(
        "char1",
        KnowledgeCategory::Character,
        "Sakura",
        "A girl.",
    );
    entry2.tags = vec!["friendly".to_string(), "nature".to_string()];
    
    kb.add_entry(entry1);
    kb.add_entry(entry2);
    
    let nature_entries = kb.get_by_tag("nature");
    assert_eq!(nature_entries.len(), 2);
    
    let peaceful_entries = kb.get_by_tag("peaceful");
    assert_eq!(peaceful_entries.len(), 1);
}

#[test]
fn test_knowledge_entry_importance() {
    let mut kb = KnowledgeBase::new();
    
    let mut entry1 = KnowledgeEntry::new(
        "low",
        KnowledgeCategory::Lore,
        "Low Importance",
        "Not very important.",
    );
    entry1.importance = 0.2;
    
    let mut entry2 = KnowledgeEntry::new(
        "high",
        KnowledgeCategory::Lore,
        "High Importance",
        "Very important!",
    );
    entry2.importance = 0.9;
    
    kb.add_entry(entry1);
    kb.add_entry(entry2);
    
    // Search should rank higher importance first
    let results = kb.search("importance", 10);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].title, "High Importance");
}

#[test]
fn test_knowledge_base_empty_search() {
    let mut kb = KnowledgeBase::new();
    kb.add_entry(KnowledgeEntry::new(
        "test",
        KnowledgeCategory::Other("test".to_string()),
        "Test",
        "Test content.",
    ));
    
    // Empty query should return no results
    let results = kb.search("", 10);
    assert!(results.is_empty());
}
