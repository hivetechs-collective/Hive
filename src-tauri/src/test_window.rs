use tauri::Manager;

fn main() {
    println!("Starting minimal Tauri app...");
    
    tauri::Builder::default()
        .setup(|app| {
            println!("Setup called!");
            
            // Create window manually
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External("https://google.com".parse().unwrap())
            )
            .title("Test Window")
            .inner_size(800.0, 600.0)
            .visible(true)
            .build()
            .expect("Failed to create window");
            
            window.show().unwrap();
            println!("Window created and shown!");
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}