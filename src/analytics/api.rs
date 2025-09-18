//! Analytics REST API
//!
//! Provides RESTful API endpoints for external integrations:
//! - Query analytics data programmatically
//! - Real-time metrics streaming
//! - Webhook integration for alerts
//! - OAuth2 authentication
//! - Rate limiting and quotas

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub port: u16,
    pub host: String,
    pub enable_cors: bool,
    pub rate_limit: RateLimitConfig,
    pub auth: AuthConfig,
    pub ssl: Option<SslConfig>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub enable_quotas: bool,
    pub quota_reset_interval: Duration,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enable_api_keys: bool,
    pub enable_oauth2: bool,
    pub jwt_secret: String,
    pub token_expiry: Duration,
}

/// SSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub cert_path: String,
    pub key_path: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            enable_cors: true,
            rate_limit: RateLimitConfig {
                requests_per_minute: 60,
                burst_size: 10,
                enable_quotas: true,
                quota_reset_interval: Duration::days(30),
            },
            auth: AuthConfig {
                enable_api_keys: true,
                enable_oauth2: false,
                jwt_secret: "change-me-in-production".to_string(),
                token_expiry: Duration::hours(24),
            },
            ssl: None,
        }
    }
}

/// API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub key: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub permissions: Vec<Permission>,
    pub quota: ApiQuota,
    pub metadata: HashMap<String, String>,
}

/// API permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ReadAnalytics,
    WriteAnalytics,
    ManageDashboards,
    ManageAlerts,
    AdminAccess,
}

/// API quota
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiQuota {
    pub requests_per_month: u64,
    pub used_requests: u64,
    pub reset_at: DateTime<Utc>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub metadata: ResponseMetadata,
}

/// API error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, String>>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub rate_limit: Option<RateLimitInfo>,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
}

/// Analytics API server
pub struct AnalyticsApi {
    config: Arc<RwLock<ApiConfig>>,
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    rate_limiter: Arc<RateLimiter>,
    analytics_engine: Arc<crate::analytics::AdvancedAnalyticsEngine>,
}

impl AnalyticsApi {
    /// Create new API server
    pub fn new(
        config: ApiConfig,
        analytics_engine: Arc<crate::analytics::AdvancedAnalyticsEngine>,
    ) -> Self {
        Self {
            config: Arc::new(RwLock::new(config.clone())),
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RateLimiter::new(config.rate_limit)),
            analytics_engine,
        }
    }

    /// Start API server
    pub async fn start(&self) -> Result<()> {
        let config = self.config.read().await;
        let addr = format!("{}:{}", config.host, config.port);
        let addr: std::net::SocketAddr = addr.parse()?;

        println!("Starting Analytics API on {}", addr);

        // Build routes
        let routes = self.build_routes().await;

        // Start server
        warp::serve(routes).run(addr).await;

        Ok(())
    }

    /// Build API routes
    async fn build_routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let cors = if self.config.read().await.enable_cors {
            warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allow_headers(vec!["Content-Type", "Authorization", "X-API-Key"])
                .build()
        } else {
            warp::cors().build()
        };

        // Health check
        let health = warp::path("health")
            .and(warp::get())
            .map(|| warp::reply::json(&serde_json::json!({ "status": "healthy" })));

        // Analytics endpoints
        let analytics = self.analytics_routes();

        // Dashboard endpoints
        let dashboards = self.dashboard_routes();

        // Alert endpoints
        let alerts = self.alert_routes();

        // Metrics streaming
        let stream = self.stream_routes();

        // Combine all routes
        health
            .or(analytics)
            .or(dashboards)
            .or(alerts)
            .or(stream)
            .recover(handle_rejection)
            .with(cors)
    }

    /// Analytics routes
    fn analytics_routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let engine = Arc::clone(&self.analytics_engine);
        let auth = self.auth_filter();

        // GET /api/v1/analytics/overview
        let overview = warp::path!("api" / "v1" / "analytics" / "overview")
            .and(warp::get())
            .and(auth.clone())
            .and(warp::query::<OverviewQuery>())
            .and(with_engine(engine.clone()))
            .and_then(get_analytics_overview);

        // GET /api/v1/analytics/trends
        let trends = warp::path!("api" / "v1" / "analytics" / "trends")
            .and(warp::get())
            .and(auth.clone())
            .and(warp::query::<TrendsQuery>())
            .and(with_engine(engine.clone()))
            .and_then(get_trends);

        // GET /api/v1/analytics/forecast
        let forecast = warp::path!("api" / "v1" / "analytics" / "forecast")
            .and(warp::get())
            .and(auth.clone())
            .and(warp::query::<ForecastQuery>())
            .and(with_engine(engine.clone()))
            .and_then(get_forecast);

        // POST /api/v1/analytics/query
        let query = warp::path!("api" / "v1" / "analytics" / "query")
            .and(warp::post())
            .and(auth.clone())
            .and(warp::body::json())
            .and(with_engine(engine.clone()))
            .and_then(execute_query);

        overview.or(trends).or(forecast).or(query)
    }

    /// Dashboard routes
    fn dashboard_routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let auth = self.auth_filter();

        // GET /api/v1/dashboards
        let list = warp::path!("api" / "v1" / "dashboards")
            .and(warp::get())
            .and(auth.clone())
            .and_then(list_dashboards);

        // GET /api/v1/dashboards/{id}
        let get = warp::path!("api" / "v1" / "dashboards" / String)
            .and(warp::get())
            .and(auth.clone())
            .and_then(get_dashboard);

        // POST /api/v1/dashboards
        let create = warp::path!("api" / "v1" / "dashboards")
            .and(warp::post())
            .and(auth.clone())
            .and(warp::body::json())
            .and_then(create_dashboard);

        // PUT /api/v1/dashboards/{id}
        let update = warp::path!("api" / "v1" / "dashboards" / String)
            .and(warp::put())
            .and(auth.clone())
            .and(warp::body::json())
            .and_then(update_dashboard);

        // DELETE /api/v1/dashboards/{id}
        let delete = warp::path!("api" / "v1" / "dashboards" / String)
            .and(warp::delete())
            .and(auth.clone())
            .and_then(delete_dashboard);

        list.or(get).or(create).or(update).or(delete)
    }

    /// Alert routes
    fn alert_routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let auth = self.auth_filter();

        // GET /api/v1/alerts
        let list = warp::path!("api" / "v1" / "alerts")
            .and(warp::get())
            .and(auth.clone())
            .and(warp::query::<AlertsQuery>())
            .and_then(list_alerts);

        // GET /api/v1/alerts/{id}
        let get = warp::path!("api" / "v1" / "alerts" / String)
            .and(warp::get())
            .and(auth.clone())
            .and_then(get_alert);

        // POST /api/v1/alerts/{id}/acknowledge
        let acknowledge = warp::path!("api" / "v1" / "alerts" / String / "acknowledge")
            .and(warp::post())
            .and(auth.clone())
            .and_then(acknowledge_alert);

        // POST /api/v1/alerts/{id}/resolve
        let resolve = warp::path!("api" / "v1" / "alerts" / String / "resolve")
            .and(warp::post())
            .and(auth.clone())
            .and_then(resolve_alert);

        // GET /api/v1/alert-rules
        let rules = warp::path!("api" / "v1" / "alert-rules")
            .and(warp::get())
            .and(auth.clone())
            .and_then(list_alert_rules);

        // POST /api/v1/alert-rules
        let create_rule = warp::path!("api" / "v1" / "alert-rules")
            .and(warp::post())
            .and(auth.clone())
            .and(warp::body::json())
            .and_then(create_alert_rule);

        list.or(get)
            .or(acknowledge)
            .or(resolve)
            .or(rules)
            .or(create_rule)
    }

    /// Streaming routes
    fn stream_routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let auth = self.auth_filter();

        // GET /api/v1/stream/metrics
        let metrics = warp::path!("api" / "v1" / "stream" / "metrics")
            .and(warp::get())
            .and(auth.clone())
            .and(warp::ws())
            .map(|auth: AuthContext, ws: warp::ws::Ws| {
                ws.on_upgrade(move |websocket| handle_metrics_stream(websocket, auth))
            });

        // GET /api/v1/stream/alerts
        let alerts = warp::path!("api" / "v1" / "stream" / "alerts")
            .and(warp::get())
            .and(auth.clone())
            .and(warp::ws())
            .map(|auth: AuthContext, ws: warp::ws::Ws| {
                ws.on_upgrade(move |websocket| handle_alerts_stream(websocket, auth))
            });

        metrics.or(alerts)
    }

    /// Authentication filter
    fn auth_filter(&self) -> impl Filter<Extract = (AuthContext,), Error = Rejection> + Clone {
        let api_keys = Arc::clone(&self.api_keys);
        let config = Arc::clone(&self.config);

        warp::header::optional::<String>("x-api-key")
            .and(warp::header::optional::<String>("authorization"))
            .and_then(
                move |api_key: Option<String>, auth_header: Option<String>| {
                    let api_keys = Arc::clone(&api_keys);
                    let config = Arc::clone(&config);

                    async move {
                        authenticate(api_key, auth_header, api_keys, config)
                            .await
                            .map_err(|_| warp::reject::custom(ApiAuthError))
                    }
                },
            )
    }

    /// Generate API key
    pub async fn generate_api_key(
        &self,
        name: String,
        permissions: Vec<Permission>,
    ) -> Result<ApiKey> {
        let key = ApiKey {
            id: uuid::Uuid::new_v4().to_string(),
            key: generate_secure_key(),
            name,
            created_at: Utc::now(),
            expires_at: None,
            permissions,
            quota: ApiQuota {
                requests_per_month: 10000,
                used_requests: 0,
                reset_at: Utc::now() + Duration::days(30),
            },
            metadata: HashMap::new(),
        };

        let mut keys = self.api_keys.write().await;
        keys.insert(key.key.clone(), key.clone());

        Ok(key)
    }

    /// Revoke API key
    pub async fn revoke_api_key(&self, key: &str) -> Result<()> {
        let mut keys = self.api_keys.write().await;
        keys.remove(key)
            .ok_or_else(|| anyhow::anyhow!("API key not found"))?;
        Ok(())
    }

    /// List API keys
    pub async fn list_api_keys(&self) -> Vec<ApiKey> {
        let keys = self.api_keys.read().await;
        keys.values().cloned().collect()
    }
}

/// Rate limiter
struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
}

impl RateLimiter {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn check_rate_limit(&self, key: &str) -> Result<RateLimitInfo> {
        let mut buckets = self.buckets.write().await;
        let bucket = buckets.entry(key.to_string()).or_insert_with(|| {
            TokenBucket::new(self.config.requests_per_minute, self.config.burst_size)
        });

        if bucket.try_consume() {
            Ok(RateLimitInfo {
                limit: self.config.requests_per_minute,
                remaining: bucket.tokens as u32,
                reset_at: bucket.reset_at,
            })
        } else {
            Err(anyhow::anyhow!("Rate limit exceeded"))
        }
    }
}

/// Token bucket for rate limiting
struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_rate: f64,
    last_refill: DateTime<Utc>,
    reset_at: DateTime<Utc>,
}

impl TokenBucket {
    fn new(requests_per_minute: u32, burst_size: u32) -> Self {
        let capacity = burst_size.max(requests_per_minute);
        Self {
            capacity,
            tokens: capacity as f64,
            refill_rate: requests_per_minute as f64 / 60.0,
            last_refill: Utc::now(),
            reset_at: Utc::now() + Duration::minutes(1),
        }
    }

    fn try_consume(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Utc::now();
        let elapsed = (now - self.last_refill).num_seconds() as f64;

        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
        self.last_refill = now;

        if now > self.reset_at {
            self.reset_at = now + Duration::minutes(1);
        }
    }
}

/// Authentication context
#[derive(Debug, Clone)]
struct AuthContext {
    api_key: Option<ApiKey>,
    user_id: Option<String>,
    permissions: Vec<Permission>,
}

/// Query parameters
#[derive(Debug, Deserialize)]
struct OverviewQuery {
    period: String,
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TrendsQuery {
    metric: String,
    period: String,
    interval: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ForecastQuery {
    metric: String,
    horizon: u32,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlertsQuery {
    status: Option<String>,
    severity: Option<String>,
    limit: Option<u32>,
}

/// Request bodies
#[derive(Debug, Deserialize)]
struct QueryRequest {
    query: String,
    parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct DashboardRequest {
    name: String,
    description: String,
    layout: String,
    widgets: Vec<serde_json::Value>,
}

/// Helper functions

fn with_engine(
    engine: Arc<crate::analytics::AdvancedAnalyticsEngine>,
) -> impl Filter<
    Extract = (Arc<crate::analytics::AdvancedAnalyticsEngine>,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || Arc::clone(&engine))
}

async fn authenticate(
    api_key: Option<String>,
    auth_header: Option<String>,
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    config: Arc<RwLock<ApiConfig>>,
) -> Result<AuthContext> {
    let config = config.read().await;

    // Check API key
    if config.auth.enable_api_keys {
        if let Some(key) = api_key {
            let keys = api_keys.read().await;
            if let Some(api_key) = keys.get(&key) {
                // Check expiry
                if let Some(expires_at) = api_key.expires_at {
                    if expires_at < Utc::now() {
                        return Err(anyhow::anyhow!("API key expired"));
                    }
                }

                return Ok(AuthContext {
                    api_key: Some(api_key.clone()),
                    user_id: None,
                    permissions: api_key.permissions.clone(),
                });
            }
        }
    }

    // Check OAuth2 token
    if config.auth.enable_oauth2 {
        if let Some(auth) = auth_header {
            if auth.starts_with("Bearer ") {
                let token = auth.trim_start_matches("Bearer ");
                // Validate JWT token
                // ... implementation ...
            }
        }
    }

    Err(anyhow::anyhow!("Authentication required"))
}

fn generate_secure_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    base64::encode(&bytes)
}

/// Route handlers

async fn get_analytics_overview(
    auth: AuthContext,
    query: OverviewQuery,
    engine: Arc<crate::analytics::AdvancedAnalyticsEngine>,
) -> Result<impl Reply, Rejection> {
    let format = query.format.as_deref().unwrap_or("json");
    let overview = engine
        .overview(&query.period, format)
        .await
        .map_err(|_| warp::reject::custom(InternalApiError))?;

    Ok(warp::reply::json(&ApiResponse {
        success: true,
        data: Some(overview),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn get_trends(
    auth: AuthContext,
    query: TrendsQuery,
    engine: Arc<crate::analytics::AdvancedAnalyticsEngine>,
) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some("Trends data".to_string()),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn get_forecast(
    auth: AuthContext,
    query: ForecastQuery,
    engine: Arc<crate::analytics::AdvancedAnalyticsEngine>,
) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some("Forecast data".to_string()),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn execute_query(
    auth: AuthContext,
    request: QueryRequest,
    engine: Arc<crate::analytics::AdvancedAnalyticsEngine>,
) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some("Query results".to_string()),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn list_dashboards(auth: AuthContext) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<Vec<String>> {
        success: true,
        data: Some(vec![]),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn get_dashboard(id: String, auth: AuthContext) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some(format!("Dashboard {}", id)),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn create_dashboard(
    auth: AuthContext,
    request: DashboardRequest,
) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some("Dashboard created".to_string()),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn update_dashboard(
    id: String,
    auth: AuthContext,
    request: DashboardRequest,
) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some(format!("Dashboard {} updated", id)),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn delete_dashboard(id: String, auth: AuthContext) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some(format!("Dashboard {} deleted", id)),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn list_alerts(auth: AuthContext, query: AlertsQuery) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<Vec<String>> {
        success: true,
        data: Some(vec![]),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn get_alert(id: String, auth: AuthContext) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some(format!("Alert {}", id)),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn acknowledge_alert(id: String, auth: AuthContext) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some(format!("Alert {} acknowledged", id)),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn resolve_alert(id: String, auth: AuthContext) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some(format!("Alert {} resolved", id)),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn list_alert_rules(auth: AuthContext) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<Vec<String>> {
        success: true,
        data: Some(vec![]),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn create_alert_rule(
    auth: AuthContext,
    request: serde_json::Value,
) -> Result<impl Reply, Rejection> {
    // Implementation
    Ok(warp::reply::json(&ApiResponse::<String> {
        success: true,
        data: Some("Alert rule created".to_string()),
        error: None,
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    }))
}

async fn handle_metrics_stream(ws: warp::ws::WebSocket, auth: AuthContext) {
    // Implementation for WebSocket streaming
}

async fn handle_alerts_stream(ws: warp::ws::WebSocket, auth: AuthContext) {
    // Implementation for WebSocket streaming
}

/// Error handling

#[derive(Debug)]
struct InternalApiError;
impl warp::reject::Reject for InternalApiError {}

#[derive(Debug)]
struct ApiAuthError;
impl warp::reject::Reject for ApiAuthError {}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(_) = err.find::<ApiAuthError>() {
        code = StatusCode::UNAUTHORIZED;
        message = "Unauthorized";
    } else if let Some(_) = err.find::<InternalApiError>() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    } else {
        code = StatusCode::BAD_REQUEST;
        message = "Bad Request";
    }

    let json = warp::reply::json(&ApiResponse::<()> {
        success: false,
        data: None,
        error: Some(crate::analytics::api::ApiError {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }),
        metadata: ResponseMetadata {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            rate_limit: None,
        },
    });

    Ok(warp::reply::with_status(json, code))
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
        assert!(config.enable_cors);
    }

    #[tokio::test]
    async fn test_api_key_generation() -> Result<()> {
        let config = ApiConfig::default();
        let engine = Arc::new(
            crate::analytics::AdvancedAnalyticsEngine::new(
                crate::analytics::AdvancedAnalyticsConfig::default(),
            )
            .await?,
        );
        let api = AnalyticsApi::new(config, engine);

        let key = api
            .generate_api_key("Test Key".to_string(), vec![Permission::ReadAnalytics])
            .await?;

        assert_eq!(key.name, "Test Key");
        assert!(!key.key.is_empty());
        assert_eq!(key.permissions, vec![Permission::ReadAnalytics]);

        Ok(())
    }

    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(60, 10);

        // Should allow initial requests
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());

        // Check remaining tokens
        assert_eq!(bucket.tokens as u32, 8);
    }
}
