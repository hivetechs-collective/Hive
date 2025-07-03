//! Performance monitoring for AST parsing and analysis
//!
//! This module tracks parsing performance to ensure we meet
//! the <2s for 1000 files and <5ms incremental update targets.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};

/// Performance monitor for tracking parse metrics
pub struct PerformanceMonitor {
    /// Total files parsed
    files_parsed: AtomicUsize,
    /// Total parse time in microseconds
    total_parse_time_us: AtomicU64,
    /// Fastest parse time in microseconds
    min_parse_time_us: AtomicU64,
    /// Slowest parse time in microseconds
    max_parse_time_us: AtomicU64,
    /// Incremental parses performed
    incremental_parses: AtomicUsize,
    /// Total incremental parse time in microseconds
    total_incremental_time_us: AtomicU64,
    /// Cache hits
    cache_hits: AtomicUsize,
    /// Cache misses
    cache_misses: AtomicUsize,
    /// Parse errors
    parse_errors: AtomicUsize,
    /// Start time for tracking uptime
    start_time: Instant,
}

/// Performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseMetrics {
    /// Total files parsed
    pub files_parsed: usize,
    /// Average parse time in milliseconds
    pub avg_parse_time_ms: f64,
    /// Minimum parse time in milliseconds
    pub min_parse_time_ms: f64,
    /// Maximum parse time in milliseconds
    pub max_parse_time_ms: f64,
    /// Incremental parses performed
    pub incremental_parses: usize,
    /// Average incremental parse time in milliseconds
    pub avg_incremental_time_ms: f64,
    /// Cache hit rate percentage
    pub cache_hit_rate: f64,
    /// Parse error rate percentage
    pub error_rate: f64,
    /// Files parsed per second
    pub throughput: f64,
    /// Uptime in seconds
    pub uptime_seconds: f64,
    /// Performance status
    pub status: PerformanceStatus,
}

/// Performance status assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceStatus {
    /// All targets met
    Optimal,
    /// Some targets missed but acceptable
    Acceptable,
    /// Performance below targets
    Degraded,
    /// Critical performance issues
    Critical,
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    name: String,
    start: Instant,
    monitor: Arc<PerformanceMonitor>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            files_parsed: AtomicUsize::new(0),
            total_parse_time_us: AtomicU64::new(0),
            min_parse_time_us: AtomicU64::new(u64::MAX),
            max_parse_time_us: AtomicU64::new(0),
            incremental_parses: AtomicUsize::new(0),
            total_incremental_time_us: AtomicU64::new(0),
            cache_hits: AtomicUsize::new(0),
            cache_misses: AtomicUsize::new(0),
            parse_errors: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }
    
    /// Start timing an operation
    pub fn start_operation(self: &Arc<Self>, name: &str) -> OperationTimer {
        OperationTimer {
            name: name.to_string(),
            start: Instant::now(),
            monitor: Arc::clone(self),
        }
    }
    
    /// Record a parse operation
    pub fn record_parse(&self, duration: Duration, is_incremental: bool) {
        let duration_us = duration.as_micros() as u64;
        
        if is_incremental {
            self.incremental_parses.fetch_add(1, Ordering::Relaxed);
            self.total_incremental_time_us.fetch_add(duration_us, Ordering::Relaxed);
            
            // Check if incremental parse exceeded 5ms target
            if duration.as_millis() > 5 {
                warn!(
                    "Incremental parse took {}ms, exceeding 5ms target",
                    duration.as_millis()
                );
            }
        } else {
            self.files_parsed.fetch_add(1, Ordering::Relaxed);
            self.total_parse_time_us.fetch_add(duration_us, Ordering::Relaxed);
            
            // Update min/max
            self.min_parse_time_us.fetch_min(duration_us, Ordering::Relaxed);
            self.max_parse_time_us.fetch_max(duration_us, Ordering::Relaxed);
        }
        
        debug!(
            "Parse completed in {}ms (incremental: {})",
            duration.as_millis(),
            is_incremental
        );
    }
    
    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a parse error
    pub fn record_parse_error(&self) {
        self.parse_errors.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> ParseMetrics {
        let files_parsed = self.files_parsed.load(Ordering::Relaxed);
        let total_parse_time_us = self.total_parse_time_us.load(Ordering::Relaxed);
        let min_parse_time_us = self.min_parse_time_us.load(Ordering::Relaxed);
        let max_parse_time_us = self.max_parse_time_us.load(Ordering::Relaxed);
        let incremental_parses = self.incremental_parses.load(Ordering::Relaxed);
        let total_incremental_time_us = self.total_incremental_time_us.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let parse_errors = self.parse_errors.load(Ordering::Relaxed);
        
        let uptime = self.start_time.elapsed().as_secs_f64();
        
        // Calculate averages
        let avg_parse_time_ms = if files_parsed > 0 {
            (total_parse_time_us as f64 / files_parsed as f64) / 1000.0
        } else {
            0.0
        };
        
        let avg_incremental_time_ms = if incremental_parses > 0 {
            (total_incremental_time_us as f64 / incremental_parses as f64) / 1000.0
        } else {
            0.0
        };
        
        let min_parse_time_ms = if min_parse_time_us == u64::MAX {
            0.0
        } else {
            min_parse_time_us as f64 / 1000.0
        };
        
        let max_parse_time_ms = max_parse_time_us as f64 / 1000.0;
        
        // Calculate rates
        let total_cache_accesses = cache_hits + cache_misses;
        let cache_hit_rate = if total_cache_accesses > 0 {
            (cache_hits as f64 / total_cache_accesses as f64) * 100.0
        } else {
            0.0
        };
        
        let total_operations = files_parsed + parse_errors;
        let error_rate = if total_operations > 0 {
            (parse_errors as f64 / total_operations as f64) * 100.0
        } else {
            0.0
        };
        
        let throughput = if uptime > 0.0 {
            files_parsed as f64 / uptime
        } else {
            0.0
        };
        
        // Assess performance status
        let status = self.assess_performance(
            avg_parse_time_ms,
            avg_incremental_time_ms,
            throughput,
            error_rate,
        );
        
        ParseMetrics {
            files_parsed,
            avg_parse_time_ms,
            min_parse_time_ms,
            max_parse_time_ms,
            incremental_parses,
            avg_incremental_time_ms,
            cache_hit_rate,
            error_rate,
            throughput,
            uptime_seconds: uptime,
            status,
        }
    }
    
    /// Assess overall performance status
    fn assess_performance(
        &self,
        avg_parse_time_ms: f64,
        avg_incremental_time_ms: f64,
        throughput: f64,
        error_rate: f64,
    ) -> PerformanceStatus {
        // Target: 1000 files in <2s = 2ms per file average
        // Target: Incremental updates in <5ms
        // Target: >500 files/second throughput
        // Target: <5% error rate
        
        let mut score = 0;
        
        // Check parse time (2ms target)
        if avg_parse_time_ms <= 2.0 {
            score += 2;
        } else if avg_parse_time_ms <= 5.0 {
            score += 1;
        }
        
        // Check incremental parse time (5ms target)
        if avg_incremental_time_ms <= 5.0 {
            score += 2;
        } else if avg_incremental_time_ms <= 10.0 {
            score += 1;
        }
        
        // Check throughput (500 files/sec target)
        if throughput >= 500.0 {
            score += 2;
        } else if throughput >= 250.0 {
            score += 1;
        }
        
        // Check error rate (<5% target)
        if error_rate < 5.0 {
            score += 2;
        } else if error_rate < 10.0 {
            score += 1;
        }
        
        match score {
            7..=8 => PerformanceStatus::Optimal,
            5..=6 => PerformanceStatus::Acceptable,
            3..=4 => PerformanceStatus::Degraded,
            _ => PerformanceStatus::Critical,
        }
    }
    
    /// Reset all metrics
    pub fn reset(&self) {
        self.files_parsed.store(0, Ordering::Relaxed);
        self.total_parse_time_us.store(0, Ordering::Relaxed);
        self.min_parse_time_us.store(u64::MAX, Ordering::Relaxed);
        self.max_parse_time_us.store(0, Ordering::Relaxed);
        self.incremental_parses.store(0, Ordering::Relaxed);
        self.total_incremental_time_us.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.parse_errors.store(0, Ordering::Relaxed);
    }
    
    /// Log current metrics
    pub fn log_metrics(&self) {
        let metrics = self.get_metrics();
        
        info!(
            files = metrics.files_parsed,
            avg_ms = format!("{:.2}", metrics.avg_parse_time_ms),
            incremental_ms = format!("{:.2}", metrics.avg_incremental_time_ms),
            cache_hit_rate = format!("{:.1}%", metrics.cache_hit_rate),
            throughput = format!("{:.1} files/s", metrics.throughput),
            status = ?metrics.status,
            "Parse performance metrics"
        );
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let _is_incremental = self.name.contains("incremental");
        
        // Record based on operation type
        match self.name.as_str() {
            "analyze_file" => self.monitor.record_parse(duration, false),
            "analyze_incremental" => self.monitor.record_parse(duration, true),
            _ => {
                // Just log other operations
                debug!(
                    operation = %self.name,
                    duration_ms = duration.as_millis(),
                    "Operation completed"
                );
            }
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark runner for performance testing
pub struct BenchmarkRunner;

impl BenchmarkRunner {
    /// Run performance benchmark
    pub async fn run_benchmark() -> Result<BenchmarkResult, anyhow::Error> {
        use crate::analysis::AnalysisEngine;
        use std::path::Path;
        use tokio::fs;
        
        info!("Starting performance benchmark...");
        
        let engine = AnalysisEngine::new();
        let start = Instant::now();
        
        // Create test files
        let temp_dir = tempfile::tempdir()?;
        let mut files = Vec::new();
        
        // Generate 1000 test files of varying complexity
        for i in 0..1000 {
            let complexity = match i % 4 {
                0 => "simple",
                1 => "medium",
                2 => "complex",
                _ => "large",
            };
            
            let content = generate_test_file(complexity, i);
            let filename = format!("test_{}.rs", i);
            let path = temp_dir.path().join(&filename);
            
            fs::write(&path, &content).await?;
            files.push((path, content));
        }
        
        // Benchmark full parsing
        let parse_start = Instant::now();
        for (path, content) in &files {
            engine.analyze_file(path, content).await?;
        }
        let parse_duration = parse_start.elapsed();
        
        // Benchmark incremental parsing
        let mut incremental_times = Vec::new();
        for (path, content) in files.iter().take(100) {
            let mut modified = content.clone();
            modified.push_str("\n// Modified");
            
            let edit = super::incremental::EditDetector::detect_edit(content, &modified).unwrap();
            
            let inc_start = Instant::now();
            engine.analyze_incremental(path, content, &modified, &edit).await?;
            incremental_times.push(inc_start.elapsed());
        }
        
        let total_duration = start.elapsed();
        let metrics = engine.get_metrics();
        
        Ok(BenchmarkResult {
            total_files: 1000,
            total_time: total_duration,
            parse_time: parse_duration,
            avg_incremental_time: Duration::from_secs_f64(
                incremental_times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / incremental_times.len() as f64
            ),
            metrics,
            passed: parse_duration.as_secs() < 2 && 
                    incremental_times.iter().all(|d| d.as_millis() < 5),
        })
    }
}

/// Benchmark result
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub total_files: usize,
    pub total_time: Duration,
    pub parse_time: Duration,
    pub avg_incremental_time: Duration,
    pub metrics: ParseMetrics,
    pub passed: bool,
}

/// Generate test file content
fn generate_test_file(complexity: &str, index: usize) -> String {
    match complexity {
        "simple" => format!(
            r#"// Simple file {}
fn main() {{
    println!("Hello, world!");
}}
"#,
            index
        ),
        "medium" => format!(
            r#"// Medium complexity file {}
use std::collections::HashMap;

struct Data {{
    id: u64,
    name: String,
    values: Vec<i32>,
}}

impl Data {{
    fn new(id: u64, name: String) -> Self {{
        Self {{ id, name, values: Vec::new() }}
    }}
    
    fn add_value(&mut self, value: i32) {{
        self.values.push(value);
    }}
}}

fn process_data(data: &[Data]) -> HashMap<u64, String> {{
    data.iter()
        .map(|d| (d.id, d.name.clone()))
        .collect()
}}
"#,
            index
        ),
        "complex" => format!(
            r#"// Complex file {}
use std::{{sync::Arc, collections::{{HashMap, BTreeMap}}}};
use tokio::sync::{{Mutex, RwLock}};

#[derive(Debug, Clone)]
pub struct ComplexSystem<T> where T: Send + Sync + 'static {{
    data: Arc<RwLock<HashMap<String, T>>>,
    cache: Arc<Mutex<BTreeMap<u64, String>>>,
    config: SystemConfig,
}}

#[derive(Debug, Clone)]
struct SystemConfig {{
    max_size: usize,
    timeout: std::time::Duration,
    retry_count: u32,
}}

impl<T> ComplexSystem<T> where T: Send + Sync + Clone + 'static {{
    pub async fn new(config: SystemConfig) -> Self {{
        Self {{
            data: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(Mutex::new(BTreeMap::new())),
            config,
        }}
    }}
    
    pub async fn insert(&self, key: String, value: T) -> Result<(), String> {{
        let mut data = self.data.write().await;
        if data.len() >= self.config.max_size {{
            return Err("System at capacity".to_string());
        }}
        data.insert(key, value);
        Ok(())
    }}
    
    pub async fn get(&self, key: &str) -> Option<T> {{
        let data = self.data.read().await;
        data.get(key).cloned()
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[tokio::test]
    async fn test_complex_system() {{
        let config = SystemConfig {{
            max_size: 100,
            timeout: std::time::Duration::from_secs(30),
            retry_count: 3,
        }};
        
        let system = ComplexSystem::<String>::new(config).await;
        system.insert("key".to_string(), "value".to_string()).await.unwrap();
        assert_eq!(system.get("key").await, Some("value".to_string()));
    }}
}}
"#,
            index
        ),
        _ => {
            // Large file with many functions
            let mut content = format!("// Large file {}\n\n", index);
            for i in 0..50 {
                content.push_str(&format!(
                    r#"
fn function_{}() -> Result<(), Box<dyn std::error::Error>> {{
    let data = vec![1, 2, 3, 4, 5];
    let sum: i32 = data.iter().sum();
    println!("Sum: {{}}", sum);
    Ok(())
}}
"#,
                    i
                ));
            }
            content
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();
        
        // Record some operations
        monitor.record_parse(Duration::from_millis(2), false);
        monitor.record_parse(Duration::from_millis(3), false);
        monitor.record_parse(Duration::from_millis(4), true);
        monitor.record_cache_hit();
        monitor.record_cache_miss();
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.files_parsed, 2);
        assert_eq!(metrics.incremental_parses, 1);
        assert_eq!(metrics.cache_hit_rate, 50.0);
    }
    
    #[test]
    fn test_performance_assessment() {
        let monitor = PerformanceMonitor::new();
        
        // Simulate good performance
        for _ in 0..100 {
            monitor.record_parse(Duration::from_millis(1), false);
        }
        for _ in 0..50 {
            monitor.record_parse(Duration::from_millis(3), true);
        }
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.status, PerformanceStatus::Optimal);
    }
}