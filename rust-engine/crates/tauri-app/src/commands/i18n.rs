//! Internationalization (i18n) scaffold.
//!
//! Provides commands for loading locale data and translating UI strings.
//! Creators can supply locale JSON files in a `locales/` directory.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleData {
    pub locale: String,
    pub strings: HashMap<String, String>,
}

fn locale_file_path(project_root: &Path, locale: &str) -> Result<(String, PathBuf), String> {
    let locale_id = normalize_locale_id(locale)?;
    let root = project_root.join("locales");
    let path = root.join(format!("{locale_id}.json"));

    if path.parent() != Some(root.as_path()) {
        return Err("Locale file path must stay directly inside project locales.".to_string());
    }

    Ok((locale_id, path))
}

fn normalize_locale_id(locale: &str) -> Result<String, String> {
    let id = locale.trim();
    if id.is_empty() || id.chars().any(char::is_control) {
        return Err("Locale id is required and cannot contain control characters.".to_string());
    }
    if id.len() > 32 {
        return Err("Locale id must be 32 characters or fewer.".to_string());
    }

    let segments = id.split('-').collect::<Vec<_>>();
    if segments.iter().any(|segment| segment.is_empty()) {
        return Err("Locale ids cannot contain empty segments.".to_string());
    }
    if segments
        .iter()
        .any(|segment| !segment.chars().all(|ch| ch.is_ascii_alphanumeric()))
    {
        return Err(
            "Locale ids can contain only ASCII letters, numbers, and hyphen separators."
                .to_string(),
        );
    }

    Ok(id.to_string())
}

fn load_locale_from_project(project_root: &Path, locale: &str) -> Result<LocaleData, String> {
    let (locale_id, path) = locale_file_path(project_root, locale)?;
    if !path.exists() {
        return Ok(LocaleData {
            locale: locale_id,
            strings: HashMap::new(),
        });
    }

    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let data: LocaleData = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(LocaleData {
        locale: locale_id,
        strings: data.strings,
    })
}

fn list_locale_ids(project_root: &Path) -> Result<Vec<String>, String> {
    let dir = project_root.join("locales");
    if !dir.exists() {
        return Ok(vec!["en".into()]);
    }

    let mut locales = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !entry.file_type().map_err(|e| e.to_string())?.is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("json")
        {
            continue;
        }
        if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
            if let Ok(locale_id) = normalize_locale_id(name) {
                locales.push(locale_id);
            }
        }
    }
    locales.sort();
    locales.dedup();
    if locales.is_empty() {
        locales.push("en".into());
    }
    Ok(locales)
}

fn translate_from_project(
    project_root: &Path,
    key: String,
    locale: Option<String>,
) -> Result<String, String> {
    let loc = locale.unwrap_or_else(|| "en".into());
    let data = load_locale_from_project(project_root, &loc)?;
    Ok(data.strings.get(&key).cloned().unwrap_or(key))
}

/// Load a locale file from the project locales directory.
#[tauri::command]
pub async fn load_locale(state: State<'_, AppState>, locale: String) -> Result<LocaleData, String> {
    let project_root = state.current_project_data_root().await;
    load_locale_from_project(&project_root, &locale)
}

/// List available locales.
#[tauri::command]
pub async fn list_locales(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let project_root = state.current_project_data_root().await;
    list_locale_ids(&project_root)
}

/// Translate a key using the loaded locale.
#[tauri::command]
pub async fn translate(
    state: State<'_, AppState>,
    key: String,
    locale: Option<String>,
) -> Result<String, String> {
    let project_root = state.current_project_data_root().await;
    translate_from_project(&project_root, key, locale)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_i18n_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn locale_file_paths_stay_inside_project_locales() {
        let root = PathBuf::from("project-data");

        assert_eq!(
            locale_file_path(&root, "zh-CN").unwrap(),
            ("zh-CN".to_string(), root.join("locales").join("zh-CN.json"))
        );
        assert_eq!(
            locale_file_path(&root, " en ").unwrap(),
            ("en".to_string(), root.join("locales").join("en.json"))
        );
    }

    #[test]
    fn locale_file_paths_reject_escape_attempts() {
        let root = PathBuf::from("project-data");
        for locale in [
            "",
            "../settings",
            "locales/en",
            "locales\\en",
            "C:/Users/example/en",
            "https://example.test/en",
            "en.json",
            "en..",
            "-en",
            "en-",
            "en--US",
            "en US",
            "en_US",
            "en!",
        ] {
            assert!(
                locale_file_path(&root, locale).is_err(),
                "{locale} should be rejected"
            );
        }
    }

    #[test]
    fn locale_loading_lists_and_translates_safe_locale_ids() {
        let root = temp_root("load");
        let locale_dir = root.join("locales");
        std::fs::create_dir_all(&locale_dir).unwrap();
        std::fs::write(
            locale_dir.join("en.json"),
            r#"{"locale":"en","strings":{"hello":"Hello"}}"#,
        )
        .unwrap();
        std::fs::write(
            locale_dir.join("zh-CN.json"),
            r#"{"locale":"zh-CN","strings":{"hello":"\u4f60\u597d"}}"#,
        )
        .unwrap();
        std::fs::write(
            locale_dir.join("..evil.json"),
            r#"{"locale":"..evil","strings":{"hello":"bad"}}"#,
        )
        .unwrap();
        std::fs::create_dir_all(locale_dir.join("ja-JP.json")).unwrap();

        assert_eq!(
            list_locale_ids(&root).unwrap(),
            vec!["en".to_string(), "zh-CN".to_string()]
        );

        let zh = load_locale_from_project(&root, "zh-CN").unwrap();
        assert_eq!(zh.locale, "zh-CN");
        assert_eq!(
            zh.strings.get("hello").map(String::as_str),
            Some("\u{4f60}\u{597d}")
        );
        assert_eq!(
            translate_from_project(&root, "hello".to_string(), Some("en".to_string())).unwrap(),
            "Hello"
        );
        assert!(load_locale_from_project(&root, "../settings").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
