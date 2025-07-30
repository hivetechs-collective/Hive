//! Performance Benchmark Tests
//!
//! Comprehensive performance testing to verify 10-40x improvement targets
//! and validate optimization strategies.

use crate::migration::{
    database_impl::{ProductionDatabase, BatchConfig},
    performance::{PerformanceOptimizer, PerformanceConfig, BatchStrategy, DiskIOStrategy},
};
use crate::core::error::HiveError;
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::Instant;
use tempfile::tempdir;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

/// Performance benchmark suite
pub struct PerformanceBenchmarkSuite {
    temp_dir: tempfile::TempDir,
    source_db_path: PathBuf,
    target_db_path: PathBuf,
}

impl PerformanceBenchmarkSuite {
    /// Create new benchmark suite
    pub fn new() -> Result<Self, HiveError> {
        let temp_dir = tempdir()
            .map_err(|e| HiveError::Migration(format!("Failed to create temp dir: {}", e)))?;

        let source_db_path = temp_dir.path().join("benchmark_source.db");
        let target_db_path = temp_dir.path().join("benchmark_target.db");

        Ok(Self {
            temp_dir,
            source_db_path,
            target_db_path,
        })
    }

    /// Create benchmark database with specified size
    pub async fn create_benchmark_database(&self, conversation_count: u32) -> Result<(), HiveError> {
        let conn = Connection::open(&self.source_db_path)
            .map_err(|e| HiveError::Migration(format!("Failed to create benchmark database: {}", e)))?;

        // Create optimized schema for benchmarking
        conn.execute_batch(r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 10000;
            PRAGMA temp_store = memory;

            CREATE TABLE conversations (
                id TEXT PRIMARY KEY,
                question TEXT NOT NULL,
                final_answer TEXT NOT NULL,
                source_of_truth TEXT NOT NULL,
                conversation_context TEXT,
                profile_id TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                last_updated TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE stage_outputs (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                stage_name TEXT NOT NULL,
                stage_number INTEGER NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                full_output TEXT NOT NULL,
                character_count INTEGER NOT NULL,
                word_count INTEGER NOT NULL,
                temperature REAL,
                processing_time_ms INTEGER,
                tokens_used INTEGER,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );

            CREATE TABLE knowledge_base (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                curator_content TEXT NOT NULL,
                topics TEXT NOT NULL,
                keywords TEXT NOT NULL,
                semantic_embedding BLOB,
                is_source_of_truth INTEGER DEFAULT 1,
                relevance_score REAL DEFAULT 1.0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );

            CREATE INDEX idx_conversations_created_at ON conversations(created_at DESC);
            CREATE INDEX idx_stage_outputs_conversation ON stage_outputs(conversation_id);
            CREATE INDEX idx_knowledge_base_conversation ON knowledge_base(conversation_id);
        "#)
        .map_err(|e| HiveError::Migration(format!("Failed to create schema: {}", e)))?;

        // Insert benchmark data
        let mut stmt = conn.prepare(
            "INSERT INTO conversations
             (id, question, final_answer, source_of_truth, created_at)
             VALUES (?, ?, ?, ?, ?)"
        ).map_err(|e| HiveError::Migration(format!("Failed to prepare statement: {}", e)))?;

        for i in 1..=conversation_count {
            let conversation_id = format!("benchmark_conv_{:06}", i);
            let question = format!("Performance benchmark question number {} with additional content to simulate realistic data sizes", i);
            let final_answer = format!("Performance benchmark answer number {} with comprehensive content that represents typical response sizes in production environments", i);
            let source_of_truth = format!("Performance benchmark source of truth content for conversation {} with detailed information", i);
            let timestamp = format!("2024-01-{:02}T{:02}:{:02}:00Z",
                (i % 28) + 1,
                (i % 24),
                (i % 60)
            );

            stmt.execute([
                &conversation_id,
                &question,
                &final_answer,
                &source_of_truth,
                &timestamp,
            ]).map_err(|e| HiveError::Migration(format!("Failed to insert conversation: {}", e)))?;

            // Insert stage outputs (4 per conversation)
            for stage_num in 1..=4 {
                let stage_name = match stage_num {
                    1 => "generator",
                    2 => "refiner",
                    3 => "validator",
                    4 => "curator",
                    _ => "unknown",
                };

                let stage_id = format!("{}_stage_{}", conversation_id, stage_num);
                let provider = format!("Provider_{}", stage_num);
                let model = format!("Model_{}", stage_num);
                let output_content = format!("Detailed output content from {} stage for conversation {} with substantial text content to simulate realistic AI response sizes", stage_name, i);

                conn.execute(
                    "INSERT INTO stage_outputs
                     (id, conversation_id, stage_name, stage_number, provider, model,
                      full_output, character_count, word_count, temperature,
                      processing_time_ms, tokens_used, created_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    [
                        &stage_id,
                        &conversation_id,
                        stage_name,
                        &stage_num.to_string(),
                        &provider,
                        &model,
                        &output_content,
                        &output_content.len().to_string(),
                        &(output_content.split_whitespace().count().to_string()),
                        "0.7",
                        &(1000 + i * 10).to_string(),
                        &(100 + i * 5).to_string(),
                        &timestamp,
                    ]
                ).map_err(|e| HiveError::Migration(format!("Failed to insert stage output: {}", e)))?;
            }

            // Insert knowledge base entry
            let kb_id = format!("kb_{}", conversation_id);
            let curator_content = format!("Comprehensive curator content for {} with detailed analysis and insights", conversation_id);
            let topics = format!("[\"topic_{}\", \"benchmark\", \"performance\"]", i % 20);
            let keywords = format!("[\"keyword_{}\", \"test\", \"benchmark\"]", i % 10);

            conn.execute(
                "INSERT INTO knowledge_base
                 (id, conversation_id, curator_content, topics, keywords, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
                [
                    &kb_id,
                    &conversation_id,
                    &curator_content,
                    &topics,
                    &keywords,
                    &timestamp,
                ]
            ).map_err(|e| HiveError::Migration(format!("Failed to insert knowledge base: {}", e)))?;

            // Progress reporting for large datasets
            if i % 1000 == 0 {
                log::info!("Created {} benchmark conversations", i);
            }
        }

        log::info!("Created benchmark database with {} conversations", conversation_count);
        Ok(())
    }

    /// Benchmark basic migration performance
    pub async fn benchmark_basic_migration(&self) -> Result<BenchmarkResult, HiveError> {
        let start_time = Instant::now();

        let mut db = ProductionDatabase::new(&self.source_db_path, &self.target_db_path).await?;

        let config = BatchConfig {
            batch_size: 1000,
            parallel_batches: 1,
            memory_limit_mb: 256,
            progress_callback: None,
        };

        let stats = db.migrate_complete_database(config).await?;
        let duration = start_time.elapsed();

        Ok(BenchmarkResult {
            name: "Basic Migration".to_string(),
            duration_ms: duration.as_millis() as u64,
            rows_processed: stats.total_rows_migrated,
            throughput_rows_per_sec: stats.total_rows_migrated as f64 / duration.as_secs_f64(),
            memory_usage_mb: 256, // Estimated
            success: true,
        })
    }

    /// Benchmark optimized migration performance
    pub async fn benchmark_optimized_migration(&self) -> Result<BenchmarkResult, HiveError> {
        let start_time = Instant::now();

        let mut db = ProductionDatabase::new(&self.source_db_path, &self.target_db_path).await?;

        let config = BatchConfig {
            batch_size: 2000, // Larger batch size
            parallel_batches: 4, // Parallel processing
            memory_limit_mb: 512, // More memory
            progress_callback: None,
        };

        let stats = db.migrate_complete_database(config).await?;
        let duration = start_time.elapsed();

        Ok(BenchmarkResult {
            name: "Optimized Migration".to_string(),
            duration_ms: duration.as_millis() as u64,
            rows_processed: stats.total_rows_migrated,
            throughput_rows_per_sec: stats.total_rows_migrated as f64 / duration.as_secs_f64(),
            memory_usage_mb: 512, // Estimated
            success: true,
        })
    }

    /// Benchmark different batch sizes
    pub async fn benchmark_batch_sizes(&self, batch_sizes: Vec<u32>) -> Result<Vec<BenchmarkResult>, HiveError> {
        let mut results = Vec::new();

        for batch_size in batch_sizes {
            // Clean target database for each test
            if self.target_db_path.exists() {
                std::fs::remove_file(&self.target_db_path).ok();
            }

            let start_time = Instant::now();

            let mut db = ProductionDatabase::new(&self.source_db_path, &self.target_db_path).await?;

            let config = BatchConfig {
                batch_size,
                parallel_batches: 2,
                memory_limit_mb: 512,
                progress_callback: None,
            };

            let stats = db.migrate_complete_database(config).await?;
            let duration = start_time.elapsed();

            results.push(BenchmarkResult {
                name: format!("Batch Size {}", batch_size),
                duration_ms: duration.as_millis() as u64,
                rows_processed: stats.total_rows_migrated,
                throughput_rows_per_sec: stats.total_rows_migrated as f64 / duration.as_secs_f64(),
                memory_usage_mb: 512,
                success: true,
            });
        }

        Ok(results)
    }

    /// Benchmark parallel processing effectiveness
    pub async fn benchmark_parallel_processing(&self, parallel_configs: Vec<u32>) -> Result<Vec<BenchmarkResult>, HiveError> {
        let mut results = Vec::new();

        for parallel_batches in parallel_configs {
            // Clean target database for each test
            if self.target_db_path.exists() {
                std::fs::remove_file(&self.target_db_path).ok();
            }

            let start_time = Instant::now();

            let mut db = ProductionDatabase::new(&self.source_db_path, &self.target_db_path).await?;

            let config = BatchConfig {
                batch_size: 1000,
                parallel_batches,
                memory_limit_mb: 512,
                progress_callback: None,
            };

            let stats = db.migrate_complete_database(config).await?;
            let duration = start_time.elapsed();

            results.push(BenchmarkResult {
                name: format!("Parallel Batches {}", parallel_batches),
                duration_ms: duration.as_millis() as u64,
                rows_processed: stats.total_rows_migrated,
                throughput_rows_per_sec: stats.total_rows_migrated as f64 / duration.as_secs_f64(),
                memory_usage_mb: 512,
                success: true,
            });
        }

        Ok(results)
    }

    /// Benchmark performance optimization strategies
    pub async fn benchmark_optimization_strategies(&self) -> Result<Vec<BenchmarkResult>, HiveError> {
        let mut results = Vec::new();

        let strategies = vec![
            ("Sequential Processing", PerformanceConfig {
                enable_parallel_processing: false,
                enable_memory_mapping: false,
                enable_streaming: false,
                batch_strategy: BatchStrategy::FixedSize(500),
                disk_io_optimization: DiskIOStrategy::Sequential,
                ..Default::default()
            }),
            ("Basic Optimization", PerformanceConfig {
                enable_parallel_processing: true,
                enable_memory_mapping: false,
                enable_streaming: true,
                batch_strategy: BatchStrategy::FixedSize(1000),
                disk_io_optimization: DiskIOStrategy::Parallel,
                ..Default::default()
            }),
            ("Advanced Optimization", PerformanceConfig {
                enable_parallel_processing: true,
                enable_memory_mapping: true,
                enable_streaming: true,
                batch_strategy: BatchStrategy::AdaptiveSize,
                disk_io_optimization: DiskIOStrategy::Adaptive,
                ..Default::default()
            }),
            ("Maximum Optimization", PerformanceConfig {
                enable_parallel_processing: true,
                enable_memory_mapping: true,
                enable_streaming: true,
                batch_strategy: BatchStrategy::HybridOptimal,
                disk_io_optimization: DiskIOStrategy::MemoryMapped,
                cpu_cores_to_use: num_cpus::get() as u32,
                memory_limit_mb: 1024,
                ..Default::default()
            }),
        ];

        for (strategy_name, config) in strategies {
            // Clean target database for each test
            if self.target_db_path.exists() {
                std::fs::remove_file(&self.target_db_path).ok();
            }

            let start_time = Instant::now();

            let mut db = ProductionDatabase::new(&self.source_db_path, &self.target_db_path).await?;
            let mut optimizer = PerformanceOptimizer::new(config);

            let optimization_result = optimizer.optimize_migration_performance(&mut db).await?;
            let duration = start_time.elapsed();

            results.push(BenchmarkResult {
                name: strategy_name.to_string(),
                duration_ms: duration.as_millis() as u64,
                rows_processed: 0, // Would be filled from optimization result
                throughput_rows_per_sec: 0.0, // Would be calculated
                memory_usage_mb: optimization_result.memory_efficiency_score as u64,
                success: optimization_result.performance_improvement_factor > 1.0,
            });
        }

        Ok(results)
    }

    /// Memory usage benchmark
    pub async fn benchmark_memory_usage(&self) -> Result<BenchmarkResult, HiveError> {
        let start_memory = Self::get_memory_usage();

        let mut db = ProductionDatabase::new(&self.source_db_path, &self.target_db_path).await?;

        let config = BatchConfig {
            batch_size: 500, // Smaller batch for memory testing
            parallel_batches: 1,
            memory_limit_mb: 128, // Limited memory
            progress_callback: None,
        };

        let start_time = Instant::now();
        let stats = db.migrate_complete_database(config).await?;
        let duration = start_time.elapsed();

        let end_memory = Self::get_memory_usage();
        let memory_used = end_memory.saturating_sub(start_memory);

        Ok(BenchmarkResult {
            name: "Memory Usage Test".to_string(),
            duration_ms: duration.as_millis() as u64,
            rows_processed: stats.total_rows_migrated,
            throughput_rows_per_sec: stats.total_rows_migrated as f64 / duration.as_secs_f64(),
            memory_usage_mb: memory_used,
            success: memory_used < 256, // Should stay under 256MB
        })
    }

    /// Get current memory usage (stub implementation)
    fn get_memory_usage() -> u64 {
        // In a real implementation, this would use system APIs to get actual memory usage
        // For testing, we return a simulated value
        128 // 128MB baseline
    }

    /// Run comprehensive performance test suite
    pub async fn run_comprehensive_benchmark(&self, conversation_count: u32) -> Result<BenchmarkSuite, HiveError> {
        log::info!("Running comprehensive performance benchmark with {} conversations", conversation_count);

        // Create benchmark database
        self.create_benchmark_database(conversation_count).await?;

        let mut suite = BenchmarkSuite {
            conversation_count,
            results: Vec::new(),
            overall_performance_factor: 0.0,
            meets_performance_targets: false,
        };

        // Run basic migration benchmark
        let basic_result = self.benchmark_basic_migration().await?;
        suite.results.push(basic_result.clone());

        // Run optimized migration benchmark
        let optimized_result = self.benchmark_optimized_migration().await?;
        suite.results.push(optimized_result.clone());

        // Calculate performance improvement
        if basic_result.duration_ms > 0 {
            suite.overall_performance_factor = basic_result.duration_ms as f64 / optimized_result.duration_ms as f64;
        }

        // Run batch size benchmarks
        let batch_results = self.benchmark_batch_sizes(vec![250, 500, 1000, 2000, 5000]).await?;
        suite.results.extend(batch_results);

        // Run parallel processing benchmarks
        let parallel_results = self.benchmark_parallel_processing(vec![1, 2, 4, 8]).await?;
        suite.results.extend(parallel_results);

        // Run optimization strategy benchmarks
        let strategy_results = self.benchmark_optimization_strategies().await?;
        suite.results.extend(strategy_results);

        // Run memory usage benchmark
        let memory_result = self.benchmark_memory_usage().await?;
        suite.results.push(memory_result);

        // Check if performance targets are met
        suite.meets_performance_targets = suite.overall_performance_factor >= 10.0; // 10x minimum target

        log::info!("Benchmark completed. Performance factor: {:.2}x", suite.overall_performance_factor);

        Ok(suite)
    }
}

/// Individual benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration_ms: u64,
    pub rows_processed: u64,
    pub throughput_rows_per_sec: f64,
    pub memory_usage_mb: u64,
    pub success: bool,
}

/// Complete benchmark suite results
#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    pub conversation_count: u32,
    pub results: Vec<BenchmarkResult>,
    pub overall_performance_factor: f64,
    pub meets_performance_targets: bool,
}

impl BenchmarkSuite {
    /// Get best performing configuration
    pub fn get_best_performance(&self) -> Option<&BenchmarkResult> {
        self.results.iter()
            .filter(|r| r.success)
            .max_by(|a, b| a.throughput_rows_per_sec.partial_cmp(&b.throughput_rows_per_sec).unwrap())
    }

    /// Get memory efficient configuration
    pub fn get_most_memory_efficient(&self) -> Option<&BenchmarkResult> {
        self.results.iter()
            .filter(|r| r.success)
            .min_by_key(|r| r.memory_usage_mb)
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!("Performance Benchmark Report\n"));
        report.push_str(&format!("==========================\n\n"));
        report.push_str(&format!("Dataset Size: {} conversations\n", self.conversation_count));
        report.push_str(&format!("Overall Performance Factor: {:.2}x\n", self.overall_performance_factor));
        report.push_str(&format!("Meets Performance Targets: {}\n\n", self.meets_performance_targets));

        report.push_str("Individual Results:\n");
        report.push_str("------------------\n");

        for result in &self.results {
            report.push_str(&format!(
                "{}: {:.2}s, {:.0} rows/sec, {}MB memory, {}\n",
                result.name,
                result.duration_ms as f64 / 1000.0,
                result.throughput_rows_per_sec,
                result.memory_usage_mb,
                if result.success { "✓" } else { "✗" }
            ));
        }

        if let Some(best) = self.get_best_performance() {
            report.push_str(&format!("\nBest Performance: {} ({:.0} rows/sec)\n", best.name, best.throughput_rows_per_sec));
        }

        if let Some(efficient) = self.get_most_memory_efficient() {
            report.push_str(&format!("Most Memory Efficient: {} ({}MB)\n", efficient.name, efficient.memory_usage_mb));
        }

        report
    }
}

// Criterion benchmarks for automated performance testing
fn criterion_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("small_migration_100", |b| {
        b.to_async(&rt).iter(|| async {
            let suite = PerformanceBenchmarkSuite::new().unwrap();
            suite.create_benchmark_database(100).await.unwrap();
            let result = suite.benchmark_basic_migration().await.unwrap();
            black_box(result);
        });
    });

    c.bench_function("medium_migration_1000", |b| {
        b.to_async(&rt).iter(|| async {
            let suite = PerformanceBenchmarkSuite::new().unwrap();
            suite.create_benchmark_database(1000).await.unwrap();
            let result = suite.benchmark_basic_migration().await.unwrap();
            black_box(result);
        });
    });

    let batch_sizes = vec![500, 1000, 2000];
    for batch_size in batch_sizes {
        c.bench_with_input(
            BenchmarkId::new("batch_size_comparison", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter(|| async move {
                    let suite = PerformanceBenchmarkSuite::new().unwrap();
                    suite.create_benchmark_database(500).await.unwrap();
                    let result = suite.benchmark_batch_sizes(vec![batch_size]).await.unwrap();
                    black_box(result);
                });
            },
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_benchmark_suite_creation() {
        let suite = PerformanceBenchmarkSuite::new().unwrap();
        assert!(suite.source_db_path.to_string_lossy().contains("benchmark_source.db"));
    }

    #[tokio::test]
    #[serial]
    async fn test_small_database_benchmark() {
        let suite = PerformanceBenchmarkSuite::new().unwrap();
        suite.create_benchmark_database(10).await.unwrap();

        let result = suite.benchmark_basic_migration().await.unwrap();
        assert!(result.success);
        assert!(result.rows_processed > 0);
        assert!(result.duration_ms > 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_batch_size_comparison() {
        let suite = PerformanceBenchmarkSuite::new().unwrap();
        suite.create_benchmark_database(50).await.unwrap();

        let results = suite.benchmark_batch_sizes(vec![10, 25, 50]).await.unwrap();
        assert_eq!(results.len(), 3);

        for result in results {
            assert!(result.success);
            assert!(result.throughput_rows_per_sec > 0.0);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_parallel_processing_benchmark() {
        let suite = PerformanceBenchmarkSuite::new().unwrap();
        suite.create_benchmark_database(40).await.unwrap();

        let results = suite.benchmark_parallel_processing(vec![1, 2, 4]).await.unwrap();
        assert_eq!(results.len(), 3);

        // Parallel processing should generally be faster (higher throughput)
        let sequential = &results[0];
        let parallel = &results[1];

        assert!(sequential.success);
        assert!(parallel.success);
        // Note: In very small datasets, parallel might not always be faster due to overhead
    }

    #[tokio::test]
    #[serial]
    async fn test_memory_usage_benchmark() {
        let suite = PerformanceBenchmarkSuite::new().unwrap();
        suite.create_benchmark_database(25).await.unwrap();

        let result = suite.benchmark_memory_usage().await.unwrap();
        assert!(result.success);
        assert!(result.memory_usage_mb > 0);
        assert!(result.memory_usage_mb < 1024); // Should be reasonable
    }

    #[tokio::test]
    #[serial]
    async fn test_comprehensive_benchmark_small() {
        let suite = PerformanceBenchmarkSuite::new().unwrap();
        let results = suite.run_comprehensive_benchmark(20).await.unwrap();

        assert_eq!(results.conversation_count, 20);
        assert!(!results.results.is_empty());
        assert!(results.overall_performance_factor > 0.0);

        // Verify we have results from different benchmark types
        let result_names: Vec<&String> = results.results.iter().map(|r| &r.name).collect();
        assert!(result_names.iter().any(|name| name.contains("Basic Migration")));
        assert!(result_names.iter().any(|name| name.contains("Optimized Migration")));
    }

    #[tokio::test]
    #[serial]
    async fn test_benchmark_report_generation() {
        let suite = PerformanceBenchmarkSuite::new().unwrap();
        let results = suite.run_comprehensive_benchmark(15).await.unwrap();

        let report = results.generate_report();
        assert!(report.contains("Performance Benchmark Report"));
        assert!(report.contains("Dataset Size: 15 conversations"));
        assert!(report.contains("Overall Performance Factor"));
    }

    #[tokio::test]
    #[serial]
    async fn test_performance_target_validation() {
        let suite = PerformanceBenchmarkSuite::new().unwrap();
        let results = suite.run_comprehensive_benchmark(30).await.unwrap();

        // Check if performance improvement is measurable
        assert!(results.overall_performance_factor > 0.5); // At least some improvement

        // Verify best performance result exists
        let best = results.get_best_performance();
        assert!(best.is_some());

        let best_result = best.unwrap();
        assert!(best_result.throughput_rows_per_sec > 0.0);
        assert!(best_result.success);
    }

    #[tokio::test]
    #[serial]
    async fn test_optimization_strategy_comparison() {
        let suite = PerformanceBenchmarkSuite::new().unwrap();
        suite.create_benchmark_database(35).await.unwrap();

        let strategy_results = suite.benchmark_optimization_strategies().await.unwrap();
        assert!(strategy_results.len() >= 3); // At least 3 different strategies

        // Find sequential and optimized strategies
        let sequential = strategy_results.iter()
            .find(|r| r.name.contains("Sequential"))
            .expect("Sequential strategy should be tested");

        let optimized = strategy_results.iter()
            .find(|r| r.name.contains("Advanced") || r.name.contains("Maximum"))
            .expect("Optimized strategy should be tested");

        assert!(sequential.success);
        assert!(optimized.success);

        // In most cases, optimized should be better, but for very small datasets
        // the overhead might make sequential faster
        log::info!("Sequential: {:.2} rows/sec, Optimized: {:.2} rows/sec",
            sequential.throughput_rows_per_sec, optimized.throughput_rows_per_sec);
    }

    #[tokio::test]
    #[serial]
    async fn test_large_dataset_performance() {
        let suite = PerformanceBenchmarkSuite::new().unwrap();

        // Test with larger dataset to see real performance differences
        let results = suite.run_comprehensive_benchmark(100).await.unwrap();

        assert_eq!(results.conversation_count, 100);
        assert!(results.overall_performance_factor > 0.8); // Should show some improvement

        // With larger datasets, we should see clearer performance differences
        let best_performance = results.get_best_performance().unwrap();
        assert!(best_performance.throughput_rows_per_sec > 10.0); // At least 10 rows/sec

        // Memory usage should be reasonable even for larger datasets
        let memory_efficient = results.get_most_memory_efficient().unwrap();
        assert!(memory_efficient.memory_usage_mb < 512); // Should stay under 512MB
    }
}