//! AI Helper Ecosystem
//! 
//! This module implements the multi-layered AI architecture where specialized
//! open-source models handle infrastructure and knowledge management, allowing
//! the user's chosen consensus models to focus on their expertise.

pub mod knowledge_indexer;
pub mod context_retriever;
pub mod pattern_recognizer;
pub mod quality_analyzer;
pub mod knowledge_synthesizer;
pub mod vector_store;

use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;

// Re-export main components
pub use knowledge_indexer::KnowledgeIndexer;
pub use context_retriever::ContextRetriever;
pub use pattern_recognizer::PatternRecognizer;
pub use quality_analyzer::QualityAnalyzer;
pub use knowledge_synthesizer::KnowledgeSynthesizer;
pub use vector_store::ChromaVectorStore;

/// Processed knowledge from AI helpers
#[derive(Debug, Clone)]
pub struct ProcessedKnowledge {
    pub indexed: IndexedKnowledge,
    pub patterns: Vec<Pattern>,
    pub quality: QualityReport,
    pub insights: Vec<Insight>,
}

/// Indexed knowledge with embeddings
#[derive(Debug, Clone)]
pub struct IndexedKnowledge {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: serde_json::Value,
}

/// Discovered pattern
#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub confidence: f64,
    pub examples: Vec<String>,
}

/// Pattern types
#[derive(Debug, Clone)]
pub enum PatternType {
    Recurring,
    Evolution,
    Contradiction,
    Relationship,
    Insight,
}

/// Quality analysis report
#[derive(Debug, Clone)]
pub struct QualityReport {
    pub overall_score: f64,
    pub consistency_score: f64,
    pub completeness_score: f64,
    pub accuracy_score: f64,
    pub issues: Vec<QualityIssue>,
}

/// Quality issue
#[derive(Debug, Clone)]
pub struct QualityIssue {
    pub issue_type: String,
    pub description: String,
    pub severity: Severity,
}

/// Severity levels
#[derive(Debug, Clone)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Generated insight
#[derive(Debug, Clone)]
pub struct Insight {
    pub insight_type: InsightType,
    pub content: String,
    pub supporting_facts: Vec<String>,
    pub confidence: f64,
}

/// Insight types
#[derive(Debug, Clone)]
pub enum InsightType {
    Trend,
    Anomaly,
    Relationship,
    Prediction,
    Recommendation,
}

/// Stage-specific context
#[derive(Debug, Clone)]
pub struct StageContext {
    pub stage: crate::consensus::types::Stage,
    pub relevant_facts: Vec<String>,
    pub patterns: Vec<Pattern>,
    pub insights: Vec<Insight>,
    pub custom_guidance: Option<String>,
}

/// Main AI Helper Ecosystem coordinator
pub struct AIHelperEcosystem {
    /// CodeBERT/CodeT5+ for indexing
    pub knowledge_indexer: Arc<KnowledgeIndexer>,
    
    /// GraphCodeBERT + LangChain for retrieval
    pub context_retriever: Arc<ContextRetriever>,
    
    /// UniXcoder for pattern recognition
    pub pattern_recognizer: Arc<PatternRecognizer>,
    
    /// CodeT5+ for quality analysis
    pub quality_analyzer: Arc<QualityAnalyzer>,
    
    /// Local LLM for synthesis
    pub knowledge_synthesizer: Arc<KnowledgeSynthesizer>,
    
    /// Chroma for vector storage
    pub vector_store: Arc<ChromaVectorStore>,
    
    /// Shared state
    state: Arc<RwLock<HelperState>>,
}

/// Internal state for AI helpers
struct HelperState {
    /// Total facts indexed
    total_facts: usize,
    
    /// Total patterns discovered
    total_patterns: usize,
    
    /// Last processing time
    last_processing_time: Option<std::time::Duration>,
}

impl AIHelperEcosystem {
    /// Create a new AI Helper Ecosystem
    pub async fn new(database: Arc<crate::core::database::DatabaseManager>) -> Result<Self> {
        // Initialize vector store
        let vector_store = Arc::new(ChromaVectorStore::new().await?);
        
        // Initialize helpers
        let knowledge_indexer = Arc::new(KnowledgeIndexer::new(vector_store.clone()).await?);
        let context_retriever = Arc::new(ContextRetriever::new(vector_store.clone()).await?);
        let pattern_recognizer = Arc::new(PatternRecognizer::new().await?);
        let quality_analyzer = Arc::new(QualityAnalyzer::new().await?);
        let knowledge_synthesizer = Arc::new(KnowledgeSynthesizer::new().await?);
        
        let state = Arc::new(RwLock::new(HelperState {
            total_facts: 0,
            total_patterns: 0,
            last_processing_time: None,
        }));
        
        Ok(Self {
            knowledge_indexer,
            context_retriever,
            pattern_recognizer,
            quality_analyzer,
            knowledge_synthesizer,
            vector_store,
            state,
        })
    }
    
    /// Process Curator output through all helpers
    pub async fn process_curator_output(
        &self,
        curator_output: &str,
        source_question: &str,
        conversation_id: &str,
    ) -> Result<ProcessedKnowledge> {
        let start = std::time::Instant::now();
        
        // 1. Index the new knowledge
        let indexed = self.knowledge_indexer
            .index_output(curator_output, source_question, conversation_id)
            .await?;
        
        // 2. Find patterns with existing knowledge
        let patterns = self.pattern_recognizer
            .analyze_patterns(&indexed)
            .await?;
        
        // 3. Check quality and consistency
        let quality = self.quality_analyzer
            .evaluate_quality(&indexed, curator_output)
            .await?;
        
        // 4. Synthesize new insights
        let insights = self.knowledge_synthesizer
            .generate_insights(&indexed, &patterns, &quality)
            .await?;
        
        // Update state
        let mut state = self.state.write().await;
        state.total_facts += 1;
        state.total_patterns += patterns.len();
        state.last_processing_time = Some(start.elapsed());
        
        Ok(ProcessedKnowledge {
            indexed,
            patterns,
            quality,
            insights,
        })
    }
    
    /// Prepare context for a consensus stage
    pub async fn prepare_stage_context(
        &self,
        question: &str,
        stage: crate::consensus::types::Stage,
        context_limit: usize,
    ) -> Result<StageContext> {
        // Retrieve relevant context based on stage needs
        use crate::consensus::types::Stage;
        
        match stage {
            Stage::Generator => {
                // Generator needs broad context
                self.context_retriever
                    .get_generator_context(question, context_limit)
                    .await
            }
            Stage::Refiner => {
                // Refiner needs quality examples
                self.context_retriever
                    .get_refiner_context(question, context_limit)
                    .await
            }
            Stage::Validator => {
                // Validator needs contradictions and edge cases
                self.context_retriever
                    .get_validator_context(question, context_limit)
                    .await
            }
            Stage::Curator => {
                // Curator needs synthesis opportunities
                self.context_retriever
                    .get_curator_context(question, context_limit)
                    .await
            }
        }
    }
    
    /// Get system statistics
    pub async fn get_stats(&self) -> HelperStats {
        let state = self.state.read().await;
        HelperStats {
            total_facts: state.total_facts,
            total_patterns: state.total_patterns,
            last_processing_time: state.last_processing_time,
            vector_store_size: self.vector_store.get_size().await.unwrap_or(0),
        }
    }
}

/// Statistics about the AI Helper system
#[derive(Debug, Clone)]
pub struct HelperStats {
    pub total_facts: usize,
    pub total_patterns: usize,
    pub last_processing_time: Option<std::time::Duration>,
    pub vector_store_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ai_helper_creation() {
        // Test helper system initialization
    }
}