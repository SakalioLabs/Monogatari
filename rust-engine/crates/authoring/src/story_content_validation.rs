//! Headless scene and ending catalogs shared by desktop and agent validation.

use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use llm_game::dialogue::DialogueManager;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const STORY_ENDING_SCHEMA_V1: &str = "monogatari-story-ending/v1";
const BACKGROUND_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "gif", "svg"];
const MODEL_3D_EXTENSIONS: &[&str] = &["glb", "gltf"];
const MAX_SCENE_FILES: usize = 512;
const MAX_SCENE_FILE_BYTES: u64 = 64 * 1024;
const MAX_SCENE_TAGS: usize = 64;
const MAX_ENDING_FILES: usize = 256;
const MAX_ENDING_FILE_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SceneDefinition {
    pub id: String,
    pub name: String,
    #[serde(default, alias = "backgroundPath")]
    pub background_path: Option<String>,
    #[serde(default, alias = "model3dPath", alias = "model3DPath")]
    pub model_3d_path: Option<String>,
    #[serde(default, alias = "bgmPath")]
    pub bgm_path: Option<String>,
    #[serde(default)]
    pub weather: Option<String>,
    #[serde(default)]
    pub time_of_day: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StoryEndingDefinition {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub description: String,
    pub scene_id: String,
    pub dialogue_id: String,
}

#[derive(Debug, Clone)]
pub struct LoadedSceneDefinition {
    pub scene: SceneDefinition,
    pub source_path: String,
    pub absolute_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LoadedStoryEnding {
    pub ending: StoryEndingDefinition,
    pub source_path: String,
    pub absolute_path: PathBuf,
}

pub fn load_scene_documents(project_root: &Path) -> Result<Vec<LoadedSceneDefinition>, String> {
    let files = regular_json_files(
        project_root,
        "scenes",
        MAX_SCENE_FILES,
        MAX_SCENE_FILE_BYTES,
    )?;
    let mut seen = HashSet::new();
    let mut loaded = Vec::with_capacity(files.len());
    for path in files {
        let content = std::fs::read_to_string(&path).map_err(|error| {
            format!(
                "Failed to read scene metadata `{}`: {error}",
                path.display()
            )
        })?;
        let scene = serde_json::from_str(&content).map_err(|error| {
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
            absolute_path: path,
        });
    }
    loaded.sort_by(|left, right| left.scene.id.cmp(&right.scene.id));
    Ok(loaded)
}

pub fn load_story_ending_sources(project_root: &Path) -> Result<Vec<LoadedStoryEnding>, String> {
    let files = regular_json_files(
        project_root,
        "endings",
        MAX_ENDING_FILES,
        MAX_ENDING_FILE_BYTES,
    )?;
    let mut seen = HashSet::new();
    let mut loaded = Vec::with_capacity(files.len());
    for path in files {
        let content = std::fs::read_to_string(&path).map_err(|error| {
            format!("Failed to read story ending `{}`: {error}", path.display())
        })?;
        let ending: StoryEndingDefinition = serde_json::from_str(&content).map_err(|error| {
            format!("Invalid story ending JSON in `{}`: {error}", path.display())
        })?;
        validate_ending(&ending, &path)?;
        if !seen.insert(ending.id.clone()) {
            return Err(format!("Duplicate story ending id `{}`.", ending.id));
        }
        loaded.push(LoadedStoryEnding {
            ending,
            source_path: source_label(project_root, &path),
            absolute_path: path,
        });
    }
    loaded.sort_by(|left, right| left.ending.id.cmp(&right.ending.id));
    Ok(loaded)
}

pub fn scene_ids(
    project_root: &Path,
    scenes: &[LoadedSceneDefinition],
) -> Result<HashSet<String>, String> {
    let mut ids = scenes
        .iter()
        .map(|loaded| loaded.scene.id.clone())
        .collect::<HashSet<_>>();
    for directory in [
        project_root.join("assets/backgrounds"),
        project_root.join("assets/scenes"),
        project_root.join("backgrounds"),
    ] {
        collect_inferred_scene_ids(project_root, &directory, &mut ids)?;
    }
    Ok(ids)
}

pub fn validate_ending_references(
    endings: &[LoadedStoryEnding],
    scene_ids: &HashSet<String>,
    dialogues: &DialogueManager,
) -> Vec<(String, String)> {
    let mut issues = Vec::new();
    for loaded in endings {
        let ending = &loaded.ending;
        if !scene_ids.contains(&ending.scene_id) {
            issues.push((
                "ending_scene_missing".into(),
                format!(
                    "Story ending `{}` references unknown scene `{}`.",
                    ending.id, ending.scene_id
                ),
            ));
        }
        if !dialogues.has_script(&ending.dialogue_id) {
            issues.push((
                "ending_dialogue_missing".into(),
                format!(
                    "Story ending `{}` references unknown dialogue `{}`.",
                    ending.id, ending.dialogue_id
                ),
            ));
        }
    }
    issues
}

fn regular_json_files(
    root: &Path,
    directory: &str,
    max_files: usize,
    max_bytes: u64,
) -> Result<Vec<PathBuf>, String> {
    let directory_path = root.join(directory);
    if !directory_path.exists() {
        return Ok(Vec::new());
    }
    let metadata = std::fs::symlink_metadata(&directory_path)
        .map_err(|error| format!("Failed to inspect `{}`: {error}", directory_path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "Catalog path must be a regular directory: {}",
            directory_path.display()
        ));
    }
    let canonical_root = directory_path
        .canonicalize()
        .map_err(|error| format!("Failed to resolve `{}`: {error}", directory_path.display()))?;
    let mut files = std::fs::read_dir(&directory_path)
        .map_err(|error| format!("Failed to read `{}`: {error}", directory_path.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    files.sort();
    if files.len() > max_files {
        return Err(format!(
            "Catalog `{directory}` contains {} JSON files; the limit is {max_files}.",
            files.len()
        ));
    }
    for path in &mut files {
        let metadata = std::fs::symlink_metadata(&*path)
            .map_err(|error| format!("Failed to inspect `{}`: {error}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Catalog document must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > max_bytes {
            return Err(format!(
                "Catalog document `{}` is {} bytes; the limit is {max_bytes} bytes.",
                path.display(),
                metadata.len()
            ));
        }
        let canonical = path
            .canonicalize()
            .map_err(|error| format!("Failed to resolve `{}`: {error}", path.display()))?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "Catalog document escapes its directory: {}",
                path.display()
            ));
        }
        *path = canonical;
    }
    Ok(files)
}

pub fn normalize_scene_definition(mut scene: SceneDefinition) -> SceneDefinition {
    scene.id = scene.id.trim().to_string();
    scene.name = scene.name.trim().to_string();
    scene.background_path = normalize_optional(scene.background_path);
    scene.model_3d_path = normalize_optional(scene.model_3d_path);
    scene.bgm_path = normalize_optional(scene.bgm_path);
    scene.weather = normalize_optional(scene.weather);
    scene.time_of_day = normalize_optional(scene.time_of_day);
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

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn validate_scene_definition(root: &Path, scene: &SceneDefinition) -> Result<(), String> {
    if !is_portable_id(&scene.id) {
        return Err(format!("Scene id `{}` is not a portable id.", scene.id));
    }
    bounded(&scene.name, "name", 256, &scene.id)?;
    for (label, value) in [
        ("weather", scene.weather.as_deref()),
        ("time_of_day", scene.time_of_day.as_deref()),
    ] {
        if let Some(value) = value {
            bounded(value, label, 64, &scene.id)?;
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
        bounded(tag, "tag", 64, &scene.id)?;
    }
    if let Some(path) = &scene.background_path {
        let resolved = resolve_project_relative(root, path)?;
        if !supported_background(Path::new(path)) {
            return Err(format!(
                "Scene `{}` background uses an unsupported file type: {path}",
                scene.id
            ));
        }
        if !resolved.is_file() {
            return Err(format!(
                "Scene `{}` background does not exist: {path}",
                scene.id
            ));
        }
    }
    if let Some(path) = &scene.model_3d_path {
        let resolved = resolve_project_relative(root, path)?;
        if !supported_extension(Path::new(path), MODEL_3D_EXTENSIONS) {
            return Err(format!(
                "Scene `{}` 3D model uses an unsupported file type: {path}",
                scene.id
            ));
        }
        if !resolved.is_file() {
            return Err(format!(
                "Scene `{}` 3D model does not exist: {path}",
                scene.id
            ));
        }
    }
    if let Some(path) = &scene.bgm_path {
        resolve_project_relative(root, path)?;
        let extension = Path::new(path)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !["mp3", "ogg", "wav", "m4a", "aac", "flac"].contains(&extension.as_str()) {
            return Err(format!(
                "Scene `{}` BGM uses an unsupported file type: {path}",
                scene.id
            ));
        }
    }
    Ok(())
}

pub fn validate_ending(ending: &StoryEndingDefinition, path: &Path) -> Result<(), String> {
    if ending.schema != STORY_ENDING_SCHEMA_V1 {
        return Err(format!(
            "Story ending `{}` uses unsupported schema `{}`.",
            path.display(),
            ending.schema
        ));
    }
    for (label, value) in [
        ("id", &ending.id),
        ("scene_id", &ending.scene_id),
        ("dialogue_id", &ending.dialogue_id),
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

fn bounded(value: &str, label: &str, max: usize, id: &str) -> Result<(), String> {
    let count = value.chars().count();
    if count == 0 || count > max || value.chars().any(char::is_control) {
        return Err(format!(
            "Scene `{id}` {label} must contain 1 to {max} non-control characters."
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
fn supported_background(path: &Path) -> bool {
    supported_extension(path, BACKGROUND_EXTENSIONS)
}

fn supported_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| extensions.contains(&value.to_ascii_lowercase().as_str()))
}

fn resolve_project_relative(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty()
        || relative.trim() != relative
        || relative.contains('\\')
        || relative.contains(':')
        || relative.chars().any(char::is_control)
        || relative
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err("Asset paths must use portable non-empty project-relative segments.".into());
    }
    let path = Path::new(relative);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err("Asset paths cannot escape the project root.".into());
    }
    Ok(root.join(path))
}

fn collect_inferred_scene_ids(
    root: &Path,
    directory: &Path,
    ids: &mut HashSet<String>,
) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }
    let mut stack = vec![directory.to_path_buf()];
    while let Some(current) = stack.pop() {
        let mut entries = std::fs::read_dir(&current)
            .map_err(|error| {
                format!(
                    "Failed to read background directory `{}`: {error}",
                    current.display()
                )
            })?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                stack.push(path);
            } else if supported_background(&path) {
                ids.insert(derive_scene_id(&source_label(root, &path)));
            }
        }
    }
    Ok(())
}

fn derive_scene_id(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
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

fn source_label(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests;
