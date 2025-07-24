//! Verified Context Builder - Conditional repository verification for consensus stages
//!
//! This module ensures that consensus stages receive verified repository facts
//! only when the question is repository-related, preventing irrelevant context injection.

use std::sync::Arc;
use anyhow::{Result, Context};
use regex::Regex;
use crate::consensus::verification::{RepositoryVerifier, RepositoryFacts, build_stage_context};
use crate::consensus::types::Stage;
use crate::consensus::repository_context::RepositoryContextManager;
use crate::consensus::temporal::TemporalContext;
use crate::ai_helpers::AIHelperEcosystem;

/// Enhanced context builder with mandatory repository verification
pub struct VerifiedContextBuilder {
    repository_verifier: Option<RepositoryVerifier>,
    repository_facts: Option<RepositoryFacts>,
}

impl VerifiedContextBuilder {
    /// Create a new verified context builder
    pub fn new() -> Self {
        Self {
            repository_verifier: None,
            repository_facts: None,
        }
    }
    
    /// Check if the question is related to the repository/codebase
    fn is_repository_related_question(&self, question: &str) -> bool {
        let question_lower = question.to_lowercase();
        
        // Repository-specific keywords that indicate the question is about the codebase
        let repo_keywords = [
            // Code-specific terms
            "this code", "the code", "this function", "this method", "this class",
            "this file", "this module", "this project", "this repo", "this repository",
            "this codebase", "this implementation", "this feature", "this component",
            "this system", "this architecture", "this design", "this structure",
            
            // Direct references
            "here", "above", "below", "this", "these",
            
            // Code analysis terms
            "analyze this", "explain this", "debug this", "fix this", "refactor this",
            "improve this", "optimize this", "test this", "document this",
            
            // File/path references
            "src/", ".rs", ".toml", ".md", "cargo", "target/",
            
            // Questions about the current context
            "what does this", "how does this", "why does this", "where is this",
            "what is this", "can you explain this", "help me understand this",
            
            // Development tasks
            "implement", "add a feature", "create a function", "write a test",
            "fix the bug", "update the", "modify the", "change the",
            
            // Hive-specific terms
            "hive", "consensus", "ai helper", "curator", "validator", "generator", "refiner",
        ];
        
        // Check if question contains repository-specific terms
        let contains_repo_keyword = repo_keywords.iter()
            .any(|&keyword| question_lower.contains(keyword));
        
        // General knowledge indicators (NOT repository-related)
        let general_knowledge_patterns = [
            // Geographic/demographic questions
            r"(?i)(city|cities|country|countries|population|capital|state|province)",
            r"(?i)(largest|biggest|smallest|top \d+|most populous)",
            
            // General facts
            r"(?i)(what is|what are|who is|who was|when did|where is)(?!.*(?:this|here|above|below))",
            r"(?i)(define|definition of|meaning of)(?!.*(?:this|in this))",
            
            // Math/science
            r"(?i)^\d+\s*[\+\-\*/]\s*\d+",
            r"(?i)(calculate|compute|solve)",
            
            // Historical/factual
            r"(?i)(history|historical|invented|discovered|founded)",
            r"(?i)(fact|facts about)(?!.*(?:this|repo|code))",
            
            // Current events
            r"(?i)(news|current|latest|recent)(?!.*(?:commit|update|change|version))",
            
            // Tutorial/how-to (general, not about this code)
            r"(?i)^(how to|how do i|how can i)(?!.*(?:this|here|in this))",
        ];
        
        // Check if it matches general knowledge patterns
        let is_general_knowledge = general_knowledge_patterns.iter()
            .any(|pattern| regex::Regex::new(pattern).unwrap().is_match(&question_lower));
        
        // If it's clearly general knowledge, it's NOT repository-related
        if is_general_knowledge && !contains_repo_keyword {
            return false;
        }
        
        // If it contains repository keywords, it IS repository-related
        if contains_repo_keyword {
            return true;
        }
        
        // Default: if we have repository facts loaded, be conservative and assume
        // ambiguous questions might be repository-related
        self.repository_facts.is_some() && !is_general_knowledge
    }
    
    /// Initialize with repository path for verification
    pub async fn with_repository_verification(&mut self, repo_path: std::path::PathBuf) -> Result<&mut Self> {
        tracing::info!("Initializing repository verification for: {}", repo_path.display());
        
        let mut verifier = RepositoryVerifier::new(repo_path);
        let facts = verifier.verify_repository_context().await
            .context("Failed to verify repository context")?;
        
        tracing::info!("Repository verification complete: {} v{} ({} deps, {} modules)", 
            facts.name, facts.version, facts.dependency_count, facts.module_count);
        
        self.repository_verifier = Some(verifier);
        self.repository_facts = Some(facts);
        
        Ok(self)
    }
    
    /// Build verified context for a specific consensus stage
    pub async fn build_verified_stage_context(
        &self,
        stage: Stage,
        question: &str,
        semantic_context: Option<String>,
        temporal_context: Option<TemporalContext>,
        memory_context: Option<String>,
        repository_context: Option<Arc<RepositoryContextManager>>,
        ai_helpers: Option<Arc<AIHelperEcosystem>>,
    ) -> Result<String> {
        let mut contexts = Vec::new();
        
        // Check if this question is repository-related
        let is_repo_related = self.is_repository_related_question(question);
        
        // 1. Repository verification context (ONLY if question is repository-related)
        if is_repo_related {
            if let Some(facts) = &self.repository_facts {
                let verification_context = build_stage_context(facts, stage);
                contexts.push(verification_context);
                tracing::info!("Question is repository-related, added verification context for {:?} stage", stage);
            } else {
                tracing::warn!("Repository-related question but no facts available!");
                contexts.push(format!(
                    "\n⚠️  WARNING: This appears to be a repository-specific question, but no repository verification was performed.\n"
                ));
            }
        } else {
            tracing::info!("Question is NOT repository-related (e.g., general knowledge), skipping repository context");
        }
        
        // 2. AI Helper enhanced context (with repository facts)
        if let Some(ai_helpers) = ai_helpers {
            match self.get_ai_helper_context(&ai_helpers, question, stage).await {
                Ok(helper_context) => {
                    if !helper_context.is_empty() {
                        contexts.push(format!("## 🤖 AI HELPER INSIGHTS\n{}", helper_context));
                        tracing::debug!("Added AI helper context for {:?} stage", stage);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to get AI helper context for {:?} stage: {}", stage, e);
                }
            }
        }
        
        // 3. Memory context (previous curator results)
        if let Some(memory) = memory_context {
            if !memory.is_empty() {
                contexts.push(format!("## 📚 MEMORY CONTEXT\n{}", memory));
                tracing::debug!("Added memory context");
            }
        }
        
        // 4. Semantic context (codebase search results)
        if let Some(semantic) = semantic_context {
            if !semantic.is_empty() {
                contexts.push(format!("## 🔍 SEMANTIC SEARCH RESULTS\n{}", semantic));
                tracing::debug!("Added semantic context");
            }
        }
        
        // 5. Temporal context (current time and web search if needed)
        if let Some(temporal) = temporal_context {
            let temporal_str = self.format_temporal_context(temporal);
            if !temporal_str.is_empty() {
                contexts.push(format!("## ⏰ TEMPORAL CONTEXT\n{}", temporal_str));
                tracing::debug!("Added temporal context");
            }
        }
        
        // 6. Repository context (current project state - only if repository-related)
        if is_repo_related {
            if let Some(repo_ctx) = repository_context {
                let repo_info = repo_ctx.get_context_for_prompts().await;
                if !repo_info.is_empty() {
                    contexts.push(format!("## 📁 REPOSITORY CONTEXT\n{}", repo_info));
                    tracing::debug!("Added repository context");
                }
            }
        }
        
        // 7. Stage-specific guidance
        let stage_guidance = self.get_stage_specific_guidance(stage);
        if !stage_guidance.is_empty() {
            contexts.push(format!("## 🎯 STAGE GUIDANCE\n{}", stage_guidance));
            tracing::debug!("Added stage-specific guidance for {:?}", stage);
        }
        
        let full_context = contexts.join("\n\n");
        tracing::info!("Built verified context for {:?} stage: {} characters", stage, full_context.len());
        
        Ok(full_context)
    }
    
    /// Get AI helper context enhanced with repository facts
    async fn get_ai_helper_context(
        &self,
        ai_helpers: &AIHelperEcosystem,
        question: &str,
        stage: Stage,
    ) -> Result<String> {
        // Only update AI helpers with repository facts if the question is repository-related
        if self.is_repository_related_question(question) {
            if let Err(e) = ai_helpers.update_repository_facts(self.repository_facts.clone()).await {
                tracing::warn!("Failed to update AI helpers with repository facts: {}", e);
            }
        } else {
            // Clear repository facts from AI helpers for non-repository questions
            if let Err(e) = ai_helpers.update_repository_facts(None).await {
                tracing::warn!("Failed to clear repository facts from AI helpers: {}", e);
            }
        }
        
        // Use the AI helpers with repository-enhanced context
        match ai_helpers.prepare_stage_context(question, stage, 2048).await {
            Ok(stage_context) => {
                // Format the StageContext into a string
                let formatted_context = self.format_stage_context(stage_context);
                
                // If we have repository facts, enhance the AI helper context
                if let Some(facts) = &self.repository_facts {
                    Ok(format!(
                        "Repository: {} v{} ({} dependencies, {} modules)\n\n{}",
                        facts.name, facts.version, facts.dependency_count, facts.module_count,
                        formatted_context
                    ))
                } else {
                    Ok(formatted_context)
                }
            },
            Err(e) => {
                tracing::warn!("AI helpers failed for {:?} stage: {}", stage, e);
                Ok(String::new())
            }
        }
    }
    
    /// Format a StageContext into a readable string
    fn format_stage_context(&self, context: crate::ai_helpers::StageContext) -> String {
        let mut parts = Vec::new();
        
        if !context.relevant_facts.is_empty() {
            parts.push("Relevant Facts:".to_string());
            for fact in context.relevant_facts {
                parts.push(format!("- {}", fact));
            }
        }
        
        if !context.patterns.is_empty() {
            parts.push("Patterns:".to_string());
            for pattern in context.patterns {
                parts.push(format!("- {:?}: {} (confidence: {:.2})", pattern.pattern_type, pattern.description, pattern.confidence));
            }
        }
        
        if !context.insights.is_empty() {
            parts.push("Insights:".to_string());
            for insight in context.insights {
                parts.push(format!("- {:?}: {}", insight.insight_type, insight.content));
            }
        }
        
        if let Some(guidance) = context.custom_guidance {
            parts.push(format!("Guidance: {}", guidance));
        }
        
        parts.join("\n")
    }
    
    /// Format temporal context for inclusion
    fn format_temporal_context(&self, temporal: TemporalContext) -> String {
        let mut parts = Vec::new();
        
        parts.push(format!("Current date: {}", temporal.current_date));
        parts.push(format!("Current time: {}", temporal.current_datetime));
        
        if !temporal.temporal_awareness.is_empty() {
            parts.push(format!("Temporal awareness: {}", temporal.temporal_awareness));
        }
        
        if let Some(business_context) = temporal.business_context {
            parts.push(format!("Business context: Quarter {}, Market hours: {}", 
                business_context.quarter, business_context.is_market_hours));
        }
        
        parts.join("\n")
    }
    
    /// Get stage-specific guidance
    fn get_stage_specific_guidance(&self, stage: Stage) -> String {
        match stage {
            Stage::Generator => {
                "Focus on comprehensive understanding and creative solutions. \
                Base your analysis on the verified repository facts above. \
                Generate solutions that acknowledge the actual project structure and dependencies.".to_string()
            },
            Stage::Refiner => {
                "Improve clarity, accuracy, and completeness while staying true to the verified facts. \
                Any refinements must be consistent with the actual repository characteristics. \
                Reference high-quality examples from past responses.".to_string()
            },
            Stage::Validator => {
                "Validate responses against the verified repository facts above. \
                Flag any claims that contradict the actual repository structure, dependencies, or characteristics. \
                Be critical but constructive in validation.".to_string()
            },
            Stage::Curator => {
                "Synthesize the final response using only information consistent with the verified facts. \
                Ensure the final answer accurately reflects the actual repository. \
                Create lasting knowledge that will benefit future queries.".to_string()
            },
        }
    }
    
    /// Get repository facts for external use
    pub fn get_repository_facts(&self) -> Option<&RepositoryFacts> {
        self.repository_facts.as_ref()
    }
    
    /// Check if repository verification is available
    pub fn has_verification(&self) -> bool {
        self.repository_facts.is_some()
    }
}

impl Default for VerifiedContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    
    async fn create_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();
        
        // Create a minimal Cargo.toml
        let cargo_content = r#"
[package]
name = "test-project"
version = "1.0.0"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#;
        let mut cargo_file = File::create(root_path.join("Cargo.toml")).unwrap();
        cargo_file.write_all(cargo_content.as_bytes()).unwrap();
        
        // Create some Rust files
        std::fs::create_dir(root_path.join("src")).unwrap();
        let mut main_file = File::create(root_path.join("src").join("main.rs")).unwrap();
        main_file.write_all(b"fn main() {}\n").unwrap();
        
        temp_dir
    }
    
    #[tokio::test]
    async fn test_verified_context_builder() {
        let temp_dir = create_test_repo().await;
        let root_path = temp_dir.path().to_path_buf();
        
        let mut builder = VerifiedContextBuilder::new();
        builder.with_repository_verification(root_path).await.unwrap();
        
        assert!(builder.has_verification());
        
        let facts = builder.get_repository_facts().unwrap();
        assert_eq!(facts.name, "test-project");
        assert_eq!(facts.version, "1.0.0");
        assert_eq!(facts.dependency_count, 2);
    }
    
    #[tokio::test]
    async fn test_stage_context_building() {
        let temp_dir = create_test_repo().await;
        let root_path = temp_dir.path().to_path_buf();
        
        let mut builder = VerifiedContextBuilder::new();
        builder.with_repository_verification(root_path).await.unwrap();
        
        let context = builder.build_verified_stage_context(
            Stage::Generator,
            "What does this project do?",
            None,
            None,
            None,
            None,
            None,
        ).await.unwrap();
        
        assert!(context.contains("test-project v1.0.0"));
        assert!(context.contains("GENERATOR:"));
        assert!(context.contains("2 external crates"));
    }
}