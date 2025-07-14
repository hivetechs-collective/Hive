//! Model Maintenance System - Automatic Model Updates and Replacements
//!
//! This module handles:
//! - Validating models in profiles and templates
//! - Finding replacements for deprecated/unavailable models
//! - Migrating user profiles when models change
//! - Periodic sync with OpenRouter API

use crate::consensus::models::{ModelInfo, ModelManager};
use crate::consensus::profiles::ExpertTemplate;
use crate::core::database::DatabaseManager;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Template maintenance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceConfig {
    /// How often to run maintenance (in hours)
    pub interval_hours: u32,
    /// Whether to automatically migrate profiles
    pub auto_migrate: bool,
    /// Whether to validate on startup
    pub validate_on_startup: bool,
}

impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self {
            interval_hours: 24,
            auto_migrate: true,
            validate_on_startup: true,
        }
    }
}

/// Model validation result
#[derive(Debug, Clone)]
pub struct ModelValidation {
    pub model_id: String,
    pub is_valid: bool,
    pub reason: Option<String>,
    pub suggested_replacement: Option<String>,
}

/// Template maintenance manager
pub struct TemplateMaintenanceManager {
    db: Arc<DatabaseManager>,
    model_manager: ModelManager,
    config: MaintenanceConfig,
    last_maintenance: Option<DateTime<Utc>>,
}

impl TemplateMaintenanceManager {
    /// Create new maintenance manager
    pub fn new(db: Arc<DatabaseManager>, api_key: Option<String>) -> Self {
        Self {
            db,
            model_manager: ModelManager::new(api_key),
            config: MaintenanceConfig::default(),
            last_maintenance: None,
        }
    }

    /// Run full maintenance cycle
    pub async fn run_maintenance(&mut self) -> Result<MaintenanceReport> {
        info!("Starting template maintenance cycle");

        let mut report = MaintenanceReport::default();

        // Step 1: Sync models from OpenRouter
        match self.model_manager.sync_models(&self.db).await {
            Ok(count) => {
                info!("Synced {} models from OpenRouter", count);
                report.models_synced = count;
            }
            Err(e) => {
                error!("Failed to sync models: {}", e);
                report.errors.push(format!("Model sync failed: {}", e));
            }
        }

        // Step 2: Validate all profiles
        match self.validate_all_profiles().await {
            Ok(validations) => {
                for (profile_id, validation) in validations {
                    if !validation.iter().all(|v| v.is_valid) {
                        report.invalid_profiles.push(profile_id.clone());

                        // Step 3: Migrate invalid profiles
                        if self.config.auto_migrate {
                            match self.migrate_profile(&profile_id).await {
                                Ok(_) => {
                                    info!("Successfully migrated profile: {}", profile_id);
                                    report.migrated_profiles.push(profile_id);
                                }
                                Err(e) => {
                                    error!("Failed to migrate profile {}: {}", profile_id, e);
                                    report
                                        .errors
                                        .push(format!("Profile migration failed: {}", e));
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to validate profiles: {}", e);
                report
                    .errors
                    .push(format!("Profile validation failed: {}", e));
            }
        }

        self.last_maintenance = Some(Utc::now());
        info!("Template maintenance completed: {:?}", report);

        Ok(report)
    }

    /// Validate a specific model
    pub async fn validate_model(&self, model_id: &str) -> Result<ModelValidation> {
        let conn = self.db.get_connection()?;

        // Check if model exists and is active
        let exists: Option<(bool, String)> = conn
            .query_row(
                "SELECT is_active, name FROM openrouter_models WHERE openrouter_id = ?1",
                [model_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        match exists {
            Some((true, _name)) => Ok(ModelValidation {
                model_id: model_id.to_string(),
                is_valid: true,
                reason: None,
                suggested_replacement: None,
            }),
            Some((false, name)) => {
                // Model exists but is inactive
                let replacement = self.find_model_replacement(model_id).await?;
                Ok(ModelValidation {
                    model_id: model_id.to_string(),
                    is_valid: false,
                    reason: Some(format!("Model '{}' is no longer active", name)),
                    suggested_replacement: replacement,
                })
            }
            None => {
                // Model doesn't exist
                let replacement = self.find_model_replacement(model_id).await?;
                Ok(ModelValidation {
                    model_id: model_id.to_string(),
                    is_valid: false,
                    reason: Some(format!("Model '{}' not found", model_id)),
                    suggested_replacement: replacement,
                })
            }
        }
    }

    /// Find replacement for an unavailable model
    pub async fn find_model_replacement(&self, model_id: &str) -> Result<Option<String>> {
        let conn = self.db.get_connection()?;

        // Extract provider from model ID
        let provider = model_id.split('/').next().unwrap_or("unknown");

        // Strategy 1: Same provider, similar model name
        let similar: Option<String> = conn
            .query_row(
                "SELECT openrouter_id FROM openrouter_models
             WHERE provider_name = ?1
             AND is_active = 1
             AND openrouter_id != ?2
             AND (
                 (openrouter_id LIKE '%turbo%' AND ?2 LIKE '%turbo%') OR
                 (openrouter_id LIKE '%claude%' AND ?2 LIKE '%claude%') OR
                 (openrouter_id LIKE '%gemini%' AND ?2 LIKE '%gemini%') OR
                 (openrouter_id LIKE '%llama%' AND ?2 LIKE '%llama%')
             )
             ORDER BY context_window DESC, pricing_input ASC
             LIMIT 1",
                [provider, model_id],
                |row| row.get(0),
            )
            .optional()?;

        if similar.is_some() {
            return Ok(similar);
        }

        // Strategy 2: Same provider, best available model
        let best_from_provider: Option<String> = conn
            .query_row(
                "SELECT openrouter_id FROM openrouter_models
             WHERE provider_name = ?1
             AND is_active = 1
             ORDER BY context_window DESC, pricing_input ASC
             LIMIT 1",
                [provider],
                |row| row.get(0),
            )
            .optional()?;

        if best_from_provider.is_some() {
            return Ok(best_from_provider);
        }

        // Strategy 3: Cross-provider replacement based on tier
        let tier = self.determine_model_tier(model_id);
        let replacement: Option<String> = conn
            .query_row(
                "SELECT openrouter_id FROM openrouter_models
             WHERE is_active = 1
             AND (
                 (provider_name = 'anthropic' AND openrouter_id LIKE '%claude%') OR
                 (provider_name = 'openai' AND openrouter_id LIKE '%gpt%') OR
                 (provider_name = 'google' AND openrouter_id LIKE '%gemini%')
             )
             ORDER BY
                 CASE
                     WHEN ?1 = 'flagship' THEN context_window
                     WHEN ?1 = 'standard' THEN pricing_input
                     ELSE pricing_input
                 END DESC
             LIMIT 1",
                [&tier],
                |row| row.get(0),
            )
            .optional()?;

        Ok(replacement)
    }

    /// Determine model tier based on name patterns
    fn determine_model_tier(&self, model_id: &str) -> String {
        if model_id.contains("opus") || model_id.contains("gpt-4") || model_id.contains("ultra") {
            "flagship".to_string()
        } else if model_id.contains("turbo")
            || model_id.contains("flash")
            || model_id.contains("haiku")
        {
            "fast".to_string()
        } else {
            "standard".to_string()
        }
    }

    /// Validate all user profiles
    async fn validate_all_profiles(&self) -> Result<HashMap<String, Vec<ModelValidation>>> {
        let conn = self.db.get_connection()?;
        let mut validations = HashMap::new();

        // Get all profiles - they now store OpenRouter IDs directly
        let profiles = {
            let mut stmt = conn.prepare(
                "SELECT cp.id, cp.name,
                        gm.openrouter_id,
                        rm.openrouter_id,
                        vm.openrouter_id,
                        cm.openrouter_id
                 FROM consensus_profiles cp
                 JOIN openrouter_models gm ON cp.generator_model_id = gm.internal_id
                 JOIN openrouter_models rm ON cp.refiner_model_id = rm.internal_id
                 JOIN openrouter_models vm ON cp.validator_model_id = vm.internal_id
                 JOIN openrouter_models cm ON cp.curator_model_id = cm.internal_id",
            )?;

            let profile_rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    vec![
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                    ],
                ))
            })?;

            // Collect into a vector first to avoid Send issues
            profile_rows.collect::<Result<Vec<_>, _>>()?
        };

        for (id, name, models) in profiles {
            info!("Validating profile: {} ({})", name, id);

            let mut profile_validations = Vec::new();
            for model_id in models {
                profile_validations.push(self.validate_model(&model_id).await?);
            }

            validations.insert(id, profile_validations);
        }

        Ok(validations)
    }

    /// Migrate a profile to use valid models
    async fn migrate_profile(&self, profile_id: &str) -> Result<()> {
        let conn = self.db.get_connection()?;

        // Get current models - they are stored directly as OpenRouter IDs
        let (gen_id, ref_id, val_id, cur_id): (String, String, String, String) = conn.query_row(
            "SELECT generator_model, refiner_model, validator_model, curator_model
             FROM consensus_profiles
             WHERE id = ?1",
            [profile_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;

        // Find replacements for invalid models
        let gen_replacement = if self.validate_model(&gen_id).await?.is_valid {
            gen_id
        } else {
            self.find_model_replacement(&gen_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("No replacement found for generator model"))?
        };

        let ref_replacement = if self.validate_model(&ref_id).await?.is_valid {
            ref_id
        } else {
            self.find_model_replacement(&ref_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("No replacement found for refiner model"))?
        };

        let val_replacement = if self.validate_model(&val_id).await?.is_valid {
            val_id
        } else {
            self.find_model_replacement(&val_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("No replacement found for validator model"))?
        };

        let cur_replacement = if self.validate_model(&cur_id).await?.is_valid {
            cur_id
        } else {
            self.find_model_replacement(&cur_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("No replacement found for curator model"))?
        };

        // Update profile with new model IDs directly
        conn.execute(
            "UPDATE consensus_profiles
             SET generator_model = ?1,
                 refiner_model = ?2,
                 validator_model = ?3,
                 curator_model = ?4,
                 updated_at = datetime('now')
             WHERE id = ?5",
            [
                &gen_replacement,
                &ref_replacement,
                &val_replacement,
                &cur_replacement,
                profile_id,
            ],
        )?;

        info!("Migrated profile {} with new models", profile_id);
        Ok(())
    }

    /// Check if maintenance is needed
    pub fn needs_maintenance(&self) -> bool {
        match self.last_maintenance {
            None => true,
            Some(last) => {
                let elapsed = Utc::now() - last;
                elapsed > Duration::hours(self.config.interval_hours as i64)
            }
        }
    }
}

/// Maintenance report
#[derive(Debug, Default)]
pub struct MaintenanceReport {
    pub models_synced: u32,
    pub invalid_profiles: Vec<String>,
    pub migrated_profiles: Vec<String>,
    pub errors: Vec<String>,
}
