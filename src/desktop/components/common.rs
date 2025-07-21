// Common UI Components
// Reusable components for the desktop UI

use dioxus::prelude::*;
use crate::desktop::styles::theme::ThemeColors;

/// Button component props
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    /// Button label
    pub label: String,
    
    /// Button variant
    #[props(default = "primary")]
    pub variant: &'static str,
    
    /// Theme colors
    pub theme: ThemeColors,
    
    /// Click handler
    pub on_click: EventHandler<()>,
    
    /// Whether the button is disabled
    #[props(default = false)]
    pub disabled: bool,
    
    /// Additional CSS classes
    #[props(default = "")]
    pub class: &'static str,
}

/// Button component
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let (bg_color, text_color, hover_color) = match props.variant {
        "primary" => (props.theme.primary.clone(), props.theme.background.clone(), darken_color(&props.theme.primary, 0.1)),
        "secondary" => (props.theme.background_secondary.clone(), props.theme.text.clone(), darken_color(&props.theme.background_secondary, 0.1)),
        "success" => (props.theme.success.clone(), props.theme.background.clone(), darken_color(&props.theme.success, 0.1)),
        "danger" => (props.theme.error.clone(), props.theme.background.clone(), darken_color(&props.theme.error, 0.1)),
        "warning" => (props.theme.warning.clone(), props.theme.background.clone(), darken_color(&props.theme.warning, 0.1)),
        _ => (props.theme.primary.clone(), props.theme.background.clone(), darken_color(&props.theme.primary, 0.1)),
    };
    
    let opacity = if props.disabled { "0.5" } else { "1" };
    let cursor = if props.disabled { "not-allowed" } else { "pointer" };
    
    rsx! {
        button {
            class: "button {props.class}",
            style: "
                background: {bg_color};
                color: {text_color};
                border: none;
                padding: 8px 16px;
                border-radius: 6px;
                font-size: 14px;
                font-weight: 500;
                cursor: {cursor};
                opacity: {opacity};
                transition: all 0.2s;
                user-select: none;
            ",
            onclick: move |_| {
                if !props.disabled {
                    props.on_click.call(());
                }
            },
            disabled: props.disabled,
            "{props.label}"
        }
    }
}

/// Toggle component props
#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    /// Toggle label
    pub label: String,
    
    /// Whether the toggle is checked
    pub checked: bool,
    
    /// Theme colors
    pub theme: ThemeColors,
    
    /// Change handler
    pub on_change: EventHandler<bool>,
    
    /// Whether the toggle is disabled
    #[props(default = false)]
    pub disabled: bool,
}

/// Toggle component
#[component]
pub fn Toggle(props: ToggleProps) -> Element {
    let bg_color = if props.checked {
        props.theme.primary.clone()
    } else {
        props.theme.border.clone()
    };
    
    let cursor = if props.disabled { "not-allowed" } else { "pointer" };
    let opacity = if props.disabled { "0.5" } else { "1" };
    let left_position = if props.checked { "22px" } else { "2px" };
    
    rsx! {
        label {
            style: "
                display: flex;
                align-items: center;
                gap: 12px;
                cursor: {cursor};
                opacity: {opacity};
                user-select: none;
            ",
            
            div {
                style: "
                    position: relative;
                    width: 44px;
                    height: 24px;
                    background: {bg_color};
                    border-radius: 12px;
                    transition: background 0.2s;
                    cursor: {cursor};
                ",
                onclick: move |_| {
                    if !props.disabled {
                        props.on_change.call(!props.checked);
                    }
                },
                
                div {
                    style: "
                        position: absolute;
                        top: 2px;
                        left: {left_position};
                        width: 20px;
                        height: 20px;
                        background: {props.theme.background};
                        border-radius: 50%;
                        transition: left 0.2s;
                        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
                    ",
                }
            }
            
            span {
                style: "
                    color: {props.theme.text};
                    font-size: 14px;
                ",
                "{props.label}"
            }
        }
    }
}

/// Select component props
#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    /// Options (value, label)
    pub options: Vec<(&'static str, &'static str)>,
    
    /// Currently selected value
    #[props(default = "")]
    pub value: &'static str,
    
    /// Theme colors
    pub theme: ThemeColors,
    
    /// Change handler
    pub on_change: EventHandler<String>,
    
    /// Whether the select is disabled
    #[props(default = false)]
    pub disabled: bool,
}

/// Select component
#[component]
pub fn Select(props: SelectProps) -> Element {
    rsx! {
        select {
            style: "
                background: {props.theme.background_secondary};
                color: {props.theme.text};
                border: 1px solid {props.theme.border};
                padding: 8px 12px;
                border-radius: 6px;
                font-size: 14px;
                cursor: pointer;
                min-width: 150px;
                appearance: none;
                background-image: url('data:image/svg+xml;charset=US-ASCII,%3Csvg%20width%3D%2714%27%20height%3D%278%27%20viewBox%3D%270%200%2014%208%27%20xmlns%3D%27http%3A//www.w3.org/2000/svg%27%3E%3Cpath%20d%3D%27M1%201l6%206%206-6%27%20stroke%3D%27%23999%27%20stroke-width%3D%272%27%20fill%3D%27none%27%20fill-rule%3D%27evenodd%27/%3E%3C/svg%3E');
                background-repeat: no-repeat;
                background-position: right 8px center;
                padding-right: 30px;
            ",
            value: props.value,
            onchange: move |evt| props.on_change.call(evt.value()),
            disabled: props.disabled,
            
            for (value, label) in props.options {
                option {
                    value: "{value}",
                    selected: props.value == value,
                    "{label}"
                }
            }
        }
    }
}

/// Input component props
#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    /// Input placeholder
    #[props(default = "")]
    pub placeholder: &'static str,
    
    /// Input value
    pub value: String,
    
    /// Theme colors
    pub theme: ThemeColors,
    
    /// Change handler
    pub on_change: EventHandler<String>,
    
    /// Input type
    #[props(default = "text")]
    pub input_type: &'static str,
    
    /// Whether the input is disabled
    #[props(default = false)]
    pub disabled: bool,
}

/// Input component
#[component]
pub fn Input(props: InputProps) -> Element {
    rsx! {
        input {
            r#type: "{props.input_type}",
            style: "
                background: {props.theme.background_secondary};
                color: {props.theme.text};
                border: 1px solid {props.theme.border};
                padding: 8px 12px;
                border-radius: 6px;
                font-size: 14px;
                width: 100%;
                transition: border-color 0.2s;
            ",
            placeholder: "{props.placeholder}",
            value: "{props.value}",
            oninput: move |evt| props.on_change.call(evt.value()),
            disabled: props.disabled,
        }
    }
}

/// Card component props
#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    /// Card title
    #[props(default = "")]
    pub title: &'static str,
    
    /// Card content
    pub content: Element,
    
    /// Theme colors
    pub theme: ThemeColors,
    
    /// Whether the card has a border
    #[props(default = true)]
    pub bordered: bool,
    
    /// Additional CSS classes
    #[props(default = "")]
    pub class: &'static str,
}

/// Card component
#[component]
pub fn Card(props: CardProps) -> Element {
    let border = if props.bordered {
        format!("1px solid {}", props.theme.border)
    } else {
        "none".to_string()
    };
    
    rsx! {
        div {
            class: "card {props.class}",
            style: "
                background: {props.theme.background_secondary};
                border: {border};
                border-radius: 8px;
                padding: 20px;
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
            ",
            
            if !props.title.is_empty() {
                h3 {
                    style: "
                        margin: 0 0 16px;
                        color: {props.theme.text};
                        font-size: 18px;
                        font-weight: 600;
                    ",
                    "{props.title}"
                }
            }
            
            div {
                class: "card-content",
                {props.content}
            }
        }
    }
}

/// Helper function to darken a color
fn darken_color(color: &str, amount: f32) -> String {
    // Simple darkening - in production would use proper color manipulation
    if color.starts_with('#') && color.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&color[1..3], 16),
            u8::from_str_radix(&color[3..5], 16),
            u8::from_str_radix(&color[5..7], 16),
        ) {
            let r = ((r as f32) * (1.0 - amount)) as u8;
            let g = ((g as f32) * (1.0 - amount)) as u8;
            let b = ((b as f32) * (1.0 - amount)) as u8;
            return format!("#{:02x}{:02x}{:02x}", r, g, b);
        }
    }
    color.to_string()
}

/// Loading spinner component props
#[derive(Props, Clone, PartialEq)]
pub struct SpinnerProps {
    /// Spinner size in pixels
    #[props(default = 24)]
    pub size: u32,
    
    /// Theme colors
    pub theme: ThemeColors,
}

/// Loading spinner component
#[component]
pub fn Spinner(props: SpinnerProps) -> Element {
    rsx! {
        div {
            style: "
                width: {props.size}px;
                height: {props.size}px;
                border: 3px solid {props.theme.border};
                border-top-color: {props.theme.primary};
                border-radius: 50%;
                animation: spin 1s linear infinite;
            ",
        }
    }
}

/// Progress bar component props
#[derive(Props, Clone, PartialEq)]
pub struct ProgressBarProps {
    /// Progress value (0-100)
    pub value: f32,
    
    /// Theme colors
    pub theme: ThemeColors,
    
    /// Whether to show percentage
    #[props(default = true)]
    pub show_percentage: bool,
    
    /// Progress bar height
    #[props(default = 8)]
    pub height: u32,
}

/// Progress bar component
#[component]
pub fn ProgressBar(props: ProgressBarProps) -> Element {
    let value = props.value.clamp(0.0, 100.0);
    
    rsx! {
        div {
            style: "width: 100%;",
            
            div {
                style: "
                    height: {props.height}px;
                    background: {props.theme.background};
                    border-radius: {props.height / 2}px;
                    overflow: hidden;
                    position: relative;
                ",
                
                div {
                    style: "
                        height: 100%;
                        background: linear-gradient(90deg, {props.theme.primary}, {props.theme.success});
                        width: {value}%;
                        transition: width 0.3s ease;
                    ",
                }
            }
            
            if props.show_percentage {
                div {
                    style: "
                        text-align: center;
                        margin-top: 4px;
                        font-size: 12px;
                        color: {props.theme.text_secondary};
                    ",
                    {format!("{:.0}%", value)}
                }
            }
        }
    }
}

/// CSS for animations (add to your global styles)
pub const COMMON_STYLES: &str = r#"
@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

.button:active {
    transform: scale(0.98);
}

.card {
    transition: box-shadow 0.2s;
}

.card:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}
"#;