//! Template marketplace scaffold.
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
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

fn marketplace_catalog() -> Vec<MarketplaceEntry> {
    vec![
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
    ]
}

fn marketplace_catalog_manifest(template_id: &str) -> Option<ExportManifest> {
    marketplace_catalog()
        .into_iter()
        .find(|entry| entry.id == template_id)
        .map(|entry| ExportManifest {
            name: entry.name,
            description: entry.description,
            author: entry.author,
            version: entry.version,
            entry_type: entry.entry_type,
            files: Vec::new(),
            tags: entry.tags,
        })
}

fn template_dir_in_project(
    project_root: &Path,
    template_ref: &str,
) -> Result<(String, PathBuf), String> {
    let mut segments = normalize_template_ref(template_ref)?;
    if segments.first().map(String::as_str) == Some("templates") {
        segments.remove(0);
    }
    if segments.is_empty() {
        return Err("Marketplace template references must name a template directory.".to_string());
    }

    let root = project_root.join("templates");
    let path = segments
        .iter()
        .fold(root.clone(), |path, segment| path.join(segment));

    if !path.starts_with(&root) {
        return Err("Marketplace template path must stay inside project templates.".to_string());
    }

    Ok((segments.join("/"), path))
}

fn normalize_template_ref(template_ref: &str) -> Result<Vec<String>, String> {
    let normalized = template_ref.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.chars().any(char::is_control) {
        return Err(
            "Marketplace template references must be non-empty and cannot contain control characters."
                .to_string(),
        );
    }
    if normalized.contains(':') {
        return Err(
            "Marketplace template references cannot contain drive prefixes or URI schemes."
                .to_string(),
        );
    }

    let segments = normalized.split('/').collect::<Vec<_>>();
    if segments
        .iter()
        .any(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    {
        return Err(
            "Marketplace template references cannot contain empty, current, or parent directory segments."
                .to_string(),
        );
    }
    if segments.iter().any(|segment| {
        !segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    }) {
        return Err(
            "Marketplace template references can contain only ASCII letters, numbers, underscores, hyphens, or separators."
                .to_string(),
        );
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
        return Err(
            "Marketplace template references must be relative to project templates.".to_string(),
        );
    }

    Ok(segments.into_iter().map(str::to_string).collect())
}

fn export_template_to_project(
    project_root: &Path,
    manifest: &ExportManifest,
    output_path: &str,
) -> Result<PathBuf, String> {
    let (_, output) = template_dir_in_project(project_root, output_path)?;
    std::fs::create_dir_all(&output).map_err(|e| e.to_string())?;
    let manifest_path = output.join("manifest.json");
    let json = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    std::fs::write(&manifest_path, json).map_err(|e| e.to_string())?;
    Ok(manifest_path)
}

fn import_template_from_project(
    project_root: &Path,
    template_path: &str,
) -> Result<ExportManifest, String> {
    let (template_id, src) = template_dir_in_project(project_root, template_path)?;
    let manifest_path = src.join("manifest.json");
    if manifest_path.exists() {
        let content = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        return serde_json::from_str(&content).map_err(|e| e.to_string());
    }

    marketplace_catalog_manifest(&template_id)
        .ok_or_else(|| format!("No marketplace template manifest found for {template_id}"))
}

#[tauri::command]
pub async fn list_marketplace_entries(
    _state: State<'_, AppState>,
    _entry_type: Option<String>,
) -> Result<Vec<MarketplaceEntry>, String> {
    Ok(marketplace_catalog())
}

#[tauri::command]
pub async fn export_template(
    state: State<'_, AppState>,
    manifest: ExportManifest,
    output_path: String,
) -> Result<String, String> {
    let project_root = state.current_project_data_root().await;
    let manifest_path = export_template_to_project(&project_root, &manifest, &output_path)?;
    Ok(format!("Exported template to {}", manifest_path.display()))
}

#[tauri::command]
pub async fn import_template(
    state: State<'_, AppState>,
    template_path: String,
) -> Result<String, String> {
    let project_root = state.current_project_data_root().await;
    let manifest = import_template_from_project(&project_root, &template_path)?;
    Ok(format!("Imported template '{}'", manifest.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_marketplace_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn sample_manifest() -> ExportManifest {
        ExportManifest {
            name: "Sample Template".to_string(),
            description: "A test export.".to_string(),
            author: "Monogatari".to_string(),
            version: "1.0.0".to_string(),
            entry_type: "workflow".to_string(),
            files: vec!["workflows/sample.json".to_string()],
            tags: vec!["test".to_string()],
        }
    }

    #[test]
    fn marketplace_template_dirs_resolve_under_project_templates() {
        let root = PathBuf::from("project-data");

        assert_eq!(
            template_dir_in_project(&root, "sakura_demo").unwrap(),
            (
                "sakura_demo".to_string(),
                root.join("templates").join("sakura_demo")
            )
        );
        assert_eq!(
            template_dir_in_project(&root, "templates/luna_stargazing").unwrap(),
            (
                "luna_stargazing".to_string(),
                root.join("templates").join("luna_stargazing")
            )
        );
        assert_eq!(
            template_dir_in_project(&root, "packs/story_demo").unwrap(),
            (
                "packs/story_demo".to_string(),
                root.join("templates").join("packs").join("story_demo")
            )
        );
    }

    #[test]
    fn marketplace_template_dirs_reject_escape_attempts() {
        let root = PathBuf::from("project-data");
        for template_ref in [
            "",
            "../settings",
            "templates/../settings",
            "templates//sakura_demo",
            "templates/./sakura_demo",
            "C:/Users/example/template",
            "https://example.test/template",
            "/tmp/template",
            "template.json",
            "template pack",
            "template!",
        ] {
            assert!(
                template_dir_in_project(&root, template_ref).is_err(),
                "{template_ref} should be rejected"
            );
        }
    }

    #[test]
    fn export_template_writes_manifest_inside_project_templates() {
        let root = temp_root("export_scope");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("settings.json"), "keep me").unwrap();

        let manifest_path =
            export_template_to_project(&root, &sample_manifest(), "templates/sample_demo").unwrap();

        assert_eq!(
            manifest_path,
            root.join("templates")
                .join("sample_demo")
                .join("manifest.json")
        );
        assert_eq!(
            std::fs::read_to_string(root.join("settings.json")).unwrap(),
            "keep me"
        );
        assert!(export_template_to_project(&root, &sample_manifest(), "../settings").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn import_template_reads_project_manifest_or_catalog_entry() {
        let root = temp_root("import_scope");
        let template_dir = root.join("templates").join("sample_demo");
        std::fs::create_dir_all(&template_dir).unwrap();
        std::fs::write(
            template_dir.join("manifest.json"),
            serde_json::to_string_pretty(&sample_manifest()).unwrap(),
        )
        .unwrap();

        let imported = import_template_from_project(&root, "sample_demo").unwrap();
        assert_eq!(imported.name, "Sample Template");

        let catalog = import_template_from_project(&root, "sakura_demo").unwrap();
        assert_eq!(catalog.name, "Sakura Park Demo");

        assert!(import_template_from_project(&root, "../settings").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
