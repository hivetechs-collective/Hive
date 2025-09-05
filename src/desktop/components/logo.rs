//! HiveTechs Logo Component

use dioxus::prelude::*;

/// HiveTechs Logo Component
#[component]
pub fn HiveLogo(
    #[props(default = 24)] size: u32,
    #[props(default = "#FFC107")] color: &'static str,
) -> Element {
    let hex_path = format!(
        "M{} {}L{} {}L{} {}L{} {}L{} {}L{} {}Z",
        size / 2,
        size / 8, // top
        size * 7 / 8,
        size / 4, // top-right
        size * 7 / 8,
        size * 3 / 4, // bottom-right
        size / 2,
        size * 7 / 8, // bottom
        size / 8,
        size * 3 / 4, // bottom-left
        size / 8,
        size / 4 // top-left
    );

    rsx! {
        div {
            style: "display: inline-flex; align-items: center; justify-content: center; width: {size}px; height: {size}px;",

            svg {
                width: "{size}",
                height: "{size}",
                view_box: "0 0 {size} {size}",
                fill: "none",
                xmlns: "http://www.w3.org/2000/svg",

                // Hexagon outline
                path {
                    d: "{hex_path}",
                    stroke: "{color}",
                    stroke_width: "2",
                    fill: "none"
                }

                // Inner bee wings (simplified)
                circle {
                    cx: "{size * 3 / 8}",
                    cy: "{size / 2}",
                    r: "{size / 8}",
                    fill: "{color}",
                    opacity: "0.7"
                }
                circle {
                    cx: "{size * 5 / 8}",
                    cy: "{size / 2}",
                    r: "{size / 8}",
                    fill: "{color}",
                    opacity: "0.7"
                }

                // Center body
                rect {
                    x: "{size * 7 / 16}",
                    y: "{size * 3 / 8}",
                    width: "{size / 8}",
                    height: "{size / 4}",
                    fill: "{color}",
                    rx: "2"
                }
            }
        }
    }
}

/// Small logo for menu bar
#[component]
pub fn HiveLogoSmall() -> Element {
    rsx! {
        HiveLogo { size: 18, color: "#FFC107" }
    }
}

/// Large logo for dialogs
#[component]
pub fn HiveLogoLarge() -> Element {
    rsx! {
        HiveLogo { size: 64, color: "#FFC107" }
    }
}
