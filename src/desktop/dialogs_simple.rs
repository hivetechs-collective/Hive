//! Simplified dialog system that actually works

use dioxus::prelude::*;

/// Simplified Onboarding Dialog - minimal version that closes properly
#[component]
pub fn OnboardingDialog(
    show_onboarding: Signal<bool>,
    openrouter_key: Signal<String>,
    hive_key: Signal<String>,
    current_step: Signal<i32>
) -> Element {
    // If dialog should not be shown, return empty
    if !show_onboarding() {
        return None;
    }

    // Simple close handler
    let close_dialog = move |_| {
        tracing::info!("Closing onboarding dialog");
        show_onboarding.set(false);
    };

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: close_dialog,

            div {
                style: "background: #2d2d30; padding: 40px; border-radius: 8px; width: 500px;",
                onclick: move |evt| evt.stop_propagation(),

                h2 { "Welcome to Hive" }
                p { "This is a simplified dialog that will actually close." }

                button {
                    style: "background: #007acc; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer;",
                    onclick: close_dialog,
                    "Close Dialog"
                }
            }
        }
    }
}