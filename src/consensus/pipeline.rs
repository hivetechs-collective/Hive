// Consensus Pipeline - Orchestrates the 4-stage consensus process
// Manages flow from Generator → Refiner → Validator → Curator

use crate::consensus::stages::{ConsensusStage, CuratorStage, GeneratorStage, RefinerStage, ValidatorStage};
use crate::consensus::streaming::{ConsoleCallbacks, ProgressTracker, StreamingCallbacks, ProgressInfo};
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
use rusqlite::{params, OptionalExtension};
use crate::consensus::models::ModelManager;
use crate::core::database::DatabaseManager;
use crate::core::usage_tracker::UsageTracker;
use crate::subscription::conversation_gateway::ConversationGateway;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
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
    database: Option<Arc<DatabaseManager>>,
    api_key: Option<String>,
    usage_tracker: Option<Arc<UsageTracker>>,
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
        let (openrouter_client, model_manager) = if let Some(ref key) = api_key {
            let client = Arc::new(OpenRouterClient::new(key.clone()));
            let manager = Arc::new(ModelManager::new(Some(key.clone())));
            (Some(client), Some(manager))
        } else {
            (None, None)
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
            database: None, // Will be set later when needed
            api_key,
            usage_tracker: None, // Will be set when database is provided
        }
    }

    /// Create with custom callbacks
    pub fn with_callbacks(mut self, callbacks: Arc<dyn StreamingCallbacks>) -> Self {
        self.callbacks = callbacks;
        self
    }

    /// Set the database for model management
    pub fn with_database(mut self, database: Arc<DatabaseManager>) -> Self {
        self.database = Some(database.clone());
        // Initialize usage tracker with unified database
        self.usage_tracker = Some(Arc::new(UsageTracker::new(database)));
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
        user_id: Option<String>,
    ) -> Result<ConsensusResult> {
        // TODO: Re-enable usage tracking when database is unified
        // Check usage limits before running consensus
        // if let (Some(user_id), Some(usage_tracker)) = (user_id.as_ref(), &self.usage_tracker) {
        //     let usage_check = usage_tracker.check_usage_before_conversation(user_id).await?;
        //     
        //     if !usage_check.allowed {
        //         return Err(anyhow::anyhow!(
        //             "Usage limit reached: {}. {} conversations remaining today.",
        //             usage_check.reason,
        //             usage_check.remaining_conversations
        //         ));
        //     }
        //     
        //     tracing::info!(
        //         "Usage check passed for user {}: {} - {} remaining",
        //         user_id,
        //         usage_check.reason,
        //         usage_check.remaining_conversations
        //     );
        // }
        
        // Check if maintenance has run recently
        if let Some(db) = &self.database {
            use crate::consensus::maintenance::TemplateMaintenanceManager;
            
            let mut maintenance = TemplateMaintenanceManager::new(
                db.clone(), 
                self.api_key.clone()
            );
            
            if maintenance.needs_maintenance() {
                tracing::info!("Running model maintenance before consensus...");
                match maintenance.run_maintenance().await {
                    Ok(report) => {
                        tracing::info!("Model maintenance complete: {} models synced, {} profiles migrated", 
                                     report.models_synced, report.migrated_profiles.len());
                    }
                    Err(e) => {
                        tracing::warn!("Model maintenance failed: {}. Continuing anyway.", e);
                    }
                }
            }
        }
        
        let conversation_id = Uuid::new_v4().to_string();
        let pipeline_start = Instant::now();
        let mut total_cost = 0.0;

        // UPDATE D1 AND REFRESH UI IMMEDIATELY BEFORE CONSENSUS STARTS
        // This ensures the count is updated in real-time as soon as user initiates consensus
        let license_key_for_d1 = if let Some(db) = &self.database {
            // Get the current license key from configurations
            let conn = db.get_connection()?;
            let license_key = tokio::task::spawn_blocking(move || -> Result<Option<String>> {
                // Get current license key only - D1 will validate and return user info
                let license_key: Option<String> = conn.query_row(
                    "SELECT value FROM configurations WHERE key = 'hive_license_key'",
                    [],
                    |row| row.get(0)
                ).optional()?;
                Ok(license_key)
            }).await??;
            
            if let Some(license_key) = license_key {
                tracing::info!("Starting consensus with license key: {}", 
                             &license_key[..8]); // Log only first 8 chars for security
                
                // Call D1 to validate license and increment conversation count IMMEDIATELY
                // D1 is the source of truth for user information
                let gateway = ConversationGateway::new()?;
                match gateway.request_conversation_authorization(question, &license_key).await {
                    Ok(auth) => {
                        tracing::info!("D1 authorization successful! {} conversations remaining today", 
                                     auth.remaining);
                        
                        // Store the authorization for later verification
                        // The remaining count is already updated in D1 and should refresh UI
                        Some((license_key, auth))
                    }
                    Err(e) => {
                        tracing::error!("Failed to authorize with D1: {}", e);
                        // Return the error to stop consensus if authorization fails
                        return Err(e);
                    }
                }
            } else {
                tracing::info!("No license key configured, running without D1 authorization");
                None
            }
        } else {
            None
        };
        
        // Store the D1 authorization for later verification
        let d1_auth = license_key_for_d1.map(|(_, auth)| auth);

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

        // Get memory context (recent + thematic) - CRITICAL for multi-conversation intelligence
        tracing::info!("Retrieving memory context for question: {}", question);
        let memory_context = self.get_memory_context(question).await?;
        if let Some(ref mem_ctx) = memory_context {
            tracing::info!("Retrieved memory context with {} characters", mem_ctx.len());
            tracing::debug!("Memory context preview: {}", &mem_ctx.chars().take(200).collect::<String>());
        } else {
            tracing::info!("No memory context found (this is normal for first conversation)");
        }

        // Build full context including semantic, temporal, and memory
        let full_context = self.build_full_context(context, temporal_context, memory_context).await?;

        let mut previous_answer: Option<String> = None;
        let mut stage_results = Vec::new();

        // Run through all 4 stages
        for (i, stage_handler) in self.stages.iter().enumerate() {
            let stage = stage_handler.stage();
            
            // Use model directly from profile - the maintenance system ensures these are always valid
            let model = self.profile.get_model_for_stage(stage).to_string();

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
            // Only provide memory context to Generator stage (first stage)
            let stage_context = match stage {
                Stage::Generator => full_context.as_deref(),
                _ => None, // Other stages don't get memory context directly
            };
            
            let stage_result = self
                .run_single_stage(
                    stage,
                    stage_handler.as_ref(),
                    question,
                    previous_answer.as_deref(),
                    stage_context,
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
            stages: stage_results.clone(),
            conversation_id: conversation_id.clone(),
            total_duration: pipeline_start.elapsed().as_secs_f64(),
            total_cost,
        };

        // CRITICAL: Store conversation and curator result in database (like TypeScript implementation)
        if let Some(db) = &self.database {
            tracing::info!("Database available, attempting to store consensus result for conversation {}", conversation_id);
            if let Err(e) = self.store_consensus_result(
                &conversation_id,
                question,
                &final_answer,
                &stage_results,
                total_cost,
                db.clone()
            ).await {
                tracing::error!("Failed to store consensus result in database: {}", e);
                // Don't fail the entire consensus for storage errors, but log them
            }
        } else {
            tracing::warn!("No database available - consensus results will not be persisted!");
        }
        
        // TODO: Re-enable usage tracking when database is unified
        // Record usage after successful consensus completion
        // if let (Some(user_id), Some(usage_tracker)) = (user_id.as_ref(), &self.usage_tracker) {
        //     if let Err(e) = usage_tracker.record_conversation_usage(user_id, &conversation_id).await {
        //         tracing::error!("Failed to record conversation usage: {}", e);
        //         // Don't fail the consensus for usage tracking errors
        //     } else {
        //         tracing::info!("Recorded conversation usage for user {} (conversation: {})", user_id, conversation_id);
        //     }
        // }

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

        // Report conversation completion to D1 if we have authorization
        if let Some(auth) = d1_auth {
            let gateway = ConversationGateway::new()?;
            match gateway.report_conversation_completion(
                &auth.conversation_token,
                &conversation_id,
                &auth.question_hash
            ).await {
                Ok(verification) => {
                    if verification.verified {
                        tracing::info!("Conversation verified with D1 ({} remaining)", verification.remaining_conversations);
                    } else {
                        tracing::warn!("D1 verification failed but continuing");
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to verify conversation completion with D1: {}", e);
                    // Don't fail the whole consensus over verification
                }
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

        // Call model with retry logic for fallback models
        let mut response = self
            .call_model(model, &messages, &mut tracker)
            .await
            .with_context(|| format!("Failed to call model {} for {}", model, stage.display_name()))?;
        
        // Check if we need to retry with a different model
        if response.content.starts_with("RETRY_WITH_MODEL:") {
            let replacement_model = response.content.trim_start_matches("RETRY_WITH_MODEL:");
            tracing::info!("Retrying with replacement model: {}", replacement_model);
            
            // Call with replacement model
            response = self
                .call_model(replacement_model, &messages, &mut tracker)
                .await
                .with_context(|| format!("Failed to call replacement model {} for {}", replacement_model, stage.display_name()))?;
        }

        // Mark complete
        tracker.complete()?;

        // Build stage result
        let stage_result = StageResult {
            stage_id,
            stage_name: stage.as_str().to_string(),
            question: question.to_string(),
            answer: response.content,
            model: response.model,
            conversation_id: conversation_id.to_string(),
            timestamp: Utc::now(),
            usage: Some(response.usage),
            analytics: Some(StageAnalytics {
                duration: stage_start.elapsed().as_secs_f64(),
                cost: response.analytics.cost,
                provider: response.analytics.provider,
                model_internal_id: response.analytics.model_internal_id,
                quality_score: response.analytics.quality_score,
                error_count: response.analytics.error_count,
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

    /// Build full context combining semantic, temporal, and memory
    async fn build_full_context(
        &self,
        semantic_context: Option<String>,
        temporal_context: Option<crate::consensus::temporal::TemporalContext>,
        memory_context: Option<String>,
    ) -> Result<Option<String>> {
        let mut contexts = Vec::new();

        // Add memory context first (most important for multi-conversation intelligence)
        // This is the AUTHORITATIVE knowledge from previous curator results
        if let Some(memory) = memory_context {
            if !memory.is_empty() {
                contexts.push(memory); // Already formatted with headers
            }
        }

        if let Some(temporal) = temporal_context {
            contexts.push(format!(
                "## Temporal Context\n{}\n{}",
                temporal.search_instruction, temporal.temporal_awareness
            ));
        }

        if let Some(semantic) = semantic_context {
            contexts.push(format!("## Semantic Context\n{}", semantic));
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
            // Create a custom callback that forwards to our tracker
            let stage = tracker.stage.clone();
            let tracker_callbacks = tracker.callbacks.clone();
            
            let callbacks = Box::new(TrackerForwardingCallbacks {
                tracker_callbacks,
                stage,
            }) as Box<dyn OpenRouterStreamingCallbacks>;

            match openrouter_client.chat_completion_stream(request, Some(callbacks)).await {
                Ok(response_content) => {
                    // The streaming callbacks have already been called during streaming
                    // Just update the final content
                    tracker.content = response_content.clone();

                    Ok(ModelResponse {
                        model: model.to_string(),
                        content: response_content,
                        usage: TokenUsage {
                            prompt_tokens: 0,
                            completion_tokens: 0,
                            total_tokens: 0,
                        },
                        analytics: StageAnalytics {
                            duration: 0.0,
                            cost: 0.0,
                            provider: "openrouter".to_string(),
                            model_internal_id: model.to_string(),
                            quality_score: 1.0,
                            error_count: 0,
                            fallback_used: false,
                            rate_limit_hit: false,
                            retry_count: 0,
                            start_time: Utc::now(),
                            end_time: Utc::now(),
                            memory_usage: None,
                            features: crate::consensus::types::AnalyticsFeatures {
                                streaming: self.config.enable_streaming,
                                routing_variant: "balanced".to_string(),
                                optimization_applied: Some(true),
                            },
                        },
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
                        model: model.to_string(),
                        content,
                        usage: usage.unwrap_or(TokenUsage {
                            prompt_tokens: 0,
                            completion_tokens: 0,
                            total_tokens: 0,
                        }),
                        analytics: StageAnalytics {
                            duration: 0.0,
                            cost,
                            provider: "openrouter".to_string(),
                            model_internal_id: model.to_string(),
                            quality_score: 1.0,
                            error_count: 0,
                            fallback_used: false,
                            rate_limit_hit: false,
                            retry_count: 0,
                            start_time: Utc::now(),
                            end_time: Utc::now(),
                            memory_usage: None,
                            features: crate::consensus::types::AnalyticsFeatures {
                                streaming: self.config.enable_streaming,
                                routing_variant: "balanced".to_string(),
                                optimization_applied: Some(true),
                            },
                        },
                    })
                }
                Err(e) => {
                    self.handle_api_error(e, tracker, model).await
                }
            }
        }
    }

    /// Handle API errors
    async fn handle_api_error(
        &self,
        error: anyhow::Error,
        tracker: &mut ProgressTracker,
        model: &str,
    ) -> Result<ModelResponse> {
        // Check if this is a model unavailable error
        let error_str = error.to_string();
        let is_model_unavailable = error_str.contains("404") || 
                                   error_str.contains("model not found") ||
                                   error_str.contains("model unavailable");
        
        if is_model_unavailable {
            tracing::warn!("Model {} is unavailable, attempting to find replacement", model);
            
            // Try to find a replacement model
            if let Some(db) = &self.database {
                use crate::consensus::maintenance::TemplateMaintenanceManager;
                
                let maintenance = TemplateMaintenanceManager::new(
                    db.clone(),
                    self.api_key.clone()
                );
                
                if let Ok(Some(replacement)) = maintenance.find_model_replacement(model).await {
                    tracing::info!("Found replacement model: {} -> {}", model, replacement);
                    
                    // Return a special response indicating we need to retry with a different model
                    return Ok(ModelResponse {
                        model: replacement.clone(),
                        content: format!("RETRY_WITH_MODEL:{}", replacement),
                        usage: TokenUsage {
                            prompt_tokens: 0,
                            completion_tokens: 0,
                            total_tokens: 0,
                        },
                        analytics: StageAnalytics {
                            duration: 0.0,
                            cost: 0.0,
                            provider: "fallback".to_string(),
                            model_internal_id: "0".to_string(),
                            quality_score: 0.0,
                            error_count: 1,
                            fallback_used: true,
                            rate_limit_hit: false,
                            retry_count: 0,
                            start_time: Utc::now(),
                            end_time: Utc::now(),
                            memory_usage: None,
                            features: crate::consensus::types::AnalyticsFeatures {
                                streaming: self.config.enable_streaming,
                                routing_variant: "fallback".to_string(),
                                optimization_applied: Some(false),
                            },
                        },
                    });
                }
            }
        }
        
        // For other errors or if no replacement found, return the error
        Err(anyhow::anyhow!(
            "OpenRouter API call failed for model {}: {}",
            model, 
            error
        ))
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

    /// Store consensus result in database (matching TypeScript implementation)
    async fn store_consensus_result(
        &self,
        conversation_id: &str,
        question: &str,
        final_answer: &str,
        stage_results: &[StageResult],
        total_cost: f64,
        database: Arc<DatabaseManager>,
    ) -> Result<()> {
        tracing::info!("Starting to store consensus result for conversation {}", conversation_id);
        tracing::debug!("Question: {}", question);
        tracing::debug!("Number of stage results: {}", stage_results.len());
        tracing::debug!("Profile ID: {}", self.profile.id);
        
        let now = Utc::now().to_rfc3339();
        let mut conn = database.get_connection()?;
        tracing::debug!("Got database connection");
        
        // Use spawn_blocking for the entire database transaction
        let conversation_id = conversation_id.to_string();
        let question = question.to_string();
        let final_answer = final_answer.to_string();
        let stage_results = stage_results.to_vec();
        let profile_id = self.profile.id.clone(); // Capture profile ID before closure
        
        tokio::task::spawn_blocking(move || -> Result<()> {
            tracing::debug!("Inside spawn_blocking, starting transaction");
            let tx = conn.transaction()?;
            
            // 1. Store conversation record
            tracing::debug!("Storing conversation record with ID: {}", conversation_id);
            let rows_affected = tx.execute(
                "INSERT OR REPLACE INTO conversations (
                    id, user_id, consensus_profile_id, total_cost, 
                    input_tokens, output_tokens, start_time, end_time, 
                    success, quality_score, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    conversation_id,
                    None::<String>, // user_id is optional, like TypeScript version
                    profile_id, // Use the actual profile ID from the pipeline
                    total_cost,
                    stage_results.iter().map(|s| s.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0)).sum::<u32>(),
                    stage_results.iter().map(|s| s.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0)).sum::<u32>(),
                    &now, // start_time
                    &now, // end_time
                    1i32, // success (true)
                    0.95f64, // quality_score (default high quality)
                    &now, // created_at
                    &now  // updated_at
                ]
            )?;
            tracing::debug!("Conversation record stored, {} rows affected", rows_affected);
            
            // 2. Store all stage outputs as messages for audit trail
            for stage_result in &stage_results {
                // Store user message (question) - only for first stage
                if stage_result.stage_name == "generator" {
                    tx.execute(
                        "INSERT INTO messages (
                            id, conversation_id, role, content, stage, model_used, timestamp
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            uuid::Uuid::new_v4().to_string(),
                            conversation_id,
                            "user",
                            question,
                            None::<String>, // User message has no stage
                            None::<String>, // User message has no model
                            &now
                        ]
                    )?;
                }
                
                // Store assistant message for each stage
                tx.execute(
                    "INSERT INTO messages (
                        id, conversation_id, role, content, stage, model_used, timestamp
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        conversation_id,
                        "assistant",
                        stage_result.answer,
                        stage_result.stage_name,
                        stage_result.model,
                        stage_result.timestamp.to_rfc3339()
                    ]
                )?;
            }
            
            // 3. Store in knowledge_conversations (extended format matching TypeScript)
            tx.execute(
                "INSERT OR REPLACE INTO knowledge_conversations (
                    id, conversation_id, question, final_answer, source_of_truth, 
                    conversation_context, profile_id, created_at, last_updated
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    conversation_id,
                    question,
                    final_answer,
                    final_answer, // Final answer is the "source of truth" from Curator
                    None::<String>, // TODO: Add conversation context if available
                    profile_id, // Use the actual profile ID
                    &now,
                    &now
                ]
            )?;
            
            // 4. Store conversation usage for tracking - using current license key
            // Get the currently configured license key first
            let current_license_key: Option<String> = tx.query_row(
                "SELECT value FROM configurations WHERE key = 'hive_license_key'",
                [],
                |row| row.get(0)
            ).optional()?;
            
            // Then get the user_id that matches the current license key
            let user_id_result: Option<String> = if let Some(license_key) = current_license_key {
                tx.query_row(
                    "SELECT id FROM users WHERE license_key = ?1",
                    params![&license_key],
                    |row| row.get(0)
                ).optional()?
            } else {
                None
            };
            
            if let Some(user_id) = user_id_result {
                // Check if we already recorded this at the start (to avoid duplicates)
                let already_recorded: bool = tx.query_row(
                    "SELECT 1 FROM conversation_usage WHERE conversation_id = ?1 AND user_id = ?2",
                    params![conversation_id, &user_id],
                    |_| Ok(true)
                ).unwrap_or(false);
                
                if !already_recorded {
                    // Insert into conversation_usage table for tracking
                    tx.execute(
                        "INSERT INTO conversation_usage (
                            conversation_id, user_id, timestamp
                        ) VALUES (?1, ?2, ?3)",
                        params![
                            conversation_id,
                            user_id,
                            &now
                        ]
                    )?;
                    tracing::info!("Recorded conversation usage for user {}", user_id);
                } else {
                    tracing::debug!("Conversation usage already recorded for user {}", user_id);
                }
            } else {
                tracing::warn!("No user found with license key, skipping usage tracking");
            }
            
            // 5. Store curator truth (flagged as source of truth - critical for memory system)
            tracing::debug!("Looking for curator stage result");
            let curator_result = stage_results.iter()
                .find(|s| s.stage_name == "curator")
                .ok_or_else(|| {
                    tracing::error!("No curator stage result found! Stage names: {:?}", 
                        stage_results.iter().map(|s| &s.stage_name).collect::<Vec<_>>());
                    anyhow::anyhow!("No curator stage result found")
                })?;
            tracing::debug!("Found curator result");
            
            let confidence_score = curator_result.analytics
                .as_ref()
                .map(|a| a.quality_score)
                .unwrap_or(0.8); // Default confidence
            
            let topic_summary = Self::extract_topic_summary(&question, &final_answer);
            
            tx.execute(
                "INSERT OR REPLACE INTO curator_truths (
                    conversation_id, curator_output, confidence_score, topic_summary, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    conversation_id,
                    curator_result.answer,
                    confidence_score,
                    topic_summary,
                    &now
                ]
            )?;
            
            tracing::debug!("Committing transaction");
            tx.commit()?;
            tracing::info!("Successfully stored consensus result for conversation {} in database", conversation_id);
            Ok(())
        }).await??;
        
        Ok(())
    }
    
    /// Extract topic summary from question and answer (simple implementation)
    fn extract_topic_summary(question: &str, answer: &str) -> String {
        // Simple topic extraction - in production this would use AI or NLP
        let combined = format!("{} {}", question, answer);
        let words: Vec<&str> = combined
            .split_whitespace()
            .filter(|w| w.len() > 4) // Filter short words
            .take(5) // Take first 5 meaningful words
            .collect();
        
        if words.is_empty() {
            "general_topic".to_string()
        } else {
            words.join("_").to_lowercase()
        }
    }

    /// Get memory context for question (temporal + thematic)
    async fn get_memory_context(&self, question: &str) -> Result<Option<String>> {
        if let Some(db) = &self.database {
            let context = self.build_memory_context(question, db.clone()).await?;
            if context.is_empty() {
                Ok(None)
            } else {
                Ok(Some(context))
            }
        } else {
            Ok(None)
        }
    }

    /// Build memory context combining recent and thematic memories (matching TypeScript)
    async fn build_memory_context(&self, query: &str, database: Arc<DatabaseManager>) -> Result<String> {
        tracing::debug!("Building memory context for query: {}", query);
        let mut conn = database.get_connection()?;
        let query = query.to_string();

        // Use spawn_blocking for database queries
        let results = tokio::task::spawn_blocking(move || -> Result<String> {
            let mut context_parts = Vec::new();

            // TEMPORAL PRIORITY SYSTEM (matches TypeScript implementation)
            // Priority 1: Recent conversations (24h) - NEWEST FIRST
            let twenty_four_hours_ago = (Utc::now() - chrono::Duration::hours(24)).to_rfc3339();
            tracing::debug!("Looking for recent curator knowledge since: {}", twenty_four_hours_ago);

            // First check if this is a follow-up query (simple pattern matching)
            let query_lower = query.to_lowercase();
            let is_follow_up = [
                "give me", "tell me more", "can you explain", "show me", "what about",
                "examples", "code example", "how does", " it ", " that ", " this ",
                " those ", " these ", " them ", " they "
            ].iter().any(|pattern| query_lower.contains(pattern));
            
            tracing::debug!("Is follow-up query: {}", is_follow_up);

            // TEMPORAL SEARCH: One conversation at a time, most recent first
            // Step 1: Search backwards through time, one conversation at a time
            let mut found_context = false;
            let mut selected_memory: Option<(String, String, f64, String)> = None;
            
            // Get ALL conversations from the last 24 hours, ordered by recency
            let mut stmt = conn.prepare(
                "SELECT source_of_truth, question, 1.0 as confidence_score, created_at
                 FROM knowledge_conversations
                 WHERE created_at > ?1
                 ORDER BY created_at DESC"
            )?;

            let all_recent_results = stmt.query_map([&twenty_four_hours_ago], |row| {
                Ok((
                    row.get::<_, String>(0)?, // curator_output
                    row.get::<_, String>(1)?, // question
                    row.get::<_, f64>(2)?,    // confidence_score
                    row.get::<_, String>(3)?, // created_at
                ))
            })?;

            // Search one by one until we find relevant context
            for result in all_recent_results {
                let (curator_output, question, confidence, created_at) = result?;
                
                // For follow-up queries, the most recent is ALWAYS relevant
                if is_follow_up && selected_memory.is_none() {
                    tracing::debug!("Follow-up detected - using most recent conversation: {}", question);
                    selected_memory = Some((curator_output, question, confidence, created_at));
                    found_context = true;
                    break;
                }
                
                // For non-follow-up, check if this conversation might be relevant
                let query_words: Vec<String> = query.to_lowercase()
                    .split_whitespace()
                    .filter(|w| w.len() > 2)
                    .map(|w| w.to_string())
                    .collect();
                
                let conversation_relevant = query_words.iter().any(|word| {
                    question.to_lowercase().contains(word) || 
                    curator_output.to_lowercase().contains(word)
                });
                
                if conversation_relevant {
                    tracing::debug!("Found relevant conversation: {}", question);
                    selected_memory = Some((curator_output, question, confidence, created_at));
                    found_context = true;
                    break;
                }
            }
            
            // If we found context in recent conversations, use it
            if found_context && selected_memory.is_some() {
                let (curator_output, question, _, created_at) = selected_memory.unwrap();
                let date = chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|_| "recent".to_string());
                
                context_parts.push("🧠 AUTHORITATIVE MEMORY CONTEXT:".to_string());
                context_parts.push("Here is the most relevant recent conversation:".to_string());
                context_parts.push("".to_string());
                context_parts.push(format!(
                    "From {}: Question: {}\n\nVerified Answer:\n{}\n\n---",
                    date, question, curator_output
                ));
                context_parts.push("".to_string());
                
                if is_follow_up {
                    context_parts.push("🚨 FOLLOW-UP DETECTED: This is a follow-up question.".to_string());
                    context_parts.push("You MUST provide examples/clarification about the topic above.".to_string());
                    context_parts.push("Do NOT reference any other topics or previous conversations.".to_string());
                } else {
                    context_parts.push("⚡ Use this context to inform your response.".to_string());
                }
                
                return Ok(context_parts.join("\n"));
            }
            
            tracing::debug!("No relevant context found in recent 24h conversations, falling back to thematic search");
            
            // If follow-up but no 24h context, try broader window (72 hours)
            if is_follow_up {
                let seventy_two_hours_ago = (Utc::now() - chrono::Duration::hours(72)).to_rfc3339();
                tracing::debug!("Follow-up detected, checking broader window (72h)");
                
                let mut stmt = conn.prepare(
                    "SELECT source_of_truth, question, 1.0 as confidence_score, created_at
                     FROM knowledge_conversations
                     WHERE created_at > ?1 AND created_at <= ?2
                     ORDER BY created_at DESC
                     LIMIT 3"
                )?;
                
                let broader_results = stmt.query_map(
                    rusqlite::params![&seventy_two_hours_ago, &twenty_four_hours_ago], 
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?, // source_of_truth (curator output)  
                            row.get::<_, String>(1)?, // question
                            row.get::<_, f64>(2)?,    // confidence_score
                            row.get::<_, String>(3)?, // created_at
                        ))
                    }
                )?;
                
                let mut broader_memories = Vec::new();
                for result in broader_results {
                    broader_memories.push(result?);
                }
                
                if !broader_memories.is_empty() {
                    context_parts.push("🔄 RECENT FOLLOW-UP CONTEXT (72h):".to_string());
                    for (i, (curator_output, question, _confidence, created_at)) in broader_memories.iter().enumerate() {
                        let date = chrono::DateTime::parse_from_rfc3339(created_at)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_else(|_| "recent".to_string());
                        
                        context_parts.push(format!(
                            "RECENT CONVERSATION {} (from {}):\nQuestion: {}\n\nAnswer:\n{}\n\n---",
                            i + 1, date, question, curator_output
                        ));
                    }
                    
                    return Ok(context_parts.join("\n"));
                }
            }

            // 2. THEMATIC SEARCH: Search ALL memory, no time limits, but ordered by recency
            tracing::debug!("Starting thematic search for: {}", query);
            let keywords: Vec<String> = query
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .take(5) // Take more keywords for better matching
                .map(|w| format!("%{}%", w.to_lowercase()))
                .collect();

            if !keywords.is_empty() {
                let keyword_query = keywords.iter()
                    .map(|_| "(source_of_truth LIKE ? OR question LIKE ?)")
                    .collect::<Vec<_>>()
                    .join(" OR ");

                // Search ALL history, no time limit, ordered by recency
                let full_query = format!(
                    "SELECT DISTINCT source_of_truth, question, 0.8 as confidence_score, created_at
                     FROM knowledge_conversations
                     WHERE {}
                     ORDER BY created_at DESC",
                    keyword_query
                );

                let mut stmt = conn.prepare(&full_query)?;
                
                // Double the keywords for both source_of_truth and question matching
                let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
                for keyword in &keywords {
                    params.push(keyword as &dyn rusqlite::ToSql);
                    params.push(keyword as &dyn rusqlite::ToSql);
                }

                let thematic_results = stmt.query_map(&params[..], |row| {
                    Ok((
                        row.get::<_, String>(0)?, // curator_output
                        row.get::<_, String>(1)?, // question
                        row.get::<_, f64>(2)?,    // confidence_score
                        row.get::<_, String>(3)?, // created_at
                    ))
                })?;

                // Process thematic results one by one, most recent first
                let mut found_thematic = false;
                for result in thematic_results {
                    let (curator_output, question, _, created_at) = result?;
                    
                    // Skip if this was already checked in temporal search
                    let created_dt = chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now());
                    if created_dt > chrono::Utc::now() - chrono::Duration::hours(24) {
                        continue; // Already checked in temporal search
                    }
                    
                    // For thematic search, be more selective - require stronger match
                    let strong_match = keywords.iter()
                        .filter(|k| k.len() > 5) // Only check meaningful keywords
                        .any(|keyword| {
                            let clean_keyword = keyword.replace("%", "").to_lowercase();
                            question.to_lowercase().contains(&clean_keyword) ||
                            curator_output.to_lowercase().contains(&clean_keyword)
                        });
                    
                    if strong_match {
                        tracing::debug!("Found thematic match: {}", question);
                        let date = created_dt.format("%Y-%m-%d").to_string();
                        
                        context_parts.push("📚 THEMATIC CONTEXT:".to_string());
                        context_parts.push(format!(
                            "Related conversation from {}: Question: {}\n\nAnswer:\n{}\n\n---",
                            date, question, curator_output
                        ));
                        context_parts.push("".to_string());
                        context_parts.push("⚡ This is older context that may be relevant to your query.".to_string());
                        
                        found_thematic = true;
                        break; // Use only the most recent thematic match
                    }
                }
                
                if !found_thematic && is_follow_up {
                    // For follow-ups with no context found, provide a clear message
                    context_parts.push("⚠️ NO RECENT CONTEXT FOUND".to_string());
                    context_parts.push("This appears to be a follow-up question, but no recent conversation context was found.".to_string());
                    context_parts.push("Please provide a complete, standalone answer.".to_string());
                }
            }

            Ok(context_parts.join("\n"))
        }).await??;

        Ok(results)
    }
}

/// Response from model API
struct ModelResponse {
    model: String,
    content: String,
    usage: TokenUsage,
    analytics: StageAnalytics,
}

/// Callbacks that forward OpenRouter streaming to our consensus callbacks
struct TrackerForwardingCallbacks {
    tracker_callbacks: Arc<dyn StreamingCallbacks>,
    stage: Stage,
}

impl OpenRouterStreamingCallbacks for TrackerForwardingCallbacks {
    fn on_start(&self) {
        // Already handled by on_stage_start
    }
    
    fn on_chunk(&self, chunk: String, total_content: String) {
        // Forward to our consensus callbacks
        tracing::debug!("TrackerForwardingCallbacks: Received chunk for stage {:?}: '{}'", self.stage, chunk);
        let result = self.tracker_callbacks.on_stage_chunk(self.stage, &chunk, &total_content);
        if let Err(e) = result {
            tracing::error!("Failed to forward chunk: {}", e);
        }
    }
    
    fn on_progress(&self, progress: crate::consensus::openrouter::StreamingProgress) {
        // Convert and forward
        if let Some(tokens) = progress.tokens {
            let percentage = progress.percentage.unwrap_or(0.0);
            let info = ProgressInfo {
                tokens,
                estimated_total: None, // We don't have a good estimate from OpenRouter
                percentage: (percentage / 100.0) as f32, // Convert to 0-1 range
            };
            let _ = self.tracker_callbacks.on_stage_progress(self.stage, info);
        }
    }
    
    fn on_complete(&self, _final_content: String, _usage: Option<crate::consensus::openrouter::Usage>) {
        // Will be handled by on_stage_complete
    }
    
    fn on_error(&self, error: &anyhow::Error) {
        let _ = self.tracker_callbacks.on_error(self.stage, error);
    }
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