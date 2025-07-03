//! Status Line Component with Auto-Accept Toggle and Context Information

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::TuiStatus;

/// Professional status line component
pub struct StatusLine {
    theme: StatusTheme,
}

/// Status line visual theme
struct StatusTheme {
    background_color: Color,
    text_color: Color,
    highlight_color: Color,
    warning_color: Color,
}

impl Default for StatusTheme {
    fn default() -> Self {
        Self {
            background_color: Color::DarkGray,
            text_color: Color::White,
            highlight_color: Color::Cyan,
            warning_color: Color::Yellow,
        }
    }
}

impl StatusLine {
    /// Create new status line
    pub fn new() -> Self {
        Self {
            theme: StatusTheme::default(),
        }
    }
    
    /// Draw the professional status line
    pub fn draw(&self, frame: &mut Frame, area: Rect, status: &TuiStatus) {
        let status_text = self.format_status_text(status);
        
        let paragraph = Paragraph::new(status_text)
            .style(Style::default()
                .bg(self.theme.background_color)
                .fg(self.theme.text_color))
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center);
        
        frame.render_widget(paragraph, area);
    }
    
    /// Format the status line text with all information
    fn format_status_text(&self, status: &TuiStatus) -> String {
        // Auto-accept status
        let auto_accept_text = if status.auto_accept {
            "⏵⏵ auto-accept: ON"
        } else {
            "⏸⏸ auto-accept: OFF"
        };
        
        // Context information
        let context_text = format!(
            "Context: {}%",
            status.context_percentage
        );
        
        // Current mode
        let mode_text = format!(
            "Mode: {}",
            status.current_mode
        );
        
        // Cost information
        let cost_text = if status.cost_estimate > 0.0 {
            format!("Cost: ${:.3}", status.cost_estimate)
        } else {
            "Cost: $0.000".to_string()
        };
        
        // Keyboard shortcuts hint
        let shortcuts_hint = "F1-F4: Panels | Shift+Tab: Toggle | ?: Help | Ctrl+C: Exit";
        
        // Combine all status information
        format!(
            "{}  |  {}  |  {}  |  {}  |  {}",
            auto_accept_text,
            context_text,
            mode_text,
            cost_text,
            shortcuts_hint
        )
    }
    
    /// Get warning color for low context
    fn get_context_color(&self, percentage: u8) -> Color {
        if percentage < 20 {
            self.theme.warning_color
        } else {
            self.theme.text_color
        }
    }
}

/// Status line information display utilities
impl StatusLine {
    /// Format auto-accept status with visual indicator
    pub fn format_auto_accept(enabled: bool) -> String {
        if enabled {
            "⏵⏵ auto-accept: ON".to_string()
        } else {
            "⏸⏸ auto-accept: OFF".to_string()
        }
    }
    
    /// Format context percentage with warning if low
    pub fn format_context_percentage(percentage: u8) -> String {
        let warning = if percentage < 20 { " ⚠️" } else { "" };
        format!("Context: {}%{}", percentage, warning)
    }
    
    /// Format cost with appropriate precision
    pub fn format_cost(cost: f64) -> String {
        if cost < 0.001 {
            "Cost: $0.000".to_string()
        } else if cost < 1.0 {
            format!("Cost: ${:.3}", cost)
        } else {
            format!("Cost: ${:.2}", cost)
        }
    }
    
    /// Format current mode with appropriate styling
    pub fn format_mode(mode: &str) -> String {
        format!("Mode: {}", mode)
    }
    
    /// Get full shortcuts help text
    pub fn get_shortcuts_help() -> String {
        "F1-F4: Panels | Shift+Tab: Toggle | ?: Help | Ctrl+C: Exit".to_string()
    }
}