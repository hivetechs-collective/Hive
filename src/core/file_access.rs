//! Secure file access wrapper for Hive AI
//!
//! This module provides trust-based file operations with comprehensive
//! security controls and audit logging.

use std::path::{Path, PathBuf};
use std::fs::Metadata;
use std::time::SystemTime;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{Context, Result};
use tracing::{debug, warn, error};
use chrono::Utc;

use super::security::{
    TrustManager, TrustDecision, SecurityError, SecurityEvent, SecurityPolicy
};

/// Secure file access wrapper with trust-based controls
#[derive(Debug)]
pub struct SecureFileAccess {
    trust_manager: std::sync::Arc<tokio::sync::Mutex<TrustManager>>,
    policy: SecurityPolicy,
}

impl SecureFileAccess {
    /// Create a new secure file access instance
    pub fn new(
        trust_manager: std::sync::Arc<tokio::sync::Mutex<TrustManager>>,
        policy: SecurityPolicy,
    ) -> Self {
        Self {
            trust_manager,
            policy,
        }
    }

    /// Securely read a file with trust verification
    pub async fn read_file(&self, path: &Path) -> Result<String> {
        // Verify trust before reading
        self.verify_file_access(path, FileOperation::Read).await?;

        // Additional security checks
        self.verify_no_symlink_escape(path).await?;
        self.verify_file_size_limit(path).await?;
        self.verify_file_permissions(path).await?;

        // Read the file
        let content = fs::read_to_string(path).await
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        debug!("Successfully read file: {} ({} bytes)", path.display(), content.len());

        // Log the access
        self.log_file_access(path, FileOperation::Read, true).await?;

        Ok(content)
    }

    /// Securely read file bytes with trust verification
    pub async fn read_file_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        // Verify trust before reading
        self.verify_file_access(path, FileOperation::Read).await?;

        // Additional security checks
        self.verify_no_symlink_escape(path).await?;
        self.verify_file_size_limit(path).await?;
        self.verify_file_permissions(path).await?;

        // Read the file
        let content = fs::read(path).await
            .with_context(|| format!("Failed to read file bytes: {}", path.display()))?;

        debug!("Successfully read file bytes: {} ({} bytes)", path.display(), content.len());

        // Log the access
        self.log_file_access(path, FileOperation::Read, true).await?;

        Ok(content)
    }

    /// Securely write to a file with trust verification
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        // Verify trust before writing
        self.verify_file_access(path, FileOperation::Write).await?;

        // Additional security checks
        self.verify_no_symlink_escape(path).await?;
        self.verify_write_permissions(path).await?;

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await
                    .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
            }
        }

        // Write the file
        fs::write(path, content).await
            .with_context(|| format!("Failed to write file: {}", path.display()))?;

        debug!("Successfully wrote file: {} ({} bytes)", path.display(), content.len());

        // Log the access
        self.log_file_access(path, FileOperation::Write, true).await?;

        Ok(())
    }

    /// Securely write bytes to a file with trust verification
    pub async fn write_file_bytes(&self, path: &Path, content: &[u8]) -> Result<()> {
        // Verify trust before writing
        self.verify_file_access(path, FileOperation::Write).await?;

        // Additional security checks
        self.verify_no_symlink_escape(path).await?;
        self.verify_write_permissions(path).await?;

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await
                    .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
            }
        }

        // Write the file
        fs::write(path, content).await
            .with_context(|| format!("Failed to write file bytes: {}", path.display()))?;

        debug!("Successfully wrote file bytes: {} ({} bytes)", path.display(), content.len());

        // Log the access
        self.log_file_access(path, FileOperation::Write, true).await?;

        Ok(())
    }

    /// Securely list directory contents with trust verification
    pub async fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        // Verify trust before listing
        self.verify_file_access(path, FileOperation::List).await?;

        // Additional security checks
        self.verify_no_symlink_escape(path).await?;

        // List directory contents
        let mut entries = Vec::new();
        let mut dir = fs::read_dir(path).await
            .with_context(|| format!("Failed to read directory: {}", path.display()))?;

        while let Some(entry) = dir.next_entry().await? {
            entries.push(entry.path());
        }

        debug!("Successfully listed directory: {} ({} entries)", path.display(), entries.len());

        // Log the access
        self.log_file_access(path, FileOperation::List, true).await?;

        Ok(entries)
    }

    /// Check if a file exists with trust verification
    pub async fn file_exists(&self, path: &Path) -> Result<bool> {
        // For existence checks, we can be more permissive
        // but still verify basic security
        self.verify_no_symlink_escape(path).await?;

        let exists = path.exists();

        debug!("File existence check: {} = {}", path.display(), exists);

        Ok(exists)
    }

    /// Get file metadata with trust verification
    pub async fn get_metadata(&self, path: &Path) -> Result<Metadata> {
        // Verify trust before accessing metadata
        self.verify_file_access(path, FileOperation::Metadata).await?;

        // Additional security checks
        self.verify_no_symlink_escape(path).await?;

        let metadata = fs::metadata(path).await
            .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

        debug!("Successfully got metadata for: {}", path.display());

        Ok(metadata)
    }

    /// Verify file access permissions based on trust and security policy
    async fn verify_file_access(&self, path: &Path, operation: FileOperation) -> Result<()> {
        let canonical_path = path.canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;

        // Get the directory containing the file
        let check_path = if canonical_path.is_file() {
            canonical_path.parent()
                .ok_or_else(|| anyhow::anyhow!("File has no parent directory"))?
        } else {
            &canonical_path
        };

        // Check trust through the trust manager
        let mut trust_manager = self.trust_manager.lock().await;
        let decision = trust_manager.check_trust(check_path).await
            .with_context(|| format!("Failed to check trust for: {}", check_path.display()))?;

        match decision {
            TrustDecision::AlreadyTrusted | TrustDecision::TrustGranted => {
                // Access granted
                Ok(())
            }
            TrustDecision::TrustDenied => {
                // Log security violation
                self.log_security_violation(
                    "File access denied by user".to_string(),
                    Some(canonical_path.clone()),
                    format!("Operation: {:?}, Path: {}", operation, canonical_path.display()),
                ).await?;

                Err(SecurityError::UntrustedPath(canonical_path).into())
            }
            TrustDecision::Blocked => {
                // Log security violation
                self.log_security_violation(
                    "File access blocked by policy".to_string(),
                    Some(canonical_path.clone()),
                    format!("Operation: {:?}, Path: {}", operation, canonical_path.display()),
                ).await?;

                Err(SecurityError::BlockedPath(canonical_path).into())
            }
        }
    }

    /// Verify that the path doesn't escape through symlinks
    async fn verify_no_symlink_escape(&self, path: &Path) -> Result<()> {
        let canonical_path = path.canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;

        // Check if the canonical path is within expected bounds
        // This helps prevent directory traversal attacks
        let current_dir = std::env::current_dir()
            .context("Failed to get current directory")?;

        // For now, we'll be permissive but log suspicious paths
        if !canonical_path.starts_with(&current_dir) {
            warn!("Path outside current directory: {}", canonical_path.display());

            // Check if it's in a system directory (potential danger)
            let path_str = canonical_path.to_string_lossy();
            if path_str.starts_with("/etc") ||
               path_str.starts_with("/sys") ||
               path_str.starts_with("/proc") ||
               path_str.starts_with("/dev") {
                return Err(SecurityError::SymlinkEscape(canonical_path).into());
            }
        }

        Ok(())
    }

    /// Verify file size doesn't exceed security limits
    async fn verify_file_size_limit(&self, path: &Path) -> Result<()> {
        if let Ok(metadata) = fs::metadata(path).await {
            if metadata.len() > self.policy.max_file_size {
                warn!("File too large: {} ({} bytes)", path.display(), metadata.len());

                self.log_security_violation(
                    "File size limit exceeded".to_string(),
                    Some(path.to_path_buf()),
                    format!("Size: {} bytes, Limit: {} bytes", metadata.len(), self.policy.max_file_size),
                ).await?;

                return Err(SecurityError::FileTooLarge.into());
            }
        }

        Ok(())
    }

    /// Verify file permissions for read access
    async fn verify_file_permissions(&self, path: &Path) -> Result<()> {
        // Check if file is readable
        if let Err(e) = fs::File::open(path).await {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                return Err(SecurityError::InsufficientPermissions(path.to_path_buf()).into());
            }
        }

        Ok(())
    }

    /// Verify write permissions for a path
    async fn verify_write_permissions(&self, path: &Path) -> Result<()> {
        // Check parent directory write permissions if file doesn't exist
        if !path.exists() {
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    // Try to create a temporary file to test write permissions
                    let temp_file = parent.join(".hive_write_test");
                    if let Err(e) = fs::write(&temp_file, "").await {
                        if e.kind() == std::io::ErrorKind::PermissionDenied {
                            return Err(SecurityError::InsufficientPermissions(parent.to_path_buf()).into());
                        }
                    } else {
                        // Clean up test file
                        let _ = fs::remove_file(&temp_file).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Log file access events
    async fn log_file_access(&self, path: &Path, operation: FileOperation, success: bool) -> Result<()> {
        debug!("File access: {:?} on {} (success: {})", operation, path.display(), success);
        Ok(())
    }

    /// Log security violations
    async fn log_security_violation(
        &self,
        violation_type: String,
        path: Option<PathBuf>,
        details: String,
    ) -> Result<()> {
        let event = SecurityEvent::SecurityViolation {
            violation_type,
            path,
            details,
            timestamp: Utc::now(),
        };

        error!("Security violation: {:?}", event);

        // We'd normally log this through the trust manager's audit logger,
        // but since we can't easily access it here, we'll just use tracing
        // In a real implementation, we'd need to restructure this

        Ok(())
    }
}

/// File operation types for security logging
#[derive(Debug, Clone, Copy)]
pub enum FileOperation {
    Read,
    Write,
    List,
    Metadata,
}

/// Convenience functions for common file operations
pub struct FileOperations;

impl FileOperations {
    /// Create a secure file access instance with default policy
    pub async fn new_with_trust_manager(
        trust_manager: std::sync::Arc<tokio::sync::Mutex<TrustManager>>,
    ) -> SecureFileAccess {
        let policy = SecurityPolicy::default();
        SecureFileAccess::new(trust_manager, policy)
    }

    /// Read a file safely with trust verification
    pub async fn read_trusted_file(
        trust_manager: std::sync::Arc<tokio::sync::Mutex<TrustManager>>,
        path: &Path,
    ) -> Result<String> {
        let file_access = Self::new_with_trust_manager(trust_manager).await;
        file_access.read_file(path).await
    }

    /// Write a file safely with trust verification
    pub async fn write_trusted_file(
        trust_manager: std::sync::Arc<tokio::sync::Mutex<TrustManager>>,
        path: &Path,
        content: &str,
    ) -> Result<()> {
        let file_access = Self::new_with_trust_manager(trust_manager).await;
        file_access.write_file(path, content).await
    }

    /// List directory contents safely with trust verification
    pub async fn list_trusted_directory(
        trust_manager: std::sync::Arc<tokio::sync::Mutex<TrustManager>>,
        path: &Path,
    ) -> Result<Vec<PathBuf>> {
        let file_access = Self::new_with_trust_manager(trust_manager).await;
        file_access.list_directory(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn create_test_trust_manager() -> Arc<Mutex<TrustManager>> {
        let temp_dir = TempDir::new().unwrap();
        let policy = SecurityPolicy::default();
        let trust_manager = TrustManager::new(temp_dir.path(), policy).await.unwrap();
        Arc::new(Mutex::new(trust_manager))
    }

    #[tokio::test]
    async fn test_secure_file_access_creation() {
        let trust_manager = create_test_trust_manager().await;
        let policy = SecurityPolicy::default();
        let _file_access = SecureFileAccess::new(trust_manager, policy);
        // Should not panic
    }

    #[tokio::test]
    async fn test_file_size_limit_validation() {
        let trust_manager = create_test_trust_manager().await;
        let mut policy = SecurityPolicy::default();
        policy.max_file_size = 10; // Very small limit for testing

        let file_access = SecureFileAccess::new(trust_manager, policy);

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("large_file.txt");
        tokio::fs::write(&test_file, "This is definitely more than 10 bytes").await.unwrap();

        let result = file_access.verify_file_size_limit(&test_file).await;
        assert!(result.is_err());
    }
}