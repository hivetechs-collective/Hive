//! Advanced TUI Mode - VS Code-like terminal experience
//!
//! This module provides a comprehensive multi-panel interface with:
//! - File explorer with Git status
//! - Code editor with syntax highlighting
//! - Consensus progress panel
//! - Integrated terminal
//! - VS Code-like keybindings and navigation

pub mod panels;
pub mod explorer;
pub mod editor;
pub mod terminal;
pub mod layout;
pub mod keybindings;
pub mod consensus;

use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use crate::tui::themes::Theme;
use crate::tui::accessibility::AccessibilityManager;
use crate::core::temporal::TemporalContext;

/// Advanced TUI application state
pub struct AdvancedTuiApp {
    /// Current active panel
    active_panel: PanelType,
    /// File explorer panel
    pub explorer: explorer::ExplorerPanel,
    /// Code editor panel  
    pub editor: editor::EditorPanel,
    /// Terminal panel
    pub terminal: terminal::TerminalPanel,
    /// Current theme
    pub theme: Theme,
    /// Accessibility manager
    pub accessibility: AccessibilityManager,
    /// Temporal context for time-aware features
    pub temporal: TemporalContext,
    /// Layout manager
    layout: layout::LayoutManager,
    /// Should quit application
    should_quit: bool,
    /// Command palette state
    command_palette_open: bool,
    /// Quick search state
    quick_search_open: bool,
}

/// Available panels in advanced TUI
#[derive(Debug, Clone, PartialEq)]
pub enum PanelType {
    Explorer,
    Editor,
    Terminal,
    ConsensusProgress,
}

impl AdvancedTuiApp {
    /// Create new advanced TUI application
    pub async fn new() -> Result<Self> {
        let theme = Theme::default();
        let accessibility = AccessibilityManager::new();
        let temporal = TemporalContext::new();
        
        Ok(Self {
            active_panel: PanelType::Explorer,
            explorer: explorer::ExplorerPanel::new().await?,
            editor: editor::EditorPanel::new(),
            terminal: terminal::TerminalPanel::new()?,
            theme,
            accessibility,
            temporal,
            layout: layout::LayoutManager::new(),
            should_quit: false,
            command_palette_open: false,
            quick_search_open: false,
        })
    }

    /// Render the advanced TUI interface
    pub fn render(&mut self, frame: &mut Frame) {
        let size = frame.size();
        
        // Apply accessibility adjustments
        self.accessibility.adjust_for_screen_reader(&mut self.theme);
        
        // Create main layout
        let chunks = self.layout.calculate_layout(size, &self.theme);
        
        // Render title bar
        self.render_title_bar(frame, chunks.title_bar);
        
        // Render main content area
        self.render_main_content(frame, chunks.main_content);
        
        // Render status bar
        self.render_status_bar(frame, chunks.status_bar);
        
        // Render overlays (command palette, quick search)
        self.render_overlays(frame, size);
    }

    /// Handle key events with VS Code-like keybindings
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        // Handle global keybindings first
        if self.handle_global_keybindings(key).await? {
            return Ok(self.should_quit);
        }
        
        // Handle overlay keybindings
        if self.command_palette_open || self.quick_search_open {
            return self.handle_overlay_keybindings(key).await;
        }
        
        // Handle panel-specific keybindings
        match self.active_panel {
            PanelType::Explorer => {
                self.explorer.handle_key_event(key, &self.theme).await?
            }
            PanelType::Editor => {
                self.editor.handle_key_event(key, &self.theme).await?
            }
            PanelType::Terminal => {
                self.terminal.handle_key_event(key, &self.theme).await?
            }
            PanelType::ConsensusProgress => {
                // Consensus panel is read-only, just navigate
                false
            }
        };
        
        Ok(self.should_quit)
    }

    /// Handle global VS Code-like keybindings
    async fn handle_global_keybindings(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.modifiers, key.code) {
            // Ctrl+Shift+P - Command Palette
            (KeyModifiers::CONTROL | KeyModifiers::SHIFT, KeyCode::Char('P')) => {
                self.command_palette_open = !self.command_palette_open;
                Ok(true)
            }
            // Ctrl+P - Quick File Search
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                self.quick_search_open = !self.quick_search_open;
                Ok(true)
            }
            // Ctrl+` - Toggle Terminal
            (KeyModifiers::CONTROL, KeyCode::Char('`')) => {
                self.toggle_terminal();
                Ok(true)
            }
            // F1-F4 - Switch panels
            (KeyModifiers::NONE, KeyCode::F(1)) => {
                self.active_panel = PanelType::Explorer;
                Ok(true)
            }
            (KeyModifiers::NONE, KeyCode::F(2)) => {
                self.active_panel = PanelType::Editor;
                Ok(true)
            }
            (KeyModifiers::NONE, KeyCode::F(3)) => {
                self.active_panel = PanelType::Terminal;
                Ok(true)
            }
            (KeyModifiers::NONE, KeyCode::F(4)) => {
                self.active_panel = PanelType::ConsensusProgress;
                Ok(true)
            }
            // Ctrl+Q - Quit
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                self.should_quit = true;
                Ok(true)
            }
            // Tab - Cycle through panels
            (KeyModifiers::NONE, KeyCode::Tab) => {
                self.cycle_active_panel();
                Ok(true)
            }
            _ => Ok(false)
        }
    }

    /// Handle overlay keybindings (command palette, quick search)
    async fn handle_overlay_keybindings(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.command_palette_open = false;
                self.quick_search_open = false;
                Ok(false)
            }
            _ => {
                // Handle specific overlay input
                if self.command_palette_open {
                    // TODO: Implement command palette input handling
                }
                if self.quick_search_open {
                    // TODO: Implement quick search input handling
                }
                Ok(false)
            }
        }
    }

    /// Toggle terminal panel visibility
    fn toggle_terminal(&mut self) {
        if self.active_panel == PanelType::Terminal {
            self.active_panel = PanelType::Explorer;
        } else {
            self.active_panel = PanelType::Terminal;
        }
    }

    /// Cycle through available panels
    fn cycle_active_panel(&mut self) {
        self.active_panel = match self.active_panel {
            PanelType::Explorer => PanelType::Editor,
            PanelType::Editor => PanelType::Terminal,
            PanelType::Terminal => PanelType::ConsensusProgress,
            PanelType::ConsensusProgress => PanelType::Explorer,
        };
    }

    /// Render title bar with branding and current time
    fn render_title_bar(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::style::{Style, Modifier};
        use ratatui::text::{Span, Line};
        
        let title = format!(
            "🐝 HiveTechs Consensus | {} | {}",
            self.temporal.current_time_formatted(),
            self.active_panel_name()
        );
        
        let title_widget = Paragraph::new(Line::from(vec![
            Span::styled(title, Style::default().add_modifier(Modifier::BOLD))
        ]))
        .block(Block::default().borders(Borders::BOTTOM))
        .style(self.theme.title_bar_style());
        
        frame.render_widget(title_widget, area);
    }

    /// Render main content area with panels
    fn render_main_content(&mut self, frame: &mut Frame, area: Rect) {
        let layout = self.layout.get_main_layout(area);
        
        // Render explorer panel
        self.explorer.render(frame, layout.explorer, &self.theme, 
                           self.active_panel == PanelType::Explorer);
        
        // Render editor panel
        self.editor.render(frame, layout.editor, &self.theme,
                         self.active_panel == PanelType::Editor);
        
        // Render terminal panel (if visible)
        if layout.terminal.height > 0 {
            self.terminal.render(frame, layout.terminal, &self.theme,
                               self.active_panel == PanelType::Terminal);
        }
        
        // Render consensus progress panel
        if layout.consensus.width > 0 {
            self.render_consensus_panel(frame, layout.consensus);
        }
    }

    /// Render status bar with current status and shortcuts
    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::text::{Span, Line};
        use ratatui::style::{Style, Color};
        
        let status_text = format!(
            "F1:Explorer F2:Editor F3:Terminal F4:Consensus | Ctrl+P:Search Ctrl+Shift+P:Commands | {}",
            self.get_current_status()
        );
        
        let status_widget = Paragraph::new(Line::from(vec![
            Span::styled(status_text, Style::default().fg(Color::Gray))
        ]))
        .block(Block::default().borders(Borders::TOP))
        .style(self.theme.status_bar_style());
        
        frame.render_widget(status_widget, area);
    }

    /// Render overlays (command palette, quick search)
    fn render_overlays(&self, frame: &mut Frame, area: Rect) {
        if self.command_palette_open {
            self.render_command_palette(frame, area);
        }
        
        if self.quick_search_open {
            self.render_quick_search(frame, area);
        }
    }

    /// Render consensus progress panel
    fn render_consensus_panel(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::text::{Span, Line};
        
        let consensus_widget = Paragraph::new(vec![
            Line::from(vec![Span::raw("🧠 Consensus Progress")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("Generator: Ready")]),
            Line::from(vec![Span::raw("Refiner: Idle")]),
            Line::from(vec![Span::raw("Validator: Idle")]),
            Line::from(vec![Span::raw("Curator: Idle")]),
        ])
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Consensus")
            .border_style(if self.active_panel == PanelType::ConsensusProgress {
                self.theme.active_border_style()
            } else {
                self.theme.inactive_border_style()
            }))
        .style(self.theme.panel_style());
        
        frame.render_widget(consensus_widget, area);
    }

    /// Render command palette overlay
    fn render_command_palette(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::{Block, Borders, Clear, Paragraph};
        use ratatui::text::{Span, Line};
        
        // Center the command palette
        let popup_area = layout::centered_rect(60, 20, area);
        
        frame.render_widget(Clear, popup_area);
        
        let command_palette = Paragraph::new(vec![
            Line::from(vec![Span::raw("Command Palette")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("> Type command...")]),
        ])
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Commands"))
        .style(self.theme.popup_style());
        
        frame.render_widget(command_palette, popup_area);
    }

    /// Render quick search overlay  
    fn render_quick_search(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::{Block, Borders, Clear, Paragraph};
        use ratatui::text::{Span, Line};
        
        // Center the quick search
        let popup_area = layout::centered_rect(60, 20, area);
        
        frame.render_widget(Clear, popup_area);
        
        let quick_search = Paragraph::new(vec![
            Line::from(vec![Span::raw("Quick File Search")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("> Search files...")]),
        ])
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Go to File"))
        .style(self.theme.popup_style());
        
        frame.render_widget(quick_search, popup_area);
    }

    /// Get name of active panel
    fn active_panel_name(&self) -> &'static str {
        match self.active_panel {
            PanelType::Explorer => "Explorer",
            PanelType::Editor => "Editor",
            PanelType::Terminal => "Terminal", 
            PanelType::ConsensusProgress => "Consensus",
        }
    }

    /// Get current status for status bar
    fn get_current_status(&self) -> String {
        format!(
            "Theme: {} | Mode: {} | Time: {}",
            self.theme.name(),
            if self.accessibility.screen_reader_mode() { "Accessible" } else { "Standard" },
            self.temporal.current_time_formatted()
        )
    }
    
    /// Check if should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Handle async events
    pub async fn handle_async_events(&mut self) -> Result<()> {
        // TODO: Implement async event handling
        // For now, just return success
        Ok(())
    }
}