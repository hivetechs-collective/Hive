//! Type definitions for the planning engine

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::fmt;

/// A comprehensive development plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub title: String,
    pub description: String,
    pub mode: ModeType,
    pub tasks: Vec<Task>,
    pub risks: Vec<Risk>,
    pub dependencies: DependencyGraph,
    pub timeline: Timeline,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub context: PlanningContext,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Individual task within a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub priority: Priority,
    pub estimated_duration: Duration,
    pub dependencies: Vec<String>, // Task IDs this depends on
    pub required_skills: Vec<String>,
    pub resources: Vec<Resource>,
    pub acceptance_criteria: Vec<String>,
    pub subtasks: Vec<Task>, // For nested task structures
}

/// Type of task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    Implementation,
    Testing,
    Documentation,
    Review,
    Deployment,
    Research,
    Design,
    Refactoring,
    BugFix,
    Configuration,
}

/// Task priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Risk associated with the plan or tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: RiskSeverity,
    pub probability: f32, // 0.0 to 1.0
    pub impact: RiskImpact,
    pub mitigation_strategies: Vec<MitigationStrategy>,
    pub affected_tasks: Vec<String>, // Task IDs
}

/// Risk severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Risk impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskImpact {
    pub timeline_impact: Duration,
    pub cost_impact: f64,
    pub quality_impact: QualityImpact,
    pub scope_impact: ScopeImpact,
}

/// Quality impact levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityImpact {
    Severe,
    Significant,
    Moderate,
    Minor,
    Negligible,
}

/// Scope impact levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScopeImpact {
    Major,
    Moderate,
    Minor,
    None,
}

/// Risk mitigation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationStrategy {
    pub id: String,
    pub description: String,
    pub effectiveness: f32, // 0.0 to 1.0
    pub cost: f64,
    pub implementation_time: Duration,
}

/// Dependency between tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub from_task: String,
    pub to_task: String,
    pub dependency_type: DependencyType,
    pub lag_time: Option<Duration>,
}

/// Type of dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    FinishToStart,    // Task B can't start until Task A finishes
    StartToStart,     // Task B can't start until Task A starts
    FinishToFinish,   // Task B can't finish until Task A finishes
    StartToFinish,    // Task B can't finish until Task A starts
}

/// Dependency graph for task ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, Task>,
    pub edges: Vec<Dependency>,
    pub critical_path: Vec<String>, // Task IDs in critical path
}

/// Timeline for the entire plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub total_duration: Duration,
    pub milestones: Vec<Milestone>,
    pub task_schedules: HashMap<String, TaskSchedule>,
}

/// Project milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub date: DateTime<Utc>,
    pub tasks: Vec<String>, // Task IDs that must be completed
    pub deliverables: Vec<String>,
}

/// Schedule for individual task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSchedule {
    pub task_id: String,
    pub planned_start: DateTime<Utc>,
    pub planned_end: DateTime<Utc>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub progress: f32, // 0.0 to 1.0
}

/// Resource required for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub name: String,
    pub quantity: f64,
    pub availability: ResourceAvailability,
}

/// Type of resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Human,
    Tool,
    Service,
    Infrastructure,
    Data,
    Documentation,
}

/// Resource availability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceAvailability {
    Available,
    Limited,
    RequiresApproval,
    NotAvailable,
}

/// Operating mode for the planning engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModeType {
    Planning,      // Focus on creating comprehensive plans
    Execution,     // Focus on implementing and coding
    Hybrid,        // Balance between planning and execution
    Analysis,      // Focus on understanding existing code
    Learning,      // Adaptive mode that learns preferences
}

/// Context for planning operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningContext {
    pub project_type: ProjectType,
    pub team_size: usize,
    pub experience_level: ExperienceLevel,
    pub time_constraints: Option<Duration>,
    pub budget_constraints: Option<f64>,
    pub technology_stack: Vec<String>,
    pub existing_codebase: bool,
    pub repository_path: Option<String>,
    pub user_preferences: UserPreferences,
}

/// Type of project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectType {
    WebApplication,
    MobileApp,
    DesktopApplication,
    Library,
    API,
    Microservice,
    DataPipeline,
    MachineLearning,
    Infrastructure,
    Other(String),
}

/// Team experience level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// User preferences for planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_mode: ModeType,
    pub detail_level: DetailLevel,
    pub risk_tolerance: RiskTolerance,
    pub automation_level: AutomationLevel,
    pub collaboration_style: CollaborationStyle,
}

/// Level of detail in plans
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DetailLevel {
    High,      // Very detailed, step-by-step
    Medium,    // Balanced detail
    Low,       // High-level overview
}

/// Risk tolerance levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskTolerance {
    Conservative,  // Minimize all risks
    Balanced,      // Balance risk vs reward
    Aggressive,    // Accept higher risks for faster delivery
}

/// Automation preference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutomationLevel {
    Full,      // Automate everything possible
    Guided,    // Automate with user confirmation
    Manual,    // User controls all decisions
}

/// Collaboration style
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollaborationStyle {
    Solo,         // Working alone
    PairProgramming,
    SmallTeam,    // 2-5 people
    LargeTeam,    // 6+ people
    Distributed,  // Remote team
}

impl Default for PlanningContext {
    fn default() -> Self {
        Self {
            project_type: ProjectType::Other("Unknown".to_string()),
            team_size: 1,
            experience_level: ExperienceLevel::Intermediate,
            time_constraints: None,
            budget_constraints: None,
            technology_stack: Vec::new(),
            existing_codebase: false,
            repository_path: None,
            user_preferences: UserPreferences::default(),
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_mode: ModeType::Hybrid,
            detail_level: DetailLevel::Medium,
            risk_tolerance: RiskTolerance::Balanced,
            automation_level: AutomationLevel::Guided,
            collaboration_style: CollaborationStyle::Solo,
        }
    }
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskType::Implementation => write!(f, "Implementation"),
            TaskType::Testing => write!(f, "Testing"),
            TaskType::Documentation => write!(f, "Documentation"),
            TaskType::Review => write!(f, "Review"),
            TaskType::Deployment => write!(f, "Deployment"),
            TaskType::Research => write!(f, "Research"),
            TaskType::Design => write!(f, "Design"),
            TaskType::Refactoring => write!(f, "Refactoring"),
            TaskType::BugFix => write!(f, "Bug Fix"),
            TaskType::Configuration => write!(f, "Configuration"),
        }
    }
}