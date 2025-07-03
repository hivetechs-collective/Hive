//! TUI Theming System for HiveTechs Consensus
//!
//! Provides customizable themes for the terminal interface with TOML configuration.

use anyhow::Result;
use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// TUI theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiTheme {
    pub name: String,
    pub description: String,
    pub colors: ThemeColors,
    pub styles: ThemeStyles,
    pub accessibility: AccessibilityOptions,
}

/// Color palette for the theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // Basic colors
    pub background: ThemeColor,
    pub foreground: ThemeColor,
    pub border: ThemeColor,
    pub cursor: ThemeColor,
    
    // Status colors
    pub success: ThemeColor,
    pub warning: ThemeColor,
    pub error: ThemeColor,
    pub info: ThemeColor,
    
    // Consensus stage colors
    pub generator: ThemeColor,
    pub refiner: ThemeColor,
    pub validator: ThemeColor,
    pub curator: ThemeColor,
    
    // Progress colors
    pub progress_active: ThemeColor,
    pub progress_complete: ThemeColor,
    pub progress_waiting: ThemeColor,
    pub progress_error: ThemeColor,
    
    // Message type colors
    pub user_input: ThemeColor,
    pub system_response: ThemeColor,
    pub consensus_progress: ThemeColor,
    pub help_text: ThemeColor,
    pub status_text: ThemeColor,
    
    // Syntax highlighting (for future code editor)
    pub syntax_keyword: ThemeColor,
    pub syntax_string: ThemeColor,
    pub syntax_comment: ThemeColor,
    pub syntax_number: ThemeColor,
    pub syntax_function: ThemeColor,
}

/// Style definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeStyles {
    pub title: TextStyle,
    pub input: TextStyle,
    pub status_line: TextStyle,
    pub progress_bar: TextStyle,
    pub button: TextStyle,
    pub selected: TextStyle,
    pub focus: TextStyle,
}

/// Text style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub strikethrough: bool,
}

/// Theme color definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThemeColor {
    Named(String),      // "red", "blue", etc.
    Rgb { r: u8, g: u8, b: u8 },
    Indexed(u8),        // 0-255
}

/// Accessibility options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityOptions {
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub screen_reader_friendly: bool,
    pub color_blind_friendly: bool,
}

/// Theme manager for TUI
pub struct ThemeManager {
    current_theme: TuiTheme,
    available_themes: HashMap<String, TuiTheme>,
}

impl ThemeManager {
    /// Create a new theme manager with default theme
    pub fn new() -> Self {
        let mut available_themes = HashMap::new();
        
        // Add built-in themes
        available_themes.insert("dark".to_string(), Self::dark_theme());
        available_themes.insert("light".to_string(), Self::light_theme());
        available_themes.insert("solarized".to_string(), Self::solarized_theme());
        available_themes.insert("high_contrast".to_string(), Self::high_contrast_theme());
        available_themes.insert("monokai".to_string(), Self::monokai_theme());
        
        let current_theme = Self::dark_theme(); // Default to dark theme
        
        Self {
            current_theme,
            available_themes,
        }
    }

    /// Load theme from configuration
    pub fn load_theme(&mut self, theme_name: &str) -> Result<()> {
        if let Some(theme) = self.available_themes.get(theme_name).cloned() {
            self.current_theme = theme;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Theme '{}' not found", theme_name))
        }
    }

    /// Get current theme
    pub fn current_theme(&self) -> &TuiTheme {
        &self.current_theme
    }

    /// Get available theme names
    pub fn available_themes(&self) -> Vec<&String> {
        self.available_themes.keys().collect()
    }

    /// Convert theme color to ratatui Color
    pub fn to_ratatui_color(&self, theme_color: &ThemeColor) -> Color {
        match theme_color {
            ThemeColor::Named(name) => match name.as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "gray" => Color::Gray,
                "white" => Color::White,
                "light_red" => Color::LightRed,
                "light_green" => Color::LightGreen,
                "light_yellow" => Color::LightYellow,
                "light_blue" => Color::LightBlue,
                "light_magenta" => Color::LightMagenta,
                "light_cyan" => Color::LightCyan,
                "dark_gray" => Color::DarkGray,
                _ => Color::White, // Fallback
            },
            ThemeColor::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
            ThemeColor::Indexed(index) => Color::Indexed(*index),
        }
    }

    /// Convert text style to ratatui Style
    pub fn to_ratatui_style(&self, text_style: &TextStyle, color: &ThemeColor) -> Style {
        let mut style = Style::default().fg(self.to_ratatui_color(color));
        
        if text_style.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if text_style.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if text_style.underlined {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        if text_style.strikethrough {
            style = style.add_modifier(Modifier::CROSSED_OUT);
        }
        
        style
    }

    // Built-in theme definitions
    fn dark_theme() -> TuiTheme {
        TuiTheme {
            name: "Dark".to_string(),
            description: "Classic dark theme with cyan accents".to_string(),
            colors: ThemeColors {
                background: ThemeColor::Named("black".to_string()),
                foreground: ThemeColor::Named("white".to_string()),
                border: ThemeColor::Named("cyan".to_string()),
                cursor: ThemeColor::Named("white".to_string()),
                
                success: ThemeColor::Named("green".to_string()),
                warning: ThemeColor::Named("yellow".to_string()),
                error: ThemeColor::Named("red".to_string()),
                info: ThemeColor::Named("blue".to_string()),
                
                generator: ThemeColor::Named("cyan".to_string()),
                refiner: ThemeColor::Named("yellow".to_string()),
                validator: ThemeColor::Named("green".to_string()),
                curator: ThemeColor::Named("magenta".to_string()),
                
                progress_active: ThemeColor::Named("yellow".to_string()),
                progress_complete: ThemeColor::Named("green".to_string()),
                progress_waiting: ThemeColor::Named("gray".to_string()),
                progress_error: ThemeColor::Named("red".to_string()),
                
                user_input: ThemeColor::Named("green".to_string()),
                system_response: ThemeColor::Named("white".to_string()),
                consensus_progress: ThemeColor::Named("yellow".to_string()),
                help_text: ThemeColor::Named("blue".to_string()),
                status_text: ThemeColor::Named("gray".to_string()),
                
                syntax_keyword: ThemeColor::Named("magenta".to_string()),
                syntax_string: ThemeColor::Named("green".to_string()),
                syntax_comment: ThemeColor::Named("gray".to_string()),
                syntax_number: ThemeColor::Named("cyan".to_string()),
                syntax_function: ThemeColor::Named("yellow".to_string()),
            },
            styles: Self::default_styles(),
            accessibility: Self::default_accessibility(),
        }
    }

    fn light_theme() -> TuiTheme {
        TuiTheme {
            name: "Light".to_string(),
            description: "Clean light theme with dark text".to_string(),
            colors: ThemeColors {
                background: ThemeColor::Named("white".to_string()),
                foreground: ThemeColor::Named("black".to_string()),
                border: ThemeColor::Named("blue".to_string()),
                cursor: ThemeColor::Named("black".to_string()),
                
                success: ThemeColor::Named("green".to_string()),
                warning: ThemeColor::Rgb { r: 255, g: 140, b: 0 }, // Orange
                error: ThemeColor::Named("red".to_string()),
                info: ThemeColor::Named("blue".to_string()),
                
                generator: ThemeColor::Named("blue".to_string()),
                refiner: ThemeColor::Rgb { r: 255, g: 140, b: 0 }, // Orange
                validator: ThemeColor::Named("green".to_string()),
                curator: ThemeColor::Named("magenta".to_string()),
                
                progress_active: ThemeColor::Rgb { r: 255, g: 140, b: 0 },
                progress_complete: ThemeColor::Named("green".to_string()),
                progress_waiting: ThemeColor::Named("gray".to_string()),
                progress_error: ThemeColor::Named("red".to_string()),
                
                user_input: ThemeColor::Named("green".to_string()),
                system_response: ThemeColor::Named("black".to_string()),
                consensus_progress: ThemeColor::Rgb { r: 255, g: 140, b: 0 },
                help_text: ThemeColor::Named("blue".to_string()),
                status_text: ThemeColor::Named("dark_gray".to_string()),
                
                syntax_keyword: ThemeColor::Named("magenta".to_string()),
                syntax_string: ThemeColor::Named("green".to_string()),
                syntax_comment: ThemeColor::Named("gray".to_string()),
                syntax_number: ThemeColor::Named("blue".to_string()),
                syntax_function: ThemeColor::Rgb { r: 255, g: 140, b: 0 },
            },
            styles: Self::default_styles(),
            accessibility: Self::default_accessibility(),
        }
    }

    fn solarized_theme() -> TuiTheme {
        TuiTheme {
            name: "Solarized".to_string(),
            description: "Solarized dark theme with carefully balanced colors".to_string(),
            colors: ThemeColors {
                background: ThemeColor::Rgb { r: 0, g: 43, b: 54 },      // base03
                foreground: ThemeColor::Rgb { r: 131, g: 148, b: 150 },  // base0
                border: ThemeColor::Rgb { r: 42, g: 161, b: 152 },       // cyan
                cursor: ThemeColor::Rgb { r: 238, g: 232, b: 213 },      // base2
                
                success: ThemeColor::Rgb { r: 133, g: 153, b: 0 },       // green
                warning: ThemeColor::Rgb { r: 181, g: 137, b: 0 },       // yellow
                error: ThemeColor::Rgb { r: 220, g: 50, b: 47 },         // red
                info: ThemeColor::Rgb { r: 38, g: 139, b: 210 },         // blue
                
                generator: ThemeColor::Rgb { r: 42, g: 161, b: 152 },    // cyan
                refiner: ThemeColor::Rgb { r: 181, g: 137, b: 0 },       // yellow
                validator: ThemeColor::Rgb { r: 133, g: 153, b: 0 },     // green
                curator: ThemeColor::Rgb { r: 211, g: 54, b: 130 },      // magenta
                
                progress_active: ThemeColor::Rgb { r: 181, g: 137, b: 0 },
                progress_complete: ThemeColor::Rgb { r: 133, g: 153, b: 0 },
                progress_waiting: ThemeColor::Rgb { r: 88, g: 110, b: 117 },
                progress_error: ThemeColor::Rgb { r: 220, g: 50, b: 47 },
                
                user_input: ThemeColor::Rgb { r: 133, g: 153, b: 0 },
                system_response: ThemeColor::Rgb { r: 131, g: 148, b: 150 },
                consensus_progress: ThemeColor::Rgb { r: 181, g: 137, b: 0 },
                help_text: ThemeColor::Rgb { r: 38, g: 139, b: 210 },
                status_text: ThemeColor::Rgb { r: 88, g: 110, b: 117 },
                
                syntax_keyword: ThemeColor::Rgb { r: 211, g: 54, b: 130 },
                syntax_string: ThemeColor::Rgb { r: 42, g: 161, b: 152 },
                syntax_comment: ThemeColor::Rgb { r: 88, g: 110, b: 117 },
                syntax_number: ThemeColor::Rgb { r: 220, g: 50, b: 47 },
                syntax_function: ThemeColor::Rgb { r: 38, g: 139, b: 210 },
            },
            styles: Self::default_styles(),
            accessibility: Self::default_accessibility(),
        }
    }

    fn high_contrast_theme() -> TuiTheme {
        TuiTheme {
            name: "High Contrast".to_string(),
            description: "High contrast theme for better accessibility".to_string(),
            colors: ThemeColors {
                background: ThemeColor::Named("black".to_string()),
                foreground: ThemeColor::Named("white".to_string()),
                border: ThemeColor::Named("white".to_string()),
                cursor: ThemeColor::Named("white".to_string()),
                
                success: ThemeColor::Named("light_green".to_string()),
                warning: ThemeColor::Named("light_yellow".to_string()),
                error: ThemeColor::Named("light_red".to_string()),
                info: ThemeColor::Named("light_blue".to_string()),
                
                generator: ThemeColor::Named("light_cyan".to_string()),
                refiner: ThemeColor::Named("light_yellow".to_string()),
                validator: ThemeColor::Named("light_green".to_string()),
                curator: ThemeColor::Named("light_magenta".to_string()),
                
                progress_active: ThemeColor::Named("white".to_string()),
                progress_complete: ThemeColor::Named("light_green".to_string()),
                progress_waiting: ThemeColor::Named("dark_gray".to_string()),
                progress_error: ThemeColor::Named("light_red".to_string()),
                
                user_input: ThemeColor::Named("light_green".to_string()),
                system_response: ThemeColor::Named("white".to_string()),
                consensus_progress: ThemeColor::Named("light_yellow".to_string()),
                help_text: ThemeColor::Named("light_blue".to_string()),
                status_text: ThemeColor::Named("light_cyan".to_string()),
                
                syntax_keyword: ThemeColor::Named("light_magenta".to_string()),
                syntax_string: ThemeColor::Named("light_green".to_string()),
                syntax_comment: ThemeColor::Named("gray".to_string()),
                syntax_number: ThemeColor::Named("light_cyan".to_string()),
                syntax_function: ThemeColor::Named("light_yellow".to_string()),
            },
            styles: Self::default_styles(),
            accessibility: AccessibilityOptions {
                high_contrast: true,
                reduced_motion: true,
                screen_reader_friendly: true,
                color_blind_friendly: true,
            },
        }
    }

    fn monokai_theme() -> TuiTheme {
        TuiTheme {
            name: "Monokai".to_string(),
            description: "Popular Monokai theme with vibrant colors".to_string(),
            colors: ThemeColors {
                background: ThemeColor::Rgb { r: 39, g: 40, b: 34 },      // Dark gray
                foreground: ThemeColor::Rgb { r: 248, g: 248, b: 242 },   // Light gray
                border: ThemeColor::Rgb { r: 166, g: 226, b: 46 },        // Green
                cursor: ThemeColor::Rgb { r: 248, g: 248, b: 242 },
                
                success: ThemeColor::Rgb { r: 166, g: 226, b: 46 },       // Green
                warning: ThemeColor::Rgb { r: 244, g: 191, b: 117 },      // Orange
                error: ThemeColor::Rgb { r: 249, g: 38, b: 114 },         // Pink
                info: ThemeColor::Rgb { r: 102, g: 217, b: 239 },         // Cyan
                
                generator: ThemeColor::Rgb { r: 102, g: 217, b: 239 },    // Cyan
                refiner: ThemeColor::Rgb { r: 244, g: 191, b: 117 },      // Orange
                validator: ThemeColor::Rgb { r: 166, g: 226, b: 46 },     // Green
                curator: ThemeColor::Rgb { r: 174, g: 129, b: 255 },      // Purple
                
                progress_active: ThemeColor::Rgb { r: 244, g: 191, b: 117 },
                progress_complete: ThemeColor::Rgb { r: 166, g: 226, b: 46 },
                progress_waiting: ThemeColor::Rgb { r: 117, g: 113, b: 94 },
                progress_error: ThemeColor::Rgb { r: 249, g: 38, b: 114 },
                
                user_input: ThemeColor::Rgb { r: 166, g: 226, b: 46 },
                system_response: ThemeColor::Rgb { r: 248, g: 248, b: 242 },
                consensus_progress: ThemeColor::Rgb { r: 244, g: 191, b: 117 },
                help_text: ThemeColor::Rgb { r: 102, g: 217, b: 239 },
                status_text: ThemeColor::Rgb { r: 117, g: 113, b: 94 },
                
                syntax_keyword: ThemeColor::Rgb { r: 249, g: 38, b: 114 },
                syntax_string: ThemeColor::Rgb { r: 230, g: 219, b: 116 },
                syntax_comment: ThemeColor::Rgb { r: 117, g: 113, b: 94 },
                syntax_number: ThemeColor::Rgb { r: 174, g: 129, b: 255 },
                syntax_function: ThemeColor::Rgb { r: 166, g: 226, b: 46 },
            },
            styles: Self::default_styles(),
            accessibility: Self::default_accessibility(),
        }
    }

    fn default_styles() -> ThemeStyles {
        ThemeStyles {
            title: TextStyle { bold: true, italic: false, underlined: false, strikethrough: false },
            input: TextStyle { bold: false, italic: false, underlined: false, strikethrough: false },
            status_line: TextStyle { bold: false, italic: false, underlined: false, strikethrough: false },
            progress_bar: TextStyle { bold: false, italic: false, underlined: false, strikethrough: false },
            button: TextStyle { bold: true, italic: false, underlined: false, strikethrough: false },
            selected: TextStyle { bold: true, italic: false, underlined: false, strikethrough: false },
            focus: TextStyle { bold: true, italic: false, underlined: true, strikethrough: false },
        }
    }

    fn default_accessibility() -> AccessibilityOptions {
        AccessibilityOptions {
            high_contrast: false,
            reduced_motion: false,
            screen_reader_friendly: false,
            color_blind_friendly: false,
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();
        assert_eq!(manager.current_theme().name, "Dark");
        assert!(manager.available_themes().len() >= 5);
    }

    #[test]
    fn test_theme_loading() {
        let mut manager = ThemeManager::new();
        assert!(manager.load_theme("light").is_ok());
        assert_eq!(manager.current_theme().name, "Light");
        
        assert!(manager.load_theme("nonexistent").is_err());
    }

    #[test]
    fn test_color_conversion() {
        let manager = ThemeManager::new();
        
        // Test named color
        let red = ThemeColor::Named("red".to_string());
        assert_eq!(manager.to_ratatui_color(&red), Color::Red);
        
        // Test RGB color
        let rgb = ThemeColor::Rgb { r: 255, g: 128, b: 0 };
        assert_eq!(manager.to_ratatui_color(&rgb), Color::Rgb(255, 128, 0));
        
        // Test indexed color
        let indexed = ThemeColor::Indexed(42);
        assert_eq!(manager.to_ratatui_color(&indexed), Color::Indexed(42));
    }
}