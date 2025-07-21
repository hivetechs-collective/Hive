//! Theme management for VS Code-style theming
//!
//! Supports dark, light, and high contrast themes with dynamic switching

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    HighContrast,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

impl Theme {
    /// Get the CSS overrides for this theme
    pub fn css_overrides(&self) -> &'static str {
        match self {
            Self::Dark => "", // Default theme, no overrides needed
            Self::Light => super::themes::LIGHT_THEME_OVERRIDES,
            Self::HighContrast => super::themes::HIGH_CONTRAST_THEME_OVERRIDES,
        }
    }

    /// Get the theme name for display
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::HighContrast => "High Contrast",
        }
    }

    /// Get the theme icon
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Dark => "moon",
            Self::Light => "sun",
            Self::HighContrast => "adjust",
        }
    }
}

/// Theme colors for UI components
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeColors {
    pub background: String,
    pub background_secondary: String,
    pub text: String,
    pub text_secondary: String,
    pub primary: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub border: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl ThemeColors {
    pub fn dark_theme() -> Self {
        Self {
            background: "#1e1e1e".to_string(),
            background_secondary: "#252526".to_string(),
            text: "#cccccc".to_string(),
            text_secondary: "#999999".to_string(),
            primary: "#0e639c".to_string(),
            success: "#73c991".to_string(),
            warning: "#e9a23b".to_string(),
            error: "#f14c4c".to_string(),
            info: "#3794ff".to_string(),
            border: "#474747".to_string(),
        }
    }
    
    pub fn light_theme() -> Self {
        Self {
            background: "#ffffff".to_string(),
            background_secondary: "#f3f3f3".to_string(),
            text: "#333333".to_string(),
            text_secondary: "#666666".to_string(),
            primary: "#0066cc".to_string(),
            success: "#00a65a".to_string(),
            warning: "#f39c12".to_string(),
            error: "#dd4b39".to_string(),
            info: "#3c8dbc".to_string(),
            border: "#d4d4d4".to_string(),
        }
    }
    
    pub fn high_contrast_theme() -> Self {
        Self {
            background: "#000000".to_string(),
            background_secondary: "#2d2d30".to_string(),
            text: "#ffffff".to_string(),
            text_secondary: "#cccccc".to_string(),
            primary: "#00bcd4".to_string(),
            success: "#00e676".to_string(),
            warning: "#ffea00".to_string(),
            error: "#ff5252".to_string(),
            info: "#448aff".to_string(),
            border: "#6fc3df".to_string(),
        }
    }
    
    pub fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self::dark_theme(),
            Theme::Light => Self::light_theme(),
            Theme::HighContrast => Self::high_contrast_theme(),
        }
    }
}

/// Theme context provider
#[component]
pub fn ThemeProvider(children: Element) -> Element {
    let theme = use_signal(|| Theme::default());

    use_context_provider(|| theme);

    // Inject theme-specific styles
    let theme_styles = theme.read().css_overrides();

    rsx! {
        style {
            dangerous_inner_html: "{theme_styles}"
        }
        {children}
    }
}

/// Hook to access the current theme
pub fn use_theme() -> Signal<Theme> {
    use_context::<Signal<Theme>>()
}

/// Theme switcher component
#[component]
pub fn ThemeSwitcher() -> Element {
    let mut theme = use_theme();
    let theme_icon = theme.read().icon();

    rsx! {
        div {
            class: "theme-switcher",
            button {
                class: "btn btn-icon",
                title: "Switch theme",
                onclick: move |_| {
                    let current = *theme.read();
                    let next_theme = match current {
                        Theme::Dark => Theme::Light,
                        Theme::Light => Theme::HighContrast,
                        Theme::HighContrast => Theme::Dark,
                    };
                    theme.set(next_theme);
                },
                i { class: "fa fa-{theme_icon}" }
            }
        }
    }
}

/// Apply theme-aware classes
pub fn themed_class(base: &str, theme: Theme) -> String {
    match theme {
        Theme::Dark => format!("{} dark", base),
        Theme::Light => format!("{} light", base),
        Theme::HighContrast => format!("{} high-contrast", base),
    }
}

/// Get platform-specific adjustments
pub fn platform_adjustments() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        r#"
        /* macOS specific adjustments */
        .titlebar { -webkit-app-region: drag; }
        .titlebar button { -webkit-app-region: no-drag; }
        "#
    }

    #[cfg(target_os = "windows")]
    {
        r#"
        /* Windows specific adjustments */
        ::-webkit-scrollbar { width: 17px; height: 17px; }
        "#
    }

    #[cfg(target_os = "linux")]
    {
        r#"
        /* Linux specific adjustments */
        * { scrollbar-width: thin; }
        "#
    }
}

/// Load user theme preference from config
pub async fn load_theme_preference() -> Theme {
    // TODO: Load from user config
    Theme::Dark
}

/// Save user theme preference
pub async fn save_theme_preference(theme: Theme) {
    // TODO: Save to user config
    tracing::info!("Saving theme preference: {:?}", theme);
}
