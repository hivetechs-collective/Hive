//! Menu bar component for IDE-like interface
//!
//! Provides a VS Code-style menu bar with proper actions

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;

/// Menu bar state and logic
pub struct MenuBar {
    /// Currently open menu
    active_menu: Option<MenuType>,
    /// Selected item in active menu
    selected_item: usize,
    /// Whether menu bar is focused
    is_focused: bool,
    /// Menu definitions
    menus: HashMap<MenuType, Vec<MenuItem>>,
}

/// Available menu types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuType {
    File,
    View,
    Help,
}

/// Menu item definition
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
    pub shortcut: Option<String>,
    pub enabled: bool,
}

/// Menu actions (proper IDE behaviors, no chat messages!)
#[derive(Debug, Clone)]
pub enum MenuAction {
    // File menu actions
    OpenFile,
    OpenFolder,
    OpenRecent,
    Save,
    SaveAs,
    CloseFolder,
    Exit,

    // View menu actions
    CommandPalette,
    ThemeSelector,
    ToggleTerminal,
    ToggleExplorer,
    ToggleConsensus,

    // Help menu actions
    ShowWelcome,
    ShowDocumentation,
    ShowAbout,

    // Separator
    Separator,
}

/// Result of menu interaction
#[derive(Debug)]
pub enum MenuResult {
    /// No action needed
    None,
    /// Execute a menu action
    Action(MenuAction),
    /// Close menu and return focus
    Close,
    /// Keep menu open
    Continue,
}

impl MenuBar {
    /// Create new menu bar
    pub fn new() -> Self {
        let mut menu_bar = Self {
            active_menu: None,
            selected_item: 0,
            is_focused: false,
            menus: HashMap::new(),
        };

        menu_bar.setup_menus();
        menu_bar
    }

    /// Setup menu definitions
    fn setup_menus(&mut self) {
        // File menu
        self.menus.insert(
            MenuType::File,
            vec![
                MenuItem {
                    label: "Open File...".to_string(),
                    action: MenuAction::OpenFile,
                    shortcut: Some("Ctrl+O".to_string()),
                    enabled: true,
                },
                MenuItem {
                    label: "Open Folder...".to_string(),
                    action: MenuAction::OpenFolder,
                    shortcut: Some("Ctrl+K Ctrl+O".to_string()),
                    enabled: true,
                },
                MenuItem {
                    label: "Open Recent".to_string(),
                    action: MenuAction::OpenRecent,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "".to_string(),
                    action: MenuAction::Separator,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "Save".to_string(),
                    action: MenuAction::Save,
                    shortcut: Some("Ctrl+S".to_string()),
                    enabled: true,
                },
                MenuItem {
                    label: "Save As...".to_string(),
                    action: MenuAction::SaveAs,
                    shortcut: Some("Ctrl+Shift+S".to_string()),
                    enabled: true,
                },
                MenuItem {
                    label: "".to_string(),
                    action: MenuAction::Separator,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "Close Folder".to_string(),
                    action: MenuAction::CloseFolder,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "".to_string(),
                    action: MenuAction::Separator,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "Exit".to_string(),
                    action: MenuAction::Exit,
                    shortcut: Some("Ctrl+Q".to_string()),
                    enabled: true,
                },
            ],
        );

        // View menu
        self.menus.insert(
            MenuType::View,
            vec![
                MenuItem {
                    label: "Command Palette...".to_string(),
                    action: MenuAction::CommandPalette,
                    shortcut: Some("Ctrl+Shift+P".to_string()),
                    enabled: true,
                },
                MenuItem {
                    label: "".to_string(),
                    action: MenuAction::Separator,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "Terminal".to_string(),
                    action: MenuAction::ToggleTerminal,
                    shortcut: Some("Ctrl+`".to_string()),
                    enabled: true,
                },
                MenuItem {
                    label: "Explorer".to_string(),
                    action: MenuAction::ToggleExplorer,
                    shortcut: Some("Ctrl+Shift+E".to_string()),
                    enabled: true,
                },
                MenuItem {
                    label: "Consensus Panel".to_string(),
                    action: MenuAction::ToggleConsensus,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "".to_string(),
                    action: MenuAction::Separator,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "Theme".to_string(),
                    action: MenuAction::ThemeSelector,
                    shortcut: None,
                    enabled: true,
                },
            ],
        );

        // Help menu
        self.menus.insert(
            MenuType::Help,
            vec![
                MenuItem {
                    label: "Welcome".to_string(),
                    action: MenuAction::ShowWelcome,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "Documentation".to_string(),
                    action: MenuAction::ShowDocumentation,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "".to_string(),
                    action: MenuAction::Separator,
                    shortcut: None,
                    enabled: true,
                },
                MenuItem {
                    label: "About HiveTechs Consensus".to_string(),
                    action: MenuAction::ShowAbout,
                    shortcut: None,
                    enabled: true,
                },
            ],
        );
    }

    /// Render menu bar
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &crate::tui::themes::Theme) {
        // Create horizontal layout for menu items
        let menu_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(6), // File
                Constraint::Length(6), // View
                Constraint::Length(6), // Help
                Constraint::Min(0),    // Rest of space
            ])
            .split(area);

        // Render each menu title
        self.render_menu_title(frame, menu_chunks[0], MenuType::File, "File", theme);
        self.render_menu_title(frame, menu_chunks[1], MenuType::View, "View", theme);
        self.render_menu_title(frame, menu_chunks[2], MenuType::Help, "Help", theme);

        // Render dropdown if a menu is active
        if let Some(active_menu) = self.active_menu {
            self.render_dropdown(frame, area, active_menu, theme);
        }
    }

    /// Render individual menu title
    fn render_menu_title(
        &self,
        frame: &mut Frame,
        area: Rect,
        menu_type: MenuType,
        label: &str,
        theme: &crate::tui::themes::Theme,
    ) {
        let is_active = self.active_menu == Some(menu_type);
        let style = if is_active {
            Style::default()
                .fg(theme.primary_color())
                .bg(theme.selection_bg())
                .add_modifier(Modifier::BOLD)
        } else if self.is_focused {
            Style::default().fg(theme.text_color())
        } else {
            Style::default().fg(theme.muted_color())
        };

        let paragraph = Paragraph::new(label)
            .style(style)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Render dropdown menu
    fn render_dropdown(
        &self,
        frame: &mut Frame,
        menu_bar_area: Rect,
        menu_type: MenuType,
        theme: &crate::tui::themes::Theme,
    ) {
        let items = self.menus.get(&menu_type).unwrap();

        // Calculate dropdown position
        let x_offset = match menu_type {
            MenuType::File => 0,
            MenuType::View => 6,
            MenuType::Help => 12,
        };

        // Calculate dropdown size
        let width = 40;
        let height = (items.len() + 2).min(20) as u16;

        let dropdown_area = Rect {
            x: menu_bar_area.x + x_offset,
            y: menu_bar_area.y + 1,
            width,
            height,
        };

        // Clear background
        frame.render_widget(Clear, dropdown_area);

        // Create menu items
        let menu_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                if matches!(item.action, MenuAction::Separator) {
                    ListItem::new(Line::from("─".repeat(width as usize - 2)))
                        .style(Style::default().fg(theme.border_color()))
                } else {
                    let is_selected = idx == self.selected_item;
                    let style = if is_selected {
                        Style::default()
                            .fg(theme.primary_color())
                            .bg(theme.selection_bg())
                    } else if item.enabled {
                        Style::default().fg(theme.text_color())
                    } else {
                        Style::default().fg(theme.muted_color())
                    };

                    // Format item with shortcut
                    let content = if let Some(shortcut) = &item.shortcut {
                        format!("{:<25} {}", item.label, shortcut)
                    } else {
                        item.label.clone()
                    };

                    ListItem::new(content).style(style)
                }
            })
            .collect();

        let menu_list = List::new(menu_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .style(Style::default().bg(theme.background_color())),
        );

        frame.render_widget(menu_list, dropdown_area);
    }

    /// Handle key events
    pub fn handle_key_event(&mut self, key: KeyEvent) -> MenuResult {
        if !self.is_focused && self.active_menu.is_none() {
            return MenuResult::None;
        }

        match key.code {
            KeyCode::Esc => {
                self.active_menu = None;
                self.selected_item = 0;
                self.is_focused = false;
                MenuResult::Close
            }
            KeyCode::Left => {
                if let Some(current) = self.active_menu {
                    self.active_menu = Some(match current {
                        MenuType::File => MenuType::Help,
                        MenuType::View => MenuType::File,
                        MenuType::Help => MenuType::View,
                    });
                    self.selected_item = 0;
                }
                MenuResult::Continue
            }
            KeyCode::Right => {
                if let Some(current) = self.active_menu {
                    self.active_menu = Some(match current {
                        MenuType::File => MenuType::View,
                        MenuType::View => MenuType::Help,
                        MenuType::Help => MenuType::File,
                    });
                    self.selected_item = 0;
                }
                MenuResult::Continue
            }
            KeyCode::Up => {
                if self.active_menu.is_some() {
                    self.move_selection_up();
                }
                MenuResult::Continue
            }
            KeyCode::Down => {
                if self.active_menu.is_some() {
                    self.move_selection_down();
                } else if self.is_focused {
                    // Open first menu
                    self.active_menu = Some(MenuType::File);
                    self.selected_item = 0;
                }
                MenuResult::Continue
            }
            KeyCode::Enter => {
                if let Some(menu_type) = self.active_menu {
                    if let Some(items) = self.menus.get(&menu_type) {
                        if let Some(item) = items.get(self.selected_item) {
                            if item.enabled && !matches!(item.action, MenuAction::Separator) {
                                self.active_menu = None;
                                self.selected_item = 0;
                                self.is_focused = false;
                                return MenuResult::Action(item.action.clone());
                            }
                        }
                    }
                } else if self.is_focused {
                    // Open first menu
                    self.active_menu = Some(MenuType::File);
                    self.selected_item = 0;
                }
                MenuResult::Continue
            }
            KeyCode::Char(c) => {
                // Alt+F, Alt+V, Alt+H shortcuts
                if key.modifiers.contains(KeyModifiers::ALT) {
                    match c.to_lowercase().next() {
                        Some('f') => {
                            self.active_menu = Some(MenuType::File);
                            self.selected_item = 0;
                            self.is_focused = true;
                            return MenuResult::Continue;
                        }
                        Some('v') => {
                            self.active_menu = Some(MenuType::View);
                            self.selected_item = 0;
                            self.is_focused = true;
                            return MenuResult::Continue;
                        }
                        Some('h') => {
                            self.active_menu = Some(MenuType::Help);
                            self.selected_item = 0;
                            self.is_focused = true;
                            return MenuResult::Continue;
                        }
                        _ => {}
                    }
                }
                MenuResult::None
            }
            _ => MenuResult::None,
        }
    }

    /// Move selection up in menu
    fn move_selection_up(&mut self) {
        if let Some(menu_type) = self.active_menu {
            if let Some(items) = self.menus.get(&menu_type) {
                let mut new_idx = self.selected_item;
                loop {
                    if new_idx == 0 {
                        new_idx = items.len() - 1;
                    } else {
                        new_idx -= 1;
                    }

                    // Skip separators
                    if !matches!(items[new_idx].action, MenuAction::Separator) {
                        self.selected_item = new_idx;
                        break;
                    }

                    // Prevent infinite loop
                    if new_idx == self.selected_item {
                        break;
                    }
                }
            }
        }
    }

    /// Move selection down in menu
    fn move_selection_down(&mut self) {
        if let Some(menu_type) = self.active_menu {
            if let Some(items) = self.menus.get(&menu_type) {
                let mut new_idx = self.selected_item;
                loop {
                    new_idx = (new_idx + 1) % items.len();

                    // Skip separators
                    if !matches!(items[new_idx].action, MenuAction::Separator) {
                        self.selected_item = new_idx;
                        break;
                    }

                    // Prevent infinite loop
                    if new_idx == self.selected_item {
                        break;
                    }
                }
            }
        }
    }

    /// Focus the menu bar
    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    /// Unfocus the menu bar
    pub fn unfocus(&mut self) {
        self.is_focused = false;
        self.active_menu = None;
        self.selected_item = 0;
    }

    /// Check if menu is active
    pub fn is_active(&self) -> bool {
        self.active_menu.is_some()
    }
}
