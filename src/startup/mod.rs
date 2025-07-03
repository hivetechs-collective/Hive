//! Startup Optimization Module
//! 
//! Provides comprehensive startup optimization for achieving <25ms startup time.

pub mod fast_boot;

pub use fast_boot::{FastBootOptimizer, FastBootConfig, StartupMetrics};