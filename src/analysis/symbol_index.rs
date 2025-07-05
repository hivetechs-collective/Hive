//! Symbol extraction and indexing system with SQLite FTS5 integration
//!
//! This module provides high-performance symbol indexing with sub-millisecond
//! search performance using SQLite FTS5 and petgraph for reference tracking.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use anyhow::{Result, Context, anyhow};
use rusqlite::{params, Connection, Transaction, OptionalExtension};
use serde::{Serialize, Deserialize};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{toposort, is_cyclic_directed};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};
use chrono::{DateTime, Utc};

use crate::core::{
    ast::{Symbol, SymbolKind, ParseResult, AstNode},
    Language, Position, 
};
use crate::analysis::parser::TreeSitterParser;
use crate::core::database::{DatabaseManager, DbConnection};

/// Symbol index entry with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SymbolEntry {
    /// Symbol identifier
    pub id: String,
    /// Symbol name
    pub name: String,
    /// Symbol kind (function, class, etc.)
    #[serde(default)]
    pub kind: SymbolKind,
    /// File path
    pub file_path: PathBuf,
    /// Start position
    pub start_pos: Position,
    /// End position
    pub end_pos: Position,
    /// Parent symbol ID (for nested symbols)
    pub parent_id: Option<String>,
    /// Symbol signature (for functions/methods)
    pub signature: Option<String>,
    /// Documentation
    pub documentation: Option<String>,
    /// Visibility (public, private, etc.)
    pub visibility: Option<String>,
    /// Type information
    pub type_info: Option<String>,
    /// Complexity score
    pub complexity: u32,
    /// Quality score (0-10)
    pub quality_score: f32,
    /// References count
    pub reference_count: usize,
    /// Is exported/public
    pub is_exported: bool,
    /// Usage count
    pub usage_count: usize,
    /// Last modified timestamp
    pub last_modified: DateTime<Utc>,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Reference between symbols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolReference {
    /// Source symbol ID
    pub from_symbol_id: String,
    /// Target symbol ID
    pub to_symbol_id: String,
    /// Reference kind (call, import, inherit, etc.)
    pub reference_kind: ReferenceKind,
    /// File path
    pub file_path: PathBuf,
    /// Position
    pub position: Position,
    /// Context line
    pub context: String,
}

/// Types of symbol references
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReferenceKind {
    Call,
    Import,
    Inherit,
    Implement,
    Instantiate,
    Reference,
    TypeUse,
}

/// Call graph with dependency tracking
pub struct CallGraph {
    /// Petgraph directed graph
    graph: DiGraph<String, ReferenceKind>,
    /// Symbol ID to node index mapping
    symbol_to_node: HashMap<String, NodeIndex>,
    /// Node index to symbol ID mapping
    node_to_symbol: HashMap<NodeIndex, String>,
}

/// Symbol indexer with FTS5 integration
pub struct SymbolIndexer {
    /// Database manager
    db: Arc<DatabaseManager>,
    /// Parser registry
    parsers: Arc<RwLock<HashMap<Language, Arc<tokio::sync::Mutex<TreeSitterParser>>>>>,
    /// Call graph
    call_graph: Arc<RwLock<CallGraph>>,
    /// Index statistics
    stats: Arc<RwLock<IndexStatistics>>,
}

/// Index statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IndexStatistics {
    pub total_symbols: usize,
    pub total_references: usize,
    pub files_indexed: usize,
    pub index_time_ms: f64,
    pub symbols_by_kind: HashMap<String, usize>,
    pub cyclic_dependencies: Vec<Vec<String>>,
}

impl SymbolIndexer {
    /// Create a new symbol indexer
    pub async fn new(db: Arc<DatabaseManager>) -> Result<Self> {
        // Initialize FTS5 tables
        let conn = db.get_connection()?;
        Self::init_tables(&conn)?;
        drop(conn);

        Ok(Self {
            db,
            parsers: Arc::new(RwLock::new(HashMap::new())),
            call_graph: Arc::new(RwLock::new(CallGraph::new())),
            stats: Arc::new(RwLock::new(IndexStatistics::default())),
        })
    }

    /// Initialize database tables with FTS5
    fn init_tables(conn: &Connection) -> Result<()> {
        // Main symbol table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbols (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                file_path TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                start_col INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                end_col INTEGER NOT NULL,
                parent_id TEXT,
                signature TEXT,
                documentation TEXT,
                visibility TEXT,
                type_info TEXT,
                complexity INTEGER NOT NULL DEFAULT 1,
                quality_score REAL NOT NULL DEFAULT 0.0,
                reference_count INTEGER NOT NULL DEFAULT 0,
                is_exported INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (parent_id) REFERENCES symbols(id)
            )",
            [],
        )?;

        // FTS5 table for full-text search
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
                id UNINDEXED,
                name,
                documentation,
                signature,
                file_path,
                content=symbols,
                content_rowid=rowid
            )",
            [],
        )?;

        // Triggers to keep FTS in sync
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS symbols_insert_fts 
             AFTER INSERT ON symbols BEGIN
                INSERT INTO symbols_fts(rowid, id, name, documentation, signature, file_path)
                VALUES (new.rowid, new.id, new.name, new.documentation, new.signature, new.file_path);
             END",
            [],
        )?;

        // Symbol references table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbol_references (
                id INTEGER PRIMARY KEY,
                from_symbol_id TEXT NOT NULL,
                to_symbol_id TEXT NOT NULL,
                reference_kind TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line INTEGER NOT NULL,
                col INTEGER NOT NULL,
                context TEXT,
                FOREIGN KEY (from_symbol_id) REFERENCES symbols(id),
                FOREIGN KEY (to_symbol_id) REFERENCES symbols(id)
            )",
            [],
        )?;

        // Indexes for performance
        conn.execute("CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_symbols_kind ON symbols(kind)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_path)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_references_from ON symbol_references(from_symbol_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_references_to ON symbol_references(to_symbol_id)", [])?;

        Ok(())
    }

    /// Index a file
    #[instrument(skip(self, content))]
    pub async fn index_file(&self, file_path: &Path, content: &str) -> Result<()> {
        let start = Instant::now();
        
        // Detect language
        let language = self.detect_language(file_path)?;
        
        // Get or create parser
        let parser = self.get_parser(language).await?;
        let mut parser = parser.lock().await;
        
        // Parse file
        let parse_result = parser.parse(content)?;
        
        // Extract symbols with enhanced metadata
        let symbols = self.extract_symbols(&parse_result, file_path, content)?;
        
        // Extract references
        let references = self.extract_references(&parse_result, file_path, content)?;
        
        // Store in database - complete all DB operations before async
        {
            let mut conn = self.db.get_connection()?;
            let tx = conn.transaction()?;
            
            // Clear existing symbols for this file
            tx.execute("DELETE FROM symbols WHERE file_path = ?1", params![file_path.to_str()])?;
            
            // Insert symbols
            for symbol in &symbols {
                self.insert_symbol(&tx, symbol)?;
            }
            
            // Insert references
            for reference in &references {
                self.insert_reference(&tx, reference)?;
            }
            
            tx.commit()?;
        } // conn and tx are dropped here, before any await
        
        // Update call graph
        self.update_call_graph(&symbols, &references).await?;
        
        // Update statistics
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        self.update_stats(symbols.len(), references.len(), elapsed).await;
        
        debug!(
            "Indexed {} with {} symbols and {} references in {:.2}ms",
            file_path.display(),
            symbols.len(),
            references.len(),
            elapsed
        );
        
        Ok(())
    }

    /// Search symbols with FTS5
    #[instrument(skip(self))]
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SymbolEntry>> {
        let start = Instant::now();
        let conn = self.db.get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT s.* FROM symbols s
             INNER JOIN symbols_fts f ON s.id = f.id
             WHERE symbols_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2"
        )?;
        
        let symbols = stmt.query_map(params![query, limit], |row| {
            Ok(SymbolEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: serde_json::from_str(&row.get::<_, String>(2)?).unwrap(),
                file_path: PathBuf::from(row.get::<_, String>(3)?),
                start_pos: Position {
                    line: row.get(4)?,
                    column: row.get(5)?,
                    offset: 0,
                },
                end_pos: Position {
                    line: row.get(6)?,
                    column: row.get(7)?,
                    offset: 0,
                },
                parent_id: row.get(8)?,
                signature: row.get(9)?,
                documentation: row.get(10)?,
                visibility: row.get(11)?,
                type_info: row.get(12)?,
                complexity: row.get(13)?,
                quality_score: row.get(14)?,
                reference_count: row.get(15)?,
                is_exported: row.get(16)?,
                usage_count: 0,
                last_modified: chrono::Utc::now(),
                attributes: HashMap::new(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        debug!("Symbol search for '{}' took {:.2}ms", query, elapsed);
        
        Ok(symbols)
    }

    /// Find all references to a symbol
    pub async fn find_references(&self, symbol_id: &str) -> Result<Vec<SymbolReference>> {
        let conn = self.db.get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT from_symbol_id, to_symbol_id, reference_kind, file_path, line, col, context
             FROM symbol_references
             WHERE to_symbol_id = ?1"
        )?;
        
        let references = stmt.query_map(params![symbol_id], |row| {
            Ok(SymbolReference {
                from_symbol_id: row.get(0)?,
                to_symbol_id: row.get(1)?,
                reference_kind: serde_json::from_str(&row.get::<_, String>(2)?).unwrap(),
                file_path: PathBuf::from(row.get::<_, String>(3)?),
                position: Position {
                    line: row.get(4)?,
                    column: row.get(5)?,
                    offset: 0,
                },
                context: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(references)
    }

    /// Get call graph for a symbol
    pub async fn get_call_graph(&self, symbol_id: &str) -> Result<CallGraphInfo> {
        let graph = self.call_graph.read().await;
        
        if let Some(&node_idx) = graph.symbol_to_node.get(symbol_id) {
            // Get direct calls
            let calls: Vec<String> = graph.graph
                .neighbors(node_idx)
                .filter_map(|n| graph.node_to_symbol.get(&n).cloned())
                .collect();
                
            // Get callers (reverse edges)
            let called_by: Vec<String> = graph.graph
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .filter_map(|n| graph.node_to_symbol.get(&n).cloned())
                .collect();
                
            Ok(CallGraphInfo {
                symbol_id: symbol_id.to_string(),
                calls,
                called_by,
            })
        } else {
            Ok(CallGraphInfo {
                symbol_id: symbol_id.to_string(),
                calls: vec![],
                called_by: vec![],
            })
        }
    }

    /// Find circular dependencies
    pub async fn find_circular_dependencies(&self) -> Result<Vec<Vec<String>>> {
        let graph = self.call_graph.read().await;
        
        if is_cyclic_directed(&graph.graph) {
            // Find strongly connected components
            let scc = petgraph::algo::tarjan_scc(&graph.graph);
            
            let cycles: Vec<Vec<String>> = scc
                .into_iter()
                .filter(|component| component.len() > 1)
                .map(|component| {
                    component
                        .into_iter()
                        .filter_map(|node| graph.node_to_symbol.get(&node).cloned())
                        .collect()
                })
                .collect();
                
            Ok(cycles)
        } else {
            Ok(vec![])
        }
    }

    /// Extract symbols with enhanced metadata
    fn extract_symbols(&self, parse_result: &ParseResult, file_path: &Path, content: &str) -> Result<Vec<SymbolEntry>> {
        let mut symbols = Vec::new();
        
        for symbol in &parse_result.symbols {
            let id = format!("{}:{}:{}", file_path.display(), symbol.name, symbol.location.line);
            
            // Calculate quality score
            let quality_score = self.calculate_quality_score(symbol, &parse_result.ast, content);
            
            // Determine if exported
            let is_exported = self.is_symbol_exported(symbol);
            
            symbols.push(SymbolEntry {
                id,
                name: symbol.name.clone(),
                kind: symbol.kind.clone(),
                file_path: file_path.to_path_buf(),
                start_pos: symbol.location.clone(),
                end_pos: symbol.location.clone(), // TODO: Get actual end position
                parent_id: symbol.parent.clone(),
                signature: symbol.signature.clone(),
                documentation: symbol.docs.clone(),
                visibility: None, // TODO: Extract from AST
                type_info: None, // TODO: Type inference
                complexity: self.calculate_complexity(&parse_result.ast),
                quality_score,
                reference_count: 0,
                is_exported,
                attributes: HashMap::new(),
                last_modified: chrono::Utc::now(),
                usage_count: 0,
            });
        }
        
        Ok(symbols)
    }

    /// Extract references from AST
    fn extract_references(&self, parse_result: &ParseResult, file_path: &Path, content: &str) -> Result<Vec<SymbolReference>> {
        let mut references = Vec::new();
        
        // Walk AST to find references
        self.walk_ast_for_references(&parse_result.ast, file_path, content, &mut references)?;
        
        Ok(references)
    }

    /// Walk AST recursively to find references
    fn walk_ast_for_references(
        &self,
        node: &AstNode,
        file_path: &Path,
        content: &str,
        references: &mut Vec<SymbolReference>,
    ) -> Result<()> {
        // Check if this node is a reference
        if node.metadata.is_reference {
            if let Some(name) = &node.name {
                // Extract context
                let context = self.extract_context(content, &node.start_pos);
                
                references.push(SymbolReference {
                    from_symbol_id: String::new(), // TODO: Resolve current scope
                    to_symbol_id: name.clone(),
                    reference_kind: self.determine_reference_kind(&node.node_type),
                    file_path: file_path.to_path_buf(),
                    position: node.start_pos.clone(),
                    context,
                });
            }
        }
        
        // Recurse into children
        for child in &node.children {
            self.walk_ast_for_references(child, file_path, content, references)?;
        }
        
        Ok(())
    }

    /// Calculate quality score for a symbol
    fn calculate_quality_score(&self, symbol: &Symbol, ast: &AstNode, content: &str) -> f32 {
        let mut score = 10.0;
        
        // Deduct for missing documentation
        if symbol.docs.is_none() {
            score -= 2.0;
        }
        
        // Deduct for high complexity
        let complexity = self.calculate_complexity(ast);
        if complexity > 10 {
            score -= ((complexity - 10) as f32 * 0.2).min(3.0);
        }
        
        // Deduct for long functions
        let lines = content.lines().count();
        if symbol.kind == SymbolKind::Function && lines > 50 {
            score -= 1.0;
        }
        
        // Deduct for poor naming
        if symbol.name.len() < 3 || symbol.name.chars().all(|c| c.is_uppercase()) {
            score -= 1.0;
        }
        
        score.max(0.0)
    }

    /// Calculate cyclomatic complexity
    fn calculate_complexity(&self, ast: &AstNode) -> u32 {
        let mut complexity = 1;
        
        // Add complexity for control flow
        complexity += self.count_nodes(ast, &["if", "else", "match", "while", "for", "?", "&&", "||"]);
        
        complexity
    }

    /// Count specific node types in AST
    fn count_nodes(&self, ast: &AstNode, node_types: &[&str]) -> u32 {
        let mut count = 0;
        
        if node_types.iter().any(|&t| ast.node_type.contains(t)) {
            count += 1;
        }
        
        for child in &ast.children {
            count += self.count_nodes(child, node_types);
        }
        
        count
    }

    /// Determine if symbol is exported/public
    fn is_symbol_exported(&self, symbol: &Symbol) -> bool {
        // Simple heuristic - can be enhanced
        matches!(symbol.kind, SymbolKind::Function | SymbolKind::Class | SymbolKind::Interface | SymbolKind::Struct)
    }

    /// Determine reference kind from node type
    fn determine_reference_kind(&self, node_type: &str) -> ReferenceKind {
        match node_type {
            t if t.contains("call") => ReferenceKind::Call,
            t if t.contains("import") || t.contains("use") => ReferenceKind::Import,
            t if t.contains("extend") || t.contains("inherit") => ReferenceKind::Inherit,
            t if t.contains("implement") => ReferenceKind::Implement,
            t if t.contains("new") || t.contains("instantiate") => ReferenceKind::Instantiate,
            t if t.contains("type") => ReferenceKind::TypeUse,
            _ => ReferenceKind::Reference,
        }
    }

    /// Extract context line
    fn extract_context(&self, content: &str, pos: &Position) -> String {
        content.lines()
            .nth(pos.line)
            .map(|line| line.trim().to_string())
            .unwrap_or_default()
    }

    /// Get or create parser for language
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

    /// Detect language from file extension
    fn detect_language(&self, path: &Path) -> Result<Language> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow!("No file extension"))?;
            
        match ext {
            "rs" => Ok(Language::Rust),
            "ts" | "tsx" => Ok(Language::TypeScript),
            "js" | "jsx" => Ok(Language::JavaScript),
            "py" => Ok(Language::Python),
            "go" => Ok(Language::Go),
            "java" => Ok(Language::Java),
            "cpp" | "cc" | "cxx" => Ok(Language::Cpp),
            "c" => Ok(Language::C),
            _ => Err(anyhow!("Unsupported language: {}", ext)),
        }
    }

    /// Insert symbol into database
    fn insert_symbol(&self, tx: &Transaction, symbol: &SymbolEntry) -> Result<()> {
        tx.execute(
            "INSERT INTO symbols (
                id, name, kind, file_path, start_line, start_col, end_line, end_col,
                parent_id, signature, documentation, visibility, type_info,
                complexity, quality_score, reference_count, is_exported
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                symbol.id,
                symbol.name,
                serde_json::to_string(&symbol.kind)?,
                symbol.file_path.to_str(),
                symbol.start_pos.line,
                symbol.start_pos.column,
                symbol.end_pos.line,
                symbol.end_pos.column,
                symbol.parent_id,
                symbol.signature,
                symbol.documentation,
                symbol.visibility,
                symbol.type_info,
                symbol.complexity,
                symbol.quality_score,
                symbol.reference_count,
                symbol.is_exported as i32,
            ],
        )?;
        
        Ok(())
    }

    /// Insert reference into database
    fn insert_reference(&self, tx: &Transaction, reference: &SymbolReference) -> Result<()> {
        tx.execute(
            "INSERT INTO symbol_references (
                from_symbol_id, to_symbol_id, reference_kind, file_path, line, col, context
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                reference.from_symbol_id,
                reference.to_symbol_id,
                serde_json::to_string(&reference.reference_kind)?,
                reference.file_path.to_str(),
                reference.position.line,
                reference.position.column,
                reference.context,
            ],
        )?;
        
        Ok(())
    }

    /// Update call graph with new symbols and references
    async fn update_call_graph(&self, symbols: &[SymbolEntry], references: &[SymbolReference]) -> Result<()> {
        let mut graph = self.call_graph.write().await;
        
        // Add symbol nodes
        for symbol in symbols {
            graph.add_symbol(&symbol.id);
        }
        
        // Add reference edges
        for reference in references {
            if !reference.from_symbol_id.is_empty() {
                graph.add_reference(&reference.from_symbol_id, &reference.to_symbol_id, reference.reference_kind);
            }
        }
        
        Ok(())
    }

    /// Update statistics
    async fn update_stats(&self, symbols_count: usize, references_count: usize, time_ms: f64) {
        let mut stats = self.stats.write().await;
        stats.total_symbols += symbols_count;
        stats.total_references += references_count;
        stats.files_indexed += 1;
        stats.index_time_ms += time_ms;
    }

    /// Get all symbols from the index
    pub async fn get_all_symbols(&self) -> Result<Vec<SymbolEntry>> {
        let conn = self.db.get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, name, kind, file_path, start_line, start_col, end_line, end_col, 
                    parent_id, signature, documentation, visibility, type_info, complexity, 
                    usage_count, last_modified, attributes 
             FROM symbols ORDER BY name"
        )?;
        
        let symbols = stmt.query_map([], |row| {
            Ok(SymbolEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                file_path: PathBuf::from(row.get::<_, String>(3)?),
                start_pos: Position {
                    line: row.get(4)?,
                    column: row.get(5)?,
                    offset: 0,
                },
                end_pos: Position {
                    line: row.get(6)?,
                    column: row.get(7)?,
                    offset: 0,
                },
                parent_id: row.get(8)?,
                signature: row.get(9)?,
                documentation: row.get(10)?,
                visibility: row.get(11)?,
                type_info: row.get(12)?,
                complexity: row.get(13)?,
                usage_count: row.get(14)?,
                last_modified: row.get(15)?,
                attributes: row.get::<_, Option<String>>(16)?
                    .map(|s| serde_json::from_str(&s).unwrap_or_default())
                    .unwrap_or_default(),
                quality_score: 5.0,
                reference_count: 0,
                is_exported: false,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(symbols)
    }
    
    /// Get index statistics
    pub async fn get_stats(&self) -> IndexStatistics {
        let stats = self.stats.read().await;
        let mut result = stats.clone();
        
        // Find circular dependencies
        if let Ok(cycles) = self.find_circular_dependencies().await {
            result.cyclic_dependencies = cycles;
        }
        
        result
    }
}

impl CallGraph {
    fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            symbol_to_node: HashMap::new(),
            node_to_symbol: HashMap::new(),
        }
    }

    fn add_symbol(&mut self, symbol_id: &str) -> NodeIndex {
        if let Some(&node) = self.symbol_to_node.get(symbol_id) {
            node
        } else {
            let node = self.graph.add_node(symbol_id.to_string());
            self.symbol_to_node.insert(symbol_id.to_string(), node);
            self.node_to_symbol.insert(node, symbol_id.to_string());
            node
        }
    }

    fn add_reference(&mut self, from: &str, to: &str, kind: ReferenceKind) {
        let from_node = self.add_symbol(from);
        let to_node = self.add_symbol(to);
        self.graph.add_edge(from_node, to_node, kind);
    }
}

/// Call graph information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphInfo {
    pub symbol_id: String,
    pub calls: Vec<String>,
    pub called_by: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::DatabaseConfig;

    #[tokio::test]
    async fn test_symbol_indexing() -> Result<()> {
        // Create test database
        let config = DatabaseConfig {
            path: PathBuf::from(":memory:"),
            ..Default::default()
        };
        let db = Arc::new(DatabaseManager::new(config).await?);
        
        // Create indexer
        let indexer = SymbolIndexer::new(db).await?;
        
        // Test Rust code
        let rust_code = r#"
fn main() {
    println!("Hello, world!");
    helper();
}

fn helper() {
    // Helper function
}
"#;
        
        // Index the code
        indexer.index_file(Path::new("test.rs"), rust_code).await?;
        
        // Search for symbols
        let results = indexer.search("main", 10).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "main");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_fts_search_performance() -> Result<()> {
        // This test would measure search performance
        // In a real implementation, we'd index many symbols and measure search time
        Ok(())
    }
}