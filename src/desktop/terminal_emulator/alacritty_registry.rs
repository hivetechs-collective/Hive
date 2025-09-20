//! Global registry for alacritty terminal instances

use crate::desktop::terminal_emulator::backend::EventProxy;
use alacritty_terminal::{sync::FairMutex, Term};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Global alacritty terminal registry
pub static ALACRITTY_TERMINALS: Lazy<
    Arc<Mutex<HashMap<String, Arc<FairMutex<Term<EventProxy>>>>>>,
> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Register an alacritty terminal
pub fn register_alacritty_terminal(id: String, terminal: Arc<FairMutex<Term<EventProxy>>>) {
    if let Ok(mut registry) = ALACRITTY_TERMINALS.lock() {
        tracing::info!("📝 Registering alacritty terminal: {}", id);
        registry.insert(id, terminal);
    }
}

/// Unregister an alacritty terminal
pub fn unregister_alacritty_terminal(id: &str) {
    if let Ok(mut registry) = ALACRITTY_TERMINALS.lock() {
        tracing::info!("🗑️ Unregistering alacritty terminal: {}", id);
        registry.remove(id);
    }
}

/// Get content from active alacritty terminal
pub fn get_alacritty_terminal_content() -> Option<String> {
    if let Ok(registry) = ALACRITTY_TERMINALS.lock() {
        // Get the first terminal for now
        if let Some((id, terminal)) = registry.iter().next() {
            tracing::debug!("📋 Extracting content from alacritty terminal: {}", id);
            let term = terminal.lock();

            // Get all visible lines plus scrollback
            let mut content = String::new();
            let grid = term.grid();

            // Get visible content
            for line in grid.display_iter() {
                let mut line_text = String::new();
                for cell in line.into_iter() {
                    if cell.c != '\0' {
                        line_text.push(cell.c);
                    } else {
                        line_text.push(' ');
                    }
                }
                content.push_str(line_text.trim_end());
                content.push('\n');
            }

            return Some(content.trim().to_string());
        }
    }
    None
}
