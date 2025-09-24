//! HTTP connection pooling for OpenRouter
//!
//! Implements connection pooling as per CONSENSUS_ARCHITECTURE_2025.md
//! to reduce latency and improve throughput.

use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;

/// Global HTTP client with connection pooling
pub static POOLED_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        // Connection pooling settings
        .pool_max_idle_per_host(4)
        .pool_idle_timeout(Duration::from_secs(90))
        
        // Use HTTP/2 for multiplexing
        .http2_prior_knowledge()
        
        // Timeouts
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(300)) // 5 minutes for long consensus
        
        // Headers
        .user_agent("Hive-AI/2.0 (Rust; High-Performance)")
        
        // Build
        .build()
        .expect("Failed to create HTTP client")
});

/// Performance monitoring for HTTP requests
#[derive(Debug, Clone, Default)]
pub struct HttpMetrics {
    pub total_requests: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub average_latency_ms: f64,
    pub connection_reuse_count: u64,
}

use std::sync::Arc;
use parking_lot::Mutex;

pub static METRICS: Lazy<Arc<Mutex<HttpMetrics>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HttpMetrics::default()))
});

/// Wrapper for making requests with the pooled client
pub async fn pooled_request(
    url: &str,
    api_key: &str,
    body: serde_json::Value,
) -> Result<reqwest::Response, reqwest::Error> {
    let start = std::time::Instant::now();
    
    let response = POOLED_CLIENT
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://github.com/hivetechs/hive")
        .header("X-Title", "Hive AI Consensus")
        .json(&body)
        .send()
        .await?;
    
    // Update metrics
    let elapsed = start.elapsed();
    let mut metrics = METRICS.lock();
    metrics.total_requests += 1;
    metrics.average_latency_ms = 
        (metrics.average_latency_ms * (metrics.total_requests - 1) as f64 + elapsed.as_millis() as f64) 
        / metrics.total_requests as f64;
    
    Ok(response)
}

/// Get current metrics
pub fn get_metrics() -> HttpMetrics {
    METRICS.lock().clone()
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        // Ensure client is created successfully
        let _ = &*POOLED_CLIENT;
    }
    
    #[tokio::test]
    async fn test_metrics() {
        // Reset metrics
        *METRICS.lock() = HttpMetrics::default();
        
        // Simulate a request (would fail without valid endpoint)
        let _ = pooled_request(
            "https://example.com",
            "test-key",
            serde_json::json!({"test": true}),
        ).await;
        
        let metrics = get_metrics();
        assert!(metrics.total_requests > 0);
    }
}