//! Enterprise Hooks System for HiveTechs Consensus
//! 
//! Provides event-driven automation and enterprise control over the consensus pipeline.
//! Supports secure hook execution, approval workflows, and comprehensive audit logging.

pub mod registry;
pub mod execution;
pub mod events;
pub mod conditions;
pub mod config;
pub mod security;
pub mod audit;
pub mod approval;
pub mod rbac;
pub mod dispatcher;
pub mod consensus_integration;
pub mod approval_workflow;
pub mod quality_gates;
pub mod cost_control;

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use registry::{HookRegistry, Hook, HookId, HookAction, HookMetadata};
pub use execution::{HookExecutor, ExecutionContext, ExecutionResult};
pub use events::{HookEvent, EventType, EventHandler, EventSource, EventBuilder};
pub use conditions::{HookCondition, ConditionEvaluator};
pub use config::{HookConfig, HookLoader};
pub use security::{HookSecurityValidator, SecurityPolicy};
pub use audit::{HookAuditLogger, AuditEvent, AuditEventType};
pub use approval::{ApprovalWorkflow as BaseApprovalWorkflow, ApprovalRequest as BaseApprovalRequest, ApprovalStatus as BaseApprovalStatus};
pub use rbac::{HookRbacManager, User, Role, Team, Permission};
pub use dispatcher::{EventDispatcher, DispatcherConfig, RoutingRule, DispatcherStats};
pub use consensus_integration::{
    ConsensusIntegration, ConsensusIntegrationConfig, StageHookResult,
    QualityGateResult, CostSummary, QualityGateStatus, PerformanceStatus,
};
pub use approval_workflow::{
    ApprovalWorkflow as EnhancedApprovalWorkflow,
    ApprovalWorkflowConfig, ApprovalRequest as EnhancedApprovalRequest,
    ApprovalDecision, ApprovalStatus as EnhancedApprovalStatus,
    ApprovalPriority, ApprovalProcessResult, ApprovalStatistics,
};
pub use quality_gates::{
    QualityGateManager, QualityGate, QualityCriterion, QualityMetric,
    QualityThreshold, QualityGateAction, QualityViolation, QualityFailureType,
    QualityViolationSeverity, QualityEvaluationResult, QualityActionRequired,
    QualityStatistics, RemediationSuggestion,
};
pub use cost_control::{
    CostController, CostControlConfig, Budget, CostThreshold,
    CostEstimationContext, CostEstimate, CostApprovalRequirement,
    BudgetUpdateResult, CostSummary as DetailedCostSummary,
    CostOptimizationRecommendation, CostAlert, AlertSeverity,
};

/// Main hooks system that coordinates all hook-related functionality
pub struct HooksSystem {
    registry: Arc<RwLock<HookRegistry>>,
    executor: Arc<HookExecutor>,
    event_handler: Arc<EventHandler>,
    event_dispatcher: Arc<EventDispatcher>,
    condition_evaluator: Arc<ConditionEvaluator>,
    security_validator: Arc<HookSecurityValidator>,
    audit_logger: Arc<HookAuditLogger>,
    approval_workflow: Arc<approval_workflow::ApprovalWorkflow>,
    rbac_manager: Arc<rbac::HookRbacManager>,
}

impl HooksSystem {
    /// Create a new hooks system with the given configuration
    pub async fn new(config_dir: PathBuf) -> Result<Self> {
        let registry = Arc::new(RwLock::new(HookRegistry::new()));
        let security_validator = Arc::new(HookSecurityValidator::new()?);
        let audit_logger = Arc::new(HookAuditLogger::new(config_dir.join("hooks_audit.log")).await?);
        let approval_workflow = Arc::new(approval_workflow::ApprovalWorkflow::new());
        let rbac_manager = Arc::new(rbac::HookRbacManager::new());
        
        // Initialize default roles
        rbac_manager.initialize().await?;
        
        let executor = Arc::new(HookExecutor::new(
            security_validator.clone(),
            audit_logger.clone(),
            approval_workflow.clone(),
        ));
        
        let event_handler = Arc::new(EventHandler::new(
            registry.clone(),
            executor.clone(),
        ));
        
        let event_dispatcher = Arc::new(EventDispatcher::new(
            DispatcherConfig::default(),
            event_handler.clone(),
        ));
        
        let condition_evaluator = Arc::new(ConditionEvaluator::new());
        
        Ok(Self {
            registry,
            executor,
            event_handler,
            event_dispatcher,
            condition_evaluator,
            security_validator,
            audit_logger,
            approval_workflow,
            rbac_manager,
        })
    }
    
    /// Load hooks from configuration directory
    pub async fn load_hooks(&self, hooks_dir: PathBuf) -> Result<()> {
        let loader = HookLoader::new(self.security_validator.clone());
        let hooks = loader.load_from_directory(hooks_dir).await?;
        
        let mut registry = self.registry.write().await;
        for hook in hooks {
            registry.register(hook)?;
        }
        
        Ok(())
    }
    
    /// Register a new hook
    pub async fn register_hook(&self, config_path: PathBuf) -> Result<HookId> {
        let loader = HookLoader::new(self.security_validator.clone());
        let hook = loader.load_from_file(config_path).await?;
        
        let hook_id = hook.id.clone();
        let mut registry = self.registry.write().await;
        registry.register(hook)?;
        
        self.audit_logger.log_hook_registered(&hook_id).await?;
        
        Ok(hook_id)
    }
    
    /// List all registered hooks
    pub async fn list_hooks(&self) -> Result<Vec<Hook>> {
        let registry = self.registry.read().await;
        Ok(registry.list_all())
    }
    
    /// Remove a hook
    pub async fn remove_hook(&self, hook_id: &HookId) -> Result<()> {
        let mut registry = self.registry.write().await;
        registry.unregister(hook_id)?;
        
        self.audit_logger.log_hook_removed(hook_id).await?;
        
        Ok(())
    }
    
    /// Enable a hook
    pub async fn enable_hook(&self, hook_id: &HookId) -> Result<()> {
        let mut registry = self.registry.write().await;
        registry.set_enabled(hook_id, true)?;
        
        self.audit_logger.log_hook_enabled(hook_id).await?;
        
        Ok(())
    }
    
    /// Disable a hook
    pub async fn disable_hook(&self, hook_id: &HookId) -> Result<()> {
        let mut registry = self.registry.write().await;
        registry.set_enabled(hook_id, false)?;
        
        self.audit_logger.log_hook_disabled(hook_id).await?;
        
        Ok(())
    }
    
    /// Test a hook configuration without executing
    pub async fn test_hook(&self, config_path: PathBuf, event: HookEvent) -> Result<bool> {
        let loader = HookLoader::new(self.security_validator.clone());
        let hook = loader.load_from_file(config_path).await?;
        
        // Evaluate conditions
        let context = ExecutionContext::from_event(&event)?;
        let passes = self.condition_evaluator.evaluate(&hook.conditions, &context).await?;
        
        Ok(passes)
    }
    
    /// Dispatch an event to trigger matching hooks
    pub async fn dispatch_event(&self, event: HookEvent) -> Result<()> {
        // Use dispatcher for priority queue processing
        self.event_dispatcher.dispatch(event).await
    }
    
    /// Start the hooks system (starts dispatcher workers)
    pub async fn start(&mut self) -> Result<()> {
        // Start the event dispatcher
        Arc::get_mut(&mut self.event_dispatcher)
            .ok_or_else(|| anyhow::anyhow!("Cannot start dispatcher - already in use"))?
            .start()
            .await?;
        
        // Start background queue processor for event handler
        self.event_handler.clone().spawn_queue_processor();
        
        Ok(())
    }
    
    /// Get dispatcher statistics
    pub async fn get_dispatcher_stats(&self) -> DispatcherStats {
        self.event_dispatcher.get_stats().await
    }
    
    /// Get audit logs for hooks
    pub async fn get_audit_logs(&self, limit: usize) -> Result<Vec<AuditEvent>> {
        self.audit_logger.get_recent_logs(limit).await
    }
    
    /// Clear all hooks (requires confirmation)
    pub async fn clear_all_hooks(&self, confirm: bool) -> Result<()> {
        if !confirm {
            return Err(anyhow::anyhow!("Confirmation required to clear all hooks"));
        }
        
        let mut registry = self.registry.write().await;
        registry.clear_all()?;
        
        self.audit_logger.log_all_hooks_cleared().await?;
        
        Ok(())
    }
    
    /// Get access to RBAC manager
    pub fn rbac(&self) -> Arc<rbac::HookRbacManager> {
        self.rbac_manager.clone()
    }
    
    /// Check if user has permission for hook operation
    pub async fn check_user_permission(&self, user_id: &str, permission: &Permission, hook_id: Option<&HookId>) -> Result<bool> {
        self.rbac_manager.check_permission(user_id, permission, hook_id).await
    }
    
    /// Create a new team for hook management
    pub async fn create_team(&self, team: Team) -> Result<()> {
        self.rbac_manager.create_team(team).await
    }
    
    /// Add user to team
    pub async fn add_user_to_team(&self, user_id: &str, team_name: &str) -> Result<()> {
        self.rbac_manager.add_user_to_team(user_id, team_name).await
    }
    
    /// Remove user from team
    pub async fn remove_user_from_team(&self, user_id: &str, team_name: &str) -> Result<()> {
        self.rbac_manager.remove_user_from_team(user_id, team_name).await
    }
    
    /// Grant team access to specific hooks
    pub async fn grant_team_hook_access(&self, team_name: &str, hook_ids: Vec<HookId>) -> Result<()> {
        let team = self.rbac_manager.get_team(team_name).await?
            .ok_or_else(|| anyhow::anyhow!("Team not found: {}", team_name))?;
        
        let mut updated_team = team;
        for hook_id in hook_ids {
            updated_team.hook_access.allowed_hooks.insert(hook_id);
        }
        
        self.rbac_manager.update_team(updated_team).await
    }
    
    /// Revoke team access to specific hooks
    pub async fn revoke_team_hook_access(&self, team_name: &str, hook_ids: Vec<HookId>) -> Result<()> {
        let team = self.rbac_manager.get_team(team_name).await?
            .ok_or_else(|| anyhow::anyhow!("Team not found: {}", team_name))?;
        
        let mut updated_team = team;
        for hook_id in hook_ids {
            updated_team.hook_access.allowed_hooks.remove(&hook_id);
        }
        
        self.rbac_manager.update_team(updated_team).await
    }
    
    /// List all teams
    pub async fn list_teams(&self) -> Result<Vec<Team>> {
        self.rbac_manager.list_teams().await
    }
    
    /// Get hooks accessible to a team
    pub async fn get_team_hooks(&self, team_name: &str) -> Result<Vec<Hook>> {
        let team = self.rbac_manager.get_team(team_name).await?
            .ok_or_else(|| anyhow::anyhow!("Team not found: {}", team_name))?;
        
        let all_hooks = self.list_hooks().await?;
        
        let accessible_hooks = all_hooks.into_iter()
            .filter(|hook| {
                // Check if team has explicit access to this hook
                if team.hook_access.allowed_hooks.contains(&hook.id) {
                    return true;
                }
                
                // Check if team is explicitly denied access
                if team.hook_access.denied_hooks.contains(&hook.id) {
                    return false;
                }
                
                // Check pattern matching
                for pattern in &team.hook_access.hook_patterns {
                    if self.matches_hook_pattern(&hook.name, pattern) {
                        return true;
                    }
                }
                
                false
            })
            .collect();
        
        Ok(accessible_hooks)
    }
    
    /// Check if hook name matches pattern
    fn matches_hook_pattern(&self, hook_name: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        // Simple glob matching
        if pattern.contains('*') {
            let regex_pattern = pattern
                .replace('.', r"\.")
                .replace('*', ".*")
                .replace('?', ".");
            
            if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                return regex.is_match(hook_name);
            }
        }
        
        hook_name == pattern
    }
}

/// Hook priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HookPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for HookPriority {
    fn default() -> Self {
        Self::Normal
    }
}