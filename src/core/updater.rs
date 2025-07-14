// Auto-update mechanism for global Hive AI binary
// Provides secure, atomic updates with rollback capability

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub download_url: String,
    pub checksum: String,
    pub changelog: String,
    pub release_date: DateTime<Utc>,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_info: Option<ReleaseInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub auto_check: bool,
    pub auto_install: bool,
    pub check_interval_hours: u64,
    pub last_check: Option<DateTime<Utc>>,
    pub update_channel: UpdateChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateChannel {
    Stable,
    Beta,
    Nightly,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check: true,
            auto_install: false,
            check_interval_hours: 24,
            last_check: None,
            update_channel: UpdateChannel::Stable,
        }
    }
}

pub struct AutoUpdater {
    client: Client,
    config: UpdateConfig,
    current_exe: PathBuf,
    backup_dir: PathBuf,
}

impl AutoUpdater {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent(format!("hive-ai/{}", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let current_exe = env::current_exe().context("Failed to get current executable path")?;

        let backup_dir = current_exe
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".hive-backups");

        let config = UpdateConfig::default();

        Ok(Self {
            client,
            config,
            current_exe,
            backup_dir,
        })
    }

    pub fn with_config(mut self, config: UpdateConfig) -> Self {
        self.config = config;
        self
    }

    /// Check for available updates
    pub async fn check_for_updates(&self) -> Result<UpdateInfo> {
        let current_version = env!("CARGO_PKG_VERSION");

        info!("Checking for updates (current: {})", current_version);

        let url = match self.config.update_channel {
            UpdateChannel::Stable => "https://api.hivetechs.com/releases/latest",
            UpdateChannel::Beta => "https://api.hivetechs.com/releases/beta",
            UpdateChannel::Nightly => "https://api.hivetechs.com/releases/nightly",
        };

        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to fetch release information")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Release API returned {}",
                response.status()
            ));
        }

        let release_info: ReleaseInfo = response
            .json()
            .await
            .context("Failed to parse release information")?;

        let current_ver =
            semver::Version::parse(current_version).context("Failed to parse current version")?;
        let latest_ver = semver::Version::parse(&release_info.version)
            .context("Failed to parse latest version")?;

        let update_available = latest_ver > current_ver;

        Ok(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: release_info.version.clone(),
            update_available,
            release_info: if update_available {
                Some(release_info)
            } else {
                None
            },
        })
    }

    /// Perform automatic update if available and configured
    pub async fn auto_update(&mut self) -> Result<bool> {
        if !self.should_check_for_updates() {
            return Ok(false);
        }

        let update_info = self.check_for_updates().await?;
        self.config.last_check = Some(Utc::now());

        if !update_info.update_available {
            debug!("No updates available");
            return Ok(false);
        }

        let release_info = update_info
            .release_info
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing release information"))?;

        // Only auto-install if configured or if it's a critical update
        if self.config.auto_install || release_info.critical {
            info!("Installing update to version {}", release_info.version);
            self.install_update(release_info).await?;
            Ok(true)
        } else {
            info!(
                "Update available: {} (auto-install disabled)",
                release_info.version
            );
            Ok(false)
        }
    }

    /// Install a specific update
    pub async fn install_update(&self, release_info: &ReleaseInfo) -> Result<()> {
        info!(
            "Installing Hive AI update to version {}",
            release_info.version
        );

        // Create backup directory
        async_fs::create_dir_all(&self.backup_dir)
            .await
            .context("Failed to create backup directory")?;

        // Create backup of current binary
        let backup_path = self.create_backup().await?;
        info!("Created backup at: {}", backup_path.display());

        // Download new binary
        let temp_binary = self.download_binary(release_info).await?;

        // Verify checksum
        self.verify_checksum(&temp_binary, &release_info.checksum)
            .await?;

        // Atomic replacement
        self.replace_binary(&temp_binary).await?;

        // Clean up
        async_fs::remove_file(temp_binary).await.ok();

        info!(
            "✅ Successfully updated to version {}",
            release_info.version
        );
        info!("📝 Changelog: {}", release_info.changelog);

        Ok(())
    }

    /// Download binary from release URL
    async fn download_binary(&self, release_info: &ReleaseInfo) -> Result<PathBuf> {
        let platform_url = self.get_platform_download_url(&release_info.download_url)?;

        info!("Downloading from: {}", platform_url);

        let response = self
            .client
            .get(&platform_url)
            .send()
            .await
            .context("Failed to download binary")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Download failed with status {}",
                response.status()
            ));
        }

        let temp_path = self
            .current_exe
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("hive-update-{}", uuid::Uuid::new_v4()));

        let bytes = response
            .bytes()
            .await
            .context("Failed to read download response")?;

        async_fs::write(&temp_path, bytes)
            .await
            .context("Failed to write downloaded binary")?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = async_fs::metadata(&temp_path).await?.permissions();
            perms.set_mode(0o755);
            async_fs::set_permissions(&temp_path, perms).await?;
        }

        Ok(temp_path)
    }

    /// Get platform-specific download URL
    fn get_platform_download_url(&self, base_url: &str) -> Result<String> {
        let platform = match env::consts::OS {
            "macos" => match env::consts::ARCH {
                "aarch64" => "macos-arm64",
                "x86_64" => "macos-x64",
                arch => return Err(anyhow::anyhow!("Unsupported macOS architecture: {}", arch)),
            },
            "linux" => match env::consts::ARCH {
                "aarch64" => "linux-arm64",
                "x86_64" => "linux-x64",
                arch => return Err(anyhow::anyhow!("Unsupported Linux architecture: {}", arch)),
            },
            "windows" => match env::consts::ARCH {
                "x86_64" => "windows-x64.exe",
                arch => {
                    return Err(anyhow::anyhow!(
                        "Unsupported Windows architecture: {}",
                        arch
                    ))
                }
            },
            os => return Err(anyhow::anyhow!("Unsupported operating system: {}", os)),
        };

        let url = format!("{}/hive-{}", base_url, platform);
        Ok(url)
    }

    /// Verify downloaded binary checksum
    async fn verify_checksum(&self, binary_path: &Path, expected_checksum: &str) -> Result<()> {
        use sha2::{Digest, Sha256};

        let contents = async_fs::read(binary_path)
            .await
            .context("Failed to read binary for checksum verification")?;

        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let checksum = format!("{:x}", hasher.finalize());

        if checksum != expected_checksum {
            return Err(anyhow::anyhow!(
                "Checksum verification failed: expected {}, got {}",
                expected_checksum,
                checksum
            ));
        }

        debug!("Checksum verified: {}", checksum);
        Ok(())
    }

    /// Create backup of current binary
    async fn create_backup(&self) -> Result<PathBuf> {
        let backup_name = format!(
            "hive-{}-{}",
            env!("CARGO_PKG_VERSION"),
            Utc::now().format("%Y%m%d-%H%M%S")
        );

        let backup_path = self.backup_dir.join(backup_name);

        async_fs::copy(&self.current_exe, &backup_path)
            .await
            .context("Failed to create backup")?;

        Ok(backup_path)
    }

    /// Atomically replace current binary
    async fn replace_binary(&self, new_binary: &Path) -> Result<()> {
        // On Windows, we need to rename the current binary first
        #[cfg(windows)]
        {
            let old_backup = self.current_exe.with_extension("old");
            if old_backup.exists() {
                async_fs::remove_file(&old_backup).await.ok();
            }
            async_fs::rename(&self.current_exe, &old_backup)
                .await
                .context("Failed to backup current binary on Windows")?;
        }

        // Replace the binary
        async_fs::rename(new_binary, &self.current_exe)
            .await
            .context("Failed to replace binary")?;

        Ok(())
    }

    /// Check if we should check for updates based on configuration
    fn should_check_for_updates(&self) -> bool {
        if !self.config.auto_check {
            return false;
        }

        if let Some(last_check) = self.config.last_check {
            let elapsed = Utc::now().signed_duration_since(last_check);
            elapsed.num_hours() >= self.config.check_interval_hours as i64
        } else {
            true
        }
    }

    /// Rollback to previous version
    pub async fn rollback(&self) -> Result<()> {
        // Find the most recent backup
        let mut backups = vec![];

        if self.backup_dir.exists() {
            let mut entries = async_fs::read_dir(&self.backup_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("hive-") {
                        backups.push(entry.path());
                    }
                }
            }
        }

        if backups.is_empty() {
            return Err(anyhow::anyhow!("No backups found for rollback"));
        }

        // Sort by modification time (most recent first)
        backups.sort_by_key(|path| {
            std::fs::metadata(path)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH)
        });
        backups.reverse();

        let latest_backup = &backups[0];

        info!("Rolling back to: {}", latest_backup.display());

        // Create backup of current version
        self.create_backup().await?;

        // Replace with backup
        async_fs::copy(latest_backup, &self.current_exe)
            .await
            .context("Failed to restore from backup")?;

        info!("✅ Successfully rolled back to previous version");

        Ok(())
    }

    /// Clean old backups (keep only the 5 most recent)
    pub async fn clean_backups(&self) -> Result<()> {
        if !self.backup_dir.exists() {
            return Ok(());
        }

        let mut backups = vec![];
        let mut entries = async_fs::read_dir(&self.backup_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("hive-") {
                    if let Ok(metadata) = entry.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            backups.push((entry.path(), modified));
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by_key(|(_, time)| *time);
        backups.reverse();

        // Remove all but the 5 most recent
        for (path, _) in backups.iter().skip(5) {
            if let Err(e) = async_fs::remove_file(path).await {
                warn!("Failed to remove old backup {}: {}", path.display(), e);
            } else {
                debug!("Removed old backup: {}", path.display());
            }
        }

        Ok(())
    }

    /// Get current update configuration
    pub fn get_config(&self) -> &UpdateConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: UpdateConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let fake_exe = temp_dir.path().join("hive");
        std::fs::write(&fake_exe, "fake binary").unwrap();

        let backup_dir = temp_dir.path().join("backups");

        let updater = AutoUpdater {
            client: Client::new(),
            config: UpdateConfig::default(),
            current_exe: fake_exe,
            backup_dir,
        };

        let backup_path = updater.create_backup().await.unwrap();
        assert!(backup_path.exists());

        let backup_content = std::fs::read_to_string(backup_path).unwrap();
        assert_eq!(backup_content, "fake binary");
    }

    #[tokio::test]
    async fn test_checksum_verification() {
        use sha2::{Digest, Sha256};

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test");
        let content = b"test content";
        std::fs::write(&test_file, content).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(content);
        let expected_checksum = format!("{:x}", hasher.finalize());

        let updater = AutoUpdater {
            client: Client::new(),
            config: UpdateConfig::default(),
            current_exe: temp_dir.path().join("dummy"),
            backup_dir: temp_dir.path().join("backups"),
        };

        // Should succeed with correct checksum
        updater
            .verify_checksum(&test_file, &expected_checksum)
            .await
            .unwrap();

        // Should fail with incorrect checksum
        assert!(updater
            .verify_checksum(&test_file, "wrong_checksum")
            .await
            .is_err());
    }
}
