// Operation Preview System with Before/After Code Generation
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use similar::{ChangeTag, TextDiff};

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::operation_parser::EnhancedFileOperation;
use crate::consensus::operation_clustering::OperationCluster;
use crate::ai_helpers::knowledge_synthesizer::KnowledgeSynthesizer;

/// Generates comprehensive previews of file operations
#[derive(Debug, Clone)]
pub struct OperationPreviewGenerator {
    /// Workspace root for resolving paths
    workspace_root: PathBuf,
    /// AI helper for enhanced previews
    knowledge_synthesizer: Option<Arc<KnowledgeSynthesizer>>,
    /// Configuration for preview generation
    config: PreviewConfig,
    /// Cache for file contents
    file_cache: Arc<RwLock<HashMap<PathBuf, String>>>,
    /// Preview generation cache
    preview_cache: Arc<RwLock<HashMap<String, OperationPreviewSet>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfig {
    /// Maximum lines to show in diff preview
    pub max_diff_lines: usize,
    /// Context lines around changes
    pub context_lines: usize,
    /// Generate syntax-highlighted previews
    pub syntax_highlight: bool,
    /// Include AI-generated explanations
    pub include_ai_explanations: bool,
    /// Show impact analysis
    pub show_impact_analysis: bool,
    /// Generate visual diff
    pub visual_diff: bool,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            max_diff_lines: 100,
            context_lines: 3,
            syntax_highlight: true,
            include_ai_explanations: true,
            show_impact_analysis: true,
            visual_diff: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPreviewSet {
    /// Individual operation previews
    pub previews: Vec<OperationPreview>,
    /// Combined impact analysis
    pub combined_impact: CombinedImpact,
    /// Execution timeline
    pub timeline: ExecutionTimeline,
    /// Visual summary
    pub visual_summary: VisualSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPreview {
    /// The operation being previewed
    pub operation: FileOperation,
    /// File state before operation
    pub before_state: FileState,
    /// File state after operation
    pub after_state: FileState,
    /// Visual diff representation
    pub diff_view: Option<DiffView>,
    /// Impact analysis for this operation
    pub impact: OperationImpact,
    /// AI-generated explanation
    pub ai_explanation: Option<String>,
    /// Preview confidence
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    /// File path
    pub path: PathBuf,
    /// Whether file exists
    pub exists: bool,
    /// File content (if applicable)
    pub content: Option<String>,
    /// File metadata
    pub metadata: FileMetadata,
    /// Syntax-highlighted content
    pub highlighted_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: Option<u64>,
    /// Line count
    pub line_count: Option<usize>,
    /// Language/file type
    pub language: Option<String>,
    /// Encoding
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffView {
    /// Unified diff format
    pub unified_diff: String,
    /// Side-by-side diff chunks
    pub side_by_side: Vec<DiffChunk>,
    /// Line-by-line changes
    pub line_changes: LineChangeSummary,
    /// Visual diff representation
    pub visual_diff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChunk {
    /// Starting line in original file
    pub old_start: usize,
    /// Number of lines in original
    pub old_count: usize,
    /// Starting line in new file
    pub new_start: usize,
    /// Number of lines in new
    pub new_count: usize,
    /// The actual diff lines
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    /// Line type (context, addition, deletion)
    pub line_type: LineType,
    /// Line number in old file
    pub old_line_no: Option<usize>,
    /// Line number in new file
    pub new_line_no: Option<usize>,
    /// The line content
    pub content: String,
    /// Syntax highlighting for this line
    pub highlighted: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LineType {
    Context,
    Addition,
    Deletion,
    FileInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineChangeSummary {
    pub additions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub total_changes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationImpact {
    /// Affected code elements (functions, classes, etc.)
    pub affected_elements: Vec<CodeElement>,
    /// Dependencies that might be affected
    pub affected_dependencies: Vec<String>,
    /// Potential side effects
    pub side_effects: Vec<String>,
    /// Risk level of this change
    pub risk_level: ImpactRiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeElement {
    pub element_type: ElementType,
    pub name: String,
    pub line_range: (usize, usize),
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Function,
    Class,
    Method,
    Variable,
    Import,
    Export,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactRiskLevel {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedImpact {
    /// Total files affected
    pub total_files: usize,
    /// Total lines changed
    pub total_lines_changed: LineChangeSummary,
    /// All affected code elements
    pub all_affected_elements: Vec<CodeElement>,
    /// Overall risk assessment
    pub overall_risk: ImpactRiskLevel,
    /// Execution order importance
    pub order_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTimeline {
    /// Ordered list of operations
    pub steps: Vec<TimelineStep>,
    /// Estimated total time
    pub estimated_duration_ms: u64,
    /// Critical path operations
    pub critical_path: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineStep {
    /// Step index
    pub index: usize,
    /// Operation description
    pub description: String,
    /// Dependencies on previous steps
    pub dependencies: Vec<usize>,
    /// Estimated duration
    pub estimated_duration_ms: u64,
    /// Can be parallelized
    pub parallelizable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualSummary {
    /// ASCII art representation of changes
    pub ascii_visualization: String,
    /// File tree showing changes
    pub file_tree_diff: String,
    /// Change statistics
    pub statistics: ChangeStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStatistics {
    pub files_created: usize,
    pub files_modified: usize,
    pub files_deleted: usize,
    pub files_moved: usize,
    pub total_size_change: i64,
}

impl OperationPreviewGenerator {
    pub fn new(
        workspace_root: PathBuf,
        knowledge_synthesizer: Option<Arc<KnowledgeSynthesizer>>,
        config: Option<PreviewConfig>,
    ) -> Self {
        Self {
            workspace_root,
            knowledge_synthesizer,
            config: config.unwrap_or_default(),
            file_cache: Arc::new(RwLock::new(HashMap::new())),
            preview_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn generate_previews(
        &self,
        operations: &[EnhancedFileOperation],
        clusters: Option<&[OperationCluster]>,
    ) -> Result<OperationPreviewSet> {
        let cache_key = self.generate_cache_key(operations);
        
        // Check cache
        if let Some(cached) = self.preview_cache.read().await.get(&cache_key) {
            debug!("Using cached preview");
            return Ok(cached.clone());
        }

        let mut previews = Vec::new();
        let mut all_elements = Vec::new();
        let mut total_changes = LineChangeSummary {
            additions: 0,
            deletions: 0,
            modifications: 0,
            total_changes: 0,
        };

        // Generate preview for each operation
        for (i, enhanced_op) in operations.iter().enumerate() {
            let preview = self.generate_single_preview(&enhanced_op.operation, i).await?;
            
            // Accumulate impact data
            all_elements.extend(preview.impact.affected_elements.clone());
            if let Some(diff) = &preview.diff_view {
                total_changes.additions += diff.line_changes.additions;
                total_changes.deletions += diff.line_changes.deletions;
                total_changes.modifications += diff.line_changes.modifications;
                total_changes.total_changes += diff.line_changes.total_changes;
            }
            
            previews.push(preview);
        }

        // Generate combined impact analysis
        let combined_impact = self.analyze_combined_impact(&previews, &all_elements, total_changes)?;
        
        // Generate execution timeline
        let timeline = self.generate_timeline(operations, clusters)?;
        
        // Generate visual summary
        let visual_summary = self.generate_visual_summary(&previews)?;

        let preview_set = OperationPreviewSet {
            previews,
            combined_impact,
            timeline,
            visual_summary,
        };

        // Cache the result
        self.preview_cache.write().await.insert(cache_key, preview_set.clone());

        Ok(preview_set)
    }

    async fn generate_single_preview(
        &self,
        operation: &FileOperation,
        index: usize,
    ) -> Result<OperationPreview> {
        let before_state = self.get_before_state(operation).await?;
        let after_state = self.get_after_state(operation, &before_state).await?;
        let diff_view = self.generate_diff_view(&before_state, &after_state)?;
        let impact = self.analyze_operation_impact(operation, &before_state, &after_state)?;
        
        let ai_explanation = if self.config.include_ai_explanations {
            self.generate_ai_explanation(operation, &impact).await?
        } else {
            None
        };

        let confidence = self.calculate_preview_confidence(operation, &before_state);

        Ok(OperationPreview {
            operation: operation.clone(),
            before_state,
            after_state,
            diff_view,
            impact,
            ai_explanation,
            confidence,
        })
    }

    async fn get_before_state(&self, operation: &FileOperation) -> Result<FileState> {
        let path = match operation {
            FileOperation::Create { path, .. } => path,
            FileOperation::Update { path, .. } => path,
            FileOperation::Delete { path } => path,
            FileOperation::Rename { old_path, .. } => old_path,
            FileOperation::Move { source, .. } => source,
        };

        let full_path = self.workspace_root.join(path);
        let exists = tokio::fs::metadata(&full_path).await.is_ok();
        
        let (content, metadata) = if exists {
            let content = self.read_file_cached(&full_path).await?;
            let metadata = self.get_file_metadata(&full_path, &content).await?;
            (Some(content), metadata)
        } else {
            (None, FileMetadata {
                size: None,
                line_count: None,
                language: self.detect_language(path),
                encoding: "UTF-8".to_string(),
            })
        };

        let highlighted_content = if self.config.syntax_highlight && content.is_some() {
            self.syntax_highlight(content.as_ref().unwrap(), &metadata.language).await?
        } else {
            None
        };

        Ok(FileState {
            path: path.clone(),
            exists,
            content,
            metadata,
            highlighted_content,
        })
    }

    async fn get_after_state(&self, operation: &FileOperation, before: &FileState) -> Result<FileState> {
        match operation {
            FileOperation::Create { path, content } => {
                let metadata = FileMetadata {
                    size: Some(content.len() as u64),
                    line_count: Some(content.lines().count()),
                    language: self.detect_language(path),
                    encoding: "UTF-8".to_string(),
                };

                let highlighted_content = if self.config.syntax_highlight {
                    self.syntax_highlight(content, &metadata.language).await?
                } else {
                    None
                };

                Ok(FileState {
                    path: path.clone(),
                    exists: true,
                    content: Some(content.clone()),
                    metadata,
                    highlighted_content,
                })
            }
            FileOperation::Update { path, new_content, .. } => {
                let metadata = FileMetadata {
                    size: Some(new_content.len() as u64),
                    line_count: Some(new_content.lines().count()),
                    language: before.metadata.language.clone(),
                    encoding: before.metadata.encoding.clone(),
                };

                let highlighted_content = if self.config.syntax_highlight {
                    self.syntax_highlight(new_content, &metadata.language).await?
                } else {
                    None
                };

                Ok(FileState {
                    path: path.clone(),
                    exists: true,
                    content: Some(new_content.clone()),
                    metadata,
                    highlighted_content,
                })
            }
            FileOperation::Delete { path } => {
                Ok(FileState {
                    path: path.clone(),
                    exists: false,
                    content: None,
                    metadata: FileMetadata {
                        size: None,
                        line_count: None,
                        language: None,
                        encoding: "UTF-8".to_string(),
                    },
                    highlighted_content: None,
                })
            }
            FileOperation::Rename { new_path, .. } => {
                let mut after = before.clone();
                after.path = new_path.clone();
                Ok(after)
            }
            FileOperation::Move { destination, .. } => {
                let mut after = before.clone();
                after.path = destination.clone();
                Ok(after)
            }
        }
    }

    fn generate_diff_view(&self, before: &FileState, after: &FileState) -> Result<Option<DiffView>> {
        let (old_content, new_content) = match (&before.content, &after.content) {
            (Some(old), Some(new)) => (old, new),
            (None, Some(new)) => ("", new),
            (Some(old), None) => (old, ""),
            (None, None) => return Ok(None),
        };

        // Generate unified diff
        let diff = TextDiff::from_lines(old_content, new_content);
        let unified_diff = diff
            .unified_diff()
            .context_radius(self.config.context_lines)
            .header(&format!("--- {}", before.path.display()), &format!("+++ {}", after.path.display()))
            .to_string();

        // Generate side-by-side chunks
        let mut chunks = Vec::new();
        let mut current_chunk: Option<DiffChunk> = None;
        let mut old_line = 1;
        let mut new_line = 1;

        for change in diff.iter_all_changes() {
            let tag = change.tag();
            let line_content = change.to_string_lossy();
            
            let diff_line = match tag {
                ChangeTag::Equal => {
                    let line = DiffLine {
                        line_type: LineType::Context,
                        old_line_no: Some(old_line),
                        new_line_no: Some(new_line),
                        content: line_content.to_string(),
                        highlighted: None,
                    };
                    old_line += 1;
                    new_line += 1;
                    line
                }
                ChangeTag::Delete => {
                    let line = DiffLine {
                        line_type: LineType::Deletion,
                        old_line_no: Some(old_line),
                        new_line_no: None,
                        content: line_content.to_string(),
                        highlighted: None,
                    };
                    old_line += 1;
                    line
                }
                ChangeTag::Insert => {
                    let line = DiffLine {
                        line_type: LineType::Addition,
                        old_line_no: None,
                        new_line_no: Some(new_line),
                        content: line_content.to_string(),
                        highlighted: None,
                    };
                    new_line += 1;
                    line
                }
            };

            // Group into chunks
            match (&mut current_chunk, tag) {
                (None, ChangeTag::Equal) => {
                    // Start context before change
                    current_chunk = Some(DiffChunk {
                        old_start: old_line - 1,
                        old_count: 1,
                        new_start: new_line - 1,
                        new_count: 1,
                        lines: vec![diff_line],
                    });
                }
                (Some(chunk), _) => {
                    chunk.lines.push(diff_line);
                    chunk.old_count = old_line - chunk.old_start;
                    chunk.new_count = new_line - chunk.new_start;
                }
                (None, _) => {
                    current_chunk = Some(DiffChunk {
                        old_start: old_line - 1,
                        old_count: 0,
                        new_start: new_line - 1,
                        new_count: 0,
                        lines: vec![diff_line],
                    });
                }
            }
        }

        if let Some(chunk) = current_chunk {
            chunks.push(chunk);
        }

        // Calculate line change summary
        let additions = diff.iter_all_changes().filter(|c| c.tag() == ChangeTag::Insert).count();
        let deletions = diff.iter_all_changes().filter(|c| c.tag() == ChangeTag::Delete).count();
        let modifications = additions.min(deletions);
        let total_changes = additions + deletions;

        let line_changes = LineChangeSummary {
            additions: additions - modifications,
            deletions: deletions - modifications,
            modifications,
            total_changes,
        };

        // Generate visual diff if enabled
        let visual_diff = if self.config.visual_diff {
            Some(self.generate_visual_diff(&diff)?)
        } else {
            None
        };

        Ok(Some(DiffView {
            unified_diff,
            side_by_side: chunks,
            line_changes,
            visual_diff,
        }))
    }

    fn generate_visual_diff(&self, diff: &TextDiff<str>) -> Result<String> {
        let mut visual = String::new();
        
        for (idx, group) in diff.grouped_ops(self.config.context_lines).iter().enumerate() {
            if idx > 0 {
                visual.push_str("\n...\n");
            }

            for op in group {
                for change in diff.iter_changes(op) {
                    let sign = match change.tag() {
                        ChangeTag::Delete => "-",
                        ChangeTag::Insert => "+",
                        ChangeTag::Equal => " ",
                    };
                    visual.push_str(&format!("{} {}", sign, change));
                }
            }
        }

        Ok(visual)
    }

    fn analyze_operation_impact(
        &self,
        operation: &FileOperation,
        before: &FileState,
        after: &FileState,
    ) -> Result<OperationImpact> {
        let mut affected_elements = Vec::new();
        let mut side_effects = Vec::new();

        // Analyze based on operation type
        match operation {
            FileOperation::Create { path, content } => {
                affected_elements.extend(self.extract_code_elements(content, path)?);
                side_effects.push("New file added to project".to_string());
            }
            FileOperation::Update { path, old_content, new_content } => {
                let old_elements = self.extract_code_elements(old_content, path)?;
                let new_elements = self.extract_code_elements(new_content, path)?;
                
                // Find changed elements
                for new_elem in &new_elements {
                    if !old_elements.iter().any(|old| old.name == new_elem.name) {
                        affected_elements.push(new_elem.clone());
                    }
                }
                
                side_effects.push("Existing functionality modified".to_string());
            }
            FileOperation::Delete { path } => {
                if let Some(content) = &before.content {
                    affected_elements.extend(self.extract_code_elements(content, path)?);
                }
                side_effects.push("File and all its contents will be removed".to_string());
            }
            FileOperation::Rename { old_path, new_path } => {
                side_effects.push(format!("All imports of {} need updating", old_path.display()));
            }
            FileOperation::Move { source, destination } => {
                side_effects.push(format!("All imports of {} need updating", source.display()));
            }
        }

        // Determine risk level
        let risk_level = match operation {
            FileOperation::Delete { .. } => ImpactRiskLevel::High,
            FileOperation::Update { .. } if affected_elements.len() > 5 => ImpactRiskLevel::Medium,
            FileOperation::Create { .. } => ImpactRiskLevel::Low,
            _ => ImpactRiskLevel::Low,
        };

        Ok(OperationImpact {
            affected_elements,
            affected_dependencies: Vec::new(), // Could be enhanced with dependency analysis
            side_effects,
            risk_level,
        })
    }

    fn extract_code_elements(&self, content: &str, path: &Path) -> Result<Vec<CodeElement>> {
        let mut elements = Vec::new();
        let language = self.detect_language(path);

        // Simple pattern-based extraction (could be enhanced with tree-sitter)
        match language.as_deref() {
            Some("rust") => {
                // Extract Rust functions
                let fn_regex = regex::Regex::new(r"(?m)^[\s]*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)")?;
                for (line_no, line) in content.lines().enumerate() {
                    if let Some(caps) = fn_regex.captures(line) {
                        elements.push(CodeElement {
                            element_type: ElementType::Function,
                            name: caps[1].to_string(),
                            line_range: (line_no + 1, line_no + 1),
                            description: None,
                        });
                    }
                }
            }
            Some("javascript") | Some("typescript") => {
                // Extract JS/TS functions and classes
                let fn_regex = regex::Regex::new(r"(?:function|const|let|var)\s+(\w+)\s*[=\(]")?;
                let class_regex = regex::Regex::new(r"class\s+(\w+)")?;
                
                for (line_no, line) in content.lines().enumerate() {
                    if let Some(caps) = fn_regex.captures(line) {
                        elements.push(CodeElement {
                            element_type: ElementType::Function,
                            name: caps[1].to_string(),
                            line_range: (line_no + 1, line_no + 1),
                            description: None,
                        });
                    } else if let Some(caps) = class_regex.captures(line) {
                        elements.push(CodeElement {
                            element_type: ElementType::Class,
                            name: caps[1].to_string(),
                            line_range: (line_no + 1, line_no + 1),
                            description: None,
                        });
                    }
                }
            }
            _ => {
                // Generic extraction for other languages
            }
        }

        Ok(elements)
    }

    async fn generate_ai_explanation(
        &self,
        operation: &FileOperation,
        impact: &OperationImpact,
    ) -> Result<Option<String>> {
        if let Some(synthesizer) = &self.knowledge_synthesizer {
            // In a real implementation, this would use the AI helper
            // For now, generate a template-based explanation
            let explanation = match operation {
                FileOperation::Create { path, .. } => {
                    format!(
                        "Creating new file '{}' which will introduce {} new code elements. {}",
                        path.display(),
                        impact.affected_elements.len(),
                        if impact.risk_level == ImpactRiskLevel::Low {
                            "This is a safe operation with minimal risk."
                        } else {
                            "Please review the new functionality carefully."
                        }
                    )
                }
                FileOperation::Update { path, .. } => {
                    format!(
                        "Updating '{}' will modify {} code elements. {}",
                        path.display(),
                        impact.affected_elements.len(),
                        match impact.risk_level {
                            ImpactRiskLevel::High => "This is a high-risk change that could affect system behavior.",
                            ImpactRiskLevel::Medium => "This change has moderate risk and should be tested.",
                            _ => "This appears to be a routine update.",
                        }
                    )
                }
                FileOperation::Delete { path } => {
                    format!(
                        "Deleting '{}' will remove {} code elements. This is a high-risk operation that cannot be easily undone.",
                        path.display(),
                        impact.affected_elements.len()
                    )
                }
                _ => "This operation will modify the file structure.".to_string(),
            };

            Ok(Some(explanation))
        } else {
            Ok(None)
        }
    }

    fn calculate_preview_confidence(&self, operation: &FileOperation, before: &FileState) -> f32 {
        match operation {
            FileOperation::Create { .. } => 0.95, // High confidence for creates
            FileOperation::Update { .. } if before.exists => 0.90, // Good confidence for updates
            FileOperation::Delete { .. } if before.exists => 0.95, // High confidence for deletes
            FileOperation::Rename { .. } | FileOperation::Move { .. } if before.exists => 0.85,
            _ => 0.50, // Lower confidence if file doesn't exist
        }
    }

    async fn read_file_cached(&self, path: &Path) -> Result<String> {
        // Check cache first
        if let Some(content) = self.file_cache.read().await.get(path) {
            return Ok(content.clone());
        }

        // Read from disk
        let content = tokio::fs::read_to_string(path).await?;
        
        // Cache it
        self.file_cache.write().await.insert(path.to_path_buf(), content.clone());
        
        Ok(content)
    }

    async fn get_file_metadata(&self, path: &Path, content: &str) -> Result<FileMetadata> {
        let metadata = tokio::fs::metadata(path).await?;
        
        Ok(FileMetadata {
            size: Some(metadata.len()),
            line_count: Some(content.lines().count()),
            language: self.detect_language(path),
            encoding: "UTF-8".to_string(), // Could be enhanced with encoding detection
        })
    }

    fn detect_language(&self, path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext {
                "rs" => "rust",
                "js" => "javascript",
                "ts" => "typescript",
                "py" => "python",
                "go" => "go",
                "java" => "java",
                "cpp" | "cc" | "cxx" => "cpp",
                "c" => "c",
                "rb" => "ruby",
                "php" => "php",
                "swift" => "swift",
                "kt" => "kotlin",
                "scala" => "scala",
                "sh" | "bash" => "bash",
                "yml" | "yaml" => "yaml",
                "json" => "json",
                "xml" => "xml",
                "html" => "html",
                "css" => "css",
                "scss" | "sass" => "scss",
                "md" => "markdown",
                _ => ext,
            }.to_string())
    }

    async fn syntax_highlight(&self, content: &str, language: &Option<String>) -> Result<Option<String>> {
        // In a real implementation, this would use a syntax highlighting library
        // For now, just return None
        Ok(None)
    }

    fn analyze_combined_impact(
        &self,
        previews: &[OperationPreview],
        all_elements: &[CodeElement],
        total_changes: LineChangeSummary,
    ) -> Result<CombinedImpact> {
        let total_files = previews.len();
        
        // Determine overall risk
        let high_risk_count = previews.iter()
            .filter(|p| matches!(p.impact.risk_level, ImpactRiskLevel::High))
            .count();
        
        let overall_risk = if high_risk_count > 0 {
            ImpactRiskLevel::High
        } else if previews.iter().any(|p| matches!(p.impact.risk_level, ImpactRiskLevel::Medium)) {
            ImpactRiskLevel::Medium
        } else if total_files > 10 {
            ImpactRiskLevel::Medium
        } else {
            ImpactRiskLevel::Low
        };

        // Check if order is important
        let order_sensitive = previews.iter().enumerate().any(|(i, preview)| {
            match &preview.operation {
                FileOperation::Update { path, .. } => {
                    // Check if any earlier operation creates this file
                    previews[..i].iter().any(|p| matches!(&p.operation, FileOperation::Create { path: p, .. } if p == path))
                }
                _ => false,
            }
        });

        Ok(CombinedImpact {
            total_files,
            total_lines_changed: total_changes,
            all_affected_elements: all_elements.to_vec(),
            overall_risk,
            order_sensitive,
        })
    }

    fn generate_timeline(
        &self,
        operations: &[EnhancedFileOperation],
        clusters: Option<&[OperationCluster]>,
    ) -> Result<ExecutionTimeline> {
        let mut steps = Vec::new();
        let mut critical_path = Vec::new();

        for (i, op) in operations.iter().enumerate() {
            let dependencies = op.context.dependencies.clone();
            let is_critical = matches!(op.operation, FileOperation::Delete { .. }) ||
                           op.context.dependencies.len() > 2;
            
            if is_critical {
                critical_path.push(i);
            }

            steps.push(TimelineStep {
                index: i,
                description: self.describe_operation(&op.operation),
                dependencies,
                estimated_duration_ms: self.estimate_duration(&op.operation),
                parallelizable: dependencies.is_empty(),
            });
        }

        let estimated_duration_ms = steps.iter()
            .map(|s| s.estimated_duration_ms)
            .sum();

        Ok(ExecutionTimeline {
            steps,
            estimated_duration_ms,
            critical_path,
        })
    }

    fn describe_operation(&self, operation: &FileOperation) -> String {
        match operation {
            FileOperation::Create { path, .. } => format!("Create {}", path.display()),
            FileOperation::Update { path, .. } => format!("Update {}", path.display()),
            FileOperation::Delete { path } => format!("Delete {}", path.display()),
            FileOperation::Rename { old_path, new_path } => {
                format!("Rename {} to {}", old_path.display(), new_path.display())
            }
            FileOperation::Move { source, destination } => {
                format!("Move {} to {}", source.display(), destination.display())
            }
        }
    }

    fn estimate_duration(&self, operation: &FileOperation) -> u64 {
        match operation {
            FileOperation::Create { content, .. } => {
                10 + (content.len() as u64 / 1000) // Base 10ms + 1ms per KB
            }
            FileOperation::Update { .. } => 15,
            FileOperation::Delete { .. } => 5,
            FileOperation::Rename { .. } => 10,
            FileOperation::Move { .. } => 15,
        }
    }

    fn generate_visual_summary(&self, previews: &[OperationPreview]) -> Result<VisualSummary> {
        let mut stats = ChangeStatistics {
            files_created: 0,
            files_modified: 0,
            files_deleted: 0,
            files_moved: 0,
            total_size_change: 0,
        };

        for preview in previews {
            match &preview.operation {
                FileOperation::Create { .. } => stats.files_created += 1,
                FileOperation::Update { .. } => stats.files_modified += 1,
                FileOperation::Delete { .. } => stats.files_deleted += 1,
                FileOperation::Rename { .. } | FileOperation::Move { .. } => stats.files_moved += 1,
            }

            // Calculate size change
            let before_size = preview.before_state.metadata.size.unwrap_or(0) as i64;
            let after_size = preview.after_state.metadata.size.unwrap_or(0) as i64;
            stats.total_size_change += after_size - before_size;
        }

        // Generate ASCII visualization
        let ascii_visualization = format!(
            r#"
File Operations Summary
======================
📁 Created:  {} file(s)
✏️  Modified: {} file(s)
🗑️  Deleted:  {} file(s)
📦 Moved:    {} file(s)

Total size change: {} bytes
"#,
            stats.files_created,
            stats.files_modified,
            stats.files_deleted,
            stats.files_moved,
            stats.total_size_change
        );

        // Generate file tree diff (simplified)
        let file_tree_diff = self.generate_file_tree_diff(previews)?;

        Ok(VisualSummary {
            ascii_visualization,
            file_tree_diff,
            statistics: stats,
        })
    }

    fn generate_file_tree_diff(&self, previews: &[OperationPreview]) -> Result<String> {
        let mut tree = String::from("Project Structure Changes:\n");
        
        // Group by directory
        let mut dirs: HashMap<String, Vec<String>> = HashMap::new();
        
        for preview in previews {
            let (indicator, path) = match &preview.operation {
                FileOperation::Create { path, .. } => ("+", path),
                FileOperation::Update { path, .. } => ("~", path),
                FileOperation::Delete { path } => ("-", path),
                FileOperation::Rename { old_path, new_path } => {
                    tree.push_str(&format!("  {} {} -> {}\n", "→", old_path.display(), new_path.display()));
                    continue;
                }
                FileOperation::Move { source, destination } => {
                    tree.push_str(&format!("  {} {} -> {}\n", "→", source.display(), destination.display()));
                    continue;
                }
            };

            let dir = path.parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());
            
            let filename = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            dirs.entry(dir).or_default().push(format!("{} {}", indicator, filename));
        }

        // Format tree
        for (dir, files) in dirs {
            tree.push_str(&format!("\n{}:\n", dir));
            for file in files {
                tree.push_str(&format!("  {}\n", file));
            }
        }

        Ok(tree)
    }

    fn generate_cache_key(&self, operations: &[EnhancedFileOperation]) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        for op in operations {
            format!("{:?}", op.operation).hash(&mut hasher);
        }
        
        format!("preview_{:x}", hasher.finish())
    }

    pub async fn clear_cache(&self) {
        self.file_cache.write().await.clear();
        self.preview_cache.write().await.clear();
        debug!("Preview caches cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_preview_generation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let generator = OperationPreviewGenerator::new(
            temp_dir.path().to_path_buf(),
            None,
            None,
        );

        // Create test file
        let test_path = temp_dir.path().join("test.rs");
        tokio::fs::write(&test_path, "fn main() {\n    println!(\"Hello\");\n}").await.unwrap();

        // Test update operation
        let operation = FileOperation::Update {
            path: PathBuf::from("test.rs"),
            old_content: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
            new_content: "fn main() {\n    println!(\"Hello, World!\");\n}".to_string(),
        };

        let enhanced_op = EnhancedFileOperation {
            operation,
            context: Default::default(),
            parsing_confidence: 0.9,
        };

        let preview_set = generator.generate_previews(&[enhanced_op], None).await.unwrap();
        
        assert_eq!(preview_set.previews.len(), 1);
        let preview = &preview_set.previews[0];
        
        assert!(preview.before_state.exists);
        assert!(preview.after_state.exists);
        assert!(preview.diff_view.is_some());
        
        let diff = preview.diff_view.as_ref().unwrap();
        assert!(diff.line_changes.modifications > 0);
    }

    #[test]
    fn test_language_detection() {
        let generator = OperationPreviewGenerator::new(
            PathBuf::from("/test"),
            None,
            None,
        );

        assert_eq!(generator.detect_language(&PathBuf::from("test.rs")), Some("rust".to_string()));
        assert_eq!(generator.detect_language(&PathBuf::from("test.js")), Some("javascript".to_string()));
        assert_eq!(generator.detect_language(&PathBuf::from("test.py")), Some("python".to_string()));
    }
}