//! Hybrid Chat Message Processor
//! 
//! Handles message processing for the hybrid Claude Code + Hive Consensus experience

use crate::consensus::claude_code_integration::{HybridMessage, HybridMessageType};
use crate::desktop::{
    claude_integration_manager::get_claude_integration,
    consensus_integration::DesktopConsensusManager,
    state::{AppState, ChatMessage, MessageMetadata, MessageType},
};
use dioxus::prelude::*;
use tracing::{error, info};

/// Process a message from the chat input using hybrid Claude Code integration
pub fn process_message(
    text: String,
    app_state: &mut Signal<AppState>,
    input_text: &mut Signal<String>,
    consensus_manager: &Option<DesktopConsensusManager>,
    show_onboarding: &mut Signal<bool>,
) {
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

    spawn(async move {
        match process_hybrid_message(text, &consensus_manager_clone).await {
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
                    || e.to_string().contains("Claude Code binary not found") {
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
) -> anyhow::Result<Vec<HybridMessage>> {
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
    
    // Pass to Claude Code integration
    if let Some(claude_integration) = get_claude_integration().await {
        // Send to actual Claude Code process
        claude_integration.process_message(trimmed).await
    } else {
        // Claude Code not available
        Ok(vec![HybridMessage {
            content: format!(
                "⚠️ Claude Code integration is not available. \n\n\
                To use native Claude Code commands like /login, /help, etc., ensure:\n\
                1. Claude Code is installed on your system\n\
                2. The binary is accessible in your PATH\n\n\
                Your message: \"{}\"", 
                trimmed
            ),
            message_type: HybridMessageType::SystemMessage,
            metadata: None,
        }])
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