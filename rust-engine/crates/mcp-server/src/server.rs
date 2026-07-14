//! MCP tool router delegating all project semantics to `llm-authoring`.

use std::path::PathBuf;
use std::sync::Arc;

use llm_authoring::agent_transaction::{
    apply_agent_project_transaction_with_validator, plan_agent_project_transaction,
    AgentProjectTransaction, AgentProjectTransactionPlan,
};
use llm_authoring::delivery_validation::{validate_project_delivery, DeliveryValidationReport};
use llm_authoring::json_catalog::{
    inspect_project_json_catalog, read_project_json, JsonCatalogDocument, JsonCatalogReport,
};
use llm_authoring::project::{canonical_project_root, inspect_project_config, ProjectConfigState};
use llm_authoring::runtime_validation::{
    validate_core_runtime_project, CoreRuntimeValidationReport,
};
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router, Json, ServerHandler};
use tokio::sync::RwLock;

use crate::project_lease::ProjectLease;
use crate::protocol::{
    ApplyTransactionOutput, ApplyTransactionRequest, InspectProjectOutput, ListProjectJsonRequest,
    McpToolError, ReadProjectJsonRequest, MCP_INSPECTION_SCHEMA_V1,
};
use crate::validation::validate_candidate_core_runtime;

#[derive(Debug, Clone)]
pub struct MonogatariMcpServer {
    project_root: PathBuf,
    allow_write: bool,
    _project_lease: Arc<ProjectLease>,
    access: Arc<RwLock<()>>,
    tool_router: ToolRouter<Self>,
}

impl MonogatariMcpServer {
    pub fn new(project_root: PathBuf, allow_write: bool) -> Result<Self, String> {
        let project_root = canonical_project_root(&project_root)?;
        let state = inspect_project_config(&project_root)?;
        if !state.settings_exists {
            return Err("MCP project root must contain settings.json.".to_string());
        }
        let project_lease = Arc::new(ProjectLease::acquire(&project_root, allow_write)?);
        Ok(Self {
            project_root,
            allow_write,
            _project_lease: project_lease,
            access: Arc::new(RwLock::new(())),
            tool_router: Self::tool_router(),
        })
    }

    pub fn project_root(&self) -> &std::path::Path {
        &self.project_root
    }

    pub const fn write_enabled(&self) -> bool {
        self.allow_write
    }

    fn inspect_config(&self) -> Result<ProjectConfigState, McpToolError> {
        inspect_project_config(&self.project_root)
            .map_err(|message| McpToolError::project(message, None))
    }
}

#[tool_router]
impl MonogatariMcpServer {
    /// Inspect scrubbed project settings and document-level JSON readiness.
    #[tool(annotations(
        title = "Inspect project",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn inspect_project(&self) -> Result<Json<InspectProjectOutput>, Json<McpToolError>> {
        let _guard = self.access.read().await;
        let project = self.inspect_config().map_err(Json)?;
        let json_catalog = inspect_project_json_catalog(&self.project_root, None)
            .map_err(McpToolError::catalog)
            .map_err(Json)?;
        Ok(Json(InspectProjectOutput {
            schema: MCP_INSPECTION_SCHEMA_V1.to_string(),
            acceptance_level: json_catalog.acceptance_level,
            write_enabled: self.allow_write,
            project,
            json_catalog,
        }))
    }

    /// Validate the current project through every shared headless authoring gate without writing.
    #[tool(annotations(
        title = "Validate project",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn validate_project(
        &self,
    ) -> Result<Json<CoreRuntimeValidationReport>, Json<McpToolError>> {
        let _guard = self.access.read().await;
        validate_core_runtime_project(&self.project_root)
            .await
            .map(Json)
            .map_err(|message| McpToolError::project(message, None))
            .map_err(Json)
    }

    /// Validate delivery asset readiness on top of the shared headless runtime report.
    #[tool(annotations(
        title = "Validate delivery",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn validate_delivery(
        &self,
    ) -> Result<Json<DeliveryValidationReport>, Json<McpToolError>> {
        let _guard = self.access.read().await;
        validate_project_delivery(&self.project_root)
            .await
            .map(Json)
            .map_err(|message| McpToolError::project(message, None))
            .map_err(Json)
    }

    /// List safe JSON metadata and exact SHA-256 preconditions, optionally by catalog.
    #[tool(annotations(
        title = "List project JSON",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn list_project_json(
        &self,
        Parameters(request): Parameters<ListProjectJsonRequest>,
    ) -> Result<Json<JsonCatalogReport>, Json<McpToolError>> {
        let _guard = self.access.read().await;
        inspect_project_json_catalog(&self.project_root, request.catalog)
            .map(Json)
            .map_err(McpToolError::catalog)
            .map_err(Json)
    }

    /// Read one authorable JSON document with its exact and semantic fingerprints.
    #[tool(annotations(
        title = "Read project JSON",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn read_project_json(
        &self,
        Parameters(request): Parameters<ReadProjectJsonRequest>,
    ) -> Result<Json<JsonCatalogDocument>, Json<McpToolError>> {
        let _guard = self.access.read().await;
        read_project_json(&self.project_root, &request.path)
            .map(Json)
            .map_err(McpToolError::catalog)
            .map_err(Json)
    }

    /// Validate and deterministically plan an optimistic multi-file JSON transaction without writing.
    #[tool(annotations(
        title = "Plan project transaction",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn plan_transaction(
        &self,
        Parameters(transaction): Parameters<AgentProjectTransaction>,
    ) -> Result<Json<AgentProjectTransactionPlan>, Json<McpToolError>> {
        let _guard = self.access.read().await;
        plan_agent_project_transaction(&self.project_root, &transaction)
            .map(Json)
            .map_err(McpToolError::transaction)
            .map_err(Json)
    }

    /// Apply a reviewed transaction, validate the candidate project, and roll back on failure.
    #[tool(annotations(
        title = "Apply project transaction",
        read_only_hint = false,
        destructive_hint = true,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    pub async fn apply_transaction(
        &self,
        Parameters(request): Parameters<ApplyTransactionRequest>,
    ) -> Result<Json<ApplyTransactionOutput>, Json<McpToolError>> {
        if !self.allow_write {
            return Err(Json(McpToolError::write_disabled()));
        }
        let _guard = self.access.write().await;
        let plan = plan_agent_project_transaction(&self.project_root, &request.transaction)
            .map_err(McpToolError::transaction)
            .map_err(Json)?;
        if plan.precondition_fingerprint != request.expected_precondition_fingerprint {
            return Err(Json(McpToolError::fingerprint_mismatch()));
        }

        apply_agent_project_transaction_with_validator(
            &self.project_root,
            &request.transaction,
            |candidate_root| async move {
                let validation = validate_candidate_core_runtime(&candidate_root).await?;
                serde_json::to_value(validation).map_err(|_| {
                    llm_authoring::agent_transaction::AgentTransactionError::candidate_validation(
                        "Candidate validation result could not be serialized.",
                    )
                })
            },
        )
        .await
        .map(Json)
        .map_err(McpToolError::transaction)
        .map_err(Json)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for MonogatariMcpServer {
    fn get_info(&self) -> ServerInfo {
        let mode = if self.allow_write {
            "Writes are enabled, but apply_transaction still requires a current reviewed plan fingerprint."
        } else {
            "Writes are disabled; restart with --allow-write to enable apply_transaction."
        };
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "monogatari-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(format!(
                "Author Monogatari visual novels inside the fixed project root. Inspect, list, and read before planning. Use validate_project for read-only headless acceptance and validate_delivery for declared asset readiness. Plan before apply. Package archives, Quality execution, and rendered visual acceptance remain higher gates. {mode}"
            ))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use llm_authoring::json_catalog::AuthorableJsonCatalog;
    use llm_authoring::project::default_project_config;

    use super::*;

    static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

    fn temp_project() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "monogatari-mcp-server-unit-{}-{}",
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
        root
    }

    #[test]
    fn exposes_seven_schema_backed_tools_with_write_annotations() {
        let root = temp_project();
        let server = MonogatariMcpServer::new(root.clone(), false).unwrap();
        let tools = server.tool_router.list_all();
        let names = tools
            .iter()
            .map(|tool| tool.name.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "apply_transaction",
                "inspect_project",
                "list_project_json",
                "plan_transaction",
                "read_project_json",
                "validate_delivery",
                "validate_project"
            ]
        );
        assert!(tools.iter().all(|tool| tool.output_schema.is_some()));
        let apply = tools
            .iter()
            .find(|tool| tool.name == "apply_transaction")
            .unwrap();
        assert_eq!(
            apply.annotations.as_ref().unwrap().destructive_hint,
            Some(true)
        );
        assert_eq!(
            apply.annotations.as_ref().unwrap().read_only_hint,
            Some(false)
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn coordinates_server_instances_without_mutating_the_project_root() {
        let root = temp_project();
        let before = std::fs::read_dir(&root).unwrap().count();
        let first_reader = MonogatariMcpServer::new(root.clone(), false).unwrap();
        let second_reader = MonogatariMcpServer::new(root.clone(), false).unwrap();
        assert!(MonogatariMcpServer::new(root.clone(), true).is_err());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), before);
        drop(first_reader);
        drop(second_reader);

        let writer = MonogatariMcpServer::new(root.clone(), true).unwrap();
        assert!(MonogatariMcpServer::new(root.clone(), true).is_err());
        assert!(MonogatariMcpServer::new(root.clone(), false).is_err());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), before);
        drop(writer);

        let replacement_reader = MonogatariMcpServer::new(root.clone(), false).unwrap();
        drop(replacement_reader);
        std::fs::remove_dir_all(root).unwrap();
    }
}
