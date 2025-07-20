// Consensus Engine Module - 4-Stage Pipeline Implementation
// Preserves exact behavior from TypeScript version with Rust performance

pub mod cancellation;
pub mod engine;
pub mod file_operations;
pub mod maintenance;
pub mod models;
pub mod openrouter;
pub mod pipeline;
pub mod profiles;
pub mod repository_context;
pub mod stages;
pub mod streaming;
pub mod temporal;
pub mod types;
pub mod codebase_intelligence;
pub mod verification;
pub mod fact_checker;
pub mod cross_validator;
pub mod verified_context_builder;
pub mod operation_intelligence;
pub mod operation_analysis;
pub mod operation_history;
pub mod smart_decision_engine;
pub mod operation_clustering;
pub mod intelligent_feedback;
pub mod confidence_scoring;
pub mod operation_parser;
pub mod operation_validator;
pub mod operation_preview;
pub mod dependency_graph;
pub mod rollback_plan;
pub mod rollback_planner;
pub mod rollback_executor;
pub mod outcome_tracker;
pub mod learning_system;
pub mod safety_guardrails;
pub mod file_executor;
pub mod ai_operation_parser;
pub mod operation_preview_generator;
// pub mod optimize; // TODO: Fix profile field usage

pub use cancellation::{CancellationToken, CancellationReceiver, CancellationReason, CancellableOperation, CancellationChecker};
pub use engine::ConsensusEngine;
pub use pipeline::ConsensusPipeline;
pub use repository_context::{RepositoryContext, RepositoryContextManager};
pub use streaming::{
    ChannelStreamingCallbacks, ConsensusEvent, ConsensusResponseResult, ConsensusStage,
    StreamingCallbacks, StreamingResponse,
};
pub use temporal::TemporalContextProvider;
pub use types::{
    ConsensusConfig, ConsensusProfile, ConsensusRequest, ConsensusResponse, ConsensusResult,
    ResponseMetadata, Stage,
};
pub use verification::{RepositoryFacts, RepositoryVerifier, build_stage_context};
pub use fact_checker::{FactChecker, ValidationResult, Contradiction, RecommendedAction as FactCheckerRecommendedAction};
pub use cross_validator::{CrossValidator, ConsensusHealth, StageDiscrepancy, ConsensusReport, SemanticContradiction, ContradictionType, ContradictionSeverity};
pub use verified_context_builder::VerifiedContextBuilder;
pub use operation_intelligence::OperationIntelligenceCoordinator;
pub use operation_analysis::{
    OperationAnalysis, ActionRecommendation, ActionPriority,
    UnifiedScore, ComponentScores, ScoringFactors, OperationGroups,
    OperationContext, RecommendedAction, AnalysisStatistics,
    OperationOutcome, AutoAcceptMode
};
pub use smart_decision_engine::{
    SmartDecisionEngine, ExecutionDecision, UserDecision, UserChoice,
    UserPreferences, CustomRule, RuleAction, DecisionMetrics
};
pub use operation_clustering::{
    OperationClusterer, OperationCluster, ClusterType, ClusteringResult,
    ClusteringConfig
};
pub use intelligent_feedback::{
    IntelligentFeedbackGenerator, UserFeedback, RiskExplanation, ConfidenceExplanation,
    AIInsight, Suggestion, OperationPreview as FeedbackOperationPreview, VisualIndicators, SuggestionAction
};
pub use confidence_scoring::{
    ConfidenceScoringEngine, ScoringWeights, ScoringResult, ScoreBreakdown,
    ConfidenceInterval, PrimaryFactor, ImprovementSuggestion, ScoringStatistics
};
pub use operation_parser::{
    OperationParser, EnhancementConfig, ParsedOperations, EnhancedFileOperation,
    OperationContext as ParsedOperationContext, GlobalContext, ComplexityLevel,
    ParsingMetadata, AIOperationInsights
};
pub use operation_validator::{
    OperationValidator, ValidationConfig, ValidationResult as OpValidationResult, ValidationStatus,
    ValidationCheck, ValidationError, ValidationWarning, SuggestedFix,
    RiskAssessment, RiskLevel as OpRiskLevel, CheckCategory, Severity
};
pub use operation_preview::{
    OperationPreviewGenerator, PreviewConfig, OperationPreviewSet, OperationPreview,
    FileState, DiffView, DiffChunk, LineChangeSummary, OperationImpact,
    CombinedImpact, ExecutionTimeline, VisualSummary
};
pub use dependency_graph::{
    DependencyGraphGenerator, GraphConfig, DependencyGraph, OperationNode,
    NodeMetadata, RiskLevel as GraphRiskLevel, DependencyEdge, DependencyType, ParallelGroup,
    GraphAnalysis, RiskSummary, GraphVisuals
};
pub use rollback_planner::{
    RollbackPlanner, RollbackConfig, RollbackPlan, RollbackStrategy,
    RollbackStep, RollbackOperation, BackupInfo, BackupMethod,
    RollbackRiskAssessment, RollbackRisk, RiskCategory, DataLossPotential,
    VerificationStep, VerificationType, RollbackExecutionResult
};
pub use rollback_executor::{
    RollbackExecutor, RollbackExecutionConfig, RollbackExecution, RollbackExecutionStatus,
    RollbackStepResult, RollbackStepStatus, RollbackError, RollbackErrorType,
    RollbackSummary, ProgressTracker, RollbackProgress, VerificationResult,
    BackupManager, GitManager, VerificationSystem
};
pub use outcome_tracker::{
    OperationOutcomeTracker, OutcomeTrackingConfig, TrackedOutcome, OperationResult,
    OutcomeAccuracy, PredictionError, PredictionErrorType, AccuracyMetrics,
    HelperAccuracy, LearningInsights, ErrorPattern, RetrainingResult
};
pub use learning_system::{
    ContinuousLearningSystem, LearningSystemConfig, LearningState, LearningExperiment,
    ExperimentType, ExperimentStatus, ExperimentResult, LearningCycleResult,
    PendingImprovement, LearningSystemStatus, LearningTrend
};
pub use safety_guardrails::{
    SafetyGuardrailSystem, SafetyConfig, SafetyValidator, SafetyValidationResult,
    SafetyViolation, ViolationSeverity, SafetyContext, SafetyRule, RulePattern,
    ComprehensiveSafetyResult, EnforcementAction, ExecutionRequirements, SafetyMetrics
};
pub use file_executor::{
    FileOperationExecutor, ExecutorConfig, ExecutionResult, ExecutionSummary,
    BackupInfo as FileBackupInfo, BackupManager as FileBackupManager, SyntaxValidator
};
pub use ai_operation_parser::{
    AIOperationParser, ParsedOperations as AIParsedOperations, FileOperationWithMetadata, SourceLocation,
    LanguageParser
};
pub use operation_preview_generator::{
    OperationPreviewGenerator as AIOperationPreviewGenerator, PreviewConfig as AIPreviewConfig,
    OperationPreviewSet as AIOperationPreviewSet, OperationPreview as AIOperationPreview,
    FileState as AIFileState, DiffView as AIDiffView, OperationImpact as AIOperationImpact
};
pub use rollback_plan::{
    RollbackPlan as RollbackPlanV2, RollbackPlanGenerator, 
    RollbackConfig as RollbackConfigV2, RollbackExecutor as RollbackExecutorV2,
    RollbackOperation as RollbackOperationV2, RollbackAction, 
    RollbackExecutionResult as RollbackExecutionResultV2
};

// Re-export commonly used items
pub use models::{DynamicModelSelector, ModelInfo, ModelManager};
pub use openrouter::OpenRouterClient;
pub use profiles::ExpertProfileManager;
pub use stages::{CuratorStage, GeneratorStage, RefinerStage, ValidatorStage};
