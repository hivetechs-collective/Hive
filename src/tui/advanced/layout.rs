//! Layout manager for advanced TUI
//!
//! Provides VS Code-like panel layout with responsive design

use crate::tui::themes::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Layout manager for advanced TUI panels
pub struct LayoutManager {
    /// Show terminal panel
    show_terminal: bool,
    /// Show consensus panel
    show_consensus: bool,
    /// Terminal panel height percentage
    terminal_height_percent: u16,
    /// Consensus panel width percentage
    consensus_width_percent: u16,
}

/// Main layout chunks for advanced TUI
pub struct MainLayoutChunks {
    pub title_bar: Rect,
    pub main_content: Rect,
    pub status_bar: Rect,
}

/// Content area layout chunks
pub struct ContentLayoutChunks {
    pub explorer: Rect,
    pub editor: Rect,
    pub terminal: Rect,
    pub consensus: Rect,
}

impl LayoutManager {
    /// Create new layout manager
    pub fn new() -> Self {
        Self {
            show_terminal: true,
            show_consensus: true,
            terminal_height_percent: 30,
            consensus_width_percent: 25,
        }
    }

    /// Calculate main layout (title bar, content, status bar)
    pub fn calculate_layout(&self, area: Rect, _theme: &Theme) -> MainLayoutChunks {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Title bar (menu + title)
                Constraint::Min(0),    // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        MainLayoutChunks {
            title_bar: chunks[0],
            main_content: chunks[1],
            status_bar: chunks[2],
        }
    }

    /// Get main content layout with panels
    pub fn get_main_layout(&self, area: Rect) -> ContentLayoutChunks {
        // Split horizontally first (explorer + editor/terminal | consensus)
        let horizontal_constraints = if self.show_consensus {
            vec![
                Constraint::Percentage(100 - self.consensus_width_percent),
                Constraint::Percentage(self.consensus_width_percent),
            ]
        } else {
            vec![Constraint::Percentage(100)]
        };

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraints)
            .split(area);

        let main_area = horizontal_chunks[0];
        let consensus_area = if self.show_consensus {
            horizontal_chunks[1]
        } else {
            Rect::default()
        };

        // Split main area horizontally (explorer | editor/terminal)
        let main_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Explorer
                Constraint::Percentage(75), // Editor/Terminal
            ])
            .split(main_area);

        let explorer_area = main_horizontal[0];
        let editor_terminal_area = main_horizontal[1];

        // Split editor/terminal area vertically if terminal is shown
        let (editor_area, terminal_area) = if self.show_terminal {
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(100 - self.terminal_height_percent),
                    Constraint::Percentage(self.terminal_height_percent),
                ])
                .split(editor_terminal_area);
            (vertical_chunks[0], vertical_chunks[1])
        } else {
            (editor_terminal_area, Rect::default())
        };

        ContentLayoutChunks {
            explorer: explorer_area,
            editor: editor_area,
            terminal: terminal_area,
            consensus: consensus_area,
        }
    }

    /// Toggle terminal panel visibility
    pub fn toggle_terminal(&mut self) {
        self.show_terminal = !self.show_terminal;
    }

    /// Toggle consensus panel visibility
    pub fn toggle_consensus(&mut self) {
        self.show_consensus = !self.show_consensus;
    }

    /// Set terminal height percentage
    pub fn set_terminal_height(&mut self, percentage: u16) {
        self.terminal_height_percent = percentage.clamp(10, 80);
    }

    /// Set consensus width percentage
    pub fn set_consensus_width(&mut self, percentage: u16) {
        self.consensus_width_percent = percentage.clamp(15, 50);
    }

    /// Check if terminal is visible
    pub fn terminal_visible(&self) -> bool {
        self.show_terminal
    }

    /// Check if consensus panel is visible
    pub fn consensus_visible(&self) -> bool {
        self.show_consensus
    }
}

/// Create a centered rectangle for popups
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Calculate responsive layout based on terminal size
pub fn responsive_layout(area: Rect) -> (bool, bool) {
    let width = area.width;
    let height = area.height;

    // Determine what to show based on terminal size
    let show_consensus = width >= 120; // Need at least 120 columns for consensus panel
    let show_terminal = height >= 30; // Need at least 30 rows for terminal panel

    (show_consensus, show_terminal)
}

/// Get minimum required terminal size for advanced TUI
pub fn minimum_terminal_size() -> (u16, u16) {
    (80, 24) // Minimum 80x24 for basic functionality
}

/// Check if terminal size supports advanced TUI
pub fn supports_advanced_tui(area: Rect) -> bool {
    let (min_width, min_height) = minimum_terminal_size();
    area.width >= min_width && area.height >= min_height
}
