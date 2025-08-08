// This file is now deprecated - all settings commands are in bridge.rs
// Keeping for backwards compatibility during migration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyApiKeyStatus {
    pub openrouter_configured: bool,
    pub hive_configured: bool,
    pub anthropic_configured: bool,
}