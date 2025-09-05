/// Example demonstrating OpenRouter integration
///
/// Run with: cargo run --example openrouter_test
use crate::providers::openrouter::{
    create_client, CostCalculator, ModelSelectionStrategy, ModelSelector, PerformanceTracker,
    TaskComplexity,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("OpenRouter Integration Test\n");

    // Test model selection
    println!("1. Testing Model Selection");
    let selector = ModelSelector::new(ModelSelectionStrategy::Balanced);

    let selection = selector.select_model(
        "Analyze this Rust code for performance issues",
        TaskComplexity::Complex,
        vec![],
        None,
    )?;

    println!("   Selected model: {}", selection.primary);
    println!("   Fallback models: {:?}", selection.fallbacks);
    println!("   Estimated cost: ${:.4}", selection.estimated_cost);
    println!("   Reasoning: {}", selection.reasoning);
    println!();

    // Test cost calculation
    println!("2. Testing Cost Calculation");
    let calculator = CostCalculator::new();

    let cost = calculator.calculate_cost("openai/gpt-4", 1000, 500)?;
    println!("   Model: {}", cost.model_id);
    println!("   Input cost: ${:.4}", cost.input_cost);
    println!("   Output cost: ${:.4}", cost.output_cost);
    println!("   Total cost: ${:.4}", cost.total_cost);
    println!();

    // Test performance tracking
    println!("3. Testing Performance Tracking");
    let tracker = PerformanceTracker::new(5);

    // Simulate some performance data
    tracker
        .track_performance(
            "openai/gpt-4",
            1500,
            200,
            true,
            None,
            Some(0.9),
            "test",
            None,
        )
        .await?;

    if let Some(metrics) = tracker.get_metrics("openai/gpt-4").await {
        println!("   Total requests: {}", metrics.total_requests);
        println!("   Success rate: {:.1}%", metrics.success_rate * 100.0);
        println!("   Average latency: {:.0}ms", metrics.average_latency_ms);
        println!("   Quality score: {:.2}", metrics.quality_score);
    }
    println!();

    // Test API key validation (no actual API call)
    println!("4. Testing API Client Creation");
    match create_client("sk-or-test-key".to_string()) {
        Ok(_) => println!("   ✓ Valid API key format accepted"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    match create_client("invalid-key".to_string()) {
        Ok(_) => println!("   ✗ Invalid key should have been rejected"),
        Err(_) => println!("   ✓ Invalid API key format correctly rejected"),
    }
    println!();

    // List available models
    println!("5. Available Models by Tier");
    let models = selector.list_models();
    let mut by_tier = std::collections::HashMap::new();

    for model in models.iter().take(10) {
        by_tier
            .entry(format!("{:?}", model.tier))
            .or_insert_with(Vec::new)
            .push(model.name.clone());
    }

    for (tier, names) in by_tier {
        println!("   {}: {}", tier, names.join(", "));
    }

    println!("\nOpenRouter integration test completed successfully!");
    Ok(())
}
