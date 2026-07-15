//! MCP-specific request, result, and error envelopes.

use llm_authoring::agent_transaction::{
    AgentProjectTransaction, AgentProjectTransactionResult, AgentTransactionError,
};
use llm_authoring::json_catalog::{
    AuthorableJsonCatalog, JsonAcceptanceLevel, JsonCatalogError, JsonCatalogReport,
};
use llm_authoring::project::ProjectConfigState;
use llm_authoring::project_package::{ProjectPackageExportResult, ProjectPackageInspection};
use llm_authoring::quality_suite_execution::QualitySuiteReport;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MCP_INSPECTION_SCHEMA_V1: &str = "monogatari-mcp-project-inspection/v1";
pub const MCP_PACKAGE_EXPORT_SCHEMA_V1: &str = "monogatari-mcp-package-export/v1";
pub const MCP_PACKAGE_INSPECTION_SCHEMA_V1: &str = "monogatari-mcp-package-inspection/v1";
pub const MCP_PACKAGE_PREVIEW_SCHEMA_V1: &str = "monogatari-mcp-package-preview/v1";
pub const MCP_QUALITY_SUITE_RUN_SCHEMA_V1: &str = "monogatari-mcp-quality-suite-run/v1";
pub const MCP_TOOL_ERROR_SCHEMA_V1: &str = "monogatari-mcp-tool-error/v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct InspectProjectOutput {
    pub schema: String,
    pub acceptance_level: JsonAcceptanceLevel,
    pub write_enabled: bool,
    pub package_output_configured: bool,
    pub project: ProjectConfigState,
    pub json_catalog: JsonCatalogReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PreviewProjectPackageOutput {
    pub schema: String,
    pub package_output_configured: bool,
    pub project_title: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub content_sha256: String,
    pub manifest: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExportProjectPackageRequest {
    pub file_name: String,
    pub expected_content_sha256: String,
    #[serde(default)]
    pub replace_existing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExportProjectPackageOutput {
    pub schema: String,
    pub package: ProjectPackageExportResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InspectProjectPackageRequest {
    pub file_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct InspectProjectPackageOutput {
    pub schema: String,
    pub package: ProjectPackageInspection,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunQualitySuiteRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunQualitySuiteOutput {
    pub schema: String,
    pub passed: bool,
    pub report: QualitySuiteReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ApplyTransactionRequest {
    pub transaction: AgentProjectTransaction,
    pub expected_precondition_fingerprint: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpToolErrorCode {
    WriteDisabled,
    PlanFingerprintMismatch,
    PackageOutputUnavailable,
    PackageFingerprintMismatch,
    PackageError,
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

    pub fn package_output_unavailable() -> Self {
        Self::new(
            McpToolErrorCode::PackageOutputUnavailable,
            "Project package directory is not configured. Restart the server with --package-output-dir pointing to a reviewed external directory.",
            None,
        )
    }

    pub fn package_fingerprint_mismatch(expected: &str, current: &str) -> Self {
        Self::new(
            McpToolErrorCode::PackageFingerprintMismatch,
            "The supplied package content fingerprint does not match the current project. Preview and review the package again before exporting.",
            Some(serde_json::json!({
                "expected_content_sha256": expected,
                "current_content_sha256": current,
            })),
        )
    }

    pub fn package(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new(McpToolErrorCode::PackageError, message, details)
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
