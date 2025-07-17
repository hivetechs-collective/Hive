//! Quality Analyzer - Uses CodeT5+ for analyzing and validating Curator outputs
//! 
//! This module evaluates the quality of Curator outputs, detects contradictions,
//! measures confidence, and ensures consistency across the knowledge base.

use std::sync::Arc;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::ai_helpers::{QualityReport, QualityIssue, Severity, IndexedKnowledge};
use super::python_models::{PythonModelService, ModelRequest, ModelResponse};

/// Configuration for Quality Analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Model for quality analysis
    pub analysis_model: String,
    
    /// Minimum acceptable quality score
    pub min_quality_score: f64,
    
    /// Weights for different quality aspects
    pub consistency_weight: f64,
    pub completeness_weight: f64,
    pub accuracy_weight: f64,
    pub clarity_weight: f64,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            analysis_model: "Salesforce/codet5p-110m-embedding".to_string(),
            min_quality_score: 0.7,
            consistency_weight: 0.3,
            completeness_weight: 0.25,
            accuracy_weight: 0.3,
            clarity_weight: 0.15,
        }
    }
}

/// Quality Analyzer using CodeT5+
pub struct QualityAnalyzer {
    config: QualityConfig,
    
    /// Python model service
    python_service: Arc<PythonModelService>,
    
    /// Historical quality metrics
    quality_history: Arc<RwLock<QualityHistory>>,
    
    /// Cache of recent analyses
    analysis_cache: Arc<RwLock<lru::LruCache<String, QualityReport>>>,
}

/// Historical quality tracking
#[derive(Default)]
struct QualityHistory {
    /// Average quality scores over time
    score_history: Vec<(chrono::DateTime<chrono::Utc>, f64)>,
    
    /// Common issues and their frequencies
    issue_frequencies: std::collections::HashMap<String, usize>,
    
    /// Quality trends
    trends: Vec<QualityTrend>,
}

#[derive(Debug, Clone)]
struct QualityTrend {
    trend_type: TrendType,
    description: String,
    start_time: chrono::DateTime<chrono::Utc>,
    strength: f64,
}

#[derive(Debug, Clone, Copy)]
enum TrendType {
    Improving,
    Declining,
    Stable,
    Volatile,
}

impl QualityAnalyzer {
    /// Create a new Quality Analyzer
    pub async fn new(python_service: Arc<PythonModelService>) -> Result<Self> {
        let config = QualityConfig::default();
        let quality_history = Arc::new(RwLock::new(QualityHistory::default()));
        let analysis_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(200).unwrap()
        )));
        
        Ok(Self {
            config,
            python_service,
            quality_history,
            analysis_cache,
        })
    }
    
    /// Evaluate the quality of a Curator output
    pub async fn evaluate_quality(
        &self,
        indexed: &IndexedKnowledge,
        raw_output: &str,
    ) -> Result<QualityReport> {
        // Check cache
        if let Some(cached) = self.analysis_cache.read().await.peek(&indexed.id) {
            return Ok(cached.clone());
        }
        
        // Perform comprehensive quality analysis
        let consistency_score = self.analyze_consistency(indexed, raw_output).await?;
        let completeness_score = self.analyze_completeness(indexed, raw_output).await?;
        let accuracy_score = self.analyze_accuracy(indexed, raw_output).await?;
        let clarity_score = self.analyze_clarity(raw_output).await?;
        
        // Calculate weighted overall score
        let overall_score = 
            consistency_score * self.config.consistency_weight +
            completeness_score * self.config.completeness_weight +
            accuracy_score * self.config.accuracy_weight +
            clarity_score * self.config.clarity_weight;
        
        // Detect quality issues
        let issues = self.detect_issues(
            consistency_score,
            completeness_score,
            accuracy_score,
            clarity_score,
            raw_output,
        ).await?;
        
        let report = QualityReport {
            overall_score,
            consistency_score,
            completeness_score,
            accuracy_score,
            issues,
        };
        
        // Update history
        self.update_quality_history(&report).await?;
        
        // Cache result
        self.analysis_cache.write().await.put(indexed.id.clone(), report.clone());
        
        Ok(report)
    }
    
    /// Analyze consistency with existing knowledge
    async fn analyze_consistency(
        &self,
        indexed: &IndexedKnowledge,
        raw_output: &str,
    ) -> Result<f64> {
        // Use AI model to analyze consistency
        let analysis_result = self.python_service
            .analyze_code(
                &self.config.analysis_model,
                raw_output,
                "consistency"
            )
            .await?;
        
        // Extract consistency score from analysis
        let mut score = analysis_result
            .get("consistency_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.85);
        
        // Additional heuristic checks
        if raw_output.contains("however") || raw_output.contains("actually") {
            score -= 0.1;
        }
        
        // Check metadata consistency
        if let Some(source) = indexed.metadata.get("source_question") {
            if !raw_output.to_lowercase().contains(&source.as_str().unwrap_or("").to_lowercase()) {
                score -= 0.05;
            }
        }
        
        Ok(score.max(0.0).min(1.0))
    }
    
    /// Analyze completeness of the response
    async fn analyze_completeness(
        &self,
        indexed: &IndexedKnowledge,
        raw_output: &str,
    ) -> Result<f64> {
        // Check if response fully addresses the question
        let mut score: f64 = 1.0;
        
        // Basic heuristics
        let word_count = raw_output.split_whitespace().count();
        if word_count < 20 {
            score -= 0.2; // Too brief
        }
        
        // Check for common completeness indicators
        if raw_output.contains("for example") || raw_output.contains("such as") {
            score += 0.05; // Provides examples
        }
        
        if raw_output.contains("in summary") || raw_output.contains("to conclude") {
            score += 0.05; // Has conclusion
        }
        
        Ok(score.min(1.0).max(0.0))
    }
    
    /// Analyze accuracy of the information
    async fn analyze_accuracy(
        &self,
        indexed: &IndexedKnowledge,
        raw_output: &str,
    ) -> Result<f64> {
        // Use AI model to analyze accuracy
        let analysis_result = self.python_service
            .analyze_code(
                &self.config.analysis_model,
                raw_output,
                "accuracy"
            )
            .await?;
        
        // Extract accuracy score from analysis
        let mut score = analysis_result
            .get("accuracy_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.9);
        
        // Additional checks for uncertainty markers
        if raw_output.contains("might") || raw_output.contains("possibly") {
            score -= 0.1;
        }
        
        // Boost for specific claims
        if raw_output.contains("specifically") || raw_output.contains("exactly") {
            score += 0.05;
        }
        
        Ok(score.min(1.0).max(0.0))
    }
    
    /// Analyze clarity of the response
    async fn analyze_clarity(&self, raw_output: &str) -> Result<f64> {
        let mut score: f64 = 1.0;
        
        // Check sentence structure
        let sentences: Vec<&str> = raw_output.split(". ").collect();
        let avg_sentence_length = sentences.iter()
            .map(|s| s.split_whitespace().count())
            .sum::<usize>() as f64 / sentences.len().max(1) as f64;
        
        // Penalize overly long sentences
        if avg_sentence_length > 30.0 {
            score -= 0.1;
        }
        
        // Check for jargon without explanation
        let technical_terms = ["algorithm", "implementation", "architecture", "framework"];
        let mut unexplained_jargon = 0;
        for term in technical_terms {
            if raw_output.contains(term) && !raw_output.contains(&format!("{} is", term)) {
                unexplained_jargon += 1;
            }
        }
        score -= unexplained_jargon as f64 * 0.05;
        
        Ok(score.max(0.0))
    }
    
    /// Detect specific quality issues
    async fn detect_issues(
        &self,
        consistency: f64,
        completeness: f64,
        accuracy: f64,
        clarity: f64,
        raw_output: &str,
    ) -> Result<Vec<QualityIssue>> {
        let mut issues = Vec::new();
        
        // Consistency issues
        if consistency < 0.7 {
            issues.push(QualityIssue {
                issue_type: "Consistency".to_string(),
                description: "Response may contradict existing knowledge".to_string(),
                severity: if consistency < 0.5 { Severity::High } else { Severity::Medium },
            });
        }
        
        // Completeness issues
        if completeness < 0.7 {
            issues.push(QualityIssue {
                issue_type: "Completeness".to_string(),
                description: "Response may not fully address the question".to_string(),
                severity: if completeness < 0.5 { Severity::High } else { Severity::Medium },
            });
        }
        
        // Accuracy issues
        if accuracy < 0.8 {
            issues.push(QualityIssue {
                issue_type: "Accuracy".to_string(),
                description: "Response contains uncertain or unverifiable claims".to_string(),
                severity: if accuracy < 0.6 { Severity::High } else { Severity::Medium },
            });
        }
        
        // Clarity issues
        if clarity < 0.7 {
            issues.push(QualityIssue {
                issue_type: "Clarity".to_string(),
                description: "Response could be clearer or more concise".to_string(),
                severity: Severity::Low,
            });
        }
        
        // Check for critical issues
        if raw_output.contains("error") || raw_output.contains("fail") {
            issues.push(QualityIssue {
                issue_type: "Error Reference".to_string(),
                description: "Response mentions errors or failures".to_string(),
                severity: Severity::Medium,
            });
        }
        
        Ok(issues)
    }
    
    /// Update quality history with new report
    async fn update_quality_history(&self, report: &QualityReport) -> Result<()> {
        let mut history = self.quality_history.write().await;
        let now = chrono::Utc::now();
        
        // Add to score history
        history.score_history.push((now, report.overall_score));
        
        // Update issue frequencies
        for issue in &report.issues {
            *history.issue_frequencies.entry(issue.issue_type.clone()).or_insert(0) += 1;
        }
        
        // Detect trends if we have enough data
        if history.score_history.len() > 10 {
            self.detect_quality_trends(&mut history);
        }
        
        // Keep only recent history (last 1000 entries)
        if history.score_history.len() > 1000 {
            history.score_history.drain(0..100);
        }
        
        Ok(())
    }
    
    /// Detect quality trends
    fn detect_quality_trends(&self, history: &mut QualityHistory) {
        // Simple trend detection based on recent scores
        let recent_scores: Vec<f64> = history.score_history
            .iter()
            .rev()
            .take(20)
            .map(|(_, score)| *score)
            .collect();
        
        if recent_scores.len() < 5 {
            return;
        }
        
        let avg_recent = recent_scores.iter().sum::<f64>() / recent_scores.len() as f64;
        let avg_older = history.score_history
            .iter()
            .rev()
            .skip(20)
            .take(20)
            .map(|(_, score)| *score)
            .sum::<f64>() / 20.0;
        
        let trend_type = if (avg_recent - avg_older).abs() < 0.05 {
            TrendType::Stable
        } else if avg_recent > avg_older {
            TrendType::Improving
        } else {
            TrendType::Declining
        };
        
        history.trends.push(QualityTrend {
            trend_type,
            description: format!("Quality trend detected: {:?}", trend_type),
            start_time: chrono::Utc::now(),
            strength: (avg_recent - avg_older).abs(),
        });
    }
    
    /// Get quality statistics
    pub async fn get_stats(&self) -> QualityStats {
        let history = self.quality_history.read().await;
        
        let average_score = if history.score_history.is_empty() {
            0.0
        } else {
            history.score_history.iter().map(|(_, s)| s).sum::<f64>() 
                / history.score_history.len() as f64
        };
        
        QualityStats {
            total_analyses: history.score_history.len(),
            average_score,
            common_issues: history.issue_frequencies.clone(),
            current_trend: history.trends.last().map(|t| format!("{:?}", t.trend_type)),
        }
    }
}

/// Quality statistics
#[derive(Debug, Clone)]
pub struct QualityStats {
    pub total_analyses: usize,
    pub average_score: f64,
    pub common_issues: std::collections::HashMap<String, usize>,
    pub current_trend: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_quality_evaluation() {
        // Test quality evaluation logic
    }
}