//! Continuous Learner - Learns from every interaction and outcome
//! 
//! This module implements continuous learning from all consensus interactions,
//! building a knowledge base that enhances future consensus decisions.
//! It tracks patterns, outcomes, and user feedback to improve over time.

use std::sync::Arc;
use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use chrono::{DateTime, Utc};

use crate::ai_helpers::{ChromaVectorStore, Pattern, PatternType};
use crate::consensus::types::{Stage, StageResult};
use crate::consensus::operation_intelligence::{OperationOutcome, OperationContext};
use super::python_models::PythonModelService;

/// Learning event types that trigger continuous learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningEvent {
    /// A consensus stage completed successfully
    StageCompleted {
        stage: String,
        question: String,
        answer: String,
        model: String,
        duration_ms: u64,
        tokens_used: u64,
    },
    
    /// User provided feedback on a response
    UserFeedback {
        conversation_id: String,
        stage: String,
        feedback_type: FeedbackType,
        details: Option<String>,
    },
    
    /// An operation was executed (file creation, etc.)
    OperationExecuted {
        operation_type: String,
        success: bool,
        context: OperationContext,
        outcome: OperationOutcome,
    },
    
    /// A pattern was detected
    PatternDetected {
        pattern_type: String,
        confidence: f64,
        examples: Vec<String>,
    },
    
    /// Model performance observation
    ModelPerformance {
        model: String,
        stage: String,
        success_rate: f64,
        avg_quality_score: f64,
    },
}

/// Types of user feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackType {
    Positive,
    Negative,
    Correction(String),
    Suggestion(String),
}

/// Learned knowledge from continuous learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedKnowledge {
    /// ID for this piece of knowledge
    pub id: String,
    
    /// The learning event that created this knowledge
    pub event: LearningEvent,
    
    /// Timestamp when learned
    pub learned_at: DateTime<Utc>,
    
    /// Embedding for similarity search
    pub embedding: Vec<f32>,
    
    /// Patterns extracted from this learning
    pub patterns: Vec<LearnedPattern>,
    
    /// Confidence in this knowledge (0-1)
    pub confidence: f64,
    
    /// Times this knowledge has been successfully applied
    pub application_count: u32,
    
    /// Success rate when applied (0-1)
    pub success_rate: f64,
}

/// A pattern learned from interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Pattern description
    pub description: String,
    
    /// Pattern category
    pub category: PatternCategory,
    
    /// Confidence in pattern (0-1)
    pub confidence: f64,
    
    /// Conditions when pattern applies
    pub conditions: Vec<PatternCondition>,
    
    /// Recommended actions based on pattern
    pub recommendations: Vec<String>,
}

/// Categories of learned patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternCategory {
    /// Question type patterns (e.g., "debug questions often need validator stage")
    QuestionType,
    
    /// Model behavior patterns (e.g., "GPT-4 better for code generation")
    ModelBehavior,
    
    /// User preference patterns (e.g., "user prefers detailed explanations")
    UserPreference,
    
    /// Operation success patterns (e.g., "file creates in src/ usually succeed")
    OperationSuccess,
    
    /// Error patterns (e.g., "permission errors common in system directories")
    ErrorPattern,
    
    /// Performance patterns (e.g., "complex questions take 2x time")
    Performance,
}

/// Conditions for pattern application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
    Matches,
}

/// Enhanced context based on continuous learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedContext {
    /// Relevant past experiences
    pub past_experiences: Vec<PastExperience>,
    
    /// Applicable patterns
    pub applicable_patterns: Vec<LearnedPattern>,
    
    /// Model recommendations based on history
    pub model_recommendations: HashMap<String, ModelRecommendation>,
    
    /// Potential pitfalls to avoid
    pub warnings: Vec<LearningWarning>,
    
    /// Success strategies that worked before
    pub success_strategies: Vec<String>,
}

/// A relevant past experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastExperience {
    pub question_similarity: f64,
    pub outcome: String,
    pub key_insights: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Model recommendation based on learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub model: String,
    pub stage: String,
    pub confidence: f64,
    pub reasoning: String,
    pub historical_performance: f64,
}

/// Warning based on learned patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningWarning {
    pub severity: LearningWarningSeverity,
    pub message: String,
    pub pattern_source: String,
    pub avoidance_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningWarningSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Continuous learner that builds knowledge from all interactions
pub struct ContinuousLearner {
    /// Vector store for learned knowledge
    vector_store: Arc<ChromaVectorStore>,
    
    /// Python model service for embeddings and analysis
    python_service: Arc<PythonModelService>,
    
    /// In-memory cache of recent learnings
    recent_learnings: Arc<RwLock<Vec<LearnedKnowledge>>>,
    
    /// Pattern detection thresholds
    pattern_thresholds: PatternThresholds,
    
    /// Learning statistics
    stats: Arc<RwLock<LearningStats>>,
}

/// Thresholds for pattern detection
#[derive(Debug, Clone)]
struct PatternThresholds {
    min_confidence: f64,
    min_examples: usize,
    decay_factor: f64,
}

impl Default for PatternThresholds {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            min_examples: 3,
            decay_factor: 0.95, // Slight decay for old patterns
        }
    }
}

/// Statistics about continuous learning
#[derive(Debug, Clone, Default)]
pub struct LearningStats {
    pub total_events: u64,
    pub total_patterns: u64,
    pub successful_applications: u64,
    pub failed_applications: u64,
}

impl ContinuousLearner {
    /// Create a new continuous learner
    pub async fn new(
        vector_store: Arc<ChromaVectorStore>,
        python_service: Arc<PythonModelService>,
    ) -> Result<Self> {
        Ok(Self {
            vector_store,
            python_service,
            recent_learnings: Arc::new(RwLock::new(Vec::new())),
            pattern_thresholds: PatternThresholds::default(),
            stats: Arc::new(RwLock::new(LearningStats::default())),
        })
    }
    
    /// Learn from a new event
    pub async fn learn_from_event(&self, event: LearningEvent) -> Result<()> {
        info!("🧠 Learning from event: {:?}", event);
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_events += 1;
        }
        
        // Create embedding for the event
        let embedding = self.create_event_embedding(&event).await?;
        
        // Extract patterns from the event
        let patterns = self.extract_patterns_from_event(&event).await?;
        
        // Create learned knowledge
        let knowledge = LearnedKnowledge {
            id: uuid::Uuid::new_v4().to_string(),
            event: event.clone(),
            learned_at: Utc::now(),
            embedding,
            patterns: patterns.clone(),
            confidence: self.calculate_initial_confidence(&event),
            application_count: 0,
            success_rate: 0.0,
        };
        
        // Store in vector store
        self.store_knowledge(&knowledge).await?;
        
        // Update recent learnings cache
        {
            let mut recent = self.recent_learnings.write().await;
            recent.push(knowledge.clone());
            
            // Keep only last 100 learnings in memory
            if recent.len() > 100 {
                let drain_count = recent.len() - 100;
                recent.drain(0..drain_count);
            }
        }
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_patterns += patterns.len() as u64;
        }
        
        info!("✅ Learned {} new patterns from event", patterns.len());
        
        Ok(())
    }
    
    /// Get enhanced context for a consensus stage based on learning
    pub async fn get_learned_context(
        &self,
        question: &str,
        stage: Stage,
        context_limit: usize,
    ) -> Result<LearnedContext> {
        debug!("Getting learned context for stage {:?}", stage);
        
        // Find similar past experiences
        let past_experiences = self.find_similar_experiences(question, 5).await?;
        
        // Find applicable patterns
        let applicable_patterns = self.find_applicable_patterns(question, stage).await?;
        
        // Get model recommendations
        let model_recommendations = self.get_model_recommendations(stage).await?;
        
        // Identify potential warnings
        let warnings = self.identify_warnings(&past_experiences, &applicable_patterns).await?;
        
        // Extract success strategies
        let success_strategies = self.extract_success_strategies(&past_experiences).await?;
        
        Ok(LearnedContext {
            past_experiences,
            applicable_patterns,
            model_recommendations,
            warnings,
            success_strategies,
        })
    }
    
    /// Apply feedback to update learned knowledge
    pub async fn apply_feedback(
        &self,
        conversation_id: &str,
        feedback: FeedbackType,
    ) -> Result<()> {
        info!("Applying feedback for conversation {}: {:?}", conversation_id, feedback);
        
        // Find related knowledge
        let related_knowledge = self.find_knowledge_by_conversation(conversation_id).await?;
        
        // Update confidence and success rates based on feedback
        for mut knowledge in related_knowledge {
            match &feedback {
                FeedbackType::Positive => {
                    knowledge.confidence = (knowledge.confidence * 1.1).min(1.0);
                    knowledge.success_rate = 
                        (knowledge.success_rate * knowledge.application_count as f64 + 1.0) / 
                        (knowledge.application_count + 1) as f64;
                }
                FeedbackType::Negative => {
                    knowledge.confidence *= 0.9;
                    knowledge.success_rate = 
                        (knowledge.success_rate * knowledge.application_count as f64) / 
                        (knowledge.application_count + 1) as f64;
                }
                FeedbackType::Correction(correction) => {
                    // Learn from the correction
                    knowledge.confidence *= 0.8;
                    // TODO: Store correction as new pattern
                }
                FeedbackType::Suggestion(suggestion) => {
                    // Learn from the suggestion
                    // TODO: Store suggestion as new pattern
                }
            }
            
            knowledge.application_count += 1;
            
            // Update in vector store
            self.store_knowledge(&knowledge).await?;
        }
        
        Ok(())
    }
    
    /// Create embedding for a learning event
    async fn create_event_embedding(&self, event: &LearningEvent) -> Result<Vec<f32>> {
        let event_text = self.event_to_text(event);
        
        // Use the python service to generate embeddings
        let embeddings = self.python_service
            .generate_embeddings("text-embedding-ada-002", vec![event_text])
            .await
            .context("Failed to create embedding")?;
        
        embeddings.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No embeddings returned"))
    }
    
    /// Convert event to text for embedding
    fn event_to_text(&self, event: &LearningEvent) -> String {
        match event {
            LearningEvent::StageCompleted { stage, question, answer, model, .. } => {
                format!("Stage: {}, Question: {}, Answer: {}, Model: {}", stage, question, answer, model)
            }
            LearningEvent::UserFeedback { stage, feedback_type, details, .. } => {
                format!("Feedback on {}: {:?} - {:?}", stage, feedback_type, details)
            }
            LearningEvent::OperationExecuted { operation_type, success, .. } => {
                format!("Operation: {}, Success: {}", operation_type, success)
            }
            LearningEvent::PatternDetected { pattern_type, confidence, .. } => {
                format!("Pattern: {}, Confidence: {}", pattern_type, confidence)
            }
            LearningEvent::ModelPerformance { model, stage, success_rate, .. } => {
                format!("Model {} on {}: {}% success", model, stage, success_rate * 100.0)
            }
        }
    }
    
    /// Extract patterns from an event
    async fn extract_patterns_from_event(&self, event: &LearningEvent) -> Result<Vec<LearnedPattern>> {
        let mut patterns = Vec::new();
        
        match event {
            LearningEvent::StageCompleted { stage, question, model, duration_ms, .. } => {
                // Pattern: Question complexity vs duration
                if *duration_ms > 5000 {
                    patterns.push(LearnedPattern {
                        description: format!("Complex questions in {} stage take longer", stage),
                        category: PatternCategory::Performance,
                        confidence: 0.8,
                        conditions: vec![
                            PatternCondition {
                                field: "stage".to_string(),
                                operator: ConditionOperator::Equals,
                                value: serde_json::json!(stage),
                            },
                            PatternCondition {
                                field: "question_length".to_string(),
                                operator: ConditionOperator::GreaterThan,
                                value: serde_json::json!(100),
                            },
                        ],
                        recommendations: vec![
                            "Consider using streaming for better UX".to_string(),
                            "Pre-process complex questions".to_string(),
                        ],
                    });
                }
            }
            LearningEvent::OperationExecuted { operation_type, success, context, .. } => {
                if !success {
                    patterns.push(LearnedPattern {
                        description: format!("{} operations may fail in this context", operation_type),
                        category: PatternCategory::ErrorPattern,
                        confidence: 0.7,
                        conditions: vec![
                            PatternCondition {
                                field: "operation_type".to_string(),
                                operator: ConditionOperator::Equals,
                                value: serde_json::json!(operation_type),
                            },
                        ],
                        recommendations: vec![
                            "Check permissions before attempting".to_string(),
                            "Validate input parameters".to_string(),
                        ],
                    });
                }
            }
            _ => {}
        }
        
        Ok(patterns)
    }
    
    /// Calculate initial confidence for new knowledge
    fn calculate_initial_confidence(&self, event: &LearningEvent) -> f64 {
        match event {
            LearningEvent::StageCompleted { .. } => 0.8,
            LearningEvent::UserFeedback { feedback_type, .. } => {
                match feedback_type {
                    FeedbackType::Positive => 0.9,
                    FeedbackType::Negative => 0.6,
                    FeedbackType::Correction(_) => 0.7,
                    FeedbackType::Suggestion(_) => 0.75,
                }
            }
            LearningEvent::OperationExecuted { success, .. } => {
                if *success { 0.85 } else { 0.65 }
            }
            LearningEvent::PatternDetected { confidence, .. } => *confidence,
            LearningEvent::ModelPerformance { success_rate, .. } => *success_rate,
        }
    }
    
    /// Store knowledge in vector store
    async fn store_knowledge(&self, knowledge: &LearnedKnowledge) -> Result<()> {
        let metadata = serde_json::json!({
            "type": "learned_knowledge",
            "learned_at": knowledge.learned_at.to_rfc3339(),
            "confidence": knowledge.confidence,
            "pattern_count": knowledge.patterns.len(),
        });
        
        self.vector_store
            .add_document(
                &knowledge.id,
                &serde_json::to_string(&knowledge.event)?,
                &knowledge.embedding,
                &metadata,
            )
            .await
    }
    
    /// Find similar past experiences
    async fn find_similar_experiences(
        &self,
        question: &str,
        limit: usize,
    ) -> Result<Vec<PastExperience>> {
        // Create embedding for question
        let question_embedding = self.create_event_embedding(&LearningEvent::StageCompleted {
            stage: "query".to_string(),
            question: question.to_string(),
            answer: String::new(),
            model: String::new(),
            duration_ms: 0,
            tokens_used: 0,
        }).await?;
        
        // Search vector store
        let results = self.vector_store
            .search(&question_embedding, limit)
            .await?;
        
        // Convert to past experiences
        let mut experiences = Vec::new();
        for (id, content, embedding, metadata) in results {
            if let Ok(event) = serde_json::from_str::<LearningEvent>(&content) {
                if let LearningEvent::StageCompleted { question: past_q, answer, .. } = event {
                    // Calculate similarity score (cosine similarity)
                    let similarity = self.cosine_similarity(&question_embedding, &embedding);
                    
                    experiences.push(PastExperience {
                        question_similarity: similarity,
                        outcome: if answer.is_empty() { "Failed".to_string() } else { "Success".to_string() },
                        key_insights: vec![
                            format!("Similar question asked before with {:.2}% similarity", similarity * 100.0),
                        ],
                        timestamp: metadata.get("learned_at")
                            .and_then(|v| v.as_str())
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(Utc::now),
                    });
                }
            }
        }
        
        Ok(experiences)
    }
    
    /// Find applicable patterns for current context
    async fn find_applicable_patterns(
        &self,
        question: &str,
        stage: Stage,
    ) -> Result<Vec<LearnedPattern>> {
        let recent = self.recent_learnings.read().await;
        
        let mut applicable = Vec::new();
        for knowledge in recent.iter() {
            for pattern in &knowledge.patterns {
                if self.pattern_applies(pattern, question, stage) {
                    applicable.push(pattern.clone());
                }
            }
        }
        
        // Sort by confidence
        applicable.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        Ok(applicable)
    }
    
    /// Check if a pattern applies to current context
    fn pattern_applies(&self, pattern: &LearnedPattern, question: &str, stage: Stage) -> bool {
        for condition in &pattern.conditions {
            match (condition.field.as_str(), &condition.operator) {
                ("stage", ConditionOperator::Equals) => {
                    if let Ok(stage_str) = serde_json::from_value::<String>(condition.value.clone()) {
                        if stage_str != format!("{:?}", stage) {
                            return false;
                        }
                    }
                }
                ("question_length", ConditionOperator::GreaterThan) => {
                    if let Ok(min_len) = serde_json::from_value::<usize>(condition.value.clone()) {
                        if question.len() <= min_len {
                            return false;
                        }
                    }
                }
                _ => {}
            }
        }
        true
    }
    
    /// Get model recommendations based on historical performance
    async fn get_model_recommendations(&self, stage: Stage) -> Result<HashMap<String, ModelRecommendation>> {
        let mut recommendations = HashMap::new();
        
        // TODO: Analyze historical model performance for this stage
        // For now, return empty recommendations
        
        Ok(recommendations)
    }
    
    /// Identify warnings based on patterns and experiences
    async fn identify_warnings(
        &self,
        experiences: &[PastExperience],
        patterns: &[LearnedPattern],
    ) -> Result<Vec<LearningWarning>> {
        let mut warnings = Vec::new();
        
        // Check for error patterns
        for pattern in patterns {
            if matches!(pattern.category, PatternCategory::ErrorPattern) {
                warnings.push(LearningWarning {
                    severity: LearningWarningSeverity::Medium,
                    message: pattern.description.clone(),
                    pattern_source: "Learned pattern".to_string(),
                    avoidance_strategy: pattern.recommendations.first().cloned(),
                });
            }
        }
        
        // Check for repeated failures
        let failure_count = experiences.iter()
            .filter(|e| e.outcome == "Failed")
            .count();
        
        if failure_count > experiences.len() / 2 {
            warnings.push(LearningWarning {
                severity: LearningWarningSeverity::High,
                message: "Similar questions have frequently failed".to_string(),
                pattern_source: "Historical analysis".to_string(),
                avoidance_strategy: Some("Consider rephrasing or breaking down the question".to_string()),
            });
        }
        
        Ok(warnings)
    }
    
    /// Extract success strategies from past experiences
    async fn extract_success_strategies(&self, experiences: &[PastExperience]) -> Result<Vec<String>> {
        let mut strategies = Vec::new();
        
        // Find successful experiences
        let successful = experiences.iter()
            .filter(|e| e.outcome == "Success")
            .collect::<Vec<_>>();
        
        if !successful.is_empty() {
            strategies.push(format!(
                "Similar questions succeeded {} out of {} times",
                successful.len(),
                experiences.len()
            ));
            
            // Extract common insights
            let mut insight_counts = HashMap::new();
            for exp in successful {
                for insight in &exp.key_insights {
                    *insight_counts.entry(insight.clone()).or_insert(0) += 1;
                }
            }
            
            // Add most common insights as strategies
            let mut insights: Vec<_> = insight_counts.into_iter().collect();
            insights.sort_by_key(|(_, count)| -*count);
            
            for (insight, _) in insights.iter().take(3) {
                strategies.push(insight.clone());
            }
        }
        
        Ok(strategies)
    }
    
    /// Find knowledge related to a conversation
    async fn find_knowledge_by_conversation(&self, conversation_id: &str) -> Result<Vec<LearnedKnowledge>> {
        // TODO: Implement conversation tracking
        // For now, return recent learnings
        let recent = self.recent_learnings.read().await;
        Ok(recent.clone())
    }
    
    /// Get learning statistics
    pub async fn get_stats(&self) -> LearningStats {
        self.stats.read().await.clone()
    }
    
    /// Calculate cosine similarity between two vectors
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_continuous_learning() {
        // Test learning from events
    }
    
    #[tokio::test]
    async fn test_pattern_extraction() {
        // Test pattern extraction from different event types
    }
    
    #[tokio::test]
    async fn test_learned_context() {
        // Test context generation for consensus stages
    }
}