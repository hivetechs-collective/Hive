// Consensus Engine Module - 4-Stage Pipeline Implementation
// Preserves exact behavior from TypeScript version with Rust performance

pub mod ai_file_executor;
pub mod ai_operation_parser;
pub mod ai_rollback_executor;
pub mod cancellation;
pub mod codebase_intelligence;
pub mod confidence_scoring;
pub mod cross_validator;
pub mod curator_output_format;
pub mod dependency_graph;
pub mod direct_executor;
pub mod engine;
pub mod fact_checker;
pub mod file_executor;
pub mod file_operations;
pub mod file_planner;
pub mod intelligent_feedback;
pub mod iteration_handler;
pub mod learning_system;
pub mod maintenance;
pub mod memory;
pub mod mode_detector;
pub mod models;
pub mod openrouter;
pub mod operation_analysis;
pub mod operation_clustering;
pub mod operation_history;
pub mod operation_intelligence;
pub mod operation_parser;
pub mod operation_preview;
pub mod operation_preview_generator;
pub mod operation_validator;
pub mod outcome_tracker;
pub mod pipeline;
pub mod profiles;
pub mod repository_context;
pub mod rollback_executor;
pub mod rollback_plan;
pub mod rollback_planner;
pub mod rollback_planner_v2;
pub mod safety_guardrails;
pub mod smart_decision_engine;
pub mod stages;
pub mod streaming;
pub mod streaming_executor;
pub mod temporal;
pub mod types;
pub mod verification;
pub mod verified_context_builder;
// pub mod ai_enhanced_executor; // Removed - violated architecture (see docs/AI_ENHANCED_CAPABILITIES_DESIGN.md)
// pub mod optimize; // TODO: Fix profile field usage

pub use ai_operation_parser::{
    AIOperationParser, FileOperationWithMetadata, LanguageParser,
    ParsedOperations as AIParsedOperations, SourceLocation,
};
pub use cancellation::{
    CancellableOperation, CancellationChecker, CancellationReason, CancellationReceiver,
    CancellationToken,
};
pub use confidence_scoring::{
    ConfidenceInterval, ConfidenceScoringEngine, ImprovementSuggestion, PrimaryFactor,
    ScoreBreakdown, ScoringResult, ScoringStatistics, ScoringWeights,
};
pub use cross_validator::{
    ConsensusHealth, ConsensusReport, ContradictionSeverity, ContradictionType, CrossValidator,
    SemanticContradiction, StageDiscrepancy,
};
pub use dependency_graph::{
    DependencyEdge, DependencyGraph, DependencyGraphGenerator, DependencyType, GraphAnalysis,
    GraphConfig, GraphVisuals, NodeMetadata, OperationNode, ParallelGroup,
    RiskLevel as GraphRiskLevel, RiskSummary,
};
pub use direct_executor::{DirectExecutionBuilder, DirectExecutionHandler, InlineOperationParser};
pub use engine::ConsensusEngine;
pub use fact_checker::{
    Contradiction, FactChecker, RecommendedAction as FactCheckerRecommendedAction, ValidationResult,
};
pub use file_executor::{
    BackupInfo as FileBackupInfo, BackupManager as FileBackupManager, ExecutionResult,
    ExecutionSummary, ExecutorConfig, FileOperationExecutor, SyntaxValidator,
};
pub use intelligent_feedback::{
    AIInsight, ConfidenceExplanation, IntelligentFeedbackGenerator,
    OperationPreview as FeedbackOperationPreview, RiskExplanation, Suggestion, SuggestionAction,
    UserFeedback, VisualIndicators,
};
pub use iteration_handler::{
    ErrorType, ExecutedOperation, ExecutionResult as IterationExecutionResult, IterationContext,
    IterationError, IterationHandler, NextIteration, Recommendation, TestResult,
};
pub use learning_system::{
    ContinuousLearningSystem, ExperimentResult, ExperimentStatus, ExperimentType,
    LearningCycleResult, LearningExperiment, LearningState, LearningSystemConfig,
    LearningSystemStatus, LearningTrend, PendingImprovement,
};
pub use mode_detector::{ComplexityAnalyzer, ExecutionMode, ModeDetector, PatternMatcher};
pub use operation_analysis::{
    ActionPriority, ActionRecommendation, AnalysisStatistics, AutoAcceptMode, ComponentScores,
    OperationAnalysis, OperationContext, OperationGroups, OperationOutcome, RecommendedAction,
    ScoringFactors, UnifiedScore,
};
pub use operation_clustering::{
    ClusterType, ClusteringConfig, ClusteringResult, OperationCluster, OperationClusterer,
};
pub use operation_intelligence::OperationIntelligenceCoordinator;
pub use operation_parser::{
    AIOperationInsights, ComplexityLevel, EnhancedFileOperation, EnhancementConfig, GlobalContext,
    OperationContext as ParsedOperationContext, OperationParser, ParsedOperations, ParsingMetadata,
};
pub use operation_preview::{
    CombinedImpact, DiffChunk, DiffView, ExecutionTimeline, FileState, LineChangeSummary,
    OperationImpact, OperationPreview, OperationPreviewGenerator, OperationPreviewSet,
    PreviewConfig, VisualSummary,
};
pub use operation_preview_generator::{
    DiffView as AIDiffView, FileState as AIFileState, OperationImpact as AIOperationImpact,
    OperationPreview as AIOperationPreview,
    OperationPreviewGenerator as AIOperationPreviewGenerator,
    OperationPreviewSet as AIOperationPreviewSet, PreviewConfig as AIPreviewConfig,
};
pub use operation_validator::{
    CheckCategory, OperationValidator, RiskAssessment, RiskLevel as OpRiskLevel, Severity,
    SuggestedFix, ValidationCheck, ValidationConfig, ValidationError,
    ValidationResult as OpValidationResult, ValidationStatus, ValidationWarning,
};
pub use outcome_tracker::{
    AccuracyMetrics, ErrorPattern, HelperAccuracy, LearningInsights, OperationOutcomeTracker,
    OperationResult, OutcomeAccuracy, OutcomeTrackingConfig, PredictionError, PredictionErrorType,
    RetrainingResult, TrackedOutcome,
};
pub use pipeline::ConsensusPipeline;
pub use repository_context::{RepositoryContext, RepositoryContextManager};
pub use rollback_executor::{
    BackupManager, GitManager, ProgressTracker, RollbackError, RollbackErrorType,
    RollbackExecution, RollbackExecutionConfig, RollbackExecutionStatus, RollbackExecutor,
    RollbackProgress, RollbackStepResult, RollbackStepStatus, RollbackSummary, VerificationResult,
    VerificationSystem,
};
pub use rollback_plan::{
    RollbackAction, RollbackConfig as RollbackConfigV2,
    RollbackExecutionResult as RollbackExecutionResultV2, RollbackExecutor as RollbackExecutorV2,
    RollbackOperation as RollbackOperationV2, RollbackPlan as RollbackPlanV2,
    RollbackPlanGenerator,
};
pub use rollback_planner::{
    BackupInfo, BackupMethod, DataLossPotential, RiskCategory, RollbackConfig,
    RollbackExecutionResult, RollbackOperation, RollbackPlan, RollbackPlanner, RollbackRisk,
    RollbackRiskAssessment, RollbackStep, RollbackStrategy, VerificationStep, VerificationType,
};
pub use safety_guardrails::{
    ComprehensiveSafetyResult, EnforcementAction, ExecutionRequirements, RulePattern, SafetyConfig,
    SafetyContext, SafetyGuardrailSystem, SafetyMetrics, SafetyRule, SafetyValidationResult,
    SafetyValidator, SafetyViolation, ViolationSeverity,
};
pub use smart_decision_engine::{
    CustomRule, DecisionMetrics, ExecutionDecision, RuleAction, SmartDecisionEngine, UserChoice,
    UserDecision, UserPreferences,
};
pub use streaming::{
    ChannelStreamingCallbacks, ConsensusEvent, ConsensusResponseResult, ConsensusStage,
    StreamingCallbacks, StreamingResponse,
};
pub use streaming_executor::{
    ExecutionStatus, StatusCallback, StreamingExecutorBuilder, StreamingOperationExecutor,
};
pub use temporal::TemporalContextProvider;
pub use types::{
    ConsensusConfig, ConsensusProfile, ConsensusRequest, ConsensusResponse, ConsensusResult,
    ResponseMetadata, Stage,
};
pub use verification::{build_stage_context, RepositoryFacts, RepositoryVerifier};
pub use verified_context_builder::VerifiedContextBuilder;

// Re-export commonly used items
pub use models::{DynamicModelSelector, ModelInfo, ModelManager};
pub use openrouter::OpenRouterClient;
pub use profiles::ExpertProfileManager;
pub use stages::{CuratorStage, GeneratorStage, RefinerStage, ValidatorStage};
