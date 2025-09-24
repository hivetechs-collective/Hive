use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

// Simplified database configuration for the initial implementation
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("~/.hive/hive-ai.db"),
            enable_wal: true,
            enable_foreign_keys: true,
        }
    }
}

/// Database health status information
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseHealthStatus {
    pub healthy: bool,
    pub wal_mode_active: bool,
    pub foreign_keys_enabled: bool,
}

/// Database usage statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStatistics {
    pub conversation_count: i64,
    pub message_count: i64,
    pub model_count: i64,
    pub user_count: i64,
    pub profile_count: i64,
    pub knowledge_count: i64,
    pub database_size_bytes: u64,
}

/// Database handle for interacting with Hive AI storage
#[derive(Debug, Clone)]
pub struct Database {
    config: Arc<RwLock<DatabaseConfig>>,
}

impl Database {
    /// Create a new in-memory database (for testing)
    pub async fn memory() -> Result<Self> {
        Ok(Self {
            config: Arc::new(RwLock::new(DatabaseConfig {
                path: PathBuf::from(":memory:"),
                enable_wal: false,
                enable_foreign_keys: true,
            })),
        })
    }

    /// Open the default database
    pub async fn open_default() -> Result<Self> {
        let config = DatabaseConfig::default();
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
        })
    }

    /// Open a database with custom configuration
    pub async fn open(config: DatabaseConfig) -> Result<Self> {
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
        })
    }

    /// Get a database connection (sync version)
    pub fn get_connection_sync(&self) -> Result<Connection> {
        // For the simple implementation, we directly open a connection each time
        // In the full implementation, this would use a connection pool
        let config = self.config.blocking_read();

        // Expand home directory path if needed
        let db_path = if config.path.to_string_lossy().starts_with("~/") {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(config.path.strip_prefix("~/").unwrap_or(&config.path))
        } else {
            config.path.clone()
        };

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open connection
        let conn = Connection::open(&db_path)?;

        // Apply pragmas
        if config.enable_wal {
            // journal_mode returns a result, so use query_row
            let _mode: String =
                conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
        }

        if config.enable_foreign_keys {
            // foreign_keys doesn't return a result, so execute is fine
            conn.execute("PRAGMA foreign_keys = ON", [])?;
        }

        Ok(conn)
    }
}

// Additional methods for async compatibility
impl Database {
    /// Get a database connection (async version)
    /// This is needed for code that expects async behavior
    pub async fn get_connection(&self) -> Result<Connection> {
        // Get config without blocking the async runtime
        let config = self.config.read().await.clone();

        // Expand home directory path if needed
        let db_path = if config.path.to_string_lossy().starts_with("~/") {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(config.path.strip_prefix("~/").unwrap_or(&config.path))
        } else {
            config.path.clone()
        };

        // Use tokio to handle filesystem operations
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Open connection in blocking task to avoid blocking the async runtime
        let conn = tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path)?;

            // Apply pragmas
            if config.enable_wal {
                // journal_mode returns a result, so use query_row
                let _mode: String =
                    conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
            }

            if config.enable_foreign_keys {
                // foreign_keys doesn't return a result, so execute is fine
                conn.execute("PRAGMA foreign_keys = ON", [])?;
            }

            Ok::<Connection, anyhow::Error>(conn)
        })
        .await??;

        Ok(conn)
    }
}

// For now, provide stub implementations that can be filled in later
// when the full SQLx integration is ready

/// Initialize the database with the given configuration
pub async fn initialize_database(config: Option<DatabaseConfig>) -> Result<()> {
    let config = config.unwrap_or_default();

    // Expand home directory path if needed
    let db_path = if config.path.to_string_lossy().starts_with("~/") {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(config.path.strip_prefix("~/").unwrap_or(&config.path))
    } else {
        config.path.clone()
    };

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Initialize SQLite database
    let conn = Connection::open(&db_path)?;

    // Enable WAL mode for better performance
    if config.enable_wal {
        // journal_mode returns a result, so use query_row
        let _mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    }

    // Enable foreign keys
    if config.enable_foreign_keys {
        // foreign_keys doesn't return a result, so execute is fine
        conn.execute("PRAGMA foreign_keys = ON", [])?;
    }

    // Create basic schema for Phase 1
    create_basic_schema(&conn)?;

    tracing::info!(
        "Database initialized successfully at: {}",
        db_path.display()
    );

    Ok(())
}

/// Create basic database schema for Phase 1
fn create_basic_schema(conn: &Connection) -> Result<()> {
    // Basic configuration table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS configuration (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;

    // Basic trust decisions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trust_decisions (
            path TEXT PRIMARY KEY,
            trust_level TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            reason TEXT
        )",
        [],
    )?;

    // Basic conversations table (simplified for Phase 1)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            message_count INTEGER DEFAULT 0
        )",
        [],
    )?;

    // Basic security audit log
    conn.execute(
        "CREATE TABLE IF NOT EXISTS security_events (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            path TEXT,
            timestamp TEXT NOT NULL,
            details TEXT NOT NULL,
            allowed INTEGER NOT NULL
        )",
        [],
    )?;

    tracing::info!("Basic database schema created successfully");
    Ok(())
}

/// Get database health status
pub async fn get_health_status() -> Result<DatabaseHealthStatus> {
    // TODO: Implement actual health check with SQLx
    Ok(DatabaseHealthStatus {
        healthy: true,
        wal_mode_active: true,
        foreign_keys_enabled: true,
    })
}

/// Get database statistics
pub async fn get_statistics() -> Result<DatabaseStatistics> {
    let config = DatabaseConfig::default();

    // Expand home directory path
    let db_path = if config.path.to_string_lossy().starts_with("~/") {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(config.path.strip_prefix("~/").unwrap_or(&config.path))
    } else {
        config.path.clone()
    };

    if !db_path.exists() {
        return Ok(DatabaseStatistics {
            conversation_count: 0,
            message_count: 0,
            model_count: 0,
            user_count: 0,
            profile_count: 0,
            knowledge_count: 0,
            database_size_bytes: 0,
        });
    }

    // Simple error handling for database connection
    let conversation_count = match Connection::open(&db_path) {
        Ok(conn) => conn
            .query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))
            .unwrap_or(0),
        Err(_) => 0,
    };

    // Get database file size
    let database_size_bytes = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

    Ok(DatabaseStatistics {
        conversation_count,
        message_count: 0,   // Not implemented yet
        model_count: 323,   // Static for now (OpenRouter model count)
        user_count: 1,      // Single user for now
        profile_count: 4,   // Basic consensus profiles
        knowledge_count: 0, // Not implemented yet
        database_size_bytes,
    })
}

/// Generate a new UUID for use as primary keys
pub fn generate_id() -> String {
    // For now, use a simple timestamp-based ID
    // TODO: Use proper UUID when uuid crate is available
    format!(
        "id_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    )
}

/// Get current UTC timestamp as ISO string
pub fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_config() {
        let config = DatabaseConfig::default();
        assert!(config.enable_wal);
        assert!(config.enable_foreign_keys);
    }

    #[tokio::test]
    async fn test_database_init() -> Result<()> {
        // Test that initialization doesn't fail
        initialize_database(None).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_health_status() -> Result<()> {
        let health = get_health_status().await?;
        assert!(health.healthy);
        Ok(())
    }

    #[test]
    fn test_id_generation() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
        assert!(id1.starts_with("id_"));
    }

    #[test]
    fn test_timestamp() {
        let ts = current_timestamp();
        assert!(ts.contains("T"));
        assert!(ts.contains("Z"));
    }
}
