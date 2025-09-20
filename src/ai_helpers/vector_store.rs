//! Vector Store - Chroma integration for local vector storage and retrieval
//!
//! This module provides persistent vector storage for embeddings, enabling
//! fast similarity search and knowledge retrieval.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for Chroma Vector Store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    /// Path to the Chroma database
    pub db_path: PathBuf,

    /// Collection name
    pub collection_name: String,

    /// Distance metric (e.g., "cosine", "euclidean", "dot")
    pub distance_metric: String,

    /// Maximum number of results to return
    pub max_results: usize,

    /// Minimum similarity score for results
    pub min_similarity: f64,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            db_path: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("hive")
                .join("chroma"),
            collection_name: "hive_knowledge".to_string(),
            distance_metric: "cosine".to_string(),
            max_results: 100,
            min_similarity: 0.7,
        }
    }
}

/// Chroma Vector Store for persistent embedding storage
pub struct ChromaVectorStore {
    config: VectorStoreConfig,

    /// Connection to Chroma (would be actual client in production)
    client: Arc<RwLock<ChromaClient>>,

    /// Cache of recent searches
    search_cache: Arc<RwLock<lru::LruCache<String, Vec<SearchResult>>>>,
}

/// Placeholder for Chroma client
struct ChromaClient {
    // In production, this would be the actual Chroma client
    // For now, we'll use in-memory storage
    documents: Vec<Document>,
}

/// Document in the vector store
#[derive(Debug, Clone)]
struct Document {
    id: String,
    content: String,
    embedding: Vec<f32>,
    metadata: serde_json::Value,
}

/// Search result from vector store
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: serde_json::Value,
    pub similarity: f64,
}

impl ChromaVectorStore {
    /// Create a new Chroma Vector Store
    pub async fn new() -> Result<Self> {
        let config = VectorStoreConfig::default();

        // Ensure database directory exists
        tokio::fs::create_dir_all(&config.db_path)
            .await
            .context("Failed to create Chroma database directory")?;

        let client = Arc::new(RwLock::new(ChromaClient {
            documents: Vec::new(),
        }));

        let search_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(100).unwrap(),
        )));

        Ok(Self {
            config,
            client,
            search_cache,
        })
    }

    /// Add a document to the vector store
    pub async fn add_document(
        &self,
        id: &str,
        content: &str,
        embedding: &[f32],
        metadata: &serde_json::Value,
    ) -> Result<()> {
        let document = Document {
            id: id.to_string(),
            content: content.to_string(),
            embedding: embedding.to_vec(),
            metadata: metadata.clone(),
        };

        // Add to store
        self.client.write().await.documents.push(document);

        // Clear search cache as results may have changed
        self.search_cache.write().await.clear();

        tracing::debug!("Added document {} to vector store", id);
        Ok(())
    }

    /// Search for similar documents
    pub async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, String, Vec<f32>, serde_json::Value)>> {
        // Check cache
        let cache_key = format!(
            "{:?}_{}",
            &query_embedding[..5.min(query_embedding.len())],
            limit
        );

        if let Some(cached) = self.search_cache.read().await.peek(&cache_key) {
            return Ok(cached
                .iter()
                .map(|r| {
                    (
                        r.id.clone(),
                        r.content.clone(),
                        r.embedding.clone(),
                        r.metadata.clone(),
                    )
                })
                .collect());
        }

        // Perform similarity search
        let results = self.similarity_search(query_embedding, limit).await?;

        // Cache results
        self.search_cache
            .write()
            .await
            .put(cache_key, results.clone());

        // Convert to expected format
        Ok(results
            .into_iter()
            .map(|r| (r.id, r.content, r.embedding, r.metadata))
            .collect())
    }

    /// Perform similarity search
    async fn similarity_search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let client = self.client.read().await;

        // Calculate similarities for all documents
        let mut scored_docs: Vec<(f64, &Document)> = client
            .documents
            .iter()
            .map(|doc| {
                let similarity = self.calculate_similarity(query_embedding, &doc.embedding);
                (similarity, doc)
            })
            .filter(|(sim, _)| *sim >= self.config.min_similarity)
            .collect();

        // Sort by similarity (descending)
        scored_docs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Take top results
        let results: Vec<SearchResult> = scored_docs
            .into_iter()
            .take(limit.min(self.config.max_results))
            .map(|(similarity, doc)| SearchResult {
                id: doc.id.clone(),
                content: doc.content.clone(),
                embedding: doc.embedding.clone(),
                metadata: doc.metadata.clone(),
                similarity,
            })
            .collect();

        Ok(results)
    }

    /// Calculate similarity between two embeddings
    fn calculate_similarity(&self, embedding1: &[f32], embedding2: &[f32]) -> f64 {
        match self.config.distance_metric.as_str() {
            "cosine" => self.cosine_similarity(embedding1, embedding2),
            "euclidean" => self.euclidean_similarity(embedding1, embedding2),
            "dot" => self.dot_product(embedding1, embedding2),
            _ => self.cosine_similarity(embedding1, embedding2),
        }
    }

    /// Cosine similarity
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            (dot_product / (norm_a * norm_b)) as f64
        }
    }

    /// Euclidean similarity (converted from distance)
    fn euclidean_similarity(&self, a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }

        let distance: f32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();

        // Convert distance to similarity (closer = higher similarity)
        1.0 / (1.0 + distance as f64)
    }

    /// Dot product
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }

        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>() as f64
    }

    /// Get the current size of the vector store
    pub async fn get_size(&self) -> Result<usize> {
        Ok(self.client.read().await.documents.len())
    }

    /// Delete a document by ID
    pub async fn delete_document(&self, id: &str) -> Result<()> {
        let mut client = self.client.write().await;
        client.documents.retain(|doc| doc.id != id);

        // Clear cache
        self.search_cache.write().await.clear();

        Ok(())
    }

    /// Clear all documents
    pub async fn clear(&self) -> Result<()> {
        self.client.write().await.documents.clear();
        self.search_cache.write().await.clear();

        Ok(())
    }

    /// Persist the vector store to disk
    pub async fn persist(&self) -> Result<()> {
        // TODO: Implement actual persistence
        // For now, this is a no-op as we're using in-memory storage

        tracing::info!("Persisting vector store to {:?}", self.config.db_path);
        Ok(())
    }

    /// Load the vector store from disk
    pub async fn load(&self) -> Result<()> {
        // TODO: Implement actual loading
        // For now, this is a no-op as we're using in-memory storage

        tracing::info!("Loading vector store from {:?}", self.config.db_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_operations() {
        let store = ChromaVectorStore::new().await.unwrap();

        // Test adding and searching
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let metadata = serde_json::json!({"test": true});

        store
            .add_document("test1", "Test content", &embedding, &metadata)
            .await
            .unwrap();

        let results = store.search(&embedding, 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "test1");
    }

    #[test]
    fn test_similarity_calculations() {
        let store = ChromaVectorStore {
            config: VectorStoreConfig::default(),
            client: Arc::new(RwLock::new(ChromaClient { documents: vec![] })),
            search_cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(100).unwrap(),
            ))),
        };

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let c = vec![1.0, 0.0, 0.0];

        // Test cosine similarity
        assert!((store.cosine_similarity(&a, &c) - 1.0).abs() < 0.001);
        assert!((store.cosine_similarity(&a, &b)).abs() < 0.001);
    }
}
