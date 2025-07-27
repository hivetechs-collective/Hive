//! Resizable panel system for the Hive Consensus UI
//! 
//! Provides draggable dividers to resize panels in the application layout

use dioxus::prelude::*;

/// Represents a draggable divider between panels
#[component]
pub fn ResizableDivider(
    direction: ResizeDirection,
    size: Signal<f64>,
    min_size: f64,
    max_size: f64,
    #[props(default = false)] invert_drag: bool,
) -> Element {
    let mut is_dragging = use_signal(|| false);
    let mut drag_start = use_signal(|| 0.0);
    let mut size_start = use_signal(|| 0.0);

    let handle_mouse_move = move |evt: MouseEvent| {
        if *is_dragging.read() {
            let pos = match direction {
                ResizeDirection::Horizontal => evt.client_coordinates().x,
                ResizeDirection::Vertical => evt.client_coordinates().y,
            };
            let delta = pos - *drag_start.read();
            // Apply inversion based on direction and invert_drag flag
            let adjusted_delta = match direction {
                ResizeDirection::Horizontal => if invert_drag { -delta } else { delta },
                ResizeDirection::Vertical => -delta,   // Always invert for terminal (dragging from top)
            };
            let new_size = (*size_start.read() + adjusted_delta).clamp(min_size, max_size);
            size.set(new_size);
        }
    };

    let handle_mouse_up = move |_evt: MouseEvent| {
        is_dragging.set(false);
    };

    let handle_mouse_down = move |evt: MouseEvent| {
        is_dragging.set(true);
        let pos = match direction {
            ResizeDirection::Horizontal => evt.client_coordinates().x,
            ResizeDirection::Vertical => evt.client_coordinates().y,
        };
        drag_start.set(pos);
        size_start.set(*size.read());
        evt.prevent_default();
        evt.stop_propagation();
    };

    let divider_style = match direction {
        ResizeDirection::Horizontal => format!(
            "position: relative; width: 8px; height: 100%; \
            cursor: col-resize; z-index: 999; \
            background: transparent; flex-shrink: 0; \
            &:hover {{ background: rgba(255, 193, 7, 0.2); }}"
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
        // Full-screen overlay when dragging to capture all mouse events
        if *is_dragging.read() {
            div {
                style: match direction {
                    ResizeDirection::Horizontal => "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 10000; cursor: col-resize;",
                    ResizeDirection::Vertical => "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 10000; cursor: row-resize;",
                },
                onmousemove: handle_mouse_move,
                onmouseup: handle_mouse_up,
            }
        }
        
        // The actual divider
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


/// Direction of resize
#[derive(Clone, Copy, PartialEq, Debug)]
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