//! Binary distribution and installation management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::fs as afs;

/// Binary distribution manager
#[derive(Debug, Clone)]
pub struct BinaryManager {
    /// Target installation directory
    pub install_dir: PathBuf,
    /// Binary name (always "hive")
    pub binary_name: String,
}

/// Binary installation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInfo {
    /// Binary version
    pub version: String,
    /// Installation timestamp
    pub installed_at: chrono::DateTime<chrono::Utc>,
    /// Binary size in bytes
    pub size: u64,
    /// SHA256 checksum
    pub checksum: String,
    /// Installation method
    pub method: BinaryInstallMethod,
}

/// Binary installation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryInstallMethod {
    /// Copied from current executable
    SelfCopy,
    /// Downloaded from release
    Download,
    /// Installed from package manager
    Package,
}

impl BinaryManager {
    /// Create a new binary manager
    pub fn new(install_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            install_dir,
            binary_name: "hive".to_string(),
        })
    }

    /// Install the binary to the target directory
    pub async fn install(&self) -> Result<()> {
        println!("📦 Installing Hive binary...");

        // Get current executable path
        let current_exe =
            std::env::current_exe().context("Failed to get current executable path")?;

        // Target binary path
        let target_path = self.install_dir.join(&self.binary_name);

        // Copy binary
        fs::copy(&current_exe, &target_path).context("Failed to copy binary")?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&target_path)?.permissions();
            perms.set_mode(0o755); // rwxr-xr-x
            fs::set_permissions(&target_path, perms)
                .context("Failed to set executable permissions")?;
        }

        // Save binary info
        let info = BinaryInfo {
            version: crate::VERSION.to_string(),
            installed_at: chrono::Utc::now(),
            size: fs::metadata(&target_path)?.len(),
            checksum: self.calculate_checksum(&target_path).await?,
            method: BinaryInstallMethod::SelfCopy,
        };

        self.save_binary_info(&info).await?;

        println!("✅ Binary installed: {}", target_path.display());

        Ok(())
    }

    /// Uninstall the binary
    pub async fn uninstall(&self) -> Result<()> {
        let binary_path = self.install_dir.join(&self.binary_name);

        if binary_path.exists() {
            fs::remove_file(&binary_path).context("Failed to remove binary")?;
            println!("🗑️  Binary removed: {}", binary_path.display());
        }

        // Remove binary info
        let info_path = self.get_binary_info_path();
        if info_path.exists() {
            fs::remove_file(&info_path).context("Failed to remove binary info")?;
        }

        Ok(())
    }

    /// Check if binary is installed
    pub fn is_installed(&self) -> bool {
        self.install_dir.join(&self.binary_name).exists()
    }

    /// Get binary information
    pub async fn get_info(&self) -> Result<Option<BinaryInfo>> {
        let info_path = self.get_binary_info_path();

        if !info_path.exists() {
            return Ok(None);
        }

        let content = afs::read_to_string(&info_path).await?;
        let info: BinaryInfo = serde_json::from_str(&content)?;

        Ok(Some(info))
    }

    /// Update binary to new version
    pub async fn update(&self, new_binary_path: &PathBuf) -> Result<()> {
        println!("🔄 Updating Hive binary...");

        let target_path = self.install_dir.join(&self.binary_name);

        // Backup current binary
        let backup_path = target_path.with_extension("backup");
        if target_path.exists() {
            fs::copy(&target_path, &backup_path).context("Failed to create backup")?;
        }

        // Copy new binary
        fs::copy(new_binary_path, &target_path).context("Failed to copy new binary")?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&target_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&target_path, perms)
                .context("Failed to set executable permissions")?;
        }

        // Verify new binary works
        if let Err(e) = self.verify_binary(&target_path).await {
            // Restore backup if verification fails
            if backup_path.exists() {
                fs::copy(&backup_path, &target_path).context("Failed to restore backup")?;
            }
            return Err(e);
        }

        // Update binary info
        let info = BinaryInfo {
            version: crate::VERSION.to_string(),
            installed_at: chrono::Utc::now(),
            size: fs::metadata(&target_path)?.len(),
            checksum: self.calculate_checksum(&target_path).await?,
            method: BinaryInstallMethod::Download,
        };

        self.save_binary_info(&info).await?;

        // Clean up backup
        if backup_path.exists() {
            fs::remove_file(&backup_path).ok();
        }

        println!("✅ Binary updated successfully!");

        Ok(())
    }

    /// Verify binary integrity and functionality
    async fn verify_binary(&self, binary_path: &PathBuf) -> Result<()> {
        // Check if file exists and is executable
        if !binary_path.exists() {
            return Err(anyhow::anyhow!("Binary does not exist"));
        }

        // Try to run --version command
        let output = tokio::process::Command::new(binary_path)
            .arg("--version")
            .output()
            .await
            .context("Failed to run binary")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Binary failed to run"));
        }

        // Check if output contains version string
        let version_output = String::from_utf8_lossy(&output.stdout);
        if !version_output.contains("hive") {
            return Err(anyhow::anyhow!("Binary does not appear to be Hive"));
        }

        Ok(())
    }

    /// Calculate SHA256 checksum of binary
    async fn calculate_checksum(&self, binary_path: &PathBuf) -> Result<String> {
        use sha2::{Digest, Sha256};

        let content = afs::read(binary_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash))
    }

    /// Save binary information
    async fn save_binary_info(&self, info: &BinaryInfo) -> Result<()> {
        let info_path = self.get_binary_info_path();

        // Ensure directory exists
        if let Some(parent) = info_path.parent() {
            afs::create_dir_all(parent).await?;
        }

        let json = serde_json::to_string_pretty(info)?;
        afs::write(&info_path, json).await?;

        Ok(())
    }

    /// Get path to binary info file
    fn get_binary_info_path(&self) -> PathBuf {
        let config_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(|home| PathBuf::from(home).join(".hive"))
            .unwrap_or_else(|_| PathBuf::from(".hive"));

        config_dir.join("binary_info.json")
    }

    /// Get binary statistics
    pub async fn get_stats(&self) -> Result<BinaryStats> {
        let binary_path = self.install_dir.join(&self.binary_name);

        if !binary_path.exists() {
            return Ok(BinaryStats {
                installed: false,
                size: 0,
                version: None,
                last_used: None,
            });
        }

        let metadata = fs::metadata(&binary_path)?;
        let info = self.get_info().await?;

        Ok(BinaryStats {
            installed: true,
            size: metadata.len(),
            version: info.map(|i| i.version),
            last_used: Some(metadata.accessed().unwrap_or(std::time::SystemTime::now())),
        })
    }
}

/// Binary statistics
#[derive(Debug, Clone, Serialize)]
pub struct BinaryStats {
    pub installed: bool,
    pub size: u64,
    pub version: Option<String>,
    pub last_used: Option<std::time::SystemTime>,
}

/// Create a self-extracting installer
pub struct InstallerBuilder {
    binary_path: PathBuf,
    target_platform: String,
    include_completions: bool,
    include_config: bool,
}

impl InstallerBuilder {
    pub fn new(binary_path: PathBuf) -> Self {
        Self {
            binary_path,
            target_platform: get_target_platform(),
            include_completions: true,
            include_config: true,
        }
    }

    pub fn with_completions(mut self, include: bool) -> Self {
        self.include_completions = include;
        self
    }

    pub fn with_config(mut self, include: bool) -> Self {
        self.include_config = include;
        self
    }

    pub async fn build(&self, output_path: &PathBuf) -> Result<()> {
        println!("🏗️  Building installer for {}...", self.target_platform);

        // Create installer package
        let installer = InstallerPackage {
            binary_path: self.binary_path.clone(),
            platform: self.target_platform.clone(),
            version: crate::VERSION.to_string(),
            include_completions: self.include_completions,
            include_config: self.include_config,
        };

        installer.create(output_path).await?;

        println!("✅ Installer created: {}", output_path.display());

        Ok(())
    }
}

/// Self-extracting installer package
#[derive(Debug, Clone)]
struct InstallerPackage {
    binary_path: PathBuf,
    platform: String,
    version: String,
    include_completions: bool,
    include_config: bool,
}

impl InstallerPackage {
    async fn create(&self, output_path: &PathBuf) -> Result<()> {
        // Create a shell script installer for Unix systems
        #[cfg(unix)]
        {
            self.create_unix_installer(output_path).await
        }

        // Create a PowerShell installer for Windows
        #[cfg(windows)]
        {
            self.create_windows_installer(output_path).await
        }
    }

    #[cfg(unix)]
    async fn create_unix_installer(&self, output_path: &PathBuf) -> Result<()> {
        let installer_script = format!(
            r#"#!/bin/bash
# Hive AI Installer v{}
# Auto-generated installer script

set -e

HIVE_VERSION="{}"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.hive"

echo "🐝 Installing Hive AI v$HIVE_VERSION..."

# Check for required commands
if ! command -v curl &> /dev/null; then
    echo "❌ curl is required but not installed."
    exit 1
fi

# Create directories
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"

# Extract and install binary
echo "📦 Installing binary..."
# Binary would be embedded here in a real installer

# Set permissions
chmod +x "$INSTALL_DIR/hive"

# Install completions
if [ "$1" != "--no-completions" ]; then
    echo "🔧 Installing shell completions..."
    # Completion installation logic
fi

# Verify installation
if "$INSTALL_DIR/hive" --version &> /dev/null; then
    echo "✅ Hive AI installed successfully!"
    echo "   Run 'hive' to get started"
else
    echo "❌ Installation verification failed"
    exit 1
fi
"#,
            crate::VERSION,
            self.version
        );

        afs::write(output_path, installer_script).await?;

        // Make installer executable
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(output_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(output_path, perms)?;
        }

        Ok(())
    }

    #[cfg(windows)]
    async fn create_windows_installer(&self, output_path: &PathBuf) -> Result<()> {
        let installer_script = format!(
            r#"# Hive AI Installer v{}
# Auto-generated PowerShell installer script

param(
    [switch]$NoCompletions
)

$ErrorActionPreference = "Stop"

$HIVE_VERSION = "{}"
$INSTALL_DIR = "$env:ProgramFiles\Hive"
$CONFIG_DIR = "$env:USERPROFILE\.hive"

Write-Host "🐝 Installing Hive AI v$HIVE_VERSION..." -ForegroundColor Green

# Check for admin privileges
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {{
    Write-Host "❌ This installer requires administrator privileges." -ForegroundColor Red
    exit 1
}}

# Create directories
New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
New-Item -ItemType Directory -Force -Path $CONFIG_DIR | Out-Null

# Extract and install binary
Write-Host "📦 Installing binary..." -ForegroundColor Blue
# Binary would be embedded here in a real installer

# Add to PATH
$path = [Environment]::GetEnvironmentVariable("PATH", "Machine")
if ($path -notlike "*$INSTALL_DIR*") {{
    [Environment]::SetEnvironmentVariable("PATH", "$path;$INSTALL_DIR", "Machine")
    Write-Host "✅ Added Hive to system PATH" -ForegroundColor Green
}}

# Verify installation
try {{
    & "$INSTALL_DIR\hive.exe" --version | Out-Null
    Write-Host "✅ Hive AI installed successfully!" -ForegroundColor Green
    Write-Host "   Run 'hive' to get started" -ForegroundColor Cyan
}} catch {{
    Write-Host "❌ Installation verification failed" -ForegroundColor Red
    exit 1
}}
"#,
            crate::VERSION,
            self.version
        );

        afs::write(output_path, installer_script).await?;

        Ok(())
    }
}

/// Get the target platform string
fn get_target_platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}
