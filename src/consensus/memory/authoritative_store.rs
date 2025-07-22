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
use crate::core::database::DatabaseManager;

/// Unique identifier for a fact
pub type FactId = String;

/// Unique identifier for a topic
pub type TopicId = String;

/// Stores curator-validated facts with semantic indexing
pub struct AuthoritativeKnowledgeStore {
    /// Database connection
    database: Arc<DatabaseManager>,
    
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
        database: Arc<DatabaseManager>,
        fingerprinter: Arc<SemanticFingerprinter>,
    ) -> Result<Self> {
        // Initialize database tables
        Self::initialize_tables(&database).await?;
        
        // Load recent facts into cache
        let fact_cache = Arc::new(RwLock::new(HashMap::new()));
        let temporal_index = Arc::new(RwLock::new(BTreeMap::new()));
        let topic_clusters = Arc::new(RwLock::new(HashMap::new()));
        
        let store = Self {
            database,
            fact_cache,
            temporal_index,
            topic_clusters,
            fingerprinter,
        };
        
        // Load recent facts
        store.load_recent_facts().await?;
        
        Ok(store)
    }
    
    /// Initialize database tables
    async fn initialize_tables(database: &DatabaseManager) -> Result<()> {
        let conn = database.get_connection()?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS consensus_facts (
                id TEXT PRIMARY KEY,
                semantic_fingerprint TEXT UNIQUE NOT NULL,
                content TEXT NOT NULL,
                embedding BLOB,
                curator_confidence REAL NOT NULL,
                source_question TEXT NOT NULL,
                source_conversation_id TEXT,
                consensus_stages TEXT NOT NULL, -- JSON
                created_at TIMESTAMP NOT NULL,
                last_accessed TIMESTAMP NOT NULL,
                access_count INTEGER DEFAULT 1,
                related_facts TEXT, -- JSON array
                topics TEXT, -- JSON array
                entities TEXT, -- JSON array
                metadata TEXT NOT NULL -- JSON
            )",
            [],
        )?;
        
        // Create indices for efficient retrieval
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_facts_created_at ON consensus_facts(created_at)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_facts_fingerprint ON consensus_facts(semantic_fingerprint)",
            [],
        )?;
        
        Ok(())
    }
    
    /// Store a new fact
    pub async fn store_fact(&self, fact: CuratedFact) -> Result<()> {
        // Check for duplicates using semantic fingerprint
        if self.fact_exists(&fact.semantic_fingerprint).await? {
            tracing::info!("Fact already exists with fingerprint: {}", fact.semantic_fingerprint);
            return Ok(());
        }
        
        // Store in database
        let conn = self.database.get_connection()?;
        
        conn.execute(
            "INSERT INTO consensus_facts (
                id, semantic_fingerprint, content, curator_confidence,
                source_question, source_conversation_id, consensus_stages,
                created_at, last_accessed, access_count, related_facts,
                topics, entities, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                &fact.id,
                &fact.semantic_fingerprint,
                &fact.content,
                fact.curator_confidence,
                &fact.source_question,
                &fact.source_conversation_id,
                serde_json::to_string(&fact.consensus_stages)?,
                fact.created_at,
                fact.last_accessed,
                fact.access_count,
                serde_json::to_string(&fact.related_facts)?,
                serde_json::to_string(&fact.topics)?,
                serde_json::to_string(&fact.entities)?,
                serde_json::to_string(&fact.metadata)?,
            ],
        )?;
        
        // Update caches
        self.update_caches(&fact).await?;
        
        Ok(())
    }
    
    /// Check if a fact already exists
    async fn fact_exists(&self, fingerprint: &str) -> Result<bool> {
        let conn = self.database.get_connection()?;
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM consensus_facts WHERE semantic_fingerprint = ?1",
            [fingerprint],
            |row| row.get(0),
        )?;
        
        Ok(count > 0)
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
        let conn = self.database.get_connection()?;
        let cutoff = Utc::now() - Duration::days(7);
        
        let mut stmt = conn.prepare(
            "SELECT id, semantic_fingerprint, content, curator_confidence,
                    source_question, source_conversation_id, consensus_stages,
                    created_at, last_accessed, access_count, related_facts,
                    topics, entities, metadata
             FROM consensus_facts
             WHERE created_at > ?1
             ORDER BY created_at DESC
             LIMIT 1000"
        )?;
        
        let facts = stmt.query_map([cutoff], |row| {
            Ok(CuratedFact {
                id: row.get(0)?,
                semantic_fingerprint: row.get(1)?,
                content: row.get(2)?,
                curator_confidence: row.get(3)?,
                source_question: row.get(4)?,
                source_conversation_id: row.get(5)?,
                consensus_stages: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                created_at: row.get(7)?,
                last_accessed: row.get(8)?,
                access_count: row.get(9)?,
                related_facts: serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default(),
                topics: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
                entities: serde_json::from_str(&row.get::<_, String>(12)?).unwrap_or_default(),
                metadata: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_default(),
            })
        })?;
        
        for fact_result in facts {
            if let Ok(fact) = fact_result {
                self.update_caches(&fact).await?;
            }
        }
        
        Ok(())
    }
    
    /// Find similar facts based on semantic similarity
    pub async fn find_similar(&self, content: &str, limit: usize) -> Result<Vec<CuratedFact>> {
        // Generate fingerprint for the query
        let query_fingerprint = self.fingerprinter.fingerprint(content).await?;
        
        // For now, return recent facts - full semantic search to be implemented
        // This would use vector similarity search in production
        let cache = self.fact_cache.read().await;
        let mut facts: Vec<_> = cache.values().cloned().collect();
        facts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        facts.truncate(limit);
        
        Ok(facts)
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
        let conn = self.database.get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, semantic_fingerprint, content, curator_confidence,
                    source_question, source_conversation_id, consensus_stages,
                    created_at, last_accessed, access_count, related_facts,
                    topics, entities, metadata
             FROM consensus_facts 
             ORDER BY created_at DESC"
        )?;
        
        let facts = stmt.query_map([], |row| {
            Ok(CuratedFact {
                id: row.get(0)?,
                semantic_fingerprint: row.get(1)?,
                content: row.get(2)?,
                curator_confidence: row.get(3)?,
                source_question: row.get(4)?,
                source_conversation_id: row.get(5)?,
                consensus_stages: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                created_at: row.get::<_, String>(7)?.parse::<DateTime<Utc>>().unwrap_or(Utc::now()),
                last_accessed: row.get::<_, String>(8)?.parse::<DateTime<Utc>>().unwrap_or(Utc::now()),
                access_count: row.get(9)?,
                related_facts: serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default(),
                topics: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
                entities: serde_json::from_str(&row.get::<_, String>(12)?).unwrap_or_default(),
                metadata: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_else(|_| FactMetadata {
                    fact_type: FactType::Other("unknown".to_string()),
                    domain: None,
                    language: "en".to_string(),
                    complexity_score: 0.5,
                    verification_status: VerificationStatus::Unverified,
                }),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(facts)
    }
    
    /// Update access statistics for a fact
    pub async fn record_access(&self, fact_id: &str) -> Result<()> {
        let conn = self.database.get_connection()?;
        
        conn.execute(
            "UPDATE consensus_facts 
             SET last_accessed = ?1, access_count = access_count + 1
             WHERE id = ?2",
            rusqlite::params![Utc::now(), fact_id],
        )?;
        
        // Update cache if present
        if let Some(fact) = self.fact_cache.write().await.get_mut(fact_id) {
            fact.last_accessed = Utc::now();
            fact.access_count += 1;
        }
        
        Ok(())
    }
}