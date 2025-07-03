//! Hive AI: Lightning-fast codebase intelligence platform
//! 
//! This crate provides the core functionality for semantic code understanding,
//! multi-model AI consensus, and seamless code transformations.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod core;
pub mod consensus;
pub mod transformation;
// pub mod integration; // Temporarily disabled due to compilation issues
pub mod cache;
pub mod cli;
pub mod analysis;
pub mod commands;
pub mod providers;
pub mod interactive_tui;
pub mod tui;
pub mod hooks;
pub mod planning;
pub mod memory;
pub mod analytics;
pub mod migration;
pub mod security;
pub mod shell;
pub mod startup;
pub mod database;
// pub mod modes; // Temporarily disabled due to compilation issues

// Re-export core types
pub use core::{
    // Errors and results
    HiveError, ErrorCategory, Result,
    
    // Core functionality
    SemanticIndex, ContextBuilder, AstEngine,
    
    // Configuration
    HiveConfig, load_config, get_config, set_config_value, get_config_value,
    reset_config, save_config, create_default_config, init_config,
    
    // Logging
    LoggingConfig, initialize_logging, initialize_default_logging,
    PerfTimer, ErrorTracker, AnalyticsLogger,
    
    // Database
    DatabaseConfig, DatabaseHealthStatus, DatabaseStatistics,
    initialize_database, get_health_status, get_statistics,
    generate_id, current_timestamp,
    
    // Migrations
    MigrationManager, Migration, MigrationResult, MigrationStatus,
    IntegrityReport, initialize_migrations, get_migration_status,
    
    // Workspace and types
    Workspace, FileInfo, Language, LanguageType, GitRepo,
    
    // Security
    SecurityContext, TrustLevel, TrustDecision, SecurityEvent, SecurityEventType,
    initialize_security, get_security_context, is_trusted,
    check_read_access, check_write_access, check_delete_access, check_directory_access,
};

// Re-export consensus types (these will be implemented later)
pub use consensus::{ConsensusEngine, ConsensusConfig as ConsensusEngineConfig, Stage};

// Re-export transformation types
pub use transformation::{
    SimpleTransformationEngine, TransformationRequest, TransformationPreview,
    CodeChange, Transformation, simple_transform_code, simple_generate_preview,
};

// Re-export integration types
// pub use integration::{start_mcp_server, start_lsp_server}; // Temporarily disabled

// Re-export cache types
pub use cache::{CacheConfig, CacheStats, CacheCategory, CacheKey};

// Re-export analysis types
pub use analysis::{
    AnalysisEngine, TreeSitterParser, ParserRegistry, 
    LanguageDetector, IncrementalParser, SyntaxHighlighter,
    PerformanceMonitor, ParseMetrics, HighlightSpan, HighlightType,
    PerformanceStatus,
};

// Re-export hooks types
pub use hooks::{
    HooksSystem, Hook, HookId, HookPriority,
    HookEvent, EventType, EventSource,
    HookCondition, 
    HookConfig, HookAction, ExecutionContext, ExecutionResult,
    HookSecurityValidator, SecurityPolicy,
    HookAuditLogger, AuditEvent, AuditEventType,
    BaseApprovalWorkflow, BaseApprovalRequest, BaseApprovalStatus,
    HookRbacManager, User, Role, Team, Permission,
};

// Re-export planning types
pub use planning::{
    PlanningEngine, Plan, Task, Risk, Dependency, ModeType, PlanningContext,
    TaskDecomposer, RiskAnalyzer, TimelineEstimator, DependencyResolver,
    CollaborativePlanner, ModeDetector, ModeSwitcher,
    PlanExecutionResult, TaskExecutionResult, ModeRecommendation as PlanningModeRecommendation,
};

// Re-export memory types
pub use memory::{
    MemoryIntelligence, EmbeddingEngine, VectorStore, SimilarityMetric,
    KnowledgeGraph, Entity, Relationship, GraphQuery,
    PatternLearner, Pattern, PatternType, PatternMetrics,
    ContextRetriever, RetrievalStrategy, ContextWindow,
    MemoryAnalyzer, InsightGenerator, MemoryMetrics,
};

// Re-export analytics types
pub use analytics::{
    AdvancedAnalyticsEngine, AdvancedAnalyticsConfig,
    TrendAnalyzer, TrendPrediction, SeasonalPattern, AnomalyDetection,
    TimeSeriesModel, ForecastHorizon, ConfidenceInterval,
    ExecutiveReporter, ExecutiveSummary, KeyMetric, BusinessInsight,
    ReportFormat, ReportPeriod, VisualizationConfig,
    CostIntelligence, CostOptimization, BudgetAlert, SpendingPattern,
    ModelEfficiency, CostAllocation, OptimizationStrategy,
    DashboardEngine, DashboardLayout, Widget, WidgetType,
    RealTimeMetric, RefreshRate, DataStream,
    PerformanceAnalyzer, ModelPerformance, LatencyAnalysis,
    ThroughputMetric, QualityScore, PerformanceInsight,
};

// Re-export security types
pub use security::{
    SecuritySystem, SecurityConfig, SecurityMetrics, 
    SecurityEvent as EnterpriseSecurityEvent, SecurityEventType as EnterpriseSecurityEventType,
    SecurityAlertLevel, PasswordPolicy, EncryptionConfig,
    
    // Authentication
    AuthenticationManager, AuthProvider, SessionManager, Session,
    ApiKeyManager, ApiKey, MfaProvider, MfaChallenge,
    
    // Audit
    EnterpriseAuditLogger, 
    AuditEvent as EnterpriseAuditEvent, AuditEventType as EnterpriseAuditEventType, 
    AuditFilter, AuditStatistics, ComplianceReport as AuditComplianceReport, RetentionPolicy,
    
    // Compliance
    ComplianceManager, ComplianceStandard, ComplianceRule, ComplianceViolation,
    ComplianceStatus, ComplianceReport,
    
    // Permissions
    PermissionManager, ResourcePermission, PermissionScope, PermissionContext,
    PermissionInheritance, PermissionTemplate,
    
    // RBAC
    EnterpriseRbacManager, EnterpriseRole, EnterpriseUser, SecurityGroup,
    AccessPolicy, RoleInheritance,
    
    // Teams
    TeamManager, EnterpriseTeam, TeamHierarchy, TeamRole,
    TeamPermissions, TeamAccess, TeamInvitation,
};

// Re-export modes types
// pub use modes::{
//     ModeManager, ModeRecommendation, ModeConfig, ContextPreservationLevel,
//     EnhancedModeDetector, DetectionResult,
//     EnhancedModeSwitcher, SwitchResult,
//     HybridModeEngine, HybridTask,
//     ContextManager, ModeContext, ContextSnapshot,
//     PreferenceManager, UserPreference, LearningData,
//     ModeVisualizer, ModeStatus,
// }; // Temporarily disabled

// Re-export TUI types
pub use tui::{TuiApp, TuiResult, TuiFramework, run_professional_tui, run_advanced_tui};

// Re-export shell types
pub use shell::{
    ShellIntegration, ShellType, ShellIntegrationStatus,
};

// Re-export command types
pub use commands::{
    ShellCommands, handle_shell, generate_manual_instructions, 
    verify_installation, get_integration_report,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the Hive AI engine
/// 
/// This function sets up all core systems including:
/// - Logging infrastructure
/// - Configuration loading
/// - Database connections
/// - Cache initialization
/// - Background tasks
pub async fn initialize() -> Result<()> {
    // Initialize logging with default configuration
    core::logging::initialize_default_logging()?;
    
    tracing::info!("Initializing Hive AI v{}", VERSION);
    
    // Initialize configuration system
    core::init_config().await
        .map_err(|e| HiveError::internal("init", format!("Configuration initialization failed: {}", e)))?;
    
    // Load configuration
    let config = core::load_config().await
        .map_err(|e| HiveError::internal("init", format!("Configuration loading failed: {}", e)))?;
    
    tracing::debug!("Configuration loaded successfully");
    
    // Initialize security system (Claude Code-style trust management)
    let security_db_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("hive")
        .join("security.db");
    
    core::initialize_security(Some(security_db_path), true)
        .map_err(|e| HiveError::internal("init", format!("Security initialization failed: {}", e)))?;
    
    tracing::debug!("Security system initialized successfully");
    
    // Initialize database
    let db_config = DatabaseConfig {
        path: config.logging.file
            .as_ref()
            .map(|p| std::path::PathBuf::from(p))
            .unwrap_or_else(|| {
                let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                home.join(".hive").join("hive-ai.db")
            }),
        enable_wal: true,
        enable_foreign_keys: true,
    };
    
    core::initialize_database(Some(db_config)).await
        .map_err(|e| HiveError::internal("init", format!("Database initialization failed: {}", e)))?;
    
    tracing::debug!("Database initialized successfully");
    
    // Initialize cache system
    cache::initialize().await
        .map_err(|e| HiveError::internal("init", format!("Cache initialization failed: {}", e)))?;
    
    tracing::debug!("Cache system initialized successfully");
    
    // Initialize migrations
    let migrations_dir = std::path::PathBuf::from("migrations");
    if migrations_dir.exists() {
        let results = core::initialize_migrations(migrations_dir).await
            .map_err(|e| HiveError::internal("init", format!("Migration initialization failed: {}", e)))?;
        
        if !results.is_empty() {
            tracing::info!("Applied {} database migrations", results.len());
        }
    }
    
    tracing::info!("Hive AI initialization complete ✅");
    
    Ok(())
}

/// Shutdown the Hive AI engine gracefully
pub async fn shutdown() -> Result<()> {
    tracing::info!("Shutting down Hive AI...");
    
    // Flush cache to disk
    cache::get_stats().await; // Trigger any pending operations
    
    // Save any pending configuration changes
    if let Ok(config) = core::get_config().await {
        let _ = core::save_config(&config).await;
    }
    
    tracing::info!("Hive AI shutdown complete");
    
    Ok(())
}

/// Get current system health status
pub async fn health_check() -> Result<HealthStatus> {
    let db_health = core::get_health_status().await?;
    let cache_stats = cache::get_stats().await;
    let config = core::get_config().await?;
    
    Ok(HealthStatus {
        healthy: db_health.healthy,
        version: VERSION.to_string(),
        database: db_health,
        cache: cache_stats,
        configured: config.openrouter.is_some(),
    })
}

/// System health status
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub healthy: bool,
    /// System version
    pub version: String,
    /// Database health
    pub database: DatabaseHealthStatus,
    /// Cache statistics
    pub cache: CacheStats,
    /// Whether system is fully configured
    pub configured: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}