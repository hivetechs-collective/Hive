//! Database schema definitions and migration system
//!
//! This module contains the complete database schema that matches the TypeScript
//! implementation 100%, including all tables for models, profiles, conversations,
//! rankings, and analytics.

use crate::core::{HiveError, Result};
use rusqlite::Connection;

/// Database schema version for migration tracking
pub const SCHEMA_VERSION: i32 = 1;

/// Initialize complete database schema
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Create schema version table
    create_schema_version_table(conn)?;

    // Create all core tables
    create_openrouter_models_table(conn)?;
    create_model_rankings_table(conn)?;
    create_consensus_profiles_table(conn)?;
    create_conversations_table(conn)?;
    create_conversation_messages_table(conn)?;
    create_conversation_memory_table(conn)?;
    create_analytics_events_table(conn)?;
    create_user_profiles_table(conn)?;
    create_sync_queue_table(conn)?;
    create_cloudflare_sync_table(conn)?;
    create_thematic_clusters_table(conn)?;
    create_context_continuity_table(conn)?;
    create_performance_metrics_table(conn)?;
    create_cost_tracking_table(conn)?;
    create_configuration_table(conn)?;
    create_security_trust_table(conn)?;

    // Create indexes for performance
    create_indexes(conn)?;

    // Insert default data
    insert_default_data(conn)?;

    Ok(())
}

/// Check if schema needs migration
pub fn check_schema_version(conn: &Connection) -> Result<bool> {
    let current_version: i32 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(current_version < SCHEMA_VERSION)
}

/// Run database migrations
pub fn migrate_schema(conn: &Connection) -> Result<()> {
    let current_version: i32 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < SCHEMA_VERSION {
        // Run migrations from current version to latest
        for version in (current_version + 1)..=SCHEMA_VERSION {
            match version {
                1 => {
                    // Initial schema creation is handled by initialize_schema
                    update_schema_version(conn, 1)?;
                }
                _ => {
                    return Err(HiveError::Migration {
                        message: format!("Unknown migration version: {}", version),
                    });
                }
            }
        }
    }

    Ok(())
}

/// Create schema version tracking table
fn create_schema_version_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version INTEGER NOT NULL,
            migrated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    Ok(())
}

/// Update schema version
fn update_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        [version],
    )?;

    Ok(())
}

/// Create OpenRouter models table
fn create_openrouter_models_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS openrouter_models (
            internal_id INTEGER PRIMARY KEY AUTOINCREMENT,
            openrouter_id TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            provider_name TEXT NOT NULL,
            description TEXT,
            context_window INTEGER,
            pricing_input REAL NOT NULL DEFAULT 0.0,
            pricing_output REAL NOT NULL DEFAULT 0.0,
            pricing_image REAL,
            pricing_request REAL,
            architecture_modality TEXT,
            architecture_tokenizer TEXT,
            architecture_instruct_type TEXT,
            top_provider_max_completion_tokens INTEGER,
            top_provider_is_moderated BOOLEAN,
            per_request_limits_prompt_tokens TEXT,
            per_request_limits_completion_tokens TEXT,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            last_updated TEXT NOT NULL DEFAULT (datetime('now')),
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    Ok(())
}

/// Create model rankings table
fn create_model_rankings_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS model_rankings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            model_internal_id INTEGER NOT NULL,
            ranking_source TEXT NOT NULL,
            rank_position INTEGER NOT NULL,
            score REAL,
            category TEXT,
            last_updated TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (model_internal_id) REFERENCES openrouter_models (internal_id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}

/// Create consensus profiles table
fn create_consensus_profiles_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS consensus_profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            user_id TEXT,
            generator_model_id INTEGER NOT NULL,
            generator_temperature REAL NOT NULL DEFAULT 0.7,
            generator_max_tokens INTEGER,
            refiner_model_id INTEGER NOT NULL,
            refiner_temperature REAL NOT NULL DEFAULT 0.5,
            refiner_max_tokens INTEGER,
            validator_model_id INTEGER NOT NULL,
            validator_temperature REAL NOT NULL DEFAULT 0.3,
            validator_max_tokens INTEGER,
            curator_model_id INTEGER NOT NULL,
            curator_temperature REAL NOT NULL DEFAULT 0.6,
            curator_max_tokens INTEGER,
            is_default BOOLEAN NOT NULL DEFAULT 0,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (generator_model_id) REFERENCES openrouter_models (internal_id),
            FOREIGN KEY (refiner_model_id) REFERENCES openrouter_models (internal_id),
            FOREIGN KEY (validator_model_id) REFERENCES openrouter_models (internal_id),
            FOREIGN KEY (curator_model_id) REFERENCES openrouter_models (internal_id)
        )",
        [],
    )?;

    Ok(())
}

/// Create conversations table
fn create_conversations_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS conversations (
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
            tags TEXT, -- JSON array of tags
            metadata TEXT, -- JSON metadata
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (profile_id) REFERENCES consensus_profiles (id)
        )",
        [],
    )?;

    Ok(())
}

/// Create conversation messages table
fn create_conversation_messages_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS conversation_messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL, -- 'user', 'assistant', 'system'
            content TEXT NOT NULL,
            stage TEXT, -- 'generator', 'refiner', 'validator', 'curator', 'final'
            model_id INTEGER,
            model_name TEXT,
            temperature REAL,
            cost REAL DEFAULT 0.0,
            tokens_input INTEGER DEFAULT 0,
            tokens_output INTEGER DEFAULT 0,
            latency_ms INTEGER,
            quality_score REAL,
            sequence_number INTEGER NOT NULL,
            parent_message_id TEXT,
            metadata TEXT, -- JSON metadata
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (conversation_id) REFERENCES conversations (id) ON DELETE CASCADE,
            FOREIGN KEY (model_id) REFERENCES openrouter_models (internal_id),
            FOREIGN KEY (parent_message_id) REFERENCES conversation_messages (id)
        )",
        [],
    )?;

    Ok(())
}

/// Create conversation memory table for thematic clustering
fn create_conversation_memory_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS conversation_memory (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conversation_id TEXT NOT NULL,
            memory_type TEXT NOT NULL, -- 'summary', 'key_points', 'decisions', 'context'
            content TEXT NOT NULL,
            importance_score REAL DEFAULT 0.5,
            embedding_vector TEXT, -- JSON array for semantic search
            cluster_id INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (conversation_id) REFERENCES conversations (id) ON DELETE CASCADE,
            FOREIGN KEY (cluster_id) REFERENCES thematic_clusters (id)
        )",
        [],
    )?;

    Ok(())
}

/// Create analytics events table
fn create_analytics_events_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS analytics_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_type TEXT NOT NULL,
            event_name TEXT NOT NULL,
            user_id TEXT,
            conversation_id TEXT,
            profile_id INTEGER,
            model_id INTEGER,
            properties TEXT, -- JSON properties
            metrics TEXT, -- JSON metrics
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            session_id TEXT,
            FOREIGN KEY (conversation_id) REFERENCES conversations (id),
            FOREIGN KEY (profile_id) REFERENCES consensus_profiles (id),
            FOREIGN KEY (model_id) REFERENCES openrouter_models (internal_id)
        )",
        [],
    )?;

    Ok(())
}

/// Create user profiles table
fn create_user_profiles_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_profiles (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE,
            license_key TEXT,
            license_tier TEXT NOT NULL DEFAULT 'free',
            daily_limit INTEGER NOT NULL DEFAULT 10,
            daily_usage INTEGER NOT NULL DEFAULT 0,
            usage_reset_date TEXT,
            features TEXT, -- JSON array of enabled features
            preferences TEXT, -- JSON user preferences
            is_active BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    Ok(())
}

/// Create sync queue table for Cloudflare D1 synchronization
fn create_sync_queue_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            table_name TEXT NOT NULL,
            record_id TEXT NOT NULL,
            operation TEXT NOT NULL, -- 'INSERT', 'UPDATE', 'DELETE'
            data TEXT, -- JSON data to sync
            priority INTEGER DEFAULT 0,
            retry_count INTEGER DEFAULT 0,
            max_retries INTEGER DEFAULT 3,
            error_message TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            synced_at TEXT,
            failed_at TEXT
        )",
        [],
    )?;

    Ok(())
}

/// Create Cloudflare sync status table
fn create_cloudflare_sync_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cloudflare_sync_status (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            last_sync_at TEXT,
            last_successful_sync_at TEXT,
            sync_errors INTEGER DEFAULT 0,
            total_records_synced INTEGER DEFAULT 0,
            sync_status TEXT DEFAULT 'idle', -- 'idle', 'syncing', 'error'
            error_message TEXT,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    Ok(())
}

/// Create thematic clusters table
fn create_thematic_clusters_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS thematic_clusters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            keywords TEXT, -- JSON array of keywords
            centroid_vector TEXT, -- JSON array for cluster centroid
            conversation_count INTEGER DEFAULT 0,
            last_activity TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    Ok(())
}

/// Create context continuity table
fn create_context_continuity_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS context_continuity (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conversation_id TEXT NOT NULL,
            context_window_start INTEGER NOT NULL,
            context_window_end INTEGER NOT NULL,
            summary TEXT NOT NULL,
            key_entities TEXT, -- JSON array of key entities
            importance_score REAL DEFAULT 0.5,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (conversation_id) REFERENCES conversations (id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}

/// Create performance metrics table
fn create_performance_metrics_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS performance_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            metric_type TEXT NOT NULL, -- 'response_time', 'token_throughput', 'cost_efficiency', etc.
            metric_name TEXT NOT NULL,
            metric_value REAL NOT NULL,
            model_id INTEGER,
            profile_id INTEGER,
            conversation_id TEXT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            metadata TEXT, -- JSON metadata
            FOREIGN KEY (model_id) REFERENCES openrouter_models (internal_id),
            FOREIGN KEY (profile_id) REFERENCES consensus_profiles (id),
            FOREIGN KEY (conversation_id) REFERENCES conversations (id)
        )",
        [],
    )?;

    Ok(())
}

/// Create cost tracking table
fn create_cost_tracking_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cost_tracking (
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
            billing_period TEXT, -- 'daily', 'monthly', 'yearly'
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (conversation_id) REFERENCES conversations (id),
            FOREIGN KEY (model_id) REFERENCES openrouter_models (internal_id)
        )",
        [],
    )?;

    Ok(())
}

/// Create configuration table
fn create_configuration_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS configuration (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            type TEXT NOT NULL DEFAULT 'string', -- 'string', 'number', 'boolean', 'json'
            description TEXT,
            is_encrypted BOOLEAN DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    Ok(())
}

/// Create security trust table for file access
fn create_security_trust_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS security_trust (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            trust_level TEXT NOT NULL, -- 'trusted', 'untrusted', 'restricted'
            granted_at TEXT NOT NULL DEFAULT (datetime('now')),
            granted_by TEXT, -- user ID or system
            expires_at TEXT,
            is_recursive BOOLEAN DEFAULT 0,
            permissions TEXT, -- JSON array of permissions
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    Ok(())
}

/// Create database indexes for performance
fn create_indexes(conn: &Connection) -> Result<()> {
    // OpenRouter models indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_openrouter_models_provider ON openrouter_models (provider_name)", [])?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_openrouter_models_active ON openrouter_models (is_active)",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_openrouter_models_pricing ON openrouter_models (pricing_input, pricing_output)", [])?;

    // Model rankings indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_model_rankings_model ON model_rankings (model_internal_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_model_rankings_source ON model_rankings (ranking_source)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_model_rankings_position ON model_rankings (rank_position)",
        [],
    )?;

    // Consensus profiles indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_consensus_profiles_user ON consensus_profiles (user_id)",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_consensus_profiles_default ON consensus_profiles (is_default)", [])?;

    // Conversations indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_conversations_user ON conversations (user_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_conversations_profile ON conversations (profile_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_conversations_created ON conversations (created_at)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_conversations_archived ON conversations (is_archived)",
        [],
    )?;

    // Conversation messages indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_messages_conversation ON conversation_messages (conversation_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_messages_sequence ON conversation_messages (conversation_id, sequence_number)", [])?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_model ON conversation_messages (model_id)",
        [],
    )?;

    // Analytics events indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_analytics_type ON analytics_events (event_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_analytics_user ON analytics_events (user_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_analytics_timestamp ON analytics_events (timestamp)",
        [],
    )?;

    // Memory and clustering indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_conversation ON conversation_memory (conversation_id)", [])?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memory_cluster ON conversation_memory (cluster_id)",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_importance ON conversation_memory (importance_score)", [])?;

    // Performance metrics indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_performance_type ON performance_metrics (metric_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_performance_model ON performance_metrics (model_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_performance_timestamp ON performance_metrics (timestamp)",
        [],
    )?;

    // Cost tracking indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cost_user ON cost_tracking (user_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cost_model ON cost_tracking (model_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cost_created ON cost_tracking (created_at)",
        [],
    )?;

    // Sync queue indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_sync_queue_priority ON sync_queue (priority DESC, created_at)", [])?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sync_queue_table ON sync_queue (table_name)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sync_queue_synced ON sync_queue (synced_at)",
        [],
    )?;

    Ok(())
}

/// Insert default configuration data
fn insert_default_data(conn: &Connection) -> Result<()> {
    // Insert default configuration values
    let default_configs = vec![
        (
            "app_version",
            "2.0.0",
            "string",
            "Current application version",
        ),
        (
            "schema_version",
            "1",
            "number",
            "Current database schema version",
        ),
        (
            "openrouter_api_key",
            "",
            "string",
            "OpenRouter API key for model access",
        ),
        ("license_key", "", "string", "Hive AI license key"),
        (
            "max_conversation_length",
            "50",
            "number",
            "Maximum messages per conversation",
        ),
        (
            "default_temperature_generator",
            "0.7",
            "number",
            "Default temperature for generator stage",
        ),
        (
            "default_temperature_refiner",
            "0.5",
            "number",
            "Default temperature for refiner stage",
        ),
        (
            "default_temperature_validator",
            "0.3",
            "number",
            "Default temperature for validator stage",
        ),
        (
            "default_temperature_curator",
            "0.6",
            "number",
            "Default temperature for curator stage",
        ),
        (
            "enable_analytics",
            "true",
            "boolean",
            "Enable analytics collection",
        ),
        (
            "enable_cloudflare_sync",
            "false",
            "boolean",
            "Enable Cloudflare D1 synchronization",
        ),
        (
            "auto_create_clusters",
            "true",
            "boolean",
            "Automatically create thematic clusters",
        ),
        (
            "performance_monitoring",
            "true",
            "boolean",
            "Enable performance monitoring",
        ),
        ("cost_tracking", "true", "boolean", "Enable cost tracking"),
        (
            "security_trust_mode",
            "strict",
            "string",
            "Security trust mode: strict, moderate, permissive",
        ),
    ];

    for (key, value, type_str, description) in default_configs {
        conn.execute(
            "INSERT OR IGNORE INTO configuration (key, value, type, description) VALUES (?1, ?2, ?3, ?4)",
            [key, value, type_str, description],
        )?;
    }

    // Insert initial schema version
    conn.execute(
        "INSERT OR IGNORE INTO schema_version (version) VALUES (?1)",
        [SCHEMA_VERSION],
    )?;

    // Insert initial Cloudflare sync status
    conn.execute(
        "INSERT OR IGNORE INTO cloudflare_sync_status (id, sync_status) VALUES (1, 'idle')",
        [],
    )?;

    Ok(())
}

/// Health check for database schema
pub fn health_check(conn: &Connection) -> Result<Vec<String>> {
    let mut issues = Vec::new();

    // Check if all required tables exist
    let required_tables = vec![
        "schema_version",
        "openrouter_models",
        "model_rankings",
        "consensus_profiles",
        "conversations",
        "conversation_messages",
        "conversation_memory",
        "analytics_events",
        "user_profiles",
        "sync_queue",
        "cloudflare_sync_status",
        "thematic_clusters",
        "context_continuity",
        "performance_metrics",
        "cost_tracking",
        "configuration",
        "security_trust",
    ];

    for table in required_tables {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !exists {
            issues.push(format!("Missing table: {}", table));
        }
    }

    // Check schema version
    let current_version: i32 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < SCHEMA_VERSION {
        issues.push(format!(
            "Schema outdated: current {}, expected {}",
            current_version, SCHEMA_VERSION
        ));
    }

    // Check for foreign key issues
    let fk_errors: i32 = conn
        .query_row("PRAGMA foreign_key_check", [], |row| row.get(0))
        .unwrap_or(0);
    if fk_errors > 0 {
        issues.push(format!("Foreign key constraint violations: {}", fk_errors));
    }

    Ok(issues)
}

/// Repair database schema issues
pub fn repair_schema(conn: &Connection) -> Result<Vec<String>> {
    let mut repairs = Vec::new();

    // Check and repair missing tables
    let issues = health_check(conn)?;

    for issue in issues {
        if issue.starts_with("Missing table:") {
            let table_name = issue.replace("Missing table: ", "");
            match table_name.as_str() {
                "openrouter_models" => {
                    create_openrouter_models_table(conn)?;
                    repairs.push(format!("Recreated table: {}", table_name));
                }
                "model_rankings" => {
                    create_model_rankings_table(conn)?;
                    repairs.push(format!("Recreated table: {}", table_name));
                }
                "consensus_profiles" => {
                    create_consensus_profiles_table(conn)?;
                    repairs.push(format!("Recreated table: {}", table_name));
                }
                // Add more table repairs as needed
                _ => {
                    repairs.push(format!("Cannot repair table: {}", table_name));
                }
            }
        } else if issue.starts_with("Schema outdated:") {
            migrate_schema(conn)?;
            repairs.push("Schema migrated to latest version".to_string());
        }
    }

    // Recreate indexes if needed
    create_indexes(conn)?;
    repairs.push("Indexes recreated".to_string());

    Ok(repairs)
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_schema_initialization() {
        let conn = Connection::open_in_memory().unwrap();

        assert!(initialize_schema(&conn).is_ok());

        // Verify tables exist
        let table_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(table_count >= 15); // Should have at least 15 tables
    }

    #[test]
    fn test_health_check() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        let issues = health_check(&conn).unwrap();
        assert!(issues.is_empty());
    }
}
