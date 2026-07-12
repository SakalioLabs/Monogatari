//! Candidate-project acceptance used by the write tool.

use std::path::Path;

use llm_authoring::agent_transaction::AgentTransactionError;
use llm_authoring::runtime_validation::{
    validate_core_runtime_project, CoreRuntimeValidationReport,
};

pub async fn validate_candidate_core_runtime(
    project_root: &Path,
) -> Result<CoreRuntimeValidationReport, AgentTransactionError> {
    let report = validate_core_runtime_project(project_root)
        .await
        .map_err(|_| {
            AgentTransactionError::candidate_validation(
                "Candidate core runtime validation could not be completed.",
            )
        })?;
    if !report.valid {
        let evidence = report
            .issues
            .iter()
            .take(3)
            .map(|issue| {
                issue.path.as_deref().map_or_else(
                    || issue.code.clone(),
                    |path| format!("{}: {path}", issue.code),
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        return Err(AgentTransactionError::candidate_validation(format!(
            "Candidate project failed core runtime acceptance: {evidence}."
        )));
    }
    Ok(report)
}
