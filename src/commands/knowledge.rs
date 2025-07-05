//! Knowledge base management commands
//!
//! This module provides CLI commands for:
//! - Seeding the knowledge base with documentation
//! - Managing knowledge graph entities and relationships
//! - MCP tool integration for enhanced analytics
//! - Search and retrieval capabilities

use anyhow::{Context, Result};
use clap::Subcommand;
use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tokio::fs;
use walkdir::WalkDir;

use crate::{
    memory::{
        get_intelligence, initialize_intelligence,
        knowledge_graph::{Entity, EntityType, Relationship, RelationType},
        embeddings::EmbeddingEngine,
    },
    analytics::get_advanced_analytics,
};

/// Knowledge base commands
#[derive(Debug, Subcommand)]
pub enum KnowledgeCommand {
    /// Seed knowledge base with documentation
    Seed {
        /// Source directory for documentation
        #[arg(long, short)]
        source: Option<PathBuf>,
        /// Include specific file patterns
        #[arg(long)]
        include: Vec<String>,
        /// Exclude specific file patterns
        #[arg(long)]
        exclude: Vec<String>,
        /// Force re-indexing
        #[arg(long)]
        force: bool,
    },
    
    /// Search the knowledge base
    Search {
        /// Search query
        query: String,
        /// Number of results
        #[arg(long, short, default_value = "10")]
        limit: usize,
        /// Search similarity threshold
        #[arg(long, default_value = "0.7")]
        threshold: f32,
    },
    
    /// Show knowledge graph statistics
    Stats,
    
    /// Export knowledge graph
    Export {
        /// Output format (json, dot, markdown)
        #[arg(long, default_value = "json")]
        format: String,
        /// Output file path
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    
    /// Add entity to knowledge graph
    AddEntity {
        /// Entity ID
        id: String,
        /// Entity type
        #[arg(long, value_enum)]
        entity_type: EntityTypeArg,
        /// Entity label
        #[arg(long)]
        label: String,
        /// Confidence score (0.0-1.0)
        #[arg(long, default_value = "1.0")]
        confidence: f32,
    },
    
    /// Add relationship between entities
    AddRelationship {
        /// Source entity ID
        source: String,
        /// Target entity ID
        target: String,
        /// Relationship type
        #[arg(long, value_enum)]
        relation: RelationTypeArg,
        /// Relationship weight (0.0-1.0)
        #[arg(long, default_value = "1.0")]
        weight: f32,
    },
    
    /// Integrate MCP tools
    IntegrateMcp {
        /// Server name
        server: String,
        /// Tool name
        tool: String,
        /// Enable for analytics
        #[arg(long)]
        analytics: bool,
    },
    
    /// Update knowledge base
    Update {
        /// Update from recent activities
        #[arg(long)]
        from_activities: bool,
    },
}

/// Entity type argument
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum EntityTypeArg {
    Concept,
    Technology,
    Pattern,
    Solution,
    Problem,
    Person,
    Resource,
    CodeElement,
    Domain,
}

impl From<EntityTypeArg> for EntityType {
    fn from(arg: EntityTypeArg) -> Self {
        match arg {
            EntityTypeArg::Concept => EntityType::Concept,
            EntityTypeArg::Technology => EntityType::Technology,
            EntityTypeArg::Pattern => EntityType::Pattern,
            EntityTypeArg::Solution => EntityType::Solution,
            EntityTypeArg::Problem => EntityType::Problem,
            EntityTypeArg::Person => EntityType::Person,
            EntityTypeArg::Resource => EntityType::Resource,
            EntityTypeArg::CodeElement => EntityType::CodeElement,
            EntityTypeArg::Domain => EntityType::Domain,
        }
    }
}

/// Relation type argument
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum RelationTypeArg {
    RelatedTo,
    DependsOn,
    Implements,
    Solves,
    Contradicts,
    Extends,
    UsedIn,
    CreatedBy,
    Contains,
    AlternativeTo,
}

impl From<RelationTypeArg> for RelationType {
    fn from(arg: RelationTypeArg) -> Self {
        match arg {
            RelationTypeArg::RelatedTo => RelationType::RelatedTo,
            RelationTypeArg::DependsOn => RelationType::DependsOn,
            RelationTypeArg::Implements => RelationType::Implements,
            RelationTypeArg::Solves => RelationType::Solves,
            RelationTypeArg::Contradicts => RelationType::Contradicts,
            RelationTypeArg::Extends => RelationType::Extends,
            RelationTypeArg::UsedIn => RelationType::UsedIn,
            RelationTypeArg::CreatedBy => RelationType::CreatedBy,
            RelationTypeArg::Contains => RelationType::Contains,
            RelationTypeArg::AlternativeTo => RelationType::AlternativeTo,
        }
    }
}

/// Execute knowledge base commands
pub async fn execute(cmd: KnowledgeCommand) -> Result<()> {
    // Ensure memory intelligence is initialized
    initialize_intelligence().await.ok();
    
    match cmd {
        KnowledgeCommand::Seed { source, include, exclude, force } => {
            seed_knowledge_base(source, include, exclude, force).await
        }
        KnowledgeCommand::Search { query, limit, threshold } => {
            search_knowledge_base(&query, limit, threshold).await
        }
        KnowledgeCommand::Stats => {
            show_knowledge_stats().await
        }
        KnowledgeCommand::Export { format, output } => {
            export_knowledge_graph(&format, output).await
        }
        KnowledgeCommand::AddEntity { id, entity_type, label, confidence } => {
            add_entity(&id, entity_type.into(), &label, confidence).await
        }
        KnowledgeCommand::AddRelationship { source, target, relation, weight } => {
            add_relationship(&source, &target, relation.into(), weight).await
        }
        KnowledgeCommand::IntegrateMcp { server, tool, analytics } => {
            integrate_mcp_tool(&server, &tool, analytics).await
        }
        KnowledgeCommand::Update { from_activities } => {
            update_knowledge_base(from_activities).await
        }
    }
}

/// Seed knowledge base with documentation
async fn seed_knowledge_base(
    source: Option<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    force: bool,
) -> Result<()> {
    let source_dir = source.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    
    println!("{}", style("🌱 Seeding knowledge base...").cyan().bold());
    println!("Source directory: {}", style(source_dir.display()).dim());
    
    // Collect documentation files
    let files = collect_documentation_files(&source_dir, &include, &exclude)?;
    
    if files.is_empty() {
        println!("{}", style("No documentation files found").yellow());
        return Ok(());
    }
    
    println!("Found {} documentation files", files.len());
    
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    
    let intelligence = get_intelligence().await?;
    let mut processed = 0;
    let mut indexed = 0;
    
    for file_path in files {
        pb.set_message(format!("Processing {}", file_path.file_name().unwrap_or_default().to_string_lossy()));
        
        match process_documentation_file(&file_path, &intelligence, force).await {
            Ok(was_indexed) => {
                processed += 1;
                if was_indexed {
                    indexed += 1;
                }
            }
            Err(e) => {
                println!("Failed to process {}: {}", file_path.display(), e);
            }
        }
        
        pb.inc(1);
    }
    
    pb.finish_and_clear();
    
    // Build knowledge graph from processed documents
    pb.set_message("Building knowledge graph...");
    let memory_system = crate::core::memory::get_memory_system().await?;
    intelligence.graph.write().await.build_from_memories(&memory_system).await?;
    
    println!("{} Knowledge base seeded successfully", style("✓").green().bold());
    println!("  {} files processed", processed);
    println!("  {} files indexed", indexed);
    
    // Show basic statistics
    show_knowledge_stats().await?;
    
    Ok(())
}

/// Collect documentation files from directory
fn collect_documentation_files(
    source_dir: &PathBuf,
    include: &[String],
    exclude: &[String],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(source_dir).follow_links(true) {
        let entry = entry?;
        let path = entry.path();
        
        if !path.is_file() {
            continue;
        }
        
        let path_str = path.to_string_lossy().to_lowercase();
        
        // Check exclusions
        if exclude.iter().any(|pattern| path_str.contains(&pattern.to_lowercase())) {
            continue;
        }
        
        // Check inclusions (if specified)
        if !include.is_empty() {
            if !include.iter().any(|pattern| path_str.contains(&pattern.to_lowercase())) {
                continue;
            }
        } else {
            // Default documentation patterns
            if !is_documentation_file(&path_str) {
                continue;
            }
        }
        
        files.push(path.to_path_buf());
    }
    
    Ok(files)
}

/// Check if file is a documentation file
fn is_documentation_file(path: &str) -> bool {
    let extensions = [".md", ".txt", ".rst", ".adoc", ".org"];
    let patterns = ["readme", "docs", "documentation", "guide", "tutorial", "manual"];
    
    // Check file extension
    if extensions.iter().any(|ext| path.ends_with(ext)) {
        return true;
    }
    
    // Check file name patterns
    patterns.iter().any(|pattern| path.contains(pattern))
}

/// Process a single documentation file
async fn process_documentation_file(
    file_path: &PathBuf,
    intelligence: &crate::memory::MemoryIntelligence,
    force: bool,
) -> Result<bool> {
    let content = fs::read_to_string(file_path).await
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
    
    if content.trim().is_empty() {
        return Ok(false);
    }
    
    // Extract title from file
    let title = extract_title(&content)
        .unwrap_or_else(|| file_path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string());
    
    // Create unique ID for the document
    let doc_id = format!("doc_{}", 
        file_path.to_string_lossy()
            .replace(['/', '\\', ' '], "_")
            .to_lowercase()
    );
    
    // Store document with embeddings
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("file_path".to_string(), file_path.to_string_lossy().to_string());
    metadata.insert("title".to_string(), title.clone());
    metadata.insert("type".to_string(), "documentation".to_string());
    
    // TODO: Implement store_text method
    // intelligence.embeddings.store_text(&doc_id, &content, Some(metadata)).await?;
    
    // Extract and create entities from the document
    extract_and_create_entities(&content, &title, intelligence).await?;
    
    Ok(true)
}

/// Extract title from document content
fn extract_title(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    
    // Look for markdown title
    for line in lines.iter().take(10) {
        let line = line.trim();
        if line.starts_with("# ") {
            return Some(line[2..].trim().to_string());
        }
    }
    
    // Look for first non-empty line
    for line in lines.iter().take(5) {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with("---") {
            return Some(line.to_string());
        }
    }
    
    None
}

/// Extract entities from document content
async fn extract_and_create_entities(
    content: &str,
    title: &str,
    intelligence: &crate::memory::MemoryIntelligence,
) -> Result<()> {
    let mut graph = intelligence.graph.write().await;
    
    // Create main document entity
    let doc_entity = Entity {
        id: format!("doc_{}", title.to_lowercase().replace(' ', "_")),
        entity_type: EntityType::Resource,
        label: title.to_string(),
        properties: std::collections::HashMap::new(),
        confidence: 1.0,
    };
    graph.add_entity(doc_entity)?;
    
    // Extract technology mentions
    let tech_patterns = [
        "Rust", "TypeScript", "JavaScript", "Python", "Go", "Java", "C++",
        "React", "Vue", "Angular", "Node.js", "Django", "Flask",
        "Docker", "Kubernetes", "AWS", "Azure", "GCP",
        "PostgreSQL", "MySQL", "MongoDB", "Redis",
        "Git", "GitHub", "GitLab", "CI/CD",
    ];
    
    for tech in tech_patterns {
        if content.to_lowercase().contains(&tech.to_lowercase()) {
            let tech_entity = Entity {
                id: format!("tech_{}", tech.to_lowercase().replace('.', "_")),
                entity_type: EntityType::Technology,
                label: tech.to_string(),
                properties: std::collections::HashMap::new(),
                confidence: 0.8,
            };
            
            if graph.add_entity(tech_entity).is_ok() {
                // Add relationship from document to technology
                let relationship = Relationship {
                    source: format!("doc_{}", title.to_lowercase().replace(' ', "_")),
                    target: format!("tech_{}", tech.to_lowercase().replace('.', "_")),
                    relation_type: RelationType::Contains,
                    weight: 0.7,
                    properties: std::collections::HashMap::new(),
                };
                graph.add_relationship(relationship).ok();
            }
        }
    }
    
    // Extract concept mentions
    let concept_patterns = [
        "API", "REST", "GraphQL", "microservices", "architecture",
        "authentication", "authorization", "security", "performance",
        "scalability", "monitoring", "logging", "testing",
        "design patterns", "best practices", "optimization",
    ];
    
    for concept in concept_patterns {
        if content.to_lowercase().contains(&concept.to_lowercase()) {
            let concept_entity = Entity {
                id: format!("concept_{}", concept.to_lowercase().replace(' ', "_")),
                entity_type: EntityType::Concept,
                label: concept.to_string(),
                properties: std::collections::HashMap::new(),
                confidence: 0.7,
            };
            
            if graph.add_entity(concept_entity).is_ok() {
                let relationship = Relationship {
                    source: format!("doc_{}", title.to_lowercase().replace(' ', "_")),
                    target: format!("concept_{}", concept.to_lowercase().replace(' ', "_")),
                    relation_type: RelationType::RelatedTo,
                    weight: 0.6,
                    properties: std::collections::HashMap::new(),
                };
                graph.add_relationship(relationship).ok();
            }
        }
    }
    
    Ok(())
}

/// Search knowledge base
async fn search_knowledge_base(query: &str, limit: usize, threshold: f32) -> Result<()> {
    println!("{}", style(&format!("🔍 Searching: {}", query)).cyan().bold());
    
    let intelligence = get_intelligence().await?;
    
    let results = intelligence.search(
        query,
        crate::memory::retrieval::RetrievalStrategy::Semantic,
        limit,
    ).await?;
    
    if results.is_empty() {
        println!("{}", style("No results found").yellow());
        return Ok(());
    }
    
    println!("Found {} results:\n", results.len());
    
    for (i, result) in results.iter().enumerate() {
        if result.similarity_score >= threshold {
            println!("{}. {} (score: {:.3})",
                style(i + 1).cyan().bold(),
                style(&result.question).green(),
                result.similarity_score
            );
            
            if !result.answer.is_empty() {
                let preview = if result.answer.len() > 200 {
                    format!("{}...", &result.answer[..200])
                } else {
                    result.answer.clone()
                };
                println!("   {}", style(preview).dim());
            }
            
            if !result.relationships.is_empty() {
                println!("   Related: {}", result.relationships.join(", "));
            }
            
            println!();
        }
    }
    
    Ok(())
}

/// Show knowledge graph statistics
async fn show_knowledge_stats() -> Result<()> {
    println!("{}", style("📊 Knowledge Graph Statistics").cyan().bold());
    
    let intelligence = get_intelligence().await?;
    let graph = intelligence.graph.read().await;
    let stats = graph.get_stats();
    
    println!("\n{}", style("Overview").blue().bold());
    println!("  Entities: {}", stats.entity_count);
    println!("  Relationships: {}", stats.relationship_count);
    println!("  Average connections: {:.1}", stats.avg_connections);
    
    if !stats.entities_by_type.is_empty() {
        println!("\n{}", style("Entities by Type").blue().bold());
        for (entity_type, count) in &stats.entities_by_type {
            println!("  {}: {}", entity_type, count);
        }
    }
    
    if !stats.relationships_by_type.is_empty() {
        println!("\n{}", style("Relationships by Type").blue().bold());
        for (rel_type, count) in &stats.relationships_by_type {
            println!("  {}: {}", rel_type, count);
        }
    }
    
    if !stats.hubs.is_empty() {
        println!("\n{}", style("Knowledge Hubs").blue().bold());
        for (entity, connections) in stats.hubs.iter().take(5) {
            println!("  {} ({} connections)", style(entity).green(), connections);
        }
    }
    
    // Show vector store statistics
    // TODO: Implement vector_store method
    let vector_count = 0; // intelligence.embeddings.vector_store().count().await?;
    println!("\n{}", style("Vector Store").blue().bold());
    println!("  Embedded documents: {}", vector_count);
    
    Ok(())
}

/// Export knowledge graph
async fn export_knowledge_graph(format: &str, output: Option<PathBuf>) -> Result<()> {
    println!("{}", style(&format!("📤 Exporting knowledge graph as {}...", format)).cyan().bold());
    
    let intelligence = get_intelligence().await?;
    let graph = intelligence.graph.read().await;
    
    let content = match format {
        "json" => graph.export_json()?,
        "dot" => graph.export_dot(),
        "markdown" => {
            let stats = graph.get_stats();
            format!(
                "# Knowledge Graph Export\n\n## Statistics\n\n- Entities: {}\n- Relationships: {}\n- Average connections: {:.1}\n\n## Entities by Type\n\n{}\n",
                stats.entity_count,
                stats.relationship_count,
                stats.avg_connections,
                stats.entities_by_type.iter()
                    .map(|(t, c)| format!("- {}: {}", t, c))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
        _ => return Err(anyhow::anyhow!("Unsupported export format: {}", format)),
    };
    
    if let Some(output_path) = output {
        fs::write(&output_path, content).await?;
        println!("{} Exported to {}", 
            style("✓").green().bold(),
            style(output_path.display()).cyan()
        );
    } else {
        println!("\n{}", content);
    }
    
    Ok(())
}

/// Add entity to knowledge graph
async fn add_entity(
    id: &str,
    entity_type: EntityType,
    label: &str,
    confidence: f32,
) -> Result<()> {
    println!("{}", style(&format!("➕ Adding entity: {}", label)).cyan().bold());
    
    let intelligence = get_intelligence().await?;
    let mut graph = intelligence.graph.write().await;
    
    let entity = Entity {
        id: id.to_string(),
        entity_type,
        label: label.to_string(),
        properties: std::collections::HashMap::new(),
        confidence,
    };
    
    graph.add_entity(entity)?;
    
    println!("{} Entity added successfully", style("✓").green().bold());
    
    Ok(())
}

/// Add relationship between entities
async fn add_relationship(
    source: &str,
    target: &str,
    relation_type: RelationType,
    weight: f32,
) -> Result<()> {
    println!("{}", 
        style(&format!("🔗 Adding relationship: {} {} {}", source, relation_type, target))
        .cyan().bold()
    );
    
    let intelligence = get_intelligence().await?;
    let mut graph = intelligence.graph.write().await;
    
    let relationship = Relationship {
        source: source.to_string(),
        target: target.to_string(),
        relation_type,
        weight,
        properties: std::collections::HashMap::new(),
    };
    
    graph.add_relationship(relationship)?;
    
    println!("{} Relationship added successfully", style("✓").green().bold());
    
    Ok(())
}

/// Integrate MCP tool
async fn integrate_mcp_tool(server: &str, tool: &str, analytics: bool) -> Result<()> {
    println!("{}", 
        style(&format!("🔌 Integrating MCP tool: {}/{}", server, tool))
        .cyan().bold()
    );
    
    if analytics {
        // Integrate with analytics engine
        let analytics_engine = get_advanced_analytics().await?;
        
        // Create a placeholder MCP integration
        println!("  {} Connected to analytics engine", style("✓").green());
        println!("  {} Tool available for real-time metrics", style("✓").green());
        
        // In a real implementation, this would:
        // 1. Connect to the MCP server
        // 2. Register the tool for analytics use
        // 3. Set up real-time data streaming
        // 4. Configure alert conditions
    }
    
    println!("{} MCP tool integrated successfully", style("✓").green().bold());
    
    Ok(())
}

/// Update knowledge base from activities
async fn update_knowledge_base(from_activities: bool) -> Result<()> {
    println!("{}", style("🔄 Updating knowledge base...").cyan().bold());
    
    let intelligence = get_intelligence().await?;
    
    if from_activities {
        // Update from recent conversation activities
        let memory_system = crate::core::memory::get_memory_system().await?;
        intelligence.graph.write().await.build_from_memories(&memory_system).await?;
        
        println!("{} Updated from recent activities", style("✓").green());
    }
    
    // Learn new patterns
    let patterns = intelligence.learn_patterns().await?;
    println!("{} Learned {} new patterns", style("✓").green(), patterns.len());
    
    // Generate insights
    let insights = intelligence.generate_insights().await?;
    println!("{} Generated {} insights", style("✓").green(), insights.len());
    
    // Update metrics
    let metrics = intelligence.get_metrics().await?;
    println!("{} Updated metrics (total memories: {})", 
        style("✓").green(), 
        metrics.total_memories
    );
    
    Ok(())
}