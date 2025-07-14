//! Expert profile templates and consensus pipeline management
//!
//! This module provides pre-built expert templates for different use cases,
//! dynamic model selection, and consensus pipeline configuration.

use crate::core::database::DatabaseManager;
use crate::core::{HiveError, Result};
use anyhow::Context;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expert template category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileCategory {
    Speed,
    Quality,
    Cost,
    Specialized,
    Production,
}

/// Expert level for templates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpertLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Model selection strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SelectionStrategy {
    Fixed,   // Use specific models
    Dynamic, // Select based on current rankings
    Hybrid,  // Mix of fixed and dynamic
}

/// Budget priority setting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BudgetPriority {
    Cost,
    Performance,
    Balanced,
}

/// Performance priority setting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformancePriority {
    Speed,
    Quality,
    Balanced,
}

/// Cost range categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CostRange {
    UltraLow,
    Low,
    Medium,
    High,
    Premium,
}

/// Consensus pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PipelineStage {
    Generator,
    Refiner,
    Validator,
    Curator,
}

/// Model selection criteria for dynamic selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCriteria {
    pub providers: Option<Vec<String>>,
    pub ranking_position: Option<String>, // "top-5", "top-10", "cost-efficient", etc.
    pub context_window: Option<String>,   // "small", "medium", "large", "xl"
    pub capabilities: Option<Vec<String>>,
    pub cost_range: Option<CostRange>,
    pub fallback_model: Option<String>, // OpenRouter ID or internal ID as string
}

/// Budget profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetProfile {
    pub priority: BudgetPriority,
    pub max_cost_per_conversation: Option<f64>,
    pub preferred_cost_range: CostRange,
}

/// Performance profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub priority: PerformancePriority,
    pub max_latency_ms: Option<u32>,
    pub quality_threshold: Option<f64>,
}

/// Stage temperature settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTemperatures {
    pub generator: f64,
    pub refiner: f64,
    pub validator: f64,
    pub curator: f64,
}

/// Fixed model configuration (using internal IDs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedModels {
    pub generator: i64,
    pub refiner: i64,
    pub validator: i64,
    pub curator: i64,
}

/// Dynamic selection criteria for all stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSelectionCriteria {
    pub generator: ModelCriteria,
    pub refiner: ModelCriteria,
    pub validator: ModelCriteria,
    pub curator: ModelCriteria,
}

/// Expert template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: ProfileCategory,
    pub expert_level: ExpertLevel,
    pub selection_strategy: SelectionStrategy,

    // Model configuration
    pub fixed_models: Option<FixedModels>,
    pub selection_criteria: Option<StageSelectionCriteria>,

    // Temperature settings
    pub temperatures: StageTemperatures,

    // Profiles
    pub budget_profile: Option<BudgetProfile>,
    pub performance_profile: Option<PerformanceProfile>,

    // Metadata
    pub use_cases: Vec<String>,
    pub tags: Vec<String>,
}

/// Stored consensus profile
#[derive(Debug, Clone)]
pub struct ConsensusProfile {
    pub id: i64,
    pub name: String,
    pub user_id: Option<String>,
    pub generator_model_id: i64,
    pub generator_temperature: f64,
    pub refiner_model_id: i64,
    pub refiner_temperature: f64,
    pub validator_model_id: i64,
    pub validator_temperature: f64,
    pub curator_model_id: i64,
    pub curator_temperature: f64,
    pub is_default: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Expert template manager
pub struct ExpertTemplateManager {
    db: DatabaseManager,
    templates: Vec<ExpertTemplate>,
}

impl ExpertTemplateManager {
    /// Create new template manager
    pub fn new(db: DatabaseManager) -> Self {
        let templates = Self::create_builtin_templates();

        Self { db, templates }
    }

    /// Get all available templates
    pub fn get_templates(&self) -> &Vec<ExpertTemplate> {
        &self.templates
    }

    /// Get template by ID
    pub fn get_template(&self, id: &str) -> Option<&ExpertTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    /// Get templates by category
    pub fn get_templates_by_category(&self, category: &ProfileCategory) -> Vec<&ExpertTemplate> {
        self.templates
            .iter()
            .filter(|t| &t.category == category)
            .collect()
    }

    /// Get recommended templates for user level
    pub fn get_recommendations(&self, expert_level: &ExpertLevel) -> Vec<&ExpertTemplate> {
        match expert_level {
            ExpertLevel::Beginner => vec![
                self.get_template("lightning-fast").unwrap(),
                self.get_template("budget-optimizer").unwrap(),
                self.get_template("startup-mvp").unwrap(),
            ],
            ExpertLevel::Intermediate => vec![
                self.get_template("startup-mvp").unwrap(),
                self.get_template("debugging-detective").unwrap(),
                self.get_template("research-deep-dive").unwrap(),
            ],
            ExpertLevel::Advanced => vec![
                self.get_template("precision-architect").unwrap(),
                self.get_template("creative-innovator").unwrap(),
                self.get_template("ml-ai-specialist").unwrap(),
            ],
            ExpertLevel::Expert => vec![
                self.get_template("enterprise-grade").unwrap(),
                self.get_template("security-focused").unwrap(),
                self.get_template("precision-architect").unwrap(),
            ],
        }
    }

    /// Create consensus profile from template
    pub async fn create_profile_from_template(
        &self,
        template_id: &str,
        profile_name: &str,
        user_id: Option<&str>,
    ) -> Result<ConsensusProfile> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| HiveError::Internal {
                context: "profiles".to_string(),
                message: format!("Template not found: {}", template_id),
            })?;

        // Resolve models for the profile
        let model_ids = self.resolve_models_for_template(template).await?;

        // Create the profile in database
        let conn = self.db.get_connection()?;
        let now = chrono::Utc::now().to_rfc3339();

        // Check if profile name already exists
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM consensus_profiles WHERE name = ?1",
                [profile_name],
                |row| row.get(0),
            )
            .optional()?;

        if existing.is_some() {
            return Err(HiveError::Internal {
                context: "profiles".to_string(),
                message: format!("Profile '{}' already exists", profile_name),
            });
        }

        // Insert new profile
        conn.execute(
            "INSERT INTO consensus_profiles
             (name, user_id, generator_model_id, generator_temperature,
              refiner_model_id, refiner_temperature, validator_model_id, validator_temperature,
              curator_model_id, curator_temperature, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                profile_name,
                user_id,
                model_ids.generator,
                template.temperatures.generator,
                model_ids.refiner,
                template.temperatures.refiner,
                model_ids.validator,
                template.temperatures.validator,
                model_ids.curator,
                template.temperatures.curator,
                false, // is_default - will be set separately if needed
                now,
                now
            ],
        )?;

        let profile_id = conn.last_insert_rowid();

        // Set as default if this is the first profile
        let profile_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM consensus_profiles", [], |row| {
                row.get(0)
            })?;

        let is_default = profile_count == 1;
        if is_default {
            conn.execute(
                "UPDATE consensus_profiles SET is_default = 1 WHERE id = ?1",
                [profile_id],
            )?;
        }

        Ok(ConsensusProfile {
            id: profile_id,
            name: profile_name.to_string(),
            user_id: user_id.map(|s| s.to_string()),
            generator_model_id: model_ids.generator,
            generator_temperature: template.temperatures.generator,
            refiner_model_id: model_ids.refiner,
            refiner_temperature: template.temperatures.refiner,
            validator_model_id: model_ids.validator,
            validator_temperature: template.temperatures.validator,
            curator_model_id: model_ids.curator,
            curator_temperature: template.temperatures.curator,
            is_default,
            created_at: chrono::DateTime::parse_from_rfc3339(&now)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&now)
                .unwrap()
                .with_timezone(&chrono::Utc),
        })
    }

    /// Resolve models for template based on strategy
    async fn resolve_models_for_template(&self, template: &ExpertTemplate) -> Result<FixedModels> {
        match template.selection_strategy {
            SelectionStrategy::Fixed => {
                template
                    .fixed_models
                    .clone()
                    .ok_or_else(|| HiveError::ConfigInvalid {
                        message: "Fixed strategy requires fixed_models".to_string(),
                    })
            }
            SelectionStrategy::Dynamic => {
                let criteria = template.selection_criteria.as_ref().ok_or_else(|| {
                    HiveError::ConfigInvalid {
                        message: "Dynamic strategy requires selection_criteria".to_string(),
                    }
                })?;

                self.select_models_from_criteria(criteria).await
            }
            SelectionStrategy::Hybrid => {
                // Try dynamic first, fall back to fixed
                if let Some(criteria) = &template.selection_criteria {
                    match self.select_models_from_criteria(criteria).await {
                        Ok(models) => Ok(models),
                        Err(_) => {
                            template
                                .fixed_models
                                .clone()
                                .ok_or_else(|| HiveError::ConfigInvalid {
                                    message: "Hybrid strategy requires fixed_models as fallback"
                                        .to_string(),
                                })
                        }
                    }
                } else {
                    template
                        .fixed_models
                        .clone()
                        .ok_or_else(|| HiveError::ConfigInvalid {
                            message:
                                "Hybrid strategy requires either selection_criteria or fixed_models"
                                    .to_string(),
                        })
                }
            }
        }
    }

    /// Select models based on criteria
    async fn select_models_from_criteria(
        &self,
        criteria: &StageSelectionCriteria,
    ) -> Result<FixedModels> {
        Ok(FixedModels {
            generator: self
                .select_model_for_stage(&criteria.generator, "generator")
                .await?,
            refiner: self
                .select_model_for_stage(&criteria.refiner, "refiner")
                .await?,
            validator: self
                .select_model_for_stage(&criteria.validator, "validator")
                .await?,
            curator: self
                .select_model_for_stage(&criteria.curator, "curator")
                .await?,
        })
    }

    /// Select a single model for a stage based on criteria
    async fn select_model_for_stage(&self, criteria: &ModelCriteria, stage: &str) -> Result<i64> {
        let conn = self.db.get_connection()?;

        // Build SQL query based on criteria
        let mut query = "SELECT internal_id FROM openrouter_models WHERE is_active = 1".to_string();
        let mut params = Vec::new();

        // Add provider filter
        if let Some(providers) = &criteria.providers {
            let placeholders: Vec<&str> = providers.iter().map(|_| "?").collect();
            query.push_str(&format!(
                " AND provider_name IN ({})",
                placeholders.join(",")
            ));
            for provider in providers {
                params.push(provider.as_str());
            }
        }

        // Add context window filter
        let mut min_str = String::new();
        let mut max_str = String::new();
        if let Some(context_window) = &criteria.context_window {
            let (min, max) = match context_window.as_str() {
                "small" => (0, 8000),
                "medium" => (8000, 32000),
                "large" => (32000, 128000),
                "xl" => (128000, 999999999),
                _ => (0, 999999999),
            };
            query.push_str(" AND context_window >= ? AND context_window < ?");
            min_str = min.to_string();
            max_str = max.to_string();
            params.push(&min_str);
            params.push(&max_str);
        }

        // Add cost range filter
        let mut min_cost_str = String::new();
        let mut max_cost_str = String::new();
        if let Some(cost_range) = &criteria.cost_range {
            let (min_cost, max_cost) = match cost_range {
                CostRange::UltraLow => (0.0, 0.000001),
                CostRange::Low => (0.000001, 0.00001),
                CostRange::Medium => (0.00001, 0.0001),
                CostRange::High => (0.0001, 0.001),
                CostRange::Premium => (0.001, 999.0),
            };
            query.push_str(" AND (pricing_input + pricing_output) >= ? AND (pricing_input + pricing_output) < ?");
            min_cost_str = min_cost.to_string();
            max_cost_str = max_cost.to_string();
            params.push(&min_cost_str);
            params.push(&max_cost_str);
        }

        // Add ordering based on ranking position or cost
        if let Some(ranking_position) = &criteria.ranking_position {
            match ranking_position.as_str() {
                "top-5" | "top-10" | "top-15" | "top-20" => {
                    query.push_str(" ORDER BY COALESCE((SELECT rank_position FROM model_rankings WHERE model_internal_id = openrouter_models.internal_id), 999)");
                }
                "cost-efficient" => {
                    query.push_str(" ORDER BY (pricing_input + pricing_output)");
                }
                _ => {
                    query.push_str(" ORDER BY internal_id");
                }
            }
        } else {
            query.push_str(" ORDER BY internal_id");
        }

        query.push_str(" LIMIT 1");

        // Execute query
        let stmt_params: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        let result: Option<i64> = conn
            .query_row(&query, stmt_params.as_slice(), |row| row.get(0))
            .optional()?;

        match result {
            Some(internal_id) => Ok(internal_id),
            None => {
                // Try fallback model if specified
                if let Some(fallback) = &criteria.fallback_model {
                    self.resolve_fallback_model(fallback).await
                } else {
                    // Use any available model for this stage
                    self.get_any_available_model(stage).await
                }
            }
        }
    }

    /// Resolve fallback model from OpenRouter ID or internal ID
    async fn resolve_fallback_model(&self, fallback: &str) -> Result<i64> {
        let conn = self.db.get_connection()?;

        // Try as internal ID first
        if let Ok(internal_id) = fallback.parse::<i64>() {
            let exists: Option<i64> = conn.query_row(
                "SELECT internal_id FROM openrouter_models WHERE internal_id = ? AND is_active = 1",
                [internal_id],
                |row| row.get(0),
            ).optional()?;

            if exists.is_some() {
                return Ok(internal_id);
            }
        }

        // Try as OpenRouter ID
        let result: Option<i64> = conn.query_row(
            "SELECT internal_id FROM openrouter_models WHERE openrouter_id = ? AND is_active = 1",
            [fallback],
            |row| row.get(0),
        ).optional()?;

        result.ok_or_else(|| HiveError::Internal {
            context: "profiles".to_string(),
            message: format!("Fallback model not found: {}", fallback),
        })
    }

    /// Get any available model for emergency fallback
    async fn get_any_available_model(&self, stage: &str) -> Result<i64> {
        let conn = self.db.get_connection()?;

        // Stage-specific fallback preferences
        let query = match stage {
            "generator" => "SELECT internal_id FROM openrouter_models WHERE is_active = 1 ORDER BY context_window DESC LIMIT 1",
            "validator" => "SELECT internal_id FROM openrouter_models WHERE is_active = 1 ORDER BY (pricing_input + pricing_output) ASC LIMIT 1",
            _ => "SELECT internal_id FROM openrouter_models WHERE is_active = 1 ORDER BY internal_id LIMIT 1",
        };

        let result: Option<i64> = conn.query_row(query, [], |row| row.get(0)).optional()?;

        result.ok_or_else(|| HiveError::Internal {
            context: "profiles".to_string(),
            message: format!("No models available for stage: {}", stage),
        })
    }

    /// Get all consensus profiles
    pub async fn get_all_profiles(&self) -> Result<Vec<ConsensusProfile>> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, user_id, generator_model_id, generator_temperature,
                    refiner_model_id, refiner_temperature, validator_model_id, validator_temperature,
                    curator_model_id, curator_temperature, is_default, created_at, updated_at
             FROM consensus_profiles
             ORDER BY is_default DESC, created_at DESC"
        )?;

        let profile_iter = stmt.query_map([], |row| {
            let created_at_str: String = row.get(12)?;
            let updated_at_str: String = row.get(13)?;

            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        12,
                        "created_at".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?
                .with_timezone(&chrono::Utc);

            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        13,
                        "updated_at".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?
                .with_timezone(&chrono::Utc);

            Ok(ConsensusProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                user_id: row.get(2)?,
                generator_model_id: row.get(3)?,
                generator_temperature: row.get(4)?,
                refiner_model_id: row.get(5)?,
                refiner_temperature: row.get(6)?,
                validator_model_id: row.get(7)?,
                validator_temperature: row.get(8)?,
                curator_model_id: row.get(9)?,
                curator_temperature: row.get(10)?,
                is_default: row.get(11)?,
                created_at,
                updated_at,
            })
        })?;

        let mut profiles = Vec::new();
        for profile in profile_iter {
            profiles.push(profile?);
        }

        Ok(profiles)
    }

    /// Create all expert templates
    pub async fn create_all_expert_templates(&self, user_id: Option<&str>) -> Result<Vec<String>> {
        let mut created_profiles = Vec::new();

        for template in &self.templates {
            let profile_name = format!("{} Profile", template.name);

            match self
                .create_profile_from_template(&template.id, &profile_name, user_id)
                .await
            {
                Ok(_) => {
                    created_profiles.push(profile_name);
                }
                Err(HiveError::Internal { context, message })
                    if message.contains("already exists") =>
                {
                    // Profile already exists, skip
                    continue;
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to create profile for template {}: {}",
                        template.name, e
                    );
                    continue;
                }
            }
        }

        Ok(created_profiles)
    }

    /// Create built-in expert templates
    fn create_builtin_templates() -> Vec<ExpertTemplate> {
        vec![
            // Lightning Fast - Speed optimized
            ExpertTemplate {
                id: "lightning-fast".to_string(),
                name: "Lightning Fast".to_string(),
                description: "Ultra-high-speed consensus optimized for rapid prototyping and quick answers".to_string(),
                category: ProfileCategory::Speed,
                expert_level: ExpertLevel::Beginner,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string(), "openai".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Low),
                        fallback_model: Some("anthropic/claude-3-haiku".to_string()),
                    },
                    refiner: ModelCriteria {
                        providers: None,
                        ranking_position: Some("cost-efficient".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::UltraLow),
                        fallback_model: Some("openai/gpt-3.5-turbo".to_string()),
                    },
                    validator: ModelCriteria {
                        providers: Some(vec!["google".to_string()]),
                        ranking_position: Some("cost-efficient".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::UltraLow),
                        fallback_model: Some("google/gemini-flash".to_string()),
                    },
                    curator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Low),
                        fallback_model: Some("anthropic/claude-3-haiku".to_string()),
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.3,
                    refiner: 0.2,
                    validator: 0.1,
                    curator: 0.3,
                },
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Cost,
                    max_cost_per_conversation: Some(0.05),
                    preferred_cost_range: CostRange::UltraLow,
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Speed,
                    max_latency_ms: Some(2000),
                    quality_threshold: Some(0.7),
                }),
                use_cases: vec![
                    "Quick prototyping".to_string(),
                    "Simple questions".to_string(),
                    "Learning".to_string(),
                    "Debugging small issues".to_string(),
                ],
                tags: vec!["speed".to_string(), "budget-friendly".to_string(), "beginner".to_string()],
            },

            // Precision Architect - Quality focused
            ExpertTemplate {
                id: "precision-architect".to_string(),
                name: "Precision Architect".to_string(),
                description: "Maximum quality consensus for complex architectural decisions and critical code".to_string(),
                category: ProfileCategory::Quality,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        capabilities: None,
                        cost_range: Some(CostRange::Premium),
                        fallback_model: Some("anthropic/claude-3-opus".to_string()),
                    },
                    refiner: ModelCriteria {
                        providers: Some(vec!["openai".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("large".to_string()),
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("openai/gpt-4".to_string()),
                    },
                    validator: ModelCriteria {
                        providers: Some(vec!["google".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Medium),
                        fallback_model: Some("google/gemini-pro".to_string()),
                    },
                    curator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        capabilities: None,
                        cost_range: Some(CostRange::Premium),
                        fallback_model: Some("anthropic/claude-3-opus".to_string()),
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.7,
                    refiner: 0.5,
                    validator: 0.3,
                    curator: 0.6,
                },
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: Some(1.00),
                    preferred_cost_range: CostRange::Premium,
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.95),
                }),
                use_cases: vec![
                    "System architecture".to_string(),
                    "Complex algorithms".to_string(),
                    "Production code review".to_string(),
                    "Security analysis".to_string(),
                ],
                tags: vec!["quality".to_string(), "expert".to_string(), "architecture".to_string(), "premium".to_string()],
            },

            // Budget Optimizer - Cost efficient
            ExpertTemplate {
                id: "budget-optimizer".to_string(),
                name: "Budget Optimizer".to_string(),
                description: "Cost-efficient consensus that maximizes value while minimizing expenses".to_string(),
                category: ProfileCategory::Cost,
                expert_level: ExpertLevel::Intermediate,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        providers: Some(vec!["meta-llama".to_string(), "mistralai".to_string()]),
                        ranking_position: Some("cost-efficient".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Low),
                        fallback_model: Some("meta-llama/llama-3-8b-instruct".to_string()),
                    },
                    refiner: ModelCriteria {
                        providers: Some(vec!["google".to_string()]),
                        ranking_position: Some("cost-efficient".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::UltraLow),
                        fallback_model: Some("google/gemini-flash".to_string()),
                    },
                    validator: ModelCriteria {
                        providers: None,
                        ranking_position: Some("cost-efficient".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::UltraLow),
                        fallback_model: Some("google/gemini-flash".to_string()),
                    },
                    curator: ModelCriteria {
                        providers: Some(vec!["mistralai".to_string()]),
                        ranking_position: Some("cost-efficient".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Low),
                        fallback_model: Some("mistralai/mistral-7b-instruct".to_string()),
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.4,
                    refiner: 0.3,
                    validator: 0.2,
                    curator: 0.4,
                },
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Cost,
                    max_cost_per_conversation: Some(0.10),
                    preferred_cost_range: CostRange::Low,
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Balanced,
                    max_latency_ms: None,
                    quality_threshold: Some(0.8),
                }),
                use_cases: vec![
                    "Learning projects".to_string(),
                    "Personal development".to_string(),
                    "Small team projects".to_string(),
                    "Budget-conscious development".to_string(),
                ],
                tags: vec!["cost-effective".to_string(), "budget".to_string(), "efficient".to_string()],
            },

            // Research Deep Dive - Specialized research
            ExpertTemplate {
                id: "research-deep-dive".to_string(),
                name: "Research Deep Dive".to_string(),
                description: "Comprehensive analysis for research, documentation, and knowledge discovery".to_string(),
                category: ProfileCategory::Specialized,
                expert_level: ExpertLevel::Advanced,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: Some("xl".to_string()),
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("anthropic/claude-3-sonnet".to_string()),
                    },
                    refiner: ModelCriteria {
                        providers: Some(vec!["openai".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: Some("large".to_string()),
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("openai/gpt-4-turbo".to_string()),
                    },
                    validator: ModelCriteria {
                        providers: Some(vec!["google".to_string()]),
                        ranking_position: Some("top-15".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Medium),
                        fallback_model: Some("google/gemini-pro".to_string()),
                    },
                    curator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("anthropic/claude-3-opus".to_string()),
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.8,
                    refiner: 0.6,
                    validator: 0.4,
                    curator: 0.7,
                },
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: None,
                    preferred_cost_range: CostRange::High,
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.9),
                }),
                use_cases: vec![
                    "Technical research".to_string(),
                    "Market analysis".to_string(),
                    "Academic writing".to_string(),
                    "Documentation".to_string(),
                ],
                tags: vec!["research".to_string(), "analysis".to_string(), "comprehensive".to_string()],
            },

            // Startup MVP - Balanced production
            ExpertTemplate {
                id: "startup-mvp".to_string(),
                name: "Startup MVP".to_string(),
                description: "Balanced consensus for MVP development with good quality and reasonable cost".to_string(),
                category: ProfileCategory::Production,
                expert_level: ExpertLevel::Intermediate,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Medium),
                        fallback_model: Some("anthropic/claude-3-sonnet".to_string()),
                    },
                    refiner: ModelCriteria {
                        providers: Some(vec!["openai".to_string()]),
                        ranking_position: Some("top-15".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Low),
                        fallback_model: Some("openai/gpt-3.5-turbo".to_string()),
                    },
                    validator: ModelCriteria {
                        providers: Some(vec!["google".to_string()]),
                        ranking_position: Some("cost-efficient".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Low),
                        fallback_model: Some("google/gemini-flash".to_string()),
                    },
                    curator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Medium),
                        fallback_model: Some("anthropic/claude-3-sonnet".to_string()),
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.6,
                    refiner: 0.4,
                    validator: 0.3,
                    curator: 0.5,
                },
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Balanced,
                    max_cost_per_conversation: Some(0.25),
                    preferred_cost_range: CostRange::Medium,
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Balanced,
                    max_latency_ms: None,
                    quality_threshold: Some(0.85),
                }),
                use_cases: vec![
                    "MVP development".to_string(),
                    "Startup projects".to_string(),
                    "Feature prototyping".to_string(),
                    "Small team development".to_string(),
                ],
                tags: vec!["startup".to_string(), "mvp".to_string(), "balanced".to_string()],
            },

            // Enterprise Grade - Production enterprise
            ExpertTemplate {
                id: "enterprise-grade".to_string(),
                name: "Enterprise Grade".to_string(),
                description: "Production-ready consensus with enterprise security, reliability, and compliance".to_string(),
                category: ProfileCategory::Production,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Hybrid,
                fixed_models: Some(FixedModels {
                    generator: 1,  // anthropic/claude-3.5-sonnet
                    refiner: 2,    // openai/gpt-4o
                    validator: 5,  // google/gemini-pro-1.5
                    curator: 6,    // anthropic/claude-3-opus
                }),
                selection_criteria: None,
                temperatures: StageTemperatures {
                    generator: 0.5,
                    refiner: 0.3,
                    validator: 0.2,
                    curator: 0.4,
                },
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: None,
                    preferred_cost_range: CostRange::Premium,
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.95),
                }),
                use_cases: vec![
                    "Enterprise applications".to_string(),
                    "Mission-critical systems".to_string(),
                    "Financial services".to_string(),
                    "Healthcare systems".to_string(),
                ],
                tags: vec!["enterprise".to_string(), "production".to_string(), "reliable".to_string()],
            },

            // Creative Innovator - High creativity
            ExpertTemplate {
                id: "creative-innovator".to_string(),
                name: "Creative Innovator".to_string(),
                description: "High-creativity consensus for innovative solutions and creative problem solving".to_string(),
                category: ProfileCategory::Specialized,
                expert_level: ExpertLevel::Advanced,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("anthropic/claude-3-opus".to_string()),
                    },
                    refiner: ModelCriteria {
                        providers: Some(vec!["openai".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("openai/gpt-4".to_string()),
                    },
                    validator: ModelCriteria {
                        providers: Some(vec!["google".to_string()]),
                        ranking_position: Some("top-15".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Medium),
                        fallback_model: Some("google/gemini-pro".to_string()),
                    },
                    curator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("anthropic/claude-3-opus".to_string()),
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.9,
                    refiner: 0.7,
                    validator: 0.5,
                    curator: 0.8,
                },
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: None,
                    preferred_cost_range: CostRange::High,
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.85),
                }),
                use_cases: vec![
                    "Creative coding".to_string(),
                    "Innovative solutions".to_string(),
                    "Brainstorming".to_string(),
                    "Novel algorithms".to_string(),
                ],
                tags: vec!["creative".to_string(), "innovative".to_string(), "experimental".to_string()],
            },

            // Security Focused - Security specialist
            ExpertTemplate {
                id: "security-focused".to_string(),
                name: "Security Focused".to_string(),
                description: "Security-first consensus optimized for secure coding and vulnerability analysis".to_string(),
                category: ProfileCategory::Specialized,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("anthropic/claude-3-opus".to_string()),
                    },
                    refiner: ModelCriteria {
                        providers: Some(vec!["openai".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("openai/gpt-4".to_string()),
                    },
                    validator: ModelCriteria {
                        providers: Some(vec!["google".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Medium),
                        fallback_model: Some("google/gemini-pro".to_string()),
                    },
                    curator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("anthropic/claude-3-opus".to_string()),
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.4,
                    refiner: 0.3,
                    validator: 0.2,
                    curator: 0.3,
                },
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: None,
                    preferred_cost_range: CostRange::High,
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.95),
                }),
                use_cases: vec![
                    "Security audits".to_string(),
                    "Vulnerability analysis".to_string(),
                    "Secure coding".to_string(),
                    "Compliance reviews".to_string(),
                ],
                tags: vec!["security".to_string(), "analysis".to_string(), "compliance".to_string()],
            },

            // ML/AI Specialist - ML and AI focused
            ExpertTemplate {
                id: "ml-ai-specialist".to_string(),
                name: "ML/AI Specialist".to_string(),
                description: "Specialized consensus for machine learning and AI development projects".to_string(),
                category: ProfileCategory::Specialized,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("anthropic/claude-3-opus".to_string()),
                    },
                    refiner: ModelCriteria {
                        providers: Some(vec!["openai".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("openai/gpt-4".to_string()),
                    },
                    validator: ModelCriteria {
                        providers: Some(vec!["google".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Medium),
                        fallback_model: Some("google/gemini-pro".to_string()),
                    },
                    curator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-5".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::High),
                        fallback_model: Some("anthropic/claude-3-opus".to_string()),
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.6,
                    refiner: 0.4,
                    validator: 0.3,
                    curator: 0.5,
                },
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: None,
                    preferred_cost_range: CostRange::High,
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.9),
                }),
                use_cases: vec![
                    "ML model development".to_string(),
                    "AI system design".to_string(),
                    "Data science".to_string(),
                    "Neural networks".to_string(),
                ],
                tags: vec!["ml".to_string(), "ai".to_string(), "data-science".to_string()],
            },

            // Debugging Detective - Debug specialist
            ExpertTemplate {
                id: "debugging-detective".to_string(),
                name: "Debugging Detective".to_string(),
                description: "Methodical consensus optimized for debugging, troubleshooting, and error analysis".to_string(),
                category: ProfileCategory::Specialized,
                expert_level: ExpertLevel::Intermediate,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Medium),
                        fallback_model: Some("anthropic/claude-3-sonnet".to_string()),
                    },
                    refiner: ModelCriteria {
                        providers: Some(vec!["openai".to_string()]),
                        ranking_position: Some("top-15".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Medium),
                        fallback_model: Some("openai/gpt-3.5-turbo".to_string()),
                    },
                    validator: ModelCriteria {
                        providers: Some(vec!["google".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Low),
                        fallback_model: Some("google/gemini-pro".to_string()),
                    },
                    curator: ModelCriteria {
                        providers: Some(vec!["anthropic".to_string()]),
                        ranking_position: Some("top-10".to_string()),
                        context_window: None,
                        capabilities: None,
                        cost_range: Some(CostRange::Medium),
                        fallback_model: Some("anthropic/claude-3-sonnet".to_string()),
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.3,
                    refiner: 0.2,
                    validator: 0.1,
                    curator: 0.3,
                },
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Balanced,
                    max_cost_per_conversation: None,
                    preferred_cost_range: CostRange::Medium,
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.9),
                }),
                use_cases: vec![
                    "Bug debugging".to_string(),
                    "Error analysis".to_string(),
                    "Code troubleshooting".to_string(),
                    "System diagnostics".to_string(),
                ],
                tags: vec!["debugging".to_string(), "troubleshooting".to_string(), "methodical".to_string()],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_template_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db = DatabaseManager::new(temp_dir.path().join("test.db")).unwrap();
        let manager = ExpertTemplateManager::new(db);

        assert!(!manager.get_templates().is_empty());
        assert!(manager.get_template("lightning-fast").is_some());
        assert!(manager.get_template("precision-architect").is_some());
        assert!(manager.get_template("budget-optimizer").is_some());
    }

    #[test]
    fn test_recommendations() {
        let temp_dir = TempDir::new().unwrap();
        let db = DatabaseManager::new(temp_dir.path().join("test.db")).unwrap();
        let manager = ExpertTemplateManager::new(db);

        let beginner_recs = manager.get_recommendations(&ExpertLevel::Beginner);
        assert_eq!(beginner_recs.len(), 3);
        assert!(beginner_recs.iter().any(|t| t.id == "lightning-fast"));

        let expert_recs = manager.get_recommendations(&ExpertLevel::Expert);
        assert_eq!(expert_recs.len(), 3);
        assert!(expert_recs.iter().any(|t| t.id == "precision-architect"));
    }
}
