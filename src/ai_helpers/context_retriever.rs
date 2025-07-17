//! Context Retriever - Uses GraphCodeBERT + LangChain for intelligent retrieval
//! 
//! This module finds relevant past knowledge, ranks by relevance to the current question,
//! and compresses information for optimal context preparation.

use std::sync::Arc;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid;

use crate::ai_helpers::{ChromaVectorStore, StageContext, Pattern, Insight};
use crate::consensus::types::Stage;
use crate::consensus::verification::RepositoryFacts;
use super::python_models::{PythonModelService, ModelRequest, ModelResponse};

/// Decision about whether to use repository context
#[derive(Debug, Clone)]
pub struct ContextDecision {
    pub should_use_repo: bool,
    pub confidence: f64,
    pub category: String,
    pub reasoning: String,
}

/// Analysis result from AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnalysis {
    pub category: String,
    pub confidence: f64,
    pub reasoning: String,
}

/// Configuration for Context Retriever
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieverConfig {
    /// Model for understanding code relationships
    pub relationship_model: String,
    
    /// Maximum context size per stage (in tokens)
    pub max_context_tokens: usize,
    
    /// Number of candidates to retrieve before ranking
    pub retrieval_candidates: usize,
    
    /// Relevance threshold for inclusion
    pub relevance_threshold: f64,
}

impl Default for RetrieverConfig {
    fn default() -> Self {
        Self {
            relationship_model: "microsoft/graphcodebert-base".to_string(),
            max_context_tokens: 2048,
            retrieval_candidates: 50,
            relevance_threshold: 0.7,
        }
    }
}

/// Context Retriever with stage-specific intelligence
pub struct ContextRetriever {
    config: RetrieverConfig,
    vector_store: Arc<ChromaVectorStore>,
    
    /// Python model service
    python_service: Arc<PythonModelService>,
    
    /// Cache of recent retrievals
    retrieval_cache: Arc<RwLock<lru::LruCache<String, StageContext>>>,
    
    /// Repository facts for enhanced context
    repository_facts: Arc<RwLock<Option<RepositoryFacts>>>,
}

impl ContextRetriever {
    /// Create a new Context Retriever
    pub async fn new(
        vector_store: Arc<ChromaVectorStore>,
        python_service: Arc<PythonModelService>,
    ) -> Result<Self> {
        let config = RetrieverConfig::default();
        let retrieval_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(100).unwrap()
        )));
        
        Ok(Self {
            config,
            vector_store,
            python_service,
            retrieval_cache,
            repository_facts: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Update repository facts for enhanced context preparation
    pub async fn update_repository_facts(&self, facts: Option<RepositoryFacts>) -> Result<()> {
        let mut repo_facts = self.repository_facts.write().await;
        *repo_facts = facts.clone();
        
        if let Some(ref facts) = facts {
            tracing::info!(
                "Repository facts updated: {} v{} ({} deps, {} modules)", 
                facts.name, facts.version, facts.dependency_count, facts.module_count
            );
        } else {
            tracing::warn!("Repository facts cleared - context may be less accurate");
        }
        
        // Clear cache since context will be different with new facts
        self.retrieval_cache.write().await.clear();
        
        Ok(())
    }
    
    /// Get repository-enhanced facts for context
    async fn get_repository_enhanced_facts(&self, base_facts: Vec<String>) -> Vec<String> {
        let mut enhanced_facts = base_facts;
        
        if let Some(ref facts) = *self.repository_facts.read().await {
            enhanced_facts.insert(0, format!(
                "Repository Context: {} v{} ({} dependencies, {} modules, {} files, {} LOC)",
                facts.name, facts.version, facts.dependency_count, 
                facts.module_count, facts.total_files, facts.lines_of_code
            ));
            
            if facts.is_enterprise {
                enhanced_facts.insert(1, "This is an enterprise-grade project with complex architecture".to_string());
            } else {
                enhanced_facts.insert(1, "This is a standard project with straightforward structure".to_string());
            }
        }
        
        enhanced_facts
    }
    
    /// Get context optimized for Generator stage
    pub async fn get_generator_context(
        &self,
        question: &str,
        context_limit: usize,
    ) -> Result<StageContext> {
        tracing::debug!("Preparing Generator context for: {}", question);
        
        // Check cache
        let cache_key = format!("generator_{}", question);
        if let Some(cached) = self.retrieval_cache.read().await.peek(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Generator needs broad context and examples
        let base_facts = self.retrieve_broad_context(question, context_limit).await?;
        let relevant_facts = self.get_repository_enhanced_facts(base_facts).await;
        let patterns = self.find_relevant_patterns(question).await?;
        let insights = self.get_relevant_insights(question).await?;
        
        let context = StageContext {
            stage: Stage::Generator,
            relevant_facts,
            patterns,
            insights,
            custom_guidance: Some(
                "Focus on comprehensive understanding and creative solutions. \
                Draw from past examples but adapt to the specific question.".to_string()
            ),
        };
        
        // Cache the result
        self.retrieval_cache.write().await.put(cache_key, context.clone());
        
        Ok(context)
    }
    
    /// Get context optimized for Refiner stage
    pub async fn get_refiner_context(
        &self,
        question: &str,
        context_limit: usize,
    ) -> Result<StageContext> {
        tracing::debug!("Preparing Refiner context for: {}", question);
        
        // Refiner needs high-quality examples and improvement patterns
        let base_facts = self.retrieve_quality_examples(question, context_limit).await?;
        let relevant_facts = self.get_repository_enhanced_facts(base_facts).await;
        let patterns = self.find_improvement_patterns(question).await?;
        let insights = self.get_quality_insights(question).await?;
        
        Ok(StageContext {
            stage: Stage::Refiner,
            relevant_facts,
            patterns,
            insights,
            custom_guidance: Some(
                "Focus on improving clarity, accuracy, and completeness. \
                Reference high-quality examples from past responses.".to_string()
            ),
        })
    }
    
    /// Get context optimized for Validator stage
    pub async fn get_validator_context(
        &self,
        question: &str,
        context_limit: usize,
    ) -> Result<StageContext> {
        tracing::debug!("Preparing Validator context for: {}", question);
        
        // Validator needs contradictions, edge cases, and verification patterns
        let base_facts = self.retrieve_edge_cases(question, context_limit).await?;
        let relevant_facts = self.get_repository_enhanced_facts(base_facts).await;
        let patterns = self.find_contradiction_patterns(question).await?;
        let insights = self.get_validation_insights(question).await?;
        
        Ok(StageContext {
            stage: Stage::Validator,
            relevant_facts,
            patterns,
            insights,
            custom_guidance: Some(
                "Focus on identifying potential issues, contradictions, and edge cases. \
                Be critical but constructive in validation.".to_string()
            ),
        })
    }
    
    /// Get context optimized for Curator stage
    pub async fn get_curator_context(
        &self,
        question: &str,
        context_limit: usize,
    ) -> Result<StageContext> {
        tracing::debug!("Preparing Curator context for: {}", question);
        
        // Curator needs synthesis opportunities and authoritative examples
        let base_facts = self.retrieve_synthesis_context(question, context_limit).await?;
        let relevant_facts = self.get_repository_enhanced_facts(base_facts).await;
        let patterns = self.find_synthesis_patterns(question).await?;
        let insights = self.get_synthesis_insights(question).await?;
        
        Ok(StageContext {
            stage: Stage::Curator,
            relevant_facts,
            patterns,
            insights,
            custom_guidance: Some(
                "Focus on synthesizing the best elements into a coherent, authoritative response. \
                Create lasting knowledge that will benefit future queries.".to_string()
            ),
        })
    }
    
    /// Retrieve broad context for Generator
    async fn retrieve_broad_context(
        &self,
        question: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        // TODO: Implement actual retrieval with GraphCodeBERT ranking
        // For now, return placeholder
        Ok(vec![
            "Example: Previous implementation of similar feature...".to_string(),
            "Pattern: Common approach to this type of problem...".to_string(),
        ])
    }
    
    /// Retrieve high-quality examples for Refiner
    async fn retrieve_quality_examples(
        &self,
        question: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        // TODO: Filter for high-confidence, well-rated responses
        Ok(vec![
            "High-quality example with clear explanation...".to_string(),
        ])
    }
    
    /// Retrieve edge cases for Validator
    async fn retrieve_edge_cases(
        &self,
        question: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        // TODO: Look for past issues, corrections, and edge cases
        Ok(vec![
            "Edge case: When X happens, consider Y...".to_string(),
        ])
    }
    
    /// Retrieve synthesis opportunities for Curator
    async fn retrieve_synthesis_context(
        &self,
        question: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        // TODO: Find related concepts that can be synthesized
        Ok(vec![
            "Related concept that can be integrated...".to_string(),
        ])
    }
    
    /// Find patterns relevant to the question
    async fn find_relevant_patterns(&self, question: &str) -> Result<Vec<Pattern>> {
        // TODO: Implement pattern matching with UniXcoder
        Ok(vec![])
    }
    
    /// Find improvement patterns
    async fn find_improvement_patterns(&self, question: &str) -> Result<Vec<Pattern>> {
        Ok(vec![])
    }
    
    /// Find contradiction patterns
    async fn find_contradiction_patterns(&self, question: &str) -> Result<Vec<Pattern>> {
        Ok(vec![])
    }
    
    /// Find synthesis patterns
    async fn find_synthesis_patterns(&self, question: &str) -> Result<Vec<Pattern>> {
        Ok(vec![])
    }
    
    /// Get relevant insights
    async fn get_relevant_insights(&self, question: &str) -> Result<Vec<Insight>> {
        Ok(vec![])
    }
    
    /// Get quality insights
    async fn get_quality_insights(&self, question: &str) -> Result<Vec<Insight>> {
        Ok(vec![])
    }
    
    /// Get validation insights
    async fn get_validation_insights(&self, question: &str) -> Result<Vec<Insight>> {
        Ok(vec![])
    }
    
    /// Get synthesis insights
    async fn get_synthesis_insights(&self, question: &str) -> Result<Vec<Insight>> {
        Ok(vec![])
    }
    
    /// Analyze question to determine if repository context should be used
    /// This uses GraphCodeBERT to understand the semantic content of the question
    pub async fn should_use_repository_context(
        &self,
        question: &str,
        has_open_repository: bool,
    ) -> Result<bool> {
        // If no repository is open, never use repository context
        if !has_open_repository {
            tracing::info!("No repository open, skipping repository context");
            return Ok(false);
        }

        // Use GraphCodeBERT to analyze the question semantically
        let context_decision = self.analyze_question_context(question).await?;
        
        tracing::debug!(
            "Repository context decision for '{}': {} (confidence: {:.2})",
            question, context_decision.should_use_repo, context_decision.confidence
        );
        
        Ok(context_decision.should_use_repo)
    }
    
    /// Analyze question using GraphCodeBERT to determine context needs
    async fn analyze_question_context(&self, question: &str) -> Result<ContextDecision> {
        let model = "microsoft/graphcodebert-base";
        
        match self.python_service.analyze_code(
            model,
            question,
            "classify_question_context"
        ).await {
            Ok(result) => {
                self.parse_context_decision_from_value(&result, question)
            }
            Err(e) => {
                tracing::warn!("Failed to analyze question context with AI model: {}, falling back to heuristics", e);
                Ok(self.fallback_context_analysis(question))
            }
        }
    }
    
    /// Parse the AI model response from a Value into a context decision
    fn parse_context_decision_from_value(&self, result: &serde_json::Value, question: &str) -> Result<ContextDecision> {
        if let Ok(analysis) = serde_json::from_value::<QuestionAnalysis>(result.clone()) {
            let should_use_repo = match analysis.category.as_str() {
                "repository_specific" => true,
                "general_programming" => {
                    // For general programming, only use repo context if high confidence
                    // and the question might benefit from examples
                    analysis.confidence > 0.8 && (
                        analysis.reasoning.contains("example") ||
                        analysis.reasoning.contains("implement") ||
                        analysis.reasoning.contains("how to")
                    )
                }
                _ => false,
            };
            
            return Ok(ContextDecision {
                should_use_repo,
                confidence: analysis.confidence,
                category: analysis.category,
                reasoning: analysis.reasoning,
            });
        }
        
        // If parsing failed, fall back to heuristics
        tracing::warn!("Failed to parse AI context analysis, using fallback");
        Ok(self.fallback_context_analysis(question))
    }
    
    /// Fallback heuristic analysis when AI model is unavailable
    fn fallback_context_analysis(&self, question: &str) -> ContextDecision {
        let question_lower = question.to_lowercase();
        
        // Repository-specific indicators
        let repo_keywords = [
            "this code", "this project", "this repo", "this repository",
            "this file", "this function", "this class", "this method",
            "my code", "my project", "my repo", "my repository", 
            "our code", "our project", "our repo", "our repository",
            "current code", "current project", "current repo", "current repository",
            "@codebase", "in this", "here", "this implementation",
            "analyze this", "review this", "check this", "fix this",
            "update this", "modify this", "change this", "improve this",
            "the code", "the project", "the repo", "the repository",
        ];
        
        // General programming indicators
        let general_keywords = [
            "difference between", "compare", "versus", "vs", "or",
            "what is", "how does", "explain", "when to use", 
            "best practice", "which is better", "pros and cons",
            "advantages", "disadvantages", "tell me about",
            "angular", "vue", "react", "svelte", "ember",
            "python", "rust", "javascript", "typescript", "java",
            "design pattern", "algorithm", "data structure",
            "performance", "optimization", "security",
        ];
        
        let repo_score = repo_keywords.iter()
            .filter(|keyword| question_lower.contains(*keyword))
            .count();
            
        let general_score = general_keywords.iter()
            .filter(|keyword| question_lower.contains(*keyword))
            .count();
        
        let should_use_repo = if repo_score > 0 && repo_score >= general_score {
            true
        } else if general_score > repo_score {
            false
        } else {
            // Ambiguous case - default to false for general questions
            false
        };
        
        let confidence = if repo_score > 0 || general_score > 0 {
            0.7 // Medium confidence for heuristic matching
        } else {
            0.5 // Low confidence for unclear questions
        };
        
        ContextDecision {
            should_use_repo,
            confidence,
            category: if should_use_repo { "repository_specific" } else { "general_programming" }.to_string(),
            reasoning: format!(
                "Heuristic analysis: repo_keywords={}, general_keywords={}", 
                repo_score, general_score
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_stage_specific_retrieval() {
        // Test that each stage gets appropriate context
    }
}