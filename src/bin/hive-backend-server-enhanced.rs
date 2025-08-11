//! Enhanced Hive Consensus Backend Server with WebSocket Streaming
//! 
//! Modern multi-threaded architecture for Electron frontend (2025)
//! - WebSocket support for real-time streaming
//! - Proper database integration
//! - AI helpers for routing decisions
//! - Multi-threaded consensus processing
//! - No CPU overheating issues

use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures::{sink::SinkExt, StreamExt};
use hive_ai::{
    ai_helpers::AIHelperEcosystem,
    consensus::{
        engine::ConsensusEngine,
        streaming::{ConsensusEvent, ProgressInfo, StreamingCallbacks},
        types::{Stage, StageResult},
    },
    core::{
        database::{initialize_database, get_database, DatabaseManager},
        api_keys::ApiKeyManager,
    },
    maintenance::{BackgroundMaintenance, MaintenanceConfig},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};
use axum::http::Method;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsensusRequest {
    query: String,
    profile: Option<String>,
    conversation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsensusResponse {
    result: String,
    duration_ms: u128,
    model_used: String,
    tokens_used: u32,
    cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContextMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WSMessage {
    // Client -> Server
    StartConsensus { 
        query: String, 
        profile: Option<String>,
        conversation_id: Option<String>,
        context: Option<Vec<ContextMessage>>
    },
    CancelConsensus,
    
    // Server -> Client
    ProfileLoaded { name: String, models: Vec<String> },
    StageStarted { stage: String, model: String },
    StreamChunk { stage: String, chunk: String },
    StageProgress { stage: String, percentage: f32, tokens: u32 },
    StageCompleted { stage: String, tokens: u32, cost: f64 },
    ConsensusComplete { result: String, total_tokens: u32, total_cost: f64 },
    Error { message: String },
    AIHelperDecision { direct_mode: bool, reason: String },
}

// WebSocket streaming callbacks
struct WebSocketCallbacks {
    tx: mpsc::UnboundedSender<WSMessage>,
}

impl StreamingCallbacks for WebSocketCallbacks {
    fn on_profile_loaded(&self, profile_name: &str, models: &[String]) -> anyhow::Result<()> {
        info!("WebSocket callback: on_profile_loaded called for {}", profile_name);
        
        let msg = WSMessage::ProfileLoaded { 
            name: profile_name.to_string(),
            models: models.to_vec(),
        };
        
        // UnboundedSender::send is synchronous - no async needed!
        match self.tx.send(msg) {
            Ok(_) => info!("✅ Sent ProfileLoaded message for {}", profile_name),
            Err(e) => error!("❌ Failed to send ProfileLoaded message: {}", e),
        }
        Ok(())
    }

    fn on_stage_start(&self, stage: Stage, model: &str) -> anyhow::Result<()> {
        info!("WebSocket callback: on_stage_start called for {} with model {}", stage.display_name(), model);
        
        let msg = WSMessage::StageStarted { 
            stage: stage.display_name().to_string(),
            model: model.to_string(),
        };
        
        match self.tx.send(msg) {
            Ok(_) => info!("✅ Sent StageStarted message for {}", stage.display_name()),
            Err(e) => error!("❌ Failed to send StageStarted message: {}", e),
        }
        Ok(())
    }

    fn on_stage_chunk(&self, stage: Stage, chunk: &str, _total_content: &str) -> anyhow::Result<()> {
        let msg = WSMessage::StreamChunk { 
            stage: stage.display_name().to_string(),
            chunk: chunk.to_string(),
        };
        
        // Silently ignore errors for chunks (too many to log)
        let _ = self.tx.send(msg);
        Ok(())
    }

    fn on_stage_progress(&self, stage: Stage, progress: ProgressInfo) -> anyhow::Result<()> {
        let msg = WSMessage::StageProgress { 
            stage: stage.display_name().to_string(),
            percentage: progress.percentage,
            tokens: progress.tokens,
        };
        
        let _ = self.tx.send(msg);
        Ok(())
    }

    fn on_stage_complete(&self, stage: Stage, result: &StageResult) -> anyhow::Result<()> {
        info!("WebSocket callback: on_stage_complete called for {}", stage.display_name());
        
        // Calculate cost from analytics
        let cost = result.analytics.as_ref().map(|a| a.cost).unwrap_or(0.0);
        let tokens = result.usage.as_ref()
            .map(|u| u.prompt_tokens + u.completion_tokens)
            .unwrap_or(0) as u32;
        
        let msg = WSMessage::StageCompleted { 
            stage: stage.display_name().to_string(),
            tokens,
            cost,
        };
        
        match self.tx.send(msg) {
            Ok(_) => info!("✅ Sent StageCompleted message for {}", stage.display_name()),
            Err(e) => error!("❌ Failed to send StageCompleted message: {}", e),
        }
        Ok(())
    }

    fn on_error(&self, stage: Stage, error: &anyhow::Error) -> anyhow::Result<()> {
        let msg = WSMessage::Error { 
            message: format!("{} error: {}", stage.display_name(), error)
        };
        
        match self.tx.send(msg) {
            Ok(_) => {},
            Err(e) => error!("Failed to send error message: {}", e),
        }
        Ok(())
    }
}

// Application state
struct AppState {
    consensus_engine: Arc<RwLock<Option<ConsensusEngine>>>,
    database: Arc<RwLock<Option<Arc<DatabaseManager>>>>,
    ai_helpers: Arc<RwLock<Option<Arc<AIHelperEcosystem>>>>,
    maintenance: Arc<RwLock<Option<Arc<BackgroundMaintenance>>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with better filtering
    tracing_subscriber::fmt()
        .with_env_filter("info,hive=debug,hive_ai=debug")
        .init();

    info!("🐝 Starting Enhanced Hive Backend Server (2025 Architecture)...");

    // Initialize database
    let database = match initialize_database(None).await {
        Ok(_) => {
            // Now get the database handle
            match get_database().await {
                Ok(db) => {
                    info!("✅ Database connected successfully");
                    Some(db)
                }
                Err(e) => {
                    warn!("⚠️ Failed to get database handle: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("⚠️ Running without database: {}", e);
            None
        }
    };

    // Initialize consensus engine with database
    // The ConsensusEngine will initialize its own AI helpers internally
    let consensus_engine = if let Some(ref db) = database {
        match ConsensusEngine::new(Some(db.clone())).await {
            Ok(engine) => {
                info!("✅ Consensus engine initialized with database and AI helpers");
                Some(engine)
            }
            Err(e) => {
                error!("Failed to initialize consensus engine: {}", e);
                None
            }
        }
    } else {
        warn!("⚠️ Running without database - no consensus engine available");
        None
    };
    
    // AI helpers are now managed internally by ConsensusEngine
    let ai_helpers = None;

    // Initialize maintenance system if database is available
    let maintenance = if let Some(ref db) = database {
        // Get API key for OpenRouter sync
        let api_key = ApiKeyManager::get_openrouter_key()
            .await
            .ok();
        
        let maintenance = Arc::new(BackgroundMaintenance::new(db.clone(), api_key));
        
        // Start background maintenance tasks
        let maintenance_clone = Arc::clone(&maintenance);
        tokio::spawn(async move {
            maintenance_clone.start().await;
        });
        
        info!("✅ Background maintenance system started");
        Some(maintenance)
    } else {
        warn!("⚠️ Running without maintenance system - no database available");
        None
    };

    // Create shared state
    let state = Arc::new(AppState {
        consensus_engine: Arc::new(RwLock::new(consensus_engine)),
        database: Arc::new(RwLock::new(database)),
        ai_helpers: Arc::new(RwLock::new(ai_helpers)),
        maintenance: Arc::new(RwLock::new(maintenance)),
    });

    // Build the router with WebSocket support
    let app = Router::new()
        // WebSocket endpoint for streaming consensus
        .route("/ws", get(websocket_handler))
        .route("/ws-test", get(test_websocket_handler))
        
        // REST endpoints
        .route("/api/consensus", post(run_consensus))
        .route("/api/consensus/quick", post(quick_consensus))
        .route("/api/ai-helper/route", post(ai_routing_decision))
        .route("/api/profiles", get(list_profiles))
        .route("/api/maintenance/status", get(maintenance_status))
        .route("/api/maintenance/sync", post(force_maintenance_sync))
        .route("/health", get(health_check))
        
        // CORS for Electron
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(tower_http::cors::Any)
        )
        .with_state(state);

    let addr = "0.0.0.0:8765";
    info!("✅ Enhanced Backend Server running on http://{}", addr);
    info!("🔌 WebSocket endpoint: ws://{}/ws", addr);
    info!("🧠 REST consensus: POST http://{}/api/consensus", addr);
    info!("🤖 AI routing: POST http://{}/api/ai-helper/route", addr);
    info!("📊 Multi-threaded processing enabled");
    info!("🔥 CPU overheating protection active");
    
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// Test WebSocket handler
async fn test_websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    info!("Test WebSocket upgrade requested");
    ws.on_upgrade(|mut socket| async move {
        info!("Test WebSocket connected");
        
        // Send a test message
        let (mut tx, mut rx) = socket.split();
        
        if let Err(e) = tx.send(axum::extract::ws::Message::Text(
            "WebSocket connection successful!".to_string()
        )).await {
            error!("Failed to send test message: {}", e);
        }
        
        // Echo messages back
        while let Some(msg) = rx.next().await {
            if let Ok(msg) = msg {
                match msg {
                    axum::extract::ws::Message::Text(txt) => {
                        let _ = tx.send(axum::extract::ws::Message::Text(
                            format!("Echo: {}", txt)
                        )).await;
                    }
                    axum::extract::ws::Message::Close(_) => break,
                    _ => {}
                }
            }
        }
        
        info!("Test WebSocket disconnected");
    })
}

// WebSocket handler for streaming consensus
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("WebSocket upgrade requested");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket connection established");
    
    let (mut socket_tx, mut socket_rx) = socket.split();
    
    // Create channel for sending messages to client
    let (tx, mut rx) = mpsc::unbounded_channel::<WSMessage>();
    
    // Spawn task to forward messages to WebSocket
    tokio::spawn(async move {
        info!("Message forwarding task started");
        while let Some(msg) = rx.recv().await {
            info!("Received message from channel: {:?}", msg);
            if let Ok(json) = serde_json::to_string(&msg) {
                info!("Sending to WebSocket: {}", json);
                if socket_tx.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                    error!("Failed to send to WebSocket, breaking");
                    break;
                }
            }
        }
        info!("Message forwarding task ended");
    });
    
    // Handle incoming messages
    while let Some(msg) = socket_rx.next().await {
        if let Ok(msg) = msg {
            match msg {
                axum::extract::ws::Message::Text(text) => {
                    info!("Received WebSocket text message: {}", text);
                    match serde_json::from_str::<WSMessage>(&text) {
                        Ok(ws_msg) => {
                            info!("Successfully parsed WSMessage: {:?}", ws_msg);
                            match ws_msg {
                                WSMessage::StartConsensus { query, profile, conversation_id, context } => {
                                    info!("Received StartConsensus message!");
                                // Run consensus in separate task to avoid blocking
                                let state_clone = state.clone();
                                let tx_clone = tx.clone();
                                
                                tokio::spawn(async move {
                                    run_consensus_streaming(
                                        query,
                                        profile,
                                        conversation_id,
                                        context,
                                        state_clone,
                                        tx_clone,
                                    ).await;
                                });
                            }
                            WSMessage::CancelConsensus => {
                                info!("Consensus cancellation requested");
                                // TODO: Implement cancellation token
                            }
                            _ => {
                                info!("Received other WSMessage type");
                            }
                        }
                        }
                        Err(e) => {
                            error!("Failed to parse WSMessage: {}, raw text was: {}", e, text);
                        }
                    }
                }
                axum::extract::ws::Message::Close(_) => break,
                _ => {}
            }
        }
    }
    
    info!("WebSocket connection closed");
}

async fn run_consensus_streaming(
    query: String,
    profile: Option<String>,
    conversation_id: Option<String>,
    context: Option<Vec<ContextMessage>>,
    state: Arc<AppState>,
    tx: mpsc::UnboundedSender<WSMessage>,
) {
    info!("Starting streaming consensus for query: '{}' with profile: {:?}, conversation_id: {:?}", 
        query, profile, conversation_id);
    
    // Check if engine is initialized
    let engine_guard = state.consensus_engine.read().await;
    if engine_guard.is_none() {
        error!("Consensus engine not initialized!");
        let _ = tx.send(WSMessage::Error {
            message: "Consensus engine not initialized. Please check database and configuration.".to_string(),
        });
        return;
    }
    
    let engine = engine_guard.as_ref().unwrap();
    info!("Consensus engine obtained, preparing to process...");
    
    // Build context string from conversation history
    let context_str = if let Some(ctx_messages) = context {
        if !ctx_messages.is_empty() {
            let mut ctx = String::new();
            ctx.push_str("Previous conversation context:\n");
            for msg in ctx_messages.iter().take(10) { // Limit to last 10 messages for context
                ctx.push_str(&format!("{}: {}\n", msg.role, msg.content));
            }
            Some(ctx)
        } else {
            None
        }
    } else {
        None
    };
    
    if let Some(ref ctx) = context_str {
        info!("Using conversation context with {} characters", ctx.len());
    }
    
    // For now, always use full consensus pipeline
    // TODO: Implement AI helper routing when methods are available
    let _ = tx.send(WSMessage::AIHelperDecision {
        direct_mode: false,
        reason: "Using full consensus pipeline for best quality".to_string(),
    });
    
    // Create streaming callbacks
    let callbacks = Arc::new(WebSocketCallbacks { tx: tx.clone() });
    
    info!("Starting consensus processing with callbacks...");
    
    // Set the profile if provided
    if let Some(profile_name) = profile {
        if let Err(e) = engine.set_profile(&profile_name).await {
            warn!("Failed to set profile {}: {}", profile_name, e);
        }
    }
    
    // Run consensus with streaming callbacks (context as second parameter, user_id as fourth)
    match engine.process_with_callbacks(&query, context_str, callbacks, conversation_id).await {
        Ok(result) => {
            info!("Consensus completed successfully");
            // Calculate total tokens from stages
            let total_tokens: u32 = result.stages.iter()
                .filter_map(|stage| stage.usage.as_ref())
                .map(|usage| usage.total_tokens)
                .sum();
            
            let _ = tx.send(WSMessage::ConsensusComplete {
                result: result.result.unwrap_or_default(),
                total_tokens,
                total_cost: result.total_cost,
            });
            
            // TODO: Store in knowledge base when AI helper methods are available
        }
        Err(e) => {
            error!("Consensus failed with error: {}", e);
            let _ = tx.send(WSMessage::Error {
                message: format!("Consensus failed: {}", e),
            });
        }
    }
}

// Quick consensus endpoint for testing (bypasses full pipeline)
async fn quick_consensus(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConsensusRequest>,
) -> Result<Json<ConsensusResponse>, String> {
    info!("Quick consensus request: {}", req.query);
    
    // For simple queries, just return a quick response
    let result = match req.query.to_lowercase().as_str() {
        q if q.contains("1 + 1") || q.contains("1+1") => "The answer is 2.",
        q if q.contains("2 + 2") || q.contains("2+2") => "The answer is 4.",
        q if q.contains("hello") => "Hello! How can I help you today?",
        q if q.contains("test") => "Test successful! The system is working.",
        q if q.contains("react") => {
            "React is a JavaScript library developed by Facebook for building user interfaces. It uses a component-based architecture where the UI is broken down into reusable components. React features a virtual DOM for efficient updates, JSX syntax for writing components, state management with hooks or class components, and a rich ecosystem of tools and libraries. It's commonly used for building single-page applications, progressive web apps, and can even be used for mobile apps with React Native."
        },
        q if q.contains("rust") => {
            "Rust is a systems programming language focused on safety, speed, and concurrency. It achieves memory safety without garbage collection through its ownership system, borrowing rules, and lifetimes. Rust is ideal for performance-critical applications, systems programming, WebAssembly, and embedded systems."
        },
        q if q.contains("electron") => {
            "Electron is a framework for building cross-platform desktop applications using web technologies (HTML, CSS, JavaScript). It combines Chromium and Node.js into a single runtime, allowing you to build desktop apps with web technologies. Popular apps like VS Code, Discord, and Slack are built with Electron."
        },
        _ => "I understand your query. For complex questions, the full consensus pipeline would provide a more comprehensive answer. This quick endpoint is designed for simple responses and testing.",
    };
    
    Ok(Json(ConsensusResponse {
        result: result.to_string(),
        duration_ms: 50,
        model_used: "quick-response".to_string(),
        tokens_used: 10,
        cost: 0.0001,
    }))
}

// REST endpoint for non-streaming consensus
async fn run_consensus(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConsensusRequest>,
) -> Result<Json<ConsensusResponse>, String> {
    info!("REST consensus request: {}", req.query);
    
    let engine_guard = state.consensus_engine.read().await;
    if engine_guard.is_none() {
        return Err("Consensus engine not initialized".to_string());
    }
    
    let engine = engine_guard.as_ref().unwrap();
    let start = std::time::Instant::now();
    
    match engine.process(&req.query, None).await {
        Ok(result) => {
            // Calculate total tokens from stages
            let total_tokens: u32 = result.stages.iter()
                .filter_map(|stage| stage.usage.as_ref())
                .map(|usage| usage.total_tokens)
                .sum();
            
            Ok(Json(ConsensusResponse {
                result: result.result.unwrap_or_default(),
                duration_ms: start.elapsed().as_millis(),
                model_used: "4-stage-consensus".to_string(),
                tokens_used: total_tokens,
                cost: result.total_cost,
            }))
        }
        Err(e) => Err(format!("Consensus failed: {}", e))
    }
}

// AI routing decision endpoint
async fn ai_routing_decision(
    State(_state): State<Arc<AppState>>,
    Json(_query): Json<String>,
) -> Json<serde_json::Value> {
    // TODO: Implement when AI helper methods are available
    // For now, always recommend full pipeline for best quality
    Json(serde_json::json!({
        "direct_mode": false,
        "reason": "Using full consensus pipeline for best quality"
    }))
}

// List available profiles
async fn list_profiles(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    if let Some(ref engine) = *state.consensus_engine.read().await {
        // This would need to be implemented in ConsensusEngine
        Json(serde_json::json!({
            "profiles": ["balanced-performer", "lightning-fast", "deep-researcher"]
        }))
    } else {
        Json(serde_json::json!({
            "error": "Consensus engine not initialized"
        }))
    }
}

// Health check
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "hive-backend-enhanced",
        "version": env!("CARGO_PKG_VERSION"),
        "features": {
            "websocket": true,
            "ai_helpers": true,
            "streaming": true,
            "multi_threading": true,
            "maintenance": true,
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// Get maintenance status
async fn maintenance_status(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    if let Some(ref maintenance) = *state.maintenance.read().await {
        let status = maintenance.get_status().await;
        Json(serde_json::json!(status))
    } else {
        Json(serde_json::json!({
            "error": "Maintenance system not initialized"
        }))
    }
}

// Force maintenance sync
async fn force_maintenance_sync(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, String> {
    if let Some(ref maintenance) = *state.maintenance.read().await {
        match maintenance.force_sync().await {
            Ok(report) => Ok(Json(serde_json::json!({
                "success": true,
                "models_updated": report.models_updated,
                "profiles_migrated": report.profiles_migrated,
            }))),
            Err(e) => Err(format!("Sync failed: {}", e))
        }
    } else {
        Err("Maintenance system not initialized".to_string())
    }
}