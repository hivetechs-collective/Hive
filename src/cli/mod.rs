//! CLI interface for Hive AI
//!
//! This module provides the command-line interface with comprehensive
//! command support, interactive mode, and TUI integration.

pub mod args;
pub mod banner;
pub mod commands;
pub mod completions;
pub mod interactive;
pub mod trust;
pub mod tui_commands;
pub mod tui_capabilities;
pub mod tui_themes;
pub mod accessibility;

pub use args::{Cli, Commands};
pub use banner::print_banner;
pub use commands::*;

use crate::core::error::Result;
use std::env;

/// Check if TUI should be launched based on terminal capabilities
pub async fn should_launch_tui() -> bool {
    use tui_capabilities::{TuiLauncher, TuiMode};
    
    match TuiLauncher::should_launch_tui().await {
        Ok(TuiMode::Enhanced) | Ok(TuiMode::Simple) => true,
        _ => false,
    }
}

/// Check TUI capabilities synchronously
pub fn check_tui_capabilities() -> bool {
    use tui_capabilities::TuiCapabilities;
    
    match TuiCapabilities::detect() {
        Ok(capabilities) => capabilities.supports_simple_tui(),
        Err(_) => false,
    }
}

/// Initialize CLI system
pub async fn init_cli() -> Result<()> {
    // Initialize CLI components
    Ok(())
}

/// Get current directory display string
pub fn get_current_dir_display() -> String {
    env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "(unknown)".to_string())
}

/// Check internet connection status
pub async fn check_internet_connection() -> bool {
    // In a real implementation, this would check actual connectivity
    // For now, simulate with a simple check
    tokio::time::timeout(
        std::time::Duration::from_secs(2),
        check_dns_resolution()
    )
    .await
    .unwrap_or(false)
}

/// Check DNS resolution as a proxy for internet connectivity
async fn check_dns_resolution() -> bool {
    // Try to resolve a well-known domain
    match tokio::net::lookup_host("1.1.1.1:443").await {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Check API status
pub async fn check_api_status() -> bool {
    // In a real implementation, this would check OpenRouter API
    // For now, return true if we have internet
    check_internet_connection().await
}

/// Get current memory usage in bytes
pub fn get_memory_usage() -> usize {
    // This is a simplified implementation
    // In production, we'd use platform-specific APIs
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<usize>() {
                            return kb * 1024; // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }
    
    // Default fallback for other platforms
    // Estimate based on a typical Rust CLI application
    25 * 1024 * 1024 // 25 MB
}