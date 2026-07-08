//! Application state management.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;

use llm_ai::InferencePipeline;
use llm_assets::{AssetManager, SaveManager};
use llm_game::{
    characters::CharacterManager, dialogue::DialogueManager, knowledge::KnowledgeBase,
    scenes::SceneManager,
};
use llm_scripting::ScriptEngine;

use crate::commands::chat::ChatSession;

/// Main application state shared across all Tauri commands.
pub struct AppState {
    pub character_manager: Arc<RwLock<CharacterManager>>,
    pub dialogue_manager: Arc<RwLock<DialogueManager>>,
    pub knowledge_base: Arc<RwLock<KnowledgeBase>>,
    pub scene_manager: Arc<RwLock<SceneManager>>,
    pub inference_pipeline: Arc<RwLock<InferencePipeline>>,
    pub save_manager: Arc<RwLock<SaveManager>>,
    #[allow(dead_code)]
    pub asset_manager: Arc<RwLock<AssetManager>>,
    pub script_engine: Arc<RwLock<ScriptEngine>>,
    pub project_path: Arc<RwLock<Option<PathBuf>>>,
    pub initialized: Arc<RwLock<bool>>,
    /// Active authoring/runtime scene selected from the scene asset catalog.
    pub active_scene_id: Arc<RwLock<Option<String>>>,
    pub scene_history: Arc<RwLock<Vec<String>>>,
    /// Chat sessions keyed by character_id.
    pub chat_sessions: Arc<RwLock<HashMap<String, ChatSession>>>,
}

impl AppState {
    /// Create a new application state with default paths.
    pub fn new() -> Self {
        let data_path = default_project_data_root();

        Self {
            character_manager: Arc::new(RwLock::new(CharacterManager::new())),
            dialogue_manager: Arc::new(RwLock::new(DialogueManager::new())),
            knowledge_base: Arc::new(RwLock::new(KnowledgeBase::new())),
            scene_manager: Arc::new(RwLock::new(SceneManager::new())),
            inference_pipeline: Arc::new(RwLock::new(InferencePipeline::new())),
            asset_manager: Arc::new(RwLock::new(AssetManager::new(&data_path))),
            save_manager: Arc::new(RwLock::new(SaveManager::new(data_path.join("saves")))),
            script_engine: Arc::new(RwLock::new(ScriptEngine::new())),
            project_path: Arc::new(RwLock::new(None)),
            initialized: Arc::new(RwLock::new(false)),
            active_scene_id: Arc::new(RwLock::new(None)),
            scene_history: Arc::new(RwLock::new(Vec::new())),
            chat_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Rebind project-scoped managers to a discovered project data root.
    pub async fn set_project_data_root(&self, data_path: PathBuf) {
        *self.asset_manager.write().await = AssetManager::new(&data_path);
        *self.save_manager.write().await = SaveManager::new(data_path.join("saves"));
        *self.project_path.write().await = Some(data_path);
    }

    /// Resolve the active project data root for project-scoped commands.
    pub async fn current_project_data_root(&self) -> PathBuf {
        self.project_path
            .read()
            .await
            .clone()
            .unwrap_or_else(default_project_data_root)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve the best development/default project data root from the process working directory.
pub fn default_project_data_root() -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    discover_project_data_root(&current_dir).unwrap_or_else(|| current_dir.join("data"))
}

/// Discover a project data root by walking upward from a start path.
pub fn discover_project_data_root(start: &Path) -> Option<PathBuf> {
    let mut first_valid = None;

    for ancestor in start.ancestors() {
        for candidate in [ancestor.to_path_buf(), ancestor.join("data")] {
            if !is_project_data_root(&candidate) {
                continue;
            }
            if first_valid.is_none() {
                first_valid = Some(candidate.clone());
            }
            if candidate.join("quality_suites").is_dir() {
                return Some(candidate);
            }
        }
    }

    first_valid
}

/// Discover a bundled `data/` resource emitted by Tauri installers/builds.
pub fn discover_bundled_project_data_root(resource_dir: &Path) -> Option<PathBuf> {
    [resource_dir.join("data"), resource_dir.to_path_buf()]
        .into_iter()
        .find(|candidate| is_project_data_root(candidate))
}

/// A project data root must include the core authoring/runtime content folders.
pub fn is_project_data_root(path: &Path) -> bool {
    path.join("characters").is_dir() && path.join("knowledge").is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{prefix}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn make_data_root(root: &Path, with_quality_suites: bool) {
        std::fs::create_dir_all(root.join("characters")).unwrap();
        std::fs::create_dir_all(root.join("knowledge")).unwrap();
        if with_quality_suites {
            std::fs::create_dir_all(root.join("quality_suites")).unwrap();
        }
    }

    #[test]
    fn discovers_project_data_root_from_nested_working_dir() {
        let root = temp_root("monogatari_state_data_root");
        let data_root = root.join("data");
        let nested = root.join("rust-engine").join("crates").join("tauri-app");
        make_data_root(&data_root, true);
        std::fs::create_dir_all(&nested).unwrap();

        let found = discover_project_data_root(&nested);
        assert_eq!(found.as_deref(), Some(data_root.as_path()));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn discovers_bundled_data_root_from_tauri_resource_dir() {
        let resource_dir = temp_root("monogatari_state_resource_dir");
        let bundled_data = resource_dir.join("data");
        make_data_root(&bundled_data, true);

        let found = discover_bundled_project_data_root(&resource_dir);
        assert_eq!(found.as_deref(), Some(bundled_data.as_path()));

        std::fs::remove_dir_all(resource_dir).unwrap();
    }
}
