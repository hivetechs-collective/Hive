//! Context building for AI queries
//!
//! This module provides intelligent context extraction for AI queries,
//! including relevant code snippets, documentation, and metadata.

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::core::{
    ast::SymbolKind,
    error::Result,
    semantic::{SearchQuery, SemanticIndex},
    Language,
};

/// Context builder for AI queries
pub struct ContextBuilder {
    /// Semantic index for code understanding
    semantic_index: Option<SemanticIndex>,
    /// Maximum context size in tokens
    max_tokens: usize,
    /// Context extraction strategies
    strategies: Vec<Box<dyn ContextStrategy>>,
}

/// Context for an AI query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    /// Primary code snippets
    pub code_snippets: Vec<CodeSnippet>,
    /// Related symbols
    pub symbols: Vec<ContextSymbol>,
    /// File summaries
    pub file_summaries: Vec<FileSummary>,
    /// Project metadata
    pub project_info: ProjectInfo,
    /// Relevant documentation
    pub documentation: Vec<Documentation>,
    /// Total context size in tokens
    pub total_tokens: usize,
}

/// Code snippet with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    /// File path
    pub file: PathBuf,
    /// Start line (1-indexed)
    pub start_line: usize,
    /// End line (1-indexed)
    pub end_line: usize,
    /// The code content
    pub content: String,
    /// Language
    pub language: Language,
    /// Relevance score (0-100)
    pub relevance: f32,
    /// Why this snippet is included
    pub reason: String,
}

/// Symbol with context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSymbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// File containing the symbol
    pub file: PathBuf,
    /// Symbol signature
    pub signature: Option<String>,
    /// Documentation
    pub documentation: Option<String>,
    /// Related symbols
    pub related: Vec<String>,
}

/// File summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    /// File path
    pub path: PathBuf,
    /// File purpose/description
    pub description: String,
    /// Key exports/definitions
    pub exports: Vec<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Project name
    pub name: String,
    /// Project type (Rust, TypeScript, etc.)
    pub project_type: String,
    /// Main technologies
    pub technologies: Vec<String>,
    /// Project structure summary
    pub structure: String,
}

/// Documentation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    /// Documentation source (README, docs, comments)
    pub source: String,
    /// Documentation content
    pub content: String,
    /// Relevance to query
    pub relevance: f32,
}

/// Context extraction strategy trait
trait ContextStrategy: Send + Sync {
    /// Extract context based on the query
    fn extract(&self, query: &str, index: &SemanticIndex) -> Result<PartialContext>;

    /// Strategy name
    fn name(&self) -> &str;
}

/// Partial context from a strategy
#[derive(Debug, Default)]
struct PartialContext {
    code_snippets: Vec<CodeSnippet>,
    symbols: Vec<ContextSymbol>,
    file_summaries: Vec<FileSummary>,
    documentation: Vec<Documentation>,
}

/// Keyword-based context extraction
struct KeywordStrategy;

impl ContextStrategy for KeywordStrategy {
    fn extract(&self, query: &str, _index: &SemanticIndex) -> Result<PartialContext> {
        // Extract keywords from query
        let keywords = self.extract_keywords(query);
        let context = PartialContext::default();

        // Search for symbols matching keywords
        for keyword in keywords {
            let _search_query = SearchQuery {
                query: keyword.clone(),
                kinds: None,
                include_patterns: None,
                exclude_patterns: None,
                limit: 5,
                fuzzy: true,
            };

            // This would use the actual search in a real implementation
            // For now, return empty context
        }

        Ok(context)
    }

    fn name(&self) -> &str {
        "keyword"
    }
}

impl KeywordStrategy {
    fn extract_keywords(&self, query: &str) -> Vec<String> {
        query
            .split_whitespace()
            .filter(|w| w.len() > 3 && !COMMON_WORDS.contains(w))
            .map(|w| w.to_string())
            .collect()
    }
}

/// Semantic similarity context extraction
struct SemanticStrategy;

impl ContextStrategy for SemanticStrategy {
    fn extract(&self, _query: &str, _index: &SemanticIndex) -> Result<PartialContext> {
        // In a real implementation, this would use embeddings
        // to find semantically similar code
        Ok(PartialContext::default())
    }

    fn name(&self) -> &str {
        "semantic"
    }
}

/// Type-based context extraction
struct TypeStrategy;

impl ContextStrategy for TypeStrategy {
    fn extract(&self, _query: &str, _index: &SemanticIndex) -> Result<PartialContext> {
        // Extract type names from query and find related code
        Ok(PartialContext::default())
    }

    fn name(&self) -> &str {
        "type"
    }
}

/// Common words to exclude from keyword search
const COMMON_WORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "as", "is", "was", "are", "were", "been", "have", "has", "had", "do", "does", "did",
    "will", "would", "could", "should", "may", "might", "must", "can", "this", "that", "these",
    "those", "what", "which", "who", "when", "where", "why", "how",
];

impl ContextBuilder {
    /// Create a new context builder
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn ContextStrategy>> = vec![
            Box::new(KeywordStrategy),
            Box::new(SemanticStrategy),
            Box::new(TypeStrategy),
        ];

        Self {
            semantic_index: None,
            max_tokens: 8000, // Default max tokens
            strategies,
        }
    }

    /// Set the semantic index
    pub fn with_index(mut self, index: SemanticIndex) -> Self {
        self.semantic_index = Some(index);
        self
    }

    /// Set maximum context size in tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Build context for a query
    pub async fn build_context(&self, query: &str) -> Result<QueryContext> {
        let index = self
            .semantic_index
            .as_ref()
            .ok_or_else(|| anyhow!("Semantic index not available"))?;

        // Extract context using all strategies
        let mut all_snippets = Vec::new();
        let mut all_symbols = Vec::new();
        let mut all_summaries = Vec::new();
        let mut all_docs = Vec::new();

        for strategy in &self.strategies {
            match strategy.extract(query, index) {
                Ok(partial) => {
                    all_snippets.extend(partial.code_snippets);
                    all_symbols.extend(partial.symbols);
                    all_summaries.extend(partial.file_summaries);
                    all_docs.extend(partial.documentation);
                }
                Err(e) => {
                    tracing::warn!("Strategy {} failed: {}", strategy.name(), e);
                }
            }
        }

        // Deduplicate and rank
        all_snippets.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        all_snippets.dedup_by(|a, b| a.file == b.file && a.start_line == b.start_line);

        all_symbols.dedup_by(|a, b| a.name == b.name);
        all_summaries.dedup_by(|a, b| a.path == b.path);
        all_docs.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());

        // Trim to fit token budget
        let mut total_tokens = 0;
        let mut final_snippets = Vec::new();

        for snippet in all_snippets {
            let snippet_tokens = self.estimate_tokens(&snippet.content);
            if total_tokens + snippet_tokens <= self.max_tokens {
                total_tokens += snippet_tokens;
                final_snippets.push(snippet);
            } else {
                break;
            }
        }

        // Build project info
        let project_info = self.build_project_info().await?;

        Ok(QueryContext {
            code_snippets: final_snippets,
            symbols: all_symbols.into_iter().take(20).collect(),
            file_summaries: all_summaries.into_iter().take(10).collect(),
            project_info,
            documentation: all_docs.into_iter().take(5).collect(),
            total_tokens,
        })
    }

    /// Build context for a specific file
    pub async fn build_file_context(&self, file: &Path) -> Result<QueryContext> {
        let content = tokio::fs::read_to_string(file).await?;
        let language = crate::core::detect_language(file);

        let snippet = CodeSnippet {
            file: file.to_path_buf(),
            start_line: 1,
            end_line: content.lines().count(),
            content: content.clone(),
            language,
            relevance: 100.0,
            reason: "Full file context".to_string(),
        };

        let project_info = self.build_project_info().await?;

        Ok(QueryContext {
            code_snippets: vec![snippet],
            symbols: vec![],
            file_summaries: vec![],
            project_info,
            documentation: vec![],
            total_tokens: self.estimate_tokens(&content),
        })
    }

    /// Build context for a symbol
    pub async fn build_symbol_context(&self, symbol_name: &str) -> Result<QueryContext> {
        let index = self
            .semantic_index
            .as_ref()
            .ok_or_else(|| anyhow!("Semantic index not available"))?;

        // Find symbol definition
        let definition = index.find_definition(symbol_name).await?;
        let mut snippets = Vec::new();
        let mut symbols = Vec::new();

        if let Some(def) = definition {
            // Add definition snippet
            if let Ok(content) = tokio::fs::read_to_string(&def.file).await {
                let lines: Vec<&str> = content.lines().collect();
                let start = def.symbol.location.line.saturating_sub(5);
                let end = (def.symbol.location.line + 10).min(lines.len());

                let snippet_content = lines[start..end].join("\n");

                snippets.push(CodeSnippet {
                    file: def.file.clone(),
                    start_line: start + 1,
                    end_line: end,
                    content: snippet_content,
                    language: crate::core::detect_language(&def.file),
                    relevance: 100.0,
                    reason: "Symbol definition".to_string(),
                });

                symbols.push(ContextSymbol {
                    name: def.symbol.name.clone(),
                    kind: def.symbol.kind,
                    file: def.file,
                    signature: def.symbol.signature,
                    documentation: def.symbol.docs,
                    related: vec![],
                });
            }
        }

        // Find references
        let references = index.find_references(symbol_name).await?;
        for (i, reference) in references.iter().take(5).enumerate() {
            if let Ok(content) = tokio::fs::read_to_string(&reference.file).await {
                let lines: Vec<&str> = content.lines().collect();
                let start = reference.line.saturating_sub(2);
                let end = (reference.line + 3).min(lines.len());

                let snippet_content = lines[start..end].join("\n");

                snippets.push(CodeSnippet {
                    file: reference.file.clone(),
                    start_line: start + 1,
                    end_line: end,
                    content: snippet_content,
                    language: crate::core::detect_language(&reference.file),
                    relevance: 80.0 - (i as f32 * 10.0),
                    reason: format!("Reference #{}", i + 1),
                });
            }
        }

        let project_info = self.build_project_info().await?;
        let total_tokens = snippets
            .iter()
            .map(|s| self.estimate_tokens(&s.content))
            .sum();

        Ok(QueryContext {
            code_snippets: snippets,
            symbols,
            file_summaries: vec![],
            project_info,
            documentation: vec![],
            total_tokens,
        })
    }

    /// Estimate token count for text
    fn estimate_tokens(&self, text: &str) -> usize {
        // Rough estimate: 1 token per 4 characters
        text.len() / 4
    }

    /// Build project information
    async fn build_project_info(&self) -> Result<ProjectInfo> {
        // In a real implementation, this would analyze the project
        Ok(ProjectInfo {
            name: "Unknown Project".to_string(),
            project_type: "Unknown".to_string(),
            technologies: vec![],
            structure: "Standard project structure".to_string(),
        })
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Format context for AI consumption
pub fn format_context(context: &QueryContext) -> String {
    let mut output = String::new();

    // Add project info
    output.push_str(&format!("# Project: {}\n", context.project_info.name));
    output.push_str(&format!("Type: {}\n", context.project_info.project_type));
    output.push_str("\n");

    // Add code snippets
    for snippet in &context.code_snippets {
        output.push_str(&format!(
            "## {} (lines {}-{})\n",
            snippet.file.display(),
            snippet.start_line,
            snippet.end_line
        ));
        output.push_str(&format!("```{}\n", language_to_string(snippet.language)));
        output.push_str(&snippet.content);
        output.push_str("\n```\n\n");
    }

    // Add symbols
    if !context.symbols.is_empty() {
        output.push_str("## Related Symbols\n");
        for symbol in &context.symbols {
            output.push_str(&format!("- {} ({:?})\n", symbol.name, symbol.kind));
            if let Some(sig) = &symbol.signature {
                output.push_str(&format!("  Signature: {}\n", sig));
            }
        }
        output.push_str("\n");
    }

    output
}

/// Convert language enum to string for syntax highlighting
fn language_to_string(lang: Language) -> &'static str {
    match lang {
        Language::Rust => "rust",
        Language::TypeScript => "typescript",
        Language::JavaScript => "javascript",
        Language::Python => "python",
        Language::Go => "go",
        Language::Java => "java",
        Language::Cpp => "cpp",
        Language::C => "c",
        Language::Ruby => "ruby",
        Language::PHP => "php",
        Language::Swift => "swift",
        Language::Unknown => "text",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let builder = ContextBuilder::new();
        assert_eq!(builder.max_tokens, 8000);
        assert_eq!(builder.strategies.len(), 3);
    }

    #[test]
    fn test_token_estimation() {
        let builder = ContextBuilder::new();
        assert_eq!(builder.estimate_tokens("Hello, world!"), 3);
        assert_eq!(
            builder.estimate_tokens("fn main() { println!(\"test\"); }"),
            8
        );
    }

    #[test]
    fn test_keyword_extraction() {
        let strategy = KeywordStrategy;
        let keywords =
            strategy.extract_keywords("How do I implement a binary search tree in Rust?");
        assert!(keywords.contains(&"implement".to_string()));
        assert!(keywords.contains(&"binary".to_string()));
        assert!(keywords.contains(&"search".to_string()));
        assert!(keywords.contains(&"tree".to_string()));
        assert!(keywords.contains(&"Rust".to_string()));
        assert!(!keywords.contains(&"How".to_string())); // Common word
    }
}
