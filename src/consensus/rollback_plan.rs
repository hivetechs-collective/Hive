// Rollback Plan Generation and Validation
// This module creates and validates rollback plans for file operations

use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};
use tracing::{debug, info, warn, error};

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::ai_operation_parser::FileOperationWithMetadata;
use crate::consensus::file_executor::{ExecutionResult, ExecutionSummary};
use crate::core::error::HiveError;

/// Rollback plan for a set of file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    /// Unique identifier for this rollback plan
    pub plan_id: String,
    
    /// Operations that can be rolled back
    pub rollback_operations: Vec<RollbackOperation>,
    
    /// Operations that cannot be rolled back (with reasons)
    pub non_rollbackable: Vec<NonRollbackableOperation>,
    
    /// Estimated time to complete rollback
    pub estimated_duration_ms: u64,
    
    /// Risk assessment for the rollback
    pub risk_assessment: RollbackRiskAssessment,
    
    /// Validation status
    pub validation_status: ValidationStatus,
    
    /// Creation timestamp
    pub created_at: SystemTime,
    
    /// Checkpoints for partial rollback
    pub checkpoints: Vec<RollbackCheckpoint>,
}

/// A single rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackOperation {
    /// ID of the original operation being rolled back
    pub original_operation_id: usize,
    
    /// The rollback action to perform
    pub rollback_action: RollbackAction,
    
    /// Priority for rollback (lower = higher priority)
    pub priority: i32,
    
    /// Dependencies on other rollback operations
    pub depends_on: Vec<usize>,
    
    /// Backup data needed for rollback
    pub backup_data: Option<BackupData>,
    
    /// Validation steps before rollback
    pub validation_steps: Vec<ValidationStep>,
}

/// Types of rollback actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackAction {
    /// Restore file from backup
    RestoreFromBackup {
        path: PathBuf,
        backup_path: PathBuf,
    },
    
    /// Delete a created file
    DeleteCreatedFile {
        path: PathBuf,
    },
    
    /// Recreate a deleted file
    RecreateDeletedFile {
        path: PathBuf,
        content: String,
        permissions: Option<u32>,
    },
    
    /// Revert a file modification
    RevertModification {
        path: PathBuf,
        original_content: String,
    },
    
    /// Undo a rename operation
    UndoRename {
        current_path: PathBuf,
        original_path: PathBuf,
    },
    
    /// Run a custom rollback script
    RunScript {
        script_path: PathBuf,
        args: Vec<String>,
    },
    
    /// No-op (for operations that don't need rollback)
    NoOp {
        reason: String,
    },
}

/// Operations that cannot be rolled back
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonRollbackableOperation {
    pub operation_id: usize,
    pub operation: FileOperation,
    pub reason: NonRollbackableReason,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NonRollbackableReason {
    /// Original data was not backed up
    NoBackup,
    
    /// Operation has cascading effects
    CascadingEffects,
    
    /// External system changes
    ExternalDependencies,
    
    /// Data loss would occur
    DataLoss,
    
    /// Insufficient permissions
    PermissionDenied,
    
    /// Custom reason
    Custom(String),
}

/// Backup data for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    /// Path to backup file
    pub backup_path: PathBuf,
    
    /// Original file content (if small enough)
    pub inline_content: Option<String>,
    
    /// File metadata
    pub metadata: FileMetadata,
    
    /// Checksum for validation
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub size: u64,
    pub permissions: Option<u32>,
    pub modified_time: SystemTime,
    pub created_time: Option<SystemTime>,
}

/// Validation step before rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStep {
    pub step_type: ValidationType,
    pub description: String,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    /// Check if file exists
    FileExists,
    
    /// Verify file checksum
    ChecksumMatch,
    
    /// Check available disk space
    DiskSpace,
    
    /// Verify no active file handles
    NoActiveHandles,
    
    /// Custom validation
    Custom,
}

/// Risk assessment for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRiskAssessment {
    /// Overall risk level (0.0 - 1.0)
    pub overall_risk: f32,
    
    /// Individual risk factors
    pub risk_factors: Vec<RiskFactor>,
    
    /// Recommended precautions
    pub precautions: Vec<String>,
    
    /// Estimated success probability
    pub success_probability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: RiskFactorType,
    pub severity: RiskSeverity,
    pub description: String,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactorType {
    DataLoss,
    SystemInstability,
    DependencyBreakage,
    PermissionIssues,
    TimingConstraints,
    Custom(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Validation status of the rollback plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatus {
    pub is_valid: bool,
    pub validation_time: SystemTime,
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_type: ValidationIssueType,
    pub severity: RiskSeverity,
    pub description: String,
    pub affected_operations: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssueType {
    MissingBackup,
    CircularDependency,
    InsufficientPermissions,
    ResourceUnavailable,
    InconsistentState,
}

/// Checkpoint for partial rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackCheckpoint {
    pub checkpoint_id: String,
    pub operations_completed: Vec<usize>,
    pub state_snapshot: StateSnapshot,
    pub can_resume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub timestamp: SystemTime,
    pub file_states: HashMap<PathBuf, FileState>,
    pub operation_results: HashMap<usize, OperationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub exists: bool,
    pub checksum: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    Success,
    Failed(String),
    Skipped(String),
    InProgress,
}

/// Rollback plan generator
pub struct RollbackPlanGenerator {
    /// Configuration for rollback generation
    config: RollbackConfig,
    
    /// Backup manager for accessing backup data
    backup_manager: BackupManager,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackConfig {
    /// Maximum inline content size (bytes)
    pub max_inline_content_size: usize,
    
    /// Include detailed validation steps
    pub detailed_validation: bool,
    
    /// Generate checkpoints for large rollbacks
    pub enable_checkpoints: bool,
    
    /// Checkpoint interval (number of operations)
    pub checkpoint_interval: usize,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            max_inline_content_size: 1024 * 1024, // 1MB
            detailed_validation: true,
            enable_checkpoints: true,
            checkpoint_interval: 10,
        }
    }
}

/// Simple backup manager (placeholder)
struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    fn new(backup_dir: PathBuf) -> Self {
        Self { backup_dir }
    }
    
    fn get_backup_path(&self, original_path: &Path) -> PathBuf {
        // Simple implementation - in reality would be more sophisticated
        let file_name = original_path.file_name()
            .unwrap_or_default()
            .to_string_lossy();
        self.backup_dir.join(format!("{}.backup", file_name))
    }
    
    fn has_backup(&self, path: &Path) -> bool {
        self.get_backup_path(path).exists()
    }
}

impl RollbackPlanGenerator {
    pub fn new(config: RollbackConfig, backup_dir: PathBuf) -> Self {
        Self {
            config,
            backup_manager: BackupManager::new(backup_dir),
        }
    }
    
    /// Generate a rollback plan for executed operations
    pub async fn generate_plan(
        &self,
        operations: &[FileOperationWithMetadata],
        execution_results: &[ExecutionResult],
    ) -> Result<RollbackPlan> {
        info!("🔄 Generating rollback plan for {} operations", operations.len());
        
        let mut rollback_operations = Vec::new();
        let mut non_rollbackable = Vec::new();
        
        // Process each operation in reverse order
        for (idx, (op_meta, result)) in operations.iter()
            .zip(execution_results.iter())
            .enumerate()
            .rev() 
        {
            match self.create_rollback_operation(idx, op_meta, result).await {
                Ok(Some(rollback_op)) => rollback_operations.push(rollback_op),
                Ok(None) => {
                    // Operation doesn't need rollback
                }
                Err(reason) => {
                    let mitigation = self.suggest_mitigation(&op_meta.operation, &reason);
                    non_rollbackable.push(NonRollbackableOperation {
                        operation_id: idx,
                        operation: op_meta.operation.clone(),
                        reason,
                        mitigation,
                    });
                }
            }
        }
        
        // Calculate dependencies
        self.calculate_rollback_dependencies(&mut rollback_operations);
        
        // Generate checkpoints if enabled
        let checkpoints = if self.config.enable_checkpoints {
            self.generate_checkpoints(&rollback_operations)
        } else {
            Vec::new()
        };
        
        // Assess risks
        let risk_assessment = self.assess_rollback_risks(&rollback_operations, &non_rollbackable);
        
        // Validate the plan
        let validation_status = self.validate_plan(&rollback_operations, &non_rollbackable);
        
        // Estimate duration
        let estimated_duration_ms = self.estimate_rollback_duration(&rollback_operations);
        
        Ok(RollbackPlan {
            plan_id: uuid::Uuid::new_v4().to_string(),
            rollback_operations,
            non_rollbackable,
            estimated_duration_ms,
            risk_assessment,
            validation_status,
            created_at: SystemTime::now(),
            checkpoints,
        })
    }
    
    /// Create a rollback operation for a single file operation
    async fn create_rollback_operation(
        &self,
        operation_id: usize,
        op_meta: &FileOperationWithMetadata,
        result: &ExecutionResult,
    ) -> Result<Option<RollbackOperation>, NonRollbackableReason> {
        if !result.success {
            // Failed operations don't need rollback
            return Ok(None);
        }
        
        let rollback_action = match &op_meta.operation {
            FileOperation::Create { path, .. } => {
                RollbackAction::DeleteCreatedFile {
                    path: path.clone(),
                }
            }
            
            FileOperation::Update { path, content } => {
                if self.backup_manager.has_backup(path) {
                    RollbackAction::RestoreFromBackup {
                        path: path.clone(),
                        backup_path: self.backup_manager.get_backup_path(path),
                    }
                } else {
                    return Err(NonRollbackableReason::NoBackup);
                }
            }
            
            FileOperation::Delete { path } => {
                if self.backup_manager.has_backup(path) {
                    // For now, assume we can restore from backup
                    // In reality, would read the backup content
                    RollbackAction::RecreateDeletedFile {
                        path: path.clone(),
                        content: String::new(), // Would be filled from backup
                        permissions: None,
                    }
                } else {
                    return Err(NonRollbackableReason::DataLoss);
                }
            }
            
            FileOperation::Rename { from, to } => {
                RollbackAction::UndoRename {
                    current_path: to.clone(),
                    original_path: from.clone(),
                }
            }
            
            FileOperation::Append { path, .. } => {
                if self.backup_manager.has_backup(path) {
                    RollbackAction::RestoreFromBackup {
                        path: path.clone(),
                        backup_path: self.backup_manager.get_backup_path(path),
                    }
                } else {
                    return Err(NonRollbackableReason::NoBackup);
                }
            }
        };
        
        // Create validation steps
        let validation_steps = self.create_validation_steps(&rollback_action);
        
        Ok(Some(RollbackOperation {
            original_operation_id: operation_id,
            rollback_action,
            priority: self.calculate_rollback_priority(&op_meta.operation),
            depends_on: Vec::new(), // Will be calculated later
            backup_data: None, // Would be populated from backup manager
            validation_steps,
        }))
    }
    
    /// Create validation steps for a rollback action
    fn create_validation_steps(&self, action: &RollbackAction) -> Vec<ValidationStep> {
        let mut steps = Vec::new();
        
        match action {
            RollbackAction::RestoreFromBackup { backup_path, .. } => {
                steps.push(ValidationStep {
                    step_type: ValidationType::FileExists,
                    description: format!("Verify backup exists at {:?}", backup_path),
                    critical: true,
                });
                steps.push(ValidationStep {
                    step_type: ValidationType::ChecksumMatch,
                    description: "Verify backup integrity".to_string(),
                    critical: true,
                });
            }
            
            RollbackAction::DeleteCreatedFile { path } => {
                steps.push(ValidationStep {
                    step_type: ValidationType::FileExists,
                    description: format!("Verify file exists at {:?}", path),
                    critical: true,
                });
                steps.push(ValidationStep {
                    step_type: ValidationType::NoActiveHandles,
                    description: "Ensure no processes are using the file".to_string(),
                    critical: false,
                });
            }
            
            RollbackAction::RecreateDeletedFile { path, .. } => {
                steps.push(ValidationStep {
                    step_type: ValidationType::FileExists,
                    description: format!("Verify file does not exist at {:?}", path),
                    critical: true,
                });
                steps.push(ValidationStep {
                    step_type: ValidationType::DiskSpace,
                    description: "Verify sufficient disk space".to_string(),
                    critical: true,
                });
            }
            
            _ => {}
        }
        
        steps
    }
    
    /// Calculate priority for rollback operations
    fn calculate_rollback_priority(&self, operation: &FileOperation) -> i32 {
        match operation {
            FileOperation::Delete { .. } => 1, // Highest priority - restore deleted files first
            FileOperation::Rename { .. } => 2,
            FileOperation::Update { .. } => 3,
            FileOperation::Append { .. } => 4,
            FileOperation::Create { .. } => 5, // Lowest priority - delete created files last
        }
    }
    
    /// Calculate dependencies between rollback operations
    fn calculate_rollback_dependencies(&self, operations: &mut [RollbackOperation]) {
        // Simple implementation - in reality would be more sophisticated
        // For now, just ensure operations on the same file are ordered
        let mut path_to_ops: HashMap<PathBuf, Vec<usize>> = HashMap::new();
        
        for (idx, op) in operations.iter().enumerate() {
            if let Some(path) = self.get_rollback_path(&op.rollback_action) {
                path_to_ops.entry(path).or_insert_with(Vec::new).push(idx);
            }
        }
        
        // Add dependencies for operations on the same file
        for ops in path_to_ops.values() {
            if ops.len() > 1 {
                for i in 1..ops.len() {
                    operations[ops[i]].depends_on.push(ops[i-1]);
                }
            }
        }
    }
    
    /// Get the path affected by a rollback action
    fn get_rollback_path(&self, action: &RollbackAction) -> Option<PathBuf> {
        match action {
            RollbackAction::RestoreFromBackup { path, .. } |
            RollbackAction::DeleteCreatedFile { path } |
            RollbackAction::RecreateDeletedFile { path, .. } |
            RollbackAction::RevertModification { path, .. } => Some(path.clone()),
            RollbackAction::UndoRename { original_path, .. } => Some(original_path.clone()),
            _ => None,
        }
    }
    
    /// Generate checkpoints for the rollback plan
    fn generate_checkpoints(&self, operations: &[RollbackOperation]) -> Vec<RollbackCheckpoint> {
        let mut checkpoints = Vec::new();
        
        for (idx, chunk) in operations.chunks(self.config.checkpoint_interval).enumerate() {
            let operations_completed: Vec<usize> = operations[..idx * self.config.checkpoint_interval]
                .iter()
                .map(|op| op.original_operation_id)
                .collect();
            
            checkpoints.push(RollbackCheckpoint {
                checkpoint_id: format!("checkpoint_{}", idx),
                operations_completed,
                state_snapshot: StateSnapshot {
                    timestamp: SystemTime::now(),
                    file_states: HashMap::new(), // Would be populated during execution
                    operation_results: HashMap::new(),
                },
                can_resume: true,
            });
        }
        
        checkpoints
    }
    
    /// Assess risks for the rollback plan
    fn assess_rollback_risks(
        &self,
        operations: &[RollbackOperation],
        non_rollbackable: &[NonRollbackableOperation],
    ) -> RollbackRiskAssessment {
        let mut risk_factors = Vec::new();
        let mut overall_risk: f32 = 0.0;
        
        // Risk from non-rollbackable operations
        if !non_rollbackable.is_empty() {
            risk_factors.push(RiskFactor {
                factor_type: RiskFactorType::DataLoss,
                severity: RiskSeverity::High,
                description: format!("{} operations cannot be rolled back", non_rollbackable.len()),
                mitigation: Some("Manual intervention may be required".to_string()),
            });
            overall_risk += 0.3;
        }
        
        // Risk from complex dependencies
        let has_complex_deps = operations.iter()
            .any(|op| op.depends_on.len() > 2);
        if has_complex_deps {
            risk_factors.push(RiskFactor {
                factor_type: RiskFactorType::DependencyBreakage,
                severity: RiskSeverity::Medium,
                description: "Complex dependencies between rollback operations".to_string(),
                mitigation: Some("Execute rollback in careful sequence".to_string()),
            });
            overall_risk += 0.2;
        }
        
        let precautions = vec![
            "Create a full system backup before rollback".to_string(),
            "Ensure no active processes are using affected files".to_string(),
            "Monitor rollback progress closely".to_string(),
        ];
        
        let success_probability = 1.0 - overall_risk.min(0.9f32);
        
        RollbackRiskAssessment {
            overall_risk: overall_risk.min(1.0f32),
            risk_factors,
            precautions,
            success_probability,
        }
    }
    
    /// Validate the rollback plan
    fn validate_plan(
        &self,
        operations: &[RollbackOperation],
        non_rollbackable: &[NonRollbackableOperation],
    ) -> ValidationStatus {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        
        // Check for circular dependencies
        if self.has_circular_dependencies(operations) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::CircularDependency,
                severity: RiskSeverity::Critical,
                description: "Circular dependencies detected in rollback plan".to_string(),
                affected_operations: Vec::new(), // Would be populated with actual operations
            });
        }
        
        // Warn about non-rollbackable operations
        if !non_rollbackable.is_empty() {
            warnings.push(format!(
                "{} operations cannot be rolled back and may require manual intervention",
                non_rollbackable.len()
            ));
        }
        
        ValidationStatus {
            is_valid: issues.is_empty(),
            validation_time: SystemTime::now(),
            issues,
            warnings,
        }
    }
    
    /// Check for circular dependencies
    fn has_circular_dependencies(&self, operations: &[RollbackOperation]) -> bool {
        // Simple DFS-based cycle detection
        let mut visited = vec![false; operations.len()];
        let mut rec_stack = vec![false; operations.len()];
        
        for i in 0..operations.len() {
            if !visited[i] && self.has_cycle_dfs(i, operations, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        
        false
    }
    
    fn has_cycle_dfs(
        &self,
        node: usize,
        operations: &[RollbackOperation],
        visited: &mut [bool],
        rec_stack: &mut [bool],
    ) -> bool {
        visited[node] = true;
        rec_stack[node] = true;
        
        for &dep in &operations[node].depends_on {
            if !visited[dep] && self.has_cycle_dfs(dep, operations, visited, rec_stack) {
                return true;
            } else if rec_stack[dep] {
                return true;
            }
        }
        
        rec_stack[node] = false;
        false
    }
    
    /// Estimate duration for rollback
    fn estimate_rollback_duration(&self, operations: &[RollbackOperation]) -> u64 {
        // Simple estimate: 50ms per operation + overhead
        let base_time = operations.len() as u64 * 50;
        let overhead = 100; // Fixed overhead
        base_time + overhead
    }
    
    /// Suggest mitigation for non-rollbackable operations
    fn suggest_mitigation(
        &self,
        operation: &FileOperation,
        reason: &NonRollbackableReason,
    ) -> Option<String> {
        match (operation, reason) {
            (FileOperation::Delete { path }, NonRollbackableReason::NoBackup) => {
                Some(format!("Manually restore {:?} from external backup", path))
            }
            (_, NonRollbackableReason::DataLoss) => {
                Some("Check version control or external backups".to_string())
            }
            _ => None,
        }
    }
}

/// Execute a rollback plan
pub struct RollbackExecutor {
    config: RollbackConfig,
}

impl RollbackExecutor {
    pub fn new(config: RollbackConfig) -> Self {
        Self { config }
    }
    
    /// Execute a rollback plan
    pub async fn execute(
        &self,
        plan: &RollbackPlan,
        checkpoint: Option<&RollbackCheckpoint>,
    ) -> Result<RollbackExecutionResult> {
        info!("🔄 Executing rollback plan {}", plan.plan_id);
        
        if !plan.validation_status.is_valid {
            return Err(anyhow!("Cannot execute invalid rollback plan"));
        }
        
        let mut completed_operations = Vec::new();
        let mut failed_operations = Vec::new();
        let mut skipped_operations = Vec::new();
        
        // Determine starting point
        let start_idx = checkpoint
            .map(|cp| cp.operations_completed.len())
            .unwrap_or(0);
        
        // Execute operations in order
        for (idx, operation) in plan.rollback_operations.iter().enumerate().skip(start_idx) {
            // Check dependencies
            let deps_satisfied = operation.depends_on.iter()
                .all(|&dep| completed_operations.contains(&dep));
            
            if !deps_satisfied {
                skipped_operations.push((idx, "Dependencies not satisfied".to_string()));
                continue;
            }
            
            // Validate before execution
            if !self.validate_operation(operation).await {
                failed_operations.push((idx, "Validation failed".to_string()));
                continue;
            }
            
            // Execute the rollback operation
            match self.execute_rollback_operation(operation).await {
                Ok(_) => {
                    completed_operations.push(idx);
                    info!("✅ Completed rollback operation {}", idx);
                }
                Err(e) => {
                    failed_operations.push((idx, e.to_string()));
                    error!("❌ Failed rollback operation {}: {}", idx, e);
                    
                    // Decide whether to continue or abort
                    if operation.validation_steps.iter().any(|s| s.critical) {
                        return Err(anyhow!("Critical rollback operation failed"));
                    }
                }
            }
        }
        
        Ok(RollbackExecutionResult {
            plan_id: plan.plan_id.clone(),
            completed_operations,
            failed_operations,
            skipped_operations,
            execution_time: SystemTime::now(),
            final_state: self.capture_final_state(&plan.rollback_operations).await,
        })
    }
    
    /// Validate a single rollback operation
    async fn validate_operation(&self, operation: &RollbackOperation) -> bool {
        for step in &operation.validation_steps {
            if !self.perform_validation_step(step, &operation.rollback_action).await {
                if step.critical {
                    return false;
                }
            }
        }
        true
    }
    
    /// Perform a validation step
    async fn perform_validation_step(
        &self,
        step: &ValidationStep,
        action: &RollbackAction,
    ) -> bool {
        match step.step_type {
            ValidationType::FileExists => {
                if let Some(path) = self.get_action_path(action) {
                    path.exists()
                } else {
                    false
                }
            }
            ValidationType::DiskSpace => {
                // Simplified - would check actual disk space
                true
            }
            _ => true, // Other validations would be implemented
        }
    }
    
    /// Execute a single rollback operation
    async fn execute_rollback_operation(&self, operation: &RollbackOperation) -> Result<()> {
        match &operation.rollback_action {
            RollbackAction::DeleteCreatedFile { path } => {
                std::fs::remove_file(path)
                    .map_err(|e| anyhow!("Failed to delete file: {}", e))?;
            }
            
            RollbackAction::RestoreFromBackup { path, backup_path } => {
                std::fs::copy(backup_path, path)
                    .map_err(|e| anyhow!("Failed to restore from backup: {}", e))?;
            }
            
            RollbackAction::UndoRename { current_path, original_path } => {
                std::fs::rename(current_path, original_path)
                    .map_err(|e| anyhow!("Failed to undo rename: {}", e))?;
            }
            
            RollbackAction::RecreateDeletedFile { path, content, .. } => {
                std::fs::write(path, content)
                    .map_err(|e| anyhow!("Failed to recreate file: {}", e))?;
            }
            
            RollbackAction::RevertModification { path, original_content } => {
                std::fs::write(path, original_content)
                    .map_err(|e| anyhow!("Failed to revert modification: {}", e))?;
            }
            
            RollbackAction::RunScript { script_path, args } => {
                std::process::Command::new(script_path)
                    .args(args)
                    .status()
                    .map_err(|e| anyhow!("Failed to run rollback script: {}", e))?;
            }
            
            RollbackAction::NoOp { reason } => {
                debug!("No-op rollback: {}", reason);
            }
        }
        
        Ok(())
    }
    
    /// Get path from rollback action
    fn get_action_path<'a>(&self, action: &'a RollbackAction) -> Option<&'a Path> {
        match action {
            RollbackAction::DeleteCreatedFile { path } |
            RollbackAction::RestoreFromBackup { path, .. } |
            RollbackAction::RecreateDeletedFile { path, .. } |
            RollbackAction::RevertModification { path, .. } => Some(path),
            RollbackAction::UndoRename { current_path, .. } => Some(current_path),
            _ => None,
        }
    }
    
    /// Capture final state after rollback
    async fn capture_final_state(&self, operations: &[RollbackOperation]) -> StateSnapshot {
        let mut file_states = HashMap::new();
        
        for operation in operations {
            if let Some(path) = self.get_action_path(&operation.rollback_action) {
                let state = FileState {
                    exists: path.exists(),
                    checksum: None, // Would calculate actual checksum
                    size: path.metadata().ok().map(|m| m.len()),
                };
                file_states.insert(path.to_path_buf(), state);
            }
        }
        
        StateSnapshot {
            timestamp: SystemTime::now(),
            file_states,
            operation_results: HashMap::new(),
        }
    }
}

/// Result of rollback execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackExecutionResult {
    pub plan_id: String,
    pub completed_operations: Vec<usize>,
    pub failed_operations: Vec<(usize, String)>,
    pub skipped_operations: Vec<(usize, String)>,
    pub execution_time: SystemTime,
    pub final_state: StateSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rollback_priority() {
        let generator = RollbackPlanGenerator::new(
            RollbackConfig::default(),
            PathBuf::from("/tmp/backups"),
        );
        
        assert_eq!(
            generator.calculate_rollback_priority(&FileOperation::Delete { 
                path: PathBuf::from("test.rs") 
            }),
            1
        );
        
        assert_eq!(
            generator.calculate_rollback_priority(&FileOperation::Create { 
                path: PathBuf::from("test.rs"),
                content: String::new(),
            }),
            5
        );
    }
    
    #[test]
    fn test_circular_dependency_detection() {
        let generator = RollbackPlanGenerator::new(
            RollbackConfig::default(),
            PathBuf::from("/tmp/backups"),
        );
        
        let mut operations = vec![
            RollbackOperation {
                original_operation_id: 0,
                rollback_action: RollbackAction::NoOp { reason: "test".to_string() },
                priority: 1,
                depends_on: vec![1], // Depends on op 1
                backup_data: None,
                validation_steps: vec![],
            },
            RollbackOperation {
                original_operation_id: 1,
                rollback_action: RollbackAction::NoOp { reason: "test".to_string() },
                priority: 2,
                depends_on: vec![0], // Depends on op 0 - circular!
                backup_data: None,
                validation_steps: vec![],
            },
        ];
        
        assert!(generator.has_circular_dependencies(&operations));
    }
}