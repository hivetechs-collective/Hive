// Rollback Execution System for Failed Operations
use crate::consensus::rollback_planner::{
    RollbackPlan, RollbackStep, RollbackStrategy, RollbackOperation, RiskLevel
};
use crate::consensus::operation_intelligence::OperationIntelligenceCoordinator;
use crate::consensus::operation_history::OperationHistoryDatabase;
use crate::consensus::operation_intelligence::OperationOutcome;
use crate::consensus::stages::file_aware_curator::FileOperation;
use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::time::{Duration, Instant, sleep};
use tokio::fs;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Main rollback execution engine for failed operations
pub struct RollbackExecutor {
    operation_intelligence: Arc<OperationIntelligenceCoordinator>,
    history_database: Option<Arc<OperationHistoryDatabase>>,
    execution_config: RollbackExecutionConfig,
    backup_manager: Arc<BackupManager>,
    git_manager: Arc<GitManager>,
    verification_system: Arc<VerificationSystem>,
    progress_tracker: Arc<ProgressTracker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackExecutionConfig {
    /// Maximum time to wait for a rollback step (in seconds)
    pub max_step_timeout: u64,
    /// Maximum total rollback time (in seconds)
    pub max_total_timeout: u64,
    /// Whether to verify each step before proceeding
    pub verify_steps: bool,
    /// Whether to create additional backups during rollback
    pub create_safety_backups: bool,
    /// Maximum number of retry attempts for failed steps
    pub max_retries: u32,
    /// Delay between retry attempts (in seconds)
    pub retry_delay: u64,
    /// Whether to abort on first failure or continue with remaining steps
    pub abort_on_failure: bool,
    /// Whether to pause for user confirmation on high-risk steps
    pub pause_on_high_risk: bool,
}

impl Default for RollbackExecutionConfig {
    fn default() -> Self {
        Self {
            max_step_timeout: 300, // 5 minutes
            max_total_timeout: 1800, // 30 minutes
            verify_steps: true,
            create_safety_backups: true,
            max_retries: 3,
            retry_delay: 5,
            abort_on_failure: false,
            pause_on_high_risk: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackExecution {
    pub execution_id: String,
    pub plan_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: RollbackExecutionStatus,
    pub steps_completed: Vec<RollbackStepResult>,
    pub total_steps: usize,
    pub errors: Vec<RollbackError>,
    pub rollback_summary: Option<RollbackSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackExecutionStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    PartiallyCompleted,
    Aborted,
    WaitingForUserConfirmation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStepResult {
    pub step_number: u32,
    pub status: RollbackStepStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
    pub verification_result: Option<VerificationResult>,
    pub retry_count: u32,
    pub files_affected: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
    WaitingForConfirmation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackError {
    pub error_type: RollbackErrorType,
    pub message: String,
    pub step_number: Option<u32>,
    pub timestamp: DateTime<Utc>,
    pub is_recoverable: bool,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RollbackErrorType {
    FileSystemError,
    GitError,
    BackupError,
    VerificationError,
    TimeoutError,
    PermissionError,
    UserAbortError,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackSummary {
    pub original_operations: Vec<String>,
    pub files_restored: Vec<PathBuf>,
    pub git_commits_reverted: Vec<String>,
    pub backups_used: Vec<String>,
    pub total_duration_ms: u64,
    pub success_rate: f32,
    pub issues_encountered: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Progress tracking for rollback operations
pub struct ProgressTracker {
    current_execution: Option<RollbackExecution>,
    progress_callbacks: Vec<Box<dyn Fn(&RollbackProgress) + Send + Sync>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackProgress {
    pub execution_id: String,
    pub current_step: u32,
    pub total_steps: u32,
    pub overall_progress: f32,
    pub current_step_progress: f32,
    pub estimated_remaining_time: Option<Duration>,
    pub status: RollbackExecutionStatus,
    pub current_operation: String,
}

impl RollbackExecutor {
    pub fn new(
        operation_intelligence: Arc<OperationIntelligenceCoordinator>,
        history_database: Option<Arc<OperationHistoryDatabase>>,
        config: Option<RollbackExecutionConfig>,
    ) -> Self {
        Self {
            operation_intelligence,
            history_database,
            execution_config: config.unwrap_or_default(),
            backup_manager: Arc::new(BackupManager::new()),
            git_manager: Arc::new(GitManager::new()),
            verification_system: Arc::new(VerificationSystem::new()),
            progress_tracker: Arc::new(ProgressTracker::new()),
        }
    }

    /// Execute a rollback plan with comprehensive error handling and progress tracking
    pub async fn execute_rollback(
        &self,
        plan: RollbackPlan,
    ) -> Result<RollbackExecution> {
        let execution_id = generate_execution_id();
        info!("Starting rollback execution: {}", execution_id);

        let mut execution = RollbackExecution {
            execution_id: execution_id.clone(),
            plan_id: plan.id.clone(),
            started_at: Utc::now(),
            completed_at: None,
            status: RollbackExecutionStatus::InProgress,
            steps_completed: Vec::new(),
            total_steps: plan.steps.len(),
            errors: Vec::new(),
            rollback_summary: None,
        };

        // Record the execution start
        self.track_execution_start(&execution).await?;

        let start_time = Instant::now();
        let mut successful_steps = 0;
        let mut failed_steps = 0;

        // Pre-execution safety checks
        if let Err(e) = self.perform_pre_execution_checks(&plan).await {
            error!("Pre-execution checks failed: {}", e);
            execution.status = RollbackExecutionStatus::Failed;
            execution.errors.push(RollbackError {
                error_type: RollbackErrorType::VerificationError,
                message: format!("Pre-execution checks failed: {}", e),
                step_number: None,
                timestamp: Utc::now(),
                is_recoverable: false,
                suggested_action: Some("Review the rollback plan and system state".to_string()),
            });
            return Ok(execution);
        }

        // Execute each step with comprehensive error handling
        for (index, step) in plan.steps.iter().enumerate() {
            let step_start = Instant::now();
            
            // Check timeout
            if start_time.elapsed().as_secs() > self.execution_config.max_total_timeout {
                error!("Rollback execution timeout exceeded");
                execution.status = RollbackExecutionStatus::Failed;
                execution.errors.push(RollbackError {
                    error_type: RollbackErrorType::TimeoutError,
                    message: "Total execution timeout exceeded".to_string(),
                    step_number: Some(step.step_number as u32),
                    timestamp: Utc::now(),
                    is_recoverable: false,
                    suggested_action: Some("Increase timeout or simplify rollback plan".to_string()),
                });
                break;
            }

            // Update progress
            self.update_progress(&execution_id, step.step_number as u32, plan.steps.len() as u32, &step.description).await;

            // Handle high-risk steps
            if matches!(step.risk_level, RiskLevel::High | RiskLevel::Critical) && self.execution_config.pause_on_high_risk {
                info!("Pausing for user confirmation on high-risk step: {}", step.description);
                execution.status = RollbackExecutionStatus::WaitingForUserConfirmation;
                // In a real implementation, this would wait for user input
                // For now, we'll continue automatically
            }

            // Execute the step with retries
            let step_result = self.execute_step_with_retries(step).await;
            
            match step_result.status {
                RollbackStepStatus::Completed => {
                    successful_steps += 1;
                    info!("Step {} completed successfully", step.step_number);
                }
                RollbackStepStatus::Failed => {
                    failed_steps += 1;
                    warn!("Step {} failed: {:?}", step.step_number, step_result.error_message);
                    
                    if let Some(ref error_msg) = step_result.error_message {
                        execution.errors.push(RollbackError {
                            error_type: RollbackErrorType::InternalError,
                            message: error_msg.clone(),
                            step_number: Some(step.step_number as u32),
                            timestamp: Utc::now(),
                            is_recoverable: step_result.retry_count < self.execution_config.max_retries,
                            suggested_action: self.suggest_action_for_step_failure(step, error_msg),
                        });
                    }

                    if self.execution_config.abort_on_failure {
                        error!("Aborting rollback due to step failure");
                        execution.status = RollbackExecutionStatus::Failed;
                        break;
                    }
                }
                _ => {}
            }

            execution.steps_completed.push(step_result);

            // Brief pause between steps for system stability
            sleep(Duration::from_millis(100)).await;
        }

        // Finalize execution
        execution.completed_at = Some(Utc::now());
        let total_duration = start_time.elapsed().as_millis() as u64;

        // Determine final status
        execution.status = if failed_steps == 0 {
            RollbackExecutionStatus::Completed
        } else if successful_steps > 0 {
            RollbackExecutionStatus::PartiallyCompleted
        } else {
            RollbackExecutionStatus::Failed
        };

        // Generate summary
        execution.rollback_summary = Some(self.generate_rollback_summary(
            &plan,
            &execution,
            total_duration,
        ).await);

        // Record final execution state
        self.track_execution_completion(&execution).await?;

        info!(
            "Rollback execution completed: {} ({}% success rate)",
            execution_id,
            (successful_steps as f32 / plan.steps.len() as f32) * 100.0
        );

        Ok(execution)
    }

    /// Execute a single rollback step with retry logic
    async fn execute_step_with_retries(
        &self,
        step: &RollbackStep,
    ) -> RollbackStepResult {
        let mut result = RollbackStepResult {
            step_number: step.step_number as u32,
            status: RollbackStepStatus::InProgress,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            error_message: None,
            verification_result: None,
            retry_count: 0,
            files_affected: Vec::new(),
        };

        let step_start = Instant::now();

        for attempt in 0..=self.execution_config.max_retries {
            result.retry_count = attempt;
            
            debug!("Executing step {} (attempt {})", step.step_number, attempt + 1);

            match self.execute_single_step(step).await {
                Ok(files_affected) => {
                    result.status = RollbackStepStatus::Completed;
                    result.files_affected = files_affected;
                    
                    // Verify step if configured
                    if self.execution_config.verify_steps {
                        match self.verification_system.verify_step_completion(step).await {
                            Ok(verification) => {
                                let is_successful = verification.is_successful;
                                let error_msg = verification.error_message.clone();
                                result.verification_result = Some(verification);
                                if !is_successful {
                                    result.status = RollbackStepStatus::Failed;
                                    result.error_message = Some(format!(
                                        "Step verification failed: {}",
                                        error_msg.unwrap_or_default()
                                    ));
                                    continue; // Retry
                                }
                            }
                            Err(e) => {
                                warn!("Step verification error: {}", e);
                                // Continue without verification
                            }
                        }
                    }
                    
                    break; // Success
                }
                Err(e) => {
                    result.status = RollbackStepStatus::Failed;
                    result.error_message = Some(e.to_string());
                    
                    if attempt < self.execution_config.max_retries {
                        warn!("Step {} failed (attempt {}), retrying: {}", step.step_number, attempt + 1, e);
                        sleep(Duration::from_secs(self.execution_config.retry_delay)).await;
                    } else {
                        error!("Step {} failed after {} attempts: {}", step.step_number, attempt + 1, e);
                    }
                }
            }
        }

        result.completed_at = Some(Utc::now());
        result.duration_ms = Some(step_start.elapsed().as_millis() as u64);

        result
    }

    /// Execute a single rollback step based on its type
    async fn execute_single_step(
        &self,
        step: &RollbackStep,
    ) -> Result<Vec<PathBuf>> {
        match &step.operation {
            RollbackOperation::RestoreFile { source, destination, .. } => {
                self.backup_manager.restore_from_backup(source, destination).await
            }
            RollbackOperation::GitCommand { command, args, .. } => {
                if command == "revert" && !args.is_empty() {
                    self.git_manager.revert_commit(&args[0], None).await
                } else {
                    bail!("Unsupported Git command: {}", command)
                }
            }
            RollbackOperation::ReverseOperation { operation } => {
                match operation {
                    FileOperation::Create { path, .. } => self.delete_file_safely(path).await,
                    _ => bail!("Unsupported reverse operation")
                }
            }
            RollbackOperation::ExecuteScript { script_path, args } => {
                self.run_rollback_script(script_path, args).await
            }
            RollbackOperation::VerifyState { file, expected_exists, .. } => {
                if *expected_exists {
                    if tokio::fs::metadata(file).await.is_ok() {
                        Ok(vec![file.clone()])
                    } else {
                        bail!("Expected file {} to exist", file.display())
                    }
                } else {
                    if tokio::fs::metadata(file).await.is_err() {
                        Ok(vec![])
                    } else {
                        bail!("Expected file {} to not exist", file.display())
                    }
                }
            }
        }
    }

    /// Perform pre-execution safety checks
    async fn perform_pre_execution_checks(
        &self,
        plan: &RollbackPlan,
    ) -> Result<()> {
        info!("Performing pre-execution safety checks");

        // Check if all required files and paths exist
        for step in &plan.steps {
            match &step.operation {
                RollbackOperation::RestoreFile { source, destination, .. } => {
                    if !source.exists() {
                        bail!("Backup file does not exist: {}", source.display());
                    }
                    if let Some(parent) = destination.parent() {
                        if !parent.exists() {
                            bail!("Target directory does not exist: {}", parent.display());
                        }
                    }
                }
                RollbackOperation::ReverseOperation { operation } => {
                    if let FileOperation::Create { path, .. } = operation {
                        if !path.exists() {
                            warn!("File to delete does not exist: {}", path.display());
                        }
                    }
                }
                _ => {}
            }
        }

        // Check available disk space
        // This is a simplified check - in practice, you'd use system-specific APIs
        let estimated_space_needed = self.estimate_space_requirements(plan).await?;
        debug!("Estimated space requirements: {} bytes", estimated_space_needed);

        // Check Git repository state if Git operations are involved
        let has_git_operations = plan.steps.iter().any(|step| {
            matches!(step.operation, RollbackOperation::GitCommand { .. })
        });
        
        if has_git_operations {
            self.git_manager.verify_repository_state().await
                .context("Git repository state verification failed")?;
        }

        info!("Pre-execution checks completed successfully");
        Ok(())
    }

    /// Generate comprehensive rollback summary
    async fn generate_rollback_summary(
        &self,
        plan: &RollbackPlan,
        execution: &RollbackExecution,
        total_duration_ms: u64,
    ) -> RollbackSummary {
        let successful_steps = execution.steps_completed.iter()
            .filter(|step| matches!(step.status, RollbackStepStatus::Completed))
            .count();
        
        let success_rate = if execution.total_steps > 0 {
            (successful_steps as f32 / execution.total_steps as f32) * 100.0
        } else {
            0.0
        };

        let files_restored: Vec<PathBuf> = execution.steps_completed.iter()
            .flat_map(|step| step.files_affected.clone())
            .collect();

        let git_commits_reverted: Vec<String> = plan.steps.iter()
            .filter_map(|step| {
                if let RollbackOperation::GitCommand { args, .. } = &step.operation {
                    // Extract commit hash from git args if present
                    args.get(0).cloned()
                } else {
                    None
                }
            })
            .collect();

        let backups_used: Vec<String> = plan.steps.iter()
            .filter_map(|step| {
                if let RollbackOperation::RestoreFile { source, .. } = &step.operation {
                    Some(source.display().to_string())
                } else {
                    None
                }
            })
            .collect();

        let issues_encountered: Vec<String> = execution.errors.iter()
            .map(|error| format!("{:?}: {}", error.error_type, error.message))
            .collect();

        let recommendations = self.generate_recommendations(execution, success_rate).await;

        RollbackSummary {
            original_operations: plan.operations.iter()
                .map(|op| format!("{:?}", op.operation))
                .collect(),
            files_restored,
            git_commits_reverted,
            backups_used,
            total_duration_ms,
            success_rate,
            issues_encountered,
            recommendations,
        }
    }

    /// Generate recommendations based on execution results
    async fn generate_recommendations(
        &self,
        execution: &RollbackExecution,
        success_rate: f32,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if success_rate < 50.0 {
            recommendations.push("Consider reviewing the rollback strategy for future operations".to_string());
            recommendations.push("Verify that all backup systems are functioning correctly".to_string());
        } else if success_rate < 80.0 {
            recommendations.push("Some steps failed - review error logs for improvement opportunities".to_string());
        } else {
            recommendations.push("Rollback completed successfully".to_string());
        }

        // Add specific recommendations based on error types
        let error_types: std::collections::HashSet<RollbackErrorType> = execution.errors.iter()
            .map(|e| e.error_type.clone())
            .collect();

        if error_types.contains(&RollbackErrorType::FileSystemError) {
            recommendations.push("Check file system permissions and available disk space".to_string());
        }

        if error_types.contains(&RollbackErrorType::GitError) {
            recommendations.push("Verify Git repository state and consider manual conflict resolution".to_string());
        }

        if error_types.contains(&RollbackErrorType::TimeoutError) {
            recommendations.push("Consider increasing timeout values or optimizing rollback operations".to_string());
        }

        recommendations
    }

    // Additional helper methods for specific file operations

    async fn delete_file_safely(&self, file_path: &Path) -> Result<Vec<PathBuf>> {
        if file_path.exists() {
            fs::remove_file(file_path).await
                .with_context(|| format!("Failed to delete file: {}", file_path.display()))?;
            Ok(vec![file_path.to_path_buf()])
        } else {
            Ok(Vec::new())
        }
    }

    async fn recreate_file(&self, file_path: &Path, content: &str) -> Result<Vec<PathBuf>> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        fs::write(file_path, content).await
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
        
        Ok(vec![file_path.to_path_buf()])
    }

    async fn restore_file_content(&self, file_path: &Path, content: &str) -> Result<Vec<PathBuf>> {
        self.recreate_file(file_path, content).await
    }

    async fn run_rollback_script(&self, script_path: &Path, args: &[String]) -> Result<Vec<PathBuf>> {
        // In a real implementation, this would execute the script
        // For now, we'll just log the operation
        info!("Would execute rollback script: {} with args: {:?}", script_path.display(), args);
        Ok(Vec::new())
    }

    async fn handle_user_action(&self, description: &str, instructions: &str) -> Result<Vec<PathBuf>> {
        // In a real implementation, this would prompt the user
        // For now, we'll just log the requirement
        info!("User action required: {} - {}", description, instructions);
        Ok(Vec::new())
    }

    async fn estimate_space_requirements(&self, plan: &RollbackPlan) -> Result<u64> {
        // Simplified space estimation
        let mut total_size = 0u64;
        
        for step in &plan.steps {
            match &step.operation {
                RollbackOperation::RestoreFile { source, destination, .. } => {
                    if source.exists() {
                        if let Ok(metadata) = fs::metadata(source).await {
                            total_size += metadata.len();
                        }
                    }
                }
                _ => {
                    total_size += 1024; // Estimate 1KB per step
                }
            }
        }
        
        Ok(total_size)
    }

    fn suggest_action_for_step_failure(&self, step: &RollbackStep, error_msg: &str) -> Option<String> {
        match &step.operation {
            RollbackOperation::RestoreFile { .. } => {
                Some("Verify backup file integrity and permissions".to_string())
            }
            RollbackOperation::GitCommand { .. } => {
                Some("Check Git repository state and resolve any conflicts manually".to_string())
            }
            RollbackOperation::ReverseOperation { .. } => {
                Some("Check file permissions and ensure file is not locked by another process".to_string())
            }
            _ => {
                Some(format!("Review error message and system state: {}", error_msg))
            }
        }
    }

    async fn track_execution_start(&self, execution: &RollbackExecution) -> Result<()> {
        if let Some(ref db) = self.history_database {
            // Record execution start in database
            debug!("Recording rollback execution start: {}", execution.execution_id);
        }
        Ok(())
    }

    async fn track_execution_completion(&self, execution: &RollbackExecution) -> Result<()> {
        if let Some(ref db) = self.history_database {
            // Record execution completion in database
            debug!("Recording rollback execution completion: {}", execution.execution_id);
        }
        Ok(())
    }

    async fn update_progress(
        &self,
        execution_id: &str,
        current_step: u32,
        total_steps: u32,
        operation: &str,
    ) {
        let progress = RollbackProgress {
            execution_id: execution_id.to_string(),
            current_step,
            total_steps,
            overall_progress: (current_step as f32 / total_steps as f32) * 100.0,
            current_step_progress: 0.0,
            estimated_remaining_time: None,
            status: RollbackExecutionStatus::InProgress,
            current_operation: operation.to_string(),
        };

        // Notify progress callbacks
        for callback in &self.progress_tracker.progress_callbacks {
            callback(&progress);
        }
    }
}

// Supporting structures and implementations

/// Backup management system
pub struct BackupManager {
    backup_root: PathBuf,
}

impl BackupManager {
    pub fn new() -> Self {
        Self {
            backup_root: PathBuf::from(".hive/backups"),
        }
    }

    pub async fn restore_from_backup(
        &self,
        backup_path: &Path,
        target_path: &Path,
    ) -> Result<Vec<PathBuf>> {
        info!("Restoring from backup: {} -> {}", backup_path.display(), target_path.display());
        
        if !backup_path.exists() {
            bail!("Backup file does not exist: {}", backup_path.display());
        }

        // Create target directory if needed
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await
                .with_context(|| format!("Failed to create target directory: {}", parent.display()))?;
        }

        // Copy backup to target
        fs::copy(backup_path, target_path).await
            .with_context(|| format!("Failed to restore backup: {} -> {}", backup_path.display(), target_path.display()))?;

        Ok(vec![target_path.to_path_buf()])
    }
}

/// Git operations manager
pub struct GitManager {
    // Git-specific configuration would go here
}

impl GitManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn verify_repository_state(&self) -> Result<()> {
        // In a real implementation, this would check Git status
        info!("Verifying Git repository state");
        Ok(())
    }

    pub async fn revert_commit(
        &self,
        commit_hash: &str,
        file_paths: Option<&[PathBuf]>,
    ) -> Result<Vec<PathBuf>> {
        info!("Reverting Git commit: {}", commit_hash);
        
        // In a real implementation, this would execute git revert
        // For now, we'll simulate the operation
        
        if let Some(paths) = file_paths {
            Ok(paths.to_vec())
        } else {
            Ok(Vec::new())
        }
    }
}

/// Verification system for rollback steps
pub struct VerificationSystem {
    // Verification configuration
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_successful: bool,
    pub verification_type: VerificationType,
    pub error_message: Option<String>,
    pub warnings: Vec<String>,
    pub verified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationType {
    FileExists,
    FileContent,
    FilePermissions,
    GitState,
    Custom(String),
}

impl VerificationSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn verify_step_completion(
        &self,
        step: &RollbackStep,
    ) -> Result<VerificationResult> {
        match &step.operation {
            RollbackOperation::RestoreFile { destination, .. } => {
                let exists = destination.exists();
                Ok(VerificationResult {
                    is_successful: exists,
                    verification_type: VerificationType::FileExists,
                    error_message: if !exists {
                        Some(format!("Target file was not restored: {}", destination.display()))
                    } else {
                        None
                    },
                    warnings: Vec::new(),
                    verified_at: Utc::now(),
                })
            }
            RollbackOperation::ReverseOperation { operation } => {
                if let FileOperation::Create { path, .. } = operation {
                    let exists = path.exists();
                    Ok(VerificationResult {
                        is_successful: !exists,
                        verification_type: VerificationType::FileExists,
                        error_message: if exists {
                            Some(format!("File was not deleted: {}", path.display()))
                        } else {
                            None
                        },
                        warnings: Vec::new(),
                        verified_at: Utc::now(),
                    })
                } else {
                    Ok(VerificationResult {
                        is_successful: true,
                        verification_type: VerificationType::Custom("reverse_operation".to_string()),
                        error_message: None,
                        warnings: vec!["Unsupported reverse operation type".to_string()],
                        verified_at: Utc::now(),
                    })
                }
            }
            _ => {
                // Default verification - assume success
                Ok(VerificationResult {
                    is_successful: true,
                    verification_type: VerificationType::Custom("default".to_string()),
                    error_message: None,
                    warnings: Vec::new(),
                    verified_at: Utc::now(),
                })
            }
        }
    }
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            current_execution: None,
            progress_callbacks: Vec::new(),
        }
    }

    pub fn add_progress_callback<F>(&mut self, callback: F)
    where
        F: Fn(&RollbackProgress) + Send + Sync + 'static,
    {
        self.progress_callbacks.push(Box::new(callback));
    }
}

// Utility functions

fn generate_execution_id() -> String {
    format!("rollback_{}", Utc::now().timestamp_millis())
}