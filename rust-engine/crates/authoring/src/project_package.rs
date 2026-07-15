//! Transport-neutral `.monogatari` package manifest and portable-path contracts.

pub const ARCHIVE_MANIFEST_PATH: &str = "monogatari-project.json";

mod archive_reader;
mod archive_writer;
mod export;
mod manifest;
mod portable_path;

pub use archive_reader::{
    extract_project_package, inspect_project_package, ProjectPackageInspection,
};
pub use archive_writer::{
    write_project_package, ProjectPackageExportResult, ProjectPackageTargetPolicy,
};
pub use export::{
    build_project_export_manifest, project_export_settings_bytes, ProjectExportProvenance,
    ProjectExportRuntimeSnapshot, PROJECT_EXPORT_DIRECTORIES,
};
pub use manifest::{
    package_fingerprint, validate_manifest, validate_package_path_topology, ArchiveExportMetadata,
    ArchiveFileRecord, ArchiveManifest, ArchivePackage, ValidatedManifest, ARCHIVE_FORMAT,
    ARCHIVE_SCHEMA, MAX_ARCHIVE_COMPRESSED_BYTES, MAX_ARCHIVE_DIRECTORIES, MAX_ARCHIVE_FILES,
    MAX_ARCHIVE_FILE_BYTES, MAX_ARCHIVE_JSON_BYTES, MAX_ARCHIVE_MANIFEST_BYTES,
    MAX_ARCHIVE_TOTAL_BYTES, PACKAGE_FINGERPRINT_ALGORITHM,
};
pub use portable_path::{is_reserved_windows_segment, portable_case_key, validate_portable_path};
