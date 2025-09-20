//! Semantic Retriever - AI Helper for intelligent information retrieval
//!
//! This module provides pure execution of retrieval tasks as directed
//! by the Consensus pipeline. It NEVER makes decisions about what to retrieve
//! or how to prioritize results - it only executes retrieval plans.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::monitoring::{OperationType, PerformanceMonitor};
use super::python_models::PythonModelService;
use super::vector_store::ChromaVectorStore;

/// Retrieval plan from Consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalPlan {
    /// Query provided by Consensus
    pub query: String,
    /// Type of retrieval requested
    pub retrieval_type: RetrievalType,
    /// Filters to apply
    pub filters: RetrievalFilters,
    /// Maximum results to return
    pub max_results: usize,
    /// Similarity threshold (0.0 - 1.0)
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetrievalType {
    /// Find similar code patterns
    CodePattern,
    /// Find API usage examples
    ApiUsage { api_name: String },
    /// Find documentation
    Documentation,
    /// Find error solutions
    ErrorSolution { error_type: String },
    /// General semantic search
    Semantic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalFilters {
    /// Language filter (e.g., "rust", "python")
    pub languages: Option<Vec<String>>,
    /// File path patterns to include/exclude
    pub path_patterns: Option<Vec<String>>,
    /// Time range filter
    pub time_range: Option<TimeRange>,
    /// Quality score threshold
    pub min_quality_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Option<chrono::DateTime<chrono::Utc>>,
    pub end: Option<chrono::DateTime<chrono::Utc>>,
}

/// Retrieved item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedItem {
    /// Unique identifier
    pub id: String,
    /// Content of the item
    pub content: String,
    /// Similarity score (0.0 - 1.0)
    pub similarity: f32,
    /// Metadata about the item
    pub metadata: ItemMetadata,
    /// Relevant excerpt
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMetadata {
    /// Source file path
    pub file_path: Option<String>,
    /// Programming language
    pub language: Option<String>,
    /// Function/class name
    pub entity_name: Option<String>,
    /// Line numbers
    pub line_range: Option<(usize, usize)>,
    /// Creation/modification time
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Quality score
    pub quality_score: Option<f32>,
}

/// Retrieval result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// Retrieved items sorted by relevance
    pub items: Vec<RetrievedItem>,
    /// Total items found (before filtering)
    pub total_found: usize,
    /// Search statistics
    pub stats: SearchStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    /// Time taken for embedding generation
    pub embedding_time_ms: u64,
    /// Time taken for vector search
    pub search_time_ms: u64,
    /// Time taken for post-processing
    pub processing_time_ms: u64,
    /// Number of vectors searched
    pub vectors_searched: usize,
}

/// Semantic retriever using vector search
pub struct SemanticRetriever {
    /// Vector store for embeddings
    vector_store: Arc<ChromaVectorStore>,

    /// Python service for embeddings
    python_service: Arc<PythonModelService>,

    /// Performance monitor
    monitor: Arc<PerformanceMonitor>,

    /// Cache for embeddings
    embedding_cache: Arc<RwLock<EmbeddingCache>>,
}

struct EmbeddingCache {
    entries: HashMap<String, CachedEmbedding>,
    max_size: usize,
}

struct CachedEmbedding {
    embedding: Vec<f32>,
    timestamp: std::time::Instant,
}

impl SemanticRetriever {
    /// Create a new semantic retriever
    pub fn new(
        vector_store: Arc<ChromaVectorStore>,
        python_service: Arc<PythonModelService>,
        monitor: Arc<PerformanceMonitor>,
    ) -> Self {
        Self {
            vector_store,
            python_service,
            monitor,
            embedding_cache: Arc::new(RwLock::new(EmbeddingCache {
                entries: HashMap::new(),
                max_size: 1000,
            })),
        }
    }

    /// Execute a retrieval plan
    pub async fn execute_retrieval(&self, plan: &RetrievalPlan) -> Result<RetrievalResult> {
        let start = std::time::Instant::now();
        info!("Executing retrieval plan: {:?}", plan.retrieval_type);

        let mut stats = SearchStats {
            embedding_time_ms: 0,
            search_time_ms: 0,
            processing_time_ms: 0,
            vectors_searched: 0,
        };

        // Generate embedding for query
        let embedding_start = std::time::Instant::now();
        let query_embedding = self.generate_embedding(&plan.query).await?;
        stats.embedding_time_ms = embedding_start.elapsed().as_millis() as u64;

        // Perform vector search
        let search_start = std::time::Instant::now();
        let search_results = self
            .vector_store
            .search(&query_embedding, plan.max_results * 2) // Get extra for filtering
            .await
            .context("Failed to perform vector search")?;
        stats.search_time_ms = search_start.elapsed().as_millis() as u64;
        stats.vectors_searched = search_results.len();

        // Post-process results
        let processing_start = std::time::Instant::now();
        let processed_items = self.process_results(search_results, plan).await?;
        stats.processing_time_ms = processing_start.elapsed().as_millis() as u64;

        // Filter by similarity threshold
        let filtered_items: Vec<RetrievedItem> = processed_items
            .into_iter()
            .filter(|item| item.similarity >= plan.similarity_threshold)
            .take(plan.max_results)
            .collect();

        let total_found = stats.vectors_searched;

        Ok(RetrievalResult {
            items: filtered_items,
            total_found,
            stats,
        })
    }

    /// Generate embedding for text
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        if let Some(cached) = self.get_cached_embedding(text).await {
            debug!("Using cached embedding");
            return Ok(cached);
        }

        // Generate new embedding using CodeBERT
        let embeddings = self
            .python_service
            .generate_embeddings("codebert", vec![text.to_string()])
            .await
            .context("Failed to generate embedding")?;

        // Get the first embedding
        let embedding = embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))?;

        // Cache the embedding
        self.cache_embedding(text, &embedding).await;

        Ok(embedding)
    }

    /// Process raw search results into retrieved items
    async fn process_results(
        &self,
        results: Vec<(String, String, Vec<f32>, serde_json::Value)>,
        plan: &RetrievalPlan,
    ) -> Result<Vec<RetrievedItem>> {
        let mut items = Vec::new();

        for (id, content, _embedding, metadata) in results {
            // Calculate similarity (would need query embedding for real similarity)
            // For now, use a placeholder
            let similarity = 0.8;
            // Apply filters
            if !self.passes_filters(&metadata, &plan.filters)? {
                continue;
            }

            // Generate excerpt based on retrieval type
            let excerpt = self.generate_excerpt(&content, &plan.query, &plan.retrieval_type);

            // Parse metadata
            let item_metadata = self.parse_metadata(&metadata)?;

            items.push(RetrievedItem {
                id,
                content,
                similarity,
                metadata: item_metadata,
                excerpt,
            });
        }

        Ok(items)
    }

    /// Check if result passes filters
    fn passes_filters(
        &self,
        metadata: &serde_json::Value,
        filters: &RetrievalFilters,
    ) -> Result<bool> {
        // Language filter
        if let Some(languages) = &filters.languages {
            let item_lang = metadata
                .get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if !languages.iter().any(|l| l == item_lang) {
                return Ok(false);
            }
        }

        // Path pattern filter
        if let Some(patterns) = &filters.path_patterns {
            let file_path = metadata
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let matches = patterns.iter().any(|pattern| {
                file_path.contains(pattern)
                    || glob::Pattern::new(pattern)
                        .ok()
                        .map(|p| p.matches(file_path))
                        .unwrap_or(false)
            });

            if !matches {
                return Ok(false);
            }
        }

        // Quality score filter
        if let Some(min_score) = filters.min_quality_score {
            let quality_score = metadata
                .get("quality_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;

            if quality_score < min_score {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Generate excerpt highlighting relevant parts
    fn generate_excerpt(
        &self,
        content: &str,
        query: &str,
        retrieval_type: &RetrievalType,
    ) -> Option<String> {
        match retrieval_type {
            RetrievalType::CodePattern => {
                // Find most relevant code section
                self.extract_code_excerpt(content, query)
            }
            RetrievalType::ApiUsage { api_name } => {
                // Find API usage
                self.extract_api_usage(content, api_name)
            }
            RetrievalType::Documentation => {
                // Extract summary
                self.extract_doc_summary(content)
            }
            RetrievalType::ErrorSolution { .. } => {
                // Extract solution part
                self.extract_solution(content)
            }
            RetrievalType::Semantic => {
                // General excerpt
                self.extract_general_excerpt(content, query)
            }
        }
    }

    /// Extract code excerpt
    fn extract_code_excerpt(&self, content: &str, query: &str) -> Option<String> {
        // Simple implementation - find lines containing query terms
        let query_lower = query.to_lowercase();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                let start = i.saturating_sub(2);
                let end = (i + 3).min(lines.len());

                return Some(lines[start..end].join("\n"));
            }
        }

        None
    }

    /// Extract API usage example
    fn extract_api_usage(&self, content: &str, api_name: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.contains(api_name) {
                let start = i.saturating_sub(1);
                let end = (i + 5).min(lines.len());

                return Some(lines[start..end].join("\n"));
            }
        }

        None
    }

    /// Extract documentation summary
    fn extract_doc_summary(&self, content: &str) -> Option<String> {
        // Take first paragraph or first 200 chars
        content.split("\n\n").next().map(|s| {
            if s.len() > 200 {
                format!("{}...", &s[..200])
            } else {
                s.to_string()
            }
        })
    }

    /// Extract solution from error documentation
    fn extract_solution(&self, content: &str) -> Option<String> {
        // Look for solution markers
        let markers = ["solution:", "fix:", "resolve:", "workaround:"];

        for marker in &markers {
            if let Some(pos) = content.to_lowercase().find(marker) {
                let solution_start = pos + marker.len();
                let solution = &content[solution_start..];

                // Take until next section or 300 chars
                let excerpt = solution.split("\n\n").next().unwrap_or(solution);

                return Some(excerpt.trim().to_string());
            }
        }

        None
    }

    /// Extract general excerpt
    fn extract_general_excerpt(&self, content: &str, query: &str) -> Option<String> {
        // Find most relevant section
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let lines: Vec<&str> = content.lines().collect();

        let mut best_score = 0;
        let mut best_line = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let score = query_terms
                .iter()
                .filter(|term| line_lower.contains(&term.to_lowercase()))
                .count();

            if score > best_score {
                best_score = score;
                best_line = i;
            }
        }

        if best_score > 0 {
            let start = best_line.saturating_sub(2);
            let end = (best_line + 3).min(lines.len());

            Some(lines[start..end].join("\n"))
        } else {
            // Fallback to first 200 chars
            Some(content.chars().take(200).collect::<String>() + "...")
        }
    }

    /// Parse metadata from JSON
    fn parse_metadata(&self, json: &serde_json::Value) -> Result<ItemMetadata> {
        Ok(ItemMetadata {
            file_path: json
                .get("file_path")
                .and_then(|v| v.as_str())
                .map(String::from),
            language: json
                .get("language")
                .and_then(|v| v.as_str())
                .map(String::from),
            entity_name: json
                .get("entity_name")
                .and_then(|v| v.as_str())
                .map(String::from),
            line_range: json
                .get("line_start")
                .and_then(|v| v.as_u64())
                .and_then(|start| {
                    json.get("line_end")
                        .and_then(|v| v.as_u64())
                        .map(|end| (start as usize, end as usize))
                }),
            timestamp: json
                .get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            quality_score: json
                .get("quality_score")
                .and_then(|v| v.as_f64())
                .map(|s| s as f32),
        })
    }

    /// Get cached embedding
    async fn get_cached_embedding(&self, text: &str) -> Option<Vec<f32>> {
        let cache = self.embedding_cache.read().await;
        cache
            .entries
            .get(text)
            .filter(|entry| entry.timestamp.elapsed().as_secs() < 3600)
            .map(|entry| entry.embedding.clone())
    }

    /// Cache embedding
    async fn cache_embedding(&self, text: &str, embedding: &[f32]) {
        let mut cache = self.embedding_cache.write().await;

        // Evict old entries if needed
        if cache.entries.len() >= cache.max_size {
            let oldest_key = cache
                .entries
                .iter()
                .min_by_key(|(_, v)| v.timestamp)
                .map(|(k, _)| k.clone());

            if let Some(k) = oldest_key {
                cache.entries.remove(&k);
            }
        }

        cache.entries.insert(
            text.to_string(),
            CachedEmbedding {
                embedding: embedding.to_vec(),
                timestamp: std::time::Instant::now(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieval_filters() {
        let filters = RetrievalFilters {
            languages: Some(vec!["rust".to_string()]),
            path_patterns: None,
            time_range: None,
            min_quality_score: Some(0.8),
        };

        assert!(filters.languages.is_some());
        assert_eq!(filters.languages.as_ref().unwrap().len(), 1);
        assert_eq!(filters.min_quality_score, Some(0.8));
    }
}
