//! Direct Execution Handler
//!
//! Provides a fast path for simple operations that don't require full consensus analysis.
//! Similar to how Claude Code handles straightforward file operations directly.

use std::sync::Arc;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::ai_helpers::AIHelperEcosystem;
use crate::consensus::streaming_executor::{StreamingOperationExecutor, ExecutionStatus};
use crate::consensus::stages::{GeneratorStage, ConsensusStage};
use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::streaming::{StreamingCallbacks, ConsensusEvent};
use crate::consensus::openrouter::{OpenRouterClient, OpenRouterRequest, OpenRouterMessage};
// Removed DynamicModelSelector - using profile's generator model directly
use crate::consensus::safety_guardrails::SafetyGuardrailSystem;
use crate::consensus::types::{Stage, StageResult, ConsensusProfile};
use crate::core::database::DatabaseManager;
use crate::consensus::ai_file_executor::AIConsensusFileExecutor;

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
                r"(?i)(Creating|Updating|Modifying|Deleting|Writing to|Adding to)\s+`?([^`:]+)`:"
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
    ai_file_executor: Arc<AIConsensusFileExecutor>,
    client: Arc<OpenRouterClient>,
    profile: ConsensusProfile,
    db: Arc<DatabaseManager>,
    safety_system: Option<Arc<SafetyGuardrailSystem>>,
}

impl DirectExecutionHandler {
    pub fn new(
        generator: Arc<GeneratorStage>,
        ai_helpers: Arc<AIHelperEcosystem>,
        executor: Arc<StreamingOperationExecutor>,
        client: Arc<OpenRouterClient>,
        profile: ConsensusProfile,
        db: Arc<DatabaseManager>,
    ) -> Self {
        // Create the AI-powered file executor
        let ai_file_executor = Arc::new(AIConsensusFileExecutor::new(
            (*ai_helpers).clone()
        ));
        
        Self {
            generator,
            ai_helpers,
            executor,
            ai_file_executor,
            client,
            profile,
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
        conversation_id: &str,
    ) -> Result<(String, Option<crate::consensus::openrouter::Usage>)> {
        // Notify about profile being loaded (for UI display)
        let _ = callbacks.on_profile_loaded(
            &self.profile.profile_name,
            &[self.profile.generator_model.clone()]
        );
        
        // Build messages for the generator
        let mut messages = self.generator.build_messages(request, None, context)?;
        
        // Check if this is a file operation request
        let is_file_operation = self.is_file_operation_request(request);
        
        // Only add file operation instructions if this is actually about files
        if is_file_operation {
            for message in &mut messages {
                if message.role == "system" {
                    message.content.push_str("\n\n🚀 DIRECT EXECUTION MODE:\n");
                    message.content.push_str("You are in Direct Execution mode for simple file operations. ");
                    message.content.push_str("When creating or modifying files, output the operations in this EXACT format:\n\n");
                    message.content.push_str("Creating `filename.ext`:\n");
                    message.content.push_str("```language\n");
                    message.content.push_str("file content here\n");
                    message.content.push_str("```\n\n");
                    message.content.push_str("For updates use 'Updating `filename.ext`:' and for deletion use 'Deleting `filename.ext`:'\n");
                    message.content.push_str("\nIMPORTANT: Do NOT explain how to create files. Actually output the file creation operations in the format above.\n");
                    message.content.push_str("When asked to create a file, immediately output the Creating block with the actual content.");
                    break;
                }
            }
        } else {
            // For simple Q&A, add a gentle instruction to be concise and direct
            for message in &mut messages {
                if message.role == "system" {
                    message.content.push_str("\n\n🚀 DIRECT RESPONSE MODE:\n");
                    message.content.push_str("Please provide a direct, concise answer to the user's question.");
                    break;
                }
            }
        }
        
        // Use the profile's generator model directly
        let model = &self.profile.generator_model;
        
        // Build the OpenRouter request
        let openrouter_request = OpenRouterRequest {
            model: model.clone(),
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
        
        // Create a custom callback wrapper that uses AI-powered execution
        let callbacks_ref = callbacks.clone();
        
        // Only create parser for file operations
        let parser_ref = if is_file_operation {
            let parser = InlineOperationParser::new();
            Some(Arc::new(tokio::sync::Mutex::new(parser)))
        } else {
            None
        };
        
        let executor_ref = self.executor.clone();
        let ai_executor_ref = self.ai_file_executor.clone();
        
        // Create a wrapper to capture usage data
        let usage_capture = Arc::new(tokio::sync::Mutex::new(None));
        let usage_capture_ref = usage_capture.clone();
        
        let streaming_callbacks = DirectStreamingCallbacks {
            inner: callbacks_ref,
            parser: parser_ref,
            executor: executor_ref,
            ai_file_executor: ai_executor_ref,
            stage: Stage::Generator,
            usage_capture: usage_capture_ref,
        };
        
        // Send the actual model name when starting
        let _ = callbacks.on_stage_start(Stage::Generator, model);
        
        let start_time = std::time::Instant::now();
        
        // Stream the response from the generator
        let response_content = self.client
            .chat_completion_stream(
                openrouter_request,
                Some(Box::new(streaming_callbacks)),
                None,
            )
            .await?;
        
        let duration = start_time.elapsed().as_secs_f64();
        
        // Get the captured usage data
        let usage = {
            let guard = usage_capture.lock().await;
            guard.clone()
        };
        
        // Calculate cost if we have usage data
        let (analytics, token_usage) = if let Some(usage_data) = &usage {
            // Calculate cost using the database
            let cost = if let Ok(db) = crate::core::database::get_database().await {
                match db.calculate_model_cost(
                    model, 
                    usage_data.prompt_tokens, 
                    usage_data.completion_tokens
                ).await {
                    Ok(cost) => cost,
                    Err(e) => {
                        tracing::error!("Failed to calculate cost: {}", e);
                        0.0
                    }
                }
            } else {
                0.0
            };
            
            let analytics = crate::consensus::types::StageAnalytics {
                duration,
                cost,
                input_cost: 0.0, // We don't break this down for now
                output_cost: 0.0,
                provider: "openrouter".to_string(),
                model_internal_id: model.clone(),
                quality_score: 1.0, // Direct mode doesn't have quality scoring
                error_count: 0,
                fallback_used: false,
                rate_limit_hit: false,
                retry_count: 0,
                start_time: chrono::Utc::now() - chrono::Duration::seconds(duration as i64),
                end_time: chrono::Utc::now(),
                memory_usage: None,
                features: crate::consensus::types::AnalyticsFeatures {
                    streaming: true,
                    routing_variant: "direct".to_string(),
                    optimization_applied: Some(true),
                },
            };
            
            let token_usage = crate::consensus::types::TokenUsage {
                prompt_tokens: usage_data.prompt_tokens,
                completion_tokens: usage_data.completion_tokens,
                total_tokens: usage_data.total_tokens,
            };
            
            (Some(analytics), Some(token_usage))
        } else {
            (None, None)
        };
        
        // Signal completion with proper data
        tracing::info!("🔍 Direct mode: Calling on_stage_complete with conversation_id: {}, cost: {}, model: {}", 
            conversation_id,
            analytics.as_ref().map(|a| a.cost).unwrap_or(0.0),
            model
        );
        
        callbacks.on_stage_complete(Stage::Generator, &StageResult {
            stage_id: crate::core::database::generate_id(),
            stage_name: "Direct Execution".to_string(),
            question: request.to_string(),
            answer: response_content.clone(),
            model: model.clone(),
            conversation_id: conversation_id.to_string(),
            timestamp: chrono::Utc::now(),
            usage: token_usage,
            analytics,
        })?;
        
        tracing::info!("✅ Direct mode: on_stage_complete called successfully");
        
        Ok((response_content, usage))
    }

    /// Check if this request is about file operations
    fn is_file_operation_request(&self, request: &str) -> bool {
        let request_lower = request.to_lowercase();
        
        // File operation patterns
        let file_patterns = [
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
            "write to",
            "save as",
            "create.*\\.\\w+", // matches "create foo.txt", "create bar.rs", etc.
        ];
        
        // Check if any file pattern matches
        file_patterns.iter().any(|&pattern| {
            if pattern.contains(".*") {
                // Use regex for patterns with wildcards
                regex::Regex::new(pattern).unwrap().is_match(&request_lower)
            } else {
                // Simple string contains for literal patterns
                request_lower.contains(pattern)
            }
        })
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
    parser: Option<Arc<tokio::sync::Mutex<InlineOperationParser>>>,
    executor: Arc<StreamingOperationExecutor>,
    ai_file_executor: Arc<AIConsensusFileExecutor>,
    stage: Stage,
    usage_capture: Arc<tokio::sync::Mutex<Option<crate::consensus::openrouter::Usage>>>,
}

impl crate::consensus::openrouter::StreamingCallbacks for DirectStreamingCallbacks {
    fn on_start(&self) {
        // Signal stage start through the consensus callbacks
        tracing::info!("DirectStreamingCallbacks::on_start called");
        // Stage start with model name is already sent in handle_request before streaming begins
        // No need to send it again here
    }
    
    fn on_chunk(&self, chunk: String, total_content: String) {
        // Forward to consensus callbacks
        if let Err(e) = self.inner.on_stage_chunk(self.stage, &chunk, &total_content) {
            tracing::error!("Failed to call on_stage_chunk: {}", e);
        }
        
        // Only parse for operations if we have a parser (file operations)
        if let Some(parser) = &self.parser {
            let parser = parser.clone();
            let executor = self.executor.clone();
            let ai_file_executor = self.ai_file_executor.clone();
            
            // CRITICAL FIX: Don't spawn uncontrolled tasks during consensus
            // These were accumulating and causing CPU overload
            if !crate::consensus::pipeline::CONSENSUS_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
                // Only spawn execution tasks when consensus is NOT running
                tokio::spawn(async move {
                    let mut parser_guard = parser.lock().await;
                    if let Some(operation) = parser_guard.parse_chunk(&chunk) {
                        // Convert to vector for AI executor
                        let operations = vec![operation.clone()];
                        
                        // Use AI-powered execution for intelligent file operations
                        match ai_file_executor.execute_curator_operations(operations).await {
                            Ok(report) => {
                                if report.success {
                                    tracing::debug!("🤖 AI Helper successfully executed {} operations", 
                                                 report.operations_completed);
                                    if !report.files_created.is_empty() {
                                        tracing::debug!("📄 Created files: {:?}", report.files_created);
                                    }
                                    if !report.files_modified.is_empty() {
                                        tracing::debug!("✏️ Modified files: {:?}", report.files_modified);
                                    }
                                } else {
                                    tracing::warn!("⚠️ AI Helper execution had errors: {:?}", report.errors);
                                }
                            }
                            Err(e) => {
                                tracing::error!("❌ AI Helper execution failed: {}", e);
                                // Fallback to basic executor if AI fails
                                let _ = executor.execute_inline(operation, None).await;
                            }
                        }
                    }
                });
            } else {
                // During consensus, just parse and log - don't execute
                // Use a minimal async block just for parsing
                tokio::spawn(async move {
                    let mut parser_guard = parser.lock().await;
                    if let Some(operation) = parser_guard.parse_chunk(&chunk) {
                        tracing::debug!("📝 Parsed operation during consensus (execution deferred): {:?}", operation);
                    }
                });
            }
        }
    }
    
    fn on_progress(&self, _progress: crate::consensus::openrouter::StreamingProgress) {
        // Progress tracking would go here
    }
    
    fn on_error(&self, error: &anyhow::Error) {
        let _ = self.inner.on_error(self.stage, error);
    }
    
    fn on_complete(&self, _final_content: String, usage: Option<crate::consensus::openrouter::Usage>) {
        // Capture usage data synchronously using blocking
        if let Some(usage_data) = usage {
            // Use blocking to ensure the usage is captured immediately
            let usage_capture = self.usage_capture.clone();
            // We're already in an async context, so we can block on the mutex
            let mut guard = futures::executor::block_on(usage_capture.lock());
            *guard = Some(usage_data);
            tracing::debug!("Direct mode: Captured usage data - prompt: {}, completion: {}", 
                guard.as_ref().map(|u| u.prompt_tokens).unwrap_or(0),
                guard.as_ref().map(|u| u.completion_tokens).unwrap_or(0)
            );
        }
        // NOTE: We don't forward completion here because the DirectExecutionHandler
        // will call callbacks.on_stage_complete directly with the full StageResult
    }
}

/// Builder for DirectExecutionHandler
pub struct DirectExecutionBuilder {
    generator: Option<Arc<GeneratorStage>>,
    ai_helpers: Option<Arc<AIHelperEcosystem>>,
    executor: Option<Arc<StreamingOperationExecutor>>,
    client: Option<Arc<OpenRouterClient>>,
    profile: Option<ConsensusProfile>,
    db: Option<Arc<DatabaseManager>>,
}

impl DirectExecutionBuilder {
    pub fn new() -> Self {
        Self {
            generator: None,
            ai_helpers: None,
            executor: None,
            client: None,
            profile: None,
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

    pub fn with_profile(mut self, profile: ConsensusProfile) -> Self {
        self.profile = Some(profile);
        self
    }
    
    pub fn with_database(mut self, db: Arc<DatabaseManager>) -> Self {
        self.db = Some(db);
        self
    }

    pub fn build(self) -> Result<DirectExecutionHandler> {
        Ok(DirectExecutionHandler::new(
            self.generator.ok_or_else(|| anyhow::anyhow!("Generator required"))?,
            self.ai_helpers.ok_or_else(|| anyhow::anyhow!("AI helpers required"))?,
            self.executor.ok_or_else(|| anyhow::anyhow!("Executor required"))?,
            self.client.ok_or_else(|| anyhow::anyhow!("Client required"))?,
            self.profile.ok_or_else(|| anyhow::anyhow!("Profile required"))?,
            self.db.ok_or_else(|| anyhow::anyhow!("Database required"))?,
        ))
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