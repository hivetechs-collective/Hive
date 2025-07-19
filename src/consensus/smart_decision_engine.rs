// Smart Decision Engine - Intelligent auto-accept decisions based on AI analysis
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::operation_analysis::{
    OperationAnalysis, AutoAcceptMode, ActionPriority, OperationOutcome
};
use crate::consensus::operation_history::OperationHistoryDatabase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionDecision {
    AutoExecute {
        reason: String,
        confidence: f32,
        risk_level: f32,
    },
    RequireConfirmation {
        reason: String,
        warnings: Vec<String>,
        suggestions: Vec<String>,
        confidence: f32,
        risk_level: f32,
    },
    Block {
        reason: String,
        critical_issues: Vec<String>,
        alternatives: Vec<String>,
        risk_level: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDecision {
    pub operation_id: String,
    pub analysis: OperationAnalysis,
    pub decision: UserChoice,
    pub timestamp: DateTime<Utc>,
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserChoice {
    Execute,
    Skip,
    ModifyAndExecute { modifications: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub risk_tolerance: f32, // 0.0 (very conservative) to 1.0 (very aggressive)
    pub auto_backup: bool,
    pub require_confirmation_for_deletions: bool,
    pub require_confirmation_for_mass_updates: bool,
    pub trust_ai_suggestions: f32, // 0.0 to 1.0
    pub preferred_mode: AutoAcceptMode,
    pub custom_rules: Vec<CustomRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    pub pattern: String, // Regex pattern for file paths
    pub action: RuleAction,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    AlwaysConfirm,
    AlwaysBlock,
    AlwaysAutoExecute,
    RequireBackup,
}

#[derive(Debug, Clone)]
pub struct DecisionMetrics {
    pub total_decisions: u64,
    pub auto_executed: u64,
    pub confirmations_required: u64,
    pub operations_blocked: u64,
    pub user_overrides: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_confidence: f32,
}

pub struct SmartDecisionEngine {
    mode: Arc<RwLock<AutoAcceptMode>>,
    user_preferences: Arc<RwLock<UserPreferences>>,
    history_db: Option<Arc<OperationHistoryDatabase>>,
    decision_cache: Arc<RwLock<HashMap<String, ExecutionDecision>>>,
    metrics: Arc<RwLock<DecisionMetrics>>,
    learning_data: Arc<RwLock<Vec<UserDecision>>>,
}

impl SmartDecisionEngine {
    pub fn new(
        mode: AutoAcceptMode,
        user_preferences: UserPreferences,
        history_db: Option<Arc<OperationHistoryDatabase>>,
    ) -> Self {
        Self {
            mode: Arc::new(RwLock::new(mode)),
            user_preferences: Arc::new(RwLock::new(user_preferences)),
            history_db,
            decision_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(DecisionMetrics {
                total_decisions: 0,
                auto_executed: 0,
                confirmations_required: 0,
                operations_blocked: 0,
                user_overrides: 0,
                successful_operations: 0,
                failed_operations: 0,
                average_confidence: 0.0,
            })),
            learning_data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn make_decision(&self, analysis: &OperationAnalysis) -> Result<ExecutionDecision> {
        let mode = self.mode.read().await.clone();
        let prefs = self.user_preferences.read().await.clone();
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_decisions += 1;
            
            // Update running average of confidence
            let new_avg = (metrics.average_confidence * (metrics.total_decisions - 1) as f32 
                + analysis.unified_score.confidence) / metrics.total_decisions as f32;
            metrics.average_confidence = new_avg;
        }

        // Check cache first
        let cache_key = self.generate_cache_key(&analysis.operations);
        if let Some(cached) = self.decision_cache.read().await.get(&cache_key) {
            debug!("Using cached decision for operations");
            return Ok(cached.clone());
        }

        // Apply custom rules first
        if let Some(rule_decision) = self.apply_custom_rules(&analysis.operations, &prefs).await? {
            self.cache_decision(&cache_key, &rule_decision).await;
            return Ok(rule_decision);
        }

        // Make mode-based decision
        let decision = match mode {
            AutoAcceptMode::Conservative => {
                self.make_conservative_decision(analysis, &prefs).await?
            }
            AutoAcceptMode::Balanced => {
                self.make_balanced_decision(analysis, &prefs).await?
            }
            AutoAcceptMode::Aggressive => {
                self.make_aggressive_decision(analysis, &prefs).await?
            }
            AutoAcceptMode::Plan => {
                // In plan mode, never auto-execute
                ExecutionDecision::RequireConfirmation {
                    reason: "Plan mode active - review operations before execution".to_string(),
                    warnings: analysis.recommendations
                        .iter()
                        .filter(|r| r.priority == ActionPriority::High)
                        .map(|r| r.description.clone())
                        .collect(),
                    suggestions: analysis.recommendations
                        .iter()
                        .filter(|r| r.priority == ActionPriority::Medium)
                        .map(|r| r.description.clone())
                        .collect(),
                    confidence: analysis.unified_score.confidence,
                    risk_level: analysis.unified_score.risk,
                }
            }
            AutoAcceptMode::Manual => {
                // In manual mode, always require confirmation
                ExecutionDecision::RequireConfirmation {
                    reason: "Manual mode - all operations require confirmation".to_string(),
                    warnings: vec![],
                    suggestions: vec![],
                    confidence: analysis.unified_score.confidence,
                    risk_level: analysis.unified_score.risk,
                }
            }
        };

        // Update metrics based on decision
        self.update_metrics_for_decision(&decision).await;
        
        // Cache the decision
        self.cache_decision(&cache_key, &decision).await;
        
        Ok(decision)
    }

    async fn make_conservative_decision(
        &self,
        analysis: &OperationAnalysis,
        prefs: &UserPreferences,
    ) -> Result<ExecutionDecision> {
        let score = &analysis.unified_score;
        
        // Conservative mode: Very high confidence (>90%) and very low risk (<15%)
        if score.confidence > 90.0 && score.risk < 15.0 {
            // Additional checks for conservative mode
            if self.has_dangerous_operations(&analysis.operations) && prefs.require_confirmation_for_deletions {
                return Ok(ExecutionDecision::RequireConfirmation {
                    reason: "Deletion operations always require confirmation in conservative mode".to_string(),
                    warnings: vec!["Operation includes file deletions".to_string()],
                    suggestions: vec!["Consider creating backups first".to_string()],
                    confidence: score.confidence,
                    risk_level: score.risk,
                });
            }

            Ok(ExecutionDecision::AutoExecute {
                reason: format!("High confidence ({:.1}%) with minimal risk ({:.1}%)", 
                    score.confidence, score.risk),
                confidence: score.confidence,
                risk_level: score.risk,
            })
        } else if score.risk > 70.0 {
            Ok(ExecutionDecision::Block {
                reason: format!("Risk too high ({:.1}%) for conservative mode", score.risk),
                critical_issues: self.extract_critical_issues(analysis),
                alternatives: self.suggest_alternatives(analysis),
                risk_level: score.risk,
            })
        } else {
            Ok(ExecutionDecision::RequireConfirmation {
                reason: format!("Confidence ({:.1}%) or risk ({:.1}%) outside conservative thresholds", 
                    score.confidence, score.risk),
                warnings: self.extract_warnings(analysis),
                suggestions: self.extract_suggestions(analysis),
                confidence: score.confidence,
                risk_level: score.risk,
            })
        }
    }

    async fn make_balanced_decision(
        &self,
        analysis: &OperationAnalysis,
        prefs: &UserPreferences,
    ) -> Result<ExecutionDecision> {
        let score = &analysis.unified_score;
        
        // Balanced mode: Good confidence (>80%) and acceptable risk (<25%)
        if score.confidence > 80.0 && score.risk < 25.0 {
            // Check for special cases
            if self.is_mass_update(&analysis.operations) && prefs.require_confirmation_for_mass_updates {
                return Ok(ExecutionDecision::RequireConfirmation {
                    reason: "Mass update operations require confirmation".to_string(),
                    warnings: vec![format!("Operation affects {} files", analysis.operations.len())],
                    suggestions: vec!["Consider executing in smaller batches".to_string()],
                    confidence: score.confidence,
                    risk_level: score.risk,
                });
            }

            // Use historical data to make smarter decisions
            if let Some(history_db) = &self.history_db {
                let similar_success_rate = self.get_similar_operation_success_rate(analysis, history_db).await?;
                
                if similar_success_rate < 0.7 {
                    return Ok(ExecutionDecision::RequireConfirmation {
                        reason: format!("Similar operations have only {:.0}% success rate", 
                            similar_success_rate * 100.0),
                        warnings: vec!["Historical data shows potential issues".to_string()],
                        suggestions: vec!["Review operation details carefully".to_string()],
                        confidence: score.confidence,
                        risk_level: score.risk,
                    });
                }
            }

            Ok(ExecutionDecision::AutoExecute {
                reason: format!("Balanced analysis shows good confidence ({:.1}%) with acceptable risk ({:.1}%)", 
                    score.confidence, score.risk),
                confidence: score.confidence,
                risk_level: score.risk,
            })
        } else if score.risk > 60.0 || score.confidence < 40.0 {
            Ok(ExecutionDecision::Block {
                reason: format!("Risk ({:.1}%) or confidence ({:.1}%) outside acceptable range", 
                    score.risk, score.confidence),
                critical_issues: self.extract_critical_issues(analysis),
                alternatives: self.suggest_alternatives(analysis),
                risk_level: score.risk,
            })
        } else {
            Ok(ExecutionDecision::RequireConfirmation {
                reason: "Analysis shows moderate risk or confidence".to_string(),
                warnings: self.extract_warnings(analysis),
                suggestions: self.extract_suggestions(analysis),
                confidence: score.confidence,
                risk_level: score.risk,
            })
        }
    }

    async fn make_aggressive_decision(
        &self,
        analysis: &OperationAnalysis,
        _prefs: &UserPreferences,
    ) -> Result<ExecutionDecision> {
        let score = &analysis.unified_score;
        
        // Aggressive mode: Lower thresholds (>70% confidence, <40% risk)
        if score.risk > 80.0 {
            // Even aggressive mode blocks very high risk
            Ok(ExecutionDecision::Block {
                reason: format!("Risk ({:.1}%) exceeds aggressive mode limits", score.risk),
                critical_issues: self.extract_critical_issues(analysis),
                alternatives: self.suggest_alternatives(analysis),
                risk_level: score.risk,
            })
        } else if score.confidence > 70.0 && score.risk < 40.0 {
            Ok(ExecutionDecision::AutoExecute {
                reason: format!("Aggressive mode accepts {:.1}% confidence with {:.1}% risk", 
                    score.confidence, score.risk),
                confidence: score.confidence,
                risk_level: score.risk,
            })
        } else {
            Ok(ExecutionDecision::RequireConfirmation {
                reason: "Below aggressive mode thresholds".to_string(),
                warnings: self.extract_warnings(analysis),
                suggestions: self.extract_suggestions(analysis),
                confidence: score.confidence,
                risk_level: score.risk,
            })
        }
    }

    pub async fn learn_from_user_decision(&self, decision: &UserDecision) -> Result<()> {
        // Store the decision
        self.learning_data.write().await.push(decision.clone());
        
        // Update operation history if available
        if let Some(history_db) = &self.history_db {
            let outcome = match &decision.decision {
                UserChoice::Execute => OperationOutcome {
                    success: true,
                    error_message: None,
                    execution_time_ms: 0,
                    rollback_required: false,
                    user_satisfaction: None,
                },
                UserChoice::Skip => OperationOutcome {
                    success: false,
                    error_message: Some("User skipped operation".to_string()),
                    execution_time_ms: 0,
                    rollback_required: false,
                    user_satisfaction: None,
                },
                UserChoice::ModifyAndExecute { .. } => OperationOutcome {
                    success: true,
                    error_message: None,
                    execution_time_ms: 0,
                    rollback_required: false,
                    user_satisfaction: None,
                },
            };
            
            // Record the outcome - need to parse operation_id as UUID
            if let Ok(uuid) = uuid::Uuid::parse_str(&decision.operation_id) {
                history_db.update_outcome(uuid, &outcome)?;
            }
        }
        
        // Analyze patterns in user decisions
        self.analyze_learning_patterns().await?;
        
        Ok(())
    }

    async fn analyze_learning_patterns(&self) -> Result<()> {
        let learning_data = self.learning_data.read().await;
        
        if learning_data.len() < 10 {
            return Ok(()); // Not enough data to learn from
        }
        
        // Analyze user override patterns
        let mut override_patterns = HashMap::new();
        
        for decision in learning_data.iter() {
            let was_auto_execute = matches!(
                self.make_decision(&decision.analysis).await?,
                ExecutionDecision::AutoExecute { .. }
            );
            
            let user_executed = matches!(
                decision.decision,
                UserChoice::Execute | UserChoice::ModifyAndExecute { .. }
            );
            
            if was_auto_execute != user_executed {
                // User overrode our decision
                let pattern_key = format!("{:.0}_{:.0}", 
                    decision.analysis.unified_score.confidence,
                    decision.analysis.unified_score.risk
                );
                
                *override_patterns.entry(pattern_key).or_insert(0) += 1;
            }
        }
        
        // If we see consistent overrides, adjust preferences
        for (pattern, count) in override_patterns {
            if count > 3 {
                info!("Detected user override pattern: {} (count: {})", pattern, count);
                // TODO: Adjust risk tolerance or confidence thresholds based on patterns
            }
        }
        
        Ok(())
    }

    async fn apply_custom_rules(
        &self,
        operations: &[FileOperation],
        prefs: &UserPreferences,
    ) -> Result<Option<ExecutionDecision>> {
        for rule in &prefs.custom_rules {
            let regex = regex::Regex::new(&rule.pattern)?;
            
            for op in operations {
                let path = match op {
                    FileOperation::Create { path, .. } |
                    FileOperation::Update { path, .. } |
                    FileOperation::Append { path, .. } |
                    FileOperation::Delete { path } |
                    FileOperation::Rename { from: path, .. } => path,
                };
                
                if regex.is_match(&path.to_string_lossy()) {
                    return Ok(Some(match rule.action {
                        RuleAction::AlwaysConfirm => ExecutionDecision::RequireConfirmation {
                            reason: format!("Custom rule: {}", rule.reason),
                            warnings: vec![],
                            suggestions: vec![],
                            confidence: 0.0, // Will be filled by analysis
                            risk_level: 0.0,
                        },
                        RuleAction::AlwaysBlock => ExecutionDecision::Block {
                            reason: format!("Custom rule: {}", rule.reason),
                            critical_issues: vec![rule.reason.clone()],
                            alternatives: vec![],
                            risk_level: 100.0,
                        },
                        RuleAction::AlwaysAutoExecute => ExecutionDecision::AutoExecute {
                            reason: format!("Custom rule: {}", rule.reason),
                            confidence: 100.0,
                            risk_level: 0.0,
                        },
                        RuleAction::RequireBackup => ExecutionDecision::RequireConfirmation {
                            reason: format!("Custom rule requires backup: {}", rule.reason),
                            warnings: vec!["Backup required before execution".to_string()],
                            suggestions: vec!["Create backup of affected files".to_string()],
                            confidence: 0.0,
                            risk_level: 0.0,
                        },
                    }));
                }
            }
        }
        
        Ok(None)
    }

    async fn get_similar_operation_success_rate(
        &self,
        _analysis: &OperationAnalysis,
        history_db: &Arc<OperationHistoryDatabase>,
    ) -> Result<f32> {
        // Query historical data for similar operations
        let stats = history_db.get_statistics()?;
        
        // Simple heuristic: if we have enough data, use success rate
        if stats.total_operations > 50 {
            Ok(stats.successful_operations as f32 / stats.total_operations as f32)
        } else {
            // Not enough data, return neutral rate
            Ok(0.8)
        }
    }

    fn has_dangerous_operations(&self, operations: &[FileOperation]) -> bool {
        operations.iter().any(|op| matches!(op, FileOperation::Delete { .. }))
    }

    fn is_mass_update(&self, operations: &[FileOperation]) -> bool {
        operations.len() > 5
    }

    fn extract_critical_issues(&self, analysis: &OperationAnalysis) -> Vec<String> {
        analysis.recommendations
            .iter()
            .filter(|r| r.priority == ActionPriority::Critical)
            .map(|r| r.description.clone())
            .collect()
    }

    fn extract_warnings(&self, analysis: &OperationAnalysis) -> Vec<String> {
        analysis.recommendations
            .iter()
            .filter(|r| r.priority == ActionPriority::High)
            .map(|r| r.description.clone())
            .collect()
    }

    fn extract_suggestions(&self, analysis: &OperationAnalysis) -> Vec<String> {
        analysis.recommendations
            .iter()
            .filter(|r| r.priority == ActionPriority::Medium)
            .map(|r| r.description.clone())
            .collect()
    }

    fn suggest_alternatives(&self, analysis: &OperationAnalysis) -> Vec<String> {
        let mut alternatives = vec![];
        
        // Suggest breaking down large operations
        if analysis.operations.len() > 10 {
            alternatives.push("Break operation into smaller batches".to_string());
        }
        
        // Suggest backups for deletions
        if self.has_dangerous_operations(&analysis.operations) {
            alternatives.push("Create backups before deletion".to_string());
            alternatives.push("Consider archiving instead of deleting".to_string());
        }
        
        // Add AI recommendations marked as alternatives
        alternatives.extend(
            analysis.recommendations
                .iter()
                .filter(|r| r.description.contains("alternative") || r.description.contains("instead"))
                .map(|r| r.description.clone())
        );
        
        alternatives
    }

    fn generate_cache_key(&self, operations: &[FileOperation]) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        for op in operations {
            format!("{:?}", op).hash(&mut hasher);
        }
        
        format!("decision_{:x}", hasher.finish())
    }

    async fn cache_decision(&self, key: &str, decision: &ExecutionDecision) {
        self.decision_cache.write().await.insert(key.to_string(), decision.clone());
    }

    async fn update_metrics_for_decision(&self, decision: &ExecutionDecision) {
        let mut metrics = self.metrics.write().await;
        
        match decision {
            ExecutionDecision::AutoExecute { .. } => metrics.auto_executed += 1,
            ExecutionDecision::RequireConfirmation { .. } => metrics.confirmations_required += 1,
            ExecutionDecision::Block { .. } => metrics.operations_blocked += 1,
        }
    }

    pub async fn get_metrics(&self) -> DecisionMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn set_mode(&self, mode: AutoAcceptMode) {
        *self.mode.write().await = mode;
        info!("Decision engine mode changed to: {:?}", mode);
    }

    pub async fn update_preferences(&self, preferences: UserPreferences) {
        *self.user_preferences.write().await = preferences;
        info!("User preferences updated");
    }

    pub async fn clear_cache(&self) {
        self.decision_cache.write().await.clear();
        debug!("Decision cache cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_conservative_mode_decisions() {
        let prefs = UserPreferences {
            risk_tolerance: 0.1,
            auto_backup: true,
            require_confirmation_for_deletions: true,
            require_confirmation_for_mass_updates: true,
            trust_ai_suggestions: 0.8,
            preferred_mode: AutoAcceptMode::Conservative,
            custom_rules: vec![],
        };

        let engine = SmartDecisionEngine::new(AutoAcceptMode::Conservative, prefs, None);

        // Test high confidence, low risk -> auto-execute
        let analysis = create_test_analysis(95.0, 10.0);
        let decision = engine.make_decision(&analysis).await.unwrap();
        assert!(matches!(decision, ExecutionDecision::AutoExecute { .. }));

        // Test high risk -> block
        let analysis = create_test_analysis(95.0, 75.0);
        let decision = engine.make_decision(&analysis).await.unwrap();
        assert!(matches!(decision, ExecutionDecision::Block { .. }));
    }

    fn create_test_analysis(confidence: f32, risk: f32) -> OperationAnalysis {
        OperationAnalysis {
            operations: vec![],
            context: OperationContext {
                repository_path: PathBuf::from("/test"),
                user_question: "Test".to_string(),
                consensus_response: "Test response".to_string(),
                timestamp: Utc::now(),
                session_id: "test".to_string(),
            },
            unified_score: UnifiedScore { confidence, risk },
            recommendations: vec![],
            groups: crate::consensus::operation_intelligence::OperationGroups {
                create_operations: vec![],
                update_operations: vec![],
                delete_operations: vec![],
                move_operations: vec![],
            },
            component_scores: crate::consensus::operation_intelligence::ComponentScores {
                knowledge_indexer: None,
                context_retriever: None,
                pattern_recognizer: None,
                quality_analyzer: None,
                knowledge_synthesizer: None,
            },
            scoring_factors: crate::consensus::operation_intelligence::ScoringFactors {
                historical_success: None,
                pattern_safety: None,
                conflict_probability: None,
                rollback_complexity: None,
                user_trust: 0.8,
            },
            statistics: None,
        }
    }
}