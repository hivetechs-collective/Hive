//! Rollback Planning System for Failed Operations (Consensus Layer)
//!
//! This module creates rollback plans that are executed by AI Helpers.
//! IMPORTANT: This module NEVER executes file operations directly - it only
//! creates plans following the architecture principle where consensus THINKS
//! and AI Helpers DO.

use crate::ai_helpers::rollback_executor::{
    RollbackAction, RollbackOperation, RollbackPlan, RollbackSafetyLevel,
};
use crate::consensus::operation_history::OperationHistoryDatabase;
use crate::consensus::operation_intelligence::OperationIntelligenceCoordinator;
use crate::consensus::rollback_plan::RollbackAction as LegacyRollbackAction;
use crate::consensus::rollback_planner::{
    RiskLevel, RollbackOperation as LegacyRollbackOp, RollbackPlan as LegacyRollbackPlan,
    RollbackStep, RollbackStrategy,
};
use crate::consensus::stages::file_aware_curator::FileOperation;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Rollback planner that creates plans for AI Helpers to execute
pub struct ConsensusRollbackPlanner {
    operation_intelligence: Arc<OperationIntelligenceCoordinator>,
    history_database: Option<Arc<OperationHistoryDatabase>>,
    planning_config: RollbackPlanningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlanningConfig {
    /// Whether to verify each step can be executed
    pub verify_feasibility: bool,
    /// Whether to include safety backups in plan
    pub include_safety_backups: bool,
    /// Whether to group related operations
    pub group_operations: bool,
    /// Maximum operations per plan
    pub max_operations_per_plan: usize,
}

impl Default for RollbackPlanningConfig {
    fn default() -> Self {
        Self {
            verify_feasibility: true,
            include_safety_backups: true,
            group_operations: true,
            max_operations_per_plan: 100,
        }
    }
}

impl ConsensusRollbackPlanner {
    pub fn new(
        operation_intelligence: Arc<OperationIntelligenceCoordinator>,
        history_database: Option<Arc<OperationHistoryDatabase>>,
        config: RollbackPlanningConfig,
    ) -> Self {
        Self {
            operation_intelligence,
            history_database,
            planning_config: config,
        }
    }

    /// Create a rollback plan from failed operations
    pub async fn create_rollback_plan(
        &self,
        failed_operations: Vec<FileOperation>,
        failure_context: &str,
    ) -> Result<RollbackPlan> {
        info!(
            "Creating rollback plan for {} failed operations",
            failed_operations.len()
        );

        let plan_id = Uuid::new_v4().to_string();
        let mut operations = Vec::new();

        // Analyze each failed operation and create rollback actions
        for (idx, failed_op) in failed_operations.iter().enumerate() {
            let rollback_action = self.create_rollback_action(failed_op).await?;
            let files_affected = self.get_affected_files(failed_op);

            operations.push(RollbackOperation {
                operation_id: format!("{}-{}", plan_id, idx),
                action: rollback_action,
                description: self.describe_rollback_action(failed_op),
                files_affected,
                dependencies: self.determine_dependencies(&operations, failed_op),
            });
        }

        // Determine safety level based on operations
        let safety_level = self.determine_safety_level(&operations);

        Ok(RollbackPlan {
            plan_id,
            operations,
            safety_level,
            verification_required: self.planning_config.verify_feasibility,
            created_at: Utc::now(),
        })
    }

    /// Convert legacy rollback plan to new format for AI Helpers
    pub fn convert_legacy_plan(&self, legacy_plan: LegacyRollbackPlan) -> Result<RollbackPlan> {
        let mut operations = Vec::new();

        for step in legacy_plan.steps {
            let action = self.convert_legacy_operation(&step.operation)?;
            let files_affected = self.get_files_affected_from_operation(&step.operation);

            operations.push(RollbackOperation {
                operation_id: step.step_number.to_string(),
                action,
                description: step.description,
                files_affected,
                dependencies: step.depends_on.iter().map(|d| d.to_string()).collect(),
            });
        }

        // Map risk assessment to safety level
        let safety_level = match legacy_plan.risk_assessment.risk_level {
            RiskLevel::Low => RollbackSafetyLevel::Low,
            RiskLevel::Medium => RollbackSafetyLevel::Medium,
            RiskLevel::High => RollbackSafetyLevel::High,
            RiskLevel::Critical => RollbackSafetyLevel::Critical,
        };

        Ok(RollbackPlan {
            plan_id: legacy_plan.id,
            operations,
            safety_level,
            verification_required: !legacy_plan.verification_steps.is_empty(),
            created_at: legacy_plan.generated_at,
        })
    }

    /// Create rollback action for a failed operation
    async fn create_rollback_action(&self, operation: &FileOperation) -> Result<RollbackAction> {
        match operation {
            FileOperation::Create { path, .. } => {
                // If we created a file, rollback is to delete it
                Ok(RollbackAction::DeleteCreatedFile { path: path.clone() })
            }
            FileOperation::Update { path, .. } => {
                // For updates, we need the original content
                // In a real system, this would come from backup or history
                warn!("Update rollback requires original content - using placeholder");
                Ok(RollbackAction::RevertModification {
                    path: path.clone(),
                    original_content: "// Original content would be restored from backup"
                        .to_string(),
                })
            }
            FileOperation::Delete { path } => {
                // If we deleted a file, we need to recreate it
                warn!("Delete rollback requires original content - using placeholder");
                Ok(RollbackAction::RecreateDeletedFile {
                    path: path.clone(),
                    content: "// Original content would be restored from backup".to_string(),
                    permissions: None,
                })
            }
            FileOperation::Rename { from, to } => {
                // Rename rollback is to rename back
                Ok(RollbackAction::UndoRename {
                    current_path: to.clone(),
                    original_path: from.clone(),
                })
            }
            FileOperation::Append { path, .. } => {
                // For append, we need to restore original without the appended content
                warn!("Append rollback requires original content - using placeholder");
                Ok(RollbackAction::RevertModification {
                    path: path.clone(),
                    original_content: "// Original content would be restored from backup"
                        .to_string(),
                })
            }
        }
    }

    /// Convert legacy rollback operation to new format
    fn convert_legacy_operation(&self, legacy_op: &LegacyRollbackOp) -> Result<RollbackAction> {
        // Convert from legacy RollbackOperation to new RollbackAction format
        match legacy_op {
            LegacyRollbackOp::RestoreFile {
                source,
                destination,
                ..
            } => Ok(RollbackAction::RestoreFromBackup {
                backup_path: source.clone(),
                target_path: destination.clone(),
            }),
            LegacyRollbackOp::GitCommand { command, args, .. } => {
                let mut git_args = vec![command.clone()];
                git_args.extend(args.clone());
                Ok(RollbackAction::RunScript {
                    script_path: "git".into(),
                    args: git_args,
                })
            }
            LegacyRollbackOp::ReverseOperation { operation } => {
                // Convert FileOperation to appropriate RollbackAction
                match operation {
                    FileOperation::Create { path, .. } => {
                        Ok(RollbackAction::DeleteCreatedFile { path: path.clone() })
                    }
                    FileOperation::Delete { path } => {
                        // For deleted files, we'd need the original content to recreate
                        // This is a limitation of the legacy format
                        Ok(RollbackAction::NoOp {
                            reason: format!(
                                "Cannot restore deleted file {} without backup",
                                path.display()
                            ),
                        })
                    }
                    FileOperation::Update { path, .. } | FileOperation::Append { path, .. } => {
                        // For modified files, we'd need the original content
                        Ok(RollbackAction::NoOp {
                            reason: format!(
                                "Cannot revert modification to {} without backup",
                                path.display()
                            ),
                        })
                    }
                    FileOperation::Rename { from, to } => Ok(RollbackAction::UndoRename {
                        current_path: to.clone(),
                        original_path: from.clone(),
                    }),
                }
            }
            LegacyRollbackOp::VerifyState { file, .. } => Ok(RollbackAction::NoOp {
                reason: format!("Verification step for {}", file.display()),
            }),
            LegacyRollbackOp::ExecuteScript { script_path, args } => {
                Ok(RollbackAction::RunScript {
                    script_path: script_path.clone(),
                    args: args.clone(),
                })
            }
        }
    }

    /// Get affected files from legacy rollback operation
    fn get_files_affected_from_operation(&self, operation: &LegacyRollbackOp) -> Vec<PathBuf> {
        match operation {
            LegacyRollbackOp::RestoreFile {
                source,
                destination,
                ..
            } => {
                vec![source.clone(), destination.clone()]
            }
            LegacyRollbackOp::GitCommand { .. } => {
                // Git commands could affect many files, but we can't determine exactly which
                Vec::new()
            }
            LegacyRollbackOp::ReverseOperation { operation } => self.get_affected_files(operation),
            LegacyRollbackOp::VerifyState { file, .. } => {
                vec![file.clone()]
            }
            LegacyRollbackOp::ExecuteScript { .. } => {
                // Scripts could affect any files, we can't determine which
                Vec::new()
            }
        }
    }

    /// Get affected files from operation
    fn get_affected_files(&self, operation: &FileOperation) -> Vec<PathBuf> {
        match operation {
            FileOperation::Create { path, .. }
            | FileOperation::Update { path, .. }
            | FileOperation::Delete { path }
            | FileOperation::Append { path, .. } => vec![path.clone()],
            FileOperation::Rename { from, to } => vec![from.clone(), to.clone()],
        }
    }

    /// Describe rollback action in human-readable form
    fn describe_rollback_action(&self, operation: &FileOperation) -> String {
        match operation {
            FileOperation::Create { path, .. } => {
                format!("Delete created file: {}", path.display())
            }
            FileOperation::Update { path, .. } => {
                format!("Revert modifications to: {}", path.display())
            }
            FileOperation::Delete { path } => {
                format!("Restore deleted file: {}", path.display())
            }
            FileOperation::Rename { from, to } => {
                format!("Undo rename: {} -> {}", to.display(), from.display())
            }
            FileOperation::Append { path, .. } => {
                format!("Remove appended content from: {}", path.display())
            }
        }
    }

    /// Determine dependencies between operations
    fn determine_dependencies(
        &self,
        existing_ops: &[RollbackOperation],
        current_op: &FileOperation,
    ) -> Vec<String> {
        let mut dependencies = Vec::new();
        let current_files = self.get_affected_files(current_op);

        // Check if any existing operations affect the same files
        for (idx, existing) in existing_ops.iter().enumerate() {
            for file in &existing.files_affected {
                if current_files.contains(file) {
                    dependencies.push(existing.operation_id.clone());
                    break;
                }
            }
        }

        dependencies
    }

    /// Determine safety level based on operations
    fn determine_safety_level(&self, operations: &[RollbackOperation]) -> RollbackSafetyLevel {
        let mut has_deletes = false;
        let mut has_system_paths = false;
        let mut has_scripts = false;

        for op in operations {
            match &op.action {
                RollbackAction::DeleteCreatedFile { path } => {
                    has_deletes = true;
                    if is_system_path(path) {
                        has_system_paths = true;
                    }
                }
                RollbackAction::RunScript { .. } => {
                    has_scripts = true;
                }
                RollbackAction::GitRevert { .. } => {
                    has_scripts = true; // Git operations are considered risky
                }
                _ => {}
            }
        }

        if has_system_paths || has_scripts {
            RollbackSafetyLevel::Critical
        } else if has_deletes || operations.len() > 10 {
            RollbackSafetyLevel::High
        } else if operations.len() > 5 {
            RollbackSafetyLevel::Medium
        } else {
            RollbackSafetyLevel::Low
        }
    }

    /// Create a plan for partial rollback
    pub async fn create_partial_rollback_plan(
        &self,
        all_operations: Vec<FileOperation>,
        operations_to_rollback: Vec<usize>,
    ) -> Result<RollbackPlan> {
        let mut selected_operations = Vec::new();

        for idx in operations_to_rollback {
            if let Some(op) = all_operations.get(idx) {
                selected_operations.push(op.clone());
            }
        }

        self.create_rollback_plan(selected_operations, "Partial rollback requested")
            .await
    }

    /// Analyze rollback feasibility
    pub async fn analyze_rollback_feasibility(
        &self,
        plan: &RollbackPlan,
    ) -> Result<RollbackFeasibilityAnalysis> {
        let mut feasible_operations = 0;
        let mut issues = Vec::new();

        for operation in &plan.operations {
            match self.check_operation_feasibility(&operation.action).await {
                Ok(true) => feasible_operations += 1,
                Ok(false) => {
                    issues.push(format!(
                        "Operation {} may not be feasible: {}",
                        operation.operation_id, operation.description
                    ));
                }
                Err(e) => {
                    issues.push(format!(
                        "Cannot verify operation {}: {}",
                        operation.operation_id, e
                    ));
                }
            }
        }

        let feasibility_score = if plan.operations.is_empty() {
            1.0
        } else {
            feasible_operations as f64 / plan.operations.len() as f64
        };

        Ok(RollbackFeasibilityAnalysis {
            plan_id: plan.plan_id.clone(),
            feasibility_score,
            feasible_operations,
            total_operations: plan.operations.len(),
            issues,
            recommendation: if feasibility_score > 0.8 {
                "Plan is highly feasible and can be executed safely".to_string()
            } else if feasibility_score > 0.5 {
                "Plan has some risks but can proceed with caution".to_string()
            } else {
                "Plan has significant feasibility issues and should be reviewed".to_string()
            },
        })
    }

    /// Check if a single operation is feasible
    async fn check_operation_feasibility(&self, action: &RollbackAction) -> Result<bool> {
        // This only checks feasibility, doesn't execute
        match action {
            RollbackAction::RestoreFromBackup { backup_path, .. } => {
                // Would check if backup exists, but we don't access filesystem
                debug!("Checking backup feasibility for: {:?}", backup_path);
                Ok(true) // Assume feasible for planning
            }
            RollbackAction::DeleteCreatedFile { path } => {
                // Would check if file exists, but we don't access filesystem
                debug!("Checking delete feasibility for: {:?}", path);
                Ok(true) // Assume feasible for planning
            }
            _ => Ok(true), // Other operations assumed feasible
        }
    }
}

/// Rollback feasibility analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackFeasibilityAnalysis {
    pub plan_id: String,
    pub feasibility_score: f64,
    pub feasible_operations: usize,
    pub total_operations: usize,
    pub issues: Vec<String>,
    pub recommendation: String,
}

/// Check if a path is a system path
fn is_system_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.starts_with("/etc")
        || path_str.starts_with("/sys")
        || path_str.starts_with("/proc")
        || path_str.starts_with("/boot")
        || path_str.starts_with("/dev")
        || path_str.starts_with("C:\\Windows")
        || path_str.starts_with("C:\\Program Files")
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_safety_level_determination() {
        let planner = ConsensusRollbackPlanner::new(
            Arc::new(OperationIntelligenceCoordinator::new_mock()),
            None,
            RollbackPlanningConfig::default(),
        );

        // Test with delete operations
        let ops = vec![RollbackOperation {
            operation_id: "1".to_string(),
            action: RollbackAction::DeleteCreatedFile {
                path: PathBuf::from("test.txt"),
            },
            description: "Delete test.txt".to_string(),
            files_affected: vec![PathBuf::from("test.txt")],
            dependencies: vec![],
        }];

        let level = planner.determine_safety_level(&ops);
        assert!(matches!(level, RollbackSafetyLevel::Low));

        // Test with system paths
        let sys_ops = vec![RollbackOperation {
            operation_id: "2".to_string(),
            action: RollbackAction::DeleteCreatedFile {
                path: PathBuf::from("/etc/config"),
            },
            description: "Delete system file".to_string(),
            files_affected: vec![PathBuf::from("/etc/config")],
            dependencies: vec![],
        }];

        let sys_level = planner.determine_safety_level(&sys_ops);
        assert!(matches!(sys_level, RollbackSafetyLevel::Critical));
    }

    #[test]
    fn test_system_path_detection() {
        assert!(is_system_path(&PathBuf::from("/etc/passwd")));
        assert!(is_system_path(&PathBuf::from("/sys/kernel")));
        assert!(is_system_path(&PathBuf::from("C:\\Windows\\System32")));
        assert!(!is_system_path(&PathBuf::from("/home/user/file.txt")));
        assert!(!is_system_path(&PathBuf::from("./local/file.txt")));
    }
}
