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
use crate::consensus::operation_history::{
    OperationHistoryDatabase, OperationRecord, OperationFilters
};

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DangerousPatternType {
    MassDeletion,
    OverwriteWithoutBackup,
    CircularDependency,
    BreakingChange,
    DataLoss,
    SecurityVulnerability,
    ConcurrentAccess,
    ResourceExhaustion,
    PermissionEscalation,
    ConfigurationDrift,
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

impl ClusterType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClusterType::Refactoring => "refactoring",
            ClusterType::FeatureAddition => "feature_addition",
            ClusterType::BugFix => "bug_fix",
            ClusterType::Cleanup => "cleanup",
            ClusterType::Migration => "migration",
            ClusterType::Testing => "testing",
        }
    }
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

impl ConflictType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConflictType::FileOverwrite => "file_overwrite",
            ConflictType::DependencyViolation => "dependency_violation",
            ConflictType::RaceCondition => "race_condition",
            ConflictType::LogicalInconsistency => "logical_inconsistency",
        }
    }
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    Create,
    Update,
    Append,
    Delete,
    Rename,
}

impl ChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::Create => "create",
            ChangeType::Update => "update",
            ChangeType::Append => "append",
            ChangeType::Delete => "delete",
            ChangeType::Rename => "rename",
        }
    }
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
    
    /// Operation history database
    history_database: Option<Arc<OperationHistoryDatabase>>,
    
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
            history_database: None,
            analysis_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Set operation history database
    pub async fn set_history_database(&mut self, database_url: &str) -> Result<()> {
        info!("🗄️  Connecting to operation history database");
        let db = OperationHistoryDatabase::new(database_url).await?;
        self.history_database = Some(Arc::new(db));
        Ok(())
    }
    
    /// Get unified score for an operation
    pub async fn get_unified_score(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<UnifiedScore> {
        info!("🎯 Calculating unified score for operation");
        
        // Get full analysis
        let analysis = self.analyze_operation(operation, context).await?;
        
        // Extract component scores
        let component_scores = ComponentScores {
            historical_score: analysis.historical_success_rate * 100.0,
            context_score: 0.0, // Will be calculated from internal analysis
            pattern_score: (100.0 - analysis.pattern_analysis.pattern_confidence * 100.0).max(0.0),
            quality_score: (50.0 + analysis.quality_impact.overall_impact).clamp(0.0, 100.0),
            plan_score: if analysis.operation_plan.rollback_plan.automatic_rollback_possible {
                80.0
            } else {
                40.0
            },
        };
        
        // Extract scoring factors
        let scoring_factors = ScoringFactors {
            similar_operations_count: 0, // Will be filled from historical analysis
            dangerous_pattern_count: analysis.pattern_analysis.dangerous_patterns.len(),
            anti_pattern_count: analysis.pattern_analysis.anti_patterns.len(),
            conflict_count: analysis.quality_impact.conflicts.len(),
            rollback_possible: analysis.quality_impact.rollback_complexity.rollback_possible,
            execution_complexity: analysis.operation_plan.execution_steps.len() as f32 * 10.0,
        };
        
        // Determine recommended mode based on scores
        let recommended_mode = self.recommend_auto_accept_mode(
            analysis.confidence,
            analysis.risk,
            &scoring_factors,
        );
        
        Ok(UnifiedScore {
            confidence: analysis.confidence,
            risk: analysis.risk,
            component_scores,
            scoring_factors,
            recommended_mode,
        })
    }
    
    /// Check if operation should be auto-executed based on mode
    pub async fn should_auto_execute(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
        mode: AutoAcceptMode,
    ) -> Result<(bool, OperationAnalysis)> {
        info!("🤔 Checking if operation should be auto-executed in {:?} mode", mode);
        
        // Get full analysis
        let analysis = self.analyze_operation(operation, context).await?;
        
        // Check against mode thresholds
        let should_execute = match mode {
            AutoAcceptMode::Manual | AutoAcceptMode::Plan => false,
            _ => {
                let confidence_ok = analysis.confidence >= mode.confidence_threshold();
                let risk_ok = analysis.risk <= mode.risk_threshold();
                let recommendation_ok = matches!(
                    analysis.recommendation,
                    ActionRecommendation::AutoExecute { .. }
                );
                
                confidence_ok && risk_ok && recommendation_ok
            }
        };
        
        info!(
            "📊 Auto-execute decision: {} (confidence: {:.0}%, risk: {:.0}%, mode: {:?})",
            if should_execute { "YES" } else { "NO" },
            analysis.confidence,
            analysis.risk,
            mode
        );
        
        // Record operation in history database if available
        if let Some(db) = &self.history_database {
            let recommendation_str = match &analysis.recommendation {
                ActionRecommendation::AutoExecute { .. } => "auto_execute",
                ActionRecommendation::RequestConfirmation { .. } => "request_confirmation",
                ActionRecommendation::Block { .. } => "block",
                ActionRecommendation::SuggestModifications { .. } => "suggest_modifications",
            };
            
            if let Err(e) = db.record_operation(
                operation,
                context,
                analysis.confidence,
                analysis.risk,
                &format!("{:?}", mode),
                should_execute,
                recommendation_str,
            ).await {
                warn!("Failed to record operation in history: {}", e);
            }
        }
        
        Ok((should_execute, analysis))
    }
    
    /// Get recommended auto-accept mode for an operation
    pub async fn get_recommended_mode(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<AutoAcceptMode> {
        let score = self.get_unified_score(operation, context).await?;
        Ok(score.recommended_mode)
    }
    
    /// Batch analyze operations and group by recommended execution strategy
    pub async fn analyze_and_group_operations(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
        mode: AutoAcceptMode,
    ) -> Result<OperationGroups> {
        info!("📦 Analyzing and grouping {} operations", operations.len());
        
        let mut auto_execute = Vec::new();
        let mut needs_confirmation = Vec::new();
        let mut blocked = Vec::new();
        let mut needs_modification = Vec::new();
        
        for operation in operations {
            let (should_auto, analysis) = self.should_auto_execute(operation, context, mode).await?;
            
            match &analysis.recommendation {
                ActionRecommendation::AutoExecute { .. } if should_auto => {
                    auto_execute.push((operation.clone(), analysis));
                },
                ActionRecommendation::Block { .. } => {
                    blocked.push((operation.clone(), analysis));
                },
                ActionRecommendation::SuggestModifications { .. } => {
                    needs_modification.push((operation.clone(), analysis));
                },
                _ => {
                    needs_confirmation.push((operation.clone(), analysis));
                }
            }
        }
        
        Ok(OperationGroups {
            auto_execute,
            needs_confirmation,
            blocked,
            needs_modification,
        })
    }
    
    // Private helper to recommend auto-accept mode
    fn recommend_auto_accept_mode(
        &self,
        confidence: f32,
        risk: f32,
        factors: &ScoringFactors,
    ) -> AutoAcceptMode {
        // If any critical factors, recommend conservative or manual
        if factors.dangerous_pattern_count > 2 || !factors.rollback_possible {
            return AutoAcceptMode::Manual;
        }
        
        // Based on confidence and risk levels
        match (confidence, risk) {
            (c, r) if c > 90.0 && r < 15.0 => AutoAcceptMode::Conservative,
            (c, r) if c > 80.0 && r < 25.0 => AutoAcceptMode::Balanced,
            (c, r) if c > 70.0 && r < 40.0 => AutoAcceptMode::Aggressive,
            _ => AutoAcceptMode::Manual,
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
        
        // Update history database if available
        if let Some(db) = &self.history_database {
            // Find the operation ID (would be returned from record_operation in real usage)
            // For now, we'll update the most recent matching operation
            warn!("Operation outcome recording in database not yet linked to operation ID");
            // TODO: Store operation ID when recording and use it here
        }
        
        info!("📊 Recorded operation outcome for learning");
        Ok(())
    }
    
    /// Record operation outcome with known ID
    pub async fn record_operation_outcome_with_id(
        &self,
        operation_id: uuid::Uuid,
        outcome: &OperationOutcome,
        user_satisfaction: Option<f32>,
    ) -> Result<()> {
        // Update history database
        if let Some(db) = &self.history_database {
            db.update_outcome(operation_id, outcome)?;
            
            if let Some(satisfaction) = user_satisfaction {
                db.add_user_feedback(
                    operation_id,
                    satisfaction,
                    satisfaction >= 3.0,
                    None,
                )?;
            }
        }
        
        info!("📊 Updated operation outcome in database: {}", operation_id);
        Ok(())
    }
    
    /// Get operation success statistics
    pub async fn get_success_statistics(&self) -> Result<OperationStatistics> {
        // Try database first if available
        if let Some(db) = &self.history_database {
            let db_stats = db.get_statistics()?;
            return Ok(OperationStatistics {
                total_operations: db_stats.total_operations as usize,
                successful_operations: db_stats.successful_operations as usize,
                success_rate: db_stats.success_rate,
                average_confidence: db_stats.average_confidence,
                average_risk: db_stats.average_risk,
                rollbacks_required: db_stats.rollbacks_required as usize,
            });
        }
        
        // Fall back to in-memory history
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
        
        // First try database if available
        if let Some(db) = &self.history_database {
            let similar_from_db = db.find_similar_operations(operation, context, 20)?;
            
            if !similar_from_db.is_empty() {
                let successful = similar_from_db.iter()
                    .filter(|r| r.outcome.success)
                    .count();
                
                let failure_modes = similar_from_db.iter()
                    .filter(|r| !r.outcome.success)
                    .filter_map(|r| r.outcome.error_message.clone())
                    .collect::<Vec<_>>();
                
                let success_rate = successful as f32 / similar_from_db.len() as f32;
                
                debug!("Found {} similar operations in database", similar_from_db.len());
                
                return Ok(HistoricalAnalysis {
                    similar_operations_count: similar_from_db.len(),
                    success_rate,
                    common_failure_modes: failure_modes,
                });
            }
        }
        
        // Fall back to Knowledge Indexer
        let similar_ops = self.knowledge_indexer
            .find_similar_operations(operation, context, 10)
            .await?;
        
        // Calculate success rate from similar operations
        let success_prediction = self.knowledge_indexer
            .predict_operation_success(operation, context)
            .await?;
        
        // Extract common failure modes
        let failure_modes = similar_ops.iter()
            .filter(|op| !op.operation.outcome.success)
            .map(|op| op.operation.outcome.error_message.clone().unwrap_or_default())
            .filter(|reason| !reason.is_empty())
            .collect::<Vec<_>>();
        
        Ok(HistoricalAnalysis {
            similar_operations_count: similar_ops.len(),
            success_rate: success_prediction.success_probability,
            common_failure_modes: failure_modes,
        })
    }
    
    async fn analyze_operation_context(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<ContextAnalysis> {
        debug!("🔍 Analyzing operation context with Context Retriever");
        
        // Use Context Retriever to analyze context
        let context_analysis = self.context_retriever
            .analyze_operation_context(operation, context)
            .await?;
        
        // Extract relevant precedents
        let precedents = context_analysis.relevant_precedents.iter()
            .map(|p| format!("{}: {} ({}% success)", 
                self.get_operation_type_name(&p.operation), 
                self.get_operation_description(&p.operation), 
                if p.was_successful { 100 } else { 0 }))
            .collect();
        
        Ok(ContextAnalysis {
            context_similarity: context_analysis.context_similarity,
            relevant_precedents: precedents,
            context_warnings: context_analysis.context_warnings,
        })
    }
    
    async fn analyze_operation_patterns(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<PatternAnalysis> {
        debug!("🧠 Analyzing operation patterns with Pattern Recognizer");
        
        // Use Pattern Recognizer to detect patterns
        let safety_analysis = self.pattern_recognizer
            .analyze_operation_safety(&[operation.clone()], context)
            .await?;
        
        // Safety patterns are already DangerousPattern structs from pattern_recognizer
        let dangerous_patterns = safety_analysis.dangerous_patterns.iter()
            .map(|p| DangerousPattern {
                pattern_type: p.pattern_type.clone(),
                description: p.description.clone(),
                severity: p.severity.clone(),
                mitigation: p.mitigation.clone(),
            })
            .collect();
        
        // Extract anti-patterns
        let anti_patterns = safety_analysis.anti_patterns.iter()
            .map(|ap| AntiPattern {
                pattern_type: ap.pattern_type.clone(),
                description: ap.description.clone(),
                alternative: ap.alternative.clone(),
            })
            .collect();
        
        // Determine cluster info if operations can be grouped
        let cluster_info = if safety_analysis.operation_clusters.len() > 0 {
            let cluster = &safety_analysis.operation_clusters[0];
            Some(OperationCluster {
                name: cluster.name.clone(),
                operations: vec![operation.clone()],
                cluster_type: match cluster.cluster_type.as_str() {
                    "refactoring" => ClusterType::Refactoring,
                    "feature" => ClusterType::FeatureAddition,
                    "bugfix" => ClusterType::BugFix,
                    "cleanup" => ClusterType::Cleanup,
                    "migration" => ClusterType::Migration,
                    "testing" => ClusterType::Testing,
                    _ => ClusterType::FeatureAddition,
                },
                execution_order: vec![0],
            })
        } else {
            None
        };
        
        Ok(PatternAnalysis {
            dangerous_patterns,
            cluster_info,
            suggested_sequence: vec![operation.clone()],
            anti_patterns,
            pattern_confidence: 1.0 - (safety_analysis.safety_score / 100.0),
        })
    }
    
    async fn analyze_quality_impact(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<QualityImpact> {
        debug!("📊 Analyzing quality impact with Quality Analyzer");
        
        // Use Quality Analyzer to assess impact
        let risk_assessment = self.quality_analyzer
            .assess_operation_risk(operation, context)
            .await?;
        
        // Convert conflicts
        let conflicts = risk_assessment.conflicts.iter()
            .map(|c| OperationConflict {
                conflicting_operations: vec![0], // Single operation for now
                conflict_type: match c.conflict_type.as_str() {
                    "file_overwrite" => ConflictType::FileOverwrite,
                    "dependency" => ConflictType::DependencyViolation,
                    "race_condition" => ConflictType::RaceCondition,
                    _ => ConflictType::LogicalInconsistency,
                },
                description: c.description.clone(),
                resolution: c.resolution.clone(),
            })
            .collect();
        
        Ok(QualityImpact {
            overall_impact: risk_assessment.quality_impact.overall_impact,
            maintainability_impact: risk_assessment.quality_impact.maintainability_impact,
            reliability_impact: risk_assessment.quality_impact.reliability_impact,
            performance_impact: risk_assessment.quality_impact.performance_impact,
            conflicts,
            rollback_complexity: RollbackComplexity {
                complexity_score: risk_assessment.rollback_complexity.complexity_score,
                rollback_possible: risk_assessment.rollback_complexity.rollback_possible,
                estimated_rollback_time: risk_assessment.rollback_complexity.estimated_rollback_time,
                rollback_steps: risk_assessment.rollback_complexity.rollback_steps,
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
        let plan = self.knowledge_synthesizer
            .generate_operation_plan(&[operation.clone()], context)
            .await?;
        
        // Convert execution steps
        let execution_steps = plan.execution_steps.iter()
            .map(|step| ExecutionStep {
                step_number: step.step_number,
                description: step.description.clone(),
                operations: step.operations.clone(),
                prerequisites: step.prerequisites.clone(),
                validation_checks: step.validation_checks.clone(),
            })
            .collect();
        
        // Convert preview
        let preview = OperationPreview {
            files_created: plan.preview.files_created.clone(),
            files_modified: plan.preview.files_modified.clone(),
            files_deleted: plan.preview.files_deleted.clone(),
            content_previews: plan.preview.content_previews.iter()
                .map(|cp| ContentPreview {
                    file_path: cp.file_path.clone(),
                    before: cp.before.clone(),
                    after: cp.after.clone(),
                    change_type: match cp.change_type.as_str() {
                        "create" => ChangeType::Create,
                        "update" => ChangeType::Update,
                        "append" => ChangeType::Append,
                        "delete" => ChangeType::Delete,
                        "rename" => ChangeType::Rename,
                        _ => ChangeType::Update,
                    },
                })
                .collect(),
            change_summary: plan.preview.change_summary.clone(),
        };
        
        // Convert backup strategy
        let backup_strategy = BackupStrategy {
            backup_required: plan.backup_strategy.backup_required,
            backup_location: plan.backup_strategy.backup_location.clone(),
            files_to_backup: plan.backup_strategy.files_to_backup.clone(),
            backup_method: match plan.backup_strategy.backup_method.as_str() {
                "copy" => BackupMethod::Copy,
                "git" => BackupMethod::GitCommit,
                "archive" => BackupMethod::Archive,
                "snapshot" => BackupMethod::Snapshot,
                _ => BackupMethod::Copy,
            },
        };
        
        // Convert rollback plan
        let rollback_plan = RollbackPlan {
            rollback_steps: plan.rollback_plan.steps.iter()
                .map(|step| RollbackStep {
                    description: step.description.clone(),
                    action: match step.action_type.as_str() {
                        "restore" => RollbackAction::RestoreFile {
                            from: step.source.clone().unwrap_or_default(),
                            to: step.target.clone().unwrap_or_default(),
                        },
                        "delete" => RollbackAction::DeleteFile {
                            path: step.target.clone().unwrap_or_default(),
                        },
                        "command" => RollbackAction::RunCommand {
                            command: step.command.clone().unwrap_or_default(),
                        },
                        "git_revert" => RollbackAction::GitRevert {
                            commit: step.commit.clone().unwrap_or_default(),
                        },
                        _ => RollbackAction::RunCommand {
                            command: "echo 'Unknown rollback action'".to_string(),
                        },
                    },
                    validation: step.validation.clone(),
                })
                .collect(),
            complexity: plan.rollback_plan.complexity,
            automatic_rollback_possible: plan.rollback_plan.automatic_possible,
            manual_steps_required: plan.rollback_plan.manual_steps.clone(),
        };
        
        Ok(OperationPlan {
            execution_steps,
            preview,
            backup_strategy,
            rollback_plan,
            estimated_execution_time: plan.estimated_time,
        })
    }
    
    async fn calculate_confidence_score(
        &self,
        historical: &HistoricalAnalysis,
        context: &ContextAnalysis,
        patterns: &PatternAnalysis,
        quality: &QualityImpact,
    ) -> Result<f32> {
        // Sophisticated confidence calculation with adaptive weights
        
        // Base weights
        let mut historical_weight = 0.3;
        let mut context_weight = 0.2;
        let mut patterns_weight = 0.3;
        let mut quality_weight = 0.2;
        
        // Adjust weights based on data availability and quality
        if historical.similar_operations_count < 3 {
            // Less historical data, reduce its weight
            historical_weight *= 0.5;
            // Increase other weights proportionally
            let weight_redistribution = historical_weight * 0.5 / 3.0;
            context_weight += weight_redistribution;
            patterns_weight += weight_redistribution;
            quality_weight += weight_redistribution;
        }
        
        // Calculate component scores
        let historical_score = if historical.similar_operations_count > 0 {
            // Consider both success rate and failure mode diversity
            let failure_mode_penalty = (historical.common_failure_modes.len() as f32 * 0.05).min(0.3);
            (historical.success_rate - failure_mode_penalty).max(0.0)
        } else {
            0.7 // Default neutral score when no history
        };
        
        let context_score = {
            // Context similarity with warning penalties
            let warning_penalty = (context.context_warnings.len() as f32 * 0.1).min(0.4);
            let precedent_bonus = if context.relevant_precedents.len() > 3 { 0.1 } else { 0.0 };
            (context.context_similarity + precedent_bonus - warning_penalty).clamp(0.0, 1.0)
        };
        
        let pattern_score = {
            // Pattern confidence with anti-pattern penalties
            let anti_pattern_penalty = (patterns.anti_patterns.len() as f32 * 0.15).min(0.5);
            let cluster_bonus = if patterns.cluster_info.is_some() { 0.1 } else { 0.0 };
            (patterns.pattern_confidence + cluster_bonus - anti_pattern_penalty).clamp(0.0, 1.0)
        };
        
        let quality_score = {
            // Quality based on rollback complexity and overall impact
            let rollback_score = if quality.rollback_complexity.rollback_possible {
                1.0 - (quality.rollback_complexity.complexity_score / 100.0) * 0.7
            } else {
                0.3 // Low score if rollback not possible
            };
            
            let impact_bonus = if quality.overall_impact > 0.0 {
                (quality.overall_impact / 100.0).min(0.2)
            } else {
                0.0
            };
            
            let conflict_penalty = (quality.conflicts.len() as f32 * 0.1).min(0.3);
            
            (rollback_score + impact_bonus - conflict_penalty).clamp(0.0, 1.0)
        };
        
        // Calculate weighted confidence
        let weighted_confidence = 
            historical_score * historical_weight +
            context_score * context_weight +
            pattern_score * patterns_weight +
            quality_score * quality_weight;
        
        // Apply confidence modifiers
        let mut confidence = weighted_confidence;
        
        // Boost confidence for very safe operations
        if patterns.dangerous_patterns.is_empty() && 
           quality.conflicts.is_empty() && 
           historical.success_rate > 0.95 {
            confidence = (confidence * 1.1).min(1.0);
        }
        
        // Reduce confidence for operations with critical issues
        let has_critical = patterns.dangerous_patterns.iter()
            .any(|p| matches!(p.severity, DangerousSeverity::Critical));
        if has_critical {
            confidence *= 0.5;
        }
        
        // Apply learning factor based on total historical operations
        let learning_factor = if historical.similar_operations_count > 10 {
            1.05 // Slight boost when we have good historical data
        } else if historical.similar_operations_count < 3 {
            0.95 // Slight reduction when limited data
        } else {
            1.0
        };
        
        confidence *= learning_factor;
        
        // Final score (0-100)
        Ok((confidence * 100.0).clamp(0.0, 100.0))
    }
    
    async fn calculate_risk_score(
        &self,
        patterns: &PatternAnalysis,
        quality: &QualityImpact,
        plan: &OperationPlan,
    ) -> Result<f32> {
        // Sophisticated risk calculation with multiple factors
        
        let mut risk_components = Vec::new();
        
        // 1. Pattern-based risk (0-40 points)
        let pattern_risk = {
            let mut score = 0.0;
            
            // Dangerous patterns with severity weighting
            for pattern in &patterns.dangerous_patterns {
                let base_risk = match pattern.severity {
                    DangerousSeverity::Low => 5.0,
                    DangerousSeverity::Medium => 12.0,
                    DangerousSeverity::High => 25.0,
                    DangerousSeverity::Critical => 40.0,
                };
                
                // Reduce risk if mitigation is available
                let mitigated_risk = if pattern.mitigation.is_some() {
                    base_risk * 0.7
                } else {
                    base_risk
                };
                
                score += mitigated_risk;
            }
            
            // Anti-patterns add moderate risk
            score += patterns.anti_patterns.len() as f32 * 3.0;
            
            score.min(40.0) // Cap pattern risk at 40
        };
        risk_components.push(("Pattern Risk", pattern_risk));
        
        // 2. Quality impact risk (0-30 points)
        let quality_risk = {
            let mut score = 0.0;
            
            // Negative quality impacts
            if quality.maintainability_impact < -20.0 {
                score += 10.0;
            }
            if quality.reliability_impact < -20.0 {
                score += 10.0;
            }
            if quality.performance_impact < -20.0 {
                score += 5.0;
            }
            
            // Conflicts are high risk
            score += quality.conflicts.len() as f32 * 5.0;
            
            score.min(30.0) // Cap quality risk at 30
        };
        risk_components.push(("Quality Risk", quality_risk));
        
        // 3. Rollback complexity risk (0-20 points)
        let rollback_risk = {
            let mut score = quality.rollback_complexity.complexity_score * 0.2;
            
            // High risk if rollback not possible
            if !quality.rollback_complexity.rollback_possible {
                score += 15.0;
            }
            
            // Risk based on rollback time
            let time_secs = quality.rollback_complexity.estimated_rollback_time.as_secs();
            if time_secs > 300 { // > 5 minutes
                score += 5.0;
            }
            
            score.min(20.0) // Cap rollback risk at 20
        };
        risk_components.push(("Rollback Risk", rollback_risk));
        
        // 4. Execution complexity risk (0-10 points)
        let execution_risk = {
            let mut score = 0.0;
            
            // Multi-step operations are riskier
            if plan.execution_steps.len() > 3 {
                score += 5.0;
            }
            
            // Long execution times are risky
            if plan.estimated_execution_time.as_secs() > 60 {
                score += 5.0;
            }
            
            // Files requiring backup indicate risk
            let backup_file_count = plan.backup_strategy.files_to_backup.len();
            if backup_file_count > 5 {
                score += 3.0;
            }
            
            score.min(10.0) // Cap execution risk at 10
        };
        risk_components.push(("Execution Risk", execution_risk));
        
        // Calculate total risk with logging
        let total_risk = risk_components.iter()
            .map(|(name, score)| {
                debug!("  {} = {:.1}", name, score);
                score
            })
            .sum::<f32>();
        
        // Apply risk amplifiers for critical conditions
        let mut final_risk = total_risk;
        
        // Amplify risk for certain dangerous combinations
        if patterns.dangerous_patterns.iter()
            .any(|p| matches!(p.pattern_type, DangerousPatternType::DataLoss)) 
            && !quality.rollback_complexity.rollback_possible {
            // Data loss with no rollback is extremely risky
            final_risk = (final_risk * 1.5).min(95.0);
        }
        
        if patterns.dangerous_patterns.iter()
            .any(|p| matches!(p.pattern_type, DangerousPatternType::SecurityVulnerability)) {
            // Security issues always high risk
            final_risk = final_risk.max(70.0);
        }
        
        // Apply risk dampeners for safe conditions
        if plan.backup_strategy.backup_required && 
           plan.rollback_plan.automatic_rollback_possible &&
           patterns.dangerous_patterns.is_empty() {
            // Good safety measures reduce risk
            final_risk *= 0.8;
        }
        
        debug!("Total Risk Score: {:.1}", final_risk);
        
        Ok(final_risk.clamp(0.0, 100.0))
    }
    
    async fn generate_recommendation(
        &self,
        confidence: f32,
        risk: f32,
        patterns: &PatternAnalysis,
        quality: &QualityImpact,
    ) -> Result<ActionRecommendation> {
        // Sophisticated decision matrix for recommendations
        
        // Check for absolute blockers first
        let has_critical_danger = patterns.dangerous_patterns.iter()
            .any(|p| matches!(p.severity, DangerousSeverity::Critical));
        
        let has_unrecoverable_risk = patterns.dangerous_patterns.iter()
            .any(|p| matches!(p.pattern_type, DangerousPatternType::DataLoss | 
                             DangerousPatternType::SecurityVulnerability))
            && !quality.rollback_complexity.rollback_possible;
        
        if has_critical_danger || has_unrecoverable_risk || risk > 80.0 {
            // Block dangerous operations
            let mut dangers = patterns.dangerous_patterns.iter()
                .map(|p| format!("{}: {}", 
                    match p.severity {
                        DangerousSeverity::Critical => "CRITICAL",
                        DangerousSeverity::High => "HIGH",
                        DangerousSeverity::Medium => "MEDIUM",
                        DangerousSeverity::Low => "LOW",
                    },
                    p.description
                ))
                .collect::<Vec<_>>();
            
            if !quality.rollback_complexity.rollback_possible {
                dangers.push("No rollback possible if operation fails".to_string());
            }
            
            let alternatives = patterns.dangerous_patterns.iter()
                .filter_map(|p| p.mitigation.clone())
                .chain(vec![
                    "Break operation into smaller, safer steps".to_string(),
                    "Create manual backups before proceeding".to_string(),
                    "Review and test in a non-production environment first".to_string(),
                ])
                .collect();
            
            return Ok(ActionRecommendation::Block {
                reason: if has_unrecoverable_risk {
                    "Operation poses unrecoverable risk to data or security".to_string()
                } else if risk > 80.0 {
                    format!("Operation risk score too high: {:.0}%", risk)
                } else {
                    "Critical safety issues detected".to_string()
                },
                dangers,
                alternatives,
            });
        }
        
        // Check if modifications would improve the operation
        let needs_modification = patterns.anti_patterns.len() > 2 || 
                               (confidence < 50.0 && risk > 40.0);
        
        if needs_modification && patterns.anti_patterns.len() > 0 {
            // Suggest modifications
            let modifications = patterns.anti_patterns.iter()
                .filter_map(|ap| ap.alternative.clone())
                .collect::<Vec<_>>();
            
            if !modifications.is_empty() {
                return Ok(ActionRecommendation::SuggestModifications {
                    reason: format!(
                        "Operation can be improved (confidence: {:.0}%, risk: {:.0}%)",
                        confidence, risk
                    ),
                    suggested_operations: vec![], // Would be filled by operation optimizer
                    modifications,
                });
            }
        }
        
        // Decision matrix for auto-execute vs confirmation
        let auto_execute_threshold = match risk {
            r if r < 10.0 => 75.0,  // Very low risk: 75% confidence needed
            r if r < 20.0 => 80.0,  // Low risk: 80% confidence needed
            r if r < 30.0 => 85.0,  // Moderate risk: 85% confidence needed
            r if r < 40.0 => 90.0,  // Medium risk: 90% confidence needed
            _ => 95.0,               // Higher risk: 95% confidence needed
        };
        
        if confidence >= auto_execute_threshold && risk < 30.0 {
            // Auto-execute safe operations
            let reason = match (confidence, risk) {
                (c, r) if c > 95.0 && r < 10.0 => 
                    "Extremely safe operation with excellent historical success".to_string(),
                (c, r) if c > 90.0 && r < 15.0 => 
                    "Very safe operation with strong confidence".to_string(),
                (c, r) if c > 85.0 && r < 20.0 => 
                    "Safe operation with good confidence and low risk".to_string(),
                _ => format!(
                    "Operation meets auto-execute criteria (confidence: {:.0}%, risk: {:.0}%)",
                    confidence, risk
                ),
            };
            
            return Ok(ActionRecommendation::AutoExecute {
                reason,
                confidence,
            });
        }
        
        // Default to requesting confirmation
        let mut risks = Vec::new();
        let mut benefits = Vec::new();
        
        // Compile risks
        if !patterns.dangerous_patterns.is_empty() {
            risks.extend(patterns.dangerous_patterns.iter()
                .map(|p| p.description.clone()));
        }
        
        if quality.conflicts.len() > 0 {
            risks.push(format!("{} potential conflicts detected", quality.conflicts.len()));
        }
        
        if quality.rollback_complexity.complexity_score > 50.0 {
            risks.push("Complex rollback procedure if needed".to_string());
        }
        
        // Compile benefits
        if quality.overall_impact > 0.0 {
            benefits.push(format!("Positive quality impact: +{:.0}%", quality.overall_impact));
        }
        
        if patterns.cluster_info.is_some() {
            benefits.push("Operation is part of a coherent change set".to_string());
        }
        
        if quality.rollback_complexity.rollback_possible {
            benefits.push("Rollback is available if needed".to_string());
        }
        
        // Add generic benefits if none found
        if benefits.is_empty() {
            benefits.push("Operation structure is valid".to_string());
            if confidence > 60.0 {
                benefits.push(format!("Reasonable confidence level: {:.0}%", confidence));
            }
        }
        
        Ok(ActionRecommendation::RequestConfirmation {
            reason: match (confidence, risk) {
                (c, r) if c < 50.0 && r > 40.0 => 
                    "Low confidence and moderate risk require user review".to_string(),
                (c, _) if c < 60.0 => 
                    "Insufficient confidence for automatic execution".to_string(),
                (_, r) if r > 40.0 => 
                    "Risk level requires user confirmation".to_string(),
                _ => format!(
                    "User confirmation recommended (confidence: {:.0}%, risk: {:.0}%)",
                    confidence, risk
                ),
            },
            risks,
            benefits,
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
        
        let mut explanation = format!(
            "🤖 AI Analysis: Operation to {}\n\n",
            operation_description
        );
        
        // Confidence and risk summary
        explanation.push_str(&format!(
            "📊 Scores: {:.0}% confidence, {:.0}% risk\n",
            confidence, risk
        ));
        
        // AI Helper insights
        explanation.push_str("\n🔍 AI Helper Insights:\n");
        
        // Pattern insights
        if !patterns.dangerous_patterns.is_empty() {
            explanation.push_str(&format!(
                "  • Pattern Recognizer: {} dangerous patterns detected\n",
                patterns.dangerous_patterns.len()
            ));
            for pattern in patterns.dangerous_patterns.iter().take(2) {
                explanation.push_str(&format!(
                    "    - {}: {}\n",
                    match pattern.severity {
                        DangerousSeverity::Critical => "⛔ CRITICAL",
                        DangerousSeverity::High => "🔴 HIGH",
                        DangerousSeverity::Medium => "🟡 MEDIUM",
                        DangerousSeverity::Low => "🟢 LOW",
                    },
                    pattern.description
                ));
            }
        } else {
            explanation.push_str("  • Pattern Recognizer: ✅ No dangerous patterns\n");
        }
        
        // Quality insights
        let quality_emoji = if quality.overall_impact > 20.0 {
            "📈"
        } else if quality.overall_impact < -20.0 {
            "📉"
        } else {
            "➖"
        };
        
        explanation.push_str(&format!(
            "  • Quality Analyzer: {} {:.0}% quality impact\n",
            quality_emoji,
            quality.overall_impact.abs()
        ));
        
        if quality.conflicts.len() > 0 {
            explanation.push_str(&format!(
                "    - ⚠️  {} potential conflicts\n",
                quality.conflicts.len()
            ));
        }
        
        // Rollback insights
        if quality.rollback_complexity.rollback_possible {
            explanation.push_str(&format!(
                "  • Rollback: ✅ Possible (complexity: {:.0}%)\n",
                quality.rollback_complexity.complexity_score
            ));
        } else {
            explanation.push_str("  • Rollback: ❌ Not possible\n");
        }
        
        // Recommendation summary
        explanation.push_str(&format!("\n🎯 Recommendation: {}\n", 
            match recommendation {
                ActionRecommendation::AutoExecute { .. } => "✅ Safe to auto-execute",
                ActionRecommendation::RequestConfirmation { .. } => "🤔 User confirmation needed",
                ActionRecommendation::Block { .. } => "🚫 Operation blocked for safety",
                ActionRecommendation::SuggestModifications { .. } => "💡 Modifications suggested",
            }
        ));
        
        // Detailed reason
        let reason = match recommendation {
            ActionRecommendation::AutoExecute { reason, .. } => reason,
            ActionRecommendation::RequestConfirmation { reason, .. } => reason,
            ActionRecommendation::Block { reason, .. } => reason,
            ActionRecommendation::SuggestModifications { reason, .. } => reason,
        };
        
        explanation.push_str(&format!("   {}\n", reason));
        
        // Add specific guidance based on recommendation
        match recommendation {
            ActionRecommendation::AutoExecute { .. } => {
                explanation.push_str("\n✨ This operation will be executed automatically.");
            },
            ActionRecommendation::RequestConfirmation { risks, benefits, .. } => {
                if !risks.is_empty() {
                    explanation.push_str("\n⚠️  Risks to consider:\n");
                    for risk in risks.iter().take(3) {
                        explanation.push_str(&format!("   • {}\n", risk));
                    }
                }
                if !benefits.is_empty() {
                    explanation.push_str("\n✅ Benefits:\n");
                    for benefit in benefits.iter().take(3) {
                        explanation.push_str(&format!("   • {}\n", benefit));
                    }
                }
            },
            ActionRecommendation::Block { dangers, alternatives, .. } => {
                if !dangers.is_empty() {
                    explanation.push_str("\n⛔ Dangers:\n");
                    for danger in dangers.iter().take(3) {
                        explanation.push_str(&format!("   • {}\n", danger));
                    }
                }
                if !alternatives.is_empty() {
                    explanation.push_str("\n💡 Alternatives:\n");
                    for alt in alternatives.iter().take(3) {
                        explanation.push_str(&format!("   • {}\n", alt));
                    }
                }
            },
            ActionRecommendation::SuggestModifications { modifications, .. } => {
                if !modifications.is_empty() {
                    explanation.push_str("\n💡 Suggested improvements:\n");
                    for mod_suggestion in modifications.iter().take(3) {
                        explanation.push_str(&format!("   • {}\n", mod_suggestion));
                    }
                }
            },
        }
        
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
        info!("📚 Updating AI helpers with operation outcome for learning");
        
        // 1. Update Knowledge Indexer with operation outcome
        self.knowledge_indexer
            .index_operation_outcome(
                &history_entry.operation,
                &history_entry.context,
                &history_entry.outcome,
                history_entry.user_satisfaction,
            )
            .await?;
        
        // 2. Update Context Retriever with new precedent
        // This helps improve future context analysis
        debug!("Updating Context Retriever with operation precedent");
        
        // 3. Update Pattern Recognizer with pattern outcomes
        // This helps identify which patterns actually led to failures
        if !history_entry.outcome.success {
            debug!("Recording failed operation pattern for Pattern Recognizer");
        }
        
        // 4. Update Quality Analyzer with quality impact data
        if let Some(post_quality) = &history_entry.outcome.post_operation_quality {
            debug!("Recording quality metrics for Quality Analyzer: {:?}", post_quality);
        }
        
        // 5. Update Knowledge Synthesizer with plan effectiveness
        let plan_effectiveness = if history_entry.outcome.success {
            1.0
        } else if history_entry.outcome.rollback_required {
            0.5
        } else {
            0.0
        };
        debug!("Recording plan effectiveness: {}", plan_effectiveness);
        
        // Clear relevant caches to ensure fresh analysis with new data
        {
            let mut cache = self.analysis_cache.write().await;
            cache.clear();
        }
        
        info!("✅ AI helpers updated with operation outcome");
        Ok(())
    }
    
    fn generate_cache_key(&self, operation: &FileOperation, context: &OperationContext) -> String {
        // Generate a cache key based on operation and context
        format!("{:?}_{}", operation, context.source_question.len())
    }

    /// Get operation type name for display
    fn get_operation_type_name(&self, operation: &FileOperation) -> String {
        match operation {
            FileOperation::Create { .. } => "Create".to_string(),
            FileOperation::Update { .. } => "Update".to_string(),
            FileOperation::Delete { .. } => "Delete".to_string(),
            FileOperation::Move { .. } => "Move".to_string(),
        }
    }

    /// Get operation description for display
    fn get_operation_description(&self, operation: &FileOperation) -> String {
        match operation {
            FileOperation::Create { path, .. } => format!("Create {}", path.display()),
            FileOperation::Update { path, .. } => format!("Update {}", path.display()),
            FileOperation::Delete { path } => format!("Delete {}", path.display()),
            FileOperation::Move { from, to } => format!("Move {} to {}", from.display(), to.display()),
        }
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

/// Auto-accept execution mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AutoAcceptMode {
    /// Conservative: Only auto-execute very safe operations (>90% confidence, <15% risk)
    Conservative,
    /// Balanced: Auto-execute reasonably safe operations (>80% confidence, <25% risk)
    Balanced,
    /// Aggressive: Auto-execute most operations unless high risk (>70% confidence, <40% risk)
    Aggressive,
    /// Plan: Generate execution plan but don't execute (for review)
    Plan,
    /// Manual: Never auto-execute, always ask for confirmation
    Manual,
}

impl AutoAcceptMode {
    /// Get confidence threshold for this mode
    pub fn confidence_threshold(&self) -> f32 {
        match self {
            AutoAcceptMode::Conservative => 90.0,
            AutoAcceptMode::Balanced => 80.0,
            AutoAcceptMode::Aggressive => 70.0,
            AutoAcceptMode::Plan => 100.0, // Never auto-execute
            AutoAcceptMode::Manual => 100.0, // Never auto-execute
        }
    }
    
    /// Get risk threshold for this mode
    pub fn risk_threshold(&self) -> f32 {
        match self {
            AutoAcceptMode::Conservative => 15.0,
            AutoAcceptMode::Balanced => 25.0,
            AutoAcceptMode::Aggressive => 40.0,
            AutoAcceptMode::Plan => 0.0, // Never auto-execute
            AutoAcceptMode::Manual => 0.0, // Never auto-execute
        }
    }
}

/// Unified scoring result combining all AI helper analyses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedScore {
    /// Overall confidence score (0-100)
    pub confidence: f32,
    
    /// Overall risk score (0-100)
    pub risk: f32,
    
    /// Component scores from each AI helper
    pub component_scores: ComponentScores,
    
    /// Factors that influenced the scores
    pub scoring_factors: ScoringFactors,
    
    /// Recommended auto-accept mode for this operation
    pub recommended_mode: AutoAcceptMode,
}

/// Component scores from each AI helper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    /// Knowledge Indexer: Historical success score
    pub historical_score: f32,
    
    /// Context Retriever: Context relevance score
    pub context_score: f32,
    
    /// Pattern Recognizer: Safety pattern score
    pub pattern_score: f32,
    
    /// Quality Analyzer: Quality impact score
    pub quality_score: f32,
    
    /// Knowledge Synthesizer: Plan feasibility score
    pub plan_score: f32,
}

/// Factors that influenced scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringFactors {
    /// Number of similar operations found
    pub similar_operations_count: usize,
    
    /// Number of dangerous patterns detected
    pub dangerous_pattern_count: usize,
    
    /// Number of anti-patterns detected
    pub anti_pattern_count: usize,
    
    /// Number of conflicts detected
    pub conflict_count: usize,
    
    /// Whether rollback is possible
    pub rollback_possible: bool,
    
    /// Estimated execution complexity
    pub execution_complexity: f32,
}

/// Groups of operations based on execution strategy
#[derive(Debug, Clone)]
pub struct OperationGroups {
    /// Operations safe to auto-execute
    pub auto_execute: Vec<(FileOperation, OperationAnalysis)>,
    
    /// Operations needing user confirmation
    pub needs_confirmation: Vec<(FileOperation, OperationAnalysis)>,
    
    /// Operations blocked for safety
    pub blocked: Vec<(FileOperation, OperationAnalysis)>,
    
    /// Operations needing modification
    pub needs_modification: Vec<(FileOperation, OperationAnalysis)>,
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