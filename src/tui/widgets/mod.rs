//! Custom Widgets for Professional TUI Interface

pub mod help_popup;
pub mod input_field;
pub mod message_list;
pub mod progress_bar;

pub use help_popup::HelpPopup;
pub use input_field::InputField;
pub use message_list::MessageList;
pub use progress_bar::ConsensusProgressBar;

use ratatui::{
    style::{Color, Style},
    widgets::Widget,
};

/// Common styling utilities for widgets
pub struct WidgetTheme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub text: Color,
    pub background: Color,
}

impl Default for WidgetTheme {
    fn default() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::White,
            background: Color::Black,
        }
    }
}

impl WidgetTheme {
    /// Get style for primary elements
    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    /// Get style for secondary elements
    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    /// Get style for accent elements
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Get style for success messages
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Get style for warning messages
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Get style for error messages
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Get style for regular text
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text).bg(self.background)
    }
}
