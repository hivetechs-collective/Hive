//! Component-specific styling helpers for Dioxus
//!
//! Provides utility functions and constants for consistent styling

use dioxus::prelude::*;

/// Apply VS Code themed classes to components
pub trait VsCodeTheme {
    fn vscode_class(&self) -> &'static str;
}

/// Button styles matching VS Code theme
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Icon,
    Ghost,
}

impl VsCodeTheme for ButtonVariant {
    fn vscode_class(&self) -> &'static str {
        match self {
            Self::Primary => "btn btn-primary",
            Self::Secondary => "btn btn-secondary",
            Self::Icon => "btn btn-icon",
            Self::Ghost => "btn btn-ghost",
        }
    }
}

/// Panel styles for different contexts
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelStyle {
    Sidebar,
    Editor,
    Terminal,
    Output,
}

impl VsCodeTheme for PanelStyle {
    fn vscode_class(&self) -> &'static str {
        match self {
            Self::Sidebar => "panel panel-sidebar",
            Self::Editor => "panel panel-editor",
            Self::Terminal => "panel panel-terminal",
            Self::Output => "panel panel-output",
        }
    }
}

/// Icon helpers for consistent sizing and styling
pub fn icon_class(size: IconSize) -> &'static str {
    match size {
        IconSize::Small => "icon icon-sm",
        IconSize::Medium => "icon icon-md",
        IconSize::Large => "icon icon-lg",
    }
}

pub enum IconSize {
    Small,  // 12px
    Medium, // 16px
    Large,  // 20px
}

/// Create a styled VS Code button component
#[component]
pub fn VsCodeButton(
    onclick: EventHandler<MouseEvent>,
    children: Element,
    #[props(default)] variant: Option<ButtonVariant>,
    #[props(default = false)] disabled: bool,
    #[props(default)] icon: Option<String>,
) -> Element {
    let variant = variant.unwrap_or(ButtonVariant::Primary);
    let class = variant.vscode_class();

    rsx! {
        button {
            class: "{class}",
            disabled: disabled,
            onclick: move |evt| onclick(evt),
            if let Some(icon_name) = icon {
                i { class: "fa fa-{icon_name} icon" }
            }
            {children}
        }
    }
}

/// Create a styled panel component
#[component]
pub fn VsCodePanel(
    children: Element,
    #[props(default)] style: Option<PanelStyle>,
    #[props(default)] title: Option<String>,
) -> Element {
    let style = style.unwrap_or(PanelStyle::Sidebar);
    let class = style.vscode_class();

    rsx! {
        div {
            class: "{class}",
            if let Some(title_text) = title {
                div {
                    class: "panel-header",
                    h3 { "{title_text}" }
                }
            }
            div {
                class: "panel-body",
                {children}
            }
        }
    }
}

/// Create a file tree item with proper styling
#[component]
pub fn FileTreeItem(
    label: String,
    #[props(default = false)] is_folder: bool,
    #[props(default = false)] is_selected: bool,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default = 0)] indent_level: usize,
) -> Element {
    let indent = format!("{}px", indent_level * 20);
    let icon = if is_folder { "folder" } else { "file" };
    let class = if is_selected {
        "tree-item selected"
    } else {
        "tree-item"
    };

    rsx! {
        div {
            class: "{class}",
            style: "padding-left: {indent}",
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler(evt);
                }
            },
            i { class: "fa fa-{icon} tree-item-icon" }
            span { class: "tree-item-label", "{label}" }
        }
    }
}

/// Create a VS Code styled tab
#[component]
pub fn VsCodeTab(
    label: String,
    #[props(default = false)] is_active: bool,
    onclick: EventHandler<MouseEvent>,
    #[props(default)] icon: Option<String>,
) -> Element {
    let class = if is_active { "tab active" } else { "tab" };

    rsx! {
        div {
            class: "{class}",
            onclick: move |evt| onclick(evt),
            if let Some(icon_name) = icon {
                i { class: "fa fa-{icon_name} icon" }
            }
            span { "{label}" }
        }
    }
}

/// Create a status bar item
#[component]
pub fn StatusBarItem(
    children: Element,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        div {
            class: "status-bar-item",
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler(evt);
                }
            },
            {children}
        }
    }
}

/// Utility function to combine classes
pub fn cx(classes: &[&str]) -> String {
    classes.join(" ")
}

/// Theme-aware syntax highlighting for code blocks
pub fn syntax_highlight(code: &str, language: &str) -> String {
    // This would integrate with a syntax highlighting library
    // For now, return the code wrapped in appropriate classes
    format!(
        r#"<pre class="code-block"><code class="language-{}">{}</code></pre>"#,
        language,
        html_escape(code)
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
