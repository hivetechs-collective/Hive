//! MCP performance optimizations
//!
//! Provides caching, connection pooling, and performance monitoring

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use std::time::{Duration, Instant};
use tracing::{info, debug, warn, error};
use tokio::time::sleep;
use lru::LruCache;
use std::num::NonZeroUsize;

/// Performance manager for MCP operations
pub struct PerformanceManager {
    cache: Arc<RwLock<ToolResultCache>>,
    connection_pool: Arc<ConnectionPool>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
    config: PerformanceConfig,
}

/// Performance configuration
#[derive(Clone, Debug)]
pub struct PerformanceConfig {
    pub cache_enabled: bool,
    pub cache_max_size: usize,
    pub cache_ttl_seconds: u64,
    pub connection_pool_size: usize,
    pub request_timeout_seconds: u64,
    pub enable_compression: bool,
    pub parallel_tool_execution: bool,
    pub max_concurrent_tools: usize,
    pub enable_result_streaming: bool,
    pub metrics_collection_interval_seconds: u64,
}

/// Tool result cache
pub struct ToolResultCache {
    cache: LruCache<String, CachedToolResult>,
    hit_count: u64,
    miss_count: u64,
    eviction_count: u64,
}

/// Cached tool result
#[derive(Clone, Debug)]
pub struct CachedToolResult {
    pub result: serde_json::Value,
    pub cached_at: Instant,
    pub ttl: Duration,
    pub access_count: u64,
    pub cache_key: String,
}

/// Connection pool for MCP clients
pub struct ConnectionPool {
    connections: Arc<RwLock<Vec<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    config: ConnectionPoolConfig,
    metrics: Arc<RwLock<ConnectionPoolMetrics>>,
}

/// Pooled connection
#[derive(Clone, Debug)]
pub struct PooledConnection {
    pub id: String,
    pub created_at: Instant,
    pub last_used: Instant,
    pub use_count: u64,
    pub is_healthy: bool,
    pub client_info: Option<ClientInfo>,
}

/// Client information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub client_id: String,
    pub client_type: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

/// Connection pool configuration
#[derive(Clone, Debug)]
pub struct ConnectionPoolConfig {
    pub max_connections: usize,
    pub min_idle_connections: usize,
    pub max_idle_time_seconds: u64,
    pub connection_timeout_seconds: u64,
    pub health_check_interval_seconds: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

/// Connection pool metrics
#[derive(Clone, Debug, Default)]
pub struct ConnectionPoolMetrics {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub failed_connections: u64,
    pub connection_errors: u64,
    pub average_connection_time_ms: f64,
    pub peak_connections: usize,
}

/// Performance metrics
#[derive(Clone, Debug, Default)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub min_response_time_ms: f64,
    pub max_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub compression_ratio: f64,
    pub throughput_requests_per_second: f64,
    pub memory_usage_mb: f64,
    pub active_tool_executions: u64,
    pub queued_requests: u64,
}

/// Request performance tracker
#[derive(Debug)]
pub struct RequestTracker {
    pub request_id: String,
    pub start_time: Instant,
    pub tool_name: String,
    pub cache_hit: bool,
    pub compressed: bool,
}

impl PerformanceManager {
    /// Create new performance manager
    pub fn new(config: PerformanceConfig) -> Self {
        let cache = Arc::new(RwLock::new(ToolResultCache::new(config.cache_max_size)));
        let connection_pool = Arc::new(ConnectionPool::new(ConnectionPoolConfig {
            max_connections: config.connection_pool_size,
            min_idle_connections: config.connection_pool_size / 4,
            max_idle_time_seconds: 300, // 5 minutes
            connection_timeout_seconds: 30,
            health_check_interval_seconds: 60,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }));
        let metrics = Arc::new(RwLock::new(PerformanceMetrics::default()));

        Self {
            cache,
            connection_pool,
            metrics,
            config,
        }
    }

    /// Start performance monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        // Start metrics collection task
        self.start_metrics_collection().await;
        
        // Start connection pool maintenance
        self.connection_pool.start_maintenance().await?;
        
        // Start cache cleanup
        self.start_cache_cleanup().await;

        info!("Performance monitoring started");
        Ok(())
    }

    /// Get cached tool result
    pub async fn get_cached_result(&self, cache_key: &str) -> Option<serde_json::Value> {
        if !self.config.cache_enabled {
            return None;
        }

        let mut cache = self.cache.write().await;
        
        // Check if entry exists and is valid
        let (is_hit, result) = if let Some(cached_result) = cache.cache.get_mut(cache_key) {
            // Check TTL
            if cached_result.cached_at.elapsed() <= cached_result.ttl {
                cached_result.access_count += 1;
                (true, Some(cached_result.result.clone()))
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };
        
        if is_hit {
            cache.hit_count += 1;
            debug!("Cache hit for key: {}", cache_key);
            return result;
        }
        
        // Handle expired entries
        if !is_hit && result.is_none() {
            if let Some(_) = cache.cache.pop(cache_key) {
                cache.eviction_count += 1;
            }
        }

        cache.miss_count += 1;
        debug!("Cache miss for key: {}", cache_key);
        None
    }

    /// Cache tool result
    pub async fn cache_result(
        &self,
        cache_key: String,
        result: serde_json::Value,
    ) -> Result<()> {
        if !self.config.cache_enabled {
            return Ok(());
        }

        let cache_key_for_debug = cache_key.clone(); // Clone for the debug message
        
        let cached_result = CachedToolResult {
            result,
            cached_at: Instant::now(),
            ttl: Duration::from_secs(self.config.cache_ttl_seconds),
            access_count: 0,
            cache_key: cache_key.clone(),
        };

        let mut cache = self.cache.write().await;
        cache.cache.put(cache_key, cached_result);
        
        debug!("Cached result for key: {}", cache_key_for_debug);
        Ok(())
    }

    /// Generate cache key for tool call
    pub fn generate_cache_key(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(tool_name.as_bytes());
        hasher.update(arguments.to_string().as_bytes());
        
        format!("tool_{}_{:x}", tool_name, hasher.finalize())
    }

    /// Start request tracking
    pub async fn start_request_tracking(
        &self,
        tool_name: String,
    ) -> RequestTracker {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.active_tool_executions += 1;

        RequestTracker {
            request_id: uuid::Uuid::new_v4().to_string(),
            start_time: Instant::now(),
            tool_name,
            cache_hit: false,
            compressed: false,
        }
    }

    /// Finish request tracking
    pub async fn finish_request_tracking(
        &self,
        mut tracker: RequestTracker,
        success: bool,
    ) -> Result<()> {
        let response_time_ms = tracker.start_time.elapsed().as_millis() as f64;

        let mut metrics = self.metrics.write().await;
        
        metrics.active_tool_executions = metrics.active_tool_executions.saturating_sub(1);
        
        if success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        // Update response time metrics
        if metrics.min_response_time_ms == 0.0 || response_time_ms < metrics.min_response_time_ms {
            metrics.min_response_time_ms = response_time_ms;
        }
        
        if response_time_ms > metrics.max_response_time_ms {
            metrics.max_response_time_ms = response_time_ms;
        }

        // Calculate moving average
        let total_completed = metrics.successful_requests + metrics.failed_requests;
        metrics.average_response_time_ms = 
            (metrics.average_response_time_ms * (total_completed - 1) as f64 + response_time_ms) 
            / total_completed as f64;

        // Update cache hit rate
        let cache = self.cache.read().await;
        let total_cache_requests = cache.hit_count + cache.miss_count;
        if total_cache_requests > 0 {
            metrics.cache_hit_rate = cache.hit_count as f64 / total_cache_requests as f64;
        }

        debug!(
            "Request {} completed in {:.2}ms (success: {})",
            tracker.request_id, response_time_ms, success
        );

        Ok(())
    }

    /// Execute tool with performance optimizations
    pub async fn execute_tool_optimized<F, Fut>(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
        executor: F,
    ) -> Result<serde_json::Value>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<serde_json::Value>>,
    {
        let mut tracker = self.start_request_tracking(tool_name.to_string()).await;

        // Check cache first
        let cache_key = self.generate_cache_key(tool_name, arguments);
        if let Some(cached_result) = self.get_cached_result(&cache_key).await {
            tracker.cache_hit = true;
            self.finish_request_tracking(tracker, true).await?;
            return Ok(cached_result);
        }

        // Execute tool
        let result = match executor().await {
            Ok(result) => {
                // Cache successful result
                self.cache_result(cache_key, result.clone()).await?;
                self.finish_request_tracking(tracker, true).await?;
                result
            }
            Err(e) => {
                self.finish_request_tracking(tracker, false).await?;
                return Err(e);
            }
        };

        Ok(result)
    }

    /// Get connection from pool
    pub async fn get_connection(&self) -> Result<PooledConnection> {
        self.connection_pool.get_connection().await
    }

    /// Return connection to pool
    pub async fn return_connection(&self, connection: PooledConnection) -> Result<()> {
        self.connection_pool.return_connection(connection).await
    }

    /// Compress data if enabled
    pub async fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enable_compression {
            return Ok(data.to_vec());
        }

        // Simple compression using flate2
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        let compressed = encoder.finish()?;

        // Update compression metrics
        let mut metrics = self.metrics.write().await;
        let ratio = compressed.len() as f64 / data.len() as f64;
        metrics.compression_ratio = (metrics.compression_ratio + ratio) / 2.0;

        Ok(compressed)
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Get cache statistics
    pub async fn get_cache_statistics(&self) -> CacheStatistics {
        let cache = self.cache.read().await;
        
        CacheStatistics {
            hit_count: cache.hit_count,
            miss_count: cache.miss_count,
            eviction_count: cache.eviction_count,
            hit_rate: if cache.hit_count + cache.miss_count > 0 {
                cache.hit_count as f64 / (cache.hit_count + cache.miss_count) as f64
            } else {
                0.0
            },
            current_size: cache.cache.len(),
            max_size: cache.cache.cap().get(),
        }
    }

    /// Start metrics collection task
    async fn start_metrics_collection(&self) {
        let metrics = self.metrics.clone();
        let interval = self.config.metrics_collection_interval_seconds;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));
            
            loop {
                interval_timer.tick().await;
                
                // Collect system metrics
                if let Err(e) = Self::collect_system_metrics(&metrics).await {
                    warn!("Failed to collect system metrics: {}", e);
                }
            }
        });
    }

    /// Collect system metrics
    async fn collect_system_metrics(metrics: &Arc<RwLock<PerformanceMetrics>>) -> Result<()> {
        let mut metrics_write = metrics.write().await;
        
        // Update memory usage (simplified)
        if let Ok(memory_info) = Self::get_memory_usage() {
            metrics_write.memory_usage_mb = memory_info;
        }

        // Calculate throughput
        let total_requests = metrics_write.successful_requests + metrics_write.failed_requests;
        if total_requests > 0 {
            // This is a simplified calculation - in production, you'd track over time windows
            metrics_write.throughput_requests_per_second = total_requests as f64 / 60.0; // Rough estimate
        }

        Ok(())
    }

    /// Get memory usage in MB
    fn get_memory_usage() -> Result<f64> {
        // Simplified memory usage calculation
        // In production, use a proper system monitoring library
        Ok(50.0) // Placeholder
    }

    /// Start cache cleanup task
    async fn start_cache_cleanup(&self) {
        let cache = self.cache.clone();
        
        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
            
            loop {
                cleanup_interval.tick().await;
                
                let mut cache_write = cache.write().await;
                let current_size = cache_write.cache.len();
                
                // Remove expired entries
                let keys_to_remove: Vec<String> = cache_write.cache.iter()
                    .filter(|(_, cached_result)| {
                        cached_result.cached_at.elapsed() > cached_result.ttl
                    })
                    .map(|(key, _)| key.clone())
                    .collect();

                for key in keys_to_remove {
                    cache_write.cache.pop(&key);
                    cache_write.eviction_count += 1;
                }

                let cleaned_count = current_size - cache_write.cache.len();
                if cleaned_count > 0 {
                    debug!("Cleaned {} expired cache entries", cleaned_count);
                }
            }
        });
    }
}

impl ToolResultCache {
    /// Create new tool result cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(max_size).unwrap()),
            hit_count: 0,
            miss_count: 0,
            eviction_count: 0,
        }
    }
}

impl ConnectionPool {
    /// Create new connection pool
    pub fn new(config: ConnectionPoolConfig) -> Self {
        Self {
            connections: Arc::new(RwLock::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            config,
            metrics: Arc::new(RwLock::new(ConnectionPoolMetrics::default())),
        }
    }

    /// Get connection from pool
    pub async fn get_connection(&self) -> Result<PooledConnection> {
        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await
            .map_err(|e| anyhow!("Failed to acquire connection permit: {}", e))?;

        // Try to get existing connection
        {
            let mut connections = self.connections.write().await;
            if let Some(mut connection) = connections.pop() {
                if connection.is_healthy && 
                   connection.last_used.elapsed().as_secs() < self.config.max_idle_time_seconds {
                    connection.last_used = Instant::now();
                    connection.use_count += 1;
                    
                    // Update metrics
                    let mut metrics = self.metrics.write().await;
                    metrics.active_connections += 1;
                    metrics.idle_connections = connections.len();
                    
                    return Ok(connection);
                }
            }
        }

        // Create new connection
        self.create_new_connection().await
    }

    /// Create new connection
    async fn create_new_connection(&self) -> Result<PooledConnection> {
        let start_time = Instant::now();
        
        // Simulate connection creation delay
        sleep(Duration::from_millis(10)).await;

        let connection = PooledConnection {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: Instant::now(),
            last_used: Instant::now(),
            use_count: 1,
            is_healthy: true,
            client_info: None,
        };

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_connections += 1;
            metrics.active_connections += 1;
            
            let connection_time = start_time.elapsed().as_millis() as f64;
            metrics.average_connection_time_ms = 
                (metrics.average_connection_time_ms + connection_time) / 2.0;
            
            if metrics.active_connections > metrics.peak_connections {
                metrics.peak_connections = metrics.active_connections;
            }
        }

        Ok(connection)
    }

    /// Return connection to pool
    pub async fn return_connection(&self, connection: PooledConnection) -> Result<()> {
        {
            let mut connections = self.connections.write().await;
            connections.push(connection);

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.active_connections = metrics.active_connections.saturating_sub(1);
            metrics.idle_connections = connections.len();
        }

        Ok(())
    }

    /// Start maintenance task
    pub async fn start_maintenance(&self) -> Result<()> {
        let connections = self.connections.clone();
        let metrics = self.metrics.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut maintenance_interval = tokio::time::interval(
                Duration::from_secs(config.health_check_interval_seconds)
            );
            
            loop {
                maintenance_interval.tick().await;
                
                if let Err(e) = Self::perform_maintenance(&connections, &metrics, &config).await {
                    error!("Connection pool maintenance failed: {}", e);
                }
            }
        });

        info!("Connection pool maintenance started");
        Ok(())
    }

    /// Perform maintenance
    async fn perform_maintenance(
        connections: &Arc<RwLock<Vec<PooledConnection>>>,
        metrics: &Arc<RwLock<ConnectionPoolMetrics>>,
        config: &ConnectionPoolConfig,
    ) -> Result<()> {
        let mut connections_write = connections.write().await;
        let initial_count = connections_write.len();
        
        // Remove unhealthy or expired connections
        connections_write.retain(|conn| {
            conn.is_healthy && 
            conn.last_used.elapsed().as_secs() < config.max_idle_time_seconds
        });

        let removed_count = initial_count - connections_write.len();
        
        // Update metrics
        {
            let mut metrics_write = metrics.write().await;
            metrics_write.idle_connections = connections_write.len();
            metrics_write.total_connections = metrics_write.total_connections.saturating_sub(removed_count);
        }

        if removed_count > 0 {
            debug!("Removed {} stale connections during maintenance", removed_count);
        }

        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Serialize)]
pub struct CacheStatistics {
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub hit_rate: f64,
    pub current_size: usize,
    pub max_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_max_size: 1000,
            cache_ttl_seconds: 3600, // 1 hour
            connection_pool_size: 10,
            request_timeout_seconds: 30,
            enable_compression: true,
            parallel_tool_execution: true,
            max_concurrent_tools: 5,
            enable_result_streaming: true,
            metrics_collection_interval_seconds: 60,
        }
    }
}