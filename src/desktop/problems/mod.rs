//! Problems Panel Integration
//! 
//! Provides VS Code-style problems panel for displaying:
//! - Git conflicts
//! - Build errors and warnings
//! - Lint issues
//! - Navigation to problem locations
//! - Real-time updates from build systems

pub mod problems_panel;
pub mod build_integration;
pub mod navigation_handler;
pub mod real_time_updates;
pub mod status_bar_integration;

pub use problems_panel::{
    ProblemsPanel, ProblemItem, ProblemSeverity, ProblemSource, ProblemsState,
    PROBLEMS_PANEL_STYLES,
};

pub use build_integration::{
    BuildSystemIntegration, BuildTool, BuildStats,
};

pub use navigation_handler::{
    ProblemNavigationHandler, NavigationResult, EditorIntegration,
    NavigationEvent, NavigationHistoryEntry, NavigationAction,
    navigation_utils,
};

pub use real_time_updates::{
    ProblemsUpdateManager, UpdateEvent, UpdateFrequency,
};

pub use status_bar_integration::{
    ProblemsStatusBar, DetailedProblemsStatusBar, STATUS_BAR_PROBLEMS_STYLES,
};