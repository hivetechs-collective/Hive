//! Enhanced memory system with intelligent vector embeddings and knowledge graphs
//!
//! This module provides advanced memory capabilities for HiveTechs Consensus:
//! - Vector embeddings for semantic search using candle
//! - Dynamic knowledge graph construction with petgraph
//! - Pattern learning with machine learning
//! - Context retrieval engine for relevant memory access
//! - Memory analytics and insights dashboard

pub mod analytics;
pub mod embeddings;
pub mod knowledge_graph;
pub mod pattern_learning;
pub mod retrieval;
pub mod thematic_clustering;
pub mod topic_extraction;

#[cfg(all(test, feature = "legacy-tests"))]
mod test;

use anyhow::{Context as _, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

pub use analytics::{InsightGenerator, MemoryAnalyzer, MemoryMetrics};
pub use embeddings::{EmbeddingEngine, SimilarityMetric, VectorStore};
pub use knowledge_graph::{Entity, GraphQuery, KnowledgeGraph, Relationship};
pub use pattern_learning::{Pattern, PatternLearner, PatternMetrics, PatternType};
pub use retrieval::{ContextRetriever, ContextWindow, RetrievalStrategy};
pub use thematic_clustering::{ConversationThread, MessageImportance, ThematicCluster, ThreadType};
pub use topic_extraction::{
    extract_topics, extract_topics_with_ai, find_conversations_by_topic, find_topics_for_query,
    tag_conversation,
};

// Re-export core memory types for convenience
pub use crate::core::memory::{
    get_memory_system, initialize_memory, InsightType, MemoryConfig, MemoryInsight, MemorySystem,
    SemanticSearchResult,
};

/// Enhanced memory intelligence system
#[derive(Debug)]
pub struct MemoryIntelligence {
    /// Vector embedding engine
    pub embeddings: Arc<EmbeddingEngine>,
    /// Knowledge graph
    pub graph: Arc<RwLock<KnowledgeGraph>>,
    /// Pattern learning system
    pub patterns: Arc<RwLock<PatternLearner>>,
    /// Context retrieval engine
    pub retriever: Arc<ContextRetriever>,
    /// Analytics engine
    pub analytics: Arc<RwLock<MemoryAnalyzer>>,
    /// Thematic clustering engine
    pub thematic: Arc<ThematicCluster>,
}

impl MemoryIntelligence {
    /// Create a new memory intelligence system
    pub async fn new() -> Result<Self> {
        info!("Initializing enhanced memory intelligence system");

        // Initialize components
        let embeddings = Arc::new(EmbeddingEngine::new().await?);
        let graph = Arc::new(RwLock::new(KnowledgeGraph::new()));
        let patterns = Arc::new(RwLock::new(PatternLearner::new()));
        let retriever = Arc::new(ContextRetriever::new(embeddings.clone()));
        let analytics = Arc::new(RwLock::new(MemoryAnalyzer::new()));
        let thematic = Arc::new(ThematicCluster::new().await?);

        Ok(Self {
            embeddings,
            graph,
            patterns,
            retriever,
            analytics,
            thematic,
        })
    }

    /// Perform intelligent semantic search
    pub async fn search(
        &self,
        query: &str,
        strategy: RetrievalStrategy,
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        debug!(
            "Performing intelligent search with strategy: {:?}",
            strategy
        );

        // Get embeddings for the query
        let query_embedding = self.embeddings.encode(query).await?;

        // Retrieve similar memories
        let results = self
            .retriever
            .retrieve(&query_embedding, strategy, limit)
            .await?;

        // Enrich with graph relationships
        let mut enriched_results = Vec::new();
        for result in results {
            let relationships = self
                .graph
                .read()
                .await
                .find_relationships(&result.id)
                .await?;

            enriched_results.push(SemanticSearchResult {
                relationships: relationships
                    .into_iter()
                    .map(|r| format!("{:?} -> {}", r.relation_type, r.target))
                    .collect(),
                ..result
            });
        }

        // Update analytics
        self.analytics
            .write()
            .await
            .record_search(query, &enriched_results)
            .await?;

        Ok(enriched_results)
    }

    /// Build comprehensive knowledge graph
    pub async fn build_graph(&self) -> Result<()> {
        info!("Building comprehensive knowledge graph");

        let mut graph = self.graph.write().await;

        // Get all memories from the system
        let memory_system = get_memory_system().await?;

        // Extract entities and build relationships
        // This would process all memories and extract entities
        graph.build_from_memories(&memory_system).await?;

        Ok(())
    }

    /// Learn patterns from memory usage
    pub async fn learn_patterns(&self) -> Result<Vec<Pattern>> {
        info!("Learning patterns from memory usage");

        let mut learner = self.patterns.write().await;

        // Get memory system
        let memory_system = get_memory_system().await?;

        // Process memories for pattern learning
        learner.learn_from_memories(&memory_system).await?;

        // Return discovered patterns
        Ok(learner.get_patterns())
    }

    /// Generate comprehensive insights
    pub async fn generate_insights(&self) -> Result<Vec<MemoryInsight>> {
        info!("Generating comprehensive memory insights");

        let analyzer = self.analytics.read().await;
        let generator = InsightGenerator::new();

        // Generate insights from various sources
        let mut insights = Vec::new();

        // Pattern-based insights
        let patterns = self.patterns.read().await;
        insights.extend(generator.from_patterns(patterns.get_patterns()).await?);

        // Graph-based insights
        let graph = self.graph.read().await;
        insights.extend(generator.from_graph(&*graph).await?);

        // Usage-based insights
        insights.extend(generator.from_analytics(&*analyzer).await?);

        Ok(insights)
    }

    /// Get memory metrics
    pub async fn get_metrics(&self) -> Result<MemoryMetrics> {
        let analytics = self.analytics.read().await;
        Ok(analytics.get_metrics())
    }

    /// Find relevant knowledge for consensus pipeline stage
    pub async fn find_stage_knowledge(&self, query: &str, stage: &str) -> Result<String> {
        self.thematic
            .find_relevant_knowledge_for_ai(query, stage)
            .await
    }

    /// Detect follow-up relationships
    pub async fn detect_follow_up(
        &self,
        query: &str,
        recent_conversations: &[String],
    ) -> Result<Option<ConversationThread>> {
        self.thematic
            .detect_follow_up(query, recent_conversations)
            .await
    }

    /// Get curator knowledge base
    pub async fn get_curator_knowledge(&self, query: &str) -> Result<String> {
        self.thematic.get_curator_knowledge_base(query).await
    }

    /// Get conversation clusters by topic
    pub async fn get_conversation_clusters(
        &self,
        limit: Option<usize>,
    ) -> Result<HashMap<String, Vec<String>>> {
        self.thematic.get_conversation_clusters(limit).await
    }

    /// Tag a conversation with topics
    pub async fn tag_conversation(&self, conversation_id: &str) -> Result<Vec<String>> {
        tag_conversation(conversation_id).await
    }
}

/// Global memory intelligence instance
static MEMORY_INTELLIGENCE: tokio::sync::OnceCell<Arc<MemoryIntelligence>> =
    tokio::sync::OnceCell::const_new();

/// Initialize the memory intelligence system
pub async fn initialize_intelligence() -> Result<()> {
    let intelligence = Arc::new(MemoryIntelligence::new().await?);

    MEMORY_INTELLIGENCE
        .set(intelligence)
        .map_err(|_| anyhow::anyhow!("Memory intelligence already initialized"))?;

    Ok(())
}

/// Get the global memory intelligence instance
pub async fn get_intelligence() -> Result<Arc<MemoryIntelligence>> {
    MEMORY_INTELLIGENCE
        .get()
        .ok_or_else(|| anyhow::anyhow!("Memory intelligence not initialized"))
        .map(Arc::clone)
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_intelligence_creation() -> Result<()> {
        let intelligence = MemoryIntelligence::new().await?;

        // Verify all components are initialized
        let metrics = intelligence.get_metrics().await?;
        assert_eq!(metrics.total_memories, 0);

        Ok(())
    }
}
