//! MCP server implementation
//!
//! Core MCP server that handles JSON-RPC communication

use super::auth::AuthManager;
use super::protocol::{
    CallToolRequest, InitializeParams, InitializeResult, ListResourcesResult, ListToolsResult,
    McpCapability, McpError, McpMessage, McpMessageContent, McpRequest, McpResponse,
    ReadResourceResult, ServerCapabilities, ServerInfo, ToolResult, MCP_VERSION,
};
use super::resources::ResourceManager;
use super::streaming::{StreamingHandler, WebSocketHandler};
use super::tools::ToolRegistry;
use crate::consensus::engine::ConsensusEngine;
use crate::core::config::{self, Config};

use anyhow::{anyhow, Result};
use futures::stream::StreamExt;
use futures::SinkExt;
use hyper::{header, Method, StatusCode};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, warn};

/// MCP server
pub struct McpServer {
    tool_registry: Arc<ToolRegistry>,
    resource_manager: Arc<ResourceManager>,
    auth_manager: Arc<AuthManager>,
    consensus_engine: Arc<RwLock<ConsensusEngine>>,
    config: Arc<Config>,
    initialized: Arc<RwLock<bool>>,
}

impl McpServer {
    /// Create new MCP server
    pub async fn new() -> Result<Self> {
        let config = Arc::new(config::load_config().await?);
        let consensus_engine = Arc::new(RwLock::new(ConsensusEngine::new(None).await?));

        let tool_registry =
            Arc::new(ToolRegistry::new(consensus_engine.clone(), config.clone()).await?);

        let resource_manager = Arc::new(ResourceManager::new(config.clone()).await?);

        let auth_manager = Arc::new(AuthManager::new(config.clone()).await?);

        Ok(Self {
            tool_registry,
            resource_manager,
            auth_manager,
            consensus_engine,
            config,
            initialized: Arc::new(RwLock::new(false)),
        })
    }

    /// Start MCP server
    pub async fn start(&self, port: u16) -> Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        let server = self.clone();
        let make_svc = make_service_fn(move |_conn| {
            let server = server.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let server = server.clone();
                    async move { server.handle_request(req).await }
                }))
            }
        });

        let server_future = Server::bind(&addr).serve(make_svc);

        info!("🚀 MCP server listening on http://{}", addr);

        if let Err(e) = server_future.await {
            error!("MCP server error: {}", e);
            return Err(anyhow!("Server failed: {}", e));
        }

        Ok(())
    }

    /// Handle HTTP request
    async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let response = match self.process_request(req).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Request processing error: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(format!("Internal server error: {}", e)))
                    .unwrap()
            }
        };

        Ok(response)
    }

    /// Process HTTP request
    async fn process_request(&self, req: Request<Body>) -> Result<Response<Body>> {
        // Handle CORS preflight
        if req.method() == Method::OPTIONS {
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
                .header(
                    "Access-Control-Allow-Headers",
                    "Content-Type, Authorization, Upgrade, Connection",
                )
                .body(Body::empty())?);
        }

        // Check for WebSocket upgrade
        if req
            .headers()
            .get(header::CONNECTION)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("Upgrade"))
            .unwrap_or(false)
            && req
                .headers()
                .get(header::UPGRADE)
                .and_then(|v| v.to_str().ok())
                .map(|v| v == "websocket")
                .unwrap_or(false)
        {
            return self.handle_websocket_upgrade(req).await;
        }

        // Only accept POST for JSON-RPC
        if req.method() != Method::POST {
            return Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::from("Only POST requests are allowed"))?);
        }

        // Parse request body
        let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
        let body_str = String::from_utf8(body_bytes.to_vec())?;

        debug!("Received MCP request: {}", body_str);

        // Parse JSON-RPC message
        let message: McpMessage = match serde_json::from_str(&body_str) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to parse JSON-RPC message: {}", e);
                let error_response = McpMessage {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    content: McpMessageContent::Response(McpResponse::Error {
                        error: McpError::parse_error(),
                    }),
                };
                let response_body = serde_json::to_string(&error_response)?;
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Body::from(response_body))?);
            }
        };

        // Process message
        let response = self.handle_message(message).await;
        let response_body = serde_json::to_string(&response)?;

        debug!("Sending MCP response: {}", response_body);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", "*")
            .body(Body::from(response_body))?)
    }

    /// Handle MCP message
    async fn handle_message(&self, message: McpMessage) -> McpMessage {
        let id = message.id.clone();

        match message.content {
            McpMessageContent::Request(request) => {
                let response = self.handle_mcp_request(request).await;
                McpMessage {
                    jsonrpc: "2.0".to_string(),
                    id,
                    content: McpMessageContent::Response(response),
                }
            }
            McpMessageContent::Notification(notification) => {
                // Handle notifications (no response expected)
                self.handle_notification(notification).await;
                // Return empty response for notifications
                McpMessage {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    content: McpMessageContent::Response(McpResponse::Success {
                        result: serde_json::Value::Null,
                    }),
                }
            }
            McpMessageContent::Response(_) => {
                // Server shouldn't receive responses
                McpMessage {
                    jsonrpc: "2.0".to_string(),
                    id,
                    content: McpMessageContent::Response(McpResponse::Error {
                        error: McpError::invalid_request(),
                    }),
                }
            }
        }
    }

    /// Handle MCP request
    async fn handle_mcp_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "initialized" => self.handle_initialized().await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_call_tool(request.params).await,
            "resources/list" => self.handle_list_resources().await,
            "resources/read" => self.handle_read_resource(request.params).await,
            _ => McpResponse::Error {
                error: McpError::method_not_found(),
            },
        }
    }

    /// Handle notification
    async fn handle_notification(&self, _notification: super::protocol::McpNotification) {
        // Handle notifications like progress updates
        debug!("Received notification");
    }

    /// Handle initialize request
    async fn handle_initialize(&self, params: serde_json::Value) -> McpResponse {
        let init_params: InitializeParams = match serde_json::from_value(params) {
            Ok(params) => params,
            Err(_) => {
                return McpResponse::Error {
                    error: McpError::invalid_params(),
                }
            }
        };

        info!(
            "Initializing MCP server for client: {} v{}",
            init_params.client_info.name, init_params.client_info.version
        );

        // Validate protocol version
        if init_params.protocol_version != MCP_VERSION {
            warn!(
                "Protocol version mismatch: client={}, server={}",
                init_params.protocol_version, MCP_VERSION
            );
        }

        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities {
                logging: Some(McpCapability { experimental: None }),
                prompts: None,
                resources: Some(McpCapability { experimental: None }),
                tools: Some(McpCapability { experimental: None }),
            },
            server_info: ServerInfo {
                name: "Hive AI MCP Server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        McpResponse::Success {
            result: serde_json::to_value(result).unwrap(),
        }
    }

    /// Handle initialized notification
    async fn handle_initialized(&self) -> McpResponse {
        let mut initialized = self.initialized.write().await;
        *initialized = true;
        info!("MCP server initialization completed");

        McpResponse::Success {
            result: serde_json::Value::Null,
        }
    }

    /// Handle list tools request
    async fn handle_list_tools(&self) -> McpResponse {
        match self.tool_registry.list_tools().await {
            Ok(tools) => {
                let result = ListToolsResult { tools };
                McpResponse::Success {
                    result: serde_json::to_value(result).unwrap(),
                }
            }
            Err(e) => {
                error!("Failed to list tools: {}", e);
                McpResponse::Error {
                    error: McpError::internal_error(),
                }
            }
        }
    }

    /// Handle call tool request
    async fn handle_call_tool(&self, params: serde_json::Value) -> McpResponse {
        let call_request: CallToolRequest = match serde_json::from_value(params) {
            Ok(req) => req,
            Err(_) => {
                return McpResponse::Error {
                    error: McpError::invalid_params(),
                }
            }
        };

        match self
            .tool_registry
            .call_tool(&call_request.name, call_request.arguments)
            .await
        {
            Ok(result) => McpResponse::Success {
                result: serde_json::to_value(result).unwrap(),
            },
            Err(e) => {
                error!("Tool execution failed: {}", e);
                McpResponse::Error {
                    error: McpError::custom(-32000, format!("Tool execution failed: {}", e)),
                }
            }
        }
    }

    /// Handle list resources request
    async fn handle_list_resources(&self) -> McpResponse {
        match self.resource_manager.list_resources().await {
            Ok(resources) => {
                let result = ListResourcesResult { resources };
                McpResponse::Success {
                    result: serde_json::to_value(result).unwrap(),
                }
            }
            Err(e) => {
                error!("Failed to list resources: {}", e);
                McpResponse::Error {
                    error: McpError::internal_error(),
                }
            }
        }
    }

    /// Handle read resource request
    async fn handle_read_resource(&self, params: serde_json::Value) -> McpResponse {
        let uri: String = match params.get("uri").and_then(|v| v.as_str()) {
            Some(uri) => uri.to_string(),
            None => {
                return McpResponse::Error {
                    error: McpError::invalid_params(),
                }
            }
        };

        match self.resource_manager.read_resource(&uri).await {
            Ok(contents) => {
                let result = ReadResourceResult { contents };
                McpResponse::Success {
                    result: serde_json::to_value(result).unwrap(),
                }
            }
            Err(e) => {
                error!("Failed to read resource: {}", e);
                McpResponse::Error {
                    error: McpError::custom(-32001, format!("Resource not found: {}", e)),
                }
            }
        }
    }

    /// Handle WebSocket upgrade request
    async fn handle_websocket_upgrade(&self, req: Request<Body>) -> Result<Response<Body>> {
        info!("WebSocket upgrade requested");

        // For now, return a simple response indicating WebSocket is supported
        // Full WebSocket implementation would require tokio-tungstenite integration
        Ok(Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header(header::CONNECTION, "Upgrade")
            .header(header::UPGRADE, "websocket")
            .header("Sec-WebSocket-Accept", "dummy") // Would need proper calculation
            .body(Body::empty())?)
    }

    /// Handle WebSocket connection
    async fn handle_websocket_connection(
        &self,
        ws: tokio_tungstenite::WebSocketStream<hyper::upgrade::Upgraded>,
    ) -> Result<()> {
        info!("WebSocket connection established");

        let (mut ws_sender, mut ws_receiver) = ws.split();
        let (msg_sender, mut msg_receiver) = mpsc::channel::<McpMessage>(100);

        // Create streaming handler
        let streaming_handler = StreamingHandler::new(msg_sender.clone());

        // Spawn task to forward messages to WebSocket
        let send_task = tokio::spawn(async move {
            while let Some(message) = msg_receiver.recv().await {
                let json = match serde_json::to_string(&message) {
                    Ok(json) => json,
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                if let Err(e) = ws_sender
                    .send(tokio_tungstenite::tungstenite::Message::Text(json))
                    .await
                {
                    error!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        });

        // Process incoming WebSocket messages
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                    debug!("Received WebSocket message: {}", text);

                    // Parse and handle message
                    match serde_json::from_str::<McpMessage>(&text) {
                        Ok(message) => {
                            let response = self.handle_message(message).await;
                            msg_sender.send(response).await?;
                        }
                        Err(e) => {
                            warn!("Failed to parse WebSocket message: {}", e);
                            let error_response = McpMessage {
                                jsonrpc: "2.0".to_string(),
                                id: None,
                                content: McpMessageContent::Response(McpResponse::Error {
                                    error: McpError::parse_error(),
                                }),
                            };
                            msg_sender.send(error_response).await?;
                        }
                    }
                }
                Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                    info!("WebSocket connection closed by client");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {} // Ignore other message types
            }
        }

        // Clean up
        send_task.abort();
        info!("WebSocket connection closed");

        Ok(())
    }
}

impl Clone for McpServer {
    fn clone(&self) -> Self {
        Self {
            tool_registry: self.tool_registry.clone(),
            resource_manager: self.resource_manager.clone(),
            auth_manager: self.auth_manager.clone(),
            consensus_engine: self.consensus_engine.clone(),
            config: self.config.clone(),
            initialized: self.initialized.clone(),
        }
    }
}
