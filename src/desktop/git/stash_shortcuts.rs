//! Keyboard shortcuts for git stash operations
//! 
//! VS Code-style keybindings:
//! - Ctrl+Shift+S: Save stash
//! - Ctrl+Shift+Q: Quick stash  
//! - Ctrl+Shift+P: Pop latest stash
//! - Ctrl+Shift+A: Apply latest stash

use dioxus::prelude::*;
use std::path::PathBuf;

use super::{GitStash, StashSaveOptions, StashApplyOptions, stash::async_ops};

/// Keyboard shortcut handler for stash operations
#[component]
pub fn StashKeyboardHandler(
    repo_path: Signal<Option<PathBuf>>,
    on_stash_action: EventHandler<StashShortcutAction>,
) -> Element {
    // Set up global keyboard event listener (simplified for desktop)
    use_effect(move || {
        // In a desktop app, keyboard shortcuts would be handled
        // at the application level or through the window manager
        // For now, we'll implement this through button actions
    });
    
    rsx! {
        // This component renders nothing but handles keyboard shortcuts
        div { style: "display: none;" }
    }
}

/// Handle stash keyboard shortcuts
pub fn handle_stash_shortcut(
    action: StashShortcutAction,
    repo_path: Option<PathBuf>,
    status_callback: Option<impl Fn(String) + 'static>,
) {
    if let Some(path) = repo_path {
        match action {
            StashShortcutAction::SaveStash => {
                // This would typically open the save stash dialog
                if let Some(callback) = status_callback {
                    callback("Opening stash save dialog...".to_string());
                }
            }
            
            StashShortcutAction::QuickStash => {
                spawn(async move {
                    if let Some(callback) = status_callback {
                        callback("Creating quick stash...".to_string());
                    }
                    
                    match tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
                        let stash = GitStash::new(&path)?;
                        stash.quick_stash()
                    }).await {
                        Ok(Ok(_)) => {
                            if let Some(callback) = status_callback {
                                callback("Quick stash created successfully".to_string());
                            }
                        }
                        Ok(Err(e)) => {
                            if let Some(callback) = status_callback {
                                callback(format!("Failed to create quick stash: {}", e));
                            }
                        }
                        Err(e) => {
                            if let Some(callback) = status_callback {
                                callback(format!("Task error: {}", e));
                            }
                        }
                    }
                });
            }
            
            StashShortcutAction::PopLatest => {
                spawn(async move {
                    if let Some(callback) = status_callback {
                        callback("Popping latest stash...".to_string());
                    }
                    
                    let opts = StashApplyOptions {
                        reinstate_index: false,
                        check_index: true,
                    };
                    
                    match async_ops::pop_stash(&path, 0, opts).await {
                        Ok(_) => {
                            if let Some(callback) = status_callback {
                                callback("Popped latest stash successfully".to_string());
                            }
                        }
                        Err(e) => {
                            if let Some(callback) = status_callback {
                                callback(format!("Failed to pop stash: {}", e));
                            }
                        }
                    }
                });
            }
            
            StashShortcutAction::ApplyLatest => {
                spawn(async move {
                    if let Some(callback) = status_callback {
                        callback("Applying latest stash...".to_string());
                    }
                    
                    let opts = StashApplyOptions {
                        reinstate_index: false,
                        check_index: true,
                    };
                    
                    match async_ops::apply_stash(&path, 0, opts).await {
                        Ok(_) => {
                            if let Some(callback) = status_callback {
                                callback("Applied latest stash successfully".to_string());
                            }
                        }
                        Err(e) => {
                            if let Some(callback) = status_callback {
                                callback(format!("Failed to apply stash: {}", e));
                            }
                        }
                    }
                });
            }
        }
    }
}

/// Stash keyboard shortcut actions
#[derive(Debug, Clone, PartialEq)]
pub enum StashShortcutAction {
    SaveStash,      // Ctrl+Shift+S
    QuickStash,     // Ctrl+Shift+Q
    PopLatest,      // Ctrl+Shift+P
    ApplyLatest,    // Ctrl+Shift+A
}

/// VS Code-style keyboard shortcuts info component
#[component]
pub fn StashShortcutsInfo() -> Element {
    rsx! {
        div {
            class: "shortcuts-info",
            style: SHORTCUTS_INFO_STYLES,
            
            h4 {
                style: "margin: 0 0 8px 0; color: #cccccc; font-size: 12px; font-weight: 600;",
                "Stash Shortcuts"
            }
            
            div {
                class: "shortcut-list",
                style: "display: flex; flex-direction: column; gap: 4px;",
                
                div {
                    class: "shortcut-item",
                    style: SHORTCUT_ITEM_STYLES,
                    
                    kbd {
                        style: KBD_STYLES,
                        "Ctrl+Shift+S"
                    }
                    
                    span {
                        style: "margin-left: 8px; color: #cccccc; font-size: 11px;",
                        "Save stash with dialog"
                    }
                }
                
                div {
                    class: "shortcut-item",
                    style: SHORTCUT_ITEM_STYLES,
                    
                    kbd {
                        style: KBD_STYLES,
                        "Ctrl+Shift+Q"
                    }
                    
                    span {
                        style: "margin-left: 8px; color: #cccccc; font-size: 11px;",
                        "Quick stash"
                    }
                }
                
                div {
                    class: "shortcut-item",
                    style: SHORTCUT_ITEM_STYLES,
                    
                    kbd {
                        style: KBD_STYLES,
                        "Ctrl+Shift+P"
                    }
                    
                    span {
                        style: "margin-left: 8px; color: #cccccc; font-size: 11px;",
                        "Pop latest stash"
                    }
                }
                
                div {
                    class: "shortcut-item",
                    style: SHORTCUT_ITEM_STYLES,
                    
                    kbd {
                        style: KBD_STYLES,
                        "Ctrl+Shift+A"
                    }
                    
                    span {
                        style: "margin-left: 8px; color: #cccccc; font-size: 11px;",
                        "Apply latest stash"
                    }
                }
            }
        }
    }
}

// Styles constants
const SHORTCUTS_INFO_STYLES: &str = "
    padding: 12px;
    background: #252526;
    border: 1px solid #333;
    border-radius: 4px;
    margin: 8px 0;
";

const SHORTCUT_ITEM_STYLES: &str = "
    display: flex;
    align-items: center;
    padding: 2px 0;
";

const KBD_STYLES: &str = "
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 3px;
    padding: 2px 6px;
    font-size: 10px;
    font-family: monospace;
    color: #f0ad4e;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
";