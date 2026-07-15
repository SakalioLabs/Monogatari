//! Ephemeral package extraction and shared runtime acceptance.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use llm_authoring::delivery_validation::validate_project_delivery;
use llm_authoring::project_package::extract_project_package;

use crate::protocol::{
    McpToolError, ValidateProjectPackageOutput, ValidateProjectPackageRequest,
    MCP_PACKAGE_VALIDATION_SCHEMA_V1,
};

use super::{canonical_regular_directory, PackageDirectoryBoundary};

const MAX_STAGING_ATTEMPTS: u64 = 128;
static NEXT_STAGING_ROOT: AtomicU64 = AtomicU64::new(0);

pub(crate) async fn validate_project_package(
    project_root: PathBuf,
    package_directory: PackageDirectoryBoundary,
    request: ValidateProjectPackageRequest,
) -> Result<ValidateProjectPackageOutput, McpToolError> {
    let file_name = request.file_name;
    let archive_path = package_directory.package_path(&file_name)?;
    let project_root_for_task = project_root;
    let archive_for_task = archive_path;
    let (staging, package_result) = tokio::task::spawn_blocking(move || {
        let staging =
            EphemeralProjectRoot::create(&project_root_for_task).map_err(McpToolError::internal)?;
        let package = extract_project_package(&archive_for_task, staging.path());
        Ok::<_, McpToolError>((staging, package))
    })
    .await
    .map_err(|_| McpToolError::internal("Project package re-import task did not complete."))??;

    let package = match package_result {
        Ok(package) => package,
        Err(message) => {
            cleanup_staging(staging).await?;
            return Err(McpToolError::package(
                message,
                Some(serde_json::json!({ "file_name": file_name })),
            ));
        }
    };

    let delivery_result = validate_project_delivery(staging.path()).await;
    cleanup_staging(staging).await?;
    let delivery = delivery_result.map_err(|message| {
        McpToolError::package(message, Some(serde_json::json!({ "file_name": file_name })))
    })?;

    Ok(ValidateProjectPackageOutput {
        schema: MCP_PACKAGE_VALIDATION_SCHEMA_V1.to_string(),
        passed: delivery.valid,
        package,
        delivery,
    })
}

async fn cleanup_staging(staging: EphemeralProjectRoot) -> Result<(), McpToolError> {
    tokio::task::spawn_blocking(move || staging.cleanup())
        .await
        .map_err(|_| McpToolError::internal("Project package staging cleanup did not complete."))?
        .map_err(McpToolError::internal)
}

struct EphemeralProjectRoot {
    path: Option<PathBuf>,
}

impl EphemeralProjectRoot {
    fn create(project_root: &Path) -> Result<Self, String> {
        let temporary_root = canonical_regular_directory(
            &std::env::temp_dir(),
            "System temporary directory for MCP package validation",
        )?;
        if temporary_root.starts_with(project_root) {
            return Err(
                "MCP package validation staging must stay outside the authored project root."
                    .to_string(),
            );
        }

        for _ in 0..MAX_STAGING_ATTEMPTS {
            let sequence = NEXT_STAGING_ROOT.fetch_add(1, Ordering::Relaxed);
            let candidate = temporary_root.join(format!(
                "monogatari-mcp-package-validation-{}-{sequence}",
                std::process::id()
            ));
            match std::fs::create_dir(&candidate) {
                Ok(()) => {
                    let path = match candidate.canonicalize() {
                        Ok(path) => path,
                        Err(error) => {
                            let _ = std::fs::remove_dir(&candidate);
                            return Err(format!(
                                "Unable to resolve MCP package validation staging: {error}"
                            ));
                        }
                    };
                    if !path.starts_with(&temporary_root) || path.starts_with(project_root) {
                        let _ = std::fs::remove_dir(&candidate);
                        return Err(
                            "MCP package validation staging escaped its temporary boundary."
                                .to_string(),
                        );
                    }
                    return Ok(Self { path: Some(path) });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "Unable to create MCP package validation staging: {error}"
                    ));
                }
            }
        }
        Err("Unable to allocate a unique MCP package validation staging directory.".to_string())
    }

    fn path(&self) -> &Path {
        self.path
            .as_deref()
            .expect("ephemeral project root must exist until cleanup")
    }

    fn cleanup(mut self) -> Result<(), String> {
        self.path
            .take()
            .map_or(Ok(()), |path| remove_validation_root(&path))
    }
}

impl Drop for EphemeralProjectRoot {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = remove_validation_root(&path);
        }
    }
}

fn remove_validation_root(path: &Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "Unable to inspect MCP package validation staging during cleanup: {error}"
            ));
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(
            "MCP package validation staging changed type before cleanup and was not removed."
                .to_string(),
        );
    }
    std::fs::remove_dir_all(path)
        .map_err(|error| format!("Unable to remove MCP package validation staging: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ephemeral_project_roots_are_unique_external_and_removed() {
        let project_root = std::env::temp_dir().join(format!(
            "monogatari-mcp-reimport-test-{}-{}",
            std::process::id(),
            NEXT_STAGING_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&project_root).unwrap();
        let project_root = project_root.canonicalize().unwrap();

        let first = EphemeralProjectRoot::create(&project_root).unwrap();
        let first_path = first.path().to_path_buf();
        let second = EphemeralProjectRoot::create(&project_root).unwrap();
        let second_path = second.path().to_path_buf();
        assert_ne!(first_path, second_path);
        assert!(!first_path.starts_with(&project_root));
        assert!(!second_path.starts_with(&project_root));
        std::fs::create_dir(first_path.join("nested")).unwrap();
        std::fs::write(first_path.join("nested/evidence.json"), b"{}").unwrap();

        first.cleanup().unwrap();
        assert!(!first_path.exists());
        drop(second);
        assert!(!second_path.exists());
        std::fs::remove_dir(project_root).unwrap();
    }
}
