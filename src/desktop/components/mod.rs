//! Desktop UI Components

// Temporarily commented out to fix compilation
// pub mod auto_accept_control;
// pub mod button;
// pub mod confidence_display;
// pub mod feedback_interface;
// pub mod logo;
// pub mod progress;
// pub mod rollback_ui;
// pub mod learning_monitor;
// pub mod spinner;
pub mod test_simple;

// Temporarily commented out to fix compilation
// pub use auto_accept_control::{AutoAcceptControl, UserFeedback};
// pub use feedback_interface::{
//     OperationConfirmationDialog, QuickFeedbackWidget, OperationFeedback, QuickFeedback
// };
// pub use rollback_ui::{
//     RollbackExecutionDialog, RollbackProgressDisplay, RollbackPlanOverview,
//     RollbackExecutionResults, RollbackStepDetails, RollbackErrorDisplay,
//     RollbackSummaryDisplay
// };
// pub use learning_monitor::{
//     LearningSystemDashboard, LearningSystemOverview, HelperPerformanceCard,
//     HelperDetailPanel, PendingImprovementsSection, LearningHistorySection
// };
// pub use button::{Button, ButtonStyle};
// pub use confidence_display::{
//     ConfidenceDisplay, ConfidenceIndicator, RiskIndicator, OverallScoreIndicator,
//     ComponentScoresBreakdown, RealTimeConfidenceMonitor, MiniConfidenceIndicator,
//     IndicatorSize
// };
// pub use logo::{HiveLogo, HiveLogoLarge, HiveLogoSmall};
// pub use progress::ProgressBar;
// pub use spinner::LoadingSpinner;
pub use test_simple::SimpleTest;

// Common components
pub mod common;

// New AI-Enhanced Auto-Accept components
pub mod auto_accept_settings;
pub mod operation_preview;
pub mod approval_interface;
pub mod progress_indicators;
pub mod notifications;
pub mod inline_operation_display;

pub use auto_accept_settings::AutoAcceptSettings;
pub use operation_preview::OperationPreview;
pub use approval_interface::{ApprovalInterface, ApprovalDecision};
pub use progress_indicators::{ProgressIndicators, OperationProgress};
pub use notifications::{NotificationSystem, Notification, NotificationType};
pub use inline_operation_display::{InlineOperationDisplay, OperationStatus, ResponseSection, parse_operations_from_content};
