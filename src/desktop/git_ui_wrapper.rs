//! LazyGit CLI Wrapper for Portal Architecture
//! 
//! This module provides a zero-maintenance Git integration by wrapping LazyGit
//! as a CLI tool in our Portal architecture.

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::desktop::git::initialize_lazygit_updater_db;

/// LazyGit wrapper that provides full Git functionality with zero code maintenance
pub struct LazyGitWrapper;

impl LazyGitWrapper {
    pub fn new() -> Self {
        Self
    }

    /// Get the name displayed in the UI
    pub fn name(&self) -> &str {
        "LazyGit"
    }

    /// Get the command to execute
    pub fn command(&self) -> &str {
        "lazygit"
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

    /// Get environment variables for LazyGit
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        // LazyGit respects standard Git environment variables
        // Configure for better terminal UI experience
        env.insert("TERM".to_string(), "xterm-256color".to_string());
        env.insert("COLORTERM".to_string(), "truecolor".to_string());
        // LazyGit config directory
        env.insert("XDG_CONFIG_HOME".to_string(), 
                  std::env::var("HOME").unwrap_or_default() + "/.config");
        env
    }

    /// Get startup arguments for LazyGit
    pub fn get_args(&self) -> Vec<String> {
        // LazyGit can be configured with custom config paths
        // For side-pane usage, we might want to customize behavior
        vec![]
    }
}

/// Check if LazyGit is installed and get the path to it
/// This will automatically download and update LazyGit if needed
/// Uses the database to track update checks every 24 hours
pub async fn ensure_lazygit_installed() -> Result<PathBuf> {
    initialize_lazygit_updater_db().await
}

/// Legacy sync version for backward compatibility
pub fn ensure_lazygit_installed_sync() -> Result<()> {
    // For now, just check if it exists anywhere
    // The async version should be used for actual installation
    if which::which("lazygit").is_err() {
        return Err(anyhow::anyhow!(
            "LazyGit not found. It will be automatically installed on next use."
        ));
    }
    Ok(())
}

/// Get LazyGit version information
pub async fn get_lazygit_version() -> Result<String> {
    use tokio::process::Command;
    
    let output = Command::new("lazygit")
        .arg("--version")
        .output()
        .await?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow::anyhow!("Failed to get LazyGit version"))
    }
}

/// Check if a directory is a valid git repository
pub fn is_git_repository(path: &Path) -> bool {
    // Check if .git directory exists
    let git_dir = path.join(".git");
    if git_dir.exists() {
        return true;
    }
    
    // Also check parent directories in case we're in a subdirectory
    if let Some(parent) = path.parent() {
        // But only check up to 5 levels to avoid excessive checking
        let mut current = parent;
        let mut levels = 0;
        while levels < 5 {
            if current.join(".git").exists() {
                return true;
            }
            match current.parent() {
                Some(p) => current = p,
                None => break,
            }
            levels += 1;
        }
    }
    
    false
}

/// Find the git repository root from a given path
pub fn find_git_root(path: &Path) -> Option<std::path::PathBuf> {
    // Check if current directory is git root
    if path.join(".git").exists() {
        return Some(path.to_path_buf());
    }
    
    // Check parent directories
    if let Some(parent) = path.parent() {
        let mut current = parent;
        let mut levels = 0;
        while levels < 10 {  // Reasonable limit to avoid excessive checking
            if current.join(".git").exists() {
                return Some(current.to_path_buf());
            }
            match current.parent() {
                Some(p) => current = p,
                None => break,
            }
            levels += 1;
        }
    }
    
    None
}

/// Configure git safe directory if needed
pub fn ensure_safe_directory(path: &Path) -> Result<()> {
    use std::process::Command;
    
    // Try to run a simple git command to check if the directory is trusted
    let output = Command::new("git")
        .arg("status")
        .current_dir(path)
        .output()?;
    
    // If the output contains "dubious ownership" error, add it as safe
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("dubious ownership") || stderr.contains("fatal: detected dubious ownership") {
        // Add the directory as safe
        Command::new("git")
            .args(&["config", "--global", "--add", "safe.directory", path.to_str().unwrap_or("*")])
            .output()?;
        
        tracing::info!("Added {} as safe directory for git", path.display());
    }
    
    Ok(())
}

/// Spawn GitUI in a specific directory
pub fn spawn_gitui_with_dir(working_dir: &std::path::Path) -> Result<()> {
    use std::process::Command;
    
    // Ensure the directory is trusted by git
    let _ = ensure_safe_directory(working_dir);
    
    Command::new("gitui")
        .current_dir(working_dir)
        .spawn()?;
    
    Ok(())
}