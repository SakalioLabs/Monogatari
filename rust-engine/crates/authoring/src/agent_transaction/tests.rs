//! Agent transaction protocol and rollback regressions.

use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;

use super::plan::{sha256_hex, ALLOWED_JSON_ROOTS};
use super::*;

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

fn temp_project(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "monogatari-agent-transaction-{label}-{}-{}",
        std::process::id(),
        NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    for directory in ALLOWED_JSON_ROOTS {
        std::fs::create_dir_all(root.join(directory)).unwrap();
    }
    root
}

fn current_sha(root: &Path, path: &str) -> String {
    sha256_hex(&std::fs::read(root.join(path)).unwrap())
}

fn transaction(operations: Vec<AgentProjectOperation>) -> AgentProjectTransaction {
    AgentProjectTransaction {
        schema: AGENT_TRANSACTION_SCHEMA_V1.to_string(),
        transaction_id: "agent_batch_1".to_string(),
        operations,
    }
}

fn put_missing(path: &str, document: Value) -> AgentProjectOperation {
    AgentProjectOperation::PutJson {
        path: path.to_string(),
        document,
        precondition: AgentFilePrecondition::Missing,
    }
}

#[test]
fn wire_format_rejects_unknown_fields_and_schema_versions() {
    let unknown = json!({
        "schema": AGENT_TRANSACTION_SCHEMA_V1,
        "transaction_id": "batch",
        "operations": [{
            "op": "put_json",
            "path": "characters/aoi.json",
            "document": {"id": "aoi"},
            "precondition": {"kind": "missing"},
            "unexpected": true
        }]
    });
    assert!(serde_json::from_value::<AgentProjectTransaction>(unknown).is_err());

    let root = temp_project("schema");
    let mut request = transaction(vec![put_missing(
        "characters/aoi.json",
        json!({"id": "aoi"}),
    )]);
    request.schema = "monogatari-agent-project-transaction/v2".to_string();
    let error = plan_agent_project_transaction(&root, &request).unwrap_err();
    assert_eq!(error.code, AgentTransactionErrorCode::SchemaMismatch);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn plan_rejects_escape_restricted_and_duplicate_paths() {
    let root = temp_project("paths");
    for path in [
        "../characters/aoi.json",
        "settings.json",
        "saves/slot.json",
        "characters/aoi.JSON",
        "characters/.hidden.json",
        "characters/aoi profile.json",
    ] {
        let request = transaction(vec![put_missing(path, json!({"id": "aoi"}))]);
        assert!(
            plan_agent_project_transaction(&root, &request).is_err(),
            "{path} should be rejected"
        );
    }

    let request = transaction(vec![
        put_missing("characters/aoi.json", json!({"id": "aoi"})),
        put_missing("characters/AOI.json", json!({"id": "aoi_2"})),
    ]);
    let error = plan_agent_project_transaction(&root, &request).unwrap_err();
    assert_eq!(error.code, AgentTransactionErrorCode::DuplicatePath);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn plan_rejects_existing_case_collisions() {
    let root = temp_project("case-collision");
    std::fs::write(root.join("characters").join("Aoi.json"), b"{}\n").unwrap();
    let request = transaction(vec![put_missing(
        "characters/aoi.json",
        json!({"id": "aoi"}),
    )]);
    let error = plan_agent_project_transaction(&root, &request).unwrap_err();
    assert_eq!(error.code, AgentTransactionErrorCode::PathCaseCollision);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn plan_requires_exact_create_update_and_delete_preconditions() {
    let root = temp_project("preconditions");
    std::fs::write(root.join("characters/aoi.json"), b"{\"id\":\"aoi\"}\n").unwrap();

    let create_over_existing = transaction(vec![put_missing(
        "characters/aoi.json",
        json!({"id": "aoi"}),
    )]);
    assert_eq!(
        plan_agent_project_transaction(&root, &create_over_existing)
            .unwrap_err()
            .code,
        AgentTransactionErrorCode::PreconditionFailed
    );

    let wrong_hash = transaction(vec![AgentProjectOperation::PutJson {
        path: "characters/aoi.json".to_string(),
        document: json!({"id": "aoi", "name": "Aoi"}),
        precondition: AgentFilePrecondition::Sha256 {
            value: "0".repeat(64),
        },
    }]);
    assert_eq!(
        plan_agent_project_transaction(&root, &wrong_hash)
            .unwrap_err()
            .code,
        AgentTransactionErrorCode::PreconditionFailed
    );

    let delete_without_hash = transaction(vec![AgentProjectOperation::DeleteJson {
        path: "characters/aoi.json".to_string(),
        precondition: AgentFilePrecondition::Missing,
    }]);
    assert_eq!(
        plan_agent_project_transaction(&root, &delete_without_hash)
            .unwrap_err()
            .code,
        AgentTransactionErrorCode::InvalidPrecondition
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn plan_is_deterministic_and_does_not_write() {
    let root = temp_project("plan");
    let request = transaction(vec![put_missing(
        "characters/aoi.json",
        json!({"id": "aoi", "name": "Aoi"}),
    )]);
    let first = plan_agent_project_transaction(&root, &request).unwrap();
    let second = plan_agent_project_transaction(&root, &request).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.schema, AGENT_TRANSACTION_PLAN_SCHEMA_V1);
    assert_eq!(first.operation_count, 1);
    assert_eq!(first.precondition_fingerprint.len(), 64);
    assert!(!root.join("characters/aoi.json").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn apply_commits_multiple_writes_and_deletion_after_validation() {
    let root = temp_project("apply");
    std::fs::write(root.join("characters/aoi.json"), b"{\"id\":\"aoi\"}\n").unwrap();
    std::fs::write(
        root.join("dialogue/obsolete.json"),
        b"{\"id\":\"obsolete\"}\n",
    )
    .unwrap();
    let aoi_sha = current_sha(&root, "characters/aoi.json");
    let obsolete_sha = current_sha(&root, "dialogue/obsolete.json");
    let request = transaction(vec![
        AgentProjectOperation::PutJson {
            path: "characters/aoi.json".to_string(),
            document: json!({"id": "aoi", "name": "Aoi"}),
            precondition: AgentFilePrecondition::Sha256 { value: aoi_sha },
        },
        put_missing("dialogue/intro.json", json!({"id": "intro", "nodes": {}})),
        AgentProjectOperation::DeleteJson {
            path: "dialogue/obsolete.json".to_string(),
            precondition: AgentFilePrecondition::Sha256 {
                value: obsolete_sha,
            },
        },
    ]);

    let result = apply_agent_project_transaction_with_validator(
        &root,
        &request,
        |candidate_root| async move {
            let aoi: Value = serde_json::from_slice(
                &std::fs::read(candidate_root.join("characters/aoi.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(aoi["name"], "Aoi");
            assert!(candidate_root.join("dialogue/intro.json").is_file());
            assert!(!candidate_root.join("dialogue/obsolete.json").exists());
            Ok(json!({"validator": "test", "valid": true}))
        },
    )
    .await
    .unwrap();

    assert_eq!(result.status, AgentTransactionStatus::Applied);
    assert_eq!(result.operations.len(), 3);
    assert!(result.cleanup_warnings.is_empty());
    assert_eq!(result.validation["valid"], true);
    assert!(root.join("dialogue/intro.json").is_file());
    assert!(!root.join("dialogue/obsolete.json").exists());
    assert_eq!(std::fs::read_dir(root.join("dialogue")).unwrap().count(), 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn validation_failure_rolls_back_every_staged_operation() {
    let root = temp_project("rollback");
    let original_aoi = b"{\"id\":\"aoi\"}\n";
    let original_dialogue = b"{\"id\":\"obsolete\"}\n";
    std::fs::write(root.join("characters/aoi.json"), original_aoi).unwrap();
    std::fs::write(root.join("dialogue/obsolete.json"), original_dialogue).unwrap();
    let request = transaction(vec![
        AgentProjectOperation::PutJson {
            path: "characters/aoi.json".to_string(),
            document: json!({"id": "aoi", "name": "Changed"}),
            precondition: AgentFilePrecondition::Sha256 {
                value: current_sha(&root, "characters/aoi.json"),
            },
        },
        put_missing("dialogue/new.json", json!({"id": "new"})),
        AgentProjectOperation::DeleteJson {
            path: "dialogue/obsolete.json".to_string(),
            precondition: AgentFilePrecondition::Sha256 {
                value: current_sha(&root, "dialogue/obsolete.json"),
            },
        },
    ]);

    let error = apply_agent_project_transaction_with_validator(&root, &request, |_| async {
        Err(AgentTransactionError::candidate_validation(
            "Candidate graph validation failed.",
        ))
    })
    .await
    .unwrap_err();

    assert_eq!(
        error.code,
        AgentTransactionErrorCode::CandidateValidationFailed
    );
    assert_eq!(
        std::fs::read(root.join("characters/aoi.json")).unwrap(),
        original_aoi
    );
    assert!(!root.join("dialogue/new.json").exists());
    assert_eq!(
        std::fs::read(root.join("dialogue/obsolete.json")).unwrap(),
        original_dialogue
    );
    assert_eq!(std::fs::read_dir(root.join("dialogue")).unwrap().count(), 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn scalar_documents_and_invalid_transaction_ids_are_rejected() {
    let root = temp_project("shape");
    let scalar = transaction(vec![put_missing("characters/aoi.json", json!("aoi"))]);
    assert_eq!(
        plan_agent_project_transaction(&root, &scalar)
            .unwrap_err()
            .code,
        AgentTransactionErrorCode::InvalidDocument
    );

    let mut invalid_id = transaction(vec![put_missing(
        "characters/aoi.json",
        json!({"id": "aoi"}),
    )]);
    invalid_id.transaction_id = "agent batch/1".to_string();
    assert_eq!(
        plan_agent_project_transaction(&root, &invalid_id)
            .unwrap_err()
            .code,
        AgentTransactionErrorCode::InvalidTransactionId
    );
    std::fs::remove_dir_all(root).unwrap();
}
