//! Dioxus Desktop Application - VS Code-like Interface
//! 
//! 100% Rust implementation providing professional development experience

pub mod app;
pub mod components;
pub mod layout;
pub mod file_explorer;
pub mod chat;
pub mod consensus;
pub mod state;
pub mod events;

pub use app::App;

use anyhow::Result;
use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder, LogicalSize};
use crate::core::config::Config as HiveConfig;

/// Launch the Dioxus desktop application
pub async fn launch_desktop_app(config: HiveConfig) -> Result<()> {
    tracing::info!("Launching HiveTechs Consensus Desktop Application");
    
    // Configure the desktop window
    let window_config = WindowBuilder::new()
        .with_title("HiveTechs Consensus")
        .with_min_inner_size(LogicalSize::new(800, 600))
        .with_inner_size(LogicalSize::new(1200, 800))
        .with_resizable(true);
    
    let desktop_config = Config::new()
        .with_window(window_config);
    
    // Launch the Dioxus app
    LaunchBuilder::desktop()
        .with_cfg(desktop_config)
        .launch(app::App);
    
    Ok(())
}

