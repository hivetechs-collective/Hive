//! AI-powered code transformation engine
//!
//! This module provides intelligent code transformation capabilities using the consensus engine
//! to suggest and apply code improvements while maintaining syntax correctness and supporting
//! full undo/redo functionality.

pub mod engine;
pub mod simple_engine;
pub mod syntax;
pub mod conflict;
pub mod preview;
pub mod history;
pub mod applier;
pub mod types;
pub mod operations;
pub mod validation;

#[cfg(test)]
mod tests;

// Export both full and simple engines
pub use engine::TransformationEngine;
pub use simple_engine::SimpleTransformationEngine;
pub use syntax::SyntaxAwareModifier;
pub use conflict::ConflictResolver;
pub use preview::PreviewSystem;
pub use history::{TransformationHistory, Transaction};
pub use applier::CodeApplier;
pub use types::*;

// Re-export commonly used items
pub use engine::transform_code;
pub use simple_engine::{simple_transform_code, simple_generate_preview};
pub use preview::generate_preview;
pub use history::undo_last_transformation;
pub use operations::{Operation, OperationsManager};
pub use validation::{TransformationValidator, ValidationReport, quick_validate};