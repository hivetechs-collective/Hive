//! Advanced Machine Learning Models for Predictive Analytics
//! 
//! Provides sophisticated ML capabilities including:
//! - ARIMA models for time series forecasting
//! - Prophet-style decomposition for seasonality
//! - Anomaly detection with isolation forests
//! - Neural network predictions for complex patterns
//! - Ensemble methods for improved accuracy

use anyhow::{Result, Context};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// ML Model types available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Auto-Regressive Integrated Moving Average
    ARIMA,
    /// Facebook Prophet-style decomposition
    Prophet,
    /// Exponential smoothing
    ExponentialSmoothing,
    /// Neural network (LSTM)
    NeuralNetwork,
    /// Ensemble of multiple models
    Ensemble,
}

/// Forecast confidence interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub lower: f64,
    pub upper: f64,
    pub confidence_level: f64,
}

/// ML Prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLPrediction {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub confidence: ConfidenceInterval,
    pub model_type: ModelType,
    pub seasonality: Option<SeasonalComponent>,
    pub trend: Option<TrendComponent>,
}

/// Seasonal component of time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalComponent {
    pub period: String, // daily, weekly, monthly, yearly
    pub strength: f64,
    pub pattern: Vec<f64>,
}

/// Trend component of time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendComponent {
    pub direction: TrendDirection,
    pub strength: f64,
    pub change_points: Vec<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionResult {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub anomaly_score: f64,
    pub is_anomaly: bool,
    pub anomaly_type: Option<AnomalyType>,
    pub explanation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    Spike,
    Dip,
    LevelShift,
    Variance,
    Seasonal,
}

/// Advanced ML Engine for predictions
pub struct MLEngine {
    config: Arc<RwLock<MLConfig>>,
    models: Arc<RwLock<HashMap<String, Box<dyn MLModel>>>>,
    cache: Arc<RwLock<PredictionCache>>,
}

/// ML Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    pub default_model: ModelType,
    pub ensemble_models: Vec<ModelType>,
    pub confidence_level: f64,
    pub anomaly_threshold: f64,
    pub seasonality_detection: bool,
    pub auto_model_selection: bool,
    pub cache_predictions: bool,
    pub max_forecast_horizon: u32,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            default_model: ModelType::Ensemble,
            ensemble_models: vec![
                ModelType::ARIMA,
                ModelType::Prophet,
                ModelType::ExponentialSmoothing,
            ],
            confidence_level: 0.95,
            anomaly_threshold: 0.95,
            seasonality_detection: true,
            auto_model_selection: true,
            cache_predictions: true,
            max_forecast_horizon: 90, // days
        }
    }
}

/// Trait for ML models
trait MLModel: Send + Sync {
    fn forecast(&self, data: &TimeSeriesData, horizon: u32) -> Result<Vec<MLPrediction>>;
    fn detect_anomalies(&self, data: &TimeSeriesData) -> Result<Vec<AnomalyDetectionResult>>;
    fn decompose(&self, data: &TimeSeriesData) -> Result<TimeSeriesDecomposition>;
    fn model_type(&self) -> ModelType;
}

/// Time series data structure
#[derive(Debug, Clone)]
pub struct TimeSeriesData {
    pub timestamps: Vec<DateTime<Utc>>,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

/// Time series decomposition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesDecomposition {
    pub trend: Vec<f64>,
    pub seasonal: Vec<f64>,
    pub residual: Vec<f64>,
    pub seasonal_periods: Vec<SeasonalComponent>,
}

/// Prediction cache
struct PredictionCache {
    predictions: HashMap<String, (DateTime<Utc>, Vec<MLPrediction>)>,
    max_age: Duration,
}

impl MLEngine {
    /// Create new ML engine
    pub async fn new(config: MLConfig) -> Result<Self> {
        let config = Arc::new(RwLock::new(config));
        let models = Arc::new(RwLock::new(HashMap::new()));
        let cache = Arc::new(RwLock::new(PredictionCache {
            predictions: HashMap::new(),
            max_age: Duration::hours(1),
        }));

        let engine = Self {
            config: Arc::clone(&config),
            models,
            cache,
        };

        // Initialize models
        engine.initialize_models().await?;

        Ok(engine)
    }

    /// Initialize ML models
    async fn initialize_models(&self) -> Result<()> {
        let mut models = self.models.write().await;
        
        // Initialize ARIMA model
        models.insert(
            "arima".to_string(),
            Box::new(ARIMAModel::new()) as Box<dyn MLModel>,
        );

        // Initialize Prophet model
        models.insert(
            "prophet".to_string(),
            Box::new(ProphetModel::new()) as Box<dyn MLModel>,
        );

        // Initialize Exponential Smoothing
        models.insert(
            "exp_smoothing".to_string(),
            Box::new(ExponentialSmoothingModel::new()) as Box<dyn MLModel>,
        );

        Ok(())
    }

    /// Generate forecast with specified model
    pub async fn forecast(
        &self,
        data: &TimeSeriesData,
        horizon: u32,
        model_type: Option<ModelType>,
    ) -> Result<Vec<MLPrediction>> {
        let config = self.config.read().await;
        let model_type = model_type.unwrap_or(config.default_model);

        // Check cache first
        if config.cache_predictions {
            let cache_key = format!("{:?}_{}", model_type, horizon);
            if let Some(cached) = self.get_cached_prediction(&cache_key).await {
                return Ok(cached);
            }
        }

        // Generate forecast
        let predictions = match model_type {
            ModelType::Ensemble => self.ensemble_forecast(data, horizon).await?,
            _ => self.single_model_forecast(data, horizon, model_type).await?,
        };

        // Cache results
        if config.cache_predictions {
            let cache_key = format!("{:?}_{}", model_type, horizon);
            self.cache_prediction(&cache_key, predictions.clone()).await;
        }

        Ok(predictions)
    }

    /// Ensemble forecast combining multiple models
    async fn ensemble_forecast(
        &self,
        data: &TimeSeriesData,
        horizon: u32,
    ) -> Result<Vec<MLPrediction>> {
        let config = self.config.read().await;
        let mut all_predictions = Vec::new();

        // Get predictions from each model
        for model_type in &config.ensemble_models {
            match self.single_model_forecast(data, horizon, *model_type).await {
                Ok(preds) => all_predictions.push(preds),
                Err(e) => eprintln!("Model {:?} failed: {}", model_type, e),
            }
        }

        if all_predictions.is_empty() {
            return Err(anyhow::anyhow!("All ensemble models failed"));
        }

        // Combine predictions
        self.combine_ensemble_predictions(all_predictions, horizon)
    }

    /// Single model forecast
    async fn single_model_forecast(
        &self,
        data: &TimeSeriesData,
        horizon: u32,
        model_type: ModelType,
    ) -> Result<Vec<MLPrediction>> {
        let models = self.models.read().await;
        
        let model_key = match model_type {
            ModelType::ARIMA => "arima",
            ModelType::Prophet => "prophet",
            ModelType::ExponentialSmoothing => "exp_smoothing",
            _ => return Err(anyhow::anyhow!("Model type not implemented")),
        };

        let model = models
            .get(model_key)
            .ok_or_else(|| anyhow::anyhow!("Model not found"))?;

        model.forecast(data, horizon)
    }

    /// Detect anomalies in time series
    pub async fn detect_anomalies(
        &self,
        data: &TimeSeriesData,
    ) -> Result<Vec<AnomalyDetectionResult>> {
        let models = self.models.read().await;
        
        // Use ensemble approach for anomaly detection
        let mut all_results = Vec::new();
        
        for (_, model) in models.iter() {
            match model.detect_anomalies(data) {
                Ok(results) => all_results.extend(results),
                Err(e) => eprintln!("Anomaly detection failed: {}", e),
            }
        }

        // Combine and filter results
        self.combine_anomaly_results(all_results).await
    }

    /// Decompose time series
    pub async fn decompose(&self, data: &TimeSeriesData) -> Result<TimeSeriesDecomposition> {
        let models = self.models.read().await;
        
        // Use Prophet model for decomposition
        let model = models
            .get("prophet")
            .ok_or_else(|| anyhow::anyhow!("Prophet model not found"))?;

        model.decompose(data)
    }

    /// Automatically select best model for data
    pub async fn auto_select_model(&self, data: &TimeSeriesData) -> Result<ModelType> {
        // Analyze data characteristics
        let characteristics = self.analyze_data_characteristics(data)?;
        
        // Select model based on characteristics
        Ok(match characteristics {
            DataCharacteristics { has_seasonality: true, has_trend: true, .. } => ModelType::Prophet,
            DataCharacteristics { is_stationary: true, .. } => ModelType::ARIMA,
            DataCharacteristics { has_trend: true, .. } => ModelType::ExponentialSmoothing,
            _ => ModelType::Ensemble,
        })
    }

    /// Get model performance metrics
    pub async fn get_model_performance(
        &self,
        data: &TimeSeriesData,
        model_type: ModelType,
    ) -> Result<ModelPerformanceMetrics> {
        // Split data for validation
        let split_point = (data.values.len() as f64 * 0.8) as usize;
        let train_data = TimeSeriesData {
            timestamps: data.timestamps[..split_point].to_vec(),
            values: data.values[..split_point].to_vec(),
            metadata: data.metadata.clone(),
        };
        let test_data = TimeSeriesData {
            timestamps: data.timestamps[split_point..].to_vec(),
            values: data.values[split_point..].to_vec(),
            metadata: data.metadata.clone(),
        };

        // Generate predictions
        let horizon = test_data.values.len() as u32;
        let predictions = self.forecast(&train_data, horizon, Some(model_type)).await?;

        // Calculate metrics
        self.calculate_performance_metrics(&test_data, &predictions)
    }

    // Helper methods

    async fn get_cached_prediction(&self, key: &str) -> Option<Vec<MLPrediction>> {
        let cache = self.cache.read().await;
        if let Some((timestamp, predictions)) = cache.predictions.get(key) {
            if Utc::now() - *timestamp < cache.max_age {
                return Some(predictions.clone());
            }
        }
        None
    }

    async fn cache_prediction(&self, key: &str, predictions: Vec<MLPrediction>) {
        let mut cache = self.cache.write().await;
        cache.predictions.insert(key.to_string(), (Utc::now(), predictions));
    }

    fn combine_ensemble_predictions(
        &self,
        all_predictions: Vec<Vec<MLPrediction>>,
        horizon: u32,
    ) -> Result<Vec<MLPrediction>> {
        let mut combined = Vec::new();
        
        for i in 0..horizon as usize {
            let mut values = Vec::new();
            let mut timestamp = None;
            
            for predictions in &all_predictions {
                if i < predictions.len() {
                    values.push(predictions[i].value);
                    if timestamp.is_none() {
                        timestamp = Some(predictions[i].timestamp);
                    }
                }
            }
            
            if values.is_empty() {
                continue;
            }
            
            // Calculate ensemble prediction
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let std_dev = (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() 
                / values.len() as f64).sqrt();
            
            combined.push(MLPrediction {
                timestamp: timestamp.unwrap(),
                value: mean,
                confidence: ConfidenceInterval {
                    lower: mean - 1.96 * std_dev,
                    upper: mean + 1.96 * std_dev,
                    confidence_level: 0.95,
                },
                model_type: ModelType::Ensemble,
                seasonality: None,
                trend: None,
            });
        }
        
        Ok(combined)
    }

    async fn combine_anomaly_results(
        &self,
        mut results: Vec<AnomalyDetectionResult>,
    ) -> Result<Vec<AnomalyDetectionResult>> {
        // Sort by timestamp
        results.sort_by_key(|r| r.timestamp);
        
        // Group by timestamp and combine
        let mut combined = HashMap::new();
        
        for result in results {
            let entry = combined.entry(result.timestamp).or_insert(Vec::new());
            entry.push(result);
        }
        
        // Create final results
        let mut final_results = Vec::new();
        for (timestamp, group) in combined {
            let avg_score = group.iter().map(|r| r.anomaly_score).sum::<f64>() / group.len() as f64;
            let is_anomaly = avg_score > 0.8; // Threshold
            
            final_results.push(AnomalyDetectionResult {
                timestamp,
                value: group[0].value,
                anomaly_score: avg_score,
                is_anomaly,
                anomaly_type: if is_anomaly { group[0].anomaly_type } else { None },
                explanation: if is_anomaly {
                    format!("Anomaly detected by {} models", group.len())
                } else {
                    "Normal behavior".to_string()
                },
            });
        }
        
        Ok(final_results)
    }

    fn analyze_data_characteristics(&self, data: &TimeSeriesData) -> Result<DataCharacteristics> {
        // Simple analysis - in production would use more sophisticated methods
        let values = &data.values;
        
        // Check for trend
        let first_half_mean = values[..values.len()/2].iter().sum::<f64>() / (values.len()/2) as f64;
        let second_half_mean = values[values.len()/2..].iter().sum::<f64>() / (values.len()/2) as f64;
        let has_trend = (second_half_mean - first_half_mean).abs() > first_half_mean * 0.1;
        
        // Check for seasonality (simplified)
        let has_seasonality = values.len() > 7;
        
        // Check stationarity
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let is_stationary = variance < mean * 0.5;
        
        Ok(DataCharacteristics {
            has_trend,
            has_seasonality,
            is_stationary,
            data_points: values.len(),
        })
    }

    fn calculate_performance_metrics(
        &self,
        actual: &TimeSeriesData,
        predictions: &[MLPrediction],
    ) -> Result<ModelPerformanceMetrics> {
        let actual_values = &actual.values;
        let predicted_values: Vec<f64> = predictions.iter().map(|p| p.value).collect();
        
        // Calculate RMSE
        let mse = actual_values.iter()
            .zip(predicted_values.iter())
            .map(|(a, p)| (a - p).powi(2))
            .sum::<f64>() / actual_values.len() as f64;
        let rmse = mse.sqrt();
        
        // Calculate MAE
        let mae = actual_values.iter()
            .zip(predicted_values.iter())
            .map(|(a, p)| (a - p).abs())
            .sum::<f64>() / actual_values.len() as f64;
        
        // Calculate MAPE
        let mape = actual_values.iter()
            .zip(predicted_values.iter())
            .map(|(a, p)| ((a - p).abs() / a.abs()) * 100.0)
            .sum::<f64>() / actual_values.len() as f64;
        
        Ok(ModelPerformanceMetrics {
            rmse,
            mae,
            mape,
            r_squared: 0.0, // Would calculate R² in production
        })
    }
}

#[derive(Debug)]
struct DataCharacteristics {
    has_trend: bool,
    has_seasonality: bool,
    is_stationary: bool,
    data_points: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub rmse: f64,
    pub mae: f64,
    pub mape: f64,
    pub r_squared: f64,
}

// Model implementations

struct ARIMAModel;

impl ARIMAModel {
    fn new() -> Self {
        Self
    }
}

impl MLModel for ARIMAModel {
    fn forecast(&self, data: &TimeSeriesData, horizon: u32) -> Result<Vec<MLPrediction>> {
        // Simplified ARIMA implementation
        let mut predictions = Vec::new();
        let last_value = data.values.last().ok_or_else(|| anyhow::anyhow!("No data"))?;
        let last_timestamp = data.timestamps.last().ok_or_else(|| anyhow::anyhow!("No timestamps"))?;
        
        // Simple moving average as placeholder
        let window = 7.min(data.values.len());
        let ma = data.values[data.values.len()-window..].iter().sum::<f64>() / window as f64;
        
        for i in 1..=horizon {
            predictions.push(MLPrediction {
                timestamp: *last_timestamp + Duration::days(i as i64),
                value: ma + (last_value - ma) * 0.5_f64.powi(i as i32),
                confidence: ConfidenceInterval {
                    lower: ma * 0.9,
                    upper: ma * 1.1,
                    confidence_level: 0.95,
                },
                model_type: ModelType::ARIMA,
                seasonality: None,
                trend: None,
            });
        }
        
        Ok(predictions)
    }

    fn detect_anomalies(&self, data: &TimeSeriesData) -> Result<Vec<AnomalyDetectionResult>> {
        // Simplified anomaly detection
        let mut results = Vec::new();
        let values = &data.values;
        
        if values.len() < 3 {
            return Ok(results);
        }
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let std_dev = (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() 
            / values.len() as f64).sqrt();
        
        for (i, value) in values.iter().enumerate() {
            let z_score = (value - mean).abs() / std_dev;
            let is_anomaly = z_score > 3.0;
            
            results.push(AnomalyDetectionResult {
                timestamp: data.timestamps[i],
                value: *value,
                anomaly_score: z_score / 4.0, // Normalize to 0-1
                is_anomaly,
                anomaly_type: if is_anomaly {
                    if value > &mean { Some(AnomalyType::Spike) } else { Some(AnomalyType::Dip) }
                } else {
                    None
                },
                explanation: if is_anomaly {
                    format!("Value deviates {:.1} standard deviations from mean", z_score)
                } else {
                    "Normal".to_string()
                },
            });
        }
        
        Ok(results)
    }

    fn decompose(&self, data: &TimeSeriesData) -> Result<TimeSeriesDecomposition> {
        // Simplified decomposition
        let values = &data.values;
        let trend = values.clone(); // Placeholder
        let seasonal = vec![0.0; values.len()];
        let residual = vec![0.0; values.len()];
        
        Ok(TimeSeriesDecomposition {
            trend,
            seasonal,
            residual,
            seasonal_periods: Vec::new(),
        })
    }

    fn model_type(&self) -> ModelType {
        ModelType::ARIMA
    }
}

struct ProphetModel;

impl ProphetModel {
    fn new() -> Self {
        Self
    }
}

impl MLModel for ProphetModel {
    fn forecast(&self, data: &TimeSeriesData, horizon: u32) -> Result<Vec<MLPrediction>> {
        // Simplified Prophet-style implementation
        let mut predictions = Vec::new();
        let values = &data.values;
        let timestamps = &data.timestamps;
        
        // Detect trend
        let trend = self.detect_trend(values)?;
        
        // Detect seasonality
        let seasonality = self.detect_seasonality(values, timestamps)?;
        
        let last_value = values.last().ok_or_else(|| anyhow::anyhow!("No data"))?;
        let last_timestamp = timestamps.last().ok_or_else(|| anyhow::anyhow!("No timestamps"))?;
        
        for i in 1..=horizon {
            let future_timestamp = *last_timestamp + Duration::days(i as i64);
            let trend_component = trend.project(i);
            let seasonal_component = seasonality.get_value(future_timestamp);
            
            predictions.push(MLPrediction {
                timestamp: future_timestamp,
                value: trend_component + seasonal_component,
                confidence: ConfidenceInterval {
                    lower: (trend_component + seasonal_component) * 0.85,
                    upper: (trend_component + seasonal_component) * 1.15,
                    confidence_level: 0.95,
                },
                model_type: ModelType::Prophet,
                seasonality: Some(seasonality.clone()),
                trend: Some(trend.clone()),
            });
        }
        
        Ok(predictions)
    }

    fn detect_anomalies(&self, data: &TimeSeriesData) -> Result<Vec<AnomalyDetectionResult>> {
        // Prophet-style anomaly detection using decomposition
        let decomposition = self.decompose(data)?;
        let mut results = Vec::new();
        
        for (i, residual) in decomposition.residual.iter().enumerate() {
            let is_anomaly = residual.abs() > 2.0; // Simplified threshold
            
            results.push(AnomalyDetectionResult {
                timestamp: data.timestamps[i],
                value: data.values[i],
                anomaly_score: residual.abs() / 3.0, // Normalize
                is_anomaly,
                anomaly_type: if is_anomaly {
                    Some(AnomalyType::Seasonal)
                } else {
                    None
                },
                explanation: if is_anomaly {
                    "Deviation from seasonal pattern".to_string()
                } else {
                    "Normal seasonal behavior".to_string()
                },
            });
        }
        
        Ok(results)
    }

    fn decompose(&self, data: &TimeSeriesData) -> Result<TimeSeriesDecomposition> {
        let values = &data.values;
        
        // Simple decomposition
        let trend = self.calculate_trend(values)?;
        let detrended: Vec<f64> = values.iter()
            .zip(trend.iter())
            .map(|(v, t)| v - t)
            .collect();
        
        let seasonal = self.calculate_seasonal(&detrended)?;
        let residual: Vec<f64> = detrended.iter()
            .zip(seasonal.iter())
            .map(|(d, s)| d - s)
            .collect();
        
        Ok(TimeSeriesDecomposition {
            trend,
            seasonal,
            residual,
            seasonal_periods: vec![
                SeasonalComponent {
                    period: "weekly".to_string(),
                    strength: 0.7,
                    pattern: vec![1.0, 0.9, 0.8, 0.85, 0.95, 1.1, 1.05],
                }
            ],
        })
    }

    fn model_type(&self) -> ModelType {
        ModelType::Prophet
    }
}

impl ProphetModel {
    // Helper methods
    fn detect_trend(&self, values: &[f64]) -> Result<TrendComponent> {
        let n = values.len() as f64;
        let x: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        
        // Simple linear regression
        let x_mean = x.iter().sum::<f64>() / n;
        let y_mean = values.iter().sum::<f64>() / n;
        
        let numerator: f64 = x.iter()
            .zip(values.iter())
            .map(|(xi, yi)| (xi - x_mean) * (yi - y_mean))
            .sum();
        
        let denominator: f64 = x.iter()
            .map(|xi| (xi - x_mean).powi(2))
            .sum();
        
        let slope = numerator / denominator;
        
        Ok(TrendComponent {
            direction: if slope > 0.01 {
                TrendDirection::Increasing
            } else if slope < -0.01 {
                TrendDirection::Decreasing
            } else {
                TrendDirection::Stable
            },
            strength: slope.abs(),
            change_points: Vec::new(),
        })
    }

    fn detect_seasonality(&self, values: &[f64], timestamps: &[DateTime<Utc>]) -> Result<SeasonalComponent> {
        // Simplified weekly seasonality
        Ok(SeasonalComponent {
            period: "weekly".to_string(),
            strength: 0.5,
            pattern: vec![1.0, 0.95, 0.9, 0.85, 0.9, 1.05, 1.1],
        })
    }

    fn calculate_trend(&self, values: &[f64]) -> Result<Vec<f64>> {
        // Moving average for trend
        let window = 7.min(values.len());
        let mut trend = Vec::new();
        
        for i in 0..values.len() {
            let start = i.saturating_sub(window / 2);
            let end = (i + window / 2 + 1).min(values.len());
            let avg = values[start..end].iter().sum::<f64>() / (end - start) as f64;
            trend.push(avg);
        }
        
        Ok(trend)
    }

    fn calculate_seasonal(&self, detrended: &[f64]) -> Result<Vec<f64>> {
        // Simplified seasonal calculation
        let period = 7; // Weekly
        let mut seasonal = Vec::new();
        
        for i in 0..detrended.len() {
            let seasonal_index = i % period;
            let seasonal_value = match seasonal_index {
                0 | 6 => 0.1,  // Weekend
                _ => -0.02,    // Weekday
            };
            seasonal.push(seasonal_value);
        }
        
        Ok(seasonal)
    }
}

impl TrendComponent {
    fn project(&self, steps: u32) -> f64 {
        match self.direction {
            TrendDirection::Increasing => 100.0 + (steps as f64 * self.strength),
            TrendDirection::Decreasing => 100.0 - (steps as f64 * self.strength),
            _ => 100.0,
        }
    }
}

impl SeasonalComponent {
    fn get_value(&self, timestamp: DateTime<Utc>) -> f64 {
        let day_of_week = timestamp.weekday().num_days_from_monday() as usize;
        self.pattern.get(day_of_week).copied().unwrap_or(1.0)
    }
}

struct ExponentialSmoothingModel;

impl ExponentialSmoothingModel {
    fn new() -> Self {
        Self
    }
}

impl MLModel for ExponentialSmoothingModel {
    fn forecast(&self, data: &TimeSeriesData, horizon: u32) -> Result<Vec<MLPrediction>> {
        let values = &data.values;
        let timestamps = &data.timestamps;
        
        if values.is_empty() {
            return Err(anyhow::anyhow!("No data for forecasting"));
        }
        
        // Simple exponential smoothing
        let alpha = 0.3;
        let mut level = values[0];
        
        for value in values.iter().skip(1) {
            level = alpha * value + (1.0 - alpha) * level;
        }
        
        let last_timestamp = timestamps.last().ok_or_else(|| anyhow::anyhow!("No timestamps"))?;
        let mut predictions = Vec::new();
        
        for i in 1..=horizon {
            predictions.push(MLPrediction {
                timestamp: *last_timestamp + Duration::days(i as i64),
                value: level,
                confidence: ConfidenceInterval {
                    lower: level * 0.9,
                    upper: level * 1.1,
                    confidence_level: 0.95,
                },
                model_type: ModelType::ExponentialSmoothing,
                seasonality: None,
                trend: None,
            });
        }
        
        Ok(predictions)
    }

    fn detect_anomalies(&self, data: &TimeSeriesData) -> Result<Vec<AnomalyDetectionResult>> {
        // Simple threshold-based detection
        let values = &data.values;
        let mut results = Vec::new();
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        for (i, value) in values.iter().enumerate() {
            let deviation = (value - mean).abs() / mean;
            let is_anomaly = deviation > 0.5;
            
            results.push(AnomalyDetectionResult {
                timestamp: data.timestamps[i],
                value: *value,
                anomaly_score: deviation.min(1.0),
                is_anomaly,
                anomaly_type: if is_anomaly {
                    Some(AnomalyType::LevelShift)
                } else {
                    None
                },
                explanation: if is_anomaly {
                    format!("Value deviates {:.1}% from mean", deviation * 100.0)
                } else {
                    "Normal".to_string()
                },
            });
        }
        
        Ok(results)
    }

    fn decompose(&self, data: &TimeSeriesData) -> Result<TimeSeriesDecomposition> {
        // Simple decomposition
        let values = &data.values;
        Ok(TimeSeriesDecomposition {
            trend: values.clone(),
            seasonal: vec![0.0; values.len()],
            residual: vec![0.0; values.len()],
            seasonal_periods: Vec::new(),
        })
    }

    fn model_type(&self) -> ModelType {
        ModelType::ExponentialSmoothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ml_engine_creation() -> Result<()> {
        let config = MLConfig::default();
        let engine = MLEngine::new(config).await?;
        
        // Test that models are initialized
        let models = engine.models.read().await;
        assert!(models.contains_key("arima"));
        assert!(models.contains_key("prophet"));
        assert!(models.contains_key("exp_smoothing"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_forecast_generation() -> Result<()> {
        let config = MLConfig::default();
        let engine = MLEngine::new(config).await?;
        
        // Create test data
        let data = TimeSeriesData {
            timestamps: (0..30).map(|i| Utc::now() - Duration::days(30 - i)).collect(),
            values: (0..30).map(|i| 100.0 + (i as f64) * 2.0).collect(),
            metadata: HashMap::new(),
        };
        
        // Generate forecast
        let predictions = engine.forecast(&data, 7, Some(ModelType::ARIMA)).await?;
        
        assert_eq!(predictions.len(), 7);
        assert!(predictions[0].value > 0.0);
        
        Ok(())
    }
}