// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use std::sync::Arc;
use tokio::sync::Mutex;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Hive Consensus.", name)
}

fn main() {
    // Initialize app state
    let app_state = Arc::new(Mutex::new(state::AppState::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            // Consensus commands
            commands::consensus::run_consensus,
            commands::consensus::run_consensus_streaming,
            commands::consensus::cancel_consensus,
            commands::consensus::get_consensus_status,
            commands::consensus::get_active_profile,
            commands::consensus::set_active_profile,
            commands::consensus::get_profiles,
            // Filesystem commands
            commands::filesystem::read_directory,
            commands::filesystem::read_file,
            commands::filesystem::write_file,
            commands::filesystem::create_directory,
            commands::filesystem::delete_file,
            commands::filesystem::get_file_info,
            // Analytics commands
            commands::analytics::get_analytics_data,
            commands::analytics::get_cost_breakdown,
            commands::analytics::export_analytics,
            // Settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::get_api_key_status,
            commands::settings::set_api_key,
            // Terminal commands
            commands::terminal::create_terminal,
            commands::terminal::write_to_terminal,
            commands::terminal::resize_terminal,
            commands::terminal::close_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}