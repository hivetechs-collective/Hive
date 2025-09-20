//! Codebase Scanner - Deep repository analysis
//!
//! Reads every file in the repository and prepares them for analysis

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::analysis::language_detector::LanguageDetector;
use crate::consensus::file_operations::{FileContent, FileReader, SecurityPolicy};
use crate::core::Language;

/// Result of scanning a repository
#[derive(Debug, Clone)]
pub struct RepositoryScan {
    pub root_path: PathBuf,
    pub files: Vec<ScannedFile>,
    pub total_size_bytes: u64,
    pub languages: Vec<(Language, usize)>, // Language and file count
    pub ignored_paths: Vec<PathBuf>,
}

/// Information about a scanned file
#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub language: Language,
    pub size_bytes: u64,
    pub line_count: usize,
    pub content: String,
    pub is_test: bool,
    pub is_generated: bool,
}

/// Scanner for deep codebase analysis
pub struct CodebaseScanner {
    file_reader: Arc<FileReader>,
    language_detector: LanguageDetector,
    max_file_size: u64,
}

impl CodebaseScanner {
    /// Create a new scanner with default settings
    pub fn new() -> Self {
        Self {
            file_reader: Arc::new(FileReader::new(SecurityPolicy::default())),
            language_detector: LanguageDetector::new(),
            max_file_size: 10 * 1024 * 1024, // 10MB max per file
        }
    }

    /// Scan an entire repository
    pub async fn scan_repository(&self, root_path: &Path) -> Result<RepositoryScan> {
        info!("Starting deep scan of repository: {}", root_path.display());

        let mut files = Vec::new();
        let mut total_size = 0u64;
        let mut language_stats = std::collections::HashMap::new();
        let mut ignored_paths = Vec::new();

        // Walk the entire directory tree
        self.scan_directory(
            root_path,
            root_path,
            &mut files,
            &mut total_size,
            &mut language_stats,
            &mut ignored_paths,
        )
        .await?;

        // Sort languages by file count
        let mut languages: Vec<_> = language_stats.into_iter().collect();
        languages.sort_by(|a, b| b.1.cmp(&a.1));

        info!(
            "Scan complete: {} files, {} bytes, {} languages",
            files.len(),
            total_size,
            languages.len()
        );

        Ok(RepositoryScan {
            root_path: root_path.to_path_buf(),
            files,
            total_size_bytes: total_size,
            languages,
            ignored_paths,
        })
    }

    /// Recursively scan a directory
    async fn scan_directory(
        &self,
        root_path: &Path,
        dir_path: &Path,
        files: &mut Vec<ScannedFile>,
        total_size: &mut u64,
        language_stats: &mut std::collections::HashMap<Language, usize>,
        ignored_paths: &mut Vec<PathBuf>,
    ) -> Result<()> {
        // Check if directory should be ignored
        if self.should_ignore_path(dir_path) {
            ignored_paths.push(dir_path.to_path_buf());
            return Ok(());
        }

        let mut entries = fs::read_dir(dir_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_type = entry.file_type().await?;

            if file_type.is_dir() {
                // Recurse into subdirectory
                Box::pin(self.scan_directory(
                    root_path,
                    &path,
                    files,
                    total_size,
                    language_stats,
                    ignored_paths,
                ))
                .await?;
            } else if file_type.is_file() {
                // Process file
                match self.scan_file(root_path, &path).await {
                    Ok(Some(scanned_file)) => {
                        *total_size += scanned_file.size_bytes;
                        *language_stats.entry(scanned_file.language).or_insert(0) += 1;
                        files.push(scanned_file);
                    }
                    Ok(None) => {
                        // File was ignored
                        ignored_paths.push(path);
                    }
                    Err(e) => {
                        warn!("Failed to scan file {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Scan a single file
    async fn scan_file(&self, root_path: &Path, file_path: &Path) -> Result<Option<ScannedFile>> {
        // Check if file should be ignored
        if self.should_ignore_path(file_path) {
            return Ok(None);
        }

        // Get file metadata
        let metadata = fs::metadata(file_path).await?;
        let size_bytes = metadata.len();

        // Skip files that are too large
        if size_bytes > self.max_file_size {
            debug!(
                "Skipping large file: {} ({} bytes)",
                file_path.display(),
                size_bytes
            );
            return Ok(None);
        }

        // Detect language
        let language = self
            .language_detector
            .detect_from_path(file_path)
            .unwrap_or(Language::Unknown);

        // Skip binary and unknown files
        if matches!(language, Language::Unknown) {
            return Ok(None);
        }

        // Read file content
        let content = match self.file_reader.read_file(file_path).await {
            Ok(FileContent { content, .. }) => content,
            Err(e) => {
                debug!("Could not read file {}: {}", file_path.display(), e);
                return Ok(None);
            }
        };

        // Calculate relative path
        let relative_path = file_path
            .strip_prefix(root_path)
            .unwrap_or(file_path)
            .to_path_buf();

        // Detect if this is a test file
        let is_test = self.is_test_file(&relative_path);

        // Detect if this is generated code
        let is_generated = self.is_generated_file(&content);

        // Count lines
        let line_count = content.lines().count();

        Ok(Some(ScannedFile {
            path: file_path.to_path_buf(),
            relative_path,
            language,
            size_bytes,
            line_count,
            content,
            is_test,
            is_generated,
        }))
    }

    /// Check if a path should be ignored
    fn should_ignore_path(&self, path: &Path) -> bool {
        // Common directories to ignore
        const IGNORE_DIRS: &[&str] = &[
            "target",
            "dist",
            "build",
            "out",
            ".git",
            "node_modules",
            ".idea",
            ".vscode",
            "coverage",
            ".pytest_cache",
            "__pycache__",
            "venv",
            ".env",
            "tmp",
            "temp",
            ".next",
            ".nuxt",
        ];

        // Common file patterns to ignore
        const IGNORE_PATTERNS: &[&str] = &[
            ".DS_Store",
            "Thumbs.db",
            ".lock",
            ".log",
            ".tmp",
            ".cache",
            ".pyc",
            ".pyo",
            ".class",
            ".o",
            ".so",
        ];

        let path_str = path.to_string_lossy();

        // Check directory names
        for component in path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if IGNORE_DIRS.contains(&name) {
                    return true;
                }
            }
        }

        // Check file patterns
        for pattern in IGNORE_PATTERNS {
            if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Detect if a file is a test file
    fn is_test_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();

        path_str.contains("test")
            || path_str.contains("spec")
            || path_str.contains("__test__")
            || path_str.contains("__tests__")
            || path_str.ends_with("_test.rs")
            || path_str.ends_with("_spec.rs")
            || path_str.ends_with(".test.ts")
            || path_str.ends_with(".spec.ts")
            || path_str.ends_with("test.py")
            || path_str.ends_with("_test.go")
    }

    /// Detect if a file is generated code
    fn is_generated_file(&self, content: &str) -> bool {
        // Look for common generated code markers in first 10 lines
        let first_lines: Vec<&str> = content.lines().take(10).collect();
        let header = first_lines.join("\n").to_lowercase();

        header.contains("generated")
            || header.contains("auto-generated")
            || header.contains("autogenerated")
            || header.contains("do not edit")
            || header.contains("do not modify")
            || header.contains("machine generated")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_scanner_basic() {
        let scanner = CodebaseScanner::new();
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create some test files
        fs::write(temp_path.join("main.rs"), "fn main() {}")
            .await
            .unwrap();
        fs::write(temp_path.join("lib.rs"), "pub fn test() {}")
            .await
            .unwrap();
        fs::create_dir(temp_path.join("src")).await.unwrap();
        fs::write(temp_path.join("src/module.rs"), "mod tests;")
            .await
            .unwrap();

        // Scan the directory
        let result = scanner.scan_repository(temp_path).await.unwrap();

        assert_eq!(result.files.len(), 3);
        assert!(result.total_size_bytes > 0);
        assert!(!result.languages.is_empty());
    }

    #[test]
    fn test_ignore_patterns() {
        let scanner = CodebaseScanner::new();

        assert!(scanner.should_ignore_path(Path::new("/project/target/debug")));
        assert!(scanner.should_ignore_path(Path::new("/project/.git/objects")));
        assert!(scanner.should_ignore_path(Path::new("/project/node_modules/package")));
        assert!(scanner.should_ignore_path(Path::new("/project/.DS_Store")));

        assert!(!scanner.should_ignore_path(Path::new("/project/src/main.rs")));
        assert!(!scanner.should_ignore_path(Path::new("/project/Cargo.toml")));
    }

    #[test]
    fn test_is_test_file() {
        let scanner = CodebaseScanner::new();

        assert!(scanner.is_test_file(Path::new("src/lib_test.rs")));
        assert!(scanner.is_test_file(Path::new("tests/integration.rs")));
        assert!(scanner.is_test_file(Path::new("component.test.ts")));
        assert!(scanner.is_test_file(Path::new("__tests__/app.js")));

        assert!(!scanner.is_test_file(Path::new("src/main.rs")));
        assert!(!scanner.is_test_file(Path::new("lib.rs")));
    }
}
