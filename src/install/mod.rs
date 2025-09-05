//! Installation and package management system
//!
//! This module provides global installation capabilities similar to Claude Code,
//! including binary distribution, shell completions, and self-updating mechanisms.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs as afs;

pub mod binary;
pub mod cache;
pub mod completions;
pub mod npm;
pub mod updater;
pub mod validation;

/// Installation manager for global Hive AI deployment
#[derive(Debug, Clone)]
pub struct InstallationManager {
    /// Installation directory (e.g., /usr/local/bin)
    pub install_dir: PathBuf,
    /// Configuration directory (e.g., ~/.hive)
    pub config_dir: PathBuf,
    /// Cache directory (e.g., ~/.hive/cache)
    pub cache_dir: PathBuf,
    /// Shell completion directory
    pub completion_dir: PathBuf,
    /// Current installation status
    pub status: InstallationStatus,
}

/// Installation status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationStatus {
    /// Whether Hive is installed globally
    pub is_installed: bool,
    /// Current version installed
    pub version: String,
    /// Installation method used (binary, npm, cargo, etc.)
    pub method: InstallMethod,
    /// Installation timestamp
    pub installed_at: chrono::DateTime<chrono::Utc>,
    /// Shell completions installed
    pub completions: Vec<String>,
    /// Auto-update enabled
    pub auto_update: bool,
}

/// Installation methods supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallMethod {
    /// Direct binary installation
    Binary,
    /// NPM package installation
    Npm,
    /// Cargo installation
    Cargo,
    /// Package manager (homebrew, apt, etc.)
    Package(String),
}

/// Installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Target installation directory
    pub install_dir: PathBuf,
    /// Enable shell completions
    pub enable_completions: bool,
    /// Shells to install completions for
    pub shells: Vec<String>,
    /// Enable auto-updates
    pub auto_update: bool,
    /// Update check frequency (in hours)
    pub update_check_interval: u32,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            install_dir: default_install_dir(),
            enable_completions: true,
            shells: vec!["bash".to_string(), "zsh".to_string(), "fish".to_string()],
            auto_update: true,
            update_check_interval: 24, // Check daily
        }
    }
}

impl InstallationManager {
    /// Create a new installation manager
    pub async fn new() -> Result<Self> {
        let config_dir = get_config_dir()?;
        let cache_dir = config_dir.join("cache");
        let completion_dir = get_completion_dir()?;
        let install_dir = default_install_dir();

        // Ensure directories exist
        afs::create_dir_all(&config_dir).await?;
        afs::create_dir_all(&cache_dir).await?;

        let status = Self::detect_installation_status(&install_dir).await?;

        Ok(Self {
            install_dir,
            config_dir,
            cache_dir,
            completion_dir,
            status,
        })
    }

    /// Detect current installation status
    async fn detect_installation_status(install_dir: &Path) -> Result<InstallationStatus> {
        let binary_path = install_dir.join("hive");
        let is_installed = binary_path.exists();

        if is_installed {
            // Try to get version from installed binary
            let version = match Command::new(&binary_path).arg("--version").output() {
                Ok(output) => String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .split_whitespace()
                    .last()
                    .unwrap_or(crate::VERSION)
                    .to_string(),
                Err(_) => crate::VERSION.to_string(),
            };

            Ok(InstallationStatus {
                is_installed: true,
                version,
                method: InstallMethod::Binary,    // Default assumption
                installed_at: chrono::Utc::now(), // Placeholder
                completions: vec![],
                auto_update: true,
            })
        } else {
            Ok(InstallationStatus {
                is_installed: false,
                version: crate::VERSION.to_string(),
                method: InstallMethod::Binary,
                installed_at: chrono::Utc::now(),
                completions: vec![],
                auto_update: true,
            })
        }
    }

    /// Install Hive globally
    pub async fn install(&mut self, config: InstallConfig) -> Result<()> {
        println!("🚀 Installing Hive AI globally...");

        // Check permissions
        self.check_permissions(&config.install_dir)?;

        // Install binary
        self.install_binary(&config.install_dir).await?;

        // Install shell completions
        if config.enable_completions {
            self.install_completions(&config.shells).await?;
        }

        // Configure auto-updates
        if config.auto_update {
            self.configure_auto_updates(config.update_check_interval)
                .await?;
        }

        // Update status
        self.status.is_installed = true;
        self.status.version = crate::VERSION.to_string();
        self.status.installed_at = chrono::Utc::now();
        self.status.auto_update = config.auto_update;

        // Save installation status
        self.save_status().await?;

        println!("✅ Hive AI installed successfully!");
        println!("   Binary: {}", config.install_dir.join("hive").display());
        println!("   Config: {}", self.config_dir.display());

        if config.enable_completions {
            println!(
                "   Shell completions installed for: {}",
                config.shells.join(", ")
            );
        }

        Ok(())
    }

    /// Uninstall Hive globally
    pub async fn uninstall(&mut self) -> Result<()> {
        println!("🗑️  Uninstalling Hive AI...");

        // Remove binary
        let binary_path = self.install_dir.join("hive");
        if binary_path.exists() {
            fs::remove_file(&binary_path).context("Failed to remove binary")?;
        }

        // Remove shell completions
        self.remove_completions().await?;

        // Remove auto-update configuration
        self.remove_auto_updates().await?;

        // Update status
        self.status.is_installed = false;
        self.save_status().await?;

        println!("✅ Hive AI uninstalled successfully!");
        println!(
            "   Configuration preserved at: {}",
            self.config_dir.display()
        );

        Ok(())
    }

    /// Check if an update is available
    pub async fn check_for_updates(&self) -> Result<Option<String>> {
        let updater = updater::UpdateManager::new()?;
        updater.check_for_updates().await
    }

    /// Update Hive to the latest version
    pub async fn update(&mut self) -> Result<bool> {
        println!("🔄 Checking for updates...");

        let updater = updater::UpdateManager::new()?;
        let updated = updater.update().await?;

        if updated {
            // Update our status
            self.status.version = crate::VERSION.to_string();
            self.save_status().await?;

            println!("✅ Hive AI updated successfully!");
        } else {
            println!("✅ Hive AI is already up to date!");
        }

        Ok(updated)
    }

    /// Install binary to target directory
    async fn install_binary(&self, install_dir: &Path) -> Result<()> {
        let binary_manager = binary::BinaryManager::new(install_dir.to_path_buf())?;
        binary_manager.install().await
    }

    /// Install shell completions
    async fn install_completions(&self, shells: &[String]) -> Result<()> {
        let completion_manager = completions::CompletionManager::new(self.completion_dir.clone())?;
        completion_manager.install(shells).await
    }

    /// Remove shell completions
    async fn remove_completions(&self) -> Result<()> {
        let completion_manager = completions::CompletionManager::new(self.completion_dir.clone())?;
        completion_manager.remove().await
    }

    /// Configure auto-updates
    async fn configure_auto_updates(&self, interval_hours: u32) -> Result<()> {
        let updater = updater::UpdateManager::new()?;
        updater.configure_auto_updates(interval_hours).await
    }

    /// Remove auto-update configuration
    async fn remove_auto_updates(&self) -> Result<()> {
        let updater = updater::UpdateManager::new()?;
        updater.remove_auto_updates().await
    }

    /// Check if we have permissions to install
    fn check_permissions(&self, install_dir: &Path) -> Result<()> {
        if !install_dir.exists() {
            fs::create_dir_all(install_dir).context("Failed to create installation directory")?;
        }

        // Try to write a test file
        let test_file = install_dir.join(".hive_test");
        fs::write(&test_file, "test")
            .context("No permission to write to installation directory")?;
        fs::remove_file(&test_file).context("Failed to clean up test file")?;

        Ok(())
    }

    /// Save installation status
    async fn save_status(&self) -> Result<()> {
        let status_path = self.config_dir.join("install_status.json");
        let json = serde_json::to_string_pretty(&self.status)?;
        afs::write(status_path, json).await?;
        Ok(())
    }

    /// Get installation information
    pub fn get_info(&self) -> InstallationInfo {
        InstallationInfo {
            is_installed: self.status.is_installed,
            version: self.status.version.clone(),
            install_dir: self.install_dir.clone(),
            config_dir: self.config_dir.clone(),
            cache_dir: self.cache_dir.clone(),
            method: self.status.method.clone(),
            installed_at: self.status.installed_at,
            completions: self.status.completions.clone(),
            auto_update: self.status.auto_update,
        }
    }
}

/// Installation information for display
#[derive(Debug, Clone, Serialize)]
pub struct InstallationInfo {
    pub is_installed: bool,
    pub version: String,
    pub install_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub method: InstallMethod,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub completions: Vec<String>,
    pub auto_update: bool,
}

/// Get the default installation directory
fn default_install_dir() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from("/usr/local/bin")
    }
    #[cfg(windows)]
    {
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            PathBuf::from(program_files).join("Hive")
        } else {
            PathBuf::from("C:\\Program Files\\Hive")
        }
    }
}

/// Get the configuration directory
fn get_config_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Cannot determine home directory")?;

    Ok(PathBuf::from(home).join(".hive"))
}

/// Get the shell completion directory
fn get_completion_dir() -> Result<PathBuf> {
    #[cfg(unix)]
    {
        Ok(PathBuf::from(
            "/usr/local/share/bash-completion/completions",
        ))
    }
    #[cfg(windows)]
    {
        // Windows doesn't have a standard completion directory
        get_config_dir().map(|p| p.join("completions"))
    }
}
