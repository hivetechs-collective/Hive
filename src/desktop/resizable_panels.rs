//! Resizable panel system for the Hive Consensus UI
//! 
//! Provides draggable dividers to resize panels in the application layout

use dioxus::prelude::*;
use dioxus::document::eval;

/// Represents a draggable divider between panels
#[component]
pub fn ResizableDivider(
    direction: ResizeDirection,
    size: Signal<f64>,
    min_size: f64,
    max_size: f64,
) -> Element {
    let mut is_dragging = use_signal(|| false);
    let mut drag_start = use_signal(|| 0.0);
    let mut size_start = use_signal(|| 0.0);

    // Use global mouse tracking when dragging
    use_effect(move || {
        if *is_dragging.read() {
            let eval_script = r#"
                // Store the handlers so we can remove them
                if (!window.__resizeHandlers) {
                    window.__resizeHandlers = {};
                }
                
                const handleMouseMove = (e) => {
                    window.__resizeMousePos = e;
                };
                
                const handleMouseUp = (e) => {
                    document.removeEventListener('mousemove', handleMouseMove);
                    document.removeEventListener('mouseup', handleMouseUp);
                    document.body.style.cursor = '';
                    window.__resizeMousePos = null;
                    window.__resizeDragEnd = true;
                };
                
                document.addEventListener('mousemove', handleMouseMove);
                document.addEventListener('mouseup', handleMouseUp);
                document.body.style.cursor = window.__resizeCursor || 'row-resize';
            "#;
            
            spawn(async move {
                let _ = dioxus::document::eval(eval_script).await;
                
                // Poll for mouse position updates
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                    
                    let check_script = r#"
                        if (window.__resizeDragEnd) {
                            window.__resizeDragEnd = false;
                            return JSON.stringify({ ended: true });
                        }
                        if (window.__resizeMousePos) {
                            const e = window.__resizeMousePos;
                            return JSON.stringify({ 
                                x: e.clientX, 
                                y: e.clientY,
                                ended: false
                            });
                        }
                        return JSON.stringify({ ended: false });
                    "#;
                    
                    if let Ok(result) = eval(check_script).await {
                        if let Ok(data) = serde_json::from_value::<MousePosData>(result) {
                            if data.ended {
                                is_dragging.set(false);
                                break;
                            }
                            
                            if let (Some(x), Some(y)) = (data.x, data.y) {
                                let pos = match direction {
                                    ResizeDirection::Horizontal => x,
                                    ResizeDirection::Vertical => y,
                                };
                                let delta = pos - *drag_start.read();
                                let new_size = (*size_start.read() + delta).clamp(min_size, max_size);
                                size.set(new_size);
                            }
                        }
                    }
                }
            });
        }
    });

    let handle_mouse_down = move |evt: MouseEvent| {
        is_dragging.set(true);
        let pos = match direction {
            ResizeDirection::Horizontal => evt.client_coordinates().x,
            ResizeDirection::Vertical => evt.client_coordinates().y,
        };
        drag_start.set(pos);
        size_start.set(*size.read());
        evt.prevent_default();
        
        // Set cursor type for dragging
        let cursor = match direction {
            ResizeDirection::Horizontal => "col-resize",
            ResizeDirection::Vertical => "row-resize",
        };
        
        spawn(async move {
            let script = format!("window.__resizeCursor = '{}';", cursor);
            let _ = dioxus::document::eval(&script).await;
        });
    };

    let divider_style = match direction {
        ResizeDirection::Horizontal => format!(
            "position: absolute; top: 0; bottom: 0; width: 8px; \
            cursor: col-resize; z-index: 999; \
            left: {}px; margin-left: -4px; \
            background: transparent; \
            &:hover {{ background: rgba(255, 193, 7, 0.2); }}",
            *size.read()
        ),
        ResizeDirection::Vertical => format!(
            "position: absolute; left: 0; right: 0; height: 8px; \
            cursor: row-resize; z-index: 999; \
            top: -4px; \
            background: transparent; \
            &:hover {{ background: rgba(255, 193, 7, 0.2); }}",
        ),
    };
    
    let active_style = if *is_dragging.read() {
        "background: rgba(255, 193, 7, 0.3) !important;"
    } else {
        ""
    };

    rsx! {
        div {
            style: "{divider_style} {active_style}",
            onmousedown: handle_mouse_down,
            
            // Visual indicator
            div {
                style: match direction {
                    ResizeDirection::Horizontal => format!(
                        "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); \
                        width: 2px; height: 40px; background: {}; \
                        border-radius: 1px; pointer-events: none;",
                        if *is_dragging.read() { "rgba(255, 193, 7, 0.8)" } else { "rgba(255, 255, 255, 0.2)" }
                    ),
                    ResizeDirection::Vertical => format!(
                        "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); \
                        width: 40px; height: 2px; background: {}; \
                        border-radius: 1px; pointer-events: none;",
                        if *is_dragging.read() { "rgba(255, 193, 7, 0.8)" } else { "rgba(255, 255, 255, 0.2)" }
                    ),
                }
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct MousePosData {
    x: Option<f64>,
    y: Option<f64>,
    ended: bool,
}

/// Direction of resize
#[derive(Clone, Copy, PartialEq)]
pub enum ResizeDirection {
    Horizontal,
    Vertical,
}

/// Simple hook for panel sizes without persistence
pub fn use_panel_sizes() -> (Signal<PanelSizes>, impl Fn()) {
    let panel_sizes = use_signal(|| PanelSizes::default());
    
    // Save function (no-op for now)
    let save_sizes = move || {
        // In the future, we can add persistence here
    };
    
    (panel_sizes, save_sizes)
}

#[derive(Clone)]
pub struct PanelSizes {
    pub sidebar_width: f64,
    pub chat_width: f64,
    pub terminal_height: f64,
}

impl Default for PanelSizes {
    fn default() -> Self {
        Self {
            sidebar_width: 250.0,
            chat_width: 400.0,
            terminal_height: 300.0,
        }
    }
}