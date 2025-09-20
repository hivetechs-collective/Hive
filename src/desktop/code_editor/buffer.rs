//! Text Buffer Implementation using Rope data structure
//!
//! Efficient text manipulation for large files with O(log n) operations

use super::cursor::Position;
use ropey::{Rope, RopeSlice};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct TextBuffer {
    /// The underlying rope data structure
    rope: Rope,

    /// Whether the buffer has unsaved changes
    dirty: bool,

    /// File path associated with this buffer
    file_path: Option<String>,

    /// Language ID for syntax highlighting
    language_id: String,
}

#[derive(Debug, Clone)]
pub enum TextEdit {
    Insert {
        position: Position,
        text: String,
    },
    Delete {
        range: Range<Position>,
    },
    Replace {
        range: Range<Position>,
        text: String,
    },
}

impl TextBuffer {
    /// Create a new empty text buffer
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            dirty: false,
            file_path: None,
            language_id: String::from("plaintext"),
        }
    }

    /// Create a text buffer from initial content
    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            dirty: false,
            file_path: None,
            language_id: String::from("plaintext"),
        }
    }

    /// Get the total length in characters
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Get the total number of lines
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get a line by index (0-based)
    pub fn line(&self, line_idx: usize) -> Option<RopeSlice> {
        if line_idx < self.len_lines() {
            Some(self.rope.line(line_idx))
        } else {
            None
        }
    }

    /// Get text in a character range
    pub fn slice(&self, range: Range<usize>) -> RopeSlice {
        self.rope.slice(range)
    }

    /// Insert text at a character position
    pub fn insert(&mut self, char_idx: usize, text: &str) {
        if char_idx <= self.len_chars() {
            self.rope.insert(char_idx, text);
            self.dirty = true;
        }
    }

    /// Remove text in a character range
    pub fn remove(&mut self, range: Range<usize>) {
        if range.start <= self.len_chars() && range.end <= self.len_chars() {
            self.rope.remove(range);
            self.dirty = true;
        }
    }

    /// Replace text in a range
    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        self.remove(range.clone());
        self.insert(range.start, text);
    }

    /// Apply a text edit
    pub fn apply_edit(&mut self, edit: TextEdit) {
        match edit {
            TextEdit::Insert { position, text } => {
                if let Some(char_idx) = self.line_col_to_char(position.line, position.column) {
                    self.insert(char_idx, &text);
                }
            }
            TextEdit::Delete { range } => {
                if let (Some(start), Some(end)) = (
                    self.line_col_to_char(range.start.line, range.start.column),
                    self.line_col_to_char(range.end.line, range.end.column),
                ) {
                    self.remove(start..end);
                }
            }
            TextEdit::Replace { range, text } => {
                if let (Some(start), Some(end)) = (
                    self.line_col_to_char(range.start.line, range.start.column),
                    self.line_col_to_char(range.end.line, range.end.column),
                ) {
                    self.replace(start..end, &text);
                }
            }
        }
    }

    /// Apply multiple edits (sorted by range start, descending)
    pub fn apply_edits(&mut self, mut edits: Vec<TextEdit>) {
        // Sort edits by start position in descending order to avoid offset issues
        edits.sort_by(|a, b| {
            let a_pos = match a {
                TextEdit::Insert { position, .. } => position,
                TextEdit::Delete { range, .. } => &range.start,
                TextEdit::Replace { range, .. } => &range.start,
            };
            let b_pos = match b {
                TextEdit::Insert { position, .. } => position,
                TextEdit::Delete { range, .. } => &range.start,
                TextEdit::Replace { range, .. } => &range.start,
            };
            b_pos.cmp(a_pos)
        });

        for edit in edits {
            self.apply_edit(edit);
        }
    }

    /// Convert line/column position to character index
    pub fn line_col_to_char(&self, line: usize, col: usize) -> Option<usize> {
        if line >= self.len_lines() {
            return None;
        }

        let line_start = self.rope.line_to_char(line);
        let line_slice = self.rope.line(line);

        if col > line_slice.len_chars() {
            None
        } else {
            Some(line_start + col)
        }
    }

    /// Convert character index to line/column position
    pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        if char_idx > self.len_chars() {
            let last_line = self.len_lines().saturating_sub(1);
            let last_col = self.line(last_line).map(|l| l.len_chars()).unwrap_or(0);
            return (last_line, last_col);
        }

        let line = self.rope.char_to_line(char_idx);
        let line_start = self.rope.line_to_char(line);
        let col = char_idx - line_start;

        (line, col)
    }

    /// Get the entire buffer as a string
    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }

    /// Check if buffer has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark buffer as saved
    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }

    /// Set the file path
    pub fn set_file_path(&mut self, path: String) {
        self.file_path = Some(path);
    }

    /// Get the file path
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    /// Set the language ID
    pub fn set_language(&mut self, language_id: String) {
        self.language_id = language_id;
    }

    /// Get the language ID
    pub fn language(&self) -> &str {
        &self.language_id
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut buffer = TextBuffer::from_text("Hello, world!");

        assert_eq!(buffer.len_chars(), 13);
        assert_eq!(buffer.len_lines(), 1);

        // Insert text
        buffer.insert(7, "Rust ");
        assert_eq!(buffer.to_string(), "Hello, Rust world!");

        // Remove text
        buffer.remove(7..12);
        assert_eq!(buffer.to_string(), "Hello, world!");

        // Replace text
        buffer.replace(7..12, "HiveTechs");
        assert_eq!(buffer.to_string(), "Hello, HiveTechs!");
    }

    #[test]
    fn test_line_col_conversion() {
        let buffer = TextBuffer::from_text("Line 1\nLine 2\nLine 3");

        // Test line/col to char
        assert_eq!(buffer.line_col_to_char(0, 0), Some(0));
        assert_eq!(buffer.line_col_to_char(1, 0), Some(7));
        assert_eq!(buffer.line_col_to_char(1, 5), Some(12));

        // Test char to line/col
        assert_eq!(buffer.char_to_line_col(0), (0, 0));
        assert_eq!(buffer.char_to_line_col(7), (1, 0));
        assert_eq!(buffer.char_to_line_col(12), (1, 5));
    }
}
