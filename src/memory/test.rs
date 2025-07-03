//! Simple test for memory module

#[cfg(test)]
mod tests {
    use super::super::*;
    
    #[tokio::test]
    async fn test_memory_module_compiles() {
        // Just test that the module compiles
        let _ = MemoryIntelligence::new().await;
        
        // Test embedding engine
        let _ = EmbeddingEngine::new().await;
        
        // Test vector store
        let _ = VectorStore::new();
        
        // Test knowledge graph
        let _ = KnowledgeGraph::new();
        
        // Test pattern learner
        let _ = PatternLearner::new();
        
        // Test memory analyzer
        let _ = MemoryAnalyzer::new();
    }
}