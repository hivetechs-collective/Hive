//! AI CLI tools updater integrated with Hive's database
//!
//! This module uses the existing sync_metadata table to track update checks
//! and integrates with the unified database system, following the LazyGit pattern.

use crate::core::database::{get_database, DatabaseManager};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs as async_fs;
use tokio::process::Command;
use tracing::{error, info, warn};

// Sync type constants for different AI CLI tools
pub const SYNC_TYPE_CLAUDE_CLI: &str = "claude_cli_update";
pub const SYNC_TYPE_GH_COPILOT_CLI: &str = "gh_copilot_cli_update";
pub const SYNC_TYPE_OPENAI_CLI: &str = "openai_cli_update";

const UPDATE_CHECK_INTERVAL_HOURS: i64 = 24;

/// Tool dependency information
#[derive(Debug, Clone)]
pub struct ToolDependency {
    pub tool: String,
    pub check_command: String,
    pub install_hint: String,
}

/// Authentication status for AI tools
#[derive(Debug, Clone, PartialEq)]
pub enum AuthStatus {
    NotRequired,
    Configured,
    Required { instructions: String },
    Invalid { error: String },
}

/// Platform support
#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    MacOS,
    Linux,
    Windows,
}

/// Installation method for tools
#[derive(Debug, Clone)]
pub enum InstallMethod {
    Npm,
    Python,
    Pipx,
    Cargo,
    GhExtension,
    Binary,
}

/// Installation script details
#[derive(Debug, Clone)]
pub enum InstallScript {
    Npm(String),
    Pip(String),
    Pipx(String),
    Cargo(String),
    GhExtension(String),
    Binary {
        url: String,
        extract: bool,
        checksum: Option<String>,
        signature: Option<String>,
    },
}

/// Configuration for a CLI tool
#[derive(Debug, Clone)]
pub struct CliToolConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    pub icon: String,
    pub install_method: InstallMethod,
    pub version_check: String,
    pub install_script: InstallScript,
    pub platforms: Vec<Platform>,
    pub dependencies: Vec<ToolDependency>,
    pub auth_status: AuthStatus,
    pub sync_type: String,
}

/// AI CLI tools updater using the unified database
pub struct AiCliUpdaterDB {
    install_dir: PathBuf,
    db: Arc<DatabaseManager>,
    tool_configs: HashMap<String, CliToolConfig>,
    use_local_npm: bool,
    use_pipx: bool,
}

impl AiCliUpdaterDB {
    /// Create a new AI CLI updater
    pub async fn new() -> Result<Self> {
        let install_dir = Self::get_install_dir()?;

        // Ensure directory exists
        fs::create_dir_all(&install_dir)?;

        // Get database instance
        let db = get_database().await?;

        // Initialize tool configurations
        let mut tool_configs = HashMap::new();
        for tool in Self::get_ai_tools() {
            tool_configs.insert(tool.id.clone(), tool);
        }

        Ok(Self {
            install_dir,
            db,
            tool_configs,
            use_local_npm: false, // TODO: Load from settings
            use_pipx: true,       // TODO: Load from settings
        })
    }

    /// Get the list of available AI CLI tools
    pub fn get_ai_tools() -> Vec<CliToolConfig> {
        // Use the registry to get enabled tools
        crate::desktop::ai_cli_registry::get_enabled_ai_tools()
    }

    /// Get the path to a specific AI CLI tool
    pub async fn get_tool_path(&self, tool_id: &str) -> Result<PathBuf> {
        let tool = self
            .tool_configs
            .get(tool_id)
            .context(format!("Unknown tool: {}", tool_id))?;

        // First check if we need to update
        if self.should_check_for_update(tool_id).await? {
            info!("🔄 Checking for {} updates...", tool.name);
            if let Err(e) = self.update_if_needed(tool_id).await {
                warn!("Failed to update {}: {}", tool.name, e);
            }
        }

        // Check our managed installation first
        let managed_path = self.get_managed_tool_path(tool_id)?;
        if managed_path.exists() {
            return Ok(managed_path);
        }

        // Check system installation
        if let Ok(system_path) = which::which(&tool.command) {
            info!("Using system {} at: {:?}", tool.name, system_path);
            return Ok(system_path);
        }

        // No tool found, install it
        info!("🚀 {} not found, installing...", tool.name);
        self.install_latest(tool_id).await?;

        Ok(self.get_managed_tool_path(tool_id)?)
    }

    /// Ensure a tool is ready (installed and available)
    pub async fn ensure_tool_ready(&self, tool_id: &str) -> Result<()> {
        let _ = self.get_tool_path(tool_id).await?;
        Ok(())
    }

    /// Check if we should check for updates
    pub async fn should_check_for_update(&self, tool_id: &str) -> Result<bool> {
        let tool = self
            .tool_configs
            .get(tool_id)
            .context(format!("Unknown tool: {}", tool_id))?;

        let conn = self.db.get_connection()?;

        // Check for the most recent update check for this tool
        let last_check: Option<String> = conn
            .query_row(
                "SELECT completed_at FROM sync_metadata 
                 WHERE sync_type = ?1 AND status = 'completed'
                 ORDER BY completed_at DESC LIMIT 1",
                params![&tool.sync_type],
                |row| row.get(0),
            )
            .optional()?;

        match last_check {
            Some(timestamp) => {
                let last_check_time = DateTime::parse_from_rfc3339(&timestamp)?.with_timezone(&Utc);
                let now = Utc::now();
                let elapsed = now - last_check_time;

                Ok(elapsed > Duration::hours(UPDATE_CHECK_INTERVAL_HOURS))
            }
            None => Ok(true), // Never checked before
        }
    }

    /// Update a tool if needed
    pub async fn update_if_needed(&self, tool_id: &str) -> Result<()> {
        let sync_id = self.start_sync_record(tool_id).await?;

        match self.perform_update(tool_id).await {
            Ok(version) => {
                self.complete_sync_record(&sync_id, &version).await?;
                Ok(())
            }
            Err(e) => {
                self.fail_sync_record(&sync_id, &e.to_string()).await?;
                Err(e)
            }
        }
    }

    /// Install the latest version of a tool
    pub async fn install_latest(&self, tool_id: &str) -> Result<()> {
        let sync_id = self.start_sync_record(tool_id).await?;

        match self.perform_install(tool_id).await {
            Ok(version) => {
                self.complete_sync_record(&sync_id, &version).await?;
                Ok(())
            }
            Err(e) => {
                self.fail_sync_record(&sync_id, &e.to_string()).await?;
                Err(e)
            }
        }
    }

    /// Perform the actual update check and installation
    async fn perform_update(&self, tool_id: &str) -> Result<String> {
        let tool = self
            .tool_configs
            .get(tool_id)
            .context(format!("Unknown tool: {}", tool_id))?;

        // For now, we'll just check if the tool is installed
        // TODO: Implement version checking for each tool type
        let installed = self.is_tool_installed(tool_id).await?;

        if !installed {
            info!("🆕 {} not installed, installing...", tool.name);
            self.install_tool(tool).await?;
        } else {
            info!("✅ {} is installed", tool.name);
        }

        Ok("latest".to_string()) // TODO: Get actual version
    }

    /// Perform the actual installation
    async fn perform_install(&self, tool_id: &str) -> Result<String> {
        let tool = self
            .tool_configs
            .get(tool_id)
            .context(format!("Unknown tool: {}", tool_id))?;

        self.install_tool(tool).await?;
        Ok("latest".to_string()) // TODO: Get actual version
    }

    /// Install a specific tool
    async fn install_tool(&self, tool: &CliToolConfig) -> Result<()> {
        info!("📦 Installing {}", tool.name);

        // Check dependencies first
        for dep in &tool.dependencies {
            if !self.check_dependency_installed(dep).await? {
                return Err(anyhow::anyhow!(
                    "Dependency missing: {}. {}",
                    dep.tool,
                    dep.install_hint
                ));
            }
        }

        // Install based on method
        match &tool.install_script {
            InstallScript::Npm(package) => {
                if self.use_local_npm {
                    // Install to local node_modules
                    let npm_dir = self.install_dir.join("node_modules");
                    fs::create_dir_all(&npm_dir)?;

                    let output = Command::new("npm")
                        .arg("install")
                        .arg(package)
                        .current_dir(&self.install_dir)
                        .output()
                        .await?;

                    if !output.status.success() {
                        return Err(anyhow::anyhow!(
                            "Failed to install {}: {}",
                            package,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                } else {
                    // Global install
                    let output = Command::new("npm")
                        .arg("install")
                        .arg("-g")
                        .arg(package)
                        .output()
                        .await?;

                    if !output.status.success() {
                        return Err(anyhow::anyhow!(
                            "Failed to install {}: {}",
                            package,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                }
            }
            // TODO: Implement other installation methods
            _ => {
                return Err(anyhow::anyhow!("Installation method not yet implemented"));
            }
        }

        // Verify installation
        self.verify_tool_installation(tool).await?;

        info!("✅ {} installed successfully", tool.name);
        Ok(())
    }

    /// Check if a dependency is installed
    async fn check_dependency_installed(&self, dep: &ToolDependency) -> Result<bool> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&dep.check_command)
            .output()
            .await?;

        Ok(output.status.success())
    }

    /// Verify a tool is properly installed
    async fn verify_tool_installation(&self, tool: &CliToolConfig) -> Result<()> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&tool.version_check)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to verify {} installation",
                tool.name
            ));
        }

        Ok(())
    }

    /// Check if a tool is installed
    pub async fn is_tool_installed(&self, tool_id: &str) -> Result<bool> {
        let tool = self
            .tool_configs
            .get(tool_id)
            .context(format!("Unknown tool: {}", tool_id))?;

        // Check managed installation
        let managed_path = self.get_managed_tool_path(tool_id)?;
        if managed_path.exists() {
            return Ok(true);
        }

        // Check system installation
        Ok(which::which(&tool.command).is_ok())
    }

    /// Get the path to our managed tool binary/script
    fn get_managed_tool_path(&self, tool_id: &str) -> Result<PathBuf> {
        let tool = self
            .tool_configs
            .get(tool_id)
            .context(format!("Unknown tool: {}", tool_id))?;

        let tool_dir = self.install_dir.join("ai_cli").join(tool_id);

        // For npm tools in local mode
        if self.use_local_npm && matches!(tool.install_method, InstallMethod::Npm) {
            return Ok(self
                .install_dir
                .join("node_modules")
                .join(".bin")
                .join(&tool.command));
        }

        // For other tools
        let binary_name = if cfg!(windows) && !tool.command.ends_with(".exe") {
            format!("{}.exe", tool.command)
        } else {
            tool.command.clone()
        };

        Ok(tool_dir.join(binary_name))
    }

    /// Start a sync record in the database
    async fn start_sync_record(&self, tool_id: &str) -> Result<String> {
        let tool = self
            .tool_configs
            .get(tool_id)
            .context(format!("Unknown tool: {}", tool_id))?;

        let conn = self.db.get_connection()?;
        let sync_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let next_sync = (Utc::now() + Duration::hours(UPDATE_CHECK_INTERVAL_HOURS)).to_rfc3339();

        conn.execute(
            "INSERT INTO sync_metadata (
                id, sync_type, started_at, status, next_sync_due, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![sync_id, &tool.sync_type, now, "in_progress", next_sync, now],
        )?;

        Ok(sync_id)
    }

    /// Complete a sync record
    async fn complete_sync_record(&self, sync_id: &str, version: &str) -> Result<()> {
        let conn = self.db.get_connection()?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE sync_metadata 
             SET completed_at = ?1, status = ?2, intelligence_version = ?3
             WHERE id = ?4",
            params![now, "completed", version, sync_id],
        )?;

        Ok(())
    }

    /// Fail a sync record
    async fn fail_sync_record(&self, sync_id: &str, error: &str) -> Result<()> {
        let conn = self.db.get_connection()?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE sync_metadata 
             SET completed_at = ?1, status = ?2, error_message = ?3
             WHERE id = ?4",
            params![now, "failed", error, sync_id],
        )?;

        Ok(())
    }

    /// Get the install directory
    fn get_install_dir() -> Result<PathBuf> {
        if let Some(home) = dirs::home_dir() {
            Ok(home.join(".hive").join("tools"))
        } else {
            Err(anyhow::anyhow!("Could not determine home directory"))
        }
    }
}

/// Initialize AI CLI updater with database integration
pub async fn initialize_ai_cli_updater_db() -> Result<AiCliUpdaterDB> {
    AiCliUpdaterDB::new().await
}

/// Get the status of all AI CLI tools
pub async fn get_ai_tools_status() -> Result<HashMap<String, bool>> {
    let updater = AiCliUpdaterDB::new().await?;
    let mut status = HashMap::new();

    for tool in AiCliUpdaterDB::get_ai_tools() {
        let installed = updater.is_tool_installed(&tool.id).await?;
        status.insert(tool.id, installed);
    }

    Ok(status)
}
