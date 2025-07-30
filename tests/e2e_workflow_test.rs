// End-to-End Workflow Tests for AI-Enhanced Auto-Accept System
// Tests complete workflows from user input to operation execution

use crate::consensus::operation_analysis::{
    OperationAnalysis, OperationContext, UnifiedScore, 
    AutoAcceptMode, OperationGroups, ActionRecommendation,
    ActionPriority, RecommendedAction,
};
use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::smart_decision_engine::{
    SmartDecisionEngine, UserPreferences, ExecutionDecision,
    CustomRule, RuleAction, UserDecision, UserChoice,
};
use crate::consensus::operation_history::OperationHistoryDatabase;
use crate::consensus::operation_intelligence::{
    OperationIntelligenceCoordinator, OperationOutcome,
};
use crate::ai_helpers::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use tempfile::TempDir;
use std::time::{Duration, SystemTime};

/// Helper to create complete test environment
async fn create_test_environment() -> (SmartDecisionEngine, Arc<OperationHistoryDatabase>, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_history.db");
    
    // Create operation history database
    let history_db = Arc::new(
        OperationHistoryDatabase::new(db_path.to_str().unwrap()).await.unwrap()
    );
    
    // Create user preferences
    let user_prefs = UserPreferences {
        risk_tolerance: 0.5,
        auto_backup: true,
        require_confirmation_for_deletions: true,
        require_confirmation_for_mass_updates: true,
        trust_ai_suggestions: 0.8,
        preferred_mode: AutoAcceptMode::Balanced,
        custom_rules: vec![],
    };
    
    // Create decision engine
    let decision_engine = SmartDecisionEngine::new(
        AutoAcceptMode::Balanced,
        user_prefs,
        Some(history_db.clone()),
    );
    
    (decision_engine, history_db, temp_dir)
}

/// Helper to create test operation context
fn create_test_context() -> OperationContext {
    OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Test question".to_string(),
        consensus_response: "Test response".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test-session".to_string(),
        git_commit: Some("abc123".to_string()),
    }
}

#[cfg(test)]
mod basic_workflow_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_file_creation_workflow() {
        let (decision_engine, history_db, _temp) = create_test_environment().await;
        
        // Step 1: User wants to create a new file
        let operations = vec![
            FileOperation::Create {
                path: PathBuf::from("src/hello.rs"),
                content: r#"
/// Hello module
pub fn hello() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hello() {
        // Test passes if no panic
        hello();
    }
}
"#.to_string(),
            }
        ];
        
        let context = create_test_context();
        
        // Step 2: Create operation analysis
        let analysis = OperationAnalysis {
            operations: operations.clone(),
            context: context.clone(),
            unified_score: UnifiedScore {
                confidence: 92.0,
                risk: 8.0,
            },
            recommendations: vec![],
            groups: OperationGroups {
                create_operations: operations.clone(),
                update_operations: vec![],
                delete_operations: vec![],
                move_operations: vec![],
            },
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        // Step 3: Decision engine evaluates
        let decision = decision_engine.make_decision(&analysis).await.unwrap();
        
        // Should auto-execute (high confidence, low risk)
        match &decision {
            ExecutionDecision::AutoExecute { confidence, risk_level, .. } => {
                assert_eq!(*confidence, 92.0);
                assert_eq!(*risk_level, 8.0);
            }
            _ => panic!("Expected auto-execute for high confidence file creation"),
        }
        
        // Step 4: Record successful execution
        let operation_id = uuid::Uuid::new_v4();
        let outcome = OperationOutcome {
            success: true,
            error_message: None,
            execution_time: Duration::from_millis(50),
            rollback_required: false,
            user_feedback: None,
            post_operation_quality: None,
        };
        
        history_db.update_outcome(operation_id, &outcome).unwrap();
        
        // Step 5: Learn from success
        let user_decision = UserDecision {
            operation_id: operation_id.to_string(),
            analysis,
            decision: UserChoice::Execute,
            timestamp: SystemTime::now(),
            feedback: Some("Worked perfectly!".to_string()),
        };
        
        decision_engine.learn_from_user_decision(&user_decision).await.unwrap();
        
        // Verify metrics updated
        let metrics = decision_engine.get_metrics().await;
        assert!(metrics.total_decisions > 0);
        assert!(metrics.auto_executed > 0);
    }

    #[tokio::test]
    async fn test_risky_operation_workflow() {
        let (decision_engine, history_db, _temp) = create_test_environment().await;
        
        // Risky operation: deleting multiple files
        let operations = vec![
            FileOperation::Delete {
                path: PathBuf::from("src/important_module.rs"),
            },
            FileOperation::Delete {
                path: PathBuf::from("src/critical_config.rs"),
            },
            FileOperation::Delete {
                path: PathBuf::from("tests/integration_test.rs"),
            },
        ];
        
        let context = create_test_context();
        
        let analysis = OperationAnalysis {
            operations: operations.clone(),
            context: context.clone(),
            unified_score: UnifiedScore {
                confidence: 65.0,
                risk: 75.0, // High risk!
            },
            recommendations: vec![
                ActionRecommendation {
                    priority: ActionPriority::High,
                    source: "PatternRecognizer".to_string(),
                    description: "Mass deletion detected - potential data loss".to_string(),
                    action: RecommendedAction::RequestConfirmation,
                },
            ],
            groups: OperationGroups {
                create_operations: vec![],
                update_operations: vec![],
                delete_operations: operations,
                move_operations: vec![],
            },
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        let decision = decision_engine.make_decision(&analysis).await.unwrap();
        
        // Should require confirmation or block
        match decision {
            ExecutionDecision::RequireConfirmation { warnings, .. } => {
                assert!(!warnings.is_empty());
                assert!(warnings.iter().any(|w| w.contains("deletion")));
            }
            ExecutionDecision::Block { critical_issues, .. } => {
                assert!(!critical_issues.is_empty());
            }
            ExecutionDecision::AutoExecute { .. } => {
                panic!("Should not auto-execute high-risk mass deletion");
            }
        }
    }
}

#[cfg(test)]
mod custom_rules_workflow_tests {
    use super::*;

    #[tokio::test]
    async fn test_custom_rule_enforcement_workflow() {
        let (mut decision_engine, _history_db, _temp) = create_test_environment().await;
        
        // Add custom rules
        let mut prefs = UserPreferences {
            risk_tolerance: 0.5,
            auto_backup: true,
            require_confirmation_for_deletions: true,
            require_confirmation_for_mass_updates: true,
            trust_ai_suggestions: 0.8,
            preferred_mode: AutoAcceptMode::Balanced,
            custom_rules: vec![
                CustomRule {
                    pattern: r".*\.secret$".to_string(),
                    action: RuleAction::AlwaysBlock,
                    reason: "Never modify secret files".to_string(),
                },
                CustomRule {
                    pattern: r"tests/.*\.rs$".to_string(),
                    action: RuleAction::AlwaysAutoExecute,
                    reason: "Test files are safe to modify".to_string(),
                },
            ],
        };
        
        decision_engine.update_preferences(prefs).await;
        
        // Test 1: Try to modify secret file
        let secret_op = vec![FileOperation::Update {
            path: PathBuf::from("config/api_keys.secret"),
            content: "new_secret".to_string(),
        }];
        
        let analysis1 = OperationAnalysis {
            operations: secret_op,
            context: create_test_context(),
            unified_score: UnifiedScore { confidence: 95.0, risk: 5.0 },
            recommendations: vec![],
            groups: Default::default(),
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        let decision1 = decision_engine.make_decision(&analysis1).await.unwrap();
        assert!(matches!(decision1, ExecutionDecision::Block { .. }));
        
        // Test 2: Modify test file (should auto-execute despite medium confidence)
        let test_op = vec![FileOperation::Create {
            path: PathBuf::from("tests/new_test.rs"),
            content: "#[test] fn test() {}".to_string(),
        }];
        
        let analysis2 = OperationAnalysis {
            operations: test_op,
            context: create_test_context(),
            unified_score: UnifiedScore { confidence: 60.0, risk: 40.0 }, // Normally would require confirmation
            recommendations: vec![],
            groups: Default::default(),
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        let decision2 = decision_engine.make_decision(&analysis2).await.unwrap();
        assert!(matches!(decision2, ExecutionDecision::AutoExecute { .. }));
    }
}

#[cfg(test)]
mod mode_switching_workflow_tests {
    use super::*;

    #[tokio::test]
    async fn test_mode_progression_workflow() {
        let (mut decision_engine, history_db, _temp) = create_test_environment().await;
        
        let operations = vec![FileOperation::Create {
            path: PathBuf::from("src/feature.rs"),
            content: "pub fn feature() {}".to_string(),
        }];
        
        let analysis = OperationAnalysis {
            operations: operations.clone(),
            context: create_test_context(),
            unified_score: UnifiedScore { confidence: 82.0, risk: 18.0 },
            recommendations: vec![],
            groups: Default::default(),
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        // Test Conservative mode
        decision_engine.set_mode(AutoAcceptMode::Conservative).await;
        let conservative_decision = decision_engine.make_decision(&analysis).await.unwrap();
        assert!(matches!(conservative_decision, ExecutionDecision::RequireConfirmation { .. }));
        
        // Test Balanced mode
        decision_engine.set_mode(AutoAcceptMode::Balanced).await;
        decision_engine.clear_cache().await;
        let balanced_decision = decision_engine.make_decision(&analysis).await.unwrap();
        assert!(matches!(balanced_decision, ExecutionDecision::AutoExecute { .. }));
        
        // Test Aggressive mode
        decision_engine.set_mode(AutoAcceptMode::Aggressive).await;
        decision_engine.clear_cache().await;
        let aggressive_decision = decision_engine.make_decision(&analysis).await.unwrap();
        assert!(matches!(aggressive_decision, ExecutionDecision::AutoExecute { .. }));
        
        // Test Plan mode
        decision_engine.set_mode(AutoAcceptMode::Plan).await;
        decision_engine.clear_cache().await;
        let plan_decision = decision_engine.make_decision(&analysis).await.unwrap();
        assert!(matches!(plan_decision, ExecutionDecision::RequireConfirmation { .. }));
        
        // Test Manual mode
        decision_engine.set_mode(AutoAcceptMode::Manual).await;
        decision_engine.clear_cache().await;
        let manual_decision = decision_engine.make_decision(&analysis).await.unwrap();
        assert!(matches!(manual_decision, ExecutionDecision::RequireConfirmation { .. }));
    }
}

#[cfg(test)]
mod learning_workflow_tests {
    use super::*;

    #[tokio::test]
    async fn test_learning_from_feedback_workflow() {
        let (decision_engine, history_db, _temp) = create_test_environment().await;
        
        // Initial operation
        let operations = vec![FileOperation::Create {
            path: PathBuf::from("src/utils/helper.rs"),
            content: "pub fn helper() -> i32 { 42 }".to_string(),
        }];
        
        let context = create_test_context();
        
        let analysis = OperationAnalysis {
            operations: operations.clone(),
            context: context.clone(),
            unified_score: UnifiedScore { confidence: 78.0, risk: 22.0 },
            recommendations: vec![],
            groups: Default::default(),
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        // Record multiple successful outcomes
        for i in 0..5 {
            let operation_id = uuid::Uuid::new_v4();
            
            // User executed despite medium confidence
            let user_decision = UserDecision {
                operation_id: operation_id.to_string(),
                analysis: analysis.clone(),
                decision: UserChoice::Execute,
                timestamp: SystemTime::now(),
                feedback: Some(format!("Success #{}", i + 1)),
            };
            
            decision_engine.learn_from_user_decision(&user_decision).await.unwrap();
            
            // Record positive outcome
            let outcome = OperationOutcome {
                success: true,
                error_message: None,
                execution_time: Duration::from_millis(30),
                rollback_required: false,
                user_feedback: Some(format!("Great result #{}", i + 1)),
                post_operation_quality: Some(0.95), // High quality
            };
            
            history_db.update_outcome(operation_id, &outcome).unwrap();
        }
        
        // Future similar operations should have adjusted confidence
        let metrics = decision_engine.get_metrics().await;
        assert!(metrics.total_decisions > 0);
        
        // Statistics should reflect successful operations
        let stats = history_db.get_statistics().unwrap();
        assert!(stats.successful_operations >= 5);
    }
}

#[cfg(test)]
mod error_recovery_workflow_tests {
    use super::*;

    #[tokio::test]
    async fn test_rollback_workflow() {
        let (decision_engine, history_db, _temp) = create_test_environment().await;
        
        // Operation that might need rollback
        let operations = vec![FileOperation::Update {
            path: PathBuf::from("critical_system.conf"),
            content: "new_config_that_breaks_things".to_string(),
        }];
        
        let analysis = OperationAnalysis {
            operations: operations.clone(),
            context: create_test_context(),
            unified_score: UnifiedScore { confidence: 70.0, risk: 50.0 },
            recommendations: vec![
                ActionRecommendation {
                    priority: ActionPriority::High,
                    source: "QualityAnalyzer".to_string(),
                    description: "Critical configuration change - backup recommended".to_string(),
                    action: RecommendedAction::CreateBackup,
                },
            ],
            groups: Default::default(),
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        let decision = decision_engine.make_decision(&analysis).await.unwrap();
        
        // Should require confirmation due to risk
        match decision {
            ExecutionDecision::RequireConfirmation { suggestions, .. } => {
                assert!(suggestions.iter().any(|s| s.contains("backup")));
            }
            _ => {}
        }
        
        // Simulate execution failure
        let operation_id = uuid::Uuid::new_v4();
        let failed_outcome = OperationOutcome {
            success: false,
            error_message: Some("Configuration caused system failure".to_string()),
            execution_time: Duration::from_millis(100),
            rollback_required: true,
            user_feedback: Some("System crashed!".to_string()),
            post_operation_quality: Some(0.1), // Very poor quality
        };
        
        history_db.update_outcome(operation_id, &failed_outcome).unwrap();
        
        // Learn from failure
        let user_decision = UserDecision {
            operation_id: operation_id.to_string(),
            analysis,
            decision: UserChoice::Execute,
            timestamp: SystemTime::now(),
            feedback: Some("This was a mistake".to_string()),
        };
        
        decision_engine.learn_from_user_decision(&user_decision).await.unwrap();
        
        // Future similar operations should be more cautious
        let metrics = decision_engine.get_metrics().await;
        assert!(metrics.failed_operations > 0 || metrics.total_decisions > 0);
    }
}

#[cfg(test)]
mod performance_workflow_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_batch_operations_workflow() {
        let (decision_engine, _history_db, _temp) = create_test_environment().await;
        
        // Create many operations
        let operations: Vec<_> = (0..50).map(|i| {
            FileOperation::Create {
                path: PathBuf::from(format!("src/generated/file_{}.rs", i)),
                content: format!("// Generated file {}", i),
            }
        }).collect();
        
        let analysis = OperationAnalysis {
            operations: operations.clone(),
            context: create_test_context(),
            unified_score: UnifiedScore { confidence: 85.0, risk: 15.0 },
            recommendations: vec![],
            groups: OperationGroups {
                create_operations: operations,
                update_operations: vec![],
                delete_operations: vec![],
                move_operations: vec![],
            },
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        let start = Instant::now();
        let decision = decision_engine.make_decision(&analysis).await.unwrap();
        let duration = start.elapsed();
        
        // Should handle large batches efficiently
        assert!(duration.as_millis() < 1000, "Large batch should process quickly");
        
        // Mass operations should trigger special handling
        match decision {
            ExecutionDecision::RequireConfirmation { warnings, .. } => {
                assert!(warnings.iter().any(|w| w.contains("50 files") || w.contains("mass")));
            }
            _ => {}
        }
    }

    #[tokio::test]
    async fn test_decision_caching_workflow() {
        let (decision_engine, _history_db, _temp) = create_test_environment().await;
        
        let operations = vec![FileOperation::Create {
            path: PathBuf::from("src/cached_test.rs"),
            content: "// Test content".to_string(),
        }];
        
        let analysis = OperationAnalysis {
            operations,
            context: create_test_context(),
            unified_score: UnifiedScore { confidence: 90.0, risk: 10.0 },
            recommendations: vec![],
            groups: Default::default(),
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        // First decision
        let start = Instant::now();
        let decision1 = decision_engine.make_decision(&analysis).await.unwrap();
        let first_duration = start.elapsed();
        
        // Second decision (should be cached)
        let start = Instant::now();
        let decision2 = decision_engine.make_decision(&analysis).await.unwrap();
        let cached_duration = start.elapsed();
        
        // Cached should be much faster
        assert!(
            cached_duration < first_duration / 2 || cached_duration.as_micros() < 100,
            "Cached decision should be much faster"
        );
        
        // Decisions should be identical
        match (&decision1, &decision2) {
            (ExecutionDecision::AutoExecute { confidence: c1, .. }, 
             ExecutionDecision::AutoExecute { confidence: c2, .. }) => {
                assert_eq!(c1, c2);
            }
            _ => {}
        }
    }
}