//! Scroll handler for desktop app
//! 
//! This module provides auto-scrolling functionality for the desktop app
//! using Dioxus's custom protocol handler to inject JavaScript.

use dioxus::desktop::{Config, WindowBuilder};

/// JavaScript code for auto-scrolling
pub const AUTO_SCROLL_JS: &str = r#"
// Auto-scroll function
window.autoScrollToBottom = function() {
    const responseArea = document.querySelector('.response-area');
    if (responseArea) {
        responseArea.scrollTop = responseArea.scrollHeight;
    }
};

// Set up mutation observer to auto-scroll on content changes
const setupAutoScroll = () => {
    const responseArea = document.querySelector('.response-area');
    if (!responseArea) return;
    
    let shouldAutoScroll = true;
    
    // Detect manual scroll
    responseArea.addEventListener('wheel', () => {
        const isAtBottom = responseArea.scrollHeight - responseArea.clientHeight <= responseArea.scrollTop + 1;
        shouldAutoScroll = isAtBottom;
    });
    
    // Observe content changes
    const observer = new MutationObserver(() => {
        if (shouldAutoScroll) {
            responseArea.scrollTop = responseArea.scrollHeight;
        }
    });
    
    observer.observe(responseArea, {
        childList: true,
        subtree: true,
        characterData: true
    });
};

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', setupAutoScroll);
} else {
    setupAutoScroll();
}
"#;

/// Inject auto-scroll JavaScript into the app
pub fn inject_auto_scroll_script() -> String {
    format!(
        r#"<script>{}</script>"#,
        AUTO_SCROLL_JS
    )
}