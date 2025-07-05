//! Minimal desktop app to test rendering

use dioxus::prelude::*;

fn main() {
    // Enable debug logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    println!("Starting minimal desktop app...");
    
    LaunchBuilder::desktop()
        .launch(app);
}

fn app() -> Element {
    println!("App component rendering...");
    
    rsx! {
        div {
            style: "background: red; color: white; padding: 50px; font-size: 24px; height: 100vh;",
            "If you see this red background with white text, the app is working!"
        }
    }
}