use super::*;
use crate::story_events::StoryEventCatalog;
use crate::workflow_validation::WorkflowNode;

fn node(id: &str, node_type: &str, connections: &[&str], config: Value) -> WorkflowNode {
    WorkflowNode {
        id: id.to_string(),
        node_type: node_type.to_string(),
        label: id.to_string(),
        x: 0.0,
        y: 0.0,
        config,
        connections: connections.iter().map(|value| value.to_string()).collect(),
    }
}

fn workflow(nodes: Vec<WorkflowNode>) -> Workflow {
    Workflow {
        id: "preview".to_string(),
        name: "Preview".to_string(),
        start_node_id: "start".to_string(),
        nodes,
    }
}

fn evaluation(overall_score: f32) -> ConversationEvaluation {
    ConversationEvaluation {
        friendliness: overall_score,
        engagement: overall_score,
        creativity: overall_score,
        overall_score,
        summary: "Preview score".to_string(),
    }
}

#[test]
fn executes_context_state_and_conditions_without_tauri() {
    let workflow = workflow(vec![
        node("start", "start", &["evaluation"], json!({})),
        node(
            "evaluation",
            "evaluation",
            &["set", "low"],
            json!({"criteria":"overall", "threshold":0.7, "variable_name":"score"}),
        ),
        node(
            "set",
            "set_flag",
            &["condition"],
            json!({"flag_name":"ready", "value":true}),
        ),
        node(
            "condition",
            "condition",
            &["relationship", "low"],
            json!({"condition":"hasFlag(\"ready\") && getVariable(\"score\") >= 0.7"}),
        ),
        node(
            "relationship",
            "relationship",
            &["end"],
            json!({"character_id":"sakura", "delta":0.3}),
        ),
        node("low", "narration", &["end"], json!({"text":"low"})),
        node("end", "end", &[], json!({})),
    ]);
    let environment = WorkflowPreviewEnvironment {
        characters: HashMap::from([(
            "sakura".to_string(),
            WorkflowPreviewCharacterState {
                relationships: HashMap::from([("player".to_string(), 0.1)]),
                ..WorkflowPreviewCharacterState::default()
            },
        )]),
        ..WorkflowPreviewEnvironment::default()
    };
    let options = WorkflowPreviewOptions {
        run_context: Some(WorkflowRunContext {
            enabled: true,
            character_id: Some("sakura".to_string()),
            evaluation: Some(evaluation(0.8)),
            relationship: Some(0.2),
            evaluation_count: Some(2),
            already_triggered_events: Vec::new(),
        }),
        ..WorkflowPreviewOptions::default()
    };

    let report = execute_workflow_preview(
        &workflow,
        &StoryEventCatalog::default(),
        environment,
        options,
    )
    .unwrap();

    assert!(report.completed);
    assert_eq!(
        report.executed_node_ids,
        [
            "start",
            "evaluation",
            "set",
            "condition",
            "relationship",
            "end"
        ]
    );
    assert_eq!(report.steps[3].output["result"], true);
    assert!((report.steps[4].output["previous"].as_f64().unwrap() - 0.2).abs() < 0.0001);
    assert!((report.steps[4].output["current"].as_f64().unwrap() - 0.5).abs() < 0.0001);
}

#[test]
fn random_branches_are_deterministic_and_injectable() {
    let workflow = workflow(vec![
        node("start", "start", &["random"], json!({})),
        node(
            "random",
            "random_branch",
            &["left", "right"],
            json!({"weights":[1, 1]}),
        ),
        node("left", "end", &[], json!({})),
        node("right", "end", &[], json!({})),
    ]);
    let catalog = StoryEventCatalog::default();
    let left = execute_workflow_preview(
        &workflow,
        &catalog,
        WorkflowPreviewEnvironment::default(),
        WorkflowPreviewOptions {
            random_values: vec![0.1],
            ..WorkflowPreviewOptions::default()
        },
    )
    .unwrap();
    let right = execute_workflow_preview(
        &workflow,
        &catalog,
        WorkflowPreviewEnvironment::default(),
        WorkflowPreviewOptions {
            random_values: vec![0.9],
            ..WorkflowPreviewOptions::default()
        },
    )
    .unwrap();
    let seeded_a = execute_workflow_preview(
        &workflow,
        &catalog,
        WorkflowPreviewEnvironment::default(),
        WorkflowPreviewOptions::default(),
    )
    .unwrap();
    let seeded_b = execute_workflow_preview(
        &workflow,
        &catalog,
        WorkflowPreviewEnvironment::default(),
        WorkflowPreviewOptions::default(),
    )
    .unwrap();

    assert_eq!(left.steps[1].next_node_id.as_deref(), Some("left"));
    assert_eq!(right.steps[1].next_node_id.as_deref(), Some("right"));
    assert_eq!(seeded_a.executed_node_ids, seeded_b.executed_node_ids);
}

#[test]
fn llm_nodes_are_explicitly_simulated_without_a_provider() {
    let workflow = workflow(vec![
        node("start", "start", &["llm"], json!({})),
        node(
            "llm",
            "llm_generate",
            &["end"],
            json!({"prompt":"Continue the scene", "system_prompt":"Stay in character"}),
        ),
        node("end", "end", &[], json!({})),
    ]);

    let report = execute_workflow_preview(
        &workflow,
        &StoryEventCatalog::default(),
        WorkflowPreviewEnvironment::default(),
        WorkflowPreviewOptions::default(),
    )
    .unwrap();

    assert!(report.completed);
    assert_eq!(report.steps[1].output["simulated"], true);
    assert_eq!(report.steps[1].output["prompt"], "Continue the scene");
}

#[test]
fn event_decisions_use_typed_context_and_trigger_history() {
    let workflow = workflow(vec![
        node("start", "start", &["event"], json!({})),
        node(
            "event",
            "trigger_event",
            &["triggered", "blocked"],
            json!({"event_id":"high_engagement", "character_id":"sakura"}),
        ),
        node("triggered", "end", &[], json!({})),
        node("blocked", "end", &[], json!({})),
    ]);
    let options = WorkflowPreviewOptions {
        run_context: Some(WorkflowRunContext {
            enabled: true,
            character_id: Some("sakura".to_string()),
            evaluation: Some(ConversationEvaluation {
                friendliness: 0.5,
                engagement: 0.9,
                creativity: 0.5,
                overall_score: 0.63,
                summary: "event".to_string(),
            }),
            relationship: Some(0.0),
            evaluation_count: Some(2),
            already_triggered_events: Vec::new(),
        }),
        ..WorkflowPreviewOptions::default()
    };

    let report = execute_workflow_preview(
        &workflow,
        &StoryEventCatalog::default(),
        WorkflowPreviewEnvironment::default(),
        options,
    )
    .unwrap();

    assert_eq!(report.steps[1].output["triggered"], true);
    assert_eq!(report.steps[1].next_node_id.as_deref(), Some("triggered"));
    assert_eq!(report.steps[1].output["applied"], false);
}
