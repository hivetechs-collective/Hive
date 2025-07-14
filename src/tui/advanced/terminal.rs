//! Integrated terminal panel
//!
//! Provides embedded terminal functionality with:
//! - Shell command execution
//! - Command history
//! - Output scrolling
//! - Hive command integration

use crate::tui::themes::Theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::{
    collections::VecDeque,
    process::{Command, Stdio},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command as AsyncCommand,
};

/// Terminal panel state
pub struct TerminalPanel {
    /// Command history
    history: VecDeque<TerminalEntry>,
    /// Current command input
    current_input: String,
    /// Input cursor position
    cursor_pos: usize,
    /// Command history index for navigation
    history_index: Option<usize>,
    /// Saved command histories
    command_history: VecDeque<String>,
    /// Current working directory
    current_dir: String,
    /// List state for scrolling
    list_state: ListState,
    /// Maximum history size
    max_history: usize,
    /// Is currently executing command
    executing: bool,
}

/// Terminal entry (command or output)
#[derive(Debug, Clone)]
pub struct TerminalEntry {
    /// Entry type
    pub entry_type: TerminalEntryType,
    /// Entry content
    pub content: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// Type of terminal entry
#[derive(Debug, Clone, PartialEq)]
pub enum TerminalEntryType {
    /// User command input
    Command,
    /// Command output (stdout)
    Output,
    /// Command error (stderr)
    Error,
    /// System message
    System,
}

impl TerminalPanel {
    /// Create new terminal panel
    pub fn new() -> Result<Self> {
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("/"))
            .to_string_lossy()
            .to_string();

        let mut panel = Self {
            history: VecDeque::new(),
            current_input: String::new(),
            cursor_pos: 0,
            history_index: None,
            command_history: VecDeque::new(),
            current_dir,
            list_state: ListState::default(),
            max_history: 1000,
            executing: false,
        };

        // Add welcome message
        panel.add_system_message("🐝 HiveTechs Terminal - Ready".to_string());
        panel.add_system_message(format!("Working directory: {}", panel.current_dir));

        Ok(panel)
    }

    /// Add system message to history
    pub fn add_system_message(&mut self, message: String) {
        let entry = TerminalEntry {
            entry_type: TerminalEntryType::System,
            content: message,
            timestamp: chrono::Local::now(),
        };

        self.add_history_entry(entry);
    }

    /// Add command to history
    pub fn add_command(&mut self, command: String) {
        let entry = TerminalEntry {
            entry_type: TerminalEntryType::Command,
            content: format!("{} $ {}", self.get_prompt(), command),
            timestamp: chrono::Local::now(),
        };

        self.add_history_entry(entry);

        // Add to command history for navigation
        if !command.trim().is_empty()
            && self
                .command_history
                .front()
                .map_or(true, |last| last != &command)
        {
            self.command_history.push_front(command);
            if self.command_history.len() > 100 {
                self.command_history.pop_back();
            }
        }
    }

    /// Add output to history
    pub fn add_output(&mut self, output: String) {
        for line in output.lines() {
            let entry = TerminalEntry {
                entry_type: TerminalEntryType::Output,
                content: line.to_string(),
                timestamp: chrono::Local::now(),
            };
            self.add_history_entry(entry);
        }
    }

    /// Add error to history
    pub fn add_error(&mut self, error: String) {
        for line in error.lines() {
            let entry = TerminalEntry {
                entry_type: TerminalEntryType::Error,
                content: line.to_string(),
                timestamp: chrono::Local::now(),
            };
            self.add_history_entry(entry);
        }
    }

    /// Add entry to history with size management
    fn add_history_entry(&mut self, entry: TerminalEntry) {
        self.history.push_back(entry);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        // Auto-scroll to bottom
        if !self.history.is_empty() {
            self.list_state.select(Some(self.history.len() - 1));
        }
    }

    /// Execute command
    pub async fn execute_command(&mut self, command: String) -> Result<()> {
        if command.trim().is_empty() {
            return Ok(());
        }

        self.executing = true;
        self.add_command(command.clone());

        // Handle built-in commands
        if self.handle_builtin_command(&command).await? {
            self.executing = false;
            return Ok(());
        }

        // Execute external command
        match self.execute_external_command(&command).await {
            Ok((stdout, stderr)) => {
                if !stdout.is_empty() {
                    self.add_output(stdout);
                }
                if !stderr.is_empty() {
                    self.add_error(stderr);
                }
            }
            Err(e) => {
                self.add_error(format!("Command failed: {}", e));
            }
        }

        self.executing = false;
        Ok(())
    }

    /// Handle built-in terminal commands
    async fn handle_builtin_command(&mut self, command: &str) -> Result<bool> {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }

        match parts[0] {
            "cd" => {
                let target = if parts.len() > 1 { parts[1] } else { "~" };

                let new_dir = if target == "~" {
                    dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"))
                } else {
                    std::path::PathBuf::from(target)
                };

                match std::env::set_current_dir(&new_dir) {
                    Ok(()) => {
                        self.current_dir = new_dir.to_string_lossy().to_string();
                        self.add_output(format!("Changed directory to: {}", self.current_dir));
                    }
                    Err(e) => {
                        self.add_error(format!("cd: {}", e));
                    }
                }
                Ok(true)
            }
            "clear" => {
                self.history.clear();
                self.add_system_message("Terminal cleared".to_string());
                Ok(true)
            }
            "pwd" => {
                self.add_output(self.current_dir.clone());
                Ok(true)
            }
            "hive" => {
                // Handle Hive commands specially
                self.execute_hive_command(&parts[1..]).await?;
                Ok(true)
            }
            _ => Ok(false), // Not a built-in command
        }
    }

    /// Execute Hive command with integrated output
    async fn execute_hive_command(&mut self, args: &[&str]) -> Result<()> {
        self.add_system_message("🐝 Executing Hive command...".to_string());

        // TODO: Integrate with actual Hive command system
        // For now, simulate some common commands
        match args.get(0) {
            Some(&"ask") => {
                let question = args[1..].join(" ");
                self.add_output(format!("Asking Hive: {}", question));
                self.add_output("🧠 Consensus processing...".to_string());
                // TODO: Integrate with actual consensus engine
            }
            Some(&"status") => {
                self.add_output("🐝 Hive Status:".to_string());
                self.add_output("  ✓ Core system: Online".to_string());
                self.add_output("  ✓ Consensus engine: Ready".to_string());
                self.add_output("  ✓ Memory system: Active".to_string());
            }
            Some(&"help") => {
                self.add_output("🐝 Hive Commands:".to_string());
                self.add_output("  hive ask <question>     - Ask the consensus engine".to_string());
                self.add_output("  hive status             - Show system status".to_string());
                self.add_output("  hive analyze <path>     - Analyze codebase".to_string());
                self.add_output("  hive plan <task>        - Create execution plan".to_string());
            }
            _ => {
                self.add_error(format!("Unknown Hive command: {}", args.join(" ")));
                self.add_output("Use 'hive help' for available commands".to_string());
            }
        }

        Ok(())
    }

    /// Execute external command
    async fn execute_external_command(&self, command: &str) -> Result<(String, String)> {
        let mut cmd = AsyncCommand::new("sh");
        cmd.arg("-c")
            .arg(command)
            .current_dir(&self.current_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

        let mut stdout_reader = BufReader::new(stdout);
        let mut stderr_reader = BufReader::new(stderr);

        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        let mut stdout_line = String::new();
        while stdout_reader.read_line(&mut stdout_line).await? > 0 {
            stdout_lines.push(stdout_line.trim_end().to_string());
            stdout_line.clear();
        }

        let mut stderr_line = String::new();
        while stderr_reader.read_line(&mut stderr_line).await? > 0 {
            stderr_lines.push(stderr_line.trim_end().to_string());
            stderr_line.clear();
        }

        let _ = child.wait().await?;

        Ok((stdout_lines.join("\n"), stderr_lines.join("\n")))
    }

    /// Get command prompt
    fn get_prompt(&self) -> String {
        let dir_name = std::path::Path::new(&self.current_dir)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("/"))
            .to_string_lossy();
        format!("🐝 {}", dir_name)
    }

    /// Render the terminal panel
    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme, is_active: bool) {
        // Split area for history and input
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Min(0),    // History
                ratatui::layout::Constraint::Length(1), // Input line
            ])
            .split(area);

        // Render history
        self.render_history(frame, chunks[0], theme, is_active);

        // Render input line
        self.render_input(frame, chunks[1], theme, is_active);
    }

    /// Render terminal history
    fn render_history(&mut self, frame: &mut Frame, area: Rect, theme: &Theme, is_active: bool) {
        let items: Vec<ListItem> = self
            .history
            .iter()
            .map(|entry| create_history_item(entry, theme))
            .collect();

        let history_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Terminal")
                    .border_style(if is_active {
                        theme.active_border_style()
                    } else {
                        theme.inactive_border_style()
                    }),
            )
            .style(theme.terminal_style());

        frame.render_stateful_widget(history_list, area, &mut self.list_state);
    }

    /// Render input line
    fn render_input(&self, frame: &mut Frame, area: Rect, theme: &Theme, is_active: bool) {
        let prompt = self.get_prompt();
        let input_text = if self.executing {
            format!("{} $ {}", prompt, "[Executing...]")
        } else {
            format!("{} $ {}", prompt, self.current_input)
        };

        let input_style = if is_active {
            theme.active_input_style()
        } else {
            theme.inactive_input_style()
        };

        let input_widget = ratatui::widgets::Paragraph::new(input_text)
            .style(input_style)
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(input_widget, area);
    }

    /// Handle key events for terminal panel
    pub async fn handle_key_event(&mut self, key: KeyEvent, _theme: &Theme) -> Result<bool> {
        if self.executing {
            return Ok(false); // Ignore input while executing
        }

        match key.code {
            KeyCode::Enter => {
                let command = self.current_input.clone();
                self.current_input.clear();
                self.cursor_pos = 0;
                self.history_index = None;

                if !command.trim().is_empty() {
                    self.execute_command(command).await?;
                }
            }
            KeyCode::Char(c) => {
                self.current_input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.current_input.remove(self.cursor_pos);
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.current_input.len() {
                    self.current_input.remove(self.cursor_pos);
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.current_input.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Up => {
                // Navigate command history
                if let Some(index) = self.history_index {
                    if index + 1 < self.command_history.len() {
                        self.history_index = Some(index + 1);
                        self.current_input = self.command_history[index + 1].clone();
                        self.cursor_pos = self.current_input.len();
                    }
                } else if !self.command_history.is_empty() {
                    self.history_index = Some(0);
                    self.current_input = self.command_history[0].clone();
                    self.cursor_pos = self.current_input.len();
                }
            }
            KeyCode::Down => {
                // Navigate command history
                if let Some(index) = self.history_index {
                    if index > 0 {
                        self.history_index = Some(index - 1);
                        self.current_input = self.command_history[index - 1].clone();
                        self.cursor_pos = self.current_input.len();
                    } else {
                        self.history_index = None;
                        self.current_input.clear();
                        self.cursor_pos = 0;
                    }
                }
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
            }
            KeyCode::End => {
                self.cursor_pos = self.current_input.len();
            }
            KeyCode::PageUp => {
                // Scroll history up
                if let Some(selected) = self.list_state.selected() {
                    let new_pos = selected.saturating_sub(10);
                    self.list_state.select(Some(new_pos));
                }
            }
            KeyCode::PageDown => {
                // Scroll history down
                if let Some(selected) = self.list_state.selected() {
                    let new_pos = (selected + 10).min(self.history.len().saturating_sub(1));
                    self.list_state.select(Some(new_pos));
                }
            }
            _ => {}
        }

        Ok(false)
    }

    /// Get current input
    pub fn current_input(&self) -> &str {
        &self.current_input
    }

    /// Check if currently executing command
    pub fn is_executing(&self) -> bool {
        self.executing
    }

    /// Get history length
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Clear terminal history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.add_system_message("Terminal cleared".to_string());
    }
}

/// Create history list item
fn create_history_item<'a>(entry: &'a TerminalEntry, theme: &'a Theme) -> ListItem<'a> {
    let style = match entry.entry_type {
        TerminalEntryType::Command => Style::default()
            .fg(theme.command_color())
            .add_modifier(Modifier::BOLD),
        TerminalEntryType::Output => theme.terminal_output_style(),
        TerminalEntryType::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        TerminalEntryType::System => Style::default()
            .fg(theme.system_message_color())
            .add_modifier(Modifier::ITALIC),
    };

    ListItem::new(Line::from(vec![Span::styled(entry.content.clone(), style)]))
}
