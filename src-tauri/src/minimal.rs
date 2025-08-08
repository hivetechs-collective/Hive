fn main() {
    println!("Starting minimal Tauri app...");
    
    tauri::Builder::default()
        .setup(|app| {
            println!("In setup!");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    println!("Tauri app finished");
}