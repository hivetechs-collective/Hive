//! Model Context Protocol (MCP) server implementation
//!
//! This module provides a complete MCP server for IDE integration

pub mod auth;
pub mod performance;
pub mod prompts;
pub mod protocol;
pub mod resources;
pub mod sampling;
pub mod server;
pub mod streaming;
pub mod subscriptions;
pub mod tools;
pub mod tools_advanced;

pub use performance::{PerformanceConfig, PerformanceManager};
pub use prompts::{PromptContext, PromptManager};
pub use protocol::{McpError, McpRequest, McpResponse};
pub use sampling::{SampleProgress, SampleRequest, SamplingManager};
pub use server::McpServer;
pub use streaming::{StreamingHandler, StreamingToolResponse};
pub use subscriptions::{SubscriptionManager, SubscriptionRequest};
pub use tools_advanced::AdvancedToolRegistry;

use anyhow::Result;

/// Start MCP server
pub async fn start_mcp_server(port: u16) -> Result<()> {
    let server = McpServer::new().await?;
    server.start(port).await
}
