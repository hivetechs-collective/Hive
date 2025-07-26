//! Claude Code Installer and Updater
//! 
//! This module handles automatic installation and updates of Claude Code CLI
//! within the Hive IDE, ensuring users always have access to the latest version.

use anyhow::{Context, Result};
use reqwest::Client;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs as async_fs;
use tracing::{debug, error, info, warn};

// Note: These URLs are placeholders - Claude Code doesn't have public direct download URLs yet
// In a real implementation, we would:
// 1. Use Claude's official API to get download URLs
// 2. Or bundle Claude Code with our installer
// 3. Or prompt user to install manually and provide the path

#[cfg(target_os = "macos")]
const CLAUDE_DOWNLOAD_URL: &str = "https://claude.ai/download/macos"; // Placeholder - redirects to download page

#[cfg(target_os = "linux")]
const CLAUDE_DOWNLOAD_URL: &str = "https://claude.ai/download/linux"; // Placeholder - redirects to download page

#[cfg(target_os = "windows")]
const CLAUDE_DOWNLOAD_URL: &str = "https://claude.ai/download/windows"; // Placeholder - redirects to download page

/// Manages Claude Code installation and updates
pub struct ClaudeInstaller {
    install_dir: PathBuf,
    binary_path: PathBuf,
}

impl ClaudeInstaller {
    /// Create a new installer instance
    pub fn new() -> Result<Self> {
        // Install in Hive's config directory
        let install_dir = crate::core::config::get_hive_config_dir()
            .join("claude-code");
        
        let binary_name = if cfg!(windows) { "claude.exe" } else { "claude" };
        let binary_path = install_dir.join("bin").join(binary_name);
        
        Ok(Self {
            install_dir,
            binary_path,
        })
    }
    
    /// Get the path to the Claude Code binary
    pub fn get_binary_path(&self) -> &Path {
        &self.binary_path
    }
    
    /// Check if Claude Code is installed
    pub async fn is_installed(&self) -> bool {
        self.binary_path.exists() && self.check_binary_works().await
    }
    
    /// Check if the binary actually works
    async fn check_binary_works(&self) -> bool {
        match Command::new(&self.binary_path)
            .arg("--version")
            .output()
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    /// Get installed version of Claude Code
    pub async fn get_installed_version(&self) -> Result<Option<String>> {
        if !self.is_installed().await {
            return Ok(None);
        }
        
        let output = Command::new(&self.binary_path)
            .arg("--version")
            .output()
            .context("Failed to get Claude Code version")?;
        
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Ok(Some(version))
        } else {
            Ok(None)
        }
    }
    
    /// Install or update Claude Code
    pub async fn install_or_update(&self) -> Result<()> {
        info!("🚀 Installing/updating Claude Code CLI...");
        
        // Create installation directory
        async_fs::create_dir_all(&self.install_dir).await
            .context("Failed to create install directory")?;
        
        // Create bin directory
        let bin_dir = self.install_dir.join("bin");
        async_fs::create_dir_all(&bin_dir).await
            .context("Failed to create bin directory")?;
        
        // Download Claude Code
        let download_path = self.download_claude().await?;
        
        // Extract and install
        self.extract_and_install(&download_path).await?;
        
        // Verify installation
        if self.is_installed().await {
            if let Ok(Some(version)) = self.get_installed_version().await {
                info!("✅ Claude Code {} installed successfully", version);
            } else {
                info!("✅ Claude Code installed successfully");
            }
        } else {
            error!("❌ Claude Code installation verification failed");
            return Err(anyhow::anyhow!("Installation verification failed"));
        }
        
        // Clean up download
        let _ = async_fs::remove_file(&download_path).await;
        
        Ok(())
    }
    
    /// Download Claude Code CLI
    /// 
    /// Since Claude Code doesn't provide direct download URLs, this will:
    /// 1. Check if user has manually placed the binary in our install directory
    /// 2. Guide user to download manually if needed
    async fn download_claude(&self) -> Result<PathBuf> {
        info!("📥 Checking for Claude Code CLI...");
        
        // Check if user has manually placed the binary
        let manual_binary = self.install_dir.join("claude");
        if manual_binary.exists() {
            info!("✅ Found manually installed Claude Code at: {}", manual_binary.display());
            // Copy to bin directory
            let bin_dir = self.install_dir.join("bin");
            async_fs::create_dir_all(&bin_dir).await?;
            let target = bin_dir.join(if cfg!(windows) { "claude.exe" } else { "claude" });
            async_fs::copy(&manual_binary, &target).await?;
            return Ok(manual_binary);
        }
        
        // For now, we can't automatically download Claude Code
        // Future enhancement: Use official API when available
        Err(anyhow::anyhow!(
            "Automatic download not yet available. Please:\n\
            1. Download Claude Code from: https://claude.ai/download\n\
            2. Place the 'claude' binary in: {}\n\
            3. Restart Hive to complete setup",
            self.install_dir.display()
        ))
    }
    
    /// Extract and install the downloaded archive
    async fn extract_and_install(&self, archive_path: &Path) -> Result<()> {
        info!("📦 Extracting Claude Code...");
        
        let bin_dir = self.install_dir.join("bin");
        
        #[cfg(unix)]
        {
            // Extract tar.gz on Unix systems
            let output = Command::new("tar")
                .args(&["-xzf", &archive_path.to_string_lossy(), "-C", &bin_dir.to_string_lossy()])
                .output()
                .context("Failed to extract archive")?;
            
            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to extract archive: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            
            // Make binary executable
            let output = Command::new("chmod")
                .args(&["+x", &self.binary_path.to_string_lossy()])
                .output()
                .context("Failed to make binary executable")?;
            
            if !output.status.success() {
                warn!("Failed to set executable permissions");
            }
        }
        
        #[cfg(windows)]
        {
            // Extract zip on Windows
            // Use PowerShell to extract
            let script = format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                archive_path.display(),
                bin_dir.display()
            );
            
            let output = Command::new("powershell")
                .args(&["-Command", &script])
                .output()
                .context("Failed to extract archive")?;
            
            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to extract archive: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
        
        info!("✅ Extraction complete");
        Ok(())
    }
    
    /// Check for updates using Claude's built-in update mechanism
    pub async fn check_for_updates(&self) -> Result<bool> {
        if !self.is_installed().await {
            return Ok(false);
        }
        
        info!("🔍 Checking for Claude Code updates...");
        
        // Use Claude's built-in update check
        let output = Command::new(&self.binary_path)
            .args(&["update", "--check"])
            .output()
            .context("Failed to check for updates")?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Claude Code returns specific text when updates are available
            Ok(stdout.contains("update available") || stdout.contains("new version"))
        } else {
            warn!("Update check failed: {}", String::from_utf8_lossy(&output.stderr));
            Ok(false)
        }
    }
    
    /// Perform update using Claude's built-in updater
    pub async fn perform_update(&self) -> Result<()> {
        info!("🔄 Updating Claude Code using built-in updater...");
        
        let output = Command::new(&self.binary_path)
            .args(&["update", "--yes"])
            .output()
            .context("Failed to run update")?;
        
        if output.status.success() {
            info!("✅ Claude Code updated successfully");
            Ok(())
        } else {
            error!("Update failed: {}", String::from_utf8_lossy(&output.stderr));
            Err(anyhow::anyhow!("Claude Code update failed"))
        }
    }
    
    /// Add Claude Code to PATH for current process
    pub fn add_to_path(&self) -> Result<()> {
        let bin_dir = self.install_dir.join("bin");
        let current_path = std::env::var("PATH").unwrap_or_default();
        let bin_dir_str = bin_dir.to_string_lossy();
        
        if !current_path.contains(&*bin_dir_str) {
            let separator = if cfg!(windows) { ";" } else { ":" };
            let new_path = format!("{}{}{}", bin_dir_str, separator, current_path);
            std::env::set_var("PATH", new_path);
            debug!("Added Claude Code to PATH: {}", bin_dir_str);
        }
        
        Ok(())
    }
    
    /// Get installation directory
    pub fn get_install_dir(&self) -> &Path {
        &self.install_dir
    }
}

/// Check and install Claude Code if needed
pub async fn ensure_claude_installed() -> Result<PathBuf> {
    let installer = ClaudeInstaller::new()?;
    
    if !installer.is_installed().await {
        info!("Claude Code not found, installing...");
        installer.install_or_update().await?;
    } else {
        // Check for updates in background
        let installer_clone = ClaudeInstaller::new()?;
        tokio::spawn(async move {
            if let Ok(has_updates) = installer_clone.check_for_updates().await {
                if has_updates {
                    info!("Claude Code update available");
                    // We'll let the user decide when to update through the UI
                }
            }
        });
    }
    
    // Add to PATH for this process
    installer.add_to_path()?;
    
    Ok(installer.get_binary_path().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_installer_creation() {
        let installer = ClaudeInstaller::new().unwrap();
        assert!(installer.install_dir.to_string_lossy().contains("claude-code"));
    }
    
    #[test]
    fn test_binary_name() {
        let installer = ClaudeInstaller::new().unwrap();
        let binary_name = installer.binary_path.file_name().unwrap().to_string_lossy();
        
        if cfg!(windows) {
            assert_eq!(binary_name, "claude.exe");
        } else {
            assert_eq!(binary_name, "claude");
        }
    }
}