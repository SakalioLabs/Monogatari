use super::*;
use serde_json::json;

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

fn step(node_id: &str) -> WorkflowExecutionStep {
    WorkflowExecutionStep {
        step_index: 0,
        node_id: node_id.to_string(),
        node_type: "narration".to_string(),
        label: node_id.to_string(),
        output: Value::Null,
        next_node_id: None,
        stopped_reason: None,
    }
}

#[test]
fn normalizes_workflow_step_limits() {
    assert_eq!(workflow_step_limit(None), DEFAULT_WORKFLOW_MAX_STEPS);
    assert_eq!(workflow_step_limit(Some(0)), 1);
    assert_eq!(workflow_step_limit(Some(32)), 32);
    assert_eq!(
        workflow_step_limit(Some(WORKFLOW_MAX_STEPS_LIMIT + 1)),
        WORKFLOW_MAX_STEPS_LIMIT
    );
}

#[test]
fn parses_workflow_config_values_consistently() {
    let config = json!({
        "empty": "  ",
        "speaker": " Narrator ",
        "choices": [" First ", 2, "", "Second"],
        "duration": "1.25",
        "selected_index": "3",
        "threshold": "0.75"
    });

    assert_eq!(
        config_string(&config, &["empty", "speaker"]),
        Some("Narrator".to_string())
    );
    assert_eq!(config_string_list(&config, "choices"), ["First", "Second"]);
    assert_eq!(config_duration_ms(&config, 500), 1_250);
    assert_eq!(config_usize(&config, &["selected_index"]), Some(3));
    assert_eq!(optional_config_f32(&config, "threshold"), Some(0.75));

    let lines = json!({"choices": " Alpha\n\n Beta "});
    assert_eq!(config_string_list(&lines, "choices"), ["Alpha", "Beta"]);
    assert_eq!(config_duration_ms(&json!({"duration_ms": "42"}), 500), 42);
    assert_eq!(config_duration_ms(&json!({"duration": -1}), 500), 0);
    assert_eq!(config_duration_ms(&json!({}), 500), 500);
}

#[test]
fn normalizes_workflow_metrics_and_reads_scores() {
    let evaluation = ConversationEvaluation {
        friendliness: 0.2,
        engagement: 0.4,
        creativity: 0.6,
        overall_score: 0.8,
        summary: "test".to_string(),
    };

    assert_eq!(
        workflow_score_metric(&json!({"criteria": "overall_score"})),
        "overall"
    );
    assert_eq!(
        workflow_score_metric(&json!({"metric": " Friendly "})),
        "friendliness"
    );
    assert_eq!(workflow_score_metric(&json!({})), "overall");
    assert_eq!(
        workflow_metric_score(&evaluation, "friendliness"),
        Some(0.2)
    );
    assert_eq!(workflow_metric_score(&evaluation, "engagement"), Some(0.4));
    assert_eq!(workflow_metric_score(&evaluation, "creativity"), Some(0.6));
    assert_eq!(workflow_metric_score(&evaluation, "overall"), Some(0.8));
    assert_eq!(workflow_metric_score(&evaluation, "unknown"), None);
}

#[test]
fn normalizes_and_selects_weighted_branches() {
    assert!(workflow_branch_weights(&json!({}), 0).is_empty());
    assert_eq!(
        workflow_branch_weights(&json!({"weights": [-1, 0, "0.25"]}), 4),
        vec![0.0, 0.0, 0.25, 1.0]
    );
    assert_eq!(
        workflow_branch_weights(&json!({"weights": [-1, 0, -0.5]}), 3),
        vec![1.0, 1.0, 1.0]
    );
    assert_eq!(
        workflow_branch_weights(&json!({"weights": "2\nbad\n-1"}), 4),
        vec![2.0, 1.0, 0.0, 1.0]
    );

    let weights = [1.0, 3.0];
    assert_eq!(select_weighted_branch(&weights, 0.0), 0);
    assert_eq!(select_weighted_branch(&weights, 0.249), 0);
    assert_eq!(select_weighted_branch(&weights, 0.25), 1);
    assert_eq!(select_weighted_branch(&weights, 0.999), 1);
    assert_eq!(select_weighted_branch(&[], 0.5), 0);
}

#[test]
fn resolves_node_transitions_for_every_branching_contract() {
    let selections = HashMap::from([("choice".to_string(), 1usize)]);
    let choice = node("choice", "choice", &["left", "right"], json!({}));
    assert_eq!(
        workflow_next_node(&choice, &json!({}), &selections),
        (Some("right".to_string()), None)
    );
    assert_eq!(
        workflow_next_node(
            &node(
                "fallback",
                "choice",
                &["left", "right"],
                json!({"default_choice_index": 0})
            ),
            &json!({}),
            &HashMap::new()
        ),
        (Some("left".to_string()), None)
    );
    assert_eq!(
        workflow_next_node(&choice, &json!({}), &HashMap::new()),
        (None, Some("awaiting_choice".to_string()))
    );
    assert_eq!(
        workflow_next_node(
            &node("choice", "choice", &["left"], json!({"selected_index": 2})),
            &json!({}),
            &HashMap::new()
        ),
        (None, Some("choice_index_out_of_range".to_string()))
    );
    assert_eq!(
        workflow_next_node(
            &node("empty", "choice", &[], json!({})),
            &json!({}),
            &HashMap::new()
        ),
        (None, Some("choice_has_no_connections".to_string()))
    );

    let condition = node("condition", "condition", &["yes", "no"], json!({}));
    assert_eq!(
        workflow_next_node(&condition, &json!({"result": true}), &HashMap::new()),
        (Some("yes".to_string()), None)
    );
    assert_eq!(
        workflow_next_node(&condition, &json!({"result": false}), &HashMap::new()),
        (Some("no".to_string()), None)
    );
    assert_eq!(
        workflow_next_node(&condition, &json!({}), &HashMap::new()),
        (Some("yes".to_string()), None)
    );
    assert_eq!(
        workflow_next_node(
            &node("condition", "condition", &[], json!({})),
            &json!({}),
            &HashMap::new()
        ),
        (None, Some("condition_result_missing".to_string()))
    );
    assert_eq!(
        workflow_next_node(
            &node("condition", "condition", &[], json!({})),
            &json!({"result": true}),
            &HashMap::new()
        ),
        (None, Some("true_branch_missing".to_string()))
    );
    assert_eq!(
        workflow_next_node(
            &node("condition", "condition", &["yes"], json!({})),
            &json!({"result": false}),
            &HashMap::new()
        ),
        (None, Some("false_branch_missing".to_string()))
    );

    assert_eq!(
        workflow_next_node(
            &node("evaluation", "evaluation", &["pass", "fail"], json!({})),
            &json!({"passed": false}),
            &HashMap::new()
        ),
        (Some("fail".to_string()), None)
    );
    assert_eq!(
        workflow_next_node(
            &node("evaluation", "evaluation", &[], json!({})),
            &json!({}),
            &HashMap::new()
        ),
        (None, Some("evaluation_threshold_missing".to_string()))
    );
    assert_eq!(
        workflow_next_node(
            &node("event", "trigger_event", &["yes", "no"], json!({})),
            &json!({"triggered": true}),
            &HashMap::new()
        ),
        (Some("yes".to_string()), None)
    );
    assert_eq!(
        workflow_next_node(
            &node("event", "trigger_event", &[], json!({})),
            &json!({}),
            &HashMap::new()
        ),
        (None, Some("event_trigger_result_missing".to_string()))
    );

    assert_eq!(
        workflow_next_node(
            &node("random", "random_branch", &["left"], json!({})),
            &json!({"chosen_connection": "left"}),
            &HashMap::new()
        ),
        (Some("left".to_string()), None)
    );
    assert_eq!(
        workflow_next_node(
            &node("random", "random_branch", &["left"], json!({})),
            &json!({}),
            &HashMap::new()
        ),
        (None, Some("random_branch_has_no_choice".to_string()))
    );
    assert_eq!(
        workflow_next_node(
            &node("end", "end", &[], json!({})),
            &json!({}),
            &HashMap::new()
        ),
        (None, Some("completed".to_string()))
    );
    assert_eq!(
        workflow_next_node(
            &node("text", "narration", &[], json!({})),
            &json!({}),
            &HashMap::new()
        ),
        (None, Some("no_next_node".to_string()))
    );
    assert_eq!(
        workflow_next_node(
            &node("text", "narration", &["end"], json!({})),
            &json!({}),
            &HashMap::new()
        ),
        (Some("end".to_string()), None)
    );
}

#[test]
fn reports_unique_execution_coverage_in_trace_order() {
    let nodes = vec![
        node("a", "start", &["b"], json!({})),
        node("b", "narration", &["c"], json!({})),
        node("c", "end", &[], json!({})),
    ];
    let steps = vec![step("b"), step("b"), step("a")];

    let coverage = workflow_execution_coverage(&nodes, &steps);

    assert_eq!(coverage.node_count, 3);
    assert_eq!(coverage.executed_node_count, 2);
    assert!((coverage.coverage_percent - (200.0 / 3.0)).abs() < 0.0001);
    assert_eq!(coverage.executed_node_ids, ["b", "a"]);
    assert_eq!(coverage.unvisited_node_ids, ["c"]);
}
