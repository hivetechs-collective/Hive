//! Terminal Tabs Component - VS Code-inspired multi-terminal management
//! 
//! This component provides a tabbed interface for multiple terminal instances,
//! allowing users to create, switch between, and manage multiple terminals.

use dioxus::prelude::*;
use std::collections::HashMap;

/// Terminal tab data
#[derive(Clone, Debug)]
pub struct TerminalTab {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub is_active: bool,
    pub working_directory: String,
}

/// Terminal tabs manager component
#[component]
pub fn TerminalTabs() -> Element {
    // Terminal instances state
    let mut terminals = use_signal(|| HashMap::<String, TerminalTab>::new());
    let mut active_terminal_id = use_signal(|| Option::<String>::None);
    let mut terminal_counter = use_signal(|| 1u32);
    
    // Create initial terminal on mount
    use_effect(move || {
        if terminals.read().is_empty() {
            create_new_terminal(&mut terminals, &mut active_terminal_id, &mut terminal_counter);
        }
    });

    // Tab bar style
    let tab_bar_style = "
        display: flex;
        align-items: center;
        height: 35px;
        background: #252526;
        border-bottom: 1px solid #1e1e1e;
        padding: 0 10px;
        gap: 2px;
    ";

    let tab_style = |is_active: bool| format!(
        "
        display: flex;
        align-items: center;
        padding: 0 12px;
        height: 35px;
        background: {};
        color: {};
        cursor: pointer;
        user-select: none;
        font-size: 13px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
        border-right: 1px solid #1e1e1e;
        gap: 6px;
        ",
        if is_active { "#1e1e1e" } else { "transparent" },
        if is_active { "#cccccc" } else { "#969696" }
    );

    let tab_close_style = "
        margin-left: 4px;
        opacity: 0.5;
        cursor: pointer;
        font-size: 16px;
        line-height: 1;
        padding: 2px;
        border-radius: 3px;
        transition: opacity 0.2s, background 0.2s;
    ";

    let new_terminal_btn_style = "
        display: flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        background: transparent;
        border: none;
        color: #cccccc;
        cursor: pointer;
        font-size: 18px;
        border-radius: 3px;
        transition: background 0.2s;
        margin-left: 4px;
    ";

    let terminal_container_style = "
        flex: 1;
        display: flex;
        flex-direction: column;
        background: #1e1e1e;
        overflow: hidden;
    ";

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%;",

            // Tab bar
            div {
                style: "{tab_bar_style}",

                // Terminal tabs
                for (id, tab) in terminals.read().iter() {
                    div {
                        key: "{id}",
                        style: "{tab_style(tab.is_active)}",
                        onclick: {
                            let id = id.clone();
                            let mut terminals = terminals.clone();
                            let mut active_terminal_id = active_terminal_id.clone();
                            move |_| {
                                set_active_terminal(&mut terminals, &mut active_terminal_id, &id);
                            }
                        },

                        // Terminal icon
                        span {
                            style: "font-size: 14px;",
                            "{tab.icon}"
                        }

                        // Terminal title
                        span {
                            style: "white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 200px;",
                            "{tab.title}"
                        }

                        // Close button
                        if terminals.read().len() > 1 {
                            span {
                                style: "{tab_close_style}",
                                onmouseenter: |_| {
                                    // Mouse enter handler
                                },
                                onclick: {
                                    let id = id.clone();
                                    let mut terminals = terminals.clone();
                                    let mut active_terminal_id = active_terminal_id.clone();
                                    move |_| {
                                        close_terminal(&mut terminals, &mut active_terminal_id, &id);
                                    }
                                },
                                onmouseover: |_| {
                                    // Would set hover style in real implementation
                                },
                                onmouseout: |_| {
                                    // Would reset style in real implementation
                                },
                                "×"
                            }
                        }
                    }
                }

                // New terminal button
                button {
                    style: "{new_terminal_btn_style}",
                    onclick: move |_| {
                        create_new_terminal(&mut terminals, &mut active_terminal_id, &mut terminal_counter);
                    },
                    onmouseover: |_| {
                        // Would set hover style in real implementation
                    },
                    onmouseout: |_| {
                        // Would reset style in real implementation  
                    },
                    title: "New Terminal",
                    "+"
                }

                // Terminal dropdown menu (VS Code style)
                div {
                    style: "margin-left: auto; display: flex; gap: 4px;",
                    
                    // Split terminal button
                    button {
                        style: "{new_terminal_btn_style}",
                        title: "Split Terminal",
                        onclick: move |_| {
                            // Split terminal functionality
                            tracing::info!("Split terminal clicked");
                        },
                        "⊞"
                    }

                    // Kill terminal button  
                    button {
                        style: "{new_terminal_btn_style}",
                        title: "Kill Terminal",
                        onclick: {
                            let mut terminals = terminals.clone();
                            let active_terminal_id_clone = active_terminal_id.clone();
                            let mut active_terminal_id_mut = active_terminal_id.clone();
                            move |_| {
                                if let Some(active_id) = active_terminal_id_clone.read().as_ref() {
                                    let active_id = active_id.clone();
                                    close_terminal(&mut terminals, &mut active_terminal_id_mut, &active_id);
                                }
                            }
                        },
                        "🗑"
                    }
                }
            }

            // Terminal content area
            div {
                style: "{terminal_container_style}",
                
                if let Some(active_id) = active_terminal_id.read().as_ref() {
                    // Import and use the existing Terminal component
                    // Pass the active terminal's working directory
                    TerminalInstance {
                        terminal_id: active_id.clone(),
                        working_directory: terminals.read().get(active_id)
                            .map(|t| t.working_directory.clone())
                            .unwrap_or_else(|| std::env::current_dir()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string())
                    }
                } else {
                    div {
                        style: "display: flex; align-items: center; justify-content: center; height: 100%; color: #666;",
                        "No terminal active"
                    }
                }
            }
        }
    }
}

/// Terminal instance wrapper that maintains state per terminal
#[component]
fn TerminalInstance(terminal_id: String, working_directory: String) -> Element {
    // Import the existing Terminal component
    use crate::desktop::terminal::Terminal;
    
    // Each terminal instance maintains its own state through the terminal_id
    rsx! {
        div {
            key: "{terminal_id}",
            style: "height: 100%; width: 100%;",
            Terminal {}
        }
    }
}

/// Create a new terminal tab
fn create_new_terminal(
    terminals: &mut Signal<HashMap<String, TerminalTab>>,
    active_terminal_id: &mut Signal<Option<String>>,
    terminal_counter: &mut Signal<u32>,
) {
    let count = *terminal_counter.read();
    let id = format!("terminal-{}", count);
    
    let new_terminal = TerminalTab {
        id: id.clone(),
        title: format!("Terminal {}", count),
        icon: "$".to_string(),
        is_active: true,
        working_directory: std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    };

    // Deactivate all other terminals
    for (_, terminal) in terminals.write().iter_mut() {
        terminal.is_active = false;
    }

    terminals.write().insert(id.clone(), new_terminal);
    active_terminal_id.set(Some(id));
    *terminal_counter.write() += 1;
}

/// Set active terminal
fn set_active_terminal(
    terminals: &mut Signal<HashMap<String, TerminalTab>>,
    active_terminal_id: &mut Signal<Option<String>>,
    terminal_id: &str,
) {
    // Deactivate all terminals
    for (_, terminal) in terminals.write().iter_mut() {
        terminal.is_active = false;
    }
    
    // Activate selected terminal
    if let Some(terminal) = terminals.write().get_mut(terminal_id) {
        terminal.is_active = true;
        active_terminal_id.set(Some(terminal_id.to_string()));
    }
}

/// Close a terminal tab
fn close_terminal(
    terminals: &mut Signal<HashMap<String, TerminalTab>>,
    active_terminal_id: &mut Signal<Option<String>>,
    terminal_id: &str,
) {
    let current_active = active_terminal_id.read().clone();
    terminals.write().remove(terminal_id);
    
    // If we closed the active terminal, activate another one
    if current_active.as_deref() == Some(terminal_id) {
        let next_id = terminals.read().iter().next().map(|(id, _)| id.clone());
        if let Some(next_id) = next_id {
            set_active_terminal(terminals, active_terminal_id, &next_id);
        } else {
            active_terminal_id.set(None);
        }
    }
}