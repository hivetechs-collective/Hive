//! Mode Visualization and Status Indicators
//!
//! Provides visual feedback and status information for mode operations
//! in both CLI and TUI interfaces.

use crate::modes::detector::DetectionResult;
use crate::modes::switcher::SwitchResult;
use crate::planning::ModeType;
use chrono::{DateTime, Utc};
use crossterm::style::{Color, Stylize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mode visualizer for status display
#[derive(Debug)]
pub struct ModeVisualizer {
    current_status: ModeStatus,
    history: Vec<StatusEvent>,
    indicators: HashMap<ModeType, ModeIndicator>,
    animation_state: AnimationState,
}

/// Current mode status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeStatus {
    pub current_mode: ModeType,
    pub confidence: f32,
    pub active_duration: std::time::Duration,
    pub last_switch: Option<DateTime<Utc>>,
    pub context_items: usize,
    pub health: HealthStatus,
}

/// Health status of current mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Excellent,
    Good,
    Warning,
    Critical,
}

/// Status event in history
#[derive(Debug, Clone)]
struct StatusEvent {
    event_type: EventType,
    timestamp: DateTime<Utc>,
    mode: ModeType,
    details: String,
}

/// Type of status event
#[derive(Debug, Clone, PartialEq)]
enum EventType {
    ModeSwitch,
    Detection,
    ContextUpdate,
    HealthChange,
}

/// Visual indicator for a mode
#[derive(Debug, Clone)]
struct ModeIndicator {
    symbol: String,
    color: Color,
    animation: AnimationType,
}

/// Type of animation
#[derive(Debug, Clone, PartialEq)]
enum AnimationType {
    None,
    Pulse,
    Spin,
    Progress,
}

/// Animation state
#[derive(Debug)]
struct AnimationState {
    frame: usize,
    last_update: std::time::Instant,
    active_animations: HashMap<ModeType, AnimationFrame>,
}

/// Animation frame data
#[derive(Debug, Clone)]
struct AnimationFrame {
    animation_type: AnimationType,
    current_frame: usize,
    total_frames: usize,
}

impl ModeVisualizer {
    /// Create a new mode visualizer
    pub fn new() -> Self {
        let mut indicators = HashMap::new();

        // Define mode indicators
        indicators.insert(
            ModeType::Planning,
            ModeIndicator {
                symbol: "📋".to_string(),
                color: Color::Blue,
                animation: AnimationType::None,
            },
        );

        indicators.insert(
            ModeType::Execution,
            ModeIndicator {
                symbol: "⚡".to_string(),
                color: Color::Green,
                animation: AnimationType::Pulse,
            },
        );

        indicators.insert(
            ModeType::Hybrid,
            ModeIndicator {
                symbol: "🔄".to_string(),
                color: Color::Cyan,
                animation: AnimationType::Spin,
            },
        );

        indicators.insert(
            ModeType::Analysis,
            ModeIndicator {
                symbol: "🔍".to_string(),
                color: Color::Yellow,
                animation: AnimationType::None,
            },
        );

        indicators.insert(
            ModeType::Learning,
            ModeIndicator {
                symbol: "🧠".to_string(),
                color: Color::Magenta,
                animation: AnimationType::Progress,
            },
        );

        Self {
            current_status: ModeStatus {
                current_mode: ModeType::Hybrid,
                confidence: 1.0,
                active_duration: std::time::Duration::ZERO,
                last_switch: None,
                context_items: 0,
                health: HealthStatus::Excellent,
            },
            history: Vec::new(),
            indicators,
            animation_state: AnimationState::new(),
        }
    }

    /// Update with detection result
    pub fn update_detection_result(&mut self, result: &DetectionResult) {
        self.add_event(StatusEvent {
            event_type: EventType::Detection,
            timestamp: Utc::now(),
            mode: result.primary_mode.clone(),
            details: format!("Detected with {:.0}% confidence", result.confidence * 100.0),
        });

        // Update confidence
        self.current_status.confidence = result.confidence;
    }

    /// Update with switch result
    pub fn update_switch_result(&mut self, result: &SwitchResult) {
        if result.success {
            self.add_event(StatusEvent {
                event_type: EventType::ModeSwitch,
                timestamp: Utc::now(),
                mode: result.to_mode.clone(),
                details: format!(
                    "Switched from {:?} in {}ms",
                    result.from_mode,
                    result.duration.as_millis()
                ),
            });

            // Update current mode
            self.current_status.current_mode = result.to_mode.clone();
            self.current_status.last_switch = Some(Utc::now());
            self.current_status.active_duration = std::time::Duration::ZERO;

            // Update context items
            self.current_status.context_items = result.context_transformation.items_preserved
                + result.context_transformation.items_transformed;
        }
    }

    /// Get current status
    pub fn get_current_status(&self, mode: &ModeType) -> ModeStatus {
        let mut status = self.current_status.clone();
        status.current_mode = mode.clone();

        // Calculate active duration
        if let Some(last_switch) = status.last_switch {
            status.active_duration = Utc::now()
                .signed_duration_since(last_switch)
                .to_std()
                .unwrap_or_default();
        }

        // Update health based on various factors
        status.health = self.calculate_health(&status);

        status
    }

    /// Reset visualizer
    pub fn reset(&mut self) {
        self.current_status = ModeStatus {
            current_mode: ModeType::Hybrid,
            confidence: 1.0,
            active_duration: std::time::Duration::ZERO,
            last_switch: None,
            context_items: 0,
            health: HealthStatus::Excellent,
        };
        self.history.clear();
    }

    /// Format mode for display
    pub fn format_mode_display(&self, mode: &ModeType, include_animation: bool) -> String {
        let default_indicator = ModeIndicator {
            symbol: "?".to_string(),
            color: Color::White,
            animation: AnimationType::None,
        };
        let indicator = self.indicators.get(mode).unwrap_or(&default_indicator);

        let symbol = if include_animation {
            self.get_animated_symbol(mode, &indicator.symbol)
        } else {
            indicator.symbol.clone()
        };

        format!("{} {:?}", symbol, mode)
    }

    /// Format status line for CLI
    pub fn format_status_line(&self) -> String {
        let mode_display = self.format_mode_display(&self.current_status.current_mode, true);
        let confidence = format!("{}%", (self.current_status.confidence * 100.0) as i32);
        let health = self.format_health(&self.current_status.health);

        format!(
            "Mode: {} | Confidence: {} | Health: {} | Context: {} items",
            mode_display,
            confidence.cyan(),
            health,
            self.current_status.context_items
        )
    }

    /// Format detailed status for TUI
    pub fn format_detailed_status(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // Header
        lines.push("╭─────────────────────────────────╮".to_string());
        lines.push("│       Mode Status               │".to_string());
        lines.push("├─────────────────────────────────┤".to_string());

        // Current mode
        let mode_line = format!(
            "│ Mode: {:<25} │",
            self.format_mode_display(&self.current_status.current_mode, true)
        );
        lines.push(mode_line);

        // Confidence bar
        let confidence_bar = self.format_confidence_bar(self.current_status.confidence);
        lines.push(format!("│ Confidence: {:<19} │", confidence_bar));

        // Duration
        let duration = self.format_duration(self.current_status.active_duration);
        lines.push(format!("│ Active: {:<23} │", duration));

        // Health
        let health = self.format_health(&self.current_status.health);
        lines.push(format!("│ Health: {:<23} │", health));

        // Context
        lines.push(format!(
            "│ Context Items: {:<16} │",
            self.current_status.context_items
        ));

        lines.push("╰─────────────────────────────────╯".to_string());

        // Recent events
        if !self.history.is_empty() {
            lines.push("".to_string());
            lines.push("Recent Events:".to_string());
            for event in self.history.iter().rev().take(5) {
                lines.push(self.format_event(event));
            }
        }

        lines
    }

    /// Get mode transition animation frames
    pub fn get_transition_animation(&self, from: &ModeType, to: &ModeType) -> Vec<String> {
        let from_indicator = self.indicators.get(from);
        let to_indicator = self.indicators.get(to);

        let mut frames = Vec::new();

        // Create transition animation frames
        for i in 0..10 {
            let progress = i as f32 / 9.0;
            let frame = self.create_transition_frame(from_indicator, to_indicator, progress);
            frames.push(frame);
        }

        frames
    }

    // Private helper methods

    fn add_event(&mut self, event: StatusEvent) {
        self.history.push(event);

        // Keep only recent history
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    fn calculate_health(&self, status: &ModeStatus) -> HealthStatus {
        // Simple health calculation based on various factors
        let mut score = 100.0;

        // Confidence affects health
        if status.confidence < 0.5 {
            score -= 30.0;
        } else if status.confidence < 0.7 {
            score -= 15.0;
        }

        // Long duration in same mode might need attention
        if status.active_duration > std::time::Duration::from_secs(3600) {
            score -= 10.0;
        }

        // Low context items might indicate issues
        if status.context_items == 0 && status.active_duration > std::time::Duration::from_secs(60)
        {
            score -= 20.0;
        }

        match score as i32 {
            90..=100 => HealthStatus::Excellent,
            70..=89 => HealthStatus::Good,
            50..=69 => HealthStatus::Warning,
            _ => HealthStatus::Critical,
        }
    }

    fn get_animated_symbol(&self, mode: &ModeType, base_symbol: &str) -> String {
        if let Some(indicator) = self.indicators.get(mode) {
            match indicator.animation {
                AnimationType::Spin => {
                    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                    let frame_idx = (self.animation_state.frame / 2) % frames.len();
                    frames[frame_idx].to_string()
                }
                AnimationType::Pulse => {
                    if (self.animation_state.frame / 5) % 2 == 0 {
                        format!("{}", base_symbol.bold())
                    } else {
                        base_symbol.to_string()
                    }
                }
                AnimationType::Progress => {
                    let progress = (self.animation_state.frame % 20) as f32 / 20.0;
                    let filled = (progress * 5.0) as usize;
                    let bar = "█".repeat(filled) + &"░".repeat(5 - filled);
                    format!("{} [{}]", base_symbol, bar)
                }
                AnimationType::None => base_symbol.to_string(),
            }
        } else {
            base_symbol.to_string()
        }
    }

    fn format_confidence_bar(&self, confidence: f32) -> String {
        let width = 15;
        let filled = (confidence * width as f32) as usize;
        let bar = "█".repeat(filled) + &"░".repeat(width - filled);

        let color = if confidence > 0.8 {
            Color::Green
        } else if confidence > 0.5 {
            Color::Yellow
        } else {
            Color::Red
        };

        format!("[{}] {:.0}%", bar.with(color), confidence * 100.0)
    }

    fn format_duration(&self, duration: std::time::Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }

    fn format_health(&self, health: &HealthStatus) -> String {
        match health {
            HealthStatus::Excellent => "Excellent".green().to_string(),
            HealthStatus::Good => "Good".cyan().to_string(),
            HealthStatus::Warning => "Warning".yellow().to_string(),
            HealthStatus::Critical => "Critical".red().to_string(),
        }
    }

    fn format_event(&self, event: &StatusEvent) -> String {
        let time = event.timestamp.format("%H:%M:%S");
        let icon = match event.event_type {
            EventType::ModeSwitch => "→",
            EventType::Detection => "◎",
            EventType::ContextUpdate => "↻",
            EventType::HealthChange => "♥",
        };

        format!("  {} {} {}", time, icon, event.details)
    }

    fn create_transition_frame(
        &self,
        from: Option<&ModeIndicator>,
        to: Option<&ModeIndicator>,
        progress: f32,
    ) -> String {
        let default_symbol = "?".to_string();
        let from_symbol = from.map(|i| &i.symbol).unwrap_or(&default_symbol);
        let to_symbol = to.map(|i| &i.symbol).unwrap_or(&default_symbol);

        if progress < 0.3 {
            from_symbol.to_string()
        } else if progress < 0.7 {
            "→".to_string()
        } else {
            to_symbol.to_string()
        }
    }
}

impl AnimationState {
    fn new() -> Self {
        Self {
            frame: 0,
            last_update: std::time::Instant::now(),
            active_animations: HashMap::new(),
        }
    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_update) > std::time::Duration::from_millis(100) {
            self.frame += 1;
            self.last_update = now;

            // Update active animations
            for (_, anim) in self.active_animations.iter_mut() {
                anim.current_frame = (anim.current_frame + 1) % anim.total_frames;
            }
        }
    }
}

/// Visual style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub enable_animations: bool,
    pub show_confidence_bar: bool,
    pub show_health_status: bool,
    pub history_size: usize,
    pub update_interval: std::time::Duration,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            enable_animations: true,
            show_confidence_bar: true,
            show_health_status: true,
            history_size: 100,
            update_interval: std::time::Duration::from_millis(100),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_creation() {
        let visualizer = ModeVisualizer::new();
        assert_eq!(visualizer.current_status.current_mode, ModeType::Hybrid);
    }

    #[test]
    fn test_mode_formatting() {
        let visualizer = ModeVisualizer::new();
        let formatted = visualizer.format_mode_display(&ModeType::Planning, false);
        assert!(formatted.contains("Planning"));
    }

    #[test]
    fn test_status_update() {
        let mut visualizer = ModeVisualizer::new();

        let result = DetectionResult {
            primary_mode: ModeType::Execution,
            confidence: 0.85,
            scores: HashMap::new(),
            reasoning: vec![],
            alternatives: vec![],
            consensus_insights: crate::modes::detector::ConsensusInsights {
                task_complexity: "medium".to_string(),
                recommended_approach: "test".to_string(),
                potential_challenges: vec![],
                success_factors: vec![],
            },
            preference_influence: 0.3,
        };

        visualizer.update_detection_result(&result);
        assert_eq!(visualizer.current_status.confidence, 0.85);
    }

    #[test]
    fn test_health_calculation() {
        let visualizer = ModeVisualizer::new();

        let mut status = ModeStatus {
            current_mode: ModeType::Planning,
            confidence: 0.9,
            active_duration: std::time::Duration::from_secs(300),
            last_switch: Some(Utc::now()),
            context_items: 5,
            health: HealthStatus::Excellent,
        };

        let health = visualizer.calculate_health(&status);
        assert_eq!(health, HealthStatus::Excellent);

        status.confidence = 0.4;
        let health = visualizer.calculate_health(&status);
        assert_eq!(health, HealthStatus::Good);
    }
}
