//! Terminal output buffer to store full conversation history
//! 
//! Since VT100 only gives us the visible screen, we need to store
//! the full output separately for features like Send to Consensus

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Maximum lines to store per terminal
const MAX_BUFFER_LINES: usize = 50000;

/// Terminal output buffer
#[derive(Debug, Clone)]
pub struct TerminalBuffer {
    lines: VecDeque<String>,
    max_lines: usize,
}

impl TerminalBuffer {
    pub fn new() -> Self {
        Self {
            lines: VecDeque::new(),
            max_lines: MAX_BUFFER_LINES,
        }
    }
    
    /// Add output to the buffer
    pub fn add_output(&mut self, text: &str) {
        for line in text.lines() {
            self.lines.push_back(line.to_string());
            
            // Remove old lines if we exceed the limit
            while self.lines.len() > self.max_lines {
                self.lines.pop_front();
            }
        }
    }
    
    /// Get all buffered content
    pub fn get_all_content(&self) -> String {
        self.lines.iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Get cleaned content without escape sequences
    pub fn get_cleaned_content(&self) -> String {
        use regex::Regex;
        
        // Regex to match ANSI escape sequences
        let ansi_regex = Regex::new(r"\x1b\[[0-9;]*[mGKHJ]|\x1b\[[0-9;]*[A-Z]|\x1b\].*?\x07|\x1b\[.*?[a-zA-Z]|\x1b\?[0-9]+[hl]|\[[\?0-9;]+[hlm]").unwrap();
        
        self.lines.iter()
            .map(|line| {
                // Remove ANSI escape sequences
                let cleaned = ansi_regex.replace_all(line, "");
                cleaned.trim().to_string()
            })
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Get recent content (last N lines)
    pub fn get_recent_content(&self, num_lines: usize) -> String {
        let start = self.lines.len().saturating_sub(num_lines);
        self.lines.iter()
            .skip(start)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Clear the buffer
    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

/// Global terminal buffers registry
pub static TERMINAL_BUFFERS: Lazy<Arc<Mutex<HashMap<String, TerminalBuffer>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Register a new terminal buffer
pub fn register_terminal_buffer(terminal_id: String) {
    if let Ok(mut buffers) = TERMINAL_BUFFERS.lock() {
        tracing::debug!("📝 Registered terminal buffer for {}", terminal_id);
        buffers.insert(terminal_id, TerminalBuffer::new());
    }
}

/// Add output to a terminal's buffer
pub fn add_to_terminal_buffer(terminal_id: &str, output: &str) {
    if let Ok(mut buffers) = TERMINAL_BUFFERS.lock() {
        if let Some(buffer) = buffers.get_mut(terminal_id) {
            buffer.add_output(output);
        }
    }
}

/// Get full content from a terminal's buffer
pub fn get_terminal_buffer_content(terminal_id: &str) -> Option<String> {
    if let Ok(buffers) = TERMINAL_BUFFERS.lock() {
        buffers.get(terminal_id).map(|b| b.get_all_content())
    } else {
        None
    }
}

/// Get content from the first active terminal buffer
pub fn get_active_terminal_buffer_content() -> Option<String> {
    if let Ok(buffers) = TERMINAL_BUFFERS.lock() {
        // Get the first terminal's buffer
        buffers.values().next().map(|b| b.get_all_content())
    } else {
        None
    }
}

/// Get cleaned content from the first active terminal buffer
pub fn get_active_terminal_buffer_cleaned_content() -> Option<String> {
    if let Ok(buffers) = TERMINAL_BUFFERS.lock() {
        // Get the first terminal's buffer
        buffers.values().next().map(|b| b.get_cleaned_content())
    } else {
        None
    }
}

/// Remove a terminal's buffer
pub fn unregister_terminal_buffer(terminal_id: &str) {
    if let Ok(mut buffers) = TERMINAL_BUFFERS.lock() {
        buffers.remove(terminal_id);
        tracing::debug!("🗑️ Unregistered terminal buffer for {}", terminal_id);
    }
}