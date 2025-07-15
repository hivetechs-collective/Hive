//! Profile Service - Manages reactive profile state

use crate::desktop::state::{ActiveProfileData, ProfileState, ACTIVE_PROFILE_STATE};
use crate::core::database::get_database;
use rusqlite::OptionalExtension;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

/// Service that manages profile state and database synchronization
pub struct ProfileService;

impl ProfileService {
    /// Start the profile service - loads initial profile and starts watching for changes
    pub fn start() {
        // Load initial profile
        spawn(async {
            Self::load_and_update_profile().await;
            
            // Start background watcher
            Self::start_watcher().await;
        });
    }

    /// Load profile from database and update global signal
    async fn load_and_update_profile() {
        info!("🔄 ProfileService: Loading profile from database");
        
        match Self::load_profile_from_database().await {
            Ok(profile_data) => {
                info!("✅ ProfileService: Loaded profile: {}", profile_data.name);
                *ACTIVE_PROFILE_STATE.write() = ProfileState::Loaded(profile_data);
            }
            Err(e) => {
                error!("❌ ProfileService: Failed to load profile: {}", e);
                *ACTIVE_PROFILE_STATE.write() = ProfileState::Error(e.to_string());
            }
        }
    }

    /// Background watcher that checks for profile changes
    async fn start_watcher() {
        info!("🔍 ProfileService: Starting background watcher");
        
        loop {
            sleep(Duration::from_secs(5)).await; // Check every 5 seconds
            
            // Check if profile has changed
            match Self::load_profile_from_database().await {
                Ok(new_profile) => {
                    let current_state = ACTIVE_PROFILE_STATE();
                    
                    // Only update if profile has changed
                    if let ProfileState::Loaded(current_profile) = &current_state {
                        if *current_profile != new_profile {
                            info!("🔄 ProfileService: Profile changed, updating to: {}", new_profile.name);
                            *ACTIVE_PROFILE_STATE.write() = ProfileState::Loaded(new_profile);
                        }
                    } else {
                        // State is Loading or Error, update it
                        *ACTIVE_PROFILE_STATE.write() = ProfileState::Loaded(new_profile);
                    }
                }
                Err(e) => {
                    error!("❌ ProfileService: Watcher failed to load profile: {}", e);
                    *ACTIVE_PROFILE_STATE.write() = ProfileState::Error(e.to_string());
                }
            }
        }
    }

    /// Load profile from database
    async fn load_profile_from_database() -> anyhow::Result<ActiveProfileData> {
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        // Get active profile ID
        let active_profile_id: Option<String> = conn.query_row(
            "SELECT value FROM consensus_settings WHERE key = 'active_profile_id'",
            [],
            |row| row.get(0)
        ).optional()?;
        
        let profile_id = active_profile_id
            .ok_or_else(|| anyhow::anyhow!("No active profile configured"))?;
        
        // Get profile data
        let profile = conn.query_row(
            "SELECT profile_name, generator_model, refiner_model, validator_model, curator_model FROM consensus_profiles WHERE id = ?1",
            rusqlite::params![profile_id],
            |row| {
                Ok(ActiveProfileData {
                    name: row.get(0)?,
                    generator_model: row.get(1)?,
                    refiner_model: row.get(2)?,
                    validator_model: row.get(3)?,
                    curator_model: row.get(4)?,
                })
            }
        )?;
        
        Ok(profile)
    }

    /// Force reload the profile (useful when settings are changed)
    pub fn force_reload() {
        spawn(async {
            Self::load_and_update_profile().await;
        });
    }
}

/// Helper function to spawn async tasks
fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}