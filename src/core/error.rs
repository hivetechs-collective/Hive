//! Comprehensive error handling system for Hive AI
//!
//! This module provides a unified error type that covers all failure modes
//! across the entire Hive AI system, from configuration loading to consensus
//! execution and TUI operations.

use std::fmt;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for all Hive AI operations
///
/// This enum covers all possible error conditions that can occur throughout
/// the system, providing structured error information for debugging and
/// user-friendly error messages.
#[derive(Error, Debug)]
pub enum HiveError {
    // Configuration errors
    /// Configuration file not found
    #[error("Configuration file not found at {path}")]
    ConfigNotFound { path: PathBuf },
    
    /// Invalid configuration format
    #[error("Invalid configuration: {message}")]
    ConfigInvalid { message: String },
    
    /// Missing required configuration field
    #[error("Missing required configuration field: {field}")]
    ConfigMissingField { field: String },
    
    /// Configuration directory creation failed
    #[error("Failed to create configuration directory: {path}")]
    ConfigDirCreation { path: PathBuf },

    // Database errors
    /// Database connection failed
    #[error("Database connection failed: {message}")]
    DatabaseConnection { message: String },
    
    /// Database migration failed
    #[error("Database migration failed: {version} -> {target}")]
    DatabaseMigration { version: String, target: String },
    
    /// Database query failed
    #[error("Database query failed: {query}")]
    DatabaseQuery { query: String },
    
    /// Database schema validation failed
    #[error("Database schema validation failed: {message}")]
    DatabaseSchema { message: String },
    
    /// Cloudflare D1 sync failed
    #[error("Cloudflare D1 sync failed: {message}")]
    CloudflareSync { message: String },
    
    /// Database migration error
    #[error("Database migration error: {message}")]
    Migration { message: String },

    // Network and API errors
    /// HTTP request failed
    #[error("HTTP request failed: {url} - {status}")]
    HttpRequest { url: String, status: u16 },
    
    /// OpenRouter API error
    #[error("OpenRouter API error: {message}")]
    OpenRouterApi { message: String },
    
    /// API rate limit exceeded
    #[error("API rate limit exceeded for {provider}. Retry after: {retry_after}s")]
    RateLimitExceeded { provider: String, retry_after: u64 },
    
    /// Network timeout
    #[error("Network timeout after {seconds}s for {operation}")]
    NetworkTimeout { operation: String, seconds: u64 },
    
    /// Authentication failed
    #[error("Authentication failed for {service}: {message}")]
    AuthenticationFailed { service: String, message: String },
    
    /// Invalid API key
    #[error("Invalid API key for {service}")]
    InvalidApiKey { service: String },

    // File I/O errors
    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    
    /// File permission denied
    #[error("Permission denied accessing file: {path}")]
    FilePermissionDenied { path: PathBuf },
    
    /// Directory not found
    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: PathBuf },
    
    /// File parsing failed
    #[error("Failed to parse file {path}: {message}")]
    FileParsing { path: PathBuf, message: String },
    
    /// File writing failed
    #[error("Failed to write file {path}: {message}")]
    FileWriting { path: PathBuf, message: String },
    
    /// File reading failed
    #[error("Failed to read file {path}: {message}")]
    FileReading { path: PathBuf, message: String },

    // Consensus pipeline errors  
    /// Consensus generation stage failed
    #[error("Consensus generation failed for model {model}: {message}")]
    ConsensusGeneration { model: String, message: String },
    
    /// Consensus refinement stage failed
    #[error("Consensus refinement failed: {message}")]
    ConsensusRefinement { message: String },
    
    /// Consensus validation stage failed
    #[error("Consensus validation failed: {message}")]
    ConsensusValidation { message: String },
    
    /// Consensus curation stage failed
    #[error("Consensus curation failed: {message}")]
    ConsensusCuration { message: String },
    
    /// No consensus models available
    #[error("No consensus models available for stage: {stage}")]
    NoConsensusModels { stage: String },
    
    /// Consensus streaming failed
    #[error("Consensus streaming failed: {message}")]
    ConsensusStreaming { message: String },
    
    /// Consensus quality threshold not met
    #[error("Consensus quality threshold not met: {quality} < {threshold}")]
    ConsensusQuality { quality: f64, threshold: f64 },

    // Security and trust errors
    /// Directory not trusted
    #[error("Directory not trusted: {path}. Run 'hive trust {path}' to grant access")]
    DirectoryNotTrusted { path: PathBuf },
    
    /// File access denied by security policy
    #[error("File access denied by security policy: {path}")]
    SecurityPolicyDenied { path: PathBuf },
    
    /// Trust database corruption
    #[error("Trust database corruption detected: {message}")]
    TrustDatabaseCorruption { message: String },
    
    /// Invalid security token
    #[error("Invalid security token: {message}")]
    InvalidSecurityToken { message: String },
    
    /// Security audit failed
    #[error("Security audit failed for {operation}: {message}")]
    SecurityAuditFailed { operation: String, message: String },

    // TUI and interface errors
    /// Terminal too small for TUI
    #[error("Terminal too small for TUI interface: {width}x{height} (minimum: 120x30)")]
    TerminalTooSmall { width: u16, height: u16 },
    
    /// TUI rendering failed
    #[error("TUI rendering failed: {message}")]
    TuiRendering { message: String },
    
    /// TUI input handling failed
    #[error("TUI input handling failed: {message}")]
    TuiInput { message: String },
    
    /// TUI event processing failed
    #[error("TUI event processing failed: {event}")]
    TuiEventProcessing { event: String },
    
    /// Terminal mode switching failed
    #[error("Terminal mode switching failed: {message}")]
    TerminalModeSwitch { message: String },

    // Parsing and analysis errors
    /// AST parsing failed
    #[error("AST parsing failed for {file}: {message}")]
    AstParsing { file: PathBuf, message: String },
    
    /// Language detection failed
    #[error("Language detection failed for {file}")]
    LanguageDetection { file: PathBuf },
    
    /// Semantic analysis failed
    #[error("Semantic analysis failed: {message}")]
    SemanticAnalysis { message: String },
    
    /// Code transformation failed
    #[error("Code transformation failed: {message}")]
    CodeTransformation { message: String },

    // Memory and caching errors
    /// Memory limit exceeded
    #[error("Memory limit exceeded: {current}MB > {limit}MB")]
    MemoryLimitExceeded { current: u64, limit: u64 },
    
    /// Cache corruption detected
    #[error("Cache corruption detected in {cache_type}: {message}")]
    CacheCorruption { cache_type: String, message: String },
    
    /// Cache eviction failed
    #[error("Cache eviction failed: {message}")]
    CacheEviction { message: String },

    // Repository and Git errors
    /// Git repository not found
    #[error("Git repository not found in {path}")]
    GitRepoNotFound { path: PathBuf },
    
    /// Git operation failed
    #[error("Git operation '{operation}' failed: {message}")]
    GitOperation { operation: String, message: String },
    
    /// Repository analysis failed
    #[error("Repository analysis failed: {message}")]
    RepositoryAnalysis { message: String },

    // Planning and task management errors
    /// Planning operation failed
    #[error("Planning operation failed: {0}")]
    Planning(String),

    // Integration errors
    /// MCP server connection failed
    #[error("MCP server connection failed: {server}")]
    McpConnection { server: String },
    
    /// LSP server initialization failed
    #[error("LSP server initialization failed: {message}")]
    LspInitialization { message: String },
    
    /// IDE integration failed
    #[error("IDE integration failed for {ide}: {message}")]
    IdeIntegration { ide: String, message: String },

    // Security errors
    /// Security operation failed
    #[error("Security error: {message}")]
    Security { message: String },
    
    // System and environment errors
    /// Environment variable not set
    #[error("Required environment variable not set: {var}")]
    EnvVarNotSet { var: String },
    
    /// System resource unavailable
    #[error("System resource unavailable: {resource}")]
    SystemResourceUnavailable { resource: String },
    
    /// License validation failed
    #[error("License validation failed: {message}")]
    LicenseValidation { message: String },
    
    /// Feature not available
    #[error("Feature not available: {feature}")]
    FeatureNotAvailable { feature: String },

    // Generic errors with context
    /// Internal error with context
    #[error("Internal error in {context}: {message}")]
    Internal { context: String, message: String },
    
    /// Operation timeout
    #[error("Operation '{operation}' timed out after {seconds}s")]
    Timeout { operation: String, seconds: u64 },
    
    /// Resource exhausted
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
}

impl HiveError {
    /// Create a configuration error for a missing file
    pub fn config_not_found(path: impl Into<PathBuf>) -> Self {
        Self::ConfigNotFound { path: path.into() }
    }
    
    /// Create a configuration error for invalid format
    pub fn config_invalid(message: impl Into<String>) -> Self {
        Self::ConfigInvalid { message: message.into() }
    }
    
    /// Create a database connection error
    pub fn database_connection(message: impl Into<String>) -> Self {
        Self::DatabaseConnection { message: message.into() }
    }
    
    /// Create an HTTP request error
    pub fn http_request(url: impl Into<String>, status: u16) -> Self {
        Self::HttpRequest { url: url.into(), status }
    }
    
    /// Create a file not found error
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: path.into() }
    }
    
    /// Create a directory not trusted error
    pub fn directory_not_trusted(path: impl Into<PathBuf>) -> Self {
        Self::DirectoryNotTrusted { path: path.into() }
    }
    
    /// Create a terminal too small error
    pub fn terminal_too_small(width: u16, height: u16) -> Self {
        Self::TerminalTooSmall { width, height }
    }
    
    /// Create an internal error with context
    pub fn internal(context: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Internal { 
            context: context.into(), 
            message: message.into() 
        }
    }
    
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors - can retry or user can fix
            Self::NetworkTimeout { .. } |
            Self::RateLimitExceeded { .. } |
            Self::DirectoryNotTrusted { .. } |
            Self::ConfigNotFound { .. } |
            Self::FileNotFound { .. } |
            Self::TerminalTooSmall { .. } => true,
            
            // Non-recoverable errors - system or logic errors
            Self::DatabaseSchema { .. } |
            Self::TrustDatabaseCorruption { .. } |
            Self::CacheCorruption { .. } |
            Self::MemoryLimitExceeded { .. } |
            Self::Internal { .. } => false,
            
            // Most other errors are potentially recoverable
            _ => true,
        }
    }
    
    /// Get the error category for logging and metrics
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::ConfigNotFound { .. } |
            Self::ConfigInvalid { .. } |
            Self::ConfigMissingField { .. } |
            Self::ConfigDirCreation { .. } => ErrorCategory::Configuration,
            
            Self::DatabaseConnection { .. } |
            Self::DatabaseMigration { .. } |
            Self::DatabaseQuery { .. } |
            Self::DatabaseSchema { .. } |
            Self::CloudflareSync { .. } |
            Self::Migration { .. } => ErrorCategory::Database,
            
            Self::HttpRequest { .. } |
            Self::OpenRouterApi { .. } |
            Self::RateLimitExceeded { .. } |
            Self::NetworkTimeout { .. } |
            Self::AuthenticationFailed { .. } |
            Self::InvalidApiKey { .. } => ErrorCategory::Network,
            
            Self::FileNotFound { .. } |
            Self::FilePermissionDenied { .. } |
            Self::DirectoryNotFound { .. } |
            Self::FileParsing { .. } |
            Self::FileWriting { .. } |
            Self::FileReading { .. } => ErrorCategory::FileSystem,
            
            Self::ConsensusGeneration { .. } |
            Self::ConsensusRefinement { .. } |
            Self::ConsensusValidation { .. } |
            Self::ConsensusCuration { .. } |
            Self::NoConsensusModels { .. } |
            Self::ConsensusStreaming { .. } |
            Self::ConsensusQuality { .. } => ErrorCategory::Consensus,
            
            Self::DirectoryNotTrusted { .. } |
            Self::SecurityPolicyDenied { .. } |
            Self::TrustDatabaseCorruption { .. } |
            Self::InvalidSecurityToken { .. } |
            Self::SecurityAuditFailed { .. } => ErrorCategory::Security,
            
            Self::TerminalTooSmall { .. } |
            Self::TuiRendering { .. } |
            Self::TuiInput { .. } |
            Self::TuiEventProcessing { .. } |
            Self::TerminalModeSwitch { .. } => ErrorCategory::Interface,
            
            Self::Planning(_) => ErrorCategory::Planning,
            
            _ => ErrorCategory::System,
        }
    }
    
    /// Get user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            Self::DirectoryNotTrusted { path } => {
                format!(
                    "Directory not trusted: {}\n\nTo grant access, run: hive trust \"{}\"",
                    path.display(),
                    path.display()
                )
            }
            Self::ConfigNotFound { path } => {
                format!(
                    "Configuration file not found: {}\n\nRun 'hive setup' to create initial configuration.",
                    path.display()
                )
            }
            Self::TerminalTooSmall { width, height } => {
                format!(
                    "Terminal too small for TUI interface: {}x{} (minimum: 120x30)\n\nPlease resize your terminal or use CLI mode.",
                    width, height
                )
            }
            Self::RateLimitExceeded { provider, retry_after } => {
                format!(
                    "Rate limit exceeded for {}\n\nPlease wait {} seconds before retrying.",
                    provider, retry_after
                )
            }
            _ => self.to_string(),
        }
    }
}

/// Error category for metrics and logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    Configuration,
    Database,
    Network,
    FileSystem,
    Consensus,
    Security,
    Interface,
    Planning,
    System,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration => write!(f, "configuration"),
            Self::Database => write!(f, "database"),
            Self::Network => write!(f, "network"),
            Self::FileSystem => write!(f, "filesystem"),
            Self::Consensus => write!(f, "consensus"),
            Self::Security => write!(f, "security"),
            Self::Interface => write!(f, "interface"),
            Self::Planning => write!(f, "planning"),
            Self::System => write!(f, "system"),
        }
    }
}

// Implement conversion traits for common error types
impl From<std::io::Error> for HiveError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Self::FileNotFound { 
                path: PathBuf::from("unknown") 
            },
            io::ErrorKind::PermissionDenied => Self::FilePermissionDenied { 
                path: PathBuf::from("unknown") 
            },
            _ => Self::Internal { 
                context: "io".to_string(), 
                message: err.to_string() 
            },
        }
    }
}

impl From<serde_json::Error> for HiveError {
    fn from(err: serde_json::Error) -> Self {
        Self::ConfigInvalid { 
            message: format!("JSON parsing error: {}", err) 
        }
    }
}

impl From<toml::de::Error> for HiveError {
    fn from(err: toml::de::Error) -> Self {
        Self::ConfigInvalid { 
            message: format!("TOML parsing error: {}", err) 
        }
    }
}

impl From<anyhow::Error> for HiveError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal { 
            context: "anyhow".to_string(), 
            message: err.to_string() 
        }
    }
}

// Temporarily disabled until reqwest is available in minimal build
/*
impl From<reqwest::Error> for HiveError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::NetworkTimeout { 
                operation: "HTTP request".to_string(), 
                seconds: 30 
            }
        } else if let Some(status) = err.status() {
            Self::HttpRequest { 
                url: err.url().map(|u| u.to_string()).unwrap_or_default(), 
                status: status.as_u16() 
            }
        } else {
            Self::Internal { 
                context: "http".to_string(), 
                message: err.to_string() 
            }
        }
    }
}
*/

impl From<rusqlite::Error> for HiveError {
    fn from(err: rusqlite::Error) -> Self {
        Self::DatabaseQuery { 
            query: format!("SQLite error: {}", err) 
        }
    }
}

impl From<tokio::task::JoinError> for HiveError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Internal { 
            context: "task".to_string(), 
            message: err.to_string() 
        }
    }
}

impl From<tree_sitter::LanguageError> for HiveError {
    fn from(err: tree_sitter::LanguageError) -> Self {
        Self::Internal { 
            context: "tree-sitter".to_string(), 
            message: format!("Language error: {:?}", err) 
        }
    }
}

impl From<std::str::Utf8Error> for HiveError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Internal { 
            context: "utf8".to_string(), 
            message: err.to_string() 
        }
    }
}

impl From<dialoguer::Error> for HiveError {
    fn from(err: dialoguer::Error) -> Self {
        match err {
            dialoguer::Error::IO(io_err) => Self::from(io_err),
        }
    }
}

impl From<std::env::VarError> for HiveError {
    fn from(err: std::env::VarError) -> Self {
        match err {
            std::env::VarError::NotPresent => Self::EnvVarNotSet { 
                var: "unknown".to_string() 
            },
            std::env::VarError::NotUnicode(_) => Self::Internal { 
                context: "env".to_string(), 
                message: "Environment variable contains invalid Unicode".to_string() 
            },
        }
    }
}

impl From<reqwest::Error> for HiveError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::NetworkTimeout { 
                operation: "HTTP request".to_string(), 
                seconds: 30 
            }
        } else if let Some(status) = err.status() {
            Self::HttpRequest { 
                url: err.url().map(|u| u.to_string()).unwrap_or_default(), 
                status: status.as_u16() 
            }
        } else {
            Self::Internal { 
                context: "network".to_string(), 
                message: err.to_string() 
            }
        }
    }
}

// Generic string error conversion (for parse errors, etc)
impl From<String> for HiveError {
    fn from(err: String) -> Self {
        Self::Internal { 
            context: "parse".to_string(), 
            message: err 
        }
    }
}

impl From<&str> for HiveError {
    fn from(err: &str) -> Self {
        Self::Internal { 
            context: "parse".to_string(), 
            message: err.to_string() 
        }
    }
}

impl From<chrono::ParseError> for HiveError {
    fn from(err: chrono::ParseError) -> Self {
        Self::Internal { 
            context: "time".to_string(), 
            message: format!("Time parsing error: {}", err) 
        }
    }
}

impl From<std::num::ParseIntError> for HiveError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::Internal { 
            context: "parse".to_string(), 
            message: format!("Integer parsing error: {}", err) 
        }
    }
}

impl From<std::num::ParseFloatError> for HiveError {
    fn from(err: std::num::ParseFloatError) -> Self {
        Self::Internal { 
            context: "parse".to_string(), 
            message: format!("Float parsing error: {}", err) 
        }
    }
}

impl From<uuid::Error> for HiveError {
    fn from(err: uuid::Error) -> Self {
        Self::Internal { 
            context: "uuid".to_string(), 
            message: format!("UUID error: {}", err) 
        }
    }
}

/// Result type alias for Hive operations
pub type Result<T> = std::result::Result<T, HiveError>;
pub type HiveResult<T> = std::result::Result<T, HiveError>;

/// Helper macro for creating internal errors with context
#[macro_export]
macro_rules! internal_error {
    ($context:expr, $msg:expr) => {
        $crate::core::error::HiveError::internal($context, $msg)
    };
    ($context:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::core::error::HiveError::internal($context, format!($fmt, $($arg)*))
    };
}

/// Helper macro for creating configuration errors
#[macro_export]
macro_rules! config_error {
    ($msg:expr) => {
        $crate::core::error::HiveError::config_invalid($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::core::error::HiveError::config_invalid(format!($fmt, $($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = HiveError::file_not_found("/tmp/test.txt");
        assert!(matches!(err, HiveError::FileNotFound { .. }));
        assert_eq!(err.category(), ErrorCategory::FileSystem);
    }

    #[test]
    fn test_error_recoverability() {
        let recoverable = HiveError::directory_not_trusted("/tmp");
        assert!(recoverable.is_recoverable());
        
        let non_recoverable = HiveError::internal("test", "system failure");
        assert!(!non_recoverable.is_recoverable());
    }

    #[test]
    fn test_user_messages() {
        let err = HiveError::terminal_too_small(80, 24);
        let msg = err.user_message();
        assert!(msg.contains("Terminal too small"));
        assert!(msg.contains("120x30"));
    }
}