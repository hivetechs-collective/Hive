//! Enhanced Message List Widget for TUI Display

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Widget},
};

use crate::tui::app::{TuiMessage, MessageType};
use super::WidgetTheme;

/// Enhanced message list for TUI display
pub struct MessageList<'a> {
    /// Messages to display
    messages: &'a [TuiMessage],
    
    /// Block around the list
    block: Option<Block<'a>>,
    
    /// Default style
    style: Style,
    
    /// Widget theme
    theme: &'a WidgetTheme,
    
    /// Show timestamps
    show_timestamps: bool,
    
    /// Scroll offset
    scroll_offset: usize,
}

impl<'a> MessageList<'a> {
    /// Create new message list
    pub fn new(messages: &'a [TuiMessage], theme: &'a WidgetTheme) -> Self {
        Self {
            messages,
            block: None,
            style: theme.text_style(),
            theme,
            show_timestamps: true,
            scroll_offset: 0,
        }
    }
    
    /// Set block around the list
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Set overall style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    
    /// Show/hide timestamps
    pub fn show_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }
    
    /// Set scroll offset
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }
    
    
}

impl<'a> Widget for MessageList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Extract fields before the block match to avoid partial move
        let messages = self.messages;
        let scroll_offset = self.scroll_offset;
        let style = self.style;
        let theme = self.theme;
        let show_timestamps = self.show_timestamps;
        
        let area = match self.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        
        // Convert messages to list items
        let items: Vec<ListItem> = messages
            .iter()
            .skip(scroll_offset)
            .map(|msg| ListItem::new(format_message(msg, theme, show_timestamps)))
            .collect();
        
        // Create and render the list
        let list = List::new(items).style(style);
        list.render(area, buf);
    }
}

/// Format message content with optional timestamp
fn format_message<'a>(message: &'a TuiMessage, theme: &WidgetTheme, show_timestamps: bool) -> Text<'a> {
    let style = get_message_style(&message.message_type, theme);
    
    if show_timestamps && !matches!(message.message_type, MessageType::Welcome | MessageType::UserInput) {
        let timestamp = message.timestamp.format("%H:%M:%S");
        let timestamp_style = Style::default().fg(Color::DarkGray);
        
        Text::from(vec![
            Line::from(vec![
                Span::styled(format!("[{}] ", timestamp), timestamp_style),
                Span::styled(&message.content, style),
            ]),
        ])
    } else {
        Text::from(Span::styled(&message.content, style))
    }
}

/// Get style for message type
fn get_message_style(message_type: &MessageType, theme: &WidgetTheme) -> Style {
    match message_type {
        MessageType::Welcome => theme.secondary_style().add_modifier(Modifier::BOLD),
        MessageType::UserInput => theme.primary_style(),
        MessageType::SystemResponse => theme.text_style(),
        MessageType::ConsensusProgress => theme.accent_style(),
        MessageType::Error => theme.error_style().add_modifier(Modifier::BOLD),
        MessageType::Status => theme.warning_style(),
        MessageType::Help => theme.secondary_style(),
        MessageType::Info => theme.accent_style(),
        MessageType::FormattedResult => theme.text_style(),
    }
}

/// Message list utilities
impl<'a> MessageList<'a> {
    /// Create message list with auto-scroll
    pub fn with_auto_scroll(
        messages: &'a [TuiMessage],
        theme: &'a WidgetTheme,
        visible_lines: usize,
    ) -> Self {
        let scroll_offset = if messages.len() > visible_lines {
            messages.len() - visible_lines
        } else {
            0
        };
        
        Self::new(messages, theme).scroll_offset(scroll_offset)
    }
    
    /// Create message list for specific message types
    pub fn filtered(
        messages: &'a [TuiMessage],
        theme: &'a WidgetTheme,
        filter: fn(&MessageType) -> bool,
    ) -> Self {
        // Note: This is a simplified implementation
        // In practice, you'd want to filter the messages first
        Self::new(messages, theme)
    }
    
    /// Create compact message list (no timestamps)
    pub fn compact(messages: &'a [TuiMessage], theme: &'a WidgetTheme) -> Self {
        Self::new(messages, theme).show_timestamps(false)
    }
}