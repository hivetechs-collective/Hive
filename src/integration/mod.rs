//! IDE and editor integration servers
//!
//! This module provides MCP and LSP servers for IDE integration

pub mod mcp;
// pub mod lsp; // Temporarily disabled due to encoding issues

use anyhow::Result;

/// Start MCP server
pub async fn start_mcp_server(port: u16) -> Result<()> {
    mcp::start_mcp_server(port).await
}

/// Start LSP server
pub async fn start_lsp_server(port: u16) -> Result<()> {
    // lsp::start_lsp_server(port).await // Temporarily disabled
    Ok(())
}
