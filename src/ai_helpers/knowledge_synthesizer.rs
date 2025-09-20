//! Knowledge Synthesizer - Uses local LLMs for knowledge synthesis and insight generation
//!
//! This module combines related facts, generates meta-insights, creates summaries,
//! provides intelligent synthesis of accumulated knowledge, and generates comprehensive
//! operation plans with previews for file operations.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::python_models::{ModelRequest, ModelResponse, PythonModelService};
use crate::ai_helpers::{
    IndexedKnowledge, Insight, InsightType, Pattern, PatternType, QualityReport,
};
use crate::consensus::operation_intelligence::{
    BackupMethod, BackupStrategy, ChangeType, ContentPreview, ExecutionStep, OperationContext,
    OperationPlan, OperationPreview, RollbackAction, RollbackPlan, RollbackStep,
};
use crate::consensus::stages::file_aware_curator::FileOperation;

/// Configuration for Knowledge Synthesizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizerConfig {
    /// Local model for synthesis (e.g., Mistral-7B, Qwen2-7B)
    pub synthesis_model: String,

    /// Model quantization (e.g., "4-bit", "8-bit", "fp16")
    pub quantization: String,

    /// Maximum tokens for synthesis
    pub max_tokens: usize,

    /// Temperature for generation
    pub temperature: f64,

    /// Minimum facts required for synthesis
    pub min_facts_for_synthesis: usize,

    /// Enable operation planning
    pub enable_operation_planning: bool,

    /// Maximum preview size in bytes
    pub max_preview_size: usize,
}

impl Default for SynthesizerConfig {
    fn default() -> Self {
        Self {
            synthesis_model: "mistral-7b-instruct".to_string(),
            quantization: "4-bit".to_string(),
            max_tokens: 512,
            temperature: 0.7,
            min_facts_for_synthesis: 3,
            enable_operation_planning: true,
            max_preview_size: 5000,
        }
    }
}

/// Knowledge Synthesizer using local LLMs
pub struct KnowledgeSynthesizer {
    config: SynthesizerConfig,

    /// Python model service
    python_service: Arc<PythonModelService>,

    /// Synthesis history
    synthesis_history: Arc<RwLock<SynthesisHistory>>,

    /// Cache of recent syntheses
    synthesis_cache: Arc<RwLock<lru::LruCache<String, Vec<Insight>>>>,

    /// Operation plan cache
    plan_cache: Arc<RwLock<lru::LruCache<String, OperationPlan>>>,

    /// Operation preview generator
    preview_generator: Arc<RwLock<PreviewGenerator>>,
}

impl std::fmt::Debug for KnowledgeSynthesizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KnowledgeSynthesizer")
            .field("config", &self.config)
            .field("python_service", &"<PythonModelService>")
            .field("synthesis_history", &"<SynthesisHistory>")
            .field("synthesis_cache", &"<LruCache>")
            .field("plan_cache", &"<LruCache>")
            .field("preview_generator", &"<PreviewGenerator>")
            .finish()
    }
}

/// History of synthesis operations
#[derive(Default)]
struct SynthesisHistory {
    /// All generated insights
    insights: Vec<GeneratedInsight>,

    /// Synthesis patterns
    patterns: Vec<SynthesisPattern>,

    /// Performance metrics
    metrics: SynthesisMetrics,
}

#[derive(Debug, Clone)]
struct GeneratedInsight {
    insight: Insight,
    timestamp: chrono::DateTime<chrono::Utc>,
    source_facts: Vec<String>,
    synthesis_method: String,
}

#[derive(Debug, Clone)]
struct SynthesisPattern {
    pattern_type: String,
    description: String,
    frequency: usize,
    effectiveness: f64,
}

#[derive(Default, Debug, Clone)]
struct SynthesisMetrics {
    total_syntheses: usize,
    successful_insights: usize,
    average_confidence: f64,
    synthesis_time_ms: Vec<u64>,
}

/// Generates previews for file operations
struct PreviewGenerator {
    /// Template cache for common operations
    templates: HashMap<String, PreviewTemplate>,

    /// File content cache for preview generation
    content_cache: lru::LruCache<PathBuf, CachedContent>,

    /// Preview generation history
    preview_history: Vec<PreviewRecord>,
}

impl Default for PreviewGenerator {
    fn default() -> Self {
        Self {
            templates: HashMap::new(),
            content_cache: lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
            preview_history: Vec::new(),
        }
    }
}

#[derive(Clone)]
struct PreviewTemplate {
    operation_type: String,
    template_content: String,
    placeholders: Vec<String>,
}

#[derive(Clone)]
struct CachedContent {
    content: String,
    cached_at: SystemTime,
    file_type: FileType,
}

#[derive(Clone)]
enum FileType {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Markdown,
    Json,
    Toml,
    Yaml,
    Other(String),
}

struct PreviewRecord {
    operation: FileOperation,
    preview: OperationPreview,
    generated_at: SystemTime,
    accuracy_score: Option<f32>,
}

impl KnowledgeSynthesizer {
    /// Create a new Knowledge Synthesizer
    pub async fn new(python_service: Arc<PythonModelService>) -> Result<Self> {
        let config = SynthesizerConfig::default();
        let synthesis_history = Arc::new(RwLock::new(SynthesisHistory::default()));
        let synthesis_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(100).unwrap(),
        )));
        let plan_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(50).unwrap(),
        )));
        let preview_generator = Arc::new(RwLock::new(PreviewGenerator {
            templates: Self::initialize_preview_templates(),
            content_cache: lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
            preview_history: Vec::new(),
        }));

        Ok(Self {
            config,
            python_service,
            synthesis_history,
            synthesis_cache,
            plan_cache,
            preview_generator,
        })
    }

    /// Initialize preview templates
    fn initialize_preview_templates() -> HashMap<String, PreviewTemplate> {
        let mut templates = HashMap::new();

        // Rust file creation template
        templates.insert(
            "rust_create".to_string(),
            PreviewTemplate {
                operation_type: "create".to_string(),
                template_content: r#"//! {module_description}

use std::sync::Arc;
use anyhow::Result;

/// {struct_description}
pub struct {struct_name} {
    // TODO: Add fields
}

impl {struct_name} {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            // TODO: Initialize fields
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let instance = {struct_name}::new();
        // TODO: Add test assertions
    }
}"#
                .to_string(),
                placeholders: vec![
                    "module_description".to_string(),
                    "struct_description".to_string(),
                    "struct_name".to_string(),
                ],
            },
        );

        // Add more templates for other file types and operations

        templates
    }

    /// Generate insights from indexed knowledge, patterns, and quality report
    pub async fn generate_insights(
        &self,
        indexed: &IndexedKnowledge,
        patterns: &[Pattern],
        quality: &QualityReport,
    ) -> Result<Vec<Insight>> {
        let start = std::time::Instant::now();

        // Check cache
        let cache_key = format!(
            "{}_{}_{}",
            indexed.id,
            patterns.len(),
            quality.overall_score
        );
        if let Some(cached) = self.synthesis_cache.read().await.peek(&cache_key) {
            return Ok(cached.clone());
        }

        let mut insights = Vec::new();

        // 1. Generate trend insights
        if let Some(trend) = self.synthesize_trend_insight(indexed, patterns).await? {
            insights.push(trend);
        }

        // 2. Generate anomaly insights
        if let Some(anomaly) = self.synthesize_anomaly_insight(indexed, quality).await? {
            insights.push(anomaly);
        }

        // 3. Generate relationship insights
        if let Some(relationship) = self
            .synthesize_relationship_insight(indexed, patterns)
            .await?
        {
            insights.push(relationship);
        }

        // 4. Generate predictive insights
        if let Some(prediction) = self
            .synthesize_predictive_insight(indexed, patterns)
            .await?
        {
            insights.push(prediction);
        }

        // 5. Generate recommendation insights
        if let Some(recommendation) = self
            .synthesize_recommendation_insight(indexed, quality)
            .await?
        {
            insights.push(recommendation);
        }

        // Update history
        self.update_synthesis_history(&insights, indexed, start.elapsed())
            .await?;

        // Cache results
        self.synthesis_cache
            .write()
            .await
            .put(cache_key, insights.clone());

        Ok(insights)
    }

    /// Synthesize trend insights
    async fn synthesize_trend_insight(
        &self,
        indexed: &IndexedKnowledge,
        patterns: &[Pattern],
    ) -> Result<Option<Insight>> {
        // Look for evolution patterns that indicate trends
        let evolution_patterns: Vec<_> = patterns
            .iter()
            .filter(|p| matches!(p.pattern_type, PatternType::Evolution))
            .collect();

        if evolution_patterns.is_empty() {
            return Ok(None);
        }

        // TODO: Use local LLM to generate trend description
        // For now, use simple heuristics

        let trend_description = format!(
            "Knowledge evolution detected: {} patterns show changing understanding",
            evolution_patterns.len()
        );

        Ok(Some(Insight {
            insight_type: InsightType::Trend,
            content: trend_description,
            supporting_facts: evolution_patterns
                .iter()
                .flat_map(|p| p.examples.clone())
                .collect(),
            confidence: 0.8,
        }))
    }

    /// Synthesize anomaly insights
    async fn synthesize_anomaly_insight(
        &self,
        indexed: &IndexedKnowledge,
        quality: &QualityReport,
    ) -> Result<Option<Insight>> {
        // Look for quality issues or unusual patterns
        if quality.overall_score < 0.6 || !quality.issues.is_empty() {
            let anomaly_description = format!(
                "Quality anomaly detected: Score {:.2} with {} issues",
                quality.overall_score,
                quality.issues.len()
            );

            return Ok(Some(Insight {
                insight_type: InsightType::Anomaly,
                content: anomaly_description,
                supporting_facts: vec![indexed.content.clone()],
                confidence: 0.9,
            }));
        }

        Ok(None)
    }

    /// Synthesize relationship insights
    async fn synthesize_relationship_insight(
        &self,
        indexed: &IndexedKnowledge,
        patterns: &[Pattern],
    ) -> Result<Option<Insight>> {
        // Look for relationship patterns
        let relationship_patterns: Vec<_> = patterns
            .iter()
            .filter(|p| matches!(p.pattern_type, PatternType::Relationship))
            .collect();

        if relationship_patterns.len() >= 2 {
            // TODO: Use LLM to identify complex relationships

            let relationship_description =
                "Multiple interconnected concepts detected forming a knowledge network";

            return Ok(Some(Insight {
                insight_type: InsightType::Relationship,
                content: relationship_description.to_string(),
                supporting_facts: relationship_patterns
                    .iter()
                    .flat_map(|p| p.examples.clone())
                    .collect(),
                confidence: 0.85,
            }));
        }

        Ok(None)
    }

    /// Synthesize predictive insights
    async fn synthesize_predictive_insight(
        &self,
        indexed: &IndexedKnowledge,
        patterns: &[Pattern],
    ) -> Result<Option<Insight>> {
        // Look for patterns that suggest future developments
        if patterns.len() >= self.config.min_facts_for_synthesis {
            // TODO: Use LLM to generate predictions based on patterns

            // Simple heuristic for now
            if indexed.content.contains("will") || indexed.content.contains("future") {
                return Ok(Some(Insight {
                    insight_type: InsightType::Prediction,
                    content: "Based on current patterns, future developments likely in this area"
                        .to_string(),
                    supporting_facts: vec![indexed.content.clone()],
                    confidence: 0.7,
                }));
            }
        }

        Ok(None)
    }

    /// Synthesize recommendation insights
    async fn synthesize_recommendation_insight(
        &self,
        indexed: &IndexedKnowledge,
        quality: &QualityReport,
    ) -> Result<Option<Insight>> {
        // Generate recommendations based on quality and patterns
        if quality.completeness_score < 0.8 {
            return Ok(Some(Insight {
                insight_type: InsightType::Recommendation,
                content:
                    "Consider exploring this topic further for more comprehensive understanding"
                        .to_string(),
                supporting_facts: vec![format!(
                    "Completeness score: {:.2}",
                    quality.completeness_score
                )],
                confidence: 0.9,
            }));
        }

        Ok(None)
    }

    /// Update synthesis history
    async fn update_synthesis_history(
        &self,
        insights: &[Insight],
        indexed: &IndexedKnowledge,
        elapsed: std::time::Duration,
    ) -> Result<()> {
        let mut history = self.synthesis_history.write().await;
        let now = chrono::Utc::now();

        // Record generated insights
        for insight in insights {
            history.insights.push(GeneratedInsight {
                insight: insight.clone(),
                timestamp: now,
                source_facts: vec![indexed.id.clone()],
                synthesis_method: "heuristic".to_string(), // TODO: Track actual method
            });
        }

        // Update metrics
        history.metrics.total_syntheses += 1;
        history.metrics.successful_insights += insights.len();
        history
            .metrics
            .synthesis_time_ms
            .push(elapsed.as_millis() as u64);

        // Update average confidence
        let total_confidence: f64 = history
            .insights
            .iter()
            .map(|gi| gi.insight.confidence)
            .sum();
        history.metrics.average_confidence =
            total_confidence / history.insights.len().max(1) as f64;

        // Detect synthesis patterns
        self.detect_synthesis_patterns(&mut history);

        Ok(())
    }

    /// Detect patterns in synthesis operations
    fn detect_synthesis_patterns(&self, history: &mut SynthesisHistory) {
        // Count insight types
        let mut type_counts = std::collections::HashMap::new();
        for gi in &history.insights {
            *type_counts
                .entry(format!("{:?}", gi.insight.insight_type))
                .or_insert(0) += 1;
        }

        // Update or create patterns
        for (insight_type, count) in type_counts {
            let effectiveness = history
                .insights
                .iter()
                .filter(|gi| format!("{:?}", gi.insight.insight_type) == insight_type)
                .map(|gi| gi.insight.confidence)
                .sum::<f64>()
                / count as f64;

            if let Some(pattern) = history
                .patterns
                .iter_mut()
                .find(|p| p.pattern_type == insight_type)
            {
                pattern.frequency = count;
                pattern.effectiveness = effectiveness;
            } else {
                history.patterns.push(SynthesisPattern {
                    pattern_type: insight_type.clone(),
                    description: format!("{} synthesis pattern", insight_type),
                    frequency: count,
                    effectiveness,
                });
            }
        }
    }

    /// Generate a comprehensive summary of recent knowledge
    pub async fn generate_summary(
        &self,
        facts: &[IndexedKnowledge],
        max_length: usize,
    ) -> Result<String> {
        if facts.len() < self.config.min_facts_for_synthesis {
            return Ok("Insufficient facts for meaningful summary".to_string());
        }

        // Prepare facts for LLM
        let facts_text = facts
            .iter()
            .take(10) // Limit to prevent context overflow
            .map(|f| format!("- {}", f.content))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Please provide a comprehensive summary of the following knowledge facts. \
             Focus on key themes, relationships, and insights. \
             Limit the summary to {} words.\n\nFacts:\n{}\n\nSummary:",
            max_length / 6, // Rough words estimate
            facts_text
        );

        // Use local LLM to generate summary
        let summary = self
            .python_service
            .generate_text(
                &self.config.synthesis_model,
                &prompt,
                max_length,
                self.config.temperature,
            )
            .await?;

        Ok(summary)
    }

    /// Get synthesis statistics
    pub async fn get_stats(&self) -> SynthesisStats {
        let history = self.synthesis_history.read().await;

        let avg_time_ms = if history.metrics.synthesis_time_ms.is_empty() {
            0
        } else {
            history.metrics.synthesis_time_ms.iter().sum::<u64>()
                / history.metrics.synthesis_time_ms.len() as u64
        };

        SynthesisStats {
            total_insights: history.insights.len(),
            average_confidence: history.metrics.average_confidence,
            synthesis_patterns: history.patterns.len(),
            average_synthesis_time_ms: avg_time_ms,
        }
    }

    /// Generate operation plan for file operations
    pub async fn generate_operation_plan(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<OperationPlan> {
        info!(
            "📋 Generating operation plan for {} operations",
            operations.len()
        );

        // Check cache first
        let cache_key = self.generate_plan_cache_key(operations, context);
        {
            let cache = self.plan_cache.read().await;
            if let Some(cached) = cache.peek(&cache_key) {
                debug!("📋 Using cached operation plan");
                return Ok(cached.clone());
            }
        }

        let start_time = std::time::Instant::now();

        // 1. Generate execution steps
        let execution_steps = self.generate_execution_steps(operations, context).await?;

        // 2. Generate operation preview
        let preview = self.generate_operation_preview(operations, context).await?;

        // 3. Generate backup strategy
        let backup_strategy = self.generate_backup_strategy(operations, context).await?;

        // 4. Generate rollback plan
        let rollback_plan = self
            .generate_rollback_plan(operations, &backup_strategy)
            .await?;

        // 5. Estimate execution time
        let estimated_execution_time = self.estimate_execution_time(operations).await?;

        let plan = OperationPlan {
            execution_steps,
            preview,
            backup_strategy,
            rollback_plan,
            estimated_execution_time,
        };

        // Cache the result
        {
            let mut cache = self.plan_cache.write().await;
            cache.put(cache_key, plan.clone());
        }

        let planning_time = start_time.elapsed();
        info!("✅ Operation plan generated in {:?}", planning_time);

        Ok(plan)
    }

    /// Generate execution steps for operations
    async fn generate_execution_steps(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<Vec<ExecutionStep>> {
        debug!("🔄 Generating execution steps");

        let mut steps = Vec::new();
        let mut operation_groups = self.group_related_operations(operations).await?;

        for (step_number, group) in operation_groups.iter().enumerate() {
            let prerequisites = self
                .identify_prerequisites(&group.operations, &steps)
                .await?;
            let validation_checks = self.generate_validation_checks(&group.operations).await?;

            steps.push(ExecutionStep {
                step_number: step_number + 1,
                description: group.description.clone(),
                operations: group.operations.clone(),
                prerequisites,
                validation_checks,
            });
        }

        Ok(steps)
    }

    /// Generate preview of operation effects
    async fn generate_operation_preview(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<OperationPreview> {
        debug!("👁️ Generating operation preview");

        let mut files_created = Vec::new();
        let mut files_modified = Vec::new();
        let mut files_deleted = Vec::new();
        let mut content_previews = Vec::new();

        let preview_generator = self.preview_generator.read().await;

        for operation in operations {
            match operation {
                FileOperation::Create { path, content } => {
                    files_created.push(path.clone());

                    let preview_content = self
                        .generate_content_preview(
                            path,
                            None,
                            Some(content),
                            ChangeType::Create,
                            &preview_generator,
                        )
                        .await?;

                    content_previews.push(preview_content);
                }
                FileOperation::Update { path, content } => {
                    files_modified.push(path.clone());

                    // Get current content if available
                    let current_content = self.get_current_file_content(path).await.ok();

                    let preview_content = self
                        .generate_content_preview(
                            path,
                            current_content.as_deref(),
                            Some(content),
                            ChangeType::Update,
                            &preview_generator,
                        )
                        .await?;

                    content_previews.push(preview_content);
                }
                FileOperation::Delete { path } => {
                    files_deleted.push(path.clone());

                    let current_content = self.get_current_file_content(path).await.ok();

                    let preview_content = self
                        .generate_content_preview(
                            path,
                            current_content.as_deref(),
                            None,
                            ChangeType::Delete,
                            &preview_generator,
                        )
                        .await?;

                    content_previews.push(preview_content);
                }
                FileOperation::Rename { from, to } => {
                    files_deleted.push(from.clone());
                    files_created.push(to.clone());

                    let preview_content = ContentPreview {
                        file_path: from.clone(),
                        before: Some(format!("File: {}", from.display())),
                        after: format!("Renamed to: {}", to.display()),
                        change_type: ChangeType::Rename,
                    };

                    content_previews.push(preview_content);
                }
                _ => {}
            }
        }

        let change_summary = self
            .generate_change_summary(&files_created, &files_modified, &files_deleted)
            .await?;

        Ok(OperationPreview {
            files_created,
            files_modified,
            files_deleted,
            content_previews,
            change_summary,
        })
    }

    /// Generate backup strategy
    async fn generate_backup_strategy(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<BackupStrategy> {
        debug!("💾 Generating backup strategy");

        let mut files_to_backup = Vec::new();
        let mut backup_required = false;

        for operation in operations {
            match operation {
                FileOperation::Update { path, .. } | FileOperation::Delete { path } => {
                    files_to_backup.push(path.clone());
                    backup_required = true;
                }
                FileOperation::Rename { from, .. } => {
                    files_to_backup.push(from.clone());
                    backup_required = true;
                }
                _ => {}
            }
        }

        let backup_method = if context.git_commit.is_some() {
            BackupMethod::GitCommit
        } else if files_to_backup.len() > 10 {
            BackupMethod::Archive
        } else {
            BackupMethod::Copy
        };

        let backup_location = Some(PathBuf::from(format!(
            ".hive_backups/{}/",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        )));

        Ok(BackupStrategy {
            backup_required,
            backup_location,
            files_to_backup,
            backup_method,
        })
    }

    /// Generate rollback plan
    async fn generate_rollback_plan(
        &self,
        operations: &[FileOperation],
        backup_strategy: &BackupStrategy,
    ) -> Result<RollbackPlan> {
        debug!("🔄 Generating rollback plan");

        let mut rollback_steps = Vec::new();
        let mut complexity = 0.0;

        // Generate rollback steps in reverse order
        for (idx, operation) in operations.iter().enumerate().rev() {
            let (step, step_complexity) = self
                .generate_rollback_step(operation, backup_strategy, idx)
                .await?;

            rollback_steps.push(step);
            complexity += step_complexity;
        }

        // Add backup restoration if needed
        if backup_strategy.backup_required {
            rollback_steps.insert(
                0,
                RollbackStep {
                    description: "Restore files from backup".to_string(),
                    action: RollbackAction::RunCommand {
                        command: format!(
                            "hive restore-backup {:?}",
                            backup_strategy.backup_location
                        ),
                    },
                    validation: Some("Verify all files restored correctly".to_string()),
                },
            );
        }

        let automatic_rollback_possible = complexity < 50.0 && backup_strategy.backup_required;

        let mut manual_steps_required = Vec::new();
        if !automatic_rollback_possible {
            manual_steps_required.push("Manual review of changes required".to_string());
            if !backup_strategy.backup_required {
                manual_steps_required.push("No automatic backup available".to_string());
            }
        }

        Ok(RollbackPlan {
            rollback_steps,
            complexity,
            automatic_rollback_possible,
            manual_steps_required,
        })
    }

    /// Estimate execution time
    async fn estimate_execution_time(&self, operations: &[FileOperation]) -> Result<Duration> {
        let mut total_ms = 0u64;

        for operation in operations {
            let operation_time = match operation {
                FileOperation::Create { content, .. } => {
                    // Estimate based on content size
                    let size = content.len();
                    if size < 1000 {
                        100
                    } else if size < 10000 {
                        200
                    } else {
                        500
                    }
                }
                FileOperation::Update { .. } => 300, // Read + write
                FileOperation::Delete { .. } => 100,
                FileOperation::Rename { .. } => 200,
                _ => 100,
            };

            total_ms += operation_time;
        }

        // Add overhead for validation and checks
        total_ms += operations.len() as u64 * 50;

        Ok(Duration::from_millis(total_ms))
    }

    // Helper methods

    /// Group related operations
    async fn group_related_operations(
        &self,
        operations: &[FileOperation],
    ) -> Result<Vec<OperationGroup>> {
        let mut groups = Vec::new();
        let mut current_group = Vec::new();
        let mut current_dir = None;

        for operation in operations {
            let op_dir = self.get_operation_directory(operation);

            if current_dir.is_none() || current_dir == op_dir {
                current_dir = op_dir.clone();
                current_group.push(operation.clone());
            } else {
                // Start new group
                if !current_group.is_empty() {
                    groups.push(OperationGroup {
                        description: self.describe_operation_group(&current_group).await?,
                        operations: current_group.clone(),
                    });
                }
                current_group = vec![operation.clone()];
                current_dir = op_dir;
            }
        }

        // Add final group
        if !current_group.is_empty() {
            groups.push(OperationGroup {
                description: self.describe_operation_group(&current_group).await?,
                operations: current_group,
            });
        }

        Ok(groups)
    }

    /// Generate rollback strategies for operations
    pub async fn generate_rollback_strategies(
        &self,
        operations: &[FileOperation],
        operation_context: &OperationContext,
    ) -> Vec<RollbackStrategy> {
        let mut strategies = Vec::new();

        // Check Git availability
        if self.check_git_available() {
            strategies.push(RollbackStrategy {
                name: "Git-based Rollback".to_string(),
                description: "Use Git to revert changes to previous state".to_string(),
                confidence: 0.95,
                prerequisites: vec![
                    "Git repository".to_string(),
                    "All files tracked".to_string(),
                ],
                steps: vec![
                    "Create Git stash of current changes".to_string(),
                    "Revert to previous commit".to_string(),
                    "Restore specific files if needed".to_string(),
                ],
            });
        }

        // Backup-based strategy
        strategies.push(RollbackStrategy {
            name: "Backup Restoration".to_string(),
            description: "Restore files from automatic backups".to_string(),
            confidence: 0.90,
            prerequisites: vec![
                "Backup files exist".to_string(),
                "Sufficient disk space".to_string(),
            ],
            steps: vec![
                "Locate backup files".to_string(),
                "Verify backup integrity".to_string(),
                "Restore files from backups".to_string(),
                "Verify restoration success".to_string(),
            ],
        });

        // Manual reversal strategy
        strategies.push(RollbackStrategy {
            name: "Manual Operation Reversal".to_string(),
            description: "Reverse each operation manually".to_string(),
            confidence: 0.75,
            prerequisites: vec![
                "Original content available".to_string(),
                "Reversible operations".to_string(),
            ],
            steps: vec![
                "Identify reverse operations".to_string(),
                "Execute in reverse order".to_string(),
                "Verify each step".to_string(),
                "Handle dependencies".to_string(),
            ],
        });

        // Hybrid strategy
        strategies.push(RollbackStrategy {
            name: "Hybrid Approach".to_string(),
            description: "Combine multiple strategies for best results".to_string(),
            confidence: 0.85,
            prerequisites: vec!["Multiple strategies available".to_string()],
            steps: vec![
                "Use Git for tracked files".to_string(),
                "Use backups for untracked files".to_string(),
                "Manual reversal for complex cases".to_string(),
                "Comprehensive verification".to_string(),
            ],
        });

        strategies
    }

    /// Check if Git is available
    fn check_git_available(&self) -> bool {
        std::process::Command::new("git")
            .args(&["rev-parse", "--git-dir"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get operation directory
    fn get_operation_directory(&self, operation: &FileOperation) -> Option<PathBuf> {
        match operation {
            FileOperation::Create { path, .. }
            | FileOperation::Update { path, .. }
            | FileOperation::Delete { path } => path.parent().map(|p| p.to_path_buf()),
            FileOperation::Rename { from, .. } => from.parent().map(|p| p.to_path_buf()),
            _ => None,
        }
    }

    /// Describe operation group
    async fn describe_operation_group(&self, operations: &[FileOperation]) -> Result<String> {
        let mut creates = 0;
        let mut updates = 0;
        let mut deletes = 0;
        let mut renames = 0;

        for op in operations {
            match op {
                FileOperation::Create { .. } => creates += 1,
                FileOperation::Update { .. } => updates += 1,
                FileOperation::Delete { .. } => deletes += 1,
                FileOperation::Rename { .. } => renames += 1,
                _ => {}
            }
        }

        let mut parts = Vec::new();
        if creates > 0 {
            parts.push(format!("Create {} files", creates));
        }
        if updates > 0 {
            parts.push(format!("Update {} files", updates));
        }
        if deletes > 0 {
            parts.push(format!("Delete {} files", deletes));
        }
        if renames > 0 {
            parts.push(format!("Rename {} files", renames));
        }

        Ok(parts.join(", "))
    }

    /// Identify prerequisites
    async fn identify_prerequisites(
        &self,
        operations: &[FileOperation],
        previous_steps: &[ExecutionStep],
    ) -> Result<Vec<String>> {
        let mut prerequisites = Vec::new();

        // Check for directory creation needs
        let mut required_dirs = HashSet::new();
        for op in operations {
            if let FileOperation::Create { path, .. } = op {
                if let Some(parent) = path.parent() {
                    required_dirs.insert(parent.to_path_buf());
                }
            }
        }

        for dir in required_dirs {
            prerequisites.push(format!("Ensure directory {} exists", dir.display()));
        }

        // Check for dependencies from previous steps
        for step in previous_steps {
            for op in &step.operations {
                if self.operation_creates_dependency(op, operations) {
                    prerequisites.push(format!("Complete step {}", step.step_number));
                    break;
                }
            }
        }

        Ok(prerequisites)
    }

    /// Check if operation creates dependency
    fn operation_creates_dependency(
        &self,
        previous_op: &FileOperation,
        current_ops: &[FileOperation],
    ) -> bool {
        if let FileOperation::Create {
            path: created_path, ..
        } = previous_op
        {
            for op in current_ops {
                match op {
                    FileOperation::Update { path, .. } | FileOperation::Append { path, .. } => {
                        if path == created_path {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Generate validation checks
    async fn generate_validation_checks(
        &self,
        operations: &[FileOperation],
    ) -> Result<Vec<String>> {
        let mut checks = Vec::new();

        for op in operations {
            match op {
                FileOperation::Create { path, .. } => {
                    checks.push(format!("Verify {} was created", path.display()));
                    checks.push(format!("Check {} has correct permissions", path.display()));
                }
                FileOperation::Update { path, .. } => {
                    checks.push(format!("Verify {} was updated", path.display()));
                    checks.push(format!("Validate {} content", path.display()));
                }
                FileOperation::Delete { path } => {
                    checks.push(format!("Confirm {} was deleted", path.display()));
                }
                FileOperation::Rename { from, to } => {
                    checks.push(format!(
                        "Verify {} renamed to {}",
                        from.display(),
                        to.display()
                    ));
                }
                _ => {}
            }
        }

        // Add general validation
        checks.push("Run syntax validation on modified files".to_string());
        checks.push("Ensure no broken imports/references".to_string());

        Ok(checks)
    }

    /// Generate content preview
    async fn generate_content_preview(
        &self,
        path: &PathBuf,
        before: Option<&str>,
        after: Option<&str>,
        change_type: ChangeType,
        preview_generator: &PreviewGenerator,
    ) -> Result<ContentPreview> {
        let file_type = self.detect_file_type(path);

        // Truncate content for preview if needed
        let truncate_content = |content: &str| -> String {
            if content.len() <= self.config.max_preview_size {
                content.to_string()
            } else {
                format!(
                    "{}... (truncated, {} more bytes)",
                    &content[..self.config.max_preview_size],
                    content.len() - self.config.max_preview_size
                )
            }
        };

        let before_preview = before.map(truncate_content);
        let after_preview = match after {
            Some(content) => {
                // For new files, potentially use templates
                if change_type == ChangeType::Create && self.config.enable_operation_planning {
                    self.apply_preview_template(path, content, &file_type, preview_generator)
                        .await
                        .unwrap_or_else(|_| truncate_content(content))
                } else {
                    truncate_content(content)
                }
            }
            None => String::new(),
        };

        Ok(ContentPreview {
            file_path: path.clone(),
            before: before_preview,
            after: after_preview,
            change_type,
        })
    }

    /// Apply preview template
    async fn apply_preview_template(
        &self,
        path: &PathBuf,
        content: &str,
        file_type: &FileType,
        preview_generator: &PreviewGenerator,
    ) -> Result<String> {
        // Use LLM to enhance preview if content is minimal
        if content.len() < 100 {
            let template_key = match file_type {
                FileType::Rust => "rust_create",
                FileType::JavaScript | FileType::TypeScript => "js_create",
                FileType::Python => "python_create",
                _ => return Ok(content.to_string()),
            };

            if let Some(template) = preview_generator.templates.get(template_key) {
                // Use LLM to fill in template
                let prompt = format!(
                    "Given the file path '{}' and initial content:\n{}\n\n\
                     Generate appropriate content using this template:\n{}\n\n\
                     Fill in the placeholders appropriately.",
                    path.display(),
                    content,
                    template.template_content
                );

                let enhanced_content = self
                    .python_service
                    .generate_text(
                        &self.config.synthesis_model,
                        &prompt,
                        self.config.max_preview_size,
                        0.3, // Lower temperature for consistency
                    )
                    .await?;

                return Ok(enhanced_content);
            }
        }

        Ok(content.to_string())
    }

    /// Get current file content
    async fn get_current_file_content(&self, path: &PathBuf) -> Result<String> {
        // In a real implementation, this would read from the file system
        // For now, return a placeholder
        Ok(format!("// Current content of {}", path.display()))
    }

    /// Detect file type
    fn detect_file_type(&self, path: &PathBuf) -> FileType {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "rs" => FileType::Rust,
            "js" => FileType::JavaScript,
            "ts" | "tsx" => FileType::TypeScript,
            "py" => FileType::Python,
            "md" => FileType::Markdown,
            "json" => FileType::Json,
            "toml" => FileType::Toml,
            "yaml" | "yml" => FileType::Yaml,
            _ => FileType::Other(extension.to_string()),
        }
    }

    /// Generate change summary
    async fn generate_change_summary(
        &self,
        files_created: &[PathBuf],
        files_modified: &[PathBuf],
        files_deleted: &[PathBuf],
    ) -> Result<String> {
        let mut summary_parts = Vec::new();

        if !files_created.is_empty() {
            summary_parts.push(format!("{} files will be created", files_created.len()));
        }
        if !files_modified.is_empty() {
            summary_parts.push(format!("{} files will be modified", files_modified.len()));
        }
        if !files_deleted.is_empty() {
            summary_parts.push(format!("{} files will be deleted", files_deleted.len()));
        }

        if summary_parts.is_empty() {
            Ok("No file changes".to_string())
        } else {
            // Use LLM to generate a more detailed summary
            let basic_summary = summary_parts.join(", ");

            if self.config.enable_operation_planning {
                let prompt = format!(
                    "Generate a concise summary of these file operations:\n\
                     Created: {:?}\n\
                     Modified: {:?}\n\
                     Deleted: {:?}\n\n\
                     Provide a brief, user-friendly summary:",
                    files_created, files_modified, files_deleted
                );

                let detailed_summary = self
                    .python_service
                    .generate_text(&self.config.synthesis_model, &prompt, 200, 0.5)
                    .await
                    .unwrap_or(basic_summary.clone());

                Ok(detailed_summary)
            } else {
                Ok(basic_summary)
            }
        }
    }

    /// Generate rollback step
    async fn generate_rollback_step(
        &self,
        operation: &FileOperation,
        backup_strategy: &BackupStrategy,
        index: usize,
    ) -> Result<(RollbackStep, f32)> {
        match operation {
            FileOperation::Create { path, .. } => {
                Ok((
                    RollbackStep {
                        description: format!("Delete created file {}", path.display()),
                        action: RollbackAction::DeleteFile { path: path.clone() },
                        validation: Some(format!("Verify {} no longer exists", path.display())),
                    },
                    10.0, // Low complexity
                ))
            }
            FileOperation::Update { path, .. } => {
                if backup_strategy.backup_required {
                    Ok((
                        RollbackStep {
                            description: format!("Restore {} from backup", path.display()),
                            action: RollbackAction::RestoreFile {
                                from: backup_strategy
                                    .backup_location
                                    .as_ref()
                                    .map(|loc| loc.join(path.file_name().unwrap()))
                                    .unwrap_or_else(|| path.clone()),
                                to: path.clone(),
                            },
                            validation: Some(format!(
                                "Verify {} restored correctly",
                                path.display()
                            )),
                        },
                        20.0, // Medium complexity
                    ))
                } else {
                    Ok((
                        RollbackStep {
                            description: format!(
                                "Manual restoration required for {}",
                                path.display()
                            ),
                            action: RollbackAction::RunCommand {
                                command: format!(
                                    "echo 'Please manually restore {}'",
                                    path.display()
                                ),
                            },
                            validation: None,
                        },
                        50.0, // High complexity
                    ))
                }
            }
            FileOperation::Delete { path } => {
                Ok((
                    RollbackStep {
                        description: format!("Restore deleted file {}", path.display()),
                        action: RollbackAction::RestoreFile {
                            from: backup_strategy
                                .backup_location
                                .as_ref()
                                .map(|loc| loc.join(path.file_name().unwrap()))
                                .unwrap_or_else(|| path.clone()),
                            to: path.clone(),
                        },
                        validation: Some(format!("Verify {} exists", path.display())),
                    },
                    30.0, // Medium-high complexity
                ))
            }
            FileOperation::Rename { from, to } => {
                Ok((
                    RollbackStep {
                        description: format!("Rename {} back to {}", to.display(), from.display()),
                        action: RollbackAction::RunCommand {
                            command: format!("mv {:?} {:?}", to, from),
                        },
                        validation: Some(format!("Verify {} exists", from.display())),
                    },
                    15.0, // Low-medium complexity
                ))
            }
            _ => Ok((
                RollbackStep {
                    description: "Unknown operation rollback".to_string(),
                    action: RollbackAction::RunCommand {
                        command: "echo 'Manual rollback required'".to_string(),
                    },
                    validation: None,
                },
                50.0,
            )),
        }
    }

    /// Generate cache key for plan
    fn generate_plan_cache_key(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for op in operations {
            format!("{:?}", op).hash(&mut hasher);
        }
        context.source_question.hash(&mut hasher);

        format!("plan_{:x}", hasher.finish())
    }
}

/// Operation group for execution
struct OperationGroup {
    description: String,
    operations: Vec<FileOperation>,
}

/// Rollback strategy suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStrategy {
    /// Strategy name
    pub name: String,

    /// Strategy description
    pub description: String,

    /// Confidence in this strategy (0-1)
    pub confidence: f32,

    /// Prerequisites for this strategy
    pub prerequisites: Vec<String>,

    /// Steps to execute
    pub steps: Vec<String>,
}

/// Synthesis statistics
#[derive(Debug, Clone)]
pub struct SynthesisStats {
    pub total_insights: usize,
    pub average_confidence: f64,
    pub synthesis_patterns: usize,
    pub average_synthesis_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insight_generation() {
        // Test insight generation logic
    }
}
