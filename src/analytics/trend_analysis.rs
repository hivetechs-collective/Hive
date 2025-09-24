//! ML-Powered Trend Analysis with Predictive Capabilities
//!
//! This module provides:
//! - Time series analysis with multiple algorithms
//! - Seasonal pattern detection
//! - Anomaly detection using statistical methods
//! - Predictive modeling with confidence intervals
//! - Trend decomposition and forecasting

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::analytics::AdvancedAnalyticsConfig;
use crate::core::database::{get_database, ActivityLog};

/// Trend analyzer with ML capabilities
pub struct TrendAnalyzer {
    config: Arc<RwLock<AdvancedAnalyticsConfig>>,
    time_series_models: Arc<RwLock<HashMap<String, TimeSeriesModel>>>,
    anomaly_detector: Arc<AnomalyDetector>,
    seasonal_analyzer: Arc<SeasonalAnalyzer>,
    predictor: Arc<Predictor>,
}

/// Time series model for a specific metric
#[derive(Debug, Clone)]
pub struct TimeSeriesModel {
    pub metric_name: String,
    pub data_points: VecDeque<DataPoint>,
    pub model_type: ModelType,
    pub parameters: ModelParameters,
    pub last_updated: DateTime<Utc>,
}

/// Data point in time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

/// Types of time series models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    MovingAverage,
    ExponentialSmoothing,
    ARIMA,
    Prophet,
    LinearRegression,
    NeuralNetwork,
}

/// Model parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    pub window_size: usize,
    pub alpha: f64, // Smoothing parameter
    pub beta: f64,  // Trend parameter
    pub gamma: f64, // Seasonal parameter
    pub seasonality_period: usize,
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            window_size: 30,
            alpha: 0.3,
            beta: 0.1,
            gamma: 0.1,
            seasonality_period: 7, // Weekly seasonality
        }
    }
}

/// Trend prediction with confidence intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPrediction {
    pub metric: String,
    pub predictions: Vec<PredictionPoint>,
    pub confidence_level: f64,
    pub model_accuracy: f64,
    pub trend_direction: TrendDirection,
    pub seasonality: Option<SeasonalPattern>,
}

/// Single prediction point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionPoint {
    pub timestamp: DateTime<Utc>,
    pub predicted_value: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub confidence: f64,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    StronglyIncreasing,
    Increasing,
    Stable,
    Decreasing,
    StronglyDecreasing,
}

/// Seasonal pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    pub pattern_type: SeasonalityType,
    pub period: usize,
    pub amplitude: f64,
    pub phase_shift: f64,
    pub strength: f64,
}

/// Types of seasonality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeasonalityType {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom(usize),
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub timestamp: DateTime<Utc>,
    pub metric: String,
    pub actual_value: f64,
    pub expected_value: f64,
    pub deviation: f64,
    pub severity: AnomalySeverity,
    pub explanation: String,
}

/// Anomaly severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Forecast horizon options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForecastHorizon {
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Custom(i64),
}

impl ForecastHorizon {
    pub fn to_duration(&self) -> Duration {
        match self {
            Self::Hour => Duration::hours(1),
            Self::Day => Duration::days(1),
            Self::Week => Duration::weeks(1),
            Self::Month => Duration::days(30),
            Self::Quarter => Duration::days(90),
            Self::Custom(days) => Duration::days(*days),
        }
    }
}

/// Confidence interval for predictions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub level: f64,
    pub lower_multiplier: f64,
    pub upper_multiplier: f64,
}

impl Default for ConfidenceInterval {
    fn default() -> Self {
        Self {
            level: 0.95,
            lower_multiplier: 1.96,
            upper_multiplier: 1.96,
        }
    }
}

/// Trend summary for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendSummary {
    pub key_trends: Vec<KeyTrend>,
    pub predictions: Vec<TrendPrediction>,
    pub anomalies: Vec<AnomalyDetection>,
    pub seasonal_patterns: Vec<SeasonalPattern>,
    pub insights: Vec<TrendInsight>,
}

/// Key trend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyTrend {
    pub metric: String,
    pub current_value: f64,
    pub previous_value: f64,
    pub change_percent: f64,
    pub trend_direction: TrendDirection,
    pub momentum: f64,
}

/// Trend insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendInsight {
    pub title: String,
    pub description: String,
    pub metrics_affected: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub confidence: f64,
}

/// Anomaly detector
struct AnomalyDetector {
    z_score_threshold: f64,
    iqr_multiplier: f64,
    min_data_points: usize,
}

/// Seasonal pattern analyzer
struct SeasonalAnalyzer {
    fft_enabled: bool,
    autocorrelation_lags: Vec<usize>,
}

/// Predictor engine
struct Predictor {
    models: HashMap<ModelType, Box<dyn PredictionModel>>,
}

/// Trait for prediction models
trait PredictionModel: Send + Sync {
    fn predict(
        &self,
        data: &[DataPoint],
        horizon: ForecastHorizon,
        confidence: ConfidenceInterval,
    ) -> Result<Vec<PredictionPoint>>;

    fn accuracy(&self, data: &[DataPoint]) -> f64;
}

impl TrendAnalyzer {
    /// Create a new trend analyzer
    pub async fn new(config: Arc<RwLock<AdvancedAnalyticsConfig>>) -> Result<Self> {
        info!("Initializing ML-powered trend analyzer");

        let time_series_models = Arc::new(RwLock::new(HashMap::new()));

        let anomaly_detector = Arc::new(AnomalyDetector {
            z_score_threshold: 3.0,
            iqr_multiplier: 1.5,
            min_data_points: 20,
        });

        let seasonal_analyzer = Arc::new(SeasonalAnalyzer {
            fft_enabled: true,
            autocorrelation_lags: vec![1, 7, 30, 90, 365],
        });

        let predictor = Arc::new(Predictor::new());

        Ok(Self {
            config,
            time_series_models,
            anomaly_detector,
            seasonal_analyzer,
            predictor,
        })
    }

    /// Analyze trends for a specific metric
    pub async fn analyze_metric(
        &self,
        metric: &str,
        horizon: Option<ForecastHorizon>,
    ) -> Result<TrendPrediction> {
        debug!("Analyzing trends for metric: {}", metric);

        // Get or create time series model
        let mut models = self.time_series_models.write().await;
        let model = models
            .entry(metric.to_string())
            .or_insert_with(|| TimeSeriesModel {
                metric_name: metric.to_string(),
                data_points: VecDeque::new(),
                model_type: ModelType::ExponentialSmoothing,
                parameters: ModelParameters::default(),
                last_updated: Utc::now(),
            });

        // Update model with latest data
        self.update_model_data(model).await?;

        // Detect trend direction
        let trend_direction = self.detect_trend_direction(&model.data_points)?;

        // Detect seasonality
        let seasonality = self
            .seasonal_analyzer
            .detect_seasonality(&model.data_points)?;

        // Generate predictions
        let config = self.config.read().await;
        let forecast_horizon = horizon.unwrap_or(ForecastHorizon::Week);
        let confidence = ConfidenceInterval {
            level: config.confidence_level,
            ..Default::default()
        };

        let predictions = self.predictor.generate_predictions(
            &model.data_points,
            model.model_type,
            forecast_horizon,
            confidence,
        )?;

        // Calculate model accuracy
        let model_accuracy = self.calculate_model_accuracy(model, &predictions)?;

        Ok(TrendPrediction {
            metric: metric.to_string(),
            predictions,
            confidence_level: confidence.level,
            model_accuracy,
            trend_direction,
            seasonality,
        })
    }

    /// Detect anomalies in recent data
    pub async fn detect_anomalies(&self, metric: &str) -> Result<Vec<AnomalyDetection>> {
        debug!("Detecting anomalies for metric: {}", metric);

        let models = self.time_series_models.read().await;
        let model = models
            .get(metric)
            .ok_or_else(|| anyhow::anyhow!("No model found for metric: {}", metric))?;

        self.anomaly_detector
            .detect_anomalies(&model.data_points, metric)
    }

    /// Get current trends summary
    pub async fn get_current_trends(&self) -> Result<TrendSummary> {
        let mut key_trends = Vec::new();
        let mut predictions = Vec::new();
        let mut anomalies = Vec::new();
        let mut seasonal_patterns = Vec::new();

        // Analyze all tracked metrics
        let models = self.time_series_models.read().await;
        for (metric, model) in models.iter() {
            // Calculate key trend
            if let Some(trend) = self.calculate_key_trend(metric, &model.data_points)? {
                key_trends.push(trend);
            }

            // Generate prediction
            if let Ok(prediction) = self
                .analyze_metric(metric, Some(ForecastHorizon::Week))
                .await
            {
                predictions.push(prediction);
            }

            // Detect anomalies
            if let Ok(metric_anomalies) = self.detect_anomalies(metric).await {
                anomalies.extend(metric_anomalies);
            }

            // Detect seasonal patterns
            if let Some(pattern) = self
                .seasonal_analyzer
                .detect_seasonality(&model.data_points)?
            {
                seasonal_patterns.push(pattern);
            }
        }

        // Generate insights
        let insights = self.generate_insights(&key_trends, &predictions, &anomalies)?;

        Ok(TrendSummary {
            key_trends,
            predictions,
            anomalies,
            seasonal_patterns,
            insights,
        })
    }

    /// Reload configuration
    pub async fn reload_config(&self) -> Result<()> {
        debug!("Reloading trend analyzer configuration");
        Ok(())
    }

    // Private helper methods

    async fn update_model_data(&self, model: &mut TimeSeriesModel) -> Result<()> {
        // Fetch recent data from database
        let db = get_database().await?;
        let activities = ActivityLog::get_recent(1000).await?;

        // Extract metric data points
        for activity in activities {
            // Example: Extract query count metric
            if model.metric_name == "query_count" && activity.event_type == "query_complete" {
                model.data_points.push_back(DataPoint {
                    timestamp: activity.created_at.parse()?,
                    value: 1.0,
                    metadata: HashMap::new(),
                });
            }
            // Add more metric extraction logic here
        }

        // Limit data points to prevent memory growth
        while model.data_points.len() > 10000 {
            model.data_points.pop_front();
        }

        model.last_updated = Utc::now();
        Ok(())
    }

    fn detect_trend_direction(&self, data: &VecDeque<DataPoint>) -> Result<TrendDirection> {
        if data.len() < 10 {
            return Ok(TrendDirection::Stable);
        }

        // Calculate linear regression slope
        let n = data.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, point) in data.iter().enumerate() {
            let x = i as f64;
            let y = point.value;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let avg_value = sum_y / n;
        let normalized_slope = slope / avg_value;

        Ok(match normalized_slope {
            s if s > 0.1 => TrendDirection::StronglyIncreasing,
            s if s > 0.02 => TrendDirection::Increasing,
            s if s < -0.1 => TrendDirection::StronglyDecreasing,
            s if s < -0.02 => TrendDirection::Decreasing,
            _ => TrendDirection::Stable,
        })
    }

    fn calculate_model_accuracy(
        &self,
        model: &TimeSeriesModel,
        predictions: &[PredictionPoint],
    ) -> Result<f64> {
        // Simple accuracy based on historical performance
        // In production, this would use backtesting
        Ok(0.85) // Placeholder
    }

    fn calculate_key_trend(
        &self,
        metric: &str,
        data: &VecDeque<DataPoint>,
    ) -> Result<Option<KeyTrend>> {
        if data.len() < 2 {
            return Ok(None);
        }

        let current = data.back().unwrap();
        let previous = data.iter().rev().nth(10).unwrap_or(data.front().unwrap());

        let change_percent = if previous.value != 0.0 {
            ((current.value - previous.value) / previous.value) * 100.0
        } else {
            0.0
        };

        let trend_direction = self.detect_trend_direction(data)?;

        // Calculate momentum (rate of change)
        let momentum = if data.len() >= 20 {
            let recent_avg = data.iter().rev().take(5).map(|p| p.value).sum::<f64>() / 5.0;
            let older_avg = data
                .iter()
                .rev()
                .skip(15)
                .take(5)
                .map(|p| p.value)
                .sum::<f64>()
                / 5.0;
            if older_avg != 0.0 {
                (recent_avg - older_avg) / older_avg
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(Some(KeyTrend {
            metric: metric.to_string(),
            current_value: current.value,
            previous_value: previous.value,
            change_percent,
            trend_direction,
            momentum,
        }))
    }

    fn generate_insights(
        &self,
        trends: &[KeyTrend],
        predictions: &[TrendPrediction],
        anomalies: &[AnomalyDetection],
    ) -> Result<Vec<TrendInsight>> {
        let mut insights = Vec::new();

        // Insight: Rapid growth detection
        for trend in trends {
            if trend.change_percent > 50.0 && trend.momentum > 0.2 {
                insights.push(TrendInsight {
                    title: format!("Rapid Growth in {}", trend.metric),
                    description: format!(
                        "{} has increased by {:.1}% with strong momentum ({:.1})",
                        trend.metric, trend.change_percent, trend.momentum
                    ),
                    metrics_affected: vec![trend.metric.clone()],
                    recommended_actions: vec![
                        "Monitor capacity limits".to_string(),
                        "Consider scaling resources".to_string(),
                        "Review cost implications".to_string(),
                    ],
                    confidence: 0.9,
                });
            }
        }

        // Insight: Anomaly patterns
        if anomalies.len() > 5 {
            let critical_count = anomalies
                .iter()
                .filter(|a| matches!(a.severity, AnomalySeverity::Critical))
                .count();

            if critical_count > 2 {
                insights.push(TrendInsight {
                    title: "Multiple Critical Anomalies Detected".to_string(),
                    description: format!(
                        "Detected {} critical anomalies in recent data, indicating potential system issues",
                        critical_count
                    ),
                    metrics_affected: anomalies.iter().map(|a| a.metric.clone()).collect(),
                    recommended_actions: vec![
                        "Investigate system health".to_string(),
                        "Check for configuration changes".to_string(),
                        "Review recent deployments".to_string(),
                    ],
                    confidence: 0.95,
                });
            }
        }

        Ok(insights)
    }
}

impl TrendSummary {
    /// Format as markdown
    pub fn format_markdown(&self) -> Result<String> {
        let mut output = String::new();

        // Key trends
        if !self.key_trends.is_empty() {
            output.push_str("### Key Trends\n\n");
            for trend in &self.key_trends {
                output.push_str(&format!(
                    "- **{}**: {:.2} ({:+.1}% change, {:?} trend)\n",
                    trend.metric, trend.current_value, trend.change_percent, trend.trend_direction
                ));
            }
            output.push_str("\n");
        }

        // Predictions
        if !self.predictions.is_empty() {
            output.push_str("### Predictions\n\n");
            for pred in &self.predictions {
                output.push_str(&format!(
                    "- **{}**: Expected {:?} trend with {:.0}% confidence\n",
                    pred.metric,
                    pred.trend_direction,
                    pred.confidence_level * 100.0
                ));
            }
            output.push_str("\n");
        }

        // Anomalies
        if !self.anomalies.is_empty() {
            output.push_str("### Anomalies Detected\n\n");
            for anomaly in &self.anomalies {
                output.push_str(&format!(
                    "- **{}** at {}: {:?} severity - {}\n",
                    anomaly.metric,
                    anomaly.timestamp.format("%Y-%m-%d %H:%M"),
                    anomaly.severity,
                    anomaly.explanation
                ));
            }
            output.push_str("\n");
        }

        // Insights
        if !self.insights.is_empty() {
            output.push_str("### Insights\n\n");
            for insight in &self.insights {
                output.push_str(&format!("**{}**\n", insight.title));
                output.push_str(&format!("{}\n", insight.description));
                if !insight.recommended_actions.is_empty() {
                    output.push_str("Recommended Actions:\n");
                    for action in &insight.recommended_actions {
                        output.push_str(&format!("- {}\n", action));
                    }
                }
                output.push_str("\n");
            }
        }

        Ok(output)
    }
}

impl SeasonalAnalyzer {
    fn detect_seasonality(&self, data: &VecDeque<DataPoint>) -> Result<Option<SeasonalPattern>> {
        if data.len() < 100 {
            return Ok(None);
        }

        // Simple seasonality detection using autocorrelation
        // In production, this would use FFT or more sophisticated methods

        let values: Vec<f64> = data.iter().map(|p| p.value).collect();
        let mut max_correlation = 0.0;
        let mut best_period = 0;

        for &lag in &self.autocorrelation_lags {
            if lag >= values.len() {
                continue;
            }

            let correlation = self.calculate_autocorrelation(&values, lag);
            if correlation > max_correlation && correlation > 0.7 {
                max_correlation = correlation;
                best_period = lag;
            }
        }

        if best_period > 0 {
            let pattern_type = match best_period {
                1 => SeasonalityType::Daily,
                7 => SeasonalityType::Weekly,
                30 => SeasonalityType::Monthly,
                90 => SeasonalityType::Quarterly,
                365 => SeasonalityType::Yearly,
                _ => SeasonalityType::Custom(best_period),
            };

            Ok(Some(SeasonalPattern {
                pattern_type,
                period: best_period,
                amplitude: self.calculate_amplitude(&values, best_period),
                phase_shift: 0.0, // Simplified
                strength: max_correlation,
            }))
        } else {
            Ok(None)
        }
    }

    fn calculate_autocorrelation(&self, values: &[f64], lag: usize) -> f64 {
        if lag >= values.len() {
            return 0.0;
        }

        let n = values.len() - lag;
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

        let mut numerator = 0.0;
        let mut denominator1 = 0.0;
        let mut denominator2 = 0.0;

        for i in 0..n {
            let x = values[i] - mean;
            let y = values[i + lag] - mean;
            numerator += x * y;
            denominator1 += x * x;
            denominator2 += y * y;
        }

        if denominator1 * denominator2 > 0.0 {
            numerator / (denominator1 * denominator2).sqrt()
        } else {
            0.0
        }
    }

    fn calculate_amplitude(&self, values: &[f64], period: usize) -> f64 {
        // Simplified amplitude calculation
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        (max_val - min_val) / 2.0
    }
}

impl AnomalyDetector {
    fn detect_anomalies(
        &self,
        data: &VecDeque<DataPoint>,
        metric: &str,
    ) -> Result<Vec<AnomalyDetection>> {
        if data.len() < self.min_data_points {
            return Ok(Vec::new());
        }

        let values: Vec<f64> = data.iter().map(|p| p.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let mut anomalies = Vec::new();

        for (i, point) in data.iter().enumerate() {
            let z_score = if std_dev > 0.0 {
                (point.value - mean).abs() / std_dev
            } else {
                0.0
            };

            if z_score > self.z_score_threshold {
                let severity = match z_score {
                    z if z > 5.0 => AnomalySeverity::Critical,
                    z if z > 4.0 => AnomalySeverity::High,
                    z if z > 3.5 => AnomalySeverity::Medium,
                    _ => AnomalySeverity::Low,
                };

                anomalies.push(AnomalyDetection {
                    timestamp: point.timestamp,
                    metric: metric.to_string(),
                    actual_value: point.value,
                    expected_value: mean,
                    deviation: z_score * std_dev,
                    severity,
                    explanation: format!(
                        "Value deviates {:.1} standard deviations from the mean",
                        z_score
                    ),
                });
            }
        }

        Ok(anomalies)
    }
}

impl Predictor {
    fn new() -> Self {
        let mut models: HashMap<ModelType, Box<dyn PredictionModel>> = HashMap::new();

        // Add prediction models
        models.insert(
            ModelType::MovingAverage,
            Box::new(MovingAverageModel::new()),
        );
        models.insert(
            ModelType::ExponentialSmoothing,
            Box::new(ExponentialSmoothingModel::new()),
        );
        models.insert(
            ModelType::LinearRegression,
            Box::new(LinearRegressionModel::new()),
        );

        Self { models }
    }

    fn generate_predictions(
        &self,
        data: &VecDeque<DataPoint>,
        model_type: ModelType,
        horizon: ForecastHorizon,
        confidence: ConfidenceInterval,
    ) -> Result<Vec<PredictionPoint>> {
        let model = self
            .models
            .get(&model_type)
            .ok_or_else(|| anyhow::anyhow!("Model type not supported: {:?}", model_type))?;

        let data_vec: Vec<DataPoint> = data.iter().cloned().collect();
        model.predict(&data_vec, horizon, confidence)
    }
}

// Simple prediction model implementations

struct MovingAverageModel {
    window_size: usize,
}

impl MovingAverageModel {
    fn new() -> Self {
        Self { window_size: 7 }
    }
}

impl PredictionModel for MovingAverageModel {
    fn predict(
        &self,
        data: &[DataPoint],
        horizon: ForecastHorizon,
        confidence: ConfidenceInterval,
    ) -> Result<Vec<PredictionPoint>> {
        if data.len() < self.window_size {
            return Ok(Vec::new());
        }

        // Calculate moving average
        let recent_values: Vec<f64> = data
            .iter()
            .rev()
            .take(self.window_size)
            .map(|p| p.value)
            .collect();

        let avg = recent_values.iter().sum::<f64>() / recent_values.len() as f64;
        let std_dev = {
            let variance = recent_values.iter().map(|v| (v - avg).powi(2)).sum::<f64>()
                / recent_values.len() as f64;
            variance.sqrt()
        };

        // Generate predictions
        let mut predictions = Vec::new();
        let last_timestamp = data.last().unwrap().timestamp;
        let duration = horizon.to_duration();

        for i in 1..=7 {
            let timestamp = last_timestamp + duration * i as i32;
            let confidence_width = std_dev * confidence.lower_multiplier * (i as f64).sqrt();

            predictions.push(PredictionPoint {
                timestamp,
                predicted_value: avg,
                lower_bound: avg - confidence_width,
                upper_bound: avg + confidence_width,
                confidence: confidence.level,
            });
        }

        Ok(predictions)
    }

    fn accuracy(&self, data: &[DataPoint]) -> f64 {
        0.75 // Simplified
    }
}

struct ExponentialSmoothingModel {
    alpha: f64,
}

impl ExponentialSmoothingModel {
    fn new() -> Self {
        Self { alpha: 0.3 }
    }
}

impl PredictionModel for ExponentialSmoothingModel {
    fn predict(
        &self,
        data: &[DataPoint],
        horizon: ForecastHorizon,
        confidence: ConfidenceInterval,
    ) -> Result<Vec<PredictionPoint>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Simple exponential smoothing
        let mut smoothed = data[0].value;
        for point in data.iter().skip(1) {
            smoothed = self.alpha * point.value + (1.0 - self.alpha) * smoothed;
        }

        // Calculate prediction error
        let mut errors = Vec::new();
        let mut current_smoothed = data[0].value;
        for point in data.iter().skip(1) {
            errors.push((point.value - current_smoothed).abs());
            current_smoothed = self.alpha * point.value + (1.0 - self.alpha) * current_smoothed;
        }

        let error_std = if !errors.is_empty() {
            let mean_error = errors.iter().sum::<f64>() / errors.len() as f64;
            let variance =
                errors.iter().map(|e| (e - mean_error).powi(2)).sum::<f64>() / errors.len() as f64;
            variance.sqrt()
        } else {
            smoothed * 0.1 // Default to 10% of value
        };

        // Generate predictions
        let mut predictions = Vec::new();
        let last_timestamp = data.last().unwrap().timestamp;
        let duration = horizon.to_duration();

        for i in 1..=7 {
            let timestamp = last_timestamp + duration * i as i32;
            let confidence_width = error_std * confidence.lower_multiplier * (i as f64).sqrt();

            predictions.push(PredictionPoint {
                timestamp,
                predicted_value: smoothed,
                lower_bound: smoothed - confidence_width,
                upper_bound: smoothed + confidence_width,
                confidence: confidence.level,
            });
        }

        Ok(predictions)
    }

    fn accuracy(&self, data: &[DataPoint]) -> f64 {
        0.82 // Simplified
    }
}

struct LinearRegressionModel;

impl LinearRegressionModel {
    fn new() -> Self {
        Self
    }
}

impl PredictionModel for LinearRegressionModel {
    fn predict(
        &self,
        data: &[DataPoint],
        horizon: ForecastHorizon,
        confidence: ConfidenceInterval,
    ) -> Result<Vec<PredictionPoint>> {
        if data.len() < 3 {
            return Ok(Vec::new());
        }

        // Simple linear regression
        let n = data.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, point) in data.iter().enumerate() {
            let x = i as f64;
            let y = point.value;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate residuals for confidence intervals
        let mut residuals = Vec::new();
        for (i, point) in data.iter().enumerate() {
            let predicted = slope * i as f64 + intercept;
            residuals.push((point.value - predicted).powi(2));
        }

        let mse = residuals.iter().sum::<f64>() / residuals.len() as f64;
        let rmse = mse.sqrt();

        // Generate predictions
        let mut predictions = Vec::new();
        let last_timestamp = data.last().unwrap().timestamp;
        let duration = horizon.to_duration();

        for i in 1..=7 {
            let timestamp = last_timestamp + duration * i as i32;
            let x = data.len() as f64 + i as f64;
            let predicted = slope * x + intercept;
            let confidence_width = rmse
                * confidence.lower_multiplier
                * (1.0 + 1.0 / n + (x - sum_x / n).powi(2) / (sum_x2 - sum_x.powi(2) / n)).sqrt();

            predictions.push(PredictionPoint {
                timestamp,
                predicted_value: predicted,
                lower_bound: predicted - confidence_width,
                upper_bound: predicted + confidence_width,
                confidence: confidence.level,
            });
        }

        Ok(predictions)
    }

    fn accuracy(&self, data: &[DataPoint]) -> f64 {
        0.78 // Simplified
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trend_analyzer_creation() -> Result<()> {
        let config = Arc::new(RwLock::new(AdvancedAnalyticsConfig::default()));
        let analyzer = TrendAnalyzer::new(config).await?;

        assert!(Arc::strong_count(&analyzer.anomaly_detector) > 0);
        assert!(Arc::strong_count(&analyzer.seasonal_analyzer) > 0);

        Ok(())
    }

    #[test]
    fn test_trend_direction_detection() {
        let analyzer = TrendAnalyzer {
            config: Arc::new(RwLock::new(AdvancedAnalyticsConfig::default())),
            time_series_models: Arc::new(RwLock::new(HashMap::new())),
            anomaly_detector: Arc::new(AnomalyDetector {
                z_score_threshold: 3.0,
                iqr_multiplier: 1.5,
                min_data_points: 20,
            }),
            seasonal_analyzer: Arc::new(SeasonalAnalyzer {
                fft_enabled: true,
                autocorrelation_lags: vec![1, 7, 30],
            }),
            predictor: Arc::new(Predictor::new()),
        };

        // Test with increasing data
        let mut increasing_data = VecDeque::new();
        for i in 0..20 {
            increasing_data.push_back(DataPoint {
                timestamp: Utc::now() + Duration::hours(i),
                value: i as f64 * 2.0,
                metadata: HashMap::new(),
            });
        }

        let direction = analyzer.detect_trend_direction(&increasing_data).unwrap();
        assert!(matches!(
            direction,
            TrendDirection::StronglyIncreasing | TrendDirection::Increasing
        ));
    }

    #[test]
    fn test_anomaly_detection() {
        let detector = AnomalyDetector {
            z_score_threshold: 3.0,
            iqr_multiplier: 1.5,
            min_data_points: 5,
        };

        let mut data = VecDeque::new();
        for i in 0..20 {
            data.push_back(DataPoint {
                timestamp: Utc::now() + Duration::hours(i),
                value: 100.0 + (i % 5) as f64,
                metadata: HashMap::new(),
            });
        }

        // Add an anomaly
        data.push_back(DataPoint {
            timestamp: Utc::now() + Duration::hours(21),
            value: 200.0, // Anomalous value
            metadata: HashMap::new(),
        });

        let anomalies = detector.detect_anomalies(&data, "test_metric").unwrap();
        assert!(!anomalies.is_empty());
        assert_eq!(anomalies[0].actual_value, 200.0);
    }
}
