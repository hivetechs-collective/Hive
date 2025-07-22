//! File-Aware Curator Stage
//!
//! Enhanced curator stage that can READ files and CREATE PLANS for file operations.
//! IMPORTANT: This stage NEVER executes file operations - it only creates plans
//! that will be executed by AI Helpers.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::consensus::file_operations::{FileReader, SecurityPolicy, FileContent};
use crate::consensus::repository_context::RepositoryContext;
use crate::consensus::stages::ConsensusStage;
use crate::consensus::stages::repository_scanner::{RepositoryScanner, FileInfo, FilePriority};
use crate::consensus::types::{Message, Stage};
use crate::consensus::curator_output_format::{CuratorGuidelines, CuratorOutputFormat};

/// File modification operation (PLAN ONLY - NOT EXECUTED HERE)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Result of file operation (PLANNED, NOT EXECUTED)
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

/// File-aware curator that can read files and create operation plans
/// IMPORTANT: This stage ONLY creates plans, it NEVER executes file operations
pub struct FileAwareCuratorStage {
    file_reader: Arc<FileReader>,
    // Removed file_writer - Curator should NEVER execute operations
}

impl FileAwareCuratorStage {
    pub fn new() -> Self {
        let security_policy = SecurityPolicy::default();
        Self {
            file_reader: Arc::new(FileReader::new(security_policy)),
        }
    }

    /// Read repository files for context with comprehensive directory scanning
    async fn read_repository_context(&self, repo_context: &RepositoryContext) -> Result<String> {
        let mut context = String::new();
        
        // Use the repository scanner to get prioritized files
        let scan_result = if let Some(root) = &repo_context.root_path {
            RepositoryScanner::scan_repository_files(root).await?
        } else {
            RepositoryScanner::scan_repository_files(std::path::Path::new(".")).await?
        };
        
        // Add repository summary
        let total_files = scan_result.len();
        let total_size: u64 = scan_result.iter().map(|f| f.size).sum();
        let has_rust = scan_result.iter().any(|f| f.path.extension().map_or(false, |ext| ext == "rs"));
        let has_tests = scan_result.iter().any(|f| f.path.to_string_lossy().contains("test"));
        let has_docs = scan_result.iter().any(|f| f.path.extension().map_or(false, |ext| ext == "md"));
        
        context.push_str(&format!(
            "Repository Analysis:\n\
             - Total files: {}\n\
             - Total size: {} KB\n\
             - Main language: {:?}\n\
             - Has tests: {}\n\
             - Has documentation: {}\n\n",
            total_files,
            total_size / 1024,
            if has_rust { "Rust" } else { "Unknown" },
            has_tests,
            has_docs
        ));
        
        // Group files by priority
        let mut high_priority_files = Vec::new();
        let mut medium_priority_files = Vec::new();
        let mut low_priority_files = Vec::new();
        
        for file in &scan_result {
            match file.priority {
                FilePriority::Critical => high_priority_files.push(file),
                FilePriority::High => high_priority_files.push(file),
                FilePriority::Medium => medium_priority_files.push(file),
                FilePriority::Normal => medium_priority_files.push(file),
                FilePriority::Low => low_priority_files.push(file),
            }
        }
        
        // Read high priority files (up to 10)
        context.push_str("=== Key Files (High Priority) ===\n\n");
        for (idx, file_info) in high_priority_files.iter().take(10).enumerate() {
            if let Ok(content) = self.file_reader.read_file(&file_info.path).await {
                context.push_str(&format!(
                    "File {}: {} ({} lines)\n",
                    idx + 1,
                    file_info.path.display(),
                    content.lines
                ));
                
                // Include file summary or first few lines
                if content.lines <= 50 {
                    context.push_str(&format!("Content:\n{}\n\n", content.content));
                } else {
                    let preview: Vec<&str> = content.content.lines().take(20).collect();
                    context.push_str(&format!(
                        "Preview (first 20 lines):\n{}\n... ({} more lines)\n\n",
                        preview.join("\n"),
                        content.lines - 20
                    ));
                }
            }
        }
        
        // List medium priority files
        if !medium_priority_files.is_empty() {
            context.push_str("\n=== Project Structure (Medium Priority) ===\n");
            for file_info in medium_priority_files.iter().take(20) {
                context.push_str(&format!(
                    "- {} ({} bytes)\n",
                    file_info.path.display(),
                    file_info.size
                ));
            }
        }
        
        // Summary of low priority files
        if !low_priority_files.is_empty() {
            context.push_str(&format!(
                "\n=== Other Files ===\n{} additional files found (tests, docs, configs, etc.)\n",
                low_priority_files.len()
            ));
        }
        
        Ok(context)
    }

    /// Create a file operation plan based on consensus
    /// This ONLY creates a plan - execution happens in AI Helpers
    fn create_operation_plan(&self, 
        operation_type: &str, 
        path: &Path, 
        content: Option<&str>,
        target_path: Option<&Path>
    ) -> FileOperation {
        match operation_type {
            "create" => FileOperation::Create {
                path: path.to_path_buf(),
                content: content.unwrap_or("").to_string(),
            },
            "update" => FileOperation::Update {
                path: path.to_path_buf(),
                content: content.unwrap_or("").to_string(),
            },
            "append" => FileOperation::Append {
                path: path.to_path_buf(),
                content: content.unwrap_or("").to_string(),
            },
            "delete" => FileOperation::Delete {
                path: path.to_path_buf(),
            },
            "rename" => FileOperation::Rename {
                from: path.to_path_buf(),
                to: target_path.unwrap_or(path).to_path_buf(),
            },
            _ => FileOperation::Create {
                path: path.to_path_buf(),
                content: content.unwrap_or("").to_string(),
            }
        }
    }

    /// Analyze safety of proposed operations (planning only)
    fn analyze_operation_safety(&self, operations: &[FileOperation]) -> String {
        let mut analysis = String::from("Safety Analysis:\n");
        
        for op in operations {
            match op {
                FileOperation::Delete { path } => {
                    analysis.push_str(&format!(
                        "⚠️  DELETE operation on {} - High risk, ensure backup\n", 
                        path.display()
                    ));
                }
                FileOperation::Update { path, .. } => {
                    analysis.push_str(&format!(
                        "✓ UPDATE operation on {} - Medium risk\n", 
                        path.display()
                    ));
                }
                FileOperation::Create { path, .. } => {
                    analysis.push_str(&format!(
                        "✓ CREATE operation on {} - Low risk\n", 
                        path.display()
                    ));
                }
                _ => {}
            }
        }
        
        analysis
    }

    /// Create a preview of operations (for user review)
    fn create_operation_preview(&self, operations: &[FileOperation]) -> String {
        let mut preview = String::from("Planned Operations:\n");
        
        for (idx, op) in operations.iter().enumerate() {
            preview.push_str(&format!("{}. ", idx + 1));
            match op {
                FileOperation::Create { path, content } => {
                    preview.push_str(&format!(
                        "Create {} ({} bytes)\n", 
                        path.display(),
                        content.len()
                    ));
                }
                FileOperation::Update { path, content } => {
                    preview.push_str(&format!(
                        "Update {} ({} bytes)\n", 
                        path.display(),
                        content.len()
                    ));
                }
                FileOperation::Append { path, content } => {
                    preview.push_str(&format!(
                        "Append to {} ({} bytes)\n", 
                        path.display(),
                        content.len()
                    ));
                }
                FileOperation::Delete { path } => {
                    preview.push_str(&format!("Delete {}\n", path.display()));
                }
                FileOperation::Rename { from, to } => {
                    preview.push_str(&format!(
                        "Rename {} to {}\n", 
                        from.display(),
                        to.display()
                    ));
                }
            }
        }
        
        preview
    }
}

#[async_trait::async_trait]
impl ConsensusStage for FileAwareCuratorStage {
    fn stage(&self) -> Stage {
        Stage::Curator
    }


    fn system_prompt(&self) -> &'static str {
        "You are the Curator stage in the consensus pipeline. Your role is to synthesize insights and create file operation plans.

CRITICAL: When your response includes file operations, you MUST use the standardized format for reliable AI Helper parsing:

## File Operation Output Format (REQUIRED)

### For Creating Files:
### Step 1: Creating `path/to/file.ext`

```language:path/to/file.ext
file content here
```

✅ **Created**: path/to/file.ext

### For Updating Files:
### Step 1: Updating `path/to/file.ext`

```language:path/to/file.ext
updated content here
```

✅ **Updated**: path/to/file.ext

### For Deleting Files:
### Step 1: Deleting `path/to/file.ext`

```delete:path/to/file.ext
# File will be deleted
```

❌ **Deleted**: path/to/file.ext

### For Renaming Files:
### Step 1: Renaming `old/path.ext` to `new/path.ext`

```rename:old/path.ext to new/path.ext
# File will be renamed
```

🔄 **Renamed**: old/path.ext → new/path.ext

IMPORTANT: Always use this exact format. AI Helpers depend on this structure for reliable parsing and execution."
    }

    fn build_messages(
        &self,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<Message>> {
        info!("🎨 File-Aware Curator Stage - Creating operation plans (NOT executing)");

        let mut messages = vec![
            Message {
                role: "system".to_string(),
                content: self.system_prompt().to_string(),
            }
        ];

        // Add format validation instructions
        messages.push(Message {
            role: "system".to_string(),
            content: CuratorGuidelines::get_format_instructions().to_string(),
        });

        // Build enhanced context with repository information if available
        if let Some(repo_context) = context {
            let enhanced_context = format!(
                "=== Repository Context ===\n{}\n\n", 
                repo_context
            );
            
            messages.push(Message {
                role: "system".to_string(),
                content: enhanced_context,
            });
        }

        // Add the previous stages' analyses
        if let Some(prev_answer) = previous_answer {
            messages.push(Message {
                role: "system".to_string(),
                content: format!(
                    "=== Previous Analysis ===\nThe Generator, Refiner, and Validator stages have analyzed the question. Here is their combined analysis:\n\n{}\n\n=== Your Task ===\nAs the Curator, synthesize this analysis into a polished final response. If file operations are needed, use the EXACT format specified above.",
                    prev_answer
                ),
            });
        }

        // Add the user's question
        messages.push(Message {
            role: "user".to_string(),
            content: question.to_string(),
        });

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_plan_creation() {
        let curator = FileAwareCuratorStage::new();
        
        let op = curator.create_operation_plan(
            "create",
            Path::new("test.rs"),
            Some("fn main() {}"),
            None
        );
        
        match op {
            FileOperation::Create { path, content } => {
                assert_eq!(path, PathBuf::from("test.rs"));
                assert_eq!(content, "fn main() {}");
            }
            _ => panic!("Expected Create operation"),
        }
    }

    #[test]
    fn test_safety_analysis() {
        let curator = FileAwareCuratorStage::new();
        
        let operations = vec![
            FileOperation::Delete { path: PathBuf::from("important.rs") },
            FileOperation::Create { path: PathBuf::from("new.rs"), content: String::new() },
        ];
        
        let analysis = curator.analyze_operation_safety(&operations);
        assert!(analysis.contains("DELETE"));
        assert!(analysis.contains("High risk"));
        assert!(analysis.contains("Low risk"));
    }
}