//! File Explorer Component

use crate::desktop::{components::*, events::MouseEventUtils, state::*};
use dioxus::events::{KeyboardEvent, MouseData};
use dioxus::prelude::*;

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
                h3 { "EXPLORER" }
                button {
                    class: "collapse-btn",
                    onclick: move |_| {
                        // TODO: Implement panel collapse
                        tracing::debug!("Toggle explorer panel");
                    },
                    i { class: "fa-solid fa-chevron-left" }
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

    let mut class_name = String::from("file-item");
    if is_selected {
        class_name.push_str(" selected");
    }

    // Add git status to class for styling
    let git_status_class = file.git_status.as_ref().map(|status| match status {
        GitFileStatus::Modified => "modified",
        GitFileStatus::Added => "added",
        GitFileStatus::Deleted => "deleted",
        GitFileStatus::Untracked => "untracked",
        GitFileStatus::Renamed => "renamed",
        GitFileStatus::Copied => "copied",
        GitFileStatus::Ignored => "ignored",
    });

    // Clone paths and directory flag for closures to avoid move issues
    let file_path_click = file.path.clone();
    let file_path_context = file.path.clone();
    let file_path_drag = file.path.clone();
    let file_path_drop = file.path.clone();
    let is_directory = file.is_directory;

    // Calculate indentation based on depth
    let indent_width = file.depth * 20; // 20px per level
    let indent_style = format!("padding-left: {}px;", indent_width);

    rsx! {
        div {
            class: "{class_name}",
            style: "{indent_style}",
            draggable: true,
            "data-git-status": git_status_class.unwrap_or(""),
            onclick: move |evt: MouseEvent| {
                if MouseEventUtils::is_left_click(&evt) {
                    let mut state = app_state.write();
                    if is_directory {
                        // Toggle directory expansion
                        state.file_explorer.toggle_directory(&file_path_click);

                        // Also select this directory for repository context
                        state.file_explorer.select_directory(file_path_click.clone());

                        // Update current project to this directory
                        // This will be picked up by the repository context manager
                        let path = file_path_click.clone();

                        // Increment the trigger to notify consensus manager
                        state.repository_context_update_trigger += 1;
                        tracing::info!("Triggered repository context update for directory: {}", path.display());

                        drop(state); // Release the write lock before async operation

                        // Use Dioxus spawn instead of tokio::spawn for better integration
                        dioxus::prelude::spawn(async move {
                            if let Ok(project_info) = crate::desktop::state::ProjectInfo::from_path(path).await {
                                tracing::info!("Updating current project to: {}", project_info.name);
                                // Update the app state with the new project info
                                app_state.write().current_project = Some(project_info);
                            }
                        });
                    } else {
                        // Select file
                        state.file_explorer.select_file(file_path_click.clone());

                        // Also trigger repository context update for file selection
                        state.repository_context_update_trigger += 1;
                        tracing::info!("Triggered repository context update for file: {}", file_path_click.display());
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

            // Expand/collapse chevron for directories
            if file.is_directory {
                span {
                    class: "expand-icon",
                    style: "display: inline-block; width: 16px; margin-right: 4px; text-align: center; cursor: pointer;",
                    i {
                        class: if file.is_expanded { "fa-solid fa-chevron-down" } else { "fa-solid fa-chevron-right" },
                        style: "font-size: 10px; color: #858585;"
                    }
                }
            } else {
                // Empty space for alignment when not a directory
                span {
                    style: "display: inline-block; width: 20px;"
                }
            }

            span {
                class: "file-icon",
                style: "margin-right: 6px;",
                i {
                    class: if file.is_directory {
                        crate::desktop::styles::get_folder_icon(file.is_expanded)
                    } else {
                        crate::desktop::styles::get_file_icon(&file.file_type.extension())
                    },
                    "data-file-type": file.file_type.extension()
                }
            }

            span {
                class: "file-name",
                "{file.name}"
            }

            if let Some(git_status) = &file.git_status {
                span {
                    class: "git-status",
                    title: match git_status {
                        GitFileStatus::Modified => "Modified",
                        GitFileStatus::Added => "Added",
                        GitFileStatus::Deleted => "Deleted",
                        GitFileStatus::Untracked => "Untracked",
                        GitFileStatus::Renamed => "Renamed",
                        GitFileStatus::Copied => "Copied",
                        GitFileStatus::Ignored => "Ignored",
                    },
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
