//! MCP sampling support
//!
//! Handles sampling requests and progress tracking for AI model operations

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, debug, error};

/// Sampling request manager
pub struct SamplingManager {
    active_samples: Arc<RwLock<HashMap<String, SampleSession>>>,
    progress_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<SampleProgress>>>>,
    config: SamplingConfig,
}

/// Sampling configuration
#[derive(Clone)]
pub struct SamplingConfig {
    pub max_concurrent_samples: usize,
    pub default_timeout_seconds: u64,
    pub progress_update_interval_ms: u64,
    pub enable_streaming: bool,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Sample session tracking
#[derive(Clone, Debug)]
pub struct SampleSession {
    pub id: String,
    pub request: SampleRequest,
    pub status: SampleStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub progress: SampleProgress,
    pub result: Option<SampleResult>,
    pub error: Option<String>,
}

/// Sample request from MCP client
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SampleRequest {
    pub method: String,
    pub prompt: String,
    pub arguments: HashMap<String, serde_json::Value>,
    pub include_context: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: Option<bool>,
}

/// Sample status enumeration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SampleStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Sample progress information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SampleProgress {
    pub stage: String,
    pub percentage: f32,
    pub tokens_generated: u32,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub current_operation: String,
    pub metrics: SampleMetrics,
}

/// Sample metrics tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SampleMetrics {
    pub tokens_per_second: f32,
    pub total_tokens: u32,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub processing_time_ms: u64,
    pub model_latency_ms: u64,
    pub queue_time_ms: u64,
}

/// Sample result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SampleResult {
    pub text: String,
    pub finish_reason: String,
    pub usage: TokenUsage,
    pub model_info: ModelInfo,
    pub processing_stats: ProcessingStats,
}

/// Token usage information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Model information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_name: String,
    pub model_version: String,
    pub provider: String,
    pub capabilities: Vec<String>,
}

/// Processing statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub total_time_ms: u64,
    pub queue_time_ms: u64,
    pub processing_time_ms: u64,
    pub consensus_stages: u32,
    pub models_used: Vec<String>,
}

impl SamplingManager {
    /// Create new sampling manager
    pub fn new(config: SamplingConfig) -> Self {
        Self {
            active_samples: Arc::new(RwLock::new(HashMap::new())),
            progress_senders: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Start a new sampling session
    pub async fn start_sample(
        &self,
        request: SampleRequest,
    ) -> Result<(String, mpsc::UnboundedReceiver<SampleProgress>)> {
        // Check concurrent limit
        let active_count = self.active_samples.read().await.len();
        if active_count >= self.config.max_concurrent_samples {
            return Err(anyhow!(
                "Maximum concurrent samples reached ({})",
                self.config.max_concurrent_samples
            ));
        }

        // Generate session ID
        let session_id = Uuid::new_v4().to_string();
        
        // Create progress channel
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();

        // Create session
        let session = SampleSession {
            id: session_id.clone(),
            request: request.clone(),
            status: SampleStatus::Queued,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            progress: SampleProgress {
                stage: "Queued".to_string(),
                percentage: 0.0,
                tokens_generated: 0,
                estimated_completion: None,
                current_operation: "Waiting in queue".to_string(),
                metrics: SampleMetrics {
                    tokens_per_second: 0.0,
                    total_tokens: 0,
                    input_tokens: 0,
                    output_tokens: 0,
                    processing_time_ms: 0,
                    model_latency_ms: 0,
                    queue_time_ms: 0,
                },
            },
            result: None,
            error: None,
        };

        // Store session and progress sender
        {
            let mut active_samples = self.active_samples.write().await;
            active_samples.insert(session_id.clone(), session);
        }

        {
            let mut progress_senders = self.progress_senders.write().await;
            progress_senders.insert(session_id.clone(), progress_tx);
        }

        info!("Started sampling session: {}", session_id);
        Ok((session_id, progress_rx))
    }

    /// Update sample progress
    pub async fn update_progress(
        &self,
        session_id: &str,
        progress: SampleProgress,
    ) -> Result<()> {
        // Update session
        {
            let mut active_samples = self.active_samples.write().await;
            if let Some(session) = active_samples.get_mut(session_id) {
                session.progress = progress.clone();
                session.updated_at = Utc::now();
            } else {
                return Err(anyhow!("Sample session not found: {}", session_id));
            }
        }

        // Send progress update
        {
            let progress_senders = self.progress_senders.read().await;
            if let Some(sender) = progress_senders.get(session_id) {
                if let Err(e) = sender.send(progress) {
                    debug!("Failed to send progress update: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Complete sample with result
    pub async fn complete_sample(
        &self,
        session_id: &str,
        result: SampleResult,
    ) -> Result<()> {
        {
            let mut active_samples = self.active_samples.write().await;
            if let Some(session) = active_samples.get_mut(session_id) {
                session.status = SampleStatus::Completed;
                session.result = Some(result);
                session.updated_at = Utc::now();
                
                // Send final progress update
                let final_progress = SampleProgress {
                    stage: "Completed".to_string(),
                    percentage: 100.0,
                    tokens_generated: session.result.as_ref()
                        .map(|r| r.usage.completion_tokens)
                        .unwrap_or(0),
                    estimated_completion: Some(Utc::now()),
                    current_operation: "Sample completed successfully".to_string(),
                    metrics: session.progress.metrics.clone(),
                };
                session.progress = final_progress.clone();

                // Send to progress channel
                {
                    let progress_senders = self.progress_senders.read().await;
                    if let Some(sender) = progress_senders.get(session_id) {
                        let _ = sender.send(final_progress);
                    }
                }
            } else {
                return Err(anyhow!("Sample session not found: {}", session_id));
            }
        }

        info!("Completed sampling session: {}", session_id);
        Ok(())
    }

    /// Fail sample with error
    pub async fn fail_sample(
        &self,
        session_id: &str,
        error: String,
    ) -> Result<()> {
        {
            let mut active_samples = self.active_samples.write().await;
            if let Some(session) = active_samples.get_mut(session_id) {
                session.status = SampleStatus::Failed;
                session.error = Some(error.clone());
                session.updated_at = Utc::now();

                // Send error progress update
                let error_progress = SampleProgress {
                    stage: "Failed".to_string(),
                    percentage: 0.0,
                    tokens_generated: 0,
                    estimated_completion: None,
                    current_operation: format!("Sample failed: {}", error),
                    metrics: session.progress.metrics.clone(),
                };
                session.progress = error_progress.clone();

                // Send to progress channel
                {
                    let progress_senders = self.progress_senders.read().await;
                    if let Some(sender) = progress_senders.get(session_id) {
                        let _ = sender.send(error_progress);
                    }
                }
            } else {
                return Err(anyhow!("Sample session not found: {}", session_id));
            }
        }

        error!("Failed sampling session {}: {}", session_id, error);
        Ok(())
    }

    /// Cancel sample
    pub async fn cancel_sample(&self, session_id: &str) -> Result<()> {
        {
            let mut active_samples = self.active_samples.write().await;
            if let Some(session) = active_samples.get_mut(session_id) {
                session.status = SampleStatus::Cancelled;
                session.updated_at = Utc::now();

                // Send cancellation progress update
                let cancel_progress = SampleProgress {
                    stage: "Cancelled".to_string(),
                    percentage: 0.0,
                    tokens_generated: 0,
                    estimated_completion: None,
                    current_operation: "Sample cancelled by request".to_string(),
                    metrics: session.progress.metrics.clone(),
                };
                session.progress = cancel_progress.clone();

                // Send to progress channel
                {
                    let progress_senders = self.progress_senders.read().await;
                    if let Some(sender) = progress_senders.get(session_id) {
                        let _ = sender.send(cancel_progress);
                    }
                }
            } else {
                return Err(anyhow!("Sample session not found: {}", session_id));
            }
        }

        info!("Cancelled sampling session: {}", session_id);
        Ok(())
    }

    /// Get sample session
    pub async fn get_sample(&self, session_id: &str) -> Option<SampleSession> {
        self.active_samples.read().await.get(session_id).cloned()
    }

    /// List all active samples
    pub async fn list_samples(&self) -> Vec<SampleSession> {
        self.active_samples.read().await.values().cloned().collect()
    }

    /// Clean up completed sessions
    pub async fn cleanup_completed(&self) -> Result<usize> {
        let mut cleaned_count = 0;
        
        {
            let mut active_samples = self.active_samples.write().await;
            let mut progress_senders = self.progress_senders.write().await;
            
            let session_ids: Vec<String> = active_samples.keys().cloned().collect();
            
            for session_id in session_ids {
                if let Some(session) = active_samples.get(&session_id) {
                    // Clean up sessions that are completed, failed, or cancelled for more than 1 hour
                    let should_cleanup = matches!(
                        session.status, 
                        SampleStatus::Completed | SampleStatus::Failed | SampleStatus::Cancelled
                    ) && (Utc::now() - session.updated_at).num_hours() >= 1;
                    
                    if should_cleanup {
                        active_samples.remove(&session_id);
                        progress_senders.remove(&session_id);
                        cleaned_count += 1;
                    }
                }
            }
        }

        if cleaned_count > 0 {
            info!("Cleaned up {} completed sampling sessions", cleaned_count);
        }

        Ok(cleaned_count)
    }

    /// Get sampling statistics
    pub async fn get_statistics(&self) -> SamplingStatistics {
        let active_samples = self.active_samples.read().await;
        
        let mut stats = SamplingStatistics {
            total_sessions: active_samples.len(),
            queued: 0,
            running: 0,
            completed: 0,
            failed: 0,
            cancelled: 0,
            average_processing_time_ms: 0.0,
            total_tokens_processed: 0,
            tokens_per_second_avg: 0.0,
        };

        let mut processing_times = Vec::new();
        let mut tokens_per_second_values = Vec::new();

        for session in active_samples.values() {
            match session.status {
                SampleStatus::Queued => stats.queued += 1,
                SampleStatus::Running => stats.running += 1,
                SampleStatus::Completed => {
                    stats.completed += 1;
                    if let Some(result) = &session.result {
                        stats.total_tokens_processed += result.usage.total_tokens;
                        processing_times.push(result.processing_stats.processing_time_ms as f64);
                    }
                }
                SampleStatus::Failed => stats.failed += 1,
                SampleStatus::Cancelled => stats.cancelled += 1,
            }

            if session.progress.metrics.tokens_per_second > 0.0 {
                tokens_per_second_values.push(session.progress.metrics.tokens_per_second);
            }
        }

        // Calculate averages
        if !processing_times.is_empty() {
            stats.average_processing_time_ms = processing_times.iter().sum::<f64>() / processing_times.len() as f64;
        }

        if !tokens_per_second_values.is_empty() {
            stats.tokens_per_second_avg = tokens_per_second_values.iter().sum::<f32>() / tokens_per_second_values.len() as f32;
        }

        stats
    }
}

/// Sampling statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct SamplingStatistics {
    pub total_sessions: usize,
    pub queued: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub average_processing_time_ms: f64,
    pub total_tokens_processed: u32,
    pub tokens_per_second_avg: f32,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_samples: 10,
            default_timeout_seconds: 300, // 5 minutes
            progress_update_interval_ms: 500, // 0.5 seconds
            enable_streaming: true,
            max_tokens: Some(4096),
            temperature: Some(0.7),
        }
    }
}

impl Default for SampleMetrics {
    fn default() -> Self {
        Self {
            tokens_per_second: 0.0,
            total_tokens: 0,
            input_tokens: 0,
            output_tokens: 0,
            processing_time_ms: 0,
            model_latency_ms: 0,
            queue_time_ms: 0,
        }
    }
}

/// Create sample progress for consensus stage
pub fn create_consensus_progress(
    stage: &str,
    percentage: f32,
    operation: &str,
    tokens_generated: u32,
    metrics: SampleMetrics,
) -> SampleProgress {
    SampleProgress {
        stage: stage.to_string(),
        percentage,
        tokens_generated,
        estimated_completion: if percentage > 0.0 {
            let remaining_time = (100.0 - percentage) / percentage * metrics.processing_time_ms as f32;
            Some(Utc::now() + chrono::Duration::milliseconds(remaining_time as i64))
        } else {
            None
        },
        current_operation: operation.to_string(),
        metrics,
    }
}