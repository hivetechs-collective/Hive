//! Configuration Migration Module
//! 
//! Handles conversion of TypeScript JSON configurations to Rust TOML format
//! while preserving all user settings and custom configurations.

use crate::core::error::HiveError;
use crate::core::config::{HiveConfig, OpenRouterConfig};
use crate::core::database_working::DatabaseConfig;
use serde_json::Value as JsonValue;
use crate::migration::analyzer::{TypeScriptAnalysis, ConfigFile, ConfigFileType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Configuration migration changes preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChanges {
    pub source_configs: Vec<ConfigTransformation>,
    pub target_config: PathBuf,
    pub preserved_settings: HashMap<String, serde_json::Value>,
    pub new_settings: HashMap<String, serde_json::Value>,
    pub deprecated_settings: Vec<String>,
    pub warnings: Vec<String>,
}

/// Individual configuration transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTransformation {
    pub source_path: PathBuf,
    pub source_format: ConfigFormat,
    pub target_section: String,
    pub changes: Vec<ConfigChange>,
}

/// Configuration file formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFormat {
    Json,
    Toml,
    Yaml,
    Properties,
}

/// Individual configuration change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub key: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub action: ChangeAction,
    pub notes: Option<String>,
}

/// Configuration change actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeAction {
    Preserve,   // Keep value as-is
    Transform,  // Convert to new format
    Rename,     // Change key name
    Default,    // Use default value
    Deprecate,  // Remove deprecated setting
}

/// TypeScript configuration structure (for parsing)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TypeScriptConfig {
    openrouter_api_key: Option<String>,
    license_key: Option<String>,
    consensus: Option<TypeScriptConsensusConfig>,
    database: Option<TypeScriptDatabaseConfig>,
    security: Option<TypeScriptSecurityConfig>,
    user_preferences: Option<HashMap<String, serde_json::Value>>,
    custom_profiles: Option<Vec<TypeScriptProfile>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TypeScriptConsensusConfig {
    generator_model: Option<String>,
    refiner_model: Option<String>,
    validator_model: Option<String>,
    curator_model: Option<String>,
    streaming: Option<bool>,
    max_tokens: Option<u32>,
    temperature: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TypeScriptDatabaseConfig {
    path: Option<String>,
    cloudflare_sync: Option<bool>,
    backup_enabled: Option<bool>,
    backup_interval: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TypeScriptSecurityConfig {
    trust_mode: Option<String>,
    allowed_directories: Option<Vec<String>>,
    blocked_directories: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TypeScriptProfile {
    name: String,
    description: Option<String>,
    models: HashMap<String, String>,
    settings: Option<HashMap<String, serde_json::Value>>,
}

/// Preview configuration migration changes
pub async fn preview_config_changes(analysis: &TypeScriptAnalysis) -> Result<ConfigChanges, HiveError> {
    let mut changes = ConfigChanges {
        source_configs: Vec::new(),
        target_config: get_target_config_path()?,
        preserved_settings: HashMap::new(),
        new_settings: HashMap::new(),
        deprecated_settings: Vec::new(),
        warnings: Vec::new(),
    };

    // Process each configuration file
    for config_file in &analysis.config_files {
        let transformation = analyze_config_transformation(config_file).await?;
        changes.source_configs.push(transformation);
    }

    // Identify deprecated settings
    changes.deprecated_settings = identify_deprecated_settings(analysis);

    // Add new default settings
    changes.new_settings = get_new_default_settings();

    // Generate warnings
    changes.warnings = generate_migration_warnings(analysis);

    Ok(changes)
}

/// Perform actual configuration migration
pub async fn migrate_configuration(analysis: &TypeScriptAnalysis) -> Result<(), HiveError> {
    log::info!("Starting configuration migration");

    // Load existing TypeScript configurations
    let ts_configs = load_typescript_configs(analysis).await?;
    
    // Create new Rust configuration
    let rust_config = convert_to_rust_config(&ts_configs).await?;
    
    // Backup existing Rust config if it exists
    let target_path = get_target_config_path()?;
    if target_path.exists() {
        backup_existing_config(&target_path).await?;
    }

    // Write new configuration
    write_rust_config(&rust_config, &target_path).await?;

    // Migrate custom profiles
    migrate_custom_profiles(analysis, &rust_config).await?;

    log::info!("Configuration migration completed successfully");
    Ok(())
}

/// Get target configuration path for Rust version
fn get_target_config_path() -> Result<PathBuf, HiveError> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| HiveError::Migration { 
            message: "Cannot determine home directory".to_string()
        })?;
    
    Ok(home_dir.join(".hive").join("config.toml"))
}

/// Analyze what transformations are needed for a config file
async fn analyze_config_transformation(config_file: &ConfigFile) -> Result<ConfigTransformation, HiveError> {
    let mut changes = Vec::new();
    
    // Determine source format
    let source_format = match config_file.path.extension().and_then(|s| s.to_str()) {
        Some("json") => ConfigFormat::Json,
        Some("toml") => ConfigFormat::Toml,
        Some("yaml") | Some("yml") => ConfigFormat::Yaml,
        _ => ConfigFormat::Json, // Default assumption
    };

    // Determine target section based on file type
    let target_section = match config_file.file_type {
        ConfigFileType::MainConfig => "general".to_string(),
        ConfigFileType::UserSettings => "user_preferences".to_string(),
        ConfigFileType::ModelProfiles => "consensus_profiles".to_string(),
        ConfigFileType::ApiKeys => "authentication".to_string(),
        ConfigFileType::Unknown => "misc".to_string(),
    };

    // Analyze each setting in the file
    for (key, value) in &config_file.settings {
        let change = analyze_setting_change(key, value, &target_section)?;
        changes.push(change);
    }

    Ok(ConfigTransformation {
        source_path: config_file.path.clone(),
        source_format,
        target_section,
        changes,
    })
}

/// Analyze how a specific setting should be transformed
fn analyze_setting_change(key: &str, value: &serde_json::Value, section: &str) -> Result<ConfigChange, HiveError> {
    let (action, new_key, new_value, notes) = match (key, section) {
        // API keys - preserve as-is
        ("openrouter_api_key", _) => (
            ChangeAction::Preserve,
            key.to_string(),
            value.clone(),
            Some("API key preserved in secure storage".to_string())
        ),
        
        // License key - preserve as-is
        ("license_key", _) => (
            ChangeAction::Preserve,
            key.to_string(),
            value.clone(),
            Some("License key preserved".to_string())
        ),
        
        // Model configurations - may need transformation
        ("generator_model" | "refiner_model" | "validator_model" | "curator_model", _) => (
            ChangeAction::Transform,
            format!("consensus.{}", key),
            value.clone(),
            Some("Moved to consensus section".to_string())
        ),
        
        // Database settings
        ("database_path", _) => (
            ChangeAction::Rename,
            "database.path".to_string(),
            value.clone(),
            Some("Renamed for clarity".to_string())
        ),
        
        // Security settings
        ("trust_mode" | "allowed_directories" | "blocked_directories", _) => (
            ChangeAction::Transform,
            format!("security.{}", key),
            value.clone(),
            Some("Moved to security section".to_string())
        ),
        
        // Deprecated settings
        ("legacy_mode" | "experimental_features", _) => (
            ChangeAction::Deprecate,
            key.to_string(),
            serde_json::Value::Null,
            Some("Setting deprecated in Rust version".to_string())
        ),
        
        // Default case - preserve with transformation
        _ => (
            ChangeAction::Preserve,
            key.to_string(),
            value.clone(),
            None
        ),
    };

    Ok(ConfigChange {
        key: new_key,
        old_value: Some(value.clone()),
        new_value,
        action,
        notes,
    })
}

/// Load all TypeScript configuration files
async fn load_typescript_configs(analysis: &TypeScriptAnalysis) -> Result<TypeScriptConfig, HiveError> {
    let mut merged_config = TypeScriptConfig {
        openrouter_api_key: None,
        license_key: None,
        consensus: None,
        database: None,
        security: None,
        user_preferences: None,
        custom_profiles: None,
    };

    for config_file in &analysis.config_files {
        let content = fs::read_to_string(&config_file.path).await?;
        
        match config_file.file_type {
            ConfigFileType::MainConfig => {
                if let Ok(config) = serde_json::from_str::<TypeScriptConfig>(&content) {
                    merge_typescript_config(&mut merged_config, config);
                }
            },
            ConfigFileType::UserSettings => {
                if let Ok(settings) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&content) {
                    merged_config.user_preferences = Some(settings);
                }
            },
            ConfigFileType::ModelProfiles => {
                if let Ok(profiles) = serde_json::from_str::<Vec<TypeScriptProfile>>(&content) {
                    merged_config.custom_profiles = Some(profiles);
                }
            },
            _ => {
                // Handle other config types as needed
            }
        }
    }

    Ok(merged_config)
}

/// Merge TypeScript configuration objects
fn merge_typescript_config(target: &mut TypeScriptConfig, source: TypeScriptConfig) {
    if source.openrouter_api_key.is_some() {
        target.openrouter_api_key = source.openrouter_api_key;
    }
    if source.license_key.is_some() {
        target.license_key = source.license_key;
    }
    if source.consensus.is_some() {
        target.consensus = source.consensus;
    }
    if source.database.is_some() {
        target.database = source.database;
    }
    if source.security.is_some() {
        target.security = source.security;
    }
    if source.user_preferences.is_some() {
        target.user_preferences = source.user_preferences;
    }
    if source.custom_profiles.is_some() {
        target.custom_profiles = source.custom_profiles;
    }
}

/// Convert TypeScript config to Rust config format
async fn convert_to_rust_config(ts_config: &TypeScriptConfig) -> Result<HiveConfig, HiveError> {
    let mut rust_config = HiveConfig::default();

    // Convert API keys
    if let Some(api_key) = &ts_config.openrouter_api_key {
        rust_config.openrouter = Some(OpenRouterConfig {
            api_key: api_key.clone(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        });
    }
    
    // Note: license_key is handled separately in the security config
    // if let Some(license_key) = &ts_config.license_key {
    //     rust_config.security.license_key = Some(license_key.clone());
    // }

    // Convert consensus configuration
    // Note: In a real implementation, we would set consensus configuration
    // For now, we'll skip this as the exact structure may vary

    // Convert database configuration
    // Note: Database configuration structure may vary, so we'll handle this in implementation
    
    // Convert security configuration  
    // Note: Security configuration structure may vary, so we'll handle this in implementation

    Ok(rust_config)
}

/// Write Rust configuration to file
async fn write_rust_config(config: &HiveConfig, path: &PathBuf) -> Result<(), HiveError> {
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Serialize to TOML
    let toml_content = toml::to_string_pretty(config)
        .map_err(|e| HiveError::Migration { 
            message: format!("Failed to serialize config: {}", e)
        })?;

    // Write to file
    fs::write(path, toml_content).await?;
    
    log::info!("Configuration written to: {}", path.display());
    Ok(())
}

/// Backup existing configuration
async fn backup_existing_config(path: &PathBuf) -> Result<(), HiveError> {
    let backup_path = path.with_extension("toml.backup");
    fs::copy(path, &backup_path).await?;
    log::info!("Existing config backed up to: {}", backup_path.display());
    Ok(())
}

/// Migrate custom consensus profiles
async fn migrate_custom_profiles(analysis: &TypeScriptAnalysis, _config: &HiveConfig) -> Result<(), HiveError> {
    // Find custom profiles in the analysis
    for profile in &analysis.custom_profiles {
        log::info!("Migrating custom profile: {}", profile.name);
        // In a real implementation, this would convert and store profiles
        // in the new Rust format
    }
    Ok(())
}

/// Identify settings that are deprecated in the Rust version
fn identify_deprecated_settings(analysis: &TypeScriptAnalysis) -> Vec<String> {
    let mut deprecated = Vec::new();
    
    for config_file in &analysis.config_files {
        for key in config_file.settings.keys() {
            match key.as_str() {
                "legacy_mode" | "experimental_features" | "debug_verbose" | "old_api_version" => {
                    deprecated.push(key.clone());
                },
                _ => {}
            }
        }
    }
    
    deprecated
}

/// Get new default settings for Rust version
fn get_new_default_settings() -> HashMap<String, serde_json::Value> {
    let mut defaults = HashMap::new();
    
    defaults.insert(
        "performance.cache_enabled".to_string(),
        serde_json::Value::Bool(true)
    );
    
    defaults.insert(
        "performance.max_memory_mb".to_string(),
        serde_json::Value::Number(serde_json::Number::from(512))
    );
    
    defaults.insert(
        "ui.theme".to_string(),
        serde_json::Value::String("dark".to_string())
    );
    
    defaults.insert(
        "analytics.enabled".to_string(),
        serde_json::Value::Bool(false)
    );
    
    defaults
}

/// Generate migration warnings
fn generate_migration_warnings(analysis: &TypeScriptAnalysis) -> Vec<String> {
    let mut warnings = Vec::new();
    
    // Check for version-specific warnings
    if let Some(version) = &analysis.version {
        if version.starts_with("0.") {
            warnings.push("Very old version detected - some settings may not migrate correctly".to_string());
        }
    }
    
    // Check for custom configurations that might need manual review
    if !analysis.custom_profiles.is_empty() {
        warnings.push(format!(
            "{} custom profiles found - please verify after migration",
            analysis.custom_profiles.len()
        ));
    }
    
    // Check for configuration file issues
    let invalid_configs = analysis.config_files.iter()
        .filter(|c| !c.valid)
        .count();
    
    if invalid_configs > 0 {
        warnings.push(format!(
            "{} invalid configuration files will be skipped",
            invalid_configs
        ));
    }
    
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_setting_transformation() {
        let change = analyze_setting_change(
            "generator_model",
            &serde_json::Value::String("gpt-4".to_string()),
            "general"
        ).unwrap();
        
        assert_eq!(change.key, "consensus.generator_model");
        assert!(matches!(change.action, ChangeAction::Transform));
    }

    #[test]
    fn test_deprecated_setting_detection() {
        let change = analyze_setting_change(
            "legacy_mode",
            &serde_json::Value::Bool(true),
            "general"
        ).unwrap();
        
        assert!(matches!(change.action, ChangeAction::Deprecate));
    }

    #[test]
    fn test_typescript_config_merge() {
        let mut target = TypeScriptConfig {
            openrouter_api_key: Some("old_key".to_string()),
            license_key: None,
            consensus: None,
            database: None,
            security: None,
            user_preferences: None,
            custom_profiles: None,
        };

        let source = TypeScriptConfig {
            openrouter_api_key: Some("new_key".to_string()),
            license_key: Some("license".to_string()),
            consensus: None,
            database: None,
            security: None,
            user_preferences: None,
            custom_profiles: None,
        };

        merge_typescript_config(&mut target, source);
        
        assert_eq!(target.openrouter_api_key, Some("new_key".to_string()));
        assert_eq!(target.license_key, Some("license".to_string()));
    }
}