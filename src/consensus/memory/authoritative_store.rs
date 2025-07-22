//! Authoritative Knowledge Store - The source of truth for all consensus knowledge
//! 
//! This module stores curator-validated facts with semantic fingerprinting for deduplication
//! and provides efficient retrieval mechanisms for context injection.

use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::consensus::memory::semantic_fingerprint::{SemanticFingerprint, SemanticFingerprinter};
use crate::consensus::types::StageResult;
use crate::core::db_actor::DatabaseService;

/// Unique identifier for a fact
pub type FactId = String;

/// Unique identifier for a topic
pub type TopicId = String;

/// Stores curator-validated facts with semantic indexing
pub struct AuthoritativeKnowledgeStore {
    /// Database service for Send-safe operations
    db_service: DatabaseService,
    
    /// In-memory cache of recent facts
    fact_cache: Arc<RwLock<HashMap<FactId, CuratedFact>>>,
    
    /// Temporal index for time-based retrieval
    temporal_index: Arc<RwLock<BTreeMap<DateTime<Utc>, Vec<FactId>>>>,
    
    /// Topic-based clustering
    topic_clusters: Arc<RwLock<HashMap<TopicId, Vec<FactId>>>>,
    
    /// Semantic fingerprinter
    fingerprinter: Arc<SemanticFingerprinter>,
}

/// A curator-validated fact with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratedFact {
    pub id: FactId,
    pub content: String,
    pub semantic_fingerprint: String, // Store as string for DB
    pub curator_confidence: f64,
    pub source_question: String,
    pub source_conversation_id: Option<String>,
    pub consensus_stages: Vec<StageContribution>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
    pub related_facts: Vec<FactId>,
    pub topics: Vec<TopicId>,
    pub entities: Vec<Entity>,
    pub metadata: FactMetadata,
}

/// Contribution from a consensus stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageContribution {
    pub stage: String,
    pub model: String,
    pub contribution: String,
    pub confidence: f64,
}

/// Entity mentioned in a fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: String,
    pub confidence: f64,
}

/// Additional metadata for a fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactMetadata {
    pub fact_type: FactType,
    pub domain: Option<String>,
    pub language: String,
    pub complexity_score: f64,
    pub verification_status: VerificationStatus,
}

/// Type of fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactType {
    Definition,
    Explanation,
    CodeExample,
    Procedure,
    Relationship,
    Insight,
    Other(String),
}

/// Verification status of a fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Contradicted,
    Updated,
}

impl Default for FactMetadata {
    fn default() -> Self {
        Self {
            fact_type: FactType::Other("general".to_string()),
            domain: None,
            language: "en".to_string(),
            complexity_score: 0.5,
            verification_status: VerificationStatus::Unverified,
        }
    }
}

impl AuthoritativeKnowledgeStore {
    /// Create a new knowledge store
    pub async fn new(
        db_service: DatabaseService,
        fingerprinter: Arc<SemanticFingerprinter>,
    ) -> Result<Self> {
        // Note: Table initialization is now handled by DatabaseManager
        
        // Load recent facts into cache
        let fact_cache = Arc::new(RwLock::new(HashMap::new()));
        let temporal_index = Arc::new(RwLock::new(BTreeMap::new()));
        let topic_clusters = Arc::new(RwLock::new(HashMap::new()));
        
        let store = Self {
            db_service,
            fact_cache,
            temporal_index,
            topic_clusters,
            fingerprinter,
        };
        
        // Load recent facts
        store.load_recent_facts().await?;
        
        Ok(store)
    }
    
    /// Store a new fact
    pub async fn store_fact(&self, fact: CuratedFact) -> Result<()> {
        // Check for duplicates using semantic fingerprint
        if self.fact_exists(&fact.semantic_fingerprint).await? {
            tracing::info!("Fact already exists with fingerprint: {}", fact.semantic_fingerprint);
            return Ok(());
        }
        
        // Store in database through service
        self.db_service.store_fact(fact.clone()).await?;
        
        // Update caches
        self.update_caches(&fact).await?;
        
        Ok(())
    }
    
    /// Check if a fact already exists
    async fn fact_exists(&self, fingerprint: &str) -> Result<bool> {
        self.db_service.check_fact_exists(fingerprint).await
    }
    
    /// Update in-memory caches
    async fn update_caches(&self, fact: &CuratedFact) -> Result<()> {
        // Update fact cache
        self.fact_cache.write().await.insert(fact.id.clone(), fact.clone());
        
        // Update temporal index
        self.temporal_index.write().await
            .entry(fact.created_at)
            .or_insert_with(Vec::new)
            .push(fact.id.clone());
            
        // Update topic clusters
        let mut topic_clusters = self.topic_clusters.write().await;
        for topic in &fact.topics {
            topic_clusters
                .entry(topic.clone())
                .or_insert_with(Vec::new)
                .push(fact.id.clone());
        }
        
        Ok(())
    }
    
    /// Load recent facts into cache
    async fn load_recent_facts(&self) -> Result<()> {
        let cutoff = Utc::now() - Duration::days(7);
        
        // Get facts through the service - no connection lifetime issues!
        let facts = self.db_service.load_recent_facts(cutoff).await?;
        
        // Update caches with async operations
        for fact in facts {
            self.update_caches(&fact).await?;
        }
        
        Ok(())
    }
    
    /// Find similar facts based on semantic similarity
    pub async fn find_similar(&self, content: &str, limit: usize) -> Result<Vec<CuratedFact>> {
        // Generate fingerprint for the query
        let _query_fingerprint = self.fingerprinter.fingerprint(content).await?;
        
        // Use database service for similarity search
        self.db_service.find_similar_facts(content, limit).await
    }
    
    /// Get recent facts
    pub async fn get_recent_facts(&self, duration: Duration) -> Result<Vec<CuratedFact>> {
        let cutoff = Utc::now() - duration;
        let cache = self.fact_cache.read().await;
        
        let facts: Vec<_> = cache
            .values()
            .filter(|f| f.created_at > cutoff)
            .cloned()
            .collect();
            
        Ok(facts)
    }
    
    /// Get facts about specific entities
    pub async fn get_facts_about_entities(&self, entities: &[String]) -> Result<Vec<CuratedFact>> {
        let cache = self.fact_cache.read().await;
        
        let facts: Vec<_> = cache
            .values()
            .filter(|f| {
                f.entities.iter().any(|e| entities.contains(&e.name))
            })
            .cloned()
            .collect();
            
        Ok(facts)
    }
    
    /// Get all facts sorted by date (newest first)
    pub async fn get_all_facts_sorted_by_date(&self) -> Result<Vec<CuratedFact>> {
        self.db_service.get_all_facts_sorted_by_date().await
    }
    
    /// Update access statistics for a fact
    pub async fn record_access(&self, fact_id: &str) -> Result<()> {
        // Update database through service
        self.db_service.record_fact_access(fact_id).await?;
        
        // Update cache if present
        if let Some(fact) = self.fact_cache.write().await.get_mut(fact_id) {
            fact.last_accessed = Utc::now();
            fact.access_count += 1;
        }
        
        Ok(())
    }
}