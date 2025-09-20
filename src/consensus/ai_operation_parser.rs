// AI-Enhanced Operation Parser
// This module provides intelligent parsing of curator output to extract file operations
// using pattern recognition, NLP, and contextual understanding

use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::ai_helpers::PatternRecognizer;
use crate::consensus::operation_intelligence::OperationContext;
use crate::consensus::stages::file_aware_curator::FileOperation;

/// Result of parsing a curator response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedOperations {
    /// Successfully parsed operations
    pub operations: Vec<FileOperationWithMetadata>,

    /// Parsing confidence score (0-100)
    pub confidence: f32,

    /// Any warnings or ambiguities detected
    pub warnings: Vec<String>,

    /// Suggested clarifications if parsing was ambiguous
    pub clarifications: Vec<String>,

    /// Raw blocks that couldn't be parsed
    pub unparsed_blocks: Vec<String>,
}

/// File operation with additional metadata from parsing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileOperationWithMetadata {
    /// The parsed operation
    pub operation: FileOperation,

    /// Confidence in this specific operation (0-100)
    pub confidence: f32,

    /// Rationale extracted from curator response
    pub rationale: Option<String>,

    /// Dependencies on other operations
    pub dependencies: Vec<usize>,

    /// Source location in the original text
    pub source_location: SourceLocation,
}

/// Location of parsed content in source text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceLocation {
    /// Starting character position
    pub start: usize,

    /// Ending character position
    pub end: usize,

    /// Line number where operation starts
    pub line: usize,
}

/// AI-Enhanced Operation Parser
pub struct AIOperationParser {
    /// Pattern recognizer for intelligent pattern matching
    pattern_recognizer: Option<PatternRecognizer>,

    /// Compiled regex patterns for operation detection
    operation_patterns: HashMap<String, Regex>,

    /// Language-specific parsers
    language_parsers: HashMap<String, Box<dyn LanguageParser>>,
}

/// Trait for language-specific parsing
pub trait LanguageParser: Send + Sync {
    /// Validate syntax for this language
    fn validate_syntax(&self, content: &str) -> Result<()>;

    /// Extract imports/dependencies
    fn extract_dependencies(&self, content: &str) -> Vec<String>;

    /// Get file extension for this language
    fn file_extension(&self) -> &str;
}

impl AIOperationParser {
    pub fn new() -> Self {
        let mut operation_patterns = HashMap::new();

        // Enhanced regex patterns for operation detection
        operation_patterns.insert(
            "create_block".to_string(),
            Regex::new(r"(?i)```(?:create|new|add)[:\s]+([^\n]+)\n([\s\S]*?)```").unwrap(),
        );

        operation_patterns.insert(
            "update_block".to_string(),
            Regex::new(r"(?i)```(?:update|modify|edit|change)[:\s]+([^\n]+)\n([\s\S]*?)```")
                .unwrap(),
        );

        operation_patterns.insert(
            "delete_block".to_string(),
            Regex::new(r"(?i)```(?:delete|remove)[:\s]+([^\n]+)\n?```").unwrap(),
        );

        operation_patterns.insert(
            "append_block".to_string(),
            Regex::new(r"(?i)```(?:append|add\s+to)[:\s]+([^\n]+)\n([\s\S]*?)```").unwrap(),
        );

        operation_patterns.insert(
            "rename_block".to_string(),
            Regex::new(r"(?i)```(?:rename|move)[:\s]+([^\n]+)\s+(?:to|->)\s+([^\n]+)\n?```")
                .unwrap(),
        );

        // Natural language patterns
        operation_patterns.insert(
            "nl_create".to_string(),
            Regex::new(r"(?i)(?:create|add|generate)\s+(?:a\s+)?(?:new\s+)?file\s+(?:called\s+|named\s+)?[`']([^`']+)[`']").unwrap()
        );

        operation_patterns.insert(
            "nl_update".to_string(),
            Regex::new(r"(?i)(?:update|modify|change|edit)\s+(?:the\s+)?file\s+[`']([^`']+)[`']")
                .unwrap(),
        );

        operation_patterns.insert(
            "inline_code".to_string(),
            Regex::new(r"(?i)(?:in|for)\s+[`']([^`']+)[`'].*?:\n```([^`]+)\n([\s\S]*?)```")
                .unwrap(),
        );

        Self {
            pattern_recognizer: None,
            operation_patterns,
            language_parsers: HashMap::new(),
        }
    }

    /// Set pattern recognizer for AI-enhanced parsing
    pub fn with_pattern_recognizer(mut self, recognizer: PatternRecognizer) -> Self {
        self.pattern_recognizer = Some(recognizer);
        self
    }

    /// Parse curator response into file operations
    pub async fn parse_response(
        &self,
        response: &str,
        context: &OperationContext,
    ) -> Result<ParsedOperations> {
        info!("🤖 AI-Enhanced parsing of curator response");

        let mut operations = Vec::new();
        let mut warnings = Vec::new();
        let mut unparsed_blocks = Vec::new();
        let mut overall_confidence = 100.0;

        // Step 1: Extract all code blocks first
        let code_blocks = self.extract_code_blocks(response);
        debug!("Found {} code blocks in response", code_blocks.len());

        // Step 2: Try to parse each code block
        for (idx, block) in code_blocks.iter().enumerate() {
            match self.parse_code_block(block, idx, response).await {
                Ok(op) => operations.push(op),
                Err(e) => {
                    warn!("Failed to parse code block: {}", e);
                    unparsed_blocks.push(block.content.clone());
                    overall_confidence *= 0.9;
                }
            }
        }

        // Step 3: Look for natural language file operations
        let nl_operations = self.parse_natural_language_operations(response).await?;
        operations.extend(nl_operations);

        // Step 4: Analyze dependencies between operations
        self.analyze_dependencies(&mut operations);

        // Step 5: Use AI pattern recognition if available
        if let Some(recognizer) = &self.pattern_recognizer {
            let ai_insights = self
                .get_ai_insights(response, &operations, context, recognizer)
                .await?;
            warnings.extend(ai_insights.warnings);
            overall_confidence *= ai_insights.confidence_modifier;
        }

        // Step 6: Validate and sanitize operations
        let validation_results = self.validate_operations(&operations);
        warnings.extend(validation_results.warnings);

        // Calculate final confidence
        let operation_confidence_avg = if operations.is_empty() {
            0.0
        } else {
            operations.iter().map(|op| op.confidence).sum::<f32>() / operations.len() as f32
        };

        let final_confidence =
            (overall_confidence * operation_confidence_avg / 100.0).clamp(0.0, 100.0);

        // Generate clarifications if confidence is low
        let clarifications = if final_confidence < 70.0 {
            self.generate_clarifications(&operations, &unparsed_blocks)
        } else {
            vec![]
        };

        Ok(ParsedOperations {
            operations,
            confidence: final_confidence,
            warnings,
            clarifications,
            unparsed_blocks,
        })
    }

    /// Extract all code blocks from text
    fn extract_code_blocks(&self, text: &str) -> Vec<CodeBlock> {
        let mut blocks = Vec::new();
        let code_block_regex = Regex::new(r"```([^\n]*)\n([\s\S]*?)```").unwrap();

        for cap in code_block_regex.captures_iter(text) {
            let header = cap.get(1).map_or("", |m| m.as_str()).trim();
            let content = cap.get(2).map_or("", |m| m.as_str());
            let start = cap.get(0).map_or(0, |m| m.start());
            let end = cap.get(0).map_or(0, |m| m.end());

            // Count line number
            let line = text[..start].matches('\n').count() + 1;

            blocks.push(CodeBlock {
                header: header.to_string(),
                content: content.to_string(),
                location: SourceLocation { start, end, line },
            });
        }

        blocks
    }

    /// Parse a single code block into an operation
    async fn parse_code_block(
        &self,
        block: &CodeBlock,
        index: usize,
        full_text: &str,
    ) -> Result<FileOperationWithMetadata> {
        // Try to parse header for operation type and path
        let (op_type, path) = self.parse_block_header(&block.header)?;

        // Determine confidence based on header clarity
        let mut confidence = if block.header.contains(':') {
            90.0
        } else {
            70.0
        };

        // Create the operation based on type
        let operation = match op_type.to_lowercase().as_str() {
            "create" | "new" | "add" => FileOperation::Create {
                path: PathBuf::from(&path),
                content: block.content.clone(),
            },
            "update" | "modify" | "edit" | "change" => FileOperation::Update {
                path: PathBuf::from(&path),
                content: block.content.clone(),
            },
            "append" | "add to" => FileOperation::Append {
                path: PathBuf::from(&path),
                content: block.content.clone(),
            },
            "delete" | "remove" => {
                confidence *= 0.9; // Slightly lower confidence for destructive operations
                FileOperation::Delete {
                    path: PathBuf::from(&path),
                }
            }
            "rename" | "move" => {
                // Try to parse destination from header
                let parts: Vec<&str> = path.split(" to ").collect();
                if parts.len() == 2 {
                    FileOperation::Rename {
                        from: PathBuf::from(parts[0].trim()),
                        to: PathBuf::from(parts[1].trim()),
                    }
                } else {
                    return Err(anyhow::anyhow!("Invalid rename operation format"));
                }
            }
            lang if self.is_language_identifier(lang) => {
                // This is likely a language-tagged code block
                // Try to infer operation from context
                self.infer_operation_from_context(block, full_text, confidence)?
            }
            _ => {
                // Unknown operation type, try to infer
                confidence *= 0.7;
                self.infer_operation_from_context(block, full_text, confidence)?
            }
        };

        // Extract rationale from surrounding text
        let rationale = self.extract_rationale(full_text, block.location.start);

        // Validate syntax if we have a language parser
        if let Some(ext) = PathBuf::from(&path).extension() {
            if let Some(parser) = self.language_parsers.get(ext.to_str().unwrap_or("")) {
                if let Err(e) = parser.validate_syntax(&block.content) {
                    confidence *= 0.8;
                    debug!("Syntax validation warning: {}", e);
                }
            }
        }

        Ok(FileOperationWithMetadata {
            operation,
            confidence,
            rationale,
            dependencies: vec![], // Will be filled by analyze_dependencies
            source_location: block.location.clone(),
        })
    }

    /// Parse block header to extract operation type and path
    fn parse_block_header(&self, header: &str) -> Result<(String, String)> {
        // Handle explicit operation:path format
        if let Some(colon_pos) = header.find(':') {
            let op_type = header[..colon_pos].trim().to_string();
            let path = header[colon_pos + 1..].trim().to_string();
            return Ok((op_type, path));
        }

        // Handle language identifier only (e.g., ```rust)
        if !header.is_empty() && !header.contains(' ') {
            return Ok((header.to_string(), String::new()));
        }

        // Try to parse other formats
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() >= 2 {
            Ok((parts[0].to_string(), parts[1..].join(" ")))
        } else if parts.len() == 1 {
            Ok((parts[0].to_string(), String::new()))
        } else {
            Ok((String::new(), String::new()))
        }
    }

    /// Check if a string is a known language identifier
    fn is_language_identifier(&self, s: &str) -> bool {
        matches!(
            s,
            "rust"
                | "rs"
                | "python"
                | "py"
                | "javascript"
                | "js"
                | "typescript"
                | "ts"
                | "go"
                | "java"
                | "cpp"
                | "c"
                | "ruby"
                | "rb"
                | "php"
                | "swift"
                | "kotlin"
                | "scala"
        )
    }

    /// Infer operation from context when header is ambiguous
    fn infer_operation_from_context(
        &self,
        block: &CodeBlock,
        full_text: &str,
        mut confidence: f32,
    ) -> Result<FileOperation> {
        // Look for clues in surrounding text
        let context_start = block.location.start.saturating_sub(200);
        let context = &full_text[context_start..block.location.start];

        // Check for file path mentions
        let path_regex = Regex::new(r"(?:file|path|in)\s+[`']([^`']+)[`']").unwrap();
        let path = if let Some(cap) = path_regex.captures(context).and_then(|c| c.get(1)) {
            cap.as_str().to_string()
        } else {
            // Try to infer from content
            confidence *= 0.7;
            self.infer_path_from_content(&block.content)
        };

        // Check for operation keywords
        let create_keywords = ["create", "new", "add", "generate"];
        let update_keywords = ["update", "modify", "change", "edit", "fix", "improve"];

        let context_lower = context.to_lowercase();

        if create_keywords.iter().any(|&kw| context_lower.contains(kw)) {
            Ok(FileOperation::Create {
                path: PathBuf::from(path),
                content: block.content.clone(),
            })
        } else if update_keywords.iter().any(|&kw| context_lower.contains(kw)) {
            Ok(FileOperation::Update {
                path: PathBuf::from(path),
                content: block.content.clone(),
            })
        } else {
            // Default to update for existing files, create for new
            confidence *= 0.6;
            Ok(FileOperation::Update {
                path: PathBuf::from(path),
                content: block.content.clone(),
            })
        }
    }

    /// Try to infer file path from content
    fn infer_path_from_content(&self, content: &str) -> String {
        // Look for module declarations, package statements, etc.
        let patterns = vec![
            (r"(?m)^module\s+(\S+)", "go"),
            (r"(?m)^package\s+(\S+)", "java"),
            (r"(?m)^pub\s+mod\s+(\S+)", "rs"),
            (r"(?m)^class\s+(\w+)", "py"),
            (
                r"(?m)^export\s+(?:default\s+)?(?:class|function)\s+(\w+)",
                "ts",
            ),
        ];

        for (pattern, ext) in patterns {
            let regex = Regex::new(pattern).unwrap();
            if let Some(cap) = regex.captures(content).and_then(|c| c.get(1)) {
                let name = cap.as_str();
                return format!("{}.{}", name.to_lowercase(), ext);
            }
        }

        // Fallback to generic name
        "untitled.txt".to_string()
    }

    /// Parse natural language file operations
    async fn parse_natural_language_operations(
        &self,
        text: &str,
    ) -> Result<Vec<FileOperationWithMetadata>> {
        let mut operations = Vec::new();

        // Look for patterns like "create a new file called 'test.rs'"
        if let Some(regex) = self.operation_patterns.get("nl_create") {
            for cap in regex.captures_iter(text) {
                if let Some(path_match) = cap.get(1) {
                    let path = path_match.as_str();
                    let start = cap.get(0).map_or(0, |m| m.start());

                    // Try to find associated code block
                    let content = self.find_associated_code_block(text, start);
                    let has_content = content.is_some();

                    operations.push(FileOperationWithMetadata {
                        operation: FileOperation::Create {
                            path: PathBuf::from(path),
                            content: content.unwrap_or_default(),
                        },
                        confidence: if has_content { 85.0 } else { 60.0 },
                        rationale: None,
                        dependencies: vec![],
                        source_location: SourceLocation {
                            start,
                            end: cap.get(0).map_or(0, |m| m.end()),
                            line: text.chars().take(start).filter(|&c| c == '\n').count() + 1,
                        },
                    });
                }
            }
        }

        Ok(operations)
    }

    /// Find code block associated with a natural language operation
    fn find_associated_code_block(&self, text: &str, start_pos: usize) -> Option<String> {
        // Ensure we start at a valid character boundary
        let start_pos = text
            .char_indices()
            .find(|(idx, _)| *idx >= start_pos)
            .map(|(idx, _)| idx)
            .unwrap_or(text.len());

        if start_pos >= text.len() {
            return None;
        }

        // Look for the next code block after the operation mention
        let remaining_text = &text[start_pos..];
        let code_block_regex = Regex::new(r"```[^\n]*\n([\s\S]*?)```").unwrap();

        if let Some(cap) = code_block_regex.captures(remaining_text) {
            // Check if the code block is within reasonable distance (e.g., 500 chars)
            if cap.get(0).map_or(0, |m| m.start()) < 500 {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }

        None
    }

    /// Extract rationale from text before an operation
    fn extract_rationale(&self, text: &str, operation_start: usize) -> Option<String> {
        // Ensure we don't slice in the middle of a UTF-8 character
        let context_start = operation_start.saturating_sub(300);

        // Find the nearest character boundary
        let context_start = if context_start > 0 {
            text.char_indices()
                .rev()
                .find(|(idx, _)| *idx <= context_start)
                .map(|(idx, _)| idx)
                .unwrap_or(0)
        } else {
            0
        };

        let operation_start = text
            .char_indices()
            .find(|(idx, _)| *idx >= operation_start)
            .map(|(idx, _)| idx)
            .unwrap_or(text.len());

        if context_start >= operation_start {
            return None;
        }

        let context = &text[context_start..operation_start];

        // Look for explanation patterns
        let patterns = vec![
            r"(?i)(?:because|since|as|to)\s+([^.!?\n]+[.!?])",
            r"(?i)(?:this will|this should|we need to)\s+([^.!?\n]+[.!?])",
            r"(?i)(?:in order to|so that)\s+([^.!?\n]+[.!?])",
        ];

        for pattern in patterns {
            let regex = Regex::new(pattern).unwrap();
            if let Some(cap) = regex.captures(context).and_then(|c| c.get(1)) {
                return Some(cap.as_str().trim().to_string());
            }
        }

        None
    }

    /// Analyze dependencies between operations
    fn analyze_dependencies(&self, operations: &mut [FileOperationWithMetadata]) {
        // Simple dependency analysis based on file paths
        for i in 0..operations.len() {
            for j in 0..i {
                if self
                    .operations_have_dependency(&operations[j].operation, &operations[i].operation)
                {
                    operations[i].dependencies.push(j);
                }
            }
        }
    }

    /// Check if operation B depends on operation A
    fn operations_have_dependency(&self, op_a: &FileOperation, op_b: &FileOperation) -> bool {
        match (op_a, op_b) {
            // Create must happen before update/append on same file
            (
                FileOperation::Create { path: path_a, .. },
                FileOperation::Update { path: path_b, .. },
            )
            | (
                FileOperation::Create { path: path_a, .. },
                FileOperation::Append { path: path_b, .. },
            ) => path_a == path_b,
            // Rename affects subsequent operations on the renamed file
            (FileOperation::Rename { to, .. }, FileOperation::Update { path, .. })
            | (FileOperation::Rename { to, .. }, FileOperation::Append { path, .. })
            | (FileOperation::Rename { to, .. }, FileOperation::Delete { path }) => to == path,
            _ => false,
        }
    }

    /// Get AI insights using pattern recognizer
    async fn get_ai_insights(
        &self,
        response: &str,
        operations: &[FileOperationWithMetadata],
        context: &OperationContext,
        recognizer: &PatternRecognizer,
    ) -> Result<AIInsights> {
        // This would use the pattern recognizer to analyze the operations
        // For now, return placeholder insights
        Ok(AIInsights {
            warnings: vec![],
            confidence_modifier: 1.0,
        })
    }

    /// Validate parsed operations
    fn validate_operations(&self, operations: &[FileOperationWithMetadata]) -> ValidationResult {
        let mut warnings = Vec::new();

        for (idx, op) in operations.iter().enumerate() {
            // Check for duplicate operations on same file
            for (other_idx, other_op) in operations.iter().enumerate() {
                if idx != other_idx && self.operations_conflict(&op.operation, &other_op.operation)
                {
                    warnings.push(format!(
                        "Operations {} and {} conflict on the same file",
                        idx + 1,
                        other_idx + 1
                    ));
                }
            }

            // Validate paths
            if let Some(warning) = self.validate_operation_path(&op.operation) {
                warnings.push(warning);
            }
        }

        ValidationResult { warnings }
    }

    /// Check if two operations conflict
    fn operations_conflict(&self, op_a: &FileOperation, op_b: &FileOperation) -> bool {
        let path_a = self.get_operation_path(op_a);
        let path_b = self.get_operation_path(op_b);

        if path_a != path_b {
            return false;
        }

        // Multiple updates to same file conflict
        matches!(
            (op_a, op_b),
            (FileOperation::Update { .. }, FileOperation::Update { .. })
                | (FileOperation::Delete { .. }, _)
                | (_, FileOperation::Delete { .. })
        )
    }

    /// Get primary path from operation
    fn get_operation_path(&self, op: &FileOperation) -> PathBuf {
        match op {
            FileOperation::Create { path, .. }
            | FileOperation::Update { path, .. }
            | FileOperation::Append { path, .. }
            | FileOperation::Delete { path } => path.clone(),
            FileOperation::Rename { from, .. } => from.clone(),
        }
    }

    /// Validate operation path
    fn validate_operation_path(&self, op: &FileOperation) -> Option<String> {
        let path = self.get_operation_path(op);

        // Check for suspicious paths
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Some(format!(
                "Path '{}' contains parent directory reference",
                path_str
            ));
        }

        if path.is_absolute() {
            return Some(format!(
                "Path '{}' is absolute, relative paths recommended",
                path_str
            ));
        }

        None
    }

    /// Generate clarification suggestions for low confidence parsing
    fn generate_clarifications(
        &self,
        operations: &[FileOperationWithMetadata],
        unparsed_blocks: &[String],
    ) -> Vec<String> {
        let mut clarifications = Vec::new();

        if operations.is_empty() && !unparsed_blocks.is_empty() {
            clarifications.push(
                "No operations could be parsed. Please use explicit format: ```CREATE:path/to/file"
                    .to_string(),
            );
        }

        for op in operations.iter().filter(|op| op.confidence < 70.0) {
            let path = self.get_operation_path(&op.operation);
            clarifications.push(format!(
                "Operation on '{}' has low confidence. Please verify the operation type and path.",
                path.display()
            ));
        }

        if operations.iter().any(|op| !op.dependencies.is_empty()) {
            clarifications.push(
                "Detected operation dependencies. Please verify the execution order is correct."
                    .to_string(),
            );
        }

        clarifications
    }
}

/// Code block extracted from text
#[derive(Debug, Clone)]
struct CodeBlock {
    header: String,
    content: String,
    location: SourceLocation,
}

/// AI insights from pattern recognition
struct AIInsights {
    warnings: Vec<String>,
    confidence_modifier: f32,
}

/// Validation results
struct ValidationResult {
    warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_explicit_create_operation() {
        let parser = AIOperationParser::new();
        let response = r#"
I'll create a new test file:

```CREATE:src/test.rs
fn main() {
    println!("Hello, world!");
}
```
        "#;

        let context = OperationContext {
            repository_path: PathBuf::from("/test"),
            git_commit: None,
            source_question: "Create a test file".to_string(),
            related_files: vec![],
            project_metadata: HashMap::new(),
        };

        let result = parser.parse_response(response, &context).await.unwrap();

        assert_eq!(result.operations.len(), 1);
        assert!(result.confidence > 85.0);

        match &result.operations[0].operation {
            FileOperation::Create { path, content } => {
                assert_eq!(path.to_str().unwrap(), "src/test.rs");
                assert!(content.contains("Hello, world!"));
            }
            _ => panic!("Expected Create operation"),
        }
    }

    #[tokio::test]
    async fn test_parse_natural_language_operation() {
        let parser = AIOperationParser::new();
        let response = r#"
I'll create a new file called 'config.toml' with the following content:

```toml
[server]
port = 8080
host = "localhost"
```
        "#;

        let context = OperationContext {
            repository_path: PathBuf::from("/test"),
            git_commit: None,
            source_question: "Create config".to_string(),
            related_files: vec![],
            project_metadata: HashMap::new(),
        };

        let result = parser.parse_response(response, &context).await.unwrap();

        assert_eq!(result.operations.len(), 1);
        match &result.operations[0].operation {
            FileOperation::Create { path, content } => {
                assert_eq!(path.to_str().unwrap(), "config.toml");
                assert!(content.contains("port = 8080"));
            }
            _ => panic!("Expected Create operation"),
        }
    }

    #[tokio::test]
    async fn test_parse_multiple_operations() {
        let parser = AIOperationParser::new();
        let response = r#"
First, I'll create the main module:

```CREATE:src/main.rs
mod utils;

fn main() {
    utils::greet();
}
```

Then update the utils module:

```UPDATE:src/utils.rs
pub fn greet() {
    println!("Hello from utils!");
}
```
        "#;

        let context = OperationContext {
            repository_path: PathBuf::from("/test"),
            git_commit: None,
            source_question: "Create modules".to_string(),
            related_files: vec![],
            project_metadata: HashMap::new(),
        };

        let result = parser.parse_response(response, &context).await.unwrap();

        assert_eq!(result.operations.len(), 2);
        assert!(result.operations[0].dependencies.is_empty());
        // Second operation should depend on first if updating existing file
        assert!(result.confidence > 80.0);
    }

    #[test]
    fn test_conflict_detection() {
        let parser = AIOperationParser::new();

        let op1 = FileOperation::Update {
            path: PathBuf::from("test.rs"),
            content: "content1".to_string(),
        };

        let op2 = FileOperation::Update {
            path: PathBuf::from("test.rs"),
            content: "content2".to_string(),
        };

        assert!(parser.operations_conflict(&op1, &op2));

        let op3 = FileOperation::Update {
            path: PathBuf::from("other.rs"),
            content: "content".to_string(),
        };

        assert!(!parser.operations_conflict(&op1, &op3));
    }
}
