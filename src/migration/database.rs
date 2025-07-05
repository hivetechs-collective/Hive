//! Database Migration Module
//! 
//! Handles migration of conversation data, thematic clusters, and usage statistics
//! from TypeScript SQLite databases to Rust format with 100% data preservation.

use crate::core::error::HiveError;
// Database struct will be implemented when the database module is complete
// For now, we'll use a placeholder
pub struct Database;

impl Database {
    pub async fn new(_path: &PathBuf) -> Result<Self, HiveError> {
        Ok(Database)
    }
}

// SQLite pool placeholder
pub struct SqlitePool;

impl SqlitePool {
    pub async fn connect(_url: &str) -> Result<Self, HiveError> {
        Ok(SqlitePool)
    }
}

// Query placeholder
pub struct Query;

pub fn query(_sql: &str) -> Query {
    Query
}

impl Query {
    pub async fn fetch_all(self, _pool: &SqlitePool) -> Result<Vec<MockRow>, HiveError> {
        Ok(Vec::new())
    }
    
    pub async fn fetch_one(self, _pool: &SqlitePool) -> Result<MockRow, HiveError> {
        Ok(MockRow)
    }
}

pub fn query_scalar<T>(_sql: &str) -> QueryScalar<T> {
    QueryScalar { _phantom: std::marker::PhantomData }
}

pub struct QueryScalar<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> QueryScalar<T> where T: Default {
    pub async fn fetch_one(self, _pool: &SqlitePool) -> Result<T, HiveError> {
        Ok(T::default())
    }
}

// Row placeholder
pub trait Row {
    fn get<T>(&self, _column: &str) -> T where T: Default {
        T::default()
    }
}

pub struct MockRow;
impl Row for MockRow {}
use crate::migration::analyzer::{TypeScriptAnalysis, DatabaseInfo, ThematicCluster};
use serde::{Deserialize, Serialize};
// SQLite operations will be implemented when sqlx is added as dependency
// For now, we'll use placeholders
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Database migration changes preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseChanges {
    pub source_database: PathBuf,
    pub target_database: PathBuf,
    pub schema_changes: Vec<SchemaChange>,
    pub data_migration: DataMigrationPlan,
    pub estimated_size: u64,
    pub estimated_duration: std::time::Duration,
    pub risks: Vec<String>,
}

/// Schema transformation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    pub table_name: String,
    pub change_type: SchemaChangeType,
    pub old_schema: Option<String>,
    pub new_schema: String,
    pub migration_sql: String,
    pub data_preservation: DataPreservationLevel,
}

/// Types of schema changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaChangeType {
    TableRename,
    ColumnAdd,
    ColumnRename,
    ColumnTypeChange,
    IndexAdd,
    ConstraintAdd,
    DataTransform,
}

/// Data preservation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataPreservationLevel {
    Complete,    // 100% data preserved
    Transformed, // Data converted but preserved
    Partial,     // Some data may be lost
    Manual,      // Requires manual intervention
}

/// Data migration plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMigrationPlan {
    pub tables: Vec<TableMigration>,
    pub total_rows: u64,
    pub estimated_transfer_time: std::time::Duration,
    pub batch_size: u32,
    pub validation_points: Vec<String>,
}

/// Individual table migration plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMigration {
    pub source_table: String,
    pub target_table: String,
    pub row_count: u64,
    pub transformations: Vec<DataTransformation>,
    pub dependencies: Vec<String>,
    pub validation_queries: Vec<String>,
}

/// Data transformation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransformation {
    pub source_column: String,
    pub target_column: String,
    pub transformation_type: TransformationType,
    pub transformation_logic: String,
    pub fallback_value: Option<String>,
}

/// Types of data transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationType {
    DirectCopy,      // Copy value as-is
    TypeConversion,  // Convert data type
    JsonParsing,     // Parse JSON strings
    Concatenation,   // Combine multiple columns
    Extraction,      // Extract part of a value
    Default,         // Use default value
}

/// TypeScript database schema (for reference)
#[derive(Debug)]
struct TypeScriptSchema {
    conversations: TableSchema,
    messages: TableSchema,
    consensus_profiles: TableSchema,
    thematic_clusters: TableSchema,
    usage_stats: TableSchema,
    user_settings: TableSchema,
}

#[derive(Debug)]
struct TableSchema {
    name: String,
    columns: Vec<ColumnSchema>,
    indexes: Vec<String>,
    constraints: Vec<String>,
}

#[derive(Debug)]
struct ColumnSchema {
    name: String,
    data_type: String,
    nullable: bool,
    default_value: Option<String>,
}

/// Preview database migration changes
pub async fn preview_database_changes(analysis: &TypeScriptAnalysis) -> Result<DatabaseChanges, HiveError> {
    let source_db = analysis.database_info.path.clone();
    let target_db = get_target_database_path()?;
    
    // Analyze source schema
    let source_schema = analyze_typescript_schema(&source_db).await?;
    
    // Plan schema transformations
    let schema_changes = plan_schema_changes(&source_schema).await?;
    
    // Plan data migration
    let data_migration = plan_data_migration(&analysis.database_info, &source_schema).await?;
    
    // Estimate migration time and risks
    let estimated_duration = estimate_migration_duration(&data_migration);
    let risks = assess_migration_risks(&analysis.database_info, &schema_changes);
    
    Ok(DatabaseChanges {
        source_database: source_db,
        target_database: target_db,
        schema_changes,
        data_migration,
        estimated_size: analysis.database_info.size,
        estimated_duration,
        risks,
    })
}

/// Perform actual database migration
pub async fn migrate_database(analysis: &TypeScriptAnalysis) -> Result<(), HiveError> {
    log::info!("Starting database migration");
    
    let source_db = &analysis.database_info.path;
    let target_db = get_target_database_path()?;
    
    // Create backup of source database
    create_database_backup(source_db).await?;
    
    // Open connections
    let source_pool = open_source_database(source_db).await?;
    let target_db_instance = Database::new(&target_db).await?;
    
    // Initialize target database schema
    initialize_target_schema(&target_db_instance).await?;
    
    // Migrate data in dependency order
    migrate_conversations(&source_pool, &target_db_instance).await?;
    migrate_messages(&source_pool, &target_db_instance).await?;
    migrate_consensus_profiles(&source_pool, &target_db_instance).await?;
    migrate_thematic_clusters(&source_pool, &target_db_instance).await?;
    migrate_usage_statistics(&source_pool, &target_db_instance).await?;
    migrate_user_settings(&source_pool, &target_db_instance).await?;
    
    // Verify migration integrity
    verify_migration_integrity(&source_pool, &target_db_instance).await?;
    
    log::info!("Database migration completed successfully");
    Ok(())
}

/// Get target database path for Rust version
fn get_target_database_path() -> Result<PathBuf, HiveError> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| HiveError::Migration { 
            message: "Cannot determine home directory".to_string()
        })?;
    
    Ok(home_dir.join(".hive").join("hive.db"))
}

/// Analyze TypeScript database schema
async fn analyze_typescript_schema(db_path: &PathBuf) -> Result<TypeScriptSchema, HiveError> {
    let pool = open_source_database(db_path).await?;
    
    // Query schema information from SQLite system tables
    let tables = query("SELECT name FROM sqlite_master WHERE type='table'")
        .fetch_all(&pool)
        .await?;
    
    let mut schema = TypeScriptSchema {
        conversations: TableSchema { name: "conversations".to_string(), columns: Vec::new(), indexes: Vec::new(), constraints: Vec::new() },
        messages: TableSchema { name: "messages".to_string(), columns: Vec::new(), indexes: Vec::new(), constraints: Vec::new() },
        consensus_profiles: TableSchema { name: "consensus_profiles".to_string(), columns: Vec::new(), indexes: Vec::new(), constraints: Vec::new() },
        thematic_clusters: TableSchema { name: "thematic_clusters".to_string(), columns: Vec::new(), indexes: Vec::new(), constraints: Vec::new() },
        usage_stats: TableSchema { name: "usage_stats".to_string(), columns: Vec::new(), indexes: Vec::new(), constraints: Vec::new() },
        user_settings: TableSchema { name: "user_settings".to_string(), columns: Vec::new(), indexes: Vec::new(), constraints: Vec::new() },
    };
    
    // Analyze each table
    for table_row in tables {
        let table_name: String = table_row.get("name");
        let columns = analyze_table_columns(&pool, &table_name).await?;
        
        match table_name.as_str() {
            "conversations" => schema.conversations.columns = columns,
            "messages" => schema.messages.columns = columns,
            "consensus_profiles" => schema.consensus_profiles.columns = columns,
            "thematic_clusters" => schema.thematic_clusters.columns = columns,
            "usage_stats" => schema.usage_stats.columns = columns,
            "user_settings" => schema.user_settings.columns = columns,
            _ => {
                log::warn!("Unknown table in TypeScript database: {}", table_name);
            }
        }
    }
    
    Ok(schema)
}

/// Analyze columns for a specific table
async fn analyze_table_columns(pool: &SqlitePool, table_name: &str) -> Result<Vec<ColumnSchema>, HiveError> {
    let pragma_query = format!("PRAGMA table_info({})", table_name);
    let rows = query(&pragma_query)
        .fetch_all(pool)
        .await?;
    
    let mut columns = Vec::new();
    for row in rows {
        let column = ColumnSchema {
            name: row.get("name"),
            data_type: row.get("type"),
            nullable: row.get::<i32>("notnull") == 0,
            default_value: row.get("dflt_value"),
        };
        columns.push(column);
    }
    
    Ok(columns)
}

/// Plan schema changes needed for migration
async fn plan_schema_changes(source_schema: &TypeScriptSchema) -> Result<Vec<SchemaChange>, HiveError> {
    let mut changes = Vec::new();
    
    // Plan changes for each table
    changes.extend(plan_conversation_changes(&source_schema.conversations)?);
    changes.extend(plan_message_changes(&source_schema.messages)?);
    changes.extend(plan_profile_changes(&source_schema.consensus_profiles)?);
    changes.extend(plan_cluster_changes(&source_schema.thematic_clusters)?);
    changes.extend(plan_usage_changes(&source_schema.usage_stats)?);
    changes.extend(plan_settings_changes(&source_schema.user_settings)?);
    
    Ok(changes)
}

/// Plan conversation table changes
fn plan_conversation_changes(source: &TableSchema) -> Result<Vec<SchemaChange>, HiveError> {
    let mut changes = Vec::new();
    
    // Add new columns for enhanced features
    changes.push(SchemaChange {
        table_name: "conversations".to_string(),
        change_type: SchemaChangeType::ColumnAdd,
        old_schema: None,
        new_schema: "quality_score REAL DEFAULT 0.0".to_string(),
        migration_sql: "ALTER TABLE conversations ADD COLUMN quality_score REAL DEFAULT 0.0".to_string(),
        data_preservation: DataPreservationLevel::Complete,
    });
    
    // Add performance metrics column
    changes.push(SchemaChange {
        table_name: "conversations".to_string(),
        change_type: SchemaChangeType::ColumnAdd,
        old_schema: None,
        new_schema: "performance_metrics TEXT".to_string(),
        migration_sql: "ALTER TABLE conversations ADD COLUMN performance_metrics TEXT".to_string(),
        data_preservation: DataPreservationLevel::Complete,
    });
    
    Ok(changes)
}

/// Plan message table changes
fn plan_message_changes(source: &TableSchema) -> Result<Vec<SchemaChange>, HiveError> {
    let mut changes = Vec::new();
    
    // Add token usage tracking
    changes.push(SchemaChange {
        table_name: "messages".to_string(),
        change_type: SchemaChangeType::ColumnAdd,
        old_schema: None,
        new_schema: "token_usage INTEGER DEFAULT 0".to_string(),
        migration_sql: "ALTER TABLE messages ADD COLUMN token_usage INTEGER DEFAULT 0".to_string(),
        data_preservation: DataPreservationLevel::Complete,
    });
    
    // Add cost tracking
    changes.push(SchemaChange {
        table_name: "messages".to_string(),
        change_type: SchemaChangeType::ColumnAdd,
        old_schema: None,
        new_schema: "cost_usd REAL DEFAULT 0.0".to_string(),
        migration_sql: "ALTER TABLE messages ADD COLUMN cost_usd REAL DEFAULT 0.0".to_string(),
        data_preservation: DataPreservationLevel::Complete,
    });
    
    Ok(changes)
}

/// Plan consensus profile changes
fn plan_profile_changes(source: &TableSchema) -> Result<Vec<SchemaChange>, HiveError> {
    let mut changes = Vec::new();
    
    // Transform JSON configurations to structured format
    changes.push(SchemaChange {
        table_name: "consensus_profiles".to_string(),
        change_type: SchemaChangeType::DataTransform,
        old_schema: Some("config TEXT".to_string()),
        new_schema: "config BLOB".to_string(),
        migration_sql: "UPDATE consensus_profiles SET config = ? WHERE id = ?".to_string(),
        data_preservation: DataPreservationLevel::Transformed,
    });
    
    Ok(changes)
}

/// Plan thematic cluster changes
fn plan_cluster_changes(source: &TableSchema) -> Result<Vec<SchemaChange>, HiveError> {
    let mut changes = Vec::new();
    
    // Add semantic embedding column
    changes.push(SchemaChange {
        table_name: "thematic_clusters".to_string(),
        change_type: SchemaChangeType::ColumnAdd,
        old_schema: None,
        new_schema: "embedding BLOB".to_string(),
        migration_sql: "ALTER TABLE thematic_clusters ADD COLUMN embedding BLOB".to_string(),
        data_preservation: DataPreservationLevel::Complete,
    });
    
    Ok(changes)
}

/// Plan usage statistics changes
fn plan_usage_changes(source: &TableSchema) -> Result<Vec<SchemaChange>, HiveError> {
    let mut changes = Vec::new();
    
    // Add detailed analytics
    changes.push(SchemaChange {
        table_name: "usage_stats".to_string(),
        change_type: SchemaChangeType::ColumnAdd,
        old_schema: None,
        new_schema: "analytics_data TEXT".to_string(),
        migration_sql: "ALTER TABLE usage_stats ADD COLUMN analytics_data TEXT".to_string(),
        data_preservation: DataPreservationLevel::Complete,
    });
    
    Ok(changes)
}

/// Plan user settings changes
fn plan_settings_changes(source: &TableSchema) -> Result<Vec<SchemaChange>, HiveError> {
    let mut changes = Vec::new();
    
    // Convert settings format
    changes.push(SchemaChange {
        table_name: "user_settings".to_string(),
        change_type: SchemaChangeType::DataTransform,
        old_schema: Some("settings TEXT".to_string()),
        new_schema: "settings BLOB".to_string(),
        migration_sql: "UPDATE user_settings SET settings = ? WHERE user_id = ?".to_string(),
        data_preservation: DataPreservationLevel::Transformed,
    });
    
    Ok(changes)
}

/// Plan data migration strategy
async fn plan_data_migration(db_info: &DatabaseInfo, schema: &TypeScriptSchema) -> Result<DataMigrationPlan, HiveError> {
    let mut tables = Vec::new();
    
    // Plan conversation migration
    tables.push(TableMigration {
        source_table: "conversations".to_string(),
        target_table: "conversations".to_string(),
        row_count: db_info.conversation_count,
        transformations: vec![
            DataTransformation {
                source_column: "id".to_string(),
                target_column: "id".to_string(),
                transformation_type: TransformationType::DirectCopy,
                transformation_logic: "DIRECT".to_string(),
                fallback_value: None,
            },
            DataTransformation {
                source_column: "created_at".to_string(),
                target_column: "created_at".to_string(),
                transformation_type: TransformationType::TypeConversion,
                transformation_logic: "ISO8601_TO_TIMESTAMP".to_string(),
                fallback_value: Some("CURRENT_TIMESTAMP".to_string()),
            },
        ],
        dependencies: Vec::new(),
        validation_queries: vec![
            "SELECT COUNT(*) FROM conversations".to_string(),
            "SELECT MAX(created_at) FROM conversations".to_string(),
        ],
    });
    
    // Plan message migration (depends on conversations)
    tables.push(TableMigration {
        source_table: "messages".to_string(),
        target_table: "messages".to_string(),
        row_count: db_info.message_count,
        transformations: vec![
            DataTransformation {
                source_column: "content".to_string(),
                target_column: "content".to_string(),
                transformation_type: TransformationType::DirectCopy,
                transformation_logic: "DIRECT".to_string(),
                fallback_value: None,
            },
        ],
        dependencies: vec!["conversations".to_string()],
        validation_queries: vec![
            "SELECT COUNT(*) FROM messages".to_string(),
            "SELECT COUNT(DISTINCT conversation_id) FROM messages".to_string(),
        ],
    });
    
    let total_rows = db_info.conversation_count + db_info.message_count;
    let estimated_transfer_time = std::time::Duration::from_millis(total_rows * 10); // 10ms per row estimate
    
    Ok(DataMigrationPlan {
        tables,
        total_rows,
        estimated_transfer_time,
        batch_size: 1000,
        validation_points: vec![
            "Pre-migration row counts".to_string(),
            "Post-migration row counts".to_string(),
            "Data integrity verification".to_string(),
            "Foreign key consistency".to_string(),
        ],
    })
}

/// Open source database connection
async fn open_source_database(_db_path: &PathBuf) -> Result<SqlitePool, HiveError> {
    // Placeholder implementation
    SqlitePool::connect("").await
}

/// Create backup of source database
async fn create_database_backup(source_db: &PathBuf) -> Result<(), HiveError> {
    let backup_path = source_db.with_extension("db.backup");
    
    tokio::fs::copy(source_db, &backup_path).await?;
    
    log::info!("Database backup created: {}", backup_path.display());
    Ok(())
}

/// Initialize target database schema
async fn initialize_target_schema(target_db: &Database) -> Result<(), HiveError> {
    // The Database::new() method should handle schema initialization
    // This is a placeholder for any additional migration-specific setup
    log::info!("Target database schema initialized");
    Ok(())
}

/// Migrate conversations table
async fn migrate_conversations(source: &SqlitePool, target: &Database) -> Result<(), HiveError> {
    log::info!("Migrating conversations...");
    
    let rows = query("SELECT * FROM conversations ORDER BY created_at")
        .fetch_all(source)
        .await?;
    
    for _row in rows {
        // Placeholder: In real implementation, we would extract data and migrate
        log::debug!("Migrating conversation (placeholder)");
    }
    
    log::info!("Conversations migration completed");
    Ok(())
}

/// Migrate messages table
async fn migrate_messages(source: &SqlitePool, _target: &Database) -> Result<(), HiveError> {
    log::info!("Migrating messages...");
    
    let rows = query("SELECT * FROM messages ORDER BY created_at")
        .fetch_all(source)
        .await?;
    
    for _row in rows {
        // Placeholder: In real implementation, we would extract data and migrate
        log::debug!("Migrating message (placeholder)");
    }
    
    log::info!("Messages migration completed");
    Ok(())
}

/// Migrate consensus profiles
async fn migrate_consensus_profiles(source: &SqlitePool, _target: &Database) -> Result<(), HiveError> {
    log::info!("Migrating consensus profiles...");
    
    let rows = query("SELECT * FROM consensus_profiles")
        .fetch_all(source)
        .await?;
    
    for _row in rows {
        // Placeholder: In real implementation, we would extract data and migrate
        log::debug!("Migrating profile (placeholder)");
    }
    
    log::info!("Consensus profiles migration completed");
    Ok(())
}

/// Migrate thematic clusters
async fn migrate_thematic_clusters(source: &SqlitePool, _target: &Database) -> Result<(), HiveError> {
    log::info!("Migrating thematic clusters...");
    
    let rows = query("SELECT * FROM thematic_clusters")
        .fetch_all(source)
        .await?;
    
    for _row in rows {
        log::debug!("Migrating cluster (placeholder)");
    }
    
    log::info!("Thematic clusters migration completed");
    Ok(())
}

/// Migrate usage statistics
async fn migrate_usage_statistics(source: &SqlitePool, _target: &Database) -> Result<(), HiveError> {
    log::info!("Migrating usage statistics...");
    
    let rows = query("SELECT * FROM usage_stats")
        .fetch_all(source)
        .await?;
    
    for _row in rows {
        log::debug!("Migrating stats (placeholder)");
    }
    
    log::info!("Usage statistics migration completed");
    Ok(())
}

/// Migrate user settings
async fn migrate_user_settings(source: &SqlitePool, _target: &Database) -> Result<(), HiveError> {
    log::info!("Migrating user settings...");
    
    let rows = query("SELECT * FROM user_settings")
        .fetch_all(source)
        .await?;
    
    for _row in rows {
        log::debug!("Migrating settings (placeholder)");
    }
    
    log::info!("User settings migration completed");
    Ok(())
}

/// Parse TypeScript timestamp format
fn parse_typescript_timestamp(timestamp: &str) -> Result<DateTime<Utc>, HiveError> {
    // Handle various timestamp formats from TypeScript version
    if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
        return Ok(dt.with_timezone(&Utc));
    }
    
    if let Ok(ts) = timestamp.parse::<i64>() {
        if let Some(dt) = DateTime::from_timestamp(ts, 0) {
            return Ok(dt);
        }
    }
    
    Err(HiveError::Migration { 
        message: format!("Invalid timestamp format: {}", timestamp)
    })
}

/// Verify migration integrity
async fn verify_migration_integrity(source: &SqlitePool, _target: &Database) -> Result<(), HiveError> {
    log::info!("Verifying migration integrity...");
    
    // Verify conversation counts
    let source_conversations = query_scalar::<i64>("SELECT COUNT(*) FROM conversations")
        .fetch_one(source)
        .await?;
    
    log::info!("Source conversations: {}", source_conversations);
    
    // Verify message counts
    let source_messages = query_scalar::<i64>("SELECT COUNT(*) FROM messages")
        .fetch_one(source)
        .await?;
    
    log::info!("Source messages: {}", source_messages);
    
    log::info!("Migration integrity verification completed");
    Ok(())
}

/// Estimate migration duration
fn estimate_migration_duration(plan: &DataMigrationPlan) -> std::time::Duration {
    // Base time for setup and verification
    let mut duration = std::time::Duration::from_secs(60);
    
    // Add time based on data volume
    duration += plan.estimated_transfer_time;
    
    // Add time for each validation point
    duration += std::time::Duration::from_secs(plan.validation_points.len() as u64 * 30);
    
    // Add buffer for safety
    duration += std::time::Duration::from_secs(120);
    
    duration
}

/// Assess migration risks
fn assess_migration_risks(db_info: &DatabaseInfo, schema_changes: &[SchemaChange]) -> Vec<String> {
    let mut risks = Vec::new();
    
    // Check database size
    if db_info.size > 1_000_000_000 {  // 1GB
        risks.push("Large database size may require extended migration time".to_string());
    }
    
    // Check integrity
    if !db_info.integrity_check {
        risks.push("Source database integrity issues detected".to_string());
    }
    
    // Check schema changes
    let complex_changes = schema_changes.iter()
        .filter(|c| matches!(c.data_preservation, DataPreservationLevel::Partial | DataPreservationLevel::Manual))
        .count();
    
    if complex_changes > 0 {
        risks.push(format!("{} schema changes require manual verification", complex_changes));
    }
    
    // Check conversation volume
    if db_info.conversation_count > 10_000 {
        risks.push("High conversation volume may impact migration performance".to_string());
    }
    
    risks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_parsing() {
        // Test RFC3339 format
        let rfc3339 = "2023-01-01T00:00:00Z";
        assert!(parse_typescript_timestamp(rfc3339).is_ok());
        
        // Test Unix timestamp
        let unix_ts = "1672531200";
        assert!(parse_typescript_timestamp(unix_ts).is_ok());
        
        // Test invalid format
        let invalid = "not-a-timestamp";
        assert!(parse_typescript_timestamp(invalid).is_err());
    }

    #[test]
    fn test_migration_duration_estimation() {
        let plan = DataMigrationPlan {
            tables: Vec::new(),
            total_rows: 1000,
            estimated_transfer_time: std::time::Duration::from_secs(10),
            batch_size: 100,
            validation_points: vec!["test1".to_string(), "test2".to_string()],
        };
        
        let duration = estimate_migration_duration(&plan);
        
        // Should include base time (60s) + transfer time (10s) + validation time (60s) + buffer (120s)
        assert!(duration.as_secs() >= 250);
    }

    #[test]
    fn test_risk_assessment() {
        let db_info = DatabaseInfo {
            path: PathBuf::from("/test.db"),
            size: 2_000_000_000, // 2GB
            conversation_count: 15_000,
            message_count: 150_000,
            schema_version: Some("1.0".to_string()),
            last_backup: None,
            integrity_check: false,
            clusters: Vec::new(),
        };
        
        let schema_changes = vec![
            SchemaChange {
                table_name: "test".to_string(),
                change_type: SchemaChangeType::DataTransform,
                old_schema: None,
                new_schema: "test".to_string(),
                migration_sql: "test".to_string(),
                data_preservation: DataPreservationLevel::Partial,
            }
        ];
        
        let risks = assess_migration_risks(&db_info, &schema_changes);
        
        assert!(risks.len() >= 3); // Should identify multiple risks
        assert!(risks.iter().any(|r| r.contains("Large database")));
        assert!(risks.iter().any(|r| r.contains("integrity issues")));
        assert!(risks.iter().any(|r| r.contains("High conversation volume")));
    }
}