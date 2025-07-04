//! File Explorer Component

use dioxus::prelude::*;
use dioxus::events::{MouseData, KeyboardEvent};
use crate::desktop::{state::*, components::*, events::MouseEventUtils};

/// File Explorer Component - VS Code-like file tree
#[component]
pub fn FileExplorer() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let state = app_state.read();
    
    rsx! {
        div {
            class: "file-explorer",
            
            // Header
            div {
                class: "explorer-header",
                h3 { "📁 Explorer" }
                button {
                    class: "collapse-btn",
                    "◀"
                }
            }
            
            // File Tree
            div {
                class: "file-tree",
                tabindex: 0,
                onkeydown: move |evt: KeyboardEvent| {
                    match evt.key() {
                        Key::ArrowUp => {
                            // evt.prevent_default(); // Not available in Dioxus 0.5
                            // TODO: Navigate up in file tree
                        },
                        Key::ArrowDown => {
                            // evt.prevent_default(); // Not available in Dioxus 0.5
                            // TODO: Navigate down in file tree
                        },
                        Key::ArrowLeft => {
                            // evt.prevent_default(); // Not available in Dioxus 0.5
                            // TODO: Collapse directory
                        },
                        Key::ArrowRight => {
                            // evt.prevent_default(); // Not available in Dioxus 0.5
                            // TODO: Expand directory
                        },
                        Key::Enter => {
                            // evt.prevent_default(); // Not available in Dioxus 0.5
                            // TODO: Open selected file
                        },
                        _ => {}
                    }
                },
                if state.file_explorer.files.is_empty() {
                    div {
                        class: "empty-state",
                        "No files to display"
                    }
                } else {
                    for file in &state.file_explorer.files {
                        FileTreeItem { file: file.clone() }
                    }
                }
            }
        }
    }
}

/// Individual file/directory item in the tree
#[component]
fn FileTreeItem(file: FileItem) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let is_selected = {
        let state = app_state.read();
        state.file_explorer.selected_file.as_ref() == Some(&file.path)
    };
    
    let class_name = if is_selected {
        "file-item selected"
    } else {
        "file-item"
    };
    
    // Clone paths and directory flag for closures to avoid move issues
    let file_path_click = file.path.clone();
    let file_path_context = file.path.clone();
    let file_path_drag = file.path.clone();
    let file_path_drop = file.path.clone();
    let is_directory = file.is_directory;
    
    rsx! {
        div {
            class: class_name,
            draggable: true,
            onclick: move |evt: MouseEvent| {
                if MouseEventUtils::is_left_click(&evt) {
                    if is_directory {
                        // Toggle directory expansion
                        app_state.write().file_explorer.toggle_directory(&file_path_click);
                    } else {
                        // Select file
                        app_state.write().file_explorer.select_file(file_path_click.clone());
                    }
                }
            },
            oncontextmenu: move |evt: Event<MouseData>| {
                // evt.prevent_default(); // TODO: Update to new Dioxus API
                // TODO: Show context menu for file operations
                tracing::debug!("Right click on file: {}", file_path_context.display());
            },
            ondragstart: move |evt| {
                // Set drag data  
                // TODO: Fix drag and drop API for Dioxus 0.5
                tracing::debug!("Drag started for: {}", file_path_drag.display());
            },
            ondragover: move |evt| {
                if is_directory {
                    // evt.prevent_default(); // TODO: Update to new Dioxus API
                }
            },
            ondrop: move |evt| {
                if is_directory {
                    // evt.prevent_default(); // TODO: Update to new Dioxus API
                    // TODO: Handle file move/copy with proper drag data API
                    tracing::debug!("Drop on directory: {}", file_path_drop.display());
                }
            },
            
            span {
                class: "file-icon",
                {file.file_type.icon()}
            }
            
            span {
                class: "file-name",
                "{file.name}"
            }
            
            if let Some(git_status) = &file.git_status {
                span {
                    class: "git-status",
                    match git_status {
                        GitFileStatus::Modified => "M",
                        GitFileStatus::Added => "A",
                        GitFileStatus::Deleted => "D",
                        GitFileStatus::Untracked => "U",
                        GitFileStatus::Renamed => "R",
                        GitFileStatus::Copied => "C",
                        GitFileStatus::Ignored => "",
                    }
                }
            }
        }
        
        // Render children if directory is expanded
        if file.is_directory && file.is_expanded {
            div {
                class: "file-children",
                for child in &file.children {
                    FileTreeItem { file: child.clone() }
                }
            }
        }
    }
}