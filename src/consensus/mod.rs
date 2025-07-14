// Consensus Engine Module - 4-Stage Pipeline Implementation
// Preserves exact behavior from TypeScript version with Rust performance

pub mod engine;
pub mod maintenance;
pub mod models;
pub mod openrouter;
pub mod pipeline;
pub mod profiles;
pub mod stages;
pub mod streaming;
pub mod temporal;
pub mod types;
// pub mod optimize; // TODO: Fix profile field usage

pub use engine::ConsensusEngine;
pub use pipeline::ConsensusPipeline;
pub use streaming::{
    ChannelStreamingCallbacks, ConsensusEvent, ConsensusResponseResult, ConsensusStage,
    StreamingCallbacks, StreamingResponse,
};
pub use temporal::TemporalContextProvider;
pub use types::{
    ConsensusConfig, ConsensusProfile, ConsensusRequest, ConsensusResponse, ConsensusResult,
    ResponseMetadata, Stage,
};

// Re-export commonly used items
pub use models::{DynamicModelSelector, ModelInfo, ModelManager};
pub use openrouter::OpenRouterClient;
pub use profiles::ExpertProfileManager;
pub use stages::{CuratorStage, GeneratorStage, RefinerStage, ValidatorStage};
