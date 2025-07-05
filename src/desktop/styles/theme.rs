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