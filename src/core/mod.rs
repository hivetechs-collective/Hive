//! Core functionality for semantic code understanding

pub mod api_keys;
pub mod ast;
pub mod semantic;
pub mod context;
pub mod config;
pub mod error;
pub mod logging;
pub mod temporal;

// Database implementation
pub mod database;
pub mod database_simple;
pub mod database_working;

// These modules require additional dependencies not in minimal build
pub mod security;
pub mod schema;
// pub mod trust_dialog;
// pub mod file_access;
// pub mod security_config;
pub mod migrations_simple;

// Enhanced memory and analytics
pub mod memory;
pub mod analytics;

// Auto-update mechanism
pub mod updater;

// License management
pub mod license;

// Usage tracking
pub mod usage_tracker;

// Profile management
pub mod profiles;

// Migration tool
pub mod migrator;

// Uninstaller
pub mod uninstaller;

// Performance optimization
pub mod performance;

pub use ast::{
    AstEngine, Position, AstNode, NodeMetadata, Symbol, SymbolKind, 
    ParseResult, ParseError, ErrorSeverity, ImportInfo, CodeMetrics
};
pub use semantic::SemanticIndex;
pub use context::ContextBuilder;
pub use config::{
    HiveConfig, Config, load_config, get_config, set_config_value, get_config_value,
    reset_config, save_config, create_default_config, init_config,
};
pub use error::{HiveError, ErrorCategory, Result, HiveResult};
pub use logging::{LoggingConfig, initialize_logging, initialize_default_logging, PerfTimer, ErrorTracker, AnalyticsLogger};
pub use temporal::{TemporalContext, BusinessHours, TimeContext, TimeOfDay};
pub use database_simple::{
    DatabaseConfig, DatabaseHealthStatus, DatabaseStatistics,
    initialize_database, get_health_status, get_statistics, generate_id, current_timestamp
};
pub use database::get_database;

// Re-export Database type for convenience
pub use database_simple::Database;
pub use migrations_simple::{
    MigrationManager, Migration, MigrationResult, MigrationStatus, IntegrityReport,
    initialize_migrations, get_migration_status
};
pub use security::{
    SecurityContext, TrustLevel, TrustDecision, SecurityEvent, SecurityEventType,
    initialize_security, get_security_context, is_trusted,
    check_read_access, check_write_access, check_delete_access, check_directory_access
};
pub use updater::{
    AutoUpdater, UpdateInfo, UpdateConfig, UpdateChannel, ReleaseInfo
};
pub use migrator::{
    HiveMigrator, MigrationPlan, MigrationResult as HiveMigrationResult, MigrationStep, 
    TypeScriptHiveData, LegacyConversation, LegacyConfig
};
pub use uninstaller::{
    HiveUninstaller, UninstallPlan, UninstallResult, UninstallComponent
};
pub use memory::{MemorySystem, MemoryManager};
pub use license::{LicenseManager, LicenseInfo, LicenseStatus};

use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Type aliases for compatibility with other modules
pub type LanguageType = Language;

/// Represents a workspace containing code files
#[derive(Debug)]
pub struct Workspace {
    /// Root directory of the workspace
    pub root: PathBuf,
    /// Cached file information  
    pub files: Arc<Mutex<HashMap<PathBuf, FileInfo>>>,
    /// Git repository (if available)
    pub git: Option<GitRepo>,
}

/// Information about a file in the workspace
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// File content
    pub content: Arc<String>,
    /// Last modification time
    pub last_modified: SystemTime,
    /// Detected language
    pub language: Language,
    /// Path to the file
    pub path: PathBuf,
    /// Code metrics for the file
    pub metrics: Option<CodeMetrics>,
}

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    Cpp,
    C,
    Ruby,
    PHP,
    Swift,
    Unknown,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::JavaScript => "javascript",
            Language::Python => "python",
            Language::Go => "go",
            Language::Java => "java",
            Language::Cpp => "cpp",
            Language::C => "c",
            Language::Ruby => "ruby",
            Language::PHP => "php",
            Language::Swift => "swift",
            Language::Unknown => "text",
        }
    }
}

/// Git repository information
#[derive(Debug)]
pub struct GitRepo {
    /// Repository root
    pub root: PathBuf,
}

impl Workspace {
    /// Create a new workspace
    pub async fn new(root: PathBuf) -> Result<Self> {
        let git = if root.join(".git").exists() {
            Some(GitRepo { root: root.clone() })
        } else {
            None
        };
        
        Ok(Self {
            root,
            files: Arc::new(Mutex::new(HashMap::new())),
            git,
        })
    }
    
    /// Get or load file information
    pub async fn get_file(&self, path: &PathBuf) -> Result<Arc<FileInfo>> {
        // Check security permissions first
        check_read_access(path)?;
        
        {
            let files = self.files.lock().unwrap();
            if let Some(info) = files.get(path) {
                // Check if file needs reloading
                let metadata = tokio::fs::metadata(path).await?;
                if metadata.modified()? <= info.last_modified {
                    return Ok(Arc::new(info.clone()));
                }
            }
        }
        
        // Load file
        let content = tokio::fs::read_to_string(path).await?;
        let language = detect_language(path);
        let info = FileInfo {
            content: Arc::new(content),
            last_modified: tokio::fs::metadata(path).await?.modified()?,
            language,
            path: path.clone(),
            metrics: None, // Metrics can be computed lazily if needed
        };
        
        let mut files = self.files.lock().unwrap();
        files.insert(path.clone(), info.clone());
        Ok(Arc::new(info))
    }
}

/// Detect the programming language of a file
pub fn detect_language(path: &Path) -> Language {
    match path.extension().and_then(|s| s.to_str()) {
        Some("rs") => Language::Rust,
        Some("ts") | Some("tsx") => Language::TypeScript,
        Some("js") | Some("jsx") => Language::JavaScript,
        Some("py") => Language::Python,
        Some("go") => Language::Go,
        Some("java") => Language::Java,
        Some("cpp") | Some("cc") | Some("cxx") => Language::Cpp,
        Some("c") | Some("h") => Language::C,
        Some("rb") => Language::Ruby,
        Some("php") => Language::PHP,
        Some("swift") => Language::Swift,
        _ => Language::Unknown,
    }
}