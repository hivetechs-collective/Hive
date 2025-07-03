//! Migration Tools Foundation
//! 
//! Provides TypeScript to Rust migration capabilities with zero data loss.
//! Handles configuration migration, database transfer, and validation.

pub mod analyzer;
pub mod config;
pub mod database;
pub mod validator;
pub mod rollback;
pub mod guide;
pub mod database_impl;
pub mod live_test;
pub mod performance;
pub mod ui;
pub mod validation_suite;

use crate::core::error::HiveError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Migration status and progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub phase: MigrationPhase,
    pub progress: f32,
    pub message: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Migration phases
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MigrationPhase {
    NotStarted,
    Analyzing,
    ConfigMigration,
    DatabaseMigration,
    Validation,
    Completed,
    Failed,
    RolledBack,
}

/// Migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    pub source_path: PathBuf,
    pub backup_path: Option<PathBuf>,
    pub preserve_original: bool,
    pub validation_level: ValidationLevel,
    pub migration_type: MigrationType,
}

/// Validation strictness levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationLevel {
    Basic,
    Standard,
    Strict,
    Paranoid,
}

/// Migration scenario types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MigrationType {
    Fresh,          // No existing TypeScript installation
    Upgrade,        // Convert existing TypeScript to Rust
    Parallel,       // Run both versions temporarily
    Staged,         // Gradual feature-by-feature migration
}

/// Main migration coordinator
pub struct MigrationManager {
    config: MigrationConfig,
    status: MigrationStatus,
}

impl MigrationManager {
    /// Create new migration manager
    pub fn new(config: MigrationConfig) -> Self {
        Self {
            config,
            status: MigrationStatus {
                phase: MigrationPhase::NotStarted,
                progress: 0.0,
                message: "Migration not started".to_string(),
                errors: Vec::new(),
                warnings: Vec::new(),
            },
        }
    }

    /// Start migration process
    pub async fn migrate(&mut self) -> Result<(), HiveError> {
        self.update_status(MigrationPhase::Analyzing, 0.1, "Analyzing TypeScript installation")?;
        
        // Phase 1: Analyze existing installation
        let analysis = analyzer::analyze_typescript_installation(&self.config.source_path).await?;
        
        if analysis.has_critical_issues() {
            return Err(HiveError::Migration {
                message: "Critical issues found in TypeScript installation".to_string()
            });
        }

        // Phase 2: Migrate configuration
        self.update_status(MigrationPhase::ConfigMigration, 0.3, "Migrating configuration")?;
        config::migrate_configuration(&analysis).await?;

        // Phase 3: Migrate database
        self.update_status(MigrationPhase::DatabaseMigration, 0.6, "Migrating database")?;
        database::migrate_database(&analysis).await?;

        // Phase 4: Validate migration
        self.update_status(MigrationPhase::Validation, 0.8, "Validating migration")?;
        let validation_result = validator::validate_migration(&self.config.validation_level).await?;
        
        if !validation_result.is_valid() {
            self.update_status(MigrationPhase::Failed, 0.8, "Migration validation failed")?;
            return Err(HiveError::Migration {
                message: format!("Migration validation failed: {}", validation_result.summary())
            });
        }

        self.update_status(MigrationPhase::Completed, 1.0, "Migration completed successfully")?;
        Ok(())
    }

    /// Get current migration status
    pub fn status(&self) -> &MigrationStatus {
        &self.status
    }

    /// Rollback migration
    pub async fn rollback(&mut self) -> Result<(), HiveError> {
        rollback::perform_rollback(&self.config).await?;
        self.status.phase = MigrationPhase::RolledBack;
        Ok(())
    }

    /// Update migration status
    fn update_status(&mut self, phase: MigrationPhase, progress: f32, message: &str) -> Result<(), HiveError> {
        self.status.phase = phase;
        self.status.progress = progress;
        self.status.message = message.to_string();
        
        // Log progress
        log::info!("Migration progress: {:.1}% - {}", progress * 100.0, message);
        
        Ok(())
    }
}

/// Quick analysis without full migration
pub async fn analyze_only(path: &PathBuf) -> Result<analyzer::TypeScriptAnalysis, HiveError> {
    analyzer::analyze_typescript_installation(path).await
}

/// Preview migration changes without applying them
pub async fn preview_migration(config: &MigrationConfig) -> Result<MigrationPreview, HiveError> {
    let analysis = analyzer::analyze_typescript_installation(&config.source_path).await?;
    let config_changes = config::preview_config_changes(&analysis).await?;
    let database_changes = database::preview_database_changes(&analysis).await?;
    
    Ok(MigrationPreview {
        analysis,
        config_changes,
        database_changes,
        estimated_duration: estimate_migration_time(&config),
        risks: assess_migration_risks(&config),
    })
}

/// Migration preview information
#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationPreview {
    pub analysis: analyzer::TypeScriptAnalysis,
    pub config_changes: config::ConfigChanges,
    pub database_changes: database::DatabaseChanges,
    pub estimated_duration: std::time::Duration,
    pub risks: Vec<String>,
}

/// Estimate migration time based on configuration and data size
fn estimate_migration_time(config: &MigrationConfig) -> std::time::Duration {
    // Base time for analysis and setup
    let mut duration = std::time::Duration::from_secs(30);
    
    // Add time based on migration type
    match config.migration_type {
        MigrationType::Fresh => duration += std::time::Duration::from_secs(60),
        MigrationType::Upgrade => duration += std::time::Duration::from_secs(180),
        MigrationType::Parallel => duration += std::time::Duration::from_secs(120),
        MigrationType::Staged => duration += std::time::Duration::from_secs(300),
    }
    
    // Add time based on validation level
    match config.validation_level {
        ValidationLevel::Basic => duration += std::time::Duration::from_secs(30),
        ValidationLevel::Standard => duration += std::time::Duration::from_secs(60),
        ValidationLevel::Strict => duration += std::time::Duration::from_secs(120),
        ValidationLevel::Paranoid => duration += std::time::Duration::from_secs(300),
    }
    
    duration
}

/// Assess migration risks
fn assess_migration_risks(config: &MigrationConfig) -> Vec<String> {
    let mut risks = Vec::new();
    
    if !config.preserve_original {
        risks.push("Original TypeScript installation will be removed".to_string());
    }
    
    if config.backup_path.is_none() {
        risks.push("No backup path specified - data loss possible".to_string());
    }
    
    match config.migration_type {
        MigrationType::Upgrade => {
            risks.push("In-place upgrade may cause temporary service interruption".to_string());
        },
        MigrationType::Staged => {
            risks.push("Staged migration requires multiple phases and careful coordination".to_string());
        },
        _ => {}
    }
    
    risks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_migration_manager_creation() {
        let config = MigrationConfig {
            source_path: PathBuf::from("/test/path"),
            backup_path: Some(PathBuf::from("/backup/path")),
            preserve_original: true,
            validation_level: ValidationLevel::Standard,
            migration_type: MigrationType::Upgrade,
        };
        
        let manager = MigrationManager::new(config);
        assert_eq!(manager.status.phase, MigrationPhase::NotStarted);
        assert_eq!(manager.status.progress, 0.0);
    }

    #[test]
    fn test_migration_time_estimation() {
        let config = MigrationConfig {
            source_path: PathBuf::from("/test/path"),
            backup_path: None,
            preserve_original: false,
            validation_level: ValidationLevel::Paranoid,
            migration_type: MigrationType::Staged,
        };
        
        let duration = estimate_migration_time(&config);
        // Should be at least 630 seconds (30 base + 300 staged + 300 paranoid)
        assert!(duration.as_secs() >= 630);
    }

    #[test]
    fn test_risk_assessment() {
        let config = MigrationConfig {
            source_path: PathBuf::from("/test/path"),
            backup_path: None,
            preserve_original: false,
            validation_level: ValidationLevel::Basic,
            migration_type: MigrationType::Upgrade,
        };
        
        let risks = assess_migration_risks(&config);
        assert!(risks.len() >= 2); // Should identify multiple risks
        assert!(risks.iter().any(|r| r.contains("Original TypeScript installation")));
        assert!(risks.iter().any(|r| r.contains("No backup path")));
    }
}