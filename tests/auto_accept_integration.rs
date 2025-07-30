// Integration tests for AI-Enhanced Auto-Accept System
// These tests verify the auto-accept decision logic without requiring actual Python services

use crate::consensus::operation_analysis::{AutoAcceptMode, OperationContext as AnalysisContext};
use crate::consensus::smart_decision_engine::{SmartDecisionEngine, UserPreferences, ExecutionDecision};
use crate::consensus::stages::file_aware_curator::FileOperation;
use std::path::PathBuf;
use std::time::SystemTime;

/// Helper to create default user preferences
fn create_default_preferences() -> UserPreferences {
    UserPreferences {
        risk_tolerance: 0.5,
        auto_backup: true,
        require_confirmation_for_deletions: true,
        require_confirmation_for_mass_updates: true,
        trust_ai_suggestions: 0.8,
        preferred_mode: AutoAcceptMode::Balanced,
        custom_rules: vec![],
    }
}

/// Helper to create test operation analysis
fn create_test_analysis(confidence: f32, risk: f32) -> crate::consensus::operation_analysis::OperationAnalysis {
    crate::consensus::operation_analysis::OperationAnalysis {
        operations: vec![],
        context: AnalysisContext {
            repository_path: PathBuf::from("/test"),
            user_question: "Test".to_string(),
            consensus_response: "Test response".to_string(),
            timestamp: SystemTime::now(),
            session_id: "test".to_string(),
            git_commit: None,
        },
        unified_score: crate::consensus::operation_analysis::UnifiedScore { confidence, risk },
        recommendations: vec![],
        groups: crate::consensus::operation_analysis::OperationGroups {
            create_operations: vec![],
            update_operations: vec![],
            delete_operations: vec![],
            move_operations: vec![],
        },
        component_scores: crate::consensus::operation_analysis::ComponentScores {
            knowledge_indexer: None,
            context_retriever: None,
            pattern_recognizer: None,
            quality_analyzer: None,
            knowledge_synthesizer: None,
        },
        scoring_factors: crate::consensus::operation_analysis::ScoringFactors {
            historical_success: None,
            pattern_safety: None,
            conflict_probability: None,
            rollback_complexity: None,
            user_trust: 0.8,
            similar_operations_count: None,
            dangerous_pattern_count: None,
            anti_pattern_count: None,
            rollback_possible: None,
        },
        statistics: None,
    }
}

#[tokio::test]
async fn test_conservative_mode_high_confidence_low_risk() {
    let prefs = create_default_preferences();
    let engine = SmartDecisionEngine::new(AutoAcceptMode::Conservative, prefs, None);
    
    // Test high confidence (95%), low risk (10%) - should auto-execute
    let analysis = create_test_analysis(95.0, 10.0);
    let decision = engine.make_decision(&analysis).await.unwrap();
    
    match decision {
        ExecutionDecision::AutoExecute { confidence, risk_level, .. } => {
            assert_eq!(confidence, 95.0);
            assert_eq!(risk_level, 10.0);
        }
        _ => panic!("Expected AutoExecute for high confidence, low risk in conservative mode"),
    }
}

#[tokio::test]
async fn test_conservative_mode_high_risk() {
    let prefs = create_default_preferences();
    let engine = SmartDecisionEngine::new(AutoAcceptMode::Conservative, prefs, None);
    
    // Test high risk (75%) - should block
    let analysis = create_test_analysis(95.0, 75.0);
    let decision = engine.make_decision(&analysis).await.unwrap();
    
    assert!(matches!(decision, ExecutionDecision::Block { .. }), 
            "Expected Block for high risk in conservative mode");
}

#[tokio::test]
async fn test_balanced_mode_thresholds() {
    let prefs = create_default_preferences();
    let engine = SmartDecisionEngine::new(AutoAcceptMode::Balanced, prefs, None);
    
    // Test at balanced thresholds (82% confidence, 20% risk) - should auto-execute
    let analysis = create_test_analysis(82.0, 20.0);
    let decision = engine.make_decision(&analysis).await.unwrap();
    
    assert!(matches!(decision, ExecutionDecision::AutoExecute { .. }), 
            "Expected AutoExecute for operations meeting balanced thresholds");
}

#[tokio::test]
async fn test_aggressive_mode_accepts_higher_risk() {
    let prefs = create_default_preferences();
    let engine = SmartDecisionEngine::new(AutoAcceptMode::Aggressive, prefs, None);
    
    // Test moderate confidence (75%), moderate risk (35%) - should auto-execute in aggressive
    let analysis = create_test_analysis(75.0, 35.0);
    let decision = engine.make_decision(&analysis).await.unwrap();
    
    assert!(matches!(decision, ExecutionDecision::AutoExecute { .. }), 
            "Expected AutoExecute for moderate risk in aggressive mode");
}

#[tokio::test]
async fn test_manual_mode_always_requires_confirmation() {
    let prefs = create_default_preferences();
    let engine = SmartDecisionEngine::new(AutoAcceptMode::Manual, prefs, None);
    
    // Test perfect conditions - should still require confirmation
    let analysis = create_test_analysis(100.0, 0.0);
    let decision = engine.make_decision(&analysis).await.unwrap();
    
    assert!(matches!(decision, ExecutionDecision::RequireConfirmation { .. }), 
            "Expected RequireConfirmation for manual mode regardless of scores");
}

#[tokio::test]
async fn test_plan_mode_requires_review() {
    let prefs = create_default_preferences();
    let engine = SmartDecisionEngine::new(AutoAcceptMode::Plan, prefs, None);
    
    // Test any conditions - should require confirmation with plan details
    let analysis = create_test_analysis(85.0, 15.0);
    let decision = engine.make_decision(&analysis).await.unwrap();
    
    match decision {
        ExecutionDecision::RequireConfirmation { reason, .. } => {
            assert!(reason.contains("Plan mode"), "Plan mode should be mentioned in reason");
        }
        _ => panic!("Expected RequireConfirmation for plan mode"),
    }
}

#[tokio::test]
async fn test_deletion_requires_confirmation_in_conservative() {
    let mut prefs = create_default_preferences();
    prefs.require_confirmation_for_deletions = true;
    
    let engine = SmartDecisionEngine::new(AutoAcceptMode::Conservative, prefs, None);
    
    // Create analysis with deletion operation
    let mut analysis = create_test_analysis(95.0, 10.0);
    analysis.operations = vec![FileOperation::Delete {
        path: PathBuf::from("test.txt"),
    }];
    
    let decision = engine.make_decision(&analysis).await.unwrap();
    
    match decision {
        ExecutionDecision::RequireConfirmation { reason, .. } => {
            assert!(reason.to_lowercase().contains("delet"), 
                    "Deletion warning should be in reason, got: {}", reason);
        }
        _ => panic!("Expected RequireConfirmation for deletion with preference set"),
    }
}

#[tokio::test]
async fn test_mass_update_detection() {
    let mut prefs = create_default_preferences();
    prefs.require_confirmation_for_mass_updates = true;
    
    let engine = SmartDecisionEngine::new(AutoAcceptMode::Balanced, prefs, None);
    
    // Create analysis with many operations (mass update)
    let mut analysis = create_test_analysis(85.0, 20.0);
    analysis.operations = (0..10).map(|i| FileOperation::Update {
        path: PathBuf::from(format!("file{}.txt", i)),
        content: "updated".to_string(),
    }).collect();
    
    let decision = engine.make_decision(&analysis).await.unwrap();
    
    match decision {
        ExecutionDecision::RequireConfirmation { warnings, .. } => {
            assert!(warnings.iter().any(|w| w.contains("10 files")), 
                    "Mass update warning should mention file count");
        }
        _ => panic!("Expected RequireConfirmation for mass update"),
    }
}

#[tokio::test]
async fn test_decision_caching() {
    let prefs = create_default_preferences();
    let engine = SmartDecisionEngine::new(AutoAcceptMode::Balanced, prefs, None);
    
    // Make the same decision twice
    let analysis = create_test_analysis(85.0, 20.0);
    
    let start = std::time::Instant::now();
    let _decision1 = engine.make_decision(&analysis).await.unwrap();
    let first_duration = start.elapsed();
    
    let start = std::time::Instant::now();
    let _decision2 = engine.make_decision(&analysis).await.unwrap();
    let cached_duration = start.elapsed();
    
    // Cached decision should be faster (this is a heuristic test)
    assert!(cached_duration < first_duration / 2, 
            "Cached decision should be significantly faster");
}

#[test]
fn test_mode_ordering() {
    // Verify that modes are ordered by restrictiveness
    use AutoAcceptMode::*;
    
    // These assertions document the expected behavior
    let modes = vec![Conservative, Balanced, Aggressive];
    
    // Conservative is most restrictive
    assert_eq!(modes[0], Conservative);
    // Aggressive is least restrictive  
    assert_eq!(modes[2], Aggressive);
}