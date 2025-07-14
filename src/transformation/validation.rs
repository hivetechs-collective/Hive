//! Safety validation and syntax checking for transformations
//!
//! This module ensures that all code transformations maintain syntactic correctness
//! and don't break the codebase.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{
    analysis::{LanguageDetector, TreeSitterParser},
    core::{security::check_write_access, Language},
    transformation::types::*,
};
use tokio::sync::Mutex;

/// Validates transformations for safety and correctness
pub struct TransformationValidator {
    parsers: std::collections::HashMap<Language, Arc<Mutex<TreeSitterParser>>>,
    language_detector: LanguageDetector,
}

impl TransformationValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            parsers: std::collections::HashMap::new(),
            language_detector: LanguageDetector::new(),
        }
    }

    /// Get or create parser for a language
    fn get_parser(&mut self, language: Language) -> Result<Arc<Mutex<TreeSitterParser>>> {
        if let Some(parser) = self.parsers.get(&language) {
            Ok(parser.clone())
        } else {
            let parser = Arc::new(Mutex::new(TreeSitterParser::new(language)?));
            self.parsers.insert(language, parser.clone());
            Ok(parser)
        }
    }

    /// Validate a complete transformation before applying
    pub async fn validate_transformation(
        &mut self,
        transformation: &Transformation,
    ) -> Result<ValidationReport> {
        let mut report = ValidationReport {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            syntax_checks: Vec::new(),
            security_checks: Vec::new(),
        };

        // Check each change
        for (idx, change) in transformation.changes.iter().enumerate() {
            self.validate_change(change, idx, &mut report).await?;
        }

        // Check for conflicting changes
        self.check_conflicts(&transformation.changes, &mut report)?;

        Ok(report)
    }

    /// Validate a single code change
    async fn validate_change(
        &mut self,
        change: &CodeChange,
        index: usize,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Security check
        match check_write_access(&change.file_path) {
            Ok(_) => {
                report.security_checks.push(SecurityCheck {
                    file_path: change.file_path.clone(),
                    passed: true,
                    message: "Write access permitted".to_string(),
                });
            }
            Err(e) => {
                report.valid = false;
                report
                    .errors
                    .push(format!("Change {}: Security check failed - {}", index, e));
                report.security_checks.push(SecurityCheck {
                    file_path: change.file_path.clone(),
                    passed: false,
                    message: e.to_string(),
                });
                return Ok(());
            }
        }

        // File existence check
        if !change.file_path.exists() {
            report.valid = false;
            report.errors.push(format!(
                "Change {}: File {} does not exist",
                index,
                change.file_path.display()
            ));
            return Ok(());
        }

        // Language detection
        let language = self.language_detector.detect_from_path(&change.file_path)?;

        // Syntax validation
        let parser = self.get_parser(language)?;
        let mut parser_guard = parser.lock().await;
        match parser_guard.parse(&change.new_content) {
            Ok(parse_result) => {
                if !parse_result.errors.is_empty() {
                    report.warnings.push(format!(
                        "Change {}: Syntax errors found in {}",
                        index,
                        change.file_path.display()
                    ));
                    report.syntax_checks.push(SyntaxCheck {
                        file_path: change.file_path.clone(),
                        passed: false,
                        errors: parse_result
                            .errors
                            .iter()
                            .map(|e| e.message.clone())
                            .collect(),
                    });
                } else {
                    report.syntax_checks.push(SyntaxCheck {
                        file_path: change.file_path.clone(),
                        passed: true,
                        errors: Vec::new(),
                    });
                }
            }
            Err(e) => {
                report.valid = false;
                report.errors.push(format!(
                    "Change {}: Parse error in {} - {}",
                    index,
                    change.file_path.display(),
                    e
                ));
                report.syntax_checks.push(SyntaxCheck {
                    file_path: change.file_path.clone(),
                    passed: false,
                    errors: vec![e.to_string()],
                });
            }
        }

        // Additional validations
        self.validate_imports(change, language, report)?;
        self.validate_formatting(change, report)?;
        self.validate_size_limits(change, index, report)?;

        Ok(())
    }

    /// Extract syntax errors from tree-sitter tree
    fn extract_syntax_errors(&self, tree: &tree_sitter::Tree) -> Vec<String> {
        let mut errors = Vec::new();
        let mut cursor = tree.walk();

        fn visit_node(cursor: &mut tree_sitter::TreeCursor, errors: &mut Vec<String>) {
            let node = cursor.node();
            if node.is_error() || node.is_missing() {
                errors.push(format!(
                    "Syntax error at {}:{}-{}:{}",
                    node.start_position().row + 1,
                    node.start_position().column + 1,
                    node.end_position().row + 1,
                    node.end_position().column + 1
                ));
            }

            if cursor.goto_first_child() {
                loop {
                    visit_node(cursor, errors);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }

        visit_node(&mut cursor, &mut errors);
        errors
    }

    /// Validate imports are properly handled
    fn validate_imports(
        &self,
        change: &CodeChange,
        language: Language,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Language-specific import validation
        match language {
            Language::Rust => self.validate_rust_imports(change, report),
            Language::Python => self.validate_python_imports(change, report),
            Language::JavaScript | Language::TypeScript => self.validate_js_imports(change, report),
            _ => Ok(()),
        }
    }

    fn validate_rust_imports(
        &self,
        change: &CodeChange,
        report: &mut ValidationReport,
    ) -> Result<()> {
        let new_uses: Vec<&str> = change
            .new_content
            .lines()
            .filter(|line| line.trim_start().starts_with("use "))
            .collect();

        let old_uses: Vec<&str> = change
            .original_content
            .lines()
            .filter(|line| line.trim_start().starts_with("use "))
            .collect();

        if new_uses.len() < old_uses.len() {
            report.warnings.push(format!(
                "Removed {} import(s) from {}",
                old_uses.len() - new_uses.len(),
                change.file_path.display()
            ));
        }

        Ok(())
    }

    fn validate_python_imports(
        &self,
        _change: &CodeChange,
        _report: &mut ValidationReport,
    ) -> Result<()> {
        // TODO: Implement Python import validation
        Ok(())
    }

    fn validate_js_imports(
        &self,
        _change: &CodeChange,
        _report: &mut ValidationReport,
    ) -> Result<()> {
        // TODO: Implement JavaScript/TypeScript import validation
        Ok(())
    }

    /// Validate formatting is preserved
    fn validate_formatting(
        &self,
        change: &CodeChange,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Check indentation consistency
        let original_indent = self.detect_indentation(&change.original_content);
        let new_indent = self.detect_indentation(&change.new_content);

        if original_indent != new_indent {
            report.warnings.push(format!(
                "Indentation style changed in {} (was: {:?}, now: {:?})",
                change.file_path.display(),
                original_indent,
                new_indent
            ));
        }

        Ok(())
    }

    fn detect_indentation(&self, content: &str) -> IndentStyle {
        let mut spaces = 0;
        let mut tabs = 0;

        for line in content.lines() {
            if line.starts_with('\t') {
                tabs += 1;
            } else if line.starts_with(' ') {
                spaces += 1;
            }
        }

        if tabs > spaces {
            IndentStyle::Tabs
        } else if spaces > 0 {
            // Detect space count
            let space_counts: Vec<usize> = content
                .lines()
                .filter(|line| line.starts_with(' '))
                .map(|line| line.chars().take_while(|&c| c == ' ').count())
                .collect();

            if space_counts.is_empty() {
                IndentStyle::Spaces(2)
            } else {
                let min_spaces = space_counts.iter().min().unwrap_or(&2);
                IndentStyle::Spaces(*min_spaces)
            }
        } else {
            IndentStyle::Unknown
        }
    }

    /// Validate size limits
    fn validate_size_limits(
        &self,
        change: &CodeChange,
        index: usize,
        report: &mut ValidationReport,
    ) -> Result<()> {
        const MAX_FILE_SIZE: usize = 1_000_000; // 1MB
        const MAX_LINE_LENGTH: usize = 500;

        if change.new_content.len() > MAX_FILE_SIZE {
            report.valid = false;
            report.errors.push(format!(
                "Change {}: File size exceeds limit ({} > {} bytes)",
                index,
                change.new_content.len(),
                MAX_FILE_SIZE
            ));
        }

        for (line_no, line) in change.new_content.lines().enumerate() {
            if line.len() > MAX_LINE_LENGTH {
                report.warnings.push(format!(
                    "Change {}: Line {} exceeds {} characters in {}",
                    index,
                    line_no + 1,
                    MAX_LINE_LENGTH,
                    change.file_path.display()
                ));
            }
        }

        Ok(())
    }

    /// Check for conflicts between changes
    fn check_conflicts(&self, changes: &[CodeChange], report: &mut ValidationReport) -> Result<()> {
        // Group changes by file
        let mut changes_by_file: std::collections::HashMap<&Path, Vec<&CodeChange>> =
            std::collections::HashMap::new();

        for change in changes {
            changes_by_file
                .entry(&change.file_path)
                .or_insert_with(Vec::new)
                .push(change);
        }

        // Check for overlapping changes in the same file
        for (file_path, file_changes) in changes_by_file {
            if file_changes.len() > 1 {
                for i in 0..file_changes.len() {
                    for j in (i + 1)..file_changes.len() {
                        let (start_a, end_a) = file_changes[i].line_range;
                        let (start_b, end_b) = file_changes[j].line_range;

                        if start_a <= end_b && start_b <= end_a {
                            report.warnings.push(format!(
                                "Overlapping changes in {}: lines {}-{} and {}-{}",
                                file_path.display(),
                                start_a,
                                end_a,
                                start_b,
                                end_b
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Validation report
#[derive(Debug)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub syntax_checks: Vec<SyntaxCheck>,
    pub security_checks: Vec<SecurityCheck>,
}

#[derive(Debug)]
pub struct SyntaxCheck {
    pub file_path: PathBuf,
    pub passed: bool,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub struct SecurityCheck {
    pub file_path: PathBuf,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, PartialEq)]
enum IndentStyle {
    Spaces(usize),
    Tabs,
    Unknown,
}

/// Quick validation for preview purposes
pub async fn quick_validate(transformation: &Transformation) -> Result<bool> {
    let mut validator = TransformationValidator::new();
    let report = validator.validate_transformation(transformation).await?;
    Ok(report.valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_indentation_detection() {
        let validator = TransformationValidator::new();

        let space2 = "fn main() {\n  println!(\"Hello\");\n}";
        assert_eq!(validator.detect_indentation(space2), IndentStyle::Spaces(2));

        let tabs = "fn main() {\n\tprintln!(\"Hello\");\n}";
        assert_eq!(validator.detect_indentation(tabs), IndentStyle::Tabs);
    }
}
