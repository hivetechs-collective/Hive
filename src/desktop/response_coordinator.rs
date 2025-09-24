//! Response Coordinator - Share content between Claude and Consensus
//!
//! Provides buttons and functionality to send responses between the two AI systems

use crate::desktop::consensus_integration::DesktopConsensusManager;
use crate::desktop::state::{AppState, ChatMessage, MessageMetadata, MessageType};
use dioxus::prelude::*;

/// Button to send terminal content to consensus
#[component]
pub fn SendToConsensusButton(app_state: Signal<AppState>) -> Element {
    let is_sending = use_signal(|| false);

    // Check if we have app_state context
    let has_app_state = app_state.read().chat.messages.len() >= 0; // Simple check to ensure it's valid
    let has_consensus = has_app_state;

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
                let mut is_sending = is_sending.clone();

                spawn(async move {
                    is_sending.set(true);

                    // Extract terminal content
                    if let Some(terminal_content) = extract_terminal_response(&app_state).await {
                        // Paste the content into the chat input field
                        app_state.write().chat.input_text = terminal_content;
                        tracing::info!("📋 Pasted Claude's response into chat input");
                    } else {
                        tracing::warn!("⚠️ Could not extract Claude's response from terminal");
                    }

                    is_sending.set(false);
                });
            },

            if *is_sending.read() {
                "Sending..."
            } else {
                "→ Send to Consensus (.hive)"
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
pub fn SendToClaudeButton(app_state: Signal<AppState>) -> Element {
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

                    // Get the latest curator result from database
                    match get_latest_curator_from_database().await {
                        Ok(Some((curator_content, timestamp))) => {
                            // Format the curator result for terminal
                            let formatted_content = format!(
                                "# Consensus Curator Result\n# Generated: {}\n# ---\n{}\n",
                                timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                                curator_content
                            );

                            // Send to terminal
                            if send_to_terminal(&app_state, &formatted_content).await {
                                tracing::info!("✅ Sent curator result to Claude terminal");
                            } else {
                                tracing::error!("❌ Failed to send curator result to terminal");
                            }
                        }
                        Ok(None) => {
                            tracing::warn!("⚠️ No curator results found in database");
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to fetch curator result: {}", e);
                        }
                    }

                    is_sending.set(false);
                });
            },

            if *is_sending.read() {
                "Sending..."
            } else {
                "← Send to Claude (.claude)"
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
    use crate::desktop::terminal_registry::get_active_terminal_content;

    tracing::info!("🔍 Attempting to extract terminal response...");

    // Get the active terminal content
    if let Some(content) = get_active_terminal_content().await {
        tracing::info!("📜 Got terminal content: {} chars", content.len());

        // For Claude Code, we need a different extraction strategy
        // Look for the most recent substantial block of text
        let lines: Vec<&str> = content.lines().collect();

        // Find the last non-empty block of text (Claude's response)
        let mut response_lines = Vec::new();
        let mut found_content = false;

        // Scan from bottom to top to find the most recent response
        for line in lines.iter().rev() {
            let trimmed = line.trim();

            // Skip empty lines at the bottom
            if !found_content && trimmed.is_empty() {
                continue;
            }

            // Skip command lines (like .hive)
            if trimmed == ".hive" || trimmed == ".h" || trimmed.starts_with("claude ") {
                if found_content {
                    // We've found the end of the response
                    break;
                }
                continue;
            }

            // Skip shell prompts
            if trimmed.ends_with('$')
                || trimmed.ends_with('%')
                || trimmed.ends_with('#')
                || trimmed.ends_with('>')
            {
                if found_content {
                    // We've found the end of the response
                    break;
                }
                continue;
            }

            // This looks like content
            if !trimmed.is_empty() {
                found_content = true;
                response_lines.push(line.to_string());
            } else if found_content {
                // Add empty lines that are part of the response
                response_lines.push(line.to_string());
            }
        }

        // Reverse to get the correct order (we scanned bottom to top)
        response_lines.reverse();

        // Join and clean up
        let response = response_lines.join("\n").trim().to_string();

        if response.len() > 50 {
            // Minimum 50 chars for a meaningful response
            tracing::info!("✅ Extracted Claude response: {} chars", response.len());
            tracing::info!(
                "📝 Response preview: {}",
                response.chars().take(100).collect::<String>()
            );
            return Some(response);
        } else {
            tracing::warn!("⚠️ Response too short or empty: {} chars", response.len());
        }
    } else {
        tracing::warn!("❌ No active terminal found in registry");
    }

    None
}

/// Extract the last consensus response
async fn extract_consensus_response(app_state: &Signal<AppState>) -> Option<String> {
    let state = app_state.read();

    // Find the last assistant message in chat
    state
        .chat
        .messages
        .iter()
        .rev()
        .find(|msg| matches!(msg.message_type, MessageType::Assistant))
        .map(|msg| msg.content.clone())
}

/// Send content to terminal as input
async fn send_to_terminal(_app_state: &Signal<AppState>, content: &str) -> bool {
    use crate::desktop::terminal_registry::send_to_active_terminal;

    // Format the content as a command for the terminal
    // We'll send it as a comment so it doesn't execute, but Claude can see it
    let formatted_content = format!(
        "# Consensus response:\n# {}\n",
        content.replace('\n', "\n# ")
    );

    if send_to_active_terminal(&formatted_content) {
        tracing::info!("📤 Sent consensus response to terminal");
        true
    } else {
        tracing::error!("❌ Failed to send to terminal");
        false
    }
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

/// Get the latest curator result from the database
async fn get_latest_curator_from_database(
) -> Result<Option<(String, chrono::DateTime<chrono::Utc>)>, anyhow::Error> {
    use crate::core::database::get_database;

    let db = get_database().await?;
    db.get_latest_curator_result().await
}

/// Copy text to clipboard using JavaScript
async fn copy_to_clipboard(text: &str) -> bool {
    use dioxus::document::eval;

    let escaped_text = text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
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
