//! MCP-specific request, result, and error envelopes.

use llm_authoring::agent_transaction::{
    AgentProjectTransaction, AgentProjectTransactionResult, AgentTransactionError,
};
use llm_authoring::json_catalog::{
    AuthorableJsonCatalog, JsonAcceptanceLevel, JsonCatalogError, JsonCatalogReport,
};
use llm_authoring::project::ProjectConfigState;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MCP_INSPECTION_SCHEMA_V1: &str = "monogatari-mcp-project-inspection/v1";
pub const MCP_CANDIDATE_VALIDATION_SCHEMA_V1: &str =
    "monogatari-mcp-candidate-document-validation/v1";
pub const MCP_TOOL_ERROR_SCHEMA_V1: &str = "monogatari-mcp-tool-error/v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct InspectProjectOutput {
    pub schema: String,
    pub acceptance_level: JsonAcceptanceLevel,
    pub write_enabled: bool,
    pub project: ProjectConfigState,
    pub json_catalog: JsonCatalogReport,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ListProjectJsonRequest {
    #[serde(default)]
    pub catalog: Option<AuthorableJsonCatalog>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReadProjectJsonRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ApplyTransactionRequest {
    pub transaction: AgentProjectTransaction,
    pub expected_precondition_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CandidateDocumentValidation {
    pub schema: String,
    pub acceptance_level: JsonAcceptanceLevel,
    pub valid: bool,
    pub document_count: usize,
    pub total_bytes: u64,
    pub warning_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpToolErrorCode {
    WriteDisabled,
    PlanFingerprintMismatch,
    ProjectInvalid,
    CatalogError,
    TransactionError,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct McpToolError {
    pub schema: String,
    pub code: McpToolErrorCode,
    pub message: String,
    pub details: Option<Value>,
}

impl McpToolError {
    pub fn write_disabled() -> Self {
        Self::new(
            McpToolErrorCode::WriteDisabled,
            "Project writes are disabled. Restart the server with --allow-write after reviewing the security boundary.",
            None,
        )
    }

    pub fn fingerprint_mismatch() -> Self {
        Self::new(
            McpToolErrorCode::PlanFingerprintMismatch,
            "The supplied plan fingerprint does not match the current transaction plan. Re-plan and review before applying.",
            None,
        )
    }

    pub fn project(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new(McpToolErrorCode::ProjectInvalid, message, details)
    }

    pub fn catalog(error: JsonCatalogError) -> Self {
        let message = error.message.clone();
        Self::new(
            McpToolErrorCode::CatalogError,
            message,
            serde_json::to_value(error).ok(),
        )
    }

    pub fn transaction(error: AgentTransactionError) -> Self {
        let message = error.message.clone();
        Self::new(
            McpToolErrorCode::TransactionError,
            message,
            serde_json::to_value(error).ok(),
        )
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(McpToolErrorCode::InternalError, message, None)
    }

    fn new(code: McpToolErrorCode, message: impl Into<String>, details: Option<Value>) -> Self {
        Self {
            schema: MCP_TOOL_ERROR_SCHEMA_V1.to_string(),
            code,
            message: message.into(),
            details,
        }
    }
}

pub type ApplyTransactionOutput = AgentProjectTransactionResult;
