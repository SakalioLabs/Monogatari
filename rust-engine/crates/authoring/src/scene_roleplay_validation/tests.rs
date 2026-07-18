use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::json;

use super::*;

static NEXT_TEMP_ROOT: AtomicUsize = AtomicUsize::new(0);

fn temp_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "monogatari-roleplay-validation-{}-{}",
        std::process::id(),
        NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(root.join("roleplays")).unwrap();
    root
}

fn roleplay_document() -> serde_json::Value {
    json!({
        "schema": "monogatari-scene-roleplay/v1",
        "id": "signal_room",
        "title": "Signal Room",
        "start_node_id": "contact",
        "exhaustion_ending_id": "silence",
        "max_total_turns": 4,
        "score_dimensions": [{
            "id": "trust",
            "label": "Trust",
            "description": "Respect for uncertainty.",
            "min": -2.0,
            "max": 2.0,
            "initial": 0.0
        }],
        "nodes": [{
            "id": "contact",
            "scene_id": "radio_room",
            "character_id": "echo",
            "opening_narration": "A signal answers.",
            "situation": "The source is uncertain.",
            "player_goal": "Verify the signal.",
            "character_goal": "Be heard without false certainty.",
            "knowledge_refs": ["signal_protocol"],
            "min_turns": 1,
            "max_turns": 2,
            "score_rules": [{
                "dimension_id": "trust",
                "guidance": "Reward respect for uncertainty.",
                "max_delta_per_turn": 1.0
            }],
            "evidence_rules": [],
            "transitions": [{
                "id": "trust_earned",
                "priority": 1,
                "target": { "kind": "ending", "ending_id": "truth" },
                "conditions": [{ "kind": "score_at_least", "dimension_id": "trust", "value": 1.0 }]
            }],
            "timeout_target": { "kind": "ending", "ending_id": "silence" }
        }],
        "inference": {
            "max_context_characters": 3000,
            "max_recent_turns": 3,
            "npc_max_tokens": 64,
            "evaluator_max_tokens": 96
        }
    })
}

#[test]
fn loads_validated_roleplay_documents() {
    let root = temp_root();
    std::fs::write(
        root.join("roleplays/signal_room.json"),
        serde_json::to_vec_pretty(&roleplay_document()).unwrap(),
    )
    .unwrap();

    let loaded = load_project_scene_roleplays(&root).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].definition.id, "signal_room");
    assert_eq!(loaded[0].source_path, "roleplays/signal_room.json");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reports_all_cross_catalog_reference_failures() {
    let root = temp_root();
    std::fs::write(
        root.join("roleplays/signal_room.json"),
        serde_json::to_vec(&roleplay_document()).unwrap(),
    )
    .unwrap();
    let loaded = load_project_scene_roleplays(&root).unwrap();
    let issues = validate_scene_roleplay_references(
        &loaded,
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
    );
    let codes = issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<HashSet<_>>();
    assert!(codes.contains("roleplay_scene_missing"));
    assert!(codes.contains("roleplay_character_missing"));
    assert!(codes.contains("roleplay_knowledge_missing"));
    assert!(codes.contains("roleplay_ending_missing"));
    assert!(codes.contains("roleplay_exhaustion_ending_missing"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_invalid_graph_before_reference_validation() {
    let root = temp_root();
    let mut document = roleplay_document();
    document["nodes"][0]["transitions"][0]["target"]["ending_id"] = json!(null);
    std::fs::write(
        root.join("roleplays/invalid.json"),
        serde_json::to_vec(&document).unwrap(),
    )
    .unwrap();
    assert!(load_project_scene_roleplays(&root).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
