//! Database migration management
//!
//! Handles running SQL migrations and initializing default data

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use std::path::Path;
use tracing::{debug, info};

use crate::database::schema;

/// Migration definition
#[derive(Debug)]
pub struct Migration {
    pub version: i32,
    pub name: String,
    pub sql: String,
}

/// Run all pending migrations
pub async fn run_migrations(conn: &Connection) -> Result<()> {
    info!("Running database migrations");

    // Create migrations tracking table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;

    // Get current version
    let current_version: Option<i32> = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .optional()?;

    let current_version = current_version.unwrap_or(0);
    info!("Current database version: {}", current_version);

    // Load migrations from the migrations directory
    let migrations = load_migrations().await?;

    // Apply pending migrations
    let mut applied_count = 0;
    for migration in migrations {
        if migration.version > current_version {
            info!(
                "Applying migration {}: {}",
                migration.version, migration.name
            );
            apply_migration(conn, &migration)?;
            applied_count += 1;
        }
    }

    if applied_count == 0 {
        info!("Database is up to date");
    } else {
        info!("Applied {} migrations", applied_count);
    }

    // Initialize default data if this is a fresh database
    if current_version == 0 && applied_count > 0 {
        info!("Fresh database detected, initializing default data");
        schema::initialize_default_data(conn)?;
    }

    Ok(())
}

/// Load migrations from the migrations directory
async fn load_migrations() -> Result<Vec<Migration>> {
    let migrations_dir = Path::new("migrations");

    if !migrations_dir.exists() {
        debug!("Migrations directory not found, using embedded migrations");
        return Ok(get_embedded_migrations());
    }

    let mut migrations = Vec::new();
    let mut entries = tokio::fs::read_dir(migrations_dir)
        .await
        .context("Failed to read migrations directory")?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sql") {
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            // Extract version from filename (e.g., "001_initial_schema.sql" -> 1)
            let version: i32 = filename
                .split('_')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            if version > 0 {
                let sql = tokio::fs::read_to_string(&path)
                    .await
                    .context("Failed to read migration file")?;

                // Extract name from metadata comment
                let name = sql
                    .lines()
                    .find(|l| l.starts_with("-- @ name:"))
                    .and_then(|l| l.strip_prefix("-- @ name:"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| format!("migration_{}", version));

                migrations.push(Migration { version, name, sql });
            }
        }
    }

    migrations.sort_by_key(|m| m.version);
    Ok(migrations)
}

/// Get embedded migrations for when the migrations directory is not available
fn get_embedded_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            name: "initial_schema".to_string(),
            sql: include_str!("../../migrations/001_initial_schema.sql").to_string(),
        },
        Migration {
            version: 2,
            name: "knowledge_tables".to_string(),
            sql: include_str!("../../migrations/002_knowledge_tables.sql").to_string(),
        },
        Migration {
            version: 3,
            name: "memory_clustering".to_string(),
            sql: include_str!("../../migrations/003_memory_clustering.sql").to_string(),
        },
        Migration {
            version: 4,
            name: "sync_tracking".to_string(),
            sql: include_str!("../../migrations/004_sync_tracking.sql").to_string(),
        },
        Migration {
            version: 5,
            name: "indexes".to_string(),
            sql: include_str!("../../migrations/005_indexes.sql").to_string(),
        },
        Migration {
            version: 6,
            name: "subscription_tracking".to_string(),
            sql: include_str!("../../migrations/006_subscription_tracking.sql").to_string(),
        },
        Migration {
            version: 7,
            name: "user_profile".to_string(),
            sql: include_str!("../../migrations/007_user_profile.sql").to_string(),
        },
        Migration {
            version: 8,
            name: "hive_api_keys".to_string(),
            sql: include_str!("../../migrations/008_hive_api_keys.sql").to_string(),
        },
        Migration {
            version: 9,
            name: "consensus_profile_models".to_string(),
            sql: include_str!("../../migrations/009_consensus_profile_models.sql").to_string(),
        },
        Migration {
            version: 10,
            name: "remove_local_credit_tracking".to_string(),
            sql: include_str!("../../migrations/010_remove_local_credit_tracking.sql").to_string(),
        },
        Migration {
            version: 11,
            name: "add_licenses_table".to_string(),
            sql: include_str!("../../migrations/011_add_licenses_table.sql").to_string(),
        },
    ]
}

/// Apply a single migration
fn apply_migration(conn: &Connection, migration: &Migration) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    // Filter out metadata comments and empty lines
    let clean_sql = migration
        .sql
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("-- @") && !trimmed.is_empty()
        })
        .collect::<Vec<&str>>()
        .join("\n");

    // Split SQL into individual statements and execute each
    for statement in clean_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            tx.execute(trimmed, [])
                .with_context(|| format!("Failed to execute migration statement: {}", trimmed))?;
        }
    }

    // Record the migration
    tx.execute(
        "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
        params![
            migration.version,
            &migration.name,
            chrono::Utc::now().to_rfc3339()
        ],
    )?;

    tx.commit()?;
    Ok(())
}

/// Check if database needs migration
pub async fn needs_migration(conn: &Connection) -> Result<bool> {
    // Check if migrations table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
            [],
            |row| row.get::<_, i32>(0).map(|count| count > 0),
        )
        .unwrap_or(false);

    if !table_exists {
        return Ok(true);
    }

    // Get current version
    let current_version: Option<i32> = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .optional()?;

    let current_version = current_version.unwrap_or(0);

    // Check if there are pending migrations
    let migrations = load_migrations().await?;
    let max_version = migrations.iter().map(|m| m.version).max().unwrap_or(0);

    Ok(current_version < max_version)
}

/// Reset database to a specific migration version (for rollback)
pub async fn reset_to_version(conn: &Connection, target_version: i32) -> Result<()> {
    info!("Resetting database to version {}", target_version);

    let tx = conn.unchecked_transaction()?;

    // Get migrations to rollback
    let versions_to_remove: Vec<i32> = tx
        .prepare("SELECT version FROM schema_migrations WHERE version > ?1 ORDER BY version DESC")?
        .query_map(params![target_version], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    if versions_to_remove.is_empty() {
        info!("No migrations to rollback");
        return Ok(());
    }

    // For now, we don't support actual rollback SQL
    // In production, each migration would have a rollback script
    return Err(anyhow::anyhow!(
        "Rollback not supported. Please restore from backup or recreate database."
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[tokio::test]
    async fn test_migration_tracking() -> Result<()> {
        let mut conn = Connection::open_in_memory()?;

        // Run migrations on empty database
        run_migrations(&conn).await?;

        // Check migrations were tracked
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })?;
        assert!(count > 0);

        // Run again - should be idempotent
        run_migrations(&conn).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_needs_migration() -> Result<()> {
        let conn = Connection::open_in_memory()?;

        // Fresh database should need migration
        assert!(needs_migration(&conn).await?);

        // After running migrations, should not need migration
        run_migrations(&conn).await?;
        assert!(!needs_migration(&conn).await?);

        Ok(())
    }
}
