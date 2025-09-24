//! AI Helper Rollback Executor
//!
//! This module provides rollback execution capabilities for AI Helpers.
//! It executes rollback plans created by the consensus layer, ensuring proper
//! separation of concerns where consensus THINKS and AI Helpers DO.

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

use super::file_executor::{AIHelperFileExecutor, ExecutionReport};
use super::intelligent_executor::IntelligentExecutor;
use super::AIHelperEcosystem;

/// Rollback operation plan from consensus layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub plan_id: String,
    pub operations: Vec<RollbackOperation>,
    pub safety_level: RollbackSafetyLevel,
    pub verification_required: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackSafetyLevel {
    Low,      // Simple file operations
    Medium,   // Multiple files, some risk
    High,     // System-wide changes, deletions
    Critical, // Irreversible operations
}

/// Individual rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackOperation {
    pub operation_id: String,
    pub action: RollbackAction,
    pub description: String,
    pub files_affected: Vec<PathBuf>,
    pub dependencies: Vec<String>, // IDs of operations that must complete first
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RollbackAction {
    DeleteCreatedFile {
        path: PathBuf,
    },
    RestoreFromBackup {
        backup_path: PathBuf,
        target_path: PathBuf,
    },
    UndoRename {
        current_path: PathBuf,
        original_path: PathBuf,
    },
    RecreateDeletedFile {
        path: PathBuf,
        content: String,
        permissions: Option<u32>,
    },
    RevertModification {
        path: PathBuf,
        original_content: String,
    },
    RestoreDirectory {
        path: PathBuf,
    },
    RunScript {
        script_path: PathBuf,
        args: Vec<String>,
    },
    GitRevert {
        commit_hash: String,
        file_paths: Option<Vec<PathBuf>>,
    },
    NoOp {
        reason: String,
    },
}

/// Rollback execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub plan_id: String,
    pub success: bool,
    pub operations_completed: usize,
    pub operations_total: usize,
    pub operation_results: Vec<OperationResult>,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub operation_id: String,
    pub success: bool,
    pub error: Option<String>,
    pub files_affected: Vec<PathBuf>,
    pub duration_ms: u64,
}

/// AI Helper Rollback Executor
pub struct AIHelperRollbackExecutor {
    file_executor: AIHelperFileExecutor,
    intelligent_executor: Option<IntelligentExecutor>,
    dry_run: bool,
}

impl AIHelperRollbackExecutor {
    pub fn new(ai_helpers: AIHelperEcosystem) -> Self {
        Self {
            file_executor: AIHelperFileExecutor::new(ai_helpers),
            intelligent_executor: None,
            dry_run: false,
        }
    }

    pub fn with_intelligent_executor(mut self, executor: IntelligentExecutor) -> Self {
        self.intelligent_executor = Some(executor);
        self
    }

    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.dry_run = enabled;
        self
    }

    /// Execute a complete rollback plan
    pub async fn execute_rollback_plan(&self, plan: RollbackPlan) -> Result<RollbackResult> {
        info!("🔄 Executing rollback plan: {}", plan.plan_id);
        let start_time = std::time::Instant::now();

        let mut result = RollbackResult {
            plan_id: plan.plan_id.clone(),
            success: true,
            operations_completed: 0,
            operations_total: plan.operations.len(),
            operation_results: Vec::new(),
            errors: Vec::new(),
            duration_ms: 0,
        };

        // Verify safety level
        if matches!(plan.safety_level, RollbackSafetyLevel::Critical) && !self.dry_run {
            warn!("⚠️ Critical rollback operations detected. Extra verification required.");
        }

        // Execute operations in dependency order
        let ordered_ops = self.order_operations_by_dependencies(&plan.operations)?;

        for op_idx in ordered_ops {
            let operation = &plan.operations[op_idx];
            info!(
                "Executing rollback operation {}/{}: {}",
                result.operations_completed + 1,
                result.operations_total,
                operation.description
            );

            let op_start = std::time::Instant::now();
            match self.execute_single_operation(operation).await {
                Ok(files) => {
                    result.operations_completed += 1;
                    result.operation_results.push(OperationResult {
                        operation_id: operation.operation_id.clone(),
                        success: true,
                        error: None,
                        files_affected: files,
                        duration_ms: op_start.elapsed().as_millis() as u64,
                    });
                }
                Err(e) => {
                    error!("❌ Rollback operation failed: {}", e);
                    let error_msg = format!("Operation {}: {}", operation.operation_id, e);
                    result.errors.push(error_msg.clone());
                    result.operation_results.push(OperationResult {
                        operation_id: operation.operation_id.clone(),
                        success: false,
                        error: Some(e.to_string()),
                        files_affected: vec![],
                        duration_ms: op_start.elapsed().as_millis() as u64,
                    });

                    // Stop on high/critical safety level operations
                    if matches!(
                        plan.safety_level,
                        RollbackSafetyLevel::High | RollbackSafetyLevel::Critical
                    ) {
                        result.success = false;
                        break;
                    }
                }
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;

        if result.operations_completed == result.operations_total {
            info!(
                "✅ Rollback completed successfully: {} operations",
                result.operations_completed
            );
        } else {
            warn!(
                "⚠️ Rollback partially completed: {}/{} operations",
                result.operations_completed, result.operations_total
            );
            result.success = false;
        }

        Ok(result)
    }

    /// Execute a single rollback operation
    async fn execute_single_operation(
        &self,
        operation: &RollbackOperation,
    ) -> Result<Vec<PathBuf>> {
        if self.dry_run {
            info!("DRY RUN: Would execute: {:?}", operation.action);
            return Ok(operation.files_affected.clone());
        }

        // Use intelligent executor if available
        if let Some(ref intelligent) = self.intelligent_executor {
            debug!("Using intelligent executor for rollback operation");
            // Could enhance with learning from rollback patterns
        }

        match &operation.action {
            RollbackAction::DeleteCreatedFile { path } => self.delete_file(path).await,
            RollbackAction::RestoreFromBackup {
                backup_path,
                target_path,
            } => self.restore_from_backup(backup_path, target_path).await,
            RollbackAction::UndoRename {
                current_path,
                original_path,
            } => self.rename_file(current_path, original_path).await,
            RollbackAction::RecreateDeletedFile {
                path,
                content,
                permissions,
            } => {
                self.recreate_file(path, content, permissions.as_ref())
                    .await
            }
            RollbackAction::RevertModification {
                path,
                original_content,
            } => self.revert_file_content(path, original_content).await,
            RollbackAction::RestoreDirectory { path } => self.restore_directory(path).await,
            RollbackAction::RunScript { script_path, args } => {
                self.run_rollback_script(script_path, args).await
            }
            RollbackAction::GitRevert {
                commit_hash,
                file_paths,
            } => self.git_revert(commit_hash, file_paths.as_ref()).await,
            RollbackAction::NoOp { reason } => {
                debug!("No-op rollback: {}", reason);
                Ok(vec![])
            }
        }
    }

    /// Delete a file safely
    async fn delete_file(&self, path: &Path) -> Result<Vec<PathBuf>> {
        if !path.exists() {
            warn!("File already deleted: {:?}", path);
            return Ok(vec![]);
        }

        // Safety check
        if path.to_string_lossy().contains("..") {
            bail!("Path traversal detected in rollback");
        }

        fs::remove_file(path).with_context(|| format!("Failed to delete file: {:?}", path))?;

        info!("🗑️ Deleted file: {:?}", path);
        Ok(vec![path.to_path_buf()])
    }

    /// Restore file from backup
    async fn restore_from_backup(
        &self,
        backup_path: &Path,
        target_path: &Path,
    ) -> Result<Vec<PathBuf>> {
        if !backup_path.exists() {
            bail!("Backup file does not exist: {:?}", backup_path);
        }

        // Create parent directory if needed
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory: {:?}", parent))?;
        }

        fs::copy(backup_path, target_path).with_context(|| {
            format!(
                "Failed to restore from backup: {:?} -> {:?}",
                backup_path, target_path
            )
        })?;

        info!("📄 Restored file from backup: {:?}", target_path);
        Ok(vec![target_path.to_path_buf()])
    }

    /// Rename/move a file
    async fn rename_file(&self, current_path: &Path, original_path: &Path) -> Result<Vec<PathBuf>> {
        if !current_path.exists() {
            bail!("Source file does not exist: {:?}", current_path);
        }

        // Create parent directory for destination if needed
        if let Some(parent) = original_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::rename(current_path, original_path).with_context(|| {
            format!(
                "Failed to rename: {:?} -> {:?}",
                current_path, original_path
            )
        })?;

        info!("📝 Renamed file: {:?} -> {:?}", current_path, original_path);
        Ok(vec![original_path.to_path_buf()])
    }

    /// Recreate a deleted file
    async fn recreate_file(
        &self,
        path: &Path,
        content: &str,
        permissions: Option<&u32>,
    ) -> Result<Vec<PathBuf>> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content).with_context(|| format!("Failed to recreate file: {:?}", path))?;

        // Set permissions if specified (Unix only)
        #[cfg(unix)]
        if let Some(perms) = permissions {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(*perms);
            fs::set_permissions(path, permissions)?;
        }

        info!("📄 Recreated file: {:?}", path);
        Ok(vec![path.to_path_buf()])
    }

    /// Revert file content to original
    async fn revert_file_content(
        &self,
        path: &Path,
        original_content: &str,
    ) -> Result<Vec<PathBuf>> {
        if !path.exists() {
            bail!("File does not exist: {:?}", path);
        }

        fs::write(path, original_content)
            .with_context(|| format!("Failed to revert file content: {:?}", path))?;

        info!("⏪ Reverted file content: {:?}", path);
        Ok(vec![path.to_path_buf()])
    }

    /// Restore a directory
    async fn restore_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to restore directory: {:?}", path))?;

        info!("📁 Restored directory: {:?}", path);
        Ok(vec![path.to_path_buf()])
    }

    /// Run a rollback script
    async fn run_rollback_script(
        &self,
        script_path: &Path,
        args: &[String],
    ) -> Result<Vec<PathBuf>> {
        if !script_path.exists() {
            bail!("Rollback script does not exist: {:?}", script_path);
        }

        let output = Command::new(script_path)
            .args(args)
            .output()
            .await
            .with_context(|| format!("Failed to run rollback script: {:?}", script_path))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Rollback script failed: {}", stderr);
        }

        info!("🔧 Executed rollback script: {:?}", script_path);
        Ok(vec![script_path.to_path_buf()])
    }

    /// Git revert operation
    async fn git_revert(
        &self,
        commit_hash: &str,
        file_paths: Option<&Vec<PathBuf>>,
    ) -> Result<Vec<PathBuf>> {
        let mut cmd = Command::new("git");
        cmd.arg("revert").arg("--no-commit").arg(commit_hash);

        if let Some(paths) = file_paths {
            cmd.arg("--");
            for path in paths {
                cmd.arg(path);
            }
        }

        let output = cmd
            .output()
            .await
            .with_context(|| format!("Failed to run git revert: {}", commit_hash))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Git revert failed: {}", stderr);
        }

        info!("🔄 Git reverted commit: {}", commit_hash);
        Ok(file_paths.cloned().unwrap_or_default())
    }

    /// Order operations by dependencies
    fn order_operations_by_dependencies(
        &self,
        operations: &[RollbackOperation],
    ) -> Result<Vec<usize>> {
        let mut ordered = Vec::new();
        let mut processed = vec![false; operations.len()];

        // First, add operations without dependencies
        for (idx, op) in operations.iter().enumerate() {
            if op.dependencies.is_empty() {
                ordered.push(idx);
                processed[idx] = true;
            }
        }

        // Then, add operations with satisfied dependencies
        while ordered.len() < operations.len() {
            let mut added_any = false;

            for (idx, op) in operations.iter().enumerate() {
                if !processed[idx] {
                    let deps_satisfied = op.dependencies.iter().all(|dep_id| {
                        operations
                            .iter()
                            .position(|o| &o.operation_id == dep_id)
                            .map(|dep_idx| processed[dep_idx])
                            .unwrap_or(true)
                    });

                    if deps_satisfied {
                        ordered.push(idx);
                        processed[idx] = true;
                        added_any = true;
                    }
                }
            }

            if !added_any {
                bail!("Circular dependencies detected in rollback operations");
            }
        }

        Ok(ordered)
    }

    /// Verify rollback operation can be executed
    pub async fn verify_operation(&self, operation: &RollbackOperation) -> Result<bool> {
        match &operation.action {
            RollbackAction::DeleteCreatedFile { path } => Ok(path.exists()),
            RollbackAction::RestoreFromBackup { backup_path, .. } => Ok(backup_path.exists()),
            RollbackAction::UndoRename {
                current_path,
                original_path,
            } => Ok(current_path.exists() && !original_path.exists()),
            RollbackAction::RecreateDeletedFile { path, .. } => Ok(!path.exists()),
            RollbackAction::RevertModification { path, .. } => Ok(path.exists()),
            RollbackAction::RestoreDirectory { path } => Ok(!path.exists()),
            RollbackAction::RunScript { script_path, .. } => Ok(script_path.exists()),
            _ => Ok(true),
        }
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_ordering() {
        let operations = vec![
            RollbackOperation {
                operation_id: "op1".to_string(),
                action: RollbackAction::NoOp {
                    reason: "test".to_string(),
                },
                description: "Op 1".to_string(),
                files_affected: vec![],
                dependencies: vec!["op2".to_string()],
            },
            RollbackOperation {
                operation_id: "op2".to_string(),
                action: RollbackAction::NoOp {
                    reason: "test".to_string(),
                },
                description: "Op 2".to_string(),
                files_affected: vec![],
                dependencies: vec![],
            },
        ];

        let executor = AIHelperRollbackExecutor::new(AIHelperEcosystem::new_mock());
        let order = executor
            .order_operations_by_dependencies(&operations)
            .unwrap();

        assert_eq!(order, vec![1, 0]); // op2 first, then op1
    }
}
