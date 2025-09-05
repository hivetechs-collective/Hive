//! Hive AI: Lightning-fast codebase intelligence platform
//!
//! Complete Rust reimplementation with 100% feature parity plus revolutionary enhancements.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![recursion_limit = "1024"]

// Full module list - ALL MODULES ENABLED
pub mod ai_helpers;
pub mod analysis;
pub mod analytics;
pub mod cache;
pub mod cli;
pub mod commands;
pub mod consensus;
pub mod core;
pub mod database;
pub mod desktop;
pub mod hooks;
pub mod ide;
pub mod install;
pub mod integration;
pub mod maintenance;
pub mod memory;
pub mod migration;
pub mod modes;
pub mod planning;
pub mod providers;
pub mod security;
pub mod shell;
pub mod startup;
pub mod subscription;
pub mod transformation;
pub mod updates;

// Version constant
pub const VERSION: &str = "2.0.2";

// Re-export core types
pub use core::{
    config::Config,
    error::{HiveError, Result},
    logging::initialize_default_logging,
};

// Re-export major subsystems
pub use cli::CliFramework;
pub use desktop::launch_desktop_app;
