//! Syntax Highlighting using Tree-sitter
//! 
//! Provides language-aware syntax highlighting with theme support

use tree_sitter::{Language, Parser, Query, QueryCursor, Node};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HighlightStyle {
    pub color: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub selection: String,
    pub cursor: String,
    pub line_number: String,
    pub current_line: String,
    pub styles: HashMap<String, HighlightStyle>,
}

impl Theme {
    pub fn hivetechs_dark() -> Self {
        let mut styles = HashMap::new();
        
        // Define highlight styles
        styles.insert("keyword".to_string(), HighlightStyle {
            color: "#FFC107".to_string(),
            bold: true,
            italic: false,
            underline: false,
        });
        
        styles.insert("function".to_string(), HighlightStyle {
            color: "#007BFF".to_string(),
            bold: false,
            italic: false,
            underline: false,
        });
        
        styles.insert("string".to_string(), HighlightStyle {
            color: "#4CAF50".to_string(),
            bold: false,
            italic: false,
            underline: false,
        });
        
        styles.insert("comment".to_string(), HighlightStyle {
            color: "#808080".to_string(),
            bold: false,
            italic: true,
            underline: false,
        });
        
        styles.insert("variable".to_string(), HighlightStyle {
            color: "#9CDCFE".to_string(),
            bold: false,
            italic: false,
            underline: false,
        });
        
        styles.insert("type".to_string(), HighlightStyle {
            color: "#4EC9B0".to_string(),
            bold: false,
            italic: false,
            underline: false,
        });
        
        styles.insert("constant".to_string(), HighlightStyle {
            color: "#D4D4D4".to_string(),
            bold: false,
            italic: false,
            underline: false,
        });
        
        styles.insert("operator".to_string(), HighlightStyle {
            color: "#D4D4D4".to_string(),
            bold: false,
            italic: false,
            underline: false,
        });
        
        Theme {
            name: "HiveTechs Dark".to_string(),
            background: "#0E1414".to_string(),
            foreground: "#FFFFFF".to_string(),
            selection: "#264F78".to_string(),
            cursor: "#FFC107".to_string(),
            line_number: "#858585".to_string(),
            current_line: "#181E21".to_string(),
            styles,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HighlightedSpan {
    pub start: usize,
    pub end: usize,
    pub style: Option<HighlightStyle>,
}

pub struct SyntaxHighlighter {
    languages: HashMap<String, Language>,
    parsers: HashMap<String, Parser>,
    theme: Theme,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let mut languages = HashMap::new();
        let mut parsers = HashMap::new();
        
        // Initialize languages with their parsers
        Self::setup_language(&mut languages, &mut parsers, "rust", tree_sitter_rust::language());
        Self::setup_language(&mut languages, &mut parsers, "javascript", tree_sitter_javascript::language());
        Self::setup_language(&mut languages, &mut parsers, "typescript", tree_sitter_typescript::language_typescript());
        Self::setup_language(&mut languages, &mut parsers, "python", tree_sitter_python::language());
        Self::setup_language(&mut languages, &mut parsers, "go", tree_sitter_go::language());
        
        Self {
            languages,
            parsers,
            theme: Theme::hivetechs_dark(),
        }
    }
    
    fn setup_language(
        languages: &mut HashMap<String, Language>,
        parsers: &mut HashMap<String, Parser>,
        name: &str,
        language: Language,
    ) {
        languages.insert(name.to_string(), language);
        
        let mut parser = Parser::new();
        parser.set_language(language).expect("Error loading grammar");
        parsers.insert(name.to_string(), parser);
    }
    
    /// Get syntax highlights for a piece of code
    pub fn highlight(&mut self, code: &str, language_id: &str) -> Vec<HighlightedSpan> {
        let mut spans = Vec::new();
        
        // Get parser for the language
        if let Some(parser) = self.parsers.get_mut(language_id) {
            // Parse the code
            if let Some(tree) = parser.parse(code, None) {
                // For now, we'll do simple keyword highlighting
                // In a real implementation, we'd use proper tree-sitter queries
                let root = tree.root_node();
                self.highlight_node(&root, code, &mut spans);
            }
        }
        
        // If no highlighting available, return a single span with no style
        if spans.is_empty() {
            spans.push(HighlightedSpan {
                start: 0,
                end: code.len(),
                style: None,
            });
        }
        
        spans
    }
    
    fn highlight_node(&self, node: Node, source: &str, spans: &mut Vec<HighlightedSpan>) {
        // Map node kinds to highlight styles
        let style = match node.kind() {
            // Keywords
            "let" | "const" | "var" | "fn" | "function" | "def" | "class" | "struct" | "enum" |
            "if" | "else" | "for" | "while" | "return" | "break" | "continue" | "async" | "await" |
            "use" | "import" | "from" | "export" | "pub" | "private" | "public" => {
                Some(self.theme.styles.get("keyword").cloned().unwrap())
            }
            
            // Functions
            "function_item" | "method_definition" | "function_definition" => {
                Some(self.theme.styles.get("function").cloned().unwrap())
            }
            
            // Strings
            "string_literal" | "string" | "template_string" => {
                Some(self.theme.styles.get("string").cloned().unwrap())
            }
            
            // Comments
            "comment" | "line_comment" | "block_comment" => {
                Some(self.theme.styles.get("comment").cloned().unwrap())
            }
            
            // Types
            "type_identifier" | "primitive_type" => {
                Some(self.theme.styles.get("type").cloned().unwrap())
            }
            
            // Variables
            "identifier" if node.parent().map(|p| p.kind() == "let_declaration").unwrap_or(false) => {
                Some(self.theme.styles.get("variable").cloned().unwrap())
            }
            
            _ => None,
        };
        
        if let Some(style) = style {
            spans.push(HighlightedSpan {
                start: node.start_byte(),
                end: node.end_byte(),
                style: Some(style),
            });
        }
        
        // Recursively highlight children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.highlight_node(child, source, spans);
            }
        }
    }
    
    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }
    
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
}

/// Get language ID from file extension
pub fn get_language_from_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "rs" => Some("rust"),
        "js" => Some("javascript"),
        "ts" => Some("typescript"),
        "py" => Some("python"),
        "go" => Some("go"),
        _ => None,
    }
}