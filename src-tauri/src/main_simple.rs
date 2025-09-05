// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("hive=info".parse().unwrap())
        )
        .init();

    tracing::info!("🚀 Starting Hive Tauri Application (Simplified)");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            tracing::info!("Setting up window...");
            
            let window = app.get_webview_window("main").unwrap();
            window.show().unwrap();
            window.set_focus().unwrap();
            
            #[cfg(debug_assertions)]
            {
                window.open_devtools();
                tracing::info!("DevTools opened");
            }
            
            tracing::info!("✅ Window setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! This is Hive Consensus.", name)
}