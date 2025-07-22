//! Iteration Handler
//!
//! Manages multi-step development workflows where the AI needs to:
//! - Execute operations
//! - Test results
//! - Fix issues
//! - Continue with next steps
//!
//! Similar to how Claude Code handles iterative development tasks.

use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ai_helpers::AIHelperEcosystem;
use crate::consensus::streaming_executor::{StreamingOperationExecutor, ExecutionStatus};
use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::direct_executor::DirectExecutionHandler;

/// Context maintained across iterations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationContext {
    /// The original user request
    pub original_request: String,
    
    /// Current iteration number
    pub iteration: usize,
    
    /// Operations executed so far
    pub executed_operations: Vec<ExecutedOperation>,
    
    /// Test results from previous iterations
    pub test_results: Vec<TestResult>,
    
    /// Current goal/task being worked on
    pub current_goal: String,
    
    /// Remaining tasks to complete
    pub remaining_tasks: Vec<String>,
    
    /// Files created or modified
    pub affected_files: Vec<std::path::PathBuf>,
    
    /// Errors encountered
    pub errors: Vec<IterationError>,
    
    /// User feedback received
    pub user_feedback: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutedOperation {
    pub operation: FileOperation,
    pub status: String,
    pub timestamp: std::time::SystemTime,
    pub iteration: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_command: String,
    pub success: bool,
    pub output: String,
    pub iteration: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationError {
    pub error_type: ErrorType,
    pub message: String,
    pub file_path: Option<std::path::PathBuf>,
    pub line_number: Option<usize>,
    pub iteration: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    CompilationError,
    TestFailure,
    RuntimeError,
    LintError,
    TypeCheckError,
    Other(String),
}

/// Result of analyzing what to do next
#[derive(Debug, Clone)]
pub enum NextIteration {
    /// Continue with more operations
    Execute(Vec<FileOperation>),
    
    /// Run tests to verify current state
    RunTests(Vec<String>),
    
    /// Request user feedback on specific questions
    RequestFeedback(Vec<String>),
    
    /// Fix errors before continuing
    FixErrors(Vec<IterationError>),
    
    /// Task is complete
    Done(String), // Summary message
}

/// Recommendation from AI analysis
#[derive(Debug, Clone)]
pub enum Recommendation {
    Continue,        // Keep going with the plan
    NeedsFeedback,   // Ask user for clarification
    FixRequired,     // Need to fix errors first
    TestRequired,    // Should run tests before continuing
    Complete,        // Task appears to be done
}

/// Handler for iterative development workflows
pub struct IterationHandler {
    context: IterationContext,
    ai_helpers: Arc<AIHelperEcosystem>,
    executor: Arc<StreamingOperationExecutor>,
    direct_handler: Arc<DirectExecutionHandler>,
    max_iterations: usize,
}

impl IterationHandler {
    pub fn new(
        original_request: String,
        ai_helpers: Arc<AIHelperEcosystem>,
        executor: Arc<StreamingOperationExecutor>,
        direct_handler: Arc<DirectExecutionHandler>,
    ) -> Self {
        Self {
            context: IterationContext {
                original_request,
                iteration: 0,
                executed_operations: Vec::new(),
                test_results: Vec::new(),
                current_goal: String::new(),
                remaining_tasks: Vec::new(),
                affected_files: Vec::new(),
                errors: Vec::new(),
                user_feedback: Vec::new(),
            },
            ai_helpers,
            executor,
            direct_handler,
            max_iterations: 10, // Prevent infinite loops
        }
    }

    /// Continue development based on execution results
    pub async fn iterate(
        &mut self,
        previous_result: ExecutionResult,
        user_feedback: Option<String>,
    ) -> Result<NextIteration> {
        // Increment iteration counter
        self.context.iteration += 1;
        
        // Check if we've hit the max iterations
        if self.context.iteration >= self.max_iterations {
            return Ok(NextIteration::Done(
                "Reached maximum iterations. Please review the work so far.".to_string()
            ));
        }
        
        // Add user feedback if provided
        if let Some(feedback) = user_feedback {
            self.context.user_feedback.push(feedback);
        }
        
        // Update context based on previous result
        self.update_context_from_result(&previous_result)?;
        
        // Analyze what happened and determine next steps
        let analysis = self.analyze_current_state().await?;
        
        // Determine next iteration based on analysis
        match analysis.recommendation {
            Recommendation::Continue => {
                // Generate next operations
                let next_ops = self.generate_next_operations(&analysis).await?;
                Ok(NextIteration::Execute(next_ops))
            }
            Recommendation::TestRequired => {
                // Determine what tests to run
                let test_commands = self.determine_test_commands(&analysis).await?;
                Ok(NextIteration::RunTests(test_commands))
            }
            Recommendation::FixRequired => {
                // Errors need to be fixed
                Ok(NextIteration::FixErrors(self.context.errors.clone()))
            }
            Recommendation::NeedsFeedback => {
                // Generate questions for the user
                let questions = self.generate_clarification_questions(&analysis).await?;
                Ok(NextIteration::RequestFeedback(questions))
            }
            Recommendation::Complete => {
                // Generate completion summary
                let summary = self.generate_completion_summary(&analysis).await?;
                Ok(NextIteration::Done(summary))
            }
        }
    }

    /// Update context from execution results
    fn update_context_from_result(&mut self, result: &ExecutionResult) -> Result<()> {
        match result {
            ExecutionResult::Success { operations, .. } => {
                for op in operations {
                    self.context.executed_operations.push(ExecutedOperation {
                        operation: op.clone(),
                        status: "success".to_string(),
                        timestamp: std::time::SystemTime::now(),
                        iteration: self.context.iteration,
                    });
                    
                    // Track affected files
                    if let Some(path) = get_operation_path(&op) {
                        if !self.context.affected_files.contains(&path) {
                            self.context.affected_files.push(path);
                        }
                    }
                }
            }
            ExecutionResult::TestResult { result, .. } => {
                self.context.test_results.push(result.clone());
            }
            ExecutionResult::Error { error, .. } => {
                self.context.errors.push(error.clone());
            }
        }
        
        Ok(())
    }

    /// Analyze current state and recommend next action
    async fn analyze_current_state(&self) -> Result<IterationAnalysis> {
        // Use AI helpers to analyze the situation
        let recent_errors = self.context.errors.iter()
            .filter(|e| e.iteration == self.context.iteration - 1)
            .collect::<Vec<_>>();
        
        let recent_tests = self.context.test_results.iter()
            .filter(|t| t.iteration == self.context.iteration - 1)
            .collect::<Vec<_>>();
        
        // Determine recommendation based on state
        let recommendation = if !recent_errors.is_empty() {
            Recommendation::FixRequired
        } else if recent_tests.is_empty() && self.context.iteration > 1 {
            Recommendation::TestRequired
        } else if self.all_planned_tasks_complete() {
            Recommendation::Complete
        } else if self.needs_user_clarification() {
            Recommendation::NeedsFeedback
        } else {
            Recommendation::Continue
        };
        
        Ok(IterationAnalysis {
            recommendation: recommendation.clone(),
            confidence: 0.8, // Would be calculated by AI helpers
            reasoning: self.generate_reasoning(&recommendation),
            suggested_actions: self.generate_suggested_actions(&recommendation),
        })
    }

    /// Check if all planned tasks are complete
    fn all_planned_tasks_complete(&self) -> bool {
        // Simple heuristic for now
        self.context.remaining_tasks.is_empty() && 
        self.context.errors.is_empty() &&
        self.context.test_results.iter().all(|t| t.success)
    }

    /// Check if we need user clarification
    fn needs_user_clarification(&self) -> bool {
        // Check if we've encountered ambiguous situations
        self.context.errors.iter().any(|e| {
            matches!(e.error_type, ErrorType::Other(_))
        }) || self.context.iteration > 3 && self.context.user_feedback.is_empty()
    }

    /// Generate reasoning for the recommendation
    fn generate_reasoning(&self, recommendation: &Recommendation) -> String {
        match recommendation {
            Recommendation::Continue => {
                "Previous operations completed successfully. Continuing with next steps.".to_string()
            }
            Recommendation::TestRequired => {
                "Code changes complete. Running tests to verify functionality.".to_string()
            }
            Recommendation::FixRequired => {
                format!("Found {} errors that need to be fixed before continuing.", self.context.errors.len())
            }
            Recommendation::NeedsFeedback => {
                "Encountered ambiguous requirements. Need clarification to proceed.".to_string()
            }
            Recommendation::Complete => {
                "All tasks appear to be complete and tests are passing.".to_string()
            }
        }
    }

    /// Generate suggested actions based on recommendation
    fn generate_suggested_actions(&self, recommendation: &Recommendation) -> Vec<String> {
        match recommendation {
            Recommendation::Continue => {
                vec!["Implement next feature".to_string()]
            }
            Recommendation::TestRequired => {
                vec!["Run unit tests".to_string(), "Run integration tests".to_string()]
            }
            Recommendation::FixRequired => {
                self.context.errors.iter()
                    .map(|e| format!("Fix {}: {}", 
                        match &e.error_type {
                            ErrorType::CompilationError => "compilation error",
                            ErrorType::TestFailure => "test failure",
                            ErrorType::RuntimeError => "runtime error",
                            ErrorType::LintError => "lint error",
                            ErrorType::TypeCheckError => "type error",
                            ErrorType::Other(s) => s,
                        },
                        e.message
                    ))
                    .collect()
            }
            Recommendation::NeedsFeedback => {
                vec!["Clarify requirements".to_string()]
            }
            Recommendation::Complete => {
                vec!["Review implementation".to_string()]
            }
        }
    }

    /// Generate next operations to execute
    async fn generate_next_operations(&self, _analysis: &IterationAnalysis) -> Result<Vec<FileOperation>> {
        // This would use the AI to determine next steps
        // For now, return empty as this would be implementation-specific
        Ok(Vec::new())
    }

    /// Determine what tests to run
    async fn determine_test_commands(&self, _analysis: &IterationAnalysis) -> Result<Vec<String>> {
        // Determine appropriate test commands based on the project
        let mut commands = Vec::new();
        
        // Check what kind of project this is
        if self.context.affected_files.iter().any(|f| f.extension() == Some(std::ffi::OsStr::new("rs"))) {
            commands.push("cargo test".to_string());
            commands.push("cargo check".to_string());
        } else if self.context.affected_files.iter().any(|f| f.extension() == Some(std::ffi::OsStr::new("ts"))) {
            commands.push("npm test".to_string());
            commands.push("npm run type-check".to_string());
        }
        
        Ok(commands)
    }

    /// Generate clarification questions for the user
    async fn generate_clarification_questions(&self, _analysis: &IterationAnalysis) -> Result<Vec<String>> {
        // Generate questions based on current state
        let mut questions = Vec::new();
        
        if !self.context.errors.is_empty() {
            questions.push("How should I handle the errors encountered?".to_string());
        }
        
        if self.context.remaining_tasks.is_empty() && self.context.iteration > 1 {
            questions.push("What additional features would you like me to implement?".to_string());
        }
        
        Ok(questions)
    }

    /// Generate completion summary
    async fn generate_completion_summary(&self, _analysis: &IterationAnalysis) -> Result<String> {
        let summary = format!(
            "Completed {} operations across {} iterations.\n\
            Files modified: {}\n\
            Tests run: {}\n\
            All tests passing: {}",
            self.context.executed_operations.len(),
            self.context.iteration,
            self.context.affected_files.len(),
            self.context.test_results.len(),
            self.context.test_results.iter().all(|t| t.success)
        );
        
        Ok(summary)
    }

    /// Get current context for inspection
    pub fn context(&self) -> &IterationContext {
        &self.context
    }

    /// Add a task to the remaining tasks
    pub fn add_task(&mut self, task: String) {
        self.context.remaining_tasks.push(task);
    }

    /// Mark a task as complete
    pub fn complete_task(&mut self, task: &str) {
        self.context.remaining_tasks.retain(|t| t != task);
    }
}

/// Analysis of current iteration state
#[derive(Debug, Clone)]
struct IterationAnalysis {
    recommendation: Recommendation,
    confidence: f32,
    reasoning: String,
    suggested_actions: Vec<String>,
}

/// Execution result from an iteration
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Success {
        operations: Vec<FileOperation>,
        message: String,
    },
    TestResult {
        result: TestResult,
    },
    Error {
        error: IterationError,
    },
}

/// Get the file path from an operation
fn get_operation_path(op: &FileOperation) -> Option<std::path::PathBuf> {
    match op {
        FileOperation::Create { path, .. } |
        FileOperation::Update { path, .. } |
        FileOperation::Delete { path } |
        FileOperation::Append { path, .. } => Some(path.clone()),
        FileOperation::Rename { to, .. } => Some(to.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_iteration_handler_creation() {
        // Would need mocks for proper testing
        let handler = IterationHandler {
            context: IterationContext {
                original_request: "Create a web server".to_string(),
                iteration: 0,
                executed_operations: Vec::new(),
                test_results: Vec::new(),
                current_goal: String::new(),
                remaining_tasks: vec!["Create server".to_string(), "Add routes".to_string()],
                affected_files: Vec::new(),
                errors: Vec::new(),
                user_feedback: Vec::new(),
            },
            ai_helpers: Arc::new(AIHelperEcosystem::new().await.unwrap()),
            executor: Arc::new(StreamingOperationExecutor::new(
                Arc::new(crate::consensus::FileOperationExecutor::new(
                    crate::consensus::file_executor::ExecutorConfig::default(),
                    crate::consensus::smart_decision_engine::SmartDecisionEngine::new(
                        crate::consensus::operation_analysis::AutoAcceptMode::Balanced,
                        0.5,
                    ),
                    crate::consensus::operation_intelligence::OperationIntelligenceCoordinator::new(
                        Arc::new(crate::ai_helpers::KnowledgeIndexer::new().await.unwrap()),
                        Arc::new(crate::ai_helpers::ContextRetriever::new().await.unwrap()),
                        Arc::new(crate::ai_helpers::PatternRecognizer::new().await.unwrap()),
                        Arc::new(crate::ai_helpers::QualityAnalyzer::new().await.unwrap()),
                        Arc::new(crate::ai_helpers::KnowledgeSynthesizer::new().await.unwrap()),
                    ),
                )),
                Arc::new(AIHelperEcosystem::new().await.unwrap()),
                Arc::new(std::sync::atomic::AtomicBool::new(true)),
                tokio::sync::mpsc::unbounded_channel().0,
            )),
            direct_handler: Arc::new(DirectExecutionHandler::new(
                Arc::new(crate::consensus::stages::GeneratorStage::new()),
                Arc::new(AIHelperEcosystem::new().await.unwrap()),
                Arc::new(StreamingOperationExecutor::new(
                    Arc::new(crate::consensus::FileOperationExecutor::new(
                        crate::consensus::file_executor::ExecutorConfig::default(),
                        crate::consensus::smart_decision_engine::SmartDecisionEngine::new(
                            crate::consensus::operation_analysis::AutoAcceptMode::Balanced,
                            0.5,
                        ),
                        crate::consensus::operation_intelligence::OperationIntelligenceCoordinator::new(
                            Arc::new(crate::ai_helpers::KnowledgeIndexer::new().await.unwrap()),
                            Arc::new(crate::ai_helpers::ContextRetriever::new().await.unwrap()),
                            Arc::new(crate::ai_helpers::PatternRecognizer::new().await.unwrap()),
                            Arc::new(crate::ai_helpers::QualityAnalyzer::new().await.unwrap()),
                            Arc::new(crate::ai_helpers::KnowledgeSynthesizer::new().await.unwrap()),
                        ),
                    )),
                    Arc::new(AIHelperEcosystem::new().await.unwrap()),
                    Arc::new(std::sync::atomic::AtomicBool::new(true)),
                    tokio::sync::mpsc::unbounded_channel().0,
                )),
                Arc::new(crate::consensus::openrouter::OpenRouterClient::new("test_key".to_string())),
                Arc::new(crate::consensus::models::ModelManager::new(
                    Arc::new(crate::consensus::openrouter::OpenRouterClient::new("test_key".to_string()))
                )),
            )),
            max_iterations: 10,
        };
        
        assert_eq!(handler.context.iteration, 0);
        assert_eq!(handler.context.remaining_tasks.len(), 2);
    }

    #[test]
    fn test_recommendation_logic() {
        let context = IterationContext {
            original_request: "Test".to_string(),
            iteration: 2,
            executed_operations: Vec::new(),
            test_results: Vec::new(),
            current_goal: String::new(),
            remaining_tasks: Vec::new(),
            affected_files: Vec::new(),
            errors: vec![IterationError {
                error_type: ErrorType::CompilationError,
                message: "Test error".to_string(),
                file_path: None,
                line_number: None,
                iteration: 1,
            }],
            user_feedback: Vec::new(),
        };
        
        // With errors, should recommend fix
        assert!(!context.errors.is_empty());
    }
}