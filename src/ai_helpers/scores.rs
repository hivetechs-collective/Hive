//! Score types for AI helpers
//!
//! Defines the score structures returned by each AI helper component.

use serde::{Deserialize, Serialize};

/// Score from Knowledge Indexer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeIndexerScore {
    /// Similarity score with historical operations (0.0-1.0)
    pub similarity_score: f32,
    
    /// Prediction confidence based on historical data (0-100)
    pub prediction_confidence: f32,
    
    /// Whether relevant context was found
    pub relevant_context_found: bool,
}

/// Score from Context Retriever
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRetrieverScore {
    /// Relevance score of retrieved context (0.0-1.0)
    pub relevance_score: f32,
    
    /// Strength of precedent found (0.0-1.0)
    pub precedent_strength: f32,
    
    /// Historical success rate for similar contexts
    pub success_rate: Option<f32>,
}

/// Score from Pattern Recognizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRecognizerScore {
    /// Safety score based on pattern analysis (0-100)
    pub safety_score: f32,
    
    /// Patterns matched
    pub pattern_matches: Vec<String>,
    
    /// Number of anti-patterns detected
    pub anti_patterns_detected: usize,
}

/// Score from Quality Analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAnalyzerScore {
    /// Risk score (0-100, higher = more risky)
    pub risk_score: f32,
    
    /// Quality impact (-1.0 to 1.0, negative = degradation)
    pub quality_impact: f32,
    
    /// Conflict probability (0.0-1.0)
    pub conflict_probability: f32,
    
    /// Rollback complexity score (0-100)
    pub rollback_complexity: f32,
}

/// Score from Knowledge Synthesizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSynthesizerScore {
    /// Plan quality score (0.0-1.0)
    pub plan_quality: f32,
    
    /// Completeness of the plan (0.0-1.0)
    pub completeness: f32,
    
    /// Execution confidence (0.0-1.0)
    pub execution_confidence: f32,
}