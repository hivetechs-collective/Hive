//! Minimal wrapper to run ConsensusEngine on Tokio thread
//!
//! This follows the CONSENSUS_ARCHITECTURE_2025.md design:
//! - Extracts the Send-able ConsensusEngine from DesktopConsensusManager
//! - Runs consensus on Tokio thread, UI updates via channels
//! - Keeps ALL existing consensus logic intact

use crate::consensus::cancellation::CancellationToken;
use crate::consensus::engine::ConsensusEngine;
use crate::consensus::pipeline::ConsensusPipeline;
use crate::consensus::repository_context::RepositoryContextManager;
use crate::consensus::streaming::StreamingCallbacks;
use crate::consensus::types::{ConsensusConfig, ConsensusProfile};
use crate::core::api_keys::ApiKeyManager;
use crate::desktop::consensus_integration::{ConsensusUIEvent, DesktopConsensusManager};
use crate::desktop::state::AppState;
use anyhow::Result;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{error, info, warn};

/// Wrapper that runs ConsensusEngine (Send) on Tokio thread
#[derive(Clone)]
pub struct ConsensusThreadWrapper {
    /// Channel to send commands to Tokio thread
    cmd_tx: mpsc::UnboundedSender<ConsensusCommand>,
}

enum ConsensusCommand {
    ProcessQuery {
        query: String,
        reply: mpsc::UnboundedSender<Result<(String, mpsc::UnboundedReceiver<ConsensusUIEvent>)>>,
    },
    Cancel(String),
    Shutdown,
}

impl ConsensusThreadWrapper {
    /// Create wrapper and spawn ConsensusEngine on Tokio
    pub async fn new(_app_state: Signal<AppState>) -> Result<Self> {
        info!("🚀 Creating ConsensusThreadWrapper - extracting Send-able engine");

        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();

        // Get database for consensus engine
        let db = crate::core::database::get_database().await?;

        // Spawn ConsensusEngine on Tokio thread (the key fix!)
        tokio::spawn(async move {
            info!("🎯 ConsensusEngine running on Tokio thread - UI stays responsive!");

            // Create engine WITHOUT Signal<AppState> (which is non-Send)
            let mut engine = match ConsensusEngine::new(Some(db)).await {
                Ok(e) => e,
                Err(e) => {
                    error!("Failed to create ConsensusEngine: {}", e);
                    return;
                }
            };

            let repository_context = match RepositoryContextManager::new().await {
                Ok(r) => Arc::new(r),
                Err(e) => {
                    error!("Failed to create RepositoryContextManager: {}", e);
                    return;
                }
            };

            if let Err(e) = engine
                .set_repository_context(repository_context.clone())
                .await
            {
                error!("Failed to set repository context: {}", e);
                return;
            }

            let mut current_cancellation: Option<CancellationToken> = None;

            info!("✅ Consensus thread ready, waiting for commands...");

            while let Some(cmd) = cmd_rx.recv().await {
                info!("📨 Received command in consensus thread");
                match cmd {
                    ConsensusCommand::ProcessQuery { query, reply } => {
                        info!("📝 Processing query on Tokio: {}", query);

                        // Cancel any existing operation
                        if let Some(token) = current_cancellation.take() {
                            token.cancel(
                                crate::consensus::cancellation::CancellationReason::UserRequested,
                            );
                        }

                        // Create new cancellation token
                        let cancellation_token = CancellationToken::new();
                        current_cancellation = Some(cancellation_token.clone());

                        // Create event channel for this query
                        let (event_tx, event_rx) = mpsc::unbounded_channel();

                        // Clone event_tx for keeping it alive
                        let event_tx_keepalive = event_tx.clone();

                        // Process using the existing engine logic
                        info!("🔄 Calling run_consensus for query: {}", query);
                        let result = Self::run_consensus(
                            &mut engine,
                            repository_context.clone(),
                            query,
                            event_tx.clone(),
                            cancellation_token,
                        )
                        .await;
                        info!("📝 run_consensus returned: {:?}", result.is_ok());

                        // Send both result and event receiver back
                        let response = match result {
                            Ok(res) => {
                                info!(
                                    "✅ Sending successful result back to UI: {} chars",
                                    res.len()
                                );
                                Ok((res, event_rx))
                            }
                            Err(e) => {
                                error!("❌ Consensus failed: {}", e);
                                Err(e)
                            }
                        };
                        let _ = reply.send(response);
                        info!("📤 Response sent to UI thread");

                        // Keep the event channel alive for a bit to ensure all events are received
                        tokio::spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            drop(event_tx_keepalive); // Explicitly drop after delay
                            info!("Event channel closed after delay");
                        });
                    }
                    ConsensusCommand::Cancel(reason) => {
                        info!("🛑 Cancelling consensus: {}", reason);
                        if let Some(token) = current_cancellation.as_ref() {
                            token.cancel(
                                crate::consensus::cancellation::CancellationReason::UserRequested,
                            );
                        }
                    }
                    ConsensusCommand::Shutdown => {
                        info!("🔚 Shutting down consensus thread gracefully");
                        break;
                    }
                }
            }

            info!("✅ Consensus thread ended cleanly");
        });

        Ok(Self { cmd_tx })
    }

    /// Process query - sends to Tokio thread
    pub async fn process_query(
        &self,
        query: String,
    ) -> Result<(String, mpsc::UnboundedReceiver<ConsensusUIEvent>)> {
        info!("🎯 Sending query to Tokio thread (UI stays responsive!)");

        let (reply_tx, mut reply_rx) = mpsc::unbounded_channel();

        info!("📤 Sending ProcessQuery command to consensus thread");

        if self.cmd_tx.is_closed() {
            error!("❌ Command channel is already closed!");
            return Err(anyhow::anyhow!("Consensus thread not running"));
        }

        self.cmd_tx.send(ConsensusCommand::ProcessQuery {
            query: query.clone(),
            reply: reply_tx,
        })?;

        info!("⏳ Waiting for response from consensus thread...");

        // Wait for result and event receiver from the Tokio thread
        let (result, event_rx) = reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Consensus thread disconnected"))??;

        // Return result and the event receiver from the consensus thread
        Ok((result, event_rx))
    }

    /// Run consensus on the Tokio thread - EXACTLY as it was in 2bd6753
    async fn run_consensus(
        engine: &mut ConsensusEngine,
        repository_context: Arc<RepositoryContextManager>,
        query: String,
        event_tx: mpsc::UnboundedSender<ConsensusUIEvent>,
        cancellation_token: CancellationToken,
    ) -> Result<String> {
        // This is the EXACT code from 2bd6753:src/desktop/consensus_integration.rs lines 1155-1263
        // We're just copying what worked before into our new thread architecture

        // Create a new pipeline for this request to avoid lock contention
        // The engine only needs to be locked briefly to get configuration
        let (api_key, profile, database, repo_context, ai_helpers) =
            engine.get_pipeline_config().await;

        let api_key = api_key.ok_or_else(|| anyhow::anyhow!("No API key configured"))?;

        // Send profile loaded event for UI
        let models = vec![
            profile.generator_model.clone(),
            profile.refiner_model.clone(),
            profile.validator_model.clone(),
            profile.curator_model.clone(),
        ];
        let _ = event_tx.send(ConsensusUIEvent::ProfileLoaded {
            profile_name: profile.profile_name.clone(),
            models,
        });

        // Create a fresh pipeline without holding the engine lock
        let config = ConsensusConfig {
            enable_streaming: true,
            show_progress: true,
            timeout_seconds: 300,
            retry_policy: crate::consensus::types::RetryPolicy::default(),
            context_injection: crate::consensus::types::ContextInjectionStrategy::Smart,
        };

        let mut pipeline =
            ConsensusPipeline::new(config.clone(), profile.clone(), Some(api_key.clone()));

        // Configure the pipeline
        if let Some(ref db) = database {
            pipeline = pipeline.with_database(db.clone());
        }

        if let Some(repo_ctx) = repo_context {
            pipeline = pipeline.with_repository_context(repo_ctx);
        } else {
            // Use the repository context passed to this function
            pipeline = pipeline.with_repository_context(repository_context.clone());
        }

        if let Some(ref helpers) = ai_helpers {
            pipeline = pipeline.with_ai_helpers(helpers.clone());
        }

        // Initialize consensus memory for AI helpers
        if let Err(e) = pipeline.initialize_consensus_memory().await {
            tracing::warn!("Failed to initialize consensus memory: {}", e);
        }

        // Initialize mode detection for AI-based routing - EXACTLY as in 2bd6753
        pipeline = pipeline.with_mode_detection().await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to initialize mode detection: {}, continuing without it", e);
                panic!("Mode detection initialization failed - this should not happen as with_mode_detection always returns Ok(self)")
            });

        // Create complete callbacks from 2bd6753 - restored exactly as they were
        let auto_accept_enabled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let conversation_id = crate::core::database::generate_id();

        let desktop_callbacks = Arc::new(DesktopStreamingCallbacks::new(
            event_tx.clone(),
            auto_accept_enabled,
            conversation_id,
            query.clone(),
        ));

        // Use batching to prevent CPU overload but with complete callbacks
        let batched_callbacks = Arc::new(BatchedUIEventCallbacks::new(desktop_callbacks));

        info!("🎛️ Using batched callbacks (10 FPS) to prevent CPU overload");

        // Now process without holding any locks - EXACTLY as in 2bd6753
        // Note: user_id should come from app_state but we don't have access here
        // For now use None like the database storage calls in the original code
        let result = pipeline
            .with_callbacks(batched_callbacks)
            .run_with_cancellation(&query, None, None, cancellation_token)
            .await;

        // Process result
        match result {
            Ok(r) => {
                info!("✅ Consensus pipeline completed successfully!");
                // Only send completion event if consensus completed successfully
                let _ = event_tx.send(ConsensusUIEvent::Completed);

                let response = r
                    .result
                    .unwrap_or_else(|| "No response received".to_string());
                info!("📝 Consensus response length: {} chars", response.len());
                Ok(response)
            }
            Err(e) => {
                // Check if this is a cancellation error
                let error_string = e.to_string();
                if error_string.contains("cancelled") || error_string.contains("Cancelled") {
                    // Send Cancelled event to update UI
                    let _ = event_tx.send(ConsensusUIEvent::Cancelled {
                        reason: "User cancelled".to_string(),
                    });

                    tracing::info!("Consensus was cancelled");

                    // Return empty result for cancelled consensus
                    Ok("".to_string())
                } else {
                    // For other errors, propagate them
                    Err(e)
                }
            }
        }
    }

    /// Cancel consensus
    pub async fn cancel_consensus(&self, reason: &str) -> Result<()> {
        self.cmd_tx
            .send(ConsensusCommand::Cancel(reason.to_string()))?;
        Ok(())
    }

    /// Shutdown the consensus thread
    pub fn shutdown(&self) {
        info!("🔄 Sending shutdown command to consensus thread");
        let _ = self.cmd_tx.send(ConsensusCommand::Shutdown);
    }
}

// Removed Drop implementation - we'll handle shutdown explicitly
// to avoid accidentally shutting down when cloning/reading signals

/// Batched streaming callbacks that reduce UI update frequency from 1000/sec to 10/sec
struct BatchedUIEventCallbacks {
    inner: Arc<DesktopStreamingCallbacks>,
    update_interval: Duration,
    last_update: Arc<RwLock<Instant>>,
    pending_content: Arc<RwLock<HashMap<crate::consensus::types::Stage, String>>>,
}

impl BatchedUIEventCallbacks {
    fn new(inner: Arc<DesktopStreamingCallbacks>) -> Self {
        Self {
            inner,
            update_interval: Duration::from_millis(100), // 10 FPS for UI updates (not 1000 FPS!)
            last_update: Arc::new(RwLock::new(Instant::now())),
            pending_content: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn should_send_update(&self) -> bool {
        let last_update = self.last_update.read().await;
        last_update.elapsed() >= self.update_interval
    }

    async fn mark_updated(&self) {
        let mut last_update = self.last_update.write().await;
        *last_update = Instant::now();
    }
}

impl StreamingCallbacks for BatchedUIEventCallbacks {
    fn on_stage_start(&self, stage: crate::consensus::types::Stage, model: &str) -> Result<()> {
        // Always send stage start immediately (not batched)
        self.inner.on_stage_start(stage, model)
    }

    fn on_stage_chunk(
        &self,
        stage: crate::consensus::types::Stage,
        chunk: &str,
        total: &str,
    ) -> Result<()> {
        // This is the critical fix - batch the chunk updates to prevent CPU overload!

        // Update pending content
        let pending_content = self.pending_content.clone();
        let inner = self.inner.clone();
        let last_update = self.last_update.clone();
        let update_interval = self.update_interval;
        let stage_copy = stage;
        let chunk_str = chunk.to_string();
        let total_str = total.to_string();

        tokio::spawn(async move {
            // Always update the accumulated content
            {
                let mut pending = pending_content.write().await;
                pending.insert(stage_copy, total_str.clone());
            }

            // Only send UI update if enough time has passed (batching!)
            let should_update = {
                let last = last_update.read().await;
                last.elapsed() >= update_interval
            };

            if should_update {
                // Send the accumulated content
                let _ = inner.on_stage_chunk(stage_copy, &chunk_str, &total_str);

                // Update timestamp
                {
                    let mut last = last_update.write().await;
                    *last = Instant::now();
                }
            }
        });

        Ok(())
    }

    fn on_stage_complete(
        &self,
        stage: crate::consensus::types::Stage,
        result: &crate::consensus::types::StageResult,
    ) -> Result<()> {
        // Always send stage completion immediately (not batched)
        self.inner.on_stage_complete(stage, result)
    }

    // Forward other methods without batching
    fn on_profile_loaded(&self, profile_name: &str, models: &[String]) -> Result<()> {
        self.inner.on_profile_loaded(profile_name, models)
    }

    fn on_stage_progress(
        &self,
        stage: crate::consensus::types::Stage,
        progress: crate::consensus::streaming::ProgressInfo,
    ) -> Result<()> {
        self.inner.on_stage_progress(stage, progress)
    }

    fn on_error(&self, stage: crate::consensus::types::Stage, error: &anyhow::Error) -> Result<()> {
        self.inner.on_error(stage, error)
    }

    fn on_d1_authorization(&self, conversations_remaining: u32) -> Result<()> {
        self.inner.on_d1_authorization(conversations_remaining)
    }
}

/// Complete working callbacks from 2bd6753 - restored exactly as they were
struct DesktopStreamingCallbacks {
    event_sender: mpsc::UnboundedSender<ConsensusUIEvent>,
    auto_accept_enabled: Arc<std::sync::atomic::AtomicBool>,
    conversation_id: String,
    question: String,
    stages_completed: Arc<
        Mutex<
            Vec<(
                crate::consensus::types::Stage,
                crate::consensus::types::StageResult,
            )>,
        >,
    >,
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

impl StreamingCallbacks for DesktopStreamingCallbacks {
    fn on_profile_loaded(&self, profile_name: &str, models: &[String]) -> Result<()> {
        tracing::info!(
            "🎯 Profile loaded callback: {} with models {:?}",
            profile_name,
            models
        );

        // Send profile loaded event to update UI
        let _ = self.event_sender.send(ConsensusUIEvent::ProfileLoaded {
            profile_name: profile_name.to_string(),
            models: models.to_vec(),
        });

        Ok(())
    }

    fn on_stage_start(&self, stage: crate::consensus::types::Stage, model: &str) -> Result<()> {
        let consensus_stage = match stage {
            crate::consensus::types::Stage::Generator => {
                crate::desktop::state::ConsensusStage::Generator
            }
            crate::consensus::types::Stage::Refiner => {
                crate::desktop::state::ConsensusStage::Refiner
            }
            crate::consensus::types::Stage::Validator => {
                crate::desktop::state::ConsensusStage::Validator
            }
            crate::consensus::types::Stage::Curator => {
                crate::desktop::state::ConsensusStage::Curator
            }
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

    fn on_stage_chunk(
        &self,
        stage: crate::consensus::types::Stage,
        chunk: &str,
        total_content: &str,
    ) -> Result<()> {
        let consensus_stage = match stage {
            crate::consensus::types::Stage::Generator => {
                crate::desktop::state::ConsensusStage::Generator
            }
            crate::consensus::types::Stage::Refiner => {
                crate::desktop::state::ConsensusStage::Refiner
            }
            crate::consensus::types::Stage::Validator => {
                crate::desktop::state::ConsensusStage::Validator
            }
            crate::consensus::types::Stage::Curator => {
                crate::desktop::state::ConsensusStage::Curator
            }
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

    fn on_stage_progress(
        &self,
        stage: crate::consensus::types::Stage,
        progress: crate::consensus::streaming::ProgressInfo,
    ) -> Result<()> {
        let consensus_stage = match stage {
            crate::consensus::types::Stage::Generator => {
                crate::desktop::state::ConsensusStage::Generator
            }
            crate::consensus::types::Stage::Refiner => {
                crate::desktop::state::ConsensusStage::Refiner
            }
            crate::consensus::types::Stage::Validator => {
                crate::desktop::state::ConsensusStage::Validator
            }
            crate::consensus::types::Stage::Curator => {
                crate::desktop::state::ConsensusStage::Curator
            }
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
        stage: crate::consensus::types::Stage,
        result: &crate::consensus::types::StageResult,
    ) -> Result<()> {
        let consensus_stage = match stage {
            crate::consensus::types::Stage::Generator => {
                crate::desktop::state::ConsensusStage::Generator
            }
            crate::consensus::types::Stage::Refiner => {
                crate::desktop::state::ConsensusStage::Refiner
            }
            crate::consensus::types::Stage::Validator => {
                crate::desktop::state::ConsensusStage::Validator
            }
            crate::consensus::types::Stage::Curator => {
                crate::desktop::state::ConsensusStage::Curator
            }
        };

        let cost = result.analytics.as_ref().map(|a| a.cost).unwrap_or(0.0);

        let _ = self.event_sender.send(ConsensusUIEvent::StageCompleted {
            stage: consensus_stage,
            cost,
        });

        Ok(())
    }

    fn on_error(&self, stage: crate::consensus::types::Stage, error: &anyhow::Error) -> Result<()> {
        let consensus_stage = match stage {
            crate::consensus::types::Stage::Generator => {
                crate::desktop::state::ConsensusStage::Generator
            }
            crate::consensus::types::Stage::Refiner => {
                crate::desktop::state::ConsensusStage::Refiner
            }
            crate::consensus::types::Stage::Validator => {
                crate::desktop::state::ConsensusStage::Validator
            }
            crate::consensus::types::Stage::Curator => {
                crate::desktop::state::ConsensusStage::Curator
            }
        };

        let _ = self.event_sender.send(ConsensusUIEvent::StageError {
            stage: consensus_stage,
            error: error.to_string(),
        });

        Ok(())
    }
}

/// Create the wrapper (called from UI)
pub async fn create_consensus_wrapper(
    app_state: Signal<AppState>,
) -> Option<ConsensusThreadWrapper> {
    match crate::core::api_keys::ApiKeyManager::has_valid_keys().await {
        Ok(true) => match ConsensusThreadWrapper::new(app_state).await {
            Ok(wrapper) => {
                info!("✅ Consensus wrapper created - engine runs on Tokio!");
                Some(wrapper)
            }
            Err(e) => {
                error!("Failed to create wrapper: {}", e);
                None
            }
        },
        _ => None,
    }
}
