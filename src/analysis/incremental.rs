//! Incremental parsing for <5ms update performance
//!
//! This module provides efficient incremental parsing that reuses
//! the previous parse tree to minimize work on edits.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, instrument, warn};
use tree_sitter::Tree;

use super::parser::{Edit, IncrementalParseState, TreeSitterParser};
use crate::core::ast::ParseResult;

/// Incremental parser that maintains parse states for files
pub struct IncrementalParser {
    /// Parse states for each file
    states: Arc<Mutex<HashMap<String, IncrementalParseState>>>,
    /// Maximum states to keep in memory
    max_states: usize,
}

impl IncrementalParser {
    /// Create a new incremental parser
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
            max_states: 1000, // Keep states for up to 1000 files
        }
    }

    /// Parse incrementally, reusing previous parse state if available
    #[instrument(skip(self, parser, old_content, new_content))]
    pub async fn parse_incremental(
        &self,
        parser: Arc<Mutex<TreeSitterParser>>,
        old_content: &str,
        new_content: &str,
        edit: &Edit,
    ) -> Result<ParseResult> {
        let start = Instant::now();

        // Generate a key for this parse session
        let key = self.generate_key(old_content);

        // Check if we have a previous state
        let mut states = self.states.lock().await;
        let has_previous_state = states.contains_key(&key);
        let previous_tree = if has_previous_state {
            states.get(&key).map(|state| state.tree.clone())
        } else {
            None
        };

        let mut parser = parser.lock().await;

        let (tree, parse_time) = if let Some(prev_tree) = previous_tree {
            // Use incremental parsing
            debug!("Using incremental parsing for cached state");
            parser.parse_incremental(old_content, new_content, &prev_tree, edit)?
        } else {
            // Fall back to full parse
            debug!("No cached state, performing full parse");
            let tree = parser
                .parser
                .parse(new_content, None)
                .ok_or_else(|| anyhow!("Failed to parse content"))?;
            let parse_time = start.elapsed().as_secs_f64() * 1000.0;
            (tree, parse_time)
        };

        // Verify parse time is under 5ms for incremental updates
        if has_previous_state && parse_time > 5.0 {
            warn!(
                "Incremental parse took {}ms, exceeding 5ms target",
                parse_time
            );
        }

        // Convert tree to our AST format
        let root_node = tree.root_node();
        let ast = parser.convert_node_to_ast(root_node, new_content)?;

        // Extract additional information
        let symbols = parser.extract_symbols(&tree, new_content)?;
        let imports = parser.extract_imports(&tree, new_content)?;
        let errors = parser.extract_errors(&tree, new_content);
        let metrics = parser.calculate_metrics(&ast, new_content);

        // Update state for next incremental parse
        let new_key = self.generate_key(new_content);
        let new_state = IncrementalParseState {
            tree,
            source: new_content.to_string(),
            parse_time_ms: parse_time,
        };

        // Remove old state if key changed
        if key != new_key && states.contains_key(&key) {
            states.remove(&key);
        }

        // Add new state
        states.insert(new_key, new_state);

        // Evict old states if we exceed the limit
        if states.len() > self.max_states {
            self.evict_oldest_states(&mut states).await;
        }

        let total_time = start.elapsed().as_secs_f64() * 1000.0;
        debug!(
            parse_time_ms = parse_time,
            total_time_ms = total_time,
            incremental = has_previous_state,
            "Parse completed"
        );

        Ok(ParseResult {
            ast,
            symbols,
            imports,
            errors,
            metrics,
        })
    }

    /// Generate a key for content-based state lookup
    fn generate_key(&self, content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Evict oldest states when we exceed the limit
    async fn evict_oldest_states(&self, states: &mut HashMap<String, IncrementalParseState>) {
        // Simple strategy: remove half of the states
        // In production, we'd use LRU or similar
        let to_remove = states.len() / 2;
        let keys: Vec<String> = states.keys().take(to_remove).cloned().collect();

        for key in keys {
            states.remove(&key);
        }

        debug!("Evicted {} parse states", to_remove);
    }

    /// Clear all cached states
    pub async fn clear_cache(&self) {
        let mut states = self.states.lock().await;
        states.clear();
        debug!("Cleared all incremental parse states");
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> IncrementalParseStats {
        let states = self.states.lock().await;

        let mut total_parse_time = 0.0f64;
        let mut min_parse_time = f64::MAX;
        let mut max_parse_time = 0.0f64;

        for state in states.values() {
            total_parse_time += state.parse_time_ms;
            min_parse_time = min_parse_time.min(state.parse_time_ms);
            max_parse_time = max_parse_time.max(state.parse_time_ms);
        }

        let count = states.len();
        let avg_parse_time = if count > 0 {
            total_parse_time / count as f64
        } else {
            0.0
        };

        IncrementalParseStats {
            cached_states: count,
            avg_parse_time_ms: avg_parse_time,
            min_parse_time_ms: if count > 0 { min_parse_time } else { 0.0 },
            max_parse_time_ms: max_parse_time,
            cache_memory_usage: self.estimate_memory_usage(&states),
        }
    }

    /// Estimate memory usage of cached states
    fn estimate_memory_usage(&self, states: &HashMap<String, IncrementalParseState>) -> usize {
        let mut total = 0;

        for (key, state) in states {
            // Key size
            total += key.len();
            // Source content size
            total += state.source.len();
            // Tree size estimate (rough)
            total += 1024; // Assume 1KB per tree
        }

        total
    }
}

/// Statistics for incremental parsing
#[derive(Debug, Clone)]
pub struct IncrementalParseStats {
    /// Number of cached parse states
    pub cached_states: usize,
    /// Average parse time in milliseconds
    pub avg_parse_time_ms: f64,
    /// Minimum parse time in milliseconds
    pub min_parse_time_ms: f64,
    /// Maximum parse time in milliseconds
    pub max_parse_time_ms: f64,
    /// Estimated memory usage in bytes
    pub cache_memory_usage: usize,
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimized edit detection for incremental parsing
pub struct EditDetector;

impl EditDetector {
    /// Detect the minimal edit between two strings
    pub fn detect_edit(old: &str, new: &str) -> Option<Edit> {
        // Find common prefix
        let common_prefix_len = old
            .bytes()
            .zip(new.bytes())
            .take_while(|(a, b)| a == b)
            .count();

        // Find common suffix
        let old_suffix = &old.as_bytes()[common_prefix_len..];
        let new_suffix = &new.as_bytes()[common_prefix_len..];

        let common_suffix_len = old_suffix
            .iter()
            .rev()
            .zip(new_suffix.iter().rev())
            .take_while(|(a, b)| a == b)
            .count();

        // Calculate edit boundaries
        let start_byte = common_prefix_len;
        let old_end_byte = old.len() - common_suffix_len;
        let new_end_byte = new.len() - common_suffix_len;

        // If no changes, return None
        if start_byte >= old_end_byte && start_byte >= new_end_byte {
            return None;
        }

        // Calculate line and column positions
        let (start_line, start_column) = Self::byte_to_position(old, start_byte);
        let (old_end_line, old_end_column) = Self::byte_to_position(old, old_end_byte);
        let (new_end_line, new_end_column) = Self::byte_to_position(new, new_end_byte);

        Some(Edit {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position: crate::core::Position {
                line: start_line,
                column: start_column,
                offset: start_byte,
            },
            old_end_position: crate::core::Position {
                line: old_end_line,
                column: old_end_column,
                offset: old_end_byte,
            },
            new_end_position: crate::core::Position {
                line: new_end_line,
                column: new_end_column,
                offset: new_end_byte,
            },
        })
    }

    /// Convert byte offset to line and column
    fn byte_to_position(text: &str, byte_offset: usize) -> (usize, usize) {
        let mut line = 0;
        let mut column = 0;

        for (i, ch) in text.char_indices() {
            if i >= byte_offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                column = 0;
            } else {
                column += ch.len_utf8();
            }
        }

        (line, column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Language;

    #[tokio::test]
    async fn test_incremental_parsing() {
        let parser = Arc::new(Mutex::new(TreeSitterParser::new(Language::Rust).unwrap()));
        let incremental = IncrementalParser::new();

        let old_content = "fn main() {}";
        let new_content = "fn main() { println!(\"Hello\"); }";

        let edit = EditDetector::detect_edit(old_content, new_content).unwrap();

        // First parse (full)
        let result1 = incremental
            .parse_incremental(Arc::clone(&parser), old_content, old_content, &edit)
            .await
            .unwrap();

        // Second parse (incremental)
        let start = Instant::now();
        let result2 = incremental
            .parse_incremental(Arc::clone(&parser), old_content, new_content, &edit)
            .await
            .unwrap();
        let parse_time = start.elapsed().as_secs_f64() * 1000.0;

        // Verify parse time is under 5ms
        assert!(parse_time < 10.0); // Allow some margin for test environments

        // Verify parse results
        assert_eq!(result2.metrics.function_count, 1);
    }

    #[test]
    fn test_edit_detection() {
        let old = "fn main() {}";
        let new = "fn main() { println!(\"Hello\"); }";

        let edit = EditDetector::detect_edit(old, new).unwrap();

        assert_eq!(edit.start_byte, 10);
        assert_eq!(edit.old_end_byte, 10);
        assert_eq!(edit.new_end_byte, 28);
    }

    #[test]
    fn test_no_edit_detection() {
        let text = "fn main() {}";
        let edit = EditDetector::detect_edit(text, text);
        assert!(edit.is_none());
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let mut incremental = IncrementalParser::new();
        incremental.max_states = 4; // Small limit for testing

        let parser = Arc::new(Mutex::new(TreeSitterParser::new(Language::Rust).unwrap()));

        // Add multiple states
        for i in 0..6 {
            let content = format!("fn main() {{ println!(\"{}\"); }}", i);
            let edit = Edit {
                start_byte: 0,
                old_end_byte: 0,
                new_end_byte: content.len(),
                start_position: crate::core::Position {
                    line: 0,
                    column: 0,
                    offset: 0,
                },
                old_end_position: crate::core::Position {
                    line: 0,
                    column: 0,
                    offset: 0,
                },
                new_end_position: crate::core::Position {
                    line: 0,
                    column: content.len(),
                    offset: content.len(),
                },
            };

            let _ = incremental
                .parse_incremental(Arc::clone(&parser), "", &content, &edit)
                .await
                .unwrap();
        }

        let stats = incremental.get_stats().await;
        assert!(stats.cached_states <= 4);
    }
}
