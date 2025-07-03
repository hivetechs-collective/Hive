//! Context retrieval engine for intelligent memory access
//! 
//! This module provides:
//! - Multiple retrieval strategies (semantic, keyword, hybrid)
//! - Context window management
//! - Relevance scoring and ranking
//! - Adaptive retrieval based on query type

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info};

use crate::core::memory::{SemanticSearchResult, MemorySystem};
use crate::core::database::KnowledgeConversation;
use crate::memory::embeddings::{EmbeddingEngine, SimilarityMetric};

/// Context retrieval engine
#[derive(Debug)]
pub struct ContextRetriever {
    /// Embedding engine for semantic search
    embedding_engine: Arc<EmbeddingEngine>,
    /// Retrieval configuration
    config: RetrievalConfig,
    /// Query cache for performance
    cache: Arc<tokio::sync::RwLock<QueryCache>>,
    /// Retrieval statistics
    stats: Arc<tokio::sync::RwLock<RetrievalStats>>,
}

/// Retrieval configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    /// Default retrieval strategy
    pub default_strategy: RetrievalStrategy,
    /// Similarity metric to use
    pub similarity_metric: SimilarityMetric,
    /// Minimum relevance score
    pub min_relevance_score: f32,
    /// Maximum results to return
    pub max_results: usize,
    /// Enable query expansion
    pub enable_query_expansion: bool,
    /// Enable result re-ranking
    pub enable_reranking: bool,
    /// Cache size limit
    pub cache_size: usize,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            default_strategy: RetrievalStrategy::Hybrid,
            similarity_metric: SimilarityMetric::Cosine,
            min_relevance_score: 0.7,
            max_results: 10,
            enable_query_expansion: true,
            enable_reranking: true,
            cache_size: 1000,
        }
    }
}

/// Retrieval strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RetrievalStrategy {
    /// Pure semantic search using embeddings
    Semantic,
    /// Traditional keyword-based search
    Keyword,
    /// Hybrid approach combining semantic and keyword
    Hybrid,
    /// Graph-based retrieval using relationships
    GraphBased,
    /// Pattern-based retrieval
    PatternBased,
    /// Adaptive strategy that chooses based on query
    Adaptive,
}

/// Context window for retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    /// Primary results
    pub primary_results: Vec<SemanticSearchResult>,
    /// Related/supporting results
    pub supporting_results: Vec<SemanticSearchResult>,
    /// Context summary
    pub summary: Option<String>,
    /// Suggested follow-up queries
    pub suggestions: Vec<String>,
    /// Confidence score for the context
    pub confidence: f32,
}

/// Query cache entry
#[derive(Debug, Clone)]
struct QueryCacheEntry {
    /// Query embedding
    embedding: Vec<f32>,
    /// Results
    results: Vec<SemanticSearchResult>,
    /// Timestamp
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Query cache
#[derive(Debug, Default)]
struct QueryCache {
    /// Cache entries
    entries: HashMap<String, QueryCacheEntry>,
    /// Access order for LRU eviction
    access_order: Vec<String>,
}

/// Retrieval statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RetrievalStats {
    /// Total queries processed
    pub total_queries: usize,
    /// Queries by strategy
    pub queries_by_strategy: HashMap<RetrievalStrategy, usize>,
    /// Average relevance score
    pub avg_relevance_score: f32,
    /// Cache hit rate
    pub cache_hit_rate: f32,
    /// Average retrieval time (ms)
    pub avg_retrieval_time: f32,
}

impl ContextRetriever {
    /// Create a new context retriever
    pub fn new(embedding_engine: Arc<EmbeddingEngine>) -> Self {
        Self::with_config(embedding_engine, RetrievalConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(embedding_engine: Arc<EmbeddingEngine>, config: RetrievalConfig) -> Self {
        Self {
            embedding_engine,
            config,
            cache: Arc::new(tokio::sync::RwLock::new(QueryCache::default())),
            stats: Arc::new(tokio::sync::RwLock::new(RetrievalStats::default())),
        }
    }
    
    /// Retrieve relevant context for a query
    pub async fn retrieve(
        &self,
        query_embedding: &[f32],
        strategy: RetrievalStrategy,
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        let start = std::time::Instant::now();
        debug!("Retrieving context with strategy: {:?}", strategy);
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_queries += 1;
            *stats.queries_by_strategy.entry(strategy).or_insert(0) += 1;
        }
        
        // Execute retrieval based on strategy
        let results = match strategy {
            RetrievalStrategy::Semantic => {
                self.semantic_retrieval(query_embedding, limit).await?
            }
            RetrievalStrategy::Keyword => {
                // For keyword search, we need the original query text
                // This is a limitation of the current interface
                self.keyword_retrieval("", limit).await?
            }
            RetrievalStrategy::Hybrid => {
                self.hybrid_retrieval(query_embedding, "", limit).await?
            }
            RetrievalStrategy::GraphBased => {
                self.graph_based_retrieval(query_embedding, limit).await?
            }
            RetrievalStrategy::PatternBased => {
                self.pattern_based_retrieval(query_embedding, limit).await?
            }
            RetrievalStrategy::Adaptive => {
                self.adaptive_retrieval(query_embedding, limit).await?
            }
        };
        
        // Apply re-ranking if enabled
        let results = if self.config.enable_reranking {
            self.rerank_results(results, query_embedding).await?
        } else {
            results
        };
        
        // Update retrieval time statistics
        let duration = start.elapsed().as_millis() as f32;
        {
            let mut stats = self.stats.write().await;
            let n = stats.total_queries as f32;
            stats.avg_retrieval_time = (stats.avg_retrieval_time * (n - 1.0) + duration) / n;
        }
        
        Ok(results)
    }
    
    /// Build a context window for comprehensive retrieval
    pub async fn build_context_window(
        &self,
        query: &str,
        strategy: RetrievalStrategy,
    ) -> Result<ContextWindow> {
        info!("Building context window for query: {}", query);
        
        // Get query embedding
        let query_embedding = self.embedding_engine.encode(query).await?;
        
        // Get primary results
        let primary_results = self.retrieve(&query_embedding, strategy, self.config.max_results).await?;
        
        // Get supporting results by expanding the query
        let supporting_results = if self.config.enable_query_expansion {
            let expanded_queries = self.expand_query(query, &primary_results).await?;
            let mut all_supporting = Vec::new();
            
            for expanded_query in &expanded_queries[..expanded_queries.len().min(3)] {
                let expanded_embedding = self.embedding_engine.encode(expanded_query).await?;
                let mut results = self.retrieve(&expanded_embedding, strategy, 5).await?;
                all_supporting.append(&mut results);
            }
            
            // Deduplicate
            self.deduplicate_results(all_supporting)
        } else {
            Vec::new()
        };
        
        // Generate context summary
        let summary = self.generate_summary(&primary_results, &supporting_results).await?;
        
        // Generate follow-up suggestions
        let suggestions = self.generate_suggestions(&primary_results).await?;
        
        // Calculate confidence
        let confidence = self.calculate_context_confidence(&primary_results, &supporting_results);
        
        Ok(ContextWindow {
            primary_results,
            supporting_results,
            summary,
            suggestions,
            confidence,
        })
    }
    
    /// Get retrieval statistics
    pub async fn get_stats(&self) -> RetrievalStats {
        self.stats.read().await.clone()
    }
    
    // Private retrieval methods
    
    async fn semantic_retrieval(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        debug!("Performing semantic retrieval");
        
        // Get all knowledge entries
        let entries = KnowledgeConversation::get_recent(1000).await?;
        let mut results = Vec::new();
        
        // Calculate similarities
        for entry in entries {
            // Generate embedding for the entry
            let text = format!("{} {}", entry.question, entry.final_answer);
            let entry_embedding = self.embedding_engine.encode(&text).await?;
            
            // Calculate similarity
            let similarity = self.embedding_engine.similarity(
                query_embedding,
                &entry_embedding,
                self.config.similarity_metric,
            );
            
            if similarity >= self.config.min_relevance_score {
                results.push(SemanticSearchResult {
                    id: entry.id,
                    question: entry.question,
                    answer: entry.final_answer,
                    similarity_score: similarity,
                    context: entry.conversation_context,
                    relationships: Vec::new(),
                });
            }
        }
        
        // Sort by similarity
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        results.truncate(limit);
        
        Ok(results)
    }
    
    async fn keyword_retrieval(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        debug!("Performing keyword retrieval");
        
        // Simple keyword search
        let entries = KnowledgeConversation::search(query, limit).await?;
        
        Ok(entries.into_iter().map(|entry| SemanticSearchResult {
            id: entry.id,
            question: entry.question,
            answer: entry.final_answer,
            similarity_score: 1.0, // Keyword match
            context: entry.conversation_context,
            relationships: Vec::new(),
        }).collect())
    }
    
    async fn hybrid_retrieval(
        &self,
        query_embedding: &[f32],
        query: &str,
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        debug!("Performing hybrid retrieval");
        
        // Get results from both methods
        let semantic_results = self.semantic_retrieval(query_embedding, limit).await?;
        let keyword_results = self.keyword_retrieval(query, limit).await?;
        
        // Merge and re-score
        let mut merged = HashMap::new();
        
        // Add semantic results
        for result in semantic_results {
            merged.insert(result.id.clone(), (result, 0.7)); // Semantic weight
        }
        
        // Add or update with keyword results
        for result in keyword_results {
            merged.entry(result.id.clone())
                .and_modify(|(r, w)| *w += 0.3) // Add keyword weight
                .or_insert((result, 0.3));
        }
        
        // Sort by combined score
        let mut final_results: Vec<_> = merged.into_iter()
            .map(|(_, (mut result, weight))| {
                result.similarity_score *= weight;
                result
            })
            .collect();
        
        final_results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        final_results.truncate(limit);
        
        Ok(final_results)
    }
    
    async fn graph_based_retrieval(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        debug!("Performing graph-based retrieval");
        
        // First get some seed results
        let seed_results = self.semantic_retrieval(query_embedding, 5).await?;
        
        // In a real implementation, this would traverse the knowledge graph
        // For now, return the seed results
        Ok(seed_results)
    }
    
    async fn pattern_based_retrieval(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        debug!("Performing pattern-based retrieval");
        
        // In a real implementation, this would use learned patterns
        // For now, fall back to semantic search
        self.semantic_retrieval(query_embedding, limit).await
    }
    
    async fn adaptive_retrieval(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        debug!("Performing adaptive retrieval");
        
        // Analyze query characteristics to choose strategy
        // For now, use hybrid as default
        self.hybrid_retrieval(query_embedding, "", limit).await
    }
    
    async fn rerank_results(
        &self,
        mut results: Vec<SemanticSearchResult>,
        query_embedding: &[f32],
    ) -> Result<Vec<SemanticSearchResult>> {
        debug!("Re-ranking {} results", results.len());
        
        // Simple re-ranking based on additional factors
        for result in &mut results {
            let mut boost = 1.0;
            
            // Boost recent results
            if result.context.is_some() {
                boost += 0.1;
            }
            
            // Boost results with relationships
            if !result.relationships.is_empty() {
                boost += 0.05 * result.relationships.len() as f32;
            }
            
            result.similarity_score *= boost.min(1.5);
        }
        
        // Re-sort
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        
        Ok(results)
    }
    
    async fn expand_query(
        &self,
        query: &str,
        results: &[SemanticSearchResult],
    ) -> Result<Vec<String>> {
        let mut expanded = Vec::new();
        
        // Add variations of the original query
        expanded.push(format!("How to {}", query));
        expanded.push(format!("Explain {}", query));
        expanded.push(format!("Best practices for {}", query));
        
        // Extract key terms from top results
        if let Some(top_result) = results.first() {
            let keywords = self.extract_keywords(&top_result.question);
            for keyword in keywords.iter().take(2) {
                expanded.push(format!("{} {}", query, keyword));
            }
        }
        
        Ok(expanded)
    }
    
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction
        text.split_whitespace()
            .filter(|w| w.len() > 4)
            .map(|w| w.to_lowercase())
            .filter(|w| !["that", "this", "with", "from", "what", "when", "where", "which"].contains(&w.as_str()))
            .take(5)
            .map(String::from)
            .collect()
    }
    
    fn deduplicate_results(&self, results: Vec<SemanticSearchResult>) -> Vec<SemanticSearchResult> {
        let mut seen = HashSet::new();
        let mut deduped = Vec::new();
        
        for result in results {
            if seen.insert(result.id.clone()) {
                deduped.push(result);
            }
        }
        
        deduped
    }
    
    async fn generate_summary(
        &self,
        primary: &[SemanticSearchResult],
        supporting: &[SemanticSearchResult],
    ) -> Result<Option<String>> {
        if primary.is_empty() {
            return Ok(None);
        }
        
        let mut summary = String::from("Based on the search results:\n\n");
        
        // Summarize primary results
        summary.push_str("Main findings:\n");
        for (i, result) in primary.iter().take(3).enumerate() {
            summary.push_str(&format!("{}. {}\n", i + 1, result.question));
        }
        
        // Add supporting context if available
        if !supporting.is_empty() {
            summary.push_str("\nRelated information:\n");
            for result in supporting.iter().take(2) {
                summary.push_str(&format!("- {}\n", result.question));
            }
        }
        
        Ok(Some(summary))
    }
    
    async fn generate_suggestions(
        &self,
        results: &[SemanticSearchResult],
    ) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();
        
        // Extract common themes from results
        let mut themes = HashMap::new();
        for result in results {
            for keyword in self.extract_keywords(&result.question) {
                *themes.entry(keyword).or_insert(0) += 1;
            }
        }
        
        // Generate suggestions based on themes
        let mut theme_vec: Vec<_> = themes.into_iter().collect();
        theme_vec.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        
        for (theme, _) in theme_vec.iter().take(3) {
            suggestions.push(format!("Tell me more about {}", theme));
            suggestions.push(format!("How does {} work?", theme));
        }
        
        Ok(suggestions)
    }
    
    fn calculate_context_confidence(
        &self,
        primary: &[SemanticSearchResult],
        supporting: &[SemanticSearchResult],
    ) -> f32 {
        if primary.is_empty() {
            return 0.0;
        }
        
        // Base confidence on primary results
        let avg_primary_score: f32 = primary.iter()
            .map(|r| r.similarity_score)
            .sum::<f32>() / primary.len() as f32;
        
        let mut confidence = avg_primary_score;
        
        // Boost for supporting results
        if !supporting.is_empty() {
            confidence += 0.1;
        }
        
        // Boost for high coherence (similar scores)
        if primary.len() > 1 {
            let score_variance = primary.iter()
                .map(|r| (r.similarity_score - avg_primary_score).powi(2))
                .sum::<f32>() / primary.len() as f32;
            
            if score_variance < 0.01 {
                confidence += 0.1;
            }
        }
        
        confidence.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_context_retriever_creation() -> Result<()> {
        let embedding_engine = Arc::new(EmbeddingEngine::new().await?);
        let retriever = ContextRetriever::new(embedding_engine);
        
        let stats = retriever.get_stats().await;
        assert_eq!(stats.total_queries, 0);
        
        Ok(())
    }
    
    #[test]
    fn test_keyword_extraction() {
        let retriever = ContextRetriever::new(Arc::new(EmbeddingEngine::new().await.unwrap()));
        let keywords = retriever.extract_keywords("How do I implement error handling in Rust?");
        
        assert!(keywords.contains(&"implement".to_string()));
        assert!(keywords.contains(&"error".to_string()));
        assert!(keywords.contains(&"handling".to_string()));
    }
}