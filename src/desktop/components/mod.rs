//! Desktop UI Components

pub mod auto_accept_control;
pub mod button;
pub mod confidence_display;
pub mod logo;
pub mod progress;
pub mod spinner;

pub use auto_accept_control::{AutoAcceptControl, UserFeedback};
pub use button::{Button, ButtonStyle};
pub use confidence_display::{
    ConfidenceDisplay, ConfidenceIndicator, RiskIndicator, OverallScoreIndicator,
    ComponentScoresBreakdown, RealTimeConfidenceMonitor, MiniConfidenceIndicator,
    IndicatorSize
};
pub use logo::{HiveLogo, HiveLogoLarge, HiveLogoSmall};
pub use progress::ProgressBar;
pub use spinner::LoadingSpinner;
