// Progress Indicators and Status Display Components
// Shows real-time progress of file operations and AI processing

use dioxus::prelude::*;
use crate::consensus::{
    stages::file_aware_curator::FileOperation,
    ai_operation_parser::FileOperationWithMetadata,
    file_executor::{ExecutionResult, ExecutionSummary},
    ConsensusStage,
};
use crate::desktop::styles::theme::ThemeColors;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Progress state for an operation
#[derive(Debug, Clone, PartialEq)]
pub struct OperationProgress {
    pub operation_id: usize,
    pub status: OperationStatus,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub progress_percentage: f32,
    pub current_step: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Pending,
    Processing,
    Executing,
    Completed,
    Failed,
    Skipped,
}

/// Progress indicators component props
#[derive(Props, Clone, PartialEq)]
pub struct ProgressIndicatorsProps {
    /// Current consensus stage
    pub consensus_stage: Option<ConsensusStage>,
    
    /// Operations being processed
    pub operations: Vec<FileOperationWithMetadata>,
    
    /// Progress for each operation
    pub operation_progress: HashMap<usize, OperationProgress>,
    
    /// Overall execution summary
    pub execution_summary: Option<ExecutionSummary>,
    
    /// Theme colors
    pub theme: ThemeColors,
    
    /// Whether to show detailed progress
    pub show_details: bool,
    
    /// Callback when user clicks on an operation
    pub on_operation_click: EventHandler<usize>,
}

/// Progress indicators component
#[component]
pub fn ProgressIndicators(props: ProgressIndicatorsProps) -> Element {
    let mut expanded_operations = use_signal(|| Vec::<usize>::new());
    let mut show_timeline = use_signal(|| true);
    
    // Calculate overall progress
    let (overall_progress, stats) = calculate_overall_progress(&props.operation_progress);
    
    rsx! {
        div {
            class: "progress-indicators",
            style: "padding: 20px;",
            
            // Overall progress card
            OverallProgressCard {
                progress: overall_progress,
                stats: stats,
                consensus_stage: props.consensus_stage.clone(),
                execution_summary: props.execution_summary.clone(),
                theme: props.theme.clone(),
            }
            
            // Stage indicator
            if let Some(stage) = &props.consensus_stage {
                StageIndicator {
                    stage: stage.clone(),
                    theme: props.theme.clone(),
                }
            }
            
            // Operation progress list
            div {
                style: "margin-top: 24px;",
                
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        margin-bottom: 16px;
                    ",
                    
                    h3 {
                        style: "margin: 0; color: {props.theme.text}; flex: 1;",
                        "Operation Progress"
                    }
                    
                    {
                        let show_timeline_value = show_timeline();
                        let bg_color = if show_timeline_value { props.theme.primary.clone() } else { "transparent".to_string() };
                        let text_color = if show_timeline_value { props.theme.background.clone() } else { props.theme.text.clone() };
                        
                        rsx! {
                            button {
                                style: "
                                    background: {bg_color};
                                    color: {text_color};
                                    border: 1px solid {props.theme.border};
                                    padding: 6px 12px;
                                    border-radius: 4px;
                                    cursor: pointer;
                                    font-size: 13px;
                                ",
                                onclick: move |_| show_timeline.set(!show_timeline()),
                                "📊 Timeline View"
                            }
                        }
                    }
                }
                
                if show_timeline() {
                    TimelineView {
                        operations: props.operations.clone(),
                        progress: props.operation_progress.clone(),
                        theme: props.theme.clone(),
                        on_operation_click: props.on_operation_click.clone(),
                    }
                } else {
                    div {
                        style: "display: flex; flex-direction: column; gap: 12px;",
                        
                        for (idx, operation) in props.operations.iter().enumerate() {
                            OperationProgressItem {
                                operation: operation.clone(),
                                progress: props.operation_progress.get(&idx).cloned(),
                                is_expanded: expanded_operations().contains(&idx),
                                theme: props.theme.clone(),
                                on_click: move |_| {
                                    props.on_operation_click.call(idx);
                                    let mut expanded = expanded_operations();
                                    if expanded.contains(&idx) {
                                        expanded.retain(|&x| x != idx);
                                    } else {
                                        expanded.push(idx);
                                    }
                                    expanded_operations.set(expanded);
                                }
                            }
                        }
                    }
                }
            }
            
            // Real-time status feed
            if props.show_details {
                StatusFeed {
                    operation_progress: props.operation_progress.clone(),
                    theme: props.theme.clone(),
                }
            }
        }
    }
}

/// Overall progress card component
#[derive(Props, Clone, PartialEq)]
struct OverallProgressCardProps {
    progress: f32,
    stats: ProgressStats,
    consensus_stage: Option<ConsensusStage>,
    execution_summary: Option<ExecutionSummary>,
    theme: ThemeColors,
}

#[derive(Debug, Clone, PartialEq)]
struct ProgressStats {
    total: usize,
    completed: usize,
    failed: usize,
    skipped: usize,
    in_progress: usize,
}

#[component]
fn OverallProgressCard(props: OverallProgressCardProps) -> Element {
    rsx! {
        div {
            style: "
                background: {props.theme.background_secondary};
                border: 1px solid {props.theme.border};
                border-radius: 8px;
                padding: 20px;
            ",
            
            div {
                style: "display: flex; align-items: center; margin-bottom: 16px;",
                
                div {
                    style: "flex: 1;",
                    
                    h2 {
                        style: "margin: 0 0 4px; color: {props.theme.text};",
                        "Overall Progress"
                    }
                    
                    if let Some(stage) = &props.consensus_stage {
                        div {
                            style: "font-size: 14px; color: {props.theme.text_secondary};",
                            {format!("Current Stage: {:?}", stage)}
                        }
                    }
                }
                
                // Circular progress indicator
                CircularProgress {
                    value: props.progress,
                    size: 80,
                    theme: props.theme.clone(),
                }
            }
            
            // Progress bar
            div {
                style: "
                    height: 8px;
                    background: {props.theme.background};
                    border-radius: 4px;
                    overflow: hidden;
                    margin-bottom: 16px;
                ",
                
                div {
                    style: "
                        height: 100%;
                        background: linear-gradient(90deg, {props.theme.success.clone()}, {props.theme.primary.clone()});
                        width: {props.progress}%;
                        transition: width 0.3s;
                    ",
                }
            }
            
            // Statistics
            div {
                style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px;",
                
                StatItem {
                    label: "Total",
                    value: props.stats.total.to_string(),
                    color: props.theme.text.clone(),
                    theme: props.theme.clone(),
                }
                
                StatItem {
                    label: "Completed",
                    value: props.stats.completed.to_string(),
                    color: props.theme.success.clone(),
                    theme: props.theme.clone(),
                }
                
                StatItem {
                    label: "Failed",
                    value: props.stats.failed.to_string(),
                    color: props.theme.error.clone(),
                    theme: props.theme.clone(),
                }
                
                StatItem {
                    label: "In Progress",
                    value: props.stats.in_progress.to_string(),
                    color: props.theme.warning.clone(),
                    theme: props.theme.clone(),
                }
            }
            
            // Execution summary
            if let Some(summary) = &props.execution_summary {
                div {
                    style: "
                        margin-top: 16px;
                        padding-top: 16px;
                        border-top: 1px solid {props.theme.border};
                    ",
                    
                    div {
                        style: "
                            display: flex;
                            justify-content: space-between;
                            font-size: 14px;
                        ",
                        
                        div {
                            style: "color: {props.theme.text_secondary};",
                            "Execution Time:"
                        }
                        div {
                            style: "color: {props.theme.text}; font-weight: bold;",
                            {format_duration(summary.total_execution_time)}
                        }
                    }
                    
                    if summary.successful_operations > 0 {
                        div {
                            style: "
                                display: flex;
                                justify-content: space-between;
                                font-size: 14px;
                                margin-top: 4px;
                            ",
                            
                            div {
                                style: "color: {props.theme.text_secondary};",
                                "Successful Operations:"
                            }
                            div {
                                style: "color: {props.theme.success};",
                                {format!("{}", summary.successful_operations)}
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Stage indicator component
#[derive(Props, Clone, PartialEq)]
struct StageIndicatorProps {
    stage: ConsensusStage,
    theme: ThemeColors,
}

#[component]
fn StageIndicator(props: StageIndicatorProps) -> Element {
    let stages = vec![
        (ConsensusStage::Generator, "Generator", "🧠"),
        (ConsensusStage::Refiner, "Refiner", "✨"),
        (ConsensusStage::Validator, "Validator", "✓"),
        (ConsensusStage::Curator, "Curator", "📋"),
    ];
    
    rsx! {
        div {
            style: "
                background: {props.theme.background_secondary};
                border: 1px solid {props.theme.border};
                border-radius: 8px;
                padding: 16px;
                margin-top: 16px;
            ",
            
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",
                
                for (stage, name, icon) in stages {
                    StageItem {
                        name: name,
                        icon: icon,
                        is_active: stage == props.stage,
                        is_completed: stage < props.stage,
                        theme: props.theme.clone(),
                    }
                }
            }
        }
    }
}

/// Stage item component
#[derive(Props, Clone, PartialEq)]
struct StageItemProps {
    name: &'static str,
    icon: &'static str,
    is_active: bool,
    is_completed: bool,
    theme: ThemeColors,
}

#[component]
fn StageItem(props: StageItemProps) -> Element {
    let (bg_color, icon_color, border_color, text_color) = if props.is_active {
        (props.theme.primary.clone(), props.theme.background.clone(), props.theme.primary.clone(), props.theme.text.clone())
    } else if props.is_completed {
        (props.theme.success.clone(), props.theme.background.clone(), props.theme.success.clone(), props.theme.text.clone())
    } else {
        ("transparent".to_string(), props.theme.text_secondary.clone(), props.theme.border.clone(), props.theme.text_secondary.clone())
    };
    
    let font_weight = if props.is_active { "bold" } else { "normal" };
    
    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 8px;
            ",
            
            div {
                style: "
                    width: 50px;
                    height: 50px;
                    background: {bg_color};
                    border: 2px solid {border_color};
                    border-radius: 50%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 20px;
                    color: {icon_color};
                    transition: all 0.3s;
                ",
                "{props.icon}"
            }
            
            div {
                style: "
                    font-size: 12px;
                    color: {text_color};
                    font-weight: {font_weight};
                ",
                "{props.name}"
            }
        }
    }
}

/// Timeline view component
#[derive(Props, Clone, PartialEq)]
struct TimelineViewProps {
    operations: Vec<FileOperationWithMetadata>,
    progress: HashMap<usize, OperationProgress>,
    theme: ThemeColors,
    on_operation_click: EventHandler<usize>,
}

#[component]
fn TimelineView(props: TimelineViewProps) -> Element {
    rsx! {
        div {
            style: "
                background: {props.theme.background_secondary};
                border: 1px solid {props.theme.border};
                border-radius: 8px;
                padding: 20px;
                overflow-x: auto;
            ",
            
            // Timeline header
            div {
                style: "
                    display: grid;
                    grid-template-columns: 200px 1fr;
                    gap: 16px;
                    margin-bottom: 16px;
                    font-size: 12px;
                    color: {props.theme.text_secondary};
                ",
                
                div { "Operation" }
                div { "Timeline" }
            }
            
            // Timeline items
            for (idx, operation) in props.operations.iter().enumerate() {
                TimelineItem {
                    operation: operation.clone(),
                    progress: props.progress.get(&idx).cloned(),
                    theme: props.theme.clone(),
                    on_click: move |_| props.on_operation_click.call(idx),
                }
            }
        }
    }
}

/// Timeline item component
#[derive(Props, Clone, PartialEq)]
struct TimelineItemProps {
    operation: FileOperationWithMetadata,
    progress: Option<OperationProgress>,
    theme: ThemeColors,
    on_click: EventHandler<()>,
}

#[component]
fn TimelineItem(props: TimelineItemProps) -> Element {
    let progress = props.progress.as_ref();
    let status_color = match progress.map(|p| &p.status) {
        Some(OperationStatus::Completed) => props.theme.success.clone(),
        Some(OperationStatus::Failed) => props.theme.error.clone(),
        Some(OperationStatus::Processing) | Some(OperationStatus::Executing) => props.theme.warning.clone(),
        Some(OperationStatus::Skipped) => props.theme.text_secondary.clone(),
        _ => props.theme.border.clone(),
    };
    
    rsx! {
        div {
            style: "
                display: grid;
                grid-template-columns: 200px 1fr;
                gap: 16px;
                padding: 12px 0;
                border-bottom: 1px solid {props.theme.border};
                cursor: pointer;
            ",
            onclick: move |_| props.on_click.call(()),
            
            // Operation info
            div {
                style: "
                    display: flex;
                    align-items: center;
                    gap: 8px;
                ",
                
                div {
                    style: "
                        width: 8px;
                        height: 8px;
                        background: {status_color};
                        border-radius: 50%;
                    ",
                }
                
                div {
                    style: "
                        font-size: 13px;
                        color: {props.theme.text};
                        white-space: nowrap;
                        overflow: hidden;
                        text-overflow: ellipsis;
                    ",
                    "{get_operation_path(&props.operation.operation)}"
                }
            }
            
            // Timeline bar
            div {
                style: "
                    height: 24px;
                    background: {props.theme.background};
                    border-radius: 4px;
                    overflow: hidden;
                    position: relative;
                ",
                
                if let Some(progress) = progress {
                    div {
                        style: "
                            position: absolute;
                            left: 0;
                            top: 0;
                            height: 100%;
                            background: {status_color};
                            width: {progress.progress_percentage}%;
                            transition: width 0.3s;
                            display: flex;
                            align-items: center;
                            padding: 0 8px;
                        ",
                        
                        if progress.progress_percentage > 20.0 {
                            div {
                                style: "
                                    font-size: 11px;
                                    color: {props.theme.background};
                                    white-space: nowrap;
                                ",
                                "{progress.current_step}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Operation progress item component
#[derive(Props, Clone, PartialEq)]
struct OperationProgressItemProps {
    operation: FileOperationWithMetadata,
    progress: Option<OperationProgress>,
    is_expanded: bool,
    theme: ThemeColors,
    on_click: EventHandler<()>,
}

#[component]
fn OperationProgressItem(props: OperationProgressItemProps) -> Element {
    let progress = props.progress.as_ref();
    
    let (icon, color) = match &props.operation.operation {
        FileOperation::Create { .. } => ("🆕", props.theme.success.clone()),
        FileOperation::Update { .. } => ("✏️", props.theme.warning.clone()),
        FileOperation::Delete { .. } => ("🗑️", props.theme.error.clone()),
        FileOperation::Rename { .. } => ("🔄", props.theme.primary.clone()),
        FileOperation::Append { .. } => ("📝", props.theme.info.clone()),
    };
    
    let status_icon = match progress.map(|p| &p.status) {
        Some(OperationStatus::Completed) => Some(("✓", props.theme.success.clone())),
        Some(OperationStatus::Failed) => Some(("✗", props.theme.error.clone())),
        Some(OperationStatus::Processing) => Some(("⚡", props.theme.warning.clone())),
        Some(OperationStatus::Executing) => Some(("🔄", props.theme.primary.clone())),
        Some(OperationStatus::Skipped) => Some(("⏭️", props.theme.text_secondary.clone())),
        _ => None,
    };
    
    rsx! {
        div {
            style: "
                background: {props.theme.background_secondary};
                border: 1px solid {props.theme.border};
                border-radius: 6px;
                overflow: hidden;
                transition: all 0.2s;
            ",
            
            // Header
            div {
                style: "
                    display: flex;
                    align-items: center;
                    padding: 12px;
                    cursor: pointer;
                ",
                onclick: move |_| props.on_click.call(()),
                
                span {
                    style: "font-size: 18px; margin-right: 12px;",
                    "{icon}"
                }
                
                div {
                    style: "flex: 1;",
                    
                    div {
                        style: "
                            font-size: 14px;
                            color: {props.theme.text};
                            margin-bottom: 2px;
                        ",
                        "{get_operation_path(&props.operation.operation)}"
                    }
                    
                    if let Some(progress) = progress {
                        div {
                            style: "
                                font-size: 12px;
                                color: {props.theme.text_secondary};
                            ",
                            "{progress.current_step}"
                        }
                    }
                }
                
                if let Some((icon, color)) = status_icon {
                    div {
                        style: "
                            font-size: 20px;
                            color: {color};
                            margin-left: 12px;
                        ",
                        "{icon}"
                    }
                }
                
                // Progress percentage
                if let Some(progress) = progress {
                    if progress.status == OperationStatus::Processing || progress.status == OperationStatus::Executing {
                        div {
                            style: "
                                margin-left: 16px;
                                font-size: 14px;
                                font-weight: bold;
                                color: {props.theme.primary};
                            ",
                            {format!("{:.0}%", progress.progress_percentage)}
                        }
                    }
                }
            }
            
            // Progress bar
            if let Some(progress) = progress {
                if progress.status == OperationStatus::Processing || progress.status == OperationStatus::Executing {
                    div {
                        style: "
                            height: 3px;
                            background: {props.theme.background};
                        ",
                        
                        div {
                            style: "
                                height: 100%;
                                background: {props.theme.primary};
                                width: {progress.progress_percentage}%;
                                transition: width 0.3s;
                            ",
                        }
                    }
                }
            }
            
            // Expanded details
            if props.is_expanded {
                if let Some(progress) = progress {
                    div {
                        style: "
                            padding: 12px;
                            border-top: 1px solid {props.theme.border};
                            font-size: 13px;
                        ",
                        
                        if let Some(error) = &progress.error {
                            div {
                                style: "
                                    background: {props.theme.error}20;
                                    color: {props.theme.error};
                                    padding: 8px;
                                    border-radius: 4px;
                                    margin-bottom: 8px;
                                ",
                                "Error: {error}"
                            }
                        }
                        
                        div {
                            style: "display: flex; gap: 24px;",
                            
                            div {
                                span {
                                    style: "color: {props.theme.text_secondary};",
                                    "Started: "
                                }
                                span {
                                    style: "color: {props.theme.text};",
                                    "{format_time_ago(progress.started_at)}"
                                }
                            }
                            
                            if let Some(completed_at) = progress.completed_at {
                                div {
                                    span {
                                        style: "color: {props.theme.text_secondary};",
                                        "Duration: "
                                    }
                                    span {
                                        style: "color: {props.theme.text};",
                                        {format_duration(completed_at.duration_since(progress.started_at))}
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

/// Status feed component
#[derive(Props, Clone, PartialEq)]
struct StatusFeedProps {
    operation_progress: HashMap<usize, OperationProgress>,
    theme: ThemeColors,
}

#[component]
fn StatusFeed(props: StatusFeedProps) -> Element {
    // Get recent status updates
    let mut recent_updates: Vec<(usize, &OperationProgress)> = props.operation_progress
        .iter()
        .filter(|(_, p)| p.status == OperationStatus::Processing || p.status == OperationStatus::Executing)
        .map(|(id, p)| (*id, p))
        .collect();
    
    recent_updates.sort_by_key(|(_, p)| std::cmp::Reverse(p.started_at));
    
    rsx! {
        div {
            style: "
                margin-top: 24px;
                background: {props.theme.background_secondary};
                border: 1px solid {props.theme.border};
                border-radius: 8px;
                padding: 16px;
            ",
            
            h4 {
                style: "margin: 0 0 12px; color: {props.theme.text};",
                "🔄 Live Status Feed"
            }
            
            if recent_updates.is_empty() {
                div {
                    style: "
                        text-align: center;
                        padding: 24px;
                        color: {props.theme.text_secondary};
                        font-style: italic;
                    ",
                    "No active operations"
                }
            } else {
                div {
                    style: "display: flex; flex-direction: column; gap: 8px;",
                    
                    for (id, progress) in recent_updates.into_iter().take(5) {
                        StatusFeedItem {
                            operation_id: id,
                            progress: progress.clone(),
                            theme: props.theme.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Status feed item component
#[derive(Props, Clone, PartialEq)]
struct StatusFeedItemProps {
    operation_id: usize,
    progress: OperationProgress,
    theme: ThemeColors,
}

#[component]
fn StatusFeedItem(props: StatusFeedItemProps) -> Element {
    let status_color = match props.progress.status {
        OperationStatus::Processing => props.theme.warning.clone(),
        OperationStatus::Executing => props.theme.primary.clone(),
        _ => props.theme.text_secondary.clone(),
    };
    
    rsx! {
        div {
            style: "
                display: flex;
                align-items: center;
                gap: 12px;
                padding: 8px;
                background: {props.theme.background};
                border-radius: 4px;
            ",
            
            // Animated indicator
            div {
                style: "position: relative;",
                
                div {
                    style: "
                        width: 8px;
                        height: 8px;
                        background: {status_color};
                        border-radius: 50%;
                    ",
                }
                
                div {
                    style: "
                        position: absolute;
                        top: 0;
                        left: 0;
                        width: 8px;
                        height: 8px;
                        background: {status_color};
                        border-radius: 50%;
                        animation: pulse 2s infinite;
                    ",
                }
            }
            
            div {
                style: "flex: 1;",
                
                div {
                    style: "
                        font-size: 13px;
                        color: {props.theme.text};
                    ",
                    "Operation #{props.operation_id}"
                }
                
                div {
                    style: "
                        font-size: 12px;
                        color: {props.theme.text_secondary};
                    ",
                    "{props.progress.current_step}"
                }
            }
            
            div {
                style: "
                    font-size: 12px;
                    color: {props.theme.text_secondary};
                ",
                "{format_time_ago(props.progress.started_at)}"
            }
        }
    }
}

/// Stat item component
#[derive(Props, Clone, PartialEq)]
struct StatItemProps {
    label: &'static str,
    value: String,
    color: String,
    theme: ThemeColors,
}

#[component]
fn StatItem(props: StatItemProps) -> Element {
    rsx! {
        div {
            style: "text-align: center;",
            
            div {
                style: "
                    font-size: 24px;
                    font-weight: bold;
                    color: {props.color};
                    margin-bottom: 4px;
                ",
                "{props.value}"
            }
            
            div {
                style: "
                    font-size: 12px;
                    color: {props.theme.text_secondary};
                    text-transform: uppercase;
                    letter-spacing: 0.5px;
                ",
                "{props.label}"
            }
        }
    }
}

/// Circular progress component
#[derive(Props, Clone, PartialEq)]
struct CircularProgressProps {
    value: f32,
    size: u32,
    theme: ThemeColors,
}

#[component]
fn CircularProgress(props: CircularProgressProps) -> Element {
    let radius = props.size / 2 - 4;
    let circumference = 2.0 * std::f32::consts::PI * radius as f32;
    let stroke_dashoffset = circumference * (1.0 - props.value / 100.0);
    
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            
            circle {
                cx: "{props.size / 2}",
                cy: "{props.size / 2}",
                r: "{radius}",
                fill: "none",
                stroke: props.theme.border,
                stroke_width: "4",
            }
            
            circle {
                cx: "{props.size / 2}",
                cy: "{props.size / 2}",
                r: "{radius}",
                fill: "none",
                stroke: props.theme.primary,
                stroke_width: "4",
                stroke_dasharray: "{circumference}",
                stroke_dashoffset: "{stroke_dashoffset}",
                transform: "rotate(-90 {props.size / 2} {props.size / 2})",
                style: "transition: stroke-dashoffset 0.3s;",
            }
            
            text {
                x: "{props.size / 2}",
                y: "{props.size / 2}",
                text_anchor: "middle",
                dominant_baseline: "middle",
                fill: props.theme.text,
                font_size: "16",
                font_weight: "bold",
                {format!("{:.0}%", props.value)}
            }
        }
    }
}

/// Helper functions

fn calculate_overall_progress(progress: &HashMap<usize, OperationProgress>) -> (f32, ProgressStats) {
    let total = progress.len();
    if total == 0 {
        return (0.0, ProgressStats {
            total: 0,
            completed: 0,
            failed: 0,
            skipped: 0,
            in_progress: 0,
        });
    }
    
    let mut stats = ProgressStats {
        total,
        completed: 0,
        failed: 0,
        skipped: 0,
        in_progress: 0,
    };
    
    for (_, op_progress) in progress {
        match op_progress.status {
            OperationStatus::Completed => stats.completed += 1,
            OperationStatus::Failed => stats.failed += 1,
            OperationStatus::Skipped => stats.skipped += 1,
            OperationStatus::Processing | OperationStatus::Executing => stats.in_progress += 1,
            _ => {}
        }
    }
    
    let progress_percentage = ((stats.completed + stats.failed + stats.skipped) as f32 / total as f32) * 100.0;
    
    (progress_percentage, stats)
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

fn format_time_ago(instant: Instant) -> String {
    let elapsed = instant.elapsed();
    let secs = elapsed.as_secs();
    
    if secs < 5 {
        "just now".to_string()
    } else if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else {
        format!("{}h ago", secs / 3600)
    }
}

fn get_operation_path(operation: &FileOperation) -> String {
    match operation {
        FileOperation::Create { path, .. } |
        FileOperation::Update { path, .. } |
        FileOperation::Append { path, .. } |
        FileOperation::Delete { path } => path.to_string_lossy().to_string(),
        FileOperation::Rename { from, to } => format!("{} → {}", from.display(), to.display()),
    }
}