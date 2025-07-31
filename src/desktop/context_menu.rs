//! Context menu component for file operations

use dioxus::prelude::*;
use dioxus::events::MouseEvent;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum ContextMenuAction {
    NewFile,
    NewFolder,
    Rename,
    Delete,
    Cut,
    Copy,
    Paste,
    Duplicate,
    OpenInTerminal,
    CopyPath,
    RevealInFinder,
    ConfigureGitDecorations,
}

#[derive(Clone, Debug)]
pub struct ContextMenuItem {
    pub label: String,
    pub action: Option<ContextMenuAction>,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub separator: bool,
}

impl ContextMenuItem {
    pub fn new(label: &str, action: ContextMenuAction) -> Self {
        Self {
            label: label.to_string(),
            action: Some(action),
            shortcut: None,
            enabled: true,
            separator: false,
        }
    }

    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn separator() -> Self {
        Self {
            label: String::new(),
            action: None,
            shortcut: None,
            enabled: false,
            separator: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ContextMenuState {
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub target_path: Option<PathBuf>,
    pub is_directory: bool,
    pub clipboard: Option<ClipboardItem>,
}

#[derive(Clone, Debug)]
pub struct ClipboardItem {
    pub path: PathBuf,
    pub is_cut: bool, // true for cut, false for copy
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self {
            visible: false,
            x: 0,
            y: 0,
            target_path: None,
            is_directory: false,
            clipboard: None,
        }
    }
}

impl ContextMenuState {
    pub fn show(&mut self, x: i32, y: i32, path: PathBuf, is_directory: bool) {
        self.visible = true;
        self.x = x;
        self.y = y;
        self.target_path = Some(path);
        self.is_directory = is_directory;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.target_path = None;
    }

    pub fn set_clipboard(&mut self, path: PathBuf, is_cut: bool) {
        self.clipboard = Some(ClipboardItem { path, is_cut });
    }

    pub fn clear_clipboard(&mut self) {
        self.clipboard = None;
    }
}

#[component]
pub fn ContextMenu(
    state: Signal<ContextMenuState>,
    on_action: EventHandler<(ContextMenuAction, PathBuf)>,
) -> Element {
    let menu_state = state.read();
    
    if !menu_state.visible {
        return rsx! { div {} };
    }

    let target_path = menu_state.target_path.clone();
    let is_directory = menu_state.is_directory;
    let has_clipboard = menu_state.clipboard.is_some();

    // Build menu items based on context - VS Code style
    let items = if is_directory {
        vec![
            ContextMenuItem::new("New File...", ContextMenuAction::NewFile),
            ContextMenuItem::new("New Folder...", ContextMenuAction::NewFolder),
            ContextMenuItem::separator(),
            ContextMenuItem::new("Cut", ContextMenuAction::Cut)
                .with_shortcut("⌘X"),
            ContextMenuItem::new("Copy", ContextMenuAction::Copy)
                .with_shortcut("⌘C"),
            ContextMenuItem::new("Paste", ContextMenuAction::Paste)
                .with_shortcut("⌘V"),
            ContextMenuItem::separator(),
            ContextMenuItem::new("Rename", ContextMenuAction::Rename)
                .with_shortcut("F2"),
            ContextMenuItem::new("Delete", ContextMenuAction::Delete)
                .with_shortcut("Delete"),
            ContextMenuItem::separator(),
            ContextMenuItem::new("Copy Path", ContextMenuAction::CopyPath)
                .with_shortcut("⌥⌘C"),
            ContextMenuItem::new("Copy Relative Path", ContextMenuAction::CopyPath),
            ContextMenuItem::separator(),
            ContextMenuItem::new("Open in Integrated Terminal", ContextMenuAction::OpenInTerminal),
            ContextMenuItem::new("Reveal in Finder", ContextMenuAction::RevealInFinder),
        ]
    } else {
        vec![
            ContextMenuItem::new("Cut", ContextMenuAction::Cut)
                .with_shortcut("⌘X"),
            ContextMenuItem::new("Copy", ContextMenuAction::Copy)
                .with_shortcut("⌘C"),
            ContextMenuItem::new("Paste", ContextMenuAction::Paste)
                .with_shortcut("⌘V"),
            ContextMenuItem::separator(),
            ContextMenuItem::new("Rename", ContextMenuAction::Rename)
                .with_shortcut("F2"),
            ContextMenuItem::new("Delete", ContextMenuAction::Delete)
                .with_shortcut("Delete"),
            ContextMenuItem::separator(),
            ContextMenuItem::new("Copy Path", ContextMenuAction::CopyPath)
                .with_shortcut("⌥⌘C"),
            ContextMenuItem::new("Copy Relative Path", ContextMenuAction::CopyPath),
            ContextMenuItem::separator(),
            ContextMenuItem::new("Reveal in Finder", ContextMenuAction::RevealInFinder),
        ]
    };

    // Enable paste if we have something in clipboard
    let items: Vec<ContextMenuItem> = items.into_iter().map(|mut item| {
        if let Some(ContextMenuAction::Paste) = &item.action {
            item.enabled = has_clipboard;
        }
        item
    }).collect();

    rsx! {
        // Backdrop to capture clicks outside menu
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 999;",
            onclick: move |_| {
                state.write().hide();
            },
            
            // The actual context menu
            div {
                class: "context-menu",
                style: format!("left: {}px; top: {}px;", menu_state.x, menu_state.y),
                onclick: move |e| {
                    e.stop_propagation();
                },
                
                for item in items {
                    if item.separator {
                        div { class: "context-menu-separator" }
                    } else {
                        div {
                            class: if item.enabled { "context-menu-item" } else { "context-menu-item disabled" },
                            onclick: {
                                let action = item.action.clone();
                                let target_path = target_path.clone();
                                let enabled = item.enabled;
                                move |e: MouseEvent| {
                                    e.stop_propagation();
                                    if enabled {
                                        if let (Some(action), Some(path)) = (action.as_ref(), target_path.as_ref()) {
                                            on_action.call((action.clone(), path.clone()));
                                            state.write().hide();
                                        }
                                    }
                                }
                            },
                            
                            span { "{item.label}" }
                            if let Some(ref shortcut) = item.shortcut {
                                span { 
                                    style: "margin-left: auto; opacity: 0.6; font-size: 11px;",
                                    "{shortcut}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Input dialog for file/folder names
#[component]
pub fn FileNameDialog(
    visible: bool,
    title: String,
    initial_value: String,
    on_confirm: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut input_value = use_signal(|| initial_value.clone());
    
    use_effect(move || {
        if visible {
            input_value.set(initial_value.clone());
        }
    });

    if !visible {
        return rsx! { div {} };
    }

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            
            div {
                style: "background: #2d2d30; border: 1px solid #464647; border-radius: 6px; padding: 20px; min-width: 400px; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);",
                
                h3 {
                    style: "margin: 0 0 15px 0; color: #cccccc; font-size: 14px;",
                    "{title}"
                }
                
                input {
                    r#type: "text",
                    value: "{input_value}",
                    placeholder: if title.contains("File") { "Enter filename (e.g., script.js, README.md)" } else { "Enter folder name" },
                    oninput: move |e| input_value.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            let value = input_value.read().clone();
                            let trimmed = value.trim();
                            if !trimmed.is_empty() {
                                on_confirm.call(trimmed.to_string());
                            }
                        } else if e.key() == Key::Escape {
                            on_cancel.call(());
                        }
                    },
                    style: "width: 100%; padding: 8px; background: #1e1e1e; border: 1px solid #3e3e42; color: #cccccc; border-radius: 3px; font-family: inherit; font-size: 13px;",
                    autofocus: true,
                }
                
                div {
                    style: "margin-top: 15px; display: flex; justify-content: flex-end; gap: 10px;",
                    
                    button {
                        style: "padding: 6px 14px; background: #0e639c; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 13px;",
                        onclick: move |_| {
                            let value = input_value.read().clone();
                            let trimmed = value.trim();
                            if !trimmed.is_empty() {
                                on_confirm.call(trimmed.to_string());
                            }
                        },
                        "OK"
                    }
                    
                    button {
                        style: "padding: 6px 14px; background: #3e3e42; color: #cccccc; border: none; border-radius: 3px; cursor: pointer; font-size: 13px;",
                        onclick: move |_| {
                            on_cancel.call(());
                        },
                        "Cancel"
                    }
                }
            }
        }
    }
}

/// Confirmation dialog for dangerous operations
#[component]
pub fn ConfirmDialog(
    visible: bool,
    title: String,
    message: String,
    confirm_text: String,
    danger: bool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    if !visible {
        return rsx! { div {} };
    }

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            
            div {
                style: "background: #2d2d30; border: 1px solid #464647; border-radius: 6px; padding: 20px; max-width: 500px; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);",
                
                h3 {
                    style: "margin: 0 0 15px 0; color: #cccccc; font-size: 14px;",
                    "{title}"
                }
                
                p {
                    style: "margin: 0 0 20px 0; color: #cccccc; font-size: 13px; line-height: 1.5;",
                    "{message}"
                }
                
                div {
                    style: "display: flex; justify-content: flex-end; gap: 10px;",
                    
                    button {
                        style: format!("padding: 6px 14px; background: {}; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 13px;",
                            if danger { "#f44336" } else { "#0e639c" }
                        ),
                        onclick: move |_| {
                            on_confirm.call(());
                        },
                        "{confirm_text}"
                    }
                    
                    button {
                        style: "padding: 6px 14px; background: #3e3e42; color: #cccccc; border: none; border-radius: 3px; cursor: pointer; font-size: 13px;",
                        onclick: move |_| {
                            on_cancel.call(());
                        },
                        "Cancel"
                    }
                }
            }
        }
    }
}