//! Scene asset catalog commands for background/scene production workflows.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

use crate::content_authoring::{
    ensure_regular_project_directory, sha256_json, source_label, stage_json_deletion,
    stage_json_replacement,
};
use crate::content_references::scene_references;
use crate::state::{default_project_data_root, AppState};
use crate::story_access::{
    ensure_story_content_access, story_content_access, StoryContentAccessEntry, StoryContentKind,
};

const BACKGROUND_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "gif", "svg"];
const SCENE_AUTHORING_CATALOG_SCHEMA_V1: &str = "monogatari-scene-authoring-catalog/v1";
const MAX_SCENE_FILES: usize = 512;
const MAX_SCENE_FILE_BYTES: u64 = 64 * 1024;
const MAX_SCENE_TAGS: usize = 64;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SceneDefinition {
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
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneAuthoringEntry {
    #[serde(flatten)]
    pub scene: SceneDefinition,
    pub source_path: Option<String>,
    pub content_fingerprint: String,
    pub metadata_authored: bool,
    pub background_exists: bool,
    pub absolute_background_path: Option<String>,
    pub access: StoryContentAccessEntry,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneAuthoringCatalogSnapshot {
    pub schema: String,
    pub catalog_fingerprint: String,
    pub scene_count: usize,
    pub metadata_scene_count: usize,
    pub inferred_scene_count: usize,
    pub scenes: Vec<SceneAuthoringEntry>,
    pub issues: Vec<SceneAssetIssue>,
}

#[derive(Debug, Clone)]
struct LoadedSceneDefinition {
    scene: SceneDefinition,
    source_path: String,
    absolute_path: PathBuf,
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

/// Return full editable scene definitions with stable catalog fingerprints.
#[tauri::command]
pub async fn get_scene_authoring_catalog(
    state: State<'_, AppState>,
) -> Result<SceneAuthoringCatalogSnapshot, String> {
    scene_authoring_catalog_snapshot(&state).await
}

/// Atomically create, promote, or update one scene metadata document.
#[tauri::command]
pub async fn save_scene_definition(
    state: State<'_, AppState>,
    scene: SceneDefinition,
    original_scene_id: Option<String>,
    expected_catalog_fingerprint: String,
) -> Result<SceneAuthoringCatalogSnapshot, String> {
    save_scene_definition_inner(
        &state,
        scene,
        original_scene_id.as_deref(),
        &expected_catalog_fingerprint,
    )
    .await
}

/// Delete scene metadata after checking project event, ending, and workflow references.
#[tauri::command]
pub async fn delete_scene_definition(
    state: State<'_, AppState>,
    scene_id: String,
    expected_catalog_fingerprint: String,
) -> Result<SceneAuthoringCatalogSnapshot, String> {
    delete_scene_definition_inner(&state, &scene_id, &expected_catalog_fingerprint).await
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

async fn scene_authoring_catalog_snapshot(
    state: &AppState,
) -> Result<SceneAuthoringCatalogSnapshot, String> {
    let project_root = project_root_inner(state).await?;
    let (loaded, catalog, _) = load_scene_authoring_state(&project_root)?;
    scene_authoring_snapshot_from_parts(state, loaded, catalog).await
}

async fn scene_authoring_snapshot_from_parts(
    state: &AppState,
    loaded: Vec<LoadedSceneDefinition>,
    catalog: SceneAssetCatalog,
) -> Result<SceneAuthoringCatalogSnapshot, String> {
    let catalog_fingerprint = scene_authoring_catalog_fingerprint(&loaded, &catalog);
    let authored = loaded
        .iter()
        .map(|loaded| (loaded.scene.id.as_str(), loaded))
        .collect::<HashMap<_, _>>();
    let event_catalog = state.story_event_catalog.read().await;
    let progress = state.story_progress.read().await;
    let mut scenes = catalog
        .scenes
        .iter()
        .map(|scene| {
            let definition = scene_definition_from_info(scene);
            let source = authored.get(definition.id.as_str());
            SceneAuthoringEntry {
                content_fingerprint: scene_content_fingerprint(&definition),
                access: story_content_access(
                    &event_catalog,
                    &progress,
                    StoryContentKind::Scene,
                    &definition.id,
                ),
                source_path: source.map(|loaded| loaded.source_path.clone()),
                metadata_authored: source.is_some(),
                background_exists: scene.background_exists,
                absolute_background_path: scene.absolute_background_path.clone(),
                scene: definition,
            }
        })
        .collect::<Vec<_>>();
    scenes.sort_by(|left, right| left.scene.id.cmp(&right.scene.id));
    let metadata_scene_count = scenes
        .iter()
        .filter(|scene| scene.metadata_authored)
        .count();
    Ok(SceneAuthoringCatalogSnapshot {
        schema: SCENE_AUTHORING_CATALOG_SCHEMA_V1.to_string(),
        catalog_fingerprint,
        scene_count: scenes.len(),
        metadata_scene_count,
        inferred_scene_count: scenes.len().saturating_sub(metadata_scene_count),
        scenes,
        issues: catalog.issues,
    })
}

async fn save_scene_definition_inner(
    state: &AppState,
    scene: SceneDefinition,
    original_scene_id: Option<&str>,
    expected_catalog_fingerprint: &str,
) -> Result<SceneAuthoringCatalogSnapshot, String> {
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    let project_root = project_root_inner(state).await?;
    let scene = normalize_scene_definition(scene);
    validate_scene_definition(&project_root, &scene)?;

    let (current, current_catalog, fingerprint) = load_scene_authoring_state(&project_root)?;
    ensure_scene_catalog_fingerprint(&fingerprint, expected_catalog_fingerprint)?;
    let scene_root =
        ensure_regular_project_directory(&project_root, "scenes", "scene metadata").await?;
    let target_path = match original_scene_id {
        Some(original_id) => {
            if original_id != scene.id {
                return Err(
                    "Scene ids are immutable after metadata creation; duplicate the scene to use a new id."
                        .to_string(),
                );
            }
            current
                .iter()
                .find(|loaded| loaded.scene.id == original_id)
                .map(|loaded| loaded.absolute_path.clone())
                .ok_or_else(|| {
                    format!(
                        "Scene metadata `{original_id}` no longer exists; reload before saving."
                    )
                })?
        }
        None => {
            if current.iter().any(|loaded| loaded.scene.id == scene.id) {
                return Err(format!(
                    "Scene metadata `{}` already exists; reload it before editing.",
                    scene.id
                ));
            }
            scene_root.join(format!("{}.json", scene.id))
        }
    };
    drop(current_catalog);

    let mut content = serde_json::to_string_pretty(&scene)
        .map_err(|error| format!("Unable to serialize scene metadata: {error}"))?;
    content.push('\n');
    let staged = stage_json_replacement(
        &target_path,
        content.as_bytes(),
        MAX_SCENE_FILE_BYTES,
        "scene metadata",
    )
    .await?;

    let (loaded, catalog, _) = match load_scene_authoring_state(&project_root) {
        Ok(state) => state,
        Err(error) => {
            staged.rollback().await?;
            return Err(format!(
                "Saved scene metadata failed project reload and was rolled back: {error}"
            ));
        }
    };
    if !loaded.iter().any(|loaded| loaded.scene == scene) {
        staged.rollback().await?;
        return Err(
            "Saved scene metadata changed during replacement; the original was restored."
                .to_string(),
        );
    }
    staged.commit().await?;
    scene_authoring_snapshot_from_parts(state, loaded, catalog).await
}

async fn delete_scene_definition_inner(
    state: &AppState,
    scene_id: &str,
    expected_catalog_fingerprint: &str,
) -> Result<SceneAuthoringCatalogSnapshot, String> {
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    let project_root = project_root_inner(state).await?;
    let (current, _, fingerprint) = load_scene_authoring_state(&project_root)?;
    ensure_scene_catalog_fingerprint(&fingerprint, expected_catalog_fingerprint)?;
    let target = current
        .iter()
        .find(|loaded| loaded.scene.id == scene_id)
        .ok_or_else(|| {
            format!(
                "Scene `{scene_id}` is inferred from a background asset and has no metadata document to delete."
            )
        })?;
    let references = scene_references(&project_root, scene_id)?;
    if !references.is_empty() {
        return Err(format!(
            "Scene `{scene_id}` is still referenced by: {}. Remove those references before deleting its metadata.",
            references.join(", ")
        ));
    }

    let staged = stage_json_deletion(&target.absolute_path, "scene metadata").await?;
    let (loaded, catalog, _) = match load_scene_authoring_state(&project_root) {
        Ok(state) if !state.0.iter().any(|loaded| loaded.scene.id == scene_id) => state,
        Ok(_) => {
            staged.rollback().await?;
            return Err(
                "Deleted scene metadata remained in the authored catalog; the file was restored."
                    .to_string(),
            );
        }
        Err(error) => {
            staged.rollback().await?;
            return Err(format!(
                "Deleting scene metadata broke the project catalog and was rolled back: {error}"
            ));
        }
    };
    staged.commit().await?;

    if state.active_scene_id.read().await.as_deref() == Some(scene_id) {
        *state.active_scene_id.write().await = None;
    }
    state
        .scene_history
        .write()
        .await
        .retain(|history_id| history_id != scene_id);
    scene_authoring_snapshot_from_parts(state, loaded, catalog).await
}

fn load_scene_authoring_state(
    project_root: &Path,
) -> Result<(Vec<LoadedSceneDefinition>, SceneAssetCatalog, String), String> {
    let loaded = load_scene_documents(project_root)?;
    let catalog = build_scene_asset_catalog(project_root)?;
    let fingerprint = scene_authoring_catalog_fingerprint(&loaded, &catalog);
    Ok((loaded, catalog, fingerprint))
}

fn ensure_scene_catalog_fingerprint(actual: &str, expected: &str) -> Result<(), String> {
    if actual != expected {
        return Err(format!(
            "Scene catalog changed since it was opened; expected `{expected}`, current `{actual}`. Reload before saving."
        ));
    }
    Ok(())
}

fn scene_authoring_catalog_fingerprint(
    loaded: &[LoadedSceneDefinition],
    catalog: &SceneAssetCatalog,
) -> String {
    let sources = loaded
        .iter()
        .map(|loaded| (loaded.scene.id.as_str(), loaded.source_path.as_str()))
        .collect::<HashMap<_, _>>();
    let mut scenes = catalog
        .scenes
        .iter()
        .map(|scene| {
            let definition = scene_definition_from_info(scene);
            json!({
                "source_path": sources.get(definition.id.as_str()),
                "metadata_authored": sources.contains_key(definition.id.as_str()),
                "scene": definition,
            })
        })
        .collect::<Vec<Value>>();
    scenes.sort_by(|left, right| {
        left["scene"]["id"]
            .as_str()
            .cmp(&right["scene"]["id"].as_str())
    });
    sha256_json(&json!({
        "schema": SCENE_AUTHORING_CATALOG_SCHEMA_V1,
        "scenes": scenes,
    }))
}

fn scene_content_fingerprint(scene: &SceneDefinition) -> String {
    sha256_json(&json!({
        "schema": "monogatari-scene-content-fingerprint/v1",
        "scene": scene,
    }))
}

fn scene_definition_from_info(scene: &SceneInfo) -> SceneDefinition {
    SceneDefinition {
        id: scene.id.clone(),
        name: scene.name.clone(),
        background_path: scene.background_path.clone(),
        bgm_path: scene.bgm_path.clone(),
        weather: scene.weather.clone(),
        time_of_day: scene.time_of_day.clone(),
        tags: scene.tags.clone(),
    }
}

fn normalize_scene_definition(mut scene: SceneDefinition) -> SceneDefinition {
    scene.id = scene.id.trim().to_string();
    scene.name = scene.name.trim().to_string();
    scene.background_path = normalize_optional_text(scene.background_path);
    scene.bgm_path = normalize_optional_text(scene.bgm_path);
    scene.weather = normalize_optional_text(scene.weather);
    scene.time_of_day = normalize_optional_text(scene.time_of_day);
    scene.tags = scene
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();
    scene.tags.sort();
    scene.tags.dedup();
    scene
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn validate_scene_definition(project_root: &Path, scene: &SceneDefinition) -> Result<(), String> {
    if !is_portable_scene_id(&scene.id) {
        return Err(format!("Scene id `{}` is not a portable id.", scene.id));
    }
    validate_bounded_scene_text(&scene.name, "name", 1, 256, &scene.id)?;
    for (label, value) in [
        ("weather", scene.weather.as_deref()),
        ("time_of_day", scene.time_of_day.as_deref()),
    ] {
        if let Some(value) = value {
            validate_bounded_scene_text(value, label, 1, 64, &scene.id)?;
        }
    }
    if scene.tags.len() > MAX_SCENE_TAGS {
        return Err(format!(
            "Scene `{}` has {} tags; the limit is {MAX_SCENE_TAGS}.",
            scene.id,
            scene.tags.len()
        ));
    }
    for tag in &scene.tags {
        validate_bounded_scene_text(tag, "tag", 1, 64, &scene.id)?;
    }
    if let Some(background_path) = scene.background_path.as_deref() {
        let resolved = resolve_project_relative(project_root, background_path)?;
        if !is_supported_background(Path::new(background_path)) {
            return Err(format!(
                "Scene `{}` background uses an unsupported file type: {background_path}",
                scene.id
            ));
        }
        if !resolved.is_file() {
            return Err(format!(
                "Scene `{}` background does not exist: {background_path}",
                scene.id
            ));
        }
    }
    if let Some(bgm_path) = scene.bgm_path.as_deref() {
        resolve_project_relative(project_root, bgm_path)?;
        let extension = Path::new(bgm_path)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !["mp3", "ogg", "wav", "m4a", "aac", "flac"].contains(&extension.as_str()) {
            return Err(format!(
                "Scene `{}` BGM uses an unsupported file type: {bgm_path}",
                scene.id
            ));
        }
    }
    Ok(())
}

fn validate_bounded_scene_text(
    value: &str,
    label: &str,
    min: usize,
    max: usize,
    scene_id: &str,
) -> Result<(), String> {
    let count = value.chars().count();
    if count < min || count > max || value.chars().any(char::is_control) {
        return Err(format!(
            "Scene `{scene_id}` {label} must contain {min} to {max} non-control characters."
        ));
    }
    Ok(())
}

fn is_portable_scene_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.trim() == value
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

pub(crate) fn build_scene_asset_catalog(project_root: &Path) -> Result<SceneAssetCatalog, String> {
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

fn load_scene_documents(project_root: &Path) -> Result<Vec<LoadedSceneDefinition>, String> {
    let scene_root = project_root.join("scenes");
    if !scene_root.exists() {
        return Ok(Vec::new());
    }
    let root_metadata = std::fs::symlink_metadata(&scene_root).map_err(|error| {
        format!(
            "Failed to inspect scene metadata directory `{}`: {error}",
            scene_root.display()
        )
    })?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(format!(
            "Scene metadata path must be a regular directory: {}",
            scene_root.display()
        ));
    }
    let canonical_root = scene_root.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve scene metadata directory `{}`: {error}",
            scene_root.display()
        )
    })?;
    let mut paths = std::fs::read_dir(&scene_root)
        .map_err(|error| {
            format!(
                "Failed to read scene metadata directory `{}`: {error}",
                scene_root.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    if paths.len() > MAX_SCENE_FILES {
        return Err(format!(
            "Scene metadata directory contains {} JSON files; the limit is {MAX_SCENE_FILES}.",
            paths.len()
        ));
    }

    let mut seen = HashSet::new();
    let mut loaded = Vec::with_capacity(paths.len());
    for path in paths {
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "Failed to inspect scene metadata `{}`: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Scene metadata must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_SCENE_FILE_BYTES {
            return Err(format!(
                "Scene metadata `{}` is {} bytes; the limit is {MAX_SCENE_FILE_BYTES} bytes.",
                path.display(),
                metadata.len()
            ));
        }
        let canonical_path = path.canonicalize().map_err(|error| {
            format!(
                "Failed to resolve scene metadata `{}`: {error}",
                path.display()
            )
        })?;
        if !canonical_path.starts_with(&canonical_root) {
            return Err(format!(
                "Scene metadata escapes the project scene directory: {}",
                path.display()
            ));
        }
        let content = std::fs::read_to_string(&canonical_path).map_err(|error| {
            format!(
                "Failed to read scene metadata `{}`: {error}",
                path.display()
            )
        })?;
        let scene: SceneDefinition = serde_json::from_str(&content).map_err(|error| {
            format!(
                "Invalid scene metadata JSON in `{}`: {error}",
                path.display()
            )
        })?;
        let scene = normalize_scene_definition(scene);
        validate_scene_definition(project_root, &scene)?;
        if !seen.insert(scene.id.clone()) {
            return Err(format!("Duplicate scene id `{}`.", scene.id));
        }
        loaded.push(LoadedSceneDefinition {
            scene,
            source_path: source_label(project_root, &path),
            absolute_path: canonical_path,
        });
    }
    loaded.sort_by(|left, right| left.scene.id.cmp(&right.scene.id));
    Ok(loaded)
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
    if relative.is_empty()
        || relative.trim() != relative
        || relative.contains('\\')
        || relative.contains(':')
        || relative.chars().any(char::is_control)
        || relative
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
    {
        return Err(
            "Asset paths must use portable non-empty project-relative segments.".to_string(),
        );
    }
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
    use crate::story_events::StoryEventCatalog;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_scene_authoring_{label}_{}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn write_authoring_project(root: &Path) {
        for directory in [
            "characters",
            "knowledge",
            "events",
            "endings",
            "scenes",
            "dialogue",
            "workflows",
            "assets/backgrounds",
        ] {
            std::fs::create_dir_all(root.join(directory)).unwrap();
        }
        std::fs::write(
            root.join("events").join("events.json"),
            r#"{"schema":"monogatari-story-event-catalog/v1","events":[]}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("assets").join("backgrounds").join("park.svg"),
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="9"></svg>"#,
        )
        .unwrap();
    }

    async fn authoring_state(root: &Path) -> AppState {
        let state = AppState::new();
        state.set_project_data_root(root.to_path_buf()).await;
        *state.story_event_catalog.write().await =
            StoryEventCatalog::load_from_project_root(root).unwrap();
        state
    }

    #[test]
    fn rejects_paths_that_escape_project_root() {
        let root = PathBuf::from("project");
        assert!(resolve_project_relative(&root, "../secrets.png").is_err());
        assert!(resolve_project_relative(&root, "assets\\backgrounds\\park.png").is_err());
        assert!(resolve_project_relative(&root, "assets/./park.png").is_err());
        assert!(resolve_project_relative(&root, "assets//park.png").is_err());
        assert!(resolve_project_relative(&root, "https://example.com/park.png").is_err());
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
        let root = temp_root("catalog");
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

    #[tokio::test]
    async fn scene_save_promotes_inferred_assets_and_rejects_stale_or_invalid_updates() {
        let root = temp_root("save");
        write_authoring_project(&root);
        let state = authoring_state(&root).await;
        let before = scene_authoring_catalog_snapshot(&state).await.unwrap();
        assert_eq!(before.scene_count, 1);
        assert_eq!(before.metadata_scene_count, 0);
        assert_eq!(before.inferred_scene_count, 1);
        assert_eq!(before.scenes[0].scene.id, "park");
        assert!(!before.scenes[0].metadata_authored);

        let definition = SceneDefinition {
            id: "park".to_string(),
            name: "Moonlit Park".to_string(),
            background_path: Some("assets/backgrounds/park.svg".to_string()),
            bgm_path: Some("assets/audio/moon.ogg".to_string()),
            weather: Some("clear".to_string()),
            time_of_day: Some("night".to_string()),
            tags: vec![
                "romance".to_string(),
                "night".to_string(),
                "night".to_string(),
            ],
        };
        let saved = save_scene_definition_inner(
            &state,
            definition.clone(),
            None,
            &before.catalog_fingerprint,
        )
        .await
        .unwrap();
        assert_eq!(saved.metadata_scene_count, 1);
        assert_eq!(saved.inferred_scene_count, 0);
        assert!(saved.scenes[0].metadata_authored);
        assert_eq!(saved.scenes[0].scene.tags, vec!["night", "romance"]);
        let path = root.join("scenes").join("park.json");
        let saved_file = std::fs::read_to_string(&path).unwrap();
        assert!(saved_file.contains("Moonlit Park"));

        let mut stale = definition.clone();
        stale.name = "Stale Park".to_string();
        assert!(save_scene_definition_inner(
            &state,
            stale,
            Some("park"),
            &before.catalog_fingerprint,
        )
        .await
        .unwrap_err()
        .contains("changed since it was opened"));

        let mut invalid = definition;
        invalid.background_path = Some("assets/backgrounds/missing.svg".to_string());
        assert!(save_scene_definition_inner(
            &state,
            invalid,
            Some("park"),
            &saved.catalog_fingerprint,
        )
        .await
        .unwrap_err()
        .contains("does not exist"));
        assert_eq!(std::fs::read_to_string(path).unwrap(), saved_file);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn scene_delete_requires_event_ending_and_workflow_references_to_be_removed() {
        let root = temp_root("delete");
        write_authoring_project(&root);
        std::fs::write(
            root.join("scenes").join("park.json"),
            r#"{"id":"park","name":"Park","background_path":"assets/backgrounds/park.svg"}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("events").join("events.json"),
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"unlock_park","event_type":"unlock","description":"Unlock park",
                "actions":[{"type":"unlock_scene","scene_id":"park"}]
              }]
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("endings").join("park_ending.json"),
            r#"{
              "schema":"monogatari-story-ending/v1","id":"park_ending","title":"Park Ending",
              "description":"An ending in the park.","scene_id":"park","dialogue_id":"park_finale"
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("workflows").join("park_route.json"),
            r#"{
              "id":"park_route","name":"Park Route","start_node_id":"scene",
              "nodes":[{
                "id":"scene","node_type":"scene_change","label":"Park","x":0.0,"y":0.0,
                "config":{"scene_id":"park"},"connections":[]
              }]
            }"#,
        )
        .unwrap();
        let state = authoring_state(&root).await;
        *state.active_scene_id.write().await = Some("park".to_string());
        state
            .scene_history
            .write()
            .await
            .extend(["intro".to_string(), "park".to_string()]);
        let before = scene_authoring_catalog_snapshot(&state).await.unwrap();

        let error = delete_scene_definition_inner(&state, "park", &before.catalog_fingerprint)
            .await
            .unwrap_err();
        assert!(error.contains("event:unlock_park"), "{error}");
        assert!(error.contains("ending:park_ending"), "{error}");
        assert!(error.contains("workflow:park_route/scene"), "{error}");
        assert!(root.join("scenes").join("park.json").is_file());

        std::fs::write(
            root.join("events").join("events.json"),
            r#"{"schema":"monogatari-story-event-catalog/v1","events":[]}"#,
        )
        .unwrap();
        std::fs::remove_file(root.join("endings").join("park_ending.json")).unwrap();
        std::fs::remove_file(root.join("workflows").join("park_route.json")).unwrap();

        let after = delete_scene_definition_inner(&state, "park", &before.catalog_fingerprint)
            .await
            .unwrap();
        assert_eq!(after.scene_count, 1);
        assert_eq!(after.metadata_scene_count, 0);
        assert_eq!(after.inferred_scene_count, 1);
        assert!(!after.scenes[0].metadata_authored);
        assert!(!root.join("scenes").join("park.json").exists());
        assert!(state.active_scene_id.read().await.is_none());
        assert_eq!(state.scene_history.read().await.as_slice(), ["intro"]);
        std::fs::remove_dir_all(root).unwrap();
    }
}
