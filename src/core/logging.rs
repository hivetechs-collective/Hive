//! Structured logging infrastructure for Hive AI
//!
//! This module provides comprehensive logging capabilities including:
//! - Console logging with colors and formatting
//! - File logging with rotation to ~/.hive/logs/
//! - Structured JSON logging for analytics
//! - Performance monitoring and metrics
//! - Error tracking and debugging

use is_terminal::IsTerminal;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::Once;
use tracing::{Level, Span};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

use crate::core::error::{HiveError, Result};

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Console logging level
    pub console_level: Level,
    /// File logging level
    pub file_level: Level,
    /// JSON logging level
    pub json_level: Level,
    /// Log directory path
    pub log_dir: PathBuf,
    /// Enable console colors
    pub console_colors: bool,
    /// Enable file logging
    pub file_logging: bool,
    /// Enable JSON logging
    pub json_logging: bool,
    /// Log file rotation
    pub rotation: LogRotation,
    /// Maximum log file size (MB)
    pub max_file_size: u64,
    /// Number of log files to keep
    pub max_files: usize,
}

/// Log rotation strategy
#[derive(Debug, Clone, Copy)]
pub enum LogRotation {
    /// Rotate daily
    Daily,
    /// Rotate hourly
    Hourly,
    /// Rotate by size
    Size,
    /// Never rotate
    Never,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let log_dir = home_dir.join(".hive").join("logs");

        Self {
            console_level: Level::INFO,
            file_level: Level::DEBUG,
            json_level: Level::WARN,
            log_dir,
            console_colors: true,
            file_logging: true,
            json_logging: true,
            rotation: LogRotation::Daily,
            max_file_size: 50, // 50MB
            max_files: 10,
        }
    }
}

/// Global logging state
static LOGGING_INITIALIZED: Once = Once::new();

/// Initialize the logging system
pub fn initialize_logging(config: LoggingConfig) -> Result<()> {
    LOGGING_INITIALIZED.call_once(|| {
        if let Err(e) = setup_logging_internal(config) {
            eprintln!("Failed to initialize logging: {}", e);
        }
    });
    Ok(())
}

/// Initialize logging with default configuration
pub fn initialize_default_logging() -> Result<()> {
    initialize_logging(LoggingConfig::default())
}

/// Internal logging setup implementation
fn setup_logging_internal(config: LoggingConfig) -> Result<()> {
    // Create log directory if it doesn't exist
    if config.file_logging || config.json_logging {
        fs::create_dir_all(&config.log_dir).map_err(|_e| HiveError::ConfigDirCreation {
            path: config.log_dir.clone(),
        })?;
    }

    // Create environment filter
    let env_filter = EnvFilter::builder()
        .with_default_directive(config.console_level.into())
        .from_env_lossy()
        .add_directive("hive_ai=trace".parse().unwrap())
        .add_directive("sqlx=warn".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap());

    let registry = tracing_subscriber::registry().with(env_filter);

    // Console layer with colors and formatting
    let console_layer = fmt::layer()
        .with_ansi(
            config.console_colors && {
                use is_terminal::IsTerminal;
                use std::io::stdout;
                stdout().is_terminal()
            },
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(std::io::stdout)
        .with_filter(tracing_subscriber::filter::LevelFilter::from_level(
            config.console_level,
        ));

    let mut layers = vec![Box::new(console_layer) as Box<dyn Layer<_> + Send + Sync>];

    // File logging layer
    if config.file_logging {
        let file_appender = match config.rotation {
            LogRotation::Daily => {
                RollingFileAppender::new(Rotation::DAILY, &config.log_dir, "hive.log")
            }
            LogRotation::Hourly => {
                RollingFileAppender::new(Rotation::HOURLY, &config.log_dir, "hive.log")
            }
            LogRotation::Size | LogRotation::Never => {
                RollingFileAppender::new(Rotation::NEVER, &config.log_dir, "hive.log")
            }
        };

        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::FULL)
            .with_writer(file_appender)
            .with_filter(tracing_subscriber::filter::LevelFilter::from_level(
                config.file_level,
            ));

        layers.push(Box::new(file_layer));
    }

    // JSON logging layer for analytics
    if config.json_logging {
        let json_appender = RollingFileAppender::new(
            match config.rotation {
                LogRotation::Daily => Rotation::DAILY,
                LogRotation::Hourly => Rotation::HOURLY,
                _ => Rotation::NEVER,
            },
            &config.log_dir,
            "hive-analytics.jsonl",
        );

        let json_layer = fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_writer(json_appender)
            .with_filter(tracing_subscriber::filter::LevelFilter::from_level(
                config.json_level,
            ));

        layers.push(Box::new(json_layer));
    }

    // Apply all layers
    registry.with(layers).try_init().map_err(|e| {
        HiveError::internal("logging", format!("Failed to initialize tracing: {}", e))
    })?;

    // Log initialization success
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %config.log_dir.display(),
        console_level = %config.console_level,
        file_level = %config.file_level,
        json_level = %config.json_level,
        "Hive AI logging initialized"
    );

    Ok(())
}

/// Performance timing utilities
pub struct PerfTimer {
    name: String,
    start: std::time::Instant,
    span: Span,
}

impl PerfTimer {
    /// Start a new performance timer
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let span = tracing::info_span!("perf_timer", operation = %name);

        Self {
            name,
            start: std::time::Instant::now(),
            span,
        }
    }

    /// Record a checkpoint
    pub fn checkpoint(&self, label: &str) {
        let elapsed = self.start.elapsed();
        tracing::debug!(
            parent: &self.span,
            operation = %self.name,
            checkpoint = label,
            elapsed_ms = elapsed.as_millis() as f64,
            "Performance checkpoint"
        );
    }

    /// Finish the timer and log results
    pub fn finish(self) {
        let elapsed = self.start.elapsed();
        tracing::info!(
            parent: &self.span,
            operation = %self.name,
            elapsed_ms = elapsed.as_millis() as f64,
            elapsed_us = elapsed.as_micros() as f64,
            "Performance timing completed"
        );
    }
}

/// Error tracking utilities
pub struct ErrorTracker;

impl ErrorTracker {
    /// Log an error with full context
    pub fn log_error(error: &HiveError, context: Option<&str>) {
        let category = error.category();
        let recoverable = error.is_recoverable();

        tracing::error!(
            error = %error,
            error_category = %category,
            recoverable = recoverable,
            context = context.unwrap_or("unknown"),
            "Hive error occurred"
        );
    }

    /// Log a warning with context
    pub fn log_warning(message: &str, context: Option<&str>) {
        tracing::warn!(
            message = message,
            context = context.unwrap_or("unknown"),
            "Hive warning"
        );
    }

    /// Log recoverable error that was handled
    pub fn log_recovered_error(error: &HiveError, recovery_action: &str) {
        tracing::info!(
            error = %error,
            error_category = %error.category(),
            recovery_action = recovery_action,
            "Error recovered successfully"
        );
    }
}

/// Structured event logging for analytics
pub struct AnalyticsLogger;

impl AnalyticsLogger {
    /// Log a user action for analytics
    pub fn log_user_action(action: &str, metadata: Option<Value>) {
        tracing::info!(
            target: "analytics",
            event_type = "user_action",
            action = action,
            metadata = ?metadata,
            timestamp = chrono::Utc::now().to_rfc3339(),
            "User action logged"
        );
    }

    /// Log consensus operation metrics
    pub fn log_consensus_metrics(
        stage: &str,
        model: &str,
        duration_ms: f64,
        tokens: Option<u32>,
        cost: Option<f64>,
    ) {
        tracing::info!(
            target: "analytics",
            event_type = "consensus_metrics",
            stage = stage,
            model = model,
            duration_ms = duration_ms,
            tokens = tokens,
            cost = cost,
            timestamp = chrono::Utc::now().to_rfc3339(),
            "Consensus metrics logged"
        );
    }

    /// Log performance metrics
    pub fn log_performance_metrics(
        operation: &str,
        duration_ms: f64,
        memory_mb: Option<f64>,
        cpu_percent: Option<f64>,
    ) {
        tracing::info!(
            target: "analytics",
            event_type = "performance_metrics",
            operation = operation,
            duration_ms = duration_ms,
            memory_mb = memory_mb,
            cpu_percent = cpu_percent,
            timestamp = chrono::Utc::now().to_rfc3339(),
            "Performance metrics logged"
        );
    }

    /// Log system health metrics
    pub fn log_health_metrics(component: &str, status: &str, details: Option<Value>) {
        tracing::info!(
            target: "analytics",
            event_type = "health_metrics",
            component = component,
            status = status,
            details = ?details,
            timestamp = chrono::Utc::now().to_rfc3339(),
            "Health metrics logged"
        );
    }
}

/// Log file management utilities
pub struct LogManager {
    config: LoggingConfig,
}

impl LogManager {
    /// Create a new log manager
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }

    /// Clean up old log files
    pub fn cleanup_old_logs(&self) -> Result<()> {
        let log_dir = &self.config.log_dir;
        if !log_dir.exists() {
            return Ok(());
        }

        let mut log_files: Vec<_> = fs::read_dir(log_dir)
            .map_err(|e| HiveError::internal("log_cleanup", e.to_string()))?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()?.to_str()? == "log" {
                    let metadata = entry.metadata().ok()?;
                    let modified = metadata.modified().ok()?;
                    Some((path, modified))
                } else {
                    None
                }
            })
            .collect();

        // Sort by modification time (newest first)
        log_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess files
        if log_files.len() > self.config.max_files {
            for (path, _) in log_files.iter().skip(self.config.max_files) {
                if let Err(e) = fs::remove_file(path) {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to remove old log file"
                    );
                }
            }
        }

        Ok(())
    }

    /// Get current log file size
    pub fn get_log_size(&self) -> Result<u64> {
        let log_file = self.config.log_dir.join("hive.log");
        if log_file.exists() {
            let metadata = fs::metadata(&log_file)
                .map_err(|e| HiveError::internal("log_size", e.to_string()))?;
            Ok(metadata.len())
        } else {
            Ok(0)
        }
    }

    /// Check if log rotation is needed
    pub fn needs_rotation(&self) -> Result<bool> {
        match self.config.rotation {
            LogRotation::Size => {
                let size_mb = self.get_log_size()? / (1024 * 1024);
                Ok(size_mb > self.config.max_file_size)
            }
            _ => Ok(false), // Other rotations are handled by tracing-appender
        }
    }
}

/// Convenience macros for structured logging
#[macro_export]
macro_rules! log_perf {
    ($name:expr, $block:block) => {{
        let timer = $crate::core::logging::PerfTimer::new($name);
        let result = $block;
        timer.finish();
        result
    }};
}

#[macro_export]
macro_rules! log_user_action {
    ($action:expr) => {
        $crate::core::logging::AnalyticsLogger::log_user_action($action, None)
    };
    ($action:expr, $metadata:expr) => {
        $crate::core::logging::AnalyticsLogger::log_user_action($action, Some($metadata))
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr) => {
        $crate::core::logging::ErrorTracker::log_error($error, None)
    };
    ($error:expr, $context:expr) => {
        $crate::core::logging::ErrorTracker::log_error($error, Some($context))
    };
}

/// Check if logging has been initialized
pub fn is_initialized() -> bool {
    LOGGING_INITIALIZED.is_completed()
}

/// Get the default log directory
pub fn default_log_dir() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home_dir.join(".hive").join("logs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.console_level, Level::INFO);
        assert_eq!(config.file_level, Level::DEBUG);
        assert!(config.console_colors);
        assert!(config.file_logging);
    }

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::new("test_operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.checkpoint("midpoint");
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.finish();
    }

    #[test]
    fn test_log_manager() {
        let temp_dir = TempDir::new().unwrap();
        let config = LoggingConfig {
            log_dir: temp_dir.path().to_path_buf(),
            max_files: 3,
            ..LoggingConfig::default()
        };

        let manager = LogManager::new(config);
        assert_eq!(manager.get_log_size().unwrap(), 0);
        assert!(!manager.needs_rotation().unwrap());
    }
}
