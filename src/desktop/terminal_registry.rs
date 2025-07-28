//! Terminal Registry - Global access to terminal instances
//!
//! Provides a way to access terminal content from anywhere in the application

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::Write;
use once_cell::sync::Lazy;
use chrono::{DateTime, Utc};

/// Terminal context for tracking conversation
#[derive(Clone, Debug)]
pub struct TerminalContext {
    pub last_user_input: String,
    pub last_claude_response: String,
    pub response_timestamp: DateTime<Utc>,
    pub input_buffer: String,  // Current line being typed
}

impl Default for TerminalContext {
    fn default() -> Self {
        Self {
            last_user_input: String::new(),
            last_claude_response: String::new(),
            response_timestamp: Utc::now(),
            input_buffer: String::new(),
        }
    }
}

/// Terminal instance info for registry
pub struct TerminalInfo {
    pub id: String,
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
    pub context: Arc<Mutex<TerminalContext>>,
}

/// Global terminal registry
pub static TERMINAL_REGISTRY: Lazy<Arc<Mutex<HashMap<String, TerminalInfo>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Register a terminal instance
pub fn register_terminal(
    id: String, 
    parser: Arc<Mutex<vt100::Parser>>,
    writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
) {
    if let Ok(mut registry) = TERMINAL_REGISTRY.lock() {
        registry.insert(id.clone(), TerminalInfo { 
            id, 
            parser, 
            writer,
            context: Arc::new(Mutex::new(TerminalContext::default())),
        });
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

/// Send text to a terminal by ID
pub fn send_to_terminal(id: &str, text: &str) -> bool {
    if let Ok(registry) = TERMINAL_REGISTRY.lock() {
        if let Some(info) = registry.get(id) {
            if let Some(writer) = &info.writer {
                if let Ok(mut w) = writer.lock() {
                    // Write the text to the terminal
                    if let Ok(_) = w.write_all(text.as_bytes()) {
                        let _ = w.flush();
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Send text to the active terminal
pub fn send_to_active_terminal(text: &str) -> bool {
    if let Ok(registry) = TERMINAL_REGISTRY.lock() {
        // For now, just send to the first terminal
        if let Some((id, _)) = registry.iter().next() {
            let id = id.clone();
            drop(registry); // Release lock before calling send_to_terminal
            return send_to_terminal(&id, text);
        }
    }
    false
}

/// Update terminal context with new input
pub fn update_terminal_input(id: &str, input: &str) {
    if let Ok(registry) = TERMINAL_REGISTRY.lock() {
        if let Some(info) = registry.get(id) {
            if let Ok(mut context) = info.context.lock() {
                // Build up the input buffer character by character
                if input == "\r" || input == "\n" {
                    // User pressed Enter - process the completed input
                    let cleaned_input = context.input_buffer.trim().to_string();
                    
                    // Check for hive command using dot notation (like hidden files)
                    if cleaned_input == ".hive" || cleaned_input == ".h" {
                        tracing::info!("🔄 Detected .hive command in buffer: '{}'", cleaned_input);
                        // This will be handled by the terminal component
                    } else if !cleaned_input.is_empty() {
                        // Regular input - store as last user input
                        context.last_user_input = cleaned_input.clone();
                        tracing::info!("📝 Stored user input: '{}'", cleaned_input);
                    }
                    
                    // Clear the buffer for next input
                    context.input_buffer.clear();
                } else if input == "\x7f" {
                    // Backspace - remove last character
                    context.input_buffer.pop();
                } else if input.chars().all(|c| !c.is_control()) {
                    // Regular character - add to buffer
                    context.input_buffer.push_str(input);
                }
            }
        }
    }
}

/// Update terminal context with Claude's response
pub fn update_terminal_response(id: &str, response: &str) {
    if let Ok(registry) = TERMINAL_REGISTRY.lock() {
        if let Some(info) = registry.get(id) {
            if let Ok(mut context) = info.context.lock() {
                // Detect Claude's response patterns
                if response.contains("╭") || response.contains("│") || response.contains("╰") {
                    // This looks like Claude's formatted output
                    context.last_claude_response = response.to_string();
                    context.response_timestamp = Utc::now();
                    tracing::info!("📝 Captured Claude response: {} chars", response.len());
                }
            }
        }
    }
}

/// Get the last Claude response with context
pub fn get_last_claude_response_with_context(id: &str) -> Option<(String, String, DateTime<Utc>)> {
    if let Ok(registry) = TERMINAL_REGISTRY.lock() {
        if let Some(info) = registry.get(id) {
            if let Ok(context) = info.context.lock() {
                if !context.last_claude_response.is_empty() {
                    return Some((
                        context.last_user_input.clone(),
                        context.last_claude_response.clone(),
                        context.response_timestamp,
                    ));
                }
            }
        }
    }
    None
}

/// Check if the current input is a .hive command
pub fn is_hive_command(id: &str) -> bool {
    if let Ok(registry) = TERMINAL_REGISTRY.lock() {
        if let Some(info) = registry.get(id) {
            if let Ok(context) = info.context.lock() {
                let input = context.input_buffer.trim();
                return input == ".hive" || input == ".h";
            }
        }
    }
    false
}