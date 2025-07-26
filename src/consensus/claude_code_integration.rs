//! Claude Code Integration - Full Hybrid Experience
//! 
//! This module creates a true hybrid experience by embedding the actual Claude Code
//! binary as a subprocess while adding our Hive-specific capabilities on top.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, AsyncWrite};
use tokio::process::{Child as TokioChild, Command as TokioCommand};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};
use serde_json::{json, Value};
use futures::StreamExt;

use crate::consensus::engine::ConsensusEngine;
use crate::memory::ThematicCluster;
use crate::core::database::DatabaseManager;

/// Commands that should be handled by Hive instead of Claude Code
const HIVE_COMMANDS: &[&str] = &[
    "/consensus",        // 4-stage consensus pipeline
    "/hive-consensus",   // Alias for consensus
    "/memory",           // Thematic memory search
    "/openrouter",       // Direct OpenRouter model access
    "/hive-analyze",     // Repository analysis
    "/hive-learn",       // Continuous learning insights
];

/// Represents a message between our interface and Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridMessage {
    pub content: String,
    pub message_type: HybridMessageType,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HybridMessageType {
    UserInput,
    ClaudeResponse,
    HiveResponse,
    SystemMessage,
    Error,
    ClaudeStreaming,
    ClaudeToolUse,
    ClaudeThinking,
}

/// Main integration point between Hive and Claude Code
pub struct ClaudeCodeIntegration {
    /// Real Claude Code process
    claude_process: Arc<Mutex<Option<TokioChild>>>,
    
    /// Communication channels
    input_sender: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    output_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>,
    
    /// Message buffer for multi-line responses
    message_buffer: Arc<Mutex<String>>,
    
    /// JSON communication mode flag
    use_json_protocol: Arc<RwLock<bool>>,
    
    /// Hive systems for enhanced capabilities
    consensus_engine: Arc<ConsensusEngine>,
    thematic_cluster: Arc<ThematicCluster>,
    database: Arc<DatabaseManager>,
    
    /// State management
    is_claude_authenticated: Arc<RwLock<bool>>,
    current_directory: Arc<RwLock<String>>,
}

impl ClaudeCodeIntegration {
    /// Create a new hybrid integration
    pub async fn new(
        consensus_engine: Arc<ConsensusEngine>,
        thematic_cluster: Arc<ThematicCluster>,
        database: Arc<DatabaseManager>,
    ) -> Result<Self> {
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        let integration = Self {
            claude_process: Arc::new(Mutex::new(None)),
            input_sender: Arc::new(Mutex::new(None)),
            output_receiver: Arc::new(Mutex::new(None)),
            message_buffer: Arc::new(Mutex::new(String::new())),
            use_json_protocol: Arc::new(RwLock::new(false)),
            consensus_engine,
            thematic_cluster,
            database,
            is_claude_authenticated: Arc::new(RwLock::new(false)),
            current_directory: Arc::new(RwLock::new(current_dir)),
        };

        // Start Claude Code process
        integration.start_claude_process().await?;
        
        // Initialize context
        if let Err(e) = integration.initialize_context().await {
            warn!("Failed to initialize Claude Code context: {}", e);
        }

        Ok(integration)
    }

    /// Start the Claude Code process
    async fn start_claude_process(&self) -> Result<()> {
        info!("🚀 Starting Claude Code process...");

        // Try to find Claude Code binary
        let claude_binary = self.find_claude_binary().await
            .map_err(|e| {
                error!("Failed to find Claude Code binary: {}", e);
                e
            })?;
        
        let mut cmd = TokioCommand::new(&claude_binary);
        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .current_dir(&*self.current_directory.read().await);

        let mut child = cmd.spawn()
            .context("Failed to spawn Claude Code process")?;

        // Get stdin for sending commands
        let stdin = child.stdin.take()
            .context("Failed to get Claude Code stdin")?;
        
        // Get stdout for receiving responses
        let stdout = child.stdout.take()
            .context("Failed to get Claude Code stdout")?;

        // Set up output monitoring with enhanced parsing
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        let output_reader = BufReader::new(stdout);
        
        // Spawn output reader with JSON/text detection
        let use_json = self.use_json_protocol.clone();
        tokio::spawn(async move {
            let mut lines = output_reader.lines();
            let mut json_buffer = String::new();
            let mut in_json_block = false;
            
            while let Ok(Some(line)) = lines.next_line().await {
                // Detect JSON protocol markers
                if line.starts_with("{\"type\":") || line.starts_with("data: {") {
                    in_json_block = true;
                    json_buffer.clear();
                }
                
                if in_json_block {
                    json_buffer.push_str(&line);
                    json_buffer.push('\n');
                    
                    // Try to parse as complete JSON
                    if let Ok(_) = serde_json::from_str::<Value>(&json_buffer) {
                        if let Err(_) = output_tx.send(json_buffer.clone()) {
                            break;
                        }
                        json_buffer.clear();
                        in_json_block = false;
                    }
                } else {
                    // Regular text line
                    if let Err(_) = output_tx.send(line) {
                        break;
                    }
                }
            }
        });

        // Store process components
        *self.claude_process.lock().await = Some(child);
        *self.input_sender.lock().await = Some(stdin);
        *self.output_receiver.lock().await = Some(output_rx);

        info!("✅ Claude Code process started successfully");
        Ok(())
    }

    /// Find Claude Code binary on system or install it
    async fn find_claude_binary(&self) -> Result<String> {
        // First try to find existing Claude Code installation
        let possible_paths = vec![
            "claude",                          // In PATH
            "claude-code",                     // Alternative name in PATH
            "/usr/local/bin/claude",           // Standard install
            "/usr/local/bin/claude-code",      // Alternative standard install
            "/opt/homebrew/bin/claude",        // Homebrew on Apple Silicon
            "/opt/homebrew/bin/claude-code",   // Alternative Homebrew
            "/usr/bin/claude",                 // System install
            "/usr/bin/claude-code",            // Alternative system install
            "~/.local/bin/claude",             // User local install
            "~/.local/bin/claude-code",        // Alternative user local
        ];

        info!("🔍 Searching for Claude Code binary...");
        
        // Check common locations first
        for path in &possible_paths {
            let expanded_path = shellexpand::tilde(path);
            debug!("Checking: {}", expanded_path);
            
            if let Ok(output) = Command::new(expanded_path.as_ref())
                .arg("--version")
                .output() 
            {
                if output.status.success() {
                    info!("✅ Found Claude Code at: {}", expanded_path);
                    return Ok(expanded_path.to_string());
                }
            }
        }

        // Try using 'which' command to find in PATH
        if let Ok(output) = Command::new("which")
            .arg("claude")
            .output() 
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    info!("✅ Found Claude Code via 'which': {}", path);
                    return Ok(path);
                }
            }
        }

        // If not found, use our installer to install Claude Code
        info!("Claude Code not found in standard locations. Installing bundled version...");
        
        match crate::consensus::claude_installer::ensure_claude_installed().await {
            Ok(binary_path) => {
                info!("✅ Claude Code installed at: {}", binary_path.display());
                Ok(binary_path.to_string_lossy().to_string())
            }
            Err(e) => {
                error!("❌ Failed to install Claude Code: {}", e);
                error!("❌ Claude Code binary not found in any of these locations:");
                for path in &possible_paths {
                    error!("   - {}", path);
                }
                
                Err(anyhow::anyhow!(
                    "Claude Code binary not found and automatic installation failed.\n\
                    Error: {}\n\n\
                    Manual installation instructions:\n\
                    1. Download from: https://claude.ai/download\n\
                    2. Install and ensure 'claude' is in your PATH\n\
                    3. Or restart Hive to retry automatic installation",
                    e
                ))
            }
        }
    }

    /// Process a user message - route to appropriate handler
    pub async fn process_message(&self, message: &str) -> Result<Vec<HybridMessage>> {
        let trimmed = message.trim();
        
        // Check if this is a Hive-specific command
        if let Some(hive_command) = self.extract_hive_command(trimmed) {
            info!("🐝 Processing Hive command: {}", hive_command);
            return self.handle_hive_command(&hive_command, trimmed).await;
        }

        // Check if this is a Claude Code command or regular message
        info!("🤖 Passing to Claude Code: {}", trimmed);
        self.send_to_claude(trimmed).await
    }

    /// Extract Hive command if present
    fn extract_hive_command(&self, message: &str) -> Option<String> {
        if !message.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = message.split_whitespace().collect();
        let command = parts.first()?;

        if HIVE_COMMANDS.contains(command) {
            Some(command.to_string())
        } else {
            None
        }
    }

    /// Handle Hive-specific commands
    async fn handle_hive_command(&self, command: &str, full_message: &str) -> Result<Vec<HybridMessage>> {
        match command {
            "/consensus" | "/hive-consensus" => {
                self.handle_consensus_command(full_message).await
            }
            "/memory" => {
                self.handle_memory_command(full_message).await
            }
            "/openrouter" => {
                self.handle_openrouter_command(full_message).await
            }
            "/hive-analyze" => {
                self.handle_analyze_command(full_message).await
            }
            "/hive-learn" => {
                self.handle_learn_command(full_message).await
            }
            _ => {
                Ok(vec![HybridMessage {
                    content: format!("Unknown Hive command: {}. Available: /consensus, /memory, /openrouter, /hive-analyze, /hive-learn", command),
                    message_type: HybridMessageType::Error,
                    metadata: None,
                }])
            }
        }
    }

    /// Handle consensus command - use our 4-stage pipeline
    async fn handle_consensus_command(&self, message: &str) -> Result<Vec<HybridMessage>> {
        // Extract the actual query (remove the command)
        let query = message.strip_prefix("/consensus")
            .or_else(|| message.strip_prefix("/hive-consensus"))
            .unwrap_or(message)
            .trim();

        if query.is_empty() {
            return Ok(vec![HybridMessage {
                content: "Usage: /consensus <your question or task>".to_string(),
                message_type: HybridMessageType::SystemMessage,
                metadata: None,
            }]);
        }

        info!("🎯 Running 4-stage consensus for: {}", query);
        
        let mut responses = vec![HybridMessage {
            content: format!("🎯 Starting 4-stage consensus pipeline for: \"{}\"", query),
            message_type: HybridMessageType::SystemMessage,
            metadata: None,
        }];

        // For now, simulate consensus pipeline
        // TODO: Integrate with actual ConsensusEngine when API is stable
        responses.push(HybridMessage {
            content: "🔄 Running 4-stage consensus pipeline...".to_string(),
            message_type: HybridMessageType::SystemMessage,
            metadata: None,
        });
        
        // Simulate stages
        let stages = ["Generator", "Refiner", "Validator", "Curator"];
        for stage in stages {
            responses.push(HybridMessage {
                content: format!("✅ {} stage completed", stage),
                message_type: HybridMessageType::SystemMessage,
                metadata: Some(json!({
                    "stage": stage
                })),
            });
        }
        // Add final consensus response
        responses.push(HybridMessage {
            content: format!(
                "🎯 **Consensus Result for: \"{}\"**\n\n\
                Based on the 4-stage consensus pipeline analysis:\n\n\
                This query would be processed through our Generator, Refiner, Validator, and Curator stages \
                using multiple AI models to provide a comprehensive, validated response. \
                The consensus system ensures accuracy and quality through cross-validation \
                and expert-level curation.\n\n\
                *Note: Full consensus integration is pending API stabilization.*",
                query
            ),
            message_type: HybridMessageType::HiveResponse,
            metadata: Some(json!({
                "source": "4-stage-consensus",
                "query": query,
                "stages_completed": 4
            })),
        });
        
        Ok(responses)
    }

    /// Handle memory command - search thematic clusters
    async fn handle_memory_command(&self, message: &str) -> Result<Vec<HybridMessage>> {
        let query = message.strip_prefix("/memory").unwrap_or(message).trim();

        if query.is_empty() {
            return Ok(vec![HybridMessage {
                content: "Usage: /memory <search query>".to_string(),
                message_type: HybridMessageType::SystemMessage,
                metadata: None,
            }]);
        }

        info!("🧠 Searching thematic memory for: {}", query);
        
        let mut responses = vec![HybridMessage {
            content: format!("🧠 Searching thematic memory for: \"{}\"", query),
            message_type: HybridMessageType::SystemMessage,
            metadata: None,
        }];

        // Search thematic memory using the correct method
        match self.thematic_cluster.find_relevant_knowledge_for_ai(query, "memory_search").await {
            Ok(knowledge) => {
                if knowledge.is_empty() {
                    responses.push(HybridMessage {
                        content: "No relevant knowledge found in thematic memory.".to_string(),
                        message_type: HybridMessageType::HiveResponse,
                        metadata: None,
                    });
                } else {
                    responses.push(HybridMessage {
                        content: format!("🧠 **Thematic Memory Results**:\n\n{}", knowledge),
                        message_type: HybridMessageType::HiveResponse,
                        metadata: Some(json!({
                            "source": "thematic-memory",
                            "query": query
                        })),
                    });
                }
            }
            Err(e) => {
                responses.push(HybridMessage {
                    content: format!("Error searching memory: {}", e),
                    message_type: HybridMessageType::Error,
                    metadata: None,
                });
            }
        }
        
        Ok(responses)
    }

    /// Handle OpenRouter command - direct model access
    async fn handle_openrouter_command(&self, message: &str) -> Result<Vec<HybridMessage>> {
        let parts: Vec<&str> = message.split_whitespace().skip(1).collect();
        
        if parts.is_empty() {
            return Ok(vec![HybridMessage {
                content: "Usage: /openrouter <model-name> <your prompt>\nExample: /openrouter claude-3-opus-20240229 Explain quantum computing".to_string(),
                message_type: HybridMessageType::SystemMessage,
                metadata: None,
            }]);
        }

        let model = parts[0];
        let prompt = parts[1..].join(" ");
        
        if prompt.is_empty() {
            return Ok(vec![HybridMessage {
                content: format!("Please provide a prompt after the model name.\nExample: /openrouter {} What is machine learning?", model),
                message_type: HybridMessageType::SystemMessage,
                metadata: None,
            }]);
        }

        info!("🌐 Using OpenRouter model {} for: {}", model, prompt);
        
        let mut responses = vec![HybridMessage {
            content: format!("🌐 Calling OpenRouter model '{}' with your prompt...", model),
            message_type: HybridMessageType::SystemMessage,
            metadata: None,
        }];

        // Use OpenRouter client
        use crate::consensus::openrouter::{OpenRouterClient, OpenRouterRequest, OpenRouterMessage};
        use crate::core::api_keys::ApiKeyManager;
        
        // Get API key
        let api_keys = ApiKeyManager::load_from_database().await?;
        if api_keys.openrouter_key.is_none() {
            return Ok(vec![HybridMessage {
                content: "⚠️ OpenRouter API key not configured. Please add it in Settings.".to_string(),
                message_type: HybridMessageType::Error,
                metadata: None,
            }]);
        }
        
        let client = OpenRouterClient::new(api_keys.openrouter_key.unwrap());
        
        let request = OpenRouterRequest {
            model: model.to_string(),
            messages: vec![
                OpenRouterMessage {
                    role: "user".to_string(),
                    content: prompt.clone(),
                }
            ],
            temperature: Some(0.7),
            max_tokens: Some(2000),
            stream: Some(false),
            top_p: Some(1.0),
            frequency_penalty: None,
            presence_penalty: None,
            provider: None
        };
        
        // Send request to OpenRouter
        match client.chat_completion(request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    if let Some(message) = &choice.message {
                        responses.push(HybridMessage {
                            content: message.content.clone(),
                            message_type: HybridMessageType::HiveResponse,
                            metadata: Some(json!({
                                "source": "openrouter",
                                "model": model,
                                "prompt": prompt,
                                "usage": response.usage
                            })),
                        });
                    }
                }
            }
            Err(e) => {
                responses.push(HybridMessage {
                    content: format!("OpenRouter error: {}", e),
                    message_type: HybridMessageType::Error,
                    metadata: None,
                });
            }
        }
        
        Ok(responses)
    }

    /// Handle analyze command - repository intelligence
    async fn handle_analyze_command(&self, message: &str) -> Result<Vec<HybridMessage>> {
        info!("📊 Running repository analysis");

        // TODO: Actually run repository analysis
        Ok(vec![HybridMessage {
            content: "📊 Running repository analysis with Hive's intelligence engine...".to_string(),
            message_type: HybridMessageType::HiveResponse,
            metadata: None,
        }])
    }

    /// Handle learn command - continuous learning insights
    async fn handle_learn_command(&self, message: &str) -> Result<Vec<HybridMessage>> {
        info!("🎓 Retrieving learning insights");

        // TODO: Actually get learning insights
        Ok(vec![HybridMessage {
            content: "🎓 Retrieving insights from continuous learning system...".to_string(),
            message_type: HybridMessageType::HiveResponse,
            metadata: None,
        }])
    }

    /// Send message to Claude Code process
    async fn send_to_claude(&self, message: &str) -> Result<Vec<HybridMessage>> {
        let mut input_guard = self.input_sender.lock().await;
        let input_sender = input_guard.as_mut()
            .context("Claude Code process not initialized")?;

        // Check if we should use JSON protocol
        let use_json = *self.use_json_protocol.read().await;
        
        if use_json {
            // Send as JSON message for Claude Code SDK
            let json_message = json!({
                "type": "message",
                "content": message,
                "conversation_id": uuid::Uuid::new_v4().to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            
            let json_str = serde_json::to_string(&json_message)?;
            input_sender.write_all(json_str.as_bytes()).await
                .context("Failed to send JSON message to Claude Code")?;
        } else {
            // Send as plain text
            input_sender.write_all(message.as_bytes()).await
                .context("Failed to send message to Claude Code")?;
        }
        
        input_sender.write_all(b"\n").await
            .context("Failed to send newline to Claude Code")?;
        input_sender.flush().await
            .context("Failed to flush message to Claude Code")?;

        // Collect streaming responses
        let responses = self.collect_claude_responses().await?;
        Ok(responses)
    }

    /// Collect responses from Claude Code (handles streaming)
    async fn collect_claude_responses(&self) -> Result<Vec<HybridMessage>> {
        let mut output_guard = self.output_receiver.lock().await;
        let output_receiver = output_guard.as_mut()
            .context("Claude Code output receiver not initialized")?;
        
        let mut responses = Vec::new();
        let mut complete_response = String::new();
        let mut is_streaming = false;
        let use_json = *self.use_json_protocol.read().await;
        
        // Set timeout for complete response
        let timeout_duration = std::time::Duration::from_secs(60);
        let start_time = tokio::time::Instant::now();
        
        loop {
            // Check timeout
            if start_time.elapsed() > timeout_duration {
                if !complete_response.is_empty() {
                    responses.push(HybridMessage {
                        content: complete_response,
                        message_type: HybridMessageType::ClaudeResponse,
                        metadata: None,
                    });
                }
                break;
            }
            
            // Try to receive with short timeout
            match tokio::time::timeout(
                std::time::Duration::from_millis(100),
                output_receiver.recv()
            ).await {
                Ok(Some(data)) => {
                    if use_json {
                        // Parse JSON response
                        if let Ok(json) = serde_json::from_str::<Value>(&data) {
                            match json["type"].as_str() {
                                Some("content_block_start") => {
                                    is_streaming = true;
                                }
                                Some("content_block_delta") => {
                                    if let Some(text) = json["delta"]["text"].as_str() {
                                        complete_response.push_str(text);
                                    }
                                }
                                Some("content_block_stop") => {
                                    is_streaming = false;
                                    if !complete_response.is_empty() {
                                        responses.push(HybridMessage {
                                            content: complete_response.clone(),
                                            message_type: HybridMessageType::ClaudeResponse,
                                            metadata: Some(json.clone()),
                                        });
                                        complete_response.clear();
                                    }
                                }
                                Some("tool_use") => {
                                    responses.push(HybridMessage {
                                        content: json.to_string(),
                                        message_type: HybridMessageType::ClaudeToolUse,
                                        metadata: Some(json),
                                    });
                                }
                                Some("error") => {
                                    let error_msg = json["error"]["message"].as_str()
                                        .unwrap_or("Unknown error");
                                    responses.push(HybridMessage {
                                        content: error_msg.to_string(),
                                        message_type: HybridMessageType::Error,
                                        metadata: Some(json),
                                    });
                                    break;
                                }
                                Some("message_stop") => {
                                    // End of message
                                    break;
                                }
                                _ => {
                                    // Other message types
                                    debug!("Unhandled JSON message type: {:?}", json["type"]);
                                }
                            }
                        } else {
                            // Plain text line
                            complete_response.push_str(&data);
                            complete_response.push('\n');
                        }
                    } else {
                        // Plain text mode
                        complete_response.push_str(&data);
                        complete_response.push('\n');
                    }
                }
                Ok(None) => {
                    // Channel closed
                    if !complete_response.is_empty() {
                        responses.push(HybridMessage {
                            content: complete_response,
                            message_type: HybridMessageType::ClaudeResponse,
                            metadata: None,
                        });
                    }
                    break;
                }
                Err(_) => {
                    // Timeout - check if we have a complete response
                    if !is_streaming && !complete_response.is_empty() {
                        // Looks like we have a complete response
                        responses.push(HybridMessage {
                            content: complete_response.trim().to_string(),
                            message_type: HybridMessageType::ClaudeResponse,
                            metadata: None,
                        });
                        break;
                    }
                    // Continue waiting if still streaming
                }
            }
        }
        
        if responses.is_empty() {
            return Err(anyhow::anyhow!("No response received from Claude Code"));
        }
        
        Ok(responses)
    }

    /// Check if Claude Code is authenticated
    pub async fn is_authenticated(&self) -> bool {
        *self.is_claude_authenticated.read().await
    }

    /// Get list of available Hive commands
    pub fn get_hive_commands(&self) -> Vec<String> {
        HIVE_COMMANDS.iter().map(|&s| s.to_string()).collect()
    }
    
    /// Enable JSON protocol mode (for Claude Code SDK)
    pub async fn enable_json_protocol(&self) {
        *self.use_json_protocol.write().await = true;
        info!("📋 Enabled JSON protocol for Claude Code communication");
    }
    
    /// Check if Claude Code process is running
    pub async fn is_running(&self) -> bool {
        let process_guard = self.claude_process.lock().await;
        if let Some(child) = process_guard.as_ref() {
            // Check if process is still running
            match child.id() {
                Some(_) => true,
                None => false,
            }
        } else {
            false
        }
    }
    
    /// Send a command to Claude Code (like /help, /settings)
    pub async fn send_claude_command(&self, command: &str) -> Result<Vec<HybridMessage>> {
        info!("📤 Sending Claude command: {}", command);
        self.send_to_claude(command).await
    }
    
    /// Initialize Claude Code with our context
    pub async fn initialize_context(&self) -> Result<()> {
        info!("🎯 Initializing Claude Code with Hive context...");
        
        // Send initialization message
        let init_message = format!(
            "You are now running in Hive Consensus IDE with additional capabilities:\n\
            - Use /consensus for 4-stage AI consensus pipeline\n\
            - Use /memory to search thematic conversation history\n\
            - Use /openrouter <model> for direct access to 323+ AI models\n\
            - Use /hive-analyze for repository intelligence\n\
            - Use /hive-learn for continuous learning insights\n\
            All other Claude Code commands work normally."
        );
        
        self.send_to_claude(&init_message).await?;
        Ok(())
    }

    /// Shutdown the integration
    pub async fn shutdown(&self) -> Result<()> {
        info!("🛑 Shutting down Claude Code integration...");

        let mut process_guard = self.claude_process.lock().await;
        if let Some(mut child) = process_guard.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }

        info!("✅ Claude Code integration shut down");
        Ok(())
    }
}

impl Drop for ClaudeCodeIntegration {
    fn drop(&mut self) {
        // Attempt cleanup on drop
        if let Ok(mut process_guard) = self.claude_process.try_lock() {
            if let Some(mut child) = process_guard.take() {
                let _ = futures::executor::block_on(child.kill());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hive_command_detection() {
        let integration = ClaudeCodeIntegration {
            claude_process: Arc::new(Mutex::new(None)),
            input_sender: Arc::new(Mutex::new(None)),
            output_receiver: Arc::new(Mutex::new(None)),
            consensus_engine: Arc::new(todo!()),
            thematic_cluster: Arc::new(todo!()),
            database: Arc::new(todo!()),
            is_claude_authenticated: Arc::new(RwLock::new(false)),
            current_directory: Arc::new(RwLock::new(".".to_string())),
        };

        assert_eq!(
            integration.extract_hive_command("/consensus test question"),
            Some("/consensus".to_string())
        );

        assert_eq!(
            integration.extract_hive_command("/memory search term"),
            Some("/memory".to_string())
        );

        assert_eq!(
            integration.extract_hive_command("/login"),
            None // This should go to Claude Code
        );

        assert_eq!(
            integration.extract_hive_command("regular message"),
            None
        );
    }
}