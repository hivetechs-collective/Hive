// Consensus Pipeline - Orchestrates the 4-stage consensus process
// Manages flow from Generator → Refiner → Validator → Curator

use crate::consensus::stages::{ConsensusStage, CuratorStage, GeneratorStage, RefinerStage, ValidatorStage};
use crate::consensus::streaming::{ConsoleCallbacks, ProgressTracker, StreamingCallbacks};
use crate::consensus::temporal::TemporalContextProvider;
use crate::consensus::types::{
    ConsensusConfig, ConsensusProfile, ConsensusResult, Stage, StageAnalytics, StageResult,
    TokenUsage,
};
#[cfg(test)]
use crate::consensus::types::ConsensusProfile;
// use crate::hooks::{HooksSystem, EventType, EventSource, HookEvent, ConsensusIntegration};
use crate::consensus::openrouter::{
    OpenRouterClient, OpenRouterMessage, OpenRouterRequest, OpenRouterResponse,
    StreamingCallbacks as OpenRouterStreamingCallbacks, SimpleStreamingCallbacks
};
use crate::consensus::models::{ModelManager, DynamicModelSelector, ModelSelectionCriteria, BudgetConstraints, PerformanceTargets, UserPreferences};
use crate::core::database_simple::Database;
use anyhow::{Context, Result};
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// The main consensus pipeline orchestrator
pub struct ConsensusPipeline {
    config: ConsensusConfig,
    profile: ConsensusProfile,
    temporal_provider: TemporalContextProvider,
    stages: Vec<Box<dyn ConsensusStage>>,
    callbacks: Arc<dyn StreamingCallbacks>,
    // hooks_system: Option<Arc<HooksSystem>>,
    // consensus_integration: Option<Arc<ConsensusIntegration>>,
    openrouter_client: Option<Arc<OpenRouterClient>>,
    model_manager: Option<Arc<ModelManager>>,
    model_selector: Option<Arc<DynamicModelSelector>>,
    database: Option<Arc<Database>>,
    api_key: Option<String>,
}

impl ConsensusPipeline {
    /// Create a new consensus pipeline
    pub fn new(config: ConsensusConfig, profile: ConsensusProfile, api_key: Option<String>) -> Self {
        let callbacks: Arc<dyn StreamingCallbacks> = if config.show_progress {
            Arc::new(ConsoleCallbacks {
                show_progress: true,
            })
        } else {
            Arc::new(ConsoleCallbacks {
                show_progress: false,
            })
        };

        // Initialize OpenRouter client and model management if API key is provided
        let (openrouter_client, model_manager, model_selector) = if let Some(ref key) = api_key {
            let client = Arc::new(OpenRouterClient::new(key.clone()));
            let manager = Arc::new(ModelManager::new(Some(key.clone())));
            let selector = Arc::new(DynamicModelSelector::new(Some(key.clone())));
            (Some(client), Some(manager), Some(selector))
        } else {
            (None, None, None)
        };

        Self {
            config,
            profile,
            temporal_provider: TemporalContextProvider::default(),
            stages: vec![
                Box::new(GeneratorStage::new()),
                Box::new(RefinerStage::new()),
                Box::new(ValidatorStage::new()),
                Box::new(CuratorStage::new()),
            ],
            callbacks,
            // hooks_system: None,
            // consensus_integration: None,
            openrouter_client,
            model_manager,
            model_selector,
            database: None, // Will be set later when needed
            api_key,
        }
    }

    /// Create with custom callbacks
    pub fn with_callbacks(mut self, callbacks: Arc<dyn StreamingCallbacks>) -> Self {
        self.callbacks = callbacks;
        self
    }

    /// Set the database for model management
    pub fn with_database(mut self, database: Arc<Database>) -> Self {
        self.database = Some(database);
        self
    }
    
    /// Set the hooks system for this pipeline
    // pub fn with_hooks(mut self, hooks_system: Arc<HooksSystem>) -> Self {
    //     self.hooks_system = Some(hooks_system);
    //     self
    // }
    
    /// Set the consensus integration for enterprise features
    // pub fn with_consensus_integration(mut self, integration: Arc<ConsensusIntegration>) -> Self {
    //     self.consensus_integration = Some(integration);
    //     self
    // }

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
//         // if let Some(hooks) = &self.hooks_system {
//         //     let event = HookEvent::new(
//         //         EventType::BeforeConsensus,
//                 EventSource::Consensus { stage: "pipeline".to_string() }
//             )
//             .with_context("question", question)
//             .with_context("conversation_id", &conversation_id);
//             
//             if let Err(e) = hooks.dispatch_event(event).await {
//                 tracing::warn!("Failed to dispatch BeforeConsensus hook event: {}", e);
//             }
//         }

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
            
            // Use dynamic model selection if available, otherwise fallback to profile
            let model = if let (Some(selector), Some(db)) = (&self.model_selector, &self.database) {
                // Create selection criteria
                let criteria = ModelSelectionCriteria {
                    stage: stage.as_str().to_string(),
                    question_complexity: if question.len() > 500 { "production".to_string() } else { "basic".to_string() },
                    question_category: "general".to_string(), // TODO: Improve categorization
                    budget_constraints: Some(BudgetConstraints {
                        max_cost_per_stage: Some(0.10),
                        max_total_cost: Some(0.50),
                        cost_efficiency_target: None,
                        prioritize_cost: false,
                    }),
                    performance_targets: Some(PerformanceTargets {
                        max_latency: Some(5000),
                        min_success_rate: Some(0.95),
                        prioritize_speed: false,
                        prioritize_quality: true,
                    }),
                    user_preferences: None,
                    profile_template: Some(self.profile.profile_name.clone()),
                };

                // Select optimal model
                match selector.select_optimal_model(db, &criteria, Some(&conversation_id)).await {
                    Ok(Some(candidate)) => candidate.openrouter_id,
                    Ok(None) => {
                        println!("⚠️ No optimal model found, using profile default");
                        self.profile.get_model_for_stage(stage).to_string()
                    }
                    Err(e) => {
                        println!("⚠️ Model selection failed: {}, using profile default", e);
                        self.profile.get_model_for_stage(stage).to_string()
                    }
                }
            } else {
                self.profile.get_model_for_stage(stage).to_string()
            };

            // Execute pre-stage hooks with enterprise integration
            // if let Some(integration) = &self.consensus_integration {
            //     // Estimate stage cost
            //     let estimated_cost = self.estimate_stage_cost(stage, model, question).await?;
            //     
            //     // Execute pre-stage hooks with cost and quality checks
            //     let pre_stage_result = integration.execute_pre_stage_hooks(
            //         stage,
            //         &conversation_id,
            //         question,
            //         model,
            //         estimated_cost,
            //     ).await?;
            //     
            //     // Check if we should proceed
            //     if !pre_stage_result.proceed {
            //         if let Some(approval_required) = pre_stage_result.approval_required {
            //             // Handle approval requirement
            //             tracing::info!("Approval required for {} stage: {}", stage.as_str(), approval_required.description);
            //             
            //             // For now, we'll continue with a warning
            //             // In a real implementation, this would wait for approval
            //             for warning in &pre_stage_result.warnings {
            //                 tracing::warn!("Pre-stage warning: {}", warning);
            //             }
            //         } else {
            //             // Stage was blocked
            //             return Err(anyhow::anyhow!("Stage {} was blocked by enterprise hooks", stage.as_str()));
            //         }
            //     }
            // }

            // Emit BeforeStage hook event
//             if let Some(hooks) = &self.hooks_system {
//                 let stage_event_type = match stage {
//                     Stage::Generator => EventType::BeforeGeneratorStage,
//                     Stage::Refiner => EventType::BeforeRefinerStage,
//                     Stage::Validator => EventType::BeforeValidatorStage,
//                     Stage::Curator => EventType::BeforeCuratorStage,
//                 };
//                 
//                 let event = HookEvent::new(
//                     stage_event_type,
//                     EventSource::Consensus { stage: stage.as_str().to_string() }
//                 )
//                 .with_context("question", question)
//                 .with_context("conversation_id", &conversation_id)
//                 .with_context("model", model)
//                 .with_context("stage_index", i)
//                 .with_context("previous_answer", previous_answer.as_deref().unwrap_or(""));
//                 
//                 if let Err(e) = hooks.dispatch_event(event).await {
//                     tracing::warn!("Failed to dispatch Before{:?}Stage hook event: {}", stage, e);
//                 }
//             }

            // Notify stage start
            self.callbacks.on_stage_start(stage, &model)?;

            // Run the stage
            let stage_result = self
                .run_single_stage(
                    stage,
                    stage_handler.as_ref(),
                    question,
                    previous_answer.as_deref(),
                    full_context.as_deref(),
                    &conversation_id,
                    &model,
                )
                .await
                .with_context(|| format!("Failed to run {} stage", stage.display_name()))?;

            // Update cost
            if let Some(analytics) = &stage_result.analytics {
                total_cost += analytics.cost;
                
                // Emit cost control hook events if needed
//                 if let Some(hooks) = &self.hooks_system {
//                     if analytics.cost > 0.05 { // Threshold for cost notifications
//                         let cost_event = HookEvent::new(
//                             EventType::CostThresholdReached,
//                             EventSource::Consensus { stage: stage.as_str().to_string() }
//                         )
//                         .with_context("estimated_cost", analytics.cost)
//                         .with_context("stage_cost", analytics.cost)
//                         .with_context("total_cost", total_cost)
//                         .with_context("model", model);
//                         
//                         if let Err(e) = hooks.dispatch_event(cost_event).await {
//                             tracing::warn!("Failed to dispatch CostThresholdReached hook event: {}", e);
//                         }
//                     }
//                 }
            }

            // Store the answer for next stage
            previous_answer = Some(stage_result.answer.clone());

            // Notify stage complete
            self.callbacks.on_stage_complete(stage, &stage_result)?;

            // Execute post-stage hooks with quality validation
            // if let Some(integration) = &self.consensus_integration {
            //     let post_stage_result = integration.execute_post_stage_hooks(
            //         stage,
            //         &stage_result,
            //     ).await?;
            //     
            //     // Check if we should continue
            //     if !post_stage_result.proceed {
            //         if let Some(approval_required) = post_stage_result.approval_required {
            //             // Handle approval requirement
            //             tracing::warn!("Quality issue in {} stage requires approval: {}", stage.as_str(), approval_required.description);
            //             
            //             // For now, we'll continue with warnings
            //             for warning in &post_stage_result.warnings {
            //                 tracing::warn!("Post-stage warning: {}", warning);
            //             }
            //         } else {
            //             // Stage result was rejected by quality gates
            //             return Err(anyhow::anyhow!("Stage {} result was rejected by quality gates", stage.as_str()));
            //         }
            //     }
            //     
            //     // Log any warnings
            //     for warning in &post_stage_result.warnings {
            //         tracing::warn!("Quality warning for {} stage: {}", stage.as_str(), warning);
            //     }
            // }

            // Emit AfterStage hook event
//             if let Some(hooks) = &self.hooks_system {
//                 let stage_event_type = match stage {
//                     Stage::Generator => EventType::AfterGeneratorStage,
//                     Stage::Refiner => EventType::AfterRefinerStage,
//                     Stage::Validator => EventType::AfterValidatorStage,
//                     Stage::Curator => EventType::AfterCuratorStage,
//                 };
//                 
//                 let event = HookEvent::new(
//                     stage_event_type,
//                     EventSource::Consensus { stage: stage.as_str().to_string() }
//                 )
//                 .with_context("question", question)
//                 .with_context("conversation_id", &conversation_id)
//                 .with_context("model", model)
//                 .with_context("answer", &stage_result.answer)
//                 .with_context("duration", stage_result.analytics.as_ref().map(|a| a.duration).unwrap_or(0.0))
//                 .with_context("cost", stage_result.analytics.as_ref().map(|a| a.cost).unwrap_or(0.0));
//                 
//                 if let Err(e) = hooks.dispatch_event(event).await {
//                     tracing::warn!("Failed to dispatch After{:?}Stage hook event: {}", stage, e);
//                 }
//             }

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
//         if let Some(hooks) = &self.hooks_system {
//             let event = HookEvent::new(
//                 EventType::AfterConsensus,
//                 EventSource::Consensus { stage: "pipeline".to_string() }
//             )
//             .with_context("question", question)
//             .with_context("conversation_id", &conversation_id)
//             .with_context("final_answer", &final_answer)
//             .with_context("total_duration", result.total_duration)
//             .with_context("total_cost", result.total_cost)
//             .with_context("stages_count", result.stages.len());
//             
//             if let Err(e) = hooks.dispatch_event(event).await {
//                 tracing::warn!("Failed to dispatch AfterConsensus hook event: {}", e);
//             }
//         }

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
                quality_score: 0.0, // Will be calculated by quality assessment
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
        // Check if OpenRouter client is available
        let openrouter_client = self.openrouter_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenRouter client not initialized. Please set OPENROUTER_API_KEY environment variable."))?;

        // Convert consensus messages to OpenRouter format
        let openrouter_messages: Vec<OpenRouterMessage> = messages
            .iter()
            .map(|msg| OpenRouterMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        // Configure request
        let request = OpenRouterRequest {
            model: model.to_string(),
            messages: openrouter_messages,
            temperature: Some(0.7),
            max_tokens: Some(4000), // Matching TypeScript hardcoded value
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: Some(self.config.enable_streaming),
            provider: None,
        };

        // Make the API call
        if self.config.enable_streaming {
            // Use streaming
            let callbacks = Box::new(SimpleStreamingCallbacks {
                model_name: model.to_string(),
            }) as Box<dyn OpenRouterStreamingCallbacks>;

            match openrouter_client.chat_completion_stream(request, Some(callbacks)).await {
                Ok(response_content) => {
                    // Update tracker with response chunks
                    for word in response_content.split_whitespace() {
                        tracker.add_chunk(word)?;
                        tracker.add_chunk(" ")?;
                        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
                    }

                    Ok(ModelResponse {
                        content: response_content,
                        usage: None, // Token usage will be populated by actual API response
                        cost: 0.0, // Cost will be calculated from actual API response
                        provider: "openrouter".to_string(),
                        model_internal_id: model.to_string(),
                    })
                }
                Err(e) => {
                    self.handle_api_error(e, tracker, model).await
                }
            }
        } else {
            // Use non-streaming
            match openrouter_client.chat_completion(request).await {
                Ok(response) => {
                    let content = response.choices.first()
                        .and_then(|choice| choice.message.as_ref())
                        .map(|message| message.content.clone())
                        .unwrap_or_else(|| "No response content".to_string());

                    // Add response to tracker
                    tracker.add_chunk(&content)?;

                    let usage = response.usage.map(|u| TokenUsage {
                        prompt_tokens: u.prompt_tokens,
                        completion_tokens: u.completion_tokens,
                        total_tokens: u.total_tokens,
                    });

                    // Calculate cost
                    let cost = if let Some(ref usage) = usage {
                        openrouter_client.estimate_cost(
                            model,
                            usage.prompt_tokens,
                            usage.completion_tokens,
                            Some(0.001), // Default input price
                            Some(0.002), // Default output price
                        )
                    } else {
                        0.0
                    };

                    Ok(ModelResponse {
                        content,
                        usage,
                        cost,
                        provider: "openrouter".to_string(),
                        model_internal_id: model.to_string(),
                    })
                }
                Err(e) => {
                    self.handle_api_error(e, tracker, model).await
                }
            }
        }
    }

    /// Handle API errors with fallback responses
    async fn handle_api_error(
        &self,
        error: anyhow::Error,
        tracker: &mut ProgressTracker,
        model: &str,
    ) -> Result<ModelResponse> {
        println!("⚠️ OpenRouter API call failed for model {}: {}. Using fallback response.", model, error);
        
        // When OpenRouter API is unavailable, return an error rather than placeholder content
        return Err(anyhow::anyhow!(
            "OpenRouter API call failed for model {}: {}. Cannot proceed without API access.",
            model, 
            error
        ));

        // This code should not be reached since we return an error above
        unreachable!("API error handling should return an error, not continue")
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
            enable_streaming: false,
            show_progress: false,
            timeout_seconds: 60,
            retry_policy: RetryPolicy::default(),
            context_injection: crate::consensus::types::ContextInjectionStrategy::Smart,
        };

        let pipeline = ConsensusPipeline::new(config, profile, None);
        let result = pipeline
            .run("What is Rust?", None)
            .await
            .expect("Pipeline should run successfully");

        assert!(result.success);
        assert!(result.result.is_some());
        assert_eq!(result.stages.len(), 4);
    }
}