//! Enhanced VS Code-style Status Bar
//! Based on VS Code's src/vs/workbench/browser/parts/statusbar/

use chrono::{DateTime, Utc};
use dioxus::prelude::*;

/// Status bar item alignment
#[derive(Clone, Debug, PartialEq)]
pub enum StatusBarAlignment {
    Left,
    Right,
}

/// Status bar item with priority and content
pub struct StatusBarItem {
    pub id: String,
    pub text: String,
    pub tooltip: Option<String>,
    pub icon: Option<String>,
    pub alignment: StatusBarAlignment,
    pub priority: i32, // Higher priority items appear first
    pub on_click: Option<Box<dyn Fn() + 'static>>,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
}

impl std::fmt::Debug for StatusBarItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatusBarItem")
            .field("id", &self.id)
            .field("text", &self.text)
            .field("tooltip", &self.tooltip)
            .field("icon", &self.icon)
            .field("alignment", &self.alignment)
            .field("priority", &self.priority)
            .field("on_click", &self.on_click.is_some())
            .field("background_color", &self.background_color)
            .field("foreground_color", &self.foreground_color)
            .finish()
    }
}

impl PartialEq for StatusBarItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Clone for StatusBarItem {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            text: self.text.clone(),
            tooltip: self.tooltip.clone(),
            icon: self.icon.clone(),
            alignment: self.alignment.clone(),
            priority: self.priority,
            on_click: None, // Cannot clone closures, so we reset this
            background_color: self.background_color.clone(),
            foreground_color: self.foreground_color.clone(),
        }
    }
}

/// Status bar state
#[derive(Clone, Debug)]
pub struct StatusBarState {
    pub items: Vec<StatusBarItem>,
    pub background_color: Option<String>, // Override background for entire bar
    pub show_feedback: bool,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            items: vec![
                // Left side items
                StatusBarItem {
                    id: "repository-selector".to_string(),
                    text: "hive".to_string(), // Default repository name
                    tooltip: Some("Select Repository".to_string()),
                    icon: Some("folder".to_string()),
                    alignment: StatusBarAlignment::Left,
                    priority: 110, // Highest priority - appears first
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
                StatusBarItem {
                    id: "git-branch".to_string(),
                    text: "main".to_string(),
                    tooltip: Some("Git: main (click to checkout branch)".to_string()),
                    icon: Some("source-control".to_string()),
                    alignment: StatusBarAlignment::Left,
                    priority: 100,
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
                StatusBarItem {
                    id: "git-sync".to_string(),
                    text: "↓0 ↑0".to_string(),
                    tooltip: Some("Synchronize Changes".to_string()),
                    icon: Some("sync".to_string()),
                    alignment: StatusBarAlignment::Left,
                    priority: 99,
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
                StatusBarItem {
                    id: "problems".to_string(),
                    text: "0".to_string(),
                    tooltip: Some("No Problems".to_string()),
                    icon: Some("error".to_string()),
                    alignment: StatusBarAlignment::Left,
                    priority: 98,
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
                StatusBarItem {
                    id: "warnings".to_string(),
                    text: "0".to_string(),
                    tooltip: Some("No Warnings".to_string()),
                    icon: Some("warning".to_string()),
                    alignment: StatusBarAlignment::Left,
                    priority: 97,
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
                // Right side items
                StatusBarItem {
                    id: "consensus-status".to_string(),
                    text: "Consensus: Ready".to_string(),
                    tooltip: Some("4-Stage Consensus Engine Status".to_string()),
                    icon: Some("circuit-board".to_string()),
                    alignment: StatusBarAlignment::Right,
                    priority: 100,
                    on_click: None,
                    background_color: None,
                    foreground_color: Some("#FFC107".to_string()), // HiveTechs yellow
                },
                StatusBarItem {
                    id: "api-usage".to_string(),
                    text: "$0.0234".to_string(),
                    tooltip: Some("API Usage Today: $0.0234".to_string()),
                    icon: Some("dashboard".to_string()),
                    alignment: StatusBarAlignment::Right,
                    priority: 99,
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
                StatusBarItem {
                    id: "cursor-position".to_string(),
                    text: "Ln 1, Col 1".to_string(),
                    tooltip: Some("Go to Line/Column".to_string()),
                    icon: None,
                    alignment: StatusBarAlignment::Right,
                    priority: 90,
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
                StatusBarItem {
                    id: "indentation".to_string(),
                    text: "Spaces: 4".to_string(),
                    tooltip: Some("Select Indentation".to_string()),
                    icon: None,
                    alignment: StatusBarAlignment::Right,
                    priority: 85,
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
                StatusBarItem {
                    id: "encoding".to_string(),
                    text: "UTF-8".to_string(),
                    tooltip: Some("Select Encoding".to_string()),
                    icon: None,
                    alignment: StatusBarAlignment::Right,
                    priority: 80,
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
                StatusBarItem {
                    id: "language-mode".to_string(),
                    text: "Rust".to_string(),
                    tooltip: Some("Select Language Mode".to_string()),
                    icon: None,
                    alignment: StatusBarAlignment::Right,
                    priority: 75,
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
                StatusBarItem {
                    id: "feedback".to_string(),
                    text: "".to_string(),
                    tooltip: Some("Tweet Feedback".to_string()),
                    icon: Some("feedback".to_string()),
                    alignment: StatusBarAlignment::Right,
                    priority: 50,
                    on_click: None,
                    background_color: None,
                    foreground_color: None,
                },
            ],
            background_color: None,
            show_feedback: true,
        }
    }
}

/// Enhanced Status Bar component
#[component]
pub fn EnhancedStatusBar(
    state: Signal<StatusBarState>,
    on_item_click: EventHandler<String>,
    on_git_branch_click: Option<EventHandler<()>>,
    on_repository_selector_click: Option<EventHandler<()>>,
) -> Element {
    let status_state = state.read();

    // Sort items by alignment and priority
    let mut left_items: Vec<_> = status_state
        .items
        .iter()
        .filter(|item| item.alignment == StatusBarAlignment::Left)
        .collect();
    left_items.sort_by(|a, b| b.priority.cmp(&a.priority));

    let mut right_items: Vec<_> = status_state
        .items
        .iter()
        .filter(|item| item.alignment == StatusBarAlignment::Right)
        .collect();
    right_items.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Determine background color
    let background_style = if let Some(color) = &status_state.background_color {
        format!("background-color: {};", color)
    } else {
        String::new()
    };

    rsx! {
        div {
            class: "status-bar",
            style: "{background_style}",
            role: "status",
            aria_label: "Status Bar",

            // Left section
            div {
                class: "status-bar-section status-bar-left",

                for item in left_items {
                    StatusBarItemComponent {
                        item: item.clone(),
                        on_click: {
                            // Special handling for specific items
                            if item.id == "repository-selector" {
                                if let Some(ref handler) = on_repository_selector_click {
                                    handler.clone()
                                } else {
                                    let id = item.id.clone();
                                    EventHandler::new(move |_| on_item_click.call(id.clone()))
                                }
                            } else if item.id == "git-branch" {
                                if let Some(ref handler) = on_git_branch_click {
                                    handler.clone()
                                } else {
                                    let id = item.id.clone();
                                    EventHandler::new(move |_| on_item_click.call(id.clone()))
                                }
                            } else {
                                let id = item.id.clone();
                                EventHandler::new(move |_| on_item_click.call(id.clone()))
                            }
                        },
                    }
                }
            }

            // Right section
            div {
                class: "status-bar-section status-bar-right",

                for item in right_items {
                    StatusBarItemComponent {
                        item: item.clone(),
                        on_click: {
                            let id = item.id.clone();
                            move |_| on_item_click.call(id.clone())
                        },
                    }
                }
            }
        }
    }
}

/// Individual status bar item
#[component]
fn StatusBarItemComponent(item: StatusBarItem, on_click: EventHandler<()>) -> Element {
    let has_click_handler = item.on_click.is_some();
    let item_class = if has_click_handler {
        "status-bar-item clickable"
    } else {
        "status-bar-item"
    };

    // Build inline styles
    let mut styles = Vec::new();
    if let Some(bg) = &item.background_color {
        styles.push(format!("background-color: {}", bg));
    }
    if let Some(fg) = &item.foreground_color {
        styles.push(format!("color: {}", fg));
    }
    let style_str = styles.join("; ");

    // Map icon names to codicons
    let icon_class = item.icon.as_ref().map(|icon| match icon.as_str() {
        "folder" => "codicon-folder",
        "source-control" => "codicon-source-control",
        "sync" => "codicon-sync",
        "error" => "codicon-error",
        "warning" => "codicon-warning",
        "circuit-board" => "codicon-circuit-board",
        "dashboard" => "codicon-dashboard",
        "feedback" => "codicon-feedback",
        _ => "codicon-circle",
    });

    rsx! {
        div {
            class: "{item_class}",
            style: "{style_str}",
            title: "{item.tooltip.as_ref().unwrap_or(&item.text)}",
            onclick: move |_| {
                if has_click_handler {
                    on_click.call(());
                }
            },

            if let Some(icon) = icon_class {
                span {
                    class: "status-bar-icon codicon {icon}",
                }
            }

            if !item.text.is_empty() {
                span {
                    class: "status-bar-text",
                    "{item.text}"
                }
            }
        }
    }
}

/// Update status bar items dynamically
impl StatusBarState {
    pub fn update_item(&mut self, id: &str, text: String) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.text = text;
        }
    }

    pub fn update_consensus_status(&mut self, status: &str, color: Option<String>) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == "consensus-status") {
            item.text = format!("Consensus: {}", status);
            item.foreground_color = color;
        }
    }

    pub fn update_api_usage(&mut self, cost: f64) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == "api-usage") {
            item.text = format!("${:.4}", cost);
            item.tooltip = Some(format!("API Usage Today: ${:.4}", cost));
        }
    }

    pub fn update_problems(&mut self, errors: u32, warnings: u32) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == "problems") {
            item.text = errors.to_string();
            item.tooltip = Some(if errors == 0 {
                "No Problems".to_string()
            } else {
                format!("{} Problem{}", errors, if errors == 1 { "" } else { "s" })
            });
        }

        if let Some(item) = self.items.iter_mut().find(|i| i.id == "warnings") {
            item.text = warnings.to_string();
            item.tooltip = Some(if warnings == 0 {
                "No Warnings".to_string()
            } else {
                format!(
                    "{} Warning{}",
                    warnings,
                    if warnings == 1 { "" } else { "s" }
                )
            });
        }
    }

    pub fn set_remote_status(&mut self) {
        self.background_color = Some("#16825d".to_string()); // VS Code remote color
    }

    pub fn set_debug_status(&mut self) {
        self.background_color = Some("#CC6633".to_string()); // VS Code debug color
    }

    pub fn update_git_branch(&mut self, branch: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == "git-branch") {
            item.text = branch.to_string();
            item.tooltip = Some(format!("Git: {} (click to checkout branch)", branch));
        }
    }

    pub fn update_git_sync_status(&mut self, ahead: u32, behind: u32) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == "git-sync") {
            item.text = format!("↓{} ↑{}", behind, ahead);
            item.tooltip = Some("Synchronize Changes".to_string());
        }
    }

    pub fn update_repository_selector(&mut self, name: &str, path: &str) {
        if let Some(item) = self
            .items
            .iter_mut()
            .find(|i| i.id == "repository-selector")
        {
            item.text = name.to_string();
            item.tooltip = Some(format!("Repository: {} ({})", name, path));
        }
    }
}

/// Status Bar CSS styles
pub const STATUS_BAR_STYLES: &str = r#"
/* Status Bar Container */
.status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 22px;
    background-color: var(--vscode-statusBar-background, #007acc);
    color: var(--vscode-statusBar-foreground, #ffffff);
    font-size: 12px;
    padding: 0 10px;
    user-select: none;
    position: relative;
    z-index: 100;
}

/* Status bar sections */
.status-bar-section {
    display: flex;
    align-items: center;
    gap: 15px;
    height: 100%;
}

.status-bar-left {
    flex: 1;
    justify-content: flex-start;
}

.status-bar-right {
    flex-shrink: 0;
    justify-content: flex-end;
}

/* Status bar items */
.status-bar-item {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 100%;
    padding: 0 5px;
    transition: background-color 0.1s ease;
    white-space: nowrap;
}

.status-bar-item.clickable {
    cursor: pointer;
}

.status-bar-item.clickable:hover {
    background-color: rgba(255, 255, 255, 0.12);
}

.status-bar-item.clickable:active {
    background-color: rgba(255, 255, 255, 0.18);
}

/* Icons */
.status-bar-icon {
    font-size: 14px;
    line-height: 1;
}

/* Text */
.status-bar-text {
    line-height: 22px;
}

/* Problem/warning colors */
.status-bar-item .codicon-error {
    color: #f48771;
}

.status-bar-item .codicon-warning {
    color: #cca700;
}

/* Consensus status (HiveTechs specific) */
.status-bar-item[title*="Consensus"] {
    font-weight: 500;
}

/* Remote indicator style */
.status-bar.remote {
    background-color: #16825d;
}

/* Debug mode style */
.status-bar.debug {
    background-color: #CC6633;
}

/* Animations */
@keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.6; }
    100% { opacity: 1; }
}

.status-bar-item.updating {
    animation: pulse 1.5s ease-in-out infinite;
}

/* Feedback icon special style */
.status-bar-item[title*="Feedback"] {
    opacity: 0.6;
}

.status-bar-item[title*="Feedback"]:hover {
    opacity: 1;
}

/* Tooltip enhancement */
.status-bar-item[title]:hover::after {
    content: attr(title);
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    margin-bottom: 4px;
    padding: 4px 8px;
    background-color: var(--vscode-tooltip-background, #252526);
    color: var(--vscode-tooltip-foreground, #ccc);
    border: 1px solid var(--vscode-tooltip-border, #454545);
    border-radius: 3px;
    font-size: 12px;
    white-space: nowrap;
    pointer-events: none;
    z-index: 1000;
    opacity: 0;
    animation: fadeIn 0.1s ease-in 0.5s forwards;
}

/* Progress indicator */
.status-bar-progress {
    position: absolute;
    top: 0;
    left: 0;
    height: 2px;
    background-color: var(--hivetechs-yellow, #FFC107);
    transition: width 0.3s ease;
}

/* Focus indicator */
.status-bar-item:focus {
    outline: 1px solid var(--hivetechs-yellow, #FFC107);
    outline-offset: -1px;
}

/* Repository selector special styling */
.status-bar-item[title*="Repository"] {
    font-weight: 600;
    padding-right: 10px;
    border-right: 1px solid rgba(255, 255, 255, 0.2);
    margin-right: 10px;
}
"#;
