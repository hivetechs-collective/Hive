//! Hybrid Chat Message Processor
//! 
//! Handles message processing for the hybrid Claude Code + Hive Consensus experience

use crate::consensus::{
    claude_sdk_client::{ClaudeSdkClient, ClaudeQuery, ClaudeOptions, SdkMessage, StatelessContext, ExecutionMode as SdkExecutionMode},
    claude_code_integration::{HybridMessage, HybridMessageType},
};
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::desktop::{
    consensus_integration::DesktopConsensusManager,
    state::{AppState, ChatMessage, MessageMetadata, MessageType},
};
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, debug};

/// Global SDK client instance (initialized on first use)
static SDK_CLIENT: Mutex<Option<Arc<ClaudeSdkClient>>> = Mutex::const_new(None);

/// Initialize the SDK client if not already initialized
async fn ensure_sdk_client() -> anyhow::Result<Arc<ClaudeSdkClient>> {
    let mut client_guard = SDK_CLIENT.lock().await;
    if let Some(client) = client_guard.as_ref() {
        return Ok(client.clone());
    }
    
    // Initialize new client
    let client = Arc::new(ClaudeSdkClient::new().await?);
    
    // Initialize with API key if available
    if let Ok(Some(api_key)) = crate::desktop::simple_db::get_config("anthropic_api_key") {
        if !api_key.is_empty() {
            info!("Initializing Claude SDK with API key");
            client.initialize(&api_key).await?;
        } else {
            error!("Anthropic API key is empty");
        }
    } else {
        error!("No Anthropic API key found in database");
    }
    
    *client_guard = Some(client.clone());
    Ok(client)
}

/// Find the Claude CLI binary
async fn find_claude_cli_binary() -> anyhow::Result<String> {
    // Try common installation paths
    let possible_paths = vec![
        "claude".to_string(),                          // In PATH
        "claude-code".to_string(),                     // Alternative name in PATH  
        "/usr/local/bin/claude".to_string(),           // Standard install
        "/opt/homebrew/bin/claude".to_string(),        // Homebrew on Apple Silicon
        "~/.local/bin/claude".to_string(),             // User local install
    ];
    
    for path in &possible_paths {
        let expanded_path = shellexpand::tilde(path).to_string();
        if let Ok(output) = Command::new(&expanded_path)
            .arg("--version")
            .output()
            .await
        {
            if output.status.success() {
                info!("✅ Found Claude CLI at: {}", expanded_path);
                return Ok(expanded_path);
            }
        }
    }
    
    // Try using 'which' command
    if let Ok(output) = Command::new("which")
        .arg("claude")
        .output()
        .await
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                info!("✅ Found Claude CLI via 'which': {}", path);
                return Ok(path);
            }
        }
    }
    
    Err(anyhow::anyhow!(
        "Claude Code CLI not found. Please install it:\n\
        npm install -g @anthropic-ai/claude-code\n\
        or download from: https://claude.ai/download"
    ))
}

/// Detect if a message is a Claude CLI command
fn is_claude_cli_command(message: &str) -> bool {
    let trimmed = message.trim();
    if !trimmed.starts_with('/') {
        return false;
    }
    
    // Get the command part (before any arguments)
    let command = trimmed.split_whitespace().next().unwrap_or("");
    
    // List of Claude CLI commands that need the actual binary
    const CLAUDE_CLI_COMMANDS: &[&str] = &[
        "/help",
        "/login", 
        "/logout",
        "/api",
        "/model",
        "/models",
        "/settings",
        "/context",
        "/reset",
        "/clear",
        "/exit",
        "/quit",
        "/version",
        "/update",
        "/feedback",
        "/export",
        "/import",
        "/continue",
        "/redo",
        "/undo",
        "/save",
        "/load",
        "/config",
        "/status",
    ];
    
    // Check if it's a Claude CLI command
    CLAUDE_CLI_COMMANDS.contains(&command)
}

/// Process a message from the chat input using hybrid Claude Code integration
pub fn process_message(
    text: String,
    app_state: &mut Signal<AppState>,
    input_text: &mut Signal<String>,
    consensus_manager: &Option<DesktopConsensusManager>,
    show_onboarding: &mut Signal<bool>,
) {
    // Check SDK status first and update UI
    let mut sdk_check_state = app_state.clone();
    spawn(async move {
        match ensure_sdk_client().await {
            Ok(_) => {
                info!("✅ Claude SDK is available and initialized");
                sdk_check_state.write().claude_sdk_connected = true;
            }
            Err(e) => {
                error!("❌ Claude SDK not available: {}", e);
                sdk_check_state.write().claude_sdk_connected = false;
            }
        }
    });
    // Add user message to chat first
    let message = ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        content: text.clone(),
        message_type: MessageType::User,
        timestamp: chrono::Utc::now(),
        metadata: MessageMetadata::default(),
    };
    app_state.write().chat.add_message(message);

    // Clear input immediately for better UX
    input_text.set(String::new());

    // Process message through hybrid integration
    let mut app_state_clone = app_state.clone();
    let mut show_onboarding_clone = show_onboarding.clone();
    let consensus_manager_clone = consensus_manager.clone();
    let state_snapshot = app_state.read().clone();

    spawn(async move {
        match process_hybrid_message(text, &consensus_manager_clone, &state_snapshot).await {
            Ok(responses) => {
                // Add all response messages to chat
                for response in responses {
                    let chat_message = convert_hybrid_to_chat_message(response);
                    app_state_clone.write().chat.add_message(chat_message);
                }
            }
            Err(e) => {
                error!("Hybrid message processing error: {}", e);
                let error_msg = ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    content: format!("Error: {}", e),
                    message_type: MessageType::Error,
                    timestamp: chrono::Utc::now(),
                    metadata: MessageMetadata::default(),
                };
                app_state_clone.write().chat.add_message(error_msg);

                // Check if we should show onboarding
                if e.to_string().contains("No valid OpenRouter API key") 
                    || e.to_string().contains("profile") 
                    || e.to_string().contains("Claude Code binary not found") 
                    || e.to_string().contains("No valid Anthropic API key") {
                    show_onboarding_clone.set(true);
                }
            }
        }
    });
}

/// Convert HybridMessage to ChatMessage
fn convert_hybrid_to_chat_message(response: HybridMessage) -> ChatMessage {
    let (message_type, metadata) = match response.message_type {
        HybridMessageType::ClaudeResponse => {
            (MessageType::Assistant, MessageMetadata::default())
        }
        HybridMessageType::HiveResponse => {
            (MessageType::Assistant, MessageMetadata {
                model: Some("Hive Consensus".to_string()),
                ..MessageMetadata::default()
            })
        }
        HybridMessageType::SystemMessage => {
            (MessageType::System, MessageMetadata::default())
        }
        HybridMessageType::Error => {
            (MessageType::Error, MessageMetadata::default())
        }
        _ => {
            (MessageType::Assistant, MessageMetadata::default())
        }
    };

    ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        content: response.content,
        message_type,
        timestamp: chrono::Utc::now(),
        metadata,
    }
}

/// Process message through hybrid Claude Code integration
async fn process_hybrid_message(
    text: String,
    consensus_manager: &Option<DesktopConsensusManager>,
    app_state: &AppState,
) -> anyhow::Result<Vec<HybridMessage>> {
    info!("🔍 process_hybrid_message called with: {}", text);
    
    // Check if this is a Hive-specific command
    let trimmed = text.trim();
    
    if trimmed.starts_with("/consensus") || trimmed.starts_with("/hive-consensus") {
        return handle_consensus_command(trimmed, consensus_manager).await;
    }
    
    if trimmed.starts_with("/memory") {
        return handle_memory_command(trimmed).await;
    }
    
    if trimmed.starts_with("/openrouter") {
        return handle_openrouter_command(trimmed).await;
    }
    
    if trimmed.starts_with("/hive-analyze") {
        return handle_analyze_command(trimmed).await;
    }
    
    if trimmed.starts_with("/hive-learn") {
        return handle_learn_command(trimmed).await;
    }
    
    // Check if we have Anthropic API key
    let has_anthropic_key = crate::desktop::simple_db::get_config("anthropic_api_key")
        .ok()
        .flatten()
        .filter(|k| !k.is_empty())
        .is_some();
    
    if !has_anthropic_key {
        return Ok(vec![HybridMessage {
            content: "⚠️ No valid Anthropic API key found. Please configure your API key in Settings.".to_string(),
            message_type: HybridMessageType::Error,
            metadata: None,
        }]);
    }

    // Use SDK client for Claude Code functionality
    info!("Initializing Claude SDK client for message processing...");
    match ensure_sdk_client().await {
        Ok(sdk_client) => {
            info!("SDK client ready, processing message");
            // Check if this is a Claude CLI command that needs the binary
            if is_claude_cli_command(trimmed) {
                info!("🔍 Processing Claude CLI command: {}", trimmed);
                process_with_cli_binary(trimmed, consensus_manager).await
            } else {
                // Regular messages and non-CLI commands go to SDK
                info!("🔍 Processing with SDK: {}", trimmed);
                process_with_sdk_client(trimmed, sdk_client, consensus_manager, app_state).await
            }
        }
        Err(e) => {
            error!("Failed to initialize Claude SDK client: {}", e);
            Ok(vec![HybridMessage {
                content: format!(
                    "⚠️ Claude Code SDK integration failed: {}\n\n\
                    Please ensure Claude Code npm package is installed.\n\n\
                    Run: npm install @anthropic-ai/claude-code\n\n\
                    Error: {}", 
                    e, e
                ),
                message_type: HybridMessageType::Error,
                metadata: None,
            }])
        }
    }
}

/// Handle Hive consensus command
async fn handle_consensus_command(
    message: &str,
    consensus_manager: &Option<DesktopConsensusManager>,
) -> anyhow::Result<Vec<HybridMessage>> {
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

    if let Some(mut consensus) = consensus_manager.clone() {
        info!("🎯 Running 4-stage consensus for: {}", query);
        
        // Run the actual consensus pipeline
        match consensus.process_query(query).await {
            Ok(response) => {
                Ok(vec![HybridMessage {
                    content: response,
                    message_type: HybridMessageType::HiveResponse,
                    metadata: Some(serde_json::json!({
                        "source": "4-stage-consensus",
                        "query": query
                    })),
                }])
            }
            Err(e) => {
                Ok(vec![HybridMessage {
                    content: format!("Consensus error: {}", e),
                    message_type: HybridMessageType::Error,
                    metadata: None,
                }])
            }
        }
    } else {
        Ok(vec![HybridMessage {
            content: "⚠️ Consensus engine not available. Please configure your API keys in Settings.".to_string(),
            message_type: HybridMessageType::Error,
            metadata: None,
        }])
    }
}

/// Handle memory search command
async fn handle_memory_command(message: &str) -> anyhow::Result<Vec<HybridMessage>> {
    let query = message.strip_prefix("/memory").unwrap_or(message).trim();

    if query.is_empty() {
        return Ok(vec![HybridMessage {
            content: "Usage: /memory <search query>".to_string(),
            message_type: HybridMessageType::SystemMessage,
            metadata: None,
        }]);
    }

    // TODO: Implement actual thematic memory search
    Ok(vec![HybridMessage {
        content: format!("🧠 Searching thematic memory for: \"{}\"", query),
        message_type: HybridMessageType::HiveResponse,
        metadata: Some(serde_json::json!({
            "source": "thematic-memory",
            "query": query
        })),
    }])
}

/// Handle OpenRouter command  
async fn handle_openrouter_command(message: &str) -> anyhow::Result<Vec<HybridMessage>> {
    let parts: Vec<&str> = message.split_whitespace().skip(1).collect();
    
    if parts.is_empty() {
        return Ok(vec![HybridMessage {
            content: "Usage: /openrouter <model-name> <your prompt>".to_string(),
            message_type: HybridMessageType::SystemMessage,
            metadata: None,
        }]);
    }

    let model = parts[0];
    let prompt = parts[1..].join(" ");

    // TODO: Implement actual OpenRouter API call
    Ok(vec![HybridMessage {
        content: format!("🌐 Using OpenRouter model '{}' for: \"{}\"", model, prompt),
        message_type: HybridMessageType::HiveResponse,
        metadata: Some(serde_json::json!({
            "source": "openrouter",
            "model": model,
            "prompt": prompt
        })),
    }])
}

/// Handle repository analysis command
async fn handle_analyze_command(message: &str) -> anyhow::Result<Vec<HybridMessage>> {
    // TODO: Implement actual repository analysis
    Ok(vec![HybridMessage {
        content: "📊 Running repository analysis with Hive's intelligence engine...".to_string(),
        message_type: HybridMessageType::HiveResponse,
        metadata: Some(serde_json::json!({
            "source": "repository-intelligence"
        })),
    }])
}

/// Handle continuous learning command
async fn handle_learn_command(message: &str) -> anyhow::Result<Vec<HybridMessage>> {
    // TODO: Implement actual learning insights
    Ok(vec![HybridMessage {
        content: "🎓 Retrieving insights from continuous learning system...".to_string(),
        message_type: HybridMessageType::HiveResponse,
        metadata: Some(serde_json::json!({
            "source": "continuous-learning"
        })),
    }])
}

/// Process message using SDK client
async fn process_with_sdk_client(
    message: &str,
    sdk_client: Arc<ClaudeSdkClient>,
    consensus_manager: &Option<DesktopConsensusManager>,
    app_state: &AppState,
) -> anyhow::Result<Vec<HybridMessage>> {
    info!("Processing message through Claude SDK: {}", message);
    
    // Build stateless context if available
    let stateless_context = if let Some(consensus) = consensus_manager {
        build_stateless_context(message, consensus).await.ok()
    } else {
        None
    };
    
    // Get execution mode from database
    let execution_mode_str = crate::desktop::simple_db::get_config("claude_execution_mode")
        .ok()
        .flatten()
        .unwrap_or_else(|| "Direct".to_string());
        
    let execution_mode = match execution_mode_str.as_str() {
        "ConsensusFirst" => SdkExecutionMode::ConsensusFirst,
        "ConsensusAssisted" => SdkExecutionMode::ConsensusAssisted,
        _ => SdkExecutionMode::Direct,
    };
    
    // Get API key from storage
    let api_key = crate::desktop::simple_db::get_config("anthropic_api_key")
        .ok()
        .flatten()
        .filter(|k| !k.is_empty());
    
    if api_key.is_none() {
        error!("No Anthropic API key found for Claude SDK");
    }
    
    // Create query with all our options
    let query = ClaudeQuery {
        prompt: message.to_string(),
        options: ClaudeOptions {
            cwd: Some(std::env::current_dir()?.to_string_lossy().to_string()),
            model: Some("claude-3-5-sonnet-20241022".to_string()),
            plan_mode: execution_mode_str == "Plan",
            auto_edit: app_state.auto_accept,
            execution_mode,
            stateless_context,
            allowed_tools: None,
            disallowed_tools: None,
            api_key,
        },
    };
    
    // Execute query and collect responses
    let mut messages = Vec::new();
    let mut response_rx = sdk_client.query(query).await?;
    let mut full_response = String::new();
    let mut session_id = String::new();
    
    while let Some(sdk_msg) = response_rx.recv().await {
        match sdk_msg {
            SdkMessage::Assistant { message, session_id: sid, .. } => {
                session_id = sid;
                if let Some(content) = message.get("content") {
                    // Handle both string content and array content (for multi-part messages)
                    if let Some(text) = content.as_str() {
                        full_response.push_str(text);
                        full_response.push('\n');
                    } else if let Some(content_array) = content.as_array() {
                        for item in content_array {
                            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                full_response.push_str(text);
                                full_response.push('\n');
                            }
                        }
                    }
                }
            }
            SdkMessage::System { subtype, data, .. } => {
                if subtype == "init" {
                    info!("Claude SDK initialized successfully");
                    if let Some(tools) = data.get("tools").and_then(|t| t.as_array()) {
                        info!("Available tools: {} tools", tools.len());
                    }
                } else {
                    debug!("System message: {} - {:?}", subtype, data);
                }
            }
            SdkMessage::Result { subtype, duration_ms, is_error, data, .. } => {
                if is_error {
                    let error_msg = data.get("error")
                        .and_then(|e| e.as_str())
                        .or_else(|| data.get("message").and_then(|m| m.as_str()))
                        .unwrap_or("Unknown error");
                    
                    return Ok(vec![HybridMessage {
                        content: format!("Claude Code error: {}", error_msg),
                        message_type: HybridMessageType::Error,
                        metadata: Some(data),
                    }]);
                }
                
                // TODO: Store the conversation in database when we have proper curator storage
                
                info!("Claude Code completed successfully in {}ms", duration_ms);
            }
            _ => {
                debug!("Unhandled SDK message type: {:?}", sdk_msg);
            }
        }
    }
    
    if !full_response.is_empty() {
        messages.push(HybridMessage {
            content: full_response.trim().to_string(),
            message_type: HybridMessageType::ClaudeResponse,
            metadata: Some(serde_json::json!({
                "source": "claude-sdk",
                "session_id": session_id,
            })),
        });
    } else {
        messages.push(HybridMessage {
            content: "No response received from Claude Code.".to_string(),
            message_type: HybridMessageType::Error,
            metadata: None,
        });
    }
    
    Ok(messages)
}

/// Process slash commands using the Claude CLI binary directly
async fn process_with_cli_binary(
    command: &str,
    _consensus_manager: &Option<DesktopConsensusManager>,
) -> anyhow::Result<Vec<HybridMessage>> {
    info!("Processing Claude CLI command through binary: {}", command);
    
    // Find the Claude CLI binary
    let claude_binary = match find_claude_cli_binary().await {
        Ok(path) => path,
        Err(e) => {
            return Ok(vec![HybridMessage {
                content: format!(
                    "⚠️ Claude Code CLI not available: {}\n\n\
                    Please ensure Claude Code is installed:\n\
                    1. Run: npm install -g @anthropic-ai/claude-code\n\
                    2. Or download from: https://claude.ai/download",
                    e
                ),
                message_type: HybridMessageType::Error,
                metadata: None,
            }]);
        }
    };
    
    // Execute the Claude CLI command
    info!("Executing Claude CLI: {} {}", claude_binary, command);
    
    // For interactive commands like /login, we need to spawn the process differently
    if command == "/login" || command == "/logout" {
        // These commands may open a browser, so we run them in a fire-and-forget manner
        match Command::new(&claude_binary)
            .args(command.split_whitespace())
            .spawn()
        {
            Ok(_) => {
                return Ok(vec![HybridMessage {
                    content: format!("Executing Claude command: {}\n\nPlease check your browser if authentication is required.", command),
                    message_type: HybridMessageType::SystemMessage,
                    metadata: None,
                }]);
            }
            Err(e) => {
                return Ok(vec![HybridMessage {
                    content: format!("Failed to execute Claude command: {}", e),
                    message_type: HybridMessageType::Error,
                    metadata: None,
                }]);
            }
        }
    }
    
    // For other commands, capture the output
    let mut child = Command::new(&claude_binary)
        .args(command.split_whitespace())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    let stdout = child.stdout.take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
    let stderr = child.stderr.take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;
    
    // Read output
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    let mut output = String::new();
    let mut error_output = String::new();
    
    // Read stdout
    while let Some(line) = stdout_reader.next_line().await? {
        output.push_str(&line);
        output.push('\n');
    }
    
    // Read stderr
    while let Some(line) = stderr_reader.next_line().await? {
        error_output.push_str(&line);
        error_output.push('\n');
    }
    
    // Wait for the process to complete
    let status = child.wait().await?;
    
    if status.success() {
        if !output.is_empty() {
            Ok(vec![HybridMessage {
                content: output.trim().to_string(),
                message_type: HybridMessageType::ClaudeResponse,
                metadata: Some(serde_json::json!({
                    "source": "claude-cli",
                    "command": command
                })),
            }])
        } else {
            Ok(vec![HybridMessage {
                content: format!("Command '{}' executed successfully.", command),
                message_type: HybridMessageType::SystemMessage,
                metadata: None,
            }])
        }
    } else {
        let error_msg = if !error_output.is_empty() {
            error_output.trim().to_string()
        } else {
            format!("Command '{}' failed with exit code: {:?}", command, status.code())
        };
        
        Ok(vec![HybridMessage {
            content: error_msg,
            message_type: HybridMessageType::Error,
            metadata: None,
        }])
    }
}

/// Build stateless context from our memory systems
async fn build_stateless_context(
    query: &str,
    consensus_manager: &DesktopConsensusManager,
) -> anyhow::Result<StatelessContext> {
    // TODO: Implement proper memory retrieval when thematic cluster is available
    // For now, return empty context
    Ok(StatelessContext {
        recent_knowledge: None,
        thematic_knowledge: None,
        learned_patterns: None,
        repository_context: None,
    })
}

