//! Stub module for terminal CWD tracking

use dioxus::prelude::*;

#[derive(Clone)]
pub struct TerminalCwdTracker;

impl TerminalCwdTracker {
    pub fn new() -> Self {
        Self
    }
}

pub fn provide_terminal_cwd_tracker() -> TerminalCwdTracker {
    use_context_provider(|| TerminalCwdTracker::new())
}

pub fn use_terminal_cwd_tracker() -> TerminalCwdTracker {
    use_context()
}
