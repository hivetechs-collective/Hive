//! Syntax-aware code modification to preserve correctness

use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::analysis::TreeSitterParser;
use crate::core::Language;

/// Handles syntax-aware code modifications
pub struct SyntaxAwareModifier {
    parser: Arc<Mutex<TreeSitterParser>>,
}

impl SyntaxAwareModifier {
    pub fn new(parser: Arc<Mutex<TreeSitterParser>>) -> Self {
        Self { parser }
    }

    /// Apply a modification while preserving syntax correctness
    pub async fn apply_modification(
        &self,
        original: &str,
        modification: &str,
        line_range: (usize, usize),
        language: Language,
    ) -> Result<String> {
        // Parse original to ensure it's valid
        let mut parser = self.parser.lock().await;
        let _original_ast = parser.parse(original)?;
        
        // Split content into lines
        let mut lines: Vec<&str> = original.lines().collect();
        let (start_line, end_line) = line_range;
        
        // Validate line range
        if start_line == 0 || start_line > lines.len() || end_line > lines.len() {
            return Err(anyhow!("Invalid line range: {:?}", line_range));
        }

        // Extract indentation from the first line being replaced
        let indentation = self.extract_indentation(lines[start_line - 1]);
        
        // Apply indentation to modification lines
        let modified_lines: Vec<String> = modification
            .lines()
            .enumerate()
            .map(|(i, line)| {
                if i == 0 && line.trim().is_empty() {
                    line.to_string()
                } else if line.trim().is_empty() {
                    String::new()
                } else {
                    format!("{}{}", indentation, line.trim_start())
                }
            })
            .collect();

        // Replace the lines
        lines.splice(
            (start_line - 1)..end_line,
            modified_lines.iter().map(|s| s.as_str()),
        );

        let result = lines.join("\n");
        
        // Verify the result is still syntactically valid
        self.verify_syntax(&result, language).await?;
        
        Ok(result)
    }

    /// Extract indentation from a line
    fn extract_indentation<'a>(&self, line: &'a str) -> &'a str {
        let trimmed = line.trim_start();
        &line[..line.len() - trimmed.len()]
    }

    /// Verify that code is syntactically valid
    pub async fn verify_syntax(&self, content: &str, language: Language) -> Result<()> {
        let mut parser = self.parser.lock().await;
        match parser.parse(content) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Syntax validation failed: {}", e)),
        }
    }

    /// Smart insertion that respects code structure
    pub fn smart_insert(
        &self,
        original: &str,
        insertion: &str,
        after_line: usize,
        language: Language,
    ) -> Result<String> {
        let lines: Vec<&str> = original.lines().collect();
        
        if after_line > lines.len() {
            return Err(anyhow!("Line {} is beyond file length", after_line));
        }

        // Determine appropriate indentation
        let indentation = if after_line > 0 {
            self.detect_block_indentation(&lines, after_line - 1)
        } else {
            String::new()
        };

        // Format the insertion with proper indentation
        let formatted_insertion = self.format_with_indentation(insertion, &indentation);
        
        // Insert the new content
        let mut result_lines = lines.to_vec();
        result_lines.insert(after_line, &formatted_insertion);
        
        let result = result_lines.join("\n");
        
        // Verify syntax
        self.verify_syntax(&result, language)?;
        
        Ok(result)
    }

    /// Detect appropriate indentation for a block
    fn detect_block_indentation(&self, lines: &[&str], line_idx: usize) -> String {
        // Look at the current line and potentially the next line
        let current_line = lines[line_idx];
        let current_indent = self.extract_indentation(current_line);
        
        // Check if this line opens a block (ends with {, :, etc.)
        let opens_block = current_line.trim_end().ends_with('{') 
            || current_line.trim_end().ends_with(':');
        
        if opens_block {
            // Increase indentation
            if current_indent.contains('\t') {
                format!("{}\t", current_indent)
            } else {
                format!("{}    ", current_indent)
            }
        } else {
            current_indent.to_string()
        }
    }

    /// Format content with specified indentation
    fn format_with_indentation(&self, content: &str, base_indent: &str) -> String {
        content
            .lines()
            .enumerate()
            .map(|(i, line)| {
                if i == 0 || line.trim().is_empty() {
                    line.to_string()
                } else {
                    format!("{}{}", base_indent, line.trim_start())
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Remove code while maintaining syntax correctness
    pub fn remove_lines(
        &self,
        original: &str,
        line_range: (usize, usize),
        language: Language,
    ) -> Result<String> {
        let lines: Vec<&str> = original.lines().collect();
        let (start_line, end_line) = line_range;
        
        if start_line == 0 || start_line > lines.len() || end_line > lines.len() {
            return Err(anyhow!("Invalid line range for removal"));
        }

        // Check if removal would break syntax (e.g., removing only opening brace)
        let removed_content = lines[(start_line - 1)..end_line].join("\n");
        if self.would_break_syntax(&removed_content) {
            return Err(anyhow!("Removal would break syntax structure"));
        }

        let mut result_lines = lines.to_vec();
        result_lines.drain((start_line - 1)..end_line);
        
        let result = result_lines.join("\n");
        
        // Verify syntax
        self.verify_syntax(&result, language)?;
        
        Ok(result)
    }

    /// Check if removing content would break syntax
    fn would_break_syntax(&self, content: &str) -> bool {
        // Count opening and closing braces/brackets/parens
        let opens = content.matches(|c| c == '{' || c == '[' || c == '(').count();
        let closes = content.matches(|c| c == '}' || c == ']' || c == ')').count();
        
        opens != closes
    }

    /// Apply multiple modifications atomically
    pub fn apply_batch_modifications(
        &self,
        original: &str,
        modifications: Vec<(String, (usize, usize))>,
        language: Language,
    ) -> Result<String> {
        // Sort modifications by line number in reverse order
        // This ensures we don't invalidate line numbers as we apply changes
        let mut sorted_mods = modifications;
        sorted_mods.sort_by(|a, b| b.1.0.cmp(&a.1.0));
        
        let mut result = original.to_string();
        
        for (modification, line_range) in sorted_mods {
            result = self.apply_modification(&result, &modification, line_range, language)?;
        }
        
        Ok(result)
    }
}