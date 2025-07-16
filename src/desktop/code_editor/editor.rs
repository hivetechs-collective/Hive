//! Main Code Editor Component
//! 
//! Integrates text buffer, cursor management, syntax highlighting, and rendering

use super::{
    buffer::{TextBuffer, TextEdit},
    cursor::{Cursor, Position, Selection},
    highlighting::{SyntaxHighlighter, Theme, HighlightedSpan, get_language_from_extension},
};
use dioxus::prelude::*;
use dioxus::document::eval;
use dioxus::events::MouseEvent;
use std::collections::HashMap;
use std::path::PathBuf;

/// Main code editor component
#[component]
pub fn CodeEditorComponent(
    file_path: String,
    initial_content: String,
    on_change: EventHandler<String>,
    on_save: EventHandler<(String, String)>,
    on_cursor_change: EventHandler<(usize, usize)>,
) -> Element {
    // Extract language from file extension
    let path = PathBuf::from(&file_path);
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let language_id = get_language_from_extension(extension).unwrap_or("text");
    
    // Initialize state
    let mut buffer = use_signal(|| TextBuffer::from_text(&initial_content));
    let mut cursor = use_signal(|| Cursor::new());
    let mut syntax_highlighter = use_signal(|| SyntaxHighlighter::new());
    let mut has_changes = use_signal(|| false);
    let mut is_focused = use_signal(|| false);
    
    // Track visible lines for rendering
    let mut scroll_offset = use_signal(|| 0usize);
    let visible_lines = 30; // Number of visible lines in editor
    
    // Auto-focus the editor when mounted
    use_effect(move || {
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let eval = eval(r#"
                const editor = document.querySelector('.code-editor');
                if (editor) {
                    editor.focus();
                }
            "#);
            let _ = eval.await;
        });
    });
    
    // Emit cursor position changes
    use_effect(move || {
        let pos = cursor.read().primary.active;
        on_cursor_change.call((pos.line + 1, pos.column + 1));
    });
    
    // Get the appropriate style based on focus state
    let editor_style = get_editor_style(*is_focused.read());
    
    rsx! {
        div {
            class: "code-editor",
            style: "{editor_style}",
            tabindex: "0",
            onfocus: move |_| {
                *is_focused.write() = true;
                tracing::debug!("Editor focused");
            },
            onblur: move |_| {
                *is_focused.write() = false;
                tracing::debug!("Editor blurred");
            },
            onclick: move |evt: MouseEvent| {
                focus_editor();
                handle_editor_click(
                    evt,
                    &mut cursor,
                    &buffer,
                    *scroll_offset.read(),
                    &on_cursor_change,
                );
            },
            onkeydown: move |evt: KeyboardEvent| {
                handle_keyboard_event(
                    evt,
                    &mut buffer,
                    &mut cursor,
                    &mut has_changes,
                    &on_change,
                    &on_save,
                    &file_path,
                    &on_cursor_change,
                );
            },
            
            // Editor content area
            div {
                class: "editor-viewport",
                style: "display: flex; height: 100%;",
                
                // Line numbers gutter
                LineNumbersGutter {
                    scroll_offset: *scroll_offset.read(),
                    visible_lines: visible_lines,
                    total_lines: buffer.read().len_lines(),
                }
                
                // Code content area
                div {
                    class: "code-content",
                    style: "flex: 1; padding: 10px; overflow-x: auto; position: relative;",
                    
                    // Render visible lines with syntax highlighting
                    for line_idx in *scroll_offset.read()..(*scroll_offset.read() + visible_lines).min(buffer.read().len_lines()) {
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
            StatusBar {
                file_path: file_path.clone(),
                language_id: language_id.to_string(),
                cursor_position: cursor.read().primary.active,
                has_changes: *has_changes.read(),
            }
        }
    }
}

/// Get editor style based on focus state
fn get_editor_style(is_focused: bool) -> &'static str {
    if is_focused {
        r#"
            width: 100%;
            height: 100%;
            background: #0E1414;
            color: #FFFFFF;
            font-family: 'JetBrains Mono', 'Consolas', monospace;
            font-size: 14px;
            line-height: 21px;
            overflow: hidden;
            position: relative;
            outline: none;
            border: 2px solid #FFC107;
            transition: border-color 0.2s ease;
        "#
    } else {
        r#"
            width: 100%;
            height: 100%;
            background: #0E1414;
            color: #FFFFFF;
            font-family: 'JetBrains Mono', 'Consolas', monospace;
            font-size: 14px;
            line-height: 21px;
            overflow: hidden;
            position: relative;
            outline: none;
            border: 2px solid transparent;
            transition: border-color 0.2s ease;
        "#
    }
}

/// Focus the editor element
fn focus_editor() {
    let eval = eval(r#"
        const editor = document.querySelector('.code-editor');
        if (editor) {
            editor.focus();
        }
    "#);
    spawn(async move {
        let _ = eval.await;
    });
}

/// Handle keyboard events
fn handle_keyboard_event(
    evt: KeyboardEvent,
    buffer: &mut Signal<TextBuffer>,
    cursor: &mut Signal<Cursor>,
    has_changes: &mut Signal<bool>,
    on_change: &EventHandler<String>,
    on_save: &EventHandler<(String, String)>,
    file_path: &str,
    on_cursor_change: &EventHandler<(usize, usize)>,
) {
    tracing::debug!("Key pressed: {:?}", evt.key());
    
    let key = evt.key();
    let ctrl = evt.modifiers().ctrl();
    
    // Prevent default for all handled keys
    evt.prevent_default();
    
    match (ctrl, key) {
        // Save file
        (true, Key::Character(s)) if s == "s" => {
            tracing::info!("Saving file");
            let content = buffer.read().to_string();
            on_save.call((file_path.to_string(), content));
            *has_changes.write() = false;
        }
        // Basic cursor movement
        (false, Key::ArrowLeft) => {
            cursor.with_mut(|c| c.move_left(&buffer.read()));
            let pos = cursor.read().primary.active;
            on_cursor_change.call((pos.line + 1, pos.column + 1));
        }
        (false, Key::ArrowRight) => {
            cursor.with_mut(|c| c.move_right(&buffer.read()));
            let pos = cursor.read().primary.active;
            on_cursor_change.call((pos.line + 1, pos.column + 1));
        }
        (false, Key::ArrowUp) => {
            cursor.with_mut(|c| c.move_up(&buffer.read()));
            let pos = cursor.read().primary.active;
            on_cursor_change.call((pos.line + 1, pos.column + 1));
        }
        (false, Key::ArrowDown) => {
            cursor.with_mut(|c| c.move_down(&buffer.read()));
            let pos = cursor.read().primary.active;
            on_cursor_change.call((pos.line + 1, pos.column + 1));
        }
        // Text input
        (false, Key::Character(ch)) => {
            tracing::debug!("Inserting character: {}", ch);
            let edit = TextEdit::Insert {
                position: cursor.read().primary.active,
                text: ch.to_string(),
            };
            buffer.write().apply_edit(edit);
            cursor.with_mut(|c| c.move_right(&buffer.read()));
            *has_changes.write() = true;
            on_change.call(buffer.read().to_string());
        }
        // Enter key
        (false, Key::Enter) => {
            tracing::debug!("Inserting newline");
            let edit = TextEdit::Insert {
                position: cursor.read().primary.active,
                text: "\n".to_string(),
            };
            buffer.write().apply_edit(edit);
            cursor.with_mut(|c| {
                c.move_down(&buffer.read());
                c.primary.active.column = 0;
            });
            *has_changes.write() = true;
            on_change.call(buffer.read().to_string());
        }
        // Backspace
        (false, Key::Backspace) => {
            let pos = cursor.read().primary.active;
            if pos.column > 0 || pos.line > 0 {
                tracing::debug!("Deleting character");
                cursor.with_mut(|c| c.move_left(&buffer.read()));
                let new_pos = cursor.read().primary.active;
                let edit = TextEdit::Delete {
                    range: new_pos..pos,
                };
                buffer.write().apply_edit(edit);
                *has_changes.write() = true;
                on_change.call(buffer.read().to_string());
            }
        }
        _ => {
            // Don't prevent default for unhandled keys
            evt.stop_propagation();
        }
    }
}

/// Handle mouse click events to position cursor
fn handle_editor_click(
    evt: MouseEvent,
    cursor: &mut Signal<Cursor>,
    buffer: &Signal<TextBuffer>,
    scroll_offset: usize,
    on_cursor_change: &EventHandler<(usize, usize)>,
) {
    // Get click coordinates relative to the editor
    let client_x = evt.client_coordinates().x;
    let client_y = evt.client_coordinates().y;
    
    // Constants for layout calculation
    let line_height = 21.0; // pixels per line
    let char_width = 8.4; // average character width in pixels
    let gutter_width = 70.0; // line numbers gutter width
    let content_padding = 10.0; // padding in content area
    
    // Calculate which line was clicked (accounting for scroll)
    let relative_y = client_y - 60.0; // Adjust for editor top offset
    let line_index = (relative_y / line_height) as usize + scroll_offset;
    
    // Calculate column position
    let relative_x = client_x - gutter_width - content_padding;
    let column = (relative_x / char_width).max(0.0) as usize;
    
    // Get the actual line to clamp column to line length
    let buffer_read = buffer.read();
    if line_index < buffer_read.len_lines() {
        let line_len = if let Some(line) = buffer_read.line(line_index) {
            line.len_chars()
        } else {
            0
        };
        
        // Update cursor position
        cursor.with_mut(|c| {
            c.primary.active.line = line_index;
            c.primary.active.column = column.min(line_len);
            c.primary.anchor = c.primary.active;
        });
        
        tracing::debug!(
            "Cursor moved to line: {}, col: {}", 
            line_index + 1, 
            column + 1
        );
        
        // Emit cursor position change
        on_cursor_change.call((line_index + 1, column + 1));
    }
}

/// Line numbers gutter component
#[component]
fn LineNumbersGutter(scroll_offset: usize, visible_lines: usize, total_lines: usize) -> Element {
    let gutter_style = r#"
        background: #181E21;
        border-right: 1px solid #2D3336;
        padding: 10px;
        min-width: 50px;
        text-align: right;
        color: #858585;
        user-select: none;
    "#;
    
    rsx! {
        div {
            class: "line-numbers",
            style: "{gutter_style}",
            
            for line_num in (scroll_offset + 1)..=(scroll_offset + visible_lines).min(total_lines) {
                div {
                    style: "height: 21px; line-height: 21px;",
                    "{line_num}"
                }
            }
        }
    }
}

/// Status bar component
#[component]
fn StatusBar(
    file_path: String,
    language_id: String,
    cursor_position: Position,
    has_changes: bool,
) -> Element {
    let status_bar_style = r#"
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 22px;
        background: #181E21;
        border-top: 1px solid #2D3336;
        display: flex;
        align-items: center;
        padding: 0 10px;
        font-size: 12px;
        color: #858585;
    "#;
    
    rsx! {
        div {
            class: "editor-status-bar",
            style: "{status_bar_style}",
            
            // Language indicator
            span {
                style: "margin-right: 20px;",
                "{language_id}"
            }
            
            // Cursor position
            span {
                style: "margin-right: 20px;",
                {format!("Ln {}, Col {}", cursor_position.line + 1, cursor_position.column + 1)}
            }
            
            // Modified indicator
            if has_changes {
                span {
                    style: "color: #FFC107;",
                    "● Modified"
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
            {render_highlighted_spans(&line_content, &highlights)}
            
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

/// Helper function to render highlighted spans
fn render_highlighted_spans(line_content: &str, highlights: &[HighlightedSpan]) -> Element {
    rsx! {
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
    }
}