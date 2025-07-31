//! Performance configuration UI component
//! 
//! Provides a user interface for configuring git performance optimizations

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use super::performance::{PerformanceConfig, PerformanceStats};

/// UI state for the performance configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceConfigUI {
    pub config: PerformanceConfig,
    pub is_editing: bool,
    pub has_changes: bool,
}

impl Default for PerformanceConfigUI {
    fn default() -> Self {
        Self {
            config: PerformanceConfig::default(),
            is_editing: false,
            has_changes: false,
        }
    }
}

/// Props for the performance configuration component
#[derive(Props, Clone, PartialEq)]
pub struct PerformanceConfigProps {
    /// Current configuration
    pub config: PerformanceConfig,
    /// Current performance statistics
    #[props(optional)]
    pub stats: Option<PerformanceStats>,
    /// Callback when configuration is updated
    pub on_config_update: EventHandler<PerformanceConfig>,
    /// Callback to clear caches
    #[props(optional)]
    pub on_clear_caches: Option<EventHandler<()>>,
    /// Whether to show in modal mode
    #[props(default = false)]
    pub modal_mode: bool,
    /// Close callback for modal mode
    #[props(optional)]
    pub on_close: Option<EventHandler<()>>,
}

/// Performance configuration UI component
#[component]
pub fn PerformanceConfigUI(props: PerformanceConfigProps) -> Element {
    let mut ui_state = use_signal(|| PerformanceConfigUI {
        config: props.config.clone(),
        is_editing: false,
        has_changes: false,
    });

    let current_config = ui_state.read().config.clone();
    let is_editing = ui_state.read().is_editing;
    let has_changes = ui_state.read().has_changes;

    // Helper to update config and mark as changed
    let mut update_config = move |update_fn: Box<dyn Fn(&mut PerformanceConfig)>| {
        ui_state.with_mut(|state| {
            update_fn(&mut state.config);
            state.has_changes = true;
            state.is_editing = true;
        });
    };

    let save_config = {
        let config = current_config.clone();
        move |_| {
            props.on_config_update.call(config.clone());
            ui_state.with_mut(|state| {
                state.has_changes = false;
                state.is_editing = false;
            });
        }
    };

    let reset_config = move |_| {
        ui_state.with_mut(|state| {
            state.config = props.config.clone();
            state.has_changes = false;
            state.is_editing = false;
        });
    };

    let clear_caches = move |_| {
        if let Some(callback) = &props.on_clear_caches {
            callback.call(());
        }
    };

    let container_style = if props.modal_mode {
        "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 1000;"
    } else {
        "width: 100%;"
    };

    let panel_style = if props.modal_mode {
        "background: #2d2d30; border: 1px solid #464647; border-radius: 6px; padding: 20px; max-width: 600px; width: 90%; max-height: 80vh; overflow-y: auto;"
    } else {
        "background: #2d2d30; border: 1px solid #464647; border-radius: 6px; padding: 16px; margin: 8px 0;"
    };

    rsx! {
        div {
            style: "{container_style}",
            
            div {
                style: "{panel_style}",
                
                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; border-bottom: 1px solid #464647; padding-bottom: 12px;",
                    
                    h3 {
                        style: "color: #cccccc; margin: 0; font-size: 18px; font-weight: 600;",
                        "🚀 Git Performance Configuration"
                    }
                    
                    if props.modal_mode {
                        if let Some(on_close) = &props.on_close {
                            button {
                                style: "background: none; border: none; color: #cccccc; font-size: 18px; cursor: pointer; padding: 4px;",
                                onclick: move |_| on_close.call(()),
                                "×"
                            }
                        }
                    }
                }

                // Performance Statistics (if available)
                if let Some(stats) = &props.stats {
                    div {
                        style: "margin-bottom: 20px; padding: 12px; background: #1a1a1a; border-radius: 4px; border-left: 3px solid #0078d4;",
                        
                        h4 {
                            style: "color: #cccccc; margin: 0 0 12px 0; font-size: 14px; font-weight: 600;",
                            "📊 Performance Statistics"
                        }
                        
                        div {
                            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; font-size: 13px;",
                            
                            div {
                                div {
                                    style: "color: #888888;",
                                    "Cache Hit Rate"
                                }
                                div {
                                    style: "color: #4dff4d; font-weight: 600;",
                                    "{(stats.cache_hit_rate() * 100.0):.1}%"
                                }
                            }
                            
                            div {
                                div {
                                    style: "color: #888888;",
                                    "Total Operations"
                                }
                                div {
                                    style: "color: #cccccc;",
                                    "{stats.total_operations}"
                                }
                            }
                            
                            div {
                                div {
                                    style: "color: #888888;",
                                    "Avg Operation Time"
                                }
                                div {
                                    style: "color: #cccccc;",
                                    "{stats.average_operation_time_ms:.1}ms"
                                }
                            }
                            
                            div {
                                div {
                                    style: "color: #888888;",
                                    "Background Tasks"
                                }
                                div {
                                    style: "color: #cccccc;",
                                    "{stats.background_tasks_completed}"
                                }
                            }
                        }
                        
                        if stats.operations_timed_out > 0 {
                            div {
                                style: "margin-top: 8px; color: #f44747; font-size: 12px;",
                                "⚠ {stats.operations_timed_out} operations timed out"
                            }
                        }
                    }
                }

                // Configuration Form
                div {
                    style: "display: grid; gap: 16px;",
                    
                    // General Performance Settings
                    div {
                        style: "border: 1px solid #464647; border-radius: 4px; padding: 16px;",
                        
                        h4 {
                            style: "color: #cccccc; margin: 0 0 12px 0; font-size: 14px; font-weight: 600;",
                            "⚡ General Performance"
                        }
                        
                        div {
                            style: "display: grid; gap: 12px;",
                            
                            // Background Processing
                            label {
                                style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: current_config.background_processing,
                                    onchange: move |e| {
                                        let checked = e.value() == "true";
                                        update_config(Box::new(move |config| config.background_processing = checked));
                                    },
                                }
                                span {
                                    style: "color: #cccccc; font-size: 13px;",
                                    "Enable background processing"
                                }
                            }
                            
                            // Memory Optimization
                            label {
                                style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: current_config.memory_optimization,
                                    onchange: move |e| {
                                        let checked = e.value() == "true";
                                        update_config(Box::new(move |config| config.memory_optimization = checked));
                                    },
                                }
                                span {
                                    style: "color: #cccccc; font-size: 13px;",
                                    "Enable memory optimization"
                                }
                            }
                            
                            // Lazy Loading
                            label {
                                style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: current_config.lazy_loading_enabled,
                                    onchange: move |e| {
                                        let checked = e.value() == "true";
                                        update_config(Box::new(move |config| config.lazy_loading_enabled = checked));
                                    },
                                }
                                span {
                                    style: "color: #cccccc; font-size: 13px;",
                                    "Enable lazy loading for large datasets"
                                }
                            }
                        }
                    }

                    // Caching Settings
                    div {
                        style: "border: 1px solid #464647; border-radius: 4px; padding: 16px;",
                        
                        h4 {
                            style: "color: #cccccc; margin: 0 0 12px 0; font-size: 14px; font-weight: 600;",
                            "💾 Caching Configuration"
                        }
                        
                        div {
                            style: "display: grid; gap: 12px;",
                            
                            // Enable Caching
                            label {
                                style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: current_config.caching_enabled,
                                    onchange: move |e| {
                                        let checked = e.value() == "true";
                                        update_config(Box::new(move |config| config.caching_enabled = checked));
                                    },
                                }
                                span {
                                    style: "color: #cccccc; font-size: 13px;",
                                    "Enable caching of git operations"
                                }
                            }
                            
                            // Cache TTL
                            if current_config.caching_enabled {
                                div {
                                    label {
                                        style: "display: block; color: #cccccc; font-size: 13px; margin-bottom: 4px;",
                                        "Cache expiry time (seconds)"
                                    }
                                    input {
                                        r#type: "number",
                                        value: "{current_config.cache_ttl_seconds}",
                                        min: "30",
                                        max: "3600",
                                        step: "30",
                                        style: "width: 100%; padding: 6px 8px; background: #3c3c3c; border: 1px solid #464647; border-radius: 3px; color: #cccccc; font-size: 13px;",
                                        onchange: move |e| {
                                            if let Ok(value) = e.value().parse::<u64>() {
                                                update_config(Box::new(move |config| config.cache_ttl_seconds = value));
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }

                    // Concurrency Settings
                    div {
                        style: "border: 1px solid #464647; border-radius: 4px; padding: 16px;",
                        
                        h4 {
                            style: "color: #cccccc; margin: 0 0 12px 0; font-size: 14px; font-weight: 600;",
                            "⚡ Concurrency & Batching"
                        }
                        
                        div {
                            style: "display: grid; gap: 12px;",
                            
                            // Max Concurrent Operations
                            div {
                                label {
                                    style: "display: block; color: #cccccc; font-size: 13px; margin-bottom: 4px;",
                                    "Max concurrent operations"
                                }
                                input {
                                    r#type: "number",
                                    value: "{current_config.max_concurrent_operations}",
                                    min: "1",
                                    max: "32",
                                    style: "width: 100%; padding: 6px 8px; background: #3c3c3c; border: 1px solid #464647; border-radius: 3px; color: #cccccc; font-size: 13px;",
                                    onchange: move |e| {
                                        if let Ok(value) = e.value().parse::<usize>() {
                                            update_config(Box::new(move |config| config.max_concurrent_operations = value));
                                        }
                                    },
                                }
                            }
                            
                            // Max Batch Size
                            div {
                                label {
                                    style: "display: block; color: #cccccc; font-size: 13px; margin-bottom: 4px;",
                                    "Max batch size"
                                }
                                input {
                                    r#type: "number",
                                    value: "{current_config.max_batch_size}",
                                    min: "10",
                                    max: "1000",
                                    step: "10",
                                    style: "width: 100%; padding: 6px 8px; background: #3c3c3c; border: 1px solid #464647; border-radius: 3px; color: #cccccc; font-size: 13px;",
                                    onchange: move |e| {
                                        if let Ok(value) = e.value().parse::<usize>() {
                                            update_config(Box::new(move |config| config.max_batch_size = value));
                                        }
                                    },
                                }
                            }
                            
                            // Page Size
                            div {
                                label {
                                    style: "display: block; color: #cccccc; font-size: 13px; margin-bottom: 4px;",
                                    "Page size for pagination"
                                }
                                input {
                                    r#type: "number",
                                    value: "{current_config.page_size}",
                                    min: "10",
                                    max: "200",
                                    step: "10",
                                    style: "width: 100%; padding: 6px 8px; background: #3c3c3c; border: 1px solid #464647; border-radius: 3px; color: #cccccc; font-size: 13px;",
                                    onchange: move |e| {
                                        if let Ok(value) = e.value().parse::<usize>() {
                                            update_config(Box::new(move |config| config.page_size = value));
                                        }
                                    },
                                }
                            }
                        }
                    }
                    
                    // Timeout Settings
                    div {
                        style: "border: 1px solid #464647; border-radius: 4px; padding: 16px;",
                        
                        h4 {
                            style: "color: #cccccc; margin: 0 0 12px 0; font-size: 14px; font-weight: 600;",
                            "⏱️ Timeout Configuration"
                        }
                        
                        div {
                            label {
                                style: "display: block; color: #cccccc; font-size: 13px; margin-bottom: 4px;",
                                "Operation timeout (milliseconds)"
                            }
                            input {
                                r#type: "number",
                                value: "{current_config.operation_timeout_ms}",
                                min: "5000",
                                max: "300000",
                                step: "5000",
                                style: "width: 100%; padding: 6px 8px; background: #3c3c3c; border: 1px solid #464647; border-radius: 3px; color: #cccccc; font-size: 13px;",
                                onchange: move |e| {
                                    if let Ok(value) = e.value().parse::<u64>() {
                                        update_config(Box::new(move |config| config.operation_timeout_ms = value));
                                    }
                                },
                            }
                        }
                    }
                }

                // Action Buttons
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-top: 20px; padding-top: 16px; border-top: 1px solid #464647;",
                    
                    // Clear Caches Button
                    if let Some(_) = &props.on_clear_caches {
                        button {
                            style: "padding: 8px 16px; background: #d73a49; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 13px;",
                            onclick: clear_caches,
                            "🗑️ Clear Caches"
                        }
                    }
                    
                    div {
                        style: "display: flex; gap: 8px;",
                        
                        // Reset Button
                        if has_changes {
                            button {
                                style: "padding: 8px 16px; background: #6b737c; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 13px;",
                                onclick: reset_config,
                                "Reset"
                            }
                        }
                        
                        // Save Button
                        button {
                            style: if has_changes {
                                "padding: 8px 16px; background: #0078d4; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 13px;"
                            } else {
                                "padding: 8px 16px; background: #3c3c3c; color: #888888; border: none; border-radius: 3px; cursor: not-allowed; font-size: 13px;"
                            },
                            disabled: !has_changes,
                            onclick: save_config,
                            if has_changes { "💾 Save Changes" } else { "✓ No Changes" }
                        }
                    }
                }
            }
        }
    }
}

/// Preset configurations for common use cases
pub struct PerformancePresets;

impl PerformancePresets {
    /// Configuration optimized for small projects (< 1000 files)
    pub fn small_project() -> PerformanceConfig {
        PerformanceConfig {
            background_processing: false,
            caching_enabled: true,
            cache_ttl_seconds: 120, // 2 minutes
            max_concurrent_operations: 4,
            max_batch_size: 50,
            lazy_loading_enabled: false,
            page_size: 100,
            operation_timeout_ms: 15000, // 15 seconds
            memory_optimization: false,
            max_memory_mb: 128,
        }
    }
    
    /// Configuration optimized for medium projects (1K-10K files)
    pub fn medium_project() -> PerformanceConfig {
        PerformanceConfig {
            background_processing: true,
            caching_enabled: true,
            cache_ttl_seconds: 300, // 5 minutes
            max_concurrent_operations: 6,
            max_batch_size: 100,
            lazy_loading_enabled: true,
            page_size: 50,
            operation_timeout_ms: 30000, // 30 seconds
            memory_optimization: true,
            max_memory_mb: 256,
        }
    }
    
    /// Configuration optimized for large projects (10K+ files)
    pub fn large_project() -> PerformanceConfig {
        PerformanceConfig {
            background_processing: true,
            caching_enabled: true,
            cache_ttl_seconds: 600, // 10 minutes
            max_concurrent_operations: 8,
            max_batch_size: 200,
            lazy_loading_enabled: true,
            page_size: 25,
            operation_timeout_ms: 60000, // 60 seconds
            memory_optimization: true,
            max_memory_mb: 512,
        }
    }
    
    /// High-performance configuration for enterprise use
    pub fn enterprise() -> PerformanceConfig {
        PerformanceConfig {
            background_processing: true,
            caching_enabled: true,
            cache_ttl_seconds: 900, // 15 minutes
            max_concurrent_operations: 12,
            max_batch_size: 500,
            lazy_loading_enabled: true,
            page_size: 20,
            operation_timeout_ms: 120000, // 2 minutes
            memory_optimization: true,
            max_memory_mb: 1024,
        }
    }
}