//! Terminal backend using alacritty_terminal

use std::sync::Arc;
use tokio::sync::Mutex;
use alacritty_terminal::{
    event::{Event as TermEvent, EventListener, WindowSize},
    event_loop::{EventLoop, Msg, Notifier},
    grid::Dimensions,
    sync::FairMutex,
    term::{Term, TermMode},
    tty::{self, Options as TtyOptions},
    Term as Terminal,
};
use anyhow::{Result, anyhow};

use super::config::TerminalConfig;

/// Terminal size wrapper that implements Dimensions
struct TerminalSize {
    cols: u16,
    lines: u16,
}

impl Dimensions for TerminalSize {
    fn columns(&self) -> usize {
        self.cols as usize
    }

    fn screen_lines(&self) -> usize {
        self.lines as usize
    }

    fn total_lines(&self) -> usize {
        self.screen_lines()
    }
}

/// Terminal backend wrapper
pub struct TerminalBackend {
    terminal: Arc<FairMutex<Terminal<EventProxy>>>,
    pty_writer: Box<dyn std::io::Write + Send>,
}

/// Event proxy for terminal events
#[derive(Clone)]
pub struct EventProxy {
    sender: tokio::sync::mpsc::UnboundedSender<TermEvent>,
}

impl EventListener for EventProxy {
    fn send_event(&self, event: TermEvent) {
        let _ = self.sender.send(event);
    }
}

impl TerminalBackend {
    /// Create a new terminal backend
    pub fn new(config: TerminalConfig) -> Result<(Self, tokio::sync::mpsc::UnboundedReceiver<TermEvent>)> {
        // Create event channel
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let event_proxy = EventProxy { sender: tx };

        // Terminal size
        let terminal_size = TerminalSize {
            cols: config.cols,
            lines: config.rows,
        };
        
        let window_size = WindowSize {
            num_lines: config.rows,
            num_cols: config.cols,
            cell_width: 8,
            cell_height: 16,
        };

        // Create terminal with dimensions
        let term_config = config.to_term_config();
        let terminal = Terminal::new(term_config, &terminal_size, event_proxy.clone());
        let terminal = Arc::new(FairMutex::new(terminal));

        // TTY options
        let tty_options = config.to_pty_options();

        // Create PTY
        let mut pty = tty::new(&tty_options, window_size, 0)
            .map_err(|e| anyhow!("Failed to create PTY: {}", e))?;
            
        // Get writer before moving pty
        let pty_writer = pty.take_writer();

        // Create event loop and spawn it
        let terminal_for_loop = Arc::clone(&terminal);
        std::thread::spawn(move || {
            let event_loop = EventLoop::new(
                terminal_for_loop,
                event_proxy,
                pty,
                false, // hold
                false, // disable_kitty_keyboard_protocol
            );
            if let Err(e) = event_loop.run() {
                tracing::error!("Event loop error: {:?}", e);
            }
        });

        Ok((
            Self {
                terminal,
                pty_writer,
            },
            rx,
        ))
    }

    /// Start the event loop
    pub fn start(self) {
        // Event loop is already started in new()
    }

    /// Get a reference to the terminal
    pub fn terminal(&self) -> Arc<FairMutex<Terminal<EventProxy>>> {
        Arc::clone(&self.terminal)
    }

    /// Write data to the PTY
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        use std::io::Write;
        self.pty_writer.write_all(data)
            .map_err(|e| anyhow!("Failed to write to PTY: {}", e))
    }

    /// Resize the terminal
    pub fn resize(&self, cols: u16, rows: u16) {
        let terminal_size = TerminalSize {
            cols,
            lines: rows,
        };
        
        // Only resize the terminal, PTY resize would need to be handled differently
        let mut terminal = self.terminal.lock();
        terminal.resize(terminal_size);
    }

    /// Get terminal dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        let terminal = self.terminal.lock();
        let cols = terminal.grid().columns();
        let lines = terminal.grid().screen_lines();
        (cols as u16, lines as u16)
    }

    /// Check if terminal is in alternate screen mode (e.g., vim)
    pub fn is_alternate_screen(&self) -> bool {
        let terminal = self.terminal.lock();
        terminal.mode().contains(TermMode::ALT_SCREEN)
    }
}