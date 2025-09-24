//! Problems Panel Component
//!
//! VS Code-style problems panel that aggregates and displays:
//! - Git merge conflicts
//! - Build compilation errors
//! - Linting warnings
//! - Other diagnostic information

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Problem severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProblemSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl ProblemSeverity {
    /// Get icon for this severity
    pub fn icon(&self) -> &'static str {
        match self {
            ProblemSeverity::Error => "error",
            ProblemSeverity::Warning => "warning",
            ProblemSeverity::Info => "info",
            ProblemSeverity::Hint => "lightbulb",
        }
    }

    /// Get color for this severity
    pub fn color(&self) -> &'static str {
        match self {
            ProblemSeverity::Error => "#f48771",
            ProblemSeverity::Warning => "#cca700",
            ProblemSeverity::Info => "#75beff",
            ProblemSeverity::Hint => "#cccccc",
        }
    }

    /// Get priority for sorting (lower number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            ProblemSeverity::Error => 0,
            ProblemSeverity::Warning => 1,
            ProblemSeverity::Info => 2,
            ProblemSeverity::Hint => 3,
        }
    }
}

/// Source of the problem
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProblemSource {
    Git,
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Clippy,
    ESLint,
    Other(String),
}

impl ProblemSource {
    /// Get display name for the source
    pub fn display_name(&self) -> &str {
        match self {
            ProblemSource::Git => "Git",
            ProblemSource::Rust => "rustc",
            ProblemSource::JavaScript => "JavaScript",
            ProblemSource::TypeScript => "TypeScript",
            ProblemSource::Python => "Python",
            ProblemSource::Go => "Go",
            ProblemSource::Clippy => "clippy",
            ProblemSource::ESLint => "ESLint",
            ProblemSource::Other(name) => name,
        }
    }
}

/// Individual problem item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProblemItem {
    /// Unique identifier for the problem
    pub id: String,
    /// Problem severity
    pub severity: ProblemSeverity,
    /// Source that generated this problem
    pub source: ProblemSource,
    /// Problem message
    pub message: String,
    /// File path where the problem occurs
    pub file_path: Option<PathBuf>,
    /// Line number (1-based)
    pub line: Option<u32>,
    /// Column number (1-based)
    pub column: Option<u32>,
    /// End line for multi-line problems
    pub end_line: Option<u32>,
    /// End column for multi-line problems
    pub end_column: Option<u32>,
    /// Optional diagnostic code
    pub code: Option<String>,
    /// When this problem was first detected
    pub timestamp: DateTime<Utc>,
    /// Whether this problem can be auto-fixed
    pub fixable: bool,
}

impl ProblemItem {
    /// Create a new problem item
    pub fn new(severity: ProblemSeverity, source: ProblemSource, message: String) -> Self {
        Self {
            id: format!(
                "problem_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            severity,
            source,
            message,
            file_path: None,
            line: None,
            column: None,
            end_line: None,
            end_column: None,
            code: None,
            timestamp: Utc::now(),
            fixable: false,
        }
    }

    /// Set file location
    pub fn with_location(mut self, file_path: PathBuf, line: u32, column: u32) -> Self {
        self.file_path = Some(file_path);
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Set range location
    pub fn with_range(
        mut self,
        file_path: PathBuf,
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
    ) -> Self {
        self.file_path = Some(file_path);
        self.line = Some(start_line);
        self.column = Some(start_col);
        self.end_line = Some(end_line);
        self.end_column = Some(end_col);
        self
    }

    /// Set diagnostic code
    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    /// Mark as fixable
    pub fn fixable(mut self) -> Self {
        self.fixable = true;
        self
    }

    /// Get location string for display
    pub fn location_string(&self) -> String {
        if let Some(path) = &self.file_path {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            if let (Some(line), Some(col)) = (self.line, self.column) {
                format!("{}:{}:{}", file_name, line, col)
            } else {
                file_name.to_string()
            }
        } else {
            "unknown".to_string()
        }
    }
}

/// Problems panel state
#[derive(Debug, Clone)]
pub struct ProblemsState {
    /// All problems organized by source
    pub problems: HashMap<ProblemSource, Vec<ProblemItem>>,
    /// Current filter settings
    pub filter: ProblemFilter,
    /// Currently selected problem
    pub selected_problem: Option<String>,
    /// Whether the panel is visible
    pub visible: bool,
}

/// Problem filtering options
#[derive(Debug, Clone)]
pub struct ProblemFilter {
    /// Show errors
    pub show_errors: bool,
    /// Show warnings
    pub show_warnings: bool,
    /// Show info
    pub show_info: bool,
    /// Show hints
    pub show_hints: bool,
    /// Filter by source
    pub source_filter: Option<ProblemSource>,
    /// Filter by file path
    pub file_filter: Option<PathBuf>,
}

impl Default for ProblemFilter {
    fn default() -> Self {
        Self {
            show_errors: true,
            show_warnings: true,
            show_info: false,
            show_hints: false,
            source_filter: None,
            file_filter: None,
        }
    }
}

impl Default for ProblemsState {
    fn default() -> Self {
        Self {
            problems: HashMap::new(),
            filter: ProblemFilter::default(),
            selected_problem: None,
            visible: false,
        }
    }
}

impl ProblemsState {
    /// Add a problem to the state
    pub fn add_problem(&mut self, problem: ProblemItem) {
        let source = problem.source.clone();
        self.problems
            .entry(source)
            .or_insert_with(Vec::new)
            .push(problem);
    }

    /// Remove all problems from a specific source
    pub fn clear_source(&mut self, source: &ProblemSource) {
        self.problems.remove(source);
    }

    /// Get filtered problems for display
    pub fn filtered_problems(&self) -> Vec<&ProblemItem> {
        let mut problems = Vec::new();

        for source_problems in self.problems.values() {
            for problem in source_problems {
                if self.filter.matches(problem) {
                    problems.push(problem);
                }
            }
        }

        // Sort by severity, then by file, then by line
        problems.sort_by(|a, b| {
            use std::cmp::Ordering;

            // First by severity (errors first)
            match a.severity.priority().cmp(&b.severity.priority()) {
                Ordering::Equal => {
                    // Then by file path
                    match (&a.file_path, &b.file_path) {
                        (Some(a_path), Some(b_path)) => {
                            match a_path.cmp(b_path) {
                                Ordering::Equal => {
                                    // Then by line number
                                    a.line.unwrap_or(0).cmp(&b.line.unwrap_or(0))
                                }
                                other => other,
                            }
                        }
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (None, None) => Ordering::Equal,
                    }
                }
                other => other,
            }
        });

        problems
    }

    /// Get counts by severity
    pub fn get_counts(&self) -> (u32, u32, u32, u32) {
        let mut errors = 0;
        let mut warnings = 0;
        let mut info = 0;
        let mut hints = 0;

        for problem in self.filtered_problems() {
            match problem.severity {
                ProblemSeverity::Error => errors += 1,
                ProblemSeverity::Warning => warnings += 1,
                ProblemSeverity::Info => info += 1,
                ProblemSeverity::Hint => hints += 1,
            }
        }

        (errors, warnings, info, hints)
    }

    /// Update problems from git conflicts
    pub fn update_git_problems(
        &mut self,
        file_statuses: &std::collections::HashMap<
            std::path::PathBuf,
            super::super::git::FileStatus,
        >,
    ) {
        // Clear existing git problems
        self.clear_source(&ProblemSource::Git);

        // Add conflict problems
        for (path, status) in file_statuses
            .iter()
            .filter(|(_, status)| status.status_type == super::super::git::StatusType::Conflicted)
        {
            let problem = ProblemItem::new(
                ProblemSeverity::Error,
                ProblemSource::Git,
                format!("Merge conflict in {}", path.display()),
            )
            .with_location(path.clone(), 1, 1);

            self.add_problem(problem);
        }
    }

    /// Update problems from build output
    pub fn update_build_problems(&mut self, source: ProblemSource, output: &str) {
        // Clear existing problems from this source
        self.clear_source(&source);

        // Parse build output (simplified parser for common formats)
        self.parse_build_output(source, output);
    }

    /// Parse build output and extract problems
    fn parse_build_output(&mut self, source: ProblemSource, output: &str) {
        for line in output.lines() {
            if let Some(problem) = self.parse_build_line(&source, line) {
                self.add_problem(problem);
            }
        }
    }

    /// Parse a single build output line (simplified without regex)
    fn parse_build_line(&self, source: &ProblemSource, line: &str) -> Option<ProblemItem> {
        // Simple parser for Rust compiler format: file.rs:line:col: error[E0xxx]: message
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 4 {
            let file_path = PathBuf::from(parts[0]);
            if let (Ok(line_num), Ok(col_num)) = (parts[1].parse::<u32>(), parts[2].parse::<u32>())
            {
                let rest = parts[3..].join(":");

                if let Some(error_start) = rest.find("error") {
                    let message = rest[error_start..].to_string();
                    let problem = ProblemItem::new(ProblemSeverity::Error, source.clone(), message)
                        .with_location(file_path, line_num, col_num);
                    return Some(problem);
                } else if let Some(warning_start) = rest.find("warning") {
                    let message = rest[warning_start..].to_string();
                    let problem =
                        ProblemItem::new(ProblemSeverity::Warning, source.clone(), message)
                            .with_location(file_path, line_num, col_num);
                    return Some(problem);
                }
            }
        }

        None
    }
}

impl ProblemFilter {
    /// Check if a problem matches the current filter
    pub fn matches(&self, problem: &ProblemItem) -> bool {
        // Check severity filter
        let severity_matches = match problem.severity {
            ProblemSeverity::Error => self.show_errors,
            ProblemSeverity::Warning => self.show_warnings,
            ProblemSeverity::Info => self.show_info,
            ProblemSeverity::Hint => self.show_hints,
        };

        if !severity_matches {
            return false;
        }

        // Check source filter
        if let Some(ref source_filter) = self.source_filter {
            if &problem.source != source_filter {
                return false;
            }
        }

        // Check file filter
        if let Some(ref file_filter) = self.file_filter {
            if let Some(ref problem_file) = problem.file_path {
                if problem_file != file_filter {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Problems Panel Component
#[component]
pub fn ProblemsPanel(
    state: Signal<ProblemsState>,
    on_problem_select: EventHandler<String>,
    on_problem_navigate: EventHandler<(PathBuf, u32, u32)>,
) -> Element {
    let problems_state = state.read();
    let filtered_problems = problems_state.filtered_problems();
    let (errors, warnings, info, hints) = problems_state.get_counts();

    if !problems_state.visible {
        return rsx! { div { class: "problems-panel hidden" } };
    }

    rsx! {
        div {
            class: "problems-panel",

            // Header with counts and filters
            div {
                class: "problems-header",

                div {
                    class: "problems-title",
                    "Problems"
                }

                div {
                    class: "problems-counts",

                    if errors > 0 {
                        span {
                            class: "problem-count error",
                            title: format!("{} error{}", errors, if errors == 1 { "" } else { "s" }),
                            span { class: "codicon codicon-error" }
                            "{errors}"
                        }
                    }

                    if warnings > 0 {
                        span {
                            class: "problem-count warning",
                            title: format!("{} warning{}", warnings, if warnings == 1 { "" } else { "s" }),
                            span { class: "codicon codicon-warning" }
                            "{warnings}"
                        }
                    }

                    if info > 0 {
                        span {
                            class: "problem-count info",
                            title: format!("{} info message{}", info, if info == 1 { "" } else { "s" }),
                            span { class: "codicon codicon-info" }
                            "{info}"
                        }
                    }

                    if hints > 0 {
                        span {
                            class: "problem-count hint",
                            title: format!("{} hint{}", hints, if hints == 1 { "" } else { "s" }),
                            span { class: "codicon codicon-lightbulb" }
                            "{hints}"
                        }
                    }
                }

                div {
                    class: "problems-actions",

                    button {
                        class: "problems-filter-btn",
                        title: "Filter Problems",
                        span { class: "codicon codicon-filter" }
                    }

                    button {
                        class: "problems-clear-btn",
                        title: "Clear All Problems",
                        onclick: move |_| {
                            state.write().problems.clear();
                        },
                        span { class: "codicon codicon-clear-all" }
                    }
                }
            }

            // Problems list
            div {
                class: "problems-list",

                if filtered_problems.is_empty() {
                    div {
                        class: "problems-empty",
                        span { class: "codicon codicon-check" }
                        "No problems detected"
                    }
                } else {
                    for problem in filtered_problems {
                        div {
                            class: if problems_state.selected_problem.as_ref() == Some(&problem.id) {
                                "problem-item selected"
                            } else {
                                "problem-item"
                            },
                            onclick: {
                                let problem_id = problem.id.clone();
                                let file_path = problem.file_path.clone();
                                let line = problem.line.unwrap_or(1);
                                let column = problem.column.unwrap_or(1);
                                move |_| {
                                    on_problem_select.call(problem_id.clone());
                                    if let Some(path) = &file_path {
                                        on_problem_navigate.call((path.clone(), line, column));
                                    }
                                }
                            },

                            div {
                                class: "problem-icon",
                                span {
                                    class: "codicon codicon-{problem.severity.icon()}",
                                    style: "color: {problem.severity.color()};",
                                }
                            }

                            div {
                                class: "problem-content",

                                div {
                                    class: "problem-message",
                                    "{problem.message}"

                                    if let Some(code) = &problem.code {
                                        span {
                                            class: "problem-code",
                                            "({code})"
                                        }
                                    }
                                }

                                div {
                                    class: "problem-location",
                                    span {
                                        class: "problem-source",
                                        "{problem.source.display_name()}"
                                    }
                                    " • "
                                    span {
                                        class: "problem-file",
                                        "{problem.location_string()}"
                                    }
                                }
                            }

                            if problem.fixable {
                                div {
                                    class: "problem-actions",
                                    button {
                                        class: "problem-fix-btn",
                                        title: "Quick Fix",
                                        span { class: "codicon codicon-lightbulb" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// CSS styles for the problems panel
pub const PROBLEMS_PANEL_STYLES: &str = r#"
.problems-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--vscode-panel-background, #1e1e1e);
    color: var(--vscode-panel-foreground, #cccccc);
    font-size: 13px;
}

.problems-panel.hidden {
    display: none;
}

.problems-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--vscode-panel-border, #2a2a2a);
    background: var(--vscode-panelTitle-activeBorder, #1e1e1e);
}

.problems-title {
    font-weight: 600;
    color: var(--vscode-panelTitle-activeForeground, #e7e7e7);
}

.problems-counts {
    display: flex;
    align-items: center;
    gap: 12px;
}

.problem-count {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    font-weight: 500;
}

.problem-count.error {
    color: #f48771;
}

.problem-count.warning {
    color: #cca700;
}

.problem-count.info {
    color: #75beff;
}

.problem-count.hint {
    color: #cccccc;
}

.problems-actions {
    display: flex;
    align-items: center;
    gap: 4px;
}

.problems-filter-btn,
.problems-clear-btn {
    background: none;
    border: none;
    color: var(--vscode-icon-foreground, #cccccc);
    cursor: pointer;
    padding: 4px;
    border-radius: 3px;
    transition: background-color 0.1s ease;
}

.problems-filter-btn:hover,
.problems-clear-btn:hover {
    background: var(--vscode-toolbar-hoverBackground, #2a2a2a);
}

.problems-list {
    flex: 1;
    overflow-y: auto;
    padding: 0;
}

.problems-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 20px;
    color: var(--vscode-descriptionForeground, #cccccc);
    font-style: italic;
}

.problem-item {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 12px;
    cursor: pointer;
    border-bottom: 1px solid transparent;
    transition: background-color 0.1s ease;
}

.problem-item:hover {
    background: var(--vscode-list-hoverBackground, #2a2a2a);
}

.problem-item.selected {
    background: var(--vscode-list-activeSelectionBackground, #0e639c);
    color: var(--vscode-list-activeSelectionForeground, #ffffff);
}

.problem-icon {
    flex-shrink: 0;
    margin-top: 2px;
}

.problem-content {
    flex: 1;
    min-width: 0;
}

.problem-message {
    font-weight: 500;
    margin-bottom: 2px;
    word-wrap: break-word;
}

.problem-code {
    color: var(--vscode-textLink-foreground, #3794ff);
    font-weight: normal;
    margin-left: 4px;
}

.problem-location {
    font-size: 12px;
    color: var(--vscode-descriptionForeground, #999999);
    display: flex;
    align-items: center;
    gap: 4px;
}

.problem-source {
    font-weight: 500;
}

.problem-file {
    opacity: 0.8;
}

.problem-actions {
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.1s ease;
}

.problem-item:hover .problem-actions {
    opacity: 1;
}

.problem-fix-btn {
    background: none;
    border: none;
    color: #cca700;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 3px;
    transition: background-color 0.1s ease;
}

.problem-fix-btn:hover {
    background: rgba(204, 167, 0, 0.2);
}

/* Scrollbar styling */
.problems-list::-webkit-scrollbar {
    width: 8px;
}

.problems-list::-webkit-scrollbar-track {
    background: var(--vscode-scrollbarSlider-background, transparent);
}

.problems-list::-webkit-scrollbar-thumb {
    background: var(--vscode-scrollbarSlider-background, #424242);
    border-radius: 4px;
}

.problems-list::-webkit-scrollbar-thumb:hover {
    background: var(--vscode-scrollbarSlider-hoverBackground, #4f4f4f);
}
"#;
