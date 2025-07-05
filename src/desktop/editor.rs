//! Code Editor Component with Syntax Highlighting

use dioxus::prelude::*;
use std::path::PathBuf;

/// Code Editor Component
#[component]
pub fn CodeEditor() -> Element {
    let app_state = use_context::<Signal<crate::desktop::state::AppState>>();
    let state = app_state.read();
    
    // Check if a file is selected
    let selected_file = state.file_explorer.selected_file.clone();
    
    // Load file content when selection changes
    let file_content = use_signal(|| String::new());
    let line_count = use_signal(|| 1usize);
    
    use_effect(move || {
        if let Some(path) = &selected_file {
            if path.is_file() {
                spawn(async move {
                    if let Ok(content) = tokio::fs::read_to_string(path).await {
                        let lines = content.lines().count().max(1);
                        file_content.set(content);
                        line_count.set(lines);
                    }
                });
            }
        }
    });
    
    // Editor styles
    let editor_container_style = "
        display: flex;
        flex-direction: column;
        height: 100%;
        background: #1e1e1e;
        color: #d4d4d4;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    ";
    
    let tabs_container_style = "
        display: flex;
        background: #252526;
        border-bottom: 1px solid #2d2d30;
        min-height: 35px;
        overflow-x: auto;
    ";
    
    let tab_style = "
        display: flex;
        align-items: center;
        padding: 0 24px 0 12px;
        background: #2d2d30;
        border-right: 1px solid #252526;
        cursor: pointer;
        position: relative;
        font-size: 13px;
        color: #cccccc;
        white-space: nowrap;
    ";
    
    let close_btn_style = "
        position: absolute;
        right: 4px;
        top: 50%;
        transform: translateY(-50%);
        width: 20px;
        height: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 3px;
        background: transparent;
        border: none;
        color: #cccccc;
        cursor: pointer;
        font-size: 12px;
    ";
    
    let close_btn_hover_style = "
        position: absolute;
        right: 4px;
        top: 50%;
        transform: translateY(-50%);
        width: 20px;
        height: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 3px;
        background: #3e3e42;
        border: none;
        color: #cccccc;
        cursor: pointer;
        font-size: 12px;
    ";
    
    let editor_header_style = "
        display: flex;
        align-items: center;
        padding: 8px 16px;
        background: #2d2d30;
        border-bottom: 1px solid #464647;
        min-height: 35px;
    ";
    
    let editor_content_style = "
        display: flex;
        flex: 1;
        overflow: hidden;
    ";
    
    let line_numbers_style = "
        background: #1e1e1e;
        color: #858585;
        padding: 16px 16px 16px 24px;
        text-align: right;
        user-select: none;
        font-size: 14px;
        line-height: 20px;
        min-width: 50px;
        border-right: 1px solid #2d2d30;
    ";
    
    let code_content_style = "
        flex: 1;
        padding: 16px 24px;
        overflow: auto;
        font-size: 14px;
        line-height: 20px;
        white-space: pre;
        tab-size: 4;
    ";
    
    let welcome_style = "
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: #858585;
    ";
    
    // State for close button hover
    let close_hovered = use_signal(|| false);
    
    rsx! {
        div {
            style: "{editor_container_style}",
            
            // Tab Bar
            if let Some(path) = &selected_file {
                div {
                    style: "{tabs_container_style}",
                    div {
                        style: "{tab_style}",
                        
                        // File icon
                        span {
                            style: "margin-right: 6px; font-size: 14px;",
                            {get_file_icon(&path)}
                        }
                        
                        // File name
                        span {
                            {path.file_name().unwrap_or_default().to_string_lossy()}
                        }
                        
                        // Close button
                        button {
                            style: if close_hovered() { close_btn_hover_style } else { close_btn_style },
                            onmouseenter: move |_| close_hovered.set(true),
                            onmouseleave: move |_| close_hovered.set(false),
                            onclick: move |evt| {
                                evt.stop_propagation();
                                app_state.write().file_explorer.selected_file = None;
                            },
                            "×"
                        }
                    }
                }
            }
            
            // Breadcrumb Header
            div {
                style: "{editor_header_style}",
                if let Some(path) = &selected_file {
                    // Show full path as breadcrumb
                    if let Ok(current_dir) = std::env::current_dir() {
                        if let Ok(relative_path) = path.strip_prefix(&current_dir) {
                            span {
                                style: "font-size: 12px; color: #858585;",
                                "{relative_path.display()}"
                            }
                        }
                    }
                } else {
                    span {
                        style: "font-size: 13px; color: #858585;",
                        "No file selected"
                    }
                }
            }
            
            // Editor Content
            if selected_file.is_some() && !file_content.read().is_empty() {
                div {
                    style: "{editor_content_style}",
                    
                    // Line Numbers
                    div {
                        style: "{line_numbers_style}",
                        for line_num in 1..=line_count() {
                            div { "{line_num}" }
                        }
                    }
                    
                    // Code Content with Syntax Highlighting
                    div {
                        style: "{code_content_style}",
                        dangerous_inner_html: get_highlighted_code(&file_content.read(), &selected_file),
                    }
                }
            } else {
                // Welcome Screen
                div {
                    style: "{welcome_style}",
                    div {
                        style: "text-align: center;",
                        span {
                            style: "font-size: 48px; display: block; margin-bottom: 16px;",
                            "🐝"
                        }
                        span {
                            style: "font-size: 16px; display: block; margin-bottom: 24px;",
                            "Select a file to edit"
                        }
                        span {
                            style: "font-size: 13px; color: #858585; display: block;",
                            "Use Ctrl+P to open command palette"
                        }
                    }
                }
            }
        }
    }
}

/// Get file icon based on extension
fn get_file_icon(path: &PathBuf) -> &'static str {
    if let Some(ext) = path.extension() {
        match ext.to_str() {
            Some("rs") => "🦀",
            Some("js") | Some("jsx") => "📙",
            Some("ts") | Some("tsx") => "📘",
            Some("py") => "🐍",
            Some("json") => "📋",
            Some("toml") => "⚙️",
            Some("md") => "📝",
            Some("html") | Some("htm") => "🌐",
            Some("css") => "🎨",
            Some("sh") | Some("bash") => "🖥️",
            Some("txt") => "📄",
            _ => "📄",
        }
    } else {
        "📄"
    }
}

/// Get syntax highlighted code as HTML
fn get_highlighted_code(content: &str, path: &Option<PathBuf>) -> String {
    let language = detect_language(path);
    
    match language {
        Language::Rust => highlight_rust(content),
        Language::JavaScript | Language::TypeScript => highlight_javascript(content),
        Language::Python => highlight_python(content),
        Language::Json => highlight_json(content),
        Language::Toml => highlight_toml(content),
        Language::Markdown => highlight_markdown(content),
        Language::Html => highlight_html(content),
        Language::Css => highlight_css(content),
        Language::Shell => highlight_shell(content),
        Language::PlainText => html_escape(content),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Json,
    Toml,
    Markdown,
    Html,
    Css,
    Shell,
    PlainText,
}

fn detect_language(path: &Option<PathBuf>) -> Language {
    if let Some(path) = path {
        if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("rs") => Language::Rust,
                Some("js") | Some("jsx") => Language::JavaScript,
                Some("ts") | Some("tsx") => Language::TypeScript,
                Some("py") => Language::Python,
                Some("json") => Language::Json,
                Some("toml") => Language::Toml,
                Some("md") => Language::Markdown,
                Some("html") | Some("htm") => Language::Html,
                Some("css") => Language::Css,
                Some("sh") | Some("bash") => Language::Shell,
                _ => Language::PlainText,
            }
        } else {
            Language::PlainText
        }
    } else {
        Language::PlainText
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn highlight_rust(code: &str) -> String {
    let keywords = &[
        "fn", "let", "mut", "const", "if", "else", "match", "for", "while", "loop",
        "impl", "trait", "struct", "enum", "pub", "use", "mod", "self", "super",
        "return", "break", "continue", "async", "await", "move", "ref", "as",
        "type", "where", "static", "extern", "unsafe", "in", "crate"
    ];
    
    let types = &[
        "String", "str", "Vec", "Option", "Result", "Box", "Rc", "Arc",
        "i8", "i16", "i32", "i64", "i128", "isize",
        "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char", "Self"
    ];
    
    highlight_with_rules(code, keywords, types)
}

fn highlight_javascript(code: &str) -> String {
    let keywords = &[
        "function", "var", "let", "const", "if", "else", "for", "while", "do",
        "return", "break", "continue", "switch", "case", "default", "try", "catch",
        "finally", "throw", "new", "this", "class", "extends", "super", "import",
        "export", "from", "async", "await", "yield", "typeof", "instanceof", "in",
        "of", "delete", "void", "null", "undefined", "true", "false"
    ];
    
    let types = &[
        "Array", "Object", "String", "Number", "Boolean", "Promise", "Map", "Set",
        "Date", "RegExp", "Error", "Function", "Symbol", "BigInt"
    ];
    
    highlight_with_rules(code, keywords, types)
}

fn highlight_python(code: &str) -> String {
    let keywords = &[
        "def", "class", "if", "elif", "else", "for", "while", "break", "continue",
        "return", "yield", "import", "from", "as", "try", "except", "finally",
        "raise", "with", "pass", "lambda", "in", "is", "not", "and", "or",
        "True", "False", "None", "self", "async", "await", "global", "nonlocal"
    ];
    
    let types = &[
        "int", "float", "str", "list", "dict", "set", "tuple", "bool", "bytes",
        "type", "object", "range", "enumerate", "zip", "map", "filter"
    ];
    
    highlight_with_rules(code, keywords, types)
}

fn highlight_json(code: &str) -> String {
    let mut result = String::new();
    let mut chars = code.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;
    
    while let Some(ch) = chars.next() {
        if escape_next {
            result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
            escape_next = false;
            continue;
        }
        
        match ch {
            '"' => {
                in_string = !in_string;
                result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
            }
            '\\' if in_string => {
                escape_next = true;
                result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
            }
            '{' | '}' | '[' | ']' => {
                result.push_str(&format!("<span style=\"color: #d4d4d4;\">{}</span>", html_escape(&ch.to_string())));
            }
            ':' | ',' => {
                result.push_str(&format!("<span style=\"color: #d4d4d4;\">{}</span>", html_escape(&ch.to_string())));
            }
            '0'..='9' | '-' | '.' if !in_string => {
                result.push_str(&format!("<span style=\"color: #b5cea8;\">{}</span>", html_escape(&ch.to_string())));
            }
            _ => {
                if in_string {
                    result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
                } else {
                    result.push_str(&html_escape(&ch.to_string()));
                }
            }
        }
    }
    
    result
}

fn highlight_toml(code: &str) -> String {
    let mut result = String::new();
    
    for line in code.lines() {
        let trimmed = line.trim_start();
        
        if trimmed.starts_with('#') {
            // Comment
            result.push_str(&format!("<span style=\"color: #6a9955;\">{}</span>\n", html_escape(line)));
        } else if trimmed.starts_with('[') {
            // Section header
            result.push_str(&format!("<span style=\"color: #569cd6;\">{}</span>\n", html_escape(line)));
        } else if let Some(eq_pos) = line.find('=') {
            // Key-value pair
            let key = &line[..eq_pos];
            let value = &line[eq_pos..];
            result.push_str(&format!("<span style=\"color: #4ec9b0;\">{}</span>", html_escape(key)));
            result.push_str(&html_escape(value));
            result.push('\n');
        } else {
            result.push_str(&html_escape(line));
            result.push('\n');
        }
    }
    
    result
}

fn highlight_markdown(code: &str) -> String {
    let mut result = String::new();
    
    for line in code.lines() {
        let trimmed = line.trim_start();
        
        if trimmed.starts_with('#') {
            // Headers
            result.push_str(&format!("<span style=\"color: #569cd6; font-weight: bold;\">{}</span>\n", html_escape(line)));
        } else if trimmed.starts_with("```") {
            // Code blocks
            result.push_str(&format!("<span style=\"color: #6a9955;\">{}</span>\n", html_escape(line)));
        } else if trimmed.starts_with('*') || trimmed.starts_with('-') || trimmed.starts_with('+') {
            // Lists
            result.push_str(&format!("<span style=\"color: #d4d4d4;\">{}</span>\n", html_escape(line)));
        } else {
            // Regular text
            result.push_str(&html_escape(line));
            result.push('\n');
        }
    }
    
    result
}

fn highlight_html(code: &str) -> String {
    let mut result = String::new();
    let mut chars = code.chars().peekable();
    let mut in_tag = false;
    let mut in_string = false;
    let mut string_char = ' ';
    
    while let Some(ch) = chars.next() {
        match ch {
            '<' if !in_string => {
                in_tag = true;
                result.push_str(&format!("<span style=\"color: #808080;\">{}</span>", html_escape(&ch.to_string())));
            }
            '>' if !in_string => {
                in_tag = false;
                result.push_str(&format!("<span style=\"color: #808080;\">{}</span>", html_escape(&ch.to_string())));
            }
            '"' | '\'' if in_tag => {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                } else if ch == string_char {
                    in_string = false;
                }
                result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
            }
            _ => {
                if in_string {
                    result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
                } else if in_tag {
                    result.push_str(&format!("<span style=\"color: #569cd6;\">{}</span>", html_escape(&ch.to_string())));
                } else {
                    result.push_str(&html_escape(&ch.to_string()));
                }
            }
        }
    }
    
    result
}

fn highlight_css(code: &str) -> String {
    let mut result = String::new();
    let mut chars = code.chars().peekable();
    let mut in_selector = true;
    let mut in_property = false;
    let mut in_value = false;
    let mut in_string = false;
    let mut string_char = ' ';
    
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                in_selector = false;
                in_property = true;
                result.push_str(&format!("<span style=\"color: #d4d4d4;\">{}</span>", html_escape(&ch.to_string())));
            }
            '}' => {
                in_selector = true;
                in_property = false;
                in_value = false;
                result.push_str(&format!("<span style=\"color: #d4d4d4;\">{}</span>", html_escape(&ch.to_string())));
            }
            ':' if in_property && !in_string => {
                in_property = false;
                in_value = true;
                result.push_str(&format!("<span style=\"color: #d4d4d4;\">{}</span>", html_escape(&ch.to_string())));
            }
            ';' if in_value && !in_string => {
                in_value = false;
                in_property = true;
                result.push_str(&format!("<span style=\"color: #d4d4d4;\">{}</span>", html_escape(&ch.to_string())));
            }
            '"' | '\'' => {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                } else if ch == string_char {
                    in_string = false;
                }
                result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
            }
            _ => {
                if in_string {
                    result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
                } else if in_selector {
                    result.push_str(&format!("<span style=\"color: #4ec9b0;\">{}</span>", html_escape(&ch.to_string())));
                } else if in_property {
                    result.push_str(&format!("<span style=\"color: #569cd6;\">{}</span>", html_escape(&ch.to_string())));
                } else {
                    result.push_str(&html_escape(&ch.to_string()));
                }
            }
        }
    }
    
    result
}

fn highlight_shell(code: &str) -> String {
    let mut result = String::new();
    
    for line in code.lines() {
        let trimmed = line.trim_start();
        
        if trimmed.starts_with('#') {
            // Comment
            result.push_str(&format!("<span style=\"color: #6a9955;\">{}</span>\n", html_escape(line)));
        } else {
            // Simple word-based highlighting
            let words: Vec<&str> = line.split_whitespace().collect();
            let mut first = true;
            for word in words {
                if !first {
                    result.push(' ');
                }
                
                if first && !word.starts_with('$') {
                    // Command
                    result.push_str(&format!("<span style=\"color: #569cd6;\">{}</span>", html_escape(word)));
                } else if word.starts_with('-') {
                    // Flag
                    result.push_str(&format!("<span style=\"color: #4ec9b0;\">{}</span>", html_escape(word)));
                } else if word.starts_with('$') {
                    // Variable
                    result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(word)));
                } else {
                    result.push_str(&html_escape(word));
                }
                
                first = false;
            }
            result.push('\n');
        }
    }
    
    result
}

fn highlight_with_rules(code: &str, keywords: &[&str], types: &[&str]) -> String {
    let mut result = String::new();
    let mut chars = code.chars().peekable();
    let mut in_string = false;
    let mut in_comment = false;
    let mut string_char = ' ';
    
    while let Some(ch) = chars.next() {
        // Handle strings
        if !in_comment && (ch == '"' || ch == '\'') {
            if !in_string {
                in_string = true;
                string_char = ch;
                result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
            } else if ch == string_char {
                in_string = false;
                result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
            } else {
                result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
            }
            continue;
        }
        
        if in_string {
            result.push_str(&format!("<span style=\"color: #ce9178;\">{}</span>", html_escape(&ch.to_string())));
            continue;
        }
        
        // Handle comments
        if ch == '/' && chars.peek() == Some(&'/') {
            in_comment = true;
        }
        
        if in_comment {
            result.push_str(&format!("<span style=\"color: #6a9955;\">{}</span>", html_escape(&ch.to_string())));
            if ch == '\n' {
                in_comment = false;
            }
            continue;
        }
        
        // Handle identifiers
        if ch.is_alphabetic() || ch == '_' {
            let word: String = std::iter::once(ch)
                .chain(chars.by_ref().take_while(|&c| c.is_alphanumeric() || c == '_'))
                .collect();
            
            if keywords.contains(&word.as_str()) {
                result.push_str(&format!("<span style=\"color: #569cd6;\">{}</span>", html_escape(&word)));
            } else if types.contains(&word.as_str()) {
                result.push_str(&format!("<span style=\"color: #4ec9b0;\">{}</span>", html_escape(&word)));
            } else {
                result.push_str(&html_escape(&word));
            }
        } else if ch.is_numeric() {
            let number: String = std::iter::once(ch)
                .chain(chars.by_ref().take_while(|&c| c.is_numeric() || c == '.'))
                .collect();
            result.push_str(&format!("<span style=\"color: #b5cea8;\">{}</span>", html_escape(&number)));
        } else {
            result.push_str(&html_escape(&ch.to_string()));
        }
    }
    
    result
}