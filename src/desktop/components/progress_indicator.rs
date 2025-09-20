//! Progress indicator component supporting both determinate and indeterminate modes
//!
//! This module provides various progress indicators including linear bars,
//! circular spinners, and step indicators.

use dioxus::prelude::*;

/// Progress indicator type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProgressType {
    Linear,
    Circular,
    Steps,
    Dots,
}

impl Default for ProgressType {
    fn default() -> Self {
        ProgressType::Linear
    }
}

/// Progress indicator size
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProgressSize {
    Small,
    Medium,
    Large,
    Custom(f64),
}

impl Default for ProgressSize {
    fn default() -> Self {
        ProgressSize::Medium
    }
}

impl ProgressSize {
    fn to_pixels(&self) -> f64 {
        match self {
            ProgressSize::Small => 16.0,
            ProgressSize::Medium => 24.0,
            ProgressSize::Large => 32.0,
            ProgressSize::Custom(size) => *size,
        }
    }
}

/// Progress indicator color scheme
#[derive(Clone, Debug, PartialEq)]
pub enum ProgressColor {
    Primary,
    Success,
    Warning,
    Error,
    Info,
    Custom(String),
}

impl Default for ProgressColor {
    fn default() -> Self {
        ProgressColor::Primary
    }
}

impl ProgressColor {
    fn to_css_var(&self) -> String {
        match self {
            ProgressColor::Primary => "var(--color-primary)".to_string(),
            ProgressColor::Success => "var(--color-success)".to_string(),
            ProgressColor::Warning => "var(--color-warning)".to_string(),
            ProgressColor::Error => "var(--color-error)".to_string(),
            ProgressColor::Info => "var(--color-info)".to_string(),
            ProgressColor::Custom(color) => color.clone(),
        }
    }
}

/// Step information for step progress indicators
#[derive(Clone, Debug, PartialEq)]
pub struct ProgressStep {
    pub label: String,
    pub description: Option<String>,
    pub completed: bool,
    pub current: bool,
    pub error: bool,
}

/// Progress indicator props
#[derive(Props, Clone, PartialEq)]
pub struct ProgressIndicatorProps {
    /// Progress value (0-100 for determinate, None for indeterminate)
    #[props(optional)]
    pub value: Option<f64>,

    /// Type of progress indicator
    #[props(default)]
    pub progress_type: ProgressType,

    /// Size of the indicator
    #[props(default)]
    pub size: ProgressSize,

    /// Color scheme
    #[props(default)]
    pub color: ProgressColor,

    /// Label to display
    #[props(optional)]
    pub label: Option<String>,

    /// Whether to show the percentage value
    #[props(default = true)]
    pub show_value: bool,

    /// Steps for step progress indicator
    #[props(default = vec![])]
    pub steps: Vec<ProgressStep>,

    /// Whether to animate the progress
    #[props(default = true)]
    pub animated: bool,

    /// Buffer value for buffered progress (e.g., video loading)
    #[props(optional)]
    pub buffer_value: Option<f64>,

    /// Custom CSS class
    #[props(default = "progress-indicator".to_string())]
    pub class: String,

    /// Thickness for linear and circular progress (in pixels)
    #[props(default = 4.0)]
    pub thickness: f64,
}

/// Main progress indicator component
#[component]
pub fn ProgressIndicator(props: ProgressIndicatorProps) -> Element {
    match props.progress_type {
        ProgressType::Linear => LinearProgress(props),
        ProgressType::Circular => CircularProgress(props),
        ProgressType::Steps => StepsProgress(props),
        ProgressType::Dots => DotsProgress(props),
    }
}

/// Linear progress bar
#[component]
fn LinearProgress(props: ProgressIndicatorProps) -> Element {
    let is_indeterminate = props.value.is_none();
    let progress = props.value.unwrap_or(0.0).clamp(0.0, 100.0);
    let buffer = props.buffer_value.unwrap_or(100.0).clamp(0.0, 100.0);

    rsx! {
        div {
            class: "{props.class} linear-progress {if is_indeterminate { \"indeterminate\" } else { \"\" }}",

            if let Some(label) = &props.label {
                div { class: "progress-label", "{label}" }
            }

            div {
                class: "progress-track",
                style: format!("height: {}px;", props.thickness),

                // Buffer layer (for buffered progress)
                if props.buffer_value.is_some() && !is_indeterminate {
                    div {
                        class: "progress-buffer",
                        style: format!(
                            "width: {}%; background-color: {};",
                            buffer,
                            props.color.to_css_var()
                        ),
                    }
                }

                // Progress fill
                div {
                    class: format!("progress-fill {}", if props.animated { "animated" } else { "" }),
                    style: format!(
                        "width: {}%; background-color: {};",
                        if is_indeterminate { 100.0 } else { progress },
                        props.color.to_css_var()
                    ),
                }
            }

            if props.show_value && !is_indeterminate {
                div {
                    class: "progress-value",
                    "{progress:.0}%"
                }
            }
        }
    }
}

/// Circular progress indicator
#[component]
fn CircularProgress(props: ProgressIndicatorProps) -> Element {
    let is_indeterminate = props.value.is_none();
    let progress = props.value.unwrap_or(0.0).clamp(0.0, 100.0);
    let size = props.size.to_pixels();
    let center = size / 2.0;
    let radius = (size - props.thickness) / 2.0;
    let circumference = 2.0 * std::f64::consts::PI * radius;
    let stroke_dashoffset = circumference * (1.0 - progress / 100.0);

    rsx! {
        div {
            class: "{props.class} circular-progress {if is_indeterminate { \"indeterminate\" } else { \"\" }}",

            if let Some(label) = &props.label {
                div { class: "progress-label", "{label}" }
            }

            div {
                class: "circular-container",
                style: format!("width: {}px; height: {}px;", size, size),

                svg {
                    width: "{size}",
                    height: "{size}",
                    class: format!("{}", if props.animated { "animated" } else { "" }),

                    // Background circle
                    circle {
                        cx: "{center}",
                        cy: "{center}",
                        r: "{radius}",
                        fill: "none",
                        stroke: "var(--progress-track-bg, #e0e0e0)",
                        stroke_width: "{props.thickness}",
                    }

                    // Progress circle
                    circle {
                        cx: "{center}",
                        cy: "{center}",
                        r: "{radius}",
                        fill: "none",
                        stroke: "{props.color.to_css_var()}",
                        stroke_width: "{props.thickness}",
                        stroke_dasharray: "{circumference}",
                        stroke_dashoffset: if is_indeterminate { "0" } else { "{stroke_dashoffset}" },
                        stroke_linecap: "round",
                        transform: format!("rotate(-90 {} {})", center, center),
                        class: if is_indeterminate { "indeterminate-rotation" } else { "" },
                    }
                }

                if props.show_value && !is_indeterminate {
                    div {
                        class: "circular-value",
                        style: format!(
                            "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);",
                        ),
                        "{progress:.0}%"
                    }
                }
            }
        }
    }
}

/// Steps progress indicator
#[component]
fn StepsProgress(props: ProgressIndicatorProps) -> Element {
    let total_steps = props.steps.len();
    let completed_steps = props.steps.iter().filter(|s| s.completed).count();
    let progress = if total_steps > 0 {
        (completed_steps as f64 / total_steps as f64) * 100.0
    } else {
        0.0
    };

    rsx! {
        div {
            class: "{props.class} steps-progress",

            if let Some(label) = &props.label {
                div { class: "progress-label", "{label}" }
            }

            div {
                class: "steps-container",

                for (index, step) in props.steps.iter().enumerate() {
                    div {
                        class: format!(
                            "step {} {} {}",
                            if step.completed { "completed" } else { "" },
                            if step.current { "current" } else { "" },
                            if step.error { "error" } else { "" }
                        ),

                        // Step indicator
                        div {
                            class: "step-indicator",
                            style: format!("background-color: {};",
                                if step.error {
                                    ProgressColor::Error.to_css_var()
                                } else if step.completed || step.current {
                                    props.color.to_css_var()
                                } else {
                                    "var(--step-inactive-bg, #e0e0e0)".to_string()
                                }
                            ),

                            if step.error {
                                "✕"
                            } else if step.completed {
                                "✓"
                            } else {
                                "{index + 1}"
                            }
                        }

                        // Step content
                        div {
                            class: "step-content",
                            div { class: "step-label", "{step.label}" }
                            if let Some(desc) = &step.description {
                                div { class: "step-description", "{desc}" }
                            }
                        }

                        // Connector line (not for last step)
                        if index < total_steps - 1 {
                            div {
                                class: "step-connector",
                                style: format!(
                                    "background-color: {};",
                                    if step.completed {
                                        props.color.to_css_var()
                                    } else {
                                        "var(--step-connector-bg, #e0e0e0)".to_string()
                                    }
                                ),
                            }
                        }
                    }
                }
            }

            if props.show_value {
                div {
                    class: "progress-value",
                    "{completed_steps} of {total_steps} completed ({progress:.0}%)"
                }
            }
        }
    }
}

/// Dots progress indicator (loading dots)
#[component]
fn DotsProgress(props: ProgressIndicatorProps) -> Element {
    let dot_count = 3;
    let size = props.size.to_pixels() / 3.0;

    rsx! {
        div {
            class: "{props.class} dots-progress",

            if let Some(label) = &props.label {
                div { class: "progress-label", "{label}" }
            }

            div {
                class: "dots-container",

                for i in 0..dot_count {
                    div {
                        class: format!("dot {}", if props.animated { "animated" } else { "" }),
                        style: format!(
                            "width: {}px; height: {}px; background-color: {}; animation-delay: {}ms;",
                            size, size,
                            props.color.to_css_var(),
                            i * 150
                        ),
                    }
                }
            }
        }
    }
}

/// Helper function to create a simple indeterminate spinner
#[component]
pub fn Spinner(
    #[props(default)] size: ProgressSize,
    #[props(default)] color: ProgressColor,
    #[props(optional)] label: Option<String>,
) -> Element {
    rsx! {
        ProgressIndicator {
            value: None,
            progress_type: ProgressType::Circular,
            size: size,
            color: color,
            label: label,
            show_value: false,
        }
    }
}

/// Helper function to create a simple loading dots indicator
#[component]
pub fn LoadingDots(
    #[props(default)] size: ProgressSize,
    #[props(default)] color: ProgressColor,
    #[props(optional)] label: Option<String>,
) -> Element {
    rsx! {
        ProgressIndicator {
            value: None,
            progress_type: ProgressType::Dots,
            size: size,
            color: color,
            label: label,
            show_value: false,
        }
    }
}
