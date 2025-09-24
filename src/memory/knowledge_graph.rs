//! Dynamic knowledge graph construction and traversal
//!
//! This module provides:
//! - Entity extraction and relationship mapping
//! - Graph construction with petgraph
//! - Graph queries and traversal algorithms
//! - Visualization and export capabilities

use anyhow::{Context as _, Result};
use petgraph::algo::{all_simple_paths, dijkstra};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use tracing::{debug, info};

use crate::core::memory::MemorySystem;

/// Entity in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier
    pub id: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Display label
    pub label: String,
    /// Additional properties
    pub properties: HashMap<String, String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// Types of entities in the knowledge graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// Programming concept
    Concept,
    /// Technology or tool
    Technology,
    /// Design pattern
    Pattern,
    /// Solution to a problem
    Solution,
    /// Problem statement
    Problem,
    /// Person or author
    Person,
    /// Resource or reference
    Resource,
    /// Code element (class, function, etc.)
    CodeElement,
    /// Business domain concept
    Domain,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityType::Concept => write!(f, "Concept"),
            EntityType::Technology => write!(f, "Technology"),
            EntityType::Pattern => write!(f, "Pattern"),
            EntityType::Solution => write!(f, "Solution"),
            EntityType::Problem => write!(f, "Problem"),
            EntityType::Person => write!(f, "Person"),
            EntityType::Resource => write!(f, "Resource"),
            EntityType::CodeElement => write!(f, "Code"),
            EntityType::Domain => write!(f, "Domain"),
        }
    }
}

/// Relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Source entity ID
    pub source: String,
    /// Target entity ID
    pub target: String,
    /// Relationship type
    pub relation_type: RelationType,
    /// Strength of relationship (0.0 - 1.0)
    pub weight: f32,
    /// Additional properties
    pub properties: HashMap<String, String>,
}

/// Types of relationships
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// General relationship
    RelatedTo,
    /// Dependency relationship
    DependsOn,
    /// Implementation relationship
    Implements,
    /// Solution relationship
    Solves,
    /// Contradiction
    Contradicts,
    /// Extension/inheritance
    Extends,
    /// Usage relationship
    UsedIn,
    /// Creation relationship
    CreatedBy,
    /// Composition
    Contains,
    /// Alternative to
    AlternativeTo,
}

impl fmt::Display for RelationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationType::RelatedTo => write!(f, "related to"),
            RelationType::DependsOn => write!(f, "depends on"),
            RelationType::Implements => write!(f, "implements"),
            RelationType::Solves => write!(f, "solves"),
            RelationType::Contradicts => write!(f, "contradicts"),
            RelationType::Extends => write!(f, "extends"),
            RelationType::UsedIn => write!(f, "used in"),
            RelationType::CreatedBy => write!(f, "created by"),
            RelationType::Contains => write!(f, "contains"),
            RelationType::AlternativeTo => write!(f, "alternative to"),
        }
    }
}

/// Knowledge graph structure
#[derive(Debug)]
pub struct KnowledgeGraph {
    /// The underlying graph
    graph: DiGraph<Entity, Relationship>,
    /// Entity ID to node index mapping
    entity_map: HashMap<String, NodeIndex>,
    /// Entity type index for fast lookup
    type_index: HashMap<EntityType, HashSet<String>>,
    /// Statistics
    stats: GraphStatistics,
}

/// Graph statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    /// Total number of entities
    pub entity_count: usize,
    /// Total number of relationships
    pub relationship_count: usize,
    /// Entity count by type
    pub entities_by_type: HashMap<EntityType, usize>,
    /// Relationship count by type
    pub relationships_by_type: HashMap<RelationType, usize>,
    /// Average connections per entity
    pub avg_connections: f32,
    /// Most connected entities
    pub hubs: Vec<(String, usize)>,
}

/// Graph query builder
#[derive(Debug, Clone)]
pub struct GraphQuery {
    /// Entity types to include
    pub entity_types: Option<Vec<EntityType>>,
    /// Relationship types to include
    pub relation_types: Option<Vec<RelationType>>,
    /// Minimum confidence score
    pub min_confidence: Option<f32>,
    /// Maximum depth for traversal
    pub max_depth: Option<usize>,
    /// Starting entity IDs
    pub start_entities: Option<Vec<String>>,
}

impl Default for GraphQuery {
    fn default() -> Self {
        Self {
            entity_types: None,
            relation_types: None,
            min_confidence: None,
            max_depth: None,
            start_entities: None,
        }
    }
}

impl KnowledgeGraph {
    /// Create a new knowledge graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            entity_map: HashMap::new(),
            type_index: HashMap::new(),
            stats: GraphStatistics::default(),
        }
    }

    /// Add an entity to the graph
    pub fn add_entity(&mut self, entity: Entity) -> Result<NodeIndex> {
        // Check if entity already exists
        if let Some(&node_index) = self.entity_map.get(&entity.id) {
            // Update existing entity
            if let Some(node) = self.graph.node_weight_mut(node_index) {
                *node = entity.clone();
            }
            return Ok(node_index);
        }

        // Add new entity
        let node_index = self.graph.add_node(entity.clone());
        self.entity_map.insert(entity.id.clone(), node_index);

        // Update type index
        self.type_index
            .entry(entity.entity_type.clone())
            .or_insert_with(HashSet::new)
            .insert(entity.id.clone());

        // Update statistics
        self.stats.entity_count += 1;
        *self
            .stats
            .entities_by_type
            .entry(entity.entity_type)
            .or_insert(0) += 1;

        Ok(node_index)
    }

    /// Get graph statistics
    pub fn stats(&self) -> &GraphStatistics {
        &self.stats
    }

    /// Add a relationship to the graph
    pub fn add_relationship(&mut self, relationship: Relationship) -> Result<()> {
        // Ensure both entities exist
        let source_index = self
            .entity_map
            .get(&relationship.source)
            .ok_or_else(|| anyhow::anyhow!("Source entity not found: {}", relationship.source))?;
        let target_index = self
            .entity_map
            .get(&relationship.target)
            .ok_or_else(|| anyhow::anyhow!("Target entity not found: {}", relationship.target))?;

        // Add edge
        self.graph
            .add_edge(*source_index, *target_index, relationship.clone());

        // Update statistics
        self.stats.relationship_count += 1;
        *self
            .stats
            .relationships_by_type
            .entry(relationship.relation_type)
            .or_insert(0) += 1;

        Ok(())
    }

    /// Find entity by ID
    pub fn find_entity(&self, id: &str) -> Option<&Entity> {
        self.entity_map
            .get(id)
            .and_then(|&index| self.graph.node_weight(index))
    }

    /// Find relationships for an entity
    pub async fn find_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>> {
        let node_index = self
            .entity_map
            .get(entity_id)
            .ok_or_else(|| anyhow::anyhow!("Entity not found: {}", entity_id))?;

        let mut relationships = Vec::new();

        // Outgoing relationships
        for edge in self.graph.edges(*node_index) {
            relationships.push(edge.weight().clone());
        }

        // Incoming relationships
        for edge in self
            .graph
            .edges_directed(*node_index, petgraph::Direction::Incoming)
        {
            let mut rel = edge.weight().clone();
            // Swap source and target for incoming edges
            std::mem::swap(&mut rel.source, &mut rel.target);
            relationships.push(rel);
        }

        Ok(relationships)
    }

    /// Execute a graph query
    pub fn query(&self, query: &GraphQuery) -> Result<Vec<Entity>> {
        let mut results = Vec::new();
        let mut visited = HashSet::new();

        // Determine starting points
        let start_nodes: Vec<NodeIndex> = if let Some(ref start_ids) = query.start_entities {
            start_ids
                .iter()
                .filter_map(|id| self.entity_map.get(id))
                .copied()
                .collect()
        } else {
            self.graph.node_indices().collect()
        };

        // BFS traversal with filters
        let mut queue = VecDeque::new();
        for node in start_nodes {
            queue.push_back((node, 0));
        }

        while let Some((node_index, depth)) = queue.pop_front() {
            if visited.contains(&node_index) {
                continue;
            }
            visited.insert(node_index);

            // Check depth limit
            if let Some(max_depth) = query.max_depth {
                if depth > max_depth {
                    continue;
                }
            }

            // Get entity
            if let Some(entity) = self.graph.node_weight(node_index) {
                // Apply filters
                if let Some(ref types) = query.entity_types {
                    if !types.contains(&entity.entity_type) {
                        continue;
                    }
                }

                if let Some(min_conf) = query.min_confidence {
                    if entity.confidence < min_conf {
                        continue;
                    }
                }

                results.push(entity.clone());

                // Add neighbors to queue
                for edge in self.graph.edges(node_index) {
                    if let Some(ref rel_types) = query.relation_types {
                        if !rel_types.contains(&edge.weight().relation_type) {
                            continue;
                        }
                    }
                    queue.push_back((edge.target(), depth + 1));
                }
            }
        }

        Ok(results)
    }

    /// Find shortest path between two entities
    pub fn find_path(&self, from: &str, to: &str) -> Result<Vec<Entity>> {
        let from_index = self
            .entity_map
            .get(from)
            .ok_or_else(|| anyhow::anyhow!("Source entity not found: {}", from))?;
        let to_index = self
            .entity_map
            .get(to)
            .ok_or_else(|| anyhow::anyhow!("Target entity not found: {}", to))?;

        // Use Dijkstra's algorithm
        let node_map = dijkstra(&self.graph, *from_index, Some(*to_index), |e| {
            e.weight().weight
        });

        // Reconstruct path
        let mut path = Vec::new();
        let mut current = *to_index;

        while current != *from_index {
            if let Some(entity) = self.graph.node_weight(current) {
                path.push(entity.clone());
            }

            // Find predecessor
            let mut found = false;
            for edge in self
                .graph
                .edges_directed(current, petgraph::Direction::Incoming)
            {
                if node_map.contains_key(&edge.source()) {
                    current = edge.source();
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(anyhow::anyhow!("No path found between {} and {}", from, to));
            }
        }

        // Add starting entity
        if let Some(entity) = self.graph.node_weight(*from_index) {
            path.push(entity.clone());
        }

        path.reverse();
        Ok(path)
    }

    /// Calculate graph statistics
    pub fn calculate_statistics(&mut self) {
        self.stats = GraphStatistics {
            entity_count: self.graph.node_count(),
            relationship_count: self.graph.edge_count(),
            entities_by_type: HashMap::new(),
            relationships_by_type: HashMap::new(),
            avg_connections: 0.0,
            hubs: Vec::new(),
        };

        // Count entities by type
        for node in self.graph.node_weights() {
            *self
                .stats
                .entities_by_type
                .entry(node.entity_type.clone())
                .or_insert(0) += 1;
        }

        // Count relationships by type
        for edge in self.graph.edge_weights() {
            *self
                .stats
                .relationships_by_type
                .entry(edge.relation_type.clone())
                .or_insert(0) += 1;
        }

        // Calculate average connections and find hubs
        let mut connection_counts: Vec<(String, usize)> = Vec::new();
        let mut total_connections = 0;

        for (id, &node_index) in &self.entity_map {
            let connections = self.graph.edges(node_index).count()
                + self
                    .graph
                    .edges_directed(node_index, petgraph::Direction::Incoming)
                    .count();
            total_connections += connections;
            connection_counts.push((id.clone(), connections));
        }

        if self.stats.entity_count > 0 {
            self.stats.avg_connections = total_connections as f32 / self.stats.entity_count as f32;
        }

        // Find top 10 hubs
        connection_counts.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
        self.stats.hubs = connection_counts.into_iter().take(10).collect();
    }

    /// Export graph to DOT format
    pub fn export_dot(&self) -> String {
        let mut dot = String::from("digraph KnowledgeGraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        // Add nodes
        for (id, &node_index) in &self.entity_map {
            if let Some(entity) = self.graph.node_weight(node_index) {
                let color = match entity.entity_type {
                    EntityType::Concept => "lightblue",
                    EntityType::Technology => "lightgreen",
                    EntityType::Pattern => "orange",
                    EntityType::Solution => "mediumpurple",
                    EntityType::Problem => "lightcoral",
                    EntityType::Person => "tan",
                    EntityType::Resource => "lightgray",
                    EntityType::CodeElement => "lightyellow",
                    EntityType::Domain => "lightpink",
                };

                dot.push_str(&format!(
                    "  \"{}\" [label=\"{}\\n[{}]\", fillcolor=\"{}\", style=filled];\n",
                    id, entity.label, entity.entity_type, color
                ));
            }
        }

        dot.push_str("\n");

        // Add edges
        for edge in self.graph.edge_references() {
            if let (Some(source), Some(target)) = (
                self.graph.node_weight(edge.source()),
                self.graph.node_weight(edge.target()),
            ) {
                let edge_data = edge.weight();
                dot.push_str(&format!(
                    "  \"{}\" -> \"{}\" [label=\"{}\", weight={:.2}];\n",
                    source.id, target.id, edge_data.relation_type, edge_data.weight
                ));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Export graph to JSON format
    pub fn export_json(&self) -> Result<String> {
        #[derive(Serialize)]
        struct GraphExport {
            entities: Vec<Entity>,
            relationships: Vec<Relationship>,
            statistics: GraphStatistics,
        }

        let entities: Vec<Entity> = self.graph.node_weights().cloned().collect();
        let relationships: Vec<Relationship> = self.graph.edge_weights().cloned().collect();

        let export = GraphExport {
            entities,
            relationships,
            statistics: self.stats.clone(),
        };

        serde_json::to_string_pretty(&export).context("Failed to serialize graph to JSON")
    }

    /// Build graph from memory system
    pub async fn build_from_memories(&mut self, _memory_system: &MemorySystem) -> Result<()> {
        info!("Building knowledge graph from memory system");

        // This is a placeholder implementation
        // In production, this would:
        // 1. Extract entities from conversations using NER
        // 2. Identify relationships using dependency parsing
        // 3. Build the graph incrementally

        // For now, add some example entities
        self.add_entity(Entity {
            id: "rust".to_string(),
            entity_type: EntityType::Technology,
            label: "Rust".to_string(),
            properties: HashMap::new(),
            confidence: 1.0,
        })?;

        self.add_entity(Entity {
            id: "memory_system".to_string(),
            entity_type: EntityType::Concept,
            label: "Memory System".to_string(),
            properties: HashMap::new(),
            confidence: 0.95,
        })?;

        self.add_relationship(Relationship {
            source: "memory_system".to_string(),
            target: "rust".to_string(),
            relation_type: RelationType::Implements,
            weight: 0.9,
            properties: HashMap::new(),
        })?;

        self.calculate_statistics();

        info!(
            "Knowledge graph built with {} entities and {} relationships",
            self.stats.entity_count, self.stats.relationship_count
        );

        Ok(())
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_graph_construction() -> Result<()> {
        let mut graph = KnowledgeGraph::new();

        // Add entities
        let entity1 = Entity {
            id: "rust".to_string(),
            entity_type: EntityType::Technology,
            label: "Rust".to_string(),
            properties: HashMap::new(),
            confidence: 1.0,
        };

        let entity2 = Entity {
            id: "memory".to_string(),
            entity_type: EntityType::Concept,
            label: "Memory Management".to_string(),
            properties: HashMap::new(),
            confidence: 0.9,
        };

        graph.add_entity(entity1)?;
        graph.add_entity(entity2)?;

        // Add relationship
        graph.add_relationship(Relationship {
            source: "rust".to_string(),
            target: "memory".to_string(),
            relation_type: RelationType::Implements,
            weight: 0.8,
            properties: HashMap::new(),
        })?;

        assert_eq!(graph.stats.entity_count, 2);
        assert_eq!(graph.stats.relationship_count, 1);

        Ok(())
    }

    #[test]
    fn test_graph_query() -> Result<()> {
        let mut graph = KnowledgeGraph::new();

        // Build a small graph
        for i in 1..=5 {
            graph.add_entity(Entity {
                id: format!("tech{}", i),
                entity_type: EntityType::Technology,
                label: format!("Technology {}", i),
                properties: HashMap::new(),
                confidence: 0.8 + (i as f32 * 0.02),
            })?;
        }

        // Query for technologies with high confidence
        let query = GraphQuery {
            entity_types: Some(vec![EntityType::Technology]),
            min_confidence: Some(0.85),
            ..Default::default()
        };

        let results = graph.query(&query)?;
        assert!(results.len() <= 5);
        assert!(results.iter().all(|e| e.confidence >= 0.85));

        Ok(())
    }
}
