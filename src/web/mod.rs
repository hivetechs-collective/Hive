use axum::{
    extract::{ws::WebSocket, Path, Query, State, WebSocketUpgrade},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

mod handlers;
mod static_files;
mod websocket;

pub use handlers::*;
pub use static_files::*;
pub use websocket::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub extension: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProgress {
    pub stage: String,
    pub progress: u8,
    pub message: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRequest {
    pub message: String,
    pub context: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub workspace_path: PathBuf,
    pub tx: broadcast::Sender<String>,
}

pub async fn start_web_server(
    port: u16,
    workspace_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, _) = broadcast::channel(1024);

    let app_state = AppState { workspace_path, tx };

    let app = Router::new()
        // Static files
        .route("/", get(serve_index))
        .route("/static/*path", get(serve_static))
        // API endpoints
        .route("/api/files", get(list_files))
        .route("/api/files/*path", get(get_file_content))
        .route("/api/chat", post(handle_chat))
        // WebSocket endpoint
        .route("/ws", get(websocket_handler))
        // Serve the main HTML page for any other route
        .fallback(serve_index)
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(Arc::new(app_state));

    info!("Starting web server on http://localhost:{}", port);
    info!("Workspace: {}", workspace_path.display());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
