//! Test the desktop file explorer functionality

use hive_ai::desktop;
use hive_ai::core::config::Config;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();
    
    // Create a default config
    let config = Config::default();
    
    // Launch the desktop app with file explorer
    desktop::launch_desktop_app(config)?;
    
    Ok(())
}