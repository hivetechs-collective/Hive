//! Auto-update system for Hive desktop app
//!
//! Provides automatic update checking and download functionality similar to VS Code.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// Update channel preference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl Default for UpdateChannel {
    fn default() -> Self {
        UpdateChannel::Stable
    }
}

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub release_date: DateTime<Utc>,
    pub changelog_url: String,
    pub download_url: String,
    pub is_critical: bool,
}

/// Release metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub release_date: String,
    pub changelog_url: String,
    pub downloads: PlatformDownloads,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDownloads {
    pub macos_arm64: String,
    pub macos_intel: String,
    pub windows_x64: String,
    pub linux_x64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasesMetadata {
    pub stable: ReleaseInfo,
    pub beta: ReleaseInfo,
}

/// Auto-update checker
pub struct UpdateChecker {
    base_url: String,
    current_version: String,
    channel: UpdateChannel,
    client: reqwest::Client,
}

impl UpdateChecker {
    /// Create new update checker
    pub fn new(current_version: String, channel: UpdateChannel) -> Self {
        Self {
            base_url: "https://releases.hivetechs.io".to_string(),
            current_version,
            channel,
            client: reqwest::Client::new(),
        }
    }

    /// Check for available updates
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        tracing::info!("Checking for updates (channel: {:?})", self.channel);

        // Fetch releases metadata
        let releases = self.fetch_releases_metadata().await?;

        // Get target release based on channel
        let target_release = match self.channel {
            UpdateChannel::Stable => &releases.stable,
            UpdateChannel::Beta => &releases.beta,
        };

        // Compare versions
        if self.is_newer_version(&target_release.version)? {
            let download_url = self.get_platform_download_url(target_release)?;

            Ok(Some(UpdateInfo {
                version: target_release.version.clone(),
                release_date: chrono::DateTime::parse_from_rfc3339(&format!(
                    "{}T00:00:00Z",
                    target_release.release_date
                ))?
                .with_timezone(&Utc),
                changelog_url: target_release.changelog_url.clone(),
                download_url,
                is_critical: false, // TODO: Add critical flag to metadata
            }))
        } else {
            tracing::info!("Already on latest version: {}", self.current_version);
            Ok(None)
        }
    }

    /// Fetch releases metadata from server
    async fn fetch_releases_metadata(&self) -> Result<ReleasesMetadata> {
        let url = format!("{}/releases.json", self.base_url);

        // Try to fetch from server, but handle network errors gracefully
        match self
            .client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if !response.status().is_success() {
                    return Err(anyhow!(
                        "Failed to fetch releases: HTTP {}",
                        response.status()
                    ));
                }

                let releases: ReleasesMetadata = response
                    .json()
                    .await
                    .map_err(|e| anyhow!("Failed to parse releases metadata: {}", e))?;

                Ok(releases)
            }
            Err(e) => {
                // If it's a DNS/network error, return a fallback that indicates no updates
                if e.is_connect() || e.is_timeout() || e.to_string().contains("dns error") {
                    tracing::warn!(
                        "Update server unreachable, assuming no updates available: {}",
                        e
                    );
                    // Return fake metadata indicating current version is latest
                    Ok(ReleasesMetadata {
                        stable: ReleaseInfo {
                            version: self.current_version.clone(), // Same as current = no update
                            release_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                            changelog_url: "https://github.com/hivetechs/hive/releases".to_string(),
                            downloads: PlatformDownloads {
                                macos_arm64: "".to_string(),
                                macos_intel: "".to_string(),
                                windows_x64: "".to_string(),
                                linux_x64: "".to_string(),
                            },
                        },
                        beta: ReleaseInfo {
                            version: self.current_version.clone(),
                            release_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                            changelog_url: "https://github.com/hivetechs/hive/releases".to_string(),
                            downloads: PlatformDownloads {
                                macos_arm64: "".to_string(),
                                macos_intel: "".to_string(),
                                windows_x64: "".to_string(),
                                linux_x64: "".to_string(),
                            },
                        },
                    })
                } else {
                    Err(anyhow!("Failed to fetch releases metadata: {}", e))
                }
            }
        }
    }

    /// Get download URL for current platform
    fn get_platform_download_url(&self, release: &ReleaseInfo) -> Result<String> {
        let platform_url = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            &release.downloads.macos_arm64
        } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
            &release.downloads.macos_intel
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            &release.downloads.windows_x64
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            &release.downloads.linux_x64
        } else {
            return Err(anyhow!("Unsupported platform for auto-updates"));
        };

        Ok(platform_url.clone())
    }

    /// Compare version strings (semantic versioning)
    fn is_newer_version(&self, new_version: &str) -> Result<bool> {
        // Simple version comparison for now
        // TODO: Implement proper semantic versioning
        let current = self.parse_version(&self.current_version)?;
        let new = self.parse_version(new_version)?;

        Ok(new > current)
    }

    /// Parse version string into comparable tuple
    fn parse_version(&self, version: &str) -> Result<(u32, u32, u32)> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid version format: {}", version));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| anyhow!("Invalid major version: {}", parts[0]))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| anyhow!("Invalid minor version: {}", parts[1]))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| anyhow!("Invalid patch version: {}", parts[2]))?;

        Ok((major, minor, patch))
    }

    /// Download update file
    pub async fn download_update(
        &self,
        update_info: &UpdateInfo,
        progress_callback: impl Fn(u64, u64),
    ) -> Result<Vec<u8>> {
        tracing::info!("Downloading update from: {}", update_info.download_url);

        let response = self
            .client
            .get(&update_info.download_url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to start download: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Download failed: HTTP {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut buffer = Vec::new();

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow!("Download error: {}", e))?;
            buffer.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }

        tracing::info!("Download complete: {} bytes", buffer.len());
        Ok(buffer)
    }
}

/// Background update checker service
pub struct UpdateService {
    checker: UpdateChecker,
    check_interval: Duration,
}

impl UpdateService {
    /// Create new update service
    pub fn new(current_version: String, channel: UpdateChannel) -> Self {
        Self {
            checker: UpdateChecker::new(current_version, channel),
            check_interval: Duration::from_secs(6 * 60 * 60), // Check every 6 hours
        }
    }

    /// Start background update checking
    pub async fn start_background_checking(&self) -> Result<()> {
        let mut interval = tokio::time::interval(self.check_interval);

        loop {
            interval.tick().await;

            match self.checker.check_for_updates().await {
                Ok(Some(update)) => {
                    tracing::info!("Update available: {}", update.version);
                    // TODO: Send notification to UI
                }
                Ok(None) => {
                    tracing::debug!("No updates available");
                }
                Err(e) => {
                    tracing::warn!("Update check failed: {}", e);
                }
            }
        }
    }

    /// Check for updates once
    pub async fn check_once(&self) -> Result<Option<UpdateInfo>> {
        self.checker.check_for_updates().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let checker = UpdateChecker::new("2.0.2".to_string(), UpdateChannel::Stable);

        assert_eq!(checker.parse_version("2.0.2").unwrap(), (2, 0, 2));
        assert_eq!(checker.parse_version("1.5.10").unwrap(), (1, 5, 10));

        assert!(checker.is_newer_version("2.0.3").unwrap());
        assert!(checker.is_newer_version("2.1.0").unwrap());
        assert!(checker.is_newer_version("3.0.0").unwrap());

        assert!(!checker.is_newer_version("2.0.2").unwrap());
        assert!(!checker.is_newer_version("2.0.1").unwrap());
        assert!(!checker.is_newer_version("1.9.9").unwrap());
    }

    #[test]
    fn test_update_channel() {
        assert_eq!(UpdateChannel::default(), UpdateChannel::Stable);
    }
}
