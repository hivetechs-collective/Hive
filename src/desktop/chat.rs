//! Chat Interface Component - Claude Code Style

use crate::desktop::{
    consensus_integration::{use_consensus_with_version, DesktopConsensusManager},
    events::KeyboardEventUtils,
    simple_chat_processor,
    state::*,
};
use dioxus::events::{KeyboardEvent, MouseEvent};
use dioxus::prelude::*;

/// Chat Interface Component
#[component]
pub fn ChatInterface() -> Element {
    tracing::info!("🎯 ChatInterface component rendering!");
    let app_state = use_context::<Signal<AppState>>();
    let state = app_state.read();
    let api_keys_version = use_context::<Signal<u32>>();
    let consensus_manager = use_consensus_with_version(*api_keys_version.read());
    
    // Watch for repository context update trigger
    let repository_context_trigger = state.repository_context_update_trigger;
    let consensus_manager_for_effect = consensus_manager.clone();
    use_effect(move || {
        // Skip the initial trigger (0)
        if repository_context_trigger > 0 {
            tracing::info!("Repository context update triggered: {}", repository_context_trigger);
            if let Some(manager) = consensus_manager_for_effect.clone() {
                dioxus::prelude::spawn(async move {
                    if let Err(e) = manager.update_repository_context().await {
                        tracing::warn!("Failed to update repository context: {}", e);
                    } else {
                        tracing::info!("✅ Repository context updated successfully from file explorer");
                    }
                });
            }
        }
    });

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

            // Cancel button area - placed between messages and input
            if state.consensus.is_running {
                div {
                    style: "display: flex; justify-content: center; padding: 12px; background: rgba(215, 58, 73, 0.05); border-top: 1px solid rgba(215, 58, 73, 0.2); border-bottom: 1px solid rgba(215, 58, 73, 0.2);",
                    
                    button {
                        style: "background: #d73a49; border: none; color: white; padding: 10px 24px; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 500; display: flex; align-items: center; gap: 8px; transition: all 0.2s ease; box-shadow: 0 2px 4px rgba(0,0,0,0.2);",
                        onmouseover: move |_| {
                            tracing::info!("🔥 Cancel button hovered!");
                        },
                        onclick: {
                            let consensus_manager = consensus_manager.clone();
                            move |_| {
                                tracing::info!("🛑 Cancel button clicked from ChatInterface!");
                                if let Some(mut manager) = consensus_manager.clone() {
                                    spawn(async move {
                                        if let Err(e) = manager.cancel_consensus("User cancelled from chat interface").await {
                                            tracing::warn!("Failed to cancel consensus: {}", e);
                                        }
                                    });
                                }
                            }
                        },
                        span { style: "font-size: 16px;", "✕" }
                        span { "Cancel Consensus Run" }
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
                        "Quick Actions"
                    }
                    div {
                        class: "command-list",
                        button {
                            style: "display: inline-flex; align-items: center; padding: 10px 20px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 500; transition: background 0.2s; margin-bottom: 20px;",
                            onmouseover: move |e| {
                                // Would set hover style if we had mutable access to style
                            },
                            onclick: {
                                let mut app_state = use_context::<Signal<AppState>>();
                                move |_| {
                                    let mut app_state = app_state.clone();
                                    spawn(async move {
                                        // Open folder dialog
                                        let dialog = rfd::AsyncFileDialog::new();
                                        if let Some(folder) = dialog.pick_folder().await {
                                            let path = folder.path().to_path_buf();
                                            tracing::info!("📁 User selected folder: {}", path.display());
                                            
                                            // Load the project
                                            if let Err(e) = app_state.write().load_project(path.clone()).await {
                                                tracing::error!("Failed to load project: {}", e);
                                            } else {
                                                tracing::info!("✅ Project loaded successfully");
                                                // The use_effect watcher in app.rs will trigger repository context update
                                            }
                                        }
                                    });
                                }
                            },
                            i { class: "fa-solid fa-folder-open", style: "margin-right: 8px;" }
                            "Open Folder"
                        }
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

// Removed old process_message function - all processing now goes through hybrid_chat_processor

/// Chat input component
#[component]
fn ChatInput() -> Element {
    tracing::info!("🎯 ChatInput component rendering!");
    let mut app_state = use_context::<Signal<AppState>>();
    let mut input_text = use_signal(String::new);
    let mut is_composing = use_signal(|| false);
    let api_keys_version = use_context::<Signal<u32>>();
    // Since the consensus engine now always loads from database, we only need API keys version
    let consensus_manager = use_consensus_with_version(*api_keys_version.read());
    let mut show_onboarding = use_context::<Signal<bool>>();

    let on_send_click = {
        let mut app_state = app_state.clone();
        let mut input_text = input_text.clone();
        let consensus_manager = consensus_manager.clone();
        let mut show_onboarding = show_onboarding.clone();

        move |_evt: MouseEvent| {
            let text = input_text.read().clone();
            if !text.trim().is_empty() {
                simple_chat_processor::process_message(
                    text,
                    &mut app_state,
                    &mut input_text,
                    &consensus_manager,
                    &mut show_onboarding,
                );
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
            if KeyboardEventUtils::is_enter_key(&evt)
                && !evt.modifiers().shift()
                && !*is_composing.read()
            {
                let text = input_text.read().clone();
                if !text.trim().is_empty() {
                    simple_chat_processor::process_message(
                        text,
                        &mut app_state,
                        &mut input_text,
                        &consensus_manager,
                        &mut show_onboarding,
                    );
                }
            } else if KeyboardEventUtils::is_escape(&evt) {
                // Clear input on Escape
                input_text.set(String::new());
            } else if evt.key() == dioxus::html::input_data::keyboard_types::Key::Tab
                && evt.modifiers().shift()
            {
                // Shift+Tab: Cycle Claude Code modes (pass to Claude Code)
                tracing::info!("🔄 Shift+Tab pressed - cycling Claude Code modes");
                cycle_claude_mode(&mut app_state);
            } else if evt.key() == dioxus::html::input_data::keyboard_types::Key::Tab
                && !evt.modifiers().shift()
            {
                // Tab: Auto-complete or show suggestions
                // TODO: Implement auto-complete
            }
        }
    };

    // Read consensus state to trigger reactivity
    let is_consensus_running = app_state.read().consensus.is_running;
    tracing::info!("🔍 ChatInput render - consensus.is_running: {}", is_consensus_running);
    tracing::info!("🔍 ChatInput render - consensus.is_active: {}", app_state.read().consensus.is_active);
    tracing::info!("🔍 ChatInput render - consensus.current_stage: {:?}", app_state.read().consensus.current_stage);

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

                // Send button (always show, but disabled when input is empty or consensus is running)
                button {
                    class: if input_text.read().trim().is_empty() || is_consensus_running { "send-btn disabled" } else { "send-btn" },
                    onclick: on_send_click,
                    disabled: input_text.read().trim().is_empty() || is_consensus_running,
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
                style: "display: flex; justify-content: space-between; align-items: center;",
                
                // Left side: Basic shortcuts
                div {
                    style: "display: flex; gap: 8px;",
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
                
                // Right side: Claude Code Controls
                div {
                    style: "display: flex; gap: 16px; align-items: center;",
                    
                    // Claude Code SDK Status
                    div {
                        style: "display: flex; align-items: center; gap: 4px; padding: 4px 8px; border-radius: 4px; background: rgba(0, 122, 204, 0.1); cursor: pointer;",
                        onclick: move |_| {
                            spawn(async {
                                tracing::info!("🧪 Testing Claude integration...");
                                let status = simple_chat_processor::get_claude_status().await;
                                tracing::info!("Claude status: {}", status);
                            });
                        },
                        span {
                            style: "font-size: 12px;",
                            if app_state.read().claude_sdk_connected {
                                "✅"
                            } else {
                                "❌"
                            }
                        }
                        span {
                            style: "font-size: 11px; color: #007ACC;",
                            if app_state.read().claude_sdk_connected {
                                "Claude Code SDK Connected"
                            } else {
                                "Claude Code SDK Offline (click to test)"
                            }
                        }
                    }
                    
                    // Claude Code mode status (read-only display)
                    div {
                        style: "display: flex; align-items: center; gap: 4px;",
                        
                        // Mode indicator
                        span {
                            style: "font-size: 14px; color: #007ACC;",
                            match app_state.read().claude_mode.as_str() {
                                "normal" => "✏️",
                                "auto-accept" => "⏵⏵",
                                "plan" => "📋",
                                _ => "✏️"
                            }
                        }
                        
                        // Mode status (read-only)
                        span {
                            style: "color: #007ACC; font-size: 11px; padding: 2px 6px; border-radius: 3px;", 
                            match app_state.read().claude_mode.as_str() {
                                "normal" => "normal mode",
                                "auto-accept" => "auto-accept on",
                                "plan" => "plan mode on",
                                _ => "normal mode"
                            }
                        }
                        
                        // Hint for mode switching
                        span {
                            style: "color: #666; font-size: 10px; margin-left: 8px;",
                            "Shift+Tab to cycle"
                        }
                    }
                    
                    // Claude auth method toggle
                    div {
                        style: "display: flex; align-items: center; gap: 4px;",
                        
                        // Auth method indicator
                        span {
                            style: "font-size: 14px; color: #28a745;",
                            match app_state.read().claude_auth_method.as_str() {
                                "ApiKey" => "🔑",
                                "OAuth" => "🎫",
                                "NotSelected" => "❓",
                                _ => "❓"
                            }
                        }
                        
                        // Auth method button
                        button {
                            style: "background: none; border: none; color: #28a745; cursor: pointer; font-size: 11px; padding: 2px 6px; border-radius: 3px; transition: all 0.2s; text-decoration: underline;",
                            onclick: {
                                let mut app_state = app_state.clone();
                                move |_| {
                                    let current = app_state.read().claude_auth_method.clone();
                                    let next_method = match current.as_str() {
                                        "NotSelected" => "ApiKey",
                                        "ApiKey" => "OAuth",
                                        "OAuth" => "ApiKey",
                                        _ => "ApiKey"
                                    };
                                    app_state.write().claude_auth_method = next_method.to_string();
                                    
                                    // Save the auth method to database
                                    if let Err(e) = crate::desktop::simple_db::save_config("claude_auth_method", next_method) {
                                        tracing::error!("Failed to save Claude auth method: {}", e);
                                    } else {
                                        tracing::info!("🔄 Claude auth method toggled to: {}", next_method);
                                        tracing::info!("📝 User switched authentication from {} to {}", current, next_method);
                                        
                                        // Check if we have credentials for the new method
                                        if next_method == "OAuth" {
                                            // Check if OAuth credentials exist
                                            if let Ok(Some(oauth_json)) = crate::desktop::simple_db::get_config("claude_oauth_credentials") {
                                                tracing::info!("✅ OAuth credentials found in storage");
                                            } else {
                                                tracing::warn!("⚠️ No OAuth credentials found");
                                                // OAuth authentication should be handled by Claude Code itself
                                                // Not triggering our own OAuth flow
                                            }
                                        } else {
                                            // Check if API key exists
                                            if let Ok(Some(key)) = crate::desktop::simple_db::get_config("anthropic_api_key") {
                                                if !key.is_empty() {
                                                    tracing::info!("✅ API key found in storage ({} chars)", key.len());
                                                } else {
                                                    tracing::warn!("⚠️ API key is empty");
                                                }
                                            } else {
                                                tracing::warn!("⚠️ No API key found in storage");
                                            }
                                        }
                                    }
                                }
                            },
                            match app_state.read().claude_auth_method.as_str() {
                                "ApiKey" => "auth: api key",
                                "OAuth" => "auth: claude pro",
                                "NotSelected" => "auth: select method",
                                _ => "auth: select method"
                            }
                        }
                    }
                }
            }
        }
    }
}

// Removed OAuth trigger - Claude Code handles its own authentication

/// Cycle through Claude Code's native modes (normal -> auto-accept -> plan -> normal)
fn cycle_claude_mode(app_state: &mut Signal<AppState>) {
    let current_mode = app_state.read().claude_mode.clone();
    let next_mode = match current_mode.as_str() {
        "normal" => "auto-accept",
        "auto-accept" => "plan", 
        "plan" => "normal",
        _ => "normal" // Default fallback
    };
    
    // Update the state
    app_state.write().claude_mode = next_mode.to_string();
    
    // Save to database for persistence
    if let Err(e) = crate::desktop::simple_db::save_config("claude_mode", next_mode) {
        tracing::error!("Failed to save Claude mode: {}", e);
    } else {
        tracing::info!("🔄 Claude mode cycled to: {} (Shift+Tab)", next_mode);
    }
    
    // In a real Claude Code integration, we would send this mode change to Claude Code
    // For now, we'll just log it and update our internal state
    match next_mode {
        "normal" => {
            tracing::info!("📝 Claude Code switched to normal mode - standard permission prompts");
        }
        "auto-accept" => {
            tracing::info!("⏵⏵ Claude Code switched to auto-accept mode - immediate execution without prompts");
        }
        "plan" => {
            tracing::info!("📋 Claude Code switched to plan mode - read-only research mode");
        }
        _ => {}
    }
}