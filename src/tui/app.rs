//! Application State Management for Professional TUI

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use std::collections::VecDeque;
use tokio::sync::mpsc;

// use crate::consensus::types::ConsensusStage; // Will be implemented later
use crate::tui::ui::TuiInterface;
use crate::tui::input::InputHandler;
use crate::tui::consensus_view::ConsensusProgress;

pub type TuiResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Main TUI Application State
pub struct TuiApp {
    /// UI rendering interface
    ui: TuiInterface,
    
    /// Input handling
    input_handler: InputHandler,
    
    /// Current message history
    messages: VecDeque<TuiMessage>,
    
    /// Current input buffer
    input_buffer: String,
    
    /// Cursor position in input
    cursor_position: usize,
    
    /// Command history
    command_history: Vec<String>,
    
    /// History navigation index
    history_index: Option<usize>,
    
    /// Current consensus progress
    consensus_progress: Option<ConsensusProgress>,
    
    /// Status line state
    status: TuiStatus,
    
    /// Application settings
    settings: TuiSettings,
    
    /// Event channel for async updates
    event_receiver: mpsc::UnboundedReceiver<TuiEvent>,
    
    /// Event sender for command processing
    event_sender: mpsc::UnboundedSender<TuiEvent>,
    
    /// Whether to exit the application
    should_exit: bool,
}

/// Message displayed in the TUI
#[derive(Clone, Debug)]
pub struct TuiMessage {
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of messages displayed in TUI
#[derive(Clone, Debug)]
pub enum MessageType {
    Welcome,
    UserInput,
    SystemResponse,
    ConsensusProgress,
    Error,
    Status,
    Help,
}

/// Current status line information
#[derive(Clone, Debug)]
pub struct TuiStatus {
    /// Auto-accept edits toggle
    pub auto_accept: bool,
    
    /// Context percentage remaining
    pub context_percentage: u8,
    
    /// Current mode (Planning, Interactive, etc.)
    pub current_mode: String,
    
    /// Current cost estimate
    pub cost_estimate: f64,
}

/// TUI application settings
#[derive(Clone, Debug)]
pub struct TuiSettings {
    /// Theme name
    pub theme: String,
    
    /// Whether banner was shown
    pub banner_shown: bool,
    
    /// Terminal size constraints
    pub min_width: u16,
    pub min_height: u16,
}

/// Events that can be sent to the TUI
#[derive(Clone, Debug)]
pub enum TuiEvent {
    /// Display a message
    Message(TuiMessage),
    
    /// Update consensus progress
    ConsensusUpdate(ConsensusProgress),
    
    /// Consensus completed
    ConsensusComplete,
    
    /// Status update
    StatusUpdate(TuiStatus),
    
    /// Error occurred
    Error(String),
    
    /// System notification
    Notification(String),
}

impl TuiApp {
    /// Create new TUI application
    pub async fn new() -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let ui = TuiInterface::new().await?;
        let input_handler = InputHandler::new();
        
        let mut app = Self {
            ui,
            input_handler,
            messages: VecDeque::new(),
            input_buffer: String::new(),
            cursor_position: 0,
            command_history: Vec::new(),
            history_index: None,
            consensus_progress: None,
            status: TuiStatus {
                auto_accept: true,
                context_percentage: 85,
                current_mode: "Interactive".to_string(),
                cost_estimate: 0.0,
            },
            settings: TuiSettings {
                theme: "dark".to_string(),
                banner_shown: false,
                min_width: 120,
                min_height: 30,
            },
            event_receiver,
            event_sender,
            should_exit: false,
        };
        
        // Add welcome message
        app.add_welcome_message().await?;
        
        Ok(app)
    }
    
    /// Render the TUI interface
    pub fn render(&mut self, frame: &mut Frame) {
        self.ui.draw(
            frame,
            &self.messages,
            &self.input_buffer,
            self.cursor_position,
            &self.consensus_progress,
            &self.status,
            &self.settings,
        );
    }
    
    /// Handle keyboard input
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            // Exit commands
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_exit = true;
                return Ok(true);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_exit = true;
                return Ok(true);
            }
            
            // Process input
            KeyCode::Enter => {
                self.process_input().await?;
            }
            
            // Text input
            KeyCode::Char(c) => {
                self.input_buffer.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            
            // Backspace
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input_buffer.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }
            
            // Cursor movement
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.input_buffer.len() {
                    self.cursor_position += 1;
                }
            }
            
            // Command history
            KeyCode::Up => {
                self.navigate_history_up();
            }
            KeyCode::Down => {
                self.navigate_history_down();
            }
            
            // Function keys for panel focus (VS Code style)
            KeyCode::F(1) => {
                self.status.current_mode = "Input Focus".to_string();
            }
            KeyCode::F(2) => {
                self.status.current_mode = "Explorer Focus".to_string();
            }
            KeyCode::F(3) => {
                self.status.current_mode = "Consensus Focus".to_string();
            }
            KeyCode::F(4) => {
                self.status.current_mode = "Terminal Focus".to_string();
            }
            
            // Quick commands (Ctrl shortcuts)
            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input_buffer = "ask ".to_string();
                self.cursor_position = 4;
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input_buffer = "analyze .".to_string();
                self.cursor_position = 9;
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input_buffer = "plan ".to_string();
                self.cursor_position = 5;
            }
            
            // Toggle auto-accept (Shift+Tab like Claude Code)
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.status.auto_accept = !self.status.auto_accept;
                self.add_message(&format!(
                    "Auto-accept edits: {}", 
                    if self.status.auto_accept { "ON" } else { "OFF" }
                ), MessageType::Status).await?;
            }
            
            // Clear screen (Ctrl+L)
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.messages.clear();
                self.add_welcome_message().await?;
            }
            
            // Help (?)
            KeyCode::Char('?') => {
                self.show_help().await?;
            }
            
            // Clear input (Escape)
            KeyCode::Esc => {
                self.input_buffer.clear();
                self.cursor_position = 0;
                self.history_index = None;
            }
            
            _ => {}
        }
        
        Ok(false)
    }
    
    /// Handle async events from the event channel
    pub async fn handle_async_events(&mut self) -> Result<()> {
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                TuiEvent::Message(message) => {
                    self.messages.push_back(message);
                    self.trim_messages();
                }
                TuiEvent::ConsensusUpdate(progress) => {
                    self.consensus_progress = Some(progress);
                }
                TuiEvent::ConsensusComplete => {
                    self.consensus_progress = None;
                }
                TuiEvent::StatusUpdate(status) => {
                    self.status = status;
                }
                TuiEvent::Error(error) => {
                    self.add_message(&format!("❌ Error: {}", error), MessageType::Error).await?;
                }
                TuiEvent::Notification(notification) => {
                    self.add_message(&notification, MessageType::Status).await?;
                }
            }
        }
        Ok(())
    }
    
    /// Process user input
    async fn process_input(&mut self) -> Result<()> {
        if self.input_buffer.is_empty() {
            return Ok(());
        }
        
        let input = self.input_buffer.clone();
        self.input_buffer.clear();
        self.cursor_position = 0;
        self.history_index = None;
        
        // Add to command history
        self.command_history.push(input.clone());
        if self.command_history.len() > 100 {
            self.command_history.remove(0);
        }
        
        // Add user input to messages
        self.add_message(&format!("> {}", input), MessageType::UserInput).await?;
        
        // Process the command
        self.input_handler.process_command(&input, self.event_sender.clone()).await?;
        
        Ok(())
    }
    
    /// Navigate command history up
    fn navigate_history_up(&mut self) {
        if let Some(idx) = self.history_index {
            if idx > 0 {
                self.history_index = Some(idx - 1);
                self.input_buffer = self.command_history[idx - 1].clone();
                self.cursor_position = self.input_buffer.len();
            }
        } else if !self.command_history.is_empty() {
            let idx = self.command_history.len() - 1;
            self.history_index = Some(idx);
            self.input_buffer = self.command_history[idx].clone();
            self.cursor_position = self.input_buffer.len();
        }
    }
    
    /// Navigate command history down
    fn navigate_history_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx < self.command_history.len() - 1 {
                self.history_index = Some(idx + 1);
                self.input_buffer = self.command_history[idx + 1].clone();
                self.cursor_position = self.input_buffer.len();
            } else {
                self.history_index = None;
                self.input_buffer.clear();
                self.cursor_position = 0;
            }
        }
    }
    
    /// Add welcome message with professional HiveTechs branding
    async fn add_welcome_message(&mut self) -> Result<()> {
        let welcome_content = self.ui.format_welcome_banner().await?;
        self.add_message(&welcome_content, MessageType::Welcome).await?;
        self.settings.banner_shown = true;
        Ok(())
    }
    
    /// Show help information
    async fn show_help(&mut self) -> Result<()> {
        let help_content = "⌨️  HiveTechs Consensus - Keyboard Shortcuts\n\
                           \n\
                           Navigation:\n\
                           ↑/↓                   - Navigate command history\n\
                           ←/→                   - Move cursor in input line\n\
                           \n\
                           Panel Focus (VS Code style):\n\
                           F1                    - Focus Input/Messages\n\
                           F2                    - Focus File Explorer\n\
                           F3                    - Focus Consensus Panel\n\
                           F4                    - Focus Terminal\n\
                           \n\
                           Quick Commands:\n\
                           Ctrl+H                - Quick ask command\n\
                           Ctrl+A                - Quick analyze command\n\
                           Ctrl+P                - Quick plan command\n\
                           \n\
                           Other:\n\
                           Ctrl+L                - Clear screen\n\
                           Shift+Tab             - Toggle auto-accept edits\n\
                           ?                     - Show this help\n\
                           Esc                   - Clear input\n\
                           Ctrl+C/Ctrl+D        - Exit application";
        
        self.add_message(help_content, MessageType::Help).await?;
        Ok(())
    }
    
    /// Add a message to the display
    async fn add_message(&mut self, content: &str, message_type: MessageType) -> Result<()> {
        let message = TuiMessage {
            content: content.to_string(),
            message_type,
            timestamp: chrono::Utc::now(),
        };
        
        self.messages.push_back(message);
        self.trim_messages();
        Ok(())
    }
    
    /// Trim messages to keep memory usage reasonable
    fn trim_messages(&mut self) {
        const MAX_MESSAGES: usize = 1000;
        while self.messages.len() > MAX_MESSAGES {
            self.messages.pop_front();
        }
    }
    
    /// Get event sender for external command processing
    pub fn event_sender(&self) -> mpsc::UnboundedSender<TuiEvent> {
        self.event_sender.clone()
    }
}