//! Consensus Memory System - Persistent Knowledge Accumulation
//! 
//! This module provides the infrastructure for storing, retrieving, and learning from
//! all consensus outputs, creating a persistent knowledge layer accessible to all AI models.

pub mod authoritative_store;
pub mod context_injector;
pub mod continuous_learner;
pub mod model_bridge;
pub mod semantic_fingerprint;

use std::sync::Arc;
use anyhow::Result;

pub use authoritative_store::{AuthoritativeKnowledgeStore, CuratedFact};
pub use context_injector::{ContextInjector, InjectedContext};
pub use continuous_learner::{ContinuousLearner, LearningResult};
pub use model_bridge::{ModelMemoryBridge, ModelContext};
pub use semantic_fingerprint::{SemanticFingerprint, SemanticFingerprinter};

/// Main consensus memory system that coordinates all components
pub struct ConsensusMemory {
    /// Stores curator-validated knowledge
    pub knowledge_store: Arc<AuthoritativeKnowledgeStore>,
    
    /// Injects relevant context for AI models
    pub context_injector: Arc<ContextInjector>,
    
    /// Learns from every interaction
    pub continuous_learner: Arc<ContinuousLearner>,
    
    /// Provides memory access to all models
    pub model_bridge: Arc<ModelMemoryBridge>,
    
    /// Creates semantic fingerprints
    pub fingerprinter: Arc<SemanticFingerprinter>,
}

impl ConsensusMemory {
    /// Create a new consensus memory system
    pub async fn new(database: Arc<crate::core::database::DatabaseManager>) -> Result<Self> {
        // Initialize components
        let fingerprinter = Arc::new(SemanticFingerprinter::new().await?);
        let knowledge_store = Arc::new(AuthoritativeKnowledgeStore::new(database.clone(), fingerprinter.clone()).await?);
        let context_injector = Arc::new(ContextInjector::new(knowledge_store.clone()).await?);
        let continuous_learner = Arc::new(ContinuousLearner::new(knowledge_store.clone()).await?);
        let model_bridge = Arc::new(ModelMemoryBridge::new(knowledge_store.clone()).await?);
        
        Ok(Self {
            knowledge_store,
            context_injector,
            continuous_learner,
            model_bridge,
            fingerprinter,
        })
    }
    
    /// Process and store curator output
    pub async fn store_curator_output(
        &self,
        curator_output: &str,
        source_question: &str,
        consensus_stages: Vec<crate::consensus::types::StageResult>,
    ) -> Result<LearningResult> {
        // Extract facts and learn
        let learning_result = self.continuous_learner
            .learn_from_curator(curator_output, source_question, consensus_stages)
            .await?;
            
        tracing::info!(
            "Stored {} new facts, detected {} patterns, built {} relationships",
            learning_result.facts_added,
            learning_result.patterns_detected,
            learning_result.relationships_built
        );
        
        Ok(learning_result)
    }
    
    /// Get context for a specific model and question
    pub async fn get_context_for_model(
        &self,
        model_id: &str,
        question: &str,
        stage: crate::consensus::types::Stage,
        context_limit: usize,
    ) -> Result<ModelContext> {
        self.model_bridge
            .get_model_context(model_id, question, stage, context_limit)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_consensus_memory_creation() {
        // Test memory system initialization
        // Will add comprehensive tests
    }
}