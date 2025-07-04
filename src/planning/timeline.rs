//! Timeline Estimation Engine
//! 
//! Provides realistic timeline estimates for tasks and projects

use crate::core::error::{HiveResult, HiveError};
use crate::planning::types::*;
use chrono::{DateTime, Duration, Utc, Weekday, Datelike, TimeZone, NaiveTime, Timelike};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

/// Timeline estimation engine
pub struct TimelineEstimator {
    estimation_factors: EstimationFactors,
    working_hours: WorkingHours,
}

/// Factors that affect time estimation
#[derive(Debug, Clone)]
struct EstimationFactors {
    complexity_multipliers: HashMap<TaskType, f32>,
    experience_multipliers: HashMap<ExperienceLevel, f32>,
    team_size_factors: HashMap<usize, f32>,
    buffer_percentages: HashMap<Priority, f32>,
}

/// Working hours configuration
#[derive(Debug, Clone)]
struct WorkingHours {
    daily_hours: f32,
    work_days: HashSet<Weekday>,
    holidays: Vec<DateTime<Utc>>,
    productivity_factor: f32,
}

impl TimelineEstimator {
    pub fn new() -> Self {
        Self {
            estimation_factors: Self::init_estimation_factors(),
            working_hours: Self::init_working_hours(),
        }
    }

    /// Estimate timeline for all tasks considering dependencies
    pub fn estimate(&self, tasks: &[Task], dependencies: &DependencyGraph) -> HiveResult<Timeline> {
        // Calculate individual task durations with factors
        let adjusted_durations = self.calculate_adjusted_durations(tasks)?;
        
        // Find critical path
        let critical_path = self.find_critical_path(tasks, dependencies, &adjusted_durations)?;
        
        // Schedule tasks considering dependencies and resources
        let task_schedules = self.schedule_tasks(tasks, dependencies, &adjusted_durations)?;
        
        // Identify milestones
        let milestones = self.identify_milestones(tasks, &task_schedules)?;
        
        // Calculate overall timeline
        let (start_date, end_date) = self.calculate_project_dates(&task_schedules)?;
        let total_duration = end_date.signed_duration_since(start_date);
        
        Ok(Timeline {
            start_date,
            end_date,
            total_duration: Duration::seconds(total_duration.num_seconds()),
            milestones,
            task_schedules,
        })
    }

    /// Optimize timeline by identifying parallelization opportunities
    pub fn optimize_timeline(&self, timeline: &mut Timeline, dependencies: &DependencyGraph) -> HiveResult<()> {
        // Find tasks that can run in parallel
        let parallel_opportunities = self.find_parallel_opportunities(dependencies)?;
        
        // Reschedule tasks to maximize parallelization
        for (task_ids, potential_savings) in parallel_opportunities {
            self.apply_parallel_scheduling(timeline, &task_ids, potential_savings)?;
        }
        
        // Recalculate end date after optimization
        if let Some(latest_end) = timeline.task_schedules.values()
            .filter_map(|s| s.planned_end.into())
            .max() {
            timeline.end_date = latest_end;
            timeline.total_duration = Duration::seconds(
                latest_end.signed_duration_since(timeline.start_date).num_seconds()
            );
        }
        
        Ok(())
    }

    /// Calculate adjusted durations based on various factors
    fn calculate_adjusted_durations(&self, tasks: &[Task]) -> HiveResult<HashMap<String, Duration>> {
        let mut adjusted = HashMap::new();
        
        for task in tasks {
            let base_duration = task.estimated_duration;
            
            // Apply complexity multiplier
            let complexity_factor = self.estimation_factors.complexity_multipliers
                .get(&task.task_type)
                .unwrap_or(&1.0);
            
            // Apply priority buffer
            let buffer_factor = 1.0 + self.estimation_factors.buffer_percentages
                .get(&task.priority)
                .unwrap_or(&0.2);
            
            // Apply productivity factor
            let productivity_adjusted = base_duration.num_seconds() as f32 
                / self.working_hours.productivity_factor;
            
            // Calculate final duration
            let final_seconds = productivity_adjusted * complexity_factor * buffer_factor;
            let final_duration = Duration::seconds(final_seconds as i64);
            
            adjusted.insert(task.id.clone(), final_duration);
        }
        
        Ok(adjusted)
    }

    /// Find the critical path through the project
    fn find_critical_path(
        &self,
        tasks: &[Task],
        dependencies: &DependencyGraph,
        durations: &HashMap<String, Duration>,
    ) -> HiveResult<Vec<String>> {
        // Build adjacency list
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        
        for task in tasks {
            adj_list.entry(task.id.clone()).or_insert_with(Vec::new);
            in_degree.entry(task.id.clone()).or_insert(0);
        }
        
        for dep in &dependencies.edges {
            adj_list.get_mut(&dep.from_task)
                .ok_or_else(|| HiveError::Planning("Invalid dependency".to_string()))?
                .push(dep.to_task.clone());
            *in_degree.get_mut(&dep.to_task).unwrap() += 1;
        }
        
        // Calculate earliest start times
        let mut earliest_start: HashMap<String, Duration> = HashMap::new();
        let mut earliest_finish: HashMap<String, Duration> = HashMap::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        
        // Initialize with tasks that have no dependencies
        for (task_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(task_id.clone());
                earliest_start.insert(task_id.clone(), Duration::zero());
                let zero_duration = Duration::zero();
                let duration = durations.get(task_id).unwrap_or(&zero_duration);
                earliest_finish.insert(task_id.clone(), *duration);
            }
        }
        
        // Process tasks in topological order
        while let Some(task_id) = queue.pop_front() {
            let task_finish = earliest_finish.get(&task_id).copied().unwrap_or(Duration::zero());
            
            if let Some(dependents) = adj_list.get(&task_id) {
                for dep_id in dependents {
                    // Update earliest start time for dependent
                    let current_start = earliest_start.get(dep_id).copied().unwrap_or(Duration::zero());
                    let new_start = current_start.max(task_finish);
                    earliest_start.insert(dep_id.clone(), new_start);
                    
                    // Update earliest finish time
                    let zero_duration = Duration::zero();
                    let dep_duration = durations.get(dep_id).unwrap_or(&zero_duration);
                    earliest_finish.insert(dep_id.clone(), new_start + *dep_duration);
                    
                    // Decrement in-degree and add to queue if ready
                    if let Some(degree) = in_degree.get_mut(dep_id) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dep_id.clone());
                        }
                    }
                }
            }
        }
        
        // Find the critical path by backtracking from the latest finish
        let mut critical_path = Vec::new();
        if let Some((last_task, _)) = earliest_finish.iter().max_by_key(|(_,&v)| v) {
            critical_path.push(last_task.clone());
            // Simple approximation - in real implementation would backtrack properly
        }
        
        Ok(critical_path)
    }

    /// Schedule tasks considering dependencies and working hours
    fn schedule_tasks(
        &self,
        tasks: &[Task],
        dependencies: &DependencyGraph,
        durations: &HashMap<String, Duration>,
    ) -> HiveResult<HashMap<String, TaskSchedule>> {
        let mut schedules = HashMap::new();
        let mut task_end_times: HashMap<String, DateTime<Utc>> = HashMap::new();
        
        // Get topological order
        let ordered_tasks = self.topological_sort(tasks, dependencies)?;
        
        // Schedule each task
        for task_id in ordered_tasks {
            let task = tasks.iter().find(|t| t.id == task_id)
                .ok_or_else(|| HiveError::Planning("Task not found".to_string()))?;
            
            // Find earliest possible start time
            let mut start_time = Utc::now();
            
            // Check dependencies
            for dep_id in &task.dependencies {
                if let Some(&dep_end) = task_end_times.get(dep_id) {
                    start_time = start_time.max(dep_end);
                }
            }
            
            // Adjust for working hours
            start_time = self.next_working_time(start_time);
            
            // Calculate end time
            let duration = durations.get(&task_id).unwrap_or(&task.estimated_duration);
            let end_time = self.add_working_duration(start_time, *duration)?;
            
            // Create schedule
            let schedule = TaskSchedule {
                task_id: task_id.clone(),
                planned_start: start_time,
                planned_end: end_time,
                actual_start: None,
                actual_end: None,
                progress: 0.0,
            };
            
            schedules.insert(task_id.clone(), schedule);
            task_end_times.insert(task_id, end_time);
        }
        
        Ok(schedules)
    }

    /// Identify project milestones
    fn identify_milestones(
        &self,
        tasks: &[Task],
        schedules: &HashMap<String, TaskSchedule>,
    ) -> HiveResult<Vec<Milestone>> {
        let mut milestones = Vec::new();
        
        // Group tasks by major deliverables
        let mut deliverable_groups: HashMap<TaskType, Vec<String>> = HashMap::new();
        for task in tasks {
            deliverable_groups.entry(task.task_type.clone())
                .or_insert_with(Vec::new)
                .push(task.id.clone());
        }
        
        // Create milestones for major task type completions
        for (task_type, task_ids) in deliverable_groups {
            // Find latest completion in this group
            let latest_date = task_ids.iter()
                .filter_map(|id| schedules.get(id))
                .map(|s| s.planned_end)
                .max();
            
            if let Some(date) = latest_date {
                milestones.push(Milestone {
                    id: Uuid::new_v4().to_string(),
                    name: format!("{:?} Complete", task_type),
                    date,
                    tasks: task_ids,
                    deliverables: vec![format!("All {:?} tasks completed", task_type)],
                });
            }
        }
        
        // Sort milestones by date
        milestones.sort_by_key(|m| m.date);
        
        Ok(milestones)
    }

    /// Calculate project start and end dates
    fn calculate_project_dates(
        &self,
        schedules: &HashMap<String, TaskSchedule>,
    ) -> HiveResult<(DateTime<Utc>, DateTime<Utc>)> {
        let start_date = schedules.values()
            .map(|s| s.planned_start)
            .min()
            .unwrap_or_else(Utc::now);
        
        let end_date = schedules.values()
            .map(|s| s.planned_end)
            .max()
            .unwrap_or_else(|| Utc::now() + Duration::days(1));
        
        Ok((start_date, end_date))
    }

    /// Find opportunities for parallel execution
    fn find_parallel_opportunities(
        &self,
        dependencies: &DependencyGraph,
    ) -> HiveResult<Vec<(Vec<String>, Duration)>> {
        let mut opportunities = Vec::new();
        
        // Find tasks with no shared dependencies
        let mut independent_groups: Vec<Vec<String>> = Vec::new();
        
        // Simple parallel detection - in real implementation would be more sophisticated
        for (task_id, task) in &dependencies.nodes {
            let mut can_parallel_with = Vec::new();
            can_parallel_with.push(task_id.clone());
            
            for (other_id, other_task) in &dependencies.nodes {
                if task_id != other_id && !self.has_dependency_conflict(task_id, other_id, dependencies) {
                    can_parallel_with.push(other_id.clone());
                }
            }
            
            if can_parallel_with.len() > 1 {
                let potential_savings = Duration::hours((can_parallel_with.len() - 1) as i64 * 2);
                opportunities.push((can_parallel_with, potential_savings));
            }
        }
        
        Ok(opportunities)
    }

    /// Apply parallel scheduling to optimize timeline
    fn apply_parallel_scheduling(
        &self,
        timeline: &mut Timeline,
        task_ids: &[String],
        savings: Duration,
    ) -> HiveResult<()> {
        // Find earliest common start time
        let start_time = task_ids.iter()
            .filter_map(|id| timeline.task_schedules.get(id))
            .map(|s| s.planned_start)
            .min()
            .unwrap_or_else(Utc::now);
        
        // Reschedule all tasks to start at the same time
        for task_id in task_ids {
            if let Some(schedule) = timeline.task_schedules.get_mut(task_id) {
                let duration = schedule.planned_end.signed_duration_since(schedule.planned_start);
                schedule.planned_start = start_time;
                schedule.planned_end = start_time + Duration::seconds(duration.num_seconds());
            }
        }
        
        Ok(())
    }

    /// Check if two tasks have dependency conflicts
    fn has_dependency_conflict(&self, task1: &str, task2: &str, dependencies: &DependencyGraph) -> bool {
        // Check if task1 depends on task2 or vice versa
        dependencies.edges.iter().any(|dep| 
            (dep.from_task == task1 && dep.to_task == task2) ||
            (dep.from_task == task2 && dep.to_task == task1)
        )
    }

    /// Topological sort of tasks
    fn topological_sort(&self, tasks: &[Task], dependencies: &DependencyGraph) -> HiveResult<Vec<String>> {
        let mut sorted = Vec::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        
        // Initialize
        for task in tasks {
            in_degree.insert(task.id.clone(), 0);
            adj_list.insert(task.id.clone(), Vec::new());
        }
        
        // Build graph
        for dep in &dependencies.edges {
            adj_list.get_mut(&dep.from_task).unwrap().push(dep.to_task.clone());
            *in_degree.get_mut(&dep.to_task).unwrap() += 1;
        }
        
        // Find all nodes with in-degree 0
        let mut queue: VecDeque<String> = in_degree.iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        // Process queue
        while let Some(task_id) = queue.pop_front() {
            sorted.push(task_id.clone());
            
            if let Some(neighbors) = adj_list.get(&task_id) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }
        
        if sorted.len() != tasks.len() {
            return Err(HiveError::Planning("Circular dependency detected".to_string()));
        }
        
        Ok(sorted)
    }

    /// Get next working time from given datetime
    fn next_working_time(&self, dt: DateTime<Utc>) -> DateTime<Utc> {
        let mut current = dt;
        
        // Skip to next working day if needed
        while !self.working_hours.work_days.contains(&current.weekday()) ||
              self.working_hours.holidays.iter().any(|h| h.date_naive() == current.date_naive()) {
            current = current + Duration::days(1);
            // Reset to start of working day
            current = current.date_naive().and_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap()).and_utc(); // 9 AM
        }
        
        // Ensure within working hours
        let hour = current.time().hour();
        if hour < 9 {
            current = current.date_naive().and_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap()).and_utc();
        } else if hour >= 17 {
            // Move to next working day
            current = self.next_working_time(current + Duration::days(1));
        }
        
        current
    }

    /// Add duration considering only working hours
    fn add_working_duration(&self, start: DateTime<Utc>, duration: Duration) -> HiveResult<DateTime<Utc>> {
        let mut current = start;
        let mut remaining_hours = duration.num_hours() as f32;
        
        while remaining_hours > 0.0 {
            // Calculate hours available today
            let hours_today = (17 - current.time().hour()).min(8) as f32;
            let hours_to_add = remaining_hours.min(hours_today);
            
            if hours_to_add > 0.0 {
                current = current + Duration::seconds((hours_to_add * 3600.0) as i64);
                remaining_hours -= hours_to_add;
            }
            
            if remaining_hours > 0.0 {
                // Move to next working day
                current = self.next_working_time(current + Duration::days(1));
            }
        }
        
        Ok(current)
    }

    // Initialization methods

    fn init_estimation_factors() -> EstimationFactors {
        let mut complexity_multipliers = HashMap::new();
        complexity_multipliers.insert(TaskType::Implementation, 1.0);
        complexity_multipliers.insert(TaskType::Testing, 0.8);
        complexity_multipliers.insert(TaskType::Documentation, 0.6);
        complexity_multipliers.insert(TaskType::Review, 0.4);
        complexity_multipliers.insert(TaskType::Deployment, 0.7);
        complexity_multipliers.insert(TaskType::Research, 1.5);
        complexity_multipliers.insert(TaskType::Design, 1.2);
        complexity_multipliers.insert(TaskType::Refactoring, 1.3);
        complexity_multipliers.insert(TaskType::BugFix, 0.9);
        complexity_multipliers.insert(TaskType::Configuration, 0.5);
        
        let mut experience_multipliers = HashMap::new();
        experience_multipliers.insert(ExperienceLevel::Beginner, 2.0);
        experience_multipliers.insert(ExperienceLevel::Intermediate, 1.3);
        experience_multipliers.insert(ExperienceLevel::Advanced, 1.0);
        experience_multipliers.insert(ExperienceLevel::Expert, 0.8);
        
        let mut team_size_factors = HashMap::new();
        team_size_factors.insert(1, 1.0);
        team_size_factors.insert(2, 0.9);
        team_size_factors.insert(3, 0.85);
        team_size_factors.insert(4, 0.8);
        team_size_factors.insert(5, 0.8);
        
        let mut buffer_percentages = HashMap::new();
        buffer_percentages.insert(Priority::Critical, 0.3);
        buffer_percentages.insert(Priority::High, 0.2);
        buffer_percentages.insert(Priority::Medium, 0.15);
        buffer_percentages.insert(Priority::Low, 0.1);
        
        EstimationFactors {
            complexity_multipliers,
            experience_multipliers,
            team_size_factors,
            buffer_percentages,
        }
    }

    fn init_working_hours() -> WorkingHours {
        let mut work_days = HashSet::new();
        work_days.insert(Weekday::Mon);
        work_days.insert(Weekday::Tue);
        work_days.insert(Weekday::Wed);
        work_days.insert(Weekday::Thu);
        work_days.insert(Weekday::Fri);
        
        WorkingHours {
            daily_hours: 8.0,
            work_days,
            holidays: Vec::new(), // Would load from configuration
            productivity_factor: 0.7, // 70% productive time
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_estimator_creation() {
        let estimator = TimelineEstimator::new();
        assert_eq!(estimator.working_hours.daily_hours, 8.0);
        assert_eq!(estimator.working_hours.productivity_factor, 0.7);
    }

    #[test]
    fn test_next_working_time() {
        let estimator = TimelineEstimator::new();
        
        // Test weekend skip
        let friday_evening = Utc::now().date().and_hms(18, 0, 0);
        let next = estimator.next_working_time(friday_evening);
        assert!(next.hour() == 9); // Should be 9 AM
    }
}