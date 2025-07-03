//! Collaborative Planning Features
//! 
//! Enables multi-user planning sessions with conflict resolution and team coordination

use crate::core::error::{HiveResult, HiveError};
use crate::planning::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Collaborative planning engine
pub struct CollaborativePlanner {
    conflict_resolver: ConflictResolver,
    assignment_optimizer: AssignmentOptimizer,
}

/// Planning session for collaborative work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningSession {
    pub id: String,
    pub plan_id: String,
    pub participants: Vec<Participant>,
    pub assignments: HashMap<String, Assignment>,
    pub conflicts: Vec<Conflict>,
    pub comments: Vec<Comment>,
    pub votes: HashMap<String, Vec<Vote>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Participant in a planning session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub role: TeamRole,
    pub skills: Vec<String>,
    pub availability: Availability,
    pub workload: f32, // Current workload percentage
}

/// Team member role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TeamRole {
    Lead,
    Developer,
    Designer,
    Tester,
    Reviewer,
    Observer,
}

/// Availability of a participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Availability {
    pub hours_per_day: f32,
    pub days_available: Vec<chrono::Weekday>,
    pub unavailable_dates: Vec<DateTime<Utc>>,
}

/// Task assignment to a participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub task_id: String,
    pub assignee_id: String,
    pub assigned_by: String,
    pub assigned_at: DateTime<Utc>,
    pub estimated_effort: f32, // Hours
    pub status: AssignmentStatus,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssignmentStatus {
    Proposed,
    Accepted,
    Declined,
    InProgress,
    Completed,
}

/// Conflict in planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub conflict_type: ConflictType,
    pub affected_items: Vec<String>,
    pub description: String,
    pub severity: ConflictSeverity,
    pub resolution_options: Vec<ResolutionOption>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    ResourceOverload,
    ScheduleConflict,
    SkillMismatch,
    DependencyIssue,
    PriorityDisagreement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Option for resolving a conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionOption {
    pub id: String,
    pub description: String,
    pub impact: String,
    pub effort: f32,
}

/// Comment on a planning item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author_id: String,
    pub target_id: String, // Task, assignment, or plan ID
    pub target_type: CommentTarget,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub parent_id: Option<String>, // For threaded comments
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommentTarget {
    Plan,
    Task,
    Assignment,
    Risk,
    Conflict,
}

/// Vote on a planning decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter_id: String,
    pub vote_type: VoteType,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteType {
    Approve,
    Reject,
    Abstain,
}

impl CollaborativePlanner {
    pub fn new() -> Self {
        Self {
            conflict_resolver: ConflictResolver::new(),
            assignment_optimizer: AssignmentOptimizer::new(),
        }
    }

    /// Create a new planning session
    pub fn create_session(&self, plan_id: String, participants: Vec<Participant>) -> HiveResult<PlanningSession> {
        Ok(PlanningSession {
            id: Uuid::new_v4().to_string(),
            plan_id,
            participants,
            assignments: HashMap::new(),
            conflicts: Vec::new(),
            comments: Vec::new(),
            votes: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Assign tasks to team members
    pub fn assign_tasks(
        &self,
        session: &mut PlanningSession,
        plan: &Plan,
        manual_assignments: Option<HashMap<String, String>>,
    ) -> HiveResult<()> {
        if let Some(assignments) = manual_assignments {
            // Apply manual assignments
            for (task_id, assignee_id) in assignments {
                self.assign_task(session, &task_id, &assignee_id, "manual")?;
            }
        } else {
            // Auto-assign based on skills and availability
            let optimal_assignments = self.assignment_optimizer.optimize(
                &plan.tasks,
                &session.participants,
            )?;
            
            for (task_id, assignee_id) in optimal_assignments {
                self.assign_task(session, &task_id, &assignee_id, "auto")?;
            }
        }
        
        // Check for conflicts
        self.detect_conflicts(session, plan)?;
        
        session.updated_at = Utc::now();
        Ok(())
    }

    /// Assign a specific task to a participant
    pub fn assign_task(
        &self,
        session: &mut PlanningSession,
        task_id: &str,
        assignee_id: &str,
        assigned_by: &str,
    ) -> HiveResult<()> {
        // Validate assignee exists
        if !session.participants.iter().any(|p| p.id == assignee_id) {
            return Err(HiveError::Planning(format!("Assignee {} not found", assignee_id)));
        }
        
        let assignment = Assignment {
            task_id: task_id.to_string(),
            assignee_id: assignee_id.to_string(),
            assigned_by: assigned_by.to_string(),
            assigned_at: Utc::now(),
            estimated_effort: 0.0, // Will be calculated
            status: AssignmentStatus::Proposed,
            notes: String::new(),
        };
        
        session.assignments.insert(task_id.to_string(), assignment);
        Ok(())
    }

    /// Detect conflicts in the current assignments
    pub fn detect_conflicts(&self, session: &mut PlanningSession, plan: &Plan) -> HiveResult<()> {
        session.conflicts.clear();
        
        // Check resource overload
        let overload_conflicts = self.check_resource_overload(session, plan)?;
        session.conflicts.extend(overload_conflicts);
        
        // Check skill mismatches
        let skill_conflicts = self.check_skill_mismatches(session, plan)?;
        session.conflicts.extend(skill_conflicts);
        
        // Check schedule conflicts
        let schedule_conflicts = self.check_schedule_conflicts(session, plan)?;
        session.conflicts.extend(schedule_conflicts);
        
        Ok(())
    }

    /// Resolve a specific conflict
    pub fn resolve_conflict(
        &self,
        session: &mut PlanningSession,
        conflict_id: &str,
        resolution_id: &str,
    ) -> HiveResult<()> {
        let conflict = session.conflicts.iter_mut()
            .find(|c| c.id == conflict_id)
            .ok_or_else(|| HiveError::Planning("Conflict not found".to_string()))?;
        
        let resolution = conflict.resolution_options.iter()
            .find(|r| r.id == resolution_id)
            .ok_or_else(|| HiveError::Planning("Resolution option not found".to_string()))?
            .clone();
        
        // Apply resolution
        self.conflict_resolver.apply_resolution(session, conflict, &resolution)?;
        
        conflict.resolved = true;
        session.updated_at = Utc::now();
        Ok(())
    }

    /// Add a comment to the planning session
    pub fn add_comment(
        &self,
        session: &mut PlanningSession,
        author_id: &str,
        target_id: &str,
        target_type: CommentTarget,
        content: String,
        parent_id: Option<String>,
    ) -> HiveResult<String> {
        let comment = Comment {
            id: Uuid::new_v4().to_string(),
            author_id: author_id.to_string(),
            target_id: target_id.to_string(),
            target_type,
            content,
            created_at: Utc::now(),
            parent_id,
        };
        
        let comment_id = comment.id.clone();
        session.comments.push(comment);
        session.updated_at = Utc::now();
        
        Ok(comment_id)
    }

    /// Add a vote on a planning item
    pub fn add_vote(
        &self,
        session: &mut PlanningSession,
        item_id: &str,
        voter_id: &str,
        vote_type: VoteType,
        reason: Option<String>,
    ) -> HiveResult<()> {
        let vote = Vote {
            voter_id: voter_id.to_string(),
            vote_type,
            reason,
            timestamp: Utc::now(),
        };
        
        session.votes.entry(item_id.to_string())
            .or_insert_with(Vec::new)
            .push(vote);
        
        session.updated_at = Utc::now();
        Ok(())
    }

    /// Get consensus status for an item
    pub fn get_consensus_status(&self, session: &PlanningSession, item_id: &str) -> ConsensusStatus {
        let votes = session.votes.get(item_id).map(|v| v.as_slice()).unwrap_or(&[]);
        
        if votes.is_empty() {
            return ConsensusStatus::NoVotes;
        }
        
        let total = votes.len();
        let approvals = votes.iter().filter(|v| v.vote_type == VoteType::Approve).count();
        let rejections = votes.iter().filter(|v| v.vote_type == VoteType::Reject).count();
        
        let approval_rate = approvals as f32 / total as f32;
        
        if approval_rate >= 0.8 {
            ConsensusStatus::StrongConsensus
        } else if approval_rate >= 0.6 {
            ConsensusStatus::WeakConsensus
        } else if rejections > approvals {
            ConsensusStatus::Rejected
        } else {
            ConsensusStatus::NoConsensus
        }
    }

    // Private helper methods
    
    fn check_resource_overload(&self, session: &PlanningSession, plan: &Plan) -> HiveResult<Vec<Conflict>> {
        let mut conflicts = Vec::new();
        let mut workload_map: HashMap<String, f32> = HashMap::new();
        
        // Calculate workload for each participant
        for (task_id, assignment) in &session.assignments {
            if let Some(task) = plan.tasks.iter().find(|t| &t.id == task_id) {
                *workload_map.entry(assignment.assignee_id.clone()).or_insert(0.0) += 
                    task.estimated_duration.num_hours() as f32;
            }
        }
        
        // Check for overload
        for participant in &session.participants {
            if let Some(&workload) = workload_map.get(&participant.id) {
                let daily_capacity = participant.availability.hours_per_day;
                let days_needed = workload / daily_capacity;
                
                if days_needed > 20.0 { // More than a month of work
                    conflicts.push(Conflict {
                        id: Uuid::new_v4().to_string(),
                        conflict_type: ConflictType::ResourceOverload,
                        affected_items: vec![participant.id.clone()],
                        description: format!("{} is overloaded with {:.1} days of work", participant.name, days_needed),
                        severity: ConflictSeverity::High,
                        resolution_options: vec![
                            ResolutionOption {
                                id: Uuid::new_v4().to_string(),
                                description: "Reassign some tasks to other team members".to_string(),
                                impact: "Distribute workload more evenly".to_string(),
                                effort: 2.0,
                            },
                            ResolutionOption {
                                id: Uuid::new_v4().to_string(),
                                description: "Extend timeline to accommodate workload".to_string(),
                                impact: "Project will take longer".to_string(),
                                effort: 1.0,
                            },
                        ],
                        resolved: false,
                    });
                }
            }
        }
        
        Ok(conflicts)
    }

    fn check_skill_mismatches(&self, session: &PlanningSession, plan: &Plan) -> HiveResult<Vec<Conflict>> {
        let mut conflicts = Vec::new();
        
        for (task_id, assignment) in &session.assignments {
            if let Some(task) = plan.tasks.iter().find(|t| &t.id == task_id) {
                if let Some(participant) = session.participants.iter().find(|p| p.id == assignment.assignee_id) {
                    // Check if participant has required skills
                    let missing_skills: Vec<String> = task.required_skills.iter()
                        .filter(|skill| !participant.skills.contains(skill))
                        .cloned()
                        .collect();
                    
                    if !missing_skills.is_empty() {
                        conflicts.push(Conflict {
                            id: Uuid::new_v4().to_string(),
                            conflict_type: ConflictType::SkillMismatch,
                            affected_items: vec![task_id.clone(), participant.id.clone()],
                            description: format!("{} lacks skills for {}: {}", 
                                participant.name, task.title, missing_skills.join(", ")),
                            severity: ConflictSeverity::Medium,
                            resolution_options: vec![
                                ResolutionOption {
                                    id: Uuid::new_v4().to_string(),
                                    description: "Reassign to team member with required skills".to_string(),
                                    impact: "Better task-skill alignment".to_string(),
                                    effort: 1.0,
                                },
                                ResolutionOption {
                                    id: Uuid::new_v4().to_string(),
                                    description: "Provide training or pair with experienced member".to_string(),
                                    impact: "Learning opportunity but slower progress".to_string(),
                                    effort: 3.0,
                                },
                            ],
                            resolved: false,
                        });
                    }
                }
            }
        }
        
        Ok(conflicts)
    }

    fn check_schedule_conflicts(&self, session: &PlanningSession, plan: &Plan) -> HiveResult<Vec<Conflict>> {
        let mut conflicts = Vec::new();
        
        // Check for parallel task assignments to same person
        let mut task_schedules: HashMap<String, Vec<(DateTime<Utc>, DateTime<Utc>, String)>> = HashMap::new();
        
        for (task_id, assignment) in &session.assignments {
            if let Some(schedule) = plan.timeline.task_schedules.get(task_id) {
                task_schedules.entry(assignment.assignee_id.clone())
                    .or_insert_with(Vec::new)
                    .push((schedule.planned_start, schedule.planned_end, task_id.clone()));
            }
        }
        
        // Check for overlaps
        for (assignee_id, schedules) in task_schedules {
            for i in 0..schedules.len() {
                for j in i+1..schedules.len() {
                    let (start1, end1, task1) = &schedules[i];
                    let (start2, end2, task2) = &schedules[j];
                    
                    // Check if schedules overlap
                    if start1 < end2 && start2 < end1 {
                        if let Some(participant) = session.participants.iter().find(|p| p.id == assignee_id) {
                            conflicts.push(Conflict {
                                id: Uuid::new_v4().to_string(),
                                conflict_type: ConflictType::ScheduleConflict,
                                affected_items: vec![task1.clone(), task2.clone()],
                                description: format!("{} has overlapping tasks scheduled", participant.name),
                                severity: ConflictSeverity::High,
                                resolution_options: vec![
                                    ResolutionOption {
                                        id: Uuid::new_v4().to_string(),
                                        description: "Reschedule one task to avoid overlap".to_string(),
                                        impact: "May affect dependent tasks".to_string(),
                                        effort: 2.0,
                                    },
                                    ResolutionOption {
                                        id: Uuid::new_v4().to_string(),
                                        description: "Reassign one task to another team member".to_string(),
                                        impact: "Distribute workload".to_string(),
                                        effort: 1.0,
                                    },
                                ],
                                resolved: false,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(conflicts)
    }
}

/// Conflict resolution engine
struct ConflictResolver {
}

impl ConflictResolver {
    fn new() -> Self {
        Self {}
    }

    fn apply_resolution(
        &self,
        session: &mut PlanningSession,
        conflict: &Conflict,
        resolution: &ResolutionOption,
    ) -> HiveResult<()> {
        // Apply resolution based on conflict type
        match conflict.conflict_type {
            ConflictType::ResourceOverload => {
                // Implementation would reassign tasks
            }
            ConflictType::SkillMismatch => {
                // Implementation would find better matches
            }
            ConflictType::ScheduleConflict => {
                // Implementation would reschedule tasks
            }
            _ => {}
        }
        
        Ok(())
    }
}

/// Assignment optimization engine
struct AssignmentOptimizer {
}

impl AssignmentOptimizer {
    fn new() -> Self {
        Self {}
    }

    fn optimize(
        &self,
        tasks: &[Task],
        participants: &[Participant],
    ) -> HiveResult<HashMap<String, String>> {
        let mut assignments = HashMap::new();
        
        // Simple round-robin assignment for now
        // In real implementation would use skills matching and workload balancing
        for (i, task) in tasks.iter().enumerate() {
            let participant = &participants[i % participants.len()];
            assignments.insert(task.id.clone(), participant.id.clone());
        }
        
        Ok(assignments)
    }
}

/// Consensus status for voting
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusStatus {
    NoVotes,
    StrongConsensus,
    WeakConsensus,
    NoConsensus,
    Rejected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaborative_planner_creation() {
        let planner = CollaborativePlanner::new();
        // Basic creation test
    }

    #[test]
    fn test_planning_session_creation() {
        let planner = CollaborativePlanner::new();
        let participants = vec![
            Participant {
                id: "p1".to_string(),
                name: "Alice".to_string(),
                role: TeamRole::Lead,
                skills: vec!["rust".to_string(), "planning".to_string()],
                availability: Availability {
                    hours_per_day: 8.0,
                    days_available: vec![chrono::Weekday::Mon, chrono::Weekday::Tue],
                    unavailable_dates: vec![],
                },
                workload: 0.0,
            },
        ];
        
        let session = planner.create_session("plan123".to_string(), participants).unwrap();
        assert_eq!(session.plan_id, "plan123");
        assert_eq!(session.participants.len(), 1);
    }

    #[test]
    fn test_consensus_calculation() {
        let planner = CollaborativePlanner::new();
        let mut session = planner.create_session("plan123".to_string(), vec![]).unwrap();
        
        // Add votes
        planner.add_vote(&mut session, "item1", "user1", VoteType::Approve, None).unwrap();
        planner.add_vote(&mut session, "item1", "user2", VoteType::Approve, None).unwrap();
        planner.add_vote(&mut session, "item1", "user3", VoteType::Approve, None).unwrap();
        planner.add_vote(&mut session, "item1", "user4", VoteType::Abstain, None).unwrap();
        
        let status = planner.get_consensus_status(&session, "item1");
        assert_eq!(status, ConsensusStatus::WeakConsensus);
    }
}