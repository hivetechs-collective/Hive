//! VT100-based Terminal Emulator for Dioxus
//!
//! A real terminal emulator using portable-pty and vt100 parser
//! that provides full terminal capabilities for interactive CLI tools

use dioxus::prelude::*;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use std::thread;
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
    let mut screen_html = use_signal(|| String::new());
    let mut update_trigger = use_signal(|| 0u32);
    
    // Initialize terminal
    let terminal_id_for_init = terminal_id.clone();
    use_effect(move || {
        if terminal_state.read().is_none() {
            tracing::info!("🚀 Initializing VT100 terminal for {}", terminal_id_for_init.clone());
            match create_terminal(initial_directory.clone()) {
                Ok((terminal, mut output_rx)) => {
                    terminal_state.set(Some(Arc::new(terminal)));
                    
                    // Spawn output handler
                    let mut screen_html = screen_html.clone();
                    let mut update_trigger = update_trigger.clone();
                    spawn(async move {
                        while let Some(()) = output_rx.recv().await {
                            // Trigger re-render
                            update_trigger.set(update_trigger() + 1);
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
        move |evt: Event<KeyboardData>| {
            evt.prevent_default();
            evt.stop_propagation();
            
            if let Some(terminal) = terminal_state.read().as_ref() {
                if let Some(input) = keyboard_to_bytes(&evt) {
                    tracing::debug!("⌨️ Sending keyboard input: {:?}", String::from_utf8_lossy(&input));
                    if let Ok(mut writer) = terminal.writer.lock() {
                        match writer.write_all(&input) {
                            Ok(_) => {
                                let _ = writer.flush();
                                tracing::debug!("✅ Keyboard input sent successfully");
                            }
                            Err(e) => tracing::error!("❌ Failed to send keyboard input: {}", e),
                        }
                    }
                }
            }
        }
    };
    
    // Terminal container style
    let container_style = "
        width: 100%;
        height: 100%;
        background: #000000;
        color: #cccccc;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: 14px;
        line-height: 18px;
        overflow: auto;
        padding: 8px;
        box-sizing: border-box;
        white-space: pre;
        cursor: text;
        position: relative;
    ";
    
    rsx! {
        div {
            class: "terminal-vt100",
            style: "{container_style}",
            tabindex: "0",
            onkeydown: handle_keydown,
            
            // Render terminal screen
            div {
                dangerous_inner_html: "{screen_html}",
                style: "width: 100%; height: 100%;"
            }
        }
    }
}

/// Create a new terminal instance
fn create_terminal(
    working_directory: Option<String>,
) -> Result<(Vt100Terminal, mpsc::UnboundedReceiver<()>), Box<dyn std::error::Error>> {
    // Terminal size - use a wider default for modern screens
    let cols = 120;
    let rows = 30;
    
    // Create PTY system
    let pty_system = NativePtySystem::default();
    
    // Set up command
    let mut cmd = CommandBuilder::new(
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    );
    
    // Set working directory
    if let Some(dir) = working_directory {
        cmd.cwd(dir);
    }
    
    // Set environment variables for proper terminal support
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    
    tracing::debug!("Creating PTY with size {}x{}", cols, rows);
    
    // Create PTY pair
    let pty_pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: cols * 8,
        pixel_height: rows * 16,
    })?;
    
    // Spawn the shell
    tracing::info!("🐚 Spawning shell: {}", std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()));
    let child = pty_pair.slave.spawn_command(cmd)?;
    std::mem::drop(pty_pair.slave); // Close slave side
    
    // Get reader and writer
    let reader = pty_pair.master.try_clone_reader()?;
    let writer = pty_pair.master.take_writer()?;
    
    // Create vt100 parser
    let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 1000)));
    
    // Create update channel
    let (tx, rx) = mpsc::unbounded_channel();
    
    // Spawn reader thread
    let parser_for_reader = Arc::clone(&parser);
    thread::spawn(move || {
        let mut reader = reader;
        let mut buf = vec![0u8; 4096];
        
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    // Process bytes through vt100 parser
                    if let Ok(mut parser) = parser_for_reader.lock() {
                        parser.process(&buf[..n]);
                        tracing::trace!("📝 Processed {} bytes from PTY", n);
                    }
                    // Notify UI to update
                    let _ = tx.send(());
                }
                Err(_) => break,
            }
        }
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

/// Render vt100 screen to HTML
fn render_screen(parser: &vt100::Parser) -> String {
    let screen = parser.screen();
    let mut html = String::new();
    
    html.push_str(r#"<div style="font-family: monospace; white-space: pre; position: relative;">"#);
    
    // Get cursor position
    let (cursor_row, cursor_col) = screen.cursor_position();
    let cursor_row = cursor_row as usize;
    let cursor_col = cursor_col as usize;
    
    // Get screen contents line by line
    let mut lines = Vec::new();
    for row in 0..screen.size().0 {
        let mut line = String::new();
        for col in 0..screen.size().1 {
            if let Some(cell) = screen.cell(row, col) {
                let ch = cell.contents();
                if ch.is_empty() || ch == " " {
                    line.push(' ');
                } else {
                    line.push_str(&ch);
                }
            } else {
                line.push(' ');
            }
        }
        lines.push(line);
    }
    
    // Render lines with cursor
    for (row_idx, line) in lines.iter().enumerate() {
        if row_idx > 0 {
            html.push_str("<br>");
        }
        
        if row_idx == cursor_row {
            // Line with cursor - split at cursor position
            let line_chars: Vec<char> = line.chars().collect();
            
            // Before cursor
            for (col_idx, ch) in line_chars.iter().enumerate() {
                if col_idx == cursor_col {
                    // Add blinking cursor
                    html.push_str(r#"<span style="background-color: #cccccc; color: #000000; animation: blink 1s step-end infinite;">▂</span>"#);
                }
                match ch {
                    '<' => html.push_str("&lt;"),
                    '>' => html.push_str("&gt;"),
                    '&' => html.push_str("&amp;"),
                    _ => html.push(*ch),
                }
            }
            
            // If cursor is at end of line
            if cursor_col >= line_chars.len() {
                html.push_str(r#"<span style="background-color: #cccccc; color: #000000; animation: blink 1s step-end infinite;">▂</span>"#);
            }
        } else {
            // Regular line without cursor
            for ch in line.chars() {
                match ch {
                    '<' => html.push_str("&lt;"),
                    '>' => html.push_str("&gt;"),
                    '&' => html.push_str("&amp;"),
                    _ => html.push(ch),
                }
            }
        }
    }
    
    // Add CSS for cursor blinking
    html.push_str(r#"
    <style>
        @keyframes blink {
            0%, 50% { opacity: 1; }
            51%, 100% { opacity: 0; }
        }
    </style>
    "#);
    
    html.push_str("</div>");
    html
}

/// Convert ANSI escape sequences to HTML
fn ansi_to_html(text: &[u8]) -> String {
    let text_str = String::from_utf8_lossy(text);
    
    // For now, let's do a simpler conversion that preserves the text
    // We'll improve ANSI parsing later
    let mut html = String::new();
    let mut in_escape = false;
    let mut escape_buffer = String::new();
    
    for ch in text_str.chars() {
        if in_escape {
            escape_buffer.push(ch);
            // Simple check for end of escape sequence
            if ch.is_alphabetic() || ch == '~' {
                in_escape = false;
                escape_buffer.clear();
            }
        } else if ch == '\x1b' {
            in_escape = true;
            escape_buffer.clear();
            escape_buffer.push(ch);
        } else {
            // Regular character
            match ch {
                '<' => html.push_str("&lt;"),
                '>' => html.push_str("&gt;"),
                '&' => html.push_str("&amp;"),
                '\n' => html.push_str("<br>"),
                '\r' => {}, // Ignore carriage returns
                _ => html.push(ch),
            }
        }
    }
    
    html
}

#[derive(Default, PartialEq)]
struct AnsiStyle {
    fg_color: Option<String>,
    bg_color: Option<String>,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl AnsiStyle {
    fn to_css(&self) -> String {
        let mut css = String::new();
        
        if let Some(fg) = &self.fg_color {
            css.push_str(&format!("color: {};", fg));
        }
        if let Some(bg) = &self.bg_color {
            css.push_str(&format!("background-color: {};", bg));
        }
        if self.bold {
            css.push_str("font-weight: bold;");
        }
        if self.italic {
            css.push_str("font-style: italic;");
        }
        if self.underline {
            css.push_str("text-decoration: underline;");
        }
        
        css
    }
}

fn process_ansi_sequence(params: &str, command: char, style: &mut AnsiStyle) {
    if command == 'm' {
        // SGR (Select Graphic Rendition)
        if params.is_empty() || params == "0" {
            *style = AnsiStyle::default();
        } else {
            for param in params.split(';') {
                match param {
                    "1" => style.bold = true,
                    "3" => style.italic = true,
                    "4" => style.underline = true,
                    "30" => style.fg_color = Some("#000000".to_string()),
                    "31" => style.fg_color = Some("#cd3131".to_string()),
                    "32" => style.fg_color = Some("#0dbc79".to_string()),
                    "33" => style.fg_color = Some("#e5e510".to_string()),
                    "34" => style.fg_color = Some("#2472c8".to_string()),
                    "35" => style.fg_color = Some("#bc3fbc".to_string()),
                    "36" => style.fg_color = Some("#11a8cd".to_string()),
                    "37" => style.fg_color = Some("#e5e5e5".to_string()),
                    _ => {}
                }
            }
        }
    }
}