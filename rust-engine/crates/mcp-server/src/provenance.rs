//! Caller-owned build and time provenance for deterministic authoring domains.

use llm_authoring::project_package::ProjectExportProvenance;
use llm_authoring::quality_suite_execution::QualitySuiteRunProvenance;

pub(crate) fn project_export_provenance() -> ProjectExportProvenance {
    ProjectExportProvenance {
        exported_at: generated_at(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: build_git_value(option_env!("MONOGATARI_GIT_COMMIT")),
        git_short_commit: build_git_value(option_env!("MONOGATARI_GIT_SHORT_COMMIT")),
    }
}

pub(crate) fn quality_suite_run_provenance() -> QualitySuiteRunProvenance {
    QualitySuiteRunProvenance {
        generated_at: generated_at(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: build_git_value(option_env!("MONOGATARI_GIT_COMMIT")),
        git_short_commit: build_git_value(option_env!("MONOGATARI_GIT_SHORT_COMMIT")),
    }
}

fn generated_at() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn build_git_value(value: Option<&str>) -> String {
    value
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_and_quality_provenance_share_the_mcp_build_identity() {
        let package = project_export_provenance();
        let quality = quality_suite_run_provenance();
        assert_eq!(package.engine_version, quality.engine_version);
        assert_eq!(package.git_commit, quality.git_commit);
        assert_eq!(package.git_short_commit, quality.git_short_commit);
        assert!(!package.exported_at.is_empty());
        assert!(!quality.generated_at.is_empty());
    }
}
