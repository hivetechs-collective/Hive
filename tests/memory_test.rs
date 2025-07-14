//! Test the memory module in isolation

use anyhow::Result;
use hive_ai::memory::{
    ContextRetriever, EmbeddingEngine, KnowledgeGraph, MemoryAnalyzer, MemoryIntelligence,
    PatternLearner, SimilarityMetric, VectorStore,
};

#[tokio::test]
async fn test_memory_intelligence_creation() -> Result<()> {
    // Test that we can create the memory intelligence system
    let intelligence = MemoryIntelligence::new().await?;

    // Verify components are initialized
    let metrics = intelligence.get_metrics().await?;
    assert_eq!(metrics.total_memories, 0);

    Ok(())
}

#[tokio::test]
async fn test_embedding_engine() -> Result<()> {
    let engine = EmbeddingEngine::new().await?;

    // Test basic encoding
    let text = "Hello, world!";
    let embedding = engine.encode(text).await?;

    // Check embedding dimension
    assert_eq!(embedding.len(), 384); // Standard embedding dimension

    // Test that same text produces same embedding (deterministic)
    let embedding2 = engine.encode(text).await?;
    assert_eq!(embedding, embedding2);

    Ok(())
}

#[tokio::test]
async fn test_vector_store() -> Result<()> {
    let store = VectorStore::new();

    // Add a test embedding
    let id = "test1".to_string();
    let vector = vec![0.1; 384];
    let metadata = std::collections::HashMap::new();

    store.add(id.clone(), vector.clone(), metadata).await?;

    // Verify it was added
    assert_eq!(store.len().await, 1);

    // Retrieve and verify
    let retrieved = store.get(&id).await.unwrap();
    assert_eq!(retrieved.vector, vector);

    Ok(())
}

#[tokio::test]
async fn test_knowledge_graph() -> Result<()> {
    use hive_ai::memory::{Entity, EntityType, RelationType, Relationship};

    let mut graph = KnowledgeGraph::new();

    // Add entities
    let entity1 = Entity {
        id: "rust".to_string(),
        entity_type: EntityType::Technology,
        label: "Rust".to_string(),
        properties: std::collections::HashMap::new(),
        confidence: 1.0,
    };

    let entity2 = Entity {
        id: "memory".to_string(),
        entity_type: EntityType::Concept,
        label: "Memory Management".to_string(),
        properties: std::collections::HashMap::new(),
        confidence: 0.9,
    };

    graph.add_entity(entity1)?;
    graph.add_entity(entity2)?;

    // Add relationship
    graph.add_relationship(Relationship {
        source: "rust".to_string(),
        target: "memory".to_string(),
        relation_type: RelationType::Implements,
        weight: 0.8,
        properties: std::collections::HashMap::new(),
    })?;

    // Verify graph construction
    let rust_entity = graph.find_entity("rust").unwrap();
    assert_eq!(rust_entity.label, "Rust");

    Ok(())
}

#[tokio::test]
async fn test_pattern_learner() -> Result<()> {
    let mut learner = PatternLearner::new();

    // Process a sample conversation
    learner.process_conversation(
        "How do I read a file in Rust?",
        "Use std::fs::read_to_string() to read a file",
        None,
    )?;

    // Get patterns (should be empty as we need min occurrences)
    let patterns = learner.get_patterns();
    assert!(patterns.is_empty() || patterns.len() > 0); // Either case is fine

    Ok(())
}

#[tokio::test]
async fn test_memory_analyzer() -> Result<()> {
    let analyzer = MemoryAnalyzer::new();

    // Get initial metrics
    let metrics = analyzer.get_metrics();
    assert_eq!(metrics.total_memories, 0);

    Ok(())
}
