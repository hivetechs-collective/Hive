//! UI Components and Rendering for Professional TUI Interface

use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::collections::VecDeque;

use crate::tui::app::{TuiMessage, MessageType, TuiStatus, TuiSettings};
use crate::tui::banner::WelcomeBanner;
use crate::tui::input::InputBox;
use crate::tui::consensus_view::{ConsensusProgress, ConsensusView};
use crate::tui::status_line::StatusLine;

/// Main UI interface for the TUI
pub struct TuiInterface {
    banner: WelcomeBanner,
    input_box: InputBox,
    consensus_view: ConsensusView,
    status_line: StatusLine,
}

impl TuiInterface {
    /// Create new TUI interface
    pub async fn new() -> Result<Self> {
        Ok(Self {
            banner: WelcomeBanner::new().await?,
            input_box: InputBox::new(),
            consensus_view: ConsensusView::new(),
            status_line: StatusLine::new(),
        })
    }
    
    /// Draw the complete TUI interface
    pub fn draw(
        &mut self,
        frame: &mut Frame,
        messages: &VecDeque<TuiMessage>,
        input_buffer: &str,
        cursor_position: usize,
        consensus_progress: &Option<ConsensusProgress>,
        status: &TuiStatus,
        settings: &TuiSettings,
    ) {
        let size = frame.size();
        
        // Create main layout
        let chunks = if consensus_progress.is_some() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(10),     // Messages area
                    Constraint::Length(6),   // Consensus progress
                    Constraint::Length(3),   // Input box
                    Constraint::Length(1),   // Status line
                ])
                .split(size)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(10),     // Messages area
                    Constraint::Length(3),   // Input box
                    Constraint::Length(1),   // Status line
                ])
                .split(size)
        };
        
        // Draw messages area
        self.draw_messages(frame, chunks[0], messages);
        
        if consensus_progress.is_some() {
            // Draw consensus progress
            self.consensus_view.draw(frame, chunks[1], consensus_progress.as_ref().unwrap());
            
            // Draw input box
            self.input_box.draw(frame, chunks[2], input_buffer, cursor_position);
            
            // Draw status line
            self.status_line.draw(frame, chunks[3], status);
        } else {
            // Draw input box
            self.input_box.draw(frame, chunks[1], input_buffer, cursor_position);
            
            // Draw status line
            self.status_line.draw(frame, chunks[2], status);
        }
    }
    
    /// Draw the messages area
    fn draw_messages(&self, frame: &mut Frame, area: Rect, messages: &VecDeque<TuiMessage>) {
        let items: Vec<ListItem> = messages
            .iter()
            .map(|msg| {
                let style = self.get_message_style(&msg.message_type);
                let content = self.format_message_content(msg);
                ListItem::new(Text::from(content)).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default());
        
        frame.render_widget(list, area);
    }
    
    /// Get style for message type
    fn get_message_style(&self, message_type: &MessageType) -> Style {
        match message_type {
            MessageType::Welcome => Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            MessageType::UserInput => Style::default()
                .fg(Color::Green),
            MessageType::SystemResponse => Style::default()
                .fg(Color::White),
            MessageType::ConsensusProgress => Style::default()
                .fg(Color::Yellow),
            MessageType::Error => Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            MessageType::Status => Style::default()
                .fg(Color::Magenta),
            MessageType::Help => Style::default()
                .fg(Color::Blue),
        }
    }
    
    /// Format message content with timestamp
    fn format_message_content(&self, msg: &TuiMessage) -> String {
        let timestamp = msg.timestamp.format("%H:%M:%S");
        
        match msg.message_type {
            MessageType::Welcome => msg.content.clone(),
            MessageType::UserInput => msg.content.clone(),
            _ => format!("[{}] {}", timestamp, msg.content),
        }
    }
    
    /// Format the professional welcome banner
    pub async fn format_welcome_banner(&self) -> Result<String> {
        self.banner.format_banner().await
    }
}