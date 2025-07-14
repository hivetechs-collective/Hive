//! Loading Spinner Component

use dioxus::prelude::*;

/// Loading spinner component
#[component]
pub fn LoadingSpinner() -> Element {
    rsx! {
        div {
            class: "loading-spinner",
            div { class: "spinner-circle" }
        }
    }
}
