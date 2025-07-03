//! Core types for the transformation engine

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Represents a code transformation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRequest {
    /// The file path to transform
    pub file_path: PathBuf,
    /// The aspect to improve (e.g., "error-handling", "performance", "readability")
    pub aspect: String,
    /// Optional specific context or instructions
    pub context: Option<String>,
    /// Whether to apply across multiple related files
    pub multi_file: bool,
}

/// Represents a single code change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    /// File path where the change applies
    pub file_path: PathBuf,
    /// Original content before the change
    pub original_content: String,
    /// New content after the change
    pub new_content: String,
    /// Line range affected (start, end)
    pub line_range: (usize, usize),
    /// Description of what changed
    pub description: String,
    /// Confidence score from consensus engine (0.0 to 1.0)
    pub confidence: f64,
}

/// Represents a complete transformation with multiple changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    /// Unique identifier for this transformation
    pub id: String,
    /// Timestamp when created
    pub timestamp: DateTime<Utc>,
    /// The original request
    pub request: TransformationRequest,
    /// All code changes in this transformation
    pub changes: Vec<CodeChange>,
    /// Overall description of the transformation
    pub description: String,
    /// Whether this transformation has been applied
    pub applied: bool,
    /// Transaction ID if applied
    pub transaction_id: Option<String>,
    /// Confidence score of the transformation
    pub confidence: f64,
    /// Impact score of the transformation
    pub impact_score: f64,
    /// Tags associated with the transformation
    pub tags: Vec<String>,
}

/// Result of a transformation preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationPreview {
    /// The transformation that would be applied
    pub transformation: Transformation,
    /// Unified diff for each file
    pub diffs: Vec<FileDiff>,
    /// Any warnings or notes
    pub warnings: Vec<String>,
    /// Estimated impact analysis
    pub impact: ImpactAnalysis,
}

/// Diff information for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_path: PathBuf,
    pub unified_diff: String,
    pub additions: usize,
    pub deletions: usize,
}

/// Impact analysis of a transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// Files directly modified
    pub files_modified: usize,
    /// Files potentially affected
    pub files_affected: usize,
    /// Functions/methods modified
    pub functions_modified: Vec<String>,
    /// Estimated risk level (low, medium, high)
    pub risk_level: RiskLevel,
    /// Whether tests need to be updated
    pub tests_affected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Result of conflict detection
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub file_path: PathBuf,
    pub conflict_type: ConflictType,
    pub description: String,
    pub resolution_options: Vec<ResolutionOption>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    /// File has been modified since analysis
    FileModified,
    /// Multiple transformations affect same lines
    OverlappingChanges,
    /// Transformation would break compilation
    SyntaxBreaking,
    /// Transformation conflicts with another pending transformation
    PendingConflict,
}

#[derive(Debug, Clone)]
pub struct ResolutionOption {
    pub name: String,
    pub description: String,
    pub action: ResolutionAction,
}

#[derive(Debug, Clone)]
pub enum ResolutionAction {
    /// Skip this specific change
    Skip,
    /// Force apply (overwrite)
    Force,
    /// Merge changes
    Merge(String), // merged content
    /// Reanalyze with updated content
    Reanalyze,
}

/// Configuration for transformation behavior
#[derive(Debug, Clone, Deserialize)]
pub struct TransformConfig {
    /// Maximum changes per transformation
    pub max_changes_per_transform: usize,
    /// Whether to preserve formatting exactly
    pub preserve_formatting: bool,
    /// Whether to run validation after each change
    pub validate_after_change: bool,
    /// Minimum confidence score to include a change
    pub min_confidence: f64,
    /// Whether to automatically fix imports
    pub auto_fix_imports: bool,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            max_changes_per_transform: 10,
            preserve_formatting: true,
            validate_after_change: true,
            min_confidence: 0.7,
            auto_fix_imports: true,
        }
    }
}