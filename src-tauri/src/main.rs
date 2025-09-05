// SIMPLE WORKING TAURI APP - Start with database, add features incrementally
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

// Test command to verify database works
#[tauri::command]
async fn test_database() -> Result<String, String> {
    println!("Testing database connection...");
    
    // Initialize and test the REAL database
    match hive_ai::core::database::get_database().await {
        Ok(db) => {
            println!("Database connected!");
            // Try to get statistics to verify it's working
            match db.get_statistics().await {
                Ok(stats) => Ok(format!("Database connected! {} conversations, {} messages", 
                    stats.conversation_count, stats.message_count)),
                Err(e) => Ok(format!("Database connected but query failed: {}", e))
            }
        },
        Err(e) => {
            println!("Database error: {}", e);
            Err(format!("Database error: {}", e))
        }
    }
}

// Get real profiles from the database
#[tauri::command]
async fn get_profiles() -> Result<Vec<serde_json::Value>, String> {
    println!("Loading profiles...");
    
    use hive_ai::consensus::profiles::ExpertProfileManager;
    
    let manager = ExpertProfileManager::new();
    let templates = manager.get_templates();
    
    println!("Found {} profiles", templates.len());
    
    let profiles: Vec<serde_json::Value> = templates.iter().map(|t| {
        serde_json::json!({
            "id": t.id,
            "name": t.name,
            "description": t.description,
            "category": format!("{:?}", t.category),
        })
    }).collect();
    
    Ok(profiles)
}

fn main() {
    // Simple logging to console
    println!("Starting Tauri app...");
    
    tauri::Builder::default()
        .setup(|app| {
            println!("Setting up Tauri window...");
            let window = app.get_webview_window("main").unwrap();
            
            // Don't auto-open dev tools - let the actual app show
            // #[cfg(debug_assertions)]
            // window.open_devtools();
            
            println!("Tauri setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            test_database,
            get_profiles,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}