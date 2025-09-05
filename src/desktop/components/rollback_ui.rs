// Rollback Execution UI Components with Real-time Progress Tracking
use dioxus::prelude::*;
use crate::consensus::rollback_executor::{
    RollbackExecution, RollbackExecutionStatus, RollbackStepResult, RollbackStepStatus,
    RollbackProgress, RollbackSummary, RollbackError, RollbackErrorType
};
use crate::consensus::rollback_planner::{RollbackPlan, RollbackStep, RiskLevel};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Main rollback execution dialog with real-time progress tracking
#[component]
pub fn RollbackExecutionDialog(
    show_dialog: Signal<bool>,
    rollback_plan: Signal<Option<RollbackPlan>>,
    execution_progress: Signal<Option<RollbackProgress>>,
    execution_result: Signal<Option<RollbackExecution>>,
    on_start_rollback: EventHandler<()>,
    on_cancel_rollback: EventHandler<()>,
    on_retry_failed_steps: EventHandler<Vec<u32>>,
) -> Element {
    let execution_started = use_signal(|| false);
    let show_step_details = use_signal(|| false);
    let selected_step = use_signal(|| None::<u32>);
    
    rsx! {
        if show_dialog() {
            div {
                class: "rollback-dialog-overlay",
                onclick: move |e| {
                    if e.target() == e.current_target() && !execution_started() {
                        show_dialog.set(false);
                    }
                },
                
                div {
                    class: "rollback-dialog",
                    
                    // Dialog header
                    div {
                        class: "dialog-header",
                        
                        h2 { 
                            class: "dialog-title",
                            "Rollback Operation"
                        }
                        
                        if let Some(plan) = rollback_plan() {
                            div {
                                class: "rollback-plan-summary",
                                span { class: "plan-id", "Plan: {plan.plan_id}" }
                                span { class: "step-count", "{plan.steps.len()} steps" }
                                span { class: "strategy", "{get_strategy_display(&plan.strategy)}" }
                            }
                        }
                        
                        if !execution_started() {
                            button {
                                class: "close-button",
                                onclick: move |_| show_dialog.set(false),
                                "×"
                            }
                        }
                    }
                    
                    // Dialog content
                    div {
                        class: "dialog-content",
                        
                        // Execution status and progress
                        if let Some(progress) = execution_progress() {
                            RollbackProgressDisplay {
                                progress: progress,
                                show_details: show_step_details,
                                on_step_selected: move |step_num| selected_step.set(Some(step_num))
                            }
                        }
                        
                        // Plan overview when not executing
                        if !execution_started() && rollback_plan().is_some() {
                            RollbackPlanOverview {
                                plan: rollback_plan().unwrap(),
                                on_step_selected: move |step_num| selected_step.set(Some(step_num))
                            }
                        }
                        
                        // Execution results
                        if let Some(execution) = execution_result() {
                            RollbackExecutionResults {
                                execution: execution,
                                on_retry_selected: on_retry_failed_steps
                            }
                        }
                        
                        // Step details panel
                        if let Some(step_num) = selected_step() {
                            if let Some(plan) = rollback_plan() {
                                if let Some(step) = plan.steps.iter().find(|s| s.step_number == step_num) {
                                    RollbackStepDetails {
                                        step: step.clone(),
                                        step_result: execution_result().and_then(|exec| {
                                            exec.steps_completed.iter()
                                                .find(|result| result.step_number == step_num)
                                                .cloned()
                                        }),
                                        on_close: move |_| selected_step.set(None)
                                    }
                                }
                            }
                        }
                    }
                    
                    // Dialog actions
                    div {
                        class: "dialog-actions",
                        
                        if !execution_started() && execution_result().is_none() {
                            // Pre-execution actions
                            button {
                                class: "action-button secondary",
                                onclick: move |_| show_dialog.set(false),
                                "Cancel"
                            }
                            
                            button {
                                class: "action-button danger",
                                onclick: move |_| {
                                    execution_started.set(true);
                                    on_start_rollback.call(());
                                },
                                disabled: rollback_plan().is_none(),
                                "Start Rollback"
                            }
                        } else if execution_started() && execution_result().is_none() {
                            // During execution
                            button {
                                class: "action-button secondary",
                                onclick: move |_| {
                                    on_cancel_rollback.call(());
                                    execution_started.set(false);
                                },
                                "Cancel Rollback"
                            }
                            
                            div {
                                class: "execution-status",
                                if let Some(progress) = execution_progress() {
                                    "Step {progress.current_step}/{progress.total_steps} - {progress.current_operation}"
                                } else {
                                    "Preparing rollback..."
                                }
                            }
                        } else if let Some(execution) = execution_result() {
                            // Post-execution actions
                            button {
                                class: "action-button secondary",
                                onclick: move |_| {
                                    show_dialog.set(false);
                                    execution_started.set(false);
                                },
                                "Close"
                            }
                            
                            if matches!(execution.status, RollbackExecutionStatus::PartiallyCompleted | RollbackExecutionStatus::Failed) {
                                button {
                                    class: "action-button primary",
                                    onclick: move |_| {
                                        let failed_steps: Vec<u32> = execution.steps_completed.iter()
                                            .filter(|step| matches!(step.status, RollbackStepStatus::Failed))
                                            .map(|step| step.step_number)
                                            .collect();
                                        on_retry_failed_steps.call(failed_steps);
                                    },
                                    "Retry Failed Steps"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Real-time progress display for rollback execution
#[component]
pub fn RollbackProgressDisplay(
    progress: RollbackProgress,
    show_details: Signal<bool>,
    on_step_selected: EventHandler<u32>,
) -> Element {
    rsx! {
        div {
            class: "rollback-progress-display",
            
            // Overall progress bar
            div {
                class: "overall-progress",
                
                div {
                    class: "progress-header",
                    h3 { "Rollback Progress" }
                    div {
                        class: "progress-percentage",
                        "{progress.overall_progress:.1}%"
                    }
                }
                
                div {
                    class: "progress-bar-container",
                    
                    div {
                        class: "progress-bar-bg"
                    }
                    
                    div {
                        class: "progress-bar-fill",
                        style: "width: {progress.overall_progress}%"
                    }
                    
                    div {
                        class: "progress-text",
                        "Step {progress.current_step} of {progress.total_steps}"
                    }
                }
            }
            
            // Current operation status
            div {
                class: "current-operation",
                
                div {
                    class: "operation-header",
                    h4 { "Current Operation" }
                    div {
                        class: "status-badge {get_status_class(&progress.status)}",
                        "{progress.status:?}"
                    }
                }
                
                div {
                    class: "operation-description",
                    "{progress.current_operation}"
                }
                
                if let Some(estimated_time) = progress.estimated_remaining_time {
                    div {
                        class: "estimated-time",
                        "Estimated remaining: {format_duration(estimated_time)}"
                    }
                }
            }
            
            // Step-by-step progress (collapsible)
            div {
                class: "step-progress-section",
                
                div {
                    class: "section-header",
                    onclick: move |_| show_details.set(!show_details()),
                    
                    h4 { "Step Details" }
                    div {
                        class: "expand-icon {if show_details() { \\\"expanded\\\" } else { \\\"\\\" }}",
                        "▼"
                    }
                }
                
                if show_details() {
                    StepProgressList {
                        current_step: progress.current_step,
                        total_steps: progress.total_steps,
                        on_step_selected: on_step_selected
                    }
                }
            }
        }
    }
}

/// Plan overview before execution starts
#[component]
pub fn RollbackPlanOverview(
    plan: RollbackPlan,
    on_step_selected: EventHandler<u32>,
) -> Element {
    let show_risk_details = use_signal(|| false);
    
    rsx! {
        div {
            class: "rollback-plan-overview",
            
            // Plan summary
            div {
                class: "plan-summary",
                
                h3 { "Rollback Plan Overview" }
                
                div {
                    class: "summary-stats",
                    
                    div {
                        class: "stat-item",
                        span { class: "stat-label", "Total Steps:" }
                        span { class: "stat-value", "{plan.steps.len()}" }
                    }
                    
                    div {
                        class: "stat-item",
                        span { class: "stat-label", "Estimated Duration:" }
                        span { class: "stat-value", "{plan.estimated_duration_ms}ms" }
                    }
                    
                    div {
                        class: "stat-item",
                        span { class: "stat-label", "Strategy:" }
                        span { class: "stat-value", "{get_strategy_display(&plan.strategy)}" }
                    }
                }
            }
            
            // Risk assessment
            div {
                class: "risk-assessment-section",
                
                div {
                    class: "risk-header",
                    onclick: move |_| show_risk_details.set(!show_risk_details()),
                    
                    h4 { "Risk Assessment" }
                    
                    div {
                        class: "risk-level {plan.risk_assessment.risk_level:?}",
                        "{plan.risk_assessment.risk_level:?} Risk"
                    }
                    
                    div {
                        class: "success-probability",
                        "{plan.risk_assessment.success_probability:.0}% success probability"
                    }
                    
                    div {
                        class: "expand-icon {if show_risk_details() { \\\"expanded\\\" } else { \\\"\\\" }}",
                        "▼"
                    }
                }
                
                if show_risk_details() {
                    div {
                        class: "risk-details",
                        
                        if !plan.risk_assessment.risks.is_empty() {
                            div {
                                class: "risks-list",
                                h5 { "Identified Risks" }
                                for risk in &plan.risk_assessment.risks {
                                    div {
                                        class: "risk-item",
                                        div { class: "risk-description", "{risk.description}" }
                                        div { class: "risk-mitigation", "Mitigation: {risk.mitigation_strategy.as_ref().unwrap_or(&\"None specified\".to_string())}" }
                                    }
                                }
                            }
                        }
                        
                        if !plan.risk_assessment.mitigations.is_empty() {
                            div {
                                class: "mitigations-list",
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
                }
            }
            
            // Steps overview
            div {
                class: "steps-overview",
                
                h4 { "Rollback Steps" }
                
                div {
                    class: "steps-list",
                    for step in plan.steps.iter().take(10) {
                        div {
                            class: "step-overview-item",
                            onclick: move |_| on_step_selected.call(step.step_number),
                            
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
                                    span { class: "step-risk {step.risk_level:?}", "{step.risk_level:?}" }
                                    if step.automatable {
                                        span { class: "step-auto", "Auto" }
                                    } else {
                                        span { class: "step-manual", "Manual" }
                                    }
                                }
                            }
                        }
                    }
                    
                    if plan.steps.len() > 10 {
                        div {
                            class: "steps-truncated",
                            "... and {plan.steps.len() - 10} more steps"
                        }
                    }
                }
            }
        }
    }
}

/// Step-by-step progress list
#[component]
pub fn StepProgressList(
    current_step: u32,
    total_steps: u32,
    on_step_selected: EventHandler<u32>,
) -> Element {
    rsx! {
        div {
            class: "step-progress-list",
            
            for step_num in 1..=total_steps {
                div {
                    class: "step-progress-item {get_step_progress_class(step_num, current_step)}",
                    onclick: move |_| on_step_selected.call(step_num),
                    
                    div {
                        class: "step-number",
                        "{step_num}"
                    }
                    
                    div {
                        class: "step-status-indicator",
                        {get_step_status_icon(step_num, current_step)}
                    }
                    
                    div {
                        class: "step-description",
                        "Step {step_num}"
                        // Note: Would need step details to show actual description
                    }
                }
            }
        }
    }
}

/// Execution results summary
#[component]
pub fn RollbackExecutionResults(
    execution: RollbackExecution,
    on_retry_selected: EventHandler<Vec<u32>>,
) -> Element {
    let show_error_details = use_signal(|| false);
    let show_summary_details = use_signal(|| false);
    
    rsx! {
        div {
            class: "rollback-execution-results",
            
            // Execution summary
            div {
                class: "execution-summary",
                
                div {
                    class: "summary-header",
                    h3 { "Execution Results" }
                    div {
                        class: "execution-status {get_execution_status_class(&execution.status)}",
                        "{execution.status:?}"
                    }
                }
                
                div {
                    class: "summary-stats",
                    
                    div {
                        class: "stat-group",
                        
                        div {
                            class: "stat-item",
                            span { class: "stat-label", "Steps Completed:" }
                            span { class: "stat-value", "{execution.steps_completed.len()}/{execution.total_steps}" }
                        }
                        
                        div {
                            class: "stat-item",
                            span { class: "stat-label", "Duration:" }
                            span { class: "stat-value", {
                                if let (Some(start), Some(end)) = (execution.started_at, execution.completed_at) {
                                    format!("{:.1}s", (end - start).num_milliseconds() as f64 / 1000.0)
                                } else {
                                    "In progress".to_string()
                                }
                            }}
                        }
                        
                        if let Some(summary) = &execution.rollback_summary {
                            div {
                                class: "stat-item",
                                span { class: "stat-label", "Success Rate:" }
                                span { class: "stat-value", "{summary.success_rate:.1}%" }
                            }
                        }
                    }
                }
            }
            
            // Error summary
            if !execution.errors.is_empty() {
                div {
                    class: "error-summary",
                    
                    div {
                        class: "error-header",
                        onclick: move |_| show_error_details.set(!show_error_details()),
                        
                        h4 { "Errors ({execution.errors.len()})" }
                        div {
                            class: "expand-icon {if show_error_details() { \\\"expanded\\\" } else { \\\"\\\" }}",
                            "▼"
                        }
                    }
                    
                    if show_error_details() {
                        div {
                            class: "error-list",
                            for error in &execution.errors {
                                RollbackErrorDisplay {
                                    error: error.clone()
                                }
                            }
                        }
                    }
                }
            }
            
            // Detailed summary
            if let Some(summary) = &execution.rollback_summary {
                div {
                    class: "detailed-summary",
                    
                    div {
                        class: "summary-section-header",
                        onclick: move |_| show_summary_details.set(!show_summary_details()),
                        
                        h4 { "Detailed Summary" }
                        div {
                            class: "expand-icon {if show_summary_details() { \\\"expanded\\\" } else { \\\"\\\" }}",
                            "▼"
                        }
                    }
                    
                    if show_summary_details() {
                        RollbackSummaryDisplay {
                            summary: summary.clone()
                        }
                    }
                }
            }
            
            // Failed steps for retry
            if matches!(execution.status, RollbackExecutionStatus::PartiallyCompleted | RollbackExecutionStatus::Failed) {
                FailedStepsRetrySection {
                    execution: execution.clone(),
                    on_retry_selected: on_retry_selected
                }
            }
        }
    }
}

/// Individual step details panel
#[component]
pub fn RollbackStepDetails(
    step: RollbackStep,
    step_result: Option<RollbackStepResult>,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "step-details-panel",
            
            div {
                class: "panel-header",
                
                h3 { "Step {step.step_number} Details" }
                
                button {
                    class: "close-button",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }
            
            div {
                class: "panel-content",
                
                // Step information
                div {
                    class: "step-info-section",
                    
                    div {
                        class: "step-description",
                        h4 { "Description" }
                        p { "{step.description}" }
                    }
                    
                    div {
                        class: "step-metadata",
                        
                        div {
                            class: "meta-item",
                            span { class: "meta-label", "Risk Level:" }
                            span { class: "meta-value risk-{step.risk_level:?}", "{step.risk_level:?}" }
                        }
                        
                        div {
                            class: "meta-item",
                            span { class: "meta-label", "Estimated Duration:" }
                            span { class: "meta-value", "{step.estimated_duration_ms}ms" }
                        }
                        
                        div {
                            class: "meta-item",
                            span { class: "meta-label", "Automatable:" }
                            span { class: "meta-value", "{if step.automatable { \"Yes\" } else { \"No\" }}" }
                        }
                    }
                    
                    div {
                        class: "step-type-details",
                        h4 { "Operation Type" }
                        StepTypeDisplay {
                            step_type: step.step_type.clone()
                        }
                    }
                }
                
                // Execution results (if available)
                if let Some(result) = step_result {
                    div {
                        class: "execution-results-section",
                        
                        h4 { "Execution Results" }
                        
                        div {
                            class: "result-summary",
                            
                            div {
                                class: "result-status {get_step_result_class(&result.status)}",
                                "{result.status:?}"
                            }
                            
                            if let Some(duration) = result.duration_ms {
                                div {
                                    class: "result-duration",
                                    "Duration: {duration}ms"
                                }
                            }
                            
                            if result.retry_count > 0 {
                                div {
                                    class: "result-retries",
                                    "Retries: {result.retry_count}"
                                }
                            }
                        }
                        
                        if let Some(error) = &result.error_message {
                            div {
                                class: "result-error",
                                h5 { "Error Message" }
                                pre { "{error}" }
                            }
                        }
                        
                        if !result.files_affected.is_empty() {
                            div {
                                class: "files-affected",
                                h5 { "Files Affected" }
                                ul {
                                    for file in &result.files_affected {
                                        li { "{file.display()}" }
                                    }
                                }
                            }
                        }
                        
                        if let Some(verification) = &result.verification_result {
                            div {
                                class: "verification-result",
                                h5 { "Verification" }
                                div {
                                    class: "verification-status {if verification.is_successful { \\\"success\\\" } else { \\\"failure\\\" }}",
                                    "{if verification.is_successful { \"✓ Verified\" } else { \"✗ Verification Failed\" }}"
                                }
                                if let Some(error) = &verification.error_message {
                                    div {
                                        class: "verification-error",
                                        "{error}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Display for step operation type
#[component]
pub fn StepTypeDisplay(step_type: crate::consensus::rollback_planner::RollbackStepType) -> Element {
    use crate::consensus::rollback_planner::RollbackStepType;
    
    rsx! {
        div {
            class: "step-type-display",
            
            match step_type {
                RollbackStepType::RestoreFromBackup { backup_path, target_path } => rsx! {
                    div {
                        class: "step-type-restore",
                        div { class: "operation-type", "Restore from Backup" }
                        div { class: "backup-path", "From: {backup_path.display()}" }
                        div { class: "target-path", "To: {target_path.display()}" }
                    }
                },
                RollbackStepType::GitRevert { commit_hash, file_paths } => rsx! {
                    div {
                        class: "step-type-git",
                        div { class: "operation-type", "Git Revert" }
                        div { class: "commit-hash", "Commit: {commit_hash}" }
                        if let Some(paths) = file_paths {
                            div { class: "file-paths", "Files: {paths.len()} files" }
                        }
                    }
                },
                RollbackStepType::DeleteFile { file_path } => rsx! {
                    div {
                        class: "step-type-delete",
                        div { class: "operation-type", "Delete File" }
                        div { class: "file-path", "Path: {file_path.display()}" }
                    }
                },
                RollbackStepType::RecreateFile { file_path, content } => rsx! {
                    div {
                        class: "step-type-recreate",
                        div { class: "operation-type", "Recreate File" }
                        div { class: "file-path", "Path: {file_path.display()}" }
                        div { class: "content-size", "Content: {content.len()} bytes" }
                    }
                },
                RollbackStepType::RestoreFileContent { file_path, original_content } => rsx! {
                    div {
                        class: "step-type-restore-content",
                        div { class: "operation-type", "Restore File Content" }
                        div { class: "file-path", "Path: {file_path.display()}" }
                        div { class: "content-size", "Content: {original_content.len()} bytes" }
                    }
                },
                RollbackStepType::RunScript { script_path, args } => rsx! {
                    div {
                        class: "step-type-script",
                        div { class: "operation-type", "Run Script" }
                        div { class: "script-path", "Script: {script_path.display()}" }
                        div { class: "script-args", "Args: {args.join(\" \")}" }
                    }
                },
                RollbackStepType::UserAction { description, instructions } => rsx! {
                    div {
                        class: "step-type-user",
                        div { class: "operation-type", "User Action Required" }
                        div { class: "action-description", "{description}" }
                        div { class: "action-instructions", "{instructions}" }
                    }
                },
            }
        }
    }
}

/// Error display component
#[component]
pub fn RollbackErrorDisplay(error: RollbackError) -> Element {
    rsx! {
        div {
            class: "rollback-error-display",
            
            div {
                class: "error-header",
                
                div {
                    class: "error-type {get_error_type_class(&error.error_type)}",
                    "{error.error_type:?}"
                }
                
                div {
                    class: "error-timestamp",
                    "{error.timestamp.format(\"%H:%M:%S\")}"
                }
                
                if let Some(step_num) = error.step_number {
                    div {
                        class: "error-step",
                        "Step {step_num}"
                    }
                }
            }
            
            div {
                class: "error-message",
                "{error.message}"
            }
            
            if let Some(action) = &error.suggested_action {
                div {
                    class: "suggested-action",
                    strong { "Suggested Action: " }
                    "{action}"
                }
            }
            
            div {
                class: "error-meta",
                span {
                    class: "recoverable {if error.is_recoverable { \\\"yes\\\" } else { \\\"no\\\" }}",
                    "{if error.is_recoverable { \"Recoverable\" } else { \"Not Recoverable\" }}"
                }
            }
        }
    }
}

/// Summary display component
#[component]
pub fn RollbackSummaryDisplay(summary: RollbackSummary) -> Element {
    rsx! {
        div {
            class: "rollback-summary-display",
            
            div {
                class: "summary-metrics",
                
                div {
                    class: "metric-item",
                    span { class: "metric-label", "Files Restored:" }
                    span { class: "metric-value", "{summary.files_restored.len()}" }
                }
                
                div {
                    class: "metric-item",
                    span { class: "metric-label", "Git Commits Reverted:" }
                    span { class: "metric-value", "{summary.git_commits_reverted.len()}" }
                }
                
                div {
                    class: "metric-item",
                    span { class: "metric-label", "Backups Used:" }
                    span { class: "metric-value", "{summary.backups_used.len()}" }
                }
                
                div {
                    class: "metric-item",
                    span { class: "metric-label", "Total Duration:" }
                    span { class: "metric-value", "{summary.total_duration_ms}ms" }
                }
            }
            
            if !summary.files_restored.is_empty() {
                div {
                    class: "files-restored-list",
                    h5 { "Files Restored" }
                    ul {
                        for file in summary.files_restored.iter().take(10) {
                            li { "{file.display()}" }
                        }
                        if summary.files_restored.len() > 10 {
                            li { "... and {summary.files_restored.len() - 10} more files" }
                        }
                    }
                }
            }
            
            if !summary.recommendations.is_empty() {
                div {
                    class: "recommendations-list",
                    h5 { "Recommendations" }
                    ul {
                        for recommendation in &summary.recommendations {
                            li { "{recommendation}" }
                        }
                    }
                }
            }
        }
    }
}

/// Failed steps retry section
#[component]
pub fn FailedStepsRetrySection(
    execution: RollbackExecution,
    on_retry_selected: EventHandler<Vec<u32>>,
) -> Element {
    let selected_steps = use_signal(|| std::collections::HashSet::<u32>::new());
    
    let failed_steps: Vec<&RollbackStepResult> = execution.steps_completed.iter()
        .filter(|step| matches!(step.status, RollbackStepStatus::Failed))
        .collect();
    
    rsx! {
        div {
            class: "failed-steps-retry-section",
            
            h4 { "Failed Steps ({failed_steps.len()})" }
            
            div {
                class: "failed-steps-list",
                for step in &failed_steps {
                    div {
                        class: "failed-step-item",
                        
                        input {
                            r#type: "checkbox",
                            checked: selected_steps().contains(&step.step_number),
                            onchange: move |e| {
                                let mut current = selected_steps();
                                if e.checked() {
                                    current.insert(step.step_number);
                                } else {
                                    current.remove(&step.step_number);
                                }
                                selected_steps.set(current);
                            }
                        }
                        
                        div {
                            class: "step-info",
                            
                            div {
                                class: "step-number",
                                "Step {step.step_number}"
                            }
                            
                            if let Some(error) = &step.error_message {
                                div {
                                    class: "step-error",
                                    "{error}"
                                }
                            }
                            
                            div {
                                class: "step-retries",
                                "Attempts: {step.retry_count + 1}"
                            }
                        }
                    }
                }
            }
            
            div {
                class: "retry-actions",
                
                button {
                    class: "select-all-button",
                    onclick: move |_| {
                        let all_steps: std::collections::HashSet<u32> = failed_steps.iter()
                            .map(|step| step.step_number)
                            .collect();
                        selected_steps.set(all_steps);
                    },
                    "Select All"
                }
                
                button {
                    class: "retry-selected-button",
                    onclick: move |_| {
                        let steps_to_retry: Vec<u32> = selected_steps().into_iter().collect();
                        on_retry_selected.call(steps_to_retry);
                    },
                    disabled: selected_steps().is_empty(),
                    "Retry Selected ({selected_steps().len()})"
                }
            }
        }
    }
}

// Helper functions

fn get_strategy_display(strategy: &crate::consensus::rollback_planner::RollbackStrategy) -> &'static str {
    use crate::consensus::rollback_planner::RollbackStrategy;
    match strategy {
        RollbackStrategy::GitRevert { .. } => "Git Revert",
        RollbackStrategy::BackupRestore { .. } => "Backup Restore",
        RollbackStrategy::ManualRollback { .. } => "Manual Rollback",
        RollbackStrategy::Hybrid { .. } => "Hybrid Strategy",
    }
}

fn get_status_class(status: &RollbackExecutionStatus) -> &'static str {
    match status {
        RollbackExecutionStatus::NotStarted => "not-started",
        RollbackExecutionStatus::InProgress => "in-progress",
        RollbackExecutionStatus::Completed => "completed",
        RollbackExecutionStatus::Failed => "failed",
        RollbackExecutionStatus::PartiallyCompleted => "partially-completed",
        RollbackExecutionStatus::Aborted => "aborted",
        RollbackExecutionStatus::WaitingForUserConfirmation => "waiting-confirmation",
    }
}

fn get_execution_status_class(status: &RollbackExecutionStatus) -> &'static str {
    match status {
        RollbackExecutionStatus::Completed => "status-success",
        RollbackExecutionStatus::Failed => "status-error",
        RollbackExecutionStatus::PartiallyCompleted => "status-warning",
        RollbackExecutionStatus::Aborted => "status-warning",
        _ => "status-info",
    }
}

fn get_step_progress_class(step_num: u32, current_step: u32) -> &'static str {
    if step_num < current_step {
        "completed"
    } else if step_num == current_step {
        "current"
    } else {
        "pending"
    }
}

fn get_step_status_icon(step_num: u32, current_step: u32) -> &'static str {
    if step_num < current_step {
        "✓"
    } else if step_num == current_step {
        "⟳"
    } else {
        "○"
    }
}

fn get_step_result_class(status: &RollbackStepStatus) -> &'static str {
    match status {
        RollbackStepStatus::Completed => "result-success",
        RollbackStepStatus::Failed => "result-error",
        RollbackStepStatus::InProgress => "result-progress",
        RollbackStepStatus::Skipped => "result-skipped",
        RollbackStepStatus::WaitingForConfirmation => "result-waiting",
        RollbackStepStatus::Pending => "result-pending",
    }
}

fn get_error_type_class(error_type: &RollbackErrorType) -> &'static str {
    match error_type {
        RollbackErrorType::FileSystemError => "error-filesystem",
        RollbackErrorType::GitError => "error-git",
        RollbackErrorType::BackupError => "error-backup",
        RollbackErrorType::VerificationError => "error-verification",
        RollbackErrorType::TimeoutError => "error-timeout",
        RollbackErrorType::PermissionError => "error-permission",
        RollbackErrorType::UserAbortError => "error-user-abort",
        RollbackErrorType::InternalError => "error-internal",
    }
}

fn format_duration(duration: tokio::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    if total_seconds < 60 {
        format!("{}s", total_seconds)
    } else if total_seconds < 3600 {
        format!("{}m {}s", total_seconds / 60, total_seconds % 60)
    } else {
        format!("{}h {}m", total_seconds / 3600, (total_seconds % 3600) / 60)
    }
}