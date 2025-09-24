/// OpenRouter Streaming Client
///
/// Advanced streaming client for real-time consensus with live token streaming,
/// progress updates, and cancellation support. Handles Server-Sent Events (SSE)
/// for OpenRouter's streaming API.
use anyhow::{Context, Result};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::Instant;

use super::client::{ChatCompletionRequest, MessageRole, OpenRouterMessage, UsageInfo};

/// Streaming options
#[derive(Debug, Clone)]
pub struct StreamingOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
}

impl Default for StreamingOptions {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            max_tokens: Some(4000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        }
    }
}

/// Stream chunk from OpenRouter
#[derive(Debug, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageInfo>,
}

/// Streaming choice
#[derive(Debug, Deserialize)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: StreamDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Delta content in stream
#[derive(Debug, Deserialize)]
pub struct StreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Streaming response
#[derive(Debug, Clone)]
pub struct StreamingResponse {
    pub id: String,
    pub model: String,
    pub content: String,
    pub usage: Option<UsageInfo>,
    pub duration_ms: u64,
    pub tokens_per_second: f32,
}

/// Progress information
#[derive(Debug, Clone)]
pub struct StreamProgress {
    pub tokens: u32,
    pub estimated_total: Option<u32>,
    pub percentage: Option<f32>,
    pub tokens_per_second: f32,
}

/// Streaming callbacks
#[derive(Clone)]
pub struct StreamingCallbacks {
    pub on_chunk: Option<Arc<dyn Fn(String, String) + Send + Sync>>,
    pub on_progress: Option<Arc<dyn Fn(StreamProgress) + Send + Sync>>,
    pub on_complete: Option<Arc<dyn Fn(StreamingResponse) + Send + Sync>>,
    pub on_error: Option<Arc<dyn Fn(anyhow::Error) + Send + Sync>>,
    pub on_start: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Default for StreamingCallbacks {
    fn default() -> Self {
        Self {
            on_chunk: None,
            on_progress: None,
            on_complete: None,
            on_error: None,
            on_start: None,
        }
    }
}

/// Streaming client state
struct StreamState {
    cancel_tx: Option<oneshot::Sender<()>>,
    stream_id: Option<String>,
}

/// OpenRouter streaming client
pub struct StreamingClient {
    client: Client,
    api_key: String,
    state: Arc<Mutex<StreamState>>,
}

impl StreamingClient {
    /// Create a new streaming client
    pub fn new(api_key: String) -> Result<Self> {
        if !api_key.starts_with("sk-or-") {
            anyhow::bail!("Invalid OpenRouter API key format. Key must start with sk-or-");
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minutes for streaming
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            api_key,
            state: Arc::new(Mutex::new(StreamState {
                cancel_tx: None,
                stream_id: None,
            })),
        })
    }

    /// Stream chat completion
    pub async fn stream_chat_completion(
        &self,
        model: &str,
        messages: Vec<OpenRouterMessage>,
        options: StreamingOptions,
        callbacks: StreamingCallbacks,
    ) -> Result<StreamingResponse> {
        let request_id = format!(
            "stream-{}-{}",
            chrono::Utc::now().timestamp_millis(),
            uuid::Uuid::new_v4().to_string()
        );

        log::info!(
            "Starting streaming request {} for model {}",
            request_id,
            model
        );

        // Create cancellation channel
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

        // Update state
        {
            let mut state = self.state.lock().await;
            state.cancel_tx = Some(cancel_tx);
            state.stream_id = Some(request_id.clone());
        }

        // Call start callback
        if let Some(on_start) = &callbacks.on_start {
            on_start();
        }

        // Perform streaming request
        let result = self
            .perform_streaming_request(
                model,
                messages,
                options,
                callbacks.clone(),
                request_id.clone(),
                cancel_rx,
            )
            .await;

        // Clear state
        {
            let mut state = self.state.lock().await;
            state.cancel_tx = None;
            state.stream_id = None;
        }

        match result {
            Ok(response) => {
                log::info!(
                    "Streaming request {} completed successfully. {} tokens in {}ms",
                    request_id,
                    response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0),
                    response.duration_ms
                );
                Ok(response)
            }
            Err(e) => {
                log::error!("Streaming request {} failed: {}", request_id, e);
                if let Some(on_error) = &callbacks.on_error {
                    on_error(anyhow::anyhow!("{}", e));
                }
                Err(e)
            }
        }
    }

    /// Perform the streaming request
    async fn perform_streaming_request(
        &self,
        model: &str,
        messages: Vec<OpenRouterMessage>,
        options: StreamingOptions,
        callbacks: StreamingCallbacks,
        request_id: String,
        mut cancel_rx: oneshot::Receiver<()>,
    ) -> Result<StreamingResponse> {
        let start_time = Instant::now();

        // Prepare request
        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages,
            temperature: options.temperature,
            max_tokens: options.max_tokens,
            stream: Some(true),
            top_p: options.top_p,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
        };

        // Send request
        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://hivetechs.io")
            .header("X-Title", "Hive.AI Streaming Consensus")
            .json(&request)
            .send()
            .await
            .context("Failed to send streaming request")?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            let error_message = match status.as_u16() {
                401 => "Invalid OpenRouter API key for streaming".to_string(),
                402 => "Insufficient credits for streaming request".to_string(),
                404 => format!("Model \"{}\" does not support streaming", model),
                429 => "Rate limit exceeded for streaming requests".to_string(),
                _ => format!("OpenRouter streaming error ({}): {}", status, error_text),
            };

            anyhow::bail!(error_message);
        }

        // Process streaming response
        let stream = response.bytes_stream();
        let (tx, mut rx) = mpsc::channel::<Result<StreamingResponse>>(1);

        // Spawn task to process stream
        let callbacks_clone = callbacks.clone();
        let model_clone = model.to_string();
        let request_id_clone = request_id.clone();

        tokio::spawn(async move {
            let result = process_stream(
                Box::pin(stream),
                callbacks_clone,
                request_id_clone,
                model_clone,
                start_time,
            )
            .await;

            let _ = tx.send(result).await;
        });

        // Wait for either completion or cancellation
        tokio::select! {
            result = rx.recv() => {
                result.ok_or_else(|| anyhow::anyhow!("Stream processing failed"))?
            }
            _ = cancel_rx => {
                log::info!("Streaming request {} was cancelled", request_id);
                anyhow::bail!("Streaming request was cancelled")
            }
        }
    }

    /// Cancel the current stream
    pub async fn cancel_stream(&self) -> bool {
        let mut state = self.state.lock().await;
        if let Some(tx) = state.cancel_tx.take() {
            tx.send(()).is_ok()
        } else {
            false
        }
    }

    /// Check if streaming is active
    pub async fn is_streaming(&self) -> bool {
        let state = self.state.lock().await;
        state.stream_id.is_some()
    }

    /// Get current stream ID
    pub async fn get_stream_id(&self) -> Option<String> {
        let state = self.state.lock().await;
        state.stream_id.clone()
    }
}

/// Process the SSE stream
async fn process_stream(
    mut stream: Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send>>,
    callbacks: StreamingCallbacks,
    request_id: String,
    model: String,
    start_time: Instant,
) -> Result<StreamingResponse> {
    let mut buffer = String::new();
    let mut full_content = String::new();
    let mut token_count = 0u32;
    let mut stream_id = request_id.clone();
    let mut final_usage: Option<UsageInfo> = None;
    let mut last_progress_time = Instant::now();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.context("Failed to read stream chunk")?;
        let chunk_str = std::str::from_utf8(&chunk).context("Invalid UTF-8 in stream")?;

        buffer.push_str(chunk_str);

        // Process complete lines
        let parts: Vec<String> = buffer.split('\n').map(|s| s.to_string()).collect();
        let lines_to_process = if parts.len() > 1 {
            // Keep the last part as incomplete
            buffer = parts.last().unwrap().clone();
            &parts[..parts.len() - 1]
        } else {
            &parts[..]
        };

        // Process complete lines
        for line in lines_to_process {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed == "data: [DONE]" {
                continue;
            }

            if let Some(json_str) = trimmed.strip_prefix("data: ") {
                match serde_json::from_str::<StreamChunk>(json_str) {
                    Ok(chunk) => {
                        if !chunk.id.is_empty() {
                            stream_id = chunk.id.clone();
                        }

                        // Process content
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                full_content.push_str(content);

                                // Estimate tokens (approximately 4 chars per token)
                                let content_tokens = std::cmp::max(1, content.len() / 4) as u32;
                                token_count += content_tokens;

                                // Call chunk callback
                                if let Some(on_chunk) = &callbacks.on_chunk {
                                    on_chunk(content.clone(), full_content.clone());
                                }

                                // Send progress updates (throttled to every 100ms)
                                if last_progress_time.elapsed() > Duration::from_millis(100) {
                                    if let Some(on_progress) = &callbacks.on_progress {
                                        let elapsed = start_time.elapsed().as_secs_f32();
                                        let tokens_per_second = if elapsed > 0.0 {
                                            token_count as f32 / elapsed
                                        } else {
                                            0.0
                                        };

                                        on_progress(StreamProgress {
                                            tokens: token_count,
                                            estimated_total: None,
                                            percentage: None,
                                            tokens_per_second,
                                        });
                                    }
                                    last_progress_time = Instant::now();
                                }
                            }

                            // Check for completion
                            if choice.finish_reason.is_some() {
                                // Send final progress
                                if let Some(on_progress) = &callbacks.on_progress {
                                    let elapsed = start_time.elapsed().as_secs_f32();
                                    let tokens_per_second = if elapsed > 0.0 {
                                        token_count as f32 / elapsed
                                    } else {
                                        0.0
                                    };

                                    on_progress(StreamProgress {
                                        tokens: token_count,
                                        estimated_total: Some(token_count),
                                        percentage: Some(100.0),
                                        tokens_per_second,
                                    });
                                }
                            }
                        }

                        // Capture usage if available
                        if let Some(usage) = chunk.usage {
                            final_usage = Some(usage);
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to parse streaming chunk: {}. Data: {}",
                            e,
                            json_str.chars().take(100).collect::<String>()
                        );
                    }
                }
            }
        }
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let tokens_per_second = if duration_ms > 0 {
        (token_count as f32 * 1000.0) / duration_ms as f32
    } else {
        0.0
    };

    let response = StreamingResponse {
        id: stream_id,
        model,
        content: full_content,
        usage: final_usage,
        duration_ms,
        tokens_per_second,
    };

    // Call completion callback
    if let Some(on_complete) = &callbacks.on_complete {
        on_complete(response.clone());
    }

    Ok(response)
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_client_creation() {
        // Valid API key
        assert!(StreamingClient::new("sk-or-test123".to_string()).is_ok());

        // Invalid API key
        assert!(StreamingClient::new("invalid-key".to_string()).is_err());
    }

    #[test]
    fn test_streaming_options_default() {
        let options = StreamingOptions::default();
        assert_eq!(options.temperature, Some(0.7));
        assert_eq!(options.max_tokens, Some(4000));
    }
}
