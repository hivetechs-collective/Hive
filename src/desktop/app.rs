//! Main Desktop Application Component

use dioxus::prelude::*;
use dioxus::events::{KeyboardEvent, MouseEvent};
use crate::desktop::{
    file_explorer::FileExplorer,
    chat::ChatInterface,
    consensus::ConsensusProgress,
    state::{AppState, ConnectionStatus},
    events::{EventDispatcher, KeyboardEventUtils},
};

/// Main Application Component
#[component]
pub fn App() -> Element {
    // Initialize application state
    let app_state = use_signal(|| AppState::new());
    
    // Initialize event dispatcher
    let event_dispatcher = use_signal(|| EventDispatcher::new());
    
    // Provide state to all child components
    use_context_provider(move || app_state);
    use_context_provider(|| event_dispatcher);
    
    // Global keyboard shortcuts
    let on_global_keydown = move |evt: KeyboardEvent| {
        // Handle global keyboard shortcuts
        if evt.modifiers().ctrl() {
            match evt.key() {
                Key::Character(c) if c == "n" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: New file/conversation
                    tracing::debug!("Ctrl+N pressed - New file");
                },
                Key::Character(c) if c == "o" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Open file dialog
                    tracing::debug!("Ctrl+O pressed - Open file");
                },
                Key::Character(c) if c == "s" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Save current file
                    tracing::debug!("Ctrl+S pressed - Save file");
                },
                Key::Character(c) if c == "f" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Search/find
                    tracing::debug!("Ctrl+F pressed - Find");
                },
                Key::Character(c) if c == "p" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Command palette
                    tracing::debug!("Ctrl+P pressed - Command palette");
                },
                Key::Character(c) if c == "w" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Close current tab/file
                    tracing::debug!("Ctrl+W pressed - Close tab");
                },
                Key::Character(c) if c == "z" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Undo
                    tracing::debug!("Ctrl+Z pressed - Undo");
                },
                Key::Character(c) if c == "y" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Redo
                    tracing::debug!("Ctrl+Y pressed - Redo");
                },
                _ => {}
            }
        } else if evt.modifiers().alt() {
            match evt.key() {
                Key::Character(c) if c == "1" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Focus file explorer
                    tracing::debug!("Alt+1 pressed - Focus file explorer");
                },
                Key::Character(c) if c == "2" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Focus chat
                    tracing::debug!("Alt+2 pressed - Focus chat");
                },
                Key::Character(c) if c == "3" => {
                    // evt.prevent_default(); // Not available in Dioxus 0.5"
                    // TODO: Focus consensus panel
                    tracing::debug!("Alt+3 pressed - Focus consensus");
                },
                _ => {}
            }
        } else if evt.key() == Key::F1 {
            // evt.prevent_default(); // Not available in Dioxus 0.5"
            // TODO: Show help
            tracing::debug!("F1 pressed - Show help");
        }
    };
    
    rsx! {
        div {
            id: "app",
            class: "app-container",
            tabindex: 0,
            onkeydown: on_global_keydown,
            
            // Menu Bar Component
            MenuBar {}
            
            // Main Content Layout
            div {
                class: "main-layout",
                
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