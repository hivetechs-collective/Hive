// Type definitions for consensus engine
// Matches TypeScript interface definitions for compatibility

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use crate::consensus::temporal::TemporalContext;

/// Consensus pipeline stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Generator,
    Refiner,
    Validator,
    Curator,
}

impl Stage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Stage::Generator => "generator",
            Stage::Refiner => "refiner",
            Stage::Validator => "validator",
            Stage::Curator => "curator",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Stage::Generator => "Generator",
            Stage::Refiner => "Refiner",
            Stage::Validator => "Validator",
            Stage::Curator => "Curator",
        }
    }
}

/// Consensus profile configuration (matches TypeScript ConsensusProfile)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProfile {
    pub id: String,
    pub profile_name: String,
    pub generator_model: String,
    pub refiner_model: String,
    pub validator_model: String,
    pub curator_model: String,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

impl ConsensusProfile {
    pub fn get_model_for_stage(&self, stage: Stage) -> &str {
        match stage {
            Stage::Generator => &self.generator_model,
            Stage::Refiner => &self.refiner_model,
            Stage::Validator => &self.validator_model,
            Stage::Curator => &self.curator_model,
        }
    }
}

/// Stage result from TypeScript ConsensusStageResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    #[serde(rename = "stageId")]
    pub stage_id: String,
    #[serde(rename = "stageName")]
    pub stage_name: String,
    pub question: String,
    pub answer: String,
    pub model: String,
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    pub timestamp: DateTime<Utc>,
    pub usage: Option<TokenUsage>,
    pub analytics: Option<StageAnalytics>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Stage analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageAnalytics {
    pub duration: f64,
    pub cost: f64,
    pub provider: String,
    #[serde(rename = "modelInternalId")]
    pub model_internal_id: String,
    #[serde(rename = "qualityScore")]
    pub quality_score: f64,
    #[serde(rename = "errorCount")]
    pub error_count: u32,
    #[serde(rename = "fallbackUsed")]
    pub fallback_used: bool,
    #[serde(rename = "rateLimitHit")]
    pub rate_limit_hit: bool,
    #[serde(rename = "retryCount")]
    pub retry_count: u32,
    #[serde(rename = "startTime")]
    pub start_time: DateTime<Utc>,
    #[serde(rename = "endTime")]
    pub end_time: DateTime<Utc>,
    #[serde(rename = "memoryUsage")]
    pub memory_usage: Option<u64>,
    pub features: AnalyticsFeatures,
}

/// Analytics feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsFeatures {
    pub streaming: bool,
    pub routing_variant: String,
    pub optimization_applied: Option<bool>,
}

/// Final consensus result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub stages: Vec<StageResult>,
    pub conversation_id: String,
    pub total_duration: f64,
    pub total_cost: f64,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub profile: ConsensusProfile,
    pub enable_streaming: bool,
    pub show_progress: bool,
    pub timeout_seconds: u64,
    pub retry_policy: RetryPolicy,
    pub context_injection: ContextInjectionStrategy,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_base: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            exponential_base: 2.0,
        }
    }
}

/// Context injection strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextInjectionStrategy {
    /// Always inject context
    Always,
    /// Only inject when temporal context is detected
    Temporal,
    /// Only inject when semantic context is available
    Semantic,
    /// Inject both temporal and semantic when available
    Smart,
    /// Never inject context
    Never,
}

impl Default for ContextInjectionStrategy {
    fn default() -> Self {
        Self::Smart
    }
}

/// Messages for OpenRouter API (matches TypeScript)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Stage-specific prompts
pub struct StagePrompts;

impl StagePrompts {
    /// System prompt for Generator stage
    pub fn generator_system() -> &'static str {
        "You are the GENERATOR in a 4-stage consensus pipeline. Your role is to provide an initial, comprehensive response to the user's question. Be thorough, accurate, and consider multiple perspectives. Your output will be refined by subsequent stages."
    }

    /// System prompt for Refiner stage
    pub fn refiner_system() -> &'static str {
        "You are the REFINER in a 4-stage consensus pipeline. Your role is to improve upon the GENERATOR's response. Enhance clarity, fix any errors, add missing important information, and ensure the response is well-structured. Build upon the existing answer rather than starting from scratch."
    }

    /// System prompt for Validator stage
    pub fn validator_system() -> &'static str {
        "You are the VALIDATOR in a 4-stage consensus pipeline. Your role is to fact-check and verify the accuracy of the refined response. Identify any errors, inconsistencies, or misleading information. Provide corrections and ensure the response is truthful and reliable."
    }

    /// System prompt for Curator stage
    pub fn curator_system() -> &'static str {
        "You are the CURATOR in a 4-stage consensus pipeline. Your role is to create the final, polished response. Synthesize the insights from all previous stages, ensure perfect formatting, maintain a consistent tone, and deliver a response that directly addresses the user's question in the most helpful way possible."
    }
}

/// Consensus request for TUI integration
#[derive(Debug, Clone)]
pub struct ConsensusRequest {
    pub query: String,
    pub context: Option<String>,
    pub temporal_context: Option<TemporalContext>,
    pub streaming: bool,
    pub profile: String,
}

/// Consensus response for TUI integration
#[derive(Debug, Clone)]
pub struct ConsensusResponse {
    pub content: String,
    pub metadata: ResponseMetadata,
    pub profile: Option<String>,
    pub latency: Duration,
    pub model_usage: HashMap<String, u32>,
    pub quality_score: f64,
}

/// Response metadata
#[derive(Debug, Clone)]
pub struct ResponseMetadata {
    pub total_tokens: u32,
    pub cost: f64,
    pub duration_ms: u64,
    pub models_used: Vec<String>,
}