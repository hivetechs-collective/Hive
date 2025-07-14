//! Consensus Pipeline Performance Benchmarks
//! Measures performance of the 4-stage consensus pipeline against TypeScript baseline

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hive_ai::consensus::{ConsensusEngine, ConsensusProfile, ConsensusRequest};
use hive_ai::core::config::HiveConfig;
use std::time::Duration;
use tokio::runtime::Runtime;

/// TypeScript baseline performance metrics (from CLAUDE.md)
const TYPESCRIPT_BASELINE: Duration = Duration::from_millis(3200);
const RUST_TARGET: Duration = Duration::from_millis(500);

/// Mock consensus request for benchmarking
fn create_test_request() -> ConsensusRequest {
    ConsensusRequest {
        query: "Explain this code function".to_string(),
        context: Some("function add(a, b) { return a + b; }".to_string()),
        profile: ConsensusProfile::Balanced,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(1000),
    }
}

/// Benchmark the full 4-stage consensus pipeline
fn bench_consensus_pipeline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = HiveConfig::default();

    let mut group = c.benchmark_group("consensus_pipeline");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    // Benchmark different consensus profiles
    for profile in &[
        ConsensusProfile::Speed,
        ConsensusProfile::Balanced,
        ConsensusProfile::Elite,
    ] {
        group.bench_with_input(
            BenchmarkId::new("full_pipeline", format!("{:?}", profile)),
            profile,
            |b, profile| {
                b.to_async(&rt).iter(|| async {
                    let engine = ConsensusEngine::new(config.clone()).await.unwrap();
                    let mut request = create_test_request();
                    request.profile = profile.clone();

                    black_box(engine.process_request(request).await)
                })
            },
        );
    }

    // Benchmark individual stages
    group.bench_function("generator_stage", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = ConsensusEngine::new(config.clone()).await.unwrap();
            let request = create_test_request();

            black_box(engine.generate_stage(&request).await)
        })
    });

    group.bench_function("refiner_stage", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = ConsensusEngine::new(config.clone()).await.unwrap();
            let request = create_test_request();
            let generated = "Sample generated response".to_string();

            black_box(engine.refine_stage(&request, &generated).await)
        })
    });

    group.bench_function("validator_stage", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = ConsensusEngine::new(config.clone()).await.unwrap();
            let request = create_test_request();
            let refined = "Sample refined response".to_string();

            black_box(engine.validate_stage(&request, &refined).await)
        })
    });

    group.bench_function("curator_stage", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = ConsensusEngine::new(config.clone()).await.unwrap();
            let request = create_test_request();
            let validated = "Sample validated response".to_string();

            black_box(engine.curate_stage(&request, &validated).await)
        })
    });

    group.finish();
}

/// Benchmark concurrent consensus requests
fn bench_concurrent_consensus(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = HiveConfig::default();

    let mut group = c.benchmark_group("concurrent_consensus");
    group.measurement_time(Duration::from_secs(30));

    for concurrency in &[1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let engine = ConsensusEngine::new(config.clone()).await.unwrap();

                    let futures: Vec<_> = (0..concurrency)
                        .map(|_| {
                            let request = create_test_request();
                            engine.process_request(request)
                        })
                        .collect();

                    black_box(futures::future::join_all(futures).await)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark streaming vs non-streaming consensus
fn bench_streaming_consensus(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = HiveConfig::default();

    let mut group = c.benchmark_group("streaming_consensus");
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("streaming_enabled", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = ConsensusEngine::new(config.clone()).await.unwrap();
            let mut request = create_test_request();
            request.stream = true;

            black_box(engine.process_request(request).await)
        })
    });

    group.bench_function("streaming_disabled", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = ConsensusEngine::new(config.clone()).await.unwrap();
            let mut request = create_test_request();
            request.stream = false;

            black_box(engine.process_request(request).await)
        })
    });

    group.finish();
}

/// Benchmark consensus with different input sizes
fn bench_input_size_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = HiveConfig::default();

    let mut group = c.benchmark_group("input_size_scaling");
    group.measurement_time(Duration::from_secs(30));

    for size in &[100, 1000, 5000, 10000] {
        let context = "x".repeat(*size);

        group.bench_with_input(
            BenchmarkId::new("context_size", size),
            &context,
            |b, context| {
                b.to_async(&rt).iter(|| async {
                    let engine = ConsensusEngine::new(config.clone()).await.unwrap();
                    let mut request = create_test_request();
                    request.context = Some(context.clone());

                    black_box(engine.process_request(request).await)
                })
            },
        );
    }

    group.finish();
}

/// Performance regression test against TypeScript baseline
fn bench_typescript_regression(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = HiveConfig::default();

    c.bench_function("typescript_regression_test", |b| {
        b.to_async(&rt).iter(|| async {
            let start = std::time::Instant::now();

            let engine = ConsensusEngine::new(config.clone()).await.unwrap();
            let request = create_test_request();
            let _result = engine.process_request(request).await;

            let duration = start.elapsed();

            // Assert we're faster than TypeScript baseline
            assert!(
                duration < TYPESCRIPT_BASELINE,
                "Consensus pipeline too slow: {:?} vs TypeScript baseline {:?}",
                duration,
                TYPESCRIPT_BASELINE
            );

            // Assert we meet our Rust target
            assert!(
                duration < RUST_TARGET,
                "Consensus pipeline doesn't meet Rust target: {:?} vs target {:?}",
                duration,
                RUST_TARGET
            );

            black_box(duration)
        })
    });
}

criterion_group!(
    benches,
    bench_consensus_pipeline,
    bench_concurrent_consensus,
    bench_streaming_consensus,
    bench_input_size_scaling,
    bench_typescript_regression
);

criterion_main!(benches);
