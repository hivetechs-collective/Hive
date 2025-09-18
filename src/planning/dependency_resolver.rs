//! Dependency Resolution and Graph Management
//!
//! Handles task dependencies, detects cycles, and optimizes execution order

use crate::core::error::{HiveError, HiveResult};
use crate::planning::types::*;
use petgraph::algo::{all_simple_paths, is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use std::collections::{HashMap, HashSet, VecDeque};

/// Dependency resolution engine
pub struct DependencyResolver {
    optimization_enabled: bool,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            optimization_enabled: true,
        }
    }

    /// Resolve dependencies between tasks and create dependency graph
    pub fn resolve(&self, tasks: &[Task]) -> HiveResult<DependencyGraph> {
        // Build task lookup map
        let mut task_map: HashMap<String, Task> = HashMap::new();
        let mut task_by_title: HashMap<String, String> = HashMap::new();

        for task in tasks {
            task_map.insert(task.id.clone(), task.clone());
            task_by_title.insert(task.title.to_lowercase(), task.id.clone());
        }

        // Create graph
        let mut graph = DiGraph::<String, DependencyType>::new();
        let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

        // Add nodes
        for task in tasks {
            let idx = graph.add_node(task.id.clone());
            node_indices.insert(task.id.clone(), idx);
        }

        // Resolve dependencies and add edges
        let mut edges = Vec::new();
        for task in tasks {
            for dep_title in &task.dependencies {
                // Try to match dependency by title
                let dep_id = if let Some(id) = task_by_title.get(&dep_title.to_lowercase()) {
                    id.clone()
                } else {
                    // Try partial match
                    task_by_title
                        .iter()
                        .find(|(title, _)| title.contains(&dep_title.to_lowercase()))
                        .map(|(_, id)| id.clone())
                        .ok_or_else(|| {
                            HiveError::Planning(format!(
                                "Cannot resolve dependency '{}' for task '{}'",
                                dep_title, task.title
                            ))
                        })?
                };

                // Add edge
                let from_idx = node_indices[&dep_id];
                let to_idx = node_indices[&task.id];
                graph.add_edge(from_idx, to_idx, DependencyType::FinishToStart);

                edges.push(Dependency {
                    from_task: dep_id,
                    to_task: task.id.clone(),
                    dependency_type: DependencyType::FinishToStart,
                    lag_time: None,
                });
            }
        }

        // Check for cycles
        if is_cyclic_directed(&graph) {
            return Err(HiveError::Planning(
                "Circular dependencies detected".to_string(),
            ));
        }

        // Find critical path
        let critical_path = self.find_critical_path_petgraph(&graph, &node_indices, tasks)?;

        Ok(DependencyGraph {
            nodes: task_map,
            edges,
            critical_path,
        })
    }

    /// Get execution order for tasks
    pub fn get_execution_order(&self, dependencies: &DependencyGraph) -> HiveResult<Vec<String>> {
        // Build graph for topological sort
        let mut graph = DiGraph::<String, ()>::new();
        let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

        // Add nodes
        for task_id in dependencies.nodes.keys() {
            let idx = graph.add_node(task_id.clone());
            node_indices.insert(task_id.clone(), idx);
        }

        // Add edges
        for dep in &dependencies.edges {
            let from_idx = node_indices[&dep.from_task];
            let to_idx = node_indices[&dep.to_task];
            graph.add_edge(from_idx, to_idx, ());
        }

        // Perform topological sort
        match toposort(&graph, None) {
            Ok(sorted_indices) => {
                let order: Vec<String> = sorted_indices
                    .into_iter()
                    .map(|idx| graph[idx].clone())
                    .collect();
                Ok(order)
            }
            Err(_) => Err(HiveError::Planning(
                "Cannot determine execution order due to cycles".to_string(),
            )),
        }
    }

    /// Optimize task ordering for efficiency
    pub fn optimize_ordering(
        &self,
        tasks: &mut Vec<Task>,
        dependencies: &DependencyGraph,
    ) -> HiveResult<()> {
        if !self.optimization_enabled {
            return Ok(());
        }

        // Get levels for each task (distance from start)
        let levels = self.calculate_task_levels(dependencies)?;

        // Sort tasks by level, then by priority, then by duration
        tasks.sort_by(|a, b| {
            let level_a = levels.get(&a.id).unwrap_or(&0);
            let level_b = levels.get(&b.id).unwrap_or(&0);

            match level_a.cmp(level_b) {
                std::cmp::Ordering::Equal => match a.priority.cmp(&b.priority) {
                    std::cmp::Ordering::Equal => a.estimated_duration.cmp(&b.estimated_duration),
                    other => other,
                },
                other => other,
            }
        });

        Ok(())
    }

    /// Validate that all dependencies are satisfied
    pub fn validate_dependencies(&self, tasks: &[Task]) -> HiveResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        let task_titles: HashSet<String> = tasks.iter().map(|t| t.title.to_lowercase()).collect();

        for task in tasks {
            // Check each dependency exists
            for dep in &task.dependencies {
                if !task_titles.contains(&dep.to_lowercase()) {
                    issues.push(ValidationIssue {
                        task_id: task.id.clone(),
                        issue_type: IssueType::MissingDependency,
                        description: format!("Dependency '{}' not found", dep),
                        severity: IssueSeverity::Error,
                    });
                }
            }

            // Check for self-dependency
            if task
                .dependencies
                .iter()
                .any(|d| d.to_lowercase() == task.title.to_lowercase())
            {
                issues.push(ValidationIssue {
                    task_id: task.id.clone(),
                    issue_type: IssueType::SelfDependency,
                    description: "Task depends on itself".to_string(),
                    severity: IssueSeverity::Error,
                });
            }
        }

        Ok(issues)
    }

    /// Find all paths between two tasks
    pub fn find_paths(
        &self,
        from_task: &str,
        to_task: &str,
        dependencies: &DependencyGraph,
    ) -> HiveResult<Vec<Vec<String>>> {
        // Build graph
        let mut graph = DiGraph::<String, ()>::new();
        let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

        for task_id in dependencies.nodes.keys() {
            let idx = graph.add_node(task_id.clone());
            node_indices.insert(task_id.clone(), idx);
        }

        for dep in &dependencies.edges {
            let from_idx = node_indices[&dep.from_task];
            let to_idx = node_indices[&dep.to_task];
            graph.add_edge(from_idx, to_idx, ());
        }

        // Find paths
        let from_idx = node_indices
            .get(from_task)
            .ok_or_else(|| HiveError::Planning(format!("Task {} not found", from_task)))?;
        let to_idx = node_indices
            .get(to_task)
            .ok_or_else(|| HiveError::Planning(format!("Task {} not found", to_task)))?;

        let paths: Vec<Vec<NodeIndex>> =
            all_simple_paths(&graph, *from_idx, *to_idx, 0, None).collect();

        // Convert to task IDs
        let task_paths: Vec<Vec<String>> = paths
            .into_iter()
            .map(|path| path.into_iter().map(|idx| graph[idx].clone()).collect())
            .collect();

        Ok(task_paths)
    }

    /// Identify tasks that can be executed in parallel
    pub fn find_parallel_tasks(
        &self,
        dependencies: &DependencyGraph,
    ) -> HiveResult<Vec<HashSet<String>>> {
        let mut parallel_groups = Vec::new();
        let levels = self.calculate_task_levels(dependencies)?;

        // Group tasks by level
        let mut level_groups: HashMap<usize, HashSet<String>> = HashMap::new();
        for (task_id, level) in levels {
            level_groups
                .entry(level)
                .or_insert_with(HashSet::new)
                .insert(task_id);
        }

        // Tasks at the same level with no dependencies between them can run in parallel
        for (_, tasks_at_level) in level_groups {
            if tasks_at_level.len() > 1 {
                // Check if any tasks at this level depend on each other
                let mut independent_groups: Vec<HashSet<String>> = Vec::new();

                for task_id in &tasks_at_level {
                    let mut placed = false;
                    for group in &mut independent_groups {
                        if !self.has_dependency_between(task_id, group, dependencies) {
                            group.insert(task_id.clone());
                            placed = true;
                            break;
                        }
                    }
                    if !placed {
                        let mut new_group = HashSet::new();
                        new_group.insert(task_id.clone());
                        independent_groups.push(new_group);
                    }
                }

                parallel_groups.extend(independent_groups.into_iter().filter(|g| g.len() > 1));
            }
        }

        Ok(parallel_groups)
    }

    // Private helper methods

    fn find_critical_path_petgraph(
        &self,
        graph: &DiGraph<String, DependencyType>,
        node_indices: &HashMap<String, NodeIndex>,
        tasks: &[Task],
    ) -> HiveResult<Vec<String>> {
        // Find nodes with no incoming edges (start nodes)
        let start_nodes: Vec<NodeIndex> = node_indices
            .values()
            .filter(|&&idx| graph.neighbors_directed(idx, Direction::Incoming).count() == 0)
            .copied()
            .collect();

        // Find nodes with no outgoing edges (end nodes)
        let end_nodes: Vec<NodeIndex> = node_indices
            .values()
            .filter(|&&idx| graph.neighbors_directed(idx, Direction::Outgoing).count() == 0)
            .copied()
            .collect();

        // Find longest path (critical path)
        let mut longest_path = Vec::new();
        let mut longest_duration = chrono::Duration::zero();

        for &start in &start_nodes {
            for &end in &end_nodes {
                // Find all paths from start to end
                let paths: Vec<Vec<NodeIndex>> =
                    all_simple_paths(graph, start, end, 0, None).collect();

                for path in paths {
                    // Calculate total duration of this path
                    let path_duration: chrono::Duration = path
                        .iter()
                        .map(|&idx| {
                            let task_id = &graph[idx];
                            tasks
                                .iter()
                                .find(|t| &t.id == task_id)
                                .map(|t| t.estimated_duration)
                                .unwrap_or(chrono::Duration::zero())
                        })
                        .sum();

                    if path_duration > longest_duration {
                        longest_duration = path_duration;
                        longest_path = path.into_iter().map(|idx| graph[idx].clone()).collect();
                    }
                }
            }
        }

        Ok(longest_path)
    }

    fn calculate_task_levels(
        &self,
        dependencies: &DependencyGraph,
    ) -> HiveResult<HashMap<String, usize>> {
        let mut levels = HashMap::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Find tasks with no dependencies (level 0)
        for task_id in dependencies.nodes.keys() {
            let has_dependencies = dependencies.edges.iter().any(|e| &e.to_task == task_id);
            if !has_dependencies {
                queue.push_back((task_id.clone(), 0));
                levels.insert(task_id.clone(), 0);
            }
        }

        // BFS to assign levels
        while let Some((task_id, level)) = queue.pop_front() {
            if visited.contains(&task_id) {
                continue;
            }
            visited.insert(task_id.clone());

            // Find tasks that depend on this one
            for edge in &dependencies.edges {
                if edge.from_task == task_id {
                    let new_level = level + 1;
                    let current_level = levels.get(&edge.to_task).copied().unwrap_or(0);
                    if new_level > current_level {
                        levels.insert(edge.to_task.clone(), new_level);
                        queue.push_back((edge.to_task.clone(), new_level));
                    }
                }
            }
        }

        Ok(levels)
    }

    fn has_dependency_between(
        &self,
        task_id: &str,
        group: &HashSet<String>,
        dependencies: &DependencyGraph,
    ) -> bool {
        for other_id in group {
            // Check if task depends on other or other depends on task
            if dependencies.edges.iter().any(|e| {
                (e.from_task == *task_id && e.to_task == *other_id)
                    || (e.from_task == *other_id && e.to_task == *task_id)
            }) {
                return true;
            }
        }
        false
    }
}

/// Validation issue found in dependencies
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub task_id: String,
    pub issue_type: IssueType,
    pub description: String,
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueType {
    MissingDependency,
    SelfDependency,
    CircularDependency,
    RedundantDependency,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_task(title: &str, dependencies: Vec<&str>) -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: format!("Test task: {}", title),
            task_type: TaskType::Implementation,
            priority: Priority::Medium,
            estimated_duration: chrono::Duration::hours(2),
            dependencies: dependencies.into_iter().map(|s| s.to_string()).collect(),
            required_skills: Vec::new(),
            resources: Vec::new(),
            acceptance_criteria: Vec::new(),
            subtasks: Vec::new(),
        }
    }

    #[test]
    fn test_dependency_resolver_creation() {
        let resolver = DependencyResolver::new();
        assert!(resolver.optimization_enabled);
    }

    #[test]
    fn test_simple_dependency_resolution() {
        let resolver = DependencyResolver::new();
        let tasks = vec![
            create_test_task("Task A", vec![]),
            create_test_task("Task B", vec!["Task A"]),
            create_test_task("Task C", vec!["Task B"]),
        ];

        let result = resolver.resolve(&tasks);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let resolver = DependencyResolver::new();
        let tasks = vec![
            create_test_task("Task A", vec!["Task C"]),
            create_test_task("Task B", vec!["Task A"]),
            create_test_task("Task C", vec!["Task B"]),
        ];

        let result = resolver.resolve(&tasks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular"));
    }

    #[test]
    fn test_execution_order() {
        let resolver = DependencyResolver::new();
        let tasks = vec![
            create_test_task("Task A", vec![]),
            create_test_task("Task B", vec!["Task A"]),
            create_test_task("Task C", vec!["Task A"]),
            create_test_task("Task D", vec!["Task B", "Task C"]),
        ];

        let graph = resolver.resolve(&tasks).unwrap();
        let order = resolver.get_execution_order(&graph).unwrap();

        // Task A should come before B and C
        let a_idx = order
            .iter()
            .position(|id| graph.nodes[id].title == "Task A")
            .unwrap();
        let b_idx = order
            .iter()
            .position(|id| graph.nodes[id].title == "Task B")
            .unwrap();
        let c_idx = order
            .iter()
            .position(|id| graph.nodes[id].title == "Task C")
            .unwrap();
        let d_idx = order
            .iter()
            .position(|id| graph.nodes[id].title == "Task D")
            .unwrap();

        assert!(a_idx < b_idx);
        assert!(a_idx < c_idx);
        assert!(b_idx < d_idx);
        assert!(c_idx < d_idx);
    }

    #[test]
    fn test_parallel_task_detection() {
        let resolver = DependencyResolver::new();
        let tasks = vec![
            create_test_task("Task A", vec![]),
            create_test_task("Task B", vec!["Task A"]),
            create_test_task("Task C", vec!["Task A"]),
            create_test_task("Task D", vec!["Task B", "Task C"]),
        ];

        let graph = resolver.resolve(&tasks).unwrap();
        let parallel_groups = resolver.find_parallel_tasks(&graph).unwrap();

        // Tasks B and C should be identified as parallelizable
        assert!(!parallel_groups.is_empty());
        let has_b_and_c = parallel_groups.iter().any(|group| {
            group.iter().any(|id| graph.nodes[id].title == "Task B")
                && group.iter().any(|id| graph.nodes[id].title == "Task C")
        });
        assert!(has_b_and_c);
    }
}
