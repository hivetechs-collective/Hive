//! Knowledge Indexer - Uses CodeBERT/CodeT5+ for semantic understanding and embedding generation
//!
//! This module indexes Curator outputs, creates semantic embeddings, maintains
//! a knowledge graph of all accumulated facts, and tracks file operation outcomes
//! for intelligent auto-accept decisions.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::python_models::PythonModelService;
use crate::ai_helpers::{ChromaVectorStore, IndexedKnowledge};
use crate::consensus::operation_intelligence::{OperationContext, OperationOutcome};
use crate::consensus::stages::file_aware_curator::FileOperation;

/// Configuration for the Knowledge Indexer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    /// Model to use for embeddings (e.g., "microsoft/unixcoder-base")
    pub embedding_model: String,

    /// Maximum sequence length for the model
    pub max_sequence_length: usize,

    /// Batch size for processing
    pub batch_size: usize,

    /// Whether to use GPU if available
    pub use_gpu: bool,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            embedding_model: "microsoft/unixcoder-base".to_string(),
            max_sequence_length: 512,
            batch_size: 32,
            use_gpu: true,
        }
    }
}

/// Indexed operation for similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedOperation {
    /// Unique identifier
    pub id: String,

    /// The file operation that was performed
    pub operation: FileOperation,

    /// Context when operation was performed
    pub context: OperationContext,

    /// Semantic embedding of operation and context
    pub embedding: Vec<f32>,

    /// Outcome of the operation
    pub outcome: OperationOutcome,

    /// User satisfaction rating (0-1)
    pub user_satisfaction: Option<f32>,

    /// When this was indexed
    pub indexed_at: SystemTime,
}

/// Operation similarity result
#[derive(Debug, Clone)]
pub struct OperationSimilarity {
    /// The similar operation
    pub operation: IndexedOperation,

    /// Similarity score (0-1, higher = more similar)
    pub similarity: f32,

    /// Reasons for similarity
    pub similarity_reasons: Vec<String>,
}

/// Operation success prediction
#[derive(Debug, Clone)]
pub struct OperationSuccessPrediction {
    /// Predicted success probability (0-1)
    pub success_probability: f32,

    /// Number of similar operations found
    pub similar_operations_count: usize,

    /// Historical success rate for similar operations
    pub historical_success_rate: f32,

    /// Common failure modes for similar operations
    pub common_failure_modes: Vec<String>,

    /// Confidence in the prediction (0-1)
    pub prediction_confidence: f32,
}

/// Knowledge Indexer using CodeBERT/UniXcoder
pub struct KnowledgeIndexer {
    config: IndexerConfig,
    vector_store: Arc<ChromaVectorStore>,

    /// Python model service for embeddings
    python_service: Arc<PythonModelService>,

    /// Cache of recent embeddings
    embedding_cache: Arc<RwLock<lru::LruCache<String, Vec<f32>>>>,

    /// In-memory operation history for quick access
    operation_history: Arc<RwLock<Vec<IndexedOperation>>>,

    /// Operation statistics cache
    operation_stats: Arc<RwLock<HashMap<String, f32>>>,
}

impl KnowledgeIndexer {
    /// Create a new Knowledge Indexer
    pub async fn new(
        vector_store: Arc<ChromaVectorStore>,
        python_service: Arc<PythonModelService>,
    ) -> Result<Self> {
        let config = IndexerConfig::default();
        let embedding_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(1000).unwrap(),
        )));

        let operation_history = Arc::new(RwLock::new(Vec::new()));
        let operation_stats = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            config,
            vector_store,
            python_service,
            embedding_cache,
            operation_history,
            operation_stats,
        })
    }

    /// Index a Curator output
    pub async fn index_output(
        &self,
        curator_output: &str,
        source_question: &str,
        conversation_id: &str,
    ) -> Result<IndexedKnowledge> {
        // Generate unique ID
        let id = format!("fact_{}", uuid::Uuid::new_v4());

        // Check cache first
        let cache_key = format!("{}{}", curator_output, source_question);
        if let Some(cached_embedding) = self.embedding_cache.read().await.peek(&cache_key) {
            return Ok(IndexedKnowledge {
                id: id.clone(),
                content: curator_output.to_string(),
                embedding: cached_embedding.clone(),
                metadata: self.create_metadata(source_question, conversation_id),
            });
        }

        // Generate embedding
        let embedding = self.generate_embedding(curator_output).await?;

        // Cache the embedding
        self.embedding_cache
            .write()
            .await
            .put(cache_key, embedding.clone());

        // Store in vector database
        self.vector_store
            .add_document(
                &id,
                curator_output,
                &embedding,
                &self.create_metadata(source_question, conversation_id),
            )
            .await?;

        Ok(IndexedKnowledge {
            id,
            content: curator_output.to_string(),
            embedding,
            metadata: self.create_metadata(source_question, conversation_id),
        })
    }

    /// Generate embedding for text
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Use the actual model through Python service
        let embeddings = self
            .python_service
            .generate_embeddings(&self.config.embedding_model, vec![text.to_string()])
            .await?;

        embeddings
            .into_iter()
            .next()
            .context("No embedding returned from model")
    }

    /// Create metadata for indexed knowledge
    fn create_metadata(&self, source_question: &str, conversation_id: &str) -> serde_json::Value {
        serde_json::json!({
            "source_question": source_question,
            "conversation_id": conversation_id,
            "indexed_at": chrono::Utc::now().to_rfc3339(),
            "model": self.config.embedding_model,
        })
    }

    /// Search for similar knowledge
    pub async fn search_similar(&self, query: &str, limit: usize) -> Result<Vec<IndexedKnowledge>> {
        // Generate query embedding
        let query_embedding = self.generate_embedding(query).await?;

        // Search in vector store
        let results = self.vector_store.search(&query_embedding, limit).await?;

        // Convert results to IndexedKnowledge
        let mut indexed_results = Vec::new();
        for (id, content, embedding, metadata) in results {
            indexed_results.push(IndexedKnowledge {
                id,
                content,
                embedding,
                metadata,
            });
        }

        Ok(indexed_results)
    }

    // === File Operation Tracking Methods ===

    /// Index a file operation with its outcome
    pub async fn index_operation_outcome(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
        outcome: &OperationOutcome,
        user_satisfaction: Option<f32>,
    ) -> Result<String> {
        info!("📚 Indexing operation outcome for learning");

        // Generate unique ID for this operation
        let operation_id = format!("op_{}", uuid::Uuid::new_v4());

        // Create operation description for embedding
        let operation_description = self.create_operation_description(operation, context);

        // Generate embedding for the operation and context
        let embedding = self.generate_embedding(&operation_description).await?;

        // Create indexed operation
        let indexed_operation = IndexedOperation {
            id: operation_id.clone(),
            operation: operation.clone(),
            context: context.clone(),
            embedding: embedding.clone(),
            outcome: outcome.clone(),
            user_satisfaction,
            indexed_at: SystemTime::now(),
        };

        // Store in vector database with operation-specific metadata
        let metadata =
            self.create_operation_metadata(operation, context, outcome, user_satisfaction);
        self.vector_store
            .add_document(&operation_id, &operation_description, &embedding, &metadata)
            .await?;

        // Add to in-memory history
        {
            let mut history = self.operation_history.write().await;
            history.push(indexed_operation);

            // Keep only recent operations in memory (last 1000)
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        // Update operation statistics
        self.update_operation_statistics(operation, outcome).await?;

        debug!(
            "✅ Successfully indexed operation outcome: {}",
            operation_id
        );
        Ok(operation_id)
    }

    /// Find similar operations for success prediction
    pub async fn find_similar_operations(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
        limit: usize,
    ) -> Result<Vec<OperationSimilarity>> {
        debug!("🔍 Finding similar operations for prediction");

        // Create operation description for similarity search
        let operation_description = self.create_operation_description(operation, context);

        // Generate embedding for the query
        let query_embedding = self.generate_embedding(&operation_description).await?;

        // Search for similar operations in vector store
        let search_results = self
            .vector_store
            .search(&query_embedding, limit * 2)
            .await?;

        let mut similarities = Vec::new();

        // Process search results and calculate detailed similarity
        for (id, content, embedding, metadata) in search_results {
            // Skip if this is not an operation (no "operation_type" in metadata)
            if !metadata.get("operation_type").is_some() {
                continue;
            }

            // Find the full operation from history
            if let Some(indexed_op) = self.find_operation_by_id(&id).await? {
                let similarity_score =
                    self.calculate_cosine_similarity(&query_embedding, &embedding);
                let similarity_reasons =
                    self.analyze_similarity_reasons(operation, context, &indexed_op);

                similarities.push(OperationSimilarity {
                    operation: indexed_op,
                    similarity: similarity_score,
                    similarity_reasons,
                });
            }
        }

        // Sort by similarity score and take top results
        similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        similarities.truncate(limit);

        debug!("Found {} similar operations", similarities.len());
        Ok(similarities)
    }

    /// Predict operation success based on historical data
    pub async fn predict_operation_success(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<OperationSuccessPrediction> {
        info!("🎯 Predicting operation success based on history");

        // Find similar operations
        let similar_operations = self.find_similar_operations(operation, context, 20).await?;

        if similar_operations.is_empty() {
            // No similar operations found, return conservative prediction
            return Ok(OperationSuccessPrediction {
                success_probability: 0.5, // Conservative default
                similar_operations_count: 0,
                historical_success_rate: 0.5,
                common_failure_modes: Vec::new(),
                prediction_confidence: 0.1, // Low confidence
            });
        }

        // Calculate success rate from similar operations
        let successful_operations = similar_operations
            .iter()
            .filter(|op| op.operation.outcome.success)
            .count();

        let success_rate = successful_operations as f32 / similar_operations.len() as f32;

        // Weight success rate by similarity scores
        let weighted_success_rate = similar_operations
            .iter()
            .map(|op| {
                if op.operation.outcome.success {
                    op.similarity
                } else {
                    0.0
                }
            })
            .sum::<f32>()
            / similar_operations
                .iter()
                .map(|op| op.similarity)
                .sum::<f32>();

        // Collect common failure modes
        let mut failure_modes = HashMap::new();
        for op in &similar_operations {
            if !op.operation.outcome.success {
                if let Some(error_msg) = &op.operation.outcome.error_message {
                    *failure_modes.entry(error_msg.clone()).or_insert(0) += 1;
                }
            }
        }

        let common_failure_modes: Vec<String> = failure_modes
            .into_iter()
            .filter(|(_, count)| *count >= 2) // At least 2 occurrences
            .map(|(mode, _)| mode)
            .collect();

        // Calculate prediction confidence based on number of similar operations and their similarity
        let avg_similarity = similar_operations
            .iter()
            .map(|op| op.similarity)
            .sum::<f32>()
            / similar_operations.len() as f32;

        let confidence =
            (avg_similarity * (similar_operations.len() as f32 / 20.0).min(1.0)).clamp(0.0, 1.0);

        let prediction = OperationSuccessPrediction {
            success_probability: weighted_success_rate,
            similar_operations_count: similar_operations.len(),
            historical_success_rate: success_rate,
            common_failure_modes,
            prediction_confidence: confidence,
        };

        info!(
            "📊 Success prediction: {:.2} probability, {:.2} confidence",
            prediction.success_probability, prediction.prediction_confidence
        );

        Ok(prediction)
    }

    /// Get operation statistics by operation type
    pub async fn get_operation_statistics(&self, operation_type: &str) -> Result<OperationStats> {
        let history = self.operation_history.read().await;

        let matching_operations: Vec<_> = history
            .iter()
            .filter(|op| self.get_operation_type(&op.operation) == operation_type)
            .collect();

        if matching_operations.is_empty() {
            return Ok(OperationStats {
                total_operations: 0,
                successful_operations: 0,
                success_rate: 0.0,
                average_execution_time: Duration::from_secs(0),
                average_user_satisfaction: None,
            });
        }

        let successful = matching_operations
            .iter()
            .filter(|op| op.outcome.success)
            .count();

        let avg_execution_time = Duration::from_nanos(
            matching_operations
                .iter()
                .map(|op| op.outcome.execution_time.as_nanos() as u64)
                .sum::<u64>()
                / matching_operations.len() as u64,
        );

        let satisfactions: Vec<f32> = matching_operations
            .iter()
            .filter_map(|op| op.user_satisfaction)
            .collect();

        let avg_satisfaction = if !satisfactions.is_empty() {
            Some(satisfactions.iter().sum::<f32>() / satisfactions.len() as f32)
        } else {
            None
        };

        Ok(OperationStats {
            total_operations: matching_operations.len(),
            successful_operations: successful,
            success_rate: successful as f32 / matching_operations.len() as f32,
            average_execution_time: avg_execution_time,
            average_user_satisfaction: avg_satisfaction,
        })
    }

    /// Clear old operation history to free memory
    pub async fn cleanup_old_operations(&self, keep_days: u32) -> Result<usize> {
        let cutoff_time = SystemTime::now() - Duration::from_secs(60 * 60 * 24 * keep_days as u64);

        let mut history = self.operation_history.write().await;
        let original_len = history.len();

        history.retain(|op| op.indexed_at > cutoff_time);

        let removed_count = original_len - history.len();

        if removed_count > 0 {
            info!("🧹 Cleaned up {} old operations", removed_count);
        }

        Ok(removed_count)
    }

    // === Private Helper Methods ===

    /// Create a textual description of an operation for embedding
    fn create_operation_description(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> String {
        let operation_text = match operation {
            FileOperation::Create { path, content } => {
                format!(
                    "Create file {} with content length {}",
                    path.display(),
                    content.len()
                )
            }
            FileOperation::Update { path, content } => {
                format!(
                    "Update file {} with content length {}",
                    path.display(),
                    content.len()
                )
            }
            FileOperation::Append { path, content } => {
                format!(
                    "Append to file {} with content length {}",
                    path.display(),
                    content.len()
                )
            }
            FileOperation::Delete { path } => {
                format!("Delete file {}", path.display())
            }
            FileOperation::Rename { from, to } => {
                format!("Rename file {} to {}", from.display(), to.display())
            }
        };

        format!(
            "Operation: {} | Question: {} | Repository: {} | Related files: {}",
            operation_text,
            context.source_question,
            context.repository_path.display(),
            context
                .related_files
                .iter()
                .map(|f| f.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    /// Create metadata for operation indexing
    fn create_operation_metadata(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
        outcome: &OperationOutcome,
        user_satisfaction: Option<f32>,
    ) -> serde_json::Value {
        serde_json::json!({
            "operation_type": self.get_operation_type(operation),
            "success": outcome.success,
            "execution_time_ms": outcome.execution_time.as_millis(),
            "user_satisfaction": user_satisfaction,
            "source_question": context.source_question,
            "repository_path": context.repository_path.display().to_string(),
            "git_commit": context.git_commit,
            "indexed_at": chrono::Utc::now().to_rfc3339(),
            "model": self.config.embedding_model,
            "rollback_required": outcome.rollback_required,
            "related_files_count": context.related_files.len(),
        })
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

    /// Update operation statistics
    async fn update_operation_statistics(
        &self,
        operation: &FileOperation,
        outcome: &OperationOutcome,
    ) -> Result<()> {
        let operation_type = self.get_operation_type(operation);
        let success_key = format!("{}_success_rate", operation_type);
        let count_key = format!("{}_count", operation_type);

        let mut stats = self.operation_stats.write().await;

        // Update count
        let current_count = stats.get(&count_key).unwrap_or(&0.0) + 1.0;
        stats.insert(count_key, current_count);

        // Update success rate
        let current_success_rate = stats.get(&success_key).unwrap_or(&0.0);
        let new_success_rate = if outcome.success {
            (current_success_rate * (current_count - 1.0) + 1.0) / current_count
        } else {
            (current_success_rate * (current_count - 1.0)) / current_count
        };
        stats.insert(success_key, new_success_rate);

        Ok(())
    }

    /// Find operation by ID in history
    async fn find_operation_by_id(&self, id: &str) -> Result<Option<IndexedOperation>> {
        let history = self.operation_history.read().await;
        Ok(history.iter().find(|op| op.id == id).cloned())
    }

    /// Calculate cosine similarity between two embeddings
    fn calculate_cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        (dot_product / (norm_a * norm_b)).clamp(0.0, 1.0)
    }

    /// Analyze why two operations are similar
    fn analyze_similarity_reasons(
        &self,
        query_op: &FileOperation,
        query_context: &OperationContext,
        similar_op: &IndexedOperation,
    ) -> Vec<String> {
        let mut reasons = Vec::new();

        // Check operation type similarity
        if self.get_operation_type(query_op) == self.get_operation_type(&similar_op.operation) {
            reasons.push("Same operation type".to_string());
        }

        // Check file extension similarity
        if let (Some(query_ext), Some(similar_ext)) = (
            self.get_file_extension(query_op),
            self.get_file_extension(&similar_op.operation),
        ) {
            if query_ext == similar_ext {
                reasons.push(format!("Same file type (.{})", query_ext));
            }
        }

        // Check repository similarity
        if query_context.repository_path == similar_op.context.repository_path {
            reasons.push("Same repository".to_string());
        }

        // Check question similarity (basic keyword matching)
        let query_words: std::collections::HashSet<String> = query_context
            .source_question
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let similar_words: std::collections::HashSet<String> = similar_op
            .context
            .source_question
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let common_words: Vec<String> = query_words.intersection(&similar_words).cloned().collect();
        if common_words.len() >= 2 {
            reasons.push(format!("Similar intent ({})", common_words.join(", ")));
        }

        if reasons.is_empty() {
            reasons.push("Semantic similarity".to_string());
        }

        reasons
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
}

/// Operation statistics
#[derive(Debug, Clone)]
pub struct OperationStats {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub success_rate: f32,
    pub average_execution_time: Duration,
    pub average_user_satisfaction: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::{
        FileClassification, IndexedFile, IndexedRepository, KnowledgeIndexer, RepositoryPrediction,
    };

    #[tokio::test]
    async fn test_indexer_creation() {
        // Test creating an indexer
    }
}
