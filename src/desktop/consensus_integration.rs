//! Consensus Engine Integration for Desktop App
//!
//! This module connects the consensus engine to the desktop UI,
//! providing real-time updates and progress tracking.

use crate::consensus::{
    engine::ConsensusEngine,
    streaming::{StreamingCallbacks, ProgressInfo},
    types::Stage,
};
use crate::core::api_keys::ApiKeyManager;
use crate::desktop::state::{AppState, ConsensusStage, StageStatus};
use anyhow::Result;
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

/// Events sent from callbacks to UI
#[derive(Debug, Clone)]
pub enum ConsensusUIEvent {
    StageStarted { stage: ConsensusStage, model: String },
    StageProgress { stage: ConsensusStage, percentage: u8, tokens: usize },
    StageCompleted { stage: ConsensusStage, cost: f64 },
    StageError { stage: ConsensusStage, error: String },
    TokenUpdate { count: usize, cost: f64 },
    StreamingChunk { stage: ConsensusStage, chunk: String, total_content: String },
    D1Authorization { total_remaining: u32 }, // New event for D1 auth info
}

/// Desktop streaming callbacks that send events to UI
pub struct DesktopStreamingCallbacks {
    event_sender: mpsc::UnboundedSender<ConsensusUIEvent>,
}

impl DesktopStreamingCallbacks {
    pub fn new(event_sender: mpsc::UnboundedSender<ConsensusUIEvent>) -> Self {
        Self { event_sender }
    }
}

/// Dual channel callbacks that send to both streaming and internal channels
pub struct DualChannelCallbacks {
    stream_sender: mpsc::UnboundedSender<ConsensusUIEvent>,
    internal_sender: mpsc::UnboundedSender<ConsensusUIEvent>,
}

impl StreamingCallbacks for DesktopStreamingCallbacks {
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
    
    fn on_stage_chunk(&self, stage: Stage, chunk: &str, total_content: &str) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };
        
        tracing::debug!("DesktopStreamingCallbacks: Sending chunk for stage {:?}: '{}'", stage, chunk);
        
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
            tracing::debug!("Successfully sent streaming chunk to UI channel for stage {:?}", stage);
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
    
    fn on_stage_complete(&self, stage: Stage, result: &crate::consensus::types::StageResult) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };
        
        let cost = if let Some(usage) = &result.usage {
            usage.total_tokens as f64 * 0.0001
        } else {
            0.0
        };
        
        let _ = self.event_sender.send(ConsensusUIEvent::StageCompleted {
            stage: consensus_stage,
            cost,
        });
        
        Ok(())
    }
    
    fn on_error(&self, stage: Stage, error: &anyhow::Error) -> Result<()> {
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
}

impl StreamingCallbacks for DualChannelCallbacks {
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
    
    fn on_stage_complete(&self, stage: Stage, result: &crate::consensus::types::StageResult) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };
        
        let cost = if let Some(usage) = &result.usage {
            usage.total_tokens as f64 * 0.0001
        } else {
            0.0
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
}

/// Process consensus UI events and update app state
pub async fn process_consensus_events(
    mut event_receiver: mpsc::UnboundedReceiver<ConsensusUIEvent>,
    mut app_state: Signal<AppState>,
) {
    while let Some(event) = event_receiver.recv().await {
        match event {
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
                state.consensus.streaming_content.push_str(&format!("## 📝 {} Stage\n\n", stage_name));
                
                let stage_index = match stage {
                    ConsensusStage::Generator => 0,
                    ConsensusStage::Refiner => 1,
                    ConsensusStage::Validator => 2,
                    ConsensusStage::Curator => 3,
                };
                
                if let Some(stage_info) = state.consensus.stages.get_mut(stage_index) {
                    stage_info.status = StageStatus::Running;
                    stage_info.model = model;
                }
            }
            ConsensusUIEvent::StageProgress { stage, percentage, tokens } => {
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
            }
            ConsensusUIEvent::StageError { stage, error: _ } => {
                let mut state = app_state.write();
                
                let stage_index = match stage {
                    ConsensusStage::Generator => 0,
                    ConsensusStage::Refiner => 1,
                    ConsensusStage::Validator => 2,
                    ConsensusStage::Curator => 3,
                };
                
                if let Some(stage_info) = state.consensus.stages.get_mut(stage_index) {
                    stage_info.status = StageStatus::Error;
                }
            }
            ConsensusUIEvent::TokenUpdate { count, cost } => {
                let mut state = app_state.write();
                state.consensus.add_tokens(count, cost);
            }
            ConsensusUIEvent::StreamingChunk { stage: _, chunk, total_content: _ } => {
                // Update the streaming content in the consensus state
                let mut state = app_state.write();
                state.consensus.streaming_content.push_str(&chunk);
            }
            ConsensusUIEvent::D1Authorization { total_remaining } => {
                // Update app state with D1 authorization info
                let mut state = app_state.write();
                state.total_conversations_remaining = Some(total_remaining);
                tracing::info!("Updated total conversations remaining: {}", total_remaining);
            }
        }
    }
}

/// Consensus manager that integrates with the desktop app
#[derive(Clone)]
pub struct DesktopConsensusManager {
    engine: Arc<Mutex<ConsensusEngine>>,
    app_state: Signal<AppState>,
}

impl DesktopConsensusManager {
    /// Create a new desktop consensus manager
    pub async fn new(app_state: Signal<AppState>) -> Result<Self> {
        // Check if we have valid API keys first
        if !ApiKeyManager::has_valid_keys().await? {
            return Err(anyhow::anyhow!(
                "No valid OpenRouter API key configured. Please configure in Settings."
            ));
        }
        
        // Use unified database manager for desktop app
        use crate::core::database::{DatabaseManager, DatabaseConfig};
        let db = Arc::new(DatabaseManager::new(DatabaseConfig::default()).await?);
        
        let engine = ConsensusEngine::new(Some(db)).await?;
        
        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            app_state,
        })
    }
    
    /// Process a query with UI updates and streaming
    pub async fn process_query_streaming(&mut self, query: &str) -> Result<(String, mpsc::UnboundedReceiver<ConsensusUIEvent>)> {
        // Start consensus in UI
        self.app_state.write().consensus.start_consensus();
        
        // Create two channels - one for streaming events to return, one for internal processing
        let (tx_stream, rx_stream) = mpsc::unbounded_channel();
        let (tx_internal, rx_internal) = mpsc::unbounded_channel();
        
        // Spawn task to process events internally
        let app_state_events = self.app_state.clone();
        dioxus::prelude::spawn(async move {
            process_consensus_events(rx_internal, app_state_events).await;
        });
        
        // Create callbacks that send to both channels
        let callbacks = Arc::new(DualChannelCallbacks {
            stream_sender: tx_stream,
            internal_sender: tx_internal,
        });
        
        // Clone what we need for the async task
        let engine = self.engine.clone();
        let query = query.to_string();
        let callbacks_clone = callbacks.clone();
        
        // Get user_id from app state
        let user_id = self.app_state.read().user_id.clone();
        
        // Clone app state for D1 info update
        let app_state_d1 = self.app_state.clone();
        
        // Spawn the consensus processing
        let handle = tokio::spawn(async move {
            let engine = engine.lock().await;
            
            // First check the current D1 authorization info
            if let Ok(auth_info) = engine.get_last_authorization_info().await {
                if let Some(remaining) = auth_info {
                    // Send D1 authorization event
                    let _ = callbacks_clone.internal_sender.send(ConsensusUIEvent::D1Authorization { 
                        total_remaining: remaining 
                    });
                }
            }
            
            let result = engine.process_with_callbacks(&query, None, callbacks_clone, user_id).await;
            
            result.map(|r| r.result.unwrap_or_else(|| "No response received".to_string()))
        });
        
        // Wait for result
        let result = handle.await??;
        
        // Complete consensus in UI after the task completes
        self.app_state.write().consensus.complete_consensus();
        
        Ok((result, rx_stream))
    }
    
    /// Process a query with UI updates
    pub async fn process_query(&mut self, query: &str) -> Result<String> {
        let (result, _rx) = self.process_query_streaming(query).await?;
        Ok(result)
    }
    
    /// Get the consensus engine for direct access
    pub fn engine(&self) -> Arc<Mutex<ConsensusEngine>> {
        self.engine.clone()
    }
}

/// Hook to use consensus in components
pub fn use_consensus() -> Option<DesktopConsensusManager> {
    let app_state = use_context::<Signal<AppState>>();
    
    let resource = use_resource(move || async move {
        match DesktopConsensusManager::new(app_state).await {
            Ok(manager) => {
                tracing::info!("Successfully created consensus manager");
                Some(manager)
            }
            Err(e) => {
                tracing::error!("Failed to create consensus manager: {}", e);
                tracing::error!("Error details: {:?}", e);
                None
            }
        }
    });
    
    let result = resource.read().as_ref().and_then(|r| r.as_ref().cloned());
    result
}