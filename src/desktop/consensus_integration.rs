//! Consensus Engine Integration for Desktop App
//!
//! This module connects the consensus engine to the desktop UI,
//! providing real-time updates and progress tracking.

use crate::consensus::{
    engine::ConsensusEngine,
    pipeline::ConsensusPipeline,
    types::{ConsensusConfig, Stage},
    repository_context::RepositoryContextManager,
    streaming::{ProgressInfo, StreamingCallbacks},
    cancellation::{CancellationToken, CancellationReason},
};
use crate::core::api_keys::ApiKeyManager;
use crate::desktop::markdown;
use crate::desktop::state::{AppState, ConsensusStage, StageInfo, StageStatus};
use anyhow::Result;
use dioxus::prelude::*;
use rusqlite::OptionalExtension;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::{sync::{mpsc, Mutex}, spawn};

/// Events sent from callbacks to UI
#[derive(Debug, Clone)]
pub enum ConsensusUIEvent {
    ProfileLoaded {
        profile_name: String,
        models: Vec<String>,
    },
    StageStarted {
        stage: ConsensusStage,
        model: String,
    },
    StageProgress {
        stage: ConsensusStage,
        percentage: u8,
        tokens: usize,
    },
    StageCompleted {
        stage: ConsensusStage,
        cost: f64,
    },
    StageError {
        stage: ConsensusStage,
        error: String,
    },
    TokenUpdate {
        count: usize,
        cost: f64,
    },
    StreamingChunk {
        stage: ConsensusStage,
        chunk: String,
        total_content: String,
    },
    D1Authorization {
        total_remaining: u32,
    }, // New event for D1 auth info
    AnalyticsRefresh, // New event for analytics refresh after consensus completion
    Cancelled { // New event for cancellation
        reason: String,
    },
    Completed, // Event when consensus completes successfully
    OperationsRequireConfirmation { // New event for file operations needing approval
        operations: Vec<crate::consensus::ai_operation_parser::FileOperationWithMetadata>,
    },
}

/// Desktop streaming callbacks that send events to UI
pub struct DesktopStreamingCallbacks {
    event_sender: mpsc::UnboundedSender<ConsensusUIEvent>,
    auto_accept_enabled: Arc<std::sync::atomic::AtomicBool>,
    conversation_id: String,
    question: String,
    stages_completed: Arc<Mutex<Vec<(Stage, crate::consensus::types::StageResult)>>>,
}

impl DesktopStreamingCallbacks {
    pub fn new(
        event_sender: mpsc::UnboundedSender<ConsensusUIEvent>, 
        auto_accept_enabled: Arc<std::sync::atomic::AtomicBool>,
        conversation_id: String,
        question: String,
    ) -> Self {
        Self { 
            event_sender, 
            auto_accept_enabled,
            conversation_id,
            question,
            stages_completed: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Dual channel callbacks that send to both streaming and internal channels
pub struct DualChannelCallbacks {
    stream_sender: mpsc::UnboundedSender<ConsensusUIEvent>,
    internal_sender: mpsc::UnboundedSender<ConsensusUIEvent>,
    auto_accept_enabled: Arc<std::sync::atomic::AtomicBool>,
    conversation_id: String,
    question: String,
    stages_completed: Arc<Mutex<Vec<(Stage, crate::consensus::types::StageResult)>>>,
}

impl StreamingCallbacks for DesktopStreamingCallbacks {
    fn on_profile_loaded(&self, profile_name: &str, models: &[String]) -> Result<()> {
        tracing::info!("🎯 Profile loaded callback: {} with models {:?}", profile_name, models);
        
        // Send profile loaded event to update UI
        let _ = self.event_sender.send(ConsensusUIEvent::ProfileLoaded {
            profile_name: profile_name.to_string(),
            models: models.to_vec(),
        });
        
        Ok(())
    }
    
    fn on_stage_start(&self, stage: Stage, model: &str) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        let _ = self.event_sender.send(ConsensusUIEvent::StageStarted {
            stage: consensus_stage,
            model: model.to_string(),
        });

        Ok(())
    }

    fn on_d1_authorization(&self, remaining: u32) -> Result<()> {
        // Send D1 authorization event immediately when received
        let _ = self.event_sender.send(ConsensusUIEvent::D1Authorization {
            total_remaining: remaining,
        });
        Ok(())
    }

    fn on_analytics_refresh(&self) -> Result<()> {
        // Send analytics refresh event after consensus completion
        let _ = self.event_sender.send(ConsensusUIEvent::AnalyticsRefresh);
        Ok(())
    }

    fn on_stage_chunk(&self, stage: Stage, chunk: &str, total_content: &str) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        tracing::debug!(
            "DesktopStreamingCallbacks: Sending chunk for stage {:?}: '{}'",
            stage,
            chunk
        );

        // Stream ALL stages so user sees the full pipeline progress in real-time
        // Only the Curator result will be saved as the final answer
        let result = self.event_sender.send(ConsensusUIEvent::StreamingChunk {
            stage: consensus_stage,
            chunk: chunk.to_string(),
            total_content: total_content.to_string(),
        });

        if let Err(e) = result {
            tracing::error!("Failed to send streaming chunk to UI: {:?}", e);
        } else {
            tracing::debug!(
                "Successfully sent streaming chunk to UI channel for stage {:?}",
                stage
            );
        }

        Ok(())
    }

    fn on_stage_progress(&self, stage: Stage, progress: ProgressInfo) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        let progress_percent = (progress.percentage * 100.0) as u8;

        let _ = self.event_sender.send(ConsensusUIEvent::StageProgress {
            stage: consensus_stage,
            percentage: progress_percent,
            tokens: progress.tokens as usize,
        });

        Ok(())
    }

    fn on_stage_complete(
        &self,
        stage: Stage,
        result: &crate::consensus::types::StageResult,
    ) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        let cost = if let Some(analytics) = &result.analytics {
            // Use actual cost from consensus pipeline analytics (calculated with real database pricing)
            tracing::info!(
                "💰 Using real analytics cost: ${:.6} for model {}",
                analytics.cost,
                result.model
            );
            analytics.cost
        } else {
            tracing::error!("💰 ERROR: No analytics cost data available for stage result");
            0.0 // This should not happen in normal operation
        };

        // Save stage completion data
        let stages_completed = self.stages_completed.clone();
        let conversation_id = self.conversation_id.clone();
        let question = self.question.clone();
        let stage_clone = stage.clone();
        let result_clone = result.clone();
        
        // Spawn task to save data to avoid blocking
        tokio::spawn(async move {
            // Add to completed stages
            {
                let mut completed = stages_completed.lock().await;
                completed.push((stage_clone, result_clone.clone()));
            }
            
            // Save to database
            if let Ok(db) = crate::core::database::get_database().await {
                // If this is the first stage, create the conversation
                if stage_clone == Stage::Generator {
                    tracing::info!("💾 Creating conversation in database: id={}, question={}", 
                        conversation_id, question);
                    
                    if let Err(e) = db.store_conversation_with_cost(
                        &conversation_id,
                        None, // user_id
                        &question,
                        0.0, // initial cost, will be updated
                        0,   // initial tokens
                        0,
                    ).await {
                        tracing::error!("Failed to create conversation: {}", e);
                    } else {
                        tracing::info!("✅ Conversation created successfully in database");
                    }
                }
                
                // Store cost tracking for this stage
                if let Some(analytics) = &result_clone.analytics {
                    tracing::info!("💰 Storing cost tracking: conversation_id={}, model={}, cost=${:.6}, stage={:?}", 
                        conversation_id, result_clone.model, analytics.cost, stage_clone);
                    
                    // Get model internal ID
                    match db.get_model_internal_id(&result_clone.model).await {
                        Ok(model_id) => {
                            // Get token counts from usage if available
                            let (input_tokens, output_tokens) = if let Some(usage) = &result_clone.usage {
                                (usage.prompt_tokens, usage.completion_tokens)
                            } else {
                                (0, 0)
                            };
                            
                            tracing::info!("📊 Tokens: input={}, output={}", input_tokens, output_tokens);
                            
                            if let Err(e) = db.store_cost_tracking(
                                &conversation_id,
                                model_id,
                                input_tokens,
                                output_tokens,
                                analytics.cost,
                            ).await {
                                tracing::error!("Failed to store cost tracking: {}", e);
                            } else {
                                tracing::info!("✅ Cost tracking stored successfully");
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to get model internal ID for {}: {}", result_clone.model, e);
                        }
                    }
                    
                    // Update conversation totals
                    let (input_tokens, output_tokens) = if let Some(usage) = &result_clone.usage {
                        (usage.prompt_tokens, usage.completion_tokens)
                    } else {
                        (0, 0)
                    };
                    
                    if let Err(e) = db.update_conversation_cost(
                        &conversation_id,
                        analytics.cost,
                        input_tokens,
                        output_tokens,
                    ).await {
                        tracing::error!("Failed to update conversation cost: {}", e);
                    }
                } else {
                    tracing::warn!("⚠️ No analytics data available for stage {:?} - skipping cost tracking", stage_clone);
                }
                
                // If this is the curator stage, we're done
                if stage_clone == Stage::Curator {
                    tracing::info!("✅ Consensus pipeline complete, saved to database");
                }
            }
        });

        let _ = self.event_sender.send(ConsensusUIEvent::StageCompleted {
            stage: consensus_stage,
            cost,
        });

        Ok(())
    }

    fn on_error(&self, stage: Stage, error: &anyhow::Error) -> Result<()> {
        let error_str = error.to_string();
        
        // Don't send error event for cancellations - they're expected
        if error_str.contains("cancelled") || error_str.contains("Cancelled") {
            tracing::info!("Stage {:?} cancelled by user", stage);
            return Ok(());
        }
        
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        let _ = self.event_sender.send(ConsensusUIEvent::StageError {
            stage: consensus_stage,
            error: error.to_string(),
        });

        tracing::error!("Consensus error at stage {:?}: {}", stage, error);

        Ok(())
    }
    
    fn get_auto_accept_state(&self) -> Option<bool> {
        // Read the auto-accept state from atomic bool
        Some(self.auto_accept_enabled.load(std::sync::atomic::Ordering::Relaxed))
    }
    
    fn on_operations_require_confirmation(
        &self,
        operations: Vec<crate::consensus::ai_operation_parser::FileOperationWithMetadata>,
    ) -> Result<()> {
        tracing::info!("Operations require confirmation: {} operations", operations.len());
        
        // Send operations to UI for confirmation
        let _ = self.event_sender.send(ConsensusUIEvent::OperationsRequireConfirmation {
            operations,
        });
        
        Ok(())
    }
}

impl StreamingCallbacks for DualChannelCallbacks {
    fn on_profile_loaded(&self, profile_name: &str, models: &[String]) -> Result<()> {
        tracing::info!("🎯 DualChannel profile loaded callback: {} with models {:?}", profile_name, models);
        
        let event = ConsensusUIEvent::ProfileLoaded {
            profile_name: profile_name.to_string(),
            models: models.to_vec(),
        };
        
        // Send to both channels
        let _ = self.stream_sender.send(event.clone());
        let _ = self.internal_sender.send(event);
        
        Ok(())
    }
    
    fn on_stage_start(&self, stage: Stage, model: &str) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        let event = ConsensusUIEvent::StageStarted {
            stage: consensus_stage,
            model: model.to_string(),
        };

        // Send to both channels
        let _ = self.stream_sender.send(event.clone());
        let _ = self.internal_sender.send(event);

        Ok(())
    }

    fn on_d1_authorization(&self, remaining: u32) -> Result<()> {
        // Send D1 authorization event to both channels immediately
        let event = ConsensusUIEvent::D1Authorization {
            total_remaining: remaining,
        };

        let _ = self.stream_sender.send(event.clone());
        let _ = self.internal_sender.send(event);

        Ok(())
    }

    fn on_analytics_refresh(&self) -> Result<()> {
        // Send analytics refresh event to both channels after consensus completion
        let event = ConsensusUIEvent::AnalyticsRefresh;

        let _ = self.stream_sender.send(event.clone());
        let _ = self.internal_sender.send(event);

        Ok(())
    }

    fn on_stage_chunk(&self, stage: Stage, chunk: &str, total_content: &str) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        // Stream ALL stages so user sees the full pipeline progress in real-time
        // Only the Curator result will be saved as the final answer
        let event = ConsensusUIEvent::StreamingChunk {
            stage: consensus_stage,
            chunk: chunk.to_string(),
            total_content: total_content.to_string(),
        };

        // Send to both channels
        let _ = self.stream_sender.send(event.clone());
        let _ = self.internal_sender.send(event);

        Ok(())
    }

    fn on_stage_progress(&self, stage: Stage, progress: ProgressInfo) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        let event = ConsensusUIEvent::StageProgress {
            stage: consensus_stage,
            percentage: (progress.percentage * 100.0) as u8,
            tokens: progress.tokens as usize,
        };

        // Send to both channels
        let _ = self.stream_sender.send(event.clone());
        let _ = self.internal_sender.send(event);

        Ok(())
    }

    fn on_stage_complete(
        &self,
        stage: Stage,
        result: &crate::consensus::types::StageResult,
    ) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        let cost = if let Some(analytics) = &result.analytics {
            // Use actual cost from consensus pipeline analytics (calculated with real database pricing)
            tracing::info!(
                "💰 Using real analytics cost: ${:.6} for model {}",
                analytics.cost,
                result.model
            );
            analytics.cost
        } else {
            tracing::error!("💰 ERROR: No analytics cost data available for stage result");
            0.0 // This should not happen in normal operation
        };

        // Save stage completion data (same as DesktopStreamingCallbacks)
        let stages_completed = self.stages_completed.clone();
        let conversation_id = self.conversation_id.clone();
        let question = self.question.clone();
        let stage_clone = stage.clone();
        let result_clone = result.clone();
        
        // Spawn task to save data to avoid blocking
        tokio::spawn(async move {
            // Add to completed stages
            {
                let mut completed = stages_completed.lock().await;
                completed.push((stage_clone, result_clone.clone()));
            }
            
            // Save to database
            if let Ok(db) = crate::core::database::get_database().await {
                // If this is the first stage, create the conversation
                if stage_clone == Stage::Generator {
                    tracing::info!("💾 Creating conversation in database: id={}, question={}", 
                        conversation_id, question);
                    
                    if let Err(e) = db.store_conversation_with_cost(
                        &conversation_id,
                        None, // user_id
                        &question,
                        0.0, // initial cost, will be updated
                        0,   // initial tokens
                        0,
                    ).await {
                        tracing::error!("Failed to create conversation: {}", e);
                    } else {
                        tracing::info!("✅ Conversation created successfully in database");
                    }
                }
                
                // Store cost tracking for this stage
                if let Some(analytics) = &result_clone.analytics {
                    tracing::info!("💰 Storing cost tracking: conversation_id={}, model={}, cost=${:.6}, stage={:?}", 
                        conversation_id, result_clone.model, analytics.cost, stage_clone);
                    
                    // Get model internal ID
                    match db.get_model_internal_id(&result_clone.model).await {
                        Ok(model_id) => {
                            // Get token counts from usage if available
                            let (input_tokens, output_tokens) = if let Some(usage) = &result_clone.usage {
                                (usage.prompt_tokens, usage.completion_tokens)
                            } else {
                                (0, 0)
                            };
                            
                            tracing::info!("📊 Tokens: input={}, output={}", input_tokens, output_tokens);
                            
                            if let Err(e) = db.store_cost_tracking(
                                &conversation_id,
                                model_id,
                                input_tokens,
                                output_tokens,
                                analytics.cost,
                            ).await {
                                tracing::error!("Failed to store cost tracking: {}", e);
                            } else {
                                tracing::info!("✅ Cost tracking stored successfully");
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to get model internal ID for {}: {}", result_clone.model, e);
                        }
                    }
                    
                    // Update conversation totals
                    let (input_tokens, output_tokens) = if let Some(usage) = &result_clone.usage {
                        (usage.prompt_tokens, usage.completion_tokens)
                    } else {
                        (0, 0)
                    };
                    
                    if let Err(e) = db.update_conversation_cost(
                        &conversation_id,
                        analytics.cost,
                        input_tokens,
                        output_tokens,
                    ).await {
                        tracing::error!("Failed to update conversation cost: {}", e);
                    }
                } else {
                    tracing::warn!("⚠️ No analytics data available for stage {:?} - skipping cost tracking", stage_clone);
                }
                
                // If this is the curator stage, we're done
                if stage_clone == Stage::Curator {
                    tracing::info!("✅ Consensus pipeline complete, saved to database");
                }
            }
        });

        let event = ConsensusUIEvent::StageCompleted {
            stage: consensus_stage,
            cost,
        };

        // Send to both channels
        let _ = self.stream_sender.send(event.clone());
        let _ = self.internal_sender.send(event);

        Ok(())
    }

    fn on_error(&self, stage: Stage, error: &anyhow::Error) -> Result<()> {
        let error_str = error.to_string();
        
        // Don't send error event for cancellations - they're expected
        if error_str.contains("cancelled") || error_str.contains("Cancelled") {
            tracing::info!("Stage {:?} cancelled by user", stage);
            return Ok(());
        }
        
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        let event = ConsensusUIEvent::StageError {
            stage: consensus_stage,
            error: error.to_string(),
        };

        // Send to both channels
        let _ = self.stream_sender.send(event.clone());
        let _ = self.internal_sender.send(event);

        tracing::error!("Consensus error at stage {:?}: {}", stage, error);

        Ok(())
    }
    
    fn get_auto_accept_state(&self) -> Option<bool> {
        // Read the auto-accept state from atomic bool
        Some(self.auto_accept_enabled.load(std::sync::atomic::Ordering::Relaxed))
    }
    
    fn on_operations_require_confirmation(
        &self,
        operations: Vec<crate::consensus::ai_operation_parser::FileOperationWithMetadata>,
    ) -> Result<()> {
        tracing::info!("Operations require confirmation: {} operations", operations.len());
        
        // Send operations to UI for confirmation via both channels
        let event = ConsensusUIEvent::OperationsRequireConfirmation {
            operations,
        };
        
        let _ = self.stream_sender.send(event.clone());
        let _ = self.internal_sender.send(event);
        
        Ok(())
    }
}

/// Process consensus UI events and update app state
pub async fn process_consensus_events(
    mut event_receiver: mpsc::UnboundedReceiver<ConsensusUIEvent>,
    mut app_state: Signal<AppState>,
    consensus_manager: Option<DesktopConsensusManager>,
) {
    while let Some(event) = event_receiver.recv().await {
        match event {
            ConsensusUIEvent::ProfileLoaded { profile_name, models } => {
                tracing::info!("🎯 UI received ProfileLoaded event: {} with models {:?}", profile_name, models);
                let mut state = app_state.write();
                state.consensus.active_profile_name = profile_name;
                state.consensus.stages = vec![
                    StageInfo::new("Generator", &models[0]),
                    StageInfo::new("Refiner", &models[1]),
                    StageInfo::new("Validator", &models[2]),
                    StageInfo::new("Curator", &models[3]),
                ];
            }
            ConsensusUIEvent::StageStarted { stage, model } => {
                let mut state = app_state.write();
                state.consensus.current_stage = Some(stage.clone());

                // Add stage header to show progression but don't clear previous content
                let stage_name = match stage {
                    ConsensusStage::Generator => "Generator",
                    ConsensusStage::Refiner => "Refiner",
                    ConsensusStage::Validator => "Validator",
                    ConsensusStage::Curator => "Curator",
                };

                // Add visual separator for new stage
                if !state.consensus.raw_streaming_content.is_empty() {
                    state.consensus.raw_streaming_content.push_str("\n\n---\n\n");
                }
                state
                    .consensus
                    .raw_streaming_content
                    .push_str(&format!("## 📝 {} Stage\n\n", stage_name));
                // Also update HTML version for display
                state.consensus.streaming_content = markdown::to_html(&state.consensus.raw_streaming_content);

                let stage_index = match stage {
                    ConsensusStage::Generator => 0,
                    ConsensusStage::Refiner => 1,
                    ConsensusStage::Validator => 2,
                    ConsensusStage::Curator => 3,
                };

                if let Some(stage_info) = state.consensus.stages.get_mut(stage_index) {
                    stage_info.status = StageStatus::Running;
                    stage_info.model = model.clone();
                }
                
                // Update all stages when Generator starts (first stage)
                // This ensures the UI shows the correct models from the loaded profile
                if matches!(stage, ConsensusStage::Generator) {
                    tracing::info!("First stage started, updating all stage models in UI");
                    // The profile name should reflect what's actually being used
                    // The engine has already loaded the active profile from database
                }
            }
            ConsensusUIEvent::StageProgress {
                stage,
                percentage,
                tokens,
            } => {
                let mut state = app_state.write();
                state.consensus.update_progress(stage, percentage);
                state.consensus.total_tokens = tokens;
            }
            ConsensusUIEvent::StageCompleted { stage, cost } => {
                let mut state = app_state.write();

                let stage_index = match stage {
                    ConsensusStage::Generator => 0,
                    ConsensusStage::Refiner => 1,
                    ConsensusStage::Validator => 2,
                    ConsensusStage::Curator => 3,
                };

                if let Some(stage_info) = state.consensus.stages.get_mut(stage_index) {
                    stage_info.status = StageStatus::Completed;
                    stage_info.progress = 100;
                }

                state.consensus.estimated_cost += cost;

                // If this is the Curator stage (final stage), trigger analytics refresh
                if matches!(stage, ConsensusStage::Curator) {
                    // Analytics refresh will happen after a short delay to ensure DB saves complete
                    // We can't use tokio::spawn with Signals (they're not Send), so we'll increment 
                    // the trigger immediately and let the analytics system handle the timing
                    state.analytics_refresh_trigger += 1;
                    tracing::info!("Consensus completed - triggered analytics refresh (DB saves will complete soon)");
                }
            }
            ConsensusUIEvent::StageError { stage, error } => {
                let mut state = app_state.write();

                let stage_index = match stage {
                    ConsensusStage::Generator => 0,
                    ConsensusStage::Refiner => 1,
                    ConsensusStage::Validator => 2,
                    ConsensusStage::Curator => 3,
                };

                if let Some(stage_info) = state.consensus.stages.get_mut(stage_index) {
                    stage_info.status = StageStatus::Error;
                    
                    // Parse error message to provide helpful guidance
                    let error_msg = if error.contains("429") || error.contains("rate-limited") {
                        format!("Model {} is temporarily rate-limited. Please try again later or choose a different model.", stage_info.model)
                    } else if error.contains("500") || error.contains("internal server error") {
                        format!("Model {} is experiencing server issues. Please choose a different model.", stage_info.model)
                    } else if error.contains("timeout") {
                        format!("Model {} timed out. Please try again or choose a faster model.", stage_info.model)
                    } else {
                        format!("Model {} encountered an error: {}. If this persists, please choose a different model.", stage_info.model, error)
                    };
                    
                    stage_info.error_message = Some(error_msg);
                    tracing::error!("Stage {} error: {}", stage_info.name, error);
                }
            }
            ConsensusUIEvent::TokenUpdate { count, cost } => {
                let mut state = app_state.write();
                state.consensus.add_tokens(count, cost);
            }
            ConsensusUIEvent::StreamingChunk {
                stage: _,
                chunk: _,
                total_content,
            } => {
                // Update the streaming content in the consensus state
                let mut state = app_state.write();
                // Store raw markdown for AI Helper processing
                state.consensus.raw_streaming_content = total_content.clone();
                
                // Debounce HTML conversion to reduce UI updates
                // Only convert to HTML every 100ms or when content length changes significantly
                let should_update_html = {
                    let current_len = total_content.len();
                    let last_len = state.consensus.last_html_update_len;
                    let now = std::time::Instant::now();
                    let time_since_last = now.duration_since(state.consensus.last_html_update_time);
                    
                    // Update if: 100ms passed OR content grew by 500+ chars OR it's the first update
                    time_since_last >= std::time::Duration::from_millis(100) ||
                    current_len > last_len + 500 ||
                    last_len == 0
                };
                
                if should_update_html {
                    // Convert markdown to HTML for UI rendering
                    state.consensus.streaming_content = markdown::to_html(&total_content);
                    state.consensus.last_html_update_time = std::time::Instant::now();
                    state.consensus.last_html_update_len = total_content.len();
                }
            }
            ConsensusUIEvent::D1Authorization { total_remaining } => {
                // Update app state with D1 authorization info
                let mut state = app_state.write();
                state.total_conversations_remaining = Some(total_remaining);
                tracing::info!("Updated total conversations remaining: {}", total_remaining);

                // Trigger subscription display refresh to show updated counts immediately
                state.subscription_refresh_trigger += 1;
            }
            ConsensusUIEvent::AnalyticsRefresh => {
                // Trigger analytics refresh after consensus completion and database storage
                let mut state = app_state.write();
                state.analytics_refresh_trigger += 1;
                tracing::info!("Analytics refresh triggered - new data available");
            }
            ConsensusUIEvent::Cancelled { reason } => {
                // Handle consensus cancellation
                let mut state = app_state.write();
                
                // Clear all stage progress and streaming content
                state.consensus.streaming_content.clear();
                state.consensus.raw_streaming_content.clear();
                state.consensus.current_stage = None;
                state.consensus.is_running = false;
                state.consensus.is_active = false;
                
                // Reset all stage states by calling complete_consensus which resets everything
                state.consensus.complete_consensus();
                
                tracing::info!("Consensus cancelled in UI: {}", reason);
            }
            ConsensusUIEvent::Completed => {
                // Handle consensus completion - extract data first without holding write lock
                let curator_output = {
                    let state = app_state.read();
                    state.consensus.raw_streaming_content.clone()
                };
                
                // Complete consensus state in separate scope to avoid blocking
                {
                    let mut state = app_state.write();
                    state.consensus.complete_consensus();
                }
                
                tracing::info!("✅ Consensus completed successfully");
                
                // Log AI Helper execution without blocking
                if !curator_output.is_empty() {
                    if consensus_manager.is_some() {
                        tracing::info!("🤖 Waking up AI Helper to execute file operations...");
                        tracing::info!("📝 Curator output length: {} characters", curator_output.len());
                        tracing::info!("🤖 AI Helper bridge activated - parsing Curator output for file operations");
                        // Note: AI Helper execution will happen in the background via existing pipeline
                    } else {
                        tracing::warn!("No consensus manager available for AI Helper execution");
                    }
                } else {
                    tracing::info!("No file operations detected in Curator output");
                }
            }
            ConsensusUIEvent::OperationsRequireConfirmation { operations } => {
                // Handle operations that need user confirmation
                tracing::info!("📁 {} operations require user confirmation", operations.len());
                
                // In Claude Code mode, we show operations inline instead of a dialog
                // For now, we'll still store them but not show the dialog
                let mut state = app_state.write();
                state.pending_operations = Some(operations.clone());
                
                // If auto-accept is on, execute immediately
                if state.auto_accept {
                    tracing::info!("🚀 Auto-accept is ON - executing operations immediately");
                    // TODO: Execute operations directly without dialog
                    state.pending_operations = None;
                } else {
                    // Show inline confirmation UI instead of dialog
                    tracing::info!("⏸ Auto-accept is OFF - showing inline confirmation");
                    // Don't show the dialog - operations will be shown inline
                    state.show_operation_confirmation_dialog = false;
                }
            }
        }
    }
}

/// Consensus manager that integrates with the desktop app
#[derive(Clone)]
pub struct DesktopConsensusManager {
    engine: Arc<Mutex<ConsensusEngine>>,
    repository_context: Arc<RepositoryContextManager>,
    app_state: Signal<AppState>,
    current_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl DesktopConsensusManager {
    /// Create a new desktop consensus manager
    pub async fn new(app_state: Signal<AppState>) -> Result<Self> {
        // Use the global database instance that was initialized at startup
        use crate::core::database::get_database;
        let db = get_database().await?;

        // Create repository context manager
        let repository_context = Arc::new(RepositoryContextManager::new().await?);

        // Initialize repository context from File Explorer state instead of hardcoded working directory
        let file_explorer_root = {
            let app_state_read = app_state.read();
            app_state_read.file_explorer.root_path.clone()
        };
        
        if let Some(explorer_root) = file_explorer_root {
            tracing::info!("🔍 Using File Explorer root for repository context: {}", explorer_root.display());
            
            // Check if File Explorer root looks like a repository
            if explorer_root.join("Cargo.toml").exists() || 
               explorer_root.join("package.json").exists() || 
               explorer_root.join(".git").exists() ||
               explorer_root.join("pyproject.toml").exists() ||
               explorer_root.join("go.mod").exists() {
                
                tracing::info!("🎯 Repository detected! Setting up context for: {}", explorer_root.display());
                
                // Note: App state updates moved to avoid reactive scope conflicts
                let project_name = explorer_root
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Current Project")
                    .to_string();
                    
                tracing::info!("📝 Repository project detected: {}", project_name);
                
                // Update repository context immediately
                {
                    let app_state_clone = app_state.read();
                    if let Err(e) = repository_context.update_from_ide_state(&app_state_clone).await {
                        tracing::warn!("Failed to initialize repository context: {}", e);
                    } else {
                        tracing::info!("✅ Repository context initialized successfully");
                    }
                }
            }
        } else {
            // Fallback to current working directory only if no File Explorer root
            let current_dir = std::env::current_dir().map_err(|e| {
                anyhow::anyhow!("Failed to get current working directory: {}", e)
            })?;
            
            tracing::info!("🔍 No File Explorer root found, using current working directory: {}", current_dir.display());
            
            // Check if current directory looks like a repository
            if current_dir.join("Cargo.toml").exists() || 
               current_dir.join("package.json").exists() || 
               current_dir.join(".git").exists() ||
               current_dir.join("pyproject.toml").exists() ||
               current_dir.join("go.mod").exists() {
                
                tracing::info!("🎯 Repository detected! Setting up context for: {}", current_dir.display());
                
                // Note: App state updates moved to avoid reactive scope conflicts
                let project_name = current_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Current Project")
                    .to_string();
                    
                tracing::info!("📝 Repository project detected: {}", project_name);
                
                // Update repository context immediately
                {
                    let app_state_clone = app_state.read();
                    if let Err(e) = repository_context.update_from_ide_state(&app_state_clone).await {
                        tracing::warn!("Failed to initialize repository context: {}", e);
                    } else {
                        tracing::info!("✅ Repository context initialized successfully");
                    }
                }
            } else {
                tracing::warn!("⚠️ No repository detected in current directory. AI Helper will ask user for clarification when needed.");
            }
        }

        // Create engine without checking keys - we'll check when processing
        let mut engine = ConsensusEngine::new(Some(db)).await?;
        
        // Connect repository context to engine
        engine.set_repository_context(repository_context.clone()).await?;

        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            repository_context,
            app_state,
            current_cancellation_token: Arc::new(Mutex::new(None)),
        })
    }

    /// Check if the consensus manager has valid API keys
    pub async fn has_valid_keys(&self) -> bool {
        ApiKeyManager::has_valid_keys().await.unwrap_or(false)
    }

    /// Update repository context from current IDE state
    pub async fn update_repository_context(&self) -> Result<()> {
        let app_state = self.app_state.read();
        tracing::info!("Updating repository context from IDE state...");
        
        if let Some(project_info) = &app_state.current_project {
            tracing::info!("Current project: {} at {}", project_info.name, project_info.root_path.display());
        } else {
            tracing::warn!("No current project in IDE state");
        }
        
        if let Some(selected_file) = &app_state.file_explorer.selected_file {
            tracing::info!("Selected file: {}", selected_file.display());
        } else {
            tracing::info!("No file selected in file explorer");
        }
        
        self.repository_context.update_from_ide_state(&app_state).await?;
        
        // Log the context after update
        let context = self.repository_context.get_context().await;
        tracing::info!("Repository context after update - Root: {:?}, Active file: {:?}", 
                      context.root_path, context.active_file);
        
        Ok(())
    }

    /// Process a query with UI updates and streaming
    pub async fn process_query_streaming(
        &mut self,
        query: &str,
    ) -> Result<(String, mpsc::UnboundedReceiver<ConsensusUIEvent>)> {
        // Check if we have valid API keys before processing
        if !ApiKeyManager::has_valid_keys().await? {
            return Err(anyhow::anyhow!(
                "No valid OpenRouter API key configured. Please configure in Settings."
            ));
        }

        // Update repository context from current IDE state before processing
        if let Err(e) = self.update_repository_context().await {
            tracing::warn!("Failed to update repository context: {}", e);
        }

        // The consensus engine will automatically load the active profile from database
        // when processing the query, so we don't need to reload it here
        
        // Get current profile from consensus engine to update UI with actual models
        // This will be the cached profile, but the engine will update it when processing
        let profile = {
            let engine = self.engine.lock().await;
            engine.get_current_profile().await
        };
        
        // Start consensus in UI with actual profile models
        {
            let mut state = self.app_state.write();
            state.consensus.start_consensus();
            
            // Update stages with actual models from the profile
            // The engine will load the latest profile from DB and update these during processing
            state.consensus.stages = vec![
                StageInfo::new("Generator", &profile.generator_model),
                StageInfo::new("Refiner", &profile.refiner_model),
                StageInfo::new("Validator", &profile.validator_model),
                StageInfo::new("Curator", &profile.curator_model),
            ];
            state.consensus.active_profile_name = profile.profile_name.clone();
        }

        // Create two channels - one for streaming events to return, one for internal processing
        let (tx_stream, rx_stream) = mpsc::unbounded_channel();
        let (tx_internal, rx_internal) = mpsc::unbounded_channel();

        // Clone internal sender for completion event before moving it
        let internal_sender_for_completion = tx_internal.clone();

        // Spawn task to process events internally
        let app_state_events = self.app_state.clone();
        let manager_for_events = Some(self.clone());
        dioxus::prelude::spawn(async move {
            process_consensus_events(rx_internal, app_state_events, manager_for_events).await;
        });

        // Create atomic bool for auto-accept state
        let auto_accept_enabled = Arc::new(std::sync::atomic::AtomicBool::new(
            self.app_state.read().auto_accept
        ));
        
        // Generate conversation ID
        let conversation_id = crate::core::database::generate_id();
        
        // Create callbacks that send to both channels
        let callbacks = Arc::new(DualChannelCallbacks {
            stream_sender: tx_stream,
            internal_sender: tx_internal,
            auto_accept_enabled,
            conversation_id: conversation_id.clone(),
            question: query.to_string(),
            stages_completed: Arc::new(Mutex::new(Vec::new())),
        });

        // Create cancellation token for this consensus operation
        let cancellation_token = CancellationToken::new();
        
        // Store the cancellation token so it can be cancelled
        {
            let mut token_guard = self.current_cancellation_token.lock().await;
            *token_guard = Some(cancellation_token.clone());
        }

        // Clone what we need for the async task
        let engine = self.engine.clone();
        let query = query.to_string();
        let callbacks_clone = callbacks.clone();
        let cancellation_token_clone = cancellation_token.clone();

        // Get user_id from app state
        let user_id = self.app_state.read().user_id.clone();
        
        // Clone cancellation token manager to clear it when done
        let token_manager = self.current_cancellation_token.clone();

        // Process consensus with minimal lock holding
        let result = {
            // Create a new pipeline for this request to avoid lock contention
            // The engine only needs to be locked briefly to get configuration
            let (api_key, profile, database, repo_context, ai_helpers) = {
                let engine = engine.lock().await;
                engine.get_pipeline_config().await
            }; // Lock released here!

            // Create a fresh pipeline without holding the engine lock
            let config = ConsensusConfig {
                enable_streaming: true,
                show_progress: true,
                timeout_seconds: 300,
                retry_policy: crate::consensus::types::RetryPolicy::default(),
                context_injection: crate::consensus::types::ContextInjectionStrategy::Smart,
            };
            
            let mut pipeline = ConsensusPipeline::new(
                config,
                profile,
                api_key,
            );

            // Configure the pipeline
            if let Some(db) = database {
                pipeline = pipeline.with_database(db);
            }

            if let Some(repo_ctx) = repo_context {
                pipeline = pipeline.with_repository_context(repo_ctx);
            }

            if let Some(helpers) = ai_helpers {
                pipeline = pipeline.with_ai_helpers(helpers);
            }

            // Initialize consensus memory for AI helpers
            if let Err(e) = pipeline.initialize_consensus_memory().await {
                tracing::warn!("Failed to initialize consensus memory: {}", e);
            }
            
            // Initialize mode detection for AI-based routing
            pipeline = pipeline.with_mode_detection().await
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to initialize mode detection: {}, continuing without it", e);
                    panic!("Mode detection initialization failed - this should not happen as with_mode_detection always returns Ok(self)")
                });

            // Now process without holding any locks
            let result = pipeline
                .with_callbacks(callbacks_clone)
                .run_with_cancellation(&query, None, user_id.clone(), cancellation_token_clone)
                .await;

            // Clear the cancellation token when done
            {
                let mut token_guard = token_manager.lock().await;
                *token_guard = None;
            }

            result.map(|r| {
                r.result
                    .unwrap_or_else(|| "No response received".to_string())
            })
        };

        // Process result
        match result {
            Ok(result) => {
                // Only send completion event if consensus completed successfully
                let _ = internal_sender_for_completion.send(ConsensusUIEvent::Completed);
                
                // Clear cached profile to force reload from database on next query
                // This ensures profile changes are picked up immediately
                {
                    let engine = self.engine.lock().await;
                    engine.clear_cached_profile().await;
                }

                Ok((result, rx_stream))
            }
            Err(e) => {
                // Check if this is a cancellation error
                let error_string = e.to_string();
                if error_string.contains("cancelled") || error_string.contains("Cancelled") {
                    // Send Cancelled event to update UI
                    let _ = internal_sender_for_completion.send(ConsensusUIEvent::Cancelled {
                        reason: "User cancelled".to_string(),
                    });
                    
                    tracing::info!("Consensus was cancelled");
                    
                    // Clear cached profile to do a full reset
                    {
                        let engine = self.engine.lock().await;
                        engine.clear_cached_profile().await;
                        tracing::info!("Cleared cached profile after cancellation for full reset");
                    }
                    
                    // Return empty result for cancelled consensus
                    Ok(("".to_string(), rx_stream))
                } else {
                    // For other errors, propagate them
                    Err(e)
                }
            }
        }
    }

    /// Process a query with UI updates
    pub async fn process_query(&mut self, query: &str) -> Result<String> {
        let (result, _rx) = self.process_query_streaming(query).await?;
        Ok(result)
    }

    /// Cancel the currently running consensus
    pub async fn cancel_consensus(&mut self, reason: &str) -> Result<()> {
        let mut token_guard = self.current_cancellation_token.lock().await;
        if let Some(token) = token_guard.as_ref() {
            token.cancel(CancellationReason::UserRequested);
            tracing::info!("Consensus cancelled by user: {}", reason);
            
            // Clear cached profile to force reset
            {
                let engine = self.engine.lock().await;
                engine.clear_cached_profile().await;
            }
            
            // Clear the token since consensus is cancelled
            *token_guard = None;
            
            Ok(())
        } else {
            // Even if no token, clear cached profile for clean state
            {
                let engine = self.engine.lock().await;
                engine.clear_cached_profile().await;
            }
            Err(anyhow::anyhow!("No consensus operation is currently running"))
        }
    }

    /// Check if consensus is currently running and can be cancelled
    pub async fn can_cancel(&self) -> bool {
        let token_guard = self.current_cancellation_token.lock().await;
        token_guard.is_some()
    }
    
    /// Update repository context with a specific path (for hive-consensus GUI compatibility)
    pub async fn update_repository_context_with_path(&self, path: PathBuf) -> Result<()> {
        use crate::consensus::repository_context::RepositoryContextManager;
        
        tracing::info!("Updating repository context with path: {}", path.display());
        
        // Create a new repository context manager
        let repository_context_manager = Arc::new(RepositoryContextManager::new().await?);
        
        // If it's a directory, analyze it
        if path.is_dir() {
            tracing::info!("Analyzing repository at: {}", path.display());
            
            // Trigger repository analysis
            if let Err(e) = repository_context_manager.analyze_repository_async(&path).await {
                tracing::warn!("Failed to analyze repository: {}", e);
            }
        }
        
        // Set the repository context on the consensus engine
        let mut engine = self.engine.lock().await;
        engine.set_repository_context(repository_context_manager).await?;
        
        tracing::info!("✅ Repository context updated with path: {}", path.display());
        Ok(())
    }

    /// Get the consensus engine for direct access
    pub fn engine(&self) -> Arc<Mutex<ConsensusEngine>> {
        self.engine.clone()
    }

    /// Reload the active profile from the database
    pub async fn reload_active_profile(&self) -> Result<()> {
        use crate::core::database::get_database;
        
        // Get the active profile ID from consensus_settings
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        let active_profile_id: Option<String> = conn.query_row(
            "SELECT value FROM consensus_settings WHERE key = 'active_profile_id'",
            [],
            |row| row.get(0)
        ).optional()?;
        
        if let Some(profile_id) = active_profile_id {
            // Get the profile name
            let profile_name: String = conn.query_row(
                "SELECT profile_name FROM consensus_profiles WHERE id = ?1",
                rusqlite::params![profile_id],
                |row| row.get(0)
            )?;
            
            // Set the profile in the consensus engine
            let engine = self.engine.lock().await;
            engine.set_profile(&profile_name).await?;
            
            // Get the updated profile to update UI
            let profile = engine.get_current_profile().await;
            drop(engine); // Release the lock before updating UI
            
            // Update UI stages with the new profile models
            let mut app_state = self.app_state;
            app_state.write().consensus.stages = vec![
                StageInfo::new("Generator", &profile.generator_model),
                StageInfo::new("Refiner", &profile.refiner_model),
                StageInfo::new("Validator", &profile.validator_model),
                StageInfo::new("Curator", &profile.curator_model),
            ];
            
            tracing::info!("Reloaded active profile: {} and updated UI", profile_name);
        } else {
            tracing::warn!("No active profile found in consensus_settings");
        }
        
        Ok(())
    }
    
    /// Execute approved file operations
    pub async fn execute_approved_operations(
        &self,
        operations: Vec<crate::consensus::stages::file_aware_curator::FileOperation>,
    ) -> Result<()> {
        use crate::consensus::{FileOperationExecutor, ExecutorConfig, SmartDecisionEngine, OperationIntelligenceCoordinator};
        use crate::consensus::smart_decision_engine::UserPreferences;
        use crate::ai_helpers::AIHelperEcosystem;
        use std::sync::Arc;
        
        // Execute the operations
        tracing::info!("Executing {} approved operations", operations.len());
        
        // Get the engine to access AI helpers and database
        let engine = self.engine.lock().await;
        
        // Get AI helpers from engine if available
        let ai_helpers = engine.get_ai_helpers().await;
        
        if let Some(ai_helpers) = ai_helpers {
            // Create user preferences with safe defaults
            let user_prefs = UserPreferences {
                risk_tolerance: 0.3,
                auto_backup: true,
                require_confirmation_for_deletions: true,
                require_confirmation_for_mass_updates: true,
                trust_ai_suggestions: 0.7,
                preferred_mode: crate::consensus::operation_analysis::AutoAcceptMode::Conservative,
                custom_rules: vec![],
            };
            
            // Create decision engine (without history database for now)
            let decision_engine = SmartDecisionEngine::new(
                crate::consensus::operation_analysis::AutoAcceptMode::Conservative,
                user_prefs,
                None, // No history database for manual execution
            );
            
            // Create intelligence coordinator with individual AI helper components
            let intelligence_coordinator = OperationIntelligenceCoordinator::new(
                ai_helpers.knowledge_indexer.clone(),
                ai_helpers.context_retriever.clone(),
                ai_helpers.pattern_recognizer.clone(),
                ai_helpers.quality_analyzer.clone(),
                ai_helpers.knowledge_synthesizer.clone(),
            );
            
            // Create executor config
            let executor_config = ExecutorConfig {
                create_backups: true,
                validate_syntax: true,
                dry_run_mode: false,
                max_file_size: 10 * 1024 * 1024, // 10MB
                allowed_extensions: vec![
                    "rs", "js", "ts", "py", "java", "cpp", "c", "h", "hpp",
                    "go", "rb", "php", "md", "txt", "json", "yaml", "yml",
                    "toml", "html", "css", "scss", "swift", "kt", "scala",
                    "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd",
                ].into_iter().map(String::from).collect(),
                forbidden_paths: vec![], // Can be configured
                stop_on_error: true,
            };
            
            // Create file executor
            let file_executor = FileOperationExecutor::new(
                executor_config,
                decision_engine,
                intelligence_coordinator,
            );
            
            // Get repository context
            let repo_context = self.repository_context.get_context().await;
            let repository_path = repo_context.root_path.unwrap_or_else(|| std::path::PathBuf::from("."));
            
            // Create operation context
            let operation_context = crate::consensus::operation_analysis::OperationContext {
                repository_path,
                user_question: "Manual execution of approved operations".to_string(),
                consensus_response: "User-approved operations".to_string(),
                timestamp: std::time::SystemTime::now(),
                session_id: "manual-execution".to_string(),
                git_commit: None,
            };
            
            // Execute the operations
            match file_executor.execute_operations_batch(operations, &operation_context).await {
                Ok(results) => {
                    let successful = results.iter().filter(|r| r.success).count();
                    let failed = results.iter().filter(|r| !r.success).count();
                    
                    tracing::info!(
                        "File operations executed: {} successful, {} failed",
                        successful,
                        failed
                    );
                    
                    // Log detailed results
                    for result in &results {
                        if result.success {
                            tracing::info!("✅ {}: {}", result.message, result.files_affected.iter()
                                .map(|p| p.display().to_string())
                                .collect::<Vec<_>>()
                                .join(", "));
                        } else {
                            tracing::error!("❌ Failed: {} - {}", 
                                result.message,
                                result.error_message.as_deref().unwrap_or("Unknown error"));
                        }
                    }
                    
                    if failed > 0 {
                        tracing::warn!(
                            "⚠️ {} file operations failed. Check logs for details.",
                            failed
                        );
                    }
                    
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to execute operations batch: {}", e);
                    Err(anyhow::anyhow!("Failed to execute operations: {}", e))
                }
            }
        } else {
            tracing::error!("AI helpers not initialized - cannot execute file operations");
            Err(anyhow::anyhow!("AI helpers not initialized. Please ensure AI helpers are configured."))
        }
    }

    /// Execute file operations from Curator output using AI Helper bridge
    pub async fn execute_curator_operations(&self, curator_output: &str) -> Result<()> {
        use crate::consensus::ai_file_executor::AIConsensusFileExecutor;
        
        tracing::info!("🤖 AI Helper bridge activated - parsing Curator output for file operations");
        
        // Get AI helpers from the engine
        let engine = self.engine.lock().await;
        if let Some(ai_helpers) = engine.get_ai_helpers().await {
            // Create the AI Helper bridge
            let ai_executor = AIConsensusFileExecutor::new((*ai_helpers).clone());
            
            // Execute operations from Curator output
            match ai_executor.execute_from_curator(curator_output).await {
                Ok(report) => {
                    tracing::info!(
                        "🎉 AI Helper execution successful: {} of {} operations completed",
                        report.operations_completed,
                        report.operations_total
                    );
                    
                    // Log file operations
                    for file in &report.files_created {
                        tracing::info!("📄 Created: {}", file.display());
                    }
                    for file in &report.files_modified {
                        tracing::info!("✏️ Modified: {}", file.display());
                    }
                    
                    if !report.errors.is_empty() {
                        tracing::warn!("⚠️ Errors occurred during execution:");
                        for error in &report.errors {
                            tracing::warn!("  - {}", error);
                        }
                    }
                    
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("❌ AI Helper execution failed: {}", e);
                    Err(e)
                }
            }
        } else {
            tracing::error!("AI helpers not available in consensus engine");
            Err(anyhow::anyhow!("AI helpers not initialized in consensus engine"))
        }
    }
}

/// Hook to use consensus in components
pub fn use_consensus() -> Option<DesktopConsensusManager> {
    use_consensus_with_version(0)
}

/// Hook to use consensus with version tracking for re-initialization
pub fn use_consensus_with_version(api_keys_version: u32) -> Option<DesktopConsensusManager> {
    let app_state = use_context::<Signal<AppState>>();

    let resource = use_resource(move || async move {
        // Version is used to force re-evaluation when API keys change
        let _version = api_keys_version;
        
        tracing::info!("Creating consensus manager with API keys version: {}", api_keys_version);

        // First check if we have valid API keys
        match ApiKeyManager::has_valid_keys().await {
            Ok(true) => {
                // Keys exist, try to create manager
                match DesktopConsensusManager::new(app_state).await {
                    Ok(manager) => {
                        tracing::info!("Successfully created consensus manager with API keys");
                        // No need to reload profile here - the engine will always load from DB
                        Some(manager)
                    }
                    Err(e) => {
                        tracing::error!("Failed to create consensus manager: {}", e);
                        tracing::error!("Error details: {:?}", e);
                        None
                    }
                }
            }
            Ok(false) => {
                tracing::info!("No valid API keys found, consensus manager not created");
                None
            }
            Err(e) => {
                tracing::error!("Failed to check API keys: {}", e);
                None
            }
        }
    });

    let result = resource.read().as_ref().and_then(|r| r.as_ref().cloned());
    result
}
