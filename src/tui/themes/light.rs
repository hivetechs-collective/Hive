//! Light theme implementation
//!
//! Professional light theme inspired by VS Code Light+ theme

use ratatui::style::{Color, Style, Modifier};
use super::{Theme, ThemeVariant, ColorPalette, StyleDefinitions};

/// Create professional light theme
pub fn create_light_theme() -> Theme {
    let colors = ColorPalette {
        // Base colors - VS Code Light+ inspired
        primary: Color::Rgb(0, 122, 204),      // VS Code blue
        secondary: Color::Rgb(248, 248, 248),  // Light background
        background: Color::Rgb(255, 255, 255), // White background
        foreground: Color::Rgb(51, 51, 51),    // Dark text
        accent: Color::Rgb(14, 99, 156),       // Accent blue
        
        // Status colors
        error: Color::Rgb(205, 49, 49),        // Red
        warning: Color::Rgb(255, 143, 0),      // Orange
        success: Color::Rgb(40, 128, 90),      // Green
        info: Color::Rgb(91, 168, 225),        // Light blue
        muted: Color::Rgb(150, 150, 150),      // Gray
        
        // UI element colors
        border_active: Color::Rgb(0, 122, 204),   // Active blue border
        border_inactive: Color::Rgb(200, 200, 200), // Inactive gray border
        selection_bg: Color::Rgb(173, 214, 255),   // Light blue selection
        selection_fg: Color::Rgb(0, 0, 0),         // Black selection text
        
        // Syntax highlighting colors (VS Code Light+ theme)
        keyword: Color::Rgb(0, 0, 255),           // Blue
        string: Color::Rgb(163, 21, 21),          // Red
        comment: Color::Rgb(0, 128, 0),           // Green
        function: Color::Rgb(121, 94, 38),        // Brown
        variable: Color::Rgb(1, 84, 128),         // Dark blue
        number: Color::Rgb(9, 134, 88),           // Dark green
        operator: Color::Rgb(51, 51, 51),         // Dark gray
        
        // File explorer colors
        directory: Color::Rgb(0, 0, 255),         // Blue
        file: Color::Rgb(51, 51, 51),             // Dark gray
        symlink: Color::Rgb(255, 143, 0),         // Orange
        executable: Color::Rgb(40, 128, 90),      // Green
        
        // Terminal colors
        command: Color::Rgb(0, 0, 255),           // Blue
        output: Color::Rgb(51, 51, 51),           // Dark gray
        system_message: Color::Rgb(0, 128, 0),    // Green
        line_number: Color::Rgb(150, 150, 150),   // Gray
        
        // Editor colors
        heading: Color::Rgb(0, 0, 255),           // Blue
        code_block: Color::Rgb(150, 150, 150),    // Gray
    };

    let styles = StyleDefinitions {
        title_bar: Style::default()
            .bg(Color::Rgb(248, 248, 248))
            .fg(Color::Rgb(51, 51, 51))
            .add_modifier(Modifier::BOLD),
        
        status_bar: Style::default()
            .bg(Color::Rgb(0, 122, 204))
            .fg(Color::Rgb(255, 255, 255)),
        
        panel: Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(51, 51, 51)),
        
        active_border: Style::default()
            .fg(Color::Rgb(0, 122, 204))
            .add_modifier(Modifier::BOLD),
        
        inactive_border: Style::default()
            .fg(Color::Rgb(200, 200, 200)),
        
        selection: Style::default()
            .bg(Color::Rgb(173, 214, 255))
            .fg(Color::Rgb(0, 0, 0))
            .add_modifier(Modifier::BOLD),
        
        tab: Style::default()
            .bg(Color::Rgb(230, 230, 230))
            .fg(Color::Rgb(100, 100, 100)),
        
        active_tab: Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(0, 0, 0))
            .add_modifier(Modifier::BOLD),
        
        editor: Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(51, 51, 51)),
        
        terminal: Style::default()
            .bg(Color::Rgb(250, 250, 250))
            .fg(Color::Rgb(51, 51, 51)),
        
        terminal_output: Style::default()
            .fg(Color::Rgb(51, 51, 51)),
        
        popup: Style::default()
            .bg(Color::Rgb(240, 240, 240))
            .fg(Color::Rgb(51, 51, 51))
            .add_modifier(Modifier::BOLD),
        
        input_active: Style::default()
            .bg(Color::Rgb(248, 248, 248))
            .fg(Color::Rgb(0, 0, 0))
            .add_modifier(Modifier::BOLD),
        
        input_inactive: Style::default()
            .bg(Color::Rgb(240, 240, 240))
            .fg(Color::Rgb(100, 100, 100)),
        
        text: Style::default()
            .fg(Color::Rgb(51, 51, 51)),
    };

    Theme {
        variant: ThemeVariant::Light,
        colors,
        styles,
    }
}

/// User preferences for light theme customization
pub struct LightThemePreferences {
    pub high_contrast: bool,
    pub reduce_glare: bool,
    pub custom_accent: Option<Color>,
}

/// Apply light theme customizations based on user preferences
pub fn customize_light_theme(base_theme: &mut Theme, preferences: &LightThemePreferences) {
    if preferences.high_contrast {
        apply_high_contrast_light(base_theme);
    }
    
    if preferences.reduce_glare {
        apply_glare_reduction(base_theme);
    }
    
    if let Some(accent_color) = preferences.custom_accent {
        base_theme.colors.accent = accent_color;
        base_theme.colors.primary = accent_color;
        base_theme.colors.border_active = accent_color;
    }
}

/// Apply high contrast modifications to light theme
fn apply_high_contrast_light(theme: &mut Theme) {
    theme.colors.foreground = Color::Black;
    theme.colors.background = Color::White;
    theme.colors.border_active = Color::Black;
    theme.colors.selection_bg = Color::Black;
    theme.colors.selection_fg = Color::White;
    
    // Make all styles more prominent
    theme.styles.text = theme.styles.text.add_modifier(Modifier::BOLD);
    theme.styles.active_border = theme.styles.active_border.fg(Color::Black);
}

/// Apply glare reduction to light theme
fn apply_glare_reduction(theme: &mut Theme) {
    // Use slightly off-white backgrounds to reduce glare
    theme.colors.background = Color::Rgb(250, 250, 250);
    theme.colors.secondary = Color::Rgb(245, 245, 245);
    
    // Adjust styles to use softer backgrounds
    theme.styles.panel = theme.styles.panel.bg(Color::Rgb(250, 250, 250));
    theme.styles.editor = theme.styles.editor.bg(Color::Rgb(250, 250, 250));
}

impl Default for LightThemePreferences {
    fn default() -> Self {
        Self {
            high_contrast: false,
            reduce_glare: true, // Default to reduced glare for comfort
            custom_accent: None,
        }
    }
}