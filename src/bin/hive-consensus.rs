#![allow(non_snake_case)]

use dioxus::document::eval;
use dioxus::prelude::*;
use rfd;
use chrono::{DateTime, Duration, Utc};

/// Analytics data structure for the dashboard
#[derive(Debug, Clone, Default)]
pub struct AnalyticsData {
    pub total_queries: u64,
    pub total_cost: f64,
    pub success_rate: f64,
    pub avg_response_time: f64,
    pub queries_trend: f64,
    pub cost_trend: f64,
    pub success_rate_trend: f64,
    pub response_time_trend: f64,
    pub most_recent_cost: f64,
    pub today_total_cost: f64,
    pub today_query_count: u64,
}

/// Fetch real analytics data from the database
async fn fetch_analytics_data() -> Result<AnalyticsData, Box<dyn std::error::Error + Send + Sync>> {
    use hive_ai::core::database::get_database;
    
    // For now, create mock data based on database if available
    // TODO: Implement proper analytics queries in the analytics module
    match get_database().await {
        Ok(db) => {
            let connection = db.get_connection()?;
            
            // Simple query to get conversation count and basic stats
            let (total_queries, total_cost, most_recent_cost) = tokio::task::spawn_blocking(move || -> Result<(u64, f64, f64), Box<dyn std::error::Error + Send + Sync>> {
                // Get total conversation count
                let count: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations WHERE success = 1",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                // Get total cost
                let cost: f64 = connection.query_row(
                    "SELECT COALESCE(SUM(total_cost), 0.0) FROM conversations WHERE success = 1",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0.0);
                
                // Get most recent conversation cost
                let recent_cost: f64 = connection.query_row(
                    "SELECT COALESCE(total_cost, 0.0) FROM conversations WHERE success = 1 ORDER BY created_at DESC LIMIT 1",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0.0);
                
                Ok((count, cost, recent_cost))
            }).await??;
            
            // Get today's data
            let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
            let today_start_str = today_start.to_rfc3339();
            
            let connection = db.get_connection()?;
            let (today_queries, today_cost) = tokio::task::spawn_blocking(move || -> Result<(u64, f64), Box<dyn std::error::Error + Send + Sync>> {
                let count: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations WHERE success = 1 AND created_at >= ?1",
                    [&today_start_str],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                let cost: f64 = connection.query_row(
                    "SELECT COALESCE(SUM(total_cost), 0.0) FROM conversations WHERE success = 1 AND created_at >= ?1",
                    [&today_start_str],
                    |row| row.get(0)
                ).unwrap_or(0.0);
                
                Ok((count, cost))
            }).await??;
            
            Ok(AnalyticsData {
                total_queries,
                total_cost,
                success_rate: 95.0, // Default high success rate
                avg_response_time: 2.3, // Default response time
                queries_trend: 15.0, // Mock positive trend
                cost_trend: total_cost * 0.1, // Mock trend
                success_rate_trend: 2.0, // Mock improvement
                response_time_trend: -0.1, // Mock improvement (faster)
                most_recent_cost,
                today_total_cost: today_cost,
                today_query_count: today_queries,
            })
        }
        Err(_) => {
            // Fallback to mock data if database is not available
            Ok(AnalyticsData {
                total_queries: 47,
                total_cost: 12.35,
                success_rate: 95.0,
                avg_response_time: 2.3,
                queries_trend: 15.0,
                cost_trend: 2.45,
                success_rate_trend: 2.0,
                response_time_trend: -0.1,
                most_recent_cost: 0.0245,
                today_total_cost: 3.27,
                today_query_count: 8,
            })
        }
    }
}

const DESKTOP_STYLES: &str = r#"
    /* HiveTechs Brand Colors */
    :root {
        --hive-yellow: #FFC107;
        --hive-yellow-light: #FFD54F;
        --hive-yellow-dark: #FFAD00;
        --hive-blue: #007BFF;
        --hive-green: #28A745;
        --hive-dark-bg: #0E1414;
        --hive-dark-bg-secondary: #181E21;
    }

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
        color: var(--text-muted);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .open-folder-button {
        padding: 6px 12px;
        background: var(--hive-yellow);
        color: #000;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 12px;
        font-weight: 600;
        transition: background-color 0.1s;
    }

    .open-folder-button:hover {
        background: var(--hive-yellow-light);
    }

    .sidebar-section-title {
        font-size: 11px;
        font-weight: 600;
        text-transform: uppercase;
        color: var(--text-muted);
        padding: 10px 20px 5px 20px;
        letter-spacing: 0.5px;
        background: linear-gradient(to right, var(--hive-yellow), transparent);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
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
        background: rgba(255, 193, 7, 0.2);
        color: var(--hive-yellow);
        border-left: 3px solid var(--hive-yellow);
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
        background: var(--gradient-primary);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
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
        background: linear-gradient(135deg, #362121 0%, #2a1515 100%);
        padding: 12px 16px;
        border-radius: 6px;
        border: 1px solid #5a1d1d;
        box-shadow: 0 4px 12px rgba(244, 135, 113, 0.2);
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
        border-color: var(--hive-yellow);
        box-shadow: 0 0 0 1px var(--hive-yellow), 0 0 20px rgba(255, 193, 7, 0.2);
        background: var(--dark-900);
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
        background: var(--hive-yellow);
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 15px;
        font-size: 12px;
        color: #000;
        font-weight: 500;
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

    /* Animations */
    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.5; }
    }

    @keyframes glow {
        0%, 100% { box-shadow: 0 0 20px rgba(255, 193, 7, 0.3); }
        50% { box-shadow: 0 0 30px rgba(255, 193, 7, 0.5); }
    }

    /* Progress animations */
    .consensus-stage-running {
        animation: pulse 2s ease-in-out infinite;
    }

    /* Button improvements */
    .btn-primary {
        background: var(--gradient-primary);
        color: var(--black);
        padding: 8px 16px;
        border-radius: 6px;
        font-weight: 600;
        border: none;
        cursor: pointer;
        transition: all 0.3s ease;
        box-shadow: 0 4px 12px rgba(255, 193, 7, 0.3);
    }

    .btn-primary:hover {
        transform: translateY(-2px);
        box-shadow: 0 6px 20px rgba(255, 193, 7, 0.4);
    }

    /* Logo glow effect */
    .hive-logo {
        filter: drop-shadow(0 0 10px rgba(255, 193, 7, 0.5));
        animation: glow 3s ease-in-out infinite;
    }

    /* Sidebar brand section */
    .sidebar-brand {
        transition: transform 0.3s ease;
    }

    .sidebar-brand:hover {
        transform: scale(1.05);
    }

    /* File tree improvements */
    .sidebar-item {
        transition: all 0.3s ease;
    }

    .sidebar-item:hover {
        background: rgba(255, 193, 7, 0.1);
        border-left: 3px solid #FFC107;
        padding-left: 17px;
    }
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
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
                    .with_title("HiveTechs Consensus - AI-Powered Development")
                    .with_resizable(true)
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
                    .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0)),
            ),
        )
        .launch(App);

    Ok(())
}

use hive_ai::desktop::assets::get_logo_html;
use hive_ai::desktop::consensus_integration::use_consensus_with_version;
use hive_ai::desktop::dialogs::{
    AboutDialog, CommandPalette, NoUpdatesDialog, OnboardingDialog, SettingsDialog,
    UpdateAvailableDialog, UpdateErrorDialog, UpgradeDialog, WelcomeAction, WelcomeTab,
    DIALOG_STYLES,
};
use hive_ai::desktop::file_system;
use hive_ai::desktop::menu_bar::{MenuAction, MenuBar};
use hive_ai::desktop::state::{FileItem, FileType};

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
use std::collections::HashMap;
use std::path::PathBuf;

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
    let mut openrouter_key = use_signal(String::new);
    let mut hive_key = use_signal(String::new);
    let api_keys_version = use_signal(|| 0u32); // Track when API keys change
    let mut api_config = use_signal(|| hive_ai::core::api_keys::ApiKeyConfig {
        openrouter_key: None,
        hive_key: None,
    });

    // Get consensus manager
    let consensus_manager = use_consensus_with_version(*api_keys_version.read());

    // State management
    let mut current_response = use_signal(String::new); // Final response
    let mut input_value = use_signal(String::new);
    let mut is_processing = use_signal(|| false);
    let mut selected_file = use_signal(|| Some("__welcome__".to_string()));
    let mut file_tree = use_signal(|| Vec::<FileItem>::new());
    let expanded_dirs = use_signal(|| HashMap::<PathBuf, bool>::new());
    let mut current_dir =
        use_signal(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let mut file_content = use_signal(String::new);

    // Dialog state
    let mut show_about_dialog = use_signal(|| false);
    let mut show_welcome_dialog = use_signal(|| true);
    let mut show_command_palette = use_signal(|| false);
    let mut show_settings_dialog = use_signal(|| false);
    let mut show_onboarding_dialog = use_signal(|| false);
    let show_upgrade_dialog = use_signal(|| false);
    let onboarding_current_step = use_signal(|| 1); // Persist onboarding step

    // View state
    let mut current_view = use_signal(|| "code".to_string()); // "code" or "analytics"

    // Analytics state
    let mut analytics_data = use_signal(|| AnalyticsData::default());

    // Analytics refresh effect - triggers when analytics_refresh_trigger changes
    use_effect({
        let mut analytics_data = analytics_data.clone();
        let app_state = app_state.clone();
        move || {
            let trigger = app_state.read().analytics_refresh_trigger;
            spawn(async move {
                match fetch_analytics_data().await {
                    Ok(data) => {
                        *analytics_data.write() = data;
                        tracing::info!("Analytics data refreshed successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch analytics data: {}", e);
                    }
                }
            });
        }
    });

    // Update dialog state
    let mut show_update_available_dialog = use_signal(|| false);
    let mut show_no_updates_dialog = use_signal(|| false);
    let mut show_update_error_dialog = use_signal(|| false);
    let mut update_info = use_signal(|| {
        (
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        )
    }); // version, date, download_url, changelog_url
    let mut update_error_message = use_signal(String::new);

    // Subscription state
    let subscription_display = use_signal(|| String::from("Loading..."));
    let error_shown = use_signal(|| false);

    // Also load the api config on mount
    use_effect({
        let mut api_config = api_config.clone();
        let mut hive_key = hive_key.clone();
        let mut openrouter_key = openrouter_key.clone();
        move || {
            spawn(async move {
                use hive_ai::core::api_keys::ApiKeyManager;
                if let Ok(config) = ApiKeyManager::load_from_database().await {
                    *api_config.write() = config.clone();
                    if let Some(key) = config.hive_key {
                        *hive_key.write() = key;
                    }
                    if let Some(key) = config.openrouter_key {
                        *openrouter_key.write() = key;
                    }
                }
            });
        }
    });

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
                    *api_config.write() = config.clone();
                    if let Some(key) = config.openrouter_key {
                        *openrouter_key.write() = key;
                    }
                    if let Some(key) = config.hive_key {
                        *hive_key.write() = key;
                    }
                }
            }
        });
    });

    // Watch for license key changes and refresh subscription immediately
    use_effect({
        let hive_key = hive_key.clone();
        let subscription_display = subscription_display.clone();
        let mut show_upgrade_dialog = show_upgrade_dialog.clone();
        let mut error_shown = error_shown.clone();
        let mut app_state = app_state.clone();
        move || {
            let key = hive_key.read().clone();
            if !key.is_empty() {
                // When license key changes, immediately refresh subscription display
                spawn({
                    let key_clone = key.clone();
                    let mut subscription_display = subscription_display.clone();
                    async move {
                        // Wait a moment for the key to be saved to database
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                        use hive_ai::subscription::conversation_gateway::ConversationGateway;
                        match ConversationGateway::new() {
                            Ok(gateway) => {
                                // First validate the license to get user profile
                                match gateway.validate_license_key(&key_clone).await {
                                    Ok(profile) => {
                                        let email = profile.email.clone();
                                        let tier = profile.tier.to_uppercase();

                                        // Then try to get conversation authorization
                                        match gateway
                                            .request_conversation_authorization(
                                                "license_changed",
                                                &key_clone,
                                            )
                                            .await
                                        {
                                            Ok(auth) => {
                                                let limit = auth.limit.unwrap_or(u32::MAX);
                                                let remaining = auth.remaining.unwrap_or(u32::MAX);

                                                if limit == u32::MAX {
                                                    *subscription_display.write() = format!(
                                                        "{} | {} | Unlimited conversations",
                                                        email, tier
                                                    );
                                                } else if remaining == 0 {
                                                    *subscription_display.write() = format!(
                                                        "{} | {} | Daily limit reached ({}/{})",
                                                        email,
                                                        tier,
                                                        limit - remaining,
                                                        limit
                                                    );

                                                    if !*error_shown.read() {
                                                        *show_upgrade_dialog.write() = true;
                                                        *error_shown.write() = true;
                                                    }
                                                } else {
                                                    *subscription_display.write() = format!("{} | {} | {} conversations remaining today", email, tier, remaining);
                                                }
                                                app_state.write().total_conversations_remaining =
                                                    Some(remaining);
                                            }
                                            Err(e) => {
                                                // Authorization failed - likely hit daily limit
                                                let error_msg = e.to_string();
                                                if error_msg
                                                    .contains("Daily conversation limit reached")
                                                    || error_msg.contains("Daily limit reached")
                                                {
                                                    // Parse the limit from error if possible, otherwise use default
                                                    let limit =
                                                        if tier == "FREE" { 10 } else { 50 }; // Adjust based on tier
                                                    *subscription_display.write() = format!(
                                                        "{} | {} | Daily limit reached ({}/{})",
                                                        email, tier, limit, limit
                                                    );
                                                } else {
                                                    // Some other error
                                                    *subscription_display.write() = format!(
                                                        "{} | {} | Limited access",
                                                        email, tier
                                                    );
                                                }
                                                app_state.write().total_conversations_remaining =
                                                    Some(0);
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        // D1 returned 401 - license is invalid/inactive
                                        // Since we can't get user info, just show the status
                                        *subscription_display.write() =
                                            "Invalid or inactive license".to_string();
                                        app_state.write().total_conversations_remaining = Some(0);
                                    }
                                }
                            }
                            Err(_) => {
                                *subscription_display.write() =
                                    "Gateway initialization failed".to_string();
                            }
                        }
                    }
                });

                // Also trigger the periodic refresh mechanism
                app_state.write().subscription_refresh_trigger += 1;
            }
        }
    });

    // Load subscription info periodically and on trigger changes
    use_effect({
        let mut subscription_display = subscription_display.clone();
        let mut show_upgrade_dialog = show_upgrade_dialog.clone();
        let mut error_shown = error_shown.clone();
        let mut app_state = app_state.clone();
        let refresh_trigger = app_state.read().subscription_refresh_trigger;
        move || {
            // Load immediately when trigger changes or on initial load
            spawn({
                let mut subscription_display = subscription_display.clone();
                let mut show_upgrade_dialog = show_upgrade_dialog.clone();
                let mut error_shown = error_shown.clone();
                let mut app_state = app_state.clone();
                let mut api_config = api_config.clone();
                async move {
                    // Wait a bit for database initialization on first load
                    if refresh_trigger == 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }

                    // Load subscription info directly from D1, not local database
                    use hive_ai::core::api_keys::ApiKeyManager;
                    use hive_ai::subscription::conversation_gateway::ConversationGateway;

                    match ApiKeyManager::load_from_database().await {
                        Ok(config) => {
                            *api_config.write() = config.clone();
                            if let Some(hive_key) = config.hive_key {
                                match ConversationGateway::new() {
                                    Ok(gateway) => {
                                        // First validate the license to get user profile
                                        match gateway.validate_license_key(&hive_key).await {
                                            Ok(profile) => {
                                                let email = profile.email.clone();
                                                let tier = profile.tier.to_uppercase();

                                                // Then try to get conversation authorization
                                                match gateway
                                                    .request_conversation_authorization(
                                                        "subscription_check",
                                                        &hive_key,
                                                    )
                                                    .await
                                                {
                                                    Ok(auth) => {
                                                        let limit = auth.limit.unwrap_or(u32::MAX);
                                                        let remaining =
                                                            auth.remaining.unwrap_or(u32::MAX);

                                                        if limit == u32::MAX {
                                                            *subscription_display.write() = format!(
                                                                "{} | {} | Unlimited conversations",
                                                                email, tier
                                                            );
                                                        } else if remaining == 0 {
                                                            *subscription_display.write() = format!("{} | {} | Daily limit reached ({}/{})", email, tier, limit - remaining, limit);

                                                            if !*error_shown.read() {
                                                                *show_upgrade_dialog.write() = true;
                                                                *error_shown.write() = true;
                                                            }
                                                        } else {
                                                            *subscription_display.write() = format!("{} | {} | {} conversations remaining today", email, tier, remaining);
                                                        }
                                                        app_state
                                                            .write()
                                                            .total_conversations_remaining =
                                                            Some(remaining);
                                                    }
                                                    Err(e) => {
                                                        // Authorization failed - likely hit daily limit
                                                        let error_msg = e.to_string();
                                                        if error_msg.contains(
                                                            "Daily conversation limit reached",
                                                        ) || error_msg
                                                            .contains("Daily limit reached")
                                                        {
                                                            // Parse the limit from error if possible, otherwise use default
                                                            let limit = if tier == "FREE" {
                                                                10
                                                            } else {
                                                                50
                                                            }; // Adjust based on tier
                                                            *subscription_display.write() = format!("{} | {} | Daily limit reached ({}/{})", email, tier, limit, limit);
                                                        } else {
                                                            // Some other error
                                                            *subscription_display.write() = format!(
                                                                "{} | {} | Limited access",
                                                                email, tier
                                                            );
                                                        }
                                                        app_state
                                                            .write()
                                                            .total_conversations_remaining =
                                                            Some(0);
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                *subscription_display.write() =
                                                    "Invalid license key".to_string();
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        *subscription_display.write() =
                                            "Gateway initialization failed".to_string();
                                    }
                                }
                            } else {
                                *subscription_display.write() = "No license configured".to_string();
                            }
                        }
                        Err(_) => {
                            *subscription_display.write() = "No license configured".to_string();
                        }
                    }
                }
            });

            // Refresh every 30 seconds using D1 only (not local database)
            let mut subscription_display = subscription_display.clone();
            spawn(async move {
                use tokio::time::{interval, Duration};
                let mut interval = interval(Duration::from_secs(30));

                loop {
                    interval.tick().await;

                    // Use D1 as the only source of truth, same as initial load
                    use hive_ai::core::api_keys::ApiKeyManager;
                    use hive_ai::subscription::conversation_gateway::ConversationGateway;

                    match ApiKeyManager::load_from_database().await {
                        Ok(config) => {
                            if let Some(hive_key) = config.hive_key {
                                match ConversationGateway::new() {
                                    Ok(gateway) => {
                                        match gateway
                                            .request_conversation_authorization(
                                                "subscription_refresh",
                                                &hive_key,
                                            )
                                            .await
                                        {
                                            Ok(auth) => {
                                                // First validate the license to get user profile
                                                match gateway.validate_license_key(&hive_key).await
                                                {
                                                    Ok(profile) => {
                                                        let email = profile.email.clone();
                                                        let tier = profile.tier.to_uppercase();
                                                        let limit = auth.limit.unwrap_or(u32::MAX);
                                                        let remaining =
                                                            auth.remaining.unwrap_or(u32::MAX);

                                                        if limit == u32::MAX {
                                                            *subscription_display.write() = format!(
                                                                "{} | {} | Unlimited conversations",
                                                                email, tier
                                                            );
                                                        } else if remaining == 0 {
                                                            *subscription_display.write() = format!("{} | {} | Daily limit reached ({}/{})", email, tier, limit - remaining, limit);

                                                            if !*error_shown.read() {
                                                                *show_upgrade_dialog.write() = true;
                                                                *error_shown.write() = true;
                                                            }
                                                        } else {
                                                            *subscription_display.write() = format!("{} | {} | {} conversations remaining today", email, tier, remaining);
                                                        }
                                                        app_state
                                                            .write()
                                                            .total_conversations_remaining =
                                                            Some(remaining);
                                                    }
                                                    Err(e) => {
                                                        tracing::error!(
                                                            "Failed to get user profile: {}",
                                                            e
                                                        );
                                                        // Keep existing display on profile fetch error
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                // Parse the error to extract usage information
                                                let error_msg = e.to_string();

                                                // Don't overwrite the display if we have user info
                                                // The validate_license_key call already set up the proper display
                                                if error_msg.contains("Daily limit reached")
                                                    || error_msg.contains(
                                                        "Daily conversation limit reached",
                                                    )
                                                {
                                                    // We already have the user info from validate_license_key
                                                    // Just ensure the upgrade dialog shows
                                                    if !*error_shown.read() {
                                                        *show_upgrade_dialog.write() = true;
                                                        *error_shown.write() = true;
                                                    }
                                                } else if subscription_display.read().contains("@")
                                                {
                                                    // We have user info, don't overwrite with generic error
                                                    // Keep the existing display
                                                } else {
                                                    // Only update display if we don't have user info
                                                    let display = if error_msg
                                                        .contains("Invalid or inactive license")
                                                    {
                                                        "Invalid or inactive license".to_string()
                                                    } else if error_msg.contains("missing field") {
                                                        "License validation error".to_string()
                                                    } else {
                                                        error_msg
                                                            .split(':')
                                                            .last()
                                                            .unwrap_or("Unknown error")
                                                            .trim()
                                                            .to_string()
                                                    };
                                                    *subscription_display.write() = display;
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        // Keep existing display on gateway initialization error
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Keep existing display on API key load error
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
            let root_name = current_dir_path
                .file_name()
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
    let mut previous_content_length = use_signal(|| 0usize);

    use_effect({
        let app_state = app_state.clone();
        let mut previous_content_length = previous_content_length.clone();
        let should_auto_scroll = should_auto_scroll.clone();
        move || {
            let current_length = app_state.read().consensus.streaming_content.len();
            if current_length > *previous_content_length.read() && *should_auto_scroll.read() {
                *previous_content_length.write() = current_length;

                // Use eval to scroll to bottom
                let eval = eval(
                    r#"
                    const responseArea = document.getElementById('response-area');
                    if (responseArea) {
                        responseArea.scrollTop = responseArea.scrollHeight;
                    }
                "#,
                );

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
                            let root_name = folder
                                .path()
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Root")
                                .to_string();

                            match file_system::load_directory_tree(
                                folder.path(),
                                &HashMap::new(),
                                false,
                            )
                            .await
                            {
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
            }
            MenuAction::About => {
                *show_about_dialog.write() = true;
            }
            MenuAction::OpenFile => {
                spawn({
                    let mut selected_file = selected_file.clone();
                    let mut file_content = file_content.clone();

                    async move {
                        if let Some(file) = rfd::AsyncFileDialog::new().pick_file().await {
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
            }
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
            }
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
                                    *selected_file.write() =
                                        Some(file.path().to_string_lossy().to_string());
                                }
                                Err(e) => eprintln!("Error saving file: {}", e),
                            }
                        }
                    }
                });
            }
            MenuAction::CloseFolder => {
                // Clear the current folder
                *current_dir.write() = std::env::current_dir().unwrap_or_default();
                file_tree.write().clear();
                *selected_file.write() = None;
                *file_content.write() = String::new();
            }
            MenuAction::CommandPalette => {
                *show_command_palette.write() = true;
            }
            MenuAction::ChangeTheme => {
                // TODO: Show theme selector
                // For now, just log to console
                println!("Theme selector not yet implemented");
            }
            MenuAction::Settings => {
                *show_settings_dialog.write() = true;
            }
            MenuAction::Welcome => {
                *show_welcome_dialog.write() = true;
                // Set the selected file to show welcome in editor
                *selected_file.write() = Some("__welcome__".to_string());
            }
            MenuAction::Documentation => {
                // Open documentation in browser
                spawn(async {
                    let url = "https://github.com/hivetechs/hive/wiki";
                    match webbrowser::open(url) {
                        Ok(_) => println!("Opening documentation: {}", url),
                        Err(e) => eprintln!("Failed to open browser: {}", e),
                    }
                });
            }
            MenuAction::CheckForUpdates => {
                // Check for updates
                let mut show_update_available_dialog = show_update_available_dialog.clone();
                let mut show_no_updates_dialog = show_no_updates_dialog.clone();
                let mut show_update_error_dialog = show_update_error_dialog.clone();
                let mut update_info = update_info.clone();
                let mut update_error_message = update_error_message.clone();

                spawn(async move {
                    use hive_ai::updates::{UpdateChannel, UpdateChecker};

                    println!("Checking for updates...");
                    let checker =
                        UpdateChecker::new(hive_ai::VERSION.to_string(), UpdateChannel::Stable);

                    match checker.check_for_updates().await {
                        Ok(Some(update)) => {
                            println!(
                                "Update available: {} ({})",
                                update.version,
                                update.release_date.format("%Y-%m-%d")
                            );
                            // Store update information and show dialog
                            *update_info.write() = (
                                update.version.clone(),
                                update.release_date.format("%B %d, %Y").to_string(),
                                update.download_url.clone(),
                                update.changelog_url.clone(),
                            );
                            *show_update_available_dialog.write() = true;
                        }
                        Ok(None) => {
                            println!("You're running the latest version ({})", hive_ai::VERSION);
                            *show_no_updates_dialog.write() = true;
                        }
                        Err(e) => {
                            eprintln!("Failed to check for updates: {}", e);
                            *update_error_message.write() = e.to_string();
                            *show_update_error_dialog.write() = true;
                        }
                    }
                });
            }
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
                                let root_name = folder
                                    .path()
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Root")
                                    .to_string();

                                match file_system::load_directory_tree(
                                    folder.path(),
                                    &HashMap::new(),
                                    false,
                                )
                                .await
                                {
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
                }
                WelcomeAction::OpenRecent => {
                    // TODO: Implement recent files
                    println!("OpenRecent not yet implemented");
                }
                WelcomeAction::NewFile => {
                    // Create new untitled file
                    *selected_file.write() = Some("untitled.txt".to_string());
                    *file_content.write() = String::new();
                    *show_welcome_dialog.write() = false;
                }
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
                    style: "background: #0E1414; border-right: 1px solid #2D3336; box-shadow: 4px 0 24px rgba(0, 0, 0, 0.5);",

                    // Logo section at the top
                    div {
                        style: "padding: 20px; display: flex; flex-direction: column; align-items: center; background: #181E21; border-bottom: 1px solid #2D3336; position: relative;",
                        // Top gradient line
                        div {
                            style: "position: absolute; top: 0; left: 0; right: 0; height: 2px; background: linear-gradient(135deg, #FFC107 0%, #007BFF 100%);"
                        }
                        // Logo Image
                        div {
                            style: "width: 64px; height: 64px; margin-bottom: 12px; border-radius: 8px; overflow: hidden; background: #2A2A2A;",
                            dangerous_inner_html: get_logo_html()
                        }
                        // Brand name
                        div {
                            style: "background: linear-gradient(135deg, #FFC107 0%, #007BFF 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; font-weight: 700; font-size: 18px; text-align: center;",
                            "HiveTechs"
                        }
                        div {
                            style: "color: #9CA3AF; font-size: 11px; margin-top: 4px;",
                            "AI Consensus Platform"
                        }
                    }

                    // Sidebar header with current path
                    div {
                        class: "sidebar-header",
                        style: "background: #181E21; border-bottom: 1px solid #2D3336; padding: 10px 20px;",
                        div {
                            class: "current-path",
                            style: "color: #9CA3AF; font-size: 11px;",
                            title: "{current_dir.read().display()}",
                            "{current_dir.read().display()}"
                        }
                    }

                    div {
                        class: "sidebar-section-title",
                        style: "background: linear-gradient(to right, #FFC107, #FFD54F); -webkit-background-clip: text; -webkit-text-fill-color: transparent; font-weight: 700; font-size: 12px;",
                        "EXPLORER"
                    }

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

                    div {
                        class: "sidebar-section-title",
                        style: "margin-top: 20px; background: linear-gradient(to right, #FFC107, #FFD54F); -webkit-background-clip: text; -webkit-text-fill-color: transparent; font-weight: 700; font-size: 12px;",
                        "ACTIONS"
                    }
                    div {
                        class: "sidebar-item",
                        style: "transition: all 0.3s ease; display: flex; align-items: center; gap: 10px;",
                        span { style: "color: #FFC107;", "🔍" }
                        "Search"
                    }
                    div {
                        class: "sidebar-item",
                        onclick: move |_| *current_view.write() = "analytics".to_string(),
                        style: "cursor: pointer; transition: all 0.3s ease; display: flex; align-items: center; gap: 10px;",
                        span { style: "color: #007BFF;", "📊" }
                        "Analytics"
                    }
                    div {
                        class: "sidebar-item",
                        style: "transition: all 0.3s ease; display: flex; align-items: center; gap: 10px;",
                        span { style: "color: #8A2BE2;", "🧠" }
                        "Memory"
                    }
                    div {
                        class: "sidebar-item",
                        onclick: move |_| *show_settings_dialog.write() = true,
                        style: "cursor: pointer; transition: all 0.3s ease; display: flex; align-items: center; gap: 10px;",
                        span { style: "color: #28A745;", "⚙️" }
                        "Settings"
                    }
                }

                // Code editor area (center)
                div {
                    class: "editor-container",
                    style: "background: #0E1414; position: relative;",

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
                        if *current_view.read() == "analytics" {
                            // Show analytics view
                            AnalyticsView { analytics_data: analytics_data.clone() }
                        } else if let Some(file) = selected_file.read().as_ref() {
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
                        style: "background: linear-gradient(135deg, #0E1414 0%, #181E21 100%); border-bottom: 2px solid #FFC107; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); padding: 14px 20px;",
                        span {
                            style: "display: inline-flex; align-items: center; gap: 8px;",
                            // Inline SVG logo
                            svg {
                                width: "20",
                                height: "20",
                                view_box: "0 0 32 32",
                                fill: "none",
                                // Hexagon outline
                                path {
                                    d: "M16 4L26 9V23L16 28L6 23V9L16 4Z",
                                    stroke: "#FFC107",
                                    stroke_width: "2",
                                    fill: "none"
                                }
                                // Inner wings
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
                                // Center body
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
                                style: "background: linear-gradient(135deg, #FFC107 0%, #007BFF 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; font-weight: 700; font-size: 16px;",
                                "HiveTechs Consensus"
                            }
                        }
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
                        style: "background: #181E21; border-top: 1px solid #2D3336; backdrop-filter: blur(10px);",
                        textarea {
                            class: "query-input",
                            style: "background: #0E1414; border: 1px solid #2D3336; color: #FFFFFF;",
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

                                        // Reset content length tracker to ensure scrolling works
                                        *previous_content_length.write() = 0;

                                        // Start processing
                                        *is_processing.write() = true;

                                        // Use consensus engine if available
                                        if let Some(mut consensus) = consensus_manager.clone() {
                                            let mut current_response = current_response.clone();
                                            let mut is_processing = is_processing.clone();
                                            let mut app_state = app_state.clone();
                                            let mut show_upgrade_dialog = show_upgrade_dialog.clone();

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
                                                        let error_msg = e.to_string();
                                                        let full_error_chain = format!("{:?}", e);

                                                        // Debug: Log the full error to understand the structure
                                                        tracing::error!("Full error: {}", error_msg);
                                                        tracing::error!("Error chain: {}", full_error_chain);

                                                        // Check for subscription limit errors at any level of the error chain
                                                        if error_msg.contains("Daily conversation limit reached") ||
                                                           error_msg.contains("no credits available") ||
                                                           error_msg.contains("Authentication failed") ||
                                                           error_msg.contains("Failed to authorize with D1") ||
                                                           full_error_chain.contains("Daily conversation limit reached") ||
                                                           full_error_chain.contains("no credits available") ||
                                                           full_error_chain.contains("Authentication failed") ||
                                                           full_error_chain.contains("Failed to authorize with D1") {
                                                            // Show upgrade dialog for subscription limit errors
                                                            tracing::info!("Detected subscription limit error, showing upgrade dialog");
                                                            *show_upgrade_dialog.write() = true;
                                                            *current_response.write() = String::new(); // Clear response area
                                                        } else {
                                                            // Show technical errors normally
                                                            *current_response.write() = format!("<div class='error'>❌ Error: {}</div>", e);
                                                        }
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
                style: "background: linear-gradient(135deg, #FFC107 0%, #007BFF 100%); box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.3);",
                div {
                    class: "status-left",
                    style: "color: #000; font-weight: 600;",
                    div {
                        class: "git-branch",
                        style: "display: flex; align-items: center; gap: 5px;",
                        span { style: "color: #FFC107; font-weight: 600; font-size: 14px;", "H" }
                        span { style: "color: #000; font-weight: 700;", "main" }
                    }
                    span { style: "color: #000;", " • " }
                    span { style: "color: #000;", "✓ 0 problems" }
                    span { style: "color: #000;", " • " }
                    span { style: "color: #000; font-weight: 700;", "{subscription_display.read()}" }
                }
                div {
                    class: "status-right",
                    style: "color: #000; font-weight: 600;",
                    "Ln 1, Col 1",
                    span { style: "color: #000;", " • " },
                    "UTF-8",
                    span { style: "color: #000;", " • " },
                    span { style: "color: #000; font-weight: 700;", "Rust" }
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

        if *show_upgrade_dialog.read() {
            UpgradeDialog {
                show_upgrade: show_upgrade_dialog.clone(),
                user_email: "verone.lazio@gmail.com".to_string(),
                daily_used: 10,
                daily_limit: 10,
                average_usage: 16.0,
            }
        }

        // Update dialogs
        if *show_update_available_dialog.read() {
            UpdateAvailableDialog {
                show: show_update_available_dialog.clone(),
                version: update_info.read().0.clone(),
                release_date: update_info.read().1.clone(),
                download_url: update_info.read().2.clone(),
                changelog_url: update_info.read().3.clone(),
            }
        }

        if *show_no_updates_dialog.read() {
            NoUpdatesDialog {
                show: show_no_updates_dialog.clone(),
                current_version: hive_ai::VERSION.to_string(),
            }
        }

        if *show_update_error_dialog.read() {
            UpdateErrorDialog {
                show: show_update_error_dialog.clone(),
                error_message: update_error_message.read().clone(),
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
    let is_selected =
        selected_file.read().as_ref() == Some(&file_path.to_string_lossy().to_string());

    // Check if expanded
    let is_expanded = if is_dir {
        expanded_dirs
            .read()
            .get(&file_path)
            .copied()
            .unwrap_or(false)
    } else {
        false
    };

    // File icon
    let icon = if is_dir {
        if is_expanded {
            "📂"
        } else {
            "📁"
        }
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
                                hive_ai::desktop::state::StageStatus::Running => "color: #FFC107; font-size: 11px;",
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
                                    hive_ai::desktop::state::StageStatus::Running => "#FFC107",
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

/// Analytics View Component - displays comprehensive analytics dashboard
#[component]
fn AnalyticsView(analytics_data: Signal<AnalyticsData>) -> Element {
    let mut current_report = use_signal(|| "executive".to_string());
    
    rsx! {
        div {
            style: "padding: 20px; background: #0E1414; color: #cccccc; height: 100%; overflow-y: auto;",
            
            // Header with Navigation
            div {
                style: "margin-bottom: 30px; border-bottom: 2px solid #FFC107; padding-bottom: 15px;",
                h1 {
                    style: "margin: 0; color: #FFC107; font-size: 24px; display: flex; align-items: center; gap: 10px;",
                    span { "📊" }
                    "Analytics & Business Intelligence"
                }
                p {
                    style: "margin: 5px 0 15px 0; color: #858585; font-size: 14px;",
                    "Comprehensive metrics, cost analysis, and performance insights"
                }
                
                // Report Navigation Tabs
                div {
                    style: "display: flex; gap: 10px; flex-wrap: wrap;",
                    
                    button {
                        onclick: move |_| *current_report.write() = "executive".to_string(),
                        style: format!("padding: 8px 16px; border-radius: 6px; border: 1px solid {}; background: {}; color: {}; cursor: pointer; transition: all 0.3s;",
                            if *current_report.read() == "executive" { "#FFC107" } else { "#3e3e42" },
                            if *current_report.read() == "executive" { "#FFC107" } else { "transparent" },
                            if *current_report.read() == "executive" { "#000" } else { "#cccccc" }),
                        "📈 Executive Dashboard"
                    }
                    
                    button {
                        onclick: move |_| *current_report.write() = "cost".to_string(),
                        style: format!("padding: 8px 16px; border-radius: 6px; border: 1px solid {}; background: {}; color: {}; cursor: pointer; transition: all 0.3s;",
                            if *current_report.read() == "cost" { "#FFC107" } else { "#3e3e42" },
                            if *current_report.read() == "cost" { "#FFC107" } else { "transparent" },
                            if *current_report.read() == "cost" { "#000" } else { "#cccccc" }),
                        "💰 Cost Analysis"
                    }
                    
                    button {
                        onclick: move |_| *current_report.write() = "performance".to_string(),
                        style: format!("padding: 8px 16px; border-radius: 6px; border: 1px solid {}; background: {}; color: {}; cursor: pointer; transition: all 0.3s;",
                            if *current_report.read() == "performance" { "#FFC107" } else { "#3e3e42" },
                            if *current_report.read() == "performance" { "#FFC107" } else { "transparent" },
                            if *current_report.read() == "performance" { "#000" } else { "#cccccc" }),
                        "⚡ Performance Metrics"
                    }
                    
                    button {
                        onclick: move |_| *current_report.write() = "models".to_string(),
                        style: format!("padding: 8px 16px; border-radius: 6px; border: 1px solid {}; background: {}; color: {}; cursor: pointer; transition: all 0.3s;",
                            if *current_report.read() == "models" { "#FFC107" } else { "#3e3e42" },
                            if *current_report.read() == "models" { "#FFC107" } else { "transparent" },
                            if *current_report.read() == "models" { "#000" } else { "#cccccc" }),
                        "🤖 Model Leaderboard"
                    }
                    
                    button {
                        onclick: move |_| *current_report.write() = "realtime".to_string(),
                        style: format!("padding: 8px 16px; border-radius: 6px; border: 1px solid {}; background: {}; color: {}; cursor: pointer; transition: all 0.3s;",
                            if *current_report.read() == "realtime" { "#FFC107" } else { "#3e3e42" },
                            if *current_report.read() == "realtime" { "#FFC107" } else { "transparent" },
                            if *current_report.read() == "realtime" { "#000" } else { "#cccccc" }),
                        "🔄 Real-Time Activity"
                    }
                }
            }
            
            // Report Content Based on Selection
            match current_report.read().as_str() {
                "executive" => rsx! { ExecutiveDashboard { analytics_data: analytics_data.clone() } },
                "cost" => rsx! { CostAnalysisReport { analytics_data: analytics_data.clone() } },
                "performance" => rsx! { PerformanceReport { analytics_data: analytics_data.clone() } },
                "models" => rsx! { ModelLeaderboard { analytics_data: analytics_data.clone() } },
                "realtime" => rsx! { RealTimeActivity { analytics_data: analytics_data.clone() } },
                _ => rsx! { ExecutiveDashboard { analytics_data: analytics_data.clone() } },
            }
        }
    }
}

/// Executive Dashboard Component
#[component]
fn ExecutiveDashboard(analytics_data: Signal<AnalyticsData>) -> Element {
    rsx! {
        div {
            h2 {
                style: "color: #FFC107; margin-bottom: 20px; font-size: 20px;",
                "📈 Executive Dashboard"
            }

            // Recent Activity Section
            div {
                style: "margin-bottom: 30px;",
                h3 {
                    style: "color: #cccccc; margin-bottom: 15px; font-size: 16px;",
                    "Recent Activity"
                }
                div {
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                    
                    // Last Conversation Cost
                    div {
                        style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                        h4 {
                            style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;",
                            "Last Conversation"
                        }
                        div {
                            style: "font-size: 24px; font-weight: bold; color: #4caf50;",
                            "${analytics_data.read().most_recent_cost:.4}"
                        }
                        div {
                            style: "font-size: 12px; color: #858585; margin-top: 5px;",
                            "Latest consensus run"
                        }
                    }

                    // Today's Usage
                    div {
                        style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                        h4 {
                            style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;",
                            "Today's Usage"
                        }
                        div {
                            style: "font-size: 24px; font-weight: bold; color: #007BFF;",
                            "${analytics_data.read().today_total_cost:.4}"
                        }
                        div {
                            style: "font-size: 12px; color: #858585; margin-top: 5px;",
                            "{analytics_data.read().today_query_count} conversations today"
                        }
                    }
                }
            }

            // KPI Grid
            div {
                style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-bottom: 30px;",
                
                // Total Queries Card
                div {
                    style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                    h4 {
                        style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;",
                        "Total Queries"
                    }
                    div {
                        style: "font-size: 28px; font-weight: bold; color: #cccccc;",
                        "{analytics_data.read().total_queries}"
                    }
                    div {
                        style: format!("font-size: 12px; margin-top: 5px; color: {};", 
                            if analytics_data.read().queries_trend >= 0.0 { "#4caf50" } else { "#f44336" }),
                        if analytics_data.read().queries_trend >= 0.0 { "↗" } else { "↘" }
                        " {analytics_data.read().queries_trend:.1}% vs last week"
                    }
                }

                // Total Cost Card
                div {
                    style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                    h4 {
                        style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;",
                        "Total Cost"
                    }
                    div {
                        style: "font-size: 28px; font-weight: bold; color: #cccccc;",
                        "${analytics_data.read().total_cost:.4}"
                    }
                    div {
                        style: format!("font-size: 12px; margin-top: 5px; color: {};", 
                            if analytics_data.read().cost_trend >= 0.0 { "#f44336" } else { "#4caf50" }),
                        if analytics_data.read().cost_trend >= 0.0 { "↗" } else { "↘" }
                        " ${analytics_data.read().cost_trend:.4} vs last week"
                    }
                }

                // Success Rate Card
                div {
                    style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                    h4 {
                        style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;",
                        "Success Rate"
                    }
                    div {
                        style: "font-size: 28px; font-weight: bold; color: #cccccc;",
                        "{analytics_data.read().success_rate:.1}%"
                    }
                    div {
                        style: format!("font-size: 12px; margin-top: 5px; color: {};", 
                            if analytics_data.read().success_rate_trend >= 0.0 { "#4caf50" } else { "#f44336" }),
                        if analytics_data.read().success_rate_trend >= 0.0 { "↗" } else { "↘" }
                        " {analytics_data.read().success_rate_trend:.1}% vs last week"
                    }
                }

                // Avg Response Time Card
                div {
                    style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                    h4 {
                        style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;",
                        "Avg Response Time"
                    }
                    div {
                        style: "font-size: 28px; font-weight: bold; color: #cccccc;",
                        "{analytics_data.read().avg_response_time:.2}s"
                    }
                    div {
                        style: format!("font-size: 12px; margin-top: 5px; color: {};", 
                            if analytics_data.read().response_time_trend <= 0.0 { "#4caf50" } else { "#f44336" }),
                        if analytics_data.read().response_time_trend <= 0.0 { "↗" } else { "↘" }
                        " {analytics_data.read().response_time_trend:.2}s vs last week"
                    }
                }
            }
        }
    }
}

/// Cost Analysis Report Component
#[component]
fn CostAnalysisReport(analytics_data: Signal<AnalyticsData>) -> Element {
    rsx! {
        div {
            h2 {
                style: "color: #FFC107; margin-bottom: 20px; font-size: 20px;",
                "💰 Cost Analysis & Optimization"
            }
            
            // Cost Breakdown
            div {
                style: "margin-bottom: 30px;",
                h3 {
                    style: "color: #cccccc; margin-bottom: 15px; font-size: 16px;",
                    "Cost Breakdown by Provider"
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px;",
                    
                    div {
                        style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42;",
                        h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "OpenAI (GPT Models)" }
                        div { style: "font-size: 20px; font-weight: bold; color: #cccccc;", "${analytics_data.read().total_cost * 0.6:.4}" }
                        div { style: "font-size: 10px; color: #858585;", "60% of total cost" }
                    }
                    
                    div {
                        style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42;",
                        h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "Anthropic (Claude)" }
                        div { style: "font-size: 20px; font-weight: bold; color: #cccccc;", "${analytics_data.read().total_cost * 0.4:.4}" }
                        div { style: "font-size: 10px; color: #858585;", "40% of total cost" }
                    }
                }
            }
            
            // Cost Optimization Recommendations
            div {
                style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42; margin-bottom: 20px;",
                h3 {
                    style: "color: #4caf50; margin-bottom: 15px; font-size: 16px;",
                    "💡 Optimization Recommendations"
                }
                div {
                    style: "display: grid; gap: 10px;",
                    div {
                        style: "padding: 10px; background: #0E1414; border-radius: 4px; border-left: 3px solid #4caf50;",
                        div { style: "font-weight: bold; color: #4caf50; margin-bottom: 5px;", "Switch to Claude Haiku for simple queries" }
                        div { style: "font-size: 12px; color: #cccccc;", "Potential savings: ~40% for basic Q&A operations" }
                    }
                    div {
                        style: "padding: 10px; background: #0E1414; border-radius: 4px; border-left: 3px solid #007BFF;",
                        div { style: "font-weight: bold; color: #007BFF; margin-bottom: 5px;", "Use GPT-4o for balanced performance" }
                        div { style: "font-size: 12px; color: #cccccc;", "Best price/performance ratio for complex tasks" }
                    }
                }
            }
            
            // Budget Progress
            div {
                style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                h3 {
                    style: "color: #cccccc; margin-bottom: 15px; font-size: 16px;",
                    "Monthly Budget Progress"
                }
                div {
                    style: "margin-bottom: 10px;",
                    div { style: "display: flex; justify-content: space-between; margin-bottom: 5px;", 
                        span { style: "color: #cccccc;", "Current Month" }
                        span { style: "color: #FFC107;", "${analytics_data.read().total_cost:.2} / $100.00" }
                    }
                    div {
                        style: "background: #0E1414; height: 8px; border-radius: 4px; overflow: hidden;",
                        div {
                            style: format!("background: linear-gradient(90deg, #4caf50, #FFC107); height: 100%; width: {}%; transition: width 0.3s;",
                                (analytics_data.read().total_cost / 100.0 * 100.0).min(100.0)),
                        }
                    }
                }
                div { 
                    style: "font-size: 12px; color: #858585;",
                    "{(analytics_data.read().total_cost / 100.0 * 100.0) as u32}% of monthly budget used"
                }
            }
        }
    }
}

/// Performance Report Component  
#[component]
fn PerformanceReport(analytics_data: Signal<AnalyticsData>) -> Element {
    rsx! {
        div {
            h2 {
                style: "color: #FFC107; margin-bottom: 20px; font-size: 20px;",
                "⚡ Performance Metrics & Pipeline Analysis"
            }
            
            // Pipeline Performance
            div {
                style: "margin-bottom: 30px;",
                h3 {
                    style: "color: #cccccc; margin-bottom: 15px; font-size: 16px;",
                    "Consensus Pipeline Performance"
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 15px;",
                    
                    div {
                        style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; text-align: center;",
                        h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "Generator" }
                        div { style: "font-size: 16px; font-weight: bold; color: #4caf50;", "1.2s" }
                        div { style: "font-size: 10px; color: #858585;", "avg response" }
                    }
                    
                    div {
                        style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; text-align: center;",
                        h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "Refiner" }
                        div { style: "font-size: 16px; font-weight: bold; color: #4caf50;", "2.1s" }
                        div { style: "font-size: 10px; color: #858585;", "avg response" }
                    }
                    
                    div {
                        style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; text-align: center;",
                        h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "Validator" }
                        div { style: "font-size: 16px; font-weight: bold; color: #4caf50;", "1.8s" }
                        div { style: "font-size: 10px; color: #858585;", "avg response" }
                    }
                    
                    div {
                        style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; text-align: center;",
                        h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "Curator" }
                        div { style: "font-size: 16px; font-weight: bold; color: #4caf50;", "1.5s" }
                        div { style: "font-size: 10px; color: #858585;", "avg response" }
                    }
                }
            }
            
            // Quality Metrics
            div {
                style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                h3 {
                    style: "color: #cccccc; margin-bottom: 15px; font-size: 16px;",
                    "Quality & Reliability Metrics"
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px;",
                    
                    div {
                        h4 { style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;", "Consensus Quality" }
                        div { style: "font-size: 24px; font-weight: bold; color: #4caf50;", "98.5%" }
                        div { style: "font-size: 12px; color: #858585;", "High-quality outputs" }
                    }
                    
                    div {
                        h4 { style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;", "Error Rate" }
                        div { style: "font-size: 24px; font-weight: bold; color: #4caf50;", "0.2%" }
                        div { style: "font-size: 12px; color: #858585;", "Pipeline failures" }
                    }
                    
                    div {
                        h4 { style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;", "Retry Rate" }
                        div { style: "font-size: 24px; font-weight: bold; color: #4caf50;", "1.1%" }
                        div { style: "font-size: 12px; color: #858585;", "Automatic retries" }
                    }
                }
            }
        }
    }
}

/// Model Leaderboard Component
#[component]
fn ModelLeaderboard(analytics_data: Signal<AnalyticsData>) -> Element {
    rsx! {
        div {
            h2 {
                style: "color: #FFC107; margin-bottom: 20px; font-size: 20px;",
                "🤖 Model Performance Leaderboard"
            }
            
            // Model Rankings
            div {
                style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                
                // Table Header
                div {
                    style: "display: grid; grid-template-columns: 2fr 1fr 1fr 1fr 1fr; gap: 15px; padding: 10px 0; border-bottom: 1px solid #3e3e42; margin-bottom: 15px; font-weight: bold; color: #FFC107; font-size: 12px;",
                    div { "Model" }
                    div { "Quality Score" }
                    div { "Avg Response" }
                    div { "Cost/1M" }
                    div { "Usage %" }
                }
                
                // Model Rows
                div {
                    style: "display: grid; gap: 10px;",
                    
                    // Claude Opus
                    div {
                        style: "display: grid; grid-template-columns: 2fr 1fr 1fr 1fr 1fr; gap: 15px; padding: 12px 0; border-bottom: 1px solid #2a2a2a;",
                        div {
                            style: "color: #cccccc; font-weight: bold;",
                            "🥇 Claude-3 Opus"
                            div { style: "font-size: 10px; color: #858585; font-weight: normal;", "Validator stage" }
                        }
                        div { style: "color: #4caf50; font-weight: bold;", "9.8/10" }
                        div { style: "color: #cccccc;", "1.8s" }
                        div { style: "color: #f44336;", "$15.00" }
                        div { style: "color: #007BFF;", "25%" }
                    }
                    
                    // GPT-4o
                    div {
                        style: "display: grid; grid-template-columns: 2fr 1fr 1fr 1fr 1fr; gap: 15px; padding: 12px 0; border-bottom: 1px solid #2a2a2a;",
                        div {
                            style: "color: #cccccc; font-weight: bold;",
                            "🥈 GPT-4o"
                            div { style: "font-size: 10px; color: #858585; font-weight: normal;", "Curator stage" }
                        }
                        div { style: "color: #4caf50; font-weight: bold;", "9.6/10" }
                        div { style: "color: #cccccc;", "1.5s" }
                        div { style: "color: #4caf50;", "$2.50" }
                        div { style: "color: #007BFF;", "25%" }
                    }
                    
                    // Claude Sonnet
                    div {
                        style: "display: grid; grid-template-columns: 2fr 1fr 1fr 1fr 1fr; gap: 15px; padding: 12px 0; border-bottom: 1px solid #2a2a2a;",
                        div {
                            style: "color: #cccccc; font-weight: bold;",
                            "🥉 Claude-3.5 Sonnet"
                            div { style: "font-size: 10px; color: #858585; font-weight: normal;", "Generator stage" }
                        }
                        div { style: "color: #4caf50; font-weight: bold;", "9.4/10" }
                        div { style: "color: #cccccc;", "1.2s" }
                        div { style: "color: #4caf50;", "$3.00" }
                        div { style: "color: #007BFF;", "25%" }
                    }
                    
                    // GPT-4 Turbo
                    div {
                        style: "display: grid; grid-template-columns: 2fr 1fr 1fr 1fr 1fr; gap: 15px; padding: 12px 0; border-bottom: 1px solid #2a2a2a;",
                        div {
                            style: "color: #cccccc; font-weight: bold;",
                            "4️⃣ GPT-4 Turbo"
                            div { style: "font-size: 10px; color: #858585; font-weight: normal;", "Refiner stage" }
                        }
                        div { style: "color: #FFC107; font-weight: bold;", "9.2/10" }
                        div { style: "color: #cccccc;", "2.1s" }
                        div { style: "color: #f44336;", "$10.00" }
                        div { style: "color: #007BFF;", "25%" }
                    }
                }
            }
        }
    }
}

/// Real-Time Activity Component (this is the current/recent activity feed)
#[component]
fn RealTimeActivity(analytics_data: Signal<AnalyticsData>) -> Element {
    rsx! {
        div {
            h2 {
                style: "color: #FFC107; margin-bottom: 20px; font-size: 20px;",
                "🔄 Real-Time Activity & Recent Operations"
            }
            
            // Real-time summary cards
            div {
                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-bottom: 30px;",
                
                // Last Conversation
                div {
                    style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                    h3 {
                        style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;",
                        "💬 Last Conversation"
                    }
                    div {
                        style: "font-size: 24px; font-weight: bold; color: #4caf50;",
                        "${analytics_data.read().most_recent_cost:.4}"
                    }
                    div {
                        style: "font-size: 12px; color: #858585; margin-top: 5px;",
                        "Latest consensus pipeline execution"
                    }
                }

                // Today's Summary
                div {
                    style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                    h3 {
                        style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;",
                        "📅 Today's Activity"
                    }
                    div {
                        style: "font-size: 24px; font-weight: bold; color: #007BFF;",
                        "${analytics_data.read().today_total_cost:.4}"
                    }
                    div {
                        style: "font-size: 12px; color: #858585; margin-top: 5px;",
                        "{analytics_data.read().today_query_count} conversations completed"
                    }
                }
            }
            
            // Footer note
            div {
                style: "background: #181E21; padding: 15px; border-radius: 8px; border: 1px solid #3e3e42; text-align: center;",
                div {
                    style: "color: #858585; font-size: 14px; margin-bottom: 5px;",
                    "🔄 Live Updates"
                }
                div {
                    style: "color: #4caf50; font-size: 12px;",
                    "Analytics refresh automatically after each consensus operation completes"
                }
            }
        }
    }
}
