//! Contextual documentation with AI-powered insights
//!
//! Provides intelligent documentation generation and contextual help

use super::protocol::*;
use crate::core::{HiveError, Result};
use crate::consensus::ConsensusEngine;
use crate::analysis::AnalysisEngine;

use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;
use tracing::{info, debug, error, warn};
use std::collections::HashMap;
use std::time::Instant;

/// Contextual documentation provider
pub struct ContextualDocumentationProvider {
    consensus_engine: Arc<RwLock<ConsensusEngine>>,
    analysis_engine: Arc<AnalysisEngine>,
    config: DocumentationConfig,
    documentation_cache: Arc<RwLock<HashMap<String, CachedDocumentation>>>,
}

/// Documentation configuration
#[derive(Debug, Clone)]
pub struct DocumentationConfig {
    /// Enable AI-powered documentation generation
    pub ai_generation: bool,
    /// Enable contextual hints
    pub contextual_hints: bool,
    /// Enable signature help
    pub signature_help: bool,
    /// Enable hover documentation
    pub hover_docs: bool,
    /// Enable inline documentation
    pub inline_docs: bool,
    /// Documentation style (brief, detailed, comprehensive)
    pub documentation_style: DocumentationStyle,
    /// Maximum documentation length
    pub max_documentation_length: usize,
    /// Cache timeout in seconds
    pub cache_timeout_seconds: u64,
    /// Performance tracking
    pub track_performance: bool,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            ai_generation: true,
            contextual_hints: true,
            signature_help: true,
            hover_docs: true,
            inline_docs: true,
            documentation_style: DocumentationStyle::Detailed,
            max_documentation_length: 2000,
            cache_timeout_seconds: 300, // 5 minutes
            track_performance: false,
        }
    }
}

/// Documentation style
#[derive(Debug, Clone)]
pub enum DocumentationStyle {
    Brief,      // Concise, one-line descriptions
    Detailed,   // Comprehensive with examples
    Comprehensive, // Full documentation with context
}

/// Cached documentation
#[derive(Debug, Clone)]
struct CachedDocumentation {
    content: DocumentationContent,
    timestamp: std::time::SystemTime,
    language: String,
}

/// Documentation content
#[derive(Debug, Clone)]
pub struct DocumentationContent {
    pub summary: String,
    pub description: String,
    pub parameters: Vec<ParameterDoc>,
    pub return_type: Option<ReturnDoc>,
    pub examples: Vec<ExampleDoc>,
    pub see_also: Vec<String>,
    pub notes: Vec<String>,
    pub warnings: Vec<String>,
    pub complexity: Option<ComplexityDoc>,
    pub ai_generated: bool,
}

/// Parameter documentation
#[derive(Debug, Clone)]
pub struct ParameterDoc {
    pub name: String,
    pub type_name: String,
    pub description: String,
    pub optional: bool,
    pub default_value: Option<String>,
}

/// Return type documentation
#[derive(Debug, Clone)]
pub struct ReturnDoc {
    pub type_name: String,
    pub description: String,
    pub possible_values: Vec<String>,
}

/// Example documentation
#[derive(Debug, Clone)]
pub struct ExampleDoc {
    pub title: String,
    pub code: String,
    pub description: String,
    pub language: String,
}

/// Complexity documentation
#[derive(Debug, Clone)]
pub struct ComplexityDoc {
    pub time_complexity: String,
    pub space_complexity: String,
    pub explanation: String,
}

/// Signature help information
#[derive(Debug, Clone)]
pub struct SignatureHelpInfo {
    pub signatures: Vec<SignatureInformation>,
    pub active_signature: Option<u32>,
    pub active_parameter: Option<u32>,
}

/// Signature information
#[derive(Debug, Clone)]
pub struct SignatureInformation {
    pub label: String,
    pub documentation: Option<MarkupContent>,
    pub parameters: Vec<ParameterInformation>,
    pub active_parameter: Option<u32>,
}

/// Parameter information
#[derive(Debug, Clone)]
pub struct ParameterInformation {
    pub label: String,
    pub documentation: Option<MarkupContent>,
}

/// Hover information with AI insights
#[derive(Debug, Clone)]
pub struct EnhancedHover {
    pub base: Hover,
    pub ai_insights: Option<String>,
    pub context_information: Vec<String>,
    pub usage_examples: Vec<String>,
    pub related_symbols: Vec<String>,
}

impl ContextualDocumentationProvider {
    /// Create new documentation provider
    pub async fn new(
        consensus_engine: Arc<RwLock<ConsensusEngine>>,
        analysis_engine: Arc<AnalysisEngine>,
        config: Option<DocumentationConfig>,
    ) -> Result<Self> {
        Ok(Self {
            consensus_engine,
            analysis_engine,
            config: config.unwrap_or_default(),
            documentation_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Provide hover documentation
    pub async fn provide_hover(
        &self,
        params: &TextDocumentPositionParams,
        document_content: &str,
        language: &str,
    ) -> Result<Option<EnhancedHover>> {
        let start_time = if self.config.track_performance {
            Some(Instant::now())
        } else {
            None
        };

        debug!(
            \"Providing hover documentation at {}:{} in {}\",
            params.position.line, params.position.character, params.text_document.uri
        );

        // Extract symbol at position
        let symbol = self.extract_symbol_at_position(
            document_content,
            &params.position,
            language,
        ).await?;

        if symbol.is_none() {
            return Ok(None);
        }

        let symbol = symbol.unwrap();

        // Check cache first
        if let Some(cached) = self.get_cached_documentation(&symbol.name, language).await {
            if !self.is_cache_expired(&cached) {
                let hover = self.create_hover_from_documentation(&cached.content, &symbol)?;
                return Ok(Some(hover));
            }
        }

        // Generate documentation
        let documentation = self.generate_documentation(&symbol, document_content, language).await?;

        // Cache the result
        self.cache_documentation(&symbol.name, &documentation, language).await;

        // Create hover response
        let enhanced_hover = self.create_enhanced_hover(&documentation, &symbol, document_content, language).await?;

        if let Some(start) = start_time {
            debug!(
                \"Hover documentation generated in {:?}\",
                start.elapsed()
            );
        }

        Ok(Some(enhanced_hover))
    }

    /// Provide signature help
    pub async fn provide_signature_help(
        &self,
        params: &TextDocumentPositionParams,
        document_content: &str,
        language: &str,
        trigger_character: Option<String>,
    ) -> Result<Option<SignatureHelpInfo>> {
        let start_time = if self.config.track_performance {
            Some(Instant::now())
        } else {
            None
        };

        debug!(
            \"Providing signature help at {}:{} in {}\",
            params.position.line, params.position.character, params.text_document.uri
        );

        // Find function call context
        let function_context = self.extract_function_call_context(
            document_content,
            &params.position,
            language,
        ).await?;

        if function_context.is_none() {
            return Ok(None);
        }

        let function_context = function_context.unwrap();

        // Get function signatures
        let signatures = self.get_function_signatures(&function_context, document_content, language).await?;

        if signatures.is_empty() {
            return Ok(None);
        }

        // Determine active signature and parameter
        let (active_signature, active_parameter) = self.determine_active_signature_and_parameter(
            &function_context,
            &signatures,
        ).await;

        let signature_help = SignatureHelpInfo {
            signatures,
            active_signature,
            active_parameter,
        };

        if let Some(start) = start_time {
            debug!(
                \"Signature help generated in {:?}\",
                start.elapsed()
            );
        }

        Ok(Some(signature_help))
    }

    /// Generate documentation for symbol
    pub async fn generate_documentation(
        &self,
        symbol: &SymbolInfo,
        context: &str,
        language: &str,
    ) -> Result<DocumentationContent> {
        debug!(\"Generating documentation for symbol: {} ({})\", symbol.name, symbol.kind);

        let mut documentation = DocumentationContent {
            summary: String::new(),
            description: String::new(),
            parameters: Vec::new(),
            return_type: None,
            examples: Vec::new(),
            see_also: Vec::new(),
            notes: Vec::new(),
            warnings: Vec::new(),
            complexity: None,
            ai_generated: false,
        };

        // 1. Basic symbol analysis
        self.analyze_symbol_basic(&mut documentation, symbol, context, language).await?;

        // 2. AI-powered enhancement
        if self.config.ai_generation {
            self.enhance_documentation_with_ai(&mut documentation, symbol, context, language).await?;
        }

        // 3. Add examples based on symbol type
        self.add_contextual_examples(&mut documentation, symbol, language).await?;

        // 4. Calculate complexity if applicable
        if matches!(symbol.kind.as_str(), \"function\" | \"method\") {
            documentation.complexity = self.calculate_symbol_complexity(symbol, context).await?;
        }

        Ok(documentation)
    }

    /// Extract symbol at position
    async fn extract_symbol_at_position(
        &self,
        content: &str,
        position: &Position,
        language: &str,
    ) -> Result<Option<SymbolInfo>> {
        // Parse the document
        let parse_result = self.analysis_engine.parse_code(content, Some(language)).await?;

        // Find symbol at position
        for symbol in parse_result.symbols {
            if self.is_position_in_symbol(&symbol, position) {
                return Ok(Some(SymbolInfo {
                    name: symbol.name,
                    kind: symbol.kind,
                    line: symbol.line,
                    column: symbol.column,
                    scope: symbol.scope,
                    type_info: symbol.type_info,
                    documentation: symbol.documentation,
                    signature: None,
                    parameters: Vec::new(),
                }));
            }
        }

        // If no exact symbol found, try to extract word at position
        let word = self.extract_word_at_position(content, position)?;
        if let Some(word) = word {
            return Ok(Some(SymbolInfo {
                name: word,
                kind: \"unknown\".to_string(),
                line: position.line as usize + 1,
                column: position.character as usize + 1,
                scope: None,
                type_info: None,
                documentation: None,
                signature: None,
                parameters: Vec::new(),
            }));
        }

        Ok(None)
    }

    /// Extract function call context
    async fn extract_function_call_context(
        &self,
        content: &str,
        position: &Position,
        language: &str,
    ) -> Result<Option<FunctionCallContext>> {
        let lines: Vec<&str> = content.lines().collect();
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;

        if line_idx >= lines.len() {
            return Ok(None);
        }

        let line = lines[line_idx];
        let text_before_cursor = if char_idx <= line.len() {
            &line[..char_idx]
        } else {
            line
        };

        // Look for function call pattern
        if let Some(function_call) = self.parse_function_call(text_before_cursor) {
            return Ok(Some(function_call));
        }

        // Look in previous lines for multiline function calls
        for i in (line_idx.saturating_sub(5)..=line_idx).rev() {
            if i < lines.len() {
                let search_text = if i == line_idx {
                    text_before_cursor
                } else {
                    lines[i]
                };

                if let Some(function_call) = self.parse_function_call(search_text) {
                    return Ok(Some(function_call));
                }
            }
        }

        Ok(None)
    }

    /// Analyze symbol basic information
    async fn analyze_symbol_basic(
        &self,
        documentation: &mut DocumentationContent,
        symbol: &SymbolInfo,
        context: &str,
        language: &str,
    ) -> Result<()> {
        // Set basic summary
        documentation.summary = match symbol.kind.as_str() {
            \"function\" | \"method\" => format!(\"Function: {}\", symbol.name),
            \"class\" => format!(\"Class: {}\", symbol.name),
            \"interface\" => format!(\"Interface: {}\", symbol.name),
            \"variable\" => format!(\"Variable: {}\", symbol.name),
            \"constant\" => format!(\"Constant: {}\", symbol.name),
            \"type\" => format!(\"Type: {}\", symbol.name),
            _ => format!(\"Symbol: {}\", symbol.name),
        };

        // Use existing documentation if available
        if let Some(existing_doc) = &symbol.documentation {
            documentation.description = existing_doc.clone();
        }

        // Extract type information
        if let Some(type_info) = &symbol.type_info {
            documentation.description.push_str(&format!(\"\\n\\nType: {}\", type_info));
        }

        // Extract parameters for functions
        if matches!(symbol.kind.as_str(), \"function\" | \"method\") {
            for param in &symbol.parameters {
                documentation.parameters.push(ParameterDoc {
                    name: param.name.clone(),
                    type_name: param.type_name.clone().unwrap_or(\"unknown\".to_string()),
                    description: param.description.clone().unwrap_or_default(),
                    optional: param.optional.unwrap_or(false),
                    default_value: param.default_value.clone(),
                });
            }
        }

        Ok(())
    }

    /// Enhance documentation with AI
    async fn enhance_documentation_with_ai(
        &self,
        documentation: &mut DocumentationContent,
        symbol: &SymbolInfo,
        context: &str,
        language: &str,
    ) -> Result<()> {
        let consensus = self.consensus_engine.read().await;

        let documentation_prompt = format!(
            \"Generate comprehensive documentation for this {} symbol in {}:\\n\\nSymbol: {} ({})\\nContext:\\n```{}\\n{}\\n```\\n\\nProvide:\\n1. Clear description\\n2. Usage examples\\n3. Parameters (if applicable)\\n4. Return value (if applicable)\\n5. Notes and warnings\\n6. Complexity analysis (if applicable)\",
            symbol.kind, language, symbol.name, symbol.kind, language, context
        );

        match consensus.ask(&documentation_prompt).await {
            Ok(response) => {
                let ai_doc = self.parse_ai_documentation(&response.content)?;
                self.merge_ai_documentation(documentation, ai_doc);
                documentation.ai_generated = true;
            }
            Err(e) => {
                warn!(\"AI documentation generation failed: {}\", e);
            }
        }

        Ok(())
    }

    /// Add contextual examples
    async fn add_contextual_examples(
        &self,
        documentation: &mut DocumentationContent,
        symbol: &SymbolInfo,
        language: &str,
    ) -> Result<()> {
        match symbol.kind.as_str() {
            \"function\" | \"method\" => {
                let example = self.generate_function_example(symbol, language).await?;
                documentation.examples.push(example);
            }
            \"class\" => {
                let example = self.generate_class_example(symbol, language).await?;
                documentation.examples.push(example);
            }
            \"variable\" | \"constant\" => {
                let example = self.generate_variable_example(symbol, language).await?;
                documentation.examples.push(example);
            }
            _ => {}
        }

        Ok(())
    }

    /// Calculate symbol complexity
    async fn calculate_symbol_complexity(
        &self,
        symbol: &SymbolInfo,
        context: &str,
    ) -> Result<Option<ComplexityDoc>> {
        // Simple complexity analysis
        if matches!(symbol.kind.as_str(), \"function\" | \"method\") {
            let lines = context.lines().count();
            let conditions = context.matches(\"if\").count() + context.matches(\"while\").count() + context.matches(\"for\").count();
            
            let time_complexity = if conditions == 0 {
                \"O(1)\".to_string()
            } else if conditions == 1 {
                \"O(n)\".to_string()
            } else {
                format!(\"O(n^{})\", conditions)
            };

            return Ok(Some(ComplexityDoc {
                time_complexity: time_complexity.clone(),
                space_complexity: \"O(1)\".to_string(),
                explanation: format!(\"Estimated {} time complexity based on {} conditional statements\", time_complexity, conditions),
            }));
        }

        Ok(None)
    }

    /// Create enhanced hover
    async fn create_enhanced_hover(
        &self,
        documentation: &DocumentationContent,
        symbol: &SymbolInfo,
        context: &str,
        language: &str,
    ) -> Result<EnhancedHover> {
        let mut content_parts = Vec::new();

        // Summary
        content_parts.push(format!(\"**{}**\", documentation.summary));

        // Description
        if !documentation.description.is_empty() {
            content_parts.push(documentation.description.clone());
        }

        // Parameters
        if !documentation.parameters.is_empty() {
            content_parts.push(\"**Parameters:**\".to_string());
            for param in &documentation.parameters {
                let optional_marker = if param.optional { \"?\" } else { \"\" };
                let default_marker = if let Some(default) = &param.default_value {
                    format!(\" = {}\", default)
                } else {
                    String::new()
                };
                content_parts.push(format!(
                    \"- `{}{}: {}{}` - {}\",
                    param.name, optional_marker, param.type_name, default_marker, param.description
                ));
            }
        }

        // Return type
        if let Some(return_doc) = &documentation.return_type {
            content_parts.push(format!(\"**Returns:** `{}` - {}\", return_doc.type_name, return_doc.description));
        }

        // Examples
        if !documentation.examples.is_empty() {
            content_parts.push(\"**Example:**\".to_string());
            for example in &documentation.examples {
                content_parts.push(format!(\"```{}\\n{}\\n```\", example.language, example.code));
                if !example.description.is_empty() {
                    content_parts.push(example.description.clone());
                }
            }
        }

        // Complexity
        if let Some(complexity) = &documentation.complexity {
            content_parts.push(format!(
                \"**Complexity:** Time: {}, Space: {}\\n{}\",
                complexity.time_complexity,
                complexity.space_complexity,
                complexity.explanation
            ));
        }

        // Warnings
        if !documentation.warnings.is_empty() {
            content_parts.push(\"**⚠️ Warnings:**\".to_string());
            for warning in &documentation.warnings {
                content_parts.push(format!(\"- {}\", warning));
            }
        }

        let content = content_parts.join(\"\\n\\n\");

        // Create base hover
        let base_hover = Hover {
            contents: MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            },
            range: Some(Range {
                start: Position {
                    line: (symbol.line - 1) as u32,
                    character: (symbol.column - 1) as u32,
                },
                end: Position {
                    line: (symbol.line - 1) as u32,
                    character: (symbol.column + symbol.name.len() - 1) as u32,
                },
            }),
        };

        // Get AI insights
        let ai_insights = if documentation.ai_generated {
            Some(\"Enhanced with AI-powered analysis\".to_string())
        } else {
            None
        };

        // Get context information
        let context_information = self.get_context_information(symbol, context, language).await?;

        // Get usage examples
        let usage_examples = documentation.examples.iter()
            .map(|e| e.code.clone())
            .collect();

        // Get related symbols
        let related_symbols = self.find_related_symbols(symbol, context, language).await?;

        Ok(EnhancedHover {
            base: base_hover,
            ai_insights,
            context_information,
            usage_examples,
            related_symbols,
        })
    }

    /// Get function signatures
    async fn get_function_signatures(
        &self,
        function_context: &FunctionCallContext,
        content: &str,
        language: &str,
    ) -> Result<Vec<SignatureInformation>> {
        let mut signatures = Vec::new();

        // Parse document to find function definitions
        let parse_result = self.analysis_engine.parse_code(content, Some(language)).await?;

        // Find matching functions
        for symbol in parse_result.symbols {
            if symbol.name == function_context.function_name && matches!(symbol.kind.as_str(), \"function\" | \"method\") {
                let signature = self.create_signature_information(&symbol).await?;
                signatures.push(signature);
            }
        }

        // If no signatures found, try AI-powered lookup
        if signatures.is_empty() && self.config.ai_generation {
            if let Ok(ai_signature) = self.get_ai_function_signature(&function_context.function_name, language).await {
                signatures.push(ai_signature);
            }
        }

        Ok(signatures)
    }

    /// Create signature information
    async fn create_signature_information(&self, symbol: &crate::analysis::Symbol) -> Result<SignatureInformation> {
        let mut label = format!(\"{}(\", symbol.name);
        let mut parameters = Vec::new();

        // Add parameters if available
        if let Some(params) = &symbol.parameters {
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    label.push_str(\", \");
                }
                label.push_str(&param.name);
                if let Some(type_name) = &param.type_name {
                    label.push_str(&format!(\": {}\", type_name));
                }

                parameters.push(ParameterInformation {
                    label: param.name.clone(),
                    documentation: param.description.as_ref().map(|desc| MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: desc.clone(),
                    }),
                });
            }
        }

        label.push(')');

        Ok(SignatureInformation {
            label,
            documentation: symbol.documentation.as_ref().map(|doc| MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc.clone(),
            }),
            parameters,
            active_parameter: None,
        })
    }

    /// Parse function call from text
    fn parse_function_call(&self, text: &str) -> Option<FunctionCallContext> {
        // Simple regex to find function calls
        if let Ok(re) = regex::Regex::new(r\"(\\w+)\\s*\\(\") {
            if let Some(captures) = re.captures(text) {
                if let Some(function_name) = captures.get(1) {
                    let args_start = text.rfind('(').unwrap_or(0);
                    let args_text = &text[args_start + 1..];
                    let current_arg = args_text.split(',').count();

                    return Some(FunctionCallContext {
                        function_name: function_name.as_str().to_string(),
                        current_argument: current_arg.saturating_sub(1),
                        arguments_so_far: args_text.split(',').map(|s| s.trim().to_string()).collect(),
                    });
                }
            }
        }
        None
    }

    /// Generate examples for different symbol types
    async fn generate_function_example(&self, symbol: &SymbolInfo, language: &str) -> Result<ExampleDoc> {
        let mut example_code = format!(\"{}(\", symbol.name);
        
        for (i, param) in symbol.parameters.iter().enumerate() {
            if i > 0 {
                example_code.push_str(\", \");
            }
            example_code.push_str(&self.generate_example_value(&param.type_name.clone().unwrap_or(\"unknown\".to_string())));
        }
        
        example_code.push(')');

        Ok(ExampleDoc {
            title: format!(\"Using {}\", symbol.name),
            code: example_code,
            description: format!(\"Example usage of the {} function\", symbol.name),
            language: language.to_string(),
        })
    }

    async fn generate_class_example(&self, symbol: &SymbolInfo, language: &str) -> Result<ExampleDoc> {
        let example_code = match language {
            \"java\" | \"typescript\" | \"javascript\" => format!(\"const instance = new {}();\", symbol.name),
            \"python\" => format!(\"instance = {}()\", symbol.name),
            \"rust\" => format!(\"let instance = {}::new();\", symbol.name),
            _ => format!(\"// Create instance of {}\", symbol.name),
        };

        Ok(ExampleDoc {
            title: format!(\"Creating {}\", symbol.name),
            code: example_code,
            description: format!(\"Example instantiation of the {} class\", symbol.name),
            language: language.to_string(),
        })
    }

    async fn generate_variable_example(&self, symbol: &SymbolInfo, language: &str) -> Result<ExampleDoc> {
        let example_code = match language {
            \"javascript\" | \"typescript\" => format!(\"console.log({});\", symbol.name),
            \"python\" => format!(\"print({})\", symbol.name),
            \"java\" => format!(\"System.out.println({});\", symbol.name),
            \"rust\" => format!(\"println!(\\\"{{}}\\\", {});\", symbol.name),
            _ => format!(\"// Use {}\", symbol.name),
        };

        Ok(ExampleDoc {
            title: format!(\"Using {}\", symbol.name),
            code: example_code,
            description: format!(\"Example usage of the {} variable\", symbol.name),
            language: language.to_string(),
        })
    }

    /// Generate example value for type
    fn generate_example_value(&self, type_name: &str) -> String {
        match type_name.to_lowercase().as_str() {
            \"string\" | \"str\" => \"\\\"example\\\"\".to_string(),
            \"number\" | \"int\" | \"integer\" | \"i32\" | \"i64\" => \"42\".to_string(),
            \"float\" | \"double\" | \"f32\" | \"f64\" => \"3.14\".to_string(),
            \"boolean\" | \"bool\" => \"true\".to_string(),
            \"array\" | \"list\" | \"vec\" => \"[]\".to_string(),
            \"object\" | \"map\" | \"dict\" => \"{}\".to_string(),
            _ => \"null\".to_string(),
        }
    }

    /// Helper methods for documentation processing
    fn is_position_in_symbol(&self, symbol: &crate::analysis::Symbol, position: &Position) -> bool {
        let symbol_line = (symbol.line - 1) as u32;
        let symbol_start_char = (symbol.column - 1) as u32;
        let symbol_end_char = symbol_start_char + symbol.name.len() as u32;

        position.line == symbol_line 
            && position.character >= symbol_start_char 
            && position.character <= symbol_end_char
    }

    fn extract_word_at_position(&self, content: &str, position: &Position) -> Result<Option<String>> {
        let lines: Vec<&str> = content.lines().collect();
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;

        if line_idx >= lines.len() {
            return Ok(None);
        }

        let line = lines[line_idx];
        if char_idx >= line.len() {
            return Ok(None);
        }

        // Find word boundaries
        let start = line[..char_idx].rfind(|c: char| !c.is_alphanumeric() && c != '_').map(|i| i + 1).unwrap_or(0);
        let end = line[char_idx..].find(|c: char| !c.is_alphanumeric() && c != '_').map(|i| char_idx + i).unwrap_or(line.len());

        if start < end {
            Ok(Some(line[start..end].to_string()))
        } else {
            Ok(None)
        }
    }

    /// Cache management
    async fn get_cached_documentation(&self, symbol_name: &str, language: &str) -> Option<CachedDocumentation> {
        let cache = self.documentation_cache.read().await;
        let cache_key = format!(\"{}:{}\", language, symbol_name);
        cache.get(&cache_key).cloned()
    }

    async fn cache_documentation(&self, symbol_name: &str, documentation: &DocumentationContent, language: &str) {
        let mut cache = self.documentation_cache.write().await;
        let cache_key = format!(\"{}:{}\", language, symbol_name);
        cache.insert(cache_key, CachedDocumentation {
            content: documentation.clone(),
            timestamp: std::time::SystemTime::now(),
            language: language.to_string(),
        });
    }

    fn is_cache_expired(&self, cached: &CachedDocumentation) -> bool {
        if let Ok(elapsed) = cached.timestamp.elapsed() {
            elapsed.as_secs() > self.config.cache_timeout_seconds
        } else {
            true
        }
    }

    /// Placeholder implementations for additional features
    async fn get_context_information(&self, symbol: &SymbolInfo, context: &str, language: &str) -> Result<Vec<String>> {
        // TODO: Implement context information extraction
        Ok(vec![format!(\"Defined in {}\", language)])
    }

    async fn find_related_symbols(&self, symbol: &SymbolInfo, context: &str, language: &str) -> Result<Vec<String>> {
        // TODO: Implement related symbol detection
        Ok(Vec::new())
    }

    async fn get_ai_function_signature(&self, function_name: &str, language: &str) -> Result<SignatureInformation> {
        // TODO: Implement AI-powered signature lookup
        Ok(SignatureInformation {
            label: format!(\"{}()\", function_name),
            documentation: None,
            parameters: Vec::new(),
            active_parameter: None,
        })
    }

    async fn determine_active_signature_and_parameter(
        &self,
        function_context: &FunctionCallContext,
        signatures: &[SignatureInformation],
    ) -> (Option<u32>, Option<u32>) {
        // Simple implementation - use first signature and current argument
        let active_signature = if !signatures.is_empty() { Some(0) } else { None };
        let active_parameter = Some(function_context.current_argument as u32);
        (active_signature, active_parameter)
    }

    fn create_hover_from_documentation(&self, documentation: &DocumentationContent, symbol: &SymbolInfo) -> Result<EnhancedHover> {
        let content = format!(\"**{}**\\n\\n{}\", documentation.summary, documentation.description);
        
        Ok(EnhancedHover {
            base: Hover {
                contents: MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                },
                range: Some(Range {
                    start: Position { line: (symbol.line - 1) as u32, character: (symbol.column - 1) as u32 },
                    end: Position { line: (symbol.line - 1) as u32, character: (symbol.column + symbol.name.len() - 1) as u32 },
                }),
            },
            ai_insights: None,
            context_information: Vec::new(),
            usage_examples: Vec::new(),
            related_symbols: Vec::new(),
        })
    }

    fn parse_ai_documentation(&self, content: &str) -> Result<DocumentationContent> {
        // TODO: Implement sophisticated AI documentation parsing
        Ok(DocumentationContent {
            summary: \"AI-generated documentation\".to_string(),
            description: content.clone(),
            parameters: Vec::new(),
            return_type: None,
            examples: Vec::new(),
            see_also: Vec::new(),
            notes: Vec::new(),
            warnings: Vec::new(),
            complexity: None,
            ai_generated: true,
        })
    }

    fn merge_ai_documentation(&self, documentation: &mut DocumentationContent, ai_doc: DocumentationContent) {
        if documentation.description.is_empty() {
            documentation.description = ai_doc.description;
        } else {
            documentation.description.push_str(&format!(\"\\n\\n{}\", ai_doc.description));
        }
    }
}

/// Symbol information for documentation
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub line: usize,
    pub column: usize,
    pub scope: Option<String>,
    pub type_info: Option<String>,
    pub documentation: Option<String>,
    pub signature: Option<String>,
    pub parameters: Vec<ParameterInfo>,
}

/// Parameter information
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub type_name: Option<String>,
    pub description: Option<String>,
    pub optional: Option<bool>,
    pub default_value: Option<String>,
}

/// Function call context
#[derive(Debug, Clone)]
pub struct FunctionCallContext {
    pub function_name: String,
    pub current_argument: usize,
    pub arguments_so_far: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_extraction() {
        let provider = ContextualDocumentationProvider {
            consensus_engine: Arc::new(RwLock::new(ConsensusEngine::default())),
            analysis_engine: Arc::new(AnalysisEngine::default()),
            config: DocumentationConfig::default(),
            documentation_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        let content = \"function test() { return; }\";
        let position = Position { line: 0, character: 5 };
        
        let word = provider.extract_word_at_position(content, &position).unwrap();
        assert_eq!(word, Some(\"function\".to_string()));
    }

    #[test]
    fn test_function_call_parsing() {
        let provider = ContextualDocumentationProvider {
            consensus_engine: Arc::new(RwLock::new(ConsensusEngine::default())),
            analysis_engine: Arc::new(AnalysisEngine::default()),
            config: DocumentationConfig::default(),
            documentation_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        let text = \"myFunction(arg1, arg2,\";
        let context = provider.parse_function_call(text);
        
        assert!(context.is_some());
        let context = context.unwrap();
        assert_eq!(context.function_name, \"myFunction\");
        assert_eq!(context.current_argument, 2);
    }

    #[test]
    fn test_example_value_generation() {
        let provider = ContextualDocumentationProvider {
            consensus_engine: Arc::new(RwLock::new(ConsensusEngine::default())),
            analysis_engine: Arc::new(AnalysisEngine::default()),
            config: DocumentationConfig::default(),
            documentation_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        assert_eq!(provider.generate_example_value(\"string\"), \"\\\"example\\\"\");
        assert_eq!(provider.generate_example_value(\"number\"), \"42\");
        assert_eq!(provider.generate_example_value(\"boolean\"), \"true\");
    }
}"