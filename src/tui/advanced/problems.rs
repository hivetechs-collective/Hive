//! Problems Panel for TUI Interface
//!
//! Provides a comprehensive problems view showing:
//! - Git conflicts and merge issues
//! - Build errors and warnings
//! - Code quality issues
//! - Navigation to problem locations

use anyhow::Result;
use git2::{Repository, Status, StatusOptions};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

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
    
    /// Build system monitor
    build_monitor: Option<BuildSystemMonitor>,
    
    /// Event receiver for real-time updates
    event_receiver: Option<mpsc::UnboundedReceiver<BuildEvent>>,
    
    /// Auto-refresh settings
    auto_refresh_enabled: bool,
    refresh_interval: Duration,
    
    /// Status tracking
    build_status: BuildStatus,
    last_build_time: Option<Instant>,
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
    TestFailure,
    FormatError,
    DependencyIssue,
    MemoryLeak,
    DeadCode,
}

/// Problem filtering options
#[derive(Debug, Clone, PartialEq)]
pub enum ProblemFilter {
    All,
    ErrorsOnly,
    WarningsOnly,
    Category(ProblemCategory),
    File(PathBuf),
    Source(String),
    RecentOnly(Duration),
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
            build_monitor: None,
            event_receiver: None,
            auto_refresh_enabled: true,
            refresh_interval: Duration::from_secs(2),
            build_status: BuildStatus::Unknown,
            last_build_time: None,
        };
        
        // Select first item if available
        if !panel.problems.is_empty() {
            panel.list_state.select(Some(0));
        }
        
        panel
    }
    
    /// Initialize build system monitoring
    pub async fn init_build_monitoring(&mut self, workspace_path: &Path) -> Result<()> {
        if workspace_path.join("Cargo.toml").exists() {
            let (monitor, receiver) = BuildSystemMonitor::new(workspace_path).await?;
            self.build_monitor = Some(monitor);
            self.event_receiver = Some(receiver);
            info!("✅ Initialized Rust build system monitoring");
        }
        Ok(())
    }
    
    /// Process pending build events
    pub async fn process_build_events(&mut self) -> Result<bool> {
        let mut updated = false;
        let mut events = Vec::new();
        
        // Collect events first to avoid borrowing conflicts
        if let Some(ref mut receiver) = self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }
        
        // Process events
        for event in events {
            match event {
                BuildEvent::BuildStarted => {
                    self.build_status = BuildStatus::Building;
                    debug!("Build started");
                }
                BuildEvent::BuildCompleted { success, problems } => {
                    self.build_status = if success { BuildStatus::Success } else { BuildStatus::Failed };
                    self.last_build_time = Some(Instant::now());
                    
                    // Update problems from build results
                    self.update_build_problems_from_events(problems);
                    updated = true;
                    
                    info!("Build completed: {}", if success { "✅ Success" } else { "❌ Failed" });
                }
                BuildEvent::FileChanged(paths) => {
                    debug!("Files changed: {:?}", paths);
                    // Trigger automatic build check if enabled
                    if self.auto_refresh_enabled {
                        updated = true;
                    }
                }
                BuildEvent::TestResults { passed, failed, problems } => {
                    info!("Test results: {} passed, {} failed", passed, failed);
                    self.update_test_problems_from_events(problems);
                    updated = true;
                }
            }
        }
        
        Ok(updated)
    }
    
    /// Update problems from all sources
    pub async fn update_problems(&mut self, workspace_path: &Path) -> Result<()> {
        // Don't clear all problems if we have real-time updates
        if self.build_monitor.is_none() {
            self.problems.clear();
        } else {
            // Only clear non-build problems to preserve real-time build results
            self.problems.retain(|p| {
                matches!(p.category, ProblemCategory::BuildError | ProblemCategory::BuildWarning | 
                        ProblemCategory::TestFailure | ProblemCategory::LintWarning)
            });
        }
        
        self.categories.clear();
        
        // Update git problems
        self.update_git_problems(workspace_path).await?;
        
        // Update build problems (if not using real-time monitoring)
        if self.build_monitor.is_none() {
            self.update_build_problems(workspace_path).await?;
            self.update_lint_problems(workspace_path).await?;
            self.update_test_problems(workspace_path).await?;
            self.update_format_problems(workspace_path).await?;
        }
        
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
    
    /// Update Cargo build problems with enhanced error parsing
    async fn update_cargo_problems(&mut self, workspace_path: &Path) -> Result<()> {
        // Run cargo check with JSON output for better parsing
        let output = Command::new("cargo")
            .args(&["check", "--message-format=json", "--all-targets"])
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
                            
                            // Extract enhanced location information
                            let (file_path, line, column, context) = self.extract_cargo_location_info(message_obj, workspace_path);
                            
                            self.problems.push(Problem {
                                severity,
                                category,
                                message: message_text,
                                file_path,
                                line,
                                column,
                                source: "cargo".to_string(),
                                context,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract enhanced location information from Cargo messages
    fn extract_cargo_location_info(&self, message_obj: &serde_json::Value, workspace_path: &Path) -> (Option<PathBuf>, Option<usize>, Option<usize>, Option<String>) {
        if let Some(spans) = message_obj.get("spans") {
            if let Some(span) = spans.as_array().and_then(|arr| arr.first()) {
                let file = span.get("file_name")
                    .and_then(|f| f.as_str())
                    .map(|f| {
                        let path = PathBuf::from(f);
                        if path.is_absolute() {
                            path
                        } else {
                            workspace_path.join(path)
                        }
                    });
                let line = span.get("line_start")
                    .and_then(|l| l.as_u64())
                    .map(|l| l as usize);
                let column = span.get("column_start")
                    .and_then(|c| c.as_u64())
                    .map(|c| c as usize);
                
                // Extract suggested fixes if available
                let context = span.get("suggested_replacement")
                    .and_then(|s| s.as_str())
                    .map(|s| format!("Suggested fix: {}", s))
                    .or_else(|| {
                        span.get("label")
                            .and_then(|l| l.as_str())
                            .map(|l| l.to_string())
                    });
                    
                return (file, line, column, context);
            }
        }
        (None, None, None, None)
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
    
    /// Update linting problems with enhanced clippy integration
    async fn update_lint_problems(&mut self, workspace_path: &Path) -> Result<()> {
        // Check for clippy if Rust project
        if workspace_path.join("Cargo.toml").exists() {
            let output = Command::new("cargo")
                .args(&["clippy", "--message-format=json", "--all-targets", "--", "-W", "clippy::all"])
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
                                    "error" => ProblemCategory::LintError,
                                    _ => ProblemCategory::LintWarning,
                                };
                                
                                let message_text = message_obj
                                    .get("message")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Clippy issue")
                                    .to_string();
                                
                                // Extract location and lint rule information
                                let (file_path, line, column, context) = self.extract_clippy_info(message_obj, workspace_path);
                                
                                self.problems.push(Problem {
                                    severity,
                                    category,
                                    message: message_text,
                                    file_path,
                                    line,
                                    column,
                                    source: "clippy".to_string(),
                                    context,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract enhanced clippy information including lint rules
    fn extract_clippy_info(&self, message_obj: &serde_json::Value, workspace_path: &Path) -> (Option<PathBuf>, Option<usize>, Option<usize>, Option<String>) {
        if let Some(spans) = message_obj.get("spans") {
            if let Some(span) = spans.as_array().and_then(|arr| arr.first()) {
                let file = span.get("file_name")
                    .and_then(|f| f.as_str())
                    .map(|f| {
                        let path = PathBuf::from(f);
                        if path.is_absolute() {
                            path
                        } else {
                            workspace_path.join(path)
                        }
                    });
                let line = span.get("line_start")
                    .and_then(|l| l.as_u64())
                    .map(|l| l as usize);
                let column = span.get("column_start")
                    .and_then(|c| c.as_u64())
                    .map(|c| c as usize);
                
                // Extract lint rule name and suggestion
                let mut context_parts = Vec::new();
                
                if let Some(code) = message_obj.get("code").and_then(|c| c.get("code")).and_then(|c| c.as_str()) {
                    context_parts.push(format!("Rule: {}", code));
                }
                
                if let Some(label) = span.get("label").and_then(|l| l.as_str()) {
                    context_parts.push(label.to_string());
                }
                
                if let Some(suggestion) = span.get("suggested_replacement").and_then(|s| s.as_str()) {
                    context_parts.push(format!("Suggestion: {}", suggestion));
                }
                
                let context = if context_parts.is_empty() {
                    None
                } else {
                    Some(context_parts.join(" | "))
                };
                    
                return (file, line, column, context);
            }
        }
        (None, None, None, None)
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
    
    /// Update test-related problems
    async fn update_test_problems(&mut self, workspace_path: &Path) -> Result<()> {
        if workspace_path.join("Cargo.toml").exists() {
            let output = Command::new("cargo")
                .args(&["test", "--no-run", "--message-format=json"])
                .current_dir(workspace_path)
                .output()
                .await;
                
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                for line in stdout.lines() {
                    if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(message_obj) = message.get("message") {
                            if let Some(level) = message_obj.get("level").and_then(|l| l.as_str()) {
                                if level == "error" {
                                    let message_text = message_obj
                                        .get("message")
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("Test compilation error")
                                        .to_string();
                                    
                                    let (file_path, line, column, _) = self.extract_cargo_location_info(message_obj, workspace_path);
                                    
                                    self.problems.push(Problem {
                                        severity: ProblemSeverity::Error,
                                        category: ProblemCategory::TestFailure,
                                        message: message_text,
                                        file_path,
                                        line,
                                        column,
                                        source: "cargo-test".to_string(),
                                        context: Some("Test compilation failed".to_string()),
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
    
    /// Update format-related problems  
    async fn update_format_problems(&mut self, workspace_path: &Path) -> Result<()> {
        if workspace_path.join("Cargo.toml").exists() {
            let output = Command::new("cargo")
                .args(&["fmt", "--check"])
                .current_dir(workspace_path)
                .output()
                .await;
                
            if let Ok(output) = output {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    
                    for line in stderr.lines() {
                        if line.starts_with("Diff in ") {
                            if let Some(file_path) = line.strip_prefix("Diff in ") {
                                let path = PathBuf::from(file_path.trim());
                                
                                self.problems.push(Problem {
                                    severity: ProblemSeverity::Warning,
                                    category: ProblemCategory::FormatError,
                                    message: "File needs formatting".to_string(),
                                    file_path: Some(workspace_path.join(path)),
                                    line: None,
                                    column: None,
                                    source: "cargo-fmt".to_string(),
                                    context: Some("Run `cargo fmt` to fix formatting".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Update problems from build events
    fn update_build_problems_from_events(&mut self, new_problems: Vec<Problem>) {
        // Remove old build problems
        self.problems.retain(|p| {
            !matches!(p.category, ProblemCategory::BuildError | ProblemCategory::BuildWarning)
        });
        
        // Add new build problems
        self.problems.extend(new_problems);
        
        // Re-categorize
        self.categorize_problems();
    }
    
    /// Update problems from test events
    fn update_test_problems_from_events(&mut self, new_problems: Vec<Problem>) {
        // Remove old test problems
        self.problems.retain(|p| {
            !matches!(p.category, ProblemCategory::TestFailure)
        });
        
        // Add new test problems
        self.problems.extend(new_problems);
        
        // Re-categorize
        self.categorize_problems();
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
            ProblemFilter::Source(source) => self.problems
                .iter()
                .filter(|p| &p.source == source)
                .collect(),
            ProblemFilter::RecentOnly(duration) => {
                let cutoff = self.last_update - *duration;
                self.problems
                    .iter()
                    .collect() // For now, show all - would need timestamps on problems for proper filtering
            }
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
    
    /// Toggle auto-refresh
    pub fn toggle_auto_refresh(&mut self) {
        self.auto_refresh_enabled = !self.auto_refresh_enabled;
        info!("Auto-refresh {}", if self.auto_refresh_enabled { "enabled" } else { "disabled" });
    }
    
    /// Set refresh interval
    pub fn set_refresh_interval(&mut self, interval: Duration) {
        self.refresh_interval = interval;
    }
    
    /// Get current build status
    pub fn build_status(&self) -> &BuildStatus {
        &self.build_status
    }
    
    /// Force refresh all problems
    pub async fn force_refresh(&mut self, workspace_path: &Path) -> Result<()> {
        info!("🔄 Force refreshing all problems...");
        
        // Trigger build check if monitor is available
        if let Some(ref monitor) = self.build_monitor {
            monitor.trigger_build_check().await?;
        }
        
        // Update all problems
        self.update_problems(workspace_path).await?;
        Ok(())
    }
    
    /// Get problem statistics
    pub fn get_detailed_stats(&self) -> DetailedProblemStats {
        let mut stats = DetailedProblemStats::default();
        
        for problem in &self.problems {
            stats.total += 1;
            
            match problem.severity {
                ProblemSeverity::Error => stats.errors += 1,
                ProblemSeverity::Warning => stats.warnings += 1,
                ProblemSeverity::Info => stats.infos += 1,
                ProblemSeverity::Hint => stats.hints += 1,
            }
            
            match problem.category {
                ProblemCategory::BuildError | ProblemCategory::BuildWarning => stats.build_issues += 1,
                ProblemCategory::LintError | ProblemCategory::LintWarning => stats.lint_issues += 1,
                ProblemCategory::TestFailure => stats.test_failures += 1,
                ProblemCategory::GitConflict | ProblemCategory::GitUnstaged | ProblemCategory::GitUntracked => stats.git_issues += 1,
                _ => stats.other_issues += 1,
            }
        }
        
        stats
    }
    
    /// Draw the problems panel
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let summary = self.get_summary();
        
        // Get filtered problems before any mutable borrows
        let filtered_problems: Vec<Problem> = self.filtered_problems().into_iter().cloned().collect();
        
        // Create main block with build status
        let build_status_text = match self.build_status {
            BuildStatus::Building => "🔄 Building",
            BuildStatus::Success => "✅ Build OK",
            BuildStatus::Failed => "❌ Build Failed",
            BuildStatus::Unknown => "",
        };
        
        let title = format!(
            " Problems ({} errors, {} warnings, {} total) {} ",
            summary.errors, summary.warnings, summary.total, build_status_text
        );
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(match self.build_status {
                BuildStatus::Building => Color::Yellow,
                BuildStatus::Success => Color::Green,
                BuildStatus::Failed => Color::Red,
                BuildStatus::Unknown => Color::Blue,
            }));
        
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
            ProblemFilter::Source(source) => format!("Source: {}", source),
            ProblemFilter::RecentOnly(duration) => format!("Recent ({}s)", duration.as_secs()),
        };
        
        let refresh_status = if self.auto_refresh_enabled {
            format!("🔄 Auto ({}s)", self.refresh_interval.as_secs())
        } else {
            "⏸️ Manual".to_string()
        };
        
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Filter: ", Style::default().fg(Color::Gray)),
                Span::styled(filter_text, Style::default().fg(Color::White)),
                Span::raw("  "),
                Span::styled("Refresh: ", Style::default().fg(Color::Gray)),
                Span::styled(refresh_status, Style::default().fg(if self.auto_refresh_enabled { Color::Green } else { Color::Yellow })),
                Span::raw("  "),
                Span::styled("(F5: Refresh, F6: Toggle Auto, Tab: Filter)", Style::default().fg(Color::DarkGray)),
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
            Self::TestFailure => "Test Failures",
            Self::FormatError => "Format Issues",
            Self::DependencyIssue => "Dependencies",
            Self::MemoryLeak => "Memory Issues",
            Self::DeadCode => "Dead Code",
        }
    }
}

impl Default for ProblemsPanel {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Build system monitor for real-time problem detection
struct BuildSystemMonitor {
    workspace_path: PathBuf,
    _file_watcher: notify::RecommendedWatcher,
    event_sender: mpsc::UnboundedSender<BuildEvent>,
}

/// Build events for real-time monitoring  
#[derive(Debug, Clone)]
pub enum BuildEvent {
    BuildStarted,
    BuildCompleted {
        success: bool,
        problems: Vec<Problem>,
    },
    FileChanged(Vec<PathBuf>),
    TestResults {
        passed: usize,
        failed: usize,
        problems: Vec<Problem>,
    },
}

/// Build status tracking
#[derive(Debug, Clone, PartialEq)]
pub enum BuildStatus {
    Unknown,
    Building,
    Success,
    Failed,
}

/// Detailed problem statistics
#[derive(Debug, Default)]
pub struct DetailedProblemStats {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub hints: usize,
    pub build_issues: usize,
    pub lint_issues: usize,
    pub test_failures: usize,
    pub git_issues: usize,
    pub other_issues: usize,
}

impl BuildSystemMonitor {
    /// Create new build system monitor
    async fn new(workspace_path: &Path) -> Result<(Self, mpsc::UnboundedReceiver<BuildEvent>)> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let (fs_sender, fs_receiver) = std::sync::mpsc::channel();
        
        let mut watcher = notify::recommended_watcher(fs_sender)?;
        
        // Watch Rust source files
        watcher.watch(&workspace_path.join("src"), RecursiveMode::Recursive)?;
        watcher.watch(&workspace_path.join("Cargo.toml"), RecursiveMode::NonRecursive)?;
        
        // Watch other common directories if they exist
        if workspace_path.join("tests").exists() {
            watcher.watch(&workspace_path.join("tests"), RecursiveMode::Recursive)?;
        }
        if workspace_path.join("benches").exists() {
            watcher.watch(&workspace_path.join("benches"), RecursiveMode::Recursive)?;
        }
        if workspace_path.join("examples").exists() {
            watcher.watch(&workspace_path.join("examples"), RecursiveMode::Recursive)?;
        }
        
        let workspace_clone = workspace_path.to_path_buf();
        let event_sender_clone = event_sender.clone();
        
        // Start file watcher thread
        std::thread::spawn(move || {
            Self::watch_file_changes(fs_receiver, event_sender_clone, workspace_clone);
        });
        
        let monitor = Self {
            workspace_path: workspace_path.to_path_buf(),
            _file_watcher: watcher,
            event_sender,
        };
        
        Ok((monitor, event_receiver))
    }
    
    /// Watch for file changes and trigger builds
    fn watch_file_changes(
        receiver: std::sync::mpsc::Receiver<notify::Result<Event>>,
        event_sender: mpsc::UnboundedSender<BuildEvent>,
        workspace_path: PathBuf,
    ) {
        let mut last_build = Instant::now();
        let build_debounce = Duration::from_millis(500);
        
        loop {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    if let Some(changed_files) = Self::process_file_event(&event) {
                        let _ = event_sender.send(BuildEvent::FileChanged(changed_files.clone()));
                        
                        // Debounce builds
                        if last_build.elapsed() > build_debounce {
                            last_build = Instant::now();
                            
                            // Trigger build in background
                            let workspace_clone = workspace_path.clone();
                            let sender_clone = event_sender.clone();
                            
                            tokio::spawn(async move {
                                if let Err(e) = Self::run_background_build(workspace_clone, sender_clone).await {
                                    error!("Background build failed: {}", e);
                                }
                            });
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("File watcher error: {}", e);
                }
                Err(_) => {
                    // Timeout, continue
                }
            }
        }
    }
    
    /// Process file system event and extract changed files
    fn process_file_event(event: &Event) -> Option<Vec<PathBuf>> {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                let mut changed_files = Vec::new();
                
                for path in &event.paths {
                    // Only monitor Rust files and Cargo.toml
                    if path.extension().map_or(false, |ext| ext == "rs") ||
                       path.file_name().map_or(false, |name| name == "Cargo.toml") {
                        changed_files.push(path.clone());
                    }
                }
                
                if !changed_files.is_empty() {
                    Some(changed_files)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Run background build and report results
    async fn run_background_build(
        workspace_path: PathBuf,
        event_sender: mpsc::UnboundedSender<BuildEvent>,
    ) -> Result<()> {
        let _ = event_sender.send(BuildEvent::BuildStarted);
        
        // Run cargo check
        let output = Command::new("cargo")
            .args(&["check", "--message-format=json", "--all-targets"])
            .current_dir(&workspace_path)
            .output()
            .await?;
        
        let mut problems = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse build results
        for line in stdout.lines() {
            if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(problem) = Self::parse_cargo_message(&message, &workspace_path) {
                    problems.push(problem);
                }
            }
        }
        
        let success = output.status.success();
        let _ = event_sender.send(BuildEvent::BuildCompleted { success, problems });
        
        Ok(())
    }
    
    /// Parse cargo message into problem
    fn parse_cargo_message(message: &serde_json::Value, workspace_path: &Path) -> Option<Problem> {
        let message_obj = message.get("message")?;
        let level = message_obj.get("level")?.as_str()?;
        
        let severity = match level {
            "error" => ProblemSeverity::Error,
            "warning" => ProblemSeverity::Warning,
            "note" | "help" => ProblemSeverity::Info,
            _ => return None,
        };
        
        let category = match level {
            "error" => ProblemCategory::BuildError,
            "warning" => ProblemCategory::BuildWarning,
            _ => ProblemCategory::BuildWarning,
        };
        
        let message_text = message_obj
            .get("message")?
            .as_str()?
            .to_string();
        
        // Extract location
        let (file_path, line, column) = if let Some(spans) = message_obj.get("spans") {
            if let Some(span) = spans.as_array()?.first() {
                let file = span.get("file_name")
                    .and_then(|f| f.as_str())
                    .map(|f| {
                        let path = PathBuf::from(f);
                        if path.is_absolute() {
                            path
                        } else {
                            workspace_path.join(path)
                        }
                    });
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
        
        Some(Problem {
            severity,
            category,
            message: message_text,
            file_path,
            line,
            column,
            source: "cargo".to_string(),
            context: None,
        })
    }
    
    /// Trigger manual build check
    async fn trigger_build_check(&self) -> Result<()> {
        let workspace_clone = self.workspace_path.clone();
        let sender_clone = self.event_sender.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::run_background_build(workspace_clone, sender_clone).await {
                error!("Manual build check failed: {}", e);
            }
        });
        
        Ok(())
    }
}