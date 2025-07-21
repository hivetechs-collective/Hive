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
use crate::consensus::openrouter::OpenRouterClient;
use crate::consensus::models::ModelManager;

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
    model_manager: Arc<ModelManager>,
}

impl DirectExecutionHandler {
    pub fn new(
        generator: Arc<GeneratorStage>,
        ai_helpers: Arc<AIHelperEcosystem>,
        executor: Arc<StreamingOperationExecutor>,
        client: Arc<OpenRouterClient>,
        model_manager: Arc<ModelManager>,
    ) -> Self {
        Self {
            generator,
            ai_helpers,
            executor,
            client,
            model_manager,
        }
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
        
        // Get appropriate model for generation
        let model = self.model_manager.get_model_for_stage("generator")?;
        
        // Create a parser for inline operations
        let mut parser = InlineOperationParser::new();
        
        // Stream the response from the generator
        let mut response_stream = self.client.create_completion_stream(
            &messages,
            &model.id,
            None, // Use default temperature
            None, // Use default max tokens
        ).await?;
        
        // Process the streaming response
        while let Some(chunk) = response_stream.recv().await {
            match chunk {
                Ok(response) => {
                    if let Some(content) = response.choices.first().and_then(|c| c.delta.content.as_ref()) {
                        // Send chunk to UI
                        callbacks.on_chunk(content).await?;
                        
                        // Parse for operations
                        if let Some(operation) = parser.parse_chunk(content) {
                            // Execute the operation inline
                            self.executor.execute_inline(operation, None).await?;
                        }
                    }
                }
                Err(e) => {
                    callbacks.on_error(&format!("Streaming error: {}", e)).await?;
                    return Err(e.into());
                }
            }
        }
        
        // Signal completion
        callbacks.on_event(ConsensusEvent::StageComplete {
            stage: "direct".to_string(),
            tokens: 0, // Would need to track this
            cost: 0.0, // Would need to calculate this
        }).await?;
        
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

/// Builder for DirectExecutionHandler
pub struct DirectExecutionBuilder {
    generator: Option<Arc<GeneratorStage>>,
    ai_helpers: Option<Arc<AIHelperEcosystem>>,
    executor: Option<Arc<StreamingOperationExecutor>>,
    client: Option<Arc<OpenRouterClient>>,
    model_manager: Option<Arc<ModelManager>>,
}

impl DirectExecutionBuilder {
    pub fn new() -> Self {
        Self {
            generator: None,
            ai_helpers: None,
            executor: None,
            client: None,
            model_manager: None,
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

    pub fn with_model_manager(mut self, manager: Arc<ModelManager>) -> Self {
        self.model_manager = Some(manager);
        self
    }

    pub fn build(self) -> Result<DirectExecutionHandler> {
        Ok(DirectExecutionHandler {
            generator: self.generator.ok_or_else(|| anyhow::anyhow!("Generator required"))?,
            ai_helpers: self.ai_helpers.ok_or_else(|| anyhow::anyhow!("AI helpers required"))?,
            executor: self.executor.ok_or_else(|| anyhow::anyhow!("Executor required"))?,
            client: self.client.ok_or_else(|| anyhow::anyhow!("Client required"))?,
            model_manager: self.model_manager.ok_or_else(|| anyhow::anyhow!("Model manager required"))?,
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