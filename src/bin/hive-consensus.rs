#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus::document::eval;
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
        flex: 1;
        background: #1e1e1e;
        display: flex;
        flex-direction: column;
        min-width: 400px;
    }
    
    .panel-header {
        background: #2d2d30;
        padding: 10px 20px;
        border-bottom: 1px solid #3e3e42;
        font-weight: 600;
        font-size: 14px;
    }
    
    /* Response area - Claude Code style */
    .response-area {
        flex: 1;
        overflow-y: auto;
        padding: 20px 24px;
        font-size: 14px;
        line-height: 1.6;
        color: #cccccc;
        scroll-behavior: smooth;
    }
    
    .response-content {
        /* Markdown content styling */
        padding-bottom: 20px;
    }
    
    .response-content h1 {
        font-size: 24px;
        font-weight: 600;
        margin: 24px 0 16px 0;
        color: #ffffff;
    }
    
    .response-content h2 {
        font-size: 20px;
        font-weight: 600;
        margin: 20px 0 12px 0;
        color: #ffffff;
    }
    
    .response-content h3 {
        font-size: 16px;
        font-weight: 600;
        margin: 16px 0 8px 0;
        color: #ffffff;
    }
    
    .response-content p {
        margin: 12px 0;
    }
    
    .response-content code {
        background: #2d2d30;
        padding: 2px 6px;
        border-radius: 3px;
        font-family: 'Cascadia Code', 'Consolas', monospace;
        font-size: 13px;
    }
    
    .response-content pre {
        background: #2d2d30;
        border: 1px solid #3e3e42;
        border-radius: 6px;
        padding: 16px;
        overflow-x: auto;
        margin: 16px 0;
    }
    
    .response-content pre code {
        background: none;
        padding: 0;
    }
    
    .response-content ul, .response-content ol {
        margin: 12px 0;
        padding-left: 24px;
    }
    
    .response-content li {
        margin: 6px 0;
    }
    
    .response-content blockquote {
        border-left: 3px solid #007acc;
        padding-left: 16px;
        margin: 16px 0;
        color: #a0a0a0;
    }
    
    .welcome-text {
        color: #808080;
        text-align: center;
        margin-top: 40%;
        transform: translateY(-50%);
        font-size: 14px;
    }
    
    .error {
        color: #f48771;
        background: #362121;
        padding: 12px 16px;
        border-radius: 6px;
        border: 1px solid #5a1d1d;
    }
    
    /* Input area - Claude Code style */
    .input-container {
        padding: 16px 24px;
        background: #252526;
        border-top: 1px solid #3e3e42;
    }
    
    .query-input {
        width: 100%;
        background: #3c3c3c;
        border: 1px solid #3e3e42;
        color: #cccccc;
        padding: 12px 16px;
        border-radius: 6px;
        font-size: 14px;
        font-family: inherit;
        transition: border-color 0.2s;
        resize: vertical;
        min-height: 60px;
        max-height: 200px;
    }
    
    .query-input:focus {
        outline: none;
        border-color: #007acc;
    }
    
    .query-input:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
    
    .query-input::placeholder {
        color: #808080;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Create a runtime to initialize the database before launching desktop
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        // Initialize the database
        let config = hive_ai::core::config::get_hive_config_dir();
        let db_path = config.join("hive-ai.db");
        let db_config = hive_ai::core::database::DatabaseConfig {
            path: db_path,
            max_connections: 10,
            connection_timeout: std::time::Duration::from_secs(5),
            idle_timeout: std::time::Duration::from_secs(300),
            enable_wal: true,
            enable_foreign_keys: true,
            cache_size: 8192,
            synchronous: "NORMAL".to_string(),
            journal_mode: "WAL".to_string(),
        };
        hive_ai::core::database::initialize_database(Some(db_config)).await?;
        Ok::<(), anyhow::Error>(())
    })?;
    
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
    
    Ok(())
}

use hive_ai::desktop::file_system;
use hive_ai::desktop::state::{FileItem, FileType};
use hive_ai::desktop::menu_bar::{MenuBar, MenuAction};
use hive_ai::desktop::dialogs::{AboutDialog, WelcomeTab, CommandPalette, SettingsDialog, OnboardingDialog, WelcomeAction, DIALOG_STYLES};
use hive_ai::desktop::consensus_integration::use_consensus_with_version;

// Simple markdown to HTML converter
mod markdown {
    use pulldown_cmark::{html, Parser};
    
    pub fn to_html(markdown: &str) -> String {
        let parser = Parser::new(markdown);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
}
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
    let mut app_state = use_signal(|| AppState::default());
    use_context_provider(|| app_state.clone());
    
    // API keys state (needed before consensus manager)
    let openrouter_key = use_signal(String::new);
    let hive_key = use_signal(String::new);
    let api_keys_version = use_signal(|| 0u32);  // Track when API keys change
    
    // Get consensus manager
    let consensus_manager = use_consensus_with_version(*api_keys_version.read());
    
    // State management
    let mut current_response = use_signal(String::new);  // Final response
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
    
    // Subscription state
    let subscription_display = use_signal(|| String::from("Loading..."));
    
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
    
    // Load subscription info periodically and on trigger changes
    use_effect({
        let mut subscription_display = subscription_display.clone();
        let refresh_trigger = app_state.read().subscription_refresh_trigger;
        move || {
            // Load immediately when trigger changes or on initial load
            spawn({
                let mut subscription_display = subscription_display.clone();
                async move {
                    // Wait a bit for database initialization on first load
                    if refresh_trigger == 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                    
                    use hive_ai::subscription::SubscriptionDisplay;
                    
                    match SubscriptionDisplay::load_from_database().await {
                        Ok(mut sub_info) => {
                            tracing::info!("Subscription loaded: user={}, tier={:?}, is_trial={}, daily_used={}, daily_remaining={}", 
                                sub_info.username, sub_info.tier, sub_info.is_trial, sub_info.daily_used, sub_info.daily_remaining);
                            // Get total remaining from app state if available
                            if let Some(total) = app_state.read().total_conversations_remaining {
                                sub_info.update_from_d1(total);
                            }
                            let formatted = sub_info.format();
                            tracing::info!("Subscription display: {}", formatted);
                            *subscription_display.write() = formatted;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load subscription info: {}", e);
                            *subscription_display.write() = "FREE | 10/10 daily".to_string();
                        }
                    }
                }
            });
            
            // Refresh every 30 seconds using tokio interval
            let mut subscription_display = subscription_display.clone();
            spawn(async move {
                use tokio::time::{interval, Duration};
                let mut interval = interval(Duration::from_secs(30));
                
                loop {
                    interval.tick().await;
                    
                    use hive_ai::subscription::SubscriptionDisplay;
                    match SubscriptionDisplay::load_from_database().await {
                        Ok(mut sub_info) => {
                            // Get total remaining from app state if available
                            if let Some(total) = app_state.read().total_conversations_remaining {
                                sub_info.update_from_d1(total);
                            }
                            *subscription_display.write() = sub_info.format();
                        }
                        Err(_) => {
                            // Keep existing display on error
                        }
                    }
                }
            });
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
    
    // Track if we should auto-scroll
    let mut should_auto_scroll = use_signal(|| true);
    
    // Auto-scroll response area when streaming content changes
    let previous_content_length = use_signal(|| 0usize);
    
    use_effect({
        let app_state = app_state.clone();
        let mut previous_content_length = previous_content_length.clone();
        let should_auto_scroll = should_auto_scroll.clone();
        move || {
            let current_length = app_state.read().consensus.streaming_content.len();
            if current_length > *previous_content_length.read() && *should_auto_scroll.read() {
                *previous_content_length.write() = current_length;
                
                // Use eval to scroll to bottom
                let eval = eval(r#"
                    const responseArea = document.getElementById('response-area');
                    if (responseArea) {
                        responseArea.scrollTop = responseArea.scrollHeight;
                    }
                "#);
                
                spawn(async move {
                    let _ = eval.await;
                });
            }
        }
    });

    // Handle menu actions
    let handle_menu_action = move |action: MenuAction| {
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
                        "🐝 Hive Consensus"
                    }
                    
                    // Consensus progress display (always visible at the top)
                    if app_state.read().consensus.is_running {
                        ConsensusProgressDisplay { 
                            consensus_state: app_state.read().consensus.clone() 
                        }
                    }
                    
                    // Response display area (Claude Code style)
                    div {
                        class: "response-area",
                        id: "response-area",
                        // Force re-render when content changes to trigger scroll
                        key: "{app_state.read().consensus.streaming_content.len()}",
                        onscroll: move |_| {
                            // Check if user scrolled away from bottom
                            let eval = eval(r#"
                                const responseArea = document.getElementById('response-area');
                                if (responseArea) {
                                    const isAtBottom = responseArea.scrollTop + responseArea.clientHeight >= responseArea.scrollHeight - 10;
                                    isAtBottom
                                } else {
                                    true
                                }
                            "#);
                            
                            spawn(async move {
                                match eval.await {
                                    Ok(result) => {
                                        if let Some(is_at_bottom) = result.as_bool() {
                                            *should_auto_scroll.write() = is_at_bottom;
                                        }
                                    }
                                    Err(_) => {
                                        // Default to auto-scroll if we can't detect position
                                        *should_auto_scroll.write() = true;
                                    }
                                }
                            });
                        },
                        if !app_state.read().consensus.streaming_content.is_empty() {
                            // Show streaming content in real-time from all stages
                            div {
                                class: "response-content",
                                dangerous_inner_html: "{markdown::to_html(&app_state.read().consensus.streaming_content)}"
                            }
                        } else if !current_response.read().is_empty() {
                            // Show final response if no streaming content
                            div {
                                class: "response-content",
                                dangerous_inner_html: "{current_response.read()}"
                            }
                        } else if *is_processing.read() && app_state.read().consensus.is_running {
                            // Show processing message while consensus starts
                            div {
                                class: "processing-message",
                                style: "color: #cccccc; text-align: center; margin-top: 20%; font-size: 14px; line-height: 1.6;",
                                "🧠 Starting 4-stage consensus pipeline..."
                                br {}
                                small {
                                    style: "color: #808080; font-size: 12px;",
                                    "Generator → Refiner → Validator → Curator"
                                }
                            }
                        } else if !*is_processing.read() {
                            div {
                                class: "welcome-text",
                                "Ask Hive anything. Your query will be processed through our 4-stage consensus pipeline."
                            }
                        }
                    }
                    
                    // Input box at the bottom (Claude Code style)
                    div {
                        class: "input-container",
                        textarea {
                            class: "query-input",
                            value: "{input_value.read()}",
                            placeholder: "Ask Hive anything...",
                            disabled: *is_processing.read(),
                            rows: "3",
                            oninput: move |evt| *input_value.write() = evt.value().clone(),
                            onkeydown: {
                                let consensus_manager = consensus_manager.clone();
                                move |evt: dioxus::events::KeyboardEvent| {
                                    // Enter without shift submits
                                    if evt.key() == dioxus::events::Key::Enter && !evt.modifiers().shift() && !input_value.read().is_empty() && !*is_processing.read() {
                                        evt.prevent_default();
                                        
                                        let user_msg = input_value.read().clone();
                                        
                                        // Clear input and response
                                        input_value.write().clear();
                                        current_response.write().clear();
                                        app_state.write().consensus.streaming_content.clear();
                                        
                                        // Re-enable auto-scroll for new query
                                        *should_auto_scroll.write() = true;
                                        
                                        // Start processing
                                        *is_processing.write() = true;
                                        
                                        // Use consensus engine if available
                                        if let Some(mut consensus) = consensus_manager.clone() {
                                            let mut current_response = current_response.clone();
                                            let mut is_processing = is_processing.clone();
                                            let mut app_state = app_state.clone();
                                            
                                            spawn(async move {
                                                // Update UI to show consensus is running
                                                app_state.write().consensus.start_consensus();
                                                
                                                // Process the query - streaming will update app_state automatically
                                                match consensus.process_query(&user_msg).await {
                                                    Ok(final_response) => {
                                                        // Set final response
                                                        let html = markdown::to_html(&final_response);
                                                        *current_response.write() = html;
                                                    }
                                                    Err(e) => {
                                                        *current_response.write() = format!("<div class='error'>❌ Error: {}</div>", e);
                                                    }
                                                }
                                                
                                                // Update UI to show consensus is complete
                                                app_state.write().consensus.complete_consensus();
                                                *is_processing.write() = false;
                                            });
                                        } else {
                                            // Show error if consensus engine not initialized
                                            *current_response.write() = "<div class='error'>⚠️ OpenRouter API key not configured. Click the Settings button to add your API key.</div>".to_string();
                                            *is_processing.write() = false;
                                            
                                            // Show onboarding dialog
                                            *show_onboarding_dialog.write() = true;
                                        }
                                    }
                                }
                            }
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
                    "•"
                    "{subscription_display.read()}"
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
                api_keys_version: api_keys_version.clone(),
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