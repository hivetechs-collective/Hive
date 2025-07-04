//! Chat Interface Component - Claude Code Style

use dioxus::prelude::*;
use dioxus::events::{KeyboardEvent, MouseEvent};
use crate::desktop::{state::*, components::*, events::KeyboardEventUtils};

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
            class: "welcome-message",
            
            div {
                class: "welcome-banner",
                h2 { "🐝 Welcome to HiveTechs Consensus" }
                p { "AI-powered development assistant with 4-stage consensus pipeline" }
                
                div {
                    class: "system-status",
                    span { class: "status-item", "✓ AI Models: 323+ available" }
                    span { class: "status-item", "✓ Repository: Ready for analysis" }
                    span { class: "status-item", "✓ Memory: Active" }
                }
                
                div {
                    class: "quick-commands",
                    h3 { "Quick Commands:" }
                    ul {
                        li { code { "ask \"your question\"" } " - Ask the AI anything" }
                        li { code { "analyze ." } " - Analyze current directory" }
                        li { code { "plan \"your goal\"" } " - Create a development plan" }
                        li { code { "help" } " - Show all commands" }
                    }
                }
            }
        }
    }
}

/// Individual chat message component
#[component]
fn ChatMessageItem(message: ChatMessage) -> Element {
    let type_text = match message.message_type {
        MessageType::User => "👤 You",
        MessageType::Assistant => "🤖 Assistant",
        MessageType::System => "⚙️ System",
        MessageType::Error => "❌ Error",
        MessageType::Welcome => "👋 Welcome",
    };
    
    let class_suffix = match message.message_type {
        MessageType::User => "user",
        MessageType::Assistant => "assistant", 
        MessageType::System => "system",
        MessageType::Error => "error",
        MessageType::Welcome => "welcome",
    };
    
    rsx! {
        div {
            class: format!("message message-{}", class_suffix),
            "Message: {message.content}"
        }
    }
}

/// Chat input component
#[component]
fn ChatInput() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut input_text = use_signal(String::new);
    
    
    let on_send = {
        let mut app_state = app_state.clone();
        let mut input_text = input_text.clone();
        move |_evt: MouseEvent| {
            let text = input_text.read().clone();
            if !text.trim().is_empty() {
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
                
                // TODO: Send to consensus engine
            }
        }
    };
    
    let on_key_down = {
        let mut app_state = app_state.clone();
        let mut input_text = input_text.clone();
        move |evt: KeyboardEvent| {
        if KeyboardEventUtils::is_enter_key(&evt) {
            let text = input_text.read().clone();
            if !text.trim().is_empty() {
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
                
                // TODO: Send to consensus engine
            }
        } else if KeyboardEventUtils::is_escape(&evt) {
            // Clear input on Escape
            input_text.set(String::new());
        } else if evt.key() == dioxus::events::Key::Tab && !evt.modifiers().shift() {
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
                input {
                    r#type: "text",
                    class: "chat-input",
                    placeholder: "Try 'ask <question>' or 'analyze .' or 'plan <goal>'",
                    value: "{input_text.read()}",
                    oninput: move |evt| input_text.set(evt.value()),
                    onkeydown: on_key_down,
                    autofocus: true,
                }
                
                button {
                    class: "send-btn",
                    onclick: on_send,
                    disabled: input_text.read().trim().is_empty(),
                    span { class: "send-icon", "→" }
                }
            }
            
            div {
                class: "input-help",
                span {
                    class: "help-text",
                    "Press Enter to send • Shift+Enter for new line • Type 'help' for commands"
                }
            }
        }
    }
}