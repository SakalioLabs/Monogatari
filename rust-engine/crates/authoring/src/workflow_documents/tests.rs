use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use crate::workflow_validation::WorkflowNode;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "monogatari_workflow_documents_{label}_{}_{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

fn simple_workflow(id: &str) -> Workflow {
    Workflow {
        id: id.to_string(),
        name: format!("Workflow {id}"),
        nodes: vec![
            WorkflowNode {
                id: "start".to_string(),
                node_type: "start".to_string(),
                label: "Start".to_string(),
                x: 0.0,
                y: 0.0,
                config: serde_json::json!({}),
                connections: vec!["end".to_string()],
            },
            WorkflowNode {
                id: "end".to_string(),
                node_type: "end".to_string(),
                label: "End".to_string(),
                x: 200.0,
                y: 0.0,
                config: serde_json::json!({}),
                connections: Vec::new(),
            },
        ],
        start_node_id: "start".to_string(),
    }
}

#[test]
fn workflow_paths_preserve_compatible_project_scoping() {
    assert_eq!(
        normalize_workflow_relative_path("workflow.json").unwrap(),
        PathBuf::from("workflow.json")
    );
    assert_eq!(
        normalize_workflow_relative_path("workflows/score_gate_demo.json").unwrap(),
        PathBuf::from("score_gate_demo.json")
    );
    assert_eq!(
        normalize_workflow_relative_path("nested\\branch.JSON").unwrap(),
        PathBuf::from("nested").join("branch.JSON")
    );
}

#[test]
fn workflow_paths_reject_escape_and_unbounded_inputs() {
    let oversized_segment = format!("{}.json", "a".repeat(MAX_WORKFLOW_PATH_SEGMENT_BYTES));
    for path in [
        "",
        "../settings.json",
        "workflows/../settings.json",
        "workflows",
        "nested//branch.json",
        "nested/./branch.json",
        "C:/Users/example/workflow.json",
        "https://example.test/workflow.json",
        "workflow.txt",
        &oversized_segment,
    ] {
        assert!(
            normalize_workflow_relative_path(path).is_err(),
            "{path} should be rejected"
        );
    }
}

#[test]
fn workflow_listing_is_sorted_scoped_and_skips_invalid_files() {
    let root = temp_root("list_scope");
    let workflow_root = root.join(WORKFLOW_DIRECTORY);
    std::fs::create_dir_all(workflow_root.join("nested")).unwrap();

    std::fs::write(
        workflow_root.join("zeta.json"),
        serde_json::to_vec_pretty(&simple_workflow("wf_first")).unwrap(),
    )
    .unwrap();
    std::fs::write(
        workflow_root.join("nested").join("alpha.json"),
        serde_json::to_vec_pretty(&simple_workflow("wf_nested")).unwrap(),
    )
    .unwrap();
    std::fs::write(workflow_root.join("broken.json"), "not json").unwrap();
    std::fs::write(root.join("outside.json"), "not a workflow").unwrap();

    let summaries = list_project_workflow_summaries(&root).unwrap();
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].path, "nested/alpha.json");
    assert_eq!(summaries[0].workflow_id, "wf_nested");
    assert_eq!(summaries[0].node_count, 2);
    assert_eq!(summaries[1].path, "zeta.json");

    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn save_and_load_are_atomic_and_scoped_to_project_workflows() {
    let root = temp_root("save_load_scope");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("settings.json"), "keep me").unwrap();
    let workflow = simple_workflow("wf_test");

    save_project_workflow(&root, &workflow, "nested/test.json")
        .await
        .unwrap();
    let loaded = load_project_workflow(&root, "workflows/nested/test.json")
        .await
        .unwrap();

    assert_eq!(loaded.id, "wf_test");
    assert!(root
        .join(WORKFLOW_DIRECTORY)
        .join("nested")
        .join("test.json")
        .exists());
    assert_eq!(
        std::fs::read_to_string(root.join("settings.json")).unwrap(),
        "keep me"
    );
    assert!(save_project_workflow(&root, &workflow, "../settings.json")
        .await
        .is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn rejected_replacements_preserve_the_previous_workflow() {
    let root = temp_root("rollback");
    std::fs::create_dir_all(&root).unwrap();
    let workflow = simple_workflow("wf_stable");
    save_project_workflow(&root, &workflow, "stable.json")
        .await
        .unwrap();
    let target = root.join(WORKFLOW_DIRECTORY).join("stable.json");
    let before = std::fs::read(&target).unwrap();

    let mut invalid = simple_workflow("wf_invalid");
    invalid.start_node_id = "missing".to_string();
    assert!(save_project_workflow(&root, &invalid, "stable.json")
        .await
        .is_err());
    assert_eq!(std::fs::read(target).unwrap(), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn workflow_saves_reject_portable_case_aliases() {
    let root = temp_root("case_alias");
    std::fs::create_dir_all(&root).unwrap();
    let workflow = simple_workflow("wf_case");
    save_project_workflow(&root, &workflow, "Branch.json")
        .await
        .unwrap();

    let error = save_project_workflow(&root, &workflow, "branch.json")
        .await
        .unwrap_err();
    assert!(error.contains("collides with existing path"), "{error}");

    std::fs::create_dir(root.join(WORKFLOW_DIRECTORY).join("Chapter")).unwrap();
    let error = save_project_workflow(&root, &workflow, "chapter/nested.json")
        .await
        .unwrap_err();
    assert!(error.contains("collides with existing path"), "{error}");
    std::fs::remove_dir_all(root).unwrap();
}
