//! Parallel Processor for AI Helpers
//!
//! This module orchestrates parallel execution of AI helper tasks to maximize
//! performance while managing resource usage.

use anyhow::{Context, Result};
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinHandle;

use super::{
    IndexedKnowledge, Insight, KnowledgeIndexer, KnowledgeSynthesizer, Pattern, PatternRecognizer,
    QualityAnalyzer, QualityReport,
};

/// Configuration for parallel processing
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,

    /// Timeout for individual tasks
    pub task_timeout: std::time::Duration,

    /// Enable parallel pattern recognition
    pub parallel_patterns: bool,

    /// Enable parallel quality analysis
    pub parallel_quality: bool,

    /// Enable parallel synthesis
    pub parallel_synthesis: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 4,
            task_timeout: std::time::Duration::from_secs(30),
            parallel_patterns: true,
            parallel_quality: true,
            parallel_synthesis: true,
        }
    }
}

/// Parallel processor for AI helpers
#[derive(Clone)]
pub struct ParallelProcessor {
    config: ParallelConfig,

    /// Semaphore to limit concurrent tasks
    semaphore: Arc<Semaphore>,

    /// Performance metrics
    metrics: Arc<RwLock<ProcessingMetrics>>,
}

/// Processing metrics
#[derive(Default)]
struct ProcessingMetrics {
    tasks_completed: usize,
    tasks_failed: usize,
    total_processing_time: std::time::Duration,
    parallel_speedup: f64,
}

/// Result of parallel processing
#[derive(Debug)]
pub struct ParallelResult {
    pub patterns: Vec<Pattern>,
    pub quality: QualityReport,
    pub insights: Vec<Insight>,
    pub processing_time: std::time::Duration,
    pub parallel_speedup: f64,
}

impl ParallelProcessor {
    /// Create a new parallel processor
    pub fn new(config: ParallelConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        let metrics = Arc::new(RwLock::new(ProcessingMetrics::default()));

        Self {
            config,
            semaphore,
            metrics,
        }
    }

    /// Process multiple AI helper tasks in parallel
    pub async fn process_parallel(
        &self,
        indexed: &IndexedKnowledge,
        curator_output: &str,
        pattern_recognizer: Arc<PatternRecognizer>,
        quality_analyzer: Arc<QualityAnalyzer>,
        knowledge_synthesizer: Arc<KnowledgeSynthesizer>,
    ) -> Result<ParallelResult> {
        let start = std::time::Instant::now();

        // Create task handles
        let mut tasks: Vec<JoinHandle<Result<TaskResult>>> = Vec::new();

        // 1. Pattern recognition task
        if self.config.parallel_patterns {
            let indexed_clone = indexed.clone();
            let recognizer = pattern_recognizer.clone();
            let semaphore = self.semaphore.clone();

            tasks.push(tokio::spawn(async move {
                let _permit = semaphore.acquire().await?;
                let patterns = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    recognizer.analyze_patterns(&indexed_clone),
                )
                .await
                .context("Pattern recognition timeout")??;

                Ok(TaskResult::Patterns(patterns))
            }));
        }

        // 2. Quality analysis task
        if self.config.parallel_quality {
            let indexed_clone = indexed.clone();
            let output_clone = curator_output.to_string();
            let analyzer = quality_analyzer.clone();
            let semaphore = self.semaphore.clone();

            tasks.push(tokio::spawn(async move {
                let _permit = semaphore.acquire().await?;
                let quality = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    analyzer.evaluate_quality(&indexed_clone, &output_clone),
                )
                .await
                .context("Quality analysis timeout")??;

                Ok(TaskResult::Quality(quality))
            }));
        }

        // Execute tasks and collect results
        let results = join_all(tasks).await;

        // Process results
        let mut patterns = Vec::new();
        let mut quality_opt = None;
        let mut insights = Vec::new();

        for result in results {
            match result {
                Ok(Ok(task_result)) => match task_result {
                    TaskResult::Patterns(p) => patterns = p,
                    TaskResult::Quality(q) => quality_opt = Some(q),
                    TaskResult::Insights(i) => insights = i,
                },
                Ok(Err(e)) => {
                    tracing::error!("Task failed: {}", e);
                    self.metrics.write().await.tasks_failed += 1;
                }
                Err(e) => {
                    tracing::error!("Task panicked: {}", e);
                    self.metrics.write().await.tasks_failed += 1;
                }
            }
        }

        // Ensure we have quality report (run sequentially if parallel failed)
        let quality = if let Some(q) = quality_opt {
            q
        } else {
            quality_analyzer
                .evaluate_quality(indexed, curator_output)
                .await?
        };

        // 3. Synthesis task (may depend on patterns and quality)
        if self.config.parallel_synthesis && !patterns.is_empty() {
            insights = knowledge_synthesizer
                .generate_insights(indexed, &patterns, &quality)
                .await?;
        }

        let processing_time = start.elapsed();

        // Calculate parallel speedup
        let sequential_estimate = std::time::Duration::from_millis(
            patterns.len() as u64 * 100 + 200 + insights.len() as u64 * 50,
        );
        let parallel_speedup = sequential_estimate.as_secs_f64() / processing_time.as_secs_f64();

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.tasks_completed += 3;
        metrics.total_processing_time += processing_time;
        metrics.parallel_speedup = parallel_speedup;

        Ok(ParallelResult {
            patterns,
            quality,
            insights,
            processing_time,
            parallel_speedup,
        })
    }

    /// Process a batch of curator outputs in parallel
    pub async fn process_batch(
        &self,
        outputs: Vec<(String, String, String)>, // (curator_output, source_question, conversation_id)
        knowledge_indexer: Arc<KnowledgeIndexer>,
        pattern_recognizer: Arc<PatternRecognizer>,
        quality_analyzer: Arc<QualityAnalyzer>,
        knowledge_synthesizer: Arc<KnowledgeSynthesizer>,
    ) -> Result<Vec<(IndexedKnowledge, ParallelResult)>> {
        let start = std::time::Instant::now();

        // First, index all outputs in parallel
        let indexing_tasks: Vec<_> = outputs
            .into_iter()
            .map(|(output, question, conv_id)| {
                let indexer = knowledge_indexer.clone();
                let semaphore = self.semaphore.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await?;
                    let indexed = indexer.index_output(&output, &question, &conv_id).await?;
                    Ok::<_, anyhow::Error>((indexed, output))
                })
            })
            .collect();

        let indexed_results = join_all(indexing_tasks).await;

        // Then process each indexed result
        let mut batch_results = Vec::new();

        for result in indexed_results {
            match result {
                Ok(Ok((indexed, curator_output))) => {
                    let parallel_result = self
                        .process_parallel(
                            &indexed,
                            &curator_output,
                            pattern_recognizer.clone(),
                            quality_analyzer.clone(),
                            knowledge_synthesizer.clone(),
                        )
                        .await?;

                    batch_results.push((indexed, parallel_result));
                }
                Ok(Err(e)) => {
                    tracing::error!("Indexing failed: {}", e);
                }
                Err(e) => {
                    tracing::error!("Indexing task panicked: {}", e);
                }
            }
        }

        let batch_time = start.elapsed();
        tracing::info!(
            "Processed batch of {} items in {:?} ({:.2}x speedup)",
            batch_results.len(),
            batch_time,
            self.metrics.read().await.parallel_speedup
        );

        Ok(batch_results)
    }

    /// Get processing statistics
    pub async fn get_stats(&self) -> ProcessingStats {
        let metrics = self.metrics.read().await;
        ProcessingStats {
            tasks_completed: metrics.tasks_completed,
            tasks_failed: metrics.tasks_failed,
            average_processing_time: if metrics.tasks_completed > 0 {
                metrics.total_processing_time / metrics.tasks_completed as u32
            } else {
                std::time::Duration::ZERO
            },
            parallel_speedup: metrics.parallel_speedup,
        }
    }
}

/// Task result enum
enum TaskResult {
    Patterns(Vec<Pattern>),
    Quality(QualityReport),
    Insights(Vec<Insight>),
}

/// Processing statistics
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub average_processing_time: std::time::Duration,
    pub parallel_speedup: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_processing() {
        // Test parallel processing logic
    }
}
