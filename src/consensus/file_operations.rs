//! File Reading Operations for Consensus Pipeline
//!
//! This module provides secure file reading capabilities for consensus stages,
//! enabling them to read actual file contents from the repository.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, instrument, warn};

use crate::core::error::{HiveError, HiveResult};

/// Security policy for file operations
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Maximum file size in bytes (default: 1MB)
    pub max_file_size: usize,
    /// Maximum files to read in one request
    pub max_files_per_request: usize,
    /// Allowed file extensions
    pub allowed_extensions: Vec<String>,
    /// Denied patterns (e.g., .git, node_modules)
    pub denied_patterns: Vec<String>,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            max_file_size: 1024 * 1024, // 1MB
            max_files_per_request: 100,
            allowed_extensions: vec![
                "rs", "toml", "md", "txt", "json", "yaml", "yml", "ts", "js", "jsx", "tsx",
                "py", "go", "java", "cs", "cpp", "c", "h", "hpp", "css", "html", "xml",
                "sql", "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd", "dockerfile",
                "makefile", "cmake", "gradle", "pom", "sbt", "lock", "sum", "mod",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            denied_patterns: vec![
                ".git", "node_modules", "target", "dist", "build", ".idea", ".vscode",
                "__pycache__", ".pytest_cache", ".mypy_cache", ".tox", "venv", ".env",
                "*.exe", "*.dll", "*.so", "*.dylib", "*.bin", "*.o", "*.a",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        }
    }
}

/// File content with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub path: PathBuf,
    pub content: String,
    pub size_bytes: usize,
    pub lines: usize,
    pub language: Option<String>,
    pub encoding: String,
    pub truncated: bool,
}

/// Search match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

/// File cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    content: FileContent,
    cached_at: std::time::SystemTime,
}

/// File reader with security and caching
pub struct FileReader {
    security_policy: SecurityPolicy,
    cache: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
    cache_ttl: std::time::Duration,
}

impl FileReader {
    /// Create a new file reader
    pub fn new(security_policy: SecurityPolicy) -> Self {
        Self {
            security_policy,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: std::time::Duration::from_secs(300), // 5 minutes
        }
    }

    /// Read a file with security checks
    #[instrument(skip(self))]
    pub async fn read_file(&self, path: &Path) -> Result<FileContent> {
        // Security checks
        self.verify_path_allowed(path)?;

        // Check cache first
        if let Some(cached) = self.get_from_cache(path).await {
            debug!("File {} found in cache", path.display());
            return Ok(cached);
        }

        // Get file metadata
        let metadata = fs::metadata(path)
            .await
            .with_context(|| format!("Failed to get metadata for {}", path.display()))?;

        if !metadata.is_file() {
            anyhow::bail!("Path {} is not a file", path.display());
        }

        let size = metadata.len() as usize;
        if size > self.security_policy.max_file_size {
            anyhow::bail!(
                "File {} is too large ({} bytes, max {} bytes)",
                path.display(),
                size,
                self.security_policy.max_file_size
            );
        }

        // Read file content
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read file {}", path.display()))?;

        let lines = content.lines().count();
        let language = detect_language(path);

        let file_content = FileContent {
            path: path.to_path_buf(),
            content: content.clone(),
            size_bytes: size,
            lines,
            language,
            encoding: "UTF-8".to_string(),
            truncated: false,
        };

        // Cache the content
        self.cache_file(path, file_content.clone()).await;

        Ok(file_content)
    }

    /// Read specific lines from a file
    #[instrument(skip(self))]
    pub async fn read_file_lines(
        &self,
        path: &Path,
        start_line: usize,
        end_line: usize,
    ) -> Result<Vec<String>> {
        let file_content = self.read_file(path).await?;
        let lines: Vec<&str> = file_content.content.lines().collect();

        let start = start_line.saturating_sub(1);
        let end = end_line.min(lines.len());

        Ok(lines[start..end].iter().map(|s| s.to_string()).collect())
    }

    /// List files in a directory
    #[instrument(skip(self))]
    pub async fn list_directory(&self, path: &Path) -> Result<Vec<tokio::fs::DirEntry>> {
        self.verify_path_allowed(path)?;

        let mut entries = Vec::new();
        let mut dir = fs::read_dir(path)
            .await
            .with_context(|| format!("Failed to read directory {}", path.display()))?;

        while let Some(entry) = dir.next_entry().await? {
            let file_type = entry.file_type().await?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip denied patterns
            if self.is_denied_pattern(&file_name_str) {
                continue;
            }

            // For files, check extensions
            if file_type.is_file() {
                if let Some(ext) = Path::new(&file_name).extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if !self.security_policy.allowed_extensions.contains(&ext_str) {
                        continue;
                    }
                }
            }

            entries.push(entry);
        }

        Ok(entries)
    }

    /// Find files matching a glob pattern
    #[instrument(skip(self))]
    pub async fn glob_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        // Use globset for pattern matching
        use globset::GlobBuilder;

        let glob = GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()?
            .compile_matcher();

        let mut matching_files = Vec::new();

        // Walk the file tree from current directory
        self.walk_directory(Path::new("."), &glob, &mut matching_files)
            .await?;

        Ok(matching_files)
    }

    /// Search for content in files
    #[instrument(skip(self))]
    pub async fn search_content(
        &self,
        pattern: &str,
        paths: &[PathBuf],
    ) -> Result<Vec<SearchMatch>> {
        use regex::Regex;

        let regex = Regex::new(pattern)?;
        let mut all_matches = Vec::new();

        for path in paths {
            if let Ok(content) = self.read_file(path).await {
                let lines: Vec<&str> = content.content.lines().collect();

                for (line_idx, line) in lines.iter().enumerate() {
                    if let Some(mat) = regex.find(line) {
                        let context_start = line_idx.saturating_sub(2);
                        let context_end = (line_idx + 3).min(lines.len());

                        all_matches.push(SearchMatch {
                            path: path.clone(),
                            line_number: line_idx + 1,
                            line_content: line.to_string(),
                            match_start: mat.start(),
                            match_end: mat.end(),
                            context_before: lines[context_start..line_idx]
                                .iter()
                                .map(|s| s.to_string())
                                .collect(),
                            context_after: lines[(line_idx + 1)..context_end]
                                .iter()
                                .map(|s| s.to_string())
                                .collect(),
                        });
                    }
                }
            }
        }

        Ok(all_matches)
    }

    /// Check if a path exists
    pub async fn path_exists(&self, path: &Path) -> Result<bool> {
        Ok(fs::metadata(path).await.is_ok())
    }

    /// Verify path is allowed by security policy
    fn verify_path_allowed(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();

        // Check for denied patterns
        for denied in &self.security_policy.denied_patterns {
            if path_str.contains(denied) {
                anyhow::bail!("Path {} contains denied pattern: {}", path_str, denied);
            }
        }

        // Check file extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if !self.security_policy.allowed_extensions.contains(&ext_str) {
                anyhow::bail!("File extension {} not allowed", ext_str);
            }
        }

        Ok(())
    }

    /// Check if a pattern is denied
    fn is_denied_pattern(&self, name: &str) -> bool {
        self.security_policy
            .denied_patterns
            .iter()
            .any(|pattern| name.contains(pattern))
    }

    /// Get file from cache if available and not expired
    async fn get_from_cache(&self, path: &Path) -> Option<FileContent> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(path) {
            if let Ok(elapsed) = entry.cached_at.elapsed() {
                if elapsed < self.cache_ttl {
                    return Some(entry.content.clone());
                }
            }
        }
        None
    }

    /// Cache file content
    async fn cache_file(&self, path: &Path, content: FileContent) {
        let mut cache = self.cache.write().await;
        cache.insert(
            path.to_path_buf(),
            CacheEntry {
                content,
                cached_at: std::time::SystemTime::now(),
            },
        );

        // Clean up old entries if cache is too large
        if cache.len() > 1000 {
            let now = std::time::SystemTime::now();
            cache.retain(|_, entry| {
                if let Ok(elapsed) = now.duration_since(entry.cached_at) {
                    elapsed < self.cache_ttl
                } else {
                    true
                }
            });
        }
    }

    /// Walk directory recursively with glob matching
    async fn walk_directory(
        &self,
        dir: &Path,
        glob: &globset::GlobMatcher,
        results: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if results.len() >= self.security_policy.max_files_per_request {
            return Ok(());
        }

        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_type = entry.file_type().await?;

            if self.is_denied_pattern(&path.to_string_lossy()) {
                continue;
            }

            if file_type.is_file() && glob.is_match(&path) {
                if self.verify_path_allowed(&path).is_ok() {
                    results.push(path);
                }
            } else if file_type.is_dir() {
                Box::pin(self.walk_directory(&path, glob, results)).await?;
            }
        }

        Ok(())
    }
}

/// Detect programming language from file extension
fn detect_language(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "ts" => "typescript",
            "js" => "javascript",
            "py" => "python",
            "go" => "go",
            "java" => "java",
            "cs" => "csharp",
            "cpp" | "cc" | "cxx" => "cpp",
            "c" => "c",
            "h" | "hpp" => "c-header",
            "md" => "markdown",
            "toml" => "toml",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "html" => "html",
            "css" => "css",
            "sql" => "sql",
            "sh" | "bash" => "bash",
            "dockerfile" => "dockerfile",
            _ => "text",
        })
        .map(String::from)
}

/// Integration with consensus context
impl crate::consensus::repository_context::RepositoryContext {
    /// Read a file for analysis with security checks
    pub async fn read_file_for_analysis(
        &self,
        file_reader: &FileReader,
        path: &Path,
    ) -> HiveResult<FileContent> {
        // Ensure path is within repository root
        if let Some(root) = &self.root_path {
            let canonical_path = path.canonicalize()?;
            let canonical_root = root.canonicalize()?;

            if !canonical_path.starts_with(&canonical_root) {
                return Err(crate::core::error::HiveError::Security { 
                    message: format!(
                        "Path {} is outside repository root {}",
                        path.display(),
                        root.display()
                    )
                }.into());
            }
        }

        file_reader
            .read_file(path)
            .await
            .map_err(|e| e.into())
    }

    /// Read multiple files for comprehensive analysis
    pub async fn read_files_for_analysis(
        &self,
        file_reader: &FileReader,
        paths: &[PathBuf],
    ) -> HiveResult<Vec<FileContent>> {
        let mut contents = Vec::new();

        for path in paths.iter().take(file_reader.security_policy.max_files_per_request) {
            match self.read_file_for_analysis(file_reader, path).await {
                Ok(content) => contents.push(content),
                Err(e) => warn!("Failed to read file {}: {}", path.display(), e),
            }
        }

        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_reader_security() {
        let reader = FileReader::new(SecurityPolicy::default());

        // Test denied patterns
        assert!(reader
            .verify_path_allowed(Path::new(".git/config"))
            .is_err());
        assert!(reader
            .verify_path_allowed(Path::new("node_modules/test.js"))
            .is_err());

        // Test allowed patterns
        assert!(reader.verify_path_allowed(Path::new("src/main.rs")).is_ok());
        assert!(reader
            .verify_path_allowed(Path::new("README.md"))
            .is_ok());
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(detect_language(Path::new("main.rs")), Some("rust".to_string()));
        assert_eq!(
            detect_language(Path::new("app.ts")),
            Some("typescript".to_string())
        );
        assert_eq!(
            detect_language(Path::new("script.py")),
            Some("python".to_string())
        );
    }
}