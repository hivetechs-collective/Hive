//! File-Aware Generator Stage
//!
//! A complete redesign of the generator stage that directly incorporates
//! file contents into the AI prompts in a clear, structured way.

use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};

use crate::consensus::file_operations::{FileReader, SecurityPolicy};
use crate::consensus::repository_context::RepositoryContext;
use crate::consensus::stages::repository_scanner::{FilePriority, RepositoryScanner};
use crate::consensus::stages::ConsensusStage;
use crate::consensus::types::{Message, Stage};

/// File-aware generator that reads and presents actual file contents
pub struct FileAwareGeneratorStage {
    file_reader: Arc<FileReader>,
}

impl FileAwareGeneratorStage {
    pub fn new() -> Self {
        Self {
            file_reader: Arc::new(FileReader::new(SecurityPolicy::default())),
        }
    }

    /// Read repository files comprehensively with intelligent prioritization
    async fn read_repository_files(&self, repo_context: &RepositoryContext) -> Result<String> {
        let mut file_contents = String::new();

        if let Some(root_path) = &repo_context.root_path {
            file_contents.push_str("# COMPREHENSIVE REPOSITORY ANALYSIS\n\n");
            file_contents.push_str(&format!("Repository: {}\n\n", root_path.display()));

            // Scan the entire repository structure
            let discovered_files = RepositoryScanner::scan_repository_files(root_path).await?;

            if discovered_files.is_empty() {
                file_contents.push_str("⚠️ **No source files found in repository.** Please ensure you have initialized your project properly.\n\n");
                return Ok(file_contents);
            }

            // Prioritize files for reading (Generator gets more files than Curator since it's the first stage)
            let prioritized_files =
                RepositoryScanner::prioritize_files_for_reading(&discovered_files);
            let max_files = 30; // Generator gets slightly more files for comprehensive understanding
            let max_context_chars = 60000; // ~15k tokens

            file_contents.push_str(&format!(
                "## Repository Structure ({} files discovered)\n\n",
                discovered_files.len()
            ));

            let mut files_read = 0;
            for file_info in prioritized_files.iter().take(max_files) {
                if file_contents.len() > max_context_chars {
                    file_contents.push_str(&format!(
                        "\n... ({} more files truncated for context limits)\n\n",
                        prioritized_files.len() - files_read
                    ));
                    break;
                }

                match self.file_reader.read_file(&file_info.path).await {
                    Ok(content) => {
                        let relative_path = file_info
                            .path
                            .strip_prefix(root_path)
                            .unwrap_or(&file_info.path);

                        // Add priority indicator for transparency
                        let priority_indicator = match file_info.priority {
                            FilePriority::Critical => "🔴 CRITICAL",
                            FilePriority::High => "🟠 HIGH",
                            FilePriority::Medium => "🟡 MEDIUM",
                            FilePriority::Normal => "🟢 NORMAL",
                            FilePriority::Low => "⚪ LOW",
                        };

                        file_contents.push_str(&format!(
                            "## File: {} ({})\n",
                            relative_path.display(),
                            priority_indicator
                        ));
                        file_contents.push_str(&format!(
                            "```{}\n",
                            content.language.as_deref().unwrap_or("")
                        ));

                        // Smart content display based on priority and size
                        let lines: Vec<&str> = content.content.lines().collect();
                        if lines.len() <= 120 {
                            // Small files: show complete content
                            file_contents.push_str(&content.content);
                        } else if file_info.priority == FilePriority::Critical {
                            // Critical files: show extensive content
                            file_contents.push_str(&lines[..200.min(lines.len())].join("\n"));
                            if lines.len() > 200 {
                                file_contents
                                    .push_str(&format!("\n... ({} more lines)", lines.len() - 200));
                            }
                        } else if file_info.priority == FilePriority::High {
                            // High priority: show substantial content
                            file_contents.push_str(&lines[..150.min(lines.len())].join("\n"));
                            if lines.len() > 150 {
                                file_contents
                                    .push_str(&format!("\n... ({} more lines)", lines.len() - 150));
                            }
                        } else {
                            // Other files: show first 100 lines
                            file_contents.push_str(&lines[..100.min(lines.len())].join("\n"));
                            if lines.len() > 100 {
                                file_contents
                                    .push_str(&format!("\n... ({} more lines)", lines.len() - 100));
                            }
                        }

                        file_contents.push_str("\n```\n\n");
                        files_read += 1;
                    }
                    Err(e) => {
                        warn!("Failed to read {}: {}", file_info.path.display(), e);
                    }
                }
            }

            file_contents.push_str(&format!(
                "✅ **Successfully analyzed {} of {} files** from your repository.\n\n",
                files_read,
                discovered_files.len()
            ));

            // Add repository summary
            RepositoryScanner::add_repository_summary(&mut file_contents, &discovered_files);
        }

        Ok(file_contents)
    }
}

impl ConsensusStage for FileAwareGeneratorStage {
    fn stage(&self) -> Stage {
        Stage::Generator
    }

    fn system_prompt(&self) -> &'static str {
        r#"You are the Generator stage of a 4-stage AI consensus pipeline, analyzing a software repository comprehensively.

You have been provided with ACTUAL FILE CONTENTS from the entire repository structure, prioritized by importance:
🔴 CRITICAL: Project definition files (Cargo.toml, package.json, etc.)
🟠 HIGH: Main entry points (main.rs, lib.rs, index.js, etc.)  
🟡 MEDIUM: Configuration files
🟢 NORMAL: Regular source files
⚪ LOW: Tests, documentation

Your role is to:
1. Provide comprehensive analysis based on the REAL files you can see
2. Generate creative solutions that work with the actual codebase structure
3. Reference specific files and code sections in your response
4. Consider the full context of the repository, not just individual files

Always base your analysis on the actual code provided, never on assumptions or generic examples."#
    }

    fn build_messages(
        &self,
        question: &str,
        _previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<Message>> {
        let mut messages = vec![];

        // System message with clear instructions
        messages.push(Message {
            role: "system".to_string(),
            content: self.system_prompt().to_string(),
        });

        // Add any additional context (like repository info)
        if let Some(ctx) = context {
            if ctx.contains("Repository Path:") || ctx.contains("CRITICAL REPOSITORY CONTEXT") {
                messages.push(Message {
                    role: "system".to_string(),
                    content: format!("Repository Information:\n{}", ctx),
                });
            }
        }

        // The user's question
        messages.push(Message {
            role: "user".to_string(),
            content: question.to_string(),
        });

        Ok(messages)
    }
}

/// Build messages with actual file contents
pub async fn build_file_aware_messages(
    stage: &FileAwareGeneratorStage,
    question: &str,
    repo_context: Option<&RepositoryContext>,
    base_context: Option<&str>,
) -> Result<Vec<Message>> {
    let mut messages = vec![];

    // System message
    messages.push(Message {
        role: "system".to_string(),
        content: stage.system_prompt().to_string(),
    });

    // Add repository context if available
    if let Some(ctx) = base_context {
        messages.push(Message {
            role: "system".to_string(),
            content: format!("Context:\n{}", ctx),
        });
    }

    // Read and add actual file contents
    if let Some(repo_ctx) = repo_context {
        let file_contents = stage.read_repository_files(repo_ctx).await?;

        if !file_contents.is_empty() {
            messages.push(Message {
                role: "system".to_string(),
                content: file_contents,
            });
        }
    }

    // The user's question - make it clear we want analysis of the actual files
    let enhanced_question = format!(
        "{}\n\nIMPORTANT: Base your response on the ACTUAL FILE CONTENTS provided above. Do not make assumptions or use generic examples. Quote and reference the real code you see in the files.",
        question
    );

    messages.push(Message {
        role: "user".to_string(),
        content: enhanced_question,
    });

    Ok(messages)
}
