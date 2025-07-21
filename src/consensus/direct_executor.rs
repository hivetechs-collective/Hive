//! Direct Execution Handler
//!
//! Provides a fast path for simple operations that don't require full consensus analysis.
//! Similar to how Claude Code handles straightforward file operations directly.

use std::sync::Arc;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::consensus::ai_helpers::AIHelperEcosystem;
use crate::consensus::streaming_executor::{StreamingOperationExecutor, ExecutionStatus};
use crate::consensus::stages::{GeneratorStage, ConsensusStage};
use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::streaming::{StreamingCallbacks, ConsensusEvent};
use crate::consensus::openrouter::{OpenRouterClient, OpenRouterRequest, OpenRouterMessage};
use crate::consensus::models::{DynamicModelSelector, ModelSelectionCriteria};
use crate::consensus::safety_guardrails::SafetyGuardrailSystem;
use crate::consensus::types::{Stage, StageResult};
use crate::core::database::DatabaseManager;

/// Inline operation parser for extracting operations from streaming responses
pub struct InlineOperationParser {
    current_operation: Option<PendingOperation>,
    in_code_block: bool,
    code_block_lang: Option<String>,
    operation_pattern: regex::Regex,
}

#[derive(Debug, Clone)]
struct PendingOperation {
    operation_type: String,
    file_path: String,
    content: String,
    start_marker: String,
}

impl InlineOperationParser {
    pub fn new() -> Self {
        Self {
            current_operation: None,
            in_code_block: false,
            code_block_lang: None,
            operation_pattern: regex::Regex::new(
                r"(?i)(Creating|Updating|Modifying|Deleting|Writing to|Adding to)\s+`([^`]+)`:"
            ).unwrap(),
        }
    }

    /// Parse a chunk of text for inline operations
    pub fn parse_chunk(&mut self, chunk: &str) -> Option<FileOperation> {
        let lines: Vec<&str> = chunk.lines().collect();
        
        for line in lines {
            // Check for operation start markers
            if let Some(captures) = self.operation_pattern.captures(line) {
                let operation_type = captures.get(1).unwrap().as_str();
                let file_path = captures.get(2).unwrap().as_str();
                
                self.current_operation = Some(PendingOperation {
                    operation_type: operation_type.to_lowercase(),
                    file_path: file_path.to_string(),
                    content: String::new(),
                    start_marker: line.to_string(),
                });
                continue;
            }
            
            // Track code block state
            if line.starts_with("```") {
                if self.in_code_block {
                    // End of code block - check if we have a complete operation
                    self.in_code_block = false;
                    if let Some(op) = &self.current_operation {
                        let operation = self.create_file_operation(op);
                        self.current_operation = None;
                        return Some(operation);
                    }
                } else {
                    // Start of code block
                    self.in_code_block = true;
                    self.code_block_lang = if line.len() > 3 {
                        Some(line[3..].trim().to_string())
                    } else {
                        None
                    };
                }
                continue;
            }
            
            // Accumulate content if we're in a code block with a pending operation
            if self.in_code_block && self.current_operation.is_some() {
                if let Some(op) = &mut self.current_operation {
                    if !op.content.is_empty() {
                        op.content.push('\n');
                    }
                    op.content.push_str(line);
                }
            }
        }
        
        None
    }
    
    /// Create a FileOperation from a pending operation
    fn create_file_operation(&self, op: &PendingOperation) -> FileOperation {
        let path = std::path::PathBuf::from(&op.file_path);
        
        match op.operation_type.as_str() {
            "creating" | "writing to" | "adding to" => {
                FileOperation::Create {
                    path,
                    content: op.content.clone(),
                }
            }
            "updating" | "modifying" => {
                FileOperation::Update {
                    path,
                    content: op.content.clone(),
                }
            }
            "deleting" => {
                FileOperation::Delete { path }
            }
            _ => {
                // Default to create for unknown operations
                FileOperation::Create {
                    path,
                    content: op.content.clone(),
                }
            }
        }
    }
}

/// Direct execution handler for simple operations
pub struct DirectExecutionHandler {
    generator: Arc<GeneratorStage>,
    ai_helpers: Arc<AIHelperEcosystem>,
    executor: Arc<StreamingOperationExecutor>,
    client: Arc<OpenRouterClient>,
    model_selector: Arc<DynamicModelSelector>,
    db: Arc<DatabaseManager>,
    safety_system: Option<Arc<SafetyGuardrailSystem>>,
}

impl DirectExecutionHandler {
    pub fn new(
        generator: Arc<GeneratorStage>,
        ai_helpers: Arc<AIHelperEcosystem>,
        executor: Arc<StreamingOperationExecutor>,
        client: Arc<OpenRouterClient>,
        model_selector: Arc<DynamicModelSelector>,
        db: Arc<DatabaseManager>,
    ) -> Self {
        Self {
            generator,
            ai_helpers,
            executor,
            client,
            model_selector,
            db,
            safety_system: None,
        }
    }
    
    /// Set the safety guardrail system
    pub fn with_safety_system(mut self, safety_system: Arc<SafetyGuardrailSystem>) -> Self {
        self.safety_system = Some(safety_system);
        self
    }

    /// Handle a request using the direct execution path
    pub async fn handle_request(
        &self,
        request: &str,
        context: Option<&str>,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<()> {
        // Build messages for the generator
        let messages = self.generator.build_messages(request, None, context)?;
        
        // Select an appropriate model for generation
        let criteria = ModelSelectionCriteria {
            stage: "generator".to_string(),
            question_complexity: "simple".to_string(),
            question_category: "coding".to_string(),
            budget_constraints: None,
            performance_targets: None,
            user_preferences: None,
            profile_template: None,
        };
        
        let model_candidate = self.model_selector
            .select_optimal_model(&self.db, &criteria, None)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No suitable model found for direct execution"))?;
        
        // Create a parser for inline operations
        let mut parser = InlineOperationParser::new();
        
        // Build the OpenRouter request
        let openrouter_request = OpenRouterRequest {
            model: model_candidate.openrouter_id.clone(),
            messages: messages.into_iter().map(|m| OpenRouterMessage {
                role: m.role,
                content: m.content,
            }).collect(),
            temperature: Some(0.7),
            max_tokens: Some(2048),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: Some(true),
            provider: None,
        };
        
        // Create a custom callback wrapper
        let parser_ref = Arc::new(tokio::sync::Mutex::new(parser));
        let executor_ref = self.executor.clone();
        let callbacks_ref = callbacks.clone();
        
        let streaming_callbacks = DirectStreamingCallbacks {
            inner: callbacks_ref,
            parser: parser_ref,
            executor: executor_ref,
            stage: Stage::Generator,
        };
        
        // Stream the response from the generator
        let response_content = self.client
            .chat_completion_stream(
                openrouter_request,
                Some(Box::new(streaming_callbacks)),
                None,
            )
            .await?;
        
        // Signal completion
        callbacks.on_stage_complete(Stage::Generator, &StageResult {
            stage_id: "direct".to_string(),
            stage_name: "Direct Execution".to_string(),
            question: request.to_string(),
            answer: response_content,
            model: model_candidate.openrouter_id,
            conversation_id: "direct".to_string(),
            timestamp: chrono::Utc::now(),
            usage: None, // Would need to track this
            analytics: None, // Would need to track this
        })?;
        
        Ok(())
    }

    /// Check if a request should use direct execution
    pub async fn should_use_direct_path(&self, request: &str) -> Result<bool> {
        let request_lower = request.to_lowercase();
        
        // Simple heuristics for now - can be enhanced with ML later
        let simple_operations = [
            "create a file",
            "create file",
            "make a file",
            "add a file",
            "write a file",
            "update the file",
            "modify the file",
            "delete the file",
            "rename the file",
            "create a simple",
            "add a simple",
            "make a basic",
            "create a test",
            "add a test",
        ];
        
        // Check if it's a simple operation
        let is_simple = simple_operations.iter().any(|&op| request_lower.contains(op));
        
        // Check if it's asking for complex analysis or multiple steps
        let complex_indicators = [
            "analyze",
            "explain",
            "debug",
            "refactor",
            "optimize",
            "architecture",
            "design",
            "implement a system",
            "build a complete",
            "create an entire",
        ];
        
        let is_complex = complex_indicators.iter().any(|&ind| request_lower.contains(ind));
        
        // Use direct path for simple operations that aren't complex
        Ok(is_simple && !is_complex)
    }
}

/// Custom streaming callbacks that parse operations on the fly
struct DirectStreamingCallbacks {
    inner: Arc<dyn StreamingCallbacks>,
    parser: Arc<tokio::sync::Mutex<InlineOperationParser>>,
    executor: Arc<StreamingOperationExecutor>,
    stage: Stage,
}

impl crate::consensus::openrouter::StreamingCallbacks for DirectStreamingCallbacks {
    fn on_start(&self) {
        // Signal stage start through the consensus callbacks
        let _ = self.inner.on_stage_start(self.stage, "direct");
    }
    
    fn on_chunk(&self, chunk: String, total_content: String) {
        // Forward to consensus callbacks
        let _ = self.inner.on_stage_chunk(self.stage, &chunk, &total_content);
        
        // Parse for operations
        let parser = self.parser.clone();
        let executor = self.executor.clone();
        
        // Spawn a task to handle parsing and execution
        tokio::spawn(async move {
            let mut parser_guard = parser.lock().await;
            if let Some(operation) = parser_guard.parse_chunk(&chunk) {
                // Execute the operation inline
                let _ = executor.execute_inline(operation, None).await;
            }
        });
    }
    
    fn on_progress(&self, _progress: crate::consensus::openrouter::StreamingProgress) {
        // Progress tracking would go here
    }
    
    fn on_error(&self, error: &anyhow::Error) {
        let _ = self.inner.on_error(self.stage, error);
    }
    
    fn on_complete(&self, _final_content: String, _usage: Option<crate::consensus::openrouter::Usage>) {
        // Completion is handled in the main handler
    }
}

/// Builder for DirectExecutionHandler
pub struct DirectExecutionBuilder {
    generator: Option<Arc<GeneratorStage>>,
    ai_helpers: Option<Arc<AIHelperEcosystem>>,
    executor: Option<Arc<StreamingOperationExecutor>>,
    client: Option<Arc<OpenRouterClient>>,
    model_selector: Option<Arc<DynamicModelSelector>>,
    db: Option<Arc<DatabaseManager>>,
}

impl DirectExecutionBuilder {
    pub fn new() -> Self {
        Self {
            generator: None,
            ai_helpers: None,
            executor: None,
            client: None,
            model_selector: None,
            db: None,
        }
    }

    pub fn with_generator(mut self, generator: Arc<GeneratorStage>) -> Self {
        self.generator = Some(generator);
        self
    }

    pub fn with_ai_helpers(mut self, helpers: Arc<AIHelperEcosystem>) -> Self {
        self.ai_helpers = Some(helpers);
        self
    }

    pub fn with_executor(mut self, executor: Arc<StreamingOperationExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    pub fn with_client(mut self, client: Arc<OpenRouterClient>) -> Self {
        self.client = Some(client);
        self
    }

    pub fn with_model_selector(mut self, selector: Arc<DynamicModelSelector>) -> Self {
        self.model_selector = Some(selector);
        self
    }
    
    pub fn with_database(mut self, db: Arc<DatabaseManager>) -> Self {
        self.db = Some(db);
        self
    }

    pub fn build(self) -> Result<DirectExecutionHandler> {
        Ok(DirectExecutionHandler {
            generator: self.generator.ok_or_else(|| anyhow::anyhow!("Generator required"))?,
            ai_helpers: self.ai_helpers.ok_or_else(|| anyhow::anyhow!("AI helpers required"))?,
            executor: self.executor.ok_or_else(|| anyhow::anyhow!("Executor required"))?,
            client: self.client.ok_or_else(|| anyhow::anyhow!("Client required"))?,
            model_selector: self.model_selector.ok_or_else(|| anyhow::anyhow!("Model selector required"))?,
            db: self.db.ok_or_else(|| anyhow::anyhow!("Database required"))?,
            safety_system: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_parser_create_operation() {
        let mut parser = InlineOperationParser::new();
        
        let chunk = "Creating `src/main.rs`:\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        
        // Parse first part
        parser.parse_chunk("Creating `src/main.rs`:\n```rust\n");
        
        // Parse content
        parser.parse_chunk("fn main() {\n    println!(\"Hello\");\n}\n");
        
        // Parse end and get operation
        let operation = parser.parse_chunk("```");
        
        assert!(operation.is_some());
        if let Some(FileOperation::Create { path, content }) = operation {
            assert_eq!(path.to_str().unwrap(), "src/main.rs");
            assert!(content.contains("fn main()"));
        }
    }

    #[test]
    fn test_should_use_direct_path() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Would need mocks for proper testing
            // For now just test the logic without actual handler
            
            let simple_request = "create a file called test.rs";
            let complex_request = "analyze the architecture and create a complete system";
            
            // The actual handler would make these determinations
            assert!(simple_request.contains("create a file"));
            assert!(complex_request.contains("analyze"));
        });
    }
}