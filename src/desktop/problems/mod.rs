//! Problems Panel Integration
//!
//! Provides VS Code-style problems panel for displaying:
//! - Git conflicts
//! - Build errors and warnings
//! - Lint issues
//! - Navigation to problem locations
//! - Real-time updates from build systems

pub mod build_integration;
pub mod navigation_handler;
pub mod problems_panel;
pub mod real_time_updates;
pub mod status_bar_integration;

pub use problems_panel::{
    ProblemItem, ProblemSeverity, ProblemSource, ProblemsPanel, ProblemsState,
    PROBLEMS_PANEL_STYLES,
};

pub use build_integration::{BuildStats, BuildSystemIntegration, BuildTool};

pub use navigation_handler::{
    navigation_utils, EditorIntegration, NavigationAction, NavigationEvent, NavigationHistoryEntry,
    NavigationResult, ProblemNavigationHandler,
};

pub use real_time_updates::{ProblemsUpdateManager, UpdateEvent, UpdateFrequency};

pub use status_bar_integration::{
    DetailedProblemsStatusBar, ProblemsStatusBar, STATUS_BAR_PROBLEMS_STYLES,
};
