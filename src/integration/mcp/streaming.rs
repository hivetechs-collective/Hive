//! MCP streaming response handling
//!
//! Provides real-time streaming for consensus and code transformation

use super::protocol::{McpMessage, McpMessageContent, McpNotification, ToolContent};
use crate::consensus::streaming::ConsensusEvent;
use crate::consensus::types::Stage;
use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use futures::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::{info, error, debug};
use serde_json::json;

/// Streaming handler for MCP responses
pub struct StreamingHandler {
    sender: mpsc::Sender<McpMessage>,
}

impl StreamingHandler {
    /// Create new streaming handler
    pub fn new(sender: mpsc::Sender<McpMessage>) -> Self {
        Self { sender }
    }

    /// Handle streaming response from consensus engine
    pub async fn handle_consensus_stream(
        &self,
        stream: Pin<Box<dyn Stream<Item = ConsensusEvent> + Send>>,
        request_id: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut stream = stream;
        let mut buffer = String::new();
        let mut stage_buffer = String::new();
        let mut current_stage = Stage::Generator; // Default stage

        while let Some(event) = stream.next().await {
            match event {
                ConsensusEvent::StageStarted { stage } => {
                    current_stage = stage.clone();
                    stage_buffer.clear();
                    
                    // Send stage start notification
                    self.send_progress_notification(
                        &format!("Stage started: {:?}", stage),
                        request_id.clone(),
                        Some(json!({
                            "stage": format!("{:?}", stage),
                            "status": "started"
                        }))
                    ).await?;
                }
                ConsensusEvent::Token { token, .. } => {
                    buffer.push_str(&token);
                    stage_buffer.push_str(&token);
                    
                    // Send token update every 50 characters for better UX
                    if buffer.len() % 50 == 0 {
                        self.send_content_notification(
                            &buffer,
                            request_id.clone(),
                            Some(json!({
                                "stage": format!("{:?}", current_stage),
                                "partial": true
                            }))
                        ).await?;
                    }
                }
                ConsensusEvent::StageCompleted { stage, response } => {
                    // Send stage completion
                    self.send_progress_notification(
                        &format!("Stage completed: {:?}", stage),
                        request_id.clone(),
                        Some(json!({
                            "stage": format!("{:?}", stage),
                            "status": "completed",
                            "response": response
                        }))
                    ).await?;
                }
                ConsensusEvent::FinalResponse { response } => {
                    // Send final response
                    self.send_content_notification(
                        &response,
                        request_id.clone(),
                        Some(json!({
                            "final": true
                        }))
                    ).await?;
                }
                ConsensusEvent::Error { error } => {
                    error!("Stream error: {}", error);
                    self.send_error_notification(
                        &error,
                        request_id.clone()
                    ).await?;
                    return Err(anyhow!("Stream error: {}", error));
                }
                ConsensusEvent::Progress { stage, percentage } => {
                    // Send progress update
                    self.send_progress_notification(
                        &format!("Progress: {:?} - {:.0}%", stage, percentage * 100.0),
                        request_id.clone(),
                        Some(json!({
                            "stage": format!("{:?}", stage),
                            "percentage": percentage
                        }))
                    ).await?;
                }
            }
        }

        Ok(())
    }

    /// Send progress notification
    async fn send_progress_notification(
        &self,
        message: &str,
        request_id: Option<serde_json::Value>,
        data: Option<serde_json::Value>,
    ) -> Result<()> {
        let notification = McpNotification {
            method: "$/progress".to_string(),
            params: json!({
                "message": message,
                "requestId": request_id,
                "data": data
            }),
        };

        self.send_notification(notification).await
    }

    /// Send content notification
    async fn send_content_notification(
        &self,
        content: &str,
        request_id: Option<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let notification = McpNotification {
            method: "$/contentUpdate".to_string(),
            params: json!({
                "content": content,
                "requestId": request_id,
                "metadata": metadata
            }),
        };

        self.send_notification(notification).await
    }

    /// Send error notification
    async fn send_error_notification(
        &self,
        error: &str,
        request_id: Option<serde_json::Value>,
    ) -> Result<()> {
        let notification = McpNotification {
            method: "$/error".to_string(),
            params: json!({
                "error": error,
                "requestId": request_id
            }),
        };

        self.send_notification(notification).await
    }

    /// Send notification to client
    async fn send_notification(&self, notification: McpNotification) -> Result<()> {
        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: None,
            content: McpMessageContent::Notification(notification),
        };

        self.sender.send(message).await
            .map_err(|e| anyhow!("Failed to send notification: {}", e))?;

        Ok(())
    }
}

/// Streaming response wrapper for tools
pub struct StreamingToolResponse {
    receiver: mpsc::Receiver<ToolContent>,
}

impl StreamingToolResponse {
    /// Create new streaming tool response
    pub fn new() -> (Self, mpsc::Sender<ToolContent>) {
        let (sender, receiver) = mpsc::channel(100);
        (Self { receiver }, sender)
    }

    /// Convert to stream
    pub fn into_stream(self) -> impl Stream<Item = ToolContent> {
        StreamingResponseStream {
            receiver: self.receiver,
        }
    }
}

/// Stream implementation for tool responses
struct StreamingResponseStream {
    receiver: mpsc::Receiver<ToolContent>,
}

impl Stream for StreamingResponseStream {
    type Item = ToolContent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(cx) {
            Poll::Ready(Some(content)) => Poll::Ready(Some(content)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// WebSocket connection handler
pub struct WebSocketHandler {
    sender: mpsc::Sender<McpMessage>,
    receiver: mpsc::Receiver<McpMessage>,
}

impl WebSocketHandler {
    /// Create new WebSocket handler
    pub fn new() -> (Self, mpsc::Receiver<McpMessage>, mpsc::Sender<McpMessage>) {
        let (client_sender, server_receiver) = mpsc::channel(100);
        let (server_sender, client_receiver) = mpsc::channel(100);
        
        (
            Self {
                sender: server_sender.clone(),
                receiver: server_receiver,
            },
            client_receiver,
            client_sender,
        )
    }

    /// Process incoming WebSocket messages
    pub async fn process_messages(&mut self) -> Result<()> {
        while let Some(message) = self.receiver.recv().await {
            debug!("Processing WebSocket message");
            // Messages are already handled by the server
            // This is just for logging/monitoring
        }

        Ok(())
    }
}

/// Enhanced streaming capabilities for consensus
pub struct EnhancedStreaming {
    /// Enable token-level streaming
    pub token_streaming: bool,
    /// Enable stage progress updates
    pub stage_updates: bool,
    /// Enable partial response updates
    pub partial_updates: bool,
    /// Update frequency (tokens)
    pub update_frequency: usize,
}

impl Default for EnhancedStreaming {
    fn default() -> Self {
        Self {
            token_streaming: true,
            stage_updates: true,
            partial_updates: true,
            update_frequency: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::wrappers::ReceiverStream;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_streaming_handler() {
        let (sender, mut receiver) = mpsc::channel(10);
        let handler = StreamingHandler::new(sender);

        // Send test notification
        handler.send_progress_notification(
            "Test progress",
            Some(json!(123)),
            None
        ).await.unwrap();

        // Verify notification received
        let message = receiver.recv().await.unwrap();
        match message.content {
            McpMessageContent::Notification(notif) => {
                assert_eq!(notif.method, "$/progress");
            }
            _ => panic!("Expected notification"),
        }
    }

    #[tokio::test]
    async fn test_streaming_response() {
        let (response, sender) = StreamingToolResponse::new();
        
        // Send test content
        sender.send(ToolContent::Text { 
            text: "Test content".to_string() 
        }).await.unwrap();
        drop(sender);

        // Collect stream
        let mut stream = response.into_stream();
        let items: Vec<_> = stream.collect().await;
        
        assert_eq!(items.len(), 1);
        match &items[0] {
            ToolContent::Text { text } => {
                assert_eq!(text, "Test content");
            }
            _ => panic!("Expected text content"),
        }
    }
}