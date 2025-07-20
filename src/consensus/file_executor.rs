// File Operation Execution Engine
// This module provides Claude Code-like abilities to create, modify, and manage files
// based on consensus decisions and AI-enhanced auto-accept logic

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::operation_analysis::{
    OperationContext as ConsensusOperationContext, OperationAnalysis as ConsensusOperationAnalysis,
    UnifiedScore, OperationGroups, ComponentScores, ScoringFactors
};
use crate::consensus::smart_decision_engine::{SmartDecisionEngine, ExecutionDecision};
use crate::consensus::operation_intelligence::{
    OperationIntelligenceCoordinator, OperationAnalysis as IntelligenceOperationAnalysis,
    OperationContext as IntelligenceOperationContext
};
use crate::consensus::ai_operation_parser::{AIOperationParser, ParsedOperations};
use crate::consensus::operation_preview_generator::{
    OperationPreviewGenerator, PreviewConfig, OperationPreviewSet
};
use std::collections::HashMap;
use crate::core::error::HiveError;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, Duration};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Represents the outcome of a file operation execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub operation_id: Uuid,
    pub operation: FileOperation,
    pub success: bool,
    pub execution_time: Duration,
    pub message: String,
    pub error_message: Option<String>,
    pub backup_created: Option<PathBuf>,
    pub rollback_required: bool,
    pub files_affected: Vec<PathBuf>,
}

/// Backup information for rollback capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub backup_id: Uuid,
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub created_at: SystemTime,
    pub operation_id: Uuid,
    pub file_hash: String,
}

/// Summary of executing operations from curator response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    /// Total number of operations found
    pub total_operations: usize,
    
    /// Number of successfully executed operations
    pub successful_operations: usize,
    
    /// Number of failed operations
    pub failed_operations: usize,
    
    /// Detailed results for each operation
    pub results: Vec<ExecutionResult>,
    
    /// Parser confidence score
    pub parser_confidence: f32,
    
    /// Warnings from parser
    pub parser_warnings: Vec<String>,
    
    /// Code blocks that couldn't be parsed
    pub unparsed_blocks: Vec<String>,
    
    /// Total execution time
    pub total_execution_time: Duration,
}

/// Configuration for file execution behavior
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub create_backups: bool,
    pub validate_syntax: bool,
    pub dry_run_mode: bool,
    pub max_file_size: u64, // in bytes
    pub allowed_extensions: Vec<String>,
    pub forbidden_paths: Vec<PathBuf>,
    pub stop_on_error: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            create_backups: true,
            validate_syntax: true,
            dry_run_mode: false,
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_extensions: vec![
                "rs".to_string(), "js".to_string(), "ts".to_string(),
                "py".to_string(), "java".to_string(), "cpp".to_string(),
                "c".to_string(), "h".to_string(), "hpp".to_string(),
                "go".to_string(), "rb".to_string(), "php".to_string(),
                "md".to_string(), "txt".to_string(), "json".to_string(),
                "yaml".to_string(), "yml".to_string(), "toml".to_string(),
                "html".to_string(), "css".to_string(), "scss".to_string(),
            ],
            forbidden_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/sys"),
                PathBuf::from("/proc"),
                PathBuf::from("/dev"),
                PathBuf::from("/boot"),
            ],
            stop_on_error: true,
        }
    }
}

/// Main file operation execution engine
pub struct FileOperationExecutor {
    config: ExecutorConfig,
    backup_manager: BackupManager,
    syntax_validator: SyntaxValidator,
    decision_engine: SmartDecisionEngine,
    intelligence_coordinator: OperationIntelligenceCoordinator,
    ai_parser: AIOperationParser,
    preview_generator: OperationPreviewGenerator,
}

/// Convert intelligence OperationContext to consensus OperationContext
fn convert_operation_context(
    intelligence_context: &IntelligenceOperationContext,
    user_question: String,
    consensus_response: String,
    session_id: String,
) -> ConsensusOperationContext {
    ConsensusOperationContext {
        repository_path: intelligence_context.repository_path.clone(),
        user_question,
        consensus_response,
        timestamp: SystemTime::now(),
        session_id,
        git_commit: intelligence_context.git_commit.clone(),
    }
}

/// Convert from intelligence analysis to consensus analysis format
fn convert_analysis(
    intelligence_analysis: IntelligenceOperationAnalysis,
    operations: Vec<FileOperation>,
    context: ConsensusOperationContext,
) -> ConsensusOperationAnalysis {
    // Group operations by type
    let mut create_operations = Vec::new();
    let mut update_operations = Vec::new();
    let mut delete_operations = Vec::new();
    let mut move_operations = Vec::new();

    for op in &operations {
        match op {
            FileOperation::Create { .. } => create_operations.push(op.clone()),
            FileOperation::Update { .. } => update_operations.push(op.clone()),
            FileOperation::Delete { .. } => delete_operations.push(op.clone()),
            FileOperation::Rename { .. } => move_operations.push(op.clone()),
            FileOperation::Append { .. } => update_operations.push(op.clone()),
        }
    }

    ConsensusOperationAnalysis {
        operations,
        context,
        unified_score: UnifiedScore {
            confidence: intelligence_analysis.confidence,
            risk: intelligence_analysis.risk,
        },
        recommendations: vec![], // Convert from intelligence_analysis.recommendation if needed
        groups: OperationGroups {
            create_operations,
            update_operations,
            delete_operations,
            move_operations,
        },
        component_scores: ComponentScores {
            knowledge_indexer: None,
            context_retriever: None,
            pattern_recognizer: None,
            quality_analyzer: None,
            knowledge_synthesizer: None,
        },
        scoring_factors: ScoringFactors {
            historical_success: None,
            pattern_safety: None,
            conflict_probability: None,
            rollback_complexity: None,
            user_trust: 0.8,
            similar_operations_count: None,
            dangerous_pattern_count: None,
            anti_pattern_count: None,
            rollback_possible: None,
        },
        statistics: None,
    }
}

impl FileOperationExecutor {
    pub fn new(
        config: ExecutorConfig,
        decision_engine: SmartDecisionEngine,
        intelligence_coordinator: OperationIntelligenceCoordinator,
    ) -> Self {
        let preview_config = PreviewConfig {
            max_diff_lines: 100,
            context_lines: 3,
            enable_syntax_highlighting: true,
            enable_impact_analysis: true,
        };
        
        Self {
            backup_manager: BackupManager::new(config.create_backups),
            syntax_validator: SyntaxValidator::new(config.validate_syntax),
            config,
            decision_engine,
            intelligence_coordinator,
            ai_parser: AIOperationParser::new(),
            preview_generator: OperationPreviewGenerator::new(preview_config),
        }
    }

    /// Execute a single file operation with full AI-enhanced analysis
    pub async fn execute_operation(
        &self,
        operation: FileOperation,
        context: &ConsensusOperationContext,
    ) -> Result<ExecutionResult, HiveError> {
        let operation_id = Uuid::new_v4();
        let start_time = SystemTime::now();

        // Step 1: Convert context for intelligence coordinator
        let intelligence_context = IntelligenceOperationContext {
            repository_path: context.repository_path.clone(),
            git_commit: context.git_commit.clone(),
            source_question: context.user_question.clone(),
            related_files: Vec::new(), // Could be populated from repository analysis
            project_metadata: HashMap::new(), // Could be populated from project files
        };

        // Step 2: AI-enhanced analysis  
        let intelligence_analysis = self.intelligence_coordinator
            .analyze_operation(&operation, &intelligence_context)
            .await?;

        // Step 3: Convert to consensus format
        let consensus_analysis = convert_analysis(
            intelligence_analysis,
            vec![operation.clone()],
            context.clone(),
        );

        // Step 4: Smart decision making
        let decision = self.decision_engine
            .make_decision(&consensus_analysis)
            .await?;

        // Step 5: Execute based on decision
        match decision {
            ExecutionDecision::AutoExecute { reason, confidence, risk_level } => {
                log::info!("Auto-executing operation with {}% confidence, {}% risk: {}", 
                          confidence, risk_level, reason);
                self.execute_with_safety_checks(operation, operation_id, start_time).await
            }
            ExecutionDecision::RequireConfirmation { warnings, suggestions, .. } => {
                log::warn!("Operation requires user confirmation: {:?}", warnings);
                Err(HiveError::OperationRequiresConfirmation { 
                    operation_id,
                    warnings,
                    suggestions,
                })
            }
            ExecutionDecision::Block { critical_issues, .. } => {
                log::error!("Operation blocked for safety: {:?}", critical_issues);
                Err(HiveError::OperationBlocked { 
                    operation_id,
                    reasons: critical_issues,
                })
            }
        }
    }

    /// Execute multiple operations as a batch with dependency analysis
    pub async fn execute_operations_batch(
        &self,
        operations: Vec<FileOperation>,
        context: &ConsensusOperationContext,
    ) -> Result<Vec<ExecutionResult>, HiveError> {
        let mut results = Vec::new();
        let mut successful_operations = Vec::new();

        // Convert context for intelligence coordinator
        let intelligence_context = IntelligenceOperationContext {
            repository_path: context.repository_path.clone(),
            git_commit: context.git_commit.clone(),
            source_question: context.user_question.clone(),
            related_files: Vec::new(), // Could be populated from repository analysis
            project_metadata: HashMap::new(), // Could be populated from project files
        };

        // Analyze all operations first
        let intelligence_analyses = self.intelligence_coordinator
            .analyze_operations_batch(&operations, &intelligence_context)
            .await?;

        // Convert to consensus format
        let consensus_analyses: Vec<_> = operations.iter()
            .zip(intelligence_analyses.into_iter())
            .map(|(op, intel_analysis)| {
                convert_analysis(intel_analysis, vec![op.clone()], context.clone())
            })
            .collect();

        // Execute operations in dependency order
        for (operation, analysis) in operations.into_iter().zip(consensus_analyses.into_iter()) {
            match self.execute_operation_with_analysis(operation.clone(), analysis, context).await {
                Ok(result) => {
                    if result.success {
                        successful_operations.push(operation);
                    }
                    results.push(result);
                }
                Err(e) => {
                    log::error!("Operation failed, rolling back previous operations: {:?}", e);
                    
                    // Rollback all successful operations
                    for prev_op in successful_operations.iter().rev() {
                        if let Err(rollback_error) = self.rollback_operation(prev_op).await {
                            log::error!("Rollback failed: {:?}", rollback_error);
                        }
                    }
                    
                    return Err(e);
                }
            }
        }

        Ok(results)
    }

    /// Execute operation with pre-computed analysis
    async fn execute_operation_with_analysis(
        &self,
        operation: FileOperation,
        analysis: ConsensusOperationAnalysis,
        context: &ConsensusOperationContext,
    ) -> Result<ExecutionResult, HiveError> {
        let operation_id = Uuid::new_v4();
        let start_time = SystemTime::now();

        let decision = self.decision_engine.make_decision(&analysis).await?;

        match decision {
            ExecutionDecision::AutoExecute { .. } => {
                self.execute_with_safety_checks(operation, operation_id, start_time).await
            }
            _ => {
                Err(HiveError::OperationRequiresConfirmation { 
                    operation_id,
                    warnings: vec!["Batch operation requires individual confirmation".to_string()],
                    suggestions: vec!["Execute operations individually".to_string()],
                })
            }
        }
    }

    /// Core execution logic with safety checks and backup creation
    async fn execute_with_safety_checks(
        &self,
        operation: FileOperation,
        operation_id: Uuid,
        start_time: SystemTime,
    ) -> Result<ExecutionResult, HiveError> {
        // Safety check: validate path
        self.validate_operation_safety(&operation)?;

        // Create backup if needed
        let backup_info = if self.config.create_backups {
            self.backup_manager.create_backup(&operation).await.ok()
        } else {
            None
        };

        // Execute the actual file operation
        let execution_result = match &operation {
            FileOperation::Create { path, content } => {
                self.execute_create_file(path, content).await
            }
            FileOperation::Update { path, content } => {
                self.execute_update_file(path, content).await
            }
            FileOperation::Delete { path } => {
                self.execute_delete_file(path).await
            }
            FileOperation::Rename { from, to } => {
                self.execute_move_file(from, to).await
            }
            FileOperation::Append { path, content } => {
                self.execute_append_file(path, content).await
            }
        };

        let execution_time = start_time.elapsed().unwrap_or(Duration::from_millis(0));

        match execution_result {
            Ok(files_affected) => {
                // Validate syntax if enabled
                if self.config.validate_syntax {
                    for file_path in &files_affected {
                        if let Err(syntax_error) = self.syntax_validator.validate_file(file_path).await {
                            log::warn!("Syntax validation failed for {}: {:?}", 
                                     file_path.display(), syntax_error);
                            
                            // Rollback if syntax is invalid
                            if let Some(backup) = &backup_info {
                                let _ = self.backup_manager.restore_backup(&backup).await;
                            }
                            
                            return Ok(ExecutionResult {
                                operation_id,
                                operation,
                                success: false,
                                execution_time,
                                message: format!("Syntax validation failed: {:?}", syntax_error),
                                error_message: Some(format!("Syntax validation failed: {:?}", syntax_error)),
                                backup_created: backup_info.as_ref().map(|b| b.backup_path.clone()),
                                rollback_required: true,
                                files_affected,
                            });
                        }
                    }
                }

                Ok(ExecutionResult {
                    operation_id,
                    operation,
                    success: true,
                    execution_time,
                    message: "Operation completed successfully".to_string(),
                    error_message: None,
                    backup_created: backup_info.as_ref().map(|b| b.backup_path.clone()),
                    rollback_required: false,
                    files_affected,
                })
            }
            Err(error) => {
                Ok(ExecutionResult {
                    operation_id,
                    operation,
                    success: false,
                    execution_time,
                    message: format!("Operation failed: {}", error),
                    error_message: Some(error.to_string()),
                    backup_created: backup_info.as_ref().map(|b| b.backup_path.clone()),
                    rollback_required: backup_info.is_some(),
                    files_affected: vec![],
                })
            }
        }
    }

    /// Validate operation safety before execution
    fn validate_operation_safety(&self, operation: &FileOperation) -> Result<(), HiveError> {
        let path = match operation {
            FileOperation::Create { path, .. } |
            FileOperation::Update { path, .. } |
            FileOperation::Delete { path } |
            FileOperation::Append { path, .. } => path,
            FileOperation::Rename { from, .. } => from,
        };

        // Check forbidden paths
        for forbidden in &self.config.forbidden_paths {
            if path.starts_with(forbidden) {
                return Err(HiveError::OperationBlocked {
                    operation_id: Uuid::new_v4(),
                    reasons: vec![format!("Path {} is in forbidden directory {}", 
                                        path.display(), forbidden.display())],
                });
            }
        }

        // Check file extension
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            if !self.config.allowed_extensions.contains(&extension.to_string()) {
                return Err(HiveError::OperationBlocked {
                    operation_id: Uuid::new_v4(),
                    reasons: vec![format!("File extension '{}' is not allowed", extension)],
                });
            }
        }

        Ok(())
    }

    /// Execute file creation
    async fn execute_create_file(&self, path: &Path, content: &str) -> Result<Vec<PathBuf>, HiveError> {
        if self.config.dry_run_mode {
            log::info!("DRY RUN: Would create file {}", path.display());
            return Ok(vec![path.to_path_buf()]);
        }

        // Check file size limit
        if content.len() as u64 > self.config.max_file_size {
            return Err(HiveError::OperationBlocked {
                operation_id: Uuid::new_v4(),
                reasons: vec![format!("File size {} exceeds limit {}", 
                                    content.len(), self.config.max_file_size)],
            });
        }

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| HiveError::FileOperationFailed {
                    operation: "create_parent_dirs".to_string(),
                    path: parent.to_path_buf(),
                    reason: e.to_string(),
                })?;
        }

        // Write the file
        fs::write(path, content)
            .map_err(|e| HiveError::FileOperationFailed {
                operation: "create_file".to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        log::info!("Successfully created file: {}", path.display());
        Ok(vec![path.to_path_buf()])
    }

    /// Execute file update
    async fn execute_update_file(&self, path: &Path, content: &str) -> Result<Vec<PathBuf>, HiveError> {
        if self.config.dry_run_mode {
            log::info!("DRY RUN: Would update file {}", path.display());
            return Ok(vec![path.to_path_buf()]);
        }

        // Check if file exists
        if !path.exists() {
            return Err(HiveError::FileOperationFailed {
                operation: "update_file".to_string(),
                path: path.to_path_buf(),
                reason: "File does not exist".to_string(),
            });
        }

        // Check file size limit
        if content.len() as u64 > self.config.max_file_size {
            return Err(HiveError::OperationBlocked {
                operation_id: Uuid::new_v4(),
                reasons: vec![format!("File size {} exceeds limit {}", 
                                    content.len(), self.config.max_file_size)],
            });
        }

        // Write the updated content
        fs::write(path, content)
            .map_err(|e| HiveError::FileOperationFailed {
                operation: "update_file".to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        log::info!("Successfully updated file: {}", path.display());
        Ok(vec![path.to_path_buf()])
    }

    /// Execute file deletion
    async fn execute_delete_file(&self, path: &Path) -> Result<Vec<PathBuf>, HiveError> {
        if self.config.dry_run_mode {
            log::info!("DRY RUN: Would delete file {}", path.display());
            return Ok(vec![path.to_path_buf()]);
        }

        // Check if file exists
        if !path.exists() {
            return Err(HiveError::FileOperationFailed {
                operation: "delete_file".to_string(),
                path: path.to_path_buf(),
                reason: "File does not exist".to_string(),
            });
        }

        // Delete the file
        fs::remove_file(path)
            .map_err(|e| HiveError::FileOperationFailed {
                operation: "delete_file".to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        log::info!("Successfully deleted file: {}", path.display());
        Ok(vec![path.to_path_buf()])
    }

    /// Execute file move
    async fn execute_move_file(&self, from: &Path, to: &Path) -> Result<Vec<PathBuf>, HiveError> {
        if self.config.dry_run_mode {
            log::info!("DRY RUN: Would move file {} to {}", from.display(), to.display());
            return Ok(vec![from.to_path_buf(), to.to_path_buf()]);
        }

        // Check if source file exists
        if !from.exists() {
            return Err(HiveError::FileOperationFailed {
                operation: "move_file".to_string(),
                path: from.to_path_buf(),
                reason: "Source file does not exist".to_string(),
            });
        }

        // Create parent directories for destination
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| HiveError::FileOperationFailed {
                    operation: "create_parent_dirs".to_string(),
                    path: parent.to_path_buf(),
                    reason: e.to_string(),
                })?;
        }

        // Move the file
        fs::rename(from, to)
            .map_err(|e| HiveError::FileOperationFailed {
                operation: "move_file".to_string(),
                path: from.to_path_buf(),
                reason: e.to_string(),
            })?;

        log::info!("Successfully moved file from {} to {}", from.display(), to.display());
        Ok(vec![from.to_path_buf(), to.to_path_buf()])
    }

    /// Execute file append
    async fn execute_append_file(&self, path: &Path, content: &str) -> Result<Vec<PathBuf>, HiveError> {
        if self.config.dry_run_mode {
            log::info!("DRY RUN: Would append to file {}", path.display());
            return Ok(vec![path.to_path_buf()]);
        }

        // Check if file exists
        if !path.exists() {
            return Err(HiveError::FileOperationFailed {
                operation: "append_file".to_string(),
                path: path.to_path_buf(),
                reason: "File does not exist".to_string(),
            });
        }

        // Check total file size after append
        let current_size = fs::metadata(path)
            .map_err(|e| HiveError::FileOperationFailed {
                operation: "check_file_size".to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?
            .len();

        if current_size + content.len() as u64 > self.config.max_file_size {
            return Err(HiveError::OperationBlocked {
                operation_id: Uuid::new_v4(),
                reasons: vec![format!("File size after append {} would exceed limit {}", 
                                    current_size + content.len() as u64, self.config.max_file_size)],
            });
        }

        // Append to the file
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| HiveError::FileOperationFailed {
                operation: "open_append".to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        file.write_all(content.as_bytes())
            .map_err(|e| HiveError::FileOperationFailed {
                operation: "append_content".to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        log::info!("Successfully appended to file: {}", path.display());
        Ok(vec![path.to_path_buf()])
    }

    /// Rollback an operation using backup
    async fn rollback_operation(&self, operation: &FileOperation) -> Result<(), HiveError> {
        // Implementation for rollback logic
        // This would use the backup manager to restore previous state
        log::info!("Rolling back operation: {:?}", operation);
        // TODO: Implement actual rollback logic
        Ok(())
    }

    /// Parse and execute operations from curator response with AI-enhanced parsing
    pub async fn parse_and_execute_curator_response(
        &self,
        curator_response: &str,
        context: &ConsensusOperationContext,
    ) -> Result<ExecutionSummary, HiveError> {
        let start_time = SystemTime::now();
        
        // Step 1: Convert context for AI parser
        let intelligence_context = IntelligenceOperationContext {
            repository_path: context.repository_path.clone(),
            git_commit: context.git_commit.clone(),
            source_question: context.user_question.clone(),
            related_files: Vec::new(),
            project_metadata: HashMap::new(),
        };

        // Step 2: Parse curator response using AI-enhanced parser
        let parsed = self.ai_parser
            .parse_response(curator_response, &intelligence_context)
            .await
            .map_err(|e| HiveError::Other(format!("Failed to parse curator response: {}", e)))?;

        log::info!("🤖 Parsed {} operations with {}% confidence", 
                  parsed.operations.len(), parsed.confidence);

        // Log warnings if any
        for warning in &parsed.warnings {
            log::warn!("Parser warning: {}", warning);
        }

        // Check if parsing confidence is too low
        if parsed.confidence < 50.0 {
            return Err(HiveError::Other(format!(
                "Parsing confidence too low: {:.0}%. Clarifications needed: {:?}",
                parsed.confidence, parsed.clarifications
            )));
        }

        // Step 3: Extract operations in dependency order
        let mut operations_in_order = Vec::new();
        let mut processed = vec![false; parsed.operations.len()];
        
        // First, add operations without dependencies
        for (idx, op_meta) in parsed.operations.iter().enumerate() {
            if op_meta.dependencies.is_empty() {
                operations_in_order.push((idx, &op_meta.operation));
                processed[idx] = true;
            }
        }
        
        // Then, add operations with satisfied dependencies
        while operations_in_order.len() < parsed.operations.len() {
            let mut added_any = false;
            
            for (idx, op_meta) in parsed.operations.iter().enumerate() {
                if !processed[idx] {
                    let deps_satisfied = op_meta.dependencies.iter()
                        .all(|&dep_idx| processed[dep_idx]);
                    
                    if deps_satisfied {
                        operations_in_order.push((idx, &op_meta.operation));
                        processed[idx] = true;
                        added_any = true;
                    }
                }
            }
            
            if !added_any {
                return Err(HiveError::Other(
                    "Circular dependencies detected in operations".to_string()
                ));
            }
        }

        // Step 4: Execute operations in order
        let mut results = Vec::new();
        let mut successful_count = 0;
        let mut failed_count = 0;

        for (idx, operation) in operations_in_order {
            log::info!("Executing operation {} of {}: {:?}", 
                      idx + 1, parsed.operations.len(), operation);

            match self.execute_operation(operation.clone(), context).await {
                Ok(result) => {
                    if result.success {
                        successful_count += 1;
                        log::info!("✅ Operation succeeded: {}", result.message);
                    } else {
                        failed_count += 1;
                        log::error!("❌ Operation failed: {}", result.message);
                    }
                    results.push(result);
                }
                Err(e) => {
                    failed_count += 1;
                    log::error!("❌ Operation execution error: {}", e);
                    
                    // Create error result
                    results.push(ExecutionResult {
                        operation_id: Uuid::new_v4(),
                        operation: operation.clone(),
                        success: false,
                        execution_time: Duration::from_millis(0),
                        message: format!("Operation execution error: {}", e),
                        error_message: Some(e.to_string()),
                        backup_created: None,
                        rollback_required: false,
                        files_affected: vec![],
                    });

                    // Optionally stop on first error
                    if self.config.stop_on_error {
                        log::error!("Stopping execution due to error (stop_on_error = true)");
                        break;
                    }
                }
            }
        }

        let total_time = start_time.elapsed().unwrap_or(Duration::from_millis(0));

        Ok(ExecutionSummary {
            total_operations: parsed.operations.len(),
            successful_operations: successful_count,
            failed_operations: failed_count,
            results,
            parser_confidence: parsed.confidence,
            parser_warnings: parsed.warnings,
            unparsed_blocks: parsed.unparsed_blocks,
            total_execution_time: total_time,
        })
    }

    /// Parse operations from curator response without executing (dry run analysis)
    pub async fn parse_curator_response(
        &self,
        curator_response: &str,
        context: &ConsensusOperationContext,
    ) -> Result<ParsedOperations, HiveError> {
        // Convert context for AI parser
        let intelligence_context = IntelligenceOperationContext {
            repository_path: context.repository_path.clone(),
            git_commit: context.git_commit.clone(),
            source_question: context.user_question.clone(),
            related_files: Vec::new(),
            project_metadata: HashMap::new(),
        };

        self.ai_parser
            .parse_response(curator_response, &intelligence_context)
            .await
            .map_err(|e| HiveError::Other(format!("Failed to parse curator response: {}", e)))
    }

    /// Generate previews for parsed operations
    pub async fn generate_operation_previews(
        &self,
        parsed_operations: &ParsedOperations,
        context: &ConsensusOperationContext,
    ) -> Result<OperationPreviewSet, HiveError> {
        // Extract just the operations from the parsed operations
        let operations: Vec<_> = parsed_operations.operations
            .iter()
            .map(|op_meta| op_meta.operation.clone())
            .collect();

        // Convert context for preview generator
        let preview_context = crate::consensus::operation_preview::OperationContext {
            repository_path: context.repository_path.clone(),
            user_question: context.user_question.clone(),
            consensus_response: context.consensus_response.clone(),
            timestamp: context.timestamp,
            session_id: context.session_id.clone(),
            git_commit: context.git_commit.clone(),
        };

        self.preview_generator
            .generate_preview_set(&operations, &preview_context)
            .await
            .map_err(|e| HiveError::Other(format!("Failed to generate operation previews: {}", e)))
    }

    /// Parse and execute operations with preview generation
    pub async fn parse_execute_with_preview(
        &self,
        curator_response: &str,
        context: &ConsensusOperationContext,
        generate_preview: bool,
    ) -> Result<(ExecutionSummary, Option<OperationPreviewSet>), HiveError> {
        // First parse the operations
        let parsed = self.parse_curator_response(curator_response, context).await?;
        
        // Generate preview if requested
        let preview = if generate_preview {
            Some(self.generate_operation_previews(&parsed, context).await?)
        } else {
            None
        };

        // Execute the operations
        let summary = self.parse_and_execute_curator_response(curator_response, context).await?;

        Ok((summary, preview))
    }
}

/// Backup manager for creating and restoring file backups
pub struct BackupManager {
    enabled: bool,
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            backup_dir: PathBuf::from(".hive_backups"),
        }
    }

    pub async fn create_backup(&self, operation: &FileOperation) -> Result<BackupInfo, HiveError> {
        if !self.enabled {
            return Err(HiveError::BackupError("Backup disabled".to_string()));
        }

        let original_path = match operation {
            FileOperation::Update { path, .. } |
            FileOperation::Delete { path } |
            FileOperation::Append { path, .. } => path.clone(),
            FileOperation::Rename { from, .. } => from.clone(),
            FileOperation::Create { .. } => {
                // No backup needed for create operations
                return Err(HiveError::BackupError("No backup needed for create operation".to_string()));
            }
        };

        if !original_path.exists() {
            return Err(HiveError::BackupError("Original file does not exist".to_string()));
        }

        // Create backup directory
        fs::create_dir_all(&self.backup_dir)
            .map_err(|e| HiveError::BackupError(e.to_string()))?;

        // Generate backup path
        let backup_id = Uuid::new_v4();
        let backup_filename = format!("{}_{}", 
                                    backup_id.to_string(), 
                                    original_path.file_name().unwrap().to_string_lossy());
        let backup_path = self.backup_dir.join(backup_filename);

        // Copy file to backup location
        fs::copy(&original_path, &backup_path)
            .map_err(|e| HiveError::BackupError(e.to_string()))?;

        // Calculate file hash for verification
        let file_content = fs::read(&original_path)
            .map_err(|e| HiveError::BackupError(e.to_string()))?;
        let file_hash = format!("{:x}", md5::compute(file_content));

        Ok(BackupInfo {
            backup_id,
            original_path,
            backup_path,
            created_at: SystemTime::now(),
            operation_id: Uuid::new_v4(),
            file_hash,
        })
    }

    pub async fn restore_backup(&self, backup_info: &BackupInfo) -> Result<(), HiveError> {
        if !backup_info.backup_path.exists() {
            return Err(HiveError::BackupError("Backup file does not exist".to_string()));
        }

        // Restore the file
        fs::copy(&backup_info.backup_path, &backup_info.original_path)
            .map_err(|e| HiveError::BackupError(e.to_string()))?;

        log::info!("Restored file from backup: {} -> {}", 
                  backup_info.backup_path.display(), 
                  backup_info.original_path.display());

        Ok(())
    }
}

/// Syntax validator for ensuring code quality
pub struct SyntaxValidator {
    enabled: bool,
}

impl SyntaxValidator {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub async fn validate_file(&self, path: &Path) -> Result<(), HiveError> {
        if !self.enabled {
            return Ok(());
        }

        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "rs" => self.validate_rust_syntax(path).await,
            "js" | "ts" => self.validate_javascript_syntax(path).await,
            "py" => self.validate_python_syntax(path).await,
            _ => Ok(()), // No validation for other file types
        }
    }

    async fn validate_rust_syntax(&self, path: &Path) -> Result<(), HiveError> {
        // Basic Rust syntax validation
        let content = fs::read_to_string(path)
            .map_err(|e| HiveError::SyntaxValidationError {
                file: path.to_path_buf(),
                message: e.to_string(),
            })?;

        // Check for basic syntax issues
        if content.contains("fn main(") && !content.contains("}") {
            return Err(HiveError::SyntaxValidationError {
                file: path.to_path_buf(),
                message: "Unmatched braces detected".to_string(),
            });
        }

        // Additional Rust-specific checks can be added here
        Ok(())
    }

    async fn validate_javascript_syntax(&self, _path: &Path) -> Result<(), HiveError> {
        // JavaScript/TypeScript syntax validation would go here
        Ok(())
    }

    async fn validate_python_syntax(&self, _path: &Path) -> Result<(), HiveError> {
        // Python syntax validation would go here
        Ok(())
    }
}