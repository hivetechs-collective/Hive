// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use std::sync::Arc;
use tokio::sync::Mutex;

mod commands;
mod state;

use state::AppState;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("hive=info".parse().unwrap())
        )
        .init();

    tracing::info!("🚀 Starting Hive Tauri Application");

    // Run the Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Initialize application state
            tauri::async_runtime::block_on(async move {
                match AppState::new().await {
                    Ok(state) => {
                        app_handle.manage(Arc::new(Mutex::new(state)));
                        tracing::info!("✅ Application state initialized successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize application state: {}", e);
                        // Still continue, but with limited functionality
                    }
                }
            });

            // Setup window
            if let Some(window) = app.get_webview_window("main") {
                // Ensure window is visible and focused
                window.show().unwrap();
                window.set_focus().unwrap();
                window.center().unwrap();
                
                #[cfg(debug_assertions)]
                {
                    window.open_devtools();
                    tracing::info!("DevTools opened for debugging");
                }
                
                tracing::info!("✨ Window created and shown successfully");
            } else {
                tracing::error!("❌ Failed to get main window! Creating new window...");
                
                // Create window manually if it doesn't exist
                let window = tauri::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::External("http://localhost:5173/test.html".parse().unwrap())
                )
                .title("Hive Consensus")
                .inner_size(1400.0, 900.0)
                .min_inner_size(800.0, 600.0)
                .center()
                .visible(true)
                .build()
                .expect("Failed to create window");
                
                window.show().unwrap();
                window.set_focus().unwrap();
                
                #[cfg(debug_assertions)]
                {
                    window.open_devtools();
                }
                
                tracing::info!("✅ Manually created window");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Consensus commands
            commands::consensus::run_consensus,
            commands::consensus::run_consensus_streaming,
            commands::consensus::cancel_consensus,
            commands::consensus::get_consensus_status,
            commands::consensus::get_active_profile,
            commands::consensus::set_active_profile,
            commands::consensus::get_profiles,
            
            // File system commands
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
            
            // Terminal commands (if we implement terminal support)
            commands::terminal::create_terminal,
            commands::terminal::write_to_terminal,
            commands::terminal::resize_terminal,
            commands::terminal::close_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Custom error type for better error handling
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Consensus error: {0}")]
    Consensus(String),
    
    #[error("File system error: {0}")]
    FileSystem(String),
    
    #[error("Settings error: {0}")]
    Settings(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}