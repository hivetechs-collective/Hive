//! GitUI CLI Wrapper for Portal Architecture
//! 
//! This module provides a zero-maintenance Git integration by wrapping GitUI
//! as a CLI tool in our Portal architecture.

use anyhow::Result;
use std::collections::HashMap;

/// GitUI wrapper that provides full Git functionality with zero code maintenance
pub struct GitUIWrapper;

impl GitUIWrapper {
    pub fn new() -> Self {
        Self
    }

    /// Get the name displayed in the UI
    pub fn name(&self) -> &str {
        "GitUI"
    }

    /// Get the command to execute
    pub fn command(&self) -> &str {
        "gitui"
    }

    /// Get the command for health check
    pub fn health_check(&self) -> Option<&str> {
        Some("--version")
    }

    /// Whether this tool requires authentication
    pub fn requires_auth(&self) -> bool {
        false
    }

    /// Whether this tool supports stdin commands
    pub fn supports_stdin_commands(&self) -> bool {
        false
    }

    /// Get the icon for display
    pub fn icon(&self) -> &str {
        "🔀"
    }

    /// Get environment variables for GitUI
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        // GitUI respects standard Git environment variables
        // We can add custom theme settings here if needed
        env.insert("GITUI_THEME".to_string(), "dark".to_string());
        env
    }

    /// Get startup arguments for GitUI
    pub fn get_args(&self) -> Vec<String> {
        // GitUI supports various command line arguments
        // For now, we'll use default behavior
        vec![]
    }
}

/// Check if GitUI is installed on the system
pub fn ensure_gitui_installed() -> Result<()> {
    if which::which("gitui").is_err() {
        #[cfg(target_os = "windows")]
        let install_cmd = "winget install gitui";
        
        #[cfg(target_os = "macos")]
        let install_cmd = "brew install gitui";
        
        #[cfg(target_os = "linux")]
        let install_cmd = "cargo install gitui";
        
        return Err(anyhow::anyhow!(
            "GitUI not found. Please install it using: {}",
            install_cmd
        ));
    }
    Ok(())
}

/// Get GitUI version information
pub async fn get_gitui_version() -> Result<String> {
    use tokio::process::Command;
    
    let output = Command::new("gitui")
        .arg("--version")
        .output()
        .await?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow::anyhow!("Failed to get GitUI version"))
    }
}

/// Spawn GitUI in a specific directory
pub fn spawn_gitui_with_dir(working_dir: &std::path::Path) -> Result<()> {
    use std::process::Command;
    
    Command::new("gitui")
        .current_dir(working_dir)
        .spawn()?;
    
    Ok(())
}