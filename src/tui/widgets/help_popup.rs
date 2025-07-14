//! Help Popup Widget for Keyboard Shortcuts and Commands

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};

use super::WidgetTheme;

/// Help popup widget for displaying shortcuts and commands
pub struct HelpPopup<'a> {
    /// Widget theme
    theme: &'a WidgetTheme,

    /// Help content type
    content_type: HelpContentType,

    /// Custom title
    title: Option<&'a str>,
}

/// Types of help content to display
#[derive(Clone, Debug)]
pub enum HelpContentType {
    KeyboardShortcuts,
    Commands,
    About,
    Full,
}

impl<'a> HelpPopup<'a> {
    /// Create new help popup
    pub fn new(theme: &'a WidgetTheme) -> Self {
        Self {
            theme,
            content_type: HelpContentType::Full,
            title: None,
        }
    }

    /// Set content type
    pub fn content_type(mut self, content_type: HelpContentType) -> Self {
        self.content_type = content_type;
        self
    }

    /// Set custom title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Create keyboard shortcuts help
    pub fn keyboard_shortcuts(theme: &'a WidgetTheme) -> Self {
        Self::new(theme)
            .content_type(HelpContentType::KeyboardShortcuts)
            .title("⌨️  Keyboard Shortcuts")
    }

    /// Create commands help
    pub fn commands(theme: &'a WidgetTheme) -> Self {
        Self::new(theme)
            .content_type(HelpContentType::Commands)
            .title("📚 Available Commands")
    }

    /// Create about dialog
    pub fn about(theme: &'a WidgetTheme) -> Self {
        Self::new(theme)
            .content_type(HelpContentType::About)
            .title("🐝 About HiveTechs Consensus")
    }

    /// Get help content based on type
    fn get_help_content(&self) -> Vec<Line> {
        match self.content_type {
            HelpContentType::KeyboardShortcuts => self.keyboard_shortcuts_content(),
            HelpContentType::Commands => self.commands_content(),
            HelpContentType::About => self.about_content(),
            HelpContentType::Full => self.full_help_content(),
        }
    }

    /// Keyboard shortcuts content
    fn keyboard_shortcuts_content(&self) -> Vec<Line> {
        vec![
            Line::from(vec![Span::styled(
                "Navigation:",
                self.theme.accent_style().add_modifier(Modifier::BOLD),
            )]),
            Line::from("↑/↓                   - Navigate command history"),
            Line::from("←/→                   - Move cursor in input line"),
            Line::from("Page Up/Down          - Scroll messages"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Panel Focus (VS Code style):",
                self.theme.accent_style().add_modifier(Modifier::BOLD),
            )]),
            Line::from("F1                    - Focus Input/Messages"),
            Line::from("F2                    - Focus File Explorer"),
            Line::from("F3                    - Focus Consensus Panel"),
            Line::from("F4                    - Focus Terminal"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Quick Commands:",
                self.theme.accent_style().add_modifier(Modifier::BOLD),
            )]),
            Line::from("Ctrl+H                - Quick ask command"),
            Line::from("Ctrl+A                - Quick analyze command"),
            Line::from("Ctrl+P                - Quick plan command"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Other:",
                self.theme.accent_style().add_modifier(Modifier::BOLD),
            )]),
            Line::from("Ctrl+L                - Clear screen"),
            Line::from("Shift+Tab             - Toggle auto-accept edits"),
            Line::from("?                     - Show/hide this help"),
            Line::from("Esc                   - Clear input"),
            Line::from("Ctrl+C/Ctrl+D        - Exit application"),
        ]
    }

    /// Commands content
    fn commands_content(&self) -> Vec<Line> {
        vec![
            Line::from(vec![Span::styled(
                "Core Commands:",
                self.theme.accent_style().add_modifier(Modifier::BOLD),
            )]),
            Line::from("ask <question>        - Ask the AI consensus a question"),
            Line::from("analyze <path>        - Analyze repository or file with ML intelligence"),
            Line::from("plan <goal>           - Create strategic development plan"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "System Commands:",
                self.theme.accent_style().add_modifier(Modifier::BOLD),
            )]),
            Line::from("help                  - Show command help"),
            Line::from("status                - Show system status"),
            Line::from("exit, quit            - Exit the application"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Examples:",
                self.theme.accent_style().add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::raw("ask "),
                Span::styled(
                    "\"What is the best way to structure this Rust project?\"",
                    self.theme.secondary_style(),
                ),
            ]),
            Line::from(vec![
                Span::raw("analyze "),
                Span::styled("src/", self.theme.secondary_style()),
            ]),
            Line::from(vec![
                Span::raw("plan "),
                Span::styled(
                    "\"Add user authentication with JWT tokens\"",
                    self.theme.secondary_style(),
                ),
            ]),
        ]
    }

    /// About content
    fn about_content(&self) -> Vec<Line> {
        vec![
            Line::from(vec![
                Span::styled(
                    "HiveTechs Consensus",
                    self.theme.primary_style().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Professional AI Assistant"),
            ]),
            Line::from(""),
            Line::from("Version: 2.0.0-dev (Rust Implementation)"),
            Line::from("Performance: 10-40x faster than TypeScript version"),
            Line::from("Memory Usage: ~25MB (vs 180MB TypeScript)"),
            Line::from("Startup Time: <50ms (vs 2.1s TypeScript)"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Features:",
                self.theme.accent_style().add_modifier(Modifier::BOLD),
            )]),
            Line::from("• 4-stage AI consensus with 323+ models"),
            Line::from("• Real-time temporal awareness"),
            Line::from("• ML-powered repository intelligence"),
            Line::from("• Enterprise hooks for deterministic AI control"),
            Line::from("• Strategic development planning mode"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Technology:",
                self.theme.accent_style().add_modifier(Modifier::BOLD),
            )]),
            Line::from("• Built with Rust for maximum performance"),
            Line::from("• ratatui for professional terminal interface"),
            Line::from("• OpenRouter for AI model access"),
            Line::from("• SQLite for local data storage"),
            Line::from(""),
            Line::from("© 2024 HiveTechs Collective"),
        ]
    }

    /// Full help content
    fn full_help_content(&self) -> Vec<Line> {
        let mut content = Vec::new();

        // Add commands section
        content.extend(self.commands_content());
        content.push(Line::from(""));
        content.push(Line::from("─".repeat(50)));
        content.push(Line::from(""));

        // Add keyboard shortcuts section
        content.extend(self.keyboard_shortcuts_content());

        content
    }
}

impl<'a> Widget for HelpPopup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate popup size (80% of screen)
        let popup_width = (area.width as f32 * 0.8) as u16;
        let popup_height = (area.height as f32 * 0.8) as u16;

        // Center the popup
        let popup_x = (area.width - popup_width) / 2;
        let popup_y = (area.height - popup_height) / 2;

        let popup_area = Rect {
            x: area.x + popup_x,
            y: area.y + popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Clear the background
        Clear.render(popup_area, buf);

        // Create the help block
        let title = self.title.unwrap_or("🆘 Help");
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.primary_style())
            .title(title)
            .title_style(self.theme.primary_style().add_modifier(Modifier::BOLD));

        let inner_area = block.inner(popup_area);
        block.render(popup_area, buf);

        // Get help content
        let content = self.get_help_content();

        // Create help text
        let help_text = Text::from(content);
        let paragraph = Paragraph::new(help_text)
            .style(self.theme.text_style())
            .wrap(Wrap { trim: true })
            .scroll((0, 0));

        paragraph.render(inner_area, buf);

        // Add footer
        let footer_area = Rect {
            x: inner_area.x,
            y: inner_area.y + inner_area.height - 1,
            width: inner_area.width,
            height: 1,
        };

        let footer = Paragraph::new("Press ? again to close this help")
            .style(self.theme.secondary_style())
            .alignment(Alignment::Center);

        footer.render(footer_area, buf);
    }
}

/// Help popup utilities
impl<'a> HelpPopup<'a> {
    /// Calculate required size for content
    pub fn required_size(&self) -> (u16, u16) {
        let content = self.get_help_content();
        let height = content.len() as u16 + 4; // Add padding for borders
        let width = content
            .iter()
            .map(|line| line.width() as u16)
            .max()
            .unwrap_or(50)
            + 4; // Add padding for borders

        (width.min(100), height.min(30))
    }

    /// Check if popup should be shown based on screen size
    pub fn fits_in_area(&self, area: Rect) -> bool {
        let (required_width, required_height) = self.required_size();
        area.width >= required_width && area.height >= required_height
    }
}
