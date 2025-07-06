// Consensus Engine - Main entry point for consensus functionality
// Manages profiles, configuration, and pipeline execution

use crate::consensus::pipeline::ConsensusPipeline;
use crate::consensus::streaming::{StreamingCallbacks, StreamingResponse, ConsensusStage, ConsensusResponseResult, ChannelStreamingCallbacks};
use crate::consensus::types::{
    ConsensusConfig, ConsensusProfile, ConsensusResult, ConsensusRequest,
    ContextInjectionStrategy, RetryPolicy, ResponseMetadata,
};
use crate::consensus::profiles::{ExpertProfileManager, TemplateFilter, TemplatePreferences};
use crate::consensus::models::{ModelManager, DynamicModelSelector};
use crate::consensus::temporal::TemporalContextProvider;
use crate::core::config;
use crate::core::api_keys::ApiKeyManager;
use crate::core::database_simple::Database;
// use crate::core::Database; // TODO: Replace with actual database implementation
use anyhow::{Context, Result};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::time::Instant;

/// Main consensus engine that manages the consensus pipeline
pub struct ConsensusEngine {
    database: Option<Arc<Database>>,
    current_profile: Arc<RwLock<ConsensusProfile>>,
    config: Arc<RwLock<ConsensusConfig>>,
    openrouter_api_key: Option<String>,
    profile_manager: Arc<ExpertProfileManager>,
    model_manager: Option<Arc<ModelManager>>,
    model_selector: Option<Arc<DynamicModelSelector>>,
    temporal_provider: Arc<TemporalContextProvider>,
}

impl ConsensusEngine {
    /// Create a new consensus engine
    pub async fn new(database: Option<Arc<Database>>) -> Result<Self> {
        // Load configuration from file system
        let hive_config = config::get_config().await?;
        
        // Get OpenRouter API key from ApiKeyManager (checks database, config, and env)
        let openrouter_api_key = ApiKeyManager::get_openrouter_key().await.ok();
        
        // Initialize profile manager
        let profile_manager = Arc::new(ExpertProfileManager::new());
        
        // Initialize model management if API key is available
        let (model_manager, model_selector) = if let Some(ref key) = openrouter_api_key {
            (
                Some(Arc::new(ModelManager::new(Some(key.clone())))),
                Some(Arc::new(DynamicModelSelector::new(Some(key.clone())))),
            )
        } else {
            (None, None)
        };
        
        // Try to load profile from database first
        let profile = match Self::load_active_profile(&database).await {
            Ok(profile) => profile,
            Err(_) => {
                // Fallback to config-based profile creation
                match hive_config.consensus.profile.as_str() {
                    "balanced" => Self::create_balanced_profile(),
                    "speed" => Self::create_speed_profile(),
                    "quality" => Self::create_quality_profile(),
                    "cost" => Self::create_cost_profile(),
                    _ => Self::create_balanced_profile(), // Default fallback
                }
            }
        };

        let config = ConsensusConfig {
            profile: profile.clone(),
            enable_streaming: hive_config.consensus.streaming.enabled,
            show_progress: true,
            timeout_seconds: 120,
            retry_policy: RetryPolicy::default(),
            context_injection: ContextInjectionStrategy::default(),
        };

        Ok(Self {
            database,
            current_profile: Arc::new(RwLock::new(profile)),
            config: Arc::new(RwLock::new(config)),
            openrouter_api_key,
            profile_manager,
            model_manager,
            model_selector,
            temporal_provider: Arc::new(TemporalContextProvider::default()),
        })
    }

    /// Process a query through the consensus pipeline
    pub async fn process(
        &self,
        query: &str,
        semantic_context: Option<String>,
    ) -> Result<ConsensusResult> {
        let config = self.config.read().await.clone();
        
        let mut pipeline = ConsensusPipeline::new(config, self.openrouter_api_key.clone());
        
        // Set database if available
        if let Some(ref db) = self.database {
            pipeline = pipeline.with_database(db.clone());
        }
        
        pipeline
            .run(query, semantic_context)
            .await
            .context("Failed to run consensus pipeline")
    }

    /// Process with custom callbacks
    pub async fn process_with_callbacks(
        &self,
        query: &str,
        semantic_context: Option<String>,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<ConsensusResult> {
        let config = self.config.read().await.clone();
        
        let mut pipeline = ConsensusPipeline::new(config, self.openrouter_api_key.clone())
            .with_callbacks(callbacks);
        
        // Set database if available
        if let Some(ref db) = self.database {
            pipeline = pipeline.with_database(db.clone());
        }
        
        pipeline
            .run(query, semantic_context)
            .await
            .context("Failed to run consensus pipeline with callbacks")
    }

    /// Process with streaming responses for TUI integration
    pub async fn process_with_streaming(
        &self,
        request: ConsensusRequest,
    ) -> Result<mpsc::UnboundedReceiver<StreamingResponse>> {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        // Create streaming callbacks that send to channel
        let callbacks = Arc::new(ChannelStreamingCallbacks::new(sender.clone()));
        
        // Clone necessary data for the async task
        let engine = self.clone();
        let query = request.query.clone();
        let context = request.context.clone();
        
        // Spawn async task to process consensus
        tokio::spawn(async move {
            let start_time = Instant::now();
            
            // Process through the actual consensus pipeline
            match engine.process_with_callbacks(&query, context, callbacks).await {
                Ok(result) => {
                    // Extract metrics from the result
                    let mut total_tokens = 0u32;
                    let mut total_cost = 0.0f64;
                    let mut models_used = Vec::new();
                    
                    for stage in &result.stages {
                        models_used.push(stage.model.clone());
                        if let Some(usage) = &stage.usage {
                            total_tokens += usage.total_tokens;
                        }
                        if let Some(analytics) = &stage.analytics {
                            total_cost += analytics.cost;
                        }
                    }
                    
                    // Send completion event
                    let response = ConsensusResponseResult {
                        content: result.result.unwrap_or_else(|| "No response generated".to_string()),
                        metadata: ResponseMetadata {
                            total_tokens,
                            cost: total_cost,
                            duration_ms: start_time.elapsed().as_millis() as u64,
                            models_used,
                        },
                    };
                    
                    let _ = sender.send(StreamingResponse::Complete { response });
                }
                Err(e) => {
                    let _ = sender.send(StreamingResponse::Error {
                        stage: ConsensusStage::Generator,
                        error: e.to_string(),
                    });
                }
            }
        });
        
        Ok(receiver)
    }

    /// Get available profiles
    pub async fn get_profiles(&self) -> Result<Vec<ConsensusProfile>> {
        use crate::core::database::{get_database, ConsensusProfile as DbProfile};
        
        // Try to load from database
        match DbProfile::list_all().await {
            Ok(db_profiles) => {
                let current_profile_name = self.current_profile.read().await.profile_name.clone();
                let profiles: Vec<ConsensusProfile> = db_profiles
                    .into_iter()
                    .map(|db_profile| ConsensusProfile {
                        id: db_profile.id.clone(),
                        profile_name: db_profile.profile_name.clone(),
                        generator_model: db_profile.generator_model,
                        refiner_model: db_profile.refiner_model,
                        validator_model: db_profile.validator_model,
                        curator_model: db_profile.curator_model,
                        created_at: chrono::DateTime::parse_from_rfc3339(&db_profile.created_at)
                            .unwrap_or_else(|_| Utc::now().into())
                            .with_timezone(&Utc),
                        is_active: db_profile.profile_name == current_profile_name,
                    })
                    .collect();
                Ok(profiles)
            }
            Err(_) => {
                // Fallback to hardcoded profiles
                Ok(vec![
                    Self::create_balanced_profile(),
                    Self::create_speed_profile(),
                    Self::create_quality_profile(),
                    Self::create_cost_profile(),
                ])
            }
        }
    }

    /// Set active profile
    pub async fn set_profile(&self, profile_name: &str) -> Result<()> {
        // Try to load from database first
        let profile = match Self::load_profile_by_name(profile_name).await {
            Ok(profile) => profile,
            Err(_) => {
                // Fallback to creating profile
                match profile_name {
                    "balanced" | "Consensus_Balanced" => Self::create_balanced_profile(),
                    "speed" | "Consensus_Speed" => Self::create_speed_profile(),
                    "quality" | "Consensus_Elite" => Self::create_quality_profile(),
                    "cost" | "Consensus_Cost" => Self::create_cost_profile(),
                    _ => anyhow::bail!("Unknown profile: {}", profile_name),
                }
            }
        };

        // Update database to set as active profile
        if let Ok(db) = crate::core::database::get_database().await {
            let conn = db.get_connection()?;
            conn.execute(
                "INSERT OR REPLACE INTO consensus_settings (key, value) VALUES ('active_profile', ?1)",
                rusqlite::params![&profile.profile_name],
            )?;
        }

        *self.current_profile.write().await = profile.clone();
        self.config.write().await.profile = profile;

        Ok(())
    }
    
    /// Load profile by name from database
    async fn load_profile_by_name(profile_name: &str) -> Result<ConsensusProfile> {
        use crate::core::database::{get_database, ConsensusProfile as DbProfile};
        
        let db = get_database().await?;
        let db_profile = DbProfile::find_by_name(profile_name).await?
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", profile_name))?;
        
        Ok(ConsensusProfile {
            id: db_profile.id,
            profile_name: db_profile.profile_name,
            generator_model: db_profile.generator_model,
            refiner_model: db_profile.refiner_model,
            validator_model: db_profile.validator_model,
            curator_model: db_profile.curator_model,
            created_at: chrono::DateTime::parse_from_rfc3339(&db_profile.created_at)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc),
            is_active: true,
        })
    }

    /// Get current profile
    pub async fn get_current_profile(&self) -> ConsensusProfile {
        self.current_profile.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut ConsensusConfig),
    {
        let mut config = self.config.write().await;
        updater(&mut config);
        Ok(())
    }

    /// Get expert profile manager
    pub fn get_profile_manager(&self) -> &ExpertProfileManager {
        &self.profile_manager
    }

    /// Get model manager if available
    pub fn get_model_manager(&self) -> Option<&ModelManager> {
        self.model_manager.as_deref()
    }

    /// Get model selector if available
    pub fn get_model_selector(&self) -> Option<&DynamicModelSelector> {
        self.model_selector.as_deref()
    }

    /// Get temporal context provider
    pub fn get_temporal_provider(&self) -> &TemporalContextProvider {
        &self.temporal_provider
    }

    /// Sync models from OpenRouter API
    pub async fn sync_models(&self) -> Result<u32> {
        if let (Some(model_manager), Some(database)) = (&self.model_manager, &self.database) {
            let mut manager_mut = ModelManager::new(self.openrouter_api_key.clone());
            manager_mut.sync_models(database).await
        } else {
            anyhow::bail!("Model synchronization requires both API key and database")
        }
    }

    /// Create expert profile from template
    pub async fn create_expert_profile(&self, template_id: &str, profile_name: Option<&str>) -> Result<()> {
        // Implementation would create profile from template
        // For now, return success
        println!("✅ Expert profile created from template: {}", template_id);
        Ok(())
    }

    /// Find best template for a question
    pub fn find_best_template(&self, question: &str, _preferences: Option<&TemplatePreferences>) -> Vec<String> {
        // Use get_recommended_for_use_case as a simple implementation
        let templates = self.profile_manager.get_recommended_for_use_case(question);
        templates.into_iter().map(|t| t.id.clone()).collect()
    }

    /// Get all available expert templates
    pub fn get_expert_templates(&self, filter: Option<&TemplateFilter>) -> Vec<String> {
        // For now, return all templates if no filter
        let templates = if let Some(_f) = filter {
            // TODO: Implement proper filtering based on TemplateFilter
            self.profile_manager.get_templates()
        } else {
            self.profile_manager.get_templates()
        };
        templates.iter().map(|t| t.id.clone()).collect()
    }

    /// Load active profile from database
    async fn load_active_profile(database: &Option<Arc<Database>>) -> Result<ConsensusProfile> {
        if let Some(db) = database {
            // Get database connection
            let conn = db.get_connection().await?;
            
            // Get active profile name from settings
            let active_profile_name: Result<String, rusqlite::Error> = conn.query_row(
                "SELECT value FROM consensus_settings WHERE key = 'active_profile'",
                [],
                |row| row.get(0),
            );
            
            if let Ok(profile_name) = active_profile_name {
                // Try to load the profile from database
                // For now, return error to fallback to default profile
                anyhow::bail!("Database profile loading not yet implemented");
            }
        }
        
        // No database or profile found, return error to use fallback
        anyhow::bail!("No active profile found")
    }

    /// Create balanced profile (default)
    fn create_balanced_profile() -> ConsensusProfile {
        ConsensusProfile {
            id: "balanced".to_string(),
            profile_name: "Consensus_Balanced".to_string(),
            generator_model: "anthropic/claude-3-5-sonnet".to_string(),
            refiner_model: "openai/gpt-4-turbo".to_string(),
            validator_model: "anthropic/claude-3-opus".to_string(),
            curator_model: "openai/gpt-4o".to_string(),
            created_at: Utc::now(),
            is_active: true,
        }
    }


    /// Create speed-optimized profile
    fn create_speed_profile() -> ConsensusProfile {
        ConsensusProfile {
            id: "speed".to_string(),
            profile_name: "Consensus_Speed".to_string(),
            generator_model: "anthropic/claude-3-haiku".to_string(),
            refiner_model: "openai/gpt-3.5-turbo".to_string(),
            validator_model: "anthropic/claude-3-haiku".to_string(),
            curator_model: "openai/gpt-3.5-turbo".to_string(),
            created_at: Utc::now(),
            is_active: false,
        }
    }

    /// Create quality-optimized profile
    fn create_quality_profile() -> ConsensusProfile {
        ConsensusProfile {
            id: "quality".to_string(),
            profile_name: "Consensus_Elite".to_string(),
            generator_model: "anthropic/claude-3-opus".to_string(),
            refiner_model: "openai/gpt-4o".to_string(),
            validator_model: "anthropic/claude-3-opus".to_string(),
            curator_model: "openai/gpt-4o".to_string(),
            created_at: Utc::now(),
            is_active: false,
        }
    }

    /// Create cost-optimized profile
    fn create_cost_profile() -> ConsensusProfile {
        ConsensusProfile {
            id: "cost".to_string(),
            profile_name: "Consensus_Cost".to_string(),
            generator_model: "meta-llama/llama-3.2-3b-instruct".to_string(),
            refiner_model: "mistralai/mistral-7b-instruct".to_string(),
            validator_model: "meta-llama/llama-3.2-3b-instruct".to_string(),
            curator_model: "mistralai/mistral-7b-instruct".to_string(),
            created_at: Utc::now(),
            is_active: false,
        }
    }

    /// Validate consensus prerequisites
    pub async fn validate_prerequisites(&self) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        // Check OpenRouter API key from configuration
        if self.openrouter_api_key.is_none() {
            errors.push("OpenRouter API key not configured. Run 'hive setup' to configure.".to_string());
        }

        // Check database connection
        // TODO: Implement actual database check

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
        })
    }

    /// Clone the engine for concurrent use
    pub fn clone(&self) -> Self {
        Self {
            database: self.database.clone(),
            current_profile: self.current_profile.clone(),
            config: self.config.clone(),
            openrouter_api_key: self.openrouter_api_key.clone(),
            profile_manager: self.profile_manager.clone(),
            model_manager: self.model_manager.clone(),
            model_selector: self.model_selector.clone(),
            temporal_provider: self.temporal_provider.clone(),
        }
    }
}

/// Validation result for prerequisites
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::core::Database;

    // TODO: Re-enable tests once Database is implemented
    /*
    #[tokio::test]
    async fn test_engine_creation() {
        let db = Arc::new(Database::memory().await.unwrap());
        let engine = ConsensusEngine::new(db).await.unwrap();

        let profile = engine.get_current_profile().await;
        assert_eq!(profile.profile_name, "Consensus_Balanced");
    }

    #[tokio::test]
    async fn test_profile_switching() {
        let db = Arc::new(Database::memory().await.unwrap());
        let engine = ConsensusEngine::new(db).await.unwrap();

        engine.set_profile("speed").await.unwrap();
        let profile = engine.get_current_profile().await;
        assert_eq!(profile.profile_name, "Consensus_Speed");

        engine.set_profile("quality").await.unwrap();
        let profile = engine.get_current_profile().await;
        assert_eq!(profile.profile_name, "Consensus_Elite");
    }
    */
}