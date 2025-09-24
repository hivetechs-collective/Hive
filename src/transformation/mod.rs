//! AI-powered code transformation engine
//!
//! This module provides intelligent code transformation capabilities using the consensus engine
//! to suggest and apply code improvements while maintaining syntax correctness and supporting
//! full undo/redo functionality.

pub mod applier;
pub mod conflict;
pub mod engine;
pub mod history;
pub mod operations;
pub mod preview;
pub mod simple_engine;
pub mod syntax;
pub mod types;
pub mod validation;

#[cfg(all(test, feature = "legacy-tests"))]
mod tests;

// Export both full and simple engines
pub use applier::CodeApplier;
pub use conflict::ConflictResolver;
pub use engine::TransformationEngine;
pub use history::{Transaction, TransformationHistory};
pub use preview::PreviewSystem;
pub use simple_engine::SimpleTransformationEngine;
pub use syntax::SyntaxAwareModifier;
pub use types::*;

// Re-export commonly used items
pub use engine::transform_code;
pub use history::undo_last_transformation;
pub use operations::{Operation, OperationsManager};
pub use preview::generate_preview;
pub use simple_engine::{simple_generate_preview, simple_transform_code};
pub use validation::{quick_validate, TransformationValidator, ValidationReport};
