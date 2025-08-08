// Bridge module - Aggregates all bridge sub-modules

pub mod profiles;
pub mod settings;
pub mod git;

// Re-export all commands for easy registration
pub use profiles::*;
pub use settings::*;
pub use git::*;