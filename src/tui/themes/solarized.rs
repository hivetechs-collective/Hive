//! Solarized theme implementation
//!
//! Professional Solarized theme (dark variant) with excellent readability

use ratatui::style::{Color, Style, Modifier};
use super::{Theme, ThemeVariant, ColorPalette, StyleDefinitions, SerializableColor};

/// Create Solarized dark theme
pub fn create_solarized_theme() -> Theme {
    // Official Solarized color palette
    let solarized = SolarizedPalette::new();
    
    let colors = ColorPalette {
        // Base colors using Solarized palette
        primary: SerializableColor(solarized.blue),
        secondary: SerializableColor(solarized.base02),
        background: SerializableColor(solarized.base03),
        foreground: SerializableColor(solarized.base0),
        accent: SerializableColor(solarized.cyan),
        
        // Status colors
        error: SerializableColor(solarized.red),
        warning: SerializableColor(solarized.yellow),
        success: SerializableColor(solarized.green),
        info: SerializableColor(solarized.blue),
        muted: SerializableColor(solarized.base01),
        
        // UI element colors
        border_active: SerializableColor(solarized.blue),
        border_inactive: SerializableColor(solarized.base01),
        selection_bg: SerializableColor(solarized.base02),
        selection_fg: SerializableColor(solarized.base1),
        
        // Syntax highlighting colors (Solarized)
        keyword: SerializableColor(solarized.blue),
        string: SerializableColor(solarized.cyan),
        comment: SerializableColor(solarized.base01),
        function: SerializableColor(solarized.yellow),
        variable: SerializableColor(solarized.base0),
        number: SerializableColor(solarized.magenta),
        operator: SerializableColor(solarized.green),
        
        // File explorer colors
        directory: SerializableColor(solarized.blue),
        file: SerializableColor(solarized.base0),
        symlink: SerializableColor(solarized.cyan),
        executable: SerializableColor(solarized.green),
        
        // Terminal colors
        command: SerializableColor(solarized.blue),
        output: SerializableColor(solarized.base0),
        system_message: SerializableColor(solarized.green),
        line_number: SerializableColor(solarized.base01),
        
        // Editor colors
        heading: SerializableColor(solarized.orange),
        code_block: SerializableColor(solarized.base01),
    };

    let styles = StyleDefinitions {
        title_bar: Style::default()
            .bg(solarized.base02)
            .fg(solarized.base0)
            .add_modifier(Modifier::BOLD),
        
        status_bar: Style::default()
            .bg(solarized.blue)
            .fg(solarized.base03),
        
        panel: Style::default()
            .bg(solarized.base03)
            .fg(solarized.base0),
        
        active_border: Style::default()
            .fg(solarized.blue)
            .add_modifier(Modifier::BOLD),
        
        inactive_border: Style::default()
            .fg(solarized.base01),
        
        selection: Style::default()
            .bg(solarized.base02)
            .fg(solarized.base1)
            .add_modifier(Modifier::BOLD),
        
        tab: Style::default()
            .bg(solarized.base02)
            .fg(solarized.base01),
        
        active_tab: Style::default()
            .bg(solarized.base03)
            .fg(solarized.base0)
            .add_modifier(Modifier::BOLD),
        
        editor: Style::default()
            .bg(solarized.base03)
            .fg(solarized.base0),
        
        terminal: Style::default()
            .bg(solarized.base03)
            .fg(solarized.base0),
        
        terminal_output: Style::default()
            .fg(solarized.base0),
        
        popup: Style::default()
            .bg(solarized.base02)
            .fg(solarized.base0)
            .add_modifier(Modifier::BOLD),
        
        input_active: Style::default()
            .bg(solarized.base02)
            .fg(solarized.base1)
            .add_modifier(Modifier::BOLD),
        
        input_inactive: Style::default()
            .bg(solarized.base02)
            .fg(solarized.base01),
        
        text: Style::default()
            .fg(solarized.base0),
    };

    Theme {
        variant: ThemeVariant::Solarized,
        colors,
        styles,
    }
}

/// Official Solarized color palette
struct SolarizedPalette {
    // Background tones (dark theme)
    base03: Color,  // Background
    base02: Color,  // Background highlights
    base01: Color,  // Comments/secondary content
    base00: Color,  // Body text/default code
    
    // Foreground tones
    base0: Color,   // Primary content
    base1: Color,   // Optional emphasized content
    base2: Color,   // Background (light theme)
    base3: Color,   // Background highlights (light theme)
    
    // Accent colors
    yellow: Color,
    orange: Color,
    red: Color,
    magenta: Color,
    violet: Color,
    blue: Color,
    cyan: Color,
    green: Color,
}

impl SolarizedPalette {
    fn new() -> Self {
        Self {
            // Background tones
            base03: Color::Rgb(0, 43, 54),      // #002b36
            base02: Color::Rgb(7, 54, 66),      // #073642
            base01: Color::Rgb(88, 110, 117),   // #586e75
            base00: Color::Rgb(101, 123, 131),  // #657b83
            
            // Foreground tones
            base0: Color::Rgb(131, 148, 150),   // #839496
            base1: Color::Rgb(147, 161, 161),   // #93a1a1
            base2: Color::Rgb(238, 232, 213),   // #eee8d5
            base3: Color::Rgb(253, 246, 227),   // #fdf6e3
            
            // Accent colors
            yellow: Color::Rgb(181, 137, 0),    // #b58900
            orange: Color::Rgb(203, 75, 22),    // #cb4b16
            red: Color::Rgb(220, 50, 47),       // #dc322f
            magenta: Color::Rgb(211, 54, 130),  // #d33682
            violet: Color::Rgb(108, 113, 196),  // #6c71c4
            blue: Color::Rgb(38, 139, 210),     // #268bd2
            cyan: Color::Rgb(42, 161, 152),     // #2aa198
            green: Color::Rgb(133, 153, 0),     // #859900
        }
    }
}

/// User preferences for Solarized theme customization
pub struct SolarizedThemePreferences {
    pub use_light_variant: bool,
    pub enhanced_contrast: bool,
    pub custom_accent: Option<Color>,
}

/// Apply Solarized theme customizations
pub fn customize_solarized_theme(base_theme: &mut Theme, preferences: &SolarizedThemePreferences) {
    if preferences.use_light_variant {
        apply_solarized_light_variant(base_theme);
    }
    
    if preferences.enhanced_contrast {
        apply_enhanced_contrast_solarized(base_theme);
    }
    
    if let Some(accent_color) = preferences.custom_accent {
        base_theme.colors.accent = accent_color;
        base_theme.colors.primary = accent_color;
        base_theme.colors.border_active = accent_color;
    }
}

/// Convert to Solarized light variant
fn apply_solarized_light_variant(theme: &mut Theme) {
    let solarized = SolarizedPalette::new();
    
    // Swap background and foreground tones for light theme
    theme.colors.background = solarized.base3;
    theme.colors.secondary = solarized.base2;
    theme.colors.foreground = solarized.base00;
    theme.colors.muted = solarized.base1;
    
    // Update styles for light variant
    theme.styles.panel = theme.styles.panel.bg(solarized.base3).fg(solarized.base00);
    theme.styles.editor = theme.styles.editor.bg(solarized.base3).fg(solarized.base00);
    theme.styles.terminal = theme.styles.terminal.bg(solarized.base3).fg(solarized.base00);
    theme.styles.title_bar = theme.styles.title_bar.bg(solarized.base2).fg(solarized.base00);
}

/// Apply enhanced contrast for Solarized theme
fn apply_enhanced_contrast_solarized(theme: &mut Theme) {
    let solarized = SolarizedPalette::new();
    
    // Use more contrasting colors
    theme.colors.foreground = solarized.base1;
    theme.colors.selection_fg = solarized.base3;
    
    // Enhance border visibility
    theme.styles.active_border = theme.styles.active_border
        .fg(solarized.cyan)
        .add_modifier(Modifier::BOLD);
}

/// Get Solarized color palette for reference
pub fn get_solarized_palette() -> Vec<(&'static str, Color)> {
    let s = SolarizedPalette::new();
    vec![
        ("Base03 (Background)", s.base03),
        ("Base02 (Highlight)", s.base02),
        ("Base01 (Comments)", s.base01),
        ("Base00 (Body Text)", s.base00),
        ("Base0 (Primary)", s.base0),
        ("Base1 (Emphasized)", s.base1),
        ("Yellow", s.yellow),
        ("Orange", s.orange),
        ("Red", s.red),
        ("Magenta", s.magenta),
        ("Violet", s.violet),
        ("Blue", s.blue),
        ("Cyan", s.cyan),
        ("Green", s.green),
    ]
}

/// Validate Solarized theme adherence
pub fn validate_solarized_adherence(theme: &Theme) -> bool {
    let solarized = SolarizedPalette::new();
    
    // Check if key colors match Solarized palette
    theme.colors.background == solarized.base03 &&
    theme.colors.foreground == solarized.base0 &&
    theme.colors.primary == solarized.blue
}

impl Default for SolarizedThemePreferences {
    fn default() -> Self {
        Self {
            use_light_variant: false,
            enhanced_contrast: false,
            custom_accent: None,
        }
    }
}