//! Terminal Registry - Global access to terminal instances
//!
//! Provides a way to access terminal content from anywhere in the application

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Terminal instance info for registry
pub struct TerminalInfo {
    pub id: String,
    pub parser: Arc<Mutex<vt100::Parser>>,
}

/// Global terminal registry
pub static TERMINAL_REGISTRY: Lazy<Arc<Mutex<HashMap<String, TerminalInfo>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Register a terminal instance
pub fn register_terminal(id: String, parser: Arc<Mutex<vt100::Parser>>) {
    if let Ok(mut registry) = TERMINAL_REGISTRY.lock() {
        registry.insert(id.clone(), TerminalInfo { id, parser });
        tracing::info!("📝 Registered terminal in global registry");
    }
}

/// Unregister a terminal instance
pub fn unregister_terminal(id: &str) {
    if let Ok(mut registry) = TERMINAL_REGISTRY.lock() {
        registry.remove(id);
        tracing::info!("🗑️ Unregistered terminal from global registry");
    }
}

/// Get terminal content by ID
pub fn get_terminal_content(id: &str) -> Option<String> {
    if let Ok(registry) = TERMINAL_REGISTRY.lock() {
        if let Some(info) = registry.get(id) {
            if let Ok(parser) = info.parser.lock() {
                return Some(crate::desktop::terminal_vt100::get_terminal_text(&*parser));
            }
        }
    }
    None
}

/// Get active terminal content (first terminal found)
pub fn get_active_terminal_content() -> Option<String> {
    if let Ok(registry) = TERMINAL_REGISTRY.lock() {
        // For now, just get the first terminal
        // In the future, we should track which terminal is active
        if let Some((_, info)) = registry.iter().next() {
            if let Ok(parser) = info.parser.lock() {
                return Some(crate::desktop::terminal_vt100::get_terminal_text(&*parser));
            }
        }
    }
    None
}

/// Extract Claude's last response from terminal content
pub fn extract_claude_response(content: &str) -> Option<String> {
    // Look for Claude's response patterns
    // Claude responses typically start after the user's input and end at the next prompt
    
    let lines: Vec<&str> = content.lines().collect();
    let mut response = Vec::new();
    let mut in_response = false;
    let mut last_prompt_idx = 0;
    
    // Find the last user prompt (usually contains ">")
    for (i, line) in lines.iter().enumerate().rev() {
        if line.contains('>') && (line.contains('$') || line.contains('%') || line.contains('#')) {
            last_prompt_idx = i;
            break;
        }
    }
    
    // If we found a prompt, everything after it until the next prompt is the response
    if last_prompt_idx > 0 && last_prompt_idx < lines.len() - 1 {
        for i in (last_prompt_idx + 1)..lines.len() {
            let line = lines[i];
            // Stop if we hit another prompt
            if line.contains('>') && (line.contains('$') || line.contains('%') || line.contains('#')) {
                break;
            }
            response.push(line);
        }
    }
    
    // Join the response lines
    let response_text = response.join("\n").trim().to_string();
    
    if response_text.is_empty() {
        None
    } else {
        Some(response_text)
    }
}