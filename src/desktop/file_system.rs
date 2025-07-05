//! File System Operations for Desktop File Explorer
//!
//! Provides async file system operations for browsing directories,
//! reading file metadata, and loading file content.

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::fs;
use anyhow::Result;
use chrono::{DateTime, Utc};

use super::state::{FileItem, FileType, GitFileStatus};

/// Load a directory tree recursively with expansion state
pub fn load_directory_tree<'a>(
    root: &'a Path,
    expanded_dirs: &'a HashMap<PathBuf, bool>,
    show_hidden: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FileItem>>> + Send + 'a>> {
    Box::pin(async move {
        load_directory_tree_inner(root, expanded_dirs, show_hidden).await
    })
}

/// Inner recursive function for loading directory tree
fn load_directory_tree_inner<'a>(
    root: &'a Path,
    expanded_dirs: &'a HashMap<PathBuf, bool>,
    show_hidden: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FileItem>>> + Send + 'a>> {
    Box::pin(async move {
    let mut entries = Vec::new();
    
    // Read directory entries
    let mut dir = fs::read_dir(root).await?;
    
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        let metadata = entry.metadata().await?;
        let name = entry.file_name().to_string_lossy().to_string();
        
        // Skip hidden files if not showing them
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        
        let is_directory = metadata.is_dir();
        let is_expanded = expanded_dirs.get(&path).copied().unwrap_or(false);
        
        // Get file type from extension
        let file_type = if is_directory {
            FileType::Directory
        } else {
            FileType::from_path(&path)
        };
        
        // Get modification time
        let modified = metadata.modified().ok().map(|t| {
            let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap();
            DateTime::<Utc>::from_timestamp(duration.as_secs() as i64, 0).unwrap()
        });
        
        // Load children if directory is expanded
        let children = if is_directory && is_expanded {
            load_directory_tree_inner(&path, expanded_dirs, show_hidden).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        
        entries.push(FileItem {
            path,
            name,
            is_directory,
            is_expanded,
            children,
            file_type,
            git_status: None, // TODO: Implement git status detection
            size: if !is_directory { Some(metadata.len()) } else { None },
            modified,
        });
    }
    
    // Sort entries: directories first, then files, alphabetically
    entries.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    Ok(entries)
    })
}

/// Read file content as string
pub async fn read_file_content(path: &Path) -> Result<String> {
    // Check file size first to avoid loading huge files
    let metadata = fs::metadata(path).await?;
    let size = metadata.len();
    
    // Limit to 10MB for safety
    if size > 10 * 1024 * 1024 {
        return Ok(format!(
            "// File too large to display ({:.2} MB)\n// Use external editor for files larger than 10MB",
            size as f64 / 1024.0 / 1024.0
        ));
    }
    
    // Read file content
    let content = fs::read_to_string(path).await?;
    Ok(content)
}

/// Get git status for files (placeholder for now)
pub async fn get_git_status(path: &Path) -> Option<GitFileStatus> {
    // TODO: Implement actual git status detection
    // For now, return None
    None
}

impl FileType {
    /// Determine file type from path extension
    pub fn from_path(path: &Path) -> Self {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "rs" => FileType::Rust,
            "ts" | "tsx" => FileType::TypeScript,
            "js" | "jsx" => FileType::JavaScript,
            "py" => FileType::Python,
            "go" => FileType::Go,
            "java" => FileType::Java,
            "cpp" | "cc" | "cxx" => FileType::CPP,
            "c" | "h" => FileType::C,
            "html" | "htm" => FileType::HTML,
            "css" | "scss" | "sass" => FileType::CSS,
            "json" => FileType::JSON,
            "toml" => FileType::TOML,
            "yaml" | "yml" => FileType::YAML,
            "xml" => FileType::XML,
            "md" | "markdown" => FileType::Markdown,
            "txt" => FileType::Text,
            "sh" | "bash" => FileType::Shell,
            "dockerfile" => FileType::Docker,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => FileType::Image,
            "exe" | "bin" | "dll" | "so" | "dylib" => FileType::Binary,
            _ => FileType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    
    #[tokio::test]
    async fn test_load_directory_tree() {
        // Create temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        // Create some test files
        fs::create_dir(root.join("src")).await.unwrap();
        File::create(root.join("Cargo.toml")).await.unwrap();
        File::create(root.join("README.md")).await.unwrap();
        File::create(root.join("src/main.rs")).await.unwrap();
        File::create(root.join(".gitignore")).await.unwrap();
        
        // Test without hidden files
        let expanded = HashMap::new();
        let entries = load_directory_tree_inner(root, &expanded, false).await.unwrap();
        
        assert_eq!(entries.len(), 3); // src/, Cargo.toml, README.md
        assert!(entries[0].is_directory); // src should be first
        assert_eq!(entries[0].name, "src");
        
        // Test with hidden files
        let entries = load_directory_tree_inner(root, &expanded, true).await.unwrap();
        assert_eq!(entries.len(), 4); // includes .gitignore
    }
    
    #[tokio::test]
    async fn test_read_file_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Write test content
        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(b"Hello, World!").await.unwrap();
        
        // Read content
        let content = read_file_content(&file_path).await.unwrap();
        assert_eq!(content, "Hello, World!");
    }
    
    #[test]
    fn test_file_type_from_path() {
        assert_eq!(FileType::from_path(Path::new("test.rs")), FileType::Rust);
        assert_eq!(FileType::from_path(Path::new("app.tsx")), FileType::TypeScript);
        assert_eq!(FileType::from_path(Path::new("script.js")), FileType::JavaScript);
        assert_eq!(FileType::from_path(Path::new("unknown.xyz")), FileType::Unknown);
    }
}