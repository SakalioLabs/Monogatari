//! Candidate-project acceptance used by the write tool.

use std::path::Path;

use llm_authoring::agent_transaction::AgentTransactionError;
use llm_authoring::json_catalog::{
    inspect_project_json_catalog, JsonAcceptanceLevel, JsonCatalogIssueSeverity,
};
use llm_authoring::project::inspect_project_config;

use crate::protocol::{CandidateDocumentValidation, MCP_CANDIDATE_VALIDATION_SCHEMA_V1};

pub fn validate_candidate_documents(
    project_root: &Path,
) -> Result<CandidateDocumentValidation, AgentTransactionError> {
    let project = inspect_project_config(project_root).map_err(|_| {
        AgentTransactionError::candidate_validation(
            "Candidate project settings could not be inspected.",
        )
    })?;
    if !project.settings_exists || !project.valid {
        return Err(AgentTransactionError::candidate_validation(
            "Candidate project settings are not document-ready.",
        ));
    }

    let report = inspect_project_json_catalog(project_root, None).map_err(|_| {
        AgentTransactionError::candidate_validation(
            "Candidate project JSON catalogs could not be inspected.",
        )
    })?;
    if !report.valid {
        let evidence = report
            .issues
            .iter()
            .filter(|issue| issue.severity == JsonCatalogIssueSeverity::Error)
            .take(3)
            .map(|issue| {
                issue.path.as_deref().map_or_else(
                    || format!("{:?}", issue.code),
                    |path| format!("{:?}: {path}", issue.code),
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        return Err(AgentTransactionError::candidate_validation(format!(
            "Candidate project failed document-level JSON acceptance: {evidence}."
        )));
    }

    Ok(CandidateDocumentValidation {
        schema: MCP_CANDIDATE_VALIDATION_SCHEMA_V1.to_string(),
        acceptance_level: JsonAcceptanceLevel::Document,
        valid: true,
        document_count: report.document_count,
        total_bytes: report.total_bytes,
        warning_count: report.warning_count,
    })
}
