//! Plugin system scaffold for custom workflow node types.
//!
//! Allows game creators to register custom node types that extend the
//! workflow editor with new behaviors. Plugins are loaded from JSON
//! manifests in the project's plugins/ directory.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// A plugin manifest defining a custom workflow node type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub node_type: String,
    pub category: String,
    pub configurable_fields: Vec<PluginField>,
    pub script_path: Option<String>,
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

/// List all installed plugins.
#[tauri::command]
pub async fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginManifest>, String> {
    let root = state.project_path.read().await;
    let plugin_dir = root.as_ref().map(|p| p.join("plugins"));
    let Some(dir) = plugin_dir.filter(|d| d.exists()) else {
        return Ok(Vec::new());
    };

    let mut plugins = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        match std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|c| serde_json::from_str::<PluginManifest>(&c).map_err(|e| e.to_string()))
        {
            Ok(manifest) => plugins.push(manifest),
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
    if manifest.id.trim().is_empty() {
        return Err("Plugin id is required.".to_string());
    }
    if manifest.node_type.trim().is_empty() {
        return Err("Plugin node_type is required.".to_string());
    }

    let root = state.project_path.read().await;
    let plugin_dir = root.as_ref().map(|p| p.join("plugins"));
    let Some(dir) = plugin_dir else {
        return Err("No project path configured.".to_string());
    };

    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", manifest.id));
    let json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;

    tracing::info!("Registered plugin: {} ({})", manifest.name, manifest.id);
    Ok(format!("Plugin {} registered.", manifest.id))
}

/// Remove a plugin by id.
#[tauri::command]
pub async fn remove_plugin(
    state: State<'_, AppState>,
    plugin_id: String,
) -> Result<String, String> {
    let root = state.project_path.read().await;
    let plugin_dir = root.as_ref().map(|p| p.join("plugins"));
    let Some(dir) = plugin_dir else {
        return Err("No project path configured.".to_string());
    };

    let path = dir.join(format!("{plugin_id}.json"));
    if !path.exists() {
        return Err(format!("Plugin not found: {plugin_id}"));
    }
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(format!("Plugin {plugin_id} removed."))
}