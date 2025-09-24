//! Standardized Curator Output Format for File Operations
//!
//! This module defines the standardized format that Curator stages should use
//! when outputting file operations. This ensures reliable parsing by AI Helpers
//! while maintaining human readability.
//!
//! Architecture Principle:
//! - Curator (THINKING): Creates formatted output using this specification
//! - AI Helpers (DOING): Parse and execute operations from this format

use crate::consensus::stages::file_aware_curator::FileOperation;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Standardized format specification for Curator output
#[derive(Debug, Clone)]
pub struct CuratorOutputFormat;

impl CuratorOutputFormat {
    /// Generate properly formatted operation text that AI Helpers can parse
    pub fn format_operations(operations: &[FileOperation]) -> String {
        let mut output = String::new();

        if operations.is_empty() {
            return output;
        }

        // Add operation header for AI parsing
        output.push_str("## File Operations\n\n");
        output.push_str("The following operations will be performed:\n\n");

        for (index, operation) in operations.iter().enumerate() {
            output.push_str(&Self::format_single_operation(operation, index + 1));
            output.push('\n');
        }

        output
    }

    /// Format a single operation following the standardized format
    pub fn format_single_operation(operation: &FileOperation, step: usize) -> String {
        match operation {
            FileOperation::Create { path, content } => {
                Self::format_create_operation(path, content, step)
            }
            FileOperation::Update { path, content } => {
                Self::format_update_operation(path, content, step)
            }
            FileOperation::Delete { path } => Self::format_delete_operation(path, step),
            FileOperation::Rename { from, to } => Self::format_rename_operation(from, to, step),
            FileOperation::Append { path, content } => {
                Self::format_append_operation(path, content, step)
            }
        }
    }

    /// Format file creation operation
    fn format_create_operation(path: &PathBuf, content: &str, step: usize) -> String {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("text");

        format!(
            "### Step {}: Creating `{}`\n\n```{}:{}\n{}\n```\n\n✅ **Created**: {}\n",
            step,
            path.display(),
            Self::get_language_identifier(ext),
            path.display(),
            content.trim(),
            path.display()
        )
    }

    /// Format file update operation
    fn format_update_operation(path: &PathBuf, content: &str, step: usize) -> String {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("text");

        format!(
            "### Step {}: Updating `{}`\n\n```{}:{}\n{}\n```\n\n✅ **Updated**: {}\n",
            step,
            path.display(),
            Self::get_language_identifier(ext),
            path.display(),
            content.trim(),
            path.display()
        )
    }

    /// Format file deletion operation
    fn format_delete_operation(path: &PathBuf, step: usize) -> String {
        format!(
            "### Step {}: Deleting `{}`\n\n```delete:{}\n# File will be deleted\n```\n\n❌ **Deleted**: {}\n",
            step,
            path.display(),
            path.display(),
            path.display()
        )
    }

    /// Format file rename operation
    fn format_rename_operation(from: &PathBuf, to: &PathBuf, step: usize) -> String {
        format!(
            "### Step {}: Renaming `{}` to `{}`\n\n```rename:{} to {}\n# File will be renamed\n```\n\n🔄 **Renamed**: {} → {}\n",
            step,
            from.display(),
            to.display(),
            from.display(),
            to.display(),
            from.display(),
            to.display()
        )
    }

    /// Format file append operation
    fn format_append_operation(path: &PathBuf, content: &str, step: usize) -> String {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("text");

        format!(
            "### Step {}: Appending to `{}`\n\n```{}:{}\n{}\n```\n\n➕ **Appended to**: {}\n",
            step,
            path.display(),
            Self::get_language_identifier(ext),
            path.display(),
            content.trim(),
            path.display()
        )
    }

    /// Get language identifier for syntax highlighting
    fn get_language_identifier(extension: &str) -> &str {
        match extension {
            "rs" => "rust",
            "js" => "javascript",
            "ts" => "typescript",
            "py" => "python",
            "java" => "java",
            "cpp" | "cc" | "cxx" => "cpp",
            "c" => "c",
            "h" | "hpp" => "c",
            "go" => "go",
            "rb" => "ruby",
            "php" => "php",
            "md" => "markdown",
            "html" => "html",
            "css" => "css",
            "scss" => "scss",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "xml" => "xml",
            "sql" => "sql",
            "sh" | "bash" => "bash",
            "ps1" => "powershell",
            "dockerfile" => "dockerfile",
            _ => "text",
        }
    }
}

/// Format specification for AI Helper parsers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatSpecification {
    /// Version of the format specification
    pub version: String,

    /// Supported operation patterns
    pub patterns: OperationPatterns,

    /// Parsing rules and guidelines
    pub parsing_rules: ParsingRules,
}

/// Operation patterns that AI Helpers should recognize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPatterns {
    /// Legacy patterns for backward compatibility
    pub legacy: LegacyPatterns,

    /// New standardized patterns
    pub standardized: StandardizedPatterns,
}

/// Legacy patterns (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyPatterns {
    pub create: Vec<String>,
    pub update: Vec<String>,
    pub delete: Vec<String>,
    pub rename: Vec<String>,
}

/// New standardized patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardizedPatterns {
    /// Code block header format: "language:path" or "operation:path"
    pub header_format: String,

    /// Step header format: "### Step N: Operation `path`"
    pub step_format: String,

    /// Status indicators: "✅ **Created**: path"
    pub status_format: String,
}

/// Parsing rules for AI Helpers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsingRules {
    /// Confidence scoring guidelines
    pub confidence_rules: ConfidenceRules,

    /// Error handling guidelines
    pub error_handling: ErrorHandling,

    /// Dependency detection rules
    pub dependency_rules: DependencyRules,
}

/// Rules for calculating parsing confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceRules {
    /// Base confidence for well-formed headers
    pub base_confidence: f32,

    /// Confidence boost for explicit operation types
    pub explicit_operation_boost: f32,

    /// Confidence reduction for ambiguous content
    pub ambiguity_penalty: f32,
}

/// Error handling guidelines for parsers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandling {
    /// Whether to continue parsing after errors
    pub continue_on_error: bool,

    /// Minimum confidence threshold to accept operations
    pub min_confidence_threshold: f32,

    /// How to handle unparseable blocks
    pub unparseable_block_strategy: String,
}

/// Rules for detecting operation dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRules {
    /// Operations that depend on file creation
    pub create_dependencies: Vec<String>,

    /// Operations that conflict with deletion
    pub delete_conflicts: Vec<String>,

    /// Order-sensitive operation pairs
    pub ordering_rules: Vec<OrderingRule>,
}

/// Rule for operation ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderingRule {
    pub before: String,
    pub after: String,
    pub reason: String,
}

impl Default for FormatSpecification {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            patterns: OperationPatterns {
                legacy: LegacyPatterns {
                    create: vec![
                        "Creating `".to_string(),
                        "Writing to `".to_string(),
                        "New file `".to_string(),
                    ],
                    update: vec![
                        "Updating `".to_string(),
                        "Modifying `".to_string(),
                        "Editing `".to_string(),
                    ],
                    delete: vec!["Deleting `".to_string(), "Removing `".to_string()],
                    rename: vec!["Renaming `".to_string(), "Moving `".to_string()],
                },
                standardized: StandardizedPatterns {
                    header_format: "language:path or operation:path".to_string(),
                    step_format: "### Step N: Operation `path`".to_string(),
                    status_format: "✅ **Operation**: path".to_string(),
                },
            },
            parsing_rules: ParsingRules {
                confidence_rules: ConfidenceRules {
                    base_confidence: 90.0,
                    explicit_operation_boost: 10.0,
                    ambiguity_penalty: 20.0,
                },
                error_handling: ErrorHandling {
                    continue_on_error: true,
                    min_confidence_threshold: 70.0,
                    unparseable_block_strategy: "log_and_skip".to_string(),
                },
                dependency_rules: DependencyRules {
                    create_dependencies: vec!["update".to_string(), "append".to_string()],
                    delete_conflicts: vec!["update".to_string(), "append".to_string()],
                    ordering_rules: vec![
                        OrderingRule {
                            before: "create".to_string(),
                            after: "update".to_string(),
                            reason: "File must exist before it can be updated".to_string(),
                        },
                        OrderingRule {
                            before: "create".to_string(),
                            after: "append".to_string(),
                            reason: "File must exist before content can be appended".to_string(),
                        },
                    ],
                },
            },
        }
    }
}

/// Curator guidelines for generating consistent output
pub struct CuratorGuidelines;

impl CuratorGuidelines {
    /// System prompt addition for Curator to use standardized format
    pub fn get_format_instructions() -> &'static str {
        r#"
## File Operation Output Format (REQUIRED)

When your response includes file operations, use this EXACT format:

### For Creating Files:
```
### Step 1: Creating `path/to/file.ext`

```language:path/to/file.ext
file content here
```

✅ **Created**: path/to/file.ext
```

### For Updating Files:
```
### Step 1: Updating `path/to/file.ext`

```language:path/to/file.ext
updated content here
```

✅ **Updated**: path/to/file.ext
```

### For Deleting Files:
```
### Step 1: Deleting `path/to/file.ext`

```delete:path/to/file.ext
# File will be deleted
```

❌ **Deleted**: path/to/file.ext
```

### For Renaming Files:
```
### Step 1: Renaming `old/path.ext` to `new/path.ext`

```rename:old/path.ext to new/path.ext
# File will be renamed
```

🔄 **Renamed**: old/path.ext → new/path.ext
```

CRITICAL: Always use the exact format above. AI Helpers depend on this structure for reliable parsing and execution.
"#
    }

    /// Validation rules for Curator output
    pub fn validate_output(output: &str) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check for proper step headers
        if !output.contains("### Step") && Self::contains_operations(output) {
            issues.push("Missing step headers for operations".to_string());
        }

        // Check for code block format
        if output.contains("Creating `") || output.contains("Updating `") {
            if !output.contains("```") {
                issues.push("Operations found but missing code blocks".to_string());
            }
        }

        // Check for status indicators
        if Self::contains_operations(output) && !output.contains("✅") {
            warnings.push("Missing status indicators for operations".to_string());
        }

        ValidationResult {
            is_valid: issues.is_empty(),
            issues,
            warnings,
        }
    }

    fn contains_operations(output: &str) -> bool {
        output.contains("Creating `")
            || output.contains("Updating `")
            || output.contains("Deleting `")
            || output.contains("Renaming `")
    }
}

/// Result of validating Curator output format
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_create_operation() {
        let operation = FileOperation::Create {
            path: PathBuf::from("src/main.rs"),
            content: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
        };

        let formatted = CuratorOutputFormat::format_single_operation(&operation, 1);

        assert!(formatted.contains("### Step 1: Creating `src/main.rs`"));
        assert!(formatted.contains("```rust:src/main.rs"));
        assert!(formatted.contains("fn main() {"));
        assert!(formatted.contains("✅ **Created**: src/main.rs"));
    }

    #[test]
    fn test_format_multiple_operations() {
        let operations = vec![
            FileOperation::Create {
                path: PathBuf::from("Cargo.toml"),
                content: "[package]\nname = \"test\"".to_string(),
            },
            FileOperation::Update {
                path: PathBuf::from("README.md"),
                content: "# Test Project".to_string(),
            },
        ];

        let formatted = CuratorOutputFormat::format_operations(&operations);

        assert!(formatted.contains("## File Operations"));
        assert!(formatted.contains("### Step 1: Creating `Cargo.toml`"));
        assert!(formatted.contains("### Step 2: Updating `README.md`"));
    }

    #[test]
    fn test_language_identifier() {
        assert_eq!(CuratorOutputFormat::get_language_identifier("rs"), "rust");
        assert_eq!(
            CuratorOutputFormat::get_language_identifier("js"),
            "javascript"
        );
        assert_eq!(CuratorOutputFormat::get_language_identifier("py"), "python");
        assert_eq!(
            CuratorOutputFormat::get_language_identifier("unknown"),
            "text"
        );
    }

    #[test]
    fn test_output_validation() {
        let valid_output = r#"
### Step 1: Creating `test.rs`

```rust:test.rs
fn test() {}
```

✅ **Created**: test.rs
"#;

        let result = CuratorGuidelines::validate_output(valid_output);
        assert!(result.is_valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_invalid_output_validation() {
        let invalid_output = "Creating `test.rs`: some content";

        let result = CuratorGuidelines::validate_output(invalid_output);
        assert!(!result.is_valid);
        assert!(!result.issues.is_empty());
    }
}
