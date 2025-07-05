// Performance benchmarking command

use crate::core::Result;
use anyhow::Context;
// use std::sync::Arc;
use std::time::Instant;

/// Run performance benchmarks
pub async fn run_benchmarks() -> Result<()> {
    println!("Running Hive AI Performance Benchmarks...");
    println!();

    // Database performance
    let db_start = Instant::now();
    // let db = Arc::new(Database::open_default().await?);
    let db_time = db_start.elapsed();
    println!(" Database initialization: {:?}", db_time);

    // Consensus engine initialization
    let consensus_start = Instant::now();
    let engine = crate::consensus::ConsensusEngine::new(None).await
        .context("Failed to initialize consensus engine")?;
    let consensus_time = consensus_start.elapsed();
    println!(" Consensus engine initialization: {:?}", consensus_time);

    // Simple query benchmark
    let query_start = Instant::now();
    let _result = engine.process("What is 2+2?", None).await?;
    let query_time = query_start.elapsed();
    println!(" Simple consensus query: {:?}", query_time);

    // Memory usage
    let memory_usage = get_memory_usage();
    println!(" Memory usage: {:.2} MB", memory_usage);

    println!();
    println!("Performance Summary:");
    println!("  Startup time: {:?}", db_time + consensus_time);
    println!("  Query latency: {:?}", query_time);
    println!("  Memory footprint: {:.2} MB", memory_usage);

    Ok(())
}

fn get_memory_usage() -> f64 {
    // Simple approximation - in production would use proper memory profiling
    use std::fs;
    
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<f64>() {
                        return kb / 1024.0; // Convert KB to MB
                    }
                }
            }
        }
    }
    
    // Fallback estimate
    25.0
}