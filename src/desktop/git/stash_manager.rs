//! Git stash manager component that integrates with VS Code-style UI
//! 
//! Provides:
//! - Stash panel integration with toolbar
//! - Stash workflow management
//! - Keyboard shortcuts and context menus
//! - Integration with existing git components

use dioxus::prelude::*;
use std::path::PathBuf;

use super::{
    GitState, StashList, StashSaveDialog, StashPreview, StashAction, 
    StashInfo, StashSaveOptions, StashApplyOptions, GitStash,
    stash::async_ops
};

/// Main stash manager component
#[component]
pub fn StashManager(
    git_state: Signal<GitState>,
    is_visible: Signal<bool>,
) -> Element {
    let mut selected_stash = use_signal(|| None::<StashInfo>);
    let mut show_save_dialog = use_signal(|| false);
    let mut show_preview = use_signal(|| true);
    let mut operation_in_progress = use_signal(|| false);
    let mut status_message = use_signal(|| None::<String>);
    
    let mut repo_path = use_signal(|| None::<PathBuf>);
    
    // Update repo_path when git_state changes
    use_effect(move || {
        let path = git_state.read().active_repo.read().as_ref().map(|repo| repo.path.clone());
        repo_path.set(path);
    });
    
    // Clear selection when repo changes
    use_effect(move || {
        selected_stash.set(None);
    });
    
    if !is_visible.read().clone() {
        return rsx! { div {} };
    }
    
    rsx! {
        div {
            class: "stash-manager",
            style: STASH_MANAGER_STYLES,
            
            // Stash toolbar
            div {
                class: "stash-toolbar",
                style: STASH_TOOLBAR_STYLES,
                
                div {
                    class: "toolbar-left",
                    style: "display: flex; align-items: center; gap: 8px;",
                    
                    h3 {
                        style: "margin: 0; color: #cccccc; font-size: 13px; font-weight: 600;",
                        "Git Stashes"
                    }
                    
                    if let Some(message) = status_message.read().as_ref() {
                        div {
                            class: "status-message",
                            style: "font-size: 11px; color: #f0ad4e; margin-left: 8px;",
                            "{message}"
                        }
                    }
                }
                
                div {
                    class: "toolbar-right",
                    style: "display: flex; align-items: center; gap: 4px;",
                    
                    button {
                        class: "toolbar-btn",
                        style: TOOLBAR_BUTTON_STYLES,
                        title: "Save Stash (Ctrl+Shift+S)",
                        onclick: move |_| show_save_dialog.set(true),
                        disabled: operation_in_progress.read().clone() || repo_path.read().is_none(),
                        "💾"
                    }
                    
                    button {
                        class: "toolbar-btn",
                        style: TOOLBAR_BUTTON_STYLES,
                        title: "Toggle Preview Panel",
                        onclick: move |_| {
                            let current_value = *show_preview.read();
                            show_preview.set(!current_value);
                        },
                        if *show_preview.read() { "📋" } else { "📄" }
                    }
                    
                    div {
                        style: "width: 1px; height: 16px; background: #333; margin: 0 4px;"
                    }
                    
                    button {
                        class: "toolbar-btn",
                        style: TOOLBAR_BUTTON_STYLES,
                        title: "Quick Stash (Ctrl+Shift+Q)",
                        onclick: move |_| {
                            if let Some(path) = repo_path.read().as_ref() {
                                let path = path.clone();
                                spawn(async move {
                                    operation_in_progress.set(true);
                                    status_message.set(Some("Creating quick stash...".to_string()));
                                    
                                    match tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
                                        let stash = GitStash::new(&path)?;
                                        stash.quick_stash()
                                    }).await {
                                        Ok(Ok(_)) => {
                                            status_message.set(Some("Quick stash created".to_string()));
                                            // Auto-clear status after 3 seconds
                                            spawn(async move {
                                                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                                status_message.set(None);
                                            });
                                        }
                                        Ok(Err(e)) => {
                                            status_message.set(Some(format!("Failed to create stash: {}", e)));
                                        }
                                        Err(e) => {
                                            status_message.set(Some(format!("Task error: {}", e)));
                                        }
                                    }
                                    operation_in_progress.set(false);
                                });
                            }
                        },
                        disabled: operation_in_progress.read().clone() || repo_path.read().is_none(),
                        "⚡"
                    }
                }
            }
            
            // Main content area
            div {
                class: "stash-content",
                style: "display: flex; flex: 1; min-height: 0;",
                
                // Stash list panel
                div {
                    class: "stash-list-panel",
                    style: if *show_preview.read() { "flex: 1; min-width: 300px;" } else { "flex: 1;" },
                    
                    StashList {
                        repo_path: repo_path,
                        on_stash_select: move |stash_info| {
                            selected_stash.set(Some(stash_info));
                        },
                        on_stash_action: move |action| {
                            handle_stash_action(
                                action,
                                repo_path.read().clone(),
                                operation_in_progress,
                                status_message,
                                show_save_dialog
                            );
                        }
                    }
                }
                
                // Stash preview panel
                if *show_preview.read() {
                    div {
                        class: "stash-preview-panel",
                        style: "flex: 1; border-left: 1px solid #333; min-width: 300px;",
                        
                        StashPreview {
                            repo_path: repo_path,
                            selected_stash: selected_stash
                        }
                    }
                }
            }
            
            // Save stash dialog
            if *show_save_dialog.read() {
                StashSaveDialog {
                    repo_path: repo_path,
                    is_open: show_save_dialog,
                    on_save: move |opts| {
                        show_save_dialog.set(false);
                        
                        if let Some(path) = repo_path.read().as_ref() {
                            let path = path.clone();
                            spawn(async move {
                                operation_in_progress.set(true);
                                status_message.set(Some("Saving stash...".to_string()));
                                
                                match async_ops::save_stash(&path, opts).await {
                                    Ok(_) => {
                                        status_message.set(Some("Stash saved successfully".to_string()));
                                        // Auto-clear status after 3 seconds
                                        spawn(async move {
                                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                            status_message.set(None);
                                        });
                                    }
                                    Err(e) => {
                                        status_message.set(Some(format!("Failed to save stash: {}", e)));
                                    }
                                }
                                operation_in_progress.set(false);
                            });
                        }
                    },
                    on_cancel: move || show_save_dialog.set(false)
                }
            }
        }
    }
}

/// Handle stash actions from context menu and buttons
fn handle_stash_action(
    action: StashAction,
    repo_path: Option<PathBuf>,
    mut operation_in_progress: Signal<bool>,
    mut status_message: Signal<Option<String>>,
    mut show_save_dialog: Signal<bool>,
) {
    match action {
        StashAction::Save => {
            show_save_dialog.set(true);
        }
        
        StashAction::Apply(index) => {
            if let Some(path) = repo_path {
                spawn(async move {
                    operation_in_progress.set(true);
                    status_message.set(Some(format!("Applying stash #{}...", index)));
                    
                    let opts = StashApplyOptions {
                        reinstate_index: false,
                        check_index: true,
                    };
                    
                    match async_ops::apply_stash(&path, index, opts).await {
                        Ok(_) => {
                            status_message.set(Some(format!("Applied stash #{}", index)));
                            // Auto-clear status after 3 seconds
                            spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                status_message.set(None);
                            });
                        }
                        Err(e) => {
                            status_message.set(Some(format!("Failed to apply stash: {}", e)));
                        }
                    }
                    operation_in_progress.set(false);
                });
            }
        }
        
        StashAction::Pop(index) => {
            if let Some(path) = repo_path {
                spawn(async move {
                    operation_in_progress.set(true);
                    status_message.set(Some(format!("Popping stash #{}...", index)));
                    
                    let opts = StashApplyOptions {
                        reinstate_index: false,
                        check_index: true,
                    };
                    
                    match async_ops::pop_stash(&path, index, opts).await {
                        Ok(_) => {
                            status_message.set(Some(format!("Popped stash #{}", index)));
                            // Auto-clear status after 3 seconds
                            spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                status_message.set(None);
                            });
                        }
                        Err(e) => {
                            status_message.set(Some(format!("Failed to pop stash: {}", e)));
                        }
                    }
                    operation_in_progress.set(false);
                });
            }
        }
        
        StashAction::Drop(index) => {
            // Show confirmation dialog for destructive operation
            if let Some(path) = repo_path {
                if confirm_drop_stash(index) {
                    spawn(async move {
                        operation_in_progress.set(true);
                        status_message.set(Some(format!("Dropping stash #{}...", index)));
                        
                        match async_ops::drop_stash(&path, index).await {
                            Ok(_) => {
                                status_message.set(Some(format!("Dropped stash #{}", index)));
                                // Auto-clear status after 3 seconds
                                spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                    status_message.set(None);
                                });
                            }
                            Err(e) => {
                                status_message.set(Some(format!("Failed to drop stash: {}", e)));
                            }
                        }
                        operation_in_progress.set(false);
                    });
                }
            }
        }
        
        StashAction::Show(index) => {
            status_message.set(Some(format!("Showing stash #{}", index)));
            // The preview panel will automatically update when stash is selected
        }
    }
}

/// Show confirmation dialog for dropping stash
fn confirm_drop_stash(index: usize) -> bool {
    // In a desktop implementation, this would show a proper dialog
    // For now, we'll always return true and add UI confirmation later
    println!("Confirm drop stash #{}: This action cannot be undone.", index);
    true // Would be replaced with actual dialog in full implementation
}

/// Stash panel for integration with main git UI
#[component]
pub fn StashPanel(
    git_state: Signal<GitState>,
) -> Element {
    let mut is_expanded = use_signal(|| false);
    let mut stash_count = use_signal(|| 0usize);
    
    let mut repo_path = use_signal(|| None::<PathBuf>);
    
    // Update repo_path when git_state changes
    use_effect(move || {
        let path = git_state.read().active_repo.read().as_ref().map(|repo| repo.path.clone());
        repo_path.set(path);
    });
    
    // Update stash count when repo changes
    use_effect(move || {
        if let Some(path) = repo_path.read().as_ref() {
            let path = path.clone();
            spawn(async move {
                match tokio::task::spawn_blocking(move || -> anyhow::Result<usize> {
                    let stash = GitStash::new(&path)?;
                    Ok(stash.stash_count())
                }).await {
                    Ok(Ok(count)) => stash_count.set(count),
                    Ok(Err(_)) => stash_count.set(0),
                    Err(_) => stash_count.set(0),
                }
            });
        } else {
            stash_count.set(0);
        }
    });
    
    rsx! {
        div {
            class: "stash-panel",
            style: STASH_PANEL_STYLES,
            
            // Panel header
            div {
                class: "panel-header",
                style: PANEL_HEADER_STYLES,
                onclick: move |_| {
                    let current_value = *is_expanded.read();
                    is_expanded.set(!current_value);
                },
                
                div {
                    class: "header-icon",
                    style: format!("transition: transform 0.2s ease; {}", 
                        if *is_expanded.read() { "transform: rotate(90deg);" } else { "" }
                    ),
                    "▶"
                }
                
                div {
                    class: "header-title",
                    style: "margin-left: 8px; font-weight: 500;",
                    "Stashes"
                }
                
                if *stash_count.read() > 0 {
                    div {
                        class: "stash-badge",
                        style: STASH_BADGE_STYLES,
                        "{stash_count.read()}"
                    }
                }
            }
            
            // Panel content
            if *is_expanded.read() {
                div {
                    class: "panel-content",
                    style: "height: 300px; border-top: 1px solid #333; background: #1e1e1e;",
                    
                    StashManager {
                        git_state: git_state,
                        is_visible: Signal::new(true)
                    }
                }
            }
        }
    }
}

// Styles constants
const STASH_MANAGER_STYLES: &str = "
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    font-family: 'Segoe UI', system-ui, sans-serif;
";

const STASH_TOOLBAR_STYLES: &str = "
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: #252526;
    border-bottom: 1px solid #333;
    flex-shrink: 0;
";

const TOOLBAR_BUTTON_STYLES: &str = "
    background: none;
    border: 1px solid transparent;
    color: #cccccc;
    padding: 6px 8px;
    cursor: pointer;
    font-size: 12px;
    border-radius: 3px;
    transition: all 0.1s ease;
    
    &:hover:not(:disabled) {
        background: #333;
        border-color: #555;
    }
    
    &:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
";

const STASH_PANEL_STYLES: &str = "
    border: 1px solid #333;
    border-radius: 4px;
    background: #252526;
    margin-bottom: 8px;
    overflow: hidden;
";

const PANEL_HEADER_STYLES: &str = "
    display: flex;
    align-items: center;
    padding: 8px 12px;
    cursor: pointer;
    font-size: 12px;
    color: #cccccc;
    background: #252526;
    
    &:hover {
        background: #2a2d2e;
    }
";

const STASH_BADGE_STYLES: &str = "
    background: #f0ad4e;
    color: #1e1e1e;
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 10px;
    margin-left: auto;
    min-width: 16px;
    text-align: center;
";