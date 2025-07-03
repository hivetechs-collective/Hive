//! Task Decomposition using AI
//! 
//! Intelligently breaks down complex tasks into manageable subtasks

use crate::core::error::{HiveResult, HiveError};
use crate::consensus::engine::ConsensusEngine;
use crate::planning::types::*;
use serde_json::json;
use uuid::Uuid;
use anyhow::anyhow;

/// AI-powered task decomposition engine
pub struct TaskDecomposer {
    decomposition_templates: Vec<DecompositionTemplate>,
}

/// Template for task decomposition patterns
#[derive(Debug, Clone)]
struct DecompositionTemplate {
    task_type: TaskType,
    pattern: Vec<&'static str>,
    required_subtasks: Vec<&'static str>,
}

impl TaskDecomposer {
    pub fn new() -> Self {
        Self {
            decomposition_templates: Self::init_templates(),
        }
    }

    /// Decompose a high-level task description into subtasks using AI
    pub async fn decompose(
        &self,
        description: &str,
        context: &PlanningContext,
        consensus_engine: &ConsensusEngine,
    ) -> HiveResult<Vec<Task>> {
        // First, use AI to understand the task and determine its type
        let task_analysis = self.analyze_task(description, context, consensus_engine).await?;
        
        // Generate initial task breakdown using AI
        let ai_tasks = self.generate_ai_tasks(description, &task_analysis, context, consensus_engine).await?;
        
        // Apply domain-specific patterns and validations
        let refined_tasks = self.refine_tasks(ai_tasks, &task_analysis, context)?;
        
        // Ensure all critical subtasks are included
        let complete_tasks = self.ensure_completeness(refined_tasks, &task_analysis)?;
        
        Ok(complete_tasks)
    }

    /// Analyze the task to determine its nature and requirements
    async fn analyze_task(
        &self,
        description: &str,
        context: &PlanningContext,
        consensus_engine: &ConsensusEngine,
    ) -> HiveResult<TaskAnalysis> {
        let prompt = format!(
            r#"Analyze this development task and provide a structured analysis:

Task: {}

Context:
- Project Type: {:?}
- Team Size: {}
- Experience Level: {:?}
- Technology Stack: {}
- Existing Codebase: {}

Please analyze and return a JSON object with:
{{
    "task_type": "one of: Implementation, Testing, Documentation, Review, Deployment, Research, Design, Refactoring, BugFix, Configuration",
    "complexity": "one of: Simple, Moderate, Complex, VeryComplex",
    "estimated_subtasks": <number>,
    "required_skills": ["skill1", "skill2", ...],
    "key_components": ["component1", "component2", ...],
    "potential_challenges": ["challenge1", "challenge2", ...],
    "suggested_approach": "brief description of recommended approach"
}}"#,
            description,
            context.project_type,
            context.team_size,
            context.experience_level,
            context.technology_stack.join(", "),
            context.existing_codebase
        );

        let result = consensus_engine.process(&prompt, None).await?;
        let analysis: TaskAnalysis = serde_json::from_str(&result.result.ok_or_else(|| anyhow!("No analysis result"))?)
            .map_err(|e| HiveError::Planning(format!("Failed to parse task analysis: {}", e)))?;
        
        Ok(analysis)
    }

    /// Generate tasks using AI consensus
    async fn generate_ai_tasks(
        &self,
        description: &str,
        analysis: &TaskAnalysis,
        context: &PlanningContext,
        consensus_engine: &ConsensusEngine,
    ) -> HiveResult<Vec<Task>> {
        let prompt = format!(
            r#"Break down this task into specific, actionable subtasks:

Task: {}
Type: {:?}
Complexity: {:?}
Key Components: {}

Context:
- Detail Level: {:?}
- Team Experience: {:?}
- Time Constraints: {}

Generate a JSON array of subtasks with this structure:
[
    {{
        "title": "Clear, action-oriented title",
        "description": "Detailed description of what needs to be done",
        "task_type": "appropriate type",
        "priority": "Critical/High/Medium/Low",
        "estimated_hours": <number>,
        "dependencies": ["titles of tasks this depends on"],
        "required_skills": ["skill1", "skill2"],
        "acceptance_criteria": ["criterion1", "criterion2"]
    }}
]

Guidelines:
- Each task should be completable in 1-8 hours
- Include all necessary steps for {}
- Consider the team's {:?} experience level
- Ensure logical dependency ordering
- Be specific and actionable"#,
            description,
            analysis.task_type,
            analysis.complexity,
            analysis.key_components.join(", "),
            context.user_preferences.detail_level,
            context.experience_level,
            context.time_constraints.map(|d| format!("{} days", d.num_days())).unwrap_or("No deadline".to_string()),
            analysis.task_type,
            context.experience_level
        );

        let result = consensus_engine.process(&prompt, None).await?;
        let raw_tasks: Vec<RawTask> = serde_json::from_str(&result.result.ok_or_else(|| anyhow!("No decomposition result"))?)
            .map_err(|e| HiveError::Planning(format!("Failed to parse tasks: {}", e)))?;
        
        // Convert raw tasks to proper Task objects
        let tasks = raw_tasks.into_iter().enumerate().map(|(index, raw)| {
            Task {
                id: Uuid::new_v4().to_string(),
                title: raw.title,
                description: raw.description,
                task_type: self.parse_task_type(&raw.task_type),
                priority: self.parse_priority(&raw.priority),
                estimated_duration: chrono::Duration::hours(raw.estimated_hours as i64),
                dependencies: Vec::new(), // Will be resolved later
                required_skills: raw.required_skills,
                resources: self.determine_resources(&raw),
                acceptance_criteria: raw.acceptance_criteria,
                subtasks: Vec::new(),
            }
        }).collect();
        
        Ok(tasks)
    }

    /// Refine AI-generated tasks with domain knowledge
    fn refine_tasks(
        &self,
        mut tasks: Vec<Task>,
        analysis: &TaskAnalysis,
        context: &PlanningContext,
    ) -> HiveResult<Vec<Task>> {
        // Apply templates based on task type
        if let Some(template) = self.decomposition_templates.iter()
            .find(|t| t.task_type == analysis.task_type) 
        {
            // Ensure required subtasks are present
            for required in &template.required_subtasks {
                if !tasks.iter().any(|t| t.title.to_lowercase().contains(&required.to_lowercase())) {
                    // Add missing required task
                    tasks.push(self.create_required_task(required, &analysis.task_type)?);
                }
            }
        }
        
        // Adjust priorities based on dependencies
        self.adjust_priorities(&mut tasks)?;
        
        // Optimize task ordering
        self.optimize_ordering(&mut tasks)?;
        
        Ok(tasks)
    }

    /// Ensure all critical aspects are covered
    fn ensure_completeness(
        &self,
        mut tasks: Vec<Task>,
        analysis: &TaskAnalysis,
    ) -> HiveResult<Vec<Task>> {
        // Always include testing if not present
        if !tasks.iter().any(|t| t.task_type == TaskType::Testing) && analysis.task_type == TaskType::Implementation {
            tasks.push(self.create_testing_task()?);
        }
        
        // Always include documentation for complex tasks
        if !tasks.iter().any(|t| t.task_type == TaskType::Documentation) && 
           (analysis.complexity == Complexity::Complex || analysis.complexity == Complexity::VeryComplex) {
            tasks.push(self.create_documentation_task()?);
        }
        
        // Add review task for critical components
        if !tasks.iter().any(|t| t.task_type == TaskType::Review) && tasks.len() > 3 {
            tasks.push(self.create_review_task()?);
        }
        
        Ok(tasks)
    }

    // Helper methods

    fn init_templates() -> Vec<DecompositionTemplate> {
        vec![
            DecompositionTemplate {
                task_type: TaskType::Implementation,
                pattern: vec!["design", "implement", "test", "document"],
                required_subtasks: vec!["implement", "test"],
            },
            DecompositionTemplate {
                task_type: TaskType::BugFix,
                pattern: vec!["reproduce", "diagnose", "fix", "test", "verify"],
                required_subtasks: vec!["diagnose", "fix", "test"],
            },
            DecompositionTemplate {
                task_type: TaskType::Refactoring,
                pattern: vec!["analyze", "plan", "refactor", "test", "verify"],
                required_subtasks: vec!["analyze", "refactor", "test"],
            },
            DecompositionTemplate {
                task_type: TaskType::Deployment,
                pattern: vec!["prepare", "validate", "deploy", "verify", "rollback-plan"],
                required_subtasks: vec!["validate", "deploy", "verify"],
            },
        ]
    }

    fn parse_task_type(&self, type_str: &str) -> TaskType {
        match type_str.to_lowercase().as_str() {
            "implementation" => TaskType::Implementation,
            "testing" => TaskType::Testing,
            "documentation" => TaskType::Documentation,
            "review" => TaskType::Review,
            "deployment" => TaskType::Deployment,
            "research" => TaskType::Research,
            "design" => TaskType::Design,
            "refactoring" => TaskType::Refactoring,
            "bugfix" | "bug fix" => TaskType::BugFix,
            "configuration" => TaskType::Configuration,
            _ => TaskType::Implementation,
        }
    }

    fn parse_priority(&self, priority_str: &str) -> Priority {
        match priority_str.to_lowercase().as_str() {
            "critical" => Priority::Critical,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::Medium,
        }
    }

    fn determine_resources(&self, raw_task: &RawTask) -> Vec<Resource> {
        let mut resources = Vec::new();
        
        // Add human resources based on skills
        for skill in &raw_task.required_skills {
            resources.push(Resource {
                resource_type: ResourceType::Human,
                name: format!("Developer with {} skills", skill),
                quantity: 1.0,
                availability: ResourceAvailability::Available,
            });
        }
        
        // Add tool resources if mentioned
        if raw_task.description.to_lowercase().contains("database") {
            resources.push(Resource {
                resource_type: ResourceType::Infrastructure,
                name: "Database Access".to_string(),
                quantity: 1.0,
                availability: ResourceAvailability::Available,
            });
        }
        
        resources
    }

    fn create_required_task(&self, task_name: &str, task_type: &TaskType) -> HiveResult<Task> {
        Ok(Task {
            id: Uuid::new_v4().to_string(),
            title: task_name.to_string(),
            description: format!("Required {} task", task_name),
            task_type: task_type.clone(),
            priority: Priority::High,
            estimated_duration: chrono::Duration::hours(2),
            dependencies: Vec::new(),
            required_skills: Vec::new(),
            resources: Vec::new(),
            acceptance_criteria: vec![format!("{} completed successfully", task_name)],
            subtasks: Vec::new(),
        })
    }

    fn create_testing_task(&self) -> HiveResult<Task> {
        Ok(Task {
            id: Uuid::new_v4().to_string(),
            title: "Write and run tests".to_string(),
            description: "Create comprehensive tests for the implemented functionality".to_string(),
            task_type: TaskType::Testing,
            priority: Priority::High,
            estimated_duration: chrono::Duration::hours(3),
            dependencies: Vec::new(),
            required_skills: vec!["testing".to_string()],
            resources: vec![Resource {
                resource_type: ResourceType::Tool,
                name: "Testing Framework".to_string(),
                quantity: 1.0,
                availability: ResourceAvailability::Available,
            }],
            acceptance_criteria: vec![
                "All tests pass".to_string(),
                "Code coverage > 80%".to_string(),
            ],
            subtasks: Vec::new(),
        })
    }

    fn create_documentation_task(&self) -> HiveResult<Task> {
        Ok(Task {
            id: Uuid::new_v4().to_string(),
            title: "Document implementation".to_string(),
            description: "Create comprehensive documentation for the implementation".to_string(),
            task_type: TaskType::Documentation,
            priority: Priority::Medium,
            estimated_duration: chrono::Duration::hours(2),
            dependencies: Vec::new(),
            required_skills: vec!["documentation".to_string()],
            resources: Vec::new(),
            acceptance_criteria: vec![
                "API documentation complete".to_string(),
                "Usage examples provided".to_string(),
            ],
            subtasks: Vec::new(),
        })
    }

    fn create_review_task(&self) -> HiveResult<Task> {
        Ok(Task {
            id: Uuid::new_v4().to_string(),
            title: "Code review".to_string(),
            description: "Review implementation for quality and standards compliance".to_string(),
            task_type: TaskType::Review,
            priority: Priority::High,
            estimated_duration: chrono::Duration::hours(1),
            dependencies: Vec::new(),
            required_skills: vec!["code review".to_string()],
            resources: Vec::new(),
            acceptance_criteria: vec![
                "Code meets quality standards".to_string(),
                "All review comments addressed".to_string(),
            ],
            subtasks: Vec::new(),
        })
    }

    fn adjust_priorities(&self, tasks: &mut Vec<Task>) -> HiveResult<()> {
        // Tasks with no dependencies should have higher priority
        for task in tasks.iter_mut() {
            if task.dependencies.is_empty() && task.priority == Priority::Medium {
                task.priority = Priority::High;
            }
        }
        Ok(())
    }

    fn optimize_ordering(&self, tasks: &mut Vec<Task>) -> HiveResult<()> {
        // Sort by priority first, then by estimated duration (shorter tasks first)
        tasks.sort_by(|a, b| {
            match a.priority.cmp(&b.priority) {
                std::cmp::Ordering::Equal => a.estimated_duration.cmp(&b.estimated_duration),
                other => other,
            }
        });
        Ok(())
    }
}

/// Analysis of a task from AI
#[derive(Debug, serde::Deserialize)]
struct TaskAnalysis {
    task_type: TaskType,
    complexity: Complexity,
    estimated_subtasks: usize,
    required_skills: Vec<String>,
    key_components: Vec<String>,
    potential_challenges: Vec<String>,
    suggested_approach: String,
}

/// Complexity levels
#[derive(Debug, serde::Deserialize, PartialEq)]
enum Complexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Raw task from AI response
#[derive(Debug, serde::Deserialize)]
struct RawTask {
    title: String,
    description: String,
    task_type: String,
    priority: String,
    estimated_hours: f64,
    dependencies: Vec<String>,
    required_skills: Vec<String>,
    acceptance_criteria: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_decomposer_creation() {
        let decomposer = TaskDecomposer::new();
        assert!(!decomposer.decomposition_templates.is_empty());
    }

    #[test]
    fn test_parse_task_type() {
        let decomposer = TaskDecomposer::new();
        assert_eq!(decomposer.parse_task_type("Implementation"), TaskType::Implementation);
        assert_eq!(decomposer.parse_task_type("bug fix"), TaskType::BugFix);
    }

    #[test]
    fn test_parse_priority() {
        let decomposer = TaskDecomposer::new();
        assert_eq!(decomposer.parse_priority("Critical"), Priority::Critical);
        assert_eq!(decomposer.parse_priority("unknown"), Priority::Medium);
    }
}