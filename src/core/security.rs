//! Security and trust management system for HiveTechs Consensus
//! 
//! This module implements a Claude Code-style trust system that requires explicit
//! user permission before accessing files in new directories. All file operations
//! must go through this security layer.

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

/// Trust level for a directory or file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Fully trusted - no warnings or prompts
    Trusted,
    /// Not trusted - operations will be blocked
    Untrusted,
    /// Temporarily trusted for this session only
    Temporary,
}

impl fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrustLevel::Trusted => write!(f, "Trusted"),
            TrustLevel::Untrusted => write!(f, "Untrusted"),
            TrustLevel::Temporary => write!(f, "Temporary"),
        }
    }
}

/// Security decision made by the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustDecision {
    /// Path that was evaluated
    pub path: PathBuf,
    /// Trust level assigned
    pub trust_level: TrustLevel,
    /// When the decision was made
    pub timestamp: DateTime<Utc>,
    /// Reason for the decision (if provided)
    pub reason: Option<String>,
}

/// Security event that occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// Type of event
    pub event_type: SecurityEventType,
    /// Path involved (if any)
    pub path: Option<PathBuf>,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Additional details
    pub details: String,
    /// Whether the operation was allowed
    pub allowed: bool,
}

/// Types of security events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// Trust prompt shown to user
    TrustPrompt,
    /// File access attempted
    FileAccess,
    /// Directory listing attempted
    DirectoryList,
    /// File write attempted
    FileWrite,
    /// File deletion attempted
    FileDeletion,
    /// Trust decision made
    TrustDecision,
    /// Security violation detected
    SecurityViolation,
}

impl fmt::Display for SecurityEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityEventType::TrustPrompt => write!(f, "Trust Prompt"),
            SecurityEventType::FileAccess => write!(f, "File Access"),
            SecurityEventType::DirectoryList => write!(f, "Directory List"),
            SecurityEventType::FileWrite => write!(f, "File Write"),
            SecurityEventType::FileDeletion => write!(f, "File Deletion"),
            SecurityEventType::TrustDecision => write!(f, "Trust Decision"),
            SecurityEventType::SecurityViolation => write!(f, "Security Violation"),
        }
    }
}

/// Security context for a session
pub struct SecurityContext {
    /// Database connection for persistent storage
    db: Arc<Mutex<Connection>>,
    /// Cache of trust decisions for performance
    trust_cache: Arc<Mutex<HashMap<PathBuf, TrustLevel>>>,
    /// Whether to use interactive prompts (can be disabled for testing)
    interactive: bool,
    /// Session ID for temporary trusts
    session_id: String,
}

impl SecurityContext {
    /// Create a new security context
    pub fn new(db_path: Option<PathBuf>, interactive: bool) -> Result<Self> {
        let db_path = db_path.unwrap_or_else(|| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("hive")
                .join("security.db")
        });

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create security database directory")?;
        }

        let conn = Connection::open(&db_path)
            .context("Failed to open security database")?;

        // Initialize database schema
        Self::initialize_database(&conn)?;

        let session_id = format!("session_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            trust_cache: Arc::new(Mutex::new(HashMap::new())),
            interactive,
            session_id,
        })
    }

    /// Initialize the database schema
    fn initialize_database(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            -- Trust decisions table
            CREATE TABLE IF NOT EXISTS trust_decisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                trust_level TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                reason TEXT,
                session_id TEXT
            );

            -- Security events table
            CREATE TABLE IF NOT EXISTS security_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                path TEXT,
                timestamp TEXT NOT NULL,
                details TEXT NOT NULL,
                allowed BOOLEAN NOT NULL,
                session_id TEXT
            );

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_trust_path ON trust_decisions(path);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON security_events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_events_type ON security_events(event_type);
            "#,
        )?;
        Ok(())
    }

    /// Check if a path is trusted
    pub fn is_trusted(&self, path: &Path) -> Result<bool> {
        let canonical = path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());
        
        // Check cache first
        {
            let cache = self.trust_cache.lock().unwrap();
            if let Some(&trust_level) = cache.get(&canonical) {
                return Ok(trust_level == TrustLevel::Trusted || trust_level == TrustLevel::Temporary);
            }
        }

        // Check database
        let trust_level = self.get_trust_level(&canonical)?;
        
        // Update cache
        {
            let mut cache = self.trust_cache.lock().unwrap();
            cache.insert(canonical.clone(), trust_level);
        }

        Ok(trust_level == TrustLevel::Trusted || trust_level == TrustLevel::Temporary)
    }

    /// Get the trust level for a path
    pub fn get_trust_level(&self, path: &Path) -> Result<TrustLevel> {
        let canonical = path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());
        
        let db = self.db.lock().unwrap();
        
        // Check exact path match
        let trust_level: Option<String> = db.query_row(
            "SELECT trust_level FROM trust_decisions WHERE path = ?1",
            params![canonical.to_string_lossy()],
            |row| row.get(0),
        ).optional()?;

        if let Some(level) = trust_level {
            return Ok(match level.as_str() {
                "Trusted" => TrustLevel::Trusted,
                "Untrusted" => TrustLevel::Untrusted,
                "Temporary" => TrustLevel::Temporary,
                _ => TrustLevel::Untrusted,
            });
        }

        // Check parent directories
        let mut current = canonical.as_path();
        while let Some(parent) = current.parent() {
            let parent_trust: Option<String> = db.query_row(
                "SELECT trust_level FROM trust_decisions WHERE path = ?1",
                params![parent.to_string_lossy()],
                |row| row.get(0),
            ).optional()?;

            if let Some(level) = parent_trust {
                return Ok(match level.as_str() {
                    "Trusted" => TrustLevel::Trusted,
                    "Untrusted" => TrustLevel::Untrusted,
                    "Temporary" => TrustLevel::Temporary,
                    _ => TrustLevel::Untrusted,
                });
            }
            
            current = parent;
        }

        // Default to untrusted
        Ok(TrustLevel::Untrusted)
    }

    /// Set trust level for a path
    pub fn set_trust_level(&self, path: &Path, level: TrustLevel, reason: Option<String>) -> Result<()> {
        let canonical = path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());
        
        let decision = TrustDecision {
            path: canonical.clone(),
            trust_level: level,
            timestamp: Utc::now(),
            reason,
        };

        // Store in database
        {
            let db = self.db.lock().unwrap();
            db.execute(
                r#"
                INSERT OR REPLACE INTO trust_decisions (path, trust_level, timestamp, reason, session_id)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                params![
                    decision.path.to_string_lossy(),
                    decision.trust_level.to_string(),
                    decision.timestamp.to_rfc3339(),
                    decision.reason,
                    if level == TrustLevel::Temporary { Some(&self.session_id) } else { None }
                ],
            )?;
        }

        // Update cache
        {
            let mut cache = self.trust_cache.lock().unwrap();
            cache.insert(canonical, level);
        }

        // Log security event
        self.log_event(SecurityEvent {
            event_type: SecurityEventType::TrustDecision,
            path: Some(decision.path),
            timestamp: decision.timestamp,
            details: format!("Trust level set to: {}", level),
            allowed: true,
        })?;

        Ok(())
    }

    /// Prompt user for trust decision
    pub fn prompt_trust(&self, path: &Path, operation: &str) -> Result<TrustLevel> {
        if !self.interactive {
            // In non-interactive mode, deny by default
            return Ok(TrustLevel::Untrusted);
        }

        let canonical = path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());

        // Log the prompt event
        self.log_event(SecurityEvent {
            event_type: SecurityEventType::TrustPrompt,
            path: Some(canonical.clone()),
            timestamp: Utc::now(),
            details: format!("Prompting for {}", operation),
            allowed: false,
        })?;

        // Show warning dialog
        println!("\n┌─────────────────────────────────────────────────────────┐");
        println!("│ 🔒 Security Warning - Directory Access Request          │");
        println!("├─────────────────────────────────────────────────────────┤");
        println!("│ HiveTechs Consensus wants to {} in:              │", operation);
        println!("│                                                         │");
        println!("│ 📁 {:<51} │", canonical.display());
        println!("│                                                         │");
        println!("│ This directory has not been trusted yet.                │");
        println!("│ Granting access allows Hive to read, analyze, and      │");
        println!("│ modify files in this directory.                        │");
        println!("└─────────────────────────────────────────────────────────┘");

        let theme = ColorfulTheme::default();
        let choices = vec![
            "✅ Trust this directory permanently",
            "⏱️  Trust temporarily (this session only)",
            "❌ Don't trust (deny access)",
        ];

        let selection = Select::with_theme(&theme)
            .with_prompt("Choose an option")
            .items(&choices)
            .default(2) // Default to deny
            .interact()?;

        let trust_level = match selection {
            0 => {
                info!("User granted permanent trust to: {}", canonical.display());
                TrustLevel::Trusted
            }
            1 => {
                info!("User granted temporary trust to: {}", canonical.display());
                TrustLevel::Temporary
            }
            _ => {
                warn!("User denied trust to: {}", canonical.display());
                TrustLevel::Untrusted
            }
        };

        // Store the decision
        self.set_trust_level(&canonical, trust_level, Some(operation.to_string()))?;

        Ok(trust_level)
    }

    /// Check file access permission
    pub fn check_file_access(&self, path: &Path, operation: &str) -> Result<()> {
        let canonical = path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());

        // Check if trusted
        let trust_level = self.get_trust_level(&canonical)?;
        
        if trust_level == TrustLevel::Trusted || trust_level == TrustLevel::Temporary {
            // Log successful access
            self.log_event(SecurityEvent {
                event_type: SecurityEventType::FileAccess,
                path: Some(canonical),
                timestamp: Utc::now(),
                details: format!("Allowed {}", operation),
                allowed: true,
            })?;
            return Ok(());
        }

        // Not trusted, prompt user
        let new_level = self.prompt_trust(&canonical, operation)?;
        
        if new_level == TrustLevel::Trusted || new_level == TrustLevel::Temporary {
            Ok(())
        } else {
            // Log denied access
            self.log_event(SecurityEvent {
                event_type: SecurityEventType::SecurityViolation,
                path: Some(canonical.clone()),
                timestamp: Utc::now(),
                details: format!("Denied {}", operation),
                allowed: false,
            })?;
            
            Err(anyhow!("Access denied to untrusted path: {}", canonical.display()))
        }
    }

    /// Check directory listing permission
    pub fn check_directory_access(&self, path: &Path) -> Result<()> {
        self.check_file_access(path, "list directory contents")
    }

    /// Check file read permission
    pub fn check_read_access(&self, path: &Path) -> Result<()> {
        self.check_file_access(path, "read file")
    }

    /// Check file write permission
    pub fn check_write_access(&self, path: &Path) -> Result<()> {
        let canonical = path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());

        // Write operations require explicit trust
        let trust_level = self.get_trust_level(&canonical)?;
        
        if trust_level == TrustLevel::Trusted || trust_level == TrustLevel::Temporary {
            // Log write access
            self.log_event(SecurityEvent {
                event_type: SecurityEventType::FileWrite,
                path: Some(canonical),
                timestamp: Utc::now(),
                details: "Write access granted".to_string(),
                allowed: true,
            })?;
            return Ok(());
        }

        // Prompt for write access
        let new_level = self.prompt_trust(&canonical, "write to file")?;
        
        if new_level == TrustLevel::Trusted || new_level == TrustLevel::Temporary {
            Ok(())
        } else {
            Err(anyhow!("Write access denied to untrusted path: {}", canonical.display()))
        }
    }

    /// Check file deletion permission
    pub fn check_delete_access(&self, path: &Path) -> Result<()> {
        let canonical = path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());

        // Deletion requires additional confirmation
        let trust_level = self.get_trust_level(&canonical)?;
        
        if trust_level != TrustLevel::Trusted && trust_level != TrustLevel::Temporary {
            let new_level = self.prompt_trust(&canonical, "delete file")?;
            if new_level != TrustLevel::Trusted && new_level != TrustLevel::Temporary {
                return Err(anyhow!("Delete access denied to untrusted path: {}", canonical.display()));
            }
        }

        // Even for trusted paths, confirm deletion
        if self.interactive {
            let theme = ColorfulTheme::default();
            let confirm = Confirm::with_theme(&theme)
                .with_prompt(format!("Are you sure you want to delete '{}'?", canonical.display()))
                .default(false)
                .interact()?;

            if !confirm {
                return Err(anyhow!("Deletion cancelled by user"));
            }
        }

        // Log deletion
        self.log_event(SecurityEvent {
            event_type: SecurityEventType::FileDeletion,
            path: Some(canonical),
            timestamp: Utc::now(),
            details: "File deletion allowed".to_string(),
            allowed: true,
        })?;

        Ok(())
    }

    /// Log a security event
    pub fn log_event(&self, event: SecurityEvent) -> Result<()> {
        let db = self.db.lock().unwrap();
        
        db.execute(
            r#"
            INSERT INTO security_events (event_type, path, timestamp, details, allowed, session_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                event.event_type.to_string(),
                event.path.as_ref().map(|p| p.to_string_lossy().to_string()),
                event.timestamp.to_rfc3339(),
                event.details,
                event.allowed,
                self.session_id,
            ],
        )?;

        // Also log to tracing
        match event.event_type {
            SecurityEventType::SecurityViolation => {
                error!(
                    path = ?event.path,
                    details = %event.details,
                    "Security violation detected"
                );
            }
            SecurityEventType::TrustPrompt | SecurityEventType::TrustDecision => {
                info!(
                    path = ?event.path,
                    details = %event.details,
                    allowed = event.allowed,
                    "Security event: {}",
                    event.event_type
                );
            }
            _ => {
                debug!(
                    event_type = %event.event_type,
                    path = ?event.path,
                    allowed = event.allowed,
                    "Security event logged"
                );
            }
        }

        Ok(())
    }

    /// Get security events for a path
    pub fn get_events_for_path(&self, path: &Path) -> Result<Vec<SecurityEvent>> {
        let canonical = path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());
        
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
            r#"
            SELECT event_type, path, timestamp, details, allowed
            FROM security_events
            WHERE path = ?1
            ORDER BY timestamp DESC
            LIMIT 100
            "#,
        )?;

        let events = stmt.query_map(params![canonical.to_string_lossy()], |row| {
            Ok(SecurityEvent {
                event_type: match row.get::<_, String>(0)?.as_str() {
                    "Trust Prompt" => SecurityEventType::TrustPrompt,
                    "File Access" => SecurityEventType::FileAccess,
                    "Directory List" => SecurityEventType::DirectoryList,
                    "File Write" => SecurityEventType::FileWrite,
                    "File Deletion" => SecurityEventType::FileDeletion,
                    "Trust Decision" => SecurityEventType::TrustDecision,
                    "Security Violation" => SecurityEventType::SecurityViolation,
                    _ => SecurityEventType::FileAccess,
                },
                path: row.get::<_, Option<String>>(1)?.map(PathBuf::from),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                details: row.get(3)?,
                allowed: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(events)
    }

    /// Get all trusted paths
    pub fn get_trusted_paths(&self) -> Result<Vec<(PathBuf, TrustLevel)>> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
            r#"
            SELECT path, trust_level
            FROM trust_decisions
            WHERE trust_level IN ('Trusted', 'Temporary')
            ORDER BY path
            "#,
        )?;

        let paths = stmt.query_map(params![], |row| {
            let path = PathBuf::from(row.get::<_, String>(0)?);
            let level = match row.get::<_, String>(1)?.as_str() {
                "Trusted" => TrustLevel::Trusted,
                "Temporary" => TrustLevel::Temporary,
                _ => TrustLevel::Untrusted,
            };
            Ok((path, level))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(paths)
    }

    /// Clear all temporary trust decisions for the current session
    pub fn clear_temporary_trusts(&self) -> Result<()> {
        let db = self.db.lock().unwrap();
        db.execute(
            "DELETE FROM trust_decisions WHERE trust_level = 'Temporary' AND session_id = ?1",
            params![self.session_id],
        )?;

        // Clear from cache
        let mut cache = self.trust_cache.lock().unwrap();
        cache.retain(|_, &mut level| level != TrustLevel::Temporary);

        info!("Cleared all temporary trust decisions for session");
        Ok(())
    }

    /// Reset all trust decisions (dangerous!)
    pub fn reset_all_trusts(&self) -> Result<()> {
        if self.interactive {
            let theme = ColorfulTheme::default();
            let confirm = Confirm::with_theme(&theme)
                .with_prompt("⚠️  This will remove ALL trust decisions. Are you absolutely sure?")
                .default(false)
                .interact()?;

            if !confirm {
                return Err(anyhow!("Reset cancelled by user"));
            }
        }

        let db = self.db.lock().unwrap();
        db.execute("DELETE FROM trust_decisions", params![])?;

        // Clear cache
        let mut cache = self.trust_cache.lock().unwrap();
        cache.clear();

        warn!("All trust decisions have been reset");
        Ok(())
    }
}

/// Global security context instance
static SECURITY_CONTEXT: once_cell::sync::OnceCell<Arc<SecurityContext>> = once_cell::sync::OnceCell::new();

/// Initialize the global security context
pub fn initialize_security(db_path: Option<PathBuf>, interactive: bool) -> Result<()> {
    let context = SecurityContext::new(db_path, interactive)?;
    SECURITY_CONTEXT.set(Arc::new(context))
        .map_err(|_| anyhow!("Security context already initialized"))?;
    Ok(())
}

/// Get the global security context
pub fn get_security_context() -> Result<Arc<SecurityContext>> {
    SECURITY_CONTEXT.get()
        .cloned()
        .ok_or_else(|| anyhow!("Security context not initialized"))
}

/// Check if a path is trusted
pub fn is_trusted(path: &Path) -> Result<bool> {
    let context = get_security_context()?;
    context.is_trusted(path)
}

/// Check file read access
pub fn check_read_access(path: &Path) -> Result<()> {
    let context = get_security_context()?;
    context.check_read_access(path)
}

/// Check file write access
pub fn check_write_access(path: &Path) -> Result<()> {
    let context = get_security_context()?;
    context.check_write_access(path)
}

/// Check file deletion access
pub fn check_delete_access(path: &Path) -> Result<()> {
    let context = get_security_context()?;
    context.check_delete_access(path)
}

/// Check directory access
pub fn check_directory_access(path: &Path) -> Result<()> {
    let context = get_security_context()?;
    context.check_directory_access(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_trust_levels() {
        assert_eq!(TrustLevel::Trusted.to_string(), "Trusted");
        assert_eq!(TrustLevel::Untrusted.to_string(), "Untrusted");
        assert_eq!(TrustLevel::Temporary.to_string(), "Temporary");
    }

    #[test]
    fn test_security_context_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let context = SecurityContext::new(Some(db_path), false).unwrap();
        assert!(!context.interactive);
    }

    #[test]
    fn test_trust_decision_storage() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let test_path = temp_dir.path().join("test.txt");
        
        let context = SecurityContext::new(Some(db_path), false).unwrap();
        
        // Set trust level
        context.set_trust_level(&test_path, TrustLevel::Trusted, Some("Test".to_string())).unwrap();
        
        // Verify it was stored
        let level = context.get_trust_level(&test_path).unwrap();
        assert_eq!(level, TrustLevel::Trusted);
    }

    #[test]
    fn test_parent_directory_trust() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let context = SecurityContext::new(Some(db_path), false).unwrap();
        
        // Trust parent directory
        context.set_trust_level(temp_dir.path(), TrustLevel::Trusted, None).unwrap();
        
        // Child should inherit trust
        let child_path = temp_dir.path().join("subdir/file.txt");
        let level = context.get_trust_level(&child_path).unwrap();
        assert_eq!(level, TrustLevel::Trusted);
    }

    #[test]
    fn test_security_event_logging() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let test_path = temp_dir.path().join("test.txt");
        
        let context = SecurityContext::new(Some(db_path), false).unwrap();
        
        // Log an event
        let event = SecurityEvent {
            event_type: SecurityEventType::FileAccess,
            path: Some(test_path.clone()),
            timestamp: Utc::now(),
            details: "Test access".to_string(),
            allowed: true,
        };
        
        context.log_event(event).unwrap();
        
        // Retrieve events
        let events = context.get_events_for_path(&test_path).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].details, "Test access");
    }

    #[test]
    fn test_trusted_paths_list() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let context = SecurityContext::new(Some(db_path), false).unwrap();
        
        // Add some trusted paths
        let path1 = temp_dir.path().join("dir1");
        let path2 = temp_dir.path().join("dir2");
        
        context.set_trust_level(&path1, TrustLevel::Trusted, None).unwrap();
        context.set_trust_level(&path2, TrustLevel::Temporary, None).unwrap();
        
        // Get trusted paths
        let trusted = context.get_trusted_paths().unwrap();
        assert_eq!(trusted.len(), 2);
    }

    #[test]
    fn test_cache_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let test_path = temp_dir.path().join("cached.txt");
        
        let context = SecurityContext::new(Some(db_path), false).unwrap();
        
        // Set trust level (should cache)
        context.set_trust_level(&test_path, TrustLevel::Trusted, None).unwrap();
        
        // Check cache
        let cache = context.trust_cache.lock().unwrap();
        assert!(cache.contains_key(&test_path));
    }
}

/// Type alias for compatibility with other modules
pub type SecurityManager = SecurityContext;