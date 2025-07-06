//! Professional theming system for advanced TUI
//!
//! Provides comprehensive theming with:
//! - Dark, light, and solarized themes
//! - Color customization
//! - Style consistency
//! - Theme switching support

pub mod dark;
pub mod light;
pub mod solarized;

use ratatui::style::{Color, Style, Modifier};
use serde::{Deserialize, Serialize, Serializer, Deserializer};

/// Serializable wrapper for ratatui Color
#[derive(Debug, Clone, PartialEq)]
pub struct SerializableColor(pub Color);

impl Serialize for SerializableColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            Color::Reset => serializer.serialize_str("reset"),
            Color::Black => serializer.serialize_str("black"),
            Color::Red => serializer.serialize_str("red"),
            Color::Green => serializer.serialize_str("green"),
            Color::Yellow => serializer.serialize_str("yellow"),
            Color::Blue => serializer.serialize_str("blue"),
            Color::Magenta => serializer.serialize_str("magenta"),
            Color::Cyan => serializer.serialize_str("cyan"),
            Color::Gray => serializer.serialize_str("gray"),
            Color::DarkGray => serializer.serialize_str("dark_gray"),
            Color::LightRed => serializer.serialize_str("light_red"),
            Color::LightGreen => serializer.serialize_str("light_green"),
            Color::LightYellow => serializer.serialize_str("light_yellow"),
            Color::LightBlue => serializer.serialize_str("light_blue"),
            Color::LightMagenta => serializer.serialize_str("light_magenta"),
            Color::LightCyan => serializer.serialize_str("light_cyan"),
            Color::White => serializer.serialize_str("white"),
            Color::Rgb(r, g, b) => {
                serializer.serialize_str(&format!("rgb({},{},{})", r, g, b))
            }
            Color::Indexed(i) => {
                serializer.serialize_str(&format!("indexed({})", i))
            }
        }
    }
}

impl<'de> Deserialize<'de> for SerializableColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let color = match s.as_str() {
            "reset" => Color::Reset,
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" => Color::Gray,
            "dark_gray" => Color::DarkGray,
            "light_red" => Color::LightRed,
            "light_green" => Color::LightGreen,
            "light_yellow" => Color::LightYellow,
            "light_blue" => Color::LightBlue,
            "light_magenta" => Color::LightMagenta,
            "light_cyan" => Color::LightCyan,
            "white" => Color::White,
            s if s.starts_with("rgb(") && s.ends_with(")") => {
                let inner = &s[4..s.len()-1];
                let parts: Vec<&str> = inner.split(',').collect();
                if parts.len() == 3 {
                    let r = parts[0].parse().map_err(serde::de::Error::custom)?;
                    let g = parts[1].parse().map_err(serde::de::Error::custom)?;
                    let b = parts[2].parse().map_err(serde::de::Error::custom)?;
                    Color::Rgb(r, g, b)
                } else {
                    return Err(serde::de::Error::custom("Invalid RGB format"));
                }
            }
            s if s.starts_with("indexed(") && s.ends_with(")") => {
                let inner = &s[8..s.len()-1];
                let index = inner.parse().map_err(serde::de::Error::custom)?;
                Color::Indexed(index)
            }
            _ => return Err(serde::de::Error::custom("Unknown color format")),
        };
        Ok(SerializableColor(color))
    }
}

impl From<Color> for SerializableColor {
    fn from(color: Color) -> Self {
        SerializableColor(color)
    }
}

impl From<SerializableColor> for Color {
    fn from(sc: SerializableColor) -> Self {
        sc.0
    }
}

/// Theme trait for consistent styling
pub trait ThemeTrait {
    fn name(&self) -> &'static str;
    fn primary_color(&self) -> Color;
    fn secondary_color(&self) -> Color;
    fn background_color(&self) -> Color;
    fn text_color(&self) -> Color;
    fn accent_color(&self) -> Color;
    fn error_color(&self) -> Color;
    fn warning_color(&self) -> Color;
    fn success_color(&self) -> Color;
    fn muted_color(&self) -> Color;
}

/// Main theme structure
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme variant
    pub variant: ThemeVariant,
    /// Color palette
    pub colors: ColorPalette,
    /// Style definitions
    pub styles: StyleDefinitions,
}

/// Available theme variants
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThemeVariant {
    Dark,
    Light,
    Solarized,
}

/// Color palette for themes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    // Base colors
    pub primary: SerializableColor,
    pub secondary: SerializableColor,
    pub background: SerializableColor,
    pub foreground: SerializableColor,
    pub accent: SerializableColor,
    
    // Status colors
    pub error: SerializableColor,
    pub warning: SerializableColor,
    pub success: SerializableColor,
    pub info: SerializableColor,
    pub muted: SerializableColor,
    
    // UI element colors
    pub border_active: SerializableColor,
    pub border_inactive: SerializableColor,
    pub selection_bg: SerializableColor,
    pub selection_fg: SerializableColor,
    
    // Syntax highlighting colors
    pub keyword: SerializableColor,
    pub string: SerializableColor,
    pub comment: SerializableColor,
    pub function: SerializableColor,
    pub variable: SerializableColor,
    pub number: SerializableColor,
    pub operator: SerializableColor,
    
    // File explorer colors
    pub directory: SerializableColor,
    pub file: SerializableColor,
    pub symlink: SerializableColor,
    pub executable: SerializableColor,
    
    // Terminal colors
    pub command: SerializableColor,
    pub output: SerializableColor,
    pub system_message: SerializableColor,
    pub line_number: SerializableColor,
    
    // Editor colors
    pub heading: SerializableColor,
    pub code_block: SerializableColor,
}

/// Style definitions for UI components
#[derive(Debug, Clone)]
pub struct StyleDefinitions {
    pub title_bar: Style,
    pub status_bar: Style,
    pub panel: Style,
    pub active_border: Style,
    pub inactive_border: Style,
    pub selection: Style,
    pub tab: Style,
    pub active_tab: Style,
    pub editor: Style,
    pub terminal: Style,
    pub terminal_output: Style,
    pub popup: Style,
    pub input_active: Style,
    pub input_inactive: Style,
    pub text: Style,
}

impl Theme {
    /// Create new theme with variant
    pub fn new(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Dark => dark::create_dark_theme(),
            ThemeVariant::Light => light::create_light_theme(),
            ThemeVariant::Solarized => solarized::create_solarized_theme(),
        }
    }

    /// Get theme name
    pub fn name(&self) -> &'static str {
        match self.variant {
            ThemeVariant::Dark => "Dark",
            ThemeVariant::Light => "Light",
            ThemeVariant::Solarized => "Solarized",
        }
    }

    /// Switch to different theme variant
    pub fn switch_variant(&mut self, variant: ThemeVariant) {
        *self = Self::new(variant);
    }

    // Style getters for UI components
    pub fn title_bar_style(&self) -> Style {
        self.styles.title_bar
    }

    pub fn status_bar_style(&self) -> Style {
        self.styles.status_bar
    }

    pub fn panel_style(&self) -> Style {
        self.styles.panel
    }

    pub fn active_border_style(&self) -> Style {
        self.styles.active_border
    }

    pub fn inactive_border_style(&self) -> Style {
        self.styles.inactive_border
    }

    pub fn selection_style(&self) -> Style {
        self.styles.selection
    }

    pub fn tab_style(&self) -> Style {
        self.styles.tab
    }

    pub fn active_tab_style(&self) -> Style {
        self.styles.active_tab
    }

    pub fn editor_style(&self) -> Style {
        self.styles.editor
    }

    pub fn terminal_style(&self) -> Style {
        self.styles.terminal
    }

    pub fn terminal_output_style(&self) -> Style {
        self.styles.terminal_output
    }

    pub fn popup_style(&self) -> Style {
        self.styles.popup
    }

    pub fn active_input_style(&self) -> Style {
        self.styles.input_active
    }

    pub fn inactive_input_style(&self) -> Style {
        self.styles.input_inactive
    }

    pub fn text_style(&self) -> Style {
        self.styles.text
    }

    // Color getters
    pub fn primary_color(&self) -> Color {
        self.colors.primary.0
    }

    pub fn secondary_color(&self) -> Color {
        self.colors.secondary.0
    }

    pub fn background_color(&self) -> Color {
        self.colors.background.0
    }

    pub fn foreground_color(&self) -> Color {
        self.colors.foreground.0
    }

    pub fn accent_color(&self) -> Color {
        self.colors.accent.0
    }

    pub fn error_color(&self) -> Color {
        self.colors.error.0
    }

    pub fn warning_color(&self) -> Color {
        self.colors.warning.0
    }

    pub fn success_color(&self) -> Color {
        self.colors.success.0
    }

    pub fn info_color(&self) -> Color {
        self.colors.info.0
    }

    pub fn muted_color(&self) -> Color {
        self.colors.muted.0
    }

    pub fn selection_bg_color(&self) -> Color {
        self.colors.selection_bg.0
    }

    pub fn selection_fg_color(&self) -> Color {
        self.colors.selection_fg.0
    }

    // Convenience methods for TUI advanced components
    pub fn selection_bg(&self) -> Color {
        self.colors.selection_bg.0
    }

    pub fn text_color(&self) -> Color {
        self.colors.foreground.0
    }

    pub fn border_color(&self) -> Color {
        self.colors.border_active.0
    }

    // Syntax highlighting colors
    pub fn keyword_color(&self) -> Color {
        self.colors.keyword.0
    }

    pub fn string_color(&self) -> Color {
        self.colors.string.0
    }

    pub fn comment_color(&self) -> Color {
        self.colors.comment.0
    }

    pub fn function_color(&self) -> Color {
        self.colors.function.0
    }

    pub fn variable_color(&self) -> Color {
        self.colors.variable.0
    }

    pub fn number_color(&self) -> Color {
        self.colors.number.0
    }

    pub fn operator_color(&self) -> Color {
        self.colors.operator.0
    }

    // File explorer colors
    pub fn directory_color(&self) -> Color {
        self.colors.directory.0
    }

    pub fn file_color(&self) -> Color {
        self.colors.file.0
    }

    pub fn symlink_color(&self) -> Color {
        self.colors.symlink.0
    }

    pub fn executable_color(&self) -> Color {
        self.colors.executable.0
    }

    // Terminal colors
    pub fn command_color(&self) -> Color {
        self.colors.command.0
    }

    pub fn output_color(&self) -> Color {
        self.colors.output.0
    }

    pub fn system_message_color(&self) -> Color {
        self.colors.system_message.0
    }

    pub fn line_number_color(&self) -> Color {
        self.colors.line_number.0
    }

    // Editor colors
    pub fn heading_color(&self) -> Color {
        self.colors.heading.0
    }

    pub fn code_block_color(&self) -> Color {
        self.colors.code_block.0
    }

    /// Apply accessibility adjustments
    pub fn apply_accessibility(&mut self, high_contrast: bool, reduce_motion: bool) {
        if high_contrast {
            self.increase_contrast();
        }
        if reduce_motion {
            self.disable_animations();
        }
    }

    /// Increase contrast for accessibility
    fn increase_contrast(&mut self) {
        // Adjust colors for higher contrast
        self.colors.border_active = SerializableColor(Color::White);
        self.colors.selection_bg = SerializableColor(Color::White);
        self.colors.selection_fg = SerializableColor(Color::Black);
        
        // Make text more prominent
        self.styles.text = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
    }

    /// Disable animations for accessibility
    fn disable_animations(&mut self) {
        // Remove any animation-related modifiers
        // This would be used if we had animation support
    }

    /// Get theme configuration for saving
    pub fn to_config(&self) -> ThemeConfig {
        ThemeConfig {
            variant: self.variant.clone(),
            custom_colors: None, // Could be extended for custom color overrides
        }
    }

    /// Load theme from configuration
    pub fn from_config(config: &ThemeConfig) -> Self {
        let mut theme = Self::new(config.variant.clone());
        
        // Apply custom colors if provided
        if let Some(custom_colors) = &config.custom_colors {
            theme.apply_custom_colors(custom_colors);
        }
        
        theme
    }

    /// Apply custom color overrides
    fn apply_custom_colors(&mut self, _custom_colors: &CustomColors) {
        // TODO: Implement custom color application
    }
}

/// Theme configuration for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub variant: ThemeVariant,
    pub custom_colors: Option<CustomColors>,
}

/// Custom color definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColors {
    // Allow users to override specific colors
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub accent: Option<String>,
    // ... could be extended for all colors
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(ThemeVariant::Dark)
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            variant: ThemeVariant::Dark,
            custom_colors: None,
        }
    }
}

/// Parse color from string (hex or named)
fn parse_color(color_str: &str) -> Option<Color> {
    match color_str.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => {
            // Try to parse as hex color
            if color_str.starts_with('#') && color_str.len() == 7 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&color_str[1..3], 16),
                    u8::from_str_radix(&color_str[3..5], 16),
                    u8::from_str_radix(&color_str[5..7], 16),
                ) {
                    Some(Color::Rgb(r, g, b))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

/// Get available theme variants
pub fn available_themes() -> Vec<ThemeVariant> {
    vec![ThemeVariant::Dark, ThemeVariant::Light, ThemeVariant::Solarized]
}

/// Get theme variant by name
pub fn theme_variant_by_name(name: &str) -> Option<ThemeVariant> {
    match name.to_lowercase().as_str() {
        "dark" => Some(ThemeVariant::Dark),
        "light" => Some(ThemeVariant::Light),
        "solarized" => Some(ThemeVariant::Solarized),
        _ => None,
    }
}

/// Theme manager for handling theme switching and persistence
pub struct ThemeManager {
    current_theme: Theme,
    config_path: std::path::PathBuf,
}

impl ThemeManager {
    /// Create new theme manager
    pub fn new(config_dir: &std::path::Path) -> Self {
        let config_path = config_dir.join("theme.toml");
        let current_theme = if config_path.exists() {
            Self::load_theme_config(&config_path)
                .unwrap_or_else(|_| Theme::default())
        } else {
            Theme::default()
        };

        Self {
            current_theme,
            config_path,
        }
    }

    /// Get current theme
    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }

    /// Switch theme variant
    pub fn switch_theme(&mut self, variant: ThemeVariant) -> anyhow::Result<()> {
        self.current_theme.switch_variant(variant);
        self.save_theme_config()
    }

    /// Apply accessibility settings
    pub fn apply_accessibility(&mut self, high_contrast: bool, reduce_motion: bool) -> anyhow::Result<()> {
        self.current_theme.apply_accessibility(high_contrast, reduce_motion);
        self.save_theme_config()
    }

    /// Save theme configuration
    fn save_theme_config(&self) -> anyhow::Result<()> {
        let config = self.current_theme.to_config();
        let config_str = toml::to_string_pretty(&config)?;
        std::fs::write(&self.config_path, config_str)?;
        Ok(())
    }

    /// Load theme configuration
    fn load_theme_config(config_path: &std::path::Path) -> anyhow::Result<Theme> {
        let config_str = std::fs::read_to_string(config_path)?;
        let config: ThemeConfig = toml::from_str(&config_str)?;
        Ok(Theme::from_config(&config))
    }
}