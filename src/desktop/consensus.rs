//! Consensus Progress Component

use dioxus::prelude::*;
use crate::desktop::{
    state::{AppState, ConsensusProgress as ConsensusProgressState, ConsensusStage, StageInfo, StageStatus}, 
    components::*
};

/// Consensus Progress Component - Shows 4-stage pipeline
#[component]
pub fn ConsensusProgress() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let state = app_state.read();
    
    if !state.consensus.is_active {
        return rsx! { div {} };
    }
    
    rsx! {
        div {
            class: "consensus-progress-overlay",
            
            div {
                class: "consensus-progress",
                
                // Header
                div {
                    class: "consensus-header",
                    "🧠 4-Stage Consensus Pipeline"
                }
                
                // Progress Stages
                div {
                    class: "consensus-stages",
                    
                    for (i, stage_info) in state.consensus.stages.iter().enumerate() {
                        ConsensusStageItem { 
                            stage_info: stage_info.clone(),
                            is_active: state.consensus.current_stage.as_ref()
                                .map(|current| stage_matches_index(current, i))
                                .unwrap_or(false),
                            progress: get_stage_progress(&state.consensus.progress, i),
                        }
                    }
                }
                
                // Overall Progress
                div {
                    class: "overall-progress",
                    ProgressBar {
                        progress: calculate_overall_progress(&state.consensus.progress),
                        label: "Overall Progress".to_string(),
                    }
                }
            }
        }
    }
}

/// Individual consensus stage component
#[component]
fn ConsensusStageItem(
    stage_info: StageInfo,
    is_active: bool,
    progress: u8,
) -> Element {
    let status_class = match stage_info.status {
        StageStatus::Waiting => "waiting",
        StageStatus::Running => "running",
        StageStatus::Completed => "completed",
        StageStatus::Error => "error",
    };
    
    let stage_class = format!("stage {status_class} {}", if is_active { "active" } else { "" });
    
    rsx! {
        div {
            class: "{stage_class}",
            
            div {
                class: "stage-header",
                span {
                    class: "stage-name",
                    "{stage_info.name}"
                }
                
                span {
                    class: "stage-status-icon",
                    {match stage_info.status {
                        StageStatus::Waiting => "⏳",
                        StageStatus::Running => "⚡",
                        StageStatus::Completed => "✅",
                        StageStatus::Error => "❌",
                    }}
                }
            }
            
            div {
                class: "stage-model",
                "{stage_info.model}"
            }
            
            div {
                class: "stage-progress-container",
                ProgressBar {
                    progress: progress,
                    label: "".to_string(),
                }
            }
        }
    }
}

// Helper functions
fn stage_matches_index(stage: &ConsensusStage, index: usize) -> bool {
    match (stage, index) {
        (ConsensusStage::Generator, 0) => true,
        (ConsensusStage::Refiner, 1) => true,
        (ConsensusStage::Validator, 2) => true,
        (ConsensusStage::Curator, 3) => true,
        _ => false,
    }
}

fn get_stage_progress(progress: &ConsensusProgressState, stage_index: usize) -> u8 {
    match stage_index {
        0 => progress.generator,
        1 => progress.refiner,
        2 => progress.validator,
        3 => progress.curator,
        _ => 0,
    }
}

fn calculate_overall_progress(progress: &ConsensusProgressState) -> u8 {
    let total = progress.generator + progress.refiner + progress.validator + progress.curator;
    (total / 4).min(100)
}