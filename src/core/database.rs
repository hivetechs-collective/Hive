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
    /// Create with default configuration
    pub async fn default() -> Result<Self> {
        Self::new(DatabaseConfig::default()).await
    }

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

        info!("Applying database pragmas...");
        // Apply performance optimizations
        if config.enable_wal {
            info!("Setting WAL mode...");
            let _mode: String = conn.query_row(
                &format!("PRAGMA journal_mode = {}", config.journal_mode),
                [],
                |row| row.get(0),
            )?;
            info!("WAL mode set successfully");
        }

        info!("Setting cache size...");
        conn.execute(&format!("PRAGMA cache_size = {}", config.cache_size), [])?;
        info!("Setting temp store...");
        conn.execute("PRAGMA temp_store = MEMORY", [])?;
        info!("Setting mmap size...");
        let _mmap_size: i64 =
            conn.query_row("PRAGMA mmap_size = 268435456", [], |row| row.get(0))?; // 256MB mmap
        info!("Setting synchronous mode...");
        conn.execute(&format!("PRAGMA synchronous = {}", config.synchronous), [])?;

        if config.enable_foreign_keys {
            info!("Enabling foreign keys...");
            conn.execute("PRAGMA foreign_keys = ON", [])?;
            info!("Foreign keys enabled");
        }

        // Drop the connection to return it to the pool
        drop(conn);

        info!(
            "Database initialized at {} with WAL mode and connection pooling",
            db_path.display()
        );

        let manager = Self { pool, config };

        // Run migrations
        info!("Starting database migrations...");
        manager.run_migrations().await?;
        info!("Database migrations completed");

        Ok(manager)
    }

    /// Get a connection from the pool
    pub fn get_connection(&self) -> Result<DbConnection> {
        self.pool
            .get()
            .context("Failed to get connection from pool")
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<()> {
        let conn = self.get_connection()?;

        // Create migrations table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            )",
            [],
        )?;

        // Get current version
        let current_version: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        info!("Current database version: {}", current_version);

        // Load and apply migrations
        let migrations_dir = PathBuf::from("migrations");
        if migrations_dir.exists() {
            let mut entries = tokio::fs::read_dir(&migrations_dir)
                .await
                .context("Failed to read migrations directory")?;

            let mut migrations = Vec::new();
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                    migrations.push(path);
                }
            }

            migrations.sort();

            for migration_path in migrations {
                let filename = migration_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                // Extract version from filename (e.g., "001_initial_schema.sql" -> 1)
                let version: i32 = filename
                    .split('_')
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);

                if version > current_version {
                    let sql = tokio::fs::read_to_string(&migration_path)
                        .await
                        .context("Failed to read migration file")?;

                    // Extract name from metadata comment
                    let name = sql
                        .lines()
                        .find(|l| l.starts_with("-- @ name:"))
                        .and_then(|l| l.strip_prefix("-- @ name:"))
                        .map(|s| s.trim())
                        .unwrap_or("unknown");

                    info!("Applying migration {}: {}", version, name);

                    // Filter out metadata comments and empty lines
                    let clean_sql = sql
                        .lines()
                        .filter(|line| {
                            let trimmed = line.trim();
                            !trimmed.starts_with("-- @") && !trimmed.is_empty()
                        })
                        .collect::<Vec<&str>>()
                        .join("\n");

                    // Apply migration in a transaction
                    let mut conn = self.get_connection()?;
                    let tx = conn.transaction()?;

                    // Split SQL into individual statements and execute each
                    for statement in clean_sql.split(';') {
                        let trimmed = statement.trim();
                        if !trimmed.is_empty() {
                            tx.execute(trimmed, [])?;
                        }
                    }
                    tx.execute(
                        "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
                        params![version, name, Utc::now().to_rfc3339()],
                    )?;
                    tx.commit()?;

                    info!("Migration {} applied successfully", version);
                }
            }
        }

        Ok(())
    }

    /// Execute a health check to ensure database is responding
    pub async fn health_check(&self) -> Result<DatabaseHealthStatus> {
        let start = Instant::now();

        let conn = self.get_connection()?;

        // Test basic connectivity
        let timestamp: String =
            conn.query_row("SELECT datetime('now') as timestamp", [], |row| row.get(0))?;

        let response_time = start.elapsed();

        // Check pool status
        let pool_state = self.pool.state();
        let pool_size = pool_state.connections;
        let idle_connections = pool_state.idle_connections;

        // Check if WAL mode is active
        let wal_mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;

        // Check foreign key enforcement
        let foreign_keys: i32 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;

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

        let (
            conversation_count,
            message_count,
            model_count,
            user_count,
            profile_count,
            knowledge_count,
        ) = conn.query_row(stats_query, [], |row| {
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

    /// Execute vacuum operation to optimize database
    pub async fn vacuum(&self) -> Result<()> {
        info!("Starting database vacuum operation");
        let start = Instant::now();

        let conn = self.get_connection()?;

        // Use incremental vacuum for WAL mode
        conn.execute("PRAGMA incremental_vacuum(1000)", [])?;

        let duration = start.elapsed();
        info!("Database vacuum completed in {:?}", duration);

        Ok(())
    }

    /// Optimize database by updating statistics and rebuilding indexes
    pub async fn optimize(&self) -> Result<()> {
        info!("Starting database optimization");
        let start = Instant::now();

        let conn = self.get_connection()?;

        // Analyze tables for query optimization
        conn.execute("ANALYZE", [])?;

        // Rebuild indexes if needed
        conn.execute("REINDEX", [])?;

        let duration = start.elapsed();
        info!("Database optimization completed in {:?}", duration);

        Ok(())
    }

    /// Calculate model cost based on usage and real pricing from database
    /// This matches the TypeScript calculateModelCost function exactly
    pub async fn calculate_model_cost(
        &self,
        model_openrouter_id: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> Result<f64> {
        let conn = self.get_connection()?;
        
        tracing::info!(
            "💰 Looking up model pricing for: {} (prompt: {}, completion: {})",
            model_openrouter_id,
            prompt_tokens,
            completion_tokens
        );

        // Look up model pricing by openrouter_id
        let model_data = conn
            .query_row(
                "SELECT internal_id, openrouter_id, pricing_input, pricing_output FROM openrouter_models WHERE openrouter_id = ?",
                [model_openrouter_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, f64>(3)?,
                    ))
                },
            )
            .optional()?;

        let (internal_id, openrouter_id, pricing_input, pricing_output) = match model_data {
            Some(data) => data,
            None => {
                return Err(anyhow::anyhow!(
                    "Model not found in database: {}. Please update model data.",
                    model_openrouter_id
                ));
            }
        };

        tracing::info!(
            "💰 Found model: {} (internal_id: {}, pricing_input: {:.10e}, pricing_output: {:.10e})",
            openrouter_id,
            internal_id,
            pricing_input,
            pricing_output
        );

        // Calculate cost exactly like TypeScript implementation
        // Pricing is stored per-token, so multiply by token count directly (not divide by 1000)
        let input_cost = prompt_tokens as f64 * pricing_input;
        let output_cost = completion_tokens as f64 * pricing_output;
        let total_cost = input_cost + output_cost;

        tracing::info!(
            "💰 Cost calculation details: {} prompt tokens * {:.10e} = ${:.10}",
            prompt_tokens,
            pricing_input,
            input_cost
        );
        tracing::info!(
            "💰 Cost calculation details: {} completion tokens * {:.10e} = ${:.10}",
            completion_tokens,
            pricing_output,
            output_cost
        );
        tracing::info!(
            "💰 Total cost: ${:.10} (input: ${:.10} + output: ${:.10})",
            total_cost,
            input_cost,
            output_cost
        );

        Ok(total_cost)
    }

    /// Store conversation with cost data
    pub async fn store_conversation_with_cost(
        &self,
        conversation_id: &str,
        user_id: Option<&str>,
        title: &str,
        total_cost: f64,
        total_tokens_input: u32,
        total_tokens_output: u32,
    ) -> Result<()> {
        let conn = self.get_connection()?;
        
        conn.execute(
            "INSERT OR REPLACE INTO conversations 
             (id, user_id, title, total_cost, total_tokens_input, total_tokens_output, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))",
            params![
                conversation_id,
                user_id,
                title,
                total_cost,
                total_tokens_input,
                total_tokens_output
            ],
        )?;

        tracing::info!(
            "💰 Stored conversation {} with total cost: ${:.6}",
            conversation_id,
            total_cost
        );

        Ok(())
    }

    /// Update conversation cost (for incremental updates during consensus stages)
    pub async fn update_conversation_cost(
        &self,
        conversation_id: &str,
        additional_cost: f64,
        additional_input_tokens: u32,
        additional_output_tokens: u32,
    ) -> Result<()> {
        let conn = self.get_connection()?;
        
        conn.execute(
            "UPDATE conversations 
             SET total_cost = COALESCE(total_cost, 0.0) + ?2,
                 total_tokens_input = COALESCE(total_tokens_input, 0) + ?3,
                 total_tokens_output = COALESCE(total_tokens_output, 0) + ?4,
                 updated_at = datetime('now')
             WHERE id = ?1",
            params![
                conversation_id,
                additional_cost,
                additional_input_tokens,
                additional_output_tokens
            ],
        )?;

        tracing::info!(
            "💰 Updated conversation {} cost by ${:.6} ({} input + {} output tokens)",
            conversation_id,
            additional_cost,
            additional_input_tokens,
            additional_output_tokens
        );

        Ok(())
    }

    /// Gracefully close all database connections
    pub async fn close(&self) {
        info!("Database connections will be closed when pool is dropped");
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

// ===== DATA MODELS =====

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

    /// Find user by email
    pub async fn find_by_email(email: &str) -> Result<Option<Self>> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let user = conn
            .query_row(
                "SELECT id, email, license_key, tier, created_at, updated_at
                 FROM users WHERE email = ?1",
                params![email],
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
    pub total_tokens_input: i32,
    pub total_tokens_output: i32,
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
            total_tokens_input: 0,
            total_tokens_output: 0,
            start_time: Some(current_timestamp()),
            end_time: None,
            success_rate: "0".to_string(), // Will be calculated after completion
            quality_score: 0.0,            // Will be calculated based on actual consensus
            consensus_improvement: 0,      // Will be calculated from stage analysis
            confidence_level: "Unknown".to_string(), // Will be determined by validator
            success: false,                // Will be set to true when consensus completes
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };

        conn.execute(
            "INSERT INTO conversations (
                id, user_id, consensus_profile_id, total_cost, total_tokens_input, total_tokens_output,
                start_time, end_time, success_rate, quality_score, consensus_improvement,
                confidence_level, success, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                &conversation.id,
                &conversation.user_id,
                &conversation.consensus_profile_id,
                &conversation.total_cost,
                &conversation.total_tokens_input,
                &conversation.total_tokens_output,
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

    /// Find conversation by ID
    pub async fn find_by_id(id: &str) -> Result<Option<Self>> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let conversation = conn
            .query_row(
                "SELECT id, user_id, consensus_profile_id, total_cost, total_tokens_input, total_tokens_output,
                        start_time, end_time, success_rate, quality_score, consensus_improvement,
                        confidence_level, success, created_at, updated_at
                 FROM conversations WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Conversation {
                        id: row.get(0)?,
                        user_id: row.get(1)?,
                        consensus_profile_id: row.get(2)?,
                        total_cost: row.get(3)?,
                        total_tokens_input: row.get(4)?,
                        total_tokens_output: row.get(5)?,
                        start_time: row.get(6)?,
                        end_time: row.get(7)?,
                        success_rate: row.get(8)?,
                        quality_score: row.get(9)?,
                        consensus_improvement: row.get(10)?,
                        confidence_level: row.get(11)?,
                        success: row.get(12)?,
                        created_at: row.get(13)?,
                        updated_at: row.get(14)?,
                    })
                },
            )
            .optional()?;

        Ok(conversation)
    }

    /// Update conversation metrics
    pub async fn update_metrics(
        &mut self,
        cost: f64,
        input_tokens: i32,
        output_tokens: i32,
    ) -> Result<()> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        self.total_cost += cost;
        self.total_tokens_input += input_tokens;
        self.total_tokens_output += output_tokens;
        self.updated_at = current_timestamp();

        conn.execute(
            "UPDATE conversations
             SET total_cost = ?1, total_tokens_input = ?2, total_tokens_output = ?3, updated_at = ?4
             WHERE id = ?5",
            params![
                &self.total_cost,
                &self.total_tokens_input,
                &self.total_tokens_output,
                &self.updated_at,
                &self.id,
            ],
        )?;

        Ok(())
    }

    /// Complete the conversation
    pub async fn complete(&mut self) -> Result<()> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        self.end_time = Some(current_timestamp());
        self.updated_at = current_timestamp();

        conn.execute(
            "UPDATE conversations
             SET end_time = ?1, updated_at = ?2
             WHERE id = ?3",
            params![&self.end_time, &self.updated_at, &self.id],
        )?;

        Ok(())
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

/// Knowledge conversation model for memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConversation {
    pub id: String,
    pub conversation_id: Option<String>,
    pub question: String,
    pub final_answer: String,
    pub source_of_truth: String,
    pub conversation_context: Option<String>,
    pub profile_id: Option<String>,
    pub created_at: String,
    pub last_updated: String,
}

impl KnowledgeConversation {
    /// Create a new knowledge entry
    pub async fn create(
        conversation_id: Option<String>,
        question: String,
        final_answer: String,
        source_of_truth: String,
    ) -> Result<Self> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let knowledge = KnowledgeConversation {
            id: generate_id(),
            conversation_id,
            question,
            final_answer,
            source_of_truth,
            conversation_context: None,
            profile_id: None,
            created_at: current_timestamp(),
            last_updated: current_timestamp(),
        };

        conn.execute(
            "INSERT INTO knowledge_conversations (
                id, conversation_id, question, final_answer, source_of_truth,
                conversation_context, profile_id, created_at, last_updated
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &knowledge.id,
                &knowledge.conversation_id,
                &knowledge.question,
                &knowledge.final_answer,
                &knowledge.source_of_truth,
                &knowledge.conversation_context,
                &knowledge.profile_id,
                &knowledge.created_at,
                &knowledge.last_updated,
            ],
        )?;

        Ok(knowledge)
    }

    /// Search knowledge by similarity (simple keyword search for now)
    pub async fn search(query: &str, limit: usize) -> Result<Vec<Self>> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let search_pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, question, final_answer, source_of_truth,
                    conversation_context, profile_id, created_at, last_updated
             FROM knowledge_conversations
             WHERE question LIKE ?1 OR final_answer LIKE ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;

        let knowledge_entries = stmt
            .query_map(params![&search_pattern, limit], |row| {
                Ok(KnowledgeConversation {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    question: row.get(2)?,
                    final_answer: row.get(3)?,
                    source_of_truth: row.get(4)?,
                    conversation_context: row.get(5)?,
                    profile_id: row.get(6)?,
                    created_at: row.get(7)?,
                    last_updated: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(knowledge_entries)
    }

    /// Get recent knowledge entries
    pub async fn get_recent(limit: usize) -> Result<Vec<Self>> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, question, final_answer, source_of_truth,
                    conversation_context, profile_id, created_at, last_updated
             FROM knowledge_conversations
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;

        let knowledge_entries = stmt
            .query_map(params![limit], |row| {
                Ok(KnowledgeConversation {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    question: row.get(2)?,
                    final_answer: row.get(3)?,
                    source_of_truth: row.get(4)?,
                    conversation_context: row.get(5)?,
                    profile_id: row.get(6)?,
                    created_at: row.get(7)?,
                    last_updated: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(knowledge_entries)
    }

    /// Find knowledge conversation by ID
    pub async fn find_by_id(id: &str) -> Result<Option<Self>> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let knowledge = conn
            .query_row(
                "SELECT id, conversation_id, question, final_answer, source_of_truth,
                        conversation_context, profile_id, created_at, last_updated
                 FROM knowledge_conversations
                 WHERE id = ?1",
                params![id],
                |row| {
                    Ok(KnowledgeConversation {
                        id: row.get(0)?,
                        conversation_id: row.get(1)?,
                        question: row.get(2)?,
                        final_answer: row.get(3)?,
                        source_of_truth: row.get(4)?,
                        conversation_context: row.get(5)?,
                        profile_id: row.get(6)?,
                        created_at: row.get(7)?,
                        last_updated: row.get(8)?,
                    })
                },
            )
            .optional()?;

        Ok(knowledge)
    }
}

/// Consensus profile for model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProfile {
    pub id: String,
    pub profile_name: String,
    pub generator_model: String,
    pub refiner_model: String,
    pub validator_model: String,
    pub curator_model: String,
    pub created_at: String,
    pub updated_at: String,
}

impl ConsensusProfile {
    /// Create a new consensus profile
    pub async fn create(
        profile_name: String,
        generator_model: String,
        refiner_model: String,
        validator_model: String,
        curator_model: String,
    ) -> Result<Self> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let profile = ConsensusProfile {
            id: generate_id(),
            profile_name,
            generator_model,
            refiner_model,
            validator_model,
            curator_model,
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };

        conn.execute(
            "INSERT INTO consensus_profiles (
                id, profile_name, generator_model, refiner_model,
                validator_model, curator_model, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &profile.id,
                &profile.profile_name,
                &profile.generator_model,
                &profile.refiner_model,
                &profile.validator_model,
                &profile.curator_model,
                &profile.created_at,
                &profile.updated_at,
            ],
        )?;

        Ok(profile)
    }

    /// Find profile by name
    pub async fn find_by_name(name: &str) -> Result<Option<Self>> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let profile = conn
            .query_row(
                "SELECT id, profile_name, generator_model, refiner_model,
                        validator_model, curator_model, created_at, updated_at
                 FROM consensus_profiles WHERE profile_name = ?1",
                params![name],
                |row| {
                    Ok(ConsensusProfile {
                        id: row.get(0)?,
                        profile_name: row.get(1)?,
                        generator_model: row.get(2)?,
                        refiner_model: row.get(3)?,
                        validator_model: row.get(4)?,
                        curator_model: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                },
            )
            .optional()?;

        Ok(profile)
    }

    /// List all profiles
    pub async fn list_all() -> Result<Vec<Self>> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, profile_name, generator_model, refiner_model,
                    validator_model, curator_model, created_at, updated_at
             FROM consensus_profiles
             ORDER BY profile_name ASC",
        )?;

        let profiles = stmt
            .query_map([], |row| {
                Ok(ConsensusProfile {
                    id: row.get(0)?,
                    profile_name: row.get(1)?,
                    generator_model: row.get(2)?,
                    refiner_model: row.get(3)?,
                    validator_model: row.get(4)?,
                    curator_model: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(profiles)
    }
}

/// Activity log entry for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: i32,
    pub event_type: String,
    pub conversation_id: Option<String>,
    pub user_id: Option<String>,
    pub stage: Option<String>,
    pub model_used: Option<String>,
    pub provider_name: Option<String>,
    pub cost: Option<f64>,
    pub duration_ms: Option<i32>,
    pub tokens_used: Option<i32>,
    pub error_message: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

impl ActivityLog {
    /// Log an activity
    pub async fn log(
        event_type: String,
        conversation_id: Option<String>,
        user_id: Option<String>,
        stage: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let metadata_str = metadata.map(|v| v.to_string());

        conn.execute(
            "INSERT INTO activity_log (
                event_type, conversation_id, user_id, stage, metadata, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &event_type,
                &conversation_id,
                &user_id,
                &stage,
                &metadata_str,
                current_timestamp(),
            ],
        )?;

        Ok(())
    }

    /// Get recent activity
    pub async fn get_recent(limit: usize) -> Result<Vec<Self>> {
        let db = get_database().await?;
        let conn = db.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, event_type, conversation_id, user_id, stage, model_used,
                    provider_name, cost, duration_ms, tokens_used, error_message,
                    metadata, created_at
             FROM activity_log
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;

        let activities = stmt
            .query_map(params![limit], |row| {
                Ok(ActivityLog {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    conversation_id: row.get(2)?,
                    user_id: row.get(3)?,
                    stage: row.get(4)?,
                    model_used: row.get(5)?,
                    provider_name: row.get(6)?,
                    cost: row.get(7)?,
                    duration_ms: row.get(8)?,
                    tokens_used: row.get(9)?,
                    error_message: row.get(10)?,
                    metadata: row.get(11)?,
                    created_at: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(activities)
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::bail;
    use tempfile::TempDir;

    async fn setup_test_db() -> Result<(Arc<DatabaseManager>, TempDir)> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let config = DatabaseConfig {
            path: db_path,
            max_connections: 5,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(60),
            enable_wal: true,
            enable_foreign_keys: true,
            cache_size: -1000,
            synchronous: "NORMAL".to_string(),
            journal_mode: "WAL".to_string(),
        };

        let db = Arc::new(DatabaseManager::new(config).await?);

        // Set global instance for tests
        DATABASE.set(db.clone()).ok();

        Ok((db, temp_dir))
    }

    #[tokio::test]
    async fn test_database_initialization() -> Result<()> {
        let (db, _temp_dir) = setup_test_db().await?;

        // Test basic connectivity
        let health = db.health_check().await?;
        assert!(health.healthy);
        assert!(health.wal_mode_active);
        assert!(health.foreign_keys_enabled);

        Ok(())
    }

    #[tokio::test]
    async fn test_user_crud() -> Result<()> {
        let (_db, _temp_dir) = setup_test_db().await?;

        // Create user
        let user = User::create(
            Some("test@example.com".to_string()),
            Some("test-license".to_string()),
        )
        .await?;

        assert!(!user.id.is_empty());
        assert_eq!(user.email, Some("test@example.com".to_string()));

        // Find by ID
        let found = User::find_by_id(&user.id).await?;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);

        // Find by email
        let found = User::find_by_email("test@example.com").await?;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_conversation_crud() -> Result<()> {
        let (_db, _temp_dir) = setup_test_db().await?;

        // Create user first
        let user = User::create(None, None).await?;

        // Create conversation
        let mut conv = Conversation::create(Some(user.id.clone()), None).await?;
        assert!(!conv.id.is_empty());
        assert_eq!(conv.user_id, Some(user.id));

        // Update metrics
        conv.update_metrics(0.05, 100, 200).await?;
        assert_eq!(conv.total_cost, 0.05);
        assert_eq!(conv.total_tokens_input, 100);
        assert_eq!(conv.total_tokens_output, 200);

        // Complete conversation
        conv.complete().await?;
        assert!(conv.end_time.is_some());

        // Find by ID
        let found = Conversation::find_by_id(&conv.id).await?;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, conv.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_message_crud() -> Result<()> {
        let (_db, _temp_dir) = setup_test_db().await?;

        // Create conversation first
        let conv = Conversation::create(None, None).await?;

        // Create messages
        let msg1 = Message::create(
            conv.id.clone(),
            "user".to_string(),
            "Hello".to_string(),
            None,
            None,
        )
        .await?;

        let msg2 = Message::create(
            conv.id.clone(),
            "assistant".to_string(),
            "Hi there!".to_string(),
            Some("generator".to_string()),
            Some("claude-3-opus".to_string()),
        )
        .await?;

        // Find messages by conversation
        let messages = Message::find_by_conversation(&conv.id).await?;
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, msg1.id);
        assert_eq!(messages[1].id, msg2.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_knowledge_search() -> Result<()> {
        let (_db, _temp_dir) = setup_test_db().await?;

        // Create knowledge entries
        let k1 = KnowledgeConversation::create(
            None,
            "How to implement a REST API?".to_string(),
            "Use a web framework like Express or FastAPI".to_string(),
            "Common web development knowledge".to_string(),
        )
        .await?;

        let k2 = KnowledgeConversation::create(
            None,
            "What is Rust ownership?".to_string(),
            "Rust's memory management system".to_string(),
            "Rust documentation".to_string(),
        )
        .await?;

        // Search for REST
        let results = KnowledgeConversation::search("REST", 10).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, k1.id);

        // Search for Rust
        let results = KnowledgeConversation::search("Rust", 10).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, k2.id);

        // Get recent
        let recent = KnowledgeConversation::get_recent(10).await?;
        assert_eq!(recent.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_consensus_profiles() -> Result<()> {
        let (_db, _temp_dir) = setup_test_db().await?;

        // Create profile
        let profile = ConsensusProfile::create(
            "Test Profile".to_string(),
            "claude-3-opus".to_string(),
            "gpt-4".to_string(),
            "claude-3-sonnet".to_string(),
            "gpt-4-turbo".to_string(),
        )
        .await?;

        // Find by name
        let found = ConsensusProfile::find_by_name("Test Profile").await?;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, profile.id);

        // List all
        let all = ConsensusProfile::list_all().await?;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, profile.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_activity_logging() -> Result<()> {
        let (_db, _temp_dir) = setup_test_db().await?;

        // Log activities
        ActivityLog::log(
            "query_start".to_string(),
            None,
            None,
            None,
            Some(serde_json::json!({"test": true})),
        )
        .await?;

        ActivityLog::log(
            "query_complete".to_string(),
            None,
            None,
            Some("generator".to_string()),
            None,
        )
        .await?;

        // Get recent
        let activities = ActivityLog::get_recent(10).await?;
        assert_eq!(activities.len(), 2);
        assert_eq!(activities[0].event_type, "query_complete");
        assert_eq!(activities[1].event_type, "query_start");

        Ok(())
    }

    #[tokio::test]
    async fn test_connection_pooling() -> Result<()> {
        let (db, _temp_dir) = setup_test_db().await?;

        // Test multiple concurrent connections
        let mut handles = vec![];

        for i in 0..10 {
            let db_clone = db.clone();
            let handle = tokio::spawn(async move {
                let conn = db_clone.get_connection()?;
                let result: i32 =
                    conn.query_row("SELECT ?1 as id", params![i], |row| row.get(0))?;
                Ok::<_, anyhow::Error>(result)
            });
            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;

        // All connections should succeed
        for (i, result) in results.into_iter().enumerate() {
            let value = result??;
            assert_eq!(value, i as i32);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_transaction_rollback() -> Result<()> {
        let (_db, _temp_dir) = setup_test_db().await?;

        // Create a user
        let user = User::create(Some("tx@test.com".to_string()), None).await?;

        // Try a failing transaction
        let result = execute_transaction(|tx| {
            // Update user
            tx.execute(
                "UPDATE users SET tier = ?1 WHERE id = ?2",
                params!["PREMIUM", &user.id],
            )?;

            // Force an error
            bail!("Simulated error");
        })
        .await;

        assert!(result.is_err());

        // Check that the update was rolled back
        let found = User::find_by_id(&user.id).await?;
        assert_eq!(found.unwrap().tier, "FREE");

        Ok(())
    }
}

// Utility functions for database operations

/// Execute a query that doesn't return results
pub async fn execute_query(query: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize> {
    let db = get_database().await?;
    let conn = db.get_connection()?;
    let result = conn.execute(query, params)?;
    Ok(result)
}

/// Fetch all results from a query
pub async fn fetch_all_query<T, F>(
    query: &str,
    params: &[&dyn rusqlite::ToSql],
    f: F,
) -> Result<Vec<T>>
where
    F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
{
    let db = get_database().await?;
    let conn = db.get_connection()?;
    let mut stmt = conn.prepare(query)?;
    let results = stmt
        .query_map(params, f)?
        .collect::<rusqlite::Result<Vec<T>>>()?;
    Ok(results)
}

/// Fetch optional result from a query
pub async fn fetch_optional_query<T, F>(
    query: &str,
    params: &[&dyn rusqlite::ToSql],
    f: F,
) -> Result<Option<T>>
where
    F: FnOnce(&rusqlite::Row) -> rusqlite::Result<T>,
{
    let db = get_database().await?;
    let conn = db.get_connection()?;
    let result = conn.query_row(query, params, f).optional()?;
    Ok(result)
}
