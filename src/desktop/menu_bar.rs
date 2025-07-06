use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuAction {
    OpenFile,
    OpenFolder,
    OpenRecent,
    Save,
    SaveAs,
    CloseFolder,
    CommandPalette,
    ChangeTheme,
    Welcome,
    Documentation,
    About,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuItem {
    pub label: String,
    pub action: Option<MenuAction>,
    pub shortcut: Option<String>,
    pub separator: bool,
    pub submenu: Option<Vec<MenuItem>>,
}

impl MenuItem {
    pub fn new(label: &str, action: MenuAction, shortcut: Option<&str>) -> Self {
        MenuItem {
            label: label.to_string(),
            action: Some(action),
            shortcut: shortcut.map(|s| s.to_string()),
            separator: false,
            submenu: None,
        }
    }

    pub fn separator() -> Self {
        MenuItem {
            label: String::new(),
            action: None,
            shortcut: None,
            separator: true,
            submenu: None,
        }
    }

    pub fn submenu(label: &str, items: Vec<MenuItem>) -> Self {
        MenuItem {
            label: label.to_string(),
            action: None,
            shortcut: None,
            separator: false,
            submenu: Some(items),
        }
    }
}

pub const MENU_STYLES: &str = r#"
    /* Menu bar styles */
    .menu-bar {
        height: 30px;
        background: #2d2d30;
        display: flex;
        align-items: center;
        padding: 0 10px;
        border-bottom: 1px solid #3e3e42;
        user-select: none;
        position: relative;
        z-index: 1000;
    }
    
    .menu-item {
        padding: 0 12px;
        height: 100%;
        display: flex;
        align-items: center;
        font-size: 13px;
        color: #cccccc;
        cursor: pointer;
        position: relative;
        transition: background-color 0.1s;
    }
    
    .menu-item:hover,
    .menu-item.active {
        background: #3e3e42;
    }
    
    /* Dropdown menu styles */
    .menu-dropdown {
        position: absolute;
        top: 100%;
        left: 0;
        background: #252526;
        border: 1px solid #3e3e42;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
        min-width: 200px;
        padding: 4px 0;
        z-index: 1001;
    }
    
    .menu-dropdown-item {
        padding: 6px 20px;
        font-size: 13px;
        color: #cccccc;
        cursor: pointer;
        display: flex;
        justify-content: space-between;
        align-items: center;
        transition: background-color 0.1s;
    }
    
    .menu-dropdown-item:hover {
        background: #094771;
        color: #ffffff;
    }
    
    .menu-dropdown-item.disabled {
        color: #5a5a5a;
        cursor: default;
    }
    
    .menu-dropdown-item.disabled:hover {
        background: transparent;
        color: #5a5a5a;
    }
    
    .menu-separator {
        height: 1px;
        background: #3e3e42;
        margin: 4px 0;
    }
    
    .menu-shortcut {
        font-size: 11px;
        color: #858585;
        margin-left: 20px;
    }
    
    .menu-dropdown-item:hover .menu-shortcut {
        color: #cccccc;
    }
    
    /* Submenu styles */
    .menu-submenu-arrow {
        margin-left: auto;
        font-size: 10px;
        color: #858585;
    }
    
    .menu-submenu {
        position: absolute;
        left: 100%;
        top: 0;
        background: #252526;
        border: 1px solid #3e3e42;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
        min-width: 150px;
        padding: 4px 0;
        display: none;
    }
    
    .menu-dropdown-item:hover .menu-submenu {
        display: block;
    }
    
    /* Click outside overlay */
    .menu-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        z-index: 999;
        background: transparent;
    }
"#;

#[component]
pub fn MenuBar(on_action: EventHandler<MenuAction>) -> Element {
    let mut active_menu = use_signal(|| None::<String>);
    let mut hover_enabled = use_signal(|| false);

    // Define menu structure
    let menus = vec![
        ("File", vec![
            MenuItem::new("Open File", MenuAction::OpenFile, Some("Cmd+O")),
            MenuItem::new("Open Folder", MenuAction::OpenFolder, Some("Cmd+Shift+O")),
            MenuItem::new("Open Recent", MenuAction::OpenRecent, None),
            MenuItem::separator(),
            MenuItem::new("Save", MenuAction::Save, Some("Cmd+S")),
            MenuItem::new("Save As...", MenuAction::SaveAs, Some("Cmd+Shift+S")),
            MenuItem::separator(),
            MenuItem::new("Close Folder", MenuAction::CloseFolder, None),
        ]),
        ("View", vec![
            MenuItem::new("Command Palette", MenuAction::CommandPalette, Some("Cmd+Shift+P")),
            MenuItem::separator(),
            MenuItem::submenu("Appearance", vec![
                MenuItem::new("Theme", MenuAction::ChangeTheme, None),
            ]),
        ]),
        ("Help", vec![
            MenuItem::new("Welcome", MenuAction::Welcome, None),
            MenuItem::new("Documentation", MenuAction::Documentation, None),
            MenuItem::separator(),
            MenuItem::new("About", MenuAction::About, None),
        ]),
    ];


    rsx! {
        style { "{MENU_STYLES}" }
        
        // Click outside overlay
        if active_menu.read().is_some() {
            div {
                class: "menu-overlay",
                onclick: move |_| {
                    *active_menu.write() = None;
                    *hover_enabled.write() = false;
                },
            }
        }
        
        div {
            class: "menu-bar",
            
            for (menu_name, items) in menus {
                div {
                    class: if active_menu.read().as_ref() == Some(&menu_name.to_string()) { 
                        "menu-item active" 
                    } else { 
                        "menu-item" 
                    },
                    onclick: {
                        let menu_name = menu_name.to_string();
                        let mut active_menu = active_menu.clone();
                        let mut hover_enabled = hover_enabled.clone();
                        move |_| {
                            let current = active_menu.read().clone();
                            if current == Some(menu_name.clone()) {
                                *active_menu.write() = None;
                                *hover_enabled.write() = false;
                            } else {
                                *active_menu.write() = Some(menu_name.clone());
                                *hover_enabled.write() = true;
                            }
                        }
                    },
                    onmouseenter: {
                        let menu_name = menu_name.to_string();
                        let mut active_menu = active_menu.clone();
                        let hover_enabled = hover_enabled.clone();
                        move |_| {
                            if *hover_enabled.read() {
                                *active_menu.write() = Some(menu_name.clone());
                            }
                        }
                    },
                    "{menu_name}"
                    
                    // Dropdown menu
                    if active_menu.read().as_ref() == Some(&menu_name.to_string()) {
                        div {
                            class: "menu-dropdown",
                            onclick: move |e| e.stop_propagation(),
                            
                            for item in items {
                                if item.separator {
                                    div { class: "menu-separator" }
                                } else if let Some(submenu_items) = &item.submenu {
                                    div {
                                        class: "menu-dropdown-item",
                                        "{item.label}"
                                        span { class: "menu-submenu-arrow", "▶" }
                                        
                                        div {
                                            class: "menu-submenu",
                                            for sub_item in submenu_items {
                                                div {
                                                    class: "menu-dropdown-item",
                                                    onclick: {
                                                        let action = sub_item.action.clone();
                                                        let mut active_menu = active_menu.clone();
                                                        let mut hover_enabled = hover_enabled.clone();
                                                        move |_| {
                                                            if let Some(action) = &action {
                                                                on_action.call(*action);
                                                                *active_menu.write() = None;
                                                                *hover_enabled.write() = false;
                                                            }
                                                        }
                                                    },
                                                    "{sub_item.label}"
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    div {
                                        class: if item.action.is_some() { 
                                            "menu-dropdown-item" 
                                        } else { 
                                            "menu-dropdown-item disabled" 
                                        },
                                        onclick: {
                                            let action = item.action.clone();
                                            let mut active_menu = active_menu.clone();
                                            let mut hover_enabled = hover_enabled.clone();
                                            move |_| {
                                                if let Some(action) = &action {
                                                    on_action.call(*action);
                                                    *active_menu.write() = None;
                                                    *hover_enabled.write() = false;
                                                }
                                            }
                                        },
                                        "{item.label}"
                                        if let Some(shortcut) = &item.shortcut {
                                            span { class: "menu-shortcut", "{shortcut}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}