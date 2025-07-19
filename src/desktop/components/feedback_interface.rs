// User Feedback Interfaces for Operation Confirmations
use dioxus::prelude::*;
use crate::consensus::operation_intelligence::{OperationAnalysis, AutoAcceptMode};
use crate::consensus::smart_decision_engine::{ExecutionDecision, UserDecision, UserChoice};
use crate::consensus::operation_preview::OperationPreview;
use crate::consensus::rollback_planner::RollbackPlan;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Main user feedback interface for operation confirmations
#[component]
pub fn OperationConfirmationDialog(
    operation_analysis: Signal<Option<OperationAnalysis>>,
    execution_decision: Signal<Option<ExecutionDecision>>,
    rollback_plan: Signal<Option<RollbackPlan>>,
    operation_preview: Signal<Option<OperationPreview>>,
    show_dialog: Signal<bool>,
    on_user_decision: EventHandler<UserDecision>,
    on_feedback_submitted: EventHandler<OperationFeedback>,
) -> Element {
    let user_choice = use_signal(|| UserChoice::Pending);
    let feedback_comment = use_signal(|| String::new());
    let show_details = use_signal(|| false);
    let show_rollback_info = use_signal(|| false);
    let trust_level_adjustment = use_signal(|| 0i32);
    
    rsx! {
        if show_dialog() {
            div {
                class: "confirmation-dialog-overlay",
                onclick: move |e| {
                    if e.target() == e.current_target() {
                        show_dialog.set(false);
                    }
                },
                
                div {
                    class: "confirmation-dialog",
                    
                    // Dialog header
                    div {
                        class: "dialog-header",
                        
                        h2 { 
                            class: "dialog-title",
                            "Operation Confirmation Required"
                        }
                        
                        if let Some(decision) = execution_decision() {
                            DecisionSummary {
                                decision: decision.clone()
                            }
                        }
                        
                        button {
                            class: "close-button",
                            onclick: move |_| show_dialog.set(false),
                            "×"
                        }
                    }
                    
                    // Dialog content
                    div {
                        class: "dialog-content",
                        
                        // Operation analysis summary
                        if let Some(analysis) = operation_analysis() {
                            OperationAnalysisSummary {
                                analysis: analysis.clone(),
                                show_details: show_details
                            }
                        }
                        
                        // Operation preview
                        if let Some(preview) = operation_preview() {
                            OperationPreviewSection {
                                preview: preview.clone()
                            }
                        }
                        
                        // Rollback information
                        if let Some(plan) = rollback_plan() {
                            RollbackInformation {
                                plan: plan.clone(),
                                show_info: show_rollback_info
                            }
                        }
                        
                        // User decision options
                        UserDecisionOptions {
                            user_choice: user_choice,
                            trust_level_adjustment: trust_level_adjustment,
                            analysis: operation_analysis(),
                            decision: execution_decision()
                        }
                        
                        // Feedback section
                        FeedbackSection {
                            comment: feedback_comment,
                            trust_adjustment: trust_level_adjustment
                        }
                    }
                    
                    // Dialog actions
                    div {
                        class: "dialog-actions",
                        
                        button {
                            class: "action-button secondary",
                            onclick: move |_| show_dialog.set(false),
                            "Cancel"
                        }
                        
                        button {
                            class: "action-button danger",
                            onclick: move |_| {
                                let decision = UserDecision {
                                    choice: UserChoice::Reject,
                                    reason: feedback_comment().clone(),
                                    trust_adjustment: trust_level_adjustment(),
                                    timestamp: Utc::now(),
                                };
                                on_user_decision.call(decision);
                                show_dialog.set(false);
                            },
                            "Reject"
                        }
                        
                        button {
                            class: "action-button primary",
                            onclick: move |_| {
                                let decision = UserDecision {
                                    choice: user_choice(),
                                    reason: feedback_comment().clone(),
                                    trust_adjustment: trust_level_adjustment(),
                                    timestamp: Utc::now(),
                                };
                                on_user_decision.call(decision);
                                
                                // Submit feedback
                                let feedback = OperationFeedback {
                                    operation_id: String::new(), // Would be set by parent
                                    user_decision: decision.choice.clone(),
                                    satisfaction_rating: calculate_satisfaction_rating(user_choice()),
                                    was_helpful: matches!(user_choice(), UserChoice::Accept | UserChoice::AcceptWithModifications),
                                    comment: feedback_comment().clone(),
                                    trust_adjustment: trust_level_adjustment(),
                                    submitted_at: Utc::now(),
                                };
                                on_feedback_submitted.call(feedback);
                                
                                show_dialog.set(false);
                            },
                            disabled: user_choice() == UserChoice::Pending,
                            "{get_action_button_text(user_choice())}"
                        }
                    }
                }
            }
        }
    }
}

/// Decision summary component
#[component]
fn DecisionSummary(decision: ExecutionDecision) -> Element {
    rsx! {
        div {
            class: "decision-summary",
            
            match &decision {
                ExecutionDecision::RequireConfirmation { reason, warnings, suggestions, confidence, risk_level } => rsx! {
                    div {
                        class: "decision-require-confirmation",
                        
                        div {
                            class: "decision-reason",
                            "⚠️ {reason}"
                        }
                        
                        div {
                            class: "decision-metrics",
                            "Confidence: {confidence:.0}% | Risk: {risk_level:.0}%"
                        }
                        
                        if !warnings.is_empty() {
                            div {
                                class: "decision-warnings",
                                h4 { "Warnings:" }
                                for warning in warnings {
                                    div { class: "warning-item", "• {warning}" }
                                }
                            }
                        }
                        
                        if !suggestions.is_empty() {
                            div {
                                class: "decision-suggestions",
                                h4 { "Suggestions:" }
                                for suggestion in suggestions {
                                    div { class: "suggestion-item", "• {suggestion}" }
                                }
                            }
                        }
                    }
                },
                ExecutionDecision::Block { reason, critical_issues, alternatives, risk_level } => rsx! {
                    div {
                        class: "decision-block",
                        
                        div {
                            class: "decision-reason",
                            "🚫 {reason}"
                        }
                        
                        div {
                            class: "decision-metrics",
                            "Risk: {risk_level:.0}% (Critical)"
                        }
                        
                        if !critical_issues.is_empty() {
                            div {
                                class: "critical-issues",
                                h4 { "Critical Issues:" }
                                for issue in critical_issues {
                                    div { class: "critical-item", "• {issue}" }
                                }
                            }
                        }
                        
                        if !alternatives.is_empty() {
                            div {
                                class: "alternatives",
                                h4 { "Alternatives:" }
                                for alt in alternatives {
                                    div { class: "alternative-item", "• {alt}" }
                                }
                            }
                        }
                    }
                },
                _ => rsx! {
                    div { "Unexpected decision state" }
                }
            }
        }
    }
}

/// Operation analysis summary
#[component]
fn OperationAnalysisSummary(
    analysis: OperationAnalysis,
    show_details: Signal<bool>,
) -> Element {
    rsx! {
        div {
            class: "operation-analysis-summary",
            
            div {
                class: "analysis-header",
                onclick: move |_| show_details.set(!show_details()),
                
                h3 { "AI Analysis" }
                
                div {
                    class: "analysis-scores",
                    
                    div {
                        class: "score-item confidence",
                        span { class: "score-label", "Confidence:" }
                        span { class: "score-value", "{analysis.unified_score.confidence:.0}%" }
                    }
                    
                    div {
                        class: "score-item risk",
                        span { class: "score-label", "Risk:" }
                        span { class: "score-value", "{analysis.unified_score.risk:.0}%" }
                    }
                }
                
                div {
                    class: "expand-icon {if show_details() { \"expanded\" } else { \"\" }}",
                    "▼"
                }
            }
            
            if show_details() {
                div {
                    class: "analysis-details",
                    
                    // AI Recommendations
                    if !analysis.recommendations.is_empty() {
                        div {
                            class: "recommendations-section",
                            
                            h4 { "AI Recommendations" }
                            
                            for rec in &analysis.recommendations {
                                div {
                                    class: "recommendation-item priority-{rec.priority:?}",
                                    
                                    div {
                                        class: "rec-header",
                                        span { class: "rec-action", "{rec.action}" }
                                        span { class: "rec-confidence", "{rec.confidence:.0}%" }
                                    }
                                    
                                    div {
                                        class: "rec-description",
                                        "{rec.description}"
                                    }
                                    
                                    if !rec.rationale.is_empty() {
                                        div {
                                            class: "rec-rationale",
                                            strong { "Why: " }
                                            {rec.rationale.join(", ")}
                                        }
                                    }
                                    
                                    if !rec.risks.is_empty() {
                                        div {
                                            class: "rec-risks",
                                            strong { "Risks: " }
                                            {rec.risks.join(", ")}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Component Scores
                    if let Some(scores) = &analysis.unified_score.component_scores {
                        div {
                            class: "component-scores-section",
                            
                            h4 { "AI Helper Contributions" }
                            
                            div {
                                class: "component-grid",
                                
                                ComponentScoreDisplay {
                                    name: "Knowledge",
                                    score: scores.knowledge_indexer,
                                    description: "File operation tracking"
                                }
                                
                                ComponentScoreDisplay {
                                    name: "Context",
                                    score: scores.context_retriever,
                                    description: "Historical patterns"
                                }
                                
                                ComponentScoreDisplay {
                                    name: "Patterns",
                                    score: scores.pattern_recognizer,
                                    description: "Safety detection"
                                }
                                
                                ComponentScoreDisplay {
                                    name: "Quality",
                                    score: scores.quality_analyzer,
                                    description: "Risk assessment"
                                }
                                
                                ComponentScoreDisplay {
                                    name: "Synthesis",
                                    score: scores.knowledge_synthesizer,
                                    description: "Planning & backup"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Component score display
#[component]
fn ComponentScoreDisplay(
    name: &'static str,
    score: Option<f32>,
    description: &'static str,
) -> Element {
    rsx! {
        div {
            class: "component-score-display",
            title: "{description}",
            
            div {
                class: "component-name",
                "{name}"
            }
            
            div {
                class: "component-score",
                if let Some(s) = score {
                    span { class: "score-value", "{s:.0}%" }
                    div {
                        class: "score-bar",
                        div {
                            class: "score-fill",
                            style: "width: {s}%"
                        }
                    }
                } else {
                    span { class: "score-none", "N/A" }
                }
            }
        }
    }
}

/// Operation preview section
#[component]
fn OperationPreviewSection(preview: OperationPreview) -> Element {
    let show_diff = use_signal(|| false);
    
    rsx! {
        div {
            class: "operation-preview-section",
            
            div {
                class: "preview-header",
                onclick: move |_| show_diff.set(!show_diff()),
                
                h3 { "Operation Preview" }
                
                div {
                    class: "operation-info",
                    span { class: "operation-type", "{get_operation_type(&preview.operation)}" }
                    span { class: "operation-file", "{get_operation_file(&preview.operation)}" }
                }
                
                div {
                    class: "expand-icon {if show_diff() { \"expanded\" } else { \"\" }}",
                    "▼"
                }
            }
            
            if show_diff() {
                div {
                    class: "preview-content",
                    
                    // Before/After states
                    div {
                        class: "state-comparison",
                        
                        div {
                            class: "state-section before",
                            h4 { "Before" }
                            
                            if preview.before_state.exists {
                                div {
                                    class: "file-info",
                                    "Size: {preview.before_state.size_bytes.unwrap_or(0)} bytes"
                                }
                                
                                if let Some(content) = &preview.before_state.content_preview {
                                    pre {
                                        class: "content-preview",
                                        "{content}"
                                    }
                                }
                            } else {
                                div {
                                    class: "file-not-exists",
                                    "File does not exist"
                                }
                            }
                        }
                        
                        div {
                            class: "state-section after",
                            h4 { "After" }
                            
                            if preview.after_state.exists {
                                div {
                                    class: "file-info",
                                    "Size: {preview.after_state.size_bytes.unwrap_or(0)} bytes"
                                }
                                
                                if let Some(content) = &preview.after_state.content_preview {
                                    pre {
                                        class: "content-preview",
                                        "{content}"
                                    }
                                }
                            } else {
                                div {
                                    class: "file-not-exists",
                                    "File will be deleted"
                                }
                            }
                        }
                    }
                    
                    // Diff view
                    if let Some(diff) = &preview.diff_view {
                        div {
                            class: "diff-view",
                            
                            h4 { "Changes" }
                            
                            for chunk in &diff.chunks {
                                div {
                                    class: "diff-chunk",
                                    
                                    div {
                                        class: "chunk-header",
                                        "@@ -{},{} +{},{} @@",
                                        chunk.old_start,
                                        chunk.old_lines,
                                        chunk.new_start,
                                        chunk.new_lines
                                    }
                                    
                                    for line in &chunk.lines {
                                        div {
                                            class: "diff-line {line.change_type:?}",
                                            "{line.content}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Impact summary
                    div {
                        class: "impact-summary",
                        
                        h4 { "Impact Analysis" }
                        
                        div {
                            class: "impact-metrics",
                            
                            div {
                                class: "impact-item",
                                span { class: "impact-label", "Confidence:" }
                                span { class: "impact-value", "{preview.confidence:.0}%" }
                            }
                            
                            if let Some(explanation) = &preview.ai_explanation {
                                div {
                                    class: "ai-explanation",
                                    strong { "AI Analysis: " }
                                    "{explanation}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Rollback information section
#[component]
fn RollbackInformation(
    plan: RollbackPlan,
    show_info: Signal<bool>,
) -> Element {
    rsx! {
        div {
            class: "rollback-information",
            
            div {
                class: "rollback-header",
                onclick: move |_| show_info.set(!show_info()),
                
                h3 { "Rollback Plan" }
                
                div {
                    class: "rollback-summary",
                    span { class: "strategy", "{get_strategy_name(&plan.strategy)}" }
                    span { class: "steps", "{plan.steps.len()} steps" }
                    span { class: "duration", "~{plan.estimated_duration_ms}ms" }
                }
                
                div {
                    class: "expand-icon {if show_info() { \"expanded\" } else { \"\" }}",
                    "▼"
                }
            }
            
            if show_info() {
                div {
                    class: "rollback-details",
                    
                    // Risk assessment
                    div {
                        class: "risk-assessment",
                        
                        h4 { "Risk Assessment" }
                        
                        div {
                            class: "risk-level {plan.risk_assessment.risk_level:?}",
                            "Risk Level: {plan.risk_assessment.risk_level:?}"
                        }
                        
                        div {
                            class: "success-probability",
                            "Success Probability: {plan.risk_assessment.success_probability:.0}%"
                        }
                        
                        if !plan.risk_assessment.risks.is_empty() {
                            div {
                                class: "identified-risks",
                                h5 { "Identified Risks" }
                                for risk in &plan.risk_assessment.risks {
                                    div {
                                        class: "risk-item",
                                        "• {risk.description}"
                                    }
                                }
                            }
                        }
                        
                        if !plan.risk_assessment.mitigations.is_empty() {
                            div {
                                class: "mitigations",
                                h5 { "Mitigation Strategies" }
                                for mitigation in &plan.risk_assessment.mitigations {
                                    div {
                                        class: "mitigation-item",
                                        "• {mitigation}"
                                    }
                                }
                            }
                        }
                    }
                    
                    // Rollback steps
                    div {
                        class: "rollback-steps",
                        
                        h4 { "Rollback Steps" }
                        
                        for step in plan.steps.iter().take(5) {
                            div {
                                class: "rollback-step",
                                
                                div {
                                    class: "step-number",
                                    "{step.step_number}"
                                }
                                
                                div {
                                    class: "step-info",
                                    
                                    div {
                                        class: "step-description",
                                        "{step.description}"
                                    }
                                    
                                    div {
                                        class: "step-meta",
                                        span { class: "step-duration", "~{step.estimated_duration_ms}ms" }
                                        span { class: "step-risk", "{step.risk_level:?} risk" }
                                        if step.automatable {
                                            span { class: "step-auto", "Automated" }
                                        } else {
                                            span { class: "step-manual", "Manual" }
                                        }
                                    }
                                }
                            }
                        }
                        
                        if plan.steps.len() > 5 {
                            div {
                                class: "steps-truncated",
                                "... and {plan.steps.len() - 5} more steps"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// User decision options
#[component]
fn UserDecisionOptions(
    user_choice: Signal<UserChoice>,
    trust_level_adjustment: Signal<i32>,
    analysis: Option<OperationAnalysis>,
    decision: Option<ExecutionDecision>,
) -> Element {
    rsx! {
        div {
            class: "user-decision-options",
            
            h3 { "Your Decision" }
            
            div {
                class: "decision-buttons",
                
                DecisionButton {
                    choice: UserChoice::Accept,
                    current_choice: user_choice(),
                    label: "Accept",
                    description: "Proceed with the operation as proposed",
                    icon: "✓",
                    class: "accept",
                    onclick: move |_| user_choice.set(UserChoice::Accept)
                }
                
                DecisionButton {
                    choice: UserChoice::AcceptWithModifications,
                    current_choice: user_choice(),
                    label: "Accept with Changes",
                    description: "Accept but modify the operation details",
                    icon: "✎",
                    class: "modify",
                    onclick: move |_| user_choice.set(UserChoice::AcceptWithModifications)
                }
                
                DecisionButton {
                    choice: UserChoice::Defer,
                    current_choice: user_choice(),
                    label: "Defer",
                    description: "Review later or get more information",
                    icon: "⏸",
                    class: "defer",
                    onclick: move |_| user_choice.set(UserChoice::Defer)
                }
                
                DecisionButton {
                    choice: UserChoice::Reject,
                    current_choice: user_choice(),
                    label: "Reject",
                    description: "Do not perform this operation",
                    icon: "✗",
                    class: "reject",
                    onclick: move |_| user_choice.set(UserChoice::Reject)
                }
            }
            
            // Trust level adjustment
            div {
                class: "trust-adjustment",
                
                h4 { "Trust Level Adjustment" }
                
                div {
                    class: "trust-slider",
                    
                    input {
                        r#type: "range",
                        min: "-10",
                        max: "10",
                        value: "{trust_level_adjustment()}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<i32>() {
                                trust_level_adjustment.set(val);
                            }
                        }
                    }
                    
                    div {
                        class: "trust-labels",
                        span { class: "trust-less", "Trust AI Less" }
                        span { class: "trust-current", "{trust_level_adjustment()}" }
                        span { class: "trust-more", "Trust AI More" }
                    }
                }
                
                div {
                    class: "trust-explanation",
                    "{get_trust_explanation(trust_level_adjustment())}"
                }
            }
        }
    }
}

/// Individual decision button
#[component]
fn DecisionButton(
    choice: UserChoice,
    current_choice: UserChoice,
    label: &'static str,
    description: &'static str,
    icon: &'static str,
    class: &'static str,
    onclick: EventHandler<()>,
) -> Element {
    let is_selected = choice == current_choice;
    
    rsx! {
        button {
            class: "decision-button {class} {if is_selected { \"selected\" } else { \"\" }}",
            onclick: move |_| onclick.call(()),
            title: "{description}",
            
            div {
                class: "button-icon",
                "{icon}"
            }
            
            div {
                class: "button-content",
                
                div {
                    class: "button-label",
                    "{label}"
                }
                
                div {
                    class: "button-description",
                    "{description}"
                }
            }
            
            if is_selected {
                div {
                    class: "selected-indicator",
                    "●"
                }
            }
        }
    }
}

/// Feedback section
#[component]
fn FeedbackSection(
    comment: Signal<String>,
    trust_adjustment: Signal<i32>,
) -> Element {
    rsx! {
        div {
            class: "feedback-section",
            
            h3 { "Additional Feedback (Optional)" }
            
            textarea {
                class: "feedback-textarea",
                placeholder: "Share your thoughts on this AI recommendation...",
                value: "{comment()}",
                oninput: move |e| comment.set(e.value()),
                rows: "3"
            }
            
            div {
                class: "feedback-suggestions",
                "💡 Help us improve by sharing:"
                ul {
                    li { "Why you agree or disagree with the AI's assessment" }
                    li { "What additional information would be helpful" }
                    li { "How the AI could better explain its reasoning" }
                }
            }
        }
    }
}

/// Operation feedback data structure
#[derive(Debug, Clone)]
pub struct OperationFeedback {
    pub operation_id: String,
    pub user_decision: UserChoice,
    pub satisfaction_rating: u8,
    pub was_helpful: bool,
    pub comment: String,
    pub trust_adjustment: i32,
    pub submitted_at: DateTime<Utc>,
}

/// Quick feedback widget for successful operations
#[component]
pub fn QuickFeedbackWidget(
    operation_id: String,
    operation_success: bool,
    ai_confidence: f32,
    user_expected_success: bool,
    on_feedback: EventHandler<QuickFeedback>,
) -> Element {
    let show_widget = use_signal(|| true);
    let rating_given = use_signal(|| false);
    
    rsx! {
        if show_widget() && !rating_given() {
            div {
                class: "quick-feedback-widget {if operation_success { \"success\" } else { \"failure\" }}",
                
                div {
                    class: "feedback-prompt",
                    if operation_success {
                        "✅ Operation completed successfully! Was the AI's confidence assessment accurate?"
                    } else {
                        "❌ Operation failed. How could the AI have better predicted this?"
                    }
                }
                
                div {
                    class: "quick-rating",
                    
                    button {
                        class: "rating-btn positive",
                        onclick: move |_| {
                            let feedback = QuickFeedback {
                                operation_id: operation_id.clone(),
                                accurate_assessment: true,
                                helpful: true,
                                confidence_appropriate: ai_confidence >= 70.0,
                            };
                            on_feedback.call(feedback);
                            rating_given.set(true);
                        },
                        "👍 Accurate"
                    }
                    
                    button {
                        class: "rating-btn neutral",
                        onclick: move |_| {
                            let feedback = QuickFeedback {
                                operation_id: operation_id.clone(),
                                accurate_assessment: operation_success == user_expected_success,
                                helpful: false,
                                confidence_appropriate: false,
                            };
                            on_feedback.call(feedback);
                            rating_given.set(true);
                        },
                        "👎 Inaccurate"
                    }
                    
                    button {
                        class: "rating-btn dismiss",
                        onclick: move |_| show_widget.set(false),
                        "✕"
                    }
                }
            }
        }
    }
}

/// Quick feedback data structure
#[derive(Debug, Clone)]
pub struct QuickFeedback {
    pub operation_id: String,
    pub accurate_assessment: bool,
    pub helpful: bool,
    pub confidence_appropriate: bool,
}

// Helper functions

fn calculate_satisfaction_rating(choice: UserChoice) -> u8 {
    match choice {
        UserChoice::Accept => 5,
        UserChoice::AcceptWithModifications => 4,
        UserChoice::Defer => 3,
        UserChoice::Reject => 2,
        UserChoice::Pending => 1,
    }
}

fn get_action_button_text(choice: UserChoice) -> &'static str {
    match choice {
        UserChoice::Accept => "Accept",
        UserChoice::AcceptWithModifications => "Accept with Changes",
        UserChoice::Defer => "Defer",
        UserChoice::Reject => "Reject",
        UserChoice::Pending => "Choose an option",
    }
}

fn get_trust_explanation(adjustment: i32) -> &'static str {
    match adjustment {
        -10..=-8 => "Significantly reduce AI auto-accept threshold",
        -7..=-5 => "Moderately reduce AI auto-accept threshold",
        -4..=-2 => "Slightly reduce AI auto-accept threshold",
        -1..=1 => "No change to AI auto-accept threshold",
        2..=4 => "Slightly increase AI auto-accept threshold",
        5..=7 => "Moderately increase AI auto-accept threshold",
        8..=10 => "Significantly increase AI auto-accept threshold",
        _ => "Invalid adjustment",
    }
}

fn get_operation_type(operation: &crate::consensus::stages::file_aware_curator::FileOperation) -> &'static str {
    match operation {
        crate::consensus::stages::file_aware_curator::FileOperation::Create { .. } => "Create",
        crate::consensus::stages::file_aware_curator::FileOperation::Update { .. } => "Update",
        crate::consensus::stages::file_aware_curator::FileOperation::Delete { .. } => "Delete",
        crate::consensus::stages::file_aware_curator::FileOperation::Rename { .. } => "Rename",
        crate::consensus::stages::file_aware_curator::FileOperation::Move { .. } => "Move",
    }
}

fn get_operation_file(operation: &crate::consensus::stages::file_aware_curator::FileOperation) -> String {
    match operation {
        crate::consensus::stages::file_aware_curator::FileOperation::Create { path, .. } |
        crate::consensus::stages::file_aware_curator::FileOperation::Update { path, .. } |
        crate::consensus::stages::file_aware_curator::FileOperation::Delete { path } => {
            path.display().to_string()
        }
        crate::consensus::stages::file_aware_curator::FileOperation::Rename { new_path, .. } => {
            new_path.display().to_string()
        }
        crate::consensus::stages::file_aware_curator::FileOperation::Move { destination, .. } => {
            destination.display().to_string()
        }
    }
}

fn get_strategy_name(strategy: &crate::consensus::rollback_planner::RollbackStrategy) -> &'static str {
    match strategy {
        crate::consensus::rollback_planner::RollbackStrategy::GitRevert { .. } => "Git Revert",
        crate::consensus::rollback_planner::RollbackStrategy::BackupRestore { .. } => "Backup Restore",
        crate::consensus::rollback_planner::RollbackStrategy::ManualRollback { .. } => "Manual Rollback",
        crate::consensus::rollback_planner::RollbackStrategy::Hybrid { .. } => "Hybrid Strategy",
    }
}