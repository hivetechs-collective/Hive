//! Consensus Engine Integration for Desktop App
//!
//! This module connects the consensus engine to the desktop UI,
//! providing real-time updates and progress tracking.

use crate::consensus::{
    engine::ConsensusEngine,
    repository_context::RepositoryContextManager,
    streaming::{ProgressInfo, StreamingCallbacks},
    types::Stage,
    cancellation::{CancellationToken, CancellationReason},
};
use crate::core::api_keys::ApiKeyManager;
use crate::desktop::state::{AppState, ConsensusStage, StageInfo, StageStatus};
use anyhow::Result;
use dioxus::prelude::*;
use rusqlite::OptionalExtension;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

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
}

impl DesktopStreamingCallbacks {
    pub fn new(event_sender: mpsc::UnboundedSender<ConsensusUIEvent>, auto_accept_enabled: Arc<std::sync::atomic::AtomicBool>) -> Self {
        Self { event_sender, auto_accept_enabled }
    }
}

/// Dual channel callbacks that send to both streaming and internal channels
pub struct DualChannelCallbacks {
    stream_sender: mpsc::UnboundedSender<ConsensusUIEvent>,
    internal_sender: mpsc::UnboundedSender<ConsensusUIEvent>,
    auto_accept_enabled: Arc<std::sync::atomic::AtomicBool>,
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
                if !state.consensus.streaming_content.is_empty() {
                    state.consensus.streaming_content.push_str("\n\n---\n\n");
                }
                state
                    .consensus
                    .streaming_content
                    .push_str(&format!("## 📝 {} Stage\n\n", stage_name));

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
                    state.analytics_refresh_trigger += 1;
                    tracing::info!("Consensus completed - triggered analytics refresh");
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
                chunk,
                total_content: _,
            } => {
                // Update the streaming content in the consensus state
                let mut state = app_state.write();
                state.consensus.streaming_content.push_str(&chunk);
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
                state.consensus.current_stage = None;
                state.consensus.is_running = false;
                state.consensus.is_active = false;
                
                // Reset all stage states by calling complete_consensus which resets everything
                state.consensus.complete_consensus();
                
                tracing::info!("Consensus cancelled in UI: {}", reason);
            }
            ConsensusUIEvent::Completed => {
                // Handle consensus completion
                let mut state = app_state.write();
                state.consensus.complete_consensus();
                tracing::info!("✅ Consensus completed successfully");
            }
            ConsensusUIEvent::OperationsRequireConfirmation { operations } => {
                // Handle operations that need user confirmation
                tracing::info!("📁 {} operations require user confirmation", operations.len());
                
                // Store operations in app state for the dialog to display
                let mut state = app_state.write();
                state.pending_operations = Some(operations);
                state.show_operation_confirmation_dialog = true;
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
        dioxus::prelude::spawn(async move {
            process_consensus_events(rx_internal, app_state_events).await;
        });

        // Create atomic bool for auto-accept state
        let auto_accept_enabled = Arc::new(std::sync::atomic::AtomicBool::new(
            self.app_state.read().auto_accept
        ));
        
        // Create callbacks that send to both channels
        let callbacks = Arc::new(DualChannelCallbacks {
            stream_sender: tx_stream,
            internal_sender: tx_internal,
            auto_accept_enabled,
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

        // Spawn the consensus processing
        let handle = tokio::spawn(async move {
            let engine = engine.lock().await;

            // D1 authorization is now sent immediately when received in the pipeline
            // No need to check here as it would be outdated by the time consensus completes

            let result = engine
                .process_with_callbacks_and_cancellation(&query, None, callbacks_clone, user_id, cancellation_token_clone)
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
        });

        // Wait for result
        match handle.await? {
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
            
            // Clear the token since consensus is cancelled
            *token_guard = None;
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("No consensus operation is currently running"))
        }
    }

    /// Check if consensus is currently running and can be cancelled
    pub async fn can_cancel(&self) -> bool {
        let token_guard = self.current_cancellation_token.lock().await;
        token_guard.is_some()
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
                mass_update_threshold: 5,
                preferred_language: "en".to_string(),
                timezone: "UTC".to_string(),
            };
            
            // Create decision engine
            let decision_engine = SmartDecisionEngine::new(
                crate::consensus::operation_analysis::AutoAcceptMode::Conservative,
                user_prefs,
                ai_helpers.clone(),
            );
            
            // Create intelligence coordinator
            let intelligence_coordinator = OperationIntelligenceCoordinator::new(ai_helpers);
            
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
