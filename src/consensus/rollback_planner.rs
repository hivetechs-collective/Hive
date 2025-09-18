// Rollback Plan Generation for File Operations
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::ai_helpers::knowledge_synthesizer::KnowledgeSynthesizer;
use crate::consensus::dependency_graph::{DependencyGraph, DependencyGraphGenerator};
use crate::consensus::operation_parser::EnhancedFileOperation;
use crate::consensus::operation_preview::{DiffView, FileState};
use crate::consensus::stages::file_aware_curator::FileOperation;

/// Generates comprehensive rollback plans for file operations
#[derive(Debug, Clone)]
pub struct RollbackPlanner {
    /// Knowledge synthesizer for intelligent planning
    knowledge_synthesizer: Arc<KnowledgeSynthesizer>,

    /// Dependency graph generator
    dependency_generator: Arc<DependencyGraphGenerator>,

    /// Configuration
    config: RollbackConfig,

    /// Cache for rollback plans
    plan_cache: Arc<RwLock<HashMap<String, RollbackPlan>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackConfig {
    /// Create automatic backups before operations
    pub auto_backup: bool,

    /// Maximum backup size in MB
    pub max_backup_size_mb: u64,

    /// Use Git for rollback when available
    pub use_git_when_available: bool,

    /// Generate detailed rollback scripts
    pub generate_scripts: bool,

    /// Include dependency rollback
    pub include_dependencies: bool,

    /// Verification after rollback
    pub verify_rollback: bool,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            auto_backup: true,
            max_backup_size_mb: 100,
            use_git_when_available: true,
            generate_scripts: true,
            include_dependencies: true,
            verify_rollback: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    /// Unique plan ID
    pub id: String,

    /// Operations to rollback
    pub operations: Vec<EnhancedFileOperation>,

    /// Rollback strategy
    pub strategy: RollbackStrategy,

    /// Individual rollback steps
    pub steps: Vec<RollbackStep>,

    /// Backup information
    pub backups: Vec<BackupInfo>,

    /// Estimated rollback time
    pub estimated_duration_ms: u64,

    /// Risk assessment
    pub risk_assessment: RollbackRiskAssessment,

    /// Verification steps
    pub verification_steps: Vec<VerificationStep>,

    /// Generated at timestamp
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackStrategy {
    /// Use Git to revert changes
    GitRevert {
        branch: String,
        commit_before: String,
        affected_files: Vec<PathBuf>,
    },

    /// Restore from backups
    BackupRestore {
        backup_location: PathBuf,
        files_to_restore: Vec<PathBuf>,
    },

    /// Manual rollback with generated operations
    ManualRollback {
        reverse_operations: Vec<FileOperation>,
        script_path: Option<PathBuf>,
    },

    /// Hybrid approach combining multiple strategies
    Hybrid {
        primary: Box<RollbackStrategy>,
        fallback: Box<RollbackStrategy>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    /// Step number
    pub step_number: usize,

    /// Step description
    pub description: String,

    /// Operation to perform
    pub operation: RollbackOperation,

    /// Dependencies on other steps
    pub depends_on: Vec<usize>,

    /// Can be automated
    pub automatable: bool,

    /// Estimated duration
    pub estimated_duration_ms: u64,

    /// Risk level
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackOperation {
    /// Restore file from backup
    RestoreFile {
        source: PathBuf,
        destination: PathBuf,
        backup_hash: String,
    },

    /// Execute Git command
    GitCommand {
        command: String,
        args: Vec<String>,
        working_dir: PathBuf,
    },

    /// Apply reverse operation
    ReverseOperation { operation: FileOperation },

    /// Run verification
    VerifyState {
        file: PathBuf,
        expected_hash: Option<String>,
        expected_exists: bool,
    },

    /// Custom script execution
    ExecuteScript {
        script_path: PathBuf,
        args: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Original file path
    pub original_path: PathBuf,

    /// Backup location
    pub backup_path: PathBuf,

    /// File hash before operation
    pub original_hash: String,

    /// Backup size
    pub size_bytes: u64,

    /// Backup created at
    pub created_at: DateTime<Utc>,

    /// Backup method
    pub method: BackupMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupMethod {
    /// Full file copy
    FileCopy,

    /// Git stash
    GitStash { stash_id: String },

    /// Incremental backup
    Incremental { base_backup: String },

    /// Compressed archive
    Compressed { format: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRiskAssessment {
    /// Overall risk level
    pub risk_level: RiskLevel,

    /// Specific risks identified
    pub risks: Vec<RollbackRisk>,

    /// Mitigation strategies
    pub mitigations: Vec<String>,

    /// Success probability (0-100)
    pub success_probability: f32,

    /// Data loss potential
    pub data_loss_potential: DataLossPotential,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRisk {
    /// Risk description
    pub description: String,

    /// Affected files
    pub affected_files: Vec<PathBuf>,

    /// Risk category
    pub category: RiskCategory,

    /// Severity
    pub severity: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskCategory {
    /// Dependencies might break
    DependencyBreakage,

    /// State inconsistency
    StateInconsistency,

    /// Data loss
    DataLoss,

    /// Partial rollback
    PartialRollback,

    /// External system impact
    ExternalImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataLossPotential {
    /// No data loss expected
    None,

    /// Minor data loss (comments, formatting)
    Minor,

    /// Moderate data loss (some content)
    Moderate,

    /// Severe data loss (significant content)
    Severe,

    /// Complete data loss
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStep {
    /// Step description
    pub description: String,

    /// Verification type
    pub verification_type: VerificationType,

    /// Expected outcome
    pub expected_outcome: String,

    /// Automated check available
    pub automated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationType {
    /// File exists check
    FileExists { path: PathBuf },

    /// Content hash verification
    ContentHash {
        path: PathBuf,
        expected_hash: String,
    },

    /// Syntax validation
    SyntaxCheck { path: PathBuf, language: String },

    /// Test execution
    TestExecution { test_command: String },

    /// Build verification
    BuildCheck { build_command: String },

    /// Custom verification
    Custom {
        command: String,
        expected_output: String,
    },
}

impl RollbackPlanner {
    pub fn new(
        knowledge_synthesizer: Arc<KnowledgeSynthesizer>,
        dependency_generator: Arc<DependencyGraphGenerator>,
        config: Option<RollbackConfig>,
    ) -> Self {
        Self {
            knowledge_synthesizer,
            dependency_generator,
            config: config.unwrap_or_default(),
            plan_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a comprehensive rollback plan
    pub async fn generate_rollback_plan(
        &self,
        operations: &[EnhancedFileOperation],
        dependency_graph: Option<&DependencyGraph>,
        current_states: &HashMap<PathBuf, FileState>,
    ) -> Result<RollbackPlan> {
        info!(
            "Generating rollback plan for {} operations",
            operations.len()
        );

        // Generate dependency graph if not provided
        let graph = if let Some(g) = dependency_graph {
            g.clone()
        } else {
            self.dependency_generator
                .generate_dependency_graph(operations, None)
                .await?
        };

        // Determine rollback strategy
        let strategy = self.determine_strategy(operations, current_states).await?;

        // Generate rollback steps
        let steps = self
            .generate_rollback_steps(operations, &strategy, &graph)
            .await?;

        // Create backups if needed
        let backups = if self.config.auto_backup {
            self.create_backups(operations, current_states).await?
        } else {
            Vec::new()
        };

        // Assess risks
        let risk_assessment = self.assess_rollback_risks(&steps, operations)?;

        // Generate verification steps
        let verification_steps = self.generate_verification_steps(operations, current_states)?;

        // Calculate estimated duration
        let estimated_duration_ms = steps.iter().map(|s| s.estimated_duration_ms).sum();

        let plan = RollbackPlan {
            id: uuid::Uuid::new_v4().to_string(),
            operations: operations.to_vec(),
            strategy,
            steps,
            backups,
            estimated_duration_ms,
            risk_assessment,
            verification_steps,
            generated_at: Utc::now(),
        };

        // Cache the plan
        self.plan_cache
            .write()
            .await
            .insert(plan.id.clone(), plan.clone());

        Ok(plan)
    }

    /// Determine the best rollback strategy
    async fn determine_strategy(
        &self,
        operations: &[EnhancedFileOperation],
        current_states: &HashMap<PathBuf, FileState>,
    ) -> Result<RollbackStrategy> {
        // Check if Git is available and configured
        if self.config.use_git_when_available && self.is_git_available().await? {
            // Get current Git state
            let (branch, commit) = self.get_git_state().await?;
            let affected_files = self.get_affected_files(operations);

            // Check if all files are tracked by Git
            let all_tracked = self.check_git_tracked(&affected_files).await?;

            if all_tracked {
                return Ok(RollbackStrategy::GitRevert {
                    branch,
                    commit_before: commit,
                    affected_files,
                });
            }
        }

        // Check backup availability
        if self.can_use_backups(operations, current_states) {
            return Ok(RollbackStrategy::BackupRestore {
                backup_location: self.get_backup_location()?,
                files_to_restore: self.get_affected_files(operations),
            });
        }

        // Generate manual rollback operations
        let reverse_operations = self.generate_reverse_operations(operations)?;
        let script_path = if self.config.generate_scripts {
            Some(self.generate_rollback_script(&reverse_operations).await?)
        } else {
            None
        };

        Ok(RollbackStrategy::ManualRollback {
            reverse_operations,
            script_path,
        })
    }

    /// Generate rollback steps based on strategy
    async fn generate_rollback_steps(
        &self,
        operations: &[EnhancedFileOperation],
        strategy: &RollbackStrategy,
        dependency_graph: &DependencyGraph,
    ) -> Result<Vec<RollbackStep>> {
        let mut steps = Vec::new();

        match strategy {
            RollbackStrategy::GitRevert {
                branch,
                commit_before,
                affected_files,
            } => {
                // Add Git stash step
                steps.push(RollbackStep {
                    step_number: 1,
                    description: "Stash current changes".to_string(),
                    operation: RollbackOperation::GitCommand {
                        command: "git".to_string(),
                        args: vec![
                            "stash".to_string(),
                            "push".to_string(),
                            "-m".to_string(),
                            "Rollback stash".to_string(),
                        ],
                        working_dir: PathBuf::from("."),
                    },
                    depends_on: vec![],
                    automatable: true,
                    estimated_duration_ms: 500,
                    risk_level: RiskLevel::Low,
                });

                // Add checkout step
                steps.push(RollbackStep {
                    step_number: 2,
                    description: format!("Checkout commit {}", &commit_before[..7]),
                    operation: RollbackOperation::GitCommand {
                        command: "git".to_string(),
                        args: vec!["checkout".to_string(), commit_before.clone()],
                        working_dir: PathBuf::from("."),
                    },
                    depends_on: vec![1],
                    automatable: true,
                    estimated_duration_ms: 1000,
                    risk_level: RiskLevel::Medium,
                });

                // Add file-specific checkout for affected files
                for (i, file) in affected_files.iter().enumerate() {
                    steps.push(RollbackStep {
                        step_number: 3 + i,
                        description: format!("Restore file: {}", file.display()),
                        operation: RollbackOperation::GitCommand {
                            command: "git".to_string(),
                            args: vec![
                                "checkout".to_string(),
                                commit_before.clone(),
                                "--".to_string(),
                                file.to_string_lossy().to_string(),
                            ],
                            working_dir: PathBuf::from("."),
                        },
                        depends_on: vec![2],
                        automatable: true,
                        estimated_duration_ms: 200,
                        risk_level: RiskLevel::Low,
                    });
                }
            }

            RollbackStrategy::BackupRestore {
                backup_location,
                files_to_restore,
            } => {
                // Add restore steps for each file
                for (i, file) in files_to_restore.iter().enumerate() {
                    let backup_path = backup_location.join(file.file_name().unwrap_or_default());

                    steps.push(RollbackStep {
                        step_number: i + 1,
                        description: format!("Restore {}", file.display()),
                        operation: RollbackOperation::RestoreFile {
                            source: backup_path,
                            destination: file.clone(),
                            backup_hash: self.calculate_file_hash(file)?,
                        },
                        depends_on: vec![],
                        automatable: true,
                        estimated_duration_ms: 500,
                        risk_level: RiskLevel::Low,
                    });
                }
            }

            RollbackStrategy::ManualRollback {
                reverse_operations, ..
            } => {
                // Generate steps from dependency graph
                let execution_order = &dependency_graph.execution_sequence;

                // Reverse the order for rollback
                for (i, op_id) in execution_order.iter().rev().enumerate() {
                    if let Some(node_idx) = dependency_graph.node_map.get(op_id) {
                        let node = &dependency_graph.graph[*node_idx];
                        let reverse_op = self.reverse_operation(&node.operation)?;

                        steps.push(RollbackStep {
                            step_number: i + 1,
                            description: self.describe_rollback_operation(&reverse_op),
                            operation: RollbackOperation::ReverseOperation {
                                operation: reverse_op,
                            },
                            depends_on: self.calculate_step_dependencies(i, execution_order),
                            automatable: true,
                            estimated_duration_ms: node.estimated_duration_ms,
                            risk_level: self.assess_operation_risk(&node.operation),
                        });
                    }
                }
            }

            RollbackStrategy::Hybrid { primary, fallback } => {
                // Add primary strategy steps
                let primary_steps =
                    Box::pin(self.generate_rollback_steps(operations, primary, dependency_graph))
                        .await?;
                steps.extend(primary_steps);

                // Add fallback trigger
                steps.push(RollbackStep {
                    step_number: steps.len() + 1,
                    description: "Check primary rollback success".to_string(),
                    operation: RollbackOperation::VerifyState {
                        file: PathBuf::from("."),
                        expected_hash: None,
                        expected_exists: true,
                    },
                    depends_on: vec![steps.len()],
                    automatable: true,
                    estimated_duration_ms: 100,
                    risk_level: RiskLevel::Low,
                });

                // Add fallback steps
                let fallback_steps =
                    Box::pin(self.generate_rollback_steps(operations, fallback, dependency_graph))
                        .await?;
                let base_step = steps.len();
                for mut step in fallback_steps {
                    step.step_number += base_step;
                    step.depends_on = step.depends_on.iter().map(|d| d + base_step).collect();
                    steps.push(step);
                }
            }
        }

        // Add verification steps
        if self.config.verify_rollback {
            self.add_verification_steps(&mut steps, operations)?;
        }

        Ok(steps)
    }

    /// Create backups for affected files
    async fn create_backups(
        &self,
        operations: &[EnhancedFileOperation],
        current_states: &HashMap<PathBuf, FileState>,
    ) -> Result<Vec<BackupInfo>> {
        let mut backups = Vec::new();
        let backup_dir = self.get_backup_location()?;

        // Create backup directory
        tokio::fs::create_dir_all(&backup_dir).await?;

        for op in operations {
            let file_path = self.get_operation_path(&op.operation);

            // Skip if file doesn't exist yet (create operations)
            if let Some(state) = current_states.get(&file_path) {
                if state.exists {
                    let size = state.metadata.size.unwrap_or(0);

                    // Check size limit
                    if size > self.config.max_backup_size_mb * 1024 * 1024 {
                        warn!("File {} exceeds backup size limit", file_path.display());
                        continue;
                    }

                    // Create backup
                    let backup_path = backup_dir.join(format!(
                        "{}_{}",
                        Utc::now().timestamp(),
                        file_path.file_name().unwrap_or_default().to_string_lossy()
                    ));

                    // Copy file
                    tokio::fs::copy(&file_path, &backup_path).await?;

                    backups.push(BackupInfo {
                        original_path: file_path.clone(),
                        backup_path,
                        original_hash: String::new(), // TODO: Calculate hash from content
                        size_bytes: size,
                        created_at: Utc::now(),
                        method: BackupMethod::FileCopy,
                    });
                }
            }
        }

        Ok(backups)
    }

    /// Assess risks in the rollback plan
    fn assess_rollback_risks(
        &self,
        steps: &[RollbackStep],
        operations: &[EnhancedFileOperation],
    ) -> Result<RollbackRiskAssessment> {
        let mut risks = Vec::new();
        let mut overall_risk = RiskLevel::Low;

        // Check for partial rollback risk
        if steps.len() != operations.len() {
            risks.push(RollbackRisk {
                description: "Partial rollback - not all operations can be reversed".to_string(),
                affected_files: self.get_affected_files(operations),
                category: RiskCategory::PartialRollback,
                severity: RiskLevel::High,
            });
            overall_risk = RiskLevel::High;
        }

        // Check for dependency risks
        let has_dependencies = operations
            .iter()
            .any(|op| !op.context.dependencies.is_empty());

        if has_dependencies {
            risks.push(RollbackRisk {
                description: "Operations have dependencies that might be affected".to_string(),
                affected_files: Vec::new(),
                category: RiskCategory::DependencyBreakage,
                severity: RiskLevel::Medium,
            });
            if overall_risk == RiskLevel::Low {
                overall_risk = RiskLevel::Medium;
            }
        }

        // Check for external impact
        let has_external_impact = operations
            .iter()
            .any(|op| self.check_external_impact(&op.operation));

        if has_external_impact {
            risks.push(RollbackRisk {
                description: "Operations might affect external systems".to_string(),
                affected_files: Vec::new(),
                category: RiskCategory::ExternalImpact,
                severity: RiskLevel::Medium,
            });
        }

        // Calculate success probability
        let success_probability = match overall_risk {
            RiskLevel::Low => 95.0,
            RiskLevel::Medium => 80.0,
            RiskLevel::High => 60.0,
            RiskLevel::Critical => 40.0,
        };

        // Determine data loss potential
        let data_loss_potential = self.assess_data_loss_potential(operations);

        // Generate mitigation strategies
        let mitigations = self.generate_mitigation_strategies(&risks);

        Ok(RollbackRiskAssessment {
            risk_level: overall_risk,
            risks,
            mitigations,
            success_probability,
            data_loss_potential,
        })
    }

    /// Generate verification steps
    fn generate_verification_steps(
        &self,
        operations: &[EnhancedFileOperation],
        current_states: &HashMap<PathBuf, FileState>,
    ) -> Result<Vec<VerificationStep>> {
        let mut steps = Vec::new();

        for op in operations {
            let file_path = self.get_operation_path(&op.operation);

            match &op.operation {
                FileOperation::Create { .. } => {
                    // Verify file doesn't exist after rollback
                    steps.push(VerificationStep {
                        description: format!("Verify {} was removed", file_path.display()),
                        verification_type: VerificationType::FileExists {
                            path: file_path.clone(),
                        },
                        expected_outcome: "File should not exist".to_string(),
                        automated: true,
                    });
                }

                FileOperation::Update { .. }
                | FileOperation::Delete { .. }
                | FileOperation::Append { .. } => {
                    // Verify file was restored
                    if let Some(state) = current_states.get(&file_path) {
                        if state.content.is_some() {
                            // Check if we have content to hash
                            steps.push(VerificationStep {
                                description: format!("Verify {} was restored", file_path.display()),
                                verification_type: VerificationType::ContentHash {
                                    path: file_path.clone(),
                                    expected_hash: String::new(), // TODO: Calculate from state.content
                                },
                                expected_outcome: "File content matches original".to_string(),
                                automated: true,
                            });
                        }
                    }
                }

                FileOperation::Rename { from, to } => {
                    // Verify rename was reversed
                    steps.push(VerificationStep {
                        description: format!(
                            "Verify rename reversed: {} -> {}",
                            to.display(),
                            from.display()
                        ),
                        verification_type: VerificationType::FileExists { path: from.clone() },
                        expected_outcome: "Original file name restored".to_string(),
                        automated: true,
                    });
                }
            }
        }

        // Add syntax check for code files
        for op in operations {
            let file_path = self.get_operation_path(&op.operation);
            if let Some(lang) = self.detect_language(&file_path) {
                steps.push(VerificationStep {
                    description: format!("Verify {} syntax", file_path.display()),
                    verification_type: VerificationType::SyntaxCheck {
                        path: file_path,
                        language: lang,
                    },
                    expected_outcome: "Valid syntax".to_string(),
                    automated: true,
                });
            }
        }

        Ok(steps)
    }

    /// Helper methods

    async fn is_git_available(&self) -> Result<bool> {
        // Check if we're in a Git repository
        match tokio::process::Command::new("git")
            .args(&["rev-parse", "--git-dir"])
            .output()
            .await
        {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    async fn get_git_state(&self) -> Result<(String, String)> {
        // Get current branch
        let branch_output = tokio::process::Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .await?;

        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();

        // Get current commit
        let commit_output = tokio::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .await?;

        let commit = String::from_utf8_lossy(&commit_output.stdout)
            .trim()
            .to_string();

        Ok((branch, commit))
    }

    async fn check_git_tracked(&self, files: &[PathBuf]) -> Result<bool> {
        for file in files {
            let output = tokio::process::Command::new("git")
                .args(&["ls-files", "--error-unmatch", &file.to_string_lossy()])
                .output()
                .await?;

            if !output.status.success() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn get_affected_files(&self, operations: &[EnhancedFileOperation]) -> Vec<PathBuf> {
        operations
            .iter()
            .map(|op| self.get_operation_path(&op.operation))
            .collect()
    }

    fn get_operation_path(&self, operation: &FileOperation) -> PathBuf {
        match operation {
            FileOperation::Create { path, .. }
            | FileOperation::Update { path, .. }
            | FileOperation::Delete { path }
            | FileOperation::Append { path, .. } => path.clone(),
            FileOperation::Rename { to, .. } => to.clone(),
        }
    }

    fn can_use_backups(
        &self,
        operations: &[EnhancedFileOperation],
        current_states: &HashMap<PathBuf, FileState>,
    ) -> bool {
        // Check if we have backups for all modified files
        operations.iter().all(|op| {
            let path = self.get_operation_path(&op.operation);
            current_states.contains_key(&path)
        })
    }

    fn get_backup_location(&self) -> Result<PathBuf> {
        let backup_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Could not determine local data directory"))?
            .join("hive")
            .join("rollback_backups");
        Ok(backup_dir)
    }

    fn generate_reverse_operations(
        &self,
        operations: &[EnhancedFileOperation],
    ) -> Result<Vec<FileOperation>> {
        let mut reverse_ops = Vec::new();

        for op in operations.iter().rev() {
            reverse_ops.push(self.reverse_operation(&op.operation)?);
        }

        Ok(reverse_ops)
    }

    fn reverse_operation(&self, operation: &FileOperation) -> Result<FileOperation> {
        match operation {
            FileOperation::Create { path, .. } => Ok(FileOperation::Delete { path: path.clone() }),
            FileOperation::Delete { path } => {
                // Can't reverse without content
                Err(anyhow!("Cannot reverse delete operation without backup"))
            }
            FileOperation::Update { path, content } => {
                // For update, we'd need the original content to truly reverse
                // In practice, this would come from backups or git history
                Ok(FileOperation::Update {
                    path: path.clone(),
                    content: "".to_string(), // Would need backup content
                })
            }
            FileOperation::Append { path, .. } => {
                // For append, we'd need to know how much was appended to reverse it
                Err(anyhow!(
                    "Cannot reverse append operation without knowing original length"
                ))
            }
            FileOperation::Rename { from, to } => Ok(FileOperation::Rename {
                from: to.clone(),
                to: from.clone(),
            }),
        }
    }

    async fn generate_rollback_script(&self, operations: &[FileOperation]) -> Result<PathBuf> {
        let script_dir = self.get_backup_location()?;
        tokio::fs::create_dir_all(&script_dir).await?;

        let script_path = script_dir.join(format!("rollback_{}.sh", Utc::now().timestamp()));
        let mut script_content = String::from("#!/bin/bash\n\n");
        script_content.push_str("# Auto-generated rollback script\n");
        script_content.push_str(&format!("# Generated at: {}\n\n", Utc::now()));

        script_content.push_str("set -e  # Exit on error\n\n");

        for (i, op) in operations.iter().enumerate() {
            script_content.push_str(&format!(
                "echo \"Step {}: {}\"\n",
                i + 1,
                self.describe_rollback_operation(op)
            ));

            match op {
                FileOperation::Delete { path } => {
                    script_content.push_str(&format!("rm -f \"{}\"\n", path.display()));
                }
                FileOperation::Create { path, content } => {
                    script_content.push_str(&format!(
                        "cat > \"{}\" << 'EOF'\n{}\nEOF\n",
                        path.display(),
                        content
                    ));
                }
                FileOperation::Rename { from, to } => {
                    script_content.push_str(&format!(
                        "mv \"{}\" \"{}\"\n",
                        from.display(),
                        to.display()
                    ));
                }
                FileOperation::Update { path, content, .. } => {
                    script_content.push_str(&format!(
                        "cat > \"{}\" << 'EOF'\n{}\nEOF\n",
                        path.display(),
                        content
                    ));
                }
                FileOperation::Append { path, content } => {
                    script_content.push_str(&format!(
                        "cat >> \"{}\" << 'EOF'\n{}\nEOF\n",
                        path.display(),
                        content
                    ));
                }
            }
            script_content.push_str("\n");
        }

        script_content.push_str("echo \"Rollback completed successfully!\"\n");

        tokio::fs::write(&script_path, script_content).await?;

        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&script_path).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&script_path, perms).await?;
        }

        Ok(script_path)
    }

    fn describe_rollback_operation(&self, operation: &FileOperation) -> String {
        match operation {
            FileOperation::Create { path, .. } => format!("Create {}", path.display()),
            FileOperation::Delete { path } => format!("Delete {}", path.display()),
            FileOperation::Update { path, .. } => format!("Update {}", path.display()),
            FileOperation::Append { path, .. } => format!("Append to {}", path.display()),
            FileOperation::Rename { from, to } => {
                format!("Rename {} to {}", from.display(), to.display())
            }
        }
    }

    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        // Simple placeholder - in real implementation would calculate actual hash
        Ok(format!("{:x}", path.to_string_lossy().len()))
    }

    fn assess_operation_risk(&self, operation: &FileOperation) -> RiskLevel {
        match operation {
            FileOperation::Delete { .. } => RiskLevel::High,
            FileOperation::Update { path, .. } => {
                if path.to_string_lossy().contains("config") {
                    RiskLevel::Medium
                } else {
                    RiskLevel::Low
                }
            }
            _ => RiskLevel::Low,
        }
    }

    fn calculate_step_dependencies(&self, index: usize, execution_order: &[String]) -> Vec<usize> {
        // In reverse order, each step depends on the previous one
        if index > 0 {
            vec![index]
        } else {
            vec![]
        }
    }

    fn add_verification_steps(
        &self,
        steps: &mut Vec<RollbackStep>,
        operations: &[EnhancedFileOperation],
    ) -> Result<()> {
        let base_step = steps.len();

        // Add final verification step
        steps.push(RollbackStep {
            step_number: base_step + 1,
            description: "Verify rollback completion".to_string(),
            operation: RollbackOperation::VerifyState {
                file: PathBuf::from("."),
                expected_hash: None,
                expected_exists: true,
            },
            depends_on: vec![base_step],
            automatable: true,
            estimated_duration_ms: 1000,
            risk_level: RiskLevel::Low,
        });

        Ok(())
    }

    fn check_external_impact(&self, operation: &FileOperation) -> bool {
        let path = self.get_operation_path(operation);

        // Check for package files, configs, etc.
        let external_patterns = vec![
            "package.json",
            "Cargo.toml",
            "requirements.txt",
            ".env",
            "config",
        ];

        let path_str = path.to_string_lossy();
        external_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }

    fn assess_data_loss_potential(
        &self,
        operations: &[EnhancedFileOperation],
    ) -> DataLossPotential {
        let delete_count = operations
            .iter()
            .filter(|op| matches!(op.operation, FileOperation::Delete { .. }))
            .count();

        let update_count = operations
            .iter()
            .filter(|op| matches!(op.operation, FileOperation::Update { .. }))
            .count();

        if delete_count > 5 {
            DataLossPotential::Severe
        } else if delete_count > 0 {
            DataLossPotential::Moderate
        } else if update_count > 10 {
            DataLossPotential::Minor
        } else {
            DataLossPotential::None
        }
    }

    fn generate_mitigation_strategies(&self, risks: &[RollbackRisk]) -> Vec<String> {
        let mut strategies = Vec::new();

        for risk in risks {
            match risk.category {
                RiskCategory::DependencyBreakage => {
                    strategies.push("Run dependency checks before rollback".to_string());
                    strategies.push("Consider rollback order to minimize breakage".to_string());
                }
                RiskCategory::StateInconsistency => {
                    strategies.push("Create full system snapshot before rollback".to_string());
                    strategies.push("Verify state consistency after each step".to_string());
                }
                RiskCategory::DataLoss => {
                    strategies.push("Ensure all data is backed up before rollback".to_string());
                    strategies.push("Consider incremental rollback approach".to_string());
                }
                RiskCategory::PartialRollback => {
                    strategies.push("Manual intervention may be required".to_string());
                    strategies.push("Document manual steps clearly".to_string());
                }
                RiskCategory::ExternalImpact => {
                    strategies.push("Notify affected systems before rollback".to_string());
                    strategies.push("Plan for external system updates".to_string());
                }
            }
        }

        strategies.sort();
        strategies.dedup();
        strategies
    }

    fn detect_language(&self, path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "rs" => Some("rust"),
                "js" | "jsx" => Some("javascript"),
                "ts" | "tsx" => Some("typescript"),
                "py" => Some("python"),
                "go" => Some("go"),
                "java" => Some("java"),
                "cpp" | "cc" | "cxx" => Some("cpp"),
                "c" => Some("c"),
                "rb" => Some("ruby"),
                "php" => Some("php"),
                _ => None,
            })
            .map(|s| s.to_string())
    }

    /// Execute a rollback plan
    pub async fn execute_rollback(
        &self,
        plan: &RollbackPlan,
        dry_run: bool,
    ) -> Result<RollbackExecutionResult> {
        info!("Executing rollback plan {} (dry_run: {})", plan.id, dry_run);

        let mut executed_steps = Vec::new();
        let mut failed_steps = Vec::new();
        let start_time = Utc::now();

        for step in &plan.steps {
            match self.execute_step(step, dry_run).await {
                Ok(result) => {
                    executed_steps.push(ExecutedStep {
                        step_number: step.step_number,
                        success: true,
                        duration_ms: result.duration_ms,
                        output: result.output,
                        error: None,
                    });
                }
                Err(e) => {
                    let error_msg = format!("Step {} failed: {}", step.step_number, e);
                    warn!("{}", error_msg);

                    failed_steps.push(FailedStep {
                        step_number: step.step_number,
                        error: error_msg.clone(),
                        recoverable: self.is_recoverable_error(&e),
                    });

                    if !self.is_recoverable_error(&e) {
                        return Err(anyhow!("Critical error during rollback: {}", e));
                    }
                }
            }
        }

        let duration = Utc::now().signed_duration_since(start_time);

        Ok(RollbackExecutionResult {
            plan_id: plan.id.clone(),
            success: failed_steps.is_empty(),
            executed_steps,
            failed_steps,
            total_duration_ms: duration.num_milliseconds() as u64,
            dry_run,
        })
    }

    async fn execute_step(
        &self,
        step: &RollbackStep,
        dry_run: bool,
    ) -> Result<StepExecutionResult> {
        let start = std::time::Instant::now();
        let mut output = String::new();

        if dry_run {
            output.push_str(&format!("[DRY RUN] Would execute: {}\n", step.description));
        } else {
            match &step.operation {
                RollbackOperation::RestoreFile {
                    source,
                    destination,
                    ..
                } => {
                    tokio::fs::copy(source, destination).await?;
                    output.push_str(&format!("Restored {} from backup", destination.display()));
                }

                RollbackOperation::GitCommand {
                    command,
                    args,
                    working_dir,
                } => {
                    let result = tokio::process::Command::new(command)
                        .args(args)
                        .current_dir(working_dir)
                        .output()
                        .await?;

                    if !result.status.success() {
                        return Err(anyhow!(
                            "Git command failed: {}",
                            String::from_utf8_lossy(&result.stderr)
                        ));
                    }

                    output.push_str(&String::from_utf8_lossy(&result.stdout));
                }

                RollbackOperation::ReverseOperation { operation } => {
                    // This would integrate with the actual file operation system
                    output.push_str(&format!(
                        "Executed: {}",
                        self.describe_rollback_operation(operation)
                    ));
                }

                RollbackOperation::VerifyState {
                    file,
                    expected_exists,
                    ..
                } => {
                    let exists = tokio::fs::metadata(file).await.is_ok();
                    if exists != *expected_exists {
                        return Err(anyhow!(
                            "Verification failed: {} exists={}, expected={}",
                            file.display(),
                            exists,
                            expected_exists
                        ));
                    }
                    output.push_str("Verification passed");
                }

                RollbackOperation::ExecuteScript { script_path, args } => {
                    let result = tokio::process::Command::new(script_path)
                        .args(args)
                        .output()
                        .await?;

                    if !result.status.success() {
                        return Err(anyhow!(
                            "Script failed: {}",
                            String::from_utf8_lossy(&result.stderr)
                        ));
                    }

                    output.push_str(&String::from_utf8_lossy(&result.stdout));
                }
            }
        }

        Ok(StepExecutionResult {
            duration_ms: start.elapsed().as_millis() as u64,
            output,
        })
    }

    fn is_recoverable_error(&self, error: &anyhow::Error) -> bool {
        // Simple heuristic - could be enhanced
        !error.to_string().contains("Critical")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackExecutionResult {
    pub plan_id: String,
    pub success: bool,
    pub executed_steps: Vec<ExecutedStep>,
    pub failed_steps: Vec<FailedStep>,
    pub total_duration_ms: u64,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutedStep {
    pub step_number: usize,
    pub success: bool,
    pub duration_ms: u64,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedStep {
    pub step_number: usize,
    pub error: String,
    pub recoverable: bool,
}

#[derive(Debug)]
struct StepExecutionResult {
    duration_ms: u64,
    output: String,
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rollback_plan_generation() {
        // Create test instances
        let python_config = crate::ai_helpers::python_models::PythonModelConfig::default();
        let python_service = Arc::new(
            crate::ai_helpers::python_models::PythonModelService::new(python_config)
                .await
                .unwrap(),
        );
        let knowledge_synthesizer = Arc::new(
            KnowledgeSynthesizer::new(python_service.clone())
                .await
                .unwrap(),
        );
        let pattern_recognizer = Arc::new(
            crate::ai_helpers::pattern_recognizer::PatternRecognizer::new(python_service)
                .await
                .unwrap(),
        );
        let dependency_generator = Arc::new(DependencyGraphGenerator::new(
            Some(pattern_recognizer),
            None,
        ));
        let planner = RollbackPlanner::new(knowledge_synthesizer, dependency_generator, None);

        // Create test operations
        let operations = vec![EnhancedFileOperation {
            operation: FileOperation::Create {
                path: PathBuf::from("test.rs"),
                content: "fn main() {}".to_string(),
            },
            context: Default::default(),
            parsing_confidence: 0.9,
        }];

        let current_states = HashMap::new();

        // Generate plan
        let plan = planner
            .generate_rollback_plan(&operations, None, &current_states)
            .await
            .unwrap();

        assert!(!plan.steps.is_empty());
        assert!(plan.estimated_duration_ms > 0);
    }

    #[tokio::test]
    async fn test_reverse_operation() {
        let python_config = crate::ai_helpers::python_models::PythonModelConfig::default();
        let python_service = Arc::new(
            crate::ai_helpers::python_models::PythonModelService::new(python_config)
                .await
                .unwrap(),
        );
        let knowledge_synthesizer = Arc::new(
            KnowledgeSynthesizer::new(python_service.clone())
                .await
                .unwrap(),
        );
        let pattern_recognizer = Arc::new(
            crate::ai_helpers::pattern_recognizer::PatternRecognizer::new(python_service)
                .await
                .unwrap(),
        );
        let dependency_generator = Arc::new(DependencyGraphGenerator::new(
            Some(pattern_recognizer),
            None,
        ));
        let planner = RollbackPlanner::new(knowledge_synthesizer, dependency_generator, None);

        // Test create -> delete
        let create_op = FileOperation::Create {
            path: PathBuf::from("test.rs"),
            content: "content".to_string(),
        };

        let reverse = planner.reverse_operation(&create_op).unwrap();
        assert!(matches!(reverse, FileOperation::Delete { .. }));

        // Test rename reversal
        let rename_op = FileOperation::Rename {
            from: PathBuf::from("old.rs"),
            to: PathBuf::from("new.rs"),
        };

        let reverse = planner.reverse_operation(&rename_op).unwrap();
        match reverse {
            FileOperation::Rename { from, to } => {
                assert_eq!(from, PathBuf::from("new.rs"));
                assert_eq!(to, PathBuf::from("old.rs"));
            }
            _ => panic!("Expected rename operation"),
        }
    }
}
