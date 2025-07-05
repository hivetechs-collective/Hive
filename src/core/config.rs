//! Configuration management for Hive AI
//!
//! This module provides comprehensive configuration management with
//! support for TOML files, environment variables, and runtime updates.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

/// Global configuration instance
static CONFIG: Lazy<RwLock<Option<HiveConfig>>> = Lazy::new(|| RwLock::new(None));

/// Type alias for compatibility with other modules
pub type Config = HiveConfig;

/// Complete Hive configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveConfig {
    pub consensus: ConsensusConfig,
    pub performance: PerformanceConfig,
    pub interface: InterfaceConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub openrouter: Option<OpenRouterConfig>,
    pub cloudflare: Option<CloudflareConfig>,
    pub license: Option<LicenseConfig>,
    pub core_dirs: CoreDirsConfig,
    pub analytics: AnalyticsConfig,
    pub database: DatabaseConfig,
    pub memory: MemoryConfig,
}

/// Core directories configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreDirsConfig {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub collection_enabled: bool,
    pub retention_days: u32,
    pub export_enabled: bool,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub connection_pool_size: usize,
    pub enable_wal: bool,
    pub auto_vacuum: bool,
}

/// Memory system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub max_conversations: usize,
    pub context_window_size: usize,
    pub clustering_enabled: bool,
    pub embedding_model: String,
}

/// Consensus engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub profile: String,
    pub models: ConsensusModels,
    pub streaming: StreamingConfig,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_seconds: u32,
}

/// Consensus model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusModels {
    pub generator: String,
    pub refiner: String,
    pub validator: String,
    pub curator: String,
}

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub enabled: bool,
    pub buffer_size: usize,
    pub chunk_delay_ms: u32,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub cache_size: String,
    pub max_workers: usize,
    pub incremental_parsing: bool,
    pub background_indexing: bool,
}

/// Interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    pub tui_mode: bool,
    pub prefer_tui: bool,
    pub tui: TuiConfig,
}

/// TUI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    pub theme: String,
    pub mouse_enabled: bool,
    pub show_line_numbers: bool,
    pub auto_completion: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub trust_mode: String,
    pub require_confirmation: bool,
    pub audit_logging: bool,
    pub telemetry: bool,
    pub enable_mfa: bool,
    pub session_timeout: u32,
    pub trust_dialog: TrustDialogConfig,
}

/// Trust dialog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustDialogConfig {
    pub enabled: bool,
    pub auto_trust_git: bool,
    pub trust_timeout: u32,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub format: String,
}

/// OpenRouter API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub timeout_seconds: u32,
    pub max_retries: u32,
}

/// Cloudflare D1 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    pub database_id: String,
    pub account_id: String,
    pub api_token: String,
    pub sync_enabled: bool,
}

/// License configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    pub key: Option<String>,
    pub email: Option<String>,
    pub tier: Option<String>,
}

impl Default for HiveConfig {
    fn default() -> Self {
        let hive_dir = get_hive_config_dir();
        
        Self {
            consensus: ConsensusConfig {
                profile: "balanced".to_string(),
                models: ConsensusModels {
                    generator: "claude-3-5-sonnet".to_string(),
                    refiner: "gpt-4-turbo".to_string(),
                    validator: "claude-3-opus".to_string(),
                    curator: "gpt-4o".to_string(),
                },
                streaming: StreamingConfig {
                    enabled: true,
                    buffer_size: 1024,
                    chunk_delay_ms: 50,
                },
                max_tokens: 4096,
                temperature: 0.7,
                timeout_seconds: 300,
            },
            performance: PerformanceConfig {
                cache_size: "256MB".to_string(),
                max_workers: num_cpus::get(),
                incremental_parsing: true,
                background_indexing: true,
            },
            interface: InterfaceConfig {
                tui_mode: true,
                prefer_tui: true,
                tui: TuiConfig {
                    theme: "dark".to_string(),
                    mouse_enabled: true,
                    show_line_numbers: true,
                    auto_completion: true,
                },
            },
            security: SecurityConfig {
                trust_mode: "explicit".to_string(),
                require_confirmation: true,
                audit_logging: true,
                telemetry: false,
                enable_mfa: false,
                session_timeout: 3600,
                trust_dialog: TrustDialogConfig {
                    enabled: true,
                    auto_trust_git: true,
                    trust_timeout: 86400,
                },
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: None,
                format: "pretty".to_string(),
            },
            openrouter: None,
            cloudflare: None,
            license: None,
            core_dirs: CoreDirsConfig {
                data_dir: hive_dir.join("data"),
                config_dir: hive_dir.clone(),
                cache_dir: hive_dir.join("cache"),
            },
            analytics: AnalyticsConfig {
                collection_enabled: true,
                retention_days: 90,
                export_enabled: true,
            },
            database: DatabaseConfig {
                path: hive_dir.join("data").join("hive.db"),
                connection_pool_size: 10,
                enable_wal: true,
                auto_vacuum: true,
            },
            memory: MemoryConfig {
                max_conversations: 1000,
                context_window_size: 4096,
                clustering_enabled: true,
                embedding_model: "text-embedding-ada-002".to_string(),
            },
        }
    }
}

/// Get the Hive configuration directory
pub fn get_hive_config_dir() -> PathBuf {
    if let Ok(hive_home) = std::env::var("HIVE_HOME") {
        PathBuf::from(hive_home)
    } else if let Some(home) = dirs::home_dir() {
        home.join(".hive")
    } else {
        PathBuf::from(".hive")
    }
}

/// Load configuration from file or create default
pub async fn load_config() -> Result<HiveConfig> {
    let config_path = get_hive_config_dir().join("config.toml");
    
    if config_path.exists() {
        let contents = fs::read_to_string(&config_path).await?;
        let config: HiveConfig = toml::from_str(&contents)?;
        
        // Store in global state
        let mut global = CONFIG.write().await;
        *global = Some(config.clone());
        
        Ok(config)
    } else {
        // Create default config
        let config = HiveConfig::default();
        
        // Store in global state
        let mut global = CONFIG.write().await;
        *global = Some(config.clone());
        
        Ok(config)
    }
}

/// Get the current configuration
pub async fn get_config() -> Result<HiveConfig> {
    let guard = CONFIG.read().await;
    if let Some(ref config) = *guard {
        Ok(config.clone())
    } else {
        drop(guard);
        load_config().await
    }
}

/// Set a configuration value using dot notation
pub async fn set_config_value(key: &str, value: &str) -> Result<()> {
    let mut config = get_config().await?;
    
    // Parse the key and update the value
    match key {
        "consensus.profile" => config.consensus.profile = value.to_string(),
        "consensus.models.generator" => config.consensus.models.generator = value.to_string(),
        "consensus.models.refiner" => config.consensus.models.refiner = value.to_string(),
        "consensus.models.validator" => config.consensus.models.validator = value.to_string(),
        "consensus.models.curator" => config.consensus.models.curator = value.to_string(),
        "consensus.streaming.enabled" => config.consensus.streaming.enabled = value.parse()?,
        "consensus.max_tokens" => config.consensus.max_tokens = value.parse()?,
        "consensus.temperature" => config.consensus.temperature = value.parse()?,
        
        "performance.cache_size" => config.performance.cache_size = value.to_string(),
        "performance.max_workers" => config.performance.max_workers = value.parse()?,
        "performance.incremental_parsing" => config.performance.incremental_parsing = value.parse()?,
        "performance.background_indexing" => config.performance.background_indexing = value.parse()?,
        
        "interface.tui_mode" => config.interface.tui_mode = value.parse()?,
        "interface.prefer_tui" => config.interface.prefer_tui = value.parse()?,
        "interface.tui.theme" => config.interface.tui.theme = value.to_string(),
        "interface.tui.mouse_enabled" => config.interface.tui.mouse_enabled = value.parse()?,
        
        "security.trust_mode" => config.security.trust_mode = value.to_string(),
        "security.require_confirmation" => config.security.require_confirmation = value.parse()?,
        "security.audit_logging" => config.security.audit_logging = value.parse()?,
        "security.telemetry" => config.security.telemetry = value.parse()?,
        "security.enable_mfa" => config.security.enable_mfa = value.parse()?,
        "security.session_timeout" => config.security.session_timeout = value.parse()?,
        "security.trust_dialog.enabled" => config.security.trust_dialog.enabled = value.parse()?,
        "security.trust_dialog.auto_trust_git" => config.security.trust_dialog.auto_trust_git = value.parse()?,
        "security.trust_dialog.trust_timeout" => config.security.trust_dialog.trust_timeout = value.parse()?,
        
        "logging.level" => config.logging.level = value.to_string(),
        "logging.format" => config.logging.format = value.to_string(),
        
        // Handle Analytics config
        "analytics.collection_enabled" => config.analytics.collection_enabled = value.parse()?,
        "analytics.retention_days" => config.analytics.retention_days = value.parse()?,
        "analytics.export_enabled" => config.analytics.export_enabled = value.parse()?,
        
        // Handle Database config
        "database.connection_pool_size" => config.database.connection_pool_size = value.parse()?,
        "database.enable_wal" => config.database.enable_wal = value.parse()?,
        "database.auto_vacuum" => config.database.auto_vacuum = value.parse()?,
        
        // Handle Memory config
        "memory.max_conversations" => config.memory.max_conversations = value.parse()?,
        "memory.context_window_size" => config.memory.context_window_size = value.parse()?,
        "memory.clustering_enabled" => config.memory.clustering_enabled = value.parse()?,
        "memory.embedding_model" => config.memory.embedding_model = value.to_string(),
        
        // Handle OpenRouter config
        key if key.starts_with("openrouter.") => {
            if config.openrouter.is_none() {
                config.openrouter = Some(OpenRouterConfig {
                    api_key: None,
                    base_url: "https://openrouter.ai/api/v1".to_string(),
                    timeout_seconds: 30,
                    max_retries: 3,
                });
            }
            
            if let Some(ref mut openrouter) = config.openrouter {
                match key {
                    "openrouter.api_key" => openrouter.api_key = Some(value.to_string()),
                    "openrouter.base_url" => openrouter.base_url = value.to_string(),
                    "openrouter.timeout_seconds" => openrouter.timeout_seconds = value.parse()?,
                    "openrouter.max_retries" => openrouter.max_retries = value.parse()?,
                    _ => return Err(anyhow!("Unknown configuration key: {}", key)),
                }
            }
        }
        
        // Handle License config
        key if key.starts_with("license.") => {
            if config.license.is_none() {
                config.license = Some(LicenseConfig {
                    key: None,
                    email: None,
                    tier: None,
                });
            }
            
            if let Some(ref mut license) = config.license {
                match key {
                    "license.key" => license.key = Some(value.to_string()),
                    "license.email" => license.email = Some(value.to_string()),
                    "license.tier" => license.tier = Some(value.to_string()),
                    _ => return Err(anyhow!("Unknown configuration key: {}", key)),
                }
            }
        }
        
        _ => return Err(anyhow!("Unknown configuration key: {}", key)),
    }
    
    // Update global state
    let mut global = CONFIG.write().await;
    *global = Some(config.clone());
    
    // Save to file
    save_config(&config).await?;
    
    Ok(())
}

/// Get a configuration value using dot notation
pub async fn get_config_value(key: &str) -> Result<String> {
    let config = get_config().await?;
    
    let value = match key {
        "consensus.profile" => config.consensus.profile,
        "consensus.models.generator" => config.consensus.models.generator,
        "consensus.models.refiner" => config.consensus.models.refiner,
        "consensus.models.validator" => config.consensus.models.validator,
        "consensus.models.curator" => config.consensus.models.curator,
        "consensus.streaming.enabled" => config.consensus.streaming.enabled.to_string(),
        "consensus.max_tokens" => config.consensus.max_tokens.to_string(),
        "consensus.temperature" => config.consensus.temperature.to_string(),
        
        "performance.cache_size" => config.performance.cache_size,
        "performance.max_workers" => config.performance.max_workers.to_string(),
        "performance.incremental_parsing" => config.performance.incremental_parsing.to_string(),
        "performance.background_indexing" => config.performance.background_indexing.to_string(),
        
        "interface.tui_mode" => config.interface.tui_mode.to_string(),
        "interface.prefer_tui" => config.interface.prefer_tui.to_string(),
        "interface.tui.theme" => config.interface.tui.theme,
        "interface.tui.mouse_enabled" => config.interface.tui.mouse_enabled.to_string(),
        
        "security.trust_mode" => config.security.trust_mode,
        "security.require_confirmation" => config.security.require_confirmation.to_string(),
        "security.audit_logging" => config.security.audit_logging.to_string(),
        "security.telemetry" => config.security.telemetry.to_string(),
        "security.enable_mfa" => config.security.enable_mfa.to_string(),
        "security.session_timeout" => config.security.session_timeout.to_string(),
        "security.trust_dialog.enabled" => config.security.trust_dialog.enabled.to_string(),
        "security.trust_dialog.auto_trust_git" => config.security.trust_dialog.auto_trust_git.to_string(),
        "security.trust_dialog.trust_timeout" => config.security.trust_dialog.trust_timeout.to_string(),
        
        "logging.level" => config.logging.level,
        "logging.format" => config.logging.format,
        
        // Handle Analytics config
        "analytics.collection_enabled" => config.analytics.collection_enabled.to_string(),
        "analytics.retention_days" => config.analytics.retention_days.to_string(),
        "analytics.export_enabled" => config.analytics.export_enabled.to_string(),
        
        // Handle Database config  
        "database.connection_pool_size" => config.database.connection_pool_size.to_string(),
        "database.enable_wal" => config.database.enable_wal.to_string(),
        "database.auto_vacuum" => config.database.auto_vacuum.to_string(),
        
        // Handle Memory config
        "memory.max_conversations" => config.memory.max_conversations.to_string(),
        "memory.context_window_size" => config.memory.context_window_size.to_string(),
        "memory.clustering_enabled" => config.memory.clustering_enabled.to_string(),
        "memory.embedding_model" => config.memory.embedding_model,
        
        // Handle Core dirs config
        "core_dirs.data_dir" => config.core_dirs.data_dir.display().to_string(),
        "core_dirs.config_dir" => config.core_dirs.config_dir.display().to_string(),
        "core_dirs.cache_dir" => config.core_dirs.cache_dir.display().to_string(),
        
        // Handle OpenRouter config
        "openrouter.api_key" => config.openrouter.as_ref().and_then(|o| o.api_key.clone()).unwrap_or_default(),
        "openrouter.base_url" => config.openrouter.as_ref().map(|o| o.base_url.clone()).unwrap_or_default(),
        "openrouter.timeout_seconds" => config.openrouter.as_ref().map(|o| o.timeout_seconds.to_string()).unwrap_or_default(),
        "openrouter.max_retries" => config.openrouter.as_ref().map(|o| o.max_retries.to_string()).unwrap_or_default(),
        
        // Handle License config
        "license.key" => config.license.as_ref().and_then(|l| l.key.clone()).unwrap_or_default(),
        "license.email" => config.license.as_ref().and_then(|l| l.email.clone()).unwrap_or_default(),
        "license.tier" => config.license.as_ref().and_then(|l| l.tier.clone()).unwrap_or_default(),
        
        _ => return Err(anyhow!("Unknown configuration key: {}", key)),
    };
    
    Ok(value)
}

/// Reset configuration to defaults
pub async fn reset_config() -> Result<()> {
    let config = HiveConfig::default();
    
    // Update global state
    let mut global = CONFIG.write().await;
    *global = Some(config.clone());
    
    // Save to file
    save_config(&config).await?;
    
    Ok(())
}

/// Save configuration to file
pub async fn save_config(config: &HiveConfig) -> Result<()> {
    let config_dir = get_hive_config_dir();
    fs::create_dir_all(&config_dir).await?;
    
    let config_path = config_dir.join("config.toml");
    let toml_str = toml::to_string_pretty(config)?;
    fs::write(&config_path, toml_str).await?;
    
    Ok(())
}

/// Create default configuration file
pub async fn create_default_config() -> Result<()> {
    let config = HiveConfig::default();
    save_config(&config).await
}

/// Initialize configuration system
pub async fn init_config() -> Result<()> {
    // Ensure config directory exists
    let config_dir = get_hive_config_dir();
    fs::create_dir_all(&config_dir).await?;
    
    // Load or create config
    let _ = load_config().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_default_config() {
        let config = HiveConfig::default();
        assert_eq!(config.consensus.profile, "balanced");
        assert_eq!(config.interface.tui_mode, true);
        assert_eq!(config.security.telemetry, false);
    }
    
    #[tokio::test]
    async fn test_config_value_access() {
        let config = HiveConfig::default();
        
        // Store in global state for testing
        let mut global = CONFIG.write().await;
        *global = Some(config);
        drop(global);
        
        // Test getting values
        let profile = get_config_value("consensus.profile").await.unwrap();
        assert_eq!(profile, "balanced");
        
        let tui_mode = get_config_value("interface.tui_mode").await.unwrap();
        assert_eq!(tui_mode, "true");
    }
    
    #[test]
    fn test_config_dir() {
        let config_dir = get_hive_config_dir();
        assert!(config_dir.to_str().unwrap().contains(".hive"));
    }
}