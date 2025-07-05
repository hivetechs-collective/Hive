// Beautiful formatted consensus results for TUI display
// Provides rich visual presentation of curator stage output

use chrono::{DateTime, Utc};
use std::fmt;

/// Formatted consensus result with rich visual presentation
#[derive(Debug, Clone)]
pub struct FormattedConsensusResult {
    /// Executive summary box
    pub executive_summary: ExecutiveSummary,
    /// Detailed findings with sections
    pub detailed_findings: Vec<FindingSection>,
    /// Performance metrics visualization
    pub performance_metrics: PerformanceMetrics,
    /// Cost breakdown table
    pub cost_breakdown: CostBreakdown,
    /// Confidence score with visual indicator
    pub confidence: ConfidenceScore,
    /// 4-stage journey visualization
    pub stage_journey: StageJourney,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Executive summary box
#[derive(Debug, Clone)]
pub struct ExecutiveSummary {
    pub title: String,
    pub key_points: Vec<String>,
    pub action_items: Vec<String>,
}

/// Finding section with formatting
#[derive(Debug, Clone)]
pub struct FindingSection {
    pub title: String,
    pub icon: &'static str,
    pub content: String,
    pub emphasis: EmphasisLevel,
}

/// Emphasis level for sections
#[derive(Debug, Clone, Copy)]
pub enum EmphasisLevel {
    Normal,
    Important,
    Critical,
}

/// Performance metrics with visual bars
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_duration: f64,
    pub stage_timings: Vec<(String, f64)>,
    pub tokens_used: u32,
    pub models_accessed: u32,
}

/// Cost breakdown with formatting
#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub total_cost: f64,
    pub stage_costs: Vec<(String, f64)>,
    pub cost_per_token: f64,
}

/// Confidence score with visual representation
#[derive(Debug, Clone)]
pub struct ConfidenceScore {
    pub score: f64, // 0.0 to 1.0
    pub factors: Vec<(String, f64)>,
}

/// 4-stage journey visualization
#[derive(Debug, Clone)]
pub struct StageJourney {
    pub stages: Vec<StageInfo>,
}

/// Information about each stage
#[derive(Debug, Clone)]
pub struct StageInfo {
    pub name: String,
    pub model: String,
    pub tokens: u32,
    pub duration: f64,
    pub status: StageStatus,
}

/// Stage completion status
#[derive(Debug, Clone, Copy)]
pub enum StageStatus {
    Complete,
    Optimized,
    Enhanced,
}

impl FormattedConsensusResult {
    /// Create a new formatted result from raw curator output
    pub fn from_curator_output(
        content: &str,
        metadata: crate::consensus::types::ResponseMetadata,
        stages: Vec<crate::consensus::types::StageResult>,
    ) -> Self {
        // Parse content to extract summary and sections
        let (executive_summary, detailed_findings) = Self::parse_curator_content(content);
        
        // Calculate performance metrics
        let performance_metrics = Self::calculate_performance_metrics(&metadata, &stages);
        
        // Calculate cost breakdown
        let cost_breakdown = Self::calculate_cost_breakdown(&metadata, &stages);
        
        // Calculate confidence score
        let confidence = Self::calculate_confidence(&stages);
        
        // Build stage journey
        let stage_journey = Self::build_stage_journey(&stages);
        
        Self {
            executive_summary,
            detailed_findings,
            performance_metrics,
            cost_breakdown,
            confidence,
            stage_journey,
            timestamp: Utc::now(),
        }
    }
    
    /// Parse curator content into structured sections
    fn parse_curator_content(content: &str) -> (ExecutiveSummary, Vec<FindingSection>) {
        let mut executive_summary = ExecutiveSummary {
            title: "Executive Summary".to_string(),
            key_points: Vec::new(),
            action_items: Vec::new(),
        };
        
        let mut detailed_findings = Vec::new();
        
        // Extract summary if present
        if let Some(summary_start) = content.find("## Summary") {
            if let Some(summary_end) = content[summary_start..].find("\n## ") {
                let summary_content = &content[summary_start..summary_start + summary_end];
                executive_summary.key_points = Self::extract_bullet_points(summary_content);
            }
        }
        
        // Extract key sections
        let sections = content.split("\n## ").skip(1);
        for section in sections {
            if let Some(newline_pos) = section.find('\n') {
                let title = section[..newline_pos].trim().to_string();
                let content = section[newline_pos + 1..].trim().to_string();
                
                let (icon, emphasis) = Self::determine_section_style(&title, &content);
                
                detailed_findings.push(FindingSection {
                    title,
                    icon,
                    content,
                    emphasis,
                });
            }
        }
        
        // Extract action items
        if content.contains("next steps") || content.contains("recommend") {
            executive_summary.action_items = Self::extract_action_items(content);
        }
        
        (executive_summary, detailed_findings)
    }
    
    /// Extract bullet points from text
    fn extract_bullet_points(text: &str) -> Vec<String> {
        text.lines()
            .filter(|line| line.trim().starts_with('-') || line.trim().starts_with('•'))
            .map(|line| line.trim_start_matches(&['-', '•', ' ']).to_string())
            .collect()
    }
    
    /// Extract action items from content
    fn extract_action_items(content: &str) -> Vec<String> {
        let mut items = Vec::new();
        
        for line in content.lines() {
            let lower = line.to_lowercase();
            if (lower.contains("you can") || lower.contains("you should") || 
                lower.contains("recommend") || lower.contains("suggest")) &&
                (line.contains('-') || line.contains('•')) {
                items.push(line.trim_start_matches(&['-', '•', ' ']).to_string());
            }
        }
        
        items
    }
    
    /// Determine section icon and emphasis based on content
    fn determine_section_style(title: &str, content: &str) -> (&'static str, EmphasisLevel) {
        let title_lower = title.to_lowercase();
        let content_lower = content.to_lowercase();
        
        if title_lower.contains("error") || content_lower.contains("critical") {
            ("⚠️", EmphasisLevel::Critical)
        } else if title_lower.contains("important") || title_lower.contains("key") {
            ("💡", EmphasisLevel::Important)
        } else if title_lower.contains("code") || title_lower.contains("implementation") {
            ("💻", EmphasisLevel::Normal)
        } else if title_lower.contains("performance") {
            ("📊", EmphasisLevel::Normal)
        } else if title_lower.contains("security") {
            ("🔒", EmphasisLevel::Important)
        } else if title_lower.contains("recommendation") || title_lower.contains("suggest") {
            ("🎯", EmphasisLevel::Important)
        } else {
            ("📝", EmphasisLevel::Normal)
        }
    }
    
    /// Calculate performance metrics
    fn calculate_performance_metrics(
        metadata: &crate::consensus::types::ResponseMetadata,
        stages: &[crate::consensus::types::StageResult],
    ) -> PerformanceMetrics {
        let stage_timings: Vec<(String, f64)> = stages.iter()
            .filter_map(|s| s.analytics.as_ref())
            .map(|a| (a.provider.clone(), a.duration))
            .collect();
        
        PerformanceMetrics {
            total_duration: metadata.duration_ms as f64 / 1000.0,
            stage_timings,
            tokens_used: metadata.total_tokens,
            models_accessed: metadata.models_used.len() as u32,
        }
    }
    
    /// Calculate cost breakdown
    fn calculate_cost_breakdown(
        metadata: &crate::consensus::types::ResponseMetadata,
        stages: &[crate::consensus::types::StageResult],
    ) -> CostBreakdown {
        let stage_costs: Vec<(String, f64)> = stages.iter()
            .filter_map(|s| s.analytics.as_ref().map(|a| (s.stage_name.clone(), a.cost)))
            .collect();
        
        let cost_per_token = if metadata.total_tokens > 0 {
            metadata.cost / metadata.total_tokens as f64
        } else {
            0.0
        };
        
        CostBreakdown {
            total_cost: metadata.cost,
            stage_costs,
            cost_per_token,
        }
    }
    
    /// Calculate confidence score based on stage results
    fn calculate_confidence(stages: &[crate::consensus::types::StageResult]) -> ConfidenceScore {
        let mut factors = Vec::new();
        let mut total_score = 0.0;
        
        // Factor 1: All stages completed successfully
        let all_completed = stages.len() == 4;
        factors.push(("Pipeline Completion".to_string(), if all_completed { 1.0 } else { 0.7 }));
        total_score += if all_completed { 0.25 } else { 0.175 };
        
        // Factor 2: Quality scores from analytics
        let avg_quality = stages.iter()
            .filter_map(|s| s.analytics.as_ref())
            .map(|a| a.quality_score)
            .sum::<f64>() / stages.len() as f64;
        factors.push(("Average Quality".to_string(), avg_quality));
        total_score += avg_quality * 0.25;
        
        // Factor 3: No errors or retries
        let no_errors = stages.iter()
            .all(|s| s.analytics.as_ref().map_or(true, |a| a.error_count == 0));
        factors.push(("Error-Free Execution".to_string(), if no_errors { 1.0 } else { 0.6 }));
        total_score += if no_errors { 0.25 } else { 0.15 };
        
        // Factor 4: Response consistency
        let consistency_score = 0.9; // Would calculate based on response similarity
        factors.push(("Response Consistency".to_string(), consistency_score));
        total_score += consistency_score * 0.25;
        
        ConfidenceScore {
            score: total_score.min(1.0),
            factors,
        }
    }
    
    /// Build stage journey visualization
    fn build_stage_journey(stages: &[crate::consensus::types::StageResult]) -> StageJourney {
        let stage_infos = stages.iter().map(|stage| {
            let status = if stage.analytics.as_ref().map_or(false, |a| a.features.optimization_applied.unwrap_or(false)) {
                StageStatus::Optimized
            } else if stage.analytics.as_ref().map_or(0.0, |a| a.quality_score) > 0.9 {
                StageStatus::Enhanced
            } else {
                StageStatus::Complete
            };
            
            StageInfo {
                name: stage.stage_name.clone(),
                model: stage.model.clone(),
                tokens: stage.usage.as_ref().map_or(0, |u| u.total_tokens),
                duration: stage.analytics.as_ref().map_or(0.0, |a| a.duration),
                status,
            }
        }).collect();
        
        StageJourney { stages: stage_infos }
    }
    
    /// Format the result as a beautiful string for display
    pub fn format_for_display(&self) -> String {
        let mut output = String::new();
        
        // Executive Summary Box
        output.push_str(&self.format_executive_summary());
        output.push_str("\n\n");
        
        // Detailed Findings
        output.push_str(&self.format_detailed_findings());
        output.push_str("\n\n");
        
        // 4-Stage Journey
        output.push_str(&self.format_stage_journey());
        output.push_str("\n\n");
        
        // Performance & Cost
        output.push_str(&self.format_metrics_and_cost());
        
        output
    }
    
    /// Format executive summary with box drawing
    fn format_executive_summary(&self) -> String {
        let mut output = String::new();
        
        output.push_str("╔═══════════════════════════════════════════════════════════════════════╗\n");
        output.push_str("║                          EXECUTIVE SUMMARY                             ║\n");
        output.push_str("╠═══════════════════════════════════════════════════════════════════════╣\n");
        
        if !self.executive_summary.key_points.is_empty() {
            output.push_str("║ 📌 Key Points:                                                        ║\n");
            for point in &self.executive_summary.key_points {
                let formatted = format!("║   • {:<65} ║", Self::truncate_string(point, 65));
                output.push_str(&formatted);
                output.push('\n');
            }
        }
        
        if !self.executive_summary.action_items.is_empty() {
            output.push_str("║                                                                       ║\n");
            output.push_str("║ 🎯 Action Items:                                                      ║\n");
            for item in &self.executive_summary.action_items {
                let formatted = format!("║   ✓ {:<65} ║", Self::truncate_string(item, 65));
                output.push_str(&formatted);
                output.push('\n');
            }
        }
        
        output.push_str("╚═══════════════════════════════════════════════════════════════════════╝");
        
        output
    }
    
    /// Format detailed findings with visual hierarchy
    fn format_detailed_findings(&self) -> String {
        let mut output = String::new();
        
        for finding in &self.detailed_findings {
            let border = match finding.emphasis {
                EmphasisLevel::Critical => "═",
                EmphasisLevel::Important => "─",
                EmphasisLevel::Normal => "·",
            };
            
            output.push_str(&format!("{} {} {}\n", finding.icon, finding.title, border.repeat(50 - finding.title.len())));
            output.push_str(&finding.content);
            output.push_str("\n\n");
        }
        
        output
    }
    
    /// Format stage journey visualization
    fn format_stage_journey(&self) -> String {
        let mut output = String::new();
        
        output.push_str("🚀 Consensus Journey\n");
        output.push_str("═══════════════════════════════════════════════════════════════════════\n\n");
        
        for (i, stage) in self.stage_journey.stages.iter().enumerate() {
            let status_icon = match stage.status {
                StageStatus::Complete => "✅",
                StageStatus::Optimized => "⚡",
                StageStatus::Enhanced => "✨",
            };
            
            let arrow = if i < self.stage_journey.stages.len() - 1 { " → " } else { "" };
            
            output.push_str(&format!(
                "{} {} ({}ms, {} tokens){}",
                status_icon,
                stage.name,
                stage.duration as u64,
                stage.tokens,
                arrow
            ));
        }
        
        output
    }
    
    /// Format performance metrics and cost breakdown
    fn format_metrics_and_cost(&self) -> String {
        let mut output = String::new();
        
        // Performance Box
        output.push_str("┌─────────────────────────┬─────────────────────────────────────────────┐\n");
        output.push_str("│  PERFORMANCE METRICS    │  COST BREAKDOWN                             │\n");
        output.push_str("├─────────────────────────┼─────────────────────────────────────────────┤\n");
        
        // Performance data
        output.push_str(&format!("│ Total Duration: {:.2}s   │ Total Cost: ${:.4}                        │\n", 
            self.performance_metrics.total_duration,
            self.cost_breakdown.total_cost
        ));
        
        output.push_str(&format!("│ Total Tokens: {:>8}  │ Cost/Token: ${:.6}                     │\n",
            self.performance_metrics.tokens_used,
            self.cost_breakdown.cost_per_token
        ));
        
        output.push_str(&format!("│ Models Used: {:>10} │ Confidence: {}                    │\n",
            self.performance_metrics.models_accessed,
            self.format_confidence_bar()
        ));
        
        output.push_str("└─────────────────────────┴─────────────────────────────────────────────┘");
        
        output
    }
    
    /// Format confidence score as a visual bar
    fn format_confidence_bar(&self) -> String {
        let percentage = (self.confidence.score * 100.0) as u32;
        let filled = (self.confidence.score * 20.0) as usize;
        let empty = 20 - filled;
        
        format!("{}{} {}%", "█".repeat(filled), "░".repeat(empty), percentage)
    }
    
    /// Truncate string to fit within width
    fn truncate_string(s: &str, max_width: usize) -> String {
        if s.len() <= max_width {
            s.to_string()
        } else {
            format!("{}...", &s[..max_width - 3])
        }
    }
}

impl fmt::Display for FormattedConsensusResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_for_display())
    }
}