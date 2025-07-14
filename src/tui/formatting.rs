// Advanced formatting utilities for beautiful TUI display
// Provides syntax highlighting, box drawing, and visual enhancements

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

/// Enhanced formatter for different content types
pub struct ContentFormatter;

impl ContentFormatter {
    /// Format content with syntax highlighting and visual enhancements
    pub fn format_with_highlighting(content: &str, content_type: ContentType) -> Text {
        match content_type {
            ContentType::ExecutiveSummary => Self::format_executive_summary(content),
            ContentType::CodeBlock => Self::format_code_block(content),
            ContentType::PerformanceMetrics => Self::format_performance_metrics(content),
            ContentType::CostBreakdown => Self::format_cost_breakdown(content),
            ContentType::StageJourney => Self::format_stage_journey(content),
            ContentType::Regular => Self::format_regular_text(content),
        }
    }

    /// Format executive summary with enhanced styling
    fn format_executive_summary(content: &str) -> Text {
        let lines: Vec<Line> = content
            .lines()
            .map(|line| {
                if line.contains("╔")
                    || line.contains("╗")
                    || line.contains("╚")
                    || line.contains("╝")
                {
                    // Box drawing characters
                    Line::from(vec![Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )])
                } else if line.contains("║") {
                    // Box content
                    if line.contains("EXECUTIVE SUMMARY") {
                        Line::from(vec![Span::styled(
                            line,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )])
                    } else if line.contains("📌") {
                        // Key points header
                        Line::from(vec![Span::styled(
                            line,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )])
                    } else if line.contains("🎯") {
                        // Action items header
                        Line::from(vec![Span::styled(
                            line,
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )])
                    } else if line.contains("• ") {
                        // Bullet points
                        Line::from(vec![
                            Span::styled(&line[..3], Style::default().fg(Color::Cyan)), // Box chars
                            Span::styled("  • ", Style::default().fg(Color::Yellow)),
                            Span::styled(&line[6..], Style::default().fg(Color::White)),
                        ])
                    } else if line.contains("✓ ") {
                        // Action items
                        Line::from(vec![
                            Span::styled(&line[..3], Style::default().fg(Color::Cyan)), // Box chars
                            Span::styled("  ✓ ", Style::default().fg(Color::Green)),
                            Span::styled(&line[6..], Style::default().fg(Color::White)),
                        ])
                    } else {
                        Line::from(vec![Span::styled(line, Style::default().fg(Color::Cyan))])
                    }
                } else {
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::White))])
                }
            })
            .collect();

        Text::from(lines)
    }

    /// Format code blocks with syntax highlighting
    fn format_code_block(content: &str) -> Text {
        let lines: Vec<Line> = content
            .lines()
            .map(|line| {
                if line.starts_with("```") {
                    Line::from(vec![Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    )])
                } else if line.trim().starts_with("//") || line.trim().starts_with("#") {
                    // Comments
                    Line::from(vec![Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    )])
                } else if line.contains("fn ")
                    || line.contains("function ")
                    || line.contains("def ")
                {
                    // Function definitions
                    Line::from(vec![Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    )])
                } else if line.contains("let ") || line.contains("const ") || line.contains("var ")
                {
                    // Variable declarations
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::Green))])
                } else {
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::White))])
                }
            })
            .collect();

        Text::from(lines)
    }

    /// Format performance metrics with visual bars
    fn format_performance_metrics(content: &str) -> Text {
        let lines: Vec<Line> = content
            .lines()
            .map(|line| {
                if line.contains("┌")
                    || line.contains("┐")
                    || line.contains("└")
                    || line.contains("┘")
                    || line.contains("├")
                    || line.contains("┤")
                    || line.contains("┼")
                {
                    // Table borders
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::Cyan))])
                } else if line.contains("PERFORMANCE METRICS") {
                    Line::from(vec![Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )])
                } else if line.contains("COST BREAKDOWN") {
                    Line::from(vec![Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )])
                } else if line.contains("Duration:")
                    || line.contains("Tokens:")
                    || line.contains("Cost:")
                {
                    // Metrics with values
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        Line::from(vec![
                            Span::styled(parts[0], Style::default().fg(Color::Cyan)),
                            Span::styled(":", Style::default().fg(Color::White)),
                            Span::styled(
                                parts[1],
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ])
                    } else {
                        Line::from(vec![Span::styled(line, Style::default().fg(Color::White))])
                    }
                } else if line.contains("█") || line.contains("░") {
                    // Progress bars
                    let chars: Vec<char> = line.chars().collect();
                    let spans: Vec<Span> = chars
                        .iter()
                        .map(|c| match c {
                            '█' => Span::styled(c.to_string(), Style::default().fg(Color::Green)),
                            '░' => {
                                Span::styled(c.to_string(), Style::default().fg(Color::DarkGray))
                            }
                            _ => Span::styled(c.to_string(), Style::default().fg(Color::White)),
                        })
                        .collect();
                    Line::from(spans)
                } else {
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::White))])
                }
            })
            .collect();

        Text::from(lines)
    }

    /// Format cost breakdown with currency highlighting
    fn format_cost_breakdown(content: &str) -> Text {
        let lines: Vec<Line> = content
            .lines()
            .map(|line| {
                if line.contains("$") {
                    // Highlight currency values
                    let parts: Vec<&str> = line.split('$').collect();
                    if parts.len() >= 2 {
                        Line::from(vec![
                            Span::styled(parts[0], Style::default().fg(Color::White)),
                            Span::styled(
                                "$",
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                parts[1],
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ])
                    } else {
                        Line::from(vec![Span::styled(line, Style::default().fg(Color::White))])
                    }
                } else {
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::White))])
                }
            })
            .collect();

        Text::from(lines)
    }

    /// Format stage journey with status indicators
    fn format_stage_journey(content: &str) -> Text {
        let lines: Vec<Line> = content
            .lines()
            .map(|line| {
                if line.contains("🚀") {
                    // Journey header
                    Line::from(vec![Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    )])
                } else if line.contains("═") {
                    // Journey separator
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::Cyan))])
                } else if line.contains("✅") {
                    // Complete status
                    Line::from(vec![
                        Span::styled("✅", Style::default().fg(Color::Green)),
                        Span::styled(&line[2..], Style::default().fg(Color::White)),
                    ])
                } else if line.contains("⚡") {
                    // Optimized status
                    Line::from(vec![
                        Span::styled("⚡", Style::default().fg(Color::Yellow)),
                        Span::styled(&line[2..], Style::default().fg(Color::White)),
                    ])
                } else if line.contains("✨") {
                    // Enhanced status
                    Line::from(vec![
                        Span::styled("✨", Style::default().fg(Color::Magenta)),
                        Span::styled(&line[2..], Style::default().fg(Color::White)),
                    ])
                } else if line.contains("→") {
                    // Arrows
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::Cyan))])
                } else {
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::White))])
                }
            })
            .collect();

        Text::from(lines)
    }

    /// Format regular text with emoji and icon highlighting
    fn format_regular_text(content: &str) -> Text {
        let lines: Vec<Line> = content
            .lines()
            .map(|line| {
                if line.contains("## ") {
                    // Headers with icons
                    if line.contains("📋")
                        || line.contains("📌")
                        || line.contains("🔍")
                        || line.contains("💻")
                        || line.contains("💡")
                        || line.contains("📊")
                        || line.contains("🔒")
                        || line.contains("⚠️")
                        || line.contains("❌")
                        || line.contains("✅")
                        || line.contains("🎯")
                    {
                        Line::from(vec![Span::styled(
                            line,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )])
                    } else {
                        Line::from(vec![Span::styled(
                            line,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )])
                    }
                } else if line.contains("═") {
                    // Separator lines
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::Cyan))])
                } else if line.starts_with("```") {
                    // Code block delimiters
                    Line::from(vec![Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    )])
                } else {
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::White))])
                }
            })
            .collect();

        Text::from(lines)
    }
}

/// Content type for formatting decisions
#[derive(Debug, Clone, Copy)]
pub enum ContentType {
    ExecutiveSummary,
    CodeBlock,
    PerformanceMetrics,
    CostBreakdown,
    StageJourney,
    Regular,
}

impl ContentType {
    /// Detect content type from text
    pub fn detect(content: &str) -> Self {
        if content.contains("EXECUTIVE SUMMARY") {
            ContentType::ExecutiveSummary
        } else if content.contains("PERFORMANCE METRICS") || content.contains("COST BREAKDOWN") {
            ContentType::PerformanceMetrics
        } else if content.contains("Consensus Journey") || content.contains("🚀") {
            ContentType::StageJourney
        } else if content.contains("```") {
            ContentType::CodeBlock
        } else {
            ContentType::Regular
        }
    }
}
