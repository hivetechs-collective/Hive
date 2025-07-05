//! Hive AI: Lightning-fast codebase intelligence platform
//! 
//! Complete Rust reimplementation with 100% feature parity plus revolutionary enhancements.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![recursion_limit = "1024"]

// Full module list - ALL MODULES ENABLED
pub mod core;
pub mod analysis;
pub mod analytics;
pub mod cache;
pub mod cli;
pub mod commands;
pub mod consensus;
pub mod database;
pub mod hooks;
pub mod integration;
pub mod memory;
pub mod migration;
pub mod modes;
pub mod planning;
pub mod providers;
pub mod security;
pub mod shell;
pub mod install;
pub mod startup;
pub mod transformation;
pub mod tui;
pub mod desktop;
pub mod interactive_tui;

// Version constant
pub const VERSION: &str = "2.0.0";

// Re-export core types
pub use core::{
    error::{HiveError, Result},
    logging::initialize_default_logging,
    config::Config,
};

// Re-export major subsystems
pub use cli::CliFramework;
pub use tui::TuiFramework;
pub use desktop::launch_desktop_app;