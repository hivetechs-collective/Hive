//! Model Context Protocol (MCP) server implementation
//!
//! This module provides a complete MCP server for IDE integration

pub mod protocol;
pub mod server;
pub mod tools;
pub mod tools_advanced;
pub mod prompts;
pub mod sampling;
pub mod subscriptions;
pub mod performance;
pub mod resources;
pub mod auth;
pub mod streaming;

pub use server::McpServer;
pub use protocol::{McpRequest, McpResponse, McpError};
pub use streaming::{StreamingHandler, StreamingToolResponse};
pub use tools_advanced::AdvancedToolRegistry;
pub use prompts::{PromptManager, PromptContext};
pub use sampling::{SamplingManager, SampleRequest, SampleProgress};
pub use subscriptions::{SubscriptionManager, SubscriptionRequest};
pub use performance::{PerformanceManager, PerformanceConfig};

use anyhow::Result;

/// Start MCP server
pub async fn start_mcp_server(port: u16) -> Result<()> {
    let server = McpServer::new().await?;
    server.start(port).await
}