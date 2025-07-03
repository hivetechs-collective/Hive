//! Integration tests for transformation module

use hive_ai::transformation::*;
use std::path::PathBuf;

#[test]
fn test_transformation_types() {
    // Test that the basic types are accessible
    let request = TransformationRequest {
        file_path: PathBuf::from("test.rs"),
        aspect: "performance".to_string(),
        context: None,
        multi_file: false,
    };
    
    assert_eq!(request.aspect, "performance");
    
    let change = CodeChange {
        file_path: PathBuf::from("test.rs"),
        original_content: "old content".to_string(),
        new_content: "new content".to_string(),
        line_range: (1, 5),
        description: "Test change".to_string(),
        confidence: 0.85,
    };
    
    assert_eq!(change.confidence, 0.85);
}

#[test]
fn test_risk_levels() {
    // Test that risk levels are properly defined
    let low = RiskLevel::Low;
    let medium = RiskLevel::Medium;
    let high = RiskLevel::High;
    
    // Just ensure they can be created and matched
    match low {
        RiskLevel::Low => assert!(true),
        _ => assert!(false),
    }
    
    match medium {
        RiskLevel::Medium => assert!(true),
        _ => assert!(false),
    }
    
    match high {
        RiskLevel::High => assert!(true),
        _ => assert!(false),
    }
}