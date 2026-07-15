//! Fixed-root MCP transport for project-package preview, inspection, and export.

mod reimport;

use std::path::{Path, PathBuf};

use llm_authoring::project_package::{
    build_project_export_manifest, inspect_project_package as inspect_package_archive,
    validate_manifest, validate_portable_path, write_project_package, ProjectExportRuntimeSnapshot,
    ProjectPackageTargetPolicy,
};

use crate::protocol::{
    ExportProjectPackageOutput, ExportProjectPackageRequest, InspectProjectPackageOutput,
    InspectProjectPackageRequest, McpToolError, PreviewProjectPackageOutput,
    MCP_PACKAGE_EXPORT_SCHEMA_V1, MCP_PACKAGE_INSPECTION_SCHEMA_V1, MCP_PACKAGE_PREVIEW_SCHEMA_V1,
};
use crate::provenance::project_export_provenance;

pub(crate) use reimport::validate_project_package;

const MAX_PACKAGE_FILE_NAME_BYTES: usize = 240;

#[derive(Debug, Clone)]
pub(crate) struct PackageDirectoryBoundary {
    root: PathBuf,
}

impl PackageDirectoryBoundary {
    pub(crate) fn new(project_root: &Path, output_root: PathBuf) -> Result<Self, String> {
        let root = canonical_regular_directory(&output_root, "MCP package directory")?;
        if root.starts_with(project_root) {
            return Err(
                "MCP package directory must stay outside the authored project root.".to_string(),
            );
        }
        Ok(Self { root })
    }

    fn package_path(&self, file_name: &str) -> Result<PathBuf, McpToolError> {
        validate_package_file_name(file_name)?;
        let current_root = canonical_regular_directory(&self.root, "MCP package directory")
            .map_err(|message| McpToolError::package(message, None))?;
        if current_root != self.root {
            return Err(McpToolError::package(
                "MCP package directory changed after server startup.",
                None,
            ));
        }
        Ok(current_root.join(file_name))
    }
}

pub(crate) async fn inspect_project_package(
    package_directory: PackageDirectoryBoundary,
    request: InspectProjectPackageRequest,
) -> Result<InspectProjectPackageOutput, McpToolError> {
    tokio::task::spawn_blocking(move || {
        let archive_path = package_directory.package_path(&request.file_name)?;
        let package = inspect_package_archive(&archive_path).map_err(|message| {
            McpToolError::package(
                message,
                Some(serde_json::json!({ "file_name": request.file_name })),
            )
        })?;
        Ok(InspectProjectPackageOutput {
            schema: MCP_PACKAGE_INSPECTION_SCHEMA_V1.to_string(),
            package,
        })
    })
    .await
    .map_err(|_| McpToolError::internal("Project package inspection task did not complete."))?
}

pub(crate) async fn preview_project_package(
    project_root: PathBuf,
    package_output_configured: bool,
) -> Result<PreviewProjectPackageOutput, McpToolError> {
    tokio::task::spawn_blocking(move || {
        build_package_preview(&project_root, package_output_configured)
    })
    .await
    .map_err(|_| McpToolError::internal("Project package preview task did not complete."))?
}

pub(crate) async fn export_project_package(
    project_root: PathBuf,
    package_directory: PackageDirectoryBoundary,
    request: ExportProjectPackageRequest,
) -> Result<ExportProjectPackageOutput, McpToolError> {
    tokio::task::spawn_blocking(move || {
        validate_expected_fingerprint(&request.expected_content_sha256)?;
        let preview = build_package_preview(&project_root, true)?;
        if preview.content_sha256 != request.expected_content_sha256 {
            return Err(McpToolError::package_fingerprint_mismatch(
                &request.expected_content_sha256,
                &preview.content_sha256,
            ));
        }
        let destination = package_directory.package_path(&request.file_name)?;
        let target_policy = if request.replace_existing {
            ProjectPackageTargetPolicy::ReplaceExisting
        } else {
            ProjectPackageTargetPolicy::CreateNew
        };
        let package =
            write_project_package(&project_root, &destination, preview.manifest, target_policy)
                .map_err(|message| {
                    McpToolError::package(
                        message,
                        Some(serde_json::json!({ "file_name": request.file_name })),
                    )
                })?;
        Ok(ExportProjectPackageOutput {
            schema: MCP_PACKAGE_EXPORT_SCHEMA_V1.to_string(),
            package,
        })
    })
    .await
    .map_err(|_| McpToolError::internal("Project package export task did not complete."))?
}

fn build_package_preview(
    project_root: &Path,
    package_output_configured: bool,
) -> Result<PreviewProjectPackageOutput, McpToolError> {
    let manifest = build_project_export_manifest(
        project_root,
        ProjectExportRuntimeSnapshot::default(),
        project_export_provenance(),
    )
    .map_err(|message| McpToolError::package(message, None))?;
    let validated = validate_manifest(manifest.clone())
        .map_err(|message| McpToolError::package(message, None))?;
    Ok(PreviewProjectPackageOutput {
        schema: MCP_PACKAGE_PREVIEW_SCHEMA_V1.to_string(),
        package_output_configured,
        project_title: validated.project_title,
        file_count: validated.parsed.package.file_count,
        total_bytes: validated.parsed.package.total_bytes,
        content_sha256: validated.parsed.package.content_sha256,
        manifest,
    })
}

fn validate_expected_fingerprint(value: &str) -> Result<(), McpToolError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(McpToolError::package(
            "expected_content_sha256 must be exactly 64 lowercase hexadecimal characters.",
            None,
        ));
    }
    Ok(())
}

fn validate_package_file_name(file_name: &str) -> Result<(), McpToolError> {
    if file_name.is_empty() || file_name.len() > MAX_PACKAGE_FILE_NAME_BYTES {
        return Err(McpToolError::package(
            format!(
                "Package file names must contain between 1 and {MAX_PACKAGE_FILE_NAME_BYTES} UTF-8 bytes."
            ),
            None,
        ));
    }
    let path = Path::new(file_name);
    if path.file_name().and_then(|name| name.to_str()) != Some(file_name) {
        return Err(McpToolError::package(
            "Package tools accept one file name, not a filesystem path.",
            None,
        ));
    }
    validate_portable_path(file_name, "Package file name")
        .map_err(|message| McpToolError::package(message, None))?;
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("monogatari"))
    {
        return Err(McpToolError::package(
            "Package file names must use the `.monogatari` extension.",
            None,
        ));
    }
    Ok(())
}

fn canonical_regular_directory(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("Unable to inspect {label} `{}`: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "{label} must be a regular directory: {}",
            path.display()
        ));
    }
    path.canonicalize()
        .map_err(|error| format!("Unable to resolve {label} `{}`: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari-mcp-package-boundary-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn package_directory_stays_external_and_accepts_only_portable_package_names() {
        let root = temp_root("paths");
        let project = root.join("project");
        let output = root.join("packages");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(project.join("packages")).unwrap();
        std::fs::create_dir_all(&output).unwrap();
        let project = project.canonicalize().unwrap();

        assert!(PackageDirectoryBoundary::new(&project, project.join("packages")).is_err());
        let boundary = PackageDirectoryBoundary::new(&project, output.clone()).unwrap();
        assert!(boundary.package_path("story.monogatari").is_ok());
        assert!(boundary.package_path("../story.monogatari").is_err());
        assert!(boundary.package_path("story.zip").is_err());
        assert!(boundary.package_path("CON.monogatari").is_err());

        std::fs::remove_dir(&output).unwrap();
        std::fs::write(&output, b"rebound").unwrap();
        assert!(boundary.package_path("story.monogatari").is_err());

        std::fs::remove_dir_all(root).unwrap();
    }
}
