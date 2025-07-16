//! Code Editor Module for HiveTechs IDE
//! 
//! Provides VS Code-like code editing capabilities with syntax highlighting,
//! IntelliSense, and Git integration.

pub mod buffer;
pub mod cursor;
pub mod editor;
pub mod highlighting;
pub mod language;
pub mod renderer;
pub mod git_integration;
pub mod completion;

pub use editor::{CodeEditor, EditorConfig};
pub use buffer::TextBuffer;
pub use cursor::{Cursor, Selection};
pub use highlighting::SyntaxHighlighter;
pub use language::LanguageService;