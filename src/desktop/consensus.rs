//! Consensus Progress Component - Always Visible Profile Panel

use crate::desktop::{
    components::*,
    state::{ActiveProfileData, ProfileState, ACTIVE_PROFILE_STATE},
};
use dioxus::prelude::*;

/// Consensus Progress Component - Shows profile and 4-stage pipeline
#[component]
pub fn ConsensusProgress() -> Element {
    tracing::info!("🔍 ConsensusProgress component rendering");
    
    // Use the global profile state - this will automatically re-render when state changes
    let profile_state = use_signal(|| {
        let state = ACTIVE_PROFILE_STATE();
        tracing::info!("🔍 Reading profile state: {:?}", state);
        state
    });
    
    rsx! {
        div {
            class: "consensus-progress-overlay always-visible",

            div {
                class: "consensus-progress",

                // Header with profile name
                div {
                    class: "consensus-header",
                    div {
                        class: "consensus-title",
                        "🧠 HiveTechs Consensus"
                    }
                    div {
                        class: "consensus-profile-name",
                        {render_profile_name(&profile_state)}
                    }
                }

                // Model stages
                div {
                    class: "consensus-stages",
                    {render_model_stages(&profile_state)}
                }

                // Stats (tokens and cost - placeholder for now)
                div {
                    class: "consensus-stats",
                    div {
                        class: "stat-item",
                        span { class: "stat-label", "Tokens:" }
                        span { class: "stat-value", "0" }
                    }
                    div {
                        class: "stat-item",
                        span { class: "stat-label", "Cost:" }
                        span { class: "stat-value", "$0.00" }
                    }
                }
            }
        }
    }
}

/// Render profile name based on current state
fn render_profile_name(profile_state: &Signal<ProfileState>) -> String {
    let state = profile_state.read();
    match &*state {
        ProfileState::Loading => "Loading...".to_string(),
        ProfileState::Loaded(profile) => format!("Profile: {}", profile.name),
        ProfileState::Error(err) => format!("Error: {}", err),
    }
}

/// Render model stages based on current state
fn render_model_stages(profile_state: &Signal<ProfileState>) -> Element {
    let state = profile_state.read();
    
    match &*state {
        ProfileState::Loading => rsx! {
            div { class: "loading-stages", "Loading models..." }
        },
        ProfileState::Loaded(profile) => rsx! {
            div { class: "stage", 
                div { class: "stage-name", "Generator" }
                div { class: "stage-model", "{profile.generator_model}" }
                div { class: "stage-progress", 
                    div { class: "progress-bar", style: "width: 0%" }
                }
            }
            div { class: "stage", 
                div { class: "stage-name", "Refiner" }
                div { class: "stage-model", "{profile.refiner_model}" }
                div { class: "stage-progress", 
                    div { class: "progress-bar", style: "width: 0%" }
                }
            }
            div { class: "stage", 
                div { class: "stage-name", "Validator" }
                div { class: "stage-model", "{profile.validator_model}" }
                div { class: "stage-progress", 
                    div { class: "progress-bar", style: "width: 0%" }
                }
            }
            div { class: "stage", 
                div { class: "stage-name", "Curator" }
                div { class: "stage-model", "{profile.curator_model}" }
                div { class: "stage-progress", 
                    div { class: "progress-bar", style: "width: 0%" }
                }
            }
        },
        ProfileState::Error(err) => rsx! {
            div { class: "error-stages", "Failed to load: {err}" }
        },
    }
}

