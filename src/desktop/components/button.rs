//! Button Component

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
