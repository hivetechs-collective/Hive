//! Language Service Integration
//! 
//! Provides language-specific features like auto-completion, hover info, and diagnostics

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, Diagnostic, DiagnosticSeverity,
    DidOpenTextDocumentParams, GotoDefinitionParams, HoverParams, InitializeParams,
    Position as LspPosition, Range as LspRange, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LanguageService {
    language_id: String,
    /// Mock completion items for demonstration
    completions: HashMap<String, Vec<CompletionItem>>,
}

impl LanguageService {
    pub fn new(language_id: &str) -> Self {
        let mut service = Self {
            language_id: language_id.to_string(),
            completions: HashMap::new(),
        };
        
        // Initialize with some mock completions for each language
        service.init_mock_completions();
        service
    }
    
    fn init_mock_completions(&mut self) {
        match self.language_id.as_str() {
            "rust" => {
                self.add_rust_completions();
            }
            "javascript" | "typescript" => {
                self.add_javascript_completions();
            }
            "python" => {
                self.add_python_completions();
            }
            _ => {}
        }
    }
    
    fn add_rust_completions(&mut self) {
        let mut completions = vec![];
        
        // Common Rust keywords
        for keyword in ["fn", "let", "mut", "const", "struct", "enum", "impl", "trait", "pub", "use", "mod", "async", "await"] {
            completions.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("Rust keyword: {}", keyword)),
                ..Default::default()
            });
        }
        
        // Common types
        for type_name in ["String", "Vec", "HashMap", "Option", "Result", "Box", "Arc", "Mutex"] {
            completions.push(CompletionItem {
                label: type_name.to_string(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some(format!("Type: {}", type_name)),
                ..Default::default()
            });
        }
        
        // Common macros
        for macro_name in ["println!", "vec!", "format!", "panic!", "dbg!", "todo!", "unimplemented!"] {
            completions.push(CompletionItem {
                label: macro_name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("Macro: {}", macro_name)),
                ..Default::default()
            });
        }
        
        self.completions.insert("rust".to_string(), completions);
    }
    
    fn add_javascript_completions(&mut self) {
        let mut completions = vec![];
        
        // JavaScript keywords
        for keyword in ["const", "let", "var", "function", "class", "async", "await", "return", "if", "else", "for", "while"] {
            completions.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("JavaScript keyword: {}", keyword)),
                ..Default::default()
            });
        }
        
        // Common methods
        for method in ["console.log", "Array.from", "Object.keys", "Promise.all", "JSON.stringify", "JSON.parse"] {
            completions.push(CompletionItem {
                label: method.to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some(format!("Method: {}", method)),
                ..Default::default()
            });
        }
        
        self.completions.insert("javascript".to_string(), completions.clone());
        self.completions.insert("typescript".to_string(), completions);
    }
    
    fn add_python_completions(&mut self) {
        let mut completions = vec![];
        
        // Python keywords
        for keyword in ["def", "class", "import", "from", "if", "elif", "else", "for", "while", "return", "async", "await"] {
            completions.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("Python keyword: {}", keyword)),
                ..Default::default()
            });
        }
        
        // Built-in functions
        for func in ["print", "len", "range", "enumerate", "zip", "map", "filter", "sum", "min", "max"] {
            completions.push(CompletionItem {
                label: func.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("Built-in function: {}", func)),
                ..Default::default()
            });
        }
        
        self.completions.insert("python".to_string(), completions);
    }
    
    /// Get completion suggestions at the current position
    pub async fn get_completions(&self, text: &str, line: usize, column: usize) -> Vec<CompletionItem> {
        // Get the current word being typed
        let lines: Vec<&str> = text.lines().collect();
        if let Some(current_line) = lines.get(line) {
            let prefix = Self::get_word_at_position(current_line, column);
            
            // Filter completions based on prefix
            if let Some(all_completions) = self.completions.get(&self.language_id) {
                return all_completions
                    .iter()
                    .filter(|item| item.label.starts_with(&prefix))
                    .cloned()
                    .collect();
            }
        }
        
        Vec::new()
    }
    
    /// Get hover information at position
    pub async fn get_hover_info(&self, text: &str, line: usize, column: usize) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        if let Some(current_line) = lines.get(line) {
            let word = Self::get_word_at_position(current_line, column);
            
            // Provide mock hover info based on word
            match self.language_id.as_str() {
                "rust" => match word.as_str() {
                    "String" => Some("A UTF-8 encoded, growable string.".to_string()),
                    "Vec" => Some("A contiguous growable array type.".to_string()),
                    "Option" => Some("The Option type represents an optional value.".to_string()),
                    _ => None,
                },
                "javascript" | "typescript" => match word.as_str() {
                    "console" => Some("The console object provides access to the browser's debugging console.".to_string()),
                    "Array" => Some("The JavaScript Array object is a global object used in the construction of arrays.".to_string()),
                    _ => None,
                },
                "python" => match word.as_str() {
                    "print" => Some("print(value, ..., sep=' ', end='\\n', file=sys.stdout, flush=False)".to_string()),
                    "len" => Some("Return the length (the number of items) of an object.".to_string()),
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }
    
    /// Get diagnostics (errors, warnings) for the current text
    pub async fn get_diagnostics(&self, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Simple example diagnostics
        match self.language_id.as_str() {
            "rust" => {
                // Check for missing semicolons
                for (line_num, line) in text.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("let") && !trimmed.ends_with(';') && !trimmed.ends_with('{') {
                        diagnostics.push(Diagnostic {
                            range: LspRange {
                                start: LspPosition {
                                    line: line_num as u32,
                                    character: line.len() as u32,
                                },
                                end: LspPosition {
                                    line: line_num as u32,
                                    character: line.len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: "Missing semicolon".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            "python" => {
                // Check for tabs vs spaces
                for (line_num, line) in text.lines().enumerate() {
                    if line.starts_with('\t') {
                        diagnostics.push(Diagnostic {
                            range: LspRange {
                                start: LspPosition {
                                    line: line_num as u32,
                                    character: 0,
                                },
                                end: LspPosition {
                                    line: line_num as u32,
                                    character: 1,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: "Use spaces instead of tabs".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            _ => {}
        }
        
        diagnostics
    }
    
    /// Helper to extract word at position
    fn get_word_at_position(line: &str, column: usize) -> String {
        let chars: Vec<char> = line.chars().collect();
        
        // Find word boundaries
        let mut start = column;
        let mut end = column;
        
        // Move start back to beginning of word
        while start > 0 && chars.get(start - 1).map(|c| c.is_alphanumeric() || *c == '_').unwrap_or(false) {
            start -= 1;
        }
        
        // Move end forward to end of word
        while end < chars.len() && chars.get(end).map(|c| c.is_alphanumeric() || *c == '_').unwrap_or(false) {
            end += 1;
        }
        
        chars[start..end].iter().collect()
    }
}

/// Represents a completion suggestion with AI enhancement
#[derive(Debug, Clone)]
pub struct AIEnhancedCompletion {
    pub base_completion: CompletionItem,
    pub ai_confidence: f32,
    pub ai_explanation: Option<String>,
    pub is_from_consensus: bool,
}