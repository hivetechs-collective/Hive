//! LSP request handlers
//!
//! Handles all LSP requests with AI-powered responses

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

/// Request handler for LSP methods
pub struct LspRequestHandler {
    consensus_engine: Arc<RwLock<ConsensusEngine>>,
    analysis_engine: Arc<AnalysisEngine>,
    documents: Arc<RwLock<HashMap<String, DocumentState>>>,
    performance_tracking: bool,
}

/// Document state
#[derive(Debug, Clone)]
pub struct DocumentState {
    pub uri: String,
    pub version: i32,
    pub content: String,
    pub language: String,
    pub last_modified: std::time::SystemTime,
    pub diagnostics: Vec<Diagnostic>,
}

impl LspRequestHandler {
    /// Create new request handler
    pub async fn new(
        consensus_engine: Arc<RwLock<ConsensusEngine>>,
        analysis_engine: Arc<AnalysisEngine>,
        performance_tracking: bool,
    ) -> Result<Self> {
        Ok(Self {
            consensus_engine,
            analysis_engine,
            documents: Arc::new(RwLock::new(HashMap::new())),
            performance_tracking,
        })
    }

    /// Handle initialize request
    pub async fn handle_initialize(&self, params: Value) -> LspResponse {
        let start_time = if self.performance_tracking { Some(Instant::now()) } else { None };

        let init_params: InitializeParams = match serde_json::from_value(params) {
            Ok(params) => params,
            Err(e) => {
                error!("Failed to parse initialize params: {}", e);
                return LspResponse::Error {
                    error: LspError::invalid_params(),
                };
            }
        };

        info!("Initializing LSP server for client: {:?}", init_params.client_info);

        // Build comprehensive server capabilities
        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::Full),
                will_save: Some(true),
                will_save_wait_until: Some(true),
                save: Some(SaveOptions {
                    include_text: Some(true),
                }),
            }),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(vec![
                    ".".to_string(),
                    "::".to_string(),
                    "->".to_string(),
                    "(".to_string(),
                    "[".to_string(),
                    "{".to_string(),
                    " ".to_string(),
                ]),
            }),
            hover_provider: Some(true),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                retrigger_characters: Some(vec![")".to_string()]),
            }),
            definition_provider: Some(true),
            references_provider: Some(true),
            document_highlight_provider: Some(true),
            document_symbol_provider: Some(true),
            workspace_symbol_provider: Some(true),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(true),
            }),
            document_formatting_provider: Some(true),
            document_range_formatting_provider: Some(true),
            document_on_type_formatting_provider: Some(DocumentOnTypeFormattingOptions {
                first_trigger_character: ";".to_string(),
                more_trigger_character: Some(vec!["}".to_string(), "\n".to_string()]),
            }),
            rename_provider: Some(RenameProviderCapability::Simple(true)),
            document_link_provider: Some(DocumentLinkOptions {
                resolve_provider: Some(true),
            }),
            color_provider: Some(ColorProviderCapability::Simple(true)),
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec![
                    "hive.explainCode".to_string(),
                    "hive.improveCode".to_string(),
                    "hive.generateTests".to_string(),
                    "hive.generateDocs".to_string(),
                    "hive.refactorCode".to_string(),
                    "hive.findSimilar".to_string(),
                    "hive.analyzeComplexity".to_string(),
                ],
            }),
            selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
            semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
                SemanticTokensOptions {
                    legend: SemanticTokensLegend {
                        token_types: vec![
                            "namespace".to_string(),
                            "type".to_string(),
                            "class".to_string(),
                            "enum".to_string(),
                            "interface".to_string(),
                            "struct".to_string(),
                            "typeParameter".to_string(),
                            "parameter".to_string(),
                            "variable".to_string(),
                            "property".to_string(),
                            "enumMember".to_string(),
                            "event".to_string(),
                            "function".to_string(),
                            "method".to_string(),
                            "macro".to_string(),
                            "keyword".to_string(),
                            "modifier".to_string(),
                            "comment".to_string(),
                            "string".to_string(),
                            "number".to_string(),
                            "regexp".to_string(),
                            "operator".to_string(),
                        ],
                        token_modifiers: vec![
                            "declaration".to_string(),
                            "definition".to_string(),
                            "readonly".to_string(),
                            "static".to_string(),
                            "deprecated".to_string(),
                            "abstract".to_string(),
                            "async".to_string(),
                            "modification".to_string(),
                            "documentation".to_string(),
                            "defaultLibrary".to_string(),
                        ],
                    },
                    range: Some(false),
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                },
            )),
            inline_value_provider: Some(InlineValueProviderCapability::Simple(true)),
            inlay_hint_provider: Some(InlayHintProviderCapability::Simple(true)),
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                identifier: Some("hive-ai".to_string()),
                inter_file_dependencies: true,
                workspace_diagnostics: true,
            })),
        };

        let result = InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "Hive AI LSP Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        };

        if let Some(start) = start_time {
            debug!("Initialize request processed in {:?}", start.elapsed());
        }

        LspResponse::Success {
            result: serde_json::to_value(result).unwrap(),
        }
    }

    /// Handle text document synchronization
    pub async fn handle_did_open(&self, params: Value) -> Result<()> {
        let did_open_params: DidOpenTextDocumentParams = serde_json::from_value(params)
            .map_err(|e| HiveError::validation("lsp", format!("Invalid didOpen params: {}", e)))?;

        let uri = did_open_params.text_document.uri.clone();
        let content = did_open_params.text_document.text.clone();
        let language = did_open_params.text_document.language_id.clone();

        debug!("Document opened: {} ({})", uri, language);

        // Store document state
        let mut documents = self.documents.write().await;
        documents.insert(uri.clone(), DocumentState {
            uri: uri.clone(),
            version: did_open_params.text_document.version,
            content: content.clone(),
            language,
            last_modified: std::time::SystemTime::now(),
            diagnostics: Vec::new(),
        });

        // Trigger initial analysis
        self.analyze_document(&uri, &content).await?;

        Ok(())
    }

    /// Handle document changes
    pub async fn handle_did_change(&self, params: Value) -> Result<()> {
        let did_change_params: DidChangeTextDocumentParams = serde_json::from_value(params)
            .map_err(|e| HiveError::validation("lsp", format!("Invalid didChange params: {}", e)))?;

        let uri = did_change_params.text_document.uri.clone();
        let version = did_change_params.text_document.version;

        debug!("Document changed: {} (version {})", uri, version);

        // Update document state
        let mut documents = self.documents.write().await;
        if let Some(doc_state) = documents.get_mut(&uri) {
            doc_state.version = version;
            doc_state.last_modified = std::time::SystemTime::now();

            // Apply changes
            for change in did_change_params.content_changes {
                if let Some(range) = change.range {
                    // Incremental change
                    self.apply_text_change(&mut doc_state.content, range, &change.text)?;
                } else {
                    // Full document change
                    doc_state.content = change.text;
                }
            }

            // Trigger analysis
            let content = doc_state.content.clone();
            drop(documents); // Release lock before async call
            self.analyze_document(&uri, &content).await?;
        }

        Ok(())
    }

    /// Handle document save
    pub async fn handle_did_save(&self, params: Value) -> Result<()> {
        let did_save_params: DidSaveTextDocumentParams = serde_json::from_value(params)
            .map_err(|e| HiveError::validation("lsp", format!("Invalid didSave params: {}", e)))?;

        let uri = did_save_params.text_document.uri.clone();
        debug!("Document saved: {}", uri);

        // Update document state and trigger comprehensive analysis
        let documents = self.documents.read().await;
        if let Some(doc_state) = documents.get(&uri) {
            let content = doc_state.content.clone();
            drop(documents);

            // Run comprehensive analysis on save
            self.comprehensive_analysis(&uri, &content).await?;
        }

        Ok(())
    }

    /// Handle document close
    pub async fn handle_did_close(&self, params: Value) -> Result<()> {
        let did_close_params: DidCloseTextDocumentParams = serde_json::from_value(params)
            .map_err(|e| HiveError::validation("lsp", format!("Invalid didClose params: {}", e)))?;

        let uri = did_close_params.text_document.uri.clone();
        debug!("Document closed: {}", uri);

        // Remove document state
        let mut documents = self.documents.write().await;
        documents.remove(&uri);

        Ok(())
    }

    /// Apply text change to document content
    fn apply_text_change(&self, content: &mut String, range: Range, text: &str) -> Result<()> {
        let lines: Vec<&str> = content.lines().collect();

        if range.start.line as usize >= lines.len() || range.end.line as usize >= lines.len() {
            return Err(HiveError::validation("lsp", "Range out of bounds"));
        }

        // Calculate byte positions
        let start_line = &lines[range.start.line as usize];
        let end_line = &lines[range.end.line as usize];

        let start_byte = lines[..range.start.line as usize].iter()
            .map(|line| line.len() + 1) // +1 for newline
            .sum::<usize>() + range.start.character as usize;

        let end_byte = lines[..range.end.line as usize].iter()
            .map(|line| line.len() + 1)
            .sum::<usize>() + range.end.character as usize;

        // Perform replacement
        content.replace_range(start_byte..end_byte, text);

        Ok(())
    }

    /// Analyze document for basic diagnostics
    async fn analyze_document(&self, uri: &str, content: &str) -> Result<()> {
        debug!("Analyzing document: {}", uri);

        // Use analysis engine for syntax analysis
        let parse_result = self.analysis_engine.parse_code(content, None).await?;

        // Convert parse errors to diagnostics
        let mut diagnostics = Vec::new();
        for error in parse_result.errors {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: error.line.saturating_sub(1) as u32,
                        character: error.column.saturating_sub(1) as u32,
                    },
                    end: Position {
                        line: error.line.saturating_sub(1) as u32,
                        character: (error.column + error.length.unwrap_or(1)).saturating_sub(1) as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::Error),
                code: Some(Value::String(error.code.unwrap_or_default())),
                source: Some("hive-ai-syntax".to_string()),
                message: error.message,
                related_information: None,
            });
        }

        // Store diagnostics
        let mut documents = self.documents.write().await;
        if let Some(doc_state) = documents.get_mut(uri) {
            doc_state.diagnostics = diagnostics;
        }

        Ok(())
    }

    /// Comprehensive analysis including AI-powered insights
    async fn comprehensive_analysis(&self, uri: &str, content: &str) -> Result<()> {
        debug!("Running comprehensive analysis: {}", uri);

        // Basic syntax analysis
        self.analyze_document(uri, content).await?;

        // AI-powered analysis via consensus engine
        let consensus = self.consensus_engine.read().await;
        let analysis_prompt = format!(
            "Analyze this code for potential issues, improvements, and patterns:\n\n```\n{}\n```\n\nProvide insights about:\n1. Code quality\n2. Potential bugs\n3. Performance issues\n4. Best practices\n5. Security concerns",
            content
        );

        match consensus.ask(&analysis_prompt).await {
            Ok(response) => {
                info!("AI analysis completed for {}: {}", uri, response.summary);
                // TODO: Convert AI insights to additional diagnostics
            }
            Err(e) => {
                warn!("AI analysis failed for {}: {}", uri, e);
            }
        }

        Ok(())
    }

    /// Get document state
    pub async fn get_document(&self, uri: &str) -> Option<DocumentState> {
        let documents = self.documents.read().await;
        documents.get(uri).cloned()
    }

    /// Get all documents
    pub async fn get_all_documents(&self) -> HashMap<String, DocumentState> {
        let documents = self.documents.read().await;
        documents.clone()
    }
}

// Extended protocol types for additional LSP 3.17 features

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureHelpOptions {
    #[serde(rename = "triggerCharacters")]
    pub trigger_characters: Option<Vec<String>>,
    #[serde(rename = "retriggerCharacters")]
    pub retrigger_characters: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CodeActionProviderCapability {
    Simple(bool),
    Options(CodeActionOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeActionOptions {
    #[serde(rename = "codeActionKinds")]
    pub code_action_kinds: Option<Vec<String>>,
    #[serde(rename = "resolveProvider")]
    pub resolve_provider: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLensOptions {
    #[serde(rename = "resolveProvider")]
    pub resolve_provider: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOnTypeFormattingOptions {
    #[serde(rename = "firstTriggerCharacter")]
    pub first_trigger_character: String,
    #[serde(rename = "moreTriggerCharacter")]
    pub more_trigger_character: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RenameProviderCapability {
    Simple(bool),
    Options(RenameOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameOptions {
    #[serde(rename = "prepareProvider")]
    pub prepare_provider: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLinkOptions {
    #[serde(rename = "resolveProvider")]
    pub resolve_provider: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorProviderCapability {
    Simple(bool),
    Options(ColorOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorOptions {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FoldingRangeProviderCapability {
    Simple(bool),
    Options(FoldingRangeOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldingRangeOptions {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCommandOptions {
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SelectionRangeProviderCapability {
    Simple(bool),
    Options(SelectionRangeOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRangeOptions {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SemanticTokensServerCapabilities {
    SemanticTokensOptions(SemanticTokensOptions),
    SemanticTokensRegistrationOptions(SemanticTokensRegistrationOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTokensOptions {
    pub legend: SemanticTokensLegend,
    pub range: Option<bool>,
    pub full: Option<SemanticTokensFullOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTokensRegistrationOptions {
    pub legend: SemanticTokensLegend,
    pub range: Option<bool>,
    pub full: Option<SemanticTokensFullOptions>,
    #[serde(rename = "documentSelector")]
    pub document_selector: Option<Vec<DocumentFilter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTokensLegend {
    #[serde(rename = "tokenTypes")]
    pub token_types: Vec<String>,
    #[serde(rename = "tokenModifiers")]
    pub token_modifiers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SemanticTokensFullOptions {
    Bool(bool),
    Options(SemanticTokensFullOptionsType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTokensFullOptionsType {
    pub delta: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFilter {
    pub language: Option<String>,
    pub scheme: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InlineValueProviderCapability {
    Simple(bool),
    Options(InlineValueOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineValueOptions {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InlayHintProviderCapability {
    Simple(bool),
    Options(InlayHintOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlayHintOptions {
    #[serde(rename = "resolveProvider")]
    pub resolve_provider: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiagnosticServerCapabilities {
    Options(DiagnosticOptions),
    RegistrationOptions(DiagnosticRegistrationOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticOptions {
    pub identifier: Option<String>,
    #[serde(rename = "interFileDependencies")]
    pub inter_file_dependencies: bool,
    #[serde(rename = "workspaceDiagnostics")]
    pub workspace_diagnostics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRegistrationOptions {
    pub identifier: Option<String>,
    #[serde(rename = "interFileDependencies")]
    pub inter_file_dependencies: bool,
    #[serde(rename = "workspaceDiagnostics")]
    pub workspace_diagnostics: bool,
    #[serde(rename = "documentSelector")]
    pub document_selector: Option<Vec<DocumentFilter>>,
}

// Document synchronization params

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidOpenTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    #[serde(rename = "languageId")]
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidChangeTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: VersionedTextDocumentIdentifier,
    #[serde(rename = "contentChanges")]
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
    #[serde(rename = "rangeLength", skip_serializing_if = "Option::is_none")]
    pub range_length: Option<u32>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidSaveTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidCloseTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
}