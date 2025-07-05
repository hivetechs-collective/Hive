#![allow(non_snake_case)]

use dioxus::prelude::*;

const DESKTOP_STYLES: &str = r#"
    /* VS Code-style CSS */
    body {
        margin: 0;
        padding: 0;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
        background: #1e1e1e;
        color: #cccccc;
        height: 100vh;
        overflow: hidden;
        font-size: 14px;
    }
    
    .app-container {
        display: flex;
        height: 100vh;
        flex-direction: column;
    }
    
    /* Header styles */
    .header {
        height: 32px;
        background: #2d2d30;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 20px;
        border-bottom: 1px solid #3e3e42;
    }
    
    .header-left {
        display: flex;
        align-items: center;
        gap: 10px;
    }
    
    .app-title {
        font-size: 14px;
        font-weight: 600;
        margin: 0;
    }
    
    .app-version {
        font-size: 12px;
        color: #858585;
    }
    
    .header-right {
        font-size: 12px;
    }
    
    .status-indicator {
        color: #4ec9b0;
    }
    
    .main-content {
        display: flex;
        flex: 1;
        overflow: hidden;
    }
    
    /* Sidebar styles */
    .sidebar {
        width: 250px;
        background: #252526;
        display: flex;
        flex-direction: column;
        border-right: 1px solid #3e3e42;
        overflow-y: auto;
    }
    
    .sidebar-section-title {
        font-size: 11px;
        font-weight: 600;
        text-transform: uppercase;
        color: #858585;
        padding: 10px 20px 5px 20px;
        letter-spacing: 0.5px;
    }
    
    .sidebar-item {
        padding: 6px 20px;
        font-size: 13px;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 8px;
        color: #cccccc;
        transition: background-color 0.1s;
    }
    
    .sidebar-item:hover {
        background: #2a2d2e;
    }
    
    .sidebar-item.active {
        background: #094771;
        color: #ffffff;
    }
    
    /* Chat container styles */
    .chat-container {
        flex: 1;
        display: flex;
        flex-direction: column;
        background: #1e1e1e;
    }
    
    .panel-header {
        background: #2d2d30;
        padding: 10px 20px;
        border-bottom: 1px solid #3e3e42;
        font-weight: 600;
        font-size: 14px;
    }
    
    .messages-area {
        flex: 1;
        overflow-y: auto;
        padding: 20px;
        display: flex;
        flex-direction: column;
        gap: 15px;
    }
    
    .message {
        padding: 12px 16px;
        border-radius: 6px;
        max-width: 70%;
        font-size: 14px;
        line-height: 1.5;
    }
    
    .user-message {
        background: #007acc;
        color: white;
        align-self: flex-end;
    }
    
    .ai-message {
        background: #2d2d30;
        border: 1px solid #3e3e42;
        align-self: flex-start;
    }
    
    /* Input area styles */
    .input-area {
        display: flex;
        padding: 16px 20px;
        gap: 12px;
        background: #252526;
        border-top: 1px solid #3e3e42;
    }
    
    .message-input {
        flex: 1;
        background: #3c3c3c;
        border: 1px solid #3e3e42;
        color: #cccccc;
        padding: 10px 14px;
        border-radius: 4px;
        font-size: 14px;
        font-family: inherit;
        outline: none;
        transition: border-color 0.1s;
    }
    
    .message-input:focus {
        border-color: #007acc;
    }
    
    .send-button {
        padding: 10px 20px;
        background: #007acc;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-weight: 600;
        font-size: 14px;
        transition: background-color 0.1s;
    }
    
    .send-button:hover {
        background: #005a9e;
    }
    
    .send-button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
    
    /* Right panel styles */
    .right-panel {
        width: 300px;
        background: #252526;
        border-left: 1px solid #3e3e42;
        padding: 20px;
    }
    
    .panel-title {
        font-size: 11px;
        font-weight: 600;
        text-transform: uppercase;
        color: #858585;
        margin-bottom: 16px;
        letter-spacing: 0.5px;
    }
    
    .stage-item {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 12px;
        font-size: 13px;
    }
    
    .stage-indicator {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        background: #3e3e42;
        flex-shrink: 0;
    }
    
    .stage-indicator.stage-generator { background: #f48771; }
    .stage-indicator.stage-refiner { background: #4ec9b0; }
    .stage-indicator.stage-validator { background: #569cd6; }
    .stage-indicator.stage-curator { background: #c586c0; }
    
    /* Status bar styles */
    .status-bar {
        height: 24px;
        background: #007acc;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 15px;
        font-size: 12px;
        color: white;
    }
    
    .status-left, .status-right {
        display: flex;
        align-items: center;
        gap: 15px;
    }
"#;

fn main() {
    // Launch the desktop app
    dioxus::launch(App);
}

#[derive(Debug, Clone, PartialEq)]
struct Message {
    text: String,
    is_user: bool,
}

fn App() -> Element {
    // State management
    let mut messages = use_signal(|| vec![
        Message { text: "🐝 Welcome to Hive AI Desktop!".to_string(), is_user: false },
        Message { text: "This is the Rust-powered version with VS Code-style interface.".to_string(), is_user: false },
    ]);
    let mut input_value = use_signal(String::new);
    let mut is_processing = use_signal(|| false);

    // Send message handler
    let send_message = move |messages: &mut Signal<Vec<Message>>, input_value: &mut Signal<String>, is_processing: &mut Signal<bool>| {
        if !input_value.read().is_empty() && !*is_processing.read() {
            let user_msg = input_value.read().clone();
            
            // Add user message
            messages.write().push(Message { text: user_msg.clone(), is_user: true });
            
            // Clear input
            input_value.write().clear();
            
            // Simulate processing
            *is_processing.write() = true;
            
            // Simulate AI response
            messages.write().push(Message { 
                text: "🐝 I'm currently running in demo mode. The full consensus engine is being integrated.".to_string(), 
                is_user: false 
            });
            
            *is_processing.write() = false;
        }
    };

    rsx! {
        // Inject VS Code-style CSS
        style { "{DESKTOP_STYLES}" }
        
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
                        for msg in messages.read().iter() {
                            div {
                                class: if msg.is_user { "message user-message" } else { "message ai-message" },
                                "{msg.text}"
                            }
                        }
                    }
                    
                    // Input area
                    div {
                        class: "input-area",
                        input {
                            class: "message-input",
                            value: "{input_value.read()}",
                            placeholder: "Ask Hive anything...",
                            disabled: *is_processing.read(),
                            oninput: move |evt| *input_value.write() = evt.value().clone(),
                            onkeypress: move |evt| {
                                if evt.code() == dioxus::html::input_data::keyboard_types::Code::Enter {
                                    send_message(&mut messages.clone(), &mut input_value.clone(), &mut is_processing.clone());
                                }
                            }
                        }
                        button {
                            class: "send-button",
                            disabled: input_value.read().is_empty() || *is_processing.read(),
                            onclick: move |_| send_message(&mut messages.clone(), &mut input_value.clone(), &mut is_processing.clone()),
                            if *is_processing.read() { "Processing..." } else { "Send" }
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
    }
}