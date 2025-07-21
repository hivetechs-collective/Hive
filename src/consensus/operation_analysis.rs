//! Updated Operation Analysis structures for consensus engine
//!
//! This module provides the updated structures that match what the
//! smart decision engine and other consensus components expect.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use crate::consensus::stages::file_aware_curator::FileOperation;

/// Complete analysis result from all AI helpers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperationAnalysis {
    /// The operations being analyzed
    pub operations: Vec<FileOperation>,
    
    /// Context of the operation
    pub context: OperationContext,
    
    /// Unified score combining all AI helper outputs
    pub unified_score: UnifiedScore,
    
    /// Recommendations from various AI helpers
    pub recommendations: Vec<ActionRecommendation>,
    
    /// Operation groups for better organization
    pub groups: OperationGroups,
    
    /// Component scores from individual AI helpers
    pub component_scores: ComponentScores,
    
    /// Scoring factors used in decision making
    pub scoring_factors: ScoringFactors,
    
    /// Statistics about the analysis
    pub statistics: Option<AnalysisStatistics>,
}

/// Unified score combining all AI analyses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedScore {
    /// Overall confidence score (0-100)
    pub confidence: f32,
    
    /// Overall risk score (0-100)
    pub risk: f32,
}

/// Action recommendation from AI helpers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecommendation {
    /// Priority of this recommendation
    pub priority: ActionPriority,
    
    /// Source AI helper
    pub source: String,
    
    /// Description of the recommendation
    pub description: String,
    
    /// Detailed action to take
    pub action: RecommendedAction,
}

/// Priority levels for recommendations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Recommended actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    Block,
    RequestConfirmation,
    AddValidation,
    CreateBackup,
    ModifyOperation,
    AddTest,
    ReviewManually,
}

/// Operation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationContext {
    /// Repository path
    pub repository_path: PathBuf,
    
    /// User's original question
    pub user_question: String,
    
    /// Consensus response that led to these operations
    pub consensus_response: String,
    
    /// Timestamp
    pub timestamp: SystemTime,
    
    /// Session ID
    pub session_id: String,
    
    /// Git commit hash if available
    pub git_commit: Option<String>,
}

/// Grouping of operations by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationGroups {
    pub create_operations: Vec<FileOperation>,
    pub update_operations: Vec<FileOperation>,
    pub delete_operations: Vec<FileOperation>,
    pub move_operations: Vec<FileOperation>,
}

/// Scores from individual AI components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    pub knowledge_indexer: Option<crate::ai_helpers::KnowledgeIndexerScore>,
    pub context_retriever: Option<crate::ai_helpers::ContextRetrieverScore>,
    pub pattern_recognizer: Option<crate::ai_helpers::PatternRecognizerScore>,
    pub quality_analyzer: Option<crate::ai_helpers::QualityAnalyzerScore>,
    pub knowledge_synthesizer: Option<crate::ai_helpers::KnowledgeSynthesizerScore>,
}

/// Factors used in scoring decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringFactors {
    pub historical_success: Option<f32>,
    pub pattern_safety: Option<f32>,
    pub conflict_probability: Option<f32>,
    pub rollback_complexity: Option<f32>,
    pub user_trust: f32,
    pub similar_operations_count: Option<usize>,
    pub dangerous_pattern_count: Option<usize>,
    pub anti_pattern_count: Option<usize>,
    pub rollback_possible: Option<bool>,
}

/// Statistics about the analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStatistics {
    pub analysis_duration_ms: u64,
    pub operations_analyzed: usize,
    pub ai_helpers_used: usize,
    pub cache_hits: usize,
}

/// Operation outcome for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationOutcome {
    pub success: bool,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub rollback_required: bool,
    pub user_satisfaction: Option<f32>,
}

/// Auto-accept mode for smart decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoAcceptMode {
    Conservative,
    Balanced,
    Aggressive,
    Plan,
    Manual,
}

impl Default for AutoAcceptMode {
    fn default() -> Self {
        Self::Balanced
    }
}

impl Default for OperationContext {
    fn default() -> Self {
        Self {
            repository_path: PathBuf::from("."),
            user_question: String::new(),
            consensus_response: String::new(),
            timestamp: SystemTime::now(),
            session_id: String::new(),
            git_commit: None,
        }
    }
}

impl Default for UnifiedScore {
    fn default() -> Self {
        Self {
            confidence: 0.0,
            risk: 0.0,
        }
    }
}

impl Default for OperationGroups {
    fn default() -> Self {
        Self {
            create_operations: Vec::new(),
            update_operations: Vec::new(),
            delete_operations: Vec::new(),
            move_operations: Vec::new(),
        }
    }
}

impl Default for ComponentScores {
    fn default() -> Self {
        Self {
            knowledge_indexer: None,
            context_retriever: None,
            pattern_recognizer: None,
            quality_analyzer: None,
            knowledge_synthesizer: None,
        }
    }
}

impl Default for ScoringFactors {
    fn default() -> Self {
        Self {
            historical_success: None,
            pattern_safety: None,
            conflict_probability: None,
            rollback_complexity: None,
            user_trust: 0.8,
            similar_operations_count: None,
            dangerous_pattern_count: None,
            anti_pattern_count: None,
            rollback_possible: None,
        }
    }
}