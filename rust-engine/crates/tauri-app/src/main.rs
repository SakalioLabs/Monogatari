//! # LLM Galgame Engine - Tauri Application
//!
//! Main entry point for the Tauri desktop application.
//! Provides Tauri commands for frontend-backend communication.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;
mod story_access;
mod story_events;
mod story_progress;

use state::{discover_bundled_project_data_root, is_project_data_root, AppState};
use tauri::Manager;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    tracing::info!("Starting LLM Galgame Engine...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .setup(|app| {
            let development_root = state::default_project_data_root();
            let bundled_root = app
                .path()
                .resource_dir()
                .map(
                    |resource_dir| match discover_bundled_project_data_root(&resource_dir) {
                        Some(root) => {
                            tracing::info!("Found bundled project data at {}", root.display());
                            Some(root)
                        }
                        None => None,
                    },
                )
                .unwrap_or_else(|error| {
                    tracing::warn!("Unable to resolve Tauri resource directory: {error}");
                    None
                });
            let data_root = if is_project_data_root(&development_root) {
                Some(development_root)
            } else {
                bundled_root
            };

            if let Some(data_root) = data_root {
                let app_state = app.state::<AppState>();
                tracing::info!("Using project data root: {}", data_root.display());
                tauri::async_runtime::block_on(app_state.set_project_data_root(data_root));
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::engine::initialize_engine,
            commands::engine::get_engine_status,
            commands::endings::list_story_endings,
            commands::endings::start_story_ending,
            commands::story_events::get_story_event_catalog,
            commands::story_events::get_story_content_access,
            commands::story_events::get_story_progress,
            commands::story_events::reload_story_event_catalog,
            commands::story_events::save_story_event_catalog,
            commands::project::get_project_config,
            commands::project::save_project_config,
            commands::project::export_project,
            commands::characters::get_characters,
            commands::characters::get_character,
            commands::characters::load_characters,
            commands::dialogue::start_dialogue,
            commands::dialogue::list_dialogues,
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
            commands::chat::get_chat_session_audit,
            commands::chat::clear_chat_history,
            commands::chat::evaluate_conversation,
            commands::chat::evaluate_conversation_report,
            commands::chat::get_relationship_score,
            commands::chat::get_available_events,
            commands::chat::preview_event_triggers,
            commands::multi_chat::start_group_chat,
            commands::multi_chat::send_group_message,
            commands::multi_chat::get_group_chat_characters,
            commands::save::save_game,
            commands::save::load_game,
            commands::save::list_saves,
            commands::save::delete_save,
            commands::scenes::list_scene_assets,
            commands::scenes::list_story_scenes,
            commands::scenes::get_current_scene,
            commands::scenes::set_scene,
            commands::scenes::enter_story_scene,
            commands::workflow::get_workflow_nodes,
            commands::workflow::execute_workflow,
            commands::workflow::execute_workflow_node,
            commands::workflow::validate_workflow,
            commands::workflow::save_workflow,
            commands::workflow::load_workflow,
            commands::quality_suite::list_quality_suites,
            commands::quality_suite::run_quality_suite,
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
