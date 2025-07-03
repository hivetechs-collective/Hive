//! Interactive trust dialog system for Hive AI
//!
//! This module provides Claude Code-style trust prompts that ask users
//! for permission before accessing files in new directories.

use std::path::Path;
use std::io::{self, Write};
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    cursor::{Hide, Show, MoveTo},
    style::{Color, Print, ResetColor, SetForegroundColor, SetBackgroundColor},
};
use tracing::{debug, info};

use super::security::TrustDecision;

/// Show interactive trust dialog for a directory
pub async fn show_trust_dialog(path: &Path) -> Result<TrustDecision> {
    // Check if we're in a terminal that supports interactive prompts
    if !atty::is(atty::Stream::Stdin) || !atty::is(atty::Stream::Stdout) {
        // Non-interactive mode - default to deny
        return Ok(TrustDecision::TrustDenied);
    }
    
    // Enable raw mode for keyboard input
    enable_raw_mode().context("Failed to enable raw mode")?;
    
    let result = show_dialog_impl(path).await;
    
    // Always disable raw mode
    let _ = disable_raw_mode();
    
    result
}

/// Internal implementation of the trust dialog
async fn show_dialog_impl(path: &Path) -> Result<TrustDecision> {
    let mut stdout = io::stdout();
    
    // Clear screen and hide cursor
    execute!(stdout, Clear(ClearType::All), Hide)?;
    
    // Format path for display (truncate if too long)
    let display_path = format_path_for_display(path);
    
    // Calculate dialog positioning
    let terminal_size = crossterm::terminal::size()?;
    let dialog_width = 58;
    let dialog_height = 20;
    let start_col = (terminal_size.0.saturating_sub(dialog_width)) / 2;
    let start_row = (terminal_size.1.saturating_sub(dialog_height)) / 2;
    
    // Draw the dialog box
    draw_dialog_box(&mut stdout, start_row, start_col, &display_path)?;
    
    // Selection state (0 = Yes, 1 = No)
    let mut selected = 0;
    
    loop {
        // Draw selection indicators
        draw_selection_buttons(&mut stdout, start_row + 16, start_col, selected)?;
        stdout.flush()?;
        
        // Handle keyboard input
        if let Event::Key(key_event) = event::read()? {
            match key_event {
                KeyEvent {
                    code: KeyCode::Left | KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    selected = 0; // Yes
                }
                KeyEvent {
                    code: KeyCode::Right | KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    selected = 1; // No
                }
                KeyEvent {
                    code: KeyCode::Tab,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    selected = 1 - selected; // Toggle
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    // Clear screen and show cursor
                    execute!(stdout, Clear(ClearType::All), Show, MoveTo(0, 0))?;
                    
                    let decision = match selected {
                        0 => {
                            info!("User granted trust for: {}", path.display());
                            TrustDecision::TrustGranted
                        }
                        _ => {
                            info!("User denied trust for: {}", path.display());
                            TrustDecision::TrustDenied
                        }
                    };
                    
                    return Ok(decision);
                }
                KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                    ..
                } | KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    // User cancelled
                    execute!(stdout, Clear(ClearType::All), Show, MoveTo(0, 0))?;
                    debug!("User cancelled trust dialog for: {}", path.display());
                    return Ok(TrustDecision::TrustDenied);
                }
                KeyEvent {
                    code: KeyCode::Char('y') | KeyCode::Char('Y'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    // Quick yes
                    execute!(stdout, Clear(ClearType::All), Show, MoveTo(0, 0))?;
                    info!("User granted trust for: {}", path.display());
                    return Ok(TrustDecision::TrustGranted);
                }
                KeyEvent {
                    code: KeyCode::Char('n') | KeyCode::Char('N'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    // Quick no
                    execute!(stdout, Clear(ClearType::All), Show, MoveTo(0, 0))?;
                    info!("User denied trust for: {}", path.display());
                    return Ok(TrustDecision::TrustDenied);
                }
                _ => {
                    // Ignore other keys
                }
            }
        }
    }
}

/// Draw the main dialog box
fn draw_dialog_box(stdout: &mut io::Stdout, start_row: u16, start_col: u16, display_path: &str) -> Result<()> {
    // Set colors
    execute!(stdout, SetForegroundColor(Color::White), SetBackgroundColor(Color::Black))?;
    
    // Top border
    execute!(stdout, MoveTo(start_col, start_row))?;
    execute!(stdout, Print("┌──────────────────────────────────────────────────────┐"))?;
    
    // Title
    execute!(stdout, MoveTo(start_col, start_row + 1))?;
    execute!(stdout, Print("│"))?;
    execute!(stdout, SetForegroundColor(Color::Yellow))?;
    execute!(stdout, Print("     Do you trust the files in this folder?"))?;
    execute!(stdout, SetForegroundColor(Color::White))?;
    execute!(stdout, Print("          │"))?;
    
    // Empty line
    execute!(stdout, MoveTo(start_col, start_row + 2))?;
    execute!(stdout, Print("│                                                      │"))?;
    
    // Path line
    execute!(stdout, MoveTo(start_col, start_row + 3))?;
    execute!(stdout, Print("│ "))?;
    execute!(stdout, SetForegroundColor(Color::Cyan))?;
    execute!(stdout, Print(&format!("{:<52}", display_path)))?;
    execute!(stdout, SetForegroundColor(Color::White))?;
    execute!(stdout, Print(" │"))?;
    
    // Empty line
    execute!(stdout, MoveTo(start_col, start_row + 4))?;
    execute!(stdout, Print("│                                                      │"))?;
    
    // Warning text
    let warning_lines = [
        "│ HiveTechs Consensus may read files in this folder.  │",
        "│ Reading untrusted files may lead HiveTechs          │",
        "│ Consensus to behave in unexpected ways.             │",
        "│                                                      │",
        "│ With your permission HiveTechs Consensus may        │",
        "│ execute code transformations and analyze files      │",
        "│ in this folder. Processing untrusted code is        │",
        "│ potentially unsafe.                                  │",
        "│                                                      │",
        "│ https://docs.hivetechs.com/security/trust           │",
        "│                                                      │",
    ];
    
    for (i, line) in warning_lines.iter().enumerate() {
        execute!(stdout, MoveTo(start_col, start_row + 5 + i as u16))?;
        execute!(stdout, Print(line))?;
    }
    
    // Instructions
    execute!(stdout, MoveTo(start_col, start_row + 17))?;
    execute!(stdout, Print("│ Use arrow keys to navigate, Enter to select, Esc to │"))?;
    execute!(stdout, MoveTo(start_col, start_row + 18))?;
    execute!(stdout, Print("│ cancel, or press Y/N for quick selection.           │"))?;
    
    // Bottom border
    execute!(stdout, MoveTo(start_col, start_row + 19))?;
    execute!(stdout, Print("└──────────────────────────────────────────────────────┘"))?;
    
    execute!(stdout, ResetColor)?;
    Ok(())
}

/// Draw the selection buttons
fn draw_selection_buttons(stdout: &mut io::Stdout, row: u16, start_col: u16, selected: usize) -> Result<()> {
    execute!(stdout, MoveTo(start_col, row))?;
    execute!(stdout, Print("│ "))?;
    
    // Yes button
    if selected == 0 {
        execute!(stdout, SetForegroundColor(Color::Black), SetBackgroundColor(Color::Green))?;
        execute!(stdout, Print(" Yes, proceed "))?;
    } else {
        execute!(stdout, SetForegroundColor(Color::Green), SetBackgroundColor(Color::Black))?;
        execute!(stdout, Print(" Yes, proceed "))?;
    }
    
    execute!(stdout, ResetColor)?;
    execute!(stdout, Print("  "))?;
    
    // No button
    if selected == 1 {
        execute!(stdout, SetForegroundColor(Color::Black), SetBackgroundColor(Color::Red))?;
        execute!(stdout, Print(" No, exit "))?;
    } else {
        execute!(stdout, SetForegroundColor(Color::Red), SetBackgroundColor(Color::Black))?;
        execute!(stdout, Print(" No, exit "))?;
    }
    
    execute!(stdout, ResetColor)?;
    execute!(stdout, Print("              │"))?;
    
    Ok(())
}

/// Format a path for display, truncating if necessary
fn format_path_for_display(path: &Path) -> String {
    let path_str = path.display().to_string();
    if path_str.len() <= 50 {
        path_str
    } else {
        // Truncate from the middle
        let start = &path_str[..20];
        let end = &path_str[path_str.len() - 27..];
        format!("{}...{}", start, end)
    }
}

/// Show a simple fallback dialog for non-interactive environments
pub fn show_fallback_dialog(path: &Path) -> Result<TrustDecision> {
    println!();
    println!("┌──────────────────────────────────────────────────────┐");
    println!("│     Do you trust the files in this folder?          │");
    println!("│                                                      │");
    println!("│ {:<52} │", format_path_for_display(path));
    println!("│                                                      │");
    println!("│ HiveTechs Consensus may read files in this folder.  │");
    println!("│ Reading untrusted files may lead HiveTechs          │");
    println!("│ Consensus to behave in unexpected ways.             │");
    println!("│                                                      │");
    println!("│ With your permission HiveTechs Consensus may        │");
    println!("│ execute code transformations and analyze files      │");
    println!("│ in this folder. Processing untrusted code is        │");
    println!("│ potentially unsafe.                                  │");
    println!("│                                                      │");
    println!("│ https://docs.hivetechs.com/security/trust           │");
    println!("│                                                      │");
    println!("└──────────────────────────────────────────────────────┘");
    println!();
    
    loop {
        print!("Do you want to proceed? [y/N]: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                info!("User granted trust for: {}", path.display());
                return Ok(TrustDecision::TrustGranted);
            }
            "n" | "no" | "" => {
                info!("User denied trust for: {}", path.display());
                return Ok(TrustDecision::TrustDenied);
            }
            _ => {
                println!("Please enter 'y' for yes or 'n' for no.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_format_path_for_display() {
        let short_path = PathBuf::from("/home/user/project");
        assert_eq!(format_path_for_display(&short_path), "/home/user/project");
        
        let long_path = PathBuf::from("/very/long/path/that/exceeds/the/maximum/allowed/length/for/display/in/the/dialog/box");
        let formatted = format_path_for_display(&long_path);
        assert!(formatted.len() <= 50);
        assert!(formatted.contains("..."));
    }
    
    #[test]
    fn test_fallback_dialog_path_formatting() {
        let path = PathBuf::from("/test/path");
        let formatted = format_path_for_display(&path);
        assert!(!formatted.is_empty());
    }
}