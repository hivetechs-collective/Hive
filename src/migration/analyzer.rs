//! TypeScript Installation Analyzer
//! 
//! Analyzes existing TypeScript Hive AI installations to understand
//! configuration, database structure, and usage patterns for migration.

use crate::core::error::HiveError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Complete analysis of TypeScript installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeScriptAnalysis {
    pub installation_path: PathBuf,
    pub version: Option<String>,
    pub config_files: Vec<ConfigFile>,
    pub database_info: DatabaseInfo,
    pub usage_stats: UsageStats,
    pub custom_profiles: Vec<ConsensusProfile>,
    pub issues: Vec<AnalysisIssue>,
    pub compatibility: CompatibilityAssessment,
}

/// Configuration file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub file_type: ConfigFileType,
    pub size: u64,
    pub valid: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Configuration file types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFileType {
    MainConfig,     // ~/.hive/config.json
    UserSettings,   // User-specific settings
    ModelProfiles,  // Custom model configurations
    ApiKeys,        // API key storage
    Unknown,
}

/// Database analysis information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub path: PathBuf,
    pub size: u64,
    pub conversation_count: u64,
    pub message_count: u64,
    pub schema_version: Option<String>,
    pub last_backup: Option<chrono::DateTime<chrono::Utc>>,
    pub integrity_check: bool,
    pub clusters: Vec<ThematicCluster>,
}

/// Thematic conversation clusters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThematicCluster {
    pub id: String,
    pub theme: String,
    pub conversation_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_conversations: u64,
    pub average_messages_per_conversation: f64,
    pub most_used_models: Vec<ModelUsage>,
    pub daily_usage_pattern: HashMap<String, u32>,
    pub feature_usage: HashMap<String, u32>,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Model usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model_name: String,
    pub usage_count: u32,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub average_cost: f64,
}

/// Custom consensus profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProfile {
    pub name: String,
    pub description: String,
    pub models: HashMap<String, String>, // stage -> model
    pub custom_prompts: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: u32,
}

/// Analysis issues found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub recommendation: String,
    pub blocking: bool,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Issue categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Configuration,
    Database,
    Permissions,
    Dependencies,
    Compatibility,
    Performance,
}

/// Compatibility assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityAssessment {
    pub overall_score: f32, // 0.0 to 1.0
    pub config_compatibility: f32,
    pub database_compatibility: f32,
    pub feature_compatibility: f32,
    pub migration_complexity: MigrationComplexity,
    pub recommended_approach: String,
}

/// Migration complexity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationComplexity {
    Simple,     // Direct migration possible
    Moderate,   // Some manual intervention needed
    Complex,    // Significant customization required
    Expert,     // Requires expert knowledge
}

impl TypeScriptAnalysis {
    /// Check if analysis found critical issues
    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(|issue| {
            matches!(issue.severity, IssueSeverity::Critical) && issue.blocking
        })
    }

    /// Get migration readiness score
    pub fn migration_readiness(&self) -> f32 {
        let mut score = self.compatibility.overall_score;
        
        // Penalize for critical issues
        let critical_issues = self.issues.iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Critical))
            .count() as f32;
        score -= critical_issues * 0.2;
        
        // Penalize for database integrity issues
        if !self.database_info.integrity_check {
            score -= 0.3;
        }
        
        score.max(0.0).min(1.0)
    }

    /// Get summary of analysis
    pub fn summary(&self) -> String {
        format!(
            "TypeScript Hive AI v{} with {} conversations, {} messages. Compatibility: {:.1}%, Readiness: {:.1}%",
            self.version.as_deref().unwrap_or("unknown"),
            self.database_info.conversation_count,
            self.database_info.message_count,
            self.compatibility.overall_score * 100.0,
            self.migration_readiness() * 100.0
        )
    }
}

/// Analyze TypeScript installation at given path
pub async fn analyze_typescript_installation(path: &PathBuf) -> Result<TypeScriptAnalysis, HiveError> {
    log::info!("Analyzing TypeScript installation at: {}", path.display());
    
    // Check if path exists
    if !path.exists() {
        return Err(HiveError::Migration { 
            message: format!("TypeScript installation not found at: {}", path.display())
        });
    }

    let mut analysis = TypeScriptAnalysis {
        installation_path: path.clone(),
        version: None,
        config_files: Vec::new(),
        database_info: DatabaseInfo {
            path: PathBuf::new(),
            size: 0,
            conversation_count: 0,
            message_count: 0,
            schema_version: None,
            last_backup: None,
            integrity_check: false,
            clusters: Vec::new(),
        },
        usage_stats: UsageStats {
            total_conversations: 0,
            average_messages_per_conversation: 0.0,
            most_used_models: Vec::new(),
            daily_usage_pattern: HashMap::new(),
            feature_usage: HashMap::new(),
            last_activity: None,
        },
        custom_profiles: Vec::new(),
        issues: Vec::new(),
        compatibility: CompatibilityAssessment {
            overall_score: 1.0,
            config_compatibility: 1.0,
            database_compatibility: 1.0,
            feature_compatibility: 1.0,
            migration_complexity: MigrationComplexity::Simple,
            recommended_approach: "Direct migration".to_string(),
        },
    };

    // Analyze package.json for version
    if let Ok(version) = analyze_package_version(path).await {
        analysis.version = Some(version);
    }

    // Find and analyze configuration files
    analysis.config_files = find_config_files(path).await?;
    
    // Analyze database
    if let Ok(db_info) = analyze_database(path).await {
        analysis.database_info = db_info;
    }

    // Analyze usage patterns
    if let Ok(usage) = analyze_usage_stats(&analysis.database_info).await {
        analysis.usage_stats = usage;
    }

    // Find custom profiles
    analysis.custom_profiles = find_custom_profiles(&analysis.config_files).await?;

    // Perform compatibility assessment
    analysis.compatibility = assess_compatibility(&analysis).await?;

    // Check for issues
    analysis.issues = find_issues(&analysis).await?;

    log::info!("Analysis complete: {}", analysis.summary());
    Ok(analysis)
}

/// Analyze package.json to determine version
async fn analyze_package_version(path: &PathBuf) -> Result<String, HiveError> {
    let package_path = path.join("package.json");
    
    if !package_path.exists() {
        // Try node_modules
        let node_modules_path = path.join("node_modules/@hivetechs/hive-ai/package.json");
        if node_modules_path.exists() {
            return Box::pin(analyze_package_version(&path.join("node_modules/@hivetechs/hive-ai"))).await;
        }
        return Err(HiveError::Migration { 
            message: "package.json not found".to_string()
        });
    }

    let content = fs::read_to_string(&package_path).await?;
    let package: serde_json::Value = serde_json::from_str(&content)?;
    
    if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
        Ok(version.to_string())
    } else {
        Err(HiveError::Migration { 
            message: "Version not found in package.json".to_string()
        })
    }
}

/// Find all configuration files
async fn find_config_files(path: &PathBuf) -> Result<Vec<ConfigFile>, HiveError> {
    let mut config_files = Vec::new();
    
    // Standard locations to check
    let locations = vec![
        (path.join(".hive"), ConfigFileType::MainConfig),
        (dirs::home_dir().unwrap_or_default().join(".hive"), ConfigFileType::MainConfig),
        (path.join("config"), ConfigFileType::UserSettings),
        (path.join("profiles"), ConfigFileType::ModelProfiles),
    ];

    for (location, file_type) in locations {
        if location.exists() {
            if let Ok(files) = scan_directory_for_configs(&location, file_type.clone()).await {
                config_files.extend(files);
            }
        }
    }

    Ok(config_files)
}

/// Scan directory for configuration files
async fn scan_directory_for_configs(dir: &PathBuf, file_type: ConfigFileType) -> Result<Vec<ConfigFile>, HiveError> {
    let mut configs = Vec::new();
    let mut entries = fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" || ext == "toml" || ext == "yaml" {
                    if let Ok(metadata) = entry.metadata().await {
                        let config = ConfigFile {
                            path: path.clone(),
                            file_type: file_type.clone(),
                            size: metadata.len(),
                            valid: validate_config_file(&path).await,
                            settings: load_config_settings(&path).await.unwrap_or_default(),
                        };
                        configs.push(config);
                    }
                }
            }
        }
    }

    Ok(configs)
}

/// Validate configuration file
async fn validate_config_file(path: &PathBuf) -> bool {
    if let Ok(content) = fs::read_to_string(path).await {
        // Try parsing as JSON first
        if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
            return true;
        }
        
        // Try TOML if JSON fails
        if path.extension().map_or(false, |ext| ext == "toml") {
            return toml::from_str::<toml::Value>(&content).is_ok();
        }
    }
    false
}

/// Load configuration settings
async fn load_config_settings(path: &PathBuf) -> Result<HashMap<String, serde_json::Value>, HiveError> {
    let content = fs::read_to_string(path).await?;
    
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(obj) = json.as_object() {
            return Ok(obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
        }
    }
    
    Ok(HashMap::new())
}

/// Analyze database information
async fn analyze_database(path: &PathBuf) -> Result<DatabaseInfo, HiveError> {
    // Look for SQLite database files
    let potential_db_paths = vec![
        path.join(".hive/hive-ai.db"),
        dirs::home_dir().unwrap_or_default().join(".hive/hive-ai.db"),
        path.join("storage/hive-ai.db"),
    ];

    for db_path in potential_db_paths {
        if db_path.exists() {
            return analyze_sqlite_database(&db_path).await;
        }
    }

    Err(HiveError::Migration { 
        message: "No database found".to_string()
    })
}

/// Analyze SQLite database
async fn analyze_sqlite_database(db_path: &PathBuf) -> Result<DatabaseInfo, HiveError> {
    let metadata = fs::metadata(db_path).await?;
    
    // Note: In a real implementation, we would open the SQLite database
    // and query for actual statistics. For now, we provide a placeholder.
    Ok(DatabaseInfo {
        path: db_path.clone(),
        size: metadata.len(),
        conversation_count: 0, // Would query: SELECT COUNT(*) FROM conversations
        message_count: 0,      // Would query: SELECT COUNT(*) FROM messages
        schema_version: None,  // Would query schema version table
        last_backup: None,     // Would check backup metadata
        integrity_check: true, // Would run PRAGMA integrity_check
        clusters: Vec::new(),  // Would query thematic clusters
    })
}

/// Analyze usage statistics
async fn analyze_usage_stats(db_info: &DatabaseInfo) -> Result<UsageStats, HiveError> {
    // In a real implementation, this would query the database for usage patterns
    Ok(UsageStats {
        total_conversations: db_info.conversation_count,
        average_messages_per_conversation: if db_info.conversation_count > 0 {
            db_info.message_count as f64 / db_info.conversation_count as f64
        } else {
            0.0
        },
        most_used_models: Vec::new(),
        daily_usage_pattern: HashMap::new(),
        feature_usage: HashMap::new(),
        last_activity: None,
    })
}

/// Find custom consensus profiles
async fn find_custom_profiles(config_files: &[ConfigFile]) -> Result<Vec<ConsensusProfile>, HiveError> {
    let mut profiles = Vec::new();
    
    for config_file in config_files {
        if matches!(config_file.file_type, ConfigFileType::ModelProfiles) {
            // Parse profile files to extract custom profiles
            // This would be implemented based on the actual file format
        }
    }
    
    Ok(profiles)
}

/// Assess migration compatibility
async fn assess_compatibility(analysis: &TypeScriptAnalysis) -> Result<CompatibilityAssessment, HiveError> {
    let mut config_compat = 1.0;
    let mut db_compat = 1.0;
    let mut feature_compat = 1.0;
    
    // Assess configuration compatibility
    for config in &analysis.config_files {
        if !config.valid {
            config_compat -= 0.2;
        }
    }
    config_compat = f64::max(config_compat, 0.0);
    
    // Assess database compatibility
    if !analysis.database_info.integrity_check {
        db_compat -= 0.5;
    }
    
    // Determine migration complexity
    let complexity = if config_compat > 0.8 && db_compat > 0.8 && feature_compat > 0.8 {
        MigrationComplexity::Simple
    } else if config_compat > 0.6 && db_compat > 0.6 {
        MigrationComplexity::Moderate
    } else if config_compat > 0.4 || db_compat > 0.4 {
        MigrationComplexity::Complex
    } else {
        MigrationComplexity::Expert
    };

    let overall = (config_compat + db_compat + feature_compat) / 3.0;
    
    Ok(CompatibilityAssessment {
        overall_score: overall as f32,
        config_compatibility: config_compat as f32,
        database_compatibility: db_compat as f32,
        feature_compatibility: feature_compat as f32,
        migration_complexity: complexity.clone(),
        recommended_approach: match complexity {
            MigrationComplexity::Simple => "Direct automated migration".to_string(),
            MigrationComplexity::Moderate => "Assisted migration with manual verification".to_string(),
            MigrationComplexity::Complex => "Staged migration with careful testing".to_string(),
            MigrationComplexity::Expert => "Expert consultation recommended".to_string(),
        },
    })
}

/// Find issues that might affect migration
async fn find_issues(analysis: &TypeScriptAnalysis) -> Result<Vec<AnalysisIssue>, HiveError> {
    let mut issues = Vec::new();
    
    // Check for version compatibility
    if let Some(version) = &analysis.version {
        if version.starts_with('0') {
            issues.push(AnalysisIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::Compatibility,
                description: format!("Very old version detected: {}", version),
                recommendation: "Consider fresh installation instead of migration".to_string(),
                blocking: false,
            });
        }
    }
    
    // Check database integrity
    if !analysis.database_info.integrity_check {
        issues.push(AnalysisIssue {
            severity: IssueSeverity::Critical,
            category: IssueCategory::Database,
            description: "Database integrity check failed".to_string(),
            recommendation: "Repair database before migration".to_string(),
            blocking: true,
        });
    }
    
    // Check for invalid configuration files
    for config in &analysis.config_files {
        if !config.valid {
            issues.push(AnalysisIssue {
                severity: IssueSeverity::Error,
                category: IssueCategory::Configuration,
                description: format!("Invalid configuration file: {}", config.path.display()),
                recommendation: "Fix or remove invalid configuration".to_string(),
                blocking: false,
            });
        }
    }
    
    // Check permissions
    if !analysis.installation_path.exists() {
        issues.push(AnalysisIssue {
            severity: IssueSeverity::Critical,
            category: IssueCategory::Permissions,
            description: "Cannot access installation directory".to_string(),
            recommendation: "Check directory permissions".to_string(),
            blocking: true,
        });
    }
    
    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_analyze_nonexistent_installation() {
        let path = PathBuf::from("/nonexistent/path");
        let result = analyze_typescript_installation(&path).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_migration_readiness_calculation() {
        let mut analysis = TypeScriptAnalysis {
            installation_path: PathBuf::from("/test"),
            version: Some("1.0.0".to_string()),
            config_files: Vec::new(),
            database_info: DatabaseInfo {
                path: PathBuf::new(),
                size: 1000,
                conversation_count: 10,
                message_count: 100,
                schema_version: Some("1.0".to_string()),
                last_backup: None,
                integrity_check: true,
                clusters: Vec::new(),
            },
            usage_stats: UsageStats {
                total_conversations: 10,
                average_messages_per_conversation: 10.0,
                most_used_models: Vec::new(),
                daily_usage_pattern: HashMap::new(),
                feature_usage: HashMap::new(),
                last_activity: None,
            },
            custom_profiles: Vec::new(),
            issues: Vec::new(),
            compatibility: CompatibilityAssessment {
                overall_score: 0.9,
                config_compatibility: 0.9,
                database_compatibility: 0.9,
                feature_compatibility: 0.9,
                migration_complexity: MigrationComplexity::Simple,
                recommended_approach: "Direct".to_string(),
            },
        };

        let readiness = analysis.migration_readiness();
        assert!((readiness - 0.9).abs() < 0.01);

        // Add critical issue
        analysis.issues.push(AnalysisIssue {
            severity: IssueSeverity::Critical,
            category: IssueCategory::Database,
            description: "Test issue".to_string(),
            recommendation: "Fix it".to_string(),
            blocking: true,
        });

        let readiness_with_issue = analysis.migration_readiness();
        assert!(readiness_with_issue < readiness);
    }
}