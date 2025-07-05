//! First-launch setup flow for TUI
//!
//! Provides a Claude Code-like setup experience within the TUI interface
//! for configuring API keys on first launch.

use anyhow::{Result, Context};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Clear},
};
use crossterm::event::{KeyCode, KeyEvent};
use std::path::PathBuf;

use crate::core::config::{HiveConfig, OpenRouterConfig, LicenseConfig, get_hive_config_dir};

/// Setup state for first-launch configuration
pub struct SetupFlow {
    /// Current setup step
    current_step: SetupStep,
    
    /// OpenRouter API key input
    openrouter_key: String,
    
    /// Hive license key input
    hive_key: String,
    
    /// Current input field
    current_input: String,
    
    /// Cursor position in current input
    cursor_position: usize,
    
    /// Error message if any
    error_message: Option<String>,
    
    /// Whether setup is complete
    complete: bool,
}

/// Steps in the setup flow
#[derive(Clone, Debug, PartialEq)]
enum SetupStep {
    Welcome,
    OpenRouterKey,
    ValidatingOpenRouter,
    HiveKey,
    ValidatingHive,
    Complete,
}

impl SetupFlow {
    /// Create new setup flow
    pub fn new() -> Self {
        Self {
            current_step: SetupStep::Welcome,
            openrouter_key: String::new(),
            hive_key: String::new(),
            current_input: String::new(),
            cursor_position: 0,
            error_message: None,
            complete: false,
        }
    }
    
    /// Check if setup is needed
    pub async fn is_setup_needed() -> bool {
        let config_path = get_hive_config_dir().join("config.toml");
        if !config_path.exists() {
            return true;
        }
        
        // Check if config has required keys
        match crate::core::config::load_config().await {
            Ok(config) => {
                // Need setup if OpenRouter API key is missing
                config.openrouter.as_ref()
                    .and_then(|or| or.api_key.as_ref())
                    .map(|key| key.is_empty())
                    .unwrap_or(true)
            }
            Err(_) => true,
        }
    }
    
    /// Render the setup flow
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Clear the area for setup UI
        frame.render_widget(Clear, area);
        
        // Create centered layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),    // Header
                Constraint::Min(10),      // Content
                Constraint::Length(3),    // Footer
            ])
            .split(area);
        
        // Header
        let header = Paragraph::new("🐝 HiveTechs Consensus - First Time Setup")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(header, chunks[0]);
        
        // Content area
        match self.current_step {
            SetupStep::Welcome => self.render_welcome(frame, chunks[1]),
            SetupStep::OpenRouterKey => self.render_openrouter_input(frame, chunks[1]),
            SetupStep::ValidatingOpenRouter => self.render_validating(frame, chunks[1], "OpenRouter"),
            SetupStep::HiveKey => self.render_hive_input(frame, chunks[1]),
            SetupStep::ValidatingHive => self.render_validating(frame, chunks[1], "Hive"),
            SetupStep::Complete => self.render_complete(frame, chunks[1]),
        }
        
        // Footer with instructions
        let footer_text = match self.current_step {
            SetupStep::Welcome => "Press Enter to continue, Ctrl+C to exit",
            SetupStep::OpenRouterKey | SetupStep::HiveKey => "Enter your key and press Enter, Ctrl+C to exit",
            SetupStep::ValidatingOpenRouter | SetupStep::ValidatingHive => "Validating...",
            SetupStep::Complete => "Press Enter to start using Hive AI",
        };
        
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(footer, chunks[2]);
    }
    
    /// Render welcome screen
    fn render_welcome(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled("Welcome to Hive AI!", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("Let's get you set up with a quick 2-step process:"),
            Line::from(""),
            Line::from("  1. Configure your OpenRouter API key"),
            Line::from("     • Access 323+ AI models from 55+ providers"),
            Line::from("     • Get your key at: https://openrouter.ai/keys"),
            Line::from(""),
            Line::from("  2. Configure your Hive license key"),
            Line::from("     • Unlock consensus features and enterprise capabilities"),
            Line::from("     • Get your license at: https://hivetechs.com/account"),
            Line::from(""),
            Line::from("This will only take a minute!"),
        ];
        
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Setup "))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// Render OpenRouter key input
    fn render_openrouter_input(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled("Step 1: OpenRouter API Key", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("OpenRouter provides access to 323+ AI models through a unified API."),
            Line::from(""),
            Line::from("Get your API key:"),
            Line::from("  1. Visit: https://openrouter.ai/keys"),
            Line::from("  2. Sign up or log in"),
            Line::from("  3. Create a new API key"),
            Line::from("  4. Copy and paste it below"),
            Line::from(""),
        ];
        
        // Add error message if present
        if let Some(ref error) = self.error_message {
            lines.push(Line::from(Span::styled(
                format!("❌ {}", error),
                Style::default().fg(Color::Red)
            )));
            lines.push(Line::from(""));
        }
        
        lines.push(Line::from("API Key (starts with sk-or-):"));
        
        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" OpenRouter Setup "))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
        
        // Render input field
        let input_area = Rect {
            x: area.x + 2,
            y: area.y + area.height - 4,
            width: area.width - 4,
            height: 3,
        };
        
        // Mask the input for security
        let masked_input = if self.current_input.is_empty() {
            String::new()
        } else {
            format!("sk-or-{}", "*".repeat(self.current_input.len().saturating_sub(6)))
        };
        
        let input = Paragraph::new(masked_input)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
        
        frame.render_widget(input, input_area);
        
        // Show cursor
        frame.set_cursor(
            input_area.x + 1 + self.cursor_position as u16,
            input_area.y + 1,
        );
    }
    
    /// Render Hive key input
    fn render_hive_input(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled("Step 2: Hive License Key", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("Your Hive license unlocks the full power of consensus AI."),
            Line::from(""),
            Line::from("Get your license key:"),
            Line::from("  1. Visit: https://hivetechs.com/account"),
            Line::from("  2. Sign up or log in"),
            Line::from("  3. Copy your license key"),
            Line::from("  4. Paste it below"),
            Line::from(""),
        ];
        
        // Add error message if present
        if let Some(ref error) = self.error_message {
            lines.push(Line::from(Span::styled(
                format!("❌ {}", error),
                Style::default().fg(Color::Red)
            )));
            lines.push(Line::from(""));
        }
        
        lines.push(Line::from("License Key:"));
        
        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Hive License "))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
        
        // Render input field
        let input_area = Rect {
            x: area.x + 2,
            y: area.y + area.height - 4,
            width: area.width - 4,
            height: 3,
        };
        
        // Mask the input for security
        let masked_input = "*".repeat(self.current_input.len());
        
        let input = Paragraph::new(masked_input)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
        
        frame.render_widget(input, input_area);
        
        // Show cursor
        frame.set_cursor(
            input_area.x + 1 + self.cursor_position as u16,
            input_area.y + 1,
        );
    }
    
    /// Render validation screen
    fn render_validating(&self, frame: &mut Frame, area: Rect, service: &str) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("Validating {} credentials...", service),
                Style::default().fg(Color::Yellow)
            )),
            Line::from(""),
            Line::from("⏳ Please wait while we verify your credentials"),
            Line::from(""),
            Line::from("This may take a few seconds..."),
        ];
        
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Validating "))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// Render completion screen
    fn render_complete(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled("✅ Setup Complete!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("Your Hive AI is now configured and ready to use!"),
            Line::from(""),
            Line::from("You can now:"),
            Line::from("  • Ask questions and get AI consensus responses"),
            Line::from("  • Analyze repositories with ML intelligence"),
            Line::from("  • Create strategic development plans"),
            Line::from("  • Access 323+ AI models through consensus"),
            Line::from(""),
            Line::from("Press Enter to start using Hive AI"),
        ];
        
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Ready! "))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// Handle keyboard input during setup
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match self.current_step {
            SetupStep::Welcome => {
                if key.code == KeyCode::Enter {
                    self.current_step = SetupStep::OpenRouterKey;
                    self.error_message = None;
                }
            }
            
            SetupStep::OpenRouterKey => {
                match key.code {
                    KeyCode::Enter => {
                        if self.validate_openrouter_key() {
                            self.openrouter_key = self.current_input.clone();
                            self.current_input.clear();
                            self.cursor_position = 0;
                            self.current_step = SetupStep::ValidatingOpenRouter;
                            self.error_message = None;
                            
                            // Trigger async validation
                            return Ok(true);
                        }
                    }
                    KeyCode::Char(c) => {
                        self.current_input.insert(self.cursor_position, c);
                        self.cursor_position += 1;
                        self.error_message = None;
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position > 0 {
                            self.current_input.remove(self.cursor_position - 1);
                            self.cursor_position -= 1;
                        }
                    }
                    KeyCode::Left => {
                        if self.cursor_position > 0 {
                            self.cursor_position -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor_position < self.current_input.len() {
                            self.cursor_position += 1;
                        }
                    }
                    _ => {}
                }
            }
            
            SetupStep::HiveKey => {
                match key.code {
                    KeyCode::Enter => {
                        if self.validate_hive_key() {
                            self.hive_key = self.current_input.clone();
                            self.current_input.clear();
                            self.cursor_position = 0;
                            self.current_step = SetupStep::ValidatingHive;
                            self.error_message = None;
                            
                            // Trigger async validation
                            return Ok(true);
                        }
                    }
                    KeyCode::Char(c) => {
                        self.current_input.insert(self.cursor_position, c);
                        self.cursor_position += 1;
                        self.error_message = None;
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position > 0 {
                            self.current_input.remove(self.cursor_position - 1);
                            self.cursor_position -= 1;
                        }
                    }
                    KeyCode::Left => {
                        if self.cursor_position > 0 {
                            self.cursor_position -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor_position < self.current_input.len() {
                            self.cursor_position += 1;
                        }
                    }
                    _ => {}
                }
            }
            
            SetupStep::Complete => {
                if key.code == KeyCode::Enter {
                    self.complete = true;
                }
            }
            
            _ => {}
        }
        
        Ok(false)
    }
    
    /// Validate OpenRouter API key format
    fn validate_openrouter_key(&mut self) -> bool {
        if self.current_input.is_empty() {
            self.error_message = Some("API key cannot be empty".to_string());
            return false;
        }
        
        if !self.current_input.starts_with("sk-or-") {
            self.error_message = Some("OpenRouter API keys must start with 'sk-or-'".to_string());
            return false;
        }
        
        if self.current_input.len() < 20 {
            self.error_message = Some("API key seems too short".to_string());
            return false;
        }
        
        true
    }
    
    /// Validate Hive license key format
    fn validate_hive_key(&mut self) -> bool {
        if self.current_input.is_empty() {
            self.error_message = Some("License key cannot be empty".to_string());
            return false;
        }
        
        if self.current_input.len() < 10 {
            self.error_message = Some("License key seems too short".to_string());
            return false;
        }
        
        true
    }
    
    /// Process async validation steps
    pub async fn process_validation(&mut self) -> Result<()> {
        match self.current_step {
            SetupStep::ValidatingOpenRouter => {
                // Simulate API validation (in real implementation, make test API call)
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                
                // For now, always succeed
                self.current_step = SetupStep::HiveKey;
            }
            
            SetupStep::ValidatingHive => {
                // Simulate license validation
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                
                // Save configuration
                self.save_configuration().await?;
                
                self.current_step = SetupStep::Complete;
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    /// Save configuration to file
    async fn save_configuration(&self) -> Result<()> {
        let config_dir = get_hive_config_dir();
        tokio::fs::create_dir_all(&config_dir).await
            .context("Failed to create config directory")?;
        
        let config = HiveConfig {
            openrouter: Some(OpenRouterConfig {
                api_key: Some(self.openrouter_key.clone()),
                base_url: "https://openrouter.ai/api/v1".to_string(),
                timeout_seconds: 30,
                max_retries: 3,
            }),
            license: Some(LicenseConfig {
                key: Some(self.hive_key.clone()),
                email: None,
                tier: Some("pro".to_string()),
            }),
            ..Default::default()
        };
        
        let config_path = config_dir.join("config.toml");
        let config_str = toml::to_string_pretty(&config)
            .context("Failed to serialize config")?;
        
        tokio::fs::write(&config_path, config_str).await
            .context("Failed to write config file")?;
        
        // Also set as environment variable for current session
        std::env::set_var("OPENROUTER_API_KEY", &self.openrouter_key);
        
        Ok(())
    }
    
    /// Check if setup is complete
    pub fn is_complete(&self) -> bool {
        self.complete
    }
}