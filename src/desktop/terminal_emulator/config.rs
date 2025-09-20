//! Terminal emulator configuration

use alacritty_terminal::{
    term::Config as TermConfig,
    tty::{Options as PtyOptions, Shell},
};

/// Terminal emulator configuration
#[derive(Clone, Debug)]
pub struct TerminalConfig {
    /// Terminal columns
    pub cols: u16,
    /// Terminal rows
    pub rows: u16,
    /// Scrollback buffer size
    pub scrollback_lines: u32,
    /// Shell program to run
    pub shell: Option<Shell>,
    /// Working directory
    pub working_directory: Option<String>,
    /// Environment variables
    pub env: Vec<(String, String)>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            scrollback_lines: 10000,
            shell: None, // Use system default
            working_directory: None,
            env: vec![
                ("TERM".to_string(), "xterm-256color".to_string()),
                ("COLORTERM".to_string(), "truecolor".to_string()),
            ],
        }
    }
}

impl TerminalConfig {
    /// Convert to alacritty terminal config
    pub fn to_term_config(&self) -> TermConfig {
        TermConfig {
            scrolling_history: self.scrollback_lines as usize,
            ..Default::default()
        }
    }

    /// Convert to PTY options
    pub fn to_pty_options(&self) -> PtyOptions {
        let mut pty_options = PtyOptions {
            working_directory: self.working_directory.clone().map(Into::into),
            shell: self.shell.clone(),
            drain_on_exit: true,
            env: Default::default(),
        };

        // Add environment variables
        for (key, value) in &self.env {
            pty_options.env.insert(key.clone(), value.clone());
        }

        pty_options
    }
}
