//! LSP protocol types and message definitions
//!
//! Implementation of the Language Server Protocol specification

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// LSP message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspMessage {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(flatten)]
    pub content: LspMessageContent,
}

/// LSP message content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LspMessageContent {
    Request(LspRequest),
    Response(LspResponse),
    Notification(LspNotification),
}

/// LSP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRequest {
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// LSP response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LspResponse {
    Success { result: Value },
    Error { error: LspError },
}

/// LSP notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspNotification {
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// LSP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Position in document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Range in document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Text document identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

/// Versioned text document identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

/// Text document position params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentPositionParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

/// Initialize params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "processId")]
    pub process_id: Option<u32>,
    #[serde(rename = "clientInfo")]
    pub client_info: Option<ClientInfo>,
    #[serde(rename = "rootPath")]
    pub root_path: Option<String>,
    #[serde(rename = "rootUri")]
    pub root_uri: Option<String>,
    #[serde(rename = "initializationOptions")]
    pub initialization_options: Option<Value>,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "workspaceFolders")]
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,
}

/// Client info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: Option<String>,
}

/// Workspace folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(rename = "textDocument")]
    pub text_document: Option<TextDocumentClientCapabilities>,
    pub workspace: Option<WorkspaceClientCapabilities>,
    #[serde(rename = "experimental")]
    pub experimental: Option<Value>,
}

/// Text document client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentClientCapabilities {
    pub completion: Option<CompletionClientCapabilities>,
    pub hover: Option<HoverClientCapabilities>,
    #[serde(rename = "codeAction")]
    pub code_action: Option<CodeActionClientCapabilities>,
    #[serde(rename = "publishDiagnostics")]
    pub publish_diagnostics: Option<PublishDiagnosticsClientCapabilities>,
}

/// Workspace client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceClientCapabilities {
    #[serde(rename = "workspaceFolders")]
    pub workspace_folders: Option<bool>,
    pub configuration: Option<bool>,
}

/// Completion client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionClientCapabilities {
    #[serde(rename = "dynamicRegistration")]
    pub dynamic_registration: Option<bool>,
    #[serde(rename = "completionItem")]
    pub completion_item: Option<CompletionItemCapabilities>,
}

/// Completion item capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItemCapabilities {
    #[serde(rename = "snippetSupport")]
    pub snippet_support: Option<bool>,
    #[serde(rename = "documentationFormat")]
    pub documentation_format: Option<Vec<MarkupKind>>,
}

/// Hover client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverClientCapabilities {
    #[serde(rename = "dynamicRegistration")]
    pub dynamic_registration: Option<bool>,
    #[serde(rename = "contentFormat")]
    pub content_format: Option<Vec<MarkupKind>>,
}

/// Code action client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeActionClientCapabilities {
    #[serde(rename = "dynamicRegistration")]
    pub dynamic_registration: Option<bool>,
    #[serde(rename = "codeActionLiteralSupport")]
    pub code_action_literal_support: Option<CodeActionLiteralSupport>,
}

/// Code action literal support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeActionLiteralSupport {
    #[serde(rename = "codeActionKind")]
    pub code_action_kind: CodeActionKindCapabilities,
}

/// Code action kind capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeActionKindCapabilities {
    #[serde(rename = "valueSet")]
    pub value_set: Vec<String>,
}

/// Publish diagnostics client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishDiagnosticsClientCapabilities {
    #[serde(rename = "relatedInformation")]
    pub related_information: Option<bool>,
    #[serde(rename = "versionSupport")]
    pub version_support: Option<bool>,
}

/// Markup kind
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkupKind {
    PlainText,
    Markdown,
}

/// Initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: Option<ServerInfo>,
}

/// Server info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(rename = "textDocumentSync")]
    pub text_document_sync: Option<TextDocumentSyncOptions>,
    #[serde(rename = "completionProvider")]
    pub completion_provider: Option<CompletionOptions>,
    #[serde(rename = "hoverProvider")]
    pub hover_provider: Option<bool>,
    #[serde(rename = "codeActionProvider")]
    pub code_action_provider: Option<bool>,
    #[serde(rename = "documentFormattingProvider")]
    pub document_formatting_provider: Option<bool>,
    #[serde(rename = "documentRangeFormattingProvider")]
    pub document_range_formatting_provider: Option<bool>,
}

/// Text document sync options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentSyncOptions {
    #[serde(rename = "openClose")]
    pub open_close: Option<bool>,
    pub change: Option<TextDocumentSyncKind>,
    #[serde(rename = "willSave")]
    pub will_save: Option<bool>,
    #[serde(rename = "willSaveWaitUntil")]
    pub will_save_wait_until: Option<bool>,
    pub save: Option<SaveOptions>,
}

/// Text document sync kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextDocumentSyncKind {
    None = 0,
    Full = 1,
    Incremental = 2,
}

/// Save options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveOptions {
    #[serde(rename = "includeText")]
    pub include_text: Option<bool>,
}

/// Completion options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionOptions {
    #[serde(rename = "resolveProvider")]
    pub resolve_provider: Option<bool>,
    #[serde(rename = "triggerCharacters")]
    pub trigger_characters: Option<Vec<String>>,
}

/// Completion list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionList {
    #[serde(rename = "isIncomplete")]
    pub is_incomplete: bool,
    pub items: Vec<CompletionItem>,
}

/// Completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<CompletionItemKind>,
    pub detail: Option<String>,
    pub documentation: Option<MarkupContent>,
    #[serde(rename = "insertText")]
    pub insert_text: Option<String>,
    #[serde(rename = "insertTextFormat")]
    pub insert_text_format: Option<InsertTextFormat>,
    #[serde(rename = "textEdit")]
    pub text_edit: Option<TextEdit>,
    #[serde(rename = "additionalTextEdits")]
    pub additional_text_edits: Option<Vec<TextEdit>>,
}

/// Completion item kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

/// Insert text format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsertTextFormat {
    PlainText = 1,
    Snippet = 2,
}

/// Markup content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkupContent {
    pub kind: MarkupKind,
    pub value: String,
}

/// Text edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: Range,
    #[serde(rename = "newText")]
    pub new_text: String,
}

/// Hover result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hover {
    pub contents: MarkupContent,
    pub range: Option<Range>,
}

/// Diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<DiagnosticSeverity>,
    pub code: Option<Value>,
    pub source: Option<String>,
    pub message: String,
    #[serde(rename = "relatedInformation")]
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,
}

/// Diagnostic severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// Diagnostic related information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
    pub location: Location,
    pub message: String,
}

/// Location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

/// Code action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<String>,
    pub diagnostics: Option<Vec<Diagnostic>>,
    pub edit: Option<WorkspaceEdit>,
    pub command: Option<Command>,
}

/// Workspace edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    pub changes: Option<HashMap<String, Vec<TextEdit>>>,
    #[serde(rename = "documentChanges")]
    pub document_changes: Option<Vec<TextDocumentEdit>>,
}

/// Text document edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentEdit {
    #[serde(rename = "textDocument")]
    pub text_document: VersionedTextDocumentIdentifier,
    pub edits: Vec<TextEdit>,
}

/// Command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub title: String,
    pub command: String,
    pub arguments: Option<Vec<Value>>,
}

impl LspError {
    /// Parse error
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    /// Invalid request
    pub fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid request".to_string(),
            data: None,
        }
    }

    /// Method not found
    pub fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }
    }

    /// Invalid parameters
    pub fn invalid_params() -> Self {
        Self {
            code: -32602,
            message: "Invalid parameters".to_string(),
            data: None,
        }
    }

    /// Internal error
    pub fn internal_error() -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: None,
        }
    }

    /// Request cancelled
    pub fn request_cancelled() -> Self {
        Self {
            code: -32800,
            message: "Request cancelled".to_string(),
            data: None,
        }
    }

    /// Content modified
    pub fn content_modified() -> Self {
        Self {
            code: -32801,
            message: "Content modified".to_string(),
            data: None,
        }
    }
}