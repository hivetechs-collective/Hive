//! Main Code Editor Component
//!
//! Integrates text buffer, cursor management, syntax highlighting, and rendering

use super::{
    buffer::{TextBuffer, TextEdit},
    cursor::{Cursor, Position, Selection},
    highlighting::{get_language_from_extension, HighlightedSpan, SyntaxHighlighter, Theme},
};
use dioxus::events::MouseEvent;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

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
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
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

    // Track click state for double-click detection
    let mut last_click_time = use_signal(|| std::time::Instant::now());
    let mut last_click_pos = use_signal(|| (0, 0));

    // Auto-focus the editor when mounted (Dioxus desktop handles focus automatically)

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

                // Code content area with VS Code-style positioning
                div {
                    class: "code-content",
                    style: "flex: 1; padding: 10px; overflow-x: auto; position: relative; font-family: 'JetBrains Mono', monospace; font-size: 14px; line-height: 21px;",

                    // Lines container with VS Code absolute positioning and click handling
                    div {
                        class: "lines-content",
                        style: format!("position: absolute; top: 0; left: 0; right: 0; height: {}px;",
                               buffer.read().len_lines() * 21),
                        onclick: move |evt: MouseEvent| {
                            tracing::info!("LINES CONTENT CLICKED!");
                            handle_lines_content_click(
                                evt,
                                &mut cursor,
                                &buffer,
                                *scroll_offset.read(),
                                &on_cursor_change,
                                &mut last_click_time,
                                &mut last_click_pos,
                            );
                        },

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
            let selection = cursor.read().primary.clone();

            if !selection.is_empty() {
                // Replace selected text
                let start = selection.start();
                let end = selection.end();
                let edit = TextEdit::Replace {
                    range: start..end,
                    text: ch.to_string(),
                };
                buffer.write().apply_edit(edit);
                cursor.with_mut(|c| {
                    c.primary.active = Position::new(start.line, start.column + 1);
                    c.primary.anchor = c.primary.active;
                });
            } else {
                // Insert at cursor
                let edit = TextEdit::Insert {
                    position: cursor.read().primary.active,
                    text: ch.to_string(),
                };
                buffer.write().apply_edit(edit);
                cursor.with_mut(|c| c.move_right(&buffer.read()));
            }

            *has_changes.write() = true;
            on_change.call(buffer.read().to_string());
            let pos = cursor.read().primary.active;
            on_cursor_change.call((pos.line + 1, pos.column + 1));
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
            let selection = cursor.read().primary.clone();
            if !selection.is_empty() {
                // Delete selected text
                tracing::debug!("Deleting selection");
                let start = selection.start();
                let end = selection.end();
                let edit = TextEdit::Delete { range: start..end };
                buffer.write().apply_edit(edit);
                cursor.with_mut(|c| {
                    c.primary.active = start;
                    c.primary.anchor = start;
                });
                *has_changes.write() = true;
                on_change.call(buffer.read().to_string());
                let pos = cursor.read().primary.active;
                on_cursor_change.call((pos.line + 1, pos.column + 1));
            } else {
                // Delete single character
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
                    on_cursor_change.call((new_pos.line + 1, new_pos.column + 1));
                }
            }
        }
        // Delete key
        (false, Key::Delete) => {
            let selection = cursor.read().primary.clone();
            if !selection.is_empty() {
                // Delete selected text
                tracing::debug!("Deleting selection");
                let start = selection.start();
                let end = selection.end();
                let edit = TextEdit::Delete { range: start..end };
                buffer.write().apply_edit(edit);
                cursor.with_mut(|c| {
                    c.primary.active = start;
                    c.primary.anchor = start;
                });
                *has_changes.write() = true;
                on_change.call(buffer.read().to_string());
                let pos = cursor.read().primary.active;
                on_cursor_change.call((pos.line + 1, pos.column + 1));
            } else {
                // Delete character at cursor
                let pos = cursor.read().primary.active;
                let (num_lines, line_len) = {
                    let buffer_read = buffer.read();
                    let num_lines = buffer_read.len_lines();
                    let line_len = buffer_read
                        .line(pos.line)
                        .map(|l| l.len_chars())
                        .unwrap_or(0);
                    (num_lines, line_len)
                };

                if pos.line < num_lines {
                    if pos.column < line_len || pos.line < num_lines - 1 {
                        let mut end_pos = pos;
                        if pos.column < line_len {
                            end_pos.column += 1;
                        } else {
                            // At end of line, delete newline
                            end_pos.line += 1;
                            end_pos.column = 0;
                        }

                        let edit = TextEdit::Delete {
                            range: pos..end_pos,
                        };
                        buffer.write().apply_edit(edit);
                        *has_changes.write() = true;
                        on_change.call(buffer.read().to_string());
                    }
                }
            }
        }
        _ => {
            // Don't prevent default for unhandled keys
            evt.stop_propagation();
        }
    }
}

/// Handle mouse click events on the lines-content area (direct line positioning)
fn handle_lines_content_click(
    evt: MouseEvent,
    cursor: &mut Signal<Cursor>,
    buffer: &Signal<TextBuffer>,
    scroll_offset: usize,
    on_cursor_change: &EventHandler<(usize, usize)>,
    last_click_time: &mut Signal<Instant>,
    last_click_pos: &mut Signal<(usize, usize)>,
) {
    // When clicking on lines-content, element coordinates are relative to lines-content
    // which has position: absolute; top: 0; left: 0; within the content area
    // No padding adjustment needed since we're directly on the lines container
    let lines_x = evt.element_coordinates().x;
    let lines_y = evt.element_coordinates().y;

    // Get ALL coordinate types from the event for debugging
    let client_x = evt.client_coordinates().x;
    let client_y = evt.client_coordinates().y;
    let page_x = evt.page_coordinates().x;
    let page_y = evt.page_coordinates().y;
    let screen_x = evt.screen_coordinates().x;
    let screen_y = evt.screen_coordinates().y;

    tracing::info!(
        "🎯 CLICK DEBUG - element: ({:.1}, {:.1}), client: ({:.1}, {:.1}), page: ({:.1}, {:.1}), screen: ({:.1}, {:.1})",
        lines_x, lines_y, client_x, client_y, page_x, page_y, screen_x, screen_y
    );

    // Constants
    let line_height = 21.0;
    let char_width = 8.4;

    // Test multiple coordinate systems to see which gives correct results
    let line_from_element = (lines_y / line_height).floor() as usize;
    let line_from_client = (client_y / line_height).floor() as usize;
    let line_from_page = (page_y / line_height).floor() as usize;

    tracing::info!(
        "📊 LINE CALCULATIONS - element→{}, client→{}, page→{} (expecting line based on visual click)",
        line_from_element, line_from_client, line_from_page
    );

    // Let's also see what the current line positions should be
    let buffer_read = buffer.read();
    let total_lines = buffer_read.len_lines();
    tracing::info!(
        "📝 FILE INFO - total_lines: {}, line_height: {}px, expected_heights: [0, 21, 42, 63, 84...]",
        total_lines, line_height
    );

    // Now that lines have pointer-events: none, clicks should go to lines-content
    // and give us the correct Y coordinates for line calculation
    let line_index = if lines_y < 0.0 {
        0
    } else {
        (lines_y / line_height).floor() as usize
    };

    tracing::info!(
        "📍 DIRECT CALC: lines_y={:.1} / line_height={:.1} = {:.2} -> line_index={}",
        lines_y,
        line_height,
        lines_y / line_height,
        line_index
    );

    let column = if lines_x < 0.0 {
        0
    } else {
        let calculated_col = (lines_x / char_width).round() as usize;
        tracing::info!(
            "LINES COL CALC: lines_x={:.2} / char_width={:.2} = {:.2} -> column={}",
            lines_x,
            char_width,
            lines_x / char_width,
            calculated_col
        );
        calculated_col
    };

    tracing::info!(
        "LINES FINAL: line_index={}, column={} (1-based: line={}, col={})",
        line_index,
        column,
        line_index + 1,
        column + 1
    );

    // Get the actual line to clamp column to line length
    let buffer_read = buffer.read();
    if line_index < buffer_read.len_lines() {
        let line_content = if let Some(line) = buffer_read.line(line_index) {
            line.to_string()
        } else {
            String::new()
        };
        let line_len = line_content.len();

        // Check for double-click
        let now = Instant::now();
        let time_since_last = now.duration_since(*last_click_time.read());
        let (last_line, last_col) = *last_click_pos.read();
        let is_double_click = time_since_last < Duration::from_millis(500)
            && last_line == line_index
            && (last_col as i32 - column as i32).abs() <= 2;

        // Update last click info
        *last_click_time.write() = now;
        *last_click_pos.write() = (line_index, column);

        if is_double_click && !line_content.is_empty() {
            // Double-click: select word at position
            let clamped_column = column.min(line_len.saturating_sub(1));
            if let Some((word_start, word_end)) =
                find_word_boundaries(&line_content, clamped_column)
            {
                cursor.with_mut(|c| {
                    c.primary.anchor.line = line_index;
                    c.primary.anchor.column = word_start;
                    c.primary.active.line = line_index;
                    c.primary.active.column = word_end;
                });

                tracing::debug!(
                    "Word selected: line {}, cols {}-{}, word: '{}'",
                    line_index + 1,
                    word_start + 1,
                    word_end + 1,
                    &line_content[word_start..word_end]
                );
            }
        } else {
            // Single click: position cursor precisely
            let final_column = column.min(line_len);
            cursor.with_mut(|c| {
                c.primary.active.line = line_index;
                c.primary.active.column = final_column;
                c.primary.anchor = c.primary.active;
            });

            tracing::info!(
                "LINES CURSOR SET: line {} -> {}, col {} -> {} (line_len={})",
                cursor.read().primary.active.line + 1,
                line_index + 1,
                cursor.read().primary.active.column + 1,
                final_column + 1,
                line_len
            );
        }

        // Emit cursor position change
        let final_pos = cursor.read().primary.active;
        on_cursor_change.call((final_pos.line + 1, final_pos.column + 1));
    }
}

/// Handle mouse click events on the content area
fn handle_content_click(
    evt: MouseEvent,
    cursor: &mut Signal<Cursor>,
    buffer: &Signal<TextBuffer>,
    scroll_offset: usize,
    on_cursor_change: &EventHandler<(usize, usize)>,
    last_click_time: &mut Signal<Instant>,
    last_click_pos: &mut Signal<(usize, usize)>,
) {
    // Pure Rust coordinate calculation for Dioxus desktop
    // Use element coordinates which are relative to the clicked element
    let element_x = evt.element_coordinates().x;
    let element_y = evt.element_coordinates().y;

    tracing::info!(
        "DIOXUS CLICK: element coordinates ({:.2}, {:.2})",
        element_x,
        element_y
    );

    // Constants matching our CSS styling
    let line_height = 21.0; // pixels per line (matches CSS line-height: 21px)
    let char_width = 8.4; // pixels per character (JetBrains Mono 14px)
    let content_padding = 10.0; // content area padding

    // Account for coordinate system:
    // - content area has padding: 10px
    // - lines-content is positioned absolute at top: 0, left: 0 within content
    // - each line is positioned at top: line_idx * 21px within lines-content
    // So element_y is relative to content area, we need to account for padding only
    let text_x = element_x - content_padding;
    let text_y = element_y - content_padding;

    tracing::info!(
        "TEXT COORDS: element({:.2}, {:.2}) -> text({:.2}, {:.2})",
        element_x,
        element_y,
        text_x,
        text_y
    );

    // Calculate line index using the absolute positioning approach
    // Since each line is positioned at top: line_idx * 21px
    let line_index = if text_y < 0.0 {
        0
    } else {
        let calculated_line = (text_y / line_height).floor() as usize;
        tracing::info!(
            "LINE CALC: text_y={:.2} / line_height={:.2} = {:.2} -> line_index={}",
            text_y,
            line_height,
            text_y / line_height,
            calculated_line
        );
        calculated_line
    };

    // Calculate column index
    let column = if text_x < 0.0 {
        0
    } else {
        let calculated_col = (text_x / char_width).round() as usize;
        tracing::info!(
            "COL CALC: text_x={:.2} / char_width={:.2} = {:.2} -> column={}",
            text_x,
            char_width,
            text_x / char_width,
            calculated_col
        );
        calculated_col
    };

    tracing::info!(
        "FINAL POSITION: line_index={}, column={} (1-based: line={}, col={})",
        line_index,
        column,
        line_index + 1,
        column + 1
    );

    // Get the actual line to clamp column to line length
    let buffer_read = buffer.read();
    if line_index < buffer_read.len_lines() {
        let line_content = if let Some(line) = buffer_read.line(line_index) {
            line.to_string()
        } else {
            String::new()
        };
        let line_len = line_content.len();

        // Check for double-click
        let now = Instant::now();
        let time_since_last = now.duration_since(*last_click_time.read());
        let (last_line, last_col) = *last_click_pos.read();
        let is_double_click = time_since_last < Duration::from_millis(500)
            && last_line == line_index
            && (last_col as i32 - column as i32).abs() <= 2;

        // Update last click info
        *last_click_time.write() = now;
        *last_click_pos.write() = (line_index, column);

        if is_double_click && !line_content.is_empty() {
            // Double-click: select word at position
            let clamped_column = column.min(line_len.saturating_sub(1));
            if let Some((word_start, word_end)) =
                find_word_boundaries(&line_content, clamped_column)
            {
                cursor.with_mut(|c| {
                    // Set selection range
                    c.primary.anchor.line = line_index;
                    c.primary.anchor.column = word_start;
                    c.primary.active.line = line_index;
                    c.primary.active.column = word_end;
                });

                tracing::debug!(
                    "Word selected: line {}, cols {}-{}, word: '{}'",
                    line_index + 1,
                    word_start + 1,
                    word_end + 1,
                    &line_content[word_start..word_end]
                );
            }
        } else {
            // Single click: position cursor precisely
            let final_column = column.min(line_len);
            cursor.with_mut(|c| {
                c.primary.active.line = line_index;
                c.primary.active.column = final_column;
                c.primary.anchor = c.primary.active;
            });

            tracing::info!(
                "CURSOR SET: line {} -> {}, col {} -> {} (line_len={})",
                cursor.read().primary.active.line + 1,
                line_index + 1,
                cursor.read().primary.active.column + 1,
                final_column + 1,
                line_len
            );

            // Cursor position set successfully
        }

        // Emit cursor position change
        let final_pos = cursor.read().primary.active;
        on_cursor_change.call((final_pos.line + 1, final_pos.column + 1));
    }
}

/// Find word boundaries at a given position in a line
fn find_word_boundaries(line: &str, position: usize) -> Option<(usize, usize)> {
    if line.is_empty() || position >= line.len() {
        return None;
    }

    let chars: Vec<char> = line.chars().collect();
    let pos = position.min(chars.len().saturating_sub(1));

    // Check if we're on a word character
    if !is_word_char(chars[pos]) {
        // If not on a word char, try to find adjacent word
        // Look left
        if pos > 0 && is_word_char(chars[pos - 1]) {
            return find_word_at_position(&chars, pos - 1);
        }
        // Look right
        if pos + 1 < chars.len() && is_word_char(chars[pos + 1]) {
            return find_word_at_position(&chars, pos + 1);
        }
        return None;
    }

    find_word_at_position(&chars, pos)
}

/// Find the start and end of a word containing the given position
fn find_word_at_position(chars: &[char], position: usize) -> Option<(usize, usize)> {
    if !is_word_char(chars[position]) {
        return None;
    }

    // Find word start
    let mut start = position;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    // Find word end
    let mut end = position;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    Some((start, end))
}

/// Determine if a character is part of a word
fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
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

    // VS Code approach: absolute positioning for precise line placement
    // CRITICAL: Each line must be positioned at exactly line_idx * 21px
    let line_top = line_idx * 21; // Line 0 at 0px, line 1 at 21px, line 2 at 42px, etc.
    let line_style = format!(
        "position: absolute; top: {}px; left: 0; right: 0; height: 21px; line-height: 21px; white-space: pre; font-family: inherit; pointer-events: none;",
        line_top
    );

    // Check if this line has selection
    let selection = cursor.read().primary.clone();
    let has_selection = !selection.is_empty()
        && line_idx >= selection.start().line
        && line_idx <= selection.end().line;

    rsx! {
        div {
            class: "code-line",
            style: "{line_style}",

            // Render selection background if applicable
            if has_selection {
                {
                    let start_col = if line_idx == selection.start().line {
                        selection.start().column
                    } else {
                        0
                    };
                    let end_col = if line_idx == selection.end().line {
                        selection.end().column
                    } else {
                        line_content.len()
                    };

                    rsx! {
                        div {
                            style: format!(
                                "position: absolute; left: {}px; width: {}px; height: 21px; background: rgba(255, 193, 7, 0.3); z-index: 0;",
                                start_col as f32 * 8.4,
                                (end_col - start_col) as f32 * 8.4
                            ),
                        }
                    }
                }
            }

            // Render highlighted spans
            div {
                style: "position: relative; z-index: 1;",
                {render_highlighted_spans(&line_content, &highlights)}
            }

            // Render cursor if on this line
            if cursor.read().primary.active.line == line_idx && selection.is_empty() {
                div {
                    style: format!(
                        "position: absolute; left: {}px; top: 0; width: 2px; height: 21px; background: #FFC107; animation: blink 1s infinite; z-index: 2;",
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
