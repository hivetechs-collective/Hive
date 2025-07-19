// Comprehensive Safety Guardrails and Validation System
use crate::consensus::operation_intelligence::{OperationAnalysis, ComponentScores};
use crate::consensus::operation_parser::EnhancedFileOperation;
use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::smart_decision_engine::{ExecutionDecision, UserDecision};
use crate::consensus::outcome_tracker::OperationOutcomeTracker;
use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, HashSet};
use tracing::{info, warn, error, debug};
use regex::Regex;

/// Multi-layered safety guardrail system for operation validation
pub struct SafetyGuardrailSystem {
    /// Core safety validators
    core_validators: Vec<Box<dyn SafetyValidator + Send + Sync>>,
    
    /// Pattern-based safety rules
    pattern_rules: Arc<RwLock<Vec<SafetyRule>>>,
    
    /// Dynamic risk assessment engine
    risk_assessor: Arc<DynamicRiskAssessor>,
    
    /// Safety policy enforcement
    policy_enforcer: Arc<SafetyPolicyEnforcer>,
    
    /// Emergency brake system
    emergency_brake: Arc<EmergencyBrakeSystem>,
    
    /// Safety metrics tracking
    safety_metrics: Arc<RwLock<SafetyMetrics>>,
    
    /// Configuration
    config: SafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Enable strict mode (stricter validation)
    pub strict_mode: bool,
    /// Maximum risk score allowed for auto-execution
    pub max_auto_risk_score: f32,
    /// Enable emergency brake system
    pub emergency_brake_enabled: bool,
    /// Minimum time between high-risk operations (seconds)
    pub high_risk_cooldown_seconds: u64,
    /// Maximum number of concurrent risky operations
    pub max_concurrent_risky_ops: u32,
    /// Enable machine learning-based risk assessment
    pub ml_risk_assessment: bool,
    /// Safety rule enforcement level
    pub enforcement_level: EnforcementLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementLevel {
    Advisory,    // Only warn, don't block
    Enforcing,   // Block dangerous operations
    Paranoid,    // Block anything potentially dangerous
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            max_auto_risk_score: 40.0,
            emergency_brake_enabled: true,
            high_risk_cooldown_seconds: 300, // 5 minutes
            max_concurrent_risky_ops: 2,
            ml_risk_assessment: true,
            enforcement_level: EnforcementLevel::Enforcing,
        }
    }
}

/// Core safety validator trait
pub trait SafetyValidator {
    fn name(&self) -> &'static str;
    fn validate(&self, operation: &EnhancedFileOperation, context: &SafetyContext) -> Result<SafetyValidationResult>;
    fn priority(&self) -> u8; // 0-255, higher = more important
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyValidationResult {
    pub validator_name: String,
    pub is_safe: bool,
    pub risk_score: f32, // 0-100
    pub violations: Vec<SafetyViolation>,
    pub warnings: Vec<String>,
    pub required_confirmations: Vec<RequiredConfirmation>,
    pub suggested_mitigations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyViolation {
    pub rule_id: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub affected_files: Vec<PathBuf>,
    pub mitigation_required: bool,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical,  // Operation must be blocked
    High,      // Requires explicit user confirmation
    Medium,    // Warning with option to proceed
    Low,       // Advisory only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredConfirmation {
    pub confirmation_type: ConfirmationType,
    pub message: String,
    pub default_response: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfirmationType {
    ExplicitUserApproval,
    SecondaryConfirmation,
    BackupVerification,
    RollbackPlanApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyContext {
    pub operation_id: String,
    pub user_id: Option<String>,
    pub project_path: PathBuf,
    pub git_status: Option<GitStatus>,
    pub recent_operations: Vec<RecentOperation>,
    pub system_state: SystemState,
    pub user_preferences: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub has_uncommitted_changes: bool,
    pub current_branch: String,
    pub is_clean_working_tree: bool,
    pub has_untracked_files: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentOperation {
    pub operation_type: String,
    pub files_affected: Vec<PathBuf>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub available_disk_space: u64,
    pub system_load: f32,
    pub backup_system_status: BackupSystemStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupSystemStatus {
    Available,
    Degraded,
    Unavailable,
}

/// Safety rule for pattern-based validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub pattern: RulePattern,
    pub severity: ViolationSeverity,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub trigger_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RulePattern {
    FilePathRegex(String),
    FileContentRegex(String),
    OperationType(String),
    FileExtension(String),
    FileSize { min: Option<u64>, max: Option<u64> },
    Complex(ComplexPattern),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexPattern {
    pub conditions: Vec<PatternCondition>,
    pub logic: PatternLogic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternCondition {
    And(Vec<PatternCondition>),
    Or(Vec<PatternCondition>),
    Not(Box<PatternCondition>),
    FileExists(PathBuf),
    FileMatches(String, String), // path pattern, content pattern
    SystemStateCheck(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternLogic {
    All, // All conditions must be true
    Any, // Any condition must be true
    None, // No conditions must be true
}

impl SafetyGuardrailSystem {
    pub fn new(config: Option<SafetyConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        let mut core_validators: Vec<Box<dyn SafetyValidator + Send + Sync>> = Vec::new();
        
        // Initialize core validators
        core_validators.push(Box::new(CriticalFileValidator::new()));
        core_validators.push(Box::new(SystemFileValidator::new()));
        core_validators.push(Box::new(BackupRequiredValidator::new()));
        core_validators.push(Box::new(GitIntegrityValidator::new()));
        core_validators.push(Box::new(PermissionValidator::new()));
        core_validators.push(Box::new(DiskSpaceValidator::new()));
        core_validators.push(Box::new(ConcurrencyValidator::new()));
        core_validators.push(Box::new(PatternMatchValidator::new()));
        
        // Sort validators by priority
        core_validators.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        Self {
            core_validators,
            pattern_rules: Arc::new(RwLock::new(Self::create_default_safety_rules())),
            risk_assessor: Arc::new(DynamicRiskAssessor::new()),
            policy_enforcer: Arc::new(SafetyPolicyEnforcer::new()),
            emergency_brake: Arc::new(EmergencyBrakeSystem::new()),
            safety_metrics: Arc::new(RwLock::new(SafetyMetrics::default())),
            config,
        }
    }

    /// Comprehensive safety validation of an operation
    pub async fn validate_operation(
        &self,
        operation: &EnhancedFileOperation,
        analysis: &OperationAnalysis,
        context: &SafetyContext,
    ) -> Result<ComprehensiveSafetyResult> {
        info!("Running comprehensive safety validation for operation {}", context.operation_id);

        // Emergency brake check first
        if self.config.emergency_brake_enabled {
            if let Some(brake_reason) = self.emergency_brake.should_brake(operation, context).await? {
                return Ok(ComprehensiveSafetyResult {
                    overall_safe: false,
                    risk_score: 100.0,
                    enforcement_action: EnforcementAction::Block,
                    validation_results: Vec::new(),
                    safety_violations: vec![SafetyViolation {
                        rule_id: "EMERGENCY_BRAKE".to_string(),
                        severity: ViolationSeverity::Critical,
                        description: brake_reason,
                        affected_files: vec![],
                        mitigation_required: true,
                        auto_fixable: false,
                    }],
                    required_confirmations: vec![RequiredConfirmation {
                        confirmation_type: ConfirmationType::ExplicitUserApproval,
                        message: "Emergency brake activated - manual override required".to_string(),
                        default_response: false,
                    }],
                    recommendations: vec!["Review system state and recent operations".to_string()],
                    execution_requirements: ExecutionRequirements::Manual,
                });
            }
        }

        // Run all core validators
        let mut validation_results = Vec::new();
        for validator in &self.core_validators {
            debug!("Running validator: {}", validator.name());
            match validator.validate(operation, context) {
                Ok(result) => validation_results.push(result),
                Err(e) => {
                    error!("Validator {} failed: {}", validator.name(), e);
                    validation_results.push(SafetyValidationResult {
                        validator_name: validator.name().to_string(),
                        is_safe: false,
                        risk_score: 50.0,
                        violations: vec![SafetyViolation {
                            rule_id: format!("{}_ERROR", validator.name().to_uppercase()),
                            severity: ViolationSeverity::Medium,
                            description: format!("Validator error: {}", e),
                            affected_files: vec![],
                            mitigation_required: false,
                            auto_fixable: false,
                        }],
                        warnings: vec![format!("Validator {} encountered an error", validator.name())],
                        required_confirmations: Vec::new(),
                        suggested_mitigations: Vec::new(),
                    });
                }
            }
        }

        // Aggregate results
        let overall_safe = validation_results.iter().all(|r| r.is_safe);
        let max_risk_score = validation_results.iter()
            .map(|r| r.risk_score)
            .fold(0.0, f32::max);

        // Collect all violations
        let mut all_violations = Vec::new();
        let mut all_confirmations = Vec::new();
        let mut all_recommendations = Vec::new();

        for result in &validation_results {
            all_violations.extend(result.violations.clone());
            all_confirmations.extend(result.required_confirmations.clone());
            all_recommendations.extend(result.suggested_mitigations.clone());
        }

        // Determine enforcement action
        let enforcement_action = self.determine_enforcement_action(
            &all_violations,
            max_risk_score,
            analysis,
        ).await;

        // Determine execution requirements
        let execution_requirements = self.determine_execution_requirements(
            &enforcement_action,
            &all_violations,
            max_risk_score,
        ).await;

        // Update safety metrics
        self.update_safety_metrics(&validation_results, &enforcement_action).await;

        Ok(ComprehensiveSafetyResult {
            overall_safe,
            risk_score: max_risk_score,
            enforcement_action,
            validation_results,
            safety_violations: all_violations,
            required_confirmations: all_confirmations,
            recommendations: all_recommendations,
            execution_requirements,
        })
    }

    /// Check if an operation should be automatically blocked
    pub async fn should_block_operation(
        &self,
        safety_result: &ComprehensiveSafetyResult,
    ) -> bool {
        matches!(
            safety_result.enforcement_action,
            EnforcementAction::Block | EnforcementAction::RequireManualOverride
        )
    }

    /// Get current safety metrics and statistics
    pub async fn get_safety_metrics(&self) -> SafetyMetrics {
        self.safety_metrics.read().await.clone()
    }

    /// Add a custom safety rule
    pub async fn add_safety_rule(&self, rule: SafetyRule) -> Result<()> {
        let mut rules = self.pattern_rules.write().await;
        rules.push(rule);
        info!("Added new safety rule");
        Ok(())
    }

    /// Remove a safety rule
    pub async fn remove_safety_rule(&self, rule_id: &str) -> Result<bool> {
        let mut rules = self.pattern_rules.write().await;
        let initial_len = rules.len();
        rules.retain(|rule| rule.rule_id != rule_id);
        let removed = rules.len() < initial_len;
        if removed {
            info!("Removed safety rule: {}", rule_id);
        }
        Ok(removed)
    }

    /// Get all current safety rules
    pub async fn get_safety_rules(&self) -> Vec<SafetyRule> {
        self.pattern_rules.read().await.clone()
    }

    // Private helper methods

    fn create_default_safety_rules() -> Vec<SafetyRule> {
        vec![
            SafetyRule {
                rule_id: "CRITICAL_SYSTEM_FILES".to_string(),
                name: "Critical System Files Protection".to_string(),
                description: "Prevents modification of critical system files".to_string(),
                pattern: RulePattern::FilePathRegex(r"^/(etc|bin|sbin|usr/(bin|sbin))/".to_string()),
                severity: ViolationSeverity::Critical,
                enabled: true,
                created_at: Utc::now(),
                last_triggered: None,
                trigger_count: 0,
            },
            SafetyRule {
                rule_id: "DOT_FILES_PROTECTION".to_string(),
                name: "Hidden Files Protection".to_string(),
                description: "Requires confirmation for modifying hidden files".to_string(),
                pattern: RulePattern::FilePathRegex(r"/\.[^/]+$".to_string()),
                severity: ViolationSeverity::Medium,
                enabled: true,
                created_at: Utc::now(),
                last_triggered: None,
                trigger_count: 0,
            },
            SafetyRule {
                rule_id: "LARGE_FILE_WARNING".to_string(),
                name: "Large File Warning".to_string(),
                description: "Warns when modifying very large files".to_string(),
                pattern: RulePattern::FileSize { min: Some(100 * 1024 * 1024), max: None }, // 100MB+
                severity: ViolationSeverity::Low,
                enabled: true,
                created_at: Utc::now(),
                last_triggered: None,
                trigger_count: 0,
            },
            SafetyRule {
                rule_id: "GIT_DIRECTORY_PROTECTION".to_string(),
                name: "Git Directory Protection".to_string(),
                description: "Prevents direct modification of .git directory".to_string(),
                pattern: RulePattern::FilePathRegex(r"\.git/".to_string()),
                severity: ViolationSeverity::High,
                enabled: true,
                created_at: Utc::now(),
                last_triggered: None,
                trigger_count: 0,
            },
            SafetyRule {
                rule_id: "EXECUTABLE_CREATION".to_string(),
                name: "Executable Creation Warning".to_string(),
                description: "Warns when creating executable files".to_string(),
                pattern: RulePattern::FileExtension("exe|sh|bat|com|cmd".to_string()),
                severity: ViolationSeverity::Medium,
                enabled: true,
                created_at: Utc::now(),
                last_triggered: None,
                trigger_count: 0,
            },
        ]
    }

    async fn determine_enforcement_action(
        &self,
        violations: &[SafetyViolation],
        risk_score: f32,
        analysis: &OperationAnalysis,
    ) -> EnforcementAction {
        // Check for critical violations
        if violations.iter().any(|v| matches!(v.severity, ViolationSeverity::Critical)) {
            return EnforcementAction::Block;
        }

        // Check risk score against configuration
        if risk_score > self.config.max_auto_risk_score {
            return match self.config.enforcement_level {
                EnforcementLevel::Advisory => EnforcementAction::Warn,
                EnforcementLevel::Enforcing => EnforcementAction::RequireConfirmation,
                EnforcementLevel::Paranoid => EnforcementAction::Block,
            };
        }

        // Check for high severity violations
        let high_severity_count = violations.iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::High))
            .count();

        if high_severity_count > 0 {
            return match self.config.enforcement_level {
                EnforcementLevel::Advisory => EnforcementAction::Warn,
                EnforcementLevel::Enforcing => EnforcementAction::RequireConfirmation,
                EnforcementLevel::Paranoid => EnforcementAction::RequireManualOverride,
            };
        }

        // Check overall confidence and risk from AI analysis
        if analysis.unified_score.confidence < 60.0 || analysis.unified_score.risk > 70.0 {
            return EnforcementAction::RequireConfirmation;
        }

        // Default action based on enforcement level
        match self.config.enforcement_level {
            EnforcementLevel::Advisory => EnforcementAction::Allow,
            EnforcementLevel::Enforcing => {
                if violations.is_empty() {
                    EnforcementAction::Allow
                } else {
                    EnforcementAction::Warn
                }
            }
            EnforcementLevel::Paranoid => {
                if violations.is_empty() && risk_score < 20.0 {
                    EnforcementAction::Allow
                } else {
                    EnforcementAction::RequireConfirmation
                }
            }
        }
    }

    async fn determine_execution_requirements(
        &self,
        enforcement_action: &EnforcementAction,
        violations: &[SafetyViolation],
        risk_score: f32,
    ) -> ExecutionRequirements {
        match enforcement_action {
            EnforcementAction::Block => ExecutionRequirements::Blocked,
            EnforcementAction::RequireManualOverride => ExecutionRequirements::Manual,
            EnforcementAction::RequireConfirmation => {
                if risk_score > 70.0 || violations.iter().any(|v| v.mitigation_required) {
                    ExecutionRequirements::ConditionalWithMitigation
                } else {
                    ExecutionRequirements::ConditionalWithConfirmation
                }
            }
            EnforcementAction::Warn => ExecutionRequirements::AutoWithWarning,
            EnforcementAction::Allow => ExecutionRequirements::Auto,
        }
    }

    async fn update_safety_metrics(
        &self,
        validation_results: &[SafetyValidationResult],
        enforcement_action: &EnforcementAction,
    ) {
        let mut metrics = self.safety_metrics.write().await;
        
        metrics.total_validations += 1;
        
        match enforcement_action {
            EnforcementAction::Block => metrics.blocked_operations += 1,
            EnforcementAction::RequireManualOverride => metrics.manual_overrides_required += 1,
            EnforcementAction::RequireConfirmation => metrics.confirmations_required += 1,
            EnforcementAction::Warn => metrics.warnings_issued += 1,
            EnforcementAction::Allow => metrics.auto_approved += 1,
        }

        let total_violations: usize = validation_results.iter()
            .map(|r| r.violations.len())
            .sum();
        
        metrics.total_violations += total_violations as u64;
        metrics.last_updated = Utc::now();
    }
}

// Core validator implementations

pub struct CriticalFileValidator;

impl CriticalFileValidator {
    pub fn new() -> Self {
        Self
    }
}

impl SafetyValidator for CriticalFileValidator {
    fn name(&self) -> &'static str {
        "CriticalFileValidator"
    }

    fn priority(&self) -> u8 {
        255 // Highest priority
    }

    fn validate(&self, operation: &EnhancedFileOperation, _context: &SafetyContext) -> Result<SafetyValidationResult> {
        let mut violations = Vec::new();
        let mut risk_score = 0.0;

        // Check for critical system files
        let critical_patterns = [
            r"^/(etc|bin|sbin|usr/(bin|sbin))/",
            r"\.git/",
            r"node_modules/",
            r"\.env$",
            r"\.ssh/",
            r"id_rsa|id_ed25519",
        ];

        for pattern in &critical_patterns {
            let regex = Regex::new(pattern)?;
            if let Some(path) = get_operation_path(&operation.operation) {
                if regex.is_match(&path.display().to_string()) {
                    violations.push(SafetyViolation {
                        rule_id: format!("CRITICAL_FILE_{}", pattern),
                        severity: ViolationSeverity::Critical,
                        description: format!("Operation affects critical file: {}", path.display()),
                        affected_files: vec![path],
                        mitigation_required: true,
                        auto_fixable: false,
                    });
                    risk_score = 100.0;
                }
            }
        }

        Ok(SafetyValidationResult {
            validator_name: self.name().to_string(),
            is_safe: violations.is_empty(),
            risk_score,
            violations,
            warnings: Vec::new(),
            required_confirmations: Vec::new(),
            suggested_mitigations: if risk_score > 0.0 {
                vec!["Create backup before proceeding".to_string(),
                     "Ensure you have administrator privileges".to_string()]
            } else {
                Vec::new()
            },
        })
    }
}

pub struct SystemFileValidator;

impl SystemFileValidator {
    pub fn new() -> Self {
        Self
    }
}

impl SafetyValidator for SystemFileValidator {
    fn name(&self) -> &'static str {
        "SystemFileValidator"
    }

    fn priority(&self) -> u8 {
        200
    }

    fn validate(&self, operation: &EnhancedFileOperation, context: &SafetyContext) -> Result<SafetyValidationResult> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut risk_score = 0.0;

        if let Some(path) = get_operation_path(&operation.operation) {
            // Check if operation is within system directories
            let path_str = path.display().to_string();
            
            if path_str.starts_with("/System/") || path_str.starts_with("/usr/") {
                violations.push(SafetyViolation {
                    rule_id: "SYSTEM_DIRECTORY".to_string(),
                    severity: ViolationSeverity::High,
                    description: "Operation in system directory requires elevated privileges".to_string(),
                    affected_files: vec![path.clone()],
                    mitigation_required: true,
                    auto_fixable: false,
                });
                risk_score = 80.0;
            }

            // Check for backup files
            if path_str.ends_with(".bak") || path_str.ends_with("~") {
                warnings.push("Modifying backup file".to_string());
                risk_score = (risk_score + 20.0).min(100.0);
            }

            // Check available disk space
            if context.system_state.available_disk_space < 1024 * 1024 * 1024 { // Less than 1GB
                violations.push(SafetyViolation {
                    rule_id: "LOW_DISK_SPACE".to_string(),
                    severity: ViolationSeverity::Medium,
                    description: "Low disk space available".to_string(),
                    affected_files: vec![],
                    mitigation_required: false,
                    auto_fixable: false,
                });
                risk_score = (risk_score + 30.0).min(100.0);
            }
        }

        Ok(SafetyValidationResult {
            validator_name: self.name().to_string(),
            is_safe: violations.iter().all(|v| !matches!(v.severity, ViolationSeverity::Critical)),
            risk_score,
            violations,
            warnings,
            required_confirmations: Vec::new(),
            suggested_mitigations: vec!["Verify system permissions".to_string()],
        })
    }
}

// Additional validator implementations would follow similar patterns...

pub struct BackupRequiredValidator;
impl BackupRequiredValidator { pub fn new() -> Self { Self } }
impl SafetyValidator for BackupRequiredValidator {
    fn name(&self) -> &'static str { "BackupRequiredValidator" }
    fn priority(&self) -> u8 { 180 }
    fn validate(&self, _operation: &EnhancedFileOperation, context: &SafetyContext) -> Result<SafetyValidationResult> {
        let mut violations = Vec::new();
        
        if matches!(context.system_state.backup_system_status, BackupSystemStatus::Unavailable) {
            violations.push(SafetyViolation {
                rule_id: "BACKUP_UNAVAILABLE".to_string(),
                severity: ViolationSeverity::High,
                description: "Backup system is unavailable".to_string(),
                affected_files: vec![],
                mitigation_required: true,
                auto_fixable: false,
            });
        }

        Ok(SafetyValidationResult {
            validator_name: self.name().to_string(),
            is_safe: violations.is_empty(),
            risk_score: if violations.is_empty() { 0.0 } else { 70.0 },
            violations,
            warnings: Vec::new(),
            required_confirmations: Vec::new(),
            suggested_mitigations: vec!["Enable backup system before proceeding".to_string()],
        })
    }
}

pub struct GitIntegrityValidator;
impl GitIntegrityValidator { pub fn new() -> Self { Self } }
impl SafetyValidator for GitIntegrityValidator {
    fn name(&self) -> &'static str { "GitIntegrityValidator" }
    fn priority(&self) -> u8 { 150 }
    fn validate(&self, _operation: &EnhancedFileOperation, context: &SafetyContext) -> Result<SafetyValidationResult> {
        let mut warnings = Vec::new();
        let mut violations = Vec::new();
        let mut risk_score = 0.0;

        if let Some(ref git_status) = context.git_status {
            if git_status.has_uncommitted_changes {
                warnings.push("Repository has uncommitted changes".to_string());
                risk_score += 20.0;
            }

            if git_status.has_untracked_files {
                warnings.push("Repository has untracked files".to_string());
                risk_score += 10.0;
            }

            if !git_status.is_clean_working_tree {
                violations.push(SafetyViolation {
                    rule_id: "DIRTY_WORKING_TREE".to_string(),
                    severity: ViolationSeverity::Medium,
                    description: "Working tree is not clean".to_string(),
                    affected_files: vec![],
                    mitigation_required: false,
                    auto_fixable: true,
                });
                risk_score += 25.0;
            }
        }

        Ok(SafetyValidationResult {
            validator_name: self.name().to_string(),
            is_safe: violations.is_empty(),
            risk_score,
            violations,
            warnings,
            required_confirmations: Vec::new(),
            suggested_mitigations: vec!["Commit or stash changes before proceeding".to_string()],
        })
    }
}

pub struct PermissionValidator;
impl PermissionValidator { pub fn new() -> Self { Self } }
impl SafetyValidator for PermissionValidator {
    fn name(&self) -> &'static str { "PermissionValidator" }
    fn priority(&self) -> u8 { 120 }
    fn validate(&self, operation: &EnhancedFileOperation, _context: &SafetyContext) -> Result<SafetyValidationResult> {
        // Simplified permission check
        Ok(SafetyValidationResult {
            validator_name: self.name().to_string(),
            is_safe: true,
            risk_score: 0.0,
            violations: Vec::new(),
            warnings: Vec::new(),
            required_confirmations: Vec::new(),
            suggested_mitigations: Vec::new(),
        })
    }
}

pub struct DiskSpaceValidator;
impl DiskSpaceValidator { pub fn new() -> Self { Self } }
impl SafetyValidator for DiskSpaceValidator {
    fn name(&self) -> &'static str { "DiskSpaceValidator" }
    fn priority(&self) -> u8 { 100 }
    fn validate(&self, _operation: &EnhancedFileOperation, context: &SafetyContext) -> Result<SafetyValidationResult> {
        let low_space_threshold = 500 * 1024 * 1024; // 500MB
        let mut violations = Vec::new();
        
        if context.system_state.available_disk_space < low_space_threshold {
            violations.push(SafetyViolation {
                rule_id: "INSUFFICIENT_DISK_SPACE".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("Only {} bytes available", context.system_state.available_disk_space),
                affected_files: vec![],
                mitigation_required: false,
                auto_fixable: false,
            });
        }

        Ok(SafetyValidationResult {
            validator_name: self.name().to_string(),
            is_safe: violations.is_empty(),
            risk_score: if violations.is_empty() { 0.0 } else { 40.0 },
            violations,
            warnings: Vec::new(),
            required_confirmations: Vec::new(),
            suggested_mitigations: vec!["Free up disk space before proceeding".to_string()],
        })
    }
}

pub struct ConcurrencyValidator;
impl ConcurrencyValidator { pub fn new() -> Self { Self } }
impl SafetyValidator for ConcurrencyValidator {
    fn name(&self) -> &'static str { "ConcurrencyValidator" }
    fn priority(&self) -> u8 { 90 }
    fn validate(&self, _operation: &EnhancedFileOperation, context: &SafetyContext) -> Result<SafetyValidationResult> {
        // Check for concurrent risky operations
        let recent_risky_ops = context.recent_operations.iter()
            .filter(|op| {
                let time_diff = Utc::now() - op.timestamp;
                time_diff < Duration::seconds(300) && !op.success // Recent failed operations
            })
            .count();

        let mut violations = Vec::new();
        if recent_risky_ops > 2 {
            violations.push(SafetyViolation {
                rule_id: "TOO_MANY_RECENT_FAILURES".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("{} recent operation failures", recent_risky_ops),
                affected_files: vec![],
                mitigation_required: false,
                auto_fixable: false,
            });
        }

        Ok(SafetyValidationResult {
            validator_name: self.name().to_string(),
            is_safe: violations.is_empty(),
            risk_score: if violations.is_empty() { 0.0 } else { 30.0 },
            violations,
            warnings: Vec::new(),
            required_confirmations: Vec::new(),
            suggested_mitigations: vec!["Wait before retrying operations".to_string()],
        })
    }
}

pub struct PatternMatchValidator;
impl PatternMatchValidator { pub fn new() -> Self { Self } }
impl SafetyValidator for PatternMatchValidator {
    fn name(&self) -> &'static str { "PatternMatchValidator" }
    fn priority(&self) -> u8 { 80 }
    fn validate(&self, operation: &EnhancedFileOperation, _context: &SafetyContext) -> Result<SafetyValidationResult> {
        // This would check against pattern-based rules
        // Simplified implementation
        Ok(SafetyValidationResult {
            validator_name: self.name().to_string(),
            is_safe: true,
            risk_score: 0.0,
            violations: Vec::new(),
            warnings: Vec::new(),
            required_confirmations: Vec::new(),
            suggested_mitigations: Vec::new(),
        })
    }
}

// Supporting structures and systems

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveSafetyResult {
    pub overall_safe: bool,
    pub risk_score: f32,
    pub enforcement_action: EnforcementAction,
    pub validation_results: Vec<SafetyValidationResult>,
    pub safety_violations: Vec<SafetyViolation>,
    pub required_confirmations: Vec<RequiredConfirmation>,
    pub recommendations: Vec<String>,
    pub execution_requirements: ExecutionRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementAction {
    Allow,                      // Operation is safe to proceed
    Warn,                       // Proceed with warnings
    RequireConfirmation,        // Require explicit user confirmation
    RequireManualOverride,      // Require manual override with justification
    Block,                      // Block operation entirely
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionRequirements {
    Auto,                                    // Can execute automatically
    AutoWithWarning,                         // Execute but show warnings
    ConditionalWithConfirmation,             // Require user confirmation
    ConditionalWithMitigation,               // Require mitigation steps first
    Manual,                                  // Require manual execution
    Blocked,                                 // Cannot execute
}

pub struct DynamicRiskAssessor;
impl DynamicRiskAssessor {
    pub fn new() -> Self { Self }
}

pub struct SafetyPolicyEnforcer;
impl SafetyPolicyEnforcer {
    pub fn new() -> Self { Self }
}

pub struct EmergencyBrakeSystem;
impl EmergencyBrakeSystem {
    pub fn new() -> Self { Self }
    
    pub async fn should_brake(
        &self,
        _operation: &EnhancedFileOperation,
        context: &SafetyContext,
    ) -> Result<Option<String>> {
        // Check for emergency conditions
        if context.system_state.system_load > 0.95 {
            return Ok(Some("System overload detected".to_string()));
        }
        
        if matches!(context.system_state.backup_system_status, BackupSystemStatus::Unavailable) {
            let recent_failures = context.recent_operations.iter()
                .filter(|op| !op.success && Utc::now() - op.timestamp < Duration::minutes(5))
                .count();
            
            if recent_failures > 3 {
                return Ok(Some("Multiple recent failures with no backup system".to_string()));
            }
        }
        
        Ok(None)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SafetyMetrics {
    pub total_validations: u64,
    pub blocked_operations: u64,
    pub manual_overrides_required: u64,
    pub confirmations_required: u64,
    pub warnings_issued: u64,
    pub auto_approved: u64,
    pub total_violations: u64,
    pub last_updated: DateTime<Utc>,
}

// Helper functions

fn get_operation_path(operation: &FileOperation) -> Option<PathBuf> {
    match operation {
        FileOperation::Create { path, .. } |
        FileOperation::Update { path, .. } |
        FileOperation::Delete { path } => Some(path.clone()),
        FileOperation::Rename { new_path, .. } => Some(new_path.clone()),
        FileOperation::Move { destination, .. } => Some(destination.clone()),
    }
}