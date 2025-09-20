// Operation Confidence and Risk Display Components
use crate::ai_helpers::scores::*;
use crate::consensus::operation_history::OperationStatistics;
use crate::consensus::operation_intelligence::{OperationAnalysis, UnifiedScore};
use dioxus::prelude::*;

/// Main confidence and risk display widget
#[component]
pub fn ConfidenceDisplay(
    operation_analysis: Signal<Option<OperationAnalysis>>,
    show_details: Signal<bool>,
    compact_mode: Option<bool>,
) -> Element {
    let compact = compact_mode.unwrap_or(false);

    rsx! {
        div {
            class: "confidence-display {if compact { \"compact\" } else { \"\" }}",

            if let Some(analysis) = operation_analysis() {
                // Main score display
                div {
                    class: "main-scores",
                    onclick: move |_| show_details.set(!show_details()),

                    ConfidenceIndicator {
                        confidence: analysis.unified_score.confidence,
                        size: if compact { IndicatorSize::Small } else { IndicatorSize::Medium },
                        show_label: !compact
                    }

                    RiskIndicator {
                        risk: analysis.unified_score.risk,
                        size: if compact { IndicatorSize::Small } else { IndicatorSize::Medium },
                        show_label: !compact
                    }

                    if !compact {
                        OverallScoreIndicator {
                            unified_score: analysis.unified_score.clone()
                        }
                    }
                }

                // Detailed breakdown (expandable)
                if show_details() && !compact {
                    div {
                        class: "score-details",

                        // Component scores breakdown
                        if let Some(scores) = &analysis.unified_score.component_scores {
                            ComponentScoresBreakdown {
                                scores: scores.clone()
                            }
                        }

                        // Scoring factors
                        if let Some(factors) = &analysis.unified_score.scoring_factors {
                            ScoringFactorsDisplay {
                                factors: factors.clone()
                            }
                        }

                        // Historical context
                        HistoricalContextDisplay {
                            analysis: analysis.clone()
                        }
                    }
                }
            } else {
                div {
                    class: "no-analysis",
                    "No operation analysis available"
                }
            }
        }
    }
}

/// Individual confidence indicator
#[component]
pub fn ConfidenceIndicator(confidence: f32, size: IndicatorSize, show_label: bool) -> Element {
    let confidence_class = get_confidence_class(confidence);
    let size_class = get_size_class(&size);

    rsx! {
        div {
            class: "confidence-indicator {confidence_class} {size_class}",
            title: "Confidence Level: {confidence:.1}%",

            div {
                class: "indicator-visual",

                // Circular progress indicator
                svg {
                    class: "confidence-circle",
                    viewBox: "0 0 42 42",

                    circle {
                        class: "circle-bg",
                        cx: "21",
                        cy: "21",
                        r: "15.915",
                        fill: "transparent",
                        stroke: "currentColor",
                        stroke_width: "2",
                        opacity: "0.3"
                    }

                    circle {
                        class: "circle-progress",
                        cx: "21",
                        cy: "21",
                        r: "15.915",
                        fill: "transparent",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_dasharray: "100",
                        stroke_dashoffset: "{100.0 - confidence}",
                        stroke_linecap: "round",
                        transform: "rotate(-90 21 21)"
                    }
                }

                // Percentage text
                div {
                    class: "indicator-text",
                    "{confidence:.0}%"
                }
            }

            if show_label {
                div {
                    class: "indicator-label",
                    "Confidence"
                }
            }
        }
    }
}

/// Individual risk indicator
#[component]
pub fn RiskIndicator(risk: f32, size: IndicatorSize, show_label: bool) -> Element {
    let risk_class = get_risk_class(risk);
    let size_class = get_size_class(&size);

    rsx! {
        div {
            class: "risk-indicator {risk_class} {size_class}",
            title: "Risk Level: {risk:.1}%",

            div {
                class: "indicator-visual",

                // Risk bar indicator
                div {
                    class: "risk-bar-container",

                    div {
                        class: "risk-bar-bg",
                    }

                    div {
                        class: "risk-bar-fill",
                        style: "width: {risk}%"
                    }

                    // Risk level markers
                    div { class: "risk-marker low", style: "left: 15%" }
                    div { class: "risk-marker medium", style: "left: 25%" }
                    div { class: "risk-marker high", style: "left: 40%" }
                    div { class: "risk-marker critical", style: "left: 60%" }
                }

                // Risk icon and text
                div {
                    class: "risk-content",

                    div {
                        class: "risk-icon",
                        {get_risk_icon(risk)}
                    }

                    div {
                        class: "risk-text",
                        "{risk:.0}%"
                    }
                }
            }

            if show_label {
                div {
                    class: "indicator-label",
                    "Risk"
                }
            }
        }
    }
}

/// Overall score indicator combining confidence and risk
#[component]
pub fn OverallScoreIndicator(unified_score: UnifiedScore) -> Element {
    let overall_score = calculate_overall_score(&unified_score);
    let recommendation = get_score_recommendation(overall_score);

    rsx! {
        div {
            class: "overall-score-indicator score-{get_score_tier(overall_score)}",
            title: "Overall Score: {overall_score:.1}% - {recommendation}",

            div {
                class: "score-visual",

                // Gauge-style indicator
                div {
                    class: "score-gauge",

                    div {
                        class: "gauge-bg",
                    }

                    div {
                        class: "gauge-fill",
                        style: "transform: rotate({(overall_score / 100.0) * 180.0 - 90.0}deg)"
                    }

                    div {
                        class: "gauge-pointer",
                        style: "transform: rotate({(overall_score / 100.0) * 180.0 - 90.0}deg)"
                    }
                }

                div {
                    class: "score-text",
                    "{overall_score:.0}"
                }
            }

            div {
                class: "score-recommendation",
                "{recommendation}"
            }
        }
    }
}

/// Component scores breakdown
#[component]
pub fn ComponentScoresBreakdown(
    scores: crate::consensus::operation_intelligence::ComponentScores,
) -> Element {
    rsx! {
        div {
            class: "component-scores-breakdown",

            h4 { "AI Helper Contributions" }

            div {
                class: "component-grid",

                ComponentScoreItem {
                    name: "Knowledge Indexer",
                    score: scores.knowledge_indexer,
                    description: "File operation tracking and similarity analysis"
                }

                ComponentScoreItem {
                    name: "Context Retriever",
                    score: scores.context_retriever,
                    description: "Historical success rates and patterns"
                }

                ComponentScoreItem {
                    name: "Pattern Recognizer",
                    score: scores.pattern_recognizer,
                    description: "Safety patterns and anti-pattern detection"
                }

                ComponentScoreItem {
                    name: "Quality Analyzer",
                    score: scores.quality_analyzer,
                    description: "Risk assessment and conflict detection"
                }

                ComponentScoreItem {
                    name: "Knowledge Synthesizer",
                    score: scores.knowledge_synthesizer,
                    description: "Operation planning and backup strategies"
                }
            }
        }
    }
}

/// Individual component score item
#[component]
pub fn ComponentScoreItem(
    name: &'static str,
    score: Option<f32>,
    description: &'static str,
) -> Element {
    rsx! {
        div {
            class: "component-score-item",
            title: "{description}",

            div {
                class: "component-header",

                div {
                    class: "component-name",
                    "{name}"
                }

                div {
                    class: "component-score",
                    if let Some(s) = score {
                        span { class: "score-value", "{s:.0}%" }
                    } else {
                        span { class: "score-none", "N/A" }
                    }
                }
            }

            if let Some(s) = score {
                div {
                    class: "component-bar",

                    div {
                        class: "bar-fill",
                        style: "width: {s}%; background: {get_score_color(s)}"
                    }
                }
            }

            div {
                class: "component-description",
                "{description}"
            }
        }
    }
}

/// Scoring factors display
#[component]
pub fn ScoringFactorsDisplay(
    factors: crate::consensus::operation_intelligence::ScoringFactors,
) -> Element {
    rsx! {
        div {
            class: "scoring-factors-display",

            h4 { "Scoring Factors" }

            div {
                class: "factors-grid",

                FactorItem {
                    name: "Historical Data",
                    impact: factors.historical_success_rate,
                    description: "Based on past operation outcomes"
                }

                FactorItem {
                    name: "Pattern Matching",
                    impact: factors.pattern_confidence,
                    description: "Similar operation patterns"
                }

                FactorItem {
                    name: "Context Quality",
                    impact: factors.context_completeness,
                    description: "Available contextual information"
                }

                FactorItem {
                    name: "Code Quality",
                    impact: factors.code_quality_indicators,
                    description: "Syntax and structure analysis"
                }

                FactorItem {
                    name: "Dependency Safety",
                    impact: factors.dependency_safety,
                    description: "Impact on related files"
                }

                FactorItem {
                    name: "User Preferences",
                    impact: factors.user_trust_level,
                    description: "User feedback and overrides"
                }
            }
        }
    }
}

/// Individual factor item
#[component]
pub fn FactorItem(name: &'static str, impact: f32, description: &'static str) -> Element {
    let impact_class = if impact > 0.0 {
        "positive"
    } else if impact < 0.0 {
        "negative"
    } else {
        "neutral"
    };

    rsx! {
        div {
            class: "factor-item {impact_class}",
            title: "{description}",

            div {
                class: "factor-info",

                div {
                    class: "factor-name",
                    "{name}"
                }

                div {
                    class: "factor-impact",
                    "{if impact > 0.0 { \"+\" } else { \"\" }}{impact:.1}%"
                }
            }

            div {
                class: "factor-bar",

                div {
                    class: "bar-center",
                }

                if impact != 0.0 {
                    div {
                        class: "bar-impact",
                        style: "width: {impact.abs() * 2.0}%; {if impact > 0.0 { \"left: 50%\" } else { \"right: 50%\" }}"
                    }
                }
            }
        }
    }
}

/// Historical context display
#[component]
pub fn HistoricalContextDisplay(analysis: OperationAnalysis) -> Element {
    rsx! {
        div {
            class: "historical-context-display",

            h4 { "Historical Context" }

            div {
                class: "context-stats",

                if let Some(stats) = analysis.operation_statistics {
                    StatItem {
                        label: "Similar Operations",
                        value: stats.similar_operations_count.to_string(),
                        subtext: format!("{:.0}% success rate", stats.similar_operations_success_rate * 100.0)
                    }

                    StatItem {
                        label: "This File Type",
                        value: stats.file_type_operations.to_string(),
                        subtext: format!("{:.0}% auto-accepted", stats.file_type_auto_accept_rate * 100.0)
                    }

                    StatItem {
                        label: "Recent Trend",
                        value: format!("{:.0}%", stats.recent_success_trend * 100.0),
                        subtext: "Last 7 days"
                    }
                } else {
                    div {
                        class: "no-historical-data",
                        "No historical data available"
                    }
                }
            }
        }
    }
}

/// Statistics item
#[component]
pub fn StatItem(label: &'static str, value: String, subtext: String) -> Element {
    rsx! {
        div {
            class: "stat-item",

            div {
                class: "stat-label",
                "{label}"
            }

            div {
                class: "stat-value",
                "{value}"
            }

            div {
                class: "stat-subtext",
                "{subtext}"
            }
        }
    }
}

/// Real-time confidence monitor for ongoing operations
#[component]
pub fn RealTimeConfidenceMonitor(
    operations: Signal<Vec<crate::consensus::operation_parser::EnhancedFileOperation>>,
    current_analysis: Signal<Option<OperationAnalysis>>,
) -> Element {
    let show_monitor = use_signal(|| false);

    rsx! {
        div {
            class: "realtime-confidence-monitor",

            // Toggle button
            button {
                class: "monitor-toggle",
                onclick: move |_| show_monitor.set(!show_monitor()),
                title: "Show real-time confidence monitoring",
                "📊"
            }

            // Monitor panel
            if show_monitor() {
                div {
                    class: "monitor-panel",

                    div {
                        class: "monitor-header",
                        h3 { "Real-Time Confidence Monitor" }
                        button {
                            class: "close-btn",
                            onclick: move |_| show_monitor.set(false),
                            "×"
                        }
                    }

                    div {
                        class: "monitor-content",

                        // Current operation
                        if let Some(analysis) = current_analysis() {
                            div {
                                class: "current-operation",

                                h4 { "Current Analysis" }

                                div {
                                    class: "current-scores",

                                    MiniConfidenceIndicator {
                                        confidence: analysis.unified_score.confidence,
                                        risk: analysis.unified_score.risk
                                    }

                                    div {
                                        class: "score-trend",
                                        // Trend indicators would go here
                                        "Trend: Stable"
                                    }
                                }
                            }
                        }

                        // Operation queue
                        div {
                            class: "operation-queue",

                            h4 { "Pending Operations ({operations().len()})" }

                            for (i, op) in operations().iter().enumerate() {
                                OperationQueueItem {
                                    operation: op.clone(),
                                    index: i,
                                    estimated_confidence: 75.0 + (i as f32 * 5.0) % 20.0
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Mini confidence indicator for compact displays
#[component]
pub fn MiniConfidenceIndicator(confidence: f32, risk: f32) -> Element {
    rsx! {
        div {
            class: "mini-confidence-indicator",

            div {
                class: "mini-confidence",
                style: "background: {get_confidence_color(confidence)}",
                "{confidence:.0}%"
            }

            div {
                class: "mini-risk",
                style: "background: {get_risk_color(risk)}",
                "{risk:.0}%"
            }
        }
    }
}

/// Operation queue item
#[component]
pub fn OperationQueueItem(
    operation: crate::consensus::operation_parser::EnhancedFileOperation,
    index: usize,
    estimated_confidence: f32,
) -> Element {
    rsx! {
        div {
            class: "operation-queue-item",

            div {
                class: "queue-index",
                "{index + 1}"
            }

            div {
                class: "operation-info",

                div {
                    class: "operation-type",
                    "{get_operation_type_label(&operation.operation)}"
                }

                div {
                    class: "operation-file",
                    "{get_operation_file_name(&operation.operation)}"
                }
            }

            div {
                class: "estimated-confidence",
                style: "color: {get_confidence_color(estimated_confidence)}",
                "~{estimated_confidence:.0}%"
            }
        }
    }
}

// Enums and helper functions

#[derive(Debug, Clone, PartialEq)]
pub enum IndicatorSize {
    Small,
    Medium,
    Large,
}

fn get_size_class(size: &IndicatorSize) -> &'static str {
    match size {
        IndicatorSize::Small => "size-small",
        IndicatorSize::Medium => "size-medium",
        IndicatorSize::Large => "size-large",
    }
}

fn get_confidence_class(confidence: f32) -> &'static str {
    match confidence {
        c if c >= 90.0 => "confidence-excellent",
        c if c >= 80.0 => "confidence-high",
        c if c >= 70.0 => "confidence-good",
        c if c >= 60.0 => "confidence-medium",
        c if c >= 50.0 => "confidence-low",
        _ => "confidence-poor",
    }
}

fn get_risk_class(risk: f32) -> &'static str {
    match risk {
        r if r <= 15.0 => "risk-minimal",
        r if r <= 25.0 => "risk-low",
        r if r <= 40.0 => "risk-medium",
        r if r <= 60.0 => "risk-high",
        _ => "risk-critical",
    }
}

fn get_confidence_color(confidence: f32) -> &'static str {
    match confidence {
        c if c >= 90.0 => "#4CAF50", // Green
        c if c >= 80.0 => "#8BC34A", // Light green
        c if c >= 70.0 => "#CDDC39", // Lime
        c if c >= 60.0 => "#FFC107", // Amber
        c if c >= 50.0 => "#FF9800", // Orange
        _ => "#F44336",              // Red
    }
}

fn get_risk_color(risk: f32) -> &'static str {
    match risk {
        r if r <= 15.0 => "#4CAF50", // Green
        r if r <= 25.0 => "#8BC34A", // Light green
        r if r <= 40.0 => "#FFC107", // Amber
        r if r <= 60.0 => "#FF9800", // Orange
        _ => "#F44336",              // Red
    }
}

fn get_score_color(score: f32) -> &'static str {
    match score {
        s if s >= 80.0 => "#4CAF50",
        s if s >= 60.0 => "#8BC34A",
        s if s >= 40.0 => "#FFC107",
        s if s >= 20.0 => "#FF9800",
        _ => "#F44336",
    }
}

fn get_risk_icon(risk: f32) -> &'static str {
    match risk {
        r if r <= 15.0 => "✓",
        r if r <= 25.0 => "!",
        r if r <= 40.0 => "⚠",
        r if r <= 60.0 => "⚠",
        _ => "⚠",
    }
}

fn calculate_overall_score(unified_score: &UnifiedScore) -> f32 {
    // Simple calculation: high confidence, low risk = high score
    (unified_score.confidence + (100.0 - unified_score.risk)) / 2.0
}

fn get_score_recommendation(score: f32) -> &'static str {
    match score {
        s if s >= 85.0 => "Excellent - Auto-execute",
        s if s >= 75.0 => "Good - Likely safe",
        s if s >= 65.0 => "Fair - Review recommended",
        s if s >= 55.0 => "Poor - Manual review required",
        _ => "Critical - Do not auto-execute",
    }
}

fn get_score_tier(score: f32) -> &'static str {
    match score {
        s if s >= 85.0 => "excellent",
        s if s >= 75.0 => "good",
        s if s >= 65.0 => "fair",
        s if s >= 55.0 => "poor",
        _ => "critical",
    }
}

fn get_operation_type_label(
    operation: &crate::consensus::stages::file_aware_curator::FileOperation,
) -> &'static str {
    match operation {
        crate::consensus::stages::file_aware_curator::FileOperation::Create { .. } => "Create",
        crate::consensus::stages::file_aware_curator::FileOperation::Update { .. } => "Update",
        crate::consensus::stages::file_aware_curator::FileOperation::Delete { .. } => "Delete",
        crate::consensus::stages::file_aware_curator::FileOperation::Rename { .. } => "Rename",
        crate::consensus::stages::file_aware_curator::FileOperation::Move { .. } => "Move",
    }
}

fn get_operation_file_name(
    operation: &crate::consensus::stages::file_aware_curator::FileOperation,
) -> String {
    match operation {
        crate::consensus::stages::file_aware_curator::FileOperation::Create { path, .. }
        | crate::consensus::stages::file_aware_curator::FileOperation::Update { path, .. }
        | crate::consensus::stages::file_aware_curator::FileOperation::Delete { path } => path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        crate::consensus::stages::file_aware_curator::FileOperation::Rename {
            new_path, ..
        } => new_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        crate::consensus::stages::file_aware_curator::FileOperation::Move {
            destination, ..
        } => destination
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    }
}
