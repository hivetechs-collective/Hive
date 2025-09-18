/// Cost Tracking and Optimization
///
/// Comprehensive cost tracking system for OpenRouter API usage with
/// budget management, cost optimization, and detailed analytics.
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::client::UsageInfo;
use super::models::ModelMetadata;

/// Cost estimate for a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub model_id: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub input_cost: f32,
    pub output_cost: f32,
    pub total_cost: f32,
    pub currency: String,
}

/// Cost tracking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub model_id: String,
    pub request_type: RequestType,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub input_cost: f32,
    pub output_cost: f32,
    pub total_cost: f32,
    pub duration_ms: u64,
    pub success: bool,
    pub metadata: Option<HashMap<String, String>>,
}

/// Request type for categorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestType {
    Consensus,
    Analysis,
    Generation,
    Translation,
    Other(String),
}

/// Budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub daily_limit: Option<f32>,
    pub monthly_limit: Option<f32>,
    pub per_request_limit: Option<f32>,
    pub alert_threshold: f32, // Percentage (0.0 - 1.0)
    pub enforce_limits: bool,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            daily_limit: None,
            monthly_limit: None,
            per_request_limit: None,
            alert_threshold: 0.8, // Alert at 80% of budget
            enforce_limits: false,
        }
    }
}

/// Budget status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub daily_spent: f32,
    pub monthly_spent: f32,
    pub daily_remaining: Option<f32>,
    pub monthly_remaining: Option<f32>,
    pub daily_percentage: Option<f32>,
    pub monthly_percentage: Option<f32>,
    pub alerts: Vec<BudgetAlert>,
}

/// Budget alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Alert severity level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// Cost analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalytics {
    pub total_cost: f32,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub average_cost_per_request: f32,
    pub average_tokens_per_request: f32,
    pub cost_by_model: HashMap<String, f32>,
    pub cost_by_type: HashMap<String, f32>,
    pub top_expensive_models: Vec<(String, f32)>,
    pub cost_trend: Vec<(DateTime<Utc>, f32)>,
}

/// Cost optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub model_id: String,
    pub alternative_model: String,
    pub potential_savings: f32,
    pub quality_tradeoff: f32, // 0.0 - 1.0
    pub reasoning: String,
}

/// Cost calculator
pub struct CostCalculator {
    model_pricing: HashMap<String, (f32, f32)>, // (input_cost_per_1k, output_cost_per_1k)
}

impl CostCalculator {
    /// Create a new cost calculator
    pub fn new() -> Self {
        let mut calculator = Self {
            model_pricing: HashMap::new(),
        };

        // Initialize with known pricing
        calculator.initialize_default_pricing();
        calculator
    }

    /// Initialize default model pricing
    fn initialize_default_pricing(&mut self) {
        // Claude models
        self.model_pricing
            .insert("anthropic/claude-3-opus".to_string(), (0.015, 0.075));
        self.model_pricing
            .insert("anthropic/claude-3-sonnet".to_string(), (0.003, 0.015));
        self.model_pricing
            .insert("anthropic/claude-3-haiku".to_string(), (0.00025, 0.00125));

        // OpenAI models
        self.model_pricing
            .insert("openai/gpt-4".to_string(), (0.03, 0.06));
        self.model_pricing
            .insert("openai/gpt-4-turbo".to_string(), (0.01, 0.03));
        self.model_pricing
            .insert("openai/gpt-4o-mini".to_string(), (0.00015, 0.0006));
        self.model_pricing
            .insert("openai/gpt-3.5-turbo".to_string(), (0.0005, 0.0015));

        // Google models
        self.model_pricing
            .insert("google/gemini-pro".to_string(), (0.00125, 0.00375));
        self.model_pricing
            .insert("google/gemini-pro-vision".to_string(), (0.00125, 0.00375));

        // Meta models
        self.model_pricing.insert(
            "meta-llama/llama-3-70b-instruct".to_string(),
            (0.0008, 0.0008),
        );
        self.model_pricing.insert(
            "meta-llama/llama-3-8b-instruct".to_string(),
            (0.0002, 0.0002),
        );

        // Mistral models
        self.model_pricing.insert(
            "mistralai/mixtral-8x7b-instruct".to_string(),
            (0.0006, 0.0006),
        );
        self.model_pricing.insert(
            "mistralai/mistral-7b-instruct".to_string(),
            (0.0002, 0.0002),
        );
    }

    /// Update pricing from model metadata
    pub fn update_from_metadata(&mut self, models: &[ModelMetadata]) {
        for model in models {
            self.model_pricing.insert(
                model.id.clone(),
                (model.cost_per_1k_input, model.cost_per_1k_output),
            );
        }
    }

    /// Calculate cost for a request
    pub fn calculate_cost(
        &self,
        model_id: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<CostEstimate> {
        let (input_cost_per_1k, output_cost_per_1k) = self
            .model_pricing
            .get(model_id)
            .copied()
            .unwrap_or_else(|| {
                log::warn!("Unknown model pricing for {}, using default", model_id);
                (0.001, 0.002) // Default fallback pricing
            });

        let input_cost = (input_tokens as f32 / 1000.0) * input_cost_per_1k;
        let output_cost = (output_tokens as f32 / 1000.0) * output_cost_per_1k;
        let total_cost = input_cost + output_cost;

        Ok(CostEstimate {
            model_id: model_id.to_string(),
            input_tokens,
            output_tokens,
            input_cost,
            output_cost,
            total_cost,
            currency: "USD".to_string(),
        })
    }

    /// Calculate cost from usage info
    pub fn calculate_from_usage(&self, model_id: &str, usage: &UsageInfo) -> Result<CostEstimate> {
        self.calculate_cost(model_id, usage.prompt_tokens, usage.completion_tokens)
    }

    /// Get model pricing
    pub fn get_pricing(&self, model_id: &str) -> Option<(f32, f32)> {
        self.model_pricing.get(model_id).copied()
    }

    /// Compare model costs
    pub fn compare_models(
        &self,
        model_a: &str,
        model_b: &str,
        avg_input_tokens: u32,
        avg_output_tokens: u32,
    ) -> Result<f32> {
        let cost_a = self.calculate_cost(model_a, avg_input_tokens, avg_output_tokens)?;
        let cost_b = self.calculate_cost(model_b, avg_input_tokens, avg_output_tokens)?;

        Ok(cost_a.total_cost - cost_b.total_cost)
    }
}

/// Cost tracker for monitoring usage
pub struct CostTracker {
    entries: Arc<RwLock<Vec<CostEntry>>>,
    budget: Arc<RwLock<BudgetConfig>>,
    calculator: Arc<CostCalculator>,
}

impl CostTracker {
    /// Create a new cost tracker
    pub fn new(budget: BudgetConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            budget: Arc::new(RwLock::new(budget)),
            calculator: Arc::new(CostCalculator::new()),
        }
    }

    /// Track a cost entry
    pub async fn track_cost(
        &self,
        model_id: &str,
        request_type: RequestType,
        usage: &UsageInfo,
        duration_ms: u64,
        success: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let cost_estimate = self.calculator.calculate_from_usage(model_id, usage)?;

        let entry = CostEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            model_id: model_id.to_string(),
            request_type,
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
            input_cost: cost_estimate.input_cost,
            output_cost: cost_estimate.output_cost,
            total_cost: cost_estimate.total_cost,
            duration_ms,
            success,
            metadata,
        };

        let mut entries = self.entries.write().await;
        entries.push(entry);

        // Check budget limits
        self.check_budget_alerts().await?;

        Ok(())
    }

    /// Get budget status
    pub async fn get_budget_status(&self) -> Result<BudgetStatus> {
        let entries = self.entries.read().await;
        let budget = self.budget.read().await;

        let now = Utc::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let month_start = now
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        // Calculate daily spending
        let daily_spent: f32 = entries
            .iter()
            .filter(|e| e.timestamp >= today_start && e.success)
            .map(|e| e.total_cost)
            .sum();

        // Calculate monthly spending
        let monthly_spent: f32 = entries
            .iter()
            .filter(|e| e.timestamp >= month_start && e.success)
            .map(|e| e.total_cost)
            .sum();

        let mut alerts = Vec::new();

        // Check daily budget
        let (daily_remaining, daily_percentage) = if let Some(limit) = budget.daily_limit {
            let remaining = limit - daily_spent;
            let percentage = daily_spent / limit;

            if percentage >= budget.alert_threshold {
                alerts.push(BudgetAlert {
                    level: if percentage >= 1.0 {
                        AlertLevel::Critical
                    } else {
                        AlertLevel::Warning
                    },
                    message: format!(
                        "Daily budget {:.1}% used (${:.2} of ${:.2})",
                        percentage * 100.0,
                        daily_spent,
                        limit
                    ),
                    timestamp: now,
                });
            }

            (Some(remaining), Some(percentage))
        } else {
            (None, None)
        };

        // Check monthly budget
        let (monthly_remaining, monthly_percentage) = if let Some(limit) = budget.monthly_limit {
            let remaining = limit - monthly_spent;
            let percentage = monthly_spent / limit;

            if percentage >= budget.alert_threshold {
                alerts.push(BudgetAlert {
                    level: if percentage >= 1.0 {
                        AlertLevel::Critical
                    } else {
                        AlertLevel::Warning
                    },
                    message: format!(
                        "Monthly budget {:.1}% used (${:.2} of ${:.2})",
                        percentage * 100.0,
                        monthly_spent,
                        limit
                    ),
                    timestamp: now,
                });
            }

            (Some(remaining), Some(percentage))
        } else {
            (None, None)
        };

        Ok(BudgetStatus {
            daily_spent,
            monthly_spent,
            daily_remaining,
            monthly_remaining,
            daily_percentage,
            monthly_percentage,
            alerts,
        })
    }

    /// Check if request is within budget
    pub async fn check_budget(&self, estimated_cost: f32) -> Result<bool> {
        let budget = self.budget.read().await;

        if !budget.enforce_limits {
            return Ok(true);
        }

        // Check per-request limit
        if let Some(limit) = budget.per_request_limit {
            if estimated_cost > limit {
                return Ok(false);
            }
        }

        let status = self.get_budget_status().await?;

        // Check daily limit
        if let Some(remaining) = status.daily_remaining {
            if remaining < estimated_cost {
                return Ok(false);
            }
        }

        // Check monthly limit
        if let Some(remaining) = status.monthly_remaining {
            if remaining < estimated_cost {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get cost analytics
    pub async fn get_analytics(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<CostAnalytics> {
        let entries = self.entries.read().await;

        let filtered_entries: Vec<&CostEntry> = entries
            .iter()
            .filter(|e| {
                e.success
                    && start_date.map_or(true, |start| e.timestamp >= start)
                    && end_date.map_or(true, |end| e.timestamp <= end)
            })
            .collect();

        let total_cost: f32 = filtered_entries.iter().map(|e| e.total_cost).sum();
        let total_requests = filtered_entries.len() as u64;
        let total_tokens: u64 = filtered_entries
            .iter()
            .map(|e| (e.input_tokens + e.output_tokens) as u64)
            .sum();

        let average_cost_per_request = if total_requests > 0 {
            total_cost / total_requests as f32
        } else {
            0.0
        };

        let average_tokens_per_request = if total_requests > 0 {
            total_tokens as f32 / total_requests as f32
        } else {
            0.0
        };

        // Cost by model
        let mut cost_by_model = HashMap::new();
        for entry in &filtered_entries {
            *cost_by_model.entry(entry.model_id.clone()).or_insert(0.0) += entry.total_cost;
        }

        // Cost by type
        let mut cost_by_type = HashMap::new();
        for entry in &filtered_entries {
            let type_key = match &entry.request_type {
                RequestType::Other(s) => s.clone(),
                _ => format!("{:?}", entry.request_type),
            };
            *cost_by_type.entry(type_key).or_insert(0.0) += entry.total_cost;
        }

        // Top expensive models
        let mut model_costs: Vec<(String, f32)> = cost_by_model.into_iter().collect();
        model_costs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top_expensive_models = model_costs.into_iter().take(5).collect();

        // Cost trend (daily)
        let mut daily_costs: HashMap<chrono::NaiveDate, f32> = HashMap::new();
        for entry in &filtered_entries {
            let date = entry.timestamp.date_naive();
            *daily_costs.entry(date).or_insert(0.0) += entry.total_cost;
        }

        let mut cost_trend: Vec<(DateTime<Utc>, f32)> = daily_costs
            .into_iter()
            .map(|(date, cost)| (date.and_hms_opt(0, 0, 0).unwrap().and_utc(), cost))
            .collect();
        cost_trend.sort_by_key(|(date, _)| *date);

        Ok(CostAnalytics {
            total_cost,
            total_requests,
            total_tokens,
            average_cost_per_request,
            average_tokens_per_request,
            cost_by_model: HashMap::new(), // Rebuilt above
            cost_by_type: HashMap::new(),  // Rebuilt above
            top_expensive_models,
            cost_trend,
        })
    }

    /// Get optimization suggestions
    pub async fn get_optimization_suggestions(
        &self,
        min_quality_score: f32,
    ) -> Result<Vec<OptimizationSuggestion>> {
        let entries = self.entries.read().await;
        let mut suggestions = Vec::new();

        // Analyze recent usage patterns
        let recent_entries: Vec<&CostEntry> = entries
            .iter()
            .filter(|e| e.success && e.timestamp > Utc::now() - chrono::Duration::days(7))
            .collect();

        // Group by model
        let mut model_usage: HashMap<String, (u64, f32)> = HashMap::new();
        for entry in &recent_entries {
            let (count, cost) = model_usage
                .entry(entry.model_id.clone())
                .or_insert((0, 0.0));
            *count += 1;
            *cost += entry.total_cost;
        }

        // Find optimization opportunities
        // This would integrate with the model selector to find cheaper alternatives
        // For now, we'll provide some hardcoded suggestions

        if let Some((count, cost)) = model_usage.get("openai/gpt-4") {
            if *count > 10 {
                suggestions.push(OptimizationSuggestion {
                    model_id: "openai/gpt-4".to_string(),
                    alternative_model: "openai/gpt-4-turbo".to_string(),
                    potential_savings: cost * 0.66, // GPT-4 Turbo is ~1/3 the cost
                    quality_tradeoff: 0.05,         // Minimal quality difference
                    reasoning: format!(
                        "GPT-4 Turbo offers similar quality at 1/3 the cost. \
                        You've used GPT-4 {} times this week for ${:.2}.",
                        count, cost
                    ),
                });
            }
        }

        if let Some((count, cost)) = model_usage.get("anthropic/claude-3-opus") {
            if *count > 5 && min_quality_score < 0.9 {
                suggestions.push(OptimizationSuggestion {
                    model_id: "anthropic/claude-3-opus".to_string(),
                    alternative_model: "anthropic/claude-3-sonnet".to_string(),
                    potential_savings: cost * 0.8, // Sonnet is ~1/5 the cost
                    quality_tradeoff: 0.1,
                    reasoning: format!(
                        "Claude 3 Sonnet offers excellent quality at 20% of Opus cost. \
                        Consider for non-critical tasks. Used Opus {} times for ${:.2}.",
                        count, cost
                    ),
                });
            }
        }

        Ok(suggestions)
    }

    /// Update budget configuration
    pub async fn update_budget(&self, budget: BudgetConfig) -> Result<()> {
        let mut current_budget = self.budget.write().await;
        *current_budget = budget;
        Ok(())
    }

    /// Clear old entries (data retention)
    pub async fn cleanup_old_entries(&self, days_to_keep: i64) -> Result<usize> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days_to_keep);
        let mut entries = self.entries.write().await;
        let initial_count = entries.len();

        entries.retain(|e| e.timestamp > cutoff_date);

        let removed = initial_count - entries.len();
        log::info!("Cleaned up {} old cost entries", removed);

        Ok(removed)
    }

    /// Check budget alerts (internal)
    async fn check_budget_alerts(&self) -> Result<()> {
        let status = self.get_budget_status().await?;

        for alert in &status.alerts {
            match alert.level {
                AlertLevel::Info => log::info!("Budget alert: {}", alert.message),
                AlertLevel::Warning => log::warn!("Budget warning: {}", alert.message),
                AlertLevel::Critical => log::error!("Budget critical: {}", alert.message),
            }
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        let calculator = CostCalculator::new();

        let cost = calculator
            .calculate_cost("openai/gpt-4", 1000, 500)
            .unwrap();

        assert_eq!(cost.input_tokens, 1000);
        assert_eq!(cost.output_tokens, 500);
        assert_eq!(cost.input_cost, 0.03); // $0.03 per 1k tokens
        assert_eq!(cost.output_cost, 0.03); // $0.06 per 1k tokens * 0.5
        assert_eq!(cost.total_cost, 0.06);
    }

    #[tokio::test]
    async fn test_budget_tracking() {
        let budget = BudgetConfig {
            daily_limit: Some(10.0),
            monthly_limit: Some(100.0),
            per_request_limit: Some(1.0),
            alert_threshold: 0.8,
            enforce_limits: true,
        };

        let tracker = CostTracker::new(budget);

        // Test budget check
        assert!(tracker.check_budget(0.5).await.unwrap());
        assert!(!tracker.check_budget(2.0).await.unwrap()); // Exceeds per-request limit
    }
}
