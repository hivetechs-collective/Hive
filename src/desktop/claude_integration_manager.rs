//! Global Claude Code Integration Manager
//! 
//! Provides a singleton instance of ClaudeCodeIntegration that can be used
//! throughout the desktop application for hybrid chat functionality.

use crate::consensus::claude_code_integration::ClaudeCodeIntegration;
use crate::core::database::DatabaseManager;
use crate::memory::ThematicCluster;
use crate::consensus::engine::ConsensusEngine;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::{error, info};

/// Global instance of Claude Code integration
static CLAUDE_INTEGRATION: OnceCell<Arc<ClaudeCodeIntegration>> = OnceCell::const_new();

/// Initialize the Claude Code integration
pub async fn initialize_claude_integration(
    consensus_engine: Arc<ConsensusEngine>,
    thematic_cluster: Arc<ThematicCluster>,
    database: Arc<DatabaseManager>,
) -> Result<()> {
    info!("🚀 Initializing Claude Code integration...");
    
    match ClaudeCodeIntegration::new(consensus_engine, thematic_cluster, database).await {
        Ok(integration) => {
            CLAUDE_INTEGRATION.set(Arc::new(integration))
                .map_err(|_| anyhow::anyhow!("Claude integration already initialized"))?;
            info!("✅ Claude Code integration initialized successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to initialize Claude Code integration: {}", e);
            Err(e)
        }
    }
}

/// Get the global Claude Code integration instance
pub async fn get_claude_integration() -> Option<Arc<ClaudeCodeIntegration>> {
    CLAUDE_INTEGRATION.get().cloned()
}

/// Check if Claude Code integration is available
pub async fn is_claude_integration_available() -> bool {
    CLAUDE_INTEGRATION.get().is_some()
}

/// Shutdown the Claude Code integration
pub async fn shutdown_claude_integration() -> Result<()> {
    if let Some(integration) = CLAUDE_INTEGRATION.get() {
        info!("🛑 Shutting down Claude Code integration...");
        integration.shutdown().await?;
    }
    Ok(())
}