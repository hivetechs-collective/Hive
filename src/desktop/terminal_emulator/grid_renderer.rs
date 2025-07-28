//! Grid renderer for converting terminal grid to Dioxus elements

use dioxus::prelude::*;
use alacritty_terminal::{
    term::cell::{Cell, Flags},
    vte::ansi::{Color as TermColor, NamedColor},
    grid::{Dimensions, GridIterator},
    index::{Point, Line, Column},
    Term,
};
use std::sync::Arc;

use super::backend::EventProxy;

/// Terminal grid renderer
pub struct GridRenderer;

impl GridRenderer {
    /// Render terminal grid to Dioxus elements
    pub fn render_grid(terminal: &Term<EventProxy>) -> Element {
        let grid = terminal.grid();
        let cursor = terminal.cursor();
        let cursor_point = cursor.point;
        let cursor_visible = terminal.cursor_style().is_some();

        // Build rows
        let mut rows = Vec::new();
        
        for (line_idx, line) in grid.display_iter().enumerate() {
            let mut cells = Vec::new();
            
            for (col_idx, cell) in line.into_iter().enumerate() {
                let is_cursor = cursor_visible && 
                    line_idx as i32 == cursor_point.line.0 && 
                    col_idx == cursor_point.column.0;
                
                cells.push(render_cell(cell, is_cursor));
            }
            
            rows.push(rsx! {
                div {
                    key: "{line_idx}",
                    class: "terminal-row",
                    style: "display: flex; height: 16px; line-height: 16px;",
                    {cells.into_iter()}
                }
            });
        }

        rsx! {
            div {
                class: "terminal-grid",
                style: "font-family: 'Consolas', 'Monaco', 'Courier New', monospace; font-size: 13px; background: #000000; color: #cccccc; overflow: hidden;",
                {rows.into_iter()}
            }
        }
    }

    /// Convert terminal grid to HTML string (alternative approach)
    pub fn render_to_html(terminal: &Term<EventProxy>) -> String {
        let mut html = String::new();
        html.push_str(r#"<div style="font-family: monospace; background: #000; color: #ccc; white-space: pre; overflow-y: auto; height: 100%;">"#);
        
        let grid = terminal.grid();
        let cursor = terminal.cursor();
        let cursor_point = cursor.point;
        
        // Get the display offset for scrollback
        let display_offset = terminal.grid().display_offset();
        let total_scrollback = terminal.grid().history_size();
        
        // If we're scrolled up, show indicator
        if display_offset > 0 {
            html.push_str(r#"<div style="background: #111; color: #888; padding: 2px 4px; font-size: 11px; position: sticky; top: 0; z-index: 10;">"#);
            html.push_str(&format!("📜 Scrolled up {} lines (total history: {})", display_offset, total_scrollback));
            html.push_str("</div>");
        }
        
        for (line_idx, line) in grid.display_iter().enumerate() {
            html.push_str(r#"<div style="height: 16px; line-height: 16px;">"#);
            
            for (col_idx, cell) in line.into_iter().enumerate() {
                let is_cursor = display_offset == 0 && 
                    line_idx as i32 == cursor_point.line.0 && 
                    col_idx == cursor_point.column.0;
                
                html.push_str(&cell_to_html(cell, is_cursor));
            }
            
            html.push_str("</div>");
        }
        
        html.push_str("</div>");
        html
    }
    
    /// Convert terminal grid to HTML string with scroll offset
    pub fn render_to_html_with_scroll(terminal: &Term<EventProxy>, scroll_offset: i32) -> String {
        // Just use the main method since alacritty handles scrolling internally
        Self::render_to_html(terminal)
    }
}

/// Render a single cell
fn render_cell(cell: &Cell, is_cursor: bool) -> Element {
    let ch = if cell.c == '\0' { ' ' } else { cell.c };
    let (fg_color, bg_color) = get_cell_colors(cell, is_cursor);
    
    let mut style = format!(
        "color: {}; background-color: {}; width: 8px; display: inline-block;",
        fg_color, bg_color
    );
    
    // Add text decorations
    if cell.flags.contains(Flags::BOLD) {
        style.push_str(" font-weight: bold;");
    }
    if cell.flags.contains(Flags::ITALIC) {
        style.push_str(" font-style: italic;");
    }
    if cell.flags.contains(Flags::UNDERLINE) {
        style.push_str(" text-decoration: underline;");
    }
    if cell.flags.contains(Flags::STRIKEOUT) {
        style.push_str(" text-decoration: line-through;");
    }
    
    rsx! {
        span {
            class: "terminal-cell",
            style: "{style}",
            "{ch}"
        }
    }
}

/// Convert cell to HTML
fn cell_to_html(cell: &Cell, is_cursor: bool) -> String {
    let ch = if cell.c == '\0' { ' ' } else { cell.c };
    let (fg_color, bg_color) = get_cell_colors(cell, is_cursor);
    
    let mut style = format!("color: {}; background-color: {};", fg_color, bg_color);
    
    if cell.flags.contains(Flags::BOLD) {
        style.push_str(" font-weight: bold;");
    }
    if cell.flags.contains(Flags::ITALIC) {
        style.push_str(" font-style: italic;");
    }
    if cell.flags.contains(Flags::UNDERLINE) {
        style.push_str(" text-decoration: underline;");
    }
    
    format!(
        r#"<span style="{}">{}</span>"#,
        style,
        html_escape::encode_text(&ch.to_string())
    )
}

/// Get foreground and background colors for a cell
fn get_cell_colors(cell: &Cell, is_cursor: bool) -> (String, String) {
    let fg = color_to_css(&cell.fg);
    let bg = if is_cursor {
        "#cccccc".to_string() // Cursor color
    } else {
        color_to_css(&cell.bg)
    };
    
    (fg, bg)
}

/// Convert terminal color to CSS color
fn color_to_css(color: &TermColor) -> String {
    match color {
        TermColor::Named(named) => {
            match named {
                NamedColor::Black => "#000000",
                NamedColor::Red => "#cd3131",
                NamedColor::Green => "#0dbc79",
                NamedColor::Yellow => "#e5e510",
                NamedColor::Blue => "#2472c8",
                NamedColor::Magenta => "#bc3fbc",
                NamedColor::Cyan => "#11a8cd",
                NamedColor::White => "#e5e5e5",
                NamedColor::BrightBlack => "#666666",
                NamedColor::BrightRed => "#f14c4c",
                NamedColor::BrightGreen => "#23d18b",
                NamedColor::BrightYellow => "#f5f543",
                NamedColor::BrightBlue => "#3b8eea",
                NamedColor::BrightMagenta => "#d670d6",
                NamedColor::BrightCyan => "#29b8db",
                NamedColor::BrightWhite => "#e5e5e5",
                NamedColor::Foreground => "#cccccc",
                NamedColor::Background => "#000000",
                _ => "#cccccc",
            }
        }
        TermColor::Spec(rgb) => {
            format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b)
        }
        TermColor::Indexed(idx) => {
            // Use a simplified 256-color palette
            indexed_color_to_css(*idx)
        }
    }
    .to_string()
}

/// Convert 256-color index to CSS color
fn indexed_color_to_css(idx: u8) -> &'static str {
    // Simplified - in a real implementation, this would be a full 256-color table
    match idx {
        0 => "#000000",
        1 => "#cd3131",
        2 => "#0dbc79",
        3 => "#e5e510",
        4 => "#2472c8",
        5 => "#bc3fbc",
        6 => "#11a8cd",
        7 => "#e5e5e5",
        _ => "#cccccc",
    }
}