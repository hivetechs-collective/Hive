//! Problems Panel for TUI Interface
//!
//! Provides a comprehensive problems view showing:
//! - Git conflicts and merge issues
//! - Build errors and warnings
//! - Code quality issues
//! - Navigation to problem locations

use anyhow::Result;
use git2::{Repository, Status, StatusOptions};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Problems panel component
pub struct ProblemsPanel {
    /// Current problems list
    problems: Vec<Problem>,
    
    /// List selection state
    list_state: ListState,
    
    /// Problem categories
    categories: HashMap<ProblemCategory, Vec<Problem>>,
    
    /// Current filter
    current_filter: ProblemFilter,
    
    /// Repository for git operations
    repo: Option<Repository>,
    
    /// Last update timestamp
    last_update: std::time::Instant,
}

/// Individual problem entry
#[derive(Debug, Clone)]
pub struct Problem {
    /// Problem severity
    pub severity: ProblemSeverity,
    
    /// Problem category
    pub category: ProblemCategory,
    
    /// Description of the problem
    pub message: String,
    
    /// File path where the problem occurs
    pub file_path: Option<PathBuf>,
    
    /// Line number (1-indexed)
    pub line: Option<usize>,
    
    /// Column number (1-indexed)
    pub column: Option<usize>,
    
    /// Source of the problem (compiler, linter, git, etc.)
    pub source: String,
    
    /// Additional context or suggestions
    pub context: Option<String>,
}

/// Problem severity levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProblemSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Problem categories
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProblemCategory {
    GitConflict,
    GitUnstaged,
    GitUntracked,
    BuildError,
    BuildWarning,
    LintError,
    LintWarning,
    TypeCheck,
    Security,
    Performance,
}

/// Problem filtering options
#[derive(Debug, Clone, PartialEq)]
pub enum ProblemFilter {
    All,
    ErrorsOnly,
    WarningsOnly,
    Category(ProblemCategory),
    File(PathBuf),
}

impl ProblemsPanel {
    /// Create new problems panel
    pub fn new(workspace_path: Option<&Path>) -> Self {
        let repo = workspace_path
            .and_then(|path| Repository::discover(path).ok());
            
        let mut panel = Self {
            problems: Vec::new(),
            list_state: ListState::default(),
            categories: HashMap::new(),
            current_filter: ProblemFilter::All,
            repo,
            last_update: std::time::Instant::now(),
        };
        
        // Select first item if available
        if !panel.problems.is_empty() {
            panel.list_state.select(Some(0));
        }
        
        panel
    }
    
    /// Update problems from all sources
    pub async fn update_problems(&mut self, workspace_path: &Path) -> Result<()> {
        self.problems.clear();
        self.categories.clear();
        
        // Update git problems
        self.update_git_problems(workspace_path).await?;
        
        // Update build problems
        self.update_build_problems(workspace_path).await?;
        
        // Update lint problems
        self.update_lint_problems(workspace_path).await?;
        
        // Categorize problems
        self.categorize_problems();
        
        // Update selection state
        if !self.problems.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        } else if self.problems.is_empty() {
            self.list_state.select(None);
        }
        
        self.last_update = std::time::Instant::now();
        Ok(())
    }
    
    /// Update git-related problems
    async fn update_git_problems(&mut self, workspace_path: &Path) -> Result<()> {
        if let Some(repo) = &self.repo {
            let mut status_options = StatusOptions::new();
            status_options.include_untracked(true);
            status_options.include_ignored(false);
            
            if let Ok(statuses) = repo.statuses(Some(&mut status_options)) {
                for entry in statuses.iter() {
                    let status = entry.status();
                    let file_path = entry.path().map(PathBuf::from);
                    
                    // Check for conflicts
                    if status.contains(Status::CONFLICTED) {
                        self.problems.push(Problem {
                            severity: ProblemSeverity::Error,
                            category: ProblemCategory::GitConflict,
                            message: "Merge conflict needs resolution".to_string(),
                            file_path: file_path.clone(),
                            line: None,
                            column: None,
                            source: "git".to_string(),
                            context: Some("Use `git mergetool` or resolve manually".to_string()),
                        });
                    }
                    
                    // Check for unstaged changes
                    if status.contains(Status::WT_MODIFIED) 
                        || status.contains(Status::WT_DELETED)
                        || status.contains(Status::WT_TYPECHANGE)
                        || status.contains(Status::WT_RENAMED) {
                        self.problems.push(Problem {
                            severity: ProblemSeverity::Warning,
                            category: ProblemCategory::GitUnstaged,
                            message: "File has unstaged changes".to_string(),
                            file_path: file_path.clone(),
                            line: None,
                            column: None,
                            source: "git".to_string(),
                            context: Some("Stage changes with `git add`".to_string()),
                        });
                    }
                    
                    // Check for untracked files
                    if status.contains(Status::WT_NEW) {
                        self.problems.push(Problem {
                            severity: ProblemSeverity::Info,
                            category: ProblemCategory::GitUntracked,
                            message: "Untracked file".to_string(),
                            file_path: file_path.clone(),
                            line: None,
                            column: None,
                            source: "git".to_string(),
                            context: Some("Add to git with `git add` or ignore".to_string()),
                        });
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Update build-related problems
    async fn update_build_problems(&mut self, workspace_path: &Path) -> Result<()> {
        // Check if this is a Rust project
        if workspace_path.join("Cargo.toml").exists() {
            self.update_cargo_problems(workspace_path).await?;
        }
        
        // Check if this is a Node.js project
        if workspace_path.join("package.json").exists() {
            self.update_npm_problems(workspace_path).await?;
        }
        
        Ok(())
    }
    
    /// Update Cargo build problems
    async fn update_cargo_problems(&mut self, workspace_path: &Path) -> Result<()> {
        let output = Command::new("cargo")
            .args(&["check", "--message-format=json"])
            .current_dir(workspace_path)
            .output()
            .await;
            
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            for line in stdout.lines() {
                if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(message_obj) = message.get("message") {
                        if let Some(level) = message_obj.get("level").and_then(|l| l.as_str()) {
                            let severity = match level {
                                "error" => ProblemSeverity::Error,
                                "warning" => ProblemSeverity::Warning,
                                "note" | "help" => ProblemSeverity::Info,
                                _ => ProblemSeverity::Hint,
                            };
                            
                            let category = match level {
                                "error" => ProblemCategory::BuildError,
                                "warning" => ProblemCategory::BuildWarning,
                                _ => ProblemCategory::BuildWarning,
                            };
                            
                            let message_text = message_obj
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("Build issue")
                                .to_string();
                            
                            // Extract location information
                            let (file_path, line, column) = if let Some(spans) = message_obj.get("spans") {
                                if let Some(span) = spans.as_array().and_then(|arr| arr.first()) {
                                    let file = span.get("file_name")
                                        .and_then(|f| f.as_str())
                                        .map(PathBuf::from);
                                    let line = span.get("line_start")
                                        .and_then(|l| l.as_u64())
                                        .map(|l| l as usize);
                                    let column = span.get("column_start")
                                        .and_then(|c| c.as_u64())
                                        .map(|c| c as usize);
                                    (file, line, column)
                                } else {
                                    (None, None, None)
                                }
                            } else {
                                (None, None, None)
                            };
                            
                            self.problems.push(Problem {
                                severity,
                                category,
                                message: message_text,
                                file_path,
                                line,
                                column,
                                source: "cargo".to_string(),
                                context: None,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Update npm/Node.js build problems
    async fn update_npm_problems(&mut self, workspace_path: &Path) -> Result<()> {
        // Check TypeScript compilation if tsconfig.json exists
        if workspace_path.join("tsconfig.json").exists() {
            let output = Command::new("npx")
                .args(&["tsc", "--noEmit", "--pretty", "false"])
                .current_dir(workspace_path)
                .output()
                .await;
                
            if let Ok(output) = output {
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                for line in stderr.lines() {
                    if line.contains("error TS") {
                        // Parse TypeScript error format: file(line,col): error TS#### message
                        if let Some(parts) = self.parse_typescript_error(line) {
                            self.problems.push(Problem {
                                severity: ProblemSeverity::Error,
                                category: ProblemCategory::TypeCheck,
                                message: parts.message,
                                file_path: Some(parts.file_path),
                                line: parts.line,
                                column: parts.column,
                                source: "typescript".to_string(),
                                context: None,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Update linting problems
    async fn update_lint_problems(&mut self, workspace_path: &Path) -> Result<()> {
        // Check for clippy if Rust project
        if workspace_path.join("Cargo.toml").exists() {
            let output = Command::new("cargo")
                .args(&["clippy", "--message-format=json"])
                .current_dir(workspace_path)
                .output()
                .await;
                
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                for line in stdout.lines() {
                    if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(message_obj) = message.get("message") {
                            if let Some(level) = message_obj.get("level").and_then(|l| l.as_str()) {
                                if level == "warning" {
                                    let message_text = message_obj
                                        .get("message")
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("Clippy warning")
                                        .to_string();
                                    
                                    // Extract location information
                                    let (file_path, line, column) = if let Some(spans) = message_obj.get("spans") {
                                        if let Some(span) = spans.as_array().and_then(|arr| arr.first()) {
                                            let file = span.get("file_name")
                                                .and_then(|f| f.as_str())
                                                .map(PathBuf::from);
                                            let line = span.get("line_start")
                                                .and_then(|l| l.as_u64())
                                                .map(|l| l as usize);
                                            let column = span.get("column_start")
                                                .and_then(|c| c.as_u64())
                                                .map(|c| c as usize);
                                            (file, line, column)
                                        } else {
                                            (None, None, None)
                                        }
                                    } else {
                                        (None, None, None)
                                    };
                                    
                                    self.problems.push(Problem {
                                        severity: ProblemSeverity::Warning,
                                        category: ProblemCategory::LintWarning,
                                        message: message_text,
                                        file_path,
                                        line,
                                        column,
                                        source: "clippy".to_string(),
                                        context: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse TypeScript error message
    fn parse_typescript_error(&self, line: &str) -> Option<TypeScriptError> {
        // Format: file(line,col): error TS#### message
        let parts: Vec<&str> = line.splitn(2, ": error ").collect();
        if parts.len() != 2 {
            return None;
        }
        
        let location_part = parts[0];
        let message = parts[1].to_string();
        
        // Extract file path and position
        if let Some(paren_pos) = location_part.rfind('(') {
            let file_path = PathBuf::from(&location_part[..paren_pos]);
            let position_part = &location_part[paren_pos + 1..];
            
            if let Some(close_paren) = position_part.find(')') {
                let position = &position_part[..close_paren];
                let coords: Vec<&str> = position.split(',').collect();
                
                if coords.len() == 2 {
                    let line = coords[0].parse().ok();
                    let column = coords[1].parse().ok();
                    
                    return Some(TypeScriptError {
                        file_path,
                        line,
                        column,
                        message,
                    });
                }
            }
        }
        
        None
    }
    
    /// Categorize problems into groups
    fn categorize_problems(&mut self) {
        self.categories.clear();
        
        for problem in &self.problems {
            self.categories
                .entry(problem.category.clone())
                .or_insert_with(Vec::new)
                .push(problem.clone());
        }
    }
    
    /// Apply current filter to problems
    pub fn filtered_problems(&self) -> Vec<&Problem> {
        match &self.current_filter {
            ProblemFilter::All => self.problems.iter().collect(),
            ProblemFilter::ErrorsOnly => self.problems
                .iter()
                .filter(|p| p.severity == ProblemSeverity::Error)
                .collect(),
            ProblemFilter::WarningsOnly => self.problems
                .iter()
                .filter(|p| matches!(p.severity, ProblemSeverity::Warning | ProblemSeverity::Info))
                .collect(),
            ProblemFilter::Category(category) => self.problems
                .iter()
                .filter(|p| &p.category == category)
                .collect(),
            ProblemFilter::File(file_path) => self.problems
                .iter()
                .filter(|p| {
                    p.file_path
                        .as_ref()
                        .map(|path| path == file_path)
                        .unwrap_or(false)
                })
                .collect(),
        }
    }
    
    /// Get summary counts
    pub fn get_summary(&self) -> ProblemSummary {
        let mut errors = 0;
        let mut warnings = 0;
        let mut infos = 0;
        
        for problem in &self.problems {
            match problem.severity {
                ProblemSeverity::Error => errors += 1,
                ProblemSeverity::Warning => warnings += 1,
                ProblemSeverity::Info | ProblemSeverity::Hint => infos += 1,
            }
        }
        
        ProblemSummary {
            total: self.problems.len(),
            errors,
            warnings,
            infos,
        }
    }
    
    /// Navigate to selected problem
    pub fn navigate_to_selected(&self) -> Option<ProblemLocation> {
        if let Some(selected) = self.list_state.selected() {
            let filtered = self.filtered_problems();
            if let Some(problem) = filtered.get(selected) {
                if let Some(file_path) = &problem.file_path {
                    return Some(ProblemLocation {
                        file_path: file_path.clone(),
                        line: problem.line,
                        column: problem.column,
                    });
                }
            }
        }
        None
    }
    
    /// Move selection up
    pub fn select_previous(&mut self) {
        let filtered_count = self.filtered_problems().len();
        if filtered_count == 0 {
            return;
        }
        
        let current = self.list_state.selected().unwrap_or(0);
        let new_index = if current == 0 {
            filtered_count - 1
        } else {
            current - 1
        };
        self.list_state.select(Some(new_index));
    }
    
    /// Move selection down
    pub fn select_next(&mut self) {
        let filtered_count = self.filtered_problems().len();
        if filtered_count == 0 {
            return;
        }
        
        let current = self.list_state.selected().unwrap_or(0);
        let new_index = if current >= filtered_count - 1 {
            0
        } else {
            current + 1
        };
        self.list_state.select(Some(new_index));
    }
    
    /// Set filter
    pub fn set_filter(&mut self, filter: ProblemFilter) {
        self.current_filter = filter;
        
        // Reset selection
        let filtered_count = self.filtered_problems().len();
        if filtered_count > 0 {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }
    
    /// Draw the problems panel
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let summary = self.get_summary();
        
        // Get filtered problems before any mutable borrows
        let filtered_problems: Vec<Problem> = self.filtered_problems().into_iter().cloned().collect();
        
        // Create main block
        let block = Block::default()
            .title(format!(
                " Problems ({} errors, {} warnings, {} total) ",
                summary.errors, summary.warnings, summary.total
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));
        
        // Split area for header and list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);
        
        // Draw header with filter info
        let filter_text = match &self.current_filter {
            ProblemFilter::All => "All Problems".to_string(),
            ProblemFilter::ErrorsOnly => "Errors Only".to_string(),
            ProblemFilter::WarningsOnly => "Warnings Only".to_string(),
            ProblemFilter::Category(cat) => format!("{:?} Problems", cat),
            ProblemFilter::File(path) => format!("File: {}", path.display()),
        };
        
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Filter: ", Style::default().fg(Color::Gray)),
                Span::styled(filter_text, Style::default().fg(Color::White)),
                Span::raw("  "),
                Span::styled("(F5: Refresh, Tab: Filter)", Style::default().fg(Color::DarkGray)),
            ])
        ])
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Left);
        
        frame.render_widget(header, chunks[0]);
        
        // Create list items
        let items: Vec<ListItem> = filtered_problems
            .iter()
            .map(|problem| {
                let severity_icon = match problem.severity {
                    ProblemSeverity::Error => "🔴",
                    ProblemSeverity::Warning => "🟡", 
                    ProblemSeverity::Info => "🔵",
                    ProblemSeverity::Hint => "💡",
                };
                
                let location = if let Some(path) = &problem.file_path {
                    if let Some(line) = problem.line {
                        if let Some(col) = problem.column {
                            format!("{}:{}:{}", path.display(), line, col)
                        } else {
                            format!("{}:{}", path.display(), line)
                        }
                    } else {
                        path.display().to_string()
                    }
                } else {
                    "—".to_string()
                };
                
                let spans = vec![
    Line::from(vec![
                        Span::raw(format!("{} ", severity_icon)),
                        Span::styled(
                            problem.message.clone(),
                            Style::default().fg(match problem.severity {
                                ProblemSeverity::Error => Color::Red,
                                ProblemSeverity::Warning => Color::Yellow,
                                ProblemSeverity::Info => Color::Cyan,
                                ProblemSeverity::Hint => Color::Gray,
                            })
                        ),
                    ]),
    Line::from(vec![
                        Span::styled("    ", Style::default()),
                        Span::styled(location, Style::default().fg(Color::DarkGray)),
                        Span::raw("  "),
                        Span::styled(
                            format!("[{}]", problem.source),
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
                        ),
                    ]),
                ];
                
                ListItem::new(spans)
            })
            .collect();
        
        // Create and render list
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("> ");
        
        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
        
        // Show details popup if a problem is selected
        if let Some(selected) = self.list_state.selected() {
            if let Some(problem) = filtered_problems.get(selected) {
                if let Some(context) = &problem.context {
                    self.draw_problem_details(frame, area, problem, context);
                }
            }
        }
    }
    
    /// Draw problem details popup
    fn draw_problem_details(&self, frame: &mut Frame, area: Rect, problem: &Problem, context: &str) {
        let popup_area = self.centered_rect(60, 20, area);
        
        // Clear the background
        frame.render_widget(Clear, popup_area);
        
        let block = Block::default()
            .title(" Problem Details ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        
        let text = vec![
            Line::from(vec![
                Span::styled("Message: ", Style::default().fg(Color::Gray)),
                Span::raw(&problem.message),
            ]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled("Source: ", Style::default().fg(Color::Gray)),
                Span::raw(&problem.source),
            ]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled("Context: ", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::raw(context),
            ]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled("Press ESC to close", Style::default().fg(Color::DarkGray)),
            ]),
        ];
        
        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, popup_area);
    }
    
    /// Create a centered rectangle
    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
}

/// Problem summary statistics
#[derive(Debug, Clone)]
pub struct ProblemSummary {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
}

/// Location to navigate to for a problem
#[derive(Debug, Clone)]
pub struct ProblemLocation {
    pub file_path: PathBuf,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

/// Parsed TypeScript error
#[derive(Debug)]
struct TypeScriptError {
    file_path: PathBuf,
    line: Option<usize>,
    column: Option<usize>,
    message: String,
}

impl ProblemSeverity {
    /// Get display icon for severity
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Error => "🔴",
            Self::Warning => "🟡",
            Self::Info => "🔵",
            Self::Hint => "💡",
        }
    }
    
    /// Get color for severity
    pub fn color(&self) -> Color {
        match self {
            Self::Error => Color::Red,
            Self::Warning => Color::Yellow,
            Self::Info => Color::Cyan,
            Self::Hint => Color::Gray,
        }
    }
}

impl ProblemCategory {
    /// Get display name for category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::GitConflict => "Git Conflicts",
            Self::GitUnstaged => "Git Unstaged",
            Self::GitUntracked => "Git Untracked",
            Self::BuildError => "Build Errors",
            Self::BuildWarning => "Build Warnings",
            Self::LintError => "Lint Errors",
            Self::LintWarning => "Lint Warnings",
            Self::TypeCheck => "Type Errors",
            Self::Security => "Security Issues",
            Self::Performance => "Performance Issues",
        }
    }
}

impl Default for ProblemsPanel {
    fn default() -> Self {
        Self::new(None)
    }
}