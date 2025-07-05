//! Risk Analysis and Mitigation
//! 
//! Identifies potential risks and provides mitigation strategies

use crate::core::error::{HiveResult, HiveError};
use crate::planning::types::*;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Duration;

/// Risk analysis engine
pub struct RiskAnalyzer {
    risk_patterns: Vec<RiskPattern>,
    mitigation_database: HashMap<RiskCategory, Vec<MitigationTemplate>>,
}

/// Pattern for identifying risks
#[derive(Debug, Clone)]
struct RiskPattern {
    category: RiskCategory,
    indicators: Vec<String>,
    base_severity: RiskSeverity,
    base_probability: f32,
}

/// Risk categories
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum RiskCategory {
    Technical,
    Timeline,
    Resource,
    Quality,
    Scope,
    Integration,
    Security,
    Performance,
}

/// Template for mitigation strategies
#[derive(Debug, Clone)]
struct MitigationTemplate {
    description: String,
    effectiveness: f32,
    cost_factor: f32,
    time_factor: f32,
}

impl RiskAnalyzer {
    pub fn new() -> Self {
        Self {
            risk_patterns: Self::init_risk_patterns(),
            mitigation_database: Self::init_mitigation_database(),
        }
    }

    /// Analyze tasks and context to identify risks
    pub fn analyze(&self, tasks: &[Task], context: &PlanningContext) -> HiveResult<Vec<Risk>> {
        let mut risks = Vec::new();
        
        // Analyze task-specific risks
        for task in tasks {
            risks.extend(self.analyze_task_risks(task, context)?);
        }
        
        // Analyze overall project risks
        risks.extend(self.analyze_project_risks(tasks, context)?);
        
        // Analyze dependency risks
        risks.extend(self.analyze_dependency_risks(tasks)?);
        
        // Analyze resource risks
        risks.extend(self.analyze_resource_risks(tasks, context)?);
        
        // Score and prioritize risks
        self.score_risks(&mut risks);
        
        // Generate mitigation strategies
        for risk in &mut risks {
            risk.mitigation_strategies = self.generate_mitigations(risk)?;
        }
        
        // Sort by severity and probability
        risks.sort_by(|a, b| {
            let a_score = self.calculate_risk_score(a);
            let b_score = self.calculate_risk_score(b);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(risks)
    }

    /// Analyze risks specific to individual tasks
    fn analyze_task_risks(&self, task: &Task, context: &PlanningContext) -> HiveResult<Vec<Risk>> {
        let mut risks = Vec::new();
        
        // Check for complexity risks
        if task.estimated_duration > Duration::hours(8) {
            risks.push(self.create_complexity_risk(task)?);
        }
        
        // Check for skill gaps
        if !task.required_skills.is_empty() {
            if let Some(risk) = self.check_skill_gap_risk(task, context)? {
                risks.push(risk);
            }
        }
        
        // Check for resource availability
        for resource in &task.resources {
            if resource.availability != ResourceAvailability::Available {
                risks.push(self.create_resource_risk(task, resource)?);
            }
        }
        
        // Check for testing risks
        if task.task_type == TaskType::Implementation && task.acceptance_criteria.is_empty() {
            risks.push(self.create_testing_risk(task)?);
        }
        
        Ok(risks)
    }

    /// Analyze overall project risks
    fn analyze_project_risks(&self, tasks: &[Task], context: &PlanningContext) -> HiveResult<Vec<Risk>> {
        let mut risks = Vec::new();
        
        // Timeline risk
        let total_duration: Duration = tasks.iter()
            .map(|t| t.estimated_duration)
            .sum();
        
        if let Some(constraint) = context.time_constraints {
            if total_duration > constraint {
                risks.push(self.create_timeline_risk(total_duration, constraint)?);
            }
        }
        
        // Team size risk
        if context.team_size == 1 && tasks.len() > 10 {
            risks.push(self.create_workload_risk(tasks.len())?);
        }
        
        // Technology risk
        if context.technology_stack.len() > 5 {
            risks.push(self.create_technology_complexity_risk(&context.technology_stack)?);
        }
        
        // Experience level risk
        if context.experience_level == ExperienceLevel::Beginner && 
           tasks.iter().any(|t| t.task_type == TaskType::Design || t.task_type == TaskType::Refactoring) {
            risks.push(self.create_experience_risk()?);
        }
        
        Ok(risks)
    }

    /// Analyze dependency-related risks
    fn analyze_dependency_risks(&self, tasks: &[Task]) -> HiveResult<Vec<Risk>> {
        let mut risks = Vec::new();
        
        // Check for circular dependencies
        if self.has_circular_dependencies(tasks) {
            risks.push(Risk {
                id: Uuid::new_v4().to_string(),
                title: "Circular Dependencies Detected".to_string(),
                description: "Tasks have circular dependencies that will prevent execution".to_string(),
                severity: RiskSeverity::Critical,
                probability: 1.0,
                impact: RiskImpact {
                    timeline_impact: Duration::days(999),
                    cost_impact: 0.0,
                    quality_impact: QualityImpact::Severe,
                    scope_impact: ScopeImpact::Major,
                },
                mitigation_strategies: Vec::new(),
                affected_tasks: tasks.iter().map(|t| t.id.clone()).collect(),
            });
        }
        
        // Check for long dependency chains
        let max_chain_length = self.find_longest_dependency_chain(tasks);
        if max_chain_length > 5 {
            risks.push(self.create_dependency_chain_risk(max_chain_length)?);
        }
        
        Ok(risks)
    }

    /// Analyze resource-related risks
    fn analyze_resource_risks(&self, tasks: &[Task], context: &PlanningContext) -> HiveResult<Vec<Risk>> {
        let mut risks = Vec::new();
        
        // Count required resources
        let mut resource_counts: HashMap<String, f64> = HashMap::new();
        for task in tasks {
            for resource in &task.resources {
                *resource_counts.entry(resource.name.clone()).or_insert(0.0) += resource.quantity;
            }
        }
        
        // Check for resource conflicts
        for (resource_name, total_quantity) in resource_counts {
            if total_quantity > 1.0 && resource_name.contains("Developer") {
                risks.push(self.create_resource_conflict_risk(&resource_name, total_quantity)?);
            }
        }
        
        Ok(risks)
    }

    /// Generate mitigation strategies for a risk
    fn generate_mitigations(&self, risk: &Risk) -> HiveResult<Vec<MitigationStrategy>> {
        let category = self.determine_risk_category(risk);
        
        let mut strategies = Vec::new();
        
        if let Some(templates) = self.mitigation_database.get(&category) {
            for template in templates {
                strategies.push(MitigationStrategy {
                    id: Uuid::new_v4().to_string(),
                    description: template.description.clone(),
                    effectiveness: template.effectiveness * self.adjust_effectiveness(risk),
                    cost: self.calculate_mitigation_cost(risk, template),
                    implementation_time: self.calculate_mitigation_time(risk, template),
                });
            }
        }
        
        // Add risk-specific mitigations
        strategies.extend(self.generate_specific_mitigations(risk)?);
        
        // Sort by effectiveness
        strategies.sort_by(|a, b| b.effectiveness.partial_cmp(&a.effectiveness).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(strategies)
    }

    // Helper methods

    fn init_risk_patterns() -> Vec<RiskPattern> {
        vec![
            RiskPattern {
                category: RiskCategory::Technical,
                indicators: vec!["complex".to_string(), "new technology".to_string(), "integration".to_string()],
                base_severity: RiskSeverity::High,
                base_probability: 0.6,
            },
            RiskPattern {
                category: RiskCategory::Timeline,
                indicators: vec!["deadline".to_string(), "tight schedule".to_string(), "dependencies".to_string()],
                base_severity: RiskSeverity::High,
                base_probability: 0.7,
            },
            RiskPattern {
                category: RiskCategory::Resource,
                indicators: vec!["limited resources".to_string(), "skill gap".to_string(), "availability".to_string()],
                base_severity: RiskSeverity::Medium,
                base_probability: 0.5,
            },
            RiskPattern {
                category: RiskCategory::Quality,
                indicators: vec!["no tests".to_string(), "unclear requirements".to_string(), "rushed".to_string()],
                base_severity: RiskSeverity::High,
                base_probability: 0.8,
            },
        ]
    }

    fn init_mitigation_database() -> HashMap<RiskCategory, Vec<MitigationTemplate>> {
        let mut database = HashMap::new();
        
        database.insert(RiskCategory::Technical, vec![
            MitigationTemplate {
                description: "Conduct proof of concept for new technologies".to_string(),
                effectiveness: 0.8,
                cost_factor: 0.2,
                time_factor: 0.3,
            },
            MitigationTemplate {
                description: "Allocate time for learning and experimentation".to_string(),
                effectiveness: 0.7,
                cost_factor: 0.1,
                time_factor: 0.4,
            },
            MitigationTemplate {
                description: "Seek expert consultation or mentoring".to_string(),
                effectiveness: 0.9,
                cost_factor: 0.5,
                time_factor: 0.2,
            },
        ]);
        
        database.insert(RiskCategory::Timeline, vec![
            MitigationTemplate {
                description: "Prioritize critical path tasks".to_string(),
                effectiveness: 0.7,
                cost_factor: 0.0,
                time_factor: 0.0,
            },
            MitigationTemplate {
                description: "Add buffer time to estimates".to_string(),
                effectiveness: 0.6,
                cost_factor: 0.0,
                time_factor: 0.2,
            },
            MitigationTemplate {
                description: "Consider parallel execution where possible".to_string(),
                effectiveness: 0.8,
                cost_factor: 0.3,
                time_factor: -0.3,
            },
        ]);
        
        database.insert(RiskCategory::Resource, vec![
            MitigationTemplate {
                description: "Cross-train team members".to_string(),
                effectiveness: 0.7,
                cost_factor: 0.2,
                time_factor: 0.3,
            },
            MitigationTemplate {
                description: "Hire contractors or consultants".to_string(),
                effectiveness: 0.9,
                cost_factor: 0.8,
                time_factor: 0.1,
            },
            MitigationTemplate {
                description: "Simplify requirements to match available resources".to_string(),
                effectiveness: 0.6,
                cost_factor: -0.2,
                time_factor: -0.2,
            },
        ]);
        
        database.insert(RiskCategory::Quality, vec![
            MitigationTemplate {
                description: "Implement comprehensive testing strategy".to_string(),
                effectiveness: 0.9,
                cost_factor: 0.3,
                time_factor: 0.4,
            },
            MitigationTemplate {
                description: "Add code review checkpoints".to_string(),
                effectiveness: 0.8,
                cost_factor: 0.1,
                time_factor: 0.2,
            },
            MitigationTemplate {
                description: "Define clear acceptance criteria upfront".to_string(),
                effectiveness: 0.7,
                cost_factor: 0.0,
                time_factor: 0.1,
            },
        ]);
        
        database
    }

    fn create_complexity_risk(&self, task: &Task) -> HiveResult<Risk> {
        Ok(Risk {
            id: Uuid::new_v4().to_string(),
            title: format!("High Complexity: {}", task.title),
            description: format!("Task '{}' has high complexity with {} hour estimate", task.title, task.estimated_duration.num_hours()),
            severity: RiskSeverity::Medium,
            probability: 0.6,
            impact: RiskImpact {
                timeline_impact: Duration::hours(task.estimated_duration.num_hours() / 2),
                cost_impact: 0.0,
                quality_impact: QualityImpact::Moderate,
                scope_impact: ScopeImpact::Minor,
            },
            mitigation_strategies: Vec::new(),
            affected_tasks: vec![task.id.clone()],
        })
    }

    fn check_skill_gap_risk(&self, task: &Task, context: &PlanningContext) -> HiveResult<Option<Risk>> {
        // Simple skill gap detection based on experience level
        if context.experience_level == ExperienceLevel::Beginner && 
           task.required_skills.iter().any(|s| s.contains("advanced") || s.contains("expert")) {
            return Ok(Some(Risk {
                id: Uuid::new_v4().to_string(),
                title: format!("Skill Gap: {}", task.title),
                description: format!("Task requires skills that may exceed team's experience level: {}", task.required_skills.join(", ")),
                severity: RiskSeverity::High,
                probability: 0.7,
                impact: RiskImpact {
                    timeline_impact: Duration::hours(task.estimated_duration.num_hours()),
                    cost_impact: 0.0,
                    quality_impact: QualityImpact::Significant,
                    scope_impact: ScopeImpact::Moderate,
                },
                mitigation_strategies: Vec::new(),
                affected_tasks: vec![task.id.clone()],
            }));
        }
        Ok(None)
    }

    fn create_resource_risk(&self, task: &Task, resource: &Resource) -> HiveResult<Risk> {
        Ok(Risk {
            id: Uuid::new_v4().to_string(),
            title: format!("Resource Availability: {}", resource.name),
            description: format!("Resource '{}' has limited availability for task '{}'", resource.name, task.title),
            severity: RiskSeverity::Medium,
            probability: 0.5,
            impact: RiskImpact {
                timeline_impact: Duration::hours(4),
                cost_impact: 0.0,
                quality_impact: QualityImpact::Minor,
                scope_impact: ScopeImpact::Minor,
            },
            mitigation_strategies: Vec::new(),
            affected_tasks: vec![task.id.clone()],
        })
    }

    fn create_testing_risk(&self, task: &Task) -> HiveResult<Risk> {
        Ok(Risk {
            id: Uuid::new_v4().to_string(),
            title: format!("Missing Test Criteria: {}", task.title),
            description: "Implementation task lacks clear acceptance criteria for testing".to_string(),
            severity: RiskSeverity::High,
            probability: 0.8,
            impact: RiskImpact {
                timeline_impact: Duration::hours(2),
                cost_impact: 0.0,
                quality_impact: QualityImpact::Significant,
                scope_impact: ScopeImpact::None,
            },
            mitigation_strategies: Vec::new(),
            affected_tasks: vec![task.id.clone()],
        })
    }

    fn create_timeline_risk(&self, total: Duration, constraint: Duration) -> HiveResult<Risk> {
        let overrun = total - constraint;
        Ok(Risk {
            id: Uuid::new_v4().to_string(),
            title: "Timeline Overrun Risk".to_string(),
            description: format!("Estimated duration ({} days) exceeds time constraint ({} days) by {} days", 
                total.num_days(), constraint.num_days(), overrun.num_days()),
            severity: RiskSeverity::Critical,
            probability: 0.9,
            impact: RiskImpact {
                timeline_impact: overrun,
                cost_impact: overrun.num_days() as f64 * 1000.0, // Rough cost estimate
                quality_impact: QualityImpact::Moderate,
                scope_impact: ScopeImpact::Major,
            },
            mitigation_strategies: Vec::new(),
            affected_tasks: Vec::new(), // Affects all tasks
        })
    }

    fn create_workload_risk(&self, task_count: usize) -> HiveResult<Risk> {
        Ok(Risk {
            id: Uuid::new_v4().to_string(),
            title: "Single Developer Workload".to_string(),
            description: format!("Single developer responsible for {} tasks may lead to burnout or delays", task_count),
            severity: RiskSeverity::High,
            probability: 0.7,
            impact: RiskImpact {
                timeline_impact: Duration::days(task_count as i64 / 5),
                cost_impact: 0.0,
                quality_impact: QualityImpact::Moderate,
                scope_impact: ScopeImpact::Moderate,
            },
            mitigation_strategies: Vec::new(),
            affected_tasks: Vec::new(),
        })
    }

    fn create_technology_complexity_risk(&self, stack: &[String]) -> HiveResult<Risk> {
        Ok(Risk {
            id: Uuid::new_v4().to_string(),
            title: "Technology Stack Complexity".to_string(),
            description: format!("Project uses {} different technologies which may increase complexity: {}", 
                stack.len(), stack.join(", ")),
            severity: RiskSeverity::Medium,
            probability: 0.6,
            impact: RiskImpact {
                timeline_impact: Duration::days(2),
                cost_impact: 0.0,
                quality_impact: QualityImpact::Moderate,
                scope_impact: ScopeImpact::Minor,
            },
            mitigation_strategies: Vec::new(),
            affected_tasks: Vec::new(),
        })
    }

    fn create_experience_risk(&self) -> HiveResult<Risk> {
        Ok(Risk {
            id: Uuid::new_v4().to_string(),
            title: "Experience Level Mismatch".to_string(),
            description: "Complex design/refactoring tasks assigned to beginner-level team".to_string(),
            severity: RiskSeverity::High,
            probability: 0.8,
            impact: RiskImpact {
                timeline_impact: Duration::days(3),
                cost_impact: 0.0,
                quality_impact: QualityImpact::Significant,
                scope_impact: ScopeImpact::Moderate,
            },
            mitigation_strategies: Vec::new(),
            affected_tasks: Vec::new(),
        })
    }

    fn create_dependency_chain_risk(&self, chain_length: usize) -> HiveResult<Risk> {
        Ok(Risk {
            id: Uuid::new_v4().to_string(),
            title: "Long Dependency Chain".to_string(),
            description: format!("Dependency chain of {} tasks may cause delays to cascade", chain_length),
            severity: RiskSeverity::Medium,
            probability: 0.6,
            impact: RiskImpact {
                timeline_impact: Duration::hours(chain_length as i64 * 2),
                cost_impact: 0.0,
                quality_impact: QualityImpact::Minor,
                scope_impact: ScopeImpact::Minor,
            },
            mitigation_strategies: Vec::new(),
            affected_tasks: Vec::new(),
        })
    }

    fn create_resource_conflict_risk(&self, resource: &str, quantity: f64) -> HiveResult<Risk> {
        Ok(Risk {
            id: Uuid::new_v4().to_string(),
            title: format!("Resource Conflict: {}", resource),
            description: format!("Multiple tasks require {} (total: {:.1}) which may cause scheduling conflicts", resource, quantity),
            severity: RiskSeverity::Medium,
            probability: 0.7,
            impact: RiskImpact {
                timeline_impact: Duration::hours((quantity * 4.0) as i64),
                cost_impact: 0.0,
                quality_impact: QualityImpact::Minor,
                scope_impact: ScopeImpact::Minor,
            },
            mitigation_strategies: Vec::new(),
            affected_tasks: Vec::new(),
        })
    }

    fn has_circular_dependencies(&self, tasks: &[Task]) -> bool {
        // Simple circular dependency check
        // In a real implementation, would use proper graph algorithms
        false
    }

    fn find_longest_dependency_chain(&self, tasks: &[Task]) -> usize {
        // Find the longest chain of dependencies
        // In a real implementation, would use proper graph algorithms
        tasks.iter().map(|t| t.dependencies.len()).max().unwrap_or(0)
    }

    fn score_risks(&self, risks: &mut [Risk]) {
        // Adjust probabilities and severities based on patterns
        for risk in risks {
            // Increase probability for risks with multiple affected tasks
            if risk.affected_tasks.len() > 3 {
                risk.probability = (risk.probability * 1.2).min(1.0);
            }
            
            // Adjust severity based on impact
            if risk.impact.timeline_impact > Duration::days(7) {
                risk.severity = match risk.severity {
                    RiskSeverity::Low => RiskSeverity::Medium,
                    RiskSeverity::Medium => RiskSeverity::High,
                    _ => risk.severity.clone(),
                };
            }
        }
    }

    fn calculate_risk_score(&self, risk: &Risk) -> f32 {
        let severity_score = match risk.severity {
            RiskSeverity::Critical => 4.0,
            RiskSeverity::High => 3.0,
            RiskSeverity::Medium => 2.0,
            RiskSeverity::Low => 1.0,
        };
        severity_score * risk.probability
    }

    fn determine_risk_category(&self, risk: &Risk) -> RiskCategory {
        // Determine category based on risk title and description
        if risk.title.contains("Technology") || risk.title.contains("Complexity") {
            RiskCategory::Technical
        } else if risk.title.contains("Timeline") || risk.title.contains("Deadline") {
            RiskCategory::Timeline
        } else if risk.title.contains("Resource") || risk.title.contains("Skill") {
            RiskCategory::Resource
        } else if risk.title.contains("Test") || risk.title.contains("Quality") {
            RiskCategory::Quality
        } else {
            RiskCategory::Technical // Default
        }
    }

    fn adjust_effectiveness(&self, risk: &Risk) -> f32 {
        // Adjust effectiveness based on risk severity
        match risk.severity {
            RiskSeverity::Critical => 0.8,
            RiskSeverity::High => 0.9,
            RiskSeverity::Medium => 1.0,
            RiskSeverity::Low => 1.1,
        }
    }

    fn calculate_mitigation_cost(&self, risk: &Risk, template: &MitigationTemplate) -> f64 {
        let base_cost = 1000.0; // Base cost per mitigation
        base_cost * (template.cost_factor as f64) * (risk.probability as f64)
    }

    fn calculate_mitigation_time(&self, risk: &Risk, template: &MitigationTemplate) -> Duration {
        let base_time = Duration::hours(4);
        let factor = template.time_factor.max(0.1);
        Duration::seconds((base_time.num_seconds() as f32 * factor) as i64)
    }

    fn generate_specific_mitigations(&self, risk: &Risk) -> HiveResult<Vec<MitigationStrategy>> {
        let mut strategies = Vec::new();
        
        // Add risk-specific mitigations based on the risk type
        if risk.title.contains("Circular Dependencies") {
            strategies.push(MitigationStrategy {
                id: Uuid::new_v4().to_string(),
                description: "Refactor task dependencies to break circular references".to_string(),
                effectiveness: 1.0,
                cost: 0.0,
                implementation_time: Duration::hours(2),
            });
        }
        
        if risk.title.contains("Timeline Overrun") {
            strategies.push(MitigationStrategy {
                id: Uuid::new_v4().to_string(),
                description: "Reduce scope to fit within timeline constraints".to_string(),
                effectiveness: 0.8,
                cost: 0.0,
                implementation_time: Duration::hours(1),
            });
        }
        
        Ok(strategies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_analyzer_creation() {
        let analyzer = RiskAnalyzer::new();
        assert!(!analyzer.risk_patterns.is_empty());
        assert!(!analyzer.mitigation_database.is_empty());
    }

    #[test]
    fn test_risk_score_calculation() {
        let analyzer = RiskAnalyzer::new();
        let risk = Risk {
            id: "test".to_string(),
            title: "Test Risk".to_string(),
            description: "Test".to_string(),
            severity: RiskSeverity::High,
            probability: 0.8,
            impact: RiskImpact {
                timeline_impact: Duration::days(1),
                cost_impact: 0.0,
                quality_impact: QualityImpact::Minor,
                scope_impact: ScopeImpact::None,
            },
            mitigation_strategies: Vec::new(),
            affected_tasks: Vec::new(),
        };
        
        let score = analyzer.calculate_risk_score(&risk);
        assert_eq!(score, 2.4); // High (3.0) * 0.8 = 2.4
    }
}