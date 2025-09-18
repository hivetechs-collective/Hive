//! Pattern learning system with machine learning capabilities
//!
//! This module provides:
//! - Pattern recognition from conversation history
//! - Clustering of similar patterns
//! - Pattern effectiveness tracking
//! - Automated learning and improvement

use anyhow::{Context as _, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use tracing::{debug, info};

use crate::core::database::KnowledgeConversation;
use crate::core::memory::MemorySystem;

/// Pattern learning system
#[derive(Debug)]
pub struct PatternLearner {
    /// Discovered patterns
    patterns: Vec<Pattern>,
    /// Pattern index by type
    pattern_index: HashMap<PatternType, Vec<usize>>,
    /// Pattern statistics
    pattern_stats: HashMap<String, PatternStatistics>,
    /// Learning configuration
    config: PatternLearningConfig,
    /// Pattern clusters
    clusters: Vec<PatternCluster>,
}

/// Configuration for pattern learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternLearningConfig {
    /// Minimum occurrences to consider a pattern
    pub min_occurrences: usize,
    /// Minimum confidence threshold
    pub min_confidence: f32,
    /// Maximum patterns to keep
    pub max_patterns: usize,
    /// Enable clustering
    pub enable_clustering: bool,
    /// Clustering threshold
    pub clustering_threshold: f32,
}

impl Default for PatternLearningConfig {
    fn default() -> Self {
        Self {
            min_occurrences: 3,
            min_confidence: 0.7,
            max_patterns: 1000,
            enable_clustering: true,
            clustering_threshold: 0.8,
        }
    }
}

/// A learned pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Unique identifier
    pub id: String,
    /// Pattern type
    pub pattern_type: PatternType,
    /// Pattern template/signature
    pub template: String,
    /// Example instances
    pub examples: Vec<PatternExample>,
    /// Pattern confidence score
    pub confidence: f32,
    /// Frequency of occurrence
    pub frequency: usize,
    /// Effectiveness score
    pub effectiveness: f32,
    /// When first discovered
    pub discovered_at: DateTime<Utc>,
    /// When last seen
    pub last_seen: DateTime<Utc>,
    /// Associated tags
    pub tags: HashSet<String>,
}

/// Example of a pattern instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExample {
    /// Input that triggered the pattern
    pub input: String,
    /// Output/response
    pub output: String,
    /// Context if available
    pub context: Option<String>,
    /// Quality score
    pub quality_score: f32,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Types of patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    /// Question-answer patterns
    QuestionAnswer,
    /// Problem-solution patterns
    ProblemSolution,
    /// Command sequence patterns
    CommandSequence,
    /// Error resolution patterns
    ErrorResolution,
    /// Best practice patterns
    BestPractice,
    /// Workflow patterns
    Workflow,
    /// Code transformation patterns
    CodeTransformation,
    /// Explanation patterns
    Explanation,
    /// Debugging patterns
    Debugging,
}

impl fmt::Display for PatternType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternType::QuestionAnswer => write!(f, "Q&A"),
            PatternType::ProblemSolution => write!(f, "Problem-Solution"),
            PatternType::CommandSequence => write!(f, "Command Sequence"),
            PatternType::ErrorResolution => write!(f, "Error Resolution"),
            PatternType::BestPractice => write!(f, "Best Practice"),
            PatternType::Workflow => write!(f, "Workflow"),
            PatternType::CodeTransformation => write!(f, "Code Transform"),
            PatternType::Explanation => write!(f, "Explanation"),
            PatternType::Debugging => write!(f, "Debugging"),
        }
    }
}

/// Pattern statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PatternStatistics {
    /// Total occurrences
    pub occurrences: usize,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f32,
    /// Average quality score
    pub avg_quality_score: f32,
    /// Usage trend (positive = increasing)
    pub usage_trend: f32,
    /// Last updated
    pub last_updated: Option<DateTime<Utc>>,
}

/// Pattern cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCluster {
    /// Cluster ID
    pub id: String,
    /// Cluster centroid pattern
    pub centroid: String,
    /// Member pattern IDs
    pub members: Vec<String>,
    /// Cluster quality score
    pub quality: f32,
    /// Common characteristics
    pub characteristics: HashMap<String, String>,
}

/// Pattern metrics for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetrics {
    /// Total patterns discovered
    pub total_patterns: usize,
    /// Patterns by type
    pub patterns_by_type: HashMap<PatternType, usize>,
    /// Average confidence
    pub avg_confidence: f32,
    /// Average effectiveness
    pub avg_effectiveness: f32,
    /// Most common pattern type
    pub most_common_type: Option<PatternType>,
    /// Pattern growth rate
    pub growth_rate: f32,
}

impl PatternLearner {
    /// Create a new pattern learner
    pub fn new() -> Self {
        Self::with_config(PatternLearningConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: PatternLearningConfig) -> Self {
        Self {
            patterns: Vec::new(),
            pattern_index: HashMap::new(),
            pattern_stats: HashMap::new(),
            config,
            clusters: Vec::new(),
        }
    }

    /// Learn patterns from memory system
    pub async fn learn_from_memories(&mut self, _memory_system: &MemorySystem) -> Result<()> {
        info!("Learning patterns from memory system");

        // Get recent conversations
        let conversations = KnowledgeConversation::get_recent(1000).await?;

        // Process each conversation
        for conv in &conversations {
            self.process_conversation(
                &conv.question,
                &conv.final_answer,
                conv.conversation_context.as_deref(),
            )?;
        }

        // Cluster similar patterns if enabled
        if self.config.enable_clustering {
            self.cluster_patterns()?;
        }

        // Prune low-quality patterns
        self.prune_patterns();

        info!("Discovered {} patterns", self.patterns.len());
        Ok(())
    }

    /// Process a single conversation
    pub fn process_conversation(
        &mut self,
        question: &str,
        answer: &str,
        context: Option<&str>,
    ) -> Result<()> {
        // Detect pattern type
        let pattern_type = self.detect_pattern_type(question, answer);

        // Extract pattern template
        let template = self.extract_template(question, answer, &pattern_type)?;

        // Find or create pattern
        let pattern_id = format!(
            "{:?}_{}",
            pattern_type,
            template.chars().take(20).collect::<String>()
        );

        // Update pattern statistics
        {
            let stats = self.pattern_stats.entry(pattern_id.clone()).or_default();
            stats.occurrences += 1;
            stats.last_updated = Some(Utc::now());
        }

        // Check if pattern meets threshold
        let occurrences = self
            .pattern_stats
            .get(&pattern_id)
            .map(|s| s.occurrences)
            .unwrap_or(0);
        if occurrences >= self.config.min_occurrences {
            // Calculate values before mutating
            let quality_score = self.calculate_quality_score(question, answer);
            let stats = self.pattern_stats.get(&pattern_id).cloned();

            // Find pattern and calculate confidence if needed
            let pattern_exists = self.patterns.iter().any(|p| p.id == pattern_id);
            let new_confidence = if pattern_exists && stats.is_some() {
                // Find the pattern temporarily to calculate confidence
                let pattern = self.patterns.iter().find(|p| p.id == pattern_id).unwrap();
                Some(self.calculate_confidence(pattern, stats.as_ref().unwrap()))
            } else {
                None
            };

            // Find existing pattern or create new one
            if let Some(pattern) = self.patterns.iter_mut().find(|p| p.id == pattern_id) {
                // Update existing pattern
                pattern.frequency = occurrences;
                pattern.last_seen = Utc::now();

                // Add example
                if pattern.examples.len() < 10 {
                    pattern.examples.push(PatternExample {
                        input: question.to_string(),
                        output: answer.to_string(),
                        context: context.map(String::from),
                        quality_score,
                        timestamp: Utc::now(),
                    });
                }

                // Update confidence
                if let Some(confidence) = new_confidence {
                    pattern.confidence = confidence;
                }
            } else {
                // Create new pattern
                let pattern = Pattern {
                    id: pattern_id.clone(),
                    pattern_type: pattern_type.clone(),
                    template: template.clone(),
                    examples: vec![PatternExample {
                        input: question.to_string(),
                        output: answer.to_string(),
                        context: context.map(String::from),
                        quality_score: self.calculate_quality_score(question, answer),
                        timestamp: Utc::now(),
                    }],
                    confidence: 0.8,
                    frequency: occurrences,
                    effectiveness: 0.8,
                    discovered_at: Utc::now(),
                    last_seen: Utc::now(),
                    tags: self.extract_tags(question, answer),
                };

                // Add to patterns
                let index = self.patterns.len();
                self.patterns.push(pattern);

                // Update index
                self.pattern_index
                    .entry(pattern_type)
                    .or_insert_with(Vec::new)
                    .push(index);
            }
        }

        Ok(())
    }

    /// Get all discovered patterns
    pub fn get_patterns(&self) -> Vec<Pattern> {
        self.patterns.clone()
    }

    /// Get patterns by type
    pub fn get_patterns_by_type(&self, pattern_type: &PatternType) -> Vec<&Pattern> {
        self.pattern_index
            .get(pattern_type)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.patterns.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find similar patterns
    pub fn find_similar(&self, input: &str, limit: usize) -> Vec<&Pattern> {
        let mut scored_patterns: Vec<(&Pattern, f32)> = self
            .patterns
            .iter()
            .map(|pattern| {
                let score = self.calculate_similarity(input, &pattern.template);
                (pattern, score)
            })
            .collect();

        // Sort by similarity score
        scored_patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        scored_patterns
            .into_iter()
            .take(limit)
            .map(|(pattern, _)| pattern)
            .collect()
    }

    /// Get pattern metrics
    pub fn get_metrics(&self) -> PatternMetrics {
        let mut patterns_by_type = HashMap::new();
        let mut total_confidence = 0.0;
        let mut total_effectiveness = 0.0;

        for pattern in &self.patterns {
            *patterns_by_type
                .entry(pattern.pattern_type.clone())
                .or_insert(0) += 1;
            total_confidence += pattern.confidence;
            total_effectiveness += pattern.effectiveness;
        }

        let total_patterns = self.patterns.len();
        let avg_confidence = if total_patterns > 0 {
            total_confidence / total_patterns as f32
        } else {
            0.0
        };

        let avg_effectiveness = if total_patterns > 0 {
            total_effectiveness / total_patterns as f32
        } else {
            0.0
        };

        let most_common_type = patterns_by_type
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(pattern_type, _)| pattern_type.clone());

        PatternMetrics {
            total_patterns,
            patterns_by_type,
            avg_confidence,
            avg_effectiveness,
            most_common_type,
            growth_rate: self.calculate_growth_rate(),
        }
    }

    // Private helper methods

    fn detect_pattern_type(&self, question: &str, answer: &str) -> PatternType {
        let q_lower = question.to_lowercase();
        let a_lower = answer.to_lowercase();

        if q_lower.starts_with("how do i") || q_lower.starts_with("how to") {
            PatternType::QuestionAnswer
        } else if q_lower.contains("error") || a_lower.contains("fix") || a_lower.contains("solve")
        {
            PatternType::ErrorResolution
        } else if q_lower.contains("best practice") || q_lower.contains("recommended") {
            PatternType::BestPractice
        } else if q_lower.contains("debug") || q_lower.contains("troubleshoot") {
            PatternType::Debugging
        } else if q_lower.contains("transform")
            || q_lower.contains("convert")
            || q_lower.contains("refactor")
        {
            PatternType::CodeTransformation
        } else if q_lower.contains("explain")
            || q_lower.contains("what is")
            || q_lower.contains("describe")
        {
            PatternType::Explanation
        } else if a_lower.contains("step") || a_lower.contains("workflow") {
            PatternType::Workflow
        } else if a_lower.contains("command") || a_lower.contains("cli") {
            PatternType::CommandSequence
        } else {
            PatternType::ProblemSolution
        }
    }

    fn extract_template(
        &self,
        question: &str,
        answer: &str,
        pattern_type: &PatternType,
    ) -> Result<String> {
        // Simple template extraction
        // In production, this would use more sophisticated NLP
        match pattern_type {
            PatternType::QuestionAnswer => {
                if question.starts_with("How do I") {
                    Ok("How do I [ACTION] -> [SOLUTION]".to_string())
                } else if question.starts_with("What is") {
                    Ok("What is [CONCEPT] -> [DEFINITION]".to_string())
                } else {
                    Ok("[QUESTION] -> [ANSWER]".to_string())
                }
            }
            PatternType::ErrorResolution => Ok("[ERROR] -> [RESOLUTION]".to_string()),
            PatternType::CommandSequence => Ok("[TASK] -> [COMMANDS]".to_string()),
            _ => Ok(format!("[{}] -> [RESPONSE]", pattern_type)),
        }
    }

    fn calculate_quality_score(&self, question: &str, answer: &str) -> f32 {
        // Simple quality heuristics
        let mut score: f32 = 0.5;

        // Length ratio
        let ratio = answer.len() as f32 / question.len() as f32;
        if ratio > 2.0 && ratio < 20.0 {
            score += 0.1;
        }

        // Contains code blocks
        if answer.contains("```") {
            score += 0.1;
        }

        // Contains structured elements
        if answer.contains("1.") || answer.contains("•") || answer.contains("-") {
            score += 0.1;
        }

        // Contains explanations
        if answer.contains("because") || answer.contains("since") || answer.contains("therefore") {
            score += 0.1;
        }

        // Well-formed question
        if question.ends_with('?') {
            score += 0.1;
        }

        score.min(1.0_f32)
    }

    fn calculate_confidence(&self, pattern: &Pattern, stats: &PatternStatistics) -> f32 {
        let mut confidence = 0.5;

        // Frequency component
        let freq_score = (stats.occurrences as f32 / 100.0).min(0.3);
        confidence += freq_score;

        // Quality component
        let avg_quality = pattern
            .examples
            .iter()
            .map(|e| e.quality_score)
            .sum::<f32>()
            / pattern.examples.len() as f32;
        confidence += avg_quality * 0.2;

        confidence.min(1.0)
    }

    fn extract_tags(&self, question: &str, answer: &str) -> HashSet<String> {
        let mut tags = HashSet::new();
        let combined = format!("{} {}", question, answer).to_lowercase();

        // Common technology tags
        let tech_tags = [
            "rust",
            "python",
            "javascript",
            "typescript",
            "api",
            "database",
            "cli",
            "web",
        ];
        for tag in &tech_tags {
            if combined.contains(tag) {
                tags.insert(tag.to_string());
            }
        }

        // Common concept tags
        let concept_tags = [
            "async",
            "performance",
            "security",
            "testing",
            "debugging",
            "optimization",
        ];
        for tag in &concept_tags {
            if combined.contains(tag) {
                tags.insert(tag.to_string());
            }
        }

        tags
    }

    fn calculate_similarity(&self, text1: &str, text2: &str) -> f32 {
        // Simple Jaccard similarity
        let words1: HashSet<&str> = text1.split_whitespace().collect();
        let words2: HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count() as f32;
        let union = words1.union(&words2).count() as f32;

        if union > 0.0 {
            intersection / union
        } else {
            0.0
        }
    }

    fn cluster_patterns(&mut self) -> Result<()> {
        debug!(
            "Clustering patterns with threshold {}",
            self.config.clustering_threshold
        );

        // Simple clustering algorithm
        // In production, use proper clustering like DBSCAN or hierarchical clustering
        let mut clusters: Vec<PatternCluster> = Vec::new();
        let mut assigned: HashSet<String> = HashSet::new();

        for pattern in &self.patterns {
            if assigned.contains(&pattern.id) {
                continue;
            }

            // Create new cluster
            let mut cluster = PatternCluster {
                id: format!("cluster_{}", clusters.len()),
                centroid: pattern.template.clone(),
                members: vec![pattern.id.clone()],
                quality: 1.0,
                characteristics: HashMap::new(),
            };

            assigned.insert(pattern.id.clone());

            // Find similar patterns
            for other in &self.patterns {
                if assigned.contains(&other.id) {
                    continue;
                }

                let similarity = self.calculate_similarity(&pattern.template, &other.template);
                if similarity >= self.config.clustering_threshold {
                    cluster.members.push(other.id.clone());
                    assigned.insert(other.id.clone());
                }
            }

            clusters.push(cluster);
        }

        self.clusters = clusters;
        debug!("Created {} pattern clusters", self.clusters.len());

        Ok(())
    }

    fn prune_patterns(&mut self) {
        // Remove patterns below confidence threshold
        self.patterns
            .retain(|p| p.confidence >= self.config.min_confidence);

        // Keep only top patterns if exceeding limit
        if self.patterns.len() > self.config.max_patterns {
            self.patterns.sort_by(|a, b| {
                b.effectiveness
                    .partial_cmp(&a.effectiveness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.patterns.truncate(self.config.max_patterns);
        }

        // Rebuild index
        self.pattern_index.clear();
        for (i, pattern) in self.patterns.iter().enumerate() {
            self.pattern_index
                .entry(pattern.pattern_type.clone())
                .or_insert_with(Vec::new)
                .push(i);
        }
    }

    fn calculate_growth_rate(&self) -> f32 {
        // Simple growth rate calculation
        // In production, use time-series analysis
        let recent_patterns = self
            .patterns
            .iter()
            .filter(|p| {
                let days_old = (Utc::now() - p.discovered_at).num_days();
                days_old <= 7
            })
            .count();

        let older_patterns = self
            .patterns
            .iter()
            .filter(|p| {
                let days_old = (Utc::now() - p.discovered_at).num_days();
                days_old > 7 && days_old <= 14
            })
            .count();

        if older_patterns > 0 {
            (recent_patterns as f32 - older_patterns as f32) / older_patterns as f32
        } else {
            0.0
        }
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_detection() {
        let learner = PatternLearner::new();

        let test_cases = vec![
            ("How do I parse JSON in Rust?", PatternType::QuestionAnswer),
            ("Error: cannot find module", PatternType::ErrorResolution),
            (
                "What is the best practice for error handling?",
                PatternType::BestPractice,
            ),
            ("Debug the async function", PatternType::Debugging),
            (
                "Transform this code to use async/await",
                PatternType::CodeTransformation,
            ),
        ];

        for (question, expected_type) in test_cases {
            let detected_type = learner.detect_pattern_type(question, "answer");
            assert_eq!(detected_type, expected_type, "Failed for: {}", question);
        }
    }

    #[test]
    fn test_pattern_learning() -> Result<()> {
        let mut learner = PatternLearner::with_config(PatternLearningConfig {
            min_occurrences: 2,
            ..Default::default()
        });

        // Process similar conversations
        for i in 0..3 {
            learner.process_conversation(
                &format!("How do I read a file in Rust?"),
                &format!("Use std::fs::read_to_string() method"),
                None,
            )?;
        }

        let patterns = learner.get_patterns();
        assert!(!patterns.is_empty());
        assert_eq!(patterns[0].pattern_type, PatternType::QuestionAnswer);
        assert!(patterns[0].frequency >= 2);

        Ok(())
    }
}
