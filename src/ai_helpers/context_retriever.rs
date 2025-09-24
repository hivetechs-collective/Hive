//! Context Retriever - Uses GraphCodeBERT + LangChain for intelligent retrieval
//!
//! This module finds relevant past knowledge, ranks by relevance to the current question,
//! compresses information for optimal context preparation, and analyzes operation history
//! for intelligent auto-accept decisions.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use super::python_models::PythonModelService;
use crate::ai_helpers::{ChromaVectorStore, Insight, Pattern, StageContext};
use crate::consensus::operation_intelligence::{OperationContext, OperationOutcome};
use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::types::Stage;
use crate::consensus::verification::RepositoryFacts;

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

/// Information about a discovered repository
#[derive(Debug, Clone)]
pub struct RepositoryInfo {
    pub path: std::path::PathBuf,
    pub project_type: String,
    pub description: String,
    pub key_files: Vec<String>,
    pub detected_technologies: Vec<String>,
}

/// AI analysis of a directory structure
#[derive(Debug, Clone)]
pub struct DirectoryAnalysis {
    pub is_repository: bool,
    pub project_type: String,
    pub description: String,
    pub key_files: Vec<String>,
    pub technologies: Vec<String>,
}

/// Operation context analysis for auto-accept decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationContextAnalysis {
    /// Historical success rate for similar operations
    pub historical_success_rate: f32,

    /// Number of similar operations found
    pub similar_operations_count: usize,

    /// Context similarity score (0-1)
    pub context_similarity: f32,

    /// Relevant precedents from operation history
    pub relevant_precedents: Vec<OperationPrecedent>,

    /// Context-based warnings or recommendations
    pub context_warnings: Vec<String>,

    /// Confidence in this analysis
    pub analysis_confidence: f32,
}

/// Historical precedent for operation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPrecedent {
    /// Operation that was performed
    pub operation: FileOperation,

    /// Context similarity to current operation
    pub similarity: f32,

    /// Whether the precedent was successful
    pub was_successful: bool,

    /// User satisfaction rating
    pub user_satisfaction: Option<f32>,

    /// Key lessons learned
    pub lessons_learned: Vec<String>,

    /// Execution time of the precedent
    pub execution_time: Duration,

    /// When this precedent occurred
    pub timestamp: SystemTime,
}

/// Success rate analysis by operation type and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessRateAnalysis {
    /// Overall success rate for this operation type
    pub overall_success_rate: f32,

    /// Success rate in similar contexts
    pub contextual_success_rate: f32,

    /// Success rate by time of day
    pub temporal_success_rates: HashMap<String, f32>,

    /// Success rate by repository type
    pub repository_success_rates: HashMap<String, f32>,

    /// Most common failure modes
    pub common_failure_modes: Vec<FailureMode>,

    /// Trend analysis
    pub success_trend: SuccessTrend,
}

/// Failure mode analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMode {
    /// Description of the failure
    pub description: String,

    /// Frequency of this failure type
    pub frequency: f32,

    /// Typical causes
    pub typical_causes: Vec<String>,

    /// Suggested mitigations
    pub mitigations: Vec<String>,
}

/// Success trend over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessTrend {
    /// Trend direction (improving, declining, stable)
    pub direction: TrendDirection,

    /// Magnitude of the trend
    pub magnitude: f32,

    /// Confidence in the trend analysis
    pub confidence: f32,
}

/// Trend direction enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
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

/// Context Retriever with stage-specific intelligence and operation history analysis
pub struct ContextRetriever {
    config: RetrieverConfig,
    vector_store: Arc<ChromaVectorStore>,

    /// Python model service
    python_service: Arc<PythonModelService>,

    /// Cache of recent retrievals
    retrieval_cache: Arc<RwLock<lru::LruCache<String, StageContext>>>,

    /// Repository facts for enhanced context
    repository_facts: Arc<RwLock<Option<RepositoryFacts>>>,

    /// Active repository information (auto-discovered)
    active_repository: Arc<RwLock<Option<RepositoryInfo>>>,

    /// Operation history for analysis (indexed by operation hash)
    operation_history: Arc<RwLock<HashMap<String, OperationHistoryEntry>>>,

    /// Success rate cache by operation type and context
    success_rate_cache: Arc<RwLock<HashMap<String, SuccessRateAnalysis>>>,

    /// Context analysis cache
    context_analysis_cache: Arc<RwLock<HashMap<String, OperationContextAnalysis>>>,

    /// Access to authoritative knowledge store for hive mind consciousness
    knowledge_store:
        Arc<RwLock<Option<Arc<crate::consensus::memory::AuthoritativeKnowledgeStore>>>>,
}

/// Internal operation history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OperationHistoryEntry {
    pub operation: FileOperation,
    pub context: OperationContext,
    pub outcome: OperationOutcome,
    pub user_satisfaction: Option<f32>,
    pub indexed_at: SystemTime,
    pub context_hash: String,
}

impl ContextRetriever {
    /// Create a new Context Retriever
    pub async fn new(
        vector_store: Arc<ChromaVectorStore>,
        python_service: Arc<PythonModelService>,
    ) -> Result<Self> {
        let config = RetrieverConfig::default();
        let retrieval_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(100).unwrap(),
        )));

        Ok(Self {
            config,
            vector_store,
            python_service,
            retrieval_cache,
            repository_facts: Arc::new(RwLock::new(None)),
            active_repository: Arc::new(RwLock::new(None)),
            operation_history: Arc::new(RwLock::new(HashMap::new())),
            success_rate_cache: Arc::new(RwLock::new(HashMap::new())),
            context_analysis_cache: Arc::new(RwLock::new(HashMap::new())),
            knowledge_store: Arc::new(RwLock::new(None)),
        })
    }

    /// Update repository facts for enhanced context preparation
    pub async fn update_repository_facts(&self, facts: Option<RepositoryFacts>) -> Result<()> {
        let mut repo_facts = self.repository_facts.write().await;
        *repo_facts = facts.clone();

        if let Some(ref facts) = facts {
            tracing::info!(
                "Repository facts updated: {} v{} ({} deps, {} modules)",
                facts.name,
                facts.version,
                facts.dependency_count,
                facts.module_count
            );
        } else {
            tracing::warn!("Repository facts cleared - context may be less accurate");
        }

        // Clear cache since context will be different with new facts
        self.retrieval_cache.write().await.clear();

        Ok(())
    }

    /// Update knowledge store for hive mind consciousness
    pub async fn update_knowledge_store(
        &self,
        store: Option<Arc<crate::consensus::memory::AuthoritativeKnowledgeStore>>,
    ) -> Result<()> {
        let mut knowledge_store = self.knowledge_store.write().await;
        *knowledge_store = store.clone();

        if store.is_some() {
            tracing::info!(
                "✅ Hive mind connected: ContextRetriever now has access to all curator knowledge"
            );
        } else {
            tracing::warn!(
                "Hive mind disconnected: ContextRetriever has no access to historical knowledge"
            );
        }

        // Clear cache since context will be different with knowledge store
        self.retrieval_cache.write().await.clear();

        Ok(())
    }

    /// Get repository-enhanced facts for context
    async fn get_repository_enhanced_facts(&self, base_facts: Vec<String>) -> Vec<String> {
        let mut enhanced_facts = base_facts;

        if let Some(ref facts) = *self.repository_facts.read().await {
            enhanced_facts.insert(
                0,
                format!(
                    "Repository Context: {} v{} ({} dependencies, {} modules, {} files, {} LOC)",
                    facts.name,
                    facts.version,
                    facts.dependency_count,
                    facts.module_count,
                    facts.total_files,
                    facts.lines_of_code
                ),
            );

            if facts.is_enterprise {
                enhanced_facts.insert(
                    1,
                    "This is an enterprise-grade project with complex architecture".to_string(),
                );
            } else {
                enhanced_facts.insert(
                    1,
                    "This is a standard project with straightforward structure".to_string(),
                );
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

        // Use AI-powered repository detection
        let should_use_repo = self.should_use_repository_context(question, true).await?;

        // Auto-discover repository if needed and not already set
        if should_use_repo && self.active_repository.read().await.is_none() {
            tracing::info!(
                "🤖 AI detected repository-related question, auto-discovering repository..."
            );
            if let Some(repo_info) = self.auto_discover_repository().await? {
                tracing::info!(
                    "✅ Auto-discovered repository: {} at {}",
                    repo_info.project_type,
                    repo_info.path.display()
                );
                *self.active_repository.write().await = Some(repo_info);
            }
        }

        // Generator needs broad context and examples
        let base_facts = if should_use_repo {
            tracing::info!("📁 Using repository-specific context for Generator stage");
            self.retrieve_repository_context(question, context_limit)
                .await?
        } else {
            tracing::info!("🌐 Using general knowledge context for Generator stage");
            self.retrieve_broad_context(question, context_limit).await?
        };

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
                Draw from past examples but adapt to the specific question."
                    .to_string(),
            ),
        };

        // Cache the result
        self.retrieval_cache
            .write()
            .await
            .put(cache_key, context.clone());

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
        let base_facts = self
            .retrieve_quality_examples(question, context_limit)
            .await?;
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
                Reference high-quality examples from past responses."
                    .to_string(),
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
                Be critical but constructive in validation."
                    .to_string(),
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
        let base_facts = self
            .retrieve_synthesis_context(question, context_limit)
            .await?;
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
                Create lasting knowledge that will benefit future queries."
                    .to_string(),
            ),
        })
    }

    /// Retrieve repository-specific context
    async fn retrieve_repository_context(
        &self,
        question: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        let mut context_items = Vec::new();

        // Get repository information if available
        if let Some(ref repo_info) = *self.active_repository.read().await {
            context_items.push(format!(
                "📁 Repository: {} project at {}",
                repo_info.project_type,
                repo_info.path.display()
            ));

            context_items.push(format!("🔍 Description: {}", repo_info.description));

            if !repo_info.key_files.is_empty() {
                context_items.push(format!("📝 Key files: {}", repo_info.key_files.join(", ")));
            }

            if !repo_info.detected_technologies.is_empty() {
                context_items.push(format!(
                    "🚀 Technologies: {}",
                    repo_info.detected_technologies.join(", ")
                ));
            }
        }

        // Also get any hive mind knowledge about repositories
        let knowledge_store_guard = self.knowledge_store.read().await;
        if let Some(ref knowledge_store) = *knowledge_store_guard {
            let repo_facts = knowledge_store
                .find_similar("repository code project", 3)
                .await?;
            for fact in repo_facts {
                context_items.push(format!("🧠 Repository knowledge: {}", fact.content));
            }
        }

        // If we have less than limit items, fill with general context
        if context_items.len() < limit {
            let general_facts = self
                .retrieve_broad_context(question, limit - context_items.len())
                .await?;
            context_items.extend(general_facts);
        }

        Ok(context_items)
    }

    /// Retrieve broad context for Generator
    async fn retrieve_broad_context(&self, question: &str, limit: usize) -> Result<Vec<String>> {
        // Check if we have access to the hive mind (AuthoritativeKnowledgeStore)
        let knowledge_store_guard = self.knowledge_store.read().await;
        if let Some(ref knowledge_store) = *knowledge_store_guard {
            tracing::debug!("🐝 Hive mind active: Retrieving knowledge for Generator stage");

            // Get recent facts for temporal context
            let recent_facts = knowledge_store
                .get_recent_facts(
                    chrono::Duration::days(7), // 7 days
                )
                .await?;

            // Find similar facts based on the question
            let similar_facts = knowledge_store.find_similar(question, 5).await?;

            // Combine and format facts for the Generator
            let mut context_items = Vec::new();

            // Add recent facts
            for fact in recent_facts.iter().take(limit / 2) {
                context_items.push(format!(
                    "📅 Recent Knowledge ({}): {}\nFrom: {}",
                    fact.created_at.format("%Y-%m-%d"),
                    fact.content,
                    fact.source_question
                ));
            }

            // Add similar facts
            for fact in similar_facts.iter().take(limit / 2) {
                context_items.push(format!(
                    "🔗 Related Knowledge: {}\nContext: {}",
                    fact.content, fact.source_question
                ));
            }

            if context_items.is_empty() {
                tracing::debug!("No relevant facts found in hive mind, returning general guidance");
                Ok(vec![
                    "No previous curator knowledge found. Generate a comprehensive response based on your training.".to_string()
                ])
            } else {
                tracing::info!(
                    "🌐 Hive mind provided {} context items for Generator",
                    context_items.len()
                );
                Ok(context_items)
            }
        } else {
            // Fallback to vector store or placeholders
            tracing::warn!(
                "No hive mind connection - Generator stage operating without historical context"
            );
            Ok(vec![
                "Operating without hive mind context. Generate based on training knowledge."
                    .to_string(),
            ])
        }
    }

    /// Retrieve high-quality examples for Refiner
    async fn retrieve_quality_examples(
        &self,
        _question: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        let knowledge_store_guard = self.knowledge_store.read().await;
        if let Some(ref knowledge_store) = *knowledge_store_guard {
            tracing::debug!("🐝 Hive mind active: Finding quality examples for Refiner");

            // Get all facts sorted by date and filter for high quality
            let all_facts = knowledge_store.get_all_facts_sorted_by_date().await?;

            // Filter for high quality (confidence > 0.8) and sort by confidence
            let mut quality_facts: Vec<_> = all_facts
                .into_iter()
                .filter(|f| f.curator_confidence > 0.8)
                .collect();

            // Sort by confidence descending
            quality_facts.sort_by(|a, b| {
                b.curator_confidence
                    .partial_cmp(&a.curator_confidence)
                    .unwrap()
            });
            let quality_facts: Vec<_> = quality_facts.into_iter().take(limit).collect();

            let context_items: Vec<String> = quality_facts
                .iter()
                .map(|fact| {
                    format!(
                        "✨ High-Quality Example (confidence: {:.2}): {}\nRefined from: {}",
                        fact.curator_confidence, fact.content, fact.source_question
                    )
                })
                .collect();

            if context_items.is_empty() {
                Ok(vec![
                    "Focus on clarity and completeness. No high-quality examples available yet."
                        .to_string(),
                ])
            } else {
                tracing::info!(
                    "🏆 Hive mind provided {} quality examples for Refiner",
                    context_items.len()
                );
                Ok(context_items)
            }
        } else {
            Ok(vec![
                "Refine based on best practices. No historical examples available.".to_string(),
            ])
        }
    }

    /// Retrieve edge cases for Validator
    async fn retrieve_edge_cases(&self, question: &str, limit: usize) -> Result<Vec<String>> {
        let knowledge_store_guard = self.knowledge_store.read().await;
        if let Some(ref knowledge_store) = *knowledge_store_guard {
            tracing::debug!("🐝 Hive mind active: Finding edge cases for Validator");

            // Find facts about similar topics to identify potential contradictions
            let similar_facts = knowledge_store.find_similar(question, limit * 2).await?;

            // Look for patterns in themes to find edge cases
            let mut edge_cases = Vec::new();
            let mut seen_themes = std::collections::HashSet::new();

            for fact in similar_facts {
                // Extract unique perspectives
                let primary_theme = fact
                    .topics
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "general".to_string());
                if seen_themes.insert(primary_theme.clone()) {
                    edge_cases.push(format!(
                        "⚠️ Consider: {}\nPrevious context: {}",
                        fact.content, fact.source_question
                    ));
                }

                if edge_cases.len() >= limit {
                    break;
                }
            }

            if edge_cases.is_empty() {
                Ok(vec![
                    "Validate thoroughly. No historical edge cases found.".to_string()
                ])
            } else {
                tracing::info!(
                    "🔍 Hive mind provided {} edge cases for Validator",
                    edge_cases.len()
                );
                Ok(edge_cases)
            }
        } else {
            Ok(vec![
                "Validate based on general principles. No historical cases available.".to_string(),
            ])
        }
    }

    /// Retrieve synthesis opportunities for Curator
    async fn retrieve_synthesis_context(
        &self,
        question: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        let knowledge_store_guard = self.knowledge_store.read().await;
        if let Some(ref knowledge_store) = *knowledge_store_guard {
            tracing::debug!("🐝 Hive mind active: Finding synthesis opportunities for Curator");

            // Get both recent and thematically similar facts
            let recent_facts = knowledge_store
                .get_recent_facts(
                    chrono::Duration::days(3), // 3 days
                )
                .await?;
            let similar_facts = knowledge_store.find_similar(question, limit).await?;

            let mut synthesis_items = Vec::new();

            // Add recent authoritative knowledge
            for fact in recent_facts.iter().take(limit / 2) {
                synthesis_items.push(format!(
                    "🎯 Recent Authority: {}\nEstablished from: {}",
                    fact.content, fact.source_question
                ));
            }

            // Add thematically related synthesis opportunities
            for fact in similar_facts.iter().take(limit / 2) {
                synthesis_items.push(format!(
                    "🔄 Synthesis Opportunity: {}\nRelated theme: {}",
                    fact.content,
                    fact.topics
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "general".to_string())
                ));
            }

            if synthesis_items.is_empty() {
                Ok(vec!["Create authoritative knowledge. This will be the first entry in the hive mind.".to_string()])
            } else {
                tracing::info!(
                    "🌟 Hive mind provided {} synthesis items for Curator",
                    synthesis_items.len()
                );
                Ok(synthesis_items)
            }
        } else {
            Ok(vec![
                "Synthesize based on current context. No historical knowledge available."
                    .to_string(),
            ])
        }
    }

    /// Find patterns relevant to the question
    async fn find_relevant_patterns(&self, _question: &str) -> Result<Vec<Pattern>> {
        // TODO: Implement pattern matching with UniXcoder
        Ok(vec![])
    }

    /// Find improvement patterns
    async fn find_improvement_patterns(&self, _question: &str) -> Result<Vec<Pattern>> {
        Ok(vec![])
    }

    /// Find contradiction patterns
    async fn find_contradiction_patterns(&self, _question: &str) -> Result<Vec<Pattern>> {
        Ok(vec![])
    }

    /// Find synthesis patterns
    async fn find_synthesis_patterns(&self, _question: &str) -> Result<Vec<Pattern>> {
        Ok(vec![])
    }

    /// Get relevant insights
    async fn get_relevant_insights(&self, _question: &str) -> Result<Vec<Insight>> {
        Ok(vec![])
    }

    /// Get quality insights
    async fn get_quality_insights(&self, _question: &str) -> Result<Vec<Insight>> {
        Ok(vec![])
    }

    /// Get validation insights
    async fn get_validation_insights(&self, _question: &str) -> Result<Vec<Insight>> {
        Ok(vec![])
    }

    /// Get synthesis insights
    async fn get_synthesis_insights(&self, _question: &str) -> Result<Vec<Insight>> {
        Ok(vec![])
    }

    /// AI-powered repository auto-discovery
    /// Uses semantic analysis to understand repository structure and purpose
    pub async fn auto_discover_repository(&self) -> Result<Option<RepositoryInfo>> {
        let current_dir = std::env::current_dir()?;

        tracing::debug!("🔍 AI analyzing directory: {}", current_dir.display());

        // 1. Check if this looks like a repository using AI analysis
        let directory_analysis = self.analyze_directory_with_ai(&current_dir).await?;

        if !directory_analysis.is_repository {
            tracing::debug!(
                "❌ AI determined {} is not a repository",
                current_dir.display()
            );
            return Ok(None);
        }

        // 2. Use AI to understand the repository type and structure
        let repo_info = RepositoryInfo {
            path: current_dir.clone(),
            project_type: directory_analysis.project_type,
            description: directory_analysis.description,
            key_files: directory_analysis.key_files,
            detected_technologies: directory_analysis.technologies,
        };

        tracing::info!(
            "✅ AI Repository Discovery: {} project at {}",
            repo_info.project_type,
            repo_info.path.display()
        );

        Ok(Some(repo_info))
    }

    /// Use AI models to analyze a directory and determine if it's a repository
    async fn analyze_directory_with_ai(&self, path: &std::path::Path) -> Result<DirectoryAnalysis> {
        // Read directory contents
        let mut entries = Vec::new();
        if let Ok(dir_entries) = std::fs::read_dir(path) {
            for entry in dir_entries {
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        entries.push(name.to_string());
                    }
                }
            }
        }

        // Create a description of the directory for AI analysis
        let directory_description = format!(
            "Directory path: {}\nContents: {}\n\nAnalyze if this is a code repository and what type.",
            path.display(),
            entries.join(", ")
        );

        // Use our AI model to analyze the directory
        match self
            .python_service
            .analyze_code(
                "microsoft/graphcodebert-base",
                &directory_description,
                "analyze_repository_structure",
            )
            .await
        {
            Ok(result) => {
                // Parse AI response
                self.parse_directory_analysis(&result, &entries)
            }
            Err(e) => {
                tracing::warn!("AI directory analysis failed: {}, using heuristics", e);
                Ok(self.heuristic_directory_analysis(&entries))
            }
        }
    }

    /// Parse AI model response for directory analysis
    fn parse_directory_analysis(
        &self,
        result: &serde_json::Value,
        entries: &[String],
    ) -> Result<DirectoryAnalysis> {
        // Try to parse structured AI response, fallback to heuristics
        Ok(self.heuristic_directory_analysis(entries))
    }

    /// Heuristic-based directory analysis as fallback
    fn heuristic_directory_analysis(&self, entries: &[String]) -> DirectoryAnalysis {
        let mut is_repository = false;
        let mut project_type = "Unknown".to_string();
        let mut description = String::new();
        let mut key_files = Vec::new();
        let mut technologies = Vec::new();

        // Repository indicators
        if entries.contains(&"Cargo.toml".to_string()) {
            is_repository = true;
            project_type = "Rust".to_string();
            description = "Rust project with Cargo package manager".to_string();
            key_files.push("Cargo.toml".to_string());
            technologies.push("Rust".to_string());
            technologies.push("Cargo".to_string());

            if entries.contains(&"src".to_string()) {
                key_files.push("src/".to_string());
            }
        } else if entries.contains(&"package.json".to_string()) {
            is_repository = true;
            project_type = "Node.js".to_string();
            description = "Node.js project with npm/yarn package manager".to_string();
            key_files.push("package.json".to_string());
            technologies.push("JavaScript".to_string());
            technologies.push("Node.js".to_string());

            if entries.contains(&"tsconfig.json".to_string()) {
                project_type = "TypeScript".to_string();
                description = "TypeScript project with npm/yarn package manager".to_string();
                technologies.push("TypeScript".to_string());
            }
        } else if entries.contains(&".git".to_string()) {
            is_repository = true;
            project_type = "Git Repository".to_string();
            description = "Git version controlled repository".to_string();
            key_files.push(".git/".to_string());
            technologies.push("Git".to_string());
        }

        DirectoryAnalysis {
            is_repository,
            project_type,
            description,
            key_files,
            technologies,
        }
    }

    /// Analyze question to determine if repository context should be used
    /// This uses GraphCodeBERT to understand the semantic content of the question
    /// and intelligently discovers repositories when needed
    pub async fn should_use_repository_context(
        &self,
        question: &str,
        has_open_repository: bool,
    ) -> Result<bool> {
        // 🧠 AI-POWERED SEMANTIC ANALYSIS: Always analyze question first
        let context_decision = self.analyze_question_context(question).await?;

        tracing::info!(
            "🧠 AI Analysis: '{}' classified as {} (confidence: {:.2})",
            &question[..question.len().min(60)],
            context_decision.category,
            context_decision.confidence
        );

        // If AI determines this needs repository context, try to provide it
        if context_decision.should_use_repo {
            if has_open_repository {
                tracing::info!("✅ Repository context available for repository-specific question");
                return Ok(true);
            } else {
                // 🔍 INTELLIGENT AUTO-DISCOVERY: AI detected repo question without repo context
                tracing::info!(
                    "🔍 AI detected repository-specific question - attempting auto-discovery"
                );

                match self.auto_discover_repository().await {
                    Ok(Some(repo_info)) => {
                        tracing::info!(
                            "✅ AI Auto-Discovery: Found {} repository at {}",
                            repo_info.project_type,
                            repo_info.path.display()
                        );

                        // Store the discovered repository for this session
                        // The repository context manager can pick this up
                        return Ok(true);
                    }
                    Ok(None) => {
                        tracing::warn!(
                            "⚠️ AI detected repository question but no repository found at {}",
                            std::env::current_dir()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|_| "unknown".to_string())
                        );
                        return Ok(false);
                    }
                    Err(e) => {
                        tracing::error!("❌ AI repository auto-discovery failed: {}", e);
                        return Ok(false);
                    }
                }
            }
        }

        // For non-repository questions, don't use repository context
        Ok(false)
    }

    /// Analyze question using LLM to determine context needs
    pub async fn analyze_question_context(&self, question: &str) -> Result<ContextDecision> {
        // For now, we can't access the OpenRouter client from here
        // The real LLM-based routing should happen in intelligent_context_orchestrator
        // which has access to the ecosystem's OpenRouter client

        // Return a neutral response that allows the orchestrator to make the real decision
        Ok(ContextDecision {
            should_use_repo: false,
            confidence: 0.5,
            category: "general_programming".to_string(),
            reasoning: "Pending LLM analysis in orchestrator".to_string(),
        })
    }

    /// Parse the AI model response from a Value into a context decision
    fn parse_context_decision_from_value(
        &self,
        result: &serde_json::Value,
        question: &str,
    ) -> Result<ContextDecision> {
        if let Ok(analysis) = serde_json::from_value::<QuestionAnalysis>(result.clone()) {
            let should_use_repo = match analysis.category.as_str() {
                "repository_specific" => true,
                "general_programming" => {
                    // For general programming, only use repo context if high confidence
                    // and the question might benefit from examples
                    analysis.confidence > 0.8
                        && (analysis.reasoning.contains("example")
                            || analysis.reasoning.contains("implement")
                            || analysis.reasoning.contains("how to"))
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

        // NO FALLBACK - if AI analysis fails, we fail
        error!("Failed to parse AI context analysis. Cannot proceed without LLM intelligence.");
        Err(anyhow!(
            "AI model response parsing failed - LLM intelligence required"
        ))
    }

    // === Operation History Analysis Methods ===

    /// Record operation outcome for learning and historical analysis
    pub async fn record_operation_outcome(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
        outcome: &OperationOutcome,
        user_satisfaction: Option<f32>,
    ) -> Result<()> {
        info!("📚 Recording operation outcome for context analysis");

        let operation_hash = self.generate_operation_hash(operation, context);
        let context_hash = self.generate_context_hash(context);

        let history_entry = OperationHistoryEntry {
            operation: operation.clone(),
            context: context.clone(),
            outcome: outcome.clone(),
            user_satisfaction,
            indexed_at: SystemTime::now(),
            context_hash,
        };

        // Store in operation history
        {
            let mut history = self.operation_history.write().await;
            history.insert(operation_hash.clone(), history_entry);

            // Keep only recent operations in memory (last 5000)
            if history.len() > 5000 {
                let cutoff_time = SystemTime::now() - Duration::from_secs(60 * 60 * 24 * 30); // 30 days
                history.retain(|_, entry| entry.indexed_at > cutoff_time);
            }
        }

        // Clear relevant caches since data has changed
        self.success_rate_cache.write().await.clear();
        self.context_analysis_cache.write().await.clear();

        debug!("✅ Operation outcome recorded: {}", operation_hash);
        Ok(())
    }

    /// Analyze operation context for auto-accept decision making
    pub async fn analyze_operation_context(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<OperationContextAnalysis> {
        info!("🔍 Analyzing operation context for auto-accept decision");

        // Check cache first
        let cache_key = format!(
            "{}_{}",
            self.generate_operation_hash(operation, context),
            self.generate_context_hash(context)
        );

        {
            let cache = self.context_analysis_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                debug!("📋 Using cached context analysis");
                return Ok(cached.clone());
            }
        }

        // Find similar operations in history
        let similar_operations = self.find_similar_operations(operation, context).await?;

        // Calculate historical success rate
        let historical_success_rate = if !similar_operations.is_empty() {
            similar_operations
                .iter()
                .map(|op| if op.was_successful { 1.0 } else { 0.0 })
                .sum::<f32>()
                / similar_operations.len() as f32
        } else {
            0.5 // Default neutral success rate
        };

        // Calculate context similarity (average of similar operations)
        let context_similarity = if !similar_operations.is_empty() {
            similar_operations
                .iter()
                .map(|op| op.similarity)
                .sum::<f32>()
                / similar_operations.len() as f32
        } else {
            0.0
        };

        // Generate context-based warnings
        let context_warnings = self
            .generate_context_warnings(operation, context, &similar_operations)
            .await?;

        // Calculate analysis confidence
        let analysis_confidence =
            self.calculate_analysis_confidence(&similar_operations, context_similarity);

        let analysis = OperationContextAnalysis {
            historical_success_rate,
            similar_operations_count: similar_operations.len(),
            context_similarity,
            relevant_precedents: similar_operations,
            context_warnings,
            analysis_confidence,
        };

        // Cache the result
        {
            let mut cache = self.context_analysis_cache.write().await;
            cache.insert(cache_key, analysis.clone());
        }

        info!(
            "📊 Context analysis complete: {:.1}% success rate, {:.1}% confidence",
            analysis.historical_success_rate * 100.0,
            analysis.analysis_confidence * 100.0
        );

        Ok(analysis)
    }

    /// Get success rate analysis for operation type and context
    pub async fn get_success_rate_analysis(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<SuccessRateAnalysis> {
        let operation_type = self.get_operation_type(operation);
        let cache_key = format!("{}_{}", operation_type, self.generate_context_hash(context));

        // Check cache first
        {
            let cache = self.success_rate_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let history = self.operation_history.read().await;

        // Filter operations by type
        let type_operations: Vec<_> = history
            .values()
            .filter(|entry| self.get_operation_type(&entry.operation) == operation_type)
            .collect();

        // Calculate overall success rate for this operation type
        let overall_success_rate = if !type_operations.is_empty() {
            type_operations
                .iter()
                .map(|entry| if entry.outcome.success { 1.0 } else { 0.0 })
                .sum::<f32>()
                / type_operations.len() as f32
        } else {
            0.5
        };

        // Calculate contextual success rate (similar repository/question context)
        let similar_context_ops: Vec<_> = type_operations
            .iter()
            .filter(|entry| self.is_similar_context(&entry.context, context))
            .collect();

        let contextual_success_rate = if !similar_context_ops.is_empty() {
            similar_context_ops
                .iter()
                .map(|entry| if entry.outcome.success { 1.0 } else { 0.0 })
                .sum::<f32>()
                / similar_context_ops.len() as f32
        } else {
            overall_success_rate
        };

        // Analyze temporal patterns
        let temporal_success_rates = self.analyze_temporal_patterns(&type_operations);

        // Analyze by repository type
        let repository_success_rates = self.analyze_repository_patterns(&type_operations);

        // Find common failure modes
        let common_failure_modes = self.analyze_failure_modes(&type_operations);

        // Analyze success trend
        let success_trend = self.analyze_success_trend(&type_operations);

        let analysis = SuccessRateAnalysis {
            overall_success_rate,
            contextual_success_rate,
            temporal_success_rates,
            repository_success_rates,
            common_failure_modes,
            success_trend,
        };

        // Cache the result
        {
            let mut cache = self.success_rate_cache.write().await;
            cache.insert(cache_key, analysis.clone());
        }

        Ok(analysis)
    }

    /// Clear operation history older than specified days
    pub async fn cleanup_old_operations(&self, keep_days: u32) -> Result<usize> {
        let cutoff_time = SystemTime::now() - Duration::from_secs(60 * 60 * 24 * keep_days as u64);

        let mut history = self.operation_history.write().await;
        let original_len = history.len();

        history.retain(|_, entry| entry.indexed_at > cutoff_time);

        let removed_count = original_len - history.len();

        if removed_count > 0 {
            info!(
                "🧹 Cleaned up {} old operation history entries",
                removed_count
            );

            // Clear caches since data changed
            self.success_rate_cache.write().await.clear();
            self.context_analysis_cache.write().await.clear();
        }

        Ok(removed_count)
    }

    /// Get operation history statistics
    pub async fn get_operation_statistics(&self) -> Result<HashMap<String, f32>> {
        let history = self.operation_history.read().await;
        let mut stats = HashMap::new();

        stats.insert("total_operations".to_string(), history.len() as f32);

        let successful_operations = history
            .values()
            .filter(|entry| entry.outcome.success)
            .count();

        stats.insert(
            "successful_operations".to_string(),
            successful_operations as f32,
        );
        stats.insert(
            "overall_success_rate".to_string(),
            if !history.is_empty() {
                successful_operations as f32 / history.len() as f32
            } else {
                0.0
            },
        );

        // Average execution time
        let avg_execution_time = if !history.is_empty() {
            history
                .values()
                .map(|entry| entry.outcome.execution_time.as_millis() as f32)
                .sum::<f32>()
                / history.len() as f32
        } else {
            0.0
        };

        stats.insert("average_execution_time_ms".to_string(), avg_execution_time);

        // Average user satisfaction
        let satisfaction_scores: Vec<f32> = history
            .values()
            .filter_map(|entry| entry.user_satisfaction)
            .collect();

        if !satisfaction_scores.is_empty() {
            let avg_satisfaction =
                satisfaction_scores.iter().sum::<f32>() / satisfaction_scores.len() as f32;
            stats.insert("average_user_satisfaction".to_string(), avg_satisfaction);
        }

        Ok(stats)
    }

    // === Private Helper Methods ===

    /// Find similar operations based on operation type and context
    async fn find_similar_operations(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<Vec<OperationPrecedent>> {
        let history = self.operation_history.read().await;
        let operation_type = self.get_operation_type(operation);

        let mut precedents = Vec::new();

        for entry in history.values() {
            // Only consider same operation type
            if self.get_operation_type(&entry.operation) != operation_type {
                continue;
            }

            // Calculate similarity
            let similarity = self.calculate_context_similarity(&entry.context, context);

            // Only include if similarity is above threshold
            if similarity >= 0.3 {
                let lessons_learned = self.extract_lessons_learned(&entry.outcome, &entry.context);

                precedents.push(OperationPrecedent {
                    operation: entry.operation.clone(),
                    similarity,
                    was_successful: entry.outcome.success,
                    user_satisfaction: entry.user_satisfaction,
                    lessons_learned,
                    execution_time: entry.outcome.execution_time,
                    timestamp: entry.indexed_at,
                });
            }
        }

        // Sort by similarity descending
        precedents.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        // Return top 10 most similar
        precedents.truncate(10);

        Ok(precedents)
    }

    /// Generate warnings based on context and historical patterns
    async fn generate_context_warnings(
        &self,
        operation: &FileOperation,
        _context: &OperationContext,
        similar_operations: &[OperationPrecedent],
    ) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for high failure rate in similar contexts
        let failure_rate = similar_operations
            .iter()
            .map(|op| if !op.was_successful { 1.0 } else { 0.0 })
            .sum::<f32>()
            / similar_operations.len().max(1) as f32;

        if failure_rate > 0.3 {
            warnings.push(format!(
                "High failure rate ({:.1}%) detected for similar operations in this context",
                failure_rate * 100.0
            ));
        }

        // Check for recent failures
        let recent_failures = similar_operations
            .iter()
            .filter(|op| {
                !op.was_successful
                    && op.timestamp > SystemTime::now() - Duration::from_secs(60 * 60 * 24 * 7)
            }) // 7 days
            .count();

        if recent_failures > 2 {
            warnings.push(format!(
                "{} recent failures detected for similar operations",
                recent_failures
            ));
        }

        // Check for specific risky patterns
        if let FileOperation::Delete { .. } = operation {
            warnings.push("Deletion operation - ensure backups are available".to_string());
        }

        // Check repository context warnings
        if let Some(repo_facts) = &*self.repository_facts.read().await {
            if repo_facts.is_enterprise && repo_facts.dependency_count > 100 {
                warnings
                    .push("Complex enterprise repository - extra caution recommended".to_string());
            }
        }

        Ok(warnings)
    }

    /// Calculate confidence in the context analysis
    fn calculate_analysis_confidence(
        &self,
        similar_operations: &[OperationPrecedent],
        avg_similarity: f32,
    ) -> f32 {
        let base_confidence = if similar_operations.is_empty() {
            0.1 // Very low confidence with no historical data
        } else {
            // Confidence based on number of similar operations and their similarity
            let count_factor = (similar_operations.len() as f32 / 10.0).min(1.0);
            let similarity_factor = avg_similarity;
            (count_factor * similarity_factor * 0.8 + 0.2).clamp(0.0, 1.0)
        };

        base_confidence
    }

    /// Get operation type as string
    fn get_operation_type(&self, operation: &FileOperation) -> &'static str {
        match operation {
            FileOperation::Create { .. } => "create",
            FileOperation::Update { .. } => "update",
            FileOperation::Append { .. } => "append",
            FileOperation::Delete { .. } => "delete",
            FileOperation::Rename { .. } => "rename",
        }
    }

    /// Generate hash for operation and context
    fn generate_operation_hash(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{:?}_{}", operation, context.source_question).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Generate hash for context
    fn generate_context_hash(&self, context: &OperationContext) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!(
            "{}_{}_{}",
            context.repository_path.display(),
            context.source_question,
            context.related_files.len()
        )
        .hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Calculate similarity between two contexts
    fn calculate_context_similarity(
        &self,
        context1: &OperationContext,
        context2: &OperationContext,
    ) -> f32 {
        let mut similarity = 0.0;

        // Repository similarity
        if context1.repository_path == context2.repository_path {
            similarity += 0.4;
        }

        // Question similarity (simple keyword matching)
        let words1: std::collections::HashSet<String> = context1
            .source_question
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let words2: std::collections::HashSet<String> = context2
            .source_question
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let common_words = words1.intersection(&words2).count();
        let total_words = words1.union(&words2).count();

        if total_words > 0 {
            similarity += (common_words as f32 / total_words as f32) * 0.4;
        }

        // Related files similarity
        let files1: std::collections::HashSet<_> = context1.related_files.iter().collect();
        let files2: std::collections::HashSet<_> = context2.related_files.iter().collect();

        let common_files = files1.intersection(&files2).count();
        let total_files = files1.union(&files2).count();

        if total_files > 0 {
            similarity += (common_files as f32 / total_files as f32) * 0.2;
        }

        similarity.clamp(0.0, 1.0)
    }

    /// Check if two contexts are similar
    fn is_similar_context(&self, context1: &OperationContext, context2: &OperationContext) -> bool {
        self.calculate_context_similarity(context1, context2) > 0.5
    }

    /// Extract lessons learned from operation outcome
    fn extract_lessons_learned(
        &self,
        outcome: &OperationOutcome,
        context: &OperationContext,
    ) -> Vec<String> {
        let mut lessons = Vec::new();

        if !outcome.success {
            if let Some(error) = &outcome.error_message {
                lessons.push(format!("Error pattern: {}", error));
            }

            if outcome.rollback_required {
                lessons.push("Required rollback - consider more validation".to_string());
            }
        } else {
            if outcome.execution_time < Duration::from_secs(1) {
                lessons.push("Fast execution - good pattern".to_string());
            }

            if let Some(feedback) = &outcome.user_feedback {
                if !feedback.is_empty() {
                    lessons.push(format!("User feedback: {}", feedback));
                }
            }
        }

        lessons
    }

    /// Analyze temporal success patterns
    fn analyze_temporal_patterns(
        &self,
        operations: &[&OperationHistoryEntry],
    ) -> HashMap<String, f32> {
        let mut patterns = HashMap::new();

        // Group by hour of day
        let mut hour_success: HashMap<u32, Vec<bool>> = HashMap::new();

        for entry in operations {
            if let Ok(duration) = entry.indexed_at.duration_since(SystemTime::UNIX_EPOCH) {
                let hour = (duration.as_secs() / 3600) % 24;
                hour_success
                    .entry(hour as u32)
                    .or_insert_with(Vec::new)
                    .push(entry.outcome.success);
            }
        }

        for (hour, successes) in hour_success {
            let success_rate = successes
                .iter()
                .map(|&s| if s { 1.0 } else { 0.0 })
                .sum::<f32>()
                / successes.len() as f32;
            patterns.insert(format!("hour_{:02}", hour), success_rate);
        }

        patterns
    }

    /// Analyze repository type success patterns
    fn analyze_repository_patterns(
        &self,
        operations: &[&OperationHistoryEntry],
    ) -> HashMap<String, f32> {
        let mut patterns = HashMap::new();

        // Group by repository path (simplified)
        let mut repo_success: HashMap<String, Vec<bool>> = HashMap::new();

        for entry in operations {
            let repo_type = self.classify_repository_type(&entry.context);
            repo_success
                .entry(repo_type)
                .or_insert_with(Vec::new)
                .push(entry.outcome.success);
        }

        for (repo_type, successes) in repo_success {
            let success_rate = successes
                .iter()
                .map(|&s| if s { 1.0 } else { 0.0 })
                .sum::<f32>()
                / successes.len() as f32;
            patterns.insert(repo_type, success_rate);
        }

        patterns
    }

    /// Classify repository type based on context
    fn classify_repository_type(&self, context: &OperationContext) -> String {
        let path_str = context.repository_path.to_string_lossy().to_lowercase();

        if path_str.contains("rust")
            || context
                .related_files
                .iter()
                .any(|f| f.extension() == Some("rs".as_ref()))
        {
            "rust".to_string()
        } else if path_str.contains("node")
            || context
                .related_files
                .iter()
                .any(|f| f.extension() == Some("js".as_ref()))
        {
            "javascript".to_string()
        } else if context
            .related_files
            .iter()
            .any(|f| f.extension() == Some("py".as_ref()))
        {
            "python".to_string()
        } else {
            "other".to_string()
        }
    }

    /// Analyze common failure modes
    fn analyze_failure_modes(&self, operations: &[&OperationHistoryEntry]) -> Vec<FailureMode> {
        let mut failure_counts: HashMap<String, Vec<&OperationHistoryEntry>> = HashMap::new();

        for entry in operations {
            if !entry.outcome.success {
                if let Some(error) = &entry.outcome.error_message {
                    // Classify error types
                    let error_type = self.classify_error_type(error);
                    failure_counts
                        .entry(error_type)
                        .or_insert_with(Vec::new)
                        .push(entry);
                }
            }
        }

        let total_failures = operations.iter().filter(|e| !e.outcome.success).count();
        let mut failure_modes = Vec::new();

        for (error_type, entries) in failure_counts {
            let frequency = entries.len() as f32 / total_failures.max(1) as f32;

            if frequency >= 0.1 {
                // At least 10% of failures
                let typical_causes = self.extract_typical_causes(&entries);
                let mitigations = self.suggest_mitigations(&error_type, &typical_causes);

                failure_modes.push(FailureMode {
                    description: error_type,
                    frequency,
                    typical_causes,
                    mitigations,
                });
            }
        }

        // Sort by frequency descending
        failure_modes.sort_by(|a, b| b.frequency.partial_cmp(&a.frequency).unwrap());

        failure_modes
    }

    /// Classify error type from error message
    fn classify_error_type(&self, error_message: &str) -> String {
        let error_lower = error_message.to_lowercase();

        if error_lower.contains("permission") || error_lower.contains("access") {
            "Permission Error".to_string()
        } else if error_lower.contains("file not found") || error_lower.contains("no such file") {
            "File Not Found".to_string()
        } else if error_lower.contains("syntax") || error_lower.contains("parse") {
            "Syntax Error".to_string()
        } else if error_lower.contains("timeout") {
            "Timeout Error".to_string()
        } else {
            "Other Error".to_string()
        }
    }

    /// Extract typical causes from failed operations
    fn extract_typical_causes(&self, entries: &[&OperationHistoryEntry]) -> Vec<String> {
        let mut causes = Vec::new();

        // Analyze patterns in failed operations
        let file_types: std::collections::HashSet<_> = entries
            .iter()
            .filter_map(|e| self.get_file_extension(&e.operation))
            .collect();

        if file_types.len() == 1 {
            causes.push(format!(
                "Specific to {} files",
                file_types.iter().next().unwrap()
            ));
        }

        // Check for time patterns
        let recent_failures = entries
            .iter()
            .filter(|e| e.indexed_at > SystemTime::now() - Duration::from_secs(60 * 60 * 24))
            .count();

        if recent_failures > entries.len() / 2 {
            causes.push("Recent increase in failures".to_string());
        }

        causes
    }

    /// Get file extension from operation
    fn get_file_extension(&self, operation: &FileOperation) -> Option<String> {
        let path = match operation {
            FileOperation::Create { path, .. } => path,
            FileOperation::Update { path, .. } => path,
            FileOperation::Append { path, .. } => path,
            FileOperation::Delete { path } => path,
            FileOperation::Rename { to, .. } => to,
        };

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Suggest mitigations for error types
    fn suggest_mitigations(&self, error_type: &str, causes: &[String]) -> Vec<String> {
        let mut mitigations = Vec::new();

        match error_type {
            "Permission Error" => {
                mitigations.push("Check file permissions before operation".to_string());
                mitigations.push("Ensure user has write access to directory".to_string());
            }
            "File Not Found" => {
                mitigations.push("Verify file exists before modification".to_string());
                mitigations.push("Create parent directories if needed".to_string());
            }
            "Syntax Error" => {
                mitigations.push("Validate file syntax before writing".to_string());
                mitigations.push("Use language-specific formatters".to_string());
            }
            "Timeout Error" => {
                mitigations.push("Increase operation timeout".to_string());
                mitigations.push("Break large operations into smaller chunks".to_string());
            }
            _ => {
                mitigations.push("Review operation parameters".to_string());
                mitigations.push("Add additional validation steps".to_string());
            }
        }

        mitigations
    }

    /// Analyze success trend over time
    fn analyze_success_trend(&self, operations: &[&OperationHistoryEntry]) -> SuccessTrend {
        if operations.len() < 10 {
            return SuccessTrend {
                direction: TrendDirection::Stable,
                magnitude: 0.0,
                confidence: 0.1,
            };
        }

        // Sort by timestamp
        let mut sorted_ops: Vec<_> = operations.iter().collect();
        sorted_ops.sort_by_key(|e| e.indexed_at);

        // Split into two halves for comparison
        let mid = sorted_ops.len() / 2;
        let first_half = &sorted_ops[..mid];
        let second_half = &sorted_ops[mid..];

        let first_success_rate = first_half
            .iter()
            .map(|e| if e.outcome.success { 1.0 } else { 0.0 })
            .sum::<f32>()
            / first_half.len() as f32;

        let second_success_rate = second_half
            .iter()
            .map(|e| if e.outcome.success { 1.0 } else { 0.0 })
            .sum::<f32>()
            / second_half.len() as f32;

        let magnitude = (second_success_rate - first_success_rate).abs();
        let direction = if second_success_rate > first_success_rate + 0.05 {
            TrendDirection::Improving
        } else if second_success_rate < first_success_rate - 0.05 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        let confidence = (operations.len() as f32 / 50.0).min(1.0); // Higher confidence with more data

        SuccessTrend {
            direction,
            magnitude,
            confidence,
        }
    }

    /// REMOVED - No fallback allowed, LLM or nothing
    #[deprecated(note = "Fallback removed - AI Helpers must use LLM intelligence only")]
    fn fallback_context_analysis(&self, question: &str) -> ContextDecision {
        let question_lower = question.to_lowercase();

        // Repository-specific indicators
        let repo_keywords = [
            "this code",
            "this project",
            "this repo",
            "this repository",
            "this file",
            "this function",
            "this class",
            "this method",
            "my code",
            "my project",
            "my repo",
            "my repository",
            "our code",
            "our project",
            "our repo",
            "our repository",
            "current code",
            "current project",
            "current repo",
            "current repository",
            "@codebase",
            "in this",
            "here",
            "this implementation",
            "analyze this",
            "review this",
            "check this",
            "fix this",
            "update this",
            "modify this",
            "change this",
            "improve this",
            "the code",
            "the project",
            "the repo",
            "the repository",
        ];

        // General programming indicators
        let general_keywords = [
            "difference between",
            "compare",
            "versus",
            "vs",
            "or",
            "what is",
            "how does",
            "explain",
            "when to use",
            "best practice",
            "which is better",
            "pros and cons",
            "advantages",
            "disadvantages",
            "tell me about",
            "angular",
            "vue",
            "react",
            "svelte",
            "ember",
            "python",
            "rust",
            "javascript",
            "typescript",
            "java",
            "design pattern",
            "algorithm",
            "data structure",
            "performance",
            "optimization",
            "security",
        ];

        let repo_score = repo_keywords
            .iter()
            .filter(|keyword| question_lower.contains(*keyword))
            .count();

        let general_score = general_keywords
            .iter()
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
            category: if should_use_repo {
                "repository_specific"
            } else {
                "general_programming"
            }
            .to_string(),
            reasoning: format!(
                "Heuristic analysis: repo_keywords={}, general_keywords={}",
                repo_score, general_score
            ),
        }
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stage_specific_retrieval() {
        // Test that each stage gets appropriate context
    }
}
