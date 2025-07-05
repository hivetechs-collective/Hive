//! Multi-language AST parser using tree-sitter
//!
//! This module provides high-performance parsing for multiple programming languages
//! with incremental parsing support and <5ms update times.

use std::sync::Arc;
use std::time::Instant;
use std::collections::HashMap;
use anyhow::{Result, anyhow, Context};
use tree_sitter::{Parser, Tree, Node, Language as TSLanguage, InputEdit, Point};
use serde::{Serialize, Deserialize};
use tracing::{debug, instrument};

use crate::core::{Language, Position, HiveError};
use crate::core::ast::{AstNode, NodeMetadata, Symbol, SymbolKind, ParseResult, ParseError, ErrorSeverity, ImportInfo, CodeMetrics};

/// Tree-sitter based parser implementation
pub struct TreeSitterParser {
    /// Language identifier
    language: Language,
    /// Tree-sitter language
    ts_language: TSLanguage,
    /// Parser instance
    pub parser: Parser,
    /// Query patterns for symbol extraction
    symbol_query: Option<tree_sitter::Query>,
    /// Query patterns for import extraction
    import_query: Option<tree_sitter::Query>,
    /// Query patterns for syntax highlighting
    highlight_query: Option<tree_sitter::Query>,
}

/// Incremental parsing state
#[derive(Debug, Clone)]
pub struct IncrementalParseState {
    /// Previous tree
    pub tree: Tree,
    /// Source code
    pub source: String,
    /// Parse time
    pub parse_time_ms: f64,
}

/// Edit operation for incremental parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edit {
    /// Start byte offset
    pub start_byte: usize,
    /// Old end byte offset
    pub old_end_byte: usize,
    /// New end byte offset
    pub new_end_byte: usize,
    /// Start position
    pub start_position: Position,
    /// Old end position
    pub old_end_position: Position,
    /// New end position
    pub new_end_position: Position,
}

/// Syntax highlight information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSpan {
    /// Start position
    pub start: Position,
    /// End position
    pub end: Position,
    /// Highlight type
    pub highlight_type: HighlightType,
}

/// Types of syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HighlightType {
    Keyword,
    Function,
    Type,
    Variable,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
    Constant,
    Parameter,
    Property,
    Label,
    Attribute,
    Namespace,
}

impl TreeSitterParser {
    /// Create a new parser for the given language
    pub fn new(language: Language) -> Result<Self> {
        let (ts_language, symbol_query, import_query, highlight_query) = match language {
            Language::Rust => {
                let lang = tree_sitter_rust::language();
                let symbol_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (function_item name: (identifier) @function.name) @function
                    (struct_item name: (type_identifier) @struct.name) @struct
                    (enum_item name: (type_identifier) @enum.name) @enum
                    (trait_item name: (type_identifier) @trait.name) @trait
                    (impl_item type: (type_identifier) @impl.type) @impl
                    (const_item name: (identifier) @const.name) @const
                    (static_item name: (identifier) @static.name) @static
                    (mod_item name: (identifier) @module.name) @module
                    (type_alias name: (type_identifier) @type_alias.name) @type_alias
                    "#
                ).ok();
                
                let import_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (use_declaration) @import
                    "#
                ).ok();
                
                let highlight_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    "fn" @keyword.function
                    "let" @keyword
                    "mut" @keyword
                    "const" @keyword
                    "static" @keyword
                    "if" @keyword.control
                    "else" @keyword.control
                    "match" @keyword.control
                    "for" @keyword.control
                    "while" @keyword.control
                    "loop" @keyword.control
                    "return" @keyword.control
                    "break" @keyword.control
                    "continue" @keyword.control
                    (string_literal) @string
                    (integer_literal) @number
                    (float_literal) @number
                    (line_comment) @comment
                    (block_comment) @comment
                    (identifier) @variable
                    (type_identifier) @type
                    "#
                ).ok();
                
                (lang, symbol_query, import_query, highlight_query)
            },
            Language::TypeScript | Language::JavaScript => {
                let lang = if language == Language::TypeScript {
                    tree_sitter_typescript::language_typescript()
                } else {
                    tree_sitter_javascript::language()
                };
                
                let symbol_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (function_declaration name: (identifier) @function.name) @function
                    (class_declaration name: (identifier) @class.name) @class
                    (interface_declaration name: (identifier) @interface.name) @interface
                    (variable_declaration (variable_declarator name: (identifier) @variable.name)) @variable
                    (method_definition name: (property_identifier) @method.name) @method
                    "#
                ).ok();
                
                let import_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (import_statement) @import
                    "#
                ).ok();
                
                let highlight_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    ["function" "const" "let" "var" "class" "interface" "type"] @keyword
                    ["if" "else" "for" "while" "do" "switch" "case" "break" "continue" "return"] @keyword.control
                    (string) @string
                    (number) @number
                    (comment) @comment
                    (identifier) @variable
                    (type_identifier) @type
                    "#
                ).ok();
                
                (lang, symbol_query, import_query, highlight_query)
            },
            Language::Python => {
                let lang = tree_sitter_python::language();
                
                let symbol_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (function_definition name: (identifier) @function.name) @function
                    (class_definition name: (identifier) @class.name) @class
                    "#
                ).ok();
                
                let import_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (import_statement) @import
                    (import_from_statement) @import
                    "#
                ).ok();
                
                let highlight_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    ["def" "class" "if" "elif" "else" "for" "while" "try" "except" "finally" "with" "return" "break" "continue"] @keyword
                    (string) @string
                    (integer) @number
                    (float) @number
                    (comment) @comment
                    (identifier) @variable
                    "#
                ).ok();
                
                (lang, symbol_query, import_query, highlight_query)
            },
            Language::Go => {
                let lang = tree_sitter_go::language();
                
                let symbol_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (function_declaration name: (identifier) @function.name) @function
                    (type_declaration (type_spec name: (type_identifier) @type.name)) @type
                    (const_declaration (const_spec name: (identifier) @const.name)) @const
                    (var_declaration (var_spec name: (identifier) @var.name)) @var
                    "#
                ).ok();
                
                let import_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (import_declaration) @import
                    "#
                ).ok();
                
                let highlight_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    ["func" "type" "struct" "interface" "const" "var" "if" "else" "for" "switch" "case" "return" "break" "continue"] @keyword
                    (string_literal) @string
                    (int_literal) @number
                    (float_literal) @number
                    (comment) @comment
                    (identifier) @variable
                    (type_identifier) @type
                    "#
                ).ok();
                
                (lang, symbol_query, import_query, highlight_query)
            },
            Language::Java => {
                let lang = tree_sitter_java::language();
                
                let symbol_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (class_declaration name: (identifier) @class.name) @class
                    (interface_declaration name: (identifier) @interface.name) @interface
                    (method_declaration name: (identifier) @method.name) @method
                    (field_declaration declarator: (variable_declarator name: (identifier) @field.name)) @field
                    "#
                ).ok();
                
                let import_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (import_declaration) @import
                    "#
                ).ok();
                
                let highlight_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    ["class" "interface" "public" "private" "protected" "static" "final" "if" "else" "for" "while" "switch" "case" "return" "break" "continue"] @keyword
                    (string_literal) @string
                    (decimal_integer_literal) @number
                    (decimal_floating_point_literal) @number
                    (comment) @comment
                    (identifier) @variable
                    (type_identifier) @type
                    "#
                ).ok();
                
                (lang, symbol_query, import_query, highlight_query)
            },
            Language::Cpp | Language::C => {
                let lang = if language == Language::Cpp {
                    tree_sitter_cpp::language()
                } else {
                    tree_sitter_c::language()
                };
                
                let symbol_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (function_definition declarator: (function_declarator declarator: (identifier) @function.name)) @function
                    (struct_specifier name: (type_identifier) @struct.name) @struct
                    (class_specifier name: (type_identifier) @class.name) @class
                    "#
                ).ok();
                
                let import_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (preproc_include) @import
                    "#
                ).ok();
                
                let highlight_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    ["if" "else" "for" "while" "do" "switch" "case" "return" "break" "continue"] @keyword.control
                    ["struct" "class" "enum" "typedef" "const" "static"] @keyword
                    (string_literal) @string
                    (number_literal) @number
                    (comment) @comment
                    (identifier) @variable
                    (type_identifier) @type
                    "#
                ).ok();
                
                (lang, symbol_query, import_query, highlight_query)
            },
            Language::Ruby => {
                let lang = tree_sitter_ruby::language();
                
                let symbol_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (method name: (identifier) @method.name) @method
                    (class name: (constant) @class.name) @class
                    (module name: (constant) @module.name) @module
                    (assignment left: (identifier) @variable.name) @variable
                    "#
                ).ok();
                
                let import_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (call method: (identifier) @method (#match? @method "^(require|require_relative|load)$")) @import
                    "#
                ).ok();
                
                let highlight_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    ["def" "class" "module" "end" "if" "elsif" "else" "unless" "case" "when" "for" "while" "until" "return" "break" "next"] @keyword
                    (string) @string
                    (integer) @number
                    (float) @number
                    (comment) @comment
                    (identifier) @variable
                    (constant) @type
                    "#
                ).ok();
                
                (lang, symbol_query, import_query, highlight_query)
            },
            Language::PHP => {
                let lang = tree_sitter_php::language_php();
                
                let symbol_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (function_definition name: (name) @function.name) @function
                    (class_declaration name: (name) @class.name) @class
                    (method_declaration name: (name) @method.name) @method
                    (property_declaration (property_element name: (variable_name) @property.name)) @property
                    "#
                ).ok();
                
                let import_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (include_expression) @import
                    (require_expression) @import
                    (include_once_expression) @import
                    (require_once_expression) @import
                    "#
                ).ok();
                
                let highlight_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    ["function" "class" "interface" "trait" "public" "private" "protected" "static" "final" "abstract" "if" "else" "elseif" "for" "foreach" "while" "do" "switch" "case" "return" "break" "continue"] @keyword
                    (string) @string
                    (integer) @number
                    (float) @number
                    (comment) @comment
                    (variable_name) @variable
                    (name) @type
                    "#
                ).ok();
                
                (lang, symbol_query, import_query, highlight_query)
            },
            Language::Swift => {
                let lang = tree_sitter_swift::LANGUAGE;
                
                let symbol_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (function_declaration name: (simple_identifier) @function.name) @function
                    (class_declaration name: (type_identifier) @class.name) @class
                    (struct_declaration name: (type_identifier) @struct.name) @struct
                    (protocol_declaration name: (type_identifier) @protocol.name) @protocol
                    (enum_declaration name: (type_identifier) @enum.name) @enum
                    "#
                ).ok();
                
                let import_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    (import_declaration) @import
                    "#
                ).ok();
                
                let highlight_query = tree_sitter::Query::new(
                    lang,
                    r#"
                    ["func" "class" "struct" "enum" "protocol" "var" "let" "if" "else" "for" "while" "switch" "case" "return" "break" "continue"] @keyword
                    (string_literal) @string
                    (integer_literal) @number
                    (real_literal) @number
                    (comment) @comment
                    (simple_identifier) @variable
                    (type_identifier) @type
                    "#
                ).ok();
                
                (lang, symbol_query, import_query, highlight_query)
            },
            _ => return Err(anyhow!("Unsupported language: {:?}", language)),
        };
        
        let mut parser = Parser::new();
        parser.set_language(ts_language)
            .context("Failed to set parser language")?;
        
        Ok(Self {
            language,
            ts_language,
            parser,
            symbol_query,
            import_query,
            highlight_query,
        })
    }
    
    /// Parse source code into AST
    #[instrument(skip(self, source))]
    pub fn parse(&mut self, source: &str) -> Result<ParseResult> {
        let start = Instant::now();
        
        let tree = self.parser.parse(source, None)
            .ok_or_else(|| anyhow!("Failed to parse source code"))?;
        
        let parse_time = start.elapsed();
        debug!(language = ?self.language, parse_time_ms = parse_time.as_millis(), "Parsed source code");
        
        let root_node = tree.root_node();
        let ast = self.convert_node_to_ast(root_node, source)?;
        
        let symbols = self.extract_symbols(&tree, source)?;
        let imports = self.extract_imports(&tree, source)?;
        let errors = self.extract_errors(&tree, source);
        let metrics = self.calculate_metrics(&ast, source);
        
        Ok(ParseResult {
            ast,
            symbols,
            imports,
            errors,
            metrics,
        })
    }
    
    /// Perform incremental parsing
    #[instrument(skip(self, _old_source, new_source))]
    pub fn parse_incremental(
        &mut self,
        _old_source: &str,
        new_source: &str,
        old_tree: &Tree,
        edit: &Edit,
    ) -> Result<(Tree, f64)> {
        let start = Instant::now();
        
        // Convert our Edit to tree-sitter InputEdit
        let ts_edit = InputEdit {
            start_byte: edit.start_byte,
            old_end_byte: edit.old_end_byte,
            new_end_byte: edit.new_end_byte,
            start_position: Point {
                row: edit.start_position.line,
                column: edit.start_position.column,
            },
            old_end_position: Point {
                row: edit.old_end_position.line,
                column: edit.old_end_position.column,
            },
            new_end_position: Point {
                row: edit.new_end_position.line,
                column: edit.new_end_position.column,
            },
        };
        
        // Edit the old tree
        let mut edited_tree = old_tree.clone();
        edited_tree.edit(&ts_edit);
        
        // Parse with the edited tree
        let new_tree = self.parser.parse(new_source, Some(&edited_tree))
            .ok_or_else(|| anyhow!("Failed to perform incremental parse"))?;
        
        let parse_time = start.elapsed().as_secs_f64() * 1000.0; // Convert to ms
        debug!(
            language = ?self.language,
            parse_time_ms = parse_time,
            "Incremental parse completed"
        );
        
        Ok((new_tree, parse_time))
    }
    
    /// Get syntax highlighting spans
    pub fn get_highlights(&self, tree: &Tree, source: &str) -> Result<Vec<HighlightSpan>> {
        let highlight_query = self.highlight_query.as_ref()
            .ok_or_else(|| anyhow!("No highlight query available for {:?}", self.language))?;
        
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(highlight_query, tree.root_node(), source.as_bytes());
        
        let mut highlights = Vec::new();
        
        for m in matches {
            for capture in m.captures {
                let node = capture.node;
                let capture_name = &highlight_query.capture_names()[capture.index as usize];
                
                let highlight_type = match capture_name.as_str() {
                    "keyword" | "keyword.control" | "keyword.function" => HighlightType::Keyword,
                    "function" | "function.name" => HighlightType::Function,
                    "type" | "type.name" => HighlightType::Type,
                    "variable" | "variable.name" => HighlightType::Variable,
                    "string" => HighlightType::String,
                    "number" => HighlightType::Number,
                    "comment" => HighlightType::Comment,
                    "operator" => HighlightType::Operator,
                    "punctuation" => HighlightType::Punctuation,
                    "constant" | "const.name" => HighlightType::Constant,
                    "parameter" => HighlightType::Parameter,
                    "property" => HighlightType::Property,
                    _ => continue,
                };
                
                highlights.push(HighlightSpan {
                    start: Position {
                        line: node.start_position().row,
                        column: node.start_position().column,
                        offset: node.start_byte(),
                    },
                    end: Position {
                        line: node.end_position().row,
                        column: node.end_position().column,
                        offset: node.end_byte(),
                    },
                    highlight_type,
                });
            }
        }
        
        highlights.sort_by_key(|h| h.start.offset);
        Ok(highlights)
    }
    
    /// Convert tree-sitter node to our AST representation
    pub fn convert_node_to_ast(&self, node: Node, source: &str) -> Result<AstNode> {
        let node_type = node.kind().to_string();
        let name = self.extract_node_name(&node, source);
        
        let start_pos = Position {
            line: node.start_position().row,
            column: node.start_position().column,
            offset: node.start_byte(),
        };
        
        let end_pos = Position {
            line: node.end_position().row,
            column: node.end_position().column,
            offset: node.end_byte(),
        };
        
        let metadata = self.extract_node_metadata(&node, source);
        
        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_extra() {
                children.push(self.convert_node_to_ast(child, source)?);
            }
        }
        
        Ok(AstNode {
            node_type,
            name,
            start_pos,
            end_pos,
            children,
            metadata,
        })
    }
    
    /// Extract node name if available
    fn extract_node_name(&self, node: &Node, source: &str) -> Option<String> {
        // Try to find a name child node
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "type_identifier" || child.kind() == "property_identifier" {
                return Some(child.utf8_text(source.as_bytes()).ok()?.to_string());
            }
        }
        None
    }
    
    /// Extract node metadata
    fn extract_node_metadata(&self, node: &Node, source: &str) -> NodeMetadata {
        let node_kind = node.kind();
        
        let is_definition = matches!(node_kind,
            "function_declaration" | "function_item" | "function_definition" |
            "class_declaration" | "class_definition" | "struct_item" |
            "interface_declaration" | "type_alias" | "const_declaration" |
            "variable_declaration" | "field_declaration"
        );
        
        let is_reference = matches!(node_kind,
            "call_expression" | "member_expression" | "identifier" |
            "type_identifier" | "property_identifier"
        );
        
        // Extract visibility/access modifiers
        let visibility = self.extract_visibility(node, source);
        
        // Extract doc comments
        let doc_comment = self.extract_doc_comment(node, source);
        
        // Simple complexity calculation based on node type
        let complexity = self.calculate_node_complexity(node);
        
        NodeMetadata {
            is_definition,
            is_reference,
            visibility,
            doc_comment,
            type_info: None, // Would require type inference
            complexity,
        }
    }
    
    /// Extract visibility modifiers
    fn extract_visibility(&self, node: &Node, _source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "public" | "private" | "protected" | "pub" => {
                    return Some(child.kind().to_string());
                },
                _ => continue,
            }
        }
        None
    }
    
    /// Extract documentation comments
    fn extract_doc_comment(&self, node: &Node, source: &str) -> Option<String> {
        // Look for preceding comment nodes
        if let Some(prev) = node.prev_sibling() {
            if prev.kind().contains("comment") {
                return prev.utf8_text(source.as_bytes()).ok().map(String::from);
            }
        }
        None
    }
    
    /// Calculate node complexity
    fn calculate_node_complexity(&self, node: &Node) -> u32 {
        let mut complexity = 1;
        
        // Add complexity for control flow nodes
        match node.kind() {
            "if_statement" | "if_expression" | "conditional_expression" => complexity += 1,
            "for_statement" | "for_expression" | "while_statement" | "while_expression" => complexity += 2,
            "match_expression" | "switch_statement" => complexity += 2,
            "try_statement" | "try_expression" => complexity += 1,
            _ => {},
        }
        
        complexity
    }
    
    /// Extract symbols from the AST
    pub fn extract_symbols(&self, tree: &Tree, source: &str) -> Result<Vec<Symbol>> {
        let symbol_query = match &self.symbol_query {
            Some(q) => q,
            None => return Ok(vec![]),
        };
        
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(symbol_query, tree.root_node(), source.as_bytes());
        
        let mut symbols = Vec::new();
        
        for m in matches {
            let mut symbol_name = None;
            let mut symbol_kind = SymbolKind::Variable;
            let mut symbol_node = None;
            
            for capture in m.captures {
                let capture_name = &symbol_query.capture_names()[capture.index as usize];
                
                if capture_name.ends_with(".name") {
                    symbol_name = capture.node.utf8_text(source.as_bytes()).ok().map(String::from);
                    symbol_node = Some(capture.node);
                }
                
                // Determine symbol kind from capture name
                symbol_kind = match capture_name.split('.').next().unwrap_or("") {
                    "function" => SymbolKind::Function,
                    "method" => SymbolKind::Method,
                    "class" => SymbolKind::Class,
                    "interface" => SymbolKind::Interface,
                    "struct" => SymbolKind::Struct,
                    "enum" => SymbolKind::Enum,
                    "const" | "constant" => SymbolKind::Constant,
                    "variable" | "var" => SymbolKind::Variable,
                    "module" | "mod" => SymbolKind::Module,
                    "type_alias" | "type" => SymbolKind::TypeAlias,
                    "import" => SymbolKind::Import,
                    _ => SymbolKind::Variable,
                };
            }
            
            if let (Some(name), Some(node)) = (symbol_name, symbol_node) {
                symbols.push(Symbol {
                    name,
                    kind: symbol_kind,
                    location: Position {
                        line: node.start_position().row,
                        column: node.start_position().column,
                        offset: node.start_byte(),
                    },
                    parent: None, // Would require scope analysis
                    signature: None, // Would require type analysis
                    docs: self.extract_doc_comment(&node, source),
                });
            }
        }
        
        Ok(symbols)
    }
    
    /// Extract import statements
    pub fn extract_imports(&self, tree: &Tree, source: &str) -> Result<Vec<ImportInfo>> {
        let import_query = match &self.import_query {
            Some(q) => q,
            None => return Ok(vec![]),
        };
        
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(import_query, tree.root_node(), source.as_bytes());
        
        let mut imports = Vec::new();
        
        for m in matches {
            for capture in m.captures {
                let node = capture.node;
                let import_text = node.utf8_text(source.as_bytes())?;
                
                // Parse import text based on language
                let (module, items, is_wildcard) = self.parse_import_text(import_text)?;
                
                imports.push(ImportInfo {
                    module,
                    items,
                    location: Position {
                        line: node.start_position().row,
                        column: node.start_position().column,
                        offset: node.start_byte(),
                    },
                    is_wildcard,
                });
            }
        }
        
        Ok(imports)
    }
    
    /// Parse import text to extract module and items
    fn parse_import_text(&self, import_text: &str) -> Result<(String, Vec<String>, bool)> {
        // This is a simplified implementation
        // In production, we'd parse based on language-specific syntax
        match self.language {
            Language::Rust => {
                if import_text.contains("use ") {
                    let parts: Vec<&str> = import_text.split("use ").collect();
                    if parts.len() > 1 {
                        let import_path = parts[1].trim_end_matches(';').trim();
                        let is_wildcard = import_path.contains("*");
                        return Ok((import_path.to_string(), vec![], is_wildcard));
                    }
                }
            },
            Language::Python => {
                if import_text.starts_with("import ") {
                    let module = import_text.trim_start_matches("import ").trim();
                    return Ok((module.to_string(), vec![], false));
                } else if import_text.starts_with("from ") {
                    let parts: Vec<&str> = import_text.split(" import ").collect();
                    if parts.len() == 2 {
                        let module = parts[0].trim_start_matches("from ").trim();
                        let items = parts[1].trim();
                        let is_wildcard = items == "*";
                        let items = if is_wildcard {
                            vec![]
                        } else {
                            items.split(',').map(|s| s.trim().to_string()).collect()
                        };
                        return Ok((module.to_string(), items, is_wildcard));
                    }
                }
            },
            _ => {
                // Generic parsing for other languages
                return Ok((import_text.to_string(), vec![], false));
            }
        }
        
        Ok((import_text.to_string(), vec![], false))
    }
    
    /// Extract parse errors
    pub fn extract_errors(&self, tree: &Tree, source: &str) -> Vec<ParseError> {
        let mut errors = Vec::new();
        let root = tree.root_node();
        
        self.collect_errors(&root, source, &mut errors);
        
        errors
    }
    
    /// Recursively collect parse errors
    fn collect_errors(&self, node: &Node, source: &str, errors: &mut Vec<ParseError>) {
        if node.is_error() {
            errors.push(ParseError {
                message: format!("Syntax error at {}", node.kind()),
                location: Position {
                    line: node.start_position().row,
                    column: node.start_position().column,
                    offset: node.start_byte(),
                },
                severity: ErrorSeverity::Error,
            });
        } else if node.is_missing() {
            errors.push(ParseError {
                message: format!("Missing {}", node.kind()),
                location: Position {
                    line: node.start_position().row,
                    column: node.start_position().column,
                    offset: node.start_byte(),
                },
                severity: ErrorSeverity::Error,
            });
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_errors(&child, source, errors);
        }
    }
    
    /// Calculate code metrics
    pub fn calculate_metrics(&self, ast: &AstNode, source: &str) -> CodeMetrics {
        let lines: Vec<&str> = source.lines().collect();
        let mut code_lines = 0;
        let mut comment_lines = 0;
        
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            // Language-specific comment detection
            let is_comment = match self.language {
                Language::Rust | Language::Cpp | Language::C | Language::Java | Language::Go | Language::JavaScript | Language::TypeScript => {
                    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*")
                },
                Language::Python => trimmed.starts_with("#"),
                _ => false,
            };
            
            if is_comment {
                comment_lines += 1;
            } else {
                code_lines += 1;
            }
        }
        
        CodeMetrics {
            lines_of_code: code_lines,
            comment_lines,
            complexity: self.calculate_ast_complexity(ast),
            function_count: self.count_ast_nodes(ast, &["function", "method"]),
            class_count: self.count_ast_nodes(ast, &["class", "struct", "interface"]),
            max_nesting: self.calculate_max_nesting(ast, 0),
        }
    }
    
    /// Calculate AST complexity
    fn calculate_ast_complexity(&self, ast: &AstNode) -> u32 {
        let mut complexity = ast.metadata.complexity;
        
        for child in &ast.children {
            complexity += self.calculate_ast_complexity(child);
        }
        
        complexity
    }
    
    /// Count specific node types in AST
    fn count_ast_nodes(&self, ast: &AstNode, node_types: &[&str]) -> usize {
        let mut count = 0;
        
        for node_type in node_types {
            if ast.node_type.contains(node_type) {
                count += 1;
            }
        }
        
        for child in &ast.children {
            count += self.count_ast_nodes(child, node_types);
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
}

/// Parser registry for managing multiple language parsers
pub struct ParserRegistry {
    parsers: HashMap<Language, Arc<tokio::sync::Mutex<TreeSitterParser>>>,
}

impl ParserRegistry {
    /// Create a new parser registry
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
        }
    }
    
    /// Get or create a parser for the given language
    pub async fn get_parser(&mut self, language: Language) -> Result<Arc<tokio::sync::Mutex<TreeSitterParser>>> {
        if let Some(parser) = self.parsers.get(&language) {
            return Ok(Arc::clone(parser));
        }
        
        let parser = TreeSitterParser::new(language)?;
        let parser = Arc::new(tokio::sync::Mutex::new(parser));
        self.parsers.insert(language, Arc::clone(&parser));
        
        Ok(parser)
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_parser() {
        let mut parser = TreeSitterParser::new(Language::Rust).unwrap();
        let source = r#"
fn main() {
    println!("Hello, world!");
}

struct Point {
    x: f64,
    y: f64,
}
"#;
        
        let result = parser.parse(source).unwrap();
        assert_eq!(result.metrics.function_count, 1);
        assert_eq!(result.metrics.class_count, 1); // struct counts as class
        assert!(result.symbols.iter().any(|s| s.name == "main" && s.kind == SymbolKind::Function));
        assert!(result.symbols.iter().any(|s| s.name == "Point" && s.kind == SymbolKind::Struct));
    }
    
    #[test]
    fn test_incremental_parsing() {
        let mut parser = TreeSitterParser::new(Language::Rust).unwrap();
        
        let old_source = "fn main() {}";
        let new_source = "fn main() { println!(\"Hello\"); }";
        
        let tree = parser.parser.parse(old_source, None).unwrap();
        
        let edit = Edit {
            start_byte: 10,
            old_end_byte: 10,
            new_end_byte: 28,
            start_position: Position { line: 0, column: 10, offset: 10 },
            old_end_position: Position { line: 0, column: 10, offset: 10 },
            new_end_position: Position { line: 0, column: 28, offset: 28 },
        };
        
        let (new_tree, parse_time) = parser.parse_incremental(old_source, new_source, &tree, &edit).unwrap();
        assert!(parse_time < 5.0); // Should be less than 5ms
        assert!(!new_tree.root_node().has_error());
    }
    
    #[test]
    fn test_syntax_highlighting() {
        let mut parser = TreeSitterParser::new(Language::Rust).unwrap();
        let source = r#"fn main() {
    let x = 42;
    println!("Hello");
}"#;
        
        let tree = parser.parser.parse(source, None).unwrap();
        let highlights = parser.get_highlights(&tree, source).unwrap();
        
        assert!(highlights.iter().any(|h| h.highlight_type == HighlightType::Keyword));
        assert!(highlights.iter().any(|h| h.highlight_type == HighlightType::String));
        assert!(highlights.iter().any(|h| h.highlight_type == HighlightType::Number));
    }
}