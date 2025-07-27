//! Response Coordinator - Share content between Claude and Consensus
//!
//! Provides buttons and functionality to send responses between the two AI systems

use dioxus::prelude::*;
use crate::desktop::state::{AppState, ChatMessage, MessageType, MessageMetadata};
use crate::desktop::consensus_integration::DesktopConsensusManager;

/// Button to send terminal content to consensus
#[component]
pub fn SendToConsensusButton(
    app_state: Signal<AppState>,
) -> Element {
    let is_sending = use_signal(|| false);
    let consensus_manager = use_context::<Option<DesktopConsensusManager>>();
    let has_consensus = consensus_manager.is_some();

    rsx! {
        button {
            class: "send-button",
            style: "
                background: rgba(255, 193, 7, 0.1);
                border: 1px solid rgba(255, 193, 7, 0.3);
                color: #FFC107;
                padding: 6px 12px;
                border-radius: 4px;
                cursor: pointer;
                font-size: 12px;
                transition: all 0.2s ease;
                display: flex;
                align-items: center;
                gap: 6px;
                margin: 4px;
            ",
            disabled: *is_sending.read() || !has_consensus,
            onclick: move |_| {
                let mut app_state = app_state.clone();
                let consensus = consensus_manager.clone();
                let mut is_sending = is_sending.clone();
                
                spawn(async move {
                    is_sending.set(true);
                    
                    // Extract terminal content
                    if let Some(terminal_content) = extract_terminal_response(&app_state).await {
                        // Send to consensus
                        if let Some(consensus) = consensus {
                            process_with_consensus(terminal_content, &mut app_state, &consensus).await;
                        }
                    }
                    
                    is_sending.set(false);
                });
            },
            
            if *is_sending.read() {
                "Sending..."
            } else {
                "→ Send to Consensus"
            }
            
            // Arrow icon
            span {
                style: "font-size: 16px;",
                "🔄"
            }
        }
    }
}

/// Button to send consensus response to terminal
#[component]
pub fn SendToClaudeButton(
    app_state: Signal<AppState>,
) -> Element {
    let is_sending = use_signal(|| false);

    rsx! {
        button {
            class: "send-button",
            style: "
                background: rgba(139, 92, 246, 0.1);
                border: 1px solid rgba(139, 92, 246, 0.3);
                color: #8B5CF6;
                padding: 6px 12px;
                border-radius: 4px;
                cursor: pointer;
                font-size: 12px;
                transition: all 0.2s ease;
                display: flex;
                align-items: center;
                gap: 6px;
                margin: 4px;
            ",
            disabled: *is_sending.read(),
            onclick: move |_| {
                let app_state = app_state.clone();
                let mut is_sending = is_sending.clone();
                
                spawn(async move {
                    is_sending.set(true);
                    
                    // Extract last consensus response
                    if let Some(consensus_content) = extract_consensus_response(&app_state).await {
                        // Send to terminal as input
                        send_to_terminal(&app_state, &consensus_content).await;
                    }
                    
                    is_sending.set(false);
                });
            },
            
            if *is_sending.read() {
                "Sending..."
            } else {
                "← Send to Claude"
            }
            
            // Arrow icon
            span {
                style: "font-size: 16px;",
                "🤖"
            }
        }
    }
}

/// Copy button for easy content sharing
#[component]
pub fn CopyResponseButton(content: String) -> Element {
    let is_copied = use_signal(|| false);

    rsx! {
        button {
            class: "copy-button",
            style: "
                background: transparent;
                border: 1px solid #3e3e42;
                color: #858585;
                padding: 4px 8px;
                border-radius: 4px;
                cursor: pointer;
                font-size: 11px;
                transition: all 0.2s ease;
            ",
            onclick: move |_| {
                // Copy to clipboard using JavaScript
                let content_to_copy = content.clone();
                let mut is_copied = is_copied.clone();
                
                spawn(async move {
                    if copy_to_clipboard(&content_to_copy).await {
                        is_copied.set(true);
                        // Reset after 2 seconds
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        is_copied.set(false);
                    }
                });
            },
            
            if *is_copied.read() {
                "✓ Copied"
            } else {
                "📋 Copy"
            }
        }
    }
}

/// Extract the last Claude response from terminal
async fn extract_terminal_response(_app_state: &Signal<AppState>) -> Option<String> {
    use crate::desktop::terminal_registry::{get_active_terminal_content, extract_claude_response};
    
    // Get the active terminal content
    if let Some(content) = get_active_terminal_content() {
        // Extract Claude's response from the content
        if let Some(response) = extract_claude_response(&content) {
            tracing::info!("📤 Extracted Claude response: {} chars", response.len());
            return Some(response);
        } else {
            tracing::warn!("Could not identify Claude response in terminal content");
        }
    } else {
        tracing::warn!("No active terminal found");
    }
    
    None
}

/// Extract the last consensus response
async fn extract_consensus_response(app_state: &Signal<AppState>) -> Option<String> {
    let state = app_state.read();
    
    // Find the last assistant message in chat
    state.chat.messages
        .iter()
        .rev()
        .find(|msg| matches!(msg.message_type, MessageType::Assistant))
        .map(|msg| msg.content.clone())
}

/// Send content to terminal as input
async fn send_to_terminal(_app_state: &Signal<AppState>, _content: &str) -> bool {
    // TODO: Implement sending to terminal
    // This will require access to the terminal's PTY writer
    
    tracing::warn!("Sending to terminal not yet implemented");
    false
}

/// Process content with consensus
async fn process_with_consensus(
    content: String,
    app_state: &mut Signal<AppState>,
    consensus_manager: &DesktopConsensusManager,
) {
    // Add a system message showing what we're doing
    let system_msg = ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        content: format!("📥 Enhancing Claude's response with consensus pipeline..."),
        message_type: MessageType::System,
        timestamp: chrono::Utc::now(),
        metadata: MessageMetadata::default(),
    };
    app_state.write().chat.add_message(system_msg);
    
    // Process through consensus
    let mut consensus_manager_mut = consensus_manager.clone();
    if let Err(e) = consensus_manager_mut.process_query(&content).await {
        let error_msg = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            content: format!("❌ Error processing with consensus: {}", e),
            message_type: MessageType::Error,
            timestamp: chrono::Utc::now(),
            metadata: MessageMetadata::default(),
        };
        app_state.write().chat.add_message(error_msg);
    }
}

/// Copy text to clipboard using JavaScript
async fn copy_to_clipboard(text: &str) -> bool {
    use dioxus::document::eval;
    
    let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    let script = format!(
        r#"
        navigator.clipboard.writeText("{}").then(() => true).catch(() => false)
        "#,
        escaped_text
    );
    
    match eval(&script).await {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("Failed to copy to clipboard: {}", e);
            false
        }
    }
}