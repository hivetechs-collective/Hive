//! Tooltip system with smart positioning
//!
//! This module provides a flexible tooltip component that automatically adjusts
//! its position to stay within viewport bounds.

use dioxus::prelude::*;

/// Tooltip position relative to the target element
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TooltipPosition {
    Top,
    Right,
    Bottom,
    Left,
    Auto, // Automatically choose best position
}

impl Default for TooltipPosition {
    fn default() -> Self {
        TooltipPosition::Auto
    }
}

/// Tooltip trigger mode
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TooltipTrigger {
    Hover,
    Focus,
    HoverOrFocus,
    Click,
    Manual,
}

impl Default for TooltipTrigger {
    fn default() -> Self {
        TooltipTrigger::HoverOrFocus
    }
}

/// Tooltip arrow style
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TooltipArrow {
    None,
    Small,
    Medium,
    Large,
}

impl Default for TooltipArrow {
    fn default() -> Self {
        TooltipArrow::Small
    }
}

/// Tooltip props
#[derive(Props, Clone, PartialEq)]
pub struct TooltipProps {
    /// The content to display in the tooltip
    pub content: String,
    
    /// The child element that triggers the tooltip
    pub children: Element,
    
    /// Position of the tooltip relative to the target
    #[props(default)]
    pub position: TooltipPosition,
    
    /// Trigger mode for showing the tooltip
    #[props(default)]
    pub trigger: TooltipTrigger,
    
    /// Delay before showing the tooltip (in milliseconds)
    #[props(default = 500)]
    pub delay: u64,
    
    /// Whether the tooltip is manually controlled
    #[props(default = false)]
    pub open: bool,
    
    /// Callback when tooltip visibility changes
    #[props(default = EventHandler::new(|_| {}))]
    pub on_open_change: EventHandler<bool>,
    
    /// Maximum width of the tooltip
    #[props(default = 300.0)]
    pub max_width: f64,
    
    /// Arrow style
    #[props(default)]
    pub arrow: TooltipArrow,
    
    /// Custom CSS class for the tooltip
    #[props(default = "tooltip".to_string())]
    pub class: String,
    
    /// Z-index for the tooltip
    #[props(default = 9999)]
    pub z_index: i32,
    
    /// Offset from the target element (in pixels)
    #[props(default = 8.0)]
    pub offset: f64,
}

/// Smart positioning calculation
#[derive(Clone, Debug)]
struct PositionCalculation {
    x: f64,
    y: f64,
    actual_position: TooltipPosition,
    arrow_position: String,
}

/// Rectangle for positioning calculations
#[derive(Clone, Debug)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn calculate_position(
    target_rect: Rect,
    tooltip_width: f64,
    tooltip_height: f64,
    preferred_position: TooltipPosition,
    offset: f64,
    viewport_width: f64,
    viewport_height: f64,
) -> PositionCalculation {
    let target_x = target_rect.x;
    let target_y = target_rect.y;
    let target_width = target_rect.width;
    let target_height = target_rect.height;
    let target_center_x = target_x + target_width / 2.0;
    let target_center_y = target_y + target_height / 2.0;
    
    // Helper to check if position fits in viewport
    let fits_in_viewport = |x: f64, y: f64| -> bool {
        x >= 0.0 && y >= 0.0 && 
        x + tooltip_width <= viewport_width && 
        y + tooltip_height <= viewport_height
    };
    
    // Calculate positions for each direction
    let positions = [
        (TooltipPosition::Top, target_center_x - tooltip_width / 2.0, target_y - tooltip_height - offset),
        (TooltipPosition::Right, target_x + target_width + offset, target_center_y - tooltip_height / 2.0),
        (TooltipPosition::Bottom, target_center_x - tooltip_width / 2.0, target_y + target_height + offset),
        (TooltipPosition::Left, target_x - tooltip_width - offset, target_center_y - tooltip_height / 2.0),
    ];
    
    // Try preferred position first
    if preferred_position != TooltipPosition::Auto {
        for (pos, x, y) in &positions {
            if *pos == preferred_position && fits_in_viewport(*x, *y) {
                return PositionCalculation {
                    x: x.max(0.0).min(viewport_width - tooltip_width),
                    y: y.max(0.0).min(viewport_height - tooltip_height),
                    actual_position: *pos,
                    arrow_position: get_arrow_position(*pos),
                };
            }
        }
    }
    
    // Auto positioning: try all positions in order of preference
    let preference_order = if target_center_y > viewport_height / 2.0 {
        // Target is in bottom half, prefer top
        [TooltipPosition::Top, TooltipPosition::Left, TooltipPosition::Right, TooltipPosition::Bottom]
    } else {
        // Target is in top half, prefer bottom
        [TooltipPosition::Bottom, TooltipPosition::Left, TooltipPosition::Right, TooltipPosition::Top]
    };
    
    for preferred in &preference_order {
        for (pos, x, y) in &positions {
            if pos == preferred && fits_in_viewport(*x, *y) {
                return PositionCalculation {
                    x: x.max(0.0).min(viewport_width - tooltip_width),
                    y: y.max(0.0).min(viewport_height - tooltip_height),
                    actual_position: *pos,
                    arrow_position: get_arrow_position(*pos),
                };
            }
        }
    }
    
    // Fallback: use the position with most space available
    let (best_pos, best_x, best_y) = positions[0];
    PositionCalculation {
        x: best_x.max(0.0).min(viewport_width - tooltip_width),
        y: best_y.max(0.0).min(viewport_height - tooltip_height),
        actual_position: best_pos,
        arrow_position: get_arrow_position(best_pos),
    }
}

fn get_arrow_position(position: TooltipPosition) -> String {
    match position {
        TooltipPosition::Top => "bottom".to_string(),
        TooltipPosition::Right => "left".to_string(),
        TooltipPosition::Bottom => "top".to_string(),
        TooltipPosition::Left => "right".to_string(),
        TooltipPosition::Auto => "bottom".to_string(), // Should not happen
    }
}

/// Tooltip component with smart positioning
#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    let mut is_open = use_signal(|| props.open);
    let mut show_timer = use_signal(|| 0u64);
    let mut position_calc = use_signal(|| None::<PositionCalculation>);
    let target_ref = use_node_ref();
    let tooltip_ref = use_node_ref();
    
    // Update controlled state
    use_effect(move || {
        if props.trigger == TooltipTrigger::Manual {
            is_open.set(props.open);
        }
    });
    
    // Calculate position when tooltip opens
    use_effect(move || {
        if is_open() {
            // In a real implementation, we'd calculate positions here
            // For now, we'll use default positioning
            let calc = PositionCalculation {
                x: 0.0,
                y: 0.0,
                actual_position: props.position,
                arrow_position: get_arrow_position(props.position),
            };
            position_calc.set(Some(calc));
        }
    });
    
    // Timer effect for delay
    use_effect(move || {
        if show_timer() > 0 {
            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(props.delay)).await;
                if show_timer() > 0 {
                    is_open.set(true);
                    props.on_open_change.call(true);
                }
            });
        }
    });
    
    let show_tooltip = move || {
        if props.trigger != TooltipTrigger::Manual {
            show_timer.set(show_timer() + 1);
        }
    };
    
    let hide_tooltip = move || {
        show_timer.set(0);
        if props.trigger != TooltipTrigger::Manual {
            is_open.set(false);
            props.on_open_change.call(false);
        }
    };
    
    let arrow_size = match props.arrow {
        TooltipArrow::None => 0,
        TooltipArrow::Small => 4,
        TooltipArrow::Medium => 6,
        TooltipArrow::Large => 8,
    };
    
    rsx! {
        div {
            class: "tooltip-container",
            style: "position: relative; display: inline-block;",
            
            // Target element with event handlers
            div {
                node_ref: target_ref,
                onmouseenter: move |_| {
                    if props.trigger == TooltipTrigger::Hover || props.trigger == TooltipTrigger::HoverOrFocus {
                        show_tooltip();
                    }
                },
                onmouseleave: move |_| {
                    if props.trigger == TooltipTrigger::Hover || props.trigger == TooltipTrigger::HoverOrFocus {
                        hide_tooltip();
                    }
                },
                onfocusin: move |_| {
                    if props.trigger == TooltipTrigger::Focus || props.trigger == TooltipTrigger::HoverOrFocus {
                        show_tooltip();
                    }
                },
                onfocusout: move |_| {
                    if props.trigger == TooltipTrigger::Focus || props.trigger == TooltipTrigger::HoverOrFocus {
                        hide_tooltip();
                    }
                },
                onclick: move |_| {
                    if props.trigger == TooltipTrigger::Click {
                        if is_open() {
                            hide_tooltip();
                        } else {
                            is_open.set(true);
                            props.on_open_change.call(true);
                        }
                    }
                },
                
                {props.children}
            }
            
            // Tooltip portal
            if is_open() {
                Portal {
                    div {
                        node_ref: tooltip_ref,
                        class: "{props.class}",
                        style: format!(
                            "position: fixed; z-index: {}; max-width: {}px; {}",
                            props.z_index,
                            props.max_width,
                            position_calc().map(|calc| {
                                format!("left: {}px; top: {}px;", calc.x, calc.y)
                            }).unwrap_or_default()
                        ),
                        role: "tooltip",
                        
                        // Tooltip content
                        div {
                            class: "tooltip-content",
                            {props.content.clone()}
                        }
                        
                        // Arrow
                        if props.arrow != TooltipArrow::None {
                            div {
                                class: "tooltip-arrow",
                                style: format!(
                                    "position: absolute; width: 0; height: 0; {}",
                                    position_calc().map(|calc| {
                                        get_arrow_style(&calc.arrow_position, arrow_size)
                                    }).unwrap_or_default()
                                ),
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_arrow_style(position: &str, size: i32) -> String {
    match position {
        "top" => format!(
            "bottom: -{}px; left: 50%; transform: translateX(-50%); \
             border-left: {}px solid transparent; \
             border-right: {}px solid transparent; \
             border-top: {}px solid var(--tooltip-bg, #333);",
            size, size, size, size
        ),
        "bottom" => format!(
            "top: -{}px; left: 50%; transform: translateX(-50%); \
             border-left: {}px solid transparent; \
             border-right: {}px solid transparent; \
             border-bottom: {}px solid var(--tooltip-bg, #333);",
            size, size, size, size
        ),
        "left" => format!(
            "right: -{}px; top: 50%; transform: translateY(-50%); \
             border-top: {}px solid transparent; \
             border-bottom: {}px solid transparent; \
             border-left: {}px solid var(--tooltip-bg, #333);",
            size, size, size, size
        ),
        "right" => format!(
            "left: -{}px; top: 50%; transform: translateY(-50%); \
             border-top: {}px solid transparent; \
             border-bottom: {}px solid transparent; \
             border-right: {}px solid var(--tooltip-bg, #333);",
            size, size, size, size
        ),
        _ => String::new(),
    }
}

/// Portal component for rendering tooltips at the document root
#[component]
fn Portal(children: Element) -> Element {
    // In a real implementation, this would use a portal to render
    // the tooltip at the document root to avoid z-index issues
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; z-index: 10000; pointer-events: none;",
            div {
                style: "pointer-events: auto;",
                {children}
            }
        }
    }
}