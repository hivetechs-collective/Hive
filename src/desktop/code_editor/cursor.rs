//! Cursor and Selection Management
//! 
//! Handles cursor positioning, selections, and multi-cursor support

use std::cmp::{max, min};
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
    
    pub fn min(self, other: Self) -> Self {
        if self.line < other.line || (self.line == other.line && self.column < other.column) {
            self
        } else {
            other
        }
    }
    
    pub fn max(self, other: Self) -> Self {
        if self.line > other.line || (self.line == other.line && self.column > other.column) {
            self
        } else {
            other
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    /// The anchor point (where selection started)
    pub anchor: Position,
    
    /// The active point (where cursor currently is)
    pub active: Position,
}

impl Selection {
    pub fn new(anchor: Position, active: Position) -> Self {
        Self { anchor, active }
    }
    
    pub fn single(pos: Position) -> Self {
        Self {
            anchor: pos,
            active: pos,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.anchor == self.active
    }
    
    pub fn start(&self) -> Position {
        self.anchor.min(self.active)
    }
    
    pub fn end(&self) -> Position {
        self.anchor.max(self.active)
    }
    
    pub fn as_char_range(&self, line_to_char: impl Fn(usize, usize) -> usize) -> Range<usize> {
        let start = line_to_char(self.start().line, self.start().column);
        let end = line_to_char(self.end().line, self.end().column);
        start..end
    }
}

#[derive(Debug, Clone)]
pub struct Cursor {
    /// Primary selection (always exists)
    pub primary: Selection,
    
    /// Additional selections for multi-cursor
    pub secondary: Vec<Selection>,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            primary: Selection::single(Position::new(0, 0)),
            secondary: Vec::new(),
        }
    }
    
    pub fn single(line: usize, column: usize) -> Self {
        Self {
            primary: Selection::single(Position::new(line, column)),
            secondary: Vec::new(),
        }
    }
    
    /// Get all selections (primary + secondary)
    pub fn all_selections(&self) -> Vec<&Selection> {
        let mut selections = vec![&self.primary];
        selections.extend(self.secondary.iter());
        selections
    }
    
    /// Add a new cursor at position
    pub fn add_cursor(&mut self, pos: Position) {
        self.secondary.push(Selection::single(pos));
        self.merge_overlapping();
    }
    
    /// Add a new selection
    pub fn add_selection(&mut self, selection: Selection) {
        self.secondary.push(selection);
        self.merge_overlapping();
    }
    
    /// Move all cursors by delta
    pub fn move_by(&mut self, line_delta: isize, col_delta: isize, max_lines: usize, max_cols: &[usize]) {
        self.primary = move_selection(self.primary.clone(), line_delta, col_delta, max_lines, max_cols);
        
        for selection in &mut self.secondary {
            *selection = move_selection(selection.clone(), line_delta, col_delta, max_lines, max_cols);
        }
    }
    
    /// Extend selection by delta
    pub fn extend_by(&mut self, line_delta: isize, col_delta: isize, max_lines: usize, max_cols: &[usize]) {
        self.primary.active = move_position(self.primary.active, line_delta, col_delta, max_lines, max_cols);
        
        for selection in &mut self.secondary {
            selection.active = move_position(selection.active, line_delta, col_delta, max_lines, max_cols);
        }
    }
    
    /// Clear all secondary cursors
    pub fn clear_secondary(&mut self) {
        self.secondary.clear();
    }
    
    /// Merge overlapping selections
    fn merge_overlapping(&mut self) {
        // TODO: Implement merging logic for overlapping selections
        // For now, just remove duplicates
        self.secondary.sort_by_key(|s| (s.start().line, s.start().column));
        self.secondary.dedup_by(|a, b| a.start() == b.start() && a.end() == b.end());
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

/// Move a position by delta, respecting bounds
fn move_position(
    pos: Position,
    line_delta: isize,
    col_delta: isize,
    max_lines: usize,
    max_cols: &[usize],
) -> Position {
    let new_line = (pos.line as isize + line_delta).clamp(0, max_lines.saturating_sub(1) as isize) as usize;
    
    let max_col = max_cols.get(new_line).copied().unwrap_or(0);
    let new_col = if new_line != pos.line {
        // When moving to a different line, clamp to that line's length
        if line_delta > 0 {
            0 // Moving down, start at beginning of line
        } else {
            max_col // Moving up, go to end of line
        }
    } else {
        // Same line, apply column delta
        (pos.column as isize + col_delta).clamp(0, max_col as isize) as usize
    };
    
    Position::new(new_line, new_col)
}

/// Move a selection by delta
fn move_selection(
    mut selection: Selection,
    line_delta: isize,
    col_delta: isize,
    max_lines: usize,
    max_cols: &[usize],
) -> Selection {
    selection.anchor = move_position(selection.anchor, line_delta, col_delta, max_lines, max_cols);
    selection.active = move_position(selection.active, line_delta, col_delta, max_lines, max_cols);
    selection
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_position_ordering() {
        let pos1 = Position::new(1, 5);
        let pos2 = Position::new(1, 10);
        let pos3 = Position::new(2, 3);
        
        assert_eq!(pos1.min(pos2), pos1);
        assert_eq!(pos1.max(pos2), pos2);
        assert_eq!(pos2.min(pos3), pos2);
    }
    
    #[test]
    fn test_selection() {
        let sel = Selection::new(Position::new(1, 5), Position::new(2, 3));
        
        assert!(!sel.is_empty());
        assert_eq!(sel.start(), Position::new(1, 5));
        assert_eq!(sel.end(), Position::new(2, 3));
    }
    
    #[test]
    fn test_cursor_movement() {
        let mut cursor = Cursor::single(1, 5);
        let max_lines = 5;
        let max_cols = vec![10, 15, 20, 25, 30];
        
        // Move right
        cursor.move_by(0, 3, max_lines, &max_cols);
        assert_eq!(cursor.primary.active, Position::new(1, 8));
        
        // Move down
        cursor.move_by(1, 0, max_lines, &max_cols);
        assert_eq!(cursor.primary.active, Position::new(2, 0));
    }
}