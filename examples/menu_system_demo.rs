//! Example demonstrating the menu system components

use dioxus::prelude::*;
use crate::desktop::components::{
    menu::{Menu, MenuProps, GenericMenuItem, MenuItemBuilder},
    tooltip::{Tooltip, TooltipProps, TooltipPosition, TooltipTrigger},
    progress_indicator::{
        ProgressIndicator, ProgressIndicatorProps, ProgressType, 
        ProgressSize, ProgressColor, ProgressStep, Spinner, LoadingDots
    },
};

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        div {
            style: "padding: 20px; font-family: Arial, sans-serif;",
            
            h1 { "Menu System Components Demo" }
            
            // Menu Component Demo
            MenuDemo {}
            
            hr { style: "margin: 40px 0;" }
            
            // Tooltip Component Demo
            TooltipDemo {}
            
            hr { style: "margin: 40px 0;" }
            
            // Progress Indicator Demo
            ProgressDemo {}
        }
    }
}

#[component]
fn MenuDemo() -> Element {
    let mut menu_visible = use_signal(|| false);
    let mut last_selected = use_signal(|| String::new());
    
    let menu_items = vec![
        MenuItemBuilder::new("New File").shortcut("Ctrl+N").icon("📄").build(),
        MenuItemBuilder::new("Open File").shortcut("Ctrl+O").icon("📁").build(),
        MenuItemBuilder::new("Save").shortcut("Ctrl+S").icon("💾").build(),
        MenuItemBuilder::separator().build(),
        MenuItemBuilder::new("Cut").shortcut("Ctrl+X").icon("✂️").build(),
        MenuItemBuilder::new("Copy").shortcut("Ctrl+C").icon("📋").build(),
        MenuItemBuilder::new("Paste").shortcut("Ctrl+V").icon("📌").enabled(false).build(),
        MenuItemBuilder::separator().build(),
        MenuItemBuilder::new("Settings").icon("⚙️").submenu().build(),
        MenuItemBuilder::new("Exit").shortcut("Alt+F4").icon("🚪").build(),
    ];
    
    rsx! {
        div {
            h2 { "Menu Component" }
            
            button {
                onclick: move |_| menu_visible.set(!menu_visible()),
                style: "padding: 8px 16px; cursor: pointer;",
                "Toggle Menu"
            }
            
            if !last_selected().is_empty() {
                p { "Last selected: {last_selected}" }
            }
            
            div {
                style: "position: relative; margin-top: 10px;",
                
                Menu {
                    items: menu_items,
                    visible: menu_visible(),
                    on_select: move |item: GenericMenuItem| {
                        last_selected.set(item.label.clone());
                        menu_visible.set(false);
                    },
                    on_close: move |_| menu_visible.set(false),
                }
            }
        }
    }
}

#[component]
fn TooltipDemo() -> Element {
    rsx! {
        div {
            h2 { "Tooltip Component" }
            
            div {
                style: "display: flex; gap: 20px; flex-wrap: wrap;",
                
                Tooltip {
                    content: "This tooltip appears on top",
                    position: TooltipPosition::Top,
                    button {
                        style: "padding: 8px 16px;",
                        "Hover me (Top)"
                    }
                }
                
                Tooltip {
                    content: "This tooltip appears on the right",
                    position: TooltipPosition::Right,
                    button {
                        style: "padding: 8px 16px;",
                        "Hover me (Right)"
                    }
                }
                
                Tooltip {
                    content: "This tooltip appears on click",
                    trigger: TooltipTrigger::Click,
                    button {
                        style: "padding: 8px 16px;",
                        "Click me"
                    }
                }
                
                Tooltip {
                    content: "This tooltip has no delay",
                    delay: 0,
                    button {
                        style: "padding: 8px 16px;",
                        "Instant tooltip"
                    }
                }
            }
        }
    }
}

#[component]
fn ProgressDemo() -> Element {
    let mut progress = use_signal(|| 0.0);
    let mut loading = use_signal(|| false);
    
    // Auto-increment progress
    use_effect(move || {
        if progress() < 100.0 && !loading() {
            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                progress.set((progress() + 2.0).min(100.0));
            });
        }
    });
    
    let steps = vec![
        ProgressStep {
            label: "Setup".to_string(),
            description: Some("Initialize project".to_string()),
            completed: true,
            current: false,
            error: false,
        },
        ProgressStep {
            label: "Build".to_string(),
            description: Some("Compile source code".to_string()),
            completed: true,
            current: false,
            error: false,
        },
        ProgressStep {
            label: "Test".to_string(),
            description: Some("Run test suite".to_string()),
            completed: false,
            current: true,
            error: false,
        },
        ProgressStep {
            label: "Deploy".to_string(),
            description: Some("Deploy to production".to_string()),
            completed: false,
            current: false,
            error: false,
        },
    ];
    
    rsx! {
        div {
            h2 { "Progress Indicators" }
            
            div {
                style: "display: flex; flex-direction: column; gap: 30px;",
                
                // Linear Progress
                div {
                    h3 { "Linear Progress" }
                    ProgressIndicator {
                        value: Some(progress()),
                        progress_type: ProgressType::Linear,
                        label: Some("Loading...".to_string()),
                        color: ProgressColor::Primary,
                    }
                    
                    button {
                        onclick: move |_| progress.set(0.0),
                        style: "margin-top: 10px; padding: 4px 8px;",
                        "Reset"
                    }
                }
                
                // Circular Progress
                div {
                    h3 { "Circular Progress" }
                    div {
                        style: "display: flex; gap: 20px; align-items: center;",
                        
                        ProgressIndicator {
                            value: Some(progress()),
                            progress_type: ProgressType::Circular,
                            size: ProgressSize::Small,
                            color: ProgressColor::Success,
                        }
                        
                        ProgressIndicator {
                            value: Some(progress()),
                            progress_type: ProgressType::Circular,
                            size: ProgressSize::Medium,
                            color: ProgressColor::Warning,
                        }
                        
                        ProgressIndicator {
                            value: Some(progress()),
                            progress_type: ProgressType::Circular,
                            size: ProgressSize::Large,
                            color: ProgressColor::Error,
                        }
                    }
                }
                
                // Indeterminate Indicators
                div {
                    h3 { "Indeterminate Progress" }
                    div {
                        style: "display: flex; gap: 20px; align-items: center;",
                        
                        Spinner {
                            size: ProgressSize::Medium,
                            color: ProgressColor::Primary,
                            label: Some("Loading...".to_string()),
                        }
                        
                        LoadingDots {
                            size: ProgressSize::Medium,
                            color: ProgressColor::Info,
                        }
                    }
                }
                
                // Steps Progress
                div {
                    h3 { "Steps Progress" }
                    ProgressIndicator {
                        progress_type: ProgressType::Steps,
                        steps: steps,
                        color: ProgressColor::Primary,
                    }
                }
            }
        }
    }
}