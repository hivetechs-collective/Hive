//! Mode Detection Algorithm
//! 
//! Determines the appropriate operating mode based on query analysis

use crate::core::error::{HiveResult, HiveError};
use crate::planning::types::*;
use std::collections::HashMap;
use regex::Regex;

/// Mode detection engine
pub struct ModeDetector {
    patterns: ModePatterns,
    weights: ModeWeights,
}

/// Pattern matching for mode detection
#[derive(Debug)]
struct ModePatterns {
    planning_patterns: Vec<Regex>,
    execution_patterns: Vec<Regex>,
    analysis_patterns: Vec<Regex>,
    hybrid_indicators: Vec<Regex>,
}

/// Weights for different detection criteria
#[derive(Debug)]
struct ModeWeights {
    keyword_weight: f32,
    context_weight: f32,
    complexity_weight: f32,
    user_preference_weight: f32,
}

/// Detection result with confidence scores
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub primary_mode: ModeType,
    pub confidence: f32,
    pub scores: HashMap<ModeType, f32>,
    pub reasoning: Vec<String>,
}

impl ModeDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::init_patterns(),
            weights: Self::init_weights(),
        }
    }

    /// Detect the most appropriate mode for a given query
    pub fn detect_mode(&self, query: &str, context: &PlanningContext) -> HiveResult<ModeType> {
        let result = self.analyze_query(query, context)?;
        Ok(result.primary_mode)
    }

    /// Get confidence score for a detected mode
    pub fn get_confidence(&self, query: &str, mode: &ModeType) -> HiveResult<f32> {
        let result = self.analyze_query(query, &PlanningContext::default())?;
        Ok(result.scores.get(mode).copied().unwrap_or(0.0))
    }

    /// Explain why a particular mode was chosen
    pub fn explain_choice(&self, query: &str, mode: &ModeType) -> HiveResult<String> {
        let result = self.analyze_query(query, &PlanningContext::default())?;
        
        let explanation = if result.primary_mode == *mode {
            format!("Selected {} mode because: {}", 
                mode_to_string(mode), 
                result.reasoning.join("; "))
        } else {
            format!("{} mode was not selected. Primary choice was {} mode.", 
                mode_to_string(mode), 
                mode_to_string(&result.primary_mode))
        };
        
        Ok(explanation)
    }

    /// Get alternative modes with their scores
    pub fn get_alternatives(&self, query: &str, context: &PlanningContext) -> HiveResult<Vec<(ModeType, f32)>> {
        let result = self.analyze_query(query, context)?;
        
        let mut alternatives: Vec<(ModeType, f32)> = result.scores.into_iter()
            .filter(|(mode, _)| *mode != result.primary_mode)
            .collect();
        
        alternatives.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(alternatives)
    }

    /// Analyze query and return detailed detection result
    pub fn analyze_query(&self, query: &str, context: &PlanningContext) -> HiveResult<DetectionResult> {
        let mut scores = HashMap::new();
        let mut reasoning = Vec::new();
        
        // Initialize scores
        scores.insert(ModeType::Planning, 0.0);
        scores.insert(ModeType::Execution, 0.0);
        scores.insert(ModeType::Analysis, 0.0);
        scores.insert(ModeType::Hybrid, 0.0);
        
        // Analyze keywords and patterns
        self.analyze_keywords(query, &mut scores, &mut reasoning)?;
        
        // Analyze context
        self.analyze_context(context, &mut scores, &mut reasoning);
        
        // Analyze complexity
        self.analyze_complexity(query, &mut scores, &mut reasoning);
        
        // Apply user preferences
        self.apply_user_preferences(context, &mut scores, &mut reasoning);
        
        // Determine primary mode
        let (primary_mode, confidence) = scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(mode, score)| (mode.clone(), *score))
            .unwrap_or((ModeType::Hybrid, 0.5));
        
        // Adjust for hybrid mode if scores are close
        let sorted_scores: Vec<f32> = {
            let mut vals: Vec<f32> = scores.values().copied().collect();
            vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            vals
        };
        
        let final_mode = if sorted_scores.len() >= 2 && 
            (sorted_scores[0] - sorted_scores[1]) < 0.2 && 
            confidence > 0.3 {
            reasoning.push("Scores are close, using hybrid mode".to_string());
            ModeType::Hybrid
        } else {
            primary_mode
        };
        
        Ok(DetectionResult {
            primary_mode: final_mode,
            confidence,
            scores,
            reasoning,
        })
    }

    // Private analysis methods

    fn analyze_keywords(&self, query: &str, scores: &mut HashMap<ModeType, f32>, reasoning: &mut Vec<String>) -> HiveResult<()> {
        let query_lower = query.to_lowercase();
        
        // Check planning patterns
        let planning_matches = self.patterns.planning_patterns.iter()
            .filter(|p| p.is_match(&query_lower))
            .count();
        
        if planning_matches > 0 {
            let score = (planning_matches as f32 * 0.3).min(1.0);
            *scores.get_mut(&ModeType::Planning).unwrap() += score * self.weights.keyword_weight;
            reasoning.push(format!("Found {} planning keywords", planning_matches));
        }
        
        // Check execution patterns
        let execution_matches = self.patterns.execution_patterns.iter()
            .filter(|p| p.is_match(&query_lower))
            .count();
        
        if execution_matches > 0 {
            let score = (execution_matches as f32 * 0.3).min(1.0);
            *scores.get_mut(&ModeType::Execution).unwrap() += score * self.weights.keyword_weight;
            reasoning.push(format!("Found {} execution keywords", execution_matches));
        }
        
        // Check analysis patterns
        let analysis_matches = self.patterns.analysis_patterns.iter()
            .filter(|p| p.is_match(&query_lower))
            .count();
        
        if analysis_matches > 0 {
            let score = (analysis_matches as f32 * 0.3).min(1.0);
            *scores.get_mut(&ModeType::Analysis).unwrap() += score * self.weights.keyword_weight;
            reasoning.push(format!("Found {} analysis keywords", analysis_matches));
        }
        
        // Check hybrid indicators
        let hybrid_matches = self.patterns.hybrid_indicators.iter()
            .filter(|p| p.is_match(&query_lower))
            .count();
        
        if hybrid_matches > 0 {
            let score = (hybrid_matches as f32 * 0.2).min(1.0);
            *scores.get_mut(&ModeType::Hybrid).unwrap() += score * self.weights.keyword_weight;
            reasoning.push(format!("Found {} hybrid indicators", hybrid_matches));
        }
        
        Ok(())
    }

    fn analyze_context(&self, context: &PlanningContext, scores: &mut HashMap<ModeType, f32>, reasoning: &mut Vec<String>) {
        // Project type influences mode choice
        match context.project_type {
            ProjectType::Library | ProjectType::API => {
                *scores.get_mut(&ModeType::Execution).unwrap() += 0.2 * self.weights.context_weight;
                reasoning.push("Library/API projects favor execution mode".to_string());
            }
            ProjectType::WebApplication | ProjectType::MobileApp => {
                *scores.get_mut(&ModeType::Hybrid).unwrap() += 0.3 * self.weights.context_weight;
                reasoning.push("Complex applications benefit from hybrid mode".to_string());
            }
            ProjectType::Infrastructure | ProjectType::DataPipeline => {
                *scores.get_mut(&ModeType::Planning).unwrap() += 0.3 * self.weights.context_weight;
                reasoning.push("Infrastructure projects need careful planning".to_string());
            }
            _ => {}
        }
        
        // Team size affects mode preference
        if context.team_size > 3 {
            *scores.get_mut(&ModeType::Planning).unwrap() += 0.2 * self.weights.context_weight;
            reasoning.push("Large teams benefit from planning mode".to_string());
        } else if context.team_size == 1 {
            *scores.get_mut(&ModeType::Execution).unwrap() += 0.1 * self.weights.context_weight;
            reasoning.push("Solo development can focus on execution".to_string());
        }
        
        // Experience level affects mode choice
        match context.experience_level {
            ExperienceLevel::Beginner => {
                *scores.get_mut(&ModeType::Planning).unwrap() += 0.2 * self.weights.context_weight;
                reasoning.push("Beginners benefit from detailed planning".to_string());
            }
            ExperienceLevel::Expert => {
                *scores.get_mut(&ModeType::Execution).unwrap() += 0.1 * self.weights.context_weight;
                reasoning.push("Experts can focus more on execution".to_string());
            }
            _ => {}
        }
        
        // Existing codebase suggests analysis mode
        if context.existing_codebase {
            *scores.get_mut(&ModeType::Analysis).unwrap() += 0.2 * self.weights.context_weight;
            reasoning.push("Existing codebase suggests analysis mode".to_string());
        }
    }

    fn analyze_complexity(&self, query: &str, scores: &mut HashMap<ModeType, f32>, reasoning: &mut Vec<String>) {
        let complexity_score = self.calculate_query_complexity(query);
        
        if complexity_score > 0.7 {
            *scores.get_mut(&ModeType::Planning).unwrap() += 0.3 * self.weights.complexity_weight;
            *scores.get_mut(&ModeType::Hybrid).unwrap() += 0.2 * self.weights.complexity_weight;
            reasoning.push("High complexity query benefits from planning".to_string());
        } else if complexity_score < 0.3 {
            *scores.get_mut(&ModeType::Execution).unwrap() += 0.2 * self.weights.complexity_weight;
            reasoning.push("Low complexity query can focus on execution".to_string());
        } else {
            *scores.get_mut(&ModeType::Hybrid).unwrap() += 0.1 * self.weights.complexity_weight;
            reasoning.push("Medium complexity suggests hybrid approach".to_string());
        }
    }

    fn apply_user_preferences(&self, context: &PlanningContext, scores: &mut HashMap<ModeType, f32>, reasoning: &mut Vec<String>) {
        let preferred_mode = &context.user_preferences.preferred_mode;
        
        if let Some(score) = scores.get_mut(preferred_mode) {
            *score += 0.3 * self.weights.user_preference_weight;
            reasoning.push(format!("User prefers {} mode", mode_to_string(preferred_mode)));
        }
        
        // Detail level affects mode preference
        match context.user_preferences.detail_level {
            DetailLevel::High => {
                *scores.get_mut(&ModeType::Planning).unwrap() += 0.1 * self.weights.user_preference_weight;
                reasoning.push("High detail preference favors planning".to_string());
            }
            DetailLevel::Low => {
                *scores.get_mut(&ModeType::Execution).unwrap() += 0.1 * self.weights.user_preference_weight;
                reasoning.push("Low detail preference favors execution".to_string());
            }
            _ => {}
        }
    }

    fn calculate_query_complexity(&self, query: &str) -> f32 {
        let words = query.split_whitespace().count();
        let sentences = query.split('.').count();
        let technical_words = query.to_lowercase()
            .split_whitespace()
            .filter(|w| self.is_technical_word(w))
            .count();
        
        // Normalize complexity score between 0 and 1
        let word_complexity = (words as f32 / 50.0).min(1.0);
        let sentence_complexity = (sentences as f32 / 5.0).min(1.0);
        let technical_complexity = (technical_words as f32 / 10.0).min(1.0);
        
        (word_complexity + sentence_complexity + technical_complexity) / 3.0
    }

    fn is_technical_word(&self, word: &str) -> bool {
        let technical_terms = [
            "implement", "refactor", "optimize", "architecture", "database", 
            "api", "algorithm", "framework", "library", "deployment",
            "testing", "security", "performance", "scalability", "integration"
        ];
        
        technical_terms.contains(&word)
    }

    // Initialization methods

    fn init_patterns() -> ModePatterns {
        ModePatterns {
            planning_patterns: vec![
                Regex::new(r"\b(plan|design|strategy|roadmap|organize)\b").unwrap(),
                Regex::new(r"\b(break down|decompose|analyze requirements)\b").unwrap(),
                Regex::new(r"\b(timeline|schedule|estimate|dependencies)\b").unwrap(),
                Regex::new(r"\b(risk|mitigation|contingency)\b").unwrap(),
                Regex::new(r"\bhow to (approach|structure|organize)\b").unwrap(),
            ],
            execution_patterns: vec![
                Regex::new(r"\b(implement|code|build|create|develop)\b").unwrap(),
                Regex::new(r"\b(fix|debug|solve|resolve)\b").unwrap(),
                Regex::new(r"\b(add|remove|modify|update|change)\b").unwrap(),
                Regex::new(r"\b(write|generate|produce)\b").unwrap(),
                Regex::new(r"\b(quick|fast|immediately|now)\b").unwrap(),
            ],
            analysis_patterns: vec![
                Regex::new(r"\b(analyze|examine|review|understand)\b").unwrap(),
                Regex::new(r"\b(what does|how does|why does)\b").unwrap(),
                Regex::new(r"\b(explain|describe|show me)\b").unwrap(),
                Regex::new(r"\b(find|search|locate|identify)\b").unwrap(),
                Regex::new(r"\b(performance|bottleneck|issue|problem)\b").unwrap(),
            ],
            hybrid_indicators: vec![
                Regex::new(r"\b(complex|comprehensive|full|complete)\b").unwrap(),
                Regex::new(r"\b(both|and|also|additionally)\b").unwrap(),
                Regex::new(r"\b(step by step|end to end)\b").unwrap(),
                Regex::new(r"\b(multiple|several|various)\b").unwrap(),
            ],
        }
    }

    fn init_weights() -> ModeWeights {
        ModeWeights {
            keyword_weight: 0.4,
            context_weight: 0.3,
            complexity_weight: 0.2,
            user_preference_weight: 0.1,
        }
    }
}

fn mode_to_string(mode: &ModeType) -> &'static str {
    match mode {
        ModeType::Planning => "Planning",
        ModeType::Execution => "Execution",
        ModeType::Hybrid => "Hybrid",
        ModeType::Analysis => "Analysis",
        ModeType::Learning => "Learning",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_detector_creation() {
        let detector = ModeDetector::new();
        assert!(detector.patterns.planning_patterns.len() > 0);
        assert!(detector.patterns.execution_patterns.len() > 0);
    }

    #[test]
    fn test_planning_mode_detection() {
        let detector = ModeDetector::new();
        let context = PlanningContext::default();
        
        let mode = detector.detect_mode("How should I plan this new feature?", &context).unwrap();
        assert_eq!(mode, ModeType::Planning);
    }

    #[test]
    fn test_execution_mode_detection() {
        let detector = ModeDetector::new();
        let context = PlanningContext::default();
        
        let mode = detector.detect_mode("Implement authentication system", &context).unwrap();
        assert_eq!(mode, ModeType::Execution);
    }

    #[test]
    fn test_analysis_mode_detection() {
        let detector = ModeDetector::new();
        let context = PlanningContext::default();
        
        let mode = detector.detect_mode("What does this function do?", &context).unwrap();
        assert_eq!(mode, ModeType::Analysis);
    }

    #[test]
    fn test_hybrid_mode_detection() {
        let detector = ModeDetector::new();
        let context = PlanningContext::default();
        
        let mode = detector.detect_mode("Plan and implement a comprehensive testing strategy", &context).unwrap();
        assert_eq!(mode, ModeType::Hybrid);
    }

    #[test]
    fn test_complexity_calculation() {
        let detector = ModeDetector::new();
        
        let simple_complexity = detector.calculate_query_complexity("Fix bug");
        let complex_complexity = detector.calculate_query_complexity(
            "Design and implement a comprehensive microservices architecture with proper API gateway, service discovery, and monitoring systems."
        );
        
        assert!(simple_complexity < complex_complexity);
    }

    #[test]
    fn test_user_preference_influence() {
        let detector = ModeDetector::new();
        let mut context = PlanningContext::default();
        context.user_preferences.preferred_mode = ModeType::Planning;
        
        let result = detector.analyze_query("implement feature", &context).unwrap();
        
        // Planning should have some score due to user preference, even for execution query
        assert!(result.scores.get(&ModeType::Planning).unwrap() > &0.0);
    }
}