//! Operation History Tracking and Database
//!
//! Stores file operation history, outcomes, and learning data to improve
//! future predictions and auto-accept decisions. This complements the
//! existing conversational memory system by tracking what was done,
//! not just what was said.

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use rusqlite::{Connection, params, OptionalExtension};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use std::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::operation_intelligence::{
    OperationContext, OperationOutcome, QualityMetrics
};

/// Operation history database manager
pub struct OperationHistoryDatabase {
    /// SQLite connection pool
    pool: Pool<SqliteConnectionManager>,
    
    /// In-memory cache for recent operations
    cache: Arc<RwLock<OperationCache>>,
    
    /// Statistics cache
    stats_cache: Arc<RwLock<Option<CachedStatistics>>>,
}

/// In-memory cache for recent operations
struct OperationCache {
    /// Recent operations by ID
    operations: HashMap<Uuid, OperationRecord>,
    
    /// Operations by file path for quick lookup
    by_path: HashMap<PathBuf, Vec<Uuid>>,
    
    /// Operations by type for pattern analysis
    by_type: HashMap<OperationType, Vec<Uuid>>,
    
    /// Maximum cache size
    max_size: usize,
}

/// Cached statistics with expiry
struct CachedStatistics {
    stats: OperationStatistics,
    expires_at: DateTime<Utc>,
}

/// Complete operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    /// Unique operation ID
    pub id: Uuid,
    
    /// Operation type
    pub operation_type: OperationType,
    
    /// File path affected
    pub file_path: PathBuf,
    
    /// Additional operation details
    pub details: OperationDetails,
    
    /// Context when operation was performed
    pub context: OperationContextRecord,
    
    /// Predicted scores
    pub prediction: PredictionRecord,
    
    /// Actual outcome
    pub outcome: OutcomeRecord,
    
    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Operation types for categorization
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    Create,
    Update,
    Append,
    Delete,
    Rename,
}

impl From<&FileOperation> for OperationType {
    fn from(op: &FileOperation) -> Self {
        match op {
            FileOperation::Create { .. } => OperationType::Create,
            FileOperation::Update { .. } => OperationType::Update,
            FileOperation::Append { .. } => OperationType::Append,
            FileOperation::Delete { .. } => OperationType::Delete,
            FileOperation::Rename { .. } => OperationType::Rename,
        }
    }
}

/// Additional operation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationDetails {
    /// Content size in bytes
    pub content_size: Option<usize>,
    
    /// File extension
    pub file_extension: Option<String>,
    
    /// Is it a test file?
    pub is_test_file: bool,
    
    /// Is it a config file?
    pub is_config_file: bool,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Stored context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationContextRecord {
    /// Repository path
    pub repository_path: PathBuf,
    
    /// Git commit hash at time of operation
    pub git_commit: Option<String>,
    
    /// Git branch
    pub git_branch: Option<String>,
    
    /// Source question that led to operation
    pub source_question: String,
    
    /// Number of related files in context
    pub related_files_count: usize,
    
    /// Project type (rust, node, python, etc)
    pub project_type: Option<String>,
}

/// Prediction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionRecord {
    /// Predicted confidence (0-100)
    pub confidence: f32,
    
    /// Predicted risk (0-100)
    pub risk: f32,
    
    /// Auto-accept mode used
    pub auto_accept_mode: String,
    
    /// Was it auto-executed?
    pub was_auto_executed: bool,
    
    /// AI recommendation
    pub recommendation: String,
}

/// Outcome record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeRecord {
    /// Did the operation succeed?
    pub success: bool,
    
    /// Error message if failed
    pub error_message: Option<String>,
    
    /// Execution time
    pub execution_time_ms: u64,
    
    /// Was rollback required?
    pub rollback_required: bool,
    
    /// Rollback time if performed
    pub rollback_time_ms: Option<u64>,
    
    /// User feedback
    pub user_feedback: Option<UserFeedback>,
    
    /// Quality metrics after operation
    pub post_quality_metrics: Option<QualityMetricsRecord>,
}

/// User feedback on operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    /// Satisfaction rating (0-5)
    pub satisfaction: f32,
    
    /// Was the operation helpful?
    pub was_helpful: bool,
    
    /// Text feedback
    pub comment: Option<String>,
    
    /// Timestamp
    pub provided_at: DateTime<Utc>,
}

/// Stored quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricsRecord {
    pub code_quality: f32,
    pub test_coverage_change: f32,
    pub performance_impact: f32,
    pub maintainability: f32,
}

/// Statistics about operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStatistics {
    pub total_operations: i64,
    pub successful_operations: i64,
    pub failed_operations: i64,
    pub auto_executed_operations: i64,
    pub rollbacks_required: i64,
    pub average_confidence: f32,
    pub average_risk: f32,
    pub average_execution_time_ms: f32,
    pub success_rate: f32,
    pub auto_execution_rate: f32,
    pub rollback_rate: f32,
    pub by_type: HashMap<OperationType, TypeStatistics>,
}

/// Statistics for a specific operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStatistics {
    pub count: i64,
    pub success_rate: f32,
    pub average_confidence: f32,
    pub average_risk: f32,
    pub common_failure_reasons: Vec<String>,
}

/// Search filters for operations
#[derive(Debug, Clone, Default)]
pub struct OperationFilters {
    /// Filter by operation type
    pub operation_type: Option<OperationType>,
    
    /// Filter by file path pattern
    pub file_path_pattern: Option<String>,
    
    /// Filter by success status
    pub success_only: Option<bool>,
    
    /// Filter by date range
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    
    /// Filter by confidence range
    pub min_confidence: Option<f32>,
    pub max_confidence: Option<f32>,
    
    /// Filter by risk range
    pub min_risk: Option<f32>,
    pub max_risk: Option<f32>,
    
    /// Limit results
    pub limit: Option<usize>,
}

impl OperationHistoryDatabase {
    /// Create new operation history database
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("🗄️  Initializing operation history database");
        
        // Create connection pool
        let manager = SqliteConnectionManager::file(database_url);
        let pool = Pool::new(manager)?;
        
        // Run migrations
        Self::run_migrations(&pool)?;
        
        // Initialize cache
        let cache = Arc::new(RwLock::new(OperationCache {
            operations: HashMap::new(),
            by_path: HashMap::new(),
            by_type: HashMap::new(),
            max_size: 1000,
        }));
        
        let stats_cache = Arc::new(RwLock::new(None));
        
        Ok(Self {
            pool,
            cache,
            stats_cache,
        })
    }
    
    /// Run database migrations
    fn run_migrations(pool: &Pool<SqliteConnectionManager>) -> Result<()> {
        debug!("Running operation history database migrations");
        
        let conn = pool.get()?;
        
        // Create main operations table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS operations (
                id TEXT PRIMARY KEY,
                operation_type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                details TEXT NOT NULL,
                context TEXT NOT NULL,
                prediction TEXT NOT NULL,
                outcome TEXT,
                created_at TIMESTAMP NOT NULL,
                completed_at TIMESTAMP,
                
                -- Denormalized fields for efficient querying
                success BOOLEAN,
                confidence REAL,
                risk REAL,
                was_auto_executed BOOLEAN,
                execution_time_ms INTEGER,
                rollback_required BOOLEAN
            )
            "#,
            [],
        )?;
        
        // Create indexes for common queries
        conn.execute("CREATE INDEX IF NOT EXISTS idx_operations_file_path ON operations(file_path)", [])?;
        
        conn.execute("CREATE INDEX IF NOT EXISTS idx_operations_type ON operations(operation_type)", [])?;
        
        conn.execute("CREATE INDEX IF NOT EXISTS idx_operations_created_at ON operations(created_at)", [])?;
        
        conn.execute("CREATE INDEX IF NOT EXISTS idx_operations_success ON operations(success)", [])?;
        
        // Create aggregated statistics table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS operation_statistics (
                id INTEGER PRIMARY KEY,
                calculated_at TIMESTAMP NOT NULL,
                statistics TEXT NOT NULL
            )
            "#,
            [],
        )?;
        
        // Create user feedback table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS operation_feedback (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_id TEXT NOT NULL,
                satisfaction REAL NOT NULL,
                was_helpful BOOLEAN NOT NULL,
                comment TEXT,
                provided_at TIMESTAMP NOT NULL,
                FOREIGN KEY (operation_id) REFERENCES operations(id)
            )
            "#,
            [],
        )?;
        
        info!("✅ Operation history database migrations completed");
        Ok(())
    }
    
    /// Record a new operation
    pub async fn record_operation(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
        confidence: f32,
        risk: f32,
        auto_accept_mode: &str,
        was_auto_executed: bool,
        recommendation: &str,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        // Extract operation details
        let operation_type = OperationType::from(operation);
        let file_path = self.get_operation_path(operation);
        let details = self.extract_operation_details(operation);
        let context_record = self.create_context_record(context);
        let prediction = PredictionRecord {
            confidence,
            risk,
            auto_accept_mode: auto_accept_mode.to_string(),
            was_auto_executed,
            recommendation: recommendation.to_string(),
        };
        
        // Insert into database
        let conn = self.pool.get()?;
        conn.execute(
            r#"
            INSERT INTO operations (
                id, operation_type, file_path, details, context, 
                prediction, created_at, confidence, risk, was_auto_executed
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                id.to_string(),
                serde_json::to_string(&operation_type)?,
                file_path.to_string_lossy(),
                serde_json::to_string(&details)?,
                serde_json::to_string(&context_record)?,
                serde_json::to_string(&prediction)?,
                now.to_rfc3339(),
                confidence,
                risk,
                was_auto_executed
            ],
        )?;
        
        // Add to cache
        let record = OperationRecord {
            id,
            operation_type,
            file_path: file_path.clone(),
            details,
            context: context_record,
            prediction,
            outcome: OutcomeRecord {
                success: false,
                error_message: None,
                execution_time_ms: 0,
                rollback_required: false,
                rollback_time_ms: None,
                user_feedback: None,
                post_quality_metrics: None,
            },
            created_at: now,
            completed_at: None,
        };
        
        self.add_to_cache(record);
        
        info!("📝 Recorded new operation: {} for {}", id, file_path.display());
        Ok(id)
    }
    
    /// Update operation outcome
    pub fn update_outcome(
        &self,
        operation_id: Uuid,
        outcome: &OperationOutcome,
    ) -> Result<()> {
        let now = Utc::now();
        
        let outcome_record = OutcomeRecord {
            success: outcome.success,
            error_message: outcome.error_message.clone(),
            execution_time_ms: outcome.execution_time.as_millis() as u64,
            rollback_required: outcome.rollback_required,
            rollback_time_ms: None,
            user_feedback: None,
            post_quality_metrics: outcome.post_operation_quality.as_ref().map(|q| {
                QualityMetricsRecord {
                    code_quality: q.code_quality,
                    test_coverage_change: q.test_coverage_change,
                    performance_impact: q.performance_impact,
                    maintainability: q.maintainability,
                }
            }),
        };
        
        let conn = self.pool.get()?;
        conn.execute(
            r#"
            UPDATE operations 
            SET outcome = ?1, completed_at = ?2, success = ?3, 
                execution_time_ms = ?4, rollback_required = ?5
            WHERE id = ?6
            "#,
            params![
                serde_json::to_string(&outcome_record)?,
                now.to_rfc3339(),
                outcome.success,
                outcome_record.execution_time_ms as i64,
                outcome.rollback_required,
                operation_id.to_string()
            ],
        )?;
        
        // Update cache
        self.update_cache_outcome(operation_id, outcome_record, now);
        
        // Invalidate statistics cache
        {
            let mut stats_cache = self.stats_cache.write().unwrap();
            *stats_cache = None;
        }
        
        info!("📊 Updated outcome for operation: {}", operation_id);
        Ok(())
    }
    
    /// Add user feedback
    pub fn add_user_feedback(
        &self,
        operation_id: Uuid,
        satisfaction: f32,
        was_helpful: bool,
        comment: Option<String>,
    ) -> Result<()> {
        let now = Utc::now();
        
        let conn = self.pool.get()?;
        conn.execute(
            r#"
            INSERT INTO operation_feedback (
                operation_id, satisfaction, was_helpful, comment, provided_at
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                operation_id.to_string(),
                satisfaction,
                was_helpful,
                comment.clone(),
                now.to_rfc3339()
            ],
        )?;
        
        // Update operation record with feedback
        let feedback = UserFeedback {
            satisfaction,
            was_helpful,
            comment,
            provided_at: now,
        };
        
        // Update in cache if present
        let mut cache = self.cache.write().unwrap();
        if let Some(record) = cache.operations.get_mut(&operation_id) {
            if let Some(outcome) = &mut record.outcome.user_feedback {
                *outcome = feedback;
            } else {
                record.outcome.user_feedback = Some(feedback);
            }
        }
        
        info!("👍 Added user feedback for operation: {}", operation_id);
        Ok(())
    }
    
    /// Find similar operations
    pub fn find_similar_operations(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
        limit: usize,
    ) -> Result<Vec<OperationRecord>> {
        let operation_type = OperationType::from(operation);
        let file_path = self.get_operation_path(operation);
        
        // Query similar operations
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, operation_type, file_path, details, context, 
                   prediction, outcome, created_at, completed_at
            FROM operations
            WHERE operation_type = ?1
              AND success IS NOT NULL
            ORDER BY 
              CASE WHEN file_path = ?2 THEN 0 ELSE 1 END,
              created_at DESC
            LIMIT ?3
            "#
        )?;
        
        let rows = stmt.query_map(
            params![
                serde_json::to_string(&operation_type)?,
                file_path.to_string_lossy(),
                limit as i64
            ],
            |row| self.row_to_record(row)
        )?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        
        Ok(results)
    }
    
    /// Get operation statistics
    pub fn get_statistics(&self) -> Result<OperationStatistics> {
        // Check cache first
        {
            let cache = self.stats_cache.read().unwrap();
            if let Some(cached) = cache.as_ref() {
                if cached.expires_at > Utc::now() {
                    return Ok(cached.stats.clone());
                }
            }
        }
        
        // Calculate fresh statistics
        let stats = self.calculate_statistics()?;
        
        // Cache for 5 minutes
        {
            let mut cache = self.stats_cache.write().unwrap();
            *cache = Some(CachedStatistics {
                stats: stats.clone(),
                expires_at: Utc::now() + chrono::Duration::minutes(5),
            });
        }
        
        Ok(stats)
    }
    
    /// Search operations with filters
    pub fn search_operations(
        &self,
        filters: &OperationFilters,
    ) -> Result<Vec<OperationRecord>> {
        let mut query = String::from(
            "SELECT id, operation_type, file_path, details, context, 
                    prediction, outcome, created_at, completed_at
             FROM operations WHERE 1=1"
        );
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        // Build dynamic query based on filters
        if let Some(op_type) = &filters.operation_type {
            query.push_str(" AND operation_type = ?");
            params.push(Box::new(serde_json::to_string(op_type)?));
        }
        
        if let Some(pattern) = &filters.file_path_pattern {
            query.push_str(" AND file_path LIKE ?");
            params.push(Box::new(format!("%{}%", pattern)));
        }
        
        if let Some(success) = filters.success_only {
            query.push_str(" AND success = ?");
            params.push(Box::new(success));
        }
        
        if let Some(after) = filters.after {
            query.push_str(" AND created_at >= ?");
            params.push(Box::new(after.to_rfc3339()));
        }
        
        if let Some(before) = filters.before {
            query.push_str(" AND created_at <= ?");
            params.push(Box::new(before.to_rfc3339()));
        }
        
        if let Some(min_conf) = filters.min_confidence {
            query.push_str(" AND confidence >= ?");
            params.push(Box::new(min_conf));
        }
        
        if let Some(max_conf) = filters.max_confidence {
            query.push_str(" AND confidence <= ?");
            params.push(Box::new(max_conf));
        }
        
        query.push_str(" ORDER BY created_at DESC");
        
        if let Some(limit) = filters.limit {
            query.push_str(" LIMIT ?");
            params.push(Box::new(limit as i64));
        }
        
        // Execute query
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(&query)?;
        
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter()
            .map(|p| p.as_ref() as &dyn rusqlite::ToSql)
            .collect();
        
        let rows = stmt.query_map(&param_refs[..], |row| self.row_to_record(row))?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        
        Ok(results)
    }
    
    // Helper methods
    
    fn get_operation_path(&self, operation: &FileOperation) -> PathBuf {
        match operation {
            FileOperation::Create { path, .. } => path.clone(),
            FileOperation::Update { path, .. } => path.clone(),
            FileOperation::Append { path, .. } => path.clone(),
            FileOperation::Delete { path } => path.clone(),
            FileOperation::Rename { from, .. } => from.clone(),
        }
    }
    
    fn extract_operation_details(&self, operation: &FileOperation) -> OperationDetails {
        let file_path = self.get_operation_path(operation);
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_string());
        
        let content_size = match operation {
            FileOperation::Create { content, .. } => Some(content.len()),
            FileOperation::Update { content, .. } => Some(content.len()),
            FileOperation::Append { content, .. } => Some(content.len()),
            _ => None,
        };
        
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        let is_test_file = file_name.contains("test") || 
                          file_name.contains("spec") ||
                          file_path.to_string_lossy().contains("/test");
        
        let is_config_file = matches!(
            extension.as_deref(),
            Some("json") | Some("toml") | Some("yaml") | Some("yml") | Some("ini")
        ) || file_name.contains("config");
        
        OperationDetails {
            content_size,
            file_extension: extension,
            is_test_file,
            is_config_file,
            metadata: HashMap::new(),
        }
    }
    
    fn create_context_record(&self, context: &OperationContext) -> OperationContextRecord {
        OperationContextRecord {
            repository_path: context.repository_path.clone(),
            git_commit: context.git_commit.clone(),
            git_branch: None, // TODO: Add git branch detection
            source_question: context.source_question.clone(),
            related_files_count: context.related_files.len(),
            project_type: self.detect_project_type(&context.project_metadata),
        }
    }
    
    fn detect_project_type(&self, metadata: &HashMap<String, String>) -> Option<String> {
        // Simple project type detection based on metadata
        if metadata.contains_key("cargo.toml") {
            Some("rust".to_string())
        } else if metadata.contains_key("package.json") {
            Some("node".to_string())
        } else if metadata.contains_key("requirements.txt") || metadata.contains_key("setup.py") {
            Some("python".to_string())
        } else {
            None
        }
    }
    
    fn add_to_cache(&self, record: OperationRecord) {
        let mut cache = self.cache.write().unwrap();
        
        // Check cache size
        if cache.operations.len() >= cache.max_size {
            // Remove oldest entries
            let mut oldest: Vec<_> = cache.operations.values()
                .map(|r| (r.created_at, r.id))
                .collect();
            oldest.sort_by_key(|&(time, _)| time);
            
            for (_, id) in oldest.iter().take(100) {
                if let Some(removed) = cache.operations.remove(id) {
                    // Remove from indexes
                    if let Some(path_ops) = cache.by_path.get_mut(&removed.file_path) {
                        path_ops.retain(|&op_id| op_id != *id);
                    }
                    if let Some(type_ops) = cache.by_type.get_mut(&removed.operation_type) {
                        type_ops.retain(|&op_id| op_id != *id);
                    }
                }
            }
        }
        
        // Add to cache
        let id = record.id;
        let path = record.file_path.clone();
        let op_type = record.operation_type;
        
        cache.operations.insert(id, record);
        cache.by_path.entry(path).or_insert_with(Vec::new).push(id);
        cache.by_type.entry(op_type).or_insert_with(Vec::new).push(id);
    }
    
    fn update_cache_outcome(
        &self,
        operation_id: Uuid,
        outcome: OutcomeRecord,
        completed_at: DateTime<Utc>,
    ) {
        let mut cache = self.cache.write().unwrap();
        if let Some(record) = cache.operations.get_mut(&operation_id) {
            record.outcome = outcome;
            record.completed_at = Some(completed_at);
        }
    }
    
    fn calculate_statistics(&self) -> Result<OperationStatistics> {
        // Get overall statistics
        let conn = self.pool.get()?;
        let (total, successful, failed, auto_executed, rollbacks, avg_confidence, avg_risk, avg_execution_time) = 
            conn.query_row(
                r#"
                SELECT 
                    COUNT(*) as total,
                    SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successful,
                    SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END) as failed,
                    SUM(CASE WHEN was_auto_executed = 1 THEN 1 ELSE 0 END) as auto_executed,
                    SUM(CASE WHEN rollback_required = 1 THEN 1 ELSE 0 END) as rollbacks,
                    AVG(confidence) as avg_confidence,
                    AVG(risk) as avg_risk,
                    AVG(execution_time_ms) as avg_execution_time
                FROM operations
                WHERE success IS NOT NULL
                "#,
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, Option<f64>>(5)?.map(|v| v as f32),
                        row.get::<_, Option<f64>>(6)?.map(|v| v as f32),
                        row.get::<_, Option<f64>>(7)?.map(|v| v as f32),
                    ))
                },
            )?;
        
        // Get per-type statistics
        let mut by_type = HashMap::new();
        
        for op_type in &[
            OperationType::Create,
            OperationType::Update,
            OperationType::Append,
            OperationType::Delete,
            OperationType::Rename,
        ] {
            let (count, type_successful, type_avg_confidence, type_avg_risk) = 
                conn.query_row(
                    r#"
                    SELECT 
                        COUNT(*) as count,
                        SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successful,
                        AVG(confidence) as avg_confidence,
                        AVG(risk) as avg_risk
                    FROM operations
                    WHERE operation_type = ?1 AND success IS NOT NULL
                    "#,
                    params![serde_json::to_string(op_type)?],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, Option<f64>>(2)?.map(|v| v as f32),
                            row.get::<_, Option<f64>>(3)?.map(|v| v as f32),
                        ))
                    },
                )?;
            
            if count > 0 {
                by_type.insert(*op_type, TypeStatistics {
                    count,
                    success_rate: type_successful as f32 / count as f32,
                    average_confidence: type_avg_confidence.unwrap_or(0.0),
                    average_risk: type_avg_risk.unwrap_or(0.0),
                    common_failure_reasons: Vec::new(), // TODO: Implement
                });
            }
        }
        
        Ok(OperationStatistics {
            total_operations: total,
            successful_operations: successful,
            failed_operations: failed,
            auto_executed_operations: auto_executed,
            rollbacks_required: rollbacks,
            average_confidence: avg_confidence.unwrap_or(0.0),
            average_risk: avg_risk.unwrap_or(0.0),
            average_execution_time_ms: avg_execution_time.unwrap_or(0.0),
            success_rate: if total > 0 { successful as f32 / total as f32 } else { 0.0 },
            auto_execution_rate: if total > 0 { auto_executed as f32 / total as f32 } else { 0.0 },
            rollback_rate: if total > 0 { rollbacks as f32 / total as f32 } else { 0.0 },
            by_type,
        })
    }
    
    fn row_to_record(&self, row: &rusqlite::Row) -> rusqlite::Result<OperationRecord> {
        let id: String = row.get("id")?;
        let operation_type: String = row.get("operation_type")?;
        let file_path: String = row.get("file_path")?;
        let details: String = row.get("details")?;
        let context: String = row.get("context")?;
        let prediction: String = row.get("prediction")?;
        let outcome: Option<String> = row.get("outcome")?;
        let created_at: String = row.get("created_at")?;
        let completed_at: Option<String> = row.get("completed_at")?;
        
        Ok(OperationRecord {
            id: Uuid::parse_str(&id).map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0, rusqlite::types::Type::Text, Box::new(e)
            ))?,
            operation_type: serde_json::from_str(&operation_type).map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                1, rusqlite::types::Type::Text, Box::new(e)
            ))?,
            file_path: PathBuf::from(file_path),
            details: serde_json::from_str(&details).map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                3, rusqlite::types::Type::Text, Box::new(e)
            ))?,
            context: serde_json::from_str(&context).map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                4, rusqlite::types::Type::Text, Box::new(e)
            ))?,
            prediction: serde_json::from_str(&prediction).map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                5, rusqlite::types::Type::Text, Box::new(e)
            ))?,
            outcome: outcome
                .map(|o| serde_json::from_str(&o).map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    6, rusqlite::types::Type::Text, Box::new(e)
                )))
                .transpose()?
                .unwrap_or_else(|| OutcomeRecord {
                    success: false,
                    error_message: None,
                    execution_time_ms: 0,
                    rollback_required: false,
                    rollback_time_ms: None,
                    user_feedback: None,
                    post_quality_metrics: None,
                }),
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    7, rusqlite::types::Type::Text, Box::new(e)
                ))?
                .with_timezone(&Utc),
            completed_at: completed_at.map(|dt| 
                DateTime::parse_from_rfc3339(&dt)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        8, rusqlite::types::Type::Text, Box::new(e)
                    ))
                    .map(|dt| dt.with_timezone(&Utc))
            ).transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_operation_history_creation() {
        // Test database creation and basic operations
        let db = OperationHistoryDatabase::new(":memory:").unwrap();
        
        // Test getting empty statistics
        let stats = db.get_statistics().unwrap();
        assert_eq!(stats.total_operations, 0);
    }
}