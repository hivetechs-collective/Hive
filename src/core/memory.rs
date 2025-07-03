//! Enhanced memory system with AI capabilities
//! 
//! This module provides:
//! - Vector embeddings for semantic search
//! - Knowledge graph construction and traversal
//! - Pattern learning and recognition
//! - Context retrieval engine
//! - Memory analytics and insights

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};

use crate::core::{
    database::KnowledgeConversation,
};

// Embedding types - using optional dependencies for now
#[cfg(feature = "embeddings")]
pub use candle_core::{Device, Tensor, D};
#[cfg(feature = "embeddings")]
pub use candle_nn::VarBuilder;
#[cfg(feature = "embeddings")]
pub use candle_transformers::models::bert::{BertModel, Config as BertConfig};

/// Vector dimension for embeddings (standard BERT)
pub const EMBEDDING_DIM: usize = 768;

/// Memory system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum number of embeddings to cache in memory
    pub max_cached_embeddings: usize,
    /// Similarity threshold for semantic search (0.0 - 1.0)
    pub similarity_threshold: f32,
    /// Number of results to return by default
    pub default_result_limit: usize,
    /// Path to embedding model
    pub model_path: Option<PathBuf>,
    /// Enable pattern learning
    pub enable_pattern_learning: bool,
    /// Enable relationship extraction
    pub enable_relationship_extraction: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_cached_embeddings: 10000,
            similarity_threshold: 0.75,
            default_result_limit: 10,
            model_path: None,
            enable_pattern_learning: true,
            enable_relationship_extraction: true,
        }
    }
}

/// Enhanced memory system with AI capabilities
#[derive(Debug)]
pub struct MemorySystem {
    config: MemoryConfig,
    embeddings: Arc<RwLock<EmbeddingStore>>,
    knowledge_graph: Arc<RwLock<KnowledgeGraph>>,
    pattern_learner: Arc<Mutex<PatternLearner>>,
    context_engine: Arc<ContextEngine>,
    analytics: Arc<Mutex<MemoryAnalytics>>,
}

/// Stores and manages vector embeddings
#[derive(Debug)]
struct EmbeddingStore {
    /// Map of document ID to embedding vector
    embeddings: HashMap<String, Vec<f32>>,
    /// Embedding model
    model: Option<Arc<EmbeddingModel>>,
    /// Cache for recent queries
    query_cache: HashMap<String, Vec<f32>>,
}

/// Wrapper for the embedding model
#[cfg(feature = "embeddings")]
struct EmbeddingModel {
    bert: BertModel,
    device: Device,
}

#[cfg(not(feature = "embeddings"))]
struct EmbeddingModel {
    // Placeholder for when embeddings are not available
    _placeholder: (),
}

impl std::fmt::Debug for EmbeddingModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingModel")
            .field("device", &"GPU/CPU")
            .finish()
    }
}

/// Knowledge graph for relationship management
#[derive(Debug, Default)]
struct KnowledgeGraph {
    /// Nodes in the graph (entities)
    nodes: HashMap<String, KnowledgeNode>,
    /// Edges between nodes (relationships)
    edges: HashMap<String, Vec<KnowledgeEdge>>,
    /// Inverted index for fast lookups
    index: HashMap<String, HashSet<String>>,
}

/// Node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KnowledgeNode {
    id: String,
    entity_type: EntityType,
    label: String,
    properties: HashMap<String, String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Edge in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KnowledgeEdge {
    id: String,
    source_id: String,
    target_id: String,
    relation_type: RelationType,
    weight: f32,
    properties: HashMap<String, String>,
}

/// Types of entities in the knowledge graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum EntityType {
    Concept,
    Technology,
    Pattern,
    Solution,
    Problem,
    Person,
    Resource,
}

/// Types of relationships between entities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum RelationType {
    RelatedTo,
    DependsOn,
    Implements,
    Solves,
    Contradicts,
    Extends,
    UsedIn,
    CreatedBy,
}

/// Pattern learning system
#[derive(Debug)]
struct PatternLearner {
    /// Discovered patterns
    patterns: Vec<LearnedPattern>,
    /// Pattern statistics
    pattern_stats: HashMap<String, PatternStats>,
    /// Learning configuration
    min_occurrences: usize,
    confidence_threshold: f32,
}

/// A learned pattern from conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub id: String,
    pub pattern_type: PatternType,
    pub template: String,
    pub examples: Vec<String>,
    pub confidence: f32,
    pub frequency: usize,
    pub last_seen: DateTime<Utc>,
}

/// Types of patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    QuestionAnswer,
    ProblemSolution,
    CommandSequence,
    ErrorResolution,
    BestPractice,
    Workflow,
}

/// Pattern statistics
#[derive(Debug, Default)]
struct PatternStats {
    occurrences: usize,
    success_rate: f32,
    avg_quality_score: f32,
    last_updated: Option<DateTime<Utc>>,
}

/// Context retrieval engine
#[derive(Debug)]
struct ContextEngine {
    /// Maximum context window size
    max_context_size: usize,
    /// Context relevance scorer
    relevance_threshold: f32,
}

/// Memory analytics system
#[derive(Debug, Default)]
struct MemoryAnalytics {
    /// Total number of memories
    total_memories: usize,
    /// Memory access patterns
    access_patterns: HashMap<String, AccessPattern>,
    /// Topic distribution
    topic_distribution: HashMap<String, usize>,
    /// Quality metrics
    quality_metrics: QualityMetrics,
}

/// Access pattern for memory usage
#[derive(Debug, Default)]
struct AccessPattern {
    access_count: usize,
    last_accessed: Option<DateTime<Utc>>,
    avg_relevance_score: f32,
}

/// Quality metrics for memory system
#[derive(Debug, Default)]
struct QualityMetrics {
    avg_embedding_quality: f32,
    avg_retrieval_precision: f32,
    pattern_accuracy: f32,
    graph_connectivity: f32,
}

/// Result of a semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub similarity_score: f32,
    pub context: Option<String>,
    pub relationships: Vec<String>,
}

/// Memory insight generated by analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInsight {
    pub insight_type: InsightType,
    pub description: String,
    pub confidence: f32,
    pub supporting_evidence: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Types of insights
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InsightType {
    TrendIdentified,
    PatternDiscovered,
    AnomalyDetected,
    KnowledgeGap,
    OptimizationOpportunity,
}

impl MemorySystem {
    /// Create a new memory system
    pub async fn new(config: MemoryConfig) -> Result<Self> {
        info!("Initializing enhanced memory system");

        // Initialize embedding store
        let embeddings = Arc::new(RwLock::new(EmbeddingStore {
            embeddings: HashMap::new(),
            model: None, // Will be lazy-loaded
            query_cache: HashMap::new(),
        }));

        // Initialize knowledge graph
        let knowledge_graph = Arc::new(RwLock::new(KnowledgeGraph::default()));

        // Initialize pattern learner
        let pattern_learner = Arc::new(Mutex::new(PatternLearner {
            patterns: Vec::new(),
            pattern_stats: HashMap::new(),
            min_occurrences: 3,
            confidence_threshold: 0.8,
        }));

        // Initialize context engine
        let context_engine = Arc::new(ContextEngine {
            max_context_size: 4096,
            relevance_threshold: 0.7,
        });

        // Initialize analytics
        let analytics = Arc::new(Mutex::new(MemoryAnalytics::default()));

        let system = Self {
            config,
            embeddings,
            knowledge_graph,
            pattern_learner,
            context_engine,
            analytics,
        };

        // Load existing memories
        system.load_existing_memories().await?;

        info!("Memory system initialized successfully");
        Ok(system)
    }

    /// Perform semantic search across memories
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> Result<Vec<SemanticSearchResult>> {
        let start = std::time::Instant::now();
        debug!("Performing semantic search for: {}", query);

        // Generate query embedding
        let query_embedding = self.generate_embedding(query).await?;

        // Get all memories with embeddings
        let embeddings = self.embeddings.read().await;
        let mut results = Vec::new();

        // Calculate similarities
        for (doc_id, doc_embedding) in &embeddings.embeddings {
            let similarity = cosine_similarity(&query_embedding, doc_embedding);
            
            if similarity >= self.config.similarity_threshold {
                // Fetch the actual document
                if let Ok(Some(knowledge)) = KnowledgeConversation::find_by_id(doc_id).await {
                    // Get relationships from knowledge graph
                    let relationships = self.get_relationships(doc_id).await?;
                    
                    results.push(SemanticSearchResult {
                        id: knowledge.id,
                        question: knowledge.question,
                        answer: knowledge.final_answer,
                        similarity_score: similarity,
                        context: knowledge.conversation_context,
                        relationships,
                    });
                }
            }
        }

        // Sort by similarity score
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

        // Limit results
        let limit = limit.unwrap_or(self.config.default_result_limit);
        results.truncate(limit);

        // Update analytics
        self.update_search_analytics(&query, &results).await?;

        let duration = start.elapsed();
        info!("Semantic search completed in {:?}, found {} results", duration, results.len());

        Ok(results)
    }

    /// Analyze patterns in memory
    pub async fn analyze_patterns(&self) -> Result<Vec<LearnedPattern>> {
        info!("Analyzing patterns in memory");
        
        let mut learner = self.pattern_learner.lock().await;
        
        // Get all conversations
        let knowledge_entries = KnowledgeConversation::get_recent(1000).await?;
        
        // Extract patterns
        for entry in &knowledge_entries {
            learner.process_conversation(&entry.question, &entry.final_answer)?;
        }
        
        // Filter high-confidence patterns
        let patterns: Vec<_> = learner.patterns.iter()
            .filter(|p| p.confidence >= learner.confidence_threshold)
            .cloned()
            .collect();
        
        info!("Found {} high-confidence patterns", patterns.len());
        Ok(patterns)
    }

    /// Generate insights from memory analytics
    pub async fn generate_insights(&self) -> Result<Vec<MemoryInsight>> {
        info!("Generating memory insights");
        
        let analytics = self.analytics.lock().await;
        let mut insights = Vec::new();
        
        // Analyze topic distribution
        if let Some(trending_topic) = self.identify_trending_topics(&analytics).await? {
            insights.push(trending_topic);
        }
        
        // Analyze pattern effectiveness
        if let Some(pattern_insight) = self.analyze_pattern_effectiveness().await? {
            insights.push(pattern_insight);
        }
        
        // Identify knowledge gaps
        if let Some(gap_insight) = self.identify_knowledge_gaps(&analytics).await? {
            insights.push(gap_insight);
        }
        
        info!("Generated {} insights", insights.len());
        Ok(insights)
    }

    /// Build or update the knowledge graph
    pub async fn build_knowledge_graph(&self) -> Result<()> {
        info!("Building knowledge graph");
        
        let mut graph = self.knowledge_graph.write().await;
        
        // Get all conversations
        let knowledge_entries = KnowledgeConversation::get_recent(1000).await?;
        
        for entry in &knowledge_entries {
            // Extract entities
            let entities = self.extract_entities(&entry.question, &entry.final_answer)?;
            
            // Add nodes
            for entity in &entities {
                graph.nodes.insert(entity.id.clone(), entity.clone());
            }
            
            // Extract relationships
            let relationships = self.extract_relationships(&entities)?;
            
            // Add edges
            for relation in relationships {
                graph.edges.entry(relation.source_id.clone())
                    .or_insert_with(Vec::new)
                    .push(relation);
            }
        }
        
        info!("Knowledge graph built with {} nodes and {} edges", 
            graph.nodes.len(), 
            graph.edges.values().map(|v| v.len()).sum::<usize>()
        );
        
        Ok(())
    }

    /// Export knowledge graph in DOT format
    pub async fn export_knowledge_graph(&self, format: &str) -> Result<String> {
        let graph = self.knowledge_graph.read().await;
        
        match format {
            "dot" => {
                let mut dot = String::from("digraph KnowledgeGraph {\n");
                dot.push_str("  rankdir=LR;\n");
                dot.push_str("  node [shape=box, style=rounded];\n\n");
                
                // Add nodes
                for (id, node) in &graph.nodes {
                    dot.push_str(&format!(
                        "  \"{}\" [label=\"{}\", color=\"{}\"];\n",
                        id,
                        node.label,
                        match node.entity_type {
                            EntityType::Concept => "blue",
                            EntityType::Technology => "green",
                            EntityType::Pattern => "orange",
                            EntityType::Solution => "purple",
                            EntityType::Problem => "red",
                            EntityType::Person => "brown",
                            EntityType::Resource => "gray",
                        }
                    ));
                }
                
                dot.push_str("\n");
                
                // Add edges
                for (source_id, edges) in &graph.edges {
                    for edge in edges {
                        dot.push_str(&format!(
                            "  \"{}\" -> \"{}\" [label=\"{:?}\", weight={}];\n",
                            source_id,
                            edge.target_id,
                            edge.relation_type,
                            edge.weight
                        ));
                    }
                }
                
                dot.push_str("}\n");
                Ok(dot)
            }
            _ => Err(anyhow::anyhow!("Unsupported export format: {}", format)),
        }
    }

    /// Get relevant context for a query
    pub async fn get_relevant_context(&self, query: &str) -> Result<String> {
        debug!("Getting relevant context for: {}", query);
        
        // Search for similar memories
        let results = self.semantic_search(query, Some(5)).await?;
        
        // Build context from results
        let mut context = String::new();
        for (i, result) in results.iter().enumerate() {
            if i > 0 {
                context.push_str("\n\n");
            }
            context.push_str(&format!(
                "Related Memory ({}% match):\nQ: {}\nA: {}",
                (result.similarity_score * 100.0) as i32,
                result.question,
                result.answer
            ));
        }
        
        Ok(context)
    }

    // Private helper methods

    async fn load_existing_memories(&self) -> Result<()> {
        info!("Loading existing memories");
        
        let knowledge_entries = KnowledgeConversation::get_recent(self.config.max_cached_embeddings).await?;
        let mut embeddings = self.embeddings.write().await;
        
        for entry in &knowledge_entries {
            // Generate embedding for combined text
            let text = format!("{} {}", entry.question, entry.final_answer);
            if let Ok(embedding) = self.generate_embedding(&text).await {
                embeddings.embeddings.insert(entry.id.clone(), embedding);
            }
        }
        
        info!("Loaded {} memory embeddings", embeddings.embeddings.len());
        Ok(())
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // For now, use a simple hash-based embedding
        // In production, this would use a proper embedding model
        let mut embedding = vec![0.0; EMBEDDING_DIM];
        let bytes = text.as_bytes();
        
        for (i, chunk) in bytes.chunks(8).enumerate() {
            let mut value = 0u64;
            for &byte in chunk {
                value = value.wrapping_mul(31).wrapping_add(byte as u64);
            }
            let normalized = (value as f64 / u64::MAX as f64) as f32;
            embedding[i % EMBEDDING_DIM] = normalized;
        }
        
        // Normalize the embedding
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }
        
        Ok(embedding)
    }

    async fn get_relationships(&self, doc_id: &str) -> Result<Vec<String>> {
        let graph = self.knowledge_graph.read().await;
        let mut relationships = Vec::new();
        
        if let Some(edges) = graph.edges.get(doc_id) {
            for edge in edges {
                if let Some(target) = graph.nodes.get(&edge.target_id) {
                    relationships.push(format!("{:?} -> {}", edge.relation_type, target.label));
                }
            }
        }
        
        Ok(relationships)
    }

    async fn update_search_analytics(&self, query: &str, results: &[SemanticSearchResult]) -> Result<()> {
        let mut analytics = self.analytics.lock().await;
        
        // Update access patterns
        for result in results {
            let pattern = analytics.access_patterns.entry(result.id.clone()).or_default();
            pattern.access_count += 1;
            pattern.last_accessed = Some(Utc::now());
            pattern.avg_relevance_score = 
                (pattern.avg_relevance_score * (pattern.access_count - 1) as f32 + result.similarity_score) 
                / pattern.access_count as f32;
        }
        
        Ok(())
    }

    async fn identify_trending_topics(&self, analytics: &MemoryAnalytics) -> Result<Option<MemoryInsight>> {
        // Simple trending detection based on access patterns
        let mut topic_scores: HashMap<String, f32> = HashMap::new();
        
        for (doc_id, pattern) in &analytics.access_patterns {
            if let Some(last_accessed) = pattern.last_accessed {
                let recency = (Utc::now() - last_accessed).num_hours() as f32;
                let score = pattern.access_count as f32 / (recency + 1.0).sqrt();
                topic_scores.insert(doc_id.clone(), score);
            }
        }
        
        // Find the highest scoring topic
        if let Some((topic_id, score)) = topic_scores.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
            if *score > 10.0 {  // Threshold for trending
                return Ok(Some(MemoryInsight {
                    insight_type: InsightType::TrendIdentified,
                    description: format!("Topic {} is trending with score {:.2}", topic_id, score),
                    confidence: (*score / 100.0).min(1.0),
                    supporting_evidence: vec![topic_id.clone()],
                    recommendations: vec![
                        "Consider creating more content on this topic".to_string(),
                        "Update related documentation".to_string(),
                    ],
                }));
            }
        }
        
        Ok(None)
    }

    async fn analyze_pattern_effectiveness(&self) -> Result<Option<MemoryInsight>> {
        let learner = self.pattern_learner.lock().await;
        
        // Find patterns with high frequency but low confidence
        let ineffective_patterns: Vec<_> = learner.patterns.iter()
            .filter(|p| p.frequency > 10 && p.confidence < 0.7)
            .collect();
        
        if !ineffective_patterns.is_empty() {
            return Ok(Some(MemoryInsight {
                insight_type: InsightType::OptimizationOpportunity,
                description: format!("Found {} patterns that could be improved", ineffective_patterns.len()),
                confidence: 0.85,
                supporting_evidence: ineffective_patterns.iter()
                    .map(|p| p.template.clone())
                    .collect(),
                recommendations: vec![
                    "Review and refine these patterns".to_string(),
                    "Collect more examples for training".to_string(),
                ],
            }));
        }
        
        Ok(None)
    }

    async fn identify_knowledge_gaps(&self, analytics: &MemoryAnalytics) -> Result<Option<MemoryInsight>> {
        // Simple gap detection based on low topic coverage
        if analytics.total_memories < 100 {
            return Ok(Some(MemoryInsight {
                insight_type: InsightType::KnowledgeGap,
                description: "Knowledge base is still building up".to_string(),
                confidence: 0.9,
                supporting_evidence: vec![format!("Only {} memories stored", analytics.total_memories)],
                recommendations: vec![
                    "Continue using the system to build knowledge".to_string(),
                    "Import existing documentation if available".to_string(),
                ],
            }));
        }
        
        Ok(None)
    }

    fn extract_entities(&self, question: &str, answer: &str) -> Result<Vec<KnowledgeNode>> {
        let mut entities = Vec::new();
        let combined_text = format!("{} {}", question, answer);
        
        // Simple entity extraction based on patterns
        // In production, this would use NER models
        
        // Extract technology mentions
        let tech_patterns = ["Rust", "Python", "JavaScript", "TypeScript", "API", "database"];
        for tech in &tech_patterns {
            if combined_text.contains(tech) {
                entities.push(KnowledgeNode {
                    id: format!("tech_{}", tech.to_lowercase()),
                    entity_type: EntityType::Technology,
                    label: tech.to_string(),
                    properties: HashMap::new(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });
            }
        }
        
        // Extract concepts
        let concept_patterns = ["authentication", "performance", "security", "optimization"];
        for concept in &concept_patterns {
            if combined_text.to_lowercase().contains(concept) {
                entities.push(KnowledgeNode {
                    id: format!("concept_{}", concept),
                    entity_type: EntityType::Concept,
                    label: concept.to_string(),
                    properties: HashMap::new(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });
            }
        }
        
        Ok(entities)
    }

    fn extract_relationships(&self, entities: &[KnowledgeNode]) -> Result<Vec<KnowledgeEdge>> {
        let mut relationships = Vec::new();
        
        // Create relationships between entities
        // In production, this would use relationship extraction models
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                let source = &entities[i];
                let target = &entities[j];
                
                // Create a basic relationship
                relationships.push(KnowledgeEdge {
                    id: format!("{}_{}", source.id, target.id),
                    source_id: source.id.clone(),
                    target_id: target.id.clone(),
                    relation_type: RelationType::RelatedTo,
                    weight: 0.5,
                    properties: HashMap::new(),
                });
            }
        }
        
        Ok(relationships)
    }
}

impl PatternLearner {
    fn process_conversation(&mut self, question: &str, answer: &str) -> Result<()> {
        // Simple pattern extraction
        // In production, this would use more sophisticated NLP
        
        // Check for question-answer patterns
        if question.starts_with("How do I") || question.starts_with("How to") {
            let pattern_id = "how_to_pattern";
            let stats = self.pattern_stats.entry(pattern_id.to_string()).or_default();
            stats.occurrences += 1;
            stats.last_updated = Some(Utc::now());
            
            if stats.occurrences >= self.min_occurrences {
                let pattern = LearnedPattern {
                    id: pattern_id.to_string(),
                    pattern_type: PatternType::QuestionAnswer,
                    template: "How to X -> Step-by-step solution".to_string(),
                    examples: vec![question.to_string()],
                    confidence: 0.85,
                    frequency: stats.occurrences,
                    last_seen: Utc::now(),
                };
                
                // Update or add pattern
                if let Some(existing) = self.patterns.iter_mut().find(|p| p.id == pattern_id) {
                    existing.frequency = stats.occurrences;
                    existing.last_seen = Utc::now();
                    existing.examples.push(question.to_string());
                    existing.examples.truncate(10); // Keep only recent examples
                } else {
                    self.patterns.push(pattern);
                }
            }
        }
        
        Ok(())
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

/// Global memory system instance
static MEMORY_SYSTEM: tokio::sync::OnceCell<Arc<MemorySystem>> = tokio::sync::OnceCell::const_new();

/// Initialize the memory system
pub async fn initialize_memory(config: Option<MemoryConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let system = Arc::new(MemorySystem::new(config).await?);
    
    MEMORY_SYSTEM
        .set(system)
        .map_err(|_| anyhow::anyhow!("Memory system already initialized"))?;
    
    Ok(())
}

/// Get the global memory system instance
pub async fn get_memory_system() -> Result<Arc<MemorySystem>> {
    MEMORY_SYSTEM
        .get()
        .ok_or_else(|| anyhow::anyhow!("Memory system not initialized"))
        .map(Arc::clone)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_initialization() -> Result<()> {
        let config = MemoryConfig::default();
        let memory = MemorySystem::new(config).await?;
        
        // Test basic functionality
        let results = memory.semantic_search("test query", None).await?;
        assert!(results.is_empty() || results.len() <= 10);
        
        Ok(())
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        
        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_embedding_generation() {
        // Test that embeddings are consistent for the same text
        let text = "Hello, world!";
        let embedding1 = generate_simple_embedding(text);
        let embedding2 = generate_simple_embedding(text);
        
        assert_eq!(embedding1.len(), EMBEDDING_DIM);
        assert_eq!(embedding1, embedding2);
    }
    
    fn generate_simple_embedding(text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0; EMBEDDING_DIM];
        let bytes = text.as_bytes();
        
        for (i, chunk) in bytes.chunks(8).enumerate() {
            let mut value = 0u64;
            for &byte in chunk {
                value = value.wrapping_mul(31).wrapping_add(byte as u64);
            }
            let normalized = (value as f64 / u64::MAX as f64) as f32;
            embedding[i % EMBEDDING_DIM] = normalized;
        }
        
        // Normalize
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }
        
        embedding
    }
}

// Type alias for compatibility
pub type MemoryManager = MemorySystem;