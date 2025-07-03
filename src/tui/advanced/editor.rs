//! Code editor panel with syntax highlighting
//!
//! Provides VS Code-like editor experience with:
//! - Multi-tab support
//! - Syntax highlighting
//! - Line numbers
//! - Find and replace
//! - Basic code completion

use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style, Modifier},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use crate::tui::themes::Theme;

/// Code editor panel state
pub struct EditorPanel {
    /// Open tabs
    tabs: Vec<EditorTab>,
    /// Current active tab index
    active_tab: usize,
    /// Find and replace state
    find_replace: FindReplaceState,
    /// Editor settings
    settings: EditorSettings,
}

/// Individual editor tab
#[derive(Debug, Clone)]
pub struct EditorTab {
    /// Tab title (filename)
    pub title: String,
    /// File path
    pub path: Option<PathBuf>,
    /// File content lines
    pub lines: Vec<String>,
    /// Current cursor position (line, column)
    pub cursor: (usize, usize),
    /// Scroll position (top line visible)
    pub scroll_offset: usize,
    /// Is file modified
    pub is_modified: bool,
    /// File language for syntax highlighting
    pub language: FileLanguage,
    /// Selection range (start, end) in (line, col) format
    pub selection: Option<((usize, usize), (usize, usize))>,
}

/// Find and replace functionality
#[derive(Debug, Default)]
pub struct FindReplaceState {
    /// Find pattern
    pub find_pattern: String,
    /// Replace pattern
    pub replace_pattern: String,
    /// Current search matches
    pub matches: Vec<(usize, usize)>,
    /// Current match index
    pub current_match: usize,
    /// Find mode active
    pub find_active: bool,
    /// Replace mode active
    pub replace_active: bool,
}

/// Editor settings
#[derive(Debug)]
pub struct EditorSettings {
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Tab width
    pub tab_width: usize,
    /// Use spaces for indentation
    pub use_spaces: bool,
    /// Word wrap
    pub word_wrap: bool,
    /// Show whitespace
    pub show_whitespace: bool,
}

/// File language for syntax highlighting
#[derive(Debug, Clone, PartialEq)]
pub enum FileLanguage {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Markdown,
    JSON,
    TOML,
    YAML,
    HTML,
    CSS,
    Plain,
}

impl EditorPanel {
    /// Create new editor panel
    pub fn new() -> Self {
        Self {
            tabs: vec![EditorTab::new_empty()],
            active_tab: 0,
            find_replace: FindReplaceState::default(),
            settings: EditorSettings::default(),
        }
    }

    /// Open file in new tab
    pub async fn open_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let language = FileLanguage::from_path(path);
        
        let tab = EditorTab {
            title: path.file_name()
                .unwrap_or_else(|| path.as_os_str())
                .to_string_lossy()
                .to_string(),
            path: Some(path.to_path_buf()),
            lines,
            cursor: (0, 0),
            scroll_offset: 0,
            is_modified: false,
            language,
            selection: None,
        };
        
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        
        Ok(())
    }

    /// Save current tab
    pub async fn save_current_tab(&mut self) -> Result<()> {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if let Some(path) = &tab.path {
                let content = tab.lines.join("\n");
                fs::write(path, content)?;
                tab.is_modified = false;
            }
        }
        Ok(())
    }

    /// Close current tab
    pub fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    /// Switch to tab by index
    pub fn switch_to_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
    }

    /// Get current tab
    pub fn current_tab(&self) -> Option<&EditorTab> {
        self.tabs.get(self.active_tab)
    }

    /// Get current tab mutably
    pub fn current_tab_mut(&mut self) -> Option<&mut EditorTab> {
        self.tabs.get_mut(self.active_tab)
    }

    /// Render the editor panel
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        is_active: bool,
    ) {
        // Split area for tabs and editor content
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(1), // Tab bar
                ratatui::layout::Constraint::Min(0),     // Editor content
            ])
            .split(area);

        // Render tab bar
        self.render_tab_bar(frame, chunks[0], theme);

        // Render editor content
        self.render_editor_content(frame, chunks[1], theme, is_active);
    }

    /// Render tab bar
    fn render_tab_bar(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let titles: Vec<Line> = self
            .tabs
            .iter()
            .map(|tab| {
                let mut title = tab.title.clone();
                if tab.is_modified {
                    title.push('*');
                }
                Line::from(vec![Span::raw(title)])
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .style(theme.tab_style())
            .highlight_style(theme.active_tab_style())
            .select(self.active_tab);

        frame.render_widget(tabs, area);
    }

    /// Render editor content
    fn render_editor_content(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        is_active: bool,
    ) {
        if let Some(tab) = self.current_tab() {
            let visible_lines = self.get_visible_lines(tab, area.height as usize);
            let content = self.format_editor_content(tab, &visible_lines, theme);

            let editor = Paragraph::new(content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Editor - {}", tab.title))
                        .border_style(if is_active {
                            theme.active_border_style()
                        } else {
                            theme.inactive_border_style()
                        }),
                )
                .style(theme.editor_style())
                .scroll((tab.scroll_offset as u16, 0));

            frame.render_widget(editor, area);

            // Render cursor if active
            if is_active {
                self.render_cursor(frame, area, tab, theme);
            }
        } else {
            // No tabs open - show welcome message
            let welcome = Paragraph::new("Welcome to HiveTechs Editor\n\nPress Ctrl+O to open a file")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Editor")
                        .border_style(theme.inactive_border_style()),
                )
                .style(theme.editor_style());

            frame.render_widget(welcome, area);
        }
    }

    /// Get visible lines for current viewport
    fn get_visible_lines<'a>(&self, tab: &'a EditorTab, height: usize) -> Vec<(usize, &'a String)> {
        let start = tab.scroll_offset;
        let end = (start + height.saturating_sub(2)).min(tab.lines.len());
        
        tab.lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| (start + i, line))
            .collect()
    }

    /// Format editor content with syntax highlighting and line numbers
    fn format_editor_content(
        &self,
        tab: &EditorTab,
        visible_lines: &[(usize, &String)],
        theme: &Theme,
    ) -> Vec<Line> {
        visible_lines
            .iter()
            .map(|(line_num, line)| {
                let mut spans = Vec::new();
                
                // Add line number if enabled
                if self.settings.show_line_numbers {
                    let line_num_str = format!("{:4} ", line_num + 1);
                    spans.push(Span::styled(
                        line_num_str,
                        Style::default().fg(theme.line_number_color()),
                    ));
                }
                
                // Add line content with syntax highlighting
                let highlighted_spans = self.apply_syntax_highlighting(line, &tab.language, theme);
                spans.extend(highlighted_spans);
                
                Line::from(spans)
            })
            .collect()
    }

    /// Apply syntax highlighting to a line
    fn apply_syntax_highlighting(
        &self,
        line: &str,
        language: &FileLanguage,
        theme: &Theme,
    ) -> Vec<Span> {
        // Basic syntax highlighting implementation
        match language {
            FileLanguage::Rust => self.highlight_rust(line, theme),
            FileLanguage::JavaScript | FileLanguage::TypeScript => self.highlight_javascript(line, theme),
            FileLanguage::Python => self.highlight_python(line, theme),
            FileLanguage::Markdown => self.highlight_markdown(line, theme),
            _ => vec![Span::raw(line.to_string())],
        }
    }

    /// Rust syntax highlighting
    fn highlight_rust(&self, line: &str, theme: &Theme) -> Vec<Span> {
        let keywords = [
            "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait",
            "use", "mod", "crate", "super", "self", "Self", "match", "if",
            "else", "while", "for", "loop", "break", "continue", "return",
        ];
        
        self.basic_keyword_highlighting(line, &keywords, theme)
    }

    /// JavaScript/TypeScript syntax highlighting
    fn highlight_javascript(&self, line: &str, theme: &Theme) -> Vec<Span> {
        let keywords = [
            "function", "const", "let", "var", "class", "interface", "type",
            "import", "export", "from", "as", "if", "else", "while", "for",
            "return", "break", "continue", "try", "catch", "finally",
        ];
        
        self.basic_keyword_highlighting(line, &keywords, theme)
    }

    /// Python syntax highlighting
    fn highlight_python(&self, line: &str, theme: &Theme) -> Vec<Span> {
        let keywords = [
            "def", "class", "import", "from", "as", "if", "elif", "else",
            "while", "for", "in", "return", "break", "continue", "try",
            "except", "finally", "with", "lambda", "yield",
        ];
        
        self.basic_keyword_highlighting(line, &keywords, theme)
    }

    /// Markdown syntax highlighting
    fn highlight_markdown(&self, line: &str, theme: &Theme) -> Vec<Span> {
        if line.starts_with('#') {
            vec![Span::styled(line.to_string(), Style::default().fg(theme.heading_color()).add_modifier(Modifier::BOLD))]
        } else if line.starts_with("```") {
            vec![Span::styled(line.to_string(), Style::default().fg(theme.code_block_color()))]
        } else {
            vec![Span::raw(line.to_string())]
        }
    }

    /// Basic keyword highlighting implementation
    fn basic_keyword_highlighting(&self, line: &str, keywords: &[&str], theme: &Theme) -> Vec<Span> {
        let mut spans = Vec::new();
        let mut current_word = String::new();
        let mut in_string = false;
        let mut string_char = '\0';
        
        for ch in line.chars() {
            match ch {
                '"' | '\'' if !in_string => {
                    if !current_word.is_empty() {
                        spans.push(self.create_word_span(&current_word, keywords, theme));
                        current_word.clear();
                    }
                    in_string = true;
                    string_char = ch;
                    current_word.push(ch);
                }
                c if c == string_char && in_string => {
                    current_word.push(ch);
                    spans.push(Span::styled(current_word.clone(), Style::default().fg(theme.string_color())));
                    current_word.clear();
                    in_string = false;
                    string_char = '\0';
                }
                _ if in_string => {
                    current_word.push(ch);
                }
                c if c.is_whitespace() => {
                    if !current_word.is_empty() {
                        spans.push(self.create_word_span(&current_word, keywords, theme));
                        current_word.clear();
                    }
                    spans.push(Span::raw(c.to_string()));
                }
                c if c.is_alphanumeric() || c == '_' => {
                    current_word.push(ch);
                }
                _ => {
                    if !current_word.is_empty() {
                        spans.push(self.create_word_span(&current_word, keywords, theme));
                        current_word.clear();
                    }
                    spans.push(Span::raw(ch.to_string()));
                }
            }
        }
        
        if !current_word.is_empty() {
            if in_string {
                spans.push(Span::styled(current_word, Style::default().fg(theme.string_color())));
            } else {
                spans.push(self.create_word_span(&current_word, keywords, theme));
            }
        }
        
        spans
    }

    /// Create span for a word (keyword or regular text)
    fn create_word_span(&self, word: &str, keywords: &[&str], theme: &Theme) -> Span {
        if keywords.contains(&word) {
            Span::styled(word.to_string(), Style::default().fg(theme.keyword_color()).add_modifier(Modifier::BOLD))
        } else {
            Span::raw(word.to_string())
        }
    }

    /// Render cursor
    fn render_cursor(&self, _frame: &mut Frame, _area: Rect, _tab: &EditorTab, _theme: &Theme) {
        // TODO: Implement cursor rendering
        // This would require more complex positioning calculations
    }

    /// Handle key events for editor panel
    pub async fn handle_key_event(&mut self, key: KeyEvent, _theme: &Theme) -> Result<bool> {
        match (key.modifiers, key.code) {
            // File operations
            (KeyModifiers::CONTROL, KeyCode::Char('o')) => {
                // TODO: Open file dialog
                Ok(false)
            }
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                self.save_current_tab().await?;
                Ok(false)
            }
            (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
                self.close_current_tab();
                Ok(false)
            }
            // Tab navigation
            (KeyModifiers::CONTROL, KeyCode::Tab) => {
                let next_tab = (self.active_tab + 1) % self.tabs.len();
                self.switch_to_tab(next_tab);
                Ok(false)
            }
            (KeyModifiers::CONTROL | KeyModifiers::SHIFT, KeyCode::Tab) => {
                let prev_tab = if self.active_tab == 0 {
                    self.tabs.len() - 1
                } else {
                    self.active_tab - 1
                };
                self.switch_to_tab(prev_tab);
                Ok(false)
            }
            // Find and replace
            (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
                self.find_replace.find_active = !self.find_replace.find_active;
                Ok(false)
            }
            (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
                self.find_replace.replace_active = !self.find_replace.replace_active;
                Ok(false)
            }
            // Editor navigation
            _ => {
                if let Some(tab) = self.current_tab_mut() {
                    self.handle_editor_navigation(tab, key).await?
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Handle editor navigation and text editing
    async fn handle_editor_navigation(&mut self, tab: &mut EditorTab, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Up => {
                if tab.cursor.0 > 0 {
                    tab.cursor.0 -= 1;
                    tab.cursor.1 = tab.cursor.1.min(tab.lines.get(tab.cursor.0).map_or(0, |l| l.len()));
                }
            }
            KeyCode::Down => {
                if tab.cursor.0 < tab.lines.len().saturating_sub(1) {
                    tab.cursor.0 += 1;
                    tab.cursor.1 = tab.cursor.1.min(tab.lines.get(tab.cursor.0).map_or(0, |l| l.len()));
                }
            }
            KeyCode::Left => {
                if tab.cursor.1 > 0 {
                    tab.cursor.1 -= 1;
                } else if tab.cursor.0 > 0 {
                    tab.cursor.0 -= 1;
                    tab.cursor.1 = tab.lines.get(tab.cursor.0).map_or(0, |l| l.len());
                }
            }
            KeyCode::Right => {
                let line_len = tab.lines.get(tab.cursor.0).map_or(0, |l| l.len());
                if tab.cursor.1 < line_len {
                    tab.cursor.1 += 1;
                } else if tab.cursor.0 < tab.lines.len().saturating_sub(1) {
                    tab.cursor.0 += 1;
                    tab.cursor.1 = 0;
                }
            }
            KeyCode::Home => {
                tab.cursor.1 = 0;
            }
            KeyCode::End => {
                tab.cursor.1 = tab.lines.get(tab.cursor.0).map_or(0, |l| l.len());
            }
            KeyCode::PageUp => {
                tab.cursor.0 = tab.cursor.0.saturating_sub(10);
            }
            KeyCode::PageDown => {
                tab.cursor.0 = (tab.cursor.0 + 10).min(tab.lines.len().saturating_sub(1));
            }
            _ => {}
        }
        
        // Update scroll offset to keep cursor visible
        self.update_scroll_offset(tab);
        
        Ok(false)
    }

    /// Update scroll offset to keep cursor visible
    fn update_scroll_offset(&self, tab: &mut EditorTab) {
        // TODO: Implement proper scroll offset calculation
        // This would need viewport height information
    }
}

impl EditorTab {
    /// Create new empty tab
    pub fn new_empty() -> Self {
        Self {
            title: "Untitled".to_string(),
            path: None,
            lines: vec![String::new()],
            cursor: (0, 0),
            scroll_offset: 0,
            is_modified: false,
            language: FileLanguage::Plain,
            selection: None,
        }
    }
}

impl FileLanguage {
    /// Determine file language from path
    pub fn from_path(path: &Path) -> Self {
        if let Some(ext) = path.extension() {
            match ext.to_string_lossy().as_ref() {
                "rs" => Self::Rust,
                "js" => Self::JavaScript,
                "ts" => Self::TypeScript,
                "py" => Self::Python,
                "md" => Self::Markdown,
                "json" => Self::JSON,
                "toml" => Self::TOML,
                "yaml" | "yml" => Self::YAML,
                "html" => Self::HTML,
                "css" => Self::CSS,
                _ => Self::Plain,
            }
        } else {
            Self::Plain
        }
    }

    /// Get language name as string
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Python => "Python",
            Self::Markdown => "Markdown",
            Self::JSON => "JSON",
            Self::TOML => "TOML",
            Self::YAML => "YAML",
            Self::HTML => "HTML",
            Self::CSS => "CSS",
            Self::Plain => "Plain Text",
        }
    }
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            tab_width: 4,
            use_spaces: true,
            word_wrap: false,
            show_whitespace: false,
        }
    }
}