//! Tests for the transformation engine

use std::path::Path;
use hive_ai::transformation::{TransformationRequest, simple_transform_code};

#[tokio::test]
async fn test_simple_transformation_engine() {
    // Create a temporary test file
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    let test_content = r#"
fn main() {
    let x = 42;
    println!("Hello, world!");
}
"#;
    
    tokio::fs::write(&test_file, test_content).await.unwrap();
    
    // Create transformation request
    let request = TransformationRequest {
        file_path: test_file,
        aspect: "error-handling".to_string(),
        context: None,
        multi_file: false,
    };
    
    // Generate transformation
    let result = simple_transform_code(request).await;
    
    assert!(result.is_ok());
    let preview = result.unwrap();
    
    // Verify transformation was generated
    assert_eq!(preview.transformation.request.aspect, "error-handling");
    assert!(!preview.transformation.changes.is_empty());
    assert_eq!(preview.impact.risk_level, hive_ai::transformation::RiskLevel::Low);
}

#[tokio::test]
async fn test_different_aspects() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    let test_content = "fn main() { println!(\"test\"); }";
    tokio::fs::write(&test_file, test_content).await.unwrap();
    
    let aspects = vec!["error-handling", "performance", "readability", "custom"];
    
    for aspect in aspects {
        let request = TransformationRequest {
            file_path: test_file.clone(),
            aspect: aspect.to_string(),
            context: None,
            multi_file: false,
        };
        
        let result = simple_transform_code(request).await;
        assert!(result.is_ok(), "Failed for aspect: {}", aspect);
        
        let preview = result.unwrap();
        assert_eq!(preview.transformation.request.aspect, aspect);
    }
}