// Consensus Engine Module - 4-Stage Pipeline Implementation
// Preserves exact behavior from TypeScript version with Rust performance

pub mod engine;
pub mod pipeline;
pub mod stages;
pub mod streaming;
pub mod temporal;
pub mod types;
pub mod optimize;
pub mod models;
pub mod openrouter;
pub mod profiles;

pub use engine::ConsensusEngine;
pub use pipeline::ConsensusPipeline;
pub use streaming::{ConsensusEvent, StreamingCallbacks, StreamingResponse, ConsensusStage, ConsensusResponseResult};
pub use temporal::TemporalContextProvider;
pub use types::{ConsensusConfig, ConsensusProfile, ConsensusResult, ConsensusRequest, ConsensusResponse, Stage};

// Re-export commonly used items
pub use stages::{CuratorStage, GeneratorStage, RefinerStage, ValidatorStage};
pub use models::{ModelInfo, ModelManager, DynamicModelSelector};
pub use openrouter::OpenRouterClient;
pub use profiles::ExpertProfileManager;