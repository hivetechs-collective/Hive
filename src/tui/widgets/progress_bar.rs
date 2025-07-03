//! Enhanced Progress Bar Widget for Consensus Display

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Widget},
};

use super::WidgetTheme;

/// Enhanced progress bar for consensus stages
pub struct ConsensusProgressBar<'a> {
    /// Progress percentage (0-100)
    percent: u16,
    
    /// Progress bar label
    label: Option<&'a str>,
    
    /// Current style
    style: Style,
    
    /// Progress bar style
    gauge_style: Style,
    
    /// Block around the progress bar
    block: Option<Block<'a>>,
    
    /// Use custom characters for progress
    use_unicode: bool,
}

impl<'a> ConsensusProgressBar<'a> {
    /// Create new progress bar
    pub fn new() -> Self {
        Self {
            percent: 0,
            label: None,
            style: Style::default(),
            gauge_style: Style::default(),
            block: None,
            use_unicode: true,
        }
    }
    
    /// Set progress percentage
    pub fn percent(mut self, percent: u16) -> Self {
        self.percent = percent.min(100);
        self
    }
    
    /// Set progress label
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
    
    /// Set overall style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    
    /// Set progress bar style
    pub fn gauge_style(mut self, style: Style) -> Self {
        self.gauge_style = style;
        self
    }
    
    /// Set block around progress bar
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Enable/disable unicode characters
    pub fn use_unicode(mut self, enable: bool) -> Self {
        self.use_unicode = enable;
        self
    }
    
    /// Create progress bar for consensus stage
    pub fn for_consensus_stage(
        percent: u16,
        label: &'a str,
        status_color: Color,
        theme: &WidgetTheme,
    ) -> Self {
        Self::new()
            .percent(percent)
            .label(label)
            .style(theme.text_style())
            .gauge_style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
            .use_unicode(true)
    }
}

impl<'a> Widget for ConsensusProgressBar<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        
        if area.height < 1 {
            return;
        }
        
        // Calculate filled and empty sections
        let ratio = f64::from(self.percent) / 100.0;
        let filled_width = (f64::from(area.width) * ratio).round() as u16;
        let empty_width = area.width - filled_width;
        
        // Choose characters based on unicode support
        let (filled_char, empty_char) = if self.use_unicode {
            ("█", "░")
        } else {
            ("#", "-")
        };
        
        // Build progress bar string
        let progress_bar = format!(
            "{}{}",
            filled_char.repeat(filled_width as usize),
            empty_char.repeat(empty_width as usize)
        );
        
        // Add percentage display
        let percent_text = format!(" {}%", self.percent);
        let display_text = if let Some(label) = self.label {
            format!("{}: {}{}", label, progress_bar, percent_text)
        } else {
            format!("{}{}", progress_bar, percent_text)
        };
        
        // Render the progress bar
        buf.set_string(
            area.x,
            area.y,
            &display_text,
            self.gauge_style.patch(self.style),
        );
    }
}

/// Predefined consensus progress bar styles
impl<'a> ConsensusProgressBar<'a> {
    /// Style for waiting stage
    pub fn waiting_style(theme: &WidgetTheme) -> Style {
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM)
    }
    
    /// Style for running stage
    pub fn running_style(theme: &WidgetTheme) -> Style {
        Style::default()
            .fg(theme.warning)
            .add_modifier(Modifier::BOLD)
    }
    
    /// Style for completed stage
    pub fn completed_style(theme: &WidgetTheme) -> Style {
        Style::default()
            .fg(theme.success)
            .add_modifier(Modifier::BOLD)
    }
    
    /// Style for error stage
    pub fn error_style(theme: &WidgetTheme) -> Style {
        Style::default()
            .fg(theme.error)
            .add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK)
    }
}