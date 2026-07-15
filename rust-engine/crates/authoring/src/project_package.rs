//! Transport-neutral `.monogatari` package manifest and portable-path contracts.

pub const ARCHIVE_MANIFEST_PATH: &str = "monogatari-project.json";

mod manifest;
mod portable_path;

pub use manifest::{
    package_fingerprint, validate_manifest, validate_package_path_topology, ArchiveExportMetadata,
    ArchiveFileRecord, ArchiveManifest, ArchivePackage, ValidatedManifest, ARCHIVE_FORMAT,
    ARCHIVE_SCHEMA, MAX_ARCHIVE_FILE_BYTES, PACKAGE_FINGERPRINT_ALGORITHM,
};
pub use portable_path::{is_reserved_windows_segment, portable_case_key, validate_portable_path};
