//! Reusable Icon component for the desktop application
//! Supports VS Code Codicon icons with customizable size and color

use dioxus::prelude::*;

/// Icon size options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconSize {
    Small,   // 12x12
    Medium,  // 16x16 (default)
    Large,   // 20x20
    XLarge,  // 24x24
}

impl IconSize {
    /// Get the actual pixel dimensions for the icon size
    pub fn dimensions(&self) -> (u16, u16) {
        match self {
            IconSize::Small => (12, 12),
            IconSize::Medium => (16, 16),
            IconSize::Large => (20, 20),
            IconSize::XLarge => (24, 24),
        }
    }

    /// Get the size as a CSS string
    pub fn as_css(&self) -> &str {
        match self {
            IconSize::Small => "12px",
            IconSize::Medium => "16px",
            IconSize::Large => "20px",
            IconSize::XLarge => "24px",
        }
    }
}

impl Default for IconSize {
    fn default() -> Self {
        IconSize::Medium
    }
}

/// Icon component properties
#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    /// The SVG content of the icon
    pub svg: String,
    /// Icon size
    #[props(default = IconSize::default())]
    pub size: IconSize,
    /// Custom CSS class
    #[props(default)]
    pub class: Option<String>,
    /// Icon label for accessibility
    #[props(default)]
    pub label: Option<String>,
    /// Click handler
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

/// Icon component for rendering SVG icons
pub fn Icon(props: IconProps) -> Element {
    let size = props.size.as_css();
    let class_name = props.class.unwrap_or_default();
    let label = props.label.unwrap_or_else(|| "icon".to_string());
    
    rsx! {
        div {
            class: "icon-wrapper {class_name}",
            style: "width: {size}; height: {size}; display: inline-flex; align-items: center; justify-content: center;",
            "aria-label": "{label}",
            onclick: move |evt| {
                if let Some(handler) = &props.onclick {
                    handler.call(evt);
                }
            },
            div {
                dangerous_inner_html: "{props.svg}",
                style: "width: 100%; height: 100%;",
            }
        }
    }
}

/// Icon with label component properties
#[derive(Props, Clone, PartialEq)]
pub struct IconWithLabelProps {
    /// The SVG content of the icon
    pub svg: String,
    /// Label text
    pub text: String,
    /// Icon size
    #[props(default = IconSize::default())]
    pub icon_size: IconSize,
    /// Spacing between icon and text
    #[props(default = 4)]
    pub spacing: u8,
    /// Custom CSS class
    #[props(default)]
    pub class: Option<String>,
    /// Click handler
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

/// Icon with label component
pub fn IconWithLabel(props: IconWithLabelProps) -> Element {
    let class_name = props.class.unwrap_or_default();
    let spacing = format!("{}px", props.spacing);
    
    rsx! {
        div {
            class: "icon-with-label {class_name}",
            style: "display: inline-flex; align-items: center; gap: {spacing}; cursor: pointer;",
            onclick: move |evt| {
                if let Some(handler) = &props.onclick {
                    handler.call(evt);
                }
            },
            Icon {
                svg: props.svg.clone(),
                size: props.icon_size,
                label: props.text.clone(),
            },
            span {
                class: "icon-label",
                "{props.text}"
            }
        }
    }
}

/// Common icon helpers for Git status
pub mod git_status_icons {
    use super::*;
    use crate::desktop::styles::theme::Theme;

    /// Get icon for modified files
    pub fn modified(_theme: &Theme) -> String {
        "M".to_string()
    }

    /// Get icon for added files
    pub fn added(_theme: &Theme) -> String {
        "+".to_string()
    }

    /// Get icon for deleted files
    pub fn deleted(_theme: &Theme) -> String {
        "-".to_string()
    }

    /// Get icon for untracked files
    pub fn untracked(_theme: &Theme) -> String {
        "?".to_string()
    }

    /// Get icon for conflicted files
    pub fn conflicted(_theme: &Theme) -> String {
        "!".to_string()
    }

    /// Get icon for staged files
    pub fn staged(_theme: &Theme) -> String {
        "✓".to_string()
    }
}

/// Common icon helpers for UI elements
pub mod ui_icons {
    use super::*;
    use crate::desktop::styles::theme::Theme;

    /// Get loading/spinner icon
    pub fn loading(_theme: &Theme) -> String {
        "⟳".to_string()
    }

    /// Get success icon
    pub fn success(_theme: &Theme) -> String {
        "✓".to_string()
    }

    /// Get error icon
    pub fn error(_theme: &Theme) -> String {
        "✕".to_string()
    }

    /// Get warning icon
    pub fn warning(_theme: &Theme) -> String {
        "⚠".to_string()
    }

    /// Get info icon
    pub fn info(_theme: &Theme) -> String {
        "ℹ".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_size_dimensions() {
        assert_eq!(IconSize::Small.dimensions(), (12, 12));
        assert_eq!(IconSize::Medium.dimensions(), (16, 16));
        assert_eq!(IconSize::Large.dimensions(), (20, 20));
        assert_eq!(IconSize::XLarge.dimensions(), (24, 24));
    }

    #[test]
    fn test_icon_size_css() {
        assert_eq!(IconSize::Small.as_css(), "12px");
        assert_eq!(IconSize::Medium.as_css(), "16px");
        assert_eq!(IconSize::Large.as_css(), "20px");
        assert_eq!(IconSize::XLarge.as_css(), "24px");
    }
}