    rsx! {
        // Inject VS Code-style CSS and dialog styles
        style { "{DESKTOP_STYLES}" }
        style { "{DIALOG_STYLES}" }
        style { "{ACTIVITY_BAR_STYLES}" }
        style { "{EXPLORER_STYLES}" }
        style { "{STATUS_BAR_STYLES}" }
        style { "{WELCOME_STYLES}" }
        style { "{CONTEXT_MENU_STYLES}" }
        
        // Main VS Code Layout
        VSCodeLayout {
            state: layout_state,
            
            // Activity Bar (Left sidebar navigation)
            activity_bar: rsx! {
                ActivityBar {
                    state: activity_bar_state,
                    on_item_click: move |id| {
                        // Update the active panel
                        activity_bar_state.write().active_item = Some(id.clone());
                        
                        // Update layout to show/hide sidebar based on selection
                        match id.as_str() {
                            "explorer" | "search" | "consensus" | "models" => {
                                layout_state.write().sidebar_visible = true;
                            }
                            _ => {}
                        }
                    },
                }
            },
            
            // Primary Sidebar (Changes based on Activity Bar selection)
            sidebar: rsx! {
                if layout_state.read().sidebar_visible {
                    match activity_bar_state.read().active_item.as_deref() {
                        Some("explorer") => rsx! {
                            EnhancedExplorer {
                                state: explorer_state,
                                on_file_select: move |path| {
                                    // Handle file selection
                                    *selected_file.write() = Some(path.display().to_string());
                                    
                                    // Load file content
                                    spawn({
                                        let path = path.clone();
                                        let mut file_content = file_content.clone();
                                        let mut open_tabs = open_tabs.clone();
                                        let mut active_tab = active_tab.clone();
                                        let mut tab_contents = tab_contents.clone();
                                        
                                        async move {
                                            match tokio::fs::read_to_string(&path).await {
                                                Ok(content) => {
                                                    let path_str = path.display().to_string();
                                                    
                                                    // Add to open tabs if not already open
                                                    if !open_tabs.read().contains(&path_str) {
                                                        open_tabs.write().push(path_str.clone());
                                                    }
                                                    
                                                    // Set as active tab
                                                    *active_tab.write() = path_str.clone();
                                                    
                                                    // Store content
                                                    tab_contents.write().insert(path_str, content.clone());
                                                    *file_content.write() = content;
                                                }
                                                Err(e) => {
                                                    eprintln!("Error reading file: {}", e);
                                                }
                                            }
                                        }
                                    });
                                },
                                on_file_open: move |path| {
                                    // Same as select for now
                                    // Handle file opening
                                },
                                on_context_menu: move |(path, x, y)| {
                                    // Show context menu
                                    context_menu_state.write().visible = true;
                                    context_menu_state.write().x = x as f64;
                                    context_menu_state.write().y = y as f64;
                                    context_menu_state.write().context = ContextMenuItem {
                                        label: "File Operations".to_string(),
                                        item_type: MenuItemType::Submenu,
                                        enabled: true,
                                        checked: false,
                                        children: vec![
                                            ContextMenuItem {
                                                label: "New File".to_string(),
                                                item_type: MenuItemType::Action,
                                                enabled: true,
                                                checked: false,
                                                children: vec![],
                                                keybinding: Some("Ctrl+N".to_string()),
                                            },
                                            ContextMenuItem {
                                                label: "New Folder".to_string(),
                                                item_type: MenuItemType::Action,
                                                enabled: true,
                                                checked: false,
                                                children: vec![],
                                                keybinding: None,
                                            },
                                        ],
                                        keybinding: None,
                                    };
                                },
                            }
                        },
                        Some("consensus") => rsx! {
                            // Consensus panel content
                            div {
                                class: "sidebar-panel",
                                ConsensusPanel {
                                    app_state: app_state.clone(),
                                    consensus_manager: consensus_manager.clone(),
                                    active_profile: active_profile.clone(),
                                    profiles: profiles.clone(),
                                    show_profile_dialog: show_profile_dialog.clone(),
                                    profile_dialog_mode: profile_dialog_mode.clone(),
                                    editing_profile: editing_profile.clone(),
                                }
                            }
                        },
                        Some("models") => rsx! {
                            // Model browser
                            div {
                                class: "sidebar-panel",
                                ModelBrowser {}
                            }
                        },
                        _ => rsx! { div {} }
                    }
                }
            },
            
            // Editor Area (Center)
            editor: rsx! {
                div {
                    class: "editor-container",
                    
                    // Editor tabs
                    div {
                        class: "editor-tabs",
                        // Welcome tab button
                        if show_welcome_dialog.read() || open_tabs.read().contains(&"__welcome__".to_string()) {
                            div {
                                class: if *active_tab.read() == "__welcome__" { "editor-tab active" } else { "editor-tab" },
                                onclick: move |_| {
                                    *active_tab.write() = "__welcome__".to_string();
                                    *current_view.write() = "welcome".to_string();
                                },
                                "Welcome"
                            }
                        }
                        
                        // Regular file tabs
                        {
                            open_tabs.read().iter().filter(|tab| *tab != "__welcome__").map(|tab| {
                                let tab_str = tab.clone();
                                let tab_for_click = tab.clone();
                                let tab_for_close = tab.clone();
                                let is_active = *active_tab.read() == *tab;
                                let display_name = if tab == "__analytics__" {
                                    "Analytics".to_string()
                                } else {
                                    let path = PathBuf::from(tab);
                                    path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or(tab)
                                        .to_string()
                                };
                                
                                rsx! {
                                    div {
                                        class: if is_active { "editor-tab active" } else { "editor-tab" },
                                        onclick: move |_| {
                                            *active_tab.write() = tab_for_click.clone();
                                            *selected_file.write() = Some(tab_for_click.clone());
                                            
                                            if tab_for_click == "__analytics__" {
                                                *current_view.write() = "analytics".to_string();
                                            } else {
                                                *current_view.write() = "code".to_string();
                                                if let Some(content) = tab_contents.read().get(&tab_for_click) {
                                                    *file_content.write() = content.clone();
                                                }
                                            }
                                        },
                                        "{display_name}"
                                        
                                        // Close button
                                        if tab_str != "__welcome__" {
                                            span {
                                                style: "margin-left: 8px; cursor: pointer; color: #858585; font-size: 16px;",
                                                onclick: move |e| {
                                                    e.stop_propagation();
                                                    
                                                    open_tabs.write().retain(|t| t != &tab_for_close);
                                                    
                                                    if *active_tab.read() == tab_for_close {
                                                        if let Some(first_tab) = open_tabs.read().first() {
                                                            *active_tab.write() = first_tab.clone();
                                                            *selected_file.write() = Some(first_tab.clone());
                                                        }
                                                    }
                                                    
                                                    tab_contents.write().remove(&tab_for_close);
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
                            })
                        }
                    }
                    
                    // Editor content
                    div {
                        class: "editor-content",
                        if *active_tab.read() == "__welcome__" {
                            // Show enhanced welcome tab
                            EnhancedWelcome {
                                state: welcome_state,
                                on_open_project: move |path| {
                                    // Handle project opening
                                    spawn({
                                        let path = PathBuf::from(path);
                                        let mut current_dir = current_dir.clone();
                                        let mut explorer_state = explorer_state.clone();
                                        
                                        async move {
                                            if path.exists() && path.is_dir() {
                                                *current_dir.write() = path.clone();
                                                explorer_state.write().root_path = path;
                                                // TODO: Reload file tree
                                            }
                                        }
                                    });
                                },
                                on_new_file: move |_| {
                                    *selected_file.write() = Some("untitled.txt".to_string());
                                    *file_content.write() = String::new();
                                    *show_welcome_dialog.write() = false;
                                    
                                    // Add to tabs
                                    if !open_tabs.read().contains(&"untitled.txt".to_string()) {
                                        open_tabs.write().push("untitled.txt".to_string());
                                    }
                                    *active_tab.write() = "untitled.txt".to_string();
                                },
                                on_new_project: move |_| {
                                    // Handle new project creation
                                },
                                on_open_folder: move |_| {
                                    // Handle folder opening
                                    spawn({
                                        let mut current_dir = current_dir.clone();
                                        async move {
                                            if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                                                *current_dir.write() = folder.path().to_path_buf();
                                            }
                                        }
                                    });
                                },
                            }
                        } else if *active_tab.read() == "__analytics__" {
                            AnalyticsView { analytics_data: analytics_data.clone() }
                        } else if !active_tab.read().is_empty() {
                            if let Some(content) = tab_contents.read().get(&*active_tab.read()) {
                                pre {
                                    style: "margin: 0; white-space: pre-wrap; word-wrap: break-word;",
                                    "{content}"
                                }
                            } else {
                                div {
                                    style: "padding: 20px; color: #858585;",
                                    "Loading file content..."
                                }
                            }
                        } else {
                            div {
                                class: "welcome-message",
                                "Select a file from the explorer to view its contents"
                            }
                        }
                    }
                }
            },
            
            // Secondary Sidebar (Consensus Chat)
            secondary_sidebar: rsx! {
                if layout_state.read().secondary_sidebar_visible {
                    div {
                        class: "chat-panel",
                        
                        // Panel header
                        div {
                            class: "panel-header",
                            style: "background: linear-gradient(135deg, #0E1414 0%, #181E21 100%); border-bottom: 2px solid #FFC107;",
                            span {
                                style: "display: inline-flex; align-items: center; gap: 8px;",
                                // HiveTechs logo
                                svg {
                                    width: "20",
                                    height: "20",
                                    view_box: "0 0 32 32",
                                    fill: "none",
                                    path {
                                        d: "M16 4L26 9V23L16 28L6 23V9L16 4Z",
                                        stroke: "#FFC107",
                                        stroke_width: "2",
                                        fill: "none"
                                    }
                                    circle {
                                        cx: "12",
                                        cy: "16",
                                        r: "4",
                                        fill: "#FFC107",
                                        opacity: "0.7"
                                    }
                                    circle {
                                        cx: "20",
                                        cy: "16",
                                        r: "4",
                                        fill: "#FFC107",
                                        opacity: "0.7"
                                    }
                                    rect {
                                        x: "14",
                                        y: "12",
                                        width: "4",
                                        height: "8",
                                        fill: "#FFC107",
                                        rx: "2"
                                    }
                                }
                                span {
                                    style: "background: linear-gradient(135deg, #FFC107 0%, #007BFF 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; font-weight: 700;",
                                    "HiveTechs Consensus"
                                }
                            }
                        }
                        
                        // Consensus content
                        ConsensusProgressDisplay {
                            consensus_state: app_state.read().consensus.clone()
                        }
                        
                        div {
                            class: "response-area",
                            id: "response-area",
                            ResponseDisplay {
                                consensus_state: app_state.read().consensus.clone()
                            }
                        }
                        
                        ChatInput {
                            consensus_manager: consensus_manager.clone(),
                            app_state: app_state.clone(),
                            api_config: api_config.clone(),
                        }
                    }
                }
            },
            
            // Panel (Bottom - can be terminal, problems, output, etc.)
            panel: rsx! {
                if layout_state.read().panel_visible {
                    div {
                        class: "bottom-panel",
                        style: "height: 200px; background: #1e1e1e; border-top: 1px solid #333;",
                        
                        div {
                            class: "panel-tabs",
                            style: "display: flex; background: #252526; border-bottom: 1px solid #333;",
                            
                            div {
                                class: "panel-tab active",
                                style: "padding: 8px 16px; cursor: pointer; color: #ccc; border-bottom: 2px solid #FFC107;",
                                "Problems"
                            }
                            div {
                                class: "panel-tab",
                                style: "padding: 8px 16px; cursor: pointer; color: #999;",
                                "Output"
                            }
                            div {
                                class: "panel-tab",
                                style: "padding: 8px 16px; cursor: pointer; color: #999;",
                                "Terminal"
                            }
                        }
                        
                        div {
                            class: "panel-content",
                            style: "padding: 16px; color: #ccc;",
                            "No problems detected"
                        }
                    }
                }
            },
            
            // Status Bar (Bottom)
            status_bar: rsx! {
                EnhancedStatusBar {
                    state: status_bar_state,
                    on_item_click: move |id| {
                        // Handle status bar item clicks
                        match id.as_str() {
                            "git-branch" => {
                                // Open branch switcher
                            }
                            "problems" => {
                                // Toggle problems panel
                                layout_state.write().panel_visible = !layout_state.read().panel_visible;
                            }
                            "consensus-status" => {
                                // Show consensus details
                            }
                            _ => {}
                        }
                    },
                }
            },
        }
        
        // VS Code Context Menu (overlays everything)
        if context_menu_state.read().visible {
            VSCodeContextMenu {
                state: context_menu_state,
                on_action: move |action| {
                    // Handle context menu actions
                    context_menu_state.write().visible = false;
                    
                    // Perform the action based on the menu item
                    match action.label.as_str() {
                        "New File" => {
                            // Create new file logic
                        }
                        "New Folder" => {
                            // Create new folder logic
                        }
                        _ => {}
                    }
                },
            }
        }
        
        // Keep existing dialogs
        CommandPalette {
            show_command_palette: show_command_palette.clone(),
            on_action: handle_command_action,
        }
        
        SettingsDialog {
            show_dialog: show_settings_dialog.clone(),
            openrouter_key: openrouter_key.clone(),
            hive_key: hive_key.clone(),
            api_keys_version: api_keys_version.clone(),
        }
        
        UpgradeDialog {
            show_dialog: show_upgrade_dialog.clone(),
        }
        
        AboutDialog {
            show_dialog: show_about_dialog.clone(),
        }
        
        if *show_new_file_dialog.read() {
            NewFileDialog {
                show_dialog: show_new_file_dialog.clone(),
                file_name: new_file_name.clone(),
                target_directory: new_file_target_dir.clone(),
                on_create: move |name: String| {
                    // File creation logic
                },
            }
        }
        
        if *show_new_folder_dialog.read() {
            NewFolderDialog {
                show_dialog: show_new_folder_dialog.clone(),
                folder_name: new_folder_name.clone(),
                target_directory: new_folder_target_dir.clone(),
                on_create: move |name: String| {
                    // Folder creation logic
                },
            }
        }
    }