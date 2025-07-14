//! Startup Performance Benchmarks
//! Validates <50ms startup time target against TypeScript baseline of 2.1s

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Instant;
use tokio;
use anyhow::Result;
use serial_test::serial;

/// Target startup time in milliseconds
const STARTUP_TIME_TARGET: u64 = 50;

/// Benchmark binary startup time
#[tokio::test]
#[serial]
async fn benchmark_binary_startup() -> Result<()> {
    let binary_path = get_hive_binary_path()?;

    // Warm up the system (run once to load shared libraries)
    let _ = Command::new(&binary_path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();

    // Run multiple measurements for accuracy
    let iterations = 10;
    let mut measurements = Vec::new();

    for i in 0..iterations {
        let start = Instant::now();

        let output = Command::new(&binary_path)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let duration = start.elapsed();

        // Verify command succeeded
        assert!(output.status.success(),
               "Hive binary should execute successfully on iteration {}", i);

        measurements.push(duration.as_millis() as f64);
    }

    // Calculate statistics
    let avg_startup = measurements.iter().sum::<f64>() / measurements.len() as f64;
    let min_startup = measurements.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_startup = measurements.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Calculate standard deviation
    let variance = measurements.iter()
        .map(|x| (x - avg_startup).powi(2))
        .sum::<f64>() / measurements.len() as f64;
    let std_dev = variance.sqrt();

    println!("Startup Time Statistics:");
    println!("  Average: {:.2}ms", avg_startup);
    println!("  Minimum: {:.2}ms", min_startup);
    println!("  Maximum: {:.2}ms", max_startup);
    println!("  Std Dev: {:.2}ms", std_dev);
    println!("  Target:  {}ms", STARTUP_TIME_TARGET);

    // Validate against target
    assert!(avg_startup < STARTUP_TIME_TARGET as f64,
           "Average startup time ({:.2}ms) must be less than target ({}ms)",
           avg_startup, STARTUP_TIME_TARGET);

    // Consistency check - 95% of measurements should be within 2 standard deviations
    let within_2_std = measurements.iter()
        .filter(|&&x| (x - avg_startup).abs() <= 2.0 * std_dev)
        .count();
    let consistency_percentage = (within_2_std as f64 / measurements.len() as f64) * 100.0;

    assert!(consistency_percentage >= 95.0,
           "Startup time should be consistent ({}% within 2 std dev, expected ≥95%)",
           consistency_percentage);

    // Performance regression check
    let typescript_baseline = 2100.0; // 2.1 seconds in ms
    let improvement_factor = typescript_baseline / avg_startup;

    println!("Performance vs TypeScript baseline:");
    println!("  TypeScript: {:.0}ms", typescript_baseline);
    println!("  Rust:       {:.2}ms", avg_startup);
    println!("  Improvement: {:.1}x faster", improvement_factor);

    assert!(improvement_factor >= 10.0,
           "Should be at least 10x faster than TypeScript baseline (actual: {:.1}x)",
           improvement_factor);

    let mut metrics = HashMap::new();
    metrics.insert("average_startup_time".to_string(), avg_startup);
    metrics.insert("min_startup_time".to_string(), min_startup);
    metrics.insert("max_startup_time".to_string(), max_startup);
    metrics.insert("startup_consistency".to_string(), consistency_percentage);
    metrics.insert("improvement_factor".to_string(), improvement_factor);

    Ok(())
}

/// Benchmark cold start performance
#[tokio::test]
#[serial]
async fn benchmark_cold_start() -> Result<()> {
    let binary_path = get_hive_binary_path()?;

    // Clear system caches (best effort)
    clear_system_caches().await;

    // Measure cold start
    let start = Instant::now();

    let output = Command::new(&binary_path)
        .arg("--help")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let cold_start_time = start.elapsed();

    assert!(output.status.success(), "Cold start should succeed");

    // Cold start should still be reasonable
    let cold_start_ms = cold_start_time.as_millis() as f64;
    assert!(cold_start_ms < STARTUP_TIME_TARGET as f64 * 2.0,
           "Cold start time ({:.2}ms) should be within 2x target ({}ms)",
           cold_start_ms, STARTUP_TIME_TARGET * 2);

    println!("Cold start time: {:.2}ms", cold_start_ms);

    let mut metrics = HashMap::new();
    metrics.insert("cold_start_time".to_string(), cold_start_ms);

    Ok(())
}

/// Benchmark startup with different configurations
#[tokio::test]
#[serial]
async fn benchmark_startup_configurations() -> Result<()> {
    let binary_path = get_hive_binary_path()?;

    let test_configurations = vec![
        ("minimal", vec!["--version"]),
        ("help", vec!["--help"]),
        ("config_check", vec!["config", "--check"]),
        ("status", vec!["status"]),
    ];

    for (config_name, args) in test_configurations {
        let mut measurements = Vec::new();

        // Run each configuration multiple times
        for _ in 0..5 {
            let start = Instant::now();

            let output = Command::new(&binary_path)
                .args(&args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()?;

            let duration = start.elapsed();

            // Allow non-zero exit codes for some commands (like config check without config)
            measurements.push(duration.as_millis() as f64);
        }

        let avg_time = measurements.iter().sum::<f64>() / measurements.len() as f64;

        println!("Configuration '{}': {:.2}ms average", config_name, avg_time);

        // All configurations should start quickly
        assert!(avg_time < STARTUP_TIME_TARGET as f64 * 1.5,
               "Configuration '{}' startup ({:.2}ms) exceeds limit",
               config_name, avg_time);
    }

    let mut metrics = HashMap::new();
    metrics.insert("configuration_tests".to_string(), test_configurations.len() as f64);

    Ok(())
}

/// Benchmark startup under load
#[tokio::test]
#[serial]
async fn benchmark_startup_under_load() -> Result<()> {
    let binary_path = get_hive_binary_path()?;

    // Create system load by running multiple instances concurrently
    let concurrent_instances = 5;
    let mut handles = Vec::new();

    let start = Instant::now();

    for i in 0..concurrent_instances {
        let binary_path_clone = binary_path.clone();

        handles.push(tokio::spawn(async move {
            let instance_start = Instant::now();

            let output = Command::new(&binary_path_clone)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output();

            let duration = instance_start.elapsed();

            match output {
                Ok(output) if output.status.success() => Ok(duration.as_millis() as f64),
                Ok(_) => Err(anyhow::anyhow!("Command failed for instance {}", i)),
                Err(e) => Err(anyhow::anyhow!("Failed to execute instance {}: {}", i, e)),
            }
        }));
    }

    // Wait for all instances to complete
    let results = futures::future::join_all(handles).await;
    let total_time = start.elapsed();

    let mut successful_times = Vec::new();
    let mut failed_count = 0;

    for result in results {
        match result {
            Ok(Ok(time)) => successful_times.push(time),
            _ => failed_count += 1,
        }
    }

    // Should have most instances succeed
    assert!(successful_times.len() >= concurrent_instances * 80 / 100,
           "At least 80% of concurrent instances should succeed");

    if !successful_times.is_empty() {
        let avg_concurrent_time = successful_times.iter().sum::<f64>() / successful_times.len() as f64;
        let max_concurrent_time = successful_times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        println!("Concurrent startup results:");
        println!("  Instances: {}", concurrent_instances);
        println!("  Successful: {}", successful_times.len());
        println!("  Failed: {}", failed_count);
        println!("  Average time: {:.2}ms", avg_concurrent_time);
        println!("  Max time: {:.2}ms", max_concurrent_time);
        println!("  Total time: {:.2}ms", total_time.as_millis());

        // Under load, startup time may be higher but should still be reasonable
        assert!(avg_concurrent_time < STARTUP_TIME_TARGET as f64 * 3.0,
               "Average concurrent startup time ({:.2}ms) should be reasonable",
               avg_concurrent_time);
    }

    let mut metrics = HashMap::new();
    metrics.insert("concurrent_instances".to_string(), concurrent_instances as f64);
    metrics.insert("successful_instances".to_string(), successful_times.len() as f64);
    metrics.insert("total_concurrent_time".to_string(), total_time.as_millis() as f64);

    Ok(())
}

/// Helper function to get the hive binary path
fn get_hive_binary_path() -> Result<String> {
    // Try different possible locations
    let possible_paths = vec![
        "target/release/hive",
        "target/debug/hive",
        "./hive",
        "/usr/local/bin/hive",
    ];

    for path in possible_paths {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }

    // Try building the release binary
    let build_output = Command::new("cargo")
        .args(&["build", "--release", "--bin", "hive"])
        .output()?;

    if build_output.status.success() {
        if std::path::Path::new("target/release/hive").exists() {
            return Ok("target/release/hive".to_string());
        }
    }

    anyhow::bail!("Could not find or build hive binary")
}

/// Helper function to clear system caches (best effort)
async fn clear_system_caches() {
    // On macOS, try to clear caches
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("sudo")
            .args(&["purge"])
            .output();
    }

    // On Linux, try to clear page cache
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("sudo")
            .args(&["sh", "-c", "echo 1 > /proc/sys/vm/drop_caches"])
            .output();
    }

    // Wait a bit for caches to clear
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

/// Benchmark memory usage during startup
#[tokio::test]
#[serial]
async fn benchmark_startup_memory() -> Result<()> {
    let binary_path = get_hive_binary_path()?;

    // Start the process and monitor memory usage
    let mut child = Command::new(&binary_path)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let pid = child.id();

    // Monitor memory usage
    let mut memory_samples = Vec::new();
    let monitoring_start = Instant::now();

    while monitoring_start.elapsed() < std::time::Duration::from_millis(200) {
        if let Ok(memory) = get_process_memory(pid) {
            memory_samples.push(memory as f64 / 1024.0 / 1024.0); // Convert to MB
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Check if process has exited
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
    }

    // Ensure process completes
    let output = child.wait_with_output()?;
    assert!(output.status.success(), "Process should complete successfully");

    if !memory_samples.is_empty() {
        let max_memory = memory_samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let avg_memory = memory_samples.iter().sum::<f64>() / memory_samples.len() as f64;

        println!("Startup memory usage:");
        println!("  Peak memory: {:.2} MB", max_memory);
        println!("  Average memory: {:.2} MB", avg_memory);
        println!("  Samples: {}", memory_samples.len());

        // Memory target: <25MB
        assert!(max_memory < 25.0,
               "Peak memory usage ({:.2} MB) should be less than 25 MB",
               max_memory);

        let mut metrics = HashMap::new();
        metrics.insert("peak_startup_memory".to_string(), max_memory);
        metrics.insert("average_startup_memory".to_string(), avg_memory);

        return Ok(());
    }

    println!("⚠️  Could not measure memory usage during startup");
    Ok(())
}

/// Helper function to get process memory usage
fn get_process_memory(pid: u32) -> Result<u64> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("ps")
            .args(&["-o", "rss=", "-p", &pid.to_string()])
            .output()?;

        if output.status.success() {
            let rss_kb = String::from_utf8(output.stdout)?
                .trim()
                .parse::<u64>()?;
            Ok(rss_kb * 1024) // Convert KB to bytes
        } else {
            anyhow::bail!("Failed to get memory for PID {}", pid);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status_path = format!("/proc/{}/status", pid);
        let content = std::fs::read_to_string(status_path)?;

        for line in content.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let rss_kb = parts[1].parse::<u64>()?;
                    return Ok(rss_kb * 1024); // Convert KB to bytes
                }
            }
        }

        anyhow::bail!("Could not find VmRSS in /proc/{}/status", pid);
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        anyhow::bail!("Memory measurement not implemented for this platform");
    }
}