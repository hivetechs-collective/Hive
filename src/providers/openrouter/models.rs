/// Model Selection and Management
///
/// Intelligent model selection supporting 323+ models from OpenRouter
/// with task complexity analysis, cost optimization, and performance balancing.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::client::ModelInfo;

/// Task complexity levels for model selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskComplexity {
    /// Simple queries, basic questions
    Simple,
    /// Moderate complexity, general analysis
    Moderate,
    /// Complex tasks requiring reasoning
    Complex,
    /// Expert-level tasks requiring top models
    Expert,
}

/// Model selection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelSelectionStrategy {
    /// Optimize for lowest cost
    CostOptimized,
    /// Balance between cost and performance
    Balanced,
    /// Prioritize performance over cost
    PerformanceOptimized,
    /// Use only the best models regardless of cost
    QualityFirst,
}

/// Model capability categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelCapability {
    Programming,
    Reasoning,
    Creative,
    Multimodal,
    LongContext,
    FastResponse,
    Specialized,
}

/// Model tier for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ModelTier {
    /// Top-tier flagship models (Claude 3 Opus, GPT-4, etc.)
    Flagship,
    /// High-performance models (Claude 3 Sonnet, GPT-4 Turbo, etc.)
    Premium,
    /// Good balance models (Claude 3 Haiku, GPT-3.5, etc.)
    Standard,
    /// Budget-friendly models
    Economy,
}

/// Model metadata for selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub tier: ModelTier,
    pub capabilities: Vec<ModelCapability>,
    pub context_window: u32,
    pub cost_per_1k_input: f32,
    pub cost_per_1k_output: f32,
    pub average_latency_ms: u32,
    pub quality_score: f32, // 0.0 - 1.0
}

/// Model selection result
#[derive(Debug, Clone)]
pub struct ModelSelection {
    pub primary: String,
    pub fallbacks: Vec<String>,
    pub estimated_cost: f32,
    pub reasoning: String,
}

/// Model selector for intelligent routing
pub struct ModelSelector {
    models: HashMap<String, ModelMetadata>,
    strategy: ModelSelectionStrategy,
}

impl ModelSelector {
    /// Create a new model selector
    pub fn new(strategy: ModelSelectionStrategy) -> Self {
        let mut selector = Self {
            models: HashMap::new(),
            strategy,
        };

        // Initialize with known models
        selector.initialize_default_models();
        selector
    }

    /// Initialize default model metadata
    fn initialize_default_models(&mut self) {
        // Claude models
        self.add_model(ModelMetadata {
            id: "anthropic/claude-3-opus".to_string(),
            provider: "anthropic".to_string(),
            name: "Claude 3 Opus".to_string(),
            tier: ModelTier::Flagship,
            capabilities: vec![
                ModelCapability::Programming,
                ModelCapability::Reasoning,
                ModelCapability::Creative,
                ModelCapability::LongContext,
            ],
            context_window: 200000,
            cost_per_1k_input: 0.015,
            cost_per_1k_output: 0.075,
            average_latency_ms: 2500,
            quality_score: 0.95,
        });

        self.add_model(ModelMetadata {
            id: "anthropic/claude-3-sonnet".to_string(),
            provider: "anthropic".to_string(),
            name: "Claude 3 Sonnet".to_string(),
            tier: ModelTier::Premium,
            capabilities: vec![
                ModelCapability::Programming,
                ModelCapability::Reasoning,
                ModelCapability::Creative,
            ],
            context_window: 200000,
            cost_per_1k_input: 0.003,
            cost_per_1k_output: 0.015,
            average_latency_ms: 1500,
            quality_score: 0.85,
        });

        self.add_model(ModelMetadata {
            id: "anthropic/claude-3-haiku".to_string(),
            provider: "anthropic".to_string(),
            name: "Claude 3 Haiku".to_string(),
            tier: ModelTier::Standard,
            capabilities: vec![ModelCapability::Programming, ModelCapability::FastResponse],
            context_window: 200000,
            cost_per_1k_input: 0.00025,
            cost_per_1k_output: 0.00125,
            average_latency_ms: 800,
            quality_score: 0.75,
        });

        // OpenAI models
        self.add_model(ModelMetadata {
            id: "openai/gpt-4".to_string(),
            provider: "openai".to_string(),
            name: "GPT-4".to_string(),
            tier: ModelTier::Flagship,
            capabilities: vec![
                ModelCapability::Programming,
                ModelCapability::Reasoning,
                ModelCapability::Creative,
            ],
            context_window: 8192,
            cost_per_1k_input: 0.03,
            cost_per_1k_output: 0.06,
            average_latency_ms: 3000,
            quality_score: 0.93,
        });

        self.add_model(ModelMetadata {
            id: "openai/gpt-4-turbo".to_string(),
            provider: "openai".to_string(),
            name: "GPT-4 Turbo".to_string(),
            tier: ModelTier::Premium,
            capabilities: vec![
                ModelCapability::Programming,
                ModelCapability::Reasoning,
                ModelCapability::LongContext,
                ModelCapability::Multimodal,
            ],
            context_window: 128000,
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.03,
            average_latency_ms: 2000,
            quality_score: 0.90,
        });

        self.add_model(ModelMetadata {
            id: "openai/gpt-4o-mini".to_string(),
            provider: "openai".to_string(),
            name: "GPT-4 Omni Mini".to_string(),
            tier: ModelTier::Standard,
            capabilities: vec![ModelCapability::Programming, ModelCapability::FastResponse],
            context_window: 128000,
            cost_per_1k_input: 0.00015,
            cost_per_1k_output: 0.0006,
            average_latency_ms: 600,
            quality_score: 0.80,
        });

        // Google models
        self.add_model(ModelMetadata {
            id: "google/gemini-pro".to_string(),
            provider: "google".to_string(),
            name: "Gemini Pro".to_string(),
            tier: ModelTier::Premium,
            capabilities: vec![
                ModelCapability::Programming,
                ModelCapability::Reasoning,
                ModelCapability::Multimodal,
            ],
            context_window: 32000,
            cost_per_1k_input: 0.00125,
            cost_per_1k_output: 0.00375,
            average_latency_ms: 1800,
            quality_score: 0.88,
        });

        // Meta models
        self.add_model(ModelMetadata {
            id: "meta-llama/llama-3-70b-instruct".to_string(),
            provider: "meta-llama".to_string(),
            name: "Llama 3 70B".to_string(),
            tier: ModelTier::Standard,
            capabilities: vec![ModelCapability::Programming, ModelCapability::Reasoning],
            context_window: 8192,
            cost_per_1k_input: 0.0008,
            cost_per_1k_output: 0.0008,
            average_latency_ms: 1200,
            quality_score: 0.82,
        });

        // Mistral models
        self.add_model(ModelMetadata {
            id: "mistralai/mixtral-8x7b-instruct".to_string(),
            provider: "mistralai".to_string(),
            name: "Mixtral 8x7B".to_string(),
            tier: ModelTier::Economy,
            capabilities: vec![ModelCapability::Programming, ModelCapability::FastResponse],
            context_window: 32768,
            cost_per_1k_input: 0.0006,
            cost_per_1k_output: 0.0006,
            average_latency_ms: 900,
            quality_score: 0.78,
        });

        // Add more models as needed...
    }

    /// Add a model to the selector
    pub fn add_model(&mut self, metadata: ModelMetadata) {
        self.models.insert(metadata.id.clone(), metadata);
    }

    /// Update models from OpenRouter API
    pub fn update_from_api(&mut self, api_models: Vec<ModelInfo>) -> Result<()> {
        for model in api_models {
            // Parse provider and model name
            let (provider, model_name) = if model.id.contains('/') {
                let parts: Vec<&str> = model.id.split('/').collect();
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("unknown".to_string(), model.id.clone())
            };

            // Determine tier based on pricing
            let input_cost = parse_cost(&model.pricing.prompt)?;
            let output_cost = parse_cost(&model.pricing.completion)?;
            let tier = determine_tier(input_cost, output_cost);

            // Determine capabilities based on model name and architecture
            let capabilities = determine_capabilities(&model_name, &model.architecture.modality);

            // Create or update metadata
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider,
                name: model.name,
                tier,
                capabilities,
                context_window: model.context_length,
                cost_per_1k_input: input_cost * 1000.0,
                cost_per_1k_output: output_cost * 1000.0,
                average_latency_ms: estimate_latency(&tier),
                quality_score: estimate_quality(&tier, &model.id),
            };

            self.add_model(metadata);
        }

        Ok(())
    }

    /// Select best model for a task
    pub fn select_model(
        &self,
        task_description: &str,
        complexity: TaskComplexity,
        required_capabilities: Vec<ModelCapability>,
        max_cost_per_1k_tokens: Option<f32>,
    ) -> Result<ModelSelection> {
        // Filter models by required capabilities
        let mut eligible_models: Vec<&ModelMetadata> = self
            .models
            .values()
            .filter(|model| {
                required_capabilities
                    .iter()
                    .all(|cap| model.capabilities.contains(cap))
            })
            .collect();

        // Filter by cost constraint if provided
        if let Some(max_cost) = max_cost_per_1k_tokens {
            eligible_models
                .retain(|model| model.cost_per_1k_input + model.cost_per_1k_output <= max_cost);
        }

        // Filter by minimum tier based on complexity
        let min_tier = match complexity {
            TaskComplexity::Simple => ModelTier::Economy,
            TaskComplexity::Moderate => ModelTier::Standard,
            TaskComplexity::Complex => ModelTier::Premium,
            TaskComplexity::Expert => ModelTier::Flagship,
        };

        eligible_models.retain(|model| model.tier >= min_tier);

        if eligible_models.is_empty() {
            anyhow::bail!("No models available matching the requirements");
        }

        // Score and sort models based on strategy
        let mut scored_models: Vec<(f32, &ModelMetadata)> = eligible_models
            .into_iter()
            .map(|model| (self.score_model(model, complexity), model))
            .collect();

        scored_models.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Select primary and fallback models
        let primary = scored_models[0].1;
        let fallbacks: Vec<String> = scored_models
            .iter()
            .skip(1)
            .take(2)
            .map(|(_, model)| model.id.clone())
            .collect();

        // Estimate cost (assuming average 1k tokens per request)
        let estimated_cost = primary.cost_per_1k_input + primary.cost_per_1k_output;

        let reasoning = format!(
            "Selected {} ({}) for {} task. Strategy: {:?}. Score: {:.2}",
            primary.name,
            primary.id,
            match complexity {
                TaskComplexity::Simple => "simple",
                TaskComplexity::Moderate => "moderate",
                TaskComplexity::Complex => "complex",
                TaskComplexity::Expert => "expert",
            },
            self.strategy,
            scored_models[0].0
        );

        Ok(ModelSelection {
            primary: primary.id.clone(),
            fallbacks,
            estimated_cost,
            reasoning,
        })
    }

    /// Score a model based on current strategy
    fn score_model(&self, model: &ModelMetadata, complexity: TaskComplexity) -> f32 {
        let cost_score = 1.0 / (1.0 + model.cost_per_1k_input + model.cost_per_1k_output);
        let speed_score = 1.0 / (1.0 + model.average_latency_ms as f32 / 1000.0);
        let quality_score = model.quality_score;

        // Adjust weights based on strategy
        let (cost_weight, speed_weight, quality_weight) = match self.strategy {
            ModelSelectionStrategy::CostOptimized => (0.7, 0.2, 0.1),
            ModelSelectionStrategy::Balanced => (0.3, 0.3, 0.4),
            ModelSelectionStrategy::PerformanceOptimized => (0.1, 0.4, 0.5),
            ModelSelectionStrategy::QualityFirst => (0.0, 0.2, 0.8),
        };

        // Apply complexity multiplier to quality weight
        let complexity_multiplier = match complexity {
            TaskComplexity::Simple => 0.5,
            TaskComplexity::Moderate => 0.75,
            TaskComplexity::Complex => 1.0,
            TaskComplexity::Expert => 1.25,
        };

        cost_score * cost_weight
            + speed_score * speed_weight
            + quality_score * quality_weight * complexity_multiplier
    }

    /// Get all available models
    pub fn list_models(&self) -> Vec<&ModelMetadata> {
        let mut models: Vec<&ModelMetadata> = self.models.values().collect();
        models.sort_by(|a, b| {
            a.tier
                .cmp(&b.tier)
                .then(b.quality_score.partial_cmp(&a.quality_score).unwrap())
        });
        models
    }

    /// Get models by capability
    pub fn get_models_by_capability(&self, capability: ModelCapability) -> Vec<&ModelMetadata> {
        self.models
            .values()
            .filter(|model| model.capabilities.contains(&capability))
            .collect()
    }

    /// Get model metadata by ID
    pub fn get_model(&self, model_id: &str) -> Option<&ModelMetadata> {
        self.models.get(model_id)
    }
}

/// Parse cost string from API (e.g., "0.000015" -> 0.000015)
fn parse_cost(cost_str: &str) -> Result<f32> {
    cost_str
        .parse::<f32>()
        .context(format!("Failed to parse cost: {}", cost_str))
}

/// Determine model tier based on pricing
fn determine_tier(input_cost: f32, output_cost: f32) -> ModelTier {
    let total_cost = input_cost + output_cost;

    if total_cost > 0.02 {
        ModelTier::Flagship
    } else if total_cost > 0.005 {
        ModelTier::Premium
    } else if total_cost > 0.001 {
        ModelTier::Standard
    } else {
        ModelTier::Economy
    }
}

/// Determine capabilities from model name and modality
fn determine_capabilities(model_name: &str, modality: &str) -> Vec<ModelCapability> {
    let mut capabilities = Vec::new();

    // All models support programming by default
    capabilities.push(ModelCapability::Programming);

    // Check for specific capabilities
    if model_name.contains("vision") || modality.contains("image") {
        capabilities.push(ModelCapability::Multimodal);
    }

    if model_name.contains("turbo") || model_name.contains("haiku") || model_name.contains("mini") {
        capabilities.push(ModelCapability::FastResponse);
    }

    if model_name.contains("opus") || model_name.contains("gpt-4") || model_name.contains("claude")
    {
        capabilities.push(ModelCapability::Reasoning);
        capabilities.push(ModelCapability::Creative);
    }

    if model_name.contains("32k") || model_name.contains("100k") || model_name.contains("200k") {
        capabilities.push(ModelCapability::LongContext);
    }

    capabilities
}

/// Estimate latency based on tier
fn estimate_latency(tier: &ModelTier) -> u32 {
    match tier {
        ModelTier::Flagship => 2500,
        ModelTier::Premium => 1500,
        ModelTier::Standard => 1000,
        ModelTier::Economy => 800,
    }
}

/// Estimate quality score based on tier and model ID
fn estimate_quality(tier: &ModelTier, model_id: &str) -> f32 {
    let base_score = match tier {
        ModelTier::Flagship => 0.9,
        ModelTier::Premium => 0.8,
        ModelTier::Standard => 0.7,
        ModelTier::Economy => 0.6,
    };

    // Adjust for known high-quality models
    if model_id.contains("claude-3-opus") || model_id.contains("gpt-4") {
        base_score + 0.05
    } else {
        base_score
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_model_selection() {
        let selector = ModelSelector::new(ModelSelectionStrategy::Balanced);

        let result = selector
            .select_model(
                "Write a function to parse JSON",
                TaskComplexity::Simple,
                vec![ModelCapability::Programming],
                None,
            )
            .unwrap();

        assert!(!result.primary.is_empty());
        assert!(!result.fallbacks.is_empty());
        assert!(result.estimated_cost > 0.0);
    }

    #[test]
    fn test_capability_filtering() {
        let selector = ModelSelector::new(ModelSelectionStrategy::Balanced);

        let multimodal_models = selector.get_models_by_capability(ModelCapability::Multimodal);
        assert!(!multimodal_models.is_empty());

        for model in multimodal_models {
            assert!(model.capabilities.contains(&ModelCapability::Multimodal));
        }
    }
}
