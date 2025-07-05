//! Self-updating mechanism for Hive AI

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;
use tokio::fs as afs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Update manager for Hive AI
#[derive(Debug, Clone)]
pub struct UpdateManager {
    /// Configuration directory
    pub config_dir: PathBuf,
    /// Update configuration
    pub config: UpdateConfig,
    /// Release information
    pub release_info: Option<ReleaseInfo>,
}

/// Update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Auto-update enabled
    pub auto_update: bool,
    /// Update check interval in hours
    pub check_interval: u32,
    /// Last update check
    pub last_check: Option<DateTime<Utc>>,
    /// Update channel (stable, beta, dev)
    pub channel: UpdateChannel,
    /// GitHub repository for updates
    pub repository: String,
    /// Pre-release versions allowed
    pub allow_prerelease: bool,
}

/// Update channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateChannel {
    /// Stable releases only
    Stable,
    /// Beta releases
    Beta,
    /// Development releases
    Dev,
}

/// Release information from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// Release tag name
    pub tag_name: String,
    /// Release name
    pub name: String,
    /// Release description
    pub body: String,
    /// Release URL
    pub html_url: String,
    /// Published timestamp
    pub published_at: DateTime<Utc>,
    /// Is prerelease
    pub prerelease: bool,
    /// Release assets
    pub assets: Vec<ReleaseAsset>,
}

/// Release asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    /// Asset name
    pub name: String,
    /// Download URL
    pub browser_download_url: String,
    /// Asset size
    pub size: u64,
    /// Content type
    pub content_type: String,
}

/// Update status
#[derive(Debug, Clone, Serialize)]
pub enum UpdateStatus {
    /// No update available
    UpToDate,
    /// Update available
    UpdateAvailable {
        current_version: String,
        latest_version: String,
        release_info: ReleaseInfo,
    },
    /// Update in progress
    Updating {
        progress: f32,
        status: String,
    },
    /// Update completed
    Updated {
        old_version: String,
        new_version: String,
    },
    /// Update failed
    Failed {
        error: String,
    },
}

/// Update notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotification {
    /// Notification message
    pub message: String,
    /// Notification type
    pub notification_type: NotificationType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Is critical update
    pub critical: bool,
}

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    /// Update available
    UpdateAvailable,
    /// Update completed
    UpdateCompleted,
    /// Update failed
    UpdateFailed,
    /// Critical security update
    CriticalUpdate,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_update: true,
            check_interval: 24, // Check daily
            last_check: None,
            channel: UpdateChannel::Stable,
            repository: "hivetechs/hive-ai".to_string(),
            allow_prerelease: false,
        }
    }
}

impl UpdateManager {
    /// Create a new update manager
    pub fn new() -> Result<Self> {
        let config_dir = get_config_dir()?;
        let config = Self::load_config(&config_dir)?;
        
        Ok(Self {
            config_dir,
            config,
            release_info: None,
        })
    }

    /// Load update configuration
    fn load_config(config_dir: &PathBuf) -> Result<UpdateConfig> {
        let config_path = config_dir.join("update_config.toml");
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: UpdateConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(UpdateConfig::default())
        }
    }

    /// Save update configuration
    async fn save_config(&self) -> Result<()> {
        let config_path = self.config_dir.join("update_config.toml");
        
        // Ensure directory exists
        afs::create_dir_all(&self.config_dir).await?;
        
        let toml = toml::to_string_pretty(&self.config)?;
        afs::write(&config_path, toml).await?;
        
        Ok(())
    }

    /// Check for updates
    pub async fn check_for_updates(&self) -> Result<Option<String>> {
        println!("🔍 Checking for updates...");
        
        // Get latest release information
        let release_info = self.get_latest_release().await?;
        
        // Compare versions
        let current_version = crate::VERSION;
        let latest_version = release_info.tag_name.trim_start_matches('v');
        
        if self.is_newer_version(current_version, latest_version)? {
            println!("✨ Update available: {} → {}", current_version, latest_version);
            Ok(Some(latest_version.to_string()))
        } else {
            println!("✅ Already up to date ({})", current_version);
            Ok(None)
        }
    }

    /// Get latest release from GitHub
    async fn get_latest_release(&self) -> Result<ReleaseInfo> {
        let client = reqwest::Client::new();
        let url = format!("https://api.github.com/repos/{}/releases/latest", self.config.repository);
        
        let response = client
            .get(&url)
            .header("User-Agent", format!("hive-ai/{}", crate::VERSION))
            .send()
            .await
            .context("Failed to fetch release information")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("GitHub API request failed: {}", response.status()));
        }
        
        let release_info: ReleaseInfo = response
            .json()
            .await
            .context("Failed to parse release information")?;
        
        Ok(release_info)
    }

    /// Check if a version is newer than current
    fn is_newer_version(&self, current: &str, latest: &str) -> Result<bool> {
        // Simple version comparison - in a real implementation, use semver
        let current_parts: Vec<u32> = current
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect();
        
        let latest_parts: Vec<u32> = latest
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect();
        
        // Compare major.minor.patch
        for i in 0..3 {
            let current_part = current_parts.get(i).copied().unwrap_or(0);
            let latest_part = latest_parts.get(i).copied().unwrap_or(0);
            
            if latest_part > current_part {
                return Ok(true);
            } else if latest_part < current_part {
                return Ok(false);
            }
        }
        
        Ok(false) // Versions are equal
    }

    /// Update to latest version
    pub async fn update(&self) -> Result<bool> {
        println!("🔄 Starting update process...");
        
        // Check for updates first
        let latest_version = match self.check_for_updates().await? {
            Some(version) => version,
            None => return Ok(false), // Already up to date
        };
        
        // Get release information
        let release_info = self.get_latest_release().await?;
        
        // Find appropriate asset for current platform
        let asset = self.find_platform_asset(&release_info.assets)?;
        
        // Download the update
        let temp_dir = std::env::temp_dir().join("hive_update");
        afs::create_dir_all(&temp_dir).await?;
        
        let download_path = temp_dir.join(&asset.name);
        self.download_asset(&asset, &download_path).await?;
        
        // Verify download
        self.verify_download(&download_path).await?;
        
        // Apply update
        self.apply_update(&download_path).await?;
        
        // Clean up
        afs::remove_dir_all(&temp_dir).await?;
        
        // Create update notification
        let notification = UpdateNotification {
            message: format!("Successfully updated to version {}", latest_version),
            notification_type: NotificationType::UpdateCompleted,
            timestamp: Utc::now(),
            critical: false,
        };
        
        self.save_notification(&notification).await?;
        
        println!("✅ Update completed successfully!");
        
        Ok(true)
    }

    /// Find appropriate release asset for current platform
    fn find_platform_asset<'a>(&self, assets: &'a [ReleaseAsset]) -> Result<&'a ReleaseAsset> {
        let platform = get_platform_string();
        
        // Look for exact platform match
        for asset in assets {
            if asset.name.contains(&platform) {
                return Ok(asset);
            }
        }
        
        // Look for generic binary
        for asset in assets {
            if asset.name.contains("hive") && !asset.name.contains(".sha256") {
                return Ok(asset);
            }
        }
        
        Err(anyhow::anyhow!("No suitable release asset found for platform: {}", platform))
    }

    /// Download release asset
    async fn download_asset(&self, asset: &ReleaseAsset, path: &PathBuf) -> Result<()> {
        println!("📥 Downloading {} ({} bytes)...", asset.name, asset.size);
        
        let client = reqwest::Client::new();
        let response = client
            .get(&asset.browser_download_url)
            .header("User-Agent", format!("hive-ai/{}", crate::VERSION))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Download failed: {}", response.status()));
        }
        
        let mut file = tokio::fs::File::create(path).await?;
        let mut stream = response.bytes_stream();
        
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }
        
        println!("✅ Download completed");
        
        Ok(())
    }

    /// Verify downloaded file
    async fn verify_download(&self, path: &PathBuf) -> Result<()> {
        println!("🔐 Verifying download...");
        
        // Check if file exists and is not empty
        let metadata = fs::metadata(path)?;
        if metadata.len() == 0 {
            return Err(anyhow::anyhow!("Downloaded file is empty"));
        }
        
        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms)?;
        }
        
        // Try to run --version command
        let output = tokio::process::Command::new(path)
            .arg("--version")
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Downloaded binary failed to run"));
        }
        
        println!("✅ Download verification passed");
        
        Ok(())
    }

    /// Apply the update
    async fn apply_update(&self, new_binary_path: &PathBuf) -> Result<()> {
        println!("🔄 Applying update...");
        
        // Get current binary path
        let current_binary = std::env::current_exe()?;
        
        // Create backup
        let backup_path = current_binary.with_extension("backup");
        fs::copy(&current_binary, &backup_path)?;
        
        // Replace binary
        fs::copy(new_binary_path, &current_binary)?;
        
        // Verify the update worked
        let output = tokio::process::Command::new(&current_binary)
            .arg("--version")
            .output()
            .await?;
        
        if !output.status.success() {
            // Restore backup if verification fails
            fs::copy(&backup_path, &current_binary)?;
            return Err(anyhow::anyhow!("Update verification failed, restored backup"));
        }
        
        // Clean up backup
        fs::remove_file(&backup_path)?;
        
        println!("✅ Update applied successfully");
        
        Ok(())
    }

    /// Configure auto-updates
    pub async fn configure_auto_updates(&self, interval_hours: u32) -> Result<()> {
        let mut config = self.config.clone();
        config.auto_update = true;
        config.check_interval = interval_hours;
        
        // Save configuration
        let mut manager = self.clone();
        manager.config = config;
        manager.save_config().await?;
        
        println!("✅ Auto-updates configured (check every {} hours)", interval_hours);
        
        Ok(())
    }

    /// Remove auto-update configuration
    pub async fn remove_auto_updates(&self) -> Result<()> {
        let mut config = self.config.clone();
        config.auto_update = false;
        
        // Save configuration
        let mut manager = self.clone();
        manager.config = config;
        manager.save_config().await?;
        
        println!("✅ Auto-updates disabled");
        
        Ok(())
    }

    /// Save update notification
    async fn save_notification(&self, notification: &UpdateNotification) -> Result<()> {
        let notifications_path = self.config_dir.join("notifications.json");
        
        // Load existing notifications
        let mut notifications = if notifications_path.exists() {
            let content = afs::read_to_string(&notifications_path).await?;
            serde_json::from_str::<Vec<UpdateNotification>>(&content)?
        } else {
            Vec::new()
        };
        
        // Add new notification
        notifications.push(notification.clone());
        
        // Keep only last 50 notifications
        if notifications.len() > 50 {
            notifications.drain(0..notifications.len() - 50);
        }
        
        // Save notifications
        let json = serde_json::to_string_pretty(&notifications)?;
        afs::write(&notifications_path, json).await?;
        
        Ok(())
    }

    /// Get recent notifications
    pub async fn get_notifications(&self) -> Result<Vec<UpdateNotification>> {
        let notifications_path = self.config_dir.join("notifications.json");
        
        if !notifications_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = afs::read_to_string(&notifications_path).await?;
        let notifications: Vec<UpdateNotification> = serde_json::from_str(&content)?;
        
        Ok(notifications)
    }

    /// Check if auto-update should run
    pub async fn should_auto_update(&self) -> Result<bool> {
        if !self.config.auto_update {
            return Ok(false);
        }
        
        // Check if enough time has passed since last check
        if let Some(last_check) = self.config.last_check {
            let elapsed = Utc::now().signed_duration_since(last_check);
            let check_interval = chrono::Duration::hours(self.config.check_interval as i64);
            
            if elapsed < check_interval {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    /// Get update status
    pub async fn get_status(&self) -> Result<UpdateStatus> {
        match self.check_for_updates().await? {
            Some(latest_version) => {
                let release_info = self.get_latest_release().await?;
                Ok(UpdateStatus::UpdateAvailable {
                    current_version: crate::VERSION.to_string(),
                    latest_version,
                    release_info,
                })
            }
            None => Ok(UpdateStatus::UpToDate),
        }
    }
}

/// Get configuration directory
fn get_config_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Cannot determine home directory")?;
    
    Ok(PathBuf::from(home).join(".hive"))
}

/// Get platform string for release assets
fn get_platform_string() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    
    match (os, arch) {
        ("linux", "x86_64") => "linux-x64".to_string(),
        ("linux", "aarch64") => "linux-arm64".to_string(),
        ("macos", "x86_64") => "darwin-x64".to_string(),
        ("macos", "aarch64") => "darwin-arm64".to_string(),
        ("windows", "x86_64") => "windows-x64".to_string(),
        ("windows", "aarch64") => "windows-arm64".to_string(),
        _ => format!("{}-{}", os, arch),
    }
}

/// Background update checker
pub struct BackgroundUpdateChecker {
    manager: UpdateManager,
}

impl BackgroundUpdateChecker {
    /// Create a new background update checker
    pub fn new() -> Result<Self> {
        Ok(Self {
            manager: UpdateManager::new()?,
        })
    }

    /// Start background update checking
    pub async fn start(&self) -> Result<()> {
        // Check if auto-update is enabled
        if !self.manager.config.auto_update {
            return Ok(());
        }
        
        // Check if we should run
        if !self.manager.should_auto_update().await? {
            return Ok(());
        }
        
        // Check for updates (not spawning a separate task since self is borrowed)
        if let Err(e) = self.check_and_notify().await {
            eprintln!("Update check failed: {}", e);
        }
        
        Ok(())
    }

    /// Check for updates and notify
    async fn check_and_notify(&self) -> Result<()> {
        if let Some(latest_version) = self.manager.check_for_updates().await? {
            let notification = UpdateNotification {
                message: format!("Update available: {}", latest_version),
                notification_type: NotificationType::UpdateAvailable,
                timestamp: Utc::now(),
                critical: false,
            };
            
            self.manager.save_notification(&notification).await?;
            
            // Show notification to user
            println!("🔔 Update available: {}", latest_version);
            println!("   Run 'hive update' to install");
        }
        
        Ok(())
    }
}