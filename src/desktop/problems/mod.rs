//! Problems Panel Integration
//! 
//! Provides VS Code-style problems panel for displaying:
//! - Git conflicts
//! - Build errors and warnings
//! - Lint issues
//! - Navigation to problem locations

pub mod problems_panel;

pub use problems_panel::{
    ProblemsPanel, ProblemItem, ProblemSeverity, ProblemSource, ProblemsState,
    PROBLEMS_PANEL_STYLES,
};