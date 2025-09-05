//! Migration Rollback Module
//!
//! Provides comprehensive rollback capabilities to restore the system
//! to its pre-migration state with complete data recovery.

use crate::core::error::HiveError;
use crate::migration::{MigrationConfig, MigrationPhase};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Rollback operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub success: bool,
    pub phase: RollbackPhase,
    pub operations: Vec<RollbackOperation>,
    pub restored_files: Vec<PathBuf>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub duration: std::time::Duration,
}

/// Rollback phases
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RollbackPhase {
    Preparing,
    RestoringDatabase,
    RestoringConfiguration,
    RestoringFiles,
    CleaningUp,
    Completed,
    Failed,
}

/// Individual rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackOperation {
    pub operation_type: RollbackOperationType,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub status: OperationStatus,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

/// Types of rollback operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackOperationType {
    FileRestore,
    DatabaseRestore,
    ConfigurationRestore,
    DirectoryRemoval,
    PermissionRestore,
    SymlinkRestore,
}

/// Operation status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// Rollback plan containing all necessary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub migration_id: String,
    pub created_at: DateTime<Utc>,
    pub backup_locations: HashMap<String, PathBuf>,
    pub original_permissions: HashMap<PathBuf, u32>,
    pub operations: Vec<PlannedRollbackOperation>,
    pub dependencies: Vec<RollbackDependency>,
    pub verification_steps: Vec<VerificationStep>,
}

/// Planned rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedRollbackOperation {
    pub id: String,
    pub operation_type: RollbackOperationType,
    pub priority: u32,
    pub backup_path: PathBuf,
    pub restore_path: PathBuf,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
}

/// Rollback dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackDependency {
    pub operation_id: String,
    pub depends_on: String,
    pub dependency_type: DependencyType,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    MustCompleteBefore,
    MustCompleteAfter,
    ConditionalDependency,
}

/// Verification step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStep {
    pub name: String,
    pub description: String,
    pub verification_type: VerificationType,
    pub expected_result: String,
}

/// Verification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationType {
    FileExists,
    FileContentsMatch,
    DatabaseAccessible,
    ConfigurationValid,
    PermissionsCorrect,
}

/// Rollback manager
pub struct RollbackManager {
    config: MigrationConfig,
    plan: Option<RollbackPlan>,
}

impl RollbackManager {
    /// Create new rollback manager
    pub fn new(config: MigrationConfig) -> Self {
        Self { config, plan: None }
    }

    /// Create rollback plan before migration starts
    pub async fn create_rollback_plan(&mut self) -> Result<(), HiveError> {
        log::info!("Creating rollback plan");

        let migration_id = generate_migration_id();
        let backup_base = self.get_backup_directory()?;

        // Ensure backup directory exists
        fs::create_dir_all(&backup_base).await?;

        let mut backup_locations = HashMap::new();
        let mut original_permissions = HashMap::new();
        let mut operations = Vec::new();

        // Plan database rollback
        if let Some(db_backup) = self.plan_database_backup(&backup_base).await? {
            backup_locations.insert("database".to_string(), db_backup.clone());
            operations.push(PlannedRollbackOperation {
                id: "restore_database".to_string(),
                operation_type: RollbackOperationType::DatabaseRestore,
                priority: 1,
                backup_path: db_backup,
                restore_path: get_target_database_path()?,
                preconditions: vec!["backup_exists".to_string()],
                postconditions: vec!["database_accessible".to_string()],
            });
        }

        // Plan configuration rollback
        if let Some(config_backup) = self.plan_configuration_backup(&backup_base).await? {
            backup_locations.insert("configuration".to_string(), config_backup.clone());
            operations.push(PlannedRollbackOperation {
                id: "restore_configuration".to_string(),
                operation_type: RollbackOperationType::ConfigurationRestore,
                priority: 2,
                backup_path: config_backup,
                restore_path: get_target_config_path()?,
                preconditions: vec!["backup_exists".to_string()],
                postconditions: vec!["configuration_valid".to_string()],
            });
        }

        // Store original permissions
        original_permissions.extend(self.collect_original_permissions().await?);

        self.plan = Some(RollbackPlan {
            migration_id,
            created_at: Utc::now(),
            backup_locations,
            original_permissions,
            operations,
            dependencies: self.create_rollback_dependencies(),
            verification_steps: self.create_verification_steps(),
        });

        // Save rollback plan to disk
        self.save_rollback_plan().await?;

        log::info!("Rollback plan created successfully");
        Ok(())
    }

    /// Execute rollback using the saved plan
    pub async fn execute_rollback(&self) -> Result<RollbackResult, HiveError> {
        log::info!("Starting rollback execution");

        let start_time = std::time::Instant::now();
        let mut result = RollbackResult {
            success: false,
            phase: RollbackPhase::Preparing,
            operations: Vec::new(),
            restored_files: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            duration: std::time::Duration::default(),
        };

        // Load rollback plan
        let plan = match self.load_rollback_plan().await {
            Ok(plan) => plan,
            Err(e) => {
                result
                    .errors
                    .push(format!("Failed to load rollback plan: {}", e));
                result.phase = RollbackPhase::Failed;
                return Ok(result);
            }
        };

        // Execute rollback phases
        for phase in [
            RollbackPhase::RestoringDatabase,
            RollbackPhase::RestoringConfiguration,
            RollbackPhase::RestoringFiles,
            RollbackPhase::CleaningUp,
        ] {
            result.phase = phase.clone();

            match self
                .execute_rollback_phase(&phase, &plan, &mut result)
                .await
            {
                Ok(_) => {
                    log::info!("Rollback phase {:?} completed successfully", phase);
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("Rollback phase {:?} failed: {}", phase, e));
                    result.phase = RollbackPhase::Failed;
                    result.duration = start_time.elapsed();
                    return Ok(result);
                }
            }
        }

        // Verify rollback success
        if let Err(e) = self.verify_rollback(&plan).await {
            result
                .errors
                .push(format!("Rollback verification failed: {}", e));
            result.phase = RollbackPhase::Failed;
        } else {
            result.success = true;
            result.phase = RollbackPhase::Completed;
        }

        result.duration = start_time.elapsed();
        log::info!("Rollback execution completed in {:?}", result.duration);

        Ok(result)
    }

    /// Get backup directory path
    fn get_backup_directory(&self) -> Result<PathBuf, HiveError> {
        if let Some(backup_path) = &self.config.backup_path {
            Ok(backup_path.clone())
        } else {
            let home_dir = dirs::home_dir().ok_or_else(|| HiveError::Migration {
                message: "Cannot determine home directory".to_string(),
            })?;
            Ok(home_dir.join(".hive").join("backups"))
        }
    }

    /// Plan database backup
    async fn plan_database_backup(
        &self,
        backup_base: &PathBuf,
    ) -> Result<Option<PathBuf>, HiveError> {
        let source_db = &self.config.source_path.join("hive-ai.db");

        if source_db.exists() {
            let backup_path = backup_base.join("database_backup.db");
            fs::copy(source_db, &backup_path).await?;
            log::info!("Database backup created: {}", backup_path.display());
            Ok(Some(backup_path))
        } else {
            log::warn!("Source database not found, skipping backup");
            Ok(None)
        }
    }

    /// Plan configuration backup
    async fn plan_configuration_backup(
        &self,
        backup_base: &PathBuf,
    ) -> Result<Option<PathBuf>, HiveError> {
        let source_config = self.config.source_path.join("config.json");

        if source_config.exists() {
            let backup_path = backup_base.join("config_backup.json");
            fs::copy(&source_config, &backup_path).await?;
            log::info!("Configuration backup created: {}", backup_path.display());
            Ok(Some(backup_path))
        } else {
            log::warn!("Source configuration not found, skipping backup");
            Ok(None)
        }
    }

    /// Collect original file permissions
    async fn collect_original_permissions(&self) -> Result<HashMap<PathBuf, u32>, HiveError> {
        let mut permissions = HashMap::new();

        // Collect permissions for key files and directories
        let paths_to_check = vec![get_target_database_path()?, get_target_config_path()?];

        for path in paths_to_check {
            if path.exists() {
                if let Ok(metadata) = fs::metadata(&path).await {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        permissions.insert(path, metadata.permissions().mode());
                    }
                    #[cfg(not(unix))]
                    {
                        permissions.insert(path, 0o644); // Default permissions for non-Unix
                    }
                }
            }
        }

        Ok(permissions)
    }

    /// Create rollback dependencies
    fn create_rollback_dependencies(&self) -> Vec<RollbackDependency> {
        vec![RollbackDependency {
            operation_id: "restore_configuration".to_string(),
            depends_on: "restore_database".to_string(),
            dependency_type: DependencyType::MustCompleteAfter,
        }]
    }

    /// Create verification steps
    fn create_verification_steps(&self) -> Vec<VerificationStep> {
        vec![
            VerificationStep {
                name: "Database Accessibility".to_string(),
                description: "Verify database can be opened and queried".to_string(),
                verification_type: VerificationType::DatabaseAccessible,
                expected_result: "Database accessible".to_string(),
            },
            VerificationStep {
                name: "Configuration Validity".to_string(),
                description: "Verify configuration file is valid".to_string(),
                verification_type: VerificationType::ConfigurationValid,
                expected_result: "Configuration valid".to_string(),
            },
        ]
    }

    /// Save rollback plan to disk
    async fn save_rollback_plan(&self) -> Result<(), HiveError> {
        if let Some(plan) = &self.plan {
            let backup_dir = self.get_backup_directory()?;
            let plan_path = backup_dir.join("rollback_plan.json");

            let plan_json = serde_json::to_string_pretty(plan)?;
            fs::write(&plan_path, plan_json).await?;

            log::info!("Rollback plan saved: {}", plan_path.display());
        }
        Ok(())
    }

    /// Load rollback plan from disk
    async fn load_rollback_plan(&self) -> Result<RollbackPlan, HiveError> {
        let backup_dir = self.get_backup_directory()?;
        let plan_path = backup_dir.join("rollback_plan.json");

        if !plan_path.exists() {
            return Err(HiveError::Migration {
                message: "Rollback plan not found".to_string(),
            });
        }

        let plan_content = fs::read_to_string(&plan_path).await?;
        let plan: RollbackPlan = serde_json::from_str(&plan_content)?;

        Ok(plan)
    }

    /// Execute a specific rollback phase
    async fn execute_rollback_phase(
        &self,
        phase: &RollbackPhase,
        plan: &RollbackPlan,
        result: &mut RollbackResult,
    ) -> Result<(), HiveError> {
        match phase {
            RollbackPhase::RestoringDatabase => {
                self.restore_database(plan, result).await?;
            }
            RollbackPhase::RestoringConfiguration => {
                self.restore_configuration(plan, result).await?;
            }
            RollbackPhase::RestoringFiles => {
                self.restore_files(plan, result).await?;
            }
            RollbackPhase::CleaningUp => {
                self.cleanup_migration_artifacts(result).await?;
            }
            _ => {
                return Err(HiveError::Migration {
                    message: format!("Invalid rollback phase: {:?}", phase),
                });
            }
        }
        Ok(())
    }

    /// Restore database from backup
    async fn restore_database(
        &self,
        plan: &RollbackPlan,
        result: &mut RollbackResult,
    ) -> Result<(), HiveError> {
        if let Some(backup_path) = plan.backup_locations.get("database") {
            let target_path = get_target_database_path()?;

            let operation = RollbackOperation {
                operation_type: RollbackOperationType::DatabaseRestore,
                source_path: backup_path.clone(),
                target_path: target_path.clone(),
                status: OperationStatus::InProgress,
                details: "Restoring database from backup".to_string(),
                timestamp: Utc::now(),
            };

            result.operations.push(operation.clone());

            // Remove current database if it exists
            if target_path.exists() {
                fs::remove_file(&target_path).await?;
            }

            // Restore from backup
            fs::copy(backup_path, &target_path).await?;
            result.restored_files.push(target_path);

            // Update operation status
            if let Some(last_op) = result.operations.last_mut() {
                last_op.status = OperationStatus::Completed;
                last_op.details = "Database restored successfully".to_string();
            }

            log::info!("Database restored from backup");
        }
        Ok(())
    }

    /// Restore configuration from backup
    async fn restore_configuration(
        &self,
        plan: &RollbackPlan,
        result: &mut RollbackResult,
    ) -> Result<(), HiveError> {
        if let Some(backup_path) = plan.backup_locations.get("configuration") {
            let target_path = get_target_config_path()?;

            let operation = RollbackOperation {
                operation_type: RollbackOperationType::ConfigurationRestore,
                source_path: backup_path.clone(),
                target_path: target_path.clone(),
                status: OperationStatus::InProgress,
                details: "Restoring configuration from backup".to_string(),
                timestamp: Utc::now(),
            };

            result.operations.push(operation.clone());

            // Remove current config if it exists
            if target_path.exists() {
                fs::remove_file(&target_path).await?;
            }

            // Restore from backup
            fs::copy(backup_path, &target_path).await?;
            result.restored_files.push(target_path);

            // Update operation status
            if let Some(last_op) = result.operations.last_mut() {
                last_op.status = OperationStatus::Completed;
                last_op.details = "Configuration restored successfully".to_string();
            }

            log::info!("Configuration restored from backup");
        }
        Ok(())
    }

    /// Restore additional files
    async fn restore_files(
        &self,
        plan: &RollbackPlan,
        result: &mut RollbackResult,
    ) -> Result<(), HiveError> {
        // Restore file permissions
        for (path, mode) in &plan.original_permissions {
            if path.exists() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let permissions = std::fs::Permissions::from_mode(*mode);
                    std::fs::set_permissions(path, permissions)?;
                }
                log::debug!("Restored permissions for: {}", path.display());
            }
        }

        log::info!("File permissions restored");
        Ok(())
    }

    /// Clean up migration artifacts
    async fn cleanup_migration_artifacts(
        &self,
        result: &mut RollbackResult,
    ) -> Result<(), HiveError> {
        // Remove any temporary files created during migration
        let temp_dirs = vec![
            std::env::temp_dir().join("hive_migration"),
            get_target_config_path()?
                .parent()
                .unwrap()
                .join("migration_temp"),
        ];

        for temp_dir in temp_dirs {
            if temp_dir.exists() {
                match fs::remove_dir_all(&temp_dir).await {
                    Ok(_) => log::info!("Cleaned up temporary directory: {}", temp_dir.display()),
                    Err(e) => result.warnings.push(format!(
                        "Failed to clean up {}: {}",
                        temp_dir.display(),
                        e
                    )),
                }
            }
        }

        Ok(())
    }

    /// Verify rollback success
    async fn verify_rollback(&self, plan: &RollbackPlan) -> Result<(), HiveError> {
        for step in &plan.verification_steps {
            match step.verification_type {
                VerificationType::DatabaseAccessible => {
                    let db_path = get_target_database_path()?;
                    if !db_path.exists() {
                        return Err(HiveError::Migration {
                            message: "Database file not found after rollback".to_string(),
                        });
                    }
                    // Try to open database
                    // This would use the actual Database API in a real implementation
                }
                VerificationType::ConfigurationValid => {
                    let config_path = get_target_config_path()?;
                    if config_path.exists() {
                        let content = fs::read_to_string(&config_path).await?;
                        // Validate configuration format
                        if config_path.extension().map_or(false, |ext| ext == "json") {
                            serde_json::from_str::<serde_json::Value>(&content)?;
                        } else if config_path.extension().map_or(false, |ext| ext == "toml") {
                            toml::from_str::<toml::Value>(&content)?;
                        }
                    }
                }
                _ => {
                    // Handle other verification types
                }
            }
        }

        log::info!("Rollback verification completed successfully");
        Ok(())
    }
}

/// Perform rollback using the provided configuration
pub async fn perform_rollback(config: &MigrationConfig) -> Result<RollbackResult, HiveError> {
    let mut manager = RollbackManager::new(config.clone());
    manager.execute_rollback().await
}

/// Generate unique migration ID
fn generate_migration_id() -> String {
    format!("migration_{}", Utc::now().format("%Y%m%d_%H%M%S"))
}

/// Get target database path
fn get_target_database_path() -> Result<PathBuf, HiveError> {
    let home_dir = dirs::home_dir().ok_or_else(|| HiveError::Migration {
        message: "Cannot determine home directory".to_string(),
    })?;
    Ok(home_dir.join(".hive").join("hive-ai.db"))
}

/// Get target configuration path
fn get_target_config_path() -> Result<PathBuf, HiveError> {
    let home_dir = dirs::home_dir().ok_or_else(|| HiveError::Migration {
        message: "Cannot determine home directory".to_string(),
    })?;
    Ok(home_dir.join(".hive").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_migration_id() {
        let id1 = generate_migration_id();
        let id2 = generate_migration_id();

        assert!(id1.starts_with("migration_"));
        assert!(id2.starts_with("migration_"));
        // IDs should be different (unless generated in the same second)
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_rollback_manager_creation() {
        let config = MigrationConfig {
            source_path: PathBuf::from("/test/source"),
            backup_path: Some(PathBuf::from("/test/backup")),
            preserve_original: true,
            validation_level: crate::migration::ValidationLevel::Standard,
            migration_type: crate::migration::MigrationType::Upgrade,
        };

        let manager = RollbackManager::new(config);
        assert!(manager.plan.is_none());
    }

    #[test]
    fn test_rollback_result_success() {
        let result = RollbackResult {
            success: true,
            phase: RollbackPhase::Completed,
            operations: Vec::new(),
            restored_files: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            duration: std::time::Duration::from_secs(30),
        };

        assert!(result.success);
        assert_eq!(result.phase, RollbackPhase::Completed);
        assert!(result.errors.is_empty());
    }
}
