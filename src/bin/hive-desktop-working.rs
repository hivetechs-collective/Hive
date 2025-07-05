#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

fn main() {
    // Launch the desktop app
    dioxus_desktop::launch_cfg(
        App,
        Config::default()
            .with_window(WindowBuilder::new()
                .with_title("🐝 Hive AI Desktop")
                .with_inner_size(dioxus_desktop::LogicalSize::new(1200.0, 800.0))
            )
    );
}

#[derive(Debug, Clone, PartialEq)]
struct Message {
    text: String,
    is_user: bool,
}

fn App(cx: Scope) -> Element {
    // State management
    let messages = use_state(cx, || vec![
        Message { text: "🐝 Welcome to Hive AI Desktop!".to_string(), is_user: false },
        Message { text: "This is the Rust-powered version with VS Code-style interface.".to_string(), is_user: false },
    ]);
    let input_value = use_state(cx, String::new);
    let is_processing = use_state(cx, || false);

    // Send message handler
    let send_message = |messages: &UseState<Vec<Message>>, input_value: &UseState<String>, is_processing: &UseState<bool>| {
        if !input_value.is_empty() && !**is_processing {
            let user_msg = input_value.get().clone();
            
            // Add user message
            messages.modify(|msgs| {
                msgs.push(Message { text: user_msg.clone(), is_user: true });
            });
            
            // Clear input
            input_value.set(String::new());
            
            // Simulate processing
            is_processing.set(true);
            
            // Simulate AI response
            messages.modify(|msgs| {
                msgs.push(Message { 
                    text: "🐝 I'm currently running in demo mode. The full consensus engine is being integrated.".to_string(), 
                    is_user: false 
                });
            });
            
            is_processing.set(false);
        }
    };

    cx.render(rsx! {
        // Inject VS Code-style CSS
        style { include_str!("../../desktop_styles.css") }
        
        div {
            class: "app-container",
            
            // Header
            div {
                class: "header",
                div {
                    class: "header-left",
                    h1 { class: "app-title", "🐝 Hive AI Desktop" }
                    span { class: "app-version", "v2.0.2" }
                }
                div {
                    class: "header-right",
                    span { class: "status-indicator", "✓ Ready" }
                }
            }
            
            // Main content
            div {
                class: "main-content",
                
                // Sidebar
                div {
                    class: "sidebar",
                    div { class: "sidebar-section-title", "EXPLORER" }
                    div { class: "sidebar-item", "📁 Project Files" }
                    div { class: "sidebar-item", "🔍 Search" }
                    div { class: "sidebar-item", "⚙️ Settings" }
                    
                    div { class: "sidebar-section-title", style: "margin-top: 20px;", "CONSENSUS" }
                    div { class: "sidebar-item active", "💬 Chat" }
                    div { class: "sidebar-item", "📊 Analytics" }
                    div { class: "sidebar-item", "🧠 Memory" }
                }
                
                // Chat area
                div {
                    class: "chat-container",
                    
                    // Messages
                    div {
                        class: "messages-area",
                        messages.iter().map(|msg| rsx! {
                            div {
                                class: if msg.is_user { "message user-message" } else { "message ai-message" },
                                "{msg.text}"
                            }
                        })
                    }
                    
                    // Input area
                    div {
                        class: "input-area",
                        input {
                            class: "message-input",
                            value: "{input_value}",
                            placeholder: "Ask Hive anything...",
                            disabled: **is_processing,
                            oninput: move |evt| input_value.set(evt.value.clone()),
                            onkeypress: move |evt| {
                                if evt.key() == Key::Enter {
                                    send_message(&messages, &input_value, &is_processing);
                                }
                            }
                        }
                        button {
                            class: "send-button",
                            disabled: input_value.is_empty() || **is_processing,
                            onclick: move |_| send_message(&messages, &input_value, &is_processing),
                            if **is_processing { "Processing..." } else { "Send" }
                        }
                    }
                }
                
                // Right panel (consensus stages)
                div {
                    class: "right-panel",
                    div { class: "panel-title", "CONSENSUS PIPELINE" }
                    
                    div {
                        class: "stage-item",
                        div { class: "stage-indicator stage-generator" }
                        span { "Generator" }
                    }
                    div {
                        class: "stage-item",
                        div { class: "stage-indicator stage-refiner" }
                        span { "Refiner" }
                    }
                    div {
                        class: "stage-item",
                        div { class: "stage-indicator stage-validator" }
                        span { "Validator" }
                    }
                    div {
                        class: "stage-item",
                        div { class: "stage-indicator stage-curator" }
                        span { "Curator" }
                    }
                }
            }
            
            // Status bar
            div {
                class: "status-bar",
                div { class: "status-left", "Rust Desktop Mode" }
                div { class: "status-right", "UTF-8 | Ready" }
            }
        }
    })
}