//! AST engine for multi-language code parsing and analysis
//!
//! This module provides a unified AST parsing interface that supports
//! multiple programming languages using tree-sitter parsers.

use std::path::Path;
use std::sync::Arc;
use anyhow::anyhow;
use serde::{Serialize, Deserialize};

use crate::{
    core::{Language, error::Result},
    cache::{self, CacheCategory, CacheKey},
};

// Re-exports are handled through the struct/enum definitions themselves

/// AST engine for parsing code across multiple languages
pub struct AstEngine {
    /// Parser cache for different languages
    parsers: std::collections::HashMap<Language, Arc<dyn LanguageParser>>,
}

/// AST node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    /// Node type (e.g., "function_declaration", "class_definition")
    pub node_type: String,
    /// Node name if available (e.g., function name)
    pub name: Option<String>,
    /// Start position in the source code
    pub start_pos: Position,
    /// End position in the source code
    pub end_pos: Position,
    /// Child nodes
    pub children: Vec<AstNode>,
    /// Additional metadata
    pub metadata: NodeMetadata,
}

/// Position in source code
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub column: usize,
    /// Byte offset
    pub offset: usize,
}

/// Node metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Whether this is a definition (function, class, etc.)
    pub is_definition: bool,
    /// Whether this is a reference/usage
    pub is_reference: bool,
    /// Visibility/access modifier
    pub visibility: Option<String>,
    /// Documentation comment
    pub doc_comment: Option<String>,
    /// Type information if available
    pub type_info: Option<String>,
    /// Complexity score
    pub complexity: u32,
}

/// Language-specific parser trait
pub trait LanguageParser: Send + Sync {
    /// Parse source code into AST
    fn parse(&self, source: &str) -> Result<AstNode>;
    
    /// Get language name
    fn language(&self) -> Language;
    
    /// Extract symbols from AST
    fn extract_symbols(&self, ast: &AstNode) -> Vec<Symbol>;
}

/// Symbol information extracted from AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind (function, class, variable, etc.)
    pub kind: SymbolKind,
    /// Symbol location
    pub location: Position,
    /// Parent symbol (for nested symbols)
    pub parent: Option<String>,
    /// Symbol signature/type
    pub signature: Option<String>,
    /// Documentation
    pub docs: Option<String>,
}

/// Symbol kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SymbolKind {
    #[default]
    Function,
    Method,
    Class,
    Interface,
    Enum,
    Struct,
    Variable,
    Constant,
    Module,
    Namespace,
    Property,
    Parameter,
    TypeAlias,
    Import,
    Trait,
}

impl SymbolKind {
    /// Get string representation of the symbol kind
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Class => "class",
            SymbolKind::Interface => "interface",
            SymbolKind::Enum => "enum",
            SymbolKind::Struct => "struct",
            SymbolKind::Variable => "variable",
            SymbolKind::Constant => "constant",
            SymbolKind::Module => "module",
            SymbolKind::Namespace => "namespace",
            SymbolKind::Property => "property",
            SymbolKind::Parameter => "parameter",
            SymbolKind::TypeAlias => "type_alias",
            SymbolKind::Import => "import",
            SymbolKind::Trait => "trait",
        }
    }
}

/// Parsing result with AST and extracted information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// The parsed AST
    pub ast: AstNode,
    /// Extracted symbols
    pub symbols: Vec<Symbol>,
    /// Import statements
    pub imports: Vec<ImportInfo>,
    /// Detected errors
    pub errors: Vec<ParseError>,
    /// Code metrics
    pub metrics: CodeMetrics,
}

/// Import information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    /// Imported module/package
    pub module: String,
    /// Imported items
    pub items: Vec<String>,
    /// Import location
    pub location: Position,
    /// Whether it's a wildcard import
    pub is_wildcard: bool,
}

/// Parse error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    /// Error message
    pub message: String,
    /// Error location
    pub location: Position,
    /// Error severity
    pub severity: ErrorSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Code metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeMetrics {
    /// Lines of code (excluding comments and blank lines)
    pub lines_of_code: usize,
    /// Comment lines
    pub comment_lines: usize,
    /// Cyclomatic complexity
    pub complexity: u32,
    /// Number of functions
    pub function_count: usize,
    /// Number of classes
    pub class_count: usize,
    /// Maximum nesting depth
    pub max_nesting: u32,
}

/// Generic parser implementation (placeholder)
struct GenericParser {
    language: Language,
}

impl LanguageParser for GenericParser {
    fn parse(&self, source: &str) -> Result<AstNode> {
        // This is a simplified implementation
        // In production, this would use tree-sitter or similar
        Ok(AstNode {
            node_type: "source_file".to_string(),
            name: None,
            start_pos: Position { line: 0, column: 0, offset: 0 },
            end_pos: Position {
                line: source.lines().count().saturating_sub(1),
                column: source.lines().last().map(|l| l.len()).unwrap_or(0),
                offset: source.len(),
            },
            children: vec![],
            metadata: NodeMetadata {
                is_definition: false,
                is_reference: false,
                visibility: None,
                doc_comment: None,
                type_info: None,
                complexity: 1,
            },
        })
    }
    
    fn language(&self) -> Language {
        self.language
    }
    
    fn extract_symbols(&self, _ast: &AstNode) -> Vec<Symbol> {
        // Placeholder implementation
        vec![]
    }
}

impl AstEngine {
    /// Create a new AST engine
    pub fn new() -> Self {
        let mut parsers = std::collections::HashMap::new();
        
        // Register parsers for each language
        for lang in &[
            Language::Rust,
            Language::TypeScript,
            Language::JavaScript,
            Language::Python,
            Language::Go,
            Language::Java,
            Language::Cpp,
            Language::C,
        ] {
            parsers.insert(
                *lang,
                Arc::new(GenericParser { language: *lang }) as Arc<dyn LanguageParser>
            );
        }
        
        Self { parsers }
    }
    
    /// Parse a file into an AST
    pub async fn parse_file(&self, path: &Path, content: &str) -> Result<ParseResult> {
        let language = crate::core::detect_language(path);
        
        // Check cache first
        let cache_key = CacheKey::ast(path);
        if let Some(cached) = cache::get(&cache_key, CacheCategory::Ast).await {
            if let Ok(result) = serde_json::from_slice::<ParseResult>(&cached) {
                tracing::debug!("AST cache hit for {}", path.display());
                return Ok(result);
            }
        }
        
        // Parse the file
        let result = self.parse_content(content, language).await?;
        
        // Cache the result
        if let Ok(serialized) = serde_json::to_vec(&result) {
            let _ = cache::put(cache_key, serialized, CacheCategory::Ast).await;
        }
        
        Ok(result)
    }
    
    /// Parse content with a specific language
    pub async fn parse_content(&self, content: &str, language: Language) -> Result<ParseResult> {
        let parser = self.parsers.get(&language)
            .ok_or_else(|| anyhow!("No parser available for {:?}", language))?;
        
        let ast = parser.parse(content)?;
        let symbols = parser.extract_symbols(&ast);
        
        // Extract additional information
        let imports = self.extract_imports(&ast, language);
        let metrics = self.calculate_metrics(&ast, content);
        
        Ok(ParseResult {
            ast,
            symbols,
            imports,
            errors: vec![], // Placeholder
            metrics,
        })
    }
    
    /// Extract import statements from AST
    fn extract_imports(&self, _ast: &AstNode, _language: Language) -> Vec<ImportInfo> {
        // Placeholder implementation
        vec![]
    }
    
    /// Calculate code metrics from AST
    fn calculate_metrics(&self, ast: &AstNode, content: &str) -> CodeMetrics {
        let lines: Vec<&str> = content.lines().collect();
        let mut code_lines = 0;
        let mut comment_lines = 0;
        
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*") {
                comment_lines += 1;
            } else {
                code_lines += 1;
            }
        }
        
        CodeMetrics {
            lines_of_code: code_lines,
            comment_lines,
            complexity: self.calculate_complexity(ast),
            function_count: self.count_nodes(ast, "function"),
            class_count: self.count_nodes(ast, "class"),
            max_nesting: self.calculate_max_nesting(ast, 0),
        }
    }
    
    /// Calculate cyclomatic complexity
    fn calculate_complexity(&self, ast: &AstNode) -> u32 {
        let mut complexity = 1;
        
        // Add complexity for control flow nodes
        if matches!(ast.node_type.as_str(), "if_statement" | "while_statement" | "for_statement" | "match_expression" | "case_statement") {
            complexity += 1;
        }
        
        for child in &ast.children {
            complexity += self.calculate_complexity(child);
        }
        
        complexity
    }
    
    /// Count nodes of a specific type
    fn count_nodes(&self, ast: &AstNode, node_type: &str) -> usize {
        let mut count = 0;
        
        if ast.node_type.contains(node_type) {
            count += 1;
        }
        
        for child in &ast.children {
            count += self.count_nodes(child, node_type);
        }
        
        count
    }
    
    /// Calculate maximum nesting depth
    fn calculate_max_nesting(&self, ast: &AstNode, current_depth: u32) -> u32 {
        let mut max_depth = current_depth;
        
        for child in &ast.children {
            let child_depth = self.calculate_max_nesting(child, current_depth + 1);
            max_depth = max_depth.max(child_depth);
        }
        
        max_depth
    }
    
    /// Find symbol at a specific position
    pub fn find_symbol_at<'a>(&self, result: &'a ParseResult, line: usize, column: usize) -> Option<&'a Symbol> {
        result.symbols.iter().find(|s| {
            s.location.line == line && 
            s.location.column <= column
        })
    }
    
    /// Get all symbols of a specific kind
    pub fn get_symbols_by_kind<'a>(&self, result: &'a ParseResult, kind: SymbolKind) -> Vec<&'a Symbol> {
        result.symbols.iter()
            .filter(|s| s.kind == kind)
            .collect()
    }
}

impl Default for AstEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ast_engine() -> Result<()> {
        let engine = AstEngine::new();
        let content = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        
        let result = engine.parse_content(content, Language::Rust).await?;
        assert_eq!(result.metrics.lines_of_code, 3);
        
        Ok(())
    }
    
    #[test]
    fn test_code_metrics() {
        let engine = AstEngine::new();
        let ast = AstNode {
            node_type: "function".to_string(),
            name: Some("test".to_string()),
            start_pos: Position { line: 0, column: 0, offset: 0 },
            end_pos: Position { line: 5, column: 0, offset: 100 },
            children: vec![],
            metadata: NodeMetadata {
                is_definition: true,
                is_reference: false,
                visibility: Some("public".to_string()),
                doc_comment: None,
                type_info: None,
                complexity: 1,
            },
        };
        
        let count = engine.count_nodes(&ast, "function");
        assert_eq!(count, 1);
    }
}