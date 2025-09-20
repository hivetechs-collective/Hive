//! Enhanced File Explorer Component
//!
//! VS Code-style file tree with proper width and functionality

use crate::desktop::git::{
    use_git_decoration_manager, use_git_decoration_watcher, DecorationEvent, FileGitDecoration,
    GitDecorationManager, GitDecorationWatcher,
};
use crate::desktop::state::{AppState, FileItem, FileType, GitFileStatus};
use dioxus::prelude::*;
use std::path::{Path, PathBuf};

/// File Explorer with proper VS Code styling
#[component]
pub fn FileExplorer() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut search_query = use_signal(String::new);
    let mut show_hidden = use_signal(|| false);

    // Initialize git decoration system
    let decoration_manager = use_git_decoration_manager();
    let decoration_watcher = use_git_decoration_watcher(decoration_manager.clone());

    // Start watching for git changes if we have a project loaded
    use_effect(move || {
        if let Some(project) = &app_state.read().current_project {
            let repo_path = project.root_path.clone();
            let mut watcher = decoration_watcher.clone();
            spawn(async move {
                if let Err(e) = watcher.start_watching(repo_path).await {
                    tracing::error!("Failed to start git decoration watcher: {}", e);
                }
            });
        }
    });

    rsx! {
        div {
            class: "file-explorer-container",
            style: "width: 240px; background: var(--vscode-sideBar-background); height: 100%; display: flex; flex-direction: column;",

            // Explorer Header
            div {
                class: "explorer-header",
                style: "padding: 12px 20px; display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--vscode-sideBar-border);",

                h3 {
                    style: "font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; margin: 0; color: var(--vscode-sideBar-foreground);",
                    "EXPLORER"
                }

                div {
                    class: "explorer-actions",
                    style: "display: flex; gap: 4px;",

                    button {
                        class: "btn-icon",
                        title: "New File",
                        onclick: move |_| {
                            tracing::info!("New file requested");
                            // TODO: Implement new file dialog
                        },
                        i { class: "fa-solid fa-file-plus" }
                    }

                    button {
                        class: "btn-icon",
                        title: "New Folder",
                        onclick: move |_| {
                            tracing::info!("New folder requested");
                            // TODO: Implement new folder dialog
                        },
                        i { class: "fa-solid fa-folder-plus" }
                    }

                    button {
                        class: "btn-icon",
                        title: "Refresh Explorer",
                        onclick: move |_| {
                            app_state.write().file_explorer.refresh();
                        },
                        i { class: "fa-solid fa-arrow-rotate-right" }
                    }

                    button {
                        class: "btn-icon",
                        title: "Collapse All",
                        onclick: move |_| {
                            app_state.write().file_explorer.collapse_all();
                        },
                        i { class: "fa-solid fa-angles-up" }
                    }
                }
            }

            // Search Box
            div {
                class: "explorer-search",
                style: "padding: 8px 12px; border-bottom: 1px solid var(--vscode-sideBar-border);",

                div {
                    class: "search-input",
                    style: "display: flex; align-items: center; background: var(--vscode-input-background); border: 1px solid var(--vscode-input-border); border-radius: 4px; padding: 4px 8px;",

                    i {
                        class: "fa-solid fa-magnifying-glass",
                        style: "font-size: 12px; margin-right: 6px; opacity: 0.6;"
                    }

                    input {
                        style: "background: transparent; border: none; outline: none; flex: 1; font-size: 13px; color: var(--vscode-input-foreground);",
                        placeholder: "Search files...",
                        value: "{search_query}",
                        oninput: move |evt| {
                            search_query.set(evt.value());
                            app_state.write().file_explorer.filter_files(&evt.value());
                        }
                    }

                    if !search_query.read().is_empty() {
                        button {
                            class: "btn-icon",
                            style: "width: 16px; height: 16px; padding: 0;",
                            onclick: move |_| {
                                search_query.set(String::new());
                                app_state.write().file_explorer.clear_filter();
                            },
                            i { class: "fa-solid fa-xmark", style: "font-size: 10px;" }
                        }
                    }
                }
            }

            // Workspace Info
            div {
                class: "workspace-info",
                style: "padding: 8px 12px; border-bottom: 1px solid var(--vscode-sideBar-border);",

                div {
                    style: "display: flex; align-items: center; justify-content: space-between;",

                    span {
                        style: "font-size: 12px; font-weight: 600; color: var(--vscode-sideBar-foreground);",
                        "HIVE AI"
                    }

                    button {
                        class: "btn-icon",
                        style: "width: 20px; height: 20px;",
                        title: if *show_hidden.read() { "Hide hidden files" } else { "Show hidden files" },
                        onclick: move |_| {
                            show_hidden.set(!*show_hidden.read());
                            app_state.write().file_explorer.set_show_hidden(*show_hidden.read());
                        },
                        i {
                            class: if *show_hidden.read() { "fa-solid fa-eye" } else { "fa-solid fa-eye-slash" },
                            style: "font-size: 12px;"
                        }
                    }
                }
            }

            // File Tree
            div {
                class: "file-tree-container",
                style: "flex: 1; overflow-y: auto; overflow-x: hidden; padding: 4px 0;",

                if app_state.read().file_explorer.files.is_empty() {
                    div {
                        style: "padding: 20px; text-align: center; color: var(--vscode-tab-inactiveForeground); font-size: 13px;",
                        "No folder opened"
                        br {}
                        button {
                            class: "btn btn-primary",
                            style: "margin-top: 12px; font-size: 12px;",
                            onclick: move |_| {
                                // TODO: Implement folder picker
                                tracing::info!("Open folder requested");
                            },
                            "Open Folder"
                        }
                    }
                } else {
                    for file in &app_state.read().file_explorer.files {
                        FileTreeItem {
                            file: file.clone(),
                            depth: 0
                        }
                    }
                }
            }

            // Status Line
            div {
                class: "explorer-status",
                style: "padding: 6px 12px; border-top: 1px solid var(--vscode-sideBar-border); font-size: 11px; color: var(--vscode-tab-inactiveForeground);",

                if let Some(selected) = &app_state.read().file_explorer.selected_file {
                    "{selected.file_name().unwrap_or_default().to_string_lossy()}"
                } else {
                    "{app_state.read().file_explorer.files.len()} items"
                }
            }
        }
    }
}

/// Individual file/folder item in the tree
#[component]
fn FileTreeItem(file: FileItem, depth: usize) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut is_renaming = use_signal(|| false);
    let mut rename_value = use_signal(|| file.name.clone());
    let decoration_manager = use_context::<GitDecorationManager>();

    let is_selected = {
        let state = app_state.read();
        state.file_explorer.selected_file.as_ref() == Some(&file.path)
    };

    let indent = depth * 20;
    let file_path = file.path.clone();
    let is_directory = file.is_directory;

    // Git status styling
    let git_class = file
        .git_status
        .as_ref()
        .map(|status| match status {
            GitFileStatus::Modified => "git-modified",
            GitFileStatus::Added => "git-added",
            GitFileStatus::Deleted => "git-deleted",
            GitFileStatus::Untracked => "git-untracked",
            GitFileStatus::Renamed => "git-renamed",
            GitFileStatus::Copied => "git-copied",
            GitFileStatus::Ignored => "git-ignored",
        })
        .unwrap_or("");

    rsx! {
        div {
            class: "file-tree-item {git_class}",
            "data-selected": is_selected,
            style: "display: flex; flex-direction: column;",

            div {
                class: "file-item-row",
                style: "display: flex; align-items: center; padding: 2px 8px; cursor: pointer; user-select: none;",
                style: if is_selected { "background: var(--vscode-list-activeSelectionBackground); color: var(--vscode-list-activeSelectionForeground);" } else { "" },
                onmouseover: |_| {
                    // Add hover effect via JS if needed
                },
                onclick: move |evt| {
                    evt.stop_propagation();
                    if is_directory {
                        app_state.write().file_explorer.toggle_directory(&file_path);
                    } else {
                        app_state.write().file_explorer.select_file(file_path.clone());
                        // TODO: Open file in editor
                        app_state.write().open_file(&file_path);
                    }
                },
                oncontextmenu: move |evt| {
                    evt.prevent_default();
                    // TODO: Show context menu
                    tracing::debug!("Context menu for: {}", file_path.display());
                },

                // Indentation
                span { style: "width: {indent}px; flex-shrink: 0;" }

                // Expand/Collapse arrow for directories
                if is_directory {
                    span {
                        class: "tree-arrow",
                        style: "width: 16px; text-align: center; flex-shrink: 0;",
                        style: if file.children.is_empty() { "opacity: 0.3;" } else { "" },
                        onclick: move |evt| {
                            evt.stop_propagation();
                            app_state.write().file_explorer.toggle_directory(&file.path);
                        },
                        i {
                            class: if file.is_expanded { "fa-solid fa-chevron-down" } else { "fa-solid fa-chevron-right" },
                            style: "font-size: 10px;"
                        }
                    }
                } else {
                    span { style: "width: 16px; flex-shrink: 0;" }
                }

                // File/Folder icon
                span {
                    class: "file-icon",
                    style: "width: 16px; text-align: center; margin-right: 6px; flex-shrink: 0;",
                    i {
                        class: if file.is_directory {
                            if file.is_expanded { "fa-solid fa-folder-open" } else { "fa-solid fa-folder" }
                        } else {
                            get_file_icon(&file.file_type)
                        },
                        style: "font-size: 14px;",
                        style: if file.is_directory {
                            "color: #dcb67a;"
                        } else {
                            get_file_color(&file.file_type)
                        }
                    }
                }

                // File name
                if *is_renaming.read() {
                    input {
                        class: "rename-input",
                        style: "background: var(--vscode-input-background); border: 1px solid var(--vscode-inputOption-activeBorder); padding: 2px 4px; font-size: 13px; flex: 1;",
                        value: "{rename_value}",
                        oninput: move |evt| rename_value.set(evt.value()),
                        onkeydown: move |evt| {
                            match evt.key() {
                                Key::Enter => {
                                    // TODO: Perform rename
                                    is_renaming.set(false);
                                    tracing::info!("Rename {} to {}", file.name, rename_value.read());
                                },
                                Key::Escape => {
                                    is_renaming.set(false);
                                    rename_value.set(file.name.clone());
                                },
                                _ => {}
                            }
                        },
                        onblur: move |_| {
                            is_renaming.set(false);
                            rename_value.set(file.name.clone());
                        }
                    }
                } else {
                    span {
                        class: "file-name",
                        style: "flex: 1; font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                        ondblclick: move |evt| {
                            evt.stop_propagation();
                            if !is_directory {
                                // Open file on double click
                                app_state.write().open_file(&file.path);
                            }
                        },
                        "{file.name}"
                    }
                }

                // Git decoration using new system
                FileGitDecoration {
                    file_path: file.path.clone(),
                    is_directory: file.is_directory,
                    decoration_manager: decoration_manager.clone(),
                }
            }

            // Render children if expanded
            if file.is_directory && file.is_expanded && !file.children.is_empty() {
                div {
                    class: "file-children",
                    for child in &file.children {
                        FileTreeItem {
                            file: child.clone(),
                            depth: depth + 1
                        }
                    }
                }
            }
        }
    }
}

/// Get appropriate icon for file type
fn get_file_icon(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::Rust => "fa-brands fa-rust",
        FileType::JavaScript => "fa-brands fa-js",
        FileType::TypeScript => "fa-brands fa-js",
        FileType::Python => "fa-brands fa-python",
        FileType::Markdown => "fa-brands fa-markdown",
        FileType::Json => "fa-solid fa-brackets-curly",
        FileType::Toml => "fa-solid fa-gear",
        FileType::Yaml => "fa-solid fa-gear",
        FileType::Html => "fa-brands fa-html5",
        FileType::Css => "fa-brands fa-css3-alt",
        FileType::Image => "fa-solid fa-image",
        FileType::Video => "fa-solid fa-video",
        FileType::Audio => "fa-solid fa-music",
        FileType::Archive => "fa-solid fa-file-zipper",
        FileType::Text => "fa-solid fa-file-lines",
        FileType::Binary => "fa-solid fa-file",
        FileType::Other(ext) => match ext.as_str() {
            "go" => "fa-brands fa-golang",
            "java" => "fa-brands fa-java",
            "php" => "fa-brands fa-php",
            "rb" => "fa-solid fa-gem",
            "swift" => "fa-brands fa-swift",
            "vue" => "fa-brands fa-vuejs",
            "react" | "jsx" | "tsx" => "fa-brands fa-react",
            "sql" => "fa-solid fa-database",
            "sh" | "bash" | "zsh" => "fa-solid fa-terminal",
            "git" | "gitignore" => "fa-brands fa-git-alt",
            _ => "fa-solid fa-file",
        },
    }
}

/// Get color for file type icons
fn get_file_color(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::Rust => "color: #ce422b;",
        FileType::JavaScript => "color: #f7df1e;",
        FileType::TypeScript => "color: #3178c6;",
        FileType::Python => "color: #3776ab;",
        FileType::Markdown => "color: #083fa1;",
        FileType::Json => "color: #cbcb41;",
        FileType::Html => "color: #e34c26;",
        FileType::Css => "color: #1572b6;",
        FileType::Image => "color: #a074c4;",
        FileType::Git => "color: #f05032;",
        _ => "",
    }
}
