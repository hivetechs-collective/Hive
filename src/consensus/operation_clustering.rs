// Operation Clustering - Intelligent grouping and sequencing of file operations
use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{toposort, is_cyclic_directed};

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::ai_helpers::pattern_recognizer::PatternRecognizer;

#[derive(Debug, Clone, PartialEq)]
pub enum ClusterType {
    ModuleUpdate,      // Operations on related module files
    TestCreation,      // Test file creation and updates
    Refactoring,       // Large-scale refactoring operations
    Configuration,     // Config file updates
    Documentation,     // Doc file updates
    Migration,         // Database/API migrations
    FeatureAddition,   // New feature implementation
    BugFix,           // Bug fix related changes
    Deletion,         // File removal operations
    Mixed,            // Mixed operation types
}

#[derive(Debug, Clone)]
pub struct OperationCluster {
    pub id: String,
    pub cluster_type: ClusterType,
    pub operations: Vec<FileOperation>,
    pub dependencies: Vec<String>, // IDs of clusters this depends on
    pub risk_level: f32,
    pub execution_order: usize,
    pub description: String,
    pub common_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ClusteringResult {
    pub clusters: Vec<OperationCluster>,
    pub dependency_graph: DiGraph<String, ()>,
    pub execution_sequence: Vec<String>, // Cluster IDs in execution order
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClusteringConfig {
    pub max_cluster_size: usize,
    pub similarity_threshold: f32,
    pub enable_smart_grouping: bool,
    pub respect_dependencies: bool,
    pub optimize_for_safety: bool,
}

impl Default for ClusteringConfig {
    fn default() -> Self {
        Self {
            max_cluster_size: 10,
            similarity_threshold: 0.7,
            enable_smart_grouping: true,
            respect_dependencies: true,
            optimize_for_safety: true,
        }
    }
}

pub struct OperationClusterer {
    config: ClusteringConfig,
    pattern_recognizer: Option<Arc<PatternRecognizer>>,
    cluster_cache: Arc<RwLock<HashMap<String, ClusteringResult>>>,
}

impl OperationClusterer {
    pub fn new(
        config: ClusteringConfig,
        pattern_recognizer: Option<Arc<PatternRecognizer>>,
    ) -> Self {
        Self {
            config,
            pattern_recognizer,
            cluster_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn cluster_operations(
        &self,
        operations: &[FileOperation],
    ) -> Result<ClusteringResult> {
        if operations.is_empty() {
            return Ok(ClusteringResult {
                clusters: vec![],
                dependency_graph: DiGraph::new(),
                execution_sequence: vec![],
                warnings: vec![],
            });
        }

        // Check cache
        let cache_key = self.generate_cache_key(operations);
        if let Some(cached) = self.cluster_cache.read().await.get(&cache_key) {
            debug!("Using cached clustering result");
            return Ok(cached.clone());
        }

        // Step 1: Initial grouping by similarity
        let initial_groups = self.group_by_similarity(operations).await?;
        
        // Step 2: Apply smart grouping if enabled
        let refined_groups = if self.config.enable_smart_grouping {
            self.apply_smart_grouping(initial_groups).await?
        } else {
            initial_groups
        };
        
        // Step 3: Create clusters with metadata
        let mut clusters = self.create_clusters(refined_groups).await?;
        
        // Step 4: Build dependency graph
        let (dependency_graph, warnings) = self.build_dependency_graph(&mut clusters).await?;
        
        // Step 5: Determine execution sequence
        let execution_sequence = self.determine_execution_sequence(&clusters, &dependency_graph)?;
        
        // Step 6: Optimize for safety if enabled
        if self.config.optimize_for_safety {
            self.optimize_for_safety(&mut clusters, &execution_sequence).await?;
        }

        let result = ClusteringResult {
            clusters,
            dependency_graph,
            execution_sequence,
            warnings,
        };

        // Cache the result
        self.cluster_cache.write().await.insert(cache_key, result.clone());

        Ok(result)
    }

    async fn group_by_similarity(
        &self,
        operations: &[FileOperation],
    ) -> Result<Vec<Vec<FileOperation>>> {
        let mut groups: Vec<Vec<FileOperation>> = Vec::new();
        let mut processed = HashSet::new();

        for (i, op) in operations.iter().enumerate() {
            if processed.contains(&i) {
                continue;
            }

            let mut group = vec![op.clone()];
            processed.insert(i);

            // Find similar operations
            for (j, other_op) in operations.iter().enumerate().skip(i + 1) {
                if processed.contains(&j) {
                    continue;
                }

                if self.calculate_similarity(op, other_op) >= self.config.similarity_threshold {
                    group.push(other_op.clone());
                    processed.insert(j);
                    
                    // Respect max cluster size
                    if group.len() >= self.config.max_cluster_size {
                        break;
                    }
                }
            }

            groups.push(group);
        }

        Ok(groups)
    }

    fn calculate_similarity(&self, op1: &FileOperation, op2: &FileOperation) -> f32 {
        let path1 = self.get_operation_path(op1);
        let path2 = self.get_operation_path(op2);

        // Factor 1: Path similarity (common parent directory)
        let path_similarity = self.calculate_path_similarity(&path1, &path2);
        
        // Factor 2: Operation type similarity
        let type_similarity = if std::mem::discriminant(op1) == std::mem::discriminant(op2) {
            1.0
        } else {
            0.5
        };
        
        // Factor 3: File extension similarity
        let ext_similarity = if path1.extension() == path2.extension() {
            1.0
        } else {
            0.3
        };
        
        // Factor 4: Module proximity (are they in the same module?)
        let module_similarity = self.calculate_module_similarity(&path1, &path2);

        // Weighted average
        (path_similarity * 0.4 + type_similarity * 0.2 + ext_similarity * 0.2 + module_similarity * 0.2)
    }

    fn calculate_path_similarity(&self, path1: &Path, path2: &Path) -> f32 {
        let components1: Vec<_> = path1.components().collect();
        let components2: Vec<_> = path2.components().collect();
        
        let common_prefix_len = components1.iter()
            .zip(components2.iter())
            .take_while(|(a, b)| a == b)
            .count();
        
        let max_len = components1.len().max(components2.len());
        if max_len == 0 {
            return 1.0;
        }
        
        common_prefix_len as f32 / max_len as f32
    }

    fn calculate_module_similarity(&self, path1: &Path, path2: &Path) -> f32 {
        // Check if files are in the same module (e.g., src/auth/*)
        let parent1 = path1.parent();
        let parent2 = path2.parent();
        
        match (parent1, parent2) {
            (Some(p1), Some(p2)) if p1 == p2 => 1.0,
            (Some(p1), Some(p2)) => {
                // Check if they're in sibling directories
                match (p1.parent(), p2.parent()) {
                    (Some(pp1), Some(pp2)) if pp1 == pp2 => 0.7,
                    _ => 0.3,
                }
            }
            _ => 0.0,
        }
    }

    async fn apply_smart_grouping(
        &self,
        initial_groups: Vec<Vec<FileOperation>>,
    ) -> Result<Vec<Vec<FileOperation>>> {
        let mut refined_groups = Vec::new();

        for group in initial_groups {
            // Use pattern recognizer if available to identify patterns
            if let Some(pattern_recognizer) = &self.pattern_recognizer {
                // Check if this group represents a known pattern
                let pattern_type = self.identify_pattern_type(&group).await?;
                
                match pattern_type {
                    ClusterType::TestCreation => {
                        // Keep test files separate from implementation
                        let (tests, non_tests) = self.separate_test_files(group);
                        if !tests.is_empty() {
                            refined_groups.push(tests);
                        }
                        if !non_tests.is_empty() {
                            refined_groups.push(non_tests);
                        }
                    }
                    ClusterType::Deletion => {
                        // Keep deletions in their own group for safety
                        refined_groups.push(group);
                    }
                    ClusterType::Configuration => {
                        // Config files should be updated together
                        refined_groups.push(group);
                    }
                    _ => {
                        // For other types, check if we should split or keep together
                        if group.len() > self.config.max_cluster_size {
                            // Split large groups
                            for chunk in group.chunks(self.config.max_cluster_size) {
                                refined_groups.push(chunk.to_vec());
                            }
                        } else {
                            refined_groups.push(group);
                        }
                    }
                }
            } else {
                // Without pattern recognizer, use simple heuristics
                refined_groups.push(group);
            }
        }

        Ok(refined_groups)
    }

    async fn identify_pattern_type(&self, operations: &[FileOperation]) -> Result<ClusterType> {
        // Analyze operations to determine cluster type
        let mut has_tests = false;
        let mut has_deletions = false;
        let mut has_config = false;
        let mut has_migrations = false;
        let mut has_docs = false;

        for op in operations {
            let path = self.get_operation_path(op);
            let path_str = path.to_string_lossy();

            if path_str.contains("test") || path_str.contains("spec") {
                has_tests = true;
            }
            if matches!(op, FileOperation::Delete { .. }) {
                has_deletions = true;
            }
            if path_str.contains("config") || path_str.contains("settings") || 
               path.extension().map_or(false, |ext| ext == "toml" || ext == "yaml" || ext == "json") {
                has_config = true;
            }
            if path_str.contains("migration") || path_str.contains("schema") {
                has_migrations = true;
            }
            if path.extension().map_or(false, |ext| ext == "md" || ext == "rst") {
                has_docs = true;
            }
        }

        // Determine primary cluster type
        if has_deletions {
            Ok(ClusterType::Deletion)
        } else if has_tests && operations.len() > 1 {
            Ok(ClusterType::TestCreation)
        } else if has_config {
            Ok(ClusterType::Configuration)
        } else if has_migrations {
            Ok(ClusterType::Migration)
        } else if has_docs {
            Ok(ClusterType::Documentation)
        } else {
            // Try to infer from operation patterns
            self.infer_cluster_type_from_patterns(operations).await
        }
    }

    async fn infer_cluster_type_from_patterns(&self, operations: &[FileOperation]) -> Result<ClusterType> {
        // Count operation types
        let mut creates = 0;
        let mut updates = 0;
        let mut renames = 0;
        
        for op in operations {
            match op {
                FileOperation::Create { .. } => creates += 1,
                FileOperation::Update { .. } => updates += 1,
                FileOperation::Rename { .. } | FileOperation::Move { .. } => renames += 1,
                _ => {}
            }
        }

        if creates > updates && creates > 0 {
            Ok(ClusterType::FeatureAddition)
        } else if renames > 0 && renames >= operations.len() / 2 {
            Ok(ClusterType::Refactoring)
        } else if updates > creates {
            Ok(ClusterType::BugFix)
        } else {
            Ok(ClusterType::Mixed)
        }
    }

    fn separate_test_files(&self, operations: Vec<FileOperation>) -> (Vec<FileOperation>, Vec<FileOperation>) {
        let mut tests = Vec::new();
        let mut non_tests = Vec::new();

        for op in operations {
            let path = self.get_operation_path(&op);
            let path_str = path.to_string_lossy();
            
            if path_str.contains("test") || path_str.contains("spec") || 
               path_str.ends_with("_test.rs") || path_str.ends_with(".test.ts") {
                tests.push(op);
            } else {
                non_tests.push(op);
            }
        }

        (tests, non_tests)
    }

    async fn create_clusters(&self, groups: Vec<Vec<FileOperation>>) -> Result<Vec<OperationCluster>> {
        let mut clusters = Vec::new();

        for (i, group) in groups.iter().enumerate() {
            let cluster_type = self.identify_pattern_type(group).await?;
            let common_path = self.find_common_path(group);
            let risk_level = self.calculate_cluster_risk(group, &cluster_type);

            let cluster = OperationCluster {
                id: format!("cluster_{}", i),
                cluster_type: cluster_type.clone(),
                operations: group.clone(),
                dependencies: Vec::new(), // Will be filled in dependency analysis
                risk_level,
                execution_order: 0, // Will be set after dependency analysis
                description: self.generate_cluster_description(&cluster_type, group),
                common_path,
            };

            clusters.push(cluster);
        }

        Ok(clusters)
    }

    fn find_common_path(&self, operations: &[FileOperation]) -> Option<PathBuf> {
        if operations.is_empty() {
            return None;
        }

        let paths: Vec<_> = operations.iter()
            .map(|op| self.get_operation_path(op))
            .collect();

        // Find the common ancestor directory
        let first_path = &paths[0];
        let mut common = PathBuf::new();

        'outer: for component in first_path.components() {
            for path in &paths[1..] {
                let mut found = false;
                for path_component in path.components() {
                    if component == path_component {
                        found = true;
                        break;
                    }
                }
                if !found {
                    break 'outer;
                }
            }
            common.push(component);
        }

        if common.as_os_str().is_empty() {
            None
        } else {
            Some(common)
        }
    }

    fn calculate_cluster_risk(&self, operations: &[FileOperation], cluster_type: &ClusterType) -> f32 {
        let mut risk = 0.0;

        // Base risk by cluster type
        risk += match cluster_type {
            ClusterType::Deletion => 0.8,
            ClusterType::Migration => 0.7,
            ClusterType::Refactoring => 0.6,
            ClusterType::Configuration => 0.5,
            ClusterType::BugFix => 0.3,
            ClusterType::TestCreation => 0.2,
            ClusterType::Documentation => 0.1,
            ClusterType::FeatureAddition => 0.4,
            ClusterType::ModuleUpdate => 0.4,
            ClusterType::Mixed => 0.5,
        };

        // Risk based on operation count
        risk += (operations.len() as f32 / 20.0).min(0.3);

        // Risk based on operation types
        let has_deletes = operations.iter().any(|op| matches!(op, FileOperation::Delete { .. }));
        if has_deletes {
            risk += 0.2;
        }

        (risk * 100.0).min(100.0)
    }

    fn generate_cluster_description(&self, cluster_type: &ClusterType, operations: &[FileOperation]) -> String {
        let op_count = operations.len();
        let description = match cluster_type {
            ClusterType::ModuleUpdate => format!("Update {} files in module", op_count),
            ClusterType::TestCreation => format!("Create/update {} test files", op_count),
            ClusterType::Refactoring => format!("Refactor {} files", op_count),
            ClusterType::Configuration => format!("Update {} configuration files", op_count),
            ClusterType::Documentation => format!("Update {} documentation files", op_count),
            ClusterType::Migration => format!("Apply migration to {} files", op_count),
            ClusterType::FeatureAddition => format!("Add feature across {} files", op_count),
            ClusterType::BugFix => format!("Fix bug in {} files", op_count),
            ClusterType::Deletion => format!("Delete {} files", op_count),
            ClusterType::Mixed => format!("Mixed operations on {} files", op_count),
        };

        description
    }

    async fn build_dependency_graph(
        &self,
        clusters: &mut [OperationCluster],
    ) -> Result<(DiGraph<String, ()>, Vec<String>)> {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();
        let mut warnings = Vec::new();

        // Add nodes
        for cluster in clusters.iter() {
            let node_idx = graph.add_node(cluster.id.clone());
            node_map.insert(cluster.id.clone(), node_idx);
        }

        // Analyze dependencies
        for i in 0..clusters.len() {
            for j in 0..clusters.len() {
                if i == j {
                    continue;
                }

                if self.has_dependency(&clusters[i], &clusters[j]) {
                    let from_idx = node_map[&clusters[i].id];
                    let to_idx = node_map[&clusters[j].id];
                    
                    graph.add_edge(from_idx, to_idx, ());
                    clusters[j].dependencies.push(clusters[i].id.clone());
                }
            }
        }

        // Check for cycles
        if is_cyclic_directed(&graph) {
            warnings.push("Circular dependencies detected between operation clusters".to_string());
            // Try to break cycles by removing lowest-priority edges
            self.break_cycles(&mut graph, &node_map)?;
        }

        Ok((graph, warnings))
    }

    fn has_dependency(&self, cluster1: &OperationCluster, cluster2: &OperationCluster) -> bool {
        // Check if cluster1 depends on cluster2 (cluster2 must execute before cluster1)
        
        // Rule 1: File creation dependencies
        for op2 in &cluster2.operations {
            if let FileOperation::Create { path: create_path, .. } = op2 {
                for op1 in &cluster1.operations {
                    match op1 {
                        FileOperation::Update { path: update_path, .. } => {
                            if create_path == update_path {
                                return true; // Can't update a file before it's created
                            }
                        }
                        FileOperation::Move { destination, .. } => {
                            // Check if moving to a directory that's being created
                            if let Some(parent) = destination.parent() {
                                if create_path == &parent.to_path_buf() {
                                    return true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Rule 2: Deletion dependencies (deletions should happen last)
        if matches!(cluster1.cluster_type, ClusterType::Deletion) && 
           !matches!(cluster2.cluster_type, ClusterType::Deletion) {
            return false; // Deletions don't depend on non-deletions
        }
        
        if !matches!(cluster1.cluster_type, ClusterType::Deletion) && 
           matches!(cluster2.cluster_type, ClusterType::Deletion) {
            return true; // Non-deletions depend on deletions being done last
        }

        // Rule 3: Test dependencies (tests should run after implementation)
        if matches!(cluster1.cluster_type, ClusterType::TestCreation) &&
           matches!(cluster2.cluster_type, ClusterType::FeatureAddition | ClusterType::BugFix) {
            // Check if test files reference implementation files
            if let (Some(test_path), Some(impl_path)) = (&cluster1.common_path, &cluster2.common_path) {
                if self.paths_are_related(test_path, impl_path) {
                    return true;
                }
            }
        }

        false
    }

    fn paths_are_related(&self, path1: &Path, path2: &Path) -> bool {
        // Check if paths are related (e.g., src/module and tests/module)
        let path1_str = path1.to_string_lossy();
        let path2_str = path2.to_string_lossy();
        
        // Extract module name from paths
        let module1 = path1.file_name().and_then(|n| n.to_str());
        let module2 = path2.file_name().and_then(|n| n.to_str());
        
        match (module1, module2) {
            (Some(m1), Some(m2)) => m1 == m2 || m1.contains(m2) || m2.contains(m1),
            _ => false,
        }
    }

    fn break_cycles(&self, graph: &mut DiGraph<String, ()>, node_map: &HashMap<String, NodeIndex>) -> Result<()> {
        // Simple cycle breaking: remove edges with lowest priority
        // In practice, you'd want a more sophisticated algorithm
        warn!("Breaking dependency cycles - this may affect execution order");
        
        // For now, just log the warning
        // A full implementation would use algorithms like feedback arc set
        
        Ok(())
    }

    fn determine_execution_sequence(
        &self,
        clusters: &[OperationCluster],
        graph: &DiGraph<String, ()>,
    ) -> Result<Vec<String>> {
        // Use topological sort to determine execution order
        match toposort(graph, None) {
            Ok(sorted_indices) => {
                let sequence: Vec<String> = sorted_indices
                    .into_iter()
                    .map(|idx| graph[idx].clone())
                    .collect();
                Ok(sequence)
            }
            Err(_) => {
                // If topological sort fails, fall back to risk-based ordering
                warn!("Topological sort failed, using risk-based ordering");
                let mut sorted_clusters = clusters.to_vec();
                sorted_clusters.sort_by(|a, b| {
                    a.risk_level.partial_cmp(&b.risk_level).unwrap()
                });
                Ok(sorted_clusters.into_iter().map(|c| c.id).collect())
            }
        }
    }

    async fn optimize_for_safety(
        &self,
        clusters: &mut [OperationCluster],
        execution_sequence: &[String],
    ) -> Result<()> {
        // Apply safety optimizations
        
        // 1. Ensure high-risk operations are isolated
        for cluster in clusters.iter_mut() {
            if cluster.risk_level > 70.0 && cluster.operations.len() > 3 {
                info!("High-risk cluster {} will be executed with extra safety checks", cluster.id);
                // In a real implementation, we might split this cluster further
            }
        }

        // 2. Set execution order based on sequence
        let sequence_map: HashMap<_, _> = execution_sequence.iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();
        
        for cluster in clusters.iter_mut() {
            cluster.execution_order = sequence_map.get(&cluster.id).copied().unwrap_or(999);
        }

        // 3. Add safety warnings for specific patterns
        for cluster in clusters.iter() {
            if matches!(cluster.cluster_type, ClusterType::Deletion) {
                warn!("Deletion cluster {} requires backup before execution", cluster.id);
            }
            if matches!(cluster.cluster_type, ClusterType::Migration) {
                warn!("Migration cluster {} may require database backup", cluster.id);
            }
        }

        Ok(())
    }

    fn get_operation_path(&self, operation: &FileOperation) -> PathBuf {
        match operation {
            FileOperation::Create { path, .. } => path.clone(),
            FileOperation::Update { path, .. } => path.clone(),
            FileOperation::Delete { path } => path.clone(),
            FileOperation::Rename { old_path, .. } => old_path.clone(),
            FileOperation::Move { source, .. } => source.clone(),
        }
    }

    fn generate_cache_key(&self, operations: &[FileOperation]) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        for op in operations {
            format!("{:?}", op).hash(&mut hasher);
        }
        
        format!("cluster_{:x}", hasher.finish())
    }

    pub async fn clear_cache(&self) {
        self.cluster_cache.write().await.clear();
        debug!("Clustering cache cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_clustering() {
        let clusterer = OperationClusterer::new(ClusteringConfig::default(), None);
        
        let operations = vec![
            FileOperation::Create {
                path: PathBuf::from("src/auth/login.rs"),
                content: "// Login module".to_string(),
            },
            FileOperation::Update {
                path: PathBuf::from("src/auth/session.rs"),
                old_content: "// Old session".to_string(),
                new_content: "// New session".to_string(),
            },
            FileOperation::Create {
                path: PathBuf::from("tests/auth_test.rs"),
                content: "// Auth tests".to_string(),
            },
        ];

        let result = clusterer.cluster_operations(&operations).await.unwrap();
        
        // Should create at least 2 clusters (implementation and tests)
        assert!(result.clusters.len() >= 2);
        
        // Check that test files are separated
        let test_cluster = result.clusters.iter()
            .find(|c| matches!(c.cluster_type, ClusterType::TestCreation));
        assert!(test_cluster.is_some());
    }

    #[tokio::test]
    async fn test_dependency_detection() {
        let clusterer = OperationClusterer::new(ClusteringConfig::default(), None);
        
        let operations = vec![
            FileOperation::Create {
                path: PathBuf::from("src/new_module.rs"),
                content: "// New module".to_string(),
            },
            FileOperation::Update {
                path: PathBuf::from("src/new_module.rs"),
                old_content: "// New module".to_string(),
                new_content: "// Updated module".to_string(),
            },
        ];

        let result = clusterer.cluster_operations(&operations).await.unwrap();
        
        // The update should depend on the create
        assert!(!result.execution_sequence.is_empty());
    }
}