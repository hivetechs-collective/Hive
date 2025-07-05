//! Quick file search implementation
//!
//! VS Code-like quick file search with:
//! - Fuzzy file matching
//! - File opening
//! - Recent files
//! - Search in subdirectories

use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode};
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
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::SystemTime,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use tokio::fs;
use crate::tui::themes::Theme;

/// Quick search state
pub struct QuickSearch {
    /// Is quick search open
    pub is_open: bool,
    /// Current input filter
    pub input: String,
    /// Cursor position in input
    pub cursor_pos: usize,
    /// All found files
    all_files: Vec<FileEntry>,
    /// Filtered and sorted files
    filtered_files: Vec<(FileEntry, i64)>, // (file, score)
    /// List state for navigation
    list_state: ListState,
    /// Fuzzy matcher
    matcher: SkimMatcherV2,
    /// Recent files
    recent_files: Vec<PathBuf>,
    /// Search root directory
    search_root: PathBuf,
    /// File to open (result)
    file_to_open: Option<PathBuf>,
    /// Loading state
    is_loading: bool,
    /// Ignored patterns
    ignored_patterns: Vec<String>,
    /// Max search depth
    max_depth: usize,
}

/// File entry for search results
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Full file path
    pub path: PathBuf,
    /// File name (for display)
    pub name: String,
    /// Relative path from search root
    pub relative_path: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified time
    pub modified: Option<SystemTime>,
    /// File type category
    pub file_type: FileTypeCategory,
}

/// File type categories for better display
#[derive(Debug, Clone, PartialEq)]
pub enum FileTypeCategory {
    Source,
    Config,
    Documentation,
    Image,
    Archive,
    Data,
    Other,
}

impl QuickSearch {
    /// Create new quick search
    pub fn new(search_root: PathBuf) -> Self {
        Self {
            is_open: false,
            input: String::new(),
            cursor_pos: 0,
            all_files: Vec::new(),
            filtered_files: Vec::new(),
            list_state: ListState::default(),
            matcher: SkimMatcherV2::default(),
            recent_files: Vec::new(),
            search_root,
            file_to_open: None,
            is_loading: false,
            ignored_patterns: vec![
                ".git".to_string(),
                ".gitignore".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".DS_Store".to_string(),
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".cache".to_string(),
                "__pycache__".to_string(),
                "*.pyc".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
            ],
            max_depth: 10,
        }
    }

    /// Open quick search and start file indexing
    pub async fn open(&mut self) -> Result<()> {
        self.is_open = true;
        self.input.clear();
        self.cursor_pos = 0;
        self.file_to_open = None;
        
        // If files not loaded or search root changed, reload
        if self.all_files.is_empty() {
            self.is_loading = true;
            self.load_files().await?;
            self.is_loading = false;
        }
        
        self.filter_files();
        self.list_state.select(Some(0));
        
        Ok(())
    }

    /// Close quick search
    pub fn close(&mut self) {
        self.is_open = false;
        self.input.clear();
        self.cursor_pos = 0;
        self.file_to_open = None;
    }

    /// Load files from search root
    async fn load_files(&mut self) -> Result<()> {
        self.all_files.clear();
        self.scan_directory(&self.search_root.clone(), 0).await?;
        
        // Sort by name for consistent ordering
        self.all_files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        Ok(())
    }

    /// Recursively scan directory for files
    async fn scan_directory(&mut self, dir: &Path, depth: usize) -> Result<()> {
        if depth > self.max_depth {
            return Ok(());
        }

        let mut entries = fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Skip ignored files/directories
            if self.should_ignore(&name, &path) {
                continue;
            }
            
            if path.is_file() {
                let metadata = entry.metadata().await?;
                let relative_path = self.get_relative_path(&path);
                let file_type = Self::determine_file_type(&path);
                
                let file_entry = FileEntry {
                    path: path.clone(),
                    name,
                    relative_path,
                    size: metadata.len(),
                    modified: metadata.modified().ok(),
                    file_type,
                };
                
                self.all_files.push(file_entry);
            } else if path.is_dir() {
                // Recursively scan subdirectory
                Box::pin(self.scan_directory(&path, depth + 1)).await?;
            }
        }
        
        Ok(())
    }

    /// Check if file/directory should be ignored
    fn should_ignore(&self, name: &str, path: &Path) -> bool {
        // Skip hidden files
        if name.starts_with('.') && name != ".." {
            return true;
        }
        
        // Check against ignore patterns
        for pattern in &self.ignored_patterns {
            if pattern.contains('*') {
                // Simple glob pattern matching
                if self.glob_match(pattern, name) {
                    return true;
                }
            } else if name == pattern || path.file_name().map_or(false, |n| n.to_str() == Some(pattern)) {
                return true;
            }
        }
        
        false
    }

    /// Simple glob pattern matching
    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                text.starts_with(parts[0]) && text.ends_with(parts[1])
            } else {
                false
            }
        } else {
            pattern == text
        }
    }

    /// Get relative path from search root
    fn get_relative_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.search_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    /// Determine file type category
    fn determine_file_type(path: &Path) -> FileTypeCategory {
        if let Some(ext) = path.extension() {
            match ext.to_string_lossy().to_lowercase().as_str() {
                // Source code
                "rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "h" | "hpp" | 
                "go" | "php" | "rb" | "swift" | "kt" | "cs" | "scala" | "clj" => {
                    FileTypeCategory::Source
                }
                
                // Configuration
                "toml" | "json" | "yaml" | "yml" | "xml" | "ini" | "conf" | 
                "config" | "cfg" | "env" => {
                    FileTypeCategory::Config
                }
                
                // Documentation
                "md" | "txt" | "rst" | "adoc" | "tex" | "pdf" | "doc" | "docx" => {
                    FileTypeCategory::Documentation
                }
                
                // Images
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "ico" | "webp" => {
                    FileTypeCategory::Image
                }
                
                // Archives
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => {
                    FileTypeCategory::Archive
                }
                
                // Data
                "csv" | "db" | "sql" | "sqlite" => {
                    FileTypeCategory::Data
                }
                
                _ => FileTypeCategory::Other,
            }
        } else {
            FileTypeCategory::Other
        }
    }

    /// Filter files based on input
    fn filter_files(&mut self) {
        if self.input.is_empty() {
            // Show recent files first, then all files
            let mut files = Vec::new();
            
            // Add recent files first
            for recent_path in &self.recent_files {
                if let Some(file_entry) = self.all_files.iter().find(|f| &f.path == recent_path) {
                    files.push((file_entry.clone(), 1000)); // High score for recent files
                }
            }
            
            // Add remaining files
            for file_entry in &self.all_files {
                if !self.recent_files.contains(&file_entry.path) {
                    files.push((file_entry.clone(), 0));
                }
            }
            
            self.filtered_files = files;
        } else {
            // Fuzzy match files
            let mut matches = Vec::new();
            
            for file_entry in &self.all_files {
                // Try to match against file name and relative path
                let name_score = self.matcher.fuzzy_match(&file_entry.name, &self.input);
                let path_score = self.matcher.fuzzy_match(&file_entry.relative_path, &self.input);
                
                let best_score = name_score.max(path_score);
                
                if let Some(score) = best_score {
                    // Boost score for recent files
                    let final_score = if self.recent_files.contains(&file_entry.path) {
                        score + 100
                    } else {
                        score
                    };
                    
                    matches.push((file_entry.clone(), final_score));
                }
            }
            
            // Sort by score (highest first)
            matches.sort_by(|a, b| b.1.cmp(&a.1));
            
            self.filtered_files = matches;
        }
        
        // Reset selection to first item
        if !self.filtered_files.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Handle key events for quick search
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.close();
                Ok(false)
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some((file_entry, _)) = self.filtered_files.get(selected) {
                        self.file_to_open = Some(file_entry.path.clone());
                        self.add_to_recent(file_entry.path.clone());
                        self.close();
                        return Ok(true); // File selected
                    }
                }
                Ok(false)
            }
            KeyCode::Up => {
                if let Some(selected) = self.list_state.selected() {
                    if selected > 0 {
                        self.list_state.select(Some(selected - 1));
                    }
                }
                Ok(false)
            }
            KeyCode::Down => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.filtered_files.len().saturating_sub(1) {
                        self.list_state.select(Some(selected + 1));
                    }
                }
                Ok(false)
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.filter_files();
                Ok(false)
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.input.remove(self.cursor_pos);
                    self.filter_files();
                }
                Ok(false)
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input.len() {
                    self.input.remove(self.cursor_pos);
                    self.filter_files();
                }
                Ok(false)
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                Ok(false)
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
                Ok(false)
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                Ok(false)
            }
            KeyCode::End => {
                self.cursor_pos = self.input.len();
                Ok(false)
            }
            _ => Ok(false)
        }
    }

    /// Add file to recent files
    fn add_to_recent(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);
        
        // Add to front
        self.recent_files.insert(0, path);
        
        // Keep only last 20 recent files
        if self.recent_files.len() > 20 {
            self.recent_files.truncate(20);
        }
    }

    /// Get selected file and clear
    pub fn take_selected_file(&mut self) -> Option<PathBuf> {
        self.file_to_open.take()
    }

    /// Update search root
    pub fn set_search_root(&mut self, root: PathBuf) {
        if self.search_root != root {
            self.search_root = root;
            self.all_files.clear(); // Force reload on next open
        }
    }

    /// Render quick search
    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if !self.is_open {
            return;
        }

        // Create centered popup area
        let popup_area = Self::centered_rect(90, 70, area);
        
        // Clear the area
        frame.render_widget(Clear, popup_area);

        // Split popup into input and list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input area
                Constraint::Min(0),    // File list
            ])
            .split(popup_area);

        // Render input area
        self.render_input(frame, chunks[0], theme);
        
        // Render file list
        self.render_file_list(frame, chunks[1], theme);
    }

    /// Render input area
    fn render_input(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let input_text = if self.input.is_empty() {
            "Search files...".to_string()
        } else {
            self.input.clone()
        };

        let title = if self.is_loading {
            "Quick Open - Loading..."
        } else {
            "Quick Open"
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
                    .title(title)
                    .border_style(theme.active_border_style())
            );

        frame.render_widget(input_widget, area);
    }

    /// Render file list
    fn render_file_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if self.is_loading {
            let loading_widget = Paragraph::new("Loading files...")
                .style(Style::default().fg(theme.muted_color()))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(loading_widget, area);
            return;
        }

        let recent_files_len = self.recent_files.len();
        let items: Vec<ListItem> = self.filtered_files
            .iter()
            .enumerate()
            .map(|(i, (file_entry, score))| {
                Self::create_file_item_static(file_entry, *score, i < recent_files_len, theme)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Files ({}/{})", 
                        self.filtered_files.len(), 
                        self.all_files.len()
                    ))
            )
            .style(theme.panel_style())
            .highlight_style(
                Style::default()
                    .bg(theme.selection_bg_color())
                    .add_modifier(Modifier::BOLD)
            );

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Create file list item (static version)
    fn create_file_item_static<'a>(file_entry: &'a FileEntry, _score: i64, is_recent: bool, theme: &'a Theme) -> ListItem<'a> {
        let type_icon = match file_entry.file_type {
            FileTypeCategory::Source => "🦀",
            FileTypeCategory::Config => "⚙️",
            FileTypeCategory::Documentation => "📝",
            FileTypeCategory::Image => "🖼️",
            FileTypeCategory::Archive => "📦",
            FileTypeCategory::Data => "📊",
            FileTypeCategory::Other => "📄",
        };

        let type_color = match file_entry.file_type {
            FileTypeCategory::Source => theme.keyword_color(),
            FileTypeCategory::Config => theme.warning_color(),
            FileTypeCategory::Documentation => theme.info_color(),
            FileTypeCategory::Image => theme.success_color(),
            FileTypeCategory::Archive => theme.muted_color(),
            FileTypeCategory::Data => theme.accent_color(),
            FileTypeCategory::Other => theme.foreground_color(),
        };

        let recent_indicator = if is_recent { " ⭐" } else { "" };

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(type_icon, Style::default().fg(type_color)),
                Span::raw(" "),
                Span::styled(
                    &file_entry.name, 
                    theme.text_style().add_modifier(Modifier::BOLD)
                ),
                Span::styled(recent_indicator, Style::default().fg(theme.accent_color())),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    &file_entry.relative_path,
                    Style::default().fg(theme.muted_color())
                ),
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

    /// Refresh file list
    pub async fn refresh(&mut self) -> Result<()> {
        self.load_files().await
    }

    /// Get file count
    pub fn file_count(&self) -> usize {
        self.all_files.len()
    }

    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }
}