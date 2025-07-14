//! Security configuration management for Hive AI
//!
//! This module handles security policy configuration, trust management
//! settings, and enterprise security controls.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use tokio::fs;
use tracing::{info, warn, debug};

use super::security::{SecurityPolicy, TrustLevel};

/// Complete security configuration for Hive AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Core security policy
    pub policy: SecurityPolicy,
    /// Trust management settings
    pub trust_settings: TrustSettings,
    /// Audit configuration
    pub audit_config: AuditConfig,
    /// Enterprise security controls
    pub enterprise: EnterpriseSecurityConfig,
    /// Network security settings
    pub network: NetworkSecurityConfig,
    /// Execution security settings
    pub execution: ExecutionSecurityConfig,
}

/// Trust management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustSettings {
    /// Enable interactive trust prompts
    pub interactive_prompts: bool,
    /// Auto-trust subdirectories of trusted directories
    pub auto_trust_subdirs: bool,
    /// Trust expiry time in hours (0 = never expire)
    pub trust_expiry_hours: u32,
    /// Require confirmation for sensitive operations
    pub require_confirmation: bool,
    /// Maximum number of trusted paths
    pub max_trusted_paths: usize,
    /// Default trust level for new paths
    pub default_trust_level: TrustLevel,
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Audit log file path
    pub log_path: PathBuf,
    /// Maximum log file size in MB
    pub max_log_size_mb: u64,
    /// Number of log files to retain
    pub log_retention_count: usize,
    /// Log level (debug, info, warn, error)
    pub log_level: String,
    /// Include file content hashes in audit log
    pub include_content_hashes: bool,
}

/// Enterprise security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseSecurityConfig {
    /// Enable enterprise security mode
    pub enabled: bool,
    /// Require multi-factor authentication for sensitive operations
    pub require_mfa: bool,
    /// Centralized policy server URL
    pub policy_server_url: Option<String>,
    /// Policy refresh interval in minutes
    pub policy_refresh_minutes: u32,
    /// Allowed user groups
    pub allowed_groups: HashSet<String>,
    /// Blocked user groups
    pub blocked_groups: HashSet<String>,
    /// LDAP/Active Directory integration
    pub ldap_config: Option<LdapConfig>,
}

/// LDAP configuration for enterprise integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapConfig {
    /// LDAP server URL
    pub server_url: String,
    /// Base DN for user searches
    pub base_dn: String,
    /// User search filter
    pub user_filter: String,
    /// Group search filter
    pub group_filter: String,
    /// Bind DN for authentication
    pub bind_dn: String,
    /// Connection timeout in seconds
    pub timeout_seconds: u32,
}

/// Network security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecurityConfig {
    /// Enable network request filtering
    pub filter_requests: bool,
    /// Allowed domains for network requests
    pub allowed_domains: HashSet<String>,
    /// Blocked domains
    pub blocked_domains: HashSet<String>,
    /// Require HTTPS for all requests
    pub require_https: bool,
    /// Enable certificate validation
    pub validate_certificates: bool,
    /// Proxy server configuration
    pub proxy_config: Option<ProxyConfig>,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy server URL
    pub url: String,
    /// Proxy username (optional)
    pub username: Option<String>,
    /// Proxy password (optional)
    pub password: Option<String>,
    /// Bypass proxy for these domains
    pub bypass_domains: HashSet<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Maximum requests per minute
    pub max_requests_per_minute: u32,
    /// Maximum requests per hour
    pub max_requests_per_hour: u32,
    /// Burst allowance
    pub burst_allowance: u32,
}

/// Code execution security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSecurityConfig {
    /// Enable sandboxed execution
    pub enable_sandbox: bool,
    /// Allowed commands for execution
    pub allowed_commands: HashSet<String>,
    /// Blocked commands
    pub blocked_commands: HashSet<String>,
    /// Maximum execution time in seconds
    pub max_execution_time: u32,
    /// Environment variable whitelist
    pub allowed_env_vars: HashSet<String>,
    /// Working directory restrictions
    pub restrict_working_dir: bool,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Resource limits for code execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: u32,
    /// Maximum number of open files
    pub max_open_files: u32,
    /// Maximum number of processes
    pub max_processes: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            policy: SecurityPolicy::default(),
            trust_settings: TrustSettings::default(),
            audit_config: AuditConfig::default(),
            enterprise: EnterpriseSecurityConfig::default(),
            network: NetworkSecurityConfig::default(),
            execution: ExecutionSecurityConfig::default(),
        }
    }
}

impl Default for TrustSettings {
    fn default() -> Self {
        Self {
            interactive_prompts: true,
            auto_trust_subdirs: true,
            trust_expiry_hours: 0, // Never expire
            require_confirmation: false,
            max_trusted_paths: 1000,
            default_trust_level: TrustLevel::Untrusted,
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_path: PathBuf::from("~/.hive/security_audit.log"),
            max_log_size_mb: 100,
            log_retention_count: 5,
            log_level: "info".to_string(),
            include_content_hashes: false,
        }
    }
}

impl Default for EnterpriseSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            require_mfa: false,
            policy_server_url: None,
            policy_refresh_minutes: 60,
            allowed_groups: HashSet::new(),
            blocked_groups: HashSet::new(),
            ldap_config: None,
        }
    }
}

impl Default for NetworkSecurityConfig {
    fn default() -> Self {
        let mut allowed_domains = HashSet::new();
        allowed_domains.insert("openrouter.ai".to_string());
        allowed_domains.insert("workers.dev".to_string());
        allowed_domains.insert("cloudflare.com".to_string());

        Self {
            filter_requests: true,
            allowed_domains,
            blocked_domains: HashSet::new(),
            require_https: true,
            validate_certificates: true,
            proxy_config: None,
            rate_limiting: RateLimitConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_requests_per_minute: 60,
            max_requests_per_hour: 1000,
            burst_allowance: 10,
        }
    }
}

impl Default for ExecutionSecurityConfig {
    fn default() -> Self {
        let mut allowed_commands = HashSet::new();
        allowed_commands.insert("cargo".to_string());
        allowed_commands.insert("git".to_string());
        allowed_commands.insert("npm".to_string());
        allowed_commands.insert("hive".to_string());

        let mut allowed_env_vars = HashSet::new();
        allowed_env_vars.insert("PATH".to_string());
        allowed_env_vars.insert("HOME".to_string());
        allowed_env_vars.insert("USER".to_string());
        allowed_env_vars.insert("CARGO_HOME".to_string());
        allowed_env_vars.insert("RUSTUP_HOME".to_string());

        Self {
            enable_sandbox: false,
            allowed_commands,
            blocked_commands: HashSet::new(),
            max_execution_time: 300, // 5 minutes
            allowed_env_vars,
            restrict_working_dir: false,
            resource_limits: ResourceLimits::default(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024, // 1GB
            max_cpu_percent: 80,
            max_open_files: 1024,
            max_processes: 100,
        }
    }
}

/// Security configuration manager
#[derive(Debug)]
pub struct SecurityConfigManager {
    config: SecurityConfig,
    config_path: PathBuf,
}

impl SecurityConfigManager {
    /// Create a new security configuration manager
    pub async fn new(config_dir: &Path) -> Result<Self> {
        let config_path = config_dir.join("security.toml");

        // Ensure config directory exists
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).await
                .context("Failed to create config directory")?;
        }

        let config = if config_path.exists() {
            Self::load_config(&config_path).await?
        } else {
            let default_config = SecurityConfig::default();
            Self::save_config(&config_path, &default_config).await?;
            default_config
        };

        Ok(Self {
            config,
            config_path,
        })
    }

    /// Load security configuration from file
    async fn load_config(path: &Path) -> Result<SecurityConfig> {
        let content = fs::read_to_string(path).await
            .with_context(|| format!("Failed to read security config: {}", path.display()))?;

        let config: SecurityConfig = toml::from_str(&content)
            .context("Failed to parse security configuration")?;

        info!("Loaded security configuration from: {}", path.display());
        Ok(config)
    }

    /// Save security configuration to file
    async fn save_config(path: &Path, config: &SecurityConfig) -> Result<()> {
        let content = toml::to_string_pretty(config)
            .context("Failed to serialize security configuration")?;

        fs::write(path, content).await
            .with_context(|| format!("Failed to write security config: {}", path.display()))?;

        debug!("Saved security configuration to: {}", path.display());
        Ok(())
    }

    /// Get the current security configuration
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Update the security configuration
    pub async fn update_config(&mut self, config: SecurityConfig) -> Result<()> {
        self.config = config;
        Self::save_config(&self.config_path, &self.config).await?;
        info!("Updated security configuration");
        Ok(())
    }

    /// Update a specific security policy setting
    pub async fn update_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        self.config.policy = policy;
        Self::save_config(&self.config_path, &self.config).await?;
        info!("Updated security policy");
        Ok(())
    }

    /// Update trust settings
    pub async fn update_trust_settings(&mut self, trust_settings: TrustSettings) -> Result<()> {
        self.config.trust_settings = trust_settings;
        Self::save_config(&self.config_path, &self.config).await?;
        info!("Updated trust settings");
        Ok(())
    }

    /// Enable or disable enterprise mode
    pub async fn set_enterprise_mode(&mut self, enabled: bool) -> Result<()> {
        self.config.enterprise.enabled = enabled;
        Self::save_config(&self.config_path, &self.config).await?;
        info!("Enterprise mode {}", if enabled { "enabled" } else { "disabled" });
        Ok(())
    }

    /// Validate the current configuration
    pub fn validate_config(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check if trust prompts are disabled (dangerous)
        if !self.config.policy.trust_prompts_enabled {
            warnings.push("Trust prompts are disabled - this is dangerous in production".to_string());
        }

        // Check if audit logging is disabled
        if !self.config.audit_config.enabled {
            warnings.push("Audit logging is disabled - security events will not be recorded".to_string());
        }

        // Check for overly permissive network settings
        if self.config.network.allowed_domains.is_empty() && self.config.network.filter_requests {
            warnings.push("Network filtering is enabled but no domains are allowed".to_string());
        }

        // Check resource limits
        if self.config.execution.resource_limits.max_memory_mb > 8192 {
            warnings.push("Memory limit is very high (>8GB) - consider reducing for security".to_string());
        }

        // Check for empty allowed commands
        if self.config.execution.allowed_commands.is_empty() {
            warnings.push("No commands are allowed for execution - functionality may be limited".to_string());
        }

        if warnings.is_empty() {
            info!("Security configuration validation passed");
        } else {
            for warning in &warnings {
                warn!("Security config warning: {}", warning);
            }
        }

        Ok(warnings)
    }

    /// Reset configuration to defaults
    pub async fn reset_to_defaults(&mut self) -> Result<()> {
        self.config = SecurityConfig::default();
        Self::save_config(&self.config_path, &self.config).await?;
        info!("Reset security configuration to defaults");
        Ok(())
    }

    /// Export configuration as JSON for backup/sharing
    pub fn export_as_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.config)
            .context("Failed to serialize config as JSON")
    }

    /// Import configuration from JSON
    pub async fn import_from_json(&mut self, json: &str) -> Result<()> {
        let config: SecurityConfig = serde_json::from_str(json)
            .context("Failed to parse JSON configuration")?;

        // Validate the imported configuration
        let temp_manager = SecurityConfigManager {
            config: config.clone(),
            config_path: self.config_path.clone(),
        };

        let warnings = temp_manager.validate_config()?;
        if !warnings.is_empty() {
            warn!("Imported configuration has warnings: {:?}", warnings);
        }

        self.config = config;
        Self::save_config(&self.config_path, &self.config).await?;
        info!("Imported security configuration from JSON");
        Ok(())
    }
}

/// Security configuration utilities
pub struct SecurityConfigUtils;

impl SecurityConfigUtils {
    /// Generate a secure default configuration for production use
    pub fn production_config() -> SecurityConfig {
        let mut config = SecurityConfig::default();

        // More restrictive settings for production
        config.policy.trust_prompts_enabled = true;
        config.policy.sandbox_mode = true;
        config.policy.max_file_size = 5 * 1024 * 1024; // 5MB

        config.trust_settings.require_confirmation = true;
        config.trust_settings.trust_expiry_hours = 24; // Expire after 24 hours

        config.network.require_https = true;
        config.network.validate_certificates = true;

        config.execution.enable_sandbox = true;
        config.execution.max_execution_time = 60; // 1 minute
        config.execution.resource_limits.max_memory_mb = 512; // 512MB

        config
    }

    /// Generate a development-friendly configuration
    pub fn development_config() -> SecurityConfig {
        let mut config = SecurityConfig::default();

        // More permissive settings for development
        config.policy.trust_prompts_enabled = true;
        config.policy.sandbox_mode = false;
        config.policy.max_file_size = 50 * 1024 * 1024; // 50MB

        config.trust_settings.require_confirmation = false;
        config.trust_settings.trust_expiry_hours = 0; // Never expire

        config.execution.enable_sandbox = false;
        config.execution.max_execution_time = 300; // 5 minutes
        config.execution.resource_limits.max_memory_mb = 2048; // 2GB

        config
    }

    /// Generate an enterprise configuration template
    pub fn enterprise_config() -> SecurityConfig {
        let mut config = Self::production_config();

        // Enterprise-specific settings
        config.enterprise.enabled = true;
        config.enterprise.require_mfa = true;
        config.enterprise.policy_refresh_minutes = 15;

        config.audit_config.include_content_hashes = true;
        config.audit_config.log_level = "debug".to_string();

        config.network.rate_limiting.max_requests_per_minute = 30;
        config.network.rate_limiting.max_requests_per_hour = 500;

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_security_config_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SecurityConfigManager::new(temp_dir.path()).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_config_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SecurityConfigManager::new(temp_dir.path()).await.unwrap();

        // Modify config
        let mut config = manager.get_config().clone();
        config.policy.max_file_size = 12345;
        manager.update_config(config).await.unwrap();

        // Create new manager to test persistence
        let manager2 = SecurityConfigManager::new(temp_dir.path()).await.unwrap();
        assert_eq!(manager2.get_config().policy.max_file_size, 12345);
    }

    #[test]
    fn test_config_validation() {
        let mut config = SecurityConfig::default();
        config.policy.trust_prompts_enabled = false;

        let manager = SecurityConfigManager {
            config,
            config_path: PathBuf::from("/tmp/test"),
        };

        let warnings = manager.validate_config().unwrap();
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("Trust prompts are disabled"));
    }

    #[test]
    fn test_production_config() {
        let config = SecurityConfigUtils::production_config();
        assert!(config.policy.trust_prompts_enabled);
        assert!(config.policy.sandbox_mode);
        assert!(config.execution.enable_sandbox);
        assert_eq!(config.trust_settings.trust_expiry_hours, 24);
    }

    #[test]
    fn test_development_config() {
        let config = SecurityConfigUtils::development_config();
        assert!(config.policy.trust_prompts_enabled);
        assert!(!config.policy.sandbox_mode);
        assert!(!config.execution.enable_sandbox);
        assert_eq!(config.trust_settings.trust_expiry_hours, 0);
    }
}