//! Navigation Handler for Problems Panel
//!
//! Handles click-to-navigate functionality for problems, integrating with
//! the code editor and file system to navigate to specific problem locations.

use super::problems_panel::ProblemItem;
use crate::desktop::events::{Event, EventHandler, EventType};
use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// Navigation handler for problems
pub struct ProblemNavigationHandler {
    workspace_path: PathBuf,
    editor_integration: Option<EditorIntegration>,
}

/// Editor integration types
#[derive(Debug, Clone)]
pub enum EditorIntegration {
    Internal,  // Use internal code editor
    VSCode,    // Open in VS Code
    Vim,       // Open in Vim
    Custom(String), // Custom command
}

/// Navigation result
#[derive(Debug)]
pub enum NavigationResult {
    Success {
        file_path: PathBuf,
        line: u32,
        column: u32,
    },
    FileNotFound(PathBuf),
    EditorError(String),
    InvalidLocation {
        file_path: PathBuf,
        line: u32,
        column: u32,
    },
}

impl ProblemNavigationHandler {
    /// Create new navigation handler
    pub fn new(workspace_path: PathBuf) -> Self {
        Self {
            workspace_path,
            editor_integration: Some(EditorIntegration::Internal),
        }
    }

    /// Set editor integration type
    pub fn set_editor_integration(&mut self, integration: EditorIntegration) {
        self.editor_integration = Some(integration);
        info!("📝 Editor integration set to: {:?}", integration);
    }

    /// Navigate to a problem location
    pub async fn navigate_to_problem(&self, problem: &ProblemItem) -> Result<NavigationResult> {
        debug!("🧭 Navigating to problem: {}", problem.message);

        // Extract location information
        let file_path = match &problem.file_path {
            Some(path) => path.clone(),
            None => {
                warn!("Problem has no file path: {}", problem.message);
                return Ok(NavigationResult::FileNotFound(PathBuf::new()));
            }
        };

        let line = problem.line.unwrap_or(1);
        let column = problem.column.unwrap_or(1);

        // Validate file exists
        if !file_path.exists() {
            warn!("File not found: {}", file_path.display());
            return Ok(NavigationResult::FileNotFound(file_path));
        }

        // Validate location is reasonable
        if let Err(e) = self.validate_location(&file_path, line, column).await {
            warn!("Invalid location {}:{}:{} - {}", file_path.display(), line, column, e);
            return Ok(NavigationResult::InvalidLocation { file_path, line, column });
        }

        // Perform navigation based on editor integration
        match self.perform_navigation(&file_path, line, column).await {
            Ok(()) => {
                info!("✅ Successfully navigated to {}:{}:{}", file_path.display(), line, column);
                Ok(NavigationResult::Success { file_path, line, column })
            }
            Err(e) => {
                error!("❌ Navigation failed: {}", e);
                Ok(NavigationResult::EditorError(e.to_string()))
            }
        }
    }

    /// Validate that a location is reasonable
    async fn validate_location(&self, file_path: &Path, line: u32, column: u32) -> Result<()> {
        // Basic validation - ensure line and column are reasonable
        if line == 0 || column == 0 {
            return Err(anyhow::anyhow!("Line and column must be >= 1"));
        }

        // For very large line numbers, we might want to validate against actual file length
        if line > 100000 {
            return Err(anyhow::anyhow!("Line number {} seems unreasonably large", line));
        }

        // Could add more sophisticated validation here:
        // - Check if line exists in file
        // - Validate column is within line bounds
        // - Check file permissions

        Ok(())
    }

    /// Perform the actual navigation
    async fn perform_navigation(&self, file_path: &Path, line: u32, column: u32) -> Result<()> {
        let integration = self.editor_integration.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No editor integration configured"))?;

        match integration {
            EditorIntegration::Internal => {
                self.navigate_internal_editor(file_path, line, column).await
            }
            EditorIntegration::VSCode => {
                self.navigate_vscode(file_path, line, column).await
            }
            EditorIntegration::Vim => {
                self.navigate_vim(file_path, line, column).await
            }
            EditorIntegration::Custom(command) => {
                self.navigate_custom(command, file_path, line, column).await
            }
        }
    }

    /// Navigate using internal editor
    async fn navigate_internal_editor(&self, file_path: &Path, line: u32, column: u32) -> Result<()> {
        debug!("📝 Opening in internal editor: {}:{}:{}", file_path.display(), line, column);

        // This would integrate with the internal code editor component
        // For now, we'll create the event structure

        let navigation_event = NavigationEvent {
            file_path: file_path.to_path_buf(),
            line,
            column,
            source: "problems_panel".to_string(),
        };

        // In a real implementation, this would dispatch an event to the code editor
        self.dispatch_navigation_event(navigation_event).await?;

        Ok(())
    }

    /// Navigate using VS Code
    async fn navigate_vscode(&self, file_path: &Path, line: u32, column: u32) -> Result<()> {
        debug!("📝 Opening in VS Code: {}:{}:{}", file_path.display(), line, column);

        let command = "code";
        let arg = format!("{}:{}:{}", file_path.display(), line, column);

        let output = tokio::process::Command::new(command)
            .arg("--goto")
            .arg(&arg)
            .current_dir(&self.workspace_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("VS Code command failed: {}", error));
        }

        Ok(())
    }

    /// Navigate using Vim
    async fn navigate_vim(&self, file_path: &Path, line: u32, column: u32) -> Result<()> {
        debug!("📝 Opening in Vim: {}:{}:{}", file_path.display(), line, column);

        let command = "vim";
        let position_arg = format!("+{}", line);
        let column_cmd = format!("normal {}|", column);

        let mut cmd = tokio::process::Command::new(command);
        cmd.arg(&position_arg)
           .arg("-c")
           .arg(&column_cmd)
           .arg(file_path)
           .current_dir(&self.workspace_path);

        // For terminal editors like Vim, we might want to handle this differently
        // depending on whether we're in a GUI or terminal environment
        let output = cmd.output().await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Vim command failed: {}", error));
        }

        Ok(())
    }

    /// Navigate using custom command
    async fn navigate_custom(&self, command_template: &str, file_path: &Path, line: u32, column: u32) -> Result<()> {
        debug!("📝 Opening with custom command: {} {}:{}:{}", command_template, file_path.display(), line, column);

        // Replace placeholders in command template
        let command = command_template
            .replace("{file}", &file_path.to_string_lossy())
            .replace("{line}", &line.to_string())
            .replace("{column}", &column.to_string())
            .replace("{workspace}", &self.workspace_path.to_string_lossy());

        // Parse command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty custom command"));
        }

        let (cmd, args) = parts.split_first().unwrap();

        let output = tokio::process::Command::new(cmd)
            .args(args)
            .current_dir(&self.workspace_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Custom command failed: {}", error));
        }

        Ok(())
    }

    /// Dispatch navigation event to the editor
    async fn dispatch_navigation_event(&self, event: NavigationEvent) -> Result<()> {
        // This would integrate with the event system to notify the code editor
        debug!("📡 Dispatching navigation event: {}:{}:{}", 
               event.file_path.display(), event.line, event.column);

        // In a real implementation, this would use the event bus:
        // let event = Event::navigation_request(event);
        // event_bus().publish_async(event).await?;

        Ok(())
    }

    /// Get navigation history
    pub fn get_navigation_history(&self) -> Vec<NavigationHistoryEntry> {
        // This would return the history of navigations
        // For now, return empty
        Vec::new()
    }

    /// Navigate back in history
    pub async fn navigate_back(&self) -> Result<Option<NavigationResult>> {
        // This would navigate to the previous location in history
        debug!("⬅️ Navigating back in history");
        Ok(None)
    }

    /// Navigate forward in history
    pub async fn navigate_forward(&self) -> Result<Option<NavigationResult>> {
        // This would navigate to the next location in history
        debug!("➡️ Navigating forward in history");
        Ok(None)
    }

    /// Quick navigation to next problem of same type
    pub async fn navigate_to_next_problem_of_type(&self, current_problem: &ProblemItem, problems: &[ProblemItem]) -> Result<Option<NavigationResult>> {
        debug!("⏭️ Navigating to next problem of type: {:?}", current_problem.severity);

        // Find next problem of same severity and source
        let current_index = problems.iter().position(|p| p.id == current_problem.id);
        
        if let Some(current_idx) = current_index {
            for (idx, problem) in problems.iter().enumerate().skip(current_idx + 1) {
                if problem.severity == current_problem.severity && problem.source == current_problem.source {
                    return Ok(Some(self.navigate_to_problem(problem).await?));
                }
            }
        }

        Ok(None)
    }

    /// Quick navigation to previous problem of same type
    pub async fn navigate_to_previous_problem_of_type(&self, current_problem: &ProblemItem, problems: &[ProblemItem]) -> Result<Option<NavigationResult>> {
        debug!("⏮️ Navigating to previous problem of type: {:?}", current_problem.severity);

        // Find previous problem of same severity and source
        let current_index = problems.iter().position(|p| p.id == current_problem.id);
        
        if let Some(current_idx) = current_index {
            for problem in problems.iter().take(current_idx).rev() {
                if problem.severity == current_problem.severity && problem.source == current_problem.source {
                    return Ok(Some(self.navigate_to_problem(problem).await?));
                }
            }
        }

        Ok(None)
    }

    /// Create keyboard shortcut handler for navigation
    pub fn create_keyboard_handler(&self) -> NavigationKeyboardHandler {
        NavigationKeyboardHandler::new()
    }
}

/// Navigation event for internal editor
#[derive(Debug, Clone)]
pub struct NavigationEvent {
    pub file_path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub source: String,
}

/// Navigation history entry
#[derive(Debug, Clone)]
pub struct NavigationHistoryEntry {
    pub file_path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub timestamp: std::time::Instant,
}

/// Keyboard handler for navigation shortcuts
pub struct NavigationKeyboardHandler {
    // This would handle keyboard shortcuts for navigation
}

impl NavigationKeyboardHandler {
    pub fn new() -> Self {
        Self {}
    }

    /// Handle keyboard shortcut
    pub async fn handle_shortcut(&self, key: &str) -> Result<Option<NavigationAction>> {
        match key {
            "F12" => Ok(Some(NavigationAction::GoToDefinition)),
            "Shift+F12" => Ok(Some(NavigationAction::FindReferences)),
            "Ctrl+G" => Ok(Some(NavigationAction::GoToLine)),
            "F8" => Ok(Some(NavigationAction::NextProblem)),
            "Shift+F8" => Ok(Some(NavigationAction::PreviousProblem)),
            "Alt+Left" => Ok(Some(NavigationAction::NavigateBack)),
            "Alt+Right" => Ok(Some(NavigationAction::NavigateForward)),
            _ => Ok(None),
        }
    }
}

/// Navigation actions
#[derive(Debug, Clone)]
pub enum NavigationAction {
    GoToDefinition,
    FindReferences,
    GoToLine,
    NextProblem,
    PreviousProblem,
    NavigateBack,
    NavigateForward,
}

impl Default for NavigationKeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for common navigation patterns
pub mod navigation_utils {
    use super::*;

    /// Extract relative path for display
    pub fn get_relative_path(workspace_path: &Path, file_path: &Path) -> PathBuf {
        file_path.strip_prefix(workspace_path)
            .unwrap_or(file_path)
            .to_path_buf()
    }

    /// Format location for display
    pub fn format_location(file_path: &Path, line: Option<u32>, column: Option<u32>) -> String {
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        match (line, column) {
            (Some(l), Some(c)) => format!("{}:{}:{}", file_name, l, c),
            (Some(l), None) => format!("{}:{}", file_name, l),
            _ => file_name.to_string(),
        }
    }

    /// Check if file is editable (basic heuristic)
    pub fn is_file_editable(file_path: &Path) -> bool {
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            matches!(extension.to_lowercase().as_str(), 
                "rs" | "js" | "ts" | "tsx" | "jsx" | "py" | "go" | "java" | "cpp" | "c" | "h" | 
                "css" | "scss" | "sass" | "html" | "xml" | "json" | "yaml" | "yml" | "toml" | 
                "md" | "txt" | "sh" | "bat" | "ps1")
        } else {
            false
        }
    }

    /// Get line preview from file (for hover/tooltip)
    pub async fn get_line_preview(file_path: &Path, line: u32, context_lines: usize) -> Result<Vec<String>> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let lines: Vec<&str> = content.lines().collect();
        
        let start_line = line.saturating_sub(context_lines as u32 + 1) as usize;
        let end_line = std::cmp::min((line + context_lines as u32) as usize, lines.len());
        
        let preview: Vec<String> = lines[start_line..end_line]
            .iter()
            .map(|s| s.to_string())
            .collect();
            
        Ok(preview)
    }
}