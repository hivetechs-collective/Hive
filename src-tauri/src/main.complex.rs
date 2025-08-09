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
            
            // Core consensus commands
            bridge::run_consensus,
            bridge::run_consensus_streaming,
            bridge::get_analytics_data,
            
            // Profile management commands
            bridge::profiles::get_available_profiles,
            bridge::profiles::get_profiles,  // React app compatibility
            bridge::profiles::get_active_profile,  // React app compatibility  
            bridge::profiles::set_active_profile,
            bridge::profiles::get_profile_config,
            bridge::profiles::get_custom_profiles,
            bridge::profiles::create_custom_profile,
            bridge::profiles::delete_custom_profile,
            
            // Settings and API key commands
            bridge::settings::get_settings,
            bridge::settings::save_settings,
            bridge::settings::get_api_keys_status,
            bridge::settings::save_api_key_secure,
            bridge::settings::validate_api_key,
            bridge::settings::remove_api_key,
            bridge::settings::get_api_usage,
            
            // Git integration commands
            bridge::git::create_lazygit_terminal,
            bridge::git::get_git_status,
            bridge::git::get_current_branch,
            bridge::git::get_branches,
            bridge::git::switch_branch,
            bridge::git::create_branch,
            bridge::git::stage_files,
            bridge::git::commit_changes,
            
            // Filesystem commands (keep existing)
            commands::filesystem::read_directory,
            commands::filesystem::read_file,
            commands::filesystem::write_file,
            commands::filesystem::create_directory,
            commands::filesystem::delete_file,
            commands::filesystem::get_file_info,
            
            // Terminal commands (now bridged to existing PTY implementation)
            bridge::create_terminal,
            bridge::write_to_terminal,
            bridge::resize_terminal,
            bridge::close_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}