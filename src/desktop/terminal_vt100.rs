//! VT100-based Terminal Emulator for Dioxus
//!
//! A real terminal emulator using portable-pty and vt100 parser
//! that provides full terminal capabilities for interactive CLI tools

use dioxus::prelude::*;
use dioxus::document::eval;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write, ErrorKind};
use crate::desktop::terminal_registry::{register_terminal, unregister_terminal};
use crate::desktop::state::{AppState, ChatMessage, MessageType, MessageMetadata};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use vt100;

/// Terminal emulator state
pub struct Vt100Terminal {
    parser: Arc<Mutex<vt100::Parser>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    size: (u16, u16), // (cols, rows)
}

/// Terminal component using vt100 parser
#[component]
pub fn TerminalVt100(
    terminal_id: String,
    initial_directory: Option<String>,
) -> Element {
    // Terminal state
    let mut terminal_state = use_signal(|| None::<Arc<Vt100Terminal>>);
    let mut screen_html = use_signal(|| String::from(r#"<div style="color: #666; padding: 10px;">Initializing terminal...</div>"#));
    let mut update_trigger = use_signal(|| 0u32);
    let mut is_ready = use_signal(|| false);
    
    // Cleanup on unmount
    let terminal_id_for_cleanup = terminal_id.clone();
    use_drop(move || {
        unregister_terminal(&terminal_id_for_cleanup);
    });
    
    // Initialize terminal
    let terminal_id_for_init = terminal_id.clone();
    use_effect(move || {
        if terminal_state.read().is_none() {
            tracing::info!("🚀 Initializing VT100 terminal for {}", terminal_id_for_init.clone());
            match create_terminal(initial_directory.clone()) {
                Ok((terminal, mut output_rx)) => {
                    let terminal_arc = Arc::new(terminal);
                    terminal_state.set(Some(terminal_arc.clone()));
                    
                    // Register terminal in global registry with writer
                    register_terminal(
                        terminal_id_for_init.clone(), 
                        terminal_arc.parser.clone(),
                        Some(terminal_arc.writer.clone())
                    );
                    
                    // Set terminal as ready immediately - don't wait for shell prompt
                    is_ready.set(true);
                    tracing::info!("🚀 Terminal ready immediately - accepting input");
                    
                    // Spawn output handler
                    let mut screen_html = screen_html.clone();
                    let mut update_trigger = update_trigger.clone();
                    spawn(async move {
                        while let Some(()) = output_rx.recv().await {
                            // Trigger re-render
                            update_trigger.set(update_trigger() + 1);
                        }
                    });
                    
                    // Ensure terminal gets focus after initialization
                    // Try multiple times to ensure focus is acquired
                    spawn(async move {
                        for i in 0..5 {
                            tokio::time::sleep(Duration::from_millis(50 + i * 50)).await;
                            let result = eval(r#"
                                const terminal = document.querySelector('.terminal-vt100[tabindex="0"]');
                                if (terminal && document.activeElement !== terminal) {
                                    terminal.focus();
                                    console.log('Terminal auto-focused after init');
                                    return true;
                                } else if (terminal && document.activeElement === terminal) {
                                    console.log('Terminal already has focus');
                                    return true;
                                }
                                return false;
                            "#).await;
                            
                            // If focus was successful, break
                            if let Ok(focused) = result {
                                if focused == "true" {
                                    tracing::info!("✅ Terminal focus acquired on attempt {}", i + 1);
                                    break;
                                }
                            }
                        }
                    });
                    
                    tracing::info!("✅ VT100 terminal initialized successfully for {}", terminal_id_for_init);
                }
                Err(e) => {
                    tracing::error!("Failed to create terminal: {}", e);
                }
            }
        }
    });
    
    // Update screen HTML when triggered
    let terminal_id_for_update = terminal_id.clone();
    use_effect(move || {
        let trigger_value = update_trigger(); // Subscribe to updates
        if let Some(terminal) = terminal_state.read().as_ref() {
            if let Ok(parser) = terminal.parser.lock() {
                let html = render_screen(&parser);
                screen_html.set(html.clone());
                if !html.is_empty() {
                    tracing::debug!("🖥️ Terminal screen updated (trigger: {})", trigger_value);
                }
            }
        }
    });
    
    // Handle keyboard input
    let handle_keydown = {
        let terminal_state = terminal_state.clone();
        let is_ready = is_ready.clone();
        let terminal_id_for_input = terminal_id.clone();
        let mut app_state = use_context::<Signal<AppState>>();
        
        move |evt: Event<KeyboardData>| {
            // Log immediately to verify events are being received
            tracing::info!("🎹 Keyboard event received: key={:?}, ready={}", evt.key(), is_ready());
            
            // IMPORTANT: Do NOT stop propagation - let it bubble up naturally
            // Only prevent default to stop browser behavior
            evt.prevent_default();
            
            // Only process input if terminal is ready
            if !is_ready() {
                tracing::warn!("⏳ Terminal not ready yet, ignoring input");
                return;
            }
            
            if let Some(terminal) = terminal_state.read().as_ref() {
                if let Some(input) = keyboard_to_bytes(&evt) {
                    let input_str = String::from_utf8_lossy(&input);
                    tracing::debug!("⌨️ Sending keyboard input: {:?}", input_str);
                    
                    // Send input to terminal
                    if let Ok(mut writer) = terminal.writer.lock() {
                        match writer.write_all(&input) {
                            Ok(_) => {
                                match writer.flush() {
                                    Ok(_) => {
                                        tracing::debug!("✅ Keyboard input sent and flushed successfully");
                                    }
                                    Err(e) => {
                                        tracing::error!("❌ Failed to flush after keyboard input: {}", e);
                                    }
                                }
                            }
                            Err(e) => tracing::error!("❌ Failed to send keyboard input: {}", e),
                        }
                    } else {
                        tracing::error!("❌ Failed to acquire writer lock for keyboard input");
                    }
                }
            }
        }
    };
    
    // Terminal container style - simplified to avoid any CSS performance issues
    let container_style = "
        width: 100%;
        height: 100%;
        background: #000000;
        color: #cccccc;
        font-family: monospace;
        font-size: 14px;
        line-height: 18px;
        overflow-y: auto;
        padding: 8px;
        box-sizing: border-box;
        white-space: pre;
        cursor: text;
    ";
    
    rsx! {
        div {
            class: "terminal-vt100",
            style: "{container_style}",
            tabindex: "0",
            autofocus: "true",
            onkeydown: handle_keydown,
            onkeyup: move |_| {
                // Keep focus on terminal
                tracing::trace!("Key up event");
            },
            onclick: move |_| {
                // Focus terminal on click
                tracing::debug!("Terminal clicked, ensuring focus");
                
                // Use eval to ensure the terminal has focus
                spawn(async move {
                    let _ = eval(r#"
                        const terminal = document.querySelector('.terminal-vt100[tabindex="0"]');
                        if (terminal && document.activeElement !== terminal) {
                            terminal.focus();
                            console.log('Terminal focused on click');
                        }
                    "#).await;
                });
            },
            onmounted: move |evt| {
                // Auto-focus the terminal when mounted
                tracing::info!("🎯 Terminal mounted, requesting focus");
                
                // Use JavaScript eval to focus the element after a short delay
                // This ensures the element is fully ready in the DOM
                spawn(async move {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    let _ = eval(r#"
                        const terminal = document.querySelector('.terminal-vt100[tabindex="0"]');
                        if (terminal) {
                            terminal.focus();
                            console.log('Terminal focused via eval on mount');
                        } else {
                            console.error('Terminal element not found');
                        }
                    "#).await;
                    tracing::info!("🎯 Focus script executed via eval");
                });
            },
            onfocusin: move |_| {
                tracing::info!("✅ Terminal gained focus");
            },
            onfocusout: move |_| {
                tracing::info!("❌ Terminal lost focus");
            },
            
            // Render terminal screen without JavaScript
            div {
                dangerous_inner_html: "{screen_html}",
                style: "width: 100%; height: 100%; pointer-events: none;"
            }
        }
    }
}

/// Create a new terminal instance
fn create_terminal(
    working_directory: Option<String>,
) -> Result<(Vt100Terminal, mpsc::UnboundedReceiver<()>), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    // Terminal size - use a wider default for modern screens
    let cols = 120;
    let rows = 30;
    
    // Create PTY system
    tracing::debug!("⏱️ Creating PTY system...");
    let pty_system = NativePtySystem::default();
    
    // Set up command
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    tracing::info!("🐚 Using shell: {}", shell);
    let mut cmd = CommandBuilder::new(shell.clone());
    
    // Set working directory
    if let Some(dir) = working_directory {
        cmd.cwd(dir);
    }
    
    // Set environment variables for proper terminal support
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    // Disable shell RC files for faster startup (optional - comment out if you need shell customization)
    // cmd.env("BASH_ENV", "");
    // cmd.env("ENV", "");
    
    tracing::debug!("⏱️ Creating PTY with size {}x{} ({}ms elapsed)", cols, rows, start_time.elapsed().as_millis());
    
    // Create PTY pair
    let pty_pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: cols * 8,
        pixel_height: rows * 16,
    })?;
    
    tracing::debug!("⏱️ PTY created ({}ms elapsed)", start_time.elapsed().as_millis());
    
    // Spawn the shell
    tracing::info!("⏱️ Spawning shell process... ({}ms elapsed)", start_time.elapsed().as_millis());
    let _child = pty_pair.slave.spawn_command(cmd)?;
    std::mem::drop(pty_pair.slave); // Close slave side
    
    tracing::debug!("⏱️ Shell spawned ({}ms elapsed)", start_time.elapsed().as_millis());
    
    // Get reader and writer
    let reader = pty_pair.master.try_clone_reader()?;
    let writer = pty_pair.master.take_writer()?;
    
    // IMPORTANT: Don't send any input immediately - let the shell initialize naturally
    tracing::debug!("⏱️ PTY reader/writer obtained ({}ms elapsed)", start_time.elapsed().as_millis());
    
    // Create vt100 parser with larger scrollback buffer
    let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 10000)));
    
    // Create update channel
    let (tx, rx) = mpsc::unbounded_channel();
    
    // Spawn reader thread - simplified to avoid blocking issues
    let parser_for_reader = Arc::clone(&parser);
    thread::spawn(move || {
        tracing::debug!("📖 Reader thread started");
        
        let mut reader = reader;
        let mut buf = vec![0u8; 4096];
        
        // Simple read loop - let the shell initialize naturally
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    tracing::info!("PTY EOF - shell process ended");
                    break;
                }
                Ok(n) => {
                    // Process bytes through vt100 parser
                    if let Ok(mut parser) = parser_for_reader.lock() {
                        parser.process(&buf[..n]);
                        tracing::trace!("📝 Processed {} bytes from PTY", n);
                    }
                    // Notify UI to update
                    let _ = tx.send(());
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    // Interrupted, retry immediately
                    continue;
                }
                Err(e) => {
                    tracing::error!("Error reading from PTY: {}", e);
                    break;
                }
            }
        }
        
        tracing::debug!("📖 Reader thread exiting");
    });
    
    Ok((
        Vt100Terminal {
            parser,
            writer: Arc::new(Mutex::new(writer)),
            size: (cols, rows),
        },
        rx,
    ))
}

/// Convert keyboard event to terminal input bytes
fn keyboard_to_bytes(event: &Event<KeyboardData>) -> Option<Vec<u8>> {
    let key = event.key();
    let _shift = event.modifiers().shift();
    let ctrl = event.modifiers().ctrl();
    let alt = event.modifiers().alt();
    
    match key {
        Key::Enter => Some(b"\r".to_vec()),
        Key::Tab => Some(b"\t".to_vec()),
        Key::Backspace => Some(vec![0x7f]),
        Key::Escape => Some(b"\x1b".to_vec()),
        
        // Arrow keys
        Key::ArrowUp => Some(b"\x1b[A".to_vec()),
        Key::ArrowDown => Some(b"\x1b[B".to_vec()),
        Key::ArrowRight => Some(b"\x1b[C".to_vec()),
        Key::ArrowLeft => Some(b"\x1b[D".to_vec()),
        
        // Page Up/Down
        Key::PageUp => Some(b"\x1b[5~".to_vec()),
        Key::PageDown => Some(b"\x1b[6~".to_vec()),
        
        // Home/End
        Key::Home => Some(b"\x1b[H".to_vec()),
        Key::End => Some(b"\x1b[F".to_vec()),
        
        // Character keys
        Key::Character(s) => {
            if s.len() == 1 {
                let ch = s.chars().next().unwrap();
                
                // Handle Ctrl+key
                if ctrl && !alt {
                    match ch {
                        'a'..='z' => Some(vec![(ch as u8) - b'a' + 1]),
                        'A'..='Z' => Some(vec![(ch.to_lowercase().next().unwrap() as u8) - b'a' + 1]),
                        'c' | 'C' => Some(vec![0x03]), // Ctrl+C
                        'd' | 'D' => Some(vec![0x04]), // Ctrl+D
                        'l' | 'L' => Some(vec![0x0c]), // Ctrl+L (clear)
                        _ => Some(s.as_bytes().to_vec()),
                    }
                } else if alt {
                    // Alt+key sends ESC followed by key
                    let mut bytes = vec![0x1b];
                    bytes.extend_from_slice(s.as_bytes());
                    Some(bytes)
                } else {
                    Some(s.as_bytes().to_vec())
                }
            } else {
                Some(s.as_bytes().to_vec())
            }
        }
        
        _ => None,
    }
}

/// Get plain text content from terminal including as much history as possible
pub fn get_terminal_text(parser: &vt100::Parser) -> String {
    // VT100 parser only gives us access to the visible screen buffer
    // For full scrollback, we'd need to implement our own buffer
    let screen = parser.screen();
    let (rows, cols) = screen.size();
    let mut text = String::new();
    
    // Get the visible screen content
    for row in 0..rows {
        let mut line = String::new();
        for col in 0..cols {
            if let Some(cell) = screen.cell(row, col) {
                line.push_str(&cell.contents());
            }
        }
        // Trim trailing whitespace from each line
        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            text.push_str(trimmed);
            text.push('\n');
        }
    }
    
    text.trim().to_string()
}

/// Render vt100 screen to HTML
fn render_screen(parser: &vt100::Parser) -> String {
    let screen = parser.screen();
    let mut html = String::new();
    
    // Add styles including cursor animation
    html.push_str(r#"
    <style>
        @keyframes blink {
            0%, 50% { opacity: 1; }
            51%, 100% { opacity: 0; }
        }
        .terminal-cursor {
            background-color: #cccccc;
            animation: blink 1s step-end infinite;
        }
    </style>
    "#);
    
    html.push_str(r#"<div style="font-family: monospace; white-space: pre; position: relative;">"#);
    
    // Get cursor position
    let (cursor_row, cursor_col) = screen.cursor_position();
    let cursor_row = cursor_row as usize;
    let cursor_col = cursor_col as usize;
    
    // Process screen with colors and formatting
    for row in 0..screen.size().0 {
        if row > 0 {
            html.push_str("<br>");
        }
        
        for col in 0..screen.size().1 {
            let is_cursor = row as usize == cursor_row && col as usize == cursor_col;
            
            if let Some(cell) = screen.cell(row, col) {
                let ch = cell.contents();
                let character = if ch.is_empty() { " " } else { &ch };
                
                // Get cell attributes
                let mut style_parts = Vec::new();
                
                // Foreground color
                let fg = cell.fgcolor();
                if fg != vt100::Color::Default {
                    style_parts.push(format!("color: {}", vt100_color_to_css(fg)));
                }
                
                // Background color  
                let bg = cell.bgcolor();
                if bg != vt100::Color::Default {
                    style_parts.push(format!("background-color: {}", vt100_color_to_css(bg)));
                }
                
                // Bold
                if cell.bold() {
                    style_parts.push("font-weight: bold".to_string());
                }
                
                // Italic
                if cell.italic() {
                    style_parts.push("font-style: italic".to_string());
                }
                
                // Underline
                if cell.underline() {
                    style_parts.push("text-decoration: underline".to_string());
                }
                
                // Render cell with styling
                if is_cursor {
                    html.push_str(r#"<span class="terminal-cursor">"#);
                } else if !style_parts.is_empty() {
                    html.push_str(&format!(r#"<span style="{}">"#, style_parts.join("; ")));
                }
                
                // Escape HTML characters
                match character {
                    "<" => html.push_str("&lt;"),
                    ">" => html.push_str("&gt;"),
                    "&" => html.push_str("&amp;"),
                    " " if is_cursor => html.push('▂'),
                    _ => html.push_str(character),
                }
                
                if is_cursor || !style_parts.is_empty() {
                    html.push_str("</span>");
                }
            } else {
                // Empty cell
                if is_cursor {
                    html.push_str(r#"<span class="terminal-cursor">▂</span>"#);
                } else {
                    html.push(' ');
                }
            }
        }
    }
    
    html.push_str("</div>");
    
    // Add a marker element at the end for auto-scrolling (no JavaScript)
    
    html
}

/// Convert vt100 color to CSS color
fn vt100_color_to_css(color: vt100::Color) -> String {
    match color {
        vt100::Color::Default => "#cccccc".to_string(),
        vt100::Color::Idx(idx) => match idx {
            0 => "#000000".to_string(),   // Black
            1 => "#cd3131".to_string(),   // Red
            2 => "#0dbc79".to_string(),   // Green
            3 => "#e5e510".to_string(),   // Yellow
            4 => "#2472c8".to_string(),   // Blue
            5 => "#bc3fbc".to_string(),   // Magenta
            6 => "#11a8cd".to_string(),   // Cyan
            7 => "#e5e5e5".to_string(),   // White
            8 => "#666666".to_string(),   // Bright Black
            9 => "#f14c4c".to_string(),   // Bright Red
            10 => "#23d18b".to_string(),  // Bright Green
            11 => "#f5f543".to_string(),  // Bright Yellow
            12 => "#3b8eea".to_string(),  // Bright Blue
            13 => "#d670d6".to_string(),  // Bright Magenta
            14 => "#29b8db".to_string(),  // Bright Cyan
            15 => "#e5e5e5".to_string(),  // Bright White
            // Extended 256 color palette - just use default for now
            _ => "#cccccc".to_string(),
        },
        vt100::Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
    }
}

