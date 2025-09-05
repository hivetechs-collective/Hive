//! Enhanced File Explorer with Git Decorations
//!
//! Extended file explorer component that integrates the complete git decoration system

use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::desktop::{
    file_explorer::FileExplorer,
    git::{
        GitDecorationManager, GitDecorationWatcher, use_git_decoration_manager,
        use_git_decoration_watcher, GitDecorationConfigUI, GitDecorationPreview,
        DecorationEvent
    },
    state::AppState,
};

#[derive(Props, Clone, PartialEq)]
pub struct EnhancedFileExplorerProps {
    /// Optional callback when file is selected
    pub on_file_selected: Option<EventHandler<PathBuf>>,
    /// Optional callback when directory is selected  
    pub on_directory_selected: Option<EventHandler<PathBuf>>,
    /// Show git decoration config button
    pub show_config_button: Option<bool>,
}

/// Enhanced file explorer with integrated git decorations
#[component]
pub fn EnhancedFileExplorer(props: EnhancedFileExplorerProps) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut show_decoration_config = use_signal(|| false);
    let mut show_decoration_preview = use_signal(|| false);
    
    // Initialize git decoration system
    let decoration_manager = use_git_decoration_manager();
    let decoration_watcher = use_git_decoration_watcher(decoration_manager.clone());
    
    // Clone watcher for use in the effect
    let decoration_watcher_for_effect = decoration_watcher.clone();
    
    // Start watching for git changes when project changes
    use_effect(move || {
        if let Some(project) = &app_state.read().current_project {
            let repo_path = project.root_path.clone();
            let mut watcher = decoration_watcher_for_effect.clone();
            spawn(async move {
                if let Err(e) = watcher.start_watching(repo_path).await {
                    tracing::error!("Failed to start git decoration watcher: {}", e);
                }
            });
        }
    });

    rsx! {
        div {
            class: "enhanced-file-explorer",
            style: "width: 100%; height: 100%; display: flex; flex-direction: column;",

            // Explorer toolbar with git decoration controls
            div {
                class: "explorer-toolbar",
                style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-bottom: 1px solid var(--vscode-sideBar-border); background: var(--vscode-sideBar-background);",

                div {
                    class: "explorer-title",
                    style: "font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--vscode-sideBar-foreground);",
                    "EXPLORER"
                }

                if props.show_config_button.unwrap_or(true) {
                    div {
                        class: "git-decoration-controls",
                        style: "display: flex; gap: 4px;",

                        button {
                            class: "btn-icon",
                            title: "Configure Git Decorations",
                            style: "width: 24px; height: 24px; padding: 0; background: transparent; border: none; color: var(--vscode-foreground); cursor: pointer; border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                            style: if decoration_manager.is_enabled() { "" } else { "opacity: 0.5;" },
                            onclick: move |_| {
                                show_decoration_config.set(true);
                            },
                            i { 
                                class: "fa-solid fa-palette",
                                style: "font-size: 12px;"
                            }
                        }

                        button {
                            class: "btn-icon",
                            title: "Preview Git Decorations",
                            style: "width: 24px; height: 24px; padding: 0; background: transparent; border: none; color: var(--vscode-foreground); cursor: pointer; border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                            onclick: move |_| {
                                let current_value = *show_decoration_preview.read();
                                show_decoration_preview.set(!current_value);
                            },
                            i { 
                                class: "fa-solid fa-eye",
                                style: "font-size: 12px;"
                            }
                        }

                        button {
                            class: "btn-icon",
                            title: "Refresh Git Status",
                            style: "width: 24px; height: 24px; padding: 0; background: transparent; border: none; color: var(--vscode-foreground); cursor: pointer; border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                            onclick: {
                                let watcher = decoration_watcher.clone();
                                move |_| {
                                    let watcher = watcher.clone();
                                    spawn(async move {
                                        watcher.trigger_update(DecorationEvent::ManualRefresh).await;
                                    });
                                }
                            },
                            i { 
                                class: "fa-solid fa-arrows-rotate",
                                style: "font-size: 12px;"
                            }
                        }

                        // Git decoration status indicator
                        div {
                            class: "decoration-status",
                            style: "width: 8px; height: 8px; border-radius: 50%; margin-left: 4px;",
                            style: if decoration_manager.is_enabled() {
                                "background-color: #73c991;" // Green for enabled
                            } else {
                                "background-color: #6b6b6b;" // Gray for disabled
                            },
                            title: if decoration_manager.is_enabled() {
                                "Git decorations enabled"
                            } else {
                                "Git decorations disabled"
                            }
                        }
                    }
                }
            }

            // Preview panel (collapsible)
            if *show_decoration_preview.read() {
                div {
                    class: "decoration-preview-panel",
                    style: "border-bottom: 1px solid var(--vscode-sideBar-border); background: var(--vscode-sideBar-background);",

                    div {
                        class: "preview-header",
                        style: "display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; border-bottom: 1px solid var(--vscode-sideBar-border);",

                        span {
                            style: "font-size: 12px; font-weight: 600; color: var(--vscode-sideBar-foreground);",
                            "Decoration Preview"
                        }

                        button {
                            class: "btn-icon",
                            style: "width: 20px; height: 20px; padding: 0; background: transparent; border: none; color: var(--vscode-foreground); cursor: pointer;",
                            onclick: move |_| show_decoration_preview.set(false),
                            i { class: "fa-solid fa-xmark", style: "font-size: 10px;" }
                        }
                    }

                    div {
                        style: "padding: 8px 12px;",
                        GitDecorationPreview {
                            config: decoration_manager.config.read().clone()
                        }
                    }
                }
            }

            // Main file explorer
            div {
                class: "file-explorer-main",
                style: "flex: 1; overflow: hidden;",

                FileExplorer {}
            }

            // Git decoration statistics panel
            if decoration_manager.is_enabled() {
                div {
                    class: "git-stats-panel",
                    style: "border-top: 1px solid var(--vscode-sideBar-border); padding: 8px 12px; background: var(--vscode-sideBar-background);",

                    GitDecorationStats {
                        decoration_manager: decoration_manager.clone()
                    }
                }
            }
        }

        // Configuration dialog
        if *show_decoration_config.read() {
            GitDecorationConfigUI {
                decoration_manager: Arc::new(Mutex::new(decoration_manager.clone())),
                visible: true,
                on_close: move |_| show_decoration_config.set(false),
            }
        }
    }
}

/// Git decoration statistics display
#[component]
fn GitDecorationStats(decoration_manager: GitDecorationManager) -> Element {
    let file_statuses = decoration_manager.file_statuses.read();
    let folder_decorations = decoration_manager.folder_decorations.read();
    
    let total_files = file_statuses.len();
    let modified_count = file_statuses.values().filter(|s| matches!(s.status, crate::desktop::state::GitFileStatus::Modified)).count();
    let added_count = file_statuses.values().filter(|s| matches!(s.status, crate::desktop::state::GitFileStatus::Added)).count();
    let deleted_count = file_statuses.values().filter(|s| matches!(s.status, crate::desktop::state::GitFileStatus::Deleted)).count();
    let conflicts_count = file_statuses.values().filter(|s| s.is_conflicted).count();
    let total_folders = folder_decorations.len();

    if total_files == 0 {
        return rsx! {
            div {
                style: "font-size: 11px; color: var(--vscode-tab-inactiveForeground); text-align: center;",
                "No git changes detected"
            }
        };
    }

    rsx! {
        div {
            class: "git-stats",
            style: "font-size: 11px; color: var(--vscode-tab-inactiveForeground);",

            div {
                class: "stats-row",
                style: "display: flex; justify-content: space-between; margin-bottom: 4px;",

                span { "Files tracked: {total_files}" }
                span { "Folders: {total_folders}" }
            }

            if modified_count > 0 || added_count > 0 || deleted_count > 0 {
                div {
                    class: "change-stats",
                    style: "display: flex; gap: 12px; font-size: 10px;",

                    if modified_count > 0 {
                        span {
                            style: "color: #e2c08d;",
                            "M: {modified_count}"
                        }
                    }

                    if added_count > 0 {
                        span {
                            style: "color: #73c991;",
                            "A: {added_count}"
                        }
                    }

                    if deleted_count > 0 {
                        span {
                            style: "color: #f48771;",
                            "D: {deleted_count}"
                        }
                    }

                    if conflicts_count > 0 {
                        span {
                            style: "color: #f44747; font-weight: 600;",
                            "!: {conflicts_count}"
                        }
                    }
                }
            }
        }
    }
}

/// Integration component for existing file explorers
#[component]
pub fn GitDecorationIntegration() -> Element {
    let decoration_manager = use_git_decoration_manager();
    let decoration_watcher = use_git_decoration_watcher(decoration_manager.clone());
    let app_state = use_context::<Signal<AppState>>();

    // Auto-start git watching when project is loaded
    use_effect({
        let decoration_watcher = decoration_watcher.clone();
        move || {
            if let Some(project) = &app_state.read().current_project {
                let repo_path = project.root_path.clone();
                let mut watcher = decoration_watcher.clone();
                spawn(async move {
                    if let Err(e) = watcher.start_watching(repo_path).await {
                        tracing::error!("Failed to initialize git decorations: {}", e);
                    } else {
                        tracing::info!("Git decorations initialized successfully");
                    }
                });
            }
        }
    });

    rsx! {
        // This component just initializes the git decoration system
        // It doesn't render anything visible
        div { style: "display: none;" }
    }
}

/// Hook to provide git decoration functionality to components
pub fn use_git_decorations() -> (GitDecorationManager, GitDecorationWatcher) {
    let decoration_manager = use_git_decoration_manager();
    let decoration_watcher = use_git_decoration_watcher(decoration_manager.clone());
    (decoration_manager, decoration_watcher)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_file_explorer_props() {
        // Test that props can be created with default values
        let props = EnhancedFileExplorerProps {
            on_file_selected: None,
            on_directory_selected: None,
            show_config_button: Some(true),
        };
        
        assert!(props.show_config_button.unwrap_or(false));
    }
}