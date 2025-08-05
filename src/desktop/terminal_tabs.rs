//! Terminal Tabs Component - VS Code-inspired multi-terminal management
//! 
//! This component provides a tabbed interface for multiple terminal instances,
//! allowing users to create, switch between, and manage multiple terminals.

use dioxus::prelude::*;
use dioxus::document::eval;
use std::collections::HashMap;
use crate::desktop::state::AppState;
use crate::desktop::consensus_integration::DesktopConsensusManager;
use crate::desktop::ai_cli_updater::{AiCliUpdaterDB, AuthStatus};
use crate::desktop::ai_cli_registry::get_enabled_ai_tools;
use crate::desktop::ai_cli_controller::{AiCliController, AiCliEvent, ToolStatus as ControllerToolStatus};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Terminal tab data
#[derive(Clone, Debug)]
pub struct TerminalTab {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub is_active: bool,
    pub working_directory: String,
}

/// AI tool tab data
#[derive(Clone, Debug, PartialEq)]
pub struct AiToolTab {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub command: String,
    pub status: ToolStatus,
    pub terminal_id: Option<String>,
    pub auth_status: AuthStatus,
}

/// Tool status
#[derive(Clone, Debug, PartialEq)]
pub enum ToolStatus {
    Available,      // Can be installed
    Installing,     // Currently installing
    Ready,          // Installed and ready
    Starting,       // Terminal starting
    Running,        // Terminal active
    Error(String),  // Installation/run error
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
    
    // AI tools state
    let mut ai_tools = use_signal(|| Vec::<AiToolTab>::new());
    let mut ai_tools_expanded = use_signal(|| true);
    let ai_cli_controller = use_signal(|| Option::<Arc<AiCliController>>::None);
    let ai_event_rx = use_signal(|| Option::<mpsc::UnboundedReceiver<AiCliEvent>>::None);
    
    // Initialize AI tools on component creation
    {
        // Create initial tool tabs immediately
        let tools = get_enabled_ai_tools();
        let mut tool_tabs = Vec::new();
        
        for tool in tools {
            tool_tabs.push(AiToolTab {
                id: tool.id,
                name: tool.name,
                icon: tool.icon,
                command: tool.command,
                status: ToolStatus::Available, // Will be updated by controller
                terminal_id: None,
                auth_status: tool.auth_status,
            });
        }
        
        ai_tools.set(tool_tabs);
    }
    
    // Create initial terminal on mount - separate effect for terminal creation
    use_effect({
        let mut terminals = terminals.clone();
        let mut active_terminal_id = active_terminal_id.clone();
        let mut terminal_counter = terminal_counter.clone();
        let mut tab_scroll_offset = tab_scroll_offset.clone();
        
        move || {
            tracing::info!("🔍 TerminalTabs component mounted - checking terminals");
            if terminals.read().is_empty() {
                tracing::info!("📝 Creating initial terminal");
                create_new_terminal(&mut terminals, &mut active_terminal_id, &mut terminal_counter, &mut tab_scroll_offset, max_visible_tabs);
            } else {
                tracing::info!("✅ Terminals already exist: {}", terminals.read().len());
            }
        }
    });
    
    // Initialize AI CLI controller - separate effect
    use_effect({
        let ai_tools = ai_tools.clone();
        let ai_cli_controller = ai_cli_controller.clone();
        let ai_event_rx = ai_event_rx.clone();
        
        move || {
            tracing::info!("🤖 Initializing AI CLI tools");
            
            // Initialize AI CLI controller using Dioxus spawn
            let mut ai_tools_clone = ai_tools.clone();
            let mut ai_cli_controller_signal = ai_cli_controller.clone();
            let mut ai_event_rx_signal = ai_event_rx.clone();
            
            dioxus::prelude::spawn(async move {
                // Create event channel
                let (event_tx, event_rx) = mpsc::unbounded_channel();
                
                // Store receiver first
                ai_event_rx_signal.set(Some(event_rx));
                
                // Create controller
                match AiCliController::new(event_tx).await {
                    Ok(controller) => {
                        let controller_arc = Arc::new(controller);
                        
                        // Get initial tool status
                        let tool_ids: Vec<String> = ai_tools_clone.read()
                            .iter()
                            .map(|t| t.id.clone())
                            .collect();
                        
                        // Check status of all tools
                        for tool_id in tool_ids {
                            if let Ok(status) = controller_arc.check_tool_status(&tool_id).await {
                                // Update tool status
                                let mut tools = ai_tools_clone.write();
                                if let Some(tool) = tools.iter_mut().find(|t| t.id == tool_id) {
                                    tool.status = match status {
                                        ControllerToolStatus::Available => ToolStatus::Available,
                                        ControllerToolStatus::Ready => ToolStatus::Ready,
                                        ControllerToolStatus::Error(e) => ToolStatus::Error(e),
                                        _ => tool.status.clone(),
                                    };
                                }
                            }
                        }
                        
                        // Store controller
                        ai_cli_controller_signal.set(Some(controller_arc));
                        
                        tracing::info!("✅ AI CLI controller initialized");
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to initialize AI CLI controller: {}", e);
                    }
                }
            });
        }
    });
    
    // Process AI CLI events - separate effect
    use_effect({
        let ai_tools = ai_tools.clone();
        let ai_event_rx = ai_event_rx.clone();
        
        move || {
            let mut ai_tools = ai_tools.clone();
            let mut ai_event_rx = ai_event_rx.clone();
            
            dioxus::prelude::spawn(async move {
                if let Some(mut rx) = ai_event_rx.write().take() {
                    while let Some(event) = rx.recv().await {
                    match event {
                        AiCliEvent::ToolStatusChanged { tool_id, status } => {
                            tracing::info!("📊 Tool status changed: {} -> {:?}", tool_id, status);
                            
                            let mut tools = ai_tools.write();
                            if let Some(tool) = tools.iter_mut().find(|t| t.id == tool_id) {
                                tool.status = match status {
                                    ControllerToolStatus::Available => ToolStatus::Available,
                                    ControllerToolStatus::Installing => ToolStatus::Installing,
                                    ControllerToolStatus::Ready => ToolStatus::Ready,
                                    ControllerToolStatus::Starting => ToolStatus::Starting,
                                    ControllerToolStatus::Running => ToolStatus::Running,
                                    ControllerToolStatus::Error(e) => ToolStatus::Error(e),
                                };
                            }
                        }
                        AiCliEvent::InstallationProgress { tool_id, message } => {
                            tracing::info!("📦 Installation progress for {}: {}", tool_id, message);
                        }
                        AiCliEvent::InstallationComplete { tool_id, success, error } => {
                            tracing::info!("✅ Installation complete for {}: success={}, error={:?}", 
                                tool_id, success, error);
                        }
                    }
                    }
                }
            });
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
    
    let ai_tools_section_style = "
        background: #252526;
        border-bottom: 1px solid #1e1e1e;
        padding: 8px;
    ";
    
    let ai_tools_header_style = "
        display: flex;
        align-items: center;
        gap: 6px;
        color: #cccccc;
        font-size: 13px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
        cursor: pointer;
        user-select: none;
        margin-bottom: 8px;
    ";
    
    let ai_tool_button_style = |status: &ToolStatus| format!(
        "
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 6px 12px;
        margin: 2px 4px;
        background: {};
        color: {};
        border: 1px solid {};
        border-radius: 4px;
        cursor: {};
        font-size: 12px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
        transition: all 0.2s ease;
        ",
        match status {
            ToolStatus::Ready => "#007acc",
            ToolStatus::Available => "#3c3c3c",
            ToolStatus::Installing => "#d79922",
            ToolStatus::Running => "#4ec9b0",
            ToolStatus::Error(_) => "#f44747",
            _ => "#3c3c3c",
        },
        match status {
            ToolStatus::Error(_) => "#ffffff",
            _ => "#cccccc",
        },
        match status {
            ToolStatus::Ready => "#0098ff",
            ToolStatus::Available => "#5a5a5a",
            ToolStatus::Installing => "#f0ab00",
            ToolStatus::Running => "#6edcd2",
            ToolStatus::Error(_) => "#ff6b6b",
            _ => "#5a5a5a",
        },
        match status {
            ToolStatus::Installing => "wait",
            _ => "pointer",
        }
    );

    // Debug log render
    tracing::info!("🎨 TerminalTabs rendering with {} terminals", terminals.read().len());
    
    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%;",

            // AI Tools Section
            if ai_tools.read().len() > 0 {
                div {
                    style: "{ai_tools_section_style}",
                    
                    // Collapsible header
                    div {
                        style: "{ai_tools_header_style}",
                        onclick: move |_| {
                            let current = *ai_tools_expanded.read();
                            ai_tools_expanded.set(!current);
                        },
                        
                        // Chevron icon
                        span {
                            style: "font-size: 10px; transition: transform 0.2s;",
                            style: if *ai_tools_expanded.read() { 
                                "transform: rotate(90deg);" 
                            } else { 
                                "transform: rotate(0deg);" 
                            },
                            "▶"
                        }
                        
                        span { "AI CLI Tools" }
                        
                        // Tool count
                        span {
                            style: "margin-left: auto; opacity: 0.6; font-size: 11px;",
                            "{ai_tools.read().len()}"
                        }
                    }
                    
                    // Tool buttons (only show when expanded)
                    if *ai_tools_expanded.read() {
                        div {
                            style: "display: flex; flex-wrap: wrap;",
                            
                            for tool in ai_tools.read().iter() {
                                button {
                                    key: "{tool.id}",
                                    style: "{ai_tool_button_style(&tool.status)}",
                                    disabled: matches!(tool.status, ToolStatus::Installing),
                                    onclick: {
                                        let tool_id = tool.id.clone();
                                        let mut ai_tools = ai_tools.clone();
                                        let ai_cli_controller = ai_cli_controller.clone();
                                        let mut terminals = terminals.clone();
                                        let mut active_terminal_id = active_terminal_id.clone();
                                        let mut terminal_counter = terminal_counter.clone();
                                        let mut tab_scroll_offset = tab_scroll_offset.clone();
                                        move |_| {
                                            activate_ai_tool(
                                                &tool_id,
                                                &mut ai_tools,
                                                &ai_cli_controller,
                                                &mut terminals,
                                                &mut active_terminal_id,
                                                &mut terminal_counter,
                                                &mut tab_scroll_offset,
                                                max_visible_tabs,
                                            );
                                        }
                                    },
                                    
                                    // Tool icon
                                    span { "{tool.icon}" }
                                    
                                    // Tool name
                                    span { "{tool.name}" }
                                    
                                    // Status indicator
                                    match &tool.status {
                                        ToolStatus::Installing => rsx! {
                                            span {
                                                style: "margin-left: 4px; animation: spin 1s linear infinite;",
                                                "⏳"
                                            }
                                        },
                                        ToolStatus::Error(msg) => rsx! {
                                            span {
                                                style: "margin-left: 4px;",
                                                title: "{msg}",
                                                "❌"
                                            }
                                        },
                                        ToolStatus::Running => rsx! {
                                            span {
                                                style: "margin-left: 4px; color: #4ec9b0;",
                                                "●"
                                            }
                                        },
                                        _ => rsx! {}
                                    }
                                    
                                    // Auth status indicator
                                    match &tool.auth_status {
                                        AuthStatus::Required { instructions } => rsx! {
                                            span {
                                                style: "margin-left: 4px; opacity: 0.7;",
                                                title: "{instructions}",
                                                "🔐"
                                            }
                                        },
                                        AuthStatus::Invalid { error } => rsx! {
                                            span {
                                                style: "margin-left: 4px; color: #f44747;",
                                                title: "{error}",
                                                "⚠️"
                                            }
                                        },
                                        _ => rsx! {}
                                    }
                                }
                            }
                        }
                    }
                }
            }

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

                                    // Close button - always allow closing terminals as long as there's more than one
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
                                    
                                    // Safely close terminal with error handling
                                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                        close_terminal(&mut terminals, &mut active_terminal_id_mut, &active_id);
                                    })) {
                                        Ok(_) => tracing::info!("✅ Terminal closed successfully"),
                                        Err(e) => {
                                            tracing::error!("❌ Failed to close terminal: {:?}", e);
                                            // Don't crash the GUI - just log the error
                                        }
                                    }
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
                
                // Render all terminals but only show the active one
                for (id, terminal) in terminals.read().iter() {
                    div {
                        key: "{id}",
                        style: if Some(id) == active_terminal_id.read().as_ref() {
                            "position: relative; height: 100%; width: 100%; visibility: visible; z-index: 1;"
                        } else {
                            "position: absolute; top: 0; left: 0; height: 100%; width: 100%; visibility: hidden; z-index: 0;"
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
    // Import the xterm.js Terminal component
    use super::terminal_xterm_simple::TerminalXterm;
    
    // Each terminal instance maintains its own state through the terminal_id
    rsx! {
        div {
            key: "{terminal_id}",
            style: "height: 100%; width: 100%;",
            TerminalXterm {
                terminal_id: terminal_id.clone(),
                initial_directory: Some(working_directory),
                command: None,
                args: vec![]
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
    let count = terminal_counter.peek().clone();
    // All terminals follow the same numbering pattern
    let (id, title, icon) = {
        (format!("terminal-{}", count), format!("Terminal {}", count), "$".to_string())
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
        
        // Refresh the terminal to fix rendering issues when switching
        let terminal_id_refresh = terminal_id.to_string();
        spawn(async move {
            // Small delay to ensure DOM update completes
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            
            let refresh_script = format!(r#"
                if (window.terminals && window.terminals['{}']) {{
                    const term = window.terminals['{}'];
                    // Force xterm.js to refresh its rendering
                    term.refresh(0, term.rows - 1);
                    // Ensure fit addon recalculates dimensions
                    if (term._addonManager && term._addonManager._addons) {{
                        const fitAddon = Object.values(term._addonManager._addons)
                            .find(addon => addon.instance && addon.instance.fit);
                        if (fitAddon && fitAddon.instance.fit) {{
                            fitAddon.instance.fit();
                        }}
                    }}
                    // Scroll to bottom to show latest content
                    term.scrollToBottom();
                    console.log('🔄 Refreshed terminal: {}');
                }}
            "#, terminal_id_refresh, terminal_id_refresh, terminal_id_refresh);
            
            if let Err(e) = eval(&refresh_script).await {
                tracing::warn!("Failed to refresh terminal {}: {}", terminal_id_refresh, e);
            }
        });
        
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
    // Don't allow closing the last terminal
    if terminals.read().len() <= 1 {
        tracing::warn!("⚠️ Cannot close the last terminal - at least one must remain open");
        return;
    }
    
    // First, unregister from global terminal registry to prevent dangling references
    use crate::desktop::terminal_registry::unregister_terminal;
    use crate::desktop::terminal_buffer::unregister_terminal_buffer;
    unregister_terminal(terminal_id);
    unregister_terminal_buffer(terminal_id);
    tracing::info!("🗑️ Closing terminal: {}", terminal_id);
    
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

/// Activate an AI tool - install if needed, then launch in terminal
fn activate_ai_tool(
    tool_id: &str,
    ai_tools: &mut Signal<Vec<AiToolTab>>,
    ai_cli_controller: &Signal<Option<Arc<AiCliController>>>,
    terminals: &mut Signal<HashMap<String, TerminalTab>>,
    active_terminal_id: &mut Signal<Option<String>>,
    terminal_counter: &mut Signal<u32>,
    tab_scroll_offset: &mut Signal<usize>,
    max_visible_tabs: usize,
) {
    tracing::info!("🚀 Activating AI tool: {}", tool_id);
    
    // Find the tool
    let tool_index = ai_tools.read().iter().position(|t| t.id == tool_id);
    if tool_index.is_none() {
        tracing::error!("Tool not found: {}", tool_id);
        return;
    }
    
    let tool = ai_tools.read()[tool_index.unwrap()].clone();
    
    // If tool is already running, switch to its terminal
    if let ToolStatus::Running = tool.status {
        if let Some(terminal_id) = &tool.terminal_id {
            set_active_terminal(terminals, active_terminal_id, terminal_id, tab_scroll_offset, max_visible_tabs);
            return;
        }
    }
    
    // If tool is not ready, install it first
    if !matches!(tool.status, ToolStatus::Ready) {
        // Get controller
        let controller = ai_cli_controller.read().clone();
        if controller.is_none() {
            tracing::error!("AI CLI controller not initialized");
            return;
        }
        
        let controller = controller.unwrap();
        let tool_id_clone = tool_id.to_string();
        
        // Install tool using Dioxus spawn
        dioxus::prelude::spawn(async move {
            tracing::info!("📦 Starting installation for tool: {}", tool_id_clone);
            
            if let Err(e) = controller.install_tool(tool_id_clone.clone()).await {
                tracing::error!("❌ Failed to install tool {}: {}", tool_id_clone, e);
            }
        });
        
        return;
    }
    
    // Tool is ready, create a terminal for it
    let terminal_id = format!("ai-tool-{}", tool_id);
    let title = tool.name.clone();
    let icon = tool.icon.clone();
    
    // Create new terminal tab
    let new_terminal = TerminalTab {
        id: terminal_id.clone(),
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
    
    terminals.write().insert(terminal_id.clone(), new_terminal);
    active_terminal_id.set(Some(terminal_id.clone()));
    
    // Update tool status
    {
        let mut tools = ai_tools.write();
        if let Some(tool) = tools.iter_mut().find(|t| t.id == tool_id) {
            tool.status = ToolStatus::Running;
            tool.terminal_id = Some(terminal_id);
        }
    }
    
    // Scroll to show the new tab if necessary
    let terminal_count = terminals.read().len();
    if terminal_count > max_visible_tabs {
        *tab_scroll_offset.write() = terminal_count.saturating_sub(max_visible_tabs);
    }
}