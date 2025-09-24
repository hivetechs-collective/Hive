//! AI CLI Controller - Manages AI CLI tool interactions
//!
//! This module provides a controller for AI CLI tools, handling their
//! installation, status updates, and terminal launching.

use crate::desktop::ai_cli_updater::AiCliUpdaterDB;
use crate::desktop::terminal_registry::send_to_terminal;
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Events that can be sent from the controller to the UI
#[derive(Debug, Clone)]
pub enum AiCliEvent {
    ToolStatusChanged {
        tool_id: String,
        status: ToolStatus,
    },
    InstallationProgress {
        tool_id: String,
        message: String,
    },
    InstallationComplete {
        tool_id: String,
        success: bool,
        error: Option<String>,
    },
}

/// Tool status (mirror of the one in terminal_tabs.rs)
#[derive(Debug, Clone, PartialEq)]
pub enum ToolStatus {
    Available,
    Installing,
    Ready,
    Starting,
    Running,
    Error(String),
}

/// Controller for AI CLI tools
pub struct AiCliController {
    updater: Arc<AiCliUpdaterDB>,
    event_tx: mpsc::UnboundedSender<AiCliEvent>,
}

impl AiCliController {
    /// Create a new AI CLI controller
    pub async fn new(event_tx: mpsc::UnboundedSender<AiCliEvent>) -> Result<Self> {
        let updater = Arc::new(AiCliUpdaterDB::new().await?);

        Ok(Self { updater, event_tx })
    }

    /// Check and update tool status
    pub async fn check_tool_status(&self, tool_id: &str) -> Result<ToolStatus> {
        match self.updater.is_tool_installed(tool_id).await {
            Ok(true) => Ok(ToolStatus::Ready),
            Ok(false) => Ok(ToolStatus::Available),
            Err(e) => Ok(ToolStatus::Error(format!("Check failed: {}", e))),
        }
    }

    /// Install a tool asynchronously
    pub async fn install_tool(&self, tool_id: String) -> Result<()> {
        info!("🚀 Starting installation for tool: {}", tool_id);

        // Send status update
        let _ = self.event_tx.send(AiCliEvent::ToolStatusChanged {
            tool_id: tool_id.clone(),
            status: ToolStatus::Installing,
        });

        // Send progress update
        let _ = self.event_tx.send(AiCliEvent::InstallationProgress {
            tool_id: tool_id.clone(),
            message: "Checking dependencies...".to_string(),
        });

        // Perform the installation
        match self.updater.install_latest(&tool_id).await {
            Ok(()) => {
                info!("✅ Tool {} installed successfully", tool_id);

                let _ = self.event_tx.send(AiCliEvent::ToolStatusChanged {
                    tool_id: tool_id.clone(),
                    status: ToolStatus::Ready,
                });

                let _ = self.event_tx.send(AiCliEvent::InstallationComplete {
                    tool_id: tool_id.clone(),
                    success: true,
                    error: None,
                });
            }
            Err(e) => {
                error!("❌ Failed to install tool {}: {}", tool_id, e);

                let error_msg = format!("Installation failed: {}", e);
                let _ = self.event_tx.send(AiCliEvent::ToolStatusChanged {
                    tool_id: tool_id.clone(),
                    status: ToolStatus::Error(error_msg.clone()),
                });

                let _ = self.event_tx.send(AiCliEvent::InstallationComplete {
                    tool_id: tool_id.clone(),
                    success: false,
                    error: Some(error_msg),
                });
            }
        }

        Ok(())
    }

    /// Get the path to a tool executable
    pub async fn get_tool_path(&self, tool_id: &str) -> Result<std::path::PathBuf> {
        self.updater.get_tool_path(tool_id).await
    }

    /// Check all tools and update their status
    pub async fn refresh_all_tools(&self, tools: &[String]) -> Result<()> {
        for tool_id in tools {
            let status = self.check_tool_status(tool_id).await?;
            let _ = self.event_tx.send(AiCliEvent::ToolStatusChanged {
                tool_id: tool_id.clone(),
                status,
            });
        }
        Ok(())
    }
}

/// Background service that manages AI CLI tools
pub struct AiCliService {
    controller: Arc<AiCliController>,
    shutdown_rx: mpsc::Receiver<()>,
}

impl AiCliService {
    /// Create and start the AI CLI service
    pub async fn start(event_tx: mpsc::UnboundedSender<AiCliEvent>) -> Result<mpsc::Sender<()>> {
        let controller = Arc::new(AiCliController::new(event_tx).await?);
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        let service = Self {
            controller,
            shutdown_rx,
        };

        // Spawn the service
        tokio::spawn(async move {
            service.run().await;
        });

        Ok(shutdown_tx)
    }

    /// Run the service
    async fn run(mut self) {
        info!("🤖 AI CLI Service started");

        // Check for updates periodically
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    debug!("⏰ Periodic AI CLI tool check");
                    // In the future, we could check for tool updates here
                }
                _ = self.shutdown_rx.recv() => {
                    info!("🛑 AI CLI Service shutting down");
                    break;
                }
            }
        }
    }
}
