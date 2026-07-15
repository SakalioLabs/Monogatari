use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use llm_authoring::agent_transaction::{
    AgentFilePrecondition, AgentProjectOperation, AgentProjectTransaction,
    AgentProjectTransactionPlan, AgentProjectTransactionResult, AgentTransactionStatus,
    AGENT_TRANSACTION_SCHEMA_V1,
};
use llm_authoring::delivery_validation::DeliveryValidationReport;
use llm_authoring::json_catalog::{
    AuthorableJsonCatalog, JsonAcceptanceLevel, JsonCatalogDocument, JsonCatalogReport,
};
use llm_authoring::project::default_project_config;
use llm_authoring::quality_suite_validation::MAX_QUALITY_SUITE_FILE_BYTES;
use llm_authoring::runtime_validation::CoreRuntimeValidationReport;
use monogatari_mcp::protocol::{
    ExportProjectPackageOutput, InspectProjectOutput, McpToolError, McpToolErrorCode,
    PreviewProjectPackageOutput, RunQualitySuiteOutput, MCP_PACKAGE_EXPORT_SCHEMA_V1,
    MCP_PACKAGE_PREVIEW_SCHEMA_V1, MCP_QUALITY_SUITE_RUN_SCHEMA_V1,
};
use rmcp::model::{CallToolRequestParams, JsonObject};
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

struct TestProject {
    root: PathBuf,
    package_output: PathBuf,
}

impl TestProject {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "monogatari-mcp-e2e-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        let package_output = root.with_extension("packages");
        for catalog in AuthorableJsonCatalog::ALL {
            std::fs::create_dir_all(root.join(catalog.as_str())).unwrap();
        }
        std::fs::create_dir_all(&package_output).unwrap();
        std::fs::write(
            root.join("settings.json"),
            serde_json::to_vec_pretty(&default_project_config()).unwrap(),
        )
        .unwrap();
        Self {
            root,
            package_output,
        }
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::remove_dir_all(&self.package_output);
    }
}

#[tokio::test]
async fn real_stdio_handshake_lists_and_reads_schema_backed_tools() -> anyhow::Result<()> {
    let project = TestProject::new("read");
    std::fs::write(
        project.root.join("characters/aoi.json"),
        b"{\"id\":\"aoi\",\"name\":\"Aoi\"}\n",
    )?;
    std::fs::write(
        project.root.join("quality_suites/mcp.json"),
        r#"{
            "version":"1",
            "name":"MCP Quality",
            "description":"Structured execution evidence",
            "scenarios":[
                {
                    "id":"passing",
                    "category":"quality",
                    "description":"Passing evidence",
                    "messages":[{"role":"player","content":"Hello"}],
                    "expect":{"min_overall":0.0}
                },
                {
                    "id":"failing",
                    "category":"quality",
                    "description":"Actionable failure evidence",
                    "messages":[{"role":"player","content":"Hello"}],
                    "expect":{"min_overall":1.0}
                }
            ]
        }"#,
    )?;
    let client = connect(&project.root, false).await?;
    let second_reader = connect(&project.root, false).await?;
    assert_no_project_lease_sidecar(&project);
    assert_competing_start_rejected(&project.root, true).await?;
    assert_eq!(second_reader.list_all_tools().await?.len(), 10);
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
            "export_project_package",
            "inspect_project",
            "list_project_json",
            "plan_transaction",
            "preview_project_package",
            "read_project_json",
            "run_quality_suite",
            "validate_delivery",
            "validate_project"
        ]
    );
    assert!(tools.iter().all(|tool| tool.output_schema.is_some()));

    let inspection = client
        .call_tool(CallToolRequestParams::new("inspect_project"))
        .await?;
    let inspection: InspectProjectOutput = structured(&inspection)?;
    assert!(!inspection.write_enabled);
    assert!(!inspection.package_output_configured);
    assert_eq!(inspection.acceptance_level, JsonAcceptanceLevel::Document);
    assert!(inspection.project.config["ai"]["api"]["api_key"] == "");

    let validation = client
        .call_tool(CallToolRequestParams::new("validate_project"))
        .await?;
    let validation: CoreRuntimeValidationReport = structured(&validation)?;
    assert!(validation.valid, "{:?}", validation.issues);
    assert_eq!(validation.character_count, 1);
    assert_eq!(
        validation.acceptance_level,
        JsonAcceptanceLevel::CoreRuntime
    );

    let delivery = client
        .call_tool(CallToolRequestParams::new("validate_delivery"))
        .await?;
    let delivery: DeliveryValidationReport = structured(&delivery)?;
    assert!(delivery.valid, "{:?}", delivery.issues);
    assert_eq!(delivery.placeholder_character_count, 1);

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

    let quality_listing = client
        .call_tool(
            CallToolRequestParams::new("list_project_json")
                .with_arguments(arguments(json!({"catalog": "quality_suites"}))),
        )
        .await?;
    let quality_listing: JsonCatalogReport = structured(&quality_listing)?;
    assert_eq!(quality_listing.document_count, 1);

    let quality = client
        .call_tool(
            CallToolRequestParams::new("run_quality_suite")
                .with_arguments(arguments(json!({"path": "quality_suites/mcp.json"}))),
        )
        .await?;
    assert_eq!(quality.is_error, Some(false));
    let quality: RunQualitySuiteOutput = structured(&quality)?;
    assert_eq!(quality.schema, MCP_QUALITY_SUITE_RUN_SCHEMA_V1);
    assert!(!quality.passed);
    assert_eq!(quality.report.total, 2);
    assert_eq!(quality.report.passed, 1);
    assert_eq!(quality.report.failed, 1);
    assert_eq!(
        quality.report.audit_summary.failed_scenario_ids,
        ["failing".to_string()]
    );
    assert!(quality.report.scenarios[1]
        .issues
        .iter()
        .any(|issue| issue.contains("overall expected >= 1")));
    assert_eq!(
        quality.report.run_metadata.suite_path,
        "quality_suites/mcp.json"
    );
    assert_eq!(
        quality.report.run_metadata.suite_sha256,
        quality_listing.documents[0].sha256
    );
    assert!(!quality.report.run_metadata.generated_at.is_empty());

    let wrong_catalog = client
        .call_tool(
            CallToolRequestParams::new("run_quality_suite")
                .with_arguments(arguments(json!({"path": "characters/aoi.json"}))),
        )
        .await?;
    assert_eq!(wrong_catalog.is_error, Some(true));
    let error: McpToolError = structured(&wrong_catalog)?;
    assert_eq!(error.code, McpToolErrorCode::ProjectInvalid);

    let oversized_suite = format!(
        "{{\"padding\":\"{}\"}}",
        "a".repeat(MAX_QUALITY_SUITE_FILE_BYTES as usize)
    );
    std::fs::write(
        project.root.join("quality_suites/oversized.json"),
        oversized_suite,
    )?;
    let oversized = client
        .call_tool(
            CallToolRequestParams::new("run_quality_suite")
                .with_arguments(arguments(json!({"path": "quality_suites/oversized.json"}))),
        )
        .await?;
    assert_eq!(oversized.is_error, Some(true));
    let error: McpToolError = structured(&oversized)?;
    assert_eq!(error.code, McpToolErrorCode::ProjectInvalid);
    assert!(error.message.contains("cannot exceed"));

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn package_preview_and_export_are_bound_to_reviewed_content_and_output_root(
) -> anyhow::Result<()> {
    let project = TestProject::new("package");
    let mut settings = default_project_config();
    settings["render"]["title"] = json!("MCP Package");
    settings["ai"]["api"]["api_key"] = json!("must-not-ship");
    std::fs::write(
        project.root.join("settings.json"),
        serde_json::to_vec_pretty(&settings)?,
    )?;
    std::fs::write(
        project.root.join("characters/aoi.json"),
        b"{\"id\":\"aoi\",\"name\":\"Aoi\"}\n",
    )?;

    let readonly =
        connect_with_package_output(&project.root, false, Some(&project.package_output)).await?;
    let preview = readonly
        .call_tool(CallToolRequestParams::new("preview_project_package"))
        .await?;
    assert_eq!(preview.is_error, Some(false));
    let preview: PreviewProjectPackageOutput = structured(&preview)?;
    assert_eq!(preview.schema, MCP_PACKAGE_PREVIEW_SCHEMA_V1);
    assert!(preview.package_output_configured);
    assert_eq!(preview.project_title, "MCP Package");
    assert!(preview.file_count >= 2);
    assert_eq!(preview.content_sha256.len(), 64);
    assert_eq!(
        preview.manifest["settings"]["ai"]["api"]["api_key"],
        "<redacted>"
    );

    let refused = readonly
        .call_tool(
            CallToolRequestParams::new("export_project_package").with_arguments(arguments(json!({
                "file_name": "readonly.monogatari",
                "expected_content_sha256": preview.content_sha256,
                "replace_existing": false
            }))),
        )
        .await?;
    assert_eq!(refused.is_error, Some(true));
    let error: McpToolError = structured(&refused)?;
    assert_eq!(error.code, McpToolErrorCode::WriteDisabled);
    assert!(!project.package_output.join("readonly.monogatari").exists());
    readonly.cancel().await?;

    let unconfigured = connect(&project.root, true).await?;
    let unavailable = unconfigured
        .call_tool(
            CallToolRequestParams::new("export_project_package").with_arguments(arguments(json!({
                "file_name": "unconfigured.monogatari",
                "expected_content_sha256": preview.content_sha256,
                "replace_existing": false
            }))),
        )
        .await?;
    assert_eq!(unavailable.is_error, Some(true));
    let error: McpToolError = structured(&unavailable)?;
    assert_eq!(error.code, McpToolErrorCode::PackageOutputUnavailable);
    assert!(!project
        .package_output
        .join("unconfigured.monogatari")
        .exists());
    unconfigured.cancel().await?;

    let client =
        connect_with_package_output(&project.root, true, Some(&project.package_output)).await?;
    let reviewed = client
        .call_tool(CallToolRequestParams::new("preview_project_package"))
        .await?;
    let reviewed: PreviewProjectPackageOutput = structured(&reviewed)?;

    let escape_name = format!(
        "{}-escape.monogatari",
        project.root.file_name().unwrap().to_string_lossy()
    );
    let escaped = client
        .call_tool(
            CallToolRequestParams::new("export_project_package").with_arguments(arguments(json!({
                "file_name": format!("../{escape_name}"),
                "expected_content_sha256": reviewed.content_sha256,
                "replace_existing": false
            }))),
        )
        .await?;
    assert_eq!(escaped.is_error, Some(true));
    let error: McpToolError = structured(&escaped)?;
    assert_eq!(error.code, McpToolErrorCode::PackageError);
    assert!(!project
        .package_output
        .parent()
        .unwrap()
        .join(&escape_name)
        .exists());

    std::fs::write(
        project.root.join("characters/aoi.json"),
        b"{\"id\":\"aoi\",\"name\":\"Aoi Updated\"}\n",
    )?;
    let stale = client
        .call_tool(
            CallToolRequestParams::new("export_project_package").with_arguments(arguments(json!({
                "file_name": "story.monogatari",
                "expected_content_sha256": reviewed.content_sha256,
                "replace_existing": false
            }))),
        )
        .await?;
    assert_eq!(stale.is_error, Some(true));
    let error: McpToolError = structured(&stale)?;
    assert_eq!(error.code, McpToolErrorCode::PackageFingerprintMismatch);
    assert!(!project.package_output.join("story.monogatari").exists());

    let current = client
        .call_tool(CallToolRequestParams::new("preview_project_package"))
        .await?;
    let current: PreviewProjectPackageOutput = structured(&current)?;
    assert_ne!(current.content_sha256, reviewed.content_sha256);
    let exported = client
        .call_tool(
            CallToolRequestParams::new("export_project_package").with_arguments(arguments(json!({
                "file_name": "story.monogatari",
                "expected_content_sha256": current.content_sha256,
                "replace_existing": false
            }))),
        )
        .await?;
    assert_eq!(exported.is_error, Some(false));
    let exported: ExportProjectPackageOutput = structured(&exported)?;
    assert_eq!(exported.schema, MCP_PACKAGE_EXPORT_SCHEMA_V1);
    assert_eq!(exported.package.content_sha256, current.content_sha256);
    let archive = project.package_output.join("story.monogatari");
    assert_eq!(
        PathBuf::from(&exported.package.archive_path),
        archive.canonicalize()?
    );
    assert!(std::fs::read(&archive)?.starts_with(b"PK"));

    let duplicate = client
        .call_tool(
            CallToolRequestParams::new("export_project_package").with_arguments(arguments(json!({
                "file_name": "story.monogatari",
                "expected_content_sha256": current.content_sha256,
                "replace_existing": false
            }))),
        )
        .await?;
    assert_eq!(duplicate.is_error, Some(true));
    let error: McpToolError = structured(&duplicate)?;
    assert_eq!(error.code, McpToolErrorCode::PackageError);

    let replaced = client
        .call_tool(
            CallToolRequestParams::new("export_project_package").with_arguments(arguments(json!({
                "file_name": "story.monogatari",
                "expected_content_sha256": current.content_sha256,
                "replace_existing": true
            }))),
        )
        .await?;
    assert_eq!(replaced.is_error, Some(false));
    let replaced: ExportProjectPackageOutput = structured(&replaced)?;
    assert_eq!(replaced.package.content_sha256, current.content_sha256);
    assert!(archive.is_file());

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn readonly_stdio_plans_but_structurally_rejects_apply() -> anyhow::Result<()> {
    let project = TestProject::new("readonly");
    let client = connect(&project.root, false).await?;
    assert_no_project_lease_sidecar(&project);
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
    assert_no_project_lease_sidecar(&project);
    Ok(())
}

fn assert_no_project_lease_sidecar(project: &TestProject) {
    assert!(!project.root.join(".monogatari-mcp-project.lock").exists());
}

#[tokio::test]
async fn readonly_validation_returns_structured_invalid_evidence() -> anyhow::Result<()> {
    let project = TestProject::new("validate-invalid");
    std::fs::write(
        project.root.join("quality_suites/rejected.json"),
        r#"{"version":"1","name":"Rejected","description":"Invalid refs","scenarios":[{"id":"missing","category":"story","description":"Missing character","character_id":"missing","expect":{}}]}"#,
    )?;
    let client = connect(&project.root, false).await?;

    let validation = client
        .call_tool(CallToolRequestParams::new("validate_project"))
        .await?;
    assert_eq!(validation.is_error, Some(false));
    let validation: CoreRuntimeValidationReport = structured(&validation)?;

    assert!(!validation.valid);
    assert!(validation
        .issues
        .iter()
        .any(|issue| issue.code == "quality_character_missing"));
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn readonly_delivery_validation_reports_missing_declared_assets() -> anyhow::Result<()> {
    let project = TestProject::new("delivery-invalid");
    std::fs::write(
        project.root.join("characters/aoi.json"),
        r#"{"id":"aoi","name":"Aoi","portrait_path":"assets/portraits/missing.png"}"#,
    )?;
    let client = connect(&project.root, false).await?;

    let delivery = client
        .call_tool(CallToolRequestParams::new("validate_delivery"))
        .await?;
    assert_eq!(delivery.is_error, Some(false));
    let delivery: DeliveryValidationReport = structured(&delivery)?;

    assert!(delivery.core_runtime.valid);
    assert!(!delivery.valid);
    assert!(delivery
        .issues
        .iter()
        .any(|issue| issue.code == "asset_missing"));
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn writable_stdio_requires_reviewed_fingerprint_and_rolls_back_invalid_candidate(
) -> anyhow::Result<()> {
    let clean = TestProject::new("write");
    let client = connect(&clean.root, true).await?;
    assert_competing_start_rejected(&clean.root, false).await?;
    assert_competing_start_rejected(&clean.root, true).await?;
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

    let invalid_quality = TestProject::new("quality-runtime-rollback");
    let client = connect(&invalid_quality.root, true).await?;
    let transaction = AgentProjectTransaction {
        schema: AGENT_TRANSACTION_SCHEMA_V1.to_string(),
        transaction_id: "quality_reference_rollback".to_string(),
        operations: vec![AgentProjectOperation::PutJson {
            path: "quality_suites/rejected.json".to_string(),
            document: json!({
                "version": "1",
                "name": "Rejected",
                "description": "Unknown project references must roll back.",
                "scenarios": [{
                    "id": "missing",
                    "category": "story",
                    "description": "Missing character",
                    "character_id": "missing",
                    "expect": {}
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
    assert!(error.message.contains("quality_character_missing"));
    assert!(!invalid_quality
        .root
        .join("quality_suites/rejected.json")
        .exists());
    client.cancel().await?;
    Ok(())
}

async fn connect(
    project_root: &Path,
    allow_write: bool,
) -> anyhow::Result<RunningService<RoleClient, ()>> {
    connect_with_package_output(project_root, allow_write, None).await
}

async fn connect_with_package_output(
    project_root: &Path,
    allow_write: bool,
    package_output_dir: Option<&Path>,
) -> anyhow::Result<RunningService<RoleClient, ()>> {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_monogatari-mcp"));
    command.arg("--project-root").arg(project_root);
    if let Some(output_dir) = package_output_dir {
        command.arg("--package-output-dir").arg(output_dir);
    }
    if allow_write {
        command.arg("--allow-write");
    }
    let transport = TokioChildProcess::new(command)?;
    Ok(().serve(transport).await?)
}

async fn assert_competing_start_rejected(
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
