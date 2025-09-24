//! Build System Integration for Problems Panel
//!
//! Connects build system monitors with the problems panel to provide
//! real-time updates of compilation errors, warnings, and other issues.

use super::problems_panel::{ProblemItem, ProblemSeverity, ProblemSource, ProblemsState};
use crate::desktop::events::{Event, EventHandler, EventType};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Output;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Build system integration manager
pub struct BuildSystemIntegration {
    workspace_path: PathBuf,
    active_monitors: HashMap<BuildTool, BuildMonitor>,
    problems_state: ProblemsState,
}

/// Supported build tools
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildTool {
    Cargo,
    Npm,
    Clippy,
    RustFmt,
    TypeScript,
    ESLint,
}

/// Individual build monitor
struct BuildMonitor {
    tool: BuildTool,
    last_run: Option<std::time::Instant>,
    current_problems: Vec<ProblemItem>,
}

impl BuildSystemIntegration {
    /// Create new build system integration
    pub fn new(workspace_path: PathBuf) -> Self {
        Self {
            workspace_path,
            active_monitors: HashMap::new(),
            problems_state: ProblemsState::default(),
        }
    }

    /// Initialize build system integration
    pub async fn initialize(&mut self) -> Result<()> {
        info!(
            "🔧 Initializing build system integration for: {}",
            self.workspace_path.display()
        );

        // Detect available build tools and create monitors
        self.detect_and_setup_monitors().await?;

        // Set up event handlers for file changes
        self.setup_file_change_handlers().await?;

        // Run initial build checks
        self.run_initial_checks().await?;

        info!(
            "✅ Build system integration initialized with {} monitors",
            self.active_monitors.len()
        );
        Ok(())
    }

    /// Detect available build tools and set up monitors
    async fn detect_and_setup_monitors(&mut self) -> Result<()> {
        // Check for Rust project (Cargo.toml)
        if self.workspace_path.join("Cargo.toml").exists() {
            self.add_monitor(BuildTool::Cargo).await?;
            self.add_monitor(BuildTool::Clippy).await?;
            self.add_monitor(BuildTool::RustFmt).await?;
            debug!("📦 Detected Rust project - added Cargo, Clippy, RustFmt monitors");
        }

        // Check for Node.js project (package.json)
        if self.workspace_path.join("package.json").exists() {
            self.add_monitor(BuildTool::Npm).await?;
            debug!("📦 Detected Node.js project - added NPM monitor");

            // Check for TypeScript
            if self.workspace_path.join("tsconfig.json").exists() {
                self.add_monitor(BuildTool::TypeScript).await?;
                debug!("📦 Detected TypeScript - added TypeScript monitor");
            }

            // Check for ESLint
            if self.workspace_path.join(".eslintrc.json").exists()
                || self.workspace_path.join(".eslintrc.js").exists()
                || self.workspace_path.join("eslint.config.js").exists()
            {
                self.add_monitor(BuildTool::ESLint).await?;
                debug!("📦 Detected ESLint - added ESLint monitor");
            }
        }

        Ok(())
    }

    /// Add a build monitor for a specific tool
    async fn add_monitor(&mut self, tool: BuildTool) -> Result<()> {
        let monitor = BuildMonitor {
            tool: tool.clone(),
            last_run: None,
            current_problems: Vec::new(),
        };

        self.active_monitors.insert(tool, monitor);
        Ok(())
    }

    /// Set up event handlers for file changes
    async fn setup_file_change_handlers(&self) -> Result<()> {
        // This would integrate with the existing event system
        // For now, we'll provide the interface structure
        debug!("📡 Setting up file change handlers for build integration");
        Ok(())
    }

    /// Run initial build checks for all detected tools
    async fn run_initial_checks(&mut self) -> Result<()> {
        info!("🔍 Running initial build checks...");

        let tools: Vec<BuildTool> = self.active_monitors.keys().cloned().collect();

        for tool in tools {
            match self.run_build_check(&tool).await {
                Ok(()) => debug!("✅ Initial check completed for {:?}", tool),
                Err(e) => warn!("⚠️ Initial check failed for {:?}: {}", tool, e),
            }
        }

        Ok(())
    }

    /// Run build check for a specific tool
    pub async fn run_build_check(&mut self, tool: &BuildTool) -> Result<()> {
        debug!("🔧 Running build check for {:?}", tool);

        let start_time = std::time::Instant::now();
        let output = self.execute_build_command(tool).await?;
        let duration = start_time.elapsed();

        // Parse the output and extract problems
        let problems = self.parse_build_output(tool, &output).await?;

        // Update the monitor with new problems
        if let Some(monitor) = self.active_monitors.get_mut(tool) {
            monitor.last_run = Some(start_time);
            monitor.current_problems = problems.clone();
        }

        // Update the problems state
        self.update_problems_state(tool, problems).await?;

        info!(
            "🔧 Build check for {:?} completed in {:?} - found {} problems",
            tool,
            duration,
            self.active_monitors
                .get(tool)
                .map(|m| m.current_problems.len())
                .unwrap_or(0)
        );

        Ok(())
    }

    /// Execute build command for a specific tool
    async fn execute_build_command(&self, tool: &BuildTool) -> Result<Output> {
        let (command, args) = self.get_build_command(tool);

        debug!("Executing: {} {}", command, args.join(" "));

        let output = Command::new(command)
            .args(&args)
            .current_dir(&self.workspace_path)
            .output()
            .await?;

        Ok(output)
    }

    /// Get build command and arguments for a tool
    fn get_build_command(&self, tool: &BuildTool) -> (&str, Vec<&str>) {
        match tool {
            BuildTool::Cargo => (
                "cargo",
                vec!["check", "--message-format=json", "--all-targets"],
            ),
            BuildTool::Clippy => (
                "cargo",
                vec![
                    "clippy",
                    "--message-format=json",
                    "--all-targets",
                    "--",
                    "-W",
                    "clippy::all",
                ],
            ),
            BuildTool::RustFmt => ("cargo", vec!["fmt", "--check"]),
            BuildTool::Npm => ("npm", vec!["run", "build"]),
            BuildTool::TypeScript => ("npx", vec!["tsc", "--noEmit", "--pretty", "false"]),
            BuildTool::ESLint => ("npx", vec!["eslint", ".", "--format", "json"]),
        }
    }

    /// Parse build output and extract problems
    async fn parse_build_output(
        &self,
        tool: &BuildTool,
        output: &Output,
    ) -> Result<Vec<ProblemItem>> {
        let mut problems = Vec::new();

        match tool {
            BuildTool::Cargo | BuildTool::Clippy => {
                problems.extend(self.parse_cargo_json_output(&output.stdout).await?);
            }
            BuildTool::RustFmt => {
                problems.extend(self.parse_rustfmt_output(&output.stderr).await?);
            }
            BuildTool::TypeScript => {
                problems.extend(self.parse_typescript_output(&output.stderr).await?);
            }
            BuildTool::ESLint => {
                problems.extend(self.parse_eslint_output(&output.stdout).await?);
            }
            BuildTool::Npm => {
                problems.extend(self.parse_npm_output(&output.stderr).await?);
            }
        }

        Ok(problems)
    }

    /// Parse Cargo/Clippy JSON output
    async fn parse_cargo_json_output(&self, stdout: &[u8]) -> Result<Vec<ProblemItem>> {
        let mut problems = Vec::new();
        let stdout_str = String::from_utf8_lossy(stdout);

        for line in stdout_str.lines() {
            if let Ok(message) = serde_json::from_str::<Value>(line) {
                if let Some(problem) = self.parse_cargo_message(&message).await {
                    problems.push(problem);
                }
            }
        }

        Ok(problems)
    }

    /// Parse individual Cargo message
    async fn parse_cargo_message(&self, message: &Value) -> Option<ProblemItem> {
        let message_obj = message.get("message")?;
        let level = message_obj.get("level")?.as_str()?;

        let severity = match level {
            "error" => ProblemSeverity::Error,
            "warning" => ProblemSeverity::Warning,
            "note" | "help" => ProblemSeverity::Info,
            _ => ProblemSeverity::Hint,
        };

        let source = if message.get("message")?.get("code").is_some() {
            ProblemSource::Clippy
        } else {
            ProblemSource::Rust
        };

        let message_text = message_obj.get("message")?.as_str()?.to_string();

        // Extract location information
        let (file_path, line, column, code) = self.extract_cargo_location(message_obj);

        let mut problem = ProblemItem::new(severity, source, message_text);

        if let Some(path) = file_path {
            problem = problem.with_location(path, line.unwrap_or(1), column.unwrap_or(1));
        }

        if let Some(diagnostic_code) = code {
            problem = problem.with_code(diagnostic_code);
        }

        Some(problem)
    }

    /// Extract location information from Cargo message
    fn extract_cargo_location(
        &self,
        message_obj: &Value,
    ) -> (Option<PathBuf>, Option<u32>, Option<u32>, Option<String>) {
        if let Some(spans) = message_obj.get("spans") {
            if let Some(span) = spans.as_array().and_then(|arr| arr.first()) {
                let file_path = span.get("file_name").and_then(|f| f.as_str()).map(|f| {
                    let path = PathBuf::from(f);
                    if path.is_absolute() {
                        path
                    } else {
                        self.workspace_path.join(path)
                    }
                });

                let line = span
                    .get("line_start")
                    .and_then(|l| l.as_u64())
                    .map(|l| l as u32);

                let column = span
                    .get("column_start")
                    .and_then(|c| c.as_u64())
                    .map(|c| c as u32);

                let code = message_obj
                    .get("code")
                    .and_then(|c| c.get("code"))
                    .and_then(|c| c.as_str())
                    .map(|c| c.to_string());

                return (file_path, line, column, code);
            }
        }
        (None, None, None, None)
    }

    /// Parse rustfmt output
    async fn parse_rustfmt_output(&self, stderr: &[u8]) -> Result<Vec<ProblemItem>> {
        let mut problems = Vec::new();
        let stderr_str = String::from_utf8_lossy(stderr);

        for line in stderr_str.lines() {
            if line.starts_with("Diff in ") {
                if let Some(file_path) = line.strip_prefix("Diff in ") {
                    let path = PathBuf::from(file_path.trim());
                    let full_path = self.workspace_path.join(path);

                    let problem = ProblemItem::new(
                        ProblemSeverity::Warning,
                        ProblemSource::Other("rustfmt".to_string()),
                        "File needs formatting".to_string(),
                    )
                    .with_location(full_path, 1, 1);

                    problems.push(problem);
                }
            }
        }

        Ok(problems)
    }

    /// Parse TypeScript output
    async fn parse_typescript_output(&self, stderr: &[u8]) -> Result<Vec<ProblemItem>> {
        let mut problems = Vec::new();
        let stderr_str = String::from_utf8_lossy(stderr);

        for line in stderr_str.lines() {
            if line.contains("error TS") {
                if let Some(problem) = self.parse_typescript_line(line) {
                    problems.push(problem);
                }
            }
        }

        Ok(problems)
    }

    /// Parse single TypeScript error line
    fn parse_typescript_line(&self, line: &str) -> Option<ProblemItem> {
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
                    if let (Ok(line), Ok(column)) =
                        (coords[0].parse::<u32>(), coords[1].parse::<u32>())
                    {
                        let full_path = self.workspace_path.join(file_path);
                        return Some(
                            ProblemItem::new(
                                ProblemSeverity::Error,
                                ProblemSource::TypeScript,
                                message,
                            )
                            .with_location(full_path, line, column),
                        );
                    }
                }
            }
        }

        None
    }

    /// Parse ESLint output
    async fn parse_eslint_output(&self, stdout: &[u8]) -> Result<Vec<ProblemItem>> {
        let mut problems = Vec::new();
        let stdout_str = String::from_utf8_lossy(stdout);

        if let Ok(eslint_results) = serde_json::from_str::<Value>(&stdout_str) {
            if let Some(files) = eslint_results.as_array() {
                for file_result in files {
                    if let Some(messages) = file_result.get("messages").and_then(|m| m.as_array()) {
                        let file_path = file_result
                            .get("filePath")
                            .and_then(|p| p.as_str())
                            .map(PathBuf::from)
                            .unwrap_or_default();

                        for message in messages {
                            if let Some(problem) = self.parse_eslint_message(&file_path, message) {
                                problems.push(problem);
                            }
                        }
                    }
                }
            }
        }

        Ok(problems)
    }

    /// Parse single ESLint message
    fn parse_eslint_message(&self, file_path: &PathBuf, message: &Value) -> Option<ProblemItem> {
        let severity = match message.get("severity")?.as_u64()? {
            1 => ProblemSeverity::Warning,
            2 => ProblemSeverity::Error,
            _ => ProblemSeverity::Info,
        };

        let message_text = message.get("message")?.as_str()?.to_string();
        let line = message.get("line")?.as_u64()? as u32;
        let column = message.get("column")?.as_u64()? as u32;
        let rule_id = message
            .get("ruleId")
            .and_then(|r| r.as_str())
            .map(|r| r.to_string());

        let mut problem = ProblemItem::new(severity, ProblemSource::ESLint, message_text)
            .with_location(file_path.clone(), line, column);

        if let Some(rule) = rule_id {
            problem = problem.with_code(rule);
        }

        Some(problem)
    }

    /// Parse NPM output
    async fn parse_npm_output(&self, stderr: &[u8]) -> Result<Vec<ProblemItem>> {
        let mut problems = Vec::new();
        let stderr_str = String::from_utf8_lossy(stderr);

        // Simple NPM error parsing - can be enhanced
        for line in stderr_str.lines() {
            if line.contains("ERROR") || line.contains("FAILED") {
                let problem = ProblemItem::new(
                    ProblemSeverity::Error,
                    ProblemSource::Other("npm".to_string()),
                    line.to_string(),
                );
                problems.push(problem);
            } else if line.contains("WARNING") || line.contains("WARN") {
                let problem = ProblemItem::new(
                    ProblemSeverity::Warning,
                    ProblemSource::Other("npm".to_string()),
                    line.to_string(),
                );
                problems.push(problem);
            }
        }

        Ok(problems)
    }

    /// Update problems state with new problems from a tool
    async fn update_problems_state(
        &mut self,
        tool: &BuildTool,
        problems: Vec<ProblemItem>,
    ) -> Result<()> {
        // Clear previous problems from this tool
        let source = self.tool_to_problem_source(tool);
        self.problems_state.clear_source(&source);

        // Add new problems
        for problem in problems {
            self.problems_state.add_problem(problem);
        }

        debug!(
            "Updated problems state for {:?}: {} problems",
            tool,
            self.problems_state
                .problems
                .get(&source)
                .map(|p| p.len())
                .unwrap_or(0)
        );

        Ok(())
    }

    /// Convert build tool to problem source
    fn tool_to_problem_source(&self, tool: &BuildTool) -> ProblemSource {
        match tool {
            BuildTool::Cargo => ProblemSource::Rust,
            BuildTool::Clippy => ProblemSource::Clippy,
            BuildTool::RustFmt => ProblemSource::Other("rustfmt".to_string()),
            BuildTool::Npm => ProblemSource::Other("npm".to_string()),
            BuildTool::TypeScript => ProblemSource::TypeScript,
            BuildTool::ESLint => ProblemSource::ESLint,
        }
    }

    /// Get current problems state
    pub fn get_problems_state(&self) -> &ProblemsState {
        &self.problems_state
    }

    /// Get mutable reference to problems state
    pub fn get_problems_state_mut(&mut self) -> &mut ProblemsState {
        &mut self.problems_state
    }

    /// Trigger build check for all active tools
    pub async fn refresh_all(&mut self) -> Result<()> {
        info!("🔄 Refreshing all build monitors...");

        let tools: Vec<BuildTool> = self.active_monitors.keys().cloned().collect();

        for tool in tools {
            if let Err(e) = self.run_build_check(&tool).await {
                error!("Failed to refresh {:?}: {}", tool, e);
            }
        }

        Ok(())
    }

    /// Get build statistics
    pub fn get_build_stats(&self) -> BuildStats {
        let mut stats = BuildStats::default();

        for (tool, monitor) in &self.active_monitors {
            stats.total_monitors += 1;
            stats.total_problems += monitor.current_problems.len();

            for problem in &monitor.current_problems {
                match problem.severity {
                    ProblemSeverity::Error => stats.total_errors += 1,
                    ProblemSeverity::Warning => stats.total_warnings += 1,
                    ProblemSeverity::Info => stats.total_info += 1,
                    ProblemSeverity::Hint => stats.total_hints += 1,
                }
            }

            if monitor.last_run.is_some() {
                stats.active_monitors += 1;
            }
        }

        stats
    }
}

/// Build statistics
#[derive(Debug, Default)]
pub struct BuildStats {
    pub total_monitors: usize,
    pub active_monitors: usize,
    pub total_problems: usize,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub total_info: usize,
    pub total_hints: usize,
}

impl BuildTool {
    /// Get display name for the build tool
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Cargo => "Cargo",
            Self::Npm => "NPM",
            Self::Clippy => "Clippy",
            Self::RustFmt => "RustFmt",
            Self::TypeScript => "TypeScript",
            Self::ESLint => "ESLint",
        }
    }

    /// Get icon for the build tool
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Cargo => "📦",
            Self::Npm => "📦",
            Self::Clippy => "📎",
            Self::RustFmt => "🎨",
            Self::TypeScript => "🔷",
            Self::ESLint => "🔍",
        }
    }
}
