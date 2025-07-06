//! Main Desktop Application Component

use dioxus::prelude::*;
use dioxus::events::{KeyboardEvent, MouseEvent};
use crate::desktop::{
    file_explorer::FileExplorer,
    chat::ChatInterface,
    consensus::ConsensusProgress,
    state::{AppState, ConnectionStatus},
    events::{EventDispatcher, KeyboardEventUtils},
    styles::get_global_styles,
    dialogs::{OnboardingDialog, SettingsDialog, AboutDialog, CommandPalette},
};
use crate::core::api_keys::ApiKeyManager;

/// Main Application Component
#[component]
pub fn App() -> Element {
    // Initialize application state
    let app_state = use_signal(|| AppState::new());
    
    // Initialize event dispatcher
    let event_dispatcher = use_signal(|| EventDispatcher::new());
    
    // Dialog visibility states
    let mut show_onboarding = use_signal(|| false);
    let mut show_settings = use_signal(|| false);
    let mut show_about = use_signal(|| false);
    let mut show_command_palette = use_signal(|| false);
    
    // API key states
    let mut openrouter_key = use_signal(|| String::new());
    let mut hive_key = use_signal(|| String::new());
    
    // Check for API keys and onboarding status on startup (only once)
    use_effect(move || {
        let mut show_onboarding = show_onboarding.clone();
        let mut openrouter_key = openrouter_key.clone();
        let mut hive_key = hive_key.clone();
        
        spawn(async move {
            // Import needed functions
            use crate::desktop::dialogs::load_existing_profiles;
            use crate::core::database::{DatabaseManager, DatabaseConfig};
            use crate::core::config::get_hive_config_dir;
            
            // Check if onboarding has been completed
            let db_path = get_hive_config_dir().join("hive.db");
            let onboarding_completed = if !db_path.exists() {
                false
            } else {
                let db_config = DatabaseConfig {
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
                
                if let Ok(db) = DatabaseManager::new(db_config).await {
                    if let Ok(conn) = db.get_connection() {
                        if let Ok(mut stmt) = conn.prepare(
                            "SELECT value FROM configurations WHERE key = 'onboarding_completed'"
                        ) {
                            if let Ok(mut rows) = stmt.query([]) {
                                if let Ok(Some(_)) = rows.next() {
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            
            // Check if onboarding is already completed
            if onboarding_completed {
                tracing::info!("Onboarding already completed - loading keys");
                // Just load the keys, don't show onboarding
                if let Ok(config) = ApiKeyManager::load_from_database().await {
                    if let Some(key) = config.openrouter_key {
                        *openrouter_key.write() = key;
                    }
                    if let Some(key) = config.hive_key {
                        *hive_key.write() = key;
                    }
                }
                return;
            }
            
            // Otherwise, check if we need to show onboarding
            match ApiKeyManager::has_valid_keys().await {
                Ok(has_keys) => {
                    if !has_keys {
                        tracing::info!("No API keys found - showing onboarding");
                        *show_onboarding.write() = true;
                    } else {
                        // Load existing keys
                        if let Ok(config) = ApiKeyManager::load_from_database().await {
                            if let Some(key) = config.openrouter_key {
                                *openrouter_key.write() = key;
                            }
                            if let Some(key) = config.hive_key {
                                *hive_key.write() = key;
                            }
                            
                            // Check if profiles exist
                            if let Ok(profiles) = load_existing_profiles().await {
                                if profiles.is_empty() {
                                    tracing::info!("Keys exist but no profiles - showing onboarding");
                                    *show_onboarding.write() = true;
                                } else {
                                    tracing::info!("Setup complete - keys and profiles exist");
                                    // Mark onboarding as complete since we have everything
                                    spawn(async {
                                        use crate::desktop::dialogs::mark_onboarding_complete;
                                        if let Err(e) = mark_onboarding_complete().await {
                                            tracing::warn!("Failed to mark onboarding complete: {}", e);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to check API keys: {}", e);
                    *show_onboarding.write() = true;
                }
            }
        });
    });
    
    // Provide state to all child components
    use_context_provider(move || app_state);
    use_context_provider(|| event_dispatcher);
    use_context_provider(|| show_settings.clone());
    use_context_provider(|| show_about.clone());
    use_context_provider(|| show_command_palette.clone());
    
    // Global keyboard shortcuts
    let on_global_keydown = move |evt: KeyboardEvent| {
        // Handle global keyboard shortcuts
        if evt.modifiers().ctrl() {
            match evt.key() {
                Key::Character(c) if c == "n" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: New file/conversation
                    tracing::debug!("Ctrl+N pressed - New file");
                },
                Key::Character(c) if c == "o" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Open file dialog
                    tracing::debug!("Ctrl+O pressed - Open file");
                },
                Key::Character(c) if c == "s" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Save current file
                    tracing::debug!("Ctrl+S pressed - Save file");
                },
                Key::Character(c) if c == "f" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Search/find
                    tracing::debug!("Ctrl+F pressed - Find");
                },
                Key::Character(c) if c == "p" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    *show_command_palette.write() = true;
                    tracing::debug!("Ctrl+P pressed - Command palette");
                },
                Key::Character(c) if c == "w" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Close current tab/file
                    tracing::debug!("Ctrl+W pressed - Close tab");
                },
                Key::Character(c) if c == "z" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Undo
                    tracing::debug!("Ctrl+Z pressed - Undo");
                },
                Key::Character(c) if c == "y" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Redo
                    tracing::debug!("Ctrl+Y pressed - Redo");
                },
                _ => {}
            }
        } else if evt.modifiers().alt() {
            match evt.key() {
                Key::Character(c) if c == "1" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Focus file explorer
                    tracing::debug!("Alt+1 pressed - Focus file explorer");
                },
                Key::Character(c) if c == "2" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Focus chat
                    tracing::debug!("Alt+2 pressed - Focus chat");
                },
                Key::Character(c) if c == "3" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Focus consensus panel
                    tracing::debug!("Alt+3 pressed - Focus consensus");
                },
                _ => {}
            }
        } else if evt.key() == Key::F1 {
            // evt.prevent_default(); // Not available in Dioxus 0.5"
            // TODO: Show help
            tracing::debug!("F1 pressed - Show help");
        }
    };
    
    rsx! {
        style { {get_global_styles()} }
        
        div {
            id: "app",
            class: "app-container",
            tabindex: 0,
            onkeydown: on_global_keydown,
            
            // Menu Bar Component
            MenuBar {}
            
            // Main Content Layout
            div {
                class: "main-layout",
                
                // Left Sidebar - File Explorer
                FileExplorer {}
                
                // Right Panel - Chat Interface  
                ChatInterface {}
                
                // Consensus Progress (overlay when active)
                ConsensusProgress {}
            }
            
            // Status Bar
            StatusBar {}
            
            // Dialogs (rendered on top of everything)
            if *show_onboarding.read() {
                OnboardingDialog { 
                    show_onboarding, 
                    openrouter_key,
                    hive_key
                }
            }
            
            if *show_settings.read() {
                SettingsDialog { 
                    show_settings, 
                    openrouter_key,
                    hive_key
                }
            }
            
            if *show_about.read() {
                AboutDialog { show_about }
            }
            
            if *show_command_palette.read() {
                CommandPalette { show_palette: show_command_palette }
            }
        }
    }
}

/// Menu Bar Component
#[component]
fn MenuBar() -> Element {
    // Get the show_settings signal from parent
    let mut show_settings = use_context::<Signal<bool>>();
    // VS Code-like colors
    let menu_bar_style = "
        display: flex;
        justify-content: space-between;
        align-items: center;
        height: 30px;
        background-color: #2d2d30;
        border-bottom: 1px solid #474747;
        padding: 0 10px;
        -webkit-app-region: drag;
        user-select: none;
    ";
    
    let title_style = "
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 13px;
        color: #cccccc;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'SF Pro', system-ui, sans-serif;
    ";
    
    let menu_actions_style = "
        display: flex;
        align-items: center;
        gap: 0;
        -webkit-app-region: no-drag;
    ";
    
    let menu_btn_style = "
        background: transparent;
        border: none;
        color: #cccccc;
        width: 46px;
        height: 30px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 16px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'SF Pro', system-ui, sans-serif;
        cursor: pointer;
        transition: background-color 0.1s;
        padding: 0;
        margin: 0;
        outline: none;
    ";
    
    let menu_btn_hover_style = "
        background: transparent;
        border: none;
        color: #cccccc;
        width: 46px;
        height: 30px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 16px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'SF Pro', system-ui, sans-serif;
        cursor: pointer;
        transition: background-color 0.1s;
        padding: 0;
        margin: 0;
        outline: none;
    ";
    
    // State for hover effects
    let mut settings_hovered = use_signal(|| false);
    let mut minimize_hovered = use_signal(|| false);
    let mut close_hovered = use_signal(|| false);
    
    rsx! {
        div {
            style: "{menu_bar_style}",
            
            div {
                style: "{title_style}",
                // Using text icon instead of emoji for modern look
                span { 
                    style: "font-weight: 600; font-size: 14px;",
                    "◆" 
                }
                span { 
                    style: "font-weight: 400;",
                    "HiveTechs Consensus" 
                }
            }
            
            div {
                style: "{menu_actions_style}",
                button {
                    style: if settings_hovered() { 
                        format!("{} background-color: #3e3e42;", menu_btn_style) 
                    } else { 
                        menu_btn_style.to_string() 
                    },
                    onmouseenter: move |_| settings_hovered.set(true),
                    onmouseleave: move |_| settings_hovered.set(false),
                    onclick: move |_| {
                        *show_settings.write() = true;
                    },
                    "⚙"  // Cleaner gear icon without emoji variant
                }
                button {
                    style: if minimize_hovered() { 
                        format!("{} background-color: #3e3e42;", menu_btn_style) 
                    } else { 
                        menu_btn_style.to_string() 
                    },
                    onmouseenter: move |_| minimize_hovered.set(true),
                    onmouseleave: move |_| minimize_hovered.set(false),
                    onclick: |_| {
                        // Handle minimize
                    },
                    "‒"  // Proper minus sign (en dash)
                }
                button {
                    style: if close_hovered() { 
                        format!("{} background-color: #e81123; color: white;", menu_btn_style) 
                    } else { 
                        menu_btn_style.to_string() 
                    },
                    onmouseenter: move |_| close_hovered.set(true),
                    onmouseleave: move |_| close_hovered.set(false),
                    onclick: |_| {
                        // Handle close
                    },
                    "✕"  // Modern X symbol
                }
            }
        }
    }
}

/// Status Bar Component
#[component]
fn StatusBar() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let state = app_state.read();
    
    rsx! {
        div {
            class: "status-bar",
            
            div {
                class: "status-left",
                // Connection Status
                div {
                    class: "status-item",
                    span {
                        class: match state.connection_status {
                            ConnectionStatus::Connected => "status-indicator connected",
                            ConnectionStatus::Connecting => "status-indicator connecting",
                            ConnectionStatus::Disconnected => "status-indicator disconnected",
                        }
                    }
                    span {
                        match state.connection_status {
                            ConnectionStatus::Connected => "Connected",
                            ConnectionStatus::Connecting => "Connecting",
                            ConnectionStatus::Disconnected => "Disconnected",
                        }
                    }
                }
                
                // Current Project
                div {
                    class: "status-item",
                    {state.current_project.as_ref().map(|p| p.name.as_str()).unwrap_or("No workspace")}
                }
                
                // Git Branch (if available)
                if let Some(project) = &state.current_project {
                    if let Some(branch) = &project.git_branch {
                        div {
                            class: "status-item",
                            "git: {branch}"
                        }
                    }
                }
            }
            
            div {
                class: "status-right",
                // Cost Indicator
                div {
                    class: "status-item",
                    "Cost: ${state.total_cost:.3}"
                }
                
                // Context Usage
                div {
                    class: "status-item",
                    "Context: {state.context_usage}%"
                }
                
                // Auto-accept Toggle
                div {
                    class: if state.auto_accept { "status-item auto-accept-toggle enabled" } else { "status-item auto-accept-toggle disabled" },
                    onclick: move |_| {
                        let current_value = app_state.read().auto_accept;
                        app_state.write().auto_accept = !current_value;
                    },
                    if state.auto_accept { "Auto: ON" } else { "Auto: OFF" }
                }
                
                // Model Indicator
                if let Some(model) = &state.current_model {
                    div {
                        class: "status-item",
                        "{model}"
                    }
                }
            }
        }
    }
}