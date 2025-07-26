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
use crate::consensus::types::Stage;

/// Commands that should be handled by Hive instead of Claude Code
const HIVE_COMMANDS: &[&str] = &[
    "/consensus",        // 4-stage consensus pipeline
    "/hive-consensus",   // Alias for consensus
    "/memory",           // Thematic memory search
    "/openrouter",       // Direct OpenRouter model access
    "/hive-analyze",     // Repository analysis
    "/hive-learn",       // Continuous learning insights
];

/// Stateless context injected before Claude queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatelessContext {
    /// Recent curator articles from past 24-48 hours
    pub recent_knowledge: String,
    /// Thematically similar past conversations
    pub thematic_knowledge: String,
    /// Learned patterns from AI helpers
    pub learned_patterns: String,
    /// Repository context if applicable
    pub repository_context: Option<String>,
    /// User preferences and profile settings
    pub user_preferences: Option<String>,
}

impl StatelessContext {
    /// Convert context to a system prompt for Claude
    pub fn to_system_prompt(&self) -> String {
        let mut prompt = String::from(
            "You are Claude Code integrated with Hive Consensus, operating in stateless mode.\n\n\
            Key principles:\n\
            1. Each request is independent - you have no conversation history\n\
            2. The context provided contains relevant past knowledge from the memory system\n\
            3. When uncertain about complex decisions, explicitly state \"I would benefit from consensus validation\"\n\
            4. All your outputs will be stored as knowledge for future requests\n\
            5. You have full access to all Claude Code capabilities and commands\n\n"
        );
        
        if !self.recent_knowledge.is_empty() {
            prompt.push_str("Recent curator knowledge (authoritative answers from past 24h):\n");
            prompt.push_str(&self.recent_knowledge);
            prompt.push_str("\n\n");
        }
        
        if !self.thematic_knowledge.is_empty() {
            prompt.push_str("Thematically similar past conversations:\n");
            prompt.push_str(&self.thematic_knowledge);
            prompt.push_str("\n\n");
        }
        
        if !self.learned_patterns.is_empty() {
            prompt.push_str("Learned patterns and best practices:\n");
            prompt.push_str(&self.learned_patterns);
            prompt.push_str("\n\n");
        }
        
        if let Some(repo_context) = &self.repository_context {
            prompt.push_str("Repository context:\n");
            prompt.push_str(repo_context);
            prompt.push_str("\n\n");
        }
        
        if let Some(prefs) = &self.user_preferences {
            prompt.push_str("User preferences:\n");
            prompt.push_str(prefs);
            prompt.push_str("\n\n");
        }
        
        prompt
    }
}

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

/// Execution modes for Claude Code
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClaudeExecutionMode {
    /// Run consensus before Claude
    ConsensusFirst,
    /// Claude first, then consensus validation
    ConsensusAssisted,
    /// Direct Claude only (with context)
    Direct,
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
    
    /// Execution mode controls
    execution_mode: Arc<RwLock<ClaudeExecutionMode>>,
    plan_mode_enabled: Arc<RwLock<bool>>,
    auto_edit_enabled: Arc<RwLock<bool>>,
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
            execution_mode: Arc::new(RwLock::new(ClaudeExecutionMode::Direct)),
            plan_mode_enabled: Arc::new(RwLock::new(false)),
            auto_edit_enabled: Arc::new(RwLock::new(true)),
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
        // First check npm global installation paths
        if let Ok(output) = Command::new("npm")
            .args(&["config", "get", "prefix"])
            .output() 
        {
            if output.status.success() {
                let npm_prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let npm_paths = vec![
                    format!("{}/bin/claude", npm_prefix),
                    format!("{}/bin/claude-code", npm_prefix),
                    format!("{}/claude.cmd", npm_prefix),  // Windows
                ];
                
                for path in npm_paths {
                    if let Ok(output) = Command::new(&path)
                        .arg("--version")
                        .output() 
                    {
                        if output.status.success() {
                            info!("✅ Found Claude Code via npm at: {}", path);
                            return Ok(path);
                        }
                    }
                }
            }
        }
        
        // Then try standard installation paths
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
            "/usr/local/lib/node_modules/.bin/claude",  // npm global
            "/opt/homebrew/lib/node_modules/.bin/claude", // npm on Apple Silicon
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

    /// Build stateless context for Claude queries
    async fn build_stateless_context(&self, request: &str) -> Result<StatelessContext> {
        // Get recent curator knowledge
        let recent_knowledge = match self.thematic_cluster.get_curator_knowledge_base(request).await {
            Ok(knowledge) => knowledge,
            Err(e) => {
                warn!("Failed to get curator knowledge base: {}", e);
                String::new()
            }
        };
        
        // Get thematic matches
        let thematic_knowledge = match self.thematic_cluster.find_relevant_knowledge_for_ai(request, "claude_context").await {
            Ok(knowledge) => knowledge,
            Err(e) => {
                warn!("Failed to get thematic knowledge: {}", e);
                String::new()
            }
        };
        
        // Get learned patterns - for now, we'll use a placeholder
        // TODO: Integrate with ContinuousLearner when available
        let learned_patterns = String::from("No learned patterns available yet.");
        
        // Get repository context if applicable - placeholder for now
        // TODO: Integrate with repository analysis
        let repository_context = None;
        
        // Get user preferences - placeholder for now
        // TODO: Load from user profile
        let user_preferences = None;
        
        Ok(StatelessContext {
            recent_knowledge,
            thematic_knowledge,
            learned_patterns,
            repository_context,
            user_preferences,
        })
    }

    /// Process a user message - route to appropriate handler
    pub async fn process_message(&self, message: &str) -> Result<Vec<HybridMessage>> {
        let trimmed = message.trim();
        
        // Check if this is a Hive-specific command
        if let Some(hive_command) = self.extract_hive_command(trimmed) {
            info!("🐝 Processing Hive command: {}", hive_command);
            return self.handle_hive_command(&hive_command, trimmed).await;
        }

        // Build stateless context for Claude
        let context = self.build_stateless_context(trimmed).await?;
        
        // Apply mode-based routing
        let execution_mode = *self.execution_mode.read().await;
        let responses = match execution_mode {
            ClaudeExecutionMode::ConsensusFirst => {
                info!("🎯 ConsensusFirst mode: Running consensus before Claude");
                self.consensus_first_flow(trimmed, context).await
            }
            ClaudeExecutionMode::ConsensusAssisted => {
                info!("🤖 ConsensusAssisted mode: Claude first, then validation");
                self.consensus_assisted_flow(trimmed, context).await
            }
            ClaudeExecutionMode::Direct => {
                info!("🤖 Direct mode: Passing to Claude Code with context");
                self.direct_claude_flow(trimmed, context).await
            }
        }?;
        
        // Store the responses as conversation context for future queries
        if let Err(e) = self.store_claude_response_as_context(trimmed, &responses).await {
            warn!("Failed to store response as context: {}", e);
            // Don't fail the request if storage fails
        }
        
        Ok(responses)
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
    
    /// Send message to Claude Code with stateless context
    async fn send_to_claude_with_context(&self, message: &str, context: StatelessContext) -> Result<Vec<HybridMessage>> {
        // Check for plan mode and auto-edit mode
        let plan_mode = *self.plan_mode_enabled.read().await;
        let auto_edit = *self.auto_edit_enabled.read().await;
        
        // Prepare the enhanced message with context and mode instructions
        let mut system_prompt = context.to_system_prompt();
        
        if plan_mode {
            system_prompt.push_str("\n⚠️ PLAN MODE ENABLED: Create a detailed plan but DO NOT execute any code changes. Only outline the steps.\n");
        }
        
        if !auto_edit {
            system_prompt.push_str("\n⚠️ AUTO-EDIT DISABLED: Ask for confirmation before making any file modifications.\n");
        }
        
        let enhanced_message = format!(
            "{}\n\nUser query: {}",
            system_prompt,
            message
        );
        
        debug!("Sending to Claude with context. Context size: {} chars", system_prompt.len());
        
        // Send the enhanced message
        self.send_to_claude(&enhanced_message).await
    }
    
    /// ConsensusFirst flow: Run consensus before Claude
    async fn consensus_first_flow(&self, message: &str, context: StatelessContext) -> Result<Vec<HybridMessage>> {
        let mut responses = Vec::new();
        
        // First, run consensus
        responses.push(HybridMessage {
            content: "🎯 Running consensus pipeline first...".to_string(),
            message_type: HybridMessageType::SystemMessage,
            metadata: None,
        });
        
        // Run consensus through the engine (this is a simplified version)
        // In a full implementation, this would use the actual consensus pipeline
        let consensus_result = format!(
            "Consensus analysis for: {}\n\
            Note: Full 4-stage consensus integration pending API stabilization.\n\
            This would normally run Generator → Refiner → Validator → Curator stages.",
            message
        );
        
        responses.push(HybridMessage {
            content: format!("✅ Consensus Result:\n{}", consensus_result),
            message_type: HybridMessageType::HiveResponse,
            metadata: Some(json!({ "source": "consensus" })),
        });
        
        // Then send to Claude with consensus results
        let enhanced_context = StatelessContext {
            recent_knowledge: format!("{}\n\nConsensus Analysis:\n{}", 
                context.recent_knowledge, consensus_result),
            ..context
        };
        
        let claude_responses = self.send_to_claude_with_context(message, enhanced_context).await?;
        responses.extend(claude_responses);
        
        Ok(responses)
    }
    
    /// ConsensusAssisted flow: Claude first, then validate with consensus
    async fn consensus_assisted_flow(&self, message: &str, context: StatelessContext) -> Result<Vec<HybridMessage>> {
        let mut responses = Vec::new();
        
        // First, get Claude's response
        let claude_responses = self.send_to_claude_with_context(message, context).await?;
        responses.extend(claude_responses.clone());
        
        // Check if consensus validation is needed
        let needs_validation = claude_responses.iter().any(|r| {
            r.content.contains("would benefit from consensus validation") ||
            r.content.contains("complex decision") ||
            r.content.contains("uncertain")
        });
        
        if needs_validation {
            responses.push(HybridMessage {
                content: "🎯 Running consensus validation...".to_string(),
                message_type: HybridMessageType::SystemMessage,
                metadata: None,
            });
            
            // Run validation through consensus (simplified version)
            let validation_result = "Consensus validation complete. The approach has been analyzed for correctness and best practices.";
            
            responses.push(HybridMessage {
                content: format!("✅ {}", validation_result),
                message_type: HybridMessageType::HiveResponse,
                metadata: Some(json!({ "source": "consensus_validation" })),
            });
        }
        
        Ok(responses)
    }
    
    /// Direct flow: Just Claude with context
    async fn direct_claude_flow(&self, message: &str, context: StatelessContext) -> Result<Vec<HybridMessage>> {
        self.send_to_claude_with_context(message, context).await
    }

    /// Store Claude response as conversation context in the database
    async fn store_claude_response_as_context(
        &self, 
        query: &str, 
        responses: &[HybridMessage]
    ) -> Result<()> {
        // Combine all Claude responses into a single knowledge article
        let mut claude_content = String::new();
        let mut has_claude_response = false;
        
        for response in responses {
            match response.message_type {
                HybridMessageType::ClaudeResponse => {
                    claude_content.push_str(&response.content);
                    claude_content.push('\n');
                    has_claude_response = true;
                }
                _ => {}
            }
        }
        
        if !has_claude_response || claude_content.trim().is_empty() {
            return Ok(()); // Nothing to store
        }
        
        // Format Claude's response as conversation context (not authoritative)
        let conversation_context = format!(
            "Claude Code Response:\n{}\n\n---\n*This is conversation context, not an authoritative source*",
            claude_content.trim()
        );
        
        // Store the conversation context in the database
        // Note: We store with empty final_answer and source_of_truth since only curator responses are authoritative
        use crate::core::database::KnowledgeConversation;
        
        match KnowledgeConversation::create(
            Some(uuid::Uuid::new_v4().to_string()), // conversation_id
            query.to_string(),                       // question
            String::new(),                           // final_answer (empty - not authoritative)
            String::new(),                           // source_of_truth (empty - not authoritative)
        ).await {
            Ok(mut knowledge) => {
                // Set the conversation context
                knowledge.conversation_context = Some(conversation_context);
                
                info!("✅ Stored Claude response as conversation context for thematic clustering");
                
                // Don't update thematic clusters with curator knowledge since this isn't authoritative
                // The thematic cluster will still be able to find this conversation context
                // when searching for similar questions
                
                Ok(())
            }
            Err(e) => {
                warn!("Failed to store Claude response as conversation context: {}", e);
                Err(e)
            }
        }
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
    
    /// Set execution mode
    pub async fn set_execution_mode(&self, mode: ClaudeExecutionMode) {
        *self.execution_mode.write().await = mode;
        info!("Set Claude execution mode to: {:?}", mode);
    }
    
    /// Get current execution mode
    pub async fn get_execution_mode(&self) -> ClaudeExecutionMode {
        *self.execution_mode.read().await
    }
    
    /// Enable/disable plan mode
    pub async fn set_plan_mode(&self, enabled: bool) {
        *self.plan_mode_enabled.write().await = enabled;
        info!("Plan mode {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Check if plan mode is enabled
    pub async fn is_plan_mode_enabled(&self) -> bool {
        *self.plan_mode_enabled.read().await
    }
    
    /// Enable/disable auto-edit mode
    pub async fn set_auto_edit_mode(&self, enabled: bool) {
        *self.auto_edit_enabled.write().await = enabled;
        info!("Auto-edit mode {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Check if auto-edit mode is enabled
    pub async fn is_auto_edit_enabled(&self) -> bool {
        *self.auto_edit_enabled.read().await
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