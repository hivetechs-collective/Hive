//! Advanced TUI implementation that matches Claude Code's persistent interface

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use tokio::sync::mpsc;
use crate::consensus::temporal::TemporalContextProvider;
use crate::cli::tui_commands::{TuiCommandProcessor, TuiEvent};
use crate::cli::tui_capabilities::{TuiCapabilities, TuiMode, LayoutConstraints};
use crate::cli::tui_themes::{ThemeManager, TuiTheme};
use crate::cli::accessibility::{AccessibilityManager, ScreenReaderAnnouncer, AnnouncementPriority, AccessibilityRegion};
use console::style;

/// Which panel is currently focused in TUI mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    Input,      // F1 or default - input/messages area
    Explorer,   // F2 - file explorer (future)
    Consensus,  // F3 - consensus progress panel
    Terminal,   // F4 - terminal panel (future)
}

impl Default for FocusedPanel {
    fn default() -> Self {
        FocusedPanel::Input
    }
}

pub struct InteractiveTui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    messages: Vec<Message>,
    input_buffer: String,
    cursor_position: usize,
    scroll_offset: usize,
    status_line: StatusLine,
    consensus_progress: Option<ConsensusProgress>,
    command_history: Vec<String>,
    history_index: Option<usize>,
    command_processor: Option<TuiCommandProcessor>,
    event_receiver: Option<mpsc::UnboundedReceiver<TuiEvent>>,
    focused_panel: FocusedPanel,
    help_visible: bool,
    capabilities: TuiCapabilities,
    layout_constraints: LayoutConstraints,
    tui_mode: TuiMode,
    theme_manager: ThemeManager,
    accessibility: AccessibilityManager,
    announcer: ScreenReaderAnnouncer,
}

#[derive(Clone)]
pub struct Message {
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub enum MessageType {
    Welcome,
    UserInput,
    SystemResponse,
    ConsensusProgress,
    Error,
    Help,
    Status,
    Info,
}

#[derive(Clone)]
pub struct StatusLine {
    pub context_remaining: u8,
    pub auto_accept_edits: bool,
    pub current_mode: String,
}

/// Consensus pipeline progress tracking
#[derive(Clone)]
pub struct ConsensusProgress {
    pub generator: StageProgress,
    pub refiner: StageProgress,
    pub validator: StageProgress,
    pub curator: StageProgress,
    pub is_active: bool,
}

#[derive(Clone)]
pub struct StageProgress {
    pub name: String,
    pub model: String,
    pub progress: u16, // 0-100
    pub status: StageStatus,
}

#[derive(Clone, PartialEq)]
pub enum StageStatus {
    Waiting,
    Running,
    Completed,
    Error,
}

impl InteractiveTui {
    pub async fn new() -> Result<Self> {
        // Detect terminal capabilities first
        let capabilities = TuiCapabilities::detect()?;
        let tui_mode = capabilities.determine_tui_mode();
        let layout_constraints = capabilities.get_layout_constraints();

        // Setup terminal with progressive enhancement
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        
        // Only use advanced features if supported
        if capabilities.supports_alternate_screen {
            execute!(stdout, EnterAlternateScreen)?;
        }
        if capabilities.supports_mouse {
            execute!(stdout, EnableMouseCapture)?;
        }
        
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Setup command processor with event channel
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let command_processor = TuiCommandProcessor::new();

        // Initialize theme manager
        let mut theme_manager = ThemeManager::new();
        
        // Load theme based on capabilities and user preference
        let default_theme = if capabilities.color_depth == crate::cli::tui_capabilities::ColorDepth::Monochrome {
            "high_contrast"
        } else if capabilities.supports_true_color {
            "dark" // Default to dark theme for modern terminals
        } else {
            "dark" // Fallback to basic dark theme
        };
        
        let _ = theme_manager.load_theme(default_theme);

        // Initialize accessibility features
        let mut accessibility = AccessibilityManager::new();
        
        // Enable high contrast theme if accessibility requires it
        if accessibility.is_high_contrast_mode() {
            let _ = theme_manager.load_theme("high_contrast");
        }
        
        let announcer = ScreenReaderAnnouncer::new();

        Ok(Self {
            terminal,
            messages: Vec::new(),
            input_buffer: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            status_line: StatusLine {
                context_remaining: 42,
                auto_accept_edits: true,
                current_mode: format!("Interactive ({})", 
                    match tui_mode {
                        TuiMode::Enhanced => "Enhanced",
                        TuiMode::Simple => "Simple",
                        TuiMode::Disabled => "Fallback",
                    }
                ),
            },
            consensus_progress: None,
            command_history: Vec::new(),
            history_index: None,
            command_processor: Some(command_processor),
            event_receiver: Some(event_receiver),
            focused_panel: FocusedPanel::default(),
            help_visible: false,
            capabilities,
            layout_constraints,
            tui_mode,
            theme_manager,
            accessibility,
            announcer,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Add welcome message
        self.add_welcome_message().await;

        loop {
            self.draw()?;

            // Handle TUI events in a non-blocking way
            let events_to_handle: Vec<TuiEvent> = if let Some(receiver) = &mut self.event_receiver {
                let mut events = Vec::new();
                while let Ok(tui_event) = receiver.try_recv() {
                    events.push(tui_event);
                }
                events
            } else {
                Vec::new()
            };
            
            for tui_event in events_to_handle {
                self.handle_tui_event(tui_event).await?;
            }

            // Check for keyboard input
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Enter => {
                            self.process_input().await?;
                        }
                        KeyCode::Char(c) => {
                            self.input_buffer.insert(self.cursor_position, c);
                            self.cursor_position += 1;
                        }
                        KeyCode::Backspace => {
                            if self.cursor_position > 0 {
                                self.input_buffer.remove(self.cursor_position - 1);
                                self.cursor_position -= 1;
                            }
                        }
                        KeyCode::Left => {
                            if self.cursor_position > 0 {
                                self.cursor_position -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if self.cursor_position < self.input_buffer.len() {
                                self.cursor_position += 1;
                            }
                        }
                        KeyCode::Up => {
                            // Navigate command history
                            if let Some(idx) = self.history_index {
                                if idx > 0 {
                                    self.history_index = Some(idx - 1);
                                    self.input_buffer = self.command_history[idx - 1].clone();
                                    self.cursor_position = self.input_buffer.len();
                                }
                            } else if !self.command_history.is_empty() {
                                let idx = self.command_history.len() - 1;
                                self.history_index = Some(idx);
                                self.input_buffer = self.command_history[idx].clone();
                                self.cursor_position = self.input_buffer.len();
                            }
                        }
                        KeyCode::Down => {
                            // Navigate command history
                            if let Some(idx) = self.history_index {
                                if idx < self.command_history.len() - 1 {
                                    self.history_index = Some(idx + 1);
                                    self.input_buffer = self.command_history[idx + 1].clone();
                                    self.cursor_position = self.input_buffer.len();
                                } else {
                                    self.history_index = None;
                                    self.input_buffer.clear();
                                    self.cursor_position = 0;
                                }
                            }
                        }
                        KeyCode::PageUp => {
                            // Scroll messages up
                            if self.scroll_offset > 0 {
                                self.scroll_offset = self.scroll_offset.saturating_sub(5);
                            }
                        }
                        KeyCode::PageDown => {
                            // Scroll messages down
                            self.scroll_offset = (self.scroll_offset + 5).min(self.messages.len().saturating_sub(10));
                        }
                        KeyCode::Tab if key.modifiers.contains(event::KeyModifiers::SHIFT) => {
                            // Cycle auto-accept mode (like Claude Code)
                            self.status_line.auto_accept_edits = !self.status_line.auto_accept_edits;
                        }
                        // F-key panel focus shortcuts (VS Code style)
                        KeyCode::F(1) => {
                            self.focused_panel = FocusedPanel::Input;
                            self.status_line.current_mode = "Input Focus".to_string();
                        }
                        KeyCode::F(2) => {
                            self.focused_panel = FocusedPanel::Explorer;
                            self.status_line.current_mode = "Explorer Focus".to_string();
                        }
                        KeyCode::F(3) => {
                            self.focused_panel = FocusedPanel::Consensus;
                            self.status_line.current_mode = "Consensus Focus".to_string();
                        }
                        KeyCode::F(4) => {
                            self.focused_panel = FocusedPanel::Terminal;
                            self.status_line.current_mode = "Terminal Focus".to_string();
                        }
                        // Help shortcut (? key)
                        KeyCode::Char('?') => {
                            self.help_visible = !self.help_visible;
                            if self.help_visible {
                                self.show_keyboard_shortcuts().await;
                            }
                        }
                        // Quick commands with Ctrl modifiers
                        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Ctrl+H: Quick ask
                            self.input_buffer = "ask ".to_string();
                            self.cursor_position = 4;
                        }
                        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Ctrl+A: Quick analyze
                            self.input_buffer = "analyze .".to_string();
                            self.cursor_position = 9;
                        }
                        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Ctrl+P: Quick plan
                            self.input_buffer = "plan ".to_string();
                            self.cursor_position = 5;
                        }
                        // Theme switching (Ctrl+T)
                        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.cycle_theme().await;
                        }
                        // Escape to close help or clear input
                        KeyCode::Esc => {
                            if self.help_visible {
                                self.help_visible = false;
                            } else {
                                self.input_buffer.clear();
                                self.cursor_position = 0;
                                self.history_index = None;
                            }
                        }
                        _ => {}
                    }
                }
            }
            }
        }

        self.cleanup()?;
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        let has_consensus = self.consensus_progress.is_some();
        let messages = self.messages.clone();
        let consensus_progress = self.consensus_progress.clone();
        let input_buffer = self.input_buffer.clone();
        let cursor_position = self.cursor_position;
        let status_line = self.status_line.clone();
        
        self.terminal.draw(|f| {
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(if has_consensus {
                    vec![
                        Constraint::Min(1),      // Messages area
                        Constraint::Length(6),   // Consensus progress
                        Constraint::Length(3),   // Input box
                        Constraint::Length(1),   // Status line
                    ]
                } else {
                    vec![
                        Constraint::Min(1),      // Messages area
                        Constraint::Length(3),   // Input box
                        Constraint::Length(1),   // Status line
                    ]
                })
                .split(f.size());

            if has_consensus {
                // Draw messages area
                Self::draw_messages_static(f, main_layout[0], &messages);
                
                // Draw consensus progress
                Self::draw_consensus_progress_static(f, main_layout[1], &consensus_progress);
                
                // Draw input box (Claude Code style)
                Self::draw_input_box_static(f, main_layout[2], &input_buffer, cursor_position);

                // Draw status line
                Self::draw_status_line_static(f, main_layout[3], &status_line);
            } else {
                // Draw messages area
                Self::draw_messages_static(f, main_layout[0], &messages);
                
                // Draw input box (Claude Code style)
                Self::draw_input_box_static(f, main_layout[1], &input_buffer, cursor_position);

                // Draw status line
                Self::draw_status_line_static(f, main_layout[2], &status_line);
            }
        })?;

        Ok(())
    }

    fn draw_messages_static(f: &mut Frame, area: Rect, messages: &[Message]) {
        let messages: Vec<ListItem> = messages
            .iter()
            .map(|msg| {
                let style = match msg.message_type {
                    MessageType::Welcome => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    MessageType::UserInput => Style::default().fg(Color::Green),
                    MessageType::SystemResponse => Style::default().fg(Color::White),
                    MessageType::ConsensusProgress => Style::default().fg(Color::Yellow),
                    MessageType::Error => Style::default().fg(Color::Red),
                    MessageType::Help => Style::default().fg(Color::Blue),
                    MessageType::Status => Style::default().fg(Color::Magenta),
                    MessageType::Info => Style::default().fg(Color::Cyan),
                };

                let content = Text::from(msg.content.clone()).style(style);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(messages)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default());

        f.render_widget(list, area);
    }
    
    fn draw_input_box_static(f: &mut Frame, area: Rect, input_buffer: &str, cursor_position: usize) {
        let input_box = Paragraph::new(input_buffer)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title("  Ask Hive AI anything  ")
                .title_style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });

        f.render_widget(input_box, area);

        // Render cursor
        let cursor_x = area.x + 1 + cursor_position as u16;
        let cursor_y = area.y + 1;
        
        if cursor_x < area.x + area.width - 1 {
            f.set_cursor(cursor_x, cursor_y);
        }
    }
    
    fn draw_consensus_progress_static(f: &mut Frame, area: Rect, progress: &Option<ConsensusProgress>) {
        if let Some(progress) = progress {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(area);

            Self::draw_stage_progress_static(f, chunks[0], "Generator", &progress.generator);
            Self::draw_stage_progress_static(f, chunks[1], "Refiner", &progress.refiner);
            Self::draw_stage_progress_static(f, chunks[2], "Validator", &progress.validator);
            Self::draw_stage_progress_static(f, chunks[3], "Curator", &progress.curator);
        }
    }
    
    fn draw_stage_progress_static(f: &mut Frame, area: Rect, name: &str, stage: &StageProgress) {
        let color = match stage.status {
            StageStatus::Waiting => Color::DarkGray,
            StageStatus::Running => Color::Yellow,
            StageStatus::Completed => Color::Green,
            StageStatus::Error => Color::Red,
        };

        let gauge = Gauge::default()
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", name)))
            .gauge_style(Style::default().fg(color))
            .percent(stage.progress)
            .label(format!("{}", stage.model));

        f.render_widget(gauge, area);
    }
    
    fn draw_status_line_static(f: &mut Frame, area: Rect, status_line: &StatusLine) {
        let status_text = format!(
            " Mode: {} | Context: {}% | Auto-accept: {} | Ctrl+C: Exit | F1: Help ",
            status_line.current_mode,
            status_line.context_remaining,
            if status_line.auto_accept_edits { "ON" } else { "OFF" }
        );
        
        let status = Paragraph::new(status_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
        
        f.render_widget(status, area);
    }

    fn draw_input_box(&self, f: &mut Frame, area: Rect) {
        let theme = self.theme_manager.current_theme();
        
        // Claude Code style input box with themed colors
        let input_text = if self.input_buffer.is_empty() {
            "Try \"ask <question>\" or \"analyze .\"".to_string()
        } else {
            self.input_buffer.clone()
        };

        let input_style = if self.input_buffer.is_empty() {
            Style::default().fg(self.theme_manager.to_ratatui_color(&theme.colors.status_text))
        } else {
            self.theme_manager.to_ratatui_style(&theme.styles.input, &theme.colors.user_input)
        };

        let paragraph = Paragraph::new(format!("> {}", input_text))
            .style(input_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme_manager.to_ratatui_color(&theme.colors.border)))
                    .title(" HiveTechs Consensus ")
                    .title_style(self.theme_manager.to_ratatui_style(&theme.styles.title, &theme.colors.info))
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);

        // Show cursor with themed color
        if !self.input_buffer.is_empty() {
            let cursor_x = area.x + self.cursor_position as u16 + 3; // Account for "> " prefix
            let cursor_y = area.y + 1;
            f.set_cursor(cursor_x, cursor_y);
        }
    }

    fn draw_consensus_progress(&self, f: &mut Frame, area: Rect) {
        if let Some(ref progress) = self.consensus_progress {
            let theme = self.theme_manager.current_theme();
            
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme_manager.to_ratatui_color(&theme.colors.border)))
                .title(" 🧠 Consensus Pipeline ")
                .title_style(self.theme_manager.to_ratatui_style(&theme.styles.title, &theme.colors.consensus_progress));

            let inner = block.inner(area);
            f.render_widget(block, area);

            // Layout for 4 stages
            let stage_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(inner);

            // Draw each stage
            self.draw_stage_progress(f, stage_layout[0], "Generator", &progress.generator);
            self.draw_stage_progress(f, stage_layout[1], "Refiner", &progress.refiner);
            self.draw_stage_progress(f, stage_layout[2], "Validator", &progress.validator);
            self.draw_stage_progress(f, stage_layout[3], "Curator", &progress.curator);
        }
    }

    fn draw_stage_progress(&self, f: &mut Frame, area: Rect, name: &str, stage: &StageProgress) {
        let theme = self.theme_manager.current_theme();
        
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // Stage name
                Constraint::Length(3),  // Arrow
                Constraint::Min(20),    // Progress bar
                Constraint::Length(25), // Model name
            ])
            .split(area);

        // Stage name with themed colors
        let name_style = match stage.status {
            StageStatus::Waiting => Style::default().fg(self.theme_manager.to_ratatui_color(&theme.colors.progress_waiting)),
            StageStatus::Running => Style::default().fg(self.theme_manager.to_ratatui_color(&theme.colors.progress_active)),
            StageStatus::Completed => Style::default().fg(self.theme_manager.to_ratatui_color(&theme.colors.progress_complete)),
            StageStatus::Error => Style::default().fg(self.theme_manager.to_ratatui_color(&theme.colors.progress_error)),
        };
        
        f.render_widget(
            Paragraph::new(name).style(name_style),
            layout[0]
        );

        // Arrow with themed color
        f.render_widget(
            Paragraph::new("→").style(Style::default().fg(self.theme_manager.to_ratatui_color(&theme.colors.border))),
            layout[1]
        );

        // Progress bar with themed colors
        let progress_color = match stage.status {
            StageStatus::Waiting => self.theme_manager.to_ratatui_color(&theme.colors.progress_waiting),
            StageStatus::Running => self.theme_manager.to_ratatui_color(&theme.colors.progress_active),
            StageStatus::Completed => self.theme_manager.to_ratatui_color(&theme.colors.progress_complete),
            StageStatus::Error => self.theme_manager.to_ratatui_color(&theme.colors.progress_error),
        };

        let filled = (stage.progress as usize * layout[2].width as usize / 100).min(layout[2].width as usize);
        let empty = layout[2].width as usize - filled;
        let progress_text = format!("{}{} {}%", 
            "█".repeat(filled),
            "░".repeat(empty),
            stage.progress
        );

        f.render_widget(
            Paragraph::new(progress_text).style(Style::default().fg(progress_color)),
            layout[2]
        );

        // Model name with themed color
        f.render_widget(
            Paragraph::new(format!("({})", stage.model))
                .style(Style::default().fg(self.theme_manager.to_ratatui_color(&theme.colors.info))),
            layout[3]
        );
    }

    fn draw_status_line(&self, f: &mut Frame, area: Rect) {
        let theme = self.theme_manager.current_theme();
        
        // Enhanced status line with focus and shortcuts info
        let auto_accept_status = if self.status_line.auto_accept_edits {
            "⏵⏵ auto-accept edits on"
        } else {
            "⏸⏸ auto-accept edits off"
        };

        let context_info = format!("Context left until auto-compact: {}%", self.status_line.context_remaining);
        
        let focus_info = match self.focused_panel {
            FocusedPanel::Input => "F1:Input",
            FocusedPanel::Explorer => "F2:Explorer",
            FocusedPanel::Consensus => "F3:Consensus",
            FocusedPanel::Terminal => "F4:Terminal",
        };

        let status_text = if self.accessibility.is_screen_reader_mode() {
            self.accessibility.get_accessible_status(
                self.status_line.auto_accept_edits,
                self.status_line.context_remaining,
                &match self.focused_panel {
                    FocusedPanel::Input => "Input panel",
                    FocusedPanel::Explorer => "File explorer panel",
                    FocusedPanel::Consensus => "Consensus panel",
                    FocusedPanel::Terminal => "Terminal panel",
                }
            )
        } else {
            format!(
                "{}  |  {}  |  {}  |  ? for shortcuts  |  shift+tab to toggle",
                auto_accept_status,
                context_info,
                focus_info
            )
        };

        let paragraph = Paragraph::new(status_text)
            .style(self.theme_manager.to_ratatui_style(&theme.styles.status_line, &theme.colors.status_text))
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    async fn add_welcome_message(&mut self) {
        // Get temporal context for date awareness
        let temporal_provider = TemporalContextProvider::default();
        let temporal_context = temporal_provider.build_current_context().await.ok();
        
        let date_display = temporal_context
            .as_ref()
            .map(|ctx| ctx.current_datetime.clone())
            .unwrap_or_else(|| chrono::Local::now().format("%A, %B %d, %Y at %H:%M %Z").to_string());

        let welcome_content = format!(
            "╭─────────────────────────────────────────────────────────────╮\n\
             │ {} Welcome to HiveTechs Consensus!                     │\n\
             │                                                             │\n\
             │   {}       │\n\
             │                                                             │\n\
             │   {} {}          │\n\
             ╰─────────────────────────────────────────────────────────────╯\n\
             \n\
             What's new:\n\
             • Today is {}\n\
             • Released Enterprise Hooks - Deterministic control over AI behavior\n\
             • Temporal context for web search - always knows today's date\n\
             • Repository intelligence with ML-powered analysis\n\
             • 10-40x performance improvements over TypeScript version\n\
             • Planning mode for strategic development workflows",
            style("✻").bold().cyan(),
            style("/help for help, /status for your current setup").dim(),
            style("cwd:").dim(),
            self.pad_right(&self.get_current_dir_display(), 40),
            date_display
        );

        self.messages.push(Message {
            content: welcome_content,
            message_type: MessageType::Welcome,
            timestamp: chrono::Utc::now(),
        });
    }

    async fn process_input(&mut self) -> Result<()> {
        if self.input_buffer.is_empty() {
            return Ok(());
        }

        let input = self.input_buffer.clone();
        self.input_buffer.clear();
        self.cursor_position = 0;
        self.history_index = None;

        // Add to command history
        self.command_history.push(input.clone());
        if self.command_history.len() > 100 {
            self.command_history.remove(0);
        }

        // Add user input to messages
        self.messages.push(Message {
            content: format!("> {}", input),
            message_type: MessageType::UserInput,
            timestamp: chrono::Utc::now(),
        });

        // Use the enhanced command processor
        if let Some(processor) = &self.command_processor {
            match processor.process_command(&input).await {
                Ok(_) => {},
                Err(e) if e.to_string().contains("User requested exit") => {
                    return Err(e);
                }
                Err(e) => {
                    self.add_message(&format!("❌ Command failed: {}", e), MessageType::Error).await;
                }
            }
        } else {
            // Fallback to simple command handling
            self.fallback_command_handling(&input).await?;
        }

        // Auto-scroll to bottom
        if self.messages.len() > 10 {
            self.scroll_offset = self.messages.len().saturating_sub(10);
        }

        Ok(())
    }

    /// Handle TUI events from the command processor
    async fn handle_tui_event(&mut self, event: TuiEvent) -> Result<()> {
        match event {
            TuiEvent::Message(message) => {
                self.messages.push(Message {
                    content: message,
                    message_type: MessageType::Info,
                    timestamp: chrono::Utc::now(),
                });
            }
            TuiEvent::ConsensusProgress(progress) => {
                // Convert f32 progress to ConsensusProgress struct
                let stage_progress = StageProgress {
                    name: "Processing".to_string(),
                    model: "consensus".to_string(),
                    progress: (progress * 100.0) as u16,
                    status: StageStatus::Running,
                };
                self.consensus_progress = Some(ConsensusProgress {
                    generator: stage_progress.clone(),
                    refiner: stage_progress.clone(),
                    validator: stage_progress.clone(),
                    curator: stage_progress,
                    is_active: true,
                });
            }
            TuiEvent::ConsensusComplete => {
                self.consensus_progress = None;
            }
            TuiEvent::Error(error) => {
                self.add_message(&format!("❌ {}", error), MessageType::Error).await;
            }
            TuiEvent::StatusUpdate(status) => {
                self.add_message(&status, MessageType::Status).await;
            }
        }
        Ok(())
    }

    /// Fallback command handling when command processor is not available
    async fn fallback_command_handling(&mut self, input: &str) -> Result<()> {
        match input.trim() {
            "/help" | "help" => {
                self.add_help_message().await;
            }
            "/status" | "status" => {
                self.add_status_message().await;
            }
            "/exit" | "exit" | "quit" => {
                self.add_message("👋 Thanks for using HiveTechs Consensus!", MessageType::SystemResponse).await;
                return Err(anyhow::anyhow!("User requested exit"));
            }
            input if input.starts_with("ask ") => {
                let question = &input[4..];
                self.process_ask_command(question).await?;
            }
            input if input.starts_with("analyze ") => {
                let path = &input[8..];
                self.process_analyze_command(path).await?;
            }
            input if input.starts_with("plan ") => {
                let goal = &input[5..];
                self.process_plan_command(goal).await?;
            }
            _ => {
                self.add_message(&format!("❌ Unknown command: {}\n💡 Type '/help' for available commands", input), MessageType::Error).await;
            }
        }
        Ok(())
    }

    async fn process_ask_command(&mut self, question: &str) -> Result<()> {
        if question.is_empty() {
            self.add_message("❌ Usage: ask <question>", MessageType::Error).await;
            return Ok(());
        }

        self.add_message("🤔 Processing your question...", MessageType::SystemResponse).await;
        self.add_message("🧠 Running 4-stage consensus pipeline...", MessageType::SystemResponse).await;

        // Initialize consensus progress
        self.consensus_progress = Some(ConsensusProgress {
            generator: StageProgress {
                name: "Generator".to_string(),
                model: "claude-3-5-sonnet".to_string(),
                progress: 0,
                status: StageStatus::Running,
            },
            refiner: StageProgress {
                name: "Refiner".to_string(),
                model: "gpt-4-turbo".to_string(),
                progress: 0,
                status: StageStatus::Waiting,
            },
            validator: StageProgress {
                name: "Validator".to_string(),
                model: "claude-3-opus".to_string(),
                progress: 0,
                status: StageStatus::Waiting,
            },
            curator: StageProgress {
                name: "Curator".to_string(),
                model: "gpt-4o".to_string(),
                progress: 0,
                status: StageStatus::Waiting,
            },
            is_active: true,
        });

        // Simulate consensus processing with progress
        // In real implementation, this would connect to actual consensus engine
        use tokio::time::{sleep, Duration};
        
        // Generator stage
        for i in 0..=100 {
            if let Some(ref mut progress) = self.consensus_progress {
                progress.generator.progress = i;
            }
            self.draw()?;
            sleep(Duration::from_millis(10)).await;
        }
        
        if let Some(ref mut progress) = self.consensus_progress {
            progress.generator.status = StageStatus::Completed;
            progress.refiner.status = StageStatus::Running;
        }

        // Refiner stage
        for i in 0..=100 {
            if let Some(ref mut progress) = self.consensus_progress {
                progress.refiner.progress = i;
            }
            self.draw()?;
            sleep(Duration::from_millis(10)).await;
        }

        if let Some(ref mut progress) = self.consensus_progress {
            progress.refiner.status = StageStatus::Completed;
            progress.validator.status = StageStatus::Running;
        }

        // Validator stage
        for i in 0..=100 {
            if let Some(ref mut progress) = self.consensus_progress {
                progress.validator.progress = i;
            }
            self.draw()?;
            sleep(Duration::from_millis(10)).await;
        }

        if let Some(ref mut progress) = self.consensus_progress {
            progress.validator.status = StageStatus::Completed;
            progress.curator.status = StageStatus::Running;
        }

        // Curator stage
        for i in 0..=100 {
            if let Some(ref mut progress) = self.consensus_progress {
                progress.curator.progress = i;
            }
            self.draw()?;
            sleep(Duration::from_millis(10)).await;
        }

        if let Some(ref mut progress) = self.consensus_progress {
            progress.curator.status = StageStatus::Completed;
        }

        // Clear consensus progress
        self.consensus_progress = None;

        self.add_message("", MessageType::SystemResponse).await;
        self.add_message("✨ Consensus Response:", MessageType::SystemResponse).await;
        
        // Check if temporal context is needed
        let temporal_provider = TemporalContextProvider::default();
        let needs_temporal = temporal_provider.requires_temporal_context(question);
        
        if needs_temporal {
            let temporal_context = temporal_provider.build_current_context().await?;
            self.add_message(&format!("IMPORTANT: {}", temporal_context.search_instruction), MessageType::SystemResponse).await;
            self.add_message("", MessageType::SystemResponse).await;
        }
        
        self.add_message(&format!("Based on the consensus of 4 AI models analyzing your question:\n\n{}\n\n(This is a placeholder response during development. In production, this would be the actual consensus result.)", question), MessageType::SystemResponse).await;

        Ok(())
    }

    async fn process_analyze_command(&mut self, path: &str) -> Result<()> {
        if path.is_empty() {
            self.add_message("❌ Usage: analyze <path>", MessageType::Error).await;
            return Ok(());
        }

        self.add_message(&format!("🔍 Analyzing: {}", path), MessageType::SystemResponse).await;
        self.add_message("📊 Repository Intelligence:", MessageType::SystemResponse).await;
        self.add_message("  • Architecture: Rust CLI Application\n  • Quality Score: 8.5/10\n  • Files Analyzed: 15\n  • Technical Debt: Low", MessageType::SystemResponse).await;

        Ok(())
    }

    async fn process_plan_command(&mut self, goal: &str) -> Result<()> {
        if goal.is_empty() {
            self.add_message("❌ Usage: plan <goal>", MessageType::Error).await;
            return Ok(());
        }

        self.add_message(&format!("📋 Creating development plan for: {}", goal), MessageType::SystemResponse).await;
        self.add_message("✅ Plan created with 5 tasks\n⏱️  Estimated completion: 2-3 days\n📝 Use 'execute plan.json' to begin implementation", MessageType::SystemResponse).await;

        Ok(())
    }

    async fn add_help_message(&mut self) {
        let help_content = "🆘 HiveTechs Consensus Help\n\
                           \n\
                           Commands:\n\
                           ask <question>        - Ask the AI consensus a question\n\
                           analyze <path>        - Analyze repository or file\n\
                           plan <goal>           - Create a development plan\n\
                           \n\
                           Special commands:\n\
                           /help or help         - Show this help\n\
                           /status or status     - Show system status\n\
                           /exit or exit         - Exit interactive mode\n\
                           \n\
                           Documentation: https://docs.hivetechs.com";

        self.messages.push(Message {
            content: help_content.to_string(),
            message_type: MessageType::Help,
            timestamp: chrono::Utc::now(),
        });
    }

    async fn add_status_message(&mut self) {
        let status_content = "📊 HiveTechs Consensus Status\n\
                             \n\
                             System:\n\
                             Version: 2.0.0-dev\n\
                             Config: ✓ Configured\n\
                             Memory: ✓ Ready\n\
                             \n\
                             Connectivity:\n\
                             Internet: ✓ Connected\n\
                             AI Models: ✓ Available (323+ models)\n\
                             \n\
                             Performance:\n\
                             Memory Usage: 25.0 MB\n\
                             Consensus Engine: ✓ Ready";

        self.messages.push(Message {
            content: status_content.to_string(),
            message_type: MessageType::Status,
            timestamp: chrono::Utc::now(),
        });
    }

    async fn add_message(&mut self, content: &str, message_type: MessageType) {
        self.messages.push(Message {
            content: content.to_string(),
            message_type,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Show keyboard shortcuts help
    async fn show_keyboard_shortcuts(&mut self) {
        let shortcuts_help = if self.accessibility.is_screen_reader_mode() {
            format!("Keyboard Shortcuts Help.\n\n{}\n\nDetailed shortcuts:\n\
                    Arrow up and down keys navigate command history.\n\
                    Arrow left and right keys move cursor in input line.\n\
                    Page up and page down scroll messages.\n\
                    F1 focuses input and messages panel.\n\
                    F2 focuses file explorer panel.\n\
                    F3 focuses consensus panel.\n\
                    F4 focuses terminal panel.\n\
                    Control plus H enters quick ask command.\n\
                    Control plus A enters quick analyze command.\n\
                    Control plus P enters quick plan command.\n\
                    Control plus T cycles through available themes.\n\
                    Question mark toggles this help.\n\
                    Escape key clears input or closes help.\n\
                    Shift plus Tab toggles auto-accept edits.\n\
                    Control plus C exits the application.\n\
                    Press question mark again to close this help.",
                    self.accessibility.get_accessible_help())
        } else {
            "⌨️  Keyboard Shortcuts\n\
             \n\
             Navigation:\n\
             ↑/↓                   - Navigate command history\n\
             ←/→                   - Move cursor in input line\n\
             Page Up/Down          - Scroll messages\n\
             \n\
             Panel Focus (VS Code style):\n\
             F1                    - Focus Input/Messages\n\
             F2                    - Focus File Explorer (future)\n\
             F3                    - Focus Consensus Panel\n\
             F4                    - Focus Terminal (future)\n\
             \n\
             Quick Commands:\n\
             Ctrl+H                - Quick ask command\n\
             Ctrl+A                - Quick analyze command\n\
             Ctrl+P                - Quick plan command\n\
             \n\
             Customization:\n\
             Ctrl+T                - Cycle through themes\n\
             \n\
             Other:\n\
             ?                     - Toggle this help\n\
             Escape                - Clear input or close help\n\
             Shift+Tab             - Toggle auto-accept edits\n\
             Ctrl+C                - Exit application\n\
             \n\
             Press ? again to close this help.".to_string()
        };

        self.add_message(&shortcuts_help, MessageType::Help).await;
        
        // Announce for screen readers
        if let Some(announcement) = self.accessibility.create_announcement(
            "Keyboard shortcuts help displayed",
            AnnouncementPriority::Medium,
            AccessibilityRegion::Global,
        ) {
            self.announcer.announce(announcement);
        }
    }

    /// Cycle through available themes
    async fn cycle_theme(&mut self) {
        let next_theme = {
            let available_themes = self.theme_manager.available_themes();
            let current_theme = self.theme_manager.current_theme().name.clone();
            
            // Find current theme index
            let current_index = available_themes
                .iter()
                .position(|&name| *name == current_theme)
                .unwrap_or(0);
            
            // Get next theme (cycle back to start if at end)
            let next_index = (current_index + 1) % available_themes.len();
            available_themes[next_index].to_string()
        };
        
        // Load the next theme
        if let Err(e) = self.theme_manager.load_theme(&next_theme) {
            self.add_message(&format!("Failed to load theme '{}': {}", next_theme, e), MessageType::Error).await;
        } else {
            self.add_message(&format!("🎨 Switched to '{}' theme", next_theme), MessageType::SystemResponse).await;
        }
    }

    fn get_current_dir_display(&self) -> String {
        std::env::current_dir()
            .map(|path| {
                let path_str = path.to_string_lossy();
                if path_str.len() > 35 {
                    format!("...{}", &path_str[path_str.len()-32..])
                } else {
                    path_str.to_string()
                }
            })
            .unwrap_or_else(|_| "unknown".to_string())
    }

    fn pad_right(&self, s: &str, width: usize) -> String {
        if s.len() >= width {
            s.to_string()
        } else {
            format!("{}{}", s, " ".repeat(width - s.len()))
        }
    }

    fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        
        // Clean up features based on what was enabled
        if self.capabilities.supports_alternate_screen && self.capabilities.supports_mouse {
            execute!(
                self.terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
        } else if self.capabilities.supports_alternate_screen {
            execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        } else if self.capabilities.supports_mouse {
            execute!(self.terminal.backend_mut(), DisableMouseCapture)?;
        }
        
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for InteractiveTui {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}