//! Chat Interface Component - Claude Code Style

use dioxus::prelude::*;
use dioxus::events::{KeyboardEvent, MouseEvent};
use crate::desktop::{state::*, components::*, events::KeyboardEventUtils, consensus_integration::{use_consensus, DesktopConsensusManager}};

/// Chat Interface Component
#[component]
pub fn ChatInterface() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let state = app_state.read();
    
    rsx! {
        div {
            class: "chat-container",
            
            // Chat Messages Area
            div {
                class: "chat-messages",
                if state.chat.messages.is_empty() {
                    WelcomeMessage {}
                } else {
                    for message in &state.chat.messages {
                        ChatMessageItem { message: message.clone() }
                    }
                }
            }
            
            // Chat Input Area
            ChatInput {}
        }
    }
}

/// Welcome message component
#[component]
fn WelcomeMessage() -> Element {
    rsx! {
        div {
            class: "welcome-container",
            
            div {
                class: "welcome-content",
                h1 { 
                    class: "welcome-title",
                    "HiveTechs Consensus" 
                }
                p { 
                    class: "welcome-subtitle",
                    "AI-powered development assistant with 4-stage consensus pipeline" 
                }
                
                div {
                    class: "system-status-grid",
                    div { 
                        class: "status-card",
                        div { class: "status-icon success", "✓" }
                        div { class: "status-text", "323+ AI Models" }
                    }
                    div { 
                        class: "status-card",
                        div { class: "status-icon success", "✓" }
                        div { class: "status-text", "Repository Ready" }
                    }
                    div { 
                        class: "status-card",
                        div { class: "status-icon success", "✓" }
                        div { class: "status-text", "Memory Active" }
                    }
                }
                
                div {
                    class: "commands-section",
                    h3 { 
                        class: "commands-title",
                        "Quick Commands" 
                    }
                    div {
                        class: "command-list",
                        div { 
                            class: "command-item",
                            code { class: "command-code", "ask \"your question\"" }
                            span { class: "command-desc", "Ask the AI anything" }
                        }
                        div { 
                            class: "command-item",
                            code { class: "command-code", "analyze ." }
                            span { class: "command-desc", "Analyze current directory" }
                        }
                        div { 
                            class: "command-item",
                            code { class: "command-code", "plan \"your goal\"" }
                            span { class: "command-desc", "Create a development plan" }
                        }
                        div { 
                            class: "command-item",
                            code { class: "command-code", "help" }
                            span { class: "command-desc", "Show all commands" }
                        }
                    }
                }
            }
        }
    }
}

/// Individual chat message component
#[component]
fn ChatMessageItem(message: ChatMessage) -> Element {
    let is_user = matches!(message.message_type, MessageType::User);
    let is_error = matches!(message.message_type, MessageType::Error);
    
    let message_class = match message.message_type {
        MessageType::User => "message-row user",
        MessageType::Assistant => "message-row assistant",
        MessageType::System => "message-row system",
        MessageType::Error => "message-row error",
        MessageType::Welcome => "message-row welcome",
    };
    
    rsx! {
        div {
            class: message_class,
            
            div {
                class: "message-bubble",
                
                // Message header (for non-user messages)
                if !is_user {
                    div {
                        class: "message-header",
                        match message.message_type {
                            MessageType::Assistant => "Assistant",
                            MessageType::System => "System",
                            MessageType::Error => "Error",
                            _ => ""
                        }
                    }
                }
                
                // Message content
                div {
                    class: "message-content",
                    dangerous_inner_html: format_message_content(&message.content),
                }
                
                // Timestamp
                div {
                    class: "message-timestamp",
                    {format_timestamp(&message.timestamp)}
                }
            }
        }
    }
}

/// Format message content with basic markdown support
fn format_message_content(content: &str) -> String {
    // Basic markdown formatting
    let formatted = content
        .replace("```", "<pre><code>")
        .replace("```", "</code></pre>")
        .replace("`", "<code>")
        .replace("`", "</code>")
        .replace("\n", "<br>");
    
    formatted
}

/// Format timestamp to human readable
fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    timestamp.format("%H:%M").to_string()
}

/// Process a message from the chat input
fn process_message(
    text: String,
    app_state: &mut Signal<AppState>,
    input_text: &mut Signal<String>,
    consensus_manager: &Option<DesktopConsensusManager>,
    show_onboarding: &mut Signal<bool>,
) {
    // Check if we need to show onboarding (no profiles configured)
    if consensus_manager.is_none() {
        // Check if profiles exist
        let mut app_state_clone = app_state.clone();
        let mut show_onboarding_clone = show_onboarding.clone();
        spawn(async move {
            use crate::desktop::dialogs::load_existing_profiles;
            
            match load_existing_profiles().await {
                Ok(profiles) if profiles.is_empty() => {
                    tracing::info!("No profiles configured - showing onboarding");
                    show_onboarding_clone.set(true);
                }
                Ok(_) => {
                    tracing::error!("Consensus manager not available but profiles exist");
                    let error_msg = ChatMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        content: "Error: Consensus engine not initialized. Please restart the application.".to_string(),
                        message_type: MessageType::Error,
                        timestamp: chrono::Utc::now(),
                        metadata: MessageMetadata::default(),
                    };
                    app_state_clone.write().chat.add_message(error_msg);
                }
                Err(e) => {
                    tracing::error!("Failed to load profiles: {}", e);
                    show_onboarding_clone.set(true);
                }
            }
        });
        return;
    }
    
    // Add user message
    let message = ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        content: text.clone(),
        message_type: MessageType::User,
        timestamp: chrono::Utc::now(),
        metadata: MessageMetadata::default(),
    };
    
    app_state.write().chat.add_message(message);
    
    // Start consensus processing
    app_state.write().consensus.start_consensus();
    
    // Clear input
    input_text.set(String::new());
    
    // Send to consensus engine
    if let Some(consensus) = consensus_manager {
        let query = text.clone();
        let mut app_state_consensus = app_state.clone();
        let mut show_onboarding_clone = show_onboarding.clone();
        let mut consensus_clone = consensus.clone();
        
        spawn(async move {
            match consensus_clone.process_query(&query).await {
                Ok(response) => {
                    let response_msg = ChatMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        content: response,
                        message_type: MessageType::Assistant,
                        timestamp: chrono::Utc::now(),
                        metadata: MessageMetadata {
                            cost: Some(app_state_consensus.read().consensus.estimated_cost),
                            model: Some("4-stage consensus".to_string()),
                            processing_time: None,
                            token_count: Some(app_state_consensus.read().consensus.total_tokens as u32),
                        },
                    };
                    app_state_consensus.write().chat.add_message(response_msg);
                }
                Err(e) => {
                    tracing::error!("Consensus processing error: {}", e);
                    let error_msg = ChatMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        content: format!("Error processing request: {}", e),
                        message_type: MessageType::Error,
                        timestamp: chrono::Utc::now(),
                        metadata: MessageMetadata::default(),
                    };
                    app_state_consensus.write().chat.add_message(error_msg);
                    
                    // If the error is about missing profiles or API key, show onboarding
                    if e.to_string().contains("No valid OpenRouter API key") || 
                       e.to_string().contains("profile") {
                        show_onboarding_clone.set(true);
                    }
                }
            }
            
            // Complete consensus
            app_state_consensus.write().consensus.complete_consensus();
        });
    }
}

/// Chat input component
#[component]
fn ChatInput() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut input_text = use_signal(String::new);
    let mut is_composing = use_signal(|| false);
    let consensus_manager = use_consensus();
    let mut show_onboarding = use_context::<Signal<bool>>();
    
    let on_send_click = {
        let mut app_state = app_state.clone();
        let mut input_text = input_text.clone();
        let consensus_manager = consensus_manager.clone();
        let mut show_onboarding = show_onboarding.clone();
        
        move |_evt: MouseEvent| {
            let text = input_text.read().clone();
            if !text.trim().is_empty() {
                process_message(text, &mut app_state, &mut input_text, &consensus_manager, &mut show_onboarding);
            }
        }
    };
    
    let on_key_down = {
        let mut app_state = app_state.clone();
        let mut input_text = input_text.clone();
        let consensus_manager = consensus_manager.clone();
        let mut show_onboarding = show_onboarding.clone();
        let is_composing = is_composing.clone();
        
        move |evt: KeyboardEvent| {
            if KeyboardEventUtils::is_enter_key(&evt) && !evt.modifiers().shift() && !*is_composing.read() {
                let text = input_text.read().clone();
                if !text.trim().is_empty() {
                    process_message(text, &mut app_state, &mut input_text, &consensus_manager, &mut show_onboarding);
                }
            } else if KeyboardEventUtils::is_escape(&evt) {
                // Clear input on Escape
                input_text.set(String::new());
            } else if evt.key() == dioxus::html::input_data::keyboard_types::Key::Tab && !evt.modifiers().shift() {
                // Auto-complete or show suggestions
                // TODO: Implement auto-complete
            }
        }
    };
    
    rsx! {
        div {
            class: "chat-input-container",
            
            div {
                class: "input-wrapper",
                textarea {
                    class: "chat-input",
                    placeholder: "Message Hive...",
                    value: "{input_text.read()}",
                    rows: "1",
                    oninput: move |evt| {
                        input_text.set(evt.value());
                        // Auto-resize logic would go here
                    },
                    onkeydown: on_key_down,
                    onfocus: move |_| is_composing.set(false),
                    oncompositionstart: move |_| is_composing.set(true),
                    oncompositionend: move |_| is_composing.set(false),
                    autofocus: true,
                    spellcheck: "false",
                }
                
                button {
                    class: if input_text.read().trim().is_empty() { "send-btn disabled" } else { "send-btn" },
                    onclick: on_send_click,
                    disabled: input_text.read().trim().is_empty(),
                    svg {
                        width: "20",
                        height: "20",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path {
                            d: "M22 2L11 13"
                        }
                        path {
                            d: "M22 2L15 22L11 13L2 9L22 2Z"
                        }
                    }
                }
            }
            
            div {
                class: "input-shortcuts",
                span {
                    class: "shortcut-hint",
                    "Enter to send"
                }
                span {
                    class: "shortcut-divider",
                    "·"
                }
                span {
                    class: "shortcut-hint",
                    "Shift+Enter for new line"
                }
            }
        }
    }
}