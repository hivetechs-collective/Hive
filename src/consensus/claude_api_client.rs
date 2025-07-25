//! Claude API Client
//! 
//! Implements the Anthropic Messages API with streaming support

use anyhow::{Context, Result};
use async_stream::stream;
use futures::stream::Stream;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn};

use super::claude_auth::{ClaudeAuth, AuthType};
use crate::consensus::streaming::StreamingCallbacks;

/// Claude API base URL
const CLAUDE_API_BASE: &str = "https://api.anthropic.com";

/// Claude API version
const CLAUDE_API_VERSION: &str = "2023-06-01";

/// Default model to use
const DEFAULT_MODEL: &str = "claude-3-5-sonnet-20241022";

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// Claude API request
#[derive(Debug, Clone, Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

/// Claude API response (non-streaming)
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    id: String,
    model: String,
    role: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: Usage,
}

/// Content block in response
#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

/// Usage statistics
#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Streaming event types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: MessageStartData },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: usize, content_block: ContentBlockData },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: DeltaData },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessageDeltaData, usage: Usage },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "error")]
    Error { error: ErrorData },
}

#[derive(Debug, Deserialize)]
struct MessageStartData {
    id: String,
    model: String,
    role: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlockData {
    #[serde(rename = "type")]
    block_type: String,
}

#[derive(Debug, Deserialize)]
struct DeltaData {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageDeltaData {
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorData {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// Claude API client
pub struct ClaudeApiClient {
    client: Client,
    auth: Arc<ClaudeAuth>,
    model: String,
}

impl ClaudeApiClient {
    /// Create a new Claude API client
    pub fn new(auth: Arc<ClaudeAuth>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minute timeout for long responses
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            auth,
            model: DEFAULT_MODEL.to_string(),
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    /// Send a message to Claude (non-streaming)
    pub async fn send_message(
        &self,
        messages: Vec<Message>,
        system_prompt: Option<String>,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String> {
        let auth_header = self.auth.get_auth_header().await?;
        
        let request_body = ClaudeRequest {
            model: self.model.clone(),
            messages,
            max_tokens,
            temperature,
            stream: false,
            system: system_prompt,
        };

        let response = self.client
            .post(format!("{}/v1/messages", CLAUDE_API_BASE))
            .header("anthropic-version", CLAUDE_API_VERSION)
            .header("x-api-key", auth_header)
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Claude API error: {}", error_text));
        }

        let claude_response: ClaudeResponse = response.json().await?;
        
        // Extract text from content blocks
        let text = claude_response.content
            .into_iter()
            .filter_map(|block| block.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(text)
    }

    /// Send a message to Claude with streaming
    pub async fn send_message_streaming(
        &self,
        messages: Vec<Message>,
        system_prompt: Option<String>,
        max_tokens: u32,
        temperature: f32,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<String> {
        let auth_header = self.auth.get_auth_header().await?;
        
        let request_body = ClaudeRequest {
            model: self.model.clone(),
            messages,
            max_tokens,
            temperature,
            stream: true,
            system: system_prompt,
        };

        let response = self.client
            .post(format!("{}/v1/messages", CLAUDE_API_BASE))
            .header("anthropic-version", CLAUDE_API_VERSION)
            .header("x-api-key", auth_header)
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Claude API error: {}", error_text));
        }

        // Process streaming response
        let mut full_response = String::new();
        let mut stream = self.create_event_stream(response);

        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::MessageStart { message }) => {
                    debug!("Stream started: model={}, id={}", message.model, message.id);
                    callbacks.on_stage_start(crate::consensus::types::Stage::Generator, &message.model)?;
                }
                Ok(StreamEvent::ContentBlockDelta { index: _, delta }) => {
                    if let Some(text) = delta.text {
                        full_response.push_str(&text);
                        callbacks.on_stage_chunk(
                            crate::consensus::types::Stage::Generator,
                            &text,
                            &full_response
                        )?;
                    }
                }
                Ok(StreamEvent::MessageDelta { delta, usage }) => {
                    debug!("Message delta: {:?}, tokens: in={}, out={}", 
                        delta.stop_reason, usage.input_tokens, usage.output_tokens);
                }
                Ok(StreamEvent::MessageStop) => {
                    debug!("Stream completed");
                    break;
                }
                Ok(StreamEvent::Error { error }) => {
                    error!("Stream error: {} - {}", error.error_type, error.message);
                    return Err(anyhow::anyhow!("Claude API error: {}", error.message));
                }
                Ok(event) => {
                    debug!("Other event: {:?}", event);
                }
                Err(e) => {
                    warn!("Failed to parse event: {}", e);
                }
            }
        }

        Ok(full_response)
    }

    /// Create an event stream from the response
    fn create_event_stream(&self, response: Response) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
        Box::pin(stream! {
            let stream = response.bytes_stream();
            let mut reader = BufReader::new(tokio_util::io::StreamReader::new(
                stream.map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
            ));
            
            let mut buffer = String::new();
            
            loop {
                buffer.clear();
                match reader.read_line(&mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let line = buffer.trim();
                        
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                break;
                            }
                            
                            match serde_json::from_str::<StreamEvent>(data) {
                                Ok(event) => yield Ok(event),
                                Err(e) => {
                                    debug!("Failed to parse event: {} - data: {}", e, data);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Stream read error: {}", e);
                        break;
                    }
                }
            }
        })
    }

    /// Create a simple message
    pub fn create_message(role: MessageRole, content: &str) -> Message {
        Message {
            role,
            content: content.to_string(),
        }
    }

    /// Check if Claude API is available
    pub async fn health_check(&self) -> Result<bool> {
        // Try to get auth header first
        let auth_header = match self.auth.get_auth_header().await {
            Ok(header) => header,
            Err(_) => return Ok(false),
        };

        // Send a minimal request to check API availability
        let messages = vec![
            Self::create_message(MessageRole::User, "Hi"),
        ];

        match self.send_message(messages, None, 10, 0.0).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Claude API health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_message_creation() {
        let msg = ClaudeApiClient::create_message(MessageRole::User, "Hello, Claude!");
        assert_eq!(msg.content, "Hello, Claude!");
        match msg.role {
            MessageRole::User => (),
            _ => panic!("Wrong role"),
        }
    }

    #[test]
    fn test_request_serialization() {
        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            messages: vec![
                Message {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                },
            ],
            max_tokens: 1024,
            temperature: 0.7,
            stream: false,
            system: Some("You are a helpful assistant".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"claude-3-5-sonnet-20241022\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"system\":\"You are a helpful assistant\""));
    }
}