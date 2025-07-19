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
pub use fact_checker::{FactChecker, ValidationResult, Contradiction, RecommendedAction};
pub use cross_validator::{CrossValidator, ConsensusHealth, StageDiscrepancy, ConsensusReport, SemanticContradiction, ContradictionType, ContradictionSeverity};
pub use verified_context_builder::VerifiedContextBuilder;
pub use operation_intelligence::{
    OperationIntelligenceCoordinator, OperationAnalysis, ActionRecommendation, 
    OperationStatistics, AutoAcceptMode, UnifiedScore, ComponentScores, 
    ScoringFactors, OperationGroups
};

// Re-export commonly used items
pub use models::{DynamicModelSelector, ModelInfo, ModelManager};
pub use openrouter::OpenRouterClient;
pub use profiles::ExpertProfileManager;
pub use stages::{CuratorStage, GeneratorStage, RefinerStage, ValidatorStage};
