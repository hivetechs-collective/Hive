// Approval Interface Component
// Provides UI for reviewing and approving/rejecting file operations with keyboard shortcuts

use dioxus::prelude::*;
use dioxus::events::{Key, KeyboardEvent};
use crate::consensus::{
    ai_operation_parser::FileOperationWithMetadata,
    operation_preview_generator::OperationPreview as PreviewData,
    operation_clustering::{OperationCluster, ClusterType},
    smart_decision_engine::{ExecutionDecision, UserDecision, UserChoice},
    operation_analysis::OperationAnalysis,
};
use crate::desktop::styles::theme::ThemeColors;
use crate::desktop::components::common::{Button, Card};
use crate::desktop::components::operation_preview::OperationPreview;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Decision made by the user
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalDecision {
    ApproveAll,
    RejectAll,
    ApproveSelected(Vec<usize>),
    RejectSelected(Vec<usize>),
    Deferred,
}

/// Approval interface component props
#[derive(Props, Clone, PartialEq)]
pub struct ApprovalInterfaceProps {
    /// Operations to review
    pub operations: Vec<FileOperationWithMetadata>,
    
    /// Preview data for operations
    pub previews: HashMap<usize, PreviewData>,
    
    /// Clusters of related operations
    pub clusters: Vec<OperationCluster>,
    
    /// AI decisions for operations
    pub ai_decisions: HashMap<usize, ExecutionDecision>,
    
    /// Theme colors
    pub theme: ThemeColors,
    
    /// Callback when user makes decision
    pub on_decision: EventHandler<(ApprovalDecision, UserDecision)>,
    
    /// Callback to request preview generation
    pub on_request_preview: EventHandler<usize>,
    
    /// Whether to show keyboard shortcuts
    pub show_shortcuts: bool,
}

/// Approval interface component
#[component]
pub fn ApprovalInterface(props: ApprovalInterfaceProps) -> Element {
    let mut selected_operation = use_signal(|| 0usize);
    let mut selected_operations = use_signal(|| Vec::<usize>::new());
    let mut multi_select_mode = use_signal(|| false);
    let mut show_ai_reasoning = use_signal(|| true);
    let mut filter_mode = use_signal(|| FilterMode::All);
    let mut time_spent = use_signal(|| Duration::default());
    let start_time = use_signal(|| Instant::now());
    
    // Update time spent
    use_future(move || async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            time_spent.set(start_time().elapsed());
        }
    });
    
    // Keyboard event handling
    let operations_for_handler = props.operations.clone();
    let previews_for_handler = props.previews.clone();
    let on_request_preview = props.on_request_preview.clone();
    let on_decision = props.on_decision.clone();
    
    let keyboard_handler = move |evt: KeyboardEvent| {
        match evt.key() {
            // Navigation
            Key::ArrowUp => {
                if selected_operation() > 0 {
                    selected_operation.set(selected_operation() - 1);
                }
            }
            Key::Character(c) if c == "k" => {
                if selected_operation() > 0 {
                    selected_operation.set(selected_operation() - 1);
                }
            }
            Key::ArrowDown => {
                if selected_operation() < operations_for_handler.len() - 1 {
                    selected_operation.set(selected_operation() + 1);
                }
            }
            Key::Character(c) if c == "j" => {
                if selected_operation() < operations_for_handler.len() - 1 {
                    selected_operation.set(selected_operation() + 1);
                }
            }
            Key::Home => selected_operation.set(0),
            Key::End => selected_operation.set(operations_for_handler.len() - 1),
            
            // Selection
            Key::Character(c) if c == " " => {
                let current = selected_operation();
                let mut selections = selected_operations();
                if selections.contains(&current) {
                    selections.retain(|&x| x != current);
                } else {
                    selections.push(current);
                }
                selected_operations.set(selections);
            }
            
            // Quick actions
            Key::Character(c) if c == "y" || c == "Y" => {
                // Approve current or selected
                if selected_operations().is_empty() {
                    on_decision.call((
                        ApprovalDecision::ApproveSelected(vec![selected_operation()]),
                        create_user_decision(&operations_for_handler[selected_operation()], UserChoice::Execute, time_spent())
                    ));
                } else {
                    let decisions: Vec<UserDecision> = selected_operations()
                        .iter()
                        .map(|&idx| create_user_decision(&operations_for_handler[idx], UserChoice::Execute, time_spent()))
                        .collect();
                    on_decision.call((
                        ApprovalDecision::ApproveSelected(selected_operations()),
                        decisions[0].clone() // For now, just pass first decision
                    ));
                }
            }
            Key::Character(c) if c == "n" || c == "N" => {
                // Reject current or selected
                if selected_operations().is_empty() {
                    on_decision.call((
                        ApprovalDecision::RejectSelected(vec![selected_operation()]),
                        create_user_decision(&operations_for_handler[selected_operation()], UserChoice::Skip, time_spent())
                    ));
                } else {
                    let decisions: Vec<UserDecision> = selected_operations()
                        .iter()
                        .map(|&idx| create_user_decision(&operations_for_handler[idx], UserChoice::Skip, time_spent()))
                        .collect();
                    on_decision.call((
                        ApprovalDecision::RejectSelected(selected_operations()),
                        decisions[0].clone() // For now, just pass first decision
                    ));
                }
            }
            
            // Batch actions
            Key::Character(c) if c == "A" && evt.modifiers().shift() => {
                // Approve all
                use std::time::SystemTime;
                let decision = UserDecision {
                    operation_id: "batch_approve_all".to_string(),
                    analysis: OperationAnalysis::default(),
                    decision: UserChoice::Execute,
                    timestamp: SystemTime::now(),
                    feedback: Some("Batch approval".to_string()),
                };
                props.on_decision.call((ApprovalDecision::ApproveAll, decision));
            }
            Key::Character(c) if c == "R" && evt.modifiers().shift() => {
                // Reject all
                use std::time::SystemTime;
                let decision = UserDecision {
                    operation_id: "batch_reject_all".to_string(),
                    analysis: OperationAnalysis::default(),
                    decision: UserChoice::Skip,
                    timestamp: SystemTime::now(),
                    feedback: Some("Batch rejection".to_string()),
                };
                props.on_decision.call((ApprovalDecision::RejectAll, decision));
            }
            
            // Filter modes
            Key::Character(c) if c == "1" => filter_mode.set(FilterMode::All),
            Key::Character(c) if c == "2" => filter_mode.set(FilterMode::AIApproved),
            Key::Character(c) if c == "3" => filter_mode.set(FilterMode::AIRejected),
            Key::Character(c) if c == "4" => filter_mode.set(FilterMode::HighRisk),
            
            // Toggle modes
            Key::Character(c) if c == "m" => multi_select_mode.set(!multi_select_mode()),
            Key::Character(c) if c == "i" => show_ai_reasoning.set(!show_ai_reasoning()),
            
            // Preview
            Key::Character(c) if c == "p" => {
                if !previews_for_handler.contains_key(&selected_operation()) {
                    on_request_preview.call(selected_operation());
                }
            }
            
            _ => {}
        }
    };
    
    // Filter operations
    let filtered_operations = filter_operations(&props.operations, &props.ai_decisions, filter_mode());
    
    // Statistics
    let stats = calculate_statistics(&props.operations, &props.ai_decisions);
    
    rsx! {
        div {
            class: "approval-interface",
            style: "display: flex; height: 100vh; background: {props.theme.background};",
            onkeydown: keyboard_handler,
            
            // Left sidebar - Operation list
            div {
                style: "
                    width: 300px;
                    background: {props.theme.background_secondary};
                    border-right: 1px solid {props.theme.border};
                    overflow-y: auto;
                ",
                
                // Header
                div {
                    style: "
                        padding: 16px;
                        border-bottom: 1px solid {props.theme.border};
                        position: sticky;
                        top: 0;
                        background: {props.theme.background_secondary};
                    ",
                    
                    h3 {
                        style: "margin: 0 0 12px; color: {props.theme.text};",
                        "Operations ({filtered_operations.len()})"
                    }
                    
                    // Filter tabs
                    div {
                        style: "display: flex; gap: 8px; font-size: 12px;",
                        
                        FilterTab {
                            label: "All",
                            mode: FilterMode::All,
                            current_mode: filter_mode(),
                            count: props.operations.len(),
                            theme: props.theme.clone(),
                            on_click: move |mode| filter_mode.set(mode),
                        }
                        
                        FilterTab {
                            label: "AI ✓",
                            mode: FilterMode::AIApproved,
                            current_mode: filter_mode(),
                            count: stats.ai_approved,
                            theme: props.theme.clone(),
                            on_click: move |mode| filter_mode.set(mode),
                        }
                        
                        FilterTab {
                            label: "AI ✗",
                            mode: FilterMode::AIRejected,
                            current_mode: filter_mode(),
                            count: stats.ai_rejected,
                            theme: props.theme.clone(),
                            on_click: move |mode| filter_mode.set(mode),
                        }
                        
                        FilterTab {
                            label: "High Risk",
                            mode: FilterMode::HighRisk,
                            current_mode: filter_mode(),
                            count: stats.high_risk,
                            theme: props.theme.clone(),
                            on_click: move |mode| filter_mode.set(mode),
                        }
                    }
                }
                
                // Operation list
                div {
                    style: "padding: 8px;",
                    
                    for (idx, &op_idx) in filtered_operations.iter().enumerate() {
                        OperationListItem {
                            operation: props.operations[op_idx].clone(),
                            index: op_idx,
                            is_selected: selected_operation() == op_idx,
                            is_multi_selected: selected_operations().contains(&op_idx),
                            ai_decision: props.ai_decisions.get(&op_idx).cloned(),
                            theme: props.theme.clone(),
                            on_click: move |idx| {
                                selected_operation.set(idx);
                                if multi_select_mode() {
                                    let mut selections = selected_operations();
                                    if selections.contains(&idx) {
                                        selections.retain(|&x| x != idx);
                                    } else {
                                        selections.push(idx);
                                    }
                                    selected_operations.set(selections);
                                }
                            }
                        }
                    }
                }
            }
            
            // Main content area
            div {
                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                
                // Top toolbar
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        padding: 16px;
                        background: {props.theme.background_secondary};
                        border-bottom: 1px solid {props.theme.border};
                    ",
                    
                    // Batch actions
                    div {
                        style: "display: flex; gap: 12px;",
                        
                        Button {
                            label: "✓ Approve All",
                            variant: "success",
                            theme: props.theme.clone(),
                            on_click: move |_| {
                                use std::time::SystemTime;
                                let decision = UserDecision {
                                    operation_id: "batch_approve_all".to_string(),
                                    analysis: OperationAnalysis::default(),
                                    decision: UserChoice::Execute,
                                    timestamp: SystemTime::now(),
                                    feedback: Some("Batch approval".to_string()),
                                };
                                on_decision.call((ApprovalDecision::ApproveAll, decision));
                            }
                        }
                        
                        Button {
                            label: "✗ Reject All",
                            variant: "danger",
                            theme: props.theme.clone(),
                            on_click: move |_| {
                                use std::time::SystemTime;
                                let decision = UserDecision {
                                    operation_id: "batch_reject_all".to_string(),
                                    analysis: OperationAnalysis::default(),
                                    decision: UserChoice::Skip,
                                    timestamp: SystemTime::now(),
                                    feedback: Some("Batch rejection".to_string()),
                                };
                                on_decision.call((ApprovalDecision::RejectAll, decision));
                            }
                        }
                        
                        if !selected_operations().is_empty() {
                            {
                                let operations_for_button = props.operations.clone();
                                let on_decision_for_button = props.on_decision.clone();
                                rsx! {
                                    Button {
                                label: format!("✓ Approve Selected ({})", selected_operations().len()),
                                variant: "primary",
                                theme: props.theme.clone(),
                                on_click: move |_| {
                                    let decisions: Vec<UserDecision> = selected_operations()
                                        .iter()
                                        .map(|&idx| create_user_decision(&operations_for_button[idx], UserChoice::Execute, time_spent()))
                                        .collect();
                                    on_decision_for_button.call((
                                        ApprovalDecision::ApproveSelected(selected_operations()),
                                        decisions[0].clone()
                                    ));
                                }
                                    }
                                }
                            }
                        }
                    }
                    
                    div { style: "flex: 1;" }
                    
                    // Statistics
                    div {
                        style: "
                            display: flex;
                            gap: 24px;
                            font-size: 14px;
                            color: {props.theme.text_secondary};
                        ",
                        
                        div {
                            "Time: "
                            span {
                                style: "color: {props.theme.text}; font-weight: bold;",
                                "{format_duration(time_spent())}"
                            }
                        }
                        
                        div {
                            "AI Agree: "
                            span {
                                style: "color: {props.theme.success}; font-weight: bold;",
                                "{stats.ai_approved}"
                            }
                        }
                        
                        div {
                            "AI Disagree: "
                            span {
                                style: "color: {props.theme.error}; font-weight: bold;",
                                "{stats.ai_rejected}"
                            }
                        }
                    }
                }
                
                // Content area
                div {
                    style: "flex: 1; overflow-y: auto; padding: 20px;",
                    
                    if let Some(operation) = props.operations.get(selected_operation()) {
                        {
                            let operation_for_approve = operation.clone();
                            let operation_for_reject = operation.clone();
                            let on_decision_approve = props.on_decision.clone();
                            let on_decision_reject = props.on_decision.clone();
                            
                            rsx! {
                                // Current operation preview
                                OperationPreview {
                            operation: operation.clone(),
                            preview: props.previews.get(&selected_operation()).cloned(),
                            theme: props.theme.clone(),
                            on_approve: move |_| {
                                on_decision_approve.call((
                                    ApprovalDecision::ApproveSelected(vec![selected_operation()]),
                                    create_user_decision(&operation_for_approve, UserChoice::Execute, time_spent())
                                ));
                            },
                            on_reject: move |_| {
                                on_decision_reject.call((
                                    ApprovalDecision::RejectSelected(vec![selected_operation()]),
                                    create_user_decision(&operation_for_reject, UserChoice::Skip, time_spent())
                                ));
                            },
                            is_selected: true,
                        }
                        
                                // AI reasoning (if enabled)
                                if show_ai_reasoning() {
                                    if let Some(ai_decision) = props.ai_decisions.get(&selected_operation()) {
                                        AIReasoningPanel {
                                            decision: ai_decision.clone(),
                                            theme: props.theme.clone(),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Right sidebar - Keyboard shortcuts
            if props.show_shortcuts {
                div {
                    style: "
                        width: 250px;
                        background: {props.theme.background_secondary};
                        border-left: 1px solid {props.theme.border};
                        padding: 20px;
                        overflow-y: auto;
                    ",
                        
                        h4 {
                            style: "margin: 0 0 16px; color: {props.theme.text};",
                            "⌨️ Keyboard Shortcuts"
                        }
                        
                        ShortcutSection {
                            title: "Navigation",
                            shortcuts: vec![
                                ("↑/k", "Previous operation"),
                                ("↓/j", "Next operation"),
                                ("Home", "First operation"),
                                ("End", "Last operation"),
                            ],
                            theme: props.theme.clone(),
                        }
                        
                        ShortcutSection {
                            title: "Actions",
                            shortcuts: vec![
                                ("y", "Approve current"),
                                ("n", "Reject current"),
                                ("Space", "Toggle selection"),
                                ("p", "Preview operation"),
                            ],
                            theme: props.theme.clone(),
                        }
                        
                        ShortcutSection {
                            title: "Batch Actions",
                            shortcuts: vec![
                                ("Shift+A", "Approve all"),
                                ("Shift+R", "Reject all"),
                            ],
                            theme: props.theme.clone(),
                        }
                        
                        ShortcutSection {
                            title: "Filters",
                            shortcuts: vec![
                                ("1", "Show all"),
                                ("2", "AI approved"),
                                ("3", "AI rejected"),
                                ("4", "High risk"),
                            ],
                            theme: props.theme.clone(),
                        }
                        
                        ShortcutSection {
                            title: "Toggles",
                            shortcuts: vec![
                                ("m", "Multi-select mode"),
                                ("i", "AI reasoning"),
                            ],
                            theme: props.theme.clone(),
                        }
                    }
                }
            }
        }
    }

/// Filter mode for operations
#[derive(Debug, Clone, Copy, PartialEq)]
enum FilterMode {
    All,
    AIApproved,
    AIRejected,
    HighRisk,
}

/// Filter tab component
#[derive(Props, Clone, PartialEq)]
struct FilterTabProps {
    label: &'static str,
    mode: FilterMode,
    current_mode: FilterMode,
    count: usize,
    theme: ThemeColors,
    on_click: EventHandler<FilterMode>,
}

#[component]
fn FilterTab(props: FilterTabProps) -> Element {
    let is_active = props.mode == props.current_mode;
    let bg_color = if is_active { props.theme.primary.clone() } else { "transparent".to_string() };
    let text_color = if is_active { props.theme.background.clone() } else { props.theme.text.clone() };
    let border_color = if is_active { props.theme.primary.clone() } else { props.theme.border.clone() };
    
    rsx! {
        button {
            style: "
                padding: 4px 8px;
                background: {bg_color};
                color: {text_color};
                border: 1px solid {border_color};
                border-radius: 4px;
                cursor: pointer;
                font-size: 12px;
            ",
            onclick: move |_| props.on_click.call(props.mode),
            "{props.label} ({props.count})"
        }
    }
}

/// Operation list item component
#[derive(Props, Clone, PartialEq)]
struct OperationListItemProps {
    operation: FileOperationWithMetadata,
    index: usize,
    is_selected: bool,
    is_multi_selected: bool,
    ai_decision: Option<ExecutionDecision>,
    theme: ThemeColors,
    on_click: EventHandler<usize>,
}

#[component]
fn OperationListItem(props: OperationListItemProps) -> Element {
    use crate::consensus::stages::file_aware_curator::FileOperation;
    
    let (icon, color) = match &props.operation.operation {
        FileOperation::Create { .. } => ("🆕", props.theme.success.clone()),
        FileOperation::Update { .. } => ("✏️", props.theme.warning.clone()),
        FileOperation::Delete { .. } => ("🗑️", props.theme.error.clone()),
        FileOperation::Rename { .. } => ("🔄", props.theme.primary.clone()),
        FileOperation::Append { .. } => ("📝", props.theme.info.clone()),
    };
    
    let ai_indicator = props.ai_decision.as_ref().map(|d| {
        if d.should_execute() {
            ("✓", props.theme.success.clone())
        } else {
            ("✗", props.theme.error.clone())
        }
    });
    
    let bg_color = if props.is_selected { props.theme.background.clone() } else { "transparent".to_string() };
    let border_color = if props.is_selected { 
        props.theme.primary.clone() 
    } else if props.is_multi_selected { 
        props.theme.border.clone() 
    } else { 
        "transparent".to_string() 
    };
    
    rsx! {
        div {
            style: "
                display: flex;
                align-items: center;
                padding: 8px;
                margin-bottom: 4px;
                background: {bg_color};
                border: 1px solid {border_color};
                border-radius: 4px;
                cursor: pointer;
                transition: all 0.1s;
            ",
            onclick: move |_| props.on_click.call(props.index),
            
            // Multi-select checkbox
            if props.is_multi_selected {
                div {
                    style: "
                        width: 16px;
                        height: 16px;
                        background: {props.theme.primary};
                        border-radius: 3px;
                        margin-right: 8px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        color: {props.theme.background};
                        font-size: 12px;
                    ",
                    "✓"
                }
            } else {
                div {
                    style: "
                        width: 16px;
                        height: 16px;
                        border: 1px solid {props.theme.border};
                        border-radius: 3px;
                        margin-right: 8px;
                    ",
                }
            }
            
            // Operation icon
            span {
                style: "font-size: 16px; margin-right: 8px;",
                "{icon}"
            }
            
            // File path
            div {
                style: "flex: 1; min-width: 0;",
                
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
                
                div {
                    style: "
                        font-size: 11px;
                        color: {props.theme.text_secondary};
                        margin-top: 2px;
                    ",
                    {format!("{:.0}% confidence", props.operation.confidence)}
                }
            }
            
            // AI indicator
            if let Some((indicator, color)) = ai_indicator {
                div {
                    style: "
                        color: {color};
                        font-weight: bold;
                        font-size: 14px;
                        margin-left: 8px;
                    ",
                    "{indicator}"
                }
            }
        }
    }
}

/// AI reasoning panel component
#[derive(Props, Clone, PartialEq)]
struct AIReasoningPanelProps {
    decision: ExecutionDecision,
    theme: ThemeColors,
}

#[component]
fn AIReasoningPanel(props: AIReasoningPanelProps) -> Element {
    let decision_color = if props.decision.should_execute() { 
        props.theme.success.clone() 
    } else { 
        props.theme.error.clone() 
    };
    let decision_text = if props.decision.should_execute() { "Execute" } else { "Skip" };
    let confidence_text = format!("{:.0}%", props.decision.confidence() * 100.0);
    
    let content = rsx! {
        div {
            style: "font-size: 14px;",
            
            // Decision
            div {
                style: "margin-bottom: 12px;",
                
                span {
                    style: "color: {props.theme.text_secondary};",
                    "Decision: "
                }
                span {
                    style: "
                        color: {decision_color};
                        font-weight: bold;
                    ",
                    {decision_text}
                }
            }
            
            // Confidence
            div {
                style: "margin-bottom: 12px;",
                
                span {
                    style: "color: {props.theme.text_secondary};",
                    "Confidence: "
                }
                span {
                    style: "font-weight: bold;",
                    {confidence_text}
                }
            }
            
            // Primary reasoning
            if !props.decision.primary_reasoning().is_empty() {
                div {
                    style: "margin-bottom: 12px;",
                    
                    div {
                        style: "color: {props.theme.text_secondary}; margin-bottom: 4px;",
                        "Primary Reasoning:"
                    }
                    ul {
                        style: "margin: 0; padding-left: 20px;",
                        
                        for reason in &props.decision.primary_reasoning() {
                            li {
                                style: "color: {props.theme.text};",
                                "{reason}"
                            }
                        }
                    }
                }
            }
            
            // Risk factors
            if !props.decision.risk_factors().is_empty() {
                div {
                    div {
                        style: "color: {props.theme.text_secondary}; margin-bottom: 4px;",
                        "Risk Factors:"
                    }
                    ul {
                        style: "margin: 0; padding-left: 20px;",
                        
                        for factor in &props.decision.risk_factors() {
                            li {
                                style: "color: {props.theme.error};",
                                "{factor}"
                            }
                        }
                    }
                }
            }
        }
    };
    
    rsx! {
        Card {
            theme: props.theme.clone(),
            title: "🤖 AI Reasoning",
            content: content,
        }
    }
}

/// Keyboard shortcut section component
#[derive(Props, Clone, PartialEq)]
struct ShortcutSectionProps {
    title: &'static str,
    shortcuts: Vec<(&'static str, &'static str)>,
    theme: ThemeColors,
}

#[component]
fn ShortcutSection(props: ShortcutSectionProps) -> Element {
    rsx! {
        div {
            style: "margin-bottom: 20px;",
            
            h5 {
                style: "
                    margin: 0 0 8px;
                    color: {props.theme.text_secondary};
                    font-size: 12px;
                    text-transform: uppercase;
                    letter-spacing: 0.5px;
                ",
                "{props.title}"
            }
            
            div {
                style: "display: flex; flex-direction: column; gap: 4px;",
                
                for (key, description) in props.shortcuts {
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        
                        kbd {
                            style: "
                                background: {props.theme.background};
                                border: 1px solid {props.theme.border};
                                border-radius: 3px;
                                padding: 2px 6px;
                                font-family: monospace;
                                font-size: 11px;
                                min-width: 40px;
                                text-align: center;
                            ",
                            "{key}"
                        }
                        
                        span {
                            style: "font-size: 12px; color: {props.theme.text};",
                            "{description}"
                        }
                    }
                }
            }
        }
    }
}

/// Helper functions

fn filter_operations(
    operations: &[FileOperationWithMetadata],
    ai_decisions: &HashMap<usize, ExecutionDecision>,
    mode: FilterMode,
) -> Vec<usize> {
    operations.iter().enumerate()
        .filter(|(idx, op)| {
            match mode {
                FilterMode::All => true,
                FilterMode::AIApproved => {
                    ai_decisions.get(idx)
                        .map(|d| d.should_execute())
                        .unwrap_or(false)
                }
                FilterMode::AIRejected => {
                    ai_decisions.get(idx)
                        .map(|d| !d.should_execute())
                        .unwrap_or(false)
                }
                FilterMode::HighRisk => {
                    ai_decisions.get(idx)
                        .map(|d| !d.risk_factors().is_empty())
                        .unwrap_or(false)
                }
            }
        })
        .map(|(idx, _)| idx)
        .collect()
}

struct Statistics {
    ai_approved: usize,
    ai_rejected: usize,
    high_risk: usize,
}

fn calculate_statistics(
    operations: &[FileOperationWithMetadata],
    ai_decisions: &HashMap<usize, ExecutionDecision>,
) -> Statistics {
    let mut stats = Statistics {
        ai_approved: 0,
        ai_rejected: 0,
        high_risk: 0,
    };
    
    for (idx, _) in operations.iter().enumerate() {
        if let Some(decision) = ai_decisions.get(&idx) {
            if decision.should_execute() {
                stats.ai_approved += 1;
            } else {
                stats.ai_rejected += 1;
            }
            if !decision.risk_factors().is_empty() {
                stats.high_risk += 1;
            }
        }
    }
    
    stats
}

fn create_user_decision(
    operation: &FileOperationWithMetadata,
    choice: UserChoice,
    _time_taken: Duration,
) -> UserDecision {
    use std::time::SystemTime;
    UserDecision {
        operation_id: format!("op_{}", operation.source_location.start), // Use source location as ID
        analysis: OperationAnalysis::default(), // Would need proper analysis in real implementation
        decision: choice,
        timestamp: SystemTime::now(),
        feedback: None,
    }
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else {
        format!("{}m {}s", secs / 60, secs % 60)
    }
}

fn get_operation_path(operation: &crate::consensus::stages::file_aware_curator::FileOperation) -> String {
    use crate::consensus::stages::file_aware_curator::FileOperation;
    
    match operation {
        FileOperation::Create { path, .. } |
        FileOperation::Update { path, .. } |
        FileOperation::Append { path, .. } |
        FileOperation::Delete { path } => path.to_string_lossy().to_string(),
        FileOperation::Rename { from, to } => format!("{} → {}", from.display(), to.display()),
    }
}