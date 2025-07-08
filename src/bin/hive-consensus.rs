#![allow(non_snake_case)]

use dioxus::prelude::*;
use rfd;

const DESKTOP_STYLES: &str = r#"
    /* VS Code-style CSS */
    body {
        margin: 0;
        padding: 0;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
        background: #1e1e1e;
        color: #cccccc;
        height: 100vh;
        overflow: hidden;
        font-size: 14px;
    }
    
    .app-container {
        display: flex;
        height: 100vh;
        flex-direction: column;
    }
    
    /* Main content area */
    .main-content {
        display: flex;
        flex: 1;
        overflow: hidden;
    }
    
    /* Sidebar styles (left) */
    .sidebar {
        width: 250px;
        background: #252526;
        display: flex;
        flex-direction: column;
        border-right: 1px solid #3e3e42;
        overflow-y: auto;
    }
    
    .sidebar-header {
        padding: 10px 20px;
        background: #2d2d30;
        border-bottom: 1px solid #3e3e42;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }
    
    .current-path {
        font-size: 11px;
        color: #858585;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    
    .open-folder-button {
        padding: 6px 12px;
        background: #007acc;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 12px;
        font-weight: 600;
        transition: background-color 0.1s;
    }
    
    .open-folder-button:hover {
        background: #005a9e;
    }
    
    .sidebar-section-title {
        font-size: 11px;
        font-weight: 600;
        text-transform: uppercase;
        color: #858585;
        padding: 10px 20px 5px 20px;
        letter-spacing: 0.5px;
    }
    
    .sidebar-item {
        padding: 6px 20px;
        font-size: 13px;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 8px;
        color: #cccccc;
        transition: background-color 0.1s;
    }
    
    .sidebar-item:hover {
        background: #2a2d2e;
    }
    
    .sidebar-item.active {
        background: #094771;
        color: #ffffff;
    }
    
    /* Code editor area (center) */
    .editor-container {
        flex: 1;
        display: flex;
        flex-direction: column;
        background: #1e1e1e;
        min-width: 0;
    }
    
    .editor-tabs {
        height: 35px;
        background: #2d2d30;
        display: flex;
        align-items: center;
        border-bottom: 1px solid #3e3e42;
        padding: 0 10px;
    }
    
    .editor-tab {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 0 15px;
        height: 100%;
        background: #2d2d30;
        border-right: 1px solid #3e3e42;
        font-size: 13px;
        cursor: pointer;
        transition: background-color 0.1s;
    }
    
    .editor-tab.active {
        background: #1e1e1e;
        border-bottom: 1px solid #1e1e1e;
    }
    
    .editor-tab:hover {
        background: #323234;
    }
    
    .editor-content {
        flex: 1;
        padding: 20px;
        overflow-y: auto;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: 13px;
        line-height: 1.6;
    }
    
    .welcome-message {
        color: #858585;
        text-align: center;
        margin-top: 100px;
    }
    
    /* Chat panel (right) */
    .chat-panel {
        width: 350px;
        background: #252526;
        border-left: 1px solid #3e3e42;
        display: flex;
        flex-direction: column;
    }
    
    .panel-header {
        background: #2d2d30;
        padding: 10px 20px;
        border-bottom: 1px solid #3e3e42;
        font-weight: 600;
        font-size: 14px;
    }
    
    .messages-area {
        flex: 1;
        overflow-y: auto;
        padding: 20px;
        display: flex;
        flex-direction: column;
        gap: 15px;
    }
    
    .message {
        padding: 12px 16px;
        border-radius: 6px;
        max-width: 85%;
        font-size: 14px;
        line-height: 1.5;
    }
    
    .user-message {
        background: #007acc;
        color: white;
        align-self: flex-end;
    }
    
    .ai-message {
        background: #2d2d30;
        border: 1px solid #3e3e42;
        align-self: flex-start;
    }
    
    /* Input area styles */
    .input-area {
        display: flex;
        padding: 16px 20px;
        gap: 12px;
        background: #1e1e1e;
        border-top: 1px solid #3e3e42;
    }
    
    .message-input {
        flex: 1;
        background: #3c3c3c;
        border: 1px solid #3e3e42;
        color: #cccccc;
        padding: 10px 14px;
        border-radius: 4px;
        font-size: 14px;
        font-family: inherit;
        outline: none;
        transition: border-color 0.1s;
    }
    
    .message-input:focus {
        border-color: #007acc;
    }
    
    .send-button {
        padding: 10px 20px;
        background: #007acc;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-weight: 600;
        font-size: 14px;
        transition: background-color 0.1s;
    }
    
    .send-button:hover {
        background: #005a9e;
    }
    
    .send-button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
    
    /* Status bar styles */
    .status-bar {
        height: 24px;
        background: #007acc;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 15px;
        font-size: 12px;
        color: white;
    }
    
    .status-left, .status-right {
        display: flex;
        align-items: center;
        gap: 15px;
    }
    
    .git-branch {
        display: flex;
        align-items: center;
        gap: 5px;
    }
"#;

fn main() {
    // Launch the desktop app with proper title using Dioxus 0.6 LaunchBuilder
    use dioxus::desktop::{Config, WindowBuilder};
    
    // TODO: Native menu bar support will be added in Dioxus 0.8
    // For now, we use in-app UI elements for file operations
    // Future menu structure:
    // - File: Open, Open Folder, Open Recent, Save, Save As, Close Folder
    // - View: Appearance settings, Toggle panels
    // - Help: About, Version, Documentation
    
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title("Hive Consensus")
                    .with_resizable(true)
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
                    .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0))
            ),
        )
        .launch(App);
}

#[derive(Debug, Clone, PartialEq)]
struct Message {
    text: String,
    is_user: bool,
}

use hive_ai::desktop::file_system;
use hive_ai::desktop::state::{FileItem, FileType};
use hive_ai::desktop::menu_bar::{MenuBar, MenuAction};
use hive_ai::desktop::dialogs::{AboutDialog, WelcomeTab, CommandPalette, SettingsDialog, OnboardingDialog, WelcomeAction, DIALOG_STYLES};
use hive_ai::desktop::consensus_integration::{DesktopConsensusManager, use_consensus};
use hive_ai::desktop::state::{AppState, ConsensusState};
use std::path::PathBuf;
use std::collections::HashMap;

fn App() -> Element {
    // Initialize database on first render
    use_effect(move || {
        spawn(async move {
            use hive_ai::core::database::{initialize_database, DatabaseConfig};
            
            // Initialize database with proper config
            let config = DatabaseConfig::default();
            if let Err(e) = initialize_database(Some(config)).await {
                // Only log if it's not "already initialized"
                if !e.to_string().contains("already initialized") {
                    eprintln!("Failed to initialize database: {}", e);
                }
            }
        });
    });
    
    // Initialize app state
    let app_state = use_signal(|| AppState::default());
    use_context_provider(|| app_state.clone());
    
    // Get consensus manager
    let consensus_manager = use_consensus();
    
    // State management
    let mut messages = use_signal(|| vec![
        Message { text: "🐝 Welcome to Hive Consensus!".to_string(), is_user: false },
        Message { text: "This is the Rust-powered version with VS Code-style interface.".to_string(), is_user: false },
    ]);
    let mut input_value = use_signal(String::new);
    let mut is_processing = use_signal(|| false);
    let mut selected_file = use_signal(|| Some("__welcome__".to_string()));
    let mut file_tree = use_signal(|| Vec::<FileItem>::new());
    let expanded_dirs = use_signal(|| HashMap::<PathBuf, bool>::new());
    let mut current_dir = use_signal(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let mut file_content = use_signal(String::new);
    
    // Dialog state
    let mut show_about_dialog = use_signal(|| false);
    let mut show_welcome_dialog = use_signal(|| true);
    let mut show_command_palette = use_signal(|| false);
    let mut show_settings_dialog = use_signal(|| false);
    let mut show_onboarding_dialog = use_signal(|| false);
    let mut onboarding_current_step = use_signal(|| 1);  // Persist onboarding step
    
    // API keys state
    let mut openrouter_key = use_signal(String::new);
    let mut hive_key = use_signal(String::new);
    
    // Check if we need to show onboarding (only once on mount)
    use_effect(move || {
        let mut show_onboarding_dialog = show_onboarding_dialog.clone();
        let mut openrouter_key = openrouter_key.clone();
        spawn(async move {
            use hive_ai::core::api_keys::ApiKeyManager;
            
            // Check if API keys are configured
            if !ApiKeyManager::has_valid_keys().await.unwrap_or(false) {
                *show_onboarding_dialog.write() = true;
            } else {
                // Load existing key for settings
                if let Ok(config) = ApiKeyManager::load_from_database().await {
                    if let Some(key) = config.openrouter_key {
                        *openrouter_key.write() = key;
                    }
                }
            }
        });
    });
    
    // Check consensus manager status (only once)
    use_effect(move || {
        let consensus_manager = consensus_manager.clone();
        let mut show_onboarding = show_onboarding_dialog.clone();
        let mut show_welcome = show_welcome_dialog.clone();
        
        if consensus_manager.is_none() {
            // No consensus manager means no API key
            *show_onboarding.write() = true;
            *show_welcome.write() = false;
        }
    });
    
    // Load initial directory
    {
        let mut file_tree = file_tree.clone();
        let current_dir_path = current_dir.read().clone();
        let expanded_map = expanded_dirs.read().clone();
        
        spawn(async move {
            // Create root folder item
            let root_name = current_dir_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Root")
                .to_string();
            
            match file_system::load_directory_tree(&current_dir_path, &expanded_map, false).await {
                Ok(files) => {
                    // Create root folder item with children
                    let root_item = FileItem {
                        path: current_dir_path.clone(),
                        name: root_name,
                        is_directory: true,
                        is_expanded: true, // Root is expanded by default
                        children: files,
                        file_type: FileType::Directory,
                        git_status: None,
                        size: None,
                        modified: None,
                    };
                    
                    file_tree.write().clear();
                    file_tree.write().push(root_item);
                }
                Err(e) => {
                    eprintln!("Error loading directory: {}", e);
                }
            }
        });
    }
    
    // File selection is handled directly in the onclick handler

    // Handle menu actions
    let mut handle_menu_action = move |action: MenuAction| {
        match action {
            MenuAction::OpenFolder => {
                // Open folder dialog
                spawn({
                    let mut current_dir = current_dir.clone();
                    let mut file_tree = file_tree.clone();
                    let mut expanded_dirs = expanded_dirs.clone();
                    let mut selected_file = selected_file.clone();
                    let mut file_content = file_content.clone();
                    
                    async move {
                        let current_path = current_dir.read().clone();
                        if let Some(folder) = rfd::AsyncFileDialog::new()
                            .set_directory(&current_path)
                            .pick_folder()
                            .await
                        {
                            // Update current directory
                            *current_dir.write() = folder.path().to_path_buf();
                            
                            // Clear selected file and content
                            *selected_file.write() = None;
                            *file_content.write() = String::new();
                            
                            // Clear expanded dirs for new folder
                            expanded_dirs.write().clear();
                            
                            // Load new directory tree
                            let root_name = folder.path().file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Root")
                                .to_string();
                            
                            match file_system::load_directory_tree(folder.path(), &HashMap::new(), false).await {
                                Ok(files) => {
                                    // Create root folder item with children
                                    let root_item = FileItem {
                                        path: folder.path().to_path_buf(),
                                        name: root_name,
                                        is_directory: true,
                                        is_expanded: true, // Root is expanded by default
                                        children: files,
                                        file_type: FileType::Directory,
                                        git_status: None,
                                        size: None,
                                        modified: None,
                                    };
                                    
                                    file_tree.write().clear();
                                    file_tree.write().push(root_item);
                                }
                                Err(e) => {
                                    eprintln!("Error loading directory: {}", e);
                                }
                            }
                        }
                    }
                });
            },
            MenuAction::About => {
                *show_about_dialog.write() = true;
            },
            MenuAction::OpenFile => {
                spawn({
                    let mut selected_file = selected_file.clone();
                    let mut file_content = file_content.clone();
                    
                    async move {
                        if let Some(file) = rfd::AsyncFileDialog::new()
                            .pick_file()
                            .await
                        {
                            let path_string = file.path().to_string_lossy().to_string();
                            *selected_file.write() = Some(path_string.clone());
                            
                            match file_system::read_file_content(file.path()).await {
                                Ok(content) => {
                                    *file_content.write() = content;
                                }
                                Err(e) => {
                                    eprintln!("Error reading file: {}", e);
                                    *file_content.write() = format!("// Error reading file: {}", e);
                                }
                            }
                        }
                    }
                });
            },
            MenuAction::Save => {
                // Save current file if one is selected
                if let Some(file_path) = selected_file.read().as_ref() {
                    if file_path != "__welcome__" {
                        let content = file_content.read().clone();
                        let path = PathBuf::from(file_path);
                        spawn(async move {
                            match tokio::fs::write(&path, content).await {
                                Ok(_) => println!("File saved: {}", path.display()),
                                Err(e) => eprintln!("Error saving file: {}", e),
                            }
                        });
                    }
                }
            },
            MenuAction::SaveAs => {
                // Save As dialog
                spawn({
                    let mut selected_file = selected_file.clone();
                    let file_content = file_content.clone();
                    
                    async move {
                        if let Some(file) = rfd::AsyncFileDialog::new()
                            .set_file_name("untitled.txt")
                            .save_file()
                            .await
                        {
                            let content = file_content.read().clone();
                            match tokio::fs::write(file.path(), content).await {
                                Ok(_) => {
                                    println!("File saved as: {}", file.path().display());
                                    *selected_file.write() = Some(file.path().to_string_lossy().to_string());
                                }
                                Err(e) => eprintln!("Error saving file: {}", e),
                            }
                        }
                    }
                });
            },
            MenuAction::CloseFolder => {
                // Clear the current folder
                *current_dir.write() = std::env::current_dir().unwrap_or_default();
                file_tree.write().clear();
                *selected_file.write() = None;
                *file_content.write() = String::new();
            },
            MenuAction::CommandPalette => {
                *show_command_palette.write() = true;
            },
            MenuAction::ChangeTheme => {
                // TODO: Show theme selector
                // For now, just log to console
                println!("Theme selector not yet implemented");
            },
            MenuAction::Settings => {
                *show_settings_dialog.write() = true;
            },
            MenuAction::Welcome => {
                *show_welcome_dialog.write() = true;
                // Set the selected file to show welcome in editor
                *selected_file.write() = Some("__welcome__".to_string());
            },
            MenuAction::Documentation => {
                // Open documentation in browser
                spawn(async {
                    let url = "https://github.com/hivetechs/hive/wiki";
                    match webbrowser::open(url) {
                        Ok(_) => println!("Opening documentation: {}", url),
                        Err(e) => eprintln!("Failed to open browser: {}", e),
                    }
                });
            },
            _ => {
                // Other actions not yet implemented
                println!("{:?} action not implemented yet", action);
            }
        }
    };

    // Handle welcome page actions
    let handle_welcome_action = {
        let mut selected_file = selected_file.clone();
        let mut file_content = file_content.clone();
        let mut show_welcome_dialog = show_welcome_dialog.clone();
        let current_dir = current_dir.clone();
        let file_tree = file_tree.clone();
        let expanded_dirs = expanded_dirs.clone();
        
        move |action: WelcomeAction| {
            match action {
                WelcomeAction::OpenFolder => {
                    // Open folder dialog
                    spawn({
                        let mut current_dir = current_dir.clone();
                        let mut file_tree = file_tree.clone();
                        let mut expanded_dirs = expanded_dirs.clone();
                        let mut selected_file = selected_file.clone();
                        let mut file_content = file_content.clone();
                        
                        async move {
                            let current_path = current_dir.read().clone();
                            if let Some(folder) = rfd::AsyncFileDialog::new()
                                .set_directory(&current_path)
                                .pick_folder()
                                .await
                            {
                                // Update current directory
                                *current_dir.write() = folder.path().to_path_buf();
                                
                                // Clear selected file and content
                                *selected_file.write() = None;
                                *file_content.write() = String::new();
                                
                                // Clear expanded dirs for new folder
                                expanded_dirs.write().clear();
                                
                                // Load new directory tree
                                let root_name = folder.path().file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Root")
                                    .to_string();
                                
                                match file_system::load_directory_tree(folder.path(), &HashMap::new(), false).await {
                                    Ok(files) => {
                                        // Create root folder item with children
                                        let root_item = FileItem {
                                            path: folder.path().to_path_buf(),
                                            name: root_name,
                                            is_directory: true,
                                            is_expanded: true, // Root is expanded by default
                                            children: files,
                                            file_type: FileType::Directory,
                                            git_status: None,
                                            size: None,
                                            modified: None,
                                        };
                                        
                                        file_tree.write().clear();
                                        file_tree.write().push(root_item);
                                    }
                                    Err(e) => {
                                        eprintln!("Error loading directory: {}", e);
                                    }
                                }
                            }
                        }
                    });
                },
                WelcomeAction::OpenRecent => {
                    // TODO: Implement recent files
                    println!("OpenRecent not yet implemented");
                },
                WelcomeAction::NewFile => {
                    // Create new untitled file
                    *selected_file.write() = Some("untitled.txt".to_string());
                    *file_content.write() = String::new();
                    *show_welcome_dialog.write() = false;
                },
            }
        }
    };

    rsx! {
        // Inject VS Code-style CSS and dialog styles
        style { "{DESKTOP_STYLES}" }
        style { "{DIALOG_STYLES}" }
        
        div {
            class: "app-container",
            
            // Menu bar at the top
            MenuBar {
                on_action: handle_menu_action,
            }
            
            // Main content (below menu bar)
            div {
                class: "main-content",
                
                // Sidebar (left)
                div {
                    class: "sidebar",
                    
                    // Sidebar header with current path and open folder button
                    div {
                        class: "sidebar-header",
                        div {
                            class: "current-path",
                            title: "{current_dir.read().display()}",
                            "{current_dir.read().display()}"
                        }
                    }
                    
                    div { class: "sidebar-section-title", "EXPLORER" }
                    
                    // File tree
                    for file in file_tree.read().iter() {
                        FileTreeItem { 
                            file: file.clone(),
                            level: 0,
                            selected_file: selected_file.clone(),
                            expanded_dirs: expanded_dirs.clone(),
                            file_tree: file_tree.clone(),
                            current_dir: current_dir.clone(),
                            file_content: file_content.clone(),
                        }
                    }
                    
                    if file_tree.read().is_empty() {
                        div { 
                            class: "sidebar-item",
                            style: "color: #858585; font-style: italic;",
                            "No files in directory" 
                        }
                    }
                    
                    div { class: "sidebar-section-title", style: "margin-top: 20px;", "ACTIONS" }
                    div { class: "sidebar-item", "🔍 Search" }
                    div { class: "sidebar-item", "📊 Analytics" }
                    div { class: "sidebar-item", "🧠 Memory" }
                    div { 
                        class: "sidebar-item",
                        onclick: move |_| *show_settings_dialog.write() = true,
                        style: "cursor: pointer;",
                        "⚙️ Settings" 
                    }
                }
                
                // Code editor area (center)
                div {
                    class: "editor-container",
                    
                    // Editor tabs
                    div {
                        class: "editor-tabs",
                        {
                            selected_file.read().as_ref().map(|file_path| {
                                let display_name = if file_path == "__welcome__" {
                                    "Welcome".to_string()
                                } else {
                                    let path = PathBuf::from(file_path);
                                    path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or(file_path)
                                        .to_string()
                                };
                                rsx! {
                                    div {
                                        class: "editor-tab active",
                                        "{display_name}"
                                    }
                                }
                            })
                        }
                    }
                    
                    // Editor content
                    div {
                        class: "editor-content",
                        if let Some(file) = selected_file.read().as_ref() {
                            if file == "__welcome__" && *show_welcome_dialog.read() {
                                // Show welcome tab in editor area
                                WelcomeTab {
                                    show_welcome: show_welcome_dialog.clone(),
                                    on_action: handle_welcome_action,
                                }
                            } else {
                                // Show file content
                                pre { 
                                    style: "margin: 0; white-space: pre-wrap; word-wrap: break-word;",
                                    "{file_content.read()}"
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
                
                // Chat panel (right)
                div {
                    class: "chat-panel",
                    
                    // Panel header
                    div {
                        class: "panel-header",
                        "🐝 Hive Consensus Chat"
                    }
                    
                    // Consensus progress display
                    if app_state.read().consensus.is_running {
                        ConsensusProgressDisplay { 
                            consensus_state: app_state.read().consensus.clone() 
                        }
                    }
                    
                    // Messages
                    div {
                        class: "messages-area",
                        for msg in messages.read().iter() {
                            div {
                                class: if msg.is_user { "message user-message" } else { "message ai-message" },
                                "{msg.text}"
                            }
                        }
                    }
                    
                    // Input area
                    div {
                        class: "input-area",
                        input {
                            class: "message-input",
                            value: "{input_value.read()}",
                            placeholder: "Ask Hive anything...",
                            disabled: *is_processing.read(),
                            oninput: move |evt| *input_value.write() = evt.value().clone(),
                            onkeypress: {
                                let consensus_manager = consensus_manager.clone();
                                move |evt: dioxus::events::KeyboardEvent| {
                                if evt.code() == dioxus::events::Code::Enter && !input_value.read().is_empty() && !*is_processing.read() {
                                    let user_msg = input_value.read().clone();
                                    
                                    // Add user message
                                    messages.write().push(Message { text: user_msg.clone(), is_user: true });
                                    
                                    // Clear input
                                    input_value.write().clear();
                                    
                                    // Start processing
                                    *is_processing.write() = true;
                                    
                                    // Use consensus engine if available
                                    if let Some(mut consensus) = consensus_manager.clone() {
                                        let mut messages = messages.clone();
                                        let mut is_processing = is_processing.clone();
                                        let app_state = app_state.clone();
                                        
                                        spawn(async move {
                                            // Update UI to show consensus is running
                                            let mut app_state = app_state.clone();
                                            app_state.write().consensus.start_consensus();
                                            
                                            match consensus.process_query(&user_msg).await {
                                                Ok(response) => {
                                                    messages.write().push(Message { 
                                                        text: response, 
                                                        is_user: false 
                                                    });
                                                }
                                                Err(e) => {
                                                    messages.write().push(Message { 
                                                        text: format!("❌ Error: {}", e), 
                                                        is_user: false 
                                                    });
                                                }
                                            }
                                            
                                            // Update UI to show consensus is complete
                                            let mut app_state = app_state.clone();
                                            app_state.write().consensus.complete_consensus();
                                            *is_processing.write() = false;
                                        });
                                    } else {
                                        // Show onboarding if consensus engine not initialized
                                        messages.write().push(Message { 
                                            text: "⚠️ OpenRouter API key not configured. Click the Settings button to add your API key.".to_string(), 
                                            is_user: false 
                                        });
                                        *is_processing.write() = false;
                                        
                                        // Show onboarding dialog
                                        *show_onboarding_dialog.write() = true;
                                    }
                                }
                            }}
                        }
                        button {
                            class: "send-button",
                            disabled: input_value.read().is_empty() || *is_processing.read(),
                            onclick: {
                                let consensus_manager = consensus_manager.clone();
                                move |_| {
                                if !input_value.read().is_empty() && !*is_processing.read() {
                                    let user_msg = input_value.read().clone();
                                    
                                    // Add user message
                                    messages.write().push(Message { text: user_msg.clone(), is_user: true });
                                    
                                    // Clear input
                                    input_value.write().clear();
                                    
                                    // Start processing
                                    *is_processing.write() = true;
                                    
                                    // Use consensus engine if available
                                    if let Some(mut consensus) = consensus_manager.clone() {
                                        let mut messages = messages.clone();
                                        let mut is_processing = is_processing.clone();
                                        let app_state = app_state.clone();
                                        
                                        spawn(async move {
                                            // Update UI to show consensus is running
                                            let mut app_state = app_state.clone();
                                            app_state.write().consensus.start_consensus();
                                            
                                            match consensus.process_query(&user_msg).await {
                                                Ok(response) => {
                                                    messages.write().push(Message { 
                                                        text: response, 
                                                        is_user: false 
                                                    });
                                                }
                                                Err(e) => {
                                                    messages.write().push(Message { 
                                                        text: format!("❌ Error: {}", e), 
                                                        is_user: false 
                                                    });
                                                }
                                            }
                                            
                                            // Update UI to show consensus is complete
                                            let mut app_state = app_state.clone();
                                            app_state.write().consensus.complete_consensus();
                                            *is_processing.write() = false;
                                        });
                                    } else {
                                        // Show onboarding if consensus engine not initialized
                                        messages.write().push(Message { 
                                            text: "⚠️ OpenRouter API key not configured. Click the Settings button to add your API key.".to_string(), 
                                            is_user: false 
                                        });
                                        *is_processing.write() = false;
                                        
                                        // Show onboarding dialog
                                        *show_onboarding_dialog.write() = true;
                                    }
                                }
                            }},
                            if *is_processing.read() { "Processing..." } else { "Send" }
                        }
                    }
                }
            }
            
            // Status bar
            div {
                class: "status-bar",
                div { 
                    class: "status-left",
                    div {
                        class: "git-branch",
                        "🌿 main"
                    }
                    "•"
                    "✓ 0 problems"
                }
                div { 
                    class: "status-right",
                    "Ln 1, Col 1",
                    "•",
                    "UTF-8",
                    "•",
                    "Rust"
                }
            }
        }
        
        // Render dialogs
        if *show_about_dialog.read() {
            AboutDialog {
                show_about: show_about_dialog.clone(),
            }
        }
        
        if *show_command_palette.read() {
            CommandPalette {
                show_palette: show_command_palette.clone(),
            }
        }
        
        if *show_settings_dialog.read() {
            SettingsDialog {
                show_settings: show_settings_dialog.clone(),
                openrouter_key: openrouter_key.clone(),
                hive_key: hive_key.clone(),
            }
        }
        
        if *show_onboarding_dialog.read() {
            OnboardingDialog {
                show_onboarding: show_onboarding_dialog.clone(),
                openrouter_key: openrouter_key.clone(),
                hive_key: hive_key.clone(),
                current_step: onboarding_current_step.clone(),
            }
        }
    }
}

#[component]
fn FileTreeItem(
    file: FileItem,
    level: usize,
    selected_file: Signal<Option<String>>,
    expanded_dirs: Signal<HashMap<PathBuf, bool>>,
    file_tree: Signal<Vec<FileItem>>,
    current_dir: Signal<PathBuf>,
    file_content: Signal<String>,
) -> Element {
    let file_path = file.path.clone();
    let file_name = file.name.clone();
    let is_dir = file.is_directory;
    
    // Calculate indentation
    let indent = level * 20;
    
    // Check if selected
    let is_selected = selected_file.read().as_ref() == Some(&file_path.to_string_lossy().to_string());
    
    // Check if expanded
    let is_expanded = if is_dir {
        expanded_dirs.read().get(&file_path).copied().unwrap_or(false)
    } else {
        false
    };
    
    // File icon
    let icon = if is_dir {
        if is_expanded { "📂" } else { "📁" }
    } else {
        file.file_type.icon()
    };
    
    rsx! {
        div {
            class: if is_selected { "sidebar-item active" } else { "sidebar-item" },
            style: "padding-left: {indent + 20}px;",
            onclick: move |_| {
                if is_dir {
                    // Toggle expansion
                    let current = expanded_dirs.read().get(&file_path).copied().unwrap_or(false);
                    expanded_dirs.write().insert(file_path.clone(), !current);
                    
                    // Trigger reload by changing a dummy state
                    // This will cause the coroutine to re-run
                    // (In a real app, we'd use a proper reload trigger)
                    // Just log for now
                    println!("Directory expanded/collapsed");
                } else {
                    // Select file
                    println!("File clicked: {}", file_path.display());
                    let path_string = file_path.to_string_lossy().to_string();
                    *selected_file.write() = Some(path_string.clone());
                    
                    // Load file content immediately
                    let mut file_content = file_content.clone();
                    let file_path = file_path.clone();
                    spawn(async move {
                        match file_system::read_file_content(&file_path).await {
                            Ok(content) => {
                                println!("File content loaded immediately, {} bytes", content.len());
                                *file_content.write() = content;
                            }
                            Err(e) => {
                                println!("Error reading file immediately: {}", e);
                                *file_content.write() = format!("// Error reading file: {}", e);
                            }
                        }
                    });
                }
            },
            "{icon} {file_name}"
        }
        
        // Render children if expanded
        if is_dir && is_expanded {
            for child in &file.children {
                FileTreeItem {
                    file: child.clone(),
                    level: level + 1,
                    selected_file: selected_file.clone(),
                    expanded_dirs: expanded_dirs.clone(),
                    file_tree: file_tree.clone(),
                    current_dir: current_dir.clone(),
                    file_content: file_content.clone(),
                }
            }
        }
    }
}

#[component]
fn ConsensusProgressDisplay(consensus_state: ConsensusState) -> Element {
    rsx! {
        div {
            style: "padding: 10px; background: #2d2d30; border-bottom: 1px solid #3e3e42;",
            
            // Show all 4 stages
            for (_idx, stage) in consensus_state.stages.iter().enumerate() {
                div {
                    style: "margin: 5px 0;",
                    
                    // Stage info
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 2px;",
                        span { 
                            style: "color: #cccccc; font-size: 12px; font-weight: 600;",
                            "{stage.name}"
                        }
                        span { 
                            style: "color: #858585; font-size: 11px;",
                            "{stage.model}"
                        }
                        span { 
                            style: match stage.status {
                                hive_ai::desktop::state::StageStatus::Waiting => "color: #666666; font-size: 11px;",
                                hive_ai::desktop::state::StageStatus::Running => "color: #007acc; font-size: 11px;",
                                hive_ai::desktop::state::StageStatus::Completed => "color: #4caf50; font-size: 11px;",
                                hive_ai::desktop::state::StageStatus::Error => "color: #f44336; font-size: 11px;",
                            },
                            match stage.status {
                                hive_ai::desktop::state::StageStatus::Waiting => "Waiting",
                                hive_ai::desktop::state::StageStatus::Running => "Running",
                                hive_ai::desktop::state::StageStatus::Completed => "Complete",
                                hive_ai::desktop::state::StageStatus::Error => "Error",
                            }
                        }
                    }
                    
                    // Progress bar
                    div {
                        style: "background: #1e1e1e; height: 4px; border-radius: 2px; overflow: hidden;",
                        div {
                            style: format!("background: {}; height: 100%; width: {}%; transition: width 0.3s;",
                                match stage.status {
                                    hive_ai::desktop::state::StageStatus::Waiting => "#666666",
                                    hive_ai::desktop::state::StageStatus::Running => "#007acc",
                                    hive_ai::desktop::state::StageStatus::Completed => "#4caf50",
                                    hive_ai::desktop::state::StageStatus::Error => "#f44336",
                                },
                                stage.progress
                            ),
                        }
                    }
                }
            }
            
            // Show cost and tokens
            if consensus_state.total_tokens > 0 {
                div {
                    style: "margin-top: 10px; padding-top: 10px; border-top: 1px solid #3e3e42; display: flex; justify-content: space-between; font-size: 11px; color: #858585;",
                    span { "Tokens: {consensus_state.total_tokens}" }
                    span { "Cost: ${consensus_state.estimated_cost:.4}" }
                }
            }
        }
    }
}