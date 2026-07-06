//! # LLM Galgame Engine - Assets
//!
//! Asset management and game save/load system.

pub mod asset_manager;
pub mod save_manager;

pub use asset_manager::AssetManager;
pub use save_manager::{GameSave, SaveManager};
