//! Main Desktop Application Component

use dioxus::prelude::*;
use crate::desktop::{
    components::*,
    layout::*,
    state::*,
    file_explorer::FileExplorer,
    chat::ChatInterface,
    consensus::ConsensusProgress,
};

/// Main Application Component
#[component]
pub fn App() -> Element {
    // Initialize application state
    let app_state = use_signal(|| AppState::new());
    
    // Provide state to all child components
    use_context_provider(|| app_state);
    
    rsx! {
        div {
            id: "app",
            class: "app-container",
            
            // Menu Bar Component
            MenuBar {}
            
            // Main Content Layout
            MainLayout {
                // Left Sidebar - File Explorer
                FileExplorer {}
                
                // Right Panel - Chat Interface
                ChatInterface {}
                
                // Consensus Progress (overlay when active)
                ConsensusProgress {}
            }
            
            // Status Bar
            StatusBar {}
        }
    }
}

/// Menu Bar Component
#[component]
fn MenuBar() -> Element {
    rsx! {
        div {
            class: "menu-bar",
            
            div {
                class: "menu-title",
                span { class: "logo", "🐝" }
                span { class: "title", "HiveTechs Consensus" }
            }
            
            div {
                class: "menu-actions",
                button {
                    class: "menu-btn",
                    onclick: |_| {
                        // Handle settings
                    },
                    "⚙️"
                }
                button {
                    class: "menu-btn", 
                    onclick: |_| {
                        // Handle minimize
                    },
                    "−"
                }
                button {
                    class: "menu-btn",
                    onclick: |_| {
                        // Handle close
                    },
                    "×"
                }
            }
        }
    }
}

/// Status Bar Component
#[component]
fn StatusBar() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let state = app_state.read();
    
    rsx! {
        div {
            class: "status-bar",
            
            div {
                class: "status-left",
                span {
                    class: "status-item",
                    match state.connection_status {
                        ConnectionStatus::Connected => "🟢 Connected",
                        ConnectionStatus::Connecting => "🟡 Connecting",
                        ConnectionStatus::Disconnected => "🔴 Disconnected",
                    }
                }
                span {
                    class: "status-item",
                    "📁 {state.current_project.as_ref().map(|p| p.name.as_str()).unwrap_or(\"No project\")}"
                }
            }
            
            div {
                class: "status-right",
                span {
                    class: "status-item",
                    "Cost: ${state.total_cost:.3}"
                }
                span {
                    class: "status-item", 
                    "Context: {state.context_usage}%"
                }
                span {
                    class: "status-item",
                    if state.auto_accept { "Auto-accept: ON" } else { "Auto-accept: OFF" }
                }
            }
        }
    }
}