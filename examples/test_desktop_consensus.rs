//! Test the desktop chat with consensus integration

use anyhow::Result;
use hive_ai::desktop;
use hive_ai::core::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("hive_ai=debug,dioxus=info")
        .init();
    
    println!("Starting desktop app with consensus integration test...");
    
    // Load config or create default
    let config = Config::default();
    
    // Launch the desktop app
    desktop::launch_desktop_app(config)?;
    
    Ok(())
}