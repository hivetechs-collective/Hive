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

pub use app::DesktopApp;

use anyhow::Result;
use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use crate::core::config::Config as HiveConfig;

/// Launch the Dioxus desktop application
pub async fn launch_desktop_app(config: HiveConfig) -> Result<()> {
    tracing::info!("Launching HiveTechs Consensus Desktop Application");
    
    // Configure the desktop window
    let window_config = WindowBuilder::new()
        .with_title("HiveTechs Consensus")
        .with_min_inner_size(dioxus_desktop::LogicalSize::new(800, 600))
        .with_inner_size(dioxus_desktop::LogicalSize::new(1200, 800))
        .with_resizable(true);
    
    let desktop_config = Config::new()
        .with_window(window_config)
        .with_menu(Some(create_menu()));
    
    // Launch the Dioxus app
    LaunchBuilder::desktop()
        .with_cfg(desktop_config)
        .launch(app::App);
    
    Ok(())
}

/// Create the application menu
fn create_menu() -> dioxus_desktop::tao::menu::MenuBar {
    use dioxus_desktop::tao::menu::*;
    
    let mut menu = MenuBar::new();
    
    // File menu
    let mut file_menu = Menu::new();
    file_menu.add_item(MenuItemAttributes::new("New File").with_accelerators(&[Accelerator::new(Some(Modifiers::CONTROL), KeyCode::KeyN)]));
    file_menu.add_item(MenuItemAttributes::new("Open File").with_accelerators(&[Accelerator::new(Some(Modifiers::CONTROL), KeyCode::KeyO)]));
    file_menu.add_separator();
    file_menu.add_item(MenuItemAttributes::new("Exit").with_accelerators(&[Accelerator::new(Some(Modifiers::CONTROL), KeyCode::KeyQ)]));
    menu.add_submenu("File", true, file_menu);
    
    // Edit menu
    let mut edit_menu = Menu::new();
    edit_menu.add_item(MenuItemAttributes::new("Copy").with_accelerators(&[Accelerator::new(Some(Modifiers::CONTROL), KeyCode::KeyC)]));
    edit_menu.add_item(MenuItemAttributes::new("Paste").with_accelerators(&[Accelerator::new(Some(Modifiers::CONTROL), KeyCode::KeyV)]));
    edit_menu.add_item(MenuItemAttributes::new("Select All").with_accelerators(&[Accelerator::new(Some(Modifiers::CONTROL), KeyCode::KeyA)]));
    menu.add_submenu("Edit", true, edit_menu);
    
    // View menu
    let mut view_menu = Menu::new();
    view_menu.add_item(MenuItemAttributes::new("Toggle File Explorer").with_accelerators(&[Accelerator::new(Some(Modifiers::CONTROL), KeyCode::KeyB)]));
    view_menu.add_item(MenuItemAttributes::new("Command Palette").with_accelerators(&[Accelerator::new(Some(Modifiers::CONTROL), KeyCode::KeyP)]));
    menu.add_submenu("View", true, view_menu);
    
    // Help menu
    let mut help_menu = Menu::new();
    help_menu.add_item(MenuItemAttributes::new("About"));
    help_menu.add_item(MenuItemAttributes::new("Documentation"));
    menu.add_submenu("Help", true, help_menu);
    
    menu
}