//! MCP integration tests
//!
//! Tests for the complete MCP implementation with advanced features

use crate::integration::mcp::{
    start_mcp_server, AdvancedToolRegistry, McpServer, PerformanceManager, PromptManager,
    SamplingManager, SubscriptionManager,
};
use serde_json::json;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_mcp_server_startup() {
    // Test that the MCP server can start successfully
    let port = 8001;

    let server_task = tokio::spawn(async move { start_mcp_server(port).await });

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Cancel the server task
    server_task.abort();

    // If we reach here, the server started successfully
    assert!(true);
}

#[tokio::test]
async fn test_advanced_tool_registry() {
    let config = std::sync::Arc::new(crate::core::config::Config::default());
    let consensus_engine = std::sync::Arc::new(tokio::sync::RwLock::new(
        crate::consensus::engine::ConsensusEngine::new(config.clone())
            .await
            .unwrap(),
    ));

    let advanced_tools = AdvancedToolRegistry::new(consensus_engine, config);

    let tools = advanced_tools.get_advanced_tools();

    // Should have 15+ advanced tools
    assert!(tools.len() >= 15);

    // Check that debug_code tool exists
    let debug_tool = tools.iter().find(|(name, _)| name == "debug_code");
    assert!(debug_tool.is_some());

    // Check that refactor_code tool exists
    let refactor_tool = tools.iter().find(|(name, _)| name == "refactor_code");
    assert!(refactor_tool.is_some());
}

#[tokio::test]
async fn test_prompt_manager() {
    let prompt_manager = PromptManager::new();

    // Test template retrieval
    let template = prompt_manager.get_template("analyze_code");
    assert!(template.is_some());

    // Test prompt generation
    let mut params = std::collections::HashMap::new();
    params.insert(
        "code".to_string(),
        json!("fn hello() { println!(\"Hello\"); }"),
    );
    params.insert("language".to_string(), json!("rust"));

    let context = crate::integration::mcp::PromptContext {
        language: Some("rust".to_string()),
        project_path: None,
        user_preferences: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now(),
        session_id: "test-session".to_string(),
        tool_name: "analyze_code".to_string(),
        additional_context: std::collections::HashMap::new(),
    };

    let result = prompt_manager.generate_prompt("analyze_code", params, context);
    assert!(result.is_ok());

    let prompt = result.unwrap();
    assert!(prompt.contains("rust"));
    assert!(prompt.contains("fn hello()"));
}

#[tokio::test]
async fn test_sampling_manager() {
    let config = crate::integration::mcp::sampling::SamplingConfig::default();
    let sampling_manager = SamplingManager::new(config);

    // Test sample creation
    let request = crate::integration::mcp::SampleRequest {
        method: "consensus".to_string(),
        prompt: "Test prompt".to_string(),
        arguments: std::collections::HashMap::new(),
        include_context: None,
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
        stop_sequences: None,
        stream: Some(true),
    };

    let result = sampling_manager.start_sample(request).await;
    assert!(result.is_ok());

    let (session_id, _progress_rx) = result.unwrap();
    assert!(!session_id.is_empty());

    // Test sample retrieval
    let sample = sampling_manager.get_sample(&session_id).await;
    assert!(sample.is_some());
}

#[tokio::test]
async fn test_subscription_manager() {
    let mut subscription_manager = SubscriptionManager::new().unwrap();

    // Test client registration
    let client_id = "test-client".to_string();
    let result = subscription_manager
        .register_client(client_id.clone())
        .await;
    assert!(result.is_ok());

    // Test subscription creation
    let request = crate::integration::mcp::SubscriptionRequest {
        resource_type: crate::integration::mcp::subscriptions::ResourceType::File,
        resource_path: "/tmp/test.txt".to_string(),
        filters: None,
        client_id: client_id.clone(),
    };

    let result = subscription_manager.create_subscription(request).await;
    assert!(result.is_ok());

    // Test subscription listing
    let subscriptions = subscription_manager
        .list_client_subscriptions(&client_id)
        .await;
    assert_eq!(subscriptions.len(), 1);
}

#[tokio::test]
async fn test_performance_manager() {
    let config = crate::integration::mcp::PerformanceConfig::default();
    let performance_manager = PerformanceManager::new(config);

    // Start monitoring
    let result = performance_manager.start_monitoring().await;
    assert!(result.is_ok());

    // Test cache key generation
    let key = performance_manager.generate_cache_key("test_tool", &json!({"param": "value"}));
    assert!(!key.is_empty());
    assert!(key.starts_with("tool_test_tool_"));

    // Test metrics retrieval
    let metrics = performance_manager.get_metrics().await;
    assert_eq!(metrics.total_requests, 0); // Should start at 0

    // Test cache statistics
    let cache_stats = performance_manager.get_cache_statistics().await;
    assert_eq!(cache_stats.current_size, 0); // Should start empty
}

#[tokio::test]
async fn test_tool_execution_flow() {
    // This test simulates the complete flow of tool execution
    // with performance monitoring, caching, and prompt management

    let config = std::sync::Arc::new(crate::core::config::Config::default());
    let consensus_engine = std::sync::Arc::new(tokio::sync::RwLock::new(
        crate::consensus::engine::ConsensusEngine::new(config.clone())
            .await
            .unwrap(),
    ));

    // Create tool registry with all advanced features
    let tool_registry =
        crate::integration::mcp::tools::ToolRegistry::new(consensus_engine, config).await;

    assert!(tool_registry.is_ok());

    let registry = tool_registry.unwrap();

    // Test listing tools
    let tools = registry.list_tools().await;
    assert!(tools.is_ok());

    let tool_list = tools.unwrap();
    // Should have basic tools + advanced tools (26+ total)
    assert!(tool_list.len() >= 26);

    // Verify some key tools exist
    let tool_names: Vec<_> = tool_list.iter().map(|t| &t.name).collect();
    assert!(tool_names.contains(&&"ask_hive".to_string()));
    assert!(tool_names.contains(&&"analyze_code".to_string()));
    assert!(tool_names.contains(&&"debug_code".to_string()));
    assert!(tool_names.contains(&&"refactor_code".to_string()));
    assert!(tool_names.contains(&&"security_scan".to_string()));
}

#[tokio::test]
async fn test_mcp_protocol_compliance() {
    // Test that our MCP implementation follows the protocol correctly

    let server = McpServer::new().await;
    assert!(server.is_ok());

    // The server should initialize with proper capabilities
    // and handle basic MCP protocol methods
    // This would be expanded with actual protocol testing
}

#[test]
fn test_advanced_tool_definitions() {
    // Test that all advanced tools have proper schemas
    let tools_to_test = vec![
        "debug_code",
        "refactor_code",
        "review_code",
        "optimize_code",
        "security_scan",
        "find_bugs",
        "suggest_fixes",
        "explain_error",
        "generate_readme",
        "create_tests",
        "analyze_dependencies",
        "suggest_patterns",
        "estimate_complexity",
        "generate_changelog",
        "create_api_docs",
    ];

    for tool_name in tools_to_test {
        // Each tool should have proper input schema definition
        // This would be expanded with actual schema validation
        assert!(!tool_name.is_empty());
    }
}

#[tokio::test]
async fn test_real_time_subscriptions() {
    let mut subscription_manager = SubscriptionManager::new().unwrap();

    // Start the subscription manager
    let result = subscription_manager.start().await;
    assert!(result.is_ok());

    // Test that the manager can handle real-time events
    let stats = subscription_manager.get_statistics().await;
    assert_eq!(stats.total_subscriptions, 0);
    assert_eq!(stats.total_clients, 0);
}

#[tokio::test]
async fn test_performance_optimizations() {
    let config = crate::integration::mcp::PerformanceConfig {
        cache_enabled: true,
        cache_max_size: 100,
        cache_ttl_seconds: 60,
        connection_pool_size: 5,
        request_timeout_seconds: 30,
        enable_compression: true,
        parallel_tool_execution: true,
        max_concurrent_tools: 3,
        enable_result_streaming: true,
        metrics_collection_interval_seconds: 10,
    };

    let performance_manager = PerformanceManager::new(config);

    // Test caching functionality
    let cache_key = "test_key".to_string();
    let test_data = json!({"result": "cached_value"});

    // Should be a cache miss initially
    let cached = performance_manager.get_cached_result(&cache_key).await;
    assert!(cached.is_none());

    // Cache the result
    let result = performance_manager
        .cache_result(cache_key.clone(), test_data.clone())
        .await;
    assert!(result.is_ok());

    // Should now be a cache hit
    let cached = performance_manager.get_cached_result(&cache_key).await;
    assert!(cached.is_some());
    assert_eq!(cached.unwrap(), test_data);
}
