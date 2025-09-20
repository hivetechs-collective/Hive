//! Consensus Runtime - Minimal wrapper to run existing consensus on Tokio thread
//!
//! This module provides a thin wrapper around the existing DesktopConsensusManager
//! to run it on a dedicated Tokio thread instead of the UI thread.
//! ALL existing consensus logic remains unchanged.

pub mod http_pool;
pub mod pools;
pub mod wrapper;

pub use http_pool::{get_metrics, pooled_request, POOLED_CLIENT};
pub use pools::{BufferPool, MemoryPools, TokenPool, GLOBAL_POOLS};
pub use wrapper::ConsensusThreadWrapper;
