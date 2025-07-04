//! File Explorer Component

use dioxus::prelude::*;
use crate::desktop::{state::*, components::*};

/// File Explorer Component - VS Code-like file tree
#[component]
pub fn FileExplorer() -> Element {
    let app_state = use_context::<Signal<AppState>>();
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
    let app_state = use_context::<Signal<AppState>>();
    let is_selected = {
        let state = app_state.read();
        state.file_explorer.selected_file.as_ref() == Some(&file.path)
    };
    
    rsx! {
        div {
            class: "file-item {if is_selected { \"selected\" } else { \"\" }}",
            onclick: move |_| {
                if file.is_directory {
                    // Toggle directory expansion
                    app_state.write().file_explorer.toggle_directory(&file.path);
                } else {
                    // Select file
                    app_state.write().file_explorer.select_file(file.path.clone());
                }
            },
            
            span {
                class: "file-icon",
                "{file.file_type.icon()}"
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