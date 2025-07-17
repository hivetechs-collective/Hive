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
pub mod parallel_processor;
pub mod model_downloader;
pub mod monitoring;

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
pub use parallel_processor::{ParallelProcessor, ParallelConfig};
pub use model_downloader::{ModelDownloader, DownloaderConfig, ModelInfo, DownloadEvent};
pub use monitoring::{PerformanceMonitor, MonitoringConfig, OperationType, PerformanceStats};

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
    
    /// Parallel processor for performance
    parallel_processor: ParallelProcessor,
    
    /// Performance monitor
    performance_monitor: Arc<PerformanceMonitor>,
    
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
        // First, ensure all required models are downloaded
        Self::ensure_models_available().await?;
        
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
        
        // Create parallel processor with default config
        let parallel_processor = ParallelProcessor::new(ParallelConfig::default());
        
        // Create performance monitor
        let performance_monitor = Arc::new(PerformanceMonitor::new(MonitoringConfig::default()));
        
        Ok(Self {
            knowledge_indexer,
            context_retriever,
            pattern_recognizer,
            quality_analyzer,
            knowledge_synthesizer,
            vector_store,
            parallel_processor,
            performance_monitor,
            state,
        })
    }
    
    /// Ensure all required models are available
    async fn ensure_models_available() -> Result<()> {
        let downloader = ModelDownloader::new(DownloaderConfig::default()).await?;
        let missing = downloader.check_missing_models().await;
        
        if !missing.is_empty() {
            tracing::info!("First-time setup: downloading {} AI helper models", missing.len());
            tracing::info!("This may take a while depending on your internet connection...");
            
            downloader.initialize_models().await?;
            
            tracing::info!("✓ All AI helper models downloaded successfully");
        }
        
        Ok(())
    }
    
    /// Process Curator output through all helpers
    pub async fn process_curator_output(
        &self,
        curator_output: &str,
        source_question: &str,
        conversation_id: &str,
    ) -> Result<ProcessedKnowledge> {
        // Start monitoring the overall operation
        let op_id = self.performance_monitor
            .start_operation(OperationType::ParallelProcessing, "AIHelperEcosystem")
            .await;
        
        let result = async {
            let start = std::time::Instant::now();
            
            // 1. Index the new knowledge (with monitoring)
            let index_op_id = self.performance_monitor
                .start_operation(OperationType::IndexKnowledge, "KnowledgeIndexer")
                .await;
            
            let indexed = match self.knowledge_indexer
                .index_output(curator_output, source_question, conversation_id)
                .await
            {
                Ok(indexed) => {
                    self.performance_monitor.complete_operation(
                        &index_op_id, true, None, serde_json::json!({})
                    ).await?;
                    indexed
                }
                Err(e) => {
                    self.performance_monitor.complete_operation(
                        &index_op_id, false, Some(e.to_string()), serde_json::json!({})
                    ).await?;
                    return Err(e);
                }
            };
            
            // 2. Process patterns, quality, and insights in parallel
            let parallel_result = self.parallel_processor
                .process_parallel(
                    &indexed,
                    curator_output,
                    self.pattern_recognizer.clone(),
                    self.quality_analyzer.clone(),
                    self.knowledge_synthesizer.clone(),
                )
                .await?;
            
            // Update state
            let mut state = self.state.write().await;
            state.total_facts += 1;
            state.total_patterns += parallel_result.patterns.len();
            state.last_processing_time = Some(start.elapsed());
            
            tracing::info!(
                "Processed curator output in {:?} with {:.2}x parallel speedup",
                parallel_result.processing_time,
                parallel_result.parallel_speedup
            );
            
            Ok(ProcessedKnowledge {
                indexed,
                patterns: parallel_result.patterns,
                quality: parallel_result.quality,
                insights: parallel_result.insights,
            })
        }.await;
        
        // Complete monitoring
        match &result {
            Ok(_) => {
                self.performance_monitor.complete_operation(
                    &op_id, true, None, 
                    serde_json::json!({
                        "source_length": curator_output.len(),
                        "conversation_id": conversation_id
                    })
                ).await?;
            }
            Err(e) => {
                self.performance_monitor.complete_operation(
                    &op_id, false, Some(e.to_string()), serde_json::json!({})
                ).await?;
            }
        }
        
        result
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
    
    /// Process multiple curator outputs in batch
    pub async fn process_batch(
        &self,
        outputs: Vec<(String, String, String)>, // (curator_output, source_question, conversation_id)
    ) -> Result<Vec<ProcessedKnowledge>> {
        tracing::info!("Processing batch of {} curator outputs", outputs.len());
        
        let batch_results = self.parallel_processor
            .process_batch(
                outputs,
                self.knowledge_indexer.clone(),
                self.pattern_recognizer.clone(),
                self.quality_analyzer.clone(),
                self.knowledge_synthesizer.clone(),
            )
            .await?;
        
        // Convert to ProcessedKnowledge format
        let processed: Vec<ProcessedKnowledge> = batch_results
            .into_iter()
            .map(|(indexed, result)| ProcessedKnowledge {
                indexed,
                patterns: result.patterns,
                quality: result.quality,
                insights: result.insights,
            })
            .collect();
        
        // Update state
        let mut state = self.state.write().await;
        state.total_facts += processed.len();
        for p in &processed {
            state.total_patterns += p.patterns.len();
        }
        
        Ok(processed)
    }
    
    /// Get system statistics
    pub async fn get_stats(&self) -> HelperStats {
        let state = self.state.read().await;
        let parallel_stats = self.parallel_processor.get_stats().await;
        let perf_stats = self.performance_monitor.get_stats().await;
        
        HelperStats {
            total_facts: state.total_facts,
            total_patterns: state.total_patterns,
            last_processing_time: state.last_processing_time,
            vector_store_size: self.vector_store.get_size().await.unwrap_or(0),
            parallel_speedup: parallel_stats.parallel_speedup,
            tasks_completed: parallel_stats.tasks_completed,
            tasks_failed: parallel_stats.tasks_failed,
            performance_stats: Some(perf_stats),
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
    pub parallel_speedup: f64,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub performance_stats: Option<PerformanceStats>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ai_helper_creation() {
        // Test helper system initialization
    }
}