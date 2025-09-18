//! Performance monitoring component for git operations
//!
//! Displays real-time performance statistics and allows users to monitor
//! the effectiveness of performance optimizations

use dioxus::prelude::*;
use std::time::{Duration, Instant};

use super::performance::PerformanceStats;
use super::performance_config_ui::{PerformanceConfigProps, PerformanceConfigUI};

/// Props for the performance monitor component
#[derive(Props, Clone, PartialEq)]
pub struct PerformanceMonitorProps {
    /// Current performance statistics
    pub stats: PerformanceStats,
    /// Whether to show detailed view
    #[props(default = false)]
    pub detailed: bool,
    /// Whether to show inline (compact) view
    #[props(default = false)]
    pub inline: bool,
    /// Callback to open configuration
    #[props(optional)]
    pub on_configure: Option<EventHandler<()>>,
    /// Callback to clear caches
    #[props(optional)]
    pub on_clear_caches: Option<EventHandler<()>>,
}

/// Performance monitor component
#[component]
pub fn PerformanceMonitor(props: PerformanceMonitorProps) -> Element {
    let stats = &props.stats;

    if props.inline {
        // Compact inline view for status bar
        rsx! {
            div {
                style: "display: flex; align-items: center; gap: 8px; font-size: 11px; color: #888;",

                // Cache hit rate indicator
                div {
                    style: format!("color: {};", if stats.cache_hit_rate() > 0.8 { "#4dff4d" } else if stats.cache_hit_rate() > 0.5 { "#e2c08d" } else { "#f44747" }),
                    title: "Cache hit rate",
                    "⚡ {(stats.cache_hit_rate() * 100.0):.0}%"
                }

                // Total operations
                if stats.total_operations > 0 {
                    div {
                        style: "color: #888;",
                        title: "Total operations",
                        "({stats.total_operations})"
                    }
                }

                // Warning for timeouts
                if stats.operations_timed_out > 0 {
                    div {
                        style: "color: #f44747;",
                        title: "{stats.operations_timed_out} operations timed out",
                        "⚠"
                    }
                }
            }
        }
    } else if props.detailed {
        // Detailed monitoring panel
        rsx! {
            div {
                style: "background: #2d2d30; border: 1px solid #464647; border-radius: 6px; padding: 16px;",

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; border-bottom: 1px solid #464647; padding-bottom: 8px;",

                    h3 {
                        style: "color: #cccccc; margin: 0; font-size: 16px; font-weight: 600;",
                        "⚡ Git Performance Monitor"
                    }

                    div {
                        style: "display: flex; gap: 8px;",

                        if let Some(on_clear_caches) = props.on_clear_caches.clone() {
                            button {
                                style: "padding: 4px 8px; background: #d73a49; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 11px;",
                                onclick: move |_| on_clear_caches.call(()),
                                "Clear Cache"
                            }
                        }

                        if let Some(on_configure) = props.on_configure.clone() {
                            button {
                                style: "padding: 4px 8px; background: #0078d4; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 11px;",
                                onclick: move |_| on_configure.call(()),
                                "Configure"
                            }
                        }
                    }
                }

                // Performance metrics grid
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 16px;",

                    // Cache Performance
                    div {
                        style: "background: #1a1a1a; border-radius: 4px; padding: 12px; border-left: 3px solid #0078d4;",

                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                            div {
                                style: "color: #888; font-size: 12px;",
                                "Cache Hit Rate"
                            }
                            div {
                                style: format!("font-weight: 600; font-size: 14px; color: {};",
                                    if stats.cache_hit_rate() > 0.8 { "#4dff4d" }
                                    else if stats.cache_hit_rate() > 0.5 { "#e2c08d" }
                                    else { "#f44747" }),
                                "{(stats.cache_hit_rate() * 100.0):.1}%"
                            }
                        }

                        div {
                            style: "font-size: 11px; color: #666;",
                            "Hits: {stats.cache_hits} | Misses: {stats.cache_misses}"
                        }

                        // Cache hit rate bar
                        div {
                            style: "width: 100%; height: 4px; background: #3c3c3c; border-radius: 2px; margin-top: 8px; overflow: hidden;",
                            div {
                                style: format!("height: 100%; background: {}; width: {}%; transition: width 0.3s ease;",
                                    if stats.cache_hit_rate() > 0.8 { "#4dff4d" }
                                    else if stats.cache_hit_rate() > 0.5 { "#e2c08d" }
                                    else { "#f44747" },
                                    (stats.cache_hit_rate() * 100.0).min(100.0)),
                            }
                        }
                    }

                    // Operation Performance
                    div {
                        style: "background: #1a1a1a; border-radius: 4px; padding: 12px; border-left: 3px solid #28a745;",

                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                            div {
                                style: "color: #888; font-size: 12px;",
                                "Avg Operation Time"
                            }
                            div {
                                style: format!("font-weight: 600; font-size: 14px; color: {};",
                                    if stats.average_operation_time_ms < 100.0 { "#4dff4d" }
                                    else if stats.average_operation_time_ms < 500.0 { "#e2c08d" }
                                    else { "#f44747" }),
                                "{stats.average_operation_time_ms:.1}ms"
                            }
                        }

                        div {
                            style: "font-size: 11px; color: #666;",
                            "Total: {stats.total_operations} operations"
                        }
                    }

                    // Background Tasks
                    div {
                        style: "background: #1a1a1a; border-radius: 4px; padding: 12px; border-left: 3px solid #6f42c1;",

                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                            div {
                                style: "color: #888; font-size: 12px;",
                                "Background Tasks"
                            }
                            div {
                                style: "font-weight: 600; font-size: 14px; color: #cccccc;",
                                "{stats.background_tasks_completed}"
                            }
                        }

                        div {
                            style: "font-size: 11px; color: #666;",
                            "Completed tasks"
                        }
                    }

                    // Memory Usage (if available)
                    if stats.memory_usage_mb > 0 {
                        div {
                            style: "background: #1a1a1a; border-radius: 4px; padding: 12px; border-left: 3px solid #fd7e14;",

                            div {
                                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                                div {
                                    style: "color: #888; font-size: 12px;",
                                    "Memory Usage"
                                }
                                div {
                                    style: format!("font-weight: 600; font-size: 14px; color: {};",
                                        if stats.memory_usage_mb < 256 { "#4dff4d" }
                                        else if stats.memory_usage_mb < 512 { "#e2c08d" }
                                        else { "#f44747" }),
                                    "{stats.memory_usage_mb}MB"
                                }
                            }
                        }
                    }
                }

                // Issues and warnings
                if stats.operations_timed_out > 0 || stats.cache_hit_rate() < 0.3 {
                    div {
                        style: "background: #4a1a1a; border: 1px solid #8b0000; border-radius: 4px; padding: 12px;",

                        h4 {
                            style: "color: #f44747; margin: 0 0 8px 0; font-size: 14px; font-weight: 600;",
                            "⚠ Performance Issues"
                        }

                        div {
                            style: "font-size: 13px; color: #cccccc;",

                            if stats.operations_timed_out > 0 {
                                div {
                                    style: "margin-bottom: 4px;",
                                    "• {stats.operations_timed_out} operations timed out - consider increasing timeout values"
                                }
                            }

                            if stats.cache_hit_rate() < 0.3 {
                                div {
                                    style: "margin-bottom: 4px;",
                                    "• Low cache hit rate ({(stats.cache_hit_rate() * 100.0):.1}%) - consider increasing cache TTL"
                                }
                            }
                        }
                    }
                }

                // Performance recommendations
                if stats.total_operations > 100 {
                    div {
                        style: "background: #1a2f1a; border: 1px solid #28a745; border-radius: 4px; padding: 12px;",

                        h4 {
                            style: "color: #4dff4d; margin: 0 0 8px 0; font-size: 14px; font-weight: 600;",
                            "💡 Performance Tips"
                        }

                        div {
                            style: "font-size: 13px; color: #cccccc;",

                            if stats.cache_hit_rate() > 0.8 {
                                div {
                                    style: "margin-bottom: 4px;",
                                    "• Excellent cache performance! Your git operations are well optimized."
                                }
                            }

                            if stats.average_operation_time_ms < 100.0 {
                                div {
                                    style: "margin-bottom: 4px;",
                                    "• Fast operation times indicate good performance optimization."
                                }
                            }

                            if stats.background_tasks_completed > 20 {
                                div {
                                    style: "margin-bottom: 4px;",
                                    "• Background processing is actively improving your experience."
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Standard monitoring view
        rsx! {
            div {
                style: "background: #2d2d30; border: 1px solid #464647; border-radius: 4px; padding: 12px;",

                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",

                    div {
                        style: "color: #cccccc; font-size: 14px; font-weight: 600;",
                        "⚡ Git Performance"
                    }

                    if let Some(on_configure) = props.on_configure.clone() {
                        button {
                            style: "padding: 4px 8px; background: #0078d4; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 11px;",
                            onclick: move |_| on_configure.call(()),
                            "⚙"
                        }
                    }
                }

                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 12px; font-size: 12px;",

                    div {
                        div {
                            style: "color: #888; margin-bottom: 2px;",
                            "Hit Rate"
                        }
                        div {
                            style: format!("color: {}; font-weight: 600;",
                                if stats.cache_hit_rate() > 0.8 { "#4dff4d" }
                                else if stats.cache_hit_rate() > 0.5 { "#e2c08d" }
                                else { "#f44747" }),
                            "{(stats.cache_hit_rate() * 100.0):.0}%"
                        }
                    }

                    div {
                        div {
                            style: "color: #888; margin-bottom: 2px;",
                            "Avg Time"
                        }
                        div {
                            style: format!("color: {}; font-weight: 600;",
                                if stats.average_operation_time_ms < 100.0 { "#4dff4d" }
                                else if stats.average_operation_time_ms < 500.0 { "#e2c08d" }
                                else { "#f44747" }),
                            "{stats.average_operation_time_ms:.0}ms"
                        }
                    }

                    div {
                        div {
                            style: "color: #888; margin-bottom: 2px;",
                            "Operations"
                        }
                        div {
                            style: "color: #cccccc; font-weight: 600;",
                            "{stats.total_operations}"
                        }
                    }

                    if stats.operations_timed_out > 0 {
                        div {
                            div {
                                style: "color: #888; margin-bottom: 2px;",
                                "Timeouts"
                            }
                            div {
                                style: "color: #f44747; font-weight: 600;",
                                "{stats.operations_timed_out}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Component that automatically updates performance statistics at regular intervals
#[component]
pub fn AutoRefreshPerformanceMonitor(props: PerformanceMonitorProps) -> Element {
    let mut last_update = use_signal(|| Instant::now());
    let initial_stats = props.stats.clone();
    let mut auto_stats = use_signal(|| initial_stats.clone());

    // Auto-refresh every 5 seconds
    use_effect(move || {
        let interval = Duration::from_secs(5);
        let stats_for_async = initial_stats.clone();

        spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let now = Instant::now();

                if now.duration_since(*last_update.read()) >= interval {
                    // In a real implementation, you would fetch fresh stats here
                    // For now, we'll just use the provided stats
                    *auto_stats.write() = stats_for_async.clone();
                    *last_update.write() = now;
                }
            }
        });
    });

    rsx! {
        PerformanceMonitor {
            stats: auto_stats.read().clone(),
            detailed: props.detailed,
            inline: props.inline,
            on_configure: props.on_configure,
            on_clear_caches: props.on_clear_caches,
        }
    }
}

/// Real-time performance graph component (simplified)
#[component]
pub fn PerformanceGraph(history: Vec<(Instant, PerformanceStats)>, height: Option<u32>) -> Element {
    let height = height.unwrap_or(60);

    if history.is_empty() {
        return rsx! {
            div {
                style: "display: flex; align-items: center; justify-content: center; height: {height}px; background: #1a1a1a; border-radius: 4px; color: #888; font-size: 12px;",
                "No performance data available"
            }
        };
    }

    // Calculate basic statistics for visualization
    let max_time = history
        .iter()
        .map(|(_, stats)| stats.average_operation_time_ms)
        .fold(0.0, f64::max);

    let max_hit_rate = 100.0;

    rsx! {
        div {
            style: "background: #1a1a1a; border-radius: 4px; padding: 8px; height: {height}px; position: relative; overflow: hidden;",

            // Simple bar chart representation
            div {
                style: "display: flex; align-items: end; height: 100%; gap: 1px;",

                for (i, (_, stats)) in history.iter().enumerate() {
                    div {
                        key: "{i}",
                        style: format!(
                            "flex: 1; background: {}; min-height: 2px; height: {}%; border-radius: 1px; opacity: {};",
                            if stats.cache_hit_rate() > 0.8 { "#4dff4d" }
                            else if stats.cache_hit_rate() > 0.5 { "#e2c08d" }
                            else { "#f44747" },
                            (stats.cache_hit_rate() * 100.0).max(5.0),
                            0.7 + (i as f64 / history.len() as f64) * 0.3
                        ),
                        title: "Hit rate: {(stats.cache_hit_rate() * 100.0):.1}%, Time: {stats.average_operation_time_ms:.1}ms",
                    }
                }
            }

            // Overlay with current values
            div {
                style: "position: absolute; top: 4px; right: 4px; font-size: 10px; color: #888;",
                if let Some((_, latest)) = history.last() {
                    "{(latest.cache_hit_rate() * 100.0):.0}% | {latest.average_operation_time_ms:.0}ms"
                }
            }
        }
    }
}
