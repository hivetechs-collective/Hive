//! Operation Intelligence Infrastructure
//!
//! Coordinates all AI helpers to provide intelligent analysis of file operations.
//! This system analyzes proposed operations for safety, confidence, patterns,
//! and generates smart recommendations for auto-accept decisions.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use crate::ai_helpers::{
    KnowledgeIndexer, ContextRetriever, PatternRecognizer, 
    QualityAnalyzer, KnowledgeSynthesizer
};
use crate::consensus::stages::file_aware_curator::FileOperation;

/// Complete analysis of a file operation by all AI helpers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationAnalysis {
    /// Overall confidence score (0-100)
    pub confidence: f32,
    
    /// Overall risk score (0-100, higher = more risky)
    pub risk: f32,
    
    /// Historical success rate for similar operations
    pub historical_success_rate: f32,
    
    /// Pattern analysis results
    pub pattern_analysis: PatternAnalysis,
    
    /// Quality impact assessment
    pub quality_impact: QualityImpact,
    
    /// Operation preview and rollback plan
    pub operation_plan: OperationPlan,
    
    /// User-friendly explanation
    pub explanation: String,
    
    /// Recommended action
    pub recommendation: ActionRecommendation,
    
    /// Analysis timestamp
    pub analyzed_at: SystemTime,
}

/// Pattern analysis from Pattern Recognizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    /// Detected dangerous patterns
    pub dangerous_patterns: Vec<DangerousPattern>,
    
    /// Operation cluster information
    pub cluster_info: Option<OperationCluster>,
    
    /// Suggested operation sequence
    pub suggested_sequence: Vec<FileOperation>,
    
    /// Anti-patterns detected
    pub anti_patterns: Vec<AntiPattern>,
    
    /// Pattern confidence score
    pub pattern_confidence: f32,
}

/// Dangerous pattern detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerousPattern {
    /// Pattern type
    pub pattern_type: DangerousPatternType,
    
    /// Description of the danger
    pub description: String,
    
    /// Severity level
    pub severity: DangerousSeverity,
    
    /// Suggested mitigation
    pub mitigation: Option<String>,
}

/// Types of dangerous patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DangerousPatternType {
    MassDeletion,
    OverwriteWithoutBackup,
    CircularDependency,
    BreakingChange,
    DataLoss,
    SecurityVulnerability,
}

/// Severity of dangerous patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DangerousSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Operation cluster for grouping related operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationCluster {
    /// Cluster name/description
    pub name: String,
    
    /// Operations in this cluster
    pub operations: Vec<FileOperation>,
    
    /// Cluster type
    pub cluster_type: ClusterType,
    
    /// Execution order dependency
    pub execution_order: Vec<usize>,
}

/// Types of operation clusters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterType {
    Refactoring,
    FeatureAddition,
    BugFix,
    Cleanup,
    Migration,
    Testing,
}

/// Anti-pattern that creates technical debt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    /// Anti-pattern type
    pub pattern_type: String,
    
    /// Why this is problematic
    pub description: String,
    
    /// Suggested alternative approach
    pub alternative: Option<String>,
}

/// Quality impact assessment from Quality Analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityImpact {
    /// Overall quality impact score (-100 to +100)
    pub overall_impact: f32,
    
    /// Impact on code maintainability
    pub maintainability_impact: f32,
    
    /// Impact on code reliability
    pub reliability_impact: f32,
    
    /// Impact on performance
    pub performance_impact: f32,
    
    /// Detected conflicts with other operations
    pub conflicts: Vec<OperationConflict>,
    
    /// Rollback complexity assessment
    pub rollback_complexity: RollbackComplexity,
}

/// Conflict between operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationConflict {
    /// Operations involved in conflict
    pub conflicting_operations: Vec<usize>,
    
    /// Type of conflict
    pub conflict_type: ConflictType,
    
    /// Description of the conflict
    pub description: String,
    
    /// Suggested resolution
    pub resolution: Option<String>,
}

/// Types of operation conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    FileOverwrite,
    DependencyViolation,
    RaceCondition,
    LogicalInconsistency,
}

/// Rollback complexity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackComplexity {
    /// Complexity score (0-100, higher = more complex)
    pub complexity_score: f32,
    
    /// Whether rollback is possible
    pub rollback_possible: bool,
    
    /// Estimated rollback time
    pub estimated_rollback_time: Duration,
    
    /// Required rollback steps
    pub rollback_steps: Vec<String>,
}

/// Operation plan from Knowledge Synthesizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPlan {
    /// Execution plan steps
    pub execution_steps: Vec<ExecutionStep>,
    
    /// Before/after preview
    pub preview: OperationPreview,
    
    /// Backup strategy
    pub backup_strategy: BackupStrategy,
    
    /// Rollback plan
    pub rollback_plan: RollbackPlan,
    
    /// Estimated execution time
    pub estimated_execution_time: Duration,
}

/// Single execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number
    pub step_number: usize,
    
    /// Description of the step
    pub description: String,
    
    /// Operations in this step
    pub operations: Vec<FileOperation>,
    
    /// Prerequisites for this step
    pub prerequisites: Vec<String>,
    
    /// Validation checks after this step
    pub validation_checks: Vec<String>,
}

/// Preview of operation effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPreview {
    /// Files that will be created
    pub files_created: Vec<PathBuf>,
    
    /// Files that will be modified
    pub files_modified: Vec<PathBuf>,
    
    /// Files that will be deleted
    pub files_deleted: Vec<PathBuf>,
    
    /// Before/after content samples
    pub content_previews: Vec<ContentPreview>,
    
    /// Summary of changes
    pub change_summary: String,
}

/// Preview of file content changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPreview {
    /// File path
    pub file_path: PathBuf,
    
    /// Content before change
    pub before: Option<String>,
    
    /// Content after change
    pub after: String,
    
    /// Change type
    pub change_type: ChangeType,
}

/// Types of content changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Create,
    Update,
    Append,
    Delete,
    Rename,
}

/// Backup strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStrategy {
    /// Whether backup is required
    pub backup_required: bool,
    
    /// Backup location
    pub backup_location: Option<PathBuf>,
    
    /// Files to backup
    pub files_to_backup: Vec<PathBuf>,
    
    /// Backup method
    pub backup_method: BackupMethod,
}

/// Backup methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupMethod {
    Copy,
    GitCommit,
    Archive,
    Snapshot,
}

/// Rollback plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    /// Rollback steps in reverse order
    pub rollback_steps: Vec<RollbackStep>,
    
    /// Rollback complexity score
    pub complexity: f32,
    
    /// Whether automatic rollback is possible
    pub automatic_rollback_possible: bool,
    
    /// Manual intervention required
    pub manual_steps_required: Vec<String>,
}

/// Single rollback step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    /// Step description
    pub description: String,
    
    /// Command or action to perform
    pub action: RollbackAction,
    
    /// Validation after this step
    pub validation: Option<String>,
}

/// Rollback actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackAction {
    RestoreFile { from: PathBuf, to: PathBuf },
    DeleteFile { path: PathBuf },
    RunCommand { command: String },
    GitRevert { commit: String },
}

/// Recommended action for the operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionRecommendation {
    /// Auto-execute with high confidence
    AutoExecute {
        reason: String,
        confidence: f32,
    },
    
    /// Request user confirmation
    RequestConfirmation {
        reason: String,
        risks: Vec<String>,
        benefits: Vec<String>,
    },
    
    /// Block operation due to safety concerns
    Block {
        reason: String,
        dangers: Vec<String>,
        alternatives: Vec<String>,
    },
    
    /// Suggest modifications to the operation
    SuggestModifications {
        reason: String,
        suggested_operations: Vec<FileOperation>,
        modifications: Vec<String>,
    },
}

/// Historical operation data for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationHistory {
    /// Operation that was performed
    pub operation: FileOperation,
    
    /// Context at time of operation
    pub context: OperationContext,
    
    /// Predicted confidence at time
    pub predicted_confidence: f32,
    
    /// Predicted risk at time
    pub predicted_risk: f32,
    
    /// Actual outcome
    pub outcome: OperationOutcome,
    
    /// User satisfaction rating
    pub user_satisfaction: Option<f32>,
    
    /// Timestamp
    pub timestamp: SystemTime,
}

/// Context when operation was performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationContext {
    /// Repository state
    pub repository_path: PathBuf,
    
    /// Git commit hash
    pub git_commit: Option<String>,
    
    /// User question that led to operation
    pub source_question: String,
    
    /// Related files in context
    pub related_files: Vec<PathBuf>,
    
    /// Project metadata
    pub project_metadata: HashMap<String, String>,
}

/// Outcome of an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationOutcome {
    /// Whether operation succeeded
    pub success: bool,
    
    /// Error message if failed
    pub error_message: Option<String>,
    
    /// Time taken to complete
    pub execution_time: Duration,
    
    /// Whether rollback was required
    pub rollback_required: bool,
    
    /// User feedback
    pub user_feedback: Option<String>,
    
    /// Quality metrics after operation
    pub post_operation_quality: Option<QualityMetrics>,
}

/// Quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Code quality score
    pub code_quality: f32,
    
    /// Test coverage impact
    pub test_coverage_change: f32,
    
    /// Performance impact
    pub performance_impact: f32,
    
    /// Maintainability score
    pub maintainability: f32,
}

/// Main coordinator for operation intelligence
pub struct OperationIntelligenceCoordinator {
    /// AI Helpers
    knowledge_indexer: Arc<KnowledgeIndexer>,
    context_retriever: Arc<ContextRetriever>,
    pattern_recognizer: Arc<PatternRecognizer>,
    quality_analyzer: Arc<QualityAnalyzer>,
    knowledge_synthesizer: Arc<KnowledgeSynthesizer>,
    
    /// Operation history storage
    operation_history: Arc<RwLock<Vec<OperationHistory>>>,
    
    /// Performance metrics
    analysis_cache: Arc<RwLock<HashMap<String, OperationAnalysis>>>,
}

impl OperationIntelligenceCoordinator {
    /// Create new operation intelligence coordinator
    pub fn new(
        knowledge_indexer: Arc<KnowledgeIndexer>,
        context_retriever: Arc<ContextRetriever>,
        pattern_recognizer: Arc<PatternRecognizer>,
        quality_analyzer: Arc<QualityAnalyzer>,
        knowledge_synthesizer: Arc<KnowledgeSynthesizer>,
    ) -> Self {
        Self {
            knowledge_indexer,
            context_retriever,
            pattern_recognizer,
            quality_analyzer,
            knowledge_synthesizer,
            operation_history: Arc::new(RwLock::new(Vec::new())),
            analysis_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Analyze a file operation using all AI helpers
    pub async fn analyze_operation(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<OperationAnalysis> {
        info!("🔍 Starting comprehensive operation analysis");
        
        // Check cache first
        let cache_key = self.generate_cache_key(operation, context);
        {
            let cache = self.analysis_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                debug!("📋 Using cached analysis for operation");
                return Ok(cached.clone());
            }
        }
        
        // Run analysis with all AI helpers in parallel
        let start_time = std::time::Instant::now();
        
        // 1. Knowledge Indexer: Find similar operations
        let historical_analysis = self.analyze_historical_patterns(operation, context).await?;
        
        // 2. Context Retriever: Get relevant context and success rates
        let context_analysis = self.analyze_operation_context(operation, context).await?;
        
        // 3. Pattern Recognizer: Detect patterns and risks
        let pattern_analysis = self.analyze_operation_patterns(operation, context).await?;
        
        // 4. Quality Analyzer: Assess risks and conflicts
        let quality_impact = self.analyze_quality_impact(operation, context).await?;
        
        // 5. Knowledge Synthesizer: Generate execution plan
        let operation_plan = self.generate_operation_plan(operation, context).await?;
        
        // Combine all analyses into unified scores
        let confidence = self.calculate_confidence_score(
            &historical_analysis,
            &context_analysis,
            &pattern_analysis,
            &quality_impact,
        ).await?;
        
        let risk = self.calculate_risk_score(
            &pattern_analysis,
            &quality_impact,
            &operation_plan,
        ).await?;
        
        // Generate recommendation
        let recommendation = self.generate_recommendation(
            confidence,
            risk,
            &pattern_analysis,
            &quality_impact,
        ).await?;
        
        // Generate user-friendly explanation
        let explanation = self.generate_explanation(
            operation,
            confidence,
            risk,
            &pattern_analysis,
            &quality_impact,
            &recommendation,
        ).await?;
        
        let analysis = OperationAnalysis {
            confidence,
            risk,
            historical_success_rate: historical_analysis.success_rate,
            pattern_analysis,
            quality_impact,
            operation_plan,
            explanation,
            recommendation,
            analyzed_at: SystemTime::now(),
        };
        
        // Cache the result
        {
            let mut cache = self.analysis_cache.write().await;
            cache.insert(cache_key, analysis.clone());
        }
        
        let analysis_time = start_time.elapsed();
        info!("✅ Operation analysis completed in {:?}", analysis_time);
        
        Ok(analysis)
    }
    
    /// Analyze multiple operations as a batch
    pub async fn analyze_operations_batch(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<Vec<OperationAnalysis>> {
        info!("🔍 Analyzing batch of {} operations", operations.len());
        
        let mut analyses = Vec::new();
        
        // Analyze each operation
        for operation in operations {
            let analysis = self.analyze_operation(operation, context).await?;
            analyses.push(analysis);
        }
        
        // Detect cross-operation conflicts and dependencies
        self.analyze_cross_operation_effects(&mut analyses, operations).await?;
        
        Ok(analyses)
    }
    
    /// Record operation outcome for learning
    pub async fn record_operation_outcome(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
        predicted_confidence: f32,
        predicted_risk: f32,
        outcome: &OperationOutcome,
        user_satisfaction: Option<f32>,
    ) -> Result<()> {
        let history_entry = OperationHistory {
            operation: operation.clone(),
            context: context.clone(),
            predicted_confidence,
            predicted_risk,
            outcome: outcome.clone(),
            user_satisfaction,
            timestamp: SystemTime::now(),
        };
        
        // Store in history
        {
            let mut history = self.operation_history.write().await;
            history.push(history_entry.clone());
        }
        
        // Update AI helpers with learning data
        self.update_helpers_with_outcome(&history_entry).await?;
        
        info!("📊 Recorded operation outcome for learning");
        Ok(())
    }
    
    /// Get operation success statistics
    pub async fn get_success_statistics(&self) -> Result<OperationStatistics> {
        let history = self.operation_history.read().await;
        
        let total_operations = history.len();
        let successful_operations = history.iter()
            .filter(|h| h.outcome.success)
            .count();
        
        let average_confidence = if !history.is_empty() {
            history.iter()
                .map(|h| h.predicted_confidence)
                .sum::<f32>() / history.len() as f32
        } else {
            0.0
        };
        
        let average_risk = if !history.is_empty() {
            history.iter()
                .map(|h| h.predicted_risk)
                .sum::<f32>() / history.len() as f32
        } else {
            0.0
        };
        
        Ok(OperationStatistics {
            total_operations,
            successful_operations,
            success_rate: if total_operations > 0 {
                successful_operations as f32 / total_operations as f32
            } else {
                0.0
            },
            average_confidence,
            average_risk,
            rollbacks_required: history.iter()
                .filter(|h| h.outcome.rollback_required)
                .count(),
        })
    }
    
    // Private helper methods
    
    async fn analyze_historical_patterns(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<HistoricalAnalysis> {
        debug!("📚 Analyzing historical patterns with Knowledge Indexer");
        
        // Use Knowledge Indexer to find similar operations
        // This will be implemented when we enhance the Knowledge Indexer
        
        // For now, return placeholder data
        Ok(HistoricalAnalysis {
            similar_operations_count: 0,
            success_rate: 0.8, // Default success rate
            common_failure_modes: Vec::new(),
        })
    }
    
    async fn analyze_operation_context(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<ContextAnalysis> {
        debug!("🔍 Analyzing operation context with Context Retriever");
        
        // Use Context Retriever to analyze context
        // This will be implemented when we enhance the Context Retriever
        
        Ok(ContextAnalysis {
            context_similarity: 0.7,
            relevant_precedents: Vec::new(),
            context_warnings: Vec::new(),
        })
    }
    
    async fn analyze_operation_patterns(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<PatternAnalysis> {
        debug!("🧠 Analyzing operation patterns with Pattern Recognizer");
        
        // Use Pattern Recognizer to detect patterns
        // This will be implemented when we enhance the Pattern Recognizer
        
        Ok(PatternAnalysis {
            dangerous_patterns: Vec::new(),
            cluster_info: None,
            suggested_sequence: vec![operation.clone()],
            anti_patterns: Vec::new(),
            pattern_confidence: 0.8,
        })
    }
    
    async fn analyze_quality_impact(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<QualityImpact> {
        debug!("📊 Analyzing quality impact with Quality Analyzer");
        
        // Use Quality Analyzer to assess impact
        // This will be implemented when we enhance the Quality Analyzer
        
        Ok(QualityImpact {
            overall_impact: 0.0,
            maintainability_impact: 0.0,
            reliability_impact: 0.0,
            performance_impact: 0.0,
            conflicts: Vec::new(),
            rollback_complexity: RollbackComplexity {
                complexity_score: 20.0,
                rollback_possible: true,
                estimated_rollback_time: Duration::from_secs(30),
                rollback_steps: vec!["Restore from backup".to_string()],
            },
        })
    }
    
    async fn generate_operation_plan(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<OperationPlan> {
        debug!("🔄 Generating operation plan with Knowledge Synthesizer");
        
        // Use Knowledge Synthesizer to generate plan
        // This will be implemented when we enhance the Knowledge Synthesizer
        
        Ok(OperationPlan {
            execution_steps: vec![
                ExecutionStep {
                    step_number: 1,
                    description: "Execute operation".to_string(),
                    operations: vec![operation.clone()],
                    prerequisites: Vec::new(),
                    validation_checks: Vec::new(),
                }
            ],
            preview: OperationPreview {
                files_created: Vec::new(),
                files_modified: Vec::new(),
                files_deleted: Vec::new(),
                content_previews: Vec::new(),
                change_summary: "Operation will be executed".to_string(),
            },
            backup_strategy: BackupStrategy {
                backup_required: true,
                backup_location: None,
                files_to_backup: Vec::new(),
                backup_method: BackupMethod::Copy,
            },
            rollback_plan: RollbackPlan {
                rollback_steps: Vec::new(),
                complexity: 20.0,
                automatic_rollback_possible: true,
                manual_steps_required: Vec::new(),
            },
            estimated_execution_time: Duration::from_secs(5),
        })
    }
    
    async fn calculate_confidence_score(
        &self,
        historical: &HistoricalAnalysis,
        context: &ContextAnalysis,
        patterns: &PatternAnalysis,
        quality: &QualityImpact,
    ) -> Result<f32> {
        // Weighted combination of all factors
        let historical_weight = 0.3;
        let context_weight = 0.2;
        let patterns_weight = 0.3;
        let quality_weight = 0.2;
        
        let confidence = 
            historical.success_rate * historical_weight +
            context.context_similarity * context_weight +
            patterns.pattern_confidence * patterns_weight +
            (100.0 - quality.rollback_complexity.complexity_score) / 100.0 * quality_weight;
        
        Ok((confidence * 100.0).clamp(0.0, 100.0))
    }
    
    async fn calculate_risk_score(
        &self,
        patterns: &PatternAnalysis,
        quality: &QualityImpact,
        plan: &OperationPlan,
    ) -> Result<f32> {
        let mut risk_score = 0.0;
        
        // Add risk from dangerous patterns
        for pattern in &patterns.dangerous_patterns {
            risk_score += match pattern.severity {
                DangerousSeverity::Low => 10.0,
                DangerousSeverity::Medium => 25.0,
                DangerousSeverity::High => 50.0,
                DangerousSeverity::Critical => 80.0,
            };
        }
        
        // Add risk from conflicts
        risk_score += quality.conflicts.len() as f32 * 15.0;
        
        // Add risk from rollback complexity
        risk_score += quality.rollback_complexity.complexity_score * 0.3;
        
        Ok(risk_score.clamp(0.0, 100.0))
    }
    
    async fn generate_recommendation(
        &self,
        confidence: f32,
        risk: f32,
        patterns: &PatternAnalysis,
        quality: &QualityImpact,
    ) -> Result<ActionRecommendation> {
        // Decision logic based on confidence and risk
        if risk > 70.0 {
            return Ok(ActionRecommendation::Block {
                reason: "High risk operation detected".to_string(),
                dangers: patterns.dangerous_patterns.iter()
                    .map(|p| p.description.clone())
                    .collect(),
                alternatives: vec!["Consider manual execution with careful review".to_string()],
            });
        }
        
        if confidence > 85.0 && risk < 20.0 {
            return Ok(ActionRecommendation::AutoExecute {
                reason: "High confidence, low risk operation".to_string(),
                confidence,
            });
        }
        
        Ok(ActionRecommendation::RequestConfirmation {
            reason: "Moderate confidence/risk - user confirmation recommended".to_string(),
            risks: patterns.dangerous_patterns.iter()
                .map(|p| p.description.clone())
                .collect(),
            benefits: vec!["Operation appears to be well-formed".to_string()],
        })
    }
    
    async fn generate_explanation(
        &self,
        operation: &FileOperation,
        confidence: f32,
        risk: f32,
        patterns: &PatternAnalysis,
        quality: &QualityImpact,
        recommendation: &ActionRecommendation,
    ) -> Result<String> {
        let operation_description = match operation {
            FileOperation::Create { path, .. } => format!("create file {}", path.display()),
            FileOperation::Update { path, .. } => format!("update file {}", path.display()),
            FileOperation::Append { path, .. } => format!("append to file {}", path.display()),
            FileOperation::Delete { path } => format!("delete file {}", path.display()),
            FileOperation::Rename { from, to } => format!("rename {} to {}", from.display(), to.display()),
        };
        
        let explanation = format!(
            "Operation to {} has {:.0}% confidence and {:.0}% risk. {}",
            operation_description,
            confidence,
            risk,
            match recommendation {
                ActionRecommendation::AutoExecute { reason, .. } => reason.clone(),
                ActionRecommendation::RequestConfirmation { reason, .. } => reason.clone(),
                ActionRecommendation::Block { reason, .. } => reason.clone(),
                ActionRecommendation::SuggestModifications { reason, .. } => reason.clone(),
            }
        );
        
        Ok(explanation)
    }
    
    async fn analyze_cross_operation_effects(
        &self,
        analyses: &mut [OperationAnalysis],
        operations: &[FileOperation],
    ) -> Result<()> {
        // Analyze conflicts and dependencies between operations
        // This is a placeholder - will be enhanced with actual logic
        Ok(())
    }
    
    async fn update_helpers_with_outcome(
        &self,
        history_entry: &OperationHistory,
    ) -> Result<()> {
        // Update AI helpers with learning data
        // This will be implemented when we enhance each helper
        Ok(())
    }
    
    fn generate_cache_key(&self, operation: &FileOperation, context: &OperationContext) -> String {
        // Generate a cache key based on operation and context
        format!("{:?}_{}", operation, context.source_question.len())
    }
}

/// Statistics about operation success
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStatistics {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub success_rate: f32,
    pub average_confidence: f32,
    pub average_risk: f32,
    pub rollbacks_required: usize,
}

/// Historical analysis results
#[derive(Debug, Clone)]
struct HistoricalAnalysis {
    similar_operations_count: usize,
    success_rate: f32,
    common_failure_modes: Vec<String>,
}

/// Context analysis results
#[derive(Debug, Clone)]
struct ContextAnalysis {
    context_similarity: f32,
    relevant_precedents: Vec<String>,
    context_warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_operation_analysis_creation() {
        // Test that we can create basic operation analysis structures
        let operation = FileOperation::Create {
            path: PathBuf::from("test.rs"),
            content: "fn main() {}".to_string(),
        };
        
        let context = OperationContext {
            repository_path: PathBuf::from("/test"),
            git_commit: None,
            source_question: "Create a test file".to_string(),
            related_files: Vec::new(),
            project_metadata: HashMap::new(),
        };
        
        // Just verify we can create the structures
        assert_eq!(operation.clone(), operation);
        assert_eq!(context.repository_path, PathBuf::from("/test"));
    }
}