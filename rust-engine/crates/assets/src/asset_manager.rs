//! Asset loading and management.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use tracing::debug;

use llm_core::{EngineError, Result};

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
    pub fn asset_path(&self, relative: &str) -> Result<PathBuf> {
        self.safe_asset_path(relative)
    }

    /// Load a text file asset.
    pub async fn load_text(&self, relative: &str) -> Result<String> {
        let path = self.asset_path(relative)?;
        debug!("Loading text asset: {}", path.display());
        let content = tokio::fs::read_to_string(&path).await?;
        Ok(content)
    }

    /// Load a binary file asset.
    pub async fn load_bytes(&self, relative: &str) -> Result<Vec<u8>> {
        let path = self.asset_path(relative)?;
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
        self.asset_path(relative)
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    /// List files in a directory relative to the base path.
    pub async fn list_directory(&self, relative: &str) -> Result<Vec<PathBuf>> {
        let dir = self.asset_path(relative)?;
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

    fn safe_asset_path(&self, relative: &str) -> Result<PathBuf> {
        let relative_path = normalize_asset_relative_path(relative)?;
        let root = self.base_path.clone();
        let path = root.join(relative_path);

        if !path.starts_with(&root) {
            return Err(asset_path_error(
                "Asset path must stay inside the asset root.",
            ));
        }

        Ok(path)
    }
}

fn normalize_asset_relative_path(relative: &str) -> Result<PathBuf> {
    let normalized = relative.replace('\\', "/");
    if normalized.is_empty() || normalized.chars().any(char::is_control) {
        return Err(asset_path_error(
            "Asset paths must be non-empty and cannot contain control characters.",
        ));
    }
    if normalized.contains(':') {
        return Err(asset_path_error(
            "Asset paths cannot contain drive prefixes or URI schemes.",
        ));
    }
    if normalized
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(asset_path_error(
            "Asset paths cannot contain empty, current, or parent directory segments.",
        ));
    }

    let path = Path::new(&normalized);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err(asset_path_error(
            "Asset paths must be relative to the asset root.",
        ));
    }

    Ok(path.to_path_buf())
}

fn asset_path_error(message: &str) -> EngineError {
    EngineError::config("asset_path", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_asset_manager_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[tokio::test]
    async fn load_text_rejects_paths_that_escape_asset_root() {
        let root = temp_root("load_text_rejects_escape");
        let assets_dir = root.join("assets");
        std::fs::create_dir_all(&assets_dir).unwrap();
        let outside = root.join("settings.json");
        std::fs::write(&outside, "keep me").unwrap();
        let manager = AssetManager::new(&assets_dir);

        let error = manager.load_text("../settings.json").await.unwrap_err();

        assert!(error.to_string().contains("asset_path"));
        assert_eq!(std::fs::read_to_string(&outside).unwrap(), "keep me");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn load_bytes_rejects_absolute_asset_paths() {
        let root = temp_root("load_bytes_rejects_absolute");
        let assets_dir = root.join("assets");
        std::fs::create_dir_all(&assets_dir).unwrap();
        let outside = root.join("secret.bin");
        std::fs::write(&outside, b"secret").unwrap();
        let manager = AssetManager::new(&assets_dir);

        let error = manager
            .load_bytes(outside.to_string_lossy().as_ref())
            .await
            .unwrap_err();

        assert!(error.to_string().contains("asset_path"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn list_directory_rejects_parent_traversal() {
        let root = temp_root("list_rejects_escape");
        let assets_dir = root.join("assets");
        std::fs::create_dir_all(&assets_dir).unwrap();
        let manager = AssetManager::new(&assets_dir);

        let error = manager.list_directory("../").await.unwrap_err();

        assert!(error.to_string().contains("asset_path"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn loads_nested_project_asset_paths() {
        let root = temp_root("loads_nested_paths");
        let assets_dir = root.join("assets");
        let characters_dir = assets_dir.join("characters");
        std::fs::create_dir_all(&characters_dir).unwrap();
        std::fs::write(characters_dir.join("sakura.txt"), "hello").unwrap();
        let manager = AssetManager::new(&assets_dir);

        let content = manager.load_text("characters\\sakura.txt").await.unwrap();

        assert_eq!(content, "hello");
        assert!(manager.exists("characters/sakura.txt"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn exists_returns_false_for_invalid_asset_paths() {
        let manager = AssetManager::new("assets");

        assert!(!manager.exists("../settings.json"));
        assert!(!manager.exists("https://example.test/sprite.png"));
    }
}
