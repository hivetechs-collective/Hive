// Simple test component to verify Dioxus syntax
use dioxus::prelude::*;

#[component]
pub fn SimpleTest() -> Element {
    rsx! {
        div {
            "Hello World"
        }
    }
}
