use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use llm_authoring::agent_transaction::{
    AgentFilePrecondition, AgentProjectOperation, AgentProjectTransaction,
    AgentProjectTransactionPlan, AgentProjectTransactionResult, AgentTransactionStatus,
    AGENT_TRANSACTION_SCHEMA_V1,
};
use llm_authoring::json_catalog::{
    AuthorableJsonCatalog, JsonAcceptanceLevel, JsonCatalogDocument, JsonCatalogReport,
};
use llm_authoring::project::default_project_config;
use monogatari_mcp::protocol::{InspectProjectOutput, McpToolError, McpToolErrorCode};
use rmcp::model::{CallToolRequestParams, JsonObject};
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

struct TestProject {
    root: PathBuf,
}

impl TestProject {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "monogatari-mcp-e2e-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        for catalog in AuthorableJsonCatalog::ALL {
            std::fs::create_dir_all(root.join(catalog.as_str())).unwrap();
        }
        std::fs::write(
            root.join("settings.json"),
            serde_json::to_vec_pretty(&default_project_config()).unwrap(),
        )
        .unwrap();
        Self { root }
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[tokio::test]
async fn real_stdio_handshake_lists_and_reads_schema_backed_tools() -> anyhow::Result<()> {
    let project = TestProject::new("read");
    std::fs::write(
        project.root.join("characters/aoi.json"),
        b"{\"id\":\"aoi\",\"name\":\"Aoi\"}\n",
    )?;
    let client = connect(&project.root, false).await?;
    let second_reader = connect(&project.root, false).await?;
    assert_eq!(second_reader.list_all_tools().await?.len(), 5);
    second_reader.cancel().await?;

    let tools = client.list_all_tools().await?;
    let mut names = tools
        .iter()
        .map(|tool| tool.name.to_string())
        .collect::<Vec<_>>();
    names.sort();
    assert_eq!(
        names,
        [
            "apply_transaction",
            "inspect_project",
            "list_project_json",
            "plan_transaction",
            "read_project_json"
        ]
    );
    assert!(tools.iter().all(|tool| tool.output_schema.is_some()));

    let inspection = client
        .call_tool(CallToolRequestParams::new("inspect_project"))
        .await?;
    let inspection: InspectProjectOutput = structured(&inspection)?;
    assert!(!inspection.write_enabled);
    assert_eq!(inspection.acceptance_level, JsonAcceptanceLevel::Document);
    assert!(inspection.project.config["ai"]["api"]["api_key"] == "");

    let listing = client
        .call_tool(
            CallToolRequestParams::new("list_project_json")
                .with_arguments(arguments(json!({"catalog": "characters"}))),
        )
        .await?;
    let listing: JsonCatalogReport = structured(&listing)?;
    assert_eq!(listing.document_count, 1);
    assert_eq!(listing.documents[0].path, "characters/aoi.json");
    assert_eq!(listing.documents[0].sha256.len(), 64);

    let read = client
        .call_tool(
            CallToolRequestParams::new("read_project_json")
                .with_arguments(arguments(json!({"path": "characters/aoi.json"}))),
        )
        .await?;
    let read: JsonCatalogDocument = structured(&read)?;
    assert_eq!(read.document["name"], "Aoi");
    assert_eq!(read.metadata.content_fingerprint.len(), 64);

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn readonly_stdio_plans_but_structurally_rejects_apply() -> anyhow::Result<()> {
    let project = TestProject::new("readonly");
    let client = connect(&project.root, false).await?;
    let transaction = create_transaction("readonly_batch", "characters/emi.json");

    let plan = call_plan(&client, &transaction).await?;
    assert_eq!(plan.operation_count, 1);
    assert!(!project.root.join("characters/emi.json").exists());

    let apply = call_apply(&client, &transaction, &plan.precondition_fingerprint).await?;
    assert_eq!(apply.is_error, Some(true));
    let error: McpToolError = structured(&apply)?;
    assert_eq!(error.code, McpToolErrorCode::WriteDisabled);
    assert!(!project.root.join("characters/emi.json").exists());

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn writable_stdio_requires_reviewed_fingerprint_and_rolls_back_invalid_candidate(
) -> anyhow::Result<()> {
    let clean = TestProject::new("write");
    let client = connect(&clean.root, true).await?;
    assert_start_rejected_while_writer_holds_lease(&clean.root, false).await?;
    assert_start_rejected_while_writer_holds_lease(&clean.root, true).await?;
    let transaction = create_transaction("write_batch", "dialogue/intro.json");
    let plan = call_plan(&client, &transaction).await?;

    let mismatch = call_apply(&client, &transaction, &"0".repeat(64)).await?;
    assert_eq!(mismatch.is_error, Some(true));
    let error: McpToolError = structured(&mismatch)?;
    assert_eq!(error.code, McpToolErrorCode::PlanFingerprintMismatch);
    assert!(!clean.root.join("dialogue/intro.json").exists());

    let applied = call_apply(&client, &transaction, &plan.precondition_fingerprint).await?;
    assert_eq!(applied.is_error, Some(false));
    let applied: AgentProjectTransactionResult = structured(&applied)?;
    assert_eq!(applied.status, AgentTransactionStatus::Applied);
    assert_eq!(applied.validation["acceptance_level"], "core_runtime");
    assert_eq!(applied.validation["dialogue_count"], 1);
    assert!(clean.root.join("dialogue/intro.json").is_file());
    client.cancel().await?;

    let invalid = TestProject::new("rollback");
    std::fs::write(invalid.root.join("characters/broken.json"), b"{")?;
    let client = connect(&invalid.root, true).await?;
    let transaction = create_transaction("rollback_batch", "dialogue/rejected.json");
    let plan = call_plan(&client, &transaction).await?;
    let rejected = call_apply(&client, &transaction, &plan.precondition_fingerprint).await?;
    assert_eq!(rejected.is_error, Some(true));
    let error: McpToolError = structured(&rejected)?;
    assert_eq!(error.code, McpToolErrorCode::TransactionError);
    assert!(!invalid.root.join("dialogue/rejected.json").exists());
    assert_eq!(std::fs::read_dir(invalid.root.join("dialogue"))?.count(), 0);
    client.cancel().await?;

    let invalid_reference = TestProject::new("runtime-rollback");
    let client = connect(&invalid_reference.root, true).await?;
    let transaction = AgentProjectTransaction {
        schema: AGENT_TRANSACTION_SCHEMA_V1.to_string(),
        transaction_id: "runtime_reference_rollback".to_string(),
        operations: vec![AgentProjectOperation::PutJson {
            path: "dialogue/rejected.json".to_string(),
            document: json!({
                "id": "rejected",
                "title": "Rejected",
                "start_node_id": "start",
                "nodes": {"start": {"speaker_id": "missing", "text": "Rejected", "is_ending": true}}
            }),
            precondition: AgentFilePrecondition::Missing,
        }],
    };
    let plan = call_plan(&client, &transaction).await?;
    let rejected = call_apply(&client, &transaction, &plan.precondition_fingerprint).await?;
    assert_eq!(rejected.is_error, Some(true));
    let error: McpToolError = structured(&rejected)?;
    assert_eq!(error.code, McpToolErrorCode::TransactionError);
    assert!(error.message.contains("dialogue_speaker_missing"));
    assert!(!invalid_reference
        .root
        .join("dialogue/rejected.json")
        .exists());
    client.cancel().await?;

    let invalid_ending = TestProject::new("ending-runtime-rollback");
    let client = connect(&invalid_ending.root, true).await?;
    let transaction = AgentProjectTransaction {
        schema: AGENT_TRANSACTION_SCHEMA_V1.to_string(),
        transaction_id: "ending_reference_rollback".to_string(),
        operations: vec![AgentProjectOperation::PutJson {
            path: "endings/rejected.json".to_string(),
            document: json!({
                "schema": "monogatari-story-ending/v1",
                "id": "rejected",
                "title": "Rejected",
                "description": "Invalid references must roll back.",
                "scene_id": "missing_scene",
                "dialogue_id": "missing_dialogue"
            }),
            precondition: AgentFilePrecondition::Missing,
        }],
    };
    let plan = call_plan(&client, &transaction).await?;
    let rejected = call_apply(&client, &transaction, &plan.precondition_fingerprint).await?;
    assert_eq!(rejected.is_error, Some(true));
    let error: McpToolError = structured(&rejected)?;
    assert_eq!(error.code, McpToolErrorCode::TransactionError);
    assert!(error.message.contains("ending_scene_missing"));
    assert!(error.message.contains("ending_dialogue_missing"));
    assert!(!invalid_ending.root.join("endings/rejected.json").exists());
    client.cancel().await?;

    let invalid_event = TestProject::new("event-runtime-rollback");
    let client = connect(&invalid_event.root, true).await?;
    let transaction = AgentProjectTransaction {
        schema: AGENT_TRANSACTION_SCHEMA_V1.to_string(),
        transaction_id: "event_reference_rollback".to_string(),
        operations: vec![AgentProjectOperation::PutJson {
            path: "events/rejected.json".to_string(),
            document: json!({
                "schema": "monogatari-story-event-catalog/v1",
                "events": [{
                    "event_id": "rejected",
                    "event_type": "story",
                    "description": "Unknown content must be rejected.",
                    "actions": [{"type": "unlock_scene", "scene_id": "missing_scene"}]
                }]
            }),
            precondition: AgentFilePrecondition::Missing,
        }],
    };
    let plan = call_plan(&client, &transaction).await?;
    let rejected = call_apply(&client, &transaction, &plan.precondition_fingerprint).await?;
    assert_eq!(rejected.is_error, Some(true));
    let error: McpToolError = structured(&rejected)?;
    assert_eq!(error.code, McpToolErrorCode::TransactionError);
    assert!(error.message.contains("story_event_content_missing"));
    assert!(!invalid_event.root.join("events/rejected.json").exists());
    client.cancel().await?;

    let invalid_workflow = TestProject::new("workflow-runtime-rollback");
    let client = connect(&invalid_workflow.root, true).await?;
    let transaction = AgentProjectTransaction {
        schema: AGENT_TRANSACTION_SCHEMA_V1.to_string(),
        transaction_id: "workflow_reference_rollback".to_string(),
        operations: vec![AgentProjectOperation::PutJson {
            path: "workflows/rejected.json".to_string(),
            document: json!({
                "id": "rejected",
                "name": "Rejected",
                "start_node_id": "start",
                "nodes": [
                    {"id":"start","node_type":"start","label":"Start","x":0,"y":0,"config":{},"connections":["scene"]},
                    {"id":"scene","node_type":"scene_change","label":"Scene","x":1,"y":0,"config":{"scene_id":"missing"},"connections":["end"]},
                    {"id":"end","node_type":"end","label":"End","x":2,"y":0,"config":{},"connections":[]}
                ]
            }),
            precondition: AgentFilePrecondition::Missing,
        }],
    };
    let plan = call_plan(&client, &transaction).await?;
    let rejected = call_apply(&client, &transaction, &plan.precondition_fingerprint).await?;
    assert_eq!(rejected.is_error, Some(true));
    let error: McpToolError = structured(&rejected)?;
    assert_eq!(error.code, McpToolErrorCode::TransactionError);
    assert!(error.message.contains("workflow_scene_missing"));
    assert!(!invalid_workflow
        .root
        .join("workflows/rejected.json")
        .exists());
    client.cancel().await?;
    Ok(())
}

async fn connect(
    project_root: &Path,
    allow_write: bool,
) -> anyhow::Result<RunningService<RoleClient, ()>> {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_monogatari-mcp"));
    command.arg("--project-root").arg(project_root);
    if allow_write {
        command.arg("--allow-write");
    }
    let transport = TokioChildProcess::new(command)?;
    Ok(().serve(transport).await?)
}

async fn assert_start_rejected_while_writer_holds_lease(
    project_root: &Path,
    allow_write: bool,
) -> anyhow::Result<()> {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_monogatari-mcp"));
    command
        .arg("--project-root")
        .arg(project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if allow_write {
        command.arg("--allow-write");
    }
    let output = tokio::time::timeout(Duration::from_secs(5), command.output())
        .await
        .map_err(|_| {
            anyhow::anyhow!("competing MCP process did not exit after lock rejection")
        })??;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("already"));
    Ok(())
}

async fn call_plan(
    client: &RunningService<RoleClient, ()>,
    transaction: &AgentProjectTransaction,
) -> anyhow::Result<AgentProjectTransactionPlan> {
    let result = client
        .call_tool(
            CallToolRequestParams::new("plan_transaction")
                .with_arguments(arguments(serde_json::to_value(transaction)?)),
        )
        .await?;
    structured(&result)
}

async fn call_apply(
    client: &RunningService<RoleClient, ()>,
    transaction: &AgentProjectTransaction,
    fingerprint: &str,
) -> anyhow::Result<rmcp::model::CallToolResult> {
    Ok(client
        .call_tool(
            CallToolRequestParams::new("apply_transaction").with_arguments(arguments(json!({
                "transaction": transaction,
                "expected_precondition_fingerprint": fingerprint
            }))),
        )
        .await?)
}

fn create_transaction(id: &str, path: &str) -> AgentProjectTransaction {
    let document = if path.starts_with("dialogue/") {
        json!({
            "id": "generated",
            "title": "Generated",
            "start_node_id": "start",
            "nodes": {"start": {"text": "Generated dialogue.", "is_ending": true}}
        })
    } else {
        json!({"id": "generated", "name": "Generated"})
    };
    AgentProjectTransaction {
        schema: AGENT_TRANSACTION_SCHEMA_V1.to_string(),
        transaction_id: id.to_string(),
        operations: vec![AgentProjectOperation::PutJson {
            path: path.to_string(),
            document,
            precondition: AgentFilePrecondition::Missing,
        }],
    }
}

fn arguments(value: Value) -> JsonObject {
    value
        .as_object()
        .expect("tool arguments must be an object")
        .clone()
}

fn structured<T: DeserializeOwned>(result: &rmcp::model::CallToolResult) -> anyhow::Result<T> {
    let value = result
        .structured_content
        .clone()
        .ok_or_else(|| anyhow::anyhow!("tool result did not contain structured content"))?;
    Ok(serde_json::from_value(value)?)
}
