//! Repository Scanner - Shared functionality for comprehensive file scanning
//!
//! This module provides shared scanning capabilities used by both the FileAwareGeneratorStage
//! and FileAwareCuratorStage to discover and prioritize repository files.

use std::path::{Path, PathBuf};
use anyhow::Result;
use tracing::{info, warn};
use tokio::fs;

/// File priority for reading order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilePriority {
    Critical = 0,  // Project definition files (Cargo.toml, package.json, etc.)
    High = 1,      // Main entry points (main.rs, lib.rs, index.js, etc.)
    Medium = 2,    // Configuration files
    Normal = 3,    // Regular source files
    Low = 4,       // Tests, documentation, etc.
}

/// Information about a discovered file
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub priority: FilePriority,
    pub size: u64,
}

/// Repository scanner for discovering and prioritizing files
pub struct RepositoryScanner;

impl RepositoryScanner {
    /// Scan repository files recursively with intelligent filtering
    pub async fn scan_repository_files(root_path: &Path) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        Self::scan_directory_recursive(root_path, root_path, &mut files, 0).await?;
        
        // Sort by priority and relevance
        files.sort_by(|a, b| {
            match a.priority.cmp(&b.priority) {
                std::cmp::Ordering::Equal => a.path.cmp(&b.path),
                other => other
            }
        });
        
        info!("📁 Repository scan complete: {} total files", files.len());
        Ok(files)
    }
    
    /// Recursively scan directory with depth limiting
    async fn scan_directory_recursive(
        current_path: &Path,
        root_path: &Path,
        files: &mut Vec<FileInfo>,
        depth: u32,
    ) -> Result<()> {
        // Prevent infinite recursion and skip very deep directories
        if depth > 8 {
            return Ok(());
        }
        
        let mut entries = match fs::read_dir(current_path).await {
            Ok(entries) => entries,
            Err(_) => return Ok(()), // Skip directories we can't read
        };
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Skip hidden files and directories (except .gitignore, .env files)
            if file_name.starts_with('.') && !Self::is_important_dotfile(file_name) {
                continue;
            }
            
            // Skip common build/cache directories
            if Self::should_skip_directory(file_name) {
                continue;
            }
            
            if path.is_dir() {
                // Recursively scan subdirectories
                Self::scan_directory_recursive(&path, root_path, files, depth + 1).await?;
            } else if Self::is_source_file(&path) {
                let priority = Self::determine_file_priority(&path, root_path);
                files.push(FileInfo {
                    path,
                    priority,
                    size: entry.metadata().await.map(|m| m.len()).unwrap_or(0),
                });
            }
        }
        
        Ok(())
    }
    
    /// Check if a dotfile is important to include
    fn is_important_dotfile(name: &str) -> bool {
        matches!(name, ".gitignore" | ".env" | ".env.example" | ".dockerignore" | ".editorconfig")
    }
    
    /// Check if we should skip a directory entirely
    fn should_skip_directory(name: &str) -> bool {
        matches!(name,
            "target" | "node_modules" | "dist" | "build" | ".git" | 
            "coverage" | ".vscode" | ".idea" | "__pycache__" |
            "vendor" | "deps" | ".next" | ".nuxt" | "out"
        )
    }
    
    /// Check if a file is a source file we should read
    fn is_source_file(path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            matches!(extension.to_lowercase().as_str(),
                "rs" | "js" | "ts" | "jsx" | "tsx" | "py" | "go" | "java" | 
                "cpp" | "c" | "h" | "hpp" | "cs" | "php" | "rb" | "swift" |
                "kt" | "scala" | "clj" | "hs" | "ml" | "elm" | "vue" | "svelte" |
                "toml" | "yaml" | "yml" | "json" | "xml" | "md" | "txt" |
                "dockerfile" | "makefile" | "cmake" | "proto" | "sql"
            )
        } else {
            // Check for files without extensions that might be important
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                matches!(name.to_lowercase().as_str(),
                    "dockerfile" | "makefile" | "rakefile" | "gemfile" | 
                    "procfile" | "requirements" | "license" | "readme"
                )
            } else {
                false
            }
        }
    }
    
    /// Determine file priority for reading order
    fn determine_file_priority(path: &Path, root_path: &Path) -> FilePriority {
        let relative_path = path.strip_prefix(root_path).unwrap_or(path);
        let path_str = relative_path.to_string_lossy().to_lowercase();
        
        // Critical files that define the project
        if path_str.ends_with("cargo.toml") || path_str.ends_with("package.json") ||
           path_str.ends_with("pyproject.toml") || path_str.ends_with("go.mod") ||
           path_str == "readme.md" || path_str == "license" {
            return FilePriority::Critical;
        }
        
        // Main entry points
        if path_str.ends_with("main.rs") || path_str.ends_with("lib.rs") ||
           path_str.ends_with("main.py") || path_str.ends_with("__init__.py") ||
           path_str.ends_with("index.js") || path_str.ends_with("index.ts") ||
           path_str.ends_with("app.js") || path_str.ends_with("app.py") {
            return FilePriority::High;
        }
        
        // Configuration files
        if path_str.contains("config") || path_str.ends_with(".toml") ||
           path_str.ends_with(".yaml") || path_str.ends_with(".yml") ||
           path_str.ends_with(".env") {
            return FilePriority::Medium;
        }
        
        // Test files
        if path_str.contains("test") || path_str.contains("spec") ||
           path_str.starts_with("tests/") {
            return FilePriority::Low;
        }
        
        // Documentation
        if path_str.ends_with(".md") || path_str.contains("doc") {
            return FilePriority::Low;
        }
        
        // Everything else
        FilePriority::Normal
    }
    
    /// Prioritize files for reading based on importance
    pub fn prioritize_files_for_reading(files: &[FileInfo]) -> Vec<&FileInfo> {
        let mut prioritized = files.iter().collect::<Vec<_>>();
        
        prioritized.sort_by(|a, b| {
            // First by priority
            match a.priority.cmp(&b.priority) {
                std::cmp::Ordering::Equal => {
                    // Then by size (smaller files first for context efficiency)
                    a.size.cmp(&b.size)
                }
                other => other
            }
        });
        
        prioritized
    }
    
    /// Add a summary of repository structure to context
    pub fn add_repository_summary(context: &mut String, files: &[FileInfo]) {
        context.push_str("## Repository Analysis Summary\n\n");
        
        // Count files by type
        let mut by_language: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for file in files {
            if let Some(ext) = file.path.extension().and_then(|e| e.to_str()) {
                *by_language.entry(ext.to_lowercase()).or_insert(0) += 1;
            }
        }
        
        context.push_str("**Languages detected:**\n");
        for (lang, count) in by_language.iter() {
            context.push_str(&format!("- {}: {} files\n", lang, count));
        }
        
        context.push_str(&format!("\n**Total files scanned:** {}\n", files.len()));
        context.push_str(&format!("**Critical files:** {}\n", 
            files.iter().filter(|f| f.priority == FilePriority::Critical).count()));
        context.push_str(&format!("**High priority files:** {}\n\n", 
            files.iter().filter(|f| f.priority == FilePriority::High).count()));
    }
}