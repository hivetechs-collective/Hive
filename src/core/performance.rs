//! Core Performance Optimization Utilities
//!
//! This module provides the foundational performance optimization infrastructure
//! for achieving revolutionary performance targets in HiveTechs Consensus.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Performance metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub startup_time: Duration,
    pub memory_usage: u64,
    pub consensus_latency: Duration,
    pub file_parse_time: Duration,
    pub database_latency: Duration,
    pub timestamp: std::time::SystemTime,
}

/// Performance targets from CLAUDE.md (Wave 6 enhanced targets)
#[derive(Debug, Clone)]
pub struct PerformanceTargets {
    pub startup_time: Duration,      // <25ms (enhanced from 50ms)
    pub memory_usage: u64,           // <20MB (enhanced from 25MB)
    pub file_parse_time: Duration,   // <2ms/file (enhanced from 5ms)
    pub consensus_latency: Duration, // <300ms (enhanced from 500ms)
    pub database_latency: Duration,  // <1ms (enhanced from 3ms)
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            startup_time: Duration::from_millis(25),
            memory_usage: 20 * 1024 * 1024, // 20MB
            file_parse_time: Duration::from_millis(2),
            consensus_latency: Duration::from_millis(300),
            database_latency: Duration::from_millis(1),
        }
    }
}

/// Memory pool for high-frequency allocations
pub struct MemoryPool {
    buffers: Arc<RwLock<Vec<Vec<u8>>>>,
    buffer_size: usize,
    max_buffers: usize,
}

impl MemoryPool {
    pub fn new(buffer_size: usize, max_buffers: usize) -> Self {
        Self {
            buffers: Arc::new(RwLock::new(Vec::with_capacity(max_buffers))),
            buffer_size,
            max_buffers,
        }
    }

    /// Get a buffer from the pool or create a new one
    pub async fn get_buffer(&self) -> Vec<u8> {
        let mut buffers = self.buffers.write().await;
        match buffers.pop() {
            Some(buffer) => buffer,
            None => Vec::with_capacity(self.buffer_size),
        }
    }

    /// Return a buffer to the pool
    pub async fn return_buffer(&self, mut buffer: Vec<u8>) {
        buffer.clear();
        let mut buffers = self.buffers.write().await;
        if buffers.len() < self.max_buffers {
            buffers.push(buffer);
        }
    }
}

/// Performance profiler for measuring and optimizing operations
pub struct PerformanceProfiler {
    targets: PerformanceTargets,
    metrics_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    memory_pool: MemoryPool,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            targets: PerformanceTargets::default(),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            memory_pool: MemoryPool::new(64 * 1024, 100), // 64KB buffers, max 100
        }
    }

    /// Record performance metrics
    pub async fn record_metrics(&self, metrics: PerformanceMetrics) -> Result<()> {
        let mut history = self.metrics_history.write().await;

        // Validate against targets
        self.validate_targets(&metrics)?;

        history.push(metrics.clone());

        // Keep only last 1000 metrics for memory efficiency
        let len = history.len();
        if len > 1000 {
            history.drain(0..len - 1000);
        }

        info!("Performance metrics recorded: {:?}", metrics);
        Ok(())
    }

    /// Validate metrics against performance targets
    fn validate_targets(&self, metrics: &PerformanceMetrics) -> Result<()> {
        let mut warnings = Vec::new();

        if metrics.startup_time > self.targets.startup_time {
            warnings.push(format!(
                "Startup time exceeded target: {:?} > {:?}",
                metrics.startup_time, self.targets.startup_time
            ));
        }

        if metrics.memory_usage > self.targets.memory_usage {
            warnings.push(format!(
                "Memory usage exceeded target: {} > {} bytes",
                metrics.memory_usage, self.targets.memory_usage
            ));
        }

        if metrics.consensus_latency > self.targets.consensus_latency {
            warnings.push(format!(
                "Consensus latency exceeded target: {:?} > {:?}",
                metrics.consensus_latency, self.targets.consensus_latency
            ));
        }

        if metrics.file_parse_time > self.targets.file_parse_time {
            warnings.push(format!(
                "File parse time exceeded target: {:?} > {:?}",
                metrics.file_parse_time, self.targets.file_parse_time
            ));
        }

        if metrics.database_latency > self.targets.database_latency {
            warnings.push(format!(
                "Database latency exceeded target: {:?} > {:?}",
                metrics.database_latency, self.targets.database_latency
            ));
        }

        for warning in warnings {
            warn!("{}", warning);
        }

        Ok(())
    }

    /// Get performance statistics
    pub async fn get_statistics(&self) -> Result<PerformanceStatistics> {
        let history = self.metrics_history.read().await;

        if history.is_empty() {
            return Ok(PerformanceStatistics::default());
        }

        let len = history.len() as f64;

        let avg_startup_time = history
            .iter()
            .map(|m| m.startup_time.as_nanos() as f64)
            .sum::<f64>()
            / len;

        let avg_memory_usage = history.iter().map(|m| m.memory_usage as f64).sum::<f64>() / len;

        let avg_consensus_latency = history
            .iter()
            .map(|m| m.consensus_latency.as_nanos() as f64)
            .sum::<f64>()
            / len;

        let avg_file_parse_time = history
            .iter()
            .map(|m| m.file_parse_time.as_nanos() as f64)
            .sum::<f64>()
            / len;

        let avg_database_latency = history
            .iter()
            .map(|m| m.database_latency.as_nanos() as f64)
            .sum::<f64>()
            / len;

        Ok(PerformanceStatistics {
            sample_count: history.len(),
            avg_startup_time: Duration::from_nanos(avg_startup_time as u64),
            avg_memory_usage: avg_memory_usage as u64,
            avg_consensus_latency: Duration::from_nanos(avg_consensus_latency as u64),
            avg_file_parse_time: Duration::from_nanos(avg_file_parse_time as u64),
            avg_database_latency: Duration::from_nanos(avg_database_latency as u64),
            targets_met: self.calculate_targets_met(&history),
        })
    }

    /// Calculate percentage of metrics that met targets
    fn calculate_targets_met(&self, history: &[PerformanceMetrics]) -> f64 {
        if history.is_empty() {
            return 0.0;
        }

        let met_count = history
            .iter()
            .filter(|m| {
                m.startup_time <= self.targets.startup_time
                    && m.memory_usage <= self.targets.memory_usage
                    && m.consensus_latency <= self.targets.consensus_latency
                    && m.file_parse_time <= self.targets.file_parse_time
                    && m.database_latency <= self.targets.database_latency
            })
            .count();

        (met_count as f64 / history.len() as f64) * 100.0
    }

    /// Get memory pool for efficient allocation
    pub fn memory_pool(&self) -> &MemoryPool {
        &self.memory_pool
    }
}

/// Performance statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatistics {
    pub sample_count: usize,
    pub avg_startup_time: Duration,
    pub avg_memory_usage: u64,
    pub avg_consensus_latency: Duration,
    pub avg_file_parse_time: Duration,
    pub avg_database_latency: Duration,
    pub targets_met: f64, // Percentage
}

impl Default for PerformanceStatistics {
    fn default() -> Self {
        Self {
            sample_count: 0,
            avg_startup_time: Duration::ZERO,
            avg_memory_usage: 0,
            avg_consensus_latency: Duration::ZERO,
            avg_file_parse_time: Duration::ZERO,
            avg_database_latency: Duration::ZERO,
            targets_met: 0.0,
        }
    }
}

/// High-performance timer for micro-benchmarking
pub struct PerfTimer {
    start: Instant,
    label: String,
}

impl PerfTimer {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            label: label.into(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn finish(self) -> Duration {
        let elapsed = self.elapsed();
        debug!("PerfTimer [{}]: {:?}", self.label, elapsed);
        elapsed
    }
}

/// Cache-friendly data structure for hot paths
#[derive(Debug)]
pub struct HotPathCache<T> {
    data: Arc<RwLock<lru::LruCache<String, T>>>,
    capacity: usize,
}

impl<T: Clone> HotPathCache<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap(),
            ))),
            capacity,
        }
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        let mut cache = self.data.write().await;
        cache.get(key).cloned()
    }

    pub async fn put(&self, key: String, value: T) {
        let mut cache = self.data.write().await;
        cache.put(key, value);
    }

    pub async fn contains(&self, key: &str) -> bool {
        let cache = self.data.read().await;
        cache.contains(key)
    }

    pub async fn len(&self) -> usize {
        let cache = self.data.read().await;
        cache.len()
    }
}

/// SIMD-optimized operations for high-performance computing
pub mod simd {
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    /// SIMD-optimized string comparison
    #[cfg(target_feature = "avx2")]
    pub unsafe fn fast_string_compare(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let len = a.len();
        let chunks = len / 32;

        for i in 0..chunks {
            let offset = i * 32;
            let a_chunk = _mm256_loadu_si256(a.as_ptr().add(offset) as *const __m256i);
            let b_chunk = _mm256_loadu_si256(b.as_ptr().add(offset) as *const __m256i);
            let cmp = _mm256_cmpeq_epi8(a_chunk, b_chunk);

            if _mm256_movemask_epi8(cmp) != -1 {
                return false;
            }
        }

        // Handle remaining bytes
        let remaining = len % 32;
        if remaining > 0 {
            let offset = chunks * 32;
            for i in 0..remaining {
                if a[offset + i] != b[offset + i] {
                    return false;
                }
            }
        }

        true
    }

    /// Fallback for non-AVX2 systems
    #[cfg(not(target_feature = "avx2"))]
    pub fn fast_string_compare(a: &[u8], b: &[u8]) -> bool {
        a == b
    }
}

/// Global performance profiler instance
static PROFILER: once_cell::sync::Lazy<PerformanceProfiler> =
    once_cell::sync::Lazy::new(|| PerformanceProfiler::new());

/// Get global performance profiler
pub fn global_profiler() -> &'static PerformanceProfiler {
    &PROFILER
}

/// Macro for easy performance timing
#[macro_export]
macro_rules! perf_time {
    ($label:expr, $expr:expr) => {{
        let timer = $crate::core::performance::PerfTimer::new($label);
        let result = $expr;
        timer.finish();
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_memory_pool() {
        let pool = MemoryPool::new(1024, 10);

        let buffer1 = pool.get_buffer().await;
        assert_eq!(buffer1.capacity(), 1024);

        pool.return_buffer(buffer1).await;

        let buffer2 = pool.get_buffer().await;
        assert_eq!(buffer2.len(), 0);
    }

    #[test]
    async fn test_performance_profiler() {
        let profiler = PerformanceProfiler::new();

        let metrics = PerformanceMetrics {
            startup_time: Duration::from_millis(20),
            memory_usage: 15 * 1024 * 1024,
            consensus_latency: Duration::from_millis(250),
            file_parse_time: Duration::from_millis(1),
            database_latency: Duration::from_nanos(500_000),
            timestamp: std::time::SystemTime::now(),
        };

        profiler.record_metrics(metrics).await.unwrap();

        let stats = profiler.get_statistics().await.unwrap();
        assert_eq!(stats.sample_count, 1);
        assert!(stats.targets_met > 99.0);
    }

    #[test]
    async fn test_hot_path_cache() {
        let cache = HotPathCache::new(100);

        cache.put("key1".to_string(), "value1".to_string()).await;

        let value = cache.get("key1").await;
        assert_eq!(value, Some("value1".to_string()));

        assert!(cache.contains("key1").await);
        assert_eq!(cache.len().await, 1);
    }

    #[tokio::test]
    async fn test_simd_string_compare() {
        let a = b"hello world test string";
        let b = b"hello world test string";
        let c = b"hello world different";

        assert!(simd::fast_string_compare(a, b));
        assert!(!simd::fast_string_compare(a, c));
    }
}
