//! Scene asset catalog commands for background/scene production workflows.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::{default_project_data_root, AppState};
use crate::story_access::{
    ensure_story_content_access, story_content_access, StoryContentAccessEntry, StoryContentKind,
};

const BACKGROUND_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "gif", "svg"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneInfo {
    pub id: String,
    pub name: String,
    #[serde(default, alias = "backgroundPath")]
    pub background_path: Option<String>,
    #[serde(default, alias = "bgmPath")]
    pub bgm_path: Option<String>,
    #[serde(default)]
    pub weather: Option<String>,
    #[serde(default)]
    pub time_of_day: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_scene_source")]
    pub source: String,
    #[serde(default)]
    pub background_exists: bool,
    #[serde(default)]
    pub absolute_background_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorySceneInfo {
    #[serde(flatten)]
    pub scene: SceneInfo,
    pub access: StoryContentAccessEntry,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackgroundAsset {
    pub id: String,
    pub file_name: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub extension: String,
    pub file_size: u64,
    pub linked_scene_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneAssetIssue {
    pub severity: String,
    pub code: String,
    pub scene_id: Option<String>,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneAssetCatalog {
    pub project_path: Option<String>,
    pub valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub scenes: Vec<SceneInfo>,
    pub backgrounds: Vec<BackgroundAsset>,
    pub issues: Vec<SceneAssetIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveScene {
    pub scene: Option<SceneInfo>,
    pub scene_history: Vec<String>,
}

/// List scene metadata and background files from the active project.
#[tauri::command]
pub async fn list_scene_assets(state: State<'_, AppState>) -> Result<SceneAssetCatalog, String> {
    let root = project_root(&state).await?;
    build_scene_asset_catalog(&root)
}

/// List project scenes with the event-derived runtime access decision for each scene.
#[tauri::command]
pub async fn list_story_scenes(state: State<'_, AppState>) -> Result<Vec<StorySceneInfo>, String> {
    let root = project_root(&state).await?;
    let scene_catalog = build_scene_asset_catalog(&root)?;
    let event_catalog = state.story_event_catalog.read().await;
    let progress = state.story_progress.read().await;
    Ok(scene_catalog
        .scenes
        .into_iter()
        .map(|scene| StorySceneInfo {
            access: story_content_access(
                &event_catalog,
                &progress,
                StoryContentKind::Scene,
                &scene.id,
            ),
            scene,
        })
        .collect())
}

/// Get the currently active scene selected by the authoring UI/runtime.
#[tauri::command]
pub async fn get_current_scene(state: State<'_, AppState>) -> Result<ActiveScene, String> {
    let root = project_root(&state).await?;
    let catalog = build_scene_asset_catalog(&root)?;
    let active_scene_id = state.active_scene_id.read().await.clone();
    let history = state.scene_history.read().await.clone();
    let scene = active_scene_id
        .as_deref()
        .and_then(|id| catalog.scenes.iter().find(|scene| scene.id == id))
        .cloned();

    Ok(ActiveScene {
        scene,
        scene_history: history,
    })
}

/// Set the active scene. Existing scene ids are resolved through the catalog.
#[tauri::command]
pub async fn set_scene(
    state: State<'_, AppState>,
    scene_id: String,
    name: Option<String>,
    background_path: Option<String>,
    bgm_path: Option<String>,
) -> Result<SceneInfo, String> {
    set_scene_inner(&state, scene_id, name, background_path, bgm_path).await
}

/// Enter a project scene through the player runtime, enforcing story unlocks.
#[tauri::command]
pub async fn enter_story_scene(
    state: State<'_, AppState>,
    scene_id: String,
) -> Result<SceneInfo, String> {
    {
        let catalog = state.story_event_catalog.read().await;
        let progress = state.story_progress.read().await;
        ensure_story_content_access(&catalog, &progress, StoryContentKind::Scene, &scene_id)?;
    }
    set_scene_inner(&state, scene_id, None, None, None).await
}

pub(crate) async fn set_scene_inner(
    state: &AppState,
    scene_id: String,
    name: Option<String>,
    background_path: Option<String>,
    bgm_path: Option<String>,
) -> Result<SceneInfo, String> {
    if scene_id.trim().is_empty() {
        return Err("scene_id is required".to_string());
    }

    let root = project_root_inner(state).await?;
    let catalog = build_scene_asset_catalog(&root)?;
    let mut scene = catalog
        .scenes
        .iter()
        .find(|scene| scene.id == scene_id)
        .cloned()
        .unwrap_or_else(|| SceneInfo {
            id: scene_id.clone(),
            name: name.clone().unwrap_or_else(|| humanize_id(&scene_id)),
            background_path: background_path.clone(),
            bgm_path: bgm_path.clone(),
            weather: None,
            time_of_day: None,
            tags: Vec::new(),
            source: "runtime".to_string(),
            background_exists: false,
            absolute_background_path: None,
        });

    if let Some(custom_name) = name {
        if !custom_name.trim().is_empty() {
            scene.name = custom_name;
        }
    }
    if let Some(custom_background) = background_path {
        scene.background_path = Some(custom_background);
    }
    if let Some(custom_bgm) = bgm_path {
        scene.bgm_path = Some(custom_bgm);
    }

    attach_background_status(&root, &mut scene)?;
    if scene.background_path.is_some() && !scene.background_exists {
        return Err(format!("Scene background is missing for `{}`", scene.id));
    }

    *state.active_scene_id.write().await = Some(scene.id.clone());
    let mut history = state.scene_history.write().await;
    if history.last() != Some(&scene.id) {
        history.push(scene.id.clone());
    }
    if history.len() > 24 {
        let overflow = history.len() - 24;
        history.drain(0..overflow);
    }

    Ok(scene)
}

pub(crate) async fn resolve_project_scene(
    state: &AppState,
    scene_id: &str,
) -> Result<SceneInfo, String> {
    let root = project_root_inner(state).await?;
    build_scene_asset_catalog(&root)?
        .scenes
        .into_iter()
        .find(|scene| scene.id == scene_id)
        .ok_or_else(|| format!("Story scene `{scene_id}` does not exist in the active project."))
}

fn build_scene_asset_catalog(project_root: &Path) -> Result<SceneAssetCatalog, String> {
    let mut issues = Vec::new();
    let mut scenes = load_scene_metadata(project_root, &mut issues)?;
    let backgrounds = list_background_assets(project_root)?;

    let mut background_by_relative: HashMap<String, BackgroundAsset> = backgrounds
        .iter()
        .cloned()
        .map(|asset| (asset.relative_path.clone(), asset))
        .collect();

    for scene in &mut scenes {
        if let Err(error) = attach_background_status(project_root, scene) {
            push_issue(
                &mut issues,
                "error",
                "scene_background_path_invalid",
                Some(scene.id.clone()),
                scene.background_path.clone(),
                error,
            );
        } else if let Some(background_path) = &scene.background_path {
            if scene.background_exists {
                background_by_relative.remove(background_path);
            } else {
                push_issue(
                    &mut issues,
                    "error",
                    "scene_background_missing",
                    Some(scene.id.clone()),
                    Some(background_path.clone()),
                    "Scene background file does not exist.",
                );
            }
        } else if scene.source == "metadata" {
            push_issue(
                &mut issues,
                "warning",
                "scene_background_not_set",
                Some(scene.id.clone()),
                None,
                "Scene metadata has no background_path.",
            );
        }
    }

    let existing_scene_ids: HashSet<String> = scenes.iter().map(|scene| scene.id.clone()).collect();
    for asset in background_by_relative.values() {
        let virtual_id = derive_scene_id(&asset.relative_path);
        if existing_scene_ids.contains(&virtual_id) {
            continue;
        }
        scenes.push(SceneInfo {
            id: virtual_id,
            name: humanize_id(&asset.id),
            background_path: Some(asset.relative_path.clone()),
            bgm_path: None,
            weather: None,
            time_of_day: None,
            tags: vec!["background".to_string()],
            source: "background".to_string(),
            background_exists: true,
            absolute_background_path: Some(asset.absolute_path.clone()),
        });
    }

    if scenes.is_empty() && backgrounds.is_empty() {
        push_issue(
            &mut issues,
            "warning",
            "scene_assets_empty",
            None,
            None,
            "No scenes or background images were found. Add scenes/*.json or assets/backgrounds images.",
        );
    }

    link_backgrounds_to_scenes(&mut scenes, backgrounds, project_root, issues)
}

fn link_backgrounds_to_scenes(
    scenes: &mut [SceneInfo],
    mut backgrounds: Vec<BackgroundAsset>,
    project_root: &Path,
    issues: Vec<SceneAssetIssue>,
) -> Result<SceneAssetCatalog, String> {
    for asset in &mut backgrounds {
        asset.linked_scene_id = scenes
            .iter()
            .find(|scene| scene.background_path.as_deref() == Some(asset.relative_path.as_str()))
            .map(|scene| scene.id.clone());
    }

    scenes.sort_by(|a, b| a.name.cmp(&b.name));
    backgrounds.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .count();
    let warning_count = issues
        .iter()
        .filter(|issue| issue.severity == "warning")
        .count();

    Ok(SceneAssetCatalog {
        project_path: Some(project_root.to_string_lossy().to_string()),
        valid: error_count == 0,
        error_count,
        warning_count,
        scenes: scenes.to_vec(),
        backgrounds,
        issues,
    })
}

fn load_scene_metadata(
    project_root: &Path,
    issues: &mut Vec<SceneAssetIssue>,
) -> Result<Vec<SceneInfo>, String> {
    let scene_dir = project_root.join("scenes");
    if !scene_dir.exists() {
        return Ok(Vec::new());
    }

    let mut scenes = Vec::new();
    for entry in std::fs::read_dir(&scene_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let relative = relative_path(project_root, &path);
        match std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|content| {
                serde_json::from_str::<SceneInfo>(&content).map_err(|e| e.to_string())
            }) {
            Ok(mut scene) => {
                scene.id = scene.id.trim().to_string();
                scene.name = scene.name.trim().to_string();
                scene.source = "metadata".to_string();
                if scene.id.is_empty() {
                    scene.id = derive_scene_id(&relative);
                }
                if scene.name.is_empty() {
                    scene.name = humanize_id(&scene.id);
                }
                scenes.push(scene);
            }
            Err(error) => push_issue(
                issues,
                "error",
                "scene_metadata_invalid",
                None,
                Some(relative),
                format!("Unable to parse scene metadata: {error}"),
            ),
        }
    }

    let mut ids = HashSet::new();
    for scene in &scenes {
        if !ids.insert(scene.id.clone()) {
            push_issue(
                issues,
                "error",
                "scene_id_duplicate",
                Some(scene.id.clone()),
                None,
                "Scene ids must be unique.",
            );
        }
    }

    Ok(scenes)
}

fn list_background_assets(project_root: &Path) -> Result<Vec<BackgroundAsset>, String> {
    let mut assets = Vec::new();
    for directory in background_directories(project_root) {
        if !directory.exists() {
            continue;
        }
        let mut stack = vec![directory];
        while let Some(current) = stack.pop() {
            for entry in std::fs::read_dir(&current).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if !is_supported_background(&path) {
                    continue;
                }
                let metadata = entry.metadata().map_err(|e| e.to_string())?;
                let relative = relative_path(project_root, &path);
                let extension = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                assets.push(BackgroundAsset {
                    id: derive_scene_id(&relative),
                    file_name: path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default()
                        .to_string(),
                    relative_path: relative,
                    absolute_path: path.to_string_lossy().to_string(),
                    extension,
                    file_size: metadata.len(),
                    linked_scene_id: None,
                });
            }
        }
    }
    Ok(assets)
}

fn attach_background_status(project_root: &Path, scene: &mut SceneInfo) -> Result<(), String> {
    scene.background_exists = false;
    scene.absolute_background_path = None;

    let Some(background_path) = scene.background_path.as_deref() else {
        return Ok(());
    };
    let resolved = resolve_project_relative(project_root, background_path)?;
    scene.background_exists = resolved.exists();
    if scene.background_exists {
        scene.absolute_background_path = Some(resolved.to_string_lossy().to_string());
    }
    Ok(())
}

fn background_directories(project_root: &Path) -> Vec<PathBuf> {
    vec![
        project_root.join("assets").join("backgrounds"),
        project_root.join("assets").join("scenes"),
        project_root.join("backgrounds"),
    ]
}

fn is_supported_background(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| BACKGROUND_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn resolve_project_relative(project_root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() {
        return Err("Asset paths must be relative to the project root.".to_string());
    }
    if relative_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir
        )
    }) {
        return Err("Asset paths cannot escape the project root.".to_string());
    }
    Ok(project_root.join(relative_path))
}

async fn project_root(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    project_root_inner(state).await
}

async fn project_root_inner(state: &AppState) -> Result<PathBuf, String> {
    if let Some(path) = state.project_path.read().await.clone() {
        return Ok(path);
    }

    Ok(default_project_data_root())
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn derive_scene_id(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path);
    stem.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn humanize_id(id: &str) -> String {
    id.split(|ch: char| ch == '_' || ch == '-' || ch == '/')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn push_issue(
    issues: &mut Vec<SceneAssetIssue>,
    severity: impl Into<String>,
    code: impl Into<String>,
    scene_id: Option<String>,
    path: Option<String>,
    message: impl Into<String>,
) {
    issues.push(SceneAssetIssue {
        severity: severity.into(),
        code: code.into(),
        scene_id,
        path,
        message: message.into(),
    });
}

fn default_scene_source() -> String {
    "metadata".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_paths_that_escape_project_root() {
        let root = PathBuf::from("project");
        assert!(resolve_project_relative(&root, "../secrets.png").is_err());
        assert!(resolve_project_relative(&root, "assets/backgrounds/park.png").is_ok());
    }

    #[test]
    fn detects_supported_background_extensions() {
        assert!(is_supported_background(Path::new(
            "assets/backgrounds/park.PNG"
        )));
        assert!(is_supported_background(Path::new(
            "assets/backgrounds/park.webp"
        )));
        assert!(!is_supported_background(Path::new(
            "assets/backgrounds/readme.txt"
        )));
    }

    #[test]
    fn derives_stable_scene_ids_from_paths() {
        assert_eq!(
            derive_scene_id("assets/backgrounds/Sakura Park.png"),
            "sakura_park"
        );
        assert_eq!(humanize_id("sakura_park"), "Sakura Park");
    }

    #[test]
    fn builds_catalog_from_scene_metadata_and_backgrounds() {
        let root = std::env::temp_dir().join(format!(
            "monogatari_scene_catalog_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let scenes_dir = root.join("scenes");
        let backgrounds_dir = root.join("assets").join("backgrounds");
        std::fs::create_dir_all(&scenes_dir).unwrap();
        std::fs::create_dir_all(&backgrounds_dir).unwrap();
        std::fs::write(
            scenes_dir.join("park.json"),
            r#"{
              "id": "sakura_park",
              "name": "Sakura Park",
              "background_path": "assets/backgrounds/sakura_park.svg"
            }"#,
        )
        .unwrap();
        std::fs::write(backgrounds_dir.join("sakura_park.svg"), "<svg></svg>").unwrap();

        let catalog = build_scene_asset_catalog(&root).unwrap();
        assert!(catalog.valid, "{:?}", catalog.issues);
        assert_eq!(catalog.scenes.len(), 1);
        assert_eq!(catalog.backgrounds.len(), 1);
        assert_eq!(
            catalog.backgrounds[0].linked_scene_id.as_deref(),
            Some("sakura_park")
        );

        std::fs::remove_dir_all(root).unwrap();
    }
}
