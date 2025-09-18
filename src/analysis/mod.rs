//! Code analysis module for HiveTechs Consensus
//!
//! This module provides comprehensive code analysis capabilities including:
//! - Multi-language AST parsing with tree-sitter
//! - Incremental parsing with <5ms updates
//! - Syntax highlighting for TUI
//! - Language detection
//! - Performance monitoring
//! - Symbol indexing with FTS5
//! - Dependency analysis with petgraph
//! - Repository intelligence

pub mod dependency;
pub mod fast_parse;
pub mod file_analyzer;
pub mod incremental;
pub mod language_detector;
pub mod parser;
pub mod performance;
pub mod repository_intelligence;
pub mod symbol_index;
pub mod syntax_highlighter;
pub mod types;

pub use language_detector::{detect_language, LanguageDetector};
pub use parser::{
    Edit, HighlightSpan, HighlightType, IncrementalParseState, ParserRegistry, TreeSitterParser,
};

// Type aliases for compatibility with other modules
pub type Parser = TreeSitterParser;
pub type AST = crate::core::AstNode;
pub use dependency::{
    DependencyAnalysis, DependencyAnalyzer, DependencyEdge, DependencyGraph, DependencyKind,
    ModuleNode,
};
pub use file_analyzer::FileAnalyzer;
pub use incremental::IncrementalParser;
pub use performance::{ParseMetrics, PerformanceMonitor, PerformanceStatus};
pub use repository_intelligence::{
    ArchitectureInfo, ArchitecturePattern, PerformanceReport, QualityReport, RepositoryAnalysis,
    RepositoryAnalyzer, SecurityReport, TechnicalDebtReport,
};
pub use symbol_index::{
    CallGraphInfo, IndexStatistics, ReferenceKind, SymbolEntry, SymbolIndexer, SymbolReference,
};
pub use syntax_highlighter::SyntaxHighlighter;

// Re-export core types for convenience
pub use crate::core::ParseResult;

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cache::{self, CacheCategory, CacheKey};
use crate::core::{FileInfo, Language};

/// Main analysis engine that coordinates all analysis operations
pub struct AnalysisEngine {
    /// Parser registry for different languages
    parser_registry: Arc<Mutex<ParserRegistry>>,
    /// Language detector
    language_detector: LanguageDetector,
    /// Incremental parser
    incremental_parser: IncrementalParser,
    /// Syntax highlighter
    syntax_highlighter: SyntaxHighlighter,
    /// Performance monitor
    performance_monitor: Arc<PerformanceMonitor>,
}

impl AnalysisEngine {
    /// Create a new analysis engine
    pub fn new() -> Self {
        Self {
            parser_registry: Arc::new(Mutex::new(ParserRegistry::new())),
            language_detector: LanguageDetector::new(),
            incremental_parser: IncrementalParser::new(),
            syntax_highlighter: SyntaxHighlighter::new(),
            performance_monitor: Arc::new(PerformanceMonitor::new()),
        }
    }

    /// Analyze a file
    pub async fn analyze_file(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<crate::core::ParseResult> {
        let _timer = self.performance_monitor.start_operation("analyze_file");

        // Detect language
        let language = self.language_detector.detect_from_path(path)?;

        // Check cache first
        let cache_key = CacheKey::ast(path);
        if let Some(cached) = cache::get(&cache_key, CacheCategory::Ast).await {
            if let Ok(result) = serde_json::from_slice::<ParseResult>(&cached) {
                self.performance_monitor.record_cache_hit();
                return Ok(result);
            }
        }

        // Parse the file
        let mut registry = self.parser_registry.lock().await;
        let parser = registry.get_parser(language).await?;
        let mut parser = parser.lock().await;

        let result = parser.parse(content)?;

        // Cache the result
        if let Ok(serialized) = serde_json::to_vec(&result) {
            let _ = cache::put(cache_key, serialized, CacheCategory::Ast).await;
        }

        self.performance_monitor.record_cache_miss();
        Ok(result)
    }

    /// Perform incremental analysis
    pub async fn analyze_incremental(
        &self,
        path: &Path,
        old_content: &str,
        new_content: &str,
        edit: &Edit,
    ) -> Result<crate::core::ParseResult> {
        let _timer = self
            .performance_monitor
            .start_operation("analyze_incremental");

        let language = self.language_detector.detect_from_path(path)?;

        let mut registry = self.parser_registry.lock().await;
        let parser = registry.get_parser(language).await?;

        self.incremental_parser
            .parse_incremental(parser, old_content, new_content, edit)
            .await
    }

    /// Get syntax highlighting for content
    pub async fn get_highlights(&self, path: &Path, content: &str) -> Result<Vec<HighlightSpan>> {
        let _timer = self.performance_monitor.start_operation("get_highlights");

        let language = self.language_detector.detect_from_path(path)?;

        let mut registry = self.parser_registry.lock().await;
        let parser = registry.get_parser(language).await?;

        self.syntax_highlighter.highlight(parser, content).await
    }

    /// Detect language from file path or content
    pub fn detect_language(&self, path: &Path, content: Option<&str>) -> Result<Language> {
        if let Some(content) = content {
            self.language_detector.detect_from_content(path, content)
        } else {
            self.language_detector.detect_from_path(path)
        }
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> ParseMetrics {
        self.performance_monitor.get_metrics()
    }

    /// Reset performance metrics
    pub fn reset_metrics(&self) {
        self.performance_monitor.reset();
    }
}

impl Default for AnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_analysis_engine() {
        let engine = AnalysisEngine::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn main() {{ println!(\"Hello\"); }}").unwrap();

        let content = "fn main() { println!(\"Hello\"); }";
        let result = engine
            .analyze_file(temp_file.path(), content)
            .await
            .unwrap();

        assert_eq!(result.metrics.function_count, 1);
        assert_eq!(result.metrics.lines_of_code, 1);
    }

    #[tokio::test]
    async fn test_language_detection() {
        let engine = AnalysisEngine::new();

        let rust_path = Path::new("test.rs");
        let python_path = Path::new("test.py");

        assert_eq!(
            engine.detect_language(rust_path, None).unwrap(),
            Language::Rust
        );
        assert_eq!(
            engine.detect_language(python_path, None).unwrap(),
            Language::Python
        );
    }
}
