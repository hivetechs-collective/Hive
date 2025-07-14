//! Enhanced Input Field Widget for Professional TUI

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
};

use super::WidgetTheme;

/// Enhanced input field widget
pub struct InputField<'a> {
    /// Input content
    content: &'a str,

    /// Cursor position
    cursor_position: usize,

    /// Placeholder text
    placeholder: Option<&'a str>,

    /// Block around the input
    block: Option<Block<'a>>,

    /// Input style
    style: Style,

    /// Placeholder style
    placeholder_style: Style,

    /// Cursor style
    cursor_style: Style,

    /// Widget theme
    theme: &'a WidgetTheme,

    /// Show cursor
    show_cursor: bool,

    /// Input is focused
    focused: bool,
}

impl<'a> InputField<'a> {
    /// Create new input field
    pub fn new(content: &'a str, theme: &'a WidgetTheme) -> Self {
        Self {
            content,
            cursor_position: content.len(),
            placeholder: None,
            block: None,
            style: theme.text_style(),
            placeholder_style: Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            cursor_style: theme.accent_style().add_modifier(Modifier::REVERSED),
            theme,
            show_cursor: true,
            focused: true,
        }
    }

    /// Set cursor position
    pub fn cursor_position(mut self, position: usize) -> Self {
        self.cursor_position = position.min(self.content.len());
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// Set block around input
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set input style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set placeholder style
    pub fn placeholder_style(mut self, style: Style) -> Self {
        self.placeholder_style = style;
        self
    }

    /// Set cursor style
    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }

    /// Show/hide cursor
    pub fn show_cursor(mut self, show: bool) -> Self {
        self.show_cursor = show;
        self
    }

    /// Set focus state
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Create professional input field with Claude Code styling
    pub fn professional(content: &'a str, cursor_position: usize, theme: &'a WidgetTheme) -> Self {
        Self::new(content, theme)
            .cursor_position(cursor_position)
            .placeholder("Try \"ask <question>\" or \"analyze .\" or \"plan <goal>\"")
            .style(theme.text_style())
            .placeholder_style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )
            .cursor_style(theme.accent_style().add_modifier(Modifier::REVERSED))
            .focused(true)
    }
}

impl<'a> Widget for InputField<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = match self.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if area.height < 1 {
            return;
        }

        // Determine what to display
        let (display_text, text_style) = if self.content.is_empty() {
            if let Some(placeholder) = self.placeholder {
                (placeholder, self.placeholder_style)
            } else {
                ("", self.style)
            }
        } else {
            (self.content, self.style)
        };

        // Add prompt prefix
        let full_text = format!("> {}", display_text);

        // Create paragraph
        let paragraph = Paragraph::new(full_text)
            .style(text_style)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);

        // Render cursor if needed
        if self.show_cursor && self.focused && !self.content.is_empty() {
            let cursor_x = area.x + self.cursor_position as u16 + 2; // Account for "> " prefix
            let cursor_y = area.y;

            if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
                // Get character at cursor position or space
                let cursor_char = self
                    .content
                    .chars()
                    .nth(self.cursor_position)
                    .unwrap_or(' ');

                buf.get_mut(cursor_x, cursor_y)
                    .set_char(cursor_char)
                    .set_style(self.cursor_style);
            }
        }
    }
}

/// Input field utilities
impl<'a> InputField<'a> {
    /// Create input field for commands
    pub fn for_commands(content: &'a str, cursor_position: usize, theme: &'a WidgetTheme) -> Self {
        Self::professional(content, cursor_position, theme)
            .placeholder("Enter command (ask, analyze, plan, help)")
    }

    /// Create input field for questions
    pub fn for_questions(content: &'a str, cursor_position: usize, theme: &'a WidgetTheme) -> Self {
        Self::professional(content, cursor_position, theme)
            .placeholder("Ask the AI consensus anything...")
    }

    /// Create input field for file paths
    pub fn for_paths(content: &'a str, cursor_position: usize, theme: &'a WidgetTheme) -> Self {
        Self::professional(content, cursor_position, theme)
            .placeholder("Enter file or directory path...")
    }
}
