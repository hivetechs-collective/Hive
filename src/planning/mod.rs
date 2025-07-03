//! Planning Engine for HiveTechs Consensus
//! 
//! Provides intelligent task decomposition, risk analysis, timeline estimation,
//! and dependency resolution for complex development tasks.

pub mod decomposer;
pub mod risk_analyzer;
pub mod timeline;
pub mod dependency_resolver;
pub mod collaborative;
pub mod mode_detector;
pub mod mode_switcher;
pub mod types;
pub mod integration;

use crate::core::error::HiveResult;
use crate::consensus::engine::ConsensusEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::path::Path;

pub use self::types::{Plan, Task, Risk, Dependency, ModeType, PlanningContext};
pub use self::decomposer::TaskDecomposer;
pub use self::risk_analyzer::RiskAnalyzer;
pub use self::timeline::TimelineEstimator;
pub use self::dependency_resolver::DependencyResolver;
pub use self::collaborative::CollaborativePlanner;
pub use self::mode_detector::ModeDetector;
pub use self::mode_switcher::ModeSwitcher;
pub use self::integration::{RepositoryIntelligence, RepositoryContext};

/// Planning Engine - orchestrates task planning and execution
pub struct PlanningEngine {
    decomposer: TaskDecomposer,
    risk_analyzer: RiskAnalyzer,
    timeline_estimator: TimelineEstimator,
    dependency_resolver: DependencyResolver,
    collaborative_planner: CollaborativePlanner,
    mode_detector: ModeDetector,
    mode_switcher: ModeSwitcher,
    pub(crate) consensus_engine: ConsensusEngine,
    repository_intelligence: RepositoryIntelligence,
}

impl PlanningEngine {
    /// Create a new planning engine
    pub async fn new(consensus_engine: ConsensusEngine) -> HiveResult<Self> {
        Ok(Self {
            decomposer: TaskDecomposer::new(),
            risk_analyzer: RiskAnalyzer::new(),
            timeline_estimator: TimelineEstimator::new(),
            dependency_resolver: DependencyResolver::new(),
            collaborative_planner: CollaborativePlanner::new(),
            mode_detector: ModeDetector::new(),
            mode_switcher: ModeSwitcher::new(),
            consensus_engine,
            repository_intelligence: RepositoryIntelligence::new().await?,
        })
    }

    /// Create a comprehensive plan from a high-level task description
    pub async fn create_plan(&self, description: &str, context: PlanningContext) -> HiveResult<Plan> {
        // Detect the appropriate mode
        let mode = self.mode_detector.detect_mode(description, &context)?;
        
        // Decompose the task into subtasks using AI
        let tasks = self.decomposer.decompose(description, &context, &self.consensus_engine).await?;
        
        // Analyze dependencies between tasks
        let dependency_graph = self.dependency_resolver.resolve(&tasks)?;
        
        // Estimate timelines for each task
        let timeline = self.timeline_estimator.estimate(&tasks, &dependency_graph)?;
        
        // Identify and analyze risks
        let risks = self.risk_analyzer.analyze(&tasks, &context)?;
        
        // Create the plan
        let mut plan = Plan {
            id: uuid::Uuid::new_v4().to_string(),
            title: self.generate_title(description).await?,
            description: description.to_string(),
            mode,
            tasks,
            risks,
            dependencies: dependency_graph,
            timeline,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            context,
            metadata: HashMap::new(),
        };
        
        // Optimize the plan
        self.optimize_plan(&mut plan)?;
        
        Ok(plan)
    }

    /// Create a plan with repository context
    pub async fn create_plan_with_repository(
        &mut self,
        description: &str,
        repository_path: &Path,
        mut context: PlanningContext,
    ) -> HiveResult<Plan> {
        // Analyze repository for context
        let repo_context = self.repository_intelligence.analyze_repository(repository_path).await?;
        
        // Enhance planning context with repository intelligence
        context = self.repository_intelligence.enhance_planning_context(context, &repo_context)?;
        
        // Add repository metrics to metadata
        let mut metadata = HashMap::new();
        metadata.insert("repository_analyzed".to_string(), serde_json::json!(true));
        metadata.insert("code_quality_score".to_string(), serde_json::json!(repo_context.quality_indicators.code_quality_score));
        metadata.insert("technical_debt_score".to_string(), serde_json::json!(repo_context.code_metrics.technical_debt.score));
        
        // Create plan with enhanced context
        let mut plan = self.create_plan(description, context).await?;
        
        // Enhance tasks with repository-specific context
        for task in &mut plan.tasks {
            let task_context = self.repository_intelligence.get_task_context(task, &repo_context).await?;
            
            // Add relevant files to task metadata
            if !task_context.relevant_files.is_empty() {
                let files_json = task_context.relevant_files.iter()
                    .map(|f| f.to_string_lossy().to_string())
                    .collect::<Vec<_>>();
                metadata.insert(format!("task_{}_files", task.id), serde_json::json!(files_json));
            }
            
            // Add suggested approach to task description
            if !task_context.suggested_approach.is_empty() {
                task.description = format!("{}\n\nSuggested Approach: {}", 
                    task.description, 
                    task_context.suggested_approach
                );
            }
        }
        
        plan.metadata = metadata;
        Ok(plan)
    }

    /// Execute a plan with validation and progress tracking
    pub async fn execute_plan(&self, plan: &Plan, validate: bool) -> HiveResult<PlanExecutionResult> {
        let mut results = Vec::new();
        let mut failed_tasks = Vec::new();
        
        // Get execution order from dependency resolver
        let execution_order = self.dependency_resolver.get_execution_order(&plan.dependencies)?;
        
        for task_id in execution_order {
            let task = plan.tasks.iter().find(|t| t.id == task_id)
                .ok_or_else(|| crate::core::error::HiveError::Planning(
                    format!("Task {} not found in plan", task_id)
                ))?;
            
            if validate {
                // Validate dependencies are satisfied
                if !self.validate_dependencies(task, &results)? {
                    failed_tasks.push(task.clone());
                    continue;
                }
            }
            
            // Execute the task (in real implementation, this would trigger actual work)
            let result = self.execute_task(task, &plan.mode).await?;
            results.push(result);
        }
        
        Ok(PlanExecutionResult {
            plan_id: plan.id.clone(),
            executed_tasks: results.len(),
            failed_tasks,
            duration: std::time::Duration::from_secs(0), // TODO: Track actual duration
            mode: plan.mode.clone(),
        })
    }

    /// Switch between planning and execution modes
    pub async fn switch_mode(&self, current_mode: &ModeType, target_mode: ModeType, context: &PlanningContext) -> HiveResult<ModeType> {
        self.mode_switcher.switch(current_mode, target_mode, context).await
    }

    /// Detect the appropriate mode for a given query
    pub fn detect_mode(&self, query: &str, context: &PlanningContext) -> HiveResult<ModeType> {
        self.mode_detector.detect_mode(query, context)
    }

    /// Get mode recommendation with explanation
    pub fn get_mode_recommendation(&self, query: &str, context: &PlanningContext) -> HiveResult<ModeRecommendation> {
        let mode = self.detect_mode(query, context)?;
        let confidence = self.mode_detector.get_confidence(query, &mode)?;
        let explanation = self.mode_detector.explain_choice(query, &mode)?;
        
        Ok(ModeRecommendation {
            mode,
            confidence,
            explanation,
            alternatives: self.mode_detector.get_alternatives(query, context)?,
        })
    }

    // Private helper methods
    
    async fn generate_title(&self, description: &str) -> HiveResult<String> {
        // Use consensus engine to generate a concise title
        let prompt = format!("Generate a concise title (max 6 words) for this task: {}", description);
        let result = self.consensus_engine.process(&prompt, None).await?;
        let title = result.final_response;
        Ok(title.trim().to_string())
    }

    fn optimize_plan(&self, plan: &mut Plan) -> HiveResult<()> {
        // Optimize task ordering for efficiency
        self.dependency_resolver.optimize_ordering(&mut plan.tasks, &plan.dependencies)?;
        
        // Adjust timelines based on parallelization opportunities
        self.timeline_estimator.optimize_timeline(&mut plan.timeline, &plan.dependencies)?;
        
        // Re-evaluate risks after optimization
        plan.risks = self.risk_analyzer.analyze(&plan.tasks, &plan.context)?;
        
        Ok(())
    }

    fn validate_dependencies(&self, task: &Task, completed_results: &[TaskExecutionResult]) -> HiveResult<bool> {
        for dep_id in &task.dependencies {
            let is_completed = completed_results.iter().any(|r| &r.task_id == dep_id && r.success);
            if !is_completed {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn execute_task(&self, task: &Task, mode: &ModeType) -> HiveResult<TaskExecutionResult> {
        // In a real implementation, this would execute the actual task
        // For now, we'll simulate execution
        Ok(TaskExecutionResult {
            task_id: task.id.clone(),
            success: true,
            output: format!("Task '{}' executed successfully", task.title),
            duration: std::time::Duration::from_secs(1),
            timestamp: Utc::now(),
        })
    }
}

/// Result of plan execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExecutionResult {
    pub plan_id: String,
    pub executed_tasks: usize,
    pub failed_tasks: Vec<Task>,
    pub duration: std::time::Duration,
    pub mode: ModeType,
}

/// Result of individual task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    pub task_id: String,
    pub success: bool,
    pub output: String,
    pub duration: std::time::Duration,
    pub timestamp: DateTime<Utc>,
}

/// Mode recommendation with confidence and explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeRecommendation {
    pub mode: ModeType,
    pub confidence: f32,
    pub explanation: String,
    pub alternatives: Vec<(ModeType, f32)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_planning_engine_creation() {
        // Test planning engine initialization
    }

    #[tokio::test]
    async fn test_plan_creation() {
        // Test creating a plan from description
    }

    #[tokio::test]
    async fn test_mode_detection() {
        // Test mode detection for various queries
    }
}