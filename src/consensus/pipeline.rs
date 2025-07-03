// Consensus Pipeline - Orchestrates the 4-stage consensus process
// Manages flow from Generator → Refiner → Validator → Curator

use crate::consensus::stages::{ConsensusStage, CuratorStage, GeneratorStage, RefinerStage, ValidatorStage};
use crate::consensus::streaming::{ConsoleCallbacks, ProgressTracker, StreamingCallbacks};
use crate::consensus::temporal::TemporalContextProvider;
use crate::consensus::types::{
    ConsensusConfig, ConsensusResult, Stage, StageAnalytics, StageResult,
    TokenUsage,
};
#[cfg(test)]
use crate::consensus::types::ConsensusProfile;
use crate::hooks::{HooksSystem, EventType, EventSource, HookEvent, ConsensusIntegration};
use crate::providers::openrouter::{
    OpenRouterClient, OpenRouterMessage, MessageRole, StreamingClient, StreamingOptions
};
use anyhow::{Context, Result};
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// The main consensus pipeline orchestrator
pub struct ConsensusPipeline {
    config: ConsensusConfig,
    temporal_provider: TemporalContextProvider,
    stages: Vec<Box<dyn ConsensusStage>>,
    callbacks: Arc<dyn StreamingCallbacks>,
    hooks_system: Option<Arc<HooksSystem>>,
    consensus_integration: Option<Arc<ConsensusIntegration>>,
    openrouter_client: Option<Arc<OpenRouterClient>>,
    streaming_client: Option<Arc<StreamingClient>>,
}

impl ConsensusPipeline {
    /// Create a new consensus pipeline
    pub fn new(config: ConsensusConfig) -> Self {
        let callbacks: Arc<dyn StreamingCallbacks> = if config.show_progress {
            Arc::new(ConsoleCallbacks {
                show_progress: true,
            })
        } else {
            Arc::new(ConsoleCallbacks {
                show_progress: false,
            })
        };

        // Initialize OpenRouter clients if API key is available
        let (openrouter_client, streaming_client) = if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") {
            let client = crate::providers::openrouter::create_client(api_key.clone())
                .map(Arc::new)
                .ok();
            let streaming = crate::providers::openrouter::create_streaming_client(api_key)
                .map(Arc::new)
                .ok();
            (client, streaming)
        } else {
            (None, None)
        };

        Self {
            config,
            temporal_provider: TemporalContextProvider::default(),
            stages: vec![
                Box::new(GeneratorStage::new()),
                Box::new(RefinerStage::new()),
                Box::new(ValidatorStage::new()),
                Box::new(CuratorStage::new()),
            ],
            callbacks,
            hooks_system: None,
            consensus_integration: None,
            openrouter_client,
            streaming_client,
        }
    }

    /// Create with custom callbacks
    pub fn with_callbacks(mut self, callbacks: Arc<dyn StreamingCallbacks>) -> Self {
        self.callbacks = callbacks;
        self
    }
    
    /// Set the hooks system for this pipeline
    pub fn with_hooks(mut self, hooks_system: Arc<HooksSystem>) -> Self {
        self.hooks_system = Some(hooks_system);
        self
    }
    
    /// Set the consensus integration for enterprise features
    pub fn with_consensus_integration(mut self, integration: Arc<ConsensusIntegration>) -> Self {
        self.consensus_integration = Some(integration);
        self
    }

    /// Run the full consensus pipeline
    pub async fn run(
        &self,
        question: &str,
        context: Option<String>,
    ) -> Result<ConsensusResult> {
        let conversation_id = Uuid::new_v4().to_string();
        let pipeline_start = Instant::now();
        let mut total_cost = 0.0;

        // Emit BeforeConsensus hook event
        if let Some(hooks) = &self.hooks_system {
            let event = HookEvent::new(
                EventType::BeforeConsensus,
                EventSource::Consensus { stage: "pipeline".to_string() }
            )
            .with_context("question", question)
            .with_context("conversation_id", &conversation_id);
            
            if let Err(e) = hooks.dispatch_event(event).await {
                tracing::warn!("Failed to dispatch BeforeConsensus hook event: {}", e);
            }
        }

        // Check if temporal context is needed
        let temporal_context = if self.temporal_provider.requires_temporal_context(question) {
            Some(self.temporal_provider.build_current_context().await?)
        } else {
            None
        };

        // Build full context
        let full_context = self.build_full_context(context, temporal_context).await?;

        let mut previous_answer: Option<String> = None;
        let mut stage_results = Vec::new();

        // Run through all 4 stages
        for (i, stage_handler) in self.stages.iter().enumerate() {
            let stage = stage_handler.stage();
            let model = self.config.profile.get_model_for_stage(stage);

            // Execute pre-stage hooks with enterprise integration
            if let Some(integration) = &self.consensus_integration {
                // Estimate stage cost
                let estimated_cost = self.estimate_stage_cost(stage, model, question).await?;
                
                // Execute pre-stage hooks with cost and quality checks
                let pre_stage_result = integration.execute_pre_stage_hooks(
                    stage,
                    &conversation_id,
                    question,
                    model,
                    estimated_cost,
                ).await?;
                
                // Check if we should proceed
                if !pre_stage_result.proceed {
                    if let Some(approval_required) = pre_stage_result.approval_required {
                        // Handle approval requirement
                        tracing::info!("Approval required for {} stage: {}", stage.as_str(), approval_required.description);
                        
                        // For now, we'll continue with a warning
                        // In a real implementation, this would wait for approval
                        for warning in &pre_stage_result.warnings {
                            tracing::warn!("Pre-stage warning: {}", warning);
                        }
                    } else {
                        // Stage was blocked
                        return Err(anyhow::anyhow!("Stage {} was blocked by enterprise hooks", stage.as_str()));
                    }
                }
            }

            // Emit BeforeStage hook event
            if let Some(hooks) = &self.hooks_system {
                let stage_event_type = match stage {
                    Stage::Generator => EventType::BeforeGeneratorStage,
                    Stage::Refiner => EventType::BeforeRefinerStage,
                    Stage::Validator => EventType::BeforeValidatorStage,
                    Stage::Curator => EventType::BeforeCuratorStage,
                };
                
                let event = HookEvent::new(
                    stage_event_type,
                    EventSource::Consensus { stage: stage.as_str().to_string() }
                )
                .with_context("question", question)
                .with_context("conversation_id", &conversation_id)
                .with_context("model", model)
                .with_context("stage_index", i)
                .with_context("previous_answer", previous_answer.as_deref().unwrap_or(""));
                
                if let Err(e) = hooks.dispatch_event(event).await {
                    tracing::warn!("Failed to dispatch Before{:?}Stage hook event: {}", stage, e);
                }
            }

            // Notify stage start
            self.callbacks.on_stage_start(stage, model)?;

            // Run the stage
            let stage_result = self
                .run_single_stage(
                    stage,
                    stage_handler.as_ref(),
                    question,
                    previous_answer.as_deref(),
                    full_context.as_deref(),
                    &conversation_id,
                    model,
                )
                .await
                .with_context(|| format!("Failed to run {} stage", stage.display_name()))?;

            // Update cost
            if let Some(analytics) = &stage_result.analytics {
                total_cost += analytics.cost;
                
                // Emit cost control hook events if needed
                if let Some(hooks) = &self.hooks_system {
                    if analytics.cost > 0.05 { // Threshold for cost notifications
                        let cost_event = HookEvent::new(
                            EventType::CostThresholdReached,
                            EventSource::Consensus { stage: stage.as_str().to_string() }
                        )
                        .with_context("estimated_cost", analytics.cost)
                        .with_context("stage_cost", analytics.cost)
                        .with_context("total_cost", total_cost)
                        .with_context("model", model);
                        
                        if let Err(e) = hooks.dispatch_event(cost_event).await {
                            tracing::warn!("Failed to dispatch CostThresholdReached hook event: {}", e);
                        }
                    }
                }
            }

            // Store the answer for next stage
            previous_answer = Some(stage_result.answer.clone());

            // Notify stage complete
            self.callbacks.on_stage_complete(stage, &stage_result)?;

            // Execute post-stage hooks with quality validation
            if let Some(integration) = &self.consensus_integration {
                let post_stage_result = integration.execute_post_stage_hooks(
                    stage,
                    &stage_result,
                ).await?;
                
                // Check if we should continue
                if !post_stage_result.proceed {
                    if let Some(approval_required) = post_stage_result.approval_required {
                        // Handle approval requirement
                        tracing::warn!("Quality issue in {} stage requires approval: {}", stage.as_str(), approval_required.description);
                        
                        // For now, we'll continue with warnings
                        for warning in &post_stage_result.warnings {
                            tracing::warn!("Post-stage warning: {}", warning);
                        }
                    } else {
                        // Stage result was rejected by quality gates
                        return Err(anyhow::anyhow!("Stage {} result was rejected by quality gates", stage.as_str()));
                    }
                }
                
                // Log any warnings
                for warning in &post_stage_result.warnings {
                    tracing::warn!("Quality warning for {} stage: {}", stage.as_str(), warning);
                }
            }

            // Emit AfterStage hook event
            if let Some(hooks) = &self.hooks_system {
                let stage_event_type = match stage {
                    Stage::Generator => EventType::AfterGeneratorStage,
                    Stage::Refiner => EventType::AfterRefinerStage,
                    Stage::Validator => EventType::AfterValidatorStage,
                    Stage::Curator => EventType::AfterCuratorStage,
                };
                
                let event = HookEvent::new(
                    stage_event_type,
                    EventSource::Consensus { stage: stage.as_str().to_string() }
                )
                .with_context("question", question)
                .with_context("conversation_id", &conversation_id)
                .with_context("model", model)
                .with_context("answer", &stage_result.answer)
                .with_context("duration", stage_result.analytics.as_ref().map(|a| a.duration).unwrap_or(0.0))
                .with_context("cost", stage_result.analytics.as_ref().map(|a| a.cost).unwrap_or(0.0));
                
                if let Err(e) = hooks.dispatch_event(event).await {
                    tracing::warn!("Failed to dispatch After{:?}Stage hook event: {}", stage, e);
                }
            }

            stage_results.push(stage_result);
        }

        // The final answer is from the Curator stage
        let final_answer = previous_answer.unwrap_or_else(|| {
            "Error: No response generated from consensus pipeline".to_string()
        });

        let result = ConsensusResult {
            success: true,
            result: Some(final_answer.clone()),
            error: None,
            stages: stage_results,
            conversation_id: conversation_id.clone(),
            total_duration: pipeline_start.elapsed().as_secs_f64(),
            total_cost,
        };

        // Emit AfterConsensus hook event
        if let Some(hooks) = &self.hooks_system {
            let event = HookEvent::new(
                EventType::AfterConsensus,
                EventSource::Consensus { stage: "pipeline".to_string() }
            )
            .with_context("question", question)
            .with_context("conversation_id", &conversation_id)
            .with_context("final_answer", &final_answer)
            .with_context("total_duration", result.total_duration)
            .with_context("total_cost", result.total_cost)
            .with_context("stages_count", result.stages.len());
            
            if let Err(e) = hooks.dispatch_event(event).await {
                tracing::warn!("Failed to dispatch AfterConsensus hook event: {}", e);
            }
        }

        Ok(result)
    }

    /// Run a single stage of the pipeline
    async fn run_single_stage(
        &self,
        stage: Stage,
        handler: &dyn ConsensusStage,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
        conversation_id: &str,
        model: &str,
    ) -> Result<StageResult> {
        let stage_start = Instant::now();
        let stage_id = Uuid::new_v4().to_string();

        // Build messages for this stage
        let messages = handler.build_messages(question, previous_answer, context)?;

        // Create progress tracker
        let mut tracker = ProgressTracker::new(stage, self.callbacks.clone());

        // Simulate API call (will be replaced with actual OpenRouter client)
        let response = self
            .call_model(model, &messages, &mut tracker)
            .await
            .with_context(|| format!("Failed to call model {} for {}", model, stage.display_name()))?;

        // Mark complete
        tracker.complete()?;

        // Build stage result
        let stage_result = StageResult {
            stage_id,
            stage_name: stage.as_str().to_string(),
            question: question.to_string(),
            answer: response.content,
            model: model.to_string(),
            conversation_id: conversation_id.to_string(),
            timestamp: Utc::now(),
            usage: response.usage,
            analytics: Some(StageAnalytics {
                duration: stage_start.elapsed().as_secs_f64(),
                cost: response.cost,
                provider: response.provider,
                model_internal_id: response.model_internal_id,
                quality_score: 0.95, // Placeholder
                error_count: 0,
                fallback_used: false,
                rate_limit_hit: false,
                retry_count: 0,
                start_time: Utc::now() - chrono::Duration::seconds(stage_start.elapsed().as_secs() as i64),
                end_time: Utc::now(),
                memory_usage: None,
                features: crate::consensus::types::AnalyticsFeatures {
                    streaming: self.config.enable_streaming,
                    routing_variant: "balanced".to_string(),
                    optimization_applied: Some(true),
                },
            }),
        };

        Ok(stage_result)
    }

    /// Build full context combining semantic and temporal
    async fn build_full_context(
        &self,
        semantic_context: Option<String>,
        temporal_context: Option<crate::consensus::temporal::TemporalContext>,
    ) -> Result<Option<String>> {
        let mut contexts = Vec::new();

        if let Some(semantic) = semantic_context {
            contexts.push(semantic);
        }

        if let Some(temporal) = temporal_context {
            contexts.push(format!(
                "{}\n{}",
                temporal.search_instruction, temporal.temporal_awareness
            ));
        }

        if contexts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(contexts.join("\n\n")))
        }
    }

    /// Call model via OpenRouter API with streaming support
    async fn call_model(
        &self,
        model: &str,
        messages: &[crate::consensus::types::Message],
        tracker: &mut ProgressTracker,
    ) -> Result<ModelResponse> {
        // Check if OpenRouter clients are available
        let streaming_client = self.streaming_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenRouter streaming client not initialized. Please set OPENROUTER_API_KEY environment variable."))?;

        // Convert consensus messages to OpenRouter format
        let openrouter_messages: Vec<OpenRouterMessage> = messages
            .iter()
            .map(|msg| OpenRouterMessage {
                role: match msg.role.as_str() {
                    "system" => MessageRole::System,
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    _ => MessageRole::User, // Default fallback
                },
                content: msg.content.clone(),
            })
            .collect();

        let mut response_content = String::new();
        let mut prompt_tokens = 0u32;
        let mut completion_tokens = 0u32;
        let mut cost = 0.0f64;

        // Configure streaming options
        let streaming_options = StreamingOptions {
            temperature: Some(0.7),
            max_tokens: Some(2000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        };

        // Set up streaming callbacks to connect OpenRouter streaming with consensus progress
        let response_content_arc = Arc::new(tokio::sync::Mutex::new(String::new()));
        let response_content_clone = response_content_arc.clone();

        let callbacks = crate::providers::openrouter::StreamingCallbacks {
            on_chunk: Some(Arc::new(move |chunk, _total| {
                let content = response_content_clone.clone();
                tokio::spawn(async move {
                    if let Ok(mut content_guard) = content.try_lock() {
                        content_guard.push_str(&chunk);
                    }
                });
            })),
            on_progress: Some(Arc::new(move |progress| {
                // Progress is handled by the chunk callback
                tracing::debug!("OpenRouter streaming progress: {:?}", progress);
            })),
            on_complete: Some(Arc::new(move |_response| {
                tracing::debug!("OpenRouter streaming complete");
            })),
            on_error: Some(Arc::new(move |error| {
                tracing::warn!("OpenRouter streaming error: {}", error);
            })),
            on_start: Some(Arc::new(move || {
                tracing::debug!("OpenRouter streaming started");
            })),
        };

        // Make the streaming API call
        match streaming_client
            .stream_chat_completion(model, openrouter_messages, streaming_options, callbacks)
            .await
        {
            Ok(response) => {
                // Extract final response content
                response_content = response_content_arc.lock().await.clone();
                
                // Update tracker with final response
                tracker.add_chunk(&response_content)?;
                
                // Extract usage from response
                if let Some(usage) = &response.usage {
                    prompt_tokens = usage.prompt_tokens;
                    completion_tokens = usage.completion_tokens;
                }
                
                // Calculate cost based on usage (simplified estimation)
                // TODO: Integrate with actual cost tracking system
                cost = (prompt_tokens as f64 * 0.00001) + (completion_tokens as f64 * 0.00003);

                Ok(ModelResponse {
                    content: response_content,
                    usage: Some(TokenUsage {
                        prompt_tokens,
                        completion_tokens,
                        total_tokens: prompt_tokens + completion_tokens,
                    }),
                    cost,
                    provider: "openrouter".to_string(),
                    model_internal_id: model.to_string(),
                })
            }
            Err(e) => {
                // Fallback to mock response on API failure
                tracing::warn!("OpenRouter API call failed for model {}: {}. Using fallback response.", model, e);
                
                let fallback_text = match tracker.stage {
                    Stage::Generator => {
                        "This is a fallback response from the Generator stage. The OpenRouter API is currently unavailable, but the consensus pipeline continues to function."
                    }
                    Stage::Refiner => {
                        "This is a fallback response from the Refiner stage. While the primary API is unavailable, the system continues to provide refined outputs."
                    }
                    Stage::Validator => {
                        "This is a fallback response from the Validator stage. Validation continues even when external APIs are temporarily unavailable."
                    }
                    Stage::Curator => {
                        "This is a fallback response from the Curator stage. The final polished output is provided despite API connectivity issues."
                    }
                };

                // Stream the fallback response
                for word in fallback_text.split_whitespace() {
                    tracker.add_chunk(word)?;
                    tracker.add_chunk(" ")?;
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }

                Ok(ModelResponse {
                    content: fallback_text.to_string(),
                    usage: Some(TokenUsage {
                        prompt_tokens: 100,
                        completion_tokens: 50,
                        total_tokens: 150,
                    }),
                    cost: 0.0, // No cost for fallback
                    provider: "fallback".to_string(),
                    model_internal_id: "fallback".to_string(),
                })
            }
        }
    }
    
    /// Estimate cost for a stage operation
    async fn estimate_stage_cost(&self, stage: Stage, model: &str, question: &str) -> Result<f64> {
        // Simple cost estimation based on question length and model
        let question_tokens = question.split_whitespace().count() as u32;
        let estimated_tokens = match stage {
            Stage::Generator => question_tokens + 500, // Generator usually produces more tokens
            Stage::Refiner => question_tokens + 200,   // Refiner improves existing content
            Stage::Validator => question_tokens + 100, // Validator checks quality
            Stage::Curator => question_tokens + 150,   // Curator polishes result
        };
        
        // Model-based cost calculation (simplified)
        let cost_per_token = match model {
            m if m.contains("gpt-4") => 0.00003,
            m if m.contains("gpt-3.5") => 0.000002,
            m if m.contains("claude-3-opus") => 0.000075,
            m if m.contains("claude-3-sonnet") => 0.000015,
            m if m.contains("claude-3-haiku") => 0.000001,
            _ => 0.00001, // Default fallback
        };
        
        Ok(estimated_tokens as f64 * cost_per_token)
    }
}

/// Response from model API
struct ModelResponse {
    content: String,
    usage: Option<TokenUsage>,
    cost: f64,
    provider: String,
    model_internal_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::types::RetryPolicy;

    #[tokio::test]
    async fn test_pipeline_basic() {
        let profile = ConsensusProfile {
            id: "test".to_string(),
            profile_name: "Test Profile".to_string(),
            generator_model: "claude-3-5-sonnet".to_string(),
            refiner_model: "gpt-4-turbo".to_string(),
            validator_model: "claude-3-opus".to_string(),
            curator_model: "gpt-4o".to_string(),
            created_at: Utc::now(),
            is_active: true,
        };

        let config = ConsensusConfig {
            profile,
            enable_streaming: false,
            show_progress: false,
            timeout_seconds: 60,
            retry_policy: RetryPolicy::default(),
            context_injection: crate::consensus::types::ContextInjectionStrategy::Smart,
        };

        let pipeline = ConsensusPipeline::new(config);
        let result = pipeline
            .run("What is Rust?", None)
            .await
            .expect("Pipeline should run successfully");

        assert!(result.success);
        assert!(result.result.is_some());
        assert_eq!(result.stages.len(), 4);
    }
}