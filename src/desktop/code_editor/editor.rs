//! Main Code Editor Component
//! 
//! Integrates text buffer, cursor management, syntax highlighting, and rendering

use super::{
    buffer::{TextBuffer, TextEdit},
    cursor::{Cursor, Position, Selection},
    highlighting::{SyntaxHighlighter, Theme, HighlightedSpan, get_language_from_extension},
};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// Main code editor component
#[component]
pub fn CodeEditorComponent(
    file_path: String,
    initial_content: String,
    on_change: EventHandler<String>,
    on_save: EventHandler<(String, String)>,
) -> Element {
    // Extract language from file extension
    let path = PathBuf::from(&file_path);
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let language_id = get_language_from_extension(extension).unwrap_or("text");
    
    // Initialize state
    let mut buffer = use_signal(|| TextBuffer::new_with_content(&initial_content));
    let mut cursor = use_signal(|| Cursor::new());
    let mut syntax_highlighter = use_signal(|| SyntaxHighlighter::new());
    let mut has_changes = use_signal(|| false);
    
    // Track visible lines for rendering
    let mut scroll_offset = use_signal(|| 0usize);
    let visible_lines = 30; // Number of visible lines in editor
    
    let editor_style = r#"
        width: 100%;
        height: 100%;
        background: #0E1414;
        color: #FFFFFF;
        font-family: 'JetBrains Mono', 'Consolas', monospace;
        font-size: 14px;
        line-height: 21px;
        overflow: hidden;
        position: relative;
    "#;
    
    rsx! {
        div {
            class: "code-editor",
            style: "{editor_style}",
            tabindex: "0",
            
            // Keyboard event handling
            onkeydown: move |evt: KeyboardEvent| {
                let key = evt.key();
                let ctrl = evt.modifiers().ctrl();
                let shift = evt.modifiers().shift();
                
                match (ctrl, key) {
                    // Save file
                    (true, Key::Character('s')) => {
                        evt.prevent_default();
                        let content = buffer.read().to_string();
                        on_save.call((file_path.clone(), content));
                        *has_changes.write() = false;
                    }
                    // Basic cursor movement
                    (false, Key::ArrowLeft) => {
                        cursor.write().move_left(&buffer.read());
                    }
                    (false, Key::ArrowRight) => {
                        cursor.write().move_right(&buffer.read());
                    }
                    (false, Key::ArrowUp) => {
                        cursor.write().move_up(&buffer.read());
                    }
                    (false, Key::ArrowDown) => {
                        cursor.write().move_down(&buffer.read());
                    }
                    // Text input
                    (false, Key::Character(ch)) => {
                        let edit = TextEdit::Insert {
                            position: cursor.read().primary.active,
                            text: ch.to_string(),
                        };
                        buffer.write().apply_edit(edit);
                        cursor.write().move_right(&buffer.read());
                        *has_changes.write() = true;
                        on_change.call(buffer.read().to_string());
                    }
                    // Enter key
                    (false, Key::Enter) => {
                        let edit = TextEdit::Insert {
                            position: cursor.read().primary.active,
                            text: "\n".to_string(),
                        };
                        buffer.write().apply_edit(edit);
                        cursor.write().move_down(&buffer.read());
                        cursor.write().primary.active.column = 0;
                        *has_changes.write() = true;
                        on_change.call(buffer.read().to_string());
                    }
                    // Backspace
                    (false, Key::Backspace) => {
                        let pos = cursor.read().primary.active;
                        if pos.column > 0 || pos.line > 0 {
                            cursor.write().move_left(&buffer.read());
                            let new_pos = cursor.read().primary.active;
                            let edit = TextEdit::Delete {
                                range: new_pos..pos,
                            };
                            buffer.write().apply_edit(edit);
                            *has_changes.write() = true;
                            on_change.call(buffer.read().to_string());
                        }
                    }
                    _ => {}
                }
            },
            
            // Editor content area
            div {
                class: "editor-viewport",
                style: "display: flex; height: 100%;",
                
                // Line numbers gutter
                div {
                    class: "line-numbers",
                    style: "background: #181E21; border-right: 1px solid #2D3336; padding: 10px; min-width: 50px; text-align: right; color: #858585; user-select: none;",
                    
                    for line_num in (scroll_offset.read() + 1)..=(scroll_offset.read() + visible_lines).min(buffer.read().len_lines()) {
                        div {
                            style: "height: 21px; line-height: 21px;",
                            "{line_num}"
                        }
                    }
                }
                
                // Code content area
                div {
                    class: "code-content",
                    style: "flex: 1; padding: 10px; overflow-x: auto; position: relative;",
                    
                    // Render visible lines with syntax highlighting
                    for line_idx in *scroll_offset.read()..(scroll_offset.read() + visible_lines).min(buffer.read().len_lines()) {
                        RenderLine {
                            buffer: buffer.clone(),
                            highlighter: syntax_highlighter.clone(),
                            line_idx,
                            language_id: language_id.to_string(),
                            cursor: cursor.clone(),
                        }
                    }
                }
            }
            
            // Status bar
            div {
                class: "editor-status-bar",
                style: "position: absolute; bottom: 0; left: 0; right: 0; height: 22px; background: #181E21; border-top: 1px solid #2D3336; display: flex; align-items: center; padding: 0 10px; font-size: 12px; color: #858585;",
                
                // Language indicator
                span {
                    style: "margin-right: 20px;",
                    "{language_id}"
                }
                
                // Cursor position
                span {
                    style: "margin-right: 20px;",
                    "Ln {}, Col {}",
                    cursor.read().primary.active.line + 1,
                    cursor.read().primary.active.column + 1
                }
                
                // Modified indicator
                if *has_changes.read() {
                    span {
                        style: "color: #FFC107;",
                        "● Modified"
                    }
                }
            }
        }
    }
}

/// Render a single line with syntax highlighting
#[component]
fn RenderLine(
    buffer: Signal<TextBuffer>,
    highlighter: Signal<SyntaxHighlighter>,
    line_idx: usize,
    language_id: String,
    cursor: Signal<Cursor>,
) -> Element {
    let line_content = if let Some(line) = buffer.read().line(line_idx) {
        line.to_string()
    } else {
        String::new()
    };
    
    // Get syntax highlights
    let highlights = highlighter.write().highlight(&line_content, &language_id);
    
    let line_style = "height: 21px; line-height: 21px; white-space: pre; font-family: inherit; position: relative;";
    
    rsx! {
        div {
            class: "code-line",
            style: "{line_style}",
            
            // Render highlighted spans
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
                        "{&line_content[span.start..span.end.min(line_content.len())]}"
                    }
                } else {
                    span {
                        "{&line_content[span.start..span.end.min(line_content.len())]}"
                    }
                }
            }
            
            // Render cursor if on this line
            if cursor.read().primary.active.line == line_idx {
                div {
                    style: format!(
                        "position: absolute; left: {}px; top: 0; width: 2px; height: 21px; background: #FFC107; animation: blink 1s infinite;",
                        cursor.read().primary.active.column as f32 * 8.4
                    ),
                }
            }
        }
    }
}