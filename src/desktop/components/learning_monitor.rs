// Learning System Monitoring and Control UI Components
use crate::consensus::learning_system::{
    ExperimentRecommendation, ExperimentStatus, ExperimentType, LearningCycleResult,
    LearningSystemStatus, LearningTrend, PendingImprovement,
};
use crate::consensus::outcome_tracker::{AccuracyMetrics, ErrorPattern, HelperAccuracy};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use std::collections::HashMap;

/// Main learning system monitoring dashboard
#[component]
pub fn LearningSystemDashboard(
    learning_status: Signal<Option<LearningSystemStatus>>,
    pending_improvements: Signal<Vec<PendingImprovement>>,
    recent_cycles: Signal<Vec<LearningCycleResult>>,
    show_dashboard: Signal<bool>,
    on_approve_improvement: EventHandler<String>,
    on_reject_improvement: EventHandler<String>,
    on_trigger_learning_cycle: EventHandler<()>,
) -> Element {
    let show_details = use_signal(|| false);
    let selected_helper = use_signal(|| None::<String>);

    rsx! {
        if show_dashboard() {
            div {
                class: "learning-dashboard-overlay",
                onclick: move |e| {
                    if e.target() == e.current_target() {
                        show_dashboard.set(false);
                    }
                },

                div {
                    class: "learning-dashboard",

                    // Dashboard header
                    div {
                        class: "dashboard-header",

                        h2 {
                            class: "dashboard-title",
                            "🧠 AI Learning System"
                        }

                        if let Some(status) = learning_status() {
                            div {
                                class: "system-status",

                                div {
                                    class: "status-indicator {if status.is_active { \\\"active\\\" } else { \\\"inactive\\\" }}",
                                    {if status.is_active { "●" } else { "○" }}
                                }

                                span {
                                    class: "status-text",
                                    {if status.is_active { "Active" } else { "Inactive" }}
                                }

                                div {
                                    class: "trend-indicator {get_trend_class(&status.learning_trend)}",
                                    "{get_trend_icon(&status.learning_trend)} {status.learning_trend:?}"
                                }
                            }
                        }

                        button {
                            class: "close-button",
                            onclick: move |_| show_dashboard.set(false),
                            "×"
                        }
                    }

                    // Dashboard content
                    div {
                        class: "dashboard-content",

                        // System overview
                        if let Some(status) = learning_status() {
                            LearningSystemOverview {
                                status: status,
                                show_details: show_details,
                                selected_helper: selected_helper
                            }
                        }

                        // Pending improvements section
                        if !pending_improvements().is_empty() {
                            PendingImprovementsSection {
                                improvements: pending_improvements(),
                                on_approve: on_approve_improvement,
                                on_reject: on_reject_improvement
                            }
                        }

                        // Learning history
                        if !recent_cycles().is_empty() {
                            LearningHistorySection {
                                cycles: recent_cycles()
                            }
                        }
                    }

                    // Dashboard actions
                    div {
                        class: "dashboard-actions",

                        button {
                            class: "action-button secondary",
                            onclick: move |_| show_details.set(!show_details()),
                            "{if show_details() { \"Hide Details\" } else { \"Show Details\" }}"
                        }

                        button {
                            class: "action-button primary",
                            onclick: move |_| on_trigger_learning_cycle.call(()),
                            "🔄 Trigger Learning Cycle"
                        }

                        button {
                            class: "action-button secondary",
                            onclick: move |_| show_dashboard.set(false),
                            "Close"
                        }
                    }
                }
            }
        }
    }
}

/// System overview with key metrics and helper performance
#[component]
pub fn LearningSystemOverview(
    status: LearningSystemStatus,
    show_details: Signal<bool>,
    selected_helper: Signal<Option<String>>,
) -> Element {
    rsx! {
        div {
            class: "learning-system-overview",

            // Key metrics
            div {
                class: "metrics-section",

                h3 { "System Metrics" }

                div {
                    class: "metrics-grid",

                    MetricCard {
                        title: "Learning Cycles",
                        value: status.total_cycles.to_string(),
                        subtitle: format!("{} successful", status.successful_improvements),
                        trend: Some(calculate_success_rate(&status))
                    }

                    MetricCard {
                        title: "Overall Accuracy",
                        value: format!("{:.1}%", status.current_performance.overall_accuracy * 100.0),
                        subtitle: "Current performance",
                        trend: Some(get_accuracy_trend(&status.learning_trend))
                    }

                    MetricCard {
                        title: "Pending Improvements",
                        value: status.pending_improvements.to_string(),
                        subtitle: "Awaiting approval",
                        trend: None
                    }

                    MetricCard {
                        title: "Active Helpers",
                        value: status.current_weights.len().to_string(),
                        subtitle: "AI components",
                        trend: None
                    }
                }
            }

            // Helper performance breakdown
            div {
                class: "helpers-section",

                div {
                    class: "section-header",
                    onclick: move |_| show_details.set(!show_details()),

                    h3 { "AI Helper Performance" }
                    div {
                        class: "expand-icon {if show_details() { \\\"expanded\\\" } else { \\\"\\\" }}",
                        "▼"
                    }
                }

                if show_details() {
                    div {
                        class: "helpers-grid",
                        for (helper_name, accuracy) in &status.current_performance.by_helper {
                            HelperPerformanceCard {
                                helper_name: helper_name.clone(),
                                accuracy: accuracy.clone(),
                                current_weight: status.current_weights.get(helper_name).copied().unwrap_or(1.0),
                                is_selected: selected_helper() == Some(helper_name.clone()),
                                on_select: move |name| selected_helper.set(Some(name))
                            }
                        }
                    }
                }
            }

            // Selected helper details
            if let Some(ref helper) = selected_helper() {
                if let Some(accuracy) = status.current_performance.by_helper.get(helper) {
                    HelperDetailPanel {
                        helper_name: helper.clone(),
                        accuracy: accuracy.clone(),
                        current_weight: status.current_weights.get(helper).copied().unwrap_or(1.0),
                        on_close: move |_| selected_helper.set(None)
                    }
                }
            }
        }
    }
}

/// Individual metric card
#[component]
pub fn MetricCard(title: String, value: String, subtitle: String, trend: Option<f32>) -> Element {
    rsx! {
        div {
            class: "metric-card",

            div {
                class: "metric-header",
                h4 { "{title}" }
                if let Some(trend_value) = trend {
                    div {
                        class: "trend-indicator {if trend_value > 0.0 { \\\"positive\\\" } else if trend_value < 0.0 { \\\"negative\\\" } else { \\\"neutral\\\" }}",
                        "{if trend_value > 0.0 { \"↗\" } else if trend_value < 0.0 { \"↘\" } else { \"→\" }}"
                    }
                }
            }

            div {
                class: "metric-value",
                "{value}"
            }

            div {
                class: "metric-subtitle",
                "{subtitle}"
            }
        }
    }
}

/// Helper performance card
#[component]
pub fn HelperPerformanceCard(
    helper_name: String,
    accuracy: HelperAccuracy,
    current_weight: f32,
    is_selected: bool,
    on_select: EventHandler<String>,
) -> Element {
    let accuracy_percentage = accuracy.prediction_accuracy * 100.0;
    let accuracy_class = get_accuracy_class(accuracy.prediction_accuracy);

    rsx! {
        div {
            class: "helper-performance-card {if is_selected { \\\"selected\\\" } else { \\\"\\\" }}",
            onclick: move |_| on_select.call(helper_name.clone()),

            div {
                class: "helper-header",

                div {
                    class: "helper-name",
                    "{get_helper_display_name(&helper_name)}"
                }

                div {
                    class: "helper-weight",
                    "Weight: {current_weight:.2}x"
                }
            }

            div {
                class: "accuracy-display",

                div {
                    class: "accuracy-circle {accuracy_class}",

                    svg {
                        class: "accuracy-svg",
                        viewBox: "0 0 42 42",

                        circle {
                            class: "accuracy-bg",
                            cx: "21",
                            cy: "21",
                            r: "15.915",
                            fill: "transparent",
                            stroke: "currentColor",
                            stroke_width: "2",
                            opacity: "0.3"
                        }

                        circle {
                            class: "accuracy-progress",
                            cx: "21",
                            cy: "21",
                            r: "15.915",
                            fill: "transparent",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_dasharray: "100",
                            stroke_dashoffset: "{100.0 - accuracy_percentage}",
                            stroke_linecap: "round",
                            transform: "rotate(-90 21 21)"
                        }
                    }

                    div {
                        class: "accuracy-text",
                        "{accuracy_percentage:.0}%"
                    }
                }
            }

            div {
                class: "helper-metrics",

                div {
                    class: "metric-item",
                    span { class: "metric-label", "False Positive:" }
                    span { class: "metric-value", "{accuracy.false_positive_rate:.1}%" }
                }

                div {
                    class: "metric-item",
                    span { class: "metric-label", "False Negative:" }
                    span { class: "metric-value", "{accuracy.false_negative_rate:.1}%" }
                }

                div {
                    class: "metric-item",
                    span { class: "metric-label", "Correlation:" }
                    span { class: "metric-value", "{accuracy.confidence_correlation:.2}" }
                }
            }

            div {
                class: "trend-display",
                if !accuracy.recent_trend.is_empty() {
                    MiniTrendChart {
                        data: accuracy.recent_trend.clone()
                    }
                }
            }
        }
    }
}

/// Helper detail panel
#[component]
pub fn HelperDetailPanel(
    helper_name: String,
    accuracy: HelperAccuracy,
    current_weight: f32,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "helper-detail-panel",

            div {
                class: "panel-header",

                h3 { "{get_helper_display_name(&helper_name)} Details" }

                button {
                    class: "close-button",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            div {
                class: "panel-content",

                // Performance metrics
                div {
                    class: "metrics-section",

                    h4 { "Performance Metrics" }

                    div {
                        class: "detailed-metrics",

                        div {
                            class: "metric-row",
                            span { class: "label", "Prediction Accuracy:" }
                            span { class: "value", "{accuracy.prediction_accuracy * 100.0:.1}%" }
                            div { class: "bar-container",
                                div {
                                    class: "bar-fill",
                                    style: "width: {accuracy.prediction_accuracy * 100.0}%; background: {get_accuracy_color(accuracy.prediction_accuracy)}"
                                }
                            }
                        }

                        div {
                            class: "metric-row",
                            span { class: "label", "Confidence Correlation:" }
                            span { class: "value", "{accuracy.confidence_correlation:.2}" }
                            div { class: "bar-container",
                                div {
                                    class: "bar-fill",
                                    style: "width: {(accuracy.confidence_correlation + 1.0) * 50.0}%; background: {get_correlation_color(accuracy.confidence_correlation)}"
                                }
                            }
                        }

                        div {
                            class: "metric-row",
                            span { class: "label", "False Positive Rate:" }
                            span { class: "value", "{accuracy.false_positive_rate:.1}%" }
                            div { class: "bar-container",
                                div {
                                    class: "bar-fill error",
                                    style: "width: {accuracy.false_positive_rate}%"
                                }
                            }
                        }

                        div {
                            class: "metric-row",
                            span { class: "label", "False Negative Rate:" }
                            span { class: "value", "{accuracy.false_negative_rate:.1}%" }
                            div { class: "bar-container",
                                div {
                                    class: "bar-fill error",
                                    style: "width: {accuracy.false_negative_rate}%"
                                }
                            }
                        }

                        div {
                            class: "metric-row",
                            span { class: "label", "Current Weight:" }
                            span { class: "value", "{current_weight:.2}x" }
                            div { class: "bar-container",
                                div {
                                    class: "bar-fill",
                                    style: "width: {(current_weight * 50.0).min(100.0)}%; background: {get_weight_color(current_weight)}"
                                }
                            }
                        }

                        div {
                            class: "metric-row",
                            span { class: "label", "Suggested Adjustment:" }
                            span { class: "value", "{accuracy.suggested_weight_adjustment:.2}x" }
                            div {
                                class: "adjustment-indicator {if accuracy.suggested_weight_adjustment > 1.0 { \\\"increase\\\" } else if accuracy.suggested_weight_adjustment < 1.0 { \\\"decrease\\\" } else { \\\"maintain\\\" }}",
                                "{get_adjustment_icon(accuracy.suggested_weight_adjustment)}"
                            }
                        }
                    }
                }

                // Performance trend
                div {
                    class: "trend-section",

                    h4 { "Performance Trend" }

                    if !accuracy.recent_trend.is_empty() {
                        DetailedTrendChart {
                            data: accuracy.recent_trend.clone(),
                            helper_name: helper_name.clone()
                        }
                    } else {
                        div {
                            class: "no-trend-data",
                            "Insufficient data for trend analysis"
                        }
                    }
                }

                // Helper description
                div {
                    class: "description-section",

                    h4 { "Helper Description" }

                    p {
                        class: "helper-description",
                        "{get_helper_description(&helper_name)}"
                    }
                }
            }
        }
    }
}

/// Pending improvements section
#[component]
pub fn PendingImprovementsSection(
    improvements: Vec<PendingImprovement>,
    on_approve: EventHandler<String>,
    on_reject: EventHandler<String>,
) -> Element {
    let show_improvements = use_signal(|| true);

    rsx! {
        div {
            class: "pending-improvements-section",

            div {
                class: "section-header",
                onclick: move |_| show_improvements.set(!show_improvements()),

                h3 { "Pending Improvements ({improvements.len()})" }

                div {
                    class: "expand-icon {if show_improvements() { \\\"expanded\\\" } else { \\\"\\\" }}",
                    "▼"
                }
            }

            if show_improvements() {
                div {
                    class: "improvements-list",
                    for improvement in &improvements {
                        ImprovementCard {
                            improvement: improvement.clone(),
                            on_approve: on_approve,
                            on_reject: on_reject
                        }
                    }
                }
            }
        }
    }
}

/// Individual improvement card
#[component]
pub fn ImprovementCard(
    improvement: PendingImprovement,
    on_approve: EventHandler<String>,
    on_reject: EventHandler<String>,
) -> Element {
    let show_details = use_signal(|| false);

    rsx! {
        div {
            class: "improvement-card",

            div {
                class: "improvement-header",
                onclick: move |_| show_details.set(!show_details()),

                div {
                    class: "improvement-title",
                    "{improvement.description}"
                }

                div {
                    class: "improvement-metrics",

                    div {
                        class: "expected-improvement",
                        "Expected: +{improvement.expected_improvement * 100.0:.1}%"
                    }

                    div {
                        class: "confidence-level",
                        "Confidence: {improvement.confidence * 100.0:.0}%"
                    }
                }

                div {
                    class: "expand-icon {if show_details() { \\\"expanded\\\" } else { \\\"\\\" }}",
                    "▼"
                }
            }

            if show_details() {
                div {
                    class: "improvement-details",

                    // Proposed changes
                    div {
                        class: "proposed-changes",
                        h5 { "Proposed Changes" }
                        for (helper, adjustment) in &improvement.proposed_changes {
                            div {
                                class: "change-item",
                                span { class: "helper-name", "{get_helper_display_name(helper)}" }
                                span {
                                    class: "adjustment {if *adjustment > 0.0 { \\\"increase\\\" } else { \\\"decrease\\\" }}",
                                    "{if *adjustment > 0.0 { \\\"+\\\" } else { \\\"\\\" }}{adjustment * 100.0:.1}%"
                                }
                            }
                        }
                    }

                    // Recommendation
                    div {
                        class: "recommendation",
                        h5 { "Recommendation" }
                        div {
                            class: "recommendation-text {get_recommendation_class(&improvement.recommendation)}",
                            "{format_recommendation(&improvement.recommendation)}"
                        }
                    }
                }
            }

            div {
                class: "improvement-actions",

                button {
                    class: "action-button secondary",
                    onclick: move |_| on_reject.call(improvement.experiment_id.clone()),
                    "Reject"
                }

                button {
                    class: "action-button primary",
                    onclick: move |_| on_approve.call(improvement.experiment_id.clone()),
                    "Approve"
                }
            }
        }
    }
}

/// Learning history section
#[component]
pub fn LearningHistorySection(cycles: Vec<LearningCycleResult>) -> Element {
    let show_history = use_signal(|| false);

    rsx! {
        div {
            class: "learning-history-section",

            div {
                class: "section-header",
                onclick: move |_| show_history.set(!show_history()),

                h3 { "Learning History ({cycles.len()} cycles)" }

                div {
                    class: "expand-icon {if show_history() { \\\"expanded\\\" } else { \\\"\\\" }}",
                    "▼"
                }
            }

            if show_history() {
                div {
                    class: "history-timeline",
                    for cycle in cycles.iter().rev().take(10) {
                        LearningCycleCard {
                            cycle: cycle.clone()
                        }
                    }
                }
            }
        }
    }
}

/// Learning cycle result card
#[component]
pub fn LearningCycleCard(cycle: LearningCycleResult) -> Element {
    rsx! {
        div {
            class: "learning-cycle-card",

            div {
                class: "cycle-header",

                div {
                    class: "cycle-number",
                    "Cycle #{cycle.cycle_number}"
                }

                div {
                    class: "cycle-timestamp",
                    "{cycle.timestamp.format(\"%Y-%m-%d %H:%M\")}"
                }

                div {
                    class: "cycle-duration",
                    "{cycle.duration_ms}ms"
                }
            }

            div {
                class: "cycle-metrics",

                div {
                    class: "experiments-count",
                    "🧪 {cycle.experiments_conducted.len()} experiments"
                }

                div {
                    class: "improvements-count",
                    "✅ {cycle.improvements_applied.len()} improvements"
                }

                div {
                    class: "performance-change {if cycle.performance_change > 0.0 { \\\"positive\\\" } else if cycle.performance_change < 0.0 { \\\"negative\\\" } else { \\\"neutral\\\" }}",
                    "{if cycle.performance_change > 0.0 { \\\"+\\\" } else { \\\"\\\" }}{cycle.performance_change * 100.0:.1}%"
                }
            }

            div {
                class: "cycle-summary",

                div {
                    class: "success-rate",
                    "Success Rate: {cycle.insights.recent_success_rate * 100.0:.1}%"
                }

                div {
                    class: "error-patterns",
                    "Error Patterns: {cycle.insights.error_patterns.len()}"
                }
            }
        }
    }
}

/// Mini trend chart for helper cards
#[component]
pub fn MiniTrendChart(data: Vec<f32>) -> Element {
    let points = generate_trend_points(&data, 40, 20);

    rsx! {
        div {
            class: "mini-trend-chart",

            svg {
                class: "trend-svg",
                viewBox: "0 0 40 20",

                polyline {
                    class: "trend-line",
                    points: "{points}",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "1",
                    opacity: "0.8"
                }
            }
        }
    }
}

/// Detailed trend chart for helper detail panel
#[component]
pub fn DetailedTrendChart(data: Vec<f32>, helper_name: String) -> Element {
    let points = generate_trend_points(&data, 300, 100);

    rsx! {
        div {
            class: "detailed-trend-chart",

            svg {
                class: "detailed-trend-svg",
                viewBox: "0 0 300 100",

                // Grid lines
                for i in 0..=4 {
                    line {
                        x1: "0",
                        y1: "{i * 25}",
                        x2: "300",
                        y2: "{i * 25}",
                        stroke: "#333",
                        stroke_width: "0.5",
                        opacity: "0.3"
                    }
                }

                for i in 0..=10 {
                    line {
                        x1: "{i * 30}",
                        y1: "0",
                        x2: "{i * 30}",
                        y2: "100",
                        stroke: "#333",
                        stroke_width: "0.5",
                        opacity: "0.3"
                    }
                }

                // Trend line
                polyline {
                    class: "detailed-trend-line",
                    points: "{points}",
                    fill: "none",
                    stroke: "#007ACC",
                    stroke_width: "2"
                }

                // Data points
                for (i, value) in data.iter().enumerate() {
                    circle {
                        cx: "{i as f32 * 300.0 / (data.len() - 1).max(1) as f32}",
                        cy: "{100.0 - (value * 100.0)}",
                        r: "2",
                        fill: "#007ACC"
                    }
                }
            }

            div {
                class: "chart-legend",
                "Performance trend for {get_helper_display_name(&helper_name)}"
            }
        }
    }
}

// Helper functions

fn get_trend_class(trend: &LearningTrend) -> &'static str {
    match trend {
        LearningTrend::Improving => "improving",
        LearningTrend::Stable => "stable",
        LearningTrend::Declining => "declining",
        LearningTrend::Insufficient => "insufficient",
    }
}

fn get_trend_icon(trend: &LearningTrend) -> &'static str {
    match trend {
        LearningTrend::Improving => "📈",
        LearningTrend::Stable => "📊",
        LearningTrend::Declining => "📉",
        LearningTrend::Insufficient => "❓",
    }
}

fn calculate_success_rate(status: &LearningSystemStatus) -> f32 {
    if status.total_cycles > 0 {
        status.successful_improvements as f32 / status.total_cycles as f32
    } else {
        0.0
    }
}

fn get_accuracy_trend(trend: &LearningTrend) -> f32 {
    match trend {
        LearningTrend::Improving => 0.1,
        LearningTrend::Stable => 0.0,
        LearningTrend::Declining => -0.1,
        LearningTrend::Insufficient => 0.0,
    }
}

fn get_accuracy_class(accuracy: f32) -> &'static str {
    match accuracy {
        a if a >= 0.9 => "excellent",
        a if a >= 0.8 => "good",
        a if a >= 0.7 => "fair",
        a if a >= 0.6 => "poor",
        _ => "critical",
    }
}

fn get_accuracy_color(accuracy: f32) -> &'static str {
    match accuracy {
        a if a >= 0.9 => "#4CAF50",
        a if a >= 0.8 => "#8BC34A",
        a if a >= 0.7 => "#CDDC39",
        a if a >= 0.6 => "#FFC107",
        _ => "#F44336",
    }
}

fn get_correlation_color(correlation: f32) -> &'static str {
    match correlation {
        c if c >= 0.7 => "#4CAF50",
        c if c >= 0.3 => "#8BC34A",
        c if c >= 0.0 => "#FFC107",
        c if c >= -0.3 => "#FF9800",
        _ => "#F44336",
    }
}

fn get_weight_color(weight: f32) -> &'static str {
    match weight {
        w if w >= 1.5 => "#2196F3",
        w if w >= 1.1 => "#4CAF50",
        w if w >= 0.9 => "#8BC34A",
        w if w >= 0.7 => "#FF9800",
        _ => "#F44336",
    }
}

fn get_adjustment_icon(adjustment: f32) -> &'static str {
    if adjustment > 1.0 {
        "↗ Increase"
    } else if adjustment < 1.0 {
        "↘ Decrease"
    } else {
        "→ Maintain"
    }
}

fn get_helper_display_name(helper_name: &str) -> &'static str {
    match helper_name {
        "knowledge_indexer" => "Knowledge Indexer",
        "context_retriever" => "Context Retriever",
        "pattern_recognizer" => "Pattern Recognizer",
        "quality_analyzer" => "Quality Analyzer",
        "knowledge_synthesizer" => "Knowledge Synthesizer",
        _ => "Unknown Helper",
    }
}

fn get_helper_description(helper_name: &str) -> &'static str {
    match helper_name {
        "knowledge_indexer" => "Tracks file operations and analyzes similarity patterns to predict operation success based on historical data.",
        "context_retriever" => "Retrieves relevant historical context and success rates for similar operations to inform confidence scoring.",
        "pattern_recognizer" => "Detects safety patterns and anti-patterns in code to identify potentially risky operations before execution.",
        "quality_analyzer" => "Performs comprehensive risk assessment and conflict detection to evaluate operation safety and complexity.",
        "knowledge_synthesizer" => "Synthesizes insights from all helpers to generate operation plans, previews, and backup strategies.",
        _ => "No description available for this helper.",
    }
}

fn get_recommendation_class(recommendation: &ExperimentRecommendation) -> &'static str {
    match recommendation {
        ExperimentRecommendation::Apply => "apply",
        ExperimentRecommendation::ApplyWithModifications(_) => "apply-modified",
        ExperimentRecommendation::Reject => "reject",
        ExperimentRecommendation::ExtendExperiment => "extend",
    }
}

fn format_recommendation(recommendation: &ExperimentRecommendation) -> String {
    match recommendation {
        ExperimentRecommendation::Apply => "✅ Apply as proposed".to_string(),
        ExperimentRecommendation::ApplyWithModifications(_) => {
            "⚠️ Apply with modifications".to_string()
        }
        ExperimentRecommendation::Reject => "❌ Reject - insufficient improvement".to_string(),
        ExperimentRecommendation::ExtendExperiment => {
            "🔄 Extend experiment for more data".to_string()
        }
    }
}

fn generate_trend_points(data: &[f32], width: u32, height: u32) -> String {
    if data.is_empty() {
        return String::new();
    }

    let points: Vec<String> = data
        .iter()
        .enumerate()
        .map(|(i, &value)| {
            let x = i as f32 * width as f32 / (data.len() - 1).max(1) as f32;
            let y = height as f32 - (value * height as f32);
            format!("{:.1},{:.1}", x, y)
        })
        .collect();

    points.join(" ")
}
