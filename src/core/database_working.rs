use anyhow::{Context, Result};
use chrono::Utc;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::OnceCell;
use tracing::info;
use uuid::Uuid;

/// Global database instance with connection pooling
static DATABASE: OnceCell<Arc<DatabaseManager>> = OnceCell::const_new();

/// Database configuration for optimal SQLite performance
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub cache_size: i32,
    pub synchronous: String,
    pub journal_mode: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("~/.hive/hive-ai.db"),
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            enable_wal: true,
            enable_foreign_keys: true,
            cache_size: -32000, // 32MB cache
            synchronous: "NORMAL".to_string(),
            journal_mode: "WAL".to_string(),
        }
    }
}

/// Main database manager with connection pooling and WAL mode
#[derive(Debug)]
pub struct DatabaseManager {
    pool: Pool<SqliteConnectionManager>,
    config: DatabaseConfig,
}

/// Type alias for pooled connection
pub type DbConnection = PooledConnection<SqliteConnectionManager>;

impl DatabaseManager {
    /// Initialize database with optimized settings for performance
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        // Expand home directory if needed
        let db_path = shellexpand::full(&config.path.to_string_lossy())
            .context("Failed to expand database path")?
            .to_string();
        let db_path = PathBuf::from(db_path);

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create database directory")?;
        }

        // Create connection manager
        let manager = SqliteConnectionManager::file(&db_path);

        // Build connection pool
        let pool = Pool::builder()
            .max_size(config.max_connections)
            .connection_timeout(config.connection_timeout)
            .idle_timeout(Some(config.idle_timeout))
            .test_on_check_out(true)
            .build(manager)
            .context("Failed to create connection pool")?;

        // Get a connection to apply initial pragmas
        let conn = pool.get().context("Failed to get initial connection")?;
        
        // Apply performance optimizations
        if config.enable_wal {
            let _mode: String = conn.query_row(
                &format!("PRAGMA journal_mode = {}", config.journal_mode),
                [],
                |row| row.get(0)
            )?;
        }
        
        conn.execute(&format!("PRAGMA cache_size = {}", config.cache_size), [])?;
        conn.execute("PRAGMA temp_store = MEMORY", [])?;
        conn.execute("PRAGMA mmap_size = 268435456", [])?; // 256MB mmap
        conn.execute(&format!("PRAGMA synchronous = {}", config.synchronous), [])?;
        
        if config.enable_foreign_keys {
            conn.execute("PRAGMA foreign_keys = ON", [])?;
        }

        // Drop the connection to return it to the pool
        drop(conn);

        info!(
            "Database initialized at {} with WAL mode and connection pooling",
            db_path.display()
        );

        let manager = Self { pool, config };
        
        // Create minimal schema
        manager.create_schema().await?;
        
        Ok(manager)
    }

    /// Get a connection from the pool
    pub fn get_connection(&self) -> Result<DbConnection> {
        self.pool
            .get()
            .context("Failed to get connection from pool")
    }

    /// Create minimal database schema
    async fn create_schema(&self) -> Result<()> {
        let conn = self.get_connection()?;
        
        // Create users table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE,
                license_key TEXT,
                tier TEXT DEFAULT 'FREE',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create conversations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                consensus_profile_id TEXT,
                total_cost REAL DEFAULT 0,
                input_tokens INTEGER DEFAULT 0,
                output_tokens INTEGER DEFAULT 0,
                start_time TEXT,
                end_time TEXT,
                success_rate TEXT DEFAULT '0',
                quality_score REAL DEFAULT 0.0,
                consensus_improvement INTEGER DEFAULT 0,
                confidence_level TEXT DEFAULT 'Unknown',
                success INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )",
            [],
        )?;

        // Create messages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                stage TEXT,
                model_used TEXT,
                timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            )",
            [],
        )?;

        // Create consensus_profiles table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS consensus_profiles (
                id TEXT PRIMARY KEY,
                profile_name TEXT NOT NULL UNIQUE,
                pipeline_profile_id TEXT,
                generator_model TEXT NOT NULL,
                refiner_model TEXT NOT NULL,
                validator_model TEXT NOT NULL,
                curator_model TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create knowledge_conversations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS knowledge_conversations (
                id TEXT PRIMARY KEY,
                conversation_id TEXT,
                question TEXT NOT NULL,
                final_answer TEXT NOT NULL,
                source_of_truth TEXT NOT NULL,
                conversation_context TEXT,
                profile_id TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                last_updated TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            )",
            [],
        )?;

        // Create activity_log table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS activity_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                conversation_id TEXT,
                user_id TEXT,
                stage TEXT,
                model_used TEXT,
                provider_name TEXT,
                cost REAL,
                duration_ms INTEGER,
                tokens_used INTEGER,
                error_message TEXT,
                metadata TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id),
                FOREIGN KEY (user_id) REFERENCES users(id)
            )",
            [],
        )?;

        // Create OpenRouter models table for API model tracking
        conn.execute(
            "CREATE TABLE IF NOT EXISTS openrouter_models (
                internal_id INTEGER PRIMARY KEY AUTOINCREMENT,
                openrouter_id TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                provider_id TEXT NOT NULL,
                provider_name TEXT NOT NULL
            )",
            [],
        )?;

        info!("Database schema created successfully");
        Ok(())
    }

    /// Execute a health check to ensure database is responding
    pub async fn health_check(&self) -> Result<DatabaseHealthStatus> {
        let start = Instant::now();
        
        let conn = self.get_connection()?;
        
        // Test basic connectivity
        let timestamp: String = conn.query_row(
            "SELECT datetime('now') as timestamp",
            [],
            |row| row.get(0),
        )?;

        let response_time = start.elapsed();

        // Check pool status
        let pool_state = self.pool.state();
        let pool_size = pool_state.connections;
        let idle_connections = pool_state.idle_connections;

        // Check if WAL mode is active
        let wal_mode: String = conn.query_row(
            "PRAGMA journal_mode",
            [],
            |row| row.get(0),
        )?;

        // Check foreign key enforcement
        let foreign_keys: i32 = conn.query_row(
            "PRAGMA foreign_keys",
            [],
            |row| row.get(0),
        )?;

        Ok(DatabaseHealthStatus {
            healthy: true,
            response_time,
            timestamp,
            pool_size,
            idle_connections,
            wal_mode_active: wal_mode.to_uppercase() == "WAL",
            foreign_keys_enabled: foreign_keys == 1,
        })
    }

    /// Get database statistics for monitoring
    pub async fn get_statistics(&self) -> Result<DatabaseStatistics> {
        let conn = self.get_connection()?;
        
        let stats_query = r#"
            SELECT 
                (SELECT COUNT(*) FROM conversations) as conversation_count,
                (SELECT COUNT(*) FROM messages) as message_count,
                (SELECT COUNT(*) FROM openrouter_models) as model_count,
                (SELECT COUNT(*) FROM users) as user_count,
                (SELECT COUNT(*) FROM consensus_profiles) as profile_count,
                (SELECT COUNT(*) FROM knowledge_conversations) as knowledge_count
        "#;

        let (conversation_count, message_count, model_count, user_count, profile_count, knowledge_count) = 
            conn.query_row(stats_query, [], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })?;

        // Get database file size
        let db_size = tokio::fs::metadata(&self.config.path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        let pool_state = self.pool.state();

        Ok(DatabaseStatistics {
            conversation_count,
            message_count,
            model_count,
            user_count,
            profile_count,
            knowledge_count,
            database_size_bytes: db_size,
            pool_size: pool_state.connections,
            idle_connections: pool_state.idle_connections,
        })
    }
}

/// Database health status information
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseHealthStatus {
    pub healthy: bool,
    pub response_time: Duration,
    pub timestamp: String,
    pub pool_size: u32,
    pub idle_connections: u32,
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
    pub pool_size: u32,
    pub idle_connections: u32,
}

/// Initialize the global database instance
pub async fn initialize_database(config: Option<DatabaseConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let db_manager = Arc::new(DatabaseManager::new(config).await?);
    
    DATABASE
        .set(db_manager)
        .map_err(|_| anyhow::anyhow!("Database already initialized"))?;

    info!("Global database instance initialized successfully");
    Ok(())
}

/// Get a reference to the global database instance
pub async fn get_database() -> Result<Arc<DatabaseManager>> {
    DATABASE
        .get()
        .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
        .map(Arc::clone)
}

/// Generate a new UUID for use as primary keys
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Get current UTC timestamp as ISO string
pub fn current_timestamp() -> String {
    Utc::now().to_rfc3339()
}

// ===== DATA MODELS (same as before) =====

/// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub license_key: Option<String>,
    pub tier: String,
    pub created_at: String,
    pub updated_at: String,
}

impl User {
    /// Create a new user
    pub async fn create(email: Option<String>, license_key: Option<String>) -> Result<Self> {
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        let user = User {
            id: generate_id(),
            email,
            license_key,
            tier: "FREE".to_string(),
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };

        conn.execute(
            "INSERT INTO users (id, email, license_key, tier, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &user.id,
                &user.email,
                &user.license_key,
                &user.tier,
                &user.created_at,
                &user.updated_at,
            ],
        )?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(id: &str) -> Result<Option<Self>> {
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        let user = conn
            .query_row(
                "SELECT id, email, license_key, tier, created_at, updated_at 
                 FROM users WHERE id = ?1",
                params![id],
                |row| {
                    Ok(User {
                        id: row.get(0)?,
                        email: row.get(1)?,
                        license_key: row.get(2)?,
                        tier: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .optional()?;

        Ok(user)
    }
}

/// Conversation model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub user_id: Option<String>,
    pub consensus_profile_id: Option<String>,
    pub total_cost: f64,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub success_rate: String,
    pub quality_score: f64,
    pub consensus_improvement: i32,
    pub confidence_level: String,
    pub success: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Conversation {
    /// Create a new conversation
    pub async fn create(
        user_id: Option<String>,
        consensus_profile_id: Option<String>,
    ) -> Result<Self> {
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        let conversation = Conversation {
            id: generate_id(),
            user_id,
            consensus_profile_id,
            total_cost: 0.0,
            input_tokens: 0,
            output_tokens: 0,
            start_time: Some(current_timestamp()),
            end_time: None,
            success_rate: "0".to_string(), // Will be calculated after completion
            quality_score: 0.0, // Will be calculated based on actual consensus
            consensus_improvement: 0, // Will be calculated from stage analysis
            confidence_level: "Unknown".to_string(), // Will be determined by validator
            success: false, // Will be set to true when consensus completes
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };

        conn.execute(
            "INSERT INTO conversations (
                id, user_id, consensus_profile_id, total_cost, input_tokens, output_tokens,
                start_time, end_time, success_rate, quality_score, consensus_improvement,
                confidence_level, success, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                &conversation.id,
                &conversation.user_id,
                &conversation.consensus_profile_id,
                &conversation.total_cost,
                &conversation.input_tokens,
                &conversation.output_tokens,
                &conversation.start_time,
                &conversation.end_time,
                &conversation.success_rate,
                &conversation.quality_score,
                &conversation.consensus_improvement,
                &conversation.confidence_level,
                &conversation.success,
                &conversation.created_at,
                &conversation.updated_at,
            ],
        )?;

        Ok(conversation)
    }
}

/// Message model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub stage: Option<String>,
    pub model_used: Option<String>,
    pub timestamp: String,
}

impl Message {
    /// Create a new message
    pub async fn create(
        conversation_id: String,
        role: String,
        content: String,
        stage: Option<String>,
        model_used: Option<String>,
    ) -> Result<Self> {
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        let message = Message {
            id: generate_id(),
            conversation_id,
            role,
            content,
            stage,
            model_used,
            timestamp: current_timestamp(),
        };

        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, stage, model_used, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &message.id,
                &message.conversation_id,
                &message.role,
                &message.content,
                &message.stage,
                &message.model_used,
                &message.timestamp,
            ],
        )?;

        Ok(message)
    }

    /// Find all messages for a conversation
    pub async fn find_by_conversation(conversation_id: &str) -> Result<Vec<Self>> {
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, role, content, stage, model_used, timestamp
             FROM messages 
             WHERE conversation_id = ?1
             ORDER BY timestamp ASC",
        )?;

        let messages = stmt
            .query_map(params![conversation_id], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    stage: row.get(4)?,
                    model_used: row.get(5)?,
                    timestamp: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }
}

/// Execute a transaction with automatic rollback on error
pub async fn execute_transaction<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&Transaction) -> Result<R>,
{
    let db = get_database().await?;
    let mut conn = db.get_connection()?;
    
    let tx = conn.transaction()?;
    match f(&tx) {
        Ok(result) => {
            tx.commit()?;
            Ok(result)
        }
        Err(e) => {
            // Transaction automatically rolls back on drop
            Err(e)
        }
    }
}