/// OpenRouter Provider Module
/// 
/// Comprehensive integration with OpenRouter API providing access to 323+ AI models
/// with streaming, cost tracking, performance monitoring, and intelligent routing.

pub mod client;
pub mod cost;
pub mod models;
pub mod performance;
pub mod streaming;

// Re-export commonly used types
pub use client::{
    ChatCompletionRequest, ChatCompletionResponse, MessageRole, ModelInfo, OpenRouterClient,
    OpenRouterMessage, OpenRouterResponse, UsageInfo,
};

pub use models::{ModelSelector, ModelSelectionStrategy, TaskComplexity};

pub use streaming::{StreamingCallbacks, StreamingClient, StreamingOptions, StreamingResponse};

pub use cost::{CostCalculator, CostEstimate, CostTracker};

pub use performance::{
    ModelPerformance, PerformanceMetrics, PerformanceTracker, ScoringWeights, 
    ABTestConfig, ABTestStatus, ModelRanking, TaskRecommendation, HealthStatus, CircuitState
};

/// Create a new OpenRouter client with the provided API key
pub fn create_client(api_key: String) -> anyhow::Result<OpenRouterClient> {
    OpenRouterClient::new(api_key)
}

/// Create a new streaming client with the provided API key
pub fn create_streaming_client(api_key: String) -> anyhow::Result<StreamingClient> {
    StreamingClient::new(api_key)
}