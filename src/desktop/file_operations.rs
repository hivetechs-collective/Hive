//! File system operations for the desktop IDE

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Create a new file with optional initial content
pub async fn create_file(path: &Path, content: Option<&str>) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await
            .context("Failed to create parent directories")?;
    }
    
    // Create the file
    let mut file = tokio::fs::File::create(path).await
        .context("Failed to create file")?;
    
    // Write initial content if provided
    if let Some(content) = content {
        file.write_all(content.as_bytes()).await
            .context("Failed to write initial content")?;
    }
    
    Ok(())
}

/// Create a new directory
pub async fn create_folder(path: &Path) -> Result<()> {
    tokio::fs::create_dir_all(path).await
        .context("Failed to create directory")?;
    Ok(())
}

/// Rename a file or directory
pub async fn rename_item(old_path: &Path, new_path: &Path) -> Result<()> {
    tokio::fs::rename(old_path, new_path).await
        .context("Failed to rename item")?;
    Ok(())
}

/// Delete a file or directory
pub async fn delete_item(path: &Path) -> Result<()> {
    if path.is_dir() {
        tokio::fs::remove_dir_all(path).await
            .context("Failed to remove directory")?;
    } else {
        tokio::fs::remove_file(path).await
            .context("Failed to remove file")?;
    }
    Ok(())
}

/// Copy a file or directory
pub async fn copy_item(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        copy_dir_recursive(src, dst).await?;
    } else {
        // Ensure parent directory exists
        if let Some(parent) = dst.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::copy(src, dst).await
            .context("Failed to copy file")?;
    }
    Ok(())
}

/// Move a file or directory (cut & paste)
pub async fn move_item(src: &Path, dst: &Path) -> Result<()> {
    // Try to rename first (most efficient if on same filesystem)
    if tokio::fs::rename(src, dst).await.is_ok() {
        return Ok(());
    }
    
    // If rename fails (different filesystem), copy then delete
    copy_item(src, dst).await?;
    delete_item(src).await?;
    Ok(())
}

/// Duplicate a file or directory with a new name
pub async fn duplicate_item(path: &Path) -> Result<PathBuf> {
    let new_path = generate_duplicate_name(path)?;
    copy_item(path, &new_path).await?;
    Ok(new_path)
}

/// Get file extension template content
pub fn get_file_template(extension: &str) -> &'static str {
    match extension {
        "rs" => r#"//! Module description

use anyhow::Result;

/// Function description
pub fn example() -> Result<()> {
    todo!("Implement me!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert!(example().is_err());
    }
}"#,
        "ts" | "tsx" => r#"/**
 * Module description
 */

export function example(): void {
  // TODO: Implement me!
}
"#,
        "js" | "jsx" => r#"/**
 * Module description
 */

export function example() {
  // TODO: Implement me!
}
"#,
        "py" => r#"#!/usr/bin/env python3
"""Module description."""


def example():
    """Function description."""
    raise NotImplementedError("Implement me!")


if __name__ == "__main__":
    example()
"#,
        "go" => r#"package main

import "fmt"

// Example function
func example() error {
    return fmt.Errorf("not implemented")
}

func main() {
    if err := example(); err != nil {
        fmt.Printf("Error: %v\n", err)
    }
}
"#,
        "html" => r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>New Page</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
        }
    </style>
</head>
<body>
    <h1>Hello, World!</h1>
</body>
</html>"#,
        "css" => r#"/* Styles */
* {
    box-sizing: border-box;
}

body {
    margin: 0;
    padding: 0;
}
"#,
        "json" => r#"{
  "name": "example",
  "version": "1.0.0",
  "description": ""
}"#,
        "md" => r#"# Title

## Description

Content goes here.
"#,
        "toml" => r#"[package]
name = "example"
version = "0.1.0"

[dependencies]
"#,
        "yaml" | "yml" => r#"name: example
version: 1.0.0
description: ""
"#,
        _ => "", // Empty file for unknown extensions
    }
}

/// Copy a directory recursively
async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    tokio::fs::create_dir_all(dst).await?;
    
    let mut entries = tokio::fs::read_dir(src).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            tokio::fs::copy(&src_path, &dst_path).await?;
        }
    }
    
    Ok(())
}

/// Generate a unique name for duplicating a file/folder
fn generate_duplicate_name(path: &Path) -> Result<PathBuf> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let file_stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let extension = path.extension()
        .and_then(|s| s.to_str());
    
    // Try different suffixes until we find one that doesn't exist
    for i in 1..1000 {
        let new_name = if let Some(ext) = extension {
            format!("{} copy {}.{}", file_stem, i, ext)
        } else {
            format!("{} copy {}", file_stem, i)
        };
        
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return Ok(new_path);
        }
    }
    
    anyhow::bail!("Could not generate unique name for duplicate")
}

/// Open a path in the system file manager
pub fn reveal_in_finder(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .context("Failed to open Finder")?;
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try different file managers
        let managers = ["xdg-open", "nautilus", "dolphin", "thunar", "pcmanfm"];
        let mut success = false;
        
        for manager in managers {
            if std::process::Command::new(manager)
                .arg(path.parent().unwrap_or(path))
                .spawn()
                .is_ok()
            {
                success = true;
                break;
            }
        }
        
        if !success {
            anyhow::bail!("Could not find a file manager to open");
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()
            .context("Failed to open Explorer")?;
    }
    
    Ok(())
}

/// Open a terminal at the given directory
pub fn open_in_terminal(path: &Path) -> Result<()> {
    let dir = if path.is_dir() { path } else { path.parent().unwrap_or(Path::new(".")) };
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(dir)
            .spawn()
            .context("Failed to open Terminal")?;
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try different terminals
        let terminals = ["gnome-terminal", "konsole", "xfce4-terminal", "xterm"];
        let mut success = false;
        
        for terminal in terminals {
            if std::process::Command::new(terminal)
                .current_dir(dir)
                .spawn()
                .is_ok()
            {
                success = true;
                break;
            }
        }
        
        if !success {
            anyhow::bail!("Could not find a terminal emulator");
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg("cmd")
            .current_dir(dir)
            .spawn()
            .context("Failed to open Command Prompt")?;
    }
    
    Ok(())
}

/// Copy path to system clipboard
pub fn copy_path_to_clipboard(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy().to_string();
    
    // Use arboard for cross-platform clipboard support
    let mut clipboard = arboard::Clipboard::new()
        .context("Failed to access clipboard")?;
    clipboard.set_text(path_str)
        .context("Failed to copy to clipboard")?;
    
    Ok(())
}