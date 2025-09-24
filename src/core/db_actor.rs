//! Database Actor Pattern for Send-safe database operations
//!
//! This module provides a thread-safe actor pattern for database operations,
//! solving the Send trait issues with rusqlite connections in async contexts.

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn};
use uuid;

use crate::consensus::memory::authoritative_store::CuratedFact;
use crate::consensus::types::{ConsensusProfile, StageResult};
use crate::core::database::{current_timestamp, ConsensusProfile as DbProfile, DatabaseManager};
use rusqlite::OptionalExtension;

/// Commands that can be sent to the database actor
#[derive(Debug)]
pub enum DbCommand {
    // Consensus operations
    StoreConversation {
        id: String,
        user_id: Option<String>,
        profile_id: Option<String>,
        respond_to: oneshot::Sender<Result<()>>,
    },
    UpdateConversationCost {
        conversation_id: String,
        cost: f64,
        tokens_in: u32,
        tokens_out: u32,
        respond_to: oneshot::Sender<Result<()>>,
    },

    // AI Helpers memory operations
    StoreFact {
        fact: CuratedFact,
        respond_to: oneshot::Sender<Result<()>>,
    },
    LoadRecentFacts {
        cutoff: DateTime<Utc>,
        respond_to: oneshot::Sender<Result<Vec<CuratedFact>>>,
    },
    FindSimilarFacts {
        content: String,
        limit: usize,
        respond_to: oneshot::Sender<Result<Vec<CuratedFact>>>,
    },
    CheckFactExists {
        fingerprint: String,
        respond_to: oneshot::Sender<Result<bool>>,
    },
    RecordFactAccess {
        fact_id: String,
        respond_to: oneshot::Sender<Result<()>>,
    },
    GetAllFactsSortedByDate {
        respond_to: oneshot::Sender<Result<Vec<CuratedFact>>>,
    },

    // Profile operations
    GetActiveProfile {
        respond_to: oneshot::Sender<Result<ConsensusProfile>>,
    },
    SetActiveProfile {
        profile_id: String,
        respond_to: oneshot::Sender<Result<()>>,
    },
    GetProfileByName {
        name: String,
        respond_to: oneshot::Sender<Result<Option<ConsensusProfile>>>,
    },

    // Configuration operations
    GetLicenseKey {
        respond_to: oneshot::Sender<Result<Option<String>>>,
    },

    // Analytics operations
    StoreStageUsage {
        conversation_id: String,
        stage_name: String,
        model_id: String,
        prompt_tokens: u32,
        completion_tokens: u32,
        total_tokens: u32,
        cost: f64,
        duration_ms: u64,
        respond_to: oneshot::Sender<Result<()>>,
    },

    // Health check
    HealthCheck {
        respond_to: oneshot::Sender<Result<()>>,
    },
}

/// The database actor that runs on a dedicated thread
struct DatabaseActor {
    manager: Arc<DatabaseManager>,
    receiver: mpsc::Receiver<DbCommand>,
}

impl DatabaseActor {
    /// Run the actor event loop
    async fn run(&mut self) {
        info!("Database actor started");

        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                DbCommand::StoreFact { fact, respond_to } => {
                    let result = self.handle_store_fact(fact);
                    let _ = respond_to.send(result);
                }
                DbCommand::LoadRecentFacts { cutoff, respond_to } => {
                    let result = self.handle_load_recent_facts(cutoff);
                    let _ = respond_to.send(result);
                }
                DbCommand::FindSimilarFacts {
                    content,
                    limit,
                    respond_to,
                } => {
                    let result = self.handle_find_similar_facts(&content, limit);
                    let _ = respond_to.send(result);
                }
                DbCommand::CheckFactExists {
                    fingerprint,
                    respond_to,
                } => {
                    let result = self.handle_check_fact_exists(&fingerprint);
                    let _ = respond_to.send(result);
                }
                DbCommand::RecordFactAccess {
                    fact_id,
                    respond_to,
                } => {
                    let result = self.handle_record_fact_access(&fact_id);
                    let _ = respond_to.send(result);
                }
                DbCommand::GetAllFactsSortedByDate { respond_to } => {
                    let result = self.handle_get_all_facts_sorted_by_date();
                    let _ = respond_to.send(result);
                }
                DbCommand::GetActiveProfile { respond_to } => {
                    let result = self.handle_get_active_profile();
                    let _ = respond_to.send(result);
                }
                DbCommand::SetActiveProfile {
                    profile_id,
                    respond_to,
                } => {
                    let result = self.handle_set_active_profile(&profile_id);
                    let _ = respond_to.send(result);
                }
                DbCommand::GetProfileByName { name, respond_to } => {
                    let result = self.handle_get_profile_by_name(&name);
                    let _ = respond_to.send(result);
                }
                DbCommand::StoreConversation {
                    id,
                    user_id,
                    profile_id,
                    respond_to,
                } => {
                    let result = self.handle_store_conversation(
                        &id,
                        user_id.as_deref(),
                        profile_id.as_deref(),
                    );
                    let _ = respond_to.send(result);
                }
                DbCommand::UpdateConversationCost {
                    conversation_id,
                    cost,
                    tokens_in,
                    tokens_out,
                    respond_to,
                } => {
                    let result = self.handle_update_conversation_cost(
                        &conversation_id,
                        cost,
                        tokens_in,
                        tokens_out,
                    );
                    let _ = respond_to.send(result);
                }
                DbCommand::GetLicenseKey { respond_to } => {
                    let result = self.handle_get_license_key();
                    let _ = respond_to.send(result);
                }
                DbCommand::StoreStageUsage {
                    conversation_id,
                    stage_name,
                    model_id,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                    cost,
                    duration_ms,
                    respond_to,
                } => {
                    let result = self.handle_store_stage_usage(
                        &conversation_id,
                        &stage_name,
                        &model_id,
                        prompt_tokens,
                        completion_tokens,
                        total_tokens,
                        cost,
                        duration_ms,
                    );
                    let _ = respond_to.send(result);
                }
                DbCommand::HealthCheck { respond_to } => {
                    let _ = respond_to.send(Ok(()));
                }
            }
        }

        warn!("Database actor shutting down");
    }

    // Handler methods for consensus facts

    fn handle_store_fact(&self, fact: CuratedFact) -> Result<()> {
        let conn = self.manager.get_connection()?;

        // Check for duplicates first
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM consensus_facts WHERE semantic_fingerprint = ?1",
            [&fact.semantic_fingerprint],
            |row| row.get(0),
        )?;

        if count > 0 {
            info!(
                "Fact already exists with fingerprint: {}",
                fact.semantic_fingerprint
            );
            return Ok(());
        }

        // Store the fact
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

        Ok(())
    }

    fn handle_load_recent_facts(&self, cutoff: DateTime<Utc>) -> Result<Vec<CuratedFact>> {
        let conn = self.manager.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, semantic_fingerprint, content, curator_confidence,
                    source_question, source_conversation_id, consensus_stages,
                    created_at, last_accessed, access_count, related_facts,
                    topics, entities, metadata
             FROM consensus_facts
             WHERE created_at > ?1
             ORDER BY created_at DESC
             LIMIT 1000",
        )?;

        let facts = stmt
            .query_map([cutoff], |row| {
                Ok(CuratedFact {
                    id: row.get(0)?,
                    semantic_fingerprint: row.get(1)?,
                    content: row.get(2)?,
                    curator_confidence: row.get(3)?,
                    source_question: row.get(4)?,
                    source_conversation_id: row.get(5)?,
                    consensus_stages: serde_json::from_str(&row.get::<_, String>(6)?)
                        .unwrap_or_default(),
                    created_at: row.get(7)?,
                    last_accessed: row.get(8)?,
                    access_count: row.get(9)?,
                    related_facts: serde_json::from_str(&row.get::<_, String>(10)?)
                        .unwrap_or_default(),
                    topics: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
                    entities: serde_json::from_str(&row.get::<_, String>(12)?).unwrap_or_default(),
                    metadata: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(facts)
    }

    fn handle_find_similar_facts(&self, _content: &str, limit: usize) -> Result<Vec<CuratedFact>> {
        // For now, return recent facts - full semantic search to be implemented
        let conn = self.manager.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, semantic_fingerprint, content, curator_confidence,
                    source_question, source_conversation_id, consensus_stages,
                    created_at, last_accessed, access_count, related_facts,
                    topics, entities, metadata
             FROM consensus_facts
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;

        let facts = stmt
            .query_map([limit], |row| {
                Ok(CuratedFact {
                    id: row.get(0)?,
                    semantic_fingerprint: row.get(1)?,
                    content: row.get(2)?,
                    curator_confidence: row.get(3)?,
                    source_question: row.get(4)?,
                    source_conversation_id: row.get(5)?,
                    consensus_stages: serde_json::from_str(&row.get::<_, String>(6)?)
                        .unwrap_or_default(),
                    created_at: row.get(7)?,
                    last_accessed: row.get(8)?,
                    access_count: row.get(9)?,
                    related_facts: serde_json::from_str(&row.get::<_, String>(10)?)
                        .unwrap_or_default(),
                    topics: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
                    entities: serde_json::from_str(&row.get::<_, String>(12)?).unwrap_or_default(),
                    metadata: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(facts)
    }

    fn handle_check_fact_exists(&self, fingerprint: &str) -> Result<bool> {
        let conn = self.manager.get_connection()?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM consensus_facts WHERE semantic_fingerprint = ?1",
            [fingerprint],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    fn handle_record_fact_access(&self, fact_id: &str) -> Result<()> {
        let conn = self.manager.get_connection()?;

        conn.execute(
            "UPDATE consensus_facts 
             SET last_accessed = ?1, access_count = access_count + 1
             WHERE id = ?2",
            rusqlite::params![Utc::now(), fact_id],
        )?;

        Ok(())
    }

    fn handle_get_all_facts_sorted_by_date(&self) -> Result<Vec<CuratedFact>> {
        let conn = self.manager.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, semantic_fingerprint, content, curator_confidence,
                    source_question, source_conversation_id, consensus_stages,
                    created_at, last_accessed, access_count, related_facts,
                    topics, entities, metadata
             FROM consensus_facts 
             ORDER BY created_at DESC",
        )?;

        let facts = stmt
            .query_map([], |row| {
                Ok(CuratedFact {
                    id: row.get(0)?,
                    semantic_fingerprint: row.get(1)?,
                    content: row.get(2)?,
                    curator_confidence: row.get(3)?,
                    source_question: row.get(4)?,
                    source_conversation_id: row.get(5)?,
                    consensus_stages: serde_json::from_str(&row.get::<_, String>(6)?)
                        .unwrap_or_default(),
                    created_at: row
                        .get::<_, String>(7)?
                        .parse::<DateTime<Utc>>()
                        .unwrap_or(Utc::now()),
                    last_accessed: row
                        .get::<_, String>(8)?
                        .parse::<DateTime<Utc>>()
                        .unwrap_or(Utc::now()),
                    access_count: row.get(9)?,
                    related_facts: serde_json::from_str(&row.get::<_, String>(10)?)
                        .unwrap_or_default(),
                    topics: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
                    entities: serde_json::from_str(&row.get::<_, String>(12)?).unwrap_or_default(),
                    metadata: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_default(),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(facts)
    }

    // Handler methods for profiles

    fn handle_get_active_profile(&self) -> Result<ConsensusProfile> {
        use rusqlite::OptionalExtension;

        let conn = self.manager.get_connection()?;

        // Get the active profile ID
        let active_profile_id: Option<String> = conn
            .query_row(
                "SELECT value FROM consensus_settings WHERE key = 'active_profile_id'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        let profile_id =
            active_profile_id.ok_or_else(|| anyhow::anyhow!("No active profile configured"))?;

        // Get the profile by ID
        let profile_name: String = conn.query_row(
            "SELECT profile_name FROM consensus_profiles WHERE id = ?1",
            rusqlite::params![profile_id],
            |row| row.get(0),
        )?;

        // Load full profile using existing method
        self.load_profile_by_name(&profile_name)
    }

    fn handle_set_active_profile(&self, profile_id: &str) -> Result<()> {
        let conn = self.manager.get_connection()?;

        conn.execute(
            "INSERT OR REPLACE INTO consensus_settings (key, value) VALUES ('active_profile_id', ?1)",
            rusqlite::params![profile_id],
        )?;

        Ok(())
    }

    fn handle_get_profile_by_name(&self, name: &str) -> Result<Option<ConsensusProfile>> {
        let conn = self.manager.get_connection()?;

        let profile = conn
            .query_row(
                "SELECT id, profile_name, generator_model, refiner_model,
                    validator_model, curator_model, created_at, updated_at
             FROM consensus_profiles WHERE profile_name = ?1",
                rusqlite::params![name],
                |row| {
                    Ok(DbProfile {
                        id: row.get(0)?,
                        profile_name: row.get(1)?,
                        generator_model: row.get(2)?,
                        refiner_model: row.get(3)?,
                        validator_model: row.get(4)?,
                        curator_model: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                },
            )
            .optional()?;

        Ok(profile.map(|p| ConsensusProfile {
            id: p.id,
            profile_name: p.profile_name,
            generator_model: p.generator_model,
            refiner_model: p.refiner_model,
            validator_model: p.validator_model,
            curator_model: p.curator_model,
            created_at: Utc::now(), // TODO: Parse from DB
            is_active: true,
        }))
    }

    fn load_profile_by_name(&self, name: &str) -> Result<ConsensusProfile> {
        self.handle_get_profile_by_name(name)?
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", name))
    }

    // Handler methods for conversations

    fn handle_store_conversation(
        &self,
        id: &str,
        user_id: Option<&str>,
        profile_id: Option<&str>,
    ) -> Result<()> {
        let conn = self.manager.get_connection()?;

        conn.execute(
            "INSERT OR REPLACE INTO conversations 
             (id, user_id, consensus_profile_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                id,
                user_id,
                profile_id,
                current_timestamp(),
                current_timestamp()
            ],
        )?;

        Ok(())
    }

    fn handle_update_conversation_cost(
        &self,
        conversation_id: &str,
        cost: f64,
        tokens_in: u32,
        tokens_out: u32,
    ) -> Result<()> {
        let conn = self.manager.get_connection()?;

        conn.execute(
            "UPDATE conversations 
             SET total_cost = COALESCE(total_cost, 0.0) + ?2,
                 total_tokens_input = COALESCE(total_tokens_input, 0) + ?3,
                 total_tokens_output = COALESCE(total_tokens_output, 0) + ?4,
                 updated_at = ?5
             WHERE id = ?1",
            rusqlite::params![
                conversation_id,
                cost,
                tokens_in,
                tokens_out,
                current_timestamp()
            ],
        )?;

        Ok(())
    }

    // Handler methods for configuration

    fn handle_get_license_key(&self) -> Result<Option<String>> {
        let conn = self.manager.get_connection()?;

        let license_key: Option<String> = conn
            .query_row(
                "SELECT value FROM configurations WHERE key = 'hive_license_key'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(license_key)
    }

    // Handler methods for analytics

    fn handle_store_stage_usage(
        &self,
        conversation_id: &str,
        stage_name: &str,
        model_id: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        total_tokens: u32,
        cost: f64,
        duration_ms: u64,
    ) -> Result<()> {
        let conn = self.manager.get_connection()?;

        conn.execute(
            "INSERT INTO stage_usage (
                id, conversation_id, stage_name, model_id,
                prompt_tokens, completion_tokens, total_tokens,
                cost, duration_ms, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                conversation_id,
                stage_name,
                model_id,
                prompt_tokens,
                completion_tokens,
                total_tokens,
                cost,
                duration_ms,
                chrono::Utc::now(),
            ],
        )?;

        Ok(())
    }
}

/// Public Send+Clone service for database operations
#[derive(Clone)]
pub struct DatabaseService {
    sender: mpsc::Sender<DbCommand>,
}

impl DatabaseService {
    /// Spawn the database actor and return the service
    pub fn spawn(manager: Arc<DatabaseManager>) -> Self {
        let (sender, receiver) = mpsc::channel(1000);

        // Spawn on dedicated OS thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime for database actor");

            rt.block_on(async {
                let mut actor = DatabaseActor { manager, receiver };
                actor.run().await;
            });
        });

        info!("Database service spawned");
        DatabaseService { sender }
    }

    // Public API methods

    pub async fn store_fact(&self, fact: CuratedFact) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::StoreFact {
                fact,
                respond_to: tx,
            })
            .await?;
        rx.await?
    }

    pub async fn load_recent_facts(&self, cutoff: DateTime<Utc>) -> Result<Vec<CuratedFact>> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::LoadRecentFacts {
                cutoff,
                respond_to: tx,
            })
            .await?;
        rx.await?
    }

    pub async fn find_similar_facts(
        &self,
        content: &str,
        limit: usize,
    ) -> Result<Vec<CuratedFact>> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::FindSimilarFacts {
                content: content.to_string(),
                limit,
                respond_to: tx,
            })
            .await?;
        rx.await?
    }

    pub async fn check_fact_exists(&self, fingerprint: &str) -> Result<bool> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::CheckFactExists {
                fingerprint: fingerprint.to_string(),
                respond_to: tx,
            })
            .await?;
        rx.await?
    }

    pub async fn record_fact_access(&self, fact_id: &str) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::RecordFactAccess {
                fact_id: fact_id.to_string(),
                respond_to: tx,
            })
            .await?;
        rx.await?
    }

    pub async fn get_all_facts_sorted_by_date(&self) -> Result<Vec<CuratedFact>> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::GetAllFactsSortedByDate { respond_to: tx })
            .await?;
        rx.await?
    }

    pub async fn get_active_profile(&self) -> Result<ConsensusProfile> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::GetActiveProfile { respond_to: tx })
            .await?;
        rx.await?
    }

    pub async fn set_active_profile(&self, profile_id: &str) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::SetActiveProfile {
                profile_id: profile_id.to_string(),
                respond_to: tx,
            })
            .await?;
        rx.await?
    }

    pub async fn get_profile_by_name(&self, name: &str) -> Result<Option<ConsensusProfile>> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::GetProfileByName {
                name: name.to_string(),
                respond_to: tx,
            })
            .await?;
        rx.await?
    }

    pub async fn store_conversation(
        &self,
        id: &str,
        user_id: Option<String>,
        profile_id: Option<String>,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::StoreConversation {
                id: id.to_string(),
                user_id,
                profile_id,
                respond_to: tx,
            })
            .await?;
        rx.await?
    }

    pub async fn update_conversation_cost(
        &self,
        conversation_id: &str,
        cost: f64,
        tokens_in: u32,
        tokens_out: u32,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::UpdateConversationCost {
                conversation_id: conversation_id.to_string(),
                cost,
                tokens_in,
                tokens_out,
                respond_to: tx,
            })
            .await?;
        rx.await?
    }

    pub async fn health_check(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::HealthCheck { respond_to: tx })
            .await?;
        rx.await?
    }

    pub async fn get_license_key(&self) -> Result<Option<String>> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::GetLicenseKey { respond_to: tx })
            .await?;
        rx.await?
    }

    pub async fn store_stage_usage(
        &self,
        conversation_id: &str,
        stage_name: &str,
        model_id: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        total_tokens: u32,
        cost: f64,
        duration_ms: u64,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbCommand::StoreStageUsage {
                conversation_id: conversation_id.to_string(),
                stage_name: stage_name.to_string(),
                model_id: model_id.to_string(),
                prompt_tokens,
                completion_tokens,
                total_tokens,
                cost,
                duration_ms,
                respond_to: tx,
            })
            .await?;
        rx.await?
    }
}
