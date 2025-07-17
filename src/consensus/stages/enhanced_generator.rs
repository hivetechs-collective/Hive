//! Enhanced Generator Stage with File Reading Capabilities
//!
//! This module extends the Generator stage to read actual file contents
//! when analyzing repositories.

use std::sync::Arc;
use anyhow::Result;
use tracing::{debug, info, instrument};

use crate::consensus::file_operations::{FileReader, SecurityPolicy};
use crate::consensus::repository_context::RepositoryContext;
use crate::consensus::stages::{ConsensusStage, GeneratorStage};
use crate::consensus::types::{Message, Stage};

/// Enhanced Generator with file reading capabilities
pub struct EnhancedGeneratorStage {
    base_generator: GeneratorStage,
    file_reader: Arc<FileReader>,
}

impl EnhancedGeneratorStage {
    /// Create a new enhanced generator stage
    pub fn new() -> Self {
        Self {
            base_generator: GeneratorStage::new(),
            file_reader: Arc::new(FileReader::new(SecurityPolicy::default())),
        }
    }

    /// Analyze repository with file reading
    #[instrument(skip(self))]
    pub async fn analyze_repository_with_files(
        &self,
        context: &RepositoryContext,
        question: &str,
    ) -> Result<String> {
        let mut analysis = String::new();

        // Add repository overview
        analysis.push_str("## Repository Analysis with File Contents\n\n");

        if let Some(root_path) = &context.root_path {
            analysis.push_str(&format!("### Repository: {}\n", root_path.display()));
            analysis.push_str(&format!("Project Type: {:?}\n\n", context.project_type));

            info!("Analyzing repository at: {}", root_path.display());

            // Read key files to understand the project
            let key_files = self.identify_key_files_to_read(context, question).await?;
            info!("Identified {} key files to read", key_files.len());
            
            if !key_files.is_empty() {
                analysis.push_str("### Key Files Analyzed:\n\n");
                
                for file_path in key_files.iter().take(10) {
                    info!("Attempting to read file: {}", file_path.display());
                    match self.file_reader.read_file(file_path).await {
                        Ok(content) => {
                            info!("Successfully read file: {} ({} bytes)", file_path.display(), content.size_bytes);
                            analysis.push_str(&format!("#### File: {}\n", file_path.display()));
                            analysis.push_str(&format!("Language: {}\n", content.language.as_deref().unwrap_or("unknown")));
                            analysis.push_str(&format!("Size: {} bytes, {} lines\n\n", content.size_bytes, content.lines));
                            
                            // Include relevant excerpts based on the question
                            if let Some(excerpt) = self.extract_relevant_excerpt(&content.content, question) {
                                analysis.push_str("**Actual code from this file:**\n");
                                analysis.push_str("```");
                                if let Some(lang) = &content.language {
                                    analysis.push_str(lang);
                                }
                                analysis.push_str("\n");
                                analysis.push_str(&excerpt);
                                analysis.push_str("\n```\n\n");
                                analysis.push_str("☝️ This is REAL CODE from the repository - use it in your analysis!\n\n");
                            }
                        }
                        Err(e) => {
                            debug!("Failed to read file {}: {}", file_path.display(), e);
                        }
                    }
                }
            }

            // Search for specific patterns if the question asks about something specific
            if let Some(search_terms) = self.extract_search_terms(question) {
                analysis.push_str("### Search Results:\n\n");
                
                for term in search_terms {
                    let search_results = self.search_codebase(&term, root_path).await?;
                    if !search_results.is_empty() {
                        analysis.push_str(&format!("Found {} matches for '{}'\n", search_results.len(), term));
                        
                        for (idx, result) in search_results.iter().take(5).enumerate() {
                            analysis.push_str(&format!("\nMatch {}: {} (line {})\n", idx + 1, result.path.display(), result.line_number));
                            analysis.push_str(&format!("```\n{}\n```\n", result.line_content));
                        }
                    }
                }
            }
        }

        Ok(analysis)
    }

    /// Identify key files to read based on project type and question
    async fn identify_key_files_to_read(
        &self,
        context: &RepositoryContext,
        question: &str,
    ) -> Result<Vec<std::path::PathBuf>> {
        let mut key_files = Vec::new();

        if let Some(root_path) = &context.root_path {
            // Always read project configuration files
            match context.project_type {
                crate::consensus::repository_context::ProjectType::Rust => {
                    key_files.push(root_path.join("Cargo.toml"));
                    key_files.push(root_path.join("src/main.rs"));
                    key_files.push(root_path.join("src/lib.rs"));
                    key_files.push(root_path.join("README.md"));
                }
                crate::consensus::repository_context::ProjectType::TypeScript
                | crate::consensus::repository_context::ProjectType::JavaScript => {
                    key_files.push(root_path.join("package.json"));
                    key_files.push(root_path.join("tsconfig.json"));
                    key_files.push(root_path.join("src/index.ts"));
                    key_files.push(root_path.join("src/index.js"));
                    key_files.push(root_path.join("README.md"));
                }
                crate::consensus::repository_context::ProjectType::Python => {
                    key_files.push(root_path.join("pyproject.toml"));
                    key_files.push(root_path.join("requirements.txt"));
                    key_files.push(root_path.join("setup.py"));
                    key_files.push(root_path.join("__main__.py"));
                    key_files.push(root_path.join("main.py"));
                    key_files.push(root_path.join("README.md"));
                }
                _ => {
                    // Generic files for unknown projects
                    key_files.push(root_path.join("README.md"));
                    key_files.push(root_path.join("LICENSE"));
                    key_files.push(root_path.join("Makefile"));
                    key_files.push(root_path.join(".gitignore"));
                }
            }

            // Add files mentioned in the question
            if question.contains("main") {
                key_files.push(root_path.join("src/main.rs"));
                key_files.push(root_path.join("src/main.py"));
                key_files.push(root_path.join("src/main.ts"));
                key_files.push(root_path.join("src/main.js"));
            }

            // Filter out non-existent files
            let mut existing_files = Vec::new();
            for file in key_files {
                if self.file_reader.path_exists(&file).await? {
                    existing_files.push(file);
                }
            }

            key_files = existing_files;
        }

        Ok(key_files)
    }

    /// Extract relevant excerpt from file content based on question
    fn extract_relevant_excerpt(&self, content: &str, question: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        
        // If file is small, include it all
        if lines.len() <= 50 {
            return Some(content.to_string());
        }

        // Look for main function, class definitions, or important sections
        let question_lower = question.to_lowercase();
        let mut relevant_sections = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            
            // Check for relevant keywords from the question
            for word in question_lower.split_whitespace() {
                if word.len() > 3 && line_lower.contains(word) {
                    let start = idx.saturating_sub(5);
                    let end = (idx + 15).min(lines.len());
                    relevant_sections.push((start, end));
                    break;
                }
            }

            // Check for important code patterns
            if line.contains("fn main")
                || line.contains("class ")
                || line.contains("def ")
                || line.contains("export ")
                || line.contains("pub struct")
                || line.contains("impl ")
            {
                let start = idx;
                let end = (idx + 20).min(lines.len());
                relevant_sections.push((start, end));
            }
        }

        // Merge overlapping sections and take the most relevant
        if !relevant_sections.is_empty() {
            relevant_sections.sort_by_key(|&(start, _)| start);
            let (start, mut end) = relevant_sections[0];
            
            for &(s, e) in &relevant_sections[1..] {
                if s <= end + 5 {
                    end = end.max(e);
                } else {
                    break;
                }
            }

            Some(lines[start..end].join("\n"))
        } else {
            // Return first 30 lines if nothing specific found
            Some(lines[..30.min(lines.len())].join("\n"))
        }
    }

    /// Extract search terms from the question
    fn extract_search_terms(&self, question: &str) -> Option<Vec<String>> {
        let question_lower = question.to_lowercase();
        
        // Keywords that indicate search intent
        if question_lower.contains("where")
            || question_lower.contains("find")
            || question_lower.contains("search")
            || question_lower.contains("look for")
            || question_lower.contains("uses")
            || question_lower.contains("implements")
        {
            // Extract meaningful terms (longer than 3 chars, not common words)
            let common_words = vec!["the", "and", "for", "with", "that", "this", "from", "what", "where", "when", "how"];
            let terms: Vec<String> = question_lower
                .split_whitespace()
                .filter(|word| word.len() > 3 && !common_words.contains(word))
                .map(String::from)
                .collect();
            
            if !terms.is_empty() {
                return Some(terms);
            }
        }
        
        None
    }

    /// Search the codebase for a specific term
    async fn search_codebase(
        &self,
        term: &str,
        root_path: &std::path::Path,
    ) -> Result<Vec<crate::consensus::file_operations::SearchMatch>> {
        // Find relevant files to search
        let pattern = format!("**/*.{{rs,ts,js,py,go,java,cs,cpp,c,h}}");
        let files = self.file_reader.glob_files(&pattern).await?;
        
        // Limit search to reasonable number of files
        let search_files: Vec<_> = files
            .into_iter()
            .filter(|p| p.starts_with(root_path))
            .take(100)
            .collect();
        
        // Search for the term
        self.file_reader.search_content(term, &search_files).await
    }

    /// Generate enhanced context with file contents
    pub async fn generate_with_files(
        &self,
        question: &str,
        repository_context: Option<&RepositoryContext>,
    ) -> Result<String> {
        let mut enhanced_context = String::new();

        // Add repository analysis with file contents if available
        if let Some(repo_context) = repository_context {
            if repo_context.root_path.is_some() {
                info!("Analyzing repository with file reading for generator stage");
                let analysis = self.analyze_repository_with_files(repo_context, question).await?;
                enhanced_context.push_str(&analysis);
            }
        }

        Ok(enhanced_context)
    }
}

impl ConsensusStage for EnhancedGeneratorStage {
    fn stage(&self) -> Stage {
        self.base_generator.stage()
    }

    fn system_prompt(&self) -> &'static str {
        self.base_generator.system_prompt()
    }

    fn build_messages(
        &self,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<Message>> {
        // Use base generator's message building
        self.base_generator.build_messages(question, previous_answer, context)
    }
}