//! Simple Chat Processor - Beautiful GUI Preserved!
//!
//! This module processes chat messages using the simple Claude integration
//! while preserving all existing GUI functionality and adding intelligence.

use crate::consensus::SimpleClaudeIntegration;
use crate::desktop::{
    consensus_integration::DesktopConsensusManager,
    state::{AppState, ChatMessage, MessageMetadata, MessageType},
};
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// Global Claude integration instance
static CLAUDE_INTEGRATION: Mutex<Option<Arc<SimpleClaudeIntegration>>> = Mutex::const_new(None);

/// Initialize Claude integration if not already done
async fn ensure_claude_integration() -> anyhow::Result<Arc<SimpleClaudeIntegration>> {
    let mut integration_guard = CLAUDE_INTEGRATION.lock().await;
    if let Some(integration) = integration_guard.as_ref() {
        return Ok(integration.clone());
    }
    
    // Create new integration - this will fail if Claude Code is not installed
    match SimpleClaudeIntegration::new().await {
        Ok(integration) => {
            let integration = Arc::new(integration);
            *integration_guard = Some(integration.clone());
            info!("✅ Claude Code integration initialized successfully");
            Ok(integration)
        }
        Err(e) => {
            error!("❌ Failed to initialize Claude Code integration: {}", e);
            Err(anyhow::anyhow!(
                "Claude Code Integration Failed\n\n{}\n\nHive AI requires Claude Code to function properly.", e
            ))
        }
    }
}

/// Process a message from the chat input - MUCH SIMPLER!
pub fn process_message(
    text: String,
    app_state: &mut Signal<AppState>,
    input_text: &mut Signal<String>,
    consensus_manager: &Option<DesktopConsensusManager>,
    show_onboarding: &mut Signal<bool>,
) {
    info!("Processing message: {}", text);
    
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

    // Process message based on execution mode from GUI
    let mut app_state_clone = app_state.clone();
    let mut show_onboarding_clone = show_onboarding.clone();
    let consensus_manager_clone = consensus_manager.clone();
    let execution_mode = app_state.read().claude_execution_mode.clone();

    spawn(async move {
        match process_with_execution_mode(text, execution_mode, &consensus_manager_clone).await {
            Ok(response_content) => {
                let response_message = ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    content: response_content,
                    message_type: MessageType::Assistant,
                    timestamp: chrono::Utc::now(),
                    metadata: MessageMetadata {
                        model: Some("Claude Code".to_string()),
                        ..MessageMetadata::default()
                    },
                };
                app_state_clone.write().chat.add_message(response_message);
            }
            Err(e) => {
                error!("Message processing error: {}", e);
                let error_msg = ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    content: format!("Error: {}", e),
                    message_type: MessageType::Error,
                    timestamp: chrono::Utc::now(),
                    metadata: MessageMetadata::default(),
                };
                app_state_clone.write().chat.add_message(error_msg);
                
                // Show onboarding if Claude not available
                if e.to_string().contains("Claude Code not found") {
                    show_onboarding_clone.set(true);
                }
            }
        }
    });
}

/// Process message based on execution mode selected in GUI
async fn process_with_execution_mode(
    text: String,
    execution_mode: String,
    consensus_manager: &Option<DesktopConsensusManager>,
) -> anyhow::Result<String> {
    let claude_integration = ensure_claude_integration().await?;
    
    match execution_mode.as_str() {
        "Direct" => {
            // Direct Claude with memory context
            let memory_context = get_memory_context(&text).await.ok();
            claude_integration.send_message(&text, memory_context.as_deref()).await
        }
        "ConsensusFirst" => {
            // Run consensus first, then Claude
            if let Some(mut consensus) = consensus_manager.clone() {
                match consensus.process_query(&text).await {
                    Ok(consensus_result) => {
                        let enhanced_text = format!(
                            "Consensus analysis result:\n{}\n\nOriginal query: {}",
                            consensus_result, text
                        );
                        let memory_context = get_memory_context(&text).await.ok();
                        claude_integration.send_message(&enhanced_text, memory_context.as_deref()).await
                    }
                    Err(e) => {
                        error!("Consensus failed: {}", e);
                        // Fallback to direct Claude
                        let memory_context = get_memory_context(&text).await.ok();
                        claude_integration.send_message(&text, memory_context.as_deref()).await
                    }
                }
            } else {
                // No consensus available, use direct
                let memory_context = get_memory_context(&text).await.ok();
                claude_integration.send_message(&text, memory_context.as_deref()).await
            }
        }
        "ConsensusAssisted" | _ => {
            // Claude first, then consensus validation if needed
            let memory_context = get_memory_context(&text).await.ok();
            let claude_response = claude_integration.send_message(&text, memory_context.as_deref()).await?;
            
            // Check if response suggests consensus validation
            if should_validate_with_consensus(&claude_response) {
                if let Some(mut consensus) = consensus_manager.clone() {
                    match consensus.process_query(&text).await {
                        Ok(consensus_result) => {
                            Ok(format!(
                                "Claude Response:\n{}\n\n---\n\nConsensus Validation:\n{}",
                                claude_response, consensus_result
                            ))
                        }
                        Err(_) => {
                            // Consensus failed, just return Claude response
                            Ok(claude_response)
                        }
                    }
                } else {
                    Ok(claude_response)
                }
            } else {
                Ok(claude_response)
            }
        }
    }
}

/// Get memory context for the query (simplified for now)
async fn get_memory_context(query: &str) -> anyhow::Result<String> {
    // TODO: Implement actual thematic memory search
    // For now, return empty context
    Ok(String::new())
}

/// Check if Claude response suggests consensus validation
fn should_validate_with_consensus(response: &str) -> bool {
    response.contains("consensus validation") ||
    response.contains("would benefit from consensus") ||
    response.contains("complex decision") ||
    response.contains("uncertain")
}

/// Get Claude integration status for GUI
pub async fn get_claude_status() -> String {
    match ensure_claude_integration().await {
        Ok(integration) => integration.get_status(),
        Err(_) => "⚠️ Claude Code Not Available".to_string(),
    }
}

/// Process message directly with Claude (for use from hive-consensus.rs)
pub async fn process_message_direct_claude(
    query: String,
    app_state: &mut Signal<AppState>,
    current_response: &mut Signal<String>,
    is_processing: &mut Signal<bool>,
) {
    info!("🎯 process_message_direct_claude called with query: {}", query);
    
    // Ensure Claude is available
    let claude_integration = match ensure_claude_integration().await {
        Ok(integration) => integration,
        Err(e) => {
            error!("Failed to get Claude integration: {}", e);
            let error_msg = ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                content: format!("Error: {}", e),
                message_type: MessageType::Error,
                timestamp: chrono::Utc::now(),
                metadata: MessageMetadata::default(),
            };
            app_state.write().chat.add_message(error_msg);
            current_response.set(format!("Error: {}", e));
            is_processing.set(false);
            return;
        }
    };
    
    // Get memory context if available
    let memory_context = get_memory_context(&query).await.ok();
    
    // Send message to Claude
    match claude_integration.send_message(&query, memory_context.as_deref()).await {
        Ok(response) => {
            info!("✅ Received response from Claude");
            current_response.set(response.clone());
            let assistant_msg = ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                content: response,
                message_type: MessageType::Assistant,
                timestamp: chrono::Utc::now(),
                metadata: MessageMetadata {
                    model: Some("Claude Code".to_string()),
                    ..MessageMetadata::default()
                },
            };
            app_state.write().chat.add_message(assistant_msg);
        }
        Err(e) => {
            error!("Failed to get response from Claude: {}", e);
            let error_msg = format!("Error: {}", e);
            current_response.set(error_msg.clone());
            let error_chat_msg = ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                content: error_msg,
                message_type: MessageType::Error,
                timestamp: chrono::Utc::now(),
                metadata: MessageMetadata::default(),
            };
            app_state.write().chat.add_message(error_chat_msg);
        }
    }
    
    is_processing.set(false);
}