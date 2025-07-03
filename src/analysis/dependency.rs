//! Dependency analysis engine for project understanding
//!
//! This module provides comprehensive dependency analysis including:
//! - Module dependency graphs
//! - Circular dependency detection
//! - Dependency metrics and visualization

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::{Result, Context};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{toposort, is_cyclic_directed, tarjan_scc, has_path_connecting};
use petgraph::dot::{Dot, Config};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use crate::core::ast::{ImportInfo, ParseResult};
use crate::analysis::parser::TreeSitterParser;
use crate::core::Language;

/// Dependency graph for tracking module relationships
pub struct DependencyGraph {
    /// Petgraph directed graph
    graph: DiGraph<ModuleNode, DependencyEdge>,
    /// Module path to node index mapping
    module_to_node: HashMap<PathBuf, NodeIndex>,
    /// Statistics
    stats: DependencyStats,
}

/// Module node in dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleNode {
    /// Module path
    pub path: PathBuf,
    /// Module name
    pub name: String,
    /// Language
    pub language: Language,
    /// Lines of code
    pub loc: usize,
    /// Number of exports
    pub export_count: usize,
    /// Complexity score
    pub complexity: u32,
    /// Is external dependency
    pub is_external: bool,
}

/// Dependency edge between modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Import kind (static, dynamic, etc.)
    pub kind: DependencyKind,
    /// Imported items
    pub items: Vec<String>,
    /// Is circular
    pub is_circular: bool,
    /// Dependency strength (0-1)
    pub strength: f32,
}

/// Types of dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyKind {
    Static,      // Regular imports
    Dynamic,     // Dynamic imports
    Type,        // Type-only imports
    Dev,         // Development dependencies
    Optional,    // Optional dependencies
}

/// Dependency analysis statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DependencyStats {
    pub total_modules: usize,
    pub total_dependencies: usize,
    pub external_dependencies: usize,
    pub circular_dependencies: Vec<Vec<PathBuf>>,
    pub max_depth: usize,
    pub avg_dependencies_per_module: f32,
    pub most_depended_on: Vec<(PathBuf, usize)>,
    pub most_dependent: Vec<(PathBuf, usize)>,
}

/// Dependency analyzer
pub struct DependencyAnalyzer {
    /// Dependency graph
    graph: Arc<RwLock<DependencyGraph>>,
    /// Parser registry
    parsers: Arc<RwLock<HashMap<Language, Arc<tokio::sync::Mutex<TreeSitterParser>>>>>,
    /// Module cache
    module_cache: Arc<RwLock<HashMap<PathBuf, ModuleInfo>>>,
}

/// Module information cache
#[derive(Debug, Clone)]
struct ModuleInfo {
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<String>,
    pub loc: usize,
    pub complexity: u32,
}

/// Dependency analysis result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependencyAnalysis {
    pub stats: DependencyStats,
    pub circular_dependencies: Vec<CircularDependency>,
    pub dependency_layers: Vec<DependencyLayer>,
    pub suggested_refactorings: Vec<RefactoringSuggestion>,
    pub visualization_dot: String,
}

/// Circular dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub modules: Vec<PathBuf>,
    pub edges: Vec<(PathBuf, PathBuf)>,
    pub severity: Severity,
    pub breaking_points: Vec<PathBuf>,
}

/// Dependency layer (for layered architecture)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyLayer {
    pub level: usize,
    pub modules: Vec<PathBuf>,
    pub can_depend_on: Vec<usize>,
}

/// Refactoring suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    pub kind: RefactoringKind,
    pub target: PathBuf,
    pub reason: String,
    pub impact: Impact,
}

/// Types of refactoring suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefactoringKind {
    ExtractInterface,
    InvertDependency,
    ExtractModule,
    MergeModules,
    RemoveUnused,
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Impact levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Impact {
    Minimal,
    Moderate,
    Significant,
    Major,
}

impl DependencyAnalyzer {
    /// Create a new dependency analyzer
    pub async fn new() -> Result<Self> {
        Ok(Self {
            graph: Arc::new(RwLock::new(DependencyGraph::new())),
            parsers: Arc::new(RwLock::new(HashMap::new())),
            module_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Analyze dependencies for a project
    #[instrument(skip(self))]
    pub async fn analyze_project(&self, root_path: &Path) -> Result<DependencyAnalysis> {
        info!("Analyzing dependencies for {}", root_path.display());
        
        // Clear existing graph
        {
            let mut graph = self.graph.write().await;
            *graph = DependencyGraph::new();
        }
        
        // Discover all modules
        let modules = self.discover_modules(root_path).await?;
        info!("Found {} modules", modules.len());
        
        // Analyze each module
        for module_path in &modules {
            if let Err(e) = self.analyze_module(module_path).await {
                debug!("Failed to analyze {}: {}", module_path.display(), e);
            }
        }
        
        // Build dependency graph
        self.build_dependency_graph().await?;
        
        // Analyze graph
        let analysis = self.perform_analysis().await?;
        
        Ok(analysis)
    }

    /// Discover all modules in a project
    async fn discover_modules(&self, root_path: &Path) -> Result<Vec<PathBuf>> {
        let mut modules = Vec::new();
        let mut stack = vec![root_path.to_path_buf()];
        
        while let Some(dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                        
                    // Skip common non-source directories
                    if !matches!(name, ".git" | "target" | "node_modules" | ".venv" | "__pycache__" | "dist" | "build") {
                        stack.push(path);
                    }
                } else if self.is_source_file(&path) {
                    modules.push(path);
                }
            }
        }
        
        Ok(modules)
    }

    /// Check if a file is a source file
    fn is_source_file(&self, path: &Path) -> bool {
        matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "cpp" | "c" | "h" | "hpp")
        )
    }

    /// Analyze a single module
    async fn analyze_module(&self, module_path: &Path) -> Result<()> {
        // Read file content
        let content = tokio::fs::read_to_string(module_path).await?;
        
        // Detect language
        let language = self.detect_language(module_path)?;
        
        // Get parser
        let parser = self.get_parser(language).await?;
        let mut parser = parser.lock().await;
        
        // Parse module
        let parse_result = parser.parse(&content)?;
        
        // Extract module info
        let exports = self.extract_exports(&parse_result);
        let complexity = self.calculate_complexity(&parse_result);
        let module_info = ModuleInfo {
            imports: parse_result.imports,
            exports,
            loc: content.lines().count(),
            complexity,
        };
        
        // Cache module info
        {
            let mut cache = self.module_cache.write().await;
            cache.insert(module_path.to_path_buf(), module_info);
        }
        
        Ok(())
    }

    /// Build dependency graph from analyzed modules
    async fn build_dependency_graph(&self) -> Result<()> {
        let cache = self.module_cache.read().await;
        let mut graph = self.graph.write().await;
        
        // Add nodes for all modules
        for (path, info) in cache.iter() {
            let node = ModuleNode {
                path: path.clone(),
                name: path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                language: self.detect_language(path)?,
                loc: info.loc,
                export_count: info.exports.len(),
                complexity: info.complexity,
                is_external: false,
            };
            
            graph.add_module(node);
        }
        
        // Add edges for dependencies
        for (from_path, info) in cache.iter() {
            for import in &info.imports {
                // Resolve import to module path
                if let Some(to_path) = self.resolve_import(from_path, &import.module).await {
                    let strength = self.calculate_dependency_strength(info, &import);
                    
                    let edge = DependencyEdge {
                        kind: self.determine_dependency_kind(&import),
                        items: import.items.clone(),
                        is_circular: false, // Will be detected later
                        strength,
                    };
                    
                    graph.add_dependency(from_path, &to_path, edge)?;
                } else if !self.is_standard_library(&import.module) {
                    // External dependency
                    let external_node = ModuleNode {
                        path: PathBuf::from(&import.module),
                        name: import.module.clone(),
                        language: Language::Unknown,
                        loc: 0,
                        export_count: 0,
                        complexity: 0,
                        is_external: true,
                    };
                    
                    graph.add_module(external_node);
                    
                    let edge = DependencyEdge {
                        kind: DependencyKind::Static,
                        items: import.items.clone(),
                        is_circular: false,
                        strength: 0.5,
                    };
                    
                    graph.add_dependency(from_path, &PathBuf::from(&import.module), edge)?;
                }
            }
        }
        
        // Detect circular dependencies
        graph.detect_circular_dependencies();
        
        // Update statistics
        graph.update_stats();
        
        Ok(())
    }

    /// Perform comprehensive dependency analysis
    async fn perform_analysis(&self) -> Result<DependencyAnalysis> {
        let graph = self.graph.read().await;
        
        // Find circular dependencies
        let circular_dependencies = self.analyze_circular_dependencies(&graph)?;
        
        // Calculate dependency layers
        let dependency_layers = self.calculate_dependency_layers(&graph)?;
        
        // Generate refactoring suggestions
        let suggested_refactorings = self.suggest_refactorings(&graph, &circular_dependencies)?;
        
        // Generate visualization
        let visualization_dot = self.generate_visualization(&graph)?;
        
        Ok(DependencyAnalysis {
            stats: graph.stats.clone(),
            circular_dependencies,
            dependency_layers,
            suggested_refactorings,
            visualization_dot,
        })
    }

    /// Analyze circular dependencies
    fn analyze_circular_dependencies(&self, graph: &DependencyGraph) -> Result<Vec<CircularDependency>> {
        let mut circular_deps = Vec::new();
        
        for cycle in &graph.stats.circular_dependencies {
            let edges = self.get_cycle_edges(graph, cycle);
            let severity = self.calculate_cycle_severity(cycle.len(), &edges);
            let breaking_points = self.find_breaking_points(graph, cycle);
            
            circular_deps.push(CircularDependency {
                modules: cycle.clone(),
                edges,
                severity,
                breaking_points,
            });
        }
        
        Ok(circular_deps)
    }

    /// Calculate dependency layers
    fn calculate_dependency_layers(&self, graph: &DependencyGraph) -> Result<Vec<DependencyLayer>> {
        let mut layers = Vec::new();
        let mut processed = HashSet::new();
        let mut current_layer = 0;
        
        // Start with modules that have no dependencies
        let mut current_modules: Vec<PathBuf> = graph.module_to_node
            .iter()
            .filter(|(path, &node)| {
                graph.graph.neighbors_directed(node, petgraph::Direction::Outgoing).count() == 0
            })
            .map(|(path, _)| path.clone())
            .collect();
            
        while !current_modules.is_empty() {
            let layer = DependencyLayer {
                level: current_layer,
                modules: current_modules.clone(),
                can_depend_on: (0..current_layer).collect(),
            };
            
            layers.push(layer);
            processed.extend(current_modules.iter().cloned());
            
            // Find next layer (modules that only depend on processed modules)
            current_modules = graph.module_to_node
                .iter()
                .filter(|(path, &node)| {
                    !processed.contains(path.as_path()) &&
                    graph.graph.neighbors_directed(node, petgraph::Direction::Outgoing)
                        .all(|dep_node| {
                            let dep_path = &graph.graph[dep_node].path;
                            processed.contains(dep_path.as_path()) || graph.graph[dep_node].is_external
                        })
                })
                .map(|(path, _)| path.clone())
                .collect();
                
            current_layer += 1;
        }
        
        Ok(layers)
    }

    /// Suggest refactorings
    fn suggest_refactorings(
        &self,
        graph: &DependencyGraph,
        circular_deps: &[CircularDependency]
    ) -> Result<Vec<RefactoringSuggestion>> {
        let mut suggestions = Vec::new();
        
        // Suggest breaking circular dependencies
        for circular in circular_deps {
            if circular.severity >= Severity::High {
                for breaking_point in &circular.breaking_points {
                    suggestions.push(RefactoringSuggestion {
                        kind: RefactoringKind::ExtractInterface,
                        target: breaking_point.clone(),
                        reason: format!("Break circular dependency between {} modules", circular.modules.len()),
                        impact: Impact::Significant,
                    });
                }
            }
        }
        
        // Suggest extracting highly coupled modules
        for (path, &node) in &graph.module_to_node {
            let in_degree = graph.graph.neighbors_directed(node, petgraph::Direction::Incoming).count();
            let out_degree = graph.graph.neighbors_directed(node, petgraph::Direction::Outgoing).count();
            
            if in_degree > 10 && out_degree > 10 {
                suggestions.push(RefactoringSuggestion {
                    kind: RefactoringKind::ExtractModule,
                    target: path.clone(),
                    reason: format!("Module has {} dependencies and {} dependents", out_degree, in_degree),
                    impact: Impact::Major,
                });
            }
        }
        
        // Suggest removing unused modules
        for (path, &node) in &graph.module_to_node {
            let in_degree = graph.graph.neighbors_directed(node, petgraph::Direction::Incoming).count();
            let module = &graph.graph[node];
            
            if in_degree == 0 && module.export_count > 0 && !module.is_external {
                suggestions.push(RefactoringSuggestion {
                    kind: RefactoringKind::RemoveUnused,
                    target: path.clone(),
                    reason: "Module exports symbols but has no dependents".to_string(),
                    impact: Impact::Minimal,
                });
            }
        }
        
        Ok(suggestions)
    }

    /// Generate DOT visualization
    fn generate_visualization(&self, graph: &DependencyGraph) -> Result<String> {
        let dot = Dot::with_config(&graph.graph, &[Config::EdgeNoLabel]);
        Ok(format!("{:?}", dot))
    }

    /// Get edges in a cycle
    fn get_cycle_edges(&self, graph: &DependencyGraph, cycle: &[PathBuf]) -> Vec<(PathBuf, PathBuf)> {
        let mut edges = Vec::new();
        
        for i in 0..cycle.len() {
            let from = &cycle[i];
            let to = &cycle[(i + 1) % cycle.len()];
            edges.push((from.clone(), to.clone()));
        }
        
        edges
    }

    /// Calculate cycle severity
    fn calculate_cycle_severity(&self, cycle_length: usize, edges: &[(PathBuf, PathBuf)]) -> Severity {
        match cycle_length {
            2 => Severity::Medium,
            3..=5 => Severity::High,
            _ => Severity::Critical,
        }
    }

    /// Find breaking points in a cycle
    fn find_breaking_points(&self, graph: &DependencyGraph, cycle: &[PathBuf]) -> Vec<PathBuf> {
        // Simple heuristic: modules with lowest complexity
        let mut modules_with_complexity: Vec<_> = cycle
            .iter()
            .filter_map(|path| {
                graph.module_to_node.get(path).map(|&node| {
                    (path.clone(), graph.graph[node].complexity)
                })
            })
            .collect();
            
        modules_with_complexity.sort_by_key(|(_, complexity)| *complexity);
        
        modules_with_complexity
            .into_iter()
            .take(2)
            .map(|(path, _)| path)
            .collect()
    }

    /// Extract exports from parse result
    fn extract_exports(&self, parse_result: &ParseResult) -> Vec<String> {
        parse_result.symbols
            .iter()
            .filter(|s| matches!(s.kind, crate::core::ast::SymbolKind::Function | crate::core::ast::SymbolKind::Class | crate::core::ast::SymbolKind::Interface))
            .map(|s| s.name.clone())
            .collect()
    }

    /// Calculate module complexity
    fn calculate_complexity(&self, parse_result: &ParseResult) -> u32 {
        parse_result.metrics.complexity
    }

    /// Calculate dependency strength
    fn calculate_dependency_strength(&self, module_info: &ModuleInfo, import: &ImportInfo) -> f32 {
        let import_count = import.items.len().max(1) as f32;
        let total_imports = module_info.imports.len().max(1) as f32;
        
        (import_count / total_imports).min(1.0)
    }

    /// Determine dependency kind
    fn determine_dependency_kind(&self, import: &ImportInfo) -> DependencyKind {
        // Simple heuristic - can be enhanced
        if import.is_wildcard {
            DependencyKind::Static
        } else {
            DependencyKind::Static
        }
    }

    /// Resolve import to module path
    async fn resolve_import(&self, from_path: &Path, import_module: &str) -> Option<PathBuf> {
        // Simple resolution - in reality would need proper module resolution
        let parent = from_path.parent()?;
        
        // Try relative import
        let relative_path = parent.join(import_module).with_extension("rs");
        if relative_path.exists() {
            return Some(relative_path);
        }
        
        // Try with common extensions
        for ext in &["ts", "js", "py", "go", "java"] {
            let path = parent.join(import_module).with_extension(ext);
            if path.exists() {
                return Some(path);
            }
        }
        
        None
    }

    /// Check if module is from standard library
    fn is_standard_library(&self, module: &str) -> bool {
        // Language-specific standard library detection
        matches!(module,
            "std" | "core" | "alloc" | // Rust
            "fs" | "path" | "http" | "crypto" | // Node.js
            "os" | "sys" | "io" | "json" | // Python
            "fmt" | "strings" | "net" | // Go
            "java.lang" | "java.util" | "java.io" // Java
        ) || module.starts_with("std::") || module.starts_with("java.")
    }

    /// Detect language from file extension
    fn detect_language(&self, path: &Path) -> Result<Language> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
            
        match ext {
            "rs" => Ok(Language::Rust),
            "ts" | "tsx" => Ok(Language::TypeScript),
            "js" | "jsx" => Ok(Language::JavaScript),
            "py" => Ok(Language::Python),
            "go" => Ok(Language::Go),
            "java" => Ok(Language::Java),
            "cpp" | "cc" | "cxx" => Ok(Language::Cpp),
            "c" => Ok(Language::C),
            _ => Ok(Language::Unknown),
        }
    }

    /// Get or create parser
    async fn get_parser(&self, language: Language) -> Result<Arc<tokio::sync::Mutex<TreeSitterParser>>> {
        let mut parsers = self.parsers.write().await;
        
        if let Some(parser) = parsers.get(&language) {
            Ok(Arc::clone(parser))
        } else {
            let parser = TreeSitterParser::new(language)?;
            let parser = Arc::new(tokio::sync::Mutex::new(parser));
            parsers.insert(language, Arc::clone(&parser));
            Ok(parser)
        }
    }
}

impl DependencyGraph {
    fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            module_to_node: HashMap::new(),
            stats: DependencyStats::default(),
        }
    }

    fn add_module(&mut self, module: ModuleNode) -> NodeIndex {
        if let Some(&node) = self.module_to_node.get(&module.path) {
            node
        } else {
            let node = self.graph.add_node(module.clone());
            self.module_to_node.insert(module.path, node);
            node
        }
    }

    fn add_dependency(&mut self, from: &Path, to: &Path, edge: DependencyEdge) -> Result<()> {
        let from_node = *self.module_to_node.get(from)
            .ok_or_else(|| anyhow::anyhow!("Source module not found"))?;
        let to_node = *self.module_to_node.get(to)
            .ok_or_else(|| anyhow::anyhow!("Target module not found"))?;
            
        self.graph.add_edge(from_node, to_node, edge);
        Ok(())
    }

    fn detect_circular_dependencies(&mut self) {
        // Find strongly connected components
        let scc = tarjan_scc(&self.graph);
        
        // Filter cycles (components with more than one node)
        let cycles: Vec<Vec<PathBuf>> = scc
            .into_iter()
            .filter(|component| component.len() > 1)
            .map(|component| {
                component
                    .into_iter()
                    .map(|node| self.graph[node].path.clone())
                    .collect()
            })
            .collect();
            
        self.stats.circular_dependencies = cycles;
        
        // Mark circular edges
        let edges_to_update: Vec<_> = self.graph.edge_indices()
            .filter_map(|edge_idx| {
                let (source, target) = self.graph.edge_endpoints(edge_idx)?;
                if has_path_connecting(&self.graph, target, source, None) {
                    Some(edge_idx)
                } else {
                    None
                }
            })
            .collect();
            
        for edge_idx in edges_to_update {
            if let Some(edge) = self.graph.edge_weight_mut(edge_idx) {
                edge.is_circular = true;
            }
        }
    }

    fn update_stats(&mut self) {
        self.stats.total_modules = self.graph.node_count();
        self.stats.total_dependencies = self.graph.edge_count();
        
        // Count external dependencies
        self.stats.external_dependencies = self.graph
            .node_indices()
            .filter(|&node| self.graph[node].is_external)
            .count();
            
        // Calculate average dependencies per module
        if self.stats.total_modules > 0 {
            self.stats.avg_dependencies_per_module = 
                self.stats.total_dependencies as f32 / self.stats.total_modules as f32;
        }
        
        // Find most depended on modules
        let mut dependency_counts: HashMap<PathBuf, usize> = HashMap::new();
        for node in self.graph.node_indices() {
            let path = &self.graph[node].path;
            let count = self.graph
                .neighbors_directed(node, petgraph::Direction::Incoming)
                .count();
            dependency_counts.insert(path.clone(), count);
        }
        
        let mut sorted_deps: Vec<_> = dependency_counts.into_iter().collect();
        sorted_deps.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        self.stats.most_depended_on = sorted_deps.into_iter().take(10).collect();
        
        // Find most dependent modules
        let mut dependent_counts: HashMap<PathBuf, usize> = HashMap::new();
        for node in self.graph.node_indices() {
            let path = &self.graph[node].path;
            let count = self.graph
                .neighbors_directed(node, petgraph::Direction::Outgoing)
                .count();
            dependent_counts.insert(path.clone(), count);
        }
        
        let mut sorted_deps: Vec<_> = dependent_counts.into_iter().collect();
        sorted_deps.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        self.stats.most_dependent = sorted_deps.into_iter().take(10).collect();
        
        // Calculate max depth
        self.stats.max_depth = self.calculate_max_depth();
    }

    fn calculate_max_depth(&self) -> usize {
        // Use topological sort to find longest path
        if let Ok(topo) = toposort(&self.graph, None) {
            let mut depths = HashMap::new();
            let mut max_depth = 0;
            
            for node in topo {
                let depth = self.graph
                    .neighbors_directed(node, petgraph::Direction::Incoming)
                    .filter_map(|pred| depths.get(&pred))
                    .max()
                    .map(|d| d + 1)
                    .unwrap_or(0);
                    
                depths.insert(node, depth);
                max_depth = max_depth.max(depth);
            }
            
            max_depth
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dependency_analysis() -> Result<()> {
        let analyzer = DependencyAnalyzer::new().await?;
        
        // Would need a test project structure
        // let analysis = analyzer.analyze_project(Path::new("test_project")).await?;
        
        Ok(())
    }

    #[test]
    fn test_cycle_severity() {
        let analyzer = DependencyAnalyzer::new();
        
        assert_eq!(
            analyzer.calculate_cycle_severity(2, &[]),
            Severity::Medium
        );
        
        assert_eq!(
            analyzer.calculate_cycle_severity(4, &[]),
            Severity::High
        );
        
        assert_eq!(
            analyzer.calculate_cycle_severity(10, &[]),
            Severity::Critical
        );
    }
}