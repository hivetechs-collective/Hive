// Model Management System - OpenRouter Integration and Model Rankings
// Port of TypeScript dynamic-model-selector.ts and openrouter-rankings.ts

use crate::core::database_simple::Database;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub internal_id: u32,
    pub openrouter_id: String,
    pub provider_name: String,
    pub model_name: String,
    pub context_window: u32,
    pub pricing_input: Option<f64>,
    pub pricing_output: Option<f64>,
    pub capabilities: Vec<String>,
    pub is_active: bool,
    pub last_updated: DateTime<Utc>,
    pub performance_metrics: Option<PerformanceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub success_rate: f64,
    pub average_latency_ms: u32,
    pub quality_score: f64,
    pub cost_efficiency: f64,
    pub recent_usage_count: u32,
    pub last_evaluation: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRanking {
    pub model_internal_id: u32,
    pub ranking_source: String,
    pub rank_position: u32,
    pub score: f64,
    pub category: String,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub created: Option<u64>,
    pub description: Option<String>,
    pub context_length: Option<u32>,
    pub architecture: Option<Architecture>,
    pub pricing: Option<Pricing>,
    pub top_provider: Option<TopProvider>,
    pub per_request_limits: Option<PerRequestLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Architecture {
    pub modality: Option<String>,
    pub tokenizer: Option<String>,
    pub instruct_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricing {
    pub prompt: Option<String>,
    pub completion: Option<String>,
    pub image: Option<String>,
    pub request: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopProvider {
    pub max_completion_tokens: Option<u32>,
    pub is_moderated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerRequestLimits {
    pub prompt_tokens: Option<String>,
    pub completion_tokens: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCandidate {
    pub internal_id: u32,
    pub openrouter_id: String,
    pub provider: String,
    pub rank_position: Option<u32>,
    pub relative_score: Option<f64>,
    pub estimated_cost: f64,
    pub estimated_latency: u32,
    pub success_rate: f64,
    pub features: Vec<String>,
    pub context_window: u32,
    pub suitability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectionCriteria {
    pub stage: String,
    pub question_complexity: String,
    pub question_category: String,
    pub budget_constraints: Option<BudgetConstraints>,
    pub performance_targets: Option<PerformanceTargets>,
    pub user_preferences: Option<UserPreferences>,
    pub profile_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConstraints {
    pub max_cost_per_stage: Option<f64>,
    pub max_total_cost: Option<f64>,
    pub cost_efficiency_target: Option<f64>,
    pub prioritize_cost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    pub max_latency: Option<u32>,
    pub min_success_rate: Option<f64>,
    pub prioritize_speed: bool,
    pub prioritize_quality: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_providers: Vec<String>,
    pub avoided_providers: Vec<String>,
    pub preferred_features: Vec<String>,
    pub model_blacklist: Vec<String>,
}

pub struct ModelManager {
    client: Client,
    openrouter_api_key: Option<String>,
    cache_duration: Duration,
    last_sync: Option<Instant>,
}

impl ModelManager {
    pub fn new(openrouter_api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            openrouter_api_key,
            cache_duration: Duration::from_secs(3600), // 1 hour cache
            last_sync: None,
        }
    }

    /// Sync models from OpenRouter API
    pub async fn sync_models(&mut self, db: &Database) -> Result<u32> {
        let api_key = self.openrouter_api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenRouter API key not configured"))?;

        println!("🔄 Syncing models from OpenRouter API...");

        // Fetch models from OpenRouter
        let response = self.client
            .get("https://openrouter.ai/api/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://hive.ai")
            .header("X-Title", "Hive AI")
            .send()
            .await
            .context("Failed to fetch models from OpenRouter")?;

        if !response.status().is_success() {
            anyhow::bail!("OpenRouter API error: {}", response.status());
        }

        let models_response: serde_json::Value = response.json().await
            .context("Failed to parse OpenRouter response")?;

        let models: Vec<OpenRouterModel> = serde_json::from_value(
            models_response["data"].clone()
        ).context("Failed to deserialize models")?;

        println!("📥 Fetched {} models from OpenRouter", models.len());

        // Process and store models
        let conn = db.get_connection().await?;
        let mut stored_count = 0;

        conn.execute("BEGIN TRANSACTION", [])?;

        for model in models {
            // Parse pricing
            let pricing_input = model.pricing.as_ref()
                .and_then(|p| p.prompt.as_ref())
                .and_then(|s| s.parse::<f64>().ok());
            
            let pricing_output = model.pricing.as_ref()
                .and_then(|p| p.completion.as_ref())
                .and_then(|s| s.parse::<f64>().ok());

            // Determine capabilities
            let mut capabilities = Vec::new();
            if let Some(arch) = &model.architecture {
                if let Some(modality) = &arch.modality {
                    if modality.contains("text") {
                        capabilities.push("text".to_string());
                    }
                    if modality.contains("image") {
                        capabilities.push("vision".to_string());
                    }
                }
                if let Some(instruct_type) = &arch.instruct_type {
                    capabilities.push(instruct_type.clone());
                }
            }

            // Extract provider name
            let provider_name = model.id.split('/').next().unwrap_or("unknown").to_string();

            // Insert or update model
            conn.execute(
                r#"
                INSERT OR REPLACE INTO openrouter_models (
                    openrouter_id, provider_name, model_name, context_window,
                    pricing_input, pricing_output, capabilities_json, is_active,
                    last_updated
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                rusqlite::params![
                    model.id,
                    provider_name,
                    model.name.clone(),
                    model.context_length.unwrap_or(0) as i64,
                    pricing_input,
                    pricing_output,
                    serde_json::to_string(&capabilities).unwrap_or_default(),
                    true,
                    Utc::now().to_rfc3339(),
                ],
            )?;

            stored_count += 1;
        }

        conn.execute("COMMIT", [])?;

        // Update rankings
        self.update_model_rankings(db).await?;

        self.last_sync = Some(Instant::now());
        println!("✅ Stored {} models in database", stored_count);

        Ok(stored_count)
    }

    /// Update model rankings based on performance data
    pub async fn update_model_rankings(&self, db: &Database) -> Result<()> {
        println!("📊 Updating model rankings...");

        let conn = db.get_connection().await?;

        // Get all active models with performance data
        let mut stmt = conn.prepare(
            r#"
            SELECT internal_id, openrouter_id, provider_name, 
                   pricing_input, pricing_output, context_window
            FROM openrouter_models 
            WHERE is_active = 1
            ORDER BY internal_id
            "#
        )?;

        let model_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, u32>(0)?,  // internal_id
                row.get::<_, String>(1)?, // openrouter_id
                row.get::<_, String>(2)?, // provider_name
                row.get::<_, Option<f64>>(3)?, // pricing_input
                row.get::<_, Option<f64>>(4)?, // pricing_output
                row.get::<_, u32>(5)?,  // context_window
            ))
        })?;

        // Calculate rankings for different categories
        let mut programming_ranks = Vec::new();
        let mut cost_efficient_ranks = Vec::new();
        let mut performance_ranks = Vec::new();

        for model_result in model_rows {
            let (internal_id, openrouter_id, provider_name, pricing_input, pricing_output, context_window) = model_result?;

            // Calculate programming rank (based on provider reputation and context window)
            let programming_score = self.calculate_programming_score(&provider_name, context_window, pricing_input);
            programming_ranks.push((internal_id, programming_score));

            // Calculate cost efficiency rank
            let cost_score = self.calculate_cost_efficiency_score(pricing_input, pricing_output);
            cost_efficient_ranks.push((internal_id, cost_score));

            // Calculate performance rank (context window weighted)
            let performance_score = self.calculate_performance_score(&provider_name, context_window);
            performance_ranks.push((internal_id, performance_score));
        }

        // Sort rankings
        programming_ranks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        cost_efficient_ranks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        performance_ranks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Store rankings
        conn.execute("DELETE FROM model_rankings", [])?;

        // Programming rankings
        for (rank, (internal_id, score)) in programming_ranks.iter().enumerate() {
            conn.execute(
                r#"
                INSERT INTO model_rankings (
                    model_internal_id, ranking_source, rank_position, 
                    score, category, last_updated
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                rusqlite::params![
                    internal_id,
                    "openrouter_programming_weekly",
                    (rank + 1) as u32,
                    score,
                    "programming",
                    Utc::now().to_rfc3339(),
                ],
            )?;
        }

        // Cost efficiency rankings
        for (rank, (internal_id, score)) in cost_efficient_ranks.iter().enumerate() {
            conn.execute(
                r#"
                INSERT INTO model_rankings (
                    model_internal_id, ranking_source, rank_position, 
                    score, category, last_updated
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                rusqlite::params![
                    internal_id,
                    "cost_efficiency",
                    (rank + 1) as u32,
                    score,
                    "cost",
                    Utc::now().to_rfc3339(),
                ],
            )?;
        }

        // Performance rankings
        for (rank, (internal_id, score)) in performance_ranks.iter().enumerate() {
            conn.execute(
                r#"
                INSERT INTO model_rankings (
                    model_internal_id, ranking_source, rank_position, 
                    score, category, last_updated
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                rusqlite::params![
                    internal_id,
                    "performance",
                    (rank + 1) as u32,
                    score,
                    "performance",
                    Utc::now().to_rfc3339(),
                ],
            )?;
        }

        println!("✅ Updated model rankings");
        Ok(())
    }

    /// Get models by ranking position
    pub async fn get_models_by_ranking(&self, db: &Database, category: &str, top_n: usize) -> Result<Vec<ModelInfo>> {
        let conn = db.get_connection().await?;

        let mut stmt = conn.prepare(
            r#"
            SELECT om.internal_id, om.openrouter_id, om.provider_name, om.model_name,
                   om.context_window, om.pricing_input, om.pricing_output, 
                   om.capabilities_json, om.is_active, om.last_updated,
                   mr.rank_position, mr.score
            FROM openrouter_models om
            JOIN model_rankings mr ON om.internal_id = mr.model_internal_id
            WHERE mr.category = ?1 AND om.is_active = 1
            ORDER BY mr.rank_position ASC
            LIMIT ?2
            "#
        )?;

        let model_rows = stmt.query_map(rusqlite::params![category, top_n], |row| {
            let capabilities_json: String = row.get(7)?;
            let capabilities: Vec<String> = serde_json::from_str(&capabilities_json).unwrap_or_default();
            
            let last_updated_str: String = row.get(9)?;
            let last_updated = DateTime::parse_from_rfc3339(&last_updated_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            Ok(ModelInfo {
                internal_id: row.get(0)?,
                openrouter_id: row.get(1)?,
                provider_name: row.get(2)?,
                model_name: row.get(3)?,
                context_window: row.get::<_, u32>(4)?,
                pricing_input: row.get(5)?,
                pricing_output: row.get(6)?,
                capabilities,
                is_active: row.get(8)?,
                last_updated,
                performance_metrics: None, // TODO: Add performance metrics
            })
        })?;

        let mut models = Vec::new();
        for model_result in model_rows {
            models.push(model_result?);
        }

        Ok(models)
    }

    /// Get model by internal ID
    pub async fn get_model_by_id(&self, db: &Database, internal_id: u32) -> Result<Option<ModelInfo>> {
        let conn = db.get_connection().await?;

        let mut stmt = conn.prepare(
            r#"
            SELECT internal_id, openrouter_id, provider_name, model_name,
                   context_window, pricing_input, pricing_output, 
                   capabilities_json, is_active, last_updated
            FROM openrouter_models
            WHERE internal_id = ?1
            "#
        )?;

        let result = stmt.query_row(rusqlite::params![internal_id], |row| {
            let capabilities_json: String = row.get(7)?;
            let capabilities: Vec<String> = serde_json::from_str(&capabilities_json).unwrap_or_default();
            
            let last_updated_str: String = row.get(9)?;
            let last_updated = DateTime::parse_from_rfc3339(&last_updated_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            Ok(ModelInfo {
                internal_id: row.get(0)?,
                openrouter_id: row.get(1)?,
                provider_name: row.get(2)?,
                model_name: row.get(3)?,
                context_window: row.get::<_, u32>(4)?,
                pricing_input: row.get(5)?,
                pricing_output: row.get(6)?,
                capabilities,
                is_active: row.get(8)?,
                last_updated,
                performance_metrics: None,
            })
        });

        match result {
            Ok(model) => Ok(Some(model)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get model by OpenRouter ID
    pub async fn get_model_by_openrouter_id(&self, db: &Database, openrouter_id: &str) -> Result<Option<ModelInfo>> {
        let conn = db.get_connection().await?;

        let mut stmt = conn.prepare(
            r#"
            SELECT internal_id, openrouter_id, provider_name, model_name,
                   context_window, pricing_input, pricing_output, 
                   capabilities_json, is_active, last_updated
            FROM openrouter_models
            WHERE openrouter_id = ?1 AND is_active = 1
            "#
        )?;

        let result = stmt.query_row(rusqlite::params![openrouter_id], |row| {
            let capabilities_json: String = row.get(7)?;
            let capabilities: Vec<String> = serde_json::from_str(&capabilities_json).unwrap_or_default();
            
            let last_updated_str: String = row.get(9)?;
            let last_updated = DateTime::parse_from_rfc3339(&last_updated_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            Ok(ModelInfo {
                internal_id: row.get(0)?,
                openrouter_id: row.get(1)?,
                provider_name: row.get(2)?,
                model_name: row.get(3)?,
                context_window: row.get::<_, u32>(4)?,
                pricing_input: row.get(5)?,
                pricing_output: row.get(6)?,
                capabilities,
                is_active: row.get(8)?,
                last_updated,
                performance_metrics: None,
            })
        });

        match result {
            Ok(model) => Ok(Some(model)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Check if sync is needed
    pub fn needs_sync(&self) -> bool {
        match self.last_sync {
            Some(last_sync) => last_sync.elapsed() > self.cache_duration,
            None => true,
        }
    }

    /// Calculate programming score for a model
    fn calculate_programming_score(&self, provider: &str, context_window: u32, pricing: Option<f64>) -> f64 {
        let mut score = 0.0;

        // Provider reputation scores
        match provider {
            "anthropic" => score += 95.0,
            "openai" => score += 90.0,
            "google" => score += 85.0,
            "meta-llama" => score += 80.0,
            "mistralai" | "mistral" => score += 75.0,
            "cohere" => score += 70.0,
            _ => score += 50.0,
        }

        // Context window bonus
        if context_window >= 200000 {
            score += 15.0;
        } else if context_window >= 128000 {
            score += 10.0;
        } else if context_window >= 32000 {
            score += 5.0;
        }

        // Cost efficiency bonus (lower cost = higher score)
        if let Some(price) = pricing {
            if price < 0.0001 {
                score += 10.0;
            } else if price < 0.001 {
                score += 5.0;
            }
        }

        score
    }

    /// Calculate cost efficiency score
    fn calculate_cost_efficiency_score(&self, input_pricing: Option<f64>, output_pricing: Option<f64>) -> f64 {
        let avg_pricing = match (input_pricing, output_pricing) {
            (Some(input), Some(output)) => (input + output) / 2.0,
            (Some(price), None) | (None, Some(price)) => price,
            (None, None) => return 0.0,
        };

        // Invert pricing so lower cost = higher score
        if avg_pricing > 0.0 {
            1.0 / avg_pricing
        } else {
            0.0
        }
    }

    /// Calculate performance score
    fn calculate_performance_score(&self, provider: &str, context_window: u32) -> f64 {
        let mut score = 0.0;

        // Provider performance scores
        match provider {
            "anthropic" => score += 90.0,
            "openai" => score += 85.0,
            "google" => score += 80.0,
            "meta-llama" => score += 75.0,
            "mistralai" | "mistral" => score += 70.0,
            _ => score += 50.0,
        }

        // Context window performance
        score += (context_window as f64 / 1000.0).min(50.0);

        score
    }
}

/// Dynamic Model Selector for stage-specific optimization
pub struct DynamicModelSelector {
    model_manager: ModelManager,
}

impl DynamicModelSelector {
    pub fn new(openrouter_api_key: Option<String>) -> Self {
        Self {
            model_manager: ModelManager::new(openrouter_api_key),
        }
    }

    /// Select optimal model for a specific stage and criteria
    pub async fn select_optimal_model(
        &self,
        db: &Database,
        criteria: &ModelSelectionCriteria,
        conversation_id: Option<&str>,
    ) -> Result<Option<ModelCandidate>> {
        println!("🎯 Selecting optimal model for {} stage ({} complexity)", 
                 criteria.stage, criteria.question_complexity);

        // 1. Get available models based on criteria
        let candidates = self.get_candidate_models(db, criteria).await?;

        if candidates.is_empty() {
            println!("⚠️ No candidate models found, falling back to defaults");
            return self.get_fallback_model(db, &criteria.stage).await;
        }

        // 2. Apply constraint filtering
        let filtered_candidates = self.apply_constraint_filtering(candidates, criteria);

        // 3. Score models based on suitability
        let scored_candidates = self.score_model_suitability(filtered_candidates, criteria);

        // 4. Select top model
        if let Some(selected_model) = scored_candidates.first() {
            println!("✅ Selected {} (score: {:.3}) for {}", 
                     selected_model.openrouter_id, selected_model.suitability_score, criteria.stage);

            // 5. Record selection for learning
            if let Some(conv_id) = conversation_id {
                self.record_model_selection(db, selected_model, criteria, conv_id).await?;
            }

            return Ok(Some(selected_model.clone()));
        }

        // Fallback to default
        self.get_fallback_model(db, &criteria.stage).await
    }

    /// Get candidate models based on criteria
    async fn get_candidate_models(&self, db: &Database, criteria: &ModelSelectionCriteria) -> Result<Vec<ModelCandidate>> {
        let mut candidates = Vec::new();

        // Get models based on stage requirements
        let models = match criteria.stage.as_str() {
            "generator" => {
                // For generator: prefer top-ranked models with good context
                self.model_manager.get_models_by_ranking(db, "programming", 20).await?
            }
            "refiner" => {
                // For refiner: balance between quality and cost
                let mut prog_models = self.model_manager.get_models_by_ranking(db, "programming", 15).await?;
                let cost_models = self.model_manager.get_models_by_ranking(db, "cost", 10).await?;
                prog_models.extend(cost_models);
                prog_models
            }
            "validator" => {
                // For validator: prefer cost-efficient models
                self.model_manager.get_models_by_ranking(db, "cost", 15).await?
            }
            "curator" => {
                // For curator: prefer top quality models
                self.model_manager.get_models_by_ranking(db, "programming", 10).await?
            }
            _ => {
                self.model_manager.get_models_by_ranking(db, "programming", 20).await?
            }
        };

        // Convert to candidates
        for model in models {
            let candidate = ModelCandidate {
                internal_id: model.internal_id,
                openrouter_id: model.openrouter_id.clone(),
                provider: model.provider_name.clone(),
                rank_position: None, // TODO: Get from rankings
                relative_score: None,
                estimated_cost: model.pricing_input.unwrap_or(0.001),
                estimated_latency: self.estimate_latency(&model),
                success_rate: 0.95, // Default
                features: model.capabilities.clone(),
                context_window: model.context_window,
                suitability_score: 0.0, // Will be calculated
            };
            candidates.push(candidate);
        }

        Ok(candidates)
    }

    /// Apply constraint filtering
    fn apply_constraint_filtering(&self, mut candidates: Vec<ModelCandidate>, criteria: &ModelSelectionCriteria) -> Vec<ModelCandidate> {
        // Budget constraints
        if let Some(budget) = &criteria.budget_constraints {
            if let Some(max_cost) = budget.max_cost_per_stage {
                candidates.retain(|c| c.estimated_cost <= max_cost);
            }
        }

        // Performance constraints
        if let Some(perf) = &criteria.performance_targets {
            if let Some(max_latency) = perf.max_latency {
                candidates.retain(|c| c.estimated_latency <= max_latency);
            }
            if let Some(min_success_rate) = perf.min_success_rate {
                candidates.retain(|c| c.success_rate >= min_success_rate);
            }
        }

        // User preferences
        if let Some(prefs) = &criteria.user_preferences {
            // Filter preferred providers
            if !prefs.preferred_providers.is_empty() {
                candidates.retain(|c| prefs.preferred_providers.contains(&c.provider));
            }
            
            // Filter avoided providers
            candidates.retain(|c| !prefs.avoided_providers.contains(&c.provider));
            
            // Filter blacklisted models
            candidates.retain(|c| !prefs.model_blacklist.contains(&c.openrouter_id));
        }

        candidates
    }

    /// Score model suitability
    fn score_model_suitability(&self, mut candidates: Vec<ModelCandidate>, criteria: &ModelSelectionCriteria) -> Vec<ModelCandidate> {
        for candidate in &mut candidates {
            let mut score = 0.0;

            // Base score from provider reputation
            match candidate.provider.as_str() {
                "anthropic" => score += 0.3,
                "openai" => score += 0.25,
                "google" => score += 0.2,
                "meta-llama" => score += 0.15,
                "mistralai" | "mistral" => score += 0.1,
                _ => score += 0.05,
            }

            // Stage-specific scoring
            match criteria.stage.as_str() {
                "generator" => {
                    // Prefer higher context window for generator
                    score += (candidate.context_window as f64 / 200000.0).min(0.2);
                }
                "refiner" => {
                    // Balance cost and quality for refiner
                    score += 0.1 / (candidate.estimated_cost + 0.001).max(0.001);
                }
                "validator" => {
                    // Prefer cost efficiency for validator
                    score += 0.2 / (candidate.estimated_cost + 0.001).max(0.001);
                }
                "curator" => {
                    // Prefer high quality for curator
                    if candidate.provider == "anthropic" || candidate.provider == "openai" {
                        score += 0.15;
                    }
                }
                _ => {}
            }

            // Budget constraints scoring
            if let Some(budget) = &criteria.budget_constraints {
                if budget.prioritize_cost {
                    score += 0.2 / (candidate.estimated_cost + 0.001).max(0.001);
                }
            }

            // Performance constraints scoring
            if let Some(perf) = &criteria.performance_targets {
                if perf.prioritize_speed {
                    score += 0.1 / (candidate.estimated_latency as f64 / 1000.0 + 0.1);
                }
                if perf.prioritize_quality {
                    score += candidate.success_rate * 0.1;
                }
            }

            candidate.suitability_score = score;
        }

        // Sort by suitability score (descending)
        candidates.sort_by(|a, b| b.suitability_score.partial_cmp(&a.suitability_score).unwrap_or(std::cmp::Ordering::Equal));

        candidates
    }

    /// Get fallback model for a stage
    async fn get_fallback_model(&self, db: &Database, stage: &str) -> Result<Option<ModelCandidate>> {
        // Try to get any active model for the stage
        let models = match stage {
            "generator" | "curator" => {
                self.model_manager.get_models_by_ranking(db, "programming", 1).await?
            }
            "refiner" | "validator" => {
                self.model_manager.get_models_by_ranking(db, "cost", 1).await?
            }
            _ => {
                self.model_manager.get_models_by_ranking(db, "programming", 1).await?
            }
        };

        if let Some(model) = models.first() {
            let candidate = ModelCandidate {
                internal_id: model.internal_id,
                openrouter_id: model.openrouter_id.clone(),
                provider: model.provider_name.clone(),
                rank_position: Some(1),
                relative_score: Some(1.0),
                estimated_cost: model.pricing_input.unwrap_or(0.001),
                estimated_latency: self.estimate_latency(model),
                success_rate: 0.9,
                features: model.capabilities.clone(),
                context_window: model.context_window,
                suitability_score: 0.8,
            };
            Ok(Some(candidate))
        } else {
            Ok(None)
        }
    }

    /// Estimate latency for a model
    fn estimate_latency(&self, model: &ModelInfo) -> u32 {
        // Basic estimation based on provider and model size
        match model.provider_name.as_str() {
            "anthropic" => 1500,
            "openai" => 1200,
            "google" => 1800,
            "meta-llama" => 2500,
            "mistralai" | "mistral" => 2000,
            _ => 3000,
        }
    }

    /// Record model selection for learning
    async fn record_model_selection(
        &self,
        db: &Database,
        model: &ModelCandidate,
        criteria: &ModelSelectionCriteria,
        conversation_id: &str,
    ) -> Result<()> {
        let conn = db.get_connection().await?;

        conn.execute(
            r#"
            INSERT INTO model_selections (
                conversation_id, stage, model_internal_id, openrouter_id,
                selection_criteria, suitability_score, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            rusqlite::params![
                conversation_id,
                criteria.stage,
                model.internal_id,
                model.openrouter_id,
                serde_json::to_string(criteria).unwrap_or_default(),
                model.suitability_score,
                Utc::now().to_rfc3339(),
            ],
        )?;

        Ok(())
    }
}