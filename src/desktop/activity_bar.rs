//! VS Code-style Activity Bar for Hive Consensus
//! Based on VS Code's src/vs/workbench/browser/parts/activitybar/

use dioxus::prelude::*;

/// Activity Bar item with badge support
#[derive(Clone, Debug, PartialEq)]
pub struct ActivityBarItem {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub badge: Option<ActivityBadge>,
    pub enabled: bool,
}

/// Badge for activity bar items (e.g., notification count)
#[derive(Clone, Debug, PartialEq)]
pub struct ActivityBadge {
    pub value: String,
    pub is_number: bool,
}

/// Activity Bar state
#[derive(Clone, Debug)]
pub struct ActivityBarState {
    pub items: Vec<ActivityBarItem>,
    pub active_item: Option<String>,
    pub pinned_items: Vec<String>,
    pub collapsed: bool,
}

impl Default for ActivityBarState {
    fn default() -> Self {
        Self {
            items: vec![
                ActivityBarItem {
                    id: "explorer".to_string(),
                    title: "Explorer (Ctrl+Shift+E)".to_string(),
                    icon: "files".to_string(),
                    badge: None,
                    enabled: true,
                },
                ActivityBarItem {
                    id: "search".to_string(),
                    title: "Search (Ctrl+Shift+F)".to_string(),
                    icon: "search".to_string(),
                    badge: None,
                    enabled: true,
                },
                ActivityBarItem {
                    id: "consensus".to_string(),
                    title: "Consensus Intelligence".to_string(),
                    icon: "brain".to_string(),
                    badge: Some(ActivityBadge {
                        value: "4".to_string(),
                        is_number: true,
                    }),
                    enabled: true,
                },
                ActivityBarItem {
                    id: "models".to_string(),
                    title: "Model Browser".to_string(),
                    icon: "chip".to_string(),
                    badge: None,
                    enabled: true,
                },
                ActivityBarItem {
                    id: "extensions".to_string(),
                    title: "Extensions (Ctrl+Shift+X)".to_string(),
                    icon: "extensions".to_string(),
                    badge: None,
                    enabled: true,
                },
            ],
            active_item: Some("explorer".to_string()),
            pinned_items: vec!["explorer".to_string(), "consensus".to_string()],
            collapsed: false,
        }
    }
}

/// VS Code-style Activity Bar component
#[component]
pub fn ActivityBar(
    state: Signal<ActivityBarState>,
    on_item_click: EventHandler<String>,
    on_settings_click: EventHandler<()>,
    on_accounts_click: EventHandler<()>,
) -> Element {
    let bar_state = state.read();

    rsx! {
        div {
            class: "activity-bar",
            role: "navigation",
            aria_label: "Activity Bar",

            // Top section with main items
            div {
                class: "activity-bar-content",

                ul {
                    class: "actions-container",
                    role: "list",

                    for item in &bar_state.items {
                        ActivityBarItemComponent {
                            item: item.clone(),
                            is_active: bar_state.active_item.as_ref() == Some(&item.id),
                            on_click: {
                                let id = item.id.clone();
                                move |_| on_item_click.call(id.clone())
                            },
                        }
                    }
                }
            }

            // Bottom section with settings/accounts
            div {
                class: "activity-bar-bottom",

                ul {
                    class: "actions-container",
                    role: "list",

                    // Accounts button
                    li {
                        class: "action-item",
                        role: "listitem",

                        button {
                            class: "action-label",
                            title: "Accounts",
                            onclick: move |_| on_accounts_click.call(()),

                            span {
                                class: "codicon codicon-account",
                            }
                        }
                    }

                    // Settings button
                    li {
                        class: "action-item",
                        role: "listitem",

                        button {
                            class: "action-label",
                            title: "Manage (Ctrl+,)",
                            onclick: move |_| on_settings_click.call(()),

                            span {
                                class: "codicon codicon-settings-gear",
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Individual activity bar item
#[component]
fn ActivityBarItemComponent(
    item: ActivityBarItem,
    is_active: bool,
    on_click: EventHandler<()>,
) -> Element {
    let item_class = if is_active {
        "action-item checked"
    } else {
        "action-item"
    };

    let label_class = if item.enabled {
        "action-label"
    } else {
        "action-label disabled"
    };

    // Map icon names to codicons
    let icon_class = match item.icon.as_str() {
        "files" => "codicon-files",
        "search" => "codicon-search",
        "brain" => "codicon-lightbulb", // Using lightbulb for consensus
        "chip" => "codicon-circuit-board",
        "extensions" => "codicon-extensions",
        "source-control" => "codicon-source-control",
        "debug" => "codicon-debug-alt",
        _ => "codicon-circle",
    };

    rsx! {
        li {
            class: "{item_class}",
            role: "listitem",

            button {
                class: "{label_class}",
                title: "{item.title}",
                aria_label: "{item.title}",
                aria_pressed: "{is_active}",
                disabled: !item.enabled,
                onclick: move |_| on_click.call(()),

                span {
                    class: "codicon {icon_class}",
                }

                if let Some(badge) = &item.badge {
                    div {
                        class: "badge",
                        class: if badge.is_number { "badge-number" } else { "" },
                        "{badge.value}"
                    }
                }
            }
        }
    }
}

/// Activity Bar CSS styles
pub const ACTIVITY_BAR_STYLES: &str = r#"
/* Activity Bar Container */
.activity-bar {
    position: relative;
    width: 48px;
    height: 100%;
    background-color: var(--vscode-activityBar-background, #333333);
    color: var(--vscode-activityBar-foreground, #fff);
    display: flex;
    flex-direction: column;
    align-items: center;
}

/* Content sections */
.activity-bar-content {
    flex: 1;
    width: 100%;
    overflow-y: auto;
    overflow-x: hidden;
}

.activity-bar-bottom {
    width: 100%;
    border-top: 1px solid var(--vscode-activityBar-border, #1e1e1e);
}

/* Actions container */
.actions-container {
    list-style: none;
    padding: 0;
    margin: 0;
    width: 100%;
}

/* Action items */
.action-item {
    position: relative;
    width: 48px;
    height: 48px;
}

.action-item.checked::before {
    content: '';
    position: absolute;
    left: 0;
    top: 4px;
    bottom: 4px;
    width: 3px;
    background-color: var(--hivetechs-yellow, #FFC107);
    border-radius: 0 2px 2px 0;
}

/* Action labels (buttons) */
.action-label {
    display: flex;
    width: 100%;
    height: 100%;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    position: relative;
    font-size: 24px;
    transition: color 0.1s ease;
}

.action-label:hover:not(.disabled) {
    color: var(--vscode-activityBar-activeForeground, #fff);
}

.action-label.disabled {
    opacity: 0.4;
    cursor: default;
}

.action-item.checked .action-label {
    color: var(--vscode-activityBar-activeForeground, #fff);
}

/* Badges */
.badge {
    position: absolute;
    top: 8px;
    right: 8px;
    min-width: 18px;
    height: 18px;
    line-height: 18px;
    font-size: 11px;
    font-weight: 600;
    text-align: center;
    border-radius: 20px;
    padding: 0 5px;
    box-sizing: border-box;
    background-color: var(--hivetechs-yellow, #FFC107);
    color: #000;
}

.badge.badge-number {
    padding: 0 4px;
}

/* Scrollbar styling */
.activity-bar-content::-webkit-scrollbar {
    width: 0;
}

/* Focus styles */
.action-label:focus {
    outline: 1px solid var(--hivetechs-yellow, #FFC107);
    outline-offset: -1px;
}

/* Tooltip styling */
.action-label[title]:hover::after {
    content: attr(title);
    position: absolute;
    left: 100%;
    top: 50%;
    transform: translateY(-50%);
    margin-left: 4px;
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

@keyframes fadeIn {
    to { opacity: 1; }
}

/* Drag and drop support */
.action-item.drop-target {
    background-color: var(--vscode-list-dropBackground, rgba(255, 193, 7, 0.1));
}

/* Context menu indicator */
.action-item:hover .context-menu-indicator {
    opacity: 1;
}

.context-menu-indicator {
    position: absolute;
    right: 2px;
    top: 2px;
    opacity: 0;
    transition: opacity 0.1s ease;
}
"#;
