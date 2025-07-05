//! Simple desktop launcher for Hive AI

use hive_ai::{Config, launch_desktop_app};
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize logging
    if let Err(e) = hive_ai::initialize_default_logging() {
        eprintln!("Failed to initialize logging: {}", e);
    }
    
    tracing::info!("Launching Hive AI Desktop Application");
    
    // Load config or use default
    let config = Config::default();
    
    // Launch desktop app
    launch_desktop_app(config)?;
    
    Ok(())
}