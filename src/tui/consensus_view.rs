//! Real-time 4-Stage Consensus Visualization Component

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// Real-time consensus progress visualization
pub struct ConsensusView {
    theme: ConsensusTheme,
}

/// Consensus visualization theme
struct ConsensusTheme {
    border_color: Color,
    title_color: Color,
    waiting_color: Color,
    running_color: Color,
    completed_color: Color,
    error_color: Color,
    model_color: Color,
}

impl Default for ConsensusTheme {
    fn default() -> Self {
        Self {
            border_color: Color::Blue,
            title_color: Color::Cyan,
            waiting_color: Color::DarkGray,
            running_color: Color::Yellow,
            completed_color: Color::Green,
            error_color: Color::Red,
            model_color: Color::Magenta,
        }
    }
}

/// Progress tracking for consensus pipeline
#[derive(Clone, Debug)]
pub struct ConsensusProgress {
    pub generator: StageProgress,
    pub refiner: StageProgress,
    pub validator: StageProgress,
    pub curator: StageProgress,
    pub is_active: bool,
}

/// Progress for individual consensus stage
#[derive(Clone, Debug)]
pub struct StageProgress {
    pub name: String,
    pub model: String,
    pub progress: u16, // 0-100
    pub status: StageStatus,
}

/// Status of consensus stage
#[derive(Clone, Debug, PartialEq)]
pub enum StageStatus {
    Waiting,
    Running,
    Completed,
    Error,
}

impl ConsensusView {
    /// Create new consensus view
    pub fn new() -> Self {
        Self {
            theme: ConsensusTheme::default(),
        }
    }
    
    /// Draw the real-time consensus progress display
    pub fn draw(&self, frame: &mut Frame, area: Rect, progress: &ConsensusProgress) {
        // Main consensus block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_color))
            .title(" 🧠 Consensus Pipeline ")
            .title_style(
                Style::default()
                    .fg(self.theme.title_color)
                    .add_modifier(Modifier::BOLD)
            );
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        // Layout for 4 stages
        let stage_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Generator
                Constraint::Length(1), // Refiner
                Constraint::Length(1), // Validator
                Constraint::Length(1), // Curator
            ])
            .split(inner);
        
        // Draw each stage
        self.draw_stage_progress(frame, stage_layout[0], &progress.generator);
        self.draw_stage_progress(frame, stage_layout[1], &progress.refiner);
        self.draw_stage_progress(frame, stage_layout[2], &progress.validator);
        self.draw_stage_progress(frame, stage_layout[3], &progress.curator);
    }
    
    /// Draw individual stage progress
    fn draw_stage_progress(&self, frame: &mut Frame, area: Rect, stage: &StageProgress) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // Stage name
                Constraint::Length(3),  // Arrow/status
                Constraint::Min(20),    // Progress bar
                Constraint::Length(25), // Model name
            ])
            .split(area);
        
        // Stage name with status color
        let name_style = match stage.status {
            StageStatus::Waiting => Style::default().fg(self.theme.waiting_color),
            StageStatus::Running => Style::default()
                .fg(self.theme.running_color)
                .add_modifier(Modifier::BOLD),
            StageStatus::Completed => Style::default()
                .fg(self.theme.completed_color)
                .add_modifier(Modifier::BOLD),
            StageStatus::Error => Style::default()
                .fg(self.theme.error_color)
                .add_modifier(Modifier::BOLD),
        };
        
        frame.render_widget(
            Paragraph::new(stage.name.as_str()).style(name_style),
            layout[0]
        );
        
        // Status indicator
        let status_indicator = match stage.status {
            StageStatus::Waiting => "⏸",
            StageStatus::Running => "▶",
            StageStatus::Completed => "✓",
            StageStatus::Error => "❌",
        };
        
        frame.render_widget(
            Paragraph::new(status_indicator).style(name_style),
            layout[1]
        );
        
        // Progress bar
        self.draw_progress_bar(frame, layout[2], stage);
        
        // Model name
        frame.render_widget(
            Paragraph::new(format!("({})", stage.model))
                .style(Style::default().fg(self.theme.model_color)),
            layout[3]
        );
    }
    
    /// Draw progress bar for stage
    fn draw_progress_bar(&self, frame: &mut Frame, area: Rect, stage: &StageProgress) {
        let progress_color = match stage.status {
            StageStatus::Waiting => self.theme.waiting_color,
            StageStatus::Running => self.theme.running_color,
            StageStatus::Completed => self.theme.completed_color,
            StageStatus::Error => self.theme.error_color,
        };
        
        // Create visual progress bar
        let width = area.width as usize;
        let filled = (stage.progress as usize * width / 100).min(width);
        let empty = width - filled;
        
        let progress_text = if stage.status == StageStatus::Waiting {
            "░".repeat(width)
        } else {
            format!("{}{}", "█".repeat(filled), "░".repeat(empty))
        };
        
        let progress_display = format!("{} {}%", progress_text, stage.progress);
        
        frame.render_widget(
            Paragraph::new(progress_display)
                .style(Style::default().fg(progress_color)),
            area
        );
    }
}

/// Create sample consensus progress for testing
impl ConsensusProgress {
    pub fn sample() -> Self {
        Self {
            generator: StageProgress {
                name: "Generator".to_string(),
                model: "claude-3-5-sonnet".to_string(),
                progress: 75,
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
        }
    }
}