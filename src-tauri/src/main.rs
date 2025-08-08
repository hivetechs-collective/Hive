// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod bridge;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Hive Consensus.", name)
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Initialize the existing database on startup
    tauri::async_runtime::block_on(async {
        if let Err(e) = bridge::init_database().await {
            eprintln!("Failed to initialize database: {}", e);
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            
            // Bridge commands - thin wrappers to existing functionality
            bridge::run_consensus,
            bridge::run_consensus_streaming,
            bridge::get_profiles,
            bridge::get_analytics_data,
            bridge::get_api_key_status,
            bridge::save_api_key,
            
            // Filesystem commands (keep existing)
            commands::filesystem::read_directory,
            commands::filesystem::read_file,
            commands::filesystem::write_file,
            commands::filesystem::create_directory,
            commands::filesystem::delete_file,
            commands::filesystem::get_file_info,
            
            // Terminal commands (to be bridged)
            commands::terminal::create_terminal,
            commands::terminal::write_to_terminal,
            commands::terminal::resize_terminal,
            commands::terminal::close_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}