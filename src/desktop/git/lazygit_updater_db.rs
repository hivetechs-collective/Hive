//! LazyGit updater integrated with Hive's database
//! 
//! This module uses the existing sync_metadata table to track update checks
//! and integrates with the unified database system.

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tokio::fs as async_fs;
use tracing::{info, warn, error};
use flate2::read::GzDecoder;
use tar::Archive;
use crate::core::database::{get_database, DatabaseManager};
use rusqlite::{params, OptionalExtension};

const GITHUB_API_URL: &str = "https://api.github.com/repos/jesseduffield/lazygit/releases/latest";
const UPDATE_CHECK_INTERVAL_HOURS: i64 = 24;
const SYNC_TYPE_LAZYGIT: &str = "lazygit_update";

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// LazyGit updater using the unified database
pub struct LazyGitUpdaterDB {
    install_dir: PathBuf,
    db: DatabaseManager,
}

impl LazyGitUpdaterDB {
    /// Create a new LazyGit updater
    pub async fn new() -> Result<Self> {
        let install_dir = Self::get_install_dir()?;
        
        // Ensure directory exists
        fs::create_dir_all(&install_dir)?;
        
        // Get database instance
        let db = get_database().await?;
        
        Ok(Self {
            install_dir,
            db,
        })
    }
    
    /// Get the path to the LazyGit executable
    pub async fn get_lazygit_path(&self) -> Result<PathBuf> {
        // First check if we need to update
        if self.should_check_for_update().await? {
            info!("🔄 Checking for LazyGit updates...");
            if let Err(e) = self.update_if_needed().await {
                warn!("Failed to update LazyGit: {}", e);
            }
        }
        
        // Check our managed installation first
        let managed_path = self.get_managed_lazygit_path();
        if managed_path.exists() {
            return Ok(managed_path);
        }
        
        // Check system installation
        if let Ok(system_path) = which::which("lazygit") {
            info!("Using system LazyGit at: {:?}", system_path);
            return Ok(system_path);
        }
        
        // No LazyGit found, install it
        info!("🚀 LazyGit not found, installing...");
        self.install_latest().await?;
        
        Ok(self.get_managed_lazygit_path())
    }
    
    /// Check if we should check for updates
    async fn should_check_for_update(&self) -> Result<bool> {
        let conn = self.db.get_connection()?;
        
        // Check for the most recent LazyGit update check
        let last_check: Option<String> = conn
            .query_row(
                "SELECT completed_at FROM sync_metadata 
                 WHERE sync_type = ?1 AND status = 'completed'
                 ORDER BY completed_at DESC LIMIT 1",
                params![SYNC_TYPE_LAZYGIT],
                |row| row.get(0),
            )
            .optional()?;
        
        match last_check {
            Some(timestamp) => {
                let last_check_time = DateTime::parse_from_rfc3339(&timestamp)?
                    .with_timezone(&Utc);
                let now = Utc::now();
                let elapsed = now - last_check_time;
                
                Ok(elapsed > Duration::hours(UPDATE_CHECK_INTERVAL_HOURS))
            }
            None => Ok(true), // Never checked before
        }
    }
    
    /// Update LazyGit if a newer version is available
    async fn update_if_needed(&self) -> Result<()> {
        let sync_id = self.start_sync_record().await?;
        
        match self.perform_update().await {
            Ok(version) => {
                self.complete_sync_record(&sync_id, &version).await?;
                Ok(())
            }
            Err(e) => {
                self.fail_sync_record(&sync_id, &e.to_string()).await?;
                Err(e)
            }
        }
    }
    
    /// Perform the actual update check and installation
    async fn perform_update(&self) -> Result<String> {
        let latest_version = self.get_latest_version().await?;
        let current_version = self.get_installed_version().await?;
        
        if current_version.is_none() || current_version.as_ref() != Some(&latest_version) {
            info!("🆕 New LazyGit version available: {}", latest_version);
            self.install_version(&latest_version).await?;
        } else {
            info!("✅ LazyGit is up to date: {}", latest_version);
        }
        
        Ok(latest_version)
    }
    
    /// Install the latest version of LazyGit
    async fn install_latest(&self) -> Result<()> {
        let sync_id = self.start_sync_record().await?;
        
        match self.perform_install().await {
            Ok(version) => {
                self.complete_sync_record(&sync_id, &version).await?;
                Ok(())
            }
            Err(e) => {
                self.fail_sync_record(&sync_id, &e.to_string()).await?;
                Err(e)
            }
        }
    }
    
    /// Perform the actual installation
    async fn perform_install(&self) -> Result<String> {
        let latest_version = self.get_latest_version().await?;
        self.install_version(&latest_version).await?;
        Ok(latest_version)
    }
    
    /// Start a sync record in the database
    async fn start_sync_record(&self) -> Result<String> {
        let conn = self.db.get_connection()?;
        let sync_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let next_sync = (Utc::now() + Duration::hours(UPDATE_CHECK_INTERVAL_HOURS)).to_rfc3339();
        
        conn.execute(
            "INSERT INTO sync_metadata (
                id, sync_type, started_at, status, next_sync_due, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                sync_id,
                SYNC_TYPE_LAZYGIT,
                now,
                "in_progress",
                next_sync,
                now
            ],
        )?;
        
        Ok(sync_id)
    }
    
    /// Complete a sync record
    async fn complete_sync_record(&self, sync_id: &str, version: &str) -> Result<()> {
        let conn = self.db.get_connection()?;
        let now = Utc::now().to_rfc3339();
        
        conn.execute(
            "UPDATE sync_metadata 
             SET completed_at = ?1, status = ?2, intelligence_version = ?3
             WHERE id = ?4",
            params![now, "completed", version, sync_id],
        )?;
        
        Ok(())
    }
    
    /// Fail a sync record
    async fn fail_sync_record(&self, sync_id: &str, error: &str) -> Result<()> {
        let conn = self.db.get_connection()?;
        let now = Utc::now().to_rfc3339();
        
        conn.execute(
            "UPDATE sync_metadata 
             SET completed_at = ?1, status = ?2, error_message = ?3
             WHERE id = ?4",
            params![now, "failed", error, sync_id],
        )?;
        
        Ok(())
    }
    
    /// Get the latest version from GitHub
    async fn get_latest_version(&self) -> Result<String> {
        let client = reqwest::Client::new();
        let response = client
            .get(GITHUB_API_URL)
            .header("User-Agent", "HiveAI-LazyGitUpdater")
            .send()
            .await?;
        
        let release: GitHubRelease = response.json().await?;
        Ok(release.tag_name)
    }
    
    /// Install a specific version of LazyGit
    async fn install_version(&self, version: &str) -> Result<()> {
        info!("📦 Installing LazyGit {}", version);
        
        let download_url = self.get_download_url(version).await?;
        let temp_file = self.install_dir.join("lazygit_download.tar.gz");
        
        // Download the archive
        self.download_file(&download_url, &temp_file).await?;
        
        // Extract the binary
        self.extract_binary(&temp_file).await?;
        
        // Clean up
        let _ = async_fs::remove_file(&temp_file).await;
        
        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let binary_path = self.get_managed_lazygit_path();
            let mut perms = async_fs::metadata(&binary_path).await?.permissions();
            perms.set_mode(0o755);
            async_fs::set_permissions(&binary_path, perms).await?;
        }
        
        // Store version in database settings
        self.save_version(version).await?;
        
        info!("✅ LazyGit {} installed successfully", version);
        Ok(())
    }
    
    /// Get the download URL for the current platform
    async fn get_download_url(&self, version: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.github.com/repos/jesseduffield/lazygit/releases/tags/{}",
            version
        );
        
        let response = client
            .get(&url)
            .header("User-Agent", "HiveAI-LazyGitUpdater")
            .send()
            .await?;
        
        let release: GitHubRelease = response.json().await?;
        
        // Find the right asset for the current platform
        let asset_name = Self::get_platform_asset_name();
        let asset = release
            .assets
            .iter()
            .find(|a| a.name.contains(&asset_name))
            .context("No suitable LazyGit release found for this platform")?;
        
        Ok(asset.browser_download_url.clone())
    }
    
    /// Download a file from a URL
    async fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        info!("⬇️  Downloading LazyGit from {}", url);
        
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .header("User-Agent", "HiveAI-LazyGitUpdater")
            .send()
            .await?;
        
        let content = response.bytes().await?;
        async_fs::write(dest, content).await?;
        
        Ok(())
    }
    
    /// Extract the LazyGit binary from the downloaded archive
    async fn extract_binary(&self, archive_path: &Path) -> Result<()> {
        let file = std::fs::File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            
            if path.file_name() == Some(std::ffi::OsStr::new("lazygit")) {
                let dest_path = self.get_managed_lazygit_path();
                entry.unpack(&dest_path)?;
                break;
            }
        }
        
        Ok(())
    }
    
    /// Get the currently installed version from database
    async fn get_installed_version(&self) -> Result<Option<String>> {
        let conn = self.db.get_connection()?;
        
        let version: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'lazygit_version'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        
        Ok(version)
    }
    
    /// Save version to database
    async fn save_version(&self, version: &str) -> Result<()> {
        let conn = self.db.get_connection()?;
        let now = Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, description, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                "lazygit_version",
                version,
                "Currently installed LazyGit version",
                now
            ],
        )?;
        
        Ok(())
    }
    
    /// Get the path to our managed LazyGit binary
    fn get_managed_lazygit_path(&self) -> PathBuf {
        let binary_name = if cfg!(windows) {
            "lazygit.exe"
        } else {
            "lazygit"
        };
        
        self.install_dir.join(binary_name)
    }
    
    /// Get the platform-specific asset name
    fn get_platform_asset_name() -> String {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "x86_64") => "Darwin_x86_64",
            ("macos", "aarch64") => "Darwin_arm64",
            ("linux", "x86_64") => "Linux_x86_64",
            ("linux", "aarch64") => "Linux_arm64",
            ("windows", "x86_64") => "Windows_x86_64",
            _ => panic!("Unsupported platform"),
        }.to_string()
    }
    
    /// Get the install directory
    fn get_install_dir() -> Result<PathBuf> {
        if let Some(home) = dirs::home_dir() {
            Ok(home.join(".hive").join("tools").join("lazygit"))
        } else {
            Err(anyhow::anyhow!("Could not determine home directory"))
        }
    }
}

/// Initialize LazyGit updater with database integration
pub async fn initialize_lazygit_updater_db() -> Result<PathBuf> {
    let updater = LazyGitUpdaterDB::new().await?;
    updater.get_lazygit_path().await
}

/// Force update LazyGit to the latest version
pub async fn force_update_lazygit_db() -> Result<()> {
    let updater = LazyGitUpdaterDB::new().await?;
    updater.install_latest().await
}

/// Get LazyGit update history from database
pub async fn get_lazygit_update_history() -> Result<Vec<(String, String, String)>> {
    let db = get_database().await?;
    let conn = db.get_connection()?;
    
    let mut stmt = conn.prepare(
        "SELECT completed_at, status, intelligence_version 
         FROM sync_metadata 
         WHERE sync_type = ?1 
         ORDER BY completed_at DESC 
         LIMIT 10"
    )?;
    
    let history = stmt
        .query_map(params![SYNC_TYPE_LAZYGIT], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2).unwrap_or_else(|_| "Unknown".to_string()),
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(history)
}