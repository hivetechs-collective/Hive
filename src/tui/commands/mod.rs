//! TUI Command System - Professional terminal interface commands
//!
//! This module provides:
//! - Complete command dispatch system with real-time streaming
//! - Special command handlers (/help, /status, /exit)
//! - Keyboard shortcuts and command history integration
//! - Progressive enhancement for different terminal capabilities
//! - Professional VS Code-style command interface

pub mod handlers;
pub mod special;
pub mod history;
pub mod shortcuts;
pub mod streaming;
pub mod dispatch;

pub use handlers::{CommandHandler, CommandResult, CommandContext};
pub use special::{SpecialCommands, SpecialCommandHandler};
pub use history::{CommandHistory, HistorySearcher};
pub use shortcuts::{KeyboardShortcuts, ShortcutHandler};
pub use streaming::{StreamingCommandHandler, StreamingResult};
pub use dispatch::{CommandDispatcher, DispatchResult};

use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::tui::app::{TuiEvent, TuiMessage, MessageType};

/// Core TUI command system
pub struct TuiCommandSystem {
    /// Command dispatcher
    dispatcher: CommandDispatcher,

    /// Special command handler
    special_handler: SpecialCommandHandler,

    /// Command history manager
    history: CommandHistory,

    /// Keyboard shortcuts handler
    shortcuts: ShortcutHandler,

    /// Streaming command handler
    streaming_handler: StreamingCommandHandler,

    /// Event sender for UI updates
    event_sender: mpsc::UnboundedSender<TuiEvent>,
}

impl TuiCommandSystem {
    /// Create new TUI command system
    pub async fn new(event_sender: mpsc::UnboundedSender<TuiEvent>) -> Result<Self> {
        let dispatcher = CommandDispatcher::new().await?;
        let special_handler = SpecialCommandHandler::new();
        let history = CommandHistory::new().await?;
        let shortcuts = ShortcutHandler::new();
        let streaming_handler = StreamingCommandHandler::new(event_sender.clone()).await?;

        Ok(Self {
            dispatcher,
            special_handler,
            history,
            shortcuts,
            streaming_handler,
            event_sender,
        })
    }

    /// Process a command input
    pub async fn process_command(&mut self, input: &str) -> Result<()> {
        // Add to history first
        self.history.add_command(input.to_string()).await?;

        // Check for special commands first
        if input.starts_with('/') {
            return self.handle_special_command(input).await;
        }

        // Dispatch regular command
        self.dispatch_regular_command(input).await
    }

    /// Handle special commands (/help, /status, etc.)
    async fn handle_special_command(&mut self, input: &str) -> Result<()> {
        let result = self.special_handler.handle(input).await?;

        // Send result to UI
        let message = TuiMessage {
            content: result.output,
            message_type: MessageType::SystemResponse,
            timestamp: chrono::Utc::now(),
        };

        self.event_sender.send(TuiEvent::Message(message))?;

        Ok(())
    }

    /// Dispatch regular commands with streaming support
    async fn dispatch_regular_command(&mut self, input: &str) -> Result<()> {
        // Parse command and arguments
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let command = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        // Check if this is a streaming command
        if self.is_streaming_command(command) {
            self.streaming_handler.handle_streaming_command(command, args).await?;
        } else {
            self.dispatcher.dispatch(command, args).await?;
        }

        Ok(())
    }

    /// Check if command supports streaming
    fn is_streaming_command(&self, command: &str) -> bool {
        matches!(command, "ask" | "consensus" | "explain" | "analyze" | "improve" | "plan")
    }

    /// Get command suggestions for auto-completion
    pub async fn get_suggestions(&self, partial_input: &str) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();

        // Add command suggestions
        suggestions.extend(self.dispatcher.get_command_suggestions(partial_input).await?);

        // Add special command suggestions
        suggestions.extend(self.special_handler.get_suggestions(partial_input));

        // Add history-based suggestions
        suggestions.extend(self.history.get_suggestions(partial_input).await?);

        Ok(suggestions)
    }

    /// Search command history
    pub async fn search_history(&self, query: &str) -> Result<Vec<String>> {
        self.history.search(query).await
    }

    /// Get keyboard shortcut help
    pub fn get_shortcut_help(&self) -> String {
        self.shortcuts.get_help_text()
    }
}

/// Available core commands that integrate with TUI
pub const CORE_COMMANDS: &[&str] = &[
    "ask",           // Consensus question
    "consensus",     // Direct consensus call
    "explain",       // Code explanation
    "analyze",       // Code analysis
    "review",        // Code review
    "improve",       // Code improvement
    "plan",          // Planning command
    "decompose",     // Task decomposition
    "timeline",      // Timeline generation
    "search",        // Memory search
    "similar",       // Similar conversations
    "insights",      // Memory insights
    "overview",      // Analytics overview
    "trends",        // Trend analysis
    "cost",          // Cost estimation
    "models",        // Model management
    "test",          // Model testing
    "recommend",     // Model recommendations
    "apply",         // Apply transformations
    "preview",       // Preview changes
    "transform",     // Code transformation
];

/// Special commands available in TUI
pub const SPECIAL_COMMANDS: &[&str] = &[
    "/help",         // Show help
    "/status",       // System status
    "/exit",         // Exit application
    "/clear",        // Clear screen
    "/history",      // Command history
    "/mode",         // Mode switching
    "/settings",     // Configuration
    "/shortcuts",    // Keyboard shortcuts
    "/about",        // About information
];