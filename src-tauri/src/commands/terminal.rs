// Terminal commands have been moved to bridge.rs to use existing PTY implementation
// Re-export the terminal info type for backwards compatibility
pub use crate::bridge::TerminalInfo;

// These are now handled by bridge.rs:
// - create_terminal (uses existing PtyProcess)
// - write_to_terminal
// - resize_terminal
// - close_terminal