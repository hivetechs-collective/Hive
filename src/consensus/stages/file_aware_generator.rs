//! File-Aware Generator Stage
//!
//! A complete redesign of the generator stage that directly incorporates
//! file contents into the AI prompts in a clear, structured way.

use std::sync::Arc;
use anyhow::Result;
use tracing::{info, warn};

use crate::consensus::file_operations::{FileReader, SecurityPolicy};
use crate::consensus::repository_context::RepositoryContext;
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

    /// Read the most important files for understanding the repository
    async fn read_repository_files(&self, repo_context: &RepositoryContext) -> Result<String> {
        let mut file_contents = String::new();
        
        if let Some(root_path) = &repo_context.root_path {
            // Always try to read these critical files
            let critical_files = vec![
                "Cargo.toml",
                "package.json", 
                "README.md",
                "src/main.rs",
                "src/lib.rs",
                "src/index.ts",
                "src/index.js",
                "main.py",
                "go.mod",
            ];

            file_contents.push_str("# ACTUAL FILE CONTENTS FROM YOUR REPOSITORY\n\n");
            file_contents.push_str(&format!("Repository: {}\n\n", root_path.display()));

            let mut files_read = 0;
            
            for file_name in critical_files {
                let file_path = root_path.join(file_name);
                
                if let Ok(true) = self.file_reader.path_exists(&file_path).await {
                    match self.file_reader.read_file(&file_path).await {
                        Ok(content) => {
                            files_read += 1;
                            file_contents.push_str(&format!("## File: {}\n", file_name));
                            file_contents.push_str(&format!("```{}\n", content.language.as_deref().unwrap_or("")));
                            
                            // Include the full file if it's small, otherwise first 100 lines
                            let lines: Vec<&str> = content.content.lines().collect();
                            if lines.len() <= 100 {
                                file_contents.push_str(&content.content);
                            } else {
                                file_contents.push_str(&lines[..100].join("\n"));
                                file_contents.push_str(&format!("\n... ({} more lines)", lines.len() - 100));
                            }
                            
                            file_contents.push_str("\n```\n\n");
                        }
                        Err(e) => {
                            warn!("Failed to read {}: {}", file_name, e);
                        }
                    }
                }
            }

            if files_read == 0 {
                warn!("No files could be read from the repository!");
                file_contents.push_str("⚠️ Unable to read any files from the repository. The repository might be empty or have an unusual structure.\n\n");
            } else {
                info!("Successfully read {} files from repository", files_read);
            }
        }

        Ok(file_contents)
    }
}

impl ConsensusStage for FileAwareGeneratorStage {
    fn stage(&self) -> Stage {
        Stage::Generator
    }

    fn system_prompt(&self) -> &'static str {
        "You are analyzing a software repository. You will be provided with ACTUAL FILE CONTENTS from the repository. Base your analysis on these real files, not on assumptions or generic examples. Always reference the specific code you see in the files provided."
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