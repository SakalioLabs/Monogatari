//! Versioned ending assets and gated Story Mode launch commands.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::State;
use tokio::io::AsyncWriteExt;

use crate::commands::dialogue::{
    ensure_project_dialogues_loaded, start_dialogue_authoring_inner, start_dialogue_inner,
    DialogueState,
};
use crate::commands::scenes::{
    build_scene_asset_catalog, resolve_project_scene, set_scene_inner, SceneInfo,
};
use crate::state::AppState;
use crate::story_access::{
    ensure_story_content_access, story_content_access, StoryContentAccessEntry, StoryContentKind,
};
use crate::story_events::{StoryEventAction, StoryEventCatalog};

use llm_game::dialogue::DialogueManager;

const STORY_ENDING_SCHEMA_V1: &str = "monogatari-story-ending/v1";
const STORY_ENDING_CATALOG_SCHEMA_V1: &str = "monogatari-story-ending-catalog/v1";
const MAX_ENDING_FILES: usize = 256;
const MAX_ENDING_FILE_BYTES: u64 = 64 * 1024;
static ENDING_STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StoryEndingDefinition {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub description: String,
    pub scene_id: String,
    pub dialogue_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoryEndingCatalogEntry {
    #[serde(flatten)]
    pub ending: StoryEndingDefinition,
    pub access: StoryContentAccessEntry,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoryEndingAuthoringEntry {
    #[serde(flatten)]
    pub ending: StoryEndingDefinition,
    pub source_path: String,
    pub content_fingerprint: String,
    pub access: StoryContentAccessEntry,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoryEndingCatalogSnapshot {
    pub schema: String,
    pub catalog_fingerprint: String,
    pub ending_count: usize,
    pub endings: Vec<StoryEndingAuthoringEntry>,
}

#[derive(Debug, Serialize)]
pub struct StoryEndingLaunch {
    pub ending: StoryEndingDefinition,
    pub scene: SceneInfo,
    pub dialogue: DialogueState,
}

#[derive(Debug, Clone)]
struct LoadedStoryEnding {
    ending: StoryEndingDefinition,
    source_path: String,
    absolute_path: PathBuf,
}

#[tauri::command]
pub async fn list_story_endings(
    state: State<'_, AppState>,
) -> Result<Vec<StoryEndingCatalogEntry>, String> {
    list_story_endings_inner(&state).await
}

async fn list_story_endings_inner(
    state: &AppState,
) -> Result<Vec<StoryEndingCatalogEntry>, String> {
    let definitions = load_story_ending_sources(&state.current_project_data_root().await)?;
    let catalog = state.story_event_catalog.read().await;
    let progress = state.story_progress.read().await;
    Ok(definitions
        .into_iter()
        .map(|loaded| StoryEndingCatalogEntry {
            access: story_content_access(
                &catalog,
                &progress,
                StoryContentKind::Ending,
                &loaded.ending.id,
            ),
            ending: loaded.ending,
        })
        .collect())
}

/// Return editable ending definitions with deterministic concurrency fingerprints.
#[tauri::command]
pub async fn get_story_ending_catalog(
    state: State<'_, AppState>,
) -> Result<StoryEndingCatalogSnapshot, String> {
    ending_catalog_snapshot(&state).await
}

/// Validate and atomically create or replace one ending definition.
#[tauri::command]
pub async fn save_story_ending(
    state: State<'_, AppState>,
    ending: StoryEndingDefinition,
    original_ending_id: Option<String>,
    expected_catalog_fingerprint: String,
) -> Result<StoryEndingCatalogSnapshot, String> {
    save_story_ending_inner(
        &state,
        ending,
        original_ending_id.as_deref(),
        &expected_catalog_fingerprint,
    )
    .await
}

/// Delete an ending only when no active story event still unlocks it.
#[tauri::command]
pub async fn delete_story_ending(
    state: State<'_, AppState>,
    ending_id: String,
    expected_catalog_fingerprint: String,
) -> Result<StoryEndingCatalogSnapshot, String> {
    delete_story_ending_inner(&state, &ending_id, &expected_catalog_fingerprint).await
}

#[tauri::command]
pub async fn start_story_ending(
    state: State<'_, AppState>,
    ending_id: String,
) -> Result<StoryEndingLaunch, String> {
    start_story_ending_inner(&state, &ending_id).await
}

/// Launch a valid ending from the workbench without requiring player unlock progress.
#[tauri::command]
pub async fn preview_story_ending(
    state: State<'_, AppState>,
    ending_id: String,
) -> Result<StoryEndingLaunch, String> {
    preview_story_ending_inner(&state, &ending_id).await
}

async fn start_story_ending_inner(
    state: &AppState,
    ending_id: &str,
) -> Result<StoryEndingLaunch, String> {
    launch_story_ending_inner(state, ending_id, true).await
}

async fn preview_story_ending_inner(
    state: &AppState,
    ending_id: &str,
) -> Result<StoryEndingLaunch, String> {
    launch_story_ending_inner(state, ending_id, false).await
}

async fn launch_story_ending_inner(
    state: &AppState,
    ending_id: &str,
    enforce_player_access: bool,
) -> Result<StoryEndingLaunch, String> {
    let ending = load_story_endings(&state.current_project_data_root().await)?
        .into_iter()
        .find(|ending| ending.id == ending_id)
        .ok_or_else(|| {
            format!("Story ending `{ending_id}` does not exist in the active project.")
        })?;
    if enforce_player_access {
        let catalog = state.story_event_catalog.read().await;
        let progress = state.story_progress.read().await;
        ensure_story_content_access(&catalog, &progress, StoryContentKind::Ending, &ending.id)?;
        ensure_story_content_access(
            &catalog,
            &progress,
            StoryContentKind::Scene,
            &ending.scene_id,
        )?;
        ensure_story_content_access(
            &catalog,
            &progress,
            StoryContentKind::Dialogue,
            &ending.dialogue_id,
        )?;
    }

    resolve_project_scene(state, &ending.scene_id).await?;
    ensure_project_dialogues_loaded(state).await?;
    if !state
        .dialogue_manager
        .read()
        .await
        .has_script(&ending.dialogue_id)
    {
        return Err(format!(
            "Story ending `{}` references missing dialogue `{}`.",
            ending.id, ending.dialogue_id
        ));
    }

    let scene = set_scene_inner(state, ending.scene_id.clone(), None, None, None).await?;
    let dialogue = if enforce_player_access {
        start_dialogue_inner(state, &ending.dialogue_id).await?
    } else {
        start_dialogue_authoring_inner(state, &ending.dialogue_id).await?
    };
    Ok(StoryEndingLaunch {
        ending,
        scene,
        dialogue,
    })
}

async fn ending_catalog_snapshot(state: &AppState) -> Result<StoryEndingCatalogSnapshot, String> {
    let root = state.current_project_data_root().await;
    let loaded = load_story_ending_sources(&root)?;
    ending_catalog_snapshot_from_loaded(state, loaded).await
}

async fn ending_catalog_snapshot_from_loaded(
    state: &AppState,
    loaded: Vec<LoadedStoryEnding>,
) -> Result<StoryEndingCatalogSnapshot, String> {
    let catalog_fingerprint = story_ending_catalog_fingerprint(&loaded);
    let event_catalog = state.story_event_catalog.read().await;
    let progress = state.story_progress.read().await;
    let endings = loaded
        .into_iter()
        .map(|loaded| StoryEndingAuthoringEntry {
            content_fingerprint: story_ending_content_fingerprint(&loaded.ending),
            access: story_content_access(
                &event_catalog,
                &progress,
                StoryContentKind::Ending,
                &loaded.ending.id,
            ),
            source_path: loaded.source_path,
            ending: loaded.ending,
        })
        .collect::<Vec<_>>();
    Ok(StoryEndingCatalogSnapshot {
        schema: STORY_ENDING_CATALOG_SCHEMA_V1.to_string(),
        catalog_fingerprint,
        ending_count: endings.len(),
        endings,
    })
}

async fn save_story_ending_inner(
    state: &AppState,
    mut ending: StoryEndingDefinition,
    original_ending_id: Option<&str>,
    expected_catalog_fingerprint: &str,
) -> Result<StoryEndingCatalogSnapshot, String> {
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    ending.title = ending.title.trim().to_string();
    ending.description = ending.description.trim().to_string();

    let project_root = state.current_project_data_root().await;
    let current = load_story_ending_sources(&project_root)?;
    ensure_expected_catalog_fingerprint(&current, expected_catalog_fingerprint)?;
    validate_ending(&ending, Path::new("<authoring-request>"))?;
    validate_story_ending_references(&project_root, std::slice::from_ref(&ending)).await?;

    let ending_root = ensure_story_ending_directory(&project_root).await?;
    let target_path = match original_ending_id {
        Some(original_id) => {
            if original_id != ending.id {
                return Err(
                    "Story ending ids are immutable after creation; duplicate the ending to use a new id."
                        .to_string(),
                );
            }
            current
                .iter()
                .find(|loaded| loaded.ending.id == original_id)
                .map(|loaded| loaded.absolute_path.clone())
                .ok_or_else(|| {
                    format!(
                        "Story ending `{original_id}` no longer exists; reload the catalog before saving."
                    )
                })?
        }
        None => {
            if current.iter().any(|loaded| loaded.ending.id == ending.id) {
                return Err(format!(
                    "Story ending `{}` already exists; reload it before editing.",
                    ending.id
                ));
            }
            ending_root.join(format!("{}.json", ending.id))
        }
    };

    let mut content = serde_json::to_string_pretty(&ending)
        .map_err(|error| format!("Unable to serialize story ending: {error}"))?;
    content.push('\n');
    if content.len() as u64 > MAX_ENDING_FILE_BYTES {
        return Err(format!(
            "Story ending `{}` is {} bytes; the limit is {MAX_ENDING_FILE_BYTES} bytes.",
            ending.id,
            content.len()
        ));
    }

    let staged = ending_stage_paths(&target_path)?;
    let had_target =
        replace_story_ending_document(&target_path, &staged.0, &staged.1, content.as_bytes())
            .await?;

    let loaded = match load_story_ending_sources(&project_root) {
        Ok(loaded) => loaded,
        Err(error) => {
            rollback_story_ending_document(&target_path, &staged.1, had_target).await?;
            return Err(format!(
                "Saved story ending failed project reload and was rolled back: {error}"
            ));
        }
    };
    let definitions = loaded
        .iter()
        .map(|loaded| loaded.ending.clone())
        .collect::<Vec<_>>();
    if let Err(error) = validate_story_ending_references(&project_root, &definitions).await {
        rollback_story_ending_document(&target_path, &staged.1, had_target).await?;
        return Err(format!(
            "Saved story ending failed content validation and was rolled back: {error}"
        ));
    }
    if !loaded.iter().any(|loaded| loaded.ending == ending) {
        rollback_story_ending_document(&target_path, &staged.1, had_target).await?;
        return Err(
            "Saved story ending changed during replacement; the original was restored.".to_string(),
        );
    }

    cleanup_story_ending_backup(&staged.1).await;
    ending_catalog_snapshot_from_loaded(state, loaded).await
}

async fn delete_story_ending_inner(
    state: &AppState,
    ending_id: &str,
    expected_catalog_fingerprint: &str,
) -> Result<StoryEndingCatalogSnapshot, String> {
    let _authoring_guard = state.story_content_authoring_lock.lock().await;
    let project_root = state.current_project_data_root().await;
    let current = load_story_ending_sources(&project_root)?;
    ensure_expected_catalog_fingerprint(&current, expected_catalog_fingerprint)?;
    let target = current
        .iter()
        .find(|loaded| loaded.ending.id == ending_id)
        .ok_or_else(|| format!("Story ending `{ending_id}` does not exist."))?;

    let event_catalog = StoryEventCatalog::load_from_project_root(&project_root)?;
    let mut event_ids = event_catalog
        .definitions()
        .iter()
        .filter(|definition| {
            definition.actions.iter().any(|action| {
                matches!(
                    action,
                    StoryEventAction::UnlockEnding { ending_id: target_id }
                        if target_id == ending_id
                )
            })
        })
        .map(|definition| definition.event_id.clone())
        .collect::<Vec<_>>();
    event_ids.sort();
    if !event_ids.is_empty() {
        return Err(format!(
            "Story ending `{ending_id}` is still unlocked by event(s): {}. Remove those actions before deleting it.",
            event_ids.join(", ")
        ));
    }

    let (_, backup_path) = ending_stage_paths(&target.absolute_path)?;
    tokio::fs::rename(&target.absolute_path, &backup_path)
        .await
        .map_err(|error| {
            format!(
                "Failed to stage story ending `{}` for deletion: {error}",
                target.absolute_path.display()
            )
        })?;

    let loaded = match load_story_ending_sources(&project_root) {
        Ok(loaded) if !loaded.iter().any(|loaded| loaded.ending.id == ending_id) => loaded,
        Ok(_) => {
            restore_deleted_story_ending(&target.absolute_path, &backup_path).await?;
            return Err(
                "Deleted story ending remained in the project catalog; the file was restored."
                    .to_string(),
            );
        }
        Err(error) => {
            restore_deleted_story_ending(&target.absolute_path, &backup_path).await?;
            return Err(format!(
                "Deleting story ending broke the project catalog and was rolled back: {error}"
            ));
        }
    };

    cleanup_story_ending_backup(&backup_path).await;
    ending_catalog_snapshot_from_loaded(state, loaded).await
}

fn ensure_expected_catalog_fingerprint(
    current: &[LoadedStoryEnding],
    expected: &str,
) -> Result<(), String> {
    let actual = story_ending_catalog_fingerprint(current);
    if actual != expected {
        return Err(format!(
            "Story ending catalog changed since it was opened; expected `{expected}`, current `{actual}`. Reload before saving."
        ));
    }
    Ok(())
}

async fn validate_story_ending_references(
    project_root: &Path,
    endings: &[StoryEndingDefinition],
) -> Result<(), String> {
    let scene_catalog = build_scene_asset_catalog(project_root)?;
    let scene_ids = scene_catalog
        .scenes
        .iter()
        .map(|scene| scene.id.as_str())
        .collect::<HashSet<_>>();

    let dialogue_root = project_root.join("dialogue");
    let mut dialogue_manager = DialogueManager::new();
    if dialogue_root.is_dir() {
        dialogue_manager
            .load_from_directory(&dialogue_root)
            .await
            .map_err(|error| format!("Failed to load project dialogues: {error}"))?;
    }

    for ending in endings {
        if !scene_ids.contains(ending.scene_id.as_str()) {
            return Err(format!(
                "Story ending `{}` references unknown scene `{}`.",
                ending.id, ending.scene_id
            ));
        }
        if !dialogue_manager.has_script(&ending.dialogue_id) {
            return Err(format!(
                "Story ending `{}` references unknown dialogue `{}`.",
                ending.id, ending.dialogue_id
            ));
        }
    }
    Ok(())
}

async fn ensure_story_ending_directory(project_root: &Path) -> Result<PathBuf, String> {
    let ending_root = project_root.join("endings");
    tokio::fs::create_dir_all(&ending_root)
        .await
        .map_err(|error| {
            format!(
                "Failed to create story ending directory `{}`: {error}",
                ending_root.display()
            )
        })?;
    let metadata = std::fs::symlink_metadata(&ending_root).map_err(|error| {
        format!(
            "Failed to inspect story ending directory `{}`: {error}",
            ending_root.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "Story ending path must be a regular directory: {}",
            ending_root.display()
        ));
    }
    ending_root.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve story ending directory `{}`: {error}",
            ending_root.display()
        )
    })
}

fn ending_stage_paths(target_path: &Path) -> Result<(PathBuf, PathBuf), String> {
    let parent = target_path
        .parent()
        .ok_or_else(|| "Story ending target has no parent directory.".to_string())?;
    let file_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Story ending target filename is not valid UTF-8.".to_string())?;
    let nonce = ENDING_STAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let stem = format!(".{file_name}.{}.{}", std::process::id(), nonce);
    Ok((
        parent.join(format!("{stem}.tmp")),
        parent.join(format!("{stem}.bak")),
    ))
}

async fn replace_story_ending_document(
    target_path: &Path,
    temp_path: &Path,
    backup_path: &Path,
    content: &[u8],
) -> Result<bool, String> {
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temp_path)
        .await
        .map_err(|error| {
            format!(
                "Failed to stage story ending `{}`: {error}",
                temp_path.display()
            )
        })?;
    if let Err(error) = file.write_all(content).await {
        drop(file);
        let _ = tokio::fs::remove_file(temp_path).await;
        return Err(format!(
            "Failed to write staged story ending `{}`: {error}",
            temp_path.display()
        ));
    }
    if let Err(error) = file.flush().await {
        drop(file);
        let _ = tokio::fs::remove_file(temp_path).await;
        return Err(format!(
            "Failed to flush staged story ending `{}`: {error}",
            temp_path.display()
        ));
    }
    if let Err(error) = file.sync_all().await {
        drop(file);
        let _ = tokio::fs::remove_file(temp_path).await;
        return Err(format!(
            "Failed to sync staged story ending `{}`: {error}",
            temp_path.display()
        ));
    }
    drop(file);

    let had_target = target_path.exists();
    if had_target {
        if let Err(error) = tokio::fs::rename(target_path, backup_path).await {
            let _ = tokio::fs::remove_file(temp_path).await;
            return Err(format!(
                "Failed to back up story ending `{}`: {error}",
                target_path.display()
            ));
        }
    }
    if let Err(error) = tokio::fs::rename(temp_path, target_path).await {
        if had_target {
            let _ = tokio::fs::rename(backup_path, target_path).await;
        }
        let _ = tokio::fs::remove_file(temp_path).await;
        return Err(format!(
            "Failed to replace story ending `{}`: {error}",
            target_path.display()
        ));
    }
    Ok(had_target)
}

async fn rollback_story_ending_document(
    target_path: &Path,
    backup_path: &Path,
    had_target: bool,
) -> Result<(), String> {
    if target_path.exists() {
        tokio::fs::remove_file(target_path).await.map_err(|error| {
            format!(
                "Failed to remove rejected story ending `{}`: {error}",
                target_path.display()
            )
        })?;
    }
    if had_target {
        tokio::fs::rename(backup_path, target_path)
            .await
            .map_err(|error| {
                format!(
                    "Failed to restore story ending `{}`: {error}",
                    target_path.display()
                )
            })?;
    }
    Ok(())
}

async fn restore_deleted_story_ending(
    target_path: &Path,
    backup_path: &Path,
) -> Result<(), String> {
    tokio::fs::rename(backup_path, target_path)
        .await
        .map_err(|error| {
            format!(
                "Failed to restore deleted story ending `{}`: {error}",
                target_path.display()
            )
        })
}

async fn cleanup_story_ending_backup(backup_path: &Path) {
    if backup_path.exists() {
        let _ = tokio::fs::remove_file(backup_path).await;
    }
}

fn load_story_endings(project_root: &Path) -> Result<Vec<StoryEndingDefinition>, String> {
    Ok(load_story_ending_sources(project_root)?
        .into_iter()
        .map(|loaded| loaded.ending)
        .collect())
}

fn load_story_ending_sources(project_root: &Path) -> Result<Vec<LoadedStoryEnding>, String> {
    let ending_root = project_root.join("endings");
    if !ending_root.exists() {
        return Ok(Vec::new());
    }
    let root_metadata = std::fs::symlink_metadata(&ending_root).map_err(|error| {
        format!(
            "Failed to inspect story ending directory `{}`: {error}",
            ending_root.display()
        )
    })?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(format!(
            "Story ending path must be a regular directory: {}",
            ending_root.display()
        ));
    }
    let canonical_root = ending_root.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve story ending directory `{}`: {error}",
            ending_root.display()
        )
    })?;
    let mut files = std::fs::read_dir(&ending_root)
        .map_err(|error| {
            format!(
                "Failed to read story ending directory `{}`: {error}",
                ending_root.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<PathBuf>>();
    files.sort();
    if files.len() > MAX_ENDING_FILES {
        return Err(format!(
            "Story ending directory contains {} JSON files; the limit is {MAX_ENDING_FILES}.",
            files.len()
        ));
    }

    let mut seen = HashSet::new();
    let mut endings = Vec::with_capacity(files.len());
    for path in files {
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "Failed to inspect story ending `{}`: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Story ending must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_ENDING_FILE_BYTES {
            return Err(format!(
                "Story ending `{}` is {} bytes; the limit is {MAX_ENDING_FILE_BYTES} bytes.",
                path.display(),
                metadata.len()
            ));
        }
        let canonical_path = path.canonicalize().map_err(|error| {
            format!(
                "Failed to resolve story ending `{}`: {error}",
                path.display()
            )
        })?;
        if !canonical_path.starts_with(&canonical_root) {
            return Err(format!(
                "Story ending escapes the project ending directory: {}",
                path.display()
            ));
        }
        let content = std::fs::read_to_string(&canonical_path).map_err(|error| {
            format!("Failed to read story ending `{}`: {error}", path.display())
        })?;
        let ending: StoryEndingDefinition = serde_json::from_str(&content).map_err(|error| {
            format!("Invalid story ending JSON in `{}`: {error}", path.display())
        })?;
        validate_ending(&ending, &path)?;
        if !seen.insert(ending.id.clone()) {
            return Err(format!("Duplicate story ending id `{}`.", ending.id));
        }
        endings.push(LoadedStoryEnding {
            ending,
            source_path: source_label(project_root, &path),
            absolute_path: canonical_path,
        });
    }
    endings.sort_by(|left, right| left.ending.id.cmp(&right.ending.id));
    Ok(endings)
}

fn validate_ending(ending: &StoryEndingDefinition, path: &Path) -> Result<(), String> {
    if ending.schema != STORY_ENDING_SCHEMA_V1 {
        return Err(format!(
            "Story ending `{}` uses unsupported schema `{}`.",
            path.display(),
            ending.schema
        ));
    }
    for (label, value) in [
        ("id", ending.id.as_str()),
        ("scene_id", ending.scene_id.as_str()),
        ("dialogue_id", ending.dialogue_id.as_str()),
    ] {
        if !is_portable_id(value) {
            return Err(format!(
                "Story ending `{}` has invalid {label} `{value}`.",
                path.display()
            ));
        }
    }
    if ending.title.trim().is_empty() || ending.title.chars().count() > 256 {
        return Err(format!(
            "Story ending `{}` title must contain 1 to 256 characters.",
            ending.id
        ));
    }
    if ending.description.trim().is_empty() || ending.description.chars().count() > 2048 {
        return Err(format!(
            "Story ending `{}` description must contain 1 to 2048 characters.",
            ending.id
        ));
    }
    Ok(())
}

fn is_portable_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.trim() == value
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn story_ending_catalog_fingerprint(endings: &[LoadedStoryEnding]) -> String {
    let entries = endings
        .iter()
        .map(|loaded| {
            json!({
                "source_path": loaded.source_path,
                "ending": loaded.ending,
            })
        })
        .collect::<Vec<Value>>();
    sha256_json(&json!({
        "schema": STORY_ENDING_CATALOG_SCHEMA_V1,
        "endings": entries,
    }))
}

fn story_ending_content_fingerprint(ending: &StoryEndingDefinition) -> String {
    sha256_json(&json!({
        "schema": STORY_ENDING_SCHEMA_V1,
        "ending": ending,
    }))
}

fn sha256_json(value: &Value) -> String {
    let encoded = serde_json::to_vec(value).expect("story ending fingerprint should serialize");
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    format!("{:x}", hasher.finalize())
}

fn source_label(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story_events::StoryEventCatalog;

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_story_ending_{label}_{}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn write_project(root: &Path) {
        for directory in [
            "characters",
            "knowledge",
            "events",
            "endings",
            "scenes",
            "dialogue",
        ] {
            std::fs::create_dir_all(root.join(directory)).unwrap();
        }
        std::fs::write(
            root.join("events").join("events.json"),
            r#"{
              "schema":"monogatari-story-event-catalog/v1",
              "events":[{
                "event_id":"finale","event_type":"ending","description":"Finale",
                "actions":[
                  {"type":"unlock_ending","ending_id":"finale"},
                  {"type":"unlock_scene","scene_id":"finale_scene"},
                  {"type":"unlock_dialogue","dialogue_id":"finale_dialogue"}
                ]
              }]
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("endings").join("finale.json"),
            r#"{
              "schema":"monogatari-story-ending/v1","id":"finale","title":"Finale",
              "description":"The final scene.","scene_id":"finale_scene","dialogue_id":"finale_dialogue"
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("scenes").join("finale_scene.json"),
            r#"{"id":"finale_scene","name":"Finale","background_path":null}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("dialogue").join("finale_dialogue.json"),
            r#"{
              "id":"finale_dialogue","title":"Finale","start_node_id":"start",
              "nodes":{"start":{"speaker_id":null,"text":"The end.","choices":[]}}
            }"#,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn ending_launch_enforces_unlocks_then_starts_scene_and_dialogue() {
        let root = temp_root("launch");
        write_project(&root);
        let state = AppState::new();
        state.set_project_data_root(root.clone()).await;
        *state.story_event_catalog.write().await =
            StoryEventCatalog::load_from_project_root(&root).unwrap();

        assert!(start_story_ending_inner(&state, "finale").await.is_err());
        let preview = preview_story_ending_inner(&state, "finale").await.unwrap();
        assert_eq!(preview.ending.id, "finale");
        assert!(preview.dialogue.is_active);
        {
            let mut progress = state.story_progress.write().await;
            progress.unlocked_ending_ids.insert("finale".to_string());
            progress
                .unlocked_scene_ids
                .insert("finale_scene".to_string());
            progress
                .unlocked_dialogue_ids
                .insert("finale_dialogue".to_string());
        }

        let launch = start_story_ending_inner(&state, "finale").await.unwrap();
        assert_eq!(launch.ending.id, "finale");
        assert_eq!(launch.scene.id, "finale_scene");
        assert!(launch.dialogue.is_active);
        assert_eq!(
            state.active_scene_id.read().await.as_deref(),
            Some("finale_scene")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn ending_save_is_atomic_and_rejects_stale_or_invalid_updates() {
        let root = temp_root("save");
        write_project(&root);
        let state = AppState::new();
        state.set_project_data_root(root.clone()).await;
        *state.story_event_catalog.write().await =
            StoryEventCatalog::load_from_project_root(&root).unwrap();

        let before = ending_catalog_snapshot(&state).await.unwrap();
        assert_eq!(before.ending_count, 1);
        let mut replacement = before.endings[0].ending.clone();
        replacement.title = "A Better Finale".to_string();

        let saved = save_story_ending_inner(
            &state,
            replacement.clone(),
            Some("finale"),
            &before.catalog_fingerprint,
        )
        .await
        .unwrap();
        assert_ne!(saved.catalog_fingerprint, before.catalog_fingerprint);
        assert_eq!(saved.endings[0].ending.title, "A Better Finale");
        let saved_file = std::fs::read_to_string(root.join("endings").join("finale.json")).unwrap();
        assert!(saved_file.contains("A Better Finale"));

        let mut stale = replacement.clone();
        stale.title = "Stale Finale".to_string();
        assert!(save_story_ending_inner(
            &state,
            stale,
            Some("finale"),
            &before.catalog_fingerprint,
        )
        .await
        .unwrap_err()
        .contains("changed since it was opened"));

        let file_before_invalid =
            std::fs::read_to_string(root.join("endings").join("finale.json")).unwrap();
        let mut invalid = replacement;
        invalid.dialogue_id = "missing_dialogue".to_string();
        assert!(save_story_ending_inner(
            &state,
            invalid,
            Some("finale"),
            &saved.catalog_fingerprint,
        )
        .await
        .unwrap_err()
        .contains("unknown dialogue"));
        assert_eq!(
            std::fs::read_to_string(root.join("endings").join("finale.json")).unwrap(),
            file_before_invalid
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn ending_delete_requires_event_references_to_be_removed_first() {
        let root = temp_root("delete");
        write_project(&root);
        let state = AppState::new();
        state.set_project_data_root(root.clone()).await;
        *state.story_event_catalog.write().await =
            StoryEventCatalog::load_from_project_root(&root).unwrap();
        let snapshot = ending_catalog_snapshot(&state).await.unwrap();

        assert!(
            delete_story_ending_inner(&state, "finale", &snapshot.catalog_fingerprint)
                .await
                .unwrap_err()
                .contains("still unlocked by event")
        );
        assert!(root.join("endings").join("finale.json").is_file());

        std::fs::write(
            root.join("events").join("events.json"),
            r#"{"schema":"monogatari-story-event-catalog/v1","events":[]}"#,
        )
        .unwrap();
        let after = delete_story_ending_inner(&state, "finale", &snapshot.catalog_fingerprint)
            .await
            .unwrap();
        assert_eq!(after.ending_count, 0);
        assert!(!root.join("endings").join("finale.json").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ending_loader_rejects_duplicates_and_unknown_fields() {
        let root = temp_root("invalid");
        std::fs::create_dir_all(root.join("endings")).unwrap();
        std::fs::write(
            root.join("endings").join("invalid.json"),
            r#"{
              "schema":"monogatari-story-ending/v1","id":"ending","title":"Ending",
              "description":"Description","scene_id":"scene","dialogue_id":"dialogue","extra":true
            }"#,
        )
        .unwrap();

        assert!(load_story_endings(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
