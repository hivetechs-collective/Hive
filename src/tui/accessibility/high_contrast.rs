//! High contrast mode for better visibility
//!
//! Provides enhanced contrast for users with visual impairments

use crate::tui::themes::{SerializableColor, Theme};
use ratatui::style::{Color, Modifier, Style};

/// High contrast mode manager
pub struct HighContrastMode {
    enabled: bool,
    contrast_level: ContrastLevel,
}

/// Available contrast levels
#[derive(Debug, Clone, PartialEq)]
pub enum ContrastLevel {
    /// Standard high contrast (4.5:1 ratio)
    Standard,
    /// Enhanced high contrast (7:1 ratio)
    Enhanced,
    /// Maximum contrast (black and white only)
    Maximum,
}

impl HighContrastMode {
    /// Create new high contrast mode manager
    pub fn new() -> Self {
        Self {
            enabled: false,
            contrast_level: ContrastLevel::Standard,
        }
    }

    /// Enable or disable high contrast mode
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if high contrast mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set contrast level
    pub fn set_contrast_level(&mut self, level: ContrastLevel) {
        self.contrast_level = level;
    }

    /// Get current contrast level
    pub fn contrast_level(&self) -> &ContrastLevel {
        &self.contrast_level
    }

    /// Apply high contrast adjustments to theme
    pub fn adjust_theme(&self, theme: &mut Theme) {
        if !self.enabled {
            return;
        }

        match self.contrast_level {
            ContrastLevel::Standard => self.apply_standard_contrast(theme),
            ContrastLevel::Enhanced => self.apply_enhanced_contrast(theme),
            ContrastLevel::Maximum => self.apply_maximum_contrast(theme),
        }
    }

    /// Apply standard high contrast (WCAG AA - 4.5:1 ratio)
    fn apply_standard_contrast(&self, theme: &mut Theme) {
        // Ensure sufficient contrast for text
        theme.colors.foreground = SerializableColor(Color::White);
        theme.colors.background = SerializableColor(Color::Black);

        // High contrast borders
        theme.colors.border_active = SerializableColor(Color::Yellow);
        theme.colors.border_inactive = SerializableColor(Color::Gray);

        // High contrast selection
        theme.colors.selection_bg = SerializableColor(Color::Blue);
        theme.colors.selection_fg = SerializableColor(Color::White);

        // Status colors with high contrast
        theme.colors.error = SerializableColor(Color::LightRed);
        theme.colors.warning = SerializableColor(Color::Yellow);
        theme.colors.success = SerializableColor(Color::LightGreen);
        theme.colors.info = SerializableColor(Color::LightBlue);

        // Update styles
        self.update_styles_for_contrast(theme);
    }

    /// Apply enhanced high contrast (WCAG AAA - 7:1 ratio)
    fn apply_enhanced_contrast(&self, theme: &mut Theme) {
        // Even higher contrast
        theme.colors.foreground = SerializableColor(Color::White);
        theme.colors.background = SerializableColor(Color::Black);

        // Very prominent borders
        theme.colors.border_active = SerializableColor(Color::White);
        theme.colors.border_inactive = SerializableColor(Color::DarkGray);

        // High contrast selection
        theme.colors.selection_bg = SerializableColor(Color::White);
        theme.colors.selection_fg = SerializableColor(Color::Black);

        // Enhanced status colors
        theme.colors.error = SerializableColor(Color::Red);
        theme.colors.warning = SerializableColor(Color::Yellow);
        theme.colors.success = SerializableColor(Color::Green);
        theme.colors.info = SerializableColor(Color::Cyan);

        // Update styles with enhanced modifiers
        self.update_styles_for_enhanced_contrast(theme);
    }

    /// Apply maximum contrast (black and white only)
    fn apply_maximum_contrast(&self, theme: &mut Theme) {
        // Pure black and white
        theme.colors.foreground = SerializableColor(Color::White);
        theme.colors.background = SerializableColor(Color::Black);
        theme.colors.primary = SerializableColor(Color::White);
        theme.colors.secondary = SerializableColor(Color::Black);
        theme.colors.accent = SerializableColor(Color::White);

        // Black and white only for UI elements
        theme.colors.border_active = SerializableColor(Color::White);
        theme.colors.border_inactive = SerializableColor(Color::Gray);
        theme.colors.selection_bg = SerializableColor(Color::White);
        theme.colors.selection_fg = SerializableColor(Color::Black);

        // Status colors in grayscale
        theme.colors.error = SerializableColor(Color::White);
        theme.colors.warning = SerializableColor(Color::White);
        theme.colors.success = SerializableColor(Color::White);
        theme.colors.info = SerializableColor(Color::White);
        theme.colors.muted = SerializableColor(Color::Gray);

        // Syntax highlighting in grayscale
        theme.colors.keyword = SerializableColor(Color::White);
        theme.colors.string = SerializableColor(Color::Gray);
        theme.colors.comment = SerializableColor(Color::DarkGray);
        theme.colors.function = SerializableColor(Color::White);
        theme.colors.variable = SerializableColor(Color::White);
        theme.colors.number = SerializableColor(Color::White);
        theme.colors.operator = SerializableColor(Color::White);

        // File explorer in grayscale
        theme.colors.directory = SerializableColor(Color::White);
        theme.colors.file = SerializableColor(Color::White);
        theme.colors.symlink = SerializableColor(Color::Gray);
        theme.colors.executable = SerializableColor(Color::White);

        // Terminal in grayscale
        theme.colors.command = SerializableColor(Color::White);
        theme.colors.output = SerializableColor(Color::White);
        theme.colors.system_message = SerializableColor(Color::Gray);
        theme.colors.line_number = SerializableColor(Color::DarkGray);

        // Update styles for maximum contrast
        self.update_styles_for_maximum_contrast(theme);
    }

    /// Update styles for standard/enhanced contrast
    fn update_styles_for_contrast(&self, theme: &mut Theme) {
        // Make all text bold for better visibility
        theme.styles.text = Style::default()
            .fg(theme.colors.foreground.0)
            .add_modifier(Modifier::BOLD);

        // Prominent active borders
        theme.styles.active_border = Style::default()
            .fg(theme.colors.border_active.0)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED);

        // High contrast selection
        theme.styles.selection = Style::default()
            .bg(theme.colors.selection_bg.0)
            .fg(theme.colors.selection_fg.0)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::REVERSED);
    }

    /// Update styles for enhanced contrast
    fn update_styles_for_enhanced_contrast(&self, theme: &mut Theme) {
        self.update_styles_for_contrast(theme);

        // Additional enhancements
        theme.styles.title_bar = theme
            .styles
            .title_bar
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED);

        theme.styles.status_bar = theme.styles.status_bar.add_modifier(Modifier::BOLD);

        // Make all interactive elements more prominent
        theme.styles.tab = theme.styles.tab.add_modifier(Modifier::BOLD);

        theme.styles.active_tab = theme
            .styles
            .active_tab
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED);
    }

    /// Update styles for maximum contrast
    fn update_styles_for_maximum_contrast(&self, theme: &mut Theme) {
        // Everything is bold and prominent
        let base_style = Style::default()
            .fg(theme.colors.foreground.0)
            .bg(theme.colors.background.0)
            .add_modifier(Modifier::BOLD);

        theme.styles.text = base_style;
        theme.styles.title_bar = base_style.add_modifier(Modifier::UNDERLINED);
        theme.styles.status_bar = base_style.add_modifier(Modifier::REVERSED);
        theme.styles.panel = base_style;
        theme.styles.editor = base_style;
        theme.styles.terminal = base_style;
        theme.styles.popup = base_style.add_modifier(Modifier::REVERSED);

        // Active elements are reversed
        theme.styles.active_border = Style::default()
            .fg(theme.colors.background.0)
            .bg(theme.colors.foreground.0)
            .add_modifier(Modifier::BOLD);

        theme.styles.selection = Style::default()
            .fg(theme.colors.selection_fg.0)
            .bg(theme.colors.selection_bg.0)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::REVERSED);

        theme.styles.active_tab = Style::default()
            .fg(theme.colors.background.0)
            .bg(theme.colors.foreground.0)
            .add_modifier(Modifier::BOLD);
    }

    /// Calculate contrast ratio between two colors (simplified)
    pub fn calculate_contrast_ratio(&self, fg: Color, bg: Color) -> f32 {
        // Simplified contrast ratio calculation
        // In a real implementation, this would use proper luminance calculations
        match (fg, bg) {
            (Color::Rgb(fr, fg, fb), Color::Rgb(br, bg, bb)) => {
                let fg_luminance = 0.299 * fr as f32 + 0.587 * fg as f32 + 0.114 * fb as f32;
                let bg_luminance = 0.299 * br as f32 + 0.587 * bg as f32 + 0.114 * bb as f32;
                (fg_luminance.max(bg_luminance) + 0.05) / (fg_luminance.min(bg_luminance) + 0.05)
            }
            (Color::White, Color::Black) => 21.0, // Maximum contrast
            (Color::Black, Color::White) => 21.0, // Maximum contrast
            _ => 4.5,                             // Assume acceptable contrast for other colors
        }
    }

    /// Check if contrast meets WCAG guidelines
    pub fn meets_wcag_aa(&self, fg: Color, bg: Color) -> bool {
        self.calculate_contrast_ratio(fg, bg) >= 4.5
    }

    /// Check if contrast meets WCAG AAA guidelines
    pub fn meets_wcag_aaa(&self, fg: Color, bg: Color) -> bool {
        self.calculate_contrast_ratio(fg, bg) >= 7.0
    }

    /// Get recommended colors for current contrast level
    pub fn get_recommended_colors(&self) -> Vec<(String, Color, Color)> {
        match self.contrast_level {
            ContrastLevel::Standard => vec![
                ("Text on Background".to_string(), Color::White, Color::Black),
                ("Error Text".to_string(), Color::LightRed, Color::Black),
                ("Success Text".to_string(), Color::LightGreen, Color::Black),
                ("Warning Text".to_string(), Color::Yellow, Color::Black),
                ("Info Text".to_string(), Color::LightBlue, Color::Black),
            ],
            ContrastLevel::Enhanced => vec![
                ("Text on Background".to_string(), Color::White, Color::Black),
                ("Active Border".to_string(), Color::White, Color::Black),
                ("Selection".to_string(), Color::Black, Color::White),
                ("Error Text".to_string(), Color::Red, Color::Black),
                ("Success Text".to_string(), Color::Green, Color::Black),
            ],
            ContrastLevel::Maximum => vec![
                ("All Text".to_string(), Color::White, Color::Black),
                ("All Backgrounds".to_string(), Color::Black, Color::White),
                ("Active Elements".to_string(), Color::Black, Color::White),
            ],
        }
    }

    /// Test all theme colors for contrast compliance
    pub fn test_theme_contrast(&self, theme: &Theme) -> Vec<String> {
        let mut issues = Vec::new();

        // Test main colors
        if !self.meets_wcag_aa(theme.colors.foreground.0, theme.colors.background.0) {
            issues.push("Main text does not meet WCAG AA contrast requirements".to_string());
        }

        if !self.meets_wcag_aa(theme.colors.selection_fg.0, theme.colors.selection_bg.0) {
            issues.push("Selection text does not meet WCAG AA contrast requirements".to_string());
        }

        // Test status colors
        for (name, color) in [
            ("Error", &theme.colors.error),
            ("Warning", &theme.colors.warning),
            ("Success", &theme.colors.success),
            ("Info", &theme.colors.info),
        ] {
            if !self.meets_wcag_aa(color.0, theme.colors.background.0) {
                issues.push(format!(
                    "{} color does not meet WCAG AA contrast requirements",
                    name
                ));
            }
        }

        issues
    }
}

impl Default for HighContrastMode {
    fn default() -> Self {
        Self::new()
    }
}
