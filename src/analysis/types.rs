//! Types for code analysis

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents an analyzed file with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedFile {
    /// Path to the file
    pub path: String,
    /// Language of the file
    pub language: crate::core::Language,
    /// Size in bytes
    pub size: usize,
    /// Line count
    pub line_count: usize,
    /// Whether it's a test file
    pub is_test: bool,
    /// Whether it's a documentation file
    pub is_documentation: bool,
    /// Complexity score (optional)
    pub complexity: Option<f32>,
    /// Code metrics
    pub metrics: FileMetrics,
}

/// File metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileMetrics {
    /// Total lines in the file
    pub lines: usize,
    /// Lines of code (excluding comments and blanks)
    pub code_lines: usize,
    /// Comment lines
    pub comment_lines: usize,
    /// Complexity score
    pub complexity: usize,
}

impl AnalyzedFile {
    /// Create a new analyzed file
    pub fn new(path: String, language: crate::core::Language) -> Self {
        Self {
            path,
            language,
            size: 0,
            line_count: 0,
            is_test: false,
            is_documentation: false,
            complexity: None,
            metrics: FileMetrics::default(),
        }
    }
}
