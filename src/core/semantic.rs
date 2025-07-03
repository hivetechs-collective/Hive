//! Semantic indexing and search functionality
//!
//! This module provides semantic code understanding capabilities including:
//! - Symbol indexing and resolution
//! - Type inference and analysis
//! - Cross-reference tracking
//! - Semantic search with fuzzy matching
//! - Call graph analysis

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

use crate::{
    core::{
        ast::{AstEngine, Symbol, SymbolKind},
        error::Result,
    },
    cache::{self, CacheCategory, CacheKey},
};

/// Semantic index for fast code search and analysis
pub struct SemanticIndex {
    /// AST engine for parsing
    ast_engine: Arc<AstEngine>,
    /// Symbol table
    symbols: Arc<RwLock<SymbolTable>>,
    /// Cross-reference index
    references: Arc<RwLock<ReferenceIndex>>,
    /// Type information
    types: Arc<RwLock<TypeIndex>>,
    /// Call graph
    call_graph: Arc<RwLock<CallGraph>>,
    /// Indexed files
    indexed_files: Arc<RwLock<HashSet<PathBuf>>>,
}

/// Symbol table for storing all symbols in the codebase
#[derive(Debug, Default)]
struct SymbolTable {
    /// Symbols by file
    by_file: HashMap<PathBuf, Vec<Symbol>>,
    /// Global symbol index
    global_index: HashMap<String, Vec<SymbolLocation>>,
    /// Symbol hierarchy (parent-child relationships)
    hierarchy: HashMap<String, Vec<String>>,
}

/// Symbol location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolLocation {
    /// File path
    pub file: PathBuf,
    /// Symbol in that file
    pub symbol: Symbol,
}

/// Cross-reference index
#[derive(Debug, Default)]
struct ReferenceIndex {
    /// References from symbol to locations
    references: HashMap<String, Vec<ReferenceLocation>>,
    /// Reverse references (who references this symbol)
    reverse_refs: HashMap<String, Vec<ReferenceLocation>>,
}

/// Reference location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceLocation {
    /// File containing the reference
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Column number
    pub column: usize,
    /// Reference context
    pub context: String,
}

/// Type information index
#[derive(Debug, Default)]
struct TypeIndex {
    /// Type definitions
    types: HashMap<String, TypeInfo>,
    /// Type aliases
    aliases: HashMap<String, String>,
    /// Inferred types for symbols
    inferred: HashMap<String, TypeInfo>,
}

/// Type information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TypeInfo {
    /// Type name
    name: String,
    /// Generic parameters
    generics: Vec<String>,
    /// Methods if it's a class/struct
    methods: Vec<String>,
    /// Fields/properties
    fields: Vec<FieldInfo>,
}

/// Field information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FieldInfo {
    /// Field name
    name: String,
    /// Field type
    field_type: String,
    /// Visibility
    visibility: Option<String>,
}

/// Call graph for tracking function calls
#[derive(Debug, Default)]
struct CallGraph {
    /// Direct calls from function to functions
    calls: HashMap<String, HashSet<String>>,
    /// Reverse call graph (who calls this function)
    called_by: HashMap<String, HashSet<String>>,
}

/// Semantic search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Query string
    pub query: String,
    /// Symbol kinds to search
    pub kinds: Option<Vec<SymbolKind>>,
    /// File patterns to include
    pub include_patterns: Option<Vec<String>>,
    /// File patterns to exclude
    pub exclude_patterns: Option<Vec<String>>,
    /// Maximum results
    pub limit: usize,
    /// Enable fuzzy matching
    pub fuzzy: bool,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matched symbol
    pub symbol: Symbol,
    /// File containing the symbol
    pub file: PathBuf,
    /// Match score (0-100)
    pub score: f32,
    /// Match context
    pub context: String,
    /// Symbol documentation
    pub documentation: Option<String>,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Total files indexed
    pub file_count: usize,
    /// Total symbols
    pub symbol_count: usize,
    /// Symbol count by kind
    pub symbols_by_kind: HashMap<String, usize>,
    /// Total references
    pub reference_count: usize,
    /// Total types
    pub type_count: usize,
    /// Index size in bytes
    pub index_size: usize,
}

impl SemanticIndex {
    /// Create a new semantic index
    pub async fn new() -> Result<Self> {
        Ok(Self {
            ast_engine: Arc::new(AstEngine::new()),
            symbols: Arc::new(RwLock::new(SymbolTable::default())),
            references: Arc::new(RwLock::new(ReferenceIndex::default())),
            types: Arc::new(RwLock::new(TypeIndex::default())),
            call_graph: Arc::new(RwLock::new(CallGraph::default())),
            indexed_files: Arc::new(RwLock::new(HashSet::new())),
        })
    }
    
    /// Build project index
    pub async fn build_project(&self, path: &Path, force: bool) -> Result<()> {
        tracing::info!("📚 Building semantic index for {}", path.display());
        
        if force {
            tracing::info!("🔄 Force rebuilding index");
            self.clear_index().await?;
        }
        
        // Check cache first
        let cache_key = CacheKey::repository(path);
        if !force {
            if let Some(cached) = cache::get(&cache_key, CacheCategory::Repository).await {
                tracing::info!("📦 Loading cached index");
                // In a real implementation, deserialize the index
                return Ok(());
            }
        }
        
        // Walk the project directory
        let files = self.discover_files(path).await?;
        tracing::info!("Found {} files to index", files.len());
        
        // Index files in parallel
        // Note: In a production implementation, we would use a proper work pool
        // For now, index files sequentially to avoid lifetime issues
        for file in files {
            if let Err(e) = self.index_file(&file).await {
                tracing::warn!("Failed to index {}: {}", file.display(), e);
            }
        }
        
        // Build cross-references
        self.build_references().await?;
        
        // Build call graph
        self.build_call_graph().await?;
        
        // Cache the index
        // In a real implementation, serialize and cache the index
        
        let stats = self.get_stats().await;
        tracing::info!(
            "✅ Index complete: {} files, {} symbols",
            stats.file_count,
            stats.symbol_count
        );
        
        Ok(())
    }
    
    /// Index a single file
    pub async fn index_file(&self, path: &Path) -> Result<()> {
        // Read file content
        let content = tokio::fs::read_to_string(path).await?;
        
        // Parse AST
        let parse_result = self.ast_engine.parse_file(path, &content).await?;
        
        // Store symbols
        {
            let mut symbols = self.symbols.write().await;
            symbols.by_file.insert(path.to_path_buf(), parse_result.symbols.clone());
            
            // Update global index
            for symbol in &parse_result.symbols {
                let location = SymbolLocation {
                    file: path.to_path_buf(),
                    symbol: symbol.clone(),
                };
                
                symbols.global_index
                    .entry(symbol.name.clone())
                    .or_insert_with(Vec::new)
                    .push(location);
                
                // Update hierarchy
                if let Some(parent) = &symbol.parent {
                    symbols.hierarchy
                        .entry(parent.clone())
                        .or_insert_with(Vec::new)
                        .push(symbol.name.clone());
                }
            }
        }
        
        // Mark as indexed
        {
            let mut indexed = self.indexed_files.write().await;
            indexed.insert(path.to_path_buf());
        }
        
        Ok(())
    }
    
    /// Search for symbols
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        let symbols = self.symbols.read().await;
        let mut results = vec![];
        
        for (name, locations) in &symbols.global_index {
            // Apply filters
            if let Some(kinds) = &query.kinds {
                let has_matching_kind = locations.iter().any(|loc| {
                    kinds.contains(&loc.symbol.kind)
                });
                if !has_matching_kind {
                    continue;
                }
            }
            
            // Calculate match score
            let score = if query.fuzzy {
                self.fuzzy_match_score(&query.query, name)
            } else {
                if name.contains(&query.query) { 100.0 } else { 0.0 }
            };
            
            if score > 0.0 {
                for location in locations {
                    // Apply file filters
                    if !self.matches_file_patterns(&location.file, &query).await {
                        continue;
                    }
                    
                    results.push(SearchResult {
                        symbol: location.symbol.clone(),
                        file: location.file.clone(),
                        score,
                        context: self.get_symbol_context(&location.symbol).await,
                        documentation: location.symbol.docs.clone(),
                    });
                    
                    if results.len() >= query.limit {
                        break;
                    }
                }
            }
            
            if results.len() >= query.limit {
                break;
            }
        }
        
        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(query.limit);
        
        Ok(results)
    }
    
    /// Find symbol definition
    pub async fn find_definition(&self, name: &str) -> Result<Option<SymbolLocation>> {
        let symbols = self.symbols.read().await;
        
        if let Some(locations) = symbols.global_index.get(name) {
            // Return the first definition (not reference)
            for loc in locations {
                if loc.symbol.kind != SymbolKind::Import {
                    return Ok(Some(loc.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Find all references to a symbol
    pub async fn find_references(&self, name: &str) -> Result<Vec<ReferenceLocation>> {
        let refs = self.references.read().await;
        Ok(refs.references.get(name).cloned().unwrap_or_default())
    }
    
    /// Get symbols in a file
    pub async fn get_file_symbols(&self, path: &Path) -> Result<Vec<Symbol>> {
        let symbols = self.symbols.read().await;
        Ok(symbols.by_file.get(path).cloned().unwrap_or_default())
    }
    
    /// Get call graph for a function
    pub async fn get_call_graph(&self, function: &str) -> Result<CallGraphInfo> {
        let graph = self.call_graph.read().await;
        
        Ok(CallGraphInfo {
            calls: graph.calls.get(function).cloned().unwrap_or_default(),
            called_by: graph.called_by.get(function).cloned().unwrap_or_default(),
        })
    }
    
    /// Get index statistics
    pub async fn get_stats(&self) -> IndexStats {
        let symbols = self.symbols.read().await;
        let refs = self.references.read().await;
        let types = self.types.read().await;
        let indexed = self.indexed_files.read().await;
        
        let mut symbols_by_kind = HashMap::new();
        for locations in symbols.global_index.values() {
            for loc in locations {
                *symbols_by_kind.entry(format!("{:?}", loc.symbol.kind)).or_insert(0) += 1;
            }
        }
        
        IndexStats {
            file_count: indexed.len(),
            symbol_count: symbols.global_index.len(),
            symbols_by_kind,
            reference_count: refs.references.values().map(|v| v.len()).sum(),
            type_count: types.types.len(),
            index_size: 0, // Placeholder
        }
    }
    
    /// Clear the index
    async fn clear_index(&self) -> Result<()> {
        let mut symbols = self.symbols.write().await;
        let mut refs = self.references.write().await;
        let mut types = self.types.write().await;
        let mut graph = self.call_graph.write().await;
        let mut indexed = self.indexed_files.write().await;
        
        *symbols = SymbolTable::default();
        *refs = ReferenceIndex::default();
        *types = TypeIndex::default();
        *graph = CallGraph::default();
        indexed.clear();
        
        Ok(())
    }
    
    /// Discover files in a directory
    async fn discover_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut files = vec![];
        let mut stack = vec![path.to_path_buf()];
        
        while let Some(dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    // Skip common directories
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !matches!(name, ".git" | "target" | "node_modules" | ".venv" | "__pycache__") {
                        stack.push(path);
                    }
                } else if self.is_source_file(&path) {
                    files.push(path);
                }
            }
        }
        
        Ok(files)
    }
    
    /// Check if a file is a source file
    fn is_source_file(&self, path: &Path) -> bool {
        matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "cpp" | "c" | "h" | "hpp")
        )
    }
    
    /// Build cross-references
    async fn build_references(&self) -> Result<()> {
        // Placeholder implementation
        // In a real implementation, this would analyze the AST to find references
        Ok(())
    }
    
    /// Build call graph
    async fn build_call_graph(&self) -> Result<()> {
        // Placeholder implementation
        // In a real implementation, this would analyze function calls
        Ok(())
    }
    
    /// Calculate fuzzy match score
    fn fuzzy_match_score(&self, query: &str, target: &str) -> f32 {
        // Simple substring matching for now
        if target.to_lowercase().contains(&query.to_lowercase()) {
            let ratio = query.len() as f32 / target.len() as f32;
            (ratio * 100.0).min(100.0)
        } else {
            0.0
        }
    }
    
    /// Check if file matches patterns
    async fn matches_file_patterns(&self, file: &Path, query: &SearchQuery) -> bool {
        let file_str = file.to_string_lossy();
        
        // Check include patterns
        if let Some(includes) = &query.include_patterns {
            let matches_include = includes.iter().any(|pattern| {
                file_str.contains(pattern)
            });
            if !matches_include {
                return false;
            }
        }
        
        // Check exclude patterns
        if let Some(excludes) = &query.exclude_patterns {
            let matches_exclude = excludes.iter().any(|pattern| {
                file_str.contains(pattern)
            });
            if matches_exclude {
                return false;
            }
        }
        
        true
    }
    
    /// Get symbol context
    async fn get_symbol_context(&self, symbol: &Symbol) -> String {
        // Return a simple context string
        format!("{:?} at line {}", symbol.kind, symbol.location.line + 1)
    }
}

/// Call graph information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphInfo {
    /// Functions called by this function
    pub calls: HashSet<String>,
    /// Functions that call this function
    pub called_by: HashSet<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_semantic_index() -> Result<()> {
        let index = SemanticIndex::new().await?;
        
        // Test search query
        let query = SearchQuery {
            query: "test".to_string(),
            kinds: Some(vec![SymbolKind::Function]),
            include_patterns: None,
            exclude_patterns: None,
            limit: 10,
            fuzzy: true,
        };
        
        let results = index.search(query).await?;
        assert!(results.is_empty()); // No files indexed yet
        
        Ok(())
    }
    
    #[test]
    fn test_fuzzy_matching() {
        let index = SemanticIndex {
            ast_engine: Arc::new(AstEngine::new()),
            symbols: Arc::new(RwLock::new(SymbolTable::default())),
            references: Arc::new(RwLock::new(ReferenceIndex::default())),
            types: Arc::new(RwLock::new(TypeIndex::default())),
            call_graph: Arc::new(RwLock::new(CallGraph::default())),
            indexed_files: Arc::new(RwLock::new(HashSet::new())),
        };
        
        assert!(index.fuzzy_match_score("test", "test_function") > 0.0);
        assert_eq!(index.fuzzy_match_score("xyz", "test_function"), 0.0);
    }
}