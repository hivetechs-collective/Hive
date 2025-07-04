//! Inline styles for modern Claude Code-like appearance

use dioxus::prelude::*;

/// Get inline styles for the chat interface
pub fn get_chat_styles() -> String {
    r#"
    /* Chat Container */
    .chat-container {
        display: flex;
        flex-direction: column;
        height: 100%;
        position: relative;
        background-color: #1e1e1e;
        color: #e1e1e1;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        flex: 1;
    }

    /* Chat Messages Area */
    .chat-messages {
        flex: 1;
        overflow-y: auto;
        padding: 20px;
        padding-bottom: 100px;
    }

    /* Welcome Container */
    .welcome-container {
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 100%;
        padding: 40px;
    }

    .welcome-content {
        max-width: 600px;
        text-align: center;
    }

    .welcome-title {
        font-size: 32px;
        font-weight: 600;
        margin: 0 0 8px 0;
        color: #ffffff;
    }

    .welcome-subtitle {
        font-size: 16px;
        color: #a0a0a0;
        margin: 0 0 40px 0;
    }

    /* System Status Grid */
    .system-status-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 16px;
        margin-bottom: 40px;
    }

    .status-card {
        background-color: #2a2a2a;
        border-radius: 8px;
        padding: 16px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
    }

    .status-icon {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 14px;
    }

    .status-icon.success {
        background-color: #2e7d32;
        color: #ffffff;
    }

    .status-text {
        font-size: 14px;
        color: #e1e1e1;
    }

    /* Commands Section */
    .commands-section {
        text-align: left;
    }

    .commands-title {
        font-size: 18px;
        font-weight: 500;
        margin: 0 0 16px 0;
        color: #ffffff;
    }

    .command-list {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .command-item {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .command-code {
        background-color: #2a2a2a;
        padding: 4px 8px;
        border-radius: 4px;
        font-family: "SF Mono", Monaco, "Cascadia Code", monospace;
        font-size: 13px;
        color: #61afef;
        white-space: nowrap;
    }

    .command-desc {
        font-size: 14px;
        color: #a0a0a0;
    }

    /* Message Styles */
    .message-row {
        display: flex;
        margin-bottom: 16px;
    }

    .message-row.user {
        justify-content: flex-end;
    }

    .message-row.assistant,
    .message-row.system,
    .message-row.error {
        justify-content: flex-start;
    }

    .message-bubble {
        max-width: 70%;
        padding: 12px 16px;
        border-radius: 12px;
        position: relative;
    }

    .message-row.user .message-bubble {
        background-color: #3c3c3c;
        color: #ffffff;
    }

    .message-row.assistant .message-bubble {
        background-color: #2a2a2a;
        color: #e1e1e1;
    }

    .message-row.system .message-bubble {
        background-color: #1a3a52;
        color: #a0c4e4;
    }

    .message-row.error .message-bubble {
        background-color: #4a1e1e;
        color: #ff8a8a;
    }

    .message-header {
        font-size: 12px;
        font-weight: 600;
        margin-bottom: 4px;
        opacity: 0.8;
    }

    .message-content {
        font-size: 14px;
        line-height: 1.5;
        word-wrap: break-word;
    }

    .message-content code {
        background-color: rgba(255, 255, 255, 0.1);
        padding: 2px 4px;
        border-radius: 3px;
        font-family: "SF Mono", Monaco, "Cascadia Code", monospace;
        font-size: 13px;
    }

    .message-content pre {
        background-color: rgba(0, 0, 0, 0.3);
        padding: 12px;
        border-radius: 6px;
        overflow-x: auto;
        margin: 8px 0;
    }

    .message-content pre code {
        background-color: transparent;
        padding: 0;
    }

    .message-timestamp {
        font-size: 11px;
        opacity: 0.6;
        margin-top: 4px;
    }

    /* Chat Input */
    .chat-input-container {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        padding: 16px 20px;
        background-color: #1e1e1e;
        border-top: 1px solid #2a2a2a;
    }

    .input-wrapper {
        display: flex;
        align-items: flex-end;
        gap: 8px;
        background-color: #3c3c3c;
        border-radius: 8px;
        padding: 8px 8px 8px 16px;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    }

    .chat-input {
        flex: 1;
        background: transparent;
        border: none;
        outline: none;
        color: #ffffff;
        font-size: 14px;
        font-family: inherit;
        resize: none;
        min-height: 24px;
        max-height: 120px;
        line-height: 1.5;
    }

    .chat-input::placeholder {
        color: #808080;
    }

    .send-btn {
        width: 32px;
        height: 32px;
        border-radius: 6px;
        border: none;
        background-color: #007acc;
        color: #ffffff;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s ease;
    }

    .send-btn:hover:not(.disabled) {
        background-color: #0098ff;
    }

    .send-btn.disabled {
        background-color: #404040;
        cursor: not-allowed;
        opacity: 0.5;
    }

    .input-shortcuts {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-top: 8px;
        padding: 0 4px;
    }

    .shortcut-hint {
        font-size: 12px;
        color: #808080;
    }

    .shortcut-divider {
        color: #404040;
    }

    /* Scrollbar Styling */
    .chat-messages::-webkit-scrollbar {
        width: 8px;
    }

    .chat-messages::-webkit-scrollbar-track {
        background: transparent;
    }

    .chat-messages::-webkit-scrollbar-thumb {
        background-color: #404040;
        border-radius: 4px;
    }

    .chat-messages::-webkit-scrollbar-thumb:hover {
        background-color: #505050;
    }
    "#.to_string()
}

/// Get app-wide global styles
pub fn get_app_styles() -> String {
    r#"
    /* Reset and base styles */
    * {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }

    html, body {
        height: 100%;
        overflow: hidden;
    }

    body {
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
        background-color: #1e1e1e;
        color: #e1e1e1;
        line-height: 1.5;
    }

    /* App Container */
    .app-container {
        display: flex;
        flex-direction: column;
        height: 100vh;
        background-color: #1e1e1e;
    }

    /* Main Layout */
    .main-layout {
        display: flex;
        flex: 1;
        overflow: hidden;
    }

    /* Status Bar */
    .status-bar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        height: 22px;
        background-color: #007acc;
        color: #ffffff;
        padding: 0 10px;
        font-size: 12px;
    }

    .status-left,
    .status-right {
        display: flex;
        align-items: center;
        gap: 16px;
    }

    .status-item {
        display: flex;
        align-items: center;
        gap: 4px;
    }

    .status-indicator {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        margin-right: 4px;
    }

    .status-indicator.connected {
        background-color: #4ec9b0;
    }

    .status-indicator.connecting {
        background-color: #f9c74f;
        animation: pulse 1.5s ease-in-out infinite;
    }

    .status-indicator.disconnected {
        background-color: #f44747;
    }

    @keyframes pulse {
        0% { opacity: 1; }
        50% { opacity: 0.5; }
        100% { opacity: 1; }
    }

    /* Button Base Styles */
    button {
        font-family: inherit;
        font-size: inherit;
        cursor: pointer;
    }

    /* Remove default button styles */
    button:focus {
        outline: none;
    }

    /* Selection color */
    ::selection {
        background-color: #264f78;
        color: #ffffff;
    }

    /* Auto-accept toggle */
    .auto-accept-toggle {
        padding: 2px 8px;
        border-radius: 3px;
        cursor: pointer;
        transition: all 0.2s ease;
        user-select: none;
    }

    .auto-accept-toggle.enabled {
        background-color: rgba(77, 187, 95, 0.3);
        color: #4ec9b0;
    }

    .auto-accept-toggle.disabled {
        background-color: rgba(255, 255, 255, 0.1);
        color: #cccccc;
    }

    .auto-accept-toggle:hover {
        background-color: rgba(255, 255, 255, 0.2);
    }
    "#.to_string()
}