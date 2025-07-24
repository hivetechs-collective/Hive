// Consensus Pipeline - Orchestrates the 4-stage consensus process
// Manages flow from Generator → Refiner → Validator → Curator

use std::path::PathBuf;
use crate::consensus::repository_context::RepositoryContextManager;
use crate::consensus::verified_context_builder::VerifiedContextBuilder;
use crate::consensus::cancellation::{CancellationToken, CancellationReason, CancellationChecker};
use crate::consensus::stages::{
    ConsensusStage, CuratorStage, GeneratorStage, RefinerStage, ValidatorStage,
    file_aware_generator::FileAwareGeneratorStage,
    file_aware_curator::FileAwareCuratorStage,
};
use crate::consensus::streaming::{
    ConsoleCallbacks, ProgressInfo, ProgressTracker, StreamingCallbacks,
};
use crate::consensus::temporal::TemporalContextProvider;
use crate::ai_helpers::AIHelperEcosystem;
use crate::consensus::memory::ConsensusMemory;
use crate::consensus::{
    FileOperationExecutor, ExecutorConfig, SmartDecisionEngine, 
    OperationIntelligenceCoordinator, operation_analysis::{AutoAcceptMode, OperationContext as ConsensusOperationContext},
    smart_decision_engine::UserPreferences,
    ModeDetector, ExecutionMode, DirectExecutionHandler, StreamingOperationExecutor,
};
// Removed DynamicModelSelector - direct execution uses profile's generator model
use crate::consensus::types::{
    ConsensusConfig, ConsensusProfile, ConsensusResult, Stage, StageAnalytics, StageResult,
    TokenUsage, AnalyticsFeatures,
};
// use crate::hooks::{HooksSystem, EventType, EventSource, HookEvent, ConsensusIntegration};
use crate::consensus::models::ModelManager;
use crate::consensus::openrouter::{
    OpenRouterClient, OpenRouterMessage, OpenRouterRequest, OpenRouterResponse,
    SimpleStreamingCallbacks, StreamingCallbacks as OpenRouterStreamingCallbacks,
};
use crate::core::database::DatabaseManager;
use crate::core::db_actor::DatabaseService;
use crate::core::usage_tracker::UsageTracker;
use crate::subscription::conversation_gateway::ConversationGateway;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use std::sync::Arc;
use std::time::{Instant, SystemTime};
use uuid::Uuid;

/// The main consensus pipeline orchestrator
pub struct ConsensusPipeline {
    config: ConsensusConfig,
    profile: ConsensusProfile,
    temporal_provider: TemporalContextProvider,
    repository_context: Option<Arc<RepositoryContextManager>>,
    codebase_intelligence: Option<Arc<crate::consensus::codebase_intelligence::CodebaseIntelligence>>,
    verified_context_builder: VerifiedContextBuilder,
    stages: Vec<Box<dyn ConsensusStage>>,
    callbacks: Arc<dyn StreamingCallbacks>,
    // hooks_system: Option<Arc<HooksSystem>>,
    // consensus_integration: Option<Arc<ConsensusIntegration>>,
    openrouter_client: Option<Arc<OpenRouterClient>>,
    model_manager: Option<Arc<ModelManager>>,
    database: Option<Arc<DatabaseManager>>,
    db_service: Option<DatabaseService>,
    api_key: Option<String>,
    usage_tracker: Option<Arc<UsageTracker>>,
    ai_helpers: Option<Arc<AIHelperEcosystem>>,
    consensus_memory: Option<Arc<ConsensusMemory>>,
    file_executor: Option<Arc<FileOperationExecutor>>,
    mode_detector: Option<Arc<ModeDetector>>,
    direct_handler: Option<Arc<DirectExecutionHandler>>,
}

impl ConsensusPipeline {
    /// Create a new consensus pipeline
    pub fn new(
        config: ConsensusConfig,
        profile: ConsensusProfile,
        api_key: Option<String>,
    ) -> Self {
        let callbacks: Arc<dyn StreamingCallbacks> = if config.show_progress {
            Arc::new(ConsoleCallbacks {
                show_progress: true,
            })
        } else {
            Arc::new(ConsoleCallbacks {
                show_progress: false,
            })
        };

        // Initialize OpenRouter client and model management if API key is provided
        let (openrouter_client, model_manager) = if let Some(ref key) = api_key {
            let client = Arc::new(OpenRouterClient::new(key.clone()));
            let manager = Arc::new(ModelManager::new(Some(key.clone())));
            (Some(client), Some(manager))
        } else {
            (None, None)
        };

        Self {
            config,
            profile,
            temporal_provider: TemporalContextProvider::default(),
            repository_context: None, // Will be set when needed
            codebase_intelligence: None, // Will be set when needed
            verified_context_builder: VerifiedContextBuilder::new(),
            stages: vec![
                Box::new(FileAwareGeneratorStage::new()),
                Box::new(RefinerStage::new()),
                Box::new(ValidatorStage::new()),
                Box::new(FileAwareCuratorStage::new()),
            ],
            callbacks,
            // hooks_system: None,
            // consensus_integration: None,
            openrouter_client,
            model_manager,
            database: None, // Will be set later when needed
            db_service: None, // Will be set when database is provided
            api_key,
            usage_tracker: None, // Will be set when database is provided
            ai_helpers: None, // Will be set when database is provided
            consensus_memory: None, // Will be set when database is provided
            file_executor: None, // Will be set when AI helpers are configured
            mode_detector: None, // Will be set when AI helpers are configured
            direct_handler: None, // Will be set when components are configured
        }
    }

    /// Create with custom callbacks
    pub fn with_callbacks(mut self, callbacks: Arc<dyn StreamingCallbacks>) -> Self {
        self.callbacks = callbacks;
        self
    }

    /// Set the database for model management
    pub fn with_database(mut self, database: Arc<DatabaseManager>) -> Self {
        self.database = Some(database.clone());
        // Create database service for Send-safe operations
        self.db_service = Some(DatabaseService::spawn(database.clone()));
        // Initialize usage tracker with unified database
        self.usage_tracker = Some(Arc::new(UsageTracker::new(database)));
        self
    }

    /// Set the repository context manager for this pipeline
    pub fn with_repository_context(mut self, repository_context: Arc<RepositoryContextManager>) -> Self {
        self.repository_context = Some(repository_context);
        self
    }
    
    /// Configure mode detection for Claude Code-style execution
    pub async fn with_mode_detection(mut self) -> Result<Self> {
        // Only configure if we have the necessary components
        if let (Some(ai_helpers), Some(openrouter_client), Some(model_manager), Some(database)) = 
            (&self.ai_helpers, &self.openrouter_client, &self.model_manager, &self.database) {
            
            // Create mode detector
            let mode_detector = ModeDetector::new()?
                .with_ai_helpers(ai_helpers.clone());
            self.mode_detector = Some(Arc::new(mode_detector));
            
            // Create streaming executor for direct mode
            if let Some(file_executor) = &self.file_executor {
                let streaming_executor = Arc::new(
                    StreamingOperationExecutor::new(
                        file_executor.clone(),
                        ai_helpers.clone(),
                        Arc::new(std::sync::atomic::AtomicBool::new(true)), // Auto-accept for direct mode
                        tokio::sync::mpsc::unbounded_channel().0, // Status channel
                    )
                );
                
                // Create direct execution handler
                let generator_stage = Arc::new(GeneratorStage::new());
                let direct_handler = Arc::new(
                    DirectExecutionHandler::new(
                        generator_stage,
                        ai_helpers.clone(),
                        streaming_executor,
                        openrouter_client.clone(),
                        self.profile.clone(),
                        database.clone(),
                    )
                );
                
                self.direct_handler = Some(direct_handler);
            }
        }
        
        Ok(self)
    }
    
    /// Initialize repository verification for anti-hallucination
    pub async fn with_repository_verification(mut self, repo_path: std::path::PathBuf) -> Result<Self> {
        self.verified_context_builder
            .with_repository_verification(repo_path)
            .await?;
        tracing::info!("Repository verification initialized for consensus pipeline");
        Ok(self)
    }
    
    /// Set the codebase intelligence for this pipeline
    pub fn with_codebase_intelligence(mut self, codebase_intelligence: Arc<crate::consensus::codebase_intelligence::CodebaseIntelligence>) -> Self {
        self.codebase_intelligence = Some(codebase_intelligence);
        self
    }

    /// Set the AI helpers for this pipeline
    pub fn with_ai_helpers(mut self, ai_helpers: Arc<AIHelperEcosystem>) -> Self {
        self.ai_helpers = Some(ai_helpers.clone());
        
        // ConsensusMemory will be initialized asynchronously later to avoid runtime conflicts
        
        // Initialize file executor when AI helpers are available
        if let Some(ref db) = self.database {
            // Create default user preferences
            let user_prefs = UserPreferences {
                risk_tolerance: 0.3,
                auto_backup: true,
                require_confirmation_for_deletions: true,
                require_confirmation_for_mass_updates: true,
                trust_ai_suggestions: 0.8,
                preferred_mode: AutoAcceptMode::Conservative,
                custom_rules: vec![],
            };
            
            // Initialize decision engine with proper constructor
            let decision_engine = SmartDecisionEngine::new(
                AutoAcceptMode::Balanced, // Default to balanced mode
                user_prefs,
                None, // History database can be added later
            );
            
            // Extract AI helpers from ecosystem
            let intelligence_coordinator = OperationIntelligenceCoordinator::new(
                ai_helpers.knowledge_indexer.clone(),
                ai_helpers.context_retriever.clone(),
                ai_helpers.pattern_recognizer.clone(),
                ai_helpers.quality_analyzer.clone(),
                ai_helpers.knowledge_synthesizer.clone(),
            );
            
            let executor_config = ExecutorConfig {
                create_backups: true,
                validate_syntax: false, // Can be enabled based on config
                dry_run_mode: false, // Real execution mode
                max_file_size: 10 * 1024 * 1024, // 10MB
                allowed_extensions: vec![
                    "rs", "js", "ts", "py", "java", "cpp", "c", "h", 
                    "go", "rb", "php", "md", "txt", "json", "yaml", 
                    "yml", "toml", "html", "css", "scss"
                ].into_iter().map(String::from).collect(),
                forbidden_paths: vec![], // Can be configured
                stop_on_error: true,
            };
            
            let file_executor = FileOperationExecutor::new(
                executor_config,
                decision_engine,
                intelligence_coordinator,
            );
            
            self.file_executor = Some(Arc::new(file_executor));
            tracing::info!("Initialized FileOperationExecutor with AI helpers");
        }
        
        self
    }
    
    /// Initialize consensus memory asynchronously (must be called after with_ai_helpers and with_database)
    pub async fn initialize_consensus_memory(&mut self) -> Result<()> {
        if let (Some(ref db_service), Some(ref ai_helpers)) = (&self.db_service, &self.ai_helpers) {
            // Initialize ConsensusMemory for storing and retrieving knowledge
            match ConsensusMemory::new(db_service.clone(), ai_helpers.clone()).await {
                Ok(consensus_memory) => {
                    let consensus_memory_arc = Arc::new(consensus_memory);
                    
                    // Connect the hive mind - give AI helpers access to the knowledge store
                    if let Err(e) = consensus_memory_arc.connect_hive_mind(ai_helpers).await {
                        tracing::warn!("Failed to connect hive mind: {}", e);
                    } else {
                        tracing::info!("✅ ConsensusMemory initialized and hive mind connected - All stages share collective knowledge!");
                    }
                    
                    self.consensus_memory = Some(consensus_memory_arc);
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize ConsensusMemory: {}. Continuing without enhanced memory.", e);
                }
            }
        }
        Ok(())
    }

    /// Set the hooks system for this pipeline
    // pub fn with_hooks(mut self, hooks_system: Arc<HooksSystem>) -> Self {
    //     self.hooks_system = Some(hooks_system);
    //     self
    // }

    /// Set the consensus integration for enterprise features
    // pub fn with_consensus_integration(mut self, integration: Arc<ConsensusIntegration>) -> Self {
    //     self.consensus_integration = Some(integration);
    //     self
    // }
    
    /// Get the file executor if available
    pub fn get_file_executor(&self) -> Option<Arc<FileOperationExecutor>> {
        self.file_executor.clone()
    }

    /// Run the full consensus pipeline
    pub async fn run(
        &self,
        question: &str,
        context: Option<String>,
        user_id: Option<String>,
    ) -> Result<ConsensusResult> {
        let cancellation_token = CancellationToken::new();
        self.run_with_cancellation(question, context, user_id, cancellation_token).await
    }

    /// Run the full consensus pipeline with cancellation support
    pub async fn run_with_cancellation(
        &self,
        question: &str,
        context: Option<String>,
        user_id: Option<String>,
        cancellation_token: CancellationToken,
    ) -> Result<ConsensusResult> {
        // Create cancellation checker for periodic checks
        let mut cancellation_checker = CancellationChecker::new(
            cancellation_token.clone(),
            std::time::Duration::from_millis(500)
        );

        // Check for cancellation before starting
        cancellation_token.throw_if_cancelled()?;

        // AUTONOMOUS AI HELPER: Let the AI think independently about the user's question
        let autonomous_context = if let Some(ref ai_helpers) = self.ai_helpers {
            if let Some(ref autonomous_helper) = ai_helpers.autonomous_helper {
                tracing::info!("🤖 Autonomous AI Helper processing raw user input...");
                
                match autonomous_helper.process_autonomously(question).await {
                    Ok(decision) => {
                        tracing::info!(
                            "🧠 AI understands: {} (confidence: {:.2})",
                            decision.intent_understanding,
                            decision.confidence
                        );
                        
                        // Log the autonomous actions taken
                        for action in &decision.actions {
                            tracing::info!("🎯 AI autonomously decided to: {:?}", action);
                        }
                        
                        // Use the context the AI gathered autonomously
                        if let Some(ref gathered_context) = decision.gathered_context {
                            tracing::info!(
                                "✅ AI autonomously gathered {} chars of context",
                                gathered_context.len()
                            );
                            
                            // Merge with existing context if any
                            if let Some(existing_context) = context {
                                Some(format!("{}\n\n{}", existing_context, gathered_context))
                            } else {
                                Some(gathered_context.clone())
                            }
                        } else {
                            context
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Autonomous AI Helper failed: {}. Continuing with normal flow.", e);
                        context
                    }
                }
            } else {
                tracing::debug!("Autonomous AI Helper not available");
                context
            }
        } else {
            context
        };
        
        // Use the autonomous context (which may include AI-gathered information)
        let context = autonomous_context;

        // TODO: Re-enable usage tracking when database is unified
        // Check usage limits before running consensus
        // if let (Some(user_id), Some(usage_tracker)) = (user_id.as_ref(), &self.usage_tracker) {
        //     let usage_check = usage_tracker.check_usage_before_conversation(user_id).await?;
        //
        //     if !usage_check.allowed {
        //         return Err(anyhow::anyhow!(
        //             "Usage limit reached: {}. {} conversations remaining today.",
        //             usage_check.reason,
        //             usage_check.remaining_conversations
        //         ));
        //     }
        //
        //     tracing::info!(
        //         "Usage check passed for user {}: {} - {} remaining",
        //         user_id,
        //         usage_check.reason,
        //         usage_check.remaining_conversations
        //     );
        // }

        // Check if maintenance has run recently
        if let Some(db) = &self.database {
            use crate::consensus::maintenance::TemplateMaintenanceManager;

            let mut maintenance = TemplateMaintenanceManager::new(db.clone(), self.api_key.clone());

            if maintenance.needs_maintenance() {
                tracing::info!("Running model maintenance before consensus...");
                match maintenance.run_maintenance().await {
                    Ok(report) => {
                        tracing::info!(
                            "Model maintenance complete: {} models synced, {} profiles migrated",
                            report.models_synced,
                            report.migrated_profiles.len()
                        );
                    }
                    Err(e) => {
                        tracing::warn!("Model maintenance failed: {}. Continuing anyway.", e);
                    }
                }
            }
        }

        let conversation_id = Uuid::new_v4().to_string();
        let pipeline_start = Instant::now();
        let mut total_cost = 0.0;

        // UPDATE D1 AND REFRESH UI IMMEDIATELY BEFORE CONSENSUS STARTS
        // This ensures the count is updated in real-time as soon as user initiates consensus
        let license_key_for_d1 = if let Some(db_service) = &self.db_service {
            // Get the current license key from configurations through database service
            let license_key = db_service.get_license_key().await?;

            if let Some(license_key) = license_key {
                tracing::info!("Starting consensus with license key: {}", &license_key[..8]); // Log only first 8 chars for security

                // Call D1 to validate license and increment conversation count IMMEDIATELY
                // D1 is the source of truth for user information
                let gateway = ConversationGateway::new()?;
                match gateway
                    .request_conversation_authorization(question, &license_key)
                    .await
                {
                    Ok(auth) => {
                        let remaining = auth.remaining.unwrap_or(u32::MAX);
                        tracing::info!(
                            "D1 authorization successful! {} conversations remaining today",
                            if remaining == u32::MAX {
                                "Unlimited".to_string()
                            } else {
                                remaining.to_string()
                            }
                        );

                        // Notify callbacks immediately about D1 authorization
                        // This updates the UI count right away, not after consensus completes
                        if let Err(e) = self.callbacks.on_d1_authorization(remaining) {
                            tracing::warn!("Failed to notify D1 authorization: {}", e);
                        }

                        // Store the authorization for later verification
                        // The remaining count is already updated in D1 and should refresh UI
                        Some((license_key, auth))
                    }
                    Err(e) => {
                        tracing::error!("Failed to authorize with D1: {}", e);
                        // Return the error to stop consensus if authorization fails
                        return Err(e);
                    }
                }
            } else {
                tracing::info!("No license key configured, running without D1 authorization");
                None
            }
        } else {
            None
        };

        // Store the D1 authorization for later verification
        let d1_auth = license_key_for_d1.map(|(_, auth)| auth);

        // Emit BeforeConsensus hook event
        //         // if let Some(hooks) = &self.hooks_system {
        //         //     let event = HookEvent::new(
        //         //         EventType::BeforeConsensus,
        //                 EventSource::Consensus { stage: "pipeline".to_string() }
        //             )
        //             .with_context("question", question)
        //             .with_context("conversation_id", &conversation_id);
        //
        //             if let Err(e) = hooks.dispatch_event(event).await {
        //                 tracing::warn!("Failed to dispatch BeforeConsensus hook event: {}", e);
        //             }
        //         }

        // Check if temporal context is needed
        let temporal_context = if self.temporal_provider.requires_temporal_context(question) {
            Some(self.temporal_provider.build_current_context().await?)
        } else {
            None
        };

        // Get memory context (recent + thematic) - CRITICAL for multi-conversation intelligence
        tracing::info!("Retrieving memory context for question: {}", question);
        let memory_context = self.get_memory_context(question).await?;
        if let Some(ref mem_ctx) = memory_context {
            tracing::info!("Retrieved memory context with {} characters", mem_ctx.len());
            tracing::debug!(
                "Memory context preview: {}",
                &mem_ctx.chars().take(200).collect::<String>()
            );
        } else {
            tracing::info!("No memory context found (this is normal for first conversation)");
        }

        // Check if repository verification is available
        if !self.verified_context_builder.has_verification() {
            tracing::warn!("⚠️ No repository verification available - consensus may produce hallucinations!");
        } else {
            tracing::info!("✅ Repository verification is active - anti-hallucination system enabled");
        }

        let mut previous_answer: Option<String> = None;
        let mut stage_results = Vec::new();

        // Enhanced AI Helper decision-making for intelligent mode selection
        if let (Some(mode_detector), Some(direct_handler)) = (&self.mode_detector, &self.direct_handler) {
            let detected_mode = mode_detector.detect_mode(question).await;
            
            match detected_mode {
                ExecutionMode::Direct => {
                    // Use direct execution path for simple operations
                    tracing::info!("🚀 AI Helper chose DIRECT execution mode for efficiency");
                    
                    // Execute directly with the generator stage and capture the response
                    let response_content = direct_handler.handle_request(
                        question,
                        context.as_deref(),
                        self.callbacks.clone(),
                    ).await?;
                    
                    // Create a proper result for direct mode with the actual response
                    let final_result = ConsensusResult {
                        success: true,
                        result: Some(response_content.clone()),
                        error: None,
                        stages: vec![StageResult {
                            stage_id: "generator".to_string(),
                            stage_name: "Direct Mode".to_string(),
                            question: question.to_string(),
                            answer: response_content,
                            model: self.profile.generator_model.clone(),
                            conversation_id: conversation_id.clone(),
                            timestamp: Utc::now(),
                            usage: None,
                            analytics: Some(StageAnalytics {
                                duration: pipeline_start.elapsed().as_secs_f64(),
                                cost: 0.0,
                                input_cost: 0.0,
                                output_cost: 0.0,
                                provider: "direct".to_string(),
                                model_internal_id: self.profile.generator_model.clone(),
                                quality_score: 0.95,
                                error_count: 0,
                                fallback_used: false,
                                rate_limit_hit: false,
                                retry_count: 0,
                                start_time: Utc::now() - chrono::Duration::milliseconds(pipeline_start.elapsed().as_millis() as i64),
                                end_time: Utc::now(),
                                memory_usage: None,
                                features: AnalyticsFeatures {
                                    streaming: true,
                                    routing_variant: "direct".to_string(),
                                    optimization_applied: Some(true),
                                },
                            }),
                        }],
                        conversation_id: conversation_id.clone(),
                        total_duration: pipeline_start.elapsed().as_secs_f64(),
                        total_cost,
                    };
                    
                    return Ok(final_result);
                }
                ExecutionMode::HybridConsensus => {
                    tracing::info!("🔄 AI Helper chose HYBRID consensus mode (consensus with inline operations)");
                    // Continue with normal consensus but with inline operation support
                }
                ExecutionMode::Consensus => {
                    tracing::info!("🧠 AI Helper chose FULL consensus mode (4-stage collaborative analysis)");
                    // Continue with normal consensus
                }
            }
        }

        // Run through all 4 stages
        for (i, stage_handler) in self.stages.iter().enumerate() {
            // Check for cancellation before each stage
            cancellation_checker.check_if_due()?;
            
            let stage = stage_handler.stage();

            // Use model directly from profile - the maintenance system ensures these are always valid
            let model = self.profile.get_model_for_stage(stage).to_string();

            // Build verified context for this specific stage (includes mandatory verification)
            let verified_stage_context = match self.verified_context_builder.build_verified_stage_context(
                stage,
                question,
                context.clone(),
                temporal_context.clone(),
                memory_context.clone(),
                self.repository_context.clone(),
                self.ai_helpers.as_ref().cloned(),
            ).await {
                Ok(context) => {
                    tracing::info!("Built verified context for {} stage: {} chars", stage.display_name(), context.len());
                    Some(context)
                }
                Err(e) => {
                    tracing::error!("Failed to build verified context for {} stage: {}", stage.display_name(), e);
                    return Err(e);
                }
            };

            // Execute pre-stage hooks with enterprise integration
            // if let Some(integration) = &self.consensus_integration {
            //     // Estimate stage cost
            //     let estimated_cost = self.estimate_stage_cost(stage, model, question).await?;
            //
            //     // Execute pre-stage hooks with cost and quality checks
            //     let pre_stage_result = integration.execute_pre_stage_hooks(
            //         stage,
            //         &conversation_id,
            //         question,
            //         model,
            //         estimated_cost,
            //     ).await?;
            //
            //     // Check if we should proceed
            //     if !pre_stage_result.proceed {
            //         if let Some(approval_required) = pre_stage_result.approval_required {
            //             // Handle approval requirement
            //             tracing::info!("Approval required for {} stage: {}", stage.as_str(), approval_required.description);
            //
            //             // For now, we'll continue with a warning
            //             // In a real implementation, this would wait for approval
            //             for warning in &pre_stage_result.warnings {
            //                 tracing::warn!("Pre-stage warning: {}", warning);
            //             }
            //         } else {
            //             // Stage was blocked
            //             return Err(anyhow::anyhow!("Stage {} was blocked by enterprise hooks", stage.as_str()));
            //         }
            //     }
            // }

            // Emit BeforeStage hook event
            //             if let Some(hooks) = &self.hooks_system {
            //                 let stage_event_type = match stage {
            //                     Stage::Generator => EventType::BeforeGeneratorStage,
            //                     Stage::Refiner => EventType::BeforeRefinerStage,
            //                     Stage::Validator => EventType::BeforeValidatorStage,
            //                     Stage::Curator => EventType::BeforeCuratorStage,
            //                 };
            //
            //                 let event = HookEvent::new(
            //                     stage_event_type,
            //                     EventSource::Consensus { stage: stage.as_str().to_string() }
            //                 )
            //                 .with_context("question", question)
            //                 .with_context("conversation_id", &conversation_id)
            //                 .with_context("model", model)
            //                 .with_context("stage_index", i)
            //                 .with_context("previous_answer", previous_answer.as_deref().unwrap_or(""));
            //
            //                 if let Err(e) = hooks.dispatch_event(event).await {
            //                     tracing::warn!("Failed to dispatch Before{:?}Stage hook event: {}", stage, e);
            //                 }
            //             }

            // Notify stage start
            self.callbacks.on_stage_start(stage, &model)?;

            // Run the stage with verified context (includes mandatory repository verification)
            let stage_context = verified_stage_context.as_deref();

            let stage_result = self
                .run_single_stage(
                    stage,
                    stage_handler.as_ref(),
                    question,
                    previous_answer.as_deref(),
                    stage_context,
                    &conversation_id,
                    &model,
                    &cancellation_token,
                )
                .await
                .with_context(|| format!("Failed to run {} stage", stage.display_name()))?;

            // Update cost
            if let Some(analytics) = &stage_result.analytics {
                total_cost += analytics.cost;

                // Emit cost control hook events if needed
                //                 if let Some(hooks) = &self.hooks_system {
                //                     if analytics.cost > 0.05 { // Threshold for cost notifications
                //                         let cost_event = HookEvent::new(
                //                             EventType::CostThresholdReached,
                //                             EventSource::Consensus { stage: stage.as_str().to_string() }
                //                         )
                //                         .with_context("estimated_cost", analytics.cost)
                //                         .with_context("stage_cost", analytics.cost)
                //                         .with_context("total_cost", total_cost)
                //                         .with_context("model", model);
                //
                //                         if let Err(e) = hooks.dispatch_event(cost_event).await {
                //                             tracing::warn!("Failed to dispatch CostThresholdReached hook event: {}", e);
                //                         }
                //                     }
                //                 }
            }

            // Store the answer for next stage
            previous_answer = Some(stage_result.answer.clone());

            // Notify stage complete
            self.callbacks.on_stage_complete(stage, &stage_result)?;
            
            // Only learn from Curator stage as it's the authoritative final output
            // Earlier stages may contain incomplete or unvalidated information
            if stage == Stage::Curator {
                if let Some(ref ai_helpers) = self.ai_helpers {
                    if let Err(e) = ai_helpers.learn_from_stage_completion(&stage_result).await {
                        tracing::warn!("Failed to trigger continuous learning from Curator: {}", e);
                    }
                    tracing::info!("📚 Learning from Curator's authoritative output");
                }
            }

            // Execute post-stage hooks with quality validation
            // if let Some(integration) = &self.consensus_integration {
            //     let post_stage_result = integration.execute_post_stage_hooks(
            //         stage,
            //         &stage_result,
            //     ).await?;
            //
            //     // Check if we should continue
            //     if !post_stage_result.proceed {
            //         if let Some(approval_required) = post_stage_result.approval_required {
            //             // Handle approval requirement
            //             tracing::warn!("Quality issue in {} stage requires approval: {}", stage.as_str(), approval_required.description);
            //
            //             // For now, we'll continue with warnings
            //             for warning in &post_stage_result.warnings {
            //                 tracing::warn!("Post-stage warning: {}", warning);
            //             }
            //         } else {
            //             // Stage result was rejected by quality gates
            //             return Err(anyhow::anyhow!("Stage {} result was rejected by quality gates", stage.as_str()));
            //         }
            //     }
            //
            //     // Log any warnings
            //     for warning in &post_stage_result.warnings {
            //         tracing::warn!("Quality warning for {} stage: {}", stage.as_str(), warning);
            //     }
            // }

            // Emit AfterStage hook event
            //             if let Some(hooks) = &self.hooks_system {
            //                 let stage_event_type = match stage {
            //                     Stage::Generator => EventType::AfterGeneratorStage,
            //                     Stage::Refiner => EventType::AfterRefinerStage,
            //                     Stage::Validator => EventType::AfterValidatorStage,
            //                     Stage::Curator => EventType::AfterCuratorStage,
            //                 };
            //
            //                 let event = HookEvent::new(
            //                     stage_event_type,
            //                     EventSource::Consensus { stage: stage.as_str().to_string() }
            //                 )
            //                 .with_context("question", question)
            //                 .with_context("conversation_id", &conversation_id)
            //                 .with_context("model", model)
            //                 .with_context("answer", &stage_result.answer)
            //                 .with_context("duration", stage_result.analytics.as_ref().map(|a| a.duration).unwrap_or(0.0))
            //                 .with_context("cost", stage_result.analytics.as_ref().map(|a| a.cost).unwrap_or(0.0));
            //
            //                 if let Err(e) = hooks.dispatch_event(event).await {
            //                     tracing::warn!("Failed to dispatch After{:?}Stage hook event: {}", stage, e);
            //                 }
            //             }

            stage_results.push(stage_result);
            
            // Check for cancellation AFTER stage completes to prevent next stage from starting
            if cancellation_token.is_cancelled() {
                tracing::info!("Consensus cancelled after {} stage - stopping pipeline", stage.display_name());
                return Err(anyhow!("Consensus was cancelled by user"));
            }
        }

        // The final answer is from the Curator stage
        let final_answer = previous_answer
            .unwrap_or_else(|| "Error: No response generated from consensus pipeline".to_string());

        // Process Curator output through AI helpers
        if let Some(ref ai_helpers) = self.ai_helpers {
            match ai_helpers.process_curator_output(&final_answer, question, &conversation_id).await {
                Ok(processed_knowledge) => {
                    tracing::info!(
                        "AI helpers processed Curator output: {} patterns found, quality score: {:.2}",
                        processed_knowledge.patterns.len(),
                        processed_knowledge.quality.overall_score
                    );
                    
                    // Log any quality issues
                    for issue in &processed_knowledge.quality.issues {
                        tracing::warn!("Quality issue: {} - {}", issue.issue_type, issue.description);
                    }
                    
                    // Log discovered insights
                    for insight in &processed_knowledge.insights {
                        tracing::info!("Insight ({:?}): {}", insight.insight_type, insight.content);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to process Curator output with AI helpers: {}", e);
                    // Continue without helper processing
                }
            }
        }
        
        // Store curator output in ConsensusMemory for future context injection
        if let Some(ref consensus_memory) = self.consensus_memory {
            match consensus_memory.store_curator_output(&final_answer, question, stage_results.clone()).await {
                Ok(learning_result) => {
                    tracing::info!(
                        "✅ Curator output stored in ConsensusMemory: {} facts added, {} patterns detected",
                        learning_result.facts_added,
                        learning_result.patterns_detected
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to store curator output in ConsensusMemory: {}", e);
                    // Continue without storing - don't fail consensus
                }
            }
        }
        
        // Execute file operations if present in curator output
        if let Some(ref file_executor) = self.file_executor {
            if let Some(ref repo_manager) = self.repository_context {
                // Get the repository context
                let repo_context = repo_manager.get_context().await;
                
                // Create operation context
                let operation_context = ConsensusOperationContext {
                    repository_path: repo_context.root_path.unwrap_or_else(|| PathBuf::from(".")),
                    user_question: question.to_string(),
                    consensus_response: final_answer.clone(),
                    timestamp: SystemTime::now(),
                    session_id: conversation_id.clone(),
                    git_commit: repo_context.git_info.and_then(|info| info.last_commit_hash),
                };
                
                // Check if auto-accept is enabled (passed via streaming callbacks)
                let auto_accept_enabled = self.callbacks.get_auto_accept_state().unwrap_or(false);
                
                tracing::info!("🤖 Auto-accept state: {}", auto_accept_enabled);
                
                // Parse operations first
                match file_executor.parse_operations_from_curator_response(
                    &final_answer,
                    &operation_context,
                ).await {
                    Ok(parsed_operations) => {
                        if parsed_operations.operations.is_empty() {
                            tracing::debug!("No file operations found in curator response");
                        } else {
                            tracing::info!(
                                "🤖 Found {} file operations with {}% confidence",
                                parsed_operations.operations.len(),
                                parsed_operations.confidence
                            );
                            
                            // Analyze operations with decision engine
                            let mut operations_to_execute = Vec::new();
                            let mut operations_requiring_confirmation = Vec::new();
                            let mut blocked_operations = Vec::new();
                            
                            for op_with_metadata in &parsed_operations.operations {
                                match file_executor.analyze_operation_decision(
                                    &op_with_metadata.operation,
                                    &operation_context,
                                ).await {
                                    Ok(decision) => {
                                        match decision {
                                            crate::consensus::smart_decision_engine::ExecutionDecision::AutoExecute { .. } => {
                                                if auto_accept_enabled {
                                                    operations_to_execute.push(op_with_metadata.operation.clone());
                                                } else {
                                                    operations_requiring_confirmation.push(op_with_metadata.clone());
                                                }
                                            }
                                            crate::consensus::smart_decision_engine::ExecutionDecision::RequireConfirmation { .. } => {
                                                operations_requiring_confirmation.push(op_with_metadata.clone());
                                            }
                                            crate::consensus::smart_decision_engine::ExecutionDecision::Block { .. } => {
                                                blocked_operations.push(op_with_metadata.clone());
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to analyze operation: {}", e);
                                        operations_requiring_confirmation.push(op_with_metadata.clone());
                                    }
                                }
                            }
                            
                            // Report on operations requiring confirmation
                            if !operations_requiring_confirmation.is_empty() {
                                tracing::info!(
                                    "⚠️ {} operations require user confirmation",
                                    operations_requiring_confirmation.len()
                                );
                                
                                // Send operations to UI for approval
                                self.callbacks.on_operations_require_confirmation(
                                    operations_requiring_confirmation.clone()
                                ).ok();
                            }
                            
                            // Report on blocked operations
                            if !blocked_operations.is_empty() {
                                tracing::warn!(
                                    "🛑 {} operations blocked for safety",
                                    blocked_operations.len()
                                );
                                for blocked in &blocked_operations {
                                    tracing::warn!("Blocked: {:?}", blocked.operation);
                                }
                            }
                            
                            // Execute auto-approved operations
                            if !operations_to_execute.is_empty() {
                                tracing::info!(
                                    "✅ Auto-executing {} approved operations",
                                    operations_to_execute.len()
                                );
                                
                                match file_executor.execute_operations_batch(
                                    operations_to_execute,
                                    &operation_context,
                                ).await {
                                    Ok(results) => {
                                        let successful = results.iter().filter(|r| r.success).count();
                                        let failed = results.iter().filter(|r| !r.success).count();
                                        
                                        tracing::info!(
                                            "🤖 File operations executed: {} successful, {} failed",
                                            successful,
                                            failed
                                        );
                                        
                                        // Log execution details
                                        for result in &results {
                                            if result.success {
                                                tracing::info!("✅ {} operation on {}: {}", 
                                                    match &result.operation {
                                        crate::consensus::stages::file_aware_curator::FileOperation::Create { .. } => "CREATE",
                                        crate::consensus::stages::file_aware_curator::FileOperation::Update { .. } => "UPDATE",
                                        crate::consensus::stages::file_aware_curator::FileOperation::Delete { .. } => "DELETE",
                                        crate::consensus::stages::file_aware_curator::FileOperation::Rename { .. } => "RENAME",
                                        crate::consensus::stages::file_aware_curator::FileOperation::Append { .. } => "APPEND",
                                    },
                                    result.files_affected.first()
                                        .map(|p| p.display().to_string())
                                        .unwrap_or_else(|| "unknown".to_string()),
                                    result.message
                                );
                                            } else {
                                                tracing::error!("❌ Failed operation: {}", 
                                                    result.error_message.as_ref().unwrap_or(&result.message));
                                            }
                                        }
                                        
                                        if failed > 0 {
                                            tracing::warn!(
                                                "⚠️ {} file operations failed. Check logs for details.",
                                                failed
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to execute operations batch: {}", e);
                                    }
                                }
                            } else if operations_to_execute.is_empty() && operations_requiring_confirmation.is_empty() {
                                tracing::info!("No operations to execute (all blocked or no operations found)");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("No file operations executed or parsing failed: {}", e);
                        // This is not a critical error - many curator responses don't include file operations
                    }
                }
            } else {
                tracing::debug!("No repository context available - skipping file operation execution");
            }
        } else {
            tracing::debug!("File executor not initialized - skipping file operation execution");
        }

        let result = ConsensusResult {
            success: true,
            result: Some(final_answer.clone()),
            error: None,
            stages: stage_results.clone(),
            conversation_id: conversation_id.clone(),
            total_duration: pipeline_start.elapsed().as_secs_f64(),
            total_cost,
        };

        // CRITICAL: Store conversation and curator result in database (like TypeScript implementation)
        if let Some(db) = &self.database {
            tracing::info!(
                "Database available, attempting to store consensus result for conversation {}",
                conversation_id
            );
            if let Err(e) = self
                .store_consensus_result(
                    &conversation_id,
                    question,
                    &final_answer,
                    &stage_results,
                    total_cost,
                    db.clone(),
                )
                .await
            {
                tracing::error!("Failed to store consensus result in database: {}", e);
                // Don't fail the entire consensus for storage errors, but log them
            }
        } else {
            tracing::warn!("No database available - consensus results will not be persisted!");
        }

        // TODO: Re-enable usage tracking when database is unified
        // Record usage after successful consensus completion
        // if let (Some(user_id), Some(usage_tracker)) = (user_id.as_ref(), &self.usage_tracker) {
        //     if let Err(e) = usage_tracker.record_conversation_usage(user_id, &conversation_id).await {
        //         tracing::error!("Failed to record conversation usage: {}", e);
        //         // Don't fail the consensus for usage tracking errors
        //     } else {
        //         tracing::info!("Recorded conversation usage for user {} (conversation: {})", user_id, conversation_id);
        //     }
        // }

        // Emit AfterConsensus hook event
        //         if let Some(hooks) = &self.hooks_system {
        //             let event = HookEvent::new(
        //                 EventType::AfterConsensus,
        //                 EventSource::Consensus { stage: "pipeline".to_string() }
        //             )
        //             .with_context("question", question)
        //             .with_context("conversation_id", &conversation_id)
        //             .with_context("final_answer", &final_answer)
        //             .with_context("total_duration", result.total_duration)
        //             .with_context("total_cost", result.total_cost)
        //             .with_context("stages_count", result.stages.len());
        //
        //             if let Err(e) = hooks.dispatch_event(event).await {
        //                 tracing::warn!("Failed to dispatch AfterConsensus hook event: {}", e);
        //             }
        //         }

        // Report conversation completion to D1 if we have authorization
        if let Some(auth) = d1_auth {
            let gateway = ConversationGateway::new()?;
            match gateway
                .report_conversation_completion(
                    &auth.conversation_token,
                    &conversation_id,
                    &auth.question_hash,
                )
                .await
            {
                Ok(verification) => {
                    if verification.verified {
                        tracing::info!(
                            "Conversation verified with D1 ({} remaining)",
                            verification.remaining_conversations
                        );
                    } else {
                        tracing::warn!("D1 verification failed but continuing");
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to verify conversation completion with D1: {}", e);
                    // Don't fail the whole consensus over verification
                }
            }
        }

        Ok(result)
    }

    /// Run a single stage of the pipeline
    async fn run_single_stage(
        &self,
        stage: Stage,
        handler: &dyn ConsensusStage,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
        conversation_id: &str,
        model: &str,
        cancellation_token: &CancellationToken,
    ) -> Result<StageResult> {
        // Check for cancellation at start of stage
        cancellation_token.throw_if_cancelled()?;
        
        let stage_start = Instant::now();
        let stage_id = Uuid::new_v4().to_string();

        // CRITICAL FIX: Dynamically choose the right curator based on question type
        let messages = if stage == Stage::Curator {
            // Check if this is a repository-related question
            let is_repo_related = self.verified_context_builder.is_repository_related_question(question);
            
            if is_repo_related {
                tracing::info!("🎨 Using FileAwareCuratorStage for repository-related question");
                handler.build_messages(question, previous_answer, context)?
            } else {
                tracing::info!("🎨 Using regular CuratorStage for general knowledge question");
                // Create a regular curator that doesn't force repository context
                let regular_curator = CuratorStage::new();
                regular_curator.build_messages(question, previous_answer, context)?
            }
        } else {
            // For non-curator stages, use the provided handler
            handler.build_messages(question, previous_answer, context)?
        };
        
        tracing::info!("🧠 {} stage: Using verified context (already filtered for relevance)", stage.display_name());
        
        // Debug log the messages being sent to the AI
        if stage == Stage::Generator {
            for (idx, msg) in messages.iter().enumerate() {
                let preview = if msg.content.len() > 200 {
                    format!("{}...", &msg.content[..200])
                } else {
                    msg.content.clone()
                };
                tracing::debug!("Message {} ({}): {}", idx, msg.role, preview);
            }
        }

        // Create progress tracker
        let mut tracker = ProgressTracker::new(stage, self.callbacks.clone());

        // Call model with retry logic for fallback models
        let mut response = self
            .call_model(model, &messages, &mut tracker, cancellation_token)
            .await
            .with_context(|| {
                format!(
                    "Failed to call model {} for {}",
                    model,
                    stage.display_name()
                )
            })?;

        // Check if we need to retry with a different model
        if response.content.starts_with("RETRY_WITH_MODEL:") {
            let replacement_model = response.content.trim_start_matches("RETRY_WITH_MODEL:");
            tracing::info!("Retrying with replacement model: {}", replacement_model);

            // Call with replacement model
            response = self
                .call_model(replacement_model, &messages, &mut tracker, cancellation_token)
                .await
                .with_context(|| {
                    format!(
                        "Failed to call replacement model {} for {}",
                        replacement_model,
                        stage.display_name()
                    )
                })?;
        }

        // Mark complete
        tracker.complete()?;

        // Build stage result
        let stage_result = StageResult {
            stage_id,
            stage_name: stage.as_str().to_string(),
            question: question.to_string(),
            answer: response.content,
            model: response.model,
            conversation_id: conversation_id.to_string(),
            timestamp: Utc::now(),
            usage: Some(response.usage),
            analytics: Some(StageAnalytics {
                duration: stage_start.elapsed().as_secs_f64(),
                cost: response.analytics.cost,
                input_cost: response.analytics.input_cost,
                output_cost: response.analytics.output_cost,
                provider: response.analytics.provider,
                model_internal_id: response.analytics.model_internal_id,
                quality_score: response.analytics.quality_score,
                error_count: response.analytics.error_count,
                fallback_used: response.analytics.fallback_used,
                rate_limit_hit: response.analytics.rate_limit_hit,
                retry_count: response.analytics.retry_count,
                start_time: Utc::now()
                    - chrono::Duration::seconds(stage_start.elapsed().as_secs() as i64),
                end_time: Utc::now(),
                memory_usage: None,
                features: response.analytics.features,
            }),
        };

        Ok(stage_result)
    }

    /// Build full context combining semantic, temporal, memory, and codebase intelligence
    async fn build_full_context(
        &self,
        semantic_context: Option<String>,
        temporal_context: Option<crate::consensus::temporal::TemporalContext>,
        memory_context: Option<String>,
        question: &str,
        codebase_intelligence: Option<Arc<crate::consensus::codebase_intelligence::CodebaseIntelligence>>,
    ) -> Result<Option<String>> {
        let mut contexts = Vec::new();

        // Add memory context first (most important for multi-conversation intelligence)
        // This is the AUTHORITATIVE knowledge from previous curator results
        if let Some(memory) = memory_context {
            if !memory.is_empty() {
                contexts.push(memory); // Already formatted with headers
            }
        }

        // NEW: Add semantic codebase search results if available
        if let Some(intelligence) = codebase_intelligence {
            if intelligence.has_analysis().await {
                tracing::info!("🔍 Searching indexed codebase for: '{}'", question);
                match intelligence.get_context_for_question(question).await {
                    Ok(codebase_context) => {
                        if !codebase_context.is_empty() {
                            contexts.push(format!("## 🔍 SEMANTIC CODEBASE SEARCH RESULTS\n{}", codebase_context));
                            tracing::info!("Added semantic search results to context");
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to search codebase: {}", e);
                    }
                }
            }
        }

        // Add repository context (current codebase understanding)
        if let Some(repo_ctx) = &self.repository_context {
            let repo_context_info = repo_ctx.get_context_for_prompts().await;
            tracing::info!("Repository context info: '{}'", repo_context_info);
            if !repo_context_info.is_empty() {
                contexts.push(format!("## Repository Context\n{}", repo_context_info));
                tracing::info!("Added repository context to pipeline");
            } else {
                tracing::warn!("Repository context is empty - no project information available");
            }
        } else {
            tracing::warn!("No repository context manager available in pipeline");
        }

        if let Some(temporal) = temporal_context {
            contexts.push(format!(
                "## Temporal Context\n{}\n{}",
                temporal.search_instruction, temporal.temporal_awareness
            ));
        }

        if let Some(semantic) = semantic_context {
            contexts.push(format!("## Semantic Context\n{}", semantic));
        }

        if contexts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(contexts.join("\n\n")))
        }
    }

    /// Call model via OpenRouter API with streaming support
    async fn call_model(
        &self,
        model: &str,
        messages: &[crate::consensus::types::Message],
        tracker: &mut ProgressTracker,
        cancellation_token: &CancellationToken,
    ) -> Result<ModelResponse> {
        // Check for cancellation before making API call
        cancellation_token.throw_if_cancelled()?;
        
        // Also check if we should stop early
        if cancellation_token.is_cancelled() {
            tracing::info!("🛑 Model call cancelled before starting");
            return Err(anyhow::anyhow!("Operation was cancelled"));
        }
        
        // Check if OpenRouter client is available
        let openrouter_client = self.openrouter_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenRouter client not initialized. Please set OPENROUTER_API_KEY environment variable."))?;

        // Convert consensus messages to OpenRouter format
        let openrouter_messages: Vec<OpenRouterMessage> = messages
            .iter()
            .map(|msg| OpenRouterMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        // Configure request
        let request = OpenRouterRequest {
            model: model.to_string(),
            messages: openrouter_messages,
            temperature: Some(0.7),
            max_tokens: Some(4000), // Matching TypeScript hardcoded value
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: Some(self.config.enable_streaming),
            provider: None,
        };

        // Make the API call
        if self.config.enable_streaming {
            // Create a custom callback that forwards to our tracker
            let stage = tracker.stage.clone();
            let tracker_callbacks = tracker.callbacks.clone();

            let callbacks = Box::new(TrackerForwardingCallbacks {
                tracker_callbacks,
                stage,
            }) as Box<dyn OpenRouterStreamingCallbacks>;

            match openrouter_client
                .chat_completion_stream(request, Some(callbacks), Some(cancellation_token))
                .await
            {
                Ok(response_content) => {
                    // The streaming callbacks have already been called during streaming
                    // Just update the final content
                    tracker.content = response_content.clone();

                    // For streaming, we need to estimate tokens since we don't get usage from API
                    let estimated_prompt_tokens = messages.iter()
                        .map(|m| m.content.len() / 4) // Rough estimate: 1 token = 4 chars
                        .sum::<usize>() as u32;
                    let estimated_completion_tokens = (response_content.len() / 4) as u32;
                    
                    // Calculate cost using database pricing
                    let cost_result = if let Some(db) = &self.database {
                        db.calculate_model_cost(model, estimated_prompt_tokens, estimated_completion_tokens).await
                    } else {
                        Ok(0.0)
                    };
                    
                    let (total_cost, input_cost, output_cost) = match cost_result {
                        Ok(cost) => {
                            // Calculate component costs
                            let input_cost = if let Some(db) = &self.database {
                                db.calculate_model_cost(model, estimated_prompt_tokens, 0).await.unwrap_or(0.0)
                            } else {
                                0.0
                            };
                            let output_cost = if let Some(db) = &self.database {
                                db.calculate_model_cost(model, 0, estimated_completion_tokens).await.unwrap_or(0.0)
                            } else {
                                0.0
                            };
                            (cost, input_cost, output_cost)
                        }
                        Err(e) => {
                            tracing::warn!("Failed to calculate cost for {}: {}", model, e);
                            (0.0, 0.0, 0.0)
                        }
                    };
                    
                    tracing::info!(
                        "💰 Streaming response - Model: {}, Estimated tokens: {} input, {} output, Cost: ${:.8}",
                        model, estimated_prompt_tokens, estimated_completion_tokens, total_cost
                    );

                    Ok(ModelResponse {
                        model: model.to_string(),
                        content: response_content,
                        usage: TokenUsage {
                            prompt_tokens: estimated_prompt_tokens,
                            completion_tokens: estimated_completion_tokens,
                            total_tokens: estimated_prompt_tokens + estimated_completion_tokens,
                        },
                        analytics: StageAnalytics {
                            duration: 0.0,
                            cost: total_cost,
                            input_cost,
                            output_cost,
                            provider: "openrouter".to_string(),
                            model_internal_id: model.to_string(),
                            quality_score: 1.0,
                            error_count: 0,
                            fallback_used: false,
                            rate_limit_hit: false,
                            retry_count: 0,
                            start_time: Utc::now(),
                            end_time: Utc::now(),
                            memory_usage: None,
                            features: crate::consensus::types::AnalyticsFeatures {
                                streaming: self.config.enable_streaming,
                                routing_variant: "balanced".to_string(),
                                optimization_applied: Some(true),
                            },
                        },
                    })
                }
                Err(e) => self.handle_api_error(e, tracker, model).await,
            }
        } else {
            // Use non-streaming
            match openrouter_client.chat_completion(request).await {
                Ok(response) => {
                    let content = response
                        .choices
                        .first()
                        .and_then(|choice| choice.message.as_ref())
                        .map(|message| message.content.clone())
                        .unwrap_or_else(|| "No response content".to_string());

                    // Add response to tracker
                    tracker.add_chunk(&content)?;

                    let usage = response.usage.map(|u| TokenUsage {
                        prompt_tokens: u.prompt_tokens,
                        completion_tokens: u.completion_tokens,
                        total_tokens: u.total_tokens,
                    });

                    // Calculate cost using real database pricing - NO FALLBACKS
                    let (cost, input_cost, output_cost) = if let Some(ref usage) = usage {
                        if let Some(ref db) = self.database {
                        tracing::info!(
                            "💰 Calculating cost for model: {} (prompt: {}, completion: {})",
                            model,
                            usage.prompt_tokens,
                            usage.completion_tokens
                        );
                        
                        match db.calculate_model_cost(
                            model,
                            usage.prompt_tokens,
                            usage.completion_tokens,
                        ).await {
                            Ok(calculated_cost) => {
                                // Also calculate component costs
                                let input_cost = db.calculate_model_cost(model, usage.prompt_tokens, 0).await.unwrap_or(0.0);
                                let output_cost = db.calculate_model_cost(model, 0, usage.completion_tokens).await.unwrap_or(0.0);
                                
                                tracing::info!(
                                    "💰 Database cost calculation: ${:.8} for {} ({} input + {} output tokens)",
                                    calculated_cost,
                                    model,
                                    usage.prompt_tokens,
                                    usage.completion_tokens
                                );
                                (calculated_cost, input_cost, output_cost)
                            },
                            Err(e) => {
                                tracing::error!(
                                    "💰 ERROR: Failed to calculate model cost for {}: {}",
                                    model,
                                    e
                                );
                                // Return error instead of fallback
                                return Err(anyhow::anyhow!("Cost calculation failed: {}", e));
                            }
                        }
                        } else {
                            tracing::error!("💰 ERROR: No database available for cost calculation");
                            return Err(anyhow::anyhow!("Cannot calculate cost without database"));
                        }
                    } else {
                        tracing::error!("💰 ERROR: No usage data available for cost calculation");
                        return Err(anyhow::anyhow!("Cannot calculate cost without usage data"));
                    };

                    Ok(ModelResponse {
                        model: model.to_string(),
                        content,
                        usage: usage.unwrap_or(TokenUsage {
                            prompt_tokens: 0,
                            completion_tokens: 0,
                            total_tokens: 0,
                        }),
                        analytics: StageAnalytics {
                            duration: 0.0,
                            cost,
                            input_cost,
                            output_cost,
                            provider: "openrouter".to_string(),
                            model_internal_id: model.to_string(),
                            quality_score: 1.0,
                            error_count: 0,
                            fallback_used: false,
                            rate_limit_hit: false,
                            retry_count: 0,
                            start_time: Utc::now(),
                            end_time: Utc::now(),
                            memory_usage: None,
                            features: crate::consensus::types::AnalyticsFeatures {
                                streaming: self.config.enable_streaming,
                                routing_variant: "balanced".to_string(),
                                optimization_applied: Some(true),
                            },
                        },
                    })
                }
                Err(e) => self.handle_api_error(e, tracker, model).await,
            }
        }
    }

    /// Handle API errors
    async fn handle_api_error(
        &self,
        error: anyhow::Error,
        tracker: &mut ProgressTracker,
        model: &str,
    ) -> Result<ModelResponse> {
        let error_str = error.to_string();
        
        // Check if this is a cancellation error first
        if error_str.contains("cancelled") || error_str.contains("Cancelled") {
            tracing::info!("🛑 Consensus cancelled during API call");
            // Return the cancellation error immediately
            return Err(error);
        }
        
        // Check if this is a model unavailable error
        let is_model_unavailable = error_str.contains("404")
            || error_str.contains("model not found")
            || error_str.contains("model unavailable");

        if is_model_unavailable {
            tracing::warn!(
                "Model {} is unavailable, attempting to find replacement",
                model
            );

            // Try to find a replacement model
            if let Some(db) = &self.database {
                use crate::consensus::maintenance::TemplateMaintenanceManager;

                let maintenance = TemplateMaintenanceManager::new(db.clone(), self.api_key.clone());

                if let Ok(Some(replacement)) = maintenance.find_model_replacement(model).await {
                    tracing::info!("Found replacement model: {} -> {}", model, replacement);

                    // Return a special response indicating we need to retry with a different model
                    return Ok(ModelResponse {
                        model: replacement.clone(),
                        content: format!("RETRY_WITH_MODEL:{}", replacement),
                        usage: TokenUsage {
                            prompt_tokens: 0,
                            completion_tokens: 0,
                            total_tokens: 0,
                        },
                        analytics: StageAnalytics {
                            duration: 0.0,
                            cost: 0.0,
                            input_cost: 0.0,
                            output_cost: 0.0,
                            provider: "fallback".to_string(),
                            model_internal_id: "0".to_string(),
                            quality_score: 0.0,
                            error_count: 1,
                            fallback_used: true,
                            rate_limit_hit: false,
                            retry_count: 0,
                            start_time: Utc::now(),
                            end_time: Utc::now(),
                            memory_usage: None,
                            features: crate::consensus::types::AnalyticsFeatures {
                                streaming: self.config.enable_streaming,
                                routing_variant: "fallback".to_string(),
                                optimization_applied: Some(false),
                            },
                        },
                    });
                }
            }
        }

        // For other errors or if no replacement found, return the error
        Err(anyhow::anyhow!(
            "OpenRouter API call failed for model {}: {}",
            model,
            error
        ))
    }

    /// Estimate cost for a stage operation
    async fn estimate_stage_cost(&self, stage: Stage, model: &str, question: &str) -> Result<f64> {
        // Simple cost estimation based on question length and model
        let question_tokens = question.split_whitespace().count() as u32;
        let estimated_tokens = match stage {
            Stage::Generator => question_tokens + 500, // Generator usually produces more tokens
            Stage::Refiner => question_tokens + 200,   // Refiner improves existing content
            Stage::Validator => question_tokens + 100, // Validator checks quality
            Stage::Curator => question_tokens + 150,   // Curator polishes result
        };

        // Model-based cost calculation (simplified)
        let cost_per_token = match model {
            m if m.contains("gpt-4") => 0.00003,
            m if m.contains("gpt-3.5") => 0.000002,
            m if m.contains("claude-3-opus") => 0.000075,
            m if m.contains("claude-3-sonnet") => 0.000015,
            m if m.contains("claude-3-haiku") => 0.000001,
            _ => 0.00001, // Default fallback
        };

        Ok(estimated_tokens as f64 * cost_per_token)
    }

    /// Store consensus result in database (matching TypeScript implementation)
    async fn store_consensus_result(
        &self,
        conversation_id: &str,
        question: &str,
        final_answer: &str,
        stage_results: &[StageResult],
        total_cost: f64,
        database: Arc<DatabaseManager>,
    ) -> Result<()> {
        tracing::info!(
            "Starting to store consensus result for conversation {}",
            conversation_id
        );
        tracing::debug!("Question: {}", question);
        tracing::debug!("Number of stage results: {}", stage_results.len());
        tracing::debug!("Profile ID: {}", self.profile.id);

        let now = Utc::now().to_rfc3339();
        let mut conn = database.get_connection()?;
        tracing::debug!("Got database connection");

        // Use spawn_blocking for the entire database transaction
        let conversation_id = conversation_id.to_string();
        let question = question.to_string();
        let final_answer = final_answer.to_string();
        let stage_results = stage_results.to_vec();
        let profile_id = self.profile.id.clone(); // Capture profile ID before closure

        tokio::task::spawn_blocking(move || -> Result<()> {
            tracing::debug!("Inside spawn_blocking, starting transaction");
            let tx = conn.transaction()?;

            // 1. Store conversation record
            tracing::debug!("Storing conversation record with ID: {}", conversation_id);
            
            // Calculate total tokens
            let total_input_tokens: u32 = stage_results
                .iter()
                .map(|s| s.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0))
                .sum();
            let total_output_tokens: u32 = stage_results
                .iter()
                .map(|s| s.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0))
                .sum();
                
            tracing::info!(
                "💰 Storing conversation - Total cost: ${:.8}, Input tokens: {}, Output tokens: {}",
                total_cost, total_input_tokens, total_output_tokens
            );
            
            let rows_affected = tx.execute(
                "INSERT OR REPLACE INTO conversations (
                    id, user_id, title, profile_id, context_type,
                    total_cost, total_tokens_input, total_tokens_output,
                    performance_score, quality_rating, is_archived, is_favorite,
                    tags, metadata, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    conversation_id,
                    None::<String>, // user_id is optional, like TypeScript version
                    question.chars().take(100).collect::<String>(), // title from question (truncated)
                    profile_id,     // Use the actual profile ID from the pipeline
                    "consensus",    // context_type
                    total_cost,
                    total_input_tokens,
                    total_output_tokens,
                    0.95f64,       // performance_score (default high)
                    5i32,          // quality_rating (default excellent)
                    0i32,          // is_archived (false)
                    0i32,          // is_favorite (false)
                    "[]",          // tags (empty JSON array)
                    "{}",          // metadata (empty JSON object)
                    &now,          // created_at
                    &now           // updated_at
                ],
            )?;
            tracing::debug!(
                "Conversation record stored, {} rows affected",
                rows_affected
            );

            // 2. Store all stage outputs as messages for audit trail
            for stage_result in &stage_results {
                // Store user message (question) - only for first stage
                if stage_result.stage_name == "generator" {
                    tx.execute(
                        "INSERT INTO messages (
                            id, conversation_id, role, content, stage, model_used, 
                            timestamp
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            uuid::Uuid::new_v4().to_string(),
                            conversation_id,
                            "user",
                            question,
                            None::<String>, // User message has no stage
                            None::<String>, // User message has no model
                            &now
                        ],
                    )?;
                }

                // Store assistant message for each stage
                let sequence_number = match stage_result.stage_name.as_str() {
                    "generator" => 1,
                    "refiner" => 2,
                    "validator" => 3,
                    "curator" => 4,
                    _ => 5,
                };
                
                let stage_cost = stage_result.analytics.as_ref().map(|a| a.cost).unwrap_or(0.0);
                let input_tokens = stage_result.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0) as i32;
                let output_tokens = stage_result.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0) as i32;
                
                tracing::debug!(
                    "💰 Storing stage {} message - Cost: ${:.8}, Input: {}, Output: {}",
                    stage_result.stage_name, stage_cost, input_tokens, output_tokens
                );
                
                tx.execute(
                    "INSERT INTO messages (
                        id, conversation_id, role, content, stage, model_used,
                        timestamp
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        conversation_id,
                        "assistant",
                        stage_result.answer,
                        stage_result.stage_name,
                        stage_result.model,
                        &now
                    ],
                )?;
            }

            // 3. Store cost tracking records for each stage
            for stage_result in &stage_results {
                if let Some(analytics) = &stage_result.analytics {
                    let input_tokens = stage_result.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0) as i32;
                    let output_tokens = stage_result.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0) as i32;
                    
                    // Get model internal_id from openrouter_models table
                    let model_id: Option<i32> = tx.query_row(
                        "SELECT internal_id FROM openrouter_models WHERE openrouter_id = ?1",
                        params![&stage_result.model],
                        |row| row.get(0),
                    ).optional()?;
                    
                    if let Some(model_id) = model_id {
                        tx.execute(
                            "INSERT INTO cost_tracking (
                                user_id, conversation_id, model_id, 
                                tokens_input, tokens_output,
                                cost_input, cost_output, total_cost,
                                created_at
                            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            params![
                                None::<String>, // user_id (optional)
                                conversation_id,
                                model_id,
                                input_tokens,
                                output_tokens,
                                analytics.input_cost,
                                analytics.output_cost,
                                analytics.cost,
                                &now
                            ],
                        )?;
                        
                        tracing::debug!(
                            "💰 Cost tracking record added for {} - Total: ${:.8}",
                            stage_result.model, analytics.cost
                        );
                    }
                }
            }
            
            // 4. Store in knowledge_conversations (extended format matching TypeScript)
            tx.execute(
                "INSERT OR REPLACE INTO knowledge_conversations (
                    id, conversation_id, question, final_answer, source_of_truth,
                    conversation_context, profile_id, created_at, last_updated
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    conversation_id,
                    question,
                    final_answer,
                    final_answer,   // Final answer is the "source of truth" from Curator
                    None::<String>, // TODO: Add conversation context if available
                    profile_id,     // Use the actual profile ID
                    &now,
                    &now
                ],
            )?;

            // 4. Store conversation usage for tracking - using current license key
            // Get the currently configured license key first
            let current_license_key: Option<String> = tx
                .query_row(
                    "SELECT value FROM configurations WHERE key = 'hive_license_key'",
                    [],
                    |row| row.get(0),
                )
                .optional()?;

            // Then get the user_id that matches the current license key
            let user_id_result: Option<String> = if let Some(license_key) = current_license_key {
                tx.query_row(
                    "SELECT id FROM users WHERE license_key = ?1",
                    params![&license_key],
                    |row| row.get(0),
                )
                .optional()?
            } else {
                None
            };

            if let Some(user_id) = user_id_result {
                // Check if we already recorded this at the start (to avoid duplicates)
                let already_recorded: bool = tx.query_row(
                    "SELECT 1 FROM conversation_usage WHERE conversation_id = ?1 AND user_id = ?2",
                    params![conversation_id, &user_id],
                    |_| Ok(true)
                ).unwrap_or(false);

                if !already_recorded {
                    // Insert into conversation_usage table for tracking
                    tx.execute(
                        "INSERT INTO conversation_usage (
                            conversation_id, user_id, timestamp
                        ) VALUES (?1, ?2, ?3)",
                        params![conversation_id, user_id, &now],
                    )?;
                    tracing::info!("Recorded conversation usage for user {}", user_id);
                } else {
                    tracing::debug!("Conversation usage already recorded for user {}", user_id);
                }
            } else {
                tracing::warn!("No user found with license key, skipping usage tracking");
            }

            // 5. Store curator truth (flagged as source of truth - critical for memory system)
            tracing::debug!("Looking for curator stage result");
            let curator_result = stage_results
                .iter()
                .find(|s| s.stage_name == "curator")
                .ok_or_else(|| {
                    tracing::error!(
                        "No curator stage result found! Stage names: {:?}",
                        stage_results
                            .iter()
                            .map(|s| &s.stage_name)
                            .collect::<Vec<_>>()
                    );
                    anyhow::anyhow!("No curator stage result found")
                })?;
            tracing::debug!("Found curator result");

            let confidence_score = curator_result
                .analytics
                .as_ref()
                .map(|a| a.quality_score)
                .unwrap_or(0.8); // Default confidence

            let topic_summary = Self::extract_topic_summary(&question, &final_answer);

            tx.execute(
                "INSERT OR REPLACE INTO curator_truths (
                    conversation_id, curator_output, confidence_score, topic_summary, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    conversation_id,
                    curator_result.answer,
                    confidence_score,
                    topic_summary,
                    &now
                ],
            )?;

            tracing::debug!("Committing transaction");
            tx.commit()?;
            tracing::info!(
                "Successfully stored consensus result for conversation {} in database",
                conversation_id
            );
            Ok(())
        })
        .await??;

        // Trigger analytics refresh now that new data is in the database
        if let Err(e) = self.callbacks.on_analytics_refresh() {
            tracing::warn!("Failed to trigger analytics refresh: {}", e);
        } else {
            tracing::info!("Analytics refresh triggered after consensus completion");
        }

        Ok(())
    }

    /// Extract topic summary from question and answer (simple implementation)
    fn extract_topic_summary(question: &str, answer: &str) -> String {
        // Simple topic extraction - in production this would use AI or NLP
        let combined = format!("{} {}", question, answer);
        let words: Vec<&str> = combined
            .split_whitespace()
            .filter(|w| w.len() > 4) // Filter short words
            .take(5) // Take first 5 meaningful words
            .collect();

        if words.is_empty() {
            "general_topic".to_string()
        } else {
            words.join("_").to_lowercase()
        }
    }

    /// Get memory context for question (temporal + thematic)
    async fn get_memory_context(&self, question: &str) -> Result<Option<String>> {
        // Prefer ConsensusMemory if available (AI-enhanced context)
        if let Some(ref consensus_memory) = self.consensus_memory {
            match consensus_memory.context_injector
                .inject_context(question, crate::consensus::types::Stage::Generator, None)
                .await {
                Ok(injected_context) => {
                    if injected_context.formatted_context.is_empty() {
                        tracing::info!("ConsensusMemory returned empty context, falling back to direct DB");
                        // Fall back to direct database query
                        if let Some(db) = &self.database {
                            let context = self.build_memory_context(question, db.clone()).await?;
                            if context.is_empty() {
                                Ok(None)
                            } else {
                                Ok(Some(context))
                            }
                        } else {
                            Ok(None)
                        }
                    } else {
                        tracing::info!(
                            "✅ Using AI-enhanced memory context: {} facts injected",
                            injected_context.facts.len()
                        );
                        Ok(Some(injected_context.formatted_context))
                    }
                }
                Err(e) => {
                    tracing::warn!("ConsensusMemory failed: {}, falling back to direct DB", e);
                    // Fall back to direct database query
                    if let Some(db) = &self.database {
                        let context = self.build_memory_context(question, db.clone()).await?;
                        if context.is_empty() {
                            Ok(None)
                        } else {
                            Ok(Some(context))
                        }
                    } else {
                        Ok(None)
                    }
                }
            }
        } else if let Some(db) = &self.database {
            // No ConsensusMemory available, use direct database query
            let context = self.build_memory_context(question, db.clone()).await?;
            if context.is_empty() {
                Ok(None)
            } else {
                Ok(Some(context))
            }
        } else {
            Ok(None)
        }
    }

    /// Build memory context combining recent and thematic memories (matching TypeScript)
    async fn build_memory_context(
        &self,
        query: &str,
        database: Arc<DatabaseManager>,
    ) -> Result<String> {
        tracing::debug!("Building memory context for query: {}", query);
        let mut conn = database.get_connection()?;
        let query = query.to_string();

        // Use spawn_blocking for database queries
        let results = tokio::task::spawn_blocking(move || -> Result<String> {
            let mut context_parts = Vec::new();

            // TEMPORAL PRIORITY SYSTEM (matches TypeScript implementation)
            // Priority 1: Recent conversations (24h) - NEWEST FIRST
            let twenty_four_hours_ago = (Utc::now() - chrono::Duration::hours(24)).to_rfc3339();
            tracing::debug!("Looking for recent curator knowledge since: {}", twenty_four_hours_ago);

            // First check if this is a follow-up query (simple pattern matching)
            let query_lower = query.to_lowercase();
            let is_follow_up = [
                "give me", "tell me more", "can you explain", "show me", "what about",
                "examples", "code example", "how does", " it ", " that ", " this ",
                " those ", " these ", " them ", " they "
            ].iter().any(|pattern| query_lower.contains(pattern));

            tracing::debug!("Is follow-up query: {}", is_follow_up);

            // TEMPORAL SEARCH: One conversation at a time, most recent first
            // Step 1: Search backwards through time, one conversation at a time
            let mut found_context = false;
            let mut selected_memory: Option<(String, String, f64, String)> = None;

            // Get ALL conversations from the last 24 hours, ordered by recency
            let mut stmt = conn.prepare(
                "SELECT source_of_truth, question, 1.0 as confidence_score, created_at
                 FROM knowledge_conversations
                 WHERE created_at > ?1
                 ORDER BY created_at DESC"
            )?;

            let all_recent_results = stmt.query_map([&twenty_four_hours_ago], |row| {
                Ok((
                    row.get::<_, String>(0)?, // curator_output
                    row.get::<_, String>(1)?, // question
                    row.get::<_, f64>(2)?,    // confidence_score
                    row.get::<_, String>(3)?, // created_at
                ))
            })?;

            // Search one by one until we find relevant context
            for result in all_recent_results {
                let (curator_output, question, confidence, created_at) = result?;

                // For follow-up queries, the most recent is ALWAYS relevant
                if is_follow_up && selected_memory.is_none() {
                    tracing::debug!("Follow-up detected - using most recent conversation: {}", question);
                    selected_memory = Some((curator_output, question, confidence, created_at));
                    found_context = true;
                    break;
                }

                // For non-follow-up, check if this conversation might be relevant
                let query_words: Vec<String> = query.to_lowercase()
                    .split_whitespace()
                    .filter(|w| w.len() > 2)
                    .map(|w| w.to_string())
                    .collect();

                let conversation_relevant = query_words.iter().any(|word| {
                    question.to_lowercase().contains(word) ||
                    curator_output.to_lowercase().contains(word)
                });

                if conversation_relevant {
                    tracing::debug!("Found relevant conversation: {}", question);
                    selected_memory = Some((curator_output, question, confidence, created_at));
                    found_context = true;
                    break;
                }
            }

            // If we found context in recent conversations, use it
            if found_context && selected_memory.is_some() {
                let (curator_output, question, _, created_at) = selected_memory.unwrap();
                let date = chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|_| "recent".to_string());

                context_parts.push("🧠 AUTHORITATIVE MEMORY CONTEXT:".to_string());
                context_parts.push("Here is the most relevant recent conversation:".to_string());
                context_parts.push("".to_string());
                context_parts.push(format!(
                    "From {}: Question: {}\n\nVerified Answer:\n{}\n\n---",
                    date, question, curator_output
                ));
                context_parts.push("".to_string());

                if is_follow_up {
                    context_parts.push("🚨 FOLLOW-UP DETECTED: This is a follow-up question.".to_string());
                    context_parts.push("You MUST provide examples/clarification about the topic above.".to_string());
                    context_parts.push("Do NOT reference any other topics or previous conversations.".to_string());
                } else {
                    context_parts.push("⚡ Use this context to inform your response.".to_string());
                }

                return Ok(context_parts.join("\n"));
            }

            tracing::debug!("No relevant context found in recent 24h conversations, falling back to thematic search");

            // If follow-up but no 24h context, try broader window (72 hours)
            if is_follow_up {
                let seventy_two_hours_ago = (Utc::now() - chrono::Duration::hours(72)).to_rfc3339();
                tracing::debug!("Follow-up detected, checking broader window (72h)");

                let mut stmt = conn.prepare(
                    "SELECT source_of_truth, question, 1.0 as confidence_score, created_at
                     FROM knowledge_conversations
                     WHERE created_at > ?1 AND created_at <= ?2
                     ORDER BY created_at DESC
                     LIMIT 3"
                )?;

                let broader_results = stmt.query_map(
                    rusqlite::params![&seventy_two_hours_ago, &twenty_four_hours_ago],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?, // source_of_truth (curator output)
                            row.get::<_, String>(1)?, // question
                            row.get::<_, f64>(2)?,    // confidence_score
                            row.get::<_, String>(3)?, // created_at
                        ))
                    }
                )?;

                let mut broader_memories = Vec::new();
                for result in broader_results {
                    broader_memories.push(result?);
                }

                if !broader_memories.is_empty() {
                    context_parts.push("🔄 RECENT FOLLOW-UP CONTEXT (72h):".to_string());
                    for (i, (curator_output, question, _confidence, created_at)) in broader_memories.iter().enumerate() {
                        let date = chrono::DateTime::parse_from_rfc3339(created_at)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_else(|_| "recent".to_string());

                        context_parts.push(format!(
                            "RECENT CONVERSATION {} (from {}):\nQuestion: {}\n\nAnswer:\n{}\n\n---",
                            i + 1, date, question, curator_output
                        ));
                    }

                    return Ok(context_parts.join("\n"));
                }
            }

            // 2. THEMATIC SEARCH: Search ALL memory, no time limits, but ordered by recency
            tracing::debug!("Starting thematic search for: {}", query);
            let keywords: Vec<String> = query
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .take(5) // Take more keywords for better matching
                .map(|w| format!("%{}%", w.to_lowercase()))
                .collect();

            if !keywords.is_empty() {
                let keyword_query = keywords.iter()
                    .map(|_| "(source_of_truth LIKE ? OR question LIKE ?)")
                    .collect::<Vec<_>>()
                    .join(" OR ");

                // Search ALL history, no time limit, ordered by recency
                let full_query = format!(
                    "SELECT DISTINCT source_of_truth, question, 0.8 as confidence_score, created_at
                     FROM knowledge_conversations
                     WHERE {}
                     ORDER BY created_at DESC",
                    keyword_query
                );

                let mut stmt = conn.prepare(&full_query)?;

                // Double the keywords for both source_of_truth and question matching
                let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
                for keyword in &keywords {
                    params.push(keyword as &dyn rusqlite::ToSql);
                    params.push(keyword as &dyn rusqlite::ToSql);
                }

                let thematic_results = stmt.query_map(&params[..], |row| {
                    Ok((
                        row.get::<_, String>(0)?, // curator_output
                        row.get::<_, String>(1)?, // question
                        row.get::<_, f64>(2)?,    // confidence_score
                        row.get::<_, String>(3)?, // created_at
                    ))
                })?;

                // Process thematic results one by one, most recent first
                let mut found_thematic = false;
                for result in thematic_results {
                    let (curator_output, question, _, created_at) = result?;

                    // Skip if this was already checked in temporal search
                    let created_dt = chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now());
                    if created_dt > chrono::Utc::now() - chrono::Duration::hours(24) {
                        continue; // Already checked in temporal search
                    }

                    // For thematic search, be more selective - require stronger match
                    let strong_match = keywords.iter()
                        .filter(|k| k.len() > 5) // Only check meaningful keywords
                        .any(|keyword| {
                            let clean_keyword = keyword.replace("%", "").to_lowercase();
                            question.to_lowercase().contains(&clean_keyword) ||
                            curator_output.to_lowercase().contains(&clean_keyword)
                        });

                    if strong_match {
                        tracing::debug!("Found thematic match: {}", question);
                        let date = created_dt.format("%Y-%m-%d").to_string();

                        context_parts.push("📚 THEMATIC CONTEXT:".to_string());
                        context_parts.push(format!(
                            "Related conversation from {}: Question: {}\n\nAnswer:\n{}\n\n---",
                            date, question, curator_output
                        ));
                        context_parts.push("".to_string());
                        context_parts.push("⚡ This is older context that may be relevant to your query.".to_string());

                        found_thematic = true;
                        break; // Use only the most recent thematic match
                    }
                }

                if !found_thematic && is_follow_up {
                    // For follow-ups with no context found, provide a clear message
                    context_parts.push("⚠️ NO RECENT CONTEXT FOUND".to_string());
                    context_parts.push("This appears to be a follow-up question, but no recent conversation context was found.".to_string());
                    context_parts.push("Please provide a complete, standalone answer.".to_string());
                }
            }

            Ok(context_parts.join("\n"))
        }).await??;

        Ok(results)
    }

    /// Check if we should use file reading for this question
    async fn should_use_file_reading(&self, question: &str, context: Option<&str>) -> bool {
        // Store repository availability for reuse
        let has_repository = self.repository_context.is_some();

        // NEW APPROACH: If codebase has been indexed with @codebase, ALWAYS search it
        // The AI will determine relevance based on what it finds
        if let Some(ctx) = context {
            if ctx.contains("SEMANTIC CODEBASE SEARCH RESULTS") {
                tracing::info!("🧠 Using indexed codebase knowledge for: '{}'", question);
                return true;
            }
        }
        
        // Use multi-stage AI intelligence for sophisticated context analysis
        if let Some(ai_helpers) = &self.ai_helpers {
            let orchestrator = &ai_helpers.intelligent_orchestrator;
            match orchestrator.make_intelligent_context_decision(question, has_repository).await {
                Ok(decision) => {
                    if decision.should_use_repo {
                        tracing::info!("🧠 Multi-AI analysis: Using repository context for: '{}' (category: {:?}, confidence: {:.2})", 
                            question, decision.primary_category, decision.confidence);
                        tracing::debug!("🔍 AI reasoning: {}", decision.reasoning);
                    } else {
                        tracing::info!("🧠 Multi-AI analysis: Using general knowledge for: '{}' (category: {:?}, confidence: {:.2})", 
                            question, decision.primary_category, decision.confidence);
                        tracing::debug!("🔍 AI reasoning: {}", decision.reasoning);
                    }
                    return decision.should_use_repo;
                }
                Err(e) => {
                    tracing::warn!("Failed to get intelligent AI context analysis: {}, falling back to heuristics", e);
                }
            }
        }
        
        // Fallback to original logic if AI helpers aren't available
        if !has_repository {
            tracing::debug!("No repository context available, not using file reading");
            return false;
        }
        
        // If repository is open but no AI analysis available, use basic file reading
        // for obvious code-related questions (fallback)
        if let Some(ctx) = context {
            if ctx.contains("CRITICAL REPOSITORY CONTEXT") || ctx.contains("Repository Path:") {
                tracing::info!("📂 Repository context available, using fallback heuristics");
                
                // Only use basic file reading for explicit code questions when no index exists
                let question_lower = question.to_lowercase();
                let explicitly_about_code = question_lower.contains("repository")
                    || question_lower.contains("codebase")
                    || question_lower.contains("code")
                    || question_lower.contains("file")
                    || question_lower.contains("implement")
                    || question_lower.contains("this")
                    || question_lower.contains("here")
                    || question_lower.contains("@codebase");
                    
                // Removed fallback heuristics - rely on AI semantic analysis only
                tracing::debug!("No AI semantic analysis available, skipping file reading");
            }
        }
        
        tracing::debug!("⏭️ Not using file reading for: '{}'", question);
        false
    }

    /// Build enhanced generator messages with file reading
    async fn build_enhanced_generator_messages(
        &self,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<crate::consensus::types::Message>> {
        use crate::consensus::stages::file_aware_generator::{FileAwareGeneratorStage, build_file_aware_messages};

        let file_aware_generator = FileAwareGeneratorStage::new();
        
        // Get repository context if available
        if let Some(repo_ctx_manager) = &self.repository_context {
            let repo_context = repo_ctx_manager.get_context().await;
            
            // Build messages with actual file contents
            let messages = build_file_aware_messages(
                &file_aware_generator,
                question,
                Some(&repo_context),
                context
            ).await?;
            
            tracing::info!("Built file-aware messages with {} messages", messages.len());
            
            Ok(messages)
        } else {
            // Fallback to regular generator
            tracing::warn!("No repository context available, falling back to regular generator");
            let generator = crate::consensus::stages::GeneratorStage::new();
            generator.build_messages(question, previous_answer, context)
        }
    }

    /// Build enhanced curator messages with file reading and writing capabilities
    async fn build_enhanced_curator_messages(
        &self,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<crate::consensus::types::Message>> {
        use crate::consensus::stages::file_aware_curator::FileAwareCuratorStage;
        use crate::consensus::stages::ConsensusStage;

        let file_aware_curator = FileAwareCuratorStage::new();
        
        // Get repository context if available
        if let Some(repo_ctx_manager) = &self.repository_context {
            // Build messages using the trait method
            let messages = file_aware_curator.build_messages(
                question,
                previous_answer,
                context
            )?;
            
            tracing::info!("Built file-aware curator messages with {} messages", messages.len());
            
            Ok(messages)
        } else {
            // Fallback to regular curator
            tracing::warn!("No repository context available, falling back to regular curator");
            let curator = crate::consensus::stages::CuratorStage::new();
            curator.build_messages(question, previous_answer, context)
        }
    }

    /// Get intelligent context decision for a specific stage
    async fn get_intelligent_context_decision_for_stage(
        &self,
        question: &str,
        context: Option<&str>,
        stage: Stage,
    ) -> crate::ai_helpers::IntelligentContextDecision {
        let has_repository = self.repository_context.is_some();
        
        // First, try to use AI helpers for intelligent semantic analysis
        if let Some(ref ai_helpers) = self.ai_helpers {
            match ai_helpers.intelligent_orchestrator
                .make_intelligent_context_decision(question, has_repository)
                .await {
                Ok(decision) => {
                    tracing::info!("🤖 AI semantic analysis for {} stage: category={:?}, confidence={:.2}, should_use_repo={}", 
                        stage.display_name(), decision.primary_category, decision.confidence, decision.should_use_repo);
                    return decision;
                }
                Err(e) => {
                    tracing::warn!("AI semantic analysis failed for {} stage: {}, using fallback", stage.display_name(), e);
                }
            }
        }
        
        // Check for indexed codebase first (always use if available)
        if let Some(ctx) = context {
            if ctx.contains("SEMANTIC CODEBASE SEARCH RESULTS") {
                tracing::info!("🧠 {} stage: Using indexed codebase knowledge for: '{}'", stage.display_name(), question);
                return crate::ai_helpers::IntelligentContextDecision {
                    should_use_repo: true,
                    confidence: 1.0,
                    primary_category: crate::ai_helpers::QuestionCategory::RepositorySpecific,
                    secondary_categories: vec![],
                    reasoning: "Indexed codebase knowledge available".to_string(),
                    validation_score: 1.0,
                    pattern_analysis: crate::ai_helpers::intelligent_context_orchestrator::PatternAnalysis {
                        detected_patterns: vec!["indexed_codebase".to_string()],
                        code_indicators: 1.0,
                        repo_indicators: 1.0,
                        general_indicators: 0.0,
                        academic_indicators: 0.0,
                    },
                    quality_assessment: crate::ai_helpers::intelligent_context_orchestrator::QualityAssessment {
                        context_appropriateness: 1.0,
                        contamination_risk: 0.0,
                        clarity_score: 1.0,
                        complexity_level: 0.5,
                    },
                };
            }
        }

        // Use intelligent orchestrator for sophisticated context analysis
        if let Some(ai_helpers) = &self.ai_helpers {
            let orchestrator = &ai_helpers.intelligent_orchestrator;
            match orchestrator.make_intelligent_context_decision(question, has_repository).await {
                Ok(decision) => {
                    tracing::debug!("🧠 {} stage: AI context decision - category: {:?}, confidence: {:.2}, use_repo: {}", 
                        stage.display_name(), decision.primary_category, decision.confidence, decision.should_use_repo);
                    return decision;
                }
                Err(e) => {
                    tracing::warn!("Failed to get intelligent AI context analysis for {} stage: {}, falling back to heuristics", stage.display_name(), e);
                }
            }
        }

        // No fallback - require AI helpers for proper context decisions
        tracing::warn!("No AI helpers available for context decision - defaulting to no repository context");
        
        crate::ai_helpers::IntelligentContextDecision {
            should_use_repo: false,  // Default to NO repository context without proper analysis
            confidence: 0.1,  // Very low confidence without AI analysis
            primary_category: crate::ai_helpers::QuestionCategory::GeneralKnowledge,  // Default to general knowledge without AI analysis
            secondary_categories: vec![],
            reasoning: "Fallback heuristic analysis".to_string(),
            validation_score: 0.5,
            pattern_analysis: crate::ai_helpers::intelligent_context_orchestrator::PatternAnalysis {
                detected_patterns: vec!["heuristic_fallback".to_string()],
                code_indicators: 0.2,  // Low indicators without AI analysis
                repo_indicators: 0.1,  // Low indicators without AI analysis
                general_indicators: 0.8,  // High general indicators as default
                academic_indicators: 0.0,
            },
            quality_assessment: crate::ai_helpers::intelligent_context_orchestrator::QualityAssessment {
                context_appropriateness: 0.7,
                contamination_risk: 0.3,
                clarity_score: 0.6,
                complexity_level: 0.5,
            },
        }
    }

    /// Build stage messages with intelligent context guidance
    async fn build_stage_messages_with_intelligent_context(
        &self,
        stage: Stage,
        handler: &dyn ConsensusStage,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
        intelligent_decision: &crate::ai_helpers::IntelligentContextDecision,
    ) -> Result<Vec<crate::consensus::types::Message>> {
        use crate::consensus::types::Message;

        // Get the base messages from the stage handler
        let mut messages = handler.build_messages(question, previous_answer, context)?;

        // Enhance system message with intelligent context guidance
        if let Some(system_message) = messages.iter_mut().find(|m| m.role == "system") {
            let context_guidance = match intelligent_decision.primary_category {
                crate::ai_helpers::QuestionCategory::RepositorySpecific => {
                    if intelligent_decision.should_use_repo {
                        format!("\n\n🧠 INTELLIGENT CONTEXT GUIDANCE:\n\
                            This question is repository-specific (confidence: {:.2}). Focus on the current codebase and use repository context.\n\
                            Reasoning: {}\n\
                            Stay focused on repository analysis and avoid general knowledge contamination.",
                            intelligent_decision.confidence, intelligent_decision.reasoning)
                    } else {
                        format!("\n\n🧠 INTELLIGENT CONTEXT GUIDANCE:\n\
                            This appears repository-specific but no repository context is available (confidence: {:.2}).\n\
                            Reasoning: {}\n\
                            Provide general guidance that could apply to similar codebases.",
                            intelligent_decision.confidence, intelligent_decision.reasoning)
                    }
                }
                crate::ai_helpers::QuestionCategory::GeneralProgramming => {
                    format!("\n\n🧠 INTELLIGENT CONTEXT GUIDANCE:\n\
                        This is a general programming question (confidence: {:.2}). Focus on programming concepts and best practices.\n\
                        Reasoning: {}\n\
                        Avoid repository-specific details and provide broadly applicable guidance.",
                        intelligent_decision.confidence, intelligent_decision.reasoning)
                }
                crate::ai_helpers::QuestionCategory::AcademicKnowledge => {
                    format!("\n\n🧠 INTELLIGENT CONTEXT GUIDANCE:\n\
                        This is an academic/scientific question (confidence: {:.2}). Focus on theoretical knowledge and scientific concepts.\n\
                        Reasoning: {}\n\
                        Avoid programming or repository contamination. Stay in the academic domain.",
                        intelligent_decision.confidence, intelligent_decision.reasoning)
                }
                crate::ai_helpers::QuestionCategory::GeneralKnowledge => {
                    format!("\n\n🧠 INTELLIGENT CONTEXT GUIDANCE:\n\
                        This is a general knowledge question (confidence: {:.2}). Focus on factual information and broad concepts.\n\
                        Reasoning: {}\n\
                        Avoid technical programming details or repository-specific information.",
                        intelligent_decision.confidence, intelligent_decision.reasoning)
                }
                crate::ai_helpers::QuestionCategory::ComputerScience => {
                    format!("\n\n🧠 INTELLIGENT CONTEXT GUIDANCE:\n\
                        This is a computer science theory question (confidence: {:.2}). Focus on algorithms, data structures, and CS concepts.\n\
                        Reasoning: {}\n\
                        Provide theoretical CS knowledge without repository-specific implementation details.",
                        intelligent_decision.confidence, intelligent_decision.reasoning)
                }
                _ => {
                    format!("\n\n🧠 INTELLIGENT CONTEXT GUIDANCE:\n\
                        Question category: {:?} (confidence: {:.2}). Contamination risk: {:.2}\n\
                        Reasoning: {}\n\
                        Focus appropriately on the identified domain to avoid context contamination.",
                        intelligent_decision.primary_category, intelligent_decision.confidence, 
                        intelligent_decision.quality_assessment.contamination_risk, intelligent_decision.reasoning)
                }
            };

            system_message.content.push_str(&context_guidance);
        }

        Ok(messages)
    }
}

/// Response from model API
struct ModelResponse {
    model: String,
    content: String,
    usage: TokenUsage,
    analytics: StageAnalytics,
}

/// Callbacks that forward OpenRouter streaming to our consensus callbacks
struct TrackerForwardingCallbacks {
    tracker_callbacks: Arc<dyn StreamingCallbacks>,
    stage: Stage,
}

impl OpenRouterStreamingCallbacks for TrackerForwardingCallbacks {
    fn on_start(&self) {
        // Already handled by on_stage_start
    }

    fn on_chunk(&self, chunk: String, total_content: String) {
        // Forward to our consensus callbacks
        tracing::debug!(
            "TrackerForwardingCallbacks: Received chunk for stage {:?}: '{}'",
            self.stage,
            chunk
        );
        let result = self
            .tracker_callbacks
            .on_stage_chunk(self.stage, &chunk, &total_content);
        if let Err(e) = result {
            tracing::error!("Failed to forward chunk: {}", e);
        }
    }

    fn on_progress(&self, progress: crate::consensus::openrouter::StreamingProgress) {
        // Convert and forward
        if let Some(tokens) = progress.tokens {
            let percentage = progress.percentage.unwrap_or(0.0);
            let info = ProgressInfo {
                tokens,
                estimated_total: None, // We don't have a good estimate from OpenRouter
                percentage: (percentage / 100.0) as f32, // Convert to 0-1 range
            };
            let _ = self.tracker_callbacks.on_stage_progress(self.stage, info);
        }
    }

    fn on_complete(
        &self,
        _final_content: String,
        _usage: Option<crate::consensus::openrouter::Usage>,
    ) {
        // Will be handled by on_stage_complete
    }
    
    fn on_error(&self, error: &anyhow::Error) {
        // Forward the error, especially important for cancellation
        tracing::warn!("OpenRouter streaming error for stage {:?}: {}", self.stage, error);
        
        // Check if this is a cancellation error
        let error_string = error.to_string();
        if error_string.contains("cancelled") || error_string.contains("Cancelled") {
            // Send a specific cancellation error through the callbacks
            let _ = self.tracker_callbacks.on_error(self.stage, &anyhow::anyhow!("Consensus cancelled by user"));
        } else {
            // Forward other errors
            let _ = self.tracker_callbacks.on_error(self.stage, error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::types::RetryPolicy;

    #[tokio::test]
    async fn test_pipeline_basic() {
        let profile = ConsensusProfile {
            id: "test".to_string(),
            profile_name: "Test Profile".to_string(),
            generator_model: "claude-3-5-sonnet".to_string(),
            refiner_model: "gpt-4-turbo".to_string(),
            validator_model: "claude-3-opus".to_string(),
            curator_model: "gpt-4o".to_string(),
            created_at: Utc::now(),
            is_active: true,
        };

        let config = ConsensusConfig {
            enable_streaming: false,
            show_progress: false,
            timeout_seconds: 60,
            retry_policy: RetryPolicy::default(),
            context_injection: crate::consensus::types::ContextInjectionStrategy::Smart,
        };

        let pipeline = ConsensusPipeline::new(config, profile, None);
        let result = pipeline
            .run("What is Rust?", None)
            .await
            .expect("Pipeline should run successfully");

        assert!(result.success);
        assert!(result.result.is_some());
        assert_eq!(result.stages.len(), 4);
    }
}
