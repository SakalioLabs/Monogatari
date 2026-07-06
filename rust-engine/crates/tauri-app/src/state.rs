//! Application state management.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use llm_ai::InferencePipeline;
use llm_assets::{AssetManager, SaveManager};
use llm_game::{
    characters::CharacterManager, dialogue::DialogueManager, knowledge::KnowledgeBase,
    scenes::SceneManager,
};
use llm_scripting::ScriptEngine;

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
}

impl AppState {
    /// Create a new application state with default paths.
    pub fn new() -> Self {
        let data_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("data");

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
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
