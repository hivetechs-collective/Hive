// Consensus Engine - Main entry point for consensus functionality
// Manages profiles, configuration, and pipeline execution

use crate::consensus::pipeline::ConsensusPipeline;
use crate::consensus::streaming::{StreamingCallbacks, StreamingResponse, ConsensusStage, ConsensusResponseResult, ResponseMetadata};
use crate::consensus::types::{
    ConsensusConfig, ConsensusProfile, ConsensusResult, ConsensusRequest,
    ContextInjectionStrategy, RetryPolicy,
};
// use crate::core::Database; // TODO: Replace with actual database implementation
use anyhow::{Context, Result};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::time::Instant;

/// Main consensus engine that manages the consensus pipeline
pub struct ConsensusEngine {
    // db: Arc<Database>, // TODO: Replace with actual database implementation
    current_profile: Arc<RwLock<ConsensusProfile>>,
    config: Arc<RwLock<ConsensusConfig>>,
}

impl ConsensusEngine {
    /// Create a new consensus engine
    pub async fn new(/* db: Arc<Database> */) -> Result<Self> {
        // Load or create default profile
        let profile = Self::create_balanced_profile(); // TODO: Load from database

        let config = ConsensusConfig {
            profile: profile.clone(),
            enable_streaming: true,
            show_progress: true,
            timeout_seconds: 120,
            retry_policy: RetryPolicy::default(),
            context_injection: ContextInjectionStrategy::default(),
        };

        Ok(Self {
            // db,
            current_profile: Arc::new(RwLock::new(profile)),
            config: Arc::new(RwLock::new(config)),
        })
    }

    /// Process a query through the consensus pipeline
    pub async fn process(
        &self,
        query: &str,
        semantic_context: Option<String>,
    ) -> Result<ConsensusResult> {
        let config = self.config.read().await.clone();
        
        let pipeline = ConsensusPipeline::new(config);
        
        pipeline
            .run(query, semantic_context)
            .await
            .context("Failed to run consensus pipeline")
    }

    /// Process with custom callbacks
    pub async fn process_with_callbacks(
        &self,
        query: &str,
        semantic_context: Option<String>,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<ConsensusResult> {
        let config = self.config.read().await.clone();
        
        let pipeline = ConsensusPipeline::new(config).with_callbacks(callbacks);
        
        pipeline
            .run(query, semantic_context)
            .await
            .context("Failed to run consensus pipeline with callbacks")
    }

    /// Process with streaming responses for TUI integration
    pub async fn process_with_streaming(
        &self,
        request: ConsensusRequest,
    ) -> Result<mpsc::UnboundedReceiver<StreamingResponse>> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let start_time = Instant::now();
        
        // Clone necessary data for the async task
        let query = request.query.clone();
        let context = request.context.clone();
        let temporal_context = request.temporal_context.clone();
        let config = self.config.read().await.clone();
        
        // Spawn async task to process consensus
        tokio::spawn(async move {
            let models = vec![
                ("claude-3-5-sonnet".to_string(), ConsensusStage::Generator),
                ("gpt-4-turbo".to_string(), ConsensusStage::Refiner),
                ("claude-3-opus".to_string(), ConsensusStage::Validator),
                ("gpt-4o".to_string(), ConsensusStage::Curator),
            ];
            
            let mut final_content = String::new();
            let mut total_tokens = 0u32;
            let mut total_cost = 0.0f64;
            let mut models_used = Vec::new();
            
            // Process each stage
            for (model, stage) in models {
                // Send stage started event
                let _ = sender.send(StreamingResponse::StageStarted {
                    stage,
                    model: model.clone(),
                });
                
                models_used.push(model.clone());
                
                // Simulate processing with progress updates
                for progress in (0..=100).step_by(10) {
                    let _ = sender.send(StreamingResponse::StageProgress { stage, progress });
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
                
                // Simulate stage completion
                let _ = sender.send(StreamingResponse::StageCompleted { stage });
                
                // Update metrics
                total_tokens += 150; // Placeholder
                total_cost += 0.005; // Placeholder
            }
            
            // Build response based on query and context
            final_content = if let Some(temporal) = temporal_context {
                format!(
                    "Based on the 4-stage consensus analysis and current temporal context ({}):\n\n{}\n\n(This is a development placeholder. In production, this would be the actual consensus result from the OpenRouter models.)",
                    temporal.current_datetime,
                    query
                )
            } else {
                format!(
                    "Based on the 4-stage consensus analysis:\n\n{}\n\n(This is a development placeholder. In production, this would be the actual consensus result from the OpenRouter models.)",
                    query
                )
            };
            
            // Send completion event
            let response = ConsensusResponseResult {
                content: final_content,
                metadata: ResponseMetadata {
                    total_tokens,
                    cost: total_cost,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    models_used,
                },
            };
            
            let _ = sender.send(StreamingResponse::Complete { response });
        });
        
        Ok(receiver)
    }

    /// Get available profiles
    pub async fn get_profiles(&self) -> Result<Vec<ConsensusProfile>> {
        // TODO: Implement database query
        Ok(vec![
            self.get_balanced_profile(),
            self.get_speed_profile(),
            self.get_quality_profile(),
            self.get_cost_profile(),
        ])
    }

    /// Set active profile
    pub async fn set_profile(&self, profile_name: &str) -> Result<()> {
        let profile = match profile_name {
            "balanced" => self.get_balanced_profile(),
            "speed" => self.get_speed_profile(),
            "quality" => self.get_quality_profile(),
            "cost" => self.get_cost_profile(),
            _ => anyhow::bail!("Unknown profile: {}", profile_name),
        };

        *self.current_profile.write().await = profile.clone();
        self.config.write().await.profile = profile;

        Ok(())
    }

    /// Get current profile
    pub async fn get_current_profile(&self) -> ConsensusProfile {
        self.current_profile.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut ConsensusConfig),
    {
        let mut config = self.config.write().await;
        updater(&mut config);
        Ok(())
    }

    // /// Load or create default profile
    // async fn load_or_create_default_profile(db: &Arc<Database>) -> Result<ConsensusProfile> {
    //     // TODO: Implement actual database loading
    //     // For now, return balanced profile
    //     Ok(Self::create_balanced_profile())
    // }

    /// Create balanced profile (default)
    fn create_balanced_profile() -> ConsensusProfile {
        ConsensusProfile {
            id: "balanced".to_string(),
            profile_name: "Consensus_Balanced".to_string(),
            generator_model: "anthropic/claude-3-5-sonnet".to_string(),
            refiner_model: "openai/gpt-4-turbo".to_string(),
            validator_model: "anthropic/claude-3-opus".to_string(),
            curator_model: "openai/gpt-4o".to_string(),
            created_at: Utc::now(),
            is_active: true,
        }
    }

    /// Get balanced profile
    fn get_balanced_profile(&self) -> ConsensusProfile {
        Self::create_balanced_profile()
    }

    /// Get speed-optimized profile
    fn get_speed_profile(&self) -> ConsensusProfile {
        ConsensusProfile {
            id: "speed".to_string(),
            profile_name: "Consensus_Speed".to_string(),
            generator_model: "anthropic/claude-3-haiku".to_string(),
            refiner_model: "openai/gpt-3.5-turbo".to_string(),
            validator_model: "anthropic/claude-3-haiku".to_string(),
            curator_model: "openai/gpt-3.5-turbo".to_string(),
            created_at: Utc::now(),
            is_active: false,
        }
    }

    /// Get quality-optimized profile
    fn get_quality_profile(&self) -> ConsensusProfile {
        ConsensusProfile {
            id: "quality".to_string(),
            profile_name: "Consensus_Elite".to_string(),
            generator_model: "anthropic/claude-3-opus".to_string(),
            refiner_model: "openai/gpt-4o".to_string(),
            validator_model: "anthropic/claude-3-opus".to_string(),
            curator_model: "openai/gpt-4o".to_string(),
            created_at: Utc::now(),
            is_active: false,
        }
    }

    /// Get cost-optimized profile
    fn get_cost_profile(&self) -> ConsensusProfile {
        ConsensusProfile {
            id: "cost".to_string(),
            profile_name: "Consensus_Cost".to_string(),
            generator_model: "meta-llama/llama-3.2-3b-instruct".to_string(),
            refiner_model: "mistralai/mistral-7b-instruct".to_string(),
            validator_model: "meta-llama/llama-3.2-3b-instruct".to_string(),
            curator_model: "mistralai/mistral-7b-instruct".to_string(),
            created_at: Utc::now(),
            is_active: false,
        }
    }

    /// Validate consensus prerequisites
    pub async fn validate_prerequisites(&self) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        // Check OpenRouter API key
        if std::env::var("OPENROUTER_API_KEY").is_err() {
            errors.push("OpenRouter API key not configured".to_string());
        }

        // Check database connection
        // TODO: Implement actual database check

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
        })
    }

    /// Clone the engine for concurrent use
    pub fn clone(&self) -> Self {
        Self {
            current_profile: self.current_profile.clone(),
            config: self.config.clone(),
        }
    }
}

/// Validation result for prerequisites
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Database;

    #[tokio::test]
    async fn test_engine_creation() {
        let db = Arc::new(Database::memory().await.unwrap());
        let engine = ConsensusEngine::new(db).await.unwrap();

        let profile = engine.get_current_profile().await;
        assert_eq!(profile.profile_name, "Consensus_Balanced");
    }

    #[tokio::test]
    async fn test_profile_switching() {
        let db = Arc::new(Database::memory().await.unwrap());
        let engine = ConsensusEngine::new(db).await.unwrap();

        engine.set_profile("speed").await.unwrap();
        let profile = engine.get_current_profile().await;
        assert_eq!(profile.profile_name, "Consensus_Speed");

        engine.set_profile("quality").await.unwrap();
        let profile = engine.get_current_profile().await;
        assert_eq!(profile.profile_name, "Consensus_Elite");
    }
}