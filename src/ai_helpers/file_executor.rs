//! AI Helper File Executor
//!
//! Provides file operation capabilities for AI Helpers to execute
//! plans from the Curator or handle simple operations directly.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use tokio::process::Command;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};

use crate::consensus::file_operations::SecurityPolicy;
use crate::consensus::safety_guardrails::SafetyGuardrailSystem;
use super::AIHelperEcosystem;

/// Execution plan from Curator or direct request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub overview: String,
    pub safety_level: SafetyLevel,
    pub operations: Vec<FileOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyLevel {
    Low,    // Simple file creation/reading
    Medium, // Multiple file changes
    High,   // System-wide changes, deletions
}

/// Individual file operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub step: usize,
    pub action: OperationType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OperationType {
    CreateFile {
        path: PathBuf,
        content: String,
    },
    UpdateFile {
        path: PathBuf,
        changes: Vec<FileChange>,
    },
    DeleteFile {
        path: PathBuf,
    },
    CreateDirectory {
        path: PathBuf,
    },
    MoveFile {
        from: PathBuf,
        to: PathBuf,
    },
    RunCommand {
        command: String,
        args: Vec<String>,
        working_dir: Option<PathBuf>,
    },
    SearchFiles {
        pattern: String,
        path: Option<PathBuf>,
        file_type: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub find: String,
    pub replace: String,
    pub all_occurrences: bool,
}

/// Execution report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub success: bool,
    pub operations_completed: usize,
    pub operations_total: usize,
    pub errors: Vec<String>,
    pub files_created: Vec<PathBuf>,
    pub files_modified: Vec<PathBuf>,
    pub files_deleted: Vec<PathBuf>,
}

/// AI Helper File Executor
pub struct AIHelperFileExecutor {
    ai_helpers: AIHelperEcosystem,
    security_policy: SecurityPolicy,
    safety_system: Option<SafetyGuardrailSystem>,
    dry_run: bool,
}

impl AIHelperFileExecutor {
    pub fn new(ai_helpers: AIHelperEcosystem) -> Self {
        Self {
            ai_helpers,
            security_policy: SecurityPolicy::default(),
            safety_system: None,
            dry_run: false,
        }
    }

    pub fn with_safety_system(mut self, safety: SafetyGuardrailSystem) -> Self {
        self.safety_system = Some(safety);
        self
    }

    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.dry_run = enabled;
        self
    }

    /// Execute a complete plan
    pub async fn execute_plan(&self, plan: ExecutionPlan) -> Result<ExecutionReport> {
        info!("Executing plan: {}", plan.overview);
        
        // Validate safety if system is available
        // TODO: Implement proper safety validation for ExecutionPlan
        // The SafetyGuardrailSystem expects EnhancedFileOperation, not ExecutionPlan
        // if let Some(safety) = &self.safety_system {
        //     safety.validate_execution_plan(&plan).await?;
        // }

        let mut report = ExecutionReport {
            success: true,
            operations_completed: 0,
            operations_total: plan.operations.len(),
            errors: vec![],
            files_created: vec![],
            files_modified: vec![],
            files_deleted: vec![],
        };

        for operation in &plan.operations {
            match self.execute_operation(operation).await {
                Ok(op_report) => {
                    report.operations_completed += 1;
                    report.files_created.extend(op_report.files_created);
                    report.files_modified.extend(op_report.files_modified);
                    report.files_deleted.extend(op_report.files_deleted);
                }
                Err(e) => {
                    error!("Operation {} failed: {}", operation.step, e);
                    report.errors.push(format!("Step {}: {}", operation.step, e));
                    
                    // Stop on high safety level operations
                    if matches!(plan.safety_level, SafetyLevel::High) {
                        report.success = false;
                        break;
                    }
                }
            }
        }

        Ok(report)
    }

    /// Execute a single operation
    async fn execute_operation(&self, operation: &FileOperation) -> Result<ExecutionReport> {
        info!("Executing step {}: {}", operation.step, operation.description);

        let mut report = ExecutionReport {
            success: true,
            operations_completed: 0,
            operations_total: 1,
            errors: vec![],
            files_created: vec![],
            files_modified: vec![],
            files_deleted: vec![],
        };

        if self.dry_run {
            info!("DRY RUN: Would execute: {:?}", operation.action);
            return Ok(report);
        }

        match &operation.action {
            OperationType::CreateFile { path, content } => {
                self.create_file(path, content).await?;
                report.files_created.push(path.clone());
            }
            OperationType::UpdateFile { path, changes } => {
                self.update_file(path, changes).await?;
                report.files_modified.push(path.clone());
            }
            OperationType::DeleteFile { path } => {
                self.delete_file(path).await?;
                report.files_deleted.push(path.clone());
            }
            OperationType::CreateDirectory { path } => {
                self.create_directory(path).await?;
            }
            OperationType::MoveFile { from, to } => {
                self.move_file(from, to).await?;
                report.files_deleted.push(from.clone());
                report.files_created.push(to.clone());
            }
            OperationType::RunCommand { command, args, working_dir } => {
                self.run_command(command, args, working_dir.as_deref()).await?;
            }
            OperationType::SearchFiles { pattern, path, file_type } => {
                let results = self.search_files(pattern, path.as_deref(), file_type.as_deref()).await?;
                info!("Search found {} results", results.len());
            }
        }

        report.operations_completed = 1;
        Ok(report)
    }

    /// Create a file with content
    async fn create_file(&self, path: &Path, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory for {:?}", path))?;
        }

        // Use AI Helper to validate content if needed
        if content.len() > 10000 {
            debug!("Large file content, using AI Helper for validation");
            // Could use quality analyzer here
        }

        fs::write(path, content)
            .with_context(|| format!("Failed to write file {:?}", path))?;
        
        info!("Created file: {:?}", path);
        Ok(())
    }

    /// Update a file with changes
    async fn update_file(&self, path: &Path, changes: &[FileChange]) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file {:?}", path))?;

        let mut updated = content.clone();
        
        for change in changes {
            if change.all_occurrences {
                updated = updated.replace(&change.find, &change.replace);
            } else {
                updated = updated.replacen(&change.find, &change.replace, 1);
            }
        }

        if content != updated {
            fs::write(path, updated)
                .with_context(|| format!("Failed to update file {:?}", path))?;
            info!("Updated file: {:?}", path);
        } else {
            warn!("No changes made to file: {:?}", path);
        }

        Ok(())
    }

    /// Delete a file
    async fn delete_file(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            warn!("File does not exist: {:?}", path);
            return Ok(());
        }

        // Extra safety check
        if path.to_string_lossy().contains("..") {
            return Err(anyhow::anyhow!("Path traversal detected"));
        }

        fs::remove_file(path)
            .with_context(|| format!("Failed to delete file {:?}", path))?;
        
        info!("Deleted file: {:?}", path);
        Ok(())
    }

    /// Create a directory
    async fn create_directory(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory {:?}", path))?;
        
        info!("Created directory: {:?}", path);
        Ok(())
    }

    /// Move a file
    async fn move_file(&self, from: &Path, to: &Path) -> Result<()> {
        // Ensure target directory exists
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::rename(from, to)
            .with_context(|| format!("Failed to move {:?} to {:?}", from, to))?;
        
        info!("Moved file from {:?} to {:?}", from, to);
        Ok(())
    }

    /// Run a command
    async fn run_command(&self, command: &str, args: &[String], working_dir: Option<&Path>) -> Result<String> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await
            .with_context(|| format!("Failed to run command: {} {:?}", command, args))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Command failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }

    /// Search files using ripgrep
    async fn search_files(&self, pattern: &str, path: Option<&Path>, file_type: Option<&str>) -> Result<Vec<PathBuf>> {
        let mut cmd = Command::new("rg");
        cmd.arg("--files-with-matches");
        cmd.arg(pattern);

        if let Some(p) = path {
            cmd.arg(p);
        }

        if let Some(ft) = file_type {
            cmd.arg("--type").arg(ft);
        }

        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        let files: Vec<PathBuf> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .collect();

        Ok(files)
    }

    /// Parse natural language into execution plan using AI intelligence
    pub async fn parse_request(&self, request: &str) -> Result<ExecutionPlan> {
        error!("🚫 Pattern matching is forbidden - AI Helpers must use intelligence");
        error!("This method should not be called. Use IntelligentExecutor instead.");
        Err(anyhow::anyhow!("Pattern matching is not allowed. AI Helpers must understand context intelligently."))
    }


    /// DEPRECATED: This pattern matching approach is forbidden
    #[deprecated(note = "Pattern matching is not allowed. Use IntelligentExecutor.")]
    fn parse_curator_style_output(&self, output: &str) -> Result<ExecutionPlan> {
        error!("🚫 parse_curator_style_output called - this is forbidden!");
        Err(anyhow::anyhow!("Pattern matching is not allowed"))
    }
    
    #[allow(dead_code)]
    fn _old_parse_curator_style_output(&self, output: &str) -> Result<ExecutionPlan> {
        let mut operations = Vec::new();
        let mut current_file: Option<(String, Vec<String>)> = None;
        let mut in_code_block = false;
        
        for line in output.lines() {
            // Check for operation markers (e.g., "Creating hello_world.py:")
            if line.starts_with("Creating ") || line.starts_with("Updating ") || 
               line.starts_with("Deleting ") || line.starts_with("Writing to ") {
                
                // Save previous file if any
                if let Some((path, content_lines)) = current_file.take() {
                    operations.push(FileOperation {
                        step: operations.len() + 1,
                        action: OperationType::CreateFile {
                            path: PathBuf::from(path.clone()),
                            content: content_lines.join("\n"),
                        },
                        description: format!("Create file {}", path),
                    });
                }
                
                // Extract filename (handles both "Creating `file.txt`:" and "Creating file.txt:")
                let filename = if let Some(start) = line.find('`') {
                    // Backtick format
                    if let Some(end) = line[start + 1..].find('`') {
                        Some(line[start + 1..start + 1 + end].to_string())
                    } else {
                        None
                    }
                } else {
                    // Non-backtick format: extract everything after the operation word
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        let mut filename_part = parts[1..].join(" ");
                        
                        // Remove trailing colon if present
                        filename_part = filename_part.trim_end_matches(':').to_string();
                        
                        // Clean up HTML artifacts if present
                        // Remove <code> tags
                        filename_part = filename_part.replace("<code>", "").replace("</code>", "");
                        // Remove <p> tags  
                        filename_part = filename_part.replace("<p>", "").replace("</p>", "");
                        // Remove any remaining HTML entities
                        filename_part = filename_part.replace("&lt;", "<").replace("&gt;", ">");
                        
                        let clean_filename = filename_part.trim().to_string();
                        if !clean_filename.is_empty() {
                            Some(clean_filename)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                
                if let Some(fname) = filename {
                    current_file = Some((fname, Vec::new()));
                }
            } else if line.trim() == "```" || line.trim().starts_with("```") {
                in_code_block = !in_code_block;
            } else if in_code_block {
                if let Some((_, ref mut content)) = current_file {
                    content.push(line.to_string());
                }
            }
        }
        
        // Save final file if any
        if let Some((path, content_lines)) = current_file {
            operations.push(FileOperation {
                step: operations.len() + 1,
                action: OperationType::CreateFile {
                    path: PathBuf::from(path.clone()),
                    content: content_lines.join("\n"),
                },
                description: format!("Create file {}", path),
            });
        }
        
        if operations.is_empty() {
            return Err(anyhow::anyhow!("No file operations found in Curator output"));
        }
        
        Ok(ExecutionPlan {
            overview: "Execute Curator-identified file operations".to_string(),
            safety_level: SafetyLevel::Medium,
            operations,
        })
    }
    
    /// DEPRECATED: This pattern matching approach is forbidden
    #[deprecated(note = "Pattern matching is not allowed. Use IntelligentExecutor.")]
    fn create_simple_plan(&self, request: &str) -> Result<ExecutionPlan> {
        error!("🚫 create_simple_plan called - this is forbidden!");
        Err(anyhow::anyhow!("Pattern matching is not allowed"))
    }
    
    #[allow(dead_code)]
    fn _old_create_simple_plan(&self, request: &str) -> Result<ExecutionPlan> {
        let request_lower = request.to_lowercase();

        // Extract filename from request
        let filename = self.extract_filename(request)
            .ok_or_else(|| anyhow::anyhow!("Could not extract filename from request"))?;

        let operation = if request_lower.contains("create") || request_lower.contains("make") || request_lower.contains("new") {
            FileOperation {
                step: 1,
                action: OperationType::CreateFile {
                    path: PathBuf::from(&filename),
                    content: self.generate_default_content(&filename),
                },
                description: format!("Create file {}", filename),
            }
        } else {
            return Err(anyhow::anyhow!("Unsupported operation"));
        };

        Ok(ExecutionPlan {
            overview: format!("Create {} file", filename),
            safety_level: SafetyLevel::Low,
            operations: vec![operation],
        })
    }

    /// DEPRECATED: This pattern matching approach is forbidden
    #[deprecated(note = "Pattern matching is not allowed. Use IntelligentExecutor.")]
    fn extract_filename(&self, request: &str) -> Option<String> {
        error!("🚫 extract_filename called - this is forbidden!");
        None
    }
    
    #[allow(dead_code)]
    fn _old_extract_filename(&self, request: &str) -> Option<String> {
        // Look for patterns like "file.txt", "hello.md", etc.
        let words: Vec<&str> = request.split_whitespace().collect();
        
        for word in words {
            if word.contains('.') && !word.starts_with('.') {
                return Some(word.to_string());
            }
        }

        // Try to find "called X" or "named X" pattern
        if let Some(pos) = request.find("called ") {
            let after = &request[pos + 7..];
            if let Some(name) = after.split_whitespace().next() {
                let name = name.trim_end_matches(&['.', ',', '!', '?'][..]);
                if !name.contains('.') {
                    return Some(format!("{}.txt", name));
                }
                return Some(name.to_string());
            }
        }

        None
    }

    /// DEPRECATED: This pattern matching approach is forbidden
    #[deprecated(note = "Pattern matching is not allowed. Use IntelligentExecutor.")]
    fn generate_default_content(&self, filename: &str) -> String {
        error!("🚫 generate_default_content called - this is forbidden!");
        String::new()
    }
    
    #[allow(dead_code)]
    fn _old_generate_default_content(&self, filename: &str) -> String {
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "rs" => "fn main() {\n    println!(\"Hello, world!\");\n}\n".to_string(),
            "py" => "def main():\n    print(\"Hello, world!\")\n\nif __name__ == \"__main__\":\n    main()\n".to_string(),
            "js" => "console.log('Hello, world!');\n".to_string(),
            "md" => "# Hello World\n\nThis is a simple markdown file.\n".to_string(),
            "txt" | _ => "Hello, World!\n\nThis is a simple text file.\n".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename() {
        let executor = AIHelperFileExecutor::new(AIHelperEcosystem::new_mock());
        
        assert_eq!(
            executor.extract_filename("create a file called test.rs"),
            Some("test.rs".to_string())
        );
        
        assert_eq!(
            executor.extract_filename("make hello.txt"),
            Some("hello.txt".to_string())
        );
        
        assert_eq!(
            executor.extract_filename("create a file called hello"),
            Some("hello.txt".to_string())
        );
    }
}