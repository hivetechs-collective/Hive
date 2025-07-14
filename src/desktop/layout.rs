//! Main Layout Components

use dioxus::prelude::*;

/// Main layout container with VS Code-like structure
#[component]
pub fn MainLayout(children: Element) -> Element {
    rsx! {
        div {
            class: "main-layout",
            {children}
        }
    }
}

/// Resizable panel container
#[component]
pub fn ResizablePanel(
    #[props(default = 300)] width: i32,
    #[props(default = "left".to_string())] position: String,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "resizable-panel panel-{position}",
            style: "width: {width}px",
            {children}
        }
    }
}
