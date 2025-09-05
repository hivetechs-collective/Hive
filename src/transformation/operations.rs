//! Transform operations and conflict resolution
//!
//! This module provides the core operations for applying transformations
//! using operational transform (OT) algorithms for conflict resolution.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::transformation::types::*;

/// Represents a single atomic operation on code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Insert text at a position
    Insert { position: usize, content: String },
    /// Delete text at a position
    Delete { position: usize, length: usize },
    /// Replace text at a position
    Replace {
        position: usize,
        length: usize,
        content: String,
    },
}

/// Transform operations manager
pub struct OperationsManager {
    /// Active operations by file
    operations: HashMap<PathBuf, Vec<Operation>>,
}

impl OperationsManager {
    /// Create a new operations manager
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }

    /// Convert a CodeChange into a series of operations
    pub fn change_to_operations(&self, change: &CodeChange) -> Result<Vec<Operation>> {
        let mut ops = Vec::new();

        // Calculate the actual text changes
        let original_lines: Vec<&str> = change.original_content.lines().collect();
        let new_lines: Vec<&str> = change.new_content.lines().collect();

        let (start_line, end_line) = change.line_range;

        // Convert line-based changes to character-based operations
        let mut char_pos = 0;
        for (i, line) in original_lines.iter().enumerate() {
            if i < start_line - 1 {
                char_pos += line.len() + 1; // +1 for newline
                continue;
            }
            if i >= end_line {
                break;
            }
        }

        // Calculate what to delete and insert
        let delete_length = original_lines[(start_line - 1)..end_line]
            .iter()
            .map(|l| l.len() + 1)
            .sum::<usize>()
            .saturating_sub(1); // Remove last newline

        let insert_content = new_lines
            [(start_line - 1)..(start_line - 1 + (end_line - start_line + 1)).min(new_lines.len())]
            .join("\n");

        // Create replace operation
        ops.push(Operation::Replace {
            position: char_pos,
            length: delete_length,
            content: insert_content,
        });

        Ok(ops)
    }

    /// Apply an operation to text content
    pub fn apply_operation(&self, content: &str, operation: &Operation) -> Result<String> {
        let chars: Vec<char> = content.chars().collect();

        match operation {
            Operation::Insert { position, content } => {
                if *position > chars.len() {
                    return Err(anyhow!("Insert position {} out of bounds", position));
                }

                let mut result = String::new();
                result.extend(chars[..*position].iter());
                result.push_str(content);
                result.extend(chars[*position..].iter());
                Ok(result)
            }

            Operation::Delete { position, length } => {
                if *position + *length > chars.len() {
                    return Err(anyhow!("Delete range out of bounds"));
                }

                let mut result = String::new();
                result.extend(chars[..*position].iter());
                result.extend(chars[(*position + *length)..].iter());
                Ok(result)
            }

            Operation::Replace {
                position,
                length,
                content,
            } => {
                if *position + *length > chars.len() {
                    return Err(anyhow!("Replace range out of bounds"));
                }

                let mut result = String::new();
                result.extend(chars[..*position].iter());
                result.push_str(content);
                result.extend(chars[(*position + *length)..].iter());
                Ok(result)
            }
        }
    }

    /// Transform operation A against operation B (for concurrent edits)
    pub fn transform_operation(&self, op_a: &Operation, op_b: &Operation) -> Result<Operation> {
        use Operation::*;

        match (op_a, op_b) {
            // Insert vs Insert
            (
                Insert {
                    position: pos_a,
                    content: content_a,
                },
                Insert {
                    position: pos_b,
                    content: content_b,
                },
            ) => {
                if pos_a < pos_b {
                    Ok(op_a.clone())
                } else if pos_a > pos_b {
                    Ok(Insert {
                        position: pos_a + content_b.len(),
                        content: content_a.clone(),
                    })
                } else {
                    // Same position - apply in order
                    Ok(Insert {
                        position: pos_a + content_b.len(),
                        content: content_a.clone(),
                    })
                }
            }

            // Insert vs Delete
            (
                Insert {
                    position: pos_a,
                    content,
                },
                Delete {
                    position: pos_b,
                    length,
                },
            ) => {
                if *pos_a <= *pos_b {
                    Ok(op_a.clone())
                } else if *pos_a > *pos_b + *length {
                    Ok(Insert {
                        position: pos_a - length,
                        content: content.clone(),
                    })
                } else {
                    // Insert in middle of delete - put at delete position
                    Ok(Insert {
                        position: *pos_b,
                        content: content.clone(),
                    })
                }
            }

            // Delete vs Insert
            (
                Delete {
                    position: pos_a,
                    length: len_a,
                },
                Insert {
                    position: pos_b,
                    content,
                },
            ) => {
                if *pos_a + *len_a <= *pos_b {
                    Ok(op_a.clone())
                } else if *pos_a >= *pos_b {
                    Ok(Delete {
                        position: pos_a + content.len(),
                        length: *len_a,
                    })
                } else {
                    // Insert in middle of delete
                    Ok(Delete {
                        position: *pos_a,
                        length: len_a + content.len(),
                    })
                }
            }

            // Delete vs Delete
            (
                Delete {
                    position: pos_a,
                    length: len_a,
                },
                Delete {
                    position: pos_b,
                    length: len_b,
                },
            ) => {
                if *pos_a + *len_a <= *pos_b {
                    Ok(op_a.clone())
                } else if *pos_a >= *pos_b + *len_b {
                    Ok(Delete {
                        position: pos_a - len_b,
                        length: *len_a,
                    })
                } else {
                    // Overlapping deletes
                    let overlap_start = (*pos_a).max(*pos_b);
                    let overlap_end = (*pos_a + *len_a).min(*pos_b + *len_b);
                    let overlap = overlap_end - overlap_start;

                    if *pos_a < *pos_b {
                        Ok(Delete {
                            position: *pos_a,
                            length: len_a - overlap,
                        })
                    } else {
                        Ok(Delete {
                            position: *pos_b,
                            length: len_a - overlap,
                        })
                    }
                }
            }

            // Replace operations
            (
                Replace {
                    position: pos_a,
                    length: len_a,
                    content: content_a,
                },
                Replace {
                    position: pos_b,
                    length: len_b,
                    content: content_b,
                },
            ) => {
                // Simplified handling - in practice this would be more complex
                if *pos_a + *len_a <= *pos_b {
                    Ok(op_a.clone())
                } else if *pos_a >= *pos_b + *len_b {
                    let diff = content_b.len() as i32 - *len_b as i32;
                    Ok(Replace {
                        position: (*pos_a as i32 + diff).max(0) as usize,
                        length: *len_a,
                        content: content_a.clone(),
                    })
                } else {
                    // Overlapping replaces - this is a conflict
                    Err(anyhow!("Conflicting replace operations"))
                }
            }

            _ => {
                // Mixed replace with insert/delete - simplified handling
                Err(anyhow!(
                    "Complex operation transformation not yet implemented"
                ))
            }
        }
    }

    /// Check if two operations conflict
    pub fn operations_conflict(&self, op_a: &Operation, op_b: &Operation) -> bool {
        use Operation::*;

        match (op_a, op_b) {
            (
                Insert {
                    position: pos_a, ..
                },
                Insert {
                    position: pos_b, ..
                },
            ) => pos_a == pos_b,

            (
                Delete {
                    position: pos_a,
                    length: len_a,
                },
                Delete {
                    position: pos_b,
                    length: len_b,
                },
            ) => {
                let range_a = *pos_a..(*pos_a + *len_a);
                let range_b = *pos_b..(*pos_b + *len_b);
                range_a.start < range_b.end && range_b.start < range_a.end
            }

            (
                Insert {
                    position: pos_a, ..
                },
                Delete {
                    position: pos_b,
                    length: len_b,
                },
            )
            | (
                Delete {
                    position: pos_b,
                    length: len_b,
                },
                Insert {
                    position: pos_a, ..
                },
            ) => *pos_a >= *pos_b && *pos_a <= *pos_b + *len_b,

            (
                Replace {
                    position: pos_a,
                    length: len_a,
                    ..
                },
                Replace {
                    position: pos_b,
                    length: len_b,
                    ..
                },
            ) => {
                let range_a = *pos_a..(*pos_a + *len_a);
                let range_b = *pos_b..(*pos_b + *len_b);
                range_a.start < range_b.end && range_b.start < range_a.end
            }

            _ => true, // Conservative - consider mixed operations as conflicts
        }
    }

    /// Compose multiple operations into a single operation if possible
    pub fn compose_operations(&self, ops: &[Operation]) -> Result<Vec<Operation>> {
        if ops.is_empty() {
            return Ok(vec![]);
        }

        // For now, return as-is. In a full implementation, we would
        // merge adjacent inserts, combine consecutive deletes, etc.
        Ok(ops.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_operation() {
        let manager = OperationsManager::new();
        let content = "Hello world";
        let op = Operation::Insert {
            position: 5,
            content: " beautiful".to_string(),
        };

        let result = manager.apply_operation(content, &op).unwrap();
        assert_eq!(result, "Hello beautiful world");
    }

    #[test]
    fn test_delete_operation() {
        let manager = OperationsManager::new();
        let content = "Hello beautiful world";
        let op = Operation::Delete {
            position: 5,
            length: 10,
        };

        let result = manager.apply_operation(content, &op).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_transform_insert_insert() {
        let manager = OperationsManager::new();
        let op_a = Operation::Insert {
            position: 5,
            content: "A".to_string(),
        };
        let op_b = Operation::Insert {
            position: 3,
            content: "B".to_string(),
        };

        let transformed = manager.transform_operation(&op_a, &op_b).unwrap();
        match transformed {
            Operation::Insert { position, .. } => assert_eq!(position, 6),
            _ => panic!("Expected Insert operation"),
        }
    }
}
