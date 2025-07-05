// OpenRouter Integration - Real API Client with Streaming Support
// Port of TypeScript openrouter-client.ts and openrouter-streaming-client.ts

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterRequest {
    pub model: String,
    pub messages: Vec<OpenRouterMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub stream: Option<bool>,
    pub provider: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Option<Message>,
    pub delta: Option<Delta>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct StreamingChunk {
    pub content: String,
    pub is_complete: bool,
    pub usage: Option<Usage>,
}

pub trait StreamingCallbacks: Send + Sync {
    fn on_start(&self);
    fn on_chunk(&self, chunk: String, total_content: String);
    fn on_progress(&self, progress: StreamingProgress);
    fn on_error(&self, error: &anyhow::Error);
    fn on_complete(&self, final_content: String, usage: Option<Usage>);
}

#[derive(Debug, Clone)]
pub struct StreamingProgress {
    pub tokens: Option<u32>,
    pub percentage: Option<f64>,
}

pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    base_url: String,
    default_headers: HashMap<String, String>,
}

impl OpenRouterClient {
    pub fn new(api_key: String) -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert("Content-Type".to_string(), "application/json".to_string());
        default_headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        default_headers.insert("HTTP-Referer".to_string(), "https://hive.ai".to_string());
        default_headers.insert("X-Title".to_string(), "Hive AI".to_string());

        Self {
            client: Client::new(),
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
            default_headers,
        }
    }

    /// Make a standard chat completion request
    pub async fn chat_completion(&self, request: OpenRouterRequest) -> Result<OpenRouterResponse> {
        let start_time = Instant::now();
        
        println!("🚀 Calling OpenRouter API: {} (temp: {:.1})", 
                 request.model, request.temperature.unwrap_or(0.7));

        let response = self.build_request("/chat/completions")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenRouter")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenRouter API error {}: {}", status, error_text);
        }

        let openrouter_response: OpenRouterResponse = response
            .json()
            .await
            .context("Failed to parse OpenRouter response")?;

        let duration = start_time.elapsed();
        println!("✅ OpenRouter response received in {:.2}s (tokens: {})", 
                 duration.as_secs_f64(),
                 openrouter_response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0));

        Ok(openrouter_response)
    }

    /// Make a streaming chat completion request
    pub async fn chat_completion_stream(
        &self,
        mut request: OpenRouterRequest,
        callbacks: Option<Box<dyn StreamingCallbacks>>,
    ) -> Result<String> {
        request.stream = Some(true);
        
        let start_time = Instant::now();
        println!("🌊 Starting streaming request: {} (temp: {:.1})", 
                 request.model, request.temperature.unwrap_or(0.7));

        if let Some(cb) = &callbacks {
            cb.on_start();
        }

        let mut response = self.build_request("/chat/completions")
            .json(&request)
            .send()
            .await
            .context("Failed to send streaming request to OpenRouter")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            let error = anyhow::anyhow!("OpenRouter streaming API error {}: {}", status, error_text);
            if let Some(cb) = &callbacks {
                cb.on_error(&error);
            }
            return Err(error);
        }

        let mut full_content = String::new();
        let mut total_tokens = 0;
        let mut chunk_count = 0;

        // Process streaming response
        while let Some(chunk) = response.chunk().await.context("Failed to read chunk")? {
            let chunk_str = String::from_utf8_lossy(&chunk);
            
            // Parse Server-Sent Events format
            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..]; // Remove "data: " prefix
                    
                    if data == "[DONE]" {
                        break;
                    }
                    
                    if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                        if let Some(choices) = parsed["choices"].as_array() {
                            if let Some(choice) = choices.first() {
                                if let Some(delta) = choice["delta"].as_object() {
                                    if let Some(content) = delta["content"].as_str() {
                                        full_content.push_str(content);
                                        chunk_count += 1;
                                        
                                        if let Some(cb) = &callbacks {
                                            cb.on_chunk(content.to_string(), full_content.clone());
                                            
                                            // Estimate progress
                                            let progress = StreamingProgress {
                                                tokens: Some(chunk_count * 2), // Rough estimate
                                                percentage: None,
                                            };
                                            cb.on_progress(progress);
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Check for usage information
                        if let Some(usage) = parsed["usage"].as_object() {
                            if let Some(tokens) = usage["total_tokens"].as_u64() {
                                total_tokens = tokens as u32;
                            }
                        }
                    }
                }
            }
        }

        let duration = start_time.elapsed();
        println!("✅ Streaming completed in {:.2}s ({} chunks, {} tokens)", 
                 duration.as_secs_f64(), chunk_count, total_tokens);

        let usage = if total_tokens > 0 {
            Some(Usage {
                prompt_tokens: 0, // Would need to estimate from request
                completion_tokens: total_tokens,
                total_tokens,
            })
        } else {
            None
        };

        if let Some(cb) = &callbacks {
            cb.on_complete(full_content.clone(), usage);
        }

        Ok(full_content)
    }

    /// Get available models
    pub async fn get_models(&self) -> Result<Vec<Value>> {
        let response = self.build_request("/models")
            .send()
            .await
            .context("Failed to fetch models from OpenRouter")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch models: {}", response.status());
        }

        let models_response: Value = response
            .json()
            .await
            .context("Failed to parse models response")?;

        Ok(models_response["data"].as_array().unwrap_or(&vec![]).clone())
    }

    /// Get generation statistics
    pub async fn get_generation_stats(&self) -> Result<Value> {
        let response = self.build_request("/generation")
            .send()
            .await
            .context("Failed to fetch generation stats")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch generation stats: {}", response.status());
        }

        response.json().await.context("Failed to parse generation stats")
    }

    /// Build a request with default headers
    fn build_request(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.post(&url);
        
        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }
        
        request
    }

    /// Validate API key format
    pub fn validate_api_key(api_key: &str) -> bool {
        // OpenRouter API keys start with "sk-or-"
        api_key.starts_with("sk-or-") && api_key.len() > 10
    }

    /// Calculate estimated cost for a request
    pub fn estimate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        input_price_per_1k: Option<f64>,
        output_price_per_1k: Option<f64>,
    ) -> f64 {
        let input_cost = input_price_per_1k.unwrap_or(0.001) * (input_tokens as f64 / 1000.0);
        let output_cost = output_price_per_1k.unwrap_or(0.002) * (output_tokens as f64 / 1000.0);
        input_cost + output_cost
    }
}

/// Error handling for OpenRouter requests
#[derive(Debug)]
pub enum OpenRouterError {
    RateLimit { retry_after: Option<u64> },
    QuotaExceeded,
    InvalidApiKey,
    ModelNotFound,
    NetworkError(String),
    ParseError(String),
    Other(String),
}

impl std::fmt::Display for OpenRouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenRouterError::RateLimit { retry_after } => {
                write!(f, "Rate limit exceeded")?;
                if let Some(retry) = retry_after {
                    write!(f, " (retry after {} seconds)", retry)?;
                }
                Ok(())
            }
            OpenRouterError::QuotaExceeded => write!(f, "API quota exceeded"),
            OpenRouterError::InvalidApiKey => write!(f, "Invalid API key"),
            OpenRouterError::ModelNotFound => write!(f, "Model not found"),
            OpenRouterError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            OpenRouterError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            OpenRouterError::Other(msg) => write!(f, "OpenRouter error: {}", msg),
        }
    }
}

impl std::error::Error for OpenRouterError {}

/// Retry policy for OpenRouter requests
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
        }
    }
}

impl RetryPolicy {
    pub async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);
                    
                    if attempt < self.max_retries {
                        let delay = self.calculate_delay(attempt);
                        println!("⚠️ Request failed (attempt {}), retrying in {:.1}s...", 
                                 attempt + 1, delay.as_secs_f64());
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }
    
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = (self.base_delay.as_millis() as f64 * self.backoff_factor.powi(attempt as i32)) as u64;
        Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as u64))
    }
}

/// Fallback manager for model selection
pub struct FallbackManager {
    fallback_chains: HashMap<String, Vec<String>>,
}

impl Default for FallbackManager {
    fn default() -> Self {
        let mut fallback_chains = HashMap::new();
        
        // Anthropic fallbacks
        fallback_chains.insert(
            "anthropic/claude-3-opus".to_string(),
            vec![
                "anthropic/claude-3.5-sonnet".to_string(),
                "anthropic/claude-3-haiku".to_string(),
                "openai/gpt-4o".to_string(),
            ]
        );
        
        // OpenAI fallbacks
        fallback_chains.insert(
            "openai/gpt-4o".to_string(),
            vec![
                "openai/gpt-4-turbo".to_string(),
                "anthropic/claude-3.5-sonnet".to_string(),
                "openai/gpt-3.5-turbo".to_string(),
            ]
        );
        
        // Google fallbacks
        fallback_chains.insert(
            "google/gemini-pro-1.5".to_string(),
            vec![
                "google/gemini-pro".to_string(),
                "anthropic/claude-3.5-sonnet".to_string(),
                "openai/gpt-4-turbo".to_string(),
            ]
        );
        
        Self { fallback_chains }
    }
}

impl FallbackManager {
    pub fn get_fallbacks(&self, model: &str) -> Vec<String> {
        self.fallback_chains.get(model).cloned().unwrap_or_default()
    }
    
    pub fn add_fallback_chain(&mut self, model: String, fallbacks: Vec<String>) {
        self.fallback_chains.insert(model, fallbacks);
    }
}

/// Simple streaming callbacks implementation
pub struct SimpleStreamingCallbacks {
    pub model_name: String,
}

impl StreamingCallbacks for SimpleStreamingCallbacks {
    fn on_start(&self) {
        println!("🌊 Starting stream for {}", self.model_name);
    }
    
    fn on_chunk(&self, chunk: String, _total_content: String) {
        print!("{}", chunk);
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }
    
    fn on_progress(&self, progress: StreamingProgress) {
        if let Some(tokens) = progress.tokens {
            if tokens % 50 == 0 { // Progress every 50 tokens
                println!("\n[{} tokens]", tokens);
            }
        }
    }
    
    fn on_error(&self, error: &anyhow::Error) {
        eprintln!("❌ Streaming error for {}: {}", self.model_name, error);
    }
    
    fn on_complete(&self, _final_content: String, usage: Option<Usage>) {
        println!("\n✅ Stream completed for {}", self.model_name);
        if let Some(u) = usage {
            println!("📊 Tokens: {} total ({} prompt + {} completion)", 
                     u.total_tokens, u.prompt_tokens, u.completion_tokens);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_validation() {
        assert!(OpenRouterClient::validate_api_key("sk-or-abcd1234567890"));
        assert!(!OpenRouterClient::validate_api_key("sk-invalid"));
        assert!(!OpenRouterClient::validate_api_key(""));
        assert!(!OpenRouterClient::validate_api_key("sk-or-"));
    }

    #[test]
    fn test_cost_estimation() {
        let client = OpenRouterClient::new("test-key".to_string());
        
        let cost = client.estimate_cost(
            "test-model",
            1000, // input tokens
            500,  // output tokens
            Some(0.001), // $0.001 per 1k input tokens
            Some(0.002)  // $0.002 per 1k output tokens
        );
        
        assert_eq!(cost, 0.002); // 1000*0.001/1000 + 500*0.002/1000 = 0.001 + 0.001 = 0.002
    }

    #[test]
    fn test_fallback_manager() {
        let manager = FallbackManager::default();
        let fallbacks = manager.get_fallbacks("anthropic/claude-3-opus");
        
        assert!(!fallbacks.is_empty());
        assert!(fallbacks.contains(&"anthropic/claude-3.5-sonnet".to_string()));
    }
}