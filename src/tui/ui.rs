//! UI Components and Rendering for Professional TUI Interface

use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, StatefulWidget, Wrap},
    Frame,
};
use std::collections::VecDeque;

use crate::tui::app::{MessageType, TuiMessage, TuiSettings, TuiStatus};
use crate::tui::banner::WelcomeBanner;
use crate::tui::consensus_types::SimpleConsensusProgress;
use crate::tui::consensus_view::{ConsensusProgress, ConsensusView};
use crate::tui::formatting::{ContentFormatter, ContentType};
use crate::tui::input::InputBox;
use crate::tui::status_line::StatusLine;

/// Main UI interface for the TUI
pub struct TuiInterface {
    banner: WelcomeBanner,
    input_box: InputBox,
    consensus_view: ConsensusView,
    status_line: StatusLine,
    /// Scroll offset for messages (newest at bottom)
    scroll_offset: usize,
    /// Maximum visible messages
    max_visible_messages: usize,
    /// Whether user is manually scrolling (prevents auto-scroll)
    manual_scroll_active: bool,
}

impl TuiInterface {
    /// Create new TUI interface
    pub async fn new() -> Result<Self> {
        Ok(Self {
            banner: WelcomeBanner::new().await?,
            input_box: InputBox::new(),
            consensus_view: ConsensusView::new(),
            status_line: StatusLine::new(),
            scroll_offset: 0,
            max_visible_messages: 100,
            manual_scroll_active: false,
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

        // Calculate stable layout with bounded constraints to prevent input box displacement
        let terminal_height = size.height;
        let input_height = 3u16;
        let status_height = 1u16;
        let consensus_height = if consensus_progress.is_some() {
            6u16
        } else {
            0u16
        };

        // Calculate maximum height for messages area to ensure input stays visible
        let reserved_height = input_height + status_height + consensus_height;
        let max_messages_height = terminal_height.saturating_sub(reserved_height).max(5);

        // Create stable layout - always reserve space for input box
        let main_constraints = if consensus_progress.is_some() {
            vec![
                Constraint::Max(max_messages_height), // Messages area (bounded to prevent expansion)
                Constraint::Length(consensus_height), // Consensus progress (visible)
                Constraint::Length(input_height),     // Input box (always fixed at bottom)
                Constraint::Length(status_height),    // Status line (always at bottom)
            ]
        } else {
            vec![
                Constraint::Max(max_messages_height), // Messages area (bounded)
                Constraint::Length(input_height),     // Input box (always fixed at bottom)
                Constraint::Length(status_height),    // Status line (always at bottom)
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(main_constraints)
            .split(size);

        // Draw messages area
        self.draw_messages_with_consensus(frame, chunks[0], messages, consensus_progress);

        // Draw consensus progress if active (in separate panel for clarity)
        if consensus_progress.is_some() {
            if let Some(progress) = consensus_progress {
                self.draw_consensus_panel(frame, chunks[1], progress);
            }
            // Input box is at index 2 when consensus is visible
            self.input_box
                .draw(frame, chunks[2], input_buffer, cursor_position);
            // Status line at index 3
            self.status_line.draw(frame, chunks[3], status);
        } else {
            // Input box is at index 1 when consensus is hidden
            self.input_box
                .draw(frame, chunks[1], input_buffer, cursor_position);
            // Status line at index 2
            self.status_line.draw(frame, chunks[2], status);
        }
    }

    /// Draw messages area with proper scrolling (no inline consensus)
    fn draw_messages_with_consensus(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        messages: &VecDeque<TuiMessage>,
        consensus_progress: &Option<ConsensusProgress>,
    ) {
        // Calculate visible area (subtract 2 for borders)
        let visible_height = area.height.saturating_sub(2) as usize;

        // Collect all display items from messages only
        let mut display_items: Vec<(String, Style)> = Vec::new();

        // Add messages
        for msg in messages.iter() {
            let style = self.get_message_style(&msg.message_type);
            let content = self.format_message_content(msg);

            // Split multi-line messages
            for line in content.lines() {
                display_items.push((line.to_string(), style));
            }
        }

        // Calculate scroll offset with proper bounds
        let total_items = display_items.len();
        let scroll_offset = if self.manual_scroll_active {
            // Respect manual scroll position, but ensure it's still valid
            let max_offset = total_items.saturating_sub(visible_height);
            self.scroll_offset.min(max_offset)
        } else {
            // Auto-scroll to bottom to show latest messages
            if total_items > visible_height {
                total_items - visible_height
            } else {
                0
            }
        };

        // Create list items with proper scrolling and enhanced formatting
        let items: Vec<ListItem> = display_items
            .iter()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(content, style)| {
                // Check if this is a formatted result that needs special rendering
                if content.contains("EXECUTIVE SUMMARY")
                    || content.contains("PERFORMANCE METRICS")
                    || content.contains("Consensus Journey")
                    || content.contains("╔")
                {
                    let content_type = ContentType::detect(content);
                    let formatted_text =
                        ContentFormatter::format_with_highlighting(content, content_type);
                    ListItem::new(formatted_text)
                } else {
                    ListItem::new(Text::from(content.as_str())).style(*style)
                }
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 📝 Messages ")
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .style(Style::default());

        frame.render_widget(list, area);

        // Update scroll offset for next frame only if not manually scrolling
        if !self.manual_scroll_active {
            self.scroll_offset = scroll_offset;
        }
    }

    /// Get style for message type
    fn get_message_style(&self, message_type: &MessageType) -> Style {
        match message_type {
            MessageType::Welcome => Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            MessageType::UserInput => Style::default().fg(Color::Green),
            MessageType::SystemResponse => Style::default().fg(Color::White),
            MessageType::ConsensusProgress => Style::default().fg(Color::Yellow),
            MessageType::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            MessageType::Status => Style::default().fg(Color::Magenta),
            MessageType::Help => Style::default().fg(Color::Blue),
            MessageType::Info => Style::default().fg(Color::Cyan),
            MessageType::FormattedResult => Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Format message content with timestamp
    fn format_message_content(&self, msg: &TuiMessage) -> String {
        let timestamp = msg.timestamp.format("%H:%M:%S");

        match msg.message_type {
            MessageType::Welcome => msg.content.clone(),
            MessageType::UserInput => msg.content.clone(),
            MessageType::FormattedResult => {
                // No timestamp for formatted results - they have their own formatting
                format!("\n{}\n", msg.content)
            }
            _ => format!("[{}] {}", timestamp, msg.content),
        }
    }

    /// Format the professional welcome banner
    pub async fn format_welcome_banner(&self) -> Result<String> {
        self.banner.format_banner().await
    }

    /// Handle scroll up (for keyboard navigation)
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        self.manual_scroll_active = true; // User is manually scrolling
    }

    /// Handle scroll down (for keyboard navigation)
    pub fn scroll_down(&mut self, amount: usize, total_messages: usize) {
        let max_offset = total_messages.saturating_sub(self.max_visible_messages);
        let new_offset = (self.scroll_offset + amount).min(max_offset);

        // If user scrolled to bottom, disable manual scroll mode
        if new_offset == max_offset {
            self.manual_scroll_active = false;
        } else {
            self.manual_scroll_active = true;
        }

        self.scroll_offset = new_offset;
    }

    /// Draw separate consensus progress panel for better visual separation
    fn draw_consensus_panel(&self, frame: &mut Frame, area: Rect, progress: &ConsensusProgress) {
        use crate::tui::consensus_types::SimpleConsensusProgress;

        let simple = SimpleConsensusProgress::from_detailed(progress);
        let (progress_text, progress_color) = match simple {
            SimpleConsensusProgress::Thinking => ("🤔 Thinking...".to_string(), Color::Yellow),
            SimpleConsensusProgress::Generating(pct) => {
                (format!("⚡ Generating... {}%", pct), Color::Green)
            }
            SimpleConsensusProgress::Refining(pct) => {
                (format!("✨ Refining... {}%", pct), Color::Blue)
            }
            SimpleConsensusProgress::Validating(pct) => {
                (format!("🔍 Validating... {}%", pct), Color::Magenta)
            }
            SimpleConsensusProgress::Curating(pct) => {
                (format!("📝 Curating... {}%", pct), Color::Cyan)
            }
            SimpleConsensusProgress::Complete => ("✅ Complete".to_string(), Color::Green),
            SimpleConsensusProgress::Error(ref err) => (format!("❌ Error: {}", err), Color::Red),
        };

        let consensus_paragraph = Paragraph::new(progress_text)
            .style(
                Style::default()
                    .fg(progress_color)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🧠 4-Stage Consensus Pipeline ")
                    .border_style(Style::default().fg(Color::Yellow))
                    .title_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(consensus_paragraph, area);
    }

    /// Reset scroll to bottom (show latest messages) - only if not manually scrolling
    pub fn scroll_to_bottom(&mut self, total_messages: usize) {
        if !self.manual_scroll_active {
            let max_offset = total_messages.saturating_sub(self.max_visible_messages);
            self.scroll_offset = max_offset;
        }
    }
}
