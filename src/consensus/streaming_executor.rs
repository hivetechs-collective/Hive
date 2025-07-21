//! Streaming Operation Executor
//!
//! Executes file operations during response streaming for a Claude Code-like experience.
//! Operations are executed inline as they appear in the response, with real-time status updates.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::ai_helpers::AIHelperEcosystem;
use crate::consensus::FileOperationExecutor;
use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::smart_decision_engine::UserPreferences;
use crate::consensus::operation_analysis::OperationContext as ConsensusOperationContext;
use crate::consensus::operation_intelligence::OperationContext;

/// Status of an operation during streaming execution
#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    /// Operation is waiting for confirmation
    WaitingConfirmation,
    /// Operation is currently executing
    Executing,
    /// Operation completed successfully
    Completed(String), // Success message
    /// Operation failed
    Failed(String), // Error message
    /// Operation was skipped by user
    Skipped,
}

/// Callback for status updates during execution
pub trait StatusCallback: Send + Sync {
    fn on_status(&self, status: ExecutionStatus);
}

/// Streaming operation executor for inline execution
pub struct StreamingOperationExecutor {
    file_executor: Arc<FileOperationExecutor>,
    ai_helpers: Arc<AIHelperEcosystem>,
    auto_accept_mode: Arc<AtomicBool>,
    status_sender: mpsc::UnboundedSender<ExecutionStatus>,
}

impl StreamingOperationExecutor {
    /// Create a new streaming operation executor
    pub fn new(
        file_executor: Arc<FileOperationExecutor>,
        ai_helpers: Arc<AIHelperEcosystem>,
        auto_accept_mode: Arc<AtomicBool>,
        status_sender: mpsc::UnboundedSender<ExecutionStatus>,
    ) -> Self {
        Self {
            file_executor,
            ai_helpers,
            auto_accept_mode,
            status_sender,
        }
    }

    /// Execute an operation inline with status updates
    pub async fn execute_inline(
        &self,
        operation: FileOperation,
        confirmation_receiver: Option<mpsc::Receiver<bool>>,
    ) -> Result<()> {
        // Check auto-accept mode
        if self.auto_accept_mode.load(Ordering::Relaxed) {
            // Execute immediately
            self.status_sender.send(ExecutionStatus::Executing)?;
            
            // Create a simple context for execution
            let context = ConsensusOperationContext {
                repository_path: std::env::current_dir().unwrap_or_default(),
                user_question: "Streaming operation execution".to_string(),
                consensus_response: "Executing inline operation".to_string(),
                timestamp: std::time::SystemTime::now(),
                session_id: "streaming-exec".to_string(),
                git_commit: None,
            };
            
            match self.file_executor.execute_operation(operation, &context).await {
                Ok(_) => {
                    self.status_sender.send(ExecutionStatus::Completed(
                        "Operation completed successfully".to_string()
                    ))?;
                }
                Err(e) => {
                    self.status_sender.send(ExecutionStatus::Failed(
                        format!("Operation failed: {}", e)
                    ))?;
                }
            }
        } else {
            // Wait for confirmation
            self.status_sender.send(ExecutionStatus::WaitingConfirmation)?;
            
            if let Some(mut receiver) = confirmation_receiver {
                match receiver.recv().await {
                    Some(true) => {
                        // User approved
                        self.status_sender.send(ExecutionStatus::Executing)?;
                        
                        // Create a simple context for execution
            let context = ConsensusOperationContext {
                repository_path: std::env::current_dir().unwrap_or_default(),
                user_question: "Streaming operation execution".to_string(),
                consensus_response: "Executing inline operation".to_string(),
                timestamp: std::time::SystemTime::now(),
                session_id: "streaming-exec".to_string(),
                git_commit: None,
            };
            
            match self.file_executor.execute_operation(operation, &context).await {
                            Ok(_) => {
                                self.status_sender.send(ExecutionStatus::Completed(
                                    "Operation completed successfully".to_string()
                                ))?;
                            }
                            Err(e) => {
                                self.status_sender.send(ExecutionStatus::Failed(
                                    format!("Operation failed: {}", e)
                                ))?;
                            }
                        }
                    }
                    Some(false) | None => {
                        // User rejected or channel closed
                        self.status_sender.send(ExecutionStatus::Skipped)?;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Execute multiple operations with proper sequencing
    pub async fn execute_batch(
        &self,
        operations: Vec<FileOperation>,
        confirmation_receiver: Option<mpsc::Receiver<Vec<bool>>>,
    ) -> Result<Vec<ExecutionStatus>> {
        let mut results = Vec::new();
        
        if self.auto_accept_mode.load(Ordering::Relaxed) {
            // Execute all operations in sequence
            for operation in operations {
                self.status_sender.send(ExecutionStatus::Executing)?;
                
                // Create a simple context for execution
            let context = ConsensusOperationContext {
                repository_path: std::env::current_dir().unwrap_or_default(),
                user_question: "Streaming operation execution".to_string(),
                consensus_response: "Executing inline operation".to_string(),
                timestamp: std::time::SystemTime::now(),
                session_id: "streaming-exec".to_string(),
                git_commit: None,
            };
            
            match self.file_executor.execute_operation(operation, &context).await {
                    Ok(_) => {
                        let status = ExecutionStatus::Completed(
                            "Operation completed successfully".to_string()
                        );
                        self.status_sender.send(status.clone())?;
                        results.push(status);
                    }
                    Err(e) => {
                        let status = ExecutionStatus::Failed(
                            format!("Operation failed: {}", e)
                        );
                        self.status_sender.send(status.clone())?;
                        results.push(status);
                        // Continue with remaining operations
                    }
                }
            }
        } else if let Some(mut receiver) = confirmation_receiver {
            // Wait for batch confirmation
            self.status_sender.send(ExecutionStatus::WaitingConfirmation)?;
            
            if let Some(confirmations) = receiver.recv().await {
                for (operation, approved) in operations.into_iter().zip(confirmations) {
                    if approved {
                        self.status_sender.send(ExecutionStatus::Executing)?;
                        
                        // Create a simple context for execution
            let context = ConsensusOperationContext {
                repository_path: std::env::current_dir().unwrap_or_default(),
                user_question: "Streaming operation execution".to_string(),
                consensus_response: "Executing inline operation".to_string(),
                timestamp: std::time::SystemTime::now(),
                session_id: "streaming-exec".to_string(),
                git_commit: None,
            };
            
            match self.file_executor.execute_operation(operation, &context).await {
                            Ok(_) => {
                                let status = ExecutionStatus::Completed(
                                    "Operation completed successfully".to_string()
                                );
                                self.status_sender.send(status.clone())?;
                                results.push(status);
                            }
                            Err(e) => {
                                let status = ExecutionStatus::Failed(
                                    format!("Operation failed: {}", e)
                                );
                                self.status_sender.send(status.clone())?;
                                results.push(status);
                            }
                        }
                    } else {
                        let status = ExecutionStatus::Skipped;
                        self.status_sender.send(status.clone())?;
                        results.push(status);
                    }
                }
            }
        }
        
        Ok(results)
    }

    /// Analyze operation risk using AI helpers
    pub async fn analyze_operation_risk(&self, operation: &FileOperation) -> Result<f32> {
        // Create operation context for AI analysis
        let context = OperationContext {
            repository_path: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            git_commit: None, // Could be enhanced to get actual commit
            source_question: "File operation".to_string(), // Could be enhanced with actual question
            related_files: vec![],
            project_metadata: HashMap::new(),
        };
        
        // Use AI helpers to assess risk
        let risk_assessment = self.ai_helpers.quality_analyzer.assess_operation_risk(operation, &context).await?;
        Ok(risk_assessment.risk_score)
    }

    /// Get operation confidence using AI helpers
    pub async fn get_operation_confidence(&self, operation: &FileOperation) -> Result<f32> {
        // Create operation context for AI analysis
        let context = OperationContext {
            repository_path: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            git_commit: None, // Could be enhanced to get actual commit
            source_question: "File operation".to_string(), // Could be enhanced with actual question
            related_files: vec![],
            project_metadata: HashMap::new(),
        };
        
        // Use AI helpers to get confidence
        let prediction = self.ai_helpers.knowledge_indexer.predict_operation_success(operation, &context).await?;
        Ok(prediction.success_probability)
    }
}

/// Builder for creating a streaming operation executor
pub struct StreamingExecutorBuilder {
    file_executor: Option<Arc<FileOperationExecutor>>,
    ai_helpers: Option<Arc<AIHelperEcosystem>>,
    auto_accept_mode: Option<Arc<AtomicBool>>,
    status_sender: Option<mpsc::UnboundedSender<ExecutionStatus>>,
}

impl StreamingExecutorBuilder {
    pub fn new() -> Self {
        Self {
            file_executor: None,
            ai_helpers: None,
            auto_accept_mode: None,
            status_sender: None,
        }
    }

    pub fn with_file_executor(mut self, executor: Arc<FileOperationExecutor>) -> Self {
        self.file_executor = Some(executor);
        self
    }

    pub fn with_ai_helpers(mut self, helpers: Arc<AIHelperEcosystem>) -> Self {
        self.ai_helpers = Some(helpers);
        self
    }

    pub fn with_auto_accept_mode(mut self, mode: Arc<AtomicBool>) -> Self {
        self.auto_accept_mode = Some(mode);
        self
    }

    pub fn with_status_sender(mut self, sender: mpsc::UnboundedSender<ExecutionStatus>) -> Self {
        self.status_sender = Some(sender);
        self
    }

    pub fn build(self) -> Result<StreamingOperationExecutor> {
        Ok(StreamingOperationExecutor {
            file_executor: self.file_executor.ok_or_else(|| anyhow::anyhow!("File executor required"))?,
            ai_helpers: self.ai_helpers.ok_or_else(|| anyhow::anyhow!("AI helpers required"))?,
            auto_accept_mode: self.auto_accept_mode.ok_or_else(|| anyhow::anyhow!("Auto-accept mode required"))?,
            status_sender: self.status_sender.ok_or_else(|| anyhow::anyhow!("Status sender required"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_streaming_executor_auto_accept() {
        // Create temp directory
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Create mocks
        let file_executor = Arc::new(FileOperationExecutor::new(temp_dir.path().to_path_buf()));
        let ai_helpers = Arc::new(AIHelperEcosystem::new().await.unwrap());
        let auto_accept = Arc::new(AtomicBool::new(true));
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Create executor
        let executor = StreamingOperationExecutor::new(
            file_executor,
            ai_helpers,
            auto_accept,
            tx,
        );
        
        // Test operation
        let operation = FileOperation::Create {
            path: test_file.clone(),
            content: "Test content".to_string(),
        };
        
        // Execute
        executor.execute_inline(operation, None).await.unwrap();
        
        // Check status updates
        let status1 = rx.recv().await.unwrap();
        assert!(matches!(status1, ExecutionStatus::Executing));
        
        let status2 = rx.recv().await.unwrap();
        assert!(matches!(status2, ExecutionStatus::Completed(_)));
        
        // Verify file was created
        assert!(test_file.exists());
    }

    #[tokio::test]
    async fn test_streaming_executor_confirmation() {
        // Create temp directory
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Create mocks
        let file_executor = Arc::new(FileOperationExecutor::new(temp_dir.path().to_path_buf()));
        let ai_helpers = Arc::new(AIHelperEcosystem::new().await.unwrap());
        let auto_accept = Arc::new(AtomicBool::new(false)); // Confirmation required
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Create executor
        let executor = StreamingOperationExecutor::new(
            file_executor,
            ai_helpers,
            auto_accept,
            tx,
        );
        
        // Test operation
        let operation = FileOperation::Create {
            path: test_file.clone(),
            content: "Test content".to_string(),
        };
        
        // Create confirmation channel
        let (confirm_tx, confirm_rx) = mpsc::channel(1);
        
        // Execute in background
        let handle = tokio::spawn(async move {
            executor.execute_inline(operation, Some(confirm_rx)).await
        });
        
        // Check waiting status
        let status1 = rx.recv().await.unwrap();
        assert!(matches!(status1, ExecutionStatus::WaitingConfirmation));
        
        // Send approval
        confirm_tx.send(true).await.unwrap();
        
        // Wait for execution
        handle.await.unwrap().unwrap();
        
        // Check execution status
        let status2 = rx.recv().await.unwrap();
        assert!(matches!(status2, ExecutionStatus::Executing));
        
        let status3 = rx.recv().await.unwrap();
        assert!(matches!(status3, ExecutionStatus::Completed(_)));
        
        // Verify file was created
        assert!(test_file.exists());
    }
}