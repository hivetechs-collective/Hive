// Notification System Component
// Shows notifications for auto-accepted operations and other events

use dioxus::prelude::*;
use crate::consensus::{
    stages::file_aware_curator::FileOperation,
    ai_operation_parser::FileOperationWithMetadata,
    smart_decision_engine::ExecutionDecision,
    file_executor::ExecutionResult,
};
use crate::desktop::styles::theme::ThemeColors;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Notification type
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
    AutoAccept,
}

/// Notification data
#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    pub id: usize,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub details: Option<String>,
    pub operation: Option<FileOperationWithMetadata>,
    pub created_at: Instant,
    pub duration: Duration,
    pub is_dismissible: bool,
    pub action: Option<NotificationAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotificationAction {
    pub label: String,
    pub callback_id: String,
}

/// Notification system component props
#[derive(Props, Clone, PartialEq)]
pub struct NotificationSystemProps {
    /// Maximum number of notifications to display
    pub max_notifications: usize,
    
    /// Theme colors
    pub theme: ThemeColors,
    
    /// Position on screen
    pub position: NotificationPosition,
    
    /// Whether to show auto-accept summary
    pub show_auto_accept_summary: bool,
    
    /// Callback when notification action is clicked
    pub on_action: EventHandler<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

/// Notification system component
#[component]
pub fn NotificationSystem(props: NotificationSystemProps) -> Element {
    let mut notifications = use_signal(|| VecDeque::<Notification>::new());
    let mut next_id = use_signal(|| 0usize);
    let mut auto_accept_count = use_signal(|| 0usize);
    let mut last_auto_accept_batch = use_signal(|| Vec::<FileOperationWithMetadata>::new());
    
    // Context for adding notifications
    let context = NotificationContext {
        notifications: notifications.clone(),
        next_id: next_id.clone(),
        auto_accept_count: auto_accept_count.clone(),
        last_auto_accept_batch: last_auto_accept_batch.clone(),
        max_notifications: props.max_notifications,
    };
    use_context_provider(|| context);
    
    // Auto-dismiss notifications
    use_future(move || async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            let now = Instant::now();
            notifications.with_mut(|n| {
                n.retain(|notification| {
                    now.duration_since(notification.created_at) < notification.duration
                });
            });
        }
    });
    
    let position_style = match props.position {
        NotificationPosition::TopRight => "top: 20px; right: 20px;",
        NotificationPosition::TopLeft => "top: 20px; left: 20px;",
        NotificationPosition::BottomRight => "bottom: 20px; right: 20px;",
        NotificationPosition::BottomLeft => "bottom: 20px; left: 20px;",
    };
    
    rsx! {
        div {
            class: "notification-system",
            style: "
                position: fixed;
                {position_style}
                z-index: 1000;
                display: flex;
                flex-direction: column;
                gap: 12px;
                max-width: 400px;
                pointer-events: none;
            ",
            
            // Auto-accept summary (if enabled)
            if props.show_auto_accept_summary && auto_accept_count() > 0 {
                AutoAcceptSummary {
                    count: auto_accept_count(),
                    last_batch: last_auto_accept_batch(),
                    theme: props.theme.clone(),
                    on_clear: move |_| {
                        auto_accept_count.set(0);
                        last_auto_accept_batch.set(Vec::new());
                    }
                }
            }
            
            // Active notifications
            for notification in notifications() {
                NotificationItem {
                    key: "{notification.id}",
                    notification: notification.clone(),
                    theme: props.theme.clone(),
                    on_dismiss: move |id| {
                        notifications.with_mut(|n| {
                            n.retain(|notif| notif.id != id);
                        });
                    },
                    on_action: move |callback_id| {
                        props.on_action.call(callback_id);
                    }
                }
            }
        }
    }
}

/// Notification item component
#[derive(Props, Clone, PartialEq)]
struct NotificationItemProps {
    notification: Notification,
    theme: ThemeColors,
    on_dismiss: EventHandler<usize>,
    on_action: EventHandler<String>,
}

#[component]
fn NotificationItem(props: NotificationItemProps) -> Element {
    let mut elapsed = use_signal(|| Duration::default());
    
    use_future(move || async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            elapsed.set(props.notification.created_at.elapsed());
        }
    });
    
    let (bg_color, icon, icon_color) = match props.notification.notification_type {
        NotificationType::Success => (format!("{}20", props.theme.success.clone()), "✓", props.theme.success.clone()),
        NotificationType::Error => (format!("{}20", props.theme.error.clone()), "✗", props.theme.error.clone()),
        NotificationType::Warning => (format!("{}20", props.theme.warning.clone()), "⚠", props.theme.warning.clone()),
        NotificationType::Info => (format!("{}20", props.theme.info.clone()), "ℹ", props.theme.info.clone()),
        NotificationType::AutoAccept => (format!("{}20", props.theme.primary.clone()), "🤖", props.theme.primary.clone()),
    };
    
    let progress = (elapsed().as_secs_f32() / props.notification.duration.as_secs_f32()).min(1.0);
    
    rsx! {
        div {
            style: "
                background: {props.theme.background_secondary};
                border: 1px solid {props.theme.border};
                border-radius: 8px;
                padding: 16px;
                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
                pointer-events: auto;
                position: relative;
                overflow: hidden;
                animation: slideIn 0.3s ease-out;
            ",
            
            // Progress bar
            div {
                style: "
                    position: absolute;
                    bottom: 0;
                    left: 0;
                    height: 3px;
                    background: {icon_color};
                    width: {100.0 - progress * 100.0}%;
                    transition: width 0.1s linear;
                ",
            }
            
            div {
                style: "display: flex; gap: 12px;",
                
                // Icon
                div {
                    style: "
                        width: 32px;
                        height: 32px;
                        background: {bg_color};
                        border-radius: 50%;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        color: {icon_color};
                        font-size: 16px;
                        font-weight: bold;
                        flex-shrink: 0;
                    ",
                    "{icon}"
                }
                
                // Content
                div {
                    style: "flex: 1; min-width: 0;",
                    
                    div {
                        style: "
                            font-weight: bold;
                            color: {props.theme.text};
                            margin-bottom: 4px;
                        ",
                        "{props.notification.title}"
                    }
                    
                    div {
                        style: "
                            color: {props.theme.text_secondary};
                            font-size: 14px;
                            line-height: 1.4;
                        ",
                        "{props.notification.message}"
                    }
                    
                    if let Some(details) = &props.notification.details {
                        div {
                            style: "
                                margin-top: 8px;
                                padding: 8px;
                                background: {props.theme.background};
                                border-radius: 4px;
                                font-size: 13px;
                                color: {props.theme.text_secondary};
                                font-family: monospace;
                            ",
                            "{details}"
                        }
                    }
                    
                    if let Some(action) = &props.notification.action {
                        {
                            let callback_id = action.callback_id.clone();
                            let on_action_handler = props.on_action.clone();
                            rsx! {
                                button {
                                style: "
                                    margin-top: 8px;
                                    background: {props.theme.primary};
                                    color: {props.theme.background};
                                    border: none;
                                    padding: 4px 12px;
                                    border-radius: 4px;
                                    font-size: 13px;
                                    cursor: pointer;
                                    transition: opacity 0.2s;
                                ",
                                onclick: move |_| on_action_handler.call(callback_id.clone()),
                                "{action.label}"
                                }
                            }
                        }
                    }
                }
                
                // Dismiss button
                if props.notification.is_dismissible {
                    button {
                        style: "
                            background: none;
                            border: none;
                            color: {props.theme.text_secondary};
                            cursor: pointer;
                            padding: 4px;
                            font-size: 16px;
                            opacity: 0.6;
                            transition: opacity 0.2s;
                        ",
                        onclick: move |_| props.on_dismiss.call(props.notification.id),
                        "×"
                    }
                }
            }
        }
    }
}

/// Auto-accept summary component
#[derive(Props, Clone, PartialEq)]
struct AutoAcceptSummaryProps {
    count: usize,
    last_batch: Vec<FileOperationWithMetadata>,
    theme: ThemeColors,
    on_clear: EventHandler<()>,
}

#[component]
fn AutoAcceptSummary(props: AutoAcceptSummaryProps) -> Element {
    let mut show_details = use_signal(|| false);
    
    rsx! {
        div {
            style: "
                background: {props.theme.primary}20;
                border: 1px solid {props.theme.primary};
                border-radius: 8px;
                padding: 12px;
                pointer-events: auto;
                margin-bottom: 8px;
            ",
            
            div {
                style: "display: flex; align-items: center; gap: 8px;",
                
                div {
                    style: "
                        background: {props.theme.primary};
                        color: {props.theme.background};
                        padding: 4px 8px;
                        border-radius: 4px;
                        font-weight: bold;
                        font-size: 14px;
                    ",
                    "{props.count}"
                }
                
                div {
                    style: "flex: 1; color: {props.theme.text}; font-size: 14px;",
                    "Operations auto-accepted this session"
                }
                
                button {
                    style: "
                        background: none;
                        border: none;
                        color: {props.theme.primary};
                        cursor: pointer;
                        font-size: 13px;
                        text-decoration: underline;
                    ",
                    onclick: move |_| show_details.set(!show_details()),
                    if show_details() { "Hide" } else { "Show" }
                }
                
                button {
                    style: "
                        background: none;
                        border: none;
                        color: {props.theme.text_secondary};
                        cursor: pointer;
                        padding: 4px;
                        font-size: 16px;
                    ",
                    onclick: move |_| props.on_clear.call(()),
                    "×"
                }
            }
            
            if show_details() && !props.last_batch.is_empty() {
                div {
                    style: "
                        margin-top: 12px;
                        padding-top: 12px;
                        border-top: 1px solid {props.theme.border};
                    ",
                    
                    div {
                        style: "
                            font-size: 12px;
                            color: {props.theme.text_secondary};
                            margin-bottom: 8px;
                        ",
                        "Recent operations:"
                    }
                    
                    div {
                        style: "display: flex; flex-direction: column; gap: 4px;",
                            
                            for (idx, op) in props.last_batch.iter().enumerate().take(5) {
                                OperationSummaryItem {
                                    operation: op.clone(),
                                    theme: props.theme.clone(),
                                }
                            }
                            
                            if props.last_batch.len() > 5 {
                                div {
                                    style: "
                                        font-size: 12px;
                                        color: {props.theme.text_secondary};
                                        font-style: italic;
                                        margin-top: 4px;
                                    ",
                                    {format!("... and {} more", props.last_batch.len() - 5)}
                                }
                            }
                        }
                }
            }
        }
    }
}

/// Operation summary item component
#[derive(Props, Clone, PartialEq)]
struct OperationSummaryItemProps {
    operation: FileOperationWithMetadata,
    theme: ThemeColors,
}

#[component]
fn OperationSummaryItem(props: OperationSummaryItemProps) -> Element {
    let (icon, color) = match &props.operation.operation {
        FileOperation::Create { .. } => ("🆕", props.theme.success),
        FileOperation::Update { .. } => ("✏️", props.theme.warning),
        FileOperation::Delete { .. } => ("🗑️", props.theme.error),
        FileOperation::Rename { .. } => ("🔄", props.theme.primary),
        FileOperation::Append { .. } => ("📝", props.theme.info),
    };
    
    rsx! {
        div {
            style: "
                display: flex;
                align-items: center;
                gap: 8px;
                padding: 4px 8px;
                background: {props.theme.background};
                border-radius: 4px;
                font-size: 12px;
            ",
            
            span { "{icon}" }
            
            span {
                style: "
                    color: {props.theme.text};
                    white-space: nowrap;
                    overflow: hidden;
                    text-overflow: ellipsis;
                ",
                "{get_operation_summary(&props.operation.operation)}"
            }
            
            span {
                style: "
                    color: {color};
                    font-weight: bold;
                    margin-left: auto;
                ",
                {format!("{:.0}%", props.operation.confidence)}
            }
        }
    }
}

/// Notification context for other components to add notifications
#[derive(Clone, Copy)]
pub struct NotificationContext {
    notifications: Signal<VecDeque<Notification>>,
    next_id: Signal<usize>,
    auto_accept_count: Signal<usize>,
    last_auto_accept_batch: Signal<Vec<FileOperationWithMetadata>>,
    max_notifications: usize,
}

impl NotificationContext {
    pub fn add_notification(
        &self,
        notification_type: NotificationType,
        title: String,
        message: String,
        details: Option<String>,
        operation: Option<FileOperationWithMetadata>,
    ) {
        // TODO: Implement notification context methods properly with Dioxus Signal
        // For now, this is commented out to allow compilation
        // The issue is that Signal<T> requires special handling in Dioxus
        /*
        let id = (self.next_id)();
        self.next_id.set(id + 1);
        
        let notification = Notification {
            id,
            notification_type: notification_type.clone(),
            title,
            message,
            details,
            operation: operation.clone(),
            created_at: Instant::now(),
            duration: match notification_type {
                NotificationType::Error => Duration::from_secs(10),
                NotificationType::AutoAccept => Duration::from_secs(3),
                _ => Duration::from_secs(5),
            },
            is_dismissible: true,
            action: None,
        };
        
        self.notifications.with_mut(|n| {
            n.push_front(notification);
            if n.len() > self.max_notifications {
                n.pop_back();
            }
        });
        
        if notification_type == NotificationType::AutoAccept {
            self.auto_accept_count.set((self.auto_accept_count)() + 1);
            if let Some(op) = operation {
                self.last_auto_accept_batch.with_mut(|batch| batch.push(op));
            }
        }
        */
    }
    
    pub fn add_auto_accept_batch(&self, operations: Vec<FileOperationWithMetadata>) {
        // TODO: Implement notification context methods properly with Dioxus Signal
        // For now, this is commented out to allow compilation
        /*
        let count = operations.len();
        self.auto_accept_count.set((self.auto_accept_count)() + count);
        self.last_auto_accept_batch.set(operations.clone());
        
        let id = (self.next_id)();
        self.next_id.set(id + 1);
        
        let notification = Notification {
            id,
            notification_type: NotificationType::AutoAccept,
            title: format!("🤖 Auto-Accepted {} Operations", count),
            message: format!("{} operations were automatically executed", count),
            details: Some(format_batch_summary(&operations)),
            operation: None,
            created_at: Instant::now(),
            duration: Duration::from_secs(5),
            is_dismissible: true,
            action: Some(NotificationAction {
                label: "View Details".to_string(),
                callback_id: format!("view_batch_{}", id),
            }),
        };
        
        self.notifications.with_mut(|n| {
            n.push_front(notification);
            if n.len() > self.max_notifications {
                n.pop_back();
            }
        });
        */
    }
}

/// Helper function to add a notification from anywhere in the app
pub fn notify(
    notification_type: NotificationType,
    title: String,
    message: String,
    details: Option<String>,
) {
    let context = use_context::<NotificationContext>();
    context.add_notification(notification_type, title, message, details, None);
}

/// Helper function to notify about auto-accepted operations
pub fn notify_auto_accept_batch(operations: Vec<FileOperationWithMetadata>) {
    let context = use_context::<NotificationContext>();
    context.add_auto_accept_batch(operations);
}

/// Helper functions

fn format_batch_summary(operations: &[FileOperationWithMetadata]) -> String {
    use std::collections::HashMap;
    
    let mut counts: HashMap<&str, usize> = HashMap::new();
    
    for op in operations {
        let op_type = match &op.operation {
            FileOperation::Create { .. } => "created",
            FileOperation::Update { .. } => "updated",
            FileOperation::Delete { .. } => "deleted",
            FileOperation::Rename { .. } => "renamed",
            FileOperation::Append { .. } => "appended",
        };
        *counts.entry(op_type).or_insert(0) += 1;
    }
    
    let mut summary_parts = Vec::new();
    for (op_type, count) in counts {
        summary_parts.push(format!("{} {}", count, op_type));
    }
    
    summary_parts.join(", ")
}

fn get_operation_summary(operation: &FileOperation) -> String {
    match operation {
        FileOperation::Create { path, .. } => format!("Create {}", path.file_name().unwrap_or_default().to_string_lossy()),
        FileOperation::Update { path, .. } => format!("Update {}", path.file_name().unwrap_or_default().to_string_lossy()),
        FileOperation::Delete { path } => format!("Delete {}", path.file_name().unwrap_or_default().to_string_lossy()),
        FileOperation::Rename { from, to } => format!(
            "{} → {}",
            from.file_name().unwrap_or_default().to_string_lossy(),
            to.file_name().unwrap_or_default().to_string_lossy()
        ),
        FileOperation::Append { path, .. } => format!("Append to {}", path.file_name().unwrap_or_default().to_string_lossy()),
    }
}

/// CSS for animations (add to your global styles)
const NOTIFICATION_STYLES: &str = r#"
@keyframes slideIn {
    from {
        transform: translateX(100%);
        opacity: 0;
    }
    to {
        transform: translateX(0);
        opacity: 1;
    }
}

@keyframes pulse {
    0% {
        transform: scale(1);
        opacity: 1;
    }
    50% {
        transform: scale(1.5);
        opacity: 0.5;
    }
    100% {
        transform: scale(1);
        opacity: 1;
    }
}
"#;