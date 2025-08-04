//! Simplified xterm.js Terminal Emulator for Dioxus
//!
//! This provides a full-featured terminal using xterm.js in the WebView.

use dioxus::prelude::*;
use dioxus::document::eval;
use dioxus::events::{KeyboardData, Key};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use crate::desktop::terminal_registry::{register_terminal, unregister_terminal};
use crate::desktop::terminal_buffer::{register_terminal_buffer, add_to_terminal_buffer, unregister_terminal_buffer};
use once_cell::sync::Lazy;
use base64;
use std::collections::HashMap;

static OUTPUT_QUEUES: Lazy<Mutex<HashMap<String, Vec<String>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static PTY_HANDLES: Lazy<Mutex<HashMap<String, Arc<Mutex<Box<dyn MasterPty + Send>>>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Terminal emulator state
pub struct XtermTerminal {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

/// Terminal component using xterm.js
#[component]
pub fn TerminalXterm(
    terminal_id: String,
    initial_directory: Option<String>,
    #[props(default = None)] command: Option<String>,
    #[props(default = vec![])] args: Vec<String>,
) -> Element {
    let mut terminal_writer = use_signal(|| None::<Arc<Mutex<Box<dyn Write + Send>>>>);
    let mut is_initialized = use_signal(|| false);
    let terminal_div_id = format!("xterm-{}", terminal_id);
    
    // Cleanup on unmount
    let terminal_id_cleanup = terminal_id.clone();
    use_drop(move || {
        unregister_terminal(&terminal_id_cleanup);
        unregister_terminal_buffer(&terminal_id_cleanup);
        
        // Clean up PTY handle
        if let Ok(mut pty_handles) = PTY_HANDLES.lock() {
            pty_handles.remove(&terminal_id_cleanup);
            tracing::info!("🗑️ Cleaned up PTY handle for terminal {}", terminal_id_cleanup);
        }
        
        // Also clean up the output queue for this terminal
        if let Ok(mut queues) = OUTPUT_QUEUES.lock() {
            queues.remove(&terminal_id_cleanup);
            tracing::info!("🗑️ Cleaned up output queue for terminal {}", terminal_id_cleanup);
        }
    });
    
    // Initialize terminal
    let terminal_id_init = terminal_id.clone();
    let div_id_init = terminal_div_id.clone();
    use_effect(move || {
        if !is_initialized() {
            let tid = terminal_id_init.clone();
            let div_id = div_id_init.clone();
            let initial_dir = initial_directory.clone();
            let is_claude_code = tid == "claude-code";
            let cmd = command.clone();
            let cmd_args = args.clone();
            
            spawn(async move {
                // Initialize PTY
                if let Ok((writer, mut reader, pty_handle)) = create_pty(initial_dir, is_claude_code, cmd, cmd_args) {
                    terminal_writer.set(Some(writer.clone()));
                    
                    // Store PTY handle for resizing
                    if let Ok(mut pty_handles) = PTY_HANDLES.lock() {
                        pty_handles.insert(tid.clone(), pty_handle);
                        tracing::info!("📝 Stored PTY handle for terminal {}", tid);
                    }
                    
                    // Register in global registry
                    register_terminal(
                        tid.clone(),
                        Some(writer)
                    );
                    
                    // Register terminal buffer for output capture
                    register_terminal_buffer(tid.clone());
                    
                    // Initialize xterm.js
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    init_xterm(&div_id, &tid).await;
                    
                    // Start the output processor for this specific terminal
                    let tid_processor = tid.clone();
                    spawn(async move {
                        tracing::info!("🔄 Starting output processor for terminal {}", tid_processor);
                        loop {
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                            process_terminal_output_queue(&tid_processor).await;
                        }
                    });
                    
                    // Handle output in a separate task
                    let tid_output = tid.clone();
                    tokio::task::spawn_blocking(move || {
                        tracing::info!("📖 PTY reader thread started for {}", tid_output);
                        let mut buf = vec![0u8; 4096];
                        loop {
                            match reader.read(&mut buf) {
                                Ok(0) => {
                                    tracing::warn!("PTY reader got EOF");
                                    break;
                                }
                                Ok(n) => {
                                    tracing::info!("PTY read {} bytes", n);
                                    if let Ok(text) = String::from_utf8(buf[..n].to_vec()) {
                                        tracing::debug!("PTY text (UTF-8): {:?}", text);
                                        write_to_xterm(&tid_output, &text);
                                    } else {
                                        // Handle non-UTF8 data
                                        let text = String::from_utf8_lossy(&buf[..n]);
                                        tracing::debug!("PTY text (lossy): {:?}", text);
                                        write_to_xterm(&tid_output, &text);
                                    }
                                }
                                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                                Err(e) => {
                                    tracing::error!("PTY reader error: {}", e);
                                    break;
                                }
                            }
                        }
                        tracing::info!("📕 PTY reader thread ended for {}", tid_output);
                    });
                    
                    // Mark as initialized immediately - don't wait for shell prompt
                    is_initialized.set(true);
                    tracing::info!("🚀 Terminal ready immediately - accepting input");
                    
                    // For Claude Code terminal, show usage instructions after shell loads
                    if is_claude_code {
                        let writer_for_help = terminal_writer.read().clone();
                        spawn(async move {
                            // Wait for shell to be ready
                            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                            
                            // Send a clear command first to avoid interference
                            if let Some(writer_ref) = writer_for_help.as_ref() {
                                if let Ok(mut w) = writer_ref.lock() {
                                    let _ = w.write_all(b"clear\r");
                                    let _ = w.flush();
                                }
                            }
                            
                            // Wait a bit more for clear to process
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                            
                            // Send help using printf for proper ANSI handling
                            if let Some(writer_ref) = writer_for_help.as_ref() {
                                if let Ok(mut w) = writer_ref.lock() {
                                    // Use printf to properly handle ANSI escape sequences
                                    let help_commands = vec![
                                        "printf '\\033[38;2;137;180;250m🤖 Claude Code Terminal\\033[0m\\n'\r",
                                        "printf '\\033[38;2;166;227;161mType:\\033[0m claude \"your prompt\" to ask Claude\\n'\r",
                                        "printf '\\033[38;2;186;187;241mExample:\\033[0m claude \"explain this code\"\\n'\r",
                                    ];
                                    
                                    for cmd in help_commands {
                                        let _ = w.write_all(cmd.as_bytes());
                                    }
                                    let _ = w.flush();
                                }
                            }
                        });
                    }
                    
                    // Focus this specific terminal after initialization
                    let div_id_focus = div_id.clone();
                    let tid_focus = tid.clone();
                    spawn(async move {
                        for i in 0..5 {
                            tokio::time::sleep(std::time::Duration::from_millis(50 + i * 50)).await;
                            let script = format!(r#"
                                const terminalDiv = document.getElementById('{}').parentElement;
                                if (terminalDiv && document.activeElement !== terminalDiv) {{
                                    terminalDiv.focus();
                                    console.log('Terminal {} auto-focused after init');
                                    return true;
                                }} else if (terminalDiv && document.activeElement === terminalDiv) {{
                                    console.log('Terminal {} already has focus');
                                    return true;
                                }}
                                return false;
                            "#, div_id_focus, tid_focus, tid_focus);
                            
                            if let Ok(result) = eval(&script).await {
                                if result.as_str() == Some("true") {
                                    tracing::info!("✅ Terminal {} focus acquired on attempt {}", tid_focus, i + 1);
                                    break;
                                }
                            }
                        }
                    });
                }
            });
        }
    });
    
    // Handle keyboard input directly
    let handle_keydown = {
        let terminal_writer = terminal_writer.clone();
        let is_initialized = is_initialized.clone();
        let terminal_id_for_input = terminal_id.clone();
        
        move |evt: Event<KeyboardData>| {
            // Log immediately to verify events are being received
            tracing::info!("🎹 Keyboard event received: key={:?}, ready={}", evt.key(), is_initialized());
            
            // Only prevent default to stop browser behavior
            evt.prevent_default();
            
            // Only process input if terminal is ready
            if !is_initialized() {
                tracing::warn!("⏳ Terminal not ready yet, ignoring input");
                return;
            }
            
            if let Some(writer) = terminal_writer.read().as_ref() {
                if let Some(input) = keyboard_to_bytes(&evt) {
                    let input_str = String::from_utf8_lossy(&input);
                    tracing::debug!("⌨️ Sending keyboard input: {:?}", input_str);
                    
                    // Check if this is the Claude Code terminal and user pressed Enter
                    if terminal_id_for_input == "claude-code" && input == b"\r" {
                        // Check if the current command contains "claude"
                        let tid_check = terminal_id_for_input.clone();
                        spawn(async move {
                            if let Some(content) = get_xterm_content(&tid_check).await {
                                let lines: Vec<&str> = content.lines().collect();
                                if let Some(last_line) = lines.last() {
                                    if last_line.contains("claude") && !last_line.contains("claude ") {
                                        tracing::warn!("⚠️ Detected 'claude' command without arguments - this may freeze the terminal");
                                    }
                                }
                            }
                        });
                    }
                    
                    // Send input to terminal immediately
                    if let Ok(mut w) = writer.lock() {
                        match w.write_all(&input) {
                            Ok(_) => {
                                match w.flush() {
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
    
    // Note: Output processor is now started during terminal initialization to ensure proper isolation
    
    rsx! {
        div {
            class: "terminal-xterm",
            style: "width: 100%; height: 100%; background: #1e1e1e; overflow: hidden;",
            tabindex: "0",
            onkeydown: handle_keydown,
            onclick: {
                let div_id_click = terminal_div_id.clone();
                move |_| {
                    // Focus this specific terminal when clicked
                    let script = format!(r#"
                        const terminalDiv = document.getElementById('{}').parentElement;
                        if (terminalDiv) {{
                            terminalDiv.focus();
                            console.log('Terminal {} focused on click');
                        }}
                    "#, div_id_click, div_id_click);
                    spawn(async move {
                        let _ = eval(&script).await;
                    });
                }
            },
            
            div {
                id: "{terminal_div_id}",
                style: "width: 100%; height: 100%;",
            }
        }
    }
}

/// Initialize xterm.js
async fn init_xterm(div_id: &str, terminal_id: &str) {
    let script = format!(r#"
        (function() {{
            // Load xterm.js if needed
            if (!window.Terminal && !document.getElementById('xterm-js')) {{
                const script = document.createElement('script');
                script.id = 'xterm-js';
                script.src = 'https://cdn.jsdelivr.net/npm/xterm@5.3.0/lib/xterm.js';
                document.head.appendChild(script);
                
                const css = document.createElement('link');
                css.rel = 'stylesheet';
                css.href = 'https://cdn.jsdelivr.net/npm/xterm@5.3.0/css/xterm.css';
                document.head.appendChild(css);
                
                // Load FitAddon
                const fitScript = document.createElement('script');
                fitScript.id = 'xterm-fit-addon';
                fitScript.src = 'https://cdn.jsdelivr.net/npm/xterm-addon-fit@0.8.0/lib/xterm-addon-fit.js';
                document.head.appendChild(fitScript);
                
                // Load Unicode addon for better character support
                const unicodeScript = document.createElement('script');
                unicodeScript.id = 'xterm-unicode-addon';
                unicodeScript.src = 'https://cdn.jsdelivr.net/npm/xterm-addon-unicode11@0.6.0/lib/xterm-addon-unicode11.js';
                document.head.appendChild(unicodeScript);
                
                // Wait for all scripts to load
                let loadedCount = 0;
                const checkAllLoaded = function() {{
                    loadedCount++;
                    if (loadedCount >= 2) {{
                        setupTerminal();
                    }}
                }};
                
                script.onload = checkAllLoaded;
                fitScript.onload = checkAllLoaded;
                unicodeScript.onload = checkAllLoaded;
            }} else {{
                setupTerminal();
            }}
            
            function setupTerminal() {{
                window.terminals = window.terminals || {{}};
                window.terminalInput = window.terminalInput || {{}};
                
                const container = document.getElementById('{}');
                if (container && !window.terminals['{}']) {{
                    const term = new Terminal({{
                        cursorBlink: true,
                        fontSize: 13,
                        fontFamily: 'Menlo, Monaco, "Courier New", monospace',
                        theme: {{
                            background: '#1e1e1e',
                            foreground: '#cccccc',
                            cursor: '#cccccc'
                        }},
                        scrollback: 50000,
                        convertEol: true,
                        screenReaderMode: false,
                        rendererType: 'canvas',
                        allowTransparency: false,
                        scrollOnUserInput: true,
                        windowsMode: false,
                        macOptionIsMeta: true,
                        rightClickSelectsWord: true,
                        bellStyle: 'none',
                        drawBoldTextInBrightColors: true,
                        fontWeight: 'normal',
                        fontWeightBold: 'bold',
                        minimumContrastRatio: 4.5,
                        tabStopWidth: 8,
                        letterSpacing: 0,
                        lineHeight: 1.0
                    }});
                    
                    term.open(container);
                    window.terminals['{}'] = term;
                    
                    // Add fit addon for proper sizing
                    if (window.FitAddon) {{
                        const fitAddon = new window.FitAddon.FitAddon();
                        term.loadAddon(fitAddon);
                        fitAddon.fit();
                        
                        // Refit on window resize
                        window.addEventListener('resize', () => {{
                            fitAddon.fit();
                        }});
                        
                        // IMMEDIATE: Calculate actual container size and resize PTY (no delay)
                        requestAnimationFrame(() => {{
                            const containerRect = container.getBoundingClientRect();
                            console.log(`📐 Initial container size: ${{containerRect.width}}x${{containerRect.height}}`);
                            
                            // FORCE MAXIMUM SIZE: Override any constraints and use full container
                            // Calculate terminal dimensions manually to ensure full height usage
                            const charWidth = 8;  // Approximate character width
                            const charHeight = 16; // Approximate character height  
                            const maxCols = Math.floor(containerRect.width / charWidth);
                            const maxRows = Math.floor(containerRect.height / charHeight);
                            
                            console.log(`🎯 FORCING maximum terminal size: ${{maxCols}}x${{maxRows}} (container: ${{containerRect.width}}x${{containerRect.height}})`);
                            
                            // Force fit to container first
                            fitAddon.fit();
                            
                            // Then get actual terminal size after fit
                            const actualCols = term.cols;
                            const actualRows = term.rows;
                            console.log(`📊 Terminal after fit: ${{actualCols}}x${{actualRows}}`);
                            
                            // Use the larger of calculated vs actual to maximize space usage
                            const finalCols = Math.max(actualCols, maxCols);
                            const finalRows = Math.max(actualRows, maxRows);
                            console.log(`🚀 FINAL terminal size: ${{finalCols}}x${{finalRows}}`);
                            
                            // Immediately resize PTY to maximum calculated size
                            window.dioxus.postMessage({{
                                method: 'resize_pty',
                                terminal_id: '{}',
                                cols: finalCols,
                                rows: finalRows
                            }});
                            console.log(`📡 Sent MAXIMUM PTY resize: ${{finalCols}}x${{finalRows}}`);
                        }}); // Use requestAnimationFrame for immediate DOM-ready processing
                        
                        // IMMEDIATE resize when container size changes - no delays
                        if (window.ResizeObserver) {{
                            const resizeObserver = new ResizeObserver((entries) => {{
                                // Get the new container dimensions
                                const entry = entries[0];
                                const {{width, height}} = entry.contentRect;
                                
                                console.log(`⚡ IMMEDIATE resize: ${{width}}x${{height}}`);
                                
                                // FORCE MAXIMUM SIZE: Calculate manually to override constraints
                                const charWidth = 8;  // Approximate character width
                                const charHeight = 16; // Approximate character height  
                                const maxCols = Math.floor(width / charWidth);
                                const maxRows = Math.floor(height / charHeight);
                                
                                // Calculate terminal size immediately (no setTimeout!)
                                const oldCols = term.cols;
                                const oldRows = term.rows;
                                
                                // Force immediate fit - no delays
                                fitAddon.fit();
                                
                                const actualCols = term.cols;
                                const actualRows = term.rows;
                                
                                // Use maximum possible size to fill container completely
                                const finalCols = Math.max(actualCols, maxCols);
                                const finalRows = Math.max(actualRows, maxRows);
                                
                                console.log(`⚡ MAXIMUM resize: ${{oldCols}}x${{oldRows}} → ${{finalCols}}x${{finalRows}} (calculated: ${{maxCols}}x${{maxRows}}, actual: ${{actualCols}}x${{actualRows}})`);
                                
                                // Immediately notify Rust - no delays, no batching
                                if (oldCols !== finalCols || oldRows !== finalRows) {{
                                    window.dioxus.postMessage({{
                                        method: 'resize_pty',
                                        terminal_id: '{}',
                                        cols: finalCols,
                                        rows: finalRows
                                    }});
                                    console.log(`⚡ IMMEDIATE MAXIMUM PTY resize: ${{finalCols}}x${{finalRows}}`);
                                }}
                            }});
                            resizeObserver.observe(container);
                        }}
                    }}
                    
                    // Add Unicode addon for better character support
                    if (window.Unicode11Addon) {{
                        const unicodeAddon = new window.Unicode11Addon.Unicode11Addon();
                        term.loadAddon(unicodeAddon);
                        term.unicode.activeVersion = '11';
                    }}
                    
                    // Input handling - xterm.js will handle display, but we process input in Rust
                    term.onData((data) => {{
                        console.log('🔤 xterm.js onData (for paste/special events):', data);
                        // We still handle onData for paste events and special input
                        // But normal keyboard input goes through Rust onkeydown handler
                    }});
                    
                    // Auto-scroll to bottom on new output
                    term.onLineFeed(() => {{
                        term.scrollToBottom();
                    }});
                    
                    term.focus();
                }}
            }}
        }})();
    "#, div_id, terminal_id, terminal_id, terminal_id, terminal_id);
    
    let _ = eval(&script).await;
}

/// Poll for terminal input
/// Convert keyboard event to bytes for terminal
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
        
        // Function keys
        Key::F1 => Some(b"\x1bOP".to_vec()),
        Key::F2 => Some(b"\x1bOQ".to_vec()),
        Key::F3 => Some(b"\x1bOR".to_vec()),
        Key::F4 => Some(b"\x1bOS".to_vec()),
        
        // Regular characters
        Key::Character(c) => {
            if ctrl {
                // Handle Ctrl+key combinations
                if c.len() == 1 {
                    let ch = c.chars().next().unwrap();
                    if ch >= 'a' && ch <= 'z' {
                        Some(vec![(ch as u8) - b'a' + 1])
                    } else if ch >= 'A' && ch <= 'Z' {
                        Some(vec![(ch as u8) - b'A' + 1])
                    } else if ch == '@' {
                        Some(vec![0]) // Ctrl+@
                    } else if ch == '[' {
                        Some(vec![0x1b]) // Ctrl+[ (Escape)
                    } else if ch == '\\' {
                        Some(vec![0x1c]) // Ctrl+\\
                    } else if ch == ']' {
                        Some(vec![0x1d]) // Ctrl+]
                    } else if ch == '^' {
                        Some(vec![0x1e]) // Ctrl+^
                    } else if ch == '_' {
                        Some(vec![0x1f]) // Ctrl+_
                    } else {
                        Some(c.as_bytes().to_vec())
                    }
                } else {
                    Some(c.as_bytes().to_vec())
                }
            } else if alt {
                // Alt+key sends ESC followed by the key
                let mut bytes = vec![0x1b];
                bytes.extend_from_slice(c.as_bytes());
                Some(bytes)
            } else {
                Some(c.as_bytes().to_vec())
            }
        }
        
        _ => None,
    }
}

/// Write output to xterm (called from blocking thread)
fn write_to_xterm(terminal_id: &str, text: &str) {
    tracing::info!("📤 PTY output for {}: {} bytes", terminal_id, text.len());
    tracing::debug!("PTY text content: {:?}", text);
    
    // Add to terminal buffer for Send to Consensus functionality
    add_to_terminal_buffer(terminal_id, text);
    
    // Convert bytes to base64 to avoid any encoding issues
    use base64::Engine;
    let base64_text = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
    
    // Queue the output to be processed later for this specific terminal
    if let Ok(mut queues) = OUTPUT_QUEUES.lock() {
        let queue = queues.entry(terminal_id.to_string()).or_insert_with(Vec::new);
        queue.push(base64_text);
        tracing::debug!("📥 Queued output for {}, queue size: {}", terminal_id, queue.len());
    } else {
        tracing::error!("❌ Failed to lock output queues");
    }
}

/// Process output queue for a specific terminal
async fn process_terminal_output_queue(terminal_id: &str) {
    let items: Vec<String> = {
        if let Ok(mut queues) = OUTPUT_QUEUES.lock() {
            if let Some(queue) = queues.get_mut(terminal_id) {
                queue.drain(..).collect()
            } else {
                Vec::new()
            }
        } else {
            return;
        }
    };
    
    if !items.is_empty() {
        tracing::info!("🔄 Processing {} output items for terminal {}", items.len(), terminal_id);
    }
    
    for base64_text in items {
        let script = format!(r#"
            if (window.terminals && window.terminals['{}']) {{
                const term = window.terminals['{}'];
                // Decode base64 back to binary
                const binaryString = atob('{}');
                const bytes = new Uint8Array(binaryString.length);
                for (let i = 0; i < binaryString.length; i++) {{
                    bytes[i] = binaryString.charCodeAt(i);
                }}
                term.write(bytes);
                console.log('📝 Wrote ' + bytes.length + ' bytes to terminal {}');
                // Ensure we scroll to bottom after writing
                requestAnimationFrame(() => {{
                    term.scrollToBottom();
                }});
            }} else {{
                console.warn('❌ Terminal not found: {}');
            }}
        "#, terminal_id, terminal_id, base64_text, terminal_id, terminal_id);
        
        if let Err(e) = eval(&script).await {
            tracing::error!("❌ Failed to write to xterm {}: {}", terminal_id, e);
        }
    }
}

/// Resize terminal PTY by terminal ID
pub fn resize_terminal_pty(terminal_id: &str, cols: u16, rows: u16) -> bool {
    if let Ok(pty_handles) = PTY_HANDLES.lock() {
        if let Some(pty_handle) = pty_handles.get(terminal_id) {
            if let Ok(pty) = pty_handle.lock() {
                let new_size = PtySize {
                    rows,
                    cols,
                    pixel_width: cols * 10,  // Approximate pixel width
                    pixel_height: rows * 20, // Approximate pixel height
                };
                
                match pty.resize(new_size) {
                    Ok(_) => {
                        tracing::info!("✅ Resized PTY for terminal {}: {}x{}", terminal_id, cols, rows);
                        
                        // ⚡ CRITICAL FIX: Force LazyGit to refresh immediately after resize
                        // LazyGit doesn't automatically refresh when PTY is resized, so we need to trigger it
                        if terminal_id.contains("lazygit") {
                            // Get the writer for this terminal to send refresh signals
                            if let Ok(terminal_registry) = crate::desktop::terminal_registry::TERMINAL_REGISTRY.lock() {
                                if let Some(terminal_info) = terminal_registry.get(terminal_id) {
                                    if let Some(writer) = &terminal_info.writer {
                                        if let Ok(mut w) = writer.lock() {
                                            // AGGRESSIVE REFRESH: Send multiple signals to force LazyGit to redraw
                                            
                                            // 1. Send Ctrl+L (clear screen and redraw)
                                            let _ = w.write_all(&[0x0C]); // Ctrl+L (Form Feed)
                                            let _ = w.flush();
                                            
                                            // 2. Send 'r' key (LazyGit refresh command)
                                            let _ = w.write_all(b"r");
                                            let _ = w.flush();
                                            
                                            // 3. Send Escape to cancel any pending operations, then refresh again
                                            let _ = w.write_all(&[0x1B]); // Escape
                                            let _ = w.write_all(b"r");    // Refresh again
                                            let _ = w.flush();
                                            
                                            tracing::info!("🔄 Sent AGGRESSIVE refresh signals to LazyGit after resize (Ctrl+L, r, Esc+r)");
                                        }
                                    }
                                }
                            }
                        }
                        
                        return true;
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to resize PTY for terminal {}: {}", terminal_id, e);
                    }
                }
            }
        } else {
            tracing::warn!("⚠️ PTY handle not found for terminal: {}", terminal_id);
        }
    }
    false
}

/// Create PTY
fn create_pty(working_directory: Option<String>, is_claude_code: bool, command: Option<String>, args: Vec<String>) -> Result<(Arc<Mutex<Box<dyn Write + Send>>>, Box<dyn Read + Send>, Arc<Mutex<Box<dyn MasterPty + Send>>>), Box<dyn std::error::Error>> {
    let pty_system = NativePtySystem::default();
    
    let mut cmd = if let Some(custom_command) = command {
        // Use custom command (e.g., "lazygit")
        tracing::info!("🚀 Launching custom command: {} {:?} in directory: {:?}", custom_command, args, working_directory);
        let mut builder = CommandBuilder::new(custom_command);
        for arg in args {
            builder.arg(arg);
        }
        builder
    } else if is_claude_code {
        // For Claude Code terminal, use bash with a special prompt
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        tracing::info!("🤖 Launching shell for Claude Code terminal with instructions");
        let mut builder = CommandBuilder::new(shell);
        // Add a helpful prompt for Claude Code usage
        builder.env("PS1", "claude> ");
        builder
    } else {
        // For regular terminals, use the shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        tracing::info!("🐚 Launching shell terminal: {}", shell);
        CommandBuilder::new(shell)
    };
    
    if let Some(dir) = working_directory {
        cmd.cwd(dir);
    }
    
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    cmd.env("LANG", "en_US.UTF-8");
    cmd.env("LC_ALL", "en_US.UTF-8");
    // DO NOT set COLUMNS and LINES environment variables!
    // These hardcoded values (80x24) prevent LazyGit from using the full terminal size.
    // Instead, let the PTY size and dynamic resize system handle the dimensions.
    
    // Create PTY with reasonable initial size - will be resized immediately to match container
    let pty_pair = pty_system.openpty(PtySize {
        rows: 40,      // Larger initial size for better LazyGit display - resized immediately after DOM renders
        cols: 120,     // Larger initial size for better LazyGit display - resized immediately after DOM renders  
        pixel_width: 120 * 10,
        pixel_height: 40 * 20,
    })?;
    
    let _child = pty_pair.slave.spawn_command(cmd)?;
    std::mem::drop(pty_pair.slave);
    
    let reader = pty_pair.master.try_clone_reader()?;
    let writer = pty_pair.master.take_writer()?;
    let pty_handle = Arc::new(Mutex::new(pty_pair.master));
    
    Ok((Arc::new(Mutex::new(writer)), reader, pty_handle))
}

/// Get terminal content from xterm.js
pub async fn get_xterm_content(terminal_id: &str) -> Option<String> {
    let script = format!(r#"
        if (window.terminals && window.terminals['{}']) {{
            const term = window.terminals['{}'];
            const buffer = term.buffer.active;
            const lines = [];
            
            // Get all lines from the scrollback buffer too
            const totalLines = buffer.length + buffer.baseY;
            for (let i = 0; i < totalLines; i++) {{
                const line = buffer.getLine(i);
                if (line) {{
                    // Get the text without escape sequences
                    let text = line.translateToString(true);
                    // Clean up any remaining artifacts
                    text = text.replace(/\[\d+[;m]?/g, '');
                    text = text.replace(/\x1b\[[0-9;]*[a-zA-Z]/g, '');
                    lines.push(text);
                }}
            }}
            
            return lines.join('\n').trim();
        }}
        return null;
    "#, terminal_id, terminal_id);
    
    match eval(&script).await {
        Ok(result) => result.as_str().map(|s| s.to_string()),
        Err(_) => None,
    }
}