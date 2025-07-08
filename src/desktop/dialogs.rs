//! Dialog components for the desktop application

use dioxus::prelude::*;
use anyhow;

/// Information about a consensus profile
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileInfo {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WelcomeAction {
    OpenFolder,
    OpenRecent,
    NewFile,
}

/// Expert template option component
#[component]
fn ExpertTemplateOption(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    use_cases: &'static str,
    is_selected: bool,
    on_select: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: if is_selected { "template-option selected" } else { "template-option" },
            style: if is_selected { 
                "padding: 15px; background: #2d2d30; border: 2px solid #007acc; border-radius: 8px; cursor: pointer; transition: all 0.2s;" 
            } else { 
                "padding: 15px; background: #2d2d30; border: 2px solid #3e3e42; border-radius: 8px; cursor: pointer; transition: all 0.2s;" 
            },
            onclick: move |_| on_select.call(()),
            
            div {
                style: "display: flex; align-items: center; margin-bottom: 8px;",
                h4 {
                    style: "margin: 0; color: #ffffff; font-size: 16px;",
                    "{name}"
                }
                if is_selected {
                    span {
                        style: "margin-left: auto; color: #007acc;",
                        "✓"
                    }
                }
            }
            
            p {
                style: "margin: 0 0 8px 0; color: #cccccc; font-size: 13px;",
                "{description}"
            }
            
            p {
                style: "margin: 0; color: #858585; font-size: 12px;",
                "Use cases: {use_cases}"
            }
        }
    }
}

/// Existing profile option component
#[component]
fn ExistingProfileOption(
    profile: ProfileInfo,
    is_selected: bool,
    on_select: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: if is_selected { "profile-option selected" } else { "profile-option" },
            style: if is_selected { 
                "padding: 15px; background: #2d2d30; border: 2px solid #007acc; border-radius: 8px; cursor: pointer; transition: all 0.2s; margin-bottom: 10px;" 
            } else { 
                "padding: 15px; background: #2d2d30; border: 2px solid #3e3e42; border-radius: 8px; cursor: pointer; transition: all 0.2s; margin-bottom: 10px;" 
            },
            onclick: move |_| on_select.call(()),
            
            div {
                style: "display: flex; align-items: center; justify-content: space-between;",
                h4 {
                    style: "margin: 0; color: #ffffff; font-size: 16px;",
                    "{profile.name}"
                }
                if profile.is_default {
                    span {
                        style: "padding: 2px 8px; background: #007acc; border-radius: 4px; font-size: 11px; color: white;",
                        "DEFAULT"
                    }
                }
                if is_selected {
                    span {
                        style: "color: #007acc;",
                        "✓"
                    }
                }
            }
            
            p {
                style: "margin: 5px 0 0 0; color: #858585; font-size: 12px;",
                "Created: {profile.created_at}"
            }
        }
    }
}

/// About dialog component
#[component]
pub fn AboutDialog(show_about: Signal<bool>) -> Element {
    if !*show_about.read() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| *show_about.write() = false,
            
            div {
                class: "dialog-box about-dialog",
                onclick: move |e| e.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    h2 { "About Hive Consensus" }
                    button {
                        class: "dialog-close",
                        onclick: move |_| *show_about.write() = false,
                        "×"
                    }
                }
                
                div {
                    class: "dialog-content",
                    div { class: "app-icon", "🐝" }
                    h3 { "Hive Consensus" }
                    p { "Version {env!(\"CARGO_PKG_VERSION\")}" }
                    p { "A VS Code-inspired AI development environment" }
                    
                    div {
                        class: "dialog-features",
                        h4 { "Features:" }
                        ul {
                            li { "✓ Multi-model consensus engine" }
                            li { "✓ Real-time code analysis" }
                            li { "✓ Integrated file explorer" }
                            li { "✓ Syntax highlighting" }
                            li { "✓ Git integration" }
                        }
                    }
                    
                    p {
                        class: "dialog-footer-text",
                        "Built with Rust and Dioxus"
                    }
                }
            }
        }
    }
}

/// Welcome dialog/tab component
#[component]
pub fn WelcomeTab(show_welcome: Signal<bool>, on_action: EventHandler<WelcomeAction>) -> Element {
    if !*show_welcome.read() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "welcome-tab",
            
            div {
                class: "welcome-header",
                h1 { "Welcome to Hive Consensus" }
                p { "Get started with your AI-powered development environment" }
            }
            
            div {
                class: "welcome-sections",
                
                div {
                    class: "welcome-section",
                    h3 { "🚀 Quick Start" }
                    button { 
                        class: "welcome-button",
                        onclick: move |_| on_action.call(WelcomeAction::OpenFolder),
                        "Open Folder"
                    }
                    button { 
                        class: "welcome-button",
                        onclick: move |_| on_action.call(WelcomeAction::OpenRecent),
                        "Open Recent"
                    }
                    button { 
                        class: "welcome-button",
                        onclick: move |_| on_action.call(WelcomeAction::NewFile),
                        "New File"
                    }
                }
                
                div {
                    class: "welcome-section",
                    h3 { "💡 Tips" }
                    ul {
                        li { "Use Cmd/Ctrl+Shift+P for Command Palette" }
                        li { "Ask the AI assistant anything about your code" }
                        li { "Click files to view and edit them" }
                    }
                }
                
                div {
                    class: "welcome-section",
                    h3 { "📚 Resources" }
                    a { 
                        href: "#",
                        class: "welcome-link",
                        "Documentation"
                    }
                    a { 
                        href: "#",
                        class: "welcome-link",
                        "Keyboard Shortcuts"
                    }
                    a { 
                        href: "#",
                        class: "welcome-link",
                        "Report Issue"
                    }
                }
            }
            
            button {
                class: "welcome-close",
                onclick: move |_| *show_welcome.write() = false,
                "Close Welcome"
            }
        }
    }
}

/// Command palette component
#[component]
pub fn CommandPalette(show_palette: Signal<bool>) -> Element {
    let mut search_query = use_signal(String::new);
    
    if !*show_palette.read() {
        return rsx! {};
    }

    const COMMANDS: &[(&str, &str)] = &[
        ("Open File", "Ctrl+O"),
        ("Open Folder", "Ctrl+K Ctrl+O"),
        ("Save", "Ctrl+S"),
        ("Save As", "Ctrl+Shift+S"),
        ("Find", "Ctrl+F"),
        ("Replace", "Ctrl+H"),
        ("Toggle Terminal", "Ctrl+`"),
        ("Settings", "Ctrl+,"),
    ];

    let filtered_commands: Vec<_> = COMMANDS.iter()
        .filter(|(cmd, _)| {
            (*search_query.read()).is_empty() || 
            cmd.to_lowercase().contains(&(*search_query.read()).to_lowercase())
        })
        .collect();

    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| *show_palette.write() = false,
            
            div {
                class: "command-palette",
                onclick: move |e| e.stop_propagation(),
                
                input {
                    class: "command-palette-input",
                    placeholder: "Type a command...",
                    value: "{search_query.read()}",
                    oninput: move |evt| *search_query.write() = evt.value().clone(),
                    autofocus: true,
                }
                
                div {
                    class: "command-palette-results",
                    for (cmd, shortcut) in filtered_commands {
                        div {
                            class: "command-palette-item",
                            onclick: move |_| {
                                println!("Execute command: {}", cmd);
                                *show_palette.write() = false;
                            },
                            span { class: "command-name", "{cmd}" }
                            span { class: "command-shortcut", "{shortcut}" }
                        }
                    }
                }
            }
        }
    }
}

/// Settings Dialog Component
#[component]
pub fn SettingsDialog(show_settings: Signal<bool>, openrouter_key: Signal<String>, hive_key: Signal<String>) -> Element {
    let mut is_validating = use_signal(|| false);
    let mut validation_error = use_signal(|| None::<String>);
    let mut profiles = use_signal(|| Vec::<ProfileInfo>::new());
    let mut selected_profile = use_signal(|| String::new());
    let mut profiles_loading = use_signal(|| true);
    
    // Load existing keys and profiles from database on mount
    use_effect(move || {
        // Load OpenRouter key if exists
        if let Ok(Some(key)) = crate::desktop::simple_db::get_config("openrouter_api_key") {
            if !key.is_empty() && openrouter_key.read().is_empty() {
                *openrouter_key.write() = key;
            }
        }
        
        // Load Hive key if exists
        if let Ok(Some(key)) = crate::desktop::simple_db::get_config("hive_license_key") {
            if !key.is_empty() && hive_key.read().is_empty() {
                *hive_key.write() = key;
            }
        }
        
        // Load consensus profiles from database
        spawn(async move {
            match load_existing_profiles().await {
                Ok(loaded_profiles) => {
                    // Find the default profile
                    let default_profile_id = loaded_profiles.iter()
                        .find(|p| p.is_default)
                        .map(|p| p.id.to_string())
                        .or_else(|| loaded_profiles.first().map(|p| p.id.to_string()))
                        .unwrap_or_default();
                    
                    *selected_profile.write() = default_profile_id;
                    *profiles.write() = loaded_profiles;
                    *profiles_loading.write() = false;
                }
                Err(e) => {
                    tracing::error!("Failed to load profiles: {}", e);
                    *profiles_loading.write() = false;
                }
            }
        });
    });
    
    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| *show_settings.write() = false,
            
            div {
                class: "dialog settings-dialog",
                style: "width: 600px; max-height: 80vh; overflow-y: auto;",
                onclick: move |evt| evt.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    h2 { "⚙️ Settings" }
                    button {
                        class: "dialog-close",
                        onclick: move |_| *show_settings.write() = false,
                        "×"
                    }
                }
                
                div {
                    class: "dialog-content settings-content",
                    
                    // API Keys Section
                    div {
                        class: "settings-section",
                        h3 { "🔑 API Keys" }
                        p { 
                            class: "settings-description",
                            "Configure your API keys to enable Hive Consensus functionality." 
                        }
                        
                        // OpenRouter API Key
                        div {
                            class: "settings-field",
                            label { 
                                class: "settings-label",
                                "OpenRouter API Key" 
                            }
                            input {
                                class: "settings-input",
                                r#type: "password",
                                value: "{openrouter_key.read()}",
                                placeholder: "sk-or-v1-...",
                                oninput: move |evt| *openrouter_key.write() = evt.value().clone(),
                            }
                            p { 
                                class: "settings-hint",
                                "Get your API key from ",
                                a {
                                    href: "#",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        spawn(async {
                                            let _ = webbrowser::open("https://openrouter.ai/keys");
                                        });
                                    },
                                    "openrouter.ai/keys"
                                }
                            }
                        }
                        
                        // Hive API Key
                        div {
                            class: "settings-field",
                            label { 
                                class: "settings-label",
                                "Hive License Key" 
                            }
                            input {
                                class: "settings-input",
                                r#type: "password",
                                value: "{hive_key.read()}",
                                placeholder: "hive-...",
                                oninput: move |evt| *hive_key.write() = evt.value().clone(),
                            }
                            p { 
                                class: "settings-hint",
                                "Used for syncing conversations and advanced features" 
                            }
                        }
                        
                        // Show validation error if any
                        if let Some(error) = validation_error.read().as_ref() {
                            div {
                                style: "margin-top: 10px; padding: 10px; background: #5a1e1e; border: 1px solid #8b3a3a; border-radius: 4px; color: #ff6b6b;",
                                "❌ {error}"
                            }
                        }
                    }
                    
                    // Consensus Profile Section
                    div {
                        class: "settings-section",
                        h3 { "🧠 Consensus Profile" }
                        p { 
                            class: "settings-description",
                            "Choose your consensus processing profile based on your needs." 
                        }
                        
                        if *profiles_loading.read() {
                            div {
                                class: "loading-container",
                                style: "text-align: center; padding: 20px;",
                                "Loading profiles..."
                            }
                        } else if profiles.read().is_empty() {
                            div {
                                class: "empty-state",
                                style: "text-align: center; padding: 20px; color: #888;",
                                "No profiles found. Please complete onboarding to create expert profiles."
                            }
                        } else {
                            div {
                                class: "profile-grid",
                                for profile in profiles.read().iter() {
                                    DatabaseProfileOption { 
                                        profile_id: profile.id.to_string(),
                                        name: profile.name.clone(),
                                        is_selected: *selected_profile.read() == profile.id.to_string(),
                                        is_default: profile.is_default,
                                        on_select: move |id: String| {
                                            *selected_profile.write() = id;
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
                
                div {
                    class: "dialog-footer",
                    button {
                        class: "button button-secondary",
                        onclick: move |_| *show_settings.write() = false,
                        "Cancel"
                    }
                    button {
                        class: "button button-primary",
                        disabled: *is_validating.read(),
                        onclick: move |_| {
                            // Clear previous error
                            *validation_error.write() = None;
                            *is_validating.write() = true;
                            
                            // Save settings with validation
                            let openrouter = openrouter_key.read().clone();
                            let hive = hive_key.read().clone();
                            let mut is_validating = is_validating.clone();
                            let mut validation_error = validation_error.clone();
                            let mut show_settings = show_settings.clone();
                            
                            let selected_profile_id = selected_profile.read().clone();
                            
                            spawn(async move {
                                // Simple synchronous saves
                                let mut success = true;
                                let mut error_msg = String::new();
                                
                                // Save OpenRouter key
                                if !openrouter.is_empty() {
                                    if let Err(e) = crate::desktop::simple_db::save_config("openrouter_api_key", &openrouter) {
                                        success = false;
                                        error_msg = format!("Failed to save OpenRouter key: {}", e);
                                    }
                                }
                                
                                // Save Hive key
                                if !hive.is_empty() {
                                    if let Err(e) = crate::desktop::simple_db::save_config("hive_license_key", &hive) {
                                        success = false;
                                        error_msg = format!("Failed to save Hive key: {}", e);
                                    }
                                }
                                
                                // Update selected profile as default if changed
                                if !selected_profile_id.is_empty() {
                                    if let Err(e) = update_default_profile(&selected_profile_id).await {
                                        tracing::error!("Failed to update default profile: {}", e);
                                    }
                                }
                                
                                if success {
                                    // Success - close dialog
                                    *show_settings.write() = false;
                                } else {
                                    // Show error
                                    *validation_error.write() = Some(error_msg);
                                }
                                
                                *is_validating.write() = false;
                            });
                        },
                        if *is_validating.read() { "Validating..." } else { "Save Settings" }
                    }
                }
            }
        }
    }
}

/// Profile info for display
#[derive(Debug, Clone)]
struct ProfileDisplayInfo {
    id: i64,
    name: String,
    is_default: bool,
    generator_model: String,
    refiner_model: String,
    validator_model: String,
    curator_model: String,
}

#[component]
fn ProfileOption(name: &'static str, description: &'static str, models: &'static str, is_selected: bool) -> Element {
    rsx! {
        div {
            class: if is_selected { "profile-option selected" } else { "profile-option" },
            h4 { "{name}" }
            p { class: "profile-description", "{description}" }
            p { class: "profile-models", "{models}" }
        }
    }
}

#[component]
fn DatabaseProfileOption(
    profile_id: String,
    name: String,
    is_selected: bool,
    is_default: bool,
    on_select: EventHandler<String>
) -> Element {
    rsx! {
        div {
            class: if is_selected { "profile-option selected" } else { "profile-option" },
            onclick: move |_| on_select.call(profile_id.clone()),
            style: "cursor: pointer;",
            
            h4 { 
                "{name}"
                if is_default {
                    span { 
                        style: "font-size: 12px; margin-left: 8px; padding: 2px 6px; background: #4a5568; border-radius: 4px;",
                        "Default" 
                    }
                }
            }
            p { 
                class: "profile-description", 
                style: "font-size: 12px; color: #888;",
                "Expert consensus profile"
            }
        }
    }
}

/// Update the default profile in the database
async fn update_default_profile(profile_id: &str) -> anyhow::Result<()> {
    use crate::core::database::DatabaseManager;
    use crate::core::config::get_hive_config_dir;
    
    let db_path = get_hive_config_dir().join("hive-ai.db");
    if !db_path.exists() {
        return Err(anyhow::anyhow!("Database not found"));
    }
    
    let db_config = crate::core::database::DatabaseConfig {
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
    
    let db = DatabaseManager::new(db_config).await?;
    let mut conn = db.get_connection()?;
    let tx = conn.transaction()?;
    
    // First, unset all profiles as default
    tx.execute("UPDATE consensus_profiles SET is_default = 0", [])?;
    
    // Then set the selected profile as default
    tx.execute(
        "UPDATE consensus_profiles SET is_default = 1 WHERE id = ?1",
        rusqlite::params![profile_id]
    )?;
    
    tx.commit()?;
    tracing::info!("Updated default profile to: {}", profile_id);
    
    Ok(())
}

/// Onboarding Dialog for first-time users
#[component]
pub fn OnboardingDialog(
    mut show_onboarding: Signal<bool>, 
    mut openrouter_key: Signal<String>, 
    mut hive_key: Signal<String>,
    mut current_step: Signal<i32>
) -> Element {
    let mut is_validating = use_signal(|| false);
    let mut validation_error = use_signal(|| None::<String>);
    // Load the current default profile name from database
    let mut selected_profile = use_signal(|| String::new());
    let mut default_profile_id = use_signal(|| None::<i64>);
    // Initialize temp keys with existing values from database if available
    let mut temp_openrouter_key = use_signal(|| {
        if let Ok(Some(key)) = crate::desktop::simple_db::get_config("openrouter_api_key") {
            key
        } else {
            openrouter_key.read().clone()
        }
    });
    let mut temp_hive_key = use_signal(|| {
        if let Ok(Some(key)) = crate::desktop::simple_db::get_config("hive_license_key") {
            key
        } else {
            hive_key.read().clone()
        }
    });
    
    // Track if keys already exist in database
    let has_existing_openrouter = crate::desktop::simple_db::has_openrouter_key();
    let has_existing_hive = crate::desktop::simple_db::has_hive_key();
    
    // Profile configuration state
    let mut profile_mode = use_signal(|| "expert".to_string()); // expert, existing, custom
    let mut selected_template = use_signal(|| String::new());
    let mut profile_name = use_signal(|| String::new());
    let mut selected_profile_id = use_signal(|| None::<i64>);
    let mut existing_profiles = use_signal(|| Vec::<ProfileInfo>::new());
    let mut is_creating_profile = use_signal(|| false);
    let mut profile_error = use_signal(|| None::<String>);
    let mut profiles_created = use_signal(|| Vec::<String>::new());
    let mut show_profile_success = use_signal(|| false);
    let mut continue_creating_profiles = use_signal(|| false);
    let mut is_closing = use_signal(|| false);
    
    // License validation result
    let mut license_info = use_signal(|| None::<LicenseValidationResult>);
    
    // Load existing profiles on mount
    use_effect(move || {
        let mut existing_profiles = existing_profiles.clone();
        spawn(async move {
            if let Ok(profiles) = load_existing_profiles().await {
                *existing_profiles.write() = profiles;
            }
        });
    });
    
    // Load existing keys from database on mount
    use_effect(move || {
        // Load OpenRouter key if exists
        if let Ok(Some(key)) = crate::desktop::simple_db::get_config("openrouter_api_key") {
            if !key.is_empty() {
                *openrouter_key.write() = key.clone();
                *temp_openrouter_key.write() = key;
            }
        }
        
        // Load Hive key if exists
        if let Ok(Some(key)) = crate::desktop::simple_db::get_config("hive_license_key") {
            if !key.is_empty() {
                *hive_key.write() = key.clone();
                *temp_hive_key.write() = key;
            }
        }
    });
    
    // Handle closing state changes
    use_effect(move || {
        if *is_closing.read() {
            // When closing flag is set, actually close the dialog
            let mut show_onboarding_clone = show_onboarding.clone();
            spawn(async move {
                // Small delay to prevent re-render issues
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                show_onboarding_clone.set(false);
            });
        }
    });
    
    // Check onboarding state ONLY on initial mount to determine starting step
    use_effect(move || {
        let openrouter_key = openrouter_key.clone();
        let mut current_step = current_step.clone();
        let mut existing_profiles = existing_profiles.clone();
        
        // Only run this check once on mount
        spawn(async move {
            // If we have keys, check if profiles exist to determine starting step
            if !openrouter_key.read().is_empty() {
                // Load profiles to check if any exist
                if let Ok(profiles) = load_existing_profiles().await {
                    *existing_profiles.write() = profiles.clone();
                    
                    if profiles.is_empty() {
                        // We have keys but no profiles, go directly to profile step
                        tracing::info!("Keys exist but no profiles - starting at profile configuration step");
                        *current_step.write() = 4;
                    }
                    // Note: We don't close the dialog here even if profiles exist
                    // Let the user complete the flow or close it manually
                }
            }
        });
    });
    
    // Load the default profile name from database
    use_effect(move || {
        let mut selected_profile = selected_profile.clone();
        let mut default_profile_id = default_profile_id.clone();
        
        spawn(async move {
            // Load the current default profile from database
            match load_default_profile().await {
                Ok(Some((id, name))) => {
                    tracing::info!("Loaded default profile: {} (id: {})", name, id);
                    *selected_profile.write() = name;
                    *default_profile_id.write() = Some(id);
                }
                Ok(None) => {
                    tracing::info!("No default profile set");
                }
                Err(e) => {
                    tracing::error!("Failed to load default profile: {}", e);
                }
            }
        });
    });
    
    // Early return if dialog should not be shown or is closing
    let show = *show_onboarding.read();
    let closing = *is_closing.read();
    if !show || closing {
        tracing::info!("OnboardingDialog render check: show = {}, closing = {}, returning empty", show, closing);
        return rsx! {};
    }
    tracing::info!("OnboardingDialog render check: show = true, rendering dialog");
    
    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| {
                // Allow closing by clicking overlay
                tracing::info!("Overlay clicked - closing onboarding dialog");
                
                // Just set closing flag - useEffect will handle the actual close
                *is_closing.write() = true;
            },
            
            div {
                class: "dialog onboarding-dialog",
                style: "width: 700px;",
                onclick: move |evt| evt.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    h2 { "🐝 Welcome to Hive Consensus" }
                    button {
                        style: "background: none; border: none; color: #cccccc; font-size: 20px; cursor: pointer; padding: 0; width: 30px; height: 30px;",
                        onclick: move |_| {
                            tracing::info!("Close button clicked - closing dialog");
                            
                            // Just set closing flag - useEffect will handle the actual close
                            *is_closing.write() = true;
                        },
                        "×"
                    }
                }
                
                // Progress indicator
                div {
                    class: "onboarding-progress",
                    style: "display: flex; justify-content: center; padding: 20px 0;",
                    div {
                        class: if *current_step.read() >= 1 { "progress-step active" } else { "progress-step" },
                        style: "margin: 0 10px;",
                        "1. Welcome"
                    }
                    div { class: "progress-separator", "→" }
                    div {
                        class: if *current_step.read() >= 2 { "progress-step active" } else { "progress-step" },
                        style: "margin: 0 10px;",
                        "2. License Key"
                    }
                    div { class: "progress-separator", "→" }
                    div {
                        class: if *current_step.read() >= 3 { "progress-step active" } else { "progress-step" },
                        style: "margin: 0 10px;",
                        "3. OpenRouter Key"
                    }
                    div { class: "progress-separator", "→" }
                    div {
                        class: if *current_step.read() >= 4 { "progress-step active" } else { "progress-step" },
                        style: "margin: 0 10px;",
                        "4. Profile"
                    }
                    div { class: "progress-separator", "→" }
                    div {
                        class: if *current_step.read() >= 5 { "progress-step active" } else { "progress-step" },
                        style: "margin: 0 10px;",
                        "5. Complete"
                    }
                }
                
                div {
                    class: "dialog-content onboarding-content",
                    
                    if *current_step.read() == 1 {
                        div {
                            class: "onboarding-step",
                            h3 { "Let's get you started!" }
                            p { 
                                "Hive Consensus uses advanced AI models to provide the best possible responses through a 4-stage consensus process." 
                            }
                            
                            div {
                                class: "features-list",
                                div { class: "feature-item", "✅ 4-stage consensus pipeline" }
                                div { class: "feature-item", "✅ 323+ AI models available" }
                                div { class: "feature-item", "✅ VS Code-like interface" }
                                div { class: "feature-item", "✅ Full file system access" }
                            }
                            
                            p {
                                style: "margin-top: 20px; color: #cccccc;",
                                "To get started, you'll need to configure your license key and API access."
                            }
                        }
                    } else if *current_step.read() == 2 {
                        div {
                            class: "onboarding-step",
                            h3 { "🏷️ Configure Your Hive License" }
                            p { 
                                "Enter your Hive license key to unlock all features and enable cloud sync." 
                            }
                            
                            // Show existing key message if applicable
                            if has_existing_hive {
                                div {
                                    style: "margin: 10px 0; padding: 10px; background: #2a3f2a; border: 1px solid #3a5f3a; border-radius: 4px; color: #90ee90;",
                                    "✅ A Hive license key is already configured. Enter a new key to update it or click Skip to keep the current one."
                                }
                            }
                            
                            // Show current license info if saved
                            if let Some(license) = license_info.read().as_ref() {
                                div {
                                    style: "margin: 10px 0; padding: 10px; background: #1e1e1e; border: 1px solid #3e3e42; border-radius: 4px;",
                                    p {
                                        style: "margin: 0 0 5px 0; color: #4ade80; font-weight: 600;",
                                        "Current License:"
                                    }
                                    div {
                                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px; font-size: 13px;",
                                        div {
                                            "🎯 Tier: ",
                                            span { 
                                                style: "font-weight: 600; color: #4ade80;",
                                                "{license.tier}"
                                            }
                                        }
                                        div {
                                            "💬 Daily Limit: ",
                                            span { 
                                                style: "font-weight: 600; color: #4ade80;",
                                                "{license.daily_limit}"
                                            }
                                        }
                                    }
                                }
                            }
                            
                            div {
                                class: "settings-field",
                                label { 
                                    class: "settings-label",
                                    "Hive License Key" 
                                }
                                input {
                                    class: "settings-input",
                                    r#type: "password",
                                    value: "{temp_hive_key.read()}",
                                    placeholder: "hive-xxxx-xxxx-xxxx",
                                    oninput: move |evt| {
                                        let value = evt.value().clone();
                                        *temp_hive_key.write() = value;
                                    },
                                    onchange: move |evt| {
                                        let value = evt.value().clone();
                                        *temp_hive_key.write() = value;
                                    },
                                }
                                
                                div {
                                    class: "api-key-help",
                                    p { 
                                        class: "settings-hint",
                                        "Your license key is required for consensus to work properly. It enables conversation sync and premium features." 
                                    }
                                    p {
                                        style: "margin-top: 10px;",
                                        "Don't have a license? ",
                                        a {
                                            href: "#",
                                            onclick: move |evt| {
                                                evt.stop_propagation();
                                                spawn(async {
                                                    let _ = webbrowser::open("https://hivetechs.io/purchase");
                                                });
                                            },
                                            "Get one from HiveTechs"
                                        }
                                    }
                                }
                            }
                            
                            // Show validation error if any
                            if let Some(error) = validation_error.read().as_ref() {
                                div {
                                    style: "margin-top: 10px; padding: 10px; background: #5a1e1e; border: 1px solid #8b3a3a; border-radius: 4px; color: #ff6b6b;",
                                    "❌ {error}"
                                }
                            }
                        }
                    } else if *current_step.read() == 3 {
                        div {
                            class: "onboarding-step",
                            h3 { "🔗 Configure Your OpenRouter API Key" }
                            p { 
                                "To use Hive Consensus, you'll need an OpenRouter API key. This gives you access to 323+ models from various providers." 
                            }
                            
                            // Show existing key message if applicable
                            if has_existing_openrouter {
                                div {
                                    style: "margin: 10px 0; padding: 10px; background: #2a3f2a; border: 1px solid #3a5f3a; border-radius: 4px; color: #90ee90;",
                                    "✅ An OpenRouter API key already exists. Enter a new key to update it."
                                }
                            }
                            
                            // Show current key status
                            if !openrouter_key.read().is_empty() {
                                div {
                                    style: "margin: 10px 0; padding: 8px; background: #1e1e1e; border-radius: 4px; font-size: 13px; color: #858585;",
                                    {
                                        let key = openrouter_key.read();
                                        let suffix = if key.len() > 4 {
                                            key.chars().rev().take(4).collect::<String>().chars().rev().collect::<String>()
                                        } else {
                                            "****".to_string()
                                        };
                                        format!("Current key: sk-or-v1-****{}", suffix)
                                    }
                                }
                            }
                            
                            div {
                                class: "settings-field",
                                label { 
                                    class: "settings-label",
                                    "OpenRouter API Key" 
                                }
                                input {
                                    class: "settings-input",
                                    r#type: "password",
                                    value: "{temp_openrouter_key.read()}",
                                    placeholder: "sk-or-v1-...",
                                    oninput: move |evt| *temp_openrouter_key.write() = evt.value().clone(),
                                }
                                
                                div {
                                    class: "api-key-help",
                                    p { 
                                        "Don't have an API key? ",
                                        a {
                                            href: "#",
                                            onclick: move |evt| {
                                                evt.stop_propagation();
                                                spawn(async {
                                                    let _ = webbrowser::open("https://openrouter.ai/keys");
                                                });
                                            },
                                            "Get one from OpenRouter"
                                        }
                                    }
                                    p { 
                                        class: "settings-hint",
                                        "OpenRouter provides unified access to models from OpenAI, Anthropic, Google, Meta, and more." 
                                    }
                                }
                            }
                            
                            // Show validation error if any
                            if let Some(error) = validation_error.read().as_ref() {
                                div {
                                    style: "margin-top: 10px; padding: 10px; background: #5a1e1e; border: 1px solid #8b3a3a; border-radius: 4px; color: #ff6b6b;",
                                    "❌ {error}"
                                }
                            }
                        }
                    } else if *current_step.read() == 4 {
                        div {
                            class: "onboarding-step",
                            h3 { "🧠 Configure Your Consensus Profile" }
                            p { 
                                "Choose from expert-crafted profiles or create your own. Each profile uses a 4-stage AI consensus pipeline." 
                            }
                            
                            // Show license info if available
                            if let Some(license) = license_info.read().as_ref() {
                                div {
                                    style: "margin: 15px 0; padding: 15px; background: #1e3a2e; border: 1px solid #2e5a3e; border-radius: 6px;",
                                    div {
                                        style: "display: flex; align-items: center; gap: 10px; margin-bottom: 8px;",
                                        span { 
                                            style: "font-size: 18px;",
                                            "✅" 
                                        }
                                        span {
                                            style: "color: #4ade80; font-weight: 600;",
                                            "License Validated"
                                        }
                                    }
                                    div {
                                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px; color: #cccccc; font-size: 13px;",
                                        div {
                                            "🎯 Tier: ",
                                            span { 
                                                style: "font-weight: 600; color: #4ade80;",
                                                "{license.tier}"
                                            }
                                        }
                                        div {
                                            "💬 Daily Conversations: ",
                                            span { 
                                                style: "font-weight: 600; color: #4ade80;",
                                                "{license.daily_limit}"
                                            }
                                        }
                                        if let Some(email) = &license.email {
                                            div {
                                                style: "grid-column: span 2;",
                                                "📧 Account: ",
                                                span { 
                                                    style: "color: #858585;",
                                                    "{email}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Profile selection mode tabs
                            div {
                                class: "profile-tabs",
                                style: "display: flex; gap: 10px; margin: 20px 0; border-bottom: 1px solid #3e3e42; padding-bottom: 10px;",
                                button {
                                    class: if *profile_mode.read() == "expert" { "tab-button active" } else { "tab-button" },
                                    style: "padding: 8px 16px; background: transparent; border: none; color: #cccccc; cursor: pointer; border-bottom: 2px solid transparent;",
                                    onclick: move |_| *profile_mode.write() = "expert".to_string(),
                                    "🎯 Expert Templates"
                                }
                                button {
                                    class: if *profile_mode.read() == "existing" { "tab-button active" } else { "tab-button" },
                                    style: "padding: 8px 16px; background: transparent; border: none; color: #cccccc; cursor: pointer; border-bottom: 2px solid transparent;",
                                    onclick: move |_| {
                                        *profile_mode.write() = "existing".to_string();
                                        
                                        // Load existing profiles when tab is clicked
                                        let mut existing_profiles = existing_profiles.clone();
                                        spawn(async move {
                                            if let Ok(profiles) = load_existing_profiles().await {
                                                *existing_profiles.write() = profiles;
                                            }
                                        });
                                    },
                                    "📋 Existing Profiles"
                                }
                                button {
                                    class: if *profile_mode.read() == "custom" { "tab-button active" } else { "tab-button" },
                                    style: "padding: 8px 16px; background: transparent; border: none; color: #cccccc; cursor: pointer; border-bottom: 2px solid transparent;",
                                    onclick: move |_| *profile_mode.write() = "custom".to_string(),
                                    "🛠️ Custom Profile"
                                }
                            }
                            
                            // Profile content based on selected mode
                            div {
                                class: "profile-content",
                                style: "max-height: 400px; overflow-y: auto; padding: 10px 0;",
                                
                                if *profile_mode.read() == "expert" {
                                    div {
                                        p {
                                            style: "margin-bottom: 15px; color: #cccccc;",
                                            "Select an expert template optimized for specific use cases:"
                                        }
                                        
                                        // Show profile creation options prominently at the top
                                        div {
                                            style: "margin-bottom: 20px; padding: 20px; background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%); border: 2px solid #48bb78; border-radius: 12px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);",
                                            h4 {
                                                style: "margin: 0 0 10px 0; color: #ffffff; font-size: 18px;",
                                                "🌟 Quick Actions"
                                            }
                                            p {
                                                style: "margin: 0 0 15px 0; color: #a0aec0; font-size: 14px;",
                                                "Get started quickly by adding all expert-crafted profiles at once!"
                                            }
                                            div {
                                                style: "display: flex; gap: 10px; flex-wrap: wrap;",
                                                button {
                                                    class: "dialog-button",
                                                    style: "background: #48bb78; color: white; padding: 12px 24px; font-weight: 600; font-size: 16px; border-radius: 6px; box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2); transition: all 0.2s;",
                                                    disabled: *is_creating_profile.read(),
                                                    onclick: move |_| {
                                                        tracing::info!("Adding all expert templates");
                                                        let mut is_creating_profile = is_creating_profile.clone();
                                                        let mut profile_error = profile_error.clone();
                                                        let mut profiles_created = profiles_created.clone();
                                                        let mut show_profile_success = show_profile_success.clone();
                                                        let mut existing_profiles = existing_profiles.clone();
                                                        
                                                        spawn(async move {
                                                            *is_creating_profile.write() = true;
                                                            *profile_error.write() = None;
                                                            
                                                            let templates = vec![
                                                                ("lightning-fast", "Lightning Fast"),
                                                                ("precision-architect", "Precision Architect"),
                                                                ("budget-optimizer", "Budget Optimizer"),
                                                                ("research-deep-dive", "Research Deep Dive"),
                                                                ("startup-mvp", "Startup MVP"),
                                                                ("enterprise-grade", "Enterprise Grade"),
                                                                ("creative-innovator", "Creative Innovator"),
                                                                ("security-focused", "Security Focused"),
                                                                ("ml-ai-specialist", "ML/AI Specialist"),
                                                                ("debugging-detective", "Debugging Detective")
                                                            ];
                                                            
                                                            let mut created = Vec::new();
                                                            let total = templates.len();
                                                            
                                                            for (index, (template_id, name)) in templates.into_iter().enumerate() {
                                                                // Update progress
                                                                let progress = index + 1;
                                                                *profiles_created.write() = created.clone();
                                                                
                                                                match create_profile_from_template(template_id, name).await {
                                                                    Ok(_) => {
                                                                        created.push(name.to_string());
                                                                        tracing::info!("Created profile {}/{}: {}", progress, total, name);
                                                                    }
                                                                    Err(e) => {
                                                                        tracing::warn!("Failed to create profile {}: {}", name, e);
                                                                    }
                                                                }
                                                                
                                                                // Small delay to show progress
                                                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                                            }
                                                            
                                                            *profiles_created.write() = created;
                                                            *show_profile_success.write() = true;
                                                            *is_creating_profile.write() = false;
                                                            
                                                            // Reload profiles
                                                            if let Ok(profiles) = load_existing_profiles().await {
                                                                *existing_profiles.write() = profiles;
                                                            }
                                                        });
                                                    },
                                                    {
                                                        if *is_creating_profile.read() { 
                                                            format!("⏳ Creating profiles... ({}/10)", profiles_created.read().len())
                                                        } else { 
                                                            "🚀 Add All 10 Expert Templates".to_string() 
                                                        }
                                                    }
                                                }
                                                button {
                                                    class: "dialog-button",
                                                    style: "background: #4299e1; color: white; padding: 12px 24px; border-radius: 6px;",
                                                    onclick: move |_| {
                                                        tracing::info!("Moving to completion without creating profiles");
                                                        *current_step.write() = 5;
                                                    },
                                                    "➡️ Continue Without Profiles"
                                                }
                                            }
                                            if *is_creating_profile.read() {
                                                div {
                                                    style: "margin-top: 15px; padding: 10px; background: #1a202c; border-radius: 6px;",
                                                    p {
                                                        style: "margin: 0; color: #48bb78; font-size: 14px;",
                                                        "🔄 Creating {profiles_created.read().len()}/10 profiles..."
                                                    }
                                                    div {
                                                        style: "margin-top: 8px; height: 8px; background: #2d3748; border-radius: 4px; overflow: hidden;",
                                                        div {
                                                            style: {
                                                                let width = (profiles_created.read().len() as f32 / 10.0 * 100.0) as u32;
                                                                format!("height: 100%; background: #48bb78; width: {}%; transition: width 0.3s;", width)
                                                            },
                                                        }
                                                    }
                                                }
                                            }
                                            if !profiles_created.read().is_empty() && !*is_creating_profile.read() {
                                                p {
                                                    style: "margin: 15px 0 0 0; color: #90ee90; font-size: 14px; font-weight: 600;",
                                                    "✅ Successfully created {profiles_created.read().len()} profile(s)!"
                                                }
                                            }
                                        }
                                        
                                        // Expert templates grid
                                        div {
                                            class: "expert-templates-grid",
                                            style: "display: grid; grid-template-columns: repeat(2, 1fr); gap: 15px;",
                                            
                                            // Lightning Fast
                                            ExpertTemplateOption {
                                                id: "lightning-fast",
                                                name: "⚡ Lightning Fast",
                                                description: "Ultra-high-speed consensus for rapid prototyping",
                                                use_cases: "Quick prototyping, Simple questions, Learning",
                                                is_selected: *selected_template.read() == "lightning-fast",
                                                on_select: move |_| *selected_template.write() = "lightning-fast".to_string(),
                                            }
                                            
                                            // Precision Architect
                                            ExpertTemplateOption {
                                                id: "precision-architect",
                                                name: "🏗️ Precision Architect",
                                                description: "Maximum quality for complex architectural decisions",
                                                use_cases: "Architecture, Complex algorithms, Production review",
                                                is_selected: *selected_template.read() == "precision-architect",
                                                on_select: move |_| *selected_template.write() = "precision-architect".to_string(),
                                            }
                                            
                                            // Budget Optimizer
                                            ExpertTemplateOption {
                                                id: "budget-optimizer",
                                                name: "💰 Budget Optimizer",
                                                description: "Cost-efficient consensus maximizing value",
                                                use_cases: "Cost-conscious dev, High-volume, Experimentation",
                                                is_selected: *selected_template.read() == "budget-optimizer",
                                                on_select: move |_| *selected_template.write() = "budget-optimizer".to_string(),
                                            }
                                            
                                            // Research Deep Dive
                                            ExpertTemplateOption {
                                                id: "research-deep-dive",
                                                name: "🔬 Research Deep Dive",
                                                description: "Comprehensive analysis for research and knowledge discovery",
                                                use_cases: "Technical research, Market analysis, Documentation",
                                                is_selected: *selected_template.read() == "research-deep-dive",
                                                on_select: move |_| *selected_template.write() = "research-deep-dive".to_string(),
                                            }
                                            
                                            // Startup MVP
                                            ExpertTemplateOption {
                                                id: "startup-mvp",
                                                name: "🚀 Startup MVP",
                                                description: "Balanced consensus for MVP development",
                                                use_cases: "MVP development, Startup projects, Feature prototyping",
                                                is_selected: *selected_template.read() == "startup-mvp",
                                                on_select: move |_| *selected_template.write() = "startup-mvp".to_string(),
                                            }
                                            
                                            // Enterprise Grade
                                            ExpertTemplateOption {
                                                id: "enterprise-grade",
                                                name: "🏢 Enterprise Grade",
                                                description: "Production-ready with enterprise security and reliability",
                                                use_cases: "Enterprise applications, Mission-critical systems, Financial services",
                                                is_selected: *selected_template.read() == "enterprise-grade",
                                                on_select: move |_| *selected_template.write() = "enterprise-grade".to_string(),
                                            }
                                            
                                            // Security Focused
                                            ExpertTemplateOption {
                                                id: "security-focused",
                                                name: "🔐 Security Focused",
                                                description: "Security-first for secure coding and vulnerability analysis",
                                                use_cases: "Security audits, Vulnerability analysis, Compliance reviews",
                                                is_selected: *selected_template.read() == "security-focused",
                                                on_select: move |_| *selected_template.write() = "security-focused".to_string(),
                                            }
                                            
                                            // Creative Innovator
                                            ExpertTemplateOption {
                                                id: "creative-innovator",
                                                name: "🎨 Creative Innovator",
                                                description: "High-creativity for innovative solutions and creative problem solving",
                                                use_cases: "Creative coding, Innovative solutions, Brainstorming",
                                                is_selected: *selected_template.read() == "creative-innovator",
                                                on_select: move |_| *selected_template.write() = "creative-innovator".to_string(),
                                            }
                                            
                                            // ML/AI Specialist
                                            ExpertTemplateOption {
                                                id: "ml-ai-specialist",
                                                name: "🤖 ML/AI Specialist",
                                                description: "Specialized for machine learning and AI development",
                                                use_cases: "ML model development, AI system design, Data science",
                                                is_selected: *selected_template.read() == "ml-ai-specialist",
                                                on_select: move |_| *selected_template.write() = "ml-ai-specialist".to_string(),
                                            }
                                            
                                            // Debugging Detective
                                            ExpertTemplateOption {
                                                id: "debugging-detective",
                                                name: "🔍 Debugging Detective",
                                                description: "Methodical consensus for debugging and troubleshooting",
                                                use_cases: "Bug hunting, Error analysis, Performance issues",
                                                is_selected: *selected_template.read() == "debugging-detective",
                                                on_select: move |_| *selected_template.write() = "debugging-detective".to_string(),
                                            }
                                        }
                                        
                                        // Profile name input for template
                                        if !selected_template.read().is_empty() {
                                            div {
                                                style: "margin-top: 20px; padding: 15px; background: #2d2d30; border-radius: 6px;",
                                                label {
                                                    style: "display: block; margin-bottom: 8px; color: #cccccc;",
                                                    "Profile Name:"
                                                }
                                                input {
                                                    class: "settings-input",
                                                    r#type: "text",
                                                    value: "{profile_name.read()}",
                                                    placeholder: "Enter a name for your profile",
                                                    oninput: move |evt| *profile_name.write() = evt.value().clone(),
                                                }
                                                p {
                                                    style: "margin-top: 5px; font-size: 12px; color: #858585;",
                                                    "Give your profile a memorable name (e.g., 'My Production Config')"
                                                }
                                                
                                                // Create profile button
                                                button {
                                                    class: "button button-primary",
                                                    style: "margin-top: 15px; width: 100%;",
                                                    disabled: *is_creating_profile.read() || profile_name.read().is_empty(),
                                                    onclick: move |_| {
                                                        let template_id = selected_template.read().clone();
                                                        let profile_name_val = profile_name.read().clone();
                                                        
                                                        if !template_id.is_empty() && !profile_name_val.is_empty() {
                                                            tracing::info!("Creating profile from template: {} as {}", template_id, profile_name_val);
                                                            let name_for_spawn = profile_name_val.clone();
                                                            let template_for_spawn = template_id.clone();
                                                            let mut is_creating_profile = is_creating_profile.clone();
                                                            let mut profile_error = profile_error.clone();
                                                            let mut profiles_created = profiles_created.clone();
                                                            let mut show_profile_success = show_profile_success.clone();
                                                            let mut existing_profiles = existing_profiles.clone();
                                                            let mut selected_profile = selected_profile.clone();
                                                            
                                                            spawn(async move {
                                                                *is_creating_profile.write() = true;
                                                                *profile_error.write() = None;
                                                                
                                                                match create_profile_from_template(&template_for_spawn, &name_for_spawn).await {
                                                                    Ok(_) => {
                                                                        tracing::info!("Profile created successfully: {}", name_for_spawn);
                                                                        let mut created = profiles_created.read().clone();
                                                                        created.push(name_for_spawn.clone());
                                                                        *profiles_created.write() = created;
                                                                        *show_profile_success.write() = true;
                                                                        *selected_profile.write() = name_for_spawn;
                                                                        
                                                                        // Reload profiles
                                                                        if let Ok(profiles) = load_existing_profiles().await {
                                                                            *existing_profiles.write() = profiles;
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        tracing::error!("Failed to create profile: {}", e);
                                                                        *profile_error.write() = Some(e.to_string());
                                                                    }
                                                                }
                                                                
                                                                *is_creating_profile.write() = false;
                                                            });
                                                        }
                                                    },
                                                    if *is_creating_profile.read() { "Creating Profile..." } else { "Create Profile" }
                                                }
                                            }
                                        }
                                    }
                                } else if *profile_mode.read() == "existing" {
                                    div {
                                        p {
                                            style: "margin-bottom: 15px; color: #cccccc;",
                                            if existing_profiles.read().is_empty() {
                                                "No existing profiles found. Create one from expert templates or build a custom profile."
                                            } else {
                                                "Select from your existing profiles:"
                                            }
                                        }
                                        
                                        // Load and display existing profiles
                                        if !existing_profiles.read().is_empty() {
                                            div {
                                                class: "existing-profiles-list",
                                                {
                                                    let profiles = existing_profiles.read().clone();
                                                    profiles.into_iter().map(|profile| {
                                                        let profile_id = profile.id;
                                                        let is_selected = *selected_profile_id.read() == Some(profile_id);
                                                        let mut selected_profile_id = selected_profile_id.clone();
                                                        
                                                        rsx! {
                                                            ExistingProfileOption {
                                                                profile: profile,
                                                                is_selected: is_selected,
                                                                on_select: move |_| *selected_profile_id.write() = Some(profile_id),
                                                            }
                                                        }
                                                    })
                                                }
                                            }
                                        }
                                    }
                                } else if *profile_mode.read() == "custom" {
                                    div {
                                        p {
                                            style: "margin-bottom: 15px; color: #cccccc;",
                                            "Build a custom 4-stage consensus pipeline by selecting models for each stage:"
                                        }
                                        p {
                                            style: "margin-bottom: 20px; padding: 15px; background: #1e1e1e; border-radius: 6px; color: #e9c46a;",
                                            "⚠️ Custom profile creation is an advanced feature. For best results, use expert templates."
                                        }
                                        
                                        // TODO: Implement custom model selection UI
                                        // This would involve loading available models and allowing selection for each stage
                                        div {
                                            style: "text-align: center; padding: 40px; color: #858585;",
                                            "Custom profile builder coming soon..."
                                        }
                                    }
                                }
                            }
                        }
                    } else if *current_step.read() == 5 {
                        div {
                            class: "onboarding-step",
                            h3 { "🎉 You're all set!" }
                            p { 
                                "Hive Consensus is ready to use. Here are some things you can try:" 
                            }
                            
                            div {
                                class: "suggestions-list",
                                div { 
                                    class: "suggestion-item",
                                    "💡 Ask about your code: \"What does this function do?\"" 
                                }
                                div { 
                                    class: "suggestion-item",
                                    "🔧 Request help: \"How can I optimize this algorithm?\"" 
                                }
                                div { 
                                    class: "suggestion-item",
                                    "📚 Learn concepts: \"Explain async/await in Rust\"" 
                                }
                                div { 
                                    class: "suggestion-item",
                                    "🐛 Debug issues: \"Why is this code not working?\"" 
                                }
                            }
                            
                            div {
                                style: "margin-top: 20px; padding: 15px; background: #1e1e1e; border-radius: 8px;",
                                p {
                                    style: "margin: 0; color: #cccccc;",
                                    "Selected profile: ",
                                    strong { "{selected_profile.read()}" }
                                }
                            }
                        }
                    } else {
                        // Default case - should not happen
                        div {
                            class: "onboarding-step",
                            h3 { "Loading..." }
                            p { "Step {current_step.read()}" }
                        }
                    }
                }
                
                div {
                    class: "dialog-footer onboarding-footer",
                    if *current_step.read() > 1 {
                        button {
                            class: "button button-secondary",
                            onclick: move |_| {
                                let step = *current_step.read();
                                if step > 1 {
                                    *current_step.write() = step - 1;
                                    *validation_error.write() = None; // Clear errors when going back
                                }
                            },
                            "Back"
                        }
                    }
                    
                    // Skip button for optional Hive license key
                    if *current_step.read() == 2 {
                        button {
                            class: "button button-secondary",
                            onclick: move |_| {
                                *current_step.write() = 3; // Skip to OpenRouter key
                                *validation_error.write() = None;
                            },
                            "Skip"
                        }
                    }
                    
                    // Skip button for OpenRouter key if it already exists
                    if *current_step.read() == 3 && has_existing_openrouter {
                        button {
                            class: "button button-secondary",
                            onclick: move |_| {
                                *current_step.write() = 4; // Skip to Profile configuration
                                *validation_error.write() = None;
                            },
                            "Skip"
                        }
                    }
                    
                    // Show different buttons for profile creation step
                    if *current_step.read() == 4 && *show_profile_success.read() {
                        // After creating profiles, show options to create more or continue
                        button {
                            class: "button button-secondary",
                            onclick: move |_| {
                                // Reset for creating another profile
                                *selected_template.write() = String::new();
                                *profile_name.write() = String::new();
                                *show_profile_success.write() = false;
                                // Don't clear profiles_created - keep the list
                            },
                            "Create Another Profile"
                        }
                        button {
                            class: "button button-primary",
                            onclick: move |_| {
                                // Move to completion
                                *current_step.write() = 5;
                            },
                            "Continue to Finish"
                        }
                    } else {
                        // Show skip option for profile step
                        if *current_step.read() == 4 {
                            button {
                                class: "button button-secondary",
                                onclick: move |_| {
                                    tracing::info!("Skipping profile selection - moving to completion");
                                    *current_step.write() = 5;
                                },
                                "Skip to Finish"
                            }
                        }
                        
                        // Normal Next/Get Started button
                        button {
                            class: "button button-primary",
                            disabled: if (*current_step.read() == 3 && temp_openrouter_key.read().is_empty()) || 
                                     *is_validating.read() || *is_creating_profile.read() { true } else { false },
                            onclick: move |_| {
                            let step = *current_step.read();
                            tracing::info!("Button clicked at step: {}", step);
                            
                            if step == 1 {
                                // Welcome -> License Key
                                *current_step.write() = 2;
                            } else if step == 2 {
                                // Save Hive key if provided
                                let h_key = temp_hive_key.read().clone();
                                if !h_key.is_empty() {
                                    *hive_key.write() = h_key.clone();
                                    // Simple synchronous save
                                    if let Err(e) = crate::desktop::simple_db::save_config("hive_license_key", &h_key) {
                                        tracing::error!("Failed to save Hive key: {}", e);
                                        *validation_error.write() = Some(format!("Failed to save key: {}", e));
                                        return;
                                    } else {
                                        tracing::info!("Hive key saved successfully to database");
                                    }
                                }
                                // Move to next step
                                *current_step.write() = 3;
                            } else if step == 3 {
                                // Save OpenRouter key
                                let or_key = temp_openrouter_key.read().clone();
                                if or_key.is_empty() {
                                    return;
                                }
                                
                                // Simple validation - just check it starts with sk-or-v1-
                                if !or_key.starts_with("sk-or-v1-") {
                                    *validation_error.write() = Some("Invalid key format. OpenRouter keys start with 'sk-or-v1-'".to_string());
                                    return;
                                }
                                
                                // Save the key
                                *openrouter_key.write() = or_key.clone();
                                if let Err(e) = crate::desktop::simple_db::save_config("openrouter_api_key", &or_key) {
                                    tracing::error!("Failed to save OpenRouter key: {}", e);
                                    *validation_error.write() = Some(format!("Failed to save key: {}", e));
                                    return;
                                } else {
                                    tracing::info!("OpenRouter key saved successfully to database");
                                    // Move to profile selection
                                    *current_step.write() = 4;
                                }
                            } else if step == 4 {
                                // Profile creation with continuous flow support
                                tracing::info!("Step 4: Profile configuration");
                                let mode = profile_mode.read().clone();
                                let template_id = selected_template.read().clone();
                                let profile_name_val = profile_name.read().clone();
                                let existing_id = *selected_profile_id.read();
                                
                                if mode == "expert" && !template_id.is_empty() {
                                    // Create profile from template
                                    let name = if profile_name_val.is_empty() {
                                        format!("{} Profile", template_id.replace('-', " ")
                                            .split_whitespace()
                                            .map(|w| {
                                                let mut c = w.chars();
                                                match c.next() {
                                                    None => String::new(),
                                                    Some(f) => f.to_uppercase().chain(c).collect()
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                            .join(" "))
                                    } else {
                                        profile_name_val
                                    };
                                    
                                    tracing::info!("Creating profile from template: {} as {}", template_id, name);
                                    let name_for_spawn = name.clone();
                                    let template_for_spawn = template_id.clone();
                                    let mut is_creating_profile = is_creating_profile.clone();
                                    let mut profile_error = profile_error.clone();
                                    let mut profiles_created = profiles_created.clone();
                                    let mut show_profile_success = show_profile_success.clone();
                                    let mut existing_profiles = existing_profiles.clone();
                                    let mut selected_profile = selected_profile.clone();
                                    
                                    spawn(async move {
                                        *is_creating_profile.write() = true;
                                        *profile_error.write() = None;
                                        
                                        match create_profile_from_template(&template_for_spawn, &name_for_spawn).await {
                                            Ok(_) => {
                                                tracing::info!("Profile created successfully: {}", name_for_spawn);
                                                let mut created = profiles_created.read().clone();
                                                created.push(name_for_spawn.clone());
                                                *profiles_created.write() = created;
                                                *show_profile_success.write() = true;
                                                *selected_profile.write() = name_for_spawn;
                                                
                                                // Reload profiles
                                                if let Ok(profiles) = load_existing_profiles().await {
                                                    *existing_profiles.write() = profiles;
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to create profile: {}", e);
                                                *profile_error.write() = Some(e.to_string());
                                            }
                                        }
                                        
                                        *is_creating_profile.write() = false;
                                    });
                                    
                                } else if mode == "existing" && existing_id.is_some() {
                                    // Set existing profile as default
                                    if let Some(profile_id) = existing_id {
                                        tracing::info!("Setting existing profile {} as default", profile_id);
                                        let mut selected_profile = selected_profile.clone();
                                        let existing_profiles = existing_profiles.clone();
                                        let mut show_profile_success = show_profile_success.clone();
                                        
                                        spawn(async move {
                                            if let Err(e) = set_default_profile(profile_id).await {
                                                tracing::error!("Failed to set default profile: {}", e);
                                            } else {
                                                tracing::info!("Default profile set successfully");
                                                // Find the profile name from existing profiles
                                                if let Some(profile) = existing_profiles.read().iter().find(|p| p.id == profile_id) {
                                                    *selected_profile.write() = profile.name.clone();
                                                }
                                                *show_profile_success.write() = true;
                                            }
                                        });
                                    }
                                    
                                    // Move to step 5 after setting existing profile
                                    *current_step.write() = 5;
                                } else if !existing_profiles.read().is_empty() || !profiles_created.read().is_empty() {
                                    // Have some profiles already - can move forward
                                    tracing::info!("Profiles exist - moving to completion");
                                    *current_step.write() = 5;
                                } else {
                                    // No profiles at all - allow moving forward (will create default)
                                    tracing::info!("No profiles selected - moving to completion");
                                    *current_step.write() = 5;
                                }
                                
                                // Note: We don't automatically move to step 5 anymore
                                // User must click "Continue to Finish" after creating profiles
                            } else if step == 5 {
                                // Save onboarding completion and close
                                tracing::info!("Get Started clicked - saving completion and closing");
                                
                                // Simple save - no async
                                if let Err(e) = crate::desktop::simple_db::mark_onboarding_complete() {
                                    tracing::error!("Failed to mark onboarding complete: {}", e);
                                }
                                
                                // Just set closing flag - useEffect will handle the actual close
                                *is_closing.write() = true;
                                
                                tracing::info!("Dialog close initiated");
                            } else {
                                // This shouldn't happen, but log it
                                tracing::warn!("Unexpected step in button handler: {}", step);
                            }
                            },
                            if *is_validating.read() { 
                                "Validating..." 
                            } else if *is_creating_profile.read() { 
                                "Creating Profile..." 
                            } else if *current_step.read() < 5 { 
                                "Next" 
                            } else { 
                                "Get Started" 
                            }
                        }
                    }
                }
            }
        }
    }
}

/// License validation result
#[derive(Debug, Clone)]
struct LicenseValidationResult {
    valid: bool,
    tier: String,
    daily_limit: u32,
    user_id: String,
    email: Option<String>,
}

/// Save just the Hive key to database - simple synchronous version
fn save_hive_key_sync(hive_key: &str) -> anyhow::Result<()> {
    use crate::core::database::DatabaseManager;
    use crate::core::config::get_hive_config_dir;
    use rusqlite::params;
    
    let db_path = get_hive_config_dir().join("hive-ai.db");
    let db_config = crate::core::database::DatabaseConfig {
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
    
    // Use sync version
    let db = std::sync::Arc::new(tokio::runtime::Handle::current().block_on(async {
        DatabaseManager::new(db_config).await
    })?);
    
    let conn = db.get_connection()?;
    
    conn.execute(
        "INSERT OR REPLACE INTO configurations (key, value, created_at, updated_at) VALUES ('hive_license_key', ?, datetime('now'), datetime('now'))",
        params![hive_key],
    )?;
    
    tracing::info!("Hive key saved to configurations table");
    Ok(())
}

/// Save OpenRouter key synchronously
fn save_openrouter_key_sync(openrouter_key: &str) -> anyhow::Result<()> {
    use crate::core::database::DatabaseManager;
    use crate::core::config::get_hive_config_dir;
    use rusqlite::params;
    
    let db_path = get_hive_config_dir().join("hive-ai.db");
    let db_config = crate::core::database::DatabaseConfig {
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
    
    let db = std::sync::Arc::new(tokio::runtime::Handle::current().block_on(async {
        DatabaseManager::new(db_config).await
    })?);
    
    let conn = db.get_connection()?;
    
    conn.execute(
        "INSERT OR REPLACE INTO configurations (key, value, created_at, updated_at) VALUES ('openrouter_api_key', ?, datetime('now'), datetime('now'))",
        params![openrouter_key],
    )?;
    
    tracing::info!("OpenRouter key saved to configurations table");
    Ok(())
}

/// Save API keys with validation and database storage
async fn save_api_keys(openrouter_key: &str, hive_key: &str) -> anyhow::Result<Option<LicenseValidationResult>> {
    use crate::core::api_keys::ApiKeyManager;
    use crate::core::{license::LicenseManager, config::get_hive_config_dir};
    
    tracing::info!("save_api_keys called - OpenRouter: {} chars, Hive: {} chars",
                 openrouter_key.len(), if hive_key.is_empty() { 0 } else { hive_key.len() });
    
    let mut license_result = None;
    
    // Validate Hive license key if provided
    if !hive_key.is_empty() {
        tracing::info!("Validating Hive license key...");
        let license_manager = LicenseManager::new(get_hive_config_dir());
        
        match license_manager.validate_license(hive_key).await {
            Ok(validation) => {
                if validation.valid {
                    // Store license
                    license_manager.store_license(hive_key, &validation).await?;
                    
                    license_result = Some(LicenseValidationResult {
                        valid: validation.valid,
                        tier: validation.tier.clone(),
                        daily_limit: validation.daily_limit,
                        user_id: validation.user_id.clone(),
                        email: validation.email.clone(),
                    });
                    
                    tracing::info!("License validated - tier: {}, daily_limit: {}", 
                                 validation.tier, validation.daily_limit);
                } else {
                    return Err(anyhow::anyhow!("Invalid license key: {}", 
                        validation.message.unwrap_or_else(|| "Unknown error".to_string())));
                }
            }
            Err(e) => {
                tracing::warn!("License validation failed: {}", e);
                // Continue without license (free tier)
            }
        }
    }
    
    // Validate OpenRouter key format first
    if !openrouter_key.is_empty() {
        ApiKeyManager::validate_format(openrouter_key)?;
        
        // Test with live API call
        match ApiKeyManager::test_openrouter_key(openrouter_key).await {
            Ok(true) => {
                tracing::info!("OpenRouter key validated successfully");
                // Key is valid, save to database
                ApiKeyManager::save_to_database(Some(openrouter_key), Some(hive_key)).await?;
                tracing::info!("API keys saved to database");
            }
            Ok(false) => {
                return Err(anyhow::anyhow!("API key validation failed"));
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to validate API key: {}", e));
            }
        }
    } else {
        return Err(anyhow::anyhow!("OpenRouter API key is required"));
    }
    
    Ok(license_result)
}

/// Save profile preference to configuration
async fn save_profile_preference(profile: &str) -> anyhow::Result<()> {
    // For now, just log the profile preference
    // In the full implementation, this would:
    // 1. Create the profile in the pipeline_profiles table
    // 2. Set it as the default profile
    // 3. Configure the appropriate models for each stage
    
    tracing::info!("User selected profile: {}", profile);
    
    // TODO: When database is fully implemented:
    // - Create profile in pipeline_profiles table
    // - Set appropriate models for each stage based on profile type
    // - Mark as default profile
    
    Ok(())
}

/// Load existing profiles from database
pub async fn load_existing_profiles() -> anyhow::Result<Vec<ProfileInfo>> {
    use crate::core::database::DatabaseManager;
    use crate::core::config::get_hive_config_dir;
    
    let db_path = get_hive_config_dir().join("hive-ai.db");
    if !db_path.exists() {
        return Ok(vec![]);
    }
    
    let db_config = crate::core::database::DatabaseConfig {
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
    
    let db = DatabaseManager::new(db_config).await?;
    let conn = db.get_connection()?;
    
    let mut stmt = conn.prepare(
        "SELECT id, name, is_default, created_at 
         FROM consensus_profiles 
         ORDER BY is_default DESC, created_at DESC"
    )?;
    
    let profiles = stmt.query_map([], |row| {
        Ok(ProfileInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            is_default: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?
    .filter_map(Result::ok)
    .collect();
    
    Ok(profiles)
}

/// Load the current default profile from database
async fn load_default_profile() -> anyhow::Result<Option<(i64, String)>> {
    use crate::core::database::DatabaseManager;
    use crate::core::config::get_hive_config_dir;
    use rusqlite::OptionalExtension;
    
    let db_path = get_hive_config_dir().join("hive-ai.db");
    let db_config = crate::core::database::DatabaseConfig {
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
    
    let db = DatabaseManager::new(db_config).await?;
    let conn = db.get_connection()?;
    
    // Query for the default profile
    let result = conn.query_row(
        "SELECT id, name FROM consensus_profiles WHERE is_default = 1 LIMIT 1",
        [],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    ).optional()?;
    
    Ok(result)
}

/// Create profile from expert template
async fn create_profile_from_template(template_id: &str, profile_name: &str) -> anyhow::Result<()> {
    use crate::core::database::DatabaseManager;
    use crate::core::config::get_hive_config_dir;
    use crate::core::profiles::ExpertTemplateManager;
    
    let db_path = get_hive_config_dir().join("hive-ai.db");
    let db_config = crate::core::database::DatabaseConfig {
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
    
    let db = DatabaseManager::new(db_config).await?;
    let template_manager = ExpertTemplateManager::new(db);
    
    template_manager.create_profile_from_template(template_id, profile_name, None).await?;
    
    Ok(())
}

/// Set a profile as default
async fn set_default_profile(profile_id: i64) -> anyhow::Result<()> {
    use crate::core::database::DatabaseManager;
    use crate::core::config::get_hive_config_dir;
    
    let db_path = get_hive_config_dir().join("hive-ai.db");
    let db_config = crate::core::database::DatabaseConfig {
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
    
    let db = DatabaseManager::new(db_config).await?;
    let conn = db.get_connection()?;
    
    // First, unset all profiles as default
    conn.execute("UPDATE consensus_profiles SET is_default = 0", [])?;
    
    // Then set the selected profile as default
    conn.execute(
        "UPDATE consensus_profiles SET is_default = 1 WHERE id = ?1",
        [profile_id]
    )?;
    
    Ok(())
}

/// Mark onboarding as complete in the database
pub async fn mark_onboarding_complete() -> anyhow::Result<()> {
    use crate::core::database::DatabaseManager;
    use crate::core::config::get_hive_config_dir;
    
    let db_path = get_hive_config_dir().join("hive-ai.db");
    let db_config = crate::core::database::DatabaseConfig {
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
    
    let db = DatabaseManager::new(db_config).await?;
    let conn = db.get_connection()?;
    
    // Store onboarding completion in configurations table
    conn.execute(
        "INSERT OR REPLACE INTO configurations (key, value, created_at, updated_at) 
         VALUES ('onboarding_completed', 'true', datetime('now'), datetime('now'))",
        []
    )?;
    
    Ok(())
}

/// CSS styles for dialogs
pub const DIALOG_STYLES: &str = r#"
    .dialog-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }
    
    .dialog-box {
        background: #2d2d30;
        border: 1px solid #3e3e42;
        border-radius: 8px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        max-width: 90%;
        max-height: 90vh;
        overflow: auto;
    }
    
    .about-dialog {
        width: 500px;
    }
    
    .dialog-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 20px;
        border-bottom: 1px solid #3e3e42;
    }
    
    .dialog-header h2 {
        margin: 0;
        font-size: 20px;
        color: #ffffff;
    }
    
    .dialog-close {
        background: none;
        border: none;
        color: #cccccc;
        font-size: 24px;
        cursor: pointer;
        padding: 0;
        width: 30px;
        height: 30px;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    
    .dialog-close:hover {
        color: #ffffff;
        background: #3e3e42;
        border-radius: 4px;
    }
    
    .dialog-content {
        padding: 30px;
        text-align: center;
    }
    
    .app-icon {
        font-size: 64px;
        margin-bottom: 20px;
    }
    
    .dialog-content h3 {
        margin: 10px 0;
        font-size: 24px;
        color: #ffffff;
    }
    
    .dialog-content p {
        margin: 10px 0;
        color: #cccccc;
    }
    
    .dialog-features {
        margin: 30px 0;
        text-align: left;
    }
    
    .dialog-features h4 {
        margin-bottom: 10px;
        color: #ffffff;
    }
    
    .dialog-features ul {
        list-style: none;
        padding: 0;
    }
    
    .dialog-features li {
        padding: 5px 0;
        color: #cccccc;
    }
    
    .dialog-footer-text {
        margin-top: 30px;
        font-size: 12px;
        color: #858585;
    }
    
    /* Welcome tab styles */
    .welcome-tab {
        padding: 40px;
        max-width: 800px;
        margin: 0 auto;
    }
    
    .welcome-header {
        text-align: center;
        margin-bottom: 40px;
    }
    
    .welcome-header h1 {
        font-size: 36px;
        margin-bottom: 10px;
    }
    
    .welcome-sections {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 30px;
        margin-bottom: 40px;
    }
    
    .welcome-section {
        background: #2d2d30;
        padding: 20px;
        border-radius: 8px;
        border: 1px solid #3e3e42;
    }
    
    .welcome-section h3 {
        margin-bottom: 15px;
    }
    
    .welcome-button {
        display: block;
        width: 100%;
        padding: 10px;
        margin: 5px 0;
        background: #007acc;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
    }
    
    .welcome-button:hover {
        background: #005a9e;
    }
    
    .welcome-link {
        display: block;
        padding: 5px 0;
        color: #007acc;
        text-decoration: none;
    }
    
    .welcome-link:hover {
        text-decoration: underline;
    }
    
    /* Template and profile option styles */
    .template-option, .profile-option {
        transition: all 0.2s ease;
    }
    
    .template-option:hover, .profile-option:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    }
    
    .template-option.selected, .profile-option.selected {
        background: #1e1e1e !important;
        border-color: #007acc !important;
    }
    
    .expert-templates-grid {
        max-height: 350px;
        overflow-y: auto;
        padding-right: 10px;
    }
    
    .expert-templates-grid::-webkit-scrollbar {
        width: 8px;
    }
    
    .expert-templates-grid::-webkit-scrollbar-track {
        background: #1e1e1e;
        border-radius: 4px;
    }
    
    .expert-templates-grid::-webkit-scrollbar-thumb {
        background: #3e3e42;
        border-radius: 4px;
    }
    
    .expert-templates-grid::-webkit-scrollbar-thumb:hover {
        background: #4e4e52;
    }
    
    .welcome-close {
        display: block;
        margin: 0 auto;
        padding: 10px 30px;
        background: #3e3e42;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
    }
    
    .welcome-close:hover {
        background: #4e4e52;
    }
    
    /* Command palette styles */
    .command-palette {
        background: #252526;
        border: 1px solid #3e3e42;
        border-radius: 8px;
        width: 600px;
        max-height: 400px;
        overflow: hidden;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    }
    
    .command-palette-input {
        width: 100%;
        padding: 15px 20px;
        background: #3c3c3c;
        border: none;
        border-bottom: 1px solid #3e3e42;
        color: #ffffff;
        font-size: 16px;
        outline: none;
    }
    
    .command-palette-results {
        max-height: 350px;
        overflow-y: auto;
    }
    
    .command-palette-item {
        display: flex;
        justify-content: space-between;
        padding: 10px 20px;
        cursor: pointer;
    }
    
    .command-palette-item:hover {
        background: #094771;
    }
    
    .command-name {
        color: #cccccc;
    }
    
    .command-shortcut {
        color: #858585;
        font-size: 12px;
    }
    
    /* Settings dialog styles */
    .settings-dialog {
        width: 600px;
    }
    
    .settings-content {
        padding: 0;
        text-align: left;
    }
    
    .settings-section {
        padding: 20px 30px;
        border-bottom: 1px solid #3e3e42;
    }
    
    .settings-section:last-child {
        border-bottom: none;
    }
    
    .settings-section h3 {
        margin: 0 0 10px 0;
        color: #ffffff;
        font-size: 16px;
    }
    
    .settings-description {
        margin: 0 0 20px 0;
        color: #858585;
        font-size: 13px;
    }
    
    .settings-field {
        margin-bottom: 20px;
    }
    
    .settings-label {
        display: block;
        margin-bottom: 8px;
        color: #cccccc;
        font-size: 14px;
    }
    
    .settings-input {
        width: 100%;
        padding: 8px 12px;
        background: #3c3c3c;
        border: 1px solid #3e3e42;
        border-radius: 4px;
        color: #ffffff;
        font-size: 14px;
    }
    
    .settings-input:focus {
        outline: none;
        border-color: #007acc;
    }
    
    .settings-hint {
        margin: 5px 0 0 0;
        font-size: 12px;
        color: #858585;
    }
    
    .settings-hint a {
        color: #007acc;
        text-decoration: none;
    }
    
    .settings-hint a:hover {
        text-decoration: underline;
    }
    
    .profile-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 15px;
    }
    
    .profile-option {
        padding: 15px;
        background: #1e1e1e;
        border: 1px solid #3e3e42;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s;
    }
    
    .profile-option:hover {
        border-color: #007acc;
    }
    
    .profile-option.selected {
        border-color: #007acc;
        background: #094771;
    }
    
    .profile-option h4 {
        margin: 0 0 5px 0;
        color: #ffffff;
        font-size: 14px;
    }
    
    .profile-description {
        margin: 0 0 8px 0;
        color: #cccccc;
        font-size: 12px;
    }
    
    .profile-models {
        margin: 0;
        color: #858585;
        font-size: 11px;
    }
    
    /* Dialog footer */
    .dialog-footer {
        display: flex;
        justify-content: flex-end;
        gap: 10px;
        padding: 20px;
        border-top: 1px solid #3e3e42;
        background: #252526;
    }
    
    .button {
        padding: 8px 16px;
        border: none;
        border-radius: 4px;
        font-size: 14px;
        cursor: pointer;
        transition: background 0.2s;
    }
    
    .button-primary {
        background: #007acc;
        color: white;
    }
    
    .button-primary:hover {
        background: #005a9e;
    }
    
    .button-secondary {
        background: #3e3e42;
        color: #cccccc;
    }
    
    .button-secondary:hover {
        background: #4e4e52;
    }
    
    /* Onboarding dialog styles */
    .onboarding-dialog {
        width: 600px;
    }
    
    .onboarding-content {
        padding: 30px;
        text-align: left;
    }
    
    .onboarding-step h3 {
        margin: 0 0 15px 0;
        color: #ffffff;
        font-size: 20px;
    }
    
    .onboarding-step p {
        margin: 0 0 20px 0;
        color: #cccccc;
        line-height: 1.5;
    }
    
    .features-list {
        margin: 20px 0;
    }
    
    .feature-item {
        padding: 8px 0;
        color: #cccccc;
        font-size: 14px;
    }
    
    .api-key-help {
        margin-top: 15px;
        padding: 15px;
        background: #1e1e1e;
        border-radius: 6px;
        border: 1px solid #3e3e42;
    }
    
    .api-key-help p {
        margin: 0 0 10px 0;
    }
    
    .api-key-help p:last-child {
        margin: 0;
    }
    
    .suggestions-list {
        margin-top: 20px;
    }
    
    .suggestion-item {
        padding: 10px 15px;
        margin: 8px 0;
        background: #1e1e1e;
        border: 1px solid #3e3e42;
        border-radius: 6px;
        font-size: 14px;
        color: #cccccc;
    }
    
    .onboarding-footer {
        justify-content: space-between;
    }
    
    .dialog {
        background: #2d2d30;
        border: 1px solid #3e3e42;
        border-radius: 8px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        overflow: hidden;
    }
    
    /* Progress indicator styles */
    .onboarding-progress {
        display: flex;
        justify-content: center;
        align-items: center;
        padding: 20px;
        border-bottom: 1px solid #3e3e42;
    }
    
    .progress-step {
        padding: 8px 16px;
        border-radius: 20px;
        background: #2d2d30;
        color: #858585;
        font-size: 13px;
        transition: all 0.3s;
    }
    
    .progress-step.active {
        background: #007acc;
        color: #ffffff;
    }
    
    .progress-separator {
        margin: 0 5px;
        color: #858585;
    }
"#;