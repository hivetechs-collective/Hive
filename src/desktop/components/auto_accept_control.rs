// Auto-Accept Control Bar Component for Dioxus Desktop
use crate::consensus::operation_history::OperationStatistics;
use crate::consensus::operation_intelligence::{AutoAcceptMode, OperationAnalysis, UnifiedScore};
use crate::consensus::smart_decision_engine::{DecisionMetrics, ExecutionDecision};
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Auto-accept control bar with AI insights display
#[component]
pub fn AutoAcceptControl(
    current_mode: Signal<AutoAcceptMode>,
    operation_analysis: Signal<Option<OperationAnalysis>>,
    decision_metrics: Signal<Option<DecisionMetrics>>,
    operation_stats: Signal<Option<OperationStatistics>>,
    on_mode_change: EventHandler<AutoAcceptMode>,
    on_feedback: EventHandler<UserFeedback>,
) -> Element {
    // Local state for UI
    let show_details = use_signal(|| false);
    let show_history = use_signal(|| false);
    let hover_mode = use_signal(|| None::<AutoAcceptMode>);

    // Keyboard shortcut handler (Shift+Tab)
    use_effect(move || {
        let mode = current_mode.read().clone();
        // Register keyboard handler logic would go here
        // For now, we'll handle it at the app level
    });

    rsx! {
        div {
            class: "auto-accept-control-bar",

            // Main control section
            div {
                class: "control-main",

                // Mode selector
                div {
                    class: "mode-selector",

                    // Current mode display
                    div {
                        class: "current-mode",
                        onclick: move |_| show_details.set(!show_details()),

                        // Mode icon
                        div {
                            class: "mode-icon {get_mode_icon_class(&current_mode())}",
                        }

                        // Mode label
                        div {
                            class: "mode-label",
                            "{get_mode_label(&current_mode())}"
                        }

                        // Confidence indicator
                        if let Some(analysis) = operation_analysis() {
                            div {
                                class: "confidence-indicator",
                                style: "background: {get_confidence_color(analysis.unified_score.confidence)}",
                                "{analysis.unified_score.confidence:.0}%"
                            }
                        }

                        // Expand/collapse arrow
                        div {
                            class: "expand-arrow {if show_details() { \"expanded\" } else { \"\" }}",
                            "▼"
                        }
                    }

                    // Mode options dropdown
                    if show_details() {
                        div {
                            class: "mode-options",

                            // Conservative mode
                            ModeOption {
                                mode: AutoAcceptMode::Conservative,
                                current_mode: current_mode(),
                                hover_mode: hover_mode,
                                on_select: move |mode| {
                                    on_mode_change.call(mode);
                                    show_details.set(false);
                                }
                            }

                            // Balanced mode
                            ModeOption {
                                mode: AutoAcceptMode::Balanced,
                                current_mode: current_mode(),
                                hover_mode: hover_mode,
                                on_select: move |mode| {
                                    on_mode_change.call(mode);
                                    show_details.set(false);
                                }
                            }

                            // Aggressive mode
                            ModeOption {
                                mode: AutoAcceptMode::Aggressive,
                                current_mode: current_mode(),
                                hover_mode: hover_mode,
                                on_select: move |mode| {
                                    on_mode_change.call(mode);
                                    show_details.set(false);
                                }
                            }

                            // Plan mode
                            ModeOption {
                                mode: AutoAcceptMode::Plan,
                                current_mode: current_mode(),
                                hover_mode: hover_mode,
                                on_select: move |mode| {
                                    on_mode_change.call(mode);
                                    show_details.set(false);
                                }
                            }

                            // Manual mode
                            ModeOption {
                                mode: AutoAcceptMode::Manual,
                                current_mode: current_mode(),
                                hover_mode: hover_mode,
                                on_select: move |mode| {
                                    on_mode_change.call(mode);
                                    show_details.set(false);
                                }
                            }
                        }
                    }
                }

                // AI insights section
                div {
                    class: "ai-insights",

                    if let Some(analysis) = operation_analysis() {
                        // Risk indicator
                        div {
                            class: "risk-indicator",
                            title: "Risk Level",

                            div {
                                class: "risk-icon {get_risk_class(analysis.unified_score.risk)}",
                            }

                            span {
                                class: "risk-label",
                                "{format!("{:.0}%", analysis.unified_score.risk)}"
                            }
                        }

                        // Primary recommendation
                        div {
                            class: "primary-recommendation",
                            title: "AI Recommendation",

                            if let Some(rec) = analysis.recommendations.first() {
                                span {
                                    class: "recommendation-text",
                                    "{rec.description}"
                                }
                            }
                        }

                        // Decision indicator
                        if let Some(metrics) = decision_metrics() {
                            div {
                                class: "decision-indicator",
                                title: "Decision",

                                match &metrics.decision {
                                    ExecutionDecision::AutoExecute { reason, .. } => rsx! {
                                        div {
                                            class: "decision-auto",
                                            "✓ Auto"
                                        }
                                    },
                                    ExecutionDecision::RequireConfirmation { .. } => rsx! {
                                        div {
                                            class: "decision-confirm",
                                            "? Confirm"
                                        }
                                    },
                                    ExecutionDecision::Block { .. } => rsx! {
                                        div {
                                            class: "decision-block",
                                            "✗ Block"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Quick stats
                div {
                    class: "quick-stats",
                    onclick: move |_| show_history.set(!show_history()),

                    if let Some(stats) = operation_stats() {
                        div {
                            class: "stat",
                            title: "Success Rate",

                            span { class: "stat-value", "{format!("{:.0}%", stats.success_rate * 100.0)}" }
                            span { class: "stat-label", "Success" }
                        }

                        div {
                            class: "stat",
                            title: "Operations Today",

                            span { class: "stat-value", "{stats.operations_today}" }
                            span { class: "stat-label", "Today" }
                        }

                        div {
                            class: "stat",
                            title: "Auto-Accept Rate",

                            span { class: "stat-value", "{format!("{:.0}%", stats.auto_accept_rate * 100.0)}" }
                            span { class: "stat-label", "Auto" }
                        }
                    }

                    div {
                        class: "history-arrow {if show_history() { \"expanded\" } else { \"\" }}",
                        "▼"
                    }
                }

                // Keyboard shortcut hint
                div {
                    class: "keyboard-hint",
                    title: "Press Shift+Tab to cycle modes",
                    "⇧⇥"
                }
            }

            // Detailed insights panel (expandable)
            if show_details() {
                AIInsightsPanel {
                    analysis: operation_analysis(),
                    metrics: decision_metrics(),
                    on_feedback: on_feedback
                }
            }

            // History panel (expandable)
            if show_history() {
                OperationHistoryPanel {
                    stats: operation_stats()
                }
            }
        }
    }
}

/// Individual mode option component
#[component]
fn ModeOption(
    mode: AutoAcceptMode,
    current_mode: AutoAcceptMode,
    hover_mode: Signal<Option<AutoAcceptMode>>,
    on_select: EventHandler<AutoAcceptMode>,
) -> Element {
    let is_current = mode == current_mode;
    let is_hovered = hover_mode() == Some(mode);

    rsx! {
        div {
            class: "mode-option {if is_current { \"current\" } else { \"\" }} {if is_hovered { \"hovered\" } else { \"\" }}",
            onmouseenter: move |_| hover_mode.set(Some(mode)),
            onmouseleave: move |_| hover_mode.set(None),
            onclick: move |_| on_select.call(mode),

            // Mode icon
            div {
                class: "mode-icon {get_mode_icon_class(&mode)}",
            }

            // Mode info
            div {
                class: "mode-info",

                div {
                    class: "mode-name",
                    "{get_mode_label(&mode)}"
                }

                div {
                    class: "mode-description",
                    "{get_mode_description(&mode)}"
                }

                div {
                    class: "mode-thresholds",
                    "{get_mode_thresholds(&mode)}"
                }
            }

            // Current indicator
            if is_current {
                div {
                    class: "current-indicator",
                    "✓"
                }
            }
        }
    }
}

/// AI insights panel with detailed information
#[component]
fn AIInsightsPanel(
    analysis: Option<OperationAnalysis>,
    metrics: Option<DecisionMetrics>,
    on_feedback: EventHandler<UserFeedback>,
) -> Element {
    rsx! {
        div {
            class: "ai-insights-panel",

            if let Some(analysis) = analysis {
                // Unified scores
                div {
                    class: "score-section",

                    h4 { "AI Analysis Scores" }

                    ScoreBar {
                        label: "Confidence",
                        value: analysis.unified_score.confidence,
                        color: get_confidence_color(analysis.unified_score.confidence)
                    }

                    ScoreBar {
                        label: "Risk",
                        value: analysis.unified_score.risk,
                        color: get_risk_color(analysis.unified_score.risk)
                    }

                    // Component scores
                    if let Some(scores) = &analysis.unified_score.component_scores {
                        div {
                            class: "component-scores",

                            h5 { "AI Helper Contributions" }

                            ComponentScore { label: "Knowledge", value: scores.knowledge_indexer }
                            ComponentScore { label: "Context", value: scores.context_retriever }
                            ComponentScore { label: "Patterns", value: scores.pattern_recognizer }
                            ComponentScore { label: "Quality", value: scores.quality_analyzer }
                            ComponentScore { label: "Synthesis", value: scores.knowledge_synthesizer }
                        }
                    }
                }

                // Recommendations
                div {
                    class: "recommendations-section",

                    h4 { "AI Recommendations" }

                    for rec in &analysis.recommendations {
                        RecommendationCard {
                            recommendation: rec.clone(),
                            on_feedback: on_feedback
                        }
                    }
                }

                // Decision explanation
                if let Some(metrics) = metrics {
                    div {
                        class: "decision-section",

                        h4 { "Decision Explanation" }

                        match &metrics.decision {
                            ExecutionDecision::AutoExecute { reason, confidence, risk_level } => rsx! {
                                div {
                                    class: "decision-explanation auto-execute",

                                    div { class: "decision-type", "✓ Auto-Execute" }
                                    div { class: "decision-reason", "{reason}" }
                                    div { class: "decision-stats",
                                        "Confidence: {confidence:.0}% | Risk: {risk_level:.0}%"
                                    }
                                }
                            },
                            ExecutionDecision::RequireConfirmation { reason, warnings, suggestions, .. } => rsx! {
                                div {
                                    class: "decision-explanation require-confirmation",

                                    div { class: "decision-type", "? Confirmation Required" }
                                    div { class: "decision-reason", "{reason}" }

                                    if !warnings.is_empty() {
                                        div {
                                            class: "decision-warnings",
                                            h5 { "⚠️ Warnings" }
                                            for warning in warnings {
                                                div { class: "warning-item", "• {warning}" }
                                            }
                                        }
                                    }

                                    if !suggestions.is_empty() {
                                        div {
                                            class: "decision-suggestions",
                                            h5 { "💡 Suggestions" }
                                            for suggestion in suggestions {
                                                div { class: "suggestion-item", "• {suggestion}" }
                                            }
                                        }
                                    }
                                }
                            },
                            ExecutionDecision::Block { reason, critical_issues, alternatives, .. } => rsx! {
                                div {
                                    class: "decision-explanation block",

                                    div { class: "decision-type", "✗ Blocked" }
                                    div { class: "decision-reason", "{reason}" }

                                    if !critical_issues.is_empty() {
                                        div {
                                            class: "decision-critical",
                                            h5 { "🚨 Critical Issues" }
                                            for issue in critical_issues {
                                                div { class: "critical-item", "• {issue}" }
                                            }
                                        }
                                    }

                                    if !alternatives.is_empty() {
                                        div {
                                            class: "decision-alternatives",
                                            h5 { "🔄 Alternatives" }
                                            for alt in alternatives {
                                                div { class: "alternative-item", "• {alt}" }
                                            }
                                        }
                                    }
                                }
                            }
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

/// Operation history panel
#[component]
fn OperationHistoryPanel(stats: Option<OperationStatistics>) -> Element {
    rsx! {
        div {
            class: "operation-history-panel",

            if let Some(stats) = stats {
                div {
                    class: "stats-grid",

                    StatCard {
                        title: "Total Operations",
                        value: stats.total_operations.to_string(),
                        subtext: format!("{} today", stats.operations_today)
                    }

                    StatCard {
                        title: "Success Rate",
                        value: format!("{:.1}%", stats.success_rate * 100.0),
                        subtext: format!("{} successful", stats.successful_operations)
                    }

                    StatCard {
                        title: "Auto-Accept Rate",
                        value: format!("{:.1}%", stats.auto_accept_rate * 100.0),
                        subtext: format!("{} auto-executed", stats.auto_executed_count)
                    }

                    StatCard {
                        title: "Avg Confidence",
                        value: format!("{:.1}%", stats.average_confidence * 100.0),
                        subtext: "When auto-accepting"
                    }

                    StatCard {
                        title: "Avg Risk",
                        value: format!("{:.1}%", stats.average_risk * 100.0),
                        subtext: "Across all operations"
                    }

                    StatCard {
                        title: "User Overrides",
                        value: stats.user_override_count.to_string(),
                        subtext: format!("{:.1}% of decisions",
                            (stats.user_override_count as f32 / stats.total_operations as f32) * 100.0)
                    }
                }

                // Operation type breakdown
                div {
                    class: "operation-breakdown",

                    h5 { "Operations by Type" }

                    for (op_type, count) in &stats.by_operation_type {
                        div {
                            class: "operation-type-stat",

                            span { class: "op-type", "{op_type}" }
                            span { class: "op-count", "{count}" }

                            div {
                                class: "op-bar",
                                style: "width: {((*count as f32 / stats.total_operations as f32) * 100.0)}%"
                            }
                        }
                    }
                }

                // Mode usage
                div {
                    class: "mode-usage",

                    h5 { "Auto-Accept Mode Usage" }

                    for (mode, count) in &stats.by_auto_accept_mode {
                        div {
                            class: "mode-usage-stat",

                            span { class: "mode-name", "{mode}" }
                            span { class: "mode-count", "{count}" }

                            div {
                                class: "mode-bar",
                                style: "width: {((*count as f32 / stats.total_operations as f32) * 100.0)}%"
                            }
                        }
                    }
                }
            } else {
                div {
                    class: "no-stats",
                    "No operation history available"
                }
            }
        }
    }
}

/// Score bar component
#[component]
fn ScoreBar(label: &'static str, value: f32, color: &'static str) -> Element {
    rsx! {
        div {
            class: "score-bar",

            div {
                class: "score-label",
                "{label}"
            }

            div {
                class: "score-track",

                div {
                    class: "score-fill",
                    style: "width: {value}%; background: {color}",
                }
            }

            div {
                class: "score-value",
                "{value:.0}%"
            }
        }
    }
}

/// Component score display
#[component]
fn ComponentScore(label: &'static str, value: Option<f32>) -> Element {
    rsx! {
        div {
            class: "component-score",

            span { class: "comp-label", "{label}:" }

            if let Some(v) = value {
                span { class: "comp-value", "{v:.0}%" }
            } else {
                span { class: "comp-value none", "N/A" }
            }
        }
    }
}

/// Recommendation card
#[component]
fn RecommendationCard(
    recommendation: crate::consensus::operation_analysis::ActionRecommendation,
    on_feedback: EventHandler<UserFeedback>,
) -> Element {
    let show_details = use_signal(|| false);

    rsx! {
        div {
            class: "recommendation-card priority-{recommendation.priority:?}",
            onclick: move |_| show_details.set(!show_details()),

            div {
                class: "rec-header",

                div {
                    class: "rec-action",
                    "{recommendation.action}"
                }

                div {
                    class: "rec-confidence",
                    "{recommendation.confidence:.0}%"
                }
            }

            div {
                class: "rec-description",
                "{recommendation.description}"
            }

            if show_details() && !recommendation.rationale.is_empty() {
                div {
                    class: "rec-details",

                    div {
                        class: "rec-rationale",

                        h6 { "Rationale" }
                        for reason in &recommendation.rationale {
                            div { class: "rationale-item", "• {reason}" }
                        }
                    }

                    if !recommendation.risks.is_empty() {
                        div {
                            class: "rec-risks",

                            h6 { "Risks" }
                            for risk in &recommendation.risks {
                                div { class: "risk-item", "• {risk}" }
                            }
                        }
                    }

                    // Feedback buttons
                    div {
                        class: "rec-feedback",

                        button {
                            class: "feedback-btn helpful",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_feedback.call(UserFeedback {
                                    recommendation_id: recommendation.id.clone(),
                                    was_helpful: true,
                                    comment: None,
                                });
                            },
                            "👍 Helpful"
                        }

                        button {
                            class: "feedback-btn not-helpful",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_feedback.call(UserFeedback {
                                    recommendation_id: recommendation.id.clone(),
                                    was_helpful: false,
                                    comment: None,
                                });
                            },
                            "👎 Not Helpful"
                        }
                    }
                }
            }
        }
    }
}

/// Stat card component
#[component]
fn StatCard(title: &'static str, value: String, subtext: String) -> Element {
    rsx! {
        div {
            class: "stat-card",

            div { class: "stat-title", "{title}" }
            div { class: "stat-main-value", "{value}" }
            div { class: "stat-subtext", "{subtext}" }
        }
    }
}

/// User feedback structure
#[derive(Debug, Clone)]
pub struct UserFeedback {
    pub recommendation_id: String,
    pub was_helpful: bool,
    pub comment: Option<String>,
}

// Helper functions

fn get_mode_icon_class(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Conservative => "icon-shield",
        AutoAcceptMode::Balanced => "icon-balance",
        AutoAcceptMode::Aggressive => "icon-rocket",
        AutoAcceptMode::Plan => "icon-plan",
        AutoAcceptMode::Manual => "icon-hand",
    }
}

fn get_mode_label(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Conservative => "Conservative",
        AutoAcceptMode::Balanced => "Balanced",
        AutoAcceptMode::Aggressive => "Aggressive",
        AutoAcceptMode::Plan => "Plan Only",
        AutoAcceptMode::Manual => "Manual",
    }
}

fn get_mode_description(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Conservative => "Only auto-accept very safe operations",
        AutoAcceptMode::Balanced => "Balance safety with automation",
        AutoAcceptMode::Aggressive => "Maximize automation, accept more risk",
        AutoAcceptMode::Plan => "Generate plans but don't execute",
        AutoAcceptMode::Manual => "Require confirmation for all operations",
    }
}

fn get_mode_thresholds(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Conservative => "Confidence >90%, Risk <15%",
        AutoAcceptMode::Balanced => "Confidence >80%, Risk <25%",
        AutoAcceptMode::Aggressive => "Confidence >70%, Risk <40%",
        AutoAcceptMode::Plan => "Planning only",
        AutoAcceptMode::Manual => "All manual",
    }
}

fn get_confidence_color(confidence: f32) -> &'static str {
    match confidence {
        c if c >= 90.0 => "#4CAF50", // Green
        c if c >= 80.0 => "#8BC34A", // Light green
        c if c >= 70.0 => "#FFC107", // Amber
        c if c >= 60.0 => "#FF9800", // Orange
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

fn get_risk_class(risk: f32) -> &'static str {
    match risk {
        r if r <= 15.0 => "risk-low",
        r if r <= 25.0 => "risk-medium-low",
        r if r <= 40.0 => "risk-medium",
        r if r <= 60.0 => "risk-high",
        _ => "risk-critical",
    }
}
