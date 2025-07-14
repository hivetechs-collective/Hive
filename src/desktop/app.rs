//! Main Desktop Application Component

use crate::core::api_keys::ApiKeyManager;
use crate::desktop::{
    chat::ChatInterface,
    components::HiveLogoSmall,
    consensus::ConsensusProgress,
    dialogs::{AboutDialog, CommandPalette, OnboardingDialog, SettingsDialog},
    events::{EventDispatcher, KeyboardEventUtils},
    file_explorer::FileExplorer,
    state::{AppState, ConnectionStatus},
    styles::get_global_styles,
};
use dioxus::events::{KeyboardEvent, MouseEvent};
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;

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
    let mut onboarding_current_step = use_signal(|| 1); // Persist onboarding step

    // Track when API keys have been updated to force consensus re-initialization
    let api_keys_version = use_signal(|| 0u32);

    // API key states - initialize with values from database if available
    let mut openrouter_key = use_signal(|| {
        crate::desktop::simple_db::get_config("openrouter_api_key")
            .ok()
            .flatten()
            .unwrap_or_default()
    });
    let mut hive_key = use_signal(|| {
        crate::desktop::simple_db::get_config("hive_license_key")
            .ok()
            .flatten()
            .unwrap_or_default()
    });

    // Check for API keys and onboarding status on startup (only once)
    use_effect(move || {
        // Don't clone the signals - use them directly
        spawn(async move {
            // Import needed functions
            use crate::core::config::get_hive_config_dir;
            use crate::core::database::{DatabaseConfig, DatabaseManager};
            use crate::desktop::dialogs::load_existing_profiles;

            // Check if onboarding has been completed - simple version
            let onboarding_completed = crate::desktop::simple_db::is_onboarding_completed();

            // Check if onboarding is already completed
            if onboarding_completed {
                tracing::info!("Onboarding already completed - loading keys");
                // Just load the keys from simple_db, don't show onboarding
                if let Ok(Some(key)) = crate::desktop::simple_db::get_config("openrouter_api_key") {
                    openrouter_key.set(key);
                }
                if let Ok(Some(key)) = crate::desktop::simple_db::get_config("hive_license_key") {
                    hive_key.set(key);
                }
                return;
            }

            // Otherwise, check if we need to show onboarding using simple_db
            let has_openrouter = crate::desktop::simple_db::has_openrouter_key();

            if !has_openrouter {
                tracing::info!("No API keys found - showing onboarding");
                show_onboarding.set(true);
            } else {
                // Load existing keys from simple_db
                if let Ok(Some(key)) = crate::desktop::simple_db::get_config("openrouter_api_key") {
                    openrouter_key.set(key);
                }
                if let Ok(Some(key)) = crate::desktop::simple_db::get_config("hive_license_key") {
                    hive_key.set(key);
                }

                // Check if profiles exist
                if let Ok(profiles) = load_existing_profiles().await {
                    if profiles.is_empty() {
                        tracing::info!("Keys exist but no profiles - showing onboarding at step 4");
                        onboarding_current_step.set(4); // Start at profile configuration
                        show_onboarding.set(true);
                    } else {
                        tracing::info!("Setup complete - keys and profiles exist");
                        // Mark onboarding as complete since we have everything
                        if let Err(e) = crate::desktop::simple_db::mark_onboarding_complete() {
                            tracing::warn!("Failed to mark onboarding complete: {}", e);
                        }
                    }
                }
            }
        });
    });

    // Fetch and update usage information periodically
    use_effect(move || {
        let hive_key_value = hive_key.read().clone();
        let mut app_state_clone = app_state.clone();

        spawn(async move {
            // Only fetch if we have a hive key
            if !hive_key_value.is_empty() {
                // Get usage information
                if let Ok(db) = crate::core::get_database().await {
                    let usage_tracker = crate::core::usage_tracker::UsageTracker::new(db);

                    // Extract user_id from license info if available
                    if let Ok(Some(license_info)) = crate::core::license::LicenseManager::new(
                        crate::core::config::get_hive_config_dir(),
                    )
                    .load_license()
                    .await
                    {
                        if let Ok(usage_display) =
                            usage_tracker.get_usage_display(&license_info.user_id).await
                        {
                            app_state_clone.write().update_usage_info(
                                Some(license_info.user_id),
                                &license_info.tier.to_string(),
                                usage_display.daily_used,
                                usage_display.daily_limit,
                                usage_display.is_trial,
                                usage_display.trial_days_left,
                            );
                        }
                    }
                }
            }
        });

        // Set up periodic updates
        spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                let hive_key_value = hive_key.read().clone();
                let mut app_state_clone = app_state.clone();

                if !hive_key_value.is_empty() {
                    if let Ok(db) = crate::core::get_database().await {
                        let usage_tracker = crate::core::usage_tracker::UsageTracker::new(db);

                        if let Ok(Some(license_info)) = crate::core::license::LicenseManager::new(
                            crate::core::config::get_hive_config_dir(),
                        )
                        .load_license()
                        .await
                        {
                            if let Ok(usage_display) =
                                usage_tracker.get_usage_display(&license_info.user_id).await
                            {
                                app_state_clone.write().update_usage_info(
                                    Some(license_info.user_id),
                                    &license_info.tier.to_string(),
                                    usage_display.daily_used,
                                    usage_display.daily_limit,
                                    usage_display.is_trial,
                                    usage_display.trial_days_left,
                                );
                            }
                        }
                    }
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
    use_context_provider(|| show_onboarding.clone());
    use_context_provider(|| api_keys_version.clone());

    // Global keyboard shortcuts
    let on_global_keydown = move |evt: KeyboardEvent| {
        // Handle global keyboard shortcuts
        if evt.modifiers().ctrl() {
            match evt.key() {
                Key::Character(c) if c == "n" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: New file/conversation
                    tracing::debug!("Ctrl+N pressed - New file");
                }
                Key::Character(c) if c == "o" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Open file dialog
                    tracing::debug!("Ctrl+O pressed - Open file");
                }
                Key::Character(c) if c == "s" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Save current file
                    tracing::debug!("Ctrl+S pressed - Save file");
                }
                Key::Character(c) if c == "f" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Search/find
                    tracing::debug!("Ctrl+F pressed - Find");
                }
                Key::Character(c) if c == "p" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    show_command_palette.set(true);
                    tracing::debug!("Ctrl+P pressed - Command palette");
                }
                Key::Character(c) if c == "w" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Close current tab/file
                    tracing::debug!("Ctrl+W pressed - Close tab");
                }
                Key::Character(c) if c == "z" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Undo
                    tracing::debug!("Ctrl+Z pressed - Undo");
                }
                Key::Character(c) if c == "y" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Redo
                    tracing::debug!("Ctrl+Y pressed - Redo");
                }
                _ => {}
            }
        } else if evt.modifiers().alt() {
            match evt.key() {
                Key::Character(c) if c == "1" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Focus file explorer
                    tracing::debug!("Alt+1 pressed - Focus file explorer");
                }
                Key::Character(c) if c == "2" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Focus chat
                    tracing::debug!("Alt+2 pressed - Focus chat");
                }
                Key::Character(c) if c == "3" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Focus consensus panel
                    tracing::debug!("Alt+3 pressed - Focus consensus");
                }
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
            OnboardingDialog {
                show_onboarding,
                openrouter_key,
                hive_key,
                current_step: onboarding_current_step,
                api_keys_version,
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
                // HiveTechs Logo
                HiveLogoSmall {}
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
                // Usage Tracking
                div {
                    class: "status-item usage-tracker",
                    style: format!("color: {}", get_usage_color(&state)),
                    if state.is_trial_active {
                        span {
                            "🎉 Trial: {state.trial_days_remaining.unwrap_or(0)} days left"
                        }
                    } else if state.license_tier == "unlimited" || state.license_tier == "enterprise" {
                        span {
                            "♾️ Unlimited"
                        }
                    } else {
                        span {
                            "{get_usage_emoji(&state)} {state.daily_conversations_used}/{state.daily_conversations_limit}"
                        }
                    }
                }

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

/// Get usage color based on percentage used
fn get_usage_color(state: &AppState) -> &'static str {
    if state.is_trial_active {
        "#10b981" // Green for trial
    } else if state.license_tier == "unlimited" || state.license_tier == "enterprise" {
        "#3b82f6" // Blue for unlimited
    } else {
        let percentage = if state.daily_conversations_limit > 0 {
            (state.daily_conversations_used as f32 / state.daily_conversations_limit as f32) * 100.0
        } else {
            0.0
        };

        if percentage >= 90.0 {
            "#ef4444" // Red
        } else if percentage >= 75.0 {
            "#f97316" // Orange
        } else if percentage >= 50.0 {
            "#eab308" // Yellow
        } else {
            "#10b981" // Green
        }
    }
}

/// Get usage emoji based on percentage used
fn get_usage_emoji(state: &AppState) -> &'static str {
    let percentage = if state.daily_conversations_limit > 0 {
        (state.daily_conversations_used as f32 / state.daily_conversations_limit as f32) * 100.0
    } else {
        0.0
    };

    if percentage >= 90.0 {
        "🔴"
    } else if percentage >= 75.0 {
        "🟠"
    } else if percentage >= 50.0 {
        "🟡"
    } else {
        "🟢"
    }
}
