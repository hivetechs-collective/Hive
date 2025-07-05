//! Test consensus progress visualization in desktop app

use dioxus::prelude::*;
use hive::desktop::{
    state::{AppState, ConsensusState, ConsensusStage, StageStatus},
    consensus::ConsensusProgress,
    styles,
};
use std::time::Duration;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Launch the desktop app
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Hive Consensus Progress Test")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0))
                )
        )
        .launch(app);
}

fn app() -> Element {
    let mut app_state = use_signal(|| AppState::new());
    
    // Provide state to all child components
    use_context_provider(move || app_state);
    
    // Simulate consensus progress
    use_coroutine(move |_| async move {
        // Wait a bit before starting
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Start consensus
        app_state.write().consensus.start_consensus();
        
        // Simulate Generator progress
        for i in 0..=100 {
            app_state.write().consensus.update_progress(ConsensusStage::Generator, i);
            app_state.write().consensus.add_tokens(10, 0.001);
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
        
        // Simulate Refiner progress
        for i in 0..=100 {
            app_state.write().consensus.update_progress(ConsensusStage::Refiner, i);
            app_state.write().consensus.add_tokens(8, 0.0008);
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        
        // Simulate Validator progress
        for i in 0..=100 {
            app_state.write().consensus.update_progress(ConsensusStage::Validator, i);
            app_state.write().consensus.add_tokens(12, 0.0012);
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        
        // Simulate Curator progress
        for i in 0..=100 {
            app_state.write().consensus.update_progress(ConsensusStage::Curator, i);
            app_state.write().consensus.add_tokens(5, 0.0005);
            tokio::time::sleep(Duration::from_millis(15)).await;
        }
        
        // Complete consensus
        tokio::time::sleep(Duration::from_secs(1)).await;
        app_state.write().consensus.complete_consensus();
    });
    
    // Add global styles
    rsx! {
        style { {styles::get_global_styles()} }
        
        div {
            class: "app-container",
            style: "background: #1e1e1e; color: #ccc; min-height: 100vh; display: flex; align-items: center; justify-content: center;",
            
            div {
                style: "text-align: center; padding: 40px;",
                
                h1 {
                    style: "font-size: 24px; margin-bottom: 20px;",
                    "Hive Consensus Progress Test"
                }
                
                p {
                    style: "margin-bottom: 20px; color: #888;",
                    "The consensus progress overlay will appear automatically..."
                }
                
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        app_state.write().consensus.start_consensus();
                    },
                    "Start Consensus Again"
                }
            }
            
            // Render the consensus progress overlay
            ConsensusProgress {}
        }
    }
}