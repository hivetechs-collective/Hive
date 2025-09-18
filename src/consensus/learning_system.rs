// Learning Feedback Loop System for Continuous AI Helper Improvement
use crate::ai_helpers::{
    context_retriever::ContextRetriever, knowledge_indexer::KnowledgeIndexer,
    knowledge_synthesizer::KnowledgeSynthesizer, pattern_recognizer::PatternRecognizer,
    quality_analyzer::QualityAnalyzer,
};
use crate::consensus::operation_intelligence::OperationIntelligenceCoordinator;
use crate::consensus::outcome_tracker::{
    AccuracyMetrics, ErrorPattern, HelperAccuracy, LearningInsights, OperationOutcomeTracker,
    RetrainingResult, TrackedOutcome,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Comprehensive learning system that continuously improves AI helper performance
pub struct ContinuousLearningSystem {
    outcome_tracker: Arc<OperationOutcomeTracker>,
    operation_intelligence: Arc<OperationIntelligenceCoordinator>,

    // AI Helper references for retraining
    knowledge_indexer: Arc<KnowledgeIndexer>,
    context_retriever: Arc<ContextRetriever>,
    pattern_recognizer: Arc<PatternRecognizer>,
    quality_analyzer: Arc<QualityAnalyzer>,
    knowledge_synthesizer: Arc<KnowledgeSynthesizer>,

    // Learning state management
    learning_state: Arc<RwLock<LearningState>>,
    configuration: LearningSystemConfig,

    // Performance tracking
    performance_monitor: Arc<PerformanceMonitor>,
    adaptation_engine: Arc<AdaptationEngine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSystemConfig {
    /// How often to run the learning cycle (in minutes)
    pub learning_cycle_interval_minutes: u64,
    /// Minimum improvement required to apply changes
    pub min_improvement_threshold: f32,
    /// Maximum weight adjustment per learning cycle
    pub max_weight_adjustment: f32,
    /// Whether to run learning in background automatically
    pub auto_learning_enabled: bool,
    /// Whether to apply learning results automatically
    pub auto_apply_learning: bool,
    /// Minimum confidence required for automatic application
    pub auto_apply_confidence_threshold: f32,
    /// Learning rate for gradual improvements
    pub learning_rate: f32,
    /// Maximum number of concurrent learning experiments
    pub max_concurrent_experiments: usize,
}

impl Default for LearningSystemConfig {
    fn default() -> Self {
        Self {
            learning_cycle_interval_minutes: 30,
            min_improvement_threshold: 0.02, // 2% improvement
            max_weight_adjustment: 0.2,      // ±20% weight change
            auto_learning_enabled: true,
            auto_apply_learning: false, // Require manual approval by default
            auto_apply_confidence_threshold: 0.9,
            learning_rate: 0.1,
            max_concurrent_experiments: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningState {
    pub last_learning_cycle: Option<DateTime<Utc>>,
    pub total_learning_cycles: u64,
    pub successful_improvements: u64,
    pub current_weights: HashMap<String, f32>,
    pub baseline_performance: Option<PerformanceBaseline>,
    pub pending_experiments: Vec<LearningExperiment>,
    pub learning_history: Vec<LearningCycleResult>,
}

impl Default for LearningState {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert("knowledge_indexer".to_string(), 1.0);
        weights.insert("context_retriever".to_string(), 1.0);
        weights.insert("pattern_recognizer".to_string(), 1.0);
        weights.insert("quality_analyzer".to_string(), 1.0);
        weights.insert("knowledge_synthesizer".to_string(), 1.0);

        Self {
            last_learning_cycle: None,
            total_learning_cycles: 0,
            successful_improvements: 0,
            current_weights: weights,
            baseline_performance: None,
            pending_experiments: Vec::new(),
            learning_history: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub overall_accuracy: f32,
    pub helper_accuracies: HashMap<String, f32>,
    pub confidence_error: f32,
    pub risk_error: f32,
    pub established_at: DateTime<Utc>,
    pub sample_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningExperiment {
    pub experiment_id: String,
    pub experiment_type: ExperimentType,
    pub proposed_changes: HashMap<String, f32>,
    pub hypothesis: String,
    pub expected_improvement: f32,
    pub started_at: DateTime<Utc>,
    pub status: ExperimentStatus,
    pub results: Option<ExperimentResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentType {
    WeightAdjustment,
    FeatureReweighting,
    ThresholdTuning,
    PatternLearning,
    HyperparameterOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Planning,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub performance_improvement: f32,
    pub accuracy_change: f32,
    pub confidence_calibration_change: f32,
    pub risk_assessment_change: f32,
    pub statistical_significance: f32,
    pub recommendation: ExperimentRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentRecommendation {
    Apply,
    ApplyWithModifications(HashMap<String, f32>),
    Reject,
    ExtendExperiment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningCycleResult {
    pub cycle_number: u64,
    pub timestamp: DateTime<Utc>,
    pub insights: LearningInsights,
    pub experiments_conducted: Vec<String>,
    pub improvements_applied: HashMap<String, f32>,
    pub performance_change: f32,
    pub duration_ms: u64,
}

impl ContinuousLearningSystem {
    pub fn new(
        outcome_tracker: Arc<OperationOutcomeTracker>,
        operation_intelligence: Arc<OperationIntelligenceCoordinator>,
        knowledge_indexer: Arc<KnowledgeIndexer>,
        context_retriever: Arc<ContextRetriever>,
        pattern_recognizer: Arc<PatternRecognizer>,
        quality_analyzer: Arc<QualityAnalyzer>,
        knowledge_synthesizer: Arc<KnowledgeSynthesizer>,
        config: Option<LearningSystemConfig>,
    ) -> Self {
        Self {
            outcome_tracker,
            operation_intelligence,
            knowledge_indexer,
            context_retriever,
            pattern_recognizer,
            quality_analyzer,
            knowledge_synthesizer,
            learning_state: Arc::new(RwLock::new(LearningState::default())),
            configuration: config.unwrap_or_default(),
            performance_monitor: Arc::new(PerformanceMonitor::new()),
            adaptation_engine: Arc::new(AdaptationEngine::new()),
        }
    }

    /// Start the continuous learning background process
    pub async fn start_continuous_learning(&self) -> Result<()> {
        if !self.configuration.auto_learning_enabled {
            info!("Continuous learning is disabled in configuration");
            return Ok(());
        }

        info!("Starting continuous learning system");

        let mut interval = interval(Duration::from_secs(
            self.configuration.learning_cycle_interval_minutes * 60,
        ));

        loop {
            interval.tick().await;

            if let Err(e) = self.run_learning_cycle().await {
                error!("Learning cycle failed: {}", e);
            }
        }
    }

    /// Run a single learning cycle
    pub async fn run_learning_cycle(&self) -> Result<LearningCycleResult> {
        let start_time = std::time::Instant::now();
        info!("Starting learning cycle");

        // Get current insights from outcome tracker
        let insights = self.outcome_tracker.get_learning_insights().await?;

        // Update performance baseline if needed
        self.update_performance_baseline(&insights).await?;

        // Plan learning experiments based on insights
        let experiments = self.plan_learning_experiments(&insights).await?;

        // Execute experiments
        let mut experiment_results = HashMap::new();
        for experiment in &experiments {
            if let Ok(result) = self.execute_experiment(experiment).await {
                experiment_results.insert(experiment.experiment_id.clone(), result);
            }
        }

        // Analyze results and determine improvements
        let improvements = self.analyze_experiment_results(&experiment_results).await?;

        // Apply improvements if configured
        let applied_improvements = if self.configuration.auto_apply_learning {
            self.apply_improvements(&improvements).await?
        } else {
            self.queue_improvements_for_approval(&improvements).await?
        };

        // Update learning state
        let cycle_result = {
            let mut state = self.learning_state.write().await;
            state.total_learning_cycles += 1;
            state.last_learning_cycle = Some(Utc::now());

            if !applied_improvements.is_empty() {
                state.successful_improvements += 1;
            }

            let result = LearningCycleResult {
                cycle_number: state.total_learning_cycles,
                timestamp: Utc::now(),
                insights,
                experiments_conducted: experiments
                    .iter()
                    .map(|e| e.experiment_id.clone())
                    .collect(),
                improvements_applied: applied_improvements.clone(),
                performance_change: self
                    .calculate_performance_change(&applied_improvements)
                    .await,
                duration_ms: start_time.elapsed().as_millis() as u64,
            };

            state.learning_history.push(result.clone());

            // Keep only recent history
            if state.learning_history.len() > 100 {
                state.learning_history.remove(0);
            }

            result
        };

        info!(
            "Learning cycle completed: {} experiments, {} improvements applied",
            experiments.len(),
            applied_improvements.len()
        );

        Ok(cycle_result)
    }

    /// Get pending improvements that require manual approval
    pub async fn get_pending_improvements(&self) -> Vec<PendingImprovement> {
        let state = self.learning_state.read().await;

        state
            .pending_experiments
            .iter()
            .filter(|exp| matches!(exp.status, ExperimentStatus::Completed))
            .filter_map(|exp| {
                exp.results.as_ref().map(|result| PendingImprovement {
                    experiment_id: exp.experiment_id.clone(),
                    description: exp.hypothesis.clone(),
                    proposed_changes: exp.proposed_changes.clone(),
                    expected_improvement: result.performance_improvement,
                    confidence: result.statistical_significance,
                    recommendation: result.recommendation.clone(),
                })
            })
            .collect()
    }

    /// Manually approve and apply a pending improvement
    pub async fn approve_improvement(&self, experiment_id: &str) -> Result<()> {
        let experiment = {
            let mut state = self.learning_state.write().await;
            let experiment_index = state
                .pending_experiments
                .iter()
                .position(|exp| exp.experiment_id == experiment_id)
                .context("Experiment not found")?;

            state.pending_experiments.remove(experiment_index)
        };

        if let Some(result) = experiment.results {
            match result.recommendation {
                ExperimentRecommendation::Apply => {
                    self.apply_weight_changes(&experiment.proposed_changes)
                        .await?;
                    info!("Applied improvement from experiment: {}", experiment_id);
                }
                ExperimentRecommendation::ApplyWithModifications(ref changes) => {
                    self.apply_weight_changes(changes).await?;
                    info!(
                        "Applied modified improvement from experiment: {}",
                        experiment_id
                    );
                }
                _ => {
                    warn!(
                        "Cannot apply improvement with recommendation: {:?}",
                        result.recommendation
                    );
                }
            }
        }

        Ok(())
    }

    /// Reject a pending improvement
    pub async fn reject_improvement(&self, experiment_id: &str) -> Result<()> {
        let mut state = self.learning_state.write().await;
        let experiment_index = state
            .pending_experiments
            .iter()
            .position(|exp| exp.experiment_id == experiment_id)
            .context("Experiment not found")?;

        state.pending_experiments.remove(experiment_index);
        info!("Rejected improvement from experiment: {}", experiment_id);
        Ok(())
    }

    /// Get learning system status and metrics
    pub async fn get_learning_status(&self) -> LearningSystemStatus {
        let state = self.learning_state.read().await;
        let accuracy_metrics = self.outcome_tracker.get_accuracy_statistics().await;

        LearningSystemStatus {
            is_active: self.configuration.auto_learning_enabled,
            total_cycles: state.total_learning_cycles,
            successful_improvements: state.successful_improvements,
            last_cycle: state.last_learning_cycle,
            current_weights: state.current_weights.clone(),
            current_performance: accuracy_metrics,
            pending_improvements: state.pending_experiments.len(),
            learning_trend: self.calculate_learning_trend(&state.learning_history),
        }
    }

    // Private helper methods

    async fn update_performance_baseline(&self, insights: &LearningInsights) -> Result<()> {
        let mut state = self.learning_state.write().await;

        if state.baseline_performance.is_none() {
            // Establish initial baseline
            state.baseline_performance = Some(PerformanceBaseline {
                overall_accuracy: insights.average_confidence_error,
                helper_accuracies: insights
                    .helper_performance
                    .iter()
                    .map(|(k, v)| (k.clone(), v.prediction_accuracy))
                    .collect(),
                confidence_error: insights.average_confidence_error,
                risk_error: insights.average_risk_error,
                established_at: Utc::now(),
                sample_size: insights.total_outcomes,
            });

            info!("Established performance baseline");
        }

        Ok(())
    }

    async fn plan_learning_experiments(
        &self,
        insights: &LearningInsights,
    ) -> Result<Vec<LearningExperiment>> {
        let mut experiments = Vec::new();

        // Experiment 1: Adjust weights for poorly performing helpers
        for (helper_name, accuracy) in &insights.helper_performance {
            if accuracy.prediction_accuracy < 0.7 {
                let weight_adjustment = if accuracy.prediction_accuracy < 0.5 {
                    -0.2 // Reduce weight significantly
                } else {
                    -0.1 // Reduce weight moderately
                };

                experiments.push(LearningExperiment {
                    experiment_id: format!("weight_adj_{}_{}", helper_name, Utc::now().timestamp()),
                    experiment_type: ExperimentType::WeightAdjustment,
                    proposed_changes: {
                        let mut changes = HashMap::new();
                        changes.insert(helper_name.clone(), weight_adjustment);
                        changes
                    },
                    hypothesis: format!(
                        "Reducing weight of {} should improve overall accuracy",
                        helper_name
                    ),
                    expected_improvement: 0.05,
                    started_at: Utc::now(),
                    status: ExperimentStatus::Planning,
                    results: None,
                });
            }
        }

        // Experiment 2: Address common error patterns
        for pattern in &insights.error_patterns {
            if pattern.frequency > 0.1 {
                // If pattern occurs in >10% of cases
                experiments.push(LearningExperiment {
                    experiment_id: format!(
                        "pattern_fix_{}_{}",
                        pattern.pattern_type,
                        Utc::now().timestamp()
                    ),
                    experiment_type: ExperimentType::PatternLearning,
                    proposed_changes: self.pattern_to_weight_adjustments(pattern),
                    hypothesis: format!(
                        "Addressing {} pattern should reduce error rate",
                        pattern.pattern_type
                    ),
                    expected_improvement: pattern.frequency * 0.5, // Expect to fix 50% of pattern occurrences
                    started_at: Utc::now(),
                    status: ExperimentStatus::Planning,
                    results: None,
                });
            }
        }

        // Experiment 3: Boost high-performing helpers
        for (helper_name, accuracy) in &insights.helper_performance {
            if accuracy.prediction_accuracy > 0.85 && accuracy.suggested_weight_adjustment > 1.0 {
                experiments.push(LearningExperiment {
                    experiment_id: format!("boost_{}_{}", helper_name, Utc::now().timestamp()),
                    experiment_type: ExperimentType::WeightAdjustment,
                    proposed_changes: {
                        let mut changes = HashMap::new();
                        changes.insert(helper_name.clone(), 0.1); // Increase weight by 10%
                        changes
                    },
                    hypothesis: format!(
                        "Increasing weight of high-performing {} should improve accuracy",
                        helper_name
                    ),
                    expected_improvement: 0.03,
                    started_at: Utc::now(),
                    status: ExperimentStatus::Planning,
                    results: None,
                });
            }
        }

        // Limit concurrent experiments
        experiments.truncate(self.configuration.max_concurrent_experiments);

        Ok(experiments)
    }

    async fn execute_experiment(
        &self,
        experiment: &LearningExperiment,
    ) -> Result<ExperimentResult> {
        info!("Executing experiment: {}", experiment.experiment_id);

        // Simulate experiment execution (in real implementation, this would run actual tests)
        let baseline_performance = self.performance_monitor.get_current_performance().await;

        // Apply temporary changes for testing
        self.apply_temporary_changes(&experiment.proposed_changes)
            .await?;

        // Run performance evaluation
        let test_performance = self
            .performance_monitor
            .evaluate_performance_sample(50)
            .await?;

        // Revert changes
        self.revert_temporary_changes(&experiment.proposed_changes)
            .await?;

        // Calculate results
        let performance_improvement =
            test_performance.overall_accuracy - baseline_performance.overall_accuracy;
        let accuracy_change =
            test_performance.overall_accuracy - baseline_performance.overall_accuracy;

        let recommendation =
            if performance_improvement > self.configuration.min_improvement_threshold {
                if performance_improvement > 0.1 {
                    ExperimentRecommendation::Apply
                } else {
                    // Apply with reduced magnitude
                    let reduced_changes: HashMap<String, f32> = experiment
                        .proposed_changes
                        .iter()
                        .map(|(k, v)| (k.clone(), v * 0.5))
                        .collect();
                    ExperimentRecommendation::ApplyWithModifications(reduced_changes)
                }
            } else {
                ExperimentRecommendation::Reject
            };

        Ok(ExperimentResult {
            performance_improvement,
            accuracy_change,
            confidence_calibration_change: 0.0, // Would be calculated from actual metrics
            risk_assessment_change: 0.0,        // Would be calculated from actual metrics
            statistical_significance: 0.8,      // Would be calculated from statistical tests
            recommendation,
        })
    }

    async fn analyze_experiment_results(
        &self,
        results: &HashMap<String, ExperimentResult>,
    ) -> Result<Vec<WeightAdjustment>> {
        let mut adjustments = Vec::new();

        for (experiment_id, result) in results {
            if result.performance_improvement > self.configuration.min_improvement_threshold {
                adjustments.push(WeightAdjustment {
                    experiment_id: experiment_id.clone(),
                    adjustment_type: AdjustmentType::WeightChange,
                    changes: HashMap::new(), // Would be populated from experiment
                    confidence: result.statistical_significance,
                    expected_improvement: result.performance_improvement,
                });
            }
        }

        Ok(adjustments)
    }

    async fn apply_improvements(
        &self,
        improvements: &[WeightAdjustment],
    ) -> Result<HashMap<String, f32>> {
        let mut applied = HashMap::new();

        for improvement in improvements {
            if improvement.confidence > self.configuration.auto_apply_confidence_threshold {
                for (helper, adjustment) in &improvement.changes {
                    applied.insert(helper.clone(), *adjustment);
                }
                self.apply_weight_changes(&improvement.changes).await?;
            }
        }

        Ok(applied)
    }

    async fn queue_improvements_for_approval(
        &self,
        improvements: &[WeightAdjustment],
    ) -> Result<HashMap<String, f32>> {
        let mut state = self.learning_state.write().await;

        for improvement in improvements {
            let experiment = LearningExperiment {
                experiment_id: improvement.experiment_id.clone(),
                experiment_type: ExperimentType::WeightAdjustment,
                proposed_changes: improvement.changes.clone(),
                hypothesis: "Automated learning experiment".to_string(),
                expected_improvement: improvement.expected_improvement,
                started_at: Utc::now(),
                status: ExperimentStatus::Completed,
                results: Some(ExperimentResult {
                    performance_improvement: improvement.expected_improvement,
                    accuracy_change: improvement.expected_improvement,
                    confidence_calibration_change: 0.0,
                    risk_assessment_change: 0.0,
                    statistical_significance: improvement.confidence,
                    recommendation: ExperimentRecommendation::Apply,
                }),
            };

            state.pending_experiments.push(experiment);
        }

        info!(
            "Queued {} improvements for manual approval",
            improvements.len()
        );
        Ok(HashMap::new()) // No automatically applied improvements
    }

    async fn apply_weight_changes(&self, changes: &HashMap<String, f32>) -> Result<()> {
        let mut state = self.learning_state.write().await;

        for (helper_name, adjustment) in changes {
            let current_weight = state
                .current_weights
                .get(helper_name)
                .copied()
                .unwrap_or(1.0);
            let new_weight = (current_weight + adjustment).max(0.1).min(2.0); // Clamp between 0.1 and 2.0

            state
                .current_weights
                .insert(helper_name.clone(), new_weight);
            info!(
                "Updated {} weight: {:.3} -> {:.3}",
                helper_name, current_weight, new_weight
            );
        }

        // Apply changes to operation intelligence coordinator
        // In a real implementation, this would update the actual AI helper weights

        Ok(())
    }

    async fn apply_temporary_changes(&self, _changes: &HashMap<String, f32>) -> Result<()> {
        // Temporarily apply changes for testing
        Ok(())
    }

    async fn revert_temporary_changes(&self, _changes: &HashMap<String, f32>) -> Result<()> {
        // Revert temporary changes
        Ok(())
    }

    async fn calculate_performance_change(&self, _improvements: &HashMap<String, f32>) -> f32 {
        // Calculate overall performance change from applied improvements
        0.0 // Placeholder
    }

    fn calculate_learning_trend(&self, history: &[LearningCycleResult]) -> LearningTrend {
        if history.len() < 2 {
            return LearningTrend::Insufficient;
        }

        let recent_improvements: f32 = history
            .iter()
            .rev()
            .take(5)
            .map(|cycle| cycle.performance_change)
            .sum();

        if recent_improvements > 0.05 {
            LearningTrend::Improving
        } else if recent_improvements < -0.05 {
            LearningTrend::Declining
        } else {
            LearningTrend::Stable
        }
    }

    fn pattern_to_weight_adjustments(&self, pattern: &ErrorPattern) -> HashMap<String, f32> {
        let mut adjustments = HashMap::new();

        match pattern.pattern_type.as_str() {
            "overconfidence" => {
                adjustments.insert("quality_analyzer".to_string(), 0.1); // Increase risk assessment
                adjustments.insert("pattern_recognizer".to_string(), 0.1); // Improve caution
            }
            "risk_underestimation" => {
                adjustments.insert("quality_analyzer".to_string(), 0.15); // Significantly increase risk assessment
                adjustments.insert("context_retriever".to_string(), 0.05); // Better historical context
            }
            _ => {}
        }

        adjustments
    }
}

// Supporting structures and types

pub struct PerformanceMonitor {
    // Performance monitoring implementation
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_current_performance(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            overall_accuracy: 0.75,
            confidence_error: 15.0,
            risk_error: 12.0,
        }
    }

    pub async fn evaluate_performance_sample(
        &self,
        _sample_size: usize,
    ) -> Result<PerformanceMetrics> {
        Ok(PerformanceMetrics {
            overall_accuracy: 0.78,
            confidence_error: 12.0,
            risk_error: 10.0,
        })
    }
}

pub struct AdaptationEngine {
    // Adaptation algorithm implementation
}

impl AdaptationEngine {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub overall_accuracy: f32,
    pub confidence_error: f32,
    pub risk_error: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightAdjustment {
    pub experiment_id: String,
    pub adjustment_type: AdjustmentType,
    pub changes: HashMap<String, f32>,
    pub confidence: f32,
    pub expected_improvement: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentType {
    WeightChange,
    ThresholdAdjustment,
    FeatureReweighting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingImprovement {
    pub experiment_id: String,
    pub description: String,
    pub proposed_changes: HashMap<String, f32>,
    pub expected_improvement: f32,
    pub confidence: f32,
    pub recommendation: ExperimentRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSystemStatus {
    pub is_active: bool,
    pub total_cycles: u64,
    pub successful_improvements: u64,
    pub last_cycle: Option<DateTime<Utc>>,
    pub current_weights: HashMap<String, f32>,
    pub current_performance: AccuracyMetrics,
    pub pending_improvements: usize,
    pub learning_trend: LearningTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningTrend {
    Improving,
    Stable,
    Declining,
    Insufficient,
}
