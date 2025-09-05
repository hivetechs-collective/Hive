// This file is now deprecated - all analytics commands are in bridge.rs
// Keeping for backwards compatibility during migration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyConversationHistory {
    pub id: String,
    pub query: String,
    pub response: String,
    pub profile_id: String,
    pub cost: f64,
    pub tokens: u32,
    pub duration_ms: u64,
    pub created_at: String,
}