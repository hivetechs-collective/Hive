//! MCP authentication and security
//!
//! Provides secure access control for MCP server

use crate::core::config::Config;
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{info, warn};

/// Authentication manager for MCP server
pub struct AuthManager {
    config: Arc<Config>,
    trusted_clients: HashSet<String>,
}

impl AuthManager {
    /// Create new auth manager
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let mut trusted_clients = HashSet::new();

        // Add default trusted clients
        trusted_clients.insert("vscode".to_string());
        trusted_clients.insert("claude-desktop".to_string());
        trusted_clients.insert("cursor".to_string());
        trusted_clients.insert("zed".to_string());
        trusted_clients.insert("nvim".to_string());
        trusted_clients.insert("emacs".to_string());
        trusted_clients.insert("intellij".to_string());
        trusted_clients.insert("webstorm".to_string());
        trusted_clients.insert("pycharm".to_string());
        trusted_clients.insert("sublime".to_string());

        Ok(Self {
            config,
            trusted_clients,
        })
    }

    /// Validate client authentication
    pub async fn validate_client(&self, client_name: &str) -> Result<bool> {
        info!("Validating client: {}", client_name);

        // Check if client is in trusted list
        let client_lower = client_name.to_lowercase();
        for trusted in &self.trusted_clients {
            if client_lower.contains(trusted) {
                info!("Client {} is trusted", client_name);
                return Ok(true);
            }
        }

        warn!("Unknown client attempted connection: {}", client_name);

        // For now, allow all clients but log the warning
        // In production, this could be more restrictive
        Ok(true)
    }

    /// Add trusted client
    pub async fn add_trusted_client(&mut self, client_name: String) {
        info!("Adding trusted client: {}", client_name);
        self.trusted_clients.insert(client_name);
    }

    /// Remove trusted client
    pub async fn remove_trusted_client(&mut self, client_name: &str) {
        info!("Removing trusted client: {}", client_name);
        self.trusted_clients.remove(client_name);
    }

    /// List trusted clients
    pub async fn list_trusted_clients(&self) -> Vec<String> {
        self.trusted_clients.iter().cloned().collect()
    }
}
