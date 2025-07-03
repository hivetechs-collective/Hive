//! Professional TUI Module for HiveTechs Consensus
//! 
//! This module provides a Claude Code-style terminal interface with:
//! - Professional welcome banner with system status
//! - Persistent interactive CLI with input box
//! - Real-time 4-stage consensus visualization with progress bars
//! - Status line with auto-accept toggle and context percentage
//! - Temporal context display with "What's new" section
//! - Advanced TUI mode with VS Code-like interface
//! - Simple CLI fallback for compatibility

pub mod app;
pub mod ui;
pub mod banner;
pub mod input;
pub mod consensus_view;
pub mod status_line;
pub mod widgets;
pub mod advanced;
pub mod themes;
pub mod accessibility;
pub mod fallback;

pub use app::{TuiApp, TuiResult};
pub use ui::TuiInterface;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

/// TUI Framework - Main entry point for professional terminal interface
pub struct TuiFramework {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    app: TuiApp,
}

impl TuiFramework {
    /// Initialize the professional TUI framework
    pub async fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Initialize application state
        let app = TuiApp::new().await?;

        Ok(Self {
            terminal,
            app,
        })
    }

    /// Run the TUI main loop with professional interface
    pub async fn run(&mut self) -> Result<()> {
        loop {
            // Draw the interface
            self.terminal.draw(|f| {
                self.app.render(f);
            })?;

            // Handle events
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        // Process key event
                        if self.app.handle_key_event(key).await? {
                            break; // Exit requested
                        }
                    }
                }
            }

            // Process async events
            self.app.handle_async_events().await?;
        }

        self.cleanup()?;
        Ok(())
    }

    /// Cleanup terminal state
    fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for TuiFramework {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

/// Launch the Claude Code-style professional TUI
pub async fn run_professional_tui() -> Result<()> {
    // Detect terminal capabilities
    let terminal_size = crossterm::terminal::size()?;
    let supports_advanced = advanced::layout::supports_advanced_tui(
        ratatui::layout::Rect::new(0, 0, terminal_size.0, terminal_size.1)
    );
    
    if supports_advanced {
        // Launch advanced TUI mode
        run_advanced_tui().await
    } else {
        // Fall back to simple CLI mode
        fallback::run_simple_cli().await
    }
}

/// Run advanced TUI mode with VS Code-like interface
pub async fn run_advanced_tui() -> Result<()> {
    let mut tui = AdvancedTuiFramework::new().await?;
    tui.run().await
}

/// Advanced TUI Framework - VS Code-like terminal interface
pub struct AdvancedTuiFramework {
    terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: advanced::AdvancedTuiApp,
}

impl AdvancedTuiFramework {
    /// Initialize the advanced TUI framework
    pub async fn new() -> Result<Self> {
        // Setup terminal
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;

        // Initialize advanced application state
        let app = advanced::AdvancedTuiApp::new().await?;

        Ok(Self {
            terminal,
            app,
        })
    }

    /// Run the advanced TUI main loop
    pub async fn run(&mut self) -> Result<()> {
        loop {
            // Draw the interface
            self.terminal.draw(|f| {
                self.app.render(f);
            })?;

            // Handle events
            if crossterm::event::poll(std::time::Duration::from_millis(50))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        // Process key event
                        if self.app.handle_key_event(key).await? {
                            break; // Exit requested
                        }
                    }
                }
            }

            // Check if should quit
            if self.app.should_quit() {
                break;
            }
        }

        self.cleanup()?;
        Ok(())
    }

    /// Cleanup terminal state
    fn cleanup(&mut self) -> Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for AdvancedTuiFramework {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}