//! User Preference Learning and Adaptation
//!
//! Learns from user behavior to improve mode detection and switching
//! recommendations over time.

use crate::core::error::{HiveError, HiveResult};
use crate::modes::detector::DetectionResult;
use crate::modes::switcher::SwitchResult;
use crate::planning::{ModeType, PlanningContext, UserPreferences as BasePreferences};
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Preference manager with learning capabilities
pub struct PreferenceManager {
    storage: Arc<RwLock<PreferenceStorage>>,
    learner: PatternLearner,
    analyzer: BehaviorAnalyzer,
    predictor: PreferencePredictor,
}

/// Storage for user preferences and learning data
#[derive(Debug)]
struct PreferenceStorage {
    user_preferences: UserPreference,
    learning_data: LearningData,
    pattern_history: PatternHistory,
}

/// Pattern learner for user behavior
#[derive(Debug)]
struct PatternLearner {
    algorithm: LearningAlgorithm,
    confidence_threshold: f32,
    min_samples: usize,
}

/// Behavior analyzer
#[derive(Debug)]
struct BehaviorAnalyzer {
    time_windows: Vec<TimeWindow>,
    metrics: BehaviorMetrics,
}

/// Preference predictor
#[derive(Debug)]
struct PreferencePredictor {
    models: HashMap<PredictionType, Box<dyn PredictionModel + Send + Sync>>,
}

/// Learning algorithm type
#[derive(Debug, Clone, PartialEq)]
enum LearningAlgorithm {
    FrequencyBased,
    TimeSeries,
    ContextualBandits,
    ReinforcementLearning,
}

/// Time window for analysis
#[derive(Debug, Clone)]
struct TimeWindow {
    name: String,
    duration: chrono::Duration,
}

/// Behavior metrics
#[derive(Debug, Clone)]
struct BehaviorMetrics {
    mode_preferences: HashMap<ModeType, f32>,
    switch_patterns: HashMap<(ModeType, ModeType), f32>,
    time_of_day_patterns: HashMap<u32, ModeType>,
    task_type_patterns: HashMap<String, ModeType>,
}

/// Type of prediction
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum PredictionType {
    NextMode,
    OptimalSwitch,
    TaskTypeMode,
}

/// Prediction model trait
trait PredictionModel: std::fmt::Debug {
    fn predict(&self, input: &PredictionInput) -> PredictionOutput;
    fn update(&mut self, feedback: &PredictionFeedback);
}

/// User preferences with learning enhancements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    pub base_preferences: BasePreferences,
    pub learning_enabled: bool,
    pub adaptation_rate: f32,
    pub confidence_weights: ConfidenceWeights,
    pub mode_biases: HashMap<ModeType, f32>,
}

/// Weights for different confidence factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceWeights {
    pub historical: f32,
    pub contextual: f32,
    pub temporal: f32,
    pub success_rate: f32,
}

/// Learning data accumulated over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningData {
    pub total_detections: usize,
    pub total_switches: usize,
    pub detection_history: Vec<DetectionRecord>,
    pub switch_history: Vec<SwitchRecord>,
    pub pattern_cache: HashMap<String, Pattern>,
}

/// Record of a detection event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRecord {
    pub query: String,
    pub detected_mode: ModeType,
    pub actual_mode: Option<ModeType>,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
    pub context_hash: String,
}

/// Record of a switch event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchRecord {
    pub from_mode: ModeType,
    pub to_mode: ModeType,
    pub success: bool,
    pub duration: std::time::Duration,
    pub timestamp: DateTime<Utc>,
    pub user_initiated: bool,
}

/// Learned pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub confidence: f32,
    pub occurrences: usize,
    pub last_seen: DateTime<Utc>,
    pub data: PatternData,
}

/// Type of pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    QueryMode,
    SwitchSequence,
    TimeBasedMode,
    ContextualMode,
}

/// Pattern data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternData {
    QueryMode {
        query_pattern: String,
        preferred_mode: ModeType,
    },
    SwitchSequence {
        sequence: Vec<ModeType>,
        frequency: f32,
    },
    TimeBasedMode {
        hour_range: (u32, u32),
        preferred_mode: ModeType,
    },
    ContextualMode {
        context_features: Vec<String>,
        mode_affinity: HashMap<ModeType, f32>,
    },
}

/// History of patterns
#[derive(Debug)]
struct PatternHistory {
    patterns: Vec<Pattern>,
    max_patterns: usize,
    retention_days: i64,
}

/// Input for prediction
#[derive(Debug)]
struct PredictionInput {
    query: Option<String>,
    current_mode: ModeType,
    context: PlanningContext,
    timestamp: DateTime<Utc>,
}

/// Output from prediction
#[derive(Debug)]
struct PredictionOutput {
    prediction: ModeType,
    confidence: f32,
    alternatives: Vec<(ModeType, f32)>,
}

/// Feedback for prediction
#[derive(Debug)]
struct PredictionFeedback {
    predicted: ModeType,
    actual: ModeType,
    success: bool,
}

impl PreferenceManager {
    /// Create a new preference manager
    pub async fn new() -> HiveResult<Self> {
        Ok(Self {
            storage: Arc::new(RwLock::new(PreferenceStorage::new())),
            learner: PatternLearner::new(),
            analyzer: BehaviorAnalyzer::new(),
            predictor: PreferencePredictor::new(),
        })
    }

    /// Enhance context with learned preferences
    pub async fn enhance_context(
        &self,
        mut context: PlanningContext,
    ) -> HiveResult<PlanningContext> {
        let storage = self.storage.read().await;

        // Apply learned mode biases
        if let Some(bias) = storage
            .user_preferences
            .mode_biases
            .get(&context.user_preferences.preferred_mode)
        {
            context.user_preferences.preference_strength += bias * 0.3;
            context.user_preferences.preference_strength =
                context.user_preferences.preference_strength.clamp(0.0, 1.0);
        }

        // Apply temporal patterns
        let current_hour = Utc::now().time().hour() as u32;
        if let Some(pattern) = self.find_time_pattern(&storage.learning_data, current_hour) {
            if let PatternData::TimeBasedMode { preferred_mode, .. } = &pattern.data {
                context.user_preferences.preferred_mode = preferred_mode.clone();
            }
        }

        Ok(context)
    }

    /// Learn from detection result
    pub async fn learn_from_detection(
        &mut self,
        query: &str,
        result: &DetectionResult,
    ) -> HiveResult<()> {
        let mut storage = self.storage.write().await;

        // Record detection
        let record = DetectionRecord {
            query: query.to_string(),
            detected_mode: result.primary_mode.clone(),
            actual_mode: None, // Will be updated if user overrides
            confidence: result.confidence,
            timestamp: Utc::now(),
            context_hash: self.hash_context(&result),
        };

        storage.learning_data.detection_history.push(record.clone());
        storage.learning_data.total_detections += 1;

        // Learn patterns
        if storage.user_preferences.learning_enabled {
            let patterns = self
                .learner
                .learn_from_detection(&record, &storage.learning_data)?;
            for pattern in patterns {
                storage.learning_data.pattern_cache.insert(
                    format!("{:?}_{}", pattern.pattern_type, pattern.occurrences),
                    pattern,
                );
            }
        }

        // Update behavior metrics
        self.analyzer
            .update_detection_metrics(&mut storage.learning_data, &result.primary_mode);

        // Trim history if needed
        self.trim_history(&mut storage.learning_data);

        Ok(())
    }

    /// Learn from switch result
    pub async fn learn_from_switch(&mut self, result: &SwitchResult) -> HiveResult<()> {
        let mut storage = self.storage.write().await;

        // Record switch
        let record = SwitchRecord {
            from_mode: result.from_mode.clone(),
            to_mode: result.to_mode.clone(),
            success: result.success,
            duration: result.duration,
            timestamp: Utc::now(),
            user_initiated: true, // TODO: Track this properly
        };

        storage.learning_data.switch_history.push(record.clone());
        storage.learning_data.total_switches += 1;

        // Learn patterns
        if storage.user_preferences.learning_enabled && result.success {
            let patterns = self
                .learner
                .learn_from_switch(&record, &storage.learning_data)?;
            for pattern in patterns {
                storage.learning_data.pattern_cache.insert(
                    format!("{:?}_switch_{}", pattern.pattern_type, pattern.occurrences),
                    pattern,
                );
            }
        }

        // Update mode biases based on success
        if result.success {
            let bias = storage
                .user_preferences
                .mode_biases
                .entry(result.to_mode.clone())
                .or_insert(0.0);
            *bias = (*bias * 0.9 + 0.1).min(0.5); // Slowly increase bias, cap at 0.5
        }

        Ok(())
    }

    /// Learn from hybrid execution
    pub async fn learn_from_hybrid_execution(
        &mut self,
        task: &super::hybrid::HybridTask,
    ) -> HiveResult<()> {
        let mut storage = self.storage.write().await;

        // Analyze mode sequence effectiveness
        let sequence: Vec<ModeType> = task.segments.iter().map(|s| s.mode.clone()).collect();

        // Create sequence pattern
        let pattern = Pattern {
            pattern_type: PatternType::SwitchSequence,
            confidence: 0.8, // High confidence for completed sequences
            occurrences: 1,
            last_seen: Utc::now(),
            data: PatternData::SwitchSequence {
                sequence: sequence.clone(),
                frequency: 1.0,
            },
        };

        storage
            .learning_data
            .pattern_cache
            .insert(format!("hybrid_seq_{}", task.id), pattern);

        Ok(())
    }

    /// Update user preferences
    pub async fn update_preferences(&mut self, preferences: UserPreference) -> HiveResult<()> {
        let mut storage = self.storage.write().await;
        storage.user_preferences = preferences;
        Ok(())
    }

    /// Get learning statistics
    pub async fn get_learning_stats(&self) -> HiveResult<super::LearningStats> {
        let storage = self.storage.read().await;

        // Calculate mode distribution
        let mut mode_distribution = HashMap::new();
        for record in &storage.learning_data.detection_history {
            *mode_distribution
                .entry(record.detected_mode.clone())
                .or_insert(0) += 1;
        }

        // Find top patterns
        let mut pattern_counts: HashMap<String, usize> = HashMap::new();
        for pattern in storage.learning_data.pattern_cache.values() {
            let key = format!("{:?}", pattern.pattern_type);
            *pattern_counts.entry(key).or_insert(0) += pattern.occurrences;
        }

        let mut top_patterns: Vec<(String, usize)> = pattern_counts.into_iter().collect();
        top_patterns.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        top_patterns.truncate(5);

        // Calculate accuracy
        let correct_detections = storage
            .learning_data
            .detection_history
            .iter()
            .filter(|r| r.actual_mode.as_ref() == Some(&r.detected_mode))
            .count();

        let mode_accuracy = if storage.learning_data.total_detections > 0 {
            correct_detections as f32 / storage.learning_data.total_detections as f32
        } else {
            1.0
        };

        Ok(super::LearningStats {
            total_detections: storage.learning_data.total_detections,
            total_switches: storage.learning_data.total_switches,
            mode_accuracy,
            preference_influence: storage.user_preferences.adaptation_rate,
            top_patterns,
            mode_distribution,
        })
    }

    /// Predict next mode based on patterns
    pub async fn predict_next_mode(
        &self,
        query: &str,
        context: &PlanningContext,
    ) -> HiveResult<ModeType> {
        let storage = self.storage.read().await;

        let input = PredictionInput {
            query: Some(query.to_string()),
            current_mode: context.user_preferences.preferred_mode.clone(),
            context: context.clone(),
            timestamp: Utc::now(),
        };

        let output = self.predictor.predict(&input, &storage.learning_data);
        Ok(output.prediction)
    }

    // Private helper methods

    fn find_time_pattern<'a>(&self, data: &'a LearningData, hour: u32) -> Option<&'a Pattern> {
        data.pattern_cache
            .values()
            .filter(|p| p.pattern_type == PatternType::TimeBasedMode)
            .find(|p| {
                if let PatternData::TimeBasedMode { hour_range, .. } = &p.data {
                    hour >= hour_range.0 && hour <= hour_range.1
                } else {
                    false
                }
            })
    }

    fn hash_context(&self, result: &DetectionResult) -> String {
        // Simple hash of key context elements
        format!("{:?}_{}", result.primary_mode, result.confidence as i32)
    }

    fn trim_history(&self, data: &mut LearningData) {
        const MAX_DETECTION_HISTORY: usize = 1000;
        const MAX_SWITCH_HISTORY: usize = 500;

        if data.detection_history.len() > MAX_DETECTION_HISTORY {
            let excess = data.detection_history.len() - MAX_DETECTION_HISTORY;
            data.detection_history.drain(0..excess);
        }

        if data.switch_history.len() > MAX_SWITCH_HISTORY {
            let excess = data.switch_history.len() - MAX_SWITCH_HISTORY;
            data.switch_history.drain(0..excess);
        }
    }
}

impl PreferenceStorage {
    fn new() -> Self {
        Self {
            user_preferences: UserPreference::default(),
            learning_data: LearningData::new(),
            pattern_history: PatternHistory::new(),
        }
    }
}

impl PatternLearner {
    fn new() -> Self {
        Self {
            algorithm: LearningAlgorithm::FrequencyBased,
            confidence_threshold: 0.7,
            min_samples: 3,
        }
    }

    fn learn_from_detection(
        &self,
        record: &DetectionRecord,
        data: &LearningData,
    ) -> HiveResult<Vec<Pattern>> {
        let mut patterns = Vec::new();

        // Query-mode pattern
        let similar_queries = data
            .detection_history
            .iter()
            .filter(|r| self.is_similar_query(&r.query, &record.query))
            .collect::<Vec<_>>();

        if similar_queries.len() >= self.min_samples {
            let mode_counts = self.count_modes(&similar_queries);
            if let Some((mode, count)) = mode_counts.iter().max_by_key(|(_, c)| *c) {
                let confidence = *count as f32 / similar_queries.len() as f32;
                if confidence >= self.confidence_threshold {
                    patterns.push(Pattern {
                        pattern_type: PatternType::QueryMode,
                        confidence,
                        occurrences: *count,
                        last_seen: record.timestamp,
                        data: PatternData::QueryMode {
                            query_pattern: self.extract_pattern(&record.query),
                            preferred_mode: mode.clone(),
                        },
                    });
                }
            }
        }

        Ok(patterns)
    }

    fn learn_from_switch(
        &self,
        record: &SwitchRecord,
        data: &LearningData,
    ) -> HiveResult<Vec<Pattern>> {
        let mut patterns = Vec::new();

        // Switch sequence pattern
        let recent_switches: Vec<ModeType> = data
            .switch_history
            .iter()
            .rev()
            .take(5)
            .map(|r| r.to_mode.clone())
            .collect();

        if recent_switches.len() >= 3 {
            patterns.push(Pattern {
                pattern_type: PatternType::SwitchSequence,
                confidence: 0.6,
                occurrences: 1,
                last_seen: record.timestamp,
                data: PatternData::SwitchSequence {
                    sequence: recent_switches,
                    frequency: 1.0,
                },
            });
        }

        Ok(patterns)
    }

    fn is_similar_query(&self, q1: &str, q2: &str) -> bool {
        // Simple similarity check - in real implementation would use better metrics
        let words1: Vec<&str> = q1.split_whitespace().collect();
        let words2: Vec<&str> = q2.split_whitespace().collect();

        let common_words = words1.iter().filter(|w| words2.contains(w)).count();

        let similarity = common_words as f32 / words1.len().max(words2.len()) as f32;
        similarity > 0.5
    }

    fn count_modes(&self, records: &[&DetectionRecord]) -> HashMap<ModeType, usize> {
        let mut counts = HashMap::new();
        for record in records {
            *counts.entry(record.detected_mode.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn extract_pattern(&self, query: &str) -> String {
        // Extract key pattern from query
        let key_words = ["implement", "plan", "analyze", "design", "fix", "create"];

        query
            .split_whitespace()
            .filter(|w| key_words.contains(&w.to_lowercase().as_str()))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl BehaviorAnalyzer {
    fn new() -> Self {
        Self {
            time_windows: vec![
                TimeWindow {
                    name: "Last Hour".to_string(),
                    duration: chrono::Duration::hours(1),
                },
                TimeWindow {
                    name: "Last Day".to_string(),
                    duration: chrono::Duration::days(1),
                },
                TimeWindow {
                    name: "Last Week".to_string(),
                    duration: chrono::Duration::weeks(1),
                },
            ],
            metrics: BehaviorMetrics {
                mode_preferences: HashMap::new(),
                switch_patterns: HashMap::new(),
                time_of_day_patterns: HashMap::new(),
                task_type_patterns: HashMap::new(),
            },
        }
    }

    fn update_detection_metrics(&mut self, data: &mut LearningData, mode: &ModeType) {
        // Update mode preference
        let count = self
            .metrics
            .mode_preferences
            .entry(mode.clone())
            .or_insert(0.0);
        *count += 1.0;

        // Normalize preferences
        let total: f32 = self.metrics.mode_preferences.values().sum();
        if total > 0.0 {
            for value in self.metrics.mode_preferences.values_mut() {
                *value /= total;
            }
        }

        // Update time of day pattern
        let hour = Utc::now().time().hour();
        self.metrics.time_of_day_patterns.insert(hour, mode.clone());
    }
}

impl PreferencePredictor {
    fn new() -> Self {
        let mut models: HashMap<PredictionType, Box<dyn PredictionModel + Send + Sync>> =
            HashMap::new();

        models.insert(PredictionType::NextMode, Box::new(FrequencyModel::new()));
        models.insert(
            PredictionType::OptimalSwitch,
            Box::new(TransitionModel::new()),
        );
        models.insert(PredictionType::TaskTypeMode, Box::new(ContextModel::new()));

        Self { models }
    }

    fn predict(&self, input: &PredictionInput, data: &LearningData) -> PredictionOutput {
        // Use ensemble of models
        let mut predictions = Vec::new();

        for (pred_type, model) in &self.models {
            let output = model.predict(input);
            predictions.push((output.prediction, output.confidence));
        }

        // Weighted average
        let mut mode_scores: HashMap<ModeType, f32> = HashMap::new();
        for (mode, confidence) in predictions {
            *mode_scores.entry(mode).or_insert(0.0) += confidence;
        }

        let (best_mode, best_score) = mode_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(m, s)| (m.clone(), *s))
            .unwrap_or((ModeType::Hybrid, 0.5));

        PredictionOutput {
            prediction: best_mode,
            confidence: best_score / self.models.len() as f32,
            alternatives: Vec::new(),
        }
    }
}

// Model implementations

#[derive(Debug)]
struct FrequencyModel {
    frequencies: HashMap<ModeType, usize>,
}

impl FrequencyModel {
    fn new() -> Self {
        Self {
            frequencies: HashMap::new(),
        }
    }
}

impl PredictionModel for FrequencyModel {
    fn predict(&self, _input: &PredictionInput) -> PredictionOutput {
        let (mode, _) = self
            .frequencies
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(m, c)| (m.clone(), *c))
            .unwrap_or((ModeType::Hybrid, 0));

        PredictionOutput {
            prediction: mode,
            confidence: 0.6,
            alternatives: Vec::new(),
        }
    }

    fn update(&mut self, feedback: &PredictionFeedback) {
        if feedback.success {
            *self.frequencies.entry(feedback.actual.clone()).or_insert(0) += 1;
        }
    }
}

#[derive(Debug)]
struct TransitionModel;

impl TransitionModel {
    fn new() -> Self {
        Self
    }
}

impl PredictionModel for TransitionModel {
    fn predict(&self, input: &PredictionInput) -> PredictionOutput {
        // Simple transition logic
        let next_mode = match input.current_mode {
            ModeType::Planning => ModeType::Execution,
            ModeType::Execution => ModeType::Analysis,
            ModeType::Analysis => ModeType::Planning,
            _ => ModeType::Hybrid,
        };

        PredictionOutput {
            prediction: next_mode,
            confidence: 0.5,
            alternatives: Vec::new(),
        }
    }

    fn update(&mut self, _feedback: &PredictionFeedback) {
        // Update transition probabilities
    }
}

#[derive(Debug)]
struct ContextModel;

impl ContextModel {
    fn new() -> Self {
        Self
    }
}

impl PredictionModel for ContextModel {
    fn predict(&self, input: &PredictionInput) -> PredictionOutput {
        // Context-based prediction
        let mode = match input.context.project_type {
            crate::planning::types::ProjectType::Infrastructure => ModeType::Planning,
            crate::planning::types::ProjectType::Library => ModeType::Execution,
            _ => ModeType::Hybrid,
        };

        PredictionOutput {
            prediction: mode,
            confidence: 0.7,
            alternatives: Vec::new(),
        }
    }

    fn update(&mut self, _feedback: &PredictionFeedback) {
        // Update context mappings
    }
}

impl LearningData {
    fn new() -> Self {
        Self {
            total_detections: 0,
            total_switches: 0,
            detection_history: Vec::new(),
            switch_history: Vec::new(),
            pattern_cache: HashMap::new(),
        }
    }
}

impl PatternHistory {
    fn new() -> Self {
        Self {
            patterns: Vec::new(),
            max_patterns: 1000,
            retention_days: 30,
        }
    }
}

impl Default for UserPreference {
    fn default() -> Self {
        Self {
            base_preferences: BasePreferences::default(),
            learning_enabled: true,
            adaptation_rate: 0.3,
            confidence_weights: ConfidenceWeights {
                historical: 0.3,
                contextual: 0.3,
                temporal: 0.2,
                success_rate: 0.2,
            },
            mode_biases: HashMap::new(),
        }
    }
}

impl Default for BasePreferences {
    fn default() -> Self {
        Self {
            preferred_mode: ModeType::Hybrid,
            detail_level: crate::planning::types::DetailLevel::Medium,
            preference_strength: 0.5,
            risk_tolerance: crate::planning::types::RiskTolerance::Balanced,
            automation_level: crate::planning::types::AutomationLevel::Guided,
            collaboration_style: crate::planning::types::CollaborationStyle::Solo,
        }
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_preference_manager_creation() {
        // Test preference manager initialization
    }

    #[tokio::test]
    async fn test_pattern_learning() {
        // Test pattern learning from detections
    }

    #[tokio::test]
    async fn test_preference_prediction() {
        // Test mode prediction
    }

    #[tokio::test]
    async fn test_behavior_analysis() {
        // Test behavior analysis
    }
}
