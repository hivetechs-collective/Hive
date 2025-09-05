// This file is now deprecated - all consensus commands are in bridge.rs
// Keeping for backwards compatibility during migration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyConsensusResult {
    pub result: String,
    pub total_cost: f64,
    pub total_tokens: u32,
    pub duration_ms: u64,
}

// Legacy commands that just forward to bridge
// These will be removed once frontend is updated