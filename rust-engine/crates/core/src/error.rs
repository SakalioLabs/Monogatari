//! Error types for the engine.
//!
//! Provides a unified error type with rich context for debugging.

use thiserror::Error;

/// Main error type for the engine.
///
/// Each variant includes context about where and why the error occurred.
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Service not found: {0}. Did you register it before use?")]
    ServiceNotFound(String),

    #[error("Service already registered: {0}. Cannot register twice.")]
    ServiceAlreadyRegistered(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Scene error: {0}")]
    Scene(String),

    #[error("Dialogue error: {0}")]
    Dialogue(String),

    #[error("AI inference error: {0}")]
    Inference(String),

    #[error("Script error: {0}")]
    Script(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenience Result type.
pub type Result<T> = std::result::Result<T, EngineError>;

/// Extension trait for adding context to Results.
pub trait ResultExt<T> {
    /// Add context to an error.
    fn with_context(self, context: impl Into<String>) -> Result<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for std::result::Result<T, E> {
    fn with_context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| EngineError::Other(anyhow::anyhow!("{}: {}", context.into(), e)))
    }
}

/// Helper functions for creating specific errors with context.
impl EngineError {
    /// Create an asset not found error.
    pub fn asset_not_found(name: impl std::fmt::Display, expected_path: impl std::fmt::Display) -> Self {
        Self::AssetNotFound(format!("{name} at {expected_path}"))
    }

    /// Create a scene error.
    pub fn scene(scene: impl std::fmt::Display, message: impl std::fmt::Display) -> Self {
        Self::Scene(format!("[{scene}] {message}"))
    }

    /// Create a dialogue error.
    pub fn dialogue(script: impl std::fmt::Display, node: impl std::fmt::Display, message: impl std::fmt::Display) -> Self {
        Self::Dialogue(format!("[{script}/{node}] {message}"))
    }

    /// Create an inference error.
    pub fn inference(engine: impl std::fmt::Display, message: impl std::fmt::Display) -> Self {
        Self::Inference(format!("[{engine}] {message}"))
    }

    /// Create a script error with location.
    pub fn script(message: impl std::fmt::Display, line: usize, column: usize) -> Self {
        Self::Script(format!("{message} (line {line}, col {column})"))
    }

    /// Create a config error.
    pub fn config(key: impl std::fmt::Display, message: impl std::fmt::Display) -> Self {
        Self::Config(format!("[{key}] {message}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EngineError::dialogue("main", "start", "Node not found");
        assert!(err.to_string().contains("main"));
        assert!(err.to_string().contains("start"));
        assert!(err.to_string().contains("Node not found"));
    }

    #[test]
    fn test_error_helpers() {
        let err = EngineError::inference("API", "Connection timeout");
        assert!(err.to_string().contains("API"));
        assert!(err.to_string().contains("Connection timeout"));
    }

    #[test]
    fn test_result_ext() {
        let result: std::result::Result<(), String> = Err("test error".to_string());
        let with_ctx = result.with_context("Loading file");
        assert!(with_ctx.unwrap_err().to_string().contains("Loading file"));
    }
}
