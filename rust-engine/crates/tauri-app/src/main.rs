//! # LLM Galgame Engine - Tauri Application
//!
//! Main entry point for the Tauri desktop application.
//! Provides Tauri commands for frontend-backend communication.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use state::AppState;
use tracing_subscriber::EnvFilter;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    tracing::info!("Starting LLM Galgame Engine...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Engine commands
            commands::engine::initialize_engine,
            commands::engine::get_engine_status,
            // Character commands
            commands::characters::get_characters,
            commands::characters::get_character,
            commands::characters::load_characters,
            // Dialogue commands
            commands::dialogue::start_dialogue,
            commands::dialogue::advance_dialogue,
            commands::dialogue::select_choice,
            commands::dialogue::get_dialogue_state,
            commands::dialogue::load_dialogues,
            // Knowledge commands
            commands::knowledge::search_knowledge,
            commands::knowledge::load_knowledge,
            // AI commands
            commands::ai::configure_api,
            commands::ai::configure_onnx,
            commands::ai::generate_response,
            commands::ai::generate_stream,
            commands::ai::get_ai_status,
            // Chat commands (core feature)
            commands::chat::send_chat_message,
            commands::chat::send_chat_message_stream,
            commands::chat::get_chat_history,
            commands::chat::clear_chat_history,
            commands::chat::evaluate_conversation,
            commands::chat::get_relationship_score,
            commands::chat::get_available_events,
            // Save/Load commands
            commands::save::save_game,
            commands::save::load_game,
            commands::save::list_saves,
            commands::save::delete_save,
            // Workflow commands
            commands::workflow::get_workflow_nodes,
            commands::workflow::execute_workflow_node,
            commands::workflow::save_workflow,
            commands::workflow::load_workflow,
            // Script commands
            commands::script::execute_script,
            commands::script::evaluate_condition,
            commands::script::parse_script,
            // Live2D commands
            commands::live2d::load_model,
            commands::live2d::set_expression,
            commands::live2d::set_motion,
            commands::live2d::get_model_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

