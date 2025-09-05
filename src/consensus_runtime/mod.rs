//! Consensus Runtime - Minimal wrapper to run existing consensus on Tokio thread
//!
//! This module provides a thin wrapper around the existing DesktopConsensusManager
//! to run it on a dedicated Tokio thread instead of the UI thread.
//! ALL existing consensus logic remains unchanged.

pub mod wrapper;
pub mod pools;
pub mod http_pool;

pub use wrapper::ConsensusThreadWrapper;
pub use pools::{MemoryPools, TokenPool, BufferPool, GLOBAL_POOLS};
pub use http_pool::{POOLED_CLIENT, pooled_request, get_metrics};