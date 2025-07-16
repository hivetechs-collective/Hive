//! Enhanced VS Code-style Explorer Panel
//! Based on VS Code's src/vs/workbench/contrib/files/browser/

use dioxus::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;

/// File decoration types (Git status, problems, etc.)
#[derive(Clone, Debug, PartialEq)]
pub enum FileDecoration {
    GitModified,
    GitAdded,
    GitDeleted,
    GitIgnored,
    GitUntracked,
    Error(u32),
    Warning(u32),
}

/// File tree node with decorations
#[derive(Clone, Debug, PartialEq)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub is_expanded: bool,
    pub children: Vec<FileNode>,
    pub decorations: Vec<FileDecoration>,
    pub is_selected: bool,
    pub is_cut: bool, // For cut/paste operations
    pub is_renamed: bool, // For inline rename
}

/// Explorer panel state
#[derive(Clone, Debug)]
pub struct ExplorerState {
    pub root_path: PathBuf,
    pub file_tree: Vec<FileNode>,
    pub selected_files: Vec<PathBuf>,
    pub expanded_folders: HashMap<PathBuf, bool>,
    pub search_query: String,
    pub show_hidden_files: bool,
    pub sort_by: FileSortOrder,
    pub folder_first: bool,
    pub compact_folders: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileSortOrder {
    Name,
    Modified,
    Type,
    Size,
}

/// Enhanced Explorer component with VS Code features
#[component]
pub fn EnhancedExplorer(
    state: Signal<ExplorerState>,
    on_file_select: EventHandler<PathBuf>,
    on_file_open: EventHandler<PathBuf>,
    on_context_menu: EventHandler<(PathBuf, i32, i32)>,
) -> Element {
    let explorer_state = state.read();
    
    rsx! {
        div {
            class: "explorer-viewlet",
            
            // Explorer header with title and actions
            div {
                class: "explorer-header",
                
                div {
                    class: "explorer-title",
                    
                    span { class: "title-label", "EXPLORER" }
                    
                    div {
                        class: "explorer-actions",
                        
                        button {
                            class: "action-button",
                            title: "New File... (Ctrl+Alt+N)",
                            onclick: move |_: dioxus::events::MouseEvent| {
                                // Trigger new file action
                            },
                            span { class: "codicon codicon-new-file" }
                        }
                        
                        button {
                            class: "action-button",
                            title: "New Folder...",
                            onclick: move |_: dioxus::events::MouseEvent| {
                                // Trigger new folder action
                            },
                            span { class: "codicon codicon-new-folder" }
                        }
                        
                        button {
                            class: "action-button",
                            title: "Refresh Explorer",
                            onclick: move |_: dioxus::events::MouseEvent| {
                                // Trigger refresh
                            },
                            span { class: "codicon codicon-refresh" }
                        }
                        
                        button {
                            class: "action-button",
                            title: "Collapse All",
                            onclick: move |_: dioxus::events::MouseEvent| {
                                // Collapse all folders
                            },
                            span { class: "codicon codicon-collapse-all" }
                        }
                    }
                }
                
                // Search box
                div {
                    class: "explorer-search",
                    
                    input {
                        class: "search-input",
                        placeholder: "Search files by name",
                        value: "{explorer_state.search_query}",
                        oninput: move |e: dioxus::events::FormEvent| {
                            state.write().search_query = e.value();
                        },
                    }
                    
                    if !explorer_state.search_query.is_empty() {
                        button {
                            class: "search-clear",
                            onclick: move |_: dioxus::events::MouseEvent| {
                                state.write().search_query.clear();
                            },
                            span { class: "codicon codicon-close" }
                        }
                    }
                }
            }
            
            // File tree container
            div {
                class: "explorer-folders-view",
                role: "tree",
                aria_label: "Files Explorer",
                
                if explorer_state.file_tree.is_empty() {
                    div {
                        class: "explorer-empty",
                        "No folder opened"
                    }
                } else {
                    for node in &explorer_state.file_tree {
                        FileTreeNode {
                            node: node.clone(),
                            level: 0,
                            state: state.clone(),
                            on_select: on_file_select.clone(),
                            on_open: on_file_open.clone(),
                            on_context_menu: on_context_menu.clone(),
                        }
                    }
                }
            }
            
            // Outline view (collapsible section)
            div {
                class: "outline-view",
                
                div {
                    class: "outline-header",
                    "OUTLINE"
                }
                
                div {
                    class: "outline-content",
                    "No symbols in active editor"
                }
            }
        }
    }
}

/// Individual file tree node
#[component]
fn FileTreeNode(
    node: FileNode,
    level: usize,
    state: Signal<ExplorerState>,
    on_select: EventHandler<PathBuf>,
    on_open: EventHandler<PathBuf>,
    on_context_menu: EventHandler<(PathBuf, i32, i32)>,
) -> Element {
    let indent = level * 20;
    let is_expanded = node.is_expanded;
    
    // Build node classes
    let mut node_classes = vec!["file-node"];
    if node.is_selected {
        node_classes.push("selected");
    }
    if node.is_cut {
        node_classes.push("cut");
    }
    
    // Get file icon based on type/extension
    let icon_class = if node.is_directory {
        if is_expanded {
            "codicon-folder-opened"
        } else {
            "codicon-folder"
        }
    } else {
        get_file_icon(&node.name)
    };
    
    // Build decoration indicators
    let decorations = node.decorations.iter().map(|d| {
        match d {
            FileDecoration::GitModified => ("M".to_string(), "git-decoration-modified"),
            FileDecoration::GitAdded => ("A".to_string(), "git-decoration-added"),
            FileDecoration::GitDeleted => ("D".to_string(), "git-decoration-deleted"),
            FileDecoration::GitUntracked => ("U".to_string(), "git-decoration-untracked"),
            FileDecoration::GitIgnored => ("".to_string(), "git-decoration-ignored"),
            FileDecoration::Error(count) => (format!("{}", count), "problems-decoration-error"),
            FileDecoration::Warning(count) => (format!("{}", count), "problems-decoration-warning"),
        }
    }).collect::<Vec<_>>();
    
    rsx! {
        div {
            class: node_classes.join(" "),
            style: "padding-left: {indent}px",
            role: "treeitem",
            aria_expanded: if node.is_directory { Some(is_expanded.to_string()) } else { None },
            aria_selected: "{node.is_selected}",
            
            div {
                class: "file-node-content",
                onclick: {
                    let path = node.path.clone();
                    move |e: dioxus::events::MouseEvent| {
                        e.stop_propagation();
                        on_select.call(path.clone());
                    }
                },
                ondoubleclick: {
                    let path = node.path.clone();
                    let is_dir = node.is_directory;
                    move |e: dioxus::events::MouseEvent| {
                        e.stop_propagation();
                        if !is_dir {
                            on_open.call(path.clone());
                        }
                    }
                },
                oncontextmenu: {
                    let path = node.path.clone();
                    move |e: dioxus::events::MouseEvent| {
                        e.prevent_default();
                        e.stop_propagation();
                        let coords = e.client_coordinates();
                        on_context_menu.call((path.clone(), coords.x as i32, coords.y as i32));
                    }
                },
                
                // Expand/collapse chevron for directories
                if node.is_directory {
                    span {
                        class: "tree-chevron codicon codicon-chevron-right",
                        class: if is_expanded { "expanded" } else { "" },
                        onclick: {
                            let path = node.path.clone();
                            move |e: dioxus::events::MouseEvent| {
                                e.stop_propagation();
                                // Toggle expansion
                                let mut explorer = state.write();
                                explorer.expanded_folders.insert(path.clone(), !is_expanded);
                            }
                        },
                    }
                }
                
                // File/folder icon
                span {
                    class: "file-icon codicon {icon_class}",
                }
                
                // File name (with inline rename support)
                if node.is_renamed {
                    input {
                        class: "rename-input",
                        value: "{node.name}",
                        autofocus: true,
                        onblur: move |_: dioxus::events::FocusEvent| {
                            // Complete rename
                        },
                        onkeydown: move |e: dioxus::events::KeyboardEvent| {
                            match e.key() {
                                dioxus::events::Key::Enter => {
                                    // Complete rename
                                },
                                dioxus::events::Key::Escape => {
                                    // Cancel rename
                                },
                                _ => {}
                            }
                        },
                    }
                } else {
                    span {
                        class: "file-label",
                        "{node.name}"
                    }
                }
                
                // Decorations (Git status, errors, etc.)
                if !decorations.is_empty() {
                    div {
                        class: "file-decorations",
                        
                        for (text, class) in decorations {
                            span {
                                class: "decoration {class}",
                                "{text}"
                            }
                        }
                    }
                }
            }
            
            // Render children if expanded
            if node.is_directory && is_expanded {
                div {
                    class: "file-node-children",
                    
                    for child in &node.children {
                        FileTreeNode {
                            node: child.clone(),
                            level: level + 1,
                            state: state.clone(),
                            on_select: on_select.clone(),
                            on_open: on_open.clone(),
                            on_context_menu: on_context_menu.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Get appropriate file icon based on file extension
fn get_file_icon(filename: &str) -> &'static str {
    let extension = filename.split('.').last().unwrap_or("");
    
    match extension.to_lowercase().as_str() {
        "rs" => "codicon-file-code",
        "js" | "jsx" => "codicon-file-js",
        "ts" | "tsx" => "codicon-file-ts",
        "py" => "codicon-file-python",
        "json" => "codicon-json",
        "md" => "codicon-markdown",
        "toml" | "yaml" | "yml" => "codicon-settings-gear",
        "html" | "htm" => "codicon-file-html",
        "css" | "scss" | "sass" => "codicon-file-css",
        "png" | "jpg" | "jpeg" | "gif" | "svg" => "codicon-file-media",
        "zip" | "tar" | "gz" => "codicon-file-zip",
        _ => "codicon-file",
    }
}

/// Explorer CSS styles
pub const EXPLORER_STYLES: &str = r#"
/* Explorer container */
.explorer-viewlet {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--vscode-sideBar-background);
    color: var(--vscode-sideBar-foreground);
}

/* Explorer header */
.explorer-header {
    flex-shrink: 0;
    padding: 8px 12px;
    border-bottom: 1px solid var(--vscode-sideBar-border);
}

.explorer-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
}

.title-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    opacity: 0.8;
}

.explorer-actions {
    display: flex;
    gap: 2px;
}

.action-button {
    width: 22px;
    height: 22px;
    border: none;
    background: transparent;
    color: var(--vscode-sideBar-foreground);
    cursor: pointer;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
    transition: opacity 0.1s ease;
}

.action-button:hover {
    opacity: 1;
    background-color: var(--vscode-toolbar-hoverBackground);
}

/* Search box */
.explorer-search {
    position: relative;
}

.search-input {
    width: 100%;
    padding: 4px 24px 4px 8px;
    background-color: var(--vscode-input-background);
    color: var(--vscode-input-foreground);
    border: 1px solid var(--vscode-input-border);
    border-radius: 3px;
    font-size: 13px;
    outline: none;
}

.search-input:focus {
    border-color: var(--hivetechs-yellow);
}

.search-clear {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
    width: 18px;
    height: 18px;
    border: none;
    background: transparent;
    color: var(--vscode-sideBar-foreground);
    cursor: pointer;
    opacity: 0.6;
}

.search-clear:hover {
    opacity: 1;
}

/* File tree */
.explorer-folders-view {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 4px 0;
}

.explorer-empty {
    padding: 12px;
    text-align: center;
    opacity: 0.6;
    font-style: italic;
}

/* File nodes */
.file-node {
    position: relative;
    user-select: none;
}

.file-node-content {
    display: flex;
    align-items: center;
    height: 22px;
    padding: 0 8px;
    cursor: pointer;
    position: relative;
}

.file-node-content:hover {
    background-color: var(--vscode-list-hoverBackground);
}

.file-node.selected .file-node-content {
    background-color: var(--vscode-list-activeSelectionBackground);
    color: var(--vscode-list-activeSelectionForeground);
}

.file-node.cut {
    opacity: 0.5;
}

/* Tree chevron */
.tree-chevron {
    width: 16px;
    flex-shrink: 0;
    transition: transform 0.1s ease;
}

.tree-chevron.expanded {
    transform: rotate(90deg);
}

/* File icons */
.file-icon {
    width: 16px;
    margin-right: 4px;
    flex-shrink: 0;
}

/* File label */
.file-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

/* Rename input */
.rename-input {
    flex: 1;
    background-color: var(--vscode-input-background);
    color: var(--vscode-input-foreground);
    border: 1px solid var(--hivetechs-yellow);
    padding: 0 4px;
    font-size: inherit;
    font-family: inherit;
    outline: none;
}

/* File decorations */
.file-decorations {
    display: flex;
    gap: 4px;
    margin-left: auto;
    padding-left: 8px;
}

.decoration {
    font-size: 11px;
    font-weight: 600;
    padding: 0 4px;
    border-radius: 2px;
}

.git-decoration-modified {
    color: var(--vscode-gitDecoration-modifiedResourceForeground);
}

.git-decoration-added {
    color: var(--vscode-gitDecoration-untrackedResourceForeground);
}

.git-decoration-deleted {
    color: var(--vscode-gitDecoration-deletedResourceForeground);
}

.git-decoration-ignored {
    opacity: 0.5;
}

.problems-decoration-error {
    background-color: var(--vscode-inputValidation-errorBackground);
    color: var(--vscode-inputValidation-errorForeground);
}

.problems-decoration-warning {
    background-color: var(--vscode-inputValidation-warningBackground);
    color: var(--vscode-inputValidation-warningForeground);
}

/* Outline view */
.outline-view {
    border-top: 1px solid var(--vscode-sideBar-border);
    padding: 8px 12px;
}

.outline-header {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    opacity: 0.8;
    margin-bottom: 8px;
}

.outline-content {
    font-size: 13px;
    opacity: 0.6;
    font-style: italic;
}

/* Multi-select styles */
.file-node.multi-selected .file-node-content {
    background-color: var(--vscode-list-inactiveSelectionBackground);
}

/* Drag and drop */
.file-node.drag-over .file-node-content {
    background-color: var(--vscode-list-dropBackground);
    outline: 1px dashed var(--hivetechs-yellow);
}

/* Compact folders */
.compact-folder-name {
    opacity: 0.7;
}

/* Focus styles */
.file-node-content:focus {
    outline: 1px solid var(--hivetechs-yellow);
    outline-offset: -1px;
}
"#;