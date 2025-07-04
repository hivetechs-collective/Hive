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
pub mod setup;
pub mod streaming_callbacks;
pub mod consensus_types;
pub mod formatting;

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
use tracing;
use is_terminal::IsTerminal;

/// TUI Framework - Main entry point for professional terminal interface
pub struct TuiFramework {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    app: TuiApp,
    setup_flow: Option<setup::SetupFlow>,
}

impl TuiFramework {
    /// Check if terminal can be used for TUI
    fn can_use_terminal() -> bool {
        // Check if we're in an interactive terminal
        if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
            return false;
        }
        
        // Check for CI/CD environments
        if std::env::var("CI").is_ok() || 
           std::env::var("GITHUB_ACTIONS").is_ok() || 
           std::env::var("JENKINS_URL").is_ok() ||
           std::env::var("BUILDKITE").is_ok() {
            return false;
        }
        
        // Check for dumb terminal
        if std::env::var("TERM").map_or(false, |term| term == "dumb") {
            return false;
        }
        
        // Try to get terminal size as a test
        crossterm::terminal::size().is_ok()
    }
    
    /// Initialize the professional TUI framework with error handling
    pub async fn new() -> Result<Self> {
        // Check if we can actually use the terminal
        if !Self::can_use_terminal() {
            return Err(anyhow::anyhow!("Terminal not available for TUI mode"));
        }
        
        // Setup terminal with proper error handling
        enable_raw_mode().map_err(|e| {
            anyhow::anyhow!("Failed to enable raw mode: {}. Try running in a proper terminal.", e)
        })?;
        
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| {
                let _ = disable_raw_mode(); // Cleanup on failure
                anyhow::anyhow!("Failed to setup terminal: {}. Your terminal may not support TUI mode.", e)
            })?;
            
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Check if setup is needed
        let setup_flow = if setup::SetupFlow::is_setup_needed().await {
            Some(setup::SetupFlow::new())
        } else {
            None
        };

        // Initialize application state
        let app = TuiApp::new().await?;

        Ok(Self {
            terminal,
            app,
            setup_flow,
        })
    }

    /// Run the TUI main loop with professional interface
    pub async fn run(&mut self) -> Result<()> {
        // Run setup flow if needed
        if let Some(ref mut setup) = self.setup_flow {
            loop {
                // Draw the setup interface
                self.terminal.draw(|f| {
                    let area = f.size();
                    setup.render(f, area);
                })?;

                // Handle setup events
                if event::poll(std::time::Duration::from_millis(50))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            // Exit on Ctrl+C
                            if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                                self.cleanup()?;
                                return Ok(());
                            }
                            
                            // Process setup key event
                            if setup.handle_key_event(key).await? {
                                // Validation needed
                                setup.process_validation().await?;
                            }
                        }
                    }
                }

                // Check if setup is complete
                if setup.is_complete() {
                    // Reload consensus engine with new config
                    self.app.reload_consensus_engine().await?;
                    break;
                }
            }
            
            // Clear setup flow
            self.setup_flow = None;
        }

        // Run main TUI loop
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

/// Launch the Claude Code-style professional TUI with safe fallbacks
pub async fn run_professional_tui() -> Result<()> {
    // Check if we can use TUI at all
    if !can_use_terminal() {
        tracing::info!("Terminal not suitable for TUI, falling back to simple CLI");
        return fallback::run_simple_cli().await;
    }
    
    // Detect terminal capabilities safely
    let terminal_size = match crossterm::terminal::size() {
        Ok(size) => size,
        Err(e) => {
            tracing::warn!("Failed to get terminal size: {}, falling back to simple CLI", e);
            return fallback::run_simple_cli().await;
        }
    };
    
    let supports_advanced = advanced::layout::supports_advanced_tui(
        ratatui::layout::Rect::new(0, 0, terminal_size.0, terminal_size.1)
    );
    
    if supports_advanced {
        // Try to launch advanced TUI mode
        match run_advanced_tui().await {
            Ok(()) => Ok(()),
            Err(e) => {
                tracing::warn!("Advanced TUI failed: {}, falling back to simple CLI", e);
                fallback::run_simple_cli().await
            }
        }
    } else {
        // Fall back to simple CLI mode
        fallback::run_simple_cli().await
    }
}

/// Check if terminal can be used for TUI (shared function)
fn can_use_terminal() -> bool {
    // Check if we're in an interactive terminal
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return false;
    }
    
    // Check for CI/CD environments
    if std::env::var("CI").is_ok() || 
       std::env::var("GITHUB_ACTIONS").is_ok() || 
       std::env::var("JENKINS_URL").is_ok() ||
       std::env::var("BUILDKITE").is_ok() {
        return false;
    }
    
    // Check for dumb terminal
    if std::env::var("TERM").map_or(false, |term| term == "dumb") {
        return false;
    }
    
    // Try to get terminal size as a test
    crossterm::terminal::size().is_ok()
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
    /// Check if terminal can be used for TUI
    fn can_use_terminal() -> bool {
        // Check if we're in an interactive terminal
        if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
            return false;
        }
        
        // Check for CI/CD environments
        if std::env::var("CI").is_ok() || 
           std::env::var("GITHUB_ACTIONS").is_ok() || 
           std::env::var("JENKINS_URL").is_ok() ||
           std::env::var("BUILDKITE").is_ok() {
            return false;
        }
        
        // Check for dumb terminal
        if std::env::var("TERM").map_or(false, |term| term == "dumb") {
            return false;
        }
        
        // Try to get terminal size as a test
        crossterm::terminal::size().is_ok()
    }
    
    /// Initialize the advanced TUI framework with error handling
    pub async fn new() -> Result<Self> {
        // Check if we can actually use the terminal
        if !Self::can_use_terminal() {
            return Err(anyhow::anyhow!("Terminal not available for advanced TUI mode"));
        }
        
        // Setup terminal with proper error handling
        crossterm::terminal::enable_raw_mode().map_err(|e| {
            anyhow::anyhow!("Failed to enable raw mode: {}. Try running in a proper terminal.", e)
        })?;
        
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture)
            .map_err(|e| {
                let _ = crossterm::terminal::disable_raw_mode(); // Cleanup on failure
                anyhow::anyhow!("Failed to setup terminal: {}. Your terminal may not support advanced TUI mode.", e)
            })?;
            
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

            // Handle async events
            self.app.handle_async_events().await?;

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