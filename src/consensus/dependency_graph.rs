// Operation Dependency Graph Generation and Sequencing
use anyhow::{Result, anyhow};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{toposort, is_cyclic_directed, all_simple_paths};
use petgraph::dot::{Dot, Config};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::operation_parser::EnhancedFileOperation;
use crate::consensus::operation_clustering::OperationCluster;
use crate::ai_helpers::pattern_recognizer::PatternRecognizer;

/// Generates and analyzes dependency graphs for file operations
#[derive(Debug, Clone)]
pub struct DependencyGraphGenerator {
    /// AI helper for pattern analysis
    pattern_recognizer: Option<Arc<PatternRecognizer>>,
    /// Configuration for graph generation
    config: GraphConfig,
    /// Cache for generated graphs
    graph_cache: Arc<RwLock<HashMap<String, DependencyGraph>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Include implicit dependencies (e.g., imports)
    pub include_implicit_dependencies: bool,
    /// Analyze cross-file dependencies
    pub analyze_cross_file_deps: bool,
    /// Maximum dependency depth to analyze
    pub max_dependency_depth: usize,
    /// Generate visual representations
    pub generate_visuals: bool,
    /// Include AI-predicted dependencies
    pub use_ai_predictions: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            include_implicit_dependencies: true,
            analyze_cross_file_deps: true,
            max_dependency_depth: 10,
            generate_visuals: true,
            use_ai_predictions: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// The underlying graph structure
    pub graph: DiGraph<OperationNode, DependencyEdge>,
    /// Node indices mapped by operation ID
    pub node_map: HashMap<String, NodeIndex>,
    /// Execution sequence (topologically sorted)
    pub execution_sequence: Vec<String>,
    /// Critical path through the graph
    pub critical_path: Vec<String>,
    /// Parallel execution groups
    pub parallel_groups: Vec<ParallelGroup>,
    /// Graph analysis results
    pub analysis: GraphAnalysis,
    /// Visual representations
    pub visuals: Option<GraphVisuals>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationNode {
    /// Unique operation ID
    pub id: String,
    /// The operation
    pub operation: FileOperation,
    /// Node metadata
    pub metadata: NodeMetadata,
    /// Estimated execution time
    pub estimated_duration_ms: u64,
    /// Can be parallelized
    pub parallelizable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Operation type for visualization
    pub operation_type: String,
    /// File path
    pub file_path: PathBuf,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Node color for visualization
    pub color: String,
    /// Node shape for visualization
    pub shape: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Strength of dependency (0.0-1.0)
    pub strength: f32,
    /// Is this a hard requirement
    pub required: bool,
    /// Edge label for visualization
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    /// File must exist before operation
    FileExistence,
    /// Content dependency
    ContentDependency,
    /// Import/module dependency
    ImportDependency,
    /// Ordering constraint
    OrderingConstraint,
    /// AI-predicted dependency
    PredictedDependency,
    /// User-specified dependency
    UserSpecified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelGroup {
    /// Group ID
    pub id: usize,
    /// Operations that can run in parallel
    pub operations: Vec<String>,
    /// Estimated total time if run in parallel
    pub parallel_duration_ms: u64,
    /// Dependencies on other groups
    pub depends_on: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysis {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Maximum dependency depth
    pub max_depth: usize,
    /// Number of parallel groups
    pub parallel_group_count: usize,
    /// Potential speedup from parallelization
    pub parallelization_speedup: f32,
    /// Cycle detection results
    pub has_cycles: bool,
    /// Detected cycles (if any)
    pub cycles: Vec<Vec<String>>,
    /// Bottleneck operations
    pub bottlenecks: Vec<String>,
    /// Risk analysis
    pub risk_summary: RiskSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummary {
    /// High-risk operation count
    pub high_risk_count: usize,
    /// Critical dependencies
    pub critical_dependencies: Vec<(String, String)>,
    /// Risk mitigation suggestions
    pub mitigation_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphVisuals {
    /// DOT format representation
    pub dot_graph: String,
    /// ASCII art visualization
    pub ascii_graph: String,
    /// Mermaid diagram
    pub mermaid_diagram: String,
    /// Execution timeline
    pub timeline_chart: String,
}

impl DependencyGraphGenerator {
    pub fn new(
        pattern_recognizer: Option<Arc<PatternRecognizer>>,
        config: Option<GraphConfig>,
    ) -> Self {
        Self {
            pattern_recognizer,
            config: config.unwrap_or_default(),
            graph_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn generate_dependency_graph(
        &self,
        operations: &[EnhancedFileOperation],
        clusters: Option<&[OperationCluster]>,
    ) -> Result<DependencyGraph> {
        let cache_key = self.generate_cache_key(operations);
        
        // Check cache
        if let Some(cached) = self.graph_cache.read().await.get(&cache_key) {
            debug!("Using cached dependency graph");
            return Ok(cached.clone());
        }

        // Build the graph
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        // Add nodes
        for (i, enhanced_op) in operations.iter().enumerate() {
            let node = self.create_operation_node(enhanced_op, i)?;
            let idx = graph.add_node(node);
            node_map.insert(format!("op_{}", i), idx);
        }

        // Add edges based on dependencies
        self.add_explicit_dependencies(&mut graph, operations, &node_map)?;
        
        if self.config.include_implicit_dependencies {
            self.add_implicit_dependencies(&mut graph, operations, &node_map).await?;
        }

        if self.config.use_ai_predictions && self.pattern_recognizer.is_some() {
            self.add_ai_predicted_dependencies(&mut graph, operations, &node_map).await?;
        }

        // Analyze the graph
        let analysis = self.analyze_graph(&graph, &node_map)?;
        
        // Generate execution sequence
        let execution_sequence = self.generate_execution_sequence(&graph, &node_map)?;
        
        // Find critical path
        let critical_path = self.find_critical_path(&graph, &node_map)?;
        
        // Identify parallel groups
        let parallel_groups = self.identify_parallel_groups(&graph, &node_map, &execution_sequence)?;
        
        // Generate visuals if enabled
        let visuals = if self.config.generate_visuals {
            Some(self.generate_visuals(&graph, &node_map)?)
        } else {
            None
        };

        let dependency_graph = DependencyGraph {
            graph,
            node_map,
            execution_sequence,
            critical_path,
            parallel_groups,
            analysis,
            visuals,
        };

        // Cache the result
        self.graph_cache.write().await.insert(cache_key, dependency_graph.clone());

        Ok(dependency_graph)
    }

    fn create_operation_node(&self, enhanced_op: &EnhancedFileOperation, index: usize) -> Result<OperationNode> {
        let (operation_type, color, shape) = match &enhanced_op.operation {
            FileOperation::Create { .. } => ("Create", "#4CAF50", "box"),
            FileOperation::Update { .. } => ("Update", "#2196F3", "ellipse"),
            FileOperation::Delete { .. } => ("Delete", "#F44336", "diamond"),
            FileOperation::Rename { .. } => ("Rename", "#FF9800", "parallelogram"),
        };

        let file_path = self.get_operation_path(&enhanced_op.operation);
        let risk_level = self.assess_operation_risk(&enhanced_op.operation);

        let metadata = NodeMetadata {
            operation_type: operation_type.to_string(),
            file_path: file_path.clone(),
            risk_level,
            color: color.to_string(),
            shape: shape.to_string(),
        };

        Ok(OperationNode {
            id: format!("op_{}", index),
            operation: enhanced_op.operation.clone(),
            metadata,
            estimated_duration_ms: self.estimate_duration(&enhanced_op.operation),
            parallelizable: enhanced_op.context.dependencies.is_empty(),
        })
    }

    fn add_explicit_dependencies(
        &self,
        graph: &mut DiGraph<OperationNode, DependencyEdge>,
        operations: &[EnhancedFileOperation],
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<()> {
        // Add dependencies from parsed context
        for (i, enhanced_op) in operations.iter().enumerate() {
            let from_id = format!("op_{}", i);
            let from_idx = node_map[&from_id];

            for dep_index in &enhanced_op.context.dependencies {
                let to_id = format!("op_{}", dep_index);
                if let Some(&to_idx) = node_map.get(&to_id) {
                    let edge = DependencyEdge {
                        dependency_type: DependencyType::UserSpecified,
                        strength: 1.0,
                        required: true,
                        label: "depends on".to_string(),
                    };
                    graph.add_edge(from_idx, to_idx, edge);
                }
            }
        }

        // Add file existence dependencies
        for i in 0..operations.len() {
            for j in 0..operations.len() {
                if i == j {
                    continue;
                }

                let dep_type = self.check_file_dependency(&operations[i].operation, &operations[j].operation);
                if let Some(dep) = dep_type {
                    let from_idx = node_map[&format!("op_{}", i)];
                    let to_idx = node_map[&format!("op_{}", j)];
                    graph.add_edge(from_idx, to_idx, dep);
                }
            }
        }

        Ok(())
    }

    fn check_file_dependency(&self, op1: &FileOperation, op2: &FileOperation) -> Option<DependencyEdge> {
        match (op1, op2) {
            // Update depends on Create for same file
            (FileOperation::Update { path: p1, .. }, FileOperation::Create { path: p2, .. }) => {
                if p1 == p2 {
                    Some(DependencyEdge {
                        dependency_type: DependencyType::FileExistence,
                        strength: 1.0,
                        required: true,
                        label: "requires file".to_string(),
                    })
                } else {
                    None
                }
            }
            // Move/Rename depends on file existence
            (FileOperation::Rename { old_path: s1, .. }, FileOperation::Create { path: p2, .. }) => {
                if s1 == p2 {
                    Some(DependencyEdge {
                        dependency_type: DependencyType::FileExistence,
                        strength: 1.0,
                        required: true,
                        label: "requires source".to_string(),
                    })
                } else {
                    None
                }
            }
            // Delete should happen after other operations on same file
            (FileOperation::Delete { path: p1 }, FileOperation::Update { path: p2, .. }) |
            (FileOperation::Delete { path: p1 }, FileOperation::Create { path: p2, .. }) => {
                if p1 == p2 {
                    Some(DependencyEdge {
                        dependency_type: DependencyType::OrderingConstraint,
                        strength: 1.0,
                        required: true,
                        label: "delete after".to_string(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    async fn add_implicit_dependencies(
        &self,
        graph: &mut DiGraph<OperationNode, DependencyEdge>,
        operations: &[EnhancedFileOperation],
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<()> {
        if !self.config.analyze_cross_file_deps {
            return Ok(());
        }

        // Analyze content for import dependencies
        for (i, enhanced_op) in operations.iter().enumerate() {
            if let FileOperation::Create { content, .. } | FileOperation::Update { new_content: content, .. } = &enhanced_op.operation {
                let imports = self.extract_imports(content, &self.get_operation_path(&enhanced_op.operation))?;
                
                for import_path in imports {
                    // Find operations that create/modify the imported file
                    for (j, other_op) in operations.iter().enumerate() {
                        if i == j {
                            continue;
                        }

                        let other_path = self.get_operation_path(&other_op.operation);
                        if self.path_matches_import(&other_path, &import_path) {
                            let from_idx = node_map[&format!("op_{}", i)];
                            let to_idx = node_map[&format!("op_{}", j)];
                            
                            let edge = DependencyEdge {
                                dependency_type: DependencyType::ImportDependency,
                                strength: 0.8,
                                required: false,
                                label: "imports".to_string(),
                            };
                            graph.add_edge(from_idx, to_idx, edge);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn extract_imports(&self, content: &str, file_path: &Path) -> Result<Vec<PathBuf>> {
        let mut imports = Vec::new();
        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "rs" => {
                // Rust imports
                let use_regex = regex::Regex::new(r"use\s+([a-zA-Z0-9_:]+)")?;
                let mod_regex = regex::Regex::new(r"mod\s+([a-zA-Z0-9_]+)")?;
                
                for caps in use_regex.captures_iter(content) {
                    let module = &caps[1];
                    if !module.starts_with("std::") && !module.starts_with("core::") {
                        imports.push(PathBuf::from(module.replace("::", "/")));
                    }
                }
                
                for caps in mod_regex.captures_iter(content) {
                    imports.push(PathBuf::from(format!("{}.rs", &caps[1])));
                }
            }
            "js" | "ts" => {
                // JavaScript/TypeScript imports
                let import_regex = regex::Regex::new(r#"import\s+.*?\s+from\s+['"](\..*?)['""]"#)?;
                let require_regex = regex::Regex::new(r#"require\(['"](\..*?)['"]\)"#)?;
                
                for caps in import_regex.captures_iter(content) {
                    imports.push(PathBuf::from(&caps[1]));
                }
                
                for caps in require_regex.captures_iter(content) {
                    imports.push(PathBuf::from(&caps[1]));
                }
            }
            "py" => {
                // Python imports
                let import_regex = regex::Regex::new(r"from\s+([a-zA-Z0-9_.]+)\s+import")?;
                let import_direct = regex::Regex::new(r"import\s+([a-zA-Z0-9_.]+)")?;
                
                for caps in import_regex.captures_iter(content) {
                    let module = &caps[1];
                    if !module.starts_with("__") {
                        imports.push(PathBuf::from(module.replace(".", "/")));
                    }
                }
                
                for caps in import_direct.captures_iter(content) {
                    let module = &caps[1];
                    if !module.starts_with("__") {
                        imports.push(PathBuf::from(module.replace(".", "/")));
                    }
                }
            }
            _ => {}
        }

        Ok(imports)
    }

    fn path_matches_import(&self, file_path: &Path, import_path: &Path) -> bool {
        // Simple heuristic: check if the file path ends with the import path
        file_path.ends_with(import_path) ||
        file_path.file_stem() == import_path.file_stem()
    }

    async fn add_ai_predicted_dependencies(
        &self,
        graph: &mut DiGraph<OperationNode, DependencyEdge>,
        operations: &[EnhancedFileOperation],
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<()> {
        if let Some(pattern_recognizer) = &self.pattern_recognizer {
            // Use pattern recognizer to predict dependencies
            // This is a simplified version - real implementation would use AI
            
            for i in 0..operations.len() {
                for j in i+1..operations.len() {
                    let op1 = &operations[i].operation;
                    let op2 = &operations[j].operation;
                    
                    // Predict if op2 might depend on op1
                    let dependency_probability = self.predict_dependency_probability(op1, op2)?;
                    
                    if dependency_probability > 0.6 {
                        let from_idx = node_map[&format!("op_{}", j)];
                        let to_idx = node_map[&format!("op_{}", i)];
                        
                        let edge = DependencyEdge {
                            dependency_type: DependencyType::PredictedDependency,
                            strength: dependency_probability,
                            required: false,
                            label: format!("predicted ({:.0}%)", dependency_probability * 100.0),
                        };
                        
                        // Only add if edge doesn't exist
                        if !graph.contains_edge(from_idx, to_idx) {
                            graph.add_edge(from_idx, to_idx, edge);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn predict_dependency_probability(&self, op1: &FileOperation, op2: &FileOperation) -> Result<f32> {
        // Simple heuristic-based prediction
        let path1 = self.get_operation_path(op1);
        let path2 = self.get_operation_path(op2);
        
        // Same directory operations might be related
        let same_dir = path1.parent() == path2.parent();
        
        // Test files often depend on implementation
        let is_test_impl = path2.to_string_lossy().contains("test") && 
                          !path1.to_string_lossy().contains("test");
        
        // Config files might affect other operations
        let is_config = path1.extension().map_or(false, |e| 
            e == "toml" || e == "yaml" || e == "json" || e == "env"
        );

        let probability = match (same_dir, is_test_impl, is_config) {
            (true, true, _) => 0.9,    // Test depends on implementation
            (_, _, true) => 0.7,        // Config affects other files
            (true, false, false) => 0.5, // Same directory
            _ => 0.2,
        };

        Ok(probability)
    }

    fn analyze_graph(
        &self,
        graph: &DiGraph<OperationNode, DependencyEdge>,
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<GraphAnalysis> {
        let node_count = graph.node_count();
        let edge_count = graph.edge_count();
        
        // Check for cycles
        let has_cycles = is_cyclic_directed(graph);
        let cycles = if has_cycles {
            self.find_cycles(graph, node_map)?
        } else {
            Vec::new()
        };

        // Calculate max depth
        let max_depth = self.calculate_max_depth(graph)?;
        
        // Find bottlenecks (nodes with many dependents)
        let bottlenecks = self.find_bottlenecks(graph, node_map)?;
        
        // Analyze parallelization potential
        let (parallel_group_count, parallelization_speedup) = self.analyze_parallelization(graph)?;
        
        // Risk analysis
        let risk_summary = self.analyze_risk(graph, node_map)?;

        Ok(GraphAnalysis {
            node_count,
            edge_count,
            max_depth,
            parallel_group_count,
            parallelization_speedup,
            has_cycles,
            cycles,
            bottlenecks,
            risk_summary,
        })
    }

    fn find_cycles(&self, graph: &DiGraph<OperationNode, DependencyEdge>, node_map: &HashMap<String, NodeIndex>) -> Result<Vec<Vec<String>>> {
        // Simple cycle detection - in practice would use more sophisticated algorithm
        let mut cycles = Vec::new();
        
        // For now, just note that cycles exist
        if is_cyclic_directed(graph) {
            cycles.push(vec!["Cycle detected - manual resolution required".to_string()]);
        }
        
        Ok(cycles)
    }

    fn calculate_max_depth(&self, graph: &DiGraph<OperationNode, DependencyEdge>) -> Result<usize> {
        let mut max_depth = 0;
        
        // Find nodes with no incoming edges (roots)
        let roots: Vec<_> = graph.node_indices()
            .filter(|&node| graph.edges_directed(node, petgraph::Direction::Incoming).count() == 0)
            .collect();

        // BFS from each root to find max depth
        for root in roots {
            let depth = self.bfs_max_depth(graph, root)?;
            max_depth = max_depth.max(depth);
        }

        Ok(max_depth)
    }

    fn bfs_max_depth(&self, graph: &DiGraph<OperationNode, DependencyEdge>, start: NodeIndex) -> Result<usize> {
        let mut queue = VecDeque::new();
        let mut depths = HashMap::new();
        
        queue.push_back(start);
        depths.insert(start, 0);
        let mut max_depth = 0;

        while let Some(node) = queue.pop_front() {
            let current_depth = depths[&node];
            max_depth = max_depth.max(current_depth);

            for edge in graph.edges(node) {
                let target = edge.target();
                if !depths.contains_key(&target) {
                    depths.insert(target, current_depth + 1);
                    queue.push_back(target);
                }
            }
        }

        Ok(max_depth)
    }

    fn find_bottlenecks(
        &self,
        graph: &DiGraph<OperationNode, DependencyEdge>,
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<Vec<String>> {
        let mut bottlenecks = Vec::new();
        
        for (id, &idx) in node_map {
            let dependent_count = graph.edges_directed(idx, petgraph::Direction::Outgoing).count();
            if dependent_count > 3 {
                bottlenecks.push(id.clone());
            }
        }

        Ok(bottlenecks)
    }

    fn analyze_parallelization(&self, graph: &DiGraph<OperationNode, DependencyEdge>) -> Result<(usize, f32)> {
        // Simple analysis - count independent subgraphs
        let mut visited = HashSet::new();
        let mut group_count = 0;
        
        for node in graph.node_indices() {
            if !visited.contains(&node) {
                group_count += 1;
                // DFS to mark all connected nodes
                let mut stack = vec![node];
                while let Some(current) = stack.pop() {
                    if visited.insert(current) {
                        for edge in graph.edges(current) {
                            stack.push(edge.target());
                        }
                    }
                }
            }
        }

        // Calculate potential speedup (simplified)
        let sequential_time: u64 = graph.node_weights()
            .map(|n| n.estimated_duration_ms)
            .sum();
        
        let parallel_time = sequential_time / group_count.max(1) as u64;
        let speedup = sequential_time as f32 / parallel_time.max(1) as f32;

        Ok((group_count, speedup))
    }

    fn analyze_risk(
        &self,
        graph: &DiGraph<OperationNode, DependencyEdge>,
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<RiskSummary> {
        let mut high_risk_count = 0;
        let mut critical_dependencies = Vec::new();
        let mut mitigation_suggestions = Vec::new();

        for node in graph.node_weights() {
            if matches!(node.metadata.risk_level, RiskLevel::High | RiskLevel::Critical) {
                high_risk_count += 1;
            }
        }

        // Find critical dependencies (high-risk operations that others depend on)
        for (id, &idx) in node_map {
            let node = &graph[idx];
            if matches!(node.metadata.risk_level, RiskLevel::High | RiskLevel::Critical) {
                for edge in graph.edges_directed(idx, petgraph::Direction::Outgoing) {
                    let dependent_idx = edge.target();
                    let dependent = &graph[dependent_idx];
                    critical_dependencies.push((id.clone(), dependent.id.clone()));
                }
            }
        }

        // Generate mitigation suggestions
        if high_risk_count > 0 {
            mitigation_suggestions.push("Create backups before executing high-risk operations".to_string());
        }
        if !critical_dependencies.is_empty() {
            mitigation_suggestions.push("Review critical dependencies carefully".to_string());
        }
        if graph.edge_count() > graph.node_count() * 2 {
            mitigation_suggestions.push("Consider simplifying complex dependencies".to_string());
        }

        Ok(RiskSummary {
            high_risk_count,
            critical_dependencies,
            mitigation_suggestions,
        })
    }

    fn generate_execution_sequence(
        &self,
        graph: &DiGraph<OperationNode, DependencyEdge>,
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<Vec<String>> {
        // Use topological sort to determine execution order
        match toposort(graph, None) {
            Ok(sorted_indices) => {
                let mut sequence = Vec::new();
                for idx in sorted_indices {
                    if let Some(node) = graph.node_weight(idx) {
                        sequence.push(node.id.clone());
                    }
                }
                Ok(sequence)
            }
            Err(_) => {
                warn!("Graph has cycles, using alternative sequencing");
                // Fall back to simple ordering
                let mut sequence: Vec<_> = node_map.keys().cloned().collect();
                sequence.sort();
                Ok(sequence)
            }
        }
    }

    fn find_critical_path(
        &self,
        graph: &DiGraph<OperationNode, DependencyEdge>,
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<Vec<String>> {
        // Find the longest path through the graph (by execution time)
        let mut critical_path = Vec::new();
        let mut max_duration = 0;

        // Find all source nodes (no incoming edges)
        let sources: Vec<_> = graph.node_indices()
            .filter(|&node| graph.edges_directed(node, petgraph::Direction::Incoming).count() == 0)
            .collect();

        // Find all sink nodes (no outgoing edges)
        let sinks: Vec<_> = graph.node_indices()
            .filter(|&node| graph.edges_directed(node, petgraph::Direction::Outgoing).count() == 0)
            .collect();

        // Find longest path from each source to each sink
        for &source in &sources {
            for &sink in &sinks {
                if let Some(path) = self.find_longest_path(graph, source, sink)? {
                    let duration = self.calculate_path_duration(graph, &path)?;
                    if duration > max_duration {
                        max_duration = duration;
                        critical_path = path.into_iter()
                            .map(|idx| graph[idx].id.clone())
                            .collect();
                    }
                }
            }
        }

        Ok(critical_path)
    }

    fn find_longest_path(
        &self,
        graph: &DiGraph<OperationNode, DependencyEdge>,
        start: NodeIndex,
        end: NodeIndex,
    ) -> Result<Option<Vec<NodeIndex>>> {
        // Simple DFS-based longest path (works because DAG)
        let mut longest_path = None;
        let mut max_length = 0;

        // Use all_simple_paths from petgraph
        let paths: Vec<Vec<NodeIndex>> = all_simple_paths(graph, start, end, 0, None)
            .collect();

        for path in paths {
            let length = self.calculate_path_duration(graph, &path)?;
            if length > max_length {
                max_length = length;
                longest_path = Some(path);
            }
        }

        Ok(longest_path)
    }

    fn calculate_path_duration(&self, graph: &DiGraph<OperationNode, DependencyEdge>, path: &[NodeIndex]) -> Result<u64> {
        let duration = path.iter()
            .map(|&idx| graph[idx].estimated_duration_ms)
            .sum();
        Ok(duration)
    }

    fn identify_parallel_groups(
        &self,
        graph: &DiGraph<OperationNode, DependencyEdge>,
        node_map: &HashMap<String, NodeIndex>,
        execution_sequence: &[String],
    ) -> Result<Vec<ParallelGroup>> {
        let mut groups = Vec::new();
        let mut current_group = Vec::new();
        let mut group_dependencies: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut node_to_group: HashMap<String, usize> = HashMap::new();

        // Group operations that can run in parallel
        for op_id in execution_sequence {
            let idx = node_map[op_id];
            let node = &graph[idx];
            
            // Check if this operation depends on any in current group
            let depends_on_current = current_group.iter().any(|group_op: &String| {
                let group_idx = node_map[group_op];
                graph.contains_edge(idx, group_idx)
            });

            if depends_on_current || !node.parallelizable {
                // Start new group
                if !current_group.is_empty() {
                    let group_id = groups.len();
                    let parallel_duration = current_group.iter()
                        .map(|id| graph[node_map[id]].estimated_duration_ms)
                        .max()
                        .unwrap_or(0);

                    // Record which group each node belongs to
                    for op in &current_group {
                        node_to_group.insert(op.clone(), group_id);
                    }

                    groups.push(ParallelGroup {
                        id: group_id,
                        operations: current_group.clone(),
                        parallel_duration_ms: parallel_duration,
                        depends_on: vec![],
                    });
                    current_group.clear();
                }
            }
            
            current_group.push(op_id.clone());
        }

        // Don't forget the last group
        if !current_group.is_empty() {
            let group_id = groups.len();
            let parallel_duration = current_group.iter()
                .map(|id| graph[node_map[id]].estimated_duration_ms)
                .max()
                .unwrap_or(0);

            for op in &current_group {
                node_to_group.insert(op.clone(), group_id);
            }

            groups.push(ParallelGroup {
                id: group_id,
                operations: current_group,
                parallel_duration_ms: parallel_duration,
                depends_on: vec![],
            });
        }

        // Calculate group dependencies
        for (i, group) in groups.iter_mut().enumerate() {
            let mut depends_on = HashSet::new();
            
            for op_id in &group.operations {
                let idx = node_map[op_id];
                
                // Check all incoming edges
                for edge in graph.edges_directed(idx, petgraph::Direction::Incoming) {
                    let source_node = &graph[edge.source()];
                    if let Some(&source_group) = node_to_group.get(&source_node.id) {
                        if source_group != i {
                            depends_on.insert(source_group);
                        }
                    }
                }
            }
            
            group.depends_on = depends_on.into_iter().collect();
            group.depends_on.sort();
        }

        Ok(groups)
    }

    fn generate_visuals(&self, graph: &DiGraph<OperationNode, DependencyEdge>, node_map: &HashMap<String, NodeIndex>) -> Result<GraphVisuals> {
        // Generate DOT format
        let dot_graph = format!("{:?}", Dot::with_config(graph, &[Config::EdgeNoLabel]));
        
        // Generate ASCII art
        let ascii_graph = self.generate_ascii_graph(graph, node_map)?;
        
        // Generate Mermaid diagram
        let mermaid_diagram = self.generate_mermaid_diagram(graph, node_map)?;
        
        // Generate timeline chart
        let timeline_chart = self.generate_timeline_chart(graph, node_map)?;

        Ok(GraphVisuals {
            dot_graph,
            ascii_graph,
            mermaid_diagram,
            timeline_chart,
        })
    }

    fn generate_ascii_graph(&self, graph: &DiGraph<OperationNode, DependencyEdge>, node_map: &HashMap<String, NodeIndex>) -> Result<String> {
        let mut ascii = String::from("Operation Dependency Graph:\n");
        ascii.push_str("==========================\n\n");

        // Simple ASCII representation
        for (id, &idx) in node_map {
            let node = &graph[idx];
            let incoming = graph.edges_directed(idx, petgraph::Direction::Incoming).count();
            let outgoing = graph.edges_directed(idx, petgraph::Direction::Outgoing).count();
            
            ascii.push_str(&format!("[{}] {} ({})\n", 
                node.metadata.operation_type.chars().next().unwrap_or('?'),
                node.metadata.file_path.display(),
                node.id
            ));
            
            if outgoing > 0 {
                ascii.push_str("  └─> ");
                let deps: Vec<_> = graph.edges(idx)
                    .map(|e| &graph[e.target()].id)
                    .collect();
                ascii.push_str(&deps.join(", "));
                ascii.push('\n');
            }
            ascii.push('\n');
        }

        Ok(ascii)
    }

    fn generate_mermaid_diagram(&self, graph: &DiGraph<OperationNode, DependencyEdge>, node_map: &HashMap<String, NodeIndex>) -> Result<String> {
        let mut mermaid = String::from("graph TD\n");

        // Add nodes
        for (id, &idx) in node_map {
            let node = &graph[idx];
            let shape_start = match node.metadata.shape.as_str() {
                "box" => "[",
                "ellipse" => "(",
                "diamond" => "{",
                "parallelogram" => "[/",
                "hexagon" => "{{",
                _ => "[",
            };
            let shape_end = match node.metadata.shape.as_str() {
                "box" => "]",
                "ellipse" => ")",
                "diamond" => "}",
                "parallelogram" => "/]",
                "hexagon" => "}}",
                _ => "]",
            };
            
            mermaid.push_str(&format!("    {}{}{} {}{}\n",
                node.id,
                shape_start,
                node.metadata.operation_type,
                node.metadata.file_path.file_name().unwrap_or_default().to_string_lossy(),
                shape_end
            ));
        }

        // Add edges
        for edge in graph.edge_references() {
            let source = &graph[edge.source()];
            let target = &graph[edge.target()];
            let edge_data = edge.weight();
            
            mermaid.push_str(&format!("    {} -->|{}| {}\n",
                source.id,
                edge_data.label,
                target.id
            ));
        }

        // Add styling
        mermaid.push_str("\n    classDef create fill:#4CAF50,stroke:#333,stroke-width:2px\n");
        mermaid.push_str("    classDef update fill:#2196F3,stroke:#333,stroke-width:2px\n");
        mermaid.push_str("    classDef delete fill:#F44336,stroke:#333,stroke-width:2px\n");

        Ok(mermaid)
    }

    fn generate_timeline_chart(&self, graph: &DiGraph<OperationNode, DependencyEdge>, node_map: &HashMap<String, NodeIndex>) -> Result<String> {
        let mut timeline = String::from("Execution Timeline:\n");
        timeline.push_str("==================\n\n");

        let sequence = self.generate_execution_sequence(graph, node_map)?;
        let mut current_time = 0u64;

        for (i, op_id) in sequence.iter().enumerate() {
            let idx = node_map[op_id];
            let node = &graph[idx];
            
            timeline.push_str(&format!("{:>4}. [{:>6}ms] {} {}\n",
                i + 1,
                current_time,
                node.metadata.operation_type,
                node.metadata.file_path.display()
            ));
            
            current_time += node.estimated_duration_ms;
        }

        timeline.push_str(&format!("\nTotal estimated time: {}ms\n", current_time));

        Ok(timeline)
    }

    fn get_operation_path(&self, operation: &FileOperation) -> PathBuf {
        match operation {
            FileOperation::Create { path, .. } |
            FileOperation::Update { path, .. } |
            FileOperation::Delete { path } => path.clone(),
            FileOperation::Rename { new_path, .. } => new_path.clone(),
        }
    }

    fn assess_operation_risk(&self, operation: &FileOperation) -> RiskLevel {
        match operation {
            FileOperation::Delete { .. } => RiskLevel::High,
            FileOperation::Update { path, .. } => {
                if path.to_string_lossy().contains("config") ||
                   path.to_string_lossy().contains("settings") {
                    RiskLevel::Medium
                } else {
                    RiskLevel::Low
                }
            }
            FileOperation::Create { .. } => RiskLevel::Low,
            FileOperation::Rename { .. } => RiskLevel::Medium,
        }
    }

    fn estimate_duration(&self, operation: &FileOperation) -> u64 {
        match operation {
            FileOperation::Create { content, .. } => {
                10 + (content.len() as u64 / 1000)
            }
            FileOperation::Update { .. } => 15,
            FileOperation::Delete { .. } => 5,
            FileOperation::Rename { .. } => 10,
        }
    }

    fn generate_cache_key(&self, operations: &[EnhancedFileOperation]) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        for op in operations {
            format!("{:?}", op.operation).hash(&mut hasher);
        }
        
        format!("graph_{:x}", hasher.finish())
    }

    pub async fn clear_cache(&self) {
        self.graph_cache.write().await.clear();
        debug!("Dependency graph cache cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dependency_graph_generation() {
        let generator = DependencyGraphGenerator::new(None, None);
        
        // Create test operations with dependencies
        let operations = vec![
            EnhancedFileOperation {
                operation: FileOperation::Create {
                    path: PathBuf::from("module.rs"),
                    content: "pub fn hello() {}".to_string(),
                },
                context: Default::default(),
                parsing_confidence: 0.9,
            },
            EnhancedFileOperation {
                operation: FileOperation::Create {
                    path: PathBuf::from("main.rs"),
                    content: "mod module;\nfn main() { module::hello(); }".to_string(),
                },
                context: crate::consensus::operation_parser::OperationContext {
                    dependencies: vec![0], // Depends on first operation
                    ..Default::default()
                },
                parsing_confidence: 0.9,
            },
        ];

        let graph = generator.generate_dependency_graph(&operations, None).await.unwrap();
        
        assert_eq!(graph.node_map.len(), 2);
        assert_eq!(graph.execution_sequence.len(), 2);
        assert!(!graph.analysis.has_cycles);
        assert_eq!(graph.execution_sequence[0], "op_0"); // module.rs should come first
        assert_eq!(graph.execution_sequence[1], "op_1"); // main.rs depends on it
    }

    #[test]
    fn test_risk_assessment() {
        let generator = DependencyGraphGenerator::new(None, None);
        
        let delete_op = FileOperation::Delete {
            path: PathBuf::from("important.rs"),
        };
        assert_eq!(generator.assess_operation_risk(&delete_op), RiskLevel::High);
        
        let create_op = FileOperation::Create {
            path: PathBuf::from("new.rs"),
            content: String::new(),
        };
        assert_eq!(generator.assess_operation_risk(&create_op), RiskLevel::Low);
    }
}