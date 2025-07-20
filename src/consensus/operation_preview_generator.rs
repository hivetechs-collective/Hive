// Operation Preview Generator
// This module generates before/after previews of file operations to show users
// exactly what changes will be made before execution

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::{debug, info};
use similar::{ChangeTag, TextDiff};
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxSet, SyntaxReference};
use syntect::html::highlighted_html_for_string;

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::ai_operation_parser::FileOperationWithMetadata;
use crate::consensus::operation_analysis::OperationContext;
use crate::core::error::HiveError;

/// Configuration for preview generation
#[derive(Debug, Clone)]
pub struct PreviewConfig {
    /// Maximum lines to show in preview (0 for unlimited)
    pub max_preview_lines: usize,
    
    /// Context lines to show around changes
    pub context_lines: usize,
    
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    
    /// Theme for syntax highlighting
    pub highlight_theme: String,
    
    /// Show line numbers
    pub show_line_numbers: bool,
    
    /// Collapse unchanged sections
    pub collapse_unchanged: bool,
    
    /// Generate unified diff format
    pub unified_diff: bool,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            max_preview_lines: 100,
            context_lines: 3,
            syntax_highlighting: true,
            highlight_theme: "InspiredGitHub".to_string(),
            show_line_numbers: true,
            collapse_unchanged: true,
            unified_diff: true,
        }
    }
}

/// Complete preview set for multiple operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPreviewSet {
    /// Individual operation previews
    pub previews: Vec<OperationPreview>,
    
    /// Summary of all changes
    pub summary: ChangeSummary,
    
    /// Warnings about potential issues
    pub warnings: Vec<PreviewWarning>,
    
    /// Execution order based on dependencies
    pub execution_order: Vec<usize>,
}

/// Preview of a single file operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPreview {
    /// The operation being previewed
    pub operation: FileOperation,
    
    /// Confidence in this operation
    pub confidence: f32,
    
    /// File state before operation
    pub before: FileState,
    
    /// File state after operation
    pub after: FileState,
    
    /// Visual diff between before and after
    pub diff: DiffView,
    
    /// Impact analysis
    pub impact: OperationImpact,
    
    /// Preview metadata
    pub metadata: PreviewMetadata,
}

/// State of a file at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    /// File path
    pub path: PathBuf,
    
    /// Whether file exists
    pub exists: bool,
    
    /// File content (if exists and readable)
    pub content: Option<String>,
    
    /// File size in bytes
    pub size: Option<u64>,
    
    /// File permissions (unix style)
    pub permissions: Option<String>,
    
    /// Last modified time
    pub last_modified: Option<std::time::SystemTime>,
    
    /// Detected language/syntax
    pub language: Option<String>,
}

/// Visual diff representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffView {
    /// Unified diff string
    pub unified_diff: Option<String>,
    
    /// Side-by-side diff chunks
    pub chunks: Vec<DiffChunk>,
    
    /// Syntax highlighted HTML (if enabled)
    pub highlighted_html: Option<DiffHtml>,
    
    /// Line change statistics
    pub stats: LineChangeStats,
}

/// A chunk of diff showing changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChunk {
    /// Starting line in old file
    pub old_start: usize,
    
    /// Number of lines in old file
    pub old_lines: usize,
    
    /// Starting line in new file
    pub new_start: usize,
    
    /// Number of lines in new file  
    pub new_lines: usize,
    
    /// The actual diff lines
    pub lines: Vec<DiffLine>,
    
    /// Context description
    pub context: Option<String>,
}

/// A single line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    /// Line number in old file (if applicable)
    pub old_line_no: Option<usize>,
    
    /// Line number in new file (if applicable)
    pub new_line_no: Option<usize>,
    
    /// Type of change
    pub change_type: LineChangeType,
    
    /// The line content
    pub content: String,
    
    /// Syntax highlighted HTML (if enabled)
    pub highlighted: Option<String>,
}

/// Type of change for a line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineChangeType {
    Added,
    Removed,
    Modified,
    Context,
}

/// Syntax highlighted diff HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHtml {
    /// Full HTML with styling
    pub full_html: String,
    
    /// CSS styles used
    pub styles: String,
    
    /// JavaScript for interactive features
    pub script: Option<String>,
}

/// Statistics about line changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineChangeStats {
    pub lines_added: usize,
    pub lines_removed: usize,
    pub lines_modified: usize,
    pub total_changes: usize,
}

/// Impact analysis of an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationImpact {
    /// Risk level of this operation
    pub risk_level: RiskLevel,
    
    /// Affected functionality
    pub affected_features: Vec<String>,
    
    /// Potential side effects
    pub side_effects: Vec<String>,
    
    /// Dependencies that might be affected
    pub affected_dependencies: Vec<String>,
    
    /// Whether operation is reversible
    pub reversible: bool,
    
    /// Estimated complexity
    pub complexity: ComplexityLevel,
}

/// Risk level assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Trivial,
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Preview metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewMetadata {
    /// When preview was generated
    pub generated_at: std::time::SystemTime,
    
    /// Time taken to generate preview
    pub generation_time_ms: u64,
    
    /// Whether preview is truncated
    pub truncated: bool,
    
    /// Syntax detection confidence
    pub syntax_confidence: f32,
}

/// Warning about potential issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewWarning {
    /// Warning severity
    pub severity: WarningSeverity,
    
    /// Warning message
    pub message: String,
    
    /// Affected file/operation
    pub affected_path: Option<PathBuf>,
    
    /// Suggested action
    pub suggestion: Option<String>,
}

/// Warning severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

/// Summary of all changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSummary {
    /// Total files affected
    pub files_affected: usize,
    
    /// Files created
    pub files_created: usize,
    
    /// Files modified
    pub files_modified: usize,
    
    /// Files deleted
    pub files_deleted: usize,
    
    /// Files renamed
    pub files_renamed: usize,
    
    /// Total lines added
    pub total_lines_added: usize,
    
    /// Total lines removed
    pub total_lines_removed: usize,
    
    /// Estimated risk level
    pub overall_risk: RiskLevel,
    
    /// Estimated complexity
    pub overall_complexity: ComplexityLevel,
}

/// Main preview generator
pub struct OperationPreviewGenerator {
    config: PreviewConfig,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    file_cache: HashMap<PathBuf, String>,
}

impl OperationPreviewGenerator {
    pub fn new(config: PreviewConfig) -> Self {
        Self {
            config,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            file_cache: HashMap::new(),
        }
    }
    
    /// Generate previews for a set of operations
    pub async fn generate_previews(
        &mut self,
        operations: &[FileOperationWithMetadata],
    ) -> Result<OperationPreviewSet> {
        info!("🔍 Generating previews for {} operations", operations.len());
        
        let mut previews = Vec::new();
        let mut warnings = Vec::new();
        
        // Generate preview for each operation
        for (idx, op_meta) in operations.iter().enumerate() {
            match self.generate_operation_preview(op_meta, idx).await {
                Ok(preview) => previews.push(preview),
                Err(e) => {
                    warnings.push(PreviewWarning {
                        severity: WarningSeverity::Error,
                        message: format!("Failed to generate preview: {}", e),
                        affected_path: self.get_operation_path(&op_meta.operation).cloned(),
                        suggestion: Some("Check file accessibility and permissions".to_string()),
                    });
                }
            }
        }
        
        // Calculate execution order based on dependencies
        let execution_order = self.calculate_execution_order(operations);
        
        // Generate summary
        let summary = self.generate_summary(&previews);
        
        // Add any additional warnings
        warnings.extend(self.analyze_warnings(&previews, operations));
        
        Ok(OperationPreviewSet {
            previews,
            summary,
            warnings,
            execution_order,
        })
    }
    
    /// Generate preview for a single operation
    async fn generate_operation_preview(
        &mut self,
        op_meta: &FileOperationWithMetadata,
        index: usize,
    ) -> Result<OperationPreview> {
        let start_time = std::time::Instant::now();
        
        // Get before state
        let before = self.get_file_state(&op_meta.operation, true).await?;
        
        // Simulate operation to get after state
        let after = self.simulate_operation(&op_meta.operation, &before).await?;
        
        // Generate diff
        let diff = self.generate_diff(&before, &after, &op_meta.operation)?;
        
        // Analyze impact
        let impact = self.analyze_impact(&op_meta.operation, &before, &after, &diff);
        
        let generation_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(OperationPreview {
            operation: op_meta.operation.clone(),
            confidence: op_meta.confidence,
            before,
            after,
            diff,
            impact,
            metadata: PreviewMetadata {
                generated_at: std::time::SystemTime::now(),
                generation_time_ms,
                truncated: false, // Will be set if content is truncated
                syntax_confidence: 0.9, // Will be calculated based on detection
            },
        })
    }
    
    /// Get current state of a file
    async fn get_file_state(&mut self, operation: &FileOperation, is_before: bool) -> Result<FileState> {
        let path = match operation {
            FileOperation::Create { path, .. } if is_before => {
                // File doesn't exist before creation
                return Ok(FileState {
                    path: path.clone(),
                    exists: false,
                    content: None,
                    size: None,
                    permissions: None,
                    last_modified: None,
                    language: None,
                });
            }
            FileOperation::Create { path, .. } |
            FileOperation::Update { path, .. } |
            FileOperation::Append { path, .. } |
            FileOperation::Delete { path } => path,
            FileOperation::Rename { from, to } => if is_before { from } else { to },
        };
        
        // Check cache first
        let content = if let Some(cached) = self.file_cache.get(path) {
            Some(cached.clone())
        } else {
            // Try to read file
            match tokio::fs::read_to_string(path).await {
                Ok(content) => {
                    self.file_cache.insert(path.clone(), content.clone());
                    Some(content)
                }
                Err(_) => None,
            }
        };
        
        let metadata = tokio::fs::metadata(path).await.ok();
        
        let language = if let Some(ext) = path.extension() {
            self.detect_language(ext.to_str().unwrap_or(""))
        } else {
            None
        };
        
        Ok(FileState {
            path: path.clone(),
            exists: content.is_some() || metadata.is_some(),
            content,
            size: metadata.as_ref().map(|m| m.len()),
            #[cfg(unix)]
            permissions: metadata.as_ref().map(|m| {
                use std::os::unix::fs::PermissionsExt;
                format!("{:o}", m.permissions().mode())
            }),
            #[cfg(not(unix))]
            permissions: None,
            last_modified: metadata.and_then(|m| m.modified().ok()),
            language,
        })
    }
    
    /// Simulate operation to get after state
    async fn simulate_operation(&self, operation: &FileOperation, before: &FileState) -> Result<FileState> {
        let mut after = before.clone();
        
        match operation {
            FileOperation::Create { path, content } => {
                after.path = path.clone();
                after.exists = true;
                after.content = Some(content.clone());
                after.size = Some(content.len() as u64);
                after.language = if let Some(ext) = path.extension() {
                    self.detect_language(ext.to_str().unwrap_or(""))
                } else {
                    None
                };
            }
            FileOperation::Update { path, content } => {
                after.path = path.clone();
                after.exists = true;
                after.content = Some(content.clone());
                after.size = Some(content.len() as u64);
            }
            FileOperation::Append { path, content } => {
                after.path = path.clone();
                after.exists = true;
                if let Some(existing) = &before.content {
                    let new_content = format!("{}{}", existing, content);
                    after.content = Some(new_content.clone());
                    after.size = Some(new_content.len() as u64);
                } else {
                    after.content = Some(content.clone());
                    after.size = Some(content.len() as u64);
                }
            }
            FileOperation::Delete { path } => {
                after.path = path.clone();
                after.exists = false;
                after.content = None;
                after.size = None;
                after.permissions = None;
                after.language = None;
            }
            FileOperation::Rename { from: _, to } => {
                after.path = to.clone();
                // Content remains the same
            }
        }
        
        Ok(after)
    }
    
    /// Generate diff between before and after states
    fn generate_diff(
        &self,
        before: &FileState,
        after: &FileState,
        operation: &FileOperation,
    ) -> Result<DiffView> {
        let before_content = before.content.as_deref().unwrap_or("");
        let after_content = after.content.as_deref().unwrap_or("");
        
        // Generate unified diff if requested
        let unified_diff = if self.config.unified_diff {
            Some(self.create_unified_diff(
                &before.path,
                &after.path,
                before_content,
                after_content,
            ))
        } else {
            None
        };
        
        // Generate detailed chunks
        let chunks = self.create_diff_chunks(before_content, after_content);
        
        // Generate syntax highlighted HTML if enabled
        let highlighted_html = if self.config.syntax_highlighting {
            self.create_highlighted_diff(before, after, &chunks).ok()
        } else {
            None
        };
        
        // Calculate statistics
        let stats = self.calculate_diff_stats(&chunks);
        
        Ok(DiffView {
            unified_diff,
            chunks,
            highlighted_html,
            stats,
        })
    }
    
    /// Create unified diff format
    fn create_unified_diff(
        &self,
        old_path: &Path,
        new_path: &Path,
        old_content: &str,
        new_content: &str,
    ) -> String {
        let diff = TextDiff::from_lines(old_content, new_content);
        
        format!("{}", diff.unified_diff()
            .context_radius(self.config.context_lines)
            .header(&format!("{}", old_path.display()), &format!("{}", new_path.display())))
    }
    
    /// Create detailed diff chunks
    fn create_diff_chunks(&self, old_content: &str, new_content: &str) -> Vec<DiffChunk> {
        let diff = TextDiff::from_lines(old_content, new_content);
        let mut chunks = Vec::new();
        
        let mut old_line = 1;
        let mut new_line = 1;
        let mut current_chunk: Option<DiffChunk> = None;
        let mut current_lines = Vec::new();
        
        for change in diff.iter_all_changes() {
            let line = DiffLine {
                old_line_no: match change.tag() {
                    ChangeTag::Delete | ChangeTag::Equal => Some(old_line),
                    _ => None,
                },
                new_line_no: match change.tag() {
                    ChangeTag::Insert | ChangeTag::Equal => Some(new_line),
                    _ => None,
                },
                change_type: match change.tag() {
                    ChangeTag::Delete => LineChangeType::Removed,
                    ChangeTag::Insert => LineChangeType::Added,
                    ChangeTag::Equal => LineChangeType::Context,
                },
                content: change.value().trim_end().to_string(),
                highlighted: None,
            };
            
            // Update line numbers
            match change.tag() {
                ChangeTag::Delete => old_line += 1,
                ChangeTag::Insert => new_line += 1,
                ChangeTag::Equal => {
                    old_line += 1;
                    new_line += 1;
                }
            }
            
            // Group into chunks
            if line.change_type != LineChangeType::Context || current_lines.len() < self.config.context_lines * 2 {
                current_lines.push(line);
            } else {
                // Finalize current chunk
                if let Some(mut chunk) = current_chunk.take() {
                    chunk.lines = current_lines.clone();
                    chunk.old_lines = chunk.lines.iter().filter(|l| l.old_line_no.is_some()).count();
                    chunk.new_lines = chunk.lines.iter().filter(|l| l.new_line_no.is_some()).count();
                    chunks.push(chunk);
                }
                
                // Start new chunk
                current_lines.clear();
                current_lines.push(line);
            }
            
            // Initialize chunk if needed
            if current_chunk.is_none() && !current_lines.is_empty() {
                current_chunk = Some(DiffChunk {
                    old_start: current_lines[0].old_line_no.unwrap_or(1),
                    old_lines: 0,
                    new_start: current_lines[0].new_line_no.unwrap_or(1),
                    new_lines: 0,
                    lines: vec![],
                    context: None,
                });
            }
        }
        
        // Finalize last chunk
        if let Some(mut chunk) = current_chunk {
            chunk.lines = current_lines;
            chunk.old_lines = chunk.lines.iter().filter(|l| l.old_line_no.is_some()).count();
            chunk.new_lines = chunk.lines.iter().filter(|l| l.new_line_no.is_some()).count();
            chunks.push(chunk);
        }
        
        // Add context descriptions
        for chunk in &mut chunks {
            chunk.context = self.detect_chunk_context(&chunk.lines);
        }
        
        chunks
    }
    
    /// Create syntax highlighted diff
    fn create_highlighted_diff(
        &self,
        before: &FileState,
        after: &FileState,
        chunks: &[DiffChunk],
    ) -> Result<DiffHtml> {
        let language = after.language.as_deref()
            .or(before.language.as_deref())
            .unwrap_or("txt");
        
        let syntax = self.syntax_set.find_syntax_by_extension(language)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        
        let theme = &self.theme_set.themes[&self.config.highlight_theme];
        
        let mut html = String::new();
        html.push_str("<div class='diff-view'>\n");
        
        for chunk in chunks {
            html.push_str("<div class='diff-chunk'>\n");
            
            for line in &chunk.lines {
                let class = match line.change_type {
                    LineChangeType::Added => "diff-added",
                    LineChangeType::Removed => "diff-removed",
                    LineChangeType::Modified => "diff-modified",
                    LineChangeType::Context => "diff-context",
                };
                
                html.push_str(&format!("<div class='{}'>\n", class));
                
                if self.config.show_line_numbers {
                    html.push_str("<span class='line-numbers'>");
                    if let Some(old_no) = line.old_line_no {
                        html.push_str(&format!("{:4}", old_no));
                    } else {
                        html.push_str("    ");
                    }
                    html.push_str(" ");
                    if let Some(new_no) = line.new_line_no {
                        html.push_str(&format!("{:4}", new_no));
                    } else {
                        html.push_str("    ");
                    }
                    html.push_str("</span>");
                }
                
                let highlighted = highlighted_html_for_string(
                    &line.content,
                    &self.syntax_set,
                    syntax,
                    theme,
                )?;
                
                html.push_str(&highlighted);
                html.push_str("</div>\n");
            }
            
            html.push_str("</div>\n");
        }
        
        html.push_str("</div>\n");
        
        // Generate CSS
        let styles = self.generate_diff_styles();
        
        Ok(DiffHtml {
            full_html: html,
            styles,
            script: None,
        })
    }
    
    /// Calculate diff statistics
    fn calculate_diff_stats(&self, chunks: &[DiffChunk]) -> LineChangeStats {
        let mut stats = LineChangeStats {
            lines_added: 0,
            lines_removed: 0,
            lines_modified: 0,
            total_changes: 0,
        };
        
        for chunk in chunks {
            for line in &chunk.lines {
                match line.change_type {
                    LineChangeType::Added => stats.lines_added += 1,
                    LineChangeType::Removed => stats.lines_removed += 1,
                    LineChangeType::Modified => stats.lines_modified += 1,
                    _ => {}
                }
            }
        }
        
        stats.total_changes = stats.lines_added + stats.lines_removed + stats.lines_modified;
        stats
    }
    
    /// Analyze impact of operation
    fn analyze_impact(
        &self,
        operation: &FileOperation,
        before: &FileState,
        after: &FileState,
        diff: &DiffView,
    ) -> OperationImpact {
        let risk_level = self.assess_risk_level(operation, before, after, diff);
        let complexity = self.assess_complexity(operation, diff);
        let reversible = self.is_operation_reversible(operation, before);
        
        let affected_features = self.detect_affected_features(operation, before, after);
        let side_effects = self.detect_side_effects(operation, before, after);
        let affected_dependencies = self.detect_affected_dependencies(operation, before, after);
        
        OperationImpact {
            risk_level,
            affected_features,
            side_effects,
            affected_dependencies,
            reversible,
            complexity,
        }
    }
    
    /// Assess risk level of operation
    fn assess_risk_level(
        &self,
        operation: &FileOperation,
        before: &FileState,
        after: &FileState,
        diff: &DiffView,
    ) -> RiskLevel {
        match operation {
            FileOperation::Delete { .. } => {
                if before.size.unwrap_or(0) > 10000 {
                    RiskLevel::High
                } else {
                    RiskLevel::Medium
                }
            }
            FileOperation::Update { .. } => {
                if diff.stats.total_changes > 100 {
                    RiskLevel::High
                } else if diff.stats.total_changes > 20 {
                    RiskLevel::Medium
                } else {
                    RiskLevel::Low
                }
            }
            FileOperation::Create { .. } => RiskLevel::Low,
            FileOperation::Append { .. } => RiskLevel::Low,
            FileOperation::Rename { .. } => RiskLevel::Medium,
        }
    }
    
    /// Assess complexity of operation
    fn assess_complexity(&self, operation: &FileOperation, diff: &DiffView) -> ComplexityLevel {
        match operation {
            FileOperation::Delete { .. } | FileOperation::Rename { .. } => ComplexityLevel::Simple,
            FileOperation::Create { content, .. } | FileOperation::Update { content, .. } => {
                let lines = content.lines().count();
                if lines > 500 {
                    ComplexityLevel::VeryComplex
                } else if lines > 100 {
                    ComplexityLevel::Complex
                } else if lines > 20 {
                    ComplexityLevel::Moderate
                } else {
                    ComplexityLevel::Simple
                }
            }
            FileOperation::Append { .. } => ComplexityLevel::Trivial,
        }
    }
    
    /// Check if operation is reversible
    fn is_operation_reversible(&self, operation: &FileOperation, before: &FileState) -> bool {
        match operation {
            FileOperation::Create { .. } => true, // Can delete
            FileOperation::Update { .. } => before.content.is_some(), // Need original content
            FileOperation::Delete { .. } => before.content.is_some(), // Need content to restore
            FileOperation::Rename { .. } => true, // Can rename back
            FileOperation::Append { .. } => false, // Hard to determine what was appended
        }
    }
    
    /// Detect affected features based on file changes
    fn detect_affected_features(
        &self,
        operation: &FileOperation,
        before: &FileState,
        after: &FileState,
    ) -> Vec<String> {
        let mut features = Vec::new();
        
        let path = self.get_operation_path(operation).unwrap_or(&before.path);
        
        // Detect based on path patterns
        if path.to_str().unwrap_or("").contains("auth") {
            features.push("Authentication".to_string());
        }
        if path.to_str().unwrap_or("").contains("api") {
            features.push("API".to_string());
        }
        if path.to_str().unwrap_or("").contains("test") {
            features.push("Testing".to_string());
        }
        if path.to_str().unwrap_or("").contains("config") {
            features.push("Configuration".to_string());
        }
        
        features
    }
    
    /// Detect potential side effects
    fn detect_side_effects(
        &self,
        operation: &FileOperation,
        before: &FileState,
        after: &FileState,
    ) -> Vec<String> {
        let mut effects = Vec::new();
        
        match operation {
            FileOperation::Delete { path } => {
                effects.push("Other files may have broken imports".to_string());
                if path.to_str().unwrap_or("").contains("config") {
                    effects.push("Application configuration may be affected".to_string());
                }
            }
            FileOperation::Rename { from, to } => {
                effects.push(format!("Import statements referencing {} need updating", from.display()));
            }
            FileOperation::Update { path, .. } => {
                if path.to_str().unwrap_or("").contains("schema") {
                    effects.push("Database schema changes may require migration".to_string());
                }
            }
            _ => {}
        }
        
        effects
    }
    
    /// Detect affected dependencies
    fn detect_affected_dependencies(
        &self,
        operation: &FileOperation,
        before: &FileState,
        after: &FileState,
    ) -> Vec<String> {
        let mut deps = Vec::new();
        
        // Simple heuristic - in real implementation would parse imports
        if let Some(content) = &after.content {
            if content.contains("import ") || content.contains("require(") {
                deps.push("Module dependencies".to_string());
            }
        }
        
        deps
    }
    
    /// Detect context of a chunk (function, class, etc.)
    fn detect_chunk_context(&self, lines: &[DiffLine]) -> Option<String> {
        // Look for function/class definitions in context lines
        for line in lines.iter().filter(|l| l.change_type == LineChangeType::Context) {
            let content = &line.content;
            
            if content.contains("fn ") || content.contains("function ") {
                return Some("Function".to_string());
            }
            if content.contains("class ") || content.contains("struct ") {
                return Some("Class/Struct".to_string());
            }
            if content.contains("impl ") {
                return Some("Implementation".to_string());
            }
        }
        
        None
    }
    
    /// Detect language from file extension
    fn detect_language(&self, extension: &str) -> Option<String> {
        match extension {
            "rs" => Some("rust".to_string()),
            "js" => Some("javascript".to_string()),
            "ts" => Some("typescript".to_string()),
            "py" => Some("python".to_string()),
            "go" => Some("go".to_string()),
            "java" => Some("java".to_string()),
            "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
            "c" => Some("c".to_string()),
            "h" | "hpp" => Some("cpp".to_string()),
            "rb" => Some("ruby".to_string()),
            "php" => Some("php".to_string()),
            "swift" => Some("swift".to_string()),
            "kt" => Some("kotlin".to_string()),
            "scala" => Some("scala".to_string()),
            "html" => Some("html".to_string()),
            "css" => Some("css".to_string()),
            "scss" | "sass" => Some("scss".to_string()),
            "json" => Some("json".to_string()),
            "yaml" | "yml" => Some("yaml".to_string()),
            "toml" => Some("toml".to_string()),
            "md" => Some("markdown".to_string()),
            _ => None,
        }
    }
    
    /// Generate CSS styles for diff view
    fn generate_diff_styles(&self) -> String {
        r#"
.diff-view {
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 12px;
    line-height: 1.4;
    background: #f8f8f8;
    border: 1px solid #ddd;
    border-radius: 4px;
    overflow-x: auto;
}

.diff-chunk {
    margin: 10px 0;
    border-top: 1px solid #e0e0e0;
    padding-top: 10px;
}

.diff-chunk:first-child {
    border-top: none;
    padding-top: 0;
}

.diff-added {
    background-color: #e6ffed;
    color: #24292e;
}

.diff-removed {
    background-color: #ffeef0;
    color: #24292e;
}

.diff-context {
    color: #586069;
}

.line-numbers {
    display: inline-block;
    width: 80px;
    color: #959da5;
    text-align: right;
    padding-right: 10px;
    margin-right: 10px;
    border-right: 1px solid #e1e4e8;
    user-select: none;
}

.diff-added .line-numbers {
    background-color: #cdffd8;
}

.diff-removed .line-numbers {
    background-color: #ffdce0;
}
        "#.to_string()
    }
    
    /// Calculate execution order based on dependencies
    fn calculate_execution_order(&self, operations: &[FileOperationWithMetadata]) -> Vec<usize> {
        let mut order = Vec::new();
        let mut visited = vec![false; operations.len()];
        
        // First, add all operations without dependencies
        for (idx, op) in operations.iter().enumerate() {
            if op.dependencies.is_empty() && !visited[idx] {
                order.push(idx);
                visited[idx] = true;
            }
        }
        
        // Then, add operations whose dependencies are satisfied
        while order.len() < operations.len() {
            for (idx, op) in operations.iter().enumerate() {
                if !visited[idx] {
                    let deps_satisfied = op.dependencies.iter()
                        .all(|&dep_idx| visited[dep_idx]);
                    
                    if deps_satisfied {
                        order.push(idx);
                        visited[idx] = true;
                    }
                }
            }
        }
        
        order
    }
    
    /// Generate summary of all changes
    fn generate_summary(&self, previews: &[OperationPreview]) -> ChangeSummary {
        let mut summary = ChangeSummary {
            files_affected: 0,
            files_created: 0,
            files_modified: 0,
            files_deleted: 0,
            files_renamed: 0,
            total_lines_added: 0,
            total_lines_removed: 0,
            overall_risk: RiskLevel::Low,
            overall_complexity: ComplexityLevel::Simple,
        };
        
        let mut paths_seen = std::collections::HashSet::new();
        let mut max_risk = RiskLevel::Low;
        let mut max_complexity = ComplexityLevel::Simple;
        
        for preview in previews {
            let path = self.get_operation_path(&preview.operation);
            if let Some(p) = path {
                paths_seen.insert(p.clone());
            }
            
            match &preview.operation {
                FileOperation::Create { .. } => summary.files_created += 1,
                FileOperation::Update { .. } => summary.files_modified += 1,
                FileOperation::Delete { .. } => summary.files_deleted += 1,
                FileOperation::Rename { .. } => summary.files_renamed += 1,
                FileOperation::Append { .. } => summary.files_modified += 1,
            }
            
            summary.total_lines_added += preview.diff.stats.lines_added;
            summary.total_lines_removed += preview.diff.stats.lines_removed;
            
            // Track highest risk and complexity
            if preview.impact.risk_level as u8 > max_risk as u8 {
                max_risk = preview.impact.risk_level;
            }
            if preview.impact.complexity as u8 > max_complexity as u8 {
                max_complexity = preview.impact.complexity;
            }
        }
        
        summary.files_affected = paths_seen.len();
        summary.overall_risk = max_risk;
        summary.overall_complexity = max_complexity;
        
        summary
    }
    
    /// Analyze and generate additional warnings
    fn analyze_warnings(
        &self,
        previews: &[OperationPreview],
        operations: &[FileOperationWithMetadata],
    ) -> Vec<PreviewWarning> {
        let mut warnings = Vec::new();
        
        // Check for operations on critical files
        for preview in previews {
            if let Some(path) = self.get_operation_path(&preview.operation) {
                let path_str = path.to_str().unwrap_or("");
                
                if path_str.contains(".env") || path_str.contains("secrets") {
                    warnings.push(PreviewWarning {
                        severity: WarningSeverity::Warning,
                        message: "Operation affects sensitive file".to_string(),
                        affected_path: Some(path.clone()),
                        suggestion: Some("Ensure secrets are not exposed".to_string()),
                    });
                }
                
                if matches!(preview.operation, FileOperation::Delete { .. }) && 
                   preview.before.size.unwrap_or(0) > 1000 {
                    warnings.push(PreviewWarning {
                        severity: WarningSeverity::Warning,
                        message: "Deleting large file".to_string(),
                        affected_path: Some(path.clone()),
                        suggestion: Some("Consider backing up before deletion".to_string()),
                    });
                }
            }
        }
        
        // Check for circular dependencies
        for (idx, op) in operations.iter().enumerate() {
            for &dep_idx in &op.dependencies {
                if operations[dep_idx].dependencies.contains(&idx) {
                    warnings.push(PreviewWarning {
                        severity: WarningSeverity::Error,
                        message: "Circular dependency detected".to_string(),
                        affected_path: self.get_operation_path(&op.operation).cloned(),
                        suggestion: Some("Reorder operations to remove circular dependency".to_string()),
                    });
                }
            }
        }
        
        warnings
    }
    
    /// Get path from operation
    fn get_operation_path<'a>(&self, operation: &'a FileOperation) -> Option<&'a PathBuf> {
        match operation {
            FileOperation::Create { path, .. } |
            FileOperation::Update { path, .. } |
            FileOperation::Append { path, .. } |
            FileOperation::Delete { path } => Some(path),
            FileOperation::Rename { from, .. } => Some(from),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_preview_generation() {
        let config = PreviewConfig::default();
        let mut generator = OperationPreviewGenerator::new(config);
        
        let operations = vec![
            FileOperationWithMetadata {
                operation: FileOperation::Create {
                    path: PathBuf::from("test.rs"),
                    content: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
                },
                confidence: 90.0,
                rationale: Some("Creating test file".to_string()),
                dependencies: vec![],
                source_location: crate::consensus::ai_operation_parser::SourceLocation {
                    start: 0,
                    end: 100,
                    line: 1,
                },
            }
        ];
        
        let preview_set = generator.generate_previews(&operations).await.unwrap();
        
        assert_eq!(preview_set.previews.len(), 1);
        assert_eq!(preview_set.summary.files_created, 1);
        assert_eq!(preview_set.summary.files_affected, 1);
    }
}