//! True Terminal Emulator for Dioxus Desktop
//!
//! A pure Rust terminal emulator using alacritty_terminal backend
//! that provides full VT100/ANSI support for running interactive CLI tools

pub mod alacritty_registry;
pub mod backend;
pub mod config;
pub mod grid_renderer;
pub mod input;
pub mod pty_manager;
pub mod terminal_widget;

pub use terminal_widget::TerminalEmulator;

// Re-export commonly used types
pub use alacritty_terminal::{
    grid::{Dimensions, GridIterator},
    term::cell::Cell,
    vte::ansi::Color,
};
