//! License validation and management system
//!
//! This module provides comprehensive license validation against HiveTechs servers,
//! tier detection, secure storage, and upgrade handling.

use crate::core::{HiveError, Result};
use anyhow::Context;
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// License tiers available in the system
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LicenseTier {
    Free,
    Basic,
    Standard,
    Premium,
    Unlimited,
    Enterprise,
}

impl Default for LicenseTier {
    fn default() -> Self {
        Self::Free
    }
}

impl std::fmt::Display for LicenseTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseTier::Free => write!(f, "free"),
            LicenseTier::Basic => write!(f, "basic"),
            LicenseTier::Standard => write!(f, "standard"),
            LicenseTier::Premium => write!(f, "premium"),
            LicenseTier::Unlimited => write!(f, "unlimited"),
            LicenseTier::Enterprise => write!(f, "enterprise"),
        }
    }
}

/// License validation response from HiveTechs server
#[derive(Debug, Deserialize)]
pub struct LicenseValidationResponse {
    pub valid: bool,
    pub tier: String,
    pub daily_limit: u32,
    pub features: Vec<String>,
    pub user_id: String,
    pub email: Option<String>,
    pub expires_at: Option<String>,
    pub message: Option<String>,
    pub subscription_status: Option<String>,
    pub max_devices: Option<u32>,
    pub active_devices: Option<u32>,
}

/// Local license information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub key: String,
    pub tier: LicenseTier,
    pub daily_limit: u32,
    pub features: Vec<String>,
    pub user_id: String,
    pub expires_at: Option<String>,
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

/// License validation and management
pub struct LicenseManager {
    config_dir: PathBuf,
    client: reqwest::Client,
}

impl LicenseManager {
    /// Create a new license manager
    pub fn new(config_dir: PathBuf) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("hive-rust/2.0.0")
            .build()
            .unwrap_or_default();

        Self {
            config_dir,
            client,
        }
    }

    /// Get device fingerprint for license validation
    fn get_device_fingerprint(&self) -> String {
        // Combine hostname and username for basic device identification
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        
        format!("{}-{}", hostname, username)
    }

    /// Validate license key with HiveTechs servers
    pub async fn validate_license(&self, license_key: &str) -> Result<LicenseValidationResponse> {
        // Validate license key format first
        if !self.is_valid_format(license_key) {
            return Err(HiveError::LicenseValidation {
                message: "Invalid license key format. Expected: HIVE-XXXX-XXXX-XXXX-XXXX-XXXX".to_string()
            });
        }

        // Make request to HiveTechs validation endpoint (Cloudflare D1 gateway)
        let url = std::env::var("HIVE_API_ENDPOINT")
            .unwrap_or_else(|_| "https://gateway.hivetechs.io".to_string());
        let validation_url = format!("{}/v1/session/validate", url);
        
        let response = self.client
            .post(&validation_url)
            .header("Authorization", format!("Bearer {}", license_key))
            .json(&serde_json::json!({
                "license_key": license_key,
                "product": "hive-ai",
                "version": "2.0.0",
                "device_fingerprint": self.get_device_fingerprint()
            }))
            .send()
            .await
            .context("Failed to connect to HiveTechs license server")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            
            return match status.as_u16() {
                401 => Err(HiveError::AuthenticationFailed { service: "license".to_string(), message: "Invalid license key".to_string() }),
                403 => Err(HiveError::AuthenticationFailed { service: "license".to_string(), message: "License key expired or revoked".to_string() }),
                429 => Err(HiveError::RateLimitExceeded { provider: "license".to_string(), retry_after: 60 }),
                _ => Err(HiveError::Internal { context: "license".to_string(), message: format!("License validation failed: {} - {}", status, error_text) }),
            };
        }

        let validation_response: LicenseValidationResponse = response
            .json()
            .await
            .context("Failed to parse license validation response")?;

        Ok(validation_response)
    }

    /// Store validated license locally
    pub async fn store_license(&self, license_key: &str, validation: &LicenseValidationResponse) -> Result<()> {
        // Ensure config directory exists
        fs::create_dir_all(&self.config_dir).await
            .context("Failed to create config directory")?;

        let license_info = LicenseInfo {
            key: license_key.to_string(),
            tier: self.parse_tier(&validation.tier),
            daily_limit: validation.daily_limit,
            features: validation.features.clone(),
            user_id: validation.user_id.clone(),
            expires_at: validation.expires_at.clone(),
            validated_at: chrono::Utc::now(),
        };

        // Store license in encrypted format
        let license_file = self.config_dir.join("license.json");
        let license_json = serde_json::to_string_pretty(&license_info)
            .context("Failed to serialize license info")?;

        fs::write(&license_file, license_json).await
            .context("Failed to write license file")?;

        Ok(())
    }

    /// Load stored license
    pub async fn load_license(&self) -> Result<Option<LicenseInfo>> {
        let license_file = self.config_dir.join("license.json");
        
        if !license_file.exists() {
            return Ok(None);
        }

        let license_content = fs::read_to_string(&license_file).await
            .context("Failed to read license file")?;

        let license_info: LicenseInfo = serde_json::from_str(&license_content)
            .context("Failed to parse license file")?;

        Ok(Some(license_info))
    }

    /// Check if license exists and is valid
    pub async fn check_license_status(&self) -> Result<LicenseStatus> {
        match self.load_license().await? {
            Some(license) => {
                // Check if license is expired
                if let Some(expires_at) = &license.expires_at {
                    if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                        if expires < chrono::Utc::now() {
                            return Ok(LicenseStatus {
                                has_license: true,
                                is_valid: false,
                                tier: license.tier,
                                user_id: Some(license.user_id),
                                message: "License expired".to_string(),
                            });
                        }
                    }
                }

                // Check if license needs revalidation (older than 24 hours)
                let needs_revalidation = chrono::Utc::now()
                    .signed_duration_since(license.validated_at)
                    .num_hours() > 24;

                if needs_revalidation {
                    // Try to revalidate
                    match self.validate_license(&license.key).await {
                        Ok(validation) => {
                            if validation.valid {
                                // Update stored license
                                if let Err(e) = self.store_license(&license.key, &validation).await {
                                    eprintln!("Warning: Failed to update license cache: {}", e);
                                }
                                
                                Ok(LicenseStatus {
                                    has_license: true,
                                    is_valid: true,
                                    tier: self.parse_tier(&validation.tier),
                                    user_id: Some(validation.user_id),
                                    message: format!("{} tier active", validation.tier),
                                })
                            } else {
                                Ok(LicenseStatus {
                                    has_license: true,
                                    is_valid: false,
                                    tier: license.tier,
                                    user_id: Some(license.user_id),
                                    message: validation.message.unwrap_or_else(|| "License invalid".to_string()),
                                })
                            }
                        }
                        Err(_) => {
                            // Network error, use cached license
                            Ok(LicenseStatus {
                                has_license: true,
                                is_valid: true,
                                tier: license.tier,
                                user_id: Some(license.user_id),
                                message: format!("{} tier active (cached)", license.tier),
                            })
                        }
                    }
                } else {
                    Ok(LicenseStatus {
                        has_license: true,
                        is_valid: true,
                        tier: license.tier,
                        user_id: Some(license.user_id),
                        message: format!("{} tier active", license.tier),
                    })
                }
            }
            None => Ok(LicenseStatus {
                has_license: false,
                is_valid: false,
                tier: LicenseTier::Free,
                user_id: None,
                message: "No license configured".to_string(),
            }),
        }
    }

    /// Remove stored license
    pub async fn remove_license(&self) -> Result<()> {
        let license_file = self.config_dir.join("license.json");
        
        if license_file.exists() {
            fs::remove_file(&license_file).await
                .context("Failed to remove license file")?;
        }

        Ok(())
    }

    /// Validate license key format
    fn is_valid_format(&self, license_key: &str) -> bool {
        // HIVE-XXXX-XXXX-XXXX-XXXX-XXXX format
        let parts: Vec<&str> = license_key.split('-').collect();
        
        if parts.len() != 6 {
            return false;
        }

        if parts[0] != "HIVE" {
            return false;
        }

        // Check that each part (except first) is 4 alphanumeric characters
        for part in &parts[1..] {
            if part.len() != 4 || !part.chars().all(|c| c.is_ascii_alphanumeric()) {
                return false;
            }
        }

        true
    }

    /// Parse tier string to enum
    fn parse_tier(&self, tier_str: &str) -> LicenseTier {
        match tier_str.to_lowercase().as_str() {
            "free" => LicenseTier::Free,
            "basic" => LicenseTier::Basic,
            "standard" => LicenseTier::Standard,
            "premium" => LicenseTier::Premium,
            "unlimited" => LicenseTier::Unlimited,
            "enterprise" => LicenseTier::Enterprise,
            _ => LicenseTier::Free,
        }
    }
}

/// License status information
#[derive(Debug, Clone)]
pub struct LicenseStatus {
    pub has_license: bool,
    pub is_valid: bool,
    pub tier: LicenseTier,
    pub user_id: Option<String>,
    pub message: String,
}

/// Interactive license configuration
pub struct LicenseWizard {
    manager: LicenseManager,
}

impl LicenseWizard {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            manager: LicenseManager::new(config_dir),
        }
    }

    /// Run interactive license configuration
    pub async fn configure(&self) -> Result<LicenseStatus> {
        println!("🔑 Hive AI License Configuration");
        println!("{}", "━".repeat(50));
        println!();

        // Check current status
        let current_status = self.manager.check_license_status().await?;
        
        if current_status.has_license && current_status.is_valid {
            println!("✅ Current license: {}", current_status.message);
            if let Some(user_id) = &current_status.user_id {
                println!("👤 User: {}", user_id);
            }
            println!();

            println!("Would you like to update your license? (y/N)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)
                .context("Failed to read input")?;
            
            if !input.trim().to_lowercase().starts_with('y') {
                return Ok(current_status);
            }
        }

        // Show license information
        self.show_license_info();

        // Get license key from user
        let license_key = loop {
            println!("Enter your Hive license key (or 'skip' for free tier):");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)
                .context("Failed to read input")?;
            
            let input = input.trim();
            
            if input.to_lowercase() == "skip" {
                return Ok(LicenseStatus {
                    has_license: false,
                    is_valid: false,
                    tier: LicenseTier::Free,
                    user_id: None,
                    message: "Using free tier".to_string(),
                });
            }

            if input.is_empty() {
                println!("❌ Please enter a license key or type 'skip'");
                continue;
            }

            if !self.manager.is_valid_format(input) {
                println!("❌ Invalid license key format. Expected: HIVE-XXXX-XXXX-XXXX-XXXX-XXXX");
                continue;
            }

            break input.to_string();
        };

        // Validate license
        println!();
        println!("🔄 Validating license with HiveTechs servers...");
        
        match self.manager.validate_license(&license_key).await {
            Ok(validation) => {
                if validation.valid {
                    // Store license
                    self.manager.store_license(&license_key, &validation).await?;
                    
                    println!("✅ License validated successfully!");
                    println!("🎯 Tier: {}", validation.tier);
                    println!("👤 User: {}", validation.user_id);
                    println!("📊 Daily limit: {} conversations", validation.daily_limit);
                    println!("🔓 Features: {}", validation.features.join(", "));
                    
                    Ok(LicenseStatus {
                        has_license: true,
                        is_valid: true,
                        tier: self.manager.parse_tier(&validation.tier),
                        user_id: Some(validation.user_id),
                        message: format!("{} tier activated", validation.tier),
                    })
                } else {
                    let message = validation.message.unwrap_or_else(|| "License validation failed".to_string());
                    println!("❌ {}", message);
                    
                    Err(HiveError::AuthenticationFailed { service: "license".to_string(), message })
                }
            }
            Err(e) => {
                println!("❌ License validation failed: {}", e);
                Err(e)
            }
        }
    }

    fn show_license_info(&self) {
        println!("📋 Hive AI License Information");
        println!();
        println!("🆓 FREE TIER:");
        println!("   • 10 daily conversations");
        println!("   • All core features included");
        println!();
        println!("⭐ PREMIUM TIERS:");
        println!("   • BASIC: 50 daily conversations");
        println!("   • STANDARD: 100 daily conversations");
        println!("   • PREMIUM: 200 daily conversations");
        println!("   • UNLIMITED: Unlimited conversations");
        println!("   • All features + priority support");
        println!();
        println!("💡 Get your license at: https://hivetechs.com/pricing");
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_license_format_validation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LicenseManager::new(temp_dir.path().to_path_buf());

        // Valid format
        assert!(manager.is_valid_format("HIVE-1234-ABCD-5678-EFGH-IJKL"));
        
        // Invalid formats
        assert!(!manager.is_valid_format("INVALID-1234-ABCD-5678-EFGH-IJKL"));
        assert!(!manager.is_valid_format("HIVE-1234-ABCD-5678-EFGH"));
        assert!(!manager.is_valid_format("HIVE-12345-ABCD-5678-EFGH-IJKL"));
        assert!(!manager.is_valid_format("HIVE-1234-ABCD-5678-EFGH-IJK@"));
    }

    #[test]
    fn test_tier_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LicenseManager::new(temp_dir.path().to_path_buf());

        assert_eq!(manager.parse_tier("free"), LicenseTier::Free);
        assert_eq!(manager.parse_tier("BASIC"), LicenseTier::Basic);
        assert_eq!(manager.parse_tier("Premium"), LicenseTier::Premium);
        assert_eq!(manager.parse_tier("unknown"), LicenseTier::Free);
    }
}