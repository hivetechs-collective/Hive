// Streaming support for consensus pipeline
// Provides real-time updates and progress tracking

use crate::consensus::types::{ResponseMetadata, Stage, StageResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

/// Events emitted during consensus processing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConsensusEvent {
    /// Stage has started
    StageStarted { stage: Stage, model: String },
    /// Token received from model
    Token {
        stage: Stage,
        chunk: String,
        total_content: String,
    },
    /// Progress update
    Progress {
        stage: Stage,
        tokens: u32,
        estimated_total: Option<u32>,
        percentage: f32,
    },
    /// Stage completed
    StageCompleted { stage: Stage, result: StageResult },
    /// Error occurred
    Error { stage: Stage, error: String },
    /// Pipeline completed
    Completed,
    /// Final response ready
    FinalResponse { content: String },
}

/// Callbacks for streaming consensus
pub trait StreamingCallbacks: Send + Sync {
    /// Called when profile is loaded
    fn on_profile_loaded(&self, profile_name: &str, models: &[String]) -> Result<()> {
        Ok(())
    }
    
    /// Called when a stage starts
    fn on_stage_start(&self, stage: Stage, model: &str) -> Result<()> {
        Ok(())
    }

    /// Called when a token is received
    fn on_stage_chunk(&self, stage: Stage, chunk: &str, total_content: &str) -> Result<()> {
        Ok(())
    }

    /// Called on progress update
    fn on_stage_progress(&self, stage: Stage, progress: ProgressInfo) -> Result<()> {
        Ok(())
    }

    /// Called when a stage completes
    fn on_stage_complete(&self, stage: Stage, result: &StageResult) -> Result<()> {
        Ok(())
    }

    /// Called on error
    fn on_error(&self, stage: Stage, error: &anyhow::Error) -> Result<()> {
        Ok(())
    }

    /// Called when D1 authorization is received (optional)
    fn on_d1_authorization(&self, remaining: u32) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called when analytics should be refreshed after consensus completion
    fn on_analytics_refresh(&self) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub tokens: u32,
    pub estimated_total: Option<u32>,
    pub percentage: f32,
}

/// Default implementation that prints to console
pub struct ConsoleCallbacks {
    pub show_progress: bool,
}

impl StreamingCallbacks for ConsoleCallbacks {
    fn on_stage_start(&self, stage: Stage, model: &str) -> Result<()> {
        if self.show_progress {
            tracing::info!("{} → Starting ({})", stage.display_name(), model);
        }
        Ok(())
    }

    fn on_stage_chunk(&self, stage: Stage, chunk: &str, _total_content: &str) -> Result<()> {
        if !self.show_progress {
            // In non-progress mode, stream tokens directly
            print!("{}", chunk);
            use std::io::{self, Write};
            io::stdout().flush()?;
        }
        Ok(())
    }

    fn on_stage_progress(&self, stage: Stage, progress: ProgressInfo) -> Result<()> {
        if self.show_progress {
            let bar_width = 20;
            let filled = (bar_width as f32 * progress.percentage / 100.0) as usize;
            let empty = bar_width - filled;

            print!(
                "\r{} → [{}{}] {:.0}%",
                stage.display_name(),
                "█".repeat(filled),
                "░".repeat(empty),
                progress.percentage
            );
            use std::io::{self, Write};
            io::stdout().flush()?;
        }
        Ok(())
    }

    fn on_stage_complete(&self, stage: Stage, _result: &StageResult) -> Result<()> {
        if self.show_progress {
            tracing::info!("{} → [{}] 100% ✓", stage.display_name(), "█".repeat(20));
        }
        Ok(())
    }

    fn on_error(&self, stage: Stage, error: &anyhow::Error) -> Result<()> {
        tracing::error!("{} → Error: {}", stage.display_name(), error);
        Ok(())
    }
}

/// Progress tracker for stages
pub struct ProgressTracker {
    pub stage: Stage,
    pub tokens: u32,
    pub content: String,
    pub estimated_total: Option<u32>,
    pub callbacks: Arc<dyn StreamingCallbacks>,
}

impl ProgressTracker {
    pub fn new(stage: Stage, callbacks: Arc<dyn StreamingCallbacks>) -> Self {
        Self {
            stage,
            tokens: 0,
            content: String::new(),
            estimated_total: None,
            callbacks,
        }
    }

    /// Add a chunk and update progress
    pub fn add_chunk(&mut self, chunk: &str) -> Result<()> {
        self.content.push_str(chunk);
        self.tokens += chunk.split_whitespace().count() as u32;

        // Estimate total if not set (based on typical response sizes)
        if self.estimated_total.is_none() {
            self.estimated_total = Some(match self.stage {
                Stage::Generator => 500, // Typically longer
                Stage::Refiner => 400,
                Stage::Validator => 300,
                Stage::Curator => 350,
            });
        }

        let percentage = if let Some(total) = self.estimated_total {
            ((self.tokens as f32 / total as f32) * 100.0).min(99.0)
        } else {
            0.0
        };

        self.callbacks
            .on_stage_chunk(self.stage, chunk, &self.content)?;
        self.callbacks.on_stage_progress(
            self.stage,
            ProgressInfo {
                tokens: self.tokens,
                estimated_total: self.estimated_total,
                percentage,
            },
        )?;

        Ok(())
    }

    /// Mark stage as complete
    pub fn complete(&mut self) -> Result<()> {
        self.callbacks.on_stage_progress(
            self.stage,
            ProgressInfo {
                tokens: self.tokens,
                estimated_total: Some(self.tokens),
                percentage: 100.0,
            },
        )?;
        Ok(())
    }

    /// Get the accumulated content
    pub fn content(&self) -> &str {
        &self.content
    }
}

/// Streaming response for TUI integration
#[derive(Debug, Clone)]
pub enum StreamingResponse {
    StageStarted {
        stage: ConsensusStage,
        model: String,
    },
    StageProgress {
        stage: ConsensusStage,
        progress: u16,
    },
    StageCompleted {
        stage: ConsensusStage,
    },
    TokenReceived {
        token: String,
    },
    Error {
        stage: ConsensusStage,
        error: String,
    },
    Complete {
        response: ConsensusResponseResult,
    },
}

/// Consensus stage enum for TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusStage {
    Generator,
    Refiner,
    Validator,
    Curator,
}

/// Consensus response result
#[derive(Debug, Clone)]
pub struct ConsensusResponseResult {
    pub content: String,
    pub metadata: ResponseMetadata,
}

/// Enhanced progress tracker with quality metrics
pub struct EnhancedProgressTracker {
    pub stage: Stage,
    start_time: Instant,
    tokens: u32,
    content: String,
    estimated_total: Option<u32>,
    callbacks: Arc<dyn StreamingCallbacks>,
    quality_score: f32,
}

impl EnhancedProgressTracker {
    pub fn new(stage: Stage, callbacks: Arc<dyn StreamingCallbacks>) -> Self {
        Self {
            stage,
            start_time: Instant::now(),
            tokens: 0,
            content: String::new(),
            estimated_total: None,
            callbacks,
            quality_score: 0.0,
        }
    }

    /// Add chunk with real-time quality assessment
    pub fn add_chunk_with_quality(&mut self, chunk: &str) -> Result<()> {
        self.content.push_str(chunk);
        self.tokens += chunk.split_whitespace().count() as u32;

        // Update quality score in real-time
        self.update_quality_score();

        if self.estimated_total.is_none() {
            self.estimated_total = Some(match self.stage {
                Stage::Generator => 450,
                Stage::Refiner => 380,
                Stage::Validator => 320,
                Stage::Curator => 400,
            });
        }

        let percentage = if let Some(total) = self.estimated_total {
            ((self.tokens as f32 / total as f32) * 100.0).min(99.0)
        } else {
            0.0
        };

        self.callbacks
            .on_stage_chunk(self.stage, chunk, &self.content)?;
        self.callbacks.on_stage_progress(
            self.stage,
            ProgressInfo {
                tokens: self.tokens,
                estimated_total: self.estimated_total,
                percentage,
            },
        )?;

        Ok(())
    }

    fn update_quality_score(&mut self) {
        if self.content.len() < 50 {
            self.quality_score = 0.0;
            return;
        }

        let mut score: f32 = 0.0;

        // Check for structure indicators
        if self.content.contains("##") || self.content.contains("###") {
            score += 0.3;
        }

        // Check for code quality
        if self.content.contains("```") {
            let code_blocks = self.content.matches("```").count();
            if code_blocks % 2 == 0 && code_blocks > 0 {
                score += 0.2;
            }
        }

        // Check for explanatory content
        if self.content.contains("because") || self.content.contains("therefore") {
            score += 0.2;
        }

        // Check for actionable content
        if self.content.contains("you can") || self.content.contains("steps") {
            score += 0.3;
        }

        self.quality_score = score.min(1.0_f32);
    }

    pub fn get_quality_score(&self) -> f32 {
        self.quality_score
    }

    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Real-time consensus metrics for TUI display
#[derive(Debug, Clone)]
pub struct ConsensusMetrics {
    pub current_stage: Option<Stage>,
    pub stage_progress: [f32; 4], // Progress for each of the 4 stages
    pub stage_quality: [f32; 4],  // Quality scores for each stage
    pub overall_progress: f32,
    pub elapsed_time: std::time::Duration,
    pub estimated_completion: Option<std::time::Duration>,
    pub total_tokens: u32,
    pub estimated_cost: f64,
}

impl Default for ConsensusMetrics {
    fn default() -> Self {
        Self {
            current_stage: None,
            stage_progress: [0.0; 4],
            stage_quality: [0.0; 4],
            overall_progress: 0.0,
            elapsed_time: std::time::Duration::from_secs(0),
            estimated_completion: None,
            total_tokens: 0,
            estimated_cost: 0.0,
        }
    }
}

impl ConsensusMetrics {
    pub fn update_stage_progress(&mut self, stage: Stage, progress: f32, quality: f32) {
        let index = match stage {
            Stage::Generator => 0,
            Stage::Refiner => 1,
            Stage::Validator => 2,
            Stage::Curator => 3,
        };

        self.stage_progress[index] = progress;
        self.stage_quality[index] = quality;
        self.overall_progress = self.stage_progress.iter().sum::<f32>() / 4.0;
    }

    pub fn set_current_stage(&mut self, stage: Option<Stage>) {
        self.current_stage = stage;
    }

    pub fn get_stage_name(&self, index: usize) -> &'static str {
        match index {
            0 => "Generator",
            1 => "Refiner",
            2 => "Validator",
            3 => "Curator",
            _ => "Unknown",
        }
    }

    pub fn format_stage_display(&self, index: usize) -> String {
        let progress = self.stage_progress[index];
        let quality = self.stage_quality[index];
        let name = self.get_stage_name(index);

        let status = if progress >= 100.0 {
            "✓"
        } else if progress > 0.0 {
            "●"
        } else {
            "○"
        };

        format!(
            "{} {} {:.0}% (Q: {:.1})",
            status,
            name,
            progress,
            quality * 100.0
        )
    }
}

/// Channel-based streaming callbacks for consensus engine
pub struct ChannelStreamingCallbacks {
    sender: mpsc::UnboundedSender<StreamingResponse>,
}

impl ChannelStreamingCallbacks {
    pub fn new(sender: mpsc::UnboundedSender<StreamingResponse>) -> Self {
        Self { sender }
    }
}

impl StreamingCallbacks for ChannelStreamingCallbacks {
    fn on_stage_start(&self, stage: Stage, model: &str) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        let _ = self.sender.send(StreamingResponse::StageStarted {
            stage: consensus_stage,
            model: model.to_string(),
        });

        Ok(())
    }

    fn on_stage_chunk(&self, _stage: Stage, chunk: &str, _total_content: &str) -> Result<()> {
        let _ = self.sender.send(StreamingResponse::TokenReceived {
            token: chunk.to_string(),
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

        let progress_u16 = (progress.percentage as u16).min(100);

        let _ = self.sender.send(StreamingResponse::StageProgress {
            stage: consensus_stage,
            progress: progress_u16,
        });

        Ok(())
    }

    fn on_stage_complete(&self, stage: Stage, _result: &StageResult) -> Result<()> {
        let consensus_stage = match stage {
            Stage::Generator => ConsensusStage::Generator,
            Stage::Refiner => ConsensusStage::Refiner,
            Stage::Validator => ConsensusStage::Validator,
            Stage::Curator => ConsensusStage::Curator,
        };

        let _ = self.sender.send(StreamingResponse::StageCompleted {
            stage: consensus_stage,
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

        let _ = self.sender.send(StreamingResponse::Error {
            stage: consensus_stage,
            error: error.to_string(),
        });

        Ok(())
    }
}

/// Channel-based event stream
pub struct EventStream {
    receiver: mpsc::UnboundedReceiver<ConsensusEvent>,
}

impl EventStream {
    pub fn new() -> (Self, mpsc::UnboundedSender<ConsensusEvent>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (Self { receiver }, sender)
    }

    /// Get next event
    pub async fn next(&mut self) -> Option<ConsensusEvent> {
        self.receiver.recv().await
    }
}

/// Progress display formatter
pub struct ProgressFormatter;

impl ProgressFormatter {
    /// Format progress bar for terminal display
    pub fn format_progress_bar(stage: &str, percentage: f32, model: &str) -> String {
        let bar_width = 30;
        let filled = (bar_width as f32 * percentage / 100.0) as usize;
        let empty = bar_width - filled;

        format!(
            "{:<10} → [{}{}] {:>3.0}% ({})",
            stage,
            "█".repeat(filled),
            "░".repeat(empty),
            percentage,
            model
        )
    }

    /// Format stage timing
    pub fn format_timing(duration_ms: u64) -> String {
        if duration_ms < 1000 {
            format!("{}ms", duration_ms)
        } else {
            format!("{:.1}s", duration_ms as f64 / 1000.0)
        }
    }
}
