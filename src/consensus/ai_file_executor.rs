//! AI-powered File Executor for Consensus
//!
//! This module bridges the consensus engine with AI Helper file operations,
//! enabling the separation of thinking (consensus) from doing (file ops).

use anyhow::Result;
use std::sync::Arc;
use tracing::{info, debug, warn};

use crate::ai_helpers::file_executor::{
    AIHelperFileExecutor, ExecutionPlan, FileOperation, OperationType, 
    SafetyLevel, ExecutionReport
};
use crate::ai_helpers::AIHelperEcosystem;
use crate::consensus::stages::file_aware_curator::FileOperation as CuratorFileOp;

/// AI-powered consensus file executor
pub struct AIConsensusFileExecutor {
    executor: AIHelperFileExecutor,
}

impl AIConsensusFileExecutor {
    pub fn new(ai_helpers: AIHelperEcosystem) -> Self {
        Self {
            executor: AIHelperFileExecutor::new(ai_helpers),
        }
    }

    /// Execute operations from Curator output
    pub async fn execute_from_curator(&self, curator_output: &str) -> Result<ExecutionReport> {
        info!("🤖 AI Helper analyzing Curator output for file operations");
        
        // Use AI Helper's intelligence to understand the Curator output
        // The AI Helper can intelligently parse any format, not just specific patterns
        let plan = self.executor.parse_request(curator_output).await?;
        
        if plan.operations.is_empty() {
            debug!("AI Helper found no file operations in Curator output");
            return Ok(ExecutionReport {
                success: true,
                operations_completed: 0,
                operations_total: 0,
                errors: vec![],
                files_created: vec![],
                files_modified: vec![],
                files_deleted: vec![],
            });
        }

        info!("AI Helper identified {} operations to execute", plan.operations.len());
        self.executor.execute_plan(plan).await
    }

    /// Execute operations from consensus file operations
    pub async fn execute_curator_operations(&self, operations: Vec<CuratorFileOp>) -> Result<ExecutionReport> {
        info!("Converting {} curator operations for execution", operations.len());
        
        let mut file_operations = Vec::new();
        
        for (idx, op) in operations.iter().enumerate() {
            let file_op = match op {
                CuratorFileOp::Create { path, content } => {
                    FileOperation {
                        step: idx + 1,
                        action: OperationType::CreateFile {
                            path: path.clone(),
                            content: content.clone(),
                        },
                        description: format!("Create file: {:?}", path),
                    }
                }
                CuratorFileOp::Update { path, content } => {
                    FileOperation {
                        step: idx + 1,
                        action: OperationType::UpdateFile {
                            path: path.clone(),
                            changes: vec![crate::ai_helpers::file_executor::FileChange {
                                find: String::new(), // Would need actual diff
                                replace: content.clone(),
                                all_occurrences: true,
                            }],
                        },
                        description: format!("Update file: {:?}", path),
                    }
                }
                CuratorFileOp::Delete { path } => {
                    FileOperation {
                        step: idx + 1,
                        action: OperationType::DeleteFile {
                            path: path.clone(),
                        },
                        description: format!("Delete file: {:?}", path),
                    }
                }
                CuratorFileOp::Rename { from, to } => {
                    FileOperation {
                        step: idx + 1,
                        action: OperationType::MoveFile {
                            from: from.clone(),
                            to: to.clone(),
                        },
                        description: format!("Move file from {:?} to {:?}", from, to),
                    }
                }
                CuratorFileOp::Append { path, content } => {
                    // For append, we treat it as an update that preserves existing content
                    FileOperation {
                        step: idx + 1,
                        action: OperationType::UpdateFile {
                            path: path.clone(),
                            changes: vec![crate::ai_helpers::file_executor::FileChange {
                                find: String::new(), // Empty find means append at end
                                replace: format!("\n{}", content), // Append with newline
                                all_occurrences: false,
                            }],
                        },
                        description: format!("Append to file: {:?}", path),
                    }
                }
            };
            file_operations.push(file_op);
        }

        let plan = ExecutionPlan {
            overview: "Execute curator file operations".to_string(),
            safety_level: SafetyLevel::Medium,
            operations: file_operations,
        };

        self.executor.execute_plan(plan).await
    }

    /// Execute a simple file operation request
    pub async fn execute_simple_request(&self, request: &str) -> Result<ExecutionReport> {
        info!("Executing simple request: {}", request);
        
        let plan = self.executor.parse_request(request).await?;
        self.executor.execute_plan(plan).await
    }
}
