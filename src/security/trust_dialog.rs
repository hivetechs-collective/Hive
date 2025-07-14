//! Trust Dialog System for Claude Code-style security
//!
//! Provides secure directory access with user consent and persistent trust decisions.

use anyhow::Result;
use chrono::{DateTime, Utc};
use console::{style, Style};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trust decision for a directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustDecision {
    pub path: PathBuf,
    pub trusted: bool,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub reason: Option<String>,
    pub scope: TrustScope,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Vec<TrustCondition>,
}

/// Scope of trust decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustScope {
    /// Trust only the specific directory
    Directory,
    /// Trust directory and all subdirectories
    Recursive,
    /// Trust only for current session
    Session,
    /// Trust for specific operations only
    Operations(Vec<String>),
}

/// Conditions that must be met for trust to be valid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustCondition {
    /// Trust only if no files have been modified since timestamp
    NoModificationsSince(DateTime<Utc>),
    /// Trust only if directory contains specific files
    ContainsFiles(Vec<String>),
    /// Trust only if directory is a git repository
    IsGitRepository,
    /// Trust only if directory size is under limit (in bytes)
    MaxSize(u64),
    /// Trust only for specific file extensions
    FileExtensions(Vec<String>),
}

/// Trust dialog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustDialogConfig {
    /// Enable trust dialog prompts
    pub enabled: bool,
    /// Auto-trust git repositories
    pub auto_trust_git: bool,
    /// Trust timeout in seconds
    pub trust_timeout: u64,
    /// Maximum directory size for auto-trust (in bytes)
    pub max_auto_trust_size: u64,
    /// File extensions that are always trusted
    pub trusted_extensions: Vec<String>,
    /// Directories that are always trusted
    pub trusted_paths: Vec<PathBuf>,
    /// Enable interactive prompts
    pub interactive: bool,
}

impl Default for TrustDialogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_trust_git: true,
            trust_timeout: 86400,                   // 24 hours
            max_auto_trust_size: 100 * 1024 * 1024, // 100MB
            trusted_extensions: vec![
                "md".to_string(),
                "txt".to_string(),
                "json".to_string(),
                "toml".to_string(),
                "yml".to_string(),
                "yaml".to_string(),
            ],
            trusted_paths: vec![
                PathBuf::from("~/.config"),
                PathBuf::from("~/.local/share"),
                PathBuf::from("/usr/local/share"),
            ],
            interactive: true,
        }
    }
}

/// Trust dialog system
pub struct TrustDialogSystem {
    config: TrustDialogConfig,
    trust_store: Arc<RwLock<HashMap<PathBuf, TrustDecision>>>,
    audit_logger: Option<Arc<crate::security::audit::EnterpriseAuditLogger>>,
}

impl TrustDialogSystem {
    /// Create a new trust dialog system
    pub fn new(config: TrustDialogConfig) -> Self {
        Self {
            config,
            trust_store: Arc::new(RwLock::new(HashMap::new())),
            audit_logger: None,
        }
    }

    /// Set audit logger for security event logging
    pub fn with_audit_logger(
        mut self,
        logger: Arc<crate::security::audit::EnterpriseAuditLogger>,
    ) -> Self {
        self.audit_logger = Some(logger);
        self
    }

    /// Load trust decisions from storage
    pub async fn load_trust_store(&self, store_path: &Path) -> Result<()> {
        if !store_path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(store_path).await?;
        let decisions: Vec<TrustDecision> = serde_json::from_str(&content)?;

        let mut store = self.trust_store.write().await;
        for decision in decisions {
            store.insert(decision.path.clone(), decision);
        }

        Ok(())
    }

    /// Save trust decisions to storage
    pub async fn save_trust_store(&self, store_path: &Path) -> Result<()> {
        let store = self.trust_store.read().await;
        let decisions: Vec<TrustDecision> = store.values().cloned().collect();

        let content = serde_json::to_string_pretty(&decisions)?;
        tokio::fs::write(store_path, content).await?;

        Ok(())
    }

    /// Check if a directory is trusted
    pub async fn is_trusted(&self, path: &Path, operation: Option<&str>) -> Result<bool> {
        if !self.config.enabled {
            return Ok(true);
        }

        // Check if path is in trusted paths
        if self.is_always_trusted(path) {
            return Ok(true);
        }

        // Check existing trust decisions
        let store = self.trust_store.read().await;

        // Check for exact match
        if let Some(decision) = store.get(path) {
            return Ok(self.is_decision_valid(decision, operation).await?);
        }

        // Check for parent directory matches with recursive scope
        for (trusted_path, decision) in store.iter() {
            if path.starts_with(trusted_path) {
                match decision.scope {
                    TrustScope::Recursive => {
                        if self.is_decision_valid(decision, operation).await? {
                            return Ok(true);
                        }
                    }
                    TrustScope::Operations(ref ops) => {
                        if let Some(op) = operation {
                            if ops.contains(&op.to_string()) {
                                return Ok(self.is_decision_valid(decision, operation).await?);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(false)
    }

    /// Request trust for a directory with interactive dialog
    pub async fn request_trust(&self, path: &Path, operation: Option<&str>) -> Result<bool> {
        if !self.config.enabled {
            return Ok(true);
        }

        // Check if already trusted
        if self.is_trusted(path, operation).await? {
            return Ok(true);
        }

        // Auto-trust conditions
        if self.should_auto_trust(path).await? {
            let decision = TrustDecision {
                path: path.to_path_buf(),
                trusted: true,
                timestamp: Utc::now(),
                user_id: None,
                reason: Some("Auto-trusted based on configuration".to_string()),
                scope: TrustScope::Recursive,
                expires_at: Some(
                    Utc::now() + chrono::Duration::seconds(self.config.trust_timeout as i64),
                ),
                conditions: vec![],
            };

            self.store_decision(decision).await?;
            return Ok(true);
        }

        // Interactive trust dialog
        if self.config.interactive {
            return self.show_trust_dialog(path, operation).await;
        }

        // Default deny
        Ok(false)
    }

    /// Show interactive trust dialog
    async fn show_trust_dialog(&self, path: &Path, operation: Option<&str>) -> Result<bool> {
        let theme = ColorfulTheme::default();

        println!();
        println!("{}", style("🛡️  Security Trust Dialog").bold().cyan());
        println!(
            "{}",
            style(
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            )
            .dim()
        );
        println!();

        println!(
            "{} {}",
            style("Directory:").bold(),
            style(path.display()).yellow()
        );

        if let Some(op) = operation {
            println!("{} {}", style("Operation:").bold(), style(op).yellow());
        }

        println!();

        // Show directory information
        self.show_directory_info(path).await?;

        println!();
        println!(
            "{}",
            style("Hive needs permission to access this directory.").yellow()
        );
        println!(
            "{}",
            style("This is a security measure to protect your files.").dim()
        );
        println!();

        // Trust options
        let options = vec![
            "Trust this directory (this session only)",
            "Trust this directory and all subdirectories",
            "Trust for specific operations only",
            "Trust permanently",
            "Deny access",
        ];

        let selection = Select::with_theme(&theme)
            .with_prompt("Choose trust level")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                // Session only
                let decision = TrustDecision {
                    path: path.to_path_buf(),
                    trusted: true,
                    timestamp: Utc::now(),
                    user_id: None,
                    reason: Some("User granted session trust".to_string()),
                    scope: TrustScope::Session,
                    expires_at: None,
                    conditions: vec![],
                };
                self.store_decision(decision).await?;
                Ok(true)
            }
            1 => {
                // Recursive trust
                let decision = TrustDecision {
                    path: path.to_path_buf(),
                    trusted: true,
                    timestamp: Utc::now(),
                    user_id: None,
                    reason: Some("User granted recursive trust".to_string()),
                    scope: TrustScope::Recursive,
                    expires_at: Some(
                        Utc::now() + chrono::Duration::seconds(self.config.trust_timeout as i64),
                    ),
                    conditions: vec![],
                };
                self.store_decision(decision).await?;
                Ok(true)
            }
            2 => {
                // Operations only
                let operations = Input::<String>::with_theme(&theme)
                    .with_prompt("Enter operations (comma-separated)")
                    .interact()?;

                let ops: Vec<String> = operations
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                let decision = TrustDecision {
                    path: path.to_path_buf(),
                    trusted: true,
                    timestamp: Utc::now(),
                    user_id: None,
                    reason: Some("User granted operation-specific trust".to_string()),
                    scope: TrustScope::Operations(ops),
                    expires_at: Some(
                        Utc::now() + chrono::Duration::seconds(self.config.trust_timeout as i64),
                    ),
                    conditions: vec![],
                };
                self.store_decision(decision).await?;
                Ok(true)
            }
            3 => {
                // Permanent trust
                let confirm = Confirm::with_theme(&theme)
                    .with_prompt("Are you sure you want to trust this directory permanently?")
                    .interact()?;

                if confirm {
                    let decision = TrustDecision {
                        path: path.to_path_buf(),
                        trusted: true,
                        timestamp: Utc::now(),
                        user_id: None,
                        reason: Some("User granted permanent trust".to_string()),
                        scope: TrustScope::Recursive,
                        expires_at: None,
                        conditions: vec![],
                    };
                    self.store_decision(decision).await?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            4 => {
                // Deny access
                println!(
                    "{}",
                    style("Access denied. Hive will not read files from this directory.").red()
                );
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Show directory information for trust decision
    async fn show_directory_info(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            println!("{}", style("⚠️  Directory does not exist").red());
            return Ok(());
        }

        // Check if it's a git repository
        if path.join(".git").exists() {
            println!("{}", style("📁 Git repository detected").green());
        }

        // Show directory size
        if let Ok(size) = self.get_directory_size(path).await {
            println!(
                "{} {}",
                style("📊 Directory size:").bold(),
                style(format_bytes(size)).cyan()
            );
        }

        // Show file count
        if let Ok(count) = self.count_files(path).await {
            println!("{} {}", style("📄 File count:").bold(), style(count).cyan());
        }

        // Show recent modification
        if let Ok(modified) = self.get_last_modified(path).await {
            println!(
                "{} {}",
                style("🕒 Last modified:").bold(),
                style(modified.format("%Y-%m-%d %H:%M:%S UTC")).cyan()
            );
        }

        Ok(())
    }

    /// Store a trust decision
    async fn store_decision(&self, decision: TrustDecision) -> Result<()> {
        let path = decision.path.clone();
        let mut store = self.trust_store.write().await;
        store.insert(path, decision.clone());

        // Log security event
        if let Some(logger) = &self.audit_logger {
            logger
                .log_security_event(
                    crate::security::audit::AuditEventType::TrustDecision,
                    format!("Trust decision made for path: {}", decision.path.display()),
                    decision.user_id.clone(),
                )
                .await?;
        }

        Ok(())
    }

    /// Check if path is always trusted
    fn is_always_trusted(&self, path: &Path) -> bool {
        for trusted_path in &self.config.trusted_paths {
            if path.starts_with(trusted_path) {
                return true;
            }
        }
        false
    }

    /// Check if should auto-trust based on configuration
    async fn should_auto_trust(&self, path: &Path) -> Result<bool> {
        // Check if it's a git repository and auto-trust is enabled
        if self.config.auto_trust_git && path.join(".git").exists() {
            return Ok(true);
        }

        // Check directory size
        if let Ok(size) = self.get_directory_size(path).await {
            if size <= self.config.max_auto_trust_size {
                return Ok(true);
            }
        }

        // Check if contains only trusted file extensions
        if self.contains_only_trusted_extensions(path).await? {
            return Ok(true);
        }

        Ok(false)
    }

    /// Check if decision is still valid
    async fn is_decision_valid(
        &self,
        decision: &TrustDecision,
        operation: Option<&str>,
    ) -> Result<bool> {
        if !decision.trusted {
            return Ok(false);
        }

        // Check expiration
        if let Some(expires_at) = decision.expires_at {
            if Utc::now() > expires_at {
                return Ok(false);
            }
        }

        // Check operation scope
        if let TrustScope::Operations(ref ops) = decision.scope {
            if let Some(op) = operation {
                if !ops.contains(&op.to_string()) {
                    return Ok(false);
                }
            }
        }

        // Check conditions
        for condition in &decision.conditions {
            if !self.check_condition(condition, &decision.path).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if a trust condition is met
    async fn check_condition(&self, condition: &TrustCondition, path: &Path) -> Result<bool> {
        match condition {
            TrustCondition::NoModificationsSince(timestamp) => {
                if let Ok(modified) = self.get_last_modified(path).await {
                    Ok(modified <= *timestamp)
                } else {
                    Ok(false)
                }
            }
            TrustCondition::ContainsFiles(files) => {
                for file in files {
                    if !path.join(file).exists() {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            TrustCondition::IsGitRepository => Ok(path.join(".git").exists()),
            TrustCondition::MaxSize(max_size) => {
                if let Ok(size) = self.get_directory_size(path).await {
                    Ok(size <= *max_size)
                } else {
                    Ok(false)
                }
            }
            TrustCondition::FileExtensions(extensions) => {
                self.contains_only_specific_extensions(path, extensions)
                    .await
            }
        }
    }

    /// Get directory size in bytes
    async fn get_directory_size(&self, path: &Path) -> Result<u64> {
        let mut total_size = 0;
        let mut entries = tokio::fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += Box::pin(self.get_directory_size(&entry.path())).await?;
            }
        }

        Ok(total_size)
    }

    /// Count files in directory
    async fn count_files(&self, path: &Path) -> Result<usize> {
        let mut count = 0;
        let mut entries = tokio::fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                count += 1;
            } else if metadata.is_dir() {
                count += Box::pin(self.count_files(&entry.path())).await?;
            }
        }

        Ok(count)
    }

    /// Get last modified time
    async fn get_last_modified(&self, path: &Path) -> Result<DateTime<Utc>> {
        let metadata = tokio::fs::metadata(path).await?;
        let modified = metadata.modified()?;
        Ok(DateTime::from(modified))
    }

    /// Check if directory contains only trusted extensions
    async fn contains_only_trusted_extensions(&self, path: &Path) -> Result<bool> {
        let mut entries = tokio::fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                if let Some(extension) = entry.path().extension() {
                    if let Some(ext_str) = extension.to_str() {
                        if !self
                            .config
                            .trusted_extensions
                            .contains(&ext_str.to_string())
                        {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            } else if metadata.is_dir() {
                if !Box::pin(self.contains_only_trusted_extensions(&entry.path())).await? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Check if directory contains only specific extensions
    async fn contains_only_specific_extensions(
        &self,
        path: &Path,
        extensions: &[String],
    ) -> Result<bool> {
        let mut entries = tokio::fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                if let Some(extension) = entry.path().extension() {
                    if let Some(ext_str) = extension.to_str() {
                        if !extensions.contains(&ext_str.to_string()) {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            } else if metadata.is_dir() {
                if !Box::pin(self.contains_only_specific_extensions(&entry.path(), extensions))
                    .await?
                {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Revoke trust for a directory
    pub async fn revoke_trust(&self, path: &Path) -> Result<()> {
        let mut store = self.trust_store.write().await;
        store.remove(path);

        // Log security event
        if let Some(logger) = &self.audit_logger {
            logger
                .log_security_event(
                    crate::security::audit::AuditEventType::TrustRevoked,
                    format!("Trust revoked for path: {}", path.display()),
                    None,
                )
                .await?;
        }

        Ok(())
    }

    /// List all trust decisions
    pub async fn list_trust_decisions(&self) -> Result<Vec<TrustDecision>> {
        let store = self.trust_store.read().await;
        Ok(store.values().cloned().collect())
    }

    /// Clear expired trust decisions
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut store = self.trust_store.write().await;
        let now = Utc::now();
        let mut expired_count = 0;

        store.retain(|_, decision| {
            if let Some(expires_at) = decision.expires_at {
                if now > expires_at {
                    expired_count += 1;
                    return false;
                }
            }
            true
        });

        Ok(expired_count)
    }

    /// Check if a directory is trusted
    pub async fn is_directory_trusted(&self, path: &Path) -> Result<bool> {
        let store = self.trust_store.read().await;

        // Check exact path first
        if let Some(decision) = store.get(path) {
            if !decision.trusted {
                return Ok(false);
            }

            // Check if expired
            if let Some(expires_at) = decision.expires_at {
                if Utc::now() > expires_at {
                    return Ok(false);
                }
            }

            return Ok(true);
        }

        // Check parent directories if recursive trust is enabled
        let mut current = path;
        while let Some(parent) = current.parent() {
            if let Some(decision) = store.get(parent) {
                if matches!(decision.scope, TrustScope::Recursive) && decision.trusted {
                    // Check if not expired
                    if let Some(expires_at) = decision.expires_at {
                        if Utc::now() <= expires_at {
                            return Ok(true);
                        }
                    } else {
                        return Ok(true);
                    }
                }
            }
            current = parent;
        }

        Ok(false)
    }

    /// Request trust for a directory
    pub async fn request_directory_trust(&self, path: &Path, reason: &str) -> Result<bool> {
        // Show trust dialog
        let theme = ColorfulTheme::default();

        println!("\n{}", style("🔐 Directory Access Request").bold().yellow());
        println!("{}", style("─".repeat(50)).dim());
        println!("Path: {}", style(path.display()).cyan());
        println!("Reason: {}", style(reason).italic());
        println!();

        let trust = Confirm::with_theme(&theme)
            .with_prompt("Do you trust this directory?")
            .interact()?;

        if trust {
            let scope_options = vec![
                "This directory only",
                "This directory and subdirectories",
                "This session only",
            ];

            let scope_idx = Select::with_theme(&theme)
                .with_prompt("Select trust scope")
                .items(&scope_options)
                .default(0)
                .interact()?;

            let scope = match scope_idx {
                0 => TrustScope::Directory,
                1 => TrustScope::Recursive,
                2 => TrustScope::Session,
                _ => TrustScope::Directory,
            };

            let expires_at = if matches!(scope, TrustScope::Session) {
                Some(Utc::now() + chrono::Duration::hours(24))
            } else {
                None
            };

            let decision = TrustDecision {
                path: path.to_path_buf(),
                trusted: true,
                timestamp: Utc::now(),
                user_id: None,
                reason: Some(reason.to_string()),
                scope,
                expires_at,
                conditions: vec![],
            };

            let mut store = self.trust_store.write().await;
            store.insert(path.to_path_buf(), decision);

            // Log audit event if configured
            if let Some(logger) = &self.audit_logger {
                logger
                    .log_trust_decision(path, true, Some(reason.to_string()))
                    .await?;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Revoke trust for a directory
    pub async fn revoke_directory_trust(&self, path: &Path) -> Result<()> {
        let mut store = self.trust_store.write().await;
        store.remove(path);

        // Log audit event if configured
        if let Some(logger) = &self.audit_logger {
            logger.log_trust_revoked(path).await?;
        }

        Ok(())
    }

    /// Clean up expired trust decisions
    pub async fn cleanup_expired_trust(&self) -> Result<usize> {
        let mut store = self.trust_store.write().await;
        let now = Utc::now();
        let mut expired_count = 0;

        store.retain(|_, decision| {
            if let Some(expires_at) = decision.expires_at {
                if now > expires_at {
                    expired_count += 1;
                    return false;
                }
            }
            true
        });

        Ok(expired_count)
    }
}

/// Format bytes in human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_trust_dialog_creation() {
        let config = TrustDialogConfig::default();
        let trust_dialog = TrustDialogSystem::new(config);

        // Test with disabled config
        let mut disabled_config = TrustDialogConfig::default();
        disabled_config.enabled = false;
        let disabled_dialog = TrustDialogSystem::new(disabled_config);

        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        // Should always trust when disabled
        assert!(disabled_dialog.is_trusted(path, None).await.unwrap());
    }

    #[tokio::test]
    async fn test_auto_trust_git_repository() {
        let config = TrustDialogConfig::default();
        let trust_dialog = TrustDialogSystem::new(config);

        let temp_dir = tempdir().unwrap();
        let git_dir = temp_dir.path().join(".git");
        tokio::fs::create_dir(&git_dir).await.unwrap();

        // Should auto-trust git repository
        assert!(trust_dialog
            .should_auto_trust(temp_dir.path())
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_trust_store_persistence() {
        let config = TrustDialogConfig::default();
        let trust_dialog = TrustDialogSystem::new(config);

        let temp_dir = tempdir().unwrap();
        let store_path = temp_dir.path().join("trust_store.json");
        let test_path = temp_dir.path().join("test_dir");

        // Create a trust decision
        let decision = TrustDecision {
            path: test_path.clone(),
            trusted: true,
            timestamp: Utc::now(),
            user_id: None,
            reason: Some("Test decision".to_string()),
            scope: TrustScope::Directory,
            expires_at: None,
            conditions: vec![],
        };

        trust_dialog.store_decision(decision).await.unwrap();

        // Save and reload
        trust_dialog.save_trust_store(&store_path).await.unwrap();

        let new_dialog = TrustDialogSystem::new(TrustDialogConfig::default());
        new_dialog.load_trust_store(&store_path).await.unwrap();

        // Should be trusted
        assert!(new_dialog.is_trusted(&test_path, None).await.unwrap());
    }
}
