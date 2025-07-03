//! Tests for the transformation module

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transformation::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_transformation_request() {
        let request = TransformationRequest {
            file_path: PathBuf::from("test.rs"),
            aspect: "performance".to_string(),
            context: Some("Focus on loop optimizations".to_string()),
            multi_file: false,
        };
        
        assert_eq!(request.aspect, "performance");
        assert_eq!(request.file_path.to_str().unwrap(), "test.rs");
    }
    
    #[tokio::test]
    async fn test_code_change() {
        let change = CodeChange {
            file_path: PathBuf::from("test.rs"),
            original_content: "fn main() { println!(\"Hello\"); }".to_string(),
            new_content: "fn main() { println!(\"Hello, World!\"); }".to_string(),
            line_range: (1, 1),
            description: "Update greeting message".to_string(),
            confidence: 0.95,
        };
        
        assert_eq!(change.confidence, 0.95);
        assert!(change.original_content.contains("Hello"));
        assert!(change.new_content.contains("Hello, World!"));
    }
    
    #[tokio::test]
    async fn test_transformation_history() {
        let temp_dir = TempDir::new().unwrap();
        let history = TransformationHistory::new(temp_dir.path()).unwrap();
        
        // Test that history is initially empty
        let transformations = history.get_history(10).await;
        assert_eq!(transformations.len(), 0);
    }
    
    #[tokio::test]
    async fn test_preview_system() {
        let preview_system = PreviewSystem::new();
        
        let transformation = Transformation {
            id: "test-123".to_string(),
            timestamp: chrono::Utc::now(),
            request: TransformationRequest {
                file_path: PathBuf::from("test.rs"),
                aspect: "readability".to_string(),
                context: None,
                multi_file: false,
            },
            changes: vec![],
            description: "Test transformation".to_string(),
            applied: false,
            transaction_id: None,
        };
        
        let preview = preview_system.generate_preview(&transformation).await.unwrap();
        assert_eq!(preview.transformation.id, "test-123");
    }
}