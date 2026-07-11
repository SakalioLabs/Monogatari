//! Serializable Agent transaction wire models and stable error contracts.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const AGENT_TRANSACTION_SCHEMA_V1: &str = "monogatari-agent-project-transaction/v1";
pub const AGENT_TRANSACTION_PLAN_SCHEMA_V1: &str = "monogatari-agent-project-transaction-plan/v1";
pub const AGENT_TRANSACTION_RESULT_SCHEMA_V1: &str =
    "monogatari-agent-project-transaction-result/v1";
pub const AGENT_TRANSACTION_ERROR_SCHEMA_V1: &str = "monogatari-agent-project-transaction-error/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentProjectTransaction {
    pub schema: String,
    pub transaction_id: String,
    pub operations: Vec<AgentProjectOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentProjectOperation {
    PutJson {
        path: String,
        document: Value,
        precondition: AgentFilePrecondition,
    },
    DeleteJson {
        path: String,
        precondition: AgentFilePrecondition,
    },
}

impl AgentProjectOperation {
    pub(super) fn path(&self) -> &str {
        match self {
            Self::PutJson { path, .. } | Self::DeleteJson { path, .. } => path,
        }
    }

    pub(super) fn precondition(&self) -> &AgentFilePrecondition {
        match self {
            Self::PutJson { precondition, .. } | Self::DeleteJson { precondition, .. } => {
                precondition
            }
        }
    }

    pub(super) fn kind(&self) -> AgentOperationKind {
        match self {
            Self::PutJson { .. } => AgentOperationKind::PutJson,
            Self::DeleteJson { .. } => AgentOperationKind::DeleteJson,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentFilePrecondition {
    Missing,
    Sha256 { value: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentOperationKind {
    PutJson,
    DeleteJson,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentPlannedOperation {
    pub index: usize,
    pub operation: AgentOperationKind,
    pub path: String,
    pub previous_sha256: Option<String>,
    pub resulting_sha256: Option<String>,
    pub write_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentProjectTransactionPlan {
    pub schema: String,
    pub transaction_id: String,
    pub operation_count: usize,
    pub total_write_bytes: u64,
    pub precondition_fingerprint: String,
    pub operations: Vec<AgentPlannedOperation>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentTransactionStatus {
    Applied,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentProjectTransactionResult {
    pub schema: String,
    pub transaction_id: String,
    pub status: AgentTransactionStatus,
    pub precondition_fingerprint: String,
    pub validation: Value,
    pub operations: Vec<AgentPlannedOperation>,
    pub cleanup_warnings: Vec<AgentTransactionCleanupWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentTransactionCleanupWarning {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentTransactionErrorCode {
    SchemaMismatch,
    InvalidTransactionId,
    InvalidOperationCount,
    InvalidPath,
    PathNotAllowed,
    PathCaseCollision,
    DuplicatePath,
    InvalidDocument,
    PayloadTooLarge,
    InvalidPrecondition,
    PreconditionFailed,
    ProjectRootInvalid,
    IoFailure,
    CandidateValidationFailed,
    RollbackFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentTransactionError {
    pub schema: String,
    pub code: AgentTransactionErrorCode,
    pub message: String,
    pub operation_index: Option<usize>,
    pub path: Option<String>,
}

impl AgentTransactionError {
    pub fn candidate_validation(message: impl Into<String>) -> Self {
        Self::new(
            AgentTransactionErrorCode::CandidateValidationFailed,
            message,
            None,
            None,
        )
    }

    pub(super) fn new(
        code: AgentTransactionErrorCode,
        message: impl Into<String>,
        operation_index: Option<usize>,
        path: Option<String>,
    ) -> Self {
        Self {
            schema: AGENT_TRANSACTION_ERROR_SCHEMA_V1.to_string(),
            code,
            message: message.into(),
            operation_index,
            path,
        }
    }

    pub(super) fn for_operation(
        code: AgentTransactionErrorCode,
        message: impl Into<String>,
        index: usize,
        path: &str,
    ) -> Self {
        Self::new(code, message, Some(index), Some(path.to_string()))
    }
}

impl fmt::Display for AgentTransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AgentTransactionError {}
