// Tests for Operation Preview Generator
use hive_ai::consensus::{
    AIOperationPreviewGenerator, AIPreviewConfig, 
    stages::file_aware_curator::FileOperation,
    operation_analysis::OperationContext,
};
use std::path::PathBuf;
use std::time::SystemTime;
use tokio;

#[tokio::test]
async fn test_generate_simple_create_preview() {
    let config = AIPreviewConfig {
        max_diff_lines: 50,
        context_lines: 3,
        enable_syntax_highlighting: true,
        enable_impact_analysis: true,
    };
    
    let generator = AIOperationPreviewGenerator::new(config);
    
    let operation = FileOperation::Create {
        path: PathBuf::from("src/new_module.rs"),
        content: r#"pub fn hello() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hello() {
        hello();
    }
}"#.to_string(),
    };
    
    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Create a hello module".to_string(),
        consensus_response: "I'll create a new module with a hello function".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test-session".to_string(),
        git_commit: Some("abc123".to_string()),
    };
    
    let preview = generator.generate_preview(&operation, &context).await.unwrap();
    
    // Check basic preview properties
    assert_eq!(preview.operation_type, "Create");
    assert_eq!(preview.file_path, PathBuf::from("src/new_module.rs"));
    
    // Check before state (should be empty for create)
    assert_eq!(preview.before_state.content, "");
    assert!(!preview.before_state.exists);
    
    // Check after state
    assert!(preview.after_state.exists);
    assert!(preview.after_state.content.contains("pub fn hello()"));
    
    // Check diff view
    assert!(preview.diff_view.is_some());
    let diff = preview.diff_view.unwrap();
    assert!(diff.chunks.len() > 0);
    assert_eq!(diff.added_lines, 12); // Count of lines in the content
    assert_eq!(diff.removed_lines, 0);
    
    // Check impact analysis
    assert!(preview.impact.is_some());
    let impact = preview.impact.unwrap();
    assert_eq!(impact.risk_level, "Low");
    assert!(impact.confidence > 0.8);
}

#[tokio::test]
async fn test_generate_update_preview_with_mock_file() {
    let config = AIPreviewConfig {
        max_diff_lines: 50,
        context_lines: 3,
        enable_syntax_highlighting: true,
        enable_impact_analysis: true,
    };
    
    let mut generator = AIOperationPreviewGenerator::new(config);
    
    // Pre-populate file cache to simulate existing file
    let existing_content = r#"pub fn old_hello() {
    println!("Old hello!");
}"#;
    
    generator.cache_file_content(
        PathBuf::from("src/existing.rs"), 
        existing_content.to_string()
    );
    
    let operation = FileOperation::Update {
        path: PathBuf::from("src/existing.rs"),
        content: r#"pub fn hello() {
    println!("Hello, world!");
}

pub fn goodbye() {
    println!("Goodbye!");
}"#.to_string(),
    };
    
    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Update the hello function".to_string(),
        consensus_response: "I'll update the function and add goodbye".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test-session".to_string(),
        git_commit: Some("abc123".to_string()),
    };
    
    let preview = generator.generate_preview(&operation, &context).await.unwrap();
    
    // Check operation type
    assert_eq!(preview.operation_type, "Update");
    
    // Check before state
    assert!(preview.before_state.exists);
    assert_eq!(preview.before_state.content, existing_content);
    
    // Check after state
    assert!(preview.after_state.content.contains("pub fn goodbye()"));
    
    // Check diff
    let diff = preview.diff_view.unwrap();
    assert!(diff.added_lines > 0);
    assert!(diff.removed_lines > 0);
    
    // Should have at least one chunk showing the changes
    assert!(!diff.chunks.is_empty());
}

#[tokio::test]
async fn test_generate_preview_set_with_dependencies() {
    let config = AIPreviewConfig {
        max_diff_lines: 50,
        context_lines: 3,
        enable_syntax_highlighting: true,
        enable_impact_analysis: true,
    };
    
    let generator = AIOperationPreviewGenerator::new(config);
    
    let operations = vec![
        FileOperation::Create {
            path: PathBuf::from("src/models/user.rs"),
            content: "pub struct User { pub id: u64, pub name: String }".to_string(),
        },
        FileOperation::Create {
            path: PathBuf::from("src/models/mod.rs"),
            content: "pub mod user;\npub use user::User;".to_string(),
        },
        FileOperation::Update {
            path: PathBuf::from("src/lib.rs"),
            content: "pub mod models;\npub use models::User;".to_string(),
        },
    ];
    
    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Create user model".to_string(),
        consensus_response: "Creating user model with module structure".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test-session".to_string(),
        git_commit: Some("abc123".to_string()),
    };
    
    let preview_set = generator.generate_preview_set(&operations, &context).await.unwrap();
    
    // Check we got previews for all operations
    assert_eq!(preview_set.previews.len(), 3);
    assert_eq!(preview_set.total_operations, 3);
    
    // Check combined impact
    assert!(preview_set.combined_impact.total_confidence > 0.0);
    assert!(preview_set.combined_impact.total_files_affected >= 3);
    
    // Check timeline
    assert_eq!(preview_set.timeline.steps.len(), 3);
    
    // Verify first step is the user.rs creation
    assert_eq!(preview_set.timeline.steps[0].file_path, PathBuf::from("src/models/user.rs"));
    assert_eq!(preview_set.timeline.steps[0].operation_type, "Create");
}

#[tokio::test]
async fn test_delete_operation_preview() {
    let config = AIPreviewConfig {
        max_diff_lines: 50,
        context_lines: 3,
        enable_syntax_highlighting: true,
        enable_impact_analysis: true,
    };
    
    let mut generator = AIOperationPreviewGenerator::new(config);
    
    // Pre-populate file to be deleted
    let content_to_delete = "pub fn deprecated_function() { /* old code */ }";
    generator.cache_file_content(
        PathBuf::from("src/deprecated.rs"),
        content_to_delete.to_string()
    );
    
    let operation = FileOperation::Delete {
        path: PathBuf::from("src/deprecated.rs"),
    };
    
    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Remove deprecated code".to_string(),
        consensus_response: "Removing deprecated module".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test-session".to_string(),
        git_commit: Some("abc123".to_string()),
    };
    
    let preview = generator.generate_preview(&operation, &context).await.unwrap();
    
    // Check operation type
    assert_eq!(preview.operation_type, "Delete");
    
    // Before state should show the file exists
    assert!(preview.before_state.exists);
    assert_eq!(preview.before_state.content, content_to_delete);
    
    // After state should show file doesn't exist
    assert!(!preview.after_state.exists);
    assert_eq!(preview.after_state.content, "");
    
    // Diff should show all lines removed
    let diff = preview.diff_view.unwrap();
    assert_eq!(diff.added_lines, 0);
    assert_eq!(diff.removed_lines, 1); // One line of content
    
    // Impact should indicate higher risk for deletion
    let impact = preview.impact.unwrap();
    assert_eq!(impact.risk_level, "Medium"); // Deletes are medium risk
}

#[tokio::test]
async fn test_rename_operation_preview() {
    let config = AIPreviewConfig {
        max_diff_lines: 50,
        context_lines: 3,
        enable_syntax_highlighting: true,
        enable_impact_analysis: true,
    };
    
    let mut generator = AIOperationPreviewGenerator::new(config);
    
    // Pre-populate source file
    let file_content = "pub struct OldName { value: String }";
    generator.cache_file_content(
        PathBuf::from("src/old_name.rs"),
        file_content.to_string()
    );
    
    let operation = FileOperation::Rename {
        from: PathBuf::from("src/old_name.rs"),
        to: PathBuf::from("src/new_name.rs"),
    };
    
    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Rename the module".to_string(),
        consensus_response: "Renaming module to better reflect its purpose".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test-session".to_string(),
        git_commit: Some("abc123".to_string()),
    };
    
    let preview = generator.generate_preview(&operation, &context).await.unwrap();
    
    // Check operation type
    assert_eq!(preview.operation_type, "Rename");
    assert_eq!(preview.file_path, PathBuf::from("src/old_name.rs"));
    
    // The description should mention the new path
    assert!(preview.description.contains("new_name.rs"));
    
    // Impact analysis
    let impact = preview.impact.unwrap();
    assert_eq!(impact.risk_level, "Medium");
    assert!(impact.warnings.iter().any(|w| w.contains("import") || w.contains("reference")));
}

#[tokio::test]
async fn test_syntax_highlighting_in_preview() {
    let config = AIPreviewConfig {
        max_diff_lines: 50,
        context_lines: 3,
        enable_syntax_highlighting: true,
        enable_impact_analysis: false,
    };
    
    let generator = AIOperationPreviewGenerator::new(config);
    
    let operation = FileOperation::Create {
        path: PathBuf::from("src/example.rs"),
        content: r#"use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    println!("{:?}", map);
}"#.to_string(),
    };
    
    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Create example".to_string(),
        consensus_response: "Creating example".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test-session".to_string(),
        git_commit: None,
    };
    
    let preview = generator.generate_preview(&operation, &context).await.unwrap();
    
    // With syntax highlighting enabled, the after_state should have a non-empty highlighted version
    assert!(!preview.after_state.highlighted.is_empty());
    
    // The highlighted version should be different from plain content (contains ANSI codes or HTML)
    assert_ne!(preview.after_state.highlighted, preview.after_state.content);
}

#[tokio::test]
async fn test_large_file_diff_truncation() {
    let config = AIPreviewConfig {
        max_diff_lines: 10, // Small limit to test truncation
        context_lines: 2,
        enable_syntax_highlighting: false,
        enable_impact_analysis: false,
    };
    
    let mut generator = AIOperationPreviewGenerator::new(config);
    
    // Create large content
    let mut old_content = String::new();
    let mut new_content = String::new();
    
    for i in 0..50 {
        old_content.push_str(&format!("Line {} old\n", i));
        new_content.push_str(&format!("Line {} new\n", i));
    }
    
    generator.cache_file_content(
        PathBuf::from("src/large.rs"),
        old_content.clone()
    );
    
    let operation = FileOperation::Update {
        path: PathBuf::from("src/large.rs"),
        content: new_content,
    };
    
    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Update large file".to_string(),
        consensus_response: "Updating".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test-session".to_string(),
        git_commit: None,
    };
    
    let preview = generator.generate_preview(&operation, &context).await.unwrap();
    
    // Check that diff was truncated
    let diff = preview.diff_view.unwrap();
    
    // Total diff lines should be limited by max_diff_lines
    let total_diff_lines: usize = diff.chunks.iter()
        .map(|chunk| chunk.lines.len())
        .sum();
    
    assert!(total_diff_lines <= 10 + 4); // max_diff_lines + some context
}

#[tokio::test]
async fn test_visual_summary_generation() {
    let config = AIPreviewConfig {
        max_diff_lines: 50,
        context_lines: 3,
        enable_syntax_highlighting: false,
        enable_impact_analysis: true,
    };
    
    let generator = AIOperationPreviewGenerator::new(config);
    
    let operations = vec![
        FileOperation::Create {
            path: PathBuf::from("src/auth/mod.rs"),
            content: "pub mod login;\npub mod register;".to_string(),
        },
        FileOperation::Create {
            path: PathBuf::from("src/auth/login.rs"),
            content: "pub fn login() -> Result<(), Error> { Ok(()) }".to_string(),
        },
        FileOperation::Update {
            path: PathBuf::from("src/lib.rs"),
            content: "pub mod auth;\npub use auth::login;".to_string(),
        },
        FileOperation::Delete {
            path: PathBuf::from("src/old_auth.rs"),
        },
    ];
    
    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Refactor authentication".to_string(),
        consensus_response: "Refactoring auth module".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test-session".to_string(),
        git_commit: Some("abc123".to_string()),
    };
    
    let preview_set = generator.generate_preview_set(&operations, &context).await.unwrap();
    
    // Check visual summary
    assert!(!preview_set.visual_summary.summary_text.is_empty());
    assert!(!preview_set.visual_summary.key_changes.is_empty());
    
    // Should have key changes for create, update, delete
    assert!(preview_set.visual_summary.key_changes.iter().any(|c| c.contains("Create")));
    assert!(preview_set.visual_summary.key_changes.iter().any(|c| c.contains("Update")));
    assert!(preview_set.visual_summary.key_changes.iter().any(|c| c.contains("Delete")));
    
    // File tree should show the structure
    assert!(!preview_set.visual_summary.file_tree.is_empty());
    assert!(preview_set.visual_summary.file_tree.contains("auth/"));
}

// Helper impl for generator to support test caching
impl AIOperationPreviewGenerator {
    fn cache_file_content(&mut self, path: PathBuf, content: String) {
        // This would need to be implemented in the actual generator
        // For testing, we're assuming this method exists to pre-populate the file cache
        use std::collections::HashMap;
        
        // Access internal file_cache if it's accessible
        // This is a test helper that might need to be added to the actual implementation
        // self.file_cache.insert(path, content);
    }
}