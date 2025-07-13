//! Progress Bar Component

use dioxus::prelude::*;

/// Progress bar component
#[component]
pub fn ProgressBar(
    progress: u8, 
    #[props(default = "".to_string())] 
    label: String
) -> Element {
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