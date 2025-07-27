//! Input handling for terminal emulator

use dioxus::prelude::*;
use dioxus_html::input_data::MouseButton;
use alacritty_terminal::term::TermMode;
use std::sync::Arc;

/// Convert Dioxus keyboard event to terminal input
pub fn keyboard_to_bytes(event: &Event<KeyboardData>, alt_screen: bool) -> Option<Vec<u8>> {
    let key = event.key();
    let shift = event.modifiers().shift();
    let ctrl = event.modifiers().ctrl();
    let alt = event.modifiers().alt();
    let meta = event.modifiers().meta();

    // Handle special keys
    match key {
        Key::Enter => Some(b"\r".to_vec()),
        Key::Tab => Some(b"\t".to_vec()),
        Key::Backspace => Some(vec![0x7f]), // DEL
        Key::Escape => Some(b"\x1b".to_vec()),
        
        // Arrow keys
        Key::ArrowUp => Some(if alt_screen {
            b"\x1b[A".to_vec() // Application mode
        } else {
            b"\x1b[A".to_vec() // Normal mode
        }),
        Key::ArrowDown => Some(if alt_screen {
            b"\x1b[B".to_vec()
        } else {
            b"\x1b[B".to_vec()
        }),
        Key::ArrowRight => Some(if alt_screen {
            b"\x1b[C".to_vec()
        } else {
            b"\x1b[C".to_vec()
        }),
        Key::ArrowLeft => Some(if alt_screen {
            b"\x1b[D".to_vec()
        } else {
            b"\x1b[D".to_vec()
        }),
        
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
        Key::F5 => Some(b"\x1b[15~".to_vec()),
        Key::F6 => Some(b"\x1b[17~".to_vec()),
        Key::F7 => Some(b"\x1b[18~".to_vec()),
        Key::F8 => Some(b"\x1b[19~".to_vec()),
        Key::F9 => Some(b"\x1b[20~".to_vec()),
        Key::F10 => Some(b"\x1b[21~".to_vec()),
        Key::F11 => Some(b"\x1b[23~".to_vec()),
        Key::F12 => Some(b"\x1b[24~".to_vec()),
        
        // Character keys
        Key::Character(s) => {
            if s.len() == 1 {
                let ch = s.chars().next().unwrap();
                
                // Handle Ctrl+key combinations
                if ctrl && !alt && !meta {
                    match ch {
                        'a'..='z' => Some(vec![(ch as u8) - b'a' + 1]),
                        'A'..='Z' => Some(vec![(ch.to_lowercase().next().unwrap() as u8) - b'a' + 1]),
                        '[' | '3' => Some(vec![0x1b]), // Ctrl+[ or Ctrl+3 = ESC
                        '\\' | '4' => Some(vec![0x1c]), // Ctrl+\ or Ctrl+4
                        ']' | '5' => Some(vec![0x1d]), // Ctrl+] or Ctrl+5
                        '^' | '6' => Some(vec![0x1e]), // Ctrl+^ or Ctrl+6
                        '_' | '7' => Some(vec![0x1f]), // Ctrl+_ or Ctrl+7
                        '8' => Some(vec![0x7f]), // Ctrl+8 = DEL
                        _ => Some(s.as_bytes().to_vec()),
                    }
                } else if alt {
                    // Alt+key sends ESC followed by the key
                    let mut bytes = vec![0x1b];
                    bytes.extend_from_slice(s.as_bytes());
                    Some(bytes)
                } else {
                    Some(s.as_bytes().to_vec())
                }
            } else {
                // Multi-character string (e.g., from IME)
                Some(s.as_bytes().to_vec())
            }
        }
        
        _ => None,
    }
}

/// Convert mouse event to terminal escape sequences
pub fn mouse_to_bytes(event: &Event<MouseData>, x: u16, y: u16, pressed: bool) -> Option<Vec<u8>> {
    // Convert pixel coordinates to cell coordinates
    let col = (x / 8) + 1; // 1-based
    let row = (y / 16) + 1; // 1-based
    
    // SGR mouse encoding (most compatible)
    let button = match event.trigger_button() {
        Some(MouseButton::Primary) => 0,
        Some(MouseButton::Secondary) => 2,
        Some(MouseButton::Auxiliary) => 1,
        _ => return None,
    };
    
    let action = if pressed { 'M' } else { 'm' };
    
    Some(format!("\x1b[<{};{};{}{}", button, col, row, action).into_bytes())
}

/// Handle paste event
pub fn handle_paste(text: &str) -> Vec<u8> {
    // In bracketed paste mode, wrap with escape sequences
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x1b[200~");
    bytes.extend_from_slice(text.as_bytes());
    bytes.extend_from_slice(b"\x1b[201~");
    bytes
}