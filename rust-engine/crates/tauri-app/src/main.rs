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
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    tracing::info!("Starting LLM Galgame Engine...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::engine::initialize_engine,
            commands::engine::get_engine_status,
            commands::project::get_project_config,
            commands::project::save_project_config,
            commands::characters::get_characters,
            commands::characters::get_character,
            commands::characters::load_characters,
            commands::dialogue::start_dialogue,
            commands::dialogue::advance_dialogue,
            commands::dialogue::select_choice,
            commands::dialogue::get_dialogue_state,
            commands::dialogue::load_dialogues,
            commands::knowledge::search_knowledge,
            commands::knowledge::load_knowledge,
            commands::knowledge::list_knowledge_entries,
            commands::knowledge::get_knowledge_entry,
            commands::knowledge::list_knowledge_tags,
            commands::ai::configure_api,
            commands::ai::configure_onnx,
            commands::ai::generate_response,
            commands::ai::generate_stream,
            commands::ai::get_ai_status,
            commands::chat::send_chat_message,
            commands::chat::send_chat_message_stream,
            commands::chat::get_chat_history,
            commands::chat::clear_chat_history,
            commands::chat::evaluate_conversation,
            commands::chat::get_relationship_score,
            commands::chat::get_available_events,
            commands::multi_chat::start_group_chat,
            commands::multi_chat::send_group_message,
            commands::multi_chat::get_group_chat_characters,
            commands::save::save_game,
            commands::save::load_game,
            commands::save::list_saves,
            commands::save::delete_save,
            commands::scenes::list_scene_assets,
            commands::scenes::get_current_scene,
            commands::scenes::set_scene,
            commands::workflow::get_workflow_nodes,
            commands::workflow::execute_workflow_node,
            commands::workflow::validate_workflow,
            commands::workflow::save_workflow,
            commands::workflow::load_workflow,
            commands::script::execute_script,
            commands::script::evaluate_condition,
            commands::script::parse_script,
            commands::live2d::load_model,
            commands::live2d::set_expression,
            commands::live2d::set_motion,
            commands::live2d::get_model_info,
            commands::tts::configure_tts,
            commands::tts::set_character_voice,
            commands::tts::synthesize_speech,
            commands::tts::get_available_voices,
            commands::plugin::list_plugins,
            commands::plugin::register_plugin,
            commands::plugin::remove_plugin,
            commands::cloud_sync::configure_cloud_sync,
            commands::cloud_sync::get_sync_status,
            commands::cloud_sync::push_saves_to_cloud,
            commands::cloud_sync::pull_saves_from_cloud,
            commands::cloud_sync::resolve_sync_conflict,
            commands::analytics::record_analytics_event,
            commands::analytics::get_analytics_summary,
            commands::analytics::export_analytics,
            commands::character_manager::create_character,
            commands::character_manager::delete_character,
            commands::character_manager::get_character_summaries,
            commands::i18n::load_locale,
            commands::i18n::list_locales,
            commands::i18n::translate,
            commands::marketplace::list_marketplace_entries,
            commands::marketplace::export_template,
            commands::marketplace::import_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}