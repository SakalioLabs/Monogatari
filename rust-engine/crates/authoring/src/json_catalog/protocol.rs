//! Serializable JSON catalog models and stable errors.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const JSON_CATALOG_REPORT_SCHEMA_V1: &str = "monogatari-json-catalog-report/v1";
pub const JSON_CATALOG_DOCUMENT_SCHEMA_V1: &str = "monogatari-json-catalog-document/v1";
pub const JSON_CATALOG_ERROR_SCHEMA_V1: &str = "monogatari-json-catalog-error/v1";
pub const MAX_AUTHORABLE_JSON_BYTES: u64 = 4 * 1024 * 1024;
pub const AUTHORABLE_JSON_CATALOG_ROOTS: &[&str] = &[
    "assets",
    "characters",
    "dialogue",
    "endings",
    "events",
    "knowledge",
    "locales",
    "quality_suites",
    "roleplays",
    "scenes",
    "workflows",
];

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum AuthorableJsonCatalog {
    Assets,
    Characters,
    Dialogue,
    Endings,
    Events,
    Knowledge,
    Locales,
    QualitySuites,
    Roleplays,
    Scenes,
    Workflows,
}

impl AuthorableJsonCatalog {
    pub const ALL: [Self; 11] = [
        Self::Assets,
        Self::Characters,
        Self::Dialogue,
        Self::Endings,
        Self::Events,
        Self::Knowledge,
        Self::Locales,
        Self::QualitySuites,
        Self::Roleplays,
        Self::Scenes,
        Self::Workflows,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Assets => "assets",
            Self::Characters => "characters",
            Self::Dialogue => "dialogue",
            Self::Endings => "endings",
            Self::Events => "events",
            Self::Knowledge => "knowledge",
            Self::Locales => "locales",
            Self::QualitySuites => "quality_suites",
            Self::Roleplays => "roleplays",
            Self::Scenes => "scenes",
            Self::Workflows => "workflows",
        }
    }

    pub(super) fn from_root(root: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|catalog| catalog.as_str() == root)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JsonAcceptanceLevel {
    Document,
    CoreRuntime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JsonDocumentKind {
    Object,
    Array,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JsonCatalogIssueSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JsonCatalogErrorCode {
    ProjectRootInvalid,
    InvalidPath,
    PathNotAllowed,
    PathCaseCollision,
    CatalogMissing,
    SymlinkNotAllowed,
    NotRegularFile,
    FileNotFound,
    FileTooLarge,
    InvalidJson,
    InvalidDocument,
    CatalogLimitExceeded,
    IoFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JsonCatalogError {
    pub schema: String,
    pub code: JsonCatalogErrorCode,
    pub message: String,
    pub path: Option<String>,
}

impl JsonCatalogError {
    pub(super) fn new(
        code: JsonCatalogErrorCode,
        message: impl Into<String>,
        path: Option<String>,
    ) -> Self {
        Self {
            schema: JSON_CATALOG_ERROR_SCHEMA_V1.to_string(),
            code,
            message: message.into(),
            path,
        }
    }
}

impl fmt::Display for JsonCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for JsonCatalogError {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JsonCatalogIssue {
    pub severity: JsonCatalogIssueSeverity,
    pub code: JsonCatalogErrorCode,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JsonCatalogEntry {
    pub catalog: AuthorableJsonCatalog,
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub content_fingerprint: String,
    pub document_kind: JsonDocumentKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JsonCatalogStatus {
    pub catalog: AuthorableJsonCatalog,
    pub exists: bool,
    pub document_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JsonCatalogReport {
    pub schema: String,
    pub acceptance_level: JsonAcceptanceLevel,
    pub valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub document_count: usize,
    pub total_bytes: u64,
    pub catalogs: Vec<JsonCatalogStatus>,
    pub documents: Vec<JsonCatalogEntry>,
    pub issues: Vec<JsonCatalogIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct JsonCatalogDocument {
    pub schema: String,
    pub acceptance_level: JsonAcceptanceLevel,
    pub metadata: JsonCatalogEntry,
    pub document: Value,
}
