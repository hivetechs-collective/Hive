// Simplest possible Tauri app to test if windows work
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    println!("Starting simple test...");
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    println!("Tauri app finished");
}