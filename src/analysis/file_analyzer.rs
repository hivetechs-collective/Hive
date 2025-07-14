//! File analysis utilities for finding relevant files

use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Analyzes files to find relevant targets for transformations
pub struct FileAnalyzer;

impl FileAnalyzer {
    /// Create a new file analyzer
    pub fn new() -> Self {
        Self
    }

    /// Find files relevant to a given query
    pub async fn find_files(&self, path: &Path, query: Option<&str>) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        // Define common source file extensions
        let extensions = vec![
            "rs", "py", "js", "ts", "jsx", "tsx", "java", "c", "cpp", "cc", "h", "hpp", "go", "rb",
            "php", "swift", "kt", "scala", "cs", "vb", "f#", "ml", "clj", "ex", "exs", "erl",
            "hrl", "lua", "r", "jl", "nim", "zig", "dart", "vue", "svelte", "sql", "sh", "bash",
            "zsh", "fish", "ps1", "psm1", "psd1",
        ];

        // Walk directory
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip hidden directories and common build/dependency directories
            if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if dir_name.starts_with('.')
                    || matches!(
                        dir_name,
                        "target"
                            | "node_modules"
                            | "dist"
                            | "build"
                            | "__pycache__"
                            | ".git"
                            | ".svn"
                            | "vendor"
                            | "deps"
                    )
                {
                    continue;
                }
            }

            // Check if it's a file with a relevant extension
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        // If query is provided, check if file name or path contains it
                        if let Some(q) = query {
                            let path_str = path.to_string_lossy().to_lowercase();
                            let query_lower = q.to_lowercase();
                            if path_str.contains(&query_lower) {
                                files.push(path.to_path_buf());
                            }
                        } else {
                            files.push(path.to_path_buf());
                        }
                    }
                }
            }
        }

        // Sort by path for consistent ordering
        files.sort();

        Ok(files)
    }
}

impl Default for FileAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
