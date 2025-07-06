//! Dioxus Desktop Application - VS Code-like Interface
//! 
//! 100% Rust implementation providing professional development experience

pub mod app;
pub mod components;
pub mod layout;
pub mod file_explorer;
pub mod file_system;
pub mod chat;
pub mod consensus;
pub mod state;
pub mod events;
pub mod styles;
pub mod menu_bar;
pub mod dialogs;

pub use app::App;

// Re-export commonly used styling components
pub use styles::components::{
    VsCodeButton, VsCodePanel, FileTreeItem, VsCodeTab, StatusBarItem,
    ButtonVariant, PanelStyle, IconSize
};
pub use styles::theme::{Theme, ThemeProvider, ThemeSwitcher, use_theme};

use anyhow::Result;
use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder, LogicalSize};
use crate::core::config::Config as HiveConfig;

/// Launch the Dioxus desktop application
pub fn launch_desktop_app(config: HiveConfig) -> Result<()> {
    tracing::info!("Launching HiveTechs Consensus Desktop Application");
    
    // Configure the desktop window
    let window_config = WindowBuilder::new()
        .with_title("HiveTechs Consensus")
        .with_min_inner_size(LogicalSize::new(800, 600))
        .with_inner_size(LogicalSize::new(1200, 800))
        .with_resizable(true);
    
    let desktop_config = Config::new()
        .with_window(window_config)
        .with_custom_head(include_str!("styles/global_head.html").to_string())
        .with_custom_index(create_custom_index());
    
    // Launch the Dioxus app
    LaunchBuilder::desktop()
        .with_cfg(desktop_config)
        .launch(app::App);
    
    Ok(())
}

/// Create custom HTML index with VS Code theming
fn create_custom_index() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>HiveTechs Consensus</title>
    <style>{}</style>
</head>
<body>
    <div id="app"></div>
</body>
</html>"#,
        &styles::get_global_styles()
    )
}

