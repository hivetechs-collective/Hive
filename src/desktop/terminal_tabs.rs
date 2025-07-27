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
    
    // Tab overflow management - matching editor tabs behavior
    let mut tab_scroll_offset = use_signal(|| 0usize);
    let max_visible_tabs = 6; // Maximum number of tabs to display before scrolling
    
    // Create initial terminal on mount
    use_effect(move || {
        tracing::info!("🔍 TerminalTabs component mounted - checking terminals");
        if terminals.read().is_empty() {
            tracing::info!("📝 Creating initial terminal");
            create_new_terminal(&mut terminals, &mut active_terminal_id, &mut terminal_counter, &mut tab_scroll_offset, max_visible_tabs);
        } else {
            tracing::info!("✅ Terminals already exist: {}", terminals.read().len());
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
    
    // Scroll button style - matching editor tabs
    let scroll_btn_style = "
        background: rgba(255, 193, 7, 0.1);
        border: 1px solid rgba(255, 193, 7, 0.3);
        color: #FFC107;
        width: 30px;
        height: 28px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s ease;
        margin: 2px;
        border-radius: 3px;
        font-weight: bold;
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

    // Debug log render
    tracing::info!("🎨 TerminalTabs rendering with {} terminals", terminals.read().len());
    
    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%;",

            // Tab bar with scroll controls
            div {
                style: "{tab_bar_style}",
                
                // Left arrow button (only show if we can scroll left)
                if tab_scroll_offset.read().clone() > 0 {
                    button {
                        style: "{scroll_btn_style}",
                        onclick: move |_| {
                            let current = tab_scroll_offset.read().clone();
                            if current > 0 {
                                *tab_scroll_offset.write() = current - 1;
                            }
                        },
                        "‹"
                    }
                }
                
                // Tab container with overflow hidden
                div {
                    style: "flex: 1; display: flex; align-items: center; overflow: hidden; max-width: calc(100vw - 450px);", // Reserve space for right panel
                    
                    // Get terminal IDs in consistent order
                    {
                        let terminal_ids: Vec<String> = {
                            let mut ids: Vec<String> = terminals.read().keys().cloned().collect();
                            ids.sort(); // Ensure consistent ordering
                            ids
                        };
                        
                        // Get visible tabs based on scroll offset
                        let visible_tabs: Vec<(String, TerminalTab)> = terminal_ids
                            .iter()
                            .skip(tab_scroll_offset.read().clone())
                            .take(max_visible_tabs)
                            .filter_map(|id| {
                                terminals.read().get(id).map(|tab| (id.clone(), tab.clone()))
                            })
                            .collect();
                        
                        rsx! {
                            // Render visible terminal tabs
                            for (id, tab) in visible_tabs {
                                div {
                                    key: "{id}",
                                    style: "{tab_style(tab.is_active)}",
                                    onclick: {
                                        let id = id.clone();
                                        let mut terminals = terminals.clone();
                                        let mut active_terminal_id = active_terminal_id.clone();
                                        move |_| {
                                            set_active_terminal(&mut terminals, &mut active_terminal_id, &id, &mut tab_scroll_offset, max_visible_tabs);
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

                                    // Close button - don't allow closing Claude Code terminal if it's the only one
                                    if terminals.read().len() > 1 || (terminals.read().len() == 1 && id != "claude-code") {
                                        span {
                                            style: "{tab_close_style}",
                                            onmouseenter: |_| {
                                                // Mouse enter handler
                                            },
                                            onclick: {
                                                let id = id.clone();
                                                let mut terminals = terminals.clone();
                                                let mut active_terminal_id = active_terminal_id.clone();
                                                let mut tab_scroll_offset = tab_scroll_offset.clone();
                                                move |evt: MouseEvent| {
                                                    evt.stop_propagation();
                                                    close_terminal(&mut terminals, &mut active_terminal_id, &id);
                                                    
                                                    // Adjust scroll offset if necessary
                                                    let terminal_count = terminals.read().len();
                                                    let current_offset = tab_scroll_offset.read().clone();
                                                    if current_offset > 0 && current_offset >= terminal_count {
                                                        *tab_scroll_offset.write() = terminal_count.saturating_sub(max_visible_tabs);
                                                    }
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
                        }
                    }
                }
                
                // Right arrow button (only show if there are more tabs to the right)
                {
                    let terminal_count = terminals.read().len();
                    if terminal_count > tab_scroll_offset.read().clone() + max_visible_tabs {
                        rsx! {
                            button {
                                style: "{scroll_btn_style}",
                                onclick: move |_| {
                                    let current = tab_scroll_offset.read().clone();
                                    let total = terminals.read().len();
                                    if current + max_visible_tabs < total {
                                        *tab_scroll_offset.write() = current + 1;
                                    }
                                },
                                "›"
                            }
                        }
                    } else {
                        rsx! {
                            // Empty element when no right arrow needed
                        }
                    }
                }

                // New terminal button
                button {
                    style: "{new_terminal_btn_style}",
                    onclick: move |_| {
                        create_new_terminal(&mut terminals, &mut active_terminal_id, &mut terminal_counter, &mut tab_scroll_offset, max_visible_tabs);
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
                    style: "margin-left: auto; display: flex; gap: 4px; align-items: center;",
                    
                    // Keyboard shortcut hint
                    span {
                        style: "color: #969696; font-size: 11px; margin-right: 8px;",
                        if cfg!(target_os = "macos") { "Cmd+T to toggle" } else { "Ctrl+T to toggle" }
                    }
                    
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

            // Terminal content area - render all terminals but only show active
            div {
                style: "{terminal_container_style}",
                
                // Render all terminals but only display the active one
                for (id, terminal) in terminals.read().iter() {
                    div {
                        key: "{id}",
                        style: if Some(id) == active_terminal_id.read().as_ref() {
                            "display: block; height: 100%; width: 100%;"
                        } else {
                            "display: none;"
                        },
                        TerminalInstance {
                            terminal_id: id.clone(),
                            working_directory: terminal.working_directory.clone()
                        }
                    }
                }
                
                // Show message if no terminals exist
                if terminals.read().is_empty() {
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
    // Import the new VT100 Terminal component for full terminal emulation
    use super::terminal_vt100::TerminalVt100;
    
    // Each terminal instance maintains its own state through the terminal_id
    rsx! {
        div {
            key: "{terminal_id}",
            style: "height: 100%; width: 100%;",
            TerminalVt100 {
                terminal_id: terminal_id.clone(),
                initial_directory: Some(working_directory)
            }
        }
    }
}

/// Create a new terminal tab
fn create_new_terminal(
    terminals: &mut Signal<HashMap<String, TerminalTab>>,
    active_terminal_id: &mut Signal<Option<String>>,
    terminal_counter: &mut Signal<u32>,
    tab_scroll_offset: &mut Signal<usize>,
    max_visible_tabs: usize,
) {
    let count = *terminal_counter.read();
    let (id, title, icon) = if count == 1 {
        // First terminal is always Claude Code
        ("claude-code".to_string(), "Claude Code".to_string(), "🤖".to_string())
    } else {
        // Subsequent terminals are numbered starting from 1
        let terminal_number = count - 1;
        (format!("terminal-{}", terminal_number), format!("Terminal {}", terminal_number), "$".to_string())
    };
    
    let new_terminal = TerminalTab {
        id: id.clone(),
        title,
        icon,
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
    
    // Scroll to show the new tab if necessary
    let terminal_count = terminals.read().len();
    if terminal_count > max_visible_tabs {
        // Scroll to show the last tab (the new one)
        *tab_scroll_offset.write() = terminal_count.saturating_sub(max_visible_tabs);
    }
}

/// Set active terminal
fn set_active_terminal(
    terminals: &mut Signal<HashMap<String, TerminalTab>>,
    active_terminal_id: &mut Signal<Option<String>>,
    terminal_id: &str,
    tab_scroll_offset: &mut Signal<usize>,
    max_visible_tabs: usize,
) {
    // Deactivate all terminals
    for (_, terminal) in terminals.write().iter_mut() {
        terminal.is_active = false;
    }
    
    // Activate selected terminal
    if terminals.read().contains_key(terminal_id) {
        if let Some(terminal) = terminals.write().get_mut(terminal_id) {
            terminal.is_active = true;
        }
        active_terminal_id.set(Some(terminal_id.to_string()));
        
        // Ensure active tab is visible
        let mut terminal_ids: Vec<String> = terminals.read().keys().cloned().collect();
        terminal_ids.sort(); // Ensure consistent ordering
        
        if let Some(active_index) = terminal_ids.iter().position(|id| id == terminal_id) {
            let current_offset = tab_scroll_offset.read().clone();
            let visible_start = current_offset;
            let visible_end = current_offset + max_visible_tabs;
            
            // If active tab is before visible range, scroll left
            if active_index < visible_start {
                *tab_scroll_offset.write() = active_index;
            }
            // If active tab is after visible range, scroll right
            else if active_index >= visible_end {
                *tab_scroll_offset.write() = active_index.saturating_sub(max_visible_tabs - 1);
            }
        }
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
        let mut terminal_ids: Vec<String> = terminals.read().keys().cloned().collect();
        terminal_ids.sort(); // Ensure consistent ordering
        
        if let Some(next_id) = terminal_ids.first() {
            // Note: We can't call set_active_terminal here because it requires tab_scroll_offset
            // So we'll just update the active state directly
            for (_, terminal) in terminals.write().iter_mut() {
                terminal.is_active = false;
            }
            if let Some(terminal) = terminals.write().get_mut(next_id) {
                terminal.is_active = true;
                active_terminal_id.set(Some(next_id.to_string()));
            }
        } else {
            active_terminal_id.set(None);
        }
    }
}