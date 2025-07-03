//! Preview generation for code transformations

use anyhow::{Result, anyhow};
use std::path::Path;
use crate::transformation::types::*;
use similar::{ChangeTag, TextDiff};
use colored::*;

/// Generates previews for code transformations
pub struct PreviewSystem {
    max_context_lines: usize,
}

impl PreviewSystem {
    pub fn new() -> Self {
        Self {
            max_context_lines: 3,
        }
    }

    /// Generate a preview for a transformation
    pub async fn generate_preview(&self, transformation: &Transformation) -> Result<TransformationPreview> {
        let mut diffs = Vec::new();
        let mut total_additions = 0;
        let mut total_deletions = 0;
        let mut modified_functions = HashSet::new();

        // Generate diff for each file
        for change in &transformation.changes {
            let diff = self.generate_file_diff(change).await?;
            total_additions += diff.additions;
            total_deletions += diff.deletions;
            diffs.push(diff);

            // Extract modified function names (simplified)
            if let Some(functions) = self.extract_modified_functions(change) {
                modified_functions.extend(functions);
            }
        }

        // Analyze impact
        let impact = ImpactAnalysis {
            files_modified: diffs.len(),
            files_affected: self.estimate_affected_files(&transformation.changes),
            functions_modified: modified_functions.into_iter().collect(),
            risk_level: self.assess_risk_level(total_additions, total_deletions, &transformation.changes),
            tests_affected: self.check_tests_affected(&transformation.changes),
        };

        // Generate warnings
        let warnings = self.generate_warnings(&transformation.changes, &impact);

        Ok(TransformationPreview {
            transformation: transformation.clone(),
            diffs,
            warnings,
            impact,
        })
    }

    /// Generate diff for a single file change
    async fn generate_file_diff(&self, change: &CodeChange) -> Result<FileDiff> {
        let diff = TextDiff::from_lines(&change.original_content, &change.new_content);
        
        let mut unified_diff = String::new();
        let mut additions = 0;
        let mut deletions = 0;

        // Generate unified diff format
        unified_diff.push_str(&format!("--- {}\n", change.file_path.display()));
        unified_diff.push_str(&format!("+++ {}\n", change.file_path.display()));

        for (idx, group) in diff.grouped_ops(self.max_context_lines).iter().enumerate() {
            if idx > 0 {
                unified_diff.push_str("\n");
            }

            let (tag, old_range, new_range) = group[0].as_tag_tuple();
            unified_diff.push_str(&format!("@@ -{},{} +{},{} @@\n", 
                old_range.start + 1, old_range.len(), new_range.start + 1, new_range.len()));

            for op in group {
                for change in diff.iter_changes(op) {
                    let sign = match change.tag() {
                        ChangeTag::Delete => {
                            deletions += 1;
                            "-"
                        },
                        ChangeTag::Insert => {
                            additions += 1;
                            "+"
                        },
                        ChangeTag::Equal => " ",
                    };

                    unified_diff.push_str(&format!("{}{}", sign, change));
                }
            }
        }

        Ok(FileDiff {
            file_path: change.file_path.clone(),
            unified_diff,
            additions,
            deletions,
        })
    }

    /// Generate colored preview for terminal display
    pub fn format_preview_for_terminal(&self, preview: &TransformationPreview) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("\n{}\n", "=== Transformation Preview ===".bold()));
        output.push_str(&format!("{}: {}\n", "Description".bold(), preview.transformation.description));
        output.push_str(&format!("{}: {}\n", "Aspect".bold(), preview.transformation.request.aspect));
        output.push_str(&format!("{}: {} files modified\n\n", "Impact".bold(), preview.impact.files_modified));

        // Risk assessment
        let risk_color = match preview.impact.risk_level {
            RiskLevel::Low => "green",
            RiskLevel::Medium => "yellow",
            RiskLevel::High => "red",
        };
        output.push_str(&format!("{}: {}\n\n", 
            "Risk Level".bold(), 
            format!("{:?}", preview.impact.risk_level).color(risk_color)
        ));

        // File diffs
        for diff in &preview.diffs {
            output.push_str(&format!("{} {}\n", "File:".bold(), diff.file_path.display()));
            output.push_str(&format!("  {} additions, {} deletions\n\n", 
                format!("+{}", diff.additions).green(),
                format!("-{}", diff.deletions).red()
            ));

            // Show the actual diff
            for line in diff.unified_diff.lines() {
                if line.starts_with('+') && !line.starts_with("+++") {
                    output.push_str(&format!("{}\n", line.green()));
                } else if line.starts_with('-') && !line.starts_with("---") {
                    output.push_str(&format!("{}\n", line.red()));
                } else if line.starts_with("@@") {
                    output.push_str(&format!("{}\n", line.cyan()));
                } else {
                    output.push_str(&format!("{}\n", line));
                }
            }
            output.push_str("\n");
        }

        // Warnings
        if !preview.warnings.is_empty() {
            output.push_str(&format!("{}\n", "Warnings:".yellow().bold()));
            for warning in &preview.warnings {
                output.push_str(&format!("  ⚠️  {}\n", warning));
            }
            output.push_str("\n");
        }

        // Modified functions
        if !preview.impact.functions_modified.is_empty() {
            output.push_str(&format!("{} ", "Modified functions:".bold()));
            output.push_str(&preview.impact.functions_modified.join(", "));
            output.push_str("\n\n");
        }

        output
    }

    /// Extract function names that were modified
    fn extract_modified_functions(&self, change: &CodeChange) -> Option<Vec<String>> {
        // This is a simplified implementation
        // In production, we'd use the AST to accurately identify functions
        let mut functions = Vec::new();
        
        // Simple regex-based extraction for common languages
        let content = &change.original_content;
        let lines: Vec<&str> = content.lines().collect();
        
        let (start, end) = change.line_range;
        for i in start.saturating_sub(5)..end.min(lines.len() + 5) {
            if i < lines.len() {
                let line = lines[i];
                
                // Simple patterns for function detection
                if line.contains("fn ") || line.contains("function ") || 
                   line.contains("def ") || line.contains("func ") {
                    if let Some(name) = self.extract_function_name(line) {
                        functions.push(name);
                    }
                }
            }
        }

        if functions.is_empty() {
            None
        } else {
            Some(functions)
        }
    }

    /// Extract function name from a line
    fn extract_function_name(&self, line: &str) -> Option<String> {
        // Simple extraction - would be more sophisticated in production
        let patterns = vec![
            (r"fn\s+(\w+)", 1),
            (r"function\s+(\w+)", 1),
            (r"def\s+(\w+)", 1),
            (r"func\s+(\w+)", 1),
        ];

        for (pattern, group) in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(line) {
                    if let Some(name) = captures.get(group) {
                        return Some(name.as_str().to_string());
                    }
                }
            }
        }

        None
    }

    /// Estimate number of files potentially affected
    fn estimate_affected_files(&self, changes: &[CodeChange]) -> usize {
        // This would analyze imports/dependencies to estimate impact
        // For now, return a simple estimate
        changes.len() * 2
    }

    /// Assess risk level of the transformation
    fn assess_risk_level(&self, additions: usize, deletions: usize, changes: &[CodeChange]) -> RiskLevel {
        let total_changes = additions + deletions;
        
        // Simple heuristic
        if total_changes < 10 {
            RiskLevel::Low
        } else if total_changes < 50 || changes.len() <= 2 {
            RiskLevel::Medium
        } else {
            RiskLevel::High
        }
    }

    /// Check if tests are likely affected
    fn check_tests_affected(&self, changes: &[CodeChange]) -> bool {
        // Check if any changed files are in test directories
        // or if any test files import the changed modules
        changes.iter().any(|change| {
            let path_str = change.file_path.to_string_lossy();
            path_str.contains("test") || path_str.contains("spec")
        })
    }

    /// Generate warnings based on the changes
    fn generate_warnings(&self, changes: &[CodeChange], impact: &ImpactAnalysis) -> Vec<String> {
        let mut warnings = Vec::new();

        // High risk warning
        if impact.risk_level == RiskLevel::High {
            warnings.push("High risk transformation - review carefully before applying".to_string());
        }

        // Multiple file warning
        if changes.len() > 5 {
            warnings.push(format!("This transformation affects {} files", changes.len()));
        }

        // Low confidence warning
        for change in changes {
            if change.confidence < 0.7 {
                warnings.push(format!(
                    "Low confidence ({:.0}%) for change in {}", 
                    change.confidence * 100.0,
                    change.file_path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }

        // Test warning
        if impact.tests_affected {
            warnings.push("Tests may be affected - consider running test suite after applying".to_string());
        }

        warnings
    }
}

use std::collections::HashSet;

/// Generate a preview from a transformation request (convenience function)
pub async fn generate_preview(transformation: &Transformation) -> Result<TransformationPreview> {
    let preview_system = PreviewSystem::new();
    preview_system.generate_preview(transformation).await
}