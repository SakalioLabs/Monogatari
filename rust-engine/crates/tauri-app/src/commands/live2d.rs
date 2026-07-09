//! Live2D model management commands.

use serde::Serialize;
use std::path::{Component, Path, PathBuf};
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct Live2DModelInfo {
    pub model_path: String,
    pub expressions: Vec<String>,
    pub motions: Vec<String>,
    pub current_expression: Option<String>,
    pub current_motion: Option<String>,
}

fn live2d_model_path_in_project(
    project_root: &Path,
    model_path: &str,
) -> Result<(String, PathBuf), String> {
    let segments = normalize_live2d_model_ref(model_path)?;
    let relative = segments.join("/");
    let path = segments
        .iter()
        .fold(project_root.to_path_buf(), |path, segment| {
            path.join(segment)
        });

    if !path.starts_with(project_root) {
        return Err("Live2D model path must stay inside active project data root.".to_string());
    }

    Ok((relative, path))
}

fn normalize_live2d_model_ref(model_path: &str) -> Result<Vec<String>, String> {
    let normalized = model_path.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.chars().any(char::is_control) {
        return Err(
            "Live2D model paths must be non-empty and cannot contain control characters."
                .to_string(),
        );
    }
    if normalized.contains(':') {
        return Err("Live2D model paths cannot contain drive prefixes or URI schemes.".to_string());
    }

    let segments = normalized.split('/').collect::<Vec<_>>();
    if segments
        .iter()
        .any(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    {
        return Err(
            "Live2D model paths cannot contain empty, current, or parent directory segments."
                .to_string(),
        );
    }
    if segments.iter().any(|segment| {
        !segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    }) {
        return Err(
            "Live2D model paths can contain only ASCII letters, numbers, underscores, hyphens, dots, or separators."
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
            "Live2D model paths must be relative to the active project data root.".to_string(),
        );
    }

    let lower = normalized.to_ascii_lowercase();
    if !(lower.ends_with(".model3.json") || lower.ends_with(".json")) {
        return Err("Live2D model paths must point to a .model3.json or .json file.".to_string());
    }

    Ok(segments.into_iter().map(str::to_string).collect())
}

fn load_live2d_model_from_project(
    project_root: &Path,
    model_path: &str,
) -> Result<Live2DModelInfo, String> {
    let (relative_path, path) = live2d_model_path_in_project(project_root, model_path)?;
    if !path.is_file() {
        return Err(format!("Live2D model file does not exist: {relative_path}"));
    }

    let canonical_root = project_root
        .canonicalize()
        .map_err(|e| format!("Project root unavailable for Live2D model lookup: {e}"))?;
    let canonical_model = path
        .canonicalize()
        .map_err(|e| format!("Live2D model file does not exist: {relative_path}: {e}"))?;
    if !canonical_model.starts_with(&canonical_root) {
        return Err("Live2D model path must stay inside active project data root.".to_string());
    }

    let model_dir = canonical_model.parent().unwrap_or(canonical_root.as_path());
    let expressions = collect_live2d_sidecars(&model_dir.join("expressions"), "default");
    let motions = collect_live2d_sidecars(&model_dir.join("motions"), "idle");

    Ok(Live2DModelInfo {
        model_path: relative_path,
        expressions,
        motions,
        current_expression: None,
        current_motion: None,
    })
}

fn collect_live2d_sidecars(dir: &Path, fallback: &str) -> Vec<String> {
    let mut names = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    if !entry.file_type().ok()?.is_file() {
                        return None;
                    }
                    live2d_sidecar_name(&entry.path())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    names.sort();
    names.dedup();
    if names.is_empty() {
        vec![fallback.to_string()]
    } else {
        names
    }
}

fn live2d_sidecar_name(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_string_lossy();
    let lower = file_name.to_ascii_lowercase();
    for suffix in [".exp3.json", ".motion3.json", ".json"] {
        if lower.ends_with(suffix) {
            return Some(file_name[..file_name.len() - suffix.len()].to_string());
        }
    }

    path.file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
}

/// Load a Live2D model.
#[tauri::command]
pub async fn load_model(
    state: State<'_, AppState>,
    model_path: String,
) -> Result<Live2DModelInfo, String> {
    // In a full implementation, this would:
    // 1. Load the Live2D Cubism model (.model3.json)
    // 2. Parse expressions and motions from the model config
    // 3. Initialize the WebGL rendering context

    let project_root = state.current_project_data_root().await;
    load_live2d_model_from_project(&project_root, &model_path)
}

/// Set the expression on the active Live2D model.
#[tauri::command]
pub async fn set_expression(
    state: State<'_, AppState>,
    character_id: String,
    expression: String,
) -> Result<String, String> {
    // Update character emotion in the character manager
    let cm = state.character_manager.read().await;
    if let Some(character) = cm.get_character(&character_id) {
        let mut character = character.write().await;
        character.set_emotion(&expression);
    }

    Ok(format!("Expression set to: {expression}"))
}

/// Set the motion on the active Live2D model.
#[tauri::command]
pub async fn set_motion(
    _state: State<'_, AppState>,
    _character_id: String,
    motion_group: String,
    motion_index: Option<usize>,
) -> Result<String, String> {
    // In a full implementation, this would trigger the Live2D motion
    // via the Cubism SDK JavaScript API in the frontend
    Ok(format!(
        "Motion set: {} (index: {:?})",
        motion_group, motion_index
    ))
}

/// Get info about a loaded Live2D model.
#[tauri::command]
pub async fn get_model_info(
    state: State<'_, AppState>,
    character_id: String,
) -> Result<Option<Live2DModelInfo>, String> {
    let model_path = {
        let cm = state.character_manager.read().await;
        if let Some(character) = cm.get_character(&character_id) {
            let character = character.read().await;
            character.live2d_model_path.clone()
        } else {
            None
        }
    };
    if let Some(path) = model_path {
        load_model(state, path).await.map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_live2d_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn live2d_model_paths_resolve_under_project_root() {
        let root = PathBuf::from("project-data");

        assert_eq!(
            live2d_model_path_in_project(&root, "assets/live2d/hero/hero.model3.json").unwrap(),
            (
                "assets/live2d/hero/hero.model3.json".to_string(),
                root.join("assets")
                    .join("live2d")
                    .join("hero")
                    .join("hero.model3.json")
            )
        );
        assert_eq!(
            live2d_model_path_in_project(&root, "live2d\\hero\\hero.json").unwrap(),
            (
                "live2d/hero/hero.json".to_string(),
                root.join("live2d").join("hero").join("hero.json")
            )
        );
    }

    #[test]
    fn live2d_model_paths_reject_escape_attempts() {
        let root = PathBuf::from("project-data");
        for model_path in [
            "",
            "../hero.model3.json",
            "assets/../hero.model3.json",
            "assets//hero.model3.json",
            "assets/./hero.model3.json",
            "C:/Users/example/hero.model3.json",
            "https://example.test/hero.model3.json",
            "/tmp/hero.model3.json",
            "assets/live2d/hero.glb",
            "assets/live2d/hero model.model3.json",
            "assets/live2d/hero!.model3.json",
        ] {
            assert!(
                live2d_model_path_in_project(&root, model_path).is_err(),
                "{model_path} should be rejected"
            );
        }
    }

    #[test]
    fn load_live2d_model_reads_project_model_sidecars() {
        let root = temp_root("sidecars");
        let model_dir = root.join("assets").join("live2d").join("hero");
        std::fs::create_dir_all(model_dir.join("expressions")).unwrap();
        std::fs::create_dir_all(model_dir.join("motions")).unwrap();
        std::fs::write(model_dir.join("hero.model3.json"), "{}").unwrap();
        std::fs::write(model_dir.join("expressions").join("happy.exp3.json"), "{}").unwrap();
        std::fs::write(model_dir.join("motions").join("idle.motion3.json"), "{}").unwrap();

        let info =
            load_live2d_model_from_project(&root, "assets/live2d/hero/hero.model3.json").unwrap();

        assert_eq!(info.model_path, "assets/live2d/hero/hero.model3.json");
        assert_eq!(info.expressions, vec!["happy".to_string()]);
        assert_eq!(info.motions, vec!["idle".to_string()]);
        assert!(load_live2d_model_from_project(&root, "../settings.json").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
