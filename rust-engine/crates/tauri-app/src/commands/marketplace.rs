//! Template marketplace scaffold.
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub entry_type: String,
    pub tags: Vec<String>,
    pub download_count: u32,
    pub rating: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportManifest {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub entry_type: String,
    pub files: Vec<String>,
    pub tags: Vec<String>,
}

#[tauri::command]
pub async fn list_marketplace_entries(
    _state: State<'_, AppState>,
    _entry_type: Option<String>,
) -> Result<Vec<MarketplaceEntry>, String> {
    Ok(vec![
        MarketplaceEntry {
            id: "sakura_demo".to_string(),
            name: "Sakura Park Demo".to_string(),
            description:
                "A complete demo with Sakura character, park scene, and cherry blossom dialogue"
                    .to_string(),
            author: "Monogatari".to_string(),
            version: "1.0.0".to_string(),
            entry_type: "full_project".to_string(),
            tags: vec![
                "demo".to_string(),
                "romance".to_string(),
                "nature".to_string(),
            ],
            download_count: 0,
            rating: 5.0,
        },
        MarketplaceEntry {
            id: "luna_stargazing".to_string(),
            name: "Luna Stargazing".to_string(),
            description: "Luna character with observatory scene and constellation dialogue"
                .to_string(),
            author: "Monogatari".to_string(),
            version: "1.0.0".to_string(),
            entry_type: "character".to_string(),
            tags: vec![
                "sci-fi".to_string(),
                "poetic".to_string(),
                "night".to_string(),
            ],
            download_count: 0,
            rating: 4.5,
        },
    ])
}

#[tauri::command]
pub async fn export_template(
    _state: State<'_, AppState>,
    manifest: ExportManifest,
    output_path: String,
) -> Result<String, String> {
    let output = std::path::PathBuf::from(&output_path);
    std::fs::create_dir_all(&output).map_err(|e| e.to_string())?;
    let manifest_path = output.join("manifest.json");
    let json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(&manifest_path, json).map_err(|e| e.to_string())?;
    Ok(format!("Exported template to {}", output.display()))
}

#[tauri::command]
pub async fn import_template(
    _state: State<'_, AppState>,
    template_path: String,
) -> Result<String, String> {
    let src = std::path::PathBuf::from(&template_path);
    let manifest_path = src.join("manifest.json");
    if !manifest_path.exists() {
        return Err("No manifest.json found".to_string());
    }
    let content = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let manifest: ExportManifest = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(format!("Imported template '{}'", manifest.name))
}
