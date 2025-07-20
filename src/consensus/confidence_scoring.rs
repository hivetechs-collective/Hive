// AI-Powered Operation Confidence Scoring Algorithms
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use chrono::Timelike;

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::operation_analysis::{
    ComponentScores, ScoringFactors, OperationContext
};
use crate::consensus::operation_clustering::{ClusterType, OperationCluster};
use crate::consensus::operation_history::OperationHistoryDatabase;

/// Advanced scoring algorithm that combines multiple factors
#[derive(Debug, Clone)]
pub struct ConfidenceScoringEngine {
    /// Weights for different scoring components
    component_weights: ScoringWeights,
    /// Historical performance data
    history_db: Option<Arc<OperationHistoryDatabase>>,
    /// Cache for scoring results
    score_cache: Arc<RwLock<HashMap<String, ScoringResult>>>,
    /// Machine learning model for advanced predictions
    ml_model: Option<Arc<OperationPredictionModel>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    /// Weight for historical success rate (0.0-1.0)
    pub historical: f32,
    /// Weight for pattern safety (0.0-1.0)
    pub pattern: f32,
    /// Weight for context relevance (0.0-1.0)
    pub context: f32,
    /// Weight for quality impact (0.0-1.0)
    pub quality: f32,
    /// Weight for execution feasibility (0.0-1.0)
    pub feasibility: f32,
    /// Weight for user trust factor (0.0-1.0)
    pub user_trust: f32,
    /// Weight for complexity penalty (0.0-1.0)
    pub complexity: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            historical: 0.25,
            pattern: 0.20,
            context: 0.15,
            quality: 0.15,
            feasibility: 0.10,
            user_trust: 0.10,
            complexity: 0.05,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringResult {
    /// Overall confidence score (0-100)
    pub confidence: f32,
    /// Overall risk score (0-100)
    pub risk: f32,
    /// Detailed breakdown of scoring
    pub breakdown: ScoreBreakdown,
    /// Confidence intervals
    pub confidence_interval: ConfidenceInterval,
    /// Factors that most influenced the score
    pub primary_factors: Vec<PrimaryFactor>,
    /// Recommendations for improving confidence
    pub improvement_suggestions: Vec<ImprovementSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub historical_component: f32,
    pub pattern_component: f32,
    pub context_component: f32,
    pub quality_component: f32,
    pub feasibility_component: f32,
    pub user_trust_component: f32,
    pub complexity_penalty: f32,
    pub adjustment_factors: Vec<AdjustmentFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub lower_bound: f32,
    pub upper_bound: f32,
    pub variance: f32,
    pub reliability: f32, // How reliable is this interval
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryFactor {
    pub name: String,
    pub impact: f32, // -100 to +100
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementSuggestion {
    pub action: String,
    pub expected_improvement: f32,
    pub difficulty: SuggestionDifficulty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionDifficulty {
    Easy,    // Can be done automatically
    Medium,  // Requires user action
    Hard,    // Requires significant changes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentFactor {
    pub name: String,
    pub adjustment: f32, // Positive or negative adjustment
    pub reason: String,
}

/// Machine learning model for operation predictions
#[derive(Debug)]
pub struct OperationPredictionModel {
    // In a real implementation, this would contain:
    // - Neural network or other ML model
    // - Feature extractors
    // - Model weights
    // For now, we'll use heuristics
    feature_importance: HashMap<String, f32>,
}

impl ConfidenceScoringEngine {
    pub fn new(
        weights: Option<ScoringWeights>,
        history_db: Option<Arc<OperationHistoryDatabase>>,
    ) -> Self {
        let weights = weights.unwrap_or_default();
        
        // Normalize weights to sum to 1.0
        let total = weights.historical + weights.pattern + weights.context + 
                   weights.quality + weights.feasibility + weights.user_trust + weights.complexity;
        
        let normalized_weights = if total > 0.0 {
            ScoringWeights {
                historical: weights.historical / total,
                pattern: weights.pattern / total,
                context: weights.context / total,
                quality: weights.quality / total,
                feasibility: weights.feasibility / total,
                user_trust: weights.user_trust / total,
                complexity: weights.complexity / total,
            }
        } else {
            ScoringWeights::default()
        };

        Self {
            component_weights: normalized_weights,
            history_db,
            score_cache: Arc::new(RwLock::new(HashMap::new())),
            ml_model: Some(Arc::new(OperationPredictionModel::new())),
        }
    }

    pub async fn calculate_advanced_score(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
        component_scores: &ComponentScores,
        scoring_factors: &ScoringFactors,
        cluster: Option<&OperationCluster>,
    ) -> Result<ScoringResult> {
        // Check cache
        let cache_key = self.generate_cache_key(operations, context);
        if let Some(cached) = self.score_cache.read().await.get(&cache_key) {
            debug!("Using cached scoring result");
            return Ok(cached.clone());
        }

        // Calculate base components
        let historical_score = self.calculate_historical_component(scoring_factors, component_scores).await?;
        let pattern_score = self.calculate_pattern_component(scoring_factors, component_scores)?;
        let context_score = self.calculate_context_component(component_scores)?;
        let quality_score = self.calculate_quality_component(component_scores)?;
        let feasibility_score = self.calculate_feasibility_component(component_scores)?;
        let user_trust_score = scoring_factors.user_trust * 100.0;
        let complexity_penalty = self.calculate_complexity_penalty(operations, cluster)?;

        // Apply weights
        let weighted_confidence = 
            historical_score * self.component_weights.historical +
            pattern_score * self.component_weights.pattern +
            context_score * self.component_weights.context +
            quality_score * self.component_weights.quality +
            feasibility_score * self.component_weights.feasibility +
            user_trust_score * self.component_weights.user_trust -
            complexity_penalty * self.component_weights.complexity;

        // Calculate adjustment factors
        let adjustment_factors = self.calculate_adjustment_factors(operations, context, cluster).await?;
        let total_adjustment: f32 = adjustment_factors.iter().map(|a| a.adjustment).sum();
        
        // Apply adjustments
        let adjusted_confidence = (weighted_confidence + total_adjustment).clamp(0.0, 100.0);

        // Calculate risk (inverse relationship with confidence, but with nuances)
        let risk = self.calculate_nuanced_risk(
            adjusted_confidence,
            operations,
            scoring_factors,
            component_scores,
        )?;

        // Calculate confidence interval
        let confidence_interval = self.calculate_confidence_interval(
            adjusted_confidence,
            &adjustment_factors,
            scoring_factors,
        )?;

        // Identify primary factors
        let primary_factors = self.identify_primary_factors(
            &ScoreBreakdown {
                historical_component: historical_score,
                pattern_component: pattern_score,
                context_component: context_score,
                quality_component: quality_score,
                feasibility_component: feasibility_score,
                user_trust_component: user_trust_score,
                complexity_penalty,
                adjustment_factors: adjustment_factors.clone(),
            },
            adjusted_confidence,
        )?;

        // Generate improvement suggestions
        let improvement_suggestions = self.generate_improvement_suggestions(
            adjusted_confidence,
            &primary_factors,
            scoring_factors,
        )?;

        let result = ScoringResult {
            confidence: adjusted_confidence,
            risk,
            breakdown: ScoreBreakdown {
                historical_component: historical_score,
                pattern_component: pattern_score,
                context_component: context_score,
                quality_component: quality_score,
                feasibility_component: feasibility_score,
                user_trust_component: user_trust_score,
                complexity_penalty,
                adjustment_factors,
            },
            confidence_interval,
            primary_factors,
            improvement_suggestions,
        };

        // Cache the result
        self.score_cache.write().await.insert(cache_key, result.clone());

        Ok(result)
    }

    async fn calculate_historical_component(
        &self,
        factors: &ScoringFactors,
        scores: &ComponentScores,
    ) -> Result<f32> {
        let base_score = if let Some(indexer_score) = &scores.knowledge_indexer {
            indexer_score.prediction_confidence
        } else {
            50.0 // Neutral if no data
        };

        // Adjust based on success rate
        let success_adjustment = if let Some(success_rate) = factors.historical_success {
            (success_rate - 0.5) * 40.0 // ±20 points based on success rate
        } else {
            0.0
        };

        // Adjust based on sample size
        let sample_adjustment = if let Some(count) = factors.similar_operations_count {
            match count {
                0 => -20.0,      // No data is concerning
                1..=5 => -10.0,  // Limited data
                6..=20 => 0.0,   // Decent sample
                21..=50 => 5.0,  // Good sample
                _ => 10.0,       // Excellent sample
            }
        } else {
            -10.0
        };

        Ok((base_score + success_adjustment + sample_adjustment).clamp(0.0, 100.0))
    }

    fn calculate_pattern_component(&self, factors: &ScoringFactors, scores: &ComponentScores) -> Result<f32> {
        let base_score = if let Some(pattern_score) = &scores.pattern_recognizer {
            pattern_score.safety_score
        } else {
            75.0 // Assume moderately safe if no data
        };

        // Penalize dangerous patterns heavily
        let danger_penalty = factors.dangerous_pattern_count.unwrap_or(0) as f32 * 15.0;
        let anti_pattern_penalty = factors.anti_pattern_count.unwrap_or(0) as f32 * 10.0;

        Ok((base_score - danger_penalty - anti_pattern_penalty).clamp(0.0, 100.0))
    }

    fn calculate_context_component(&self, scores: &ComponentScores) -> Result<f32> {
        if let Some(context_score) = &scores.context_retriever {
            let relevance = context_score.relevance_score * 50.0;
            let precedent = context_score.precedent_strength * 50.0;
            Ok(relevance + precedent)
        } else {
            Ok(50.0) // Neutral if no context data
        }
    }

    fn calculate_quality_component(&self, scores: &ComponentScores) -> Result<f32> {
        if let Some(quality_score) = &scores.quality_analyzer {
            // Invert risk to get quality
            let quality_base = 100.0 - quality_score.risk_score;
            
            // Penalize conflicts
            let conflict_penalty = quality_score.conflict_probability * 0.5;
            
            Ok((quality_base * (1.0 - conflict_penalty)).clamp(0.0, 100.0))
        } else {
            Ok(70.0) // Assume moderate quality if no data
        }
    }

    fn calculate_feasibility_component(&self, scores: &ComponentScores) -> Result<f32> {
        if let Some(synthesizer_score) = &scores.knowledge_synthesizer {
            Ok(synthesizer_score.plan_quality * 100.0)
        } else {
            Ok(75.0) // Assume moderate feasibility
        }
    }

    fn calculate_complexity_penalty(&self, operations: &[FileOperation], cluster: Option<&OperationCluster>) -> Result<f32> {
        let mut penalty = 0.0f32;

        // Penalty based on operation count
        penalty += match operations.len() {
            0..=3 => 0.0,
            4..=10 => 5.0,
            11..=20 => 15.0,
            21..=50 => 25.0,
            _ => 40.0,
        };

        // Penalty based on operation diversity
        let operation_types: std::collections::HashSet<_> = operations.iter()
            .map(|op| std::mem::discriminant(op))
            .collect();
        
        penalty += match operation_types.len() {
            1 => 0.0,   // Single type is simple
            2 => 5.0,   // Two types is manageable
            3 => 10.0,  // Three types adds complexity
            _ => 20.0,  // Many types is complex
        };

        // Adjust based on cluster type
        if let Some(cluster) = cluster {
            penalty *= match cluster.cluster_type {
                ClusterType::TestCreation => 0.5,    // Tests are usually safer
                ClusterType::Documentation => 0.3,    // Docs are very safe
                ClusterType::Deletion => 2.0,         // Deletions are risky
                ClusterType::Migration => 1.8,        // Migrations are complex
                ClusterType::Refactoring => 1.5,      // Refactoring is moderately complex
                _ => 1.0,
            };
        }

        Ok(penalty.min(50.0f32)) // Cap at 50 points
    }

    async fn calculate_adjustment_factors(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
        cluster: Option<&OperationCluster>,
    ) -> Result<Vec<AdjustmentFactor>> {
        let mut factors = Vec::new();

        // Time-based adjustments
        let time_of_day = chrono::Local::now().time().hour();
        if time_of_day >= 22 || time_of_day <= 6 {
            factors.push(AdjustmentFactor {
                name: "Late night operation".to_string(),
                adjustment: -5.0,
                reason: "Higher risk of errors during off-hours".to_string(),
            });
        }

        // Repository state adjustments
        if context.git_commit.is_some() {
            factors.push(AdjustmentFactor {
                name: "Clean git state".to_string(),
                adjustment: 5.0,
                reason: "Easy rollback available via git".to_string(),
            });
        }

        // Operation-specific adjustments
        let has_deletions = operations.iter().any(|op| matches!(op, FileOperation::Delete { .. }));
        if has_deletions {
            factors.push(AdjustmentFactor {
                name: "Contains deletions".to_string(),
                adjustment: -10.0,
                reason: "Deletion operations carry inherent risk".to_string(),
            });
        }

        // ML model predictions
        if let Some(ml_model) = &self.ml_model {
            let ml_adjustment = ml_model.predict_adjustment(operations, context)?;
            if ml_adjustment.abs() > 1.0 {
                factors.push(AdjustmentFactor {
                    name: "ML model prediction".to_string(),
                    adjustment: ml_adjustment,
                    reason: "Based on learned patterns from historical data".to_string(),
                });
            }
        }

        // Cluster-based adjustments
        if let Some(cluster) = cluster {
            if cluster.dependencies.len() > 3 {
                factors.push(AdjustmentFactor {
                    name: "Complex dependencies".to_string(),
                    adjustment: -8.0,
                    reason: "Multiple dependencies increase failure risk".to_string(),
                });
            }
        }

        Ok(factors)
    }

    fn calculate_nuanced_risk(
        &self,
        confidence: f32,
        operations: &[FileOperation],
        factors: &ScoringFactors,
        scores: &ComponentScores,
    ) -> Result<f32> {
        // Start with inverse of confidence
        let mut risk = 100.0 - confidence;

        // Adjust based on operation characteristics
        let has_deletions = operations.iter().any(|op| matches!(op, FileOperation::Delete { .. }));
        if has_deletions {
            risk *= 1.3; // 30% higher risk for deletions
        }

        // Adjust based on rollback difficulty
        if let Some(quality_score) = &scores.quality_analyzer {
            if quality_score.rollback_complexity > 50.0 {
                risk *= 1.2; // 20% higher risk if rollback is complex
            }
        }

        // Adjust based on conflict probability
        if let Some(conflict_prob) = factors.conflict_probability {
            risk *= 1.0 + (conflict_prob * 0.5); // Up to 50% higher risk
        }

        // Consider pattern safety
        if let Some(pattern_safety) = factors.pattern_safety {
            if pattern_safety < 0.5 {
                risk *= 1.4; // 40% higher risk for unsafe patterns
            }
        }

        Ok(risk.clamp(0.0, 100.0))
    }

    fn calculate_confidence_interval(
        &self,
        confidence: f32,
        adjustments: &[AdjustmentFactor],
        factors: &ScoringFactors,
    ) -> Result<ConfidenceInterval> {
        // Calculate variance based on data availability
        let data_variance = match factors.similar_operations_count {
            Some(count) if count > 50 => 5.0,
            Some(count) if count > 20 => 10.0,
            Some(count) if count > 5 => 15.0,
            Some(_) => 20.0,
            None => 25.0,
        };

        // Add variance from adjustments
        let adjustment_variance: f32 = adjustments.iter()
            .map(|a| a.adjustment.abs() * 0.2)
            .sum();

        let total_variance = data_variance + adjustment_variance;
        
        let lower_bound = (confidence - total_variance).max(0.0);
        let upper_bound = (confidence + total_variance).min(100.0);

        // Calculate reliability based on data quality
        let reliability = match factors.similar_operations_count {
            Some(count) if count > 30 => 0.9,
            Some(count) if count > 10 => 0.7,
            Some(count) if count > 0 => 0.5,
            _ => 0.3,
        };

        Ok(ConfidenceInterval {
            lower_bound,
            upper_bound,
            variance: total_variance,
            reliability,
        })
    }

    fn identify_primary_factors(&self, breakdown: &ScoreBreakdown, final_score: f32) -> Result<Vec<PrimaryFactor>> {
        let mut factors = Vec::new();
        
        // Calculate impact of each component
        let components = vec![
            ("Historical precedent", breakdown.historical_component, self.component_weights.historical),
            ("Pattern safety", breakdown.pattern_component, self.component_weights.pattern),
            ("Context relevance", breakdown.context_component, self.component_weights.context),
            ("Code quality", breakdown.quality_component, self.component_weights.quality),
            ("Execution feasibility", breakdown.feasibility_component, self.component_weights.feasibility),
            ("User trust", breakdown.user_trust_component, self.component_weights.user_trust),
        ];

        let average_score = final_score;
        
        for (name, score, weight) in components {
            let weighted_score = score * weight;
            let impact = weighted_score - (average_score * weight);
            
            if impact.abs() > 5.0 {
                factors.push(PrimaryFactor {
                    name: name.to_string(),
                    impact,
                    description: if impact > 0.0 {
                        format!("Contributing {:.1} points above average", impact)
                    } else {
                        format!("Reducing score by {:.1} points", impact.abs())
                    },
                });
            }
        }

        // Add complexity penalty if significant
        if breakdown.complexity_penalty > 10.0 {
            factors.push(PrimaryFactor {
                name: "Operation complexity".to_string(),
                impact: -breakdown.complexity_penalty,
                description: format!("High complexity reduces confidence by {:.1} points", breakdown.complexity_penalty),
            });
        }

        // Sort by absolute impact
        factors.sort_by(|a, b| b.impact.abs().partial_cmp(&a.impact.abs()).unwrap());
        
        Ok(factors)
    }

    fn generate_improvement_suggestions(
        &self,
        confidence: f32,
        primary_factors: &[PrimaryFactor],
        scoring_factors: &ScoringFactors,
    ) -> Result<Vec<ImprovementSuggestion>> {
        let mut suggestions = Vec::new();

        // Suggest improvements based on primary negative factors
        for factor in primary_factors.iter().filter(|f| f.impact < 0.0) {
            match factor.name.as_str() {
                "Historical precedent" => {
                    suggestions.push(ImprovementSuggestion {
                        action: "Test operation on a small subset first".to_string(),
                        expected_improvement: 10.0,
                        difficulty: SuggestionDifficulty::Easy,
                    });
                }
                "Pattern safety" => {
                    suggestions.push(ImprovementSuggestion {
                        action: "Refactor to avoid detected anti-patterns".to_string(),
                        expected_improvement: 15.0,
                        difficulty: SuggestionDifficulty::Medium,
                    });
                }
                "Operation complexity" => {
                    suggestions.push(ImprovementSuggestion {
                        action: "Break operation into smaller, focused changes".to_string(),
                        expected_improvement: 20.0,
                        difficulty: SuggestionDifficulty::Medium,
                    });
                }
                _ => {}
            }
        }

        // General suggestions based on confidence level
        if confidence < 60.0 {
            suggestions.push(ImprovementSuggestion {
                action: "Create comprehensive backups before proceeding".to_string(),
                expected_improvement: 5.0,
                difficulty: SuggestionDifficulty::Easy,
            });
            
            suggestions.push(ImprovementSuggestion {
                action: "Review changes with a team member".to_string(),
                expected_improvement: 15.0,
                difficulty: SuggestionDifficulty::Medium,
            });
        }

        // Suggestions based on missing data
        if scoring_factors.similar_operations_count.unwrap_or(0) < 5 {
            suggestions.push(ImprovementSuggestion {
                action: "Run operation in test environment first".to_string(),
                expected_improvement: 25.0,
                difficulty: SuggestionDifficulty::Hard,
            });
        }

        Ok(suggestions)
    }

    fn generate_cache_key(&self, operations: &[FileOperation], context: &OperationContext) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        for op in operations {
            format!("{:?}", op).hash(&mut hasher);
        }
        context.repository_path.hash(&mut hasher);
        context.user_question.hash(&mut hasher);
        
        format!("score_{:x}", hasher.finish())
    }

    pub async fn update_weights(&mut self, new_weights: ScoringWeights) {
        self.component_weights = new_weights;
        self.score_cache.write().await.clear();
        info!("Scoring weights updated, cache cleared");
    }

    pub async fn get_scoring_statistics(&self) -> ScoringStatistics {
        let cache = self.score_cache.read().await;
        let scores: Vec<f32> = cache.values().map(|r| r.confidence).collect();
        
        let avg_confidence = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f32>() / scores.len() as f32
        };

        let avg_risk = if scores.is_empty() {
            0.0
        } else {
            cache.values().map(|r| r.risk).sum::<f32>() / cache.len() as f32
        };

        ScoringStatistics {
            total_scored: cache.len(),
            average_confidence: avg_confidence,
            average_risk: avg_risk,
            cache_size: cache.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringStatistics {
    pub total_scored: usize,
    pub average_confidence: f32,
    pub average_risk: f32,
    pub cache_size: usize,
}

impl OperationPredictionModel {
    fn new() -> Self {
        let mut feature_importance = HashMap::new();
        
        // Initialize feature importance (in real ML, these would be learned)
        feature_importance.insert("operation_count".to_string(), 0.15);
        feature_importance.insert("file_type_diversity".to_string(), 0.10);
        feature_importance.insert("path_depth".to_string(), 0.05);
        feature_importance.insert("has_tests".to_string(), 0.20);
        feature_importance.insert("touches_config".to_string(), 0.15);
        feature_importance.insert("modifies_core".to_string(), 0.25);
        feature_importance.insert("time_of_day".to_string(), 0.10);
        
        Self { feature_importance }
    }

    fn predict_adjustment(&self, operations: &[FileOperation], context: &OperationContext) -> Result<f32> {
        let mut features = HashMap::new();
        
        // Extract features
        features.insert("operation_count", operations.len() as f32);
        
        let file_types: std::collections::HashSet<_> = operations.iter()
            .filter_map(|op| {
                let path = match op {
                    FileOperation::Create { path, .. } |
                    FileOperation::Update { path, .. } |
                    FileOperation::Delete { path } |
                    FileOperation::Append { path, .. } => path,
                    FileOperation::Rename { to, .. } => to,
                };
                path.extension().and_then(|e| e.to_str()).map(|s| s.to_string())
            })
            .collect();
        
        features.insert("file_type_diversity", file_types.len() as f32);
        
        // Check for test files
        let has_tests = operations.iter().any(|op| {
            let path_str = match op {
                FileOperation::Create { path, .. } |
                FileOperation::Update { path, .. } |
                FileOperation::Delete { path } |
                FileOperation::Append { path, .. } => path.to_string_lossy(),
                FileOperation::Rename { to, .. } => to.to_string_lossy(),
            };
            path_str.contains("test") || path_str.contains("spec")
        });
        
        features.insert("has_tests", if has_tests { 1.0 } else { 0.0 });
        
        // Simple linear model (in reality, this would be a trained neural network)
        let mut adjustment = 0.0;
        
        for (feature_name, importance) in &self.feature_importance {
            if let Some(&value) = features.get(feature_name.as_str()) {
                // Normalize and apply importance
                let normalized_value = match feature_name.as_str() {
                    "operation_count" => -(value - 5.0) / 10.0, // Penalty for many ops
                    "file_type_diversity" => -(value - 2.0) / 5.0, // Penalty for diversity
                    "has_tests" => value * 2.0, // Bonus for tests
                    _ => 0.0,
                };
                
                adjustment += normalized_value * importance * 10.0; // Scale to meaningful range
            }
        }
        
        Ok(adjustment.clamp(-20.0, 20.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_confidence_scoring() {
        let engine = ConfidenceScoringEngine::new(None, None);
        
        let operations = vec![
            FileOperation::Create {
                path: PathBuf::from("test.rs"),
                content: "fn test() {}".to_string(),
            }
        ];
        
        let context = OperationContext {
            repository_path: PathBuf::from("/test"),
            user_question: "Create test".to_string(),
            consensus_response: "Creating test file".to_string(),
            timestamp: chrono::Utc::now(),
            session_id: "test-session".to_string(),
        };
        
        let component_scores = ComponentScores {
            knowledge_indexer: Some(crate::ai_helpers::scores::KnowledgeIndexerScore {
                similarity_score: 0.8,
                prediction_confidence: 85.0,
                relevant_context_found: true,
            }),
            context_retriever: Some(crate::ai_helpers::scores::ContextRetrieverScore {
                relevance_score: 0.9,
                precedent_strength: 0.7,
                success_rate: Some(0.85),
            }),
            pattern_recognizer: Some(crate::ai_helpers::scores::PatternRecognizerScore {
                safety_score: 90.0,
                pattern_matches: vec![],
                anti_patterns_detected: 0,
            }),
            quality_analyzer: Some(crate::ai_helpers::scores::QualityAnalyzerScore {
                risk_score: 15.0,
                quality_impact: 0.1,
                conflict_probability: 0.05,
                rollback_complexity: 10.0,
            }),
            knowledge_synthesizer: Some(crate::ai_helpers::scores::KnowledgeSynthesizerScore {
                plan_quality: 0.9,
                completeness: 0.95,
                execution_confidence: 0.88,
            }),
        };
        
        let scoring_factors = ScoringFactors {
            historical_success: Some(0.85),
            pattern_safety: Some(0.9),
            conflict_probability: Some(0.05),
            rollback_complexity: Some(0.1),
            user_trust: 0.8,
        };
        
        let result = engine.calculate_advanced_score(
            &operations,
            &context,
            &component_scores,
            &scoring_factors,
            None,
        ).await.unwrap();
        
        assert!(result.confidence > 70.0);
        assert!(result.risk < 30.0);
        assert!(!result.primary_factors.is_empty());
    }

    #[test]
    fn test_scoring_weights_normalization() {
        let weights = ScoringWeights {
            historical: 5.0,
            pattern: 4.0,
            context: 3.0,
            quality: 3.0,
            feasibility: 2.0,
            user_trust: 2.0,
            complexity: 1.0,
        };
        
        let engine = ConfidenceScoringEngine::new(Some(weights), None);
        
        // Check that weights sum to 1.0
        let total = engine.component_weights.historical +
                   engine.component_weights.pattern +
                   engine.component_weights.context +
                   engine.component_weights.quality +
                   engine.component_weights.feasibility +
                   engine.component_weights.user_trust +
                   engine.component_weights.complexity;
        
        assert!((total - 1.0).abs() < 0.001);
    }
}