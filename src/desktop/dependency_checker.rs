//! Dependency checker for ensuring all required components are installed

use anyhow::{Result, Context};
use std::process::Command;
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub struct DependencyStatus {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub install_command: Option<String>,
}

/// Check all required dependencies for Hive
pub async fn check_dependencies() -> Vec<DependencyStatus> {
    let mut dependencies = Vec::new();
    
    // Check Claude Code
    dependencies.push(check_claude_code().await);
    
    // Check Node.js/npm (required for Claude Code)
    dependencies.push(check_npm().await);
    
    dependencies
}

/// Check if Claude Code is installed
async fn check_claude_code() -> DependencyStatus {
    // First check if Claude binary exists
    let claude_check = Command::new("claude")
        .arg("--version")
        .output();
        
    if let Ok(output) = claude_check {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return DependencyStatus {
                name: "Claude Code".to_string(),
                installed: true,
                version: Some(version),
                install_command: None,
            };
        }
    }
    
    // Check npm global installation path
    if let Ok(npm_prefix) = Command::new("npm")
        .args(&["config", "get", "prefix"])
        .output() 
    {
        if npm_prefix.status.success() {
            let prefix = String::from_utf8_lossy(&npm_prefix.stdout).trim().to_string();
            let claude_paths = vec![
                format!("{}/bin/claude", prefix),
                format!("{}/bin/claude-code", prefix),
            ];
            
            for path in claude_paths {
                if std::path::Path::new(&path).exists() {
                    return DependencyStatus {
                        name: "Claude Code".to_string(),
                        installed: true,
                        version: Some("Installed via npm".to_string()),
                        install_command: None,
                    };
                }
            }
        }
    }
    
    DependencyStatus {
        name: "Claude Code".to_string(),
        installed: false,
        version: None,
        install_command: Some("npm install -g @anthropic-ai/claude-code".to_string()),
    }
}

/// Check if npm is installed
async fn check_npm() -> DependencyStatus {
    let npm_check = Command::new("npm")
        .arg("--version")
        .output();
        
    if let Ok(output) = npm_check {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return DependencyStatus {
                name: "Node.js/npm".to_string(),
                installed: true,
                version: Some(format!("npm {}", version)),
                install_command: None,
            };
        }
    }
    
    DependencyStatus {
        name: "Node.js/npm".to_string(),
        installed: false,
        version: None,
        install_command: Some("Install from https://nodejs.org".to_string()),
    }
}

/// Install Claude Code via npm
pub async fn install_claude_code(progress_callback: impl Fn(String)) -> Result<()> {
    progress_callback("Checking npm installation...".to_string());
    
    // First ensure npm is available
    let npm_check = Command::new("npm")
        .arg("--version")
        .output()
        .context("npm not found - please install Node.js first")?;
        
    if !npm_check.status.success() {
        return Err(anyhow::anyhow!("npm is not properly installed"));
    }
    
    progress_callback("Installing Claude Code via npm...".to_string());
    
    // Install Claude Code globally
    let install_output = Command::new("npm")
        .args(&["install", "-g", "@anthropic-ai/claude-code"])
        .output()
        .context("Failed to run npm install")?;
    
    if !install_output.status.success() {
        let error = String::from_utf8_lossy(&install_output.stderr);
        return Err(anyhow::anyhow!("npm install failed: {}", error));
    }
    
    progress_callback("Verifying Claude Code installation...".to_string());
    
    // Verify installation
    let verify = Command::new("claude")
        .arg("--version")
        .output();
        
    if let Ok(output) = verify {
        if output.status.success() {
            progress_callback("✅ Claude Code installed successfully!".to_string());
            return Ok(());
        }
    }
    
    // If claude command doesn't work, check npm bin path
    if let Ok(npm_bin) = Command::new("npm")
        .args(&["bin", "-g"])
        .output() 
    {
        if npm_bin.status.success() {
            let bin_path = String::from_utf8_lossy(&npm_bin.stdout).trim().to_string();
            progress_callback(format!("Claude Code installed at: {}/claude", bin_path));
            progress_callback("You may need to add this to your PATH or restart your terminal.".to_string());
            return Ok(());
        }
    }
    
    Err(anyhow::anyhow!("Claude Code installation completed but verification failed"))
}

/// Ensure all dependencies are installed
pub async fn ensure_dependencies_installed() -> Result<()> {
    let deps = check_dependencies().await;
    
    for dep in deps {
        if !dep.installed {
            info!("Missing dependency: {}", dep.name);
            
            if dep.name == "Claude Code" {
                // Try to install Claude Code automatically
                match install_claude_code(|msg| info!("{}", msg)).await {
                    Ok(_) => info!("✅ {} installed successfully", dep.name),
                    Err(e) => {
                        error!("Failed to install {}: {}", dep.name, e);
                        return Err(anyhow::anyhow!(
                            "{} is required but could not be installed: {}", 
                            dep.name, e
                        ));
                    }
                }
            } else if let Some(cmd) = dep.install_command {
                return Err(anyhow::anyhow!(
                    "{} is required. Please install it: {}", 
                    dep.name, cmd
                ));
            }
        }
    }
    
    Ok(())
}