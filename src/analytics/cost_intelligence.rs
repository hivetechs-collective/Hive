//! Cost Intelligence and Optimization Recommendations
//!
//! This module provides:
//! - Detailed cost analysis and tracking
//! - Model efficiency calculations
//! - Budget alerts and monitoring
//! - Cost optimization strategies
//! - Spending pattern analysis
//! - Cost allocation and attribution

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::analytics::AdvancedAnalyticsConfig;
use crate::core::database::{get_database, ActivityLog};

/// Cost intelligence engine
pub struct CostIntelligence {
    config: Arc<RwLock<AdvancedAnalyticsConfig>>,
    cost_analyzer: Arc<CostAnalyzer>,
    optimization_engine: Arc<OptimizationEngine>,
    budget_monitor: Arc<BudgetMonitor>,
    allocation_tracker: Arc<AllocationTracker>,
}

/// Cost insights summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInsights {
    pub total_spend: f64,
    pub spend_by_model: HashMap<String, ModelSpend>,
    pub spend_by_user: HashMap<String, f64>,
    pub spend_by_department: HashMap<String, f64>,
    pub efficiency_metrics: EfficiencyMetrics,
    pub optimization_opportunities: Vec<CostOptimization>,
    pub budget_status: BudgetStatus,
    pub spending_patterns: Vec<SpendingPattern>,
}

/// Model spending details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpend {
    pub total_cost: f64,
    pub query_count: u64,
    pub avg_cost_per_query: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub efficiency_score: f64,
}

/// Efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub overall_efficiency: f64,
    pub model_efficiency: HashMap<String, ModelEfficiency>,
    pub cost_per_token: f64,
    pub token_utilization: f64,
    pub cache_hit_rate: f64,
    pub redundancy_factor: f64,
}

/// Model efficiency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEfficiency {
    pub model_name: String,
    pub efficiency_score: f64,
    pub quality_score: f64,
    pub cost_effectiveness: f64,
    pub recommended_use_cases: Vec<String>,
    pub alternatives: Vec<ModelAlternative>,
}

/// Model alternative suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAlternative {
    pub model_name: String,
    pub potential_savings: f64,
    pub quality_tradeoff: f64,
    pub use_case_fit: f64,
}

/// Cost optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    pub optimization_type: OptimizationType,
    pub title: String,
    pub description: String,
    pub potential_savings: f64,
    pub implementation_effort: EffortLevel,
    pub impact_on_quality: QualityImpact,
    pub priority: OptimizationPriority,
    pub action_items: Vec<String>,
}

/// Types of optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationType {
    ModelSelection,
    CachingStrategy,
    TokenOptimization,
    BatchProcessing,
    RateLimiting,
    UserQuotas,
    PromptEngineering,
}

/// Optimization priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Quality impact assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityImpact {
    None,
    Minimal,
    Moderate,
    Significant,
}

/// Effort level for implementation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffortLevel {
    Trivial,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Budget alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub current_spend: f64,
    pub budget_limit: f64,
    pub projected_overrun: Option<f64>,
    pub triggered_at: DateTime<Utc>,
}

/// Alert types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    ApproachingLimit,
    ExceededLimit,
    UnusualSpending,
    RapidGrowth,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Budget status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub current_spend: f64,
    pub budget_limit: f64,
    pub utilization_percent: f64,
    pub days_remaining: u32,
    pub projected_spend: f64,
    pub status: BudgetHealth,
    pub alerts: Vec<BudgetAlert>,
}

/// Budget health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetHealth {
    Healthy,
    Warning,
    AtRisk,
    Exceeded,
}

/// Spending pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingPattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub time_period: String,
    pub trend: SpendingTrend,
    pub anomaly_score: f64,
    pub contributing_factors: Vec<String>,
}

/// Pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    Daily,
    Weekly,
    Hourly,
    UserBased,
    ModelBased,
    Anomalous,
}

/// Spending trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpendingTrend {
    Increasing,
    Stable,
    Decreasing,
    Volatile,
}

/// Cost allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAllocation {
    pub allocation_key: String,
    pub total_cost: f64,
    pub percentage: f64,
    pub sub_allocations: HashMap<String, f64>,
    pub tags: HashMap<String, String>,
}

/// Optimization strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStrategy {
    pub name: String,
    pub description: String,
    pub expected_savings: f64,
    pub implementation_steps: Vec<ImplementationStep>,
    pub success_metrics: Vec<String>,
    pub risks: Vec<String>,
}

/// Implementation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationStep {
    pub order: u32,
    pub title: String,
    pub description: String,
    pub estimated_hours: f64,
    pub dependencies: Vec<String>,
}

/// Cost analyzer component
struct CostAnalyzer {
    model_pricing: HashMap<String, ModelPricing>,
    cost_cache: Arc<RwLock<HashMap<String, CachedCost>>>,
}

/// Model pricing information
#[derive(Debug, Clone)]
struct ModelPricing {
    provider: String,
    input_cost_per_1k: f64,
    output_cost_per_1k: f64,
    minimum_cost: f64,
    quality_tier: QualityTier,
}

/// Quality tier for models
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QualityTier {
    Premium,
    Standard,
    Economy,
}

/// Cached cost calculation
#[derive(Debug, Clone)]
struct CachedCost {
    cost: f64,
    calculated_at: DateTime<Utc>,
    metadata: HashMap<String, String>,
}

/// Optimization engine
struct OptimizationEngine {
    optimization_rules: Vec<OptimizationRule>,
    strategy_templates: HashMap<String, OptimizationStrategy>,
}

/// Optimization rule
struct OptimizationRule {
    name: String,
    condition: Box<dyn Fn(&CostInsights) -> bool + Send + Sync>,
    generator: Box<dyn Fn(&CostInsights) -> CostOptimization + Send + Sync>,
}

/// Budget monitor
struct BudgetMonitor {
    budgets: Arc<RwLock<HashMap<String, Budget>>>,
    alert_thresholds: Vec<AlertThreshold>,
}

/// Budget configuration
#[derive(Debug, Clone)]
struct Budget {
    name: String,
    limit: f64,
    period: BudgetPeriod,
    current_spend: f64,
    alerts_enabled: bool,
}

/// Budget period
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BudgetPeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annual,
}

/// Alert threshold
struct AlertThreshold {
    percentage: f64,
    severity: AlertSeverity,
}

/// Allocation tracker
struct AllocationTracker {
    allocation_rules: HashMap<String, AllocationRule>,
    cost_centers: HashMap<String, CostCenter>,
}

/// Allocation rule
struct AllocationRule {
    name: String,
    matcher: Box<dyn Fn(&ActivityLog) -> bool + Send + Sync>,
    allocation_key: String,
}

/// Cost center
#[derive(Debug, Clone)]
struct CostCenter {
    name: String,
    owner: String,
    budget: Option<f64>,
    tags: HashMap<String, String>,
}

impl CostIntelligence {
    /// Create new cost intelligence engine
    pub async fn new(config: Arc<RwLock<AdvancedAnalyticsConfig>>) -> Result<Self> {
        info!("Initializing cost intelligence engine");

        let cost_analyzer = Arc::new(CostAnalyzer::new());
        let optimization_engine = Arc::new(OptimizationEngine::new());
        let budget_monitor = Arc::new(BudgetMonitor::new());
        let allocation_tracker = Arc::new(AllocationTracker::new());

        Ok(Self {
            config,
            cost_analyzer,
            optimization_engine,
            budget_monitor,
            allocation_tracker,
        })
    }

    /// Get cost insights
    pub async fn get_insights(&self) -> Result<CostInsights> {
        debug!("Generating cost insights");

        // Analyze recent spending
        let activities = ActivityLog::get_recent(10000).await?;
        let spend_analysis = self.cost_analyzer.analyze_spending(&activities).await?;

        // Calculate efficiency metrics
        let efficiency_metrics = self.calculate_efficiency_metrics(&spend_analysis)?;

        // Find optimization opportunities
        let optimization_opportunities = self
            .optimization_engine
            .find_opportunities(&spend_analysis, &efficiency_metrics)?;

        // Check budget status
        let budget_status = self
            .budget_monitor
            .check_budget_status(&spend_analysis)
            .await?;

        // Analyze spending patterns
        let spending_patterns = self.analyze_spending_patterns(&activities)?;

        Ok(CostInsights {
            total_spend: spend_analysis.total_spend,
            spend_by_model: spend_analysis.spend_by_model,
            spend_by_user: spend_analysis.spend_by_user,
            spend_by_department: spend_analysis.spend_by_department,
            efficiency_metrics,
            optimization_opportunities,
            budget_status,
            spending_patterns,
        })
    }

    /// Get cost optimization recommendations
    pub async fn get_optimizations(&self) -> Result<Vec<CostOptimization>> {
        let insights = self.get_insights().await?;
        Ok(insights.optimization_opportunities)
    }

    /// Set budget alert
    pub async fn set_budget_alert(&self, limit: f64, period: &str) -> Result<()> {
        self.budget_monitor.set_budget(limit, period).await
    }

    /// Analyze model efficiency
    pub async fn analyze_model_efficiency(&self, model: &str) -> Result<ModelEfficiency> {
        debug!("Analyzing efficiency for model: {}", model);

        let activities = ActivityLog::get_recent(1000).await?;
        let model_activities: Vec<_> = activities
            .into_iter()
            .filter(|a| a.model_used.as_ref() == Some(&model.to_string()))
            .collect();

        self.cost_analyzer
            .calculate_model_efficiency(model, &model_activities)
    }

    /// Get spending forecast
    pub async fn get_spending_forecast(&self, days: u32) -> Result<SpendingForecast> {
        debug!("Generating spending forecast for {} days", days);

        let historical_data = self.get_historical_spending(90).await?;
        self.forecast_spending(&historical_data, days)
    }

    /// Reload configuration
    pub async fn reload_config(&self) -> Result<()> {
        debug!("Reloading cost intelligence configuration");
        Ok(())
    }

    // Private helper methods

    fn calculate_efficiency_metrics(&self, analysis: &SpendAnalysis) -> Result<EfficiencyMetrics> {
        let mut model_efficiency = HashMap::new();

        for (model, spend) in &analysis.spend_by_model {
            let efficiency = ModelEfficiency {
                model_name: model.clone(),
                efficiency_score: self.calculate_efficiency_score(spend),
                quality_score: self.estimate_quality_score(model),
                cost_effectiveness: spend.avg_cost_per_query.recip() * 100.0,
                recommended_use_cases: self.get_recommended_use_cases(model),
                alternatives: self.find_alternatives(model, spend),
            };
            model_efficiency.insert(model.clone(), efficiency);
        }

        let overall_efficiency = if !model_efficiency.is_empty() {
            model_efficiency
                .values()
                .map(|e| e.efficiency_score)
                .sum::<f64>()
                / model_efficiency.len() as f64
        } else {
            0.0
        };

        Ok(EfficiencyMetrics {
            overall_efficiency,
            model_efficiency,
            cost_per_token: analysis.total_cost_per_token,
            token_utilization: analysis.token_utilization,
            cache_hit_rate: 0.15,   // Placeholder
            redundancy_factor: 1.2, // Placeholder
        })
    }

    fn calculate_efficiency_score(&self, spend: &ModelSpend) -> f64 {
        // Efficiency based on cost per query and token utilization
        let cost_factor = 1.0 / (1.0 + spend.avg_cost_per_query);
        let utilization_factor =
            spend.output_tokens as f64 / (spend.input_tokens + spend.output_tokens) as f64;

        (cost_factor * 0.6 + utilization_factor * 0.4) * 100.0
    }

    fn estimate_quality_score(&self, model: &str) -> f64 {
        // Placeholder quality scores
        match model {
            m if m.contains("gpt-4") => 95.0,
            m if m.contains("claude-3-opus") => 93.0,
            m if m.contains("claude-3-sonnet") => 88.0,
            m if m.contains("claude-3-haiku") => 82.0,
            _ => 75.0,
        }
    }

    fn get_recommended_use_cases(&self, model: &str) -> Vec<String> {
        match model {
            m if m.contains("gpt-4") => vec![
                "Complex reasoning".to_string(),
                "Creative writing".to_string(),
                "Technical analysis".to_string(),
            ],
            m if m.contains("claude-3-opus") => vec![
                "Long-form content".to_string(),
                "Research tasks".to_string(),
                "Code generation".to_string(),
            ],
            m if m.contains("claude-3-sonnet") => vec![
                "General queries".to_string(),
                "Summarization".to_string(),
                "Data analysis".to_string(),
            ],
            _ => vec!["Basic queries".to_string()],
        }
    }

    fn find_alternatives(&self, model: &str, spend: &ModelSpend) -> Vec<ModelAlternative> {
        let mut alternatives = Vec::new();

        // Suggest cheaper alternatives based on current model
        if model.contains("gpt-4") && spend.avg_cost_per_query > 0.10 {
            alternatives.push(ModelAlternative {
                model_name: "claude-3-sonnet".to_string(),
                potential_savings: spend.total_cost * 0.7,
                quality_tradeoff: 0.92,
                use_case_fit: 0.85,
            });
        }

        if model.contains("claude-3-opus") {
            alternatives.push(ModelAlternative {
                model_name: "claude-3-haiku".to_string(),
                potential_savings: spend.total_cost * 0.8,
                quality_tradeoff: 0.88,
                use_case_fit: 0.80,
            });
        }

        alternatives
    }

    fn analyze_spending_patterns(
        &self,
        activities: &[ActivityLog],
    ) -> Result<Vec<SpendingPattern>> {
        let mut patterns = Vec::new();

        // Analyze hourly patterns
        let hourly_pattern = self.analyze_hourly_pattern(activities)?;
        if let Some(pattern) = hourly_pattern {
            patterns.push(pattern);
        }

        // Analyze user patterns
        let user_patterns = self.analyze_user_patterns(activities)?;
        patterns.extend(user_patterns);

        // Detect anomalies
        let anomalies = self.detect_spending_anomalies(activities)?;
        patterns.extend(anomalies);

        Ok(patterns)
    }

    fn analyze_hourly_pattern(
        &self,
        activities: &[ActivityLog],
    ) -> Result<Option<SpendingPattern>> {
        // Group activities by hour
        let mut hourly_costs: HashMap<u32, f64> = HashMap::new();

        for activity in activities {
            if let (Ok(timestamp), Some(cost)) =
                (activity.created_at.parse::<DateTime<Utc>>(), activity.cost)
            {
                let hour = timestamp.time().hour();
                *hourly_costs.entry(hour).or_insert(0.0) += cost;
            }
        }

        if hourly_costs.is_empty() {
            return Ok(None);
        }

        // Find peak hours
        let max_hour = hourly_costs
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(h, _)| *h)
            .unwrap_or(0);

        Ok(Some(SpendingPattern {
            pattern_type: PatternType::Hourly,
            description: format!("Peak spending occurs at {}:00 UTC", max_hour),
            time_period: "24 hours".to_string(),
            trend: SpendingTrend::Stable,
            anomaly_score: 0.0,
            contributing_factors: vec![
                "Business hours activity".to_string(),
                "Scheduled batch processing".to_string(),
            ],
        }))
    }

    fn analyze_user_patterns(&self, activities: &[ActivityLog]) -> Result<Vec<SpendingPattern>> {
        let mut patterns = Vec::new();
        let mut user_costs: HashMap<String, f64> = HashMap::new();

        for activity in activities {
            if let (Some(user), Some(cost)) = (&activity.user_id, activity.cost) {
                *user_costs.entry(user.clone()).or_insert(0.0) += cost;
            }
        }

        // Find high-spending users
        let total_cost: f64 = user_costs.values().sum();
        for (user, cost) in user_costs {
            let percentage = (cost / total_cost) * 100.0;
            if percentage > 20.0 {
                patterns.push(SpendingPattern {
                    pattern_type: PatternType::UserBased,
                    description: format!(
                        "User {} accounts for {:.1}% of total spend",
                        user, percentage
                    ),
                    time_period: "Analysis period".to_string(),
                    trend: SpendingTrend::Stable,
                    anomaly_score: if percentage > 40.0 { 0.8 } else { 0.3 },
                    contributing_factors: vec![
                        "Heavy API usage".to_string(),
                        "Premium model selection".to_string(),
                    ],
                });
            }
        }

        Ok(patterns)
    }

    fn detect_spending_anomalies(
        &self,
        activities: &[ActivityLog],
    ) -> Result<Vec<SpendingPattern>> {
        let mut patterns = Vec::new();

        // Simple anomaly detection based on cost outliers
        let costs: Vec<f64> = activities.iter().filter_map(|a| a.cost).collect();

        if costs.len() < 10 {
            return Ok(patterns);
        }

        let mean = costs.iter().sum::<f64>() / costs.len() as f64;
        let variance = costs.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / costs.len() as f64;
        let std_dev = variance.sqrt();

        // Find outliers (> 3 standard deviations)
        for activity in activities {
            if let Some(cost) = activity.cost {
                let z_score = (cost - mean).abs() / std_dev;
                if z_score > 3.0 {
                    patterns.push(SpendingPattern {
                        pattern_type: PatternType::Anomalous,
                        description: format!("Unusual high cost detected: ${:.2}", cost),
                        time_period: activity.created_at.clone(),
                        trend: SpendingTrend::Volatile,
                        anomaly_score: z_score / 5.0, // Normalize to 0-1
                        contributing_factors: vec![
                            "Large token count".to_string(),
                            "Premium model usage".to_string(),
                        ],
                    });
                }
            }
        }

        Ok(patterns)
    }

    async fn get_historical_spending(&self, days: u32) -> Result<Vec<DailySpend>> {
        let activities = ActivityLog::get_recent(10000).await?;
        let mut daily_spending = BTreeMap::new();

        let cutoff = Utc::now() - Duration::days(days as i64);

        for activity in activities {
            if let (Ok(timestamp), Some(cost)) =
                (activity.created_at.parse::<DateTime<Utc>>(), activity.cost)
            {
                if timestamp > cutoff {
                    let date = timestamp.date_naive();
                    *daily_spending.entry(date).or_insert(0.0) += cost;
                }
            }
        }

        Ok(daily_spending
            .into_iter()
            .map(|(date, cost)| DailySpend {
                date: date.to_string(),
                cost,
            })
            .collect())
    }

    fn forecast_spending(&self, historical: &[DailySpend], days: u32) -> Result<SpendingForecast> {
        if historical.is_empty() {
            return Ok(SpendingForecast {
                predicted_spend: vec![],
                confidence_interval: (0.0, 0.0),
                trend: SpendingTrend::Stable,
            });
        }

        // Simple linear regression for forecasting
        let n = historical.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, spend) in historical.iter().enumerate() {
            let x = i as f64;
            let y = spend.cost;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Generate predictions
        let mut predicted_spend = Vec::new();
        let base_index = historical.len() as f64;

        for i in 0..days {
            let x = base_index + i as f64;
            let predicted = (slope * x + intercept).max(0.0);

            predicted_spend.push(DailySpend {
                date: (Utc::now() + Duration::days(i as i64))
                    .format("%Y-%m-%d")
                    .to_string(),
                cost: predicted,
            });
        }

        // Calculate confidence interval
        let avg_cost = sum_y / n;
        let confidence_interval = (avg_cost * 0.8, avg_cost * 1.2);

        let trend = if slope > 0.01 {
            SpendingTrend::Increasing
        } else if slope < -0.01 {
            SpendingTrend::Decreasing
        } else {
            SpendingTrend::Stable
        };

        Ok(SpendingForecast {
            predicted_spend,
            confidence_interval,
            trend,
        })
    }
}

/// Spending analysis results
#[derive(Debug)]
struct SpendAnalysis {
    total_spend: f64,
    spend_by_model: HashMap<String, ModelSpend>,
    spend_by_user: HashMap<String, f64>,
    spend_by_department: HashMap<String, f64>,
    total_cost_per_token: f64,
    token_utilization: f64,
}

/// Daily spending data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DailySpend {
    date: String,
    cost: f64,
}

/// Spending forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingForecast {
    pub predicted_spend: Vec<DailySpend>,
    pub confidence_interval: (f64, f64),
    pub trend: SpendingTrend,
}

impl CostInsights {
    /// Format as markdown
    pub fn format_markdown(&self) -> Result<String> {
        let mut output = String::new();

        output.push_str(&format!("**Total Spend**: ${:.2}\n\n", self.total_spend));

        // Top models by cost
        if !self.spend_by_model.is_empty() {
            output.push_str("### Top Models by Cost\n\n");
            let mut models: Vec<_> = self.spend_by_model.iter().collect();
            models.sort_by(|a, b| b.1.total_cost.partial_cmp(&a.1.total_cost).unwrap());
            models.truncate(5);

            for (model, spend) in models {
                output.push_str(&format!(
                    "- **{}**: ${:.2} ({} queries, ${:.4}/query)\n",
                    model, spend.total_cost, spend.query_count, spend.avg_cost_per_query
                ));
            }
            output.push_str("\n");
        }

        // Efficiency metrics
        output.push_str(&format!(
            "### Efficiency Score: {:.1}%\n\n",
            self.efficiency_metrics.overall_efficiency
        ));
        output.push_str(&format!(
            "- Cost per Token: ${:.6}\n",
            self.efficiency_metrics.cost_per_token
        ));
        output.push_str(&format!(
            "- Cache Hit Rate: {:.1}%\n",
            self.efficiency_metrics.cache_hit_rate * 100.0
        ));
        output.push_str("\n");

        // Budget status
        output.push_str(&format!(
            "### Budget Status: {:?}\n\n",
            self.budget_status.status
        ));
        output.push_str(&format!(
            "- Current: ${:.2} / ${:.2} ({:.1}%)\n",
            self.budget_status.current_spend,
            self.budget_status.budget_limit,
            self.budget_status.utilization_percent
        ));
        output.push_str(&format!(
            "- Projected: ${:.2}\n\n",
            self.budget_status.projected_spend
        ));

        // Top optimizations
        if !self.optimization_opportunities.is_empty() {
            output.push_str("### Top Optimization Opportunities\n\n");
            for (i, opt) in self.optimization_opportunities.iter().take(3).enumerate() {
                output.push_str(&format!("{}. **{}**\n", i + 1, opt.title));
                output.push_str(&format!(
                    "   - Potential Savings: ${:.2}\n",
                    opt.potential_savings
                ));
                output.push_str(&format!("   - Effort: {:?}\n", opt.implementation_effort));
            }
        }

        Ok(output)
    }
}

impl CostAnalyzer {
    fn new() -> Self {
        let mut model_pricing = HashMap::new();

        // OpenAI models
        model_pricing.insert(
            "gpt-4".to_string(),
            ModelPricing {
                provider: "openai".to_string(),
                input_cost_per_1k: 0.03,
                output_cost_per_1k: 0.06,
                minimum_cost: 0.0,
                quality_tier: QualityTier::Premium,
            },
        );

        model_pricing.insert(
            "gpt-4-turbo".to_string(),
            ModelPricing {
                provider: "openai".to_string(),
                input_cost_per_1k: 0.01,
                output_cost_per_1k: 0.03,
                minimum_cost: 0.0,
                quality_tier: QualityTier::Premium,
            },
        );

        model_pricing.insert(
            "gpt-3.5-turbo".to_string(),
            ModelPricing {
                provider: "openai".to_string(),
                input_cost_per_1k: 0.0005,
                output_cost_per_1k: 0.0015,
                minimum_cost: 0.0,
                quality_tier: QualityTier::Standard,
            },
        );

        // Anthropic models
        model_pricing.insert(
            "claude-3-opus".to_string(),
            ModelPricing {
                provider: "anthropic".to_string(),
                input_cost_per_1k: 0.015,
                output_cost_per_1k: 0.075,
                minimum_cost: 0.0,
                quality_tier: QualityTier::Premium,
            },
        );

        model_pricing.insert(
            "claude-3-sonnet".to_string(),
            ModelPricing {
                provider: "anthropic".to_string(),
                input_cost_per_1k: 0.003,
                output_cost_per_1k: 0.015,
                minimum_cost: 0.0,
                quality_tier: QualityTier::Standard,
            },
        );

        model_pricing.insert(
            "claude-3-haiku".to_string(),
            ModelPricing {
                provider: "anthropic".to_string(),
                input_cost_per_1k: 0.00025,
                output_cost_per_1k: 0.00125,
                minimum_cost: 0.0,
                quality_tier: QualityTier::Economy,
            },
        );

        Self {
            model_pricing,
            cost_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn analyze_spending(&self, activities: &[ActivityLog]) -> Result<SpendAnalysis> {
        let mut total_spend = 0.0;
        let mut spend_by_model: HashMap<String, ModelSpend> = HashMap::new();
        let mut spend_by_user: HashMap<String, f64> = HashMap::new();
        let mut spend_by_department: HashMap<String, f64> = HashMap::new();
        let mut total_tokens = 0u64;

        for activity in activities {
            if let Some(cost) = activity.cost {
                total_spend += cost;

                // By model
                if let Some(model) = &activity.model_used {
                    let model_spend = spend_by_model.entry(model.clone()).or_insert(ModelSpend {
                        total_cost: 0.0,
                        query_count: 0,
                        avg_cost_per_query: 0.0,
                        input_tokens: 0,
                        output_tokens: 0,
                        efficiency_score: 0.0,
                    });

                    model_spend.total_cost += cost;
                    model_spend.query_count += 1;

                    // Extract token counts from metadata if available
                    if let Some(metadata) = &activity.metadata {
                        if let Ok(meta) =
                            serde_json::from_str::<HashMap<String, serde_json::Value>>(metadata)
                        {
                            if let Some(input) = meta.get("input_tokens").and_then(|v| v.as_u64()) {
                                model_spend.input_tokens += input;
                                total_tokens += input;
                            }
                            if let Some(output) = meta.get("output_tokens").and_then(|v| v.as_u64())
                            {
                                model_spend.output_tokens += output;
                                total_tokens += output;
                            }
                        }
                    }
                }

                // By user
                if let Some(user) = &activity.user_id {
                    *spend_by_user.entry(user.clone()).or_insert(0.0) += cost;
                }

                // By department (placeholder - would extract from user metadata)
                let department = "Engineering".to_string();
                *spend_by_department.entry(department).or_insert(0.0) += cost;
            }
        }

        // Calculate averages
        for model_spend in spend_by_model.values_mut() {
            if model_spend.query_count > 0 {
                model_spend.avg_cost_per_query =
                    model_spend.total_cost / model_spend.query_count as f64;
            }
        }

        let total_cost_per_token = if total_tokens > 0 {
            total_spend / (total_tokens as f64 / 1000.0)
        } else {
            0.0
        };

        Ok(SpendAnalysis {
            total_spend,
            spend_by_model,
            spend_by_user,
            spend_by_department,
            total_cost_per_token,
            token_utilization: 0.75, // Placeholder
        })
    }

    fn calculate_model_efficiency(
        &self,
        model: &str,
        activities: &[ActivityLog],
    ) -> Result<ModelEfficiency> {
        let mut total_cost = 0.0;
        let mut query_count = 0;

        for activity in activities {
            if let Some(cost) = activity.cost {
                total_cost += cost;
                query_count += 1;
            }
        }

        let avg_cost = if query_count > 0 {
            total_cost / query_count as f64
        } else {
            0.0
        };
        let efficiency_score = if avg_cost > 0.0 {
            100.0 / (1.0 + avg_cost * 10.0)
        } else {
            0.0
        };

        Ok(ModelEfficiency {
            model_name: model.to_string(),
            efficiency_score,
            quality_score: self.estimate_quality_score(model),
            cost_effectiveness: efficiency_score * 0.8,
            recommended_use_cases: self.get_recommended_use_cases(model),
            alternatives: Vec::new(),
        })
    }

    fn estimate_quality_score(&self, model: &str) -> f64 {
        self.model_pricing
            .get(model)
            .map(|p| match p.quality_tier {
                QualityTier::Premium => 95.0,
                QualityTier::Standard => 85.0,
                QualityTier::Economy => 75.0,
            })
            .unwrap_or(70.0)
    }

    fn get_recommended_use_cases(&self, model: &str) -> Vec<String> {
        self.model_pricing
            .get(model)
            .map(|p| match p.quality_tier {
                QualityTier::Premium => vec![
                    "Complex reasoning".to_string(),
                    "Creative tasks".to_string(),
                    "Critical analysis".to_string(),
                ],
                QualityTier::Standard => vec![
                    "General queries".to_string(),
                    "Summarization".to_string(),
                    "Data processing".to_string(),
                ],
                QualityTier::Economy => vec![
                    "Simple queries".to_string(),
                    "Classification".to_string(),
                    "Basic extraction".to_string(),
                ],
            })
            .unwrap_or_else(|| vec!["General use".to_string()])
    }
}

impl OptimizationEngine {
    fn new() -> Self {
        let mut optimization_rules = Vec::new();

        // Model selection optimization
        optimization_rules.push(OptimizationRule {
            name: "Model Right-Sizing".to_string(),
            condition: Box::new(|insights| {
                insights
                    .spend_by_model
                    .values()
                    .any(|s| s.avg_cost_per_query > 0.15)
            }),
            generator: Box::new(|insights| {
                let high_cost_models: Vec<_> = insights
                    .spend_by_model
                    .iter()
                    .filter(|(_, s)| s.avg_cost_per_query > 0.15)
                    .collect();

                let potential_savings = high_cost_models
                    .iter()
                    .map(|(_, s)| s.total_cost * 0.4)
                    .sum();

                CostOptimization {
                    optimization_type: OptimizationType::ModelSelection,
                    title: "Optimize Model Selection".to_string(),
                    description: "Use more cost-effective models for simpler queries".to_string(),
                    potential_savings,
                    implementation_effort: EffortLevel::Medium,
                    impact_on_quality: QualityImpact::Minimal,
                    priority: OptimizationPriority::High,
                    action_items: vec![
                        "Implement query complexity analysis".to_string(),
                        "Create model routing logic".to_string(),
                        "Test quality impact on sample queries".to_string(),
                    ],
                }
            }),
        });

        // Caching optimization
        optimization_rules.push(OptimizationRule {
            name: "Semantic Caching".to_string(),
            condition: Box::new(|insights| insights.efficiency_metrics.cache_hit_rate < 0.2),
            generator: Box::new(|insights| {
                let potential_savings = insights.total_spend * 0.15;

                CostOptimization {
                    optimization_type: OptimizationType::CachingStrategy,
                    title: "Implement Semantic Caching".to_string(),
                    description: "Cache similar queries to reduce API calls".to_string(),
                    potential_savings,
                    implementation_effort: EffortLevel::Low,
                    impact_on_quality: QualityImpact::None,
                    priority: OptimizationPriority::High,
                    action_items: vec![
                        "Deploy semantic similarity matching".to_string(),
                        "Set up cache infrastructure".to_string(),
                        "Configure TTL policies".to_string(),
                    ],
                }
            }),
        });

        let strategy_templates = HashMap::new();

        Self {
            optimization_rules,
            strategy_templates,
        }
    }

    fn find_opportunities(
        &self,
        analysis: &SpendAnalysis,
        efficiency: &EfficiencyMetrics,
    ) -> Result<Vec<CostOptimization>> {
        let insights = CostInsights {
            total_spend: analysis.total_spend,
            spend_by_model: analysis.spend_by_model.clone(),
            spend_by_user: analysis.spend_by_user.clone(),
            spend_by_department: analysis.spend_by_department.clone(),
            efficiency_metrics: efficiency.clone(),
            optimization_opportunities: Vec::new(),
            budget_status: BudgetStatus {
                current_spend: analysis.total_spend,
                budget_limit: 1000.0,
                utilization_percent: (analysis.total_spend / 1000.0) * 100.0,
                days_remaining: 15,
                projected_spend: analysis.total_spend * 2.0,
                status: BudgetHealth::Healthy,
                alerts: Vec::new(),
            },
            spending_patterns: Vec::new(),
        };

        let mut opportunities = Vec::new();

        for rule in &self.optimization_rules {
            if (rule.condition)(&insights) {
                opportunities.push((rule.generator)(&insights));
            }
        }

        // Sort by priority and potential savings
        opportunities.sort_by(|a, b| {
            a.priority.cmp(&b.priority).then(
                b.potential_savings
                    .partial_cmp(&a.potential_savings)
                    .unwrap(),
            )
        });

        Ok(opportunities)
    }
}

impl BudgetMonitor {
    fn new() -> Self {
        let alert_thresholds = vec![
            AlertThreshold {
                percentage: 50.0,
                severity: AlertSeverity::Info,
            },
            AlertThreshold {
                percentage: 80.0,
                severity: AlertSeverity::Warning,
            },
            AlertThreshold {
                percentage: 100.0,
                severity: AlertSeverity::Critical,
            },
        ];

        Self {
            budgets: Arc::new(RwLock::new(HashMap::new())),
            alert_thresholds,
        }
    }

    async fn check_budget_status(&self, analysis: &SpendAnalysis) -> Result<BudgetStatus> {
        let budgets = self.budgets.read().await;

        // Use default budget if none set
        let budget = budgets.get("default").cloned().unwrap_or(Budget {
            name: "default".to_string(),
            limit: 1000.0,
            period: BudgetPeriod::Monthly,
            current_spend: analysis.total_spend,
            alerts_enabled: true,
        });

        let utilization_percent = (analysis.total_spend / budget.limit) * 100.0;

        let status = match utilization_percent {
            p if p >= 100.0 => BudgetHealth::Exceeded,
            p if p >= 90.0 => BudgetHealth::AtRisk,
            p if p >= 70.0 => BudgetHealth::Warning,
            _ => BudgetHealth::Healthy,
        };

        let mut alerts = Vec::new();

        // Check alert thresholds
        for threshold in &self.alert_thresholds {
            if utilization_percent >= threshold.percentage && budget.alerts_enabled {
                alerts.push(BudgetAlert {
                    alert_type: if utilization_percent >= 100.0 {
                        AlertType::ExceededLimit
                    } else {
                        AlertType::ApproachingLimit
                    },
                    severity: threshold.severity,
                    message: format!("Budget utilization at {:.1}%", utilization_percent),
                    current_spend: analysis.total_spend,
                    budget_limit: budget.limit,
                    projected_overrun: if utilization_percent > 80.0 {
                        Some((analysis.total_spend * 1.25) - budget.limit)
                    } else {
                        None
                    },
                    triggered_at: Utc::now(),
                });
            }
        }

        Ok(BudgetStatus {
            current_spend: analysis.total_spend,
            budget_limit: budget.limit,
            utilization_percent,
            days_remaining: 15,                          // Placeholder
            projected_spend: analysis.total_spend * 1.5, // Simple projection
            status,
            alerts,
        })
    }

    async fn set_budget(&self, limit: f64, period: &str) -> Result<()> {
        let budget_period = match period {
            "daily" => BudgetPeriod::Daily,
            "weekly" => BudgetPeriod::Weekly,
            "monthly" => BudgetPeriod::Monthly,
            "quarterly" => BudgetPeriod::Quarterly,
            "annual" => BudgetPeriod::Annual,
            _ => return Err(anyhow::anyhow!("Invalid budget period")),
        };

        let budget = Budget {
            name: "default".to_string(),
            limit,
            period: budget_period,
            current_spend: 0.0,
            alerts_enabled: true,
        };

        let mut budgets = self.budgets.write().await;
        budgets.insert("default".to_string(), budget);

        Ok(())
    }
}

impl AllocationTracker {
    fn new() -> Self {
        let allocation_rules = HashMap::new();
        let cost_centers = HashMap::new();

        Self {
            allocation_rules,
            cost_centers,
        }
    }
}

// Implement comparison for optimization priority
impl Ord for OptimizationPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Critical, Self::Critical) => std::cmp::Ordering::Equal,
            (Self::Critical, _) => std::cmp::Ordering::Less,
            (_, Self::Critical) => std::cmp::Ordering::Greater,
            (Self::High, Self::High) => std::cmp::Ordering::Equal,
            (Self::High, _) => std::cmp::Ordering::Less,
            (_, Self::High) => std::cmp::Ordering::Greater,
            (Self::Medium, Self::Medium) => std::cmp::Ordering::Equal,
            (Self::Medium, Self::Low) => std::cmp::Ordering::Less,
            (Self::Low, Self::Medium) => std::cmp::Ordering::Greater,
            (Self::Low, Self::Low) => std::cmp::Ordering::Equal,
        }
    }
}

impl PartialOrd for OptimizationPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cost_intelligence_creation() -> Result<()> {
        let config = Arc::new(RwLock::new(AdvancedAnalyticsConfig::default()));
        let intelligence = CostIntelligence::new(config).await?;

        assert!(Arc::strong_count(&intelligence.cost_analyzer) > 0);

        Ok(())
    }

    #[test]
    fn test_efficiency_calculation() {
        let spend = ModelSpend {
            total_cost: 10.0,
            query_count: 100,
            avg_cost_per_query: 0.10,
            input_tokens: 10000,
            output_tokens: 5000,
            efficiency_score: 0.0,
        };

        let config = Arc::new(RwLock::new(AdvancedAnalyticsConfig::default()));
        let intelligence = CostIntelligence {
            config,
            cost_analyzer: Arc::new(CostAnalyzer::new()),
            optimization_engine: Arc::new(OptimizationEngine::new()),
            budget_monitor: Arc::new(BudgetMonitor::new()),
            allocation_tracker: Arc::new(AllocationTracker::new()),
        };

        let score = intelligence.calculate_efficiency_score(&spend);
        assert!(score > 0.0 && score <= 100.0);
    }

    #[test]
    fn test_spending_forecast() {
        let historical = vec![
            DailySpend {
                date: "2024-01-01".to_string(),
                cost: 10.0,
            },
            DailySpend {
                date: "2024-01-02".to_string(),
                cost: 12.0,
            },
            DailySpend {
                date: "2024-01-03".to_string(),
                cost: 11.0,
            },
            DailySpend {
                date: "2024-01-04".to_string(),
                cost: 15.0,
            },
            DailySpend {
                date: "2024-01-05".to_string(),
                cost: 14.0,
            },
        ];

        let config = Arc::new(RwLock::new(AdvancedAnalyticsConfig::default()));
        let intelligence = CostIntelligence {
            config,
            cost_analyzer: Arc::new(CostAnalyzer::new()),
            optimization_engine: Arc::new(OptimizationEngine::new()),
            budget_monitor: Arc::new(BudgetMonitor::new()),
            allocation_tracker: Arc::new(AllocationTracker::new()),
        };

        let forecast = intelligence.forecast_spending(&historical, 7).unwrap();
        assert_eq!(forecast.predicted_spend.len(), 7);
        assert!(matches!(
            forecast.trend,
            SpendingTrend::Increasing | SpendingTrend::Stable
        ));
    }
}
