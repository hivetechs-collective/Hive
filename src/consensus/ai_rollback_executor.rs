//! AI-powered Rollback Executor Bridge for Consensus
//!
//! This module bridges the consensus rollback planning with AI Helper execution,
//! maintaining the separation where consensus THINKS and AI Helpers DO.

use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::ai_helpers::{
    AIHelperEcosystem, AIHelperRollbackExecutor, RollbackPlan, RollbackResult,
};
use crate::consensus::operation_history::OperationHistoryDatabase;
use crate::consensus::operation_intelligence::OperationIntelligenceCoordinator;
use crate::consensus::rollback_planner::RollbackPlan as LegacyRollbackPlan;
use crate::consensus::rollback_planner_v2::{
    ConsensusRollbackPlanner, RollbackFeasibilityAnalysis, RollbackPlanningConfig,
};
use crate::consensus::stages::file_aware_curator::FileOperation;

/// AI-powered consensus rollback executor that delegates to AI Helpers
pub struct AIConsensusRollbackExecutor {
    planner: ConsensusRollbackPlanner,
    ai_executor: AIHelperRollbackExecutor,
}

impl AIConsensusRollbackExecutor {
    pub fn new(
        ai_helpers: AIHelperEcosystem,
        operation_intelligence: Arc<OperationIntelligenceCoordinator>,
        history_database: Option<Arc<OperationHistoryDatabase>>,
    ) -> Self {
        let planner = ConsensusRollbackPlanner::new(
            operation_intelligence,
            history_database,
            RollbackPlanningConfig::default(),
        );

        let ai_executor = AIHelperRollbackExecutor::new(ai_helpers);

        Self {
            planner,
            ai_executor,
        }
    }

    /// Plan and execute rollback for failed operations
    pub async fn rollback_failed_operations(
        &self,
        failed_operations: Vec<FileOperation>,
        failure_context: &str,
    ) -> Result<RollbackResult> {
        info!(
            "🔄 Planning rollback for {} failed operations",
            failed_operations.len()
        );

        // Step 1: Create rollback plan (consensus THINKS)
        let plan = self
            .planner
            .create_rollback_plan(failed_operations, failure_context)
            .await?;

        info!(
            "📋 Created rollback plan {} with {} operations",
            plan.plan_id,
            plan.operations.len()
        );

        // Step 2: Analyze feasibility
        let feasibility = self.planner.analyze_rollback_feasibility(&plan).await?;

        if feasibility.feasibility_score < 0.5 {
            warn!(
                "⚠️ Rollback plan has low feasibility: {:.1}%",
                feasibility.feasibility_score * 100.0
            );
            for issue in &feasibility.issues {
                warn!("  - {}", issue);
            }
        }

        // Step 3: Execute plan via AI Helper (AI Helper DOES)
        info!("🤖 Delegating rollback execution to AI Helper");
        self.ai_executor.execute_rollback_plan(plan).await
    }

    /// Convert and execute a legacy rollback plan
    pub async fn execute_legacy_plan(
        &self,
        legacy_plan: LegacyRollbackPlan,
    ) -> Result<RollbackResult> {
        info!("🔄 Converting legacy rollback plan: {}", legacy_plan.id);

        // Convert legacy plan to new format
        let plan = self.planner.convert_legacy_plan(legacy_plan)?;

        info!(
            "📋 Converted to new plan format with {} operations",
            plan.operations.len()
        );

        // Execute via AI Helper
        self.ai_executor.execute_rollback_plan(plan).await
    }

    /// Create a rollback plan without executing (for review)
    pub async fn create_rollback_plan(
        &self,
        failed_operations: Vec<FileOperation>,
        failure_context: &str,
    ) -> Result<RollbackPlan> {
        self.planner
            .create_rollback_plan(failed_operations, failure_context)
            .await
    }

    /// Analyze rollback feasibility without executing
    pub async fn analyze_feasibility(
        &self,
        plan: &RollbackPlan,
    ) -> Result<RollbackFeasibilityAnalysis> {
        self.planner.analyze_rollback_feasibility(plan).await
    }

    /// Execute a pre-created rollback plan
    pub async fn execute_plan(&self, plan: RollbackPlan) -> Result<RollbackResult> {
        info!("🤖 Executing rollback plan: {}", plan.plan_id);
        self.ai_executor.execute_rollback_plan(plan).await
    }

    /// Dry run a rollback plan
    pub async fn dry_run_plan(&self, plan: RollbackPlan) -> Result<RollbackResult> {
        info!("🏃 Dry run rollback plan: {}", plan.plan_id);

        // Create a dry-run executor
        let dry_run_executor = AIHelperRollbackExecutor::new(
            AIHelperEcosystem::new_mock(), // Use mock for dry run
        )
        .dry_run(true);

        dry_run_executor.execute_rollback_plan(plan).await
    }

    /// Create partial rollback plan
    pub async fn create_partial_rollback(
        &self,
        all_operations: Vec<FileOperation>,
        operations_to_rollback: Vec<usize>,
    ) -> Result<RollbackPlan> {
        self.planner
            .create_partial_rollback_plan(all_operations, operations_to_rollback)
            .await
    }

    /// Verify individual operations can be rolled back
    pub async fn verify_operations(&self, plan: &RollbackPlan) -> Result<Vec<(String, bool)>> {
        let mut results = Vec::new();

        for operation in &plan.operations {
            let can_execute = self
                .ai_executor
                .verify_operation(operation)
                .await
                .unwrap_or(false);

            results.push((operation.operation_id.clone(), can_execute));
        }

        Ok(results)
    }
}

/// Configuration for AI rollback execution
#[derive(Debug, Clone)]
pub struct AIRollbackConfig {
    /// Whether to use intelligent executor for enhanced rollback
    pub use_intelligent_executor: bool,
    /// Whether to create additional safety backups
    pub create_safety_backups: bool,
    /// Maximum retry attempts for failed operations
    pub max_retries: u32,
    /// Whether to continue on partial failures
    pub continue_on_failure: bool,
}

impl Default for AIRollbackConfig {
    fn default() -> Self {
        Self {
            use_intelligent_executor: true,
            create_safety_backups: true,
            max_retries: 3,
            continue_on_failure: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rollback_planning() {
        let ai_helpers = AIHelperEcosystem::new_mock();
        let intelligence = Arc::new(OperationIntelligenceCoordinator::new_mock());

        let executor = AIConsensusRollbackExecutor::new(ai_helpers, intelligence, None);

        // Test creating a rollback plan
        let failed_ops = vec![FileOperation::Create {
            path: std::path::PathBuf::from("test.txt"),
            content: "test content".to_string(),
        }];

        let plan = executor
            .create_rollback_plan(failed_ops, "Test failure")
            .await
            .unwrap();

        assert_eq!(plan.operations.len(), 1);
        assert!(matches!(
            &plan.operations[0].action,
            crate::ai_helpers::RollbackAction::DeleteCreatedFile { .. }
        ));
    }

    #[tokio::test]
    async fn test_feasibility_analysis() {
        let ai_helpers = AIHelperEcosystem::new_mock();
        let intelligence = Arc::new(OperationIntelligenceCoordinator::new_mock());

        let executor = AIConsensusRollbackExecutor::new(ai_helpers, intelligence, None);

        let plan = RollbackPlan {
            plan_id: "test-plan".to_string(),
            operations: vec![],
            safety_level: crate::ai_helpers::RollbackSafetyLevel::Low,
            verification_required: true,
            created_at: chrono::Utc::now(),
        };

        let feasibility = executor.analyze_feasibility(&plan).await.unwrap();

        assert_eq!(feasibility.feasibility_score, 1.0); // Empty plan is 100% feasible
    }
}
