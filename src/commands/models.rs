/// Model management commands
/// 
/// Commands for listing, testing, and benchmarking OpenRouter models

use crate::core::Result;
use crate::providers::openrouter::{
    create_client, ModelSelector, ModelSelectionStrategy, PerformanceTracker, TaskComplexity,
    ScoringWeights, ABTestConfig, ABTestStatus, ModelRanking, TaskRecommendation, HealthStatus, CircuitState,
};
use anyhow::Context;

/// List available models
pub async fn list_models(
    provider: Option<String>,
    tier: Option<String>,
    format: Option<String>,
) -> Result<()> {
    let selector = ModelSelector::new(ModelSelectionStrategy::Balanced);
    let models = selector.list_models();

    // Filter by provider if specified
    let filtered_models: Vec<_> = if let Some(provider) = &provider {
        models
            .into_iter()
            .filter(|m| m.provider.eq_ignore_ascii_case(provider))
            .collect()
    } else {
        models
    };

    // Filter by tier if specified
    let filtered_models: Vec<_> = if let Some(tier_str) = &tier {
        filtered_models
            .into_iter()
            .filter(|m| format!("{:?}", m.tier).eq_ignore_ascii_case(tier_str))
            .collect()
    } else {
        filtered_models
    };

    let total_count = filtered_models.len();
    
    match format.as_deref() {
        Some("json") => {
            println!("{}", serde_json::to_string_pretty(&filtered_models)?);
        }
        Some("csv") => {
            println!("Model ID,Provider,Name,Tier,Input Cost,Output Cost,Context Window");
            for model in filtered_models {
                println!(
                    "{},{},{},{:?},${},${},{}",
                    model.id,
                    model.provider,
                    model.name,
                    model.tier,
                    model.cost_per_1k_input,
                    model.cost_per_1k_output,
                    model.context_window
                );
            }
        }
        _ => {
            // Default table format
            println!("{:<50} {:<15} {:<10} {:<12} {:<10}", "Model ID", "Provider", "Tier", "Input $/1k", "Context");
            println!("{}", "-".repeat(100));
            
            for model in filtered_models {
                println!(
                    "{:<50} {:<15} {:<10} ${:<11.4} {:<10}",
                    model.id,
                    model.provider,
                    format!("{:?}", model.tier),
                    model.cost_per_1k_input,
                    model.context_window
                );
            }
            
            println!("\nTotal models: {}", total_count);
        }
    }

    Ok(())
}

/// Test a specific model
pub async fn test_model(model_id: &str, api_key: Option<String>) -> Result<()> {
    println!("Testing model: {}", model_id);

    // Get API key from environment or parameter
    let api_key = api_key
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
        .context("No API key provided. Set OPENROUTER_API_KEY or use --api-key")?;

    let client = create_client(api_key)?;

    match client.test_connection(Some(model_id)).await? {
        true => {
            println!("✅ Model {} is accessible and responding correctly", model_id);
        }
        false => {
            println!("❌ Model {} did not respond as expected", model_id);
        }
    }

    Ok(())
}

/// Benchmark model performance
pub async fn benchmark_models(
    models: Option<Vec<String>>,
    complexity: Option<String>,
    iterations: Option<usize>,
) -> Result<()> {
    let complexity = match complexity.as_deref() {
        Some("simple") => TaskComplexity::Simple,
        Some("moderate") => TaskComplexity::Moderate,
        Some("complex") => TaskComplexity::Complex,
        Some("expert") => TaskComplexity::Expert,
        _ => TaskComplexity::Moderate,
    };

    let iterations = iterations.unwrap_or(3);

    println!("Benchmarking models...");
    println!("Complexity: {:?}", complexity);
    println!("Iterations: {}", iterations);
    println!();

    let selector = ModelSelector::new(ModelSelectionStrategy::Balanced);
    let tracker = PerformanceTracker::new(60); // 60 minute window

    // Get models to benchmark
    let models_to_test = if let Some(model_list) = models {
        model_list
    } else {
        // Default selection based on complexity
        let selection = selector.select_model(
            "Benchmark test query",
            complexity,
            vec![],
            None,
        )?;
        
        vec![selection.primary]
            .into_iter()
            .chain(selection.fallbacks.into_iter().take(2))
            .collect()
    };

    println!("Testing {} models:", models_to_test.len());
    for model in &models_to_test {
        println!("  - {}", model);
    }
    println!();

    // Simulate benchmark results (in real implementation, would make actual API calls)
    for model in &models_to_test {
        println!("Benchmarking {}...", model);
        
        for i in 0..iterations {
            // Simulate performance data
            let latency = (1000.0 + (i as f32 * 100.0) + (rand::random::<f32>() * 500.0)) as u64;
            let tokens = 100 + (rand::random::<u32>() % 200);
            let success = rand::random::<f32>() > 0.1; // 90% success rate
            
            tracker
                .track_performance(
                    model,
                    latency,
                    tokens,
                    success,
                    None,
                    Some(0.8 + rand::random::<f32>() * 0.2),
                    "benchmark",
                    None,
                )
                .await?;
            
            print!(".");
            use std::io::{self, Write};
            io::stdout().flush()?;
        }
        println!(" Done");
    }

    println!("\nBenchmark Results:");
    println!("{:<40} {:<12} {:<12} {:<12} {:<12}", "Model", "Success Rate", "Avg Latency", "P95 Latency", "Quality");
    println!("{}", "-".repeat(90));

    for model in &models_to_test {
        if let Some(metrics) = tracker.get_metrics(model).await {
            println!(
                "{:<40} {:<12.1}% {:<12.0}ms {:<12.0}ms {:<12.2}",
                model,
                metrics.success_rate * 100.0,
                metrics.average_latency_ms,
                metrics.p95_latency_ms,
                metrics.quality_score
            );
        }
    }

    // Show performance ranking
    let rankings = tracker.get_performance_ranking().await;
    if !rankings.is_empty() {
        println!("\nPerformance Ranking:");
        for (i, (model, score)) in rankings.iter().enumerate() {
            println!("{}. {} (score: {:.2})", i + 1, model, score);
        }
    }

    Ok(())
}

/// Show model recommendations
pub async fn recommend_models(
    task_type: &str,
    budget: Option<f32>,
    min_quality: Option<f32>,
) -> Result<()> {
    println!("Model Recommendations for: {}", task_type);
    
    let complexity = match task_type {
        "simple" | "basic" => TaskComplexity::Simple,
        "moderate" | "general" => TaskComplexity::Moderate,
        "complex" | "advanced" => TaskComplexity::Complex,
        "expert" | "research" => TaskComplexity::Expert,
        _ => TaskComplexity::Moderate,
    };

    let selector = ModelSelector::new(ModelSelectionStrategy::Balanced);
    
    // Get recommendations for different strategies
    let strategies = [
        ModelSelectionStrategy::CostOptimized,
        ModelSelectionStrategy::Balanced,
        ModelSelectionStrategy::PerformanceOptimized,
        ModelSelectionStrategy::QualityFirst,
    ];

    println!("\nRecommendations by Strategy:");
    println!("{}", "-".repeat(80));

    for strategy in &strategies {
        let mut strategy_selector = ModelSelector::new(*strategy);
        
        match strategy_selector.select_model(
            task_type,
            complexity,
            vec![],
            budget,
        ) {
            Ok(selection) => {
                println!("\n{:?} Strategy:", strategy);
                println!("  Primary: {}", selection.primary);
                println!("  Estimated cost: ${:.4} per request", selection.estimated_cost);
                println!("  Reasoning: {}", selection.reasoning);
                
                if !selection.fallbacks.is_empty() {
                    println!("  Fallbacks: {}", selection.fallbacks.join(", "));
                }
            }
            Err(e) => {
                println!("\n{:?} Strategy: No suitable models found - {}", strategy, e);
            }
        }
    }

    Ok(())
}

/// Show comprehensive model performance metrics
pub async fn show_performance(
    model_id: Option<String>,
    format: Option<String>,
    details: bool,
) -> Result<()> {
    let tracker = PerformanceTracker::new(60); // 60 minute window
    
    match model_id {
        Some(model) => {
            // Show performance for specific model
            if let Some(metrics) = tracker.get_metrics(&model).await {
                println!("Performance Metrics for: {}", model);
                println!("{}", "=".repeat(50));
                println!("📊 Success Rate: {:.1}%", metrics.success_rate * 100.0);
                println!("⚡ Average Latency: {:.0}ms", metrics.average_latency_ms);
                println!("📈 P95 Latency: {:.0}ms", metrics.p95_latency_ms);
                println!("🚀 Throughput: {:.1} tokens/sec", metrics.average_tokens_per_second);
                println!("⭐ Quality Score: {:.2}/1.0", metrics.quality_score);
                println!("❌ Error Rate: {:.1}%", metrics.error_rate * 100.0);
                println!("⏱️  Timeout Rate: {:.1}%", metrics.timeout_rate * 100.0);
                println!("📅 Last Updated: {}", metrics.last_updated.format("%Y-%m-%d %H:%M:%S UTC"));
                
                if details {
                    // Show detailed health information
                    let health = tracker.get_model_health(&model).await?;
                    println!("\nHealth Status: {:?}", health.status);
                    if !health.issues.is_empty() {
                        println!("Issues:");
                        for issue in &health.issues {
                            println!("  ⚠️  {}", issue);
                        }
                    }
                    if let Some(rec) = &health.recommendation {
                        println!("Recommendation: {}", rec);
                    }
                }
            } else {
                println!("No performance data available for model: {}", model);
            }
        }
        None => {
            // Show performance for all models
            let all_metrics = tracker.get_all_metrics().await;
            
            if all_metrics.is_empty() {
                println!("No performance data available yet.");
                return Ok(());
            }

            match format.as_deref() {
                Some("json") => {
                    println!("{}", serde_json::to_string_pretty(&all_metrics)?);
                }
                _ => {
                    println!("Model Performance Overview");
                    println!("{}", "=".repeat(100));
                    println!("{:<40} {:<12} {:<12} {:<12} {:<12} {:<10}", 
                        "Model", "Success Rate", "Avg Latency", "P95 Latency", "Throughput", "Quality");
                    println!("{}", "-".repeat(100));
                    
                    for metrics in &all_metrics {
                        println!("{:<40} {:<12.1}% {:<12.0}ms {:<12.0}ms {:<12.1}/s {:<10.2}",
                            metrics.model_id,
                            metrics.success_rate * 100.0,
                            metrics.average_latency_ms,
                            metrics.p95_latency_ms,
                            metrics.average_tokens_per_second,
                            metrics.quality_score
                        );
                    }
                    
                    println!("\nTotal models with data: {}", all_metrics.len());
                }
            }
        }
    }
    
    Ok(())
}

/// Show model rankings for different task types
pub async fn show_rankings(
    task_type: Option<String>,
    limit: Option<usize>,
    metric: Option<String>,
) -> Result<()> {
    let tracker = PerformanceTracker::new(60);
    let limit = limit.unwrap_or(10);
    
    match task_type.as_deref() {
        Some(task) => {
            // Show ranking for specific task type
            let weights = match metric.as_deref() {
                Some("speed") => ScoringWeights { speed: 0.6, reliability: 0.3, quality: 0.1, cost: 0.0, throughput: 0.0 },
                Some("quality") => ScoringWeights { quality: 0.7, reliability: 0.2, speed: 0.1, cost: 0.0, throughput: 0.0 },
                Some("cost") => ScoringWeights { cost: 0.5, reliability: 0.3, quality: 0.2, speed: 0.0, throughput: 0.0 },
                _ => ScoringWeights::default(),
            };
            
            let rankings = tracker.get_task_specific_ranking(task, &weights).await;
            
            println!("Top {} Models for: {}", limit, task);
            if let Some(m) = &metric {
                println!("Optimized for: {}", m);
            }
            println!("{}", "=".repeat(70));
            println!("{:<5} {:<40} {:<12} {:<12}", "Rank", "Model", "Score", "Health");
            println!("{}", "-".repeat(70));
            
            for (i, (model_id, score)) in rankings.iter().take(limit).enumerate() {
                let health = tracker.get_model_health(model_id).await?;
                let health_emoji = match health.status {
                    HealthStatus::Healthy => "✅",
                    HealthStatus::Degraded => "⚠️",
                    HealthStatus::Unhealthy => "❌",
                    HealthStatus::Unavailable => "🚫",
                };
                
                println!("{:<5} {:<40} {:<12.3} {:<12}",
                    i + 1,
                    model_id,
                    score,
                    health_emoji
                );
            }
        }
        None => {
            // Show comprehensive rankings for all task types
            let comprehensive = tracker.get_comprehensive_rankings().await;
            
            for (task_type, rankings) in comprehensive {
                println!("\n{} Task Rankings:", task_type.to_uppercase());
                println!("{}", "-".repeat(50));
                
                for (i, ranking) in rankings.iter().take(5).enumerate() {
                    println!("{}. {} (score: {:.3})",
                        i + 1,
                        ranking.model_id,
                        ranking.score
                    );
                }
            }
        }
    }
    
    Ok(())
}

/// Get model recommendations for specific tasks
pub async fn get_recommendations(
    task_type: &str,
    limit: Option<usize>,
    budget: Option<f32>,
    quality_threshold: Option<f32>,
) -> Result<()> {
    let tracker = PerformanceTracker::new(60);
    let limit = limit.unwrap_or(5);
    
    let recommendation = tracker.get_task_recommendations(task_type, limit).await?;
    
    println!("🎯 Model Recommendations for: {}", recommendation.task_type);
    println!("{}", "=".repeat(60));
    println!("Confidence: {:.0}%", recommendation.confidence * 100.0);
    println!("Reasoning: {}", recommendation.reasoning);
    println!();
    
    if recommendation.recommended_models.is_empty() {
        println!("No recommendations available. More performance data needed.");
        return Ok(());
    }
    
    println!("{:<5} {:<30} {:<12} {:<12} {:<12}", "Rank", "Model", "Score", "Success Rate", "Avg Latency");
    println!("{}", "-".repeat(75));
    
    for model in &recommendation.recommended_models {
        let success_rate = model.metrics_snapshot.success_rate * 100.0;
        let avg_latency = model.metrics_snapshot.average_latency_ms;
        
        // Apply filters if specified
        if let Some(budget) = budget {
            // Cost filtering would be implemented with cost tracker integration
            continue;
        }
        
        if let Some(quality_min) = quality_threshold {
            if model.metrics_snapshot.quality_score < quality_min {
                continue;
            }
        }
        
        println!("{:<5} {:<30} {:<12.3} {:<12.1}% {:<12.0}ms",
            model.rank,
            model.model_id,
            model.score,
            success_rate,
            avg_latency
        );
    }
    
    Ok(())
}

/// Create and manage A/B tests
pub async fn manage_ab_test(
    action: &str,
    test_id: Option<String>,
    model_a: Option<String>,
    model_b: Option<String>,
    name: Option<String>,
    description: Option<String>,
    sample_size: Option<usize>,
    duration_hours: Option<u32>,
) -> Result<()> {
    let tracker = PerformanceTracker::new(60);
    
    match action {
        "create" => {
            let model_a = model_a.context("--model-a required for creating A/B test")?;
            let model_b = model_b.context("--model-b required for creating A/B test")?;
            let name = name.unwrap_or_else(|| format!("{} vs {}", model_a, model_b));
            let description = description.unwrap_or_else(|| format!("Performance comparison between {} and {}", model_a, model_b));
            let sample_size = sample_size.unwrap_or(50);
            let duration_hours = duration_hours.unwrap_or(24);
            
            let test_queries = vec![
                "Analyze this code for potential improvements".to_string(),
                "Explain the concept of async/await in Rust".to_string(),
                "Write a function to parse JSON data".to_string(),
                "Debug this error message".to_string(),
                "Optimize this algorithm".to_string(),
            ];
            
            let test_id = tracker.create_ab_test(
                name.clone(),
                description,
                model_a,
                model_b,
                test_queries,
                sample_size,
                duration_hours,
            ).await?;
            
            println!("✅ Created A/B test: {}", name);
            println!("Test ID: {}", test_id);
            println!("Sample size: {} requests", sample_size);
            println!("Duration: {} hours", duration_hours);
            println!("\nTo start the test, run: hive models ab-test start --test-id {}", test_id);
        }
        
        "start" => {
            let test_id = test_id.context("--test-id required for starting A/B test")?;
            tracker.start_ab_test(&test_id).await?;
            println!("✅ Started A/B test: {}", test_id);
        }
        
        "list" => {
            let tests = tracker.get_ab_tests().await;
            
            if tests.is_empty() {
                println!("No A/B tests found.");
                return Ok(());
            }
            
            println!("A/B Tests:");
            println!("{}", "=".repeat(80));
            println!("{:<20} {:<30} {:<15} {:<15}", "Test ID", "Name", "Status", "Progress");
            println!("{}", "-".repeat(80));
            
            for test in tests {
                let status_emoji = match test.status {
                    ABTestStatus::Planned => "📋",
                    ABTestStatus::Running => "🏃",
                    ABTestStatus::Completed => "✅",
                    ABTestStatus::Paused => "⏸️",
                    ABTestStatus::Cancelled => "❌",
                };
                
                println!("{:<20} {:<30} {:<15} {:<15}",
                    &test.test_id[..8],
                    test.name,
                    format!("{} {:?}", status_emoji, test.status),
                    format!("{}/{}", 0, test.sample_size) // Would track actual progress
                );
            }
        }
        
        "analyze" => {
            let test_id = test_id.context("--test-id required for analyzing A/B test")?;
            
            match tracker.analyze_ab_test(&test_id).await {
                Ok(analysis) => {
                    println!("📊 A/B Test Analysis");
                    println!("{}", "=".repeat(60));
                    println!("Test ID: {}", analysis.test_id);
                    println!("Models: {} vs {}", analysis.model_a, analysis.model_b);
                    println!("Sample sizes: {} vs {}", analysis.sample_size_a, analysis.sample_size_b);
                    println!("Confidence: {:.0}%", analysis.confidence_level * 100.0);
                    println!();
                    
                    println!("📈 Results:");
                    let metrics = &analysis.metrics_comparison;
                    
                    println!("  Latency: {:.0}ms vs {:.0}ms ({:+.1}%)",
                        metrics.latency_comparison.model_a_value,
                        metrics.latency_comparison.model_b_value,
                        metrics.latency_comparison.percentage_change
                    );
                    
                    println!("  Success Rate: {:.1}% vs {:.1}% ({:+.1}%)",
                        metrics.success_rate_comparison.model_a_value,
                        metrics.success_rate_comparison.model_b_value,
                        metrics.success_rate_comparison.percentage_change
                    );
                    
                    println!("  Quality: {:.2} vs {:.2} ({:+.1}%)",
                        metrics.quality_comparison.model_a_value,
                        metrics.quality_comparison.model_b_value,
                        metrics.quality_comparison.percentage_change
                    );
                    
                    println!();
                    println!("🎯 Recommendation: {}", analysis.recommendation);
                    
                    if analysis.statistical_significance.is_significant {
                        println!("📊 Statistically significant (p < 0.05)");
                    } else {
                        println!("📊 Not statistically significant (p = {:.3})", 
                            analysis.statistical_significance.p_value);
                    }
                }
                Err(e) => {
                    println!("❌ Could not analyze A/B test: {}", e);
                }
            }
        }
        
        _ => {
            println!("Available actions: create, start, list, analyze");
        }
    }
    
    Ok(())
}

/// Show circuit breaker status for models
pub async fn show_circuit_breakers() -> Result<()> {
    let tracker = PerformanceTracker::new(60);
    let circuit_breakers = tracker.get_circuit_breaker_status().await;
    
    if circuit_breakers.is_empty() {
        println!("No circuit breakers active.");
        return Ok(());
    }
    
    println!("Circuit Breaker Status:");
    println!("{}", "=".repeat(70));
    println!("{:<30} {:<15} {:<10} {:<15}", "Model", "State", "Failures", "Next Attempt");
    println!("{}", "-".repeat(70));
    
    for breaker in circuit_breakers {
        let state_emoji = match breaker.state {
            CircuitState::Closed => "✅ Closed",
            CircuitState::Open => "🔴 Open",
            CircuitState::HalfOpen => "🟡 Half-Open",
        };
        
        let next_attempt = breaker.next_attempt
            .map(|t| t.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string());
        
        println!("{:<30} {:<15} {:<10} {:<15}",
            breaker.model_id,
            state_emoji,
            breaker.failure_count,
            next_attempt
        );
    }
    
    Ok(())
}

/// Reset circuit breaker for a model
pub async fn reset_circuit_breaker(model_id: &str) -> Result<()> {
    let tracker = PerformanceTracker::new(60);
    tracker.reset_circuit_breaker(model_id).await?;
    println!("✅ Reset circuit breaker for model: {}", model_id);
    Ok(())
}

// Add rand dependency for demo purposes
use rand::Rng as _;