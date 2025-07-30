//! Advanced TUI Mode - VS Code-like terminal experience
//!
//! This module provides a comprehensive multi-panel interface with:
//! - File explorer with Git status
//! - Code editor with syntax highlighting
//! - Consensus progress panel
//! - Integrated terminal
//! - VS Code-like keybindings and navigation

pub mod consensus;
pub mod dialogs;
pub mod editor;
pub mod explorer;
pub mod keybindings;
pub mod layout;
pub mod menu_bar;
pub mod panels;
pub mod problems;
pub mod terminal;

use self::dialogs::{DialogManager, DialogResult, DialogType};
use self::menu_bar::{MenuAction, MenuBar, MenuResult};
use crate::core::temporal::TemporalContext;
use crate::tui::accessibility::AccessibilityManager;
use crate::tui::themes::Theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

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
    /// Problems panel
    pub problems: problems::ProblemsPanel,
    /// Current theme
    pub theme: Theme,
    /// Accessibility manager
    pub accessibility: AccessibilityManager,
    /// Temporal context for time-aware features
    pub temporal: TemporalContext,
    /// Layout manager
    layout: layout::LayoutManager,
    /// Menu bar
    menu_bar: MenuBar,
    /// Dialog manager
    dialog_manager: DialogManager,
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
    Problems,
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
            problems: problems::ProblemsPanel::new(None),
            theme,
            accessibility,
            temporal,
            layout: layout::LayoutManager::new(),
            menu_bar: MenuBar::new(),
            dialog_manager: DialogManager::new(),
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

        // Render title bar with menu
        self.render_title_bar(frame, chunks.title_bar);

        // Render main content area
        self.render_main_content(frame, chunks.main_content);

        // Render status bar
        self.render_status_bar(frame, chunks.status_bar);

        // Render overlays (command palette, quick search, dialogs)
        self.render_overlays(frame, size);
    }

    /// Handle key events with VS Code-like keybindings
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        // Handle dialog input first
        if self.dialog_manager.is_active() {
            match self.dialog_manager.handle_key_event(key) {
                DialogResult::Closed => return Ok(false),
                DialogResult::FileSelected(path) => {
                    // Open file in editor
                    self.editor.open_file(path).await?;
                    self.active_panel = PanelType::Editor;
                    return Ok(false);
                }
                DialogResult::FolderSelected(path) => {
                    // Update explorer to show folder
                    self.explorer.set_root(path).await?;
                    return Ok(false);
                }
                DialogResult::ThemeSelected(theme_name) => {
                    // Apply theme
                    self.apply_theme(&theme_name);
                    return Ok(false);
                }
                DialogResult::Continue => return Ok(false),
            }
        }

        // Handle menu bar input
        if self.menu_bar.is_active() {
            match self.menu_bar.handle_key_event(key) {
                MenuResult::Action(action) => {
                    self.handle_menu_action(action).await?;
                    return Ok(self.should_quit);
                }
                MenuResult::Close => {
                    // Return focus to current panel
                    return Ok(false);
                }
                MenuResult::Continue => return Ok(false),
                MenuResult::None => {}
            }
        }

        // Handle global keybindings
        if self.handle_global_keybindings(key).await? {
            return Ok(self.should_quit);
        }

        // Handle overlay keybindings
        if self.command_palette_open || self.quick_search_open {
            return self.handle_overlay_keybindings(key).await;
        }

        // Handle panel-specific keybindings
        match self.active_panel {
            PanelType::Explorer => self.explorer.handle_key_event(key, &self.theme).await?,
            PanelType::Editor => self.editor.handle_key_event(key, &self.theme).await?,
            PanelType::Terminal => self.terminal.handle_key_event(key, &self.theme).await?,
            PanelType::ConsensusProgress => {
                // Consensus panel is read-only, just navigate
                false
            }
            PanelType::Problems => {
                // Handle problems panel navigation
                match key.code {
                    KeyCode::Up => {
                        self.problems.select_previous();
                        true
                    }
                    KeyCode::Down => {
                        self.problems.select_next();
                        true
                    }
                    KeyCode::Enter => {
                        // Navigate to selected problem
                        if let Some(location) = self.problems.navigate_to_selected() {
                            // Open file in editor and navigate to location
                            self.editor.open_file(location.file_path).await.unwrap_or(());
                            if let Some(line) = location.line {
                                self.editor.goto_line(line);
                            }
                            self.active_panel = PanelType::Editor;
                        }
                        true
                    }
                    KeyCode::F(5) => {
                        // Refresh problems
                        let workspace_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                        self.problems.update_problems(&workspace_path).await.unwrap_or(());
                        true
                    }
                    KeyCode::Tab => {
                        // Cycle through problem filters
                        // This will be handled by global keybindings
                        false
                    }
                    _ => false
                }
            }
        };

        Ok(self.should_quit)
    }

    /// Handle menu actions
    async fn handle_menu_action(&mut self, action: MenuAction) -> Result<()> {
        match action {
            // File menu actions
            MenuAction::OpenFile => {
                self.dialog_manager.show_dialog(DialogType::FilePicker)?;
            }
            MenuAction::OpenFolder => {
                // TODO: Implement folder picker
                self.dialog_manager.show_dialog(DialogType::FilePicker)?;
            }
            MenuAction::OpenRecent => {
                // TODO: Implement recent files
            }
            MenuAction::Save => {
                if let Some(path) = self.editor.current_file() {
                    self.editor.save_file().await?;
                }
            }
            MenuAction::SaveAs => {
                self.dialog_manager.show_dialog(DialogType::SaveAs)?;
            }
            MenuAction::CloseFolder => {
                self.explorer.clear_root().await?;
                self.editor.close_all_files();
            }
            MenuAction::Exit => {
                self.should_quit = true;
            }

            // View menu actions
            MenuAction::CommandPalette => {
                self.command_palette_open = true;
            }
            MenuAction::ThemeSelector => {
                self.dialog_manager.show_dialog(DialogType::ThemeSelector)?;
            }
            MenuAction::ToggleTerminal => {
                self.layout.toggle_terminal();
            }
            MenuAction::ToggleExplorer => {
                // TODO: Implement explorer toggle
            }
            MenuAction::ToggleConsensus => {
                self.layout.toggle_consensus();
            }

            // Help menu actions
            MenuAction::ShowWelcome => {
                self.dialog_manager.show_dialog(DialogType::Welcome)?;
            }
            MenuAction::ShowDocumentation => {
                // TODO: Open documentation in browser or help panel
            }
            MenuAction::ShowAbout => {
                self.dialog_manager.show_dialog(DialogType::About)?;
            }

            MenuAction::Separator => {}
        }
        Ok(())
    }

    /// Apply theme by name
    fn apply_theme(&mut self, theme_name: &str) {
        // TODO: Implement theme switching
        // For now, just update the theme name
        match theme_name {
            "Light" => {
                // self.theme = Theme::light();
            }
            "High Contrast" => {
                // self.theme = Theme::high_contrast();
            }
            "Solarized Dark" => {
                // self.theme = Theme::solarized_dark();
            }
            "Solarized Light" => {
                // self.theme = Theme::solarized_light();
            }
            _ => {
                // Default dark theme
            }
        }
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
            // F1-F5 - Switch panels
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
            (KeyModifiers::NONE, KeyCode::F(5)) => {
                self.active_panel = PanelType::Problems;
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
            // Alt key - Focus menu bar
            (KeyModifiers::ALT, KeyCode::Char(_)) => {
                self.menu_bar.focus();
                // Let menu bar handle the specific Alt+key combination
                self.menu_bar.handle_key_event(key);
                Ok(true)
            }
            _ => Ok(false),
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
            PanelType::ConsensusProgress => PanelType::Problems,
            PanelType::Problems => PanelType::Explorer,
        };
    }

    /// Render title bar with menu bar
    fn render_title_bar(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::style::{Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Paragraph};

        // Split title bar into menu area and title area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Menu bar
                Constraint::Length(1), // Title
            ])
            .split(area);

        // Render menu bar
        self.menu_bar.render(frame, chunks[0], &self.theme);

        // Render title
        let title = format!(
            "🐝 HiveTechs Consensus | {} | {}",
            self.temporal.current_time_formatted(),
            self.active_panel_name()
        );

        let title_widget = Paragraph::new(Line::from(vec![Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        )]))
        .style(self.theme.title_bar_style())
        .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(title_widget, chunks[1]);
    }

    /// Render main content area with panels
    fn render_main_content(&mut self, frame: &mut Frame, area: Rect) {
        let layout = self.layout.get_main_layout(area);

        // Render explorer panel
        self.explorer.render(
            frame,
            layout.explorer,
            &self.theme,
            self.active_panel == PanelType::Explorer,
        );

        // Render editor panel
        self.editor.render(
            frame,
            layout.editor,
            &self.theme,
            self.active_panel == PanelType::Editor,
        );

        // Render terminal panel (if visible)
        if layout.terminal.height > 0 {
            self.terminal.render(
                frame,
                layout.terminal,
                &self.theme,
                self.active_panel == PanelType::Terminal,
            );
        }

        // Render consensus progress panel
        if layout.consensus.width > 0 {
            self.render_consensus_panel(frame, layout.consensus);
        }

        // Render problems panel (if visible)
        if layout.problems.height > 0 {
            self.problems.draw(frame, layout.problems);
        }
    }

    /// Render status bar with current status and shortcuts
    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        use ratatui::style::{Color, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Paragraph};

        let status_text = format!(
            "F1:Explorer F2:Editor F3:Terminal F4:Consensus F5:Problems | Ctrl+P:Search Ctrl+Shift+P:Commands | {}",
            self.get_current_status()
        );

        let status_widget = Paragraph::new(Line::from(vec![Span::styled(
            status_text,
            Style::default().fg(Color::Gray),
        )]))
        .block(Block::default().borders(Borders::TOP))
        .style(self.theme.status_bar_style());

        frame.render_widget(status_widget, area);
    }

    /// Render overlays (command palette, quick search, dialogs)
    fn render_overlays(&mut self, frame: &mut Frame, area: Rect) {
        // Render dialogs first (they should be on top)
        self.dialog_manager.render(frame, area, &self.theme);

        // Then render command palette and quick search
        if self.command_palette_open {
            self.render_command_palette(frame, area);
        }

        if self.quick_search_open {
            self.render_quick_search(frame, area);
        }
    }

    /// Render consensus progress panel
    fn render_consensus_panel(&self, frame: &mut Frame, area: Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Paragraph};

        let consensus_widget = Paragraph::new(vec![
            Line::from(vec![Span::raw("🧠 Consensus Progress")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("Generator: Ready")]),
            Line::from(vec![Span::raw("Refiner: Idle")]),
            Line::from(vec![Span::raw("Validator: Idle")]),
            Line::from(vec![Span::raw("Curator: Idle")]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Consensus")
                .border_style(if self.active_panel == PanelType::ConsensusProgress {
                    self.theme.active_border_style()
                } else {
                    self.theme.inactive_border_style()
                }),
        )
        .style(self.theme.panel_style());

        frame.render_widget(consensus_widget, area);
    }

    /// Render command palette overlay
    fn render_command_palette(&self, frame: &mut Frame, area: Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Clear, Paragraph};

        // Center the command palette
        let popup_area = layout::centered_rect(60, 20, area);

        frame.render_widget(Clear, popup_area);

        let command_palette = Paragraph::new(vec![
            Line::from(vec![Span::raw("Command Palette")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("> Type command...")]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Commands"))
        .style(self.theme.popup_style());

        frame.render_widget(command_palette, popup_area);
    }

    /// Render quick search overlay
    fn render_quick_search(&self, frame: &mut Frame, area: Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Clear, Paragraph};

        // Center the quick search
        let popup_area = layout::centered_rect(60, 20, area);

        frame.render_widget(Clear, popup_area);

        let quick_search = Paragraph::new(vec![
            Line::from(vec![Span::raw("Quick File Search")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("> Search files...")]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Go to File"))
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
            PanelType::Problems => "Problems",
        }
    }

    /// Get current status for status bar
    fn get_current_status(&self) -> String {
        let problems_summary = self.problems.get_summary();
        let problems_text = if problems_summary.total > 0 {
            format!("Problems: {}🔴 {}🟡", problems_summary.errors, problems_summary.warnings)
        } else {
            "No Problems".to_string()
        };
        
        format!(
            "{} | Theme: {} | Mode: {} | Time: {}",
            problems_text,
            self.theme.name(),
            if self.accessibility.screen_reader_mode() {
                "Accessible"
            } else {
                "Standard"
            },
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
