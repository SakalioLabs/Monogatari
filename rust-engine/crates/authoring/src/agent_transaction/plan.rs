//! Side-effect-free transaction planning and optimistic filesystem validation.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::protocol::*;
#[cfg(test)]
use crate::json_catalog::AUTHORABLE_JSON_CATALOG_ROOTS;
use crate::json_catalog::{
    validate_authorable_json_path, JsonCatalogErrorCode, MAX_AUTHORABLE_JSON_BYTES,
};
use crate::paths::resolve_project_relative;
use crate::project::canonical_project_root;

const MAX_TRANSACTION_OPERATIONS: usize = 256;
pub(super) const MAX_TRANSACTION_FILE_BYTES: u64 = MAX_AUTHORABLE_JSON_BYTES;
const MAX_TRANSACTION_TOTAL_BYTES: u64 = 32 * 1024 * 1024;
const MAX_TRANSACTION_ID_BYTES: usize = 128;
#[cfg(test)]
pub(super) const ALLOWED_JSON_ROOTS: &[&str] = AUTHORABLE_JSON_CATALOG_ROOTS;

pub(super) struct PreparedTransaction {
    pub(super) root: PathBuf,
    pub(super) plan: AgentProjectTransactionPlan,
    pub(super) operations: Vec<PreparedOperation>,
}

pub(super) struct PreparedOperation {
    pub(super) index: usize,
    pub(super) operation: AgentOperationKind,
    pub(super) portable_path: String,
    pub(super) target_path: PathBuf,
    pub(super) encoded_document: Option<Vec<u8>>,
}

pub(super) fn prepare_transaction(
    project_root: &Path,
    transaction: &AgentProjectTransaction,
) -> Result<PreparedTransaction, AgentTransactionError> {
    if transaction.schema != AGENT_TRANSACTION_SCHEMA_V1 {
        return Err(AgentTransactionError::new(
            AgentTransactionErrorCode::SchemaMismatch,
            format!(
                "Unsupported agent transaction schema; expected `{AGENT_TRANSACTION_SCHEMA_V1}`."
            ),
            None,
            None,
        ));
    }
    validate_transaction_id(&transaction.transaction_id)?;
    if transaction.operations.is_empty()
        || transaction.operations.len() > MAX_TRANSACTION_OPERATIONS
    {
        return Err(AgentTransactionError::new(
            AgentTransactionErrorCode::InvalidOperationCount,
            format!("Agent transactions require 1 to {MAX_TRANSACTION_OPERATIONS} operations."),
            None,
            None,
        ));
    }

    let root = canonical_project_root(project_root).map_err(|_| {
        AgentTransactionError::new(
            AgentTransactionErrorCode::ProjectRootInvalid,
            "Project root must be an existing regular directory.",
            None,
            None,
        )
    })?;
    let mut seen_paths = HashSet::new();
    let mut planned = Vec::with_capacity(transaction.operations.len());
    let mut prepared = Vec::with_capacity(transaction.operations.len());
    let mut total_write_bytes = 0_u64;

    for (index, operation) in transaction.operations.iter().enumerate() {
        let portable_path = validate_operation_path(operation.path(), index)?;
        let portable_key = portable_path.to_ascii_lowercase();
        if !seen_paths.insert(portable_key) {
            return Err(AgentTransactionError::for_operation(
                AgentTransactionErrorCode::DuplicatePath,
                "A transaction cannot target the same portable path more than once.",
                index,
                &portable_path,
            ));
        }

        let target_path = resolve_project_relative(&root, &portable_path).map_err(|_| {
            AgentTransactionError::for_operation(
                AgentTransactionErrorCode::InvalidPath,
                "Transaction path must stay inside the project root.",
                index,
                &portable_path,
            )
        })?;
        verify_regular_parent_chain(&root, &portable_path, &target_path, index)?;
        reject_case_collision(&target_path, &portable_path, index)?;

        let current_sha256 = inspect_current_file(&target_path, &portable_path, index)?;
        validate_precondition(
            operation.precondition(),
            current_sha256.as_deref(),
            operation.kind(),
            index,
            &portable_path,
        )?;

        let encoded_document = match operation {
            AgentProjectOperation::PutJson { document, .. } => {
                if !document.is_object() && !document.is_array() {
                    return Err(AgentTransactionError::for_operation(
                        AgentTransactionErrorCode::InvalidDocument,
                        "put_json documents must contain a JSON object or array.",
                        index,
                        &portable_path,
                    ));
                }
                let mut bytes = serde_json::to_vec_pretty(document).map_err(|_| {
                    AgentTransactionError::for_operation(
                        AgentTransactionErrorCode::InvalidDocument,
                        "Unable to encode the JSON document.",
                        index,
                        &portable_path,
                    )
                })?;
                bytes.push(b'\n');
                if bytes.len() as u64 > MAX_TRANSACTION_FILE_BYTES {
                    return Err(AgentTransactionError::for_operation(
                        AgentTransactionErrorCode::PayloadTooLarge,
                        format!(
                            "A transaction JSON document cannot exceed {MAX_TRANSACTION_FILE_BYTES} bytes."
                        ),
                        index,
                        &portable_path,
                    ));
                }
                total_write_bytes = total_write_bytes
                    .checked_add(bytes.len() as u64)
                    .ok_or_else(|| {
                        AgentTransactionError::new(
                            AgentTransactionErrorCode::PayloadTooLarge,
                            "Transaction write size overflowed its supported range.",
                            None,
                            None,
                        )
                    })?;
                Some(bytes)
            }
            AgentProjectOperation::DeleteJson { .. } => None,
        };
        if total_write_bytes > MAX_TRANSACTION_TOTAL_BYTES {
            return Err(AgentTransactionError::new(
                AgentTransactionErrorCode::PayloadTooLarge,
                format!(
                    "Agent transaction writes cannot exceed {MAX_TRANSACTION_TOTAL_BYTES} bytes."
                ),
                None,
                None,
            ));
        }

        let resulting_sha256 = encoded_document.as_deref().map(sha256_hex);
        planned.push(AgentPlannedOperation {
            index,
            operation: operation.kind(),
            path: portable_path.clone(),
            previous_sha256: current_sha256,
            resulting_sha256,
            write_bytes: encoded_document
                .as_ref()
                .map_or(0, |bytes| bytes.len() as u64),
        });
        prepared.push(PreparedOperation {
            index,
            operation: operation.kind(),
            portable_path,
            target_path,
            encoded_document,
        });
    }

    let precondition_fingerprint = precondition_fingerprint(&transaction.transaction_id, &planned);
    Ok(PreparedTransaction {
        root,
        plan: AgentProjectTransactionPlan {
            schema: AGENT_TRANSACTION_PLAN_SCHEMA_V1.to_string(),
            transaction_id: transaction.transaction_id.clone(),
            operation_count: planned.len(),
            total_write_bytes,
            precondition_fingerprint,
            operations: planned,
        },
        operations: prepared,
    })
}

fn validate_transaction_id(transaction_id: &str) -> Result<(), AgentTransactionError> {
    let valid = !transaction_id.is_empty()
        && transaction_id.len() <= MAX_TRANSACTION_ID_BYTES
        && transaction_id == transaction_id.trim()
        && transaction_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'));
    if valid {
        return Ok(());
    }
    Err(AgentTransactionError::new(
        AgentTransactionErrorCode::InvalidTransactionId,
        "Transaction IDs may contain only ASCII letters, numbers, underscores, or hyphens.",
        None,
        None,
    ))
}

fn validate_operation_path(path: &str, index: usize) -> Result<String, AgentTransactionError> {
    validate_authorable_json_path(path).map_err(|error| {
        let code = if error.code == JsonCatalogErrorCode::PathNotAllowed {
            AgentTransactionErrorCode::PathNotAllowed
        } else {
            AgentTransactionErrorCode::InvalidPath
        };
        AgentTransactionError::for_operation(code, error.message, index, path)
    })?;
    Ok(path.to_string())
}

fn verify_regular_parent_chain(
    root: &Path,
    portable_path: &str,
    target_path: &Path,
    index: usize,
) -> Result<(), AgentTransactionError> {
    let relative_parent = Path::new(portable_path)
        .parent()
        .expect("validated transaction paths have a parent");
    let mut current = root.to_path_buf();
    for component in relative_parent.components() {
        current.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&current).map_err(|_| {
            AgentTransactionError::for_operation(
                AgentTransactionErrorCode::InvalidPath,
                "Every transaction parent directory must already exist.",
                index,
                portable_path,
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(AgentTransactionError::for_operation(
                AgentTransactionErrorCode::InvalidPath,
                "Transaction parent paths must be regular directories without symbolic links.",
                index,
                portable_path,
            ));
        }
    }
    let canonical_parent = target_path
        .parent()
        .expect("validated transaction paths have a parent")
        .canonicalize()
        .map_err(|_| {
            AgentTransactionError::for_operation(
                AgentTransactionErrorCode::InvalidPath,
                "Unable to resolve the transaction parent directory.",
                index,
                portable_path,
            )
        })?;
    if !canonical_parent.starts_with(root) {
        return Err(AgentTransactionError::for_operation(
            AgentTransactionErrorCode::InvalidPath,
            "Transaction paths must stay inside the project root.",
            index,
            portable_path,
        ));
    }
    Ok(())
}

fn reject_case_collision(
    target_path: &Path,
    portable_path: &str,
    index: usize,
) -> Result<(), AgentTransactionError> {
    let target_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("validated transaction filenames are ASCII");
    let parent = target_path
        .parent()
        .expect("validated transaction paths have a parent");
    let entries = std::fs::read_dir(parent).map_err(|_| {
        AgentTransactionError::for_operation(
            AgentTransactionErrorCode::IoFailure,
            "Unable to inspect the transaction target directory.",
            index,
            portable_path,
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|_| {
            AgentTransactionError::for_operation(
                AgentTransactionErrorCode::IoFailure,
                "Unable to inspect a transaction target entry.",
                index,
                portable_path,
            )
        })?;
        let Some(existing_name) = entry.file_name().to_str().map(str::to_string) else {
            return Err(AgentTransactionError::for_operation(
                AgentTransactionErrorCode::PathCaseCollision,
                "Transaction directories cannot contain non-UTF-8 filenames.",
                index,
                portable_path,
            ));
        };
        if existing_name.eq_ignore_ascii_case(target_name) && existing_name != target_name {
            return Err(AgentTransactionError::for_operation(
                AgentTransactionErrorCode::PathCaseCollision,
                "Transaction target collides with an existing path by ASCII case.",
                index,
                portable_path,
            ));
        }
    }
    Ok(())
}

fn inspect_current_file(
    target_path: &Path,
    portable_path: &str,
    index: usize,
) -> Result<Option<String>, AgentTransactionError> {
    let metadata = match std::fs::symlink_metadata(target_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err(AgentTransactionError::for_operation(
                AgentTransactionErrorCode::IoFailure,
                "Unable to inspect the transaction target.",
                index,
                portable_path,
            ));
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AgentTransactionError::for_operation(
            AgentTransactionErrorCode::InvalidPath,
            "Existing transaction targets must be regular files without symbolic links.",
            index,
            portable_path,
        ));
    }
    if metadata.len() > MAX_TRANSACTION_FILE_BYTES {
        return Err(AgentTransactionError::for_operation(
            AgentTransactionErrorCode::PayloadTooLarge,
            format!(
                "Existing transaction targets cannot exceed {MAX_TRANSACTION_FILE_BYTES} bytes."
            ),
            index,
            portable_path,
        ));
    }
    let bytes = std::fs::read(target_path).map_err(|_| {
        AgentTransactionError::for_operation(
            AgentTransactionErrorCode::IoFailure,
            "Unable to read the transaction target.",
            index,
            portable_path,
        )
    })?;
    Ok(Some(sha256_hex(&bytes)))
}

fn validate_precondition(
    precondition: &AgentFilePrecondition,
    current_sha256: Option<&str>,
    operation: AgentOperationKind,
    index: usize,
    path: &str,
) -> Result<(), AgentTransactionError> {
    match precondition {
        AgentFilePrecondition::Missing => {
            if operation == AgentOperationKind::DeleteJson {
                return Err(AgentTransactionError::for_operation(
                    AgentTransactionErrorCode::InvalidPrecondition,
                    "delete_json requires an exact SHA-256 precondition.",
                    index,
                    path,
                ));
            }
            if current_sha256.is_some() {
                return Err(AgentTransactionError::for_operation(
                    AgentTransactionErrorCode::PreconditionFailed,
                    "The transaction expected the target path to be missing.",
                    index,
                    path,
                ));
            }
        }
        AgentFilePrecondition::Sha256 { value } => {
            if !is_lowercase_sha256(value) {
                return Err(AgentTransactionError::for_operation(
                    AgentTransactionErrorCode::InvalidPrecondition,
                    "SHA-256 preconditions must be 64 lowercase hexadecimal characters.",
                    index,
                    path,
                ));
            }
            if current_sha256 != Some(value.as_str()) {
                return Err(AgentTransactionError::for_operation(
                    AgentTransactionErrorCode::PreconditionFailed,
                    "The transaction SHA-256 precondition does not match the current file.",
                    index,
                    path,
                ));
            }
        }
    }
    Ok(())
}

fn precondition_fingerprint(transaction_id: &str, operations: &[AgentPlannedOperation]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(AGENT_TRANSACTION_PLAN_SCHEMA_V1.as_bytes());
    hasher.update([0]);
    hasher.update(transaction_id.as_bytes());
    for operation in operations {
        hasher.update([0]);
        hasher.update(operation.path.as_bytes());
        hasher.update([0]);
        hasher.update(match operation.operation {
            AgentOperationKind::PutJson => b"put_json".as_slice(),
            AgentOperationKind::DeleteJson => b"delete_json".as_slice(),
        });
        hasher.update([0]);
        hasher.update(
            operation
                .previous_sha256
                .as_deref()
                .unwrap_or("<missing>")
                .as_bytes(),
        );
        hasher.update([0]);
        hasher.update(
            operation
                .resulting_sha256
                .as_deref()
                .unwrap_or("<deleted>")
                .as_bytes(),
        );
    }
    format!("{:x}", hasher.finalize())
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
