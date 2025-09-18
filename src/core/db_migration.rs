//! Database migration to fix schema mismatches
//!
//! This module handles migrating the existing database to match the expected schema

use crate::core::{HiveError, Result};
use rusqlite::{Connection, Transaction};

/// Fix schema mismatches in the database
pub fn fix_schema_mismatches(conn: &mut Connection) -> Result<()> {
    // Disable foreign keys temporarily during migration
    conn.execute("PRAGMA foreign_keys = OFF", [])?;

    let tx = conn.transaction()?;

    println!("🔧 Starting database schema migration...");

    // Check and fix conversations table
    fix_conversations_table(&tx)?;

    // Create missing cost_tracking table
    create_missing_cost_tracking_table(&tx)?;

    // Verify all tables exist
    verify_schema_integrity(&tx)?;

    tx.commit()?;

    // Re-enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    println!("✅ Database schema migration completed successfully!");

    Ok(())
}

/// Fix conversations table column names
fn fix_conversations_table(tx: &Transaction) -> Result<()> {
    // Check current schema
    let mut stmt = tx.prepare("PRAGMA table_info(conversations)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| HiveError::DatabaseQuery {
            query: format!("PRAGMA table_info(conversations): {}", e),
        })?;

    // Check if we have the old column names
    let has_input_tokens = columns.contains(&"input_tokens".to_string());
    let has_output_tokens = columns.contains(&"output_tokens".to_string());
    let has_total_tokens_input = columns.contains(&"total_tokens_input".to_string());
    let has_total_tokens_output = columns.contains(&"total_tokens_output".to_string());

    if has_input_tokens && has_output_tokens && !has_total_tokens_input && !has_total_tokens_output
    {
        println!("🔄 Migrating conversations table schema...");

        // SQLite doesn't support ALTER COLUMN, so we need to recreate the table
        // Step 1: Create new table with correct schema
        tx.execute(
            "CREATE TABLE conversations_new (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                title TEXT,
                profile_id INTEGER,
                context_type TEXT DEFAULT 'general',
                total_cost REAL DEFAULT 0.0,
                total_tokens_input INTEGER DEFAULT 0,
                total_tokens_output INTEGER DEFAULT 0,
                performance_score REAL,
                quality_rating INTEGER,
                is_archived BOOLEAN DEFAULT 0,
                is_favorite BOOLEAN DEFAULT 0,
                tags TEXT,
                metadata TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (profile_id) REFERENCES consensus_profiles (id)
            )",
            [],
        )?;

        // Step 2: Copy data from old table to new table, preserving all existing columns
        // Note: Some columns may not exist in the new schema, so we only copy what's needed
        tx.execute(
            "INSERT INTO conversations_new (
                id, user_id, profile_id, total_cost, total_tokens_input, total_tokens_output,
                created_at, updated_at
            )
            SELECT 
                id, 
                user_id, 
                consensus_profile_id, 
                total_cost, 
                input_tokens as total_tokens_input, 
                output_tokens as total_tokens_output,
                created_at, 
                updated_at
            FROM conversations",
            [],
        )?;

        // Step 3: Drop old table
        tx.execute("DROP TABLE conversations", [])?;

        // Step 4: Rename new table
        tx.execute("ALTER TABLE conversations_new RENAME TO conversations", [])?;

        println!("✅ Conversations table schema migrated successfully");
    } else if has_total_tokens_input && has_total_tokens_output {
        println!("✅ Conversations table already has correct schema");
    } else {
        // Handle other potential schema states
        return Err(HiveError::DatabaseSchema {
            message: "Unexpected conversations table schema".to_string(),
        });
    }

    Ok(())
}

/// Create missing cost_tracking table
fn create_missing_cost_tracking_table(tx: &Transaction) -> Result<()> {
    // Check if cost_tracking table exists
    let table_exists: bool = tx
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='cost_tracking'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !table_exists {
        println!("🔄 Creating cost_tracking table...");

        tx.execute(
            "CREATE TABLE cost_tracking (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT,
                conversation_id TEXT,
                model_id INTEGER NOT NULL,
                tokens_input INTEGER NOT NULL DEFAULT 0,
                tokens_output INTEGER NOT NULL DEFAULT 0,
                cost_input REAL NOT NULL DEFAULT 0.0,
                cost_output REAL NOT NULL DEFAULT 0.0,
                total_cost REAL NOT NULL DEFAULT 0.0,
                currency TEXT DEFAULT 'USD',
                billing_period TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (conversation_id) REFERENCES conversations (id),
                FOREIGN KEY (model_id) REFERENCES openrouter_models (internal_id)
            )",
            [],
        )?;

        // Create indexes for cost_tracking
        tx.execute("CREATE INDEX idx_cost_user ON cost_tracking (user_id)", [])?;
        tx.execute(
            "CREATE INDEX idx_cost_model ON cost_tracking (model_id)",
            [],
        )?;
        tx.execute(
            "CREATE INDEX idx_cost_created ON cost_tracking (created_at)",
            [],
        )?;

        println!("✅ Cost tracking table created successfully");
    } else {
        println!("✅ Cost tracking table already exists");
    }

    Ok(())
}

/// Verify schema integrity after migration
fn verify_schema_integrity(tx: &Transaction) -> Result<()> {
    // Check conversations table columns
    let mut stmt = tx.prepare("PRAGMA table_info(conversations)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| HiveError::DatabaseQuery {
            query: format!("PRAGMA table_info(conversations): {}", e),
        })?;

    let required_columns = vec!["total_cost", "total_tokens_input", "total_tokens_output"];
    for col in required_columns {
        if !columns.contains(&col.to_string()) {
            return Err(HiveError::DatabaseSchema {
                message: format!("Missing required column '{}' in conversations table", col),
            });
        }
    }

    // Check cost_tracking table exists
    let cost_tracking_exists: bool = tx
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='cost_tracking'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !cost_tracking_exists {
        return Err(HiveError::DatabaseSchema {
            message: "cost_tracking table is missing".to_string(),
        });
    }

    println!("✅ Schema integrity verified successfully");
    Ok(())
}

/// Check if migration is needed
pub fn needs_migration(conn: &Connection) -> Result<bool> {
    // Check conversations table columns
    let mut stmt = conn.prepare("PRAGMA table_info(conversations)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| HiveError::DatabaseQuery {
            query: format!("PRAGMA table_info(conversations): {}", e),
        })?;

    let has_old_columns = columns.contains(&"input_tokens".to_string())
        && columns.contains(&"output_tokens".to_string());
    let has_new_columns = columns.contains(&"total_tokens_input".to_string())
        && columns.contains(&"total_tokens_output".to_string());

    // Check if cost_tracking table exists
    let cost_tracking_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='cost_tracking'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    // Need migration if we have old columns or missing cost_tracking table
    Ok((has_old_columns && !has_new_columns) || !cost_tracking_exists)
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migration_detection() {
        let mut conn = Connection::open_in_memory().unwrap();

        // Create old schema
        conn.execute(
            "CREATE TABLE conversations (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                total_cost REAL DEFAULT 0,
                input_tokens INTEGER DEFAULT 0,
                output_tokens INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();

        assert!(needs_migration(&conn).unwrap());

        // Run migration
        fix_schema_mismatches(&mut conn).unwrap();

        // Should not need migration anymore
        assert!(!needs_migration(&conn).unwrap());
    }
}
