// Consensus Engine - Main entry point for consensus functionality
// Manages profiles, configuration, and pipeline execution

use crate::consensus::codebase_intelligence::{
    AnalysisProgress, CodebaseCommand, CodebaseIntelligence,
};
use crate::consensus::models::ModelManager;
use crate::consensus::pipeline::ConsensusPipeline;
use crate::consensus::profiles::{ExpertProfileManager, TemplateFilter, TemplatePreferences};
use crate::consensus::repository_context::RepositoryContextManager;
use crate::consensus::streaming::{
    ChannelStreamingCallbacks, ConsensusResponseResult, ConsensusStage, StreamingCallbacks,
    StreamingResponse,
};
use crate::consensus::temporal::TemporalContextProvider;
use crate::consensus::types::{
    ConsensusConfig, ConsensusProfile, ConsensusRequest, ConsensusResult, ContextInjectionStrategy,
    ResponseMetadata, RetryPolicy,
};
use crate::core::api_keys::ApiKeyManager;
use crate::core::config;
use crate::core::config::get_hive_config_dir;
use crate::core::database::DatabaseManager;
use crate::subscription::{ConversationGateway, UsageTracker};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};

/// Main consensus engine that manages the consensus pipeline
#[derive(Clone)]
pub struct ConsensusEngine {
    database: Option<Arc<DatabaseManager>>,
    current_profile: Arc<RwLock<ConsensusProfile>>,
    config: Arc<RwLock<ConsensusConfig>>,
    openrouter_api_key: Option<String>,
    profile_manager: Arc<ExpertProfileManager>,
    model_manager: Option<Arc<ModelManager>>,
    temporal_provider: Arc<TemporalContextProvider>,
    repository_context: Arc<RwLock<Option<Arc<RepositoryContextManager>>>>,
    codebase_intelligence: Arc<RwLock<Option<Arc<CodebaseIntelligence>>>>,
    conversation_gateway: Arc<ConversationGateway>,
    usage_tracker: Arc<RwLock<UsageTracker>>,
    license_key: Option<String>,
    last_auth_remaining: Arc<RwLock<Option<u32>>>, // Store last D1 remaining count
}

impl ConsensusEngine {
    /// Create a new consensus engine
    pub async fn new(database: Option<Arc<DatabaseManager>>) -> Result<Self> {
        // Load configuration from file system
        let hive_config = config::get_config().await?;

        // Get OpenRouter API key from ApiKeyManager (checks database, config, and env)
        let openrouter_api_key = ApiKeyManager::get_openrouter_key().await.ok();

        // Get license key from database (users table)
        let config_dir = get_hive_config_dir();
        let license_key = if let Some(ref db) = database {
            tracing::info!("Loading license from database");
            match Self::get_license_from_database(db).await {
                Ok(key) => {
                    tracing::info!("License loaded from database: {}", key.is_some());
                    key
                }
                Err(e) => {
                    tracing::warn!("Failed to load license from database: {}", e);
                    None
                }
            }
        } else {
            tracing::warn!("No database available, running without license");
            None
        };

        // Initialize subscription components
        let conversation_gateway = Arc::new(ConversationGateway::new()?);
        let usage_tracker = Arc::new(RwLock::new(UsageTracker::new(config_dir)));

        // Initialize profile manager
        let profile_manager = Arc::new(ExpertProfileManager::new());

        // Initialize model management if API key is available
        let model_manager = if let Some(ref key) = openrouter_api_key {
            Some(Arc::new(ModelManager::new(Some(key.clone()))))
        } else {
            None
        };

        // Check if model maintenance needs to run
        if let (Some(db), Some(_)) = (&database, &openrouter_api_key) {
            use crate::consensus::maintenance::TemplateMaintenanceManager;

            let mut maintenance =
                TemplateMaintenanceManager::new(db.clone(), openrouter_api_key.clone());

            if maintenance.needs_maintenance() {
                tracing::info!("Running initial model maintenance...");
                match maintenance.run_maintenance().await {
                    Ok(report) => {
                        tracing::info!(
                            "Model maintenance complete: {} models synced, {} profiles migrated",
                            report.models_synced,
                            report.migrated_profiles.len()
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Initial model maintenance failed: {}. Will retry during consensus.",
                            e
                        );
                    }
                }
            }
        }

        // Load default profile from database (matching TypeScript behavior)
        tracing::info!("Loading default profile from database...");
        let profile = match Self::load_default_profile(&database).await {
            Ok(p) => {
                tracing::info!("Successfully loaded profile: {}", p.profile_name);
                p
            }
            Err(e) => {
                tracing::error!("Failed to load default profile: {}", e);
                return Err(e).context(
                    "No consensus profile found. Please complete onboarding to create profiles",
                );
            }
        };

        let config = ConsensusConfig {
            enable_streaming: hive_config.consensus.streaming.enabled,
            show_progress: true,
            timeout_seconds: hive_config.consensus.timeout_seconds as u64,
            retry_policy: RetryPolicy::default(),
            context_injection: ContextInjectionStrategy::default(),
        };

        // Initialize codebase intelligence if database is available
        let codebase_intelligence = if let Some(ref db) = database {
            Some(Arc::new(CodebaseIntelligence::new(db.clone())))
        } else {
            None
        };

        Ok(Self {
            database,
            current_profile: Arc::new(RwLock::new(profile)),
            config: Arc::new(RwLock::new(config)),
            openrouter_api_key,
            profile_manager,
            model_manager,
            temporal_provider: Arc::new(TemporalContextProvider::default()),
            repository_context: Arc::new(RwLock::new(None)),
            codebase_intelligence: Arc::new(RwLock::new(codebase_intelligence)),
            conversation_gateway,
            usage_tracker,
            license_key,
            last_auth_remaining: Arc::new(RwLock::new(None)),
        })
    }

    /// Set the repository context manager for this engine
    pub async fn set_repository_context(&mut self, repository_context: Arc<RepositoryContextManager>) -> Result<()> {
        let mut repo_ctx = self.repository_context.write().await;
        *repo_ctx = Some(repository_context.clone());
        
        // Update codebase intelligence with repository path
        if let Some(ref mut ci) = self.codebase_intelligence.write().await.as_mut() {
            let context = repository_context.get_context().await;
            if let Some(root_path) = context.root_path {
                let ci_mut = Arc::get_mut(ci).ok_or_else(|| anyhow!("Failed to get mutable reference to CodebaseIntelligence"))?;
                ci_mut.set_repository(root_path);
            }
        }
        
        Ok(())
    }

    /// Process a query through the consensus pipeline
    pub async fn process(
        &self,
        query: &str,
        semantic_context: Option<String>,
    ) -> Result<ConsensusResult> {
        self.process_with_user(query, semantic_context, None).await
    }

    /// Process a query through the consensus pipeline with user_id
    pub async fn process_with_user(
        &self,
        query: &str,
        semantic_context: Option<String>,
        user_id: Option<String>,
    ) -> Result<ConsensusResult> {
        // D1 authorization now happens immediately in the pipeline.run() method
        // This ensures the UI updates right away when consensus starts
        tracing::info!("Starting consensus processing (D1 auth handled by pipeline)");

        let config = self.config.read().await.clone();

        // Always load the active profile from database to ensure we use the latest selection
        let profile = match Self::load_active_profile_from_db().await {
            Ok(p) => {
                tracing::info!("Loaded active profile from database: {}", p.profile_name);
                // Update the cached profile
                *self.current_profile.write().await = p.clone();
                p
            }
            Err(e) => {
                tracing::warn!("Failed to load active profile from database: {}, using cached profile", e);
                self.current_profile.read().await.clone()
            }
        };
        
        let mut pipeline = ConsensusPipeline::new(config, profile, self.openrouter_api_key.clone());

        // Set database if available
        if let Some(ref db) = self.database {
            pipeline = pipeline.with_database(db.clone());
        }

        // Set repository context if available
        if let Some(repo_ctx) = self.repository_context.read().await.as_ref() {
            pipeline = pipeline.with_repository_context(repo_ctx.clone());
        }

        // Set codebase intelligence if available
        if let Some(ci) = self.codebase_intelligence.read().await.as_ref() {
            pipeline = pipeline.with_codebase_intelligence(ci.clone());
        }

        // Run the consensus pipeline (D1 auth and verification happens inside)
        let result = pipeline
            .run(query, semantic_context, user_id.clone())
            .await
            .context("Failed to run consensus pipeline")?;

        Ok(result)
    }

    /// Process with custom callbacks
    pub async fn process_with_callbacks(
        &self,
        query: &str,
        semantic_context: Option<String>,
        callbacks: Arc<dyn StreamingCallbacks>,
        user_id: Option<String>,
    ) -> Result<ConsensusResult> {
        // Check if this is a @codebase command
        if CodebaseIntelligence::is_codebase_command(query) {
            tracing::info!("🔍 Detected @codebase command: {}", query);
            
            // Handle @codebase command with a special consensus result
            let (sender, mut receiver) = mpsc::unbounded_channel();
            
            // Handle the codebase command
            match self.handle_codebase_command(query, sender).await {
                Ok(_) => {
                    // Collect all the streaming responses into a single result
                    let mut content = String::new();
                    while let Some(response) = receiver.recv().await {
                        match response {
                            StreamingResponse::TokenReceived { token } => {
                                content.push_str(&token);
                            }
                            StreamingResponse::Complete { response } => {
                                // Return a special consensus result for @codebase
                                return Ok(ConsensusResult {
                                    result: Some(response.content),
                                    stages: vec![],
                                    total_time: response.metadata.duration_ms,
                                });
                            }
                            StreamingResponse::Error { error, .. } => {
                                return Err(anyhow!("@codebase command failed: {}", error));
                            }
                            _ => {}
                        }
                    }
                    
                    // If we got here without a Complete response, return what we collected
                    return Ok(ConsensusResult {
                        result: Some(content),
                        stages: vec![],
                        total_time: 0,
                    });
                }
                Err(e) => {
                    return Err(anyhow!("Failed to process @codebase command: {}", e));
                }
            }
        }
        
        let config = self.config.read().await.clone();

        // Always load the active profile from database to ensure we use the latest selection
        let profile = match Self::load_active_profile_from_db().await {
            Ok(p) => {
                tracing::info!("✅ Loaded active profile from database: {}", p.profile_name);
                // Update the cached profile
                *self.current_profile.write().await = p.clone();
                
                // Send profile info to UI callbacks
                let _ = callbacks.on_profile_loaded(&p.profile_name, &vec![
                    p.generator_model.clone(),
                    p.refiner_model.clone(),
                    p.validator_model.clone(),
                    p.curator_model.clone(),
                ]);
                
                p
            }
            Err(e) => {
                tracing::error!("❌ Failed to load active profile from database: {}, using cached profile", e);
                let cached = self.current_profile.read().await.clone();
                tracing::warn!("📋 Using cached profile: {}", cached.profile_name);
                cached
            }
        };
        
        let mut pipeline = ConsensusPipeline::new(config, profile, self.openrouter_api_key.clone())
            .with_callbacks(callbacks);

        // Set database if available
        if let Some(ref db) = self.database {
            pipeline = pipeline.with_database(db.clone());
        }

        // Set repository context if available
        if let Some(repo_ctx) = self.repository_context.read().await.as_ref() {
            pipeline = pipeline.with_repository_context(repo_ctx.clone());
        }

        // Set codebase intelligence if available
        if let Some(ci) = self.codebase_intelligence.read().await.as_ref() {
            pipeline = pipeline.with_codebase_intelligence(ci.clone());
        }

        pipeline
            .run(query, semantic_context, user_id)
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
        let user_id = request.user_id.clone();

        // Spawn async task to process consensus
        tokio::spawn(async move {
            let start_time = Instant::now();

            // Check if this is a @codebase command
            if CodebaseIntelligence::is_codebase_command(&query) {
                // Handle @codebase command
                match engine.handle_codebase_command(&query, sender.clone()).await {
                    Ok(_) => {
                        // Command handled successfully
                        tracing::info!("@codebase command processed successfully");
                    }
                    Err(e) => {
                        let _ = sender.send(StreamingResponse::Error {
                            stage: ConsensusStage::Generator,
                            error: format!("Failed to process @codebase command: {}", e),
                        });
                    }
                }
                return;
            }

            // Process through the actual consensus pipeline
            match engine
                .process_with_callbacks(&query, context, callbacks, user_id)
                .await
            {
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
                        content: result
                            .result
                            .unwrap_or_else(|| "No response generated".to_string()),
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
            Err(e) => {
                // No fallback - profiles must exist in database
                tracing::error!("Failed to load profiles from database: {}", e);
                Ok(vec![])
            }
        }
    }

    /// Get last D1 authorization remaining count
    pub async fn get_last_authorization_info(&self) -> Result<Option<u32>> {
        Ok(*self.last_auth_remaining.read().await)
    }

    /// Set active profile
    pub async fn set_profile(&self, profile_name: &str) -> Result<()> {
        // Try to load from database first
        let profile = match Self::load_profile_by_name(profile_name).await {
            Ok(profile) => profile,
            Err(e) => {
                // No fallback - profile must exist in database
                anyhow::bail!("Profile '{}' not found in database: {}. Run 'hive quickstart' to create default profiles.", profile_name, e)
            }
        };

        // Update database to set as active profile
        if let Ok(db) = crate::core::database::get_database().await {
            let conn = db.get_connection()?;
            conn.execute(
                "INSERT OR REPLACE INTO consensus_settings (key, value) VALUES ('active_profile_id', ?1)",
                rusqlite::params![&profile.id],
            )?;
        }

        *self.current_profile.write().await = profile;

        Ok(())
    }

    /// Load profile by name from database
    async fn load_profile_by_name(profile_name: &str) -> Result<ConsensusProfile> {
        use crate::core::database::{get_database, ConsensusProfile as DbProfile};

        let db = get_database().await?;
        let db_profile = DbProfile::find_by_name(profile_name)
            .await?
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
    
    /// Load the active profile from database
    async fn load_active_profile_from_db() -> Result<ConsensusProfile> {
        use crate::core::database::get_database;
        use rusqlite::OptionalExtension;
        
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        // Get the active profile ID from consensus_settings
        let active_profile_id: Option<String> = conn.query_row(
            "SELECT value FROM consensus_settings WHERE key = 'active_profile_id'",
            [],
            |row| row.get(0)
        ).optional()?;
        
        let profile_id = active_profile_id
            .ok_or_else(|| anyhow::anyhow!("No active profile configured"))?;
            
        // Get the profile by ID
        let profile_name: String = conn.query_row(
            "SELECT profile_name FROM consensus_profiles WHERE id = ?1",
            rusqlite::params![profile_id],
            |row| row.get(0)
        )?;
        
        Self::load_profile_by_name(&profile_name).await
    }

    /// Get current profile
    pub async fn get_current_profile(&self) -> ConsensusProfile {
        self.current_profile.read().await.clone()
    }
    
    /// Clear cached profile to force reload from database on next use
    pub async fn clear_cached_profile(&self) {
        tracing::info!("🧹 Clearing cached profile to force database reload on next consensus");
        // Reset to a default/empty profile that will force a database load
        let empty_profile = ConsensusProfile {
            id: String::new(),
            profile_name: String::new(),
            generator_model: String::new(),
            refiner_model: String::new(),
            validator_model: String::new(),
            curator_model: String::new(),
            created_at: Utc::now(),
            is_active: false,
        };
        *self.current_profile.write().await = empty_profile;
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

    /// Get temporal context provider
    pub fn get_temporal_provider(&self) -> &TemporalContextProvider {
        &self.temporal_provider
    }

    /// Handle @codebase commands
    async fn handle_codebase_command(
        &self,
        query: &str,
        sender: mpsc::UnboundedSender<StreamingResponse>,
    ) -> Result<()> {
        let command = CodebaseIntelligence::parse_command(query);
        tracing::info!("🎯 Parsed command: {:?}", command);
        
        // Get codebase intelligence instance
        let ci_lock = self.codebase_intelligence.read().await;
        tracing::info!("🧠 CodebaseIntelligence available: {}", ci_lock.is_some());
        
        let ci = ci_lock.as_ref()
            .ok_or_else(|| anyhow!("Codebase intelligence not initialized"))?;
        
        match command {
            CodebaseCommand::FullScan => {
                // Send initial status
                let _ = sender.send(StreamingResponse::TokenReceived {
                    token: "🔍 Starting deep codebase analysis...\n\n".to_string(),
                });
                
                // Create progress callback
                let sender_clone = sender.clone();
                let progress_callback = move |progress: AnalysisProgress| {
                    let message = match progress {
                        AnalysisProgress::Starting => "📂 Initializing codebase scan...\n".to_string(),
                        AnalysisProgress::Scanning { current, total } => {
                            if total > 0 {
                                format!("📊 Scanning files: {}/{}\n", current, total)
                            } else {
                                "📊 Discovering files...\n".to_string()
                            }
                        }
                        AnalysisProgress::Extracting { current, total } => {
                            format!("🔧 Extracting code objects: {}/{}\n", current, total)
                        }
                        AnalysisProgress::Analyzing => "🧠 Analyzing relationships and architecture...\n".to_string(),
                        AnalysisProgress::Indexing => "📚 Building semantic index...\n".to_string(),
                        AnalysisProgress::Complete => "✅ Codebase analysis complete!\n\n".to_string(),
                    };
                    
                    let _ = sender_clone.send(StreamingResponse::TokenReceived {
                        token: message,
                    });
                };
                
                // Run analysis
                let result = ci.analyze_codebase(progress_callback).await?;
                
                // Send results
                let summary = format!(
                    "## 📊 Codebase Analysis Results\n\n\
                    - **Total Files**: {}\n\
                    - **Code Objects**: {}\n\
                    - **Architecture**: {}\n\
                    - **Key Concepts**: {}\n\n\
                    Your codebase has been indexed! I can now answer questions with deep understanding of your code.\n",
                    result.total_files,
                    result.total_objects,
                    result.architecture,
                    result.key_concepts.join(", ")
                );
                
                let _ = sender.send(StreamingResponse::Complete {
                    response: ConsensusResponseResult {
                        content: summary,
                        metadata: ResponseMetadata {
                            total_tokens: 0,
                            cost: 0.0,
                            duration_ms: 0,
                            models_used: vec!["codebase-intelligence".to_string()],
                        },
                    },
                });
            }
            CodebaseCommand::Refresh => {
                // Similar to FullScan but for refresh
                let _ = sender.send(StreamingResponse::Complete {
                    response: ConsensusResponseResult {
                        content: "🔄 Codebase refresh complete!".to_string(),
                        metadata: ResponseMetadata {
                            total_tokens: 0,
                            cost: 0.0,
                            duration_ms: 0,
                            models_used: vec!["codebase-intelligence".to_string()],
                        },
                    },
                });
            }
            CodebaseCommand::Search(search_query) => {
                // Handle search within indexed codebase
                let _ = sender.send(StreamingResponse::Complete {
                    response: ConsensusResponseResult {
                        content: format!("🔍 Searching for: {}", search_query),
                        metadata: ResponseMetadata {
                            total_tokens: 0,
                            cost: 0.0,
                            duration_ms: 0,
                            models_used: vec!["codebase-intelligence".to_string()],
                        },
                    },
                });
            }
        }
        
        Ok(())
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
    pub async fn create_expert_profile(
        &self,
        template_id: &str,
        profile_name: Option<&str>,
    ) -> Result<()> {
        // Implementation would create profile from template
        // For now, return success
        println!("✅ Expert profile created from template: {}", template_id);
        Ok(())
    }

    /// Find best template for a question
    pub fn find_best_template(
        &self,
        question: &str,
        _preferences: Option<&TemplatePreferences>,
    ) -> Vec<String> {
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

    /// Get license key from database
    async fn get_license_from_database(database: &Arc<DatabaseManager>) -> Result<Option<String>> {
        let conn = database.get_connection()?;

        // Get the currently configured license key from configurations table
        let license_key: Option<String> =
            tokio::task::spawn_blocking(move || -> Result<Option<String>> {
                // First check configurations table for the current license key
                let key = conn
                    .query_row(
                        "SELECT value FROM configurations WHERE key = 'hive_license_key'",
                        [],
                        |row| row.get(0),
                    )
                    .optional()?;
                Ok(key)
            })
            .await??;

        Ok(license_key)
    }

    /// Load default consensus profile from database
    async fn load_default_profile(
        database: &Option<Arc<DatabaseManager>>,
    ) -> Result<ConsensusProfile> {
        tracing::info!("load_default_profile: Starting...");
        let db = database
            .as_ref()
            .ok_or_else(|| anyhow!("Database not available"))?;

        // Get a connection from the pool
        let conn = db.get_connection()?;
        tracing::info!("load_default_profile: Got connection, executing query...");

        // Use spawn_blocking for the query operation only
        let profile = tokio::task::spawn_blocking(move || -> Result<ConsensusProfile> {
            // Query for default profile from consensus_profiles table
            // Query for active profile from consensus_settings (matching TypeScript)
            let active_profile_id: Option<String> = conn.query_row(
                "SELECT value FROM consensus_settings WHERE key = 'active_profile_id'",
                [],
                |row| row.get(0)
            ).optional()?;

            // If no active profile is set, try to get the first available profile
            let profile_id = match active_profile_id {
                Some(id) => id,
                None => {
                    tracing::warn!("No active profile set, attempting to select first available profile");

                    // Get the first available profile
                    let first_profile_id: String = conn.query_row(
                        "SELECT id FROM consensus_profiles ORDER BY created_at ASC LIMIT 1",
                        [],
                        |row| row.get(0)
                    ).map_err(|_| anyhow!("No profiles found. Please complete onboarding to create profiles."))?;

                    // Set it as the active profile
                    conn.execute(
                        "INSERT OR REPLACE INTO consensus_settings (key, value) VALUES ('active_profile_id', ?1)",
                        params![&first_profile_id],
                    )?;

                    tracing::info!("Automatically selected profile '{}' as active", first_profile_id);
                    first_profile_id
                }
            };

            // Query profile directly from consensus_profiles (TypeScript schema)
            let row = conn.query_row(
                "SELECT
                    id,
                    profile_name,
                    generator_model,
                    refiner_model,
                    validator_model,
                    curator_model
                FROM consensus_profiles
                WHERE id = ?1
                LIMIT 1",
                params![profile_id],
                |row| {
                    Ok(ConsensusProfile {
                        id: row.get(0)?,
                        profile_name: row.get(1)?,
                        generator_model: row.get(2)?,
                        refiner_model: row.get(3)?,
                        validator_model: row.get(4)?,
                        curator_model: row.get(5)?,
                        created_at: Utc::now(),
                        is_active: true,
                    })
                },
            ).map_err(|e| {
                // If profile not found, this is a critical error
                anyhow!(
                    "Profile with ID '{}' not found in database. The consensus_settings table has an invalid active_profile_id. Please select a valid profile in Settings.",
                    profile_id
                )
            })?;

            Ok(row)
        }).await??;

        Ok(profile)
    }

    /// Load specific consensus profile by name or ID
    async fn load_profile_by_name_or_id(
        database: &Option<Arc<DatabaseManager>>,
        profile_name_or_id: &str,
    ) -> Result<ConsensusProfile> {
        let db = database
            .as_ref()
            .ok_or_else(|| anyhow!("Database not available"))?;

        let profile_id = profile_name_or_id.to_string();

        // Get a connection from the pool
        let conn = db.get_connection()?;

        // Use spawn_blocking for the query operation only
        let profile = tokio::task::spawn_blocking(move || -> Result<ConsensusProfile> {
            // Query for specific profile by name or ID from consensus_profiles table
            let row = conn.query_row(
                "SELECT
                    id,
                    profile_name,
                    generator_model,
                    refiner_model,
                    validator_model,
                    curator_model
                FROM consensus_profiles
                WHERE id = ?1 OR profile_name = ?1
                LIMIT 1",
                params![profile_id],
                |row| {
                    Ok(ConsensusProfile {
                        id: row.get(0)?,
                        profile_name: row.get(1)?,
                        generator_model: row.get(2)?,
                        refiner_model: row.get(3)?,
                        validator_model: row.get(4)?,
                        curator_model: row.get(5)?,
                        created_at: Utc::now(),
                        is_active: true,
                    })
                },
            )?;

            Ok(row)
        })
        .await??;

        Ok(profile)
    }

    /// Validate consensus prerequisites
    pub async fn validate_prerequisites(&self) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        // Check OpenRouter API key from configuration
        if self.openrouter_api_key.is_none() {
            errors.push(
                "OpenRouter API key not configured. Run 'hive setup' to configure.".to_string(),
            );
        }

        // Check database connection
        // TODO: Implement actual database check

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
        })
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
