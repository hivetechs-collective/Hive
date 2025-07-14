//! HiveTechs Consensus Testing Framework
//! Comprehensive testing suite for production readiness validation

pub mod acceptance;
pub mod integration;
pub mod performance;
pub mod security;
pub mod unit;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Test execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub category: TestCategory,
    pub status: TestStatus,
    pub duration: Duration,
    pub details: Option<String>,
    pub metrics: HashMap<String, f64>,
    pub errors: Vec<String>,
}

/// Test categories for organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestCategory {
    Unit,
    Integration,
    Performance,
    Acceptance,
    Security,
    Regression,
}

/// Test execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Timeout,
    Error,
}

/// Performance target definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTarget {
    pub metric: String,
    pub target_value: f64,
    pub unit: String,
    pub comparison: TargetComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetComparison {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
}

/// Test suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub coverage_threshold: f64,
    pub performance_targets: Vec<PerformanceTarget>,
    pub timeout_limits: HashMap<TestCategory, Duration>,
    pub real_api_testing: bool,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Basic,
    Standard,
    Comprehensive,
    Paranoid,
}

impl Default for TestConfig {
    fn default() -> Self {
        let mut timeout_limits = HashMap::new();
        timeout_limits.insert(TestCategory::Unit, Duration::from_secs(30));
        timeout_limits.insert(TestCategory::Integration, Duration::from_secs(120));
        timeout_limits.insert(TestCategory::Performance, Duration::from_secs(300));
        timeout_limits.insert(TestCategory::Acceptance, Duration::from_secs(600));
        timeout_limits.insert(TestCategory::Security, Duration::from_secs(180));

        Self {
            coverage_threshold: 90.0,
            performance_targets: vec![
                PerformanceTarget {
                    metric: "startup_time".to_string(),
                    target_value: 50.0,
                    unit: "ms".to_string(),
                    comparison: TargetComparison::LessThan,
                },
                PerformanceTarget {
                    metric: "memory_usage".to_string(),
                    target_value: 25.0,
                    unit: "MB".to_string(),
                    comparison: TargetComparison::LessThan,
                },
                PerformanceTarget {
                    metric: "consensus_time".to_string(),
                    target_value: 500.0,
                    unit: "ms".to_string(),
                    comparison: TargetComparison::LessThan,
                },
                PerformanceTarget {
                    metric: "file_parsing_rate".to_string(),
                    target_value: 5.0,
                    unit: "ms/file".to_string(),
                    comparison: TargetComparison::LessThan,
                },
                PerformanceTarget {
                    metric: "database_query_time".to_string(),
                    target_value: 3.0,
                    unit: "ms".to_string(),
                    comparison: TargetComparison::LessThan,
                },
            ],
            timeout_limits,
            real_api_testing: false,
            security_level: SecurityLevel::Comprehensive,
        }
    }
}

/// Test runner for executing test suites
pub struct TestRunner {
    config: TestConfig,
    results: Vec<TestResult>,
}

impl TestRunner {
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// Execute a test with timing and error handling
    pub async fn execute_test<F, Fut>(
        &mut self,
        name: &str,
        category: TestCategory,
        test_fn: F,
    ) -> TestResult
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<HashMap<String, f64>>>,
    {
        let start = Instant::now();
        let timeout = self
            .config
            .timeout_limits
            .get(&category)
            .cloned()
            .unwrap_or(Duration::from_secs(60));

        let result = match tokio::time::timeout(timeout, test_fn()).await {
            Ok(Ok(metrics)) => TestResult {
                name: name.to_string(),
                category,
                status: TestStatus::Passed,
                duration: start.elapsed(),
                details: None,
                metrics,
                errors: Vec::new(),
            },
            Ok(Err(e)) => TestResult {
                name: name.to_string(),
                category,
                status: TestStatus::Failed,
                duration: start.elapsed(),
                details: Some(e.to_string()),
                metrics: HashMap::new(),
                errors: vec![e.to_string()],
            },
            Err(_) => TestResult {
                name: name.to_string(),
                category,
                status: TestStatus::Timeout,
                duration: timeout,
                details: Some("Test exceeded timeout limit".to_string()),
                metrics: HashMap::new(),
                errors: vec!["Timeout".to_string()],
            },
        };

        self.results.push(result.clone());
        result
    }

    /// Generate comprehensive test report
    pub fn generate_report(&self) -> TestReport {
        let total_tests = self.results.len();
        let passed = self
            .results
            .iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count();
        let failed = self
            .results
            .iter()
            .filter(|r| r.status == TestStatus::Failed)
            .count();
        let errors = self
            .results
            .iter()
            .filter(|r| r.status == TestStatus::Error)
            .count();
        let timeouts = self
            .results
            .iter()
            .filter(|r| r.status == TestStatus::Timeout)
            .count();

        let mut category_results = HashMap::new();
        for result in &self.results {
            let entry = category_results
                .entry(result.category.clone())
                .or_insert(CategorySummary::default());
            entry.total += 1;
            match result.status {
                TestStatus::Passed => entry.passed += 1,
                TestStatus::Failed => entry.failed += 1,
                TestStatus::Error => entry.errors += 1,
                TestStatus::Timeout => entry.timeouts += 1,
                TestStatus::Skipped => entry.skipped += 1,
            }
            entry.total_duration += result.duration;
        }

        let performance_validation = self.validate_performance_targets();

        TestReport {
            total_tests,
            passed,
            failed,
            errors,
            timeouts,
            success_rate: if total_tests > 0 {
                (passed as f64 / total_tests as f64) * 100.0
            } else {
                0.0
            },
            category_results,
            performance_validation,
            coverage_percentage: 0.0, // To be filled by coverage tool
            security_score: 0.0,      // To be filled by security tests
            timestamp: chrono::Utc::now(),
        }
    }

    /// Validate performance targets against test results
    fn validate_performance_targets(&self) -> Vec<PerformanceValidation> {
        let mut validations = Vec::new();

        for target in &self.config.performance_targets {
            // Collect all metrics matching this target
            let mut values = Vec::new();
            for result in &self.results {
                if let Some(value) = result.metrics.get(&target.metric) {
                    values.push(*value);
                }
            }

            if !values.is_empty() {
                let avg_value = values.iter().sum::<f64>() / values.len() as f64;
                let max_value = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let min_value = values.iter().cloned().fold(f64::INFINITY, f64::min);

                let meets_target = match target.comparison {
                    TargetComparison::LessThan => avg_value < target.target_value,
                    TargetComparison::LessThanOrEqual => avg_value <= target.target_value,
                    TargetComparison::GreaterThan => avg_value > target.target_value,
                    TargetComparison::GreaterThanOrEqual => avg_value >= target.target_value,
                    TargetComparison::Equal => (avg_value - target.target_value).abs() < 0.001,
                };

                validations.push(PerformanceValidation {
                    metric: target.metric.clone(),
                    target_value: target.target_value,
                    actual_value: avg_value,
                    min_value,
                    max_value,
                    unit: target.unit.clone(),
                    meets_target,
                    sample_count: values.len(),
                });
            }
        }

        validations
    }
}

/// Test report structure
#[derive(Debug, Serialize, Deserialize)]
pub struct TestReport {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub timeouts: usize,
    pub success_rate: f64,
    pub category_results: HashMap<TestCategory, CategorySummary>,
    pub performance_validation: Vec<PerformanceValidation>,
    pub coverage_percentage: f64,
    pub security_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CategorySummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub timeouts: usize,
    pub skipped: usize,
    pub total_duration: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceValidation {
    pub metric: String,
    pub target_value: f64,
    pub actual_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub unit: String,
    pub meets_target: bool,
    pub sample_count: usize,
}

/// Test utilities and helpers
pub mod utils {
    use anyhow::Result;
    use std::path::Path;
    use tempfile::TempDir;
    use tokio::process::Command;

    /// Create temporary directory for testing
    pub fn create_temp_dir() -> Result<TempDir> {
        Ok(tempfile::tempdir()?)
    }

    /// Create test database with sample data
    pub async fn create_test_database(path: &Path) -> Result<()> {
        let db = hive_ai::core::database::HiveDatabase::new(path).await?;

        // Initialize with test schema
        db.initialize().await?;

        // Add sample data
        let conversation_id = uuid::Uuid::new_v4();
        db.store_conversation(
            &conversation_id,
            "Test conversation",
            &serde_json::json!({"test": "data"}),
            "test_user",
        )
        .await?;

        Ok(())
    }

    /// Run external command with timeout
    pub async fn run_command_with_timeout(
        cmd: &str,
        args: &[&str],
        timeout: std::time::Duration,
    ) -> Result<std::process::Output> {
        let output = tokio::time::timeout(timeout, Command::new(cmd).args(args).output()).await??;

        Ok(output)
    }

    /// Measure memory usage of a process
    pub async fn measure_memory_usage(pid: u32) -> Result<u64> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("ps")
                .args(&["-o", "rss=", "-p", &pid.to_string()])
                .output()
                .await?;

            if output.status.success() {
                let rss_kb = String::from_utf8(output.stdout)?.trim().parse::<u64>()?;
                Ok(rss_kb * 1024) // Convert KB to bytes
            } else {
                anyhow::bail!("Failed to get memory usage for PID {}", pid);
            }
        }

        #[cfg(target_os = "linux")]
        {
            let stat_path = format!("/proc/{}/stat", pid);
            let stat_content = tokio::fs::read_to_string(stat_path).await?;
            let fields: Vec<&str> = stat_content.split_whitespace().collect();

            if fields.len() >= 23 {
                let rss_pages = fields[23].parse::<u64>()?;
                let page_size = 4096; // Typical page size on Linux
                Ok(rss_pages * page_size)
            } else {
                anyhow::bail!("Invalid /proc/{}/stat format", pid);
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            // Fallback for other platforms
            anyhow::bail!("Memory measurement not implemented for this platform");
        }
    }

    /// Generate test coverage report
    pub async fn generate_coverage_report() -> Result<f64> {
        // Run cargo tarpaulin for coverage
        let output = Command::new("cargo")
            .args(&[
                "tarpaulin",
                "--output-dir",
                "target/coverage",
                "--out",
                "Json",
            ])
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                // Parse coverage report
                let coverage_file = "target/coverage/tarpaulin-report.json";
                if tokio::fs::metadata(coverage_file).await.is_ok() {
                    let content = tokio::fs::read_to_string(coverage_file).await?;
                    let report: serde_json::Value = serde_json::from_str(&content)?;

                    if let Some(coverage) = report.get("coverage").and_then(|v| v.as_f64()) {
                        return Ok(coverage);
                    }
                }
                Ok(0.0)
            }
            _ => {
                // Fallback to llvm-cov if tarpaulin is not available
                let output = Command::new("cargo")
                    .args(&[
                        "llvm-cov",
                        "--json",
                        "--output-path",
                        "target/coverage/llvm-cov.json",
                    ])
                    .output()
                    .await;

                match output {
                    Ok(output) if output.status.success() => {
                        let coverage_file = "target/coverage/llvm-cov.json";
                        if tokio::fs::metadata(coverage_file).await.is_ok() {
                            let content = tokio::fs::read_to_string(coverage_file).await?;
                            let report: serde_json::Value = serde_json::from_str(&content)?;

                            // Extract coverage percentage from llvm-cov format
                            if let Some(data) = report.get("data") {
                                if let Some(totals) = data.get(0).and_then(|d| d.get("totals")) {
                                    if let Some(lines) = totals.get("lines") {
                                        let covered = lines
                                            .get("covered")
                                            .and_then(|v| v.as_f64())
                                            .unwrap_or(0.0);
                                        let count = lines
                                            .get("count")
                                            .and_then(|v| v.as_f64())
                                            .unwrap_or(1.0);
                                        return Ok((covered / count) * 100.0);
                                    }
                                }
                            }
                        }
                        Ok(0.0)
                    }
                    _ => Ok(0.0),
                }
            }
        }
    }
}
