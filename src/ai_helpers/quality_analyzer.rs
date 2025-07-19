//! Quality Analyzer - Uses CodeT5+ for analyzing and validating Curator outputs
//! 
//! This module evaluates the quality of Curator outputs, detects contradictions,
//! measures confidence, ensures consistency across the knowledge base, and provides
//! comprehensive risk assessment and conflict detection for file operations.

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::ai_helpers::{QualityReport, QualityIssue, Severity, IndexedKnowledge};
use crate::consensus::operation_intelligence::{
    QualityImpact, OperationConflict, ConflictType, RollbackComplexity,
    OperationContext, QualityMetrics as OperationQualityMetrics
};
use crate::consensus::stages::file_aware_curator::FileOperation;
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
    
    /// Risk assessment configuration
    pub risk_threshold: f64,
    pub conflict_detection_enabled: bool,
    pub rollback_complexity_weight: f64,
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
            risk_threshold: 0.7,
            conflict_detection_enabled: true,
            rollback_complexity_weight: 0.2,
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
    
    /// Operation risk assessment cache
    risk_cache: Arc<RwLock<lru::LruCache<String, OperationRiskAssessment>>>,
    
    /// Conflict detection history
    conflict_history: Arc<RwLock<Vec<ConflictRecord>>>,
    
    /// File state tracker for conflict detection
    file_state_tracker: Arc<RwLock<FileStateTracker>>,
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

/// Operation risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRiskAssessment {
    /// Overall risk score (0-100, higher = more risky)
    pub risk_score: f32,
    
    /// Quality impact prediction
    pub quality_impact: QualityImpact,
    
    /// Detected conflicts
    pub conflicts: Vec<OperationConflict>,
    
    /// Risk factors identified
    pub risk_factors: Vec<RiskFactor>,
    
    /// Rollback complexity assessment
    pub rollback_complexity: RollbackComplexity,
    
    /// Risk mitigation suggestions
    pub mitigation_suggestions: Vec<String>,
    
    /// Assessment timestamp
    pub assessed_at: SystemTime,
}

/// Individual risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Type of risk
    pub risk_type: RiskType,
    
    /// Description of the risk
    pub description: String,
    
    /// Severity of the risk
    pub severity: RiskSeverity,
    
    /// Probability of occurrence
    pub probability: f32,
    
    /// Impact if risk materializes
    pub impact: f32,
}

/// Types of risks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    CodeQuality,
    Performance,
    Security,
    DataIntegrity,
    Compatibility,
    Maintainability,
    UserExperience,
    SystemStability,
}

/// Risk severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Conflict record for tracking
#[derive(Debug, Clone)]
struct ConflictRecord {
    conflict: OperationConflict,
    detected_at: SystemTime,
    resolution_status: ResolutionStatus,
    affected_operations: Vec<String>,
}

/// Conflict resolution status
#[derive(Debug, Clone)]
enum ResolutionStatus {
    Unresolved,
    AutoResolved,
    ManuallyResolved,
    Ignored,
}

/// Tracks file states for conflict detection
#[derive(Default)]
struct FileStateTracker {
    /// Current file states (path -> state info)
    file_states: HashMap<PathBuf, FileStateInfo>,
    
    /// Pending operations on files
    pending_operations: HashMap<PathBuf, Vec<PendingOperation>>,
    
    /// File dependency graph
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
}

/// Information about a file's state
#[derive(Debug, Clone)]
struct FileStateInfo {
    /// Last modified time
    last_modified: SystemTime,
    
    /// File hash for change detection
    content_hash: String,
    
    /// Current quality metrics
    quality_metrics: Option<OperationQualityMetrics>,
    
    /// Files that depend on this file
    dependents: HashSet<PathBuf>,
}

/// Pending operation information
#[derive(Debug, Clone)]
struct PendingOperation {
    /// Operation ID
    operation_id: String,
    
    /// Type of operation
    operation: FileOperation,
    
    /// When operation was queued
    queued_at: SystemTime,
    
    /// Priority level
    priority: OperationPriority,
}

/// Operation priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum OperationPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl QualityAnalyzer {
    /// Create a new Quality Analyzer
    pub async fn new(python_service: Arc<PythonModelService>) -> Result<Self> {
        let config = QualityConfig::default();
        let quality_history = Arc::new(RwLock::new(QualityHistory::default()));
        let analysis_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(200).unwrap()
        )));
        let risk_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(100).unwrap()
        )));
        let conflict_history = Arc::new(RwLock::new(Vec::new()));
        let file_state_tracker = Arc::new(RwLock::new(FileStateTracker::default()));
        
        Ok(Self {
            config,
            python_service,
            quality_history,
            analysis_cache,
            risk_cache,
            conflict_history,
            file_state_tracker,
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
    
    /// Analyze text quality for context validation
    pub async fn analyze_text_quality(&self, text: &str, task: &str) -> Result<QualityMetrics> {
        // Create simple indexed knowledge for quality analysis
        let indexed = IndexedKnowledge {
            id: format!("temp_quality_{}", chrono::Utc::now().timestamp()),
            content: text.to_string(),
            embedding: vec![],
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        };
        
        // Use existing quality evaluation
        let quality_report = self.evaluate_quality(&indexed, task).await?;
        
        // Convert to QualityMetrics format
        Ok(QualityMetrics {
            overall_score: quality_report.overall_score,
            clarity: quality_report.consistency_score, // Use consistency as clarity proxy
            complexity: quality_report.completeness_score, // Use completeness as complexity proxy
        })
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

/// Quality metrics for context validation
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub overall_score: f64,
    pub clarity: f64,
    pub complexity: f64,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            overall_score: 0.5,
            clarity: 0.5,
            complexity: 0.5,
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

impl QualityAnalyzer {
    /// Assess risk for file operations
    pub async fn assess_operation_risk(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<OperationRiskAssessment> {
        info!("🔍 Assessing risk for operation: {:?}", operation);
        
        // Check cache first
        let cache_key = self.generate_risk_cache_key(operation, context);
        {
            let cache = self.risk_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                debug!("📋 Using cached risk assessment");
                return Ok(cached.clone());
            }
        }
        
        let start_time = std::time::Instant::now();
        
        // 1. Analyze quality impact
        let quality_impact = self.analyze_quality_impact(operation, context).await?;
        
        // 2. Detect conflicts
        let conflicts = if self.config.conflict_detection_enabled {
            self.detect_operation_conflicts(operation, context).await?
        } else {
            Vec::new()
        };
        
        // 3. Identify risk factors
        let risk_factors = self.identify_risk_factors(operation, context).await?;
        
        // 4. Assess rollback complexity
        let rollback_complexity = self.assess_rollback_complexity(operation, context).await?;
        
        // 5. Calculate overall risk score
        let risk_score = self.calculate_risk_score(
            &quality_impact,
            &conflicts,
            &risk_factors,
            &rollback_complexity,
        ).await?;
        
        // 6. Generate mitigation suggestions
        let mitigation_suggestions = self.generate_mitigation_suggestions(
            &risk_factors,
            &conflicts,
            &rollback_complexity,
        ).await?;
        
        let assessment = OperationRiskAssessment {
            risk_score,
            quality_impact,
            conflicts,
            risk_factors,
            rollback_complexity,
            mitigation_suggestions,
            assessed_at: SystemTime::now(),
        };
        
        // Cache the result
        {
            let mut cache = self.risk_cache.write().await;
            cache.put(cache_key, assessment.clone());
        }
        
        // Record conflict history
        if !conflicts.is_empty() {
            self.record_conflicts(&conflicts).await?;
        }
        
        let assessment_time = start_time.elapsed();
        info!("✅ Risk assessment completed in {:?}", assessment_time);
        
        Ok(assessment)
    }
    
    /// Analyze quality impact of an operation
    async fn analyze_quality_impact(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<QualityImpact> {
        debug!("📊 Analyzing quality impact for operation");
        
        let mut maintainability_impact = 0.0;
        let mut reliability_impact = 0.0;
        let mut performance_impact = 0.0;
        
        // Analyze based on operation type
        match operation {
            FileOperation::Create { path, content } => {
                // Creating new files generally improves structure
                maintainability_impact = 5.0;
                
                // Check for code complexity
                let complexity = self.analyze_code_complexity(content).await?;
                if complexity > 10.0 {
                    maintainability_impact -= 10.0;
                }
                
                // Check for test files
                if path.to_string_lossy().contains("test") {
                    reliability_impact = 10.0;
                }
            }
            FileOperation::Update { path, content } => {
                // Updates can impact all metrics
                let current_quality = self.get_current_file_quality(path).await?;
                let new_quality = self.analyze_code_quality(content).await?;
                
                maintainability_impact = (new_quality.maintainability - current_quality.maintainability) * 100.0;
                reliability_impact = (new_quality.reliability - current_quality.reliability) * 100.0;
                performance_impact = (new_quality.performance - current_quality.performance) * 100.0;
            }
            FileOperation::Delete { path } => {
                // Deletion can reduce complexity but may break dependencies
                maintainability_impact = 5.0;
                reliability_impact = -10.0; // Potential for breaking changes
                
                // Check if it's a test file being deleted
                if path.to_string_lossy().contains("test") {
                    reliability_impact = -20.0;
                }
            }
            FileOperation::Rename { from, to } => {
                // Renames can break imports and references
                maintainability_impact = -5.0;
                reliability_impact = -15.0;
            }
            _ => {}
        }
        
        let overall_impact = (maintainability_impact + reliability_impact + performance_impact) / 3.0;
        
        Ok(QualityImpact {
            overall_impact,
            maintainability_impact,
            reliability_impact,
            performance_impact,
            conflicts: Vec::new(), // Will be filled by conflict detection
            rollback_complexity: RollbackComplexity {
                complexity_score: 0.0,
                rollback_possible: true,
                estimated_rollback_time: Duration::from_secs(0),
                rollback_steps: Vec::new(),
            },
        })
    }
    
    /// Detect conflicts with other operations or existing state
    async fn detect_operation_conflicts(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<Vec<OperationConflict>> {
        debug!("🔍 Detecting conflicts for operation");
        
        let mut conflicts = Vec::new();
        let mut tracker = self.file_state_tracker.write().await;
        
        // Get the target path for this operation
        let target_path = self.get_operation_target_path(operation);
        
        if let Some(path) = target_path {
            // Check for pending operations on the same file
            if let Some(pending_ops) = tracker.pending_operations.get(&path) {
                for pending in pending_ops {
                    let conflict_type = self.determine_conflict_type(operation, &pending.operation);
                    
                    if let Some(conflict_type) = conflict_type {
                        conflicts.push(OperationConflict {
                            conflicting_operations: vec![0, 1], // Placeholder indices
                            conflict_type,
                            description: format!(
                                "Conflict between {:?} and pending {:?}",
                                operation, pending.operation
                            ),
                            resolution: Some(self.suggest_conflict_resolution(&conflict_type)),
                        });
                    }
                }
            }
            
            // Check for dependency violations
            if let Some(deps) = tracker.dependencies.get(&path) {
                for dep in deps {
                    if matches!(operation, FileOperation::Delete { .. }) {
                        conflicts.push(OperationConflict {
                            conflicting_operations: vec![0],
                            conflict_type: ConflictType::DependencyViolation,
                            description: format!(
                                "Deleting {} would break dependency from {}",
                                path.display(), dep.display()
                            ),
                            resolution: Some("Update dependent files first".to_string()),
                        });
                    }
                }
            }
            
            // Check for race conditions with concurrent operations
            if self.detect_race_condition(&tracker, &path, operation).await? {
                conflicts.push(OperationConflict {
                    conflicting_operations: vec![0],
                    conflict_type: ConflictType::RaceCondition,
                    description: "Multiple operations targeting the same file concurrently".to_string(),
                    resolution: Some("Serialize operations or use file locking".to_string()),
                });
            }
        }
        
        Ok(conflicts)
    }
    
    /// Identify risk factors for an operation
    async fn identify_risk_factors(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> Result<Vec<RiskFactor>> {
        let mut risk_factors = Vec::new();
        
        // Code quality risks
        if let Some(content) = self.get_operation_content(operation) {
            let complexity = self.analyze_code_complexity(content).await?;
            if complexity > 20.0 {
                risk_factors.push(RiskFactor {
                    risk_type: RiskType::CodeQuality,
                    description: "High code complexity detected".to_string(),
                    severity: RiskSeverity::High,
                    probability: 0.8,
                    impact: 0.7,
                });
            }
        }
        
        // Security risks
        if self.detect_security_risks(operation).await? {
            risk_factors.push(RiskFactor {
                risk_type: RiskType::Security,
                description: "Potential security vulnerability detected".to_string(),
                severity: RiskSeverity::Critical,
                probability: 0.6,
                impact: 0.9,
            });
        }
        
        // Performance risks
        if self.detect_performance_risks(operation).await? {
            risk_factors.push(RiskFactor {
                risk_type: RiskType::Performance,
                description: "Operation may impact system performance".to_string(),
                severity: RiskSeverity::Medium,
                probability: 0.5,
                impact: 0.6,
            });
        }
        
        // Data integrity risks
        if matches!(operation, FileOperation::Delete { .. } | FileOperation::Update { .. }) {
            risk_factors.push(RiskFactor {
                risk_type: RiskType::DataIntegrity,
                description: "Operation modifies existing data".to_string(),
                severity: RiskSeverity::Medium,
                probability: 0.4,
                impact: 0.7,
            });
        }
        
        // Maintainability risks
        if self.detect_maintainability_risks(operation, context).await? {
            risk_factors.push(RiskFactor {
                risk_type: RiskType::Maintainability,
                description: "Operation may reduce code maintainability".to_string(),
                severity: RiskSeverity::Low,
                probability: 0.6,
                impact: 0.5,
            });
        }
        
        Ok(risk_factors)
    }
    
    /// Assess rollback complexity for an operation
    async fn assess_rollback_complexity(
        &self,
        operation: &FileOperation,
        _context: &OperationContext,
    ) -> Result<RollbackComplexity> {
        let mut complexity_score = 0.0;
        let mut rollback_steps = Vec::new();
        let mut estimated_time = Duration::from_secs(0);
        
        match operation {
            FileOperation::Create { path, .. } => {
                // Simple to rollback - just delete
                complexity_score = 10.0;
                rollback_steps.push(format!("Delete file {}", path.display()));
                estimated_time = Duration::from_secs(1);
            }
            FileOperation::Update { path, .. } => {
                // Requires backup restoration
                complexity_score = 30.0;
                rollback_steps.push(format!("Restore backup of {}", path.display()));
                rollback_steps.push("Verify file integrity".to_string());
                estimated_time = Duration::from_secs(5);
            }
            FileOperation::Delete { path } => {
                // Most complex - requires file recovery
                complexity_score = 50.0;
                rollback_steps.push(format!("Recover deleted file {}", path.display()));
                rollback_steps.push("Restore file permissions and metadata".to_string());
                rollback_steps.push("Rebuild dependencies".to_string());
                estimated_time = Duration::from_secs(30);
            }
            FileOperation::Rename { from, to } => {
                // Medium complexity - rename back
                complexity_score = 20.0;
                rollback_steps.push(format!("Rename {} back to {}", to.display(), from.display()));
                rollback_steps.push("Update all references".to_string());
                estimated_time = Duration::from_secs(10);
            }
            _ => {}
        }
        
        // Adjust complexity based on file importance
        if let Some(path) = self.get_operation_target_path(operation) {
            if self.is_critical_file(&path) {
                complexity_score *= 1.5;
                estimated_time = estimated_time * 2;
            }
        }
        
        Ok(RollbackComplexity {
            complexity_score,
            rollback_possible: complexity_score < 80.0,
            estimated_rollback_time: estimated_time,
            rollback_steps,
        })
    }
    
    /// Calculate overall risk score
    async fn calculate_risk_score(
        &self,
        quality_impact: &QualityImpact,
        conflicts: &[OperationConflict],
        risk_factors: &[RiskFactor],
        rollback_complexity: &RollbackComplexity,
    ) -> Result<f32> {
        let mut risk_score = 0.0;
        
        // Factor in quality impact (negative impact increases risk)
        if quality_impact.overall_impact < 0.0 {
            risk_score += quality_impact.overall_impact.abs() as f32;
        }
        
        // Add risk from conflicts
        risk_score += conflicts.len() as f32 * 15.0;
        
        // Add risk from risk factors
        for factor in risk_factors {
            let factor_score = factor.probability * factor.impact * 
                match factor.severity {
                    RiskSeverity::Low => 10.0,
                    RiskSeverity::Medium => 20.0,
                    RiskSeverity::High => 30.0,
                    RiskSeverity::Critical => 40.0,
                };
            risk_score += factor_score;
        }
        
        // Factor in rollback complexity
        risk_score += rollback_complexity.complexity_score * self.config.rollback_complexity_weight as f32;
        
        Ok(risk_score.clamp(0.0, 100.0))
    }
    
    /// Generate mitigation suggestions
    async fn generate_mitigation_suggestions(
        &self,
        risk_factors: &[RiskFactor],
        conflicts: &[OperationConflict],
        rollback_complexity: &RollbackComplexity,
    ) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();
        
        // Suggestions based on risk factors
        for factor in risk_factors {
            match factor.risk_type {
                RiskType::CodeQuality => {
                    suggestions.push("Refactor code to reduce complexity before proceeding".to_string());
                }
                RiskType::Security => {
                    suggestions.push("Run security audit and fix vulnerabilities first".to_string());
                }
                RiskType::Performance => {
                    suggestions.push("Consider performance testing after operation".to_string());
                }
                RiskType::DataIntegrity => {
                    suggestions.push("Create comprehensive backup before operation".to_string());
                }
                _ => {}
            }
        }
        
        // Suggestions based on conflicts
        if !conflicts.is_empty() {
            suggestions.push("Resolve conflicts before executing operation".to_string());
        }
        
        // Suggestions based on rollback complexity
        if rollback_complexity.complexity_score > 50.0 {
            suggestions.push("Implement automated rollback mechanism".to_string());
            suggestions.push("Test operation in staging environment first".to_string());
        }
        
        // General suggestions
        if risk_factors.len() > 3 {
            suggestions.push("Consider breaking operation into smaller, safer steps".to_string());
        }
        
        Ok(suggestions)
    }
    
    // Helper methods
    
    /// Generate cache key for risk assessment
    fn generate_risk_cache_key(&self, operation: &FileOperation, context: &OperationContext) -> String {
        format!("{:?}_{}", operation, context.source_question.len())
    }
    
    /// Get target path from operation
    fn get_operation_target_path(&self, operation: &FileOperation) -> Option<PathBuf> {
        match operation {
            FileOperation::Create { path, .. } => Some(path.clone()),
            FileOperation::Update { path, .. } => Some(path.clone()),
            FileOperation::Append { path, .. } => Some(path.clone()),
            FileOperation::Delete { path } => Some(path.clone()),
            FileOperation::Rename { to, .. } => Some(to.clone()),
        }
    }
    
    /// Get content from operation
    fn get_operation_content(&self, operation: &FileOperation) -> Option<&str> {
        match operation {
            FileOperation::Create { content, .. } => Some(content),
            FileOperation::Update { content, .. } => Some(content),
            FileOperation::Append { content, .. } => Some(content),
            _ => None,
        }
    }
    
    /// Determine conflict type between operations
    fn determine_conflict_type(&self, op1: &FileOperation, op2: &FileOperation) -> Option<ConflictType> {
        match (op1, op2) {
            (FileOperation::Update { .. }, FileOperation::Update { .. }) => {
                Some(ConflictType::FileOverwrite)
            }
            (FileOperation::Delete { .. }, _) | (_, FileOperation::Delete { .. }) => {
                Some(ConflictType::DependencyViolation)
            }
            (FileOperation::Rename { .. }, _) | (_, FileOperation::Rename { .. }) => {
                Some(ConflictType::LogicalInconsistency)
            }
            _ => None,
        }
    }
    
    /// Suggest resolution for conflict type
    fn suggest_conflict_resolution(&self, conflict_type: &ConflictType) -> String {
        match conflict_type {
            ConflictType::FileOverwrite => "Merge changes or serialize operations".to_string(),
            ConflictType::DependencyViolation => "Update dependencies before proceeding".to_string(),
            ConflictType::RaceCondition => "Use file locking or operation queuing".to_string(),
            ConflictType::LogicalInconsistency => "Review operation sequence for correctness".to_string(),
        }
    }
    
    /// Detect race conditions
    async fn detect_race_condition(
        &self,
        tracker: &FileStateTracker,
        path: &PathBuf,
        _operation: &FileOperation,
    ) -> Result<bool> {
        if let Some(pending_ops) = tracker.pending_operations.get(path) {
            // Check if any operations were queued very recently
            let now = SystemTime::now();
            for op in pending_ops {
                if let Ok(duration) = now.duration_since(op.queued_at) {
                    if duration < Duration::from_millis(100) {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
    
    /// Analyze code complexity
    async fn analyze_code_complexity(&self, content: &str) -> Result<f64> {
        // Simple complexity heuristics
        let lines = content.lines().count();
        let indentation_levels = content.lines()
            .map(|line| line.chars().take_while(|c| c.is_whitespace()).count() / 4)
            .max()
            .unwrap_or(0);
        
        let cyclomatic_complexity = content.matches("if ").count() +
            content.matches("for ").count() +
            content.matches("while ").count() +
            content.matches("match ").count() +
            content.matches("case ").count();
        
        Ok((lines as f64 * 0.1) + (indentation_levels as f64 * 2.0) + (cyclomatic_complexity as f64 * 3.0))
    }
    
    /// Get current file quality metrics
    async fn get_current_file_quality(&self, path: &PathBuf) -> Result<OperationQualityMetrics> {
        let tracker = self.file_state_tracker.read().await;
        
        if let Some(state) = tracker.file_states.get(path) {
            if let Some(metrics) = &state.quality_metrics {
                return Ok(metrics.clone());
            }
        }
        
        // Return default metrics if not found
        Ok(OperationQualityMetrics {
            code_quality: 0.5,
            test_coverage_change: 0.0,
            performance_impact: 0.0,
            maintainability: 0.5,
        })
    }
    
    /// Analyze code quality
    async fn analyze_code_quality(&self, content: &str) -> Result<OperationQualityMetrics> {
        // Use AI model for quality analysis
        let analysis_result = self.python_service
            .analyze_code(
                &self.config.analysis_model,
                content,
                "quality_metrics"
            )
            .await?;
        
        // Extract metrics from analysis
        Ok(OperationQualityMetrics {
            code_quality: analysis_result.get("code_quality")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5) as f32,
            test_coverage_change: 0.0, // Would need test analysis
            performance_impact: analysis_result.get("performance")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            maintainability: analysis_result.get("maintainability")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5) as f32,
        })
    }
    
    /// Detect security risks
    async fn detect_security_risks(&self, operation: &FileOperation) -> Result<bool> {
        if let Some(content) = self.get_operation_content(operation) {
            // Check for common security issues
            let security_patterns = [
                "eval(",
                "exec(",
                "system(",
                "shell_exec(",
                "os.system",
                "subprocess.call",
                "innerHTML",
                "dangerouslySetInnerHTML",
                "password =",
                "api_key =",
            ];
            
            for pattern in &security_patterns {
                if content.contains(pattern) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    
    /// Detect performance risks
    async fn detect_performance_risks(&self, operation: &FileOperation) -> Result<bool> {
        if let Some(content) = self.get_operation_content(operation) {
            // Check for performance anti-patterns
            let perf_patterns = [
                "SELECT * FROM",
                "N+1",
                ".forEach(",
                "async.forEach",
                "while(true)",
                "sleep(",
                "time.sleep",
            ];
            
            for pattern in &perf_patterns {
                if content.contains(pattern) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    
    /// Detect maintainability risks
    async fn detect_maintainability_risks(
        &self,
        operation: &FileOperation,
        _context: &OperationContext,
    ) -> Result<bool> {
        if let Some(content) = self.get_operation_content(operation) {
            // Check for maintainability issues
            let lines = content.lines().collect::<Vec<_>>();
            
            // Long functions
            let mut in_function = false;
            let mut function_lines = 0;
            
            for line in lines {
                if line.contains("fn ") || line.contains("function ") || 
                   line.contains("def ") || line.contains("func ") {
                    in_function = true;
                    function_lines = 0;
                }
                
                if in_function {
                    function_lines += 1;
                    if function_lines > 50 {
                        return Ok(true);
                    }
                }
                
                if line.trim() == "}" || line.trim() == "end" {
                    in_function = false;
                }
            }
            
            // Deep nesting
            let max_indent = content.lines()
                .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
                .max()
                .unwrap_or(0);
            
            if max_indent > 20 {
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    /// Check if file is critical
    fn is_critical_file(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        
        // Critical file patterns
        let critical_patterns = [
            "main.",
            "app.",
            "index.",
            "config.",
            "database.",
            "auth.",
            "security.",
            ".env",
            "package.json",
            "cargo.toml",
            "requirements.",
        ];
        
        critical_patterns.iter().any(|pattern| path_str.contains(pattern))
    }
    
    /// Record conflicts for history tracking
    async fn record_conflicts(&self, conflicts: &[OperationConflict]) -> Result<()> {
        let mut history = self.conflict_history.write().await;
        
        for conflict in conflicts {
            history.push(ConflictRecord {
                conflict: conflict.clone(),
                detected_at: SystemTime::now(),
                resolution_status: ResolutionStatus::Unresolved,
                affected_operations: Vec::new(),
            });
        }
        
        // Keep history size manageable
        if history.len() > 1000 {
            history.drain(0..100);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_quality_evaluation() {
        // Test quality evaluation logic
    }
}