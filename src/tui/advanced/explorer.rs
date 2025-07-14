//! File explorer panel with Git integration
//!
//! Provides VS Code-like file explorer with:
//! - Directory tree navigation
//! - Git status indicators
//! - File type icons
//! - Search and filtering

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
    fs,
    path::{Path, PathBuf},
};

/// File explorer panel state
pub struct ExplorerPanel {
    /// Current directory path
    current_dir: PathBuf,
    /// Directory entries
    entries: Vec<ExplorerEntry>,
    /// List state for navigation
    list_state: ListState,
    /// Show hidden files
    show_hidden: bool,
    /// Filter pattern
    filter: String,
    /// Expanded directories
    expanded_dirs: Vec<PathBuf>,
}

/// Explorer entry with metadata
#[derive(Debug, Clone)]
pub struct ExplorerEntry {
    /// File name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// Entry type
    pub entry_type: EntryType,
    /// Git status
    pub git_status: GitStatus,
    /// Indentation level for tree display
    pub indent_level: usize,
    /// Is directory expanded
    pub is_expanded: bool,
}

/// Type of filesystem entry
#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
    Directory,
    File,
    Symlink,
}

/// Git status for files
#[derive(Debug, Clone, PartialEq)]
pub enum GitStatus {
    Untracked,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    UpdatedButUnmerged,
    Clean,
    Ignored,
}

impl ExplorerPanel {
    /// Create new explorer panel
    pub async fn new() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let mut panel = Self {
            current_dir: current_dir.clone(),
            entries: Vec::new(),
            list_state: ListState::default(),
            show_hidden: false,
            filter: String::new(),
            expanded_dirs: Vec::new(),
        };

        panel.refresh_entries().await?;
        panel.list_state.select(Some(0));

        Ok(panel)
    }

    /// Refresh directory entries
    pub async fn refresh_entries(&mut self) -> Result<()> {
        self.entries.clear();
        let current_dir = self.current_dir.clone();
        self.load_directory_tree(&current_dir, 0).await?;
        Ok(())
    }

    /// Load directory tree recursively
    fn load_directory_tree<'a>(
        &'a mut self,
        dir: &'a Path,
        indent_level: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let mut entries = Vec::new();

            if let Ok(dir_entries) = fs::read_dir(dir) {
                for entry in dir_entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        let name = entry.file_name().to_string_lossy().to_string();

                        // Skip hidden files if not showing them
                        if !self.show_hidden && name.starts_with('.') {
                            continue;
                        }

                        // Apply filter if set
                        if !self.filter.is_empty()
                            && !name.to_lowercase().contains(&self.filter.to_lowercase())
                        {
                            continue;
                        }

                        let entry_type = if path.is_dir() {
                            EntryType::Directory
                        } else if path.is_symlink() {
                            EntryType::Symlink
                        } else {
                            EntryType::File
                        };

                        let git_status = self.get_git_status(&path).await;
                        let is_expanded = self.expanded_dirs.contains(&path);

                        entries.push(ExplorerEntry {
                            name,
                            path: path.clone(),
                            entry_type,
                            git_status,
                            indent_level,
                            is_expanded,
                        });
                    }
                }
            }

            // Sort entries: directories first, then files
            entries.sort_by(|a, b| match (&a.entry_type, &b.entry_type) {
                (EntryType::Directory, EntryType::File) => std::cmp::Ordering::Less,
                (EntryType::File, EntryType::Directory) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            // Add entries to main list
            for entry in entries {
                self.entries.push(entry.clone());

                // If directory is expanded, load its contents
                if entry.entry_type == EntryType::Directory && entry.is_expanded {
                    self.load_directory_tree(&entry.path, indent_level + 1)
                        .await?;
                }
            }

            Ok(())
        })
    }

    /// Get Git status for a file/directory
    async fn get_git_status(&self, _path: &Path) -> GitStatus {
        // TODO: Implement actual Git status checking
        // For now, return clean status
        GitStatus::Clean
    }

    /// Render the explorer panel
    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme, is_active: bool) {
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .map(|entry| create_list_item(entry, theme))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Explorer")
                    .border_style(if is_active {
                        theme.active_border_style()
                    } else {
                        theme.inactive_border_style()
                    }),
            )
            .style(theme.panel_style())
            .highlight_style(
                Style::default()
                    .bg(theme.selection_bg_color())
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Handle key events for explorer panel
    pub async fn handle_key_event(&mut self, key: KeyEvent, _theme: &Theme) -> Result<bool> {
        match key.code {
            KeyCode::Up => {
                if let Some(selected) = self.list_state.selected() {
                    if selected > 0 {
                        self.list_state.select(Some(selected - 1));
                    }
                }
            }
            KeyCode::Down => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.entries.len().saturating_sub(1) {
                        self.list_state.select(Some(selected + 1));
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(entry) = self.entries.get(selected) {
                        self.handle_entry_activation(entry.clone()).await?;
                    }
                }
            }
            KeyCode::Char(' ') => {
                // Toggle directory expansion
                if let Some(selected) = self.list_state.selected() {
                    if let Some(entry) = self.entries.get(selected) {
                        if entry.entry_type == EntryType::Directory {
                            let path = entry.path.clone();
                            self.toggle_directory_expansion(&path).await?;
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                // Go up one directory level
                if let Some(parent) = self.current_dir.parent() {
                    self.current_dir = parent.to_path_buf();
                    self.refresh_entries().await?;
                    self.list_state.select(Some(0));
                }
            }
            KeyCode::Char('h') => {
                // Toggle hidden files
                self.show_hidden = !self.show_hidden;
                self.refresh_entries().await?;
            }
            KeyCode::Char('r') => {
                // Refresh
                self.refresh_entries().await?;
            }
            _ => {}
        }

        Ok(false)
    }

    /// Handle entry activation (enter key)
    async fn handle_entry_activation(&mut self, entry: ExplorerEntry) -> Result<()> {
        match entry.entry_type {
            EntryType::Directory => {
                self.current_dir = entry.path;
                self.refresh_entries().await?;
                self.list_state.select(Some(0));
            }
            EntryType::File => {
                // TODO: Open file in editor panel
                // For now, just print the file path
                println!("Opening file: {:?}", entry.path);
            }
            EntryType::Symlink => {
                // TODO: Handle symlink activation
            }
        }
        Ok(())
    }

    /// Toggle directory expansion in tree view
    async fn toggle_directory_expansion(&mut self, path: &Path) -> Result<()> {
        if let Some(pos) = self.expanded_dirs.iter().position(|p| p == path) {
            self.expanded_dirs.remove(pos);
        } else {
            self.expanded_dirs.push(path.to_path_buf());
        }
        self.refresh_entries().await?;
        Ok(())
    }

    /// Get current directory
    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    /// Get selected entry
    pub fn selected_entry(&self) -> Option<&ExplorerEntry> {
        self.list_state.selected().and_then(|i| self.entries.get(i))
    }

    /// Set filter pattern
    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
    }

    /// Get current filter
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Set root directory
    pub async fn set_root(&mut self, path: PathBuf) -> Result<()> {
        self.current_dir = path;
        self.refresh_entries().await?;
        self.list_state.select(Some(0));
        Ok(())
    }

    /// Clear root directory (close folder)
    pub async fn clear_root(&mut self) -> Result<()> {
        self.current_dir = std::env::current_dir()?;
        self.entries.clear();
        self.expanded_dirs.clear();
        self.refresh_entries().await?;
        Ok(())
    }
}

/// Create list item for explorer entry
fn create_list_item<'a>(entry: &'a ExplorerEntry, theme: &'a Theme) -> ListItem<'a> {
    let indent = "  ".repeat(entry.indent_level);
    let icon = get_entry_icon(entry);
    let git_indicator = get_git_indicator(&entry.git_status);
    let name_style = get_entry_style(entry, theme);

    let content = format!("{}{} {}{}", indent, icon, entry.name, git_indicator);

    ListItem::new(Line::from(vec![Span::styled(content, name_style)]))
}

/// Get icon for entry type
fn get_entry_icon(entry: &ExplorerEntry) -> &'static str {
    match entry.entry_type {
        EntryType::Directory => {
            if entry.is_expanded {
                "📂"
            } else {
                "📁"
            }
        }
        EntryType::File => {
            // Basic file type detection
            if let Some(ext) = entry.path.extension() {
                match ext.to_string_lossy().as_ref() {
                    "rs" => "🦀",
                    "js" | "ts" => "📜",
                    "py" => "🐍",
                    "md" => "📝",
                    "json" | "toml" | "yaml" | "yml" => "⚙️",
                    "png" | "jpg" | "jpeg" | "gif" => "🖼️",
                    _ => "📄",
                }
            } else {
                "📄"
            }
        }
        EntryType::Symlink => "🔗",
    }
}

/// Get Git status indicator
fn get_git_indicator(status: &GitStatus) -> &'static str {
    match status {
        GitStatus::Untracked => " ?",
        GitStatus::Modified => " M",
        GitStatus::Added => " A",
        GitStatus::Deleted => " D",
        GitStatus::Renamed => " R",
        GitStatus::Copied => " C",
        GitStatus::UpdatedButUnmerged => " U",
        GitStatus::Clean => "",
        GitStatus::Ignored => " !",
    }
}

/// Get style for entry based on type and status
fn get_entry_style(entry: &ExplorerEntry, theme: &Theme) -> Style {
    let mut style = theme.text_style();

    match entry.entry_type {
        EntryType::Directory => {
            style = style
                .fg(theme.directory_color())
                .add_modifier(Modifier::BOLD);
        }
        EntryType::File => {
            style = style.fg(theme.file_color());
        }
        EntryType::Symlink => {
            style = style
                .fg(theme.symlink_color())
                .add_modifier(Modifier::ITALIC);
        }
    }

    // Apply Git status styling
    match entry.git_status {
        GitStatus::Modified => style = style.fg(Color::Yellow),
        GitStatus::Added => style = style.fg(Color::Green),
        GitStatus::Deleted => style = style.fg(Color::Red),
        GitStatus::Untracked => style = style.fg(Color::Cyan),
        _ => {}
    }

    style
}
