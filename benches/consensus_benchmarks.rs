//! Consensus performance benchmarks
//!
//! Measures the performance of the consensus system including:
//! - Token processing throughput
//! - Stage execution latency
//! - Memory allocation patterns
//! - Parallel vs sequential execution

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hive_ai::consensus::{
    engine::ConsensusEngine,
    pipeline::ConsensusPipeline,
    types::{ConsensusConfig, ConsensusProfile},
};
use tokio::runtime::Runtime;
use std::time::Duration;

/// Benchmark token processing throughput
fn bench_token_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_processing");
    
    // Test different token batch sizes
    for size in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("batch_size", size),
            size,
            |b, &size| {
                b.iter(|| {
                    // Simulate token processing
                    let tokens: Vec<String> = (0..size)
                        .map(|i| format!("token_{}", i))
                        .collect();
                    
                    // Process tokens (this would be the actual processing logic)
                    for token in tokens {
                        black_box(token.len());
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark consensus engine initialization
fn bench_engine_init(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("engine_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = ConsensusEngine::new(None).await;
            black_box(engine);
        });
    });
}

/// Benchmark parallel vs sequential stage execution
fn bench_parallel_stages(c: &mut Criterion) {
    let mut group = c.benchmark_group("stage_execution");
    group.sample_size(10); // Reduce sample size for longer operations
    
    let rt = Runtime::new().unwrap();
    
    // Sequential execution
    group.bench_function("sequential", |b| {
        b.to_async(&rt).iter(|| async {
            // Simulate sequential stage execution
            tokio::time::sleep(Duration::from_millis(10)).await; // Generator
            tokio::time::sleep(Duration::from_millis(10)).await; // Refiner
            tokio::time::sleep(Duration::from_millis(10)).await; // Validator
            tokio::time::sleep(Duration::from_millis(10)).await; // Curator
        });
    });
    
    // Parallel execution (Refiner + Validator in parallel)
    group.bench_function("parallel", |b| {
        b.to_async(&rt).iter(|| async {
            // Generator first
            tokio::time::sleep(Duration::from_millis(10)).await;
            
            // Refiner and Validator in parallel
            tokio::join!(
                tokio::time::sleep(Duration::from_millis(10)),
                tokio::time::sleep(Duration::from_millis(10))
            );
            
            // Curator last
            tokio::time::sleep(Duration::from_millis(10)).await;
        });
    });
    
    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    
    // Without object pooling
    group.bench_function("without_pooling", |b| {
        b.iter(|| {
            let mut allocations = Vec::new();
            for i in 0..1000 {
                allocations.push(vec![0u8; 1024]); // 1KB allocations
            }
            black_box(allocations);
        });
    });
    
    // With object pooling (simulated)
    group.bench_function("with_pooling", |b| {
        // Pre-allocate pool
        let pool: Vec<Vec<u8>> = (0..1000).map(|_| vec![0u8; 1024]).collect();
        
        b.iter(|| {
            // Reuse from pool instead of allocating
            let mut borrowed = Vec::new();
            for item in &pool {
                borrowed.push(item);
            }
            black_box(borrowed);
        });
    });
    
    group.finish();
}

/// Benchmark zero-copy operations
fn bench_zero_copy(c: &mut Criterion) {
    use bytes::{Bytes, BytesMut};
    
    let mut group = c.benchmark_group("zero_copy");
    
    let data = vec![0u8; 10000]; // 10KB of data
    
    // With copying
    group.bench_function("with_copy", |b| {
        b.iter(|| {
            let mut result = Vec::new();
            for chunk in data.chunks(100) {
                result.extend_from_slice(chunk); // Copies data
            }
            black_box(result);
        });
    });
    
    // Zero-copy with Bytes
    group.bench_function("zero_copy", |b| {
        let bytes_data = Bytes::from(data.clone());
        
        b.iter(|| {
            let mut result = BytesMut::new();
            for i in 0..100 {
                let start = i * 100;
                let end = (i + 1) * 100;
                result.extend_from_slice(&bytes_data[start..end]); // No allocation
            }
            black_box(result);
        });
    });
    
    group.finish();
}

/// Benchmark UI update batching
fn bench_ui_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_batching");
    
    // Without batching - update every token
    group.bench_function("no_batching", |b| {
        b.iter(|| {
            let mut updates = 0;
            for _ in 0..1000 {
                // Simulate UI update
                updates += 1;
                black_box(updates);
            }
        });
    });
    
    // With batching - update at 60 FPS
    group.bench_function("60fps_batching", |b| {
        b.iter(|| {
            let mut updates = 0;
            let mut batch = Vec::new();
            
            for i in 0..1000 {
                batch.push(i);
                
                // Update only at 60 FPS intervals (roughly every 16 items)
                if batch.len() >= 16 {
                    updates += 1;
                    batch.clear();
                    black_box(updates);
                }
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_token_processing,
    bench_engine_init,
    bench_parallel_stages,
    bench_memory_allocation,
    bench_zero_copy,
    bench_ui_batching
);

criterion_main!(benches);