// Maintenance System for Hive Backend
// Runs in background with minimal resource usage
// Handles OpenRouter sync, database cleanup, and dependency checks

use crate::consensus::maintenance::TemplateMaintenanceManager;
use crate::core::database::DatabaseManager;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceStatus {
    pub last_openrouter_sync: Option<DateTime<Utc>>,
    pub last_database_cleanup: Option<DateTime<Utc>>,
    pub models_updated: u32,
    pub profiles_migrated: u32,
    pub is_running: bool,
    pub next_sync: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct MaintenanceConfig {
    pub openrouter_sync_interval: Duration,
    pub database_cleanup_interval: Duration,
    pub enable_auto_sync: bool,
    pub cpu_threshold: f32,
}

impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self {
            openrouter_sync_interval: Duration::from_secs(24 * 60 * 60), // 24 hours
            database_cleanup_interval: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            enable_auto_sync: true,
            cpu_threshold: 0.5, // 50% CPU max
        }
    }
}

pub struct BackgroundMaintenance {
    db: Arc<DatabaseManager>,
    config: MaintenanceConfig,
    status: Arc<RwLock<MaintenanceStatus>>,
    api_key: Option<String>,
}

impl BackgroundMaintenance {
    pub fn new(db: Arc<DatabaseManager>, api_key: Option<String>) -> Self {
        Self {
            db,
            config: MaintenanceConfig::default(),
            status: Arc::new(RwLock::new(MaintenanceStatus {
                last_openrouter_sync: None,
                last_database_cleanup: None,
                models_updated: 0,
                profiles_migrated: 0,
                is_running: false,
                next_sync: None,
            })),
            api_key,
        }
    }

    pub fn with_config(mut self, config: MaintenanceConfig) -> Self {
        self.config = config;
        self
    }

    /// Start background maintenance tasks
    pub async fn start(self: Arc<Self>) {
        info!("🔧 Starting background maintenance system");
        
        // Update status
        {
            let mut status = self.status.write().await;
            status.is_running = true;
            status.next_sync = Some(Utc::now() + chrono::Duration::from_std(self.config.openrouter_sync_interval).unwrap());
        }

        // Spawn OpenRouter sync task
        if self.config.enable_auto_sync {
            let self_clone = Arc::clone(&self);
            tokio::spawn(async move {
                self_clone.run_openrouter_sync_loop().await;
            });
        }

        // Spawn database cleanup task
        let self_clone = Arc::clone(&self);
        tokio::spawn(async move {
            self_clone.run_database_cleanup_loop().await;
        });

        info!("✅ Background maintenance tasks started");
    }

    /// Run OpenRouter sync in a loop
    async fn run_openrouter_sync_loop(&self) {
        let mut interval = interval(self.config.openrouter_sync_interval);
        
        // Run immediately on start (after 30 seconds to let system stabilize)
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        loop {
            interval.tick().await;
            
            // Check CPU before running
            if !self.check_system_resources().await {
                warn!("System resources too high, skipping OpenRouter sync");
                continue;
            }

            info!("🔄 Running scheduled OpenRouter sync");
            
            match self.sync_openrouter_models().await {
                Ok(report) => {
                    info!("✅ OpenRouter sync completed: {} models updated, {} profiles migrated", 
                        report.models_updated, report.profiles_migrated);
                    
                    // Update status
                    let mut status = self.status.write().await;
                    status.last_openrouter_sync = Some(Utc::now());
                    status.models_updated = report.models_updated;
                    status.profiles_migrated = report.profiles_migrated;
                    status.next_sync = Some(Utc::now() + chrono::Duration::from_std(self.config.openrouter_sync_interval).unwrap());
                }
                Err(e) => {
                    error!("❌ OpenRouter sync failed: {}", e);
                }
            }
        }
    }

    /// Run database cleanup in a loop
    async fn run_database_cleanup_loop(&self) {
        let mut interval = interval(self.config.database_cleanup_interval);
        
        // Wait longer before first cleanup (1 hour)
        tokio::time::sleep(Duration::from_secs(3600)).await;
        
        loop {
            interval.tick().await;
            
            // Check CPU before running
            if !self.check_system_resources().await {
                warn!("System resources too high, skipping database cleanup");
                continue;
            }

            info!("🧹 Running scheduled database cleanup");
            
            match self.cleanup_database().await {
                Ok(cleaned) => {
                    info!("✅ Database cleanup completed: {} records cleaned", cleaned);
                    
                    // Update status
                    let mut status = self.status.write().await;
                    status.last_database_cleanup = Some(Utc::now());
                }
                Err(e) => {
                    error!("❌ Database cleanup failed: {}", e);
                }
            }
        }
    }

    /// Sync models from OpenRouter
    async fn sync_openrouter_models(&self) -> Result<SyncReport> {
        let mut manager = TemplateMaintenanceManager::new(
            self.db.clone(),
            self.api_key.clone()
        );
        
        let report = manager.run_maintenance().await?;
        
        Ok(SyncReport {
            models_updated: report.models_synced,
            profiles_migrated: report.migrated_profiles.len() as u32,
        })
    }

    /// Clean up old database records
    async fn cleanup_database(&self) -> Result<u32> {
        let conn = self.db.get_connection()?;
        let mut cleaned = 0;

        // Clean old conversations (older than 90 days)
        let result = conn.execute(
            "DELETE FROM conversations WHERE created_at < datetime('now', '-90 days')",
            [],
        )?;
        cleaned += result as u32;

        // Clean orphaned messages
        let result = conn.execute(
            "DELETE FROM messages WHERE conversation_id NOT IN (SELECT id FROM conversations)",
            [],
        )?;
        cleaned += result as u32;

        // Clean old token usage (older than 180 days)
        let result = conn.execute(
            "DELETE FROM token_usage WHERE created_at < datetime('now', '-180 days')",
            [],
        )?;
        cleaned += result as u32;

        // Optimize database
        conn.execute("VACUUM", [])?;
        
        Ok(cleaned)
    }

    /// Check if system resources allow maintenance
    async fn check_system_resources(&self) -> bool {
        // Simple check using sysinfo
        use sysinfo::System;
        let mut sys = System::new_all();
        sys.refresh_cpu();
        
        // Get average CPU usage (returns percentage 0-100)
        let cpu_usage = sys.global_cpu_info().cpu_usage() / 100.0;
        cpu_usage < self.config.cpu_threshold
    }

    /// Get current maintenance status
    pub async fn get_status(&self) -> MaintenanceStatus {
        self.status.read().await.clone()
    }

    /// Force run OpenRouter sync
    pub async fn force_sync(&self) -> Result<SyncReport> {
        info!("⚡ Force running OpenRouter sync");
        let report = self.sync_openrouter_models().await?;
        
        // Update status
        let mut status = self.status.write().await;
        status.last_openrouter_sync = Some(Utc::now());
        status.models_updated = report.models_updated;
        status.profiles_migrated = report.profiles_migrated;
        
        Ok(report)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
    pub models_updated: u32,
    pub profiles_migrated: u32,
}