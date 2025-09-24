// Advanced File Operation Parser with AI Context Enhancement
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::ai_helpers::knowledge_synthesizer::KnowledgeSynthesizer;
use crate::ai_helpers::pattern_recognizer::PatternRecognizer;
use crate::consensus::stages::file_aware_curator::FileOperation;

#[derive(Debug, Clone)]
pub struct OperationParser {
    /// AI helpers for context enhancement
    knowledge_synthesizer: Option<Arc<KnowledgeSynthesizer>>,
    pattern_recognizer: Option<Arc<PatternRecognizer>>,
    /// Parsing patterns
    patterns: ParsingPatterns,
    /// Context enhancement settings
    enhancement_config: EnhancementConfig,
    /// Cache for parsed results
    parse_cache: Arc<RwLock<HashMap<String, ParsedOperations>>>,
}

#[derive(Debug, Clone)]
struct ParsingPatterns {
    // File operation patterns
    create_file: Regex,
    update_file: Regex,
    delete_file: Regex,
    rename_file: Regex,
    move_file: Regex,

    // Code block patterns
    code_block: Regex,
    diff_block: Regex,

    // Context patterns
    explanation: Regex,
    intention: Regex,
    warning: Regex,
}

impl Default for ParsingPatterns {
    fn default() -> Self {
        Self {
            // Match patterns like "Create file: path/to/file.rs"
            create_file: Regex::new(r"(?i)create\s+(?:new\s+)?file[:\s]+([^\n]+)").unwrap(),

            // Match patterns like "Update file: path/to/file.rs" or "Modify: path/to/file.rs"
            update_file: Regex::new(r"(?i)(?:update|modify|change|edit)\s+(?:file[:\s]+)?([^\n]+)")
                .unwrap(),

            // Match patterns like "Delete file: path/to/file.rs"
            delete_file: Regex::new(r"(?i)(?:delete|remove)\s+(?:file[:\s]+)?([^\n]+)").unwrap(),

            // Match patterns like "Rename file: old.rs -> new.rs"
            rename_file: Regex::new(r"(?i)rename\s+(?:file[:\s]+)?([^\s]+)\s*(?:->|to)\s*([^\n]+)")
                .unwrap(),

            // Match patterns like "Move file: src/old.rs -> dest/new.rs"
            move_file: Regex::new(r"(?i)move\s+(?:file[:\s]+)?([^\s]+)\s*(?:->|to)\s*([^\n]+)")
                .unwrap(),

            // Match code blocks
            code_block: Regex::new(r"```(?:[\w]+)?\n([\s\S]*?)```").unwrap(),

            // Match diff blocks
            diff_block: Regex::new(r"(?:^|\n)((?:[-+].*\n)+)").unwrap(),

            // Match explanations
            explanation: Regex::new(r"(?i)(?:explanation|reason|because)[:\s]+([^\n]+)").unwrap(),

            // Match intentions
            intention: Regex::new(r"(?i)(?:to|in order to|for|will)[:\s]+([^\n]+)").unwrap(),

            // Match warnings
            warning: Regex::new(r"(?i)(?:warning|caution|note|important)[:\s]+([^\n]+)").unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementConfig {
    /// Extract semantic context from descriptions
    pub extract_semantic_context: bool,
    /// Infer operation dependencies
    pub infer_dependencies: bool,
    /// Detect operation intentions
    pub detect_intentions: bool,
    /// Extract code snippets
    pub extract_code_snippets: bool,
    /// Enhance with AI suggestions
    pub ai_enhancement: bool,
}

impl Default for EnhancementConfig {
    fn default() -> Self {
        Self {
            extract_semantic_context: true,
            infer_dependencies: true,
            detect_intentions: true,
            extract_code_snippets: true,
            ai_enhancement: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedOperations {
    /// Extracted file operations
    pub operations: Vec<EnhancedFileOperation>,
    /// Global context from the curator response
    pub global_context: GlobalContext,
    /// Parsing metadata
    pub metadata: ParsingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedFileOperation {
    /// The base file operation
    pub operation: FileOperation,
    /// Enhanced context for this operation
    pub context: OperationContext,
    /// Confidence in the parsing (0.0-1.0)
    pub parsing_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationContext {
    /// Why this operation is being performed
    pub intention: Option<String>,
    /// Explanation provided by curator
    pub explanation: Option<String>,
    /// Any warnings or notes
    pub warnings: Vec<String>,
    /// Related code snippets
    pub code_context: Vec<CodeContext>,
    /// Inferred dependencies on other operations
    pub dependencies: Vec<usize>, // Indices of dependent operations
    /// Semantic tags extracted from context
    pub semantic_tags: Vec<String>,
    /// AI-enhanced insights
    pub ai_insights: Option<AIOperationInsights>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    /// Language of the code
    pub language: Option<String>,
    /// The code snippet
    pub code: String,
    /// Whether this is a diff
    pub is_diff: bool,
    /// Start line in the file (if known)
    pub start_line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalContext {
    /// Overall intention of the operations
    pub overall_intention: Option<String>,
    /// Key phrases extracted from the response
    pub key_phrases: Vec<String>,
    /// Detected operation pattern (refactoring, feature add, bug fix, etc.)
    pub operation_pattern: Option<String>,
    /// Estimated complexity
    pub complexity_estimate: ComplexityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Simple,   // 1-3 straightforward operations
    Moderate, // 4-10 operations or some complexity
    Complex,  // Many operations or high interdependency
    Critical, // System-wide changes or high risk
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsingMetadata {
    /// Number of operations found
    pub operation_count: usize,
    /// Number of ambiguous sections
    pub ambiguous_sections: usize,
    /// Overall parsing confidence
    pub overall_confidence: f32,
    /// Parsing warnings
    pub warnings: Vec<String>,
    /// Time taken to parse (ms)
    pub parse_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIOperationInsights {
    /// Predicted impact of this operation
    pub impact_prediction: String,
    /// Suggested improvements
    pub improvements: Vec<String>,
    /// Risk factors specific to this operation
    pub risk_factors: Vec<String>,
    /// Related best practices
    pub best_practices: Vec<String>,
}

impl OperationParser {
    pub fn new(
        knowledge_synthesizer: Option<Arc<KnowledgeSynthesizer>>,
        pattern_recognizer: Option<Arc<PatternRecognizer>>,
        config: Option<EnhancementConfig>,
    ) -> Self {
        Self {
            knowledge_synthesizer,
            pattern_recognizer,
            patterns: ParsingPatterns::default(),
            enhancement_config: config.unwrap_or_default(),
            parse_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn parse_curator_response(&self, response: &str) -> Result<ParsedOperations> {
        let start_time = std::time::Instant::now();

        // Check cache
        if let Some(cached) = self.parse_cache.read().await.get(response) {
            debug!("Using cached parse result");
            return Ok(cached.clone());
        }

        // Extract global context first
        let global_context = self.extract_global_context(response)?;

        // Parse operations
        let mut operations = Vec::new();
        let mut warnings = Vec::new();
        let mut ambiguous_sections = 0;

        // Split response into sections for better parsing
        let sections = self.split_into_sections(response);

        for (i, section) in sections.iter().enumerate() {
            match self.parse_section(section, i, &sections).await {
                Ok(mut section_ops) => operations.append(&mut section_ops),
                Err(e) => {
                    warnings.push(format!("Failed to parse section {}: {}", i, e));
                    ambiguous_sections += 1;
                }
            }
        }

        // Enhance operations with AI if enabled
        if self.enhancement_config.ai_enhancement {
            operations = self.enhance_with_ai(operations, &global_context).await?;
        }

        // Infer dependencies if enabled
        if self.enhancement_config.infer_dependencies {
            self.infer_operation_dependencies(&mut operations)?;
        }

        // Calculate overall confidence
        let overall_confidence = if operations.is_empty() {
            0.0
        } else {
            operations
                .iter()
                .map(|op| op.parsing_confidence)
                .sum::<f32>()
                / operations.len() as f32
        };

        let metadata = ParsingMetadata {
            operation_count: operations.len(),
            ambiguous_sections,
            overall_confidence,
            warnings,
            parse_time_ms: start_time.elapsed().as_millis() as u64,
        };

        let result = ParsedOperations {
            operations,
            global_context,
            metadata,
        };

        // Cache the result
        self.parse_cache
            .write()
            .await
            .insert(response.to_string(), result.clone());

        Ok(result)
    }

    fn extract_global_context(&self, response: &str) -> Result<GlobalContext> {
        let mut key_phrases = Vec::new();
        let mut overall_intention = None;
        let mut operation_pattern = None;

        // Look for intention patterns
        if let Some(caps) = self.patterns.intention.captures(response) {
            overall_intention = Some(caps[1].trim().to_string());
        }

        // Extract key phrases
        let important_words = [
            "refactor",
            "implement",
            "fix",
            "update",
            "migrate",
            "optimize",
            "add",
            "remove",
        ];
        for word in &important_words {
            if response.to_lowercase().contains(word) {
                key_phrases.push(word.to_string());
            }
        }

        // Detect operation pattern
        let response_lower = response.to_lowercase();
        operation_pattern = if response_lower.contains("refactor") {
            Some("Refactoring".to_string())
        } else if response_lower.contains("bug") || response_lower.contains("fix") {
            Some("Bug Fix".to_string())
        } else if response_lower.contains("feature") || response_lower.contains("implement") {
            Some("Feature Addition".to_string())
        } else if response_lower.contains("migrate") {
            Some("Migration".to_string())
        } else if response_lower.contains("test") {
            Some("Test Addition".to_string())
        } else {
            None
        };

        // Estimate complexity based on indicators
        let complexity_estimate = self.estimate_complexity(response);

        Ok(GlobalContext {
            overall_intention,
            key_phrases,
            operation_pattern,
            complexity_estimate,
        })
    }

    fn estimate_complexity(&self, response: &str) -> ComplexityLevel {
        let operation_count = self.patterns.create_file.find_iter(response).count()
            + self.patterns.update_file.find_iter(response).count()
            + self.patterns.delete_file.find_iter(response).count();

        let has_complex_patterns = response.contains("refactor")
            || response.contains("migrate")
            || response.contains("system-wide");

        let has_warnings = self.patterns.warning.is_match(response);

        match (operation_count, has_complex_patterns, has_warnings) {
            (0..=3, false, false) => ComplexityLevel::Simple,
            (0..=3, true, _) | (4..=10, false, false) => ComplexityLevel::Moderate,
            (11..=20, _, _) | (_, true, true) => ComplexityLevel::Complex,
            _ => ComplexityLevel::Critical,
        }
    }

    fn split_into_sections(&self, response: &str) -> Vec<String> {
        // Split by common section markers
        let section_markers = [
            "\n\n",    // Double newline
            "\n---\n", // Markdown separator
            "\n###",   // Markdown headers
            "\nStep",  // Step indicators
            "\n1.", "\n2.", "\n3.", // Numbered lists
        ];

        let mut sections = vec![response.to_string()];

        for marker in &section_markers {
            let mut new_sections = Vec::new();
            for section in sections {
                let parts: Vec<_> = section.split(marker).map(|s| s.to_string()).collect();
                new_sections.extend(parts);
            }
            sections = new_sections;
        }

        // Filter out empty sections
        sections
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect()
    }

    async fn parse_section(
        &self,
        section: &str,
        section_index: usize,
        all_sections: &[String],
    ) -> Result<Vec<EnhancedFileOperation>> {
        let mut operations = Vec::new();

        // Try to parse each type of operation
        operations.extend(self.parse_create_operations(section).await?);
        operations.extend(self.parse_update_operations(section).await?);
        operations.extend(self.parse_delete_operations(section).await?);
        operations.extend(self.parse_rename_operations(section).await?);
        operations.extend(self.parse_move_operations(section).await?);

        // Extract context for each operation
        for operation in &mut operations {
            operation.context = self.extract_operation_context(section, &operation.operation)?;

            // Add semantic tags based on the operation
            operation.context.semantic_tags =
                self.extract_semantic_tags(&operation.operation, section)?;
        }

        Ok(operations)
    }

    async fn parse_create_operations(&self, section: &str) -> Result<Vec<EnhancedFileOperation>> {
        let mut operations = Vec::new();

        for caps in self.patterns.create_file.captures_iter(section) {
            let path = PathBuf::from(caps[1].trim());

            // Look for content after the file declaration
            let content = self.extract_file_content(section, &path)?;
            let has_content = content.is_some();

            let operation = FileOperation::Create {
                path: path.clone(),
                content: content.unwrap_or_default(),
            };

            let confidence = if has_content { 0.9 } else { 0.7 };

            operations.push(EnhancedFileOperation {
                operation,
                context: OperationContext::default(),
                parsing_confidence: confidence,
            });
        }

        Ok(operations)
    }

    async fn parse_update_operations(&self, section: &str) -> Result<Vec<EnhancedFileOperation>> {
        let mut operations = Vec::new();

        for caps in self.patterns.update_file.captures_iter(section) {
            let path = PathBuf::from(caps[1].trim());

            // Try to extract old and new content
            let (old_content, new_content) = self.extract_update_content(section, &path)?;
            let has_new_content = new_content.is_some();

            let operation = FileOperation::Update {
                path: path.clone(),
                content: new_content.unwrap_or_default(),
            };

            let confidence = if has_new_content { 0.9 } else { 0.7 };

            operations.push(EnhancedFileOperation {
                operation,
                context: OperationContext::default(),
                parsing_confidence: confidence,
            });
        }

        Ok(operations)
    }

    async fn parse_delete_operations(&self, section: &str) -> Result<Vec<EnhancedFileOperation>> {
        let mut operations = Vec::new();

        for caps in self.patterns.delete_file.captures_iter(section) {
            let path = PathBuf::from(caps[1].trim());

            let operation = FileOperation::Delete { path };

            operations.push(EnhancedFileOperation {
                operation,
                context: OperationContext::default(),
                parsing_confidence: 0.95, // Delete operations are usually clear
            });
        }

        Ok(operations)
    }

    async fn parse_rename_operations(&self, section: &str) -> Result<Vec<EnhancedFileOperation>> {
        let mut operations = Vec::new();

        for caps in self.patterns.rename_file.captures_iter(section) {
            let old_path = PathBuf::from(caps[1].trim());
            let new_path = PathBuf::from(caps[2].trim());

            let operation = FileOperation::Rename {
                from: old_path,
                to: new_path,
            };

            operations.push(EnhancedFileOperation {
                operation,
                context: OperationContext::default(),
                parsing_confidence: 0.9,
            });
        }

        Ok(operations)
    }

    async fn parse_move_operations(&self, section: &str) -> Result<Vec<EnhancedFileOperation>> {
        let mut operations = Vec::new();

        for caps in self.patterns.move_file.captures_iter(section) {
            let source = PathBuf::from(caps[1].trim());
            let destination = PathBuf::from(caps[2].trim());

            let operation = FileOperation::Rename {
                from: source,
                to: destination,
            };

            operations.push(EnhancedFileOperation {
                operation,
                context: OperationContext::default(),
                parsing_confidence: 0.9,
            });
        }

        Ok(operations)
    }

    fn extract_file_content(&self, section: &str, path: &PathBuf) -> Result<Option<String>> {
        // Look for code blocks after the file path mention
        let path_str = path.to_string_lossy();
        let path_position = section.find(&*path_str);

        if let Some(pos) = path_position {
            let after_path = &section[pos..];

            // Look for code blocks
            if let Some(caps) = self.patterns.code_block.captures(after_path) {
                return Ok(Some(caps[1].to_string()));
            }

            // Look for indented content
            let lines: Vec<&str> = after_path.lines().collect();
            let mut content_lines = Vec::new();
            let mut in_content = false;

            for line in lines.iter().skip(1) {
                if line.starts_with("    ") || line.starts_with("\t") {
                    in_content = true;
                    content_lines.push(line.trim_start());
                } else if in_content && line.trim().is_empty() {
                    content_lines.push("");
                } else if in_content {
                    break;
                }
            }

            if !content_lines.is_empty() {
                return Ok(Some(content_lines.join("\n")));
            }
        }

        Ok(None)
    }

    fn extract_update_content(
        &self,
        section: &str,
        path: &PathBuf,
    ) -> Result<(Option<String>, Option<String>)> {
        let path_str = path.to_string_lossy();
        let path_position = section.find(&*path_str);

        if let Some(pos) = path_position {
            let after_path = &section[pos..];

            // Look for diff blocks
            if let Some(caps) = self.patterns.diff_block.captures(after_path) {
                let diff = &caps[1];
                let mut old_lines = Vec::new();
                let mut new_lines = Vec::new();

                for line in diff.lines() {
                    if line.starts_with("-") && !line.starts_with("---") {
                        old_lines.push(&line[1..]);
                    } else if line.starts_with("+") && !line.starts_with("+++") {
                        new_lines.push(&line[1..]);
                    }
                }

                let old_content = if old_lines.is_empty() {
                    None
                } else {
                    Some(old_lines.join("\n"))
                };
                let new_content = if new_lines.is_empty() {
                    None
                } else {
                    Some(new_lines.join("\n"))
                };

                return Ok((old_content, new_content));
            }

            // Look for before/after code blocks
            let before_pattern =
                Regex::new(r"(?i)(?:before|old|original):?\s*```[\w]*\n([\s\S]*?)```").unwrap();
            let after_pattern =
                Regex::new(r"(?i)(?:after|new|updated):?\s*```[\w]*\n([\s\S]*?)```").unwrap();

            let old_content = before_pattern
                .captures(after_path)
                .map(|c| c[1].to_string());
            let new_content = after_pattern.captures(after_path).map(|c| c[1].to_string());

            if old_content.is_some() || new_content.is_some() {
                return Ok((old_content, new_content));
            }
        }

        // If no specific content found, try to extract any code block as new content
        let new_content = self.extract_file_content(section, path)?;
        Ok((None, new_content))
    }

    fn extract_operation_context(
        &self,
        section: &str,
        operation: &FileOperation,
    ) -> Result<OperationContext> {
        let mut context = OperationContext::default();

        // Extract explanation
        if let Some(caps) = self.patterns.explanation.captures(section) {
            context.explanation = Some(caps[1].trim().to_string());
        }

        // Extract intention
        if let Some(caps) = self.patterns.intention.captures(section) {
            context.intention = Some(caps[1].trim().to_string());
        }

        // Extract warnings
        for caps in self.patterns.warning.captures_iter(section) {
            context.warnings.push(caps[1].trim().to_string());
        }

        // Extract code context if enabled
        if self.enhancement_config.extract_code_snippets {
            context.code_context = self.extract_code_contexts(section)?;
        }

        Ok(context)
    }

    fn extract_code_contexts(&self, section: &str) -> Result<Vec<CodeContext>> {
        let mut contexts = Vec::new();

        // Extract code blocks
        for caps in self.patterns.code_block.captures_iter(section) {
            let full_match = caps.get(0).unwrap().as_str();
            let code = caps[1].to_string();

            // Try to detect language from the code block header
            let language = if full_match.starts_with("```") {
                let first_line = full_match.lines().next().unwrap_or("");
                let lang = first_line.trim_start_matches("```").trim();
                if !lang.is_empty() {
                    Some(lang.to_string())
                } else {
                    None
                }
            } else {
                None
            };

            contexts.push(CodeContext {
                language,
                code,
                is_diff: false,
                start_line: None,
            });
        }

        // Extract diff blocks
        for caps in self.patterns.diff_block.captures_iter(section) {
            contexts.push(CodeContext {
                language: None,
                code: caps[1].to_string(),
                is_diff: true,
                start_line: None,
            });
        }

        Ok(contexts)
    }

    fn extract_semantic_tags(
        &self,
        operation: &FileOperation,
        section: &str,
    ) -> Result<Vec<String>> {
        let mut tags = Vec::new();
        let section_lower = section.to_lowercase();

        // Operation type tags
        match operation {
            FileOperation::Create { path, .. } => {
                tags.push("create".to_string());
                if path.to_string_lossy().contains("test") {
                    tags.push("test".to_string());
                }
            }
            FileOperation::Update { .. } => tags.push("update".to_string()),
            FileOperation::Delete { .. } => tags.push("delete".to_string()),
            FileOperation::Append { .. } => tags.push("append".to_string()),
            FileOperation::Rename { .. } => tags.push("rename".to_string()),
        }

        // Content-based tags
        let tag_keywords = [
            ("security", vec!["auth", "security", "permission", "access"]),
            (
                "performance",
                vec!["optimize", "performance", "speed", "cache"],
            ),
            ("api", vec!["api", "endpoint", "route", "rest"]),
            ("database", vec!["database", "db", "sql", "query"]),
            ("ui", vec!["ui", "interface", "component", "view"]),
            ("config", vec!["config", "setting", "environment"]),
        ];

        for (tag, keywords) in &tag_keywords {
            if keywords.iter().any(|kw| section_lower.contains(kw)) {
                tags.push(tag.to_string());
            }
        }

        Ok(tags)
    }

    async fn enhance_with_ai(
        &self,
        mut operations: Vec<EnhancedFileOperation>,
        global_context: &GlobalContext,
    ) -> Result<Vec<EnhancedFileOperation>> {
        if let Some(synthesizer) = &self.knowledge_synthesizer {
            for operation in &mut operations {
                // Use AI to generate insights
                let ai_insights = self
                    .generate_ai_insights(
                        &operation.operation,
                        &operation.context,
                        global_context,
                        synthesizer,
                    )
                    .await?;

                operation.context.ai_insights = Some(ai_insights);
            }
        }

        Ok(operations)
    }

    async fn generate_ai_insights(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
        global_context: &GlobalContext,
        synthesizer: &Arc<KnowledgeSynthesizer>,
    ) -> Result<AIOperationInsights> {
        // In a real implementation, this would use the AI helper
        // For now, generate heuristic-based insights

        let mut risk_factors = Vec::new();
        let mut improvements = Vec::new();
        let mut best_practices = Vec::new();

        match operation {
            FileOperation::Delete { path } => {
                risk_factors.push("Deletion operations are irreversible".to_string());
                improvements.push("Consider archiving instead of deleting".to_string());
                best_practices.push("Always create backups before deletion".to_string());
            }
            FileOperation::Update { path, .. } => {
                if path.to_string_lossy().contains("config") {
                    risk_factors
                        .push("Configuration changes can affect system behavior".to_string());
                    best_practices.push("Test configuration changes in staging first".to_string());
                }
            }
            FileOperation::Create { path, .. } => {
                if path.to_string_lossy().contains("test") {
                    best_practices.push("Ensure test coverage for new functionality".to_string());
                }
            }
            _ => {}
        }

        let impact_prediction = match global_context.complexity_estimate {
            ComplexityLevel::Simple => "Low impact, localized changes",
            ComplexityLevel::Moderate => "Moderate impact, may affect related modules",
            ComplexityLevel::Complex => "High impact, requires careful testing",
            ComplexityLevel::Critical => "Critical impact, system-wide implications",
        }
        .to_string();

        Ok(AIOperationInsights {
            impact_prediction,
            improvements,
            risk_factors,
            best_practices,
        })
    }

    fn infer_operation_dependencies(&self, operations: &mut [EnhancedFileOperation]) -> Result<()> {
        // Simple dependency inference based on file relationships
        for i in 0..operations.len() {
            for j in 0..operations.len() {
                if i == j {
                    continue;
                }

                let depends = match (&operations[i].operation, &operations[j].operation) {
                    // Update depends on Create for same file
                    (
                        FileOperation::Update { path: p1, .. },
                        FileOperation::Create { path: p2, .. },
                    ) => p1 == p2,
                    // Move/Rename depends on operations on source file
                    (
                        FileOperation::Rename { from: s1, .. },
                        FileOperation::Create { path: p2, .. },
                    )
                    | (
                        FileOperation::Rename { from: s1, .. },
                        FileOperation::Update { path: p2, .. },
                    ) => s1 == p2,
                    _ => false,
                };

                if depends && !operations[i].context.dependencies.contains(&j) {
                    operations[i].context.dependencies.push(j);
                }
            }
        }

        Ok(())
    }

    pub async fn validate_parsed_operations(
        &self,
        parsed: &ParsedOperations,
    ) -> Result<Vec<String>> {
        let mut validation_warnings = Vec::new();

        // Check for missing content in create operations
        for (i, op) in parsed.operations.iter().enumerate() {
            if let FileOperation::Create { path, content } = &op.operation {
                if content.is_empty() {
                    validation_warnings.push(format!(
                        "Operation {}: Create operation for {} has no content",
                        i,
                        path.display()
                    ));
                }
            }
        }

        // Check for cyclic dependencies
        if self.has_cyclic_dependencies(&parsed.operations) {
            validation_warnings.push("Cyclic dependencies detected in operations".to_string());
        }

        // Check parsing confidence
        if parsed.metadata.overall_confidence < 0.7 {
            validation_warnings.push(format!(
                "Low parsing confidence: {:.0}%",
                parsed.metadata.overall_confidence * 100.0
            ));
        }

        Ok(validation_warnings)
    }

    fn has_cyclic_dependencies(&self, operations: &[EnhancedFileOperation]) -> bool {
        // Simple cycle detection using DFS
        let n = operations.len();
        let mut visited = vec![false; n];
        let mut rec_stack = vec![false; n];

        fn has_cycle_util(
            v: usize,
            operations: &[EnhancedFileOperation],
            visited: &mut [bool],
            rec_stack: &mut [bool],
        ) -> bool {
            visited[v] = true;
            rec_stack[v] = true;

            for &neighbor in &operations[v].context.dependencies {
                if !visited[neighbor] {
                    if has_cycle_util(neighbor, operations, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack[neighbor] {
                    return true;
                }
            }

            rec_stack[v] = false;
            false
        }

        for i in 0..n {
            if !visited[i] && has_cycle_util(i, operations, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }
}

impl Default for OperationContext {
    fn default() -> Self {
        Self {
            intention: None,
            explanation: None,
            warnings: Vec::new(),
            code_context: Vec::new(),
            dependencies: Vec::new(),
            semantic_tags: Vec::new(),
            ai_insights: None,
        }
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_create_operation() {
        let parser = OperationParser::new(None, None, None);

        let response = r#"
        I'll create a new test file for authentication.
        
        Create file: src/auth/test.rs
        ```rust
        #[cfg(all(test, feature = "legacy-tests"))]
        mod tests {
            #[test]
            fn test_login() {
                assert!(true);
            }
        }
        ```
        
        This will help ensure our authentication works correctly.
        "#;

        let parsed = parser.parse_curator_response(response).await.unwrap();

        assert_eq!(parsed.operations.len(), 1);
        assert!(matches!(
            &parsed.operations[0].operation,
            FileOperation::Create { path, content } if path == &PathBuf::from("src/auth/test.rs") && content.contains("test_login")
        ));
        assert!(parsed.operations[0].parsing_confidence > 0.8);
    }

    #[tokio::test]
    async fn test_parse_update_with_diff() {
        let parser = OperationParser::new(None, None, None);

        let response = r#"
        Update file: src/config.rs
        
        ```diff
        -const MAX_RETRIES: u32 = 3;
        +const MAX_RETRIES: u32 = 5;
        ```
        
        This increases the retry limit for better reliability.
        "#;

        let parsed = parser.parse_curator_response(response).await.unwrap();

        assert_eq!(parsed.operations.len(), 1);
        assert!(matches!(
            &parsed.operations[0].operation,
            FileOperation::Update { path, content }
                if path == &PathBuf::from("src/config.rs")
                && content.contains("MAX_RETRIES")
                && content.contains("u32")
        ));
    }

    #[tokio::test]
    async fn test_complexity_estimation() {
        let parser = OperationParser::new(None, None, None);

        let simple_response = "Create file: test.rs\nContent: fn main() {}";
        let complex_response = r#"
        This is a major refactoring of the authentication system.
        
        Delete file: old_auth.rs
        Create file: auth/mod.rs
        Create file: auth/login.rs
        Create file: auth/session.rs
        Update file: main.rs
        Update file: config.rs
        
        Warning: This will break existing sessions.
        "#;

        let simple_parsed = parser
            .parse_curator_response(simple_response)
            .await
            .unwrap();
        let complex_parsed = parser
            .parse_curator_response(complex_response)
            .await
            .unwrap();

        assert!(matches!(
            simple_parsed.global_context.complexity_estimate,
            ComplexityLevel::Simple
        ));
        assert!(matches!(
            complex_parsed.global_context.complexity_estimate,
            ComplexityLevel::Complex
        ));
    }
}
