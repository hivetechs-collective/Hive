//! LazyGit automatic updater and installer
//! 
//! This module handles:
//! - Checking if LazyGit is installed
//! - Downloading the latest version
//! - Automatic daily updates
//! - Platform-specific installation

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, Duration};
use serde::{Deserialize, Serialize};
use tokio::fs as async_fs;
use tracing::{info, warn, error};
use flate2::read::GzDecoder;
use tar::Archive;

const GITHUB_API_URL: &str = "https://api.github.com/repos/jesseduffield/lazygit/releases/latest";
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

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

#[derive(Debug, Serialize, Deserialize)]
struct UpdateMetadata {
    last_check: SystemTime,
    installed_version: String,
    install_path: PathBuf,
}

/// LazyGit updater and installer
pub struct LazyGitUpdater {
    app_data_dir: PathBuf,
    metadata_path: PathBuf,
    install_dir: PathBuf,
}

impl LazyGitUpdater {
    /// Create a new LazyGit updater
    pub fn new() -> Result<Self> {
        let app_data_dir = Self::get_app_data_dir()?;
        let metadata_path = app_data_dir.join("lazygit_metadata.json");
        let install_dir = app_data_dir.join("lazygit");
        
        // Ensure directories exist
        fs::create_dir_all(&app_data_dir)?;
        fs::create_dir_all(&install_dir)?;
        
        Ok(Self {
            app_data_dir,
            metadata_path,
            install_dir,
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
        if !self.metadata_path.exists() {
            return Ok(true);
        }
        
        let metadata_content = async_fs::read_to_string(&self.metadata_path).await?;
        let metadata: UpdateMetadata = serde_json::from_str(&metadata_content)?;
        
        let elapsed = SystemTime::now().duration_since(metadata.last_check)?;
        Ok(elapsed > UPDATE_CHECK_INTERVAL)
    }
    
    /// Update LazyGit if a newer version is available
    async fn update_if_needed(&self) -> Result<()> {
        let latest_version = self.get_latest_version().await?;
        let current_version = self.get_installed_version().await?;
        
        if current_version.is_none() || current_version.as_ref() != Some(&latest_version) {
            info!("🆕 New LazyGit version available: {}", latest_version);
            self.install_version(&latest_version).await?;
        } else {
            info!("✅ LazyGit is up to date: {}", latest_version);
        }
        
        // Update metadata
        self.save_metadata(&latest_version).await?;
        
        Ok(())
    }
    
    /// Install the latest version of LazyGit
    async fn install_latest(&self) -> Result<()> {
        let latest_version = self.get_latest_version().await?;
        self.install_version(&latest_version).await?;
        self.save_metadata(&latest_version).await?;
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
        let temp_file = self.app_data_dir.join("lazygit_download.tar.gz");
        
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
    
    /// Get the currently installed version
    async fn get_installed_version(&self) -> Result<Option<String>> {
        if !self.metadata_path.exists() {
            return Ok(None);
        }
        
        let content = async_fs::read_to_string(&self.metadata_path).await?;
        let metadata: UpdateMetadata = serde_json::from_str(&content)?;
        Ok(Some(metadata.installed_version))
    }
    
    /// Save update metadata
    async fn save_metadata(&self, version: &str) -> Result<()> {
        let metadata = UpdateMetadata {
            last_check: SystemTime::now(),
            installed_version: version.to_string(),
            install_path: self.get_managed_lazygit_path(),
        };
        
        let content = serde_json::to_string_pretty(&metadata)?;
        async_fs::write(&self.metadata_path, content).await?;
        
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
    
    /// Get the app data directory
    fn get_app_data_dir() -> Result<PathBuf> {
        if let Some(home) = dirs::home_dir() {
            Ok(home.join(".hive").join("tools"))
        } else {
            Err(anyhow::anyhow!("Could not determine home directory"))
        }
    }
}

/// Initialize LazyGit updater on app startup
pub async fn initialize_lazygit_updater() -> Result<PathBuf> {
    let updater = LazyGitUpdater::new()?;
    updater.get_lazygit_path().await
}

/// Force update LazyGit to the latest version
pub async fn force_update_lazygit() -> Result<()> {
    let updater = LazyGitUpdater::new()?;
    updater.install_latest().await
}