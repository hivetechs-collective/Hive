//! Example usage of VS Code styled components
//! 
//! This module demonstrates how to use the styling system

use dioxus::prelude::*;
use super::components::*;
use super::theme::*;

/// Example app showing styled components
#[component]
pub fn StyledExample() -> Element {
    let mut count = use_signal(|| 0);
    let theme = use_theme();
    
    rsx! {
        div { class: "p-4",
            h1 { "VS Code Styled Components Example" }
            
            // Button examples
            div { class: "m-2",
                h3 { "Buttons" }
                div { class: "flex items-center p-2",
                    VsCodeButton {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| count += 1,
                        "Primary Button"
                    }
                    span { class: "m-2" }
                    VsCodeButton {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| count -= 1,
                        "Secondary Button"
                    }
                    span { class: "m-2" }
                    VsCodeButton {
                        variant: ButtonVariant::Icon,
                        icon: "cog".to_string(),
                        onclick: move |_| println!("Settings clicked"),
                    }
                }
            }
            
            // Panel examples
            div { class: "m-2",
                h3 { "Panels" }
                VsCodePanel {
                    style: PanelStyle::Sidebar,
                    title: "Explorer".to_string(),
                    div {
                        FileTreeItem {
                            label: "src/".to_string(),
                            is_folder: true,
                            is_selected: false,
                            onclick: move |_| println!("Folder clicked"),
                        }
                        FileTreeItem {
                            label: "main.rs".to_string(),
                            is_folder: false,
                            is_selected: true,
                            indent_level: 1,
                            onclick: move |_| println!("File clicked"),
                        }
                    }
                }
            }
            
            // Tab examples
            div { class: "m-2",
                h3 { "Tabs" }
                div { class: "tabs",
                    VsCodeTab {
                        label: "Problems".to_string(),
                        is_active: true,
                        onclick: move |_| println!("Problems tab"),
                        icon: "exclamation-triangle".to_string(),
                    }
                    VsCodeTab {
                        label: "Output".to_string(),
                        is_active: false,
                        onclick: move |_| println!("Output tab"),
                        icon: "terminal".to_string(),
                    }
                    VsCodeTab {
                        label: "Terminal".to_string(),
                        is_active: false,
                        onclick: move |_| println!("Terminal tab"),
                        icon: "square-terminal".to_string(),
                    }
                }
            }
            
            // Theme switcher
            div { class: "m-2",
                h3 { "Theme" }
                div { class: "flex items-center",
                    span { "Current theme: {theme.read().display_name()}" }
                    span { class: "m-2" }
                    ThemeSwitcher {}
                }
            }
            
            // Counter display
            div { class: "m-2",
                h3 { "Interactive Counter" }
                div { class: "panel panel-editor p-3",
                    p { "Count: {count}" }
                    div { class: "code-block",
                        div { class: "code-block-header",
                            span { "main.rs" }
                            button { class: "btn btn-icon",
                                i { class: "fa fa-copy" }
                            }
                        }
                        div { class: "code-block-content",
                            code {
                                span { class: "keyword", "let" }
                                " count = "
                                span { class: "number", "{count}" }
                                ";"
                            }
                        }
                    }
                }
            }
            
            // Notifications example
            div { class: "m-2",
                h3 { "Notifications" }
                div { class: "flex",
                    button {
                        class: "btn btn-success m-1",
                        onclick: move |_| show_notification("Success", "Operation completed!", "success"),
                        "Success"
                    }
                    button {
                        class: "btn btn-warning m-1",
                        onclick: move |_| show_notification("Warning", "Check your input", "warning"),
                        "Warning"
                    }
                    button {
                        class: "btn btn-danger m-1",
                        onclick: move |_| show_notification("Error", "Something went wrong", "error"),
                        "Error"
                    }
                }
            }
            
            // Search input example
            div { class: "m-2",
                h3 { "Search Input" }
                div { class: "search-input",
                    i { class: "fa fa-search search-input-icon" }
                    input {
                        r#type: "text",
                        placeholder: "Search files...",
                        onchange: move |evt| println!("Search: {}", evt.value()),
                    }
                }
            }
            
            // Progress bar example
            div { class: "m-2",
                h3 { "Progress Bar" }
                div { class: "progress-bar",
                    div { 
                        class: "progress-bar-fill",
                        style: "width: {count}%",
                    }
                }
                div { class: "progress-bar indeterminate m-2",
                    div { class: "progress-bar-fill" }
                }
            }
            
            // Status bar example
            div { class: "status-bar",
                StatusBarItem {
                    i { class: "fa fa-code-branch m-1" }
                    "main"
                }
                StatusBarItem {
                    i { class: "fa fa-sync m-1" }
                    "0↑ 0↓"
                }
                StatusBarItem {
                    onclick: move |_| println!("Line/column clicked"),
                    "Ln 1, Col 1"
                }
                StatusBarItem {
                    "UTF-8"
                }
                StatusBarItem {
                    "Rust"
                }
            }
        }
    }
}

fn show_notification(title: &str, message: &str, notification_type: &str) {
    // In a real app, this would show an actual notification
    println!("Notification [{}]: {} - {}", notification_type, title, message);
}

/// Example of using the styling system in a real component
#[component]
pub fn ConsensusStageIndicator(
    stage_name: String,
    status: StageStatus,
    #[props(default)] progress: Option<f32>,
) -> Element {
    let (class, icon) = match status {
        StageStatus::Pending => ("consensus-stage", "circle"),
        StageStatus::Active => ("consensus-stage active", "spinner"),
        StageStatus::Complete => ("consensus-stage complete", "check-circle"),
    };
    
    rsx! {
        div { class: "{class}",
            i { class: "fa fa-{icon} consensus-stage-icon" }
            span { class: "consensus-stage-name", "{stage_name}" }
            if let Some(pct) = progress {
                span { class: "consensus-stage-status", "{pct:.0}%" }
            } else {
                span { class: "consensus-stage-status", "{status:?}" }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageStatus {
    Pending,
    Active,
    Complete,
}