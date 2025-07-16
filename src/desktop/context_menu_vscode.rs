//! VS Code-style context menu implementation
//! Based on VS Code's context menu system architecture

use dioxus::prelude::*;
use dioxus::events::MouseEvent;
use std::path::PathBuf;

/// Menu item types matching VS Code's implementation
#[derive(Clone, Debug, PartialEq)]
pub enum MenuItem {
    Action {
        id: String,
        label: String,
        enabled: bool,
        keybinding: Option<String>,
        icon: Option<String>,
        when: Option<String>, // VS Code "when" clause
    },
    Separator,
    Submenu {
        id: String,
        label: String,
        items: Vec<MenuItem>,
        icon: Option<String>,
    },
    Checkbox {
        id: String,
        label: String,
        checked: bool,
        enabled: bool,
        keybinding: Option<String>,
    },
}

/// Menu groups following VS Code's organization
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum MenuGroup {
    Navigation,        // New, Open
    Modification,      // Edit operations
    Workspace,         // Workspace operations
    Compare,           // Compare operations
    Search,            // Search operations
    CutCopyPaste,      // Clipboard operations
    CopyPath,          // Path operations
    ModificationExtra, // Delete, rename
    Commands,          // Other commands
}

/// Context for determining which menu items to show
#[derive(Clone, Debug)]
pub struct FileExplorerContext {
    pub path: PathBuf,
    pub is_directory: bool,
    pub is_readonly: bool,
    pub is_root: bool,
    pub has_selection: bool,
    pub multiple_selection: bool,
    pub clipboard_has_files: bool,
    pub is_git_repository: bool,
}

/// Clipboard item for cut/copy operations
#[derive(Clone, Debug)]
pub struct ClipboardItem {
    pub path: PathBuf,
    pub is_cut: bool,
}

/// VS Code-style context menu state
#[derive(Clone, Debug)]
pub struct VSCodeContextMenuState {
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub context: Option<FileExplorerContext>,
    pub selected_index: Option<usize>,
    pub submenu_open: Option<String>,
    pub clipboard: Option<ClipboardItem>,
}

impl Default for VSCodeContextMenuState {
    fn default() -> Self {
        Self {
            visible: false,
            x: 0,
            y: 0,
            context: None,
            selected_index: None,
            submenu_open: None,
            clipboard: None,
        }
    }
}

impl VSCodeContextMenuState {
    pub fn show(&mut self, x: i32, y: i32, context: FileExplorerContext) {
        self.visible = true;
        self.x = x;
        self.y = y;
        self.context = Some(context);
        self.selected_index = Some(0);
        self.submenu_open = None;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.context = None;
        self.selected_index = None;
        self.submenu_open = None;
    }
    
    pub fn set_clipboard(&mut self, path: PathBuf, is_cut: bool) {
        self.clipboard = Some(ClipboardItem { path, is_cut });
    }
    
    pub fn clear_clipboard(&mut self) {
        self.clipboard = None;
    }
}

/// Build context menu items based on VS Code's structure
pub fn build_context_menu_items(context: &FileExplorerContext) -> Vec<MenuItem> {
    let mut items = Vec::new();

    // Navigation group (for directories)
    if context.is_directory {
        items.push(MenuItem::Action {
            id: "explorer.newFile".to_string(),
            label: "New File...".to_string(),
            enabled: !context.is_readonly,
            keybinding: None,
            icon: Some("new-file".to_string()),
            when: None,
        });
        
        items.push(MenuItem::Action {
            id: "explorer.newFolder".to_string(),
            label: "New Folder...".to_string(),
            enabled: !context.is_readonly,
            keybinding: None,
            icon: Some("new-folder".to_string()),
            when: None,
        });
        
        items.push(MenuItem::Separator);
    }

    // Always show open options for files
    if !context.is_directory {
        items.push(MenuItem::Action {
            id: "explorer.openToSide".to_string(),
            label: "Open to the Side".to_string(),
            enabled: true,
            keybinding: Some("Ctrl+Enter".to_string()),
            icon: Some("split-horizontal".to_string()),
            when: None,
        });
        
        items.push(MenuItem::Submenu {
            id: "explorer.openWith".to_string(),
            label: "Open With...".to_string(),
            items: vec![
                MenuItem::Action {
                    id: "explorer.openWithDefault".to_string(),
                    label: "Default Editor".to_string(),
                    enabled: true,
                    keybinding: None,
                    icon: None,
                    when: None,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    id: "explorer.openWithOther".to_string(),
                    label: "Other...".to_string(),
                    enabled: true,
                    keybinding: None,
                    icon: None,
                    when: None,
                },
            ],
            icon: None,
        });
        
        items.push(MenuItem::Separator);
    }

    // Compare group
    if !context.is_directory && !context.multiple_selection {
        items.push(MenuItem::Action {
            id: "explorer.compareSelected".to_string(),
            label: "Select for Compare".to_string(),
            enabled: true,
            keybinding: None,
            icon: Some("diff".to_string()),
            when: None,
        });
        
        items.push(MenuItem::Separator);
    }

    // CutCopyPaste group
    items.push(MenuItem::Action {
        id: "explorer.cut".to_string(),
        label: "Cut".to_string(),
        enabled: !context.is_readonly,
        keybinding: Some("Ctrl+X".to_string()),
        icon: Some("cut".to_string()),
        when: None,
    });
    
    items.push(MenuItem::Action {
        id: "explorer.copy".to_string(),
        label: "Copy".to_string(),
        enabled: true,
        keybinding: Some("Ctrl+C".to_string()),
        icon: Some("copy".to_string()),
        when: None,
    });
    
    items.push(MenuItem::Action {
        id: "explorer.paste".to_string(),
        label: "Paste".to_string(),
        enabled: context.clipboard_has_files && !context.is_readonly,
        keybinding: Some("Ctrl+V".to_string()),
        icon: Some("paste".to_string()),
        when: None,
    });
    
    items.push(MenuItem::Separator);

    // CopyPath group
    items.push(MenuItem::Action {
        id: "explorer.copyPath".to_string(),
        label: "Copy Path".to_string(),
        enabled: true,
        keybinding: Some("Alt+Shift+C".to_string()),
        icon: None,
        when: None,
    });
    
    items.push(MenuItem::Action {
        id: "explorer.copyRelativePath".to_string(),
        label: "Copy Relative Path".to_string(),
        enabled: true,
        keybinding: None,
        icon: None,
        when: None,
    });
    
    items.push(MenuItem::Separator);

    // Search group (for directories)
    if context.is_directory {
        items.push(MenuItem::Action {
            id: "explorer.findInFolder".to_string(),
            label: "Find in Folder...".to_string(),
            enabled: true,
            keybinding: Some("Shift+Alt+F".to_string()),
            icon: Some("search".to_string()),
            when: None,
        });
        
        items.push(MenuItem::Separator);
    }

    // ModificationExtra group
    items.push(MenuItem::Action {
        id: "explorer.rename".to_string(),
        label: "Rename...".to_string(),
        enabled: !context.is_readonly && !context.is_root,
        keybinding: Some("F2".to_string()),
        icon: Some("edit".to_string()),
        when: None,
    });
    
    items.push(MenuItem::Action {
        id: "explorer.delete".to_string(),
        label: "Delete".to_string(),
        enabled: !context.is_readonly && !context.is_root,
        keybinding: Some("Delete".to_string()),
        icon: Some("trash".to_string()),
        when: None,
    });
    
    items.push(MenuItem::Separator);

    // Terminal group
    if context.is_directory {
        items.push(MenuItem::Action {
            id: "explorer.openInIntegratedTerminal".to_string(),
            label: "Open in Integrated Terminal".to_string(),
            enabled: true,
            keybinding: None,
            icon: Some("terminal".to_string()),
            when: None,
        });
    }

    // System group
    #[cfg(target_os = "macos")]
    items.push(MenuItem::Action {
        id: "explorer.revealInFinder".to_string(),
        label: "Reveal in Finder".to_string(),
        enabled: true,
        keybinding: None,
        icon: Some("folder-opened".to_string()),
        when: None,
    });
    
    #[cfg(target_os = "windows")]
    items.push(MenuItem::Action {
        id: "explorer.revealInExplorer".to_string(),
        label: "Reveal in File Explorer".to_string(),
        enabled: true,
        keybinding: None,
        icon: Some("folder-opened".to_string()),
        when: None,
    });
    
    #[cfg(target_os = "linux")]
    items.push(MenuItem::Action {
        id: "explorer.openContainingFolder".to_string(),
        label: "Open Containing Folder".to_string(),
        enabled: true,
        keybinding: None,
        icon: Some("folder-opened".to_string()),
        when: None,
    });

    items
}

/// VS Code-style context menu component
#[component]
pub fn VSCodeContextMenu(
    state: Signal<VSCodeContextMenuState>,
    on_action: EventHandler<String>, // Just pass action ID
) -> Element {
    let menu_state = state.read();
    
    if !menu_state.visible || menu_state.context.is_none() {
        return rsx! { div {} };
    }

    let context = menu_state.context.as_ref().unwrap();
    let items = build_context_menu_items(context);

    rsx! {
        // Backdrop to capture clicks outside menu
        div {
            class: "vscode-menu-backdrop",
            onclick: move |_| {
                state.write().hide();
            },
            
            // The actual context menu
            div {
                class: "monaco-menu",
                style: format!("left: {}px; top: {}px;", menu_state.x, menu_state.y),
                onclick: move |e| {
                    e.stop_propagation();
                },
                
                div {
                    class: "monaco-action-bar vertical",
                    
                    ul {
                        class: "actions-container",
                        role: "menu",
                        
                        for (index, item) in items.iter().enumerate() {
                            {render_menu_item(item, index, state, on_action)}
                        }
                    }
                }
            }
        }
    }
}

/// Render individual menu items
fn render_menu_item(
    item: &MenuItem,
    index: usize,
    state: Signal<VSCodeContextMenuState>,
    on_action: EventHandler<String>,
) -> Element {
    match item {
        MenuItem::Separator => rsx! {
            li {
                class: "action-item",
                role: "presentation",
                div { class: "monaco-action-bar-separator" }
            }
        },
        
        MenuItem::Action { id, label, enabled, keybinding, icon, .. } => {
            let is_selected = state.read().selected_index == Some(index);
            let item_class = if *enabled {
                if is_selected {
                    "action-menu-item focused"
                } else {
                    "action-menu-item"
                }
            } else {
                "action-menu-item disabled"
            };
            
            let action_id = id.clone();
            let enabled_copy = *enabled;
            
            rsx! {
                li {
                    class: "action-item",
                    role: "presentation",
                    
                    a {
                        class: item_class,
                        role: "menuitem",
                        tabindex: "-1",
                        onclick: {
                            let mut state = state.clone();
                            let action_id = action_id.clone();
                            move |e: MouseEvent| {
                                e.stop_propagation();
                                if enabled_copy {
                                    on_action.call(action_id.clone());
                                    state.write().hide();
                                }
                            }
                        },
                        onmouseenter: {
                            let mut state = state.clone();
                            move |_| {
                                state.write().selected_index = Some(index);
                            }
                        },
                        
                        if let Some(icon_name) = icon {
                            span {
                                class: format!("codicon codicon-{}", icon_name),
                                style: "margin-right: 8px;",
                            }
                        }
                        
                        span { class: "action-label", "{label}" }
                        
                        if let Some(kb) = keybinding {
                            span { 
                                class: "keybinding",
                                style: "margin-left: auto; opacity: 0.7;",
                                "{kb}"
                            }
                        }
                    }
                }
            }
        },
        
        MenuItem::Submenu { id, label, items, icon } => {
            let is_selected = state.read().selected_index == Some(index);
            let is_open = state.read().submenu_open.as_ref() == Some(id);
            
            rsx! {
                li {
                    class: "action-item",
                    role: "presentation",
                    
                    a {
                        class: if is_selected { "action-menu-item focused" } else { "action-menu-item" },
                        role: "menuitem",
                        tabindex: "-1",
                        aria_haspopup: "true",
                        aria_expanded: "{is_open}",
                        onmouseenter: {
                            let mut state = state.clone();
                            let id = id.clone();
                            move |_| {
                                let mut menu_state = state.write();
                                menu_state.selected_index = Some(index);
                                menu_state.submenu_open = Some(id.clone());
                            }
                        },
                        
                        if let Some(icon_name) = icon {
                            span {
                                class: format!("codicon codicon-{}", icon_name),
                                style: "margin-right: 8px;",
                            }
                        }
                        
                        span { class: "action-label", "{label}" }
                        
                        span { 
                            class: "submenu-indicator codicon codicon-chevron-right",
                            style: "margin-left: auto;",
                        }
                    }
                    
                    // Render submenu if open
                    if is_open {
                        div {
                            class: "monaco-submenu",
                            style: "position: absolute; left: 100%; top: 0;",
                            
                            div {
                                class: "monaco-action-bar vertical",
                                
                                ul {
                                    class: "actions-container",
                                    role: "menu",
                                    
                                    for sub_item in items {
                                        {render_submenu_item(sub_item, state, on_action)}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        
        MenuItem::Checkbox { id, label, checked, enabled, keybinding } => {
            let is_selected = state.read().selected_index == Some(index);
            let item_class = if *enabled {
                if is_selected {
                    "action-menu-item focused"
                } else {
                    "action-menu-item"
                }
            } else {
                "action-menu-item disabled"
            };
            
            let action_id = id.clone();
            let enabled_copy = *enabled;
            
            rsx! {
                li {
                    class: "action-item",
                    role: "presentation",
                    
                    a {
                        class: item_class,
                        role: "menuitemcheckbox",
                        aria_checked: "{checked}",
                        tabindex: "-1",
                        onclick: {
                            let mut state = state.clone();
                            let action_id = action_id.clone();
                            move |e: MouseEvent| {
                                e.stop_propagation();
                                if enabled_copy {
                                    on_action.call(action_id.clone());
                                    state.write().hide();
                                }
                            }
                        },
                        onmouseenter: {
                            let mut state = state.clone();
                            move |_| {
                                state.write().selected_index = Some(index);
                            }
                        },
                        
                        span {
                            class: if *checked { "codicon codicon-check" } else { "" },
                            style: "width: 16px; margin-right: 8px; display: inline-block;",
                        }
                        
                        span { class: "action-label", "{label}" }
                        
                        if let Some(kb) = keybinding {
                            span { 
                                class: "keybinding",
                                style: "margin-left: auto; opacity: 0.7;",
                                "{kb}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Render submenu items
fn render_submenu_item(
    item: &MenuItem,
    state: Signal<VSCodeContextMenuState>,
    on_action: EventHandler<String>,
) -> Element {
    match item {
        MenuItem::Separator => rsx! {
            li {
                class: "action-item",
                role: "presentation",
                div { class: "monaco-action-bar-separator" }
            }
        },
        
        MenuItem::Action { id, label, enabled, keybinding, icon, .. } => {
            let item_class = if *enabled {
                "action-menu-item"
            } else {
                "action-menu-item disabled"
            };
            
            let action_id = id.clone();
            let enabled_copy = *enabled;
            
            rsx! {
                li {
                    class: "action-item",
                    role: "presentation",
                    
                    a {
                        class: item_class,
                        role: "menuitem",
                        tabindex: "-1",
                        onclick: {
                            let mut state = state.clone();
                            let action_id = action_id.clone();
                            move |e: MouseEvent| {
                                e.stop_propagation();
                                if enabled_copy {
                                    on_action.call(action_id.clone());
                                    state.write().hide();
                                }
                            }
                        },
                        
                        if let Some(icon_name) = icon {
                            span {
                                class: format!("codicon codicon-{}", icon_name),
                                style: "margin-right: 8px;",
                            }
                        }
                        
                        span { class: "action-label", "{label}" }
                        
                        if let Some(kb) = keybinding {
                            span { 
                                class: "keybinding",
                                style: "margin-left: auto; opacity: 0.7;",
                                "{kb}"
                            }
                        }
                    }
                }
            }
        },
        
        _ => rsx! { div {} } // Nested submenus not supported yet
    }
}