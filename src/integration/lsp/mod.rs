//! Language Server Protocol (LSP) implementation
//!
//! Provides AI-powered IDE features via LSP with full LSP 3.17 protocol support

pub mod protocol;
pub mod server;
pub mod features;
pub mod handlers;
// pub mod completion; // Temporarily disabled due to syntax errors
pub mod diagnostics;
pub mod refactoring;
pub mod documentation;

pub use server::LspServer;
pub use handlers::*;
// pub use completion::*; // Temporarily disabled
pub use diagnostics::*;
pub use refactoring::*;
pub use documentation::*;

use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{info, error};

/// Start LSP server on specified port
pub async fn start_lsp_server(port: u16) -> Result<()> {
    let server = LspServer::new().await?;
    server.start_tcp(port).await
}

/// Start LSP server using stdio
pub async fn start_lsp_stdio() -> Result<()> {
    let server = LspServer::new().await?;
    server.start_stdio().await
}

/// LSP server configuration
#[derive(Debug, Clone)]
pub struct LspConfig {
    /// Enable AI-powered completion (default: true)
    pub ai_completion: bool,
    /// Enable real-time diagnostics (default: true)
    pub real_time_diagnostics: bool,
    /// Enable smart refactoring (default: true)
    pub smart_refactoring: bool,
    /// Enable contextual documentation (default: true)
    pub contextual_docs: bool,
    /// Maximum completion items to return (default: 100)
    pub max_completion_items: usize,
    /// Diagnostics update interval in ms (default: 1000)
    pub diagnostics_interval: u64,
    /// Enable performance logging (default: false)
    pub performance_logging: bool,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            ai_completion: true,
            real_time_diagnostics: true,
            smart_refactoring: true,
            contextual_docs: true,
            max_completion_items: 100,
            diagnostics_interval: 1000,
            performance_logging: false,
        }
    }
}

/// LSP capabilities supported by Hive AI
#[derive(Debug, Clone)]
pub struct HiveLspCapabilities {
    /// Text document synchronization
    pub text_document_sync: bool,
    /// Code completion with AI
    pub completion: bool,
    /// Hover information
    pub hover: bool,
    /// Signature help
    pub signature_help: bool,
    /// Go to definition
    pub definition: bool,
    /// Find references
    pub references: bool,
    /// Document symbols
    pub document_symbols: bool,
    /// Code actions
    pub code_actions: bool,
    /// Rename symbols
    pub rename: bool,
    /// Diagnostics
    pub diagnostics: bool,
    /// Document formatting (via AI)
    pub formatting: bool,
    /// Range formatting (via AI)
    pub range_formatting: bool,
    /// Semantic tokens
    pub semantic_tokens: bool,
    /// Inlay hints
    pub inlay_hints: bool,
}

impl Default for HiveLspCapabilities {
    fn default() -> Self {
        Self {
            text_document_sync: true,
            completion: true,
            hover: true,
            signature_help: true,
            definition: true,
            references: true,
            document_symbols: true,
            code_actions: true,
            rename: true,
            diagnostics: true,
            formatting: true,
            range_formatting: true,
            semantic_tokens: true,
            inlay_hints: true,
        }
    }
}