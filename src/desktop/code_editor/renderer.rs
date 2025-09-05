//! Code Editor Renderer
//! 
//! Handles the visual rendering of code with syntax highlighting and decorations

use dioxus::prelude::*;
use super::{
    buffer::TextBuffer,
    cursor::{Cursor, Selection},
    highlighting::{SyntaxHighlighter, Theme, HighlightedSpan},
    git_integration::{GitIntegration, InlineDecoration},
};

/// Render a single line of code with syntax highlighting
#[component]
pub fn CodeLine(
    line_number: usize,
    content: String,
    highlights: Vec<HighlightedSpan>,
    selections: Vec<Selection>,
    git_decorations: Vec<InlineDecoration>,
    theme: Theme,
) -> Element {
    let line_style = r#"
        display: flex;
        align-items: center;
        height: 21px;
        line-height: 21px;
        position: relative;
        white-space: pre;
        font-family: inherit;
    "#;
    
    // Check if line has any git decorations
    let line_git_class = git_decorations.iter()
        .find(|d| d.line == line_number)
        .map(|d| d.decoration_type.css_class());
    
    let line_bg = git_decorations.iter()
        .find(|d| d.line == line_number)
        .map(|d| d.decoration_type.background_color())
        .unwrap_or("transparent");
    
    rsx! {
        div {
            class: "code-line",
            style: "{line_style} background: {line_bg};",
            "data-line": "{line_number}",
            
            // Render highlighted spans
            if highlights.is_empty() {
                span {
                    style: "color: {theme.foreground};",
                    "{content}"
                }
            } else {
                for span in highlights {
                    if let Some(style) = &span.style {
                        span {
                            style: format!(
                                "color: {}; font-weight: {}; font-style: {}; text-decoration: {};",
                                style.color,
                                if style.bold { "bold" } else { "normal" },
                                if style.italic { "italic" } else { "normal" },
                                if style.underline { "underline" } else { "none" }
                            ),
                            "{&content[span.start..span.end]}"
                        }
                    } else {
                        span {
                            style: "color: {theme.foreground};",
                            "{&content[span.start..span.end]}"
                        }
                    }
                }
            }
        }
    }
}

/// Render the cursor
#[component]
pub fn CursorComponent(
    position: super::cursor::Position,
    theme: Theme,
) -> Element {
    let cursor_style = format!(
        r#"
        position: absolute;
        width: 2px;
        height: 21px;
        background: {};
        animation: blink 1s infinite;
        "#,
        theme.cursor
    );
    
    // Calculate pixel position based on character position
    let x = position.column as f32 * 8.4; // Approximate character width
    let y = position.line as f32 * 21.0; // Line height
    
    rsx! {
        div {
            class: "cursor",
            style: "{cursor_style} left: {x}px; top: {y}px;",
        }
    }
}

/// Render selection overlay
#[component]
pub fn SelectionOverlay(
    selection: Selection,
    theme: Theme,
) -> Element {
    if selection.is_empty() {
        return rsx! { div {} };
    }
    
    let selection_style = format!(
        r#"
        position: absolute;
        background: {};
        opacity: 0.3;
        pointer-events: none;
        "#,
        theme.selection
    );
    
    // Calculate selection bounds
    let start = selection.start();
    let end = selection.end();
    
    // For simplicity, assume single-line selection for now
    if start.line == end.line {
        let x = start.column as f32 * 8.4;
        let width = (end.column - start.column) as f32 * 8.4;
        let y = start.line as f32 * 21.0;
        
        rsx! {
            div {
                class: "selection",
                style: "{selection_style} left: {x}px; top: {y}px; width: {width}px; height: 21px;",
            }
        }
    } else {
        // Multi-line selection would require multiple rectangles
        rsx! { div {} }
    }
}

/// Main editor view component
#[component]
pub fn EditorView(
    buffer: Signal<TextBuffer>,
    cursor: Signal<Cursor>,
    highlighter: Signal<SyntaxHighlighter>,
    git_integration: Signal<GitIntegration>,
    show_line_numbers: bool,
    show_minimap: bool,
) -> Element {
    let editor_container_style = r#"
        display: flex;
        height: 100%;
        background: #0E1414;
        font-family: 'JetBrains Mono', 'Consolas', monospace;
        font-size: 14px;
        overflow: hidden;
    "#;
    
    let gutter_style = r#"
        background: #181E21;
        border-right: 1px solid #2D3336;
        color: #858585;
        padding: 0 10px;
        text-align: right;
        user-select: none;
        min-width: 50px;
    "#;
    
    let editor_content_style = r#"
        flex: 1;
        overflow: auto;
        padding: 10px;
        position: relative;
    "#;
    
    let minimap_style = r#"
        width: 120px;
        background: #181E21;
        border-left: 1px solid #2D3336;
        overflow: hidden;
        opacity: 0.8;
    "#;
    
    rsx! {
        div {
            class: "editor-view",
            style: "{editor_container_style}",
            
            // Line number gutter
            if show_line_numbers {
                div {
                    class: "gutter",
                    style: "{gutter_style}",
                    
                    for line_num in 1..=buffer.read().len_lines() {
                        div {
                            class: "line-number",
                            style: "height: 21px; line-height: 21px;",
                            "{line_num}"
                        }
                    }
                }
            }
            
            // Main editor content
            div {
                class: "editor-content",
                style: "{editor_content_style}",
                
                // Render each line
                for line_idx in 0..buffer.read().len_lines() {
                    if let Some(line) = buffer.read().line(line_idx) {
                        CodeLine {
                            line_number: line_idx,
                            content: line.to_string(),
                            highlights: vec![], // TODO: Get from highlighter
                            selections: vec![], // TODO: Get from cursor
                            git_decorations: vec![], // TODO: Get from git integration
                            theme: highlighter.read().get_theme().clone(),
                        }
                    }
                }
                
                // Render cursor
                CursorComponent {
                    position: cursor.read().primary.active,
                    theme: highlighter.read().get_theme().clone(),
                }
                
                // Render selections
                for selection in cursor.read().all_selections() {
                    SelectionOverlay {
                        selection: selection.clone(),
                        theme: highlighter.read().get_theme().clone(),
                    }
                }
            }
            
            // Minimap
            if show_minimap {
                div {
                    class: "minimap",
                    style: "{minimap_style}",
                    
                    // Simplified minimap rendering
                    div {
                        style: "transform: scale(0.1); transform-origin: top left; width: 1200px;",
                        
                        for line_idx in 0..buffer.read().len_lines() {
                            div {
                                style: "height: 2px; background: #444; margin-bottom: 1px;",
                            }
                        }
                    }
                }
            }
        }
    }
}

// CSS for cursor blinking animation
pub const EDITOR_STYLES: &str = r#"
@keyframes blink {
    0%, 50% { opacity: 1; }
    51%, 100% { opacity: 0; }
}

.editor-view {
    position: relative;
}

.code-line {
    transition: background-color 0.2s ease;
}

.code-line:hover {
    background-color: rgba(255, 255, 255, 0.05) !important;
}

.git-addition {
    border-left: 3px solid #4CAF50;
    padding-left: 5px;
}

.git-deletion {
    border-left: 3px solid #F44336;
    padding-left: 5px;
    opacity: 0.7;
}

.git-modification {
    border-left: 3px solid #2196F3;
    padding-left: 5px;
}

.completion-popup {
    animation: fadeIn 0.1s ease-out;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(-5px); }
    to { opacity: 1; transform: translateY(0); }
}
"#;