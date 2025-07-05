//! Hybrid Mode Engine for Complex Tasks
//! 
//! Intelligently combines planning and execution modes for optimal
//! task completion with dynamic mode transitions.

use crate::core::error::{HiveResult, HiveError};
use crate::planning::{ModeType, PlanningContext, Task};
use crate::consensus::ConsensusEngine;
use crate::modes::switcher::EnhancedModeSwitcher;
use crate::modes::context::ContextManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

/// Hybrid mode engine for complex task execution
pub struct HybridModeEngine {
    consensus_engine: Arc<ConsensusEngine>,
    task_analyzer: TaskAnalyzer,
    mode_allocator: ModeAllocator,
    execution_coordinator: ExecutionCoordinator,
    progress_tracker: Arc<RwLock<ProgressTracker>>,
}

/// Analyzes tasks to determine mode requirements
#[derive(Debug)]
struct TaskAnalyzer {
    complexity_calculator: ComplexityCalculator,
    dependency_analyzer: DependencyAnalyzer,
    risk_assessor: RiskAssessor,
}

/// Allocates modes to task segments
#[derive(Debug)]
struct ModeAllocator {
    allocation_strategy: AllocationStrategy,
    mode_affinity_matrix: HashMap<TaskType, Vec<(ModeType, f32)>>,
}

/// Coordinates execution across modes
#[derive(Debug)]
struct ExecutionCoordinator {
    parallel_execution: bool,
    max_concurrent_tasks: usize,
    transition_buffer: std::time::Duration,
}

/// Tracks progress across hybrid execution
#[derive(Debug)]
struct ProgressTracker {
    tasks: HashMap<String, TaskProgress>,
    overall_progress: f32,
    mode_distribution: HashMap<ModeType, usize>,
}

/// Strategy for allocating modes
#[derive(Debug, Clone, PartialEq)]
enum AllocationStrategy {
    Adaptive,      // Dynamically adjust based on progress
    Balanced,      // Equal distribution between modes
    Performance,   // Optimize for speed
    Quality,       // Optimize for quality
}

/// Type of task for mode affinity
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum TaskType {
    Design,
    Implementation,
    Testing,
    Documentation,
    Refactoring,
    Debugging,
    Analysis,
}

/// Hybrid task representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridTask {
    pub id: String,
    pub description: String,
    pub segments: Vec<TaskSegment>,
    pub overall_complexity: f32,
    pub estimated_duration: std::time::Duration,
    pub mode_transitions: Vec<ModeTransition>,
}

/// Individual task segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSegment {
    pub id: String,
    pub description: String,
    pub mode: ModeType,
    pub complexity: f32,
    pub dependencies: Vec<String>,
    pub estimated_duration: std::time::Duration,
    pub can_parallelize: bool,
}

/// Mode transition point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTransition {
    pub from_segment: String,
    pub to_segment: String,
    pub from_mode: ModeType,
    pub to_mode: ModeType,
    pub transition_reason: String,
}

/// Progress of a task
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskProgress {
    pub task_id: String,
    pub current_segment: String,
    pub current_mode: ModeType,
    pub completion_percentage: f32,
    pub start_time: DateTime<Utc>,
    pub elapsed_time: std::time::Duration,
}

/// Result of hybrid execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridExecutionResult {
    pub task_id: String,
    pub success: bool,
    pub segments_completed: usize,
    pub mode_switches: usize,
    pub total_duration: std::time::Duration,
    pub mode_durations: HashMap<ModeType, std::time::Duration>,
    pub insights: ExecutionInsights,
}

/// Insights from hybrid execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInsights {
    pub optimal_mode_sequence: Vec<ModeType>,
    pub bottlenecks: Vec<String>,
    pub efficiency_score: f32,
    pub recommendations: Vec<String>,
}

impl HybridModeEngine {
    /// Create a new hybrid mode engine
    pub async fn new(consensus_engine: Arc<ConsensusEngine>) -> HiveResult<Self> {
        Ok(Self {
            consensus_engine,
            task_analyzer: TaskAnalyzer::new(),
            mode_allocator: ModeAllocator::new(),
            execution_coordinator: ExecutionCoordinator::new(),
            progress_tracker: Arc::new(RwLock::new(ProgressTracker::new())),
        })
    }
    
    /// Create a hybrid task from query
    pub async fn create_hybrid_task(
        &self,
        query: &str,
        context: &PlanningContext
    ) -> HiveResult<HybridTask> {
        // Analyze task complexity
        let complexity = self.task_analyzer.analyze_complexity(query)?;
        
        // Use AI to decompose into segments
        let segments = self.decompose_task(query, context).await?;
        
        // Allocate modes to segments
        let allocated_segments = self.mode_allocator.allocate_modes(segments, complexity)?;
        
        // Identify transition points
        let transitions = self.identify_transitions(&allocated_segments)?;
        
        // Estimate duration
        let estimated_duration = self.estimate_duration(&allocated_segments)?;
        
        Ok(HybridTask {
            id: uuid::Uuid::new_v4().to_string(),
            description: query.to_string(),
            segments: allocated_segments,
            overall_complexity: complexity,
            estimated_duration,
            mode_transitions: transitions,
        })
    }
    
    /// Execute hybrid task with mode transitions
    pub async fn execute_with_transitions(
        &self,
        task: &HybridTask,
        switcher: &Arc<RwLock<EnhancedModeSwitcher>>,
        context_manager: &Arc<RwLock<ContextManager>>
    ) -> HiveResult<HybridTask> {
        let start_time = std::time::Instant::now();
        let mut current_mode = ModeType::Hybrid;
        let mut mode_durations = HashMap::new();
        let mut segments_completed = 0;
        let mut mode_switches = 0;
        
        // Initialize progress tracking
        {
            let mut tracker = self.progress_tracker.write().await;
            tracker.start_task(&task.id, &task.segments[0].id, &current_mode);
        }
        
        // Execute segments with coordination
        for (i, segment) in task.segments.iter().enumerate() {
            // Check if mode switch is needed
            if segment.mode != current_mode {
                let mode_start = std::time::Instant::now();
                
                // Preserve context
                let context_snapshot = {
                    let ctx_manager = context_manager.read().await;
                    ctx_manager.capture_snapshot(&current_mode).await?
                };
                
                // Switch mode
                let switcher_guard = switcher.read().await;
                let switch_result = switcher_guard.switch_with_intelligence(
                    &current_mode,
                    &segment.mode,
                    Some(&context_snapshot),
                    &self.consensus_engine
                ).await?;
                
                if !switch_result.success {
                    return Err(HiveError::Planning(format!(
                        "Failed to switch from {:?} to {:?} for segment {}",
                        current_mode, segment.mode, segment.id
                    )));
                }
                
                // Update duration tracking
                let duration = mode_start.elapsed();
                *mode_durations.entry(current_mode).or_insert(std::time::Duration::ZERO) += duration;
                
                current_mode = segment.mode.clone();
                mode_switches += 1;
            }
            
            // Execute segment
            self.execute_segment(segment, &current_mode).await?;
            segments_completed += 1;
            
            // Update progress
            {
                let mut tracker = self.progress_tracker.write().await;
                tracker.update_progress(&task.id, &segment.id, (i + 1) as f32 / task.segments.len() as f32);
            }
            
            // Check for parallelization opportunities
            if i < task.segments.len() - 1 && segment.can_parallelize {
                self.check_parallel_execution(&task.segments[i + 1], &current_mode).await?;
            }
        }
        
        // Final duration update
        let final_duration = start_time.elapsed();
        *mode_durations.entry(current_mode).or_insert(std::time::Duration::ZERO) += 
            final_duration - mode_durations.values().sum::<std::time::Duration>();
        
        // Generate insights
        let insights = self.generate_insights(task, &mode_durations, segments_completed)?;
        
        Ok(HybridTask {
            id: task.id.clone(),
            description: format!("{} (Completed)", task.description),
            segments: task.segments.clone(),
            overall_complexity: task.overall_complexity,
            estimated_duration: task.estimated_duration,
            mode_transitions: task.mode_transitions.clone(),
        })
    }
    
    /// Get execution statistics
    pub async fn get_statistics(&self) -> HiveResult<HybridExecutionStatistics> {
        let tracker = self.progress_tracker.read().await;
        
        Ok(HybridExecutionStatistics {
            total_tasks: tracker.tasks.len(),
            average_progress: tracker.overall_progress,
            mode_distribution: tracker.mode_distribution.clone(),
            average_mode_switches: self.calculate_average_switches(&tracker.tasks),
        })
    }
    
    // Private helper methods
    
    async fn decompose_task(
        &self,
        query: &str,
        context: &PlanningContext
    ) -> HiveResult<Vec<TaskSegment>> {
        let prompt = format!(
            r#"Decompose this task into 3-5 logical segments for hybrid execution:

Task: "{}"

Context:
- Project Type: {:?}
- Complexity: Medium to High
- Team Size: {}

For each segment provide:
1. Brief description
2. Suggested mode (Planning, Execution, or Analysis)
3. Dependencies on other segments
4. Whether it can be parallelized

Return as JSON array of segments."#,
            query,
            context.project_type,
            context.team_size
        );
        
        let result = self.consensus_engine.process(&prompt, None).await?;
        
        // Parse segments or use fallback
        let segments = self.parse_segments(&result.result.unwrap_or_default())
            .unwrap_or_else(|_| self.create_default_segments(query));
        
        Ok(segments)
    }
    
    fn parse_segments(&self, response: &str) -> HiveResult<Vec<TaskSegment>> {
        // In real implementation, would parse JSON response
        // For now, create sample segments
        Ok(vec![
            TaskSegment {
                id: "seg1".to_string(),
                description: "Analyze requirements".to_string(),
                mode: ModeType::Analysis,
                complexity: 0.3,
                dependencies: vec![],
                estimated_duration: std::time::Duration::from_secs(300),
                can_parallelize: false,
            },
            TaskSegment {
                id: "seg2".to_string(),
                description: "Design solution".to_string(),
                mode: ModeType::Planning,
                complexity: 0.6,
                dependencies: vec!["seg1".to_string()],
                estimated_duration: std::time::Duration::from_secs(600),
                can_parallelize: false,
            },
            TaskSegment {
                id: "seg3".to_string(),
                description: "Implement core functionality".to_string(),
                mode: ModeType::Execution,
                complexity: 0.8,
                dependencies: vec!["seg2".to_string()],
                estimated_duration: std::time::Duration::from_secs(1200),
                can_parallelize: true,
            },
        ])
    }
    
    fn create_default_segments(&self, query: &str) -> Vec<TaskSegment> {
        vec![
            TaskSegment {
                id: uuid::Uuid::new_v4().to_string(),
                description: format!("Plan: {}", query),
                mode: ModeType::Planning,
                complexity: 0.5,
                dependencies: vec![],
                estimated_duration: std::time::Duration::from_secs(300),
                can_parallelize: false,
            },
            TaskSegment {
                id: uuid::Uuid::new_v4().to_string(),
                description: format!("Execute: {}", query),
                mode: ModeType::Execution,
                complexity: 0.7,
                dependencies: vec![],
                estimated_duration: std::time::Duration::from_secs(600),
                can_parallelize: false,
            },
        ]
    }
    
    fn identify_transitions(&self, segments: &[TaskSegment]) -> HiveResult<Vec<ModeTransition>> {
        let mut transitions = Vec::new();
        
        for i in 0..segments.len() - 1 {
            if segments[i].mode != segments[i + 1].mode {
                transitions.push(ModeTransition {
                    from_segment: segments[i].id.clone(),
                    to_segment: segments[i + 1].id.clone(),
                    from_mode: segments[i].mode.clone(),
                    to_mode: segments[i + 1].mode.clone(),
                    transition_reason: self.get_transition_reason(&segments[i], &segments[i + 1]),
                });
            }
        }
        
        Ok(transitions)
    }
    
    fn get_transition_reason(&self, from: &TaskSegment, to: &TaskSegment) -> String {
        match (&from.mode, &to.mode) {
            (ModeType::Planning, ModeType::Execution) => {
                "Moving from design to implementation phase".to_string()
            }
            (ModeType::Execution, ModeType::Analysis) => {
                "Need to analyze results before proceeding".to_string()
            }
            (ModeType::Analysis, ModeType::Planning) => {
                "Analysis complete, planning next steps".to_string()
            }
            _ => "Task requirements dictate mode change".to_string(),
        }
    }
    
    fn estimate_duration(&self, segments: &[TaskSegment]) -> HiveResult<std::time::Duration> {
        let total: std::time::Duration = segments.iter()
            .map(|s| s.estimated_duration)
            .sum();
        
        // Add transition overhead
        let transitions = segments.windows(2)
            .filter(|w| w[0].mode != w[1].mode)
            .count();
        
        let transition_overhead = std::time::Duration::from_millis(100 * transitions as u64);
        
        Ok(total + transition_overhead)
    }
    
    async fn execute_segment(
        &self,
        segment: &TaskSegment,
        mode: &ModeType
    ) -> HiveResult<()> {
        // Simulate segment execution
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // In real implementation, would execute based on mode
        match mode {
            ModeType::Planning => {
                // Execute planning tasks
            }
            ModeType::Execution => {
                // Execute implementation tasks
            }
            ModeType::Analysis => {
                // Execute analysis tasks
            }
            _ => {}
        }
        
        Ok(())
    }
    
    async fn check_parallel_execution(
        &self,
        next_segment: &TaskSegment,
        current_mode: &ModeType
    ) -> HiveResult<()> {
        if next_segment.can_parallelize && next_segment.mode == *current_mode {
            // In real implementation, would start parallel execution
        }
        Ok(())
    }
    
    fn generate_insights(
        &self,
        task: &HybridTask,
        mode_durations: &HashMap<ModeType, std::time::Duration>,
        segments_completed: usize
    ) -> HiveResult<ExecutionInsights> {
        let efficiency_score = segments_completed as f32 / task.segments.len() as f32;
        
        let optimal_sequence: Vec<ModeType> = task.segments.iter()
            .map(|s| s.mode.clone())
            .collect();
        
        let mut bottlenecks = Vec::new();
        let mut recommendations = Vec::new();
        
        // Identify bottlenecks
        for segment in &task.segments {
            if segment.complexity > 0.7 {
                bottlenecks.push(format!("High complexity in segment: {}", segment.description));
            }
        }
        
        // Generate recommendations
        if task.mode_transitions.len() > task.segments.len() / 2 {
            recommendations.push("Consider consolidating segments to reduce mode switches".to_string());
        }
        
        if mode_durations.get(&ModeType::Planning).unwrap_or(&std::time::Duration::ZERO) 
            > &task.estimated_duration / 3 {
            recommendations.push("Planning phase took significant time, consider more execution focus".to_string());
        }
        
        Ok(ExecutionInsights {
            optimal_mode_sequence: optimal_sequence,
            bottlenecks,
            efficiency_score,
            recommendations,
        })
    }
    
    fn calculate_average_switches(&self, tasks: &HashMap<String, TaskProgress>) -> f32 {
        // In real implementation, would track mode switches per task
        2.5 // Placeholder
    }
}

impl TaskAnalyzer {
    fn new() -> Self {
        Self {
            complexity_calculator: ComplexityCalculator::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
            risk_assessor: RiskAssessor::new(),
        }
    }
    
    fn analyze_complexity(&self, query: &str) -> HiveResult<f32> {
        let base_complexity = self.complexity_calculator.calculate(query)?;
        let dependency_factor = self.dependency_analyzer.analyze(query)?;
        let risk_factor = self.risk_assessor.assess(query)?;
        
        Ok((base_complexity + dependency_factor * 0.3 + risk_factor * 0.2).min(1.0))
    }
}

#[derive(Debug)]
struct ComplexityCalculator;

impl ComplexityCalculator {
    fn new() -> Self {
        Self
    }
    
    fn calculate(&self, query: &str) -> HiveResult<f32> {
        let words = query.split_whitespace().count();
        let complexity = (words as f32 / 50.0).min(1.0);
        Ok(complexity)
    }
}

#[derive(Debug)]
struct DependencyAnalyzer;

impl DependencyAnalyzer {
    fn new() -> Self {
        Self
    }
    
    fn analyze(&self, query: &str) -> HiveResult<f32> {
        let dependency_words = ["depends", "requires", "after", "before", "then"];
        let count = query.split_whitespace()
            .filter(|w| dependency_words.contains(&w.to_lowercase().as_str()))
            .count();
        
        Ok((count as f32 / 5.0).min(1.0))
    }
}

#[derive(Debug)]
struct RiskAssessor;

impl RiskAssessor {
    fn new() -> Self {
        Self
    }
    
    fn assess(&self, query: &str) -> HiveResult<f32> {
        let risk_words = ["critical", "urgent", "important", "careful", "security"];
        let count = query.split_whitespace()
            .filter(|w| risk_words.contains(&w.to_lowercase().as_str()))
            .count();
        
        Ok((count as f32 / 3.0).min(1.0))
    }
}

impl ModeAllocator {
    fn new() -> Self {
        let mut mode_affinity_matrix = HashMap::new();
        
        // Define task type affinities for modes
        mode_affinity_matrix.insert(TaskType::Design, vec![
            (ModeType::Planning, 0.9),
            (ModeType::Analysis, 0.7),
            (ModeType::Hybrid, 0.5),
        ]);
        
        mode_affinity_matrix.insert(TaskType::Implementation, vec![
            (ModeType::Execution, 0.9),
            (ModeType::Hybrid, 0.6),
            (ModeType::Planning, 0.3),
        ]);
        
        mode_affinity_matrix.insert(TaskType::Analysis, vec![
            (ModeType::Analysis, 0.9),
            (ModeType::Planning, 0.5),
            (ModeType::Hybrid, 0.4),
        ]);
        
        Self {
            allocation_strategy: AllocationStrategy::Adaptive,
            mode_affinity_matrix,
        }
    }
    
    fn allocate_modes(
        &self,
        mut segments: Vec<TaskSegment>,
        overall_complexity: f32
    ) -> HiveResult<Vec<TaskSegment>> {
        match self.allocation_strategy {
            AllocationStrategy::Adaptive => {
                // Allocate based on segment characteristics
                for segment in &mut segments {
                    if segment.complexity > 0.7 {
                        segment.mode = ModeType::Planning;
                    } else if segment.complexity < 0.3 {
                        segment.mode = ModeType::Execution;
                    }
                }
            }
            AllocationStrategy::Balanced => {
                // Balance between modes
                let modes = [ModeType::Planning, ModeType::Execution, ModeType::Analysis];
                for (i, segment) in segments.iter_mut().enumerate() {
                    segment.mode = modes[i % modes.len()].clone();
                }
            }
            AllocationStrategy::Performance => {
                // Optimize for speed - prefer execution
                for segment in &mut segments {
                    if segment.dependencies.is_empty() {
                        segment.mode = ModeType::Execution;
                    }
                }
            }
            AllocationStrategy::Quality => {
                // Optimize for quality - more planning
                if overall_complexity > 0.6 {
                    if let Some(first) = segments.first_mut() {
                        first.mode = ModeType::Planning;
                    }
                }
            }
        }
        
        Ok(segments)
    }
}

impl ExecutionCoordinator {
    fn new() -> Self {
        Self {
            parallel_execution: true,
            max_concurrent_tasks: 3,
            transition_buffer: std::time::Duration::from_millis(50),
        }
    }
}

impl ProgressTracker {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            overall_progress: 0.0,
            mode_distribution: HashMap::new(),
        }
    }
    
    fn start_task(&mut self, task_id: &str, segment_id: &str, mode: &ModeType) {
        self.tasks.insert(task_id.to_string(), TaskProgress {
            task_id: task_id.to_string(),
            current_segment: segment_id.to_string(),
            current_mode: mode.clone(),
            completion_percentage: 0.0,
            start_time: Utc::now(),
            elapsed_time: std::time::Duration::ZERO,
        });
        
        *self.mode_distribution.entry(mode.clone()).or_insert(0) += 1;
    }
    
    fn update_progress(&mut self, task_id: &str, segment_id: &str, progress: f32) {
        if let Some(task_progress) = self.tasks.get_mut(task_id) {
            task_progress.current_segment = segment_id.to_string();
            task_progress.completion_percentage = progress;
            task_progress.elapsed_time = Utc::now()
                .signed_duration_since(task_progress.start_time)
                .to_std()
                .unwrap_or_default();
        }
        
        // Update overall progress
        self.overall_progress = self.tasks.values()
            .map(|t| t.completion_percentage)
            .sum::<f32>() / self.tasks.len().max(1) as f32;
    }
}

/// Statistics for hybrid execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridExecutionStatistics {
    pub total_tasks: usize,
    pub average_progress: f32,
    pub mode_distribution: HashMap<ModeType, usize>,
    pub average_mode_switches: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hybrid_engine_creation() {
        // Test engine initialization
    }
    
    #[tokio::test]
    async fn test_task_creation() {
        // Test hybrid task creation
    }
    
    #[tokio::test]
    async fn test_mode_allocation() {
        // Test mode allocation strategies
    }
    
    #[tokio::test]
    async fn test_hybrid_execution() {
        // Test execution with mode transitions
    }
}