//! Dialog components for IDE interface
//!
//! Provides modal dialogs for About, Welcome, etc.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Dialog types
#[derive(Debug, Clone, PartialEq)]
pub enum DialogType {
    About,
    Welcome,
    FilePicker,
    SaveAs,
    ThemeSelector,
}

/// Dialog state
pub struct DialogManager {
    /// Currently active dialog
    active_dialog: Option<DialogType>,
    /// Dialog-specific state
    about_dialog: AboutDialog,
    welcome_dialog: WelcomeDialog,
    file_picker: FilePickerDialog,
    theme_selector: ThemeSelectorDialog,
}

/// About dialog
struct AboutDialog {
    version: String,
}

/// Welcome dialog
struct WelcomeDialog {
    content: Vec<String>,
}

/// File picker dialog
struct FilePickerDialog {
    current_path: std::path::PathBuf,
    selected_index: usize,
    entries: Vec<FileEntry>,
}

/// Theme selector dialog
struct ThemeSelectorDialog {
    themes: Vec<String>,
    selected_index: usize,
}

#[derive(Clone)]
struct FileEntry {
    name: String,
    is_directory: bool,
    path: std::path::PathBuf,
}

impl DialogManager {
    /// Create new dialog manager
    pub fn new() -> Self {
        Self {
            active_dialog: None,
            about_dialog: AboutDialog {
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            welcome_dialog: WelcomeDialog {
                content: vec![
                    "Welcome to HiveTechs Consensus IDE!".to_string(),
                    "".to_string(),
                    "Get started:".to_string(),
                    "• Open a folder with Ctrl+K Ctrl+O".to_string(),
                    "• Open a file with Ctrl+O".to_string(),
                    "• Use F1-F4 to switch between panels".to_string(),
                    "• Press Ctrl+Shift+P for command palette".to_string(),
                    "".to_string(),
                    "For more information, check the documentation.".to_string(),
                ],
            },
            file_picker: FilePickerDialog {
                current_path: std::env::current_dir().unwrap_or_default(),
                selected_index: 0,
                entries: Vec::new(),
            },
            theme_selector: ThemeSelectorDialog {
                themes: vec![
                    "Dark (Default)".to_string(),
                    "Light".to_string(),
                    "High Contrast".to_string(),
                    "Solarized Dark".to_string(),
                    "Solarized Light".to_string(),
                ],
                selected_index: 0,
            },
        }
    }
    
    /// Show a dialog
    pub fn show_dialog(&mut self, dialog_type: DialogType) -> Result<()> {
        match dialog_type {
            DialogType::FilePicker => {
                self.file_picker.refresh_entries()?;
            }
            _ => {}
        }
        self.active_dialog = Some(dialog_type);
        Ok(())
    }
    
    /// Close active dialog
    pub fn close_dialog(&mut self) {
        self.active_dialog = None;
    }
    
    /// Render active dialog
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &crate::tui::themes::Theme) {
        if let Some(dialog_type) = &self.active_dialog {
            match dialog_type {
                DialogType::About => self.render_about_dialog(frame, area, theme),
                DialogType::Welcome => self.render_welcome_dialog(frame, area, theme),
                DialogType::FilePicker => self.render_file_picker(frame, area, theme),
                DialogType::SaveAs => self.render_save_as_dialog(frame, area, theme),
                DialogType::ThemeSelector => self.render_theme_selector(frame, area, theme),
            }
        }
    }
    
    /// Render about dialog
    fn render_about_dialog(&self, frame: &mut Frame, area: Rect, theme: &crate::tui::themes::Theme) {
        let dialog_area = centered_rect(50, 40, area);
        frame.render_widget(Clear, dialog_area);
        
        let content = vec![
            Line::from(vec![Span::styled(
                "🐝 HiveTechs Consensus IDE",
                Style::default().fg(theme.primary_color()).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(format!("Version {}", self.about_dialog.version)),
            Line::from(""),
            Line::from("The most advanced AI-powered development assistant"),
            Line::from("with 4-stage consensus pipeline and enterprise features."),
            Line::from(""),
            Line::from("© 2024 HiveTechs"),
            Line::from(""),
            Line::from("Press ESC to close"),
        ];
        
        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" About ")
                    .border_style(Style::default().fg(theme.border_color()))
                    .style(Style::default().bg(theme.background_color()))
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, dialog_area);
    }
    
    /// Render welcome dialog
    fn render_welcome_dialog(&self, frame: &mut Frame, area: Rect, theme: &crate::tui::themes::Theme) {
        let dialog_area = centered_rect(60, 50, area);
        frame.render_widget(Clear, dialog_area);
        
        let content: Vec<Line> = self.welcome_dialog.content
            .iter()
            .map(|line| {
                if line.starts_with("Welcome") {
                    Line::from(vec![Span::styled(
                        line.clone(),
                        Style::default().fg(theme.primary_color()).add_modifier(Modifier::BOLD),
                    )])
                } else if line.starts_with("Get started:") {
                    Line::from(vec![Span::styled(
                        line.clone(),
                        Style::default().fg(theme.secondary_color()).add_modifier(Modifier::BOLD),
                    )])
                } else {
                    Line::from(line.clone())
                }
            })
            .collect();
        
        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Welcome ")
                    .border_style(Style::default().fg(theme.border_color()))
                    .style(Style::default().bg(theme.background_color()))
            )
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, dialog_area);
    }
    
    /// Render file picker dialog
    fn render_file_picker(&self, frame: &mut Frame, area: Rect, theme: &crate::tui::themes::Theme) {
        let dialog_area = centered_rect(70, 60, area);
        frame.render_widget(Clear, dialog_area);
        
        // Split dialog into path and file list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Path
                Constraint::Min(0),     // File list
                Constraint::Length(2),  // Instructions
            ])
            .split(dialog_area);
        
        // Render current path
        let path_text = format!("📁 {}", self.file_picker.current_path.display());
        let path_widget = Paragraph::new(path_text)
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .title(" Open File ")
                    .border_style(Style::default().fg(theme.border_color()))
                    .style(Style::default().bg(theme.background_color()))
            );
        frame.render_widget(path_widget, chunks[0]);
        
        // Render file list
        use ratatui::widgets::{List, ListItem};
        let items: Vec<ListItem> = self.file_picker.entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let icon = if entry.is_directory { "📁" } else { "📄" };
                let style = if idx == self.file_picker.selected_index {
                    Style::default().fg(theme.primary_color()).bg(theme.selection_bg())
                } else {
                    Style::default().fg(theme.text_color())
                };
                ListItem::new(format!("{} {}", icon, entry.name)).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .style(Style::default().bg(theme.background_color()))
            );
        frame.render_widget(list, chunks[1]);
        
        // Render instructions
        let instructions = Paragraph::new("↑↓ Navigate | Enter: Open | Esc: Cancel")
            .block(
                Block::default()
                    .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                    .border_style(Style::default().fg(theme.border_color()))
                    .style(Style::default().bg(theme.background_color()))
            )
            .alignment(Alignment::Center);
        frame.render_widget(instructions, chunks[2]);
    }
    
    /// Render save as dialog
    fn render_save_as_dialog(&self, frame: &mut Frame, area: Rect, theme: &crate::tui::themes::Theme) {
        let dialog_area = centered_rect(60, 30, area);
        frame.render_widget(Clear, dialog_area);
        
        let content = vec![
            Line::from("Enter filename:"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "untitled.txt",
                Style::default().fg(theme.primary_color()),
            )]),
            Line::from(""),
            Line::from("Press Enter to save, Esc to cancel"),
        ];
        
        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Save As ")
                    .border_style(Style::default().fg(theme.border_color()))
                    .style(Style::default().bg(theme.background_color()))
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, dialog_area);
    }
    
    /// Render theme selector dialog
    fn render_theme_selector(&self, frame: &mut Frame, area: Rect, theme: &crate::tui::themes::Theme) {
        let dialog_area = centered_rect(40, 40, area);
        frame.render_widget(Clear, dialog_area);
        
        use ratatui::widgets::{List, ListItem};
        let items: Vec<ListItem> = self.theme_selector.themes
            .iter()
            .enumerate()
            .map(|(idx, theme_name)| {
                let style = if idx == self.theme_selector.selected_index {
                    Style::default().fg(theme.primary_color()).bg(theme.selection_bg())
                } else {
                    Style::default().fg(theme.text_color())
                };
                ListItem::new(theme_name.clone()).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Select Theme ")
                    .border_style(Style::default().fg(theme.border_color()))
                    .style(Style::default().bg(theme.background_color()))
            );
        
        frame.render_widget(list, dialog_area);
    }
    
    /// Handle key events for active dialog
    pub fn handle_key_event(&mut self, key: KeyEvent) -> DialogResult {
        match &self.active_dialog {
            Some(DialogType::About) | Some(DialogType::Welcome) => {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.close_dialog();
                        DialogResult::Closed
                    }
                    _ => DialogResult::Continue,
                }
            }
            Some(DialogType::FilePicker) => self.handle_file_picker_key(key),
            Some(DialogType::ThemeSelector) => self.handle_theme_selector_key(key),
            _ => DialogResult::Continue,
        }
    }
    
    /// Handle file picker key events
    fn handle_file_picker_key(&mut self, key: KeyEvent) -> DialogResult {
        match key.code {
            KeyCode::Esc => {
                self.close_dialog();
                DialogResult::Closed
            }
            KeyCode::Up => {
                if self.file_picker.selected_index > 0 {
                    self.file_picker.selected_index -= 1;
                }
                DialogResult::Continue
            }
            KeyCode::Down => {
                if self.file_picker.selected_index < self.file_picker.entries.len().saturating_sub(1) {
                    self.file_picker.selected_index += 1;
                }
                DialogResult::Continue
            }
            KeyCode::Enter => {
                if let Some(entry) = self.file_picker.entries.get(self.file_picker.selected_index) {
                    if entry.is_directory {
                        self.file_picker.current_path = entry.path.clone();
                        let _ = self.file_picker.refresh_entries();
                        self.file_picker.selected_index = 0;
                        DialogResult::Continue
                    } else {
                        self.close_dialog();
                        DialogResult::FileSelected(entry.path.clone())
                    }
                } else {
                    DialogResult::Continue
                }
            }
            _ => DialogResult::Continue,
        }
    }
    
    /// Handle theme selector key events
    fn handle_theme_selector_key(&mut self, key: KeyEvent) -> DialogResult {
        match key.code {
            KeyCode::Esc => {
                self.close_dialog();
                DialogResult::Closed
            }
            KeyCode::Up => {
                if self.theme_selector.selected_index > 0 {
                    self.theme_selector.selected_index -= 1;
                }
                DialogResult::Continue
            }
            KeyCode::Down => {
                if self.theme_selector.selected_index < self.theme_selector.themes.len() - 1 {
                    self.theme_selector.selected_index += 1;
                }
                DialogResult::Continue
            }
            KeyCode::Enter => {
                let theme_name = self.theme_selector.themes[self.theme_selector.selected_index].clone();
                self.close_dialog();
                DialogResult::ThemeSelected(theme_name)
            }
            _ => DialogResult::Continue,
        }
    }
    
    /// Check if a dialog is active
    pub fn is_active(&self) -> bool {
        self.active_dialog.is_some()
    }
}

/// Result of dialog interaction
#[derive(Debug)]
pub enum DialogResult {
    /// Dialog remains open
    Continue,
    /// Dialog was closed
    Closed,
    /// File was selected
    FileSelected(std::path::PathBuf),
    /// Folder was selected
    FolderSelected(std::path::PathBuf),
    /// Theme was selected
    ThemeSelected(String),
}

impl FilePickerDialog {
    /// Refresh directory entries
    fn refresh_entries(&mut self) -> Result<()> {
        self.entries.clear();
        
        // Add parent directory entry
        if let Some(parent) = self.current_path.parent() {
            self.entries.push(FileEntry {
                name: "..".to_string(),
                is_directory: true,
                path: parent.to_path_buf(),
            });
        }
        
        // Read directory entries
        if let Ok(entries) = std::fs::read_dir(&self.current_path) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();
            
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let entry = FileEntry {
                        name: entry.file_name().to_string_lossy().to_string(),
                        is_directory: metadata.is_dir(),
                        path: entry.path(),
                    };
                    
                    if entry.is_directory {
                        dirs.push(entry);
                    } else {
                        files.push(entry);
                    }
                }
            }
            
            // Sort and combine
            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            files.sort_by(|a, b| a.name.cmp(&b.name));
            
            self.entries.extend(dirs);
            self.entries.extend(files);
        }
        
        Ok(())
    }
}

/// Create a centered rectangle
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