//! Repository Analysis Performance Benchmarks
//! Measures repository analysis and code intelligence performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hive::analysis::{
    DependencyAnalyzer, LanguageDetector, RepositoryIntelligence, SymbolIndex,
};
use hive::core::config::HiveConfig;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;
use tokio::runtime::Runtime;

/// TypeScript baseline performance metrics (from CLAUDE.md)
const TYPESCRIPT_FILE_PARSE_BASELINE: Duration = Duration::from_millis(50);
const RUST_FILE_PARSE_TARGET: Duration = Duration::from_millis(5);

/// Create a sample Rust file for testing
fn create_sample_rust_file(size: usize) -> String {
    let base_content = r#"
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestStruct {
    pub id: String,
    pub name: String,
    pub data: HashMap<String, String>,
}

impl TestStruct {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            data: HashMap::new(),
        }
    }

    pub fn add_data(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

pub fn process_data(input: &[TestStruct]) -> Vec<String> {
    input.iter().map(|s| s.name.clone()).collect()
}
"#;

    // Repeat content to reach desired size
    let repetitions = (size / base_content.len()).max(1);
    base_content.repeat(repetitions)
}

/// Create a sample TypeScript file for testing
fn create_sample_typescript_file(size: usize) -> String {
    let base_content = r#"
interface TestInterface {
    id: string;
    name: string;
    data: Record<string, string>;
}

class TestClass implements TestInterface {
    public id: string;
    public name: string;
    public data: Record<string, string>;

    constructor(id: string, name: string) {
        this.id = id;
        this.name = name;
        this.data = {};
    }

    addData(key: string, value: string): void {
        this.data[key] = value;
    }

    getData(key: string): string | undefined {
        return this.data[key];
    }
}

export function processData(input: TestInterface[]): string[] {
    return input.map(item => item.name);
}
"#;

    let repetitions = (size / base_content.len()).max(1);
    base_content.repeat(repetitions)
}

/// Benchmark language detection
fn bench_language_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("language_detection");
    group.measurement_time(Duration::from_secs(20));

    let detector = LanguageDetector::new();

    let test_files = vec![
        ("test.rs", create_sample_rust_file(1000)),
        ("test.ts", create_sample_typescript_file(1000)),
        (
            "test.py",
            "def hello():\n    print('Hello, World!')".to_string(),
        ),
        (
            "test.js",
            "function hello() {\n    console.log('Hello, World!');\n}".to_string(),
        ),
        (
            "test.go",
            "package main\n\nfunc main() {\n    println(\"Hello, World!\")\n}".to_string(),
        ),
    ];

    for (filename, content) in test_files {
        group.bench_with_input(
            BenchmarkId::new("detect_language", filename),
            &(filename, &content),
            |b, (filename, content)| {
                b.iter(|| black_box(detector.detect_language(filename, content)))
            },
        );
    }

    group.finish();
}

/// Benchmark file parsing performance
fn bench_file_parsing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("file_parsing");
    group.measurement_time(Duration::from_secs(30));

    let config = HiveConfig::default();

    for size in &[100, 1000, 5000, 10000] {
        let rust_content = create_sample_rust_file(*size);
        let ts_content = create_sample_typescript_file(*size);

        group.bench_with_input(
            BenchmarkId::new("parse_rust_file", size),
            &rust_content,
            |b, content| {
                b.to_async(&rt).iter(|| async {
                    let repo_intel = RepositoryIntelligence::new(config.clone()).await.unwrap();
                    black_box(repo_intel.parse_file("test.rs", content).await)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parse_typescript_file", size),
            &ts_content,
            |b, content| {
                b.to_async(&rt).iter(|| async {
                    let repo_intel = RepositoryIntelligence::new(config.clone()).await.unwrap();
                    black_box(repo_intel.parse_file("test.ts", content).await)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark symbol indexing
fn bench_symbol_indexing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("symbol_indexing");
    group.measurement_time(Duration::from_secs(30));

    let config = HiveConfig::default();

    for file_count in &[10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::new("index_symbols", file_count),
            file_count,
            |b, &file_count| {
                b.to_async(&rt).iter(|| async {
                    let mut symbol_index = SymbolIndex::new();

                    for i in 0..file_count {
                        let filename = format!("file_{}.rs", i);
                        let content = create_sample_rust_file(1000);

                        symbol_index.index_file(&filename, &content).await.unwrap();
                    }

                    black_box(symbol_index.get_symbol_count())
                })
            },
        );
    }

    group.finish();
}

/// Benchmark dependency analysis
fn bench_dependency_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("dependency_analysis");
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("analyze_rust_dependencies", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let project_path = temp_dir.path();

            // Create a mock Cargo.toml
            let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
anyhow = "1.0"
"#;

            tokio::fs::write(project_path.join("Cargo.toml"), cargo_toml)
                .await
                .unwrap();

            let analyzer = DependencyAnalyzer::new();
            black_box(analyzer.analyze_rust_project(project_path).await)
        })
    });

    group.bench_function("analyze_typescript_dependencies", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let project_path = temp_dir.path();

            // Create a mock package.json
            let package_json = r#"
{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {
    "react": "^18.0.0",
    "typescript": "^4.9.0",
    "lodash": "^4.17.0"
  }
}
"#;

            tokio::fs::write(project_path.join("package.json"), package_json)
                .await
                .unwrap();

            let analyzer = DependencyAnalyzer::new();
            black_box(analyzer.analyze_typescript_project(project_path).await)
        })
    });

    group.finish();
}

/// Benchmark full repository analysis
fn bench_repository_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("repository_analysis");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    for file_count in &[10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("full_analysis", file_count),
            file_count,
            |b, &file_count| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = tempdir().unwrap();
                    let project_path = temp_dir.path();
                    let config = HiveConfig::default();

                    // Create mock project structure
                    tokio::fs::create_dir_all(project_path.join("src"))
                        .await
                        .unwrap();

                    for i in 0..file_count {
                        let filename = format!("file_{}.rs", i);
                        let content = create_sample_rust_file(1000);
                        tokio::fs::write(project_path.join("src").join(&filename), content)
                            .await
                            .unwrap();
                    }

                    let repo_intel = RepositoryIntelligence::new(config).await.unwrap();
                    black_box(repo_intel.analyze_repository(project_path).await)
                })
            },
        );
    }

    group.finish();
}

/// Performance regression test against TypeScript baseline
fn bench_typescript_regression(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("file_parsing_typescript_regression", |b| {
        b.to_async(&rt).iter(|| async {
            let start = std::time::Instant::now();

            let config = HiveConfig::default();
            let repo_intel = RepositoryIntelligence::new(config).await.unwrap();
            let content = create_sample_rust_file(1000);
            let _result = repo_intel.parse_file("test.rs", &content).await;

            let duration = start.elapsed();

            // Assert we're faster than TypeScript baseline
            assert!(
                duration < TYPESCRIPT_FILE_PARSE_BASELINE,
                "File parsing too slow: {:?} vs TypeScript baseline {:?}",
                duration,
                TYPESCRIPT_FILE_PARSE_BASELINE
            );

            // Assert we meet our Rust target
            assert!(
                duration < RUST_FILE_PARSE_TARGET,
                "File parsing doesn't meet Rust target: {:?} vs target {:?}",
                duration,
                RUST_FILE_PARSE_TARGET
            );

            black_box(duration)
        })
    });
}

/// Benchmark incremental analysis performance
fn bench_incremental_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("incremental_analysis");
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("incremental_file_update", |b| {
        b.to_async(&rt).iter(|| async {
            let config = HiveConfig::default();
            let repo_intel = RepositoryIntelligence::new(config).await.unwrap();

            // Initial analysis
            let content = create_sample_rust_file(1000);
            repo_intel.parse_file("test.rs", &content).await.unwrap();

            // Incremental update
            let updated_content = create_sample_rust_file(1100);
            black_box(repo_intel.update_file("test.rs", &updated_content).await)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_language_detection,
    bench_file_parsing,
    bench_symbol_indexing,
    bench_dependency_analysis,
    bench_repository_analysis,
    bench_typescript_regression,
    bench_incremental_analysis
);

criterion_main!(benches);
