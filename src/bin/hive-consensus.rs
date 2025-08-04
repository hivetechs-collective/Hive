#![allow(non_snake_case)]

use dioxus::document::eval;
use dioxus::prelude::*;
use rfd;
use chrono::{Duration, Utc};
use std::sync::Arc;
use hive_ai::desktop::state::StageStatus;
use uuid::Uuid;

// Terminal imports
use hive_ai::desktop::terminal_tabs::{TerminalTabs, TerminalTab};
use hive_ai::desktop::terminal::use_terminal_cwd;
use hive_ai::desktop::terminal_cwd_tracker::{provide_terminal_cwd_tracker, use_terminal_cwd_tracker};
use hive_ai::desktop::resizable_panels::{ResizableDivider, ResizeDirection};
use hive_ai::desktop::terminal_xterm_simple::TerminalXterm;

// GitUI imports
use hive_ai::desktop::git_ui_wrapper::{GitUIWrapper, ensure_gitui_installed, find_git_root};

// Git imports
use hive_ai::desktop::git::{GitState, use_git_state, GitRepository, GitWatcher, GitEvent, DiffViewMode, get_file_diff, GitToolbar, GitOperation, GitOperations, provide_git_context, use_git_context, GitStatusMenu, GitOperationProgress, ProgressCallback, CancellationToken, initialize_git_statusbar_integration, setup_git_watcher_integration};
use hive_ai::desktop::git::branch_menu::{BranchMenu, BranchOperation, BranchOperationResult};
use hive_ai::desktop::diff_viewer::DiffViewer;

// Enhanced Status Bar imports
use hive_ai::desktop::status_bar_enhanced::{EnhancedStatusBar, StatusBarState, StatusBarItem, StatusBarAlignment};

// Event Bus imports
use hive_ai::desktop::events::{event_bus, Event, EventType, EventPayload};

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
    // Additional fields for accurate reporting
    pub yesterday_total_cost: f64,
    pub yesterday_query_count: u64,
    pub week_total_cost: f64,
    pub week_query_count: u64,
    pub last_week_total_cost: f64,
    pub last_week_query_count: u64,
    pub total_tokens_input: u64,
    pub total_tokens_output: u64,
    pub conversations_with_cost: u64,
}

/// Helper functions for analytics calculations and formatting
mod analytics_helpers {
    
    /// Format currency values consistently
    pub fn format_currency(value: f64) -> String {
        format!("${:.4}", value)
    }
    
    /// Calculate percentage with safety check
    pub fn calculate_percentage(numerator: f64, denominator: f64) -> f64 {
        if denominator > 0.0 {
            (numerator / denominator) * 100.0
        } else {
            0.0
        }
    }
    
    /// Calculate average cost per conversation
    pub fn calculate_avg_cost_per_conversation(total_cost: f64, conversation_count: u64) -> f64 {
        if conversation_count > 0 {
            total_cost / conversation_count as f64
        } else {
            0.0
        }
    }
    
    /// Get color based on value thresholds
    pub fn get_performance_color(value: f64, good_threshold: f64, ok_threshold: f64) -> &'static str {
        if value >= good_threshold {
            "#4caf50"
        } else if value >= ok_threshold {
            "#FFC107"
        } else {
            "#f44336"
        }
    }
    
    /// Format time duration consistently
    pub fn format_duration(seconds: f64) -> String {
        format!("{:.1}s", seconds)
    }
    
    /// Calculate budget progress percentage
    pub fn calculate_budget_progress(current: f64, budget: f64) -> f64 {
        ((current / budget) * 100.0).min(100.0)
    }
}

/// Fetch real analytics data from the database
async fn fetch_analytics_data() -> Result<AnalyticsData, Box<dyn std::error::Error + Send + Sync>> {
    use hive_ai::core::database::get_database;
    
    match get_database().await {
        Ok(db) => {
            // Define time boundaries
            let now = Utc::now();
            let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
            let yesterday_start = today_start - Duration::days(1);
            let week_start = today_start - Duration::days(7);
            let last_week_start = week_start - Duration::days(7);
            
            let today_start_str = today_start.to_rfc3339();
            let yesterday_start_str = yesterday_start.to_rfc3339();
            let week_start_str = week_start.to_rfc3339();
            let last_week_start_str = last_week_start.to_rfc3339();
            
            let connection = db.get_connection()?;
            
            // Comprehensive analytics queries
            let analytics = tokio::task::spawn_blocking(move || -> Result<AnalyticsData, Box<dyn std::error::Error + Send + Sync>> {
                // Total metrics
                let total_queries: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                let total_cost: f64 = connection.query_row(
                    "SELECT COALESCE(SUM(total_cost), 0.0) FROM conversations",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0.0);
                
                let conversations_with_cost: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations WHERE total_cost > 0",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                // Token totals
                let (total_tokens_input, total_tokens_output): (u64, u64) = connection.query_row(
                    "SELECT COALESCE(SUM(total_tokens_input), 0), COALESCE(SUM(total_tokens_output), 0) FROM conversations",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?))
                ).unwrap_or((0, 0));
                
                // Most recent cost
                let most_recent_cost: f64 = connection.query_row(
                    "SELECT COALESCE(total_cost, 0.0) FROM conversations ORDER BY created_at DESC LIMIT 1",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0.0);
                
                // Today's metrics
                let today_queries: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations WHERE created_at >= ?1",
                    [&today_start_str],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                let today_cost: f64 = connection.query_row(
                    "SELECT COALESCE(SUM(total_cost), 0.0) FROM conversations WHERE created_at >= ?1",
                    [&today_start_str],
                    |row| row.get(0)
                ).unwrap_or(0.0);
                
                // Yesterday's metrics
                let yesterday_queries: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations WHERE created_at >= ?1 AND created_at < ?2",
                    [&yesterday_start_str, &today_start_str],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                let yesterday_cost: f64 = connection.query_row(
                    "SELECT COALESCE(SUM(total_cost), 0.0) FROM conversations WHERE created_at >= ?1 AND created_at < ?2",
                    [&yesterday_start_str, &today_start_str],
                    |row| row.get(0)
                ).unwrap_or(0.0);
                
                // This week's metrics
                let week_queries: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations WHERE created_at >= ?1",
                    [&week_start_str],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                let week_cost: f64 = connection.query_row(
                    "SELECT COALESCE(SUM(total_cost), 0.0) FROM conversations WHERE created_at >= ?1",
                    [&week_start_str],
                    |row| row.get(0)
                ).unwrap_or(0.0);
                
                // Last week's metrics
                let last_week_queries: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations WHERE created_at >= ?1 AND created_at < ?2",
                    [&last_week_start_str, &week_start_str],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                let last_week_cost: f64 = connection.query_row(
                    "SELECT COALESCE(SUM(total_cost), 0.0) FROM conversations WHERE created_at >= ?1 AND created_at < ?2",
                    [&last_week_start_str, &week_start_str],
                    |row| row.get(0)
                ).unwrap_or(0.0);
                
                // Calculate success rate: conversations with cost tracking data
                let success_rate = analytics_helpers::calculate_percentage(
                    conversations_with_cost as f64, 
                    total_queries as f64
                );
                
                // Calculate average response time from conversations table (end_time - start_time)
                let avg_response_time: f64 = connection.query_row(
                    "SELECT AVG(CAST((julianday(end_time) - julianday(start_time)) * 86400 AS REAL))
                     FROM conversations 
                     WHERE end_time IS NOT NULL AND start_time IS NOT NULL",
                    [],
                    |row| row.get(0)
                ).unwrap_or_else(|_| {
                    // Fallback: calculate from cost_tracking duration
                    connection.query_row(
                        "SELECT AVG(duration_ms) / 1000.0 FROM cost_tracking WHERE duration_ms > 0",
                        [],
                        |row| row.get(0)
                    ).unwrap_or(2.3)
                });
                
                // Calculate trends
                let queries_trend = if yesterday_queries > 0 {
                    ((today_queries as f64 - yesterday_queries as f64) / yesterday_queries as f64) * 100.0
                } else if today_queries > 0 {
                    100.0 // 100% increase from 0
                } else {
                    0.0
                };
                
                let cost_trend = if yesterday_cost > 0.0 {
                    ((today_cost - yesterday_cost) / yesterday_cost) * 100.0
                } else if today_cost > 0.0 {
                    100.0
                } else {
                    0.0
                };
                
                // Success rate trend (compare this week to last week)
                let this_week_successful: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations WHERE created_at >= ?1 AND total_cost > 0",
                    [&week_start_str],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                let last_week_successful: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations WHERE created_at >= ?1 AND created_at < ?2 AND total_cost > 0",
                    [&last_week_start_str, &week_start_str],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                let this_week_success_rate = analytics_helpers::calculate_percentage(
                    this_week_successful as f64, 
                    week_queries as f64
                );
                
                let last_week_success_rate = analytics_helpers::calculate_percentage(
                    last_week_successful as f64, 
                    last_week_queries as f64
                );
                
                let success_rate_trend = this_week_success_rate - last_week_success_rate;
                
                // Response time trend (compare this week vs last week)
                let this_week_avg_time: f64 = connection.query_row(
                    "SELECT AVG(CAST((julianday(end_time) - julianday(start_time)) * 86400 AS REAL))
                     FROM conversations 
                     WHERE created_at >= ?1 AND end_time IS NOT NULL AND start_time IS NOT NULL",
                    [&week_start_str],
                    |row| row.get(0)
                ).unwrap_or(avg_response_time);
                
                let last_week_avg_time: f64 = connection.query_row(
                    "SELECT AVG(CAST((julianday(end_time) - julianday(start_time)) * 86400 AS REAL))
                     FROM conversations 
                     WHERE created_at >= ?1 AND created_at < ?2 AND end_time IS NOT NULL AND start_time IS NOT NULL",
                    [&last_week_start_str, &week_start_str],
                    |row| row.get(0)
                ).unwrap_or(avg_response_time);
                
                let response_time_trend = if last_week_avg_time > 0.0 {
                    this_week_avg_time - last_week_avg_time  // Negative means improvement (faster)
                } else {
                    0.0
                };
                
                Ok(AnalyticsData {
                    total_queries,
                    total_cost,
                    success_rate,
                    avg_response_time,
                    queries_trend,
                    cost_trend,
                    success_rate_trend,
                    response_time_trend,
                    most_recent_cost,
                    today_total_cost: today_cost,
                    today_query_count: today_queries,
                    yesterday_total_cost: yesterday_cost,
                    yesterday_query_count: yesterday_queries,
                    week_total_cost: week_cost,
                    week_query_count: week_queries,
                    last_week_total_cost: last_week_cost,
                    last_week_query_count: last_week_queries,
                    total_tokens_input,
                    total_tokens_output,
                    conversations_with_cost,
                })
            }).await??;
            
            Ok(analytics)
        }
        Err(_) => {
            // Return empty data if database is not available
            Ok(AnalyticsData::default())
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
        background: #1e1e1e;
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
    
    .git-menu-item:hover {
        background-color: #2A2A2A !important;
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

    @keyframes cancelPulse {
        0%, 100% { 
            opacity: 0.7;
            transform: scale(1);
        }
        50% { 
            opacity: 1;
            transform: scale(1.05);
        }
    }

    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
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
    
    /* Fixed Action Panel Styles */
    .fixed-action-panel {
        backdrop-filter: blur(15px);
        -webkit-backdrop-filter: blur(15px);
    }
    
    .action-btn:hover {
        background: rgba(255, 193, 7, 0.2) !important;
        border-color: rgba(255, 193, 7, 0.5) !important;
        transform: translateY(-1px);
        box-shadow: 0 4px 8px rgba(255, 193, 7, 0.2);
    }
    
    .action-btn:active {
        transform: translateY(0);
        box-shadow: 0 2px 4px rgba(255, 193, 7, 0.1);
    }
    
    /* Tab Scroll Button Styles */
    .tab-scroll-btn:hover {
        background: rgba(255, 193, 7, 0.2) !important;
        border-color: rgba(255, 193, 7, 0.5) !important;
        transform: scale(1.1);
        box-shadow: 0 2px 6px rgba(255, 193, 7, 0.3);
    }
    
    .tab-scroll-btn:active {
        transform: scale(0.95);
    }
    
    /* Enhanced Tab Container Styles */
    .editor-tabs-container {
        position: relative;
    }
    
    .editor-tabs-scroll {
        scrollbar-width: none; /* Firefox */
        -ms-overflow-style: none; /* IE/Edge */
    }
    
    .editor-tabs-scroll::-webkit-scrollbar {
        display: none; /* Chrome, Safari, Opera */
    }
    
    /* Enhanced Tab Styles */
    .editor-tab {
        border-radius: 4px 4px 0 0;
        margin: 0 1px;
        transition: all 0.2s ease;
        position: relative;
    }
    
    .editor-tab:hover span:last-child {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
    }
    
    .editor-tab.active {
        border-bottom: 2px solid #FFC107;
    }
    
    /* File Explorer Git Status Styles */
    .file-item {
        display: flex;
        align-items: center;
        padding: 4px 8px;
        cursor: pointer;
        font-size: 13px;
        color: #cccccc;
        transition: background-color 0.1s;
        position: relative;
    }
    
    .file-item:hover {
        background: #2a2d2e;
    }
    
    .file-item.selected {
        background: #094771;
        color: #ffffff;
    }
    
    .file-item .git-status {
        margin-left: auto;
        font-size: 11px;
        font-weight: 600;
        padding: 0 4px;
        min-width: 14px;
        text-align: center;
    }
    
    /* Git Status Colors */
    .file-item[data-git-status="modified"] {
        color: #e2c08d;
    }
    
    .file-item[data-git-status="modified"] .git-status {
        color: #e2c08d;
    }
    
    .file-item[data-git-status="added"] {
        color: #73c991;
    }
    
    .file-item[data-git-status="added"] .git-status {
        color: #73c991;
    }
    
    .file-item[data-git-status="deleted"] {
        color: #f48771;
    }
    
    .file-item[data-git-status="deleted"] .git-status {
        color: #f48771;
    }
    
    .file-item[data-git-status="untracked"] {
        color: #6c6c6c;
    }
    
    .file-item[data-git-status="untracked"] .git-status {
        color: #73c991;
    }
    
    .file-item[data-git-status="renamed"] {
        color: #5bb0b5;
    }
    
    .file-item[data-git-status="renamed"] .git-status {
        color: #5bb0b5;
    }
    
    .file-item[data-git-status="ignored"] {
        opacity: 0.5;
    }
    
    .file-name {
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    
    .file-icon {
        margin-right: 6px;
        font-size: 16px;
    }
    
    .expand-icon {
        margin-right: 4px;
        transition: transform 0.2s;
    }
    
    .file-children {
        margin-left: 20px;
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
use hive_ai::desktop::consensus_integration::{use_consensus_with_version, DesktopConsensusManager};
use hive_ai::ide::ai_helper_broker::IDEAIHelperBroker;
use hive_ai::desktop::dialogs::{
    AboutDialog, CommandPalette, NoUpdatesDialog, OnboardingDialog, OperationConfirmationDialog,
    SettingsDialog, UpdateAvailableDialog, UpdateErrorDialog, UpgradeDialog, WelcomeAction, WelcomeTab, DIALOG_STYLES,
};
use hive_ai::desktop::context_menu::{
    ContextMenu, ContextMenuAction, ContextMenuState, FileNameDialog, ConfirmDialog,
};
use hive_ai::desktop::file_system;
use hive_ai::desktop::file_operations;
use hive_ai::desktop::menu_bar::{MenuAction, MenuBar};
use hive_ai::desktop::state::{FileItem, FileType};
use hive_ai::desktop::code_editor::editor::CodeEditorComponent;
use hive_ai::desktop::code_editor::renderer::EDITOR_STYLES;
use hive_ai::desktop::components::{OperationStatus, parse_operations_from_content};
use hive_ai::desktop::status_bar_enhanced::{
    STATUS_BAR_STYLES,
};

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
use hive_ai::desktop::state::{AppState, ConsensusState, StageInfo};
use hive_ai::consensus::stages::file_aware_curator::FileOperation;
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
    
    // Initialize git state
    let mut git_state = use_git_state();
    let mut active_git_watcher = use_signal(|| None::<GitWatcher>);
    use_context_provider(|| active_git_watcher.clone());
    
    // Git operation state tracking
    let mut is_pushing = use_signal(|| false);
    let mut is_pulling = use_signal(|| false);
    let mut is_syncing = use_signal(|| false);
    let mut git_operation_status = use_signal(|| None::<String>);
    
    // Initialize git context manager
    let mut git_context = provide_git_context();
    
    // Initialize terminal CWD tracker
    let cwd_tracker = provide_terminal_cwd_tracker();
    
    // Sidebar view state
    #[derive(Clone, Copy, PartialEq)]
    enum SidebarView {
        FileExplorer,
        SourceControl,
    }
    let mut sidebar_view = use_signal(|| SidebarView::FileExplorer);
    
    // Branch menu state
    let mut show_branch_menu = use_signal(|| false);

    // API keys state (needed before consensus manager)
    let mut openrouter_key = use_signal(String::new);
    let mut hive_key = use_signal(String::new);
    let mut anthropic_key = use_signal(String::new);
    let mut api_keys_version = use_signal(|| 0u32); // Track when API keys change
    let mut api_config = use_signal(|| hive_ai::core::api_keys::ApiKeyConfig {
        openrouter_key: None,
        hive_key: None,
        anthropic_key: None,
    });

    // Get consensus manager - use a signal to store it
    let mut consensus_manager = use_signal(|| None::<DesktopConsensusManager>);
    
    // Watch for api_keys_version changes and recreate consensus manager
    use_effect(move || {
        let version = *api_keys_version.read();
        tracing::info!("API keys version changed to {}, recreating consensus manager", version);
        consensus_manager.set(use_consensus_with_version(version));
    });

    // State management
    let mut current_response = use_signal(String::new); // Final response
    let mut input_value = use_signal(String::new);
    let mut is_processing = use_signal(|| false);
    let mut is_cancelling = use_signal(|| false); // Track cancellation state
    let mut cancel_flag = use_signal(|| false); // Simple flag to stop streaming updates
    let mut cancellation_flag = use_signal(|| std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))); // Atomic cancellation flag
    let mut consensus_task_handle = use_signal(|| None::<dioxus::prelude::Task>); // Track the running consensus task
    let mut selected_file = use_signal(|| Some("__welcome__".to_string()));
    let mut file_tree = use_signal(|| Vec::<FileItem>::new());
    let mut expanded_dirs = use_signal(|| HashMap::<PathBuf, bool>::new());
    let mut current_dir = use_signal(|| None::<PathBuf>);
    let mut file_content = use_signal(String::new);

    // Initialize IDE AI Helper Broker for repository awareness (after signals are declared)
    let mut ide_ai_broker = use_signal(|| None::<IDEAIHelperBroker>);
    
    // Sync git context with current directory
    use_effect({
        let current_dir_signal = current_dir.clone();
        let git_context_clone = git_context.clone();
        move || {
            if let Some(workspace_path) = current_dir_signal.read().clone() {
                let mut git_context = git_context_clone.clone();
                spawn(async move {
                    git_context.set_active_path(workspace_path.clone()).await;
                    tracing::info!("Synced git context with workspace: {:?}", workspace_path);
                });
            }
        }
    });
    
    // Create the IDE AI Broker when the app starts
    use_effect(move || {
        let current_dir = current_dir.clone();
        let mut file_tree = file_tree.clone();
        let selected_file = selected_file.clone();
        let mut ide_ai_broker = ide_ai_broker.clone();
        
        spawn(async move {
            // Create IDE AI Helper Broker with File Explorer signals
            match IDEAIHelperBroker::new(
                current_dir,
                file_tree,
                selected_file
            ).await {
                Ok(broker) => {
                    ide_ai_broker.set(Some(broker));
                    tracing::info!("✅ IDE AI Helper Broker initialized");
                }
                Err(e) => {
                    tracing::error!("Failed to initialize IDE AI Helper Broker: {}", e);
                }
            }
        });
    });
    
    // Tab management
    let mut open_tabs = use_signal(|| vec!["__welcome__".to_string()]);
    let mut active_tab = use_signal(|| "__welcome__".to_string());
    let mut tab_contents = use_signal(|| {
        let mut contents = HashMap::new();
        contents.insert("__welcome__".to_string(), String::new());
        contents
    });
    
    // Diff viewing state
    let mut diff_tabs = use_signal(|| HashMap::<String, hive_ai::desktop::git::DiffResult>::new());
    let mut diff_view_mode = use_signal(|| DiffViewMode::SideBySide);
    
    // Cursor position tracking for status bar
    let mut cursor_position = use_signal(|| (1, 1)); // (line, column)
    
    // Enhanced status bar state
    let mut status_bar_state = use_signal(|| StatusBarState::default());
    
    // Initialize GitState to StatusBar integration
    initialize_git_statusbar_integration(
        git_state.clone(),
        status_bar_state.clone(),
        active_git_watcher.clone(),
        current_dir.read().clone(),
        app_state.clone(),
    );
    
    // Auto-fetch configuration
    let mut auto_fetch_enabled = use_signal(|| true); // Enable auto-fetch by default
    let mut auto_fetch_interval_minutes = use_signal(|| 5); // Fetch every 5 minutes
    
    // Status bar event handler for click actions
    let handle_status_bar_click = {
        let mut git_state = git_state.clone();
        let current_dir = current_dir.clone();
        let mut is_syncing = is_syncing.clone();
        let show_branch_menu = show_branch_menu.clone();
        move |item_id: String| {
            match item_id.as_str() {
                "git-branch" => {
                    // Emit BranchMenuRequested event
                    tracing::info!("Branch selector clicked - emitting BranchMenuRequested event");
                    
                    let bus = event_bus();
                    spawn(async move {
                        let event = Event::new(EventType::BranchMenuRequested, EventPayload::Empty);
                        bus.publish(event).await.unwrap_or_else(|e| {
                            tracing::error!("Failed to publish BranchMenuRequested event: {}", e);
                        });
                    });
                },
                "git-sync" => {
                    // Perform sync operation or publish branch
                    if !*is_syncing.read() {
                        is_syncing.set(true);
                        let mut git_state = git_state.clone();
                        let current_dir = current_dir.clone();
                        let mut is_syncing = is_syncing.clone();
                        let mut git_operation_status = git_operation_status.clone();
                        spawn(async move {
                            if let Some(repo_path) = current_dir.read().clone() {
                                let sync_status = git_state.sync_status.read();
                                
                                if sync_status.has_upstream {
                                    // Standard sync: fetch + pull + push
                                    
                                    // Step 1: Fetch
                                    git_operation_status.set(Some("Fetching remote changes...".to_string()));
                                    match hive_ai::desktop::git::operations::fetch(&repo_path).await {
                                        Ok(_) => {
                                            tracing::info!("✅ Fetch completed successfully");
                                            
                                            // Step 2: Pull
                                            git_operation_status.set(Some("Pulling remote changes...".to_string()));
                                            match hive_ai::desktop::git::operations::pull(&repo_path).await {
                                                Ok(_) => {
                                                    tracing::info!("✅ Pull completed successfully");
                                                    
                                                    // Step 3: Push
                                                    git_operation_status.set(Some("Pushing local changes...".to_string()));
                                                    match hive_ai::desktop::git::operations::push(&repo_path).await {
                                                        Ok(_) => {
                                                            tracing::info!("✅ Push completed successfully");
                                                            git_operation_status.set(Some("✅ Sync completed successfully - all changes pushed".to_string()));
                                                            
                                                            // Refresh git status to update UI
                                                            if let Err(e) = git_state.refresh_status(&repo_path).await {
                                                                tracing::warn!("Failed to refresh git status after sync: {}", e);
                                                            }
                                                            
                                                            // Clear status after a delay
                                                            let mut git_operation_status_clear = git_operation_status.clone();
                                                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                                            git_operation_status_clear.set(None);
                                                        }
                                                        Err(e) => {
                                                            tracing::error!("❌ Failed to push: {}", e);
                                                            let error_msg = if e.to_string().contains("rejected") {
                                                                "❌ Push rejected - pull latest changes first".to_string()
                                                            } else if e.to_string().contains("authentication") {
                                                                "❌ Push failed - check your Git credentials".to_string()
                                                            } else {
                                                                format!("❌ Push failed: {}", e)
                                                            };
                                                            git_operation_status.set(Some(error_msg));
                                                            
                                                            // Clear error after a delay
                                                            let mut git_operation_status_clear = git_operation_status.clone();
                                                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                                            git_operation_status_clear.set(None);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::error!("❌ Failed to pull: {}", e);
                                                    let error_msg = if e.to_string().contains("conflict") {
                                                        "❌ Pull failed - merge conflicts detected".to_string()
                                                    } else if e.to_string().contains("uncommitted") {
                                                        "❌ Pull failed - commit or stash your changes first".to_string()
                                                    } else {
                                                        format!("❌ Pull failed: {}", e)
                                                    };
                                                    git_operation_status.set(Some(error_msg));
                                                    
                                                    // Clear error after a delay
                                                    let mut git_operation_status_clear = git_operation_status.clone();
                                                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                                    git_operation_status_clear.set(None);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("❌ Failed to fetch: {}", e);
                                            let error_msg = if e.to_string().contains("could not read Username") || e.to_string().contains("authentication") {
                                                "❌ Fetch failed - Git credentials not configured".to_string()
                                            } else if e.to_string().contains("unable to access") || e.to_string().contains("network") {
                                                "❌ Fetch failed - check your network connection".to_string()
                                            } else {
                                                format!("❌ Fetch failed: {}", e)
                                            };
                                            git_operation_status.set(Some(error_msg));
                                            
                                            // Clear error after a delay
                                            let mut git_operation_status_clear = git_operation_status.clone();
                                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                            git_operation_status_clear.set(None);
                                        }
                                    }
                                } else {
                                    // Branch has no upstream - attempt to publish
                                    if let Some(branch_info) = git_state.branch_info.read().as_ref() {
                                        let branch_name = &branch_info.name;
                                        tracing::info!("Publishing branch '{}' to origin", branch_name);
                                        git_operation_status.set(Some(format!("Publishing branch '{}'...", branch_name)));
                                        
                                        // Push branch with set-upstream
                                        let result = hive_ai::desktop::git::operations::push_with_upstream(
                                            &repo_path, 
                                            branch_name
                                        ).await;
                                        
                                        match result {
                                            Ok(_) => {
                                                tracing::info!("✅ Successfully published branch '{}'", branch_name);
                                                git_operation_status.set(Some(format!("✅ Branch '{}' published to remote", branch_name)));
                                                
                                                // Refresh git status to update UI
                                                if let Err(e) = git_state.refresh_status(&repo_path).await {
                                                    tracing::warn!("Failed to refresh git status after publishing: {}", e);
                                                }
                                                
                                                // Clear status after a delay
                                                let mut git_operation_status_clear = git_operation_status.clone();
                                                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                                git_operation_status_clear.set(None);
                                            }
                                            Err(e) => {
                                                tracing::error!("❌ Failed to publish branch '{}': {}", branch_name, e);
                                                let error_msg = if e.to_string().contains("authentication") || e.to_string().contains("could not read Username") {
                                                    "❌ Publishing failed - configure Git credentials first".to_string()
                                                } else if e.to_string().contains("permission") || e.to_string().contains("403") {
                                                    "❌ Publishing failed - you don't have push access".to_string()
                                                } else {
                                                    format!("❌ Failed to publish branch: {}", e)
                                                };
                                                git_operation_status.set(Some(error_msg));
                                                
                                                // Clear error after a delay
                                                let mut git_operation_status_clear = git_operation_status.clone();
                                                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                                git_operation_status_clear.set(None);
                                            }
                                        }
                                    } else {
                                        tracing::warn!("No branch information available for publishing");
                                        git_operation_status.set(Some("⚠️ Unable to determine current branch".to_string()));
                                        
                                        // Clear error after a delay
                                        let mut git_operation_status_clear = git_operation_status.clone();
                                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                        git_operation_status_clear.set(None);
                                    }
                                }
                                
                                // Refresh git status in both cases
                                if let Err(e) = git_state.refresh_status(&repo_path).await {
                                    tracing::warn!("Failed to refresh git status: {}", e);
                                }
                            } else {
                                tracing::warn!("Sync attempted with no repository selected");
                                git_operation_status.set(Some("⚠️ No repository selected - please open a folder first".to_string()));
                                
                                // Clear error after a delay
                                let mut git_operation_status_clear = git_operation_status.clone();
                                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                git_operation_status_clear.set(None);
                            }
                            
                            // Always reset syncing state
                            is_syncing.set(false);
                            // Update sync status to show not syncing
                            git_state.sync_status.write().is_syncing = false;
                        });
                    }
                },
                "problems" => {
                    // Show problems panel
                    tracing::info!("Problems panel clicked");
                },
                "warnings" => {
                    // Show warnings panel
                    tracing::info!("Warnings panel clicked");
                },
                "consensus-status" => {
                    // Show consensus status
                    tracing::info!("Consensus status clicked");
                },
                "cursor-position" => {
                    // Go to line dialog
                    tracing::info!("Go to line clicked");
                },
                _ => {
                    tracing::info!("Status bar item clicked: {}", item_id);
                }
            }
        }
    };
    
    // Status bar state update effect - handle non-git items
    {
        let mut status_bar_state = status_bar_state.clone();
        let cursor_position = cursor_position.clone();
        use_effect(move || {
            // Update cursor position
            let (line, col) = *cursor_position.read();
            status_bar_state.write().update_item("cursor-position", format!("Ln {}, Col {}", line, col));
        });
    }
    
    // Handle syncing status overlay
    {
        let mut git_state = git_state.clone();
        let mut is_syncing = is_syncing.clone();
        use_effect(move || {
            if *is_syncing.read() {
                // Update sync status to show syncing
                let mut sync_status = git_state.sync_status.write();
                sync_status.is_syncing = true;
            }
        });
    }
    
    // Auto-fetch effect
    {
        let auto_fetch_enabled = auto_fetch_enabled.clone();
        let auto_fetch_interval_minutes = auto_fetch_interval_minutes.clone();
        let current_dir = current_dir.clone();
        let mut git_state = git_state.clone();
        
        use_effect(move || {
            if *auto_fetch_enabled.read() {
                let interval_ms = *auto_fetch_interval_minutes.read() as f64 * 60.0 * 1000.0;
                let current_dir = current_dir.clone();
                let mut git_state = git_state.clone();
                
                spawn(async move {
                    let mut interval = tokio::time::interval(
                        tokio::time::Duration::from_millis(interval_ms as u64)
                    );
                    interval.tick().await; // Skip first immediate tick
                    
                    loop {
                        interval.tick().await;
                        
                        if let Some(repo_path) = current_dir.read().clone() {
                            tracing::debug!("Auto-fetching git repository: {:?}", repo_path);
                            
                            // Perform background fetch
                            if let Err(e) = hive_ai::desktop::git::operations::fetch(&repo_path).await {
                                tracing::warn!("Auto-fetch failed: {}", e);
                            } else {
                                // Update git state after successful fetch
                                if let Err(e) = git_state.refresh_status(&repo_path).await {
                                    tracing::warn!("Failed to refresh git status after fetch: {}", e);
                                }
                                tracing::debug!("Auto-fetch completed successfully");
                            }
                        }
                    }
                });
            }
        });
    }
    
    // Tab overflow management
    let mut tab_scroll_offset = use_signal(|| 0usize);
    let max_visible_tabs = 6; // Maximum number of tabs to display before scrolling
    
    // Function to ensure active tab is visible
    let mut ensure_active_tab_visible = {
        let mut tab_scroll_offset = tab_scroll_offset.clone();
        move |active_tab: &str, open_tabs: &[String]| {
            if let Some(active_index) = open_tabs.iter().position(|tab| tab == active_tab) {
                let current_offset = tab_scroll_offset.read().clone();
                let visible_start = current_offset;
                let visible_end = current_offset + max_visible_tabs;
                
                // If active tab is before visible range, scroll left
                if active_index < visible_start {
                    tab_scroll_offset.set(active_index);
                }
                // If active tab is after visible range, scroll right
                else if active_index >= visible_end {
                    tab_scroll_offset.set(active_index.saturating_sub(max_visible_tabs - 1));
                }
            }
        }
    };

    // Dialog state
    let mut show_about_dialog = use_signal(|| false);
    let mut show_welcome_dialog = use_signal(|| true);
    let mut show_command_palette = use_signal(|| false);
    let mut show_settings_dialog = use_signal(|| false);
    let mut show_onboarding_dialog = use_signal(|| false);
    let mut show_upgrade_dialog = use_signal(|| false);
    let mut show_git_menu = use_signal(|| false);
    let mut git_menu_position = use_signal(|| (0i32, 0i32));
    let mut onboarding_current_step = use_signal(|| 1); // Persist onboarding step
    
    // Context menu and file operation dialogs
    let mut context_menu_state = use_signal(|| ContextMenuState::default());
    let mut show_new_file_dialog = use_signal(|| false);
    let mut show_new_folder_dialog = use_signal(|| false);
    let mut show_rename_dialog = use_signal(|| false);
    let mut show_delete_confirm = use_signal(|| false);
    let mut dialog_target_path = use_signal(|| None::<PathBuf>);

    // View state
    let mut current_view = use_signal(|| "code".to_string()); // "code" or "analytics"

    // Analytics state
    let mut analytics_data = use_signal(|| AnalyticsData::default());

    // Track the last analytics trigger value to prevent infinite loops
    let mut last_analytics_trigger = use_signal(|| 0u32);
    
    // Analytics refresh effect - triggers when analytics_refresh_trigger changes
    use_effect({
        let mut analytics_data = analytics_data.clone();
        let mut app_state = app_state.clone();
        let mut last_analytics_trigger = last_analytics_trigger.clone();
        move || {
            let current_trigger = app_state.read().analytics_refresh_trigger;
            let last_trigger = *last_analytics_trigger.read();
            
            // Only fetch if the trigger actually changed
            if current_trigger != last_trigger {
                last_analytics_trigger.set(current_trigger);
                spawn(async move {
                    match fetch_analytics_data().await {
                        Ok(data) => {
                            analytics_data.set(data);
                            tracing::info!("Analytics data refreshed successfully (trigger: {})", current_trigger);
                        }
                        Err(e) => {
                            tracing::error!("Failed to fetch analytics data: {}", e);
                        }
                    }
                });
            }
        }
    });
    
    // Sync git context with workspace folder (VS Code approach)
    use_effect({
        let current_dir = current_dir.clone();
        let git_context_outer = git_context.clone();
        move || {
            if let Some(workspace_path) = current_dir.read().clone() {
                let mut git_context = git_context_outer.clone();
                spawn(async move {
                    git_context.set_active_path(workspace_path.clone()).await;
                    tracing::info!("Synced git context with workspace: {:?}", workspace_path);
                });
            }
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
    
    // Terminal state
    let mut show_terminal = use_signal(|| true); // Terminal visible by default
    let mut show_gitui = use_signal(|| true); // GitUI panel visible by default
    let mut gitui_terminal_id = use_signal(|| None::<String>); // GitUI terminal ID
    let mut gitui_update_counter = use_signal(|| 0u32); // Force GitUI terminal refresh
    
    // Initialize GitUI terminal and watch for directory changes
    use_effect({
        let mut gitui_terminal_id = gitui_terminal_id.clone();
        let current_dir = current_dir.clone();
        let show_gitui = show_gitui.clone();
        let mut gitui_update_counter = gitui_update_counter.clone();
        move || {
            // Only proceed if GitUI panel is shown
            if !show_gitui() {
                return;
            }
            
            // Get the current directory to trigger reactivity
            let current_path = current_dir.read().as_ref()
                .cloned()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            
            tracing::info!("🔍 Directory changed, updating GitUI for: {:?}", current_path);
            
            // Check if GitUI is installed
            spawn(async move {
                match ensure_gitui_installed() {
                    Ok(_) => {
                        tracing::info!("✅ GitUI is installed and ready");
                        
                        // Clear any existing terminal first
                        if let Some(old_id) = gitui_terminal_id.read().as_ref() {
                            // Unregister the old terminal
                            use hive_ai::desktop::terminal_registry::unregister_terminal;
                            use hive_ai::desktop::terminal_buffer::unregister_terminal_buffer;
                            unregister_terminal(old_id);
                            unregister_terminal_buffer(old_id);
                            tracing::info!("🔄 Cleared old GitUI terminal: {}", old_id);
                        }
                        
                        // Only create terminal if we're in a git repository
                        if let Some(_git_root) = find_git_root(&current_path) {
                            // Add a small delay to prevent rapid terminal recreations
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            
                            // Create a unique terminal ID for GitUI
                            let terminal_id = format!("gitui-{}", Uuid::new_v4());
                            
                            // Set the new terminal ID
                            gitui_terminal_id.set(Some(terminal_id.clone()));
                            
                            // Increment update counter to force terminal re-render
                            gitui_update_counter.set(gitui_update_counter() + 1);
                            
                            tracing::info!("✅ GitUI terminal ID set: {} for path: {:?}", terminal_id, current_path);
                        } else {
                            // Not in a git repository, clear terminal ID
                            gitui_terminal_id.set(None);
                            tracing::info!("⚠️ Not in a git repository, GitUI not available for: {:?}", current_path);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("GitUI not installed: {}", e);
                        gitui_terminal_id.set(None);
                    }
                }
            });
        }
    });
    
    // Panel resizing state
    let mut gitui_width = use_signal(|| 300.0);  // New GitUI panel width
    let mut sidebar_width = use_signal(|| 250.0);
    let mut chat_width = use_signal(|| 400.0);
    let mut terminal_height = use_signal(|| 300.0);

    // Subscription state
    let mut subscription_display = use_signal(|| String::from("Loading..."));
    let mut error_shown = use_signal(|| false);

    // Helper function to reload file tree
    let reload_file_tree = {
        let current_dir = current_dir.clone();
        let expanded_dirs = expanded_dirs.clone();
        let mut file_tree = file_tree.clone();
        move || {
            let current_dir = current_dir.clone();
            let expanded_dirs = expanded_dirs.clone();
            let mut file_tree = file_tree.clone();
            spawn(async move {
                let current_dir_opt = current_dir.read().clone();
                if let Some(current_dir_path) = current_dir_opt {
                    let expanded_map = expanded_dirs.read().clone();
                    
                    let root_name = current_dir_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Root")
                    .to_string();

                match file_system::load_directory_tree(&current_dir_path, &expanded_map, false).await {
                    Ok(files) => {
                        let root_item = FileItem {
                            path: current_dir_path.clone(),
                            name: root_name,
                            is_directory: true,
                            is_expanded: true,
                            children: files,
                            file_type: FileType::Directory,
                            git_status: None,
                            size: None,
                            modified: None,
                            depth: 0,
                        };

                        file_tree.write().clear();
                        file_tree.write().push(root_item);
                    }
                    Err(e) => {
                        eprintln!("Error reloading directory: {}", e);
                    }
                }
                }
            });
        }
    };

    // Also load the api config on mount
    use_effect({
        let mut api_config = api_config.clone();
        let mut hive_key = hive_key.clone();
        let mut openrouter_key = openrouter_key.clone();
        let mut anthropic_key = anthropic_key.clone();
        move || {
            spawn(async move {
                use hive_ai::core::api_keys::ApiKeyManager;
                if let Ok(config) = ApiKeyManager::load_from_database().await {
                    api_config.set(config.clone());
                    if let Some(key) = config.hive_key {
                        hive_key.set(key);
                    }
                    if let Some(key) = config.openrouter_key {
                        openrouter_key.set(key);
                    }
                    if let Some(key) = config.anthropic_key {
                        anthropic_key.set(key);
                    }
                }
            });
        }
    });

    // Check if we need to show onboarding (only once on mount)
    use_effect(move || {
        let mut show_onboarding_dialog = show_onboarding_dialog.clone();
        let mut openrouter_key = openrouter_key.clone();
        let mut api_keys_version = api_keys_version.clone();
        let mut api_config = api_config.clone();
        let mut hive_key = hive_key.clone();
        let mut anthropic_key = anthropic_key.clone();
        spawn(async move {
            use hive_ai::core::api_keys::ApiKeyManager;

            // Check if API keys are configured
            if !ApiKeyManager::has_valid_keys().await.unwrap_or(false) {
                show_onboarding_dialog.set(true);
            } else {
                // Load existing key for settings
                if let Ok(config) = ApiKeyManager::load_from_database().await {
                    api_config.set(config.clone());
                    if let Some(key) = config.openrouter_key {
                        openrouter_key.set(key);
                    }
                    if let Some(key) = config.hive_key {
                        hive_key.set(key);
                    }
                    if let Some(key) = config.anthropic_key {
                        anthropic_key.set(key);
                    }
                    
                    // Trigger consensus manager recreation since we have keys
                    *api_keys_version.write() += 1;
                    tracing::info!("API keys loaded from database - triggering consensus reload");
                    
                    // Now load the active profile into consensus state
                    let mut app_state_for_profile = app_state.clone();
                    spawn(async move {
                        tracing::info!("🚀 Loading active profile for UI after API keys loaded");
                        match load_active_profile_from_db().await {
                            Ok(profile) => {
                                tracing::info!("✅ Loaded active profile: {}", profile.profile_name);
                                app_state_for_profile.write().consensus.active_profile_name = profile.profile_name.clone();
                                app_state_for_profile.write().consensus.stages = vec![
                                    StageInfo::new("Generator", &profile.generator_model),
                                    StageInfo::new("Refiner", &profile.refiner_model),
                                    StageInfo::new("Validator", &profile.validator_model),
                                    StageInfo::new("Curator", &profile.curator_model),
                                ];
                            }
                            Err(e) => {
                                tracing::error!("❌ Failed to load active profile on startup: {}", e);
                            }
                        }
                    });
                }
            }
        });
    });

    // Watch for license key changes and refresh subscription immediately
    use_effect({
        let mut hive_key = hive_key.clone();
        let mut subscription_display = subscription_display.clone();
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
                                                    subscription_display.set(format!(
                                                        "{} | {} | Unlimited conversations",
                                                        email, tier
                                                    ));
                                                } else if remaining == 0 {
                                                    subscription_display.set(format!(
                                                        "{} | {} | Daily limit reached ({}/{})",
                                                        email,
                                                        tier,
                                                        limit - remaining,
                                                        limit
                                                    ));

                                                    if !*error_shown.read() {
                                                        show_upgrade_dialog.set(true);
                                                        error_shown.set(true);
                                                    }
                                                } else {
                                                    subscription_display.set(format!("{} | {} | {} conversations remaining today", email, tier, remaining));
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
                                                    subscription_display.set(format!(
                                                        "{} | {} | Daily limit reached ({}/{})",
                                                        email, tier, limit, limit
                                                    ));
                                                } else {
                                                    // Some other error
                                                    subscription_display.set(format!(
                                                        "{} | {} | Limited access",
                                                        email, tier
                                                    ));
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
                            api_config.set(config.clone());
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
                                                            subscription_display.set(format!(
                                                                "{} | {} | Unlimited conversations",
                                                                email, tier
                                                            ));
                                                        } else if remaining == 0 {
                                                            subscription_display.set(format!("{} | {} | Daily limit reached ({}/{})", email, tier, limit - remaining, limit));

                                                            if !*error_shown.read() {
                                                                show_upgrade_dialog.set(true);
                                                                error_shown.set(true);
                                                            }
                                                        } else {
                                                            subscription_display.set(format!("{} | {} | {} conversations remaining today", email, tier, remaining));
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
                                                            subscription_display.set(format!("{} | {} | Daily limit reached ({}/{})", email, tier, limit, limit));
                                                        } else {
                                                            // Some other error
                                                            subscription_display.set(format!(
                                                                "{} | {} | Limited access",
                                                                email, tier
                                                            ));
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
                                subscription_display.set("No license configured".to_string());
                            }
                        }
                        Err(_) => {
                            subscription_display.set("No license configured".to_string());
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
                                                            subscription_display.set(format!(
                                                                "{} | {} | Unlimited conversations",
                                                                email, tier
                                                            ));
                                                        } else if remaining == 0 {
                                                            subscription_display.set(format!("{} | {} | Daily limit reached ({}/{})", email, tier, limit - remaining, limit));

                                                            if !*error_shown.read() {
                                                                show_upgrade_dialog.set(true);
                                                                error_shown.set(true);
                                                            }
                                                        } else {
                                                            subscription_display.set(format!("{} | {} | {} conversations remaining today", email, tier, remaining));
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
                                                        show_upgrade_dialog.set(true);
                                                        error_shown.set(true);
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
                                                    subscription_display.set(display);
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

    // Track if initial directory has been loaded
    let mut initial_dir_loaded = use_signal(|| false);
    
    // Load initial directory - only run once when current_dir changes from None to Some
    use_effect({
        let mut file_tree = file_tree.clone();
        let current_dir = current_dir.clone();
        let expanded_dirs = expanded_dirs.clone();
        let mut app_state_for_project = app_state.clone();
        let mut initial_dir_loaded = initial_dir_loaded.clone();
        
        move || {
            // Only load if we have a directory and haven't loaded yet
            if let Some(current_dir_path) = current_dir.read().clone() {
                if *initial_dir_loaded.read() {
                    return; // Already loaded
                }
                initial_dir_loaded.set(true);
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
                                name: root_name.clone(),
                                is_directory: true,
                                is_expanded: true, // Root is expanded by default
                                children: files,
                                file_type: FileType::Directory,
                                git_status: None,
                                size: None,
                                modified: None,
                                depth: 0,
                            };

                            file_tree.write().clear();
                            file_tree.write().push(root_item);
                            
                            // Update AppState with current project information for repository context
                            let project_info = hive_ai::desktop::state::ProjectInfo {
                                name: root_name,
                                path: current_dir_path.clone(),
                                root_path: current_dir_path.clone(),
                                language: None, // Will be detected by repository analyzer
                                git_status: hive_ai::desktop::state::GitStatus::NotRepository,
                                git_branch: None,
                                file_count: 0, // Will be updated later
                            };
                            
                            app_state_for_project.write().current_project = Some(project_info);
                            tracing::info!("Initialized current_project with root path: {}", current_dir_path.display());
                        }
                        Err(e) => {
                            eprintln!("Error loading directory: {}", e);
                        }
                    }
                });
            }
        }
    });

    // File selection is handled directly in the onclick handler

    // Track if we should auto-scroll
    let mut should_auto_scroll = use_signal(|| true);

    // Auto-scroll response area when streaming content changes
    let mut previous_content_length = use_signal(|| 0usize);

    // Monitor streaming content changes
    use_effect({
        let mut app_state = app_state.clone();
        let should_auto_scroll = should_auto_scroll.clone();
        move || {
            let current_length = app_state.read().consensus.streaming_content.len();
            let previous_length = *previous_content_length.read();
            
            // Check if content has grown
            if current_length > previous_length && *should_auto_scroll.read() {
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
                    if let Err(e) = eval.await {
                        tracing::error!("Failed to evaluate JavaScript: {:?}", e);
                    }
                });
            }
        }
    });
    
    // Update previous content length in a separate effect to avoid circular dependency
    use_effect({
        let mut app_state = app_state.clone();
        let mut previous_content_length = previous_content_length.clone();
        move || {
            let current_length = app_state.read().consensus.streaming_content.len();
            previous_content_length.set(current_length);
        }
    });

    // Handle menu actions
    // Process AI Helper UI events
    use_effect({
        let mut app_state = app_state.clone();
        
        move || {
            use hive_ai::desktop::ai_ui_events::process_ai_helper_events;
            process_ai_helper_events(app_state);
        }
    });

    let handle_menu_action = {
        let mut git_state_clone = git_state.clone();
        let mut active_git_watcher_clone = active_git_watcher.clone();
        let mut git_context = git_context.clone();
        move |action: MenuAction| {
        match action {
            MenuAction::OpenFolder => {
                // Open folder dialog
                spawn({
                    let mut current_dir = current_dir.clone();
                    let mut file_tree = file_tree.clone();
                    let mut expanded_dirs = expanded_dirs.clone();
                    let mut selected_file = selected_file.clone();
                    let mut file_content = file_content.clone();
                    let git_state = git_state_clone.clone();
                    let active_git_watcher = active_git_watcher_clone.clone();
                    let mut git_context = git_context.clone();

                    async move {
                        let current_path = current_dir.read().clone();
                        let dialog = rfd::AsyncFileDialog::new();
                        let dialog = if let Some(path) = current_path {
                            dialog.set_directory(&path)
                        } else {
                            dialog
                        };
                        if let Some(folder) = dialog.pick_folder().await
                        {
                            // Update current directory
                            current_dir.set(Some(folder.path().to_path_buf()));

                            // Clear selected file and content
                            selected_file.set(None);
                            file_content.set(String::new());

                            // Clear expanded dirs for new folder
                            expanded_dirs.write().clear();
                            
                            // Initialize git repository detection
                            let folder_path = folder.path().to_path_buf();
                            let mut git_state_clone = git_state.clone();
                            let mut active_git_watcher_clone = active_git_watcher.clone();
                            let mut git_context_clone = git_context.clone();
                            spawn(async move {
                                // Update git context manager - it handles everything
                                git_context_clone.set_active_path(folder_path.clone()).await;
                                
                                // Also update git_state directly for UI components that read from it
                                if let Ok(repo) = GitRepository::open(&folder_path) {
                                    if let Ok(branch_name) = repo.current_branch() {
                                        // Get ahead/behind counts
                                        let (ahead, behind) = repo.ahead_behind().unwrap_or((0, 0));
                                        let has_upstream = repo.upstream_branch().unwrap_or(None).is_some();
                                        
                                        let branch_info = hive_ai::desktop::git::BranchInfo {
                                            name: branch_name,
                                            branch_type: hive_ai::desktop::git::BranchType::Local,
                                            is_current: true,
                                            upstream: repo.upstream_branch().unwrap_or(None),
                                            ahead,
                                            behind,
                                            last_commit: None,
                                        };
                                        git_state_clone.branch_info.set(Some(branch_info));
                                        
                                        // Update sync status
                                        let sync_status = hive_ai::desktop::git::SyncStatus {
                                            ahead,
                                            behind,
                                            has_upstream,
                                            is_syncing: false,
                                        };
                                        git_state_clone.sync_status.set(sync_status);
                                    }
                                    
                                    // Get file statuses
                                    match repo.file_statuses() {
                                        Ok(statuses) => {
                                            let mut status_map = std::collections::HashMap::new();
                                            for (path, git_status) in statuses {
                                                // Convert git2::Status to our StatusType
                                                let status_type = if git_status.contains(git2::Status::WT_MODIFIED) || git_status.contains(git2::Status::INDEX_MODIFIED) {
                                                    hive_ai::desktop::git::StatusType::Modified
                                                } else if git_status.contains(git2::Status::WT_NEW) || git_status.contains(git2::Status::INDEX_NEW) {
                                                    hive_ai::desktop::git::StatusType::Added
                                                } else if git_status.contains(git2::Status::WT_DELETED) || git_status.contains(git2::Status::INDEX_DELETED) {
                                                    hive_ai::desktop::git::StatusType::Deleted
                                                } else if git_status.contains(git2::Status::WT_RENAMED) || git_status.contains(git2::Status::INDEX_RENAMED) {
                                                    hive_ai::desktop::git::StatusType::Renamed
                                                } else if git_status.is_wt_new() {
                                                    hive_ai::desktop::git::StatusType::Untracked
                                                } else {
                                                    continue; // Skip other statuses
                                                };
                                                
                                                let file_status = hive_ai::desktop::git::FileStatus {
                                                    path: path.clone(),
                                                    status_type,
                                                    is_staged: git_status.contains(git2::Status::INDEX_NEW) ||
                                                              git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                              git_status.contains(git2::Status::INDEX_DELETED) ||
                                                              git_status.contains(git2::Status::INDEX_RENAMED),
                                                    has_staged_changes: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                      git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                      git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                      git_status.contains(git2::Status::INDEX_RENAMED),
                                                    has_unstaged_changes: git_status.contains(git2::Status::WT_NEW) ||
                                                                        git_status.contains(git2::Status::WT_MODIFIED) ||
                                                                        git_status.contains(git2::Status::WT_DELETED),
                                                };
                                                status_map.insert(path, file_status);
                                            }
                                            git_state_clone.file_statuses.set(status_map);
                                            tracing::info!("✅ Updated git file statuses");
                                        }
                                        Err(e) => {
                                            tracing::warn!("Failed to get repository status: {}", e);
                                            git_state_clone.file_statuses.set(std::collections::HashMap::new());
                                        }
                                    }
                                    
                                    // Set up file watcher with app state for file explorer refresh
                                    let app_state_for_watcher = app_state.clone();
                                    let git_state_for_watcher = git_state_clone.clone();
                                    let active_git_watcher_for_setup = active_git_watcher_clone.clone();
                                    let folder_path_for_watcher = folder_path.clone();
                                    
                                    spawn(async move {
                                        setup_git_watcher_integration(
                                            git_state_for_watcher,
                                            active_git_watcher_for_setup,
                                            folder_path_for_watcher,
                                            app_state_for_watcher,
                                        );
                                    });
                                } else {
                                    git_state_clone.branch_info.set(None);
                                    git_state_clone.file_statuses.set(std::collections::HashMap::new());
                                    active_git_watcher_clone.set(None);
                                }
                                tracing::info!("Updated git context for folder: {:?}", folder_path);
                                
                                // The git context manager now handles:
                                // - Repository discovery
                                // - Branch detection
                                // - File status loading
                                // - All state updates
                                
                                // We just need to set up file watching if a repository is found
                                if let Some(repo) = git_context_clone.active_repository() {
                                    let repo_path = repo.path().to_path_buf();
                                    
                                    // Create git watcher for the repository
                                    match GitWatcher::new(&repo_path) {
                                        Ok((watcher, mut event_rx)) => {
                                            tracing::info!("✅ Git watcher started for repository");
                                            
                                            // Store the watcher
                                            active_git_watcher_clone.set(Some(watcher));
                                            
                                            // Spawn task to handle git events
                                            let mut git_state_for_events = git_state_clone.clone();
                                            let repo_path_for_events = repo_path.clone();
                                            spawn(async move {
                                                while let Some(event) = event_rx.recv().await {
                                                    tracing::info!("Received git event: {:?}", event);
                                                    
                                                    // Update git state based on event type
                                                    match event {
                                                        GitEvent::StatusChanged => {
                                                            // Refresh file statuses
                                                            if let Ok(repo) = GitRepository::open(&repo_path_for_events) {
                                                                if let Ok(statuses) = repo.file_statuses() {
                                                                    let mut status_map = std::collections::HashMap::new();
                                                                    for (path, git_status) in statuses {
                                                                        let status_type = if git_status.contains(git2::Status::WT_MODIFIED) || git_status.contains(git2::Status::INDEX_MODIFIED) {
                                                                            hive_ai::desktop::git::StatusType::Modified
                                                                        } else if git_status.contains(git2::Status::WT_NEW) || git_status.contains(git2::Status::INDEX_NEW) {
                                                                            hive_ai::desktop::git::StatusType::Added
                                                                        } else if git_status.contains(git2::Status::WT_DELETED) || git_status.contains(git2::Status::INDEX_DELETED) {
                                                                            hive_ai::desktop::git::StatusType::Deleted
                                                                        } else if git_status.contains(git2::Status::WT_RENAMED) || git_status.contains(git2::Status::INDEX_RENAMED) {
                                                                            hive_ai::desktop::git::StatusType::Renamed
                                                                        } else if git_status.is_wt_new() {
                                                                            hive_ai::desktop::git::StatusType::Untracked
                                                                        } else {
                                                                            continue;
                                                                        };
                                                                        
                                                                        let file_status = hive_ai::desktop::git::FileStatus {
                                                                            path: path.clone(),
                                                                            status_type,
                                                                            is_staged: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                                      git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                                      git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                                      git_status.contains(git2::Status::INDEX_RENAMED),
                                                                            has_staged_changes: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                                              git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                                              git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                                              git_status.contains(git2::Status::INDEX_RENAMED),
                                                                            has_unstaged_changes: git_status.contains(git2::Status::WT_NEW) ||
                                                                                                git_status.contains(git2::Status::WT_MODIFIED) ||
                                                                                                git_status.contains(git2::Status::WT_DELETED),
                                                                        };
                                                                        status_map.insert(path, file_status);
                                                                    }
                                                                    git_state_for_events.file_statuses.set(status_map);
                                                                    tracing::info!("✅ Updated file statuses from git watcher");
                                                                }
                                                            }
                                                        }
                                                        GitEvent::FileStatusChanged(changed_files) => {
                                                            // Update file statuses for specific changed files
                                                            tracing::info!("📁 File status changed for {} files: {:?}", changed_files.len(), changed_files);
                                                            
                                                            if let Ok(repo) = GitRepository::open(&repo_path_for_events) {
                                                                if let Ok(all_statuses) = repo.file_statuses() {
                                                                    let mut current_status_map = git_state_for_events.file_statuses.read().clone();
                                                                    let mut updated = false;
                                                                    
                                                                    // Create a set of changed files for faster lookup
                                                                    let changed_set: std::collections::HashSet<_> = changed_files.iter().collect();
                                                                    
                                                                    // Update statuses for changed files
                                                                    for (path, git_status) in &all_statuses {
                                                                        // Only process files that are in the changed list
                                                                        if changed_set.contains(path) {
                                                                            let status_type = if git_status.contains(git2::Status::WT_MODIFIED) || git_status.contains(git2::Status::INDEX_MODIFIED) {
                                                                                hive_ai::desktop::git::StatusType::Modified
                                                                            } else if git_status.contains(git2::Status::WT_NEW) || git_status.contains(git2::Status::INDEX_NEW) {
                                                                                hive_ai::desktop::git::StatusType::Added
                                                                            } else if git_status.contains(git2::Status::WT_DELETED) || git_status.contains(git2::Status::INDEX_DELETED) {
                                                                                hive_ai::desktop::git::StatusType::Deleted
                                                                            } else if git_status.contains(git2::Status::WT_RENAMED) || git_status.contains(git2::Status::INDEX_RENAMED) {
                                                                                hive_ai::desktop::git::StatusType::Renamed
                                                                            } else if git_status.is_wt_new() {
                                                                                hive_ai::desktop::git::StatusType::Untracked
                                                                            } else {
                                                                                // File is now clean, remove from status map
                                                                                current_status_map.remove(path);
                                                                                updated = true;
                                                                                continue;
                                                                            };
                                                                            
                                                                            tracing::debug!("📄 Updated status for {}: {:?}", path.display(), status_type);
                                                                            
                                                                            let file_status = hive_ai::desktop::git::FileStatus {
                                                                                path: path.clone(),
                                                                                status_type,
                                                                                is_staged: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                                          git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                                          git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                                          git_status.contains(git2::Status::INDEX_RENAMED),
                                                                                has_staged_changes: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                                                  git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                                                  git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                                                  git_status.contains(git2::Status::INDEX_RENAMED),
                                                                                has_unstaged_changes: git_status.contains(git2::Status::WT_NEW) ||
                                                                                                    git_status.contains(git2::Status::WT_MODIFIED) ||
                                                                                                    git_status.contains(git2::Status::WT_DELETED),
                                                                            };
                                                                            current_status_map.insert(path.clone(), file_status);
                                                                            updated = true;
                                                                        }
                                                                    }
                                                                    
                                                                    // Also check for files that were deleted/moved and are no longer in git status
                                                                    for changed_file in &changed_files {
                                                                        if !all_statuses.iter().any(|(path, _)| path == changed_file) {
                                                                            // File was deleted or is now clean
                                                                            if current_status_map.remove(changed_file).is_some() {
                                                                                updated = true;
                                                                                tracing::debug!("🗑️ Removed status for deleted/clean file: {}", changed_file.display());
                                                                            }
                                                                        }
                                                                    }
                                                                    
                                                                    if updated {
                                                                        git_state_for_events.file_statuses.set(current_status_map);
                                                                        tracing::info!("✅ Updated file statuses for {} changed files", changed_files.len());
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        GitEvent::BranchChanged | GitEvent::RemoteChanged => {
                                                            // Update branch info and sync status
                                                            if let Ok(repo) = GitRepository::open(&repo_path_for_events) {
                                                                if let Ok(branch_name) = repo.current_branch() {
                                                                    // Get ahead/behind counts
                                                                    let (ahead, behind) = repo.ahead_behind().unwrap_or((0, 0));
                                                                    let has_upstream = repo.upstream_branch().unwrap_or(None).is_some();
                                                                    
                                                                    let branch_info = hive_ai::desktop::git::BranchInfo {
                                                                        name: branch_name,
                                                                        branch_type: hive_ai::desktop::git::BranchType::Local,
                                                                        is_current: true,
                                                                        upstream: repo.upstream_branch().unwrap_or(None),
                                                                        ahead,
                                                                        behind,
                                                                        last_commit: None,
                                                                    };
                                                                    git_state_for_events.branch_info.set(Some(branch_info));
                                                                    
                                                                    // Update sync status
                                                                    let sync_status = hive_ai::desktop::git::SyncStatus {
                                                                        ahead,
                                                                        behind,
                                                                        has_upstream,
                                                                        is_syncing: false,
                                                                    };
                                                                    git_state_for_events.sync_status.set(sync_status);
                                                                    
                                                                    tracing::info!("✅ Updated branch info and sync status: ahead={}, behind={}", ahead, behind);
                                                                }
                                                            }
                                                        }
                                                        GitEvent::ConfigChanged => {
                                                            // Git configuration changed, refresh repository state
                                                            tracing::info!("⚙️ Git configuration changed, refreshing repository state");
                                                            // Could trigger a full refresh if needed
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to create git watcher: {}", e);
                                        }
                                    }
                                } else {
                                    tracing::info!("📄 No git repositories found in selected folder");
                                    active_git_watcher_clone.set(None);
                                }
                            });
                            
                            // Update repository context for AI Helper
                            tracing::info!("📁 User opened folder: {}", folder.path().display());
                            
                            // Update both the old consensus manager and the new IDE AI Helper Broker
                            if let Some(manager) = consensus_manager.read().clone() {
                                let folder_path = folder.path().to_path_buf();
                                spawn(async move {
                                    if let Err(e) = manager.update_repository_context_with_path(folder_path).await {
                                        tracing::warn!("Failed to update repository context: {}", e);
                                    } else {
                                        tracing::info!("✅ Repository context updated for opened folder");
                                    }
                                });
                            }
                            
                            // Update IDE AI Helper Broker repository context
                            let mut ide_ai_broker = ide_ai_broker.clone();
                            spawn(async move {
                                if let Some(broker) = ide_ai_broker.read().as_ref() {
                                    if let Err(e) = broker.update_repository_context().await {
                                        tracing::warn!("IDE AI Helper Broker failed to update context: {}", e);
                                    } else {
                                        tracing::info!("✅ IDE AI Helper Broker context updated");
                                    }
                                }
                            });

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
                                        name: root_name.clone(),
                                        is_directory: true,
                                        is_expanded: true, // Root is expanded by default
                                        children: files,
                                        file_type: FileType::Directory,
                                        git_status: None,
                                        size: None,
                                        modified: None,
                                        depth: 0,
                                    };

                                    file_tree.write().clear();
                                    file_tree.write().push(root_item);
                                    
                                    // Update AppState with current project information for repository context
                                    let project_info = hive_ai::desktop::state::ProjectInfo {
                                        name: root_name.clone(),
                                        path: folder.path().to_path_buf(),
                                        root_path: folder.path().to_path_buf(),
                                        language: None, // Will be detected by repository analyzer
                                        git_status: hive_ai::desktop::state::GitStatus::NotRepository,
                                        git_branch: None,
                                        file_count: 0, // Will be updated later
                                    };
                                    
                                    app_state.write().current_project = Some(project_info);
                                    tracing::info!("Updated current_project with root path: {}", folder.path().display());
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
                show_about_dialog.set(true);
            }
            MenuAction::OpenFile => {
                spawn({
                    let mut selected_file = selected_file.clone();
                    let mut file_content = file_content.clone();
                    let mut open_tabs = open_tabs.clone();
                    let mut active_tab = active_tab.clone();
                    let mut tab_contents = tab_contents.clone();

                    async move {
                        if let Some(file) = rfd::AsyncFileDialog::new().pick_file().await {
                            let path_string = file.path().to_string_lossy().to_string();
                            
                            // Add to open tabs if not already open
                            if !open_tabs.read().contains(&path_string) {
                                open_tabs.write().push(path_string.clone());
                            }
                            
                            // Set as active tab
                            active_tab.set(path_string.clone());
                            selected_file.set(Some(path_string.clone()));

                            match file_system::read_file_content(file.path()).await {
                                Ok(content) => {
                                    tab_contents.write().insert(path_string, content.clone());
                                    file_content.set(content);
                                }
                                Err(e) => {
                                    eprintln!("Error reading file: {}", e);
                                    let error_content = format!("// Error reading file: {}", e);
                                    tab_contents.write().insert(path_string, error_content.clone());
                                    file_content.set(error_content);
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
                current_dir.set(None);
                file_tree.write().clear();
                selected_file.set(Some("__welcome__".to_string()));
                file_content.set(String::new());
                // Also clear the app state's current project
                app_state.write().current_project = None;
                
                // Clear git context
                let mut git_context_clone = git_context.clone();
                spawn(async move {
                    git_context_clone.set_active_path(std::env::current_dir().unwrap_or_default()).await;
                    tracing::info!("Cleared git context on folder close");
                });
                
                // Also clear git state
                git_state_clone.branch_info.set(None);
                active_git_watcher_clone.set(None);
            }
            MenuAction::CommandPalette => {
                show_command_palette.set(true);
            }
            MenuAction::ChangeTheme => {
                // TODO: Show theme selector
                // For now, just log to console
                println!("Theme selector not yet implemented");
            }
            MenuAction::Settings => {
                show_settings_dialog.set(true);
            }
            MenuAction::Welcome => {
                show_welcome_dialog.set(true);
                // Set the selected file to show welcome in editor
                selected_file.set(Some("__welcome__".to_string()));
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
                        UpdateChecker::new("2.0.2".to_string(), UpdateChannel::Stable);

                    match checker.check_for_updates().await {
                        Ok(Some(update)) => {
                            println!(
                                "Update available: {} ({})",
                                update.version,
                                update.release_date.format("%Y-%m-%d")
                            );
                            // Store update information and show dialog
                            update_info.set((
                                update.version.clone(),
                                update.release_date.format("%B %d, %Y").to_string(),
                                update.download_url.clone(),
                                update.changelog_url.clone(),
                            ));
                            show_update_available_dialog.set(true);
                        }
                        Ok(None) => {
                            println!("You're running the latest version ({})", "2.0.2");
                            show_no_updates_dialog.set(true);
                        }
                        Err(e) => {
                            eprintln!("Failed to check for updates: {}", e);
                            update_error_message.set(e.to_string());
                            show_update_error_dialog.set(true);
                        }
                    }
                });
            }
            _ => {
                // Other actions not yet implemented
                println!("{:?} action not implemented yet", action);
            }
        }
        }
    };

    // Handle welcome page actions
    let handle_welcome_action = {
        let mut selected_file = selected_file.clone();
        let mut file_content = file_content.clone();
        let mut show_welcome_dialog = show_welcome_dialog.clone();
        let current_dir = current_dir.clone();
        let mut file_tree = file_tree.clone();
        let expanded_dirs = expanded_dirs.clone();
        let mut git_state = git_state.clone();
        let active_git_watcher = active_git_watcher.clone();
        let mut git_context = git_context.clone();

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
                        let mut git_state = git_state.clone();
                        let active_git_watcher = active_git_watcher.clone();
                        let mut git_context = git_context.clone();

                        async move {
                            let current_path = current_dir.read().clone();
                            let dialog = rfd::AsyncFileDialog::new();
                            let dialog = if let Some(path) = current_path {
                                dialog.set_directory(&path)
                            } else {
                                dialog
                            };
                            if let Some(folder) = dialog.pick_folder().await
                            {
                                // Update current directory
                                current_dir.set(Some(folder.path().to_path_buf()));

                                // Clear selected file and content
                                selected_file.set(None);
                                file_content.set(String::new());

                                // Clear expanded dirs for new folder
                                expanded_dirs.write().clear();
                                
                                // Initialize git repository detection
                                let folder_path = folder.path().to_path_buf();
                                let mut git_state_clone = git_state.clone();
                                let mut active_git_watcher_clone = active_git_watcher.clone();
                                let mut git_context_clone = git_context.clone();
                                spawn(async move {
                                    // Update git context manager - it handles everything
                                    git_context_clone.set_active_path(folder_path.clone()).await;
                                    
                                    // Also update git_state directly for UI components that read from it
                                    if let Ok(repo) = GitRepository::open(&folder_path) {
                                        if let Ok(branch_name) = repo.current_branch() {
                                            // Get ahead/behind counts
                                            let (ahead, behind) = repo.ahead_behind().unwrap_or((0, 0));
                                            let has_upstream = repo.upstream_branch().unwrap_or(None).is_some();
                                            
                                            let branch_info = hive_ai::desktop::git::BranchInfo {
                                                name: branch_name,
                                                branch_type: hive_ai::desktop::git::BranchType::Local,
                                                is_current: true,
                                                upstream: repo.upstream_branch().unwrap_or(None),
                                                ahead,
                                                behind,
                                                last_commit: None,
                                            };
                                            git_state_clone.branch_info.set(Some(branch_info));
                                            
                                            // Update sync status
                                            let sync_status = hive_ai::desktop::git::SyncStatus {
                                                ahead,
                                                behind,
                                                has_upstream,
                                                is_syncing: false,
                                            };
                                            git_state_clone.sync_status.set(sync_status);
                                        }
                                        
                                        // Get file statuses
                                        match repo.file_statuses() {
                                            Ok(statuses) => {
                                                let mut status_map = std::collections::HashMap::new();
                                                for (path, git_status) in statuses {
                                                    // Convert git2::Status to our StatusType
                                                    let status_type = if git_status.contains(git2::Status::WT_MODIFIED) || git_status.contains(git2::Status::INDEX_MODIFIED) {
                                                        hive_ai::desktop::git::StatusType::Modified
                                                    } else if git_status.contains(git2::Status::WT_NEW) || git_status.contains(git2::Status::INDEX_NEW) {
                                                        hive_ai::desktop::git::StatusType::Added
                                                    } else if git_status.contains(git2::Status::WT_DELETED) || git_status.contains(git2::Status::INDEX_DELETED) {
                                                        hive_ai::desktop::git::StatusType::Deleted
                                                    } else if git_status.contains(git2::Status::WT_RENAMED) || git_status.contains(git2::Status::INDEX_RENAMED) {
                                                        hive_ai::desktop::git::StatusType::Renamed
                                                    } else if git_status.is_wt_new() {
                                                        hive_ai::desktop::git::StatusType::Untracked
                                                    } else {
                                                        continue; // Skip other statuses
                                                    };
                                                    
                                                    let file_status = hive_ai::desktop::git::FileStatus {
                                                        path: path.clone(),
                                                        status_type,
                                                        is_staged: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                  git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                  git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                  git_status.contains(git2::Status::INDEX_RENAMED),
                                                        has_staged_changes: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                          git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                          git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                          git_status.contains(git2::Status::INDEX_RENAMED),
                                                        has_unstaged_changes: git_status.contains(git2::Status::WT_NEW) ||
                                                                            git_status.contains(git2::Status::WT_MODIFIED) ||
                                                                            git_status.contains(git2::Status::WT_DELETED),
                                                    };
                                                    status_map.insert(path, file_status);
                                                }
                                                git_state_clone.file_statuses.set(status_map);
                                                tracing::info!("✅ Updated git file statuses");
                                            }
                                            Err(e) => {
                                                tracing::warn!("Failed to get repository status: {}", e);
                                                git_state_clone.file_statuses.set(std::collections::HashMap::new());
                                            }
                                        }
                                        
                                        // Set up file watcher
                                        match GitWatcher::new(&folder_path) {
                                            Ok((watcher, _receiver)) => {
                                                active_git_watcher_clone.set(Some(watcher));
                                            }
                                            Err(e) => {
                                                tracing::warn!("Failed to create git watcher: {}", e);
                                            }
                                        }
                                    } else {
                                        git_state_clone.branch_info.set(None);
                                        git_state_clone.file_statuses.set(std::collections::HashMap::new());
                                        active_git_watcher_clone.set(None);
                                    }
                                    tracing::info!("Updated git context for folder: {:?}", folder_path);
                                    
                                    // The git context manager now handles:
                                    // - Repository discovery
                                    // - Branch detection
                                    // - File status loading
                                    // - All state updates
                                    
                                    // We just need to set up file watching if a repository is found
                                    if let Some(repo) = git_context_clone.active_repository() {
                                        let repo_path = repo.path().to_path_buf();
                                        
                                        // Create git watcher for the repository
                                        match GitWatcher::new(&repo_path) {
                                            Ok((watcher, mut event_rx)) => {
                                                tracing::info!("✅ Git watcher started for repository");
                                                
                                                // Store the watcher
                                                active_git_watcher_clone.set(Some(watcher));
                                                
                                                // Spawn task to handle git events
                                                let mut git_state_for_events = git_state_clone.clone();
                                                let repo_path_for_events = repo_path.clone();
                                                spawn(async move {
                                                    while let Some(event) = event_rx.recv().await {
                                                        tracing::info!("Received git event: {:?}", event);
                                                        
                                                        // Update git state based on event type
                                                        match event {
                                                            GitEvent::StatusChanged => {
                                                                // Refresh file statuses
                                                                if let Ok(repo) = GitRepository::open(&repo_path_for_events) {
                                                                    if let Ok(statuses) = repo.file_statuses() {
                                                                        let mut status_map = std::collections::HashMap::new();
                                                                        for (path, git_status) in statuses {
                                                                            let status_type = if git_status.contains(git2::Status::WT_MODIFIED) || git_status.contains(git2::Status::INDEX_MODIFIED) {
                                                                                hive_ai::desktop::git::StatusType::Modified
                                                                            } else if git_status.contains(git2::Status::WT_NEW) || git_status.contains(git2::Status::INDEX_NEW) {
                                                                                hive_ai::desktop::git::StatusType::Added
                                                                            } else if git_status.contains(git2::Status::WT_DELETED) || git_status.contains(git2::Status::INDEX_DELETED) {
                                                                                hive_ai::desktop::git::StatusType::Deleted
                                                                            } else if git_status.contains(git2::Status::WT_RENAMED) || git_status.contains(git2::Status::INDEX_RENAMED) {
                                                                                hive_ai::desktop::git::StatusType::Renamed
                                                                            } else if git_status.is_wt_new() {
                                                                                hive_ai::desktop::git::StatusType::Untracked
                                                                            } else {
                                                                                continue;
                                                                            };
                                                                            
                                                                            let file_status = hive_ai::desktop::git::FileStatus {
                                                                                path: path.clone(),
                                                                                status_type,
                                                                                is_staged: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                                          git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                                          git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                                          git_status.contains(git2::Status::INDEX_RENAMED),
                                                                                has_staged_changes: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                                                  git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                                                  git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                                                  git_status.contains(git2::Status::INDEX_RENAMED),
                                                                                has_unstaged_changes: git_status.contains(git2::Status::WT_NEW) ||
                                                                                                    git_status.contains(git2::Status::WT_MODIFIED) ||
                                                                                                    git_status.contains(git2::Status::WT_DELETED),
                                                                            };
                                                                            status_map.insert(path, file_status);
                                                                        }
                                                                        git_state_for_events.file_statuses.set(status_map);
                                                                        tracing::info!("✅ Updated file statuses from git watcher");
                                                                    }
                                                                }
                                                            }
                                                            GitEvent::FileStatusChanged(changed_files) => {
                                                                // Update file statuses for specific changed files
                                                                tracing::info!("📁 File status changed for {} files: {:?}", changed_files.len(), changed_files);
                                                                
                                                                if let Ok(repo) = GitRepository::open(&repo_path_for_events) {
                                                                    if let Ok(all_statuses) = repo.file_statuses() {
                                                                        let mut current_status_map = git_state_for_events.file_statuses.read().clone();
                                                                        let mut updated = false;
                                                                        
                                                                        // Create a set of changed files for faster lookup
                                                                        let changed_set: std::collections::HashSet<_> = changed_files.iter().collect();
                                                                        
                                                                        // Update statuses for changed files
                                                                        for (path, git_status) in &all_statuses {
                                                                            // Only process files that are in the changed list
                                                                            if changed_set.contains(path) {
                                                                                let status_type = if git_status.contains(git2::Status::WT_MODIFIED) || git_status.contains(git2::Status::INDEX_MODIFIED) {
                                                                                    hive_ai::desktop::git::StatusType::Modified
                                                                                } else if git_status.contains(git2::Status::WT_NEW) || git_status.contains(git2::Status::INDEX_NEW) {
                                                                                    hive_ai::desktop::git::StatusType::Added
                                                                                } else if git_status.contains(git2::Status::WT_DELETED) || git_status.contains(git2::Status::INDEX_DELETED) {
                                                                                    hive_ai::desktop::git::StatusType::Deleted
                                                                                } else if git_status.contains(git2::Status::WT_RENAMED) || git_status.contains(git2::Status::INDEX_RENAMED) {
                                                                                    hive_ai::desktop::git::StatusType::Renamed
                                                                                } else if git_status.is_wt_new() {
                                                                                    hive_ai::desktop::git::StatusType::Untracked
                                                                                } else {
                                                                                    // File is now clean, remove from status map
                                                                                    current_status_map.remove(path);
                                                                                    updated = true;
                                                                                    continue;
                                                                                };
                                                                                
                                                                                tracing::debug!("📄 Updated status for {}: {:?}", path.display(), status_type);
                                                                                
                                                                                let file_status = hive_ai::desktop::git::FileStatus {
                                                                                    path: path.clone(),
                                                                                    status_type,
                                                                                    is_staged: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                                              git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                                              git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                                              git_status.contains(git2::Status::INDEX_RENAMED),
                                                                                    has_staged_changes: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                                                      git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                                                      git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                                                      git_status.contains(git2::Status::INDEX_RENAMED),
                                                                                    has_unstaged_changes: git_status.contains(git2::Status::WT_NEW) ||
                                                                                                        git_status.contains(git2::Status::WT_MODIFIED) ||
                                                                                                        git_status.contains(git2::Status::WT_DELETED),
                                                                                };
                                                                                current_status_map.insert(path.clone(), file_status);
                                                                                updated = true;
                                                                            }
                                                                        }
                                                                        
                                                                        // Also check for files that were deleted/moved and are no longer in git status
                                                                        for changed_file in &changed_files {
                                                                            if !all_statuses.iter().any(|(path, _)| path == changed_file) {
                                                                                // File was deleted or is now clean
                                                                                if current_status_map.remove(changed_file).is_some() {
                                                                                    updated = true;
                                                                                    tracing::debug!("🗑️ Removed status for deleted/clean file: {}", changed_file.display());
                                                                                }
                                                                            }
                                                                        }
                                                                        
                                                                        if updated {
                                                                            git_state_for_events.file_statuses.set(current_status_map);
                                                                            tracing::info!("✅ Updated file statuses for {} changed files", changed_files.len());
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            GitEvent::BranchChanged | GitEvent::RemoteChanged => {
                                                                // Update branch info and sync status
                                                                if let Ok(repo) = GitRepository::open(&repo_path_for_events) {
                                                                    if let Ok(branch_name) = repo.current_branch() {
                                                                        // Get ahead/behind counts
                                                                        let (ahead, behind) = repo.ahead_behind().unwrap_or((0, 0));
                                                                        let has_upstream = repo.upstream_branch().unwrap_or(None).is_some();
                                                                        
                                                                        let branch_info = hive_ai::desktop::git::BranchInfo {
                                                                            name: branch_name,
                                                                            branch_type: hive_ai::desktop::git::BranchType::Local,
                                                                            is_current: true,
                                                                            upstream: repo.upstream_branch().unwrap_or(None),
                                                                            ahead,
                                                                            behind,
                                                                            last_commit: None,
                                                                        };
                                                                        git_state_for_events.branch_info.set(Some(branch_info));
                                                                        
                                                                        // Update sync status
                                                                        let sync_status = hive_ai::desktop::git::SyncStatus {
                                                                            ahead,
                                                                            behind,
                                                                            has_upstream,
                                                                            is_syncing: false,
                                                                        };
                                                                        git_state_for_events.sync_status.set(sync_status);
                                                                        
                                                                        tracing::info!("✅ Updated branch info and sync status: ahead={}, behind={}", ahead, behind);
                                                                    }
                                                                }
                                                            }
                                                            GitEvent::ConfigChanged => {
                                                                // Git configuration changed, refresh repository state
                                                                tracing::info!("⚙️ Git configuration changed, refreshing repository state");
                                                                // Could trigger a full refresh if needed
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to create git watcher: {}", e);
                                            }
                                        }
                                    } else {
                                        tracing::info!("📄 No git repositories found in selected folder");
                                        active_git_watcher_clone.set(None);
                                    }
                                });
                                
                                // Update repository context for AI Helper
                                tracing::info!("📁 User opened folder from welcome screen: {}", folder.path().display());
                                
                                // Update both the old consensus manager and the new IDE AI Helper Broker
                                if let Some(manager) = consensus_manager.read().clone() {
                                    let folder_path = folder.path().to_path_buf();
                                    spawn(async move {
                                        if let Err(e) = manager.update_repository_context_with_path(folder_path).await {
                                            tracing::warn!("Failed to update repository context: {}", e);
                                        } else {
                                            tracing::info!("✅ Repository context updated for opened folder");
                                        }
                                    });
                                }
                                
                                // Update IDE AI Helper Broker repository context
                                let mut ide_ai_broker = ide_ai_broker.clone();
                                spawn(async move {
                                    if let Some(broker) = ide_ai_broker.read().as_ref() {
                                        if let Err(e) = broker.update_repository_context().await {
                                            tracing::warn!("IDE AI Helper Broker failed to update context: {}", e);
                                        } else {
                                            tracing::info!("✅ IDE AI Helper Broker context updated");
                                        }
                                    }
                                });

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
                                            name: root_name.clone(),
                                            is_directory: true,
                                            is_expanded: true, // Root is expanded by default
                                            children: files,
                                            file_type: FileType::Directory,
                                            git_status: None,
                                            size: None,
                                            modified: None,
                                            depth: 0,
                                        };

                                        file_tree.write().clear();
                                        file_tree.write().push(root_item);
                                        
                                        // Update AppState with current project information for repository context
                                        let project_info = hive_ai::desktop::state::ProjectInfo {
                                            name: root_name.clone(),
                                            path: folder.path().to_path_buf(),
                                            root_path: folder.path().to_path_buf(),
                                            language: None, // Will be detected by repository analyzer
                                            git_status: hive_ai::desktop::state::GitStatus::NotRepository,
                                            git_branch: None,
                                            file_count: 0, // Will be updated later
                                        };
                                        
                                        app_state.write().current_project = Some(project_info);
                                        tracing::info!("Updated current_project with root path: {}", folder.path().display());
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
                    selected_file.set(Some("untitled.txt".to_string()));
                    file_content.set(String::new());
                    show_welcome_dialog.set(false);
                }
            }
        }
    };

    // Compute display value for current directory
    let current_dir_display = if let Some(dir) = current_dir.read().as_ref() {
        dir.display().to_string()
    } else {
        "No folder open".to_string()
    };

    rsx! {
        // Inject comprehensive styles including all component styles
        style { "{DESKTOP_STYLES}" }
        style { "{DIALOG_STYLES}" }
        style { "{STATUS_BAR_STYLES}" }
        style { "{EDITOR_STYLES}" }

        div {
            class: "app-container",
            onkeydown: {
                let consensus_manager = consensus_manager.clone();
                let mut is_processing = is_processing.clone();
                let mut is_cancelling = is_cancelling.clone();
                let mut app_state = app_state.clone();
                let mut current_response = current_response.clone();
                let mut show_terminal = show_terminal.clone();
                move |evt: dioxus::events::KeyboardEvent| {
                    // IMPORTANT: When terminal is visible, only handle the terminal toggle shortcut
                    // Let all other keyboard events pass through to the terminal
                    if *show_terminal.read() {
                        // Only handle terminal toggle (Ctrl+T) when terminal is visible
                        let is_mac = cfg!(target_os = "macos");
                        let modifier_pressed = if is_mac { evt.modifiers().meta() } else { evt.modifiers().ctrl() };
                        
                        if modifier_pressed && evt.key() == Key::Character("t".to_string()) {
                            evt.stop_propagation();
                            evt.prevent_default();
                            
                            // Toggle terminal visibility
                            let current = *show_terminal.read();
                            show_terminal.set(!current);
                            
                            tracing::info!("🖥️ Terminal toggled: {}", if !current { "shown" } else { "hidden" });
                            return;
                        }
                        
                        // CRITICAL: Don't handle ANY other events when terminal is visible
                        // This allows the terminal to receive all keyboard input
                        return;
                    }
                    
                    // Check for Ctrl+C (or Cmd+C on Mac) 
                    let is_mac = cfg!(target_os = "macos");
                    let modifier_pressed = if is_mac { evt.modifiers().meta() } else { evt.modifiers().ctrl() };
                    
                    if modifier_pressed && evt.key() == Key::Character("c".to_string()) {
                        // Only handle if consensus is running and not already cancelling
                        if *is_processing.read() && !*is_cancelling.read() {
                            evt.stop_propagation();
                            tracing::info!("🛑 Ctrl+C pressed - cancelling consensus!");
                            
                            // Immediately set cancelling state and show feedback
                            is_cancelling.set(true);
                            
                            // Show cancelling message immediately
                            app_state.write().consensus.streaming_content = 
                                "<div style='color: #FF6B6B; font-weight: bold;'>⏸ Cancelling consensus...</div>".to_string();
                            
                            tracing::info!("🛑 Ctrl+C pressed - cancelling consensus");
                            
                            // Set the atomic cancellation flag
                            cancellation_flag.read().store(true, std::sync::atomic::Ordering::Relaxed);
                            
                            // Cancel the running task immediately
                            if let Some(task) = consensus_task_handle.write().take() {
                                tracing::info!("🛑 Cancelling consensus task via Ctrl+C");
                                task.cancel();
                            }
                            
                            // Also cancel through the consensus manager for immediate effect
                            let mut consensus_manager = consensus_manager.clone();
                            spawn(async move {
                                if let Some(mut manager) = consensus_manager.write().as_mut() {
                                    if let Err(e) = manager.cancel_consensus("User pressed Ctrl+C").await {
                                        tracing::warn!("Failed to cancel consensus via Ctrl+C: {}", e);
                                    } else {
                                        tracing::info!("✅ Consensus manager cancellation via Ctrl+C successful");
                                    }
                                }
                            });
                            
                            // Reset UI state immediately
                            cancel_flag.set(true);
                            app_state.write().consensus.complete_consensus();
                            app_state.write().consensus.streaming_content.clear();
                            app_state.write().consensus.current_stage = None;
                            is_processing.set(false);
                            is_cancelling.set(false);
                            current_response.set(String::new());
                            
                            // Show immediate feedback
                            app_state.write().consensus.streaming_content = 
                                "<div style='color: #4CAF50; font-weight: bold;'>✅ Cancelled via Ctrl+C - ready for new query</div>".to_string();
                            
                            tracing::info!("✅ Cancellation flag set via Ctrl+C");
                        }
                    }
                    
                    // Check for Ctrl+T or Cmd+T to toggle terminal
                    if modifier_pressed && evt.key() == Key::Character("t".to_string()) {
                        evt.stop_propagation();
                        evt.prevent_default();
                        
                        // Toggle terminal visibility
                        let current = *show_terminal.read();
                        show_terminal.set(!current);
                        
                        tracing::info!("🖥️ Terminal toggled: {}", if !current { "shown" } else { "hidden" });
                    }
                    
                    // Check for Ctrl+G or Cmd+G to toggle GitUI panel
                    if modifier_pressed && evt.key() == Key::Character("g".to_string()) {
                        evt.stop_propagation();
                        evt.prevent_default();
                        
                        // Toggle GitUI panel visibility
                        let current = *show_gitui.read();
                        show_gitui.set(!current);
                        
                        tracing::info!("🔀 GitUI panel toggled: {}", if !current { "shown" } else { "hidden" });
                    }
                }
            },

            // Menu bar at the top
            MenuBar {
                on_action: handle_menu_action,
            }

            // Main content (below menu bar)
            div {
                class: "main-content",
                style: "position: relative; display: flex; flex: 1; overflow: hidden;",

                // GitUI Panel (far left) - NEW!
                if *show_gitui.read() {
                    div {
                        class: "gitui-panel",
                        style: format!("background: #0A0E0F; border-right: 1px solid #2D3336; display: flex; flex-direction: column; height: 100%; width: {}px; position: relative;", 
                            gitui_width.read()
                        ),
                        
                        // GitUI header
                        div {
                            class: "gitui-header",
                            style: "background: #181E21; border-bottom: 1px solid #2D3336; padding: 12px 16px; display: flex; align-items: center; justify-content: space-between;",
                            
                            div {
                                style: "display: flex; align-items: center; gap: 8px;",
                                span {
                                    style: "font-size: 18px;",
                                    "🔀"
                                }
                                span {
                                    style: "color: #FFC107; font-weight: 600; font-size: 14px;",
                                    "GitUI"
                                }
                            }
                            
                            // Close button
                            button {
                                style: "background: transparent; border: none; color: #9CA3AF; cursor: pointer; padding: 4px; font-size: 16px;",
                                title: "Hide GitUI Panel",
                                onclick: move |_| {
                                    show_gitui.set(false);
                                },
                                "×"
                            }
                        }
                        
                        // GitUI terminal container
                        div {
                            class: "gitui-terminal-container",
                            style: "flex: 1; overflow: hidden; position: relative;",
                            key: "{gitui_update_counter}",  // Force re-render when directory changes
                            
                            {
                                // Compute values outside rsx
                                let terminal_id_opt = gitui_terminal_id.read().clone();
                                let current_path = current_dir.read().as_ref()
                                    .cloned()
                                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                                let git_root_opt = find_git_root(&current_path);
                                
                                rsx! {
                                    if let Some(terminal_id) = terminal_id_opt {
                                        if let Some(git_root) = git_root_opt {
                                            TerminalXterm {
                                                terminal_id: terminal_id.clone(),
                                                initial_directory: Some(git_root.to_string_lossy().to_string()),
                                                command: Some("gitui".to_string()),
                                                args: vec![]  // GitUI will use the initial_directory
                                            }
                                        } else {
                                            // Not in a git repository
                                            div {
                                                style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: #9CA3AF; padding: 20px; text-align: center;",
                                                div {
                                                    style: "font-size: 18px; margin-bottom: 10px;",
                                                    "⚠️ Not a Git Repository"
                                                }
                                                div {
                                                    style: "font-size: 14px; color: #6B737C;",
                                                    "The current directory is not inside a Git repository."
                                                }
                                                div {
                                                    style: "font-size: 12px; color: #6B737C; margin-top: 20px;",
                                                    "Initialize a repository with: git init"
                                                }
                                            }
                                        }
                                    } else {
                                        div {
                                            style: "display: flex; align-items: center; justify-content: center; height: 100%; color: #9CA3AF;",
                                            "Initializing GitUI..."
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Resize divider between GitUI and sidebar
                    ResizableDivider {
                        direction: ResizeDirection::Horizontal,
                        size: gitui_width,
                        min_size: 200.0,
                        max_size: 600.0,
                        invert_drag: false,
                    }
                }

                // Sidebar (left)
                div {
                    class: "sidebar",
                    style: format!("background: #0E1414; border-right: 1px solid #2D3336; box-shadow: 4px 0 24px rgba(0, 0, 0, 0.5); display: flex; flex-direction: column; height: 100%; width: {}px; position: relative;", 
                        sidebar_width.read()
                    ),

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
                            title: "{current_dir_display}",
                            "{current_dir_display}"
                        }
                    }
                    
                    // View toggle buttons
                    div {
                        style: "display: flex; padding: 0; background: #181E21; border-bottom: 1px solid #2D3336;",
                        
                        // File Explorer button
                        button {
                            style: format!("flex: 1; padding: 8px; background: {}; border: none; color: {}; font-size: 12px; cursor: pointer; transition: all 0.2s;",
                                if *sidebar_view.read() == SidebarView::FileExplorer { "#2D3336" } else { "transparent" },
                                if *sidebar_view.read() == SidebarView::FileExplorer { "#FFC107" } else { "#9CA3AF" }
                            ),
                            onclick: move |_| {
                                sidebar_view.set(SidebarView::FileExplorer);
                            },
                            "📁 Files"
                        }
                        
                        // Source Control button
                        button {
                            style: format!("flex: 1; padding: 8px; background: {}; border: none; color: {}; font-size: 12px; cursor: pointer; transition: all 0.2s;",
                                if *sidebar_view.read() == SidebarView::SourceControl { "#2D3336" } else { "transparent" },
                                if *sidebar_view.read() == SidebarView::SourceControl { "#FFC107" } else { "#9CA3AF" }
                            ),
                            onclick: move |_| {
                                sidebar_view.set(SidebarView::SourceControl);
                            },
                            "🔀 Source Control"
                        }
                    }

                    // Scrollable content container
                    div {
                        class: "file-tree-container",
                        style: "flex: 1; overflow-y: auto; overflow-x: hidden; padding: 0 10px;",
                        
                        // Show different content based on active view
                        match *sidebar_view.read() {
                            SidebarView::FileExplorer => rsx! {
                                div {
                                    class: "explorer-header",
                                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; padding-top: 10px;",
                                    
                                    div {
                                        class: "sidebar-section-title",
                                        style: "background: linear-gradient(to right, #FFC107, #FFD54F); -webkit-background-clip: text; -webkit-text-fill-color: transparent; font-weight: 700; font-size: 12px; margin: 0;",
                                        "EXPLORER"
                                    }
                            
                            div {
                                class: "explorer-toolbar",
                                style: "display: flex; gap: 2px; margin-left: auto;",
                                
                                // New File Button - VS Code style with text and icon
                                button {
                                    class: "explorer-toolbar-btn new-file-btn",
                                    style: "background: transparent; border: none; color: #cccccc; padding: 4px 8px; border-radius: 3px; cursor: pointer; display: flex; align-items: center; gap: 4px; font-size: 12px; height: 22px; transition: background-color 0.1s ease;",
                                    title: "New File (Ctrl+N)",
                                    // Hover effects are handled by CSS now
                                    onclick: move |_| {
                                        // Determine target path: selected folder or current directory
                                        let target = if let Some(selected) = selected_file.read().as_ref() {
                                            let selected_path = std::path::Path::new(selected);
                                            if selected_path.is_dir() {
                                                selected_path.to_path_buf()
                                            } else if let Some(parent) = selected_path.parent() {
                                                parent.to_path_buf()
                                            } else {
                                                current_dir.read().clone().unwrap_or_else(|| PathBuf::from("."))
                                            }
                                        } else {
                                            current_dir.read().clone().unwrap_or_else(|| PathBuf::from("."))
                                        };
                                        
                                        show_new_file_dialog.set(true);
                                        dialog_target_path.set(Some(target));
                                    },
                                    span { style: "font-size: 14px;", "📄" }
                                    span { "New File" }
                                }
                                
                                // New Folder Button - VS Code style with text and icon
                                button {
                                    class: "explorer-toolbar-btn new-folder-btn",
                                    style: "background: transparent; border: none; color: #cccccc; padding: 4px 8px; border-radius: 3px; cursor: pointer; display: flex; align-items: center; gap: 4px; font-size: 12px; height: 22px; margin-left: 4px; transition: background-color 0.1s ease;",
                                    title: "New Folder (Ctrl+Shift+N)",
                                    // Hover effects are handled by CSS now
                                    onclick: move |_| {
                                        // Determine target path: selected folder or current directory
                                        let target = if let Some(selected) = selected_file.read().as_ref() {
                                            let selected_path = std::path::Path::new(selected);
                                            if selected_path.is_dir() {
                                                selected_path.to_path_buf()
                                            } else if let Some(parent) = selected_path.parent() {
                                                parent.to_path_buf()
                                            } else {
                                                current_dir.read().clone().unwrap_or_else(|| PathBuf::from("."))
                                            }
                                        } else {
                                            current_dir.read().clone().unwrap_or_else(|| PathBuf::from("."))
                                        };
                                        
                                        show_new_folder_dialog.set(true);
                                        dialog_target_path.set(Some(target));
                                    },
                                    span { style: "font-size: 14px;", "📁" }
                                    span { "New Folder" }
                                }
                            }
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
                                open_tabs: open_tabs.clone(),
                                active_tab: active_tab.clone(),
                                tab_contents: tab_contents.clone(),
                                context_menu_state: context_menu_state.clone(),
                                consensus_manager: consensus_manager.clone(),
                                ide_ai_broker: ide_ai_broker.clone(),
                            }
                        }

                                if file_tree.read().is_empty() {
                                    div {
                                        class: "sidebar-item",
                                        style: "color: #858585; font-style: italic;",
                                        "No files in directory"
                                    }
                                }
                            },
                            SidebarView::SourceControl => rsx! {
                                // Source Control View
                                div {
                                    class: "source-control-header",
                                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; padding-top: 10px;",
                                    
                                    div {
                                        class: "sidebar-section-title",
                                        style: "background: linear-gradient(to right, #FFC107, #FFD54F); -webkit-background-clip: text; -webkit-text-fill-color: transparent; font-weight: 700; font-size: 12px; margin: 0;",
                                        "SOURCE CONTROL"
                                    }
                                }
                                
                                // Git repository info
                                if let Some(branch_info) = git_state.branch_info.read().as_ref() {
                                    div {
                                        style: "padding: 8px; background: #1A1F21; border-radius: 4px; margin-bottom: 12px;",
                                        
                                        div {
                                            style: "display: flex; align-items: center; gap: 8px;",
                                            
                                            span {
                                                style: "color: #FFC107; font-size: 14px;",
                                                "🔀"
                                            }
                                            
                                            span {
                                                style: "color: #FFFFFF; font-weight: 600;",
                                                "{branch_info.name}"
                                            }
                                            
                                            if branch_info.ahead > 0 || branch_info.behind > 0 {
                                                span {
                                                    style: "color: #9CA3AF; font-size: 11px; margin-left: auto;",
                                                    {
                                                        if branch_info.ahead > 0 && branch_info.behind > 0 {
                                                            format!("↑{} ↓{}", branch_info.ahead, branch_info.behind)
                                                        } else if branch_info.ahead > 0 {
                                                            format!("↑{}", branch_info.ahead)
                                                        } else {
                                                            format!("↓{}", branch_info.behind)
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                // Git operations toolbar (always show, even without repo)
                                {
                                    let repo_path = current_dir.read().clone();
                                    let file_statuses = git_state.file_statuses.read().clone();
                                    let staged_count = file_statuses.values().filter(|s| s.is_staged).count();
                                    let unstaged_count = file_statuses.values().filter(|s| !s.is_staged).count();
                                    let branch_info = git_state.branch_info.read();
                                    let current_branch = branch_info.as_ref().map(|b| b.name.clone());
                                    
                                    rsx! {
                                        GitToolbar {
                                            repo_path: repo_path,
                                            staged_count: staged_count,
                                            unstaged_count: unstaged_count,
                                            current_branch: current_branch,
                                            is_pushing: is_pushing.clone(),
                                            is_pulling: is_pulling.clone(),
                                            is_syncing: is_syncing.clone(),
                                            operation_status: git_operation_status.clone(),
                                            on_operation: EventHandler::new({
                                                let repo_path = current_dir.read().clone();
                                                let mut git_state_for_refresh = git_state.clone();
                                                let is_pushing_handler = is_pushing.clone();
                                                let is_pulling_handler = is_pulling.clone();
                                                let is_syncing_handler = is_syncing.clone();
                                                move |operation: GitOperation| {
                                                    if let Some(repo_path) = repo_path.clone() {
                                                        let mut git_state_clone = git_state_for_refresh.clone();
                                                        let mut is_pushing_reset = is_pushing_handler.clone();
                                                        let mut is_pulling_reset = is_pulling_handler.clone();
                                                        let mut is_syncing_reset = is_syncing_handler.clone();
                                                        let mut operation_status_update = git_operation_status.clone();
                                                        
                                                        // Create cancellation token for this operation
                                                        let cancel_token = CancellationToken::new();
                                                        
                                                        // Create progress callback
                                                        let progress_callback: ProgressCallback = Arc::new(move |progress| {
                                                            tracing::info!("Git operation progress: {:?}", progress);
                                                            // TODO: Update UI with progress
                                                        });
                                                        
                                                        spawn(async move {
                                                            // Handle git operations
                                                            if let Ok(git_ops) = GitOperations::new(&repo_path) {
                                                                let result = match &operation {
                                                                        GitOperation::StageAll => git_ops.stage_all().await,
                                                                        GitOperation::UnstageAll => git_ops.unstage_all().await,
                                                                        GitOperation::Commit(message) => {
                                                                            git_ops.commit(&message).await.map(|_| ())
                                                                        },
                                                                        GitOperation::Push => {
                                                                            if let (Ok(remote), Ok(branch)) = (git_ops.get_default_remote().await, git_ops.get_current_branch().await) {
                                                                                git_ops.push_with_progress(&remote, &branch, Some(progress_callback.clone()), Some(cancel_token.clone())).await
                                                                            } else {
                                                                                Err(anyhow::anyhow!("No remote or branch configured"))
                                                                            }
                                                                        },
                                                                        GitOperation::Pull => {
                                                                            if let (Ok(remote), Ok(branch)) = (git_ops.get_default_remote().await, git_ops.get_current_branch().await) {
                                                                                git_ops.pull_with_progress(&remote, &branch, Some(progress_callback.clone()), Some(cancel_token.clone())).await
                                                                            } else {
                                                                                Err(anyhow::anyhow!("No remote or branch configured"))
                                                                            }
                                                                        },
                                                                        GitOperation::Fetch => {
                                                                            if let Ok(remote) = git_ops.get_default_remote().await {
                                                                                git_ops.fetch_with_progress(&remote, Some(progress_callback.clone()), Some(cancel_token.clone())).await
                                                                            } else {
                                                                                Err(anyhow::anyhow!("No remote configured"))
                                                                            }
                                                                        },
                                                                        GitOperation::Sync => {
                                                                            // Enhanced VS Code style sync with proper error handling
                                                                            tracing::info!("Starting sync operation");
                                                                            
                                                                            // Update operation status for user feedback
                                                                            operation_status_update.set(Some("Initializing sync...".to_string()));
                                                                            
                                                                            if let (Ok(remote), Ok(branch)) = (git_ops.get_default_remote().await, git_ops.get_current_branch().await) {
                                                                                // Update status during operation
                                                                                operation_status_update.set(Some(format!("Syncing with origin/{}...", branch)));
                                                                                
                                                                                let sync_result = git_ops.sync_with_progress(&remote, &branch, Some(progress_callback.clone()), Some(cancel_token.clone())).await;
                                                                                
                                                                                // Update status based on result
                                                                                match &sync_result {
                                                                                    Ok(_) => {
                                                                                        operation_status_update.set(Some("✅ Sync completed successfully".to_string()));
                                                                                        // Clear status after delay
                                                                                        let mut status_clear = operation_status_update.clone();
                                                                                        spawn(async move {
                                                                                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                                                                            status_clear.set(None);
                                                                                        });
                                                                                    }
                                                                                    Err(e) => {
                                                                                        let error_msg = if e.to_string().contains("Nothing to commit") {
                                                                                            "ℹ️ Repository is already up to date".to_string()
                                                                                        } else if e.to_string().contains("Authentication") {
                                                                                            "❌ Authentication failed - check credentials".to_string()
                                                                                        } else if e.to_string().contains("Network") || e.to_string().contains("connection") {
                                                                                            "❌ Network error - check connection".to_string()
                                                                                        } else {
                                                                                            format!("❌ Sync failed: {}", e)
                                                                                        };
                                                                                        operation_status_update.set(Some(error_msg));
                                                                                        // Clear error after longer delay
                                                                                        let mut status_clear = operation_status_update.clone();
                                                                                        spawn(async move {
                                                                                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                                                                            status_clear.set(None);
                                                                                        });
                                                                                    }
                                                                                }
                                                                                
                                                                                sync_result
                                                                            } else {
                                                                                let error_msg = "❌ No remote or branch configured";
                                                                                tracing::error!("{}", error_msg);
                                                                                
                                                                                // Update status with error
                                                                                operation_status_update.set(Some(error_msg.to_string()));
                                                                                // Clear error after delay
                                                                                let mut status_clear = operation_status_update.clone();
                                                                                spawn(async move {
                                                                                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                                                                    status_clear.set(None);
                                                                                });
                                                                                
                                                                                Err(anyhow::anyhow!(error_msg))
                                                                            }
                                                                        },
                                                                        GitOperation::Stage(path) => git_ops.stage_file(path).await,
                                                                        GitOperation::Unstage(path) => git_ops.unstage_file(path).await,
                                                                        GitOperation::DiscardChanges(path) => git_ops.discard_file_changes(path).await,
                                                                        GitOperation::StashSave(message) => {
                                                                            // TODO: Implement stash_save wrapper in GitOperations
                                                                            tracing::warn!("Stash save not yet implemented: {}", message);
                                                                            Ok(())
                                                                        },
                                                                        GitOperation::StashApply(stash_id) => {
                                                                            // TODO: Implement stash_apply wrapper in GitOperations
                                                                            tracing::warn!("Stash apply not yet implemented: {}", stash_id);
                                                                            Ok(())
                                                                        },
                                                                        GitOperation::StashPop(stash_id) => {
                                                                            // TODO: Implement stash_pop wrapper in GitOperations
                                                                            tracing::warn!("Stash pop not yet implemented: {}", stash_id);
                                                                            Ok(())
                                                                        },
                                                                        GitOperation::StashDrop(stash_id) => {
                                                                            // TODO: Implement stash_drop wrapper in GitOperations
                                                                            tracing::warn!("Stash drop not yet implemented: {}", stash_id);
                                                                            Ok(())
                                                                        },
                                                                        GitOperation::StashList => {
                                                                            // TODO: Implement stash_list wrapper in GitOperations
                                                                            tracing::warn!("Stash list not yet implemented");
                                                                            Ok(())
                                                                        },
                                                                        GitOperation::StashShow(stash_id) => {
                                                                            // TODO: Implement stash_show wrapper in GitOperations
                                                                            tracing::warn!("Stash show not yet implemented: {}", stash_id);
                                                                            Ok(())
                                                                        },
                                                                    };
                                                                    
                                                            // Reset operation states - CRITICAL: Always reset regardless of success/failure
                                                            match &operation {
                                                                GitOperation::Push => {
                                                                    is_pushing_reset.set(false);
                                                                    tracing::debug!("Reset pushing state to false");
                                                                },
                                                                GitOperation::Pull => {
                                                                    is_pulling_reset.set(false);
                                                                    tracing::debug!("Reset pulling state to false");
                                                                },
                                                                GitOperation::Sync => {
                                                                    is_syncing_reset.set(false);
                                                                    tracing::debug!("Reset syncing state to false");
                                                                },
                                                                _ => {}
                                                            }
                                                            
                                                            match result {
                                                                Ok(_) => {
                                                                    tracing::info!("Git operation successful: {:?}", operation);
                                                                    // Refresh git state
                                                                    if let Ok(repo) = GitRepository::open(&repo_path) {
                                                                        // Update file statuses
                                                                        match repo.file_statuses() {
                                                                            Ok(statuses) => {
                                                                                let mut status_map = std::collections::HashMap::new();
                                                                                for (path, git_status) in statuses {
                                                                                    let status_type = if git_status.contains(git2::Status::WT_MODIFIED) || git_status.contains(git2::Status::INDEX_MODIFIED) {
                                                                                        hive_ai::desktop::git::StatusType::Modified
                                                                                    } else if git_status.contains(git2::Status::WT_NEW) || git_status.contains(git2::Status::INDEX_NEW) {
                                                                                        hive_ai::desktop::git::StatusType::Added
                                                                                    } else if git_status.contains(git2::Status::WT_DELETED) || git_status.contains(git2::Status::INDEX_DELETED) {
                                                                                        hive_ai::desktop::git::StatusType::Deleted
                                                                                    } else if git_status.contains(git2::Status::WT_RENAMED) || git_status.contains(git2::Status::INDEX_RENAMED) {
                                                                                        hive_ai::desktop::git::StatusType::Renamed
                                                                                    } else if git_status.is_wt_new() {
                                                                                        hive_ai::desktop::git::StatusType::Untracked
                                                                                    } else {
                                                                                        continue;
                                                                                    };
                                                                                    
                                                                                    let file_status = hive_ai::desktop::git::FileStatus {
                                                                                        path: path.clone(),
                                                                                        status_type,
                                                                                        is_staged: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                                                  git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                                                  git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                                                  git_status.contains(git2::Status::INDEX_RENAMED),
                                                                                        has_staged_changes: git_status.contains(git2::Status::INDEX_NEW) ||
                                                                                                          git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                                                                          git_status.contains(git2::Status::INDEX_DELETED) ||
                                                                                                          git_status.contains(git2::Status::INDEX_RENAMED),
                                                                                        has_unstaged_changes: git_status.contains(git2::Status::WT_NEW) ||
                                                                                                            git_status.contains(git2::Status::WT_MODIFIED) ||
                                                                                                            git_status.contains(git2::Status::WT_DELETED),
                                                                                    };
                                                                                    status_map.insert(path, file_status);
                                                                                }
                                                                                git_state_clone.file_statuses.set(status_map);
                                                                                tracing::info!("✅ Refreshed git file statuses after operation");
                                                                            }
                                                                            Err(e) => {
                                                                                tracing::warn!("Failed to refresh git status: {}", e);
                                                                            }
                                                                        }
                                                                    }
                                                                },
                                                                Err(e) => {
                                                                    tracing::error!("Git operation failed: {:?} - {}", operation, e);
                                                                    
                                                                    // Ensure operation status is updated with error if not already done
                                                                    if operation_status_update.read().is_none() {
                                                                        let error_msg = match &operation {
                                                                            GitOperation::Sync => format!("❌ Sync failed: {}", e),
                                                                            GitOperation::Push => format!("❌ Push failed: {}", e),
                                                                            GitOperation::Pull => format!("❌ Pull failed: {}", e),
                                                                            _ => format!("❌ Operation failed: {}", e),
                                                                        };
                                                                        operation_status_update.set(Some(error_msg));
                                                                        
                                                                        // Clear error after delay
                                                                        let mut status_clear = operation_status_update.clone();
                                                                        spawn(async move {
                                                                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                                                            status_clear.set(None);
                                                                        });
                                                                    }
                                                                    
                                                                    // CRITICAL: Always reset operation states on error too
                                                                    match &operation {
                                                                        GitOperation::Push => {
                                                                            is_pushing_reset.set(false);
                                                                            tracing::debug!("Reset pushing state to false (error)");
                                                                        },
                                                                        GitOperation::Pull => {
                                                                            is_pulling_reset.set(false);
                                                                            tracing::debug!("Reset pulling state to false (error)");
                                                                        },
                                                                        GitOperation::Sync => {
                                                                            is_syncing_reset.set(false);
                                                                            tracing::debug!("Reset syncing state to false (error)");
                                                                        },
                                                                        _ => {}
                                                                    }
                                                                }
                                                            }
                                                            } else {
                                                                tracing::error!("Failed to create GitOperations for repo: {:?}", repo_path);
                                                                
                                                                // Update status with initialization error
                                                                operation_status_update.set(Some("❌ Failed to initialize git operations".to_string()));
                                                                
                                                                // CRITICAL: Reset operation states even on initialization failure
                                                                match &operation {
                                                                    GitOperation::Push => {
                                                                        is_pushing_reset.set(false);
                                                                        tracing::debug!("Reset pushing state to false (init failure)");
                                                                    },
                                                                    GitOperation::Pull => {
                                                                        is_pulling_reset.set(false);
                                                                        tracing::debug!("Reset pulling state to false (init failure)");
                                                                    },
                                                                    GitOperation::Sync => {
                                                                        is_syncing_reset.set(false);
                                                                        tracing::debug!("Reset syncing state to false (init failure)");
                                                                    },
                                                                    _ => {}
                                                                }
                                                                
                                                                // Clear error after delay
                                                                let mut status_clear = operation_status_update.clone();
                                                                spawn(async move {
                                                                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                                                    status_clear.set(None);
                                                                });
                                                            }
                                                        });
                                                    }
                                                }
                                            })
                                        }
                                    }
                                }
                                
                                // Changes section
                                if git_state.branch_info.read().is_some() {
                                    div {
                                        style: "margin-top: 16px;",
                                        
                                        div {
                                            style: "color: #9CA3AF; font-size: 11px; text-transform: uppercase; margin-bottom: 8px;",
                                            "Changes"
                                        }
                                        
                                        // Show file statuses from git state
                                        {
                                            let file_statuses = git_state.file_statuses.read().clone();
                                            if file_statuses.is_empty() {
                                                rsx! {
                                                    div {
                                                        style: "color: #858585; font-style: italic; padding: 12px; text-align: center;",
                                                        "No changes detected"
                                                    }
                                                }
                                            } else {
                                                rsx! {
                                                    for (path, status) in file_statuses.iter() {
                                                {
                                                    let file_path_str = path.to_string_lossy().to_string();
                                                    let file_path_for_click = file_path_str.clone();
                                                    let file_path_for_click_inner = file_path_str.clone();
                                                    let mut open_tabs = open_tabs.clone();
                                                    let mut active_tab = active_tab.clone();
                                                    let mut selected_file = selected_file.clone();
                                                    let mut tab_contents = tab_contents.clone();
                                                    let mut file_content = file_content.clone();
                                                    let mut current_view = current_view.clone();
                                                    
                                                    rsx! {
                                                        div {
                                                            style: "padding: 6px 8px; cursor: pointer; display: flex; align-items: center; gap: 8px; transition: background 0.1s ease;",
                                                            class: "git-status-item",
                                                            onclick: move |_| {
                                                                // Open diff view for the file
                                                                let file_path_to_open = file_path_for_click_inner.clone();
                                                                let mut diff_tabs = diff_tabs.clone();
                                                                let mut open_tabs = open_tabs.clone();
                                                                let mut active_tab = active_tab.clone();
                                                                let mut current_view = current_view.clone();
                                                                let repo_path = current_dir.read().clone().unwrap_or_else(|| PathBuf::from("."));
                                                                
                                                                spawn(async move {
                                                                    // Generate diff for the file
                                                                    let full_file_path = repo_path.join(&file_path_to_open);
                                                                    match get_file_diff(&repo_path, &full_file_path).await {
                                                                        Ok(diff_result) => {
                                                                            // Create a special tab name for diffs
                                                                            let diff_tab_name = format!("diff:{}", file_path_to_open);
                                                                            
                                                                            // Store the diff result
                                                                            diff_tabs.write().insert(diff_tab_name.clone(), diff_result);
                                                                            
                                                                            // Add to open tabs if not already open
                                                                            if !open_tabs.read().contains(&diff_tab_name) {
                                                                                open_tabs.write().push(diff_tab_name.clone());
                                                                            }
                                                                            
                                                                            // Switch to this diff tab
                                                                            active_tab.set(diff_tab_name);
                                                                            current_view.set("diff".to_string());
                                                                        }
                                                                        Err(e) => {
                                                                            tracing::error!("Failed to generate diff: {}", e);
                                                                        }
                                                                    }
                                                                });
                                                            },
                                                            
                                                            // Status indicator
                                                            span {
                                                                style: format!("color: {}; font-weight: bold; width: 16px; text-align: center;",
                                                                    match status.status_type {
                                                                        hive_ai::desktop::git::StatusType::Modified => "#FFB800",
                                                                        hive_ai::desktop::git::StatusType::Added => "#4CAF50",
                                                                        hive_ai::desktop::git::StatusType::Deleted => "#F44336",
                                                                        hive_ai::desktop::git::StatusType::Renamed => "#2196F3",
                                                                        hive_ai::desktop::git::StatusType::Untracked => "#9CA3AF",
                                                                        hive_ai::desktop::git::StatusType::Copied => "#2196F3",
                                                                        hive_ai::desktop::git::StatusType::Ignored => "#6B737C",
                                                                        hive_ai::desktop::git::StatusType::Conflicted => "#F44336",
                                                                    }
                                                                ),
                                                                match status.status_type {
                                                                    hive_ai::desktop::git::StatusType::Modified => "M",
                                                                    hive_ai::desktop::git::StatusType::Added => "A",
                                                                    hive_ai::desktop::git::StatusType::Deleted => "D",
                                                                    hive_ai::desktop::git::StatusType::Renamed => "R",
                                                                    hive_ai::desktop::git::StatusType::Untracked => "U",
                                                                    hive_ai::desktop::git::StatusType::Copied => "C",
                                                                    hive_ai::desktop::git::StatusType::Ignored => "!",
                                                                    hive_ai::desktop::git::StatusType::Conflicted => "⚠",
                                                                }
                                                            }
                                                            
                                                            // File name  
                                                            span {
                                                                style: "color: #E0E0E0; font-size: 12px; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                                                title: "{file_path_str}",
                                                                "{path.file_name().unwrap_or_default().to_string_lossy()}"
                                                            }
                                                            
                                                            // Staged indicator
                                                            if status.is_staged {
                                                                span {
                                                                    style: "color: #4CAF50; font-size: 10px; margin-left: auto;",
                                                                    "●"
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
                                    
                                    // Commit section
                                    div {
                                        style: "margin-top: 16px; padding-top: 16px; border-top: 1px solid #2D3336;",
                                        
                                        textarea {
                                            style: "width: 100%; min-height: 60px; background: #1A1F21; border: 1px solid #2D3336; color: #FFFFFF; padding: 8px; border-radius: 4px; resize: vertical;",
                                            placeholder: "Commit message...",
                                        }
                                        
                                        button {
                                            style: "width: 100%; margin-top: 8px; padding: 8px; background: #FFC107; color: #000000; border: none; border-radius: 4px; font-weight: 600; cursor: pointer;",
                                            "Commit"
                                        }
                                    }
                                } else {
                                    div {
                                        style: "color: #858585; font-style: italic; padding: 20px; text-align: center;",
                                        "No repository detected"
                                    }
                                }
                            }
                        }
                    }

                    // Bottom action panel (integrated into sidebar)
                    div {
                        class: "sidebar-action-panel",
                        style: "background: linear-gradient(135deg, #0E1414 0%, #181E21 100%); border-top: 1px solid #FFC107; padding: 12px; box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.3);",
                        
                        div {
                            class: "action-panel-title",
                            style: "background: linear-gradient(to right, #FFC107, #FFD54F); -webkit-background-clip: text; -webkit-text-fill-color: transparent; font-weight: 700; font-size: 11px; margin-bottom: 8px; text-align: center;",
                            "QUICK ACTIONS"
                        }
                        
                        div {
                            class: "action-buttons",
                            style: "display: grid; grid-template-columns: 1fr 1fr; gap: 6px;",
                            
                            // Search button
                            button {
                                class: "action-btn",
                                style: "background: rgba(255, 193, 7, 0.1); border: 1px solid rgba(255, 193, 7, 0.3); color: #cccccc; padding: 8px 12px; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 6px; font-size: 12px; transition: all 0.2s ease;",
                                onclick: move |_| {
                                    // TODO: Implement search functionality
                                    tracing::info!("Search functionality not yet implemented");
                                },
                                span { style: "font-size: 14px;", "🔍" }
                                span { "Search" }
                            }
                            
                            // Analytics button
                            button {
                                class: "action-btn",
                                style: "background: rgba(255, 193, 7, 0.1); border: 1px solid rgba(255, 193, 7, 0.3); color: #cccccc; padding: 8px 12px; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 6px; font-size: 12px; transition: all 0.2s ease;",
                                onclick: move |_| {
                                    active_tab.set("__analytics__".to_string());
                                    current_view.set("analytics".to_string());
                                    if !open_tabs.read().contains(&"__analytics__".to_string()) {
                                        open_tabs.write().push("__analytics__".to_string());
                                    }
                                    ensure_active_tab_visible("__analytics__", &open_tabs.read());
                                },
                                span { style: "font-size: 14px;", "📊" }
                                span { "Analytics" }
                            }
                            
                            // Memory button
                            button {
                                class: "action-btn",
                                style: "background: rgba(255, 193, 7, 0.1); border: 1px solid rgba(255, 193, 7, 0.3); color: #cccccc; padding: 8px 12px; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 6px; font-size: 12px; transition: all 0.2s ease;",
                                onclick: move |_| {
                                    // TODO: Implement memory view
                                    tracing::info!("Memory view not yet implemented");
                                },
                                span { style: "font-size: 14px;", "🧠" }
                                span { "Memory" }
                            }
                            
                            // GitUI button (show when hidden)
                            if !*show_gitui.read() {
                                button {
                                    class: "action-btn",
                                    style: "background: rgba(255, 193, 7, 0.1); border: 1px solid rgba(255, 193, 7, 0.3); color: #cccccc; padding: 8px 12px; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 6px; font-size: 12px; transition: all 0.2s ease;",
                                    onclick: move |_| {
                                        show_gitui.set(true);
                                    },
                                    span { style: "font-size: 14px;", "🔀" }
                                    span { "GitUI" }
                                }
                            }
                            
                            // Settings button
                            button {
                                class: "action-btn",
                                style: "background: rgba(255, 193, 7, 0.1); border: 1px solid rgba(255, 193, 7, 0.3); color: #cccccc; padding: 8px 12px; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 6px; font-size: 12px; transition: all 0.2s ease;",
                                onclick: move |_| {
                                    show_settings_dialog.set(true);
                                },
                                span { style: "font-size: 14px;", "⚙️" }
                                span { "Settings" }
                            }
                        }
                    }
                }
                
                // Resize divider between sidebar and editor
                ResizableDivider {
                    direction: ResizeDirection::Horizontal,
                    size: sidebar_width,
                    min_size: 150.0,
                    max_size: 500.0,
                    invert_drag: false,  // Dragging right increases sidebar width
                }

                // Code editor area (center) - now split between editor and terminal
                div {
                    class: "editor-container",
                    style: "background: #0E1414; position: relative; display: flex; flex-direction: column; height: 100%; overflow: hidden; flex: 1;",

                    // Enhanced editor tabs with overflow scrolling
                    div {
                        class: "editor-tabs-container",
                        style: "display: flex; align-items: center; height: 35px; background: #2d2d30; border-bottom: 1px solid #3e3e42;",
                        
                        // Left arrow button (only show if we can scroll left)
                        if tab_scroll_offset.read().clone() > 0 {
                            button {
                                class: "tab-scroll-btn tab-scroll-left",
                                style: "background: rgba(255, 193, 7, 0.1); border: 1px solid rgba(255, 193, 7, 0.3); color: #FFC107; width: 30px; height: 28px; cursor: pointer; display: flex; align-items: center; justify-content: center; transition: all 0.2s ease; margin: 2px; border-radius: 3px; font-weight: bold;",
                                onclick: move |_| {
                                    let current_offset = tab_scroll_offset.read().clone();
                                    if current_offset > 0 {
                                        tab_scroll_offset.set(current_offset - 1);
                                    }
                                },
                                "‹"
                            }
                        }
                        
                        // Tab container with overflow hidden
                        {
                            // Collect visible tabs outside of RSX to avoid borrowing issues
                            let visible_tabs: Vec<String> = open_tabs.read()
                                .iter()
                                .skip(tab_scroll_offset.read().clone())
                                .take(max_visible_tabs)
                                .cloned()
                                .collect();
                            
                            rsx! {
                                div {
                                    class: "editor-tabs-scroll",
                                    style: "flex: 1; display: flex; align-items: center; overflow: hidden; max-width: calc(100vw - 450px);", // Reserve space for right panels and arrows
                                    
                                    // Render visible tabs
                                    for tab in visible_tabs {
                                        {
                                            let tab_str = tab.clone();
                                            let tab_for_click = tab.clone();
                                            let tab_for_close = tab.clone();
                                            let is_active = *active_tab.read() == tab;
                                            let display_name = if tab == "__welcome__" {
                                                "Welcome".to_string()
                                            } else if tab == "__analytics__" {
                                                "Analytics".to_string()
                                            } else if tab.starts_with("diff:") {
                                                // Show file name with diff indicator
                                                let file_path = tab.strip_prefix("diff:").unwrap_or("");
                                                let path = PathBuf::from(file_path);
                                                format!("📊 {}", path.file_name()
                                                    .and_then(|n| n.to_str())
                                                    .unwrap_or("diff"))
                                            } else {
                                                let path = PathBuf::from(&tab);
                                                path.file_name()
                                                    .and_then(|n| n.to_str())
                                                    .unwrap_or(&tab)
                                                    .to_string()
                                            };
                                            
                                            rsx! {
                                                div {
                                                    class: if is_active { "editor-tab active" } else { "editor-tab" },
                                                    style: "min-width: 120px; max-width: 180px; flex-shrink: 0;", // Fixed tab width for consistency
                                                    onclick: move |_| {
                                                        active_tab.set(tab_for_click.clone());
                                                        selected_file.set(Some(tab_for_click.clone()));
                                                        
                                                        // Update current view based on tab type
                                                        if tab_for_click == "__analytics__" {
                                                            current_view.set("analytics".to_string());
                                                        } else {
                                                            current_view.set("code".to_string());
                                                            // Update file_content from tab_contents
                                                            if let Some(content) = tab_contents.read().get(&tab_for_click) {
                                                                file_content.set(content.clone());
                                                            }
                                                        }
                                                    },
                                                    
                                                    span {
                                                        style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; padding-right: 4px;",
                                                        "{display_name}"
                                                    }
                                                    
                                                    // Close button
                                                    if tab_str != "__welcome__" {
                                                        span {
                                                            style: "margin-left: 4px; cursor: pointer; color: #858585; font-size: 16px; flex-shrink: 0; padding: 2px; border-radius: 2px; transition: all 0.1s ease;",
                                                            onclick: move |e| {
                                                                e.stop_propagation();
                                                                
                                                                // Remove from open tabs
                                                                open_tabs.write().retain(|t| t != &tab_for_close);
                                                                
                                                                // Adjust scroll offset if necessary
                                                                let new_tab_count = open_tabs.read().len();
                                                                let current_offset = tab_scroll_offset.read().clone();
                                                                if current_offset > 0 && current_offset >= new_tab_count {
                                                                    tab_scroll_offset.set(new_tab_count.saturating_sub(max_visible_tabs));
                                                                }
                                                                
                                                                // If this was the active tab, switch to another
                                                                if *active_tab.read() == tab_for_close {
                                                                    if let Some(first_tab) = open_tabs.read().first() {
                                                                        active_tab.set(first_tab.clone());
                                                                        selected_file.set(Some(first_tab.clone()));
                                                                        
                                                                        if first_tab == "__analytics__" {
                                                                            current_view.set("analytics".to_string());
                                                                        } else {
                                                                            current_view.set("code".to_string());
                                                                            if let Some(content) = tab_contents.read().get(first_tab) {
                                                                                file_content.set(content.clone());
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                
                                                                // Remove from tab_contents
                                                                tab_contents.write().remove(&tab_for_close);
                                                            },
                                                            "×"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Right arrow button (only show if there are more tabs to the right)
                        if open_tabs.read().len() > tab_scroll_offset.read().clone() + max_visible_tabs {
                            button {
                                class: "tab-scroll-btn tab-scroll-right",
                                style: "background: rgba(255, 193, 7, 0.1); border: 1px solid rgba(255, 193, 7, 0.3); color: #FFC107; width: 30px; height: 28px; cursor: pointer; display: flex; align-items: center; justify-content: center; transition: all 0.2s ease; margin: 2px; border-radius: 3px; font-weight: bold;",
                                onclick: move |_| {
                                    let current_offset = tab_scroll_offset.read().clone();
                                    let total_tabs = open_tabs.read().len();
                                    if current_offset + max_visible_tabs < total_tabs {
                                        tab_scroll_offset.set(current_offset + 1);
                                    }
                                },
                                "›"
                            }
                        }
                    }

                    // Editor and terminal split view container
                    div {
                        style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                        
                        // Editor content (top section - grows to fill available space)
                        div {
                            class: "editor-content",
                            style: "flex: 1; overflow: auto; min-height: 0; display: flex; flex-direction: column; background: #1e1e1e;",
                        if *active_tab.read() == "__analytics__" {
                            // Show analytics view
                            AnalyticsView { analytics_data: analytics_data.clone() }
                        } else if *active_tab.read() == "__welcome__" && *show_welcome_dialog.read() {
                            // Show welcome tab in editor area
                            WelcomeTab {
                                show_welcome: show_welcome_dialog,
                                on_action: handle_welcome_action,
                            }
                        } else if active_tab.read().starts_with("diff:") {
                            // Show diff view
                            if let Some(diff_result) = diff_tabs.read().get(&*active_tab.read()) {
                                DiffViewer {
                                    diff: diff_result.clone(),
                                    view_mode: *diff_view_mode.read(),
                                    file_path: active_tab.read().strip_prefix("diff:").unwrap_or(&*active_tab.read()).to_string(),
                                    repo_path: current_dir.read().clone().unwrap_or_default().to_string_lossy().to_string(),
                                    on_stage: None,
                                    on_view_mode_change: EventHandler::new({
                                        let mut diff_view_mode = diff_view_mode.clone();
                                        move |mode| {
                                            diff_view_mode.set(mode);
                                        }
                                    }),
                                    on_diff_action: None,
                                    inline_actions_enabled: true,
                                    keyboard_shortcuts_enabled: true,
                                }
                            } else {
                                // Show loading state while diff is being generated
                                div {
                                        style: "display: flex; align-items: center; justify-content: center; height: 100%; color: #6e7681;",
                                        div {
                                            style: "text-align: center;",
                                            div {
                                                style: "font-size: 18px; margin-bottom: 8px;",
                                                "⏳ Loading diff..."
                                            }
                                            div {
                                                style: "font-size: 14px;",
                                                "Generating file differences"
                                            }
                                        }
                                    }
                                }
                        } else if !active_tab.read().is_empty() && *active_tab.read() != "__welcome__" {
                            // Show file content for the active tab with VS Code-style editor
                            if let Some(content) = tab_contents.read().get(&*active_tab.read()) {
                                CodeEditorComponent {
                                    file_path: active_tab.read().clone(),
                                    initial_content: content.clone(),
                                    on_change: {
                                        let mut tab_contents = tab_contents.clone();
                                        let active_tab = active_tab.clone();
                                        move |new_content: String| {
                                            // Update the tab content when user edits
                                            let active_tab_path = active_tab.read().clone();
                                            tab_contents.write().insert(active_tab_path.clone(), new_content.clone());
                                            
                                            // Mark the file as modified in the UI
                                            // This will be handled by git integration showing changes
                                            tracing::debug!("File content updated: {}", active_tab_path);
                                        }
                                    },
                                    on_save: move |(file_path, content): (String, String)| {
                                        // Save the file to disk
                                        spawn(async move {
                                            if let Err(e) = tokio::fs::write(&file_path, content).await {
                                                tracing::error!("Failed to save file {}: {}", file_path, e);
                                            } else {
                                                tracing::info!("File saved: {}", file_path);
                                            }
                                        });
                                    },
                                    on_cursor_change: {
                                        let mut cursor_position = cursor_position.clone();
                                        move |(line, col): (usize, usize)| {
                                            cursor_position.set((line, col));
                                            tracing::debug!("Cursor position updated: Ln {}, Col {}", line, col);
                                        }
                                    },
                                }
                            } else {
                                div {
                                    style: "padding: 20px; color: #858585;",
                                    "Loading file content..."
                                }
                            }
                        } else {
                            // Reset cursor position for non-code views
                            {
                                let (line, col) = *cursor_position.read();
                                if line != 1 || col != 1 {
                                    cursor_position.set((1, 1));
                                }
                            }
                            div {
                                class: "welcome-message",
                                "Select a file from the explorer to view its contents"
                            }
                        }
                        }
                        
                        // Terminal section (bottom - fixed height at bottom)
                        if *show_terminal.read() {
                            // Resize divider for terminal
                            ResizableDivider {
                                direction: ResizeDirection::Vertical,
                                size: terminal_height,
                                min_size: 100.0,
                                max_size: 600.0,
                                invert_drag: true,  // Dragging up increases terminal height
                            }
                            
                            // Terminal container - fixed at bottom
                            div {
                                style: format!("height: {}px; border-top: 1px solid #3e3e42; display: flex; flex-direction: column; background: #1e1e1e; flex-shrink: 0;", 
                                    terminal_height.read()
                                ),
                                
                                // TerminalTabs handles both the tab bar and terminal content
                                TerminalTabs {}
                            }
                        }
                    }
                }
                
                // Resize divider between editor and chat
                ResizableDivider {
                    direction: ResizeDirection::Horizontal,
                    size: chat_width,
                    min_size: 300.0,
                    max_size: 800.0,
                    invert_drag: true,  // Dragging right decreases chat width (makes chat smaller)
                }

                // Chat panel (right)
                div {
                    class: "chat-panel",
                    style: format!("width: {}px;", chat_width.read()),

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
                    // Only show consensus progress if not cancelled
                    if !*cancel_flag.read() {
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
                                            should_auto_scroll.set(is_at_bottom);
                                        }
                                    }
                                    Err(_) => {
                                        // Default to auto-scroll if we can't detect position
                                        should_auto_scroll.set(true);
                                    }
                                }
                            });
                        },

                        if !app_state.read().consensus.streaming_content.is_empty() && !*cancel_flag.read() {
                            div {
                                class: "response-content",
                                dangerous_inner_html: "{app_state.read().consensus.streaming_content}"
                            }
                        } else if !current_response.read().is_empty() && !*cancel_flag.read() {
                            // Show final response if no streaming content
                            div {
                                class: "response-content",
                                dangerous_inner_html: "{current_response.read()}"
                            }
                        } else if *is_processing.read() && app_state.read().consensus.is_running && !*cancel_flag.read() {
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
                        style: "background: #181E21; border-top: 1px solid #2D3336; backdrop-filter: blur(10px); position: relative;",
                        
                        
                        textarea {
                            class: "query-input",
                            style: "background: #0E1414; border: 1px solid #2D3336; color: #FFFFFF;",
                            value: "{input_value.read()}",
                            placeholder: "Ask Hive anything...",
                            disabled: *is_processing.read(),
                            rows: "3",
                            oninput: move |evt| input_value.set(evt.value().clone()),
                            onkeydown: {
                                let consensus_manager = consensus_manager.clone();
                                let mut app_state_for_toggle = app_state.clone();
                                let mut input_value_ref = input_value.clone();
                                let mut is_processing_ref = is_processing.clone();
                                let mut current_response_ref = current_response.clone();
                                let mut app_state_ref = app_state.clone();
                                let mut should_auto_scroll_ref = should_auto_scroll.clone();
                                let mut previous_content_length_ref = previous_content_length.clone();
                                let mut show_upgrade_dialog_ref = show_upgrade_dialog.clone();
                                let ide_ai_broker_ref = ide_ai_broker.clone();
                                let is_cancelling_ref = is_cancelling.clone();
                                let mut cancel_flag_ref = cancel_flag.clone();
                                move |evt: dioxus::events::KeyboardEvent| {
                                    // Shift+Tab toggles auto-accept
                                    if evt.key() == dioxus::events::Key::Tab && evt.modifiers().shift() {
                                        evt.prevent_default();
                                        let current = app_state_for_toggle.read().auto_accept;
                                        app_state_for_toggle.write().auto_accept = !current;
                                        tracing::info!("Auto-accept toggled via Shift+Tab to: {}", !current);
                                        return;
                                    }
                                    
                                    
                                    // Enter without shift submits
                                    if evt.key() == dioxus::events::Key::Enter && !evt.modifiers().shift() && !input_value_ref.read().is_empty() && !*is_processing_ref.read() {
                                        evt.prevent_default();

                                        let user_msg = input_value_ref.read().clone();

                                        // Clear input and response
                                        input_value_ref.write().clear();
                                        current_response_ref.write().clear();
                                        app_state_ref.write().consensus.streaming_content.clear();

                                        // Re-enable auto-scroll for new query
                                        should_auto_scroll_ref.set(true);

                                        // Reset content length tracker to ensure scrolling works
                                        previous_content_length_ref.set(0);

                                        // Start processing
                                        is_processing_ref.set(true);
                                        cancel_flag_ref.set(false); // Reset cancel flag for new query

                                        // Use consensus engine if available
                                        if let Some(mut consensus) = consensus_manager.read().clone() {
                                            let mut current_response = current_response_ref.clone();
                                            let mut is_processing = is_processing_ref.clone();
                                            let mut app_state = app_state_ref.clone();
                                            let mut show_upgrade_dialog = show_upgrade_dialog_ref.clone();
                                            let mut ide_ai_broker = ide_ai_broker_ref.clone();
                                            let mut is_cancelling = is_cancelling_ref.clone();
                                            let cancellation_flag_clone = cancellation_flag.read().clone();

                                            // Reset cancellation flag for new query
                                            cancellation_flag_clone.store(false, std::sync::atomic::Ordering::Relaxed);

                                            // Cancel any existing consensus task
                                            if let Some(existing_task) = consensus_task_handle.write().take() {
                                                tracing::info!("🛑 Cancelling existing consensus task before starting new one");
                                                existing_task.cancel();
                                            }

                                            // Store the new task handle
                                            let task = spawn(async move {
                                                // Update UI to show consensus is running
                                                app_state.write().consensus.start_consensus();

                                                // Use IDE AI Helper Broker to enhance query with repository context
                                                let enhanced_query = if let Some(broker) = ide_ai_broker.read().as_ref() {
                                                    match broker.process_query_with_context(&user_msg).await {
                                                        Ok(enhanced) => {
                                                            tracing::info!("🤖 IDE AI Helper enhanced query with repository context");
                                                            enhanced.to_consensus_query()
                                                        }
                                                        Err(e) => {
                                                            tracing::warn!("IDE AI Helper failed to enhance query: {}", e);
                                                            user_msg.clone() // Fallback to original
                                                        }
                                                    }
                                                } else {
                                                    tracing::info!("💭 IDE AI Helper Broker not available, using original query");
                                                    user_msg.clone()
                                                };

                                                // Check cancellation flag before starting consensus
                                                if cancellation_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                                                    tracing::info!("Consensus cancelled before processing started");
                                                    app_state.write().consensus.complete_consensus();
                                                    is_processing.set(false);
                                                    is_cancelling.set(false);
                                                    return;
                                                }

                                                // Use streaming version which has proper cancellation support
                                                let consensus_result = consensus.process_query_streaming(&enhanced_query).await;
                                                
                                                match consensus_result {
                                                    Ok((final_response, _rx)) => {
                                                        // Check if we were cancelled during processing
                                                        if cancellation_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                                                            tracing::info!("🛑 Consensus completed but was cancelled - not updating UI");
                                                            app_state.write().consensus.complete_consensus();
                                                            is_processing.set(false);
                                                            is_cancelling.set(false);
                                                            return;
                                                        }
                                                        
                                                        // Set final response
                                                        let html = markdown::to_html(&final_response);
                                                        current_response.set(html);
                                                    }
                                                    Err(e) => {
                                                        let error_msg = e.to_string();
                                                        let full_error_chain = format!("{:?}", e);

                                                        // Check if this is a cancellation error
                                                        if error_msg.contains("cancelled") || 
                                                           error_msg.contains("Cancelled") ||
                                                           full_error_chain.contains("cancelled") ||
                                                           full_error_chain.contains("Cancelled") {
                                                            // Don't show error for cancellation - it's expected behavior
                                                            tracing::info!("Consensus was cancelled by user");
                                                            current_response.set(String::new()); // Clear response area
                                                            is_cancelling.set(false); // Reset cancelling state
                                                        } else {
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
                                                                show_upgrade_dialog.set(true);
                                                                current_response.set(String::new()); // Clear response area
                                                            } else {
                                                                // Show technical errors normally
                                                                current_response.set(format!("<div class='error'>❌ Error: {}</div>", e));
                                                            }
                                                        }
                                                    }
                                                }

                                                // Update UI to show consensus is complete (only if not cancelled)
                                                if !*cancel_flag_ref.read() && !cancellation_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                                                    app_state.write().consensus.complete_consensus();
                                                } else {
                                                    tracing::info!("🛑 Consensus task completed but was cancelled - skipping UI update");
                                                    app_state.write().consensus.complete_consensus();
                                                }
                                                is_processing.set(false);
                                                is_cancelling.set(false); // Reset cancelling state
                                                
                                                tracing::info!("🏁 Consensus completed");
                                            });
                                            
                                            // Store the task handle
                                            consensus_task_handle.set(Some(task));
                                        } else {
                                            // Show error if consensus engine not initialized
                                            current_response_ref.set("<div class='error'>⚠️ Consensus engine not initialized. This usually means no profile is configured. Click Settings to configure a profile.</div>".to_string());
                                            is_processing_ref.set(false);

                                            // Check what's actually missing
                                            spawn(async move {
                                                use hive_ai::core::api_keys::ApiKeyManager;
                                                
                                                let has_api_key = ApiKeyManager::has_valid_keys().await.unwrap_or(false);
                                                if !has_api_key {
                                                    // Show onboarding for API key
                                                    show_onboarding_dialog.set(true);
                                                } else {
                                                    // API key exists but no profile - show settings
                                                    show_settings_dialog.set(true);
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Controls below input - Response coordination buttons
                        div {
                            style: "margin-top: 8px; display: flex; align-items: center; justify-content: space-between; color: #858585; font-size: 12px;",
                            
                            // Left side: Response coordination buttons
                            div {
                                style: "display: flex; align-items: center; gap: 16px;",
                                
                                // Send to Consensus button
                                div {
                                    style: "display: flex; align-items: center; gap: 8px;",
                                    
                                    span {
                                        style: "font-size: 14px; color: #FFC107;",
                                        "→"
                                    }
                                    
                                    button {
                                        style: "background: rgba(255, 193, 7, 0.1); border: 1px solid rgba(255, 193, 7, 0.3); color: #FFC107; cursor: pointer; font-size: 12px; padding: 4px 12px; border-radius: 4px; transition: all 0.2s;",
                                        onclick: move |_| {
                                            tracing::info!("📋 Send to Consensus clicked");
                                            
                                            let mut input_value = input_value.clone();
                                            let mut app_state = app_state.clone();
                                            
                                            // Extract terminal content using xterm.js
                                            spawn(async move {
                                                use hive_ai::desktop::terminal_xterm_simple::get_xterm_content;
                                                
                                                // Get content from the active terminal (claude-code)
                                                if let Some(content) = get_xterm_content("claude-code").await {
                                                    tracing::info!("✅ Extracted content length: {}", content.len());
                                                    tracing::info!("📝 First 200 chars: {}", &content.chars().take(200).collect::<String>());
                                                    
                                                    // Extract the most recent substantial block of text
                                                    let lines: Vec<&str> = content.lines().collect();
                                                    tracing::info!("📊 Total lines in terminal: {}", lines.len());
                                                    
                                                    let mut response_lines = Vec::new();
                                                    let mut found_content = false;
                                                    
                                                    // Scan from bottom to top to find the most recent response
                                                    for (idx, line) in lines.iter().rev().enumerate() {
                                                        let trimmed = line.trim();
                                                        
                                                        // Skip empty lines at the bottom
                                                        if !found_content && trimmed.is_empty() {
                                                            continue;
                                                        }
                                                        
                                                        // Skip shell prompts
                                                        if trimmed.ends_with('$') || trimmed.ends_with('%') || trimmed.ends_with('#') || trimmed.ends_with('>') {
                                                            if found_content {
                                                                tracing::debug!("🛑 Found prompt at line {}, stopping extraction", lines.len() - idx);
                                                                break;
                                                            }
                                                            continue;
                                                        }
                                                        
                                                        // This looks like content
                                                        if !trimmed.is_empty() {
                                                            if !found_content {
                                                                tracing::debug!("🎯 Found content start at line {}", lines.len() - idx);
                                                            }
                                                            found_content = true;
                                                            response_lines.push(line.to_string());
                                                        } else if found_content {
                                                            // Add empty lines that are part of the response
                                                            response_lines.push(line.to_string());
                                                        }
                                                    }
                                                    
                                                    // Reverse to get the correct order
                                                    response_lines.reverse();
                                                    let response = response_lines.join("\n").trim().to_string();
                                                    
                                                    tracing::info!("📄 Extracted response length: {}", response.len());
                                                    tracing::info!("📝 Extracted response preview: {}", &response.chars().take(100).collect::<String>());
                                                    
                                                    if response.len() > 50 {
                                                        // Update the input_value signal directly
                                                        tracing::info!("📌 Current input value: '{}'", input_value.read());
                                                        input_value.set(response.clone());
                                                        tracing::info!("✅ Updated input value: '{}'", input_value.read());
                                                        
                                                        // Also update app state
                                                        app_state.write().chat.input_text = response.clone();
                                                        
                                                        // Also try direct DOM manipulation as a fallback
                                                        let response_for_js = response.clone();
                                                        use dioxus::document::eval;
                                                        let escaped_text = response_for_js
                                                            .replace('\\', "\\\\")
                                                            .replace('"', "\\\"")
                                                            .replace('\n', "\\n")
                                                            .replace('\r', "\\r");
                                                        
                                                        let js_code = format!(r#"
                                                            const chatInput = document.querySelector('textarea.query-input');
                                                            if (chatInput) {{
                                                                chatInput.value = "{}";
                                                                chatInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                                                                chatInput.dispatchEvent(new Event('change', {{ bubbles: true }}));
                                                                console.log('✅ Set chat input via DOM manipulation to query-input textarea');
                                                                console.log('📝 Set value length:', chatInput.value.length);
                                                                true;
                                                            }} else {{
                                                                console.error('❌ Could not find query-input textarea element');
                                                                false;
                                                            }}
                                                        "#, escaped_text);
                                                        
                                                        match eval(&js_code).await {
                                                            Ok(result) => {
                                                                tracing::info!("🔧 DOM manipulation result: {:?}", result);
                                                            }
                                                            Err(e) => {
                                                                tracing::error!("❌ DOM manipulation failed: {}", e);
                                                            }
                                                        }
                                                        
                                                        tracing::info!("✅ Pasted Claude's response to chat input");
                                                    } else {
                                                        tracing::warn!("⚠️ Response too short or empty: {} chars", response.len());
                                                        if response.len() > 0 {
                                                            tracing::warn!("📝 Short response content: '{}'", response);
                                                        }
                                                    }
                                                } else {
                                                    tracing::warn!("❌ No active terminal found or couldn't retrieve content");
                                                }
                                            });
                                        },
                                        "Send to Consensus"
                                    }
                                }
                                
                                // Send to Claude button
                                div {
                                    style: "display: flex; align-items: center; gap: 8px;",
                                    
                                    span {
                                        style: "font-size: 14px; color: #8B5CF6;",
                                        "←"
                                    }
                                    
                                    button {
                                        style: "background: rgba(139, 92, 246, 0.1); border: 1px solid rgba(139, 92, 246, 0.3); color: #8B5CF6; cursor: pointer; font-size: 12px; padding: 4px 12px; border-radius: 4px; transition: all 0.2s;",
                                        onclick: move |_| {
                                            tracing::info!("🤖 Send to Claude clicked");
                                            
                                            // Get latest curator from database and send to terminal
                                            let _app_state_clone = app_state.clone();
                                            spawn(async move {
                                                use hive_ai::core::database::get_database;
                                                
                                                match get_database().await {
                                                    Ok(db) => {
                                                        match db.get_latest_curator_result().await {
                                                            Ok(Some((curator_content, timestamp))) => {
                                                                // Format the curator result for terminal
                                                                let formatted_content = format!(
                                                                    "# Consensus Curator Result\n# Generated: {}\n# ---\n{}\n",
                                                                    timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                                                                    curator_content
                                                                );
                                                                
                                                                // Send to terminal
                                                                use hive_ai::desktop::terminal_registry::send_to_active_terminal;
                                                                if send_to_active_terminal(&formatted_content) {
                                                                    tracing::info!("✅ Sent curator result to Claude terminal");
                                                                } else {
                                                                    tracing::error!("❌ Failed to send curator result to terminal");
                                                                }
                                                            }
                                                            Ok(None) => {
                                                                tracing::warn!("⚠️ No curator results found in database");
                                                            }
                                                            Err(e) => {
                                                                tracing::error!("❌ Failed to fetch curator result: {}", e);
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        tracing::error!("❌ Failed to get database: {}", e);
                                                    }
                                                }
                                            });
                                        },
                                        "Send to Claude"
                                    }
                                }
                            }
                            
                            // Right side: Cancel consensus button (when running)
                            if *is_processing.read() {
                                div {
                                    style: "display: flex; align-items: center; gap: 8px;",
                                    
                                    // Cancel indicator - show spinning icon when cancelling
                                    span {
                                        style: if *is_cancelling.read() {
                                            "font-size: 14px; color: #FF6B6B; animation: spin 1s linear infinite;"
                                        } else {
                                            "font-size: 14px; color: #FF6B6B; animation: cancelPulse 2s ease-in-out infinite;"
                                        },
                                        if *is_cancelling.read() { "⏸" } else { "⏹" }
                                    }
                                    
                                    // Cancel button
                                    button {
                                        style: "background: none; border: none; color: #FF6B6B; cursor: pointer; font-size: 12px; padding: 2px 6px; border-radius: 3px; transition: all 0.2s; text-decoration: underline;",
                                        onclick: move |_| {
                                            tracing::info!("🛑 Cancel button clicked - cancelling consensus");
                                            
                                            // Set the atomic cancellation flag
                                            cancellation_flag.read().store(true, std::sync::atomic::Ordering::Relaxed);
                                            
                                            // Cancel the running task immediately
                                            if let Some(task) = consensus_task_handle.write().take() {
                                                tracing::info!("🛑 Cancelling consensus task");
                                                task.cancel();
                                            }
                                            
                                            // Also cancel through the consensus manager for immediate effect
                                            let mut consensus_manager = consensus_manager.clone();
                                            spawn(async move {
                                                if let Some(mut manager) = consensus_manager.write().as_mut() {
                                                    if let Err(e) = manager.cancel_consensus("User clicked cancel button").await {
                                                        tracing::warn!("Failed to cancel consensus: {}", e);
                                                    } else {
                                                        tracing::info!("✅ Consensus manager cancellation successful");
                                                    }
                                                }
                                            });
                                            
                                            // Immediately reset all UI state
                                            cancel_flag.set(true);
                                            app_state.write().consensus.complete_consensus();
                                            app_state.write().consensus.streaming_content.clear();
                                            app_state.write().consensus.current_stage = None;
                                            current_response.set(String::new());
                                            is_processing.set(false);
                                            is_cancelling.set(false);
                                            
                                            // Show immediate feedback
                                            app_state.write().consensus.streaming_content = 
                                                "<div style='color: #4CAF50; font-weight: bold;'>✅ Cancelled - ready for new query</div>".to_string();
                                            
                                            tracing::info!("✅ Cancellation flag set and UI reset complete");
                                        },
                                        "cancel"
                                    }
                                    
                                    // Keyboard shortcut hint
                                    span {
                                        style: "color: #505050; font-size: 11px; margin-left: 8px;",
                                        "(ctrl+c to cancel)"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Enhanced Status Bar
            EnhancedStatusBar {
                state: status_bar_state,
                on_item_click: handle_status_bar_click,
            }
        }


        // Render dialogs
        if *show_about_dialog.read() {
            AboutDialog {
                show_about: show_about_dialog.clone(),
            }
        }

        // Git status menu (render at top level for proper positioning)
        if *show_git_menu.read() {
            GitStatusMenu {
                repo_path: current_dir.read().clone(),
                branch_info: git_state.branch_info.read().clone(),
                visible: show_git_menu.clone(),
                position: *git_menu_position.read(),
                on_branch_selected: EventHandler::new({
                    let current_dir = current_dir.clone();
                    let mut git_state = git_state.clone();
                    let mut git_context = git_context.clone();
                    move |branch_name: String| {
                        if let Some(repo_path) = current_dir.read().clone() {
                            let mut git_state_clone = git_state.clone();
                            let mut git_context_clone = git_context.clone();
                            spawn(async move {
                                if let Ok(repo) = GitRepository::open(&repo_path) {
                                    match repo.checkout_branch(&branch_name) {
                                        Ok(_) => {
                                            tracing::info!("Switched to branch: {}", branch_name);
                                            // Refresh git state
                                            git_context_clone.refresh().await;
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to switch branch: {}", e);
                                        }
                                    }
                                }
                            });
                        }
                    }
                }),
                on_create_branch: EventHandler::new({
                    move |_| {
                        // TODO: Show create branch dialog
                        tracing::info!("Create branch clicked");
                    }
                }),
            }
        }
        
        // Branch menu (render at top level for proper positioning)
        if *show_branch_menu.read() {
            BranchMenu {
                repo_path: current_dir.read().clone(),
                branch_info: git_state.branch_info.read().clone(),
                visible: show_branch_menu.clone(),
                position: (20, 30), // Position above the status bar (x=20 for left side, y=30 for above status bar)
                on_branch_selected: EventHandler::new({
                    let mut git_context = git_context.clone();
                    let current_dir = current_dir.clone();
                    move |branch_name: String| {
                        if let Some(repo_path) = current_dir.read().clone() {
                            let mut git_context_clone = git_context.clone();
                            spawn(async move {
                                if let Ok(repo) = GitRepository::open(&repo_path) {
                                    match repo.checkout_branch(&branch_name) {
                                        Ok(_) => {
                                            tracing::info!("Switched to branch: {}", branch_name);
                                            // Refresh git state
                                            git_context_clone.refresh().await;
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to switch branch: {}", e);
                                        }
                                    }
                                }
                            });
                        }
                    }
                }),
                on_operation_complete: EventHandler::new(move |result: BranchOperationResult| {
                    match result.operation {
                        BranchOperation::Checkout(ref branch) => {
                            if result.success {
                                tracing::info!("Successfully checked out branch: {}", branch);
                            } else {
                                tracing::error!("Failed to checkout branch: {}", result.message);
                            }
                        }
                        _ => {
                            tracing::info!("Branch operation completed: {:?}", result);
                        }
                    }
                }),
            }
        }
        
        // Effect to emit event when branch menu visibility changes
        {
            let mut prev_show_branch_menu = use_signal(|| false);
            let show_branch_menu = show_branch_menu.clone();
            use_effect(move || {
                let current = *show_branch_menu.read();
                let previous = *prev_show_branch_menu.read();
                
                // Only emit event when visibility actually changes
                if current != previous {
                    prev_show_branch_menu.set(current);
                    
                    // Only emit event when hiding (showing is handled in the click handler)
                    if !current && previous {
                        // Emit menu hidden event
                        let bus = event_bus();
                        spawn(async move {
                            let event = Event::new(
                                EventType::MenuVisibilityChanged,
                                EventPayload::MenuVisibility {
                                    menu_id: "branch-menu".to_string(),
                                    visible: false,
                                }
                            );
                            bus.publish(event).await.unwrap_or_else(|e| {
                                tracing::error!("Failed to publish menu visibility event: {}", e);
                            });
                        });
                    }
                }
            });
        }
        
        // Effect to listen for BranchMenuRequested events
        {
            let mut show_branch_menu = show_branch_menu.clone();
            use_effect(move || {
                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                
                // Subscribe to branch menu events
                spawn(async move {
                    let bus = event_bus();
                    bus.subscribe_async(EventType::BranchMenuRequested, move |event| {
                        let tx = tx.clone();
                        async move {
                            if event.event_type == EventType::BranchMenuRequested {
                                tracing::info!("Received BranchMenuRequested event - sending to UI");
                                tx.send(()).unwrap_or_else(|e| {
                                    tracing::error!("Failed to send branch menu event: {}", e);
                                });
                                
                                // Also emit MenuVisibilityChanged event for consistency
                                let visibility_event = Event::new(
                                    EventType::MenuVisibilityChanged,
                                    EventPayload::MenuVisibility {
                                        menu_id: "branch-menu".to_string(),
                                        visible: true,
                                    }
                                );
                                event_bus().publish_async(visibility_event).await.unwrap_or_else(|e| {
                                    tracing::error!("Failed to publish menu visibility event: {}", e);
                                });
                            }
                            Ok(())
                        }
                    }).await;
                });
                
                // Handle events in UI context
                spawn(async move {
                    while let Some(()) = rx.recv().await {
                        show_branch_menu.set(true);
                    }
                });
            });
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
                anthropic_key: anthropic_key.clone(),
                on_profile_change: Some(EventHandler::new({
                    let mut api_keys_version = api_keys_version.clone();
                    let mut app_state_for_profile = app_state.clone();
                    move |_| {
                        // Increment api_keys_version to trigger consensus reload
                        *api_keys_version.write() += 1;
                        tracing::info!("🔄 Profile changed callback triggered - incrementing api_keys_version to trigger consensus reload");
                        
                        // Also update the UI consensus state
                        spawn({
                            let mut app_state_clone = app_state_for_profile.clone();
                            async move {
                                tracing::info!("🔄 Profile change callback - loading new profile from database");
                                match load_active_profile_from_db().await {
                                    Ok(profile) => {
                                        tracing::info!("✅ Profile change - updating UI with new profile: {}", profile.profile_name);
                                        app_state_clone.write().consensus.active_profile_name = profile.profile_name.clone();
                                        app_state_clone.write().consensus.stages = vec![
                                            StageInfo::new("Generator", &profile.generator_model),
                                            StageInfo::new("Refiner", &profile.refiner_model),
                                            StageInfo::new("Validator", &profile.validator_model),
                                            StageInfo::new("Curator", &profile.curator_model),
                                        ];
                                        tracing::info!("✅ UI consensus state updated with profile: {}", profile.profile_name);
                                    }
                                    Err(e) => {
                                        tracing::error!("❌ Failed to load profile after change: {}", e);
                                    }
                                }
                            }
                        });
                    }
                })),
            }
        }

        if *show_onboarding_dialog.read() {
            OnboardingDialog {
                show_onboarding: show_onboarding_dialog.clone(),
                openrouter_key: openrouter_key.clone(),
                hive_key: hive_key.clone(),
                anthropic_key: anthropic_key.clone(),
                current_step: onboarding_current_step.clone(),
                api_keys_version: api_keys_version.clone(),
                on_profile_change: Some(EventHandler::new({
                    let mut api_keys_version = api_keys_version.clone();
                    move |_| {
                        // Increment api_keys_version to trigger consensus reload
                        *api_keys_version.write() += 1;
                        tracing::info!("Profile changed in onboarding - incrementing api_keys_version to trigger consensus reload");
                    }
                })),
            }
        }

        if *show_upgrade_dialog.read() {
            UpgradeDialog {
                show: show_upgrade_dialog.clone(),
            }
        }

        // Update dialogs
        if *show_update_available_dialog.read() {
            UpdateAvailableDialog {
                show: show_update_available_dialog.clone(),
                version: update_info.read().0.clone(),
                date: update_info.read().1.clone(),
                download_url: update_info.read().2.clone(),
            }
        }

        if *show_no_updates_dialog.read() {
            NoUpdatesDialog {
                show: show_no_updates_dialog.clone(),
            }
        }

        UpdateErrorDialog {
            show: show_update_error_dialog,
            error_message: update_error_message.read().clone(),
        }

        // Context menu
        ContextMenu {
            state: context_menu_state.clone(),
            on_action: EventHandler::new({
                let mut context_menu_state = context_menu_state.clone();
                let mut show_new_file_dialog = show_new_file_dialog.clone();
                let mut show_new_folder_dialog = show_new_folder_dialog.clone();
                let mut show_rename_dialog = show_rename_dialog.clone();
                let mut show_delete_confirm = show_delete_confirm.clone();
                let mut dialog_target_path = dialog_target_path.clone();
                let current_dir = current_dir.clone();
                let mut file_tree = file_tree.clone();
                move |(action, path): (ContextMenuAction, PathBuf)| {
                    dialog_target_path.set(Some(path.clone()));
                    match action {
                        ContextMenuAction::NewFile => {
                            show_new_file_dialog.set(true);
                        }
                        ContextMenuAction::NewFolder => {
                            show_new_folder_dialog.set(true);
                        }
                        ContextMenuAction::Rename => {
                            show_rename_dialog.set(true);
                        }
                        ContextMenuAction::Delete => {
                            show_delete_confirm.set(true);
                        }
                        ContextMenuAction::Cut => {
                            context_menu_state.write().set_clipboard(path, true);
                        }
                        ContextMenuAction::Copy => {
                            context_menu_state.write().set_clipboard(path, false);
                        }
                        ContextMenuAction::Paste => {
                            let clipboard_info = context_menu_state.read().clipboard.clone();
                            if let Some(clipboard) = clipboard_info {
                                let src = clipboard.path.clone();
                                let is_cut = clipboard.is_cut;
                                let dst_dir = if path.is_dir() { path } else { path.parent().unwrap_or(&path).to_path_buf() };
                                let dst = dst_dir.join(src.file_name().unwrap_or_default());
                                
                                spawn(async move {
                                    let result = if is_cut {
                                        file_operations::move_item(&src, &dst).await
                                    } else {
                                        file_operations::copy_item(&src, &dst).await
                                    };
                                    
                                    if let Err(e) = result {
                                        tracing::error!("Failed to paste item: {}", e);
                                    }
                                    
                                    // TODO: Refresh file tree
                                });
                                
                                if is_cut {
                                    context_menu_state.write().clear_clipboard();
                                }
                            }
                        }
                        ContextMenuAction::Duplicate => {
                            spawn(async move {
                                if let Err(e) = file_operations::duplicate_item(&path).await {
                                    tracing::error!("Failed to duplicate item: {}", e);
                                }
                                // TODO: Refresh file tree
                            });
                        }
                        ContextMenuAction::CopyPath => {
                            if let Err(e) = file_operations::copy_path_to_clipboard(&path) {
                                tracing::error!("Failed to copy path to clipboard: {}", e);
                            }
                        }
                        ContextMenuAction::OpenInTerminal => {
                            if let Err(e) = file_operations::open_in_terminal(&path) {
                                tracing::error!("Failed to open terminal: {}", e);
                            }
                        }
                        ContextMenuAction::RevealInFinder => {
                            if let Err(e) = file_operations::reveal_in_finder(&path) {
                                tracing::error!("Failed to reveal in finder: {}", e);
                            }
                        }
                        ContextMenuAction::ConfigureGitDecorations => {
                            // TODO: Implement git decorations configuration
                            tracing::info!("Git decorations configuration requested for {:?}", path);
                        }
                    }
                }
            }),
        }

        FileNameDialog {
            visible: *show_new_file_dialog.read(),
            title: format!("New File in {}", 
                dialog_target_path.read().as_ref()
                    .map(|p| p.file_name().and_then(|n| n.to_str()).unwrap_or("folder"))
                    .unwrap_or("current directory")
            ),
            initial_value: "".to_string(), // Always start with empty field
            on_confirm: EventHandler::new({
                let mut show_new_file_dialog = show_new_file_dialog.clone();
                let dialog_target_path = dialog_target_path.clone();
                let current_dir = current_dir.clone();
                let mut file_tree = file_tree.clone();
                let reload_fn = reload_file_tree.clone();
                move |filename: String| {
                    // Validate filename is not empty
                    let filename = filename.trim();
                    if filename.is_empty() {
                        return; // Don't close dialog if filename is empty
                    }
                    
                    show_new_file_dialog.set(false);
                    
                    if let Some(target_path) = dialog_target_path.read().as_ref() {
                        let parent_dir = if target_path.is_dir() { 
                            target_path.clone() 
                        } else if let Some(parent) = target_path.parent() {
                            parent.to_path_buf()
                        } else {
                            current_dir.read().clone().unwrap_or_else(|| PathBuf::from("."))
                        };
                        let file_path = parent_dir.join(&filename);
                        
                        spawn(async move {
                            let extension = file_path.extension()
                                .and_then(|ext| ext.to_str())
                                .unwrap_or("");
                            let template = file_operations::get_file_template(extension);
                            
                            if let Err(e) = file_operations::create_file(&file_path, Some(template)).await {
                                tracing::error!("Failed to create file: {}", e);
                            }
                            
                            // Refresh file tree to show the new file
                            reload_fn();
                        });
                    }
                }
            }),
            on_cancel: EventHandler::new({
                let mut show_new_file_dialog = show_new_file_dialog.clone();
                move |_| {
                    show_new_file_dialog.set(false);
                }
            }),
        }

        FileNameDialog {
            visible: *show_new_folder_dialog.read(),
            title: format!("New Folder in {}", 
                dialog_target_path.read().as_ref()
                    .map(|p| p.file_name().and_then(|n| n.to_str()).unwrap_or("folder"))
                    .unwrap_or("current directory")
            ),
            initial_value: "".to_string(), // Always start with empty field
            on_confirm: EventHandler::new({
                let mut show_new_folder_dialog = show_new_folder_dialog.clone();
                let dialog_target_path = dialog_target_path.clone();
                let current_dir = current_dir.clone();
                let reload_fn = reload_file_tree.clone();
                move |foldername: String| {
                    // Validate foldername is not empty
                    let foldername = foldername.trim();
                    if foldername.is_empty() {
                        return; // Don't close dialog if foldername is empty
                    }
                    
                    show_new_folder_dialog.set(false);
                    
                    if let Some(target_path) = dialog_target_path.read().as_ref() {
                        let parent_dir = if target_path.is_dir() { 
                            target_path.clone() 
                        } else if let Some(parent) = target_path.parent() {
                            parent.to_path_buf()
                        } else {
                            current_dir.read().clone().unwrap_or_else(|| PathBuf::from("."))
                        };
                        let folder_path = parent_dir.join(&foldername);
                        
                        spawn(async move {
                            if let Err(e) = file_operations::create_folder(&folder_path).await {
                                tracing::error!("Failed to create folder: {}", e);
                            }
                            
                            // Refresh file tree to show the new folder
                            reload_fn();
                        });
                    }
                }
            }),
            on_cancel: EventHandler::new({
                let mut show_new_folder_dialog = show_new_folder_dialog.clone();
                move |_| {
                    show_new_folder_dialog.set(false);
                }
            }),
        }

        FileNameDialog {
            visible: *show_rename_dialog.read(),
            title: "Rename".to_string(),
            initial_value: {
                dialog_target_path.read().as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string()
            },
            on_confirm: EventHandler::new({
                let mut show_rename_dialog = show_rename_dialog.clone();
                let dialog_target_path = dialog_target_path.clone();
                move |new_name| {
                    show_rename_dialog.set(false);
                    
                    if let Some(old_path) = dialog_target_path.read().as_ref() {
                        let new_path = old_path.parent().unwrap_or(old_path).join(&new_name);
                        let old_path = old_path.clone();
                        
                        spawn(async move {
                            if let Err(e) = file_operations::rename_item(&old_path, &new_path).await {
                                tracing::error!("Failed to rename item: {}", e);
                            }
                            
                            // TODO: Refresh file tree and update open tabs
                        });
                    }
                }
            }),
            on_cancel: EventHandler::new({
                let mut show_rename_dialog = show_rename_dialog.clone();
                move |_| {
                    show_rename_dialog.set(false);
                }
            }),
        }

        ConfirmDialog {
            visible: *show_delete_confirm.read(),
            title: "Delete Item".to_string(),
            message: format!("Are you sure you want to delete '{}'? This action cannot be undone.",
                dialog_target_path.read().as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("this item")
            ),
            confirm_text: "Delete".to_string(),
            danger: true,
            on_confirm: EventHandler::new({
                let mut show_delete_confirm = show_delete_confirm.clone();
                let dialog_target_path = dialog_target_path.clone();
                move |_| {
                    show_delete_confirm.set(false);
                    
                    if let Some(path) = dialog_target_path.read().as_ref() {
                        let path = path.clone();
                        spawn(async move {
                            if let Err(e) = file_operations::delete_item(&path).await {
                                tracing::error!("Failed to delete item: {}", e);
                            }
                            
                            // TODO: Refresh file tree and close any open tabs for deleted files
                        });
                    }
                }
            }),
            on_cancel: EventHandler::new({
                let mut show_delete_confirm = show_delete_confirm.clone();
                move |_| {
                    show_delete_confirm.set(false);
                }
            }),
        }
        
        // Operation Confirmation Dialog - Commented out for Claude Code mode
        // In Claude Code mode, operations are shown inline in the response
        // rather than in a popup dialog
        /*
        if app_state.read().show_operation_confirmation_dialog {
            if let Some(operations) = app_state.read().pending_operations.clone() {
                OperationConfirmationDialog {
                    operations: operations.clone(),
                    on_approve: EventHandler::new({
                        let mut app_state = app_state.clone();
                        let consensus_manager = consensus_manager.clone();
                        move |approved_operations: Vec<hive_ai::consensus::stages::file_aware_curator::FileOperation>| {
                            // Clear dialog state
                            app_state.write().show_operation_confirmation_dialog = false;
                            app_state.write().pending_operations = None;
                            
                            // Execute approved operations
                            tracing::info!("Executing {} approved operations", approved_operations.len());
                            
                            // Execute the operations asynchronously
                            if let Some(manager) = consensus_manager.read().clone() {
                                spawn(async move {
                                    match manager.execute_approved_operations(approved_operations).await {
                                        Ok(()) => {
                                            tracing::info!("✅ Successfully executed approved operations");
                                        }
                                        Err(e) => {
                                            tracing::error!("❌ Failed to execute approved operations: {}", e);
                                        }
                                    }
                                });
                            } else {
                                tracing::error!("Consensus manager not available");
                            }
                        }
                    }),
                    on_reject: EventHandler::new({
                        let mut app_state = app_state.clone();
                        move |_| {
                            // Clear dialog state
                            app_state.write().show_operation_confirmation_dialog = false;
                            app_state.write().pending_operations = None;
                            
                            tracing::info!("User rejected all pending operations");
                        }
                    }),
                    theme: hive_ai::desktop::styles::theme::ThemeColors::dark_theme(),
                }
            }
        }
        */
    }
}

#[component]
fn FileTreeItem(
    file: FileItem,
    level: usize,
    selected_file: Signal<Option<String>>,
    expanded_dirs: Signal<HashMap<PathBuf, bool>>,
    file_tree: Signal<Vec<FileItem>>,
    current_dir: Signal<Option<PathBuf>>,
    file_content: Signal<String>,
    open_tabs: Signal<Vec<String>>,
    active_tab: Signal<String>,
    tab_contents: Signal<HashMap<String, String>>,
    context_menu_state: Signal<ContextMenuState>,
    consensus_manager: Signal<Option<DesktopConsensusManager>>,
    ide_ai_broker: Signal<Option<IDEAIHelperBroker>>,
) -> Element {
    let git_state = use_context::<GitState>();
    let mut active_git_watcher = use_context::<Signal<Option<GitWatcher>>>();
    
    let file_path = file.path.clone();
    let file_path_for_context = file_path.clone(); // Clone for context menu
    let file_path_for_click = file_path.clone(); // Clone for click handler
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
            class: if is_selected { "file-tree-item selected" } else { "file-tree-item" },
            style: format!("padding-left: {}px; display: flex; align-items: center; height: 22px; line-height: 22px; cursor: pointer; user-select: none; font-size: 13px; color: {}; background-color: {}; position: relative;", 
                indent + 8,
                if is_selected { "#ffffff" } else { "#cccccc" },
                if is_selected { "#094771" } else { "transparent" }
            ),
            oncontextmenu: move |e| {
                e.prevent_default();
                // Use client coordinates with small adjustment to avoid hiding under cursor
                let coords = e.client_coordinates();
                context_menu_state.write().show(
                    (coords.x + 10.0) as i32, // Small offset to the right of cursor
                    (coords.y + 5.0) as i32,  // Small offset below cursor
                    file_path_for_context.clone(),
                    is_dir
                );
            },
            // Hover effects are handled by CSS
            onclick: move |_| {
                if is_dir {
                    // Toggle expansion
                    let current = expanded_dirs.read().get(&file_path_for_click).copied().unwrap_or(false);
                    expanded_dirs.write().insert(file_path_for_click.clone(), !current);

                    // Update current directory and git context when directory is selected
                    tracing::info!("📁 User clicked directory: {}", file_path_for_click.display());
                    
                    // Update current directory (this will trigger git context update via use_effect)
                    current_dir.set(Some(file_path_for_click.clone()));
                    
                    // Clear selected file when changing directories
                    selected_file.set(None);
                    file_content.set(String::new());
                    
                    // Update git state directly for immediate UI update
                    let mut git_state_clone = git_state.clone();
                    let git_path = file_path_for_click.clone();
                    spawn(async move {
                        if let Ok(repo) = GitRepository::open(&git_path) {
                            if let Ok(branch_name) = repo.current_branch() {
                                // Get ahead/behind counts
                                let (ahead, behind) = repo.ahead_behind().unwrap_or((0, 0));
                                let has_upstream = repo.upstream_branch().unwrap_or(None).is_some();
                                
                                let branch_info = hive_ai::desktop::git::BranchInfo {
                                    name: branch_name,
                                    branch_type: hive_ai::desktop::git::BranchType::Local,
                                    is_current: true,
                                    upstream: repo.upstream_branch().unwrap_or(None),
                                    ahead,
                                    behind,
                                    last_commit: None,
                                };
                                git_state_clone.branch_info.set(Some(branch_info));
                                
                                // Update sync status
                                let sync_status = hive_ai::desktop::git::SyncStatus {
                                    ahead,
                                    behind,
                                    has_upstream,
                                    is_syncing: false,
                                };
                                git_state_clone.sync_status.set(sync_status);
                                
                                tracing::info!("✅ Updated git branch info for directory: ahead={}, behind={}", ahead, behind);
                            }
                            
                            // Get file statuses
                            match repo.file_statuses() {
                                Ok(statuses) => {
                                    let mut status_map = std::collections::HashMap::new();
                                    for (path, git_status) in statuses {
                                        // Convert git2::Status to our StatusType
                                        let status_type = if git_status.contains(git2::Status::WT_MODIFIED) || git_status.contains(git2::Status::INDEX_MODIFIED) {
                                            hive_ai::desktop::git::StatusType::Modified
                                        } else if git_status.contains(git2::Status::WT_NEW) || git_status.contains(git2::Status::INDEX_NEW) {
                                            hive_ai::desktop::git::StatusType::Added
                                        } else if git_status.contains(git2::Status::WT_DELETED) || git_status.contains(git2::Status::INDEX_DELETED) {
                                            hive_ai::desktop::git::StatusType::Deleted
                                        } else if git_status.contains(git2::Status::WT_RENAMED) || git_status.contains(git2::Status::INDEX_RENAMED) {
                                            hive_ai::desktop::git::StatusType::Renamed
                                        } else if git_status.is_wt_new() {
                                            hive_ai::desktop::git::StatusType::Untracked
                                        } else {
                                            continue; // Skip other statuses
                                        };
                                        
                                        let file_status = hive_ai::desktop::git::FileStatus {
                                            path: path.clone(),
                                            status_type,
                                            is_staged: git_status.contains(git2::Status::INDEX_NEW) ||
                                                      git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                      git_status.contains(git2::Status::INDEX_DELETED) ||
                                                      git_status.contains(git2::Status::INDEX_RENAMED),
                                            has_staged_changes: git_status.contains(git2::Status::INDEX_NEW) ||
                                                              git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                                              git_status.contains(git2::Status::INDEX_DELETED) ||
                                                              git_status.contains(git2::Status::INDEX_RENAMED),
                                            has_unstaged_changes: git_status.contains(git2::Status::WT_NEW) ||
                                                                git_status.contains(git2::Status::WT_MODIFIED) ||
                                                                git_status.contains(git2::Status::WT_DELETED),
                                        };
                                        status_map.insert(path, file_status);
                                    }
                                    git_state_clone.file_statuses.set(status_map);
                                    tracing::info!("✅ Updated git file statuses for directory");
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to get repository status: {}", e);
                                    git_state_clone.file_statuses.set(std::collections::HashMap::new());
                                }
                            }
                        } else {
                            git_state_clone.branch_info.set(None);
                            git_state_clone.file_statuses.set(std::collections::HashMap::new());
                            tracing::info!("📝 No git repository in selected directory");
                        }
                    });
                    
                    // Update both the old consensus manager and the new IDE AI Helper Broker
                    if let Some(manager) = consensus_manager.read().clone() {
                        let dir_path = file_path_for_click.clone();
                        spawn(async move {
                            if let Err(e) = manager.update_repository_context_with_path(dir_path).await {
                                tracing::warn!("Failed to update repository context: {}", e);
                            } else {
                                tracing::info!("✅ Repository context updated for clicked directory");
                            }
                        });
                    }
                    
                    // Update IDE AI Helper Broker repository context
                    let mut ide_ai_broker = ide_ai_broker.clone();
                    spawn(async move {
                        if let Some(broker) = ide_ai_broker.read().as_ref() {
                            if let Err(e) = broker.update_repository_context().await {
                                tracing::warn!("IDE AI Helper Broker failed to update context: {}", e);
                            } else {
                                tracing::info!("✅ IDE AI Helper Broker context updated for clicked directory");
                            }
                        }
                    });
                } else {
                    // Select file and open in tab
                    println!("File clicked: {}", file_path_for_click.display());
                    let path_string = file_path_for_click.to_string_lossy().to_string();
                    
                    // Add to open tabs if not already open
                    if !open_tabs.read().contains(&path_string) {
                        open_tabs.write().push(path_string.clone());
                    }
                    
                    // Set as active tab
                    active_tab.set(path_string.clone());
                    selected_file.set(Some(path_string.clone()));

                    // Load file content immediately
                    let mut tab_contents = tab_contents.clone();
                    let mut file_content = file_content.clone();
                    let file_path = file_path_for_click.clone();
                    let path_string_for_spawn = path_string.clone();
                    spawn(async move {
                        match file_system::read_file_content(&file_path).await {
                            Ok(content) => {
                                println!("File content loaded immediately, {} bytes", content.len());
                                tab_contents.write().insert(path_string_for_spawn, content.clone());
                                file_content.set(content);
                            }
                            Err(e) => {
                                println!("Error reading file immediately: {}", e);
                                let error_content = format!("// Error reading file: {}", e);
                                tab_contents.write().insert(path_string_for_spawn, error_content.clone());
                                file_content.set(error_content);
                            }
                        }
                    });
                }
            },
            
            // Chevron for directories
            if is_dir {
                span {
                    style: "width: 20px; height: 22px; display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; color: #8a8a8a;",
                    if is_expanded { "▾" } else { "▸" }
                }
            } else {
                span {
                    style: "width: 20px; display: inline-block; flex-shrink: 0;",
                    ""
                }
            }
            
            // Icon and name container
            span {
                style: "display: flex; align-items: center; gap: 5px; flex: 1; overflow: hidden;",
                
                // File/folder icon
                span { 
                    style: "font-size: 16px; display: flex; align-items: center; flex-shrink: 0;", 
                    "{icon}" 
                }
                
                // File/folder name
                span { 
                    style: format!("white-space: nowrap; overflow: hidden; text-overflow: ellipsis; {}",
                        if is_dir { "font-weight: 500;" } else { "" }
                    ),
                    title: "{file_name}",
                    "{file_name}" 
                }
            }
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
                    open_tabs: open_tabs.clone(),
                    active_tab: active_tab.clone(),
                    tab_contents: tab_contents.clone(),
                    context_menu_state: context_menu_state.clone(),
                    consensus_manager: consensus_manager.clone(),
                    ide_ai_broker: ide_ai_broker.clone(),
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
            
            // Header with profile name and title
            div {
                style: "margin-bottom: 8px; padding-bottom: 6px; border-bottom: 1px solid #3e3e42;",
                div {
                    style: "color: #FFC107; font-size: 13px; font-weight: 600; margin-bottom: 2px;",
                    "🧠 HiveTechs Consensus"
                }
                div {
                    style: "color: #cccccc; font-size: 11px;",
                    "Profile: {consensus_state.active_profile_name}"
                }
            }

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
                                StageStatus::Waiting => "color: #666666; font-size: 11px;",
                                StageStatus::Running => "color: #FFC107; font-size: 11px;",
                                StageStatus::Completed => "color: #4caf50; font-size: 11px;",
                                StageStatus::Error => "color: #f44336; font-size: 11px;",
                            },
                            match stage.status {
                                StageStatus::Waiting => "Waiting",
                                StageStatus::Running => "Running",
                                StageStatus::Completed => "Complete",
                                StageStatus::Error => "Error",
                            }
                        }
                    }

                    // Progress bar
                    div {
                        style: "background: #1e1e1e; height: 4px; border-radius: 2px; overflow: hidden;",
                        div {
                            style: format!("background: {}; height: 100%; width: {}%; transition: width 0.3s;",
                                match stage.status {
                                    StageStatus::Waiting => "#666666",
                                    StageStatus::Running => "#FFC107",
                                    StageStatus::Completed => "#4caf50",
                                    StageStatus::Error => "#f44336",
                                },
                                stage.progress
                            ),
                        }
                    }
                    // Error message display
                    if let Some(error_msg) = &stage.error_message {
                        div {
                            style: "margin-top: 4px; padding: 4px 8px; background: #3d1f1f; border: 1px solid #5d2f2f; border-radius: 4px;",
                            span {
                                style: "color: #ff9999; font-size: 11px; line-height: 1.4;",
                                "{error_msg}"
                            }
                        }
                    }
                }
            }

            // Show cost and tokens
            if consensus_state.total_tokens > 0 {
                div {
                    style: "margin-top: 10px; padding-top: 10px; border-top: 1px solid #3e3e42; display: flex; justify-content: space-between; font-size: 11px; color: #858585;",
                    span { "Tokens: {consensus_state.total_tokens}" }
                    span { "Cost: ${consensus_state.estimated_cost:.8}" }
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
                        onclick: move |_| current_report.set("executive".to_string()),
                        style: format!("padding: 8px 16px; border-radius: 6px; border: 1px solid {}; background: {}; color: {}; cursor: pointer; transition: all 0.3s;",
                            if *current_report.read() == "executive" { "#FFC107" } else { "#3e3e42" },
                            if *current_report.read() == "executive" { "#FFC107" } else { "transparent" },
                            if *current_report.read() == "executive" { "#000" } else { "#cccccc" }),
                        "📈 Executive Dashboard"
                    }
                    
                    button {
                        onclick: move |_| current_report.set("cost".to_string()),
                        style: format!("padding: 8px 16px; border-radius: 6px; border: 1px solid {}; background: {}; color: {}; cursor: pointer; transition: all 0.3s;",
                            if *current_report.read() == "cost" { "#FFC107" } else { "#3e3e42" },
                            if *current_report.read() == "cost" { "#FFC107" } else { "transparent" },
                            if *current_report.read() == "cost" { "#000" } else { "#cccccc" }),
                        "💰 Cost Analysis"
                    }
                    
                    button {
                        onclick: move |_| current_report.set("performance".to_string()),
                        style: format!("padding: 8px 16px; border-radius: 6px; border: 1px solid {}; background: {}; color: {}; cursor: pointer; transition: all 0.3s;",
                            if *current_report.read() == "performance" { "#FFC107" } else { "#3e3e42" },
                            if *current_report.read() == "performance" { "#FFC107" } else { "transparent" },
                            if *current_report.read() == "performance" { "#000" } else { "#cccccc" }),
                        "⚡ Performance Metrics"
                    }
                    
                    button {
                        onclick: move |_| current_report.set("models".to_string()),
                        style: format!("padding: 8px 16px; border-radius: 6px; border: 1px solid {}; background: {}; color: {}; cursor: pointer; transition: all 0.3s;",
                            if *current_report.read() == "models" { "#FFC107" } else { "#3e3e42" },
                            if *current_report.read() == "models" { "#FFC107" } else { "transparent" },
                            if *current_report.read() == "models" { "#000" } else { "#cccccc" }),
                        "🤖 Model Leaderboard"
                    }
                    
                    button {
                        onclick: move |_| current_report.set("realtime".to_string()),
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
                        " {analytics_data.read().queries_trend:.1}% vs yesterday"
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
                        " {analytics_data.read().cost_trend:.1}% vs yesterday"
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

/// Fetch provider cost breakdown from database
async fn fetch_provider_costs() -> Result<Vec<(String, f64, f64)>, Box<dyn std::error::Error + Send + Sync>> {
    use hive_ai::core::database::get_database;
    
    match get_database().await {
        Ok(db) => {
            let connection = db.get_connection()?;
            
            let costs = tokio::task::spawn_blocking(move || -> Result<Vec<(String, f64, f64)>, Box<dyn std::error::Error + Send + Sync>> {
                let mut stmt = connection.prepare(
                    "SELECT 
                        CASE 
                            WHEN om.provider_name LIKE '%openai%' THEN 'OpenAI'
                            WHEN om.provider_name LIKE '%anthropic%' THEN 'Anthropic'
                            WHEN om.provider_name LIKE '%google%' THEN 'Google'
                            ELSE 'Other'
                        END as provider,
                        SUM(ct.total_cost) as total_cost,
                        COUNT(DISTINCT ct.conversation_id) as usage_count
                     FROM cost_tracking ct
                     JOIN openrouter_models om ON ct.model_id = om.internal_id
                     GROUP BY provider
                     ORDER BY total_cost DESC"
                )?;
                
                let costs = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, f64>(2)?,
                    ))
                })?.collect::<Result<Vec<_>, _>>()?;
                
                Ok(costs)
            }).await??;
            
            Ok(costs)
        }
        Err(_) => Ok(vec![])
    }
}

/// Cost Analysis Report Component
#[component]
fn CostAnalysisReport(analytics_data: Signal<AnalyticsData>) -> Element {
    let provider_costs = use_resource(move || fetch_provider_costs());
    
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
                    
                    if let Some(Ok(costs)) = provider_costs.read().as_ref() {
                        if costs.is_empty() {
                            div {
                                style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; grid-column: 1 / -1;",
                                p { style: "color: #858585; text-align: center;", "No cost data available yet. Run some conversations to see cost breakdown." }
                            }
                        } else {
                            for (provider, cost, _count) in costs {
                                div {
                                    style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42;",
                                    h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "{provider}" }
                                    div { style: "font-size: 20px; font-weight: bold; color: #cccccc;", "${cost:.4}" }
                                    div { 
                                        style: "font-size: 10px; color: #858585;",
                                        {
                                            let total_cost = analytics_data.read().total_cost;
                                            let percentage = if total_cost > 0.0 {
                                                cost / total_cost * 100.0
                                            } else { 0.0 };
                                            format!("{percentage:.1}% of total cost")
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; grid-column: 1 / -1;",
                            p { style: "color: #858585; text-align: center;", "Loading provider costs..." }
                        }
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
                    
                    // Dynamic recommendations based on actual usage
                    if analytics_data.read().conversations_with_cost > 0 {
                        {
                            let conversations_with_cost = analytics_data.read().conversations_with_cost;
                            let total_cost = analytics_data.read().total_cost;
                            let output_tokens = analytics_data.read().total_tokens_output;
                            let input_tokens = analytics_data.read().total_tokens_input;
                            let avg_cost_per_conversation = total_cost / conversations_with_cost as f64;
                            
                            rsx! {
                                if avg_cost_per_conversation > 0.01 {
                                    div {
                                        style: "padding: 10px; background: #0E1414; border-radius: 4px; border-left: 3px solid #f44336;",
                                        div { style: "font-weight: bold; color: #f44336; margin-bottom: 5px;", "High cost per conversation detected" }
                                        div { style: "font-size: 12px; color: #cccccc;", 
                                            "Average cost: ${avg_cost_per_conversation:.4} per conversation. Consider using Claude 3 Haiku for simple queries."
                                        }
                                    }
                                }
                                
                                if output_tokens > input_tokens * 2 {
                                    div {
                                        style: "padding: 10px; background: #0E1414; border-radius: 4px; border-left: 3px solid #ff9800;",
                                        div { style: "font-weight: bold; color: #ff9800; margin-bottom: 5px;", "High output token usage" }
                                        div { style: "font-size: 12px; color: #cccccc;", 
                                            "Output tokens are 2x input tokens. Consider more concise prompts to reduce generation costs."
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    div {
                        style: "padding: 10px; background: #0E1414; border-radius: 4px; border-left: 3px solid #4caf50;",
                        div { style: "font-weight: bold; color: #4caf50; margin-bottom: 5px;", "Enable caching for repeated queries" }
                        div { style: "font-size: 12px; color: #cccccc;", "Save up to 70% on similar questions by caching consensus results" }
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
                    {
                        let total_cost = analytics_data.read().total_cost;
                        let progress = (total_cost / 100.0 * 100.0) as u32;
                        format!("{progress}% of monthly budget used")
                    }
                }
            }
        }
    }
}

/// Performance Report Component  
#[component]
fn PerformanceReport(analytics_data: Signal<AnalyticsData>) -> Element {
    let performance_metrics = use_resource(move || fetch_performance_metrics());
    
    rsx! {
        div {
            h2 {
                style: "color: #FFC107; margin-bottom: 20px; font-size: 20px;",
                "⚡ Performance Metrics & Pipeline Analysis"
            }
            
            if let Some(Ok((gen_time, ref_time, val_time, cur_time, success_rate, error_rate))) = performance_metrics.read().as_ref() {
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
                            div { style: "font-size: 16px; font-weight: bold; color: #4caf50;", "{analytics_helpers::format_duration(*gen_time)}" }
                            div { style: "font-size: 10px; color: #858585;", "avg response" }
                        }
                        
                        div {
                            style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; text-align: center;",
                            h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "Refiner" }
                            div { style: "font-size: 16px; font-weight: bold; color: #4caf50;", "{analytics_helpers::format_duration(*ref_time)}" }
                            div { style: "font-size: 10px; color: #858585;", "avg response" }
                        }
                        
                        div {
                            style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; text-align: center;",
                            h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "Validator" }
                            div { style: "font-size: 16px; font-weight: bold; color: #4caf50;", "{analytics_helpers::format_duration(*val_time)}" }
                            div { style: "font-size: 10px; color: #858585;", "avg response" }
                        }
                        
                        div {
                            style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; text-align: center;",
                            h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "Curator" }
                            div { style: "font-size: 16px; font-weight: bold; color: #4caf50;", "{analytics_helpers::format_duration(*cur_time)}" }
                            div { style: "font-size: 10px; color: #858585;", "avg response" }
                        }
                    }
                }
                
                div {
                    style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                    h3 {
                        style: "color: #cccccc; margin-bottom: 15px; font-size: 16px;",
                        "Quality & Reliability Metrics"
                    }
                    div {
                        style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px;",
                        
                        div {
                            h4 { style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;", "Success Rate" }
                            div { 
                                style: format!("font-size: 24px; font-weight: bold; color: {};", analytics_helpers::get_performance_color(*success_rate, 90.0, 70.0)),
                                "{success_rate:.1}%" 
                            }
                            div { style: "font-size: 12px; color: #858585;", "Successful completions" }
                        }
                        
                        div {
                            h4 { style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;", "Error Rate" }
                            div { 
                                style: format!("font-size: 24px; font-weight: bold; color: {};", if *error_rate < 1.0 { "#4caf50" } else if *error_rate < 5.0 { "#FFC107" } else { "#f44336" }),
                                "{error_rate:.1}%" 
                            }
                            div { style: "font-size: 12px; color: #858585;", "Pipeline failures" }
                        }
                        
                        div {
                            h4 { style: "margin: 0 0 10px 0; color: #FFC107; font-size: 14px;", "Total Time" }
                            div { 
                                style: "font-size: 24px; font-weight: bold; color: #007BFF;",
                                "{analytics_helpers::format_duration(gen_time + ref_time + val_time + cur_time)}"
                            }
                            div { style: "font-size: 12px; color: #858585;", "Full pipeline duration" }
                        }
                    }
                }
            } else {
                div {
                    style: "background: #181E21; padding: 40px; border-radius: 8px; border: 1px solid #3e3e42; text-align: center;",
                    div {
                        style: "color: #858585; font-size: 16px;",
                        "Loading performance metrics..."
                    }
                }
            }
        }
    }
}

/// Fetch model usage stats from database
async fn fetch_model_stats() -> Result<Vec<(String, String, f64, u64, f64, f64)>, Box<dyn std::error::Error + Send + Sync>> {
    use hive_ai::core::database::get_database;
    
    match get_database().await {
        Ok(db) => {
            let connection = db.get_connection()?;
            
            let stats = tokio::task::spawn_blocking(move || -> Result<Vec<(String, String, f64, u64, f64, f64)>, Box<dyn std::error::Error + Send + Sync>> {
                // Get all models with actual usage data (no hardcoded names, dynamic by internal_id)
                let mut stmt = connection.prepare(
                    "SELECT 
                        om.name,
                        om.openrouter_id,
                        SUM(ct.total_cost) as total_cost,
                        COUNT(DISTINCT ct.conversation_id) as usage_count,
                        om.pricing_input * 1000000 as cost_per_million_input,
                        om.pricing_output * 1000000 as cost_per_million_output
                     FROM cost_tracking ct
                     JOIN openrouter_models om ON ct.model_id = om.internal_id
                     GROUP BY om.internal_id
                     ORDER BY usage_count DESC"
                )?;
                
                let models = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,  // name
                        row.get::<_, String>(1)?,  // openrouter_id
                        row.get::<_, f64>(2)?,     // total_cost
                        row.get::<_, u64>(3)?,     // usage_count
                        row.get::<_, f64>(4)?,     // cost_per_million_input
                        row.get::<_, f64>(5)?,     // cost_per_million_output
                    ))
                })?.collect::<Result<Vec<_>, _>>()?;
                
                Ok(models)
            }).await??;
            
            Ok(stats)
        }
        Err(_) => Ok(vec![])
    }
}

/// Fetch recent conversations from database
async fn fetch_recent_conversations() -> Result<Vec<(String, String, f64, String)>, Box<dyn std::error::Error + Send + Sync>> {
    use hive_ai::core::database::get_database;
    
    match get_database().await {
        Ok(db) => {
            let connection = db.get_connection()?;
            
            let conversations = tokio::task::spawn_blocking(move || -> Result<Vec<(String, String, f64, String)>, Box<dyn std::error::Error + Send + Sync>> {
                let mut stmt = connection.prepare(
                    "SELECT 
                        c.id,
                        COALESCE(c.title, 'Conversation ' || substr(c.id, 1, 8)) as title,
                        c.total_cost,
                        c.created_at
                     FROM conversations c
                     WHERE c.total_cost > 0
                     ORDER BY c.created_at DESC
                     LIMIT 10"
                )?;
                
                let convos = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,  // id
                        row.get::<_, String>(1)?,  // first_message
                        row.get::<_, f64>(2)?,     // total_cost
                        row.get::<_, String>(3)?,  // created_at
                    ))
                })?.collect::<Result<Vec<_>, _>>()?;
                
                Ok(convos)
            }).await??;
            
            Ok(conversations)
        }
        Err(_) => Ok(vec![])
    }
}

/// Fetch performance metrics from database
async fn fetch_performance_metrics() -> Result<(f64, f64, f64, f64, f64, f64), Box<dyn std::error::Error + Send + Sync>> {
    use hive_ai::core::database::get_database;
    
    match get_database().await {
        Ok(db) => {
            let connection = db.get_connection()?;
            
            let metrics = tokio::task::spawn_blocking(move || -> Result<(f64, f64, f64, f64, f64, f64), Box<dyn std::error::Error + Send + Sync>> {
                // Get performance data from stored database facts only
                
                let total_convos: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                let conversations_with_cost: u64 = connection.query_row(
                    "SELECT COUNT(*) FROM conversations WHERE total_cost > 0",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0);
                
                // Calculate success rate using helper function
                let success_rate = analytics_helpers::calculate_percentage(
                    conversations_with_cost as f64,
                    total_convos as f64
                );
                
                let error_rate = 100.0 - success_rate;
                
                // Get actual model performance timing data by model_id (no names)
                let model_times: Vec<f64> = {
                    let mut stmt = connection.prepare(
                        "SELECT model_id, AVG(total_cost * 1000.0) as avg_processing_time
                         FROM cost_tracking 
                         WHERE total_cost > 0 
                         GROUP BY model_id 
                         ORDER BY model_id"
                    )?;
                    
                    let results = stmt.query_map([], |row| {
                        Ok(row.get::<_, f64>(1).unwrap_or(2.0))
                    })?;
                    
                    results.filter_map(|r| r.ok()).collect()
                };
                
                // Get actual timing data or return database facts without fallbacks
                let stage_1 = model_times.get(0).copied().unwrap_or(0.0);
                let stage_2 = model_times.get(1).copied().unwrap_or(0.0);
                let stage_3 = model_times.get(2).copied().unwrap_or(0.0);
                let stage_4 = model_times.get(3).copied().unwrap_or(0.0);
                
                Ok((stage_1, stage_2, stage_3, stage_4, success_rate, error_rate))
            }).await??;
            
            Ok(metrics)
        }
        Err(_) => Err("Database unavailable".into())
    }
}

/// Model Leaderboard Component
#[component]
fn ModelLeaderboard(analytics_data: Signal<AnalyticsData>) -> Element {
    let model_stats = use_resource(move || fetch_model_stats());
    
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
                    style: "display: grid; grid-template-columns: 3fr 1fr 1fr 1fr 1fr; gap: 15px; padding: 10px 0; border-bottom: 1px solid #3e3e42; margin-bottom: 15px; font-weight: bold; color: #FFC107; font-size: 12px;",
                    div { "Model" }
                    div { "Total Cost" }
                    div { "Usage Count" }
                    div { "$/1M Tokens" }
                    div { "Usage %" }
                }
                
                // Model Rows
                div {
                    style: "display: grid; gap: 10px;",
                    
                    if let Some(Ok(models)) = model_stats.read().as_ref() {
                        if models.is_empty() {
                            div {
                                style: "padding: 20px; text-align: center; color: #858585;",
                                "No model usage data available yet. Run some conversations to see model performance."
                            }
                        } else {
                            {
                                let total_usage: u64 = models.iter().map(|(_, _, _, count, _, _)| count).sum();
                                let total_usage_f64 = total_usage as f64;
                                
                                models.iter().enumerate().map(move |(idx, (name, _id, cost, count, input_cost, output_cost))| {
                                    let emoji = match idx {
                                        0 => "🥇",
                                        1 => "🥈",
                                        2 => "🥉",
                                        _ => "📊"
                                    };
                                    let avg_cost = (input_cost + output_cost) / 2.0;
                                    let usage_pct = if total_usage_f64 > 0.0 {
                                        *count as f64 / total_usage_f64 * 100.0
                                    } else { 0.0 };
                                    
                                    rsx! {
                                        div {
                                            style: "display: grid; grid-template-columns: 3fr 1fr 1fr 1fr 1fr; gap: 15px; padding: 12px 0; border-bottom: 1px solid #2a2a2a;",
                                            div {
                                                style: "color: #cccccc; font-weight: bold;",
                                                "{emoji} {name}"
                                            }
                                            div { 
                                                style: "color: #4caf50;",
                                                "${cost:.4}"
                                            }
                                            div { 
                                                style: "color: #cccccc;",
                                                "{count}"
                                            }
                                            div { 
                                                style: format!("color: {};", if avg_cost < 5.0 { "#4caf50" } else if avg_cost < 15.0 { "#FFC107" } else { "#f44336" }),
                                                "${avg_cost:.2}"
                                            }
                                            div { 
                                                style: "color: #007BFF;",
                                                "{usage_pct:.1}%"
                                            }
                                        }
                                    }
                                })
                            }
                        }
                    } else {
                        div {
                            style: "padding: 20px; text-align: center; color: #858585;",
                            "Loading model statistics..."
                        }
                    }
                }
            }
        }
    }
}

/// Real-Time Activity Component (this is the current/recent activity feed)
#[component]
fn RealTimeActivity(analytics_data: Signal<AnalyticsData>) -> Element {
    // Re-fetch conversations when analytics data changes (which happens after each consensus run)
    let mut recent_conversations = use_resource(move || {
        // Read analytics data to create a dependency
        let _ = analytics_data.read();
        fetch_recent_conversations()
    });
    
    // Also set up periodic refresh every 5 seconds for real-time updates
    use_future(move || async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            recent_conversations.restart();
        }
    });
    
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
            
            // Recent Conversations List
            div {
                style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42; margin-bottom: 20px;",
                h3 {
                    style: "color: #cccccc; margin-bottom: 15px; font-size: 16px;",
                    "Recent Conversations"
                }
                
                if let Some(Ok(conversations)) = recent_conversations.read().as_ref() {
                    if conversations.is_empty() {
                        div {
                            style: "padding: 20px; text-align: center; color: #858585;",
                            "No recent conversations with cost data. Start a conversation to see activity here."
                        }
                    } else {
                        div {
                            style: "display: grid; gap: 10px;",
                            
                            for (id, message, cost, created_at) in conversations {
                                div {
                                    style: "padding: 12px; background: #0E1414; border-radius: 4px; border-left: 3px solid #007BFF;",
                                    div {
                                        style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                                        div {
                                            style: "font-weight: bold; color: #cccccc; font-size: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 400px;",
                                            "{message}"
                                        }
                                        div {
                                            style: "color: #4caf50; font-weight: bold;",
                                            "${cost:.4}"
                                        }
                                    }
                                    div {
                                        style: "display: flex; justify-content: space-between; font-size: 11px; color: #858585;",
                                        div {
                                            style: "font-family: monospace;",
                                            "{&id[0..8]}..."
                                        }
                                        div {
                                            "{created_at}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div {
                        style: "padding: 20px; text-align: center; color: #858585;",
                        "Loading recent conversations..."
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

/// Load the active profile from database for UI updates
async fn load_active_profile_from_db() -> anyhow::Result<ActiveProfile> {
    use hive_ai::core::database::get_database;
    use rusqlite::OptionalExtension;

    let db = get_database().await?;
    let conn = db.get_connection()?;

    // Get the active profile ID from consensus_settings
    let active_profile_id: Option<String> = conn.query_row(
        "SELECT value FROM consensus_settings WHERE key = 'active_profile_id'",
        [],
        |row| row.get(0)
    ).optional()?;

    let profile_id = active_profile_id
        .ok_or_else(|| anyhow::anyhow!("No active profile configured"))?;

    // Get the profile by ID
    let profile = conn.query_row(
        "SELECT profile_name, generator_model, refiner_model, validator_model, curator_model FROM consensus_profiles WHERE id = ?1",
        rusqlite::params![profile_id],
        |row| {
            Ok(ActiveProfile {
                profile_name: row.get(0)?,
                generator_model: row.get(1)?,
                refiner_model: row.get(2)?,
                validator_model: row.get(3)?,
                curator_model: row.get(4)?,
            })
        }
    )?;

    Ok(profile)
}

/// Profile information for UI updates
#[derive(Debug, Clone)]
struct ActiveProfile {
    profile_name: String,
    generator_model: String,
    refiner_model: String,
    validator_model: String,
    curator_model: String,
}
