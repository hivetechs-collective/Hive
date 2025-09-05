//! File Operation Planning and Analysis Engine
//!
//! This module provides consensus-level analysis and planning for file operations.
//! IMPORTANT: This module NEVER executes file operations directly - it only creates
//! plans and analyses that are passed to AI Helpers for execution.
//!
//! Architecture Principle:
//! - Consensus (OpenRouter) = THINKING: Analysis, planning, decision making
//! - AI Helpers (Local AI) = DOING: Actual file operations and execution

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::operation_analysis::{
    OperationContext as ConsensusOperationContext, OperationAnalysis as ConsensusOperationAnalysis,
    UnifiedScore, OperationGroups, ComponentScores, ScoringFactors
};
use crate::consensus::smart_decision_engine::{SmartDecisionEngine, ExecutionDecision};
use crate::consensus::operation_intelligence::{
    OperationIntelligenceCoordinator, OperationAnalysis as IntelligenceOperationAnalysis,
    OperationContext as IntelligenceOperationContext
};
use crate::consensus::ai_operation_parser::{AIOperationParser, ParsedOperations};
use crate::consensus::operation_preview_generator::{
    OperationPreviewGenerator, PreviewConfig, OperationPreviewSet
};
use std::collections::HashMap;
use crate::core::error::HiveError;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Execution plan for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPlan {
    pub operation_id: Uuid,
    pub operation: FileOperation,
    pub decision: ExecutionPlanDecision,
    pub safety_checks: Vec<SafetyCheck>,
    pub backup_required: bool,
    pub validation_required: bool,
    pub analysis: ConsensusOperationAnalysis,
    pub created_at: SystemTime,
}

/// Decision for execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionPlanDecision {
    Approved {
        confidence: f32,
        risk_level: f32,
        reason: String,
    },
    RequiresConfirmation {
        warnings: Vec<String>,
        suggestions: Vec<String>,
    },
    Blocked {
        reasons: Vec<String>,
    },
}

/// Safety checks to be performed by AI Helpers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyCheck {
    PathValidation {
        forbidden_paths: Vec<PathBuf>,
        allowed_extensions: Vec<String>,
    },
    SizeLimit {
        max_size: u64,
        content_size: u64,
    },
    BackupRequired,
    SyntaxValidation,
    DependencyCheck {
        required_files: Vec<PathBuf>,
    },
}

/// Batch operation plan with execution order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationPlan {
    pub plans: Vec<OperationPlan>,
    pub execution_order: Vec<usize>,
    pub rollback_strategy: RollbackStrategy,
    pub stop_on_error: bool,
    pub total_operations: usize,
    pub created_at: SystemTime,
}

/// Rollback strategy for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackStrategy {
    ReverseOrder,
    DependencyBased,
    None,
}

/// Configuration for file planning behavior
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    pub create_backups: bool,
    pub validate_syntax: bool,
    pub dry_run_mode: bool,
    pub max_file_size: u64, // in bytes
    pub allowed_extensions: Vec<String>,
    pub forbidden_paths: Vec<PathBuf>,
    pub stop_on_error: bool,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            create_backups: true,
            validate_syntax: true,
            dry_run_mode: false,
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_extensions: vec![
                "rs".to_string(), "js".to_string(), "ts".to_string(),
                "py".to_string(), "java".to_string(), "cpp".to_string(),
                "c".to_string(), "h".to_string(), "hpp".to_string(),
                "go".to_string(), "rb".to_string(), "php".to_string(),
                "md".to_string(), "txt".to_string(), "json".to_string(),
                "yaml".to_string(), "yml".to_string(), "toml".to_string(),
                "html".to_string(), "css".to_string(), "scss".to_string(),
            ],
            forbidden_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/sys"),
                PathBuf::from("/proc"),
                PathBuf::from("/dev"),
                PathBuf::from("/boot"),
            ],
            stop_on_error: true,
        }
    }
}

/// File operation planning and analysis engine
/// IMPORTANT: This does NOT execute operations - it only plans and analyzes them
pub struct FileOperationPlanner {
    config: PlannerConfig,
    decision_engine: SmartDecisionEngine,
    intelligence_coordinator: OperationIntelligenceCoordinator,
    ai_parser: AIOperationParser,
    preview_generator: OperationPreviewGenerator,
}

/// Convert intelligence OperationContext to consensus OperationContext
fn convert_operation_context(
    intelligence_context: &IntelligenceOperationContext,
    user_question: String,
    consensus_response: String,
    session_id: String,
) -> ConsensusOperationContext {
    ConsensusOperationContext {
        repository_path: intelligence_context.repository_path.clone(),
        user_question,
        consensus_response,
        timestamp: SystemTime::now(),
        session_id,
        git_commit: intelligence_context.git_commit.clone(),
    }
}

/// Convert from intelligence analysis to consensus analysis format
fn convert_analysis(
    intelligence_analysis: IntelligenceOperationAnalysis,
    operations: Vec<FileOperation>,
    context: ConsensusOperationContext,
) -> ConsensusOperationAnalysis {
    // Group operations by type
    let mut create_operations = Vec::new();
    let mut update_operations = Vec::new();
    let mut delete_operations = Vec::new();
    let mut move_operations = Vec::new();

    for op in &operations {
        match op {
            FileOperation::Create { .. } => create_operations.push(op.clone()),
            FileOperation::Update { .. } => update_operations.push(op.clone()),
            FileOperation::Delete { .. } => delete_operations.push(op.clone()),
            FileOperation::Rename { .. } => move_operations.push(op.clone()),
            FileOperation::Append { .. } => update_operations.push(op.clone()),
        }
    }

    ConsensusOperationAnalysis {
        operations,
        context,
        unified_score: UnifiedScore {
            confidence: intelligence_analysis.confidence,
            risk: intelligence_analysis.risk,
        },
        recommendations: vec![], // Convert from intelligence_analysis.recommendation if needed
        groups: OperationGroups {
            create_operations,
            update_operations,
            delete_operations,
            move_operations,
        },
        component_scores: ComponentScores {
            knowledge_indexer: None,
            context_retriever: None,
            pattern_recognizer: None,
            quality_analyzer: None,
            knowledge_synthesizer: None,
        },
        scoring_factors: ScoringFactors {
            historical_success: None,
            pattern_safety: None,
            conflict_probability: None,
            rollback_complexity: None,
            user_trust: 0.8,
            similar_operations_count: None,
            dangerous_pattern_count: None,
            anti_pattern_count: None,
            rollback_possible: None,
        },
        statistics: None,
    }
}

impl FileOperationPlanner {
    pub fn new(
        config: PlannerConfig,
        decision_engine: SmartDecisionEngine,
        intelligence_coordinator: OperationIntelligenceCoordinator,
    ) -> Self {
        let preview_config = PreviewConfig {
            max_preview_lines: 100,
            context_lines: 3,
            syntax_highlighting: true,
            highlight_theme: "InspiredGitHub".to_string(),
            show_line_numbers: true,
            collapse_unchanged: true,
            unified_diff: true,
        };
        
        Self {
            config,
            decision_engine,
            intelligence_coordinator,
            ai_parser: AIOperationParser::new(),
            preview_generator: OperationPreviewGenerator::new(preview_config),
        }
    }

    /// Analyze a single file operation and create an execution plan
    /// NOTE: This does NOT execute - it returns a plan for AI Helpers to execute
    pub async fn plan_operation(
        &self,
        operation: FileOperation,
        context: &ConsensusOperationContext,
    ) -> Result<OperationPlan, HiveError> {
        let operation_id = Uuid::new_v4();
        let start_time = SystemTime::now();

        // Step 1: Convert context for intelligence coordinator
        let intelligence_context = IntelligenceOperationContext {
            repository_path: context.repository_path.clone(),
            git_commit: context.git_commit.clone(),
            source_question: context.user_question.clone(),
            related_files: Vec::new(), // Could be populated from repository analysis
            project_metadata: HashMap::new(), // Could be populated from project files
        };

        // Step 2: AI-enhanced analysis  
        let intelligence_analysis = self.intelligence_coordinator
            .analyze_operation(&operation, &intelligence_context)
            .await?;

        // Step 3: Convert to consensus format
        let consensus_analysis = convert_analysis(
            intelligence_analysis,
            vec![operation.clone()],
            context.clone(),
        );

        // Step 4: Smart decision making
        let decision = self.decision_engine
            .make_decision(&consensus_analysis)
            .await?;

        // Step 5: Create execution plan based on decision
        let plan = match decision {
            ExecutionDecision::AutoExecute { reason, confidence, risk_level } => {
                log::info!("Creating execution plan with {}% confidence, {}% risk: {}", 
                          confidence, risk_level, reason);
                OperationPlan {
                    operation_id,
                    operation: operation.clone(),
                    decision: ExecutionPlanDecision::Approved {
                        confidence,
                        risk_level,
                        reason,
                    },
                    safety_checks: self.create_safety_checks(&operation),
                    backup_required: self.config.create_backups,
                    validation_required: self.config.validate_syntax,
                    analysis: consensus_analysis,
                    created_at: start_time,
                }
            }
            ExecutionDecision::RequireConfirmation { warnings, suggestions, .. } => {
                log::warn!("Operation requires user confirmation: {:?}", warnings);
                OperationPlan {
                    operation_id,
                    operation: operation.clone(),
                    decision: ExecutionPlanDecision::RequiresConfirmation {
                        warnings,
                        suggestions,
                    },
                    safety_checks: self.create_safety_checks(&operation),
                    backup_required: self.config.create_backups,
                    validation_required: self.config.validate_syntax,
                    analysis: consensus_analysis,
                    created_at: start_time,
                }
            }
            ExecutionDecision::Block { critical_issues, .. } => {
                log::error!("Operation blocked for safety: {:?}", critical_issues);
                OperationPlan {
                    operation_id,
                    operation: operation.clone(),
                    decision: ExecutionPlanDecision::Blocked {
                        reasons: critical_issues,
                    },
                    safety_checks: self.create_safety_checks(&operation),
                    backup_required: false,
                    validation_required: false,
                    analysis: consensus_analysis,
                    created_at: start_time,
                }
            }
        };
        
        Ok(plan)
    }

    /// Plan multiple operations as a batch with dependency analysis
    /// Returns plans for AI Helpers to execute in the correct order
    pub async fn plan_operations_batch(
        &self,
        operations: Vec<FileOperation>,
        context: &ConsensusOperationContext,
    ) -> Result<BatchOperationPlan, HiveError> {
        let mut operation_plans = Vec::new();

        // Convert context for intelligence coordinator
        let intelligence_context = IntelligenceOperationContext {
            repository_path: context.repository_path.clone(),
            git_commit: context.git_commit.clone(),
            source_question: context.user_question.clone(),
            related_files: Vec::new(),
            project_metadata: HashMap::new(),
        };

        // Analyze all operations first
        let intelligence_analyses = self.intelligence_coordinator
            .analyze_operations_batch(&operations, &intelligence_context)
            .await?;

        // Convert to consensus format
        let consensus_analyses: Vec<_> = operations.iter()
            .zip(intelligence_analyses.into_iter())
            .map(|(op, intel_analysis)| {
                convert_analysis(intel_analysis, vec![op.clone()], context.clone())
            })
            .collect();

        // Create plans for operations in dependency order
        for (idx, (operation, analysis)) in operations.into_iter().zip(consensus_analyses.into_iter()).enumerate() {
            let plan = self.plan_operation_with_analysis(operation, analysis, context).await?;
            operation_plans.push(plan);
        }

        // Determine execution order based on dependencies
        let execution_order = self.determine_execution_order(&operation_plans)?;
        
        let total_operations = operation_plans.len();
        Ok(BatchOperationPlan {
            plans: operation_plans,
            execution_order,
            rollback_strategy: RollbackStrategy::ReverseOrder,
            stop_on_error: self.config.stop_on_error,
            total_operations,
            created_at: SystemTime::now(),
        })
    }

    /// Create operation plan with pre-computed analysis
    async fn plan_operation_with_analysis(
        &self,
        operation: FileOperation,
        analysis: ConsensusOperationAnalysis,
        context: &ConsensusOperationContext,
    ) -> Result<OperationPlan, HiveError> {
        let operation_id = Uuid::new_v4();
        let start_time = SystemTime::now();

        let decision = self.decision_engine.make_decision(&analysis).await?;

        // Create plan based on decision and analysis
        let plan = OperationPlan {
            operation_id,
            operation: operation.clone(),
            decision: match decision {
                ExecutionDecision::AutoExecute { reason, confidence, risk_level } => {
                    ExecutionPlanDecision::Approved { confidence, risk_level, reason }
                }
                ExecutionDecision::RequireConfirmation { warnings, suggestions, .. } => {
                    ExecutionPlanDecision::RequiresConfirmation { warnings, suggestions }
                }
                ExecutionDecision::Block { critical_issues, .. } => {
                    ExecutionPlanDecision::Blocked { reasons: critical_issues }
                }
            },
            safety_checks: self.create_safety_checks(&operation),
            backup_required: self.config.create_backups,
            validation_required: self.config.validate_syntax,
            analysis,
            created_at: start_time,
        };
        
        Ok(plan)
    }

    /// Create safety checks for the operation
    fn create_safety_checks(&self, operation: &FileOperation) -> Vec<SafetyCheck> {
        let mut checks = Vec::new();
        
        // Path validation
        checks.push(SafetyCheck::PathValidation {
            forbidden_paths: self.config.forbidden_paths.clone(),
            allowed_extensions: self.config.allowed_extensions.clone(),
        });
        
        // Size limits
        if let FileOperation::Create { content, .. } | 
           FileOperation::Update { content, .. } | 
           FileOperation::Append { content, .. } = operation {
            checks.push(SafetyCheck::SizeLimit {
                max_size: self.config.max_file_size,
                content_size: content.len() as u64,
            });
        }
        
        // Backup requirement
        if self.config.create_backups {
            checks.push(SafetyCheck::BackupRequired);
        }
        
        // Syntax validation
        if self.config.validate_syntax {
            checks.push(SafetyCheck::SyntaxValidation);
        }
        
        checks
    }

    /// Validate operation safety before execution
    fn validate_operation_safety(&self, operation: &FileOperation) -> Result<(), HiveError> {
        let path = match operation {
            FileOperation::Create { path, .. } |
            FileOperation::Update { path, .. } |
            FileOperation::Delete { path } |
            FileOperation::Append { path, .. } => path,
            FileOperation::Rename { from, .. } => from,
        };

        // Check forbidden paths
        for forbidden in &self.config.forbidden_paths {
            if path.starts_with(forbidden) {
                return Err(HiveError::OperationBlocked {
                    operation_id: Uuid::new_v4(),
                    reasons: vec![format!("Path {} is in forbidden directory {}", 
                                        path.display(), forbidden.display())],
                });
            }
        }

        // Check file extension
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            if !self.config.allowed_extensions.contains(&extension.to_string()) {
                return Err(HiveError::OperationBlocked {
                    operation_id: Uuid::new_v4(),
                    reasons: vec![format!("File extension '{}' is not allowed", extension)],
                });
            }
        }

        Ok(())
    }

    /// Determine execution order for batch operations
    fn determine_execution_order(&self, plans: &[OperationPlan]) -> Result<Vec<usize>, HiveError> {
        // For now, simple sequential order
        // In future, could analyze dependencies and optimize
        Ok((0..plans.len()).collect())
    }
    
    /// Parse operations from curator response without executing them
    pub async fn parse_operations_from_curator_response(
        &self,
        curator_response: &str,
        context: &ConsensusOperationContext,
    ) -> Result<ParsedOperations, HiveError> {
        // Use the existing parse_curator_response method
        self.parse_curator_response(curator_response, context).await
    }
    
    /// Analyze a single operation and get the decision without executing
    pub async fn analyze_operation_decision(
        &self,
        operation: &FileOperation,
        context: &ConsensusOperationContext,
    ) -> Result<ExecutionDecision, HiveError> {
        // Convert context for intelligence coordinator
        let intelligence_context = IntelligenceOperationContext {
            repository_path: context.repository_path.clone(),
            git_commit: context.git_commit.clone(),
            source_question: context.user_question.clone(),
            related_files: Vec::new(),
            project_metadata: HashMap::new(),
        };
        
        // AI-enhanced analysis  
        let intelligence_analysis = self.intelligence_coordinator
            .analyze_operation(operation, &intelligence_context)
            .await?;
        
        // Convert to consensus format
        let consensus_analysis = convert_analysis(
            intelligence_analysis,
            vec![operation.clone()],
            context.clone(),
        );
        
        // Get decision from smart decision engine
        self.decision_engine
            .make_decision(&consensus_analysis)
            .await
            .map_err(|e| HiveError::Internal {
                context: "decision_engine".to_string(),
                message: format!("Failed to make decision: {}", e)
            })
    }

    /// Parse and create plans for operations from curator response
    pub async fn parse_and_plan_curator_response(
        &self,
        curator_response: &str,
        context: &ConsensusOperationContext,
    ) -> Result<ParsedOperationPlans, HiveError> {
        let start_time = SystemTime::now();
        
        // Step 1: Convert context for AI parser
        let intelligence_context = IntelligenceOperationContext {
            repository_path: context.repository_path.clone(),
            git_commit: context.git_commit.clone(),
            source_question: context.user_question.clone(),
            related_files: Vec::new(),
            project_metadata: HashMap::new(),
        };

        // Step 2: Parse curator response using AI-enhanced parser
        let parsed = self.ai_parser
            .parse_response(curator_response, &intelligence_context)
            .await
            .map_err(|e| HiveError::Internal { 
                context: "ai_parser".to_string(),
                message: format!("Failed to parse curator response: {}", e)
            })?;

        log::info!("🤖 Parsed {} operations with {}% confidence", 
                  parsed.operations.len(), parsed.confidence);

        // Log warnings if any
        for warning in &parsed.warnings {
            log::warn!("Parser warning: {}", warning);
        }

        // Check if parsing confidence is too low
        if parsed.confidence < 50.0 {
            return Err(HiveError::Internal {
                context: "ai_parser".to_string(),
                message: format!(
                    "Parsing confidence too low: {:.0}%. Clarifications needed: {:?}",
                    parsed.confidence, parsed.clarifications
                )
            });
        }

        // Step 3: Extract operations in dependency order
        let mut operations_in_order = Vec::new();
        let mut processed = vec![false; parsed.operations.len()];
        
        // First, add operations without dependencies
        for (idx, op_meta) in parsed.operations.iter().enumerate() {
            if op_meta.dependencies.is_empty() {
                operations_in_order.push((idx, &op_meta.operation));
                processed[idx] = true;
            }
        }
        
        // Then, add operations with satisfied dependencies
        while operations_in_order.len() < parsed.operations.len() {
            let mut added_any = false;
            
            for (idx, op_meta) in parsed.operations.iter().enumerate() {
                if !processed[idx] {
                    let deps_satisfied = op_meta.dependencies.iter()
                        .all(|&dep_idx| processed[dep_idx]);
                    
                    if deps_satisfied {
                        operations_in_order.push((idx, &op_meta.operation));
                        processed[idx] = true;
                        added_any = true;
                    }
                }
            }
            
            if !added_any {
                return Err(HiveError::Internal {
                    context: "dependency_resolution".to_string(),
                    message: "Circular dependencies detected in operations".to_string()
                });
            }
        }

        // Step 4: Create plans for operations in order
        let mut plans = Vec::new();

        for (idx, operation) in operations_in_order {
            log::info!("Planning operation {} of {}: {:?}", 
                      idx + 1, parsed.operations.len(), operation);

            match self.plan_operation(operation.clone(), context).await {
                Ok(plan) => {
                    plans.push(plan);
                }
                Err(e) => {
                    log::error!("❌ Operation planning error: {}", e);
                    
                    // Create error plan
                    plans.push(OperationPlan {
                        operation_id: Uuid::new_v4(),
                        operation: operation.clone(),
                        decision: ExecutionPlanDecision::Blocked {
                            reasons: vec![format!("Planning error: {}", e)],
                        },
                        safety_checks: vec![],
                        backup_required: false,
                        validation_required: false,
                        analysis: ConsensusOperationAnalysis {
                            operations: vec![operation.clone()],
                            context: context.clone(),
                            unified_score: UnifiedScore { confidence: 0.0, risk: 1.0 },
                            recommendations: vec![],
                            groups: OperationGroups::default(),
                            component_scores: ComponentScores::default(),
                            scoring_factors: ScoringFactors::default(),
                            statistics: None,
                        },
                        created_at: SystemTime::now(),
                    });

                    // Optionally stop on first error
                    if self.config.stop_on_error {
                        log::error!("Stopping planning due to error (stop_on_error = true)");
                        break;
                    }
                }
            }
        }

        let total_time = start_time.elapsed().unwrap_or(Duration::from_millis(0));

        Ok(ParsedOperationPlans {
            plans,
            parser_confidence: parsed.confidence,
            parser_warnings: parsed.warnings,
            unparsed_blocks: parsed.unparsed_blocks,
            total_planning_time: total_time,
        })
    }

    /// Parse operations from curator response without executing (dry run analysis)
    pub async fn parse_curator_response(
        &self,
        curator_response: &str,
        context: &ConsensusOperationContext,
    ) -> Result<ParsedOperations, HiveError> {
        // Convert context for AI parser
        let intelligence_context = IntelligenceOperationContext {
            repository_path: context.repository_path.clone(),
            git_commit: context.git_commit.clone(),
            source_question: context.user_question.clone(),
            related_files: Vec::new(),
            project_metadata: HashMap::new(),
        };

        self.ai_parser
            .parse_response(curator_response, &intelligence_context)
            .await
            .map_err(|e| HiveError::Internal {
                context: "ai_parser".to_string(),
                message: format!("Failed to parse curator response: {}", e)
            })
    }

    /// Generate previews for parsed operations
    pub async fn generate_operation_previews(
        &mut self,
        parsed_operations: &ParsedOperations,
        context: &ConsensusOperationContext,
    ) -> Result<OperationPreviewSet, HiveError> {
        // Pass the parsed operations with metadata directly to the preview generator
        self.preview_generator
            .generate_previews(&parsed_operations.operations)
            .await
            .map_err(|e| HiveError::Internal {
                context: "preview_generator".to_string(),
                message: format!("Failed to generate operation previews: {}", e)
            })
    }
}

/// Result of parsing and planning operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedOperationPlans {
    pub plans: Vec<OperationPlan>,
    pub parser_confidence: f32,
    pub parser_warnings: Vec<String>,
    pub unparsed_blocks: Vec<String>,
    pub total_planning_time: Duration,
}

// Default implementations are already defined in operation_analysis.rs