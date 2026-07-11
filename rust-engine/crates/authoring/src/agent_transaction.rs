//! Versioned, optimistic, rollback-capable JSON operations for agent authoring.

use std::future::Future;
use std::path::{Path, PathBuf};

use serde_json::Value;

mod plan;
mod protocol;

pub use protocol::*;

#[cfg(test)]
mod tests;

use plan::{prepare_transaction, MAX_TRANSACTION_FILE_BYTES};

use crate::filesystem::{stage_json_deletion, stage_json_replacement, StagedContentMutation};

struct StagedOperation {
    index: usize,
    portable_path: String,
    mutation: StagedContentMutation,
}

/// Validate an agent transaction and resolve its optimistic filesystem plan without writing.
pub fn plan_agent_project_transaction(
    project_root: &Path,
    transaction: &AgentProjectTransaction,
) -> Result<AgentProjectTransactionPlan, AgentTransactionError> {
    prepare_transaction(project_root, transaction).map(|prepared| prepared.plan)
}

/// Apply a prepared candidate, run the caller's authoritative project validator, and roll back
/// every staged file when validation fails. Callers must serialize transactions per project.
pub async fn apply_agent_project_transaction_with_validator<Validator, ValidationFuture>(
    project_root: &Path,
    transaction: &AgentProjectTransaction,
    validator: Validator,
) -> Result<AgentProjectTransactionResult, AgentTransactionError>
where
    Validator: FnOnce(PathBuf) -> ValidationFuture,
    ValidationFuture: Future<Output = Result<Value, AgentTransactionError>>,
{
    let prepared = prepare_transaction(project_root, transaction)?;
    let mut staged = Vec::with_capacity(prepared.operations.len());

    for operation in &prepared.operations {
        let mutation_result = match operation.operation {
            AgentOperationKind::PutJson => {
                let bytes = operation
                    .encoded_document
                    .as_deref()
                    .expect("put_json operations always carry encoded content");
                stage_json_replacement(
                    &operation.target_path,
                    bytes,
                    MAX_TRANSACTION_FILE_BYTES,
                    "agent transaction JSON",
                )
                .await
            }
            AgentOperationKind::DeleteJson => {
                stage_json_deletion(&operation.target_path, "agent transaction JSON").await
            }
        };

        match mutation_result {
            Ok(mutation) => staged.push(StagedOperation {
                index: operation.index,
                portable_path: operation.portable_path.clone(),
                mutation,
            }),
            Err(_) => {
                rollback_staged(&mut staged).await?;
                return Err(AgentTransactionError::for_operation(
                    AgentTransactionErrorCode::IoFailure,
                    "Unable to stage the project JSON operation.",
                    operation.index,
                    &operation.portable_path,
                ));
            }
        }
    }

    let validation = match validator(prepared.root.clone()).await {
        Ok(validation) => validation,
        Err(error) => {
            rollback_staged(&mut staged).await?;
            return Err(error);
        }
    };

    let mut cleanup_warnings = Vec::new();
    for operation in staged {
        if operation.mutation.commit().await.is_err() {
            cleanup_warnings.push(AgentTransactionCleanupWarning {
                code: "backup_cleanup_failed".to_string(),
                path: operation.portable_path,
                message: "The transaction was applied, but a staged backup could not be removed."
                    .to_string(),
            });
        }
    }

    Ok(AgentProjectTransactionResult {
        schema: AGENT_TRANSACTION_RESULT_SCHEMA_V1.to_string(),
        transaction_id: prepared.plan.transaction_id,
        status: AgentTransactionStatus::Applied,
        precondition_fingerprint: prepared.plan.precondition_fingerprint,
        validation,
        operations: prepared.plan.operations,
        cleanup_warnings,
    })
}

async fn rollback_staged(staged: &mut Vec<StagedOperation>) -> Result<(), AgentTransactionError> {
    let mut first_failure = None;
    while let Some(operation) = staged.pop() {
        if operation.mutation.rollback().await.is_err() && first_failure.is_none() {
            first_failure = Some(AgentTransactionError::for_operation(
                AgentTransactionErrorCode::RollbackFailed,
                "Unable to restore a staged project JSON operation.",
                operation.index,
                &operation.portable_path,
            ));
        }
    }
    match first_failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
