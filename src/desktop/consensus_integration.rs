//! Consensus Engine Integration for Desktop App
//!
//! This module connects the consensus engine to the desktop UI,
//! providing real-time updates and progress tracking.

use crate::consensus::{
    engine::ConsensusEngine,
    streaming::{StreamingCallbacks, ProgressInfo},
    types::Stage,
};
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
    
    fn on_stage_chunk(&self, _stage: Stage, _chunk: &str, _total_content: &str) -> Result<()> {
        // For desktop, we might want to show streaming text in the future
        // For now, just update token count
        let _ = self.event_sender.send(ConsensusUIEvent::TokenUpdate {
            count: 1,
            cost: 0.0001,
        });
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
        let engine = ConsensusEngine::new(None).await?;
        
        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            app_state,
        })
    }
    
    /// Process a query with UI updates
    pub async fn process_query(&mut self, query: &str) -> Result<String> {
        // Start consensus in UI
        self.app_state.write().consensus.start_consensus();
        
        // Create event channel
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Spawn task to process events
        let app_state_events = self.app_state.clone();
        dioxus::prelude::spawn(async move {
            process_consensus_events(rx, app_state_events).await;
        });
        
        // Create callbacks
        let callbacks = Arc::new(DesktopStreamingCallbacks::new(tx));
        
        // Process with streaming
        let engine = self.engine.lock().await;
        let result = engine.process_with_callbacks(query, None, callbacks).await?;
        
        // Complete consensus in UI
        self.app_state.write().consensus.complete_consensus();
        
        Ok(result.result.unwrap_or_else(|| "No response received".to_string()))
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
        DesktopConsensusManager::new(app_state).await.ok()
    });
    
    let result = resource.read().as_ref().and_then(|r| r.as_ref().cloned());
    result
}