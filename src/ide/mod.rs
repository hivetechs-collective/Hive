//! IDE Layer - Intelligent Brokers for Development Environment
//!
//! This module contains IDE-level services that bridge between the File Explorer
//! state and the consensus engine. AI Helpers live at this layer to provide
//! intelligent context and decision-making for development workflows.

pub mod ai_helper_broker;

pub use ai_helper_broker::IDEAIHelperBroker;
