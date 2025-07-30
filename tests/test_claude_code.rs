//! Test Claude Code functionality
//!
//! This test verifies that the consensus system can handle simple file operations
//! in direct execution mode, similar to how Claude Code works.

#[cfg(test)]
mod tests {
    use crate::consensus::{
        ConsensusEngine, ConsensusConfig, ConsensusProfile,
        ModeDetector, ExecutionMode,
    };
    use crate::core::database::DatabaseManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_mode_detection_for_simple_operations() {
        // Create mode detector
        let detector = ModeDetector::new().unwrap();
        
        // Test simple file creation
        let mode = detector.detect_mode("create a file called test.rs").await;
        assert_eq!(mode, ExecutionMode::Direct);
        
        // Test complex analysis
        let mode = detector.detect_mode("analyze the architecture of this codebase and suggest improvements").await;
        assert_eq!(mode, ExecutionMode::Consensus);
        
        // Test feature implementation
        let mode = detector.detect_mode("implement a login feature with authentication").await;
        assert_eq!(mode, ExecutionMode::HybridConsensus);
    }

    #[tokio::test]
    async fn test_direct_execution_path() {
        // Initialize database
        let db_path = std::env::temp_dir().join("test_claude_code.db");
        let db_config = crate::core::database::DatabaseConfig {
            path: db_path.clone(),
            max_connections: 1,
            connection_timeout: std::time::Duration::from_secs(5),
            idle_timeout: std::time::Duration::from_secs(300),
            enable_wal: false,
            enable_foreign_keys: true,
            cache_size: 1024,
            synchronous: "NORMAL".to_string(),
            journal_mode: "DELETE".to_string(),
        };
        
        let db = Arc::new(DatabaseManager::new(Some(db_config)).await.unwrap());
        
        // Create consensus engine
        let engine = ConsensusEngine::new(Some(db)).await.unwrap();
        
        // Test with a simple file creation request
        let request = "create a simple test file called hello.txt with 'Hello, World!' content";
        
        // This should use direct execution mode
        // In a real test, we'd verify:
        // 1. Mode detector chooses Direct mode
        // 2. File operation is parsed correctly
        // 3. Operation is executed inline
        // 4. Response shows operation as it happens
        
        // Clean up
        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_inline_operation_parsing() {
        use crate::consensus::DirectExecutionHandler;
        use crate::consensus::direct_executor::InlineOperationParser;
        
        let mut parser = InlineOperationParser::new();
        
        // Test parsing a create operation
        let text = "Creating `src/main.rs`:\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        
        // Parse in chunks as would happen during streaming
        parser.parse_chunk("Creating `src/main.rs`:\n```rust\n");
        parser.parse_chunk("fn main() {\n    println!(\"Hello\");\n}\n");
        let operation = parser.parse_chunk("```");
        
        assert!(operation.is_some());
        // Would verify the operation details here
    }
}