//! Command palette implementation
//!
//! VS Code-like command palette with:
//! - Fuzzy command search
//! - Command execution
//! - Command history
//! - Category filtering

use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Span, Line},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph
    },
    Frame,
};
use std::collections::HashMap;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use crate::tui::themes::Theme;

/// Command palette state
pub struct CommandPalette {
    /// Is command palette open
    pub is_open: bool,
    /// Current input filter
    pub input: String,
    /// Cursor position in input
    pub cursor_pos: usize,
    /// Available commands
    commands: Vec<Command>,
    /// Filtered and sorted commands
    filtered_commands: Vec<(Command, i64)>, // (command, score)
    /// List state for navigation
    list_state: ListState,
    /// Fuzzy matcher
    matcher: SkimMatcherV2,
    /// Command categories
    categories: HashMap<String, Vec<Command>>,
    /// Current category filter
    category_filter: Option<String>,
    /// Command history
    history: Vec<String>,
    /// History index for navigation
    history_index: Option<usize>,
}

/// Command definition
#[derive(Debug, Clone)]
pub struct Command {
    /// Command ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Category
    pub category: String,
    /// Keybinding (if any)
    pub keybinding: Option<String>,
    /// Command type
    pub command_type: CommandType,
}

/// Type of command
#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    /// Navigation command
    Navigation,
    /// File operation
    File,
    /// Editor operation
    Editor,
    /// View operation
    View,
    /// Terminal operation
    Terminal,
    /// Hive operation
    Hive,
    /// System operation
    System,
}

/// Command execution result
#[derive(Debug)]
pub enum CommandResult {
    /// Command executed successfully
    Success(String),
    /// Command failed
    Error(String),
    /// Command requires additional input
    RequiresInput(String),
    /// Command was cancelled
    Cancelled,
}

impl CommandPalette {
    /// Create new command palette
    pub fn new() -> Self {
        let commands = Self::create_default_commands();
        let categories = Self::create_categories(&commands);
        
        Self {
            is_open: false,
            input: String::new(),
            cursor_pos: 0,
            commands: commands.clone(),
            filtered_commands: commands.into_iter().map(|cmd| (cmd, 0)).collect(),
            list_state: ListState::default(),
            matcher: SkimMatcherV2::default(),
            categories,
            category_filter: None,
            history: Vec::new(),
            history_index: None,
        }
    }

    /// Create default command set
    fn create_default_commands() -> Vec<Command> {
        vec![
            // Navigation commands
            Command {
                id: "goto.explorer".to_string(),
                name: "Go to Explorer".to_string(),
                description: "Focus the file explorer panel".to_string(),
                category: "Navigation".to_string(),
                keybinding: Some("F1".to_string()),
                command_type: CommandType::Navigation,
            },
            Command {
                id: "goto.editor".to_string(),
                name: "Go to Editor".to_string(),
                description: "Focus the code editor panel".to_string(),
                category: "Navigation".to_string(),
                keybinding: Some("F2".to_string()),
                command_type: CommandType::Navigation,
            },
            Command {
                id: "goto.terminal".to_string(),
                name: "Go to Terminal".to_string(),
                description: "Focus the integrated terminal".to_string(),
                category: "Navigation".to_string(),
                keybinding: Some("F3".to_string()),
                command_type: CommandType::Navigation,
            },
            Command {
                id: "goto.consensus".to_string(),
                name: "Go to Consensus".to_string(),
                description: "Focus the consensus panel".to_string(),
                category: "Navigation".to_string(),
                keybinding: Some("F4".to_string()),
                command_type: CommandType::Navigation,
            },
            
            // File operations
            Command {
                id: "file.open".to_string(),
                name: "Open File".to_string(),
                description: "Open a file in the editor".to_string(),
                category: "File".to_string(),
                keybinding: Some("Ctrl+O".to_string()),
                command_type: CommandType::File,
            },
            Command {
                id: "file.save".to_string(),
                name: "Save File".to_string(),
                description: "Save the current file".to_string(),
                category: "File".to_string(),
                keybinding: Some("Ctrl+S".to_string()),
                command_type: CommandType::File,
            },
            Command {
                id: "file.saveall".to_string(),
                name: "Save All Files".to_string(),
                description: "Save all open files".to_string(),
                category: "File".to_string(),
                keybinding: Some("Ctrl+Shift+S".to_string()),
                command_type: CommandType::File,
            },
            Command {
                id: "file.close".to_string(),
                name: "Close File".to_string(),
                description: "Close the current file".to_string(),
                category: "File".to_string(),
                keybinding: Some("Ctrl+W".to_string()),
                command_type: CommandType::File,
            },
            
            // Editor operations
            Command {
                id: "editor.find".to_string(),
                name: "Find in File".to_string(),
                description: "Search for text in the current file".to_string(),
                category: "Editor".to_string(),
                keybinding: Some("Ctrl+F".to_string()),
                command_type: CommandType::Editor,
            },
            Command {
                id: "editor.replace".to_string(),
                name: "Find and Replace".to_string(),
                description: "Find and replace text in the current file".to_string(),
                category: "Editor".to_string(),
                keybinding: Some("Ctrl+H".to_string()),
                command_type: CommandType::Editor,
            },
            Command {
                id: "editor.goto_line".to_string(),
                name: "Go to Line".to_string(),
                description: "Jump to a specific line number".to_string(),
                category: "Editor".to_string(),
                keybinding: Some("Ctrl+G".to_string()),
                command_type: CommandType::Editor,
            },
            
            // View operations
            Command {
                id: "view.toggle_terminal".to_string(),
                name: "Toggle Terminal".to_string(),
                description: "Show or hide the integrated terminal".to_string(),
                category: "View".to_string(),
                keybinding: Some("Ctrl+`".to_string()),
                command_type: CommandType::View,
            },
            Command {
                id: "view.toggle_consensus".to_string(),
                name: "Toggle Consensus Panel".to_string(),
                description: "Show or hide the consensus panel".to_string(),
                category: "View".to_string(),
                keybinding: None,
                command_type: CommandType::View,
            },
            Command {
                id: "view.theme_dark".to_string(),
                name: "Switch to Dark Theme".to_string(),
                description: "Change the interface to dark theme".to_string(),
                category: "View".to_string(),
                keybinding: None,
                command_type: CommandType::View,
            },
            Command {
                id: "view.theme_light".to_string(),
                name: "Switch to Light Theme".to_string(),
                description: "Change the interface to light theme".to_string(),
                category: "View".to_string(),
                keybinding: None,
                command_type: CommandType::View,
            },
            
            // Terminal operations
            Command {
                id: "terminal.clear".to_string(),
                name: "Clear Terminal".to_string(),
                description: "Clear the terminal history".to_string(),
                category: "Terminal".to_string(),
                keybinding: None,
                command_type: CommandType::Terminal,
            },
            Command {
                id: "terminal.new".to_string(),
                name: "New Terminal".to_string(),
                description: "Open a new terminal instance".to_string(),
                category: "Terminal".to_string(),
                keybinding: None,
                command_type: CommandType::Terminal,
            },
            
            // Hive operations
            Command {
                id: "hive.ask".to_string(),
                name: "Ask Hive".to_string(),
                description: "Start a new consensus query".to_string(),
                category: "Hive".to_string(),
                keybinding: None,
                command_type: CommandType::Hive,
            },
            Command {
                id: "hive.analyze".to_string(),
                name: "Analyze Codebase".to_string(),
                description: "Run Hive analysis on the current codebase".to_string(),
                category: "Hive".to_string(),
                keybinding: None,
                command_type: CommandType::Hive,
            },
            Command {
                id: "hive.plan".to_string(),
                name: "Create Plan".to_string(),
                description: "Generate an execution plan with Hive".to_string(),
                category: "Hive".to_string(),
                keybinding: None,
                command_type: CommandType::Hive,
            },
            Command {
                id: "hive.status".to_string(),
                name: "Hive Status".to_string(),
                description: "Show Hive system status".to_string(),
                category: "Hive".to_string(),
                keybinding: None,
                command_type: CommandType::Hive,
            },
            
            // System operations
            Command {
                id: "system.quit".to_string(),
                name: "Quit".to_string(),
                description: "Exit the application".to_string(),
                category: "System".to_string(),
                keybinding: Some("Ctrl+Q".to_string()),
                command_type: CommandType::System,
            },
            Command {
                id: "system.reload".to_string(),
                name: "Reload Configuration".to_string(),
                description: "Reload the application configuration".to_string(),
                category: "System".to_string(),
                keybinding: None,
                command_type: CommandType::System,
            },
        ]
    }

    /// Create command categories
    fn create_categories(commands: &[Command]) -> HashMap<String, Vec<Command>> {
        let mut categories = HashMap::new();
        
        for command in commands {
            categories
                .entry(command.category.clone())
                .or_insert_with(Vec::new)
                .push(command.clone());
        }
        
        categories
    }

    /// Open command palette
    pub fn open(&mut self) {
        self.is_open = true;
        self.input.clear();
        self.cursor_pos = 0;
        self.filter_commands();
        self.list_state.select(Some(0));
    }

    /// Close command palette
    pub fn close(&mut self) {
        self.is_open = false;
        self.input.clear();
        self.cursor_pos = 0;
        self.category_filter = None;
        self.history_index = None;
    }

    /// Filter commands based on input
    fn filter_commands(&mut self) {
        if self.input.is_empty() {
            // Show all commands, grouped by category
            self.filtered_commands = self.commands
                .iter()
                .filter(|cmd| {
                    if let Some(ref category) = self.category_filter {
                        &cmd.category == category
                    } else {
                        true
                    }
                })
                .map(|cmd| (cmd.clone(), 0))
                .collect();
        } else {
            // Fuzzy match commands
            let mut matches = Vec::new();
            
            for command in &self.commands {
                if let Some(ref category) = self.category_filter {
                    if &command.category != category {
                        continue;
                    }
                }
                
                // Try to match against name and description
                let name_score = self.matcher.fuzzy_match(&command.name, &self.input);
                let desc_score = self.matcher.fuzzy_match(&command.description, &self.input);
                
                let best_score = name_score.max(desc_score);
                
                if let Some(score) = best_score {
                    matches.push((command.clone(), score));
                }
            }
            
            // Sort by score (highest first)
            matches.sort_by(|a, b| b.1.cmp(&a.1));
            
            self.filtered_commands = matches;
        }
        
        // Reset selection to first item
        if !self.filtered_commands.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Handle key events for command palette
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<String>> {
        match key.code {
            KeyCode::Esc => {
                self.close();
                Ok(None)
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some((command, _)) = self.filtered_commands.get(selected) {
                        let command_id = command.id.clone();
                        self.add_to_history(command.name.clone());
                        self.close();
                        return Ok(Some(command_id));
                    }
                }
                Ok(None)
            }
            KeyCode::Up => {
                if let Some(selected) = self.list_state.selected() {
                    if selected > 0 {
                        self.list_state.select(Some(selected - 1));
                    }
                }
                Ok(None)
            }
            KeyCode::Down => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.filtered_commands.len().saturating_sub(1) {
                        self.list_state.select(Some(selected + 1));
                    }
                }
                Ok(None)
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.filter_commands();
                Ok(None)
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.input.remove(self.cursor_pos);
                    self.filter_commands();
                }
                Ok(None)
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input.len() {
                    self.input.remove(self.cursor_pos);
                    self.filter_commands();
                }
                Ok(None)
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                Ok(None)
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
                Ok(None)
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                Ok(None)
            }
            KeyCode::End => {
                self.cursor_pos = self.input.len();
                Ok(None)
            }
            _ => Ok(None)
        }
    }

    /// Add command to history
    fn add_to_history(&mut self, command: String) {
        self.history.insert(0, command);
        if self.history.len() > 50 {
            self.history.truncate(50);
        }
    }

    /// Set category filter
    pub fn set_category_filter(&mut self, category: Option<String>) {
        self.category_filter = category;
        self.filter_commands();
    }

    /// Render command palette
    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if !self.is_open {
            return;
        }

        // Create centered popup area
        let popup_area = Self::centered_rect(80, 60, area);
        
        // Clear the area
        frame.render_widget(Clear, popup_area);

        // Split popup into input and list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input area
                Constraint::Min(0),    // Command list
            ])
            .split(popup_area);

        // Render input area
        self.render_input(frame, chunks[0], theme);
        
        // Render command list
        self.render_command_list(frame, chunks[1], theme);
    }

    /// Render input area
    fn render_input(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let input_text = if self.input.is_empty() {
            "Type a command...".to_string()
        } else {
            self.input.clone()
        };

        let input_widget = Paragraph::new(input_text)
            .style(if self.input.is_empty() {
                Style::default().fg(theme.muted_color())
            } else {
                theme.text_style()
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Palette")
                    .border_style(theme.active_border_style())
            );

        frame.render_widget(input_widget, area);
    }

    /// Render command list
    fn render_command_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Create items first without borrowing self.list_state
        let items: Vec<ListItem> = {
            let filtered = &self.filtered_commands;
            filtered.iter()
                .map(|(command, _score)| {
                    // Inline the create_command_item logic to avoid self borrow
                    let category_style = match command.command_type {
                        CommandType::Navigation => Style::default().fg(theme.accent_color()),
                        CommandType::File => Style::default().fg(theme.success_color()),
                        CommandType::Editor => Style::default().fg(theme.warning_color()),
                        CommandType::View => Style::default().fg(theme.info_color()),
                        CommandType::Terminal => Style::default().fg(theme.muted_color()),
                        CommandType::Hive => Style::default().fg(theme.primary_color()),
                        CommandType::System => Style::default().fg(theme.error_color()),
                    };

                    let keybinding_text = command.keybinding
                        .as_ref()
                        .map(|kb| format!(" ({})", kb))
                        .unwrap_or_default();

                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(
                                format!("[{}]", command.category),
                                category_style.add_modifier(Modifier::BOLD)
                            ),
                            Span::raw(" "),
                            Span::styled(&command.name, theme.text_style().add_modifier(Modifier::BOLD)),
                            Span::styled(keybinding_text, Style::default().fg(theme.muted_color())),
                        ]),
                        Line::from(vec![
                            Span::raw("  "),
                            Span::styled(&command.description, Style::default().fg(theme.muted_color())),
                        ]),
                    ])
                })
                .collect()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Commands ({})", self.filtered_commands.len()))
            )
            .style(theme.panel_style())
            .highlight_style(
                Style::default()
                    .bg(theme.selection_bg_color())
                    .add_modifier(Modifier::BOLD)
            );

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Create command list item
    fn create_command_item<'a>(&self, command: &'a Command, _score: i64, theme: &Theme) -> ListItem<'a> {
        let category_style = match command.command_type {
            CommandType::Navigation => Style::default().fg(theme.accent_color()),
            CommandType::File => Style::default().fg(theme.success_color()),
            CommandType::Editor => Style::default().fg(theme.warning_color()),
            CommandType::View => Style::default().fg(theme.info_color()),
            CommandType::Terminal => Style::default().fg(theme.muted_color()),
            CommandType::Hive => Style::default().fg(theme.primary_color()),
            CommandType::System => Style::default().fg(theme.error_color()),
        };

        let keybinding_text = command.keybinding
            .as_ref()
            .map(|kb| format!(" ({})", kb))
            .unwrap_or_default();

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(
                    format!("[{}]", command.category),
                    category_style.add_modifier(Modifier::BOLD)
                ),
                Span::raw(" "),
                Span::styled(&command.name, theme.text_style().add_modifier(Modifier::BOLD)),
                Span::styled(keybinding_text, Style::default().fg(theme.muted_color())),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(&command.description, Style::default().fg(theme.muted_color())),
            ]),
        ])
    }

    /// Create centered rectangle for popup
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    /// Get command by ID
    pub fn get_command(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|cmd| cmd.id == id)
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}