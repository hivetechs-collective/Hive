// Intelligent User Feedback System - AI-powered explanations and suggestions
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::operation_analysis::{
    OperationAnalysis, UnifiedScore, ActionRecommendation, ComponentScores, 
    ScoringFactors, OperationContext, ActionPriority
};
use crate::consensus::smart_decision_engine::{ExecutionDecision, UserPreferences};
use crate::consensus::operation_clustering::{OperationCluster, ClusterType};
use crate::ai_helpers::knowledge_synthesizer::KnowledgeSynthesizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub decision_summary: String,
    pub risk_explanation: RiskExplanation,
    pub confidence_explanation: ConfidenceExplanation,
    pub ai_insights: Vec<AIInsight>,
    pub suggestions: Vec<Suggestion>,
    pub operation_preview: Option<OperationPreview>,
    pub learning_notes: Vec<String>,
    pub visual_indicators: VisualIndicators,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskExplanation {
    pub risk_level: String, // "Low", "Medium", "High", "Critical"
    pub main_risk_factors: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub worst_case_scenario: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceExplanation {
    pub confidence_level: String, // "Very High", "High", "Moderate", "Low"
    pub supporting_evidence: Vec<String>,
    pub uncertainty_factors: Vec<String>,
    pub similar_operations_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInsight {
    pub source: String, // Which AI helper provided this
    pub insight_type: InsightType,
    pub description: String,
    pub importance: InsightImportance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    Pattern,
    Warning,
    Suggestion,
    Historical,
    Technical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightImportance {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub title: String,
    pub description: String,
    pub action_type: SuggestionAction,
    pub expected_outcome: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionAction {
    ModifyOperation,
    AddSafetyCheck,
    CreateBackup,
    RunTests,
    ReviewManually,
    ConsultDocumentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPreview {
    pub affected_files: Vec<FilePreview>,
    pub execution_plan: Vec<String>,
    pub estimated_impact: ImpactAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub path: String,
    pub operation_type: String,
    pub preview_snippet: Option<String>,
    pub line_changes: Option<LineChanges>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineChanges {
    pub additions: usize,
    pub deletions: usize,
    pub modifications: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub scope: String, // "Local", "Module", "Application-wide"
    pub affected_functionality: Vec<String>,
    pub dependency_count: usize,
    pub test_coverage: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualIndicators {
    pub primary_color: String, // For UI display
    pub icon_type: String,
    pub animation_style: String,
    pub urgency_level: u8, // 0-10
}

pub struct IntelligentFeedbackGenerator {
    knowledge_synthesizer: Option<Arc<KnowledgeSynthesizer>>,
    feedback_templates: Arc<RwLock<HashMap<String, FeedbackTemplate>>>,
    user_preferences: Arc<RwLock<UserPreferences>>,
    feedback_history: Arc<RwLock<Vec<GeneratedFeedback>>>,
}

#[derive(Debug, Clone)]
struct FeedbackTemplate {
    template_type: String,
    base_message: String,
    variables: Vec<String>,
}

#[derive(Debug, Clone)]
struct GeneratedFeedback {
    timestamp: DateTime<Utc>,
    operation_id: String,
    feedback: UserFeedback,
    user_response: Option<String>,
}

impl IntelligentFeedbackGenerator {
    pub fn new(
        knowledge_synthesizer: Option<Arc<KnowledgeSynthesizer>>,
        user_preferences: UserPreferences,
    ) -> Self {
        let mut templates = HashMap::new();
        
        // Initialize common feedback templates
        templates.insert("high_confidence_safe".to_string(), FeedbackTemplate {
            template_type: "positive".to_string(),
            base_message: "This operation is safe to execute based on {evidence_count} supporting factors.".to_string(),
            variables: vec!["evidence_count".to_string()],
        });
        
        templates.insert("high_risk_blocked".to_string(), FeedbackTemplate {
            template_type: "warning".to_string(),
            base_message: "This operation has been blocked due to {risk_count} critical risk factors.".to_string(),
            variables: vec!["risk_count".to_string()],
        });

        Self {
            knowledge_synthesizer,
            feedback_templates: Arc::new(RwLock::new(templates)),
            user_preferences: Arc::new(RwLock::new(user_preferences)),
            feedback_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn generate_feedback(
        &self,
        analysis: &OperationAnalysis,
        decision: &ExecutionDecision,
        cluster: Option<&OperationCluster>,
    ) -> Result<UserFeedback> {
        let decision_summary = self.create_decision_summary(decision, &analysis.unified_score)?;
        let risk_explanation = self.create_risk_explanation(analysis)?;
        let confidence_explanation = self.create_confidence_explanation(analysis)?;
        let ai_insights = self.extract_ai_insights(analysis)?;
        let suggestions = self.generate_suggestions(analysis, decision)?;
        let operation_preview = self.create_operation_preview(analysis, cluster).await?;
        let learning_notes = self.generate_learning_notes(analysis)?;
        let visual_indicators = self.determine_visual_indicators(decision, &analysis.unified_score)?;

        let feedback = UserFeedback {
            decision_summary,
            risk_explanation,
            confidence_explanation,
            ai_insights,
            suggestions,
            operation_preview,
            learning_notes,
            visual_indicators,
        };

        // Store in history
        self.store_feedback_history(analysis, &feedback).await;

        Ok(feedback)
    }

    fn create_decision_summary(&self, decision: &ExecutionDecision, score: &UnifiedScore) -> Result<String> {
        let summary = match decision {
            ExecutionDecision::AutoExecute { reason, .. } => {
                format!(
                    "✅ **Auto-executing operation**\n\n{}\n\n\
                    **Confidence**: {:.0}% | **Risk**: {:.0}%",
                    reason, score.confidence, score.risk
                )
            }
            ExecutionDecision::RequireConfirmation { reason, warnings, .. } => {
                let warning_text = if warnings.is_empty() {
                    String::new()
                } else {
                    format!("\n\n**Warnings**:\n{}", warnings.iter()
                        .map(|w| format!("⚠️  {}", w))
                        .collect::<Vec<_>>()
                        .join("\n"))
                };
                
                format!(
                    "🤔 **Confirmation required**\n\n{}{}\n\n\
                    **Confidence**: {:.0}% | **Risk**: {:.0}%",
                    reason, warning_text, score.confidence, score.risk
                )
            }
            ExecutionDecision::Block { reason, critical_issues, alternatives, .. } => {
                let issues_text = critical_issues.iter()
                    .map(|i| format!("🚨 {}", i))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                let alternatives_text = if alternatives.is_empty() {
                    String::new()
                } else {
                    format!("\n\n**Alternatives**:\n{}", alternatives.iter()
                        .enumerate()
                        .map(|(i, a)| format!("{}. {}", i + 1, a))
                        .collect::<Vec<_>>()
                        .join("\n"))
                };
                
                format!(
                    "🛑 **Operation blocked for safety**\n\n{}\n\n\
                    **Critical Issues**:\n{}{}\n\n\
                    **Risk Level**: {:.0}%",
                    reason, issues_text, alternatives_text, score.risk
                )
            }
        };

        Ok(summary)
    }

    fn create_risk_explanation(&self, analysis: &OperationAnalysis) -> Result<RiskExplanation> {
        let risk = analysis.unified_score.risk;
        
        let risk_level = match risk {
            r if r < 20.0 => "Low",
            r if r < 40.0 => "Medium",
            r if r < 70.0 => "High",
            _ => "Critical",
        }.to_string();

        let mut main_risk_factors = Vec::new();
        let mut mitigation_strategies = Vec::new();

        // Analyze component scores for risk factors
        if let Some(pattern_score) = analysis.component_scores.pattern_recognizer {
            if pattern_score.safety_score < 70.0 {
                main_risk_factors.push(format!(
                    "Dangerous patterns detected (safety score: {:.0}%)",
                    pattern_score.safety_score
                ));
                mitigation_strategies.push("Review and modify operations to avoid anti-patterns".to_string());
            }
        }

        if let Some(quality_score) = analysis.component_scores.quality_analyzer {
            if quality_score.conflict_probability > 30.0 {
                main_risk_factors.push(format!(
                    "High conflict probability ({:.0}%)",
                    quality_score.conflict_probability
                ));
                mitigation_strategies.push("Check for concurrent modifications or dependencies".to_string());
            }
            
            if quality_score.rollback_complexity > 50.0 {
                main_risk_factors.push("Complex rollback scenario detected".to_string());
                mitigation_strategies.push("Create comprehensive backups before proceeding".to_string());
            }
        }

        // Check for deletions
        let has_deletions = analysis.operations.iter()
            .any(|op| matches!(op, FileOperation::Delete { .. }));
        if has_deletions {
            main_risk_factors.push("Operation includes file deletions".to_string());
            mitigation_strategies.push("Verify files are no longer needed and create backups".to_string());
        }

        // Determine worst case scenario
        let worst_case_scenario = if risk > 70.0 {
            Some("Complete feature breakage requiring manual recovery".to_string())
        } else if risk > 40.0 {
            Some("Partial functionality loss until issues are resolved".to_string())
        } else if has_deletions {
            Some("Accidental data loss if wrong files are deleted".to_string())
        } else {
            None
        };

        Ok(RiskExplanation {
            risk_level,
            main_risk_factors,
            mitigation_strategies,
            worst_case_scenario,
        })
    }

    fn create_confidence_explanation(&self, analysis: &OperationAnalysis) -> Result<ConfidenceExplanation> {
        let confidence = analysis.unified_score.confidence;
        
        let confidence_level = match confidence {
            c if c >= 90.0 => "Very High",
            c if c >= 75.0 => "High",
            c if c >= 50.0 => "Moderate",
            _ => "Low",
        }.to_string();

        let mut supporting_evidence = Vec::new();
        let mut uncertainty_factors = Vec::new();

        // Gather supporting evidence
        if let Some(historical) = analysis.component_scores.knowledge_indexer {
            if historical.prediction_confidence > 80.0 {
                supporting_evidence.push(format!(
                    "Strong historical precedent ({} similar operations)",
                    analysis.scoring_factors.similar_operations_count.unwrap_or(0)
                ));
            }
        }

        if let Some(context) = analysis.component_scores.context_retriever {
            if context.precedent_strength > 0.7 {
                supporting_evidence.push("Matches established patterns in codebase".to_string());
            }
        }

        if analysis.scoring_factors.rollback_possible.unwrap_or(false) {
            supporting_evidence.push("Rollback is possible if issues occur".to_string());
        }

        // Identify uncertainty factors
        if confidence < 70.0 {
            uncertainty_factors.push("Limited historical data for this operation type".to_string());
        }

        if analysis.scoring_factors.conflict_probability.unwrap_or(0.0) > 20.0 {
            uncertainty_factors.push("Potential conflicts with other code".to_string());
        }

        if analysis.operations.len() > 10 {
            uncertainty_factors.push("Large number of operations increases complexity".to_string());
        }

        let similar_operations_summary = if analysis.scoring_factors.similar_operations_count.unwrap_or(0) > 0 {
            Some(format!(
                "Found {} similar operations with {:.0}% success rate",
                analysis.scoring_factors.similar_operations_count.unwrap_or(0),
                analysis.scoring_factors.historical_success.unwrap_or(0.0) * 100.0
            ))
        } else {
            None
        };

        Ok(ConfidenceExplanation {
            confidence_level,
            supporting_evidence,
            uncertainty_factors,
            similar_operations_summary,
        })
    }

    fn extract_ai_insights(&self, analysis: &OperationAnalysis) -> Result<Vec<AIInsight>> {
        let mut insights = Vec::new();

        // Extract insights from recommendations
        for recommendation in &analysis.recommendations {
            let (insight_type, importance) = match recommendation.priority {
                ActionPriority::Critical => {
                    (InsightType::Warning, InsightImportance::Critical)
                }
                ActionPriority::High => {
                    (InsightType::Warning, InsightImportance::High)
                }
                ActionPriority::Medium => {
                    (InsightType::Suggestion, InsightImportance::Medium)
                }
                ActionPriority::Low => {
                    (InsightType::Suggestion, InsightImportance::Low)
                }
            };

            insights.push(AIInsight {
                source: recommendation.source.clone(),
                insight_type,
                description: recommendation.description.clone(),
                importance,
            });
        }

        // Add pattern-based insights
        if analysis.scoring_factors.dangerous_pattern_count.unwrap_or(0) > 0 {
            insights.push(AIInsight {
                source: "Pattern Recognizer".to_string(),
                insight_type: InsightType::Pattern,
                description: format!(
                    "Detected {} dangerous patterns that may cause issues",
                    analysis.scoring_factors.dangerous_pattern_count.unwrap_or(0)
                ),
                importance: InsightImportance::High,
            });
        }

        // Add historical insights
        if let Some(success_rate) = analysis.scoring_factors.historical_success {
            if success_rate < 0.7 {
                insights.push(AIInsight {
                    source: "Knowledge Indexer".to_string(),
                    insight_type: InsightType::Historical,
                    description: format!(
                        "Similar operations have only {:.0}% success rate historically",
                        success_rate * 100.0
                    ),
                    importance: InsightImportance::High,
                });
            }
        }

        // Sort by importance
        insights.sort_by_key(|i| match i.importance {
            InsightImportance::Critical => 0,
            InsightImportance::High => 1,
            InsightImportance::Medium => 2,
            InsightImportance::Low => 3,
        });

        Ok(insights)
    }

    fn generate_suggestions(&self, analysis: &OperationAnalysis, decision: &ExecutionDecision) -> Result<Vec<Suggestion>> {
        let mut suggestions = Vec::new();

        match decision {
            ExecutionDecision::RequireConfirmation { suggestions: decision_suggestions, .. } => {
                // Convert decision suggestions to our format
                for suggestion_text in decision_suggestions {
                    suggestions.push(Suggestion {
                        title: "Recommended Action".to_string(),
                        description: suggestion_text.clone(),
                        action_type: SuggestionAction::ReviewManually,
                        expected_outcome: "Reduced risk through manual verification".to_string(),
                        confidence: 0.8,
                    });
                }
            }
            ExecutionDecision::Block { alternatives, .. } => {
                // Convert alternatives to suggestions
                for alternative in alternatives {
                    suggestions.push(Suggestion {
                        title: "Alternative Approach".to_string(),
                        description: alternative.clone(),
                        action_type: SuggestionAction::ModifyOperation,
                        expected_outcome: "Safer execution path".to_string(),
                        confidence: 0.9,
                    });
                }
            }
            _ => {}
        }

        // Add general suggestions based on analysis
        if analysis.unified_score.risk > 50.0 {
            suggestions.push(Suggestion {
                title: "Create Backup".to_string(),
                description: "Create a backup of affected files before proceeding".to_string(),
                action_type: SuggestionAction::CreateBackup,
                expected_outcome: "Easy recovery if issues occur".to_string(),
                confidence: 0.95,
            });
        }

        let has_tests = analysis.operations.iter()
            .any(|op| {
                if let FileOperation::Create { path, .. } | FileOperation::Update { path, .. } = op {
                    path.to_string_lossy().contains("test")
                } else {
                    false
                }
            });

        if !has_tests && analysis.operations.len() > 3 {
            suggestions.push(Suggestion {
                title: "Add Tests".to_string(),
                description: "Consider adding tests for the modified functionality".to_string(),
                action_type: SuggestionAction::RunTests,
                expected_outcome: "Increased confidence in changes".to_string(),
                confidence: 0.85,
            });
        }

        Ok(suggestions)
    }

    async fn create_operation_preview(
        &self,
        analysis: &OperationAnalysis,
        cluster: Option<&OperationCluster>,
    ) -> Result<Option<OperationPreview>> {
        if analysis.operations.is_empty() {
            return Ok(None);
        }

        let mut affected_files = Vec::new();
        let mut execution_plan = Vec::new();
        let mut affected_functionality = Vec::new();

        // Create file previews
        for (i, operation) in analysis.operations.iter().enumerate() {
            let (path, op_type, preview) = match operation {
                FileOperation::Create { path, content } => {
                    let preview = if content.len() > 100 {
                        format!("{}...", &content[..100])
                    } else {
                        content.clone()
                    };
                    (path.clone(), "Create", Some(preview))
                }
                FileOperation::Update { path, new_content, .. } => {
                    let preview = if new_content.len() > 100 {
                        format!("{}...", &new_content[..100])
                    } else {
                        new_content.clone()
                    };
                    (path.clone(), "Update", Some(preview))
                }
                FileOperation::Delete { path } => {
                    (path.clone(), "Delete", None)
                }
                FileOperation::Rename { old_path, new_path } => {
                    (old_path.clone(), &format!("Rename to {}", new_path.display()), None)
                }
                FileOperation::Move { source, destination } => {
                    (source.clone(), &format!("Move to {}", destination.display()), None)
                }
            };

            affected_files.push(FilePreview {
                path: path.to_string_lossy().to_string(),
                operation_type: op_type.to_string(),
                preview_snippet: preview,
                line_changes: None, // Could be calculated with more sophisticated analysis
            });

            execution_plan.push(format!("{}. {} {}", i + 1, op_type, path.display()));
        }

        // Determine scope based on cluster or operations
        let scope = if let Some(cluster) = cluster {
            match cluster.cluster_type {
                ClusterType::ModuleUpdate => "Module",
                ClusterType::FeatureAddition => "Feature",
                ClusterType::Refactoring => "Application-wide",
                _ => "Local",
            }
        } else if analysis.operations.len() > 5 {
            "Module"
        } else {
            "Local"
        }.to_string();

        // Extract affected functionality from paths
        for file in &affected_files {
            if file.path.contains("auth") {
                affected_functionality.push("Authentication".to_string());
            } else if file.path.contains("api") {
                affected_functionality.push("API endpoints".to_string());
            } else if file.path.contains("database") || file.path.contains("db") {
                affected_functionality.push("Database operations".to_string());
            } else if file.path.contains("ui") || file.path.contains("view") {
                affected_functionality.push("User interface".to_string());
            }
        }

        let impact_assessment = ImpactAssessment {
            scope,
            affected_functionality,
            dependency_count: analysis.operations.len() * 2, // Rough estimate
            test_coverage: None, // Would need actual test analysis
        };

        Ok(Some(OperationPreview {
            affected_files,
            execution_plan,
            estimated_impact: impact_assessment,
        }))
    }

    fn generate_learning_notes(&self, analysis: &OperationAnalysis) -> Result<Vec<String>> {
        let mut notes = Vec::new();

        // Note about confidence vs risk relationship
        if analysis.unified_score.confidence > 80.0 && analysis.unified_score.risk > 30.0 {
            notes.push(
                "💡 High confidence doesn't always mean low risk - \
                operations can be predictable but still dangerous".to_string()
            );
        }

        // Note about similar operations
        if let Some(count) = analysis.scoring_factors.similar_operations_count {
            if count > 10 {
                notes.push(format!(
                    "📊 This type of operation has been performed {} times before",
                    count
                ));
            }
        }

        // Note about patterns
        if analysis.scoring_factors.dangerous_pattern_count.unwrap_or(0) > 0 {
            notes.push(
                "⚡ The AI detected patterns that have caused issues in the past".to_string()
            );
        }

        Ok(notes)
    }

    fn determine_visual_indicators(&self, decision: &ExecutionDecision, score: &UnifiedScore) -> Result<VisualIndicators> {
        let (primary_color, icon_type, animation_style, urgency_level) = match decision {
            ExecutionDecision::AutoExecute { .. } => {
                ("green", "check-circle", "pulse-once", 2)
            }
            ExecutionDecision::RequireConfirmation { .. } => {
                let urgency = if score.risk > 50.0 { 6 } else { 4 };
                ("yellow", "question-circle", "gentle-pulse", urgency)
            }
            ExecutionDecision::Block { .. } => {
                ("red", "shield-exclamation", "shake", 8)
            }
        };

        Ok(VisualIndicators {
            primary_color: primary_color.to_string(),
            icon_type: icon_type.to_string(),
            animation_style: animation_style.to_string(),
            urgency_level,
        })
    }

    async fn store_feedback_history(&self, analysis: &OperationAnalysis, feedback: &UserFeedback) {
        let generated = GeneratedFeedback {
            timestamp: Utc::now(),
            operation_id: format!("{:?}", analysis.operations.first()),
            feedback: feedback.clone(),
            user_response: None,
        };

        self.feedback_history.write().await.push(generated);

        // Keep only last 100 feedback items
        let mut history = self.feedback_history.write().await;
        if history.len() > 100 {
            history.drain(0..history.len() - 100);
        }
    }

    pub async fn format_for_ui(&self, feedback: &UserFeedback) -> String {
        let mut output = Vec::new();

        // Decision summary with visual styling
        output.push(feedback.decision_summary.clone());
        output.push("\n---\n".to_string());

        // Risk section
        output.push(format!("### 🎯 Risk Assessment: **{}**", feedback.risk_explanation.risk_level));
        if !feedback.risk_explanation.main_risk_factors.is_empty() {
            output.push("\n**Risk Factors:**".to_string());
            for factor in &feedback.risk_explanation.main_risk_factors {
                output.push(format!("- {}", factor));
            }
        }
        if !feedback.risk_explanation.mitigation_strategies.is_empty() {
            output.push("\n**Mitigation Strategies:**".to_string());
            for strategy in &feedback.risk_explanation.mitigation_strategies {
                output.push(format!("- ✓ {}", strategy));
            }
        }

        // Confidence section
        output.push(format!("\n### 📊 Confidence Level: **{}**", feedback.confidence_explanation.confidence_level));
        if !feedback.confidence_explanation.supporting_evidence.is_empty() {
            output.push("\n**Supporting Evidence:**".to_string());
            for evidence in &feedback.confidence_explanation.supporting_evidence {
                output.push(format!("- ✅ {}", evidence));
            }
        }

        // AI Insights
        if !feedback.ai_insights.is_empty() {
            output.push("\n### 🤖 AI Insights".to_string());
            for insight in &feedback.ai_insights {
                let icon = match insight.importance {
                    InsightImportance::Critical => "🚨",
                    InsightImportance::High => "⚠️",
                    InsightImportance::Medium => "ℹ️",
                    InsightImportance::Low => "💡",
                };
                output.push(format!("{} **{}**: {}", icon, insight.source, insight.description));
            }
        }

        // Suggestions
        if !feedback.suggestions.is_empty() {
            output.push("\n### 💡 Suggestions".to_string());
            for (i, suggestion) in feedback.suggestions.iter().enumerate() {
                output.push(format!("{}. **{}**\n   {}", i + 1, suggestion.title, suggestion.description));
            }
        }

        // Operation preview
        if let Some(preview) = &feedback.operation_preview {
            output.push("\n### 📁 Operation Preview".to_string());
            output.push(format!("**Scope**: {} | **Files**: {}", 
                preview.estimated_impact.scope, 
                preview.affected_files.len()
            ));
            
            if !preview.execution_plan.is_empty() {
                output.push("\n**Execution Plan:**".to_string());
                for step in &preview.execution_plan {
                    output.push(format!("- {}", step));
                }
            }
        }

        // Learning notes
        if !feedback.learning_notes.is_empty() {
            output.push("\n### 📚 Learning Notes".to_string());
            for note in &feedback.learning_notes {
                output.push(note.clone());
            }
        }

        output.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_feedback_generation() {
        let prefs = UserPreferences {
            risk_tolerance: 0.5,
            auto_backup: true,
            require_confirmation_for_deletions: true,
            require_confirmation_for_mass_updates: true,
            trust_ai_suggestions: 0.8,
            preferred_mode: crate::consensus::AutoAcceptMode::Balanced,
            custom_rules: vec![],
        };

        let generator = IntelligentFeedbackGenerator::new(None, prefs);

        let analysis = create_test_analysis();
        let decision = ExecutionDecision::RequireConfirmation {
            reason: "Test confirmation".to_string(),
            warnings: vec!["Test warning".to_string()],
            suggestions: vec!["Test suggestion".to_string()],
            confidence: 75.0,
            risk_level: 35.0,
        };

        let feedback = generator.generate_feedback(&analysis, &decision, None).await.unwrap();

        assert!(feedback.decision_summary.contains("Confirmation required"));
        assert_eq!(feedback.risk_explanation.risk_level, "Medium");
        assert_eq!(feedback.confidence_explanation.confidence_level, "High");
    }

    fn create_test_analysis() -> OperationAnalysis {
        OperationAnalysis {
            operations: vec![
                FileOperation::Create {
                    path: PathBuf::from("test.rs"),
                    content: "fn main() {}".to_string(),
                }
            ],
            context: OperationContext {
                repository_path: PathBuf::from("/test"),
                user_question: "Test".to_string(),
                consensus_response: "Test response".to_string(),
                timestamp: Utc::now(),
                session_id: "test".to_string(),
            },
            unified_score: UnifiedScore { 
                confidence: 75.0, 
                risk: 35.0 
            },
            recommendations: vec![],
            groups: crate::consensus::operation_intelligence::OperationGroups {
                create_operations: vec![],
                update_operations: vec![],
                delete_operations: vec![],
                move_operations: vec![],
            },
            component_scores: ComponentScores {
                knowledge_indexer: None,
                context_retriever: None,
                pattern_recognizer: None,
                quality_analyzer: None,
                knowledge_synthesizer: None,
            },
            scoring_factors: ScoringFactors {
                historical_success: Some(0.8),
                pattern_safety: Some(0.9),
                conflict_probability: Some(0.1),
                rollback_complexity: Some(0.2),
                user_trust: 0.8,
            },
            statistics: None,
        }
    }
}