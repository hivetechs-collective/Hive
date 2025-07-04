//! Reusable UI Components

use dioxus::prelude::*;

/// Button component with consistent styling
#[component]
pub fn Button(
    onclick: EventHandler<MouseEvent>,
    children: Element,
    #[props(default)] style: Option<ButtonStyle>,
    #[props(default = false)] disabled: bool,
) -> Element {
    let style = style.unwrap_or(ButtonStyle::Primary);
    let class = format!("btn btn-{}", style.class_name());
    
    rsx! {
        button {
            class: "{class}",
            disabled: disabled,
            onclick: move |evt| onclick(evt),
            {children}
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}

impl ButtonStyle {
    fn class_name(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary", 
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
        }
    }
}

/// Loading spinner component
#[component]
pub fn LoadingSpinner() -> Element {
    rsx! {
        div {
            class: "loading-spinner",
            "⏳ Loading..."
        }
    }
}

/// Progress bar component
#[component]
pub fn ProgressBar(progress: u8, #[props(default = "".to_string())] label: String) -> Element {
    let percentage = progress.min(100);
    
    rsx! {
        div {
            class: "progress-container",
            
            if !label.is_empty() {
                div { class: "progress-label", "{label}" }
            }
            
            div {
                class: "progress-bar",
                div {
                    class: "progress-fill",
                    style: "width: {percentage}%",
                }
            }
            
            div {
                class: "progress-text",
                "{percentage}%"
            }
        }
    }
}