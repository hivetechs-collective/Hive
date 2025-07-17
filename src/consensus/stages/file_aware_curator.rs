//! File-Aware Curator Stage
//!
//! Enhanced curator stage that can read and modify files based on consensus decisions.
//! This gives the curator stage Claude Code-like capabilities for file operations.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use tokio::fs;

use crate::consensus::file_operations::{FileReader, SecurityPolicy, FileContent};
use crate::consensus::repository_context::RepositoryContext;
use crate::consensus::stages::ConsensusStage;
use crate::consensus::stages::repository_scanner::{RepositoryScanner, FileInfo, FilePriority};
use crate::consensus::types::{Message, Stage};


/// File modification operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperation {
    /// Create a new file
    Create {
        path: PathBuf,
        content: String,
    },
    /// Update an existing file
    Update {
        path: PathBuf,
        content: String,
    },
    /// Append to an existing file
    Append {
        path: PathBuf,
        content: String,
    },
    /// Delete a file
    Delete {
        path: PathBuf,
    },
    /// Rename/move a file
    Rename {
        from: PathBuf,
        to: PathBuf,
    },
}

/// Result of file operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationResult {
    pub operation: FileOperation,
    pub success: bool,
    pub message: String,
    pub backup_path: Option<PathBuf>,
}

/// File modification proposal from curator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModificationProposal {
    pub operations: Vec<FileOperation>,
    pub rationale: String,
    pub safety_analysis: String,
    pub preview: String,
}

/// File writer with safety mechanisms
pub struct FileWriter {
    file_reader: Arc<FileReader>,
    security_policy: SecurityPolicy,
    backup_enabled: bool,
    dry_run: bool,
}

impl FileWriter {
    pub fn new(security_policy: SecurityPolicy) -> Self {
        Self {
            file_reader: Arc::new(FileReader::new(security_policy.clone())),
            security_policy,
            backup_enabled: true,
            dry_run: false,
        }
    }

    /// Set dry run mode (no actual file modifications)
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Execute a file operation with safety checks
    pub async fn execute_operation(&self, operation: &FileOperation) -> Result<FileOperationResult> {
        match operation {
            FileOperation::Create { path, content } => {
                self.create_file(path, content).await
            }
            FileOperation::Update { path, content } => {
                self.update_file(path, content).await
            }
            FileOperation::Append { path, content } => {
                self.append_file(path, content).await
            }
            FileOperation::Delete { path } => {
                self.delete_file(path).await
            }
            FileOperation::Rename { from, to } => {
                self.rename_file(from, to).await
            }
        }
    }

    /// Create a new file
    async fn create_file(&self, path: &Path, content: &str) -> Result<FileOperationResult> {
        // Security checks
        self.verify_write_allowed(path)?;

        if self.file_reader.path_exists(path).await? {
            return Ok(FileOperationResult {
                operation: FileOperation::Create {
                    path: path.to_path_buf(),
                    content: content.to_string(),
                },
                success: false,
                message: format!("File {} already exists", path.display()),
                backup_path: None,
            });
        }

        if self.dry_run {
            return Ok(FileOperationResult {
                operation: FileOperation::Create {
                    path: path.to_path_buf(),
                    content: content.to_string(),
                },
                success: true,
                message: format!("DRY RUN: Would create file {}", path.display()),
                backup_path: None,
            });
        }

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write the file
        fs::write(path, content).await?;

        Ok(FileOperationResult {
            operation: FileOperation::Create {
                path: path.to_path_buf(),
                content: content.to_string(),
            },
            success: true,
            message: format!("Successfully created file {}", path.display()),
            backup_path: None,
        })
    }

    /// Update an existing file
    async fn update_file(&self, path: &Path, content: &str) -> Result<FileOperationResult> {
        // Security checks
        self.verify_write_allowed(path)?;

        if !self.file_reader.path_exists(path).await? {
            return Ok(FileOperationResult {
                operation: FileOperation::Update {
                    path: path.to_path_buf(),
                    content: content.to_string(),
                },
                success: false,
                message: format!("File {} does not exist", path.display()),
                backup_path: None,
            });
        }

        let backup_path = if self.backup_enabled && !self.dry_run {
            Some(self.create_backup(path).await?)
        } else {
            None
        };

        if self.dry_run {
            return Ok(FileOperationResult {
                operation: FileOperation::Update {
                    path: path.to_path_buf(),
                    content: content.to_string(),
                },
                success: true,
                message: format!("DRY RUN: Would update file {}", path.display()),
                backup_path,
            });
        }

        // Write the updated content
        fs::write(path, content).await?;

        Ok(FileOperationResult {
            operation: FileOperation::Update {
                path: path.to_path_buf(),
                content: content.to_string(),
            },
            success: true,
            message: format!("Successfully updated file {}", path.display()),
            backup_path,
        })
    }

    /// Append to an existing file
    async fn append_file(&self, path: &Path, content: &str) -> Result<FileOperationResult> {
        // Security checks
        self.verify_write_allowed(path)?;

        if !self.file_reader.path_exists(path).await? {
            return Ok(FileOperationResult {
                operation: FileOperation::Append {
                    path: path.to_path_buf(),
                    content: content.to_string(),
                },
                success: false,
                message: format!("File {} does not exist", path.display()),
                backup_path: None,
            });
        }

        let backup_path = if self.backup_enabled && !self.dry_run {
            Some(self.create_backup(path).await?)
        } else {
            None
        };

        if self.dry_run {
            return Ok(FileOperationResult {
                operation: FileOperation::Append {
                    path: path.to_path_buf(),
                    content: content.to_string(),
                },
                success: true,
                message: format!("DRY RUN: Would append to file {}", path.display()),
                backup_path,
            });
        }

        // Read existing content and append
        let existing_content = fs::read_to_string(path).await?;
        let new_content = format!("{}{}", existing_content, content);
        fs::write(path, new_content).await?;

        Ok(FileOperationResult {
            operation: FileOperation::Append {
                path: path.to_path_buf(),
                content: content.to_string(),
            },
            success: true,
            message: format!("Successfully appended to file {}", path.display()),
            backup_path,
        })
    }

    /// Delete a file
    async fn delete_file(&self, path: &Path) -> Result<FileOperationResult> {
        // Security checks
        self.verify_write_allowed(path)?;

        if !self.file_reader.path_exists(path).await? {
            return Ok(FileOperationResult {
                operation: FileOperation::Delete {
                    path: path.to_path_buf(),
                },
                success: false,
                message: format!("File {} does not exist", path.display()),
                backup_path: None,
            });
        }

        let backup_path = if self.backup_enabled && !self.dry_run {
            Some(self.create_backup(path).await?)
        } else {
            None
        };

        if self.dry_run {
            return Ok(FileOperationResult {
                operation: FileOperation::Delete {
                    path: path.to_path_buf(),
                },
                success: true,
                message: format!("DRY RUN: Would delete file {}", path.display()),
                backup_path,
            });
        }

        // Delete the file
        fs::remove_file(path).await?;

        Ok(FileOperationResult {
            operation: FileOperation::Delete {
                path: path.to_path_buf(),
            },
            success: true,
            message: format!("Successfully deleted file {}", path.display()),
            backup_path,
        })
    }

    /// Rename/move a file
    async fn rename_file(&self, from: &Path, to: &Path) -> Result<FileOperationResult> {
        // Security checks
        self.verify_write_allowed(from)?;
        self.verify_write_allowed(to)?;

        if !self.file_reader.path_exists(from).await? {
            return Ok(FileOperationResult {
                operation: FileOperation::Rename {
                    from: from.to_path_buf(),
                    to: to.to_path_buf(),
                },
                success: false,
                message: format!("Source file {} does not exist", from.display()),
                backup_path: None,
            });
        }

        if self.file_reader.path_exists(to).await? {
            return Ok(FileOperationResult {
                operation: FileOperation::Rename {
                    from: from.to_path_buf(),
                    to: to.to_path_buf(),
                },
                success: false,
                message: format!("Destination file {} already exists", to.display()),
                backup_path: None,
            });
        }

        if self.dry_run {
            return Ok(FileOperationResult {
                operation: FileOperation::Rename {
                    from: from.to_path_buf(),
                    to: to.to_path_buf(),
                },
                success: true,
                message: format!("DRY RUN: Would rename {} to {}", from.display(), to.display()),
                backup_path: None,
            });
        }

        // Create parent directories if needed
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Rename the file
        fs::rename(from, to).await?;

        Ok(FileOperationResult {
            operation: FileOperation::Rename {
                from: from.to_path_buf(),
                to: to.to_path_buf(),
            },
            success: true,
            message: format!("Successfully renamed {} to {}", from.display(), to.display()),
            backup_path: None,
        })
    }

    /// Create a backup of a file
    async fn create_backup(&self, path: &Path) -> Result<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_path = path.with_extension(format!("{}.backup.{}", 
            path.extension().and_then(|s| s.to_str()).unwrap_or(""), timestamp));
        
        fs::copy(path, &backup_path).await?;
        Ok(backup_path)
    }

    /// Verify that writing to a path is allowed
    fn verify_write_allowed(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();

        // Check for denied patterns
        for denied in &self.security_policy.denied_patterns {
            if path_str.contains(denied) {
                anyhow::bail!("Write to path {} contains denied pattern: {}", path_str, denied);
            }
        }

        // Additional write-specific restrictions
        let write_denied_patterns = vec![
            "/etc/", "/usr/", "/var/", "/sys/", "/proc/",
            "node_modules/", ".git/", "target/", "dist/", "build/",
        ];

        for denied in &write_denied_patterns {
            if path_str.contains(denied) {
                anyhow::bail!("Write to system or build directory not allowed: {}", path_str);
            }
        }

        Ok(())
    }
}

/// File-aware curator that can read and modify files
pub struct FileAwareCuratorStage {
    file_reader: Arc<FileReader>,
    file_writer: Arc<FileWriter>,
}

impl FileAwareCuratorStage {
    pub fn new() -> Self {
        let security_policy = SecurityPolicy::default();
        Self {
            file_reader: Arc::new(FileReader::new(security_policy.clone())),
            file_writer: Arc::new(FileWriter::new(security_policy)),
        }
    }

    /// Read repository files for context with comprehensive directory scanning
    async fn read_repository_context(&self, repo_context: &RepositoryContext) -> Result<String> {
        let mut context = String::new();
        
        if let Some(root_path) = &repo_context.root_path {
            context.push_str("# CURRENT REPOSITORY CONTEXT\n\n");
            context.push_str(&format!("Repository: {}\n\n", root_path.display()));

            // First, scan and categorize all files in the repository
            let discovered_files = RepositoryScanner::scan_repository_files(root_path).await?;
            
            context.push_str(&format!("## Repository Structure ({} files)\n\n", discovered_files.len()));

            // Read files in priority order for maximum context value
            let prioritized_files = RepositoryScanner::prioritize_files_for_reading(&discovered_files);
            let mut files_read = 0;
            let max_files = 25; // Limit to prevent context overflow
            let max_context_chars = 50000; // ~12k tokens
            
            for file_info in prioritized_files.iter().take(max_files) {
                if context.len() > max_context_chars {
                    context.push_str(&format!("\n... ({} more files truncated for context limits)\n\n", 
                        prioritized_files.len() - files_read));
                    break;
                }
                
                match self.file_reader.read_file(&file_info.path).await {
                    Ok(content) => {
                        let relative_path = file_info.path.strip_prefix(root_path)
                            .unwrap_or(&file_info.path);
                        
                        context.push_str(&format!("## File: {}\n", relative_path.display()));
                        context.push_str(&format!("```{}\n", content.language.as_deref().unwrap_or("")));
                        
                        // Smart content summarization based on file size
                        let lines: Vec<&str> = content.content.lines().collect();
                        if lines.len() <= 100 {
                            // Small files: show complete content
                            context.push_str(&content.content);
                        } else if file_info.priority == FilePriority::Critical {
                            // Critical files: show more content
                            context.push_str(&lines[..150.min(lines.len())].join("\n"));
                            if lines.len() > 150 {
                                context.push_str(&format!("\n... ({} more lines)", lines.len() - 150));
                            }
                        } else {
                            // Other files: show first 75 lines
                            context.push_str(&lines[..75.min(lines.len())].join("\n"));
                            if lines.len() > 75 {
                                context.push_str(&format!("\n... ({} more lines)", lines.len() - 75));
                            }
                        }
                        
                        context.push_str("\n```\n\n");
                        files_read += 1;
                    }
                    Err(e) => {
                        warn!("Failed to read {}: {}", file_info.path.display(), e);
                    }
                }
            }
            
            // Add summary of repository structure
            RepositoryScanner::add_repository_summary(&mut context, &discovered_files);
        }

        Ok(context)
    }

    /// Parse file modification proposals from curator response
    pub fn parse_file_operations(&self, response: &str) -> Vec<FileOperation> {
        let mut operations = Vec::new();
        
        // Look for file operation blocks in the response
        // This is a simple parser - in practice, you might want a more sophisticated approach
        
        // Look for CREATE operations
        if let Some(create_match) = self.extract_operation_block(response, "CREATE") {
            if let Some(op) = self.parse_create_operation(&create_match) {
                operations.push(op);
            }
        }
        
        // Look for UPDATE operations
        if let Some(update_match) = self.extract_operation_block(response, "UPDATE") {
            if let Some(op) = self.parse_update_operation(&update_match) {
                operations.push(op);
            }
        }
        
        // Add more operation types as needed
        
        operations
    }

    fn extract_operation_block(&self, text: &str, operation_type: &str) -> Option<String> {
        // Simple pattern matching - look for blocks like:
        // ```CREATE:path/to/file
        // content here
        // ```
        
        let start_pattern = format!("```{}:", operation_type);
        if let Some(start) = text.find(&start_pattern) {
            if let Some(end) = text[start..].find("```") {
                if end > start_pattern.len() {
                    return Some(text[start..start + end].to_string());
                }
            }
        }
        
        None
    }

    fn parse_create_operation(&self, block: &str) -> Option<FileOperation> {
        // Parse CREATE:path format
        let lines: Vec<&str> = block.lines().collect();
        if lines.is_empty() {
            return None;
        }
        
        let first_line = lines[0];
        if let Some(path_start) = first_line.find("CREATE:") {
            let path_str = &first_line[path_start + 7..].trim();
            let content = lines[1..].join("\n");
            
            return Some(FileOperation::Create {
                path: PathBuf::from(path_str),
                content,
            });
        }
        
        None
    }

    fn parse_update_operation(&self, block: &str) -> Option<FileOperation> {
        // Parse UPDATE:path format
        let lines: Vec<&str> = block.lines().collect();
        if lines.is_empty() {
            return None;
        }
        
        let first_line = lines[0];
        if let Some(path_start) = first_line.find("UPDATE:") {
            let path_str = &first_line[path_start + 7..].trim();
            let content = lines[1..].join("\n");
            
            return Some(FileOperation::Update {
                path: PathBuf::from(path_str),
                content,
            });
        }
        
        None
    }
}

impl ConsensusStage for FileAwareCuratorStage {
    fn stage(&self) -> Stage {
        Stage::Curator
    }

    fn system_prompt(&self) -> &'static str {
        r#"You are a file-aware curator with the ability to read and modify files in the current repository.

Your role is to:
1. Review and improve the consensus response
2. Read relevant files to understand the current codebase
3. Propose specific file modifications when appropriate
4. Ensure all suggestions are based on actual file contents

When proposing file modifications, use this format:

```CREATE:path/to/new/file.rs
// New file content here
```

```UPDATE:path/to/existing/file.rs
// Updated file content here
```

```APPEND:path/to/file.rs
// Content to append
```

Always provide rationale for any file modifications and ensure they align with the project's existing patterns and conventions."#
    }

    fn build_messages(
        &self,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<Message>> {
        let validated_response = previous_answer.unwrap_or("No response to curate");

        let mut messages = vec![];

        // System prompt
        messages.push(Message {
            role: "system".to_string(),
            content: self.system_prompt().to_string(),
        });

        // Add repository context if available
        if let Some(ctx) = context {
            messages.push(Message {
                role: "system".to_string(),
                content: format!("Repository Context:\n{}", ctx),
            });
        }

        // Add the validated response to curate
        messages.push(Message {
            role: "user".to_string(),
            content: format!(
                "Original Question: {}\n\nValidated Response to Curate:\n{}\n\nPlease improve this response and propose any necessary file modifications using the specified format.",
                question,
                validated_response
            ),
        });

        Ok(messages)
    }
}

/// Build file-aware curator messages with repository context
pub async fn build_file_aware_curator_messages(
    stage: &FileAwareCuratorStage,
    question: &str,
    previous_answer: Option<&str>,
    repo_context: Option<&RepositoryContext>,
    base_context: Option<&str>,
) -> Result<Vec<Message>> {
    let mut messages = vec![];

    // System prompt
    messages.push(Message {
        role: "system".to_string(),
        content: stage.system_prompt().to_string(),
    });

    // Add base context if available
    if let Some(ctx) = base_context {
        messages.push(Message {
            role: "system".to_string(),
            content: format!("Context:\n{}", ctx),
        });
    }

    // Add repository context
    if let Some(repo_ctx) = repo_context {
        let repo_context_str = stage.read_repository_context(repo_ctx).await?;
        if !repo_context_str.is_empty() {
            messages.push(Message {
                role: "system".to_string(),
                content: repo_context_str,
            });
        }
    }

    // Add the validated response to curate
    let validated_response = previous_answer.unwrap_or("No response to curate");
    messages.push(Message {
        role: "user".to_string(),
        content: format!(
            "Original Question: {}\n\nValidated Response to Curate:\n{}\n\nIMPORTANT: Base your curation on the ACTUAL REPOSITORY FILES provided above. If you propose file modifications, use the specified format and ensure they align with the existing codebase patterns.",
            question,
            validated_response
        ),
    });

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_writer_security() {
        let writer = FileWriter::new(SecurityPolicy::default());
        
        // Test denied patterns
        assert!(writer.verify_write_allowed(Path::new(".git/config")).is_err());
        assert!(writer.verify_write_allowed(Path::new("node_modules/test.js")).is_err());
        assert!(writer.verify_write_allowed(Path::new("/etc/passwd")).is_err());
        
        // Test allowed patterns
        assert!(writer.verify_write_allowed(Path::new("src/main.rs")).is_ok());
        assert!(writer.verify_write_allowed(Path::new("test.txt")).is_ok());
    }

    #[test]
    fn test_operation_parsing() {
        let curator = FileAwareCuratorStage::new();
        
        let response = r#"
I suggest creating a new test file:

```CREATE:tests/new_test.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_new_feature() {
        assert!(true);
    }
}
```

And updating the main file:

```UPDATE:src/main.rs
fn main() {
    println!("Hello, updated world!");
}
```
        "#;
        
        let operations = curator.parse_file_operations(response);
        assert_eq!(operations.len(), 2);
        
        match &operations[0] {
            FileOperation::Create { path, content } => {
                assert_eq!(path.to_string_lossy(), "tests/new_test.rs");
                assert!(content.contains("test_new_feature"));
            }
            _ => panic!("Expected CREATE operation"),
        }
        
        match &operations[1] {
            FileOperation::Update { path, content } => {
                assert_eq!(path.to_string_lossy(), "src/main.rs");
                assert!(content.contains("updated world"));
            }
            _ => panic!("Expected UPDATE operation"),
        }
    }
}