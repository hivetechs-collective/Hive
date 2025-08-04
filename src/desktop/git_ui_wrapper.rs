//! GitUI CLI Wrapper for Portal Architecture
//! 
//! This module provides a zero-maintenance Git integration by wrapping GitUI
//! as a CLI tool in our Portal architecture.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

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