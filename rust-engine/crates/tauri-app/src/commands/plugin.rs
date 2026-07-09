//! Plugin system scaffold for custom workflow node types.
//!
//! Allows game creators to register custom node types that extend the
//! workflow editor with new behaviors. Plugins are loaded from JSON
//! manifests in the project's plugins/ directory.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// A plugin manifest defining a custom workflow node type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_plugin_version")]
    pub version: String,
    #[serde(default = "default_plugin_author")]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, alias = "pluginType", alias = "plugin_type", alias = "type")]
    pub node_type: String,
    #[serde(default = "default_plugin_category")]
    pub category: String,
    #[serde(default)]
    pub configurable_fields: Vec<PluginField>,
    #[serde(default)]
    pub script_path: Option<String>,
    #[serde(default = "default_plugin_enabled")]
    pub enabled: bool,
}

/// A configurable field exposed by a plugin node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginField {
    pub name: String,
    pub field_type: String,
    pub label: String,
    pub default_value: serde_json::Value,
    pub required: bool,
}

fn default_plugin_version() -> String {
    "1.0.0".to_string()
}

fn default_plugin_author() -> String {
    "Local Creator".to_string()
}

fn default_plugin_category() -> String {
    "node".to_string()
}

fn default_plugin_enabled() -> bool {
    true
}

fn plugin_file_path(project_root: &Path, plugin_id: &str) -> Result<(String, PathBuf), String> {
    let id = normalize_plugin_id(plugin_id)?;
    let root = project_root.join("plugins");
    let path = root.join(format!("{id}.json"));

    if path.parent() != Some(root.as_path()) {
        return Err("Plugin file path must stay directly inside project plugins.".to_string());
    }

    Ok((id, path))
}

fn normalize_plugin_id(plugin_id: &str) -> Result<String, String> {
    let id = plugin_id.trim();
    if id.is_empty() || id.chars().any(char::is_control) {
        return Err("Plugin id is required and cannot contain control characters.".to_string());
    }
    if id.len() > 128 {
        return Err("Plugin id must be 128 characters or fewer.".to_string());
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(
            "Plugin ids can contain only ASCII letters, numbers, underscores, or hyphens."
                .to_string(),
        );
    }

    Ok(id.to_string())
}

fn normalize_plugin_manifest(mut manifest: PluginManifest) -> Result<PluginManifest, String> {
    let id_source = if manifest.id.trim().is_empty() {
        manifest.name.as_str()
    } else {
        manifest.id.as_str()
    };
    manifest.id = normalize_plugin_id(id_source)?;

    manifest.name = manifest.name.trim().to_string();
    if manifest.name.is_empty() {
        manifest.name = manifest.id.clone();
    }

    manifest.node_type = if manifest.node_type.trim().is_empty() {
        manifest.id.clone()
    } else {
        normalize_plugin_id(&manifest.node_type)?
    };

    manifest.version = manifest.version.trim().to_string();
    if manifest.version.is_empty() {
        manifest.version = default_plugin_version();
    }

    manifest.author = manifest.author.trim().to_string();
    if manifest.author.is_empty() {
        manifest.author = default_plugin_author();
    }

    manifest.description = manifest.description.trim().to_string();
    manifest.category = manifest.category.trim().to_string();
    if manifest.category.is_empty() {
        manifest.category = default_plugin_category();
    }

    Ok(manifest)
}

/// List all installed plugins.
#[tauri::command]
pub async fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginManifest>, String> {
    let project_root = state.current_project_data_root().await;
    let dir = project_root.join("plugins");
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(file_id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if normalize_plugin_id(file_id).is_err() {
            tracing::warn!("Skipping plugin with unsafe file name: {}", path.display());
            continue;
        }
        match std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|c| serde_json::from_str::<PluginManifest>(&c).map_err(|e| e.to_string()))
            .and_then(normalize_plugin_manifest)
        {
            Ok(manifest) if manifest.id == file_id => plugins.push(manifest),
            Ok(manifest) => tracing::warn!(
                "Skipping plugin {} because manifest id {} does not match the file name",
                path.display(),
                manifest.id
            ),
            Err(e) => tracing::warn!("Failed to load plugin {}: {}", path.display(), e),
        }
    }
    Ok(plugins)
}

/// Register a new plugin from a JSON manifest.
#[tauri::command]
pub async fn register_plugin(
    state: State<'_, AppState>,
    manifest: PluginManifest,
) -> Result<String, String> {
    let manifest = normalize_plugin_manifest(manifest)?;
    let project_root = state.current_project_data_root().await;
    let (id, path) = plugin_file_path(&project_root, &manifest.id)?;
    let dir = project_root.join("plugins");

    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;

    tracing::info!("Registered plugin: {} ({})", manifest.name, id);
    Ok(format!("Plugin {id} registered."))
}

/// Remove a plugin by id.
#[tauri::command]
pub async fn remove_plugin(
    state: State<'_, AppState>,
    plugin_id: String,
) -> Result<String, String> {
    let project_root = state.current_project_data_root().await;
    let (id, path) = plugin_file_path(&project_root, &plugin_id)?;
    if !path.exists() {
        return Err(format!("Plugin not found: {id}"));
    }
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(format!("Plugin {id} removed."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_file_paths_stay_inside_project_plugins() {
        let root = PathBuf::from("project-data");

        assert_eq!(
            plugin_file_path(&root, "score_gate").unwrap(),
            (
                "score_gate".to_string(),
                root.join("plugins").join("score_gate.json")
            )
        );
        assert_eq!(
            plugin_file_path(&root, " custom-node ").unwrap(),
            (
                "custom-node".to_string(),
                root.join("plugins").join("custom-node.json")
            )
        );
    }

    #[test]
    fn plugin_file_paths_reject_escape_attempts() {
        let root = PathBuf::from("project-data");
        for id in [
            "",
            "../settings",
            "plugins/custom",
            "plugins\\custom",
            "C:/Users/example/plugin",
            "https://example.test/plugin",
            ".",
            "..",
            "custom.json",
            "custom plugin",
            "custom!",
        ] {
            assert!(
                plugin_file_path(&root, id).is_err(),
                "{id} should be rejected"
            );
        }
    }

    #[test]
    fn plugin_manifest_normalization_fills_defaults_and_safe_ids() {
        let manifest = PluginManifest {
            id: " custom_node ".to_string(),
            name: " Custom Node ".to_string(),
            version: " ".to_string(),
            author: " ".to_string(),
            description: " Adds a branch node. ".to_string(),
            node_type: " ".to_string(),
            category: " ".to_string(),
            configurable_fields: Vec::new(),
            script_path: None,
            enabled: true,
        };

        let manifest = normalize_plugin_manifest(manifest).unwrap();

        assert_eq!(manifest.id, "custom_node");
        assert_eq!(manifest.name, "Custom Node");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.author, "Local Creator");
        assert_eq!(manifest.description, "Adds a branch node.");
        assert_eq!(manifest.node_type, "custom_node");
        assert_eq!(manifest.category, "node");
        assert!(manifest.enabled);
    }
}
