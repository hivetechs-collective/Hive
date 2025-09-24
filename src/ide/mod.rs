//! IDE Layer - Intelligent Brokers for Development Environment
//!
//! This module contains IDE-level services that bridge between the File Explorer
//! state and the consensus engine. AI Helpers live at this layer to provide
//! intelligent context and decision-making for development workflows.

#[cfg(feature = "desktop-legacy")]
pub mod ai_helper_broker;

#[cfg(feature = "desktop-legacy")]
pub use ai_helper_broker::IDEAIHelperBroker;
