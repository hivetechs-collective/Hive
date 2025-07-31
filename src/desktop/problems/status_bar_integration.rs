//! Status Bar Integration for Problems Panel
//!
//! Displays error/warning counts and build status in the status bar
//! with click actions to open the problems panel.

use super::problems_panel::{ProblemSeverity, ProblemsState};
use super::build_integration::{BuildStats, BuildTool};
use dioxus::prelude::*;
use std::collections::HashMap;
use tracing::debug;

/// Status bar problems component
#[component]
pub fn ProblemsStatusBar(
    problems_state: Signal<ProblemsState>,
    build_stats: Signal<BuildStats>,
    active_builds: Signal<Vec<BuildTool>>,
    on_click: EventHandler<()>,
) -> Element {
    let problems = problems_state.read();
    let stats = build_stats.read();
    let builds = active_builds.read();
    
    let (total_errors, total_warnings, _, _) = problems.get_counts();
    let has_active_builds = !builds.is_empty();
    
    // Determine overall status
    let (status_icon, status_color, status_text) = if has_active_builds {
        ("🔄", "#FFD700", "Building...")
    } else if total_errors > 0 {
        ("❌", "#F48771", format!("{} error{}", total_errors, if total_errors == 1 { "" } else { "s" }))
    } else if total_warnings > 0 {
        ("⚠️", "#CCA700", format!("{} warning{}", total_warnings, if total_warnings == 1 { "" } else { "s" }))
    } else {
        ("✅", "#4EC9B0", "No problems".to_string())
    };

    rsx! {
        div {
            class: "status-bar-problems",
            onclick: move |_| on_click.call(()),
            title: create_tooltip_text(&problems, &builds),
            
            span {
                class: "status-icon",
                style: "color: {status_color};",
                "{status_icon}"
            }
            
            span {
                class: "status-text",
                "{status_text}"
            }
            
            if has_active_builds {
                span {
                    class: "build-progress",
                    BuildProgressIndicator { 
                        active_builds: builds.clone() 
                    }
                }
            }
            
            if total_errors > 0 || total_warnings > 0 {
                span {
                    class: "problem-counts",
                    ProblemsBreakdown { 
                        errors: total_errors,
                        warnings: total_warnings,
                    }
                }
            }
        }
    }
}

/// Build progress indicator component
#[component]
fn BuildProgressIndicator(active_builds: Vec<BuildTool>) -> Element {
    rsx! {
        div {
            class: "build-progress-indicator",
            
            for tool in active_builds {
                span {
                    class: "build-tool-indicator",
                    key: "{tool:?}",
                    title: "Building with {tool.display_name()}",
                    "{tool.icon()}"
                }
            }
            
            span {
                class: "spinner",
                "⟳"
            }
        }
    }
}

/// Problems breakdown component
#[component]
fn ProblemsBreakdown(errors: u32, warnings: u32) -> Element {
    rsx! {
        div {
            class: "problems-breakdown",
            
            if errors > 0 {
                span {
                    class: "error-count",
                    title: format!("{} error{}", errors, if errors == 1 { "" } else { "s" }),
                    span { class: "codicon codicon-error" }
                    "{errors}"
                }
            }
            
            if warnings > 0 {
                span {
                    class: "warning-count",
                    title: format!("{} warning{}", warnings, if warnings == 1 { "" } else { "s" }),
                    span { class: "codicon codicon-warning" }
                    "{warnings}"
                }
            }
        }
    }
}

/// Enhanced status bar with detailed information
#[component]
pub fn DetailedProblemsStatusBar(
    problems_state: Signal<ProblemsState>,
    build_stats: Signal<BuildStats>,
    active_builds: Signal<Vec<BuildTool>>, 
    show_details: Signal<bool>,
    on_toggle_details: EventHandler<()>,
    on_open_problems: EventHandler<()>,
) -> Element {
    let problems = problems_state.read();
    let stats = build_stats.read();
    let builds = active_builds.read();
    let details_visible = show_details.read();
    
    rsx! {
        div {
            class: "detailed-status-bar-problems",
            
            // Main status indicator
            div {
                class: "main-status",
                onclick: move |_| on_open_problems.call(()),
                ProblemsStatusBar {
                    problems_state,
                    build_stats,
                    active_builds,
                    on_click: move |_| on_open_problems.call(())
                }
            }
            
            // Toggle details button
            button {
                class: "details-toggle",
                onclick: move |_| on_toggle_details.call(()),
                title: if *details_visible { "Hide details" } else { "Show details" },
                span { 
                    class: if *details_visible { "codicon codicon-chevron-up" } else { "codicon codicon-chevron-down" } 
                }
            }
            
            // Detailed breakdown (shown when expanded)
            if *details_visible {
                div {
                    class: "problems-details",
                    ProblemsDetailBreakdown {
                        problems_state,
                        build_stats,
                        active_builds,
                    }
                }
            }
        }
    }
}

/// Detailed problems breakdown
#[component]
fn ProblemsDetailBreakdown(
    problems_state: Signal<ProblemsState>,
    build_stats: Signal<BuildStats>,
    active_builds: Signal<Vec<BuildTool>>,
) -> Element {
    let problems = problems_state.read();
    let stats = build_stats.read();
    
    // Group problems by source
    let mut source_counts: HashMap<String, (u32, u32)> = HashMap::new();
    
    for (source, source_problems) in &problems.problems {
        let mut errors = 0;
        let mut warnings = 0;
        
        for problem in source_problems {
            match problem.severity {
                ProblemSeverity::Error => errors += 1,
                ProblemSeverity::Warning => warnings += 1,
                _ => {}
            }
        }
        
        if errors > 0 || warnings > 0 {
            source_counts.insert(source.display_name().to_string(), (errors, warnings));
        }
    }
    
    rsx! {
        div {
            class: "problems-detail-breakdown",
            
            // Build status section
            if !active_builds.read().is_empty() {
                div {
                    class: "build-status-section",
                    h4 { "Active Builds" }
                    for tool in active_builds.read().iter() {
                        div {
                            class: "build-tool-status",
                            key: "{tool:?}",
                            span { class: "tool-icon", "{tool.icon()}" }
                            span { class: "tool-name", "{tool.display_name()}" }
                            span { class: "build-spinner", "⟳" }
                        }
                    }
                }
            }
            
            // Problems by source section
            if !source_counts.is_empty() {
                div {
                    class: "problems-by-source-section",
                    h4 { "Problems by Source" }
                    for (source, (errors, warnings)) in source_counts {
                        div {
                            class: "source-problem-count",
                            key: "{source}",
                            
                            span { class: "source-name", "{source}" }
                            
                            div {
                                class: "source-counts",
                                if errors > 0 {
                                    span {
                                        class: "error-count",
                                        span { class: "codicon codicon-error" }
                                        "{errors}"
                                    }
                                }
                                if warnings > 0 {
                                    span {
                                        class: "warning-count",
                                        span { class: "codicon codicon-warning" }
                                        "{warnings}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Quick actions section
            div {
                class: "quick-actions-section",
                h4 { "Quick Actions" }
                div {
                    class: "quick-actions",
                    
                    button {
                        class: "quick-action-btn",
                        title: "Refresh all problems",
                        span { class: "codicon codicon-refresh" }
                        "Refresh"
                    }
                    
                    button {
                        class: "quick-action-btn",
                        title: "Clear all problems",
                        span { class: "codicon codicon-clear-all" }
                        "Clear"
                    }
                    
                    button {
                        class: "quick-action-btn",
                        title: "Filter errors only",
                        span { class: "codicon codicon-error" }
                        "Errors Only"
                    }
                }
            }
        }
    }
}

/// Create tooltip text for status bar
fn create_tooltip_text(problems: &ProblemsState, active_builds: &[BuildTool]) -> String {
    let (errors, warnings, info, hints) = problems.get_counts();
    
    let mut tooltip = String::new();
    
    if !active_builds.is_empty() {
        tooltip.push_str("Building with: ");
        let tools: Vec<String> = active_builds.iter()
            .map(|t| t.display_name().to_string())
            .collect();
        tooltip.push_str(&tools.join(", "));
        tooltip.push('\n');
    }
    
    if errors > 0 || warnings > 0 || info > 0 || hints > 0 {
        tooltip.push_str("Problems: ");
        let mut parts = Vec::new();
        
        if errors > 0 {
            parts.push(format!("{} error{}", errors, if errors == 1 { "" } else { "s" }));
        }
        if warnings > 0 {
            parts.push(format!("{} warning{}", warnings, if warnings == 1 { "" } else { "s" }));
        }
        if info > 0 {
            parts.push(format!("{} info", info));
        }
        if hints > 0 {
            parts.push(format!("{} hint{}", hints, if hints == 1 { "" } else { "s" }));
        }
        
        tooltip.push_str(&parts.join(", "));
    } else {
        tooltip.push_str("No problems detected");
    }
    
    tooltip.push_str("\n\nClick to open Problems panel");
    tooltip
}

/// CSS styles for status bar components
pub const STATUS_BAR_PROBLEMS_STYLES: &str = r#"
.status-bar-problems {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    cursor: pointer;
    border-radius: 3px;
    transition: background-color 0.1s ease;
    font-size: 12px;
}

.status-bar-problems:hover {
    background: var(--vscode-statusBarItem-hoverBackground, rgba(255, 255, 255, 0.1));
}

.status-icon {
    font-size: 14px;
}

.status-text {
    font-weight: 500;
    color: var(--vscode-statusBar-foreground, #ffffff);
}

.build-progress-indicator {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: 4px;
}

.build-tool-indicator {
    font-size: 10px;
    opacity: 0.8;
}

.spinner {
    animation: spin 1s linear infinite;
    font-size: 12px;
    color: #FFD700;
}

@keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}

.problems-breakdown {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-left: 4px;
}

.error-count {
    display: flex;
    align-items: center;
    gap: 2px;
    color: #F48771;
    font-size: 11px;
    font-weight: 600;
}

.warning-count {
    display: flex;
    align-items: center;
    gap: 2px;
    color: #CCA700;
    font-size: 11px;
    font-weight: 600;
}

.detailed-status-bar-problems {
    position: relative;
    display: flex;
    align-items: center;
    gap: 4px;
}

.main-status {
    display: flex;
    align-items: center;
}

.details-toggle {
    background: none;
    border: none;
    color: var(--vscode-statusBar-foreground, #ffffff);
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 2px;
    opacity: 0.7;
    transition: opacity 0.1s ease, background-color 0.1s ease;
}

.details-toggle:hover {
    opacity: 1;
    background: var(--vscode-statusBarItem-hoverBackground, rgba(255, 255, 255, 0.1));
}

.problems-details {
    position: absolute;
    bottom: 100%;
    left: 0;
    background: var(--vscode-dropdown-background, #2d2d30);
    border: 1px solid var(--vscode-dropdown-border, #454545);
    border-radius: 4px;
    padding: 12px;
    min-width: 280px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 1000;
    margin-bottom: 4px;
}

.problems-detail-breakdown {
    font-size: 12px;
    color: var(--vscode-dropdown-foreground, #cccccc);
}

.problems-detail-breakdown h4 {
    margin: 0 0 8px 0;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--vscode-descriptionForeground, #999999);
}

.build-status-section {
    margin-bottom: 12px;
}

.build-tool-status {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 0;
}

.tool-icon {
    font-size: 12px;
}

.tool-name {
    font-weight: 500;
}

.build-spinner {
    animation: spin 1s linear infinite;
    color: #FFD700;
    font-size: 10px;
}

.problems-by-source-section {
    margin-bottom: 12px;
}

.source-problem-count {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 0;
}

.source-name {
    font-weight: 500;
}

.source-counts {
    display: flex;
    align-items: center;
    gap: 8px;
}

.quick-actions-section h4 {
    margin-bottom: 6px;
}

.quick-actions {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
}

.quick-action-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: none;
    border: 1px solid var(--vscode-button-secondaryBackground, #5a5a5a);
    color: var(--vscode-button-secondaryForeground, #cccccc);
    padding: 4px 8px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 11px;
    transition: all 0.1s ease;
}

.quick-action-btn:hover {
    background: var(--vscode-button-secondaryHoverBackground, #666666);
    border-color: var(--vscode-button-secondaryHoverBackground, #666666);
}

.quick-action-btn .codicon {
    font-size: 10px;
}
"#;