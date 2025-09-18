//! Enhanced Mode Detection with Consensus Intelligence
//!
//! Uses AI consensus to intelligently detect the most appropriate mode
//! based on query analysis and context understanding.

use crate::consensus::ConsensusEngine;
use crate::core::error::{HiveError, HiveResult};
use crate::planning::{ModeType, PlanningContext};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Enhanced mode detector using consensus engine
pub struct EnhancedModeDetector {
    consensus_engine: Arc<ConsensusEngine>,
    pattern_matcher: PatternMatcher,
    complexity_analyzer: ComplexityAnalyzer,
    context_analyzer: ContextAnalyzer,
    confidence_calculator: ConfidenceCalculator,
}

/// Pattern matching for mode detection
#[derive(Debug)]
struct PatternMatcher {
    patterns: HashMap<ModeType, Vec<Regex>>,
    weights: HashMap<String, f32>,
}

/// Complexity analysis for queries
#[derive(Debug)]
struct ComplexityAnalyzer {
    metrics: ComplexityMetrics,
    thresholds: ComplexityThresholds,
}

/// Context analysis for mode selection
#[derive(Debug)]
struct ContextAnalyzer {
    context_weights: HashMap<String, f32>,
    mode_affinities: HashMap<(String, ModeType), f32>,
}

/// Confidence calculation for detection results
#[derive(Debug)]
struct ConfidenceCalculator {
    base_confidence: f32,
    consensus_weight: f32,
    pattern_weight: f32,
    context_weight: f32,
}

/// Complexity metrics for a query
#[derive(Debug, Clone)]
struct ComplexityMetrics {
    word_count: usize,
    sentence_count: usize,
    technical_term_count: usize,
    dependency_count: usize,
    ambiguity_score: f32,
}

/// Thresholds for complexity levels
#[derive(Debug)]
struct ComplexityThresholds {
    low: f32,
    medium: f32,
    high: f32,
}

/// Enhanced detection result with AI insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub primary_mode: ModeType,
    pub confidence: f32,
    pub scores: HashMap<ModeType, f32>,
    pub reasoning: Vec<String>,
    pub alternatives: Vec<(ModeType, f32)>,
    pub consensus_insights: ConsensusInsights,
    pub preference_influence: f32,
}

/// Insights from consensus engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusInsights {
    pub task_complexity: String,
    pub recommended_approach: String,
    pub potential_challenges: Vec<String>,
    pub success_factors: Vec<String>,
}

impl EnhancedModeDetector {
    /// Create a new enhanced mode detector
    pub async fn new(consensus_engine: Arc<ConsensusEngine>) -> HiveResult<Self> {
        Ok(Self {
            consensus_engine,
            pattern_matcher: PatternMatcher::new(),
            complexity_analyzer: ComplexityAnalyzer::new(),
            context_analyzer: ContextAnalyzer::new(),
            confidence_calculator: ConfidenceCalculator::new(),
        })
    }

    /// Detect mode using consensus intelligence
    pub async fn detect_with_consensus(
        &self,
        query: &str,
        context: &PlanningContext,
    ) -> HiveResult<DetectionResult> {
        let mut scores = HashMap::new();
        let mut reasoning = Vec::new();

        // Initialize scores for all modes
        for mode in &[
            ModeType::Planning,
            ModeType::Execution,
            ModeType::Hybrid,
            ModeType::Analysis,
            ModeType::Learning,
        ] {
            scores.insert(mode.clone(), 0.0);
        }

        // Pattern-based detection
        let pattern_scores = self.pattern_matcher.analyze(query)?;
        self.merge_scores(
            &mut scores,
            &pattern_scores,
            self.confidence_calculator.pattern_weight,
        );
        reasoning.push(format!(
            "Pattern analysis: {:?}",
            self.get_top_pattern(query)
        ));

        // Complexity analysis
        let complexity = self.complexity_analyzer.analyze(query);
        let complexity_scores = self.complexity_to_mode_scores(&complexity);
        self.merge_scores(&mut scores, &complexity_scores, 0.3);
        reasoning.push(format!(
            "Complexity level: {}",
            self.complexity_level(&complexity)
        ));

        // Context analysis
        let context_scores = self.context_analyzer.analyze(context);
        self.merge_scores(
            &mut scores,
            &context_scores,
            self.confidence_calculator.context_weight,
        );
        reasoning.push(format!(
            "Context favors: {:?}",
            self.get_context_preference(context)
        ));

        // AI consensus analysis
        let consensus_insights = self.get_consensus_insights(query, context).await?;
        let consensus_scores = self.insights_to_mode_scores(&consensus_insights);
        self.merge_scores(
            &mut scores,
            &consensus_scores,
            self.confidence_calculator.consensus_weight,
        );
        reasoning.push(format!(
            "AI consensus: {}",
            consensus_insights.recommended_approach
        ));

        // Calculate final mode and confidence
        let (primary_mode, base_confidence) = self.get_primary_mode(&scores);
        let final_confidence =
            self.confidence_calculator
                .calculate(base_confidence, &scores, &consensus_insights);

        // Get alternatives
        let alternatives = self.get_alternatives(&scores, &primary_mode);

        Ok(DetectionResult {
            primary_mode,
            confidence: final_confidence,
            scores,
            reasoning,
            alternatives,
            consensus_insights,
            preference_influence: context.user_preferences.preference_strength,
        })
    }

    /// Get consensus insights for the query
    async fn get_consensus_insights(
        &self,
        query: &str,
        context: &PlanningContext,
    ) -> HiveResult<ConsensusInsights> {
        let prompt = format!(
            r#"Analyze this development query and provide insights:

Query: "{}"

Context:
- Project Type: {:?}
- Team Size: {}
- Experience Level: {:?}
- Existing Codebase: {}

Provide a JSON response with:
1. task_complexity: "low", "medium", or "high"
2. recommended_approach: Brief description of best approach
3. potential_challenges: Array of 2-3 potential challenges
4. success_factors: Array of 2-3 key success factors

Focus on whether this needs careful planning, immediate execution, or a hybrid approach."#,
            query,
            context.project_type,
            context.team_size,
            context.experience_level,
            context.existing_codebase
        );

        let result = self.consensus_engine.process(&prompt, None).await?;

        // Parse the response
        match serde_json::from_str::<ConsensusInsights>(&result.result.unwrap_or_default()) {
            Ok(insights) => Ok(insights),
            Err(_) => {
                // Fallback to basic insights if parsing fails
                Ok(ConsensusInsights {
                    task_complexity: self.estimate_complexity(query),
                    recommended_approach: "Balanced approach recommended".to_string(),
                    potential_challenges: vec!["Complexity management".to_string()],
                    success_factors: vec!["Clear requirements".to_string()],
                })
            }
        }
    }

    /// Convert insights to mode scores
    fn insights_to_mode_scores(&self, insights: &ConsensusInsights) -> HashMap<ModeType, f32> {
        let mut scores = HashMap::new();

        // Task complexity influences mode
        match insights.task_complexity.as_str() {
            "high" => {
                scores.insert(ModeType::Planning, 0.8);
                scores.insert(ModeType::Hybrid, 0.6);
                scores.insert(ModeType::Execution, 0.2);
            }
            "medium" => {
                scores.insert(ModeType::Hybrid, 0.8);
                scores.insert(ModeType::Planning, 0.5);
                scores.insert(ModeType::Execution, 0.5);
            }
            "low" => {
                scores.insert(ModeType::Execution, 0.8);
                scores.insert(ModeType::Hybrid, 0.4);
                scores.insert(ModeType::Planning, 0.2);
            }
            _ => {
                scores.insert(ModeType::Hybrid, 0.6);
            }
        }

        // Recommended approach keywords
        let approach_lower = insights.recommended_approach.to_lowercase();
        if approach_lower.contains("plan") || approach_lower.contains("design") {
            *scores.entry(ModeType::Planning).or_insert(0.0) += 0.3;
        }
        if approach_lower.contains("implement") || approach_lower.contains("execute") {
            *scores.entry(ModeType::Execution).or_insert(0.0) += 0.3;
        }
        if approach_lower.contains("analyze") || approach_lower.contains("understand") {
            *scores.entry(ModeType::Analysis).or_insert(0.0) += 0.3;
        }
        if approach_lower.contains("balance") || approach_lower.contains("hybrid") {
            *scores.entry(ModeType::Hybrid).or_insert(0.0) += 0.3;
        }

        scores
    }

    /// Merge scores with weights
    fn merge_scores(
        &self,
        target: &mut HashMap<ModeType, f32>,
        source: &HashMap<ModeType, f32>,
        weight: f32,
    ) {
        for (mode, score) in source {
            let current = target.entry(mode.clone()).or_insert(0.0);
            *current += score * weight;
        }
    }

    /// Get primary mode from scores
    fn get_primary_mode(&self, scores: &HashMap<ModeType, f32>) -> (ModeType, f32) {
        scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(mode, score)| (mode.clone(), *score))
            .unwrap_or((ModeType::Hybrid, 0.5))
    }

    /// Get alternative modes
    fn get_alternatives(
        &self,
        scores: &HashMap<ModeType, f32>,
        primary: &ModeType,
    ) -> Vec<(ModeType, f32)> {
        let mut alternatives: Vec<(ModeType, f32)> = scores
            .iter()
            .filter(|(mode, _)| *mode != primary)
            .map(|(mode, score)| (mode.clone(), *score))
            .collect();

        alternatives.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        alternatives.truncate(3);

        alternatives
    }

    /// Get top matching pattern
    fn get_top_pattern(&self, query: &str) -> String {
        self.pattern_matcher
            .get_top_pattern(query)
            .unwrap_or_else(|| "No specific pattern".to_string())
    }

    /// Get complexity level as string
    fn complexity_level(&self, metrics: &ComplexityMetrics) -> String {
        let score = self.complexity_analyzer.calculate_score(metrics);
        if score < self.complexity_analyzer.thresholds.low {
            "Low".to_string()
        } else if score < self.complexity_analyzer.thresholds.medium {
            "Medium".to_string()
        } else {
            "High".to_string()
        }
    }

    /// Convert complexity to mode scores
    fn complexity_to_mode_scores(&self, metrics: &ComplexityMetrics) -> HashMap<ModeType, f32> {
        let mut scores = HashMap::new();
        let complexity_score = self.complexity_analyzer.calculate_score(metrics);

        if complexity_score < self.complexity_analyzer.thresholds.low {
            scores.insert(ModeType::Execution, 0.8);
            scores.insert(ModeType::Hybrid, 0.3);
        } else if complexity_score < self.complexity_analyzer.thresholds.medium {
            scores.insert(ModeType::Hybrid, 0.8);
            scores.insert(ModeType::Planning, 0.4);
            scores.insert(ModeType::Execution, 0.4);
        } else {
            scores.insert(ModeType::Planning, 0.8);
            scores.insert(ModeType::Hybrid, 0.5);
        }

        scores
    }

    /// Get context preference
    fn get_context_preference(&self, context: &PlanningContext) -> ModeType {
        context.user_preferences.preferred_mode.clone()
    }

    /// Estimate complexity for fallback
    fn estimate_complexity(&self, query: &str) -> String {
        let words = query.split_whitespace().count();
        if words < 10 {
            "low".to_string()
        } else if words < 30 {
            "medium".to_string()
        } else {
            "high".to_string()
        }
    }
}

impl PatternMatcher {
    fn new() -> Self {
        let mut patterns = HashMap::new();

        // Planning patterns
        patterns.insert(
            ModeType::Planning,
            vec![
                Regex::new(r"\b(plan|design|architect|strategy|roadmap)\b").unwrap(),
                Regex::new(r"\b(how should I|best approach|structure)\b").unwrap(),
                Regex::new(r"\b(break down|decompose|organize|outline)\b").unwrap(),
                Regex::new(r"\b(timeline|milestone|phase|stage)\b").unwrap(),
            ],
        );

        // Execution patterns
        patterns.insert(
            ModeType::Execution,
            vec![
                Regex::new(r"\b(implement|code|build|create|write)\b").unwrap(),
                Regex::new(r"\b(fix|debug|solve|patch|update)\b").unwrap(),
                Regex::new(r"\b(add|remove|modify|change|refactor)\b").unwrap(),
                Regex::new(r"\b(quickly|immediately|now|asap)\b").unwrap(),
            ],
        );

        // Hybrid patterns
        patterns.insert(
            ModeType::Hybrid,
            vec![
                Regex::new(r"\b(plan and implement|design and build)\b").unwrap(),
                Regex::new(r"\b(comprehensive|complete|full|entire)\b").unwrap(),
                Regex::new(r"\b(step by step|iterative|incremental)\b").unwrap(),
                Regex::new(r"\b(both|and also|as well as)\b").unwrap(),
            ],
        );

        // Analysis patterns
        patterns.insert(
            ModeType::Analysis,
            vec![
                Regex::new(r"\b(analyze|examine|investigate|understand)\b").unwrap(),
                Regex::new(r"\b(what|why|how does|explain)\b").unwrap(),
                Regex::new(r"\b(review|audit|assess|evaluate)\b").unwrap(),
                Regex::new(r"\b(performance|bottleneck|issue|problem)\b").unwrap(),
            ],
        );

        Self {
            patterns,
            weights: HashMap::new(),
        }
    }

    fn analyze(&self, query: &str) -> HiveResult<HashMap<ModeType, f32>> {
        let mut scores = HashMap::new();
        let query_lower = query.to_lowercase();

        for (mode, patterns) in &self.patterns {
            let matches = patterns.iter().filter(|p| p.is_match(&query_lower)).count();

            if matches > 0 {
                let score = (matches as f32 * 0.25).min(1.0);
                scores.insert(mode.clone(), score);
            }
        }

        Ok(scores)
    }

    fn get_top_pattern(&self, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();

        for (mode, patterns) in &self.patterns {
            for pattern in patterns {
                if pattern.is_match(&query_lower) {
                    return Some(format!("{:?} pattern", mode));
                }
            }
        }

        None
    }
}

impl ComplexityAnalyzer {
    fn new() -> Self {
        Self {
            metrics: ComplexityMetrics {
                word_count: 0,
                sentence_count: 0,
                technical_term_count: 0,
                dependency_count: 0,
                ambiguity_score: 0.0,
            },
            thresholds: ComplexityThresholds {
                low: 0.3,
                medium: 0.6,
                high: 0.8,
            },
        }
    }

    fn analyze(&self, query: &str) -> ComplexityMetrics {
        let words: Vec<&str> = query.split_whitespace().collect();
        let word_count = words.len();
        let sentence_count = query.matches('.').count() + 1;

        let technical_terms = [
            "api",
            "database",
            "algorithm",
            "architecture",
            "framework",
            "microservice",
            "deployment",
            "integration",
            "authentication",
            "optimization",
            "refactor",
            "scalability",
            "performance",
        ];

        let technical_term_count = words
            .iter()
            .filter(|w| technical_terms.contains(&w.to_lowercase().as_str()))
            .count();

        let dependency_words = ["before", "after", "depends", "requires", "then"];
        let dependency_count = words
            .iter()
            .filter(|w| dependency_words.contains(&w.to_lowercase().as_str()))
            .count();

        let ambiguity_score = if query.contains('?') { 0.2 } else { 0.0 }
            + if query.contains("maybe") || query.contains("possibly") {
                0.3
            } else {
                0.0
            }
            + if query.contains("or") { 0.1 } else { 0.0 };

        ComplexityMetrics {
            word_count,
            sentence_count,
            technical_term_count,
            dependency_count,
            ambiguity_score,
        }
    }

    fn calculate_score(&self, metrics: &ComplexityMetrics) -> f32 {
        let word_score = (metrics.word_count as f32 / 50.0).min(1.0);
        let sentence_score = (metrics.sentence_count as f32 / 5.0).min(1.0);
        let technical_score = (metrics.technical_term_count as f32 / 5.0).min(1.0);
        let dependency_score = (metrics.dependency_count as f32 / 3.0).min(1.0);

        (word_score * 0.2
            + sentence_score * 0.2
            + technical_score * 0.3
            + dependency_score * 0.2
            + metrics.ambiguity_score * 0.1)
            .min(1.0)
    }
}

impl ContextAnalyzer {
    fn new() -> Self {
        let mut mode_affinities = HashMap::new();

        // Project type affinities
        mode_affinities.insert(("Infrastructure".to_string(), ModeType::Planning), 0.8);
        mode_affinities.insert(("Library".to_string(), ModeType::Execution), 0.7);
        mode_affinities.insert(("WebApplication".to_string(), ModeType::Hybrid), 0.8);

        // Experience level affinities
        mode_affinities.insert(("Beginner".to_string(), ModeType::Planning), 0.7);
        mode_affinities.insert(("Expert".to_string(), ModeType::Execution), 0.6);

        Self {
            context_weights: HashMap::new(),
            mode_affinities,
        }
    }

    fn analyze(&self, context: &PlanningContext) -> HashMap<ModeType, f32> {
        let mut scores = HashMap::new();

        // User preference has highest weight
        scores.insert(
            context.user_preferences.preferred_mode.clone(),
            context.user_preferences.preference_strength,
        );

        // Add project type affinity
        let project_key = format!("{:?}", context.project_type);
        for mode in &[
            ModeType::Planning,
            ModeType::Execution,
            ModeType::Hybrid,
            ModeType::Analysis,
        ] {
            if let Some(affinity) = self
                .mode_affinities
                .get(&(project_key.clone(), mode.clone()))
            {
                let current = scores.entry(mode.clone()).or_insert(0.0);
                *current += affinity * 0.3;
            }
        }

        // Team size influences mode
        if context.team_size > 3 {
            *scores.entry(ModeType::Planning).or_insert(0.0) += 0.2;
        } else if context.team_size == 1 {
            *scores.entry(ModeType::Execution).or_insert(0.0) += 0.2;
        }

        scores
    }
}

impl ConfidenceCalculator {
    fn new() -> Self {
        Self {
            base_confidence: 0.5,
            consensus_weight: 0.4,
            pattern_weight: 0.3,
            context_weight: 0.3,
        }
    }

    fn calculate(
        &self,
        base_score: f32,
        scores: &HashMap<ModeType, f32>,
        insights: &ConsensusInsights,
    ) -> f32 {
        let mut confidence = base_score;

        // Adjust based on score distribution
        let sorted_scores: Vec<f32> = {
            let mut vals: Vec<f32> = scores.values().copied().collect();
            vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            vals
        };

        if sorted_scores.len() >= 2 {
            let gap = sorted_scores[0] - sorted_scores[1];
            confidence += gap * 0.3; // Clear winner increases confidence
        }

        // Consensus quality affects confidence
        if !insights.potential_challenges.is_empty() {
            confidence *= 0.9; // Challenges reduce confidence slightly
        }

        if insights.success_factors.len() >= 2 {
            confidence *= 1.1; // Clear success factors increase confidence
        }

        confidence.clamp(0.1, 0.95)
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enhanced_detector_creation() {
        // Test detector initialization
    }

    #[tokio::test]
    async fn test_consensus_detection() {
        // Test detection with consensus
    }

    #[tokio::test]
    async fn test_pattern_matching() {
        // Test pattern-based detection
    }

    #[tokio::test]
    async fn test_complexity_analysis() {
        // Test complexity calculation
    }
}
