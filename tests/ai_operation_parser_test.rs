// Tests for AI-Enhanced Operation Parser
use crate::consensus::{
    AIOperationParser, FileOperationWithMetadata,
    stages::file_aware_curator::FileOperation,
    operation_intelligence::OperationContext,
};
use std::path::PathBuf;
use std::collections::HashMap;
use tokio;

#[tokio::test]
async fn test_parse_explicit_create_operations() {
    let parser = AIOperationParser::new();
    
    let curator_response = r#"
Based on your request, I'll create the authentication module:

```CREATE:src/auth/mod.rs
pub mod login;
pub mod register;
pub mod session;

pub use login::LoginHandler;
pub use register::RegisterHandler;
pub use session::SessionManager;
```

And then create the login handler:

```CREATE:src/auth/login.rs
use crate::models::User;
use anyhow::Result;

pub struct LoginHandler {
    // Implementation
}

impl LoginHandler {
    pub async fn login(&self, email: &str, password: &str) -> Result<User> {
        // Login logic here
        todo!()
    }
}
```
    "#;

    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        git_commit: Some("abc123".to_string()),
        source_question: "Create authentication module".to_string(),
        related_files: vec![],
        project_metadata: HashMap::new(),
    };

    let result = parser.parse_response(curator_response, &context).await.unwrap();
    
    assert_eq!(result.operations.len(), 2);
    assert!(result.confidence > 85.0);
    assert!(result.warnings.is_empty());
    
    // Check first operation
    match &result.operations[0].operation {
        FileOperation::Create { path, content } => {
            assert_eq!(path.to_str().unwrap(), "src/auth/mod.rs");
            assert!(content.contains("pub mod login"));
        }
        _ => panic!("Expected Create operation"),
    }
    
    // Check second operation
    match &result.operations[1].operation {
        FileOperation::Create { path, content } => {
            assert_eq!(path.to_str().unwrap(), "src/auth/login.rs");
            assert!(content.contains("LoginHandler"));
        }
        _ => panic!("Expected Create operation"),
    }
}

#[tokio::test]
async fn test_parse_update_operations() {
    let parser = AIOperationParser::new();
    
    let curator_response = r#"
I'll update the configuration file to add the new settings:

```UPDATE:config.toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://localhost/myapp"
max_connections = 100

[cache]
enabled = true
ttl = 3600
```
    "#;

    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        git_commit: None,
        source_question: "Update config with cache settings".to_string(),
        related_files: vec![],
        project_metadata: HashMap::new(),
    };

    let result = parser.parse_response(curator_response, &context).await.unwrap();
    
    assert_eq!(result.operations.len(), 1);
    
    match &result.operations[0].operation {
        FileOperation::Update { path, content } => {
            assert_eq!(path.to_str().unwrap(), "config.toml");
            assert!(content.contains("[cache]"));
            assert!(content.contains("ttl = 3600"));
        }
        _ => panic!("Expected Update operation"),
    }
}

#[tokio::test]
async fn test_parse_natural_language_operations() {
    let parser = AIOperationParser::new();
    
    let curator_response = r#"
I'll create a new file called 'utils.py' with the following helper functions:

```python
def validate_email(email: str) -> bool:
    import re
    pattern = r'^[\w\.-]+@[\w\.-]+\.\w+$'
    return re.match(pattern, email) is not None

def hash_password(password: str) -> str:
    import hashlib
    return hashlib.sha256(password.encode()).hexdigest()
```

This will provide basic validation and security utilities.
    "#;

    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        git_commit: None,
        source_question: "Create utility functions".to_string(),
        related_files: vec![],
        project_metadata: HashMap::new(),
    };

    let result = parser.parse_response(curator_response, &context).await.unwrap();
    
    assert_eq!(result.operations.len(), 1);
    
    match &result.operations[0].operation {
        FileOperation::Create { path, content } => {
            assert_eq!(path.to_str().unwrap(), "utils.py");
            assert!(content.contains("validate_email"));
            assert!(content.contains("hash_password"));
        }
        _ => panic!("Expected Create operation"),
    }
}

#[tokio::test]
async fn test_parse_multiple_operations_with_dependencies() {
    let parser = AIOperationParser::new();
    
    let curator_response = r#"
First, I'll create the base model:

```CREATE:src/models/base.rs
pub trait Model {
    fn id(&self) -> u64;
    fn validate(&self) -> Result<(), String>;
}
```

Then update the user model to implement it:

```UPDATE:src/models/user.rs
use super::base::Model;

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub name: String,
}

impl Model for User {
    fn id(&self) -> u64 {
        self.id
    }
    
    fn validate(&self) -> Result<(), String> {
        if self.email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        Ok(())
    }
}
```
    "#;

    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        git_commit: None,
        source_question: "Implement model trait".to_string(),
        related_files: vec![],
        project_metadata: HashMap::new(),
    };

    let result = parser.parse_response(curator_response, &context).await.unwrap();
    
    assert_eq!(result.operations.len(), 2);
    
    // Check dependencies - update should depend on create
    assert!(result.operations[0].dependencies.is_empty());
    // Note: In a more sophisticated implementation, the parser would detect
    // that the update depends on the create due to the import
}

#[tokio::test]
async fn test_parse_ambiguous_code_blocks() {
    let parser = AIOperationParser::new();
    
    let curator_response = r#"
Here's the updated error handling for the application:

```rust
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(String),
    Network(String),
    Validation(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}
```

This should be placed in src/error.rs to centralize error handling.
    "#;

    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        git_commit: None,
        source_question: "Add error handling".to_string(),
        related_files: vec![],
        project_metadata: HashMap::new(),
    };

    let result = parser.parse_response(curator_response, &context).await.unwrap();
    
    // Parser should infer this is likely an update/create operation
    assert_eq!(result.operations.len(), 1);
    // Confidence might be lower due to ambiguity
    assert!(result.confidence < 90.0);
    
    // Should mention the suggested path from context
    let op = &result.operations[0];
    match &op.operation {
        FileOperation::Create { path, .. } | FileOperation::Update { path, .. } => {
            // Parser should have inferred src/error.rs from the context
            assert!(path.to_str().unwrap().contains("error"));
        }
        _ => panic!("Expected Create or Update operation"),
    }
}

#[tokio::test]
async fn test_parse_delete_and_rename_operations() {
    let parser = AIOperationParser::new();
    
    let curator_response = r#"
Let me clean up the old files and reorganize:

```DELETE:src/old_module.rs
```

And rename the temporary file to the final name:

```RENAME:src/temp.rs to src/final.rs
```
    "#;

    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        git_commit: None,
        source_question: "Clean up files".to_string(),
        related_files: vec![],
        project_metadata: HashMap::new(),
    };

    let result = parser.parse_response(curator_response, &context).await.unwrap();
    
    assert_eq!(result.operations.len(), 2);
    
    // Check delete operation
    match &result.operations[0].operation {
        FileOperation::Delete { path } => {
            assert_eq!(path.to_str().unwrap(), "src/old_module.rs");
        }
        _ => panic!("Expected Delete operation"),
    }
    
    // Check rename operation
    match &result.operations[1].operation {
        FileOperation::Rename { from, to } => {
            assert_eq!(from.to_str().unwrap(), "src/temp.rs");
            assert_eq!(to.to_str().unwrap(), "src/final.rs");
        }
        _ => panic!("Expected Rename operation"),
    }
}

#[tokio::test]
async fn test_parse_with_warnings_and_low_confidence() {
    let parser = AIOperationParser::new();
    
    let curator_response = r#"
Here's some code that might help:

```
function doSomething() {
    // Not sure what file this goes in
    console.log("Hello");
}
```

You might want to put this somewhere in your project.
    "#;

    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        git_commit: None,
        source_question: "Add helper function".to_string(),
        related_files: vec![],
        project_metadata: HashMap::new(),
    };

    let result = parser.parse_response(curator_response, &context).await.unwrap();
    
    // Should have low confidence due to missing path info
    assert!(result.confidence < 70.0);
    
    // Should have warnings or clarifications
    assert!(!result.warnings.is_empty() || !result.clarifications.is_empty());
}

#[tokio::test]
async fn test_parse_append_operations() {
    let parser = AIOperationParser::new();
    
    let curator_response = r#"
I'll add the new test cases to the existing test file:

```APPEND:tests/unit_tests.rs

#[test]
fn test_edge_case_empty_input() {
    let result = process("");
    assert!(result.is_err());
}

#[test]
fn test_edge_case_very_long_input() {
    let input = "a".repeat(10000);
    let result = process(&input);
    assert!(result.is_ok());
}
```
    "#;

    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        git_commit: None,
        source_question: "Add edge case tests".to_string(),
        related_files: vec![],
        project_metadata: HashMap::new(),
    };

    let result = parser.parse_response(curator_response, &context).await.unwrap();
    
    assert_eq!(result.operations.len(), 1);
    
    match &result.operations[0].operation {
        FileOperation::Append { path, content } => {
            assert_eq!(path.to_str().unwrap(), "tests/unit_tests.rs");
            assert!(content.contains("test_edge_case_empty_input"));
            assert!(content.contains("test_edge_case_very_long_input"));
        }
        _ => panic!("Expected Append operation"),
    }
}

#[tokio::test]
async fn test_extract_rationale() {
    let parser = AIOperationParser::new();
    
    let curator_response = r#"
Since we need to handle authentication tokens securely, I'll create a dedicated token manager:

```CREATE:src/auth/token.rs
use jsonwebtoken::{encode, decode, Header, Validation};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub struct TokenManager {
    secret: String,
}
```

This will ensure all JWT operations are centralized and secure.
    "#;

    let context = OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        git_commit: None,
        source_question: "Add token management".to_string(),
        related_files: vec![],
        project_metadata: HashMap::new(),
    };

    let result = parser.parse_response(curator_response, &context).await.unwrap();
    
    assert_eq!(result.operations.len(), 1);
    
    // Check if rationale was extracted
    let op = &result.operations[0];
    assert!(op.rationale.is_some());
    if let Some(rationale) = &op.rationale {
        assert!(rationale.contains("handle authentication tokens securely"));
    }
}