//! Dark theme implementation
//!
//! Professional dark theme inspired by VS Code Dark+ theme

use ratatui::style::{Color, Style, Modifier};
use super::{Theme, ThemeVariant, ColorPalette, StyleDefinitions};

/// Create professional dark theme
pub fn create_dark_theme() -> Theme {
    let colors = ColorPalette {
        // Base colors - VS Code Dark+ inspired
        primary: Color::Rgb(0, 122, 204),      // VS Code blue
        secondary: Color::Rgb(37, 37, 38),     // Dark background
        background: Color::Rgb(30, 30, 30),    // Editor background
        foreground: Color::Rgb(204, 204, 204), // Default text
        accent: Color::Rgb(14, 99, 156),       // Accent blue
        
        // Status colors
        error: Color::Rgb(244, 71, 71),        // Red
        warning: Color::Rgb(255, 196, 0),      // Yellow/Orange
        success: Color::Rgb(73, 177, 146),     // Green
        info: Color::Rgb(91, 168, 225),        // Light blue
        muted: Color::Rgb(96, 96, 96),         // Gray
        
        // UI element colors
        border_active: Color::Rgb(0, 122, 204),   // Active blue border
        border_inactive: Color::Rgb(60, 60, 60),  // Inactive gray border
        selection_bg: Color::Rgb(38, 79, 120),    // Selection background
        selection_fg: Color::Rgb(255, 255, 255),  // Selection text
        
        // Syntax highlighting colors (VS Code Dark+ theme)
        keyword: Color::Rgb(86, 156, 214),        // Blue
        string: Color::Rgb(206, 145, 120),        // Orange
        comment: Color::Rgb(106, 153, 85),        // Green
        function: Color::Rgb(220, 220, 170),      // Light yellow
        variable: Color::Rgb(156, 220, 254),      // Light blue
        number: Color::Rgb(181, 206, 168),        // Light green
        operator: Color::Rgb(204, 204, 204),      // White
        
        // File explorer colors
        directory: Color::Rgb(86, 156, 214),      // Blue
        file: Color::Rgb(204, 204, 204),          // White
        symlink: Color::Rgb(255, 196, 0),         // Yellow
        executable: Color::Rgb(73, 177, 146),     // Green
        
        // Terminal colors
        command: Color::Rgb(86, 156, 214),        // Blue
        output: Color::Rgb(204, 204, 204),        // White
        system_message: Color::Rgb(106, 153, 85), // Green
        line_number: Color::Rgb(96, 96, 96),      // Gray
        
        // Editor colors
        heading: Color::Rgb(86, 156, 214),        // Blue
        code_block: Color::Rgb(96, 96, 96),       // Gray
    };

    let styles = StyleDefinitions {
        title_bar: Style::default()
            .bg(Color::Rgb(37, 37, 38))
            .fg(Color::Rgb(204, 204, 204))
            .add_modifier(Modifier::BOLD),
        
        status_bar: Style::default()
            .bg(Color::Rgb(0, 122, 204))
            .fg(Color::Rgb(255, 255, 255)),
        
        panel: Style::default()
            .bg(Color::Rgb(30, 30, 30))
            .fg(Color::Rgb(204, 204, 204)),
        
        active_border: Style::default()
            .fg(Color::Rgb(0, 122, 204))
            .add_modifier(Modifier::BOLD),
        
        inactive_border: Style::default()
            .fg(Color::Rgb(60, 60, 60)),
        
        selection: Style::default()
            .bg(Color::Rgb(38, 79, 120))
            .fg(Color::Rgb(255, 255, 255))
            .add_modifier(Modifier::BOLD),
        
        tab: Style::default()
            .bg(Color::Rgb(45, 45, 45))
            .fg(Color::Rgb(160, 160, 160)),
        
        active_tab: Style::default()
            .bg(Color::Rgb(30, 30, 30))
            .fg(Color::Rgb(255, 255, 255))
            .add_modifier(Modifier::BOLD),
        
        editor: Style::default()
            .bg(Color::Rgb(30, 30, 30))
            .fg(Color::Rgb(204, 204, 204)),
        
        terminal: Style::default()
            .bg(Color::Rgb(20, 20, 20))
            .fg(Color::Rgb(204, 204, 204)),
        
        terminal_output: Style::default()
            .fg(Color::Rgb(204, 204, 204)),
        
        popup: Style::default()
            .bg(Color::Rgb(45, 45, 45))
            .fg(Color::Rgb(204, 204, 204))
            .add_modifier(Modifier::BOLD),
        
        input_active: Style::default()
            .bg(Color::Rgb(37, 37, 38))
            .fg(Color::Rgb(255, 255, 255))
            .add_modifier(Modifier::BOLD),
        
        input_inactive: Style::default()
            .bg(Color::Rgb(45, 45, 45))
            .fg(Color::Rgb(160, 160, 160)),
        
        text: Style::default()
            .fg(Color::Rgb(204, 204, 204)),
    };

    Theme {
        variant: ThemeVariant::Dark,
        colors,
        styles,
    }
}

/// Get dark theme color recommendations for customization
pub fn get_dark_theme_palette() -> Vec<(&'static str, Color)> {
    vec![
        ("Primary Blue", Color::Rgb(0, 122, 204)),
        ("Background Dark", Color::Rgb(30, 30, 30)),
        ("Text Light", Color::Rgb(204, 204, 204)),
        ("Success Green", Color::Rgb(73, 177, 146)),
        ("Warning Yellow", Color::Rgb(255, 196, 0)),
        ("Error Red", Color::Rgb(244, 71, 71)),
        ("Info Blue", Color::Rgb(91, 168, 225)),
        ("Muted Gray", Color::Rgb(96, 96, 96)),
        ("Keyword Blue", Color::Rgb(86, 156, 214)),
        ("String Orange", Color::Rgb(206, 145, 120)),
        ("Comment Green", Color::Rgb(106, 153, 85)),
    ]
}

/// Validate dark theme contrast ratios
pub fn validate_contrast_ratios(theme: &Theme) -> Vec<String> {
    let mut issues = Vec::new();
    
    // Check if text has sufficient contrast against background
    if !has_sufficient_contrast(theme.colors.foreground, theme.colors.background) {
        issues.push("Foreground text may not have sufficient contrast against background".to_string());
    }
    
    // Check selection contrast
    if !has_sufficient_contrast(theme.colors.selection_fg, theme.colors.selection_bg) {
        issues.push("Selection text may not have sufficient contrast".to_string());
    }
    
    issues
}

/// Check if two colors have sufficient contrast (simplified)
fn has_sufficient_contrast(fg: Color, bg: Color) -> bool {
    // This is a simplified contrast check
    // In a real implementation, you would calculate the actual contrast ratio
    match (fg, bg) {
        (Color::Rgb(fr, fg, fb), Color::Rgb(br, bg, bb)) => {
            let fg_luminance = 0.299 * fr as f32 + 0.587 * fg as f32 + 0.114 * fb as f32;
            let bg_luminance = 0.299 * br as f32 + 0.587 * bg as f32 + 0.114 * bb as f32;
            let contrast = (fg_luminance.max(bg_luminance) + 0.05) / (fg_luminance.min(bg_luminance) + 0.05);
            contrast >= 4.5 // WCAG AA standard
        }
        _ => true, // For non-RGB colors, assume they're accessible
    }
}

/// Apply dark theme customizations based on user preferences
pub fn customize_dark_theme(base_theme: &mut Theme, preferences: &DarkThemePreferences) {
    if preferences.high_contrast {
        apply_high_contrast_dark(base_theme);
    }
    
    if preferences.blue_light_filter {
        apply_blue_light_filter(base_theme);
    }
    
    if let Some(accent_color) = preferences.custom_accent {
        base_theme.colors.accent = accent_color;
        base_theme.colors.primary = accent_color;
        base_theme.colors.border_active = accent_color;
    }
}

/// User preferences for dark theme customization
pub struct DarkThemePreferences {
    pub high_contrast: bool,
    pub blue_light_filter: bool,
    pub custom_accent: Option<Color>,
}

/// Apply high contrast modifications to dark theme
fn apply_high_contrast_dark(theme: &mut Theme) {
    theme.colors.foreground = Color::White;
    theme.colors.background = Color::Black;
    theme.colors.border_active = Color::White;
    theme.colors.selection_bg = Color::White;
    theme.colors.selection_fg = Color::Black;
    
    // Make all styles more prominent
    theme.styles.text = theme.styles.text.add_modifier(Modifier::BOLD);
    theme.styles.active_border = theme.styles.active_border.fg(Color::White);
}

/// Apply blue light filter to dark theme
fn apply_blue_light_filter(theme: &mut Theme) {
    // Reduce blue components in colors
    theme.colors.primary = Color::Rgb(100, 122, 150);  // Reduced blue
    theme.colors.accent = Color::Rgb(100, 99, 120);    // Reduced blue
    theme.colors.keyword = Color::Rgb(120, 156, 180);  // Reduced blue
    theme.colors.info = Color::Rgb(150, 168, 200);     // Reduced blue
}

impl Default for DarkThemePreferences {
    fn default() -> Self {
        Self {
            high_contrast: false,
            blue_light_filter: false,
            custom_accent: None,
        }
    }
}