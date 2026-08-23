use serde_json::json;

use super::*;

fn node(id: &str, node_type: &str, connections: &[&str]) -> WorkflowNode {
    WorkflowNode {
        id: id.to_string(),
        node_type: node_type.to_string(),
        label: id.to_string(),
        x: 0.0,
        y: 0.0,
        config: json!({}),
        connections: connections.iter().map(|value| value.to_string()).collect(),
    }
}

#[test]
fn exposes_one_authoritative_workflow_node_catalog() {
    let catalog = workflow_node_types();
    let unique_types = catalog
        .iter()
        .map(|entry| entry.node_type.as_str())
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(catalog.len(), 24);
    assert_eq!(unique_types.len(), catalog.len());
    assert!(catalog.iter().all(|entry| {
        !entry.node_type.is_empty()
            && !entry.label.is_empty()
            && !entry.description.is_empty()
            && !entry.category.is_empty()
    }));

    let dialogue = catalog
        .iter()
        .find(|entry| entry.node_type == "dialogue")
        .unwrap();
    assert_eq!(
        dialogue.configurable_fields,
        ["speaker", "text", "emotion", "use_llm"]
    );

    let llm_generate = catalog
        .iter()
        .find(|entry| entry.node_type == "llm_generate")
        .unwrap();
    assert_eq!(
        llm_generate.configurable_fields,
        ["prompt", "system_prompt", "max_tokens"]
    );
}

#[test]
fn validates_a_minimal_graph_without_tauri_state() {
    let workflow = Workflow {
        id: "minimal".into(),
        name: "Minimal".into(),
        start_node_id: "start".into(),
        nodes: vec![node("start", "start", &["end"]), node("end", "end", &[])],
    };

    let result = validate_workflow_graph(&workflow);

    assert!(result.valid, "{:?}", result.issues);
    assert_eq!(result.error_count, 0);
}

#[test]
fn rejects_connections_beyond_runtime_output_ports() {
    let workflow = Workflow {
        id: "overflow".into(),
        name: "Overflow".into(),
        start_node_id: "start".into(),
        nodes: vec![
            node("start", "start", &["first", "second"]),
            node("first", "end", &[]),
            node("second", "end", &[]),
        ],
    };

    let result = validate_workflow_graph(&workflow);

    assert!(!result.valid);
    assert!(result
        .issues
        .iter()
        .any(|issue| issue.code == "connection_output_overflow"));
}

#[test]
fn rejects_broken_links_invalid_state_keys_and_conditions() {
    let mut condition = node("condition", "condition", &["missing"]);
    condition.config = json!({"condition": "flag\u{0000}name"});
    let mut variable = node("variable", "set_variable", &[]);
    variable.config = json!({"variable_name": "../escape", "value": 1});
    let workflow = Workflow {
        id: "broken".into(),
        name: "Broken".into(),
        start_node_id: "start".into(),
        nodes: vec![node("start", "start", &["condition"]), condition, variable],
    };

    let result = validate_workflow_graph(&workflow);
    let codes = result
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>();

    assert!(!result.valid);
    assert!(codes.contains(&"connection_target_missing"));
    assert!(codes.contains(&"node_condition_invalid"));
    assert!(codes.contains(&"node_state_key_invalid"));
}

#[test]
fn resolves_trigger_nodes_against_the_shared_event_catalog() {
    let catalog = StoryEventCatalog::from_document_json(
        r#"{"schema":"monogatari-story-event-catalog/v1","events":[]}"#,
        "events/events.json",
    )
    .unwrap();
    let mut trigger = node("trigger", "trigger_event", &["end"]);
    trigger.config = json!({"event_id": "missing"});
    let workflow = Workflow {
        id: "event".into(),
        name: "Event".into(),
        start_node_id: "start".into(),
        nodes: vec![
            node("start", "start", &["trigger"]),
            trigger,
            node("end", "end", &[]),
        ],
    };

    let result = validate_workflow_with_catalog(&workflow, &catalog);

    assert!(result
        .issues
        .iter()
        .any(|issue| issue.code == "node_event_unknown"));
}

#[test]
fn resolves_story_entries_against_the_active_project_catalog() {
    let mut story_catalog = WorkflowStoryReferenceCatalog::default();
    story_catalog.register_dialogue("chapter_one", ["opening", "guild"]);
    story_catalog.register_roleplay("guild_free_talk");
    story_catalog.register_campaign("volume_one");

    let mut dialogue = node("dialogue", "dialogue_entry", &["roleplay"]);
    dialogue.config = json!({"dialogue_id":"chapter_one", "entry_node_id":"opening"});
    let mut roleplay = node("roleplay", "scene_roleplay_entry", &["campaign"]);
    roleplay.config = json!({"roleplay_id":"guild_free_talk"});
    let mut campaign = node("campaign", "roleplay_campaign_entry", &["end"]);
    campaign.config = json!({"campaign_id":"volume_one"});
    let workflow = Workflow {
        id: "story_entries".into(),
        name: "Story entries".into(),
        start_node_id: "start".into(),
        nodes: vec![
            node("start", "start", &["dialogue"]),
            dialogue,
            roleplay,
            campaign,
            node("end", "end", &[]),
        ],
    };

    let result = validate_workflow_with_project_catalog(
        &workflow,
        &StoryEventCatalog::default(),
        &story_catalog,
    );
    assert!(result.valid, "{:?}", result.issues);
    assert!(workflow_uses_story_entries(&workflow));

    let mut invalid = workflow.clone();
    invalid.nodes[1].config = json!({"dialogue_id":"chapter_one", "entry_node_id":"missing"});
    invalid.nodes[2].config = json!({"roleplay_id":"missing_roleplay"});
    invalid.nodes[3].config = json!({"campaign_id":"missing_campaign"});
    let invalid_result = validate_workflow_with_project_catalog(
        &invalid,
        &StoryEventCatalog::default(),
        &story_catalog,
    );
    let codes = invalid_result
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"node_dialogue_entry_unknown"));
    assert!(codes.contains(&"node_roleplay_unknown"));
    assert!(codes.contains(&"node_campaign_unknown"));
}
