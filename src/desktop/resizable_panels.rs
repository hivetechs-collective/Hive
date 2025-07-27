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
) -> Element {
    let mut is_dragging = use_signal(|| false);
    let mut drag_start = use_signal(|| 0.0);
    let mut size_start = use_signal(|| 0.0);

    let handle_mouse_down = move |evt: MouseEvent| {
        is_dragging.set(true);
        let pos = match direction {
            ResizeDirection::Horizontal => evt.client_coordinates().x,
            ResizeDirection::Vertical => evt.client_coordinates().y,
        };
        drag_start.set(pos);
        size_start.set(*size.read());
        evt.prevent_default();
    };

    let handle_mouse_move = move |evt: MouseEvent| {
        if *is_dragging.read() {
            let pos = match direction {
                ResizeDirection::Horizontal => evt.client_coordinates().x,
                ResizeDirection::Vertical => evt.client_coordinates().y,
            };
            let delta = pos - *drag_start.read();
            let new_size = (*size_start.read() + delta).clamp(min_size, max_size);
            size.set(new_size);
        }
    };

    let handle_mouse_up = move |_| {
        is_dragging.set(false);
    };

    let divider_style = match direction {
        ResizeDirection::Horizontal => format!(
            "position: absolute; top: 0; bottom: 0; width: 6px; \
            cursor: col-resize; z-index: 999; \
            left: {}px; \
            background: transparent;",
            *size.read() - 3.0
        ),
        ResizeDirection::Vertical => format!(
            "position: absolute; left: 0; right: 0; height: 6px; \
            cursor: row-resize; z-index: 999; \
            top: -3px; \
            background: transparent;",
        ),
    };
    
    let hover_style = if *is_dragging.read() {
        "background: rgba(255, 193, 7, 0.5);"
    } else {
        ""
    };

    rsx! {
        div {
            style: "{divider_style} {hover_style}",
            onmousedown: handle_mouse_down,
            onmousemove: handle_mouse_move,
            onmouseup: handle_mouse_up,
            onmouseleave: handle_mouse_up,
            
            // Visual indicator
            div {
                style: match direction {
                    ResizeDirection::Horizontal => format!(
                        "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); \
                        width: 2px; height: 30px; background: {}; \
                        border-radius: 1px; pointer-events: none;",
                        if *is_dragging.read() { "rgba(255, 193, 7, 0.8)" } else { "rgba(255, 255, 255, 0.2)" }
                    ),
                    ResizeDirection::Vertical => format!(
                        "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); \
                        width: 30px; height: 2px; background: {}; \
                        border-radius: 1px; pointer-events: none;",
                        if *is_dragging.read() { "rgba(255, 193, 7, 0.8)" } else { "rgba(255, 255, 255, 0.2)" }
                    ),
                }
            }
        }
    }
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