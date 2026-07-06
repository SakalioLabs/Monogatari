//! Asset loading and management.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tracing::debug;

use llm_core::Result;

/// Manages loading and caching of game assets.
pub struct AssetManager {
    base_path: PathBuf,
    cache: HashMap<String, Vec<u8>>,
}

impl AssetManager {
    /// Create a new asset manager with the given base path.
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            cache: HashMap::new(),
        }
    }

    /// Get the full path to an asset relative to the base path.
    pub fn asset_path(&self, relative: &str) -> PathBuf {
        self.base_path.join(relative)
    }

    /// Load a text file asset.
    pub async fn load_text(&self, relative: &str) -> Result<String> {
        let path = self.asset_path(relative);
        debug!("Loading text asset: {}", path.display());
        let content = tokio::fs::read_to_string(&path).await?;
        Ok(content)
    }

    /// Load a binary file asset.
    pub async fn load_bytes(&self, relative: &str) -> Result<Vec<u8>> {
        let path = self.asset_path(relative);
        debug!("Loading binary asset: {}", path.display());
        let content = tokio::fs::read(&path).await?;
        Ok(content)
    }

    /// Load a JSON file asset and deserialize it.
    pub async fn load_json<T: serde::de::DeserializeOwned>(&self, relative: &str) -> Result<T> {
        let content = self.load_text(relative).await?;
        let data: T = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// Load and cache an asset.
    pub async fn load_cached(&mut self, relative: &str) -> Result<&[u8]> {
        if !self.cache.contains_key(relative) {
            let bytes = self.load_bytes(relative).await?;
            self.cache.insert(relative.to_string(), bytes);
        }
        Ok(self.cache.get(relative).unwrap())
    }

    /// Check if an asset file exists.
    pub fn exists(&self, relative: &str) -> bool {
        self.asset_path(relative).exists()
    }

    /// List files in a directory relative to the base path.
    pub async fn list_directory(&self, relative: &str) -> Result<Vec<PathBuf>> {
        let dir = self.asset_path(relative);
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            files.push(entry.path());
        }
        Ok(files)
    }

    /// Clear the asset cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get the base path.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}
