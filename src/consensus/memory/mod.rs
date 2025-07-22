//! Consensus Memory System - Persistent Knowledge Accumulation
//! 
//! This module provides the infrastructure for storing, retrieving, and learning from
//! all consensus outputs, creating a persistent knowledge layer accessible to all AI models.

pub mod authoritative_store;
pub mod context_injector;
pub mod semantic_fingerprint;

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub use authoritative_store::{AuthoritativeKnowledgeStore, CuratedFact, StageContribution};
pub use context_injector::{ContextInjector, InjectedContext};
pub use semantic_fingerprint::{SemanticFingerprint, SemanticFingerprinter};

// Temporary structures until full implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningResult {
    pub facts_added: usize,
    pub patterns_detected: usize,
    pub relationships_built: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelContext {
    pub stage: crate::consensus::types::Stage,
    pub context: String,
    pub metadata: serde_json::Value,
}

/// Main consensus memory system that coordinates all components
pub struct ConsensusMemory {
    /// Stores curator-validated knowledge
    pub knowledge_store: Arc<AuthoritativeKnowledgeStore>,
    
    /// Injects relevant context for AI models
    pub context_injector: Arc<ContextInjector>,
    
    /// Creates semantic fingerprints
    pub fingerprinter: Arc<SemanticFingerprinter>,
}

impl ConsensusMemory {
    /// Create a new consensus memory system
    pub async fn new(database: Arc<crate::core::database::DatabaseManager>) -> Result<Self> {
        // Create AI helpers for context injection
        let ai_helpers = Arc::new(crate::ai_helpers::AIHelperEcosystem::new(database.clone()).await?);
        
        // Initialize components
        let fingerprinter = Arc::new(SemanticFingerprinter::new().await?);
        let knowledge_store = Arc::new(AuthoritativeKnowledgeStore::new(database.clone(), fingerprinter.clone()).await?);
        let context_injector = Arc::new(ContextInjector::new(knowledge_store.clone(), ai_helpers).await?);
        
        Ok(Self {
            knowledge_store,
            context_injector,
            fingerprinter,
        })
    }
    
    /// Connect AI helpers to the hive mind by giving them access to the knowledge store
    pub async fn connect_hive_mind(&self, ai_helpers: &Arc<crate::ai_helpers::AIHelperEcosystem>) -> Result<()> {
        // Give the context retriever access to the authoritative knowledge store
        ai_helpers.context_retriever
            .update_knowledge_store(Some(self.knowledge_store.clone()))
            .await?;
        
        tracing::info!("🐝🧠 Hive mind connected! All consensus stages now share collective knowledge");
        Ok(())
    }
    
    /// Process and store curator output (simplified for now)
    pub async fn store_curator_output(
        &self,
        curator_output: &str,
        source_question: &str,
        consensus_stages: Vec<crate::consensus::types::StageResult>,
    ) -> Result<LearningResult> {
        // Store the curator output as a fact with full metadata
        let fact = CuratedFact {
            id: uuid::Uuid::new_v4().to_string(),
            content: curator_output.to_string(),
            semantic_fingerprint: self.fingerprinter.fingerprint(curator_output).await?.to_string(),
            curator_confidence: 0.9, // High confidence for curator output
            source_question: source_question.to_string(),
            source_conversation_id: Some(uuid::Uuid::new_v4().to_string()),
            consensus_stages: consensus_stages.iter().map(|s| StageContribution {
                stage: s.stage_name.clone(),
                model: s.model.clone(),
                contribution: s.answer.clone(),
                confidence: s.analytics.as_ref().map(|a| a.quality_score).unwrap_or(0.8),
            }).collect(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            access_count: 1,
            related_facts: self.find_related_facts(curator_output).await.unwrap_or_default(),
            topics: self.extract_topics(curator_output, source_question).await.unwrap_or_else(|_| vec!["general".to_string()]),
            entities: self.extract_entities(curator_output).await.unwrap_or_default(),
            metadata: crate::consensus::memory::authoritative_store::FactMetadata {
                fact_type: crate::consensus::memory::authoritative_store::FactType::Other("curator_output".to_string()),
                domain: None,
                language: "en".to_string(),
                complexity_score: 0.5, // TODO: Calculate complexity
                verification_status: crate::consensus::memory::authoritative_store::VerificationStatus::Verified,
            },
        };
        
        self.knowledge_store.store_fact(fact).await?;
        
        Ok(LearningResult {
            facts_added: 1,
            patterns_detected: 0, // TODO: Implement pattern detection
            relationships_built: 0, // TODO: Implement relationship building
        })
    }
    
    /// Get context for a specific model and question
    pub async fn get_context_for_model(
        &self,
        _model_id: &str,
        question: &str,
        stage: crate::consensus::types::Stage,
        context_limit: usize,
    ) -> Result<ModelContext> {
        // Use the context injector to get relevant context
        let injected = self.context_injector
            .inject_context(question, stage, None)
            .await?;
        
        // Limit context size
        let context = if injected.formatted_context.len() > context_limit {
            injected.formatted_context.chars().take(context_limit).collect()
        } else {
            injected.formatted_context
        };
        
        Ok(ModelContext {
            stage,
            context,
            metadata: serde_json::json!({
                "facts_injected": injected.facts.len(),
                "processing_time_ms": injected.metadata.processing_time_ms,
            }),
        })
    }
    
    /// Find facts related to the given content
    async fn find_related_facts(&self, content: &str) -> Result<Vec<String>> {
        // Find similar facts using the knowledge store
        let similar_facts = self.knowledge_store.find_similar(content, 3).await?;
        Ok(similar_facts.into_iter().map(|f| f.id).collect())
    }
    
    /// Extract topics from content
    async fn extract_topics(&self, content: &str, question: &str) -> Result<Vec<String>> {
        // Use simple topic extraction based on keywords
        let mut topics = Vec::new();
        
        let content_lower = content.to_lowercase();
        let question_lower = question.to_lowercase();
        
        // Common programming topics
        if content_lower.contains("rust") || question_lower.contains("rust") {
            topics.push("rust".to_string());
        }
        if content_lower.contains("typescript") || content_lower.contains("javascript") {
            topics.push("javascript".to_string());
        }
        if content_lower.contains("python") {
            topics.push("python".to_string());
        }
        
        // Architecture topics
        if content_lower.contains("architecture") || content_lower.contains("design") {
            topics.push("architecture".to_string());
        }
        if content_lower.contains("database") || content_lower.contains("sqlite") {
            topics.push("database".to_string());
        }
        
        // AI topics
        if content_lower.contains("ai") || content_lower.contains("artificial intelligence") || content_lower.contains("consensus") {
            topics.push("ai".to_string());
        }
        
        if topics.is_empty() {
            topics.push("general".to_string());
        }
        
        Ok(topics)
    }
    
    /// Extract entities from content
    async fn extract_entities(&self, content: &str) -> Result<Vec<crate::consensus::memory::authoritative_store::Entity>> {
        let mut entities = Vec::new();
        
        // Extract file paths
        let file_regex = regex::Regex::new(r"\b[\w/]+\.(rs|ts|js|py|toml|json|md)\b").unwrap();
        for cap in file_regex.captures_iter(content) {
            if let Some(path) = cap.get(0) {
                entities.push(crate::consensus::memory::authoritative_store::Entity {
                    name: path.as_str().to_string(),
                    entity_type: "file".to_string(),
                    confidence: 0.9,
                });
            }
        }
        
        // Extract module/package names
        let module_regex = regex::Regex::new(r"\b(crate::|mod |module |package )([\w_]+)\b").unwrap();
        for cap in module_regex.captures_iter(content) {
            if let Some(name) = cap.get(2) {
                entities.push(crate::consensus::memory::authoritative_store::Entity {
                    name: name.as_str().to_string(),
                    entity_type: "module".to_string(),
                    confidence: 0.8,
                });
            }
        }
        
        // Extract function names
        let func_regex = regex::Regex::new(r"\b(fn |function |def |async fn )([\w_]+)\b").unwrap();
        for cap in func_regex.captures_iter(content) {
            if let Some(name) = cap.get(2) {
                entities.push(crate::consensus::memory::authoritative_store::Entity {
                    name: name.as_str().to_string(),
                    entity_type: "function".to_string(),
                    confidence: 0.8,
                });
            }
        }
        
        Ok(entities)
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