#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};

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
    
    /* Main content area */
    .main-content {
        display: flex;
        flex: 1;
        overflow: hidden;
    }
    
    /* Sidebar styles (left) */
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
    
    /* Code editor area (center) */
    .editor-container {
        flex: 1;
        display: flex;
        flex-direction: column;
        background: #1e1e1e;
        min-width: 0;
    }
    
    .editor-tabs {
        height: 35px;
        background: #2d2d30;
        display: flex;
        align-items: center;
        border-bottom: 1px solid #3e3e42;
        padding: 0 10px;
    }
    
    .editor-tab {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 0 15px;
        height: 100%;
        background: #2d2d30;
        border-right: 1px solid #3e3e42;
        font-size: 13px;
        cursor: pointer;
        transition: background-color 0.1s;
    }
    
    .editor-tab.active {
        background: #1e1e1e;
        border-bottom: 1px solid #1e1e1e;
    }
    
    .editor-tab:hover {
        background: #323234;
    }
    
    .editor-content {
        flex: 1;
        padding: 20px;
        overflow-y: auto;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: 13px;
        line-height: 1.6;
    }
    
    .welcome-message {
        color: #858585;
        text-align: center;
        margin-top: 100px;
    }
    
    /* Chat panel (right) */
    .chat-panel {
        width: 350px;
        background: #252526;
        border-left: 1px solid #3e3e42;
        display: flex;
        flex-direction: column;
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
        max-width: 85%;
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
        background: #1e1e1e;
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
    
    .git-branch {
        display: flex;
        align-items: center;
        gap: 5px;
    }
"#;

fn main() {
    // Configure the desktop app with proper window settings
    let config = Config::new()
        .with_window(WindowBuilder::new()
            .with_title("Hive AI Desktop")
            .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
            .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0)));
    
    // Launch with configuration
    dioxus::desktop::launch_cfg(App, config);
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
    let mut selected_file = use_signal(|| None::<String>);

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
            
            // Main content (no header bar - VS Code style)
            div {
                class: "main-content",
                
                // Sidebar (left)
                div {
                    class: "sidebar",
                    div { class: "sidebar-section-title", "EXPLORER" }
                    div { 
                        class: "sidebar-item",
                        onclick: move |_| *selected_file.write() = Some("README.md".to_string()),
                        "📄 README.md" 
                    }
                    div { 
                        class: "sidebar-item",
                        onclick: move |_| *selected_file.write() = Some("src/main.rs".to_string()),
                        "📄 src/main.rs" 
                    }
                    div { 
                        class: "sidebar-item",
                        onclick: move |_| *selected_file.write() = Some("Cargo.toml".to_string()),
                        "📄 Cargo.toml" 
                    }
                    
                    div { class: "sidebar-section-title", style: "margin-top: 20px;", "ACTIONS" }
                    div { class: "sidebar-item", "🔍 Search" }
                    div { class: "sidebar-item", "📊 Analytics" }
                    div { class: "sidebar-item", "🧠 Memory" }
                    div { class: "sidebar-item", "⚙️ Settings" }
                }
                
                // Code editor area (center)
                div {
                    class: "editor-container",
                    
                    // Editor tabs
                    div {
                        class: "editor-tabs",
                        if let Some(file) = selected_file.read().as_ref() {
                            div {
                                class: "editor-tab active",
                                "{file}"
                            }
                        }
                    }
                    
                    // Editor content
                    div {
                        class: "editor-content",
                        if let Some(file) = selected_file.read().as_ref() {
                            div { 
                                "// Content of {file} would be displayed here\n\n",
                                "// This is a placeholder for the code editor.\n",
                                "// Integration with a syntax highlighting library\n",
                                "// like CodeMirror or Monaco would go here."
                            }
                        } else {
                            div { 
                                class: "welcome-message",
                                "Select a file from the explorer to view its contents" 
                            }
                        }
                    }
                }
                
                // Chat panel (right)
                div {
                    class: "chat-panel",
                    
                    // Panel header
                    div {
                        class: "panel-header",
                        "🐝 Hive AI Chat"
                    }
                    
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
            }
            
            // Status bar
            div {
                class: "status-bar",
                div { 
                    class: "status-left",
                    div {
                        class: "git-branch",
                        "🌿 main"
                    }
                    "•"
                    "✓ 0 problems"
                }
                div { 
                    class: "status-right",
                    "Ln 1, Col 1",
                    "•",
                    "UTF-8",
                    "•",
                    "Rust"
                }
            }
        }
    }
}