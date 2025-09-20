//! Hive Consensus Backend Server
//!
//! Provides HTTP/WebSocket API for Electron frontend
//! Day 0: Proof of concept for Electron + Rust architecture

use axum::{
    http::{HeaderValue, Method},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

// Import the REAL Hive consensus engine
use hive_ai::consensus::engine::ConsensusEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestRequest {
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResponse {
    echo: String,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsensusRequest {
    query: String,
    profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsensusResponse {
    result: String,
    duration_ms: u128,
    model_used: String,
}

// Application state
struct AppState {
    consensus_engine: Arc<RwLock<Option<ConsensusEngine>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,hive=debug")
        .init();

    info!("🐝 Starting Hive Backend Server...");

    // Initialize the consensus engine (lazy load on first use)
    let consensus_engine = Arc::new(RwLock::new(None));

    // Create shared state
    let state = Arc::new(AppState { consensus_engine });

    // Build the router
    let app = Router::new()
        // Test endpoint for Day 0 validation
        .route("/test", post(test_endpoint))
        // Consensus endpoints
        .route("/api/consensus", post(run_consensus))
        // Health check
        .route("/health", get(health_check))
        // Add CORS support for Electron - allow any origin for testing
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(tower_http::cors::Any),
        )
        .with_state(state);

    // Start the server on all interfaces for better compatibility
    let addr = "0.0.0.0:8765";
    info!("✅ Hive Backend Server running on http://{}", addr);
    info!("📝 Test endpoint: POST http://{}/test", addr);
    info!("🧠 Consensus endpoint: POST http://{}/api/consensus", addr);
    info!("🌐 Also available at: http://localhost:8765 and http://127.0.0.1:8765");

    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// Test endpoint for Day 0 validation
async fn test_endpoint(Json(req): Json<String>) -> Json<TestResponse> {
    info!("Test endpoint called with: {}", req);

    Json(TestResponse {
        echo: format!("Echo: {}", req),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

// Run consensus with the REAL engine
async fn run_consensus(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<ConsensusRequest>,
) -> Result<Json<ConsensusResponse>, String> {
    info!("Running consensus for query: {}", req.query);
    let start = std::time::Instant::now();

    // Initialize engine on first use
    let mut engine_guard = state.consensus_engine.write().await;
    if engine_guard.is_none() {
        info!("Initializing consensus engine...");
        match ConsensusEngine::new(None).await {
            Ok(engine) => {
                info!("✅ Consensus engine initialized");
                *engine_guard = Some(engine);
            }
            Err(e) => {
                error!("Failed to initialize consensus engine: {}", e);
                // Return a proper JSON error response instead of string error
                let error_response = ConsensusResponse {
                    result: format!("❌ Consensus engine initialization failed: {}\n\nThis is expected for Day 0 validation - the consensus engine needs a configured database and profile.", e),
                    duration_ms: start.elapsed().as_millis(),
                    model_used: "initialization-error".to_string(),
                };
                return Ok(Json(error_response));
            }
        }
    }

    // Run consensus
    let engine = engine_guard.as_ref().unwrap();
    match engine.process(&req.query, None).await {
        Ok(result) => {
            let duration_ms = start.elapsed().as_millis();
            info!("✅ Consensus completed in {}ms", duration_ms);

            Ok(Json(ConsensusResponse {
                result: result.result.unwrap_or_else(|| "No result".to_string()),
                duration_ms,
                model_used: "4-stage-consensus".to_string(),
            }))
        }
        Err(e) => {
            error!("Consensus failed: {}", e);
            Err(format!("Consensus failed: {}", e))
        }
    }
}

// Health check
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "hive-backend-server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
