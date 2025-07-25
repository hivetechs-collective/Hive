//! Dioxus Desktop Application - VS Code-like Interface
//!
//! 100% Rust implementation providing professional development experience

pub mod activity_bar;
pub mod ai_ui_events;
pub mod app;
pub mod assets;
pub mod chat;
pub mod claude_integration_manager;
pub mod components;
pub mod consensus;
pub mod consensus_integration;
pub mod context_menu;
pub mod context_menu_vscode;
pub mod dialogs;
pub mod events;
pub mod explorer_enhanced;
pub mod file_explorer;
pub mod file_operations;
pub mod file_system;
pub mod hybrid_chat_processor;
pub mod keyboard;
pub mod layout;
pub mod layout_enhanced;
pub mod logo_svg;
pub mod markdown;
pub mod menu_bar;
pub mod model_browser;
pub mod simple_db;
pub mod state;
pub mod status_bar_enhanced;
pub mod profile_service;
pub mod styles;
pub mod welcome_enhanced;
pub mod code_editor;

pub use app::App;

// Re-export commonly used styling components
pub use dialogs::OperationConfirmationDialog;
pub use menu_bar::{MenuAction, MenuBar};
pub use styles::components::{
    ButtonVariant, FileTreeItem, IconSize, PanelStyle, StatusBarItem, VsCodeButton, VsCodePanel,
    VsCodeTab,
};
pub use styles::theme::{use_theme, Theme, ThemeProvider, ThemeSwitcher};

use crate::core::config::Config as HiveConfig;
use anyhow::Result;
use dioxus::prelude::*;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder};

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
