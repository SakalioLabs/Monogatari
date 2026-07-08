//! Internationalization (i18n) scaffold.
//!
//! Provides commands for loading locale data and translating UI strings.
//! Creators can supply locale JSON files in a `locales/` directory.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleData {
    pub locale: String,
    pub strings: HashMap<String, String>,
}

/// Load a locale file from the project locales directory.
#[tauri::command]
pub async fn load_locale(state: State<'_, AppState>, locale: String) -> Result<LocaleData, String> {
    let project = state.project_path.read().await;
    let base = project.as_ref().ok_or("No project path")?;
    let path = base.join("locales").join(format!("{locale}.json"));
    if !path.exists() {
        return Ok(LocaleData {
            locale,
            strings: HashMap::new(),
        });
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

/// List available locales.
#[tauri::command]
pub async fn list_locales(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let project = state.project_path.read().await;
    let base = project.as_ref().ok_or("No project path")?;
    let dir = base.join("locales");
    if !dir.exists() {
        return Ok(vec!["en".into()]);
    }
    let mut locales = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if let Some(name) = entry.path().file_stem().and_then(|n| n.to_str()) {
            locales.push(name.to_string());
        }
    }
    if locales.is_empty() {
        locales.push("en".into());
    }
    Ok(locales)
}

/// Translate a key using the loaded locale.
#[tauri::command]
pub async fn translate(
    state: State<'_, AppState>,
    key: String,
    locale: Option<String>,
) -> Result<String, String> {
    let loc = locale.unwrap_or_else(|| "en".into());
    let project = state.project_path.read().await;
    let base = project.as_ref().ok_or("No project path")?;
    let path = base.join("locales").join(format!("{loc}.json"));
    if !path.exists() {
        return Ok(key);
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let data: LocaleData = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(data.strings.get(&key).cloned().unwrap_or(key))
}
