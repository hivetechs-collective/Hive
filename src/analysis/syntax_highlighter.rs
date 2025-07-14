//! Syntax highlighting engine for TUI
//!
//! This module provides syntax highlighting using tree-sitter queries
//! optimized for terminal display with color schemes.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::parser::{HighlightSpan, HighlightType, TreeSitterParser};
use crate::core::Position;

/// Syntax highlighter for terminal display
pub struct SyntaxHighlighter {
    /// Color schemes
    color_schemes: ColorSchemeRegistry,
    /// Current color scheme
    current_scheme: String,
}

/// Color scheme for syntax highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    /// Scheme name
    pub name: String,
    /// Theme type (dark/light)
    pub theme_type: ThemeType,
    /// Color mappings
    pub colors: std::collections::HashMap<HighlightType, TerminalColor>,
}

/// Terminal color representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerminalColor {
    /// Foreground color
    pub fg: Color,
    /// Background color (optional)
    pub bg: Option<Color>,
    /// Text style
    pub style: TextStyle,
}

/// Color values
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Color {
    /// ANSI 16 colors
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    /// RGB color
    Rgb(u8, u8, u8),
    /// 256 color palette
    Indexed(u8),
}

/// Text styling options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

/// Theme type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeType {
    Dark,
    Light,
}

/// Color scheme registry
struct ColorSchemeRegistry {
    schemes: std::collections::HashMap<String, ColorScheme>,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter
    pub fn new() -> Self {
        let mut registry = ColorSchemeRegistry::new();
        registry.register_default_schemes();

        Self {
            color_schemes: registry,
            current_scheme: "monokai".to_string(),
        }
    }

    /// Highlight source code
    pub async fn highlight(
        &self,
        parser: Arc<Mutex<TreeSitterParser>>,
        content: &str,
    ) -> Result<Vec<HighlightSpan>> {
        let mut parser = parser.lock().await;

        // Parse the content
        let tree = parser
            .parser
            .parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse content for highlighting"))?;

        // Get highlight spans
        parser.get_highlights(&tree, content)
    }

    /// Apply color scheme to highlight spans
    pub fn apply_colors(&self, spans: &[HighlightSpan]) -> Vec<ColoredSpan> {
        let scheme = self
            .color_schemes
            .schemes
            .get(&self.current_scheme)
            .unwrap_or_else(|| self.color_schemes.schemes.get("monokai").unwrap());

        spans
            .iter()
            .map(|span| {
                let color =
                    scheme
                        .colors
                        .get(&span.highlight_type)
                        .cloned()
                        .unwrap_or(TerminalColor {
                            fg: Color::White,
                            bg: None,
                            style: TextStyle::default(),
                        });

                ColoredSpan {
                    start: span.start,
                    end: span.end,
                    color,
                }
            })
            .collect()
    }

    /// Set the current color scheme
    pub fn set_scheme(&mut self, name: &str) -> Result<()> {
        if self.color_schemes.schemes.contains_key(name) {
            self.current_scheme = name.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Color scheme '{}' not found", name))
        }
    }

    /// Get available color schemes
    pub fn list_schemes(&self) -> Vec<&str> {
        self.color_schemes
            .schemes
            .keys()
            .map(|s| s.as_str())
            .collect()
    }

    /// Get color scheme for theme type
    pub fn get_scheme_for_theme(&self, theme_type: ThemeType) -> &str {
        match theme_type {
            ThemeType::Dark => "monokai",
            ThemeType::Light => "github",
        }
    }
}

/// Span with color information
#[derive(Debug, Clone)]
pub struct ColoredSpan {
    pub start: Position,
    pub end: Position,
    pub color: TerminalColor,
}

impl ColorSchemeRegistry {
    fn new() -> Self {
        Self {
            schemes: std::collections::HashMap::new(),
        }
    }

    fn register_default_schemes(&mut self) {
        // Monokai (dark theme)
        let mut monokai_colors = std::collections::HashMap::new();
        monokai_colors.insert(
            HighlightType::Keyword,
            TerminalColor {
                fg: Color::Rgb(249, 38, 114), // Pink
                bg: None,
                style: TextStyle {
                    bold: true,
                    ..Default::default()
                },
            },
        );
        monokai_colors.insert(
            HighlightType::Function,
            TerminalColor {
                fg: Color::Rgb(166, 226, 46), // Green
                bg: None,
                style: TextStyle::default(),
            },
        );
        monokai_colors.insert(
            HighlightType::Type,
            TerminalColor {
                fg: Color::Rgb(102, 217, 239), // Cyan
                bg: None,
                style: TextStyle {
                    italic: true,
                    ..Default::default()
                },
            },
        );
        monokai_colors.insert(
            HighlightType::Variable,
            TerminalColor {
                fg: Color::White,
                bg: None,
                style: TextStyle::default(),
            },
        );
        monokai_colors.insert(
            HighlightType::String,
            TerminalColor {
                fg: Color::Rgb(230, 219, 116), // Yellow
                bg: None,
                style: TextStyle::default(),
            },
        );
        monokai_colors.insert(
            HighlightType::Number,
            TerminalColor {
                fg: Color::Rgb(174, 129, 255), // Purple
                bg: None,
                style: TextStyle::default(),
            },
        );
        monokai_colors.insert(
            HighlightType::Comment,
            TerminalColor {
                fg: Color::Rgb(117, 113, 94), // Gray
                bg: None,
                style: TextStyle {
                    italic: true,
                    ..Default::default()
                },
            },
        );
        monokai_colors.insert(
            HighlightType::Operator,
            TerminalColor {
                fg: Color::Rgb(249, 38, 114), // Pink
                bg: None,
                style: TextStyle::default(),
            },
        );
        monokai_colors.insert(
            HighlightType::Constant,
            TerminalColor {
                fg: Color::Rgb(174, 129, 255), // Purple
                bg: None,
                style: TextStyle::default(),
            },
        );

        self.schemes.insert(
            "monokai".to_string(),
            ColorScheme {
                name: "monokai".to_string(),
                theme_type: ThemeType::Dark,
                colors: monokai_colors,
            },
        );

        // GitHub (light theme)
        let mut github_colors = std::collections::HashMap::new();
        github_colors.insert(
            HighlightType::Keyword,
            TerminalColor {
                fg: Color::Rgb(215, 58, 73), // Red
                bg: None,
                style: TextStyle {
                    bold: true,
                    ..Default::default()
                },
            },
        );
        github_colors.insert(
            HighlightType::Function,
            TerminalColor {
                fg: Color::Rgb(111, 66, 193), // Purple
                bg: None,
                style: TextStyle::default(),
            },
        );
        github_colors.insert(
            HighlightType::Type,
            TerminalColor {
                fg: Color::Rgb(0, 92, 197), // Blue
                bg: None,
                style: TextStyle::default(),
            },
        );
        github_colors.insert(
            HighlightType::Variable,
            TerminalColor {
                fg: Color::Black,
                bg: None,
                style: TextStyle::default(),
            },
        );
        github_colors.insert(
            HighlightType::String,
            TerminalColor {
                fg: Color::Rgb(3, 47, 98), // Dark blue
                bg: None,
                style: TextStyle::default(),
            },
        );
        github_colors.insert(
            HighlightType::Number,
            TerminalColor {
                fg: Color::Rgb(0, 92, 197), // Blue
                bg: None,
                style: TextStyle::default(),
            },
        );
        github_colors.insert(
            HighlightType::Comment,
            TerminalColor {
                fg: Color::Rgb(106, 115, 125), // Gray
                bg: None,
                style: TextStyle {
                    italic: true,
                    ..Default::default()
                },
            },
        );
        github_colors.insert(
            HighlightType::Operator,
            TerminalColor {
                fg: Color::Black,
                bg: None,
                style: TextStyle::default(),
            },
        );
        github_colors.insert(
            HighlightType::Constant,
            TerminalColor {
                fg: Color::Rgb(0, 92, 197), // Blue
                bg: None,
                style: TextStyle::default(),
            },
        );

        self.schemes.insert(
            "github".to_string(),
            ColorScheme {
                name: "github".to_string(),
                theme_type: ThemeType::Light,
                colors: github_colors,
            },
        );

        // Dracula (dark theme)
        let mut dracula_colors = std::collections::HashMap::new();
        dracula_colors.insert(
            HighlightType::Keyword,
            TerminalColor {
                fg: Color::Rgb(255, 121, 198), // Pink
                bg: None,
                style: TextStyle {
                    bold: true,
                    ..Default::default()
                },
            },
        );
        dracula_colors.insert(
            HighlightType::Function,
            TerminalColor {
                fg: Color::Rgb(80, 250, 123), // Green
                bg: None,
                style: TextStyle::default(),
            },
        );
        dracula_colors.insert(
            HighlightType::Type,
            TerminalColor {
                fg: Color::Rgb(139, 233, 253), // Cyan
                bg: None,
                style: TextStyle {
                    italic: true,
                    ..Default::default()
                },
            },
        );
        dracula_colors.insert(
            HighlightType::Variable,
            TerminalColor {
                fg: Color::Rgb(248, 248, 242), // Foreground
                bg: None,
                style: TextStyle::default(),
            },
        );
        dracula_colors.insert(
            HighlightType::String,
            TerminalColor {
                fg: Color::Rgb(241, 250, 140), // Yellow
                bg: None,
                style: TextStyle::default(),
            },
        );
        dracula_colors.insert(
            HighlightType::Number,
            TerminalColor {
                fg: Color::Rgb(189, 147, 249), // Purple
                bg: None,
                style: TextStyle::default(),
            },
        );
        dracula_colors.insert(
            HighlightType::Comment,
            TerminalColor {
                fg: Color::Rgb(98, 114, 164), // Comment
                bg: None,
                style: TextStyle {
                    italic: true,
                    ..Default::default()
                },
            },
        );

        self.schemes.insert(
            "dracula".to_string(),
            ColorScheme {
                name: "dracula".to_string(),
                theme_type: ThemeType::Dark,
                colors: dracula_colors,
            },
        );
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert color to ANSI escape sequence
impl Color {
    pub fn to_ansi_fg(&self) -> String {
        match self {
            Color::Black => "\x1b[30m".to_string(),
            Color::Red => "\x1b[31m".to_string(),
            Color::Green => "\x1b[32m".to_string(),
            Color::Yellow => "\x1b[33m".to_string(),
            Color::Blue => "\x1b[34m".to_string(),
            Color::Magenta => "\x1b[35m".to_string(),
            Color::Cyan => "\x1b[36m".to_string(),
            Color::White => "\x1b[37m".to_string(),
            Color::BrightBlack => "\x1b[90m".to_string(),
            Color::BrightRed => "\x1b[91m".to_string(),
            Color::BrightGreen => "\x1b[92m".to_string(),
            Color::BrightYellow => "\x1b[93m".to_string(),
            Color::BrightBlue => "\x1b[94m".to_string(),
            Color::BrightMagenta => "\x1b[95m".to_string(),
            Color::BrightCyan => "\x1b[96m".to_string(),
            Color::BrightWhite => "\x1b[97m".to_string(),
            Color::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
            Color::Indexed(n) => format!("\x1b[38;5;{}m", n),
        }
    }

    pub fn to_ansi_bg(&self) -> String {
        match self {
            Color::Black => "\x1b[40m".to_string(),
            Color::Red => "\x1b[41m".to_string(),
            Color::Green => "\x1b[42m".to_string(),
            Color::Yellow => "\x1b[43m".to_string(),
            Color::Blue => "\x1b[44m".to_string(),
            Color::Magenta => "\x1b[45m".to_string(),
            Color::Cyan => "\x1b[46m".to_string(),
            Color::White => "\x1b[47m".to_string(),
            Color::BrightBlack => "\x1b[100m".to_string(),
            Color::BrightRed => "\x1b[101m".to_string(),
            Color::BrightGreen => "\x1b[102m".to_string(),
            Color::BrightYellow => "\x1b[103m".to_string(),
            Color::BrightBlue => "\x1b[104m".to_string(),
            Color::BrightMagenta => "\x1b[105m".to_string(),
            Color::BrightCyan => "\x1b[106m".to_string(),
            Color::BrightWhite => "\x1b[107m".to_string(),
            Color::Rgb(r, g, b) => format!("\x1b[48;2;{};{};{}m", r, g, b),
            Color::Indexed(n) => format!("\x1b[48;5;{}m", n),
        }
    }
}

impl TextStyle {
    pub fn to_ansi(&self) -> String {
        let mut codes = Vec::new();

        if self.bold {
            codes.push("1");
        }
        if self.italic {
            codes.push("3");
        }
        if self.underline {
            codes.push("4");
        }
        if self.strikethrough {
            codes.push("9");
        }

        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }
}

impl TerminalColor {
    pub fn to_ansi(&self) -> String {
        let mut result = String::new();

        // Add style codes
        result.push_str(&self.style.to_ansi());

        // Add foreground color
        result.push_str(&self.fg.to_ansi_fg());

        // Add background color if present
        if let Some(bg) = &self.bg {
            result.push_str(&bg.to_ansi_bg());
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Language;

    #[tokio::test]
    async fn test_syntax_highlighting() {
        let highlighter = SyntaxHighlighter::new();
        let parser = Arc::new(Mutex::new(TreeSitterParser::new(Language::Rust).unwrap()));

        let content = r#"fn main() {
    let x = 42;
    println!("Hello");
}"#;

        let spans = highlighter
            .highlight(Arc::clone(&parser), content)
            .await
            .unwrap();
        assert!(!spans.is_empty());

        // Apply colors
        let colored_spans = highlighter.apply_colors(&spans);
        assert_eq!(spans.len(), colored_spans.len());
    }

    #[test]
    fn test_color_schemes() {
        let mut highlighter = SyntaxHighlighter::new();

        // List schemes
        let schemes = highlighter.list_schemes();
        assert!(schemes.contains(&"monokai"));
        assert!(schemes.contains(&"github"));
        assert!(schemes.contains(&"dracula"));

        // Change scheme
        highlighter.set_scheme("github").unwrap();
        assert_eq!(highlighter.current_scheme, "github");

        // Invalid scheme
        assert!(highlighter.set_scheme("invalid").is_err());
    }

    #[test]
    fn test_ansi_conversion() {
        let color = Color::Rgb(255, 0, 0);
        assert_eq!(color.to_ansi_fg(), "\x1b[38;2;255;0;0m");

        let style = TextStyle {
            bold: true,
            italic: true,
            ..Default::default()
        };
        assert_eq!(style.to_ansi(), "\x1b[1;3m");
    }
}
