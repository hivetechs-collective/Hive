//! Markdown rendering utilities for the desktop application

use pulldown_cmark::{html, Parser};

/// Convert markdown to HTML with enhanced features
pub fn to_html(markdown: &str) -> String {
    // Parse markdown
    let parser = Parser::new(markdown);
    
    // Convert to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    
    // Wrap in a div for consistent styling
    format!(r#"<div class="markdown-content">{}</div>"#, html_output)
}

/// Convert markdown chunk to HTML for streaming display
/// This preserves incomplete markdown structures better
pub fn to_html_streaming(chunk: &str, accumulated: &str) -> String {
    // For streaming, we convert the entire accumulated content
    // to ensure proper markdown structure parsing
    to_html(accumulated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let markdown = "# Hello\n\nThis is **bold** text.";
        let html = to_html(markdown);
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_code_blocks() {
        let markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let html = to_html(markdown);
        assert!(html.contains("<pre>"));
        assert!(html.contains("<code"));
    }
}