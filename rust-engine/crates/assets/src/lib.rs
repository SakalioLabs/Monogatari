//! # LLM Galgame Engine - Assets
//!
//! Asset management and game save/load system.

pub mod asset_manager;
pub mod save_manager;

pub use asset_manager::AssetManager;
pub use save_manager::{
    CharacterSaveState, GameSave, SaveManager, GAME_SAVE_SCHEMA_V1, GAME_SAVE_SCHEMA_V2,
    GAME_SAVE_SCHEMA_V3, GAME_SAVE_SCHEMA_V4, MAX_GAME_SAVE_BYTES,
};
