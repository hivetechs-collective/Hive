//! Database Operations Performance Benchmarks
//! Measures database performance against TypeScript baseline and targets

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hive::core::database::{Conversation, ConversationMetadata, HiveDatabase};
use hive::core::memory::{MemoryCluster, ThematicMemory};
use std::time::Duration;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// TypeScript baseline performance metrics (from CLAUDE.md)
const TYPESCRIPT_DB_BASELINE: Duration = Duration::from_millis(35);
const RUST_DB_TARGET: Duration = Duration::from_millis(3);

/// Create a test conversation for benchmarking
fn create_test_conversation(id: Option<String>) -> Conversation {
    Conversation {
        id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        title: "Test Conversation".to_string(),
        messages: vec![
            serde_json::json!({
                "role": "user",
                "content": "Hello, can you help me with this code?"
            }),
            serde_json::json!({
                "role": "assistant",
                "content": "Of course! I'd be happy to help you with your code."
            }),
        ],
        metadata: ConversationMetadata {
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: vec!["code".to_string(), "help".to_string()],
            quality_score: Some(0.85),
            token_count: Some(100),
            cost_usd: Some(0.001),
        },
        summary: Some("User asking for code help".to_string()),
        theme_cluster: Some("programming_assistance".to_string()),
    }
}

/// Benchmark basic database operations
fn bench_basic_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("database_basic_operations");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(100);

    group.bench_function("create_conversation", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let db = HiveDatabase::new(&db_path).await.unwrap();

            let conversation = create_test_conversation(None);
            black_box(db.save_conversation(&conversation).await)
        })
    });

    group.bench_function("get_conversation", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let db = HiveDatabase::new(&db_path).await.unwrap();

            let conversation = create_test_conversation(Some("test-id".to_string()));
            db.save_conversation(&conversation).await.unwrap();

            black_box(db.get_conversation("test-id").await)
        })
    });

    group.bench_function("update_conversation", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let db = HiveDatabase::new(&db_path).await.unwrap();

            let mut conversation = create_test_conversation(Some("test-id".to_string()));
            db.save_conversation(&conversation).await.unwrap();

            conversation.title = "Updated Title".to_string();
            black_box(db.save_conversation(&conversation).await)
        })
    });

    group.bench_function("delete_conversation", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let db = HiveDatabase::new(&db_path).await.unwrap();

            let conversation = create_test_conversation(Some("test-id".to_string()));
            db.save_conversation(&conversation).await.unwrap();

            black_box(db.delete_conversation("test-id").await)
        })
    });

    group.finish();
}

/// Benchmark database queries and searches
fn bench_query_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("database_query_operations");
    group.measurement_time(Duration::from_secs(20));

    // Setup database with test data
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    rt.block_on(async {
        let db = HiveDatabase::new(&db_path).await.unwrap();

        // Insert 1000 test conversations
        for i in 0..1000 {
            let conversation = create_test_conversation(Some(format!("conv-{}", i)));
            db.save_conversation(&conversation).await.unwrap();
        }
    });

    group.bench_function("list_conversations", |b| {
        b.to_async(&rt).iter(|| async {
            let db = HiveDatabase::new(&db_path).await.unwrap();
            black_box(db.list_conversations(Some(50), Some(0)).await)
        })
    });

    group.bench_function("search_conversations", |b| {
        b.to_async(&rt).iter(|| async {
            let db = HiveDatabase::new(&db_path).await.unwrap();
            black_box(db.search_conversations("code", Some(20)).await)
        })
    });

    group.bench_function("get_conversation_stats", |b| {
        b.to_async(&rt).iter(|| async {
            let db = HiveDatabase::new(&db_path).await.unwrap();
            black_box(db.get_conversation_stats().await)
        })
    });

    group.finish();
}

/// Benchmark memory clustering operations
fn bench_memory_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("database_memory_operations");
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("create_memory_cluster", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let db = HiveDatabase::new(&db_path).await.unwrap();

            let cluster = MemoryCluster {
                id: Uuid::new_v4().to_string(),
                theme: "programming".to_string(),
                conversation_ids: vec!["conv-1".to_string(), "conv-2".to_string()],
                summary: "Programming related conversations".to_string(),
                keywords: vec!["code".to_string(), "programming".to_string()],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                quality_score: 0.8,
            };

            black_box(db.save_memory_cluster(&cluster).await)
        })
    });

    group.bench_function("find_similar_memories", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let db = HiveDatabase::new(&db_path).await.unwrap();

            black_box(db.find_similar_memories("programming help", 5).await)
        })
    });

    group.finish();
}

/// Benchmark concurrent database operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("database_concurrent_operations");
    group.measurement_time(Duration::from_secs(30));

    for concurrency in &[1, 5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_writes", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = tempdir().unwrap();
                    let db_path = temp_dir.path().join("test.db");
                    let db = HiveDatabase::new(&db_path).await.unwrap();

                    let futures: Vec<_> = (0..concurrency)
                        .map(|i| {
                            let conversation =
                                create_test_conversation(Some(format!("concurrent-{}", i)));
                            db.save_conversation(&conversation)
                        })
                        .collect();

                    black_box(futures::future::join_all(futures).await)
                })
            },
        );
    }

    for concurrency in &[1, 5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_reads", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = tempdir().unwrap();
                    let db_path = temp_dir.path().join("test.db");
                    let db = HiveDatabase::new(&db_path).await.unwrap();

                    // Setup test data
                    for i in 0..concurrency {
                        let conversation =
                            create_test_conversation(Some(format!("concurrent-{}", i)));
                        db.save_conversation(&conversation).await.unwrap();
                    }

                    let futures: Vec<_> = (0..concurrency)
                        .map(|i| db.get_conversation(&format!("concurrent-{}", i)))
                        .collect();

                    black_box(futures::future::join_all(futures).await)
                })
            },
        );
    }

    group.finish();
}

/// Performance regression test against TypeScript baseline
fn bench_typescript_regression(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("database_typescript_regression", |b| {
        b.to_async(&rt).iter(|| async {
            let start = std::time::Instant::now();

            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let db = HiveDatabase::new(&db_path).await.unwrap();

            let conversation = create_test_conversation(None);
            let _result = db.save_conversation(&conversation).await;

            let duration = start.elapsed();

            // Assert we're faster than TypeScript baseline
            assert!(
                duration < TYPESCRIPT_DB_BASELINE,
                "Database operation too slow: {:?} vs TypeScript baseline {:?}",
                duration,
                TYPESCRIPT_DB_BASELINE
            );

            // Assert we meet our Rust target
            assert!(
                duration < RUST_DB_TARGET,
                "Database operation doesn't meet Rust target: {:?} vs target {:?}",
                duration,
                RUST_DB_TARGET
            );

            black_box(duration)
        })
    });
}

/// Benchmark database with large datasets
fn bench_large_dataset_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("database_large_dataset");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    for dataset_size in &[1000, 5000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("bulk_insert", dataset_size),
            dataset_size,
            |b, &dataset_size| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = tempdir().unwrap();
                    let db_path = temp_dir.path().join("test.db");
                    let db = HiveDatabase::new(&db_path).await.unwrap();

                    let conversations: Vec<_> = (0..dataset_size)
                        .map(|i| create_test_conversation(Some(format!("bulk-{}", i))))
                        .collect();

                    for conversation in conversations {
                        db.save_conversation(&conversation).await.unwrap();
                    }

                    black_box(dataset_size)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_basic_operations,
    bench_query_operations,
    bench_memory_operations,
    bench_concurrent_operations,
    bench_typescript_regression,
    bench_large_dataset_operations
);

criterion_main!(benches);
