// Operation Preview Component
// Displays file operations with syntax-highlighted before/after previews

use dioxus::prelude::*;
use crate::consensus::{
    stages::file_aware_curator::FileOperation,
    operation_preview_generator::{OperationPreview as PreviewData, DiffView, FileState},
    ai_operation_parser::FileOperationWithMetadata,
};
use crate::desktop::styles::theme::ThemeColors;
use crate::desktop::components::common::{Button, Card};
use std::path::PathBuf;

/// Operation preview component props
#[derive(Props, Clone, PartialEq)]
pub struct OperationPreviewProps {
    /// The operation to preview
    pub operation: FileOperationWithMetadata,
    
    /// Preview data (if available)
    pub preview: Option<PreviewData>,
    
    /// Theme colors
    pub theme: ThemeColors,
    
    /// Callback when user approves
    pub on_approve: EventHandler<()>,
    
    /// Callback when user rejects
    pub on_reject: EventHandler<()>,
    
    /// Whether this operation is currently selected
    pub is_selected: bool,
}

/// Operation preview component
#[component]
pub fn OperationPreview(props: OperationPreviewProps) -> Element {
    let mut show_diff = use_signal(|| true);
    let mut show_details = use_signal(|| false);
    
    let operation_type = match &props.operation.operation {
        FileOperation::Create { .. } => ("CREATE", "🆕", props.theme.success.clone()),
        FileOperation::Update { .. } => ("UPDATE", "✏️", props.theme.warning.clone()),
        FileOperation::Append { .. } => ("APPEND", "📝", props.theme.info.clone()),
        FileOperation::Delete { .. } => ("DELETE", "🗑️", props.theme.error.clone()),
        FileOperation::Rename { .. } => ("RENAME", "🔄", props.theme.primary.clone()),
    };
    
    let file_path = get_operation_path(&props.operation.operation);
    let border_color = if props.is_selected { props.theme.primary.clone() } else { props.theme.border.clone() };
    
    let show_diff_value = show_diff();
    let show_details_value = show_details();
    
    let diff_button_bg = if show_diff_value { props.theme.primary.clone() } else { "transparent".to_string() };
    let diff_button_color = if show_diff_value { props.theme.background.clone() } else { props.theme.text.clone() };
    let details_button_bg = if show_details_value { props.theme.primary.clone() } else { "transparent".to_string() };
    let details_button_color = if show_details_value { props.theme.background.clone() } else { props.theme.text.clone() };
    
    rsx! {
        div {
            class: "operation-preview",
            style: "
                border: 2px solid {border_color};
                border-radius: 8px;
                margin-bottom: 16px;
                overflow: hidden;
                transition: all 0.2s;
            ",
            
            // Header
            div {
                style: "
                    display: flex;
                    align-items: center;
                    padding: 16px;
                    background: {props.theme.background_secondary};
                    border-bottom: 1px solid {props.theme.border};
                ",
                
                // Operation type badge
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        margin-right: 16px;
                    ",
                    
                    span {
                        style: "font-size: 20px;",
                        "{operation_type.1}"
                    }
                    
                    span {
                        style: "
                            font-weight: bold;
                            color: {operation_type.2};
                            font-size: 12px;
                            text-transform: uppercase;
                            letter-spacing: 0.5px;
                        ",
                        "{operation_type.0}"
                    }
                }
                
                // File path
                div {
                    style: "flex: 1;",
                    
                    code {
                        style: "
                            font-family: monospace;
                            color: {props.theme.text};
                            font-size: 14px;
                        ",
                        "{file_path}"
                    }
                    
                    // Confidence score
                    div {
                        style: "
                            display: inline-flex;
                            align-items: center;
                            margin-left: 12px;
                            font-size: 12px;
                            color: {props.theme.text_secondary};
                        ",
                        
                        "Confidence: "
                        
                        span {
                            style: "
                                color: {get_confidence_color(props.operation.confidence, &props.theme)};
                                font-weight: bold;
                                margin-left: 4px;
                            ",
                            {format!("{:.0}%", props.operation.confidence)}
                        }
                    }
                }
                
                // View toggles
                div {
                    style: "display: flex; gap: 8px;",
                    
                    button {
                        style: "
                            background: {diff_button_bg};
                            color: {diff_button_color};
                            border: 1px solid {props.theme.border};
                            padding: 4px 12px;
                            border-radius: 4px;
                            cursor: pointer;
                            font-size: 12px;
                        ",
                        onclick: move |_| show_diff.set(!show_diff()),
                        "Diff"
                    }
                    
                    button {
                        style: "
                            background: {details_button_bg};
                            color: {details_button_color};
                            border: 1px solid {props.theme.border};
                            padding: 4px 12px;
                            border-radius: 4px;
                            cursor: pointer;
                            font-size: 12px;
                        ",
                        onclick: move |_| show_details.set(!show_details()),
                        "Details"
                    }
                }
            }
            
            // Content
            div {
                style: "padding: 16px;",
                
                // Rationale (if available)
                if let Some(rationale) = &props.operation.rationale {
                    div {
                        style: "
                            background: {props.theme.background_secondary};
                            padding: 12px;
                            border-radius: 6px;
                            margin-bottom: 16px;
                            font-size: 14px;
                            color: {props.theme.text_secondary};
                        ",
                        "💭 {rationale}"
                    }
                }
                
                // Preview content
                if let Some(preview) = &props.preview {
                    if show_diff_value && preview.diff.unified_diff.is_some() {
                        DiffViewComponent {
                            diff: preview.diff.clone(),
                            theme: props.theme.clone(),
                        }
                    } else {
                        div {
                            style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px;",
                            
                            // Before state
                            FileStateView {
                                title: "Before",
                                state: preview.before.clone(),
                                theme: props.theme.clone(),
                                highlight: false,
                            }
                            
                            // After state
                            FileStateView {
                                title: "After",
                                state: preview.after.clone(),
                                theme: props.theme.clone(),
                                highlight: true,
                            }
                        }
                    }
                } else {
                    // Simple operation display without preview
                    OperationDetails {
                        operation: props.operation.operation.clone(),
                        theme: props.theme.clone(),
                    }
                }
                
                // Additional details
                if show_details_value {
                    div {
                        style: "
                            margin-top: 16px;
                            padding-top: 16px;
                            border-top: 1px solid {props.theme.border};
                        ",
                        
                        OperationMetadata {
                            operation: props.operation.clone(),
                            preview: props.preview.clone(),
                            theme: props.theme.clone(),
                        }
                    }
                }
            }
            
            // Action buttons
            div {
                style: "
                    display: flex;
                    gap: 12px;
                    padding: 16px;
                    background: {props.theme.background_secondary};
                    border-top: 1px solid {props.theme.border};
                ",
                
                Button {
                    label: "✓ Approve",
                    variant: "success",
                    theme: props.theme.clone(),
                    on_click: move |_| props.on_approve.call(()),
                }
                
                Button {
                    label: "✗ Reject",
                    variant: "danger",
                    theme: props.theme.clone(),
                    on_click: move |_| props.on_reject.call(()),
                }
                
                div { style: "flex: 1;" }
                
                // Keyboard shortcuts hint
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 12px;
                        font-size: 12px;
                        color: {props.theme.text_secondary};
                    ",
                    
                    span {
                        style: "
                            background: {props.theme.background};
                            padding: 2px 6px;
                            border-radius: 3px;
                            font-family: monospace;
                        ",
                        "Y"
                    }
                    "Approve"
                    
                    span {
                        style: "
                            background: {props.theme.background};
                            padding: 2px 6px;
                            border-radius: 3px;
                            font-family: monospace;
                        ",
                        "N"
                    }
                    "Reject"
                }
            }
        }
    }
}

/// Diff view component
#[derive(Props, Clone, PartialEq)]
struct DiffViewComponentProps {
    diff: DiffView,
    theme: ThemeColors,
}

#[component]
fn DiffViewComponent(props: DiffViewComponentProps) -> Element {
    rsx! {
        div {
            class: "diff-view",
            style: "
                background: {props.theme.background};
                border: 1px solid {props.theme.border};
                border-radius: 6px;
                overflow: hidden;
            ",
            
            // Diff content
            pre {
                style: "
                    margin: 0;
                    padding: 16px;
                    font-family: monospace;
                    font-size: 13px;
                    line-height: 1.5;
                    overflow-x: auto;
                ",
                
                // Render diff chunks
                if let Some(unified) = &props.diff.unified_diff {
                    code {
                        dangerous_inner_html: {render_unified_diff(unified, &props.theme)}
                    }
                }
            }
        }
    }
}

/// File state view component
#[derive(Props, Clone, PartialEq)]
struct FileStateViewProps {
    title: &'static str,
    state: FileState,
    theme: ThemeColors,
    highlight: bool,
}

#[component]
fn FileStateView(props: FileStateViewProps) -> Element {
    let border_color = if props.highlight { props.theme.primary.clone() } else { props.theme.border.clone() };
    
    rsx! {
        div {
            class: "file-state",
            style: "
                border: 1px solid {border_color};
                border-radius: 6px;
                overflow: hidden;
            ",
            
            // Header
            div {
                style: "
                    padding: 8px 12px;
                    background: {props.theme.background_secondary};
                    border-bottom: 1px solid {props.theme.border};
                    font-size: 12px;
                    font-weight: bold;
                    color: {props.theme.text_secondary};
                ",
                "{props.title}"
            }
            
            // Content
            if props.state.exists {
                if let Some(content) = &props.state.content {
                    pre {
                        style: "
                            margin: 0;
                            padding: 12px;
                            font-family: monospace;
                            font-size: 13px;
                            line-height: 1.5;
                            overflow-x: auto;
                            max-height: 300px;
                            overflow-y: auto;
                        ",
                        code {
                            dangerous_inner_html: "{syntax_highlight(content, props.state.language.as_deref(), &props.theme)}"
                        }
                    }
                } else {
                    div {
                        style: "
                            padding: 24px;
                            text-align: center;
                            color: {props.theme.text_secondary};
                            font-style: italic;
                        ",
                        "File exists (content not loaded)"
                    }
                }
            } else {
                div {
                    style: "
                        padding: 24px;
                        text-align: center;
                        color: {props.theme.text_secondary};
                        font-style: italic;
                    ",
                    "File does not exist"
                }
            }
        }
    }
}

/// Operation details component
#[derive(Props, Clone, PartialEq)]
struct OperationDetailsProps {
    operation: FileOperation,
    theme: ThemeColors,
}

#[component]
fn OperationDetails(props: OperationDetailsProps) -> Element {
    match &props.operation {
        FileOperation::Create { path, content } => {
            let path_str = format!("{:?}", path);
            rsx! {
                div {
                    class: "operation-details",
                    style: "
                        background: {props.theme.background_secondary};
                        padding: 16px;
                        border-radius: 6px;
                        font-family: monospace;
                        font-size: 13px;
                    ",
                    
                    div { style: "color: {props.theme.text_secondary};", "Create new file:" }
                    div { style: "color: {props.theme.primary}; margin: 8px 0;", {path_str} }
                    if content.len() < 500 {
                        pre {
                            style: "
                                background: {props.theme.background};
                                padding: 12px;
                                border-radius: 4px;
                                margin-top: 12px;
                                overflow-x: auto;
                            ",
                            code { "{content}" }
                        }
                    } else {
                        div {
                            style: "color: {props.theme.text_secondary}; margin-top: 8px;",
                            {format!("Content: {} bytes", content.len())}
                        }
                    }
                }
            }
        }
        FileOperation::Update { path, content } => {
            let path_str = format!("{:?}", path);
            let content_info = format!("New content: {} bytes", content.len());
            rsx! {
                div {
                    class: "operation-details",
                    style: "
                        background: {props.theme.background_secondary};
                        padding: 16px;
                        border-radius: 6px;
                        font-family: monospace;
                        font-size: 13px;
                    ",
                    
                    div { style: "color: {props.theme.text_secondary};", "Update file:" }
                    div { style: "color: {props.theme.primary}; margin: 8px 0;", {path_str} }
                    div {
                        style: "color: {props.theme.text_secondary}; margin-top: 8px;",
                        {content_info}
                    }
                }
            }
        }
        FileOperation::Delete { path } => {
            let path_str = format!("{:?}", path);
            rsx! {
                div {
                    class: "operation-details",
                    style: "
                        background: {props.theme.background_secondary};
                        padding: 16px;
                        border-radius: 6px;
                        font-family: monospace;
                        font-size: 13px;
                    ",
                    
                    div { style: "color: {props.theme.text_secondary};", "Delete file:" }
                    div { style: "color: {props.theme.error}; margin: 8px 0;", {path_str} }
                }
            }
        }
        FileOperation::Rename { from, to } => {
            let from_str = format!("From: {:?}", from);
            let to_str = format!("To: {:?}", to);
            rsx! {
                div {
                    class: "operation-details",
                    style: "
                        background: {props.theme.background_secondary};
                        padding: 16px;
                        border-radius: 6px;
                        font-family: monospace;
                        font-size: 13px;
                    ",
                    
                    div { style: "color: {props.theme.text_secondary};", "Rename file:" }
                    div { style: "color: {props.theme.error}; margin: 8px 0;", {from_str} }
                    div { style: "color: {props.theme.success}; margin: 8px 0;", {to_str} }
                }
            }
        }
        FileOperation::Append { path, content } => {
            let path_str = format!("{:?}", path);
            let content_info = format!("Content to append: {} bytes", content.len());
            rsx! {
                div {
                    class: "operation-details",
                    style: "
                        background: {props.theme.background_secondary};
                        padding: 16px;
                        border-radius: 6px;
                        font-family: monospace;
                        font-size: 13px;
                    ",
                    
                    div { style: "color: {props.theme.text_secondary};", "Append to file:" }
                    div { style: "color: {props.theme.primary}; margin: 8px 0;", {path_str} }
                    div {
                        style: "color: {props.theme.text_secondary}; margin-top: 8px;",
                        {content_info}
                    }
                }
            }
        }
    }
}

/// Operation metadata component
#[derive(Props, Clone, PartialEq)]
struct OperationMetadataProps {
    operation: FileOperationWithMetadata,
    preview: Option<PreviewData>,
    theme: ThemeColors,
}

#[component]
fn OperationMetadata(props: OperationMetadataProps) -> Element {
    rsx! {
        div {
            class: "operation-metadata",
            style: "
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                gap: 16px;
                font-size: 13px;
            ",
            
            // Source location
            div {
                div {
                    style: "color: {props.theme.text_secondary}; margin-bottom: 4px;",
                    "Source Location"
                }
                div {
                    style: "font-family: monospace;",
                    "Line {props.operation.source_location.line}, Position {props.operation.source_location.start}"
                }
            }
            
            // Dependencies
            if !props.operation.dependencies.is_empty() {
                div {
                    div {
                        style: "color: {props.theme.text_secondary}; margin-bottom: 4px;",
                        "Dependencies"
                    }
                    div {
                        for dep in &props.operation.dependencies {
                            span {
                                style: "
                                    background: {props.theme.background_secondary};
                                    padding: 2px 6px;
                                    border-radius: 3px;
                                    margin-right: 4px;
                                ",
                                "#{dep}"
                            }
                        }
                    }
                }
            }
            
            // Impact analysis
            if let Some(preview) = &props.preview {
                div {
                    div {
                        style: "color: {props.theme.text_secondary}; margin-bottom: 4px;",
                        "Impact"
                    }
                    div {
                        style: "color: {get_impact_color(&preview.impact.risk_level, &props.theme)};",
                        {format!("{:?}", preview.impact.risk_level)}
                    }
                }
            }
        }
    }
}

/// Helper functions

fn get_operation_path(operation: &FileOperation) -> String {
    match operation {
        FileOperation::Create { path, .. } |
        FileOperation::Update { path, .. } |
        FileOperation::Append { path, .. } |
        FileOperation::Delete { path } => path.to_string_lossy().to_string(),
        FileOperation::Rename { from, to } => format!("{} → {}", from.display(), to.display()),
    }
}

fn get_confidence_color(confidence: f32, theme: &ThemeColors) -> String {
    if confidence >= 90.0 {
        theme.success.clone()
    } else if confidence >= 70.0 {
        theme.warning.clone()
    } else {
        theme.error.clone()
    }
}

fn get_impact_color(risk_level: &crate::consensus::operation_preview_generator::RiskLevel, theme: &ThemeColors) -> String {
    use crate::consensus::operation_preview_generator::RiskLevel;
    match risk_level {
        RiskLevel::Low => theme.success.clone(),
        RiskLevel::Medium => theme.warning.clone(),
        RiskLevel::High => theme.error.clone(),
        RiskLevel::Critical => theme.error.clone(),
    }
}

fn render_unified_diff(diff: &str, theme: &ThemeColors) -> String {
    // Simple diff rendering - in production would use proper diff parser
    diff.lines()
        .map(|line| {
            if line.starts_with('+') && !line.starts_with("+++") {
                format!(r#"<span style="color: {};">{}</span>"#, theme.success, html_escape(line))
            } else if line.starts_with('-') && !line.starts_with("---") {
                format!(r#"<span style="color: {};">{}</span>"#, theme.error, html_escape(line))
            } else if line.starts_with("@@") {
                format!(r#"<span style="color: {};">{}</span>"#, theme.primary, html_escape(line))
            } else {
                html_escape(line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn syntax_highlight(content: &str, language: Option<&str>, theme: &ThemeColors) -> String {
    // Simple syntax highlighting - in production would use syntect
    // For now, just escape HTML and add basic styling
    format!(
        r#"<span style="color: {};">{}</span>"#,
        theme.text,
        html_escape(content)
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}