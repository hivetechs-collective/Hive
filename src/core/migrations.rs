use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info, warn, error};

use super::database::{execute_query, fetch_all_query, fetch_optional_query, get_database};

/// Migration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: String,
    pub name: String,
    pub description: String,
    pub sql: String,
    pub rollback_sql: Option<String>,
    pub checksum: String,
}

/// Migration execution result
#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationResult {
    pub version: String,
    pub name: String,
    pub success: bool,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

/// Migration manager for database schema evolution
#[derive(Debug)]
pub struct MigrationManager {
    migrations_dir: PathBuf,
    migrations: Vec<Migration>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(migrations_dir: PathBuf) -> Self {
        Self {
            migrations_dir,
            migrations: Vec::new(),
        }
    }

    /// Load all migration files from the migrations directory
    pub async fn load_migrations(&mut self) -> Result<()> {
        let mut entries = fs::read_dir(&self.migrations_dir)
            .await
            .context("Failed to read migrations directory")?;

        let mut migration_files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                migration_files.push(path);
            }
        }

        // Sort by filename to ensure proper ordering
        migration_files.sort();

        for file_path in migration_files {
            let migration = self.load_migration_file(&file_path).await?;
            self.migrations.push(migration);
        }

        info!("Loaded {} migrations", self.migrations.len());
        Ok(())
    }

    /// Load a single migration file
    async fn load_migration_file(&self, path: &PathBuf) -> Result<Migration> {
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read migration file: {}", path.display()))?;

        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Invalid migration filename")?;

        // Parse migration metadata from comments
        let (metadata, sql) = self.parse_migration_content(&content)?;
        let version = metadata.get("version")
            .or_else(|| filename.split('_').next())
            .context("Migration version not found")?
            .to_string();

        let name = metadata.get("name")
            .or_else(|| Some(filename))
            .unwrap()
            .to_string();

        let description = metadata.get("description")
            .unwrap_or(&"No description".to_string())
            .clone();

        let rollback_sql = metadata.get("rollback").cloned();

        // Calculate checksum for integrity verification
        let checksum = format!("{:x}", md5::compute(&sql));

        Ok(Migration {
            version,
            name,
            description,
            sql,
            rollback_sql,
            checksum,
        })
    }

    /// Parse migration content to extract metadata and SQL
    fn parse_migration_content(&self, content: &str) -> Result<(HashMap<String, String>, String)> {
        let mut metadata = HashMap::new();
        let mut sql_lines = Vec::new();
        let mut in_metadata = false;

        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("-- @") {
                in_metadata = true;
                let parts: Vec<&str> = trimmed[3..].splitn(2, ':').collect();
                if parts.len() == 2 {
                    metadata.insert(
                        parts[0].trim().to_lowercase(),
                        parts[1].trim().to_string(),
                    );
                }
            } else if trimmed.starts_with("--") && !in_metadata {
                // Regular comment, skip
                continue;
            } else {
                in_metadata = false;
                sql_lines.push(line);
            }
        }

        let sql = sql_lines.join("\n").trim().to_string();
        Ok((metadata, sql))
    }

    /// Initialize migration tracking table
    pub async fn initialize_migration_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                checksum TEXT NOT NULL,
                executed_at TEXT NOT NULL,
                execution_time_ms INTEGER NOT NULL,
                rollback_sql TEXT
            );
            
            CREATE INDEX IF NOT EXISTS idx_schema_migrations_executed 
            ON schema_migrations(executed_at);
        "#;

        execute_query(sql, vec![]).await?;
        debug!("Migration tracking table initialized");
        Ok(())
    }

    /// Get list of applied migrations
    pub async fn get_applied_migrations(&self) -> Result<Vec<String>> {
        let rows = fetch_all_query(
            "SELECT version FROM schema_migrations ORDER BY executed_at",
            vec![],
        ).await?;

        let versions = rows
            .into_iter()
            .map(|row| row.get::<String, _>("version"))
            .collect();

        Ok(versions)
    }

    /// Run all pending migrations
    pub async fn migrate(&self) -> Result<Vec<MigrationResult>> {
        self.initialize_migration_table().await?;
        
        let applied_migrations = self.get_applied_migrations().await?;
        let mut results = Vec::new();

        for migration in &self.migrations {
            if applied_migrations.contains(&migration.version) {
                debug!("Migration {} already applied, skipping", migration.version);
                continue;
            }

            info!("Applying migration: {} - {}", migration.version, migration.name);
            let result = self.apply_migration(migration).await;
            
            match &result {
                Ok(mr) if mr.success => {
                    info!("Migration {} completed successfully in {}ms", 
                          migration.version, mr.execution_time_ms);
                }
                Ok(mr) => {
                    error!("Migration {} failed: {}", 
                           migration.version, mr.error.as_deref().unwrap_or("Unknown error"));
                }
                Err(e) => {
                    error!("Migration {} failed with error: {}", migration.version, e);
                }
            }

            match result {
                Ok(mr) => results.push(mr),
                Err(e) => {
                    results.push(MigrationResult {
                        version: migration.version.clone(),
                        name: migration.name.clone(),
                        success: false,
                        execution_time_ms: 0,
                        error: Some(e.to_string()),
                    });
                    break; // Stop on first error
                }
            }
        }

        Ok(results)
    }

    /// Apply a single migration
    async fn apply_migration(&self, migration: &Migration) -> Result<MigrationResult> {
        let start_time = std::time::Instant::now();
        
        // Verify migration hasn't been tampered with
        let current_checksum = format!("{:x}", md5::compute(&migration.sql));
        if current_checksum != migration.checksum {
            return Ok(MigrationResult {
                version: migration.version.clone(),
                name: migration.name.clone(),
                success: false,
                execution_time_ms: 0,
                error: Some("Migration checksum mismatch - file may have been modified".to_string()),
            });
        }

        let result = match self.execute_migration_sql(&migration.sql).await {
            Ok(_) => {
                // Record successful migration
                self.record_migration(migration, start_time.elapsed().as_millis() as u64).await?;
                
                MigrationResult {
                    version: migration.version.clone(),
                    name: migration.name.clone(),
                    success: true,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                }
            }
            Err(e) => MigrationResult {
                version: migration.version.clone(),
                name: migration.name.clone(),
                success: false,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
            },
        };

        Ok(result)
    }

    /// Execute migration SQL with proper error handling
    async fn execute_migration_sql(&self, sql: &str) -> Result<()> {
        let db = get_database().await?;
        
        // Split SQL into statements and execute each one
        let statements: Vec<&str> = sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for statement in statements {
            if !statement.is_empty() {
                sqlx::query(statement)
                    .execute(db.pool())
                    .await
                    .with_context(|| format!("Failed to execute migration statement: {}", statement))?;
            }
        }

        Ok(())
    }

    /// Record a successful migration in the tracking table
    async fn record_migration(&self, migration: &Migration, execution_time_ms: u64) -> Result<()> {
        let sql = r#"
            INSERT INTO schema_migrations 
            (version, name, description, checksum, executed_at, execution_time_ms, rollback_sql)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        execute_query(
            sql,
            vec![
                &migration.version,
                &migration.name,
                &migration.description,
                &migration.checksum,
                &Utc::now().to_rfc3339(),
                &(execution_time_ms as i64),
                &migration.rollback_sql.as_deref().unwrap_or(""),
            ],
        ).await?;

        Ok(())
    }

    /// Rollback a specific migration
    pub async fn rollback(&self, version: &str) -> Result<MigrationResult> {
        let migration_info = fetch_optional_query(
            "SELECT * FROM schema_migrations WHERE version = ?",
            vec![&version],
        ).await?;

        let migration_row = migration_info
            .ok_or_else(|| anyhow::anyhow!("Migration {} not found in database", version))?;

        let rollback_sql: String = migration_row.get("rollback_sql");
        if rollback_sql.is_empty() {
            return Ok(MigrationResult {
                version: version.to_string(),
                name: migration_row.get("name"),
                success: false,
                execution_time_ms: 0,
                error: Some("No rollback SQL provided for this migration".to_string()),
            });
        }

        let start_time = std::time::Instant::now();
        let result = match self.execute_migration_sql(&rollback_sql).await {
            Ok(_) => {
                // Remove migration record
                execute_query(
                    "DELETE FROM schema_migrations WHERE version = ?",
                    vec![&version],
                ).await?;

                MigrationResult {
                    version: version.to_string(),
                    name: migration_row.get("name"),
                    success: true,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                }
            }
            Err(e) => MigrationResult {
                version: version.to_string(),
                name: migration_row.get("name"),
                success: false,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
            },
        };

        Ok(result)
    }

    /// Get migration status
    pub async fn get_migration_status(&self) -> Result<MigrationStatus> {
        let applied_migrations = self.get_applied_migrations().await?;
        let total_migrations = self.migrations.len();
        let pending_migrations: Vec<String> = self
            .migrations
            .iter()
            .filter(|m| !applied_migrations.contains(&m.version))
            .map(|m| m.version.clone())
            .collect();

        Ok(MigrationStatus {
            total_migrations,
            applied_count: applied_migrations.len(),
            pending_count: pending_migrations.len(),
            applied_migrations,
            pending_migrations,
        })
    }

    /// Verify database schema integrity
    pub async fn verify_integrity(&self) -> Result<IntegrityReport> {
        let mut issues = Vec::new();
        let applied_migrations = self.get_applied_migrations().await?;

        // Check for missing migrations
        for applied_version in &applied_migrations {
            if !self.migrations.iter().any(|m| &m.version == applied_version) {
                issues.push(format!("Applied migration {} not found in migration files", applied_version));
            }
        }

        // Verify checksums
        for migration in &self.migrations {
            if applied_migrations.contains(&migration.version) {
                let stored_checksum = fetch_optional_query(
                    "SELECT checksum FROM schema_migrations WHERE version = ?",
                    vec![&migration.version],
                ).await?;

                if let Some(row) = stored_checksum {
                    let stored: String = row.get("checksum");
                    if stored != migration.checksum {
                        issues.push(format!(
                            "Migration {} checksum mismatch (stored: {}, current: {})",
                            migration.version, stored, migration.checksum
                        ));
                    }
                }
            }
        }

        Ok(IntegrityReport {
            healthy: issues.is_empty(),
            issues,
        })
    }
}

/// Migration status information
#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub total_migrations: usize,
    pub applied_count: usize,
    pub pending_count: usize,
    pub applied_migrations: Vec<String>,
    pub pending_migrations: Vec<String>,
}

/// Database integrity report
#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub healthy: bool,
    pub issues: Vec<String>,
}

/// Initialize and run migrations
pub async fn initialize_migrations(migrations_dir: PathBuf) -> Result<Vec<MigrationResult>> {
    let mut manager = MigrationManager::new(migrations_dir);
    manager.load_migrations().await?;
    manager.migrate().await
}

/// Get current migration status
pub async fn get_migration_status(migrations_dir: PathBuf) -> Result<MigrationStatus> {
    let mut manager = MigrationManager::new(migrations_dir);
    manager.load_migrations().await?;
    manager.get_migration_status().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn create_test_migration(dir: &PathBuf, version: &str, name: &str, sql: &str) -> Result<()> {
        let filename = format!("{}_{}.sql", version, name);
        let file_path = dir.join(filename);
        
        let content = format!(
            r#"-- @ version: {}
-- @ name: {}
-- @ description: Test migration
-- @ rollback: DROP TABLE IF EXISTS test_table;

{}"#,
            version, name, sql
        );
        
        fs::write(file_path, content).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_migration_loading() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let migrations_dir = temp_dir.path().to_path_buf();

        // Create test migration files
        create_test_migration(
            &migrations_dir,
            "001",
            "create_users",
            "CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT);",
        ).await?;

        create_test_migration(
            &migrations_dir,
            "002",
            "add_email_column",
            "ALTER TABLE users ADD COLUMN email TEXT;",
        ).await?;

        let mut manager = MigrationManager::new(migrations_dir);
        manager.load_migrations().await?;

        assert_eq!(manager.migrations.len(), 2);
        assert_eq!(manager.migrations[0].version, "001");
        assert_eq!(manager.migrations[1].version, "002");

        Ok(())
    }

    #[tokio::test]
    async fn test_migration_content_parsing() -> Result<()> {
        let manager = MigrationManager::new(PathBuf::new());
        
        let content = r#"-- @ version: 001
-- @ name: test_migration
-- @ description: A test migration
-- @ rollback: DROP TABLE test;

-- This is a comment
CREATE TABLE test (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

INSERT INTO test (id, name) VALUES ('1', 'test');"#;

        let (metadata, sql) = manager.parse_migration_content(content)?;
        
        assert_eq!(metadata.get("version"), Some(&"001".to_string()));
        assert_eq!(metadata.get("name"), Some(&"test_migration".to_string()));
        assert!(sql.contains("CREATE TABLE test"));
        assert!(sql.contains("INSERT INTO test"));
        assert!(!sql.contains("-- @ version"));

        Ok(())
    }
}