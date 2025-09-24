//! Inline Operation Display Component
//!
//! Displays file operations inline in the response, similar to Claude Code.
//! Shows operations as they happen with appropriate formatting and indicators.

use dioxus::prelude::*;
use std::path::PathBuf;

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::desktop::styles::theme::ThemeColors;

/// Inline operation display component
#[component]
pub fn InlineOperationDisplay(
    operation: FileOperation,
    status: OperationStatus,
    theme: ThemeColors,
) -> Element {
    let (icon, status_text, status_color) = match &status {
        OperationStatus::Pending => ("⏳", "Pending", theme.text_secondary.clone()),
        OperationStatus::Executing => ("⚡", "Executing", theme.warning.clone()),
        OperationStatus::Completed => ("✅", "Completed", theme.success.clone()),
        OperationStatus::Failed(error) => ("❌", error.as_str(), theme.error.clone()),
        OperationStatus::Skipped(reason) => ("⏭", reason.as_str(), theme.text_secondary.clone()),
    };

    let operation_desc = match &operation {
        FileOperation::Create { path, .. } => format!("Creating `{}`", path.display()),
        FileOperation::Update { path, .. } => format!("Updating `{}`", path.display()),
        FileOperation::Delete { path } => format!("Deleting `{}`", path.display()),
        FileOperation::Rename { from, to } => {
            format!("Renaming `{}` to `{}`", from.display(), to.display())
        }
        FileOperation::Append { path, .. } => format!("Appending to `{}`", path.display()),
    };

    rsx! {
        div {
            style: "margin: 8px 0; padding: 8px 12px; background: {theme.background_secondary}; border: 1px solid {theme.border}; border-radius: 4px; font-family: monospace; font-size: 13px; display: flex; align-items: center; gap: 8px;",

            // Status icon
            span {
                style: "font-size: 14px;",
                "{icon}"
            }

            // Operation description
            span {
                style: "flex: 1; color: {theme.text};",
                "{operation_desc}"
            }

            // Status text
            if !status_text.is_empty() {
                span {
                    style: "color: {status_color}; font-size: 11px; text-transform: uppercase; font-weight: 500;",
                    "{status_text}"
                }
            }
        }
    }
}

/// Operation status
#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Pending,
    Executing,
    Completed,
    Failed(String),
    Skipped(String),
}

/// Response section component for mixed content and operations
#[component]
pub fn ResponseSection(
    content: String,
    operations: Vec<(FileOperation, OperationStatus)>,
    theme: ThemeColors,
) -> Element {
    rsx! {
        div {
            style: "margin: 0; padding: 0;",

            // Split content by operation markers and interleave
            for (idx, section) in split_content_with_operations(&content, &operations).into_iter().enumerate() {
                match section {
                    ContentSection::Text(text) => rsx! {
                        if !text.trim().is_empty() {
                            div {
                                style: "margin: 8px 0;",
                                dangerous_inner_html: "{text}"
                            }
                        }
                    },
                    ContentSection::Operation(op, status) => rsx! {
                        InlineOperationDisplay {
                            operation: op,
                            status: status,
                            theme: theme.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Content section type
#[derive(Debug, Clone)]
enum ContentSection {
    Text(String),
    Operation(FileOperation, OperationStatus),
}

/// Split content with inline operations
fn split_content_with_operations(
    content: &str,
    operations: &[(FileOperation, OperationStatus)],
) -> Vec<ContentSection> {
    // For now, simple implementation - just interleave text and operations
    // In a real implementation, we'd parse the content for operation markers

    let mut sections = Vec::new();

    // If we have operations, show them inline
    if !operations.is_empty() {
        // Add initial text if any
        let parts: Vec<&str> = content.split("```").collect();
        if !parts.is_empty() && !parts[0].trim().is_empty() {
            sections.push(ContentSection::Text(parts[0].to_string()));
        }

        // Add operations
        for (op, status) in operations {
            sections.push(ContentSection::Operation(op.clone(), status.clone()));
        }

        // Add remaining text if any
        if parts.len() > 1 {
            let remaining = parts[1..].join("```");
            if !remaining.trim().is_empty() {
                sections.push(ContentSection::Text(format!("```{}", remaining)));
            }
        }
    } else {
        // No operations, just show the text
        sections.push(ContentSection::Text(content.to_string()));
    }

    sections
}

/// Parse operations from streaming content
pub fn parse_operations_from_content(content: &str) -> Vec<FileOperation> {
    let mut operations = Vec::new();

    // Look for operation patterns in the content
    let operation_pattern = regex::Regex::new(
        r"(?i)(Creating|Updating|Modifying|Deleting|Writing to|Adding to)\s+`([^`]+)`",
    )
    .unwrap();

    for capture in operation_pattern.captures_iter(content) {
        if let (Some(action), Some(path)) = (capture.get(1), capture.get(2)) {
            let path_str = path.as_str();
            let path_buf = PathBuf::from(path_str);

            match action.as_str().to_lowercase().as_str() {
                "creating" | "writing to" | "adding to" => {
                    // Extract content after the operation line
                    if let Some(code_block_start) = content.find(&format!("`{}`:", path_str)) {
                        if let Some(code_start) = content[code_block_start..].find("```") {
                            let content_start = code_block_start + code_start + 3;
                            if let Some(code_end) = content[content_start..].find("```") {
                                let file_content =
                                    content[content_start..content_start + code_end].trim();
                                operations.push(FileOperation::Create {
                                    path: path_buf,
                                    content: file_content.to_string(),
                                });
                            }
                        }
                    }
                }
                "updating" | "modifying" => {
                    // Similar extraction for updates
                    operations.push(FileOperation::Update {
                        path: path_buf,
                        content: String::new(), // Would need to extract diff
                    });
                }
                "deleting" => {
                    operations.push(FileOperation::Delete { path: path_buf });
                }
                _ => {}
            }
        }
    }

    operations
}
