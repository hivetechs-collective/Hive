//! Event Bus System for Desktop Application
//!
//! Provides a centralized event handling system with publish-subscribe pattern
//! for managing application events across different components.

pub mod bus;
pub mod global;
pub mod types;

// Re-export main types
pub use bus::{AsyncEventHandler, EventBus, EventHandler, SubscriptionHandle};
pub use global::{event_bus, GLOBAL_EVENT_BUS};
pub use types::{
    ConsensusStage, Event, EventPayload, EventType, FileChangeType, Problem, ProblemSeverity,
};

// Re-export legacy event types for compatibility
mod legacy;
pub use legacy::{
    AppEvent, EventDispatcher, EventHandler as LegacyEventHandler, KeyboardEventUtils,
    MouseEventUtils,
};

// Include examples module for documentation
#[cfg(any(test, doc))]
pub mod examples;

// Integration examples
#[cfg(any(test, doc))]
pub mod git_integration;
#[cfg(any(test, doc))]
pub mod problems_integration;
